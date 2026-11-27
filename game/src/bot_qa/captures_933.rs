//! Captures de la carte 9.33 (l'affichage de la Pieuvre en phase 2 était cassé) :
//! les scripts de la VRAIE phase 2, jouée APRÈS la phase 1 dans le niveau de la
//! Pieuvre (pas un niveau d'essai posé dans la tour). Tests ignorés, lancés à la main :
//! `./docker-cargo.sh test -p game --no-default-features ecrire_script_pieuvre_933 -- --ignored --nocapture`
//!
//! Deux scripts, un caractère par image (`code_entrees`, base 32), rejoués par
//! `docs/capture-933.sh` (commande `s:` de `docs/capture-102.mjs`) :
//! - `docs/script-pieuvre-933.txt`, mode NORMAL : il part de la carte de titre du
//!   niveau de la Pieuvre, une fois la carte prête (son 1er caractère est l'appui
//!   Entrée qui la passe) ;
//! - `docs/script-pieuvre-933-facile.txt`, mode FACILE (c'est là que le défaut de la
//!   9.33 se voyait) : il part du TITRE, avec la sauvegarde `SAUVEGARDE_FACILE` (tout
//!   le Monde 2 fini sauf la Pieuvre, 10 défaites à la Pieuvre) : JOUER ouvre la carte
//!   des niveaux sur la Pieuvre, Entrée l'ouvre, la bannière « TROP DUR ? » propose le
//!   mode facile, Gauche puis Entrée répondent OUI.
//!
//! Le parcours est celui du robot de QA de partie : la phase 1 du robot (carte
//! 10.7, par points), puis, dès que la phase 2 commence, le PILOTE de la 9.13g
//! (`pilote_pieuvre::jouer`) depuis l'état réel de la partie. Tout est rejoué image
//! par image dans l'APPLICATION, comme la partie du robot (`jouer_depuis_le_titre`).
//! Chaque test écrit aussi `target/qa/moments-933[-facile].txt` : une ligne
//! `nom image` par moment (la fuite, le bas, le milieu et le haut de la tour, la
//! vigie, la plongée), d'où viennent les images de `docs/capture-933.sh`.

use pepin_core::entities::pieuvre::PhasePieuvre;
use pepin_core::player::Entrees;

use super::parcours::trouver_parcours;
use super::pilote_pieuvre::{en_phase_2, indice_pieuvre, pieuvre, sur_la_vigie};
use super::{code_entrees, SAUVEGARDE_MONDE_2_OUVERT};
use crate::app::Application;
use crate::ecrans::Ecran;
use crate::entrees::{Commandes, RaccourciNiveau};
use crate::navigation::Navigation;

/// La sauvegarde du script FACILE (même texte dans `docs/capture-933.sh`) : tous les
/// niveaux avant la Pieuvre finis, et 10 défaites d'affilée à la Pieuvre
/// (`facile::DEFAITES_AVANT_FACILE`) : le mode facile y est proposé.
pub const SAUVEGARDE_FACILE: &str = "version=3\n\
niveau.toits=atteint:1 medaille:1\nniveau.gouttieres=atteint:1 medaille:1\n\
niveau.marche=atteint:1 medaille:1\nniveau.clocher=atteint:1 medaille:1\n\
niveau.poursuite=atteint:1 medaille:1\nniveau.descente=atteint:1 medaille:1\n\
niveau.vents=atteint:1 medaille:1\nniveau.orage=atteint:1 medaille:1\n\
niveau.mer=atteint:1 medaille:1\nniveau.nuages=atteint:1 medaille:1\n\
niveau.cyclone=atteint:1 medaille:1\nniveau.arc=atteint:1 medaille:1\n\
niveau.caniveau=atteint:1 medaille:1\nniveau.egouts=atteint:1 medaille:1\n\
niveau.cascade=atteint:1 medaille:1\nniveau.grotte=atteint:1 medaille:1\n\
niveau.riviere=atteint:1 medaille:1\nniveau.port=atteint:1 medaille:1\n\
niveau.maree=atteint:1 medaille:1\nniveau.pieuvre=defaites:10";

