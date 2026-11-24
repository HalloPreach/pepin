//! Tests de la machine à écrans (déplacés tels quels de `app.rs`, carte 10.2b).

use pepin_core::entities::contacts::Contact;
use pepin_core::entities::Entite;
use pepin_core::levels::{self, Apres, GenreNiveau};
use pepin_core::player::Entrees;
use pepin_core::state::EtatJeu;
use pepin_core::state::POINTS_GOUTTE;
use pepin_core::vies::{VIES_DEPART, VIES_MAX};

use crate::contenu;
use crate::ecrans::fin::Bilan;
use crate::ecrans::histoire::{self, Chapitre};
use crate::ecrans::{self, carte, plus_de_vies, Ecran};
use crate::entrees::{Commandes, RaccourciNiveau};
use crate::hud;
use crate::menu::{Choix, Menu};
use crate::navigation::Navigation;
use crate::sauvegarde::Sauvegarde;

use super::*;

fn rien() -> Navigation {
    Navigation::default()
}

fn valider() -> Navigation {
    Navigation {
        valider: true,
        ..Navigation::default()
    }
}

fn pause() -> Commandes {
    Commandes {
        pause: true,
        ..Commandes::default()
    }
}

/// Fait avancer `n` pas sans entrée.
fn avancer(app: &mut Application, n: u32) {
    for _ in 0..n {
        app.pas(&Entrees::default());
    }
}

/// Avance jusqu'à ce que l'écran satisfasse `ok` (borné pour ne jamais boucler).
/// Une image de l'histoire (carte 8.4) qui n'est pas l'écran attendu est passée
/// d'un appui sur Entrée dès qu'elle l'accepte, comme le ferait le joueur. De même
/// pour la carte du niveau et « PLUS DE VIES », qui ne se passent plus seuls (carte
/// 9.31) ; quand c'est l'écran attendu, on attend qu'il accepte l'appui, pour que
/// le test puisse le passer aussitôt.
fn jusqu_a(app: &mut Application, ok: impl Fn(Ecran) -> bool) {
    for _ in 0..600 {
        let texte = matches!(app.ecran, Ecran::Carte(_) | Ecran::PlusDeVies(_));
        if ok(app.ecran) && !app.transition.est_active() && (!texte || histoire::prete(app.anim)) {
            return;
        }
        if app.texte_en_attente() {
            app.reagir(&valider(), &Commandes::default(), None);
        }
        app.pas(&Entrees::default());
    }
    panic!("écran attendu jamais atteint (écran : {:?})", app.ecran);
}

/// Lance une partie depuis le titre et attend la fin de la carte du niveau 1.
fn en_jeu() -> Application {
    let mut app = Application::new(true);
    app.reagir(&valider(), &Commandes::default(), None);
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    passer_la_carte(&mut app);
    app
}

/// Le joueur passe la carte (prête) d'un appui ; le jeu commence au pas suivant,
/// sans que le monde ait bougé (carte 9.31).
fn passer_la_carte(app: &mut Application) {
    app.reagir(&valider(), &Commandes::default(), None);
    assert_eq!(app.ecran, Ecran::Carte(carte::PASSEE));
    let pas = app.monde.pas;
    avancer(app, 1);
    assert_eq!(app.ecran, Ecran::Jeu);
    assert_eq!(app.monde.pas, pas, "aucun pas de jeu pendant celui-ci");
}

/// Carte 9.1b (AGA : « c'est juste la suite du jeu ») : jamais de ligne MONDE 2 au
/// titre ; JOUER reprend là où on en est : le Monde 1 tant que `poursuite` n'est
/// pas fini, le Monde 2 ensuite (aussi quand tout est fini).
#[test]
fn jouer_reprend_la_ou_on_en_est() {
    let mut app = Application::new(true);
    app.sauvegarde =
        Sauvegarde::depuis_texte("version=3\nparties_finies=1\nniveau.poursuite=medaille:1");
    app.ouvrir(Ecran::Titre);
    let mut attendu = vec![
        Choix::Jouer,
        Choix::NouvellePartiePlus,
        Choix::Carte,
        Choix::Boutique,
        Choix::Histoire,
        Choix::Aide,
    ];
    // NOUVELLE PARTIE + désactivée pour l'instant (AGA) : absente du titre.
    if !NOUVELLE_PARTIE_PLUS_OUVERTE {
        attendu.retain(|&c| c != Choix::NouvellePartiePlus);
    }
    assert_eq!(app.choix, attendu);
    // Porte ouverte : JOUER (1re ligne) lance la 1re étape du Monde 2.
    app.reagir(&valider(), &Commandes::default(), None);
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    assert_eq!(app.monde.infos.monde, 2);
    assert_eq!(
        Some(app.monde.indice),
        levels::debut_monde(niveaux::niveaux(), 2)
    );
    // AGA : trois niveaux du Monde 2 finis, JOUER reprend au 4e (pas au 1er).
    let mut app = Application::new(true);
    app.sauvegarde = Sauvegarde::depuis_texte(
        "version=3\nparties_finies=1\nniveau.poursuite=medaille:1\nniveau.caniveau=medaille:1\nniveau.egouts=medaille:1\nniveau.cascade=medaille:1",
    );
    app.ouvrir(Ecran::Titre);
    app.reagir(&valider(), &Commandes::default(), None);
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    assert_eq!(app.monde.infos.monde, 2);
    assert_eq!(
        app.monde.indice,
        levels::debut_monde(niveaux::niveaux(), 2).unwrap() + 3
    );
    // AGA : les gouttes déjà trouvées dans les niveaux d'avant restent comptées.
    let mut app = Application::new(true);
    app.sauvegarde = Sauvegarde::depuis_texte(
        "version=3\nparties_finies=1\nniveau.poursuite=medaille:1\nniveau.caniveau=medaille:1\nniveau.egouts=medaille:1\nniveau.cascade=medaille:1",
    );
    let d = levels::debut_monde(niveaux::niveaux(), 2).unwrap();
    if let Some(p) = app.sauvegarde.progres_mut(niveaux::niveaux()[d].cle) {
        p.gouttes = [true, true, false];
    }
    if let Some(p) = app.sauvegarde.progres_mut(niveaux::niveaux()[d + 2].cle) {
        p.gouttes = [true, false, false];
    }
    app.ouvrir(Ecran::Titre);
    app.reagir(&valider(), &Commandes::default(), None);
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    assert_eq!(app.monde.indice, d + 3);
    assert_eq!(app.monde.etat.gouttes, 3);
    // Porte fermée (`poursuite` pas fini) : JOUER lance le Monde 1, comme avant.
    app.sauvegarde = Sauvegarde::depuis_texte("version=3\nparties_finies=1");
    assert_eq!(app.monde_de_jouer(), 1);
    // AGA : « qu'il reprenne ou j'en suis quoi qu'il arrive » : au Monde 1 aussi,
    // deux niveaux finis, JOUER reprend au 3e, sans les images de l'histoire.
    let mut app = Application::new(true);
    app.sauvegarde = Sauvegarde::depuis_texte(&format!(
        "version=3\nniveau.{}=medaille:1\nniveau.{}=medaille:1",
        niveaux::niveaux()[0].cle,
        niveaux::niveaux()[1].cle
    ));
    app.ouvrir(Ecran::Titre);
    app.reagir(&valider(), &Commandes::default(), None);
    assert!(!app.partie_racontee);
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    assert_eq!(app.monde.infos.monde, 1);
    assert_eq!(app.monde.indice, 2);
}

#[test]
fn les_menus_ont_les_bonnes_lignes() {
    assert_eq!(
        choix_pour(Ecran::Titre),
        vec![
            Choix::Jouer,
            Choix::NouvellePartiePlus,
            Choix::Carte,
            Choix::Boutique,
            Choix::Histoire,
            Choix::Aide
        ]
    );
    assert_eq!(
        choix_pour(Ecran::BilanNiveau),
        vec![Choix::RejouerNiveau, Choix::Carte]
    );
    assert_eq!(
        choix_pour(Ecran::Pause),
        vec![Choix::Reprendre, Choix::Recommencer, Choix::QuitterAuTitre]
    );
    assert_eq!(
        choix_pour(Ecran::Fin),
        vec![Choix::Rejouer, Choix::RetourTitre]
    );
    assert!(choix_pour(Ecran::Jeu).is_empty());
}

