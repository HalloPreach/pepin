//! Robot de QA : un « joueur automatique » qui finit tous les niveaux avec la
//! VRAIE simulation du jeu (`Monde` : ennemis, éclairs, plateformes mobiles et
//! friables comprises), puis rejoue ce parcours à travers l'`Application`
//! (titre, cartes de niveau, transitions, écran de fin).
//!
//! Différence avec `pepin_core::faisabilite` : le robot du cœur ignore ennemis
//! et plateformes mobiles. Celui-ci les subit : un plan qui mène à la mort est
//! simplement écarté, donc le parcours trouvé est sûr par construction.
//!
//! Principe : recherche de plus court chemin (Dijkstra sur la durée) entre des
//! états « posés et immobiles » (ou « lancés » : un auvent vient de relancer
//! Pépin, et il choisit de nouveau en l'air ; voir `est_lance`). Chaque plan
//! (courir, sauter, planer, attendre) est simulé sur une COPIE du monde. Le test
//! `ignore` d'en bas écrit le script des touches dans `target/qa/` : un
//! navigateur réel le rejoue image par image (voir QA.md).
//!
//! Notions Rust illustrées ici : `BinaryHeap` + `Reverse` (file de priorité
//! « plus petit d'abord »), `Clone` d'une structure complète pour explorer des
//! futurs possibles sans toucher au présent, `HashMap` comme mémoire.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use pepin_core::entities::contacts::Contact;
use pepin_core::entities::ennemi::PhasePigeon;
use pepin_core::entities::montgolfiere::EtatMontgolfiere;
use pepin_core::entities::ventilateur::PhaseVentilateur;
use pepin_core::entities::{Entite, TypeEnnemi};
use pepin_core::faisabilite::pas_max_plan;
use pepin_core::player::{Direction, Entrees};
use pepin_core::zones::NB_NIVEAUX;

use crate::app::Application;
use crate::ecrans::Ecran;
use crate::entrees::Commandes;
use crate::monde::Monde;
use crate::navigation::Navigation;

/// Pas d'immobilité qui valident un atterrissage stable.
const PAS_STABLE: u16 = 20;
/// Pas d'attente au départ : Pépin tombe au sol et se pose.
pub(crate) const PAS_DEPART: u16 = 40;
/// Pas passés en l'air avant que le plan `planer` n'ouvre le parapluie.
const DEBUT_PLANE: u16 = 7;

/// Un plan d'entrées, simulé d'un seul bloc. `pub(crate)` : la mesure de
/// difficulté rejoue les mêmes plans, avec des erreurs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Plan {
    pub(crate) dir: Direction,
    /// Pas de course au sol avant le saut.
    pub(crate) elan: u16,
    /// Pas de maintien du saut (0 = pas de saut, on marche dans le vide).
    pub(crate) saut: u16,
    pub(crate) planer: bool,
    /// Pas de chute avant d'ouvrir le parapluie (si `planer`). Toujours 0 pour le
    /// robot (il ouvre dès le sommet du saut) ; la mesure de difficulté
    /// (`difficulte.rs`) l'allonge pour imiter un « plané tardif ».
    pub(crate) retard_plane: u16,
    pub(crate) dir_montee: bool,
    pub(crate) dir_descente: bool,
    /// Plan « attendre » : aucune entrée pendant ce nombre de pas (0 = non).
    pub(crate) attente: u16,
}

/// Comment se termine la simulation d'un plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Fin {
    Mort,
    Arrivee,
    Pose,
    /// Un auvent vient de relancer Pépin : nouveau point de décision, en l'air
    /// (voir `est_lance`).
    Lance,
    Abandon,
}

/// Tous les plans à essayer. `precis` ajoute des élans et des durées plus fins.
pub(crate) fn plans(precis: bool) -> Vec<Plan> {
    let (elans, sauts): (&[u16], &[u16]) = if precis {
        (&[0, 4, 8, 14, 22, 32, 50, 80, 120], &[0, 5, 9, 13, 40])
    } else {
        (&[0, 8, 22, 50, 120], &[0, 13, 40])
    };
    let mut v = Vec::new();
    for dir in [Direction::Gauche, Direction::Droite] {
        for &elan in elans {
            for &saut in sauts {
                for planer in [false, true] {
                    for dir_montee in [true, false] {
                        for dir_descente in [true, false] {
                            if saut == 0 && !dir_montee {
                                continue; // sans saut, « montée » n'a pas de sens
                            }
                            v.push(Plan {
                                dir,
                                elan,
                                saut,
                                planer,
                                retard_plane: 0,
                                dir_montee,
                                dir_descente,
                                attente: 0,
                            });
                        }
                    }
                }
            }
        }
    }
    // Attendre : laisse passer une plateforme mobile, un ennemi ou un éclair.
    for attente in [20, 45, 90, 180] {
        v.push(Plan {
            dir: Direction::Immobile,
            elan: 0,
            saut: 0,
            planer: false,
            retard_plane: 0,
            dir_montee: false,
            dir_descente: false,
            attente,
        });
    }
    v
}

/// Les plans « en vol », essayés juste après un rebond d'auvent (`Fin::Lance`) :
/// une direction (gauche, droite, ou aucune) gardée pendant la montée et/ou la
/// descente, parapluie ouvert ou non. Ni élan au sol, ni saut, ni attente : on
/// est en l'air. Même liste que le robot du cœur (`faisabilite::plans_en_vol`).
pub(crate) fn plans_en_vol() -> Vec<Plan> {
    let mut v = Vec::new();
    for planer in [false, true] {
        for dir in [Direction::Gauche, Direction::Droite] {
            for dir_montee in [true, false] {
                for dir_descente in [true, false] {
                    v.push(Plan {
                        dir,
                        elan: 0,
                        saut: 0,
                        planer,
                        retard_plane: 0,
                        dir_montee,
                        dir_descente,
                        attente: 0,
                    });
                }
            }
        }
        v.push(Plan {
            dir: Direction::Immobile,
            elan: 0,
            saut: 0,
            planer,
            retard_plane: 0,
            dir_montee: false,
            dir_descente: false,
            attente: 0,
        });
    }
    v
}

/// Pépin est-il en vol, lancé par un auvent (élan d'auvent pas encore éteint) ?
/// Juste après un rebond (`Fin::Lance`), c'est un point de décision en l'air :
/// on y essaie les `plans_en_vol`, pas les plans du sol.
pub(crate) fn est_lance(m: &Monde) -> bool {
    m.joueur.elan_auvent() > 0.0 && !m.joueur.au_sol
}

/// Pépin est-il mort (ou en train de réapparaître) ? `derniere_mort` n'est jamais
/// effacée par le jeu : après une mort, l'appelant doit la remettre à `None`
/// avant de simuler de nouveaux plans (voir `difficulte.rs`).
pub(crate) fn est_mort(m: &Monde) -> bool {
    m.reapparition > 0 || m.derniere_mort.is_some()
}

/// Simule `plan` depuis `depart` (copie). Renvoie la fin, le monde final et les
/// entrées de chaque pas.
pub(crate) fn simuler(depart: &Monde, plan: Plan) -> (Fin, Monde, Vec<Entrees>) {
    let mut m = depart.clone();
    let mut script = Vec::new();

    if plan.attente > 0 {
        for _ in 0..plan.attente {
            m.avancer(&Entrees::default());
            script.push(Entrees::default());
            if est_mort(&m) {
                return (Fin::Mort, m, script);
            }
            if m.termine {
                return (Fin::Arrivee, m, script);
            }
        }
        let fin = if m.joueur.au_sol {
            Fin::Pose
        } else {
            Fin::Abandon
        };
        return (fin, m, script);
    }

    let mut en_l_air: u16 = 0;
    // Lancé par un auvent : déjà en l'air, les commandes « en vol » s'appliquent
    // dès le premier pas. (Toujours faux sans auvent : rien ne change ailleurs.)
    let mut a_decolle = est_lance(&m);
    let mut stable: u16 = 0;
    let mut saut_envoye: u16 = 0;
    let mut chute: u16 = 0;
    // Durée maximale d'un plan : 10 s dans un niveau d'un écran, le temps d'un plané sur
    // toute la hauteur dans un niveau haut (même règle que le robot du cœur).
    for pas in 0..pas_max_plan(m.niveau.hauteur_px()) {
        let mut e = Entrees::default();
        let marche_finie = plan.saut == 0 && pas >= plan.elan;
        if !a_decolle && stable == 0 && !marche_finie {
            e.direction = plan.dir;
            if pas >= plan.elan {
                e.saut = true;
                saut_envoye += 1;
            }
        } else if a_decolle && stable == 0 {
            let montee = m.joueur.vitesse.y < 0.0;
            if !montee {
                chute += 1;
            }
            let garde = if montee {
                plan.dir_montee
            } else {
                plan.dir_descente
            };
            e.direction = if garde { plan.dir } else { Direction::Immobile };
            if saut_envoye > 0 && saut_envoye < plan.saut && montee {
                e.saut = true;
                saut_envoye += 1;
            } else if plan.planer && !montee && en_l_air >= DEBUT_PLANE && chute > plan.retard_plane
            {
                e.saut = true;
            }
        }
        m.avancer(&e);
        script.push(e);
        if est_mort(&m) {
            return (Fin::Mort, m, script);
        }
        if m.termine {
            return (Fin::Arrivee, m, script);
        }
        if m.contacts.contains(&Contact::Auvent) {
            return (Fin::Lance, m, script);
        }
        if !m.joueur.au_sol {
            a_decolle = true;
            en_l_air += 1;
            stable = 0;
        } else if a_decolle || marche_finie {
            stable += 1;
            if stable >= PAS_STABLE {
                return (Fin::Pose, m, script);
            }
        }
    }
    (Fin::Abandon, m, script)
}

/// Une case du niveau où Pépin choisit son prochain plan : case de 8 px en x, y
/// arrondi, « lancé par un auvent » (`est_lance`) ou posé, et l'escargot proche
/// (`escargot_proche`, 0 s'il n'y en a pas). Deux états proches fusionnent.
pub(crate) type CaseParcours = (i32, i32, bool, i32);

/// La case (`CaseParcours`) de Pépin dans `m`.
pub(crate) fn cle(m: &Monde) -> CaseParcours {
    (
        (m.joueur.boite.x / 8.0).floor() as i32,
        m.joueur.boite.y.round() as i32,
        est_lance(m),
        escargot_proche(m),
    )
}

