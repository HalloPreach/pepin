//! Tests de la proposition du mode facile (carte 9.34, AGA : « le passage au mode
//! facile apparaît trop vite quand c'est la deuxième fois qu'on joue le niveau »).
//!
//! La règle : seules les défaites de la VISITE en cours du niveau ouvrent la
//! bannière « TROP DUR ? » (`DEFAITES_AVANT_FACILE`) ; une sortie puis un retour
//! remettent le compte à zéro ; un NON tient pour la visite ; sur la carte des
//! niveaux, la bannière n'est proposée d'emblée qu'au joueur qui a déjà vaincu le
//! boss en facile (et jamais en normal). Chaque test est joué pour les DEUX boss :
//! le Grand Nuage (`cyclone`) et la Pieuvre (`pieuvre`).

use pepin_core::player::Entrees;

use crate::ecrans::Ecran;
use crate::entrees::Commandes;
use crate::facile::{Origine, DEFAITES_AVANT_FACILE};
use crate::navigation::Navigation;

use super::*;

/// Les deux boss : la clé de leur niveau.
const BOSS: [&str; 2] = ["cyclone", "pieuvre"];

fn valider() -> Navigation {
    Navigation {
        valider: true,
        ..Navigation::default()
    }
}

fn echap() -> Navigation {
    Navigation {
        retour: true,
        ..Navigation::default()
    }
}

fn avancer(app: &mut Application, n: u32) {
    for _ in 0..n {
        app.pas(&Entrees::default());
    }
}

fn indice(cle: &str) -> usize {
    niveaux::niveaux()
        .iter()
        .position(|n| n.cle == cle)
        .unwrap()
}

/// Une application dont la sauvegarde est `texte` (format de la sauvegarde v3), où
/// tous les niveaux d'avant le boss `cle` sont finis en normal (pour que JOUER le
/// lance depuis la carte des niveaux, Monde 2 compris).
fn avec_sauvegarde(cle: &str, texte: &str) -> Application {
    let mut app = Application::new(true);
    app.charger_sauvegarde_qa(texte);
    for n in &niveaux::niveaux()[..indice(cle)] {
        if let Some(p) = app.sauvegarde.progres_mut(n.cle) {
            p.atteint = true;
            p.meilleur_temps.get_or_insert(6000);
        }
    }
    app
}

/// Avance jusqu'à ce que `ok` soit vrai, en passant d'un appui les écrans de texte
/// (carte du niveau, « PLUS DE VIES ») dès qu'ils l'acceptent, comme un joueur.
fn jusqu_a(app: &mut Application, ok: impl Fn(&Application) -> bool) {
    for _ in 0..2000 {
        if ok(app) && !app.transition.est_active() {
            return;
        }
        if app.texte_en_attente() {
            app.reagir(&valider(), &Commandes::default(), None);
        }
        app.pas(&Entrees::default());
    }
    panic!("état attendu jamais atteint (écran : {:?})", app.ecran);
}

/// Entre dans le niveau du boss par la carte des niveaux (JOUER) et attend le jeu.
/// Sans bannière à la carte des niveaux : l'appel échoue s'il y en a une.
fn entrer_par_la_carte(app: &mut Application, cle: &str) {
    app.ouvrir_plan(indice(cle));
    jusqu_a(app, |a| a.ecran == Ecran::Plan);
    app.reagir(&valider(), &Commandes::default(), None);
    assert_eq!(
        app.facile.proposition, None,
        "{cle} : aucune bannière à la carte"
    );
    jusqu_a(app, |a| a.ecran == Ecran::Jeu && a.monde.infos.cle == cle);
}