#[test]
fn on_demarre_au_titre_son_coupe() {
    let app = Application::new(true);
    assert_eq!(app.ecran, Ecran::Titre);
    assert!(!app.son);
}

#[test]
fn jouer_passe_par_la_transition_puis_la_carte_puis_le_jeu() {
    let mut app = Application::new(true);
    app.reagir(&valider(), &Commandes::default(), None);
    assert_eq!(
        app.ecran,
        Ecran::Titre,
        "pas de changement avant les volets"
    );
    assert!(app.transition.est_active());
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    // La carte attend un appui (carte 9.31) : le joueur valide.
    passer_la_carte(&mut app);
    assert_eq!(app.monde.infos.numero, 1);
}

/// Carte 9.31 (AGA : « qu'on soit obligé d'appuyer pour passer les textes ») : sans
/// appui, la carte du niveau reste affichée, même 10 s, et le monde reste figé.
#[test]
fn la_carte_reste_affichee_dix_secondes_sans_appui() {
    let mut app = Application::new(true);
    app.reagir(&valider(), &Commandes::default(), None);
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    avancer(&mut app, 10 * 60);
    assert!(matches!(app.ecran, Ecran::Carte(_)), "{:?}", app.ecran);
    assert_eq!(app.monde.pas, 0, "le monde attend le joueur");
    // « PLUS DE VIES » non plus ne se passe pas seul.
    let mut app = en_jeu();
    for _ in 0..VIES_DEPART {
        tomber(&mut app);
    }
    avancer(&mut app, 10 * 60);
    assert!(matches!(app.ecran, Ecran::PlusDeVies(_)), "{:?}", app.ecran);
}

/// Carte 9.31 : un appui pendant le court délai d'ouverture (un saut encore tenu)
/// est ignoré ; après le délai, il passe la carte, et « PLUS DE VIES » de même.
#[test]
fn un_appui_avant_le_delai_est_ignore_apres_il_passe() {
    let mut app = Application::new(true);
    app.reagir(&valider(), &Commandes::default(), None);
    // Dès la fin des volets, la carte est là mais pas encore prête (l'image de
    // l'histoire d'avant est passée d'un appui, comme dans `jusqu_a`).
    for _ in 0..600 {
        if matches!(app.ecran, Ecran::Carte(_)) && !app.transition.est_active() {
            break;
        }
        if app.texte_en_attente() {
            app.reagir(&valider(), &Commandes::default(), None);
        }
        app.pas(&Entrees::default());
    }
    assert!(matches!(app.ecran, Ecran::Carte(_)));
    assert!(!histoire::prete(app.anim), "anim = {}", app.anim);
    app.reagir(&valider(), &Commandes::default(), None);
    assert!(matches!(app.ecran, Ecran::Carte(_)), "trop tôt : ignoré");
    while !histoire::prete(app.anim) {
        app.pas(&Entrees::default());
    }
    passer_la_carte(&mut app);
    // « PLUS DE VIES » : ouvert à anim = 0, un appui aussitôt est ignoré.
    for _ in 0..VIES_DEPART {
        tomber(&mut app);
    }
    assert!(matches!(app.ecran, Ecran::PlusDeVies(_)));
    app.reagir(&valider(), &Commandes::default(), None);
    assert!(!app.transition.est_active(), "trop tôt : ignoré");
    avancer(&mut app, histoire::DELAI_PAS);
    app.reagir(&valider(), &Commandes::default(), None);
    assert!(app.transition.est_active(), "les volets partent");
}

/// Carte 9.31 : l'invitation « continuer » n'apparaît qu'une fois l'appui accepté,
/// puis clignote doucement (0,5 s allumée, 0,5 s éteinte), comme dans l'histoire.
#[test]
fn l_invitation_clignote_une_fois_l_appui_accepte() {
    assert!(!histoire::invite_visible(0));
    assert!(!histoire::invite_visible(histoire::DELAI_PAS - 1));
    assert!(histoire::invite_visible(histoire::DELAI_PAS));
    assert!(!histoire::invite_visible(histoire::DELAI_PAS + 30));
    assert!(histoire::invite_visible(histoire::DELAI_PAS + 60));
}

#[test]
fn la_carte_se_passe_au_clavier_ou_au_toucher() {
    let mut app = Application::new(true);
    app.reagir(&valider(), &Commandes::default(), None);
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    let toucher = Navigation {
        clic: Some((10.0, 10.0)),
        ..Navigation::default()
    };
    app.reagir(&toucher, &Commandes::default(), None);
    avancer(&mut app, 1);
    assert_eq!(app.ecran, Ecran::Jeu);
}

#[test]
fn le_monde_est_fige_pendant_la_carte() {
    let mut app = Application::new(true);
    app.reagir(&valider(), &Commandes::default(), None);
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    let avant = app.monde.pas;
    avancer(&mut app, 30);
    assert_eq!(app.monde.pas, avant);
}

#[test]
fn la_pause_va_et_vient() {
    let mut app = en_jeu();
    app.reagir(&rien(), &pause(), None);
    assert_eq!(app.ecran, Ecran::Pause);
    let pas = app.monde.pas;
    avancer(&mut app, 20);
    assert_eq!(app.monde.pas, pas, "le monde ne bouge pas en pause");
    app.reagir(&rien(), &pause(), None);
    assert_eq!(app.ecran, Ecran::Jeu);
}

#[test]
fn echap_ferme_la_pause_aussi() {
    let mut app = en_jeu();
    app.reagir(&rien(), &pause(), None);
    let retour = Navigation {
        retour: true,
        ..Navigation::default()
    };
    app.reagir(&retour, &Commandes::default(), None);
    assert_eq!(app.ecran, Ecran::Jeu);
}

#[test]
fn quitter_au_titre_depuis_la_pause() {
    let mut app = en_jeu();
    app.reagir(&rien(), &pause(), None);
    let bas = Navigation {
        bas: true,
        ..Navigation::default()
    };
    // Reprendre, Recommencer, Quitter : on descend en bas en bouclant par le haut.
    let haut = Navigation {
        haut: true,
        ..Navigation::default()
    };
    app.reagir(&haut, &Commandes::default(), None);
    assert_eq!(app.choix[app.menu.selection()], Choix::QuitterAuTitre);
    app.reagir(&bas, &Commandes::default(), None);
    app.reagir(&haut, &Commandes::default(), None);
    app.reagir(&valider(), &Commandes::default(), None);
    jusqu_a(&mut app, |e| e == Ecran::Titre);
    assert_eq!(app.ecran, Ecran::Titre);
}

#[test]
fn la_touche_m_bascule_le_son_seulement_s_il_est_disponible() {
    let m = Navigation {
        son: true,
        ..Navigation::default()
    };
    let mut avec = Application::new(true);
    avec.reagir(&m, &Commandes::default(), None);
    assert!(avec.son);
    avec.reagir(&m, &Commandes::default(), None);
    assert!(!avec.son);
    let mut sans = Application::new(false);
    sans.reagir(&m, &Commandes::default(), None);
    assert!(!sans.son);
}

#[test]
fn l_icone_du_son_bascule_sans_quitter_le_menu() {
    // Carte 7.8b : plus de ligne SON ; un clic (ou toucher) sur l'icône en bas à
    // gauche, même un peu à côté, bascule le son et ne valide aucune ligne.
    let mut app = Application::new(true);
    let clic = Navigation {
        clic: Some((3.0, 177.0)),
        ..Navigation::default()
    };
    app.reagir(&clic, &Commandes::default(), None);
    assert!(app.son);
    assert_eq!(app.ecran, Ecran::Titre);
    assert!(!app.transition.est_active());
    app.reagir(&clic, &Commandes::default(), None);
    assert!(!app.son);
}

#[test]
fn un_clic_sur_une_ligne_la_valide() {
    let mut app = Application::new(true);
    let d = ecrans::disposition_menu(Ecran::Titre, app.choix.len()).unwrap();
    let ligne = app.choix.iter().position(|&c| c == Choix::Aide).unwrap();
    let (x, y, _, h) = d.rect_ligne(ligne);
    let clic = Navigation {
        clic: Some((x + 10.0, y + h / 2.0)),
        ..Navigation::default()
    };
    app.reagir(&clic, &Commandes::default(), None);
    assert_eq!(app.ecran, Ecran::Aide);
    // Un clic n'importe où ramène au titre, curseur sur « comment jouer ».
    app.reagir(&clic, &Commandes::default(), None);
    assert_eq!(app.ecran, Ecran::Titre);
    assert_eq!(app.choix[app.menu.selection()], Choix::Aide);
}

