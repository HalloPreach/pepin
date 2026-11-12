//! Robot de QA : un « joueur automatique » qui finit les cinq niveaux avec la
//! VRAIE simulation du jeu (`Monde` : ennemis, éclairs, plateformes mobiles et
//! friables comprises), puis rejoue ce parcours à travers l'`Application`
//! (titre, cartes de niveau, transitions, écran de fin).
//!
//! Différence avec `pepin_core::faisabilite` : le robot du cœur ignore ennemis
//! et plateformes mobiles. Celui-ci les subit : un plan qui mène à la mort est
//! simplement écarté, donc le parcours trouvé est sûr par construction.
//!
//! Principe : recherche de plus court chemin (Dijkstra sur la durée) entre des
//! états « posés et immobiles ». Chaque plan (courir, sauter, planer, attendre)
//! est simulé sur une COPIE du monde. Le test `ignore` d'en bas écrit le script
//! des touches dans `target/qa/` : un navigateur réel le rejoue image par image
//! (voir QA.md).
//!
//! Notions Rust illustrées ici : `BinaryHeap` + `Reverse` (file de priorité
//! « plus petit d'abord »), `Clone` d'une structure complète pour explorer des
//! futurs possibles sans toucher au présent, `HashMap` comme mémoire.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use pepin_core::entities::{Entite, TypeEnnemi};
use pepin_core::player::{Direction, Entrees};
use pepin_core::zones::NB_NIVEAUX;

use crate::app::Application;
use crate::ecrans::Ecran;
use crate::entrees::Commandes;
use crate::monde::Monde;
use crate::navigation::Navigation;

/// Durée maximale d'un plan, en pas (10 s).
const PAS_MAX: u16 = 600;
/// Pas d'immobilité qui valident un atterrissage stable.
const PAS_STABLE: u16 = 20;
/// Pas d'attente au départ : Pépin tombe au sol et se pose.
const PAS_DEPART: u16 = 40;

/// Un plan d'entrées, simulé d'un seul bloc.
#[derive(Clone, Copy)]
struct Plan {
    dir: Direction,
    /// Pas de course au sol avant le saut.
    elan: u16,
    /// Pas de maintien du saut (0 = pas de saut, on marche dans le vide).
    saut: u16,
    planer: bool,
    dir_montee: bool,
    dir_descente: bool,
    /// Plan « attendre » : aucune entrée pendant ce nombre de pas (0 = non).
    attente: u16,
}

/// Comment se termine la simulation d'un plan.
enum Fin {
    Mort,
    Arrivee,
    Pose,
    Abandon,
}

/// Tous les plans à essayer. `precis` ajoute des élans et des durées plus fins.
fn plans(precis: bool) -> Vec<Plan> {
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
            dir_montee: false,
            dir_descente: false,
            attente,
        });
    }
    v
}

fn est_mort(m: &Monde) -> bool {
    m.reapparition > 0 || m.derniere_mort.is_some()
}