#[test]
#[ignore = "captures de la carte 9.33 : se lance à la main"]
fn ecrire_script_pieuvre_933() {
    let indice = indice_pieuvre();
    // L'application amenée sur la carte de titre du niveau de la Pieuvre, comme le
    // build de QA : JOUER, puis le raccourci « niveau suivant » (touche N) jusqu'à
    // lui. Rien de cette mise en place n'entre dans le script.
    let mut app = Application::new(true);
    app.charger_sauvegarde_qa(SAUVEGARDE_MONDE_2_OUVERT);
    let mut image = 0;
    while !(matches!(app.ecran, Ecran::Carte(_))
        && app.monde.indice == indice
        && app.texte_en_attente())
    {
        assert!(image < 60 * 60, "la carte de la Pieuvre n'arrive pas");
        let raccourci = (app.ecran == Ecran::Jeu && app.en_jeu() && app.monde.indice != indice)
            .then_some(RaccourciNiveau::Suivant);
        let nav = Navigation {
            valider: image == 0 || app.texte_en_attente(),
            ..Navigation::default()
        };
        app.reagir(&nav, &Commandes::default(), raccourci);
        app.pas(&Entrees::default());
        image += 1;
    }
    assert!(!pieuvre(&app.monde).est_facile());
    derouler(app, String::new(), "933");
}

/// Le même en mode FACILE, depuis le titre (voir l'en-tête du module), comme un
/// joueur qui a déjà perdu 10 fois contre la Pieuvre : JOUER (image 0) ouvre la carte
/// de titre de la Pieuvre, Entrée (60) la passe, Pépin court à droite (120 à 159)
/// et tombe à l'eau ; à sa mort, la bannière « TROP DUR ? » ; Gauche (400) choisit
/// OUI, Entrée (420) : le niveau recommence en facile.
#[test]
#[ignore = "captures de la carte 9.33 : se lance à la main"]
fn ecrire_script_pieuvre_933_facile() {
    use pepin_core::player::Direction;
    let indice = indice_pieuvre();
    let mut app = Application::new(true);
    app.charger_sauvegarde_qa(SAUVEGARDE_FACILE);
    let mut script = String::new();
    for image in 0..480 {
        let valider = matches!(image, 0 | 60 | 420);
        let gauche = image == 400;
        let droite = (120..160).contains(&image);
        let nav = Navigation {
            valider,
            gauche,
            droite: image == 120,
            ..Navigation::default()
        };
        let e = Entrees {
            direction: if gauche {
                Direction::Gauche
            } else if droite {
                Direction::Droite
            } else {
                Direction::Immobile
            },
            ..Entrees::default()
        };
        app.reagir(&nav, &Commandes::default(), None);
        app.pas(&e);
        script.push(code_entrees(&e, valider));
        if matches!(image, 1 | 59 | 61 | 119 | 161 | 399 | 401 | 421 | 479) {
            println!(
                "image {image} : écran {}, niveau {}, en jeu {}, morts {}",
                super::nom_ecran(app.ecran),
                app.monde.indice,
                app.en_jeu(),
                app.monde.etat.morts
            );
        }
    }
    assert_eq!(app.monde.indice, indice, "JOUER ouvre la Pieuvre");
    assert!(
        pieuvre(&app.monde).est_facile(),
        "la Pieuvre en mode facile"
    );
    derouler(app, script, "933-facile");
}