#[test]
fn un_clic_dans_le_vide_ne_fait_rien() {
    let mut app = Application::new(true);
    let clic = Navigation {
        clic: Some((2.0, 2.0)),
        ..Navigation::default()
    };
    app.reagir(&clic, &Commandes::default(), None);
    assert_eq!(app.ecran, Ecran::Titre);
    assert!(!app.transition.est_active());
}

#[test]
fn le_survol_deplace_le_curseur_sans_valider() {
    let mut app = Application::new(true);
    let d = ecrans::disposition_menu(Ecran::Titre, app.choix.len()).unwrap();
    let (x, y, _, h) = d.rect_ligne(1);
    let survol = Navigation {
        survol: Some((x + 10.0, y + h / 2.0)),
        ..Navigation::default()
    };
    app.reagir(&survol, &Commandes::default(), None);
    assert_eq!(app.menu.selection(), 1);
    assert_eq!(app.ecran, Ecran::Titre);
}

#[test]
fn recommencer_le_niveau_garde_les_morts_mais_rend_les_gouttes() {
    let mut app = en_jeu();
    app.monde.etat.gouttes = 2;
    app.monde.etat.score = 2 * POINTS_GOUTTE;
    app.monde.etat.morts = 3;
    app.reagir(&rien(), &pause(), None);
    app.valider_choix(Choix::Recommencer);
    jusqu_a(&mut app, |e| e == Ecran::Jeu);
    assert_eq!(
        app.monde.etat.gouttes, 0,
        "les gouttes du niveau sont à reprendre"
    );
    assert_eq!(app.monde.etat.morts, 3, "les morts restent comptées");
    assert_eq!(app.monde.pas, 0, "niveau tout neuf");
}

#[test]
fn finir_un_niveau_garde_le_score_et_ouvre_le_suivant() {
    let mut app = en_jeu();
    app.monde.etat.gouttes = 4;
    app.monde.termine = true;
    avancer(&mut app, 1);
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    assert_eq!(app.monde.infos.numero, 2);
    assert_eq!(app.monde.etat.gouttes, 4);
}

/// Le dernier niveau DU MONDE 1 (l'épilogue ; carte 9.6 : la table continue avec
/// le Monde 2) mène à l'écran de fin, avec le bilan de la partie.
#[test]
fn finir_le_dernier_niveau_ouvre_l_ecran_de_fin_avec_le_bilan() {
    let mut app = en_jeu();
    let (monde, camera) = charger(niveaux::NB_NIVEAUX_MONDE_1 - 1, None);
    app.monde = monde;
    app.camera = camera;
    app.monde.etat.gouttes = 9;
    app.monde.etat.morts = 2;
    app.monde.etat.score = 900;
    app.pas_partie = 3600;
    app.monde.termine = true;
    avancer(&mut app, 1);
    jusqu_a(&mut app, |e| e == Ecran::Fin);
    // Le dernier niveau est l'épilogue (carte 5.17) : l'écran de fin du retour.
    assert_eq!(
        app.bilan,
        Bilan {
            pas: 3601,
            gouttes: 9,
            morts: 2,
            vies: VIES_DEPART,
            score: 900,
            gouttes_bonus: None,
            epilogue: true,
            lettres: Some([false; 5]),
            monde: 1,
        }
    );
    assert_eq!(app.choix, vec![Choix::Rejouer, Choix::RetourTitre]);
}

/// AGA : le Monde 1 fini (porte du Monde 2 ouverte), l'écran de fin propose
/// CONTINUER en premier, qui lance le niveau 1 du Monde 2 sans passer par la carte.
#[test]
fn finir_le_monde_1_enchaine_sur_le_monde_2() {
    let mut app = en_jeu();
    app.sauvegarde = Sauvegarde::depuis_texte("version=3\nniveau.poursuite=medaille:1");
    let (monde, camera) = charger(niveaux::NB_NIVEAUX_MONDE_1 - 1, None);
    app.monde = monde;
    app.camera = camera;
    app.monde.termine = true;
    avancer(&mut app, 1);
    jusqu_a(&mut app, |e| e == Ecran::Fin);
    assert_eq!(
        app.choix,
        vec![Choix::Continuer, Choix::Rejouer, Choix::RetourTitre]
    );
    assert_eq!(app.choix[app.menu.selection()], Choix::Continuer);
    app.reagir(&valider(), &Commandes::default(), None);
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    assert_eq!(app.monde.infos.monde, 2);
    assert_eq!(
        Some(app.monde.indice),
        levels::debut_monde(niveaux::niveaux(), 2)
    );
}

#[test]
fn rejouer_remet_la_partie_a_zero() {
    let mut app = en_jeu();
    app.monde.etat.gouttes = 5;
    app.pas_partie = 1234;
    app.monde.termine = false;
    app.ecran = Ecran::Fin;
    app.choix = choix_pour(Ecran::Fin);
    app.menu = Menu::new(2);
    app.reagir(&valider(), &Commandes::default(), None);
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    assert_eq!(app.monde.infos.numero, 1);
    assert_eq!(app.monde.etat.gouttes, 0);
    assert_eq!(app.pas_partie, 0);
}

#[test]
fn le_raccourci_de_mise_au_point_charge_le_niveau_demande() {
    let mut app = en_jeu();
    app.reagir(
        &rien(),
        &Commandes::default(),
        Some(RaccourciNiveau::Aller(3)),
    );
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    assert_eq!(app.monde.infos.numero, 4);
}

/// Touche 0 du build de QA : le niveau d'essai vertical (hors de la table), avec sa
/// caméra qui suit en hauteur ; le finir mène à l'écran de fin (rien après lui).
#[test]
fn le_raccourci_d_essai_charge_le_niveau_vertical() {
    let mut app = en_jeu();
    app.reagir(&rien(), &Commandes::default(), Some(RaccourciNiveau::Essai));
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    assert_eq!(app.monde.indice, crate::monde::INDICE_ESSAI);
    assert_eq!(app.monde.infos.nom, "Essai vertical");
    assert_eq!(app.monde.niveau.hauteur(), 45);
    // Départ en bas du niveau : caméra sur sa borne basse (720 - 180).
    assert_eq!(app.camera.y, 540.0);
    app.monde.termine = true;
    app.reagir(&valider(), &Commandes::default(), None);
    avancer(&mut app, 1);
    jusqu_a(&mut app, |e| e == Ecran::Fin);
}

/// Touche F2 du build de QA : le niveau d'essai « grêle » (hors de la table),
/// avec ses quatre colonnes de grêle.
#[test]
fn le_raccourci_d_essai_charge_le_niveau_de_grele() {
    let mut app = en_jeu();
    app.reagir(
        &rien(),
        &Commandes::default(),
        Some(RaccourciNiveau::Essai3),
    );
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    assert_eq!(app.monde.indice, crate::monde::INDICE_ESSAI_GRELE);
    assert_eq!(app.monde.infos.nom, "Essai grêle");
    let greles = app
        .monde
        .entites
        .iter()
        .filter(|e| matches!(e, pepin_core::entities::Entite::Grele(_)))
        .count();
    assert_eq!(greles, 4);
}

/// Touche F3 du build de QA : le niveau d'essai « auvent » (hors de la table),
/// avec ses six auvents.
#[test]
fn le_raccourci_d_essai_charge_le_niveau_des_auvents() {
    let mut app = en_jeu();
    app.reagir(
        &rien(),
        &Commandes::default(),
        Some(RaccourciNiveau::Essai4),
    );
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    assert_eq!(app.monde.indice, crate::monde::INDICE_ESSAI_AUVENT);
    assert_eq!(app.monde.infos.nom, "Essai auvent");
    let auvents = app
        .monde
        .entites
        .iter()
        .filter(|e| matches!(e, pepin_core::entities::Entite::Auvent(_)))
        .count();
    assert_eq!(auvents, 6);
}

