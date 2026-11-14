//! Mesure de difficulté : combien de fois un joueur « imparfait » meurt-il dans
//! chaque niveau ? Une mesure REPRODUCTIBLE (graine fixe), à relancer avant et
//! après chaque changement de niveau, d'ennemi ou de mécanique (voir QA.md,
//! section « Mesure de difficulté »).
//!
//! Principe : « intention parfaite, exécution imparfaite ».
//! 1. Pour chaque niveau, on explore toutes les cases où Pépin peut se poser
//!    (mêmes plans que le robot de QA `bot_qa` : courir, sauter, planer,
//!    attendre) et on retient, pour chacune, le plan qui mène au plus vite au
//!    drapeau (`Itineraires`).
//! 2. Un joueur simulé joue dans la VRAIE simulation (`Monde`). Avant chaque
//!    plan, il l'essaie sur une copie du monde : il voit l'ennemi arriver et ne
//!    choisit qu'un plan sûr qui le rapproche du drapeau (sinon un court détour
//!    calculé sur place, sinon il attend un peu).
//! 3. Il EXÉCUTE ensuite ce plan avec, parfois, une erreur tirée au hasard selon
//!    son profil : élan trop court ou trop long (saut trop tôt / trop tard),
//!    touche de saut lâchée trop tôt ou trop tard, parapluie ouvert en retard,
//!    direction gardée ou lâchée à tort en redescendant.
//! 4. Une mort le renvoie à la dernière lanterne, comme dans le jeu. On compte
//!    les morts jusqu'au drapeau (`monde.etat.morts`, le compteur du jeu).
//!
//! Le hasard vient d'un petit générateur maison (`Hasard`, xorshift) : pas de
//! dépendance, et la même graine donne exactement les mêmes nombres partout.
//!
//! Notions Rust illustrées ici : générateur pseudo-aléatoire en quelques lignes
//! (décalages de bits, `wrapping_mul`), algorithme de Dijkstra avec
//! `BinaryHeap` et `Reverse` (le tas de Rust sort le PLUS GRAND d'abord),
//! `std::thread::scope` (des fils d'exécution qui empruntent des données
//! locales sans `Arc`), fermeture `&dyn Fn` passée à une fonction, `format!`
//! avec une largeur de colonne variable (`{c:<l$}`).

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fmt::Write as _;

use pepin_core::aabb::TAILLE_TUILE;
use pepin_core::entities::contacts::CauseMort;
use pepin_core::levels::{clocher, gouttieres, marche, nuages, orage, toits, vents};
use pepin_core::physics::JOUEUR_L;
use pepin_core::player::{Direction, Entrees};
use pepin_core::zones::{NB_NIVEAUX, NIVEAUX};

use crate::bot_qa::{
    chercher, cle, depart_pose, plans, plans_en_vol, simuler, CaseParcours, Fin, Plan, PAS_DEPART,
};
use crate::monde::Monde;

/// Graine de la mesure officielle. NE PAS la changer d'une carte à l'autre :
/// sinon les tableaux avant/après ne sont plus comparables.
pub(crate) const GRAINE: u64 = 20_261_113;
/// Essais par profil et par niveau (la moyenne des morts porte sur ces essais).
pub(crate) const ESSAIS: u32 = 48;
/// Durée maximale d'un essai : 6 minutes de jeu. Au-delà, l'essai est compté
/// « non fini » (ses morts comptent quand même dans la moyenne).
const PAS_MAX_ESSAI: u32 = 6 * 60 * 60;
/// Noeuds développés au plus pour un détour calculé sur place (borne le temps).
const LIMITE_RECHERCHE: usize = 12;
/// Idem dans l'arène du Grand Nuage (carte 5.14) : sous ses motifs serrés, la
/// sortie est souvent à plusieurs plans de là (filer entre deux salves).
const LIMITE_RECHERCHE_NUAGE: usize = 100;
/// Arène du Grand Nuage : plans suivis d'avance pour juger un candidat
/// (`Itineraires::chaine_sure`).
const PROFONDEUR_CHAINE: usize = 8;
/// Cases explorées au plus par niveau pour les itinéraires (garde-fou). Relevé de
/// 6 000 à 20 000 par la carte 5.12 : le marché couvert (quatre chats, dont l'état
/// compte dans la case, et une rue sous deux paires d'auvents) en explore plus que
/// les autres niveaux.
const LIMITE_CASES: usize = 20_000;
/// Voisinage des cases essayées quand Pépin se pose hors des cases connues :
/// ±2 cases de 8 px en x, ±2 px en y.
const VOISINAGE_X: i32 = 2;
const VOISINAGE_Y: i32 = 2;
/// Candidats essayés au plus avant le détour calculé.
const CANDIDATS_MAX: usize = 8;
/// Attente quand aucun chemin sûr n'est trouvé tout de suite (laisser passer
/// un ennemi, un éclair, une plateforme).
const ATTENTE_SECOURS: u16 = 20;
/// Arène du Grand Nuage (carte 5.14) : tranches de temps du nuage voisines
/// essayées autour de celle de la case (une tranche = 15 pas, voir
/// `bot_qa::nuage_de_la_case`).
const VOISINAGE_TEMPS: i32 = 1;

// --- Hasard reproductible ------------------------------------------------------

/// Générateur pseudo-aléatoire xorshift64* : rapide, assez bon pour un jeu ou une
/// mesure, et surtout DÉTERMINISTE (même graine = même suite de nombres).
#[derive(Clone, Debug)]
pub(crate) struct Hasard(u64);

impl Hasard {
    /// La graine est d'abord mélangée (splitmix64) : des graines voisines
    /// (1, 2, 3...) donnent des suites sans rapport entre elles.
    pub(crate) fn new(graine: u64) -> Self {
        let mut z = graine.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        // xorshift ne doit jamais partir de 0 (il y resterait).
        Self(if z == 0 { 1 } else { z })
    }

    pub(crate) fn suivant(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Entier entre `min` et `max` inclus.
    pub(crate) fn entre(&mut self, min: u16, max: u16) -> u16 {
        debug_assert!(min <= max);
        let n = u64::from(max - min) + 1;
        min + (self.suivant() % n) as u16
    }

    /// Vrai avec une probabilité de `pour_mille` / 1000.
    pub(crate) fn chance(&mut self, pour_mille: u16) -> bool {
        self.suivant() % 1000 < u64::from(pour_mille)
    }
}

// --- Profils de joueurs --------------------------------------------------------

/// Un profil de joueur : à quelle fréquence et de combien il se trompe.
/// Les durées sont en pas de 1/60 s (6 pas = 0,1 s).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Profil {
    pub(crate) nom: &'static str,
    /// Probabilité (sur 1000) qu'un plan soit mal exécuté.
    pub(crate) erreur_pour_mille: u16,
    /// Écart maximal de l'élan : le saut part jusqu'à ce nombre de pas trop tôt
    /// ou trop tard (ou Pépin s'arrête trop tôt / trop loin au bord du vide).
    pub(crate) ecart_elan: u16,
    /// Écart maximal du maintien de la touche de saut (saut trop court / trop long).
    pub(crate) ecart_saut: u16,
    /// Retard maximal à l'ouverture du parapluie, après le sommet du saut.
    pub(crate) retard_plane: u16,
}

/// Les trois profils de la mesure, du moins au plus habile. Valeurs choisies
/// pour qu'un débutant meure quelques fois sur le jeu et un expert presque
/// jamais ; elles ne doivent plus changer (sinon la mesure n'est plus comparable).
pub(crate) const PROFILS: [Profil; 3] = [
    Profil {
        nom: "Débutant",
        erreur_pour_mille: 400,
        ecart_elan: 12,
        ecart_saut: 8,
        retard_plane: 30,
    },
    Profil {
        nom: "Habitué",
        erreur_pour_mille: 200,
        ecart_elan: 8,
        ecart_saut: 5,
        retard_plane: 15,
    },
    Profil {
        nom: "Expert",
        erreur_pour_mille: 80,
        ecart_elan: 4,
        ecart_saut: 3,
        retard_plane: 6,
    },
];

/// Les gestes qu'un joueur peut rater.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Erreur {
    /// Saut parti trop tôt ou trop tard (ou arrêt trop tôt / trop loin du bord).
    Elan,
    /// Touche de saut lâchée trop tôt ou trop tard : saut trop bas ou trop haut.
    DureeSaut,
    /// Parapluie ouvert en retard.
    PlaneTardif,
    /// En redescendant, il garde la direction au lieu de la lâcher (il dépasse
    /// la plateforme visée) ou la lâche au lieu de la garder (il tombe court).
    PiloteEnLAir,
}

