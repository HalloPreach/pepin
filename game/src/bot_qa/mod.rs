//! Robot de QA : un « joueur automatique » qui finit tous les niveaux avec la
//! VRAIE simulation du jeu (`Monde` : ennemis, éclairs, plateformes mobiles et
//! friables comprises), puis rejoue ce parcours à travers l'`Application`
//! (titre, cartes de niveau, transitions, écran de fin).
//!
//! Différence avec `pepin_core::faisabilite` : le robot du cœur ignore ennemis
//! et plateformes mobiles. Celui-ci les subit : un plan qui mène à la mort est
//! simplement écarté, donc le parcours trouvé est sûr par construction.
//!
//! ## Ce qui va où
//!
//! - `parcours.rs` : le « cerveau » du robot — plans, simulation, cases
//!   (`CaseParcours`) et recherche de plus court chemin (`chercher`).
//! - `essais.rs` : les parties des NIVEAUX D'ESSAI (`jouer_essai`, générique :
//!   le niveau et sa touche raccourci — une par niveau — sont des PARAMÈTRES)
//!   et leurs parcours de référence (`parcours_essai_*`).
//! - `essais_tests.rs` : les tests des niveaux d'essai (le robot finit sans
//!   mourir ; les parcours à horloge tiennent à une image près).
//! - `scripts.rs` : les tests `ignore` qui ÉCRivent les scripts/traces du
//!   navigateur (`target/qa/*`) pour les captures de QA.md.
//!
//! Dans ce fichier : le rejouage complet (`Trame`, `Partie`, `jouer_tout`,
//! `tous_les_parcours`, `empreinte`), le test « tous les niveaux » et le
//! script de tous les niveaux (`ecrire_script_navigateur`, test ignore).
//!
//! ## Ajouter un niveau d'essai (mécanique nouvelle)
//!
//! 1. `core/src/levels/` : la grille ASCII (module `essai_*` + `INFOS`), avec
//!    seulement les lettres réservées à l'étape en cours (voir CONSIGNES.md).
//! 2. `game/src/monde.rs` : une constante `INDICE_ESSAI_*` juste après la
//!    précédente (tests + build de QA seulement) et sa touche raccourci
//!    (`RaccourciNiveau` dans `entrees.rs` : 0, F1, F2, …).
//! 3. Apprendre la mécanique au robot, dans `parcours.rs` (voir la doc de ce
//!    module : champ dans `CaseParcours`, fin modifiée, tenue au décalage).
//! 4. `essais.rs` : un `jouer_essai_*` (une ligne : indice + raccourci) et un
//!    `parcours_essai_*` (gros puis précis, via `trouver_parcours`).
//! 5. `essais_tests.rs` : « le robot finit sans mourir » (+ « tient à une
//!    image près » si la mécanique a une horloge) et `scripts.rs` : le test
//!    `ecrire_script_essai_*` ignore (script, touches, trace).
//!
//! Le test `ignore` d'en bas écrit le script des touches dans `target/qa/` :
//! un navigateur réel le rejoue image par image (voir QA.md).
//!
//! Notions Rust illustrées ici : `Clone` d'une structure complète (rejouer la
//! partie sans la jouer), `char` comme code de commande, et un hachage FNV-1a
//! maison (empreinte reproductible du jeu, pour comparer deux builds).

mod essais;
mod essais_tests;
mod parcours;
mod scripts;

/// La recherche de `parcours` est l'interface du module : `difficulte.rs`
/// l'importe par `crate::bot_qa::*` (cases, plans, simulation). Les parties
/// d'essai (`essais`) ne sont pas réexportées : les tests y vont par
/// `crate::bot_qa::essais::*`, à côté de leurs parcours.
pub use parcours::*;

use pepin_core::entities::ennemi::PhasePigeon;
use pepin_core::entities::{Entite, TypeEnnemi};
use pepin_core::player::{Direction, Entrees};
use pepin_core::zones::NB_NIVEAUX;