/// Touche F5 du build de QA : le niveau d'essai « montgolfière » (hors de la
/// table), avec ses deux montgolfières.
#[test]
fn le_raccourci_d_essai_charge_le_niveau_des_montgolfieres() {
    let mut app = en_jeu();
    app.reagir(
        &rien(),
        &Commandes::default(),
        Some(RaccourciNiveau::Essai6),
    );
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    assert_eq!(app.monde.indice, crate::monde::INDICE_ESSAI_MONTGOLFIERE);
    assert_eq!(app.monde.infos.nom, "Essai montgolfière");
    let montgolfieres = app
        .monde
        .entites
        .iter()
        .filter(|e| matches!(e, pepin_core::entities::Entite::Montgolfiere(_)))
        .count();
    assert_eq!(montgolfieres, 2);
}

/// Touche F6 du build de QA : le niveau d'essai « escargot » (hors de la
/// table), avec ses deux escargots.
#[test]
fn le_raccourci_d_essai_charge_le_niveau_des_escargots() {
    let mut app = en_jeu();
    app.reagir(
        &rien(),
        &Commandes::default(),
        Some(RaccourciNiveau::Essai7),
    );
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    assert_eq!(app.monde.indice, crate::monde::INDICE_ESSAI_ESCARGOT);
    assert_eq!(app.monde.infos.nom, "Essai escargot");
    let escargots = app
        .monde
        .entites
        .iter()
        .filter(|e| matches!(e, pepin_core::entities::Entite::Escargot(_)))
        .count();
    assert_eq!(escargots, 2);
}

/// Touche F7 du build de QA : le niveau d'essai « chat » (hors de la table),
/// avec son chat endormi.
#[test]
fn le_raccourci_d_essai_charge_le_niveau_du_chat() {
    let mut app = en_jeu();
    app.reagir(
        &rien(),
        &Commandes::default(),
        Some(RaccourciNiveau::Essai8),
    );
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    assert_eq!(app.monde.indice, crate::monde::INDICE_ESSAI_CHAT);
    assert_eq!(app.monde.infos.nom, "Essai chat");
    let chats = app
        .monde
        .entites
        .iter()
        .filter(|e| {
            matches!(e, pepin_core::entities::Entite::Ennemi(c)
                if c.type_ennemi == pepin_core::entities::TypeEnnemi::Chat)
        })
        .count();
    assert_eq!(chats, 1);
}

/// Touche F10 du build de QA : l'arène du Grand Nuage (hors de la table), avec
/// son nuage à 3 points de vie, et sans drapeau.
#[test]
fn le_raccourci_d_essai_charge_l_arene_du_grand_nuage() {
    let mut app = en_jeu();
    app.reagir(
        &rien(),
        &Commandes::default(),
        Some(RaccourciNiveau::Essai10),
    );
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    assert_eq!(app.monde.indice, crate::monde::INDICE_ESSAI_NUAGE);
    assert_eq!(app.monde.infos.nom, "Essai nuage");
    assert_eq!(app.monde.pv_nuage(), Some(3));
    assert!(app.monde.niveau.arrivee.is_none());
}

/// Touche F8 du build de QA : le niveau d'essai « horloge » (hors de la
/// table), avec ses deux aiguilles.
#[test]
fn le_raccourci_d_essai_charge_le_niveau_de_l_horloge() {
    let mut app = en_jeu();
    app.reagir(
        &rien(),
        &Commandes::default(),
        Some(RaccourciNiveau::Essai9),
    );
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    assert_eq!(app.monde.indice, crate::monde::INDICE_ESSAI_HORLOGE);
    assert_eq!(app.monde.infos.nom, "Essai horloge");
    let aiguilles = app
        .monde
        .entites
        .iter()
        .filter(|e| matches!(e, pepin_core::entities::Entite::Mobile(m) if m.cercle().is_some()))
        .count();
    assert_eq!(aiguilles, 2);
}

#[test]
fn les_raccourcis_suivant_et_precedent_parcourent_la_table() {
    let mut app = en_jeu();
    // Au premier niveau, « précédent » ne fait rien (aucune transition).
    app.reagir(
        &rien(),
        &Commandes::default(),
        Some(RaccourciNiveau::Precedent),
    );
    assert!(!app.transition.est_active());
    // « Suivant » jusqu'au dernier niveau, en passant par tous.
    for attendu in 1..niveaux::NB_NIVEAUX {
        app.reagir(
            &rien(),
            &Commandes::default(),
            Some(RaccourciNiveau::Suivant),
        );
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        passer_la_carte(&mut app);
        assert_eq!(app.monde.indice, attendu);
    }
    app.reagir(
        &rien(),
        &Commandes::default(),
        Some(RaccourciNiveau::Suivant),
    );
    assert!(!app.transition.est_active(), "rien après le dernier niveau");
    app.reagir(
        &rien(),
        &Commandes::default(),
        Some(RaccourciNiveau::Precedent),
    );
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    assert_eq!(app.monde.indice, niveaux::NB_NIVEAUX - 2);
}

/// Chaque niveau terminé mène là où le dit `levels::apres` (avec les gouttes de la
/// partie) : le niveau suivant, ou l'écran de fin.
#[test]
fn la_suite_de_chaque_niveau_suit_la_table() {
    for indice in 0..niveaux::NB_NIVEAUX {
        for gouttes in [0, levels::gouttes_pour_le_bonus(niveaux::niveaux())] {
            let mut app = en_jeu();
            let (monde, camera) = charger(indice, None);
            app.monde = monde;
            app.camera = camera;
            app.monde.etat.gouttes = gouttes;
            app.monde.termine = true;
            avancer(&mut app, 1);
            match levels::apres(niveaux::niveaux(), indice, gouttes) {
                Apres::Niveau(j) => {
                    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
                    assert_eq!(app.monde.indice, j);
                    assert_eq!(app.monde.etat.gouttes, gouttes, "gouttes gardées");
                }
                Apres::Fin => jusqu_a(&mut app, |e| e == Ecran::Fin),
            }
        }
    }
}

/// Les DEUX chemins au sortir du 10e niveau (le dernier normal, cartes 5.16, 5.17
/// et 8.2) : avec les 30 gouttes des niveaux normaux, la carte suivante est celle du
/// bonus (« NIVEAU BONUS », l'arc-en-ciel), puis celle de l'épilogue ; avec une de
/// moins, directement l'épilogue (« ÉPILOGUE », la descente). L'épilogue fini,
/// c'est l'écran de fin du retour : les gouttes du parcours sur 33, et celles du
/// bonus à part s'il a été joué.
#[test]
fn apres_le_neuvieme_le_bonus_seulement_avec_toutes_les_gouttes_puis_l_epilogue() {
    use pepin_core::levels::nb_normaux;
    for (gouttes, bonus) in [(30, true), (29, false), (0, false)] {
        let mut app = en_jeu();
        let (monde, camera) = charger(9, None);
        app.monde = monde;
        app.camera = camera;
        assert_eq!(app.monde.infos.numero, 10);
        app.monde.etat.gouttes = gouttes;
        app.monde.termine = true;
        avancer(&mut app, 1);
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        let mut gouttes_finales = gouttes;
        if bonus {
            assert_eq!(app.monde.indice, 10);
            assert_eq!(app.monde.infos.nom, niveaux::par_cle("arc").nom);
            assert_eq!(app.monde.infos.genre, GenreNiveau::Bonus);
            assert_eq!(
                contenu::etiquette_niveau(app.monde.infos, nb_normaux(niveaux::niveaux())),
                "NIVEAU BONUS"
            );
            // Le bonus fini avec 2 de ses gouttes (32 dans la partie) : l'épilogue.
            passer_la_carte(&mut app);
            app.monde.gouttes_niveau = [true, false, true];
            app.monde.etat.gouttes = 32;
            app.monde.termine = true;
            avancer(&mut app, 1);
            assert_eq!(app.gouttes_bonus, Some(2));
            jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
            gouttes_finales = 32;
        }
        assert_eq!(app.monde.indice, 11);
        assert_eq!(app.monde.infos.nom, niveaux::par_cle("descente").nom);
        assert_eq!(app.monde.infos.genre, GenreNiveau::Epilogue);
        assert_eq!(
            contenu::etiquette_niveau(app.monde.infos, nb_normaux(niveaux::niveaux())),
            "ÉPILOGUE"
        );
        assert_eq!(app.monde.etat.gouttes, gouttes_finales, "gouttes gardées");
        // L'épilogue fini avec ses 3 gouttes : l'écran de fin du retour.
        app.reagir(&valider(), &Commandes::default(), None);
        app.monde.etat.gouttes += 3;
        app.monde.termine = true;
        avancer(&mut app, 1);
        jusqu_a(&mut app, |e| e == Ecran::Fin);
        assert!(app.bilan.epilogue);
        if bonus {
            assert_eq!((app.bilan.gouttes, app.bilan.gouttes_bonus), (33, Some(2)));
        } else {
            assert_eq!(
                (app.bilan.gouttes, app.bilan.gouttes_bonus),
                (gouttes + 3, None)
            );
        }
    }
}