/// Une mort de Pépin (une chute sous le niveau), animation comprise. Renvoie `true`
/// si la bannière du mode facile s'est ouverte.
fn mourir(app: &mut Application) -> bool {
    assert_eq!(app.ecran, Ecran::Jeu);
    assert_eq!(app.facile.proposition, None);
    let morts = app.monde.etat.morts;
    let bas = app.monde.niveau.hauteur_px();
    let x = app.monde.joueur.boite.x;
    app.monde.joueur.placer(x, bas + 100.0);
    for _ in 0..600 {
        avancer(app, 1);
        if app.facile.proposition.is_some() {
            return true;
        }
        if app.monde.etat.morts > morts && app.monde.reapparition == 0 {
            break;
        }
        if matches!(app.ecran, Ecran::PlusDeVies(_)) {
            break;
        }
    }
    assert!(app.monde.etat.morts > morts, "Pépin est mort");
    // « PLUS DE VIES » : on le passe, le niveau recommence (même visite).
    if matches!(app.ecran, Ecran::PlusDeVies(_)) {
        jusqu_a(app, |a| a.ecran == Ecran::Jeu);
    }
    false
}

/// Meurt jusqu'à la bannière ; renvoie le numéro de la mort (dans la visite) qui
/// l'a ouverte. Échoue au-delà de `max` morts.
fn mourir_jusqu_a_la_banniere(app: &mut Application, max: u32) -> u32 {
    for n in 1..=max {
        if mourir(app) {
            return n;
        }
    }
    panic!("aucune bannière en {max} morts");
}

/// La première mort de la visite après laquelle la bannière peut s'ouvrir, avec
/// 5 vies au départ : la 10e mort est la dernière vie (« plus de vies » passe
/// avant), donc c'est la 11e.
fn premiere_mort_proposee() -> u32 {
    let mut n = DEFAITES_AVANT_FACILE;
    while n % u32::from(pepin_core::vies::VIES_DEPART) == 0 {
        n += 1;
    }
    n
}

/// 15 défaites ANCIENNES dans la sauvegarde (une vieille session) : aucune bannière,
/// ni à la carte des niveaux, ni à la 1re mort ; il en faut de nouveau 10 dans la
/// visite. La sauvegarde, elle, garde son champ `defaites` (format inchangé).
#[test]
fn des_defaites_anciennes_ne_proposent_rien() {
    for cle in BOSS {
        let mut app = avec_sauvegarde(
            cle,
            &format!(
                "version=3\nniveau.{cle}=atteint:1 gouttes:000 defis:000 medaille:0 defaites:15\n"
            ),
        );
        assert_eq!(app.sauvegarde.progres(cle).defaites, 15, "{cle} : relu");
        entrer_par_la_carte(&mut app, cle);
        assert!(!mourir(&mut app), "{cle} : rien à la 1re mort");
        let n = 1 + mourir_jusqu_a_la_banniere(&mut app, 20);
        assert_eq!(n, premiere_mort_proposee(), "{cle}");
        // Le compte enregistré a continué (une statistique, toujours écrite).
        assert_eq!(app.sauvegarde.progres(cle).defaites, 15 + n, "{cle}");
        assert!(app
            .sauvegarde
            .vers_texte()
            .contains(&format!("defaites:{}", 15 + n)));
    }
}

/// Dix défaites (et plus) dans une même visite ouvrent la bannière, aussi sans
/// sauvegarde : c'est la règle de la carte 8.14, gardée.
#[test]
fn dix_defaites_dans_la_visite_ouvrent_la_banniere() {
    for cle in BOSS {
        let mut app = avec_sauvegarde(cle, &format!("version=3\nniveau.{cle}=atteint:1\n"));
        entrer_par_la_carte(&mut app, cle);
        let n = mourir_jusqu_a_la_banniere(&mut app, 20);
        assert_eq!(n, premiere_mort_proposee(), "{cle}");
        assert!(n >= DEFAITES_AVANT_FACILE);
        assert_eq!(app.facile.defaites_visite, n, "{cle}");
        let p = app.facile.proposition.unwrap();
        assert_eq!(p.origine, Origine::Mort);
    }
}