/// Portée (px) à laquelle un escargot compte dans la case de Pépin : 6 tuiles en x
/// (entre les centres), 4 tuiles en y (entre les pieds).
const PORTEE_ESCARGOT: (f32, f32) = (96.0, 64.0);

/// L'escargot le plus proche de Pépin (dans `PORTEE_ESCARGOT`), résumé en un
/// entier : `1 + 2 × (case de 16 px de son x) + (1 s'il va à droite)` ; 0 si aucun
/// escargot n'est proche (toujours 0 dans un niveau sans escargot : la case est
/// alors exactement celle d'avant la carte 5.8).
///
/// Pourquoi : l'escargot est un tremplin MOBILE, il faut l'ATTENDRE au bon endroit.
/// Sans lui dans la case, attendre sur place ramène à une case déjà connue et le
/// plan « attendre » est jeté : le robot ne pourrait jamais laisser l'escargot
/// venir sous lui. Avec lui, chaque attente où l'escargot a avancé d'une case est
/// un nouvel état. Loin des escargots, rien ne change (pas d'explosion du nombre
/// d'états).
pub(crate) fn escargot_proche(m: &Monde) -> i32 {
    let j = &m.joueur.boite;
    let centre = j.x + j.w / 2.0;
    m.entites
        .iter()
        .filter_map(|e| match e {
            Entite::Escargot(s) => {
                let dx = (s.boite.x + s.boite.w / 2.0 - centre).abs();
                let dy = (s.boite.bas() - j.bas()).abs();
                (dx <= PORTEE_ESCARGOT.0 && dy <= PORTEE_ESCARGOT.1).then_some((dx, s))
            }
            _ => None,
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map_or(0, |(_, s)| {
            1 + 2 * (s.boite.x / 16.0).floor() as i32 + i32::from(s.vers_la_droite)
        })
}

/// Un morceau de parcours : le plan joué, les entrées exactes qu'il a produites
/// et la case (`cle`) où Pépin s'est posé, a été relancé, ou a touché l'arrivée.
#[derive(Clone)]
pub(crate) struct Etape {
    pub(crate) plan: Plan,
    pub(crate) entrees: Vec<Entrees>,
    pub(crate) cle: CaseParcours,
}

/// Le monde du niveau `indice` après `PAS_DEPART` pas sans entrée (Pépin posé),
/// et ces entrées vides.
pub(crate) fn depart_pose(indice: usize) -> (Monde, Vec<Entrees>) {
    let mut depart = Monde::new(indice);
    let prefixe = vec![Entrees::default(); PAS_DEPART as usize];
    for e in &prefixe {
        depart.avancer(e);
    }
    (depart, prefixe)
}

/// Plus court chemin (en pas) depuis `depart` jusqu'à l'arrivée, ou jusqu'à un
/// atterrissage que `est_cible` accepte. `limite` : nombre maximal de noeuds
/// développés (`usize::MAX` : aucune). `None` si rien n'est trouvé.
///
/// Le robot cherche l'arrivée seule (`est_cible` toujours faux) ; la mesure de
/// difficulté s'en sert aussi pour « rejoindre le parcours » après une erreur.
pub(crate) fn chercher(
    depart: &Monde,
    tous: &[Plan],
    est_cible: &dyn Fn(&Monde) -> bool,
    limite: usize,
) -> Option<Vec<Etape>> {
    // Chaque noeud : le monde posé, son parent et l'étape qui y a mené. La file
    // de priorité ne contient que (coût, indice).
    struct Noeud {
        monde: Monde,
        parent: Option<usize>,
        etape: Option<Etape>,
    }
    let mut noeuds = vec![Noeud {
        monde: depart.clone(),
        parent: None,
        etape: None,
    }];
    let mut meilleur: HashMap<CaseParcours, u32> = HashMap::new();
    meilleur.insert(cle(&noeuds[0].monde), 0);
    let mut tas = BinaryHeap::new();
    tas.push(Reverse((0u32, 0usize)));
    let mut arrivee: Option<(u32, usize, Etape)> = None;
    let mut developpes = 0usize;
    // Après un rebond d'auvent, Pépin est en l'air : on n'essaie que les plans en vol.
    let en_vol = plans_en_vol();

    while let Some(Reverse((cout, i))) = tas.pop() {
        if let Some((cout_arrivee, _, _)) = &arrivee {
            if cout >= *cout_arrivee {
                break; // plus rien de moins cher à trouver
            }
        }
        if meilleur
            .get(&cle(&noeuds[i].monde))
            .is_some_and(|&c| c < cout)
        {
            continue;
        }
        if developpes >= limite {
            break;
        }
        developpes += 1;
        let a_essayer = if est_lance(&noeuds[i].monde) {
            &en_vol[..]
        } else {
            tous
        };
        for &plan in a_essayer {
            let (fin, monde, entrees) = simuler(&noeuds[i].monde, plan);
            let cout_plan = cout + entrees.len() as u32;
            let etape = Etape {
                plan,
                entrees,
                cle: cle(&monde),
            };
            match fin {
                Fin::Arrivee => {
                    if arrivee.as_ref().is_none_or(|(c, _, _)| cout_plan < *c) {
                        arrivee = Some((cout_plan, i, etape));
                    }
                }
                Fin::Pose | Fin::Lance if est_cible(&monde) => {
                    if arrivee.as_ref().is_none_or(|(c, _, _)| cout_plan < *c) {
                        arrivee = Some((cout_plan, i, etape));
                    }
                }
                Fin::Pose | Fin::Lance => {
                    let k = etape.cle;
                    if meilleur.get(&k).is_none_or(|&c| cout_plan < c) {
                        meilleur.insert(k, cout_plan);
                        noeuds.push(Noeud {
                            monde,
                            parent: Some(i),
                            etape: Some(etape),
                        });
                        tas.push(Reverse((cout_plan, noeuds.len() - 1)));
                    }
                }
                Fin::Mort | Fin::Abandon => {}
            }
        }
    }

    let (_, dernier, etape_finale) = arrivee?;
    // Remonte la chaîne des parents, puis remet les étapes à l'endroit.
    let mut etapes = vec![etape_finale];
    let mut courant = Some(dernier);
    while let Some(i) = courant {
        if let Some(e) = &noeuds[i].etape {
            etapes.push(e.clone());
        }
        courant = noeuds[i].parent;
    }
    etapes.reverse();
    Some(etapes)
}

/// Cherche les entrées (une par pas) qui mènent du début du niveau `indice` à
/// l'arrivée sans jamais mourir. `None` si rien n'est trouvé.
pub fn trouver_parcours(indice: usize, precis: bool) -> Option<Vec<Entrees>> {
    let (depart, mut entrees) = depart_pose(indice);
    let etapes = chercher(&depart, &plans(precis), &|_| false, usize::MAX)?;
    for e in etapes {
        entrees.extend(e.entrees);
    }
    Some(entrees)
}

/// Ce que le navigateur doit rejouer à chaque image.
#[derive(Clone, Copy, Default)]
pub struct Trame {
    pub valider: bool,
    pub entrees: Entrees,
    /// Numéro du niveau affiché (0 hors jeu) et position de Pépin : pour choisir
    /// les images à photographier.
    pub niveau: u8,
    pub x: f32,
    pub y: f32,
    pub ecran: &'static str,
    /// Position du premier corbeau actif du niveau (pour viser une capture).
    pub corbeau: Option<(f32, f32)>,
    /// Boîte de la première plateforme mobile (x, y), pour viser une capture.
    pub mobile: Option<(f32, f32)>,
    /// Premier pigeon visible du niveau : position et phase (`nom_phase`), pour
    /// viser une capture (perché, annonce, plongée...).
    pub pigeon: Option<(f32, f32, &'static str)>,
    /// État de la partie après l'image : score, gouttes, morts et chrono (pas de
    /// jeu). Ils ne vont pas dans `trace.csv` ; ils nourrissent `empreinte`.
    pub score: u32,
    pub gouttes: u32,
    pub morts: u32,
    pub chrono: u32,
}

impl Trame {
    /// Un chiffre hexadécimal : 1 = Entrée, 2 = gauche, 4 = droite, 8 = saut.
    pub fn code(&self) -> char {
        let mut v = 0u32;
        if self.valider {
            v |= 1;
        }
        match self.entrees.direction {
            Direction::Gauche => v |= 2,
            Direction::Droite => v |= 4,
            Direction::Immobile => {}
        }
        if self.entrees.saut {
            v |= 8;
        }
        char::from_digit(v, 16).unwrap()
    }
}

/// Nom court d'un état de pigeon, pour la trace (`ecrase` : écrasé, quelle que soit
/// la phase où on l'a eu).
fn nom_phase(phase: PhasePigeon, ecrase: bool) -> &'static str {
    if ecrase {
        return "ecrase";
    }
    match phase {
        PhasePigeon::Perche => "perche",
        PhasePigeon::Annonce(_) => "annonce",
        PhasePigeon::Plongee(_) => "plongee",
        PhasePigeon::Remontee(_) => "remontee",
        PhasePigeon::Repos(_) => "repos",
    }
}

fn nom_ecran(e: Ecran) -> &'static str {
    match e {
        Ecran::Titre => "titre",
        Ecran::Aide => "aide",
        Ecran::Carte(_) => "carte",
        Ecran::Jeu => "jeu",
        Ecran::Pause => "pause",
        Ecran::Fin => "fin",
    }
}

/// Résultat d'une partie complète jouée par le robot.
pub struct Partie {
    pub trames: Vec<Trame>,
    pub morts: u32,
    pub gouttes: u32,
    pub score: u32,
    /// Chrono de la partie (pas de jeu, sans menus ni transitions).
    pub chrono: u32,
    pub fin_atteinte: bool,
}

/// Empreinte (hachage FNV-1a sur 64 bits) de la partie, image par image : écran,
/// niveau affiché, score, gouttes, morts et chrono. Deux parties qui ont la même
/// empreinte ont eu le même chrono et le même score à CHAQUE image, pas seulement à
/// la fin : c'est la preuve qu'une carte « sans effet sur le jeu » n'a rien changé.
pub fn empreinte(trames: &[Trame]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut ajouter = |octets: &[u8]| {
        for &o in octets {
            h ^= u64::from(o);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    };
    for t in trames {
        ajouter(t.ecran.as_bytes());
        ajouter(&[t.niveau]);
        for v in [t.score, t.gouttes, t.morts, t.chrono] {
            ajouter(&v.to_le_bytes());
        }
    }
    h
}

/// Joue la partie entière à travers l'`Application`, image par image, comme le
/// ferait la boucle principale (un `reagir` puis un `pas` par image).
pub fn jouer_tout(parcours: &[Vec<Entrees>]) -> Partie {
    let mut app = Application::new(true);
    let mut trames: Vec<Trame> = Vec::new();
    let mut fin_atteinte = false;
    for image in 0..60 * 60 * 15 {
        // Image 0 : « JOUER » au titre. Ensuite les cartes passent toutes seules.
        let mut trame = Trame {
            valider: image == 0,
            ..Trame::default()
        };
        if app.ecran == Ecran::Jeu && app.en_jeu() {
            let n = app.monde.infos.numero as usize;
            if let Some(e) = parcours
                .get(n - 1)
                .and_then(|p| p.get(app.monde.pas as usize))
            {
                trame.entrees = *e;
            }
        }
        let nav = Navigation {
            valider: trame.valider,
            ..Navigation::default()
        };
        app.reagir(&nav, &Commandes::default(), None);
        app.pas(&trame.entrees);
        trame.ecran = nom_ecran(app.ecran);
        trame.niveau = if app.ecran == Ecran::Jeu {
            app.monde.infos.numero
        } else {
            0
        };
        trame.corbeau = app.monde.entites.iter().find_map(|e| match e {
            Entite::Ennemi(en) if en.type_ennemi == TypeEnnemi::Corbeau => {
                Some((en.boite.x, en.boite.y))
            }
            _ => None,
        });
        trame.mobile = app.monde.entites.iter().find_map(|e| match e {
            Entite::Mobile(m) => Some((m.boite.x, m.boite.y)),
            _ => None,
        });
        trame.pigeon = app.monde.entites.iter().find_map(|e| match e {
            Entite::Ennemi(p) if p.type_ennemi == TypeEnnemi::Pigeon && p.est_visible() => {
                let ecrase = !p.est_dangereux();
                Some((p.boite.x, p.boite.y, nom_phase(p.phase(), ecrase)))
            }
            _ => None,
        });
        trame.x = app.monde.joueur.boite.x;
        trame.y = app.monde.joueur.boite.y;
        trame.score = app.monde.etat.score;
        trame.gouttes = app.monde.etat.gouttes;
        trame.morts = app.monde.etat.morts;
        trame.chrono = app.chrono();
        trames.push(trame);
        if app.ecran == Ecran::Fin {
            fin_atteinte = true;
            // Quelques images de confettis, pour la capture.
            for _ in 0..240 {
                app.reagir(&Navigation::default(), &Commandes::default(), None);
                app.pas(&Entrees::default());
                trames.push(Trame {
                    ecran: "fin",
                    ..Trame::default()
                });
            }
            break;
        }
    }
    Partie {
        trames,
        morts: app.monde.etat.morts,
        gouttes: app.monde.etat.gouttes,
        score: app.monde.etat.score,
        chrono: app.chrono(),
        fin_atteinte,
    }
}

/// Cherche le parcours de chaque niveau de la table (robot « maladroit » d'abord,
/// puis précis si besoin).
pub fn tous_les_parcours() -> Vec<Vec<Entrees>> {
    (0..NB_NIVEAUX)
        .map(|i| {
            trouver_parcours(i, false)
                .or_else(|| trouver_parcours(i, true))
                .unwrap_or_else(|| panic!("le robot ne trouve aucun parcours au niveau {}", i + 1))
        })
        .collect()
}

// --- Niveau d'essai vertical (carte 5.2) -------------------------------------------

/// Une image de la partie d'essai : ce que le navigateur rejoue, et ce qu'on veut
/// vérifier de la caméra (position RENDUE, celle que la scène dessine, avec `alpha` = 1).
#[derive(Clone, Copy)]
pub struct ImageEssai {
    pub trame: Trame,
    /// La touche 0 (raccourci du niveau d'essai) est appuyée pendant cette image.
    pub touche_essai: bool,
    /// On joue le niveau d'essai (écran de jeu, monde qui avance).
    pub en_essai: bool,
    /// Caméra entière `(cx, cy)` et ordonnée de Pépin à l'écran (haut de sa boîte).
    pub camera: (i32, i32),
    pub pepin_ecran_y: i32,
    /// La flèche de repère est dessinée (Pépin au-dessus du cadre).
    pub repere: bool,
    /// Pépin plane DANS une colonne de grêle (carte 5.4) : parapluie ouvert, la grêle
    /// se brise sur la toile.
    pub sous_la_grele: bool,
    /// Un auvent vient de relancer Pépin pendant cette image (carte 5.5).
    pub rebond_auvent: bool,
    /// Pépin est posé au sol (sur une tuile ou un solide) à la fin de cette image.
    pub au_sol: bool,
    /// Haut des nacelles des deux premières montgolfières du niveau, de gauche à
    /// droite (carte 5.7 ; -1 s'il n'y en a pas).
    pub nacelles: [f32; 2],
    /// Une montgolfière monte pendant cette image (Pépin à bord, brûleur allumé).
    pub montee: bool,
    /// Phase du premier ventilateur du niveau et pas de souffle qui lui restent
    /// (carte 5.6b ; `None` s'il n'y en a pas) : sert à choisir les images à capturer.
    pub ventilateur: Option<(PhaseVentilateur, u16)>,
    /// Coin haut-gauche des deux premiers escargots du niveau, de gauche à droite
    /// au départ (carte 5.8 ; `(-1, -1)` s'il n'y en a pas), et coquille aplatie.
    pub escargots: [(f32, f32, bool); 2],
    /// Pépin vient de rebondir sur un escargot pendant cette image (carte 5.8).
    pub rebond_escargot: bool,
    /// Pépin touche un escargot sans rebondir (de côté : rien ne se passe).
    pub contre_escargot: bool,
}

/// Résultat de la partie d'essai jouée à travers l'`Application`.
pub struct PartieEssai {
    pub images: Vec<ImageEssai>,
    pub fin_atteinte: bool,
    pub morts: u32,
    pub gouttes: u32,
}

/// Joue un niveau d'essai à travers l'`Application`, comme le ferait le build de
/// QA : « JOUER » au titre (image 0), carte du niveau 1, puis, dès la première
/// image de jeu, la touche du niveau d'essai (`raccourci`) ; la carte du niveau
/// d'essai passe, puis le robot joue `parcours` jusqu'à l'écran de fin.
fn jouer_essai(
    indice_essai: usize,
    raccourci: crate::entrees::RaccourciNiveau,
    parcours: &[Entrees],
) -> PartieEssai {
    let mut app = Application::new(true);
    let mut images = Vec::new();
    let mut appuyee = false;
    let mut fin_atteinte = false;
    for image in 0..60 * 60 * 3 {
        let mut trame = Trame {
            valider: image == 0,
            ..Trame::default()
        };
        let en_essai = app.en_jeu() && app.monde.indice == indice_essai;
        let touche_essai = !appuyee && app.en_jeu() && app.monde.indice == 0;
        appuyee |= touche_essai;
        if en_essai {
            if let Some(e) = parcours.get(app.monde.pas as usize) {
                trame.entrees = *e;
            }
        }
        let nav = Navigation {
            valider: trame.valider,
            ..Navigation::default()
        };
        let raccourci_image = touche_essai.then_some(raccourci);
        app.reagir(&nav, &Commandes::default(), raccourci_image);
        app.pas(&trame.entrees);
        trame.ecran = nom_ecran(app.ecran);
        trame.x = app.monde.joueur.boite.x;
        trame.y = app.monde.joueur.boite.y;
        trame.gouttes = app.monde.etat.gouttes;
        trame.morts = app.monde.etat.morts;
        let j = &app.monde.joueur;
        let camera = app.camera.position_rendue(1.0, (j.boite.x, j.boite.y));
        let pepin_ecran_y = j.boite.y.round() as i32 - camera.1;
        let milieu = j.boite.x.round() as i32 - camera.0 + j.boite.w as i32 / 2;
        let repere = crate::scene::repere_haut(pepin_ecran_y, j.boite.h as i32, milieu).is_some();
        let sous_la_grele = j.plane
            && app.monde.entites.iter().any(|e| {
                matches!(e, pepin_core::entities::Entite::Grele(g) if g.zone.chevauche(&j.boite))
            });
        let rebond_auvent = app.monde.contacts.contains(&Contact::Auvent);
        let au_sol = j.au_sol;
        // Montgolfières rangées de gauche à droite (l'ordre du parcours dans
        // l'essai), et non dans l'ordre de lecture du niveau (de haut en bas).
        let mut montgolfieres: Vec<(f32, f32, bool)> = app
            .monde
            .entites
            .iter()
            .filter_map(|e| match e {
                Entite::Montgolfiere(m) => {
                    Some((m.boite.x, m.boite.y, m.etat() == EtatMontgolfiere::Montee))
                }
                _ => None,
            })
            .collect();
        montgolfieres.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut nacelles = [-1.0f32; 2];
        for (y, m) in nacelles.iter_mut().zip(&montgolfieres) {
            *y = m.1;
        }
        let montee = montgolfieres.iter().any(|m| m.2);
        let ventilateur = app.monde.entites.iter().find_map(|e| match e {
            Entite::Ventilateur(v) => Some((v.phase(), v.souffle_restant())),
            _ => None,
        });
        // Escargots dans l'ordre du niveau (de gauche à droite au départ : l'ordre
        // de lecture d'une même rangée).
        let mut escargots = [(-1.0f32, -1.0f32, false); 2];
        let liste = app.monde.entites.iter().filter_map(|e| match e {
            Entite::Escargot(s) => Some(s),
            _ => None,
        });
        for (place, s) in escargots.iter_mut().zip(liste) {
            *place = (s.boite.x, s.boite.y, s.est_comprime());
        }
        let touche_escargot = app
            .monde
            .entites
            .iter()
            .any(|e| matches!(e, Entite::Escargot(s) if s.boite.chevauche(&j.boite)));
        let rebond_escargot = touche_escargot && app.monde.contacts.contains(&Contact::Rebond);
        let contre_escargot = touche_escargot && !rebond_escargot && j.au_sol;
        images.push(ImageEssai {
            trame,
            touche_essai,
            en_essai: en_essai && app.en_jeu(),
            camera,
            pepin_ecran_y,
            repere,
            sous_la_grele,
            rebond_auvent,
            au_sol,
            nacelles,
            montee,
            ventilateur,
            escargots,
            rebond_escargot,
            contre_escargot,
        });
        if app.ecran == Ecran::Fin {
            fin_atteinte = true;
            break;
        }
    }
    PartieEssai {
        images,
        fin_atteinte,
        morts: app.monde.etat.morts,
        gouttes: app.monde.etat.gouttes,
    }
}

/// Joue le niveau d'essai vertical (touche 0 du build de QA) — voir `jouer_essai`.
pub fn jouer_essai_vertical(parcours: &[Entrees]) -> PartieEssai {
    use crate::entrees::RaccourciNiveau;
    jouer_essai(crate::monde::INDICE_ESSAI, RaccourciNiveau::Essai, parcours)
}

/// Joue le niveau d'essai « vent » (touche F1 du build de QA) — voir `jouer_essai`.
pub fn jouer_essai_vent(parcours: &[Entrees]) -> PartieEssai {
    use crate::entrees::RaccourciNiveau;
    jouer_essai(
        crate::monde::INDICE_ESSAI_VENT,
        RaccourciNiveau::Essai2,
        parcours,
    )
}

/// Parcours du robot dans le niveau d'essai vertical (maladroit, puis précis).
pub fn parcours_essai_vertical() -> Vec<Entrees> {
    let i = crate::monde::INDICE_ESSAI;
    trouver_parcours(i, false)
        .or_else(|| trouver_parcours(i, true))
        .expect("le robot ne trouve aucun parcours au niveau d'essai vertical")
}

/// Parcours du robot dans le niveau d'essai « vent » (maladroit, puis précis) : le
/// plané qui traverse le vide est porté par le vent latéral du niveau.
pub fn parcours_essai_vent() -> Vec<Entrees> {
    let i = crate::monde::INDICE_ESSAI_VENT;
    trouver_parcours(i, false)
        .or_else(|| trouver_parcours(i, true))
        .expect("le robot ne trouve aucun parcours au niveau d'essai vent")
}

/// Joue le niveau d'essai « grêle » (touche F2 du build de QA) — voir `jouer_essai`.
pub fn jouer_essai_grele(parcours: &[Entrees]) -> PartieEssai {
    use crate::entrees::RaccourciNiveau;
    jouer_essai(
        crate::monde::INDICE_ESSAI_GRELE,
        RaccourciNiveau::Essai3,
        parcours,
    )
}

/// Joue le niveau d'essai « auvent » (touche F3 du build de QA) — voir `jouer_essai`.
pub fn jouer_essai_auvent(parcours: &[Entrees]) -> PartieEssai {
    use crate::entrees::RaccourciNiveau;
    jouer_essai(
        crate::monde::INDICE_ESSAI_AUVENT,
        RaccourciNiveau::Essai4,
        parcours,
    )
}

/// Joue le niveau d'essai « ventilateur » (touche F4 du build de QA) — voir
/// `jouer_essai`.
pub fn jouer_essai_ventilateur(parcours: &[Entrees]) -> PartieEssai {
    use crate::entrees::RaccourciNiveau;
    jouer_essai(
        crate::monde::INDICE_ESSAI_VENTILATEUR,
        RaccourciNiveau::Essai5,
        parcours,
    )
}

/// Parcours de démonstration du niveau d'essai « ventilateur » (carte 5.6b, pour les
/// captures) : Pépin attend au départ que le PREMIER souffle soit passé, monte sur
/// la casette pendant l'arrêt, puis ne touche plus à rien. Il y est donc posé
/// pendant toute l'annonce suivante, le souffle le porte jusqu'en haut de la
/// colonne, et il retombe à l'arrêt. Le saut (élan, appui, durée) est cherché
/// sur la vraie simulation : `None` si aucun ne pose Pépin sur la casette.
pub fn parcours_essai_ventilateur_caisse() -> Option<Vec<Entrees>> {
    use crate::monde::{Monde, INDICE_ESSAI_VENTILATEUR};
    // Casette du ventilateur, sa phase.
    let casette = |m: &Monde| {
        m.entites.iter().find_map(|e| match e {
            Entite::Ventilateur(v) => Some((v.boite, v.phase())),
            _ => None,
        })
    };
    // Les pas du cycle, mesurés sur la simulation (sans entrée) : fin du premier
    // souffle, puis début de l'annonce et du souffle suivants.
    let mut m = Monde::new(INDICE_ESSAI_VENTILATEUR);
    let mut pas = 0u32;
    let mut vu_souffle = false;
    let (mut fin_souffle, mut annonce, mut souffle) = (None, None, None);
    while souffle.is_none() && pas < 1000 {
        m.avancer(&Entrees::default());
        pas += 1;
        match casette(&m)?.1 {
            PhaseVentilateur::Souffle if annonce.is_some() => souffle = Some(pas),
            PhaseVentilateur::Souffle => vu_souffle = true,
            PhaseVentilateur::Arret if vu_souffle && fin_souffle.is_none() => {
                fin_souffle = Some(pas)
            }
            PhaseVentilateur::Annonce if fin_souffle.is_some() && annonce.is_none() => {
                annonce = Some(pas)
            }
            _ => {}
        }
    }
    let (fin_souffle, annonce, souffle) = (fin_souffle?, annonce?, souffle?);
    // Préfixe : immobile jusqu'à la fin du premier souffle (quelques pas de marge).
    let attente = fin_souffle as usize + 4;
    let mut depart = Monde::new(INDICE_ESSAI_VENTILATEUR);
    for _ in 0..attente {
        depart.avancer(&Entrees::default());
    }
    for course in 1..40usize {
        for debut_saut in 0..course {
            for duree_saut in 1..16usize {
                let elan: Vec<Entrees> = (0..course)
                    .map(|k| Entrees {
                        direction: Direction::Droite,
                        saut: k >= debut_saut && k < debut_saut + duree_saut,
                    })
                    .collect();
                let mut essai = depart.clone();
                let mut pas = attente as u32;
                for e in &elan {
                    essai.avancer(e);
                    pas += 1;
                }
                if pas >= annonce {
                    continue;
                }
                // Immobile jusqu'au souffle : Pépin doit être posé sur la casette
                // pendant TOUTE l'annonce.
                let mut pose = true;
                while pas < souffle {
                    essai.avancer(&Entrees::default());
                    pas += 1;
                    let boite = casette(&essai)?.0;
                    let j = &essai.joueur;
                    if (annonce..souffle).contains(&pas)
                        && !(j.au_sol
                            && (j.boite.bas() - boite.y).abs() < 0.5
                            && j.boite.x >= boite.x
                            && j.boite.x + j.boite.w <= boite.x + boite.w)
                    {
                        pose = false;
                        break;
                    }
                }
                if !pose || essai.etat.morts > 0 {
                    continue;
                }
                let mut parcours = vec![Entrees::default(); attente];
                parcours.extend(elan);
                return Some(parcours);
            }
        }
    }
    None
}

/// Parcours du robot dans le niveau d'essai « auvent » (maladroit, puis précis) :
/// chaque rebond d'auvent est un point de décision (`Fin::Lance`), d'où il essaie
/// les plans en vol (`plans_en_vol`). Il cherche le plus RAPIDE : en gardant la
/// droite, il saute des auvents (voir `parcours_essai_auvent_rebond_par_rebond`).
pub fn parcours_essai_auvent() -> Vec<Entrees> {
    let i = crate::monde::INDICE_ESSAI_AUVENT;
    trouver_parcours(i, false)
        .or_else(|| trouver_parcours(i, true))
        .expect("le robot ne trouve aucun parcours au niveau d'essai auvent")
}

/// Parcours « rebond par rebond » du niveau d'essai « auvent » : le robot vise
/// chaque auvent à son tour (de gauche à droite), sans en sauter aucun, puis le
/// drapeau. Chaque morceau est un plus court chemin (`chercher`) dont la cible est
/// « vient d'être relancé par CET auvent ». C'est le moment fort du marché couvert
/// (six auvents à la suite sans toucher le sol), joué par le robot pour les captures.
pub fn parcours_essai_auvent_rebond_par_rebond() -> Vec<Entrees> {
    let (mut monde, mut entrees) = depart_pose(crate::monde::INDICE_ESSAI_AUVENT);
    let mut auvents: Vec<pepin_core::aabb::Aabb> = monde
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Auvent(a) => Some(a.boite),
            _ => None,
        })
        .collect();
    auvents.sort_by(|a, b| a.x.total_cmp(&b.x));
    let tous = plans(false);
    let mut jouer = |monde: &mut Monde, etapes: Vec<Etape>| {
        for e in etapes {
            for x in &e.entrees {
                monde.avancer(x);
            }
            entrees.extend(e.entrees);
        }
    };
    for cible in auvents {
        // Relancé par CET auvent : posé sur sa toile, en vol, élan tout neuf.
        let relance = |m: &Monde| {
            let b = m.joueur.boite;
            est_lance(m)
                && (b.bas() - cible.y).abs() < 0.01
                && b.x < cible.droite()
                && cible.x < b.droite()
        };
        let etapes = chercher(&monde, &tous, &relance, usize::MAX)
            .expect("le robot ne sait pas atteindre l'auvent suivant");
        jouer(&mut monde, etapes);
    }
    let etapes = chercher(&monde, &tous, &|_| false, usize::MAX)
        .expect("le robot ne sait pas finir après le dernier auvent");
    jouer(&mut monde, etapes);
    entrees
}