/// Applique (peut-être) une erreur d'exécution à `plan`. Ne change jamais la
/// direction de départ ni un plan « attendre » : le joueur sait OÙ aller, il
/// rate le geste.
pub(crate) fn perturber(plan: Plan, profil: &Profil, hasard: &mut Hasard) -> Plan {
    if plan.attente > 0
        || plan.dir == Direction::Immobile
        || !hasard.chance(profil.erreur_pour_mille)
    {
        return plan;
    }
    // Les erreurs possibles pour CE plan (pas de saut trop court sans saut...).
    let mut possibles = vec![Erreur::Elan, Erreur::PiloteEnLAir];
    if plan.saut > 0 {
        possibles.push(Erreur::DureeSaut);
    }
    if plan.planer {
        possibles.push(Erreur::PlaneTardif);
    }
    let erreur = possibles[hasard.entre(0, possibles.len() as u16 - 1) as usize];
    let mut p = plan;
    match erreur {
        Erreur::Elan => {
            let d = hasard.entre(1, profil.ecart_elan.max(1));
            p.elan = if hasard.chance(500) || p.elan == 0 {
                p.elan + d
            } else {
                p.elan.saturating_sub(d)
            };
        }
        Erreur::DureeSaut => {
            let d = hasard.entre(1, profil.ecart_saut.max(1));
            p.saut = if hasard.chance(500) {
                p.saut + d
            } else {
                // Au moins un pas : un saut reste un saut.
                p.saut.saturating_sub(d).max(1)
            };
        }
        Erreur::PlaneTardif => p.retard_plane = hasard.entre(1, profil.retard_plane.max(1)),
        Erreur::PiloteEnLAir => p.dir_descente = !p.dir_descente,
    }
    p
}

// --- Les itinéraires d'un niveau ------------------------------------------------

/// Une case du niveau où Pépin choisit son plan, posé ou lancé par un auvent
/// (voir `bot_qa::cle`).
type CaseNiveau = CaseParcours;

/// Ce qu'il faut faire depuis chaque case où Pépin peut se poser : le premier
/// plan d'un plus court chemin vers le drapeau, et la durée de ce chemin.
///
/// Calcul en deux passes, une fois par niveau :
/// 1. exploration vers l'avant (comme le robot, mais sans s'arrêter au
///    drapeau) : on note chaque « arête » case -> plan -> case (ou drapeau) ;
/// 2. passe à rebours depuis le drapeau (algorithme de Dijkstra sur les arêtes
///    retournées) : la distance de chaque case au drapeau, et le plan à jouer.
///
/// Toutes les boucles parcourent des `Vec` ou un tas trié, jamais une
/// `HashMap` : l'ordre d'itération d'une `HashMap` change d'une exécution à
/// l'autre, et la mesure ne serait plus reproductible.
pub(crate) struct Itineraires {
    indice: usize,
    /// Le niveau est-il l'arène du Grand Nuage (voir `explorer` et `intention`) ?
    nuage: bool,
    /// Les plans essayés (ceux du robot, grossiers ou précis : voir `calculer`).
    tous: Vec<Plan>,
    /// Case -> (pas restants jusqu'au drapeau, plan à jouer).
    conseils: HashMap<CaseNiveau, (u32, Plan)>,
    /// Cases explorées (pour le diagnostic).
    pub(crate) cases: usize,
}

impl Itineraires {
    /// Calcule les itinéraires du niveau `indice` (place dans la table) avec les plans
    /// grossiers du robot, puis avec ses plans précis si le drapeau reste hors
    /// d'atteinte (même repli que `bot_qa::tous_les_parcours`).
    pub(crate) fn calculer(indice: usize) -> Self {
        Self::explorer(indice, plans(false))
            .or_else(|| Self::explorer(indice, plans(true)))
            .unwrap_or_else(|| panic!("niveau {} : drapeau inatteignable", indice + 1))
    }

    /// Les deux passes avec les plans `tous`. `None` si le drapeau est
    /// inatteignable depuis le départ.
    fn explorer(indice: usize, tous: Vec<Plan>) -> Option<Self> {
        let (depart, _) = depart_pose(indice);
        let nuage = depart.pv_nuage().is_some();

        // 1. Vers l'avant. `meilleur` : coût et monde du meilleur arrivant connu
        //    de chaque case ; une case n'est développée qu'une fois, au plus tôt,
        //    puis son monde est libéré (`take`) pour garder la mémoire basse.
        let mut meilleur: HashMap<CaseNiveau, (u32, Option<Monde>)> = HashMap::new();
        let mut developpees: HashSet<CaseNiveau> = HashSet::new();
        // Arêtes : (case de départ, plan, case d'arrivée ou `None` = drapeau, pas).
        let mut aretes: Vec<(CaseNiveau, Plan, Option<CaseNiveau>, u32)> = Vec::new();
        let k0 = cle(&depart);
        meilleur.insert(k0, (0, Some(depart)));
        let en_vol = plans_en_vol();
        let mut tas = BinaryHeap::new();
        tas.push(Reverse((0u32, k0)));
        while let Some(Reverse((cout, k))) = tas.pop() {
            if developpees.contains(&k) || meilleur.get(&k).is_some_and(|(c, _)| *c < cout) {
                continue;
            }
            developpees.insert(k);
            if developpees.len() > LIMITE_CASES {
                break;
            }
            let Some(monde) = meilleur.get_mut(&k).and_then(|(_, m)| m.take()) else {
                continue;
            };
            // Case « lancée » (un auvent vient de relancer Pépin) : plans en vol.
            let a_essayer = if k.lance { &en_vol } else { &tous };
            for &plan in a_essayer {
                let (fin, apres, entrees) = simuler(&monde, plan);
                let duree = entrees.len() as u32;
                match fin {
                    Fin::Arrivee => aretes.push((k, plan, None, duree)),
                    Fin::Pose | Fin::Lance => {
                        let k2 = cle(&apres);
                        if k2 == k {
                            continue; // sur place : inutile
                        }
                        aretes.push((k, plan, Some(k2), duree));
                        let c2 = cout + duree;
                        if meilleur.get(&k2).is_none_or(|(c, _)| c2 < *c) {
                            meilleur.insert(k2, (c2, Some(apres)));
                            tas.push(Reverse((c2, k2)));
                        }
                    }
                    // Arène du Grand Nuage : une mort renvoie à la lanterne, le nuage
                    // reprend son grondement en gardant ses points perdus. Ces cases
                    // de reprise (une par nombre de points et de gouttes prises)
                    // n'arrivent jamais sans mourir : on les ajoute à l'exploration,
                    // SANS arête (mourir n'est pas un chemin conseillé), pour que le
                    // joueur sache quoi faire après une mort.
                    Fin::Mort if nuage => {
                        let reprise = reprendre_apres_une_mort(apres);
                        let k2 = cle(&reprise);
                        let c2 = cout + duree + PAS_DEPART as u32;
                        if meilleur.get(&k2).is_none_or(|(c, _)| c2 < *c) {
                            meilleur.insert(k2, (c2, Some(reprise)));
                            tas.push(Reverse((c2, k2)));
                        }
                    }
                    Fin::Mort | Fin::Abandon => {}
                }
            }
        }

        // 2. À rebours depuis le drapeau (`None`).
        let mut entrantes: HashMap<Option<CaseNiveau>, Vec<(CaseNiveau, Plan, u32)>> =
            HashMap::new();
        for &(de, plan, vers, duree) in &aretes {
            entrantes.entry(vers).or_default().push((de, plan, duree));
        }
        let mut conseils: HashMap<CaseNiveau, (u32, Plan)> = HashMap::new();
        let mut faites: HashSet<Option<CaseNiveau>> = HashSet::new();
        let mut tas = BinaryHeap::new();
        tas.push(Reverse((0u32, None::<CaseNiveau>)));
        while let Some(Reverse((d, v))) = tas.pop() {
            if !faites.insert(v) {
                continue;
            }
            for &(u, plan, duree) in entrantes.get(&v).map_or(&[][..], |l| &l[..]) {
                let du = d + duree;
                if conseils.get(&u).is_none_or(|(c, _)| du < *c) {
                    conseils.insert(u, (du, plan));
                    tas.push(Reverse((du, Some(u))));
                }
            }
        }
        if !conseils.contains_key(&k0) {
            return None;
        }
        Some(Self {
            indice,
            nuage,
            tous,
            conseils,
            cases: developpees.len(),
        })
    }