/// Neuf défaites, une sortie (retour à la carte des niveaux), un retour : le compte
/// repart de zéro, il en faut de nouveau 10 dans la nouvelle visite.
#[test]
fn une_sortie_puis_un_retour_remettent_le_compte_a_zero() {
    for cle in BOSS {
        let mut app = avec_sauvegarde(cle, &format!("version=3\nniveau.{cle}=atteint:1\n"));
        entrer_par_la_carte(&mut app, cle);
        for _ in 0..9 {
            assert!(!mourir(&mut app), "{cle}");
        }
        assert_eq!(app.facile.defaites_visite, 9);
        // On quitte, on revient (ici directement par la carte des niveaux).
        entrer_par_la_carte(&mut app, cle);
        assert_eq!(app.facile.defaites_visite, 0, "{cle} : nouvelle visite");
        assert!(
            !mourir(&mut app),
            "{cle} : la 10e défaite au total ne compte pas"
        );
        let n = 1 + mourir_jusqu_a_la_banniere(&mut app, 20);
        assert_eq!(n, premiere_mort_proposee(), "{cle}");
        assert_eq!(
            app.sauvegarde.progres(cle).defaites,
            9 + n,
            "{cle} : statistique"
        );
    }
}

/// Un NON tient pour toute la visite (« plus de vies » compris) ; la visite
/// suivante peut de nouveau proposer.
#[test]
fn un_non_tient_pour_la_visite() {
    for cle in BOSS {
        let mut app = avec_sauvegarde(cle, &format!("version=3\nniveau.{cle}=atteint:1\n"));
        entrer_par_la_carte(&mut app, cle);
        mourir_jusqu_a_la_banniere(&mut app, 20);
        // L'attente de la bannière, puis NON (Échap après une mort).
        avancer(&mut app, u32::from(crate::facile::ATTENTE_PAS));
        app.reagir(&echap(), &Commandes::default(), None);
        assert_eq!(app.facile.proposition, None, "{cle}");
        assert!(app.facile.refusee && !app.facile.actif, "{cle}");
        for _ in 0..12 {
            assert!(!mourir(&mut app), "{cle} : plus rien dans cette visite");
        }
        entrer_par_la_carte(&mut app, cle);
        assert!(!app.facile.refusee, "{cle} : nouvelle visite");
        mourir_jusqu_a_la_banniere(&mut app, 20);
    }
}

/// Sur la carte des niveaux : le boss déjà vaincu en facile (pas en normal) fait
/// proposer le choix d'emblée ; OUI le lance en facile.
#[test]
fn boss_battu_en_facile_proposition_a_la_carte() {
    for cle in BOSS {
        let mut app = avec_sauvegarde(
            cle,
            &format!(
                "version=3\nniveau.{cle}=atteint:1 gouttes:000 defis:000 medaille:0 facile:1\n"
            ),
        );
        app.ouvrir_plan(indice(cle));
        jusqu_a(&mut app, |a| a.ecran == Ecran::Plan);
        app.reagir(&valider(), &Commandes::default(), None);
        let p = app.facile.proposition.expect("bannière à la carte");
        assert_eq!(p.origine, Origine::Plan(indice(cle)), "{cle}");
        // OUI (gauche, puis Entrée) : le niveau en facile.
        let gauche = Navigation {
            gauche: true,
            ..Navigation::default()
        };
        app.reagir(&gauche, &Commandes::default(), None);
        app.reagir(&valider(), &Commandes::default(), None);
        jusqu_a(&mut app, |a| {
            a.ecran == Ecran::Jeu && a.monde.infos.cle == cle
        });
        assert!(app.facile.actif, "{cle} : en facile");
        assert_eq!(
            app.monde.points_nuage().map(|(_, max)| max),
            Some(2),
            "{cle}"
        );
    }
}

/// Boss battu en normal (un meilleur temps, une médaille) : jamais de proposition,
/// ni à la carte (même battu aussi en facile, même 40 défaites anciennes), ni en jeu
/// (15 défaites dans la visite).
#[test]
fn boss_battu_en_normal_jamais_de_proposition() {
    for cle in BOSS {
        let mut app = avec_sauvegarde(cle, &format!(
            "version=3\nniveau.{cle}=atteint:1 gouttes:000 defis:100 medaille:1 temps:2345 defaites:40 facile:1\n"
        ));
        assert!(app.sauvegarde.progres(cle).battu_en_normal());
        entrer_par_la_carte(&mut app, cle);
        for _ in 0..15 {
            assert!(!mourir(&mut app), "{cle} : jamais en jeu non plus");
        }
        assert_eq!(app.facile.defaites_visite, 15, "{cle}");
    }
}
