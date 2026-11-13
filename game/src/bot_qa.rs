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
/// arrondi, et « lancé par un auvent » (`est_lance`) ou posé. Deux états proches
/// fusionnent.
pub(crate) type CaseParcours = (i32, i32, bool);

/// La case (`CaseParcours`) de Pépin dans `m`.
pub(crate) fn cle(m: &Monde) -> CaseParcours {
    (
        (m.joueur.boite.x / 8.0).floor() as i32,
        m.joueur.boite.y.round() as i32,
        est_lance(m),
    )
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
        images.push(ImageEssai {
            trame,
            touche_essai,
            en_essai: en_essai && app.en_jeu(),
            camera,
            pepin_ecran_y,
            repere,
            sous_la_grele,
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