/// Parcours du robot dans le niveau d'essai « grêle » (maladroit, puis précis). La
/// grêle n'a rien de spécial pour lui : la VRAIE simulation (`Monde`) tue un plan qui
/// la touche parapluie fermé, ce plan est écarté, et il reste ceux qui la traversent
/// en planant (ou qui passent sous la corniche).
pub fn parcours_essai_grele() -> Vec<Entrees> {
    let i = crate::monde::INDICE_ESSAI_GRELE;
    trouver_parcours(i, false)
        .or_else(|| trouver_parcours(i, true))
        .expect("le robot ne trouve aucun parcours au niveau d'essai grêle")
}

/// Parcours du robot dans le niveau d'essai « ventilateur » (maladroit, puis précis).
/// Le ventilateur porte Pépin d'un étage entier ; le robot apprend à attendre le
/// souffle sur la casette, puis à marcher sur le balcon qu'il atteint.
pub fn parcours_essai_ventilateur() -> Vec<Entrees> {
    let i = crate::monde::INDICE_ESSAI_VENTILATEUR;
    trouver_parcours(i, false)
        .or_else(|| trouver_parcours(i, true))
        .expect("le robot ne trouve aucun parcours au niveau d'essai ventilateur")
}

/// Joue le niveau d'essai « montgolfière » (touche F5 du build de QA) — voir
/// `jouer_essai`.
pub fn jouer_essai_montgolfiere(parcours: &[Entrees]) -> PartieEssai {
    use crate::entrees::RaccourciNiveau;
    jouer_essai(
        crate::monde::INDICE_ESSAI_MONTGOLFIERE,
        RaccourciNiveau::Essai6,
        parcours,
    )
}