/// Simule `plan` depuis `depart` (copie). Renvoie la fin, le monde final et les
/// entrées de chaque pas.
fn simuler(depart: &Monde, plan: Plan) -> (Fin, Monde, Vec<Entrees>) {
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
    let mut a_decolle = false;
    let mut stable: u16 = 0;
    let mut saut_envoye: u16 = 0;
    for pas in 0..PAS_MAX {
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
            let garde = if montee {
                plan.dir_montee
            } else {
                plan.dir_descente
            };
            e.direction = if garde { plan.dir } else { Direction::Immobile };
            if saut_envoye > 0 && saut_envoye < plan.saut && montee {
                e.saut = true;
                saut_envoye += 1;
            } else if plan.planer && !montee && en_l_air >= 7 {
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

/// Deux états proches fusionnent : case de 8 px en x, y arrondi.
fn cle(m: &Monde) -> (i32, i32) {
    (
        (m.joueur.boite.x / 8.0).floor() as i32,
        m.joueur.boite.y.round() as i32,
    )
}

/// Cherche les entrées (une par pas) qui mènent du début du niveau `indice` à
/// l'arrivée sans jamais mourir. `None` si rien n'est trouvé.
pub fn trouver_parcours(indice: usize, precis: bool) -> Option<Vec<Entrees>> {
    let mut depart = Monde::new(indice);
    let mut prefixe = Vec::new();
    for _ in 0..PAS_DEPART {
        depart.avancer(&Entrees::default());
        prefixe.push(Entrees::default());
    }
    let tous = plans(precis);

    // Chaque noeud : le monde posé, son coût (pas), son parent et les entrées
    // qui y ont mené. La file de priorité ne contient que (coût, indice).
    struct Noeud {
        monde: Monde,
        parent: Option<usize>,
        entrees: Vec<Entrees>,
    }
    let mut noeuds = vec![Noeud {
        monde: depart,
        parent: None,
        entrees: prefixe,
    }];
    let mut meilleur: HashMap<(i32, i32), u32> = HashMap::new();
    meilleur.insert(cle(&noeuds[0].monde), 0);
    let mut tas = BinaryHeap::new();
    tas.push(Reverse((0u32, 0usize)));
    let mut arrivee: Option<(u32, usize, Vec<Entrees>)> = None;

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
        for &plan in &tous {
            let (fin, monde, script) = simuler(&noeuds[i].monde, plan);
            let cout_plan = cout + script.len() as u32;
            match fin {
                Fin::Arrivee => {
                    if arrivee.as_ref().is_none_or(|(c, _, _)| cout_plan < *c) {
                        arrivee = Some((cout_plan, i, script));
                    }
                }
                Fin::Pose => {
                    let k = cle(&monde);
                    if meilleur.get(&k).is_none_or(|&c| cout_plan < c) {
                        meilleur.insert(k, cout_plan);
                        noeuds.push(Noeud {
                            monde,
                            parent: Some(i),
                            entrees: script,
                        });
                        tas.push(Reverse((cout_plan, noeuds.len() - 1)));
                    }
                }
                Fin::Mort | Fin::Abandon => {}
            }
        }
    }

    let (_, dernier, fin_script) = arrivee?;
    // Remonte la chaîne des parents, puis remet les morceaux à l'endroit.
    let mut morceaux = vec![fin_script];
    let mut courant = Some(dernier);
    while let Some(i) = courant {
        morceaux.push(noeuds[i].entrees.clone());
        courant = noeuds[i].parent;
    }
    morceaux.reverse();
    Some(morceaux.concat())
}

/// Ce que le navigateur doit rejouer à chaque image.
#[derive(Clone, Copy, Default)]
pub struct Trame {
    pub valider: bool,
    pub entrees: Entrees,
    /// Niveau affiché (1 à 5, 0 hors jeu) et position de Pépin : pour choisir
    /// les images à photographier.
    pub niveau: u8,
    pub x: f32,
    pub y: f32,
    pub ecran: &'static str,
    /// Position du premier corbeau actif du niveau (pour viser une capture).
    pub corbeau: Option<(f32, f32)>,
    /// Boîte de la première plateforme mobile (x, y), pour viser une capture.
    pub mobile: Option<(f32, f32)>,
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
    pub fin_atteinte: bool,
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
        trame.x = app.monde.joueur.boite.x;
        trame.y = app.monde.joueur.boite.y;
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
        fin_atteinte,
    }
}

/// Cherche le parcours des cinq niveaux (robot « maladroit » d'abord, puis
/// précis si besoin).
pub fn tous_les_parcours() -> Vec<Vec<Entrees>> {
    (0..NB_NIVEAUX)
        .map(|i| {
            trouver_parcours(i, false)
                .or_else(|| trouver_parcours(i, true))
                .unwrap_or_else(|| panic!("le robot ne trouve aucun parcours au niveau {}", i + 1))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_robot_finit_les_cinq_niveaux_sans_mourir() {
        let parcours = tous_les_parcours();
        let partie = jouer_tout(&parcours);
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
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
        let mut trace =
            String::from("image,ecran,niveau,x,y,corbeau_x,corbeau_y,mobile_x,mobile_y\n");
        for (i, t) in partie.trames.iter().enumerate() {
            let (cx, cy) = t.corbeau.unwrap_or((-1.0, -1.0));
            let (mx, my) = t.mobile.unwrap_or((-1.0, -1.0));
            trace.push_str(&format!(
                "{},{},{},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1}\n",
                i, t.ecran, t.niveau, t.x, t.y, cx, cy, mx, my
            ));
        }
        std::fs::write("../target/qa/trace.csv", trace).unwrap();
        println!(
            "images : {} ({:.1} s) ; morts : {} ; gouttes : {}",
            partie.trames.len(),
            partie.trames.len() as f32 / 60.0,
            partie.morts,
            partie.gouttes
        );
    }
}