/// Une nouvelle partie oublie le bonus de la précédente.
#[test]
fn rejouer_oublie_les_gouttes_du_bonus() {
    let mut app = en_jeu();
    app.gouttes_bonus = Some(3);
    app.ecran = Ecran::Fin;
    app.choix = choix_pour(Ecran::Fin);
    app.menu = Menu::new(2);
    app.reagir(&valider(), &Commandes::default(), None);
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    assert_eq!(app.gouttes_bonus, None);
}

#[test]
fn rien_ne_reagit_pendant_une_transition() {
    let mut app = Application::new(true);
    app.reagir(&valider(), &Commandes::default(), None);
    assert!(app.transition.est_active());
    // Deuxième appui pendant les volets : ignoré (pas de nouvelle demande).
    app.reagir(
        &valider(),
        &Commandes::default(),
        Some(RaccourciNiveau::Aller(2)),
    );
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    assert_eq!(app.monde.infos.numero, 1);
}

#[test]
fn le_chrono_ne_compte_que_le_jeu() {
    let mut app = en_jeu();
    avancer(&mut app, 100);
    assert_eq!(app.pas_partie, 100);
    app.reagir(&rien(), &pause(), None);
    avancer(&mut app, 100);
    assert_eq!(app.pas_partie, 100, "rien ne compte en pause");
}

// --- Perte de focus (onglet masqué, autre application, téléphone verrouillé) ---

#[test]
fn la_perte_de_focus_en_partie_ouvre_la_pause_et_fige_le_temps() {
    let mut app = en_jeu();
    avancer(&mut app, 30);
    let (pas_monde, chrono) = (app.monde.pas, app.pas_partie);
    app.focus(false);
    assert_eq!(app.ecran, Ecran::Pause);
    assert_eq!(app.choix[app.menu.selection()], Choix::Reprendre);
    // Retour sur l'onglet : la pause RESTE (seul le joueur en sort).
    app.focus(true);
    assert_eq!(app.ecran, Ecran::Pause);
    avancer(&mut app, 600);
    assert_eq!(app.monde.pas, pas_monde, "le monde ne bouge pas");
    assert_eq!(app.pas_partie, chrono, "le chrono ne compte pas l'absence");
    // Reprise : le temps repart exactement d'où il était, sans rattrapage.
    app.reagir(&rien(), &pause(), None);
    assert_eq!(app.ecran, Ecran::Jeu);
    avancer(&mut app, 1);
    assert_eq!(app.monde.pas, pas_monde + 1);
    assert_eq!(app.pas_partie, chrono + 1);
}

#[test]
fn la_pause_ne_s_ouvre_pas_deux_fois() {
    let mut app = en_jeu();
    app.focus(false);
    let bas = Navigation {
        bas: true,
        ..Navigation::default()
    };
    app.reagir(&bas, &Commandes::default(), None);
    let curseur = app.menu.selection();
    assert_eq!(app.choix[curseur], Choix::Recommencer);
    // Nouvelle perte (masqué puis fenêtre quittée...) : l'écran n'est pas rouvert.
    app.focus(false);
    app.focus(false);
    assert_eq!(app.ecran, Ecran::Pause);
    assert_eq!(
        app.menu.selection(),
        curseur,
        "le curseur n'est pas remis en haut"
    );
    // Pause manuelle puis perte de focus : toujours la même pause.
    app.reagir(&rien(), &pause(), None);
    assert_eq!(app.ecran, Ecran::Jeu);
    app.reagir(&rien(), &pause(), None);
    app.focus(false);
    assert_eq!(app.ecran, Ecran::Pause);
}

#[test]
fn la_perte_de_focus_hors_partie_n_ouvre_rien() {
    // Titre
    let mut app = Application::new(true);
    app.focus(false);
    app.focus(true);
    assert_eq!(app.ecran, Ecran::Titre);
    // Aide
    app.valider_choix(Choix::Aide);
    app.focus(false);
    assert_eq!(app.ecran, Ecran::Aide);
    // Fin
    let mut app = en_jeu();
    app.ecran = Ecran::Fin;
    app.focus(false);
    assert_eq!(app.ecran, Ecran::Fin);
    // Volets vers le titre (« quitter ») : rien, ni maintenant ni après.
    let mut app = en_jeu();
    app.reagir(&rien(), &pause(), None);
    app.valider_choix(Choix::QuitterAuTitre);
    app.focus(false);
    jusqu_a(&mut app, |e| e == Ecran::Titre);
    assert!(!app.pause_differee);
}

#[test]
fn focus_perdu_pendant_la_carte_la_pause_attend_le_jeu() {
    let mut app = Application::new(true);
    app.reagir(&valider(), &Commandes::default(), None);
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    app.focus(false);
    assert!(
        matches!(app.ecran, Ecran::Carte(_)),
        "pas de pause sur la carte"
    );
    // La carte attend (carte 9.31), puis elle est passée sans que le focus soit
    // revenu : la pause s'ouvre au premier pas de jeu, AVANT que le monde ne bouge.
    avancer(&mut app, 600);
    assert!(matches!(app.ecran, Ecran::Carte(_)));
    app.reagir(&valider(), &Commandes::default(), None);
    avancer(&mut app, 2);
    assert_eq!(app.ecran, Ecran::Pause);
    assert_eq!(app.monde.pas, 0);
    assert_eq!(app.pas_partie, 0);
    // Une seule fois : après la reprise, le jeu avance normalement.
    app.reagir(&rien(), &pause(), None);
    avancer(&mut app, 10);
    assert_eq!(app.ecran, Ecran::Jeu);
    assert_eq!(app.monde.pas, 10);
}

#[test]
fn focus_revenu_pendant_la_carte_aucune_pause() {
    let mut app = Application::new(true);
    app.reagir(&valider(), &Commandes::default(), None);
    // Perte pendant les volets, retour pendant la carte (onglet masqué puis revenu).
    app.focus(false);
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    app.focus(true);
    app.reagir(&valider(), &Commandes::default(), None);
    avancer(&mut app, 12);
    assert_eq!(app.ecran, Ecran::Jeu);
    assert!(app.monde.pas > 0);
}

#[test]
fn focus_perdu_pendant_les_volets_de_recommencer() {
    // Volets qui se ferment (la suite « recommencer » est en attente).
    let mut app = en_jeu();
    app.reagir(&rien(), &pause(), None);
    app.valider_choix(Choix::Recommencer);
    app.focus(false);
    // Les volets se referment sur la pause, s'ouvrent sur le niveau tout neuf, puis la
    // pause différée s'ouvre au premier pas de jeu.
    jusqu_a(&mut app, |e| e == Ecran::Pause);
    avancer(&mut app, 30);
    assert_eq!(
        app.ecran,
        Ecran::Pause,
        "le niveau recommencé attend le joueur"
    );
    assert_eq!(app.monde.pas, 0);
    // Volets qui s'OUVRENT sur le niveau recommencé (plus rien en attente).
    app.valider_choix(Choix::Recommencer);
    for _ in 0..200 {
        if app.ecran == Ecran::Jeu {
            break;
        }
        avancer(&mut app, 1);
    }
    assert!(app.ecran == Ecran::Jeu && app.transition.est_active());
    app.focus(false);
    jusqu_a(&mut app, |e| e == Ecran::Pause);
    assert_eq!(app.monde.pas, 0);
}