/// Joue le niveau de la Pieuvre dans `app` (déjà chargé, sa carte de titre à
/// passer) et écrit le script (`script` : ce qui précède) et les moments dans
/// `target/qa/`. En normal, le parcours du robot de partie tel quel (il contient déjà
/// la phase 2 du pilote). En facile (la Pieuvre n'a que 2 têtes en phase 1, le
/// parcours du robot n'y vaut plus après le 2e coup) : la phase 1 du robot, puis le
/// pilote, lancé depuis l'état réel dès qu'il y gagne sans mourir (essayé à chaque
/// pas une fois la phase 2 commencée, la phase 1 du robot continuant en attendant).
fn derouler(mut app: Application, mut script: String, nom: &str) {
    let indice = indice_pieuvre();
    let facile = pieuvre(&app.monde).est_facile();
    let phase_1 = trouver_parcours(indice, false)
        .or_else(|| trouver_parcours(indice, true))
        .expect("le robot de partie finit la Pieuvre");
    // Les pas du pilote, et le pas du niveau où il a pris la main.
    let mut pilote: Option<(u32, Vec<Entrees>)> = None;
    let debut = script.len();
    let mut moments: Vec<(&str, usize)> = Vec::new();
    let mut bas_de_la_tour: Option<f32> = None;
    let vigie = pieuvre(&app.monde).tete_finale.map(|v| v.bas());
    let morts = app.monde.etat.morts;
    for k in debut..debut + 60 * 60 * 5 {
        let valider = app.texte_en_attente();
        let mut e = Entrees::default();
        if app.ecran == Ecran::Jeu && app.en_jeu() {
            // Le pilote prend la main dès la phase 2, et la reprend après chaque
            // mort (Pépin repart de la lanterne du bas de la tour, le départ du
            // pilote de la 9.13g) : ses pas jusqu'à sa victoire ou sa 1re mort.
            let epuise = pilote
                .as_ref()
                .is_some_and(|(depart, pas)| (app.monde.pas - depart) as usize >= pas.len());
            if facile && (pilote.is_none() || epuise) && en_phase_2(&app.monde) {
                let deja = app.monde.etat.morts;
                let mut pas = Vec::new();
                let bilan = super::pilote_pieuvre::jouer(app.monde.clone(), &mut |p| p, |m, e| {
                    if m.etat.morts > deja {
                        return false;
                    }
                    pas.push(*e);
                    true
                });
                println!(
                    "pilote : la main au pas {} (image {k}), {} pas, victoire {}",
                    app.monde.pas,
                    pas.len(),
                    bilan.victoire
                );
                pilote = Some((app.monde.pas, pas));
            }
            let suivante = match &pilote {
                Some((depart, pas)) => pas.get((app.monde.pas - depart) as usize),
                None => phase_1.get(app.monde.pas as usize),
            };
            if let Some(x) = suivante {
                e = *x;
            }
        }
        let nav = Navigation {
            valider,
            ..Navigation::default()
        };
        app.reagir(&nav, &Commandes::default(), None);
        app.pas(&e);
        script.push(code_entrees(&e, valider));
        // En normal, le robot ne meurt jamais ; en facile, le pilote peut mourir
        // dans la tour (il y repart de la lanterne du bas).
        assert!(
            facile || app.monde.etat.morts == morts,
            "mort à l'image {k}"
        );
        if app.ecran != Ecran::Jeu || app.monde.indice != indice {
            break;
        }
        let p = pieuvre(&app.monde);
        let y = app.monde.joueur.boite.bas();
        let mut noter = |nom: &'static str, vrai: bool| {
            if vrai && !moments.iter().any(|&(n, _)| n == nom) {
                moments.push((nom, k));
            }
        };
        noter("fuite", p.en_fuite());
        let montee = p.en_fuite() && p.phase() == PhasePieuvre::Montee;
        noter("montee", montee);
        if montee && bas_de_la_tour.is_none() {
            bas_de_la_tour = Some(y);
        }
        if let (Some(b), Some(v)) = (bas_de_la_tour, vigie) {
            noter("tour-milieu", montee && y < (b + v) / 2.0);
            noter("tour-haut", montee && y < v + (b - v) / 6.0);
        }
        noter("vigie", sur_la_vigie(&app.monde));
        noter("plongee", p.phase() == PhasePieuvre::Plongee);
        if k % 120 == 0 {
            println!(
                "image {k} : phase {:?}, fuite {}, Pépin y {y:.0}, mer {:?}, têtes {}",
                p.phase(),
                p.en_fuite(),
                p.surface_mer().map(|m| m.round()),
                p.pv()
            );
        }
    }
    std::fs::create_dir_all("../target/qa").unwrap();
    std::fs::write(format!("../target/qa/script-{nom}.txt"), &script).unwrap();
    let lignes: String = moments.iter().map(|(n, k)| format!("{n} {k}\n")).collect();
    std::fs::write(format!("../target/qa/moments-{nom}.txt"), &lignes).unwrap();
    println!(
        "script-{nom}.txt : {} images ; morts {} ; vigie à y = {vigie:?}\n{lignes}",
        script.len(),
        app.monde.etat.morts - morts
    );
}