    /// Pas restants jusqu'au drapeau depuis la case `k`, si elle est connue.
    fn distance(&self, k: CaseNiveau) -> Option<u32> {
        self.conseils.get(&k).map(|&(d, _)| d)
    }

    /// Les plans candidats depuis `monde` (posé) : le conseil de sa case, puis ceux
    /// des cases voisines (une erreur d'élan pose Pépin un peu à côté des cases
    /// explorées), au plus `CANDIDATS_MAX` cases, sans doublon ; et la distance de
    /// sa case (`None` si elle est inconnue), et le seuil qu'un plan doit passer
    /// pour le rapprocher de la fin.
    fn candidats(&self, monde: &Monde) -> (Option<u32>, Option<u32>, Vec<Plan>) {
        let case = cle(monde);
        let ici = self.distance(case);
        let mut voisines: Vec<(u32, i32, Plan)> = Vec::new();
        // Arène du Grand Nuage : aussi les tranches de temps voisines (une erreur
        // d'élan fait arriver Pépin quelques pas plus tôt ou plus tard).
        let temps = if self.nuage { VOISINAGE_TEMPS } else { 0 };
        for dt in -temps..=temps {
            let nuage = case.nuage + dt;
            // Même phase, mêmes points, mêmes gouttes : seule la tranche change.
            if (nuage - 1).div_euclid(64) != (case.nuage - 1).div_euclid(64) {
                continue;
            }
            for dy in -VOISINAGE_Y..=VOISINAGE_Y {
                for dx in -VOISINAGE_X..=VOISINAGE_X {
                    // Une case voisine du même genre (posée, ou lancée par un auvent).
                    let voisine = CaseNiveau {
                        x: case.x + dx,
                        y: case.y + dy,
                        nuage,
                        ..case
                    };
                    if let Some(&(d, plan)) = self.conseils.get(&voisine) {
                        voisines.push((d, dx.abs() + dy.abs() + dt.abs(), plan));
                    }
                }
            }
        }
        // La case elle-même d'abord, puis les plus proches, puis les plus avancées.
        voisines.sort_by_key(|&(d, ecart, _)| (ecart, d));
        let seuil = ici.or(voisines.iter().map(|v| v.0).max());
        let mut plans: Vec<Plan> = Vec::new();
        for &(_, _, plan) in voisines.iter().take(CANDIDATS_MAX) {
            if !plans.contains(&plan) {
                plans.push(plan);
            }
        }
        (ici, seuil, plans)
    }

    /// Le monde `m` est-il plus près de la fin que `seuil` ?
    fn progresse(&self, m: &Monde, seuil: Option<u32>) -> bool {
        self.distance(cle(m))
            .is_some_and(|d| seuil.is_none_or(|s| d < s))
    }

    /// Ce que le joueur VEUT faire depuis `monde` (posé), et pourquoi.
    ///
    /// Candidats : voir `candidats`. Chaque candidat est essayé sur une COPIE du
    /// monde (le joueur voit l'ennemi arriver) ; on garde le premier qui rapproche
    /// sans risque du drapeau. Sinon : court détour calculé vers une case plus
    /// proche du drapeau, sinon attendre.
    ///
    /// Arène du Grand Nuage (carte 5.14) : la case ne connaît le nuage qu'à 15 pas
    /// près, et le conseil d'une case, calculé pour l'heure exacte où l'exploration
    /// y est passée, peut mener sous un éclair à une autre heure de la même
    /// tranche. Le joueur, qui VOIT les colonnes annoncées, ne garde donc un
    /// candidat que si la suite des conseils, à partir de là, ne le fait pas
    /// mourir (`chaine_sure`) ; et s'il ne sait pas quoi faire, il ne reste pas
    /// sous une colonne qui va frapper (`secours_sous_les_eclairs`).
    fn intention(&self, monde: &Monde) -> (Plan, Choix) {
        let (ici, seuil, candidats) = self.candidats(monde);
        let progresse = |m: &Monde| self.progresse(m, seuil);
        for (k, &plan) in candidats.iter().enumerate() {
            let (fin, apres, _) = simuler(monde, plan);
            let ok = match fin {
                Fin::Arrivee => true,
                Fin::Pose | Fin::Lance => {
                    progresse(&apres) && (!self.nuage || self.chaine_sure(&apres))
                }
                Fin::Mort | Fin::Abandon => false,
            };
            if ok {
                let choix = if ici.is_some() && k == 0 {
                    Choix::Parcours
                } else {
                    Choix::Voisin
                };
                return (plan, choix);
            }
        }
        let limite = if self.nuage {
            LIMITE_RECHERCHE_NUAGE
        } else {
            LIMITE_RECHERCHE
        };
        let cible = |m: &Monde| progresse(m) && (!self.nuage || self.chaine_sure(m));
        if let Some(etapes) = chercher(monde, &self.tous, &cible, limite) {
            return (etapes[0].plan, Choix::Detour);
        }
        let attendre = Plan {
            dir: Direction::Immobile,
            elan: 0,
            saut: 0,
            planer: false,
            retard_plane: 0,
            dir_montee: false,
            dir_descente: false,
            attente: ATTENTE_SECOURS,
        };
        if self.nuage {
            return (
                self.secours_sous_les_eclairs(monde, attendre),
                Choix::Secours,
            );
        }
        (attendre, Choix::Secours)
    }

    /// Arène du Grand Nuage : en suivant les conseils depuis `depart` (le premier
    /// candidat qui rapproche de la fin, sans autre vérification), Pépin
    /// reste-t-il en vie pendant `PROFONDEUR_CHAINE` plans (ou jusqu'à la fin) ?
    fn chaine_sure(&self, depart: &Monde) -> bool {
        let mut m = depart.clone();
        for _ in 0..PROFONDEUR_CHAINE {
            let (_, seuil, candidats) = self.candidats(&m);
            let suivant = candidats.iter().find_map(|&plan| {
                let (fin, apres, _) = simuler(&m, plan);
                match fin {
                    Fin::Arrivee => Some(None),
                    Fin::Pose | Fin::Lance if self.progresse(&apres, seuil) => Some(Some(apres)),
                    _ => None,
                }
            });
            match suivant {
                None => return false,
                Some(None) => return true,
                Some(Some(apres)) => m = apres,
            }
        }
        true
    }

    /// Arène du Grand Nuage : le joueur VOIT les éclairs annoncés. S'il ne sait pas
    /// où aller, il attend sur place si cela le laisse en vie ET libre d'en
    /// repartir (pas coincé sous une colonne qui va frapper) ; sinon il prend le
    /// plan qui le laisse ainsi, de préférence vers la case connue la plus proche
    /// de la fin ; s'il n'y en a aucun, il attend quand même.
    fn secours_sous_les_eclairs(&self, monde: &Monde, attendre: Plan) -> Plan {
        let tient = |plan: Plan| {
            let (fin, apres, _) = simuler(monde, plan);
            (!matches!(fin, Fin::Mort | Fin::Abandon) && self.survit_ensuite(&apres))
                .then_some(apres)
        };
        if tient(attendre).is_some() {
            return attendre;
        }
        let mut meilleur: Option<(u32, Plan)> = None;
        let mut premier_sur: Option<Plan> = None;
        for &plan in &self.tous {
            let Some(apres) = tient(plan) else { continue };
            premier_sur.get_or_insert(plan);
            if let Some(d) = self.distance(cle(&apres)) {
                if meilleur.is_none_or(|(m, _)| d < m) {
                    meilleur = Some((d, plan));
                }
            }
        }
        // Coincé : au moins un plan qui le laisse en vie tout de suite.
        let vivant = || {
            self.tous
                .iter()
                .copied()
                .find(|&p| !matches!(simuler(monde, p).0, Fin::Mort | Fin::Abandon))
        };
        meilleur
            .map(|(_, p)| p)
            .or(premier_sur)
            .or_else(vivant)
            .unwrap_or(attendre)
    }

    /// Depuis `monde` (Pépin posé), au moins un plan le laisse-t-il en vie ?
    fn survit_ensuite(&self, monde: &Monde) -> bool {
        self.tous
            .iter()
            .any(|&p| !matches!(simuler(monde, p).0, Fin::Mort | Fin::Abandon))
    }
}