/// Parcours du robot dans le niveau d'essai « montgolfière » (maladroit, puis
/// précis). Rien de spécial pour lui : sur la VRAIE simulation, un plan qui finit
/// dans la nacelle la fait monter ; il apprend qu'il faut y attendre (plans
/// « attendre ») jusqu'à ce qu'elle s'arrête en haut, puis avancer vers l'étage.
/// Un Pépin trop au bord de la nacelle se cognerait la tête et l'arrêterait plus
/// bas : ces plans-là ne mènent nulle part et sont écartés.
pub fn parcours_essai_montgolfiere() -> Vec<Entrees> {
    let i = crate::monde::INDICE_ESSAI_MONTGOLFIERE;
    trouver_parcours(i, false)
        .or_else(|| trouver_parcours(i, true))
        .expect("le robot ne trouve aucun parcours au niveau d'essai montgolfière")
}

/// Parcours « jusqu'en haut » du niveau d'essai « montgolfière » : le robot monte
/// dans chaque montgolfière à son tour (de gauche à droite) et y reste jusqu'à ce
/// qu'elle s'arrête EN HAUT (8 tuiles pour la première, sous le nuage pour la
/// seconde), puis va au drapeau. Chaque morceau est un plus court chemin
/// (`chercher`) dont la cible est « posé dans CETTE nacelle, arrivée en haut ». Le
/// parcours le plus rapide, lui, saute de la nacelle avant le haut : celui-ci montre
/// toute la mécanique (montée, arrêt en haut, redescente à vide) pour les captures.
pub fn parcours_essai_montgolfiere_jusqu_en_haut() -> Vec<Entrees> {
    let (mut monde, mut entrees) = depart_pose(crate::monde::INDICE_ESSAI_MONTGOLFIERE);
    let mut nacelles: Vec<(f32, f32)> = monde
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Montgolfiere(m) => Some((m.boite.x, m.altitudes().1)),
            _ => None,
        })
        .collect();
    nacelles.sort_by(|a, b| a.0.total_cmp(&b.0));
    let tous = plans(false);
    let mut jouer = |monde: &mut Monde, etapes: Vec<Etape>| {
        for e in etapes {
            for x in &e.entrees {
                monde.avancer(x);
            }
            entrees.extend(e.entrees);
        }
    };
    for (x, haut) in nacelles {
        // Posé dans CETTE nacelle, arrivée tout en haut.
        let en_haut = |m: &Monde| {
            m.joueur.au_sol
                && m.entites.iter().any(|e| {
                    matches!(e, Entite::Montgolfiere(n)
                        if n.boite.x == x && n.boite.y == haut && n.a_un_passager())
                })
        };
        let etapes = chercher(&monde, &tous, &en_haut, usize::MAX)
            .expect("le robot ne sait pas monter en haut de la montgolfière suivante");
        jouer(&mut monde, etapes);
    }
    let etapes = chercher(&monde, &tous, &|_| false, usize::MAX)
        .expect("le robot ne sait pas finir après la dernière montgolfière");
    jouer(&mut monde, etapes);
    entrees
}