#[test]
fn focus_perdu_en_allant_vers_la_fin_aucune_pause_plus_tard() {
    let mut app = en_jeu();
    let (monde, camera) = charger(niveaux::NB_NIVEAUX_MONDE_1 - 1, None);
    app.monde = monde;
    app.camera = camera;
    app.monde.termine = true;
    avancer(&mut app, 1);
    // Volets vers l'écran de fin : perte de focus, la fenêtre reste sans focus.
    app.focus(false);
    jusqu_a(&mut app, |e| e == Ecran::Fin);
    // « Rejouer » (au clic, sans que le focus soit revenu) : le jeu démarre normalement.
    app.valider_choix(Choix::Rejouer);
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    passer_la_carte(&mut app);
    avancer(&mut app, 5);
    assert_eq!(app.ecran, Ecran::Jeu);
    assert_eq!(app.monde.pas, 5);
}

#[test]
fn la_musique_et_la_pluie_s_arretent_pendant_la_pause() {
    let app = Application::new(true);
    assert!(app.boucles_sonores(), "titre");
    let mut app = en_jeu();
    assert!(app.boucles_sonores(), "jeu");
    app.focus(false);
    assert!(!app.boucles_sonores(), "pause automatique");
    app.focus(true);
    assert!(
        !app.boucles_sonores(),
        "retour sur l'onglet : toujours en pause"
    );
    app.reagir(&rien(), &pause(), None);
    assert!(app.boucles_sonores(), "le joueur sort de la pause");
    app.reagir(&rien(), &pause(), None);
    assert!(!app.boucles_sonores(), "pause manuelle");
}
// --- Vies (carte 6.2) ------------------------------------------------------------

/// Fait tomber Pépin dans le vide : une mort au pas suivant (le vide tue même
/// pendant la protection de réapparition).
/// Fait tomber Pépin dans le vide, puis attend la fin du temps de mort (carte
/// 9.24) ou l'écran « plus de vies ».
fn tomber(app: &mut Application) {
    let bas = app.monde.niveau.hauteur_px();
    let x = app.monde.joueur.boite.x;
    app.monde.joueur.placer(x, bas + 100.0);
    avancer(app, 1);
    while app.ecran == Ecran::Jeu && app.monde.reapparition > 0 {
        avancer(app, 1);
    }
}

/// Pose Pépin sur chacune des gouttes d'or du niveau, une par pas.
fn ramasser_les_gouttes(app: &mut Application) {
    let gouttes: Vec<(f32, f32)> = app
        .monde
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Goutte(g) => Some((g.boite.x, g.boite.y)),
            _ => None,
        })
        .collect();
    assert_eq!(gouttes.len(), 3);
    for (x, y) in gouttes {
        app.monde.joueur.placer(x, y);
        avancer(app, 1);
    }
    assert_eq!(app.monde.etat.gouttes_ici(), 3, "les trois gouttes prises");
}

/// Charge le niveau `indice` de la table dans une partie en cours (comme si on y
/// était arrivé), avec la partie `etat`.
fn aller_au_niveau(app: &mut Application, indice: usize, etat: EtatJeu) {
    app.recharger(indice, Some(etat));
    app.ecran = Ecran::Jeu;
}

fn indice_de(nom: &str) -> usize {
    niveaux::niveaux()
        .iter()
        .position(|n| n.nom == nom)
        .unwrap()
}

/// Carte 7.9 : REJOUER un niveau ne recrédite rien. La descente finie avec ses
/// 3 gouttes rapporte la première fois ; rejouée de même, +0 (« 3 GOUTTES DÉJÀ
/// COMPTÉES ») et le porte-monnaie ne bouge plus. Le meilleur temps reste noté.
#[test]
fn rejouer_un_niveau_ne_recredite_pas() {
    let mut app = Application::new(true);
    app.lancer(Suite::NiveauSeul(indice_de("La descente")));
    let mut poche = Vec::new();
    for _ in 0..2 {
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        passer_la_carte(&mut app);
        ramasser_les_gouttes(&mut app);
        app.monde.termine = true;
        avancer(&mut app, 1);
        assert_eq!(app.ecran, Ecran::BilanNiveau);
        let r = app.resultat_niveau.clone().unwrap();
        poche.push((r.gain, r.deja_comptees, app.sauvegarde.gouttes_en_poche));
        app.valider_choix(Choix::RejouerNiveau);
    }
    let (gain, deja, en_poche) = poche[0];
    assert!(gain >= 3 && deja == 0 && en_poche == gain, "{poche:?}");
    assert_eq!(poche[1], (0, 3, en_poche), "rejouée : +0");
    assert!(app.sauvegarde.progres("descente").meilleur_temps.is_some());
}

#[test]
fn une_partie_commence_avec_cinq_vies_et_chaque_mort_en_coute_une() {
    let mut app = en_jeu();
    assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART);
    tomber(&mut app);
    assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART - 1);
    assert_eq!(app.monde.etat.morts, 1);
    assert_eq!(app.ecran, Ecran::Jeu, "Pépin réapparaît, on continue");
    // Le HUD les affiche, sans éclat (une vie perdue n'est pas une fête).
    let v = hud::vies(app.monde.etat.vies.restantes(), app.eclat_vie);
    assert_eq!((v.chiffre.as_str(), v.plus_un), ("4", None));
}

/// La dernière vie perdue : l'écran « PLUS DE VIES », puis le niveau EN COURS
/// recommence depuis son début, avec cinq vies, sans les gouttes de ce niveau
/// (celles d'avant gardées), le chrono qui continue et toutes les morts comptées.
#[test]
fn plus_de_vies_ouvre_l_ecran_puis_recommence_le_niveau_en_cours() {
    let mut app = en_jeu();
    // Niveau 1 fini avec deux gouttes : on arrive au niveau 2.
    app.monde.etat.gouttes = 2;
    app.monde.termine = true;
    avancer(&mut app, 1);
    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
    app.reagir(&valider(), &Commandes::default(), None);
    assert_eq!(app.monde.infos.numero, 2);
    avancer(&mut app, 30);
    ramasser_les_gouttes(&mut app);
    assert_eq!(app.monde.etat.gouttes, 5);
    assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART + 1, "+1 vie");
    for _ in 0..VIES_DEPART {
        tomber(&mut app);
        assert_eq!(app.ecran, Ecran::Jeu);
    }
    tomber(&mut app);
    assert_eq!(app.ecran, Ecran::PlusDeVies(plus_de_vies::DUREE_PAS));
    let chrono = app.pas_partie;
    let morts = app.monde.etat.morts;
    assert_eq!(morts, u32::from(VIES_DEPART) + 1);
    // L'écran attend (le monde est figé), puis le joueur appuie (carte 9.31).
    let pas_monde = app.monde.pas;
    avancer(&mut app, 60);
    assert!(matches!(app.ecran, Ecran::PlusDeVies(_)));
    assert_eq!(app.monde.pas, pas_monde, "le monde ne bouge pas");
    jusqu_a(&mut app, |e| e == Ecran::Jeu);
    assert_eq!(app.monde.infos.numero, 2, "le même niveau");
    assert_eq!(app.monde.pas, 0, "tout neuf");
    assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART);
    assert_eq!(app.monde.etat.gouttes, 2, "celles du niveau 1 restent");
    assert_eq!(app.monde.etat.gouttes_ici(), 0);
    assert_eq!(
        app.gouttes_du_niveau(),
        0,
        "les gouttes du niveau sont revenues"
    );
    assert_eq!(app.monde.etat.morts, morts, "l'historique des morts");
    assert_eq!(app.pas_partie, chrono, "le chrono reprend où il était");
    assert_eq!(
        (app.monde.joueur.boite.x, app.monde.joueur.boite.y),
        app.monde.niveau.depart,
        "au départ du niveau"
    );
    // Les trois gouttes reprises ne redonnent pas de vie (déjà donnée ici).
    avancer(&mut app, 30);
    ramasser_les_gouttes(&mut app);
    assert_eq!(app.monde.etat.gouttes, 5);
    assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART);
    // Le jeu continue : chrono et monde avancent.
    avancer(&mut app, 1);
    assert_eq!(app.ecran, Ecran::Jeu);
}