/// Comme dans le jeu (et dans `jouer_essai`) après une mort : l'animation de
/// réapparition à la lanterne, puis Pépin retombe au sol.
fn reprendre_apres_une_mort(mut monde: Monde) -> Monde {
    for _ in 0..PAS_DEPART {
        monde.avancer(&Entrees::default());
    }
    monde.derniere_mort = None;
    monde
}

/// D'où vient l'intention du joueur (pour le diagnostic de la mesure).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Choix {
    /// Le conseil de sa propre case.
    Parcours,
    /// Le conseil d'une case voisine (Pépin s'est posé un peu à côté).
    Voisin,
    /// Un court détour calculé sur place.
    Detour,
    /// Rien de sûr trouvé : attendre un peu.
    Secours,
}

// --- Un essai ------------------------------------------------------------------

/// Où et de quoi Pépin est mort : colonne de tuile (centre de Pépin au moment de
/// la mort ; pour une chute, la colonne du trou) et cause.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LieuMort {
    pub(crate) colonne: i32,
    pub(crate) cause: CauseMort,
}

impl LieuMort {
    /// Le lieu de la mort qui vient d'avoir lieu dans `monde`.
    fn de(monde: &Monde) -> Self {
        let centre = monde.pos_mort.0 + JOUEUR_L / 2.0;
        LieuMort {
            colonne: (centre / TAILLE_TUILE).floor() as i32,
            // Une mort est toujours notée avec sa cause ; « vide » par prudence.
            cause: monde.derniere_mort.unwrap_or(CauseMort::Vide),
        }
    }
}

/// Nom court d'une cause de mort, pour le tableau.
fn nom_cause(c: CauseMort) -> &'static str {
    match c {
        CauseMort::Vide => "chute",
        CauseMort::Ennemi => "ennemi",
        CauseMort::Pointe => "pointes",
        CauseMort::Eclair => "éclair",
        CauseMort::Grele => "grêle",
    }
}

/// Résultat d'un essai. Pas `Copy` : il contient la liste des lieux de mort
/// (un `Vec` possède de la mémoire sur le tas, il ne se copie pas bit à bit).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Essai {
    pub(crate) morts: u32,
    /// Lieu de chaque mort, dans l'ordre.
    pub(crate) lieux: Vec<LieuMort>,
    /// Durée de jeu simulée, en pas.
    pub(crate) pas: u32,
    /// Le drapeau a-t-il été atteint avant `PAS_MAX_ESSAI` ?
    pub(crate) fini: bool,
    /// Morts « en série » : moins de `DELAI_SERIE` pas après la précédente, donc
    /// sans que le joueur ait pu réagir (Pépin réapparaît sur un danger).
    pub(crate) en_serie: u32,
    /// Décisions prises hors des cases explorées (détours calculés) et attentes.
    pub(crate) detours: u32,
    pub(crate) secours: u32,
}

/// Une mort à moins d'une seconde de la précédente est « en série ».
const DELAI_SERIE: u32 = 60;

/// Note une mort survenue au pas `pas`.
fn noter_mort(pas: u32, derniere: &mut Option<u32>, en_serie: &mut u32) {
    if derniere.is_some_and(|d| pas - d < DELAI_SERIE) {
        *en_serie += 1;
    }
    *derniere = Some(pas);
}

/// Joue un essai du niveau d'`itineraires` avec ce profil et cette graine.
pub(crate) fn jouer_essai(itineraires: &Itineraires, profil: &Profil, graine: u64) -> Essai {
    jouer_essai_note(itineraires, profil, graine, &mut Vec::new())
}

/// Comme `jouer_essai`, en notant dans `journal` les entrées de chaque pas
/// depuis le début du niveau (pour rejouer l'essai dans le navigateur).
pub(crate) fn jouer_essai_note(
    itineraires: &Itineraires,
    profil: &Profil,
    graine: u64,
    journal: &mut Vec<Entrees>,
) -> Essai {
    let mut hasard = Hasard::new(graine);
    let (mut monde, prefixe) = depart_pose(itineraires.indice);
    let mut pas = prefixe.len() as u32;
    journal.extend(prefixe);
    let (mut detours, mut secours, mut en_serie) = (0, 0, 0);
    let mut derniere_mort: Option<u32> = None;
    let mut lieux = Vec::new();
    while pas < PAS_MAX_ESSAI && !monde.termine {
        let (voulu, choix) = itineraires.intention(&monde);
        match choix {
            Choix::Parcours | Choix::Voisin => {}
            Choix::Detour => detours += 1,
            Choix::Secours => secours += 1,
        }
        let joue = perturber(voulu, profil, &mut hasard);
        let (fin, apres, entrees) = simuler(&monde, joue);
        monde = apres;
        pas += entrees.len() as u32;
        journal.extend(entrees);
        if fin == Fin::Mort {
            noter_mort(pas, &mut derniere_mort, &mut en_serie);
            lieux.push(LieuMort::de(&monde));
            // Comme dans le jeu : animation de réapparition à la lanterne, puis
            // Pépin retombe au sol. Le monde continue d'avancer (et peut tuer
            // encore : un ennemi qui passe sur la lanterne, par exemple).
            for _ in 0..PAS_DEPART {
                let morts = monde.etat.morts;
                monde.avancer(&Entrees::default());
                journal.push(Entrees::default());
                pas += 1;
                if monde.etat.morts > morts {
                    noter_mort(pas, &mut derniere_mort, &mut en_serie);
                    lieux.push(LieuMort::de(&monde));
                }
            }
            // `derniere_mort` ne s'efface jamais seule (elle sert aux effets) :
            // sans cette remise à zéro, `simuler` croirait Pépin encore mort.
            monde.derniere_mort = None;
        }
    }
    Essai {
        morts: monde.etat.morts,
        lieux,
        pas,
        fini: monde.termine,
        en_serie,
        detours,
        secours,
    }
}

/// Graine d'un essai : dépend de la graine de la mesure, du profil, du niveau et
/// du numéro d'essai, pas de l'ordre dans lequel les fils d'exécution tournent.
/// `niveau` est le RANG DE GRAINE du niveau (`rang_de_graine`), pas sa place dans
/// la table : insérer un niveau ne change pas la mesure des autres.
fn graine_essai(graine: u64, profil: usize, niveau: usize, essai: u32) -> u64 {
    graine
        .wrapping_mul(1_000_003)
        .wrapping_add((profil as u64) << 40)
        .wrapping_add((niveau as u64) << 32)
        .wrapping_add(u64::from(essai))
}

/// Les niveaux du jeu dans l'ordre de leur ARRIVÉE dans le jeu (pas dans l'ordre
/// du parcours) : la place d'un niveau dans cette liste est son « rang de graine ».
/// Avant la carte 5.12, la graine d'un niveau dépendait de sa place dans la table :
/// insérer le marché couvert en 3e place aurait changé les tirages (donc les
/// chiffres) des trois niveaux suivants, sans qu'aucun ne change. Les cinq premiers
/// rangs sont leurs anciennes places : leur mesure reste identique à l'octet. Un
/// nouveau niveau s'ajoute À LA FIN (le test `chaque_niveau_a_un_rang_de_graine`
/// échoue sinon).
const RANGS_DE_GRAINE: &[&str] = &[
    toits::INFOS.nom,
    gouttieres::INFOS.nom,
    orage::INFOS.nom,
    vents::INFOS.nom,
    nuages::INFOS.nom,
    marche::INFOS.nom,
    clocher::INFOS.nom,
];

/// Rang de graine du niveau d'indice `n` de la table (voir `RANGS_DE_GRAINE`).
fn rang_de_graine(n: usize) -> usize {
    let nom = NIVEAUX[n].nom;
    RANGS_DE_GRAINE
        .iter()
        .position(|&r| r == nom)
        .unwrap_or_else(|| panic!("« {nom} » n'a pas de rang de graine (RANGS_DE_GRAINE)"))
}

// --- La mesure complète -------------------------------------------------------

/// Bilan d'une case du tableau (un profil sur un niveau).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Case {
    pub(crate) morts_moyennes: f32,
    /// Dont morts en série (voir `Essai::en_serie`), en moyenne.
    pub(crate) en_serie_moyennes: f32,
    /// Le pire essai.
    pub(crate) morts_max: u32,
    pub(crate) finis: u32,
    pub(crate) essais: u32,
    /// Durée moyenne d'un essai, en secondes de jeu.
    pub(crate) duree_s: f32,
    /// Lieux de toutes les morts de tous les essais.
    pub(crate) lieux: Vec<LieuMort>,
}