/// Joue le niveau d'essai « escargot » (touche F6 du build de QA) — voir
/// `jouer_essai`.
pub fn jouer_essai_escargot(parcours: &[Entrees]) -> PartieEssai {
    use crate::entrees::RaccourciNiveau;
    jouer_essai(
        crate::monde::INDICE_ESSAI_ESCARGOT,
        RaccourciNiveau::Essai7,
        parcours,
    )
}

/// Parcours du robot dans le niveau d'essai « escargot » (maladroit, puis précis).
/// Rien de spécial pour lui : sur la VRAIE simulation, un plan qui retombe sur la
/// coquille de l'escargot le fait rebondir. Il apprend où et quand sauter (plans
/// « attendre », élans) pour que l'escargot soit sous lui au bon moment, au pied
/// de chaque falaise ; un plan où l'escargot est déjà passé retombe au sol et ne
/// mène nulle part.
pub fn parcours_essai_escargot() -> Vec<Entrees> {
    let i = crate::monde::INDICE_ESSAI_ESCARGOT;
    trouver_parcours(i, false)
        .or_else(|| trouver_parcours(i, true))
        .expect("le robot ne trouve aucun parcours au niveau d'essai escargot")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le robot finit aussi le niveau d'essai « vent » (un écran, carte 5.3) sans
    /// mourir, à travers l'`Application` (touche F1 du build de QA). Le plané qui
    /// traverse le vide est porté par le vent latéral du niveau ; la caméra reste
    /// au sol (niveau d'un écran) et les trois gouttes sont ramassées.
    #[test]
    fn le_robot_finit_le_niveau_d_essai_vent() {
        let parcours = parcours_essai_vent();
        let partie = jouer_essai_vent(&parcours);
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
        assert_eq!(
            partie.gouttes, 3,
            "les trois gouttes du niveau d'essai vent"
        );
        assert_eq!(
            partie.images.iter().filter(|i| i.touche_essai).count(),
            1,
            "la touche F1, une seule fois"
        );
        let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
        assert!(essai.len() > 100, "{} images de jeu", essai.len());
        // Niveau d'un écran : la caméra reste au bas du niveau (y = -4 : le niveau de
        // 176 px est aligné au bas de l'écran de 180 px, le ciel remplit le reste).
        assert!(
            essai.iter().all(|i| i.camera.1 == -4),
            "la caméra d'un niveau plat reste au sol"
        );
        // Le robot a bel et bien traversé le vide : il arrive à droite de la zone de
        // vent (x > 400), où se trouve le drapeau.
        let dernier = essai.last().unwrap();
        assert!(
            dernier.trame.x > 400.0,
            "arrivée à x = {} (le drapeau est à droite du vide)",
            dernier.trame.x
        );
        println!(
            "essai vent : {} images de jeu, gouttes {}, fin à x = {:.1}",
            essai.len(),
            partie.gouttes,
            dernier.trame.x
        );
    }

    /// Le robot finit le niveau d'essai « grêle » (carte 5.4) sans mourir, à travers
    /// l'`Application` (touche F2 du build de QA) : trois gouttes (dont celle de la
    /// corniche, SOUS la grêle arrêtée), et il a bel et bien plané DANS la grêle.
    #[test]
    fn le_robot_finit_le_niveau_d_essai_grele() {
        let parcours = parcours_essai_grele();
        let partie = jouer_essai_grele(&parcours);
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
        assert_eq!(
            partie.gouttes, 3,
            "les trois gouttes du niveau d'essai grêle"
        );
        assert_eq!(
            partie.images.iter().filter(|i| i.touche_essai).count(),
            1,
            "la touche F2, une seule fois"
        );
        let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
        assert!(essai.len() > 100, "{} images de jeu", essai.len());
        assert!(
            essai.iter().all(|i| i.camera.1 == -4),
            "la caméra d'un niveau plat reste au sol"
        );
        let sous_la_grele = essai.iter().filter(|i| i.sous_la_grele).count();
        assert!(
            sous_la_grele > 20,
            "{sous_la_grele} images à planer dans la grêle"
        );
        let dernier = essai.last().unwrap();
        assert!(dernier.trame.x > 500.0, "arrivée à x = {}", dernier.trame.x);
        println!(
            "essai grêle : {} images de jeu, {sous_la_grele} sous la grêle, gouttes {}, fin à x = {:.1}",
            essai.len(),
            partie.gouttes,
            dernier.trame.x
        );
    }

    /// Rebonds d'auvent d'une partie d'essai : indices (dans les images de jeu) des
    /// images où un auvent relance Pépin.
    fn rebonds(essai: &[&ImageEssai]) -> Vec<usize> {
        (0..essai.len())
            .filter(|&k| essai[k].rebond_auvent)
            .collect()
    }

    /// Le robot (le plus rapide) finit le niveau d'essai « auvent » (carte 5.5) sans
    /// mourir, à travers l'`Application` (touche F3 du build de QA) : il traverse le
    /// vide par les auvents sans toucher le sol entre son premier et son dernier
    /// rebond (en gardant la droite, il en saute certains : c'est permis).
    #[test]
    fn le_robot_finit_le_niveau_d_essai_auvent() {
        let partie = jouer_essai_auvent(&parcours_essai_auvent());
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
        assert_eq!(
            partie.images.iter().filter(|i| i.touche_essai).count(),
            1,
            "la touche F3, une seule fois"
        );
        let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
        let r = rebonds(&essai);
        assert!(r.len() >= 3, "{} rebonds d'auvent", r.len());
        let (premier, dernier) = (r[0], *r.last().unwrap());
        assert!(
            essai[premier..=dernier].iter().all(|i| !i.au_sol),
            "Pépin a touché le sol entre deux auvents"
        );
        println!(
            "essai auvent (le plus rapide) : {} images de jeu, {} rebonds, gouttes {}",
            essai.len(),
            r.len(),
            partie.gouttes
        );
    }

    /// Rebond par rebond (le parcours des captures) : les SIX auvents, chacun à son
    /// tour, sans toucher le sol du premier au dernier ; trois gouttes (dont celles
    /// en l'air, entre deux rebonds), aucune mort.
    #[test]
    fn le_robot_enchaine_les_six_auvents_de_l_essai() {
        let partie = jouer_essai_auvent(&parcours_essai_auvent_rebond_par_rebond());
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
        assert_eq!(
            partie.gouttes, 3,
            "les trois gouttes du niveau d'essai auvent"
        );
        let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
        let r = rebonds(&essai);
        // Les six auvents (colonnes 9, 13, 17, 22, 25, 29), un rebond chacun, dans
        // l'ordre : l'auvent relanceur est celui que la boîte de Pépin (10 px) touche.
        let colonnes: Vec<i32> = r
            .iter()
            .filter_map(|&k| {
                let x = essai[k].trame.x;
                [9, 13, 17, 22, 25, 29]
                    .into_iter()
                    .find(|&c| x < (c * 16 + 16) as f32 && ((c * 16) as f32) < x + 10.0)
            })
            .collect();
        assert_eq!(
            colonnes,
            vec![9, 13, 17, 22, 25, 29],
            "rebonds aux images {r:?}"
        );
        assert!(
            essai[r[0]..=r[5]].iter().all(|i| !i.au_sol),
            "Pépin a touché le sol entre deux auvents"
        );
        println!(
            "essai auvent (rebond par rebond) : {} images de jeu, rebonds aux images {r:?}",
            essai.len()
        );
    }

    /// Le robot finit le niveau d'essai « ventilateur » (carte 5.6) sans mourir, à
    /// travers l'`Application` (touche F4 du build de QA) : trois gouttes, et il
    /// monte réellement au balcon du drapeau — le seul chemin passe par le souffle
    /// du ventilateur, car le balcon est trop haut pour un saut direct.
    #[test]
    fn le_robot_finit_le_niveau_d_essai_ventilateur() {
        let partie = jouer_essai_ventilateur(&parcours_essai_ventilateur());
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
        assert_eq!(
            partie.gouttes, 3,
            "les trois gouttes du niveau d'essai ventilateur"
        );
        assert_eq!(
            partie.images.iter().filter(|i| i.touche_essai).count(),
            1,
            "la touche F4, une seule fois"
        );
        let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
        assert!(essai.len() > 100, "{} images de jeu", essai.len());
        // Le balcon du drapeau est en HAUT du niveau (y petit). Le robot y est bien
        // arrivé : sa trame finale est nettement plus haute que le sol.
        let dernier = essai.last().unwrap();
        assert!(
            dernier.trame.y < 100.0,
            "arrivée en haut du niveau (y = {:.1}) : le balcon a été atteint",
            dernier.trame.y
        );
        println!(
            "essai ventilateur : {} images de jeu, gouttes {}, fin à x = {:.1}, y = {:.1}",
            essai.len(),
            partie.gouttes,
            dernier.trame.x,
            dernier.trame.y
        );
    }

    /// Le robot finit le niveau d'essai « montgolfière » (carte 5.7) sans mourir, à
    /// travers l'`Application` (touche F5 du build de QA), par le parcours le plus
    /// RAPIDE : trois gouttes, l'arrivée tout en haut (étage du drapeau). Il saute de
    /// la nacelle dès que l'étage suivant est à portée de saut, mais il a dû y monter :
    /// la première l'a porté de plus d'un saut (56 px). Pendant toute montée où il est
    /// posé, ses pieds sont PILE sur le dessus d'une nacelle (porté, sans glisser).
    #[test]
    fn le_robot_finit_le_niveau_d_essai_montgolfiere() {
        let partie = jouer_essai_montgolfiere(&parcours_essai_montgolfiere());
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
        assert_eq!(
            partie.gouttes, 3,
            "les trois gouttes de l'essai montgolfière"
        );
        assert_eq!(
            partie.images.iter().filter(|i| i.touche_essai).count(),
            1,
            "la touche F5, une seule fois"
        );
        let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
        let dernier = essai.last().unwrap();
        assert!(
            dernier.trame.y < 80.0,
            "arrivée à l'étage du drapeau (y = {:.1})",
            dernier.trame.y
        );
        let plus_haut = |k: usize| essai.iter().map(|i| i.nacelles[k]).fold(f32::MAX, f32::min);
        assert!(plus_haut(0) < 256.0 - 48.0, "nacelle 1 : {}", plus_haut(0));
        let portees: Vec<&&ImageEssai> = essai.iter().filter(|i| i.montee && i.au_sol).collect();
        assert!(portees.len() > 60, "{} images portées", portees.len());
        for i in &portees {
            let pieds = i.trame.y + pepin_core::physics::JOUEUR_H;
            assert!(
                i.nacelles.iter().any(|&n| (pieds - n).abs() < 0.01),
                "pieds en y = {pieds}, nacelles en y = {:?}",
                i.nacelles
            );
        }
        println!(
            "essai montgolfière (le plus rapide) : {} images de jeu, {} portées, plus haut : \
             {:.1} et {:.1}",
            essai.len(),
            portees.len(),
            plus_haut(0),
            plus_haut(1)
        );
    }

    /// Le parcours « jusqu'en haut » (celui des captures) : chaque montgolfière monte
    /// JUSQU'EN HAUT puis s'arrête — la première de 8 tuiles (nacelle de y = 256 à
    /// y = 128), la seconde sous le nuage (de y = 144 à y = 64) —, Pépin y est
    /// toujours posé pendant la montée, et la première, quittée, redescend vers son
    /// quai pendant que Pépin monte dans la seconde. Trois gouttes, aucune mort.
    #[test]
    fn le_robot_monte_jusqu_en_haut_des_deux_montgolfieres() {
        let partie = jouer_essai_montgolfiere(&parcours_essai_montgolfiere_jusqu_en_haut());
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
        assert_eq!(partie.gouttes, 3);
        let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
        let dernier = essai.last().unwrap();
        let plus_haut = |k: usize| essai.iter().map(|i| i.nacelles[k]).fold(f32::MAX, f32::min);
        assert_eq!(plus_haut(0), 128.0, "la première monte de 8 tuiles");
        assert_eq!(plus_haut(1), 64.0, "la seconde s'arrête sous le nuage");
        let montees = essai.iter().filter(|i| i.montee).count();
        assert!(
            montees >= 260,
            "{montees} images de montée (128 + 80 px à 0,8 px : 260)"
        );
        // Une montée où Pépin n'est pas posé ne dure qu'une image : celle où il
        // saute de la nacelle (elle montait encore au début du pas). Dès l'image
        // suivante, plus aucune nacelle ne monte.
        for (k, i) in essai.iter().enumerate() {
            if i.montee && !i.au_sol {
                assert!(
                    essai.get(k + 1).is_none_or(|s| !s.montee),
                    "image {k} : Pépin en l'air pendant une montée"
                );
            }
        }
        assert!(
            dernier.nacelles[0] > 128.0 + 20.0,
            "quittée, la première redescend (nacelle en y = {:.1})",
            dernier.nacelles[0]
        );
        println!(
            "essai montgolfière (jusqu'en haut) : {} images de jeu, {montees} de montée, \
             nacelles à la fin en y = {:?}",
            essai.len(),
            dernier.nacelles
        );
    }

    /// Le robot finit le niveau d'essai « escargot » (touche F6 du build de QA) sans
    /// mourir : trois gouttes, l'arrivée en haut de la falaise 2 (y < 48). Il a
    /// rebondi au moins deux fois sur un escargot (une par falaise) ; chaque rebond
    /// relance Pépin vers le haut et aplatit la coquille de l'escargot touché.
    #[test]
    fn le_robot_finit_le_niveau_d_essai_escargot() {
        let partie = jouer_essai_escargot(&parcours_essai_escargot());
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
        assert_eq!(partie.gouttes, 3, "les trois gouttes de l'essai escargot");
        assert_eq!(
            partie.images.iter().filter(|i| i.touche_essai).count(),
            1,
            "la touche F6, une seule fois"
        );
        let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
        let dernier = essai.last().unwrap();
        assert!(
            dernier.trame.y < 48.0,
            "arrivée en haut de la falaise 2 (y = {:.1})",
            dernier.trame.y
        );
        let rebonds: Vec<usize> = (0..essai.len())
            .filter(|&k| essai[k].rebond_escargot)
            .collect();
        assert!(rebonds.len() >= 2, "{} rebond(s)", rebonds.len());
        for &k in &rebonds {
            assert!(
                essai[k].escargots.iter().any(|e| e.2),
                "image {k} : rebond sans coquille aplatie"
            );
        }
        let contre = essai.iter().filter(|i| i.contre_escargot).count();
        println!(
            "essai escargot : {} images de jeu, rebonds aux images {rebonds:?}, {contre} \
             image(s) au sol contre un escargot",
            essai.len()
        );
    }

    /// Le robot finit la partie sans mourir, et joue les niveaux dans l'ordre que
    /// donne `levels::apres` avec SES gouttes (bonus sauté s'il en manque).
    #[test]
    fn le_robot_finit_tous_les_niveaux_sans_mourir() {
        use pepin_core::levels::{apres, Apres, NIVEAUX};
        let parcours = tous_les_parcours();
        let partie = jouer_tout(&parcours);
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
        // Niveaux joués (numéros), dans l'ordre, et gouttes à la fin de chacun.
        let mut joues: Vec<(u8, u32)> = Vec::new();
        for t in partie.trames.iter().filter(|t| t.niveau > 0) {
            match joues.last_mut() {
                Some((n, g)) if *n == t.niveau => *g = t.gouttes,
                _ => joues.push((t.niveau, t.gouttes)),
            }
        }
        let mut attendu = vec![1u8];
        for &(n, g) in &joues {
            if let Apres::Niveau(j) = apres(NIVEAUX, n as usize - 1, g) {
                attendu.push(j as u8 + 1);
            }
        }
        let numeros: Vec<u8> = joues.iter().map(|&(n, _)| n).collect();
        assert_eq!(numeros, attendu);
    }

    /// Le robot finit aussi le niveau d'essai vertical (45 rangées) sans mourir, à travers
    /// l'`Application` (touche 0 du build de QA). Pendant tout le parcours, la caméra suit
    /// Pépin en hauteur : elle part de sa borne basse, monte jusqu'en haut du niveau et
    /// redescend ; Pépin est TOUJOURS entier à l'écran, sauf quand il est lancé au-dessus
    /// du niveau, et alors la flèche de repère est là (et seulement alors).
    #[test]
    fn le_robot_finit_le_niveau_d_essai_vertical() {
        let parcours = parcours_essai_vertical();
        let partie = jouer_essai_vertical(&parcours);
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
        assert_eq!(
            partie.images.iter().filter(|i| i.touche_essai).count(),
            1,
            "la touche 0, une seule fois"
        );
        let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
        assert!(essai.len() > 300, "{} images de jeu", essai.len());
        let ys: Vec<i32> = essai.iter().map(|i| i.camera.1).collect();
        let (haut, bas) = (*ys.iter().min().unwrap(), *ys.iter().max().unwrap());
        assert_eq!((haut, bas), (0, 540), "la caméra parcourt toute la hauteur");
        assert_eq!(ys[0], 540, "départ en bas");
        let sommet = ys.iter().position(|&y| y == 0).unwrap();
        assert!(ys[sommet..].contains(&540), "puis redescend jusqu'en bas");
        let mut reperes = 0;
        for (k, i) in essai.iter().enumerate() {
            let (y, h) = (i.pepin_ecran_y, 14);
            assert!(y + h <= 180, "image {k} : Pépin sous l'écran (y = {y})");
            if i.camera.1 > 0 {
                // Caméra sous sa borne haute : Pépin est entier à l'écran, sans flèche.
                assert!(
                    y >= 0 && !i.repere,
                    "image {k} : Pépin hors de l'écran (y = {y}, caméra {:?})",
                    i.camera
                );
            } else {
                // Caméra en haut du niveau : la flèche apparaît dès que Pépin est
                // entièrement au-dessus du cadre, jamais avant.
                assert_eq!(i.repere, y + h <= 0, "image {k} : y = {y}");
            }
            reperes += usize::from(i.repere);
        }
        assert!(reperes > 0, "le champignon lance Pépin au-dessus du niveau");
        println!(
            "essai vertical : {} images de jeu, {reperes} avec la flèche, gouttes {}",
            essai.len(),
            partie.gouttes
        );
    }

    /// Écrit le script du niveau d'essai vertical pour le navigateur (build de QA) :
    /// `target/qa/script-52.txt` (un chiffre hexadécimal par image, comme `script.txt`),
    /// `target/qa/touches-52.txt` (image du pilote où appuyer sur la touche 0) et
    /// `target/qa/trace-52.csv` (position de Pépin et caméra rendue, image par image).
    /// Dans le navigateur, le jeu a un pas de retard sur la numérotation du pilote (déjà
    /// noté à la carte 3.1b, voir QA.md) : sans effet sur des touches tenues plusieurs
    /// images, mais une touche d'UNE image doit tomber une image plus tard que dans la
    /// trace, sinon elle arrive pendant la dernière image de la carte du niveau 1 et le jeu
    /// l'ignore (vérifié : à l'image 228, rien ; à 229, la carte « NIVEAU D'ESSAI »).
    /// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai
    #[test]
    #[ignore]
    fn ecrire_script_essai_vertical() {
        let partie = jouer_essai_vertical(&parcours_essai_vertical());
        assert!(partie.fin_atteinte && partie.morts == 0);
        std::fs::create_dir_all("../target/qa").unwrap();
        let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
        std::fs::write("../target/qa/script-52.txt", &script).unwrap();
        let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
        let pilote = touche + 1;
        std::fs::write("../target/qa/touches-52.txt", format!("{pilote}:Digit0\n")).unwrap();
        let mut trace = String::from("image,ecran,essai,x,y,cam_x,cam_y,pepin_ecran_y,repere\n");
        for (k, i) in partie.images.iter().enumerate() {
            trace.push_str(&format!(
                "{k},{},{},{:.1},{:.1},{},{},{},{}\n",
                i.trame.ecran,
                u8::from(i.en_essai),
                i.trame.x,
                i.trame.y,
                i.camera.0,
                i.camera.1,
                i.pepin_ecran_y,
                u8::from(i.repere)
            ));
        }
        std::fs::write("../target/qa/trace-52.csv", trace).unwrap();
        println!(
            "images : {} ; touche 0 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
            partie.images.len(),
            partie.gouttes
        );
    }

    /// Écrit le script du niveau d'essai « vent » pour le navigateur (build de QA) :
    /// `target/qa/script-53.txt`, `target/qa/touches-53.txt` (image du pilote où
    /// appuyer sur F1) et `target/qa/trace-53.csv` (position de Pépin et caméra
    /// rendue, image par image). Comme la carte 5.2, la touche du pilote tombe une
    /// image plus tard que dans la trace (le navigateur a un pas de retard).
    /// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_vent
    #[test]
    #[ignore]
    fn ecrire_script_essai_vent() {
        let partie = jouer_essai_vent(&parcours_essai_vent());
        assert!(partie.fin_atteinte && partie.morts == 0);
        std::fs::create_dir_all("../target/qa").unwrap();
        let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
        std::fs::write("../target/qa/script-53.txt", &script).unwrap();
        let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
        let pilote = touche + 1;
        std::fs::write("../target/qa/touches-53.txt", format!("{pilote}:F1\n")).unwrap();
        let mut trace = String::from("image,ecran,essai,x,y,cam_x,cam_y\n");
        for (k, i) in partie.images.iter().enumerate() {
            trace.push_str(&format!(
                "{k},{},{},{:.1},{:.1},{},{}\n",
                i.trame.ecran,
                u8::from(i.en_essai),
                i.trame.x,
                i.trame.y,
                i.camera.0,
                i.camera.1
            ));
        }
        std::fs::write("../target/qa/trace-53.csv", trace).unwrap();
        println!(
            "images : {} ; F1 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
            partie.images.len(),
            partie.gouttes
        );
    }

    /// Écrit le script du niveau d'essai « grêle » pour le navigateur (build de QA) :
    /// `target/qa/script-54.txt`, `target/qa/touches-54.txt` (image du pilote où
    /// appuyer sur F2, une image après la trace : le navigateur a un pas de retard) et
    /// `target/qa/trace-54.csv` (position de Pépin, caméra, plané dans la grêle).
    /// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_grele
    #[test]
    #[ignore]
    fn ecrire_script_essai_grele() {
        let partie = jouer_essai_grele(&parcours_essai_grele());
        assert!(partie.fin_atteinte && partie.morts == 0);
        std::fs::create_dir_all("../target/qa").unwrap();
        let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
        std::fs::write("../target/qa/script-54.txt", &script).unwrap();
        let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
        let pilote = touche + 1;
        std::fs::write("../target/qa/touches-54.txt", format!("{pilote}:F2\n")).unwrap();
        let mut trace = String::from("image,ecran,essai,x,y,cam_x,cam_y,sous_la_grele\n");
        for (k, i) in partie.images.iter().enumerate() {
            trace.push_str(&format!(
                "{k},{},{},{:.1},{:.1},{},{},{}\n",
                i.trame.ecran,
                u8::from(i.en_essai),
                i.trame.x,
                i.trame.y,
                i.camera.0,
                i.camera.1,
                u8::from(i.sous_la_grele)
            ));
        }
        std::fs::write("../target/qa/trace-54.csv", trace).unwrap();
        println!(
            "images : {} ; F2 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
            partie.images.len(),
            partie.gouttes
        );
    }

    /// Écrit le script du niveau d'essai « auvent » (parcours rebond par rebond : les
    /// six auvents à la suite) pour le navigateur :
    /// `target/qa/script-55.txt`, `target/qa/touches-55.txt` (image du pilote où
    /// appuyer sur F3, une image après la trace : le navigateur a un pas de retard) et
    /// `target/qa/trace-55.csv` (position de Pépin, caméra, rebonds d'auvent, sol).
    /// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_auvent
    #[test]
    #[ignore]
    fn ecrire_script_essai_auvent() {
        let partie = jouer_essai_auvent(&parcours_essai_auvent_rebond_par_rebond());
        assert!(partie.fin_atteinte && partie.morts == 0);
        std::fs::create_dir_all("../target/qa").unwrap();
        let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
        std::fs::write("../target/qa/script-55.txt", &script).unwrap();
        let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
        let pilote = touche + 1;
        std::fs::write("../target/qa/touches-55.txt", format!("{pilote}:F3\n")).unwrap();
        let mut trace = String::from("image,ecran,essai,x,y,cam_x,cam_y,rebond_auvent,au_sol\n");
        for (k, i) in partie.images.iter().enumerate() {
            trace.push_str(&format!(
                "{k},{},{},{:.1},{:.1},{},{},{},{}\n",
                i.trame.ecran,
                u8::from(i.en_essai),
                i.trame.x,
                i.trame.y,
                i.camera.0,
                i.camera.1,
                u8::from(i.rebond_auvent),
                u8::from(i.au_sol)
            ));
        }
        std::fs::write("../target/qa/trace-55.csv", trace).unwrap();
        println!(
            "images : {} ; F3 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
            partie.images.len(),
            partie.gouttes
        );
    }

    /// Écrit le script du niveau d'essai « ventilateur » (parcours du robot) pour le
    /// navigateur : `target/qa/script-56b.txt`, `target/qa/touches-56b.txt` (image du
    /// pilote où appuyer sur F4, une image après la trace : le navigateur a un pas de
    /// retard) et `target/qa/trace-56b.csv` (position de Pépin, caméra, sol, phase du
    /// ventilateur et pas de souffle restants) : de quoi choisir les images à capturer
    /// pendant le souffle, juste après l'arrêt et pendant l'annonce (carte 5.6b).
    /// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_ventilateur
    #[test]
    #[ignore]
    fn ecrire_script_essai_ventilateur() {
        let partie = jouer_essai_ventilateur(&parcours_essai_ventilateur());
        assert!(partie.fin_atteinte && partie.morts == 0);
        std::fs::create_dir_all("../target/qa").unwrap();
        let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
        std::fs::write("../target/qa/script-56b.txt", &script).unwrap();
        let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
        let pilote = touche + 1;
        std::fs::write("../target/qa/touches-56b.txt", format!("{pilote}:F4\n")).unwrap();
        let mut trace =
            String::from("image,ecran,essai,x,y,cam_x,cam_y,au_sol,phase,souffle_restant\n");
        for (k, i) in partie.images.iter().enumerate() {
            let (phase, restant) = match i.ventilateur {
                Some((PhaseVentilateur::Arret, _)) => ("arret", 0),
                Some((PhaseVentilateur::Annonce, _)) => ("annonce", 0),
                Some((PhaseVentilateur::Souffle, r)) => ("souffle", r),
                None => ("-", 0),
            };
            trace.push_str(&format!(
                "{k},{},{},{:.1},{:.1},{},{},{},{phase},{restant}\n",
                i.trame.ecran,
                u8::from(i.en_essai),
                i.trame.x,
                i.trame.y,
                i.camera.0,
                i.camera.1,
                u8::from(i.au_sol)
            ));
        }
        std::fs::write("../target/qa/trace-56b.csv", trace).unwrap();
        println!(
            "images : {} ; F4 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
            partie.images.len(),
            partie.gouttes
        );
    }

    /// Démonstration de la carte 5.6b : Pépin posé sur la casette pendant toute
    /// l'annonce, porté par le souffle suivant jusqu'en haut de la colonne, puis
    /// redescendu à l'arrêt, sans mourir.
    #[test]
    fn pepin_attend_sur_la_casette_puis_le_souffle_le_porte() {
        let parcours =
            parcours_essai_ventilateur_caisse().expect("un saut pose Pépin sur la casette");
        let partie = jouer_essai_ventilateur(&parcours);
        let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
        let phase = |i: &ImageEssai| i.ventilateur.map(|v| v.0);
        // L'annonce qui suit la montée sur la casette : Pépin y est posé.
        let fin_elan = parcours.len();
        let annonce: Vec<&&ImageEssai> = essai
            .iter()
            .skip(fin_elan)
            .take_while(|i| phase(i) != Some(PhaseVentilateur::Souffle))
            .filter(|i| phase(i) == Some(PhaseVentilateur::Annonce))
            .collect();
        assert!(!annonce.is_empty(), "une annonce après l'élan");
        assert!(annonce
            .iter()
            .all(|i| i.au_sol && (i.trame.y - 298.0).abs() < 0.5));
        // Le souffle suivant le porte haut (au moins 10 tuiles), puis, à l'arrêt, il
        // retombe sur la casette.
        let (k_haut, haut) = essai
            .iter()
            .enumerate()
            .skip(fin_elan)
            .map(|(k, i)| (k, i.trame.y))
            .fold((0, f32::MAX), |a, b| if b.1 < a.1 { b } else { a });
        assert!(haut < 298.0 - 160.0, "porté jusqu'à y = {haut}");
        assert!(
            essai[k_haut..]
                .iter()
                .any(|i| i.au_sol && (i.trame.y - 298.0).abs() < 0.5),
            "Pépin retombe sur la casette"
        );
        assert_eq!(partie.morts, 0);
    }

    /// Écrit le script de la démonstration « Pépin sur la casette » (carte 5.6b) :
    /// `target/qa/script-56b-caisse.txt` (1 200 images : la montée sur la casette
    /// puis deux cycles du ventilateur) et `target/qa/trace-56b-caisse.csv` (même
    /// format que `trace-56b.csv`). Touche F4 : la même image que
    /// `touches-56b.txt` (les images avant le niveau d'essai sont les mêmes).
    /// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_ventilateur_caisse
    #[test]
    #[ignore]
    fn ecrire_script_essai_ventilateur_caisse() {
        let parcours =
            parcours_essai_ventilateur_caisse().expect("un saut pose Pépin sur la casette");
        let partie = jouer_essai_ventilateur(&parcours);
        let images = &partie.images[..partie.images.len().min(1200)];
        std::fs::create_dir_all("../target/qa").unwrap();
        let script: String = images.iter().map(|i| i.trame.code()).collect();
        std::fs::write("../target/qa/script-56b-caisse.txt", &script).unwrap();
        let mut trace =
            String::from("image,ecran,essai,x,y,cam_x,cam_y,au_sol,phase,souffle_restant\n");
        for (k, i) in images.iter().enumerate() {
            let (phase, restant) = match i.ventilateur {
                Some((PhaseVentilateur::Arret, _)) => ("arret", 0),
                Some((PhaseVentilateur::Annonce, _)) => ("annonce", 0),
                Some((PhaseVentilateur::Souffle, r)) => ("souffle", r),
                None => ("-", 0),
            };
            trace.push_str(&format!(
                "{k},{},{},{:.1},{:.1},{},{},{},{phase},{restant}\n",
                i.trame.ecran,
                u8::from(i.en_essai),
                i.trame.x,
                i.trame.y,
                i.camera.0,
                i.camera.1,
                u8::from(i.au_sol)
            ));
        }
        std::fs::write("../target/qa/trace-56b-caisse.csv", trace).unwrap();
        let touche = images.iter().position(|i| i.touche_essai).unwrap();
        println!(
            "images : {} ; élan : {} pas ; F4 à l'image {touche} (pilote : {})",
            images.len(),
            parcours.len(),
            touche + 1
        );
    }

    /// Écrit le script du niveau d'essai « montgolfière » (parcours jusqu'en haut :
    /// les deux montées complètes) pour le navigateur :
    /// `target/qa/script-57.txt`, `target/qa/touches-57.txt` (image du pilote où
    /// appuyer sur F5, une image après la trace : le navigateur a un pas de retard) et
    /// `target/qa/trace-57.csv` (position de Pépin, caméra, sol, nacelles, montée).
    /// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_montgolfiere
    #[test]
    #[ignore]
    fn ecrire_script_essai_montgolfiere() {
        let partie = jouer_essai_montgolfiere(&parcours_essai_montgolfiere_jusqu_en_haut());
        assert!(partie.fin_atteinte && partie.morts == 0);
        std::fs::create_dir_all("../target/qa").unwrap();
        let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
        std::fs::write("../target/qa/script-57.txt", &script).unwrap();
        let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
        let pilote = touche + 1;
        std::fs::write("../target/qa/touches-57.txt", format!("{pilote}:F5\n")).unwrap();
        let mut trace =
            String::from("image,ecran,essai,x,y,cam_x,cam_y,au_sol,nacelle1,nacelle2,montee\n");
        for (k, i) in partie.images.iter().enumerate() {
            trace.push_str(&format!(
                "{k},{},{},{:.1},{:.1},{},{},{},{:.1},{:.1},{}\n",
                i.trame.ecran,
                u8::from(i.en_essai),
                i.trame.x,
                i.trame.y,
                i.camera.0,
                i.camera.1,
                u8::from(i.au_sol),
                i.nacelles[0],
                i.nacelles[1],
                u8::from(i.montee)
            ));
        }
        std::fs::write("../target/qa/trace-57.csv", trace).unwrap();
        println!(
            "images : {} ; F5 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
            partie.images.len(),
            partie.gouttes
        );
    }

    /// Écrit le script du niveau d'essai « escargot » pour les captures de la carte
    /// 5.8 : `target/qa/script-58.txt`, `target/qa/touches-58.txt` (l'image où
    /// appuyer sur F6, une image après la trace : le navigateur a un pas de retard) et
    /// `target/qa/trace-58.csv` (position de Pépin, caméra, sol, escargots, rebond,
    /// contact de côté).
    /// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_escargot
    #[test]
    #[ignore]
    fn ecrire_script_essai_escargot() {
        let partie = jouer_essai_escargot(&parcours_essai_escargot());
        assert!(partie.fin_atteinte && partie.morts == 0);
        std::fs::create_dir_all("../target/qa").unwrap();
        let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
        std::fs::write("../target/qa/script-58.txt", &script).unwrap();
        let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
        let pilote = touche + 1;
        std::fs::write("../target/qa/touches-58.txt", format!("{pilote}:F6\n")).unwrap();
        let mut trace = String::from(
            "image,ecran,essai,x,y,cam_x,cam_y,au_sol,escargot1_x,escargot1_aplati,\
             escargot2_x,escargot2_aplati,rebond,contre\n",
        );
        for (k, i) in partie.images.iter().enumerate() {
            let [e1, e2] = i.escargots;
            trace.push_str(&format!(
                "{k},{},{},{:.1},{:.1},{},{},{},{:.1},{},{:.1},{},{},{}\n",
                i.trame.ecran,
                u8::from(i.en_essai),
                i.trame.x,
                i.trame.y,
                i.camera.0,
                i.camera.1,
                u8::from(i.au_sol),
                e1.0,
                u8::from(e1.2),
                e2.0,
                u8::from(e2.2),
                u8::from(i.rebond_escargot),
                u8::from(i.contre_escargot)
            ));
        }
        std::fs::write("../target/qa/trace-58.csv", trace).unwrap();
        println!(
            "images : {} ; F6 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
            partie.images.len(),
            partie.gouttes
        );
    }

    /// Écrit le script des touches pour le navigateur : `target/qa/script.txt`
    /// (un chiffre hexadécimal par image) et `target/qa/trace.csv`.
    /// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script
    #[test]
    #[ignore]
    fn ecrire_script_navigateur() {
        let parcours = tous_les_parcours();
        let partie = jouer_tout(&parcours);
        assert!(partie.fin_atteinte);
        std::fs::create_dir_all("../target/qa").unwrap();
        let script: String = partie.trames.iter().map(Trame::code).collect();
        std::fs::write("../target/qa/script.txt", &script).unwrap();
        let mut trace = String::from(
            "image,ecran,niveau,x,y,corbeau_x,corbeau_y,mobile_x,mobile_y,\
             pigeon_x,pigeon_y,pigeon_phase\n",
        );
        for (i, t) in partie.trames.iter().enumerate() {
            let (cx, cy) = t.corbeau.unwrap_or((-1.0, -1.0));
            let (mx, my) = t.mobile.unwrap_or((-1.0, -1.0));
            let (px, py, phase) = t.pigeon.unwrap_or((-1.0, -1.0, "-"));
            trace.push_str(&format!(
                "{},{},{},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{}\n",
                i, t.ecran, t.niveau, t.x, t.y, cx, cy, mx, my, px, py, phase
            ));
        }
        std::fs::write("../target/qa/trace.csv", trace).unwrap();
        println!(
            "images : {} ({:.1} s) ; morts : {} ; gouttes : {} ; score : {} ; chrono : {} pas ; \
             empreinte : {:016x}",
            partie.trames.len(),
            partie.trames.len() as f32 / 60.0,
            partie.morts,
            partie.gouttes,
            partie.score,
            partie.chrono,
            empreinte(&partie.trames)
        );
    }
}
