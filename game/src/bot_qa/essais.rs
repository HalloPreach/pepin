//! Les parties des NIVEAUX D'ESSAI (un par mécanique nouvelle : vent, grêle,
//! auvent, ventilateur, montgolfière, escargot, chat, horloge, Grand Nuage)
//! et leurs parcours de référence.
//!
//! Les niveaux d'essai n'existent QUE dans les tests et le build de QA
//! (fonctionnalité `debug_niveaux` ; `INDICE_ESSAI_*` dans `monde.rs`), jamais
//! dans le binaire de production. Chaque niveau a une touche raccourci dans
//! le build de QA (0, F1, F2, …, `RaccourciNiveau` dans `entrees.rs`) : le
//! robot l'appuie pour sauter directement au niveau, puis rejoue un parcours
//! par-dessus (le raccourci reste disponible : il n'est PAS simulé).
//!
//! ## Ce qui va où
//!
//! - `jouer_essai` : la fonction GÉNÉRIQUE — le niveau (`indice`) et sa
//!   touche raccourci sont des PARAMÈTRES ; les dix niveaux d'essai
//!   s'écrivent donc en une ligne chacun (`jouer_essai_*`, à ne pas
//!   multiplier en copies : un jour un niveau de plus, une ligne de plus).
//! - `ImageEssai` / `PartieEssai` : ce qu'on observe image par image (et ce
//!   que le trace `*.csv` des scripts de `scripts.rs` note).
//! - `parcours_essai_*` : le parcours de référence de chaque niveau, cherché
//!   par `trouver_parcours` (`parcours.rs`) : d'abord GROS (saut long, plané
//!   long), puis PRÉCIS si le gros ne finit pas. `trouver_parcours` cherche
//!   le PLUS COURT chemin, pas le plus joli : deux parcours justes ne sont
//!   pas forcément identiques ; seuls comptent l'arrivée, les morts et les
//!   gouttes.
//!
//! ## Ajouter un niveau d'essai (voir aussi la doc de `mod.rs`)
//!
//! 1. La grille ASCII dans `core/src/levels/` (lettres réservées de l'étape)
//!    et `INDICE_ESSAI_*` + touche raccourci (`monde.rs`, `entrees.rs`).
//! 2. Ici : `jouer_essai_X` (une ligne : `indice`, `raccourci`) et
//!    `parcours_essai_X` (gros puis précis).
//! 3. `essais_tests.rs` : le test « le robot finit sans mourir » (+ tenue à
//!    une image près si la mécanique a une horloge) ; `scripts.rs` : le test
//!    `ecrire_script_essai_X` ignore (script/touches/trace dans
//!    `target/qa/`, pour les captures de QA.md).
//!
//! Notions Rust illustrées ici : une fonction générique PARAMÉTRÉE (niveau,
//! raccourci) au lieu d'une copie par niveau, des `Option` et `matches!`,
//! et des const `INDICE_ESSAI_*` qui n'existent que sous `#[cfg(test)]` /
//! la fonctionnalité `debug_niveaux` (le niveau d'essai n'entre pas dans le
//! binaire de production).

use pepin_core::entities::contacts::Contact;
use pepin_core::entities::ennemi::PhaseChat;
use pepin_core::entities::montgolfiere::EtatMontgolfiere;
use pepin_core::entities::nuage::PhaseNuage;
use pepin_core::entities::ventilateur::PhaseVentilateur;
use pepin_core::entities::{Entite, TypeEnnemi};
use pepin_core::player::{Direction, Entrees};

use crate::app::Application;
use crate::ecrans::Ecran;
use crate::entrees::{Commandes, RaccourciNiveau};
use crate::monde::{
    Monde, INDICE_ESSAI, INDICE_ESSAI_AUVENT, INDICE_ESSAI_CHAT, INDICE_ESSAI_ESCARGOT,
    INDICE_ESSAI_GRELE, INDICE_ESSAI_HORLOGE, INDICE_ESSAI_MONTGOLFIERE, INDICE_ESSAI_NUAGE,
    INDICE_ESSAI_VENT, INDICE_ESSAI_VENTILATEUR,
};
use crate::navigation::Navigation;

use super::nom_ecran;
use super::Trame;
use crate::bot_qa::parcours::{
    aiguille_sous_pepin, chercher, depart_pose, est_lance, plans, trouver_parcours, Etape,
};

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
    /// Premier chat du niveau (carte 5.9 ; `None` s'il n'y en a pas) : sa phase,
    /// et `true` s'il est écrasé (ou déjà disparu).
    pub chat: Option<(PhaseChat, bool)>,
    /// Pépin vient d'écraser un chat pendant cette image.
    pub ecrase_chat: bool,
    /// Coin haut-gauche des deux premières aiguilles d'horloge du niveau, dans
    /// l'ordre des entités (carte 5.10 ; `(-1, -1)` s'il n'y en a pas).
    pub aiguilles: [(f32, f32); 2],
    /// L'aiguille qui porte Pépin à la fin de cette image (`aiguille_sous_pepin`).
    pub sur_aiguille: Option<usize>,
    /// Le Grand Nuage (carte 5.11 ; `None` s'il n'y en a pas) : sa phase, le temps
    /// passé dans cette phase et ses points de vie.
    pub nuage: Option<(PhaseNuage, u16, u8)>,
    /// Pépin vient de sauter sur la tête du Grand Nuage pendant cette image.
    pub coup_nuage: bool,
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
        let chat = app.monde.entites.iter().find_map(|e| match e {
            Entite::Ennemi(c) if c.type_ennemi == TypeEnnemi::Chat => {
                Some((c.phase_chat(), !c.est_dangereux()))
            }
            _ => None,
        });
        // Un écrasement cette image, et le chat écrasé maintenant, vivant juste avant.
        let ecrase_chat = app.monde.contacts.contains(&Contact::Ecrase)
            && chat.is_some_and(|c| c.1)
            && images
                .last()
                .is_some_and(|i: &ImageEssai| i.chat.is_some_and(|c| !c.1));
        let mut aiguilles = [(-1.0f32, -1.0f32); 2];
        let liste = app.monde.entites.iter().filter_map(|e| match e {
            Entite::Mobile(a) if a.cercle().is_some() => Some(a.boite),
            _ => None,
        });
        for (place, b) in aiguilles.iter_mut().zip(liste) {
            *place = (b.x, b.y);
        }
        let sur_aiguille = aiguille_sous_pepin(&app.monde);
        let nuage = app.monde.entites.iter().find_map(|e| match e {
            Entite::Nuage(g) => Some((g.phase(), g.ecoule(), g.pv())),
            _ => None,
        });
        let coup_nuage = app.monde.contacts.contains(&Contact::Nuage);
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
            chat,
            ecrase_chat,
            aiguilles,
            sur_aiguille,
            nuage,
            coup_nuage,
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
    jouer_essai(INDICE_ESSAI, RaccourciNiveau::Essai, parcours)
}