/// Mesure complète : `cases[niveau][profil]`.
pub(crate) fn mesurer(graine: u64, essais: u32) -> Vec<Vec<Case>> {
    // Un fil d'exécution par niveau pour les références, puis un par
    // (niveau, profil) pour les essais. `scope` attend la fin de tous les fils
    // avant de rendre la main : ils peuvent donc emprunter `itineraires`.
    let itineraires: Vec<Itineraires> = std::thread::scope(|s| {
        let fils: Vec<_> = (0..NB_NIVEAUX)
            .map(|i| s.spawn(move || Itineraires::calculer(i)))
            .collect();
        fils.into_iter()
            .map(|f| f.join().expect("calcul des itinéraires"))
            .collect()
    });
    std::thread::scope(|s| {
        let fils: Vec<Vec<_>> = itineraires
            .iter()
            .enumerate()
            .map(|(n, itineraires_niveau)| {
                PROFILS
                    .iter()
                    .enumerate()
                    .map(|(p, profil)| {
                        s.spawn(move || {
                            let liste: Vec<Essai> = (0..essais)
                                .map(|e| {
                                    jouer_essai(
                                        itineraires_niveau,
                                        profil,
                                        graine_essai(graine, p, rang_de_graine(n), e),
                                    )
                                })
                                .collect();
                            bilan(&liste)
                        })
                    })
                    .collect()
            })
            .collect();
        fils.into_iter()
            .map(|ligne| {
                ligne
                    .into_iter()
                    .map(|f| f.join().expect("essais"))
                    .collect()
            })
            .collect()
    })
}

fn bilan(liste: &[Essai]) -> Case {
    let n = liste.len().max(1) as f32;
    Case {
        morts_moyennes: liste.iter().map(|e| e.morts as f32).sum::<f32>() / n,
        en_serie_moyennes: liste.iter().map(|e| e.en_serie as f32).sum::<f32>() / n,
        morts_max: liste.iter().map(|e| e.morts).max().unwrap_or(0),
        finis: liste.iter().filter(|e| e.fini).count() as u32,
        essais: liste.len() as u32,
        duree_s: liste.iter().map(|e| e.pas as f32).sum::<f32>() / n / 60.0,
        lieux: liste.iter().flat_map(|e| e.lieux.iter().copied()).collect(),
    }
}

/// Un endroit où l'on meurt : colonnes `debut..=fin` (voisines), une cause, un
/// nombre de morts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Endroit {
    pub(crate) debut: i32,
    pub(crate) fin: i32,
    pub(crate) cause: CauseMort,
    pub(crate) morts: u32,
}

/// Regroupe des lieux de mort : même cause et colonnes distantes d'au plus
/// 2 tuiles (un même trou, un même ennemi qui patrouille). Trié du plus
/// meurtrier au moins meurtrier (à égalité : de gauche à droite).
pub(crate) fn regrouper(lieux: &[LieuMort]) -> Vec<Endroit> {
    let mut tries: Vec<LieuMort> = lieux.to_vec();
    // `sort_by_key` avec une clé (cause, colonne) : les causes ne sont pas
    // ordonnées (pas d'`Ord`), on passe par leur nom.
    tries.sort_by_key(|l| (nom_cause(l.cause), l.colonne));
    let mut endroits: Vec<Endroit> = Vec::new();
    for l in tries {
        match endroits.last_mut() {
            Some(e) if e.cause == l.cause && l.colonne - e.fin <= 2 => {
                e.fin = l.colonne;
                e.morts += 1;
            }
            _ => endroits.push(Endroit {
                debut: l.colonne,
                fin: l.colonne,
                cause: l.cause,
                morts: 1,
            }),
        }
    }
    endroits.sort_by_key(|e| (Reverse(e.morts), e.debut));
    endroits
}

/// « col. 60-63 chute 12 » : un endroit, pour le tableau.
fn decrire(e: &Endroit) -> String {
    let colonnes = if e.debut == e.fin {
        format!("{}", e.debut)
    } else {
        format!("{}-{}", e.debut, e.fin)
    };
    format!("col. {colonnes} {} {}", nom_cause(e.cause), e.morts)
}

/// Nombre à une décimale, virgule à la française : 3.25 -> « 3,2 ».
fn decimal(x: f32) -> String {
    format!("{x:.1}").replace('.', ",")
}

/// Écrit une ligne de tableau : `| a | b | c |`, colonnes de largeur fixe.
fn ligne(t: &mut String, cellules: &[String], largeurs: &[usize]) {
    t.push('|');
    for (c, &l) in cellules.iter().zip(largeurs) {
        // `{:<l$}` : aligné à gauche sur `l` caractères (largeur variable).
        let _ = write!(t, " {c:<l$} |");
    }
    t.push('\n');
}

/// Les deux tableaux à coller dans QA.md (texte aligné, lisible aussi en Markdown).
pub(crate) fn tableau(cases: &[Vec<Case>], graine: u64, essais: u32) -> String {
    let mut t = String::new();
    let profils: Vec<String> = PROFILS.iter().map(|p| p.nom.to_string()).collect();
    let noms: Vec<String> = NIVEAUX
        .iter()
        .map(|n| format!("{}. {}", n.numero, n.nom))
        .collect();

    // 1. Morts moyennes : LE chiffre à comparer avant/après.
    let _ = writeln!(
        t,
        "Morts moyennes par niveau (graine {graine}, {essais} essais par profil et par \
         niveau) ; entre parenthèses : sans les morts en série (moins d'1 s après la \
         précédente, voir le détail)."
    );
    let _ = writeln!(t);
    let largeurs = [27, 12, 12, 12];
    let mut entete = vec!["Niveau".to_string()];
    entete.extend(profils.iter().cloned());
    ligne(&mut t, &entete, &largeurs);
    let traits: Vec<String> = largeurs.iter().map(|&l| "-".repeat(l)).collect();
    ligne(&mut t, &traits, &largeurs);
    let mut totaux = vec![0.0f32; PROFILS.len()];
    let mut hors_serie = vec![0.0f32; PROFILS.len()];
    for (n, rang) in cases.iter().enumerate() {
        let mut cellules = vec![noms[n].clone()];
        for (p, c) in rang.iter().enumerate() {
            totaux[p] += c.morts_moyennes;
            hors_serie[p] += c.morts_moyennes - c.en_serie_moyennes;
            cellules.push(format!(
                "{} ({})",
                decimal(c.morts_moyennes),
                decimal(c.morts_moyennes - c.en_serie_moyennes)
            ));
        }
        ligne(&mut t, &cellules, &largeurs);
    }
    let mut cellules = vec!["Total".to_string()];
    cellules.extend(
        totaux
            .iter()
            .zip(&hors_serie)
            .map(|(&x, &h)| format!("{} ({})", decimal(x), decimal(h))),
    );
    ligne(&mut t, &cellules, &largeurs);

    // 2. Détail : essais finis, durée moyenne (s), morts en série, pire essai.
    let _ = writeln!(t);
    let _ = writeln!(
        t,
        "Détail par case : essais finis / essais, durée moyenne en secondes, \
         morts en série en moyenne (moins d'1 s après la précédente), pire essai."
    );
    let _ = writeln!(t);
    let largeurs = [27, 30, 30, 30];
    ligne(&mut t, &entete, &largeurs);
    let traits: Vec<String> = largeurs.iter().map(|&l| "-".repeat(l)).collect();
    ligne(&mut t, &traits, &largeurs);
    for (n, rang) in cases.iter().enumerate() {
        let mut cellules = vec![noms[n].clone()];
        for c in rang {
            cellules.push(format!(
                "{}/{}, {} s, série {}, max {}",
                c.finis,
                c.essais,
                c.duree_s.round(),
                decimal(c.en_serie_moyennes),
                c.morts_max
            ));
        }
        ligne(&mut t, &cellules, &largeurs);
    }

    // 3. Lieux de mort : où agir pour adoucir (ou corser) un niveau.
    let _ = writeln!(t);
    let _ = writeln!(
        t,
        "Lieux de mort : colonne de tuile où Pépin meurt, cause, morts cumulées sur les \
         {essais} essais (colonnes voisines de même cause regroupées ; {LIEUX_MAX} endroits au \
         plus, du plus meurtrier au moins meurtrier)."
    );
    for (n, rang) in cases.iter().enumerate() {
        let _ = writeln!(t);
        let _ = writeln!(t, "{}", noms[n]);
        for (p, c) in rang.iter().enumerate() {
            let endroits = regrouper(&c.lieux);
            let texte = if endroits.is_empty() {
                "aucune mort".to_string()
            } else {
                let mut morceaux: Vec<String> =
                    endroits.iter().take(LIEUX_MAX).map(decrire).collect();
                let reste: u32 = endroits.iter().skip(LIEUX_MAX).map(|e| e.morts).sum();
                if reste > 0 {
                    morceaux.push(format!("ailleurs {reste}"));
                }
                morceaux.join(" · ")
            };
            let _ = writeln!(t, "- {:<8} : {texte}", profils[p]);
        }
    }
    t
}