#[test]
fn on_passe_l_ecran_plus_de_vies_d_un_appui() {
    let mut app = en_jeu();
    for _ in 0..VIES_DEPART {
        tomber(&mut app);
    }
    assert!(matches!(app.ecran, Ecran::PlusDeVies(_)));
    // L'appui est accepté après le court délai d'ouverture (carte 9.31).
    avancer(&mut app, histoire::DELAI_PAS);
    app.reagir(&valider(), &Commandes::default(), None);
    assert!(
        app.transition.est_active(),
        "les volets partent tout de suite"
    );
    jusqu_a(&mut app, |e| e == Ecran::Jeu);
    assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART);
    assert_eq!(app.monde.pas, 0);
    // Un toucher (clic) aussi.
    for _ in 0..VIES_DEPART {
        tomber(&mut app);
    }
    let toucher = Navigation {
        clic: Some((160.0, 90.0)),
        ..Navigation::default()
    };
    avancer(&mut app, histoire::DELAI_PAS);
    app.reagir(&toucher, &Commandes::default(), None);
    jusqu_a(&mut app, |e| e == Ecran::Jeu);
    assert_eq!(app.monde.etat.morts, 2 * u32::from(VIES_DEPART));
}

/// Focus perdu (ou pause demandée) pendant l'écran « PLUS DE VIES » : l'écran
/// finit, le niveau recommence, et la pause s'ouvre AVANT son premier pas.
#[test]
fn focus_perdu_ou_pause_pendant_plus_de_vies_la_pause_attend_le_niveau() {
    for par_la_pause in [false, true] {
        let mut app = en_jeu();
        for _ in 0..VIES_DEPART {
            tomber(&mut app);
        }
        assert!(matches!(app.ecran, Ecran::PlusDeVies(_)));
        if par_la_pause {
            app.reagir(&rien(), &pause(), None);
        } else {
            app.focus(false);
        }
        assert!(
            matches!(app.ecran, Ecran::PlusDeVies(_)),
            "pas de pause ici"
        );
        let chrono = app.pas_partie;
        jusqu_a(&mut app, |e| e == Ecran::Pause);
        assert_eq!(app.monde.pas, 0, "le niveau recommencé attend le joueur");
        assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART);
        assert_eq!(app.pas_partie, chrono);
        assert!(!app.boucles_sonores());
        // Reprise : le jeu repart normalement.
        app.reagir(&rien(), &pause(), None);
        avancer(&mut app, 3);
        assert_eq!(app.ecran, Ecran::Jeu);
        assert_eq!(app.monde.pas, 3);
    }
    // Focus perdu PUIS revenu pendant l'écran : aucune pause.
    let mut app = en_jeu();
    for _ in 0..VIES_DEPART {
        tomber(&mut app);
    }
    app.focus(false);
    app.focus(true);
    jusqu_a(&mut app, |e| e == Ecran::Jeu);
    avancer(&mut app, 3);
    assert_eq!((app.ecran, app.monde.pas), (Ecran::Jeu, 3));
}

/// Dans l'arène du Grand Nuage, un « PLUS DE VIES » le remet à ses trois points
/// de vie (une simple mort, elle, les lui laisse perdus).
#[test]
fn plus_de_vies_rend_au_grand_nuage_ses_trois_points_de_vie() {
    let mut app = en_jeu();
    let etat = app.monde.etat.clone();
    aller_au_niveau(&mut app, indice_de(niveaux::par_cle("cyclone").nom), etat);
    assert_eq!(app.monde.pv_nuage(), Some(3));
    for e in &mut app.monde.entites {
        if let Entite::Nuage(g) = e {
            g.essouffler();
            g.toucher();
        }
    }
    assert_eq!(app.monde.pv_nuage(), Some(2));
    tomber(&mut app);
    assert_eq!(
        app.monde.pv_nuage(),
        Some(2),
        "une mort : il garde ses points perdus"
    );
    for _ in 1..VIES_DEPART {
        tomber(&mut app);
    }
    assert!(matches!(app.ecran, Ecran::PlusDeVies(_)));
    jusqu_a(&mut app, |e| e == Ecran::Jeu);
    assert_eq!(app.monde.pv_nuage(), Some(3), "le niveau recommence");
    assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART);
}

/// Le niveau bonus ne coûte aucune vie ; l'épilogue, si.
#[test]
fn le_niveau_bonus_ne_coute_aucune_vie() {
    let mut app = en_jeu();
    let etat = app.monde.etat.clone();
    let bonus = niveaux::niveaux()
        .iter()
        .position(|n| n.genre == GenreNiveau::Bonus)
        .unwrap();
    aller_au_niveau(&mut app, bonus, etat);
    for _ in 0..3 * VIES_DEPART {
        tomber(&mut app);
    }
    assert_eq!(app.ecran, Ecran::Jeu);
    assert_eq!(app.monde.etat.morts, 3 * u32::from(VIES_DEPART));
    assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART);
    // L'épilogue (le niveau d'après) en coûte de nouveau.
    let etat = app.monde.etat.clone();
    aller_au_niveau(&mut app, bonus + 1, etat);
    assert_eq!(app.monde.infos.genre, GenreNiveau::Epilogue);
    tomber(&mut app);
    assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART - 1);
}

/// La troisième goutte d'un niveau : une vie de plus, et le petit signe du HUD
/// (chiffre en or, « +1 ») pendant une seconde. Plafond : 9.
#[test]
fn la_troisieme_goutte_donne_une_vie_et_le_hud_le_montre() {
    let mut app = en_jeu();
    avancer(&mut app, 30);
    ramasser_les_gouttes(&mut app);
    assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART + 1);
    assert_eq!(app.eclat_vie, hud::DUREE_ECLAT_VIE);
    let v = hud::vies(app.monde.etat.vies.restantes(), app.eclat_vie);
    assert_eq!(v.chiffre, "6");
    assert!(v.plus_un.is_some());
    avancer(&mut app, u32::from(hud::DUREE_ECLAT_VIE));
    assert_eq!(app.eclat_vie, 0);
    assert!(hud::vies(6, app.eclat_vie).plus_un.is_none());
    // Recommencer (pause) : les gouttes reviennent, mais pas la vie (déjà donnée),
    // et la vie gagnée reste.
    app.reagir(&rien(), &pause(), None);
    app.valider_choix(Choix::Recommencer);
    jusqu_a(&mut app, |e| e == Ecran::Jeu);
    assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART + 1);
    avancer(&mut app, 30);
    ramasser_les_gouttes(&mut app);
    assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART + 1);
    assert_eq!(app.eclat_vie, 0, "pas de signe sans vie gagnée");
    // Plafond : avec 9 vies, la vie du niveau suivant ne compte pas.
    let mut etat = app.monde.etat.clone();
    while etat.vies.restantes() < VIES_MAX {
        etat.nouveau_niveau((0.0, 0.0), GenreNiveau::Normal);
        let mut j = app.monde.joueur.clone();
        let trois = [Contact::Goutte(0), Contact::Goutte(1), Contact::Goutte(2)];
        etat.traiter(&mut j, &trois);
    }
    aller_au_niveau(&mut app, 1, etat);
    avancer(&mut app, 30);
    ramasser_les_gouttes(&mut app);
    assert_eq!(app.monde.etat.vies.restantes(), VIES_MAX);
    assert_eq!(app.eclat_vie, 0);
}

/// « RECOMMENCER » (pause) ne rend pas les vies perdues dans le niveau.
#[test]
fn recommencer_ne_rend_pas_les_vies_perdues() {
    let mut app = en_jeu();
    tomber(&mut app);
    tomber(&mut app);
    app.reagir(&rien(), &pause(), None);
    app.valider_choix(Choix::Recommencer);
    jusqu_a(&mut app, |e| e == Ecran::Jeu);
    assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART - 2);
    assert_eq!(app.monde.etat.morts, 2);
}

/// L'écran de fin reçoit les vies restantes.
#[test]
fn le_bilan_garde_les_vies_restantes() {
    let mut app = en_jeu();
    let etat = app.monde.etat.clone();
    aller_au_niveau(&mut app, niveaux::NB_NIVEAUX_MONDE_1 - 1, etat);
    tomber(&mut app);
    app.monde.termine = true;
    avancer(&mut app, 1);
    jusqu_a(&mut app, |e| e == Ecran::Fin);
    assert_eq!(app.bilan.vies, VIES_DEPART - 1);
    assert_eq!(app.bilan.morts, 1);
}