use crate::app::Application;
use crate::ecrans::Ecran;
use crate::entrees::Commandes;
use crate::navigation::Navigation;

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

/// Parcours « complet » de chaque niveau de la table : les trois gouttes d'or
/// ramassées avant l'arrivée (`trouver_parcours_complet`, carte 5.16). Joués à la
/// suite, ils font la partie qui débloque le bonus. Un fil d'exécution par niveau
/// (`std::thread::scope` : les fils empruntent la portée, sans `Arc`).
pub fn tous_les_parcours_complets() -> Vec<Vec<Entrees>> {
    std::thread::scope(|portee| {
        let fils: Vec<_> = (0..NB_NIVEAUX)
            .map(|i| {
                portee.spawn(move || {
                    trouver_parcours_complet(i, false)
                        .or_else(|| trouver_parcours_complet(i, true))
                        .unwrap_or_else(|| panic!("aucun parcours complet au niveau {}", i + 1))
                })
            })
            .collect();
        fils.into_iter()
            .map(|f| f.join().expect("fil du parcours complet"))
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // Le robot ne ramasse pas toutes les gouttes : le bonus (10e) est SAUTÉ, la
        // fin vient juste après le 9e (carte 5.16 : l'un des deux chemins).
        assert!(partie.gouttes < pepin_core::levels::gouttes_pour_le_bonus(NIVEAUX));
        assert_eq!(numeros, (1..=9).collect::<Vec<u8>>(), "bonus sauté");
    }

    /// L'autre chemin (carte 5.16) : la partie « complète » ramasse les trois
    /// gouttes de chaque niveau ; après le 9e, avec les 27 gouttes des niveaux
    /// normaux, le BONUS est joué (« NIVEAU BONUS », l'arc-en-ciel), puis l'écran
    /// de fin, avec les 30 gouttes du jeu ; aucune mort.
    #[test]
    fn la_partie_complete_joue_le_bonus() {
        use pepin_core::zones::GOUTTES_TOTAL;
        let partie = jouer_tout(&tous_les_parcours_complets());
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0);
        let mut joues: Vec<(u8, u32)> = Vec::new();
        for t in partie.trames.iter().filter(|t| t.niveau > 0) {
            match joues.last_mut() {
                Some((n, g)) if *n == t.niveau => *g = t.gouttes,
                _ => joues.push((t.niveau, t.gouttes)),
            }
        }
        // Trois gouttes de plus à chaque niveau ; le bonus (10e) joué après le 9e.
        let attendu: Vec<(u8, u32)> = (1..=10).map(|n| (n, 3 * u32::from(n))).collect();
        assert_eq!(joues, attendu);
        assert_eq!(partie.gouttes as usize, GOUTTES_TOTAL);
        println!(
            "partie complète : {} images, chrono {} pas, empreinte {:016x}",
            partie.trames.len(),
            partie.chrono,
            empreinte(&partie.trames)
        );
    }

    /// Le bonus (« L'arc-en-ciel », carte 5.16) : la partie du robot ne le joue
    /// pas (il ne ramasse pas toutes les gouttes), on rejoue donc son parcours à
    /// part, dans la vraie simulation : le pont friable se traverse d'un bout à
    /// l'autre sans mourir, et sans jamais rester posé plus d'une demi-seconde sur
    /// la même dalle (elle tomberait).
    #[test]
    fn le_robot_traverse_l_arc_en_ciel_sans_mourir() {
        use crate::monde::Monde;
        use pepin_core::levels::{arc, GenreNiveau, NIVEAUX};
        let i = NIVEAUX
            .iter()
            .position(|n| n.genre == GenreNiveau::Bonus)
            .unwrap();
        assert_eq!(NIVEAUX[i].nom, arc::INFOS.nom);
        let parcours = trouver_parcours(i, false)
            .or_else(|| trouver_parcours(i, true))
            .expect("parcours du bonus");
        let mut m = Monde::new(i);
        for e in &parcours {
            m.avancer(e);
            assert!(m.derniere_mort.is_none(), "mort au pas {}", m.pas);
        }
        assert!(m.termine, "arrivée non atteinte");
        assert_eq!(m.etat.morts, 0);
        println!(
            "arc-en-ciel : {} pas ({:.1} s), gouttes {:?}",
            parcours.len(),
            parcours.len() as f32 / 60.0,
            m.gouttes_niveau
        );
    }

    /// Le clocher (carte 5.13) : le parcours le plus RAPIDE du robot allume les
    /// trois lanternes. Elles sont posées là où l'on passe forcément (après l'arche
    /// du bas, à la sortie de la porte de la cage, au bord du pilier d'où l'on saute
    /// sur la petite aiguille) : une chute sur les pointes ramène sur le pilier, pas
    /// tout en bas de la tour. (Première version : lanternes à côté du chemin, le
    /// robot ne touchait ni celle de la cage ni celle du pilier, et une chute le
    /// renvoyait à la galerie, vingt rangées plus bas.)
    #[test]
    fn au_clocher_le_parcours_allume_les_trois_lanternes() {
        use crate::monde::Monde;
        use pepin_core::levels::{clocher, NIVEAUX};
        let i = NIVEAUX
            .iter()
            .position(|n| n.nom == clocher::INFOS.nom)
            .unwrap();
        let parcours = trouver_parcours(i, false)
            .or_else(|| trouver_parcours(i, true))
            .unwrap();
        let mut m = Monde::new(i);
        // Une seule lanterne allumée à la fois (la dernière touchée) : on retient
        // celles qui l'ont été au moins une fois pendant le parcours.
        let mut allumees = vec![false; 3];
        for e in &parcours {
            m.avancer(e);
            let maintenant: Vec<bool> = m
                .entites
                .iter()
                .filter_map(|e| match e {
                    Entite::Lanterne(l) => Some(l.est_active()),
                    _ => None,
                })
                .collect();
            assert_eq!(maintenant.len(), 3, "trois lanternes au clocher");
            for (a, l) in allumees.iter_mut().zip(maintenant) {
                *a |= l;
            }
        }
        assert_eq!(allumees, [true; 3]);
    }

    /// Écrit le script des touches pour le navigateur : `target/qa/script.txt`
    /// (un chiffre hexadécimal par image) et `target/qa/trace.csv`.
    /// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script
    #[test]
    #[ignore]
    fn ecrire_script_navigateur() {
        let partie = jouer_tout(&tous_les_parcours());
        ecrire_partie(&partie, "script.txt", "trace.csv");
    }

    /// Écrit le script de la partie COMPLÈTE (carte 5.16 : toutes les gouttes, donc le
    /// bonus joué après le 9e niveau) : `target/qa/script-516.txt` et
    /// `target/qa/trace-516.csv`, même format que `ecrire_script_navigateur` (le
    /// navigateur rejoue l'un ou l'autre avec `SCRIPT=...`).
    /// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script
    #[test]
    #[ignore]
    fn ecrire_script_partie_complete() {
        let partie = jouer_tout(&tous_les_parcours_complets());
        ecrire_partie(&partie, "script-516.txt", "trace-516.csv");
    }

    /// Écrit le script (un chiffre hexadécimal par image) et la trace d'une partie
    /// dans `target/qa/`, puis affiche son bilan et son empreinte.
    fn ecrire_partie(partie: &Partie, fichier_script: &str, fichier_trace: &str) {
        assert!(partie.fin_atteinte);
        std::fs::create_dir_all("../target/qa").unwrap();
        let script: String = partie.trames.iter().map(Trame::code).collect();
        std::fs::write(format!("../target/qa/{fichier_script}"), &script).unwrap();
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
        std::fs::write(format!("../target/qa/{fichier_trace}"), trace).unwrap();
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