/// Endroits affichés au plus par niveau et par profil dans les lieux de mort.
const LIEUX_MAX: usize = 6;

// --- Forme de la courbe (non-régression, carte 3.3) ----------------------------

/// Rangs des profils dans `PROFILS`.
const DEBUTANT: usize = 0;
const HABITUE: usize = 1;

/// Bande admise pour les morts moyennes du profil Habitué (le « joueur
/// moyen »), niveau par niveau : `[min, max[` (le max est exclu). Les niveaux 1
/// et 2 n'ont pas été retouchés par la carte 3.3 : leur bande entoure la
/// mesure de référence (0,1 et 0,5).
const BANDES_HABITUE: [(f32, f32); NB_NIVEAUX] = [
    (0.0, 0.3),  // 1. Les toits : quasi inchangé
    (0.3, 0.8),  // 2. Gouttières et cheminées : quasi inchangé
    (0.3, 1.2),  // 3. Le marché couvert (carte 5.12) : entre les gouttières et l'orage
    (0.5, 1.5),  // 4. Le grand orage : le sommet de la courbe
    (0.0, 0.5),  // 5. Le clocher (carte 5.13) : tomber n'y tue qu'en haut, sous 0,5
    (0.3, 0.65), // 6. Vents d'altitude : redescend un peu
    (0.0, 0.2),  // 7. Au-dessus des nuages : final calme, sous 0,2
];

/// Places dans la table des trois niveaux dont la courbe juge la forme : le
/// SOMMET (le grand orage), le niveau qui le SUIT (vents d'altitude, nettement
/// plus facile) et le FINAL (au-dessus des nuages, le plus calme). Le test
/// `la_forme_de_la_courbe_est_jugee` vérifie qu'ils désignent bien ces niveaux :
/// insérer un niveau avant eux oblige à les déplacer.
///
/// Le clocher (carte 5.13), inséré entre le sommet et les vents, n'est jugé que
/// par sa bande (et, comme tous, sous le sommet) : niveau vertical où une chute
/// ne coûte en bas que de la hauteur, il se mesure plus facile que l'orage. La
/// courbe sur onze niveaux (monter jusqu'au boss) se jugera à la carte 5.18.
const SOMMET: usize = 3;
const APRES_SOMMET: usize = 5;
const FINAL: usize = 6;

/// Écart minimal entre le sommet et le niveau suivant (Habitué) : celui-ci doit
/// être NETTEMENT plus facile que le sommet, pas à égalité au hasard près.
const ECART_APRES_SOMMET: f32 = 0.15;

/// Vérifie la FORME de la courbe sur une mesure complète `cases[niveau][profil]`
/// et renvoie la liste des écarts (vide si tout va bien) :
/// - Débutant et Habitué : le SOMMET (le grand orage) est le plus meurtrier de
///   tous, et le niveau qui le suit l'est plus que le FINAL ;
/// - Habitué : chaque niveau dans sa bande (`BANDES_HABITUE`), et le niveau qui
///   suit le sommet au moins `ECART_APRES_SOMMET` sous lui.
///
/// Renvoyer une liste plutôt que s'arrêter au premier écart : en cas d'échec,
/// on voit tout ce qui ne va pas d'un coup.
pub(crate) fn ecarts_de_la_courbe(cases: &[Vec<Case>]) -> Vec<String> {
    let mut ecarts = Vec::new();
    let morts = |n: usize, p: usize| cases[n][p].morts_moyennes;
    for p in [DEBUTANT, HABITUE] {
        let nom = PROFILS[p].nom;
        for n in (0..NB_NIVEAUX).filter(|&n| n != SOMMET) {
            if morts(n, p) >= morts(SOMMET, p) {
                ecarts.push(format!(
                    "{nom} : niveau {} ({}) pas sous le niveau {} ({})",
                    n + 1,
                    decimal(morts(n, p)),
                    SOMMET + 1,
                    decimal(morts(SOMMET, p))
                ));
            }
        }
        if morts(APRES_SOMMET, p) <= morts(FINAL, p) {
            ecarts.push(format!(
                "{nom} : niveau {} ({}) pas au-dessus du niveau {} ({})",
                APRES_SOMMET + 1,
                decimal(morts(APRES_SOMMET, p)),
                FINAL + 1,
                decimal(morts(FINAL, p))
            ));
        }
    }
    for (n, &(min, max)) in BANDES_HABITUE.iter().enumerate() {
        let m = morts(n, HABITUE);
        if !(min..max).contains(&m) {
            ecarts.push(format!(
                "Habitué : niveau {} à {}, hors de sa bande [{} ; {}[",
                n + 1,
                centiemes(m),
                centiemes(min),
                centiemes(max)
            ));
        }
    }
    if morts(SOMMET, HABITUE) - morts(APRES_SOMMET, HABITUE) < ECART_APRES_SOMMET {
        ecarts.push(format!(
            "Habitué : niveau {} ({}) pas nettement sous le niveau {} ({})",
            APRES_SOMMET + 1,
            centiemes(morts(APRES_SOMMET, HABITUE)),
            SOMMET + 1,
            centiemes(morts(SOMMET, HABITUE))
        ));
    }
    ecarts
}