// --- Histoire (carte 8.4) -----------------------------------------------------

/// Avance jusqu'à la prochaine image de l'histoire ou carte de niveau (sans passer
/// l'image) ; renvoie le chapitre montré (`None` : directement la carte).
fn image_suivante(app: &mut Application) -> Option<Chapitre> {
    jusqu_a(app, |e| matches!(e, Ecran::Histoire(_) | Ecran::Carte(_)));
    match app.ecran {
        Ecran::Histoire(c) => Some(c),
        _ => None,
    }
}

/// Termine le niveau en cours (gouttes de la partie inchangées).
fn finir_le_niveau(app: &mut Application) {
    if matches!(app.ecran, Ecran::Carte(_)) {
        passer_la_carte(app);
    }
    assert_eq!(app.ecran, Ecran::Jeu);
    app.monde.termine = true;
    avancer(app, 1);
}

/// JOUER : la première image avant la carte du niveau 1. Elle ignore les appuis un
/// instant (un saut tenu ne la ferait pas disparaître sans être vue), puis un appui
/// mène à la carte du niveau 1, déjà chargé ; le chrono n'a pas bougé.
#[test]
fn jouer_montre_la_premiere_image_avant_le_niveau_1() {
    let mut app = Application::new(true);
    app.reagir(&valider(), &Commandes::default(), None);
    assert_eq!(image_suivante(&mut app), Some(Chapitre::Depart));
    assert_eq!(app.monde.indice, 0);
    app.reagir(&valider(), &Commandes::default(), None);
    assert_eq!(app.ecran, Ecran::Histoire(Chapitre::Depart), "trop tôt");
    avancer(&mut app, histoire::DELAI_PAS);
    // Rien ne la passe toute seule.
    avancer(&mut app, 600);
    assert_eq!(app.ecran, Ecran::Histoire(Chapitre::Depart));
    // La fenêtre perd le focus : aucune pause (rien ne bouge derrière l'image).
    app.focus(false);
    assert_eq!(app.ecran, Ecran::Histoire(Chapitre::Depart));
    assert!(!app.pause_differee);
    app.focus(true);
    app.reagir(&valider(), &Commandes::default(), None);
    assert_eq!(image_suivante(&mut app), None);
    assert_eq!(app.ecran, Ecran::Carte(carte::DUREE_PAS));
    assert_eq!(app.monde.indice, 0);
    assert_eq!(app.chrono(), 0, "l'histoire ne compte pas dans le chrono");
}

/// Une partie de JOUER à la fin : les trois images, chacune UNE fois, aux bons
/// niveaux (avant le 1er, après le Grand Nuage, avant la descente), bonus joué ou
/// non. Mourir sans vies ou « recommencer » ne les remontre pas. Score et gouttes
/// ne bougent pas en les passant.
#[test]
fn une_partie_montre_les_trois_images_une_fois_chacune() {
    // Toutes les gouttes du MONDE 1 (carte 9.6 : pas celles de toute la table).
    let toutes = levels::gouttes_pour_le_bonus(niveaux::niveaux_monde_1());
    for gouttes in [0, toutes] {
        let mut app = Application::new(true);
        app.reagir(&valider(), &Commandes::default(), None);
        let mut vues = Vec::new();
        loop {
            if let Some(c) = image_suivante(&mut app) {
                vues.push((c, app.monde.indice));
                let e = &app.monde.etat;
                let avant = (e.score, e.gouttes, e.morts, e.vies.restantes());
                avancer(&mut app, histoire::DELAI_PAS);
                app.reagir(&valider(), &Commandes::default(), None);
                jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
                let e = &app.monde.etat;
                let apres = (e.score, e.gouttes, e.morts, e.vies.restantes());
                assert_eq!(apres, avant, "rien ne change en passant l'image");
            }
            if app.monde.indice == 1 {
                // « Recommencer » ne remontre pas d'image.
                passer_la_carte(&mut app);
                app.valider_choix(Choix::Recommencer);
                jusqu_a(&mut app, |e| e == Ecran::Jeu);
            }
            app.monde.etat.gouttes = gouttes;
            finir_le_niveau(&mut app);
            jusqu_a(&mut app, |e| {
                matches!(e, Ecran::Histoire(_) | Ecran::Carte(_) | Ecran::Fin)
            });
            if app.ecran == Ecran::Fin {
                break;
            }
        }
        let indice_de_la_suite_du_boss = indice_de(niveaux::par_cle("cyclone").nom) + 1;
        assert_eq!(
            vues,
            vec![
                (Chapitre::Depart, 0),
                (Chapitre::Lueur, indice_de_la_suite_du_boss),
                (Chapitre::Soleil, niveaux::NB_NIVEAUX_MONDE_1 - 1),
            ],
            "gouttes : {gouttes}"
        );
    }
}

/// REJOUER (l'histoire vient d'être vue) et les raccourcis de niveau du build de
/// QA : aucune image.
#[test]
fn rejouer_et_les_raccourcis_ne_racontent_pas_l_histoire() {
    let mut app = en_jeu();
    app.ecran = Ecran::Fin;
    app.choix = choix_pour(Ecran::Fin);
    app.menu = Menu::new(2);
    app.reagir(&valider(), &Commandes::default(), None);
    assert_eq!(image_suivante(&mut app), None, "REJOUER");
    let mut app = en_jeu();
    let descente = niveaux::NB_NIVEAUX_MONDE_1 - 1;
    app.reagir(
        &rien(),
        &Commandes::default(),
        Some(RaccourciNiveau::Aller(descente)),
    );
    assert_eq!(image_suivante(&mut app), None, "raccourci");
    assert_eq!(app.monde.indice, descente);
}

/// HISTOIRE au titre : les trois images dans l'ordre (sans rien charger), puis le
/// titre, curseur sur HISTOIRE ; Échap y revient à tout moment.
#[test]
fn la_ligne_histoire_du_titre_montre_les_trois_images() {
    let mut app = Application::new(true);
    // Pas de ligne HISTOIRE avant une partie finie (carte 8.11) : 4 lignes.
    assert!(!app.choix.contains(&Choix::Histoire));
    assert_eq!(app.choix.len(), 4);
    app.sauvegarde.parties_finies = 1;
    app.ouvrir(Ecran::Titre);
    // NOUVELLE PARTIE + désactivée pour l'instant (AGA) : absente même jeu fini.
    assert_eq!(
        app.choix.contains(&Choix::NouvellePartiePlus),
        NOUVELLE_PARTIE_PLUS_OUVERTE
    );
    let ligne = app
        .choix
        .iter()
        .position(|&c| c == Choix::Histoire)
        .unwrap();
    app.menu.choisir(ligne);
    app.reagir(&valider(), &Commandes::default(), None);
    for attendu in Chapitre::TOUS {
        jusqu_a(&mut app, |e| e == Ecran::Histoire(attendu));
        avancer(&mut app, histoire::DELAI_PAS);
        app.reagir(&valider(), &Commandes::default(), None);
    }
    assert_eq!(app.ecran, Ecran::Titre);
    assert_eq!(app.choix[app.menu.selection()], Choix::Histoire);
    assert_eq!(app.chrono(), 0);
    // Échap, même tout de suite : retour au titre.
    app.reagir(&valider(), &Commandes::default(), None);
    assert_eq!(app.ecran, Ecran::Histoire(Chapitre::Depart));
    let retour = Navigation {
        retour: true,
        ..Navigation::default()
    };
    app.reagir(&retour, &Commandes::default(), None);
    assert_eq!(app.ecran, Ecran::Titre);
    assert_eq!(app.choix[app.menu.selection()], Choix::Histoire);
    // Puis JOUER raconte de nouveau l'histoire, et passe aux niveaux, quand il
    // repart du tout premier niveau (AGA : JOUER reprend où l'on en est ; une
    // partie finie ouvre tout le Monde 1, on y reprendrait donc au bout).
    app.sauvegarde = Sauvegarde::depuis_texte("version=3");
    app.menu.choisir(0);
    app.reagir(&valider(), &Commandes::default(), None);
    assert_eq!(image_suivante(&mut app), Some(Chapitre::Depart));
    avancer(&mut app, histoire::DELAI_PAS);
    app.reagir(&valider(), &Commandes::default(), None);
    assert_eq!(image_suivante(&mut app), None);
}