/// Joue le niveau d'essai « vent » (touche F1 du build de QA) — voir `jouer_essai`.
pub fn jouer_essai_vent(parcours: &[Entrees]) -> PartieEssai {
    jouer_essai(INDICE_ESSAI_VENT, RaccourciNiveau::Essai2, parcours)
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
    jouer_essai(INDICE_ESSAI_GRELE, RaccourciNiveau::Essai3, parcours)
}

/// Joue le niveau d'essai « auvent » (touche F3 du build de QA) — voir `jouer_essai`.
pub fn jouer_essai_auvent(parcours: &[Entrees]) -> PartieEssai {
    jouer_essai(INDICE_ESSAI_AUVENT, RaccourciNiveau::Essai4, parcours)
}

/// Joue le niveau d'essai « ventilateur » (touche F4 du build de QA) — voir
/// `jouer_essai`.
pub fn jouer_essai_ventilateur(parcours: &[Entrees]) -> PartieEssai {
    jouer_essai(INDICE_ESSAI_VENTILATEUR, RaccourciNiveau::Essai5, parcours)
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
    jouer_essai(INDICE_ESSAI_MONTGOLFIERE, RaccourciNiveau::Essai6, parcours)
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
    jouer_essai(INDICE_ESSAI_ESCARGOT, RaccourciNiveau::Essai7, parcours)
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

/// Joue le niveau d'essai « chat » (touche F7 du build de QA) — voir `jouer_essai`.
pub fn jouer_essai_chat(parcours: &[Entrees]) -> PartieEssai {
    jouer_essai(INDICE_ESSAI_CHAT, RaccourciNiveau::Essai8, parcours)
}

/// Parcours du robot dans le niveau d'essai « chat » (maladroit, puis précis).
/// Sur la VRAIE simulation, un plan qui heurte le chat de côté, ou qui lui tombe
/// dessus pendant qu'il est réveillé, tue Pépin : il est écarté. Le robot apprend
/// donc à attendre que le chat dorme (plans « attendre » : le moment du cycle est
/// dans la case, voir `chats_de_la_case`), puis à se laisser tomber sur lui.
pub fn parcours_essai_chat() -> Vec<Entrees> {
    let i = crate::monde::INDICE_ESSAI_CHAT;
    trouver_parcours(i, false)
        .or_else(|| trouver_parcours(i, true))
        .expect("le robot ne trouve aucun parcours au niveau d'essai chat")
}

/// Joue l'arène du Grand Nuage (touche F10 du build de QA) — voir `jouer_essai`.
pub fn jouer_essai_nuage(parcours: &[Entrees]) -> PartieEssai {
    jouer_essai(INDICE_ESSAI_NUAGE, RaccourciNiveau::Essai10, parcours)
}

/// Parcours du robot dans l'arène du Grand Nuage (maladroit, puis précis). Sur la
/// VRAIE simulation, le nuage lance ses éclairs : le robot apprend à les esquiver
/// (attendre à l'abri, filer entre deux salves : le temps du nuage est dans la case,
/// voir `nuage_de_la_case`), puis, quand il s'essouffle, à monter dans son courant
/// et à lui retomber sur la tête, trois fois. Chaque plan doit tenir avec le nuage
/// décalé d'une ou deux images (`tient_au_decalage`).
pub fn parcours_essai_nuage() -> Vec<Entrees> {
    let i = crate::monde::INDICE_ESSAI_NUAGE;
    trouver_parcours(i, false)
        .or_else(|| trouver_parcours(i, true))
        .expect("le robot ne trouve aucun parcours dans l'arène du Grand Nuage")
}

/// Joue le niveau d'essai « horloge » (touche F8 du build de QA) — voir `jouer_essai`.
pub fn jouer_essai_horloge(parcours: &[Entrees]) -> PartieEssai {
    jouer_essai(INDICE_ESSAI_HORLOGE, RaccourciNiveau::Essai9, parcours)
}

/// Parcours du robot dans le niveau d'essai « horloge » (maladroit, puis précis).
/// Sur la VRAIE simulation, l'aiguille tourne : le robot apprend à l'attendre
/// (son heure est dans la case, voir `aiguille_proche`), à sauter dessus, à se
/// laisser porter (plans « attendre » posé sur elle), puis à sauter plus haut.
pub fn parcours_essai_horloge() -> Vec<Entrees> {
    let i = crate::monde::INDICE_ESSAI_HORLOGE;
    trouver_parcours(i, false)
        .or_else(|| trouver_parcours(i, true))
        .expect("le robot ne trouve aucun parcours au niveau d'essai horloge")
}