/// Nombre à deux décimales, virgule à la française : 0.456 -> « 0,46 ».
fn centiemes(x: f32) -> String {
    format!("{x:.2}").replace('.', ",")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn saut() -> Plan {
        Plan {
            dir: Direction::Droite,
            elan: 8,
            saut: 13,
            planer: true,
            retard_plane: 0,
            dir_montee: true,
            dir_descente: true,
            attente: 0,
        }
    }

    #[test]
    fn le_hasard_est_reproductible_et_borne() {
        let mut a = Hasard::new(7);
        let mut b = Hasard::new(7);
        let mut c = Hasard::new(8);
        let sa: Vec<u64> = (0..50).map(|_| a.suivant()).collect();
        let sb: Vec<u64> = (0..50).map(|_| b.suivant()).collect();
        let sc: Vec<u64> = (0..50).map(|_| c.suivant()).collect();
        assert_eq!(sa, sb, "même graine, même suite");
        assert_ne!(sa, sc, "graines voisines, suites différentes");
        let mut h = Hasard::new(GRAINE);
        for _ in 0..1000 {
            let x = h.entre(3, 9);
            assert!((3..=9).contains(&x));
        }
        assert!(!(0..1000).any(|_| h.chance(0)), "0 pour mille : jamais");
        assert!(
            (0..1000).all(|_| h.chance(1000)),
            "1000 pour mille : toujours"
        );
    }

    #[test]
    fn perturber_respecte_le_profil() {
        let parfait = Profil {
            erreur_pour_mille: 0,
            ..PROFILS[0]
        };
        let maladroit = Profil {
            erreur_pour_mille: 1000,
            ..PROFILS[0]
        };
        let mut h = Hasard::new(1);
        for _ in 0..200 {
            assert_eq!(perturber(saut(), &parfait, &mut h), saut());
            let p = perturber(saut(), &maladroit, &mut h);
            assert_ne!(p, saut(), "erreur certaine : le plan change");
            assert_eq!(p.dir, Direction::Droite, "la direction ne change jamais");
            assert!(p.saut >= 1, "un saut reste un saut");
            assert!(p.elan.abs_diff(8) <= maladroit.ecart_elan);
            assert!(p.saut.abs_diff(13) <= maladroit.ecart_saut);
            assert!(p.retard_plane <= maladroit.retard_plane);
        }
        // Attendre reste attendre, même pour un joueur maladroit.
        let attendre = Plan {
            dir: Direction::Immobile,
            attente: 45,
            ..saut()
        };
        assert_eq!(perturber(attendre, &maladroit, &mut h), attendre);
    }

    /// Niveau 1 : sans erreur, aucune mort ; et un essai se rejoue à l'identique.
    #[test]
    fn niveau_1_sans_erreur_aucune_mort_et_essais_reproductibles() {
        let itin = Itineraires::calculer(0);
        let parfait = Profil {
            erreur_pour_mille: 0,
            ..PROFILS[2]
        };
        let e = jouer_essai(&itin, &parfait, 1);
        assert!(e.fini, "le joueur sans erreur finit le niveau 1");
        assert_eq!(e.morts, 0, "aucune mort sans erreur");

        let a = jouer_essai(&itin, &PROFILS[0], graine_essai(GRAINE, 0, 0, 0));
        let b = jouer_essai(&itin, &PROFILS[0], graine_essai(GRAINE, 0, 0, 0));
        assert_eq!(a, b, "même graine : même essai, mort pour mort");
        assert!(a.fini, "le débutant finit quand même le niveau 1 : {a:?}");
        assert_eq!(a.lieux.len() as u32, a.morts, "un lieu noté par mort");
    }

    /// Les lieux de mort se regroupent par cause et par colonnes voisines.
    #[test]
    fn regrouper_les_lieux_de_mort() {
        let l = |colonne, cause| LieuMort { colonne, cause };
        let lieux = [
            l(61, CauseMort::Vide),
            l(60, CauseMort::Vide),
            l(63, CauseMort::Vide),
            l(62, CauseMort::Ennemi),
            l(90, CauseMort::Vide),
            l(70, CauseMort::Vide),
        ];
        let e = regrouper(&lieux);
        // 60, 61, 63 : un même trou (écarts de 2 au plus) ; 70 et 90 à part ;
        // l'ennemi en 62 reste à part (autre cause).
        assert_eq!(e.len(), 4, "{e:?}");
        assert_eq!((e[0].debut, e[0].fin, e[0].morts), (60, 63, 3));
        assert_eq!(e[0].cause, CauseMort::Vide);
        assert_eq!(decrire(&e[0]), "col. 60-63 chute 3");
        let total: u32 = e.iter().map(|x| x.morts).sum();
        assert_eq!(total, 6, "aucune mort perdue");
        assert!(regrouper(&[]).is_empty());
    }

    /// Diagnostic : quelques essais par niveau, avec le détail de chacun.
    #[test]
    #[ignore]
    fn diagnostic_essais() {
        for n in 0..NB_NIVEAUX {
            let t = std::time::Instant::now();
            let itin = Itineraires::calculer(n);
            println!(
                "niveau {} : {} cases explorées en {:.1} s",
                n + 1,
                itin.cases,
                t.elapsed().as_secs_f32()
            );
            for (p, profil) in PROFILS.iter().enumerate() {
                for e in 0..3 {
                    let t = std::time::Instant::now();
                    let r =
                        jouer_essai(&itin, profil, graine_essai(GRAINE, p, rang_de_graine(n), e));
                    println!(
                        "  {} essai {e} : {r:?} en {:.1} s",
                        profil.nom,
                        t.elapsed().as_secs_f32()
                    );
                }
            }
        }
    }

    /// Écrit `target/qa/script-31b.txt` (même format que `script.txt`), preuve de la
    /// carte 3.1b : le robot finit le niveau 1, puis le niveau 2 est joué par le
    /// premier essai « Débutant » de la mesure où Pépin réapparaît à la lanterne de
    /// la colonne 104 DANS une flaque (le cas des captures 31-, où la flaque le tuait
    /// à chaque pas). Affiche, image par image autour de cette réapparition : morts,
    /// protection restante, Pépin dessiné ou caché (clignotement), flaque sur lui.
    /// De quoi choisir les images à photographier.
    ///
    /// Il remplace `ecrire_script_morts_en_serie` (`script-31.txt`) : depuis la
    /// protection de réapparition, aucun essai n'a plus de morts en série ; pour
    /// régénérer `script-31.txt`, extraire le commit 96d043e.
    /// `./docker-cargo.sh test -p game --no-default-features ecrire_script_lanterne_104 -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn ecrire_script_lanterne_104() {
        use crate::bot_qa::{jouer_tout, trouver_parcours};
        use crate::scene::pepin_cache;
        use pepin_core::entities::{Entite, TypeEnnemi};
        let lanterne = Monde::new(1)
            .entites
            .iter()
            .find_map(|e| match e {
                Entite::Lanterne(l) if (l.boite.x / 16.0) as i32 == 104 => {
                    Some(l.point_de_reapparition())
                }
                _ => None,
            })
            .expect("lanterne de la colonne 104");
        let flaque_sur_pepin = |m: &Monde| {
            m.entites.iter().any(|e| {
                matches!(e, Entite::Ennemi(f) if f.type_ennemi == TypeEnnemi::Flaque
                    && f.est_dangereux()
                    && f.boite.chevauche(&m.joueur.boite))
            })
        };
        // Un état noté après chaque pas : (morts, protection, Pépin caché, flaque
        // sur Pépin, x, y).
        type Note = (u32, u16, bool, bool, f32, f32);
        let itin = Itineraires::calculer(1);
        let (essai, journal, notes, cible) = (0..ESSAIS)
            .find_map(|e| {
                let mut journal = Vec::new();
                let graine = graine_essai(GRAINE, 0, rang_de_graine(1), e);
                jouer_essai_note(&itin, &PROFILS[0], graine, &mut journal);
                let mut m = Monde::new(1);
                let mut notes: Vec<Note> = Vec::new();
                let mut cible = None;
                for entree in &journal {
                    let avant = m.etat.morts;
                    m.avancer(entree);
                    let (j, sur) = (&m.joueur, flaque_sur_pepin(&m));
                    if m.etat.morts > avant
                        && cible.is_none()
                        && (j.boite.x, j.boite.y) == lanterne
                        && sur
                    {
                        cible = Some(notes.len());
                    }
                    let p = j.protection_restante();
                    notes.push((
                        m.etat.morts,
                        p,
                        pepin_cache(p, m.pas),
                        sur,
                        j.boite.x,
                        j.boite.y,
                    ));
                }
                cible.map(|c| (e, journal, notes, c))
            })
            .expect("aucun essai ne réapparaît à la lanterne 104 dans une flaque");
        let partie = jouer_tout(&[trouver_parcours(0, false).unwrap(), journal]);
        let f0 = partie
            .trames
            .iter()
            .position(|t| t.niveau == 2)
            .expect("le niveau 2 commence");
        // Image de la partie où l'on voit l'état noté n° 0 : celle dont la position
        // de Pépin colle à TOUTES les notes suivantes (vérifié, pas supposé).
        let base = (f0.saturating_sub(2)..f0 + 3)
            .find(|&b| {
                notes.iter().enumerate().all(|(i, n)| {
                    partie
                        .trames
                        .get(b + i)
                        .is_some_and(|t| (t.x, t.y) == (n.4, n.5))
                })
            })
            .expect("partie et rejeu concordent");
        let fin = (base + cible + 240).min(partie.trames.len());
        let script: String = partie.trames[..fin].iter().map(|t| t.code()).collect();
        std::fs::create_dir_all("../target/qa").unwrap();
        std::fs::write("../target/qa/script-31b.txt", &script).unwrap();
        let morts = notes.last().map_or(0, |n| n.0);
        println!(
            "essai Débutant n° {essai} ; niveau 2 : image {f0} ; pas 1 = image {base} ; \
             {morts} morts dans l'essai ; réapparition à la lanterne 104 dans la flaque : \
             pas {} = image {} ; script : {fin} images",
            cible + 1,
            base + cible
        );
        println!("image ; morts ; protection ; Pépin caché ; flaque sur Pépin ; x ; y");
        let debut = cible.saturating_sub(60);
        let fin_notes = (cible + 150).min(notes.len());
        for (i, &(m, p, cache, sur, x, y)) in notes[debut..fin_notes].iter().enumerate() {
            println!(
                "{} ; {m} ; {p} ; {cache} ; {sur} ; {x:.1} ; {y:.1}",
                base + debut + i
            );
        }
    }

    /// LA MESURE : `./docker-cargo.sh test -p game --no-default-features
    /// mesure_de_difficulte -- --ignored --nocapture`. Affiche le tableau et
    /// l'écrit dans `docs/mesure-difficulte.txt` (versionné : `git diff` montre
    /// l'effet d'une carte sur la difficulté).
    #[test]
    #[ignore]
    fn mesure_de_difficulte() {
        let debut = std::time::Instant::now();
        let cases = mesurer(GRAINE, ESSAIS);
        let t = tableau(&cases, GRAINE, ESSAIS);
        println!("{t}");
        println!("(calcul : {:.0} s)", debut.elapsed().as_secs_f32());
        let chemin = concat!(env!("CARGO_MANIFEST_DIR"), "/../docs/mesure-difficulte.txt");
        std::fs::write(chemin, &t).expect("écriture de docs/mesure-difficulte.txt");
        // Garde-fous : le jeu reste finissable par tous les profils, et l'ordre
        // des profils est respecté sur l'ensemble du jeu.
        for (n, ligne) in cases.iter().enumerate() {
            for (p, c) in ligne.iter().enumerate() {
                assert!(
                    c.finis * 2 >= c.essais,
                    "niveau {} : {} ne finit que {}/{} essais",
                    n + 1,
                    PROFILS[p].nom,
                    c.finis,
                    c.essais
                );
            }
        }
        let total = |p: usize| cases.iter().map(|l| l[p].morts_moyennes).sum::<f32>();
        assert!(total(0) >= total(1) && total(1) >= total(2), "{t}");
        let ecarts = ecarts_de_la_courbe(&cases);
        assert!(ecarts.is_empty(), "{}\n\n{t}", ecarts.join("\n"));
    }

    /// Une mesure inventée : morts moyennes du Débutant et de l'Habitué par
    /// niveau (l'Expert ne compte pas pour la forme de la courbe).
    fn fausse_mesure(debutant: [f32; NB_NIVEAUX], habitue: [f32; NB_NIVEAUX]) -> Vec<Vec<Case>> {
        (0..NB_NIVEAUX)
            .map(|n| {
                [debutant[n], habitue[n], 0.0]
                    .iter()
                    .map(|&m| Case {
                        morts_moyennes: m,
                        ..Case::default()
                    })
                    .collect()
            })
            .collect()
    }

    /// Chaque niveau de la table a un rang de graine, et un seul ; les cinq premiers
    /// rangs sont les places qu'avaient ces niveaux avant la carte 5.12 (leur mesure
    /// ne change pas quand on insère un niveau dans le parcours).
    #[test]
    fn chaque_niveau_a_un_rang_de_graine() {
        let mut rangs: Vec<usize> = (0..NB_NIVEAUX).map(rang_de_graine).collect();
        rangs.sort_unstable();
        rangs.dedup();
        assert_eq!(rangs.len(), NB_NIVEAUX);
        let anciennes = [
            toits::INFOS,
            gouttieres::INFOS,
            orage::INFOS,
            vents::INFOS,
            nuages::INFOS,
        ];
        for (place, infos) in anciennes.iter().enumerate() {
            let n = NIVEAUX.iter().position(|i| i.nom == infos.nom).unwrap();
            assert_eq!(rang_de_graine(n), place, "{}", infos.nom);
        }
    }

    /// Le juge de la courbe accepte la bonne forme et refuse les mauvaises.
    #[test]
    fn la_forme_de_la_courbe_est_jugee() {
        // Les places jugées désignent bien le sommet, le niveau suivant et le final.
        assert_eq!(NIVEAUX[SOMMET].nom, orage::INFOS.nom);
        assert_eq!(NIVEAUX[APRES_SOMMET].nom, vents::INFOS.nom);
        assert_eq!(NIVEAUX[FINAL].nom, nuages::INFOS.nom);

        let bonne = fausse_mesure(
            [0.4, 1.5, 2.0, 3.0, 1.0, 1.3, 0.5],
            [0.1, 0.5, 0.6, 0.8, 0.4, 0.45, 0.1],
        );
        assert_eq!(ecarts_de_la_courbe(&bonne), Vec::<String>::new());

        // La courbe d'avant la carte 3.3 : le 6 (vents) presque aussi dur que le 4
        // (orage), le 7 à 0,4.
        let avant = fausse_mesure(
            [0.4, 1.5, 2.0, 3.6, 1.0, 2.9, 2.5],
            [0.1, 0.5, 0.6, 1.0, 0.4, 0.9, 0.4],
        );
        let e = ecarts_de_la_courbe(&avant);
        assert!(e.iter().any(|x| x.contains("niveau 6 à 0,90")), "{e:?}");
        assert!(e.iter().any(|x| x.contains("niveau 7 à 0,40")), "{e:?}");
        assert!(e.iter().any(|x| x.contains("pas nettement")), "{e:?}");

        // Le 6 plus dur que le 4 (Débutant) ; le 7 aussi dur que le 6 (Habitué) ;
        // le marché (3) plus dur que l'orage (Débutant).
        let a_l_envers = fausse_mesure(
            [0.4, 1.5, 2.2, 2.0, 1.0, 2.5, 0.5],
            [0.1, 0.5, 0.6, 0.8, 0.4, 0.1, 0.1],
        );
        let e = ecarts_de_la_courbe(&a_l_envers);
        assert!(
            e.iter()
                .any(|x| x == "Débutant : niveau 6 (2,5) pas sous le niveau 4 (2,0)"),
            "{e:?}"
        );
        assert!(
            e.iter()
                .any(|x| x == "Débutant : niveau 3 (2,2) pas sous le niveau 4 (2,0)"),
            "{e:?}"
        );
        assert!(
            e.iter()
                .any(|x| x == "Habitué : niveau 6 (0,1) pas au-dessus du niveau 7 (0,1)"),
            "{e:?}"
        );
    }

    /// NON-RÉGRESSION DE LA COURBE : la vraie mesure (graine officielle, mêmes
    /// chiffres que `docs/mesure-difficulte.txt`) garde la forme voulue par la
    /// carte 3.3. Si une retouche de niveau, d'ennemi ou de mécanique la casse,
    /// ce test le dit (environ 30 s : 15 fils d'exécution en parallèle).
    #[test]
    fn courbe_de_difficulte_dans_les_bandes() {
        let cases = mesurer(GRAINE, ESSAIS);
        let ecarts = ecarts_de_la_courbe(&cases);
        assert!(
            ecarts.is_empty(),
            "{}\n\n{}",
            ecarts.join("\n"),
            tableau(&cases, GRAINE, ESSAIS)
        );
    }

    /// Bruit de la mesure : la même mesure avec plusieurs graines (variable
    /// d'environnement GRAINES, par exemple `GRAINES=1,2,3`, défaut : la graine
    /// officielle et ses 3 voisines). Montre de combien les morts moyennes
    /// bougent par le seul hasard : une différence plus petite que ce bruit ne
    /// prouve rien. Affiche aussi les lieux de mort cumulés sur toutes les
    /// graines (plus sûrs que ceux d'une seule). N'écrit aucun fichier.
    #[test]
    #[ignore]
    fn bruit_de_la_mesure() {
        let graines: Vec<u64> = std::env::var("GRAINES")
            .ok()
            .map(|g| g.split(',').filter_map(|x| x.trim().parse().ok()).collect())
            .unwrap_or_else(|| vec![GRAINE, GRAINE + 1, GRAINE + 2, GRAINE + 3]);
        let mut cumul: Vec<Vec<Vec<LieuMort>>> = vec![vec![Vec::new(); PROFILS.len()]; NB_NIVEAUX];
        for g in graines {
            let cases = mesurer(g, ESSAIS);
            let ligne: Vec<String> = cases
                .iter()
                .enumerate()
                .map(|(n, rang)| {
                    let profils: Vec<String> =
                        rang.iter().map(|c| decimal(c.morts_moyennes)).collect();
                    format!("N{} {}", n + 1, profils.join(" / "))
                })
                .collect();
            println!("graine {g} : {}", ligne.join(" · "));
            for (n, rang) in cases.iter().enumerate() {
                for (p, c) in rang.iter().enumerate() {
                    cumul[n][p].extend(c.lieux.iter().copied());
                }
            }
        }
        // Lieux de mort cumulés sur toutes les graines : plus stables qu'une seule.
        for (n, rang) in cumul.iter().enumerate() {
            for (p, lieux) in rang.iter().enumerate() {
                let endroits: Vec<String> = regrouper(lieux).iter().take(8).map(decrire).collect();
                println!("N{} {} : {}", n + 1, PROFILS[p].nom, endroits.join(" · "));
            }
        }
    }
}
