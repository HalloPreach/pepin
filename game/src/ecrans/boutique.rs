//! Écran de la boutique (carte 7.5) : porte-monnaie, onglets, une ligne par objet
//! (aperçu animé, nom, prix, état), RETOUR, et la petite fenêtre de confirmation
//! ou de refus. La logique (que fait un appui ?) est dans `crate::boutique` ; ici
//! on ne fait que dessiner et dire où sont les zones cliquables (`MiseEnPage`).
//!
//! Mise en page 320 x 180 (pixels logiques), lisible sur un téléphone en paysage :
//! textes à l'échelle 2 (noms, boutons), lignes de 30 px, aucune zone ne se chevauche
//! (vérifié à la compilation, `const _` plus bas).

use pepin_core::font::{largeur_texte, HAUTEUR_GLYPHE};
use pepin_core::palette::{
    ARDOISE, ARDOISE_CLAIRE, ARDOISE_NUIT, ARDOISE_OMBRE, BLANC_CASSE, BRUME, OR, OR_PALE,
};
use pepin_core::LARGEUR_LOGIQUE;

use super::{image, panneau, trame};
use crate::boutique::{Apercu, Dialogue, Etat, EtatBoutique, Zones};
use crate::contenu;
use crate::sauvegarde::Sauvegarde;
use crate::sprites::Banque;
use crate::texte::{self, Style};

/// Colonne des lignes (comme les menus).
const X: f32 = 40.0;
const L: f32 = 240.0;
/// Onglets : une rangée de boîtes.
const Y_ONGLETS: f32 = 26.0;
const H_ONGLET: f32 = 13.0;
/// Lignes d'objets : la première en `Y_LIGNES`, une tous les `PAS_LIGNE`, au plus
/// `MAX_VISIBLES` à la fois (la liste défile avec la sélection).
const Y_LIGNES: f32 = 42.0;
const PAS_LIGNE: f32 = 30.0;
const MAX_VISIBLES: usize = 3;
/// Ligne RETOUR, puis rappel des touches.
const Y_RETOUR: f32 = 138.0;
const H_RETOUR: f32 = 20.0;
const Y_AIDE: f32 = 165.0;
/// Fenêtre de dialogue et ses boutons.
const DIALOGUE: (f32, f32, f32, f32) = (30.0, 52.0, 260.0, 80.0);
const BOUTON_OUI: (f32, f32, f32, f32) = (70.0, 102.0, 80.0, 20.0);
const BOUTON_NON: (f32, f32, f32, f32) = (170.0, 102.0, 80.0, 20.0);
const BOUTON_OK: (f32, f32, f32, f32) = (120.0, 102.0, 80.0, 20.0);

const _: () = {
    assert!(Y_LIGNES >= Y_ONGLETS + H_ONGLET + 2.0);
    assert!(Y_RETOUR >= Y_LIGNES + MAX_VISIBLES as f32 * PAS_LIGNE + 2.0);
    assert!(Y_AIDE >= Y_RETOUR + H_RETOUR + 2.0);
    assert!(Y_AIDE + HAUTEUR_GLYPHE as f32 + 1.0 <= 176.0);
    assert!(BOUTON_OUI.1 + BOUTON_OUI.3 <= DIALOGUE.1 + DIALOGUE.3 - 4.0);
};

fn dedans((rx, ry, rl, rh): (f32, f32, f32, f32), x: f32, y: f32) -> bool {
    x >= rx && x < rx + rl && y >= ry && y < ry + rh
}

/// Première ligne d'objet visible : la liste défile pour garder la sélection à l'écran.
fn premiere_visible(selection: usize, nb: usize) -> usize {
    if nb <= MAX_VISIBLES {
        0
    } else {
        (selection.min(nb - 1) + 1)
            .saturating_sub(MAX_VISIBLES)
            .min(nb - MAX_VISIBLES)
    }
}

/// Rectangle de l'onglet `i`.
fn rect_onglet(noms: &[&str], i: usize) -> (f32, f32, f32, f32) {
    let largeur = |n: &str| largeur_texte(n, 1) as f32 + 12.0;
    let x = X + noms[..i].iter().map(|n| largeur(n) + 4.0).sum::<f32>();
    (x, Y_ONGLETS, largeur(noms[i]), H_ONGLET)
}

/// Les zones cliquables de l'écran.
pub struct MiseEnPage;

impl Zones for MiseEnPage {
    fn ligne_sous(&self, e: &EtatBoutique, x: f32, y: f32) -> Option<usize> {
        let nb = e.objets.len();
        if dedans((X, Y_RETOUR, L, H_RETOUR), x, y) {
            return Some(nb);
        }
        let debut = premiere_visible(e.menu.selection(), nb);
        (debut..nb.min(debut + MAX_VISIBLES)).find(|&i| dedans(rect_ligne(i - debut), x, y))
    }

    fn onglet_sous(&self, nb: usize, x: f32, y: f32) -> Option<usize> {
        let noms: Vec<&str> = crate::boutique::onglets().iter().map(|c| c.nom()).collect();
        (0..nb.min(noms.len())).find(|&i| dedans(rect_onglet(&noms, i), x, y))
    }

    fn bouton_sous(&self, d: Dialogue, x: f32, y: f32) -> Option<bool> {
        match d {
            Dialogue::Confirmer { .. } if dedans(BOUTON_OUI, x, y) => Some(true),
            Dialogue::Confirmer { .. } if dedans(BOUTON_NON, x, y) => Some(false),
            Dialogue::Manque { .. } if dedans(BOUTON_OK, x, y) => Some(true),
            _ => None,
        }
    }
}

/// Rectangle de la `k`-ième ligne VISIBLE.
fn rect_ligne(k: usize) -> (f32, f32, f32, f32) {
    (X, Y_LIGNES + k as f32 * PAS_LIGNE, L, PAS_LIGNE)
}

/// Cadre d'une ligne ou d'un bouton : or si choisi.
fn cadre((x, y, w, h): (f32, f32, f32, f32), choisi: bool) {
    if choisi {
        panneau(x, y, w, h - 2.0, ARDOISE_OMBRE, OR);
    } else {
        panneau(x, y, w, h - 2.0, ARDOISE_NUIT, ARDOISE);
    }
}

/// Bouton à texte centré (échelle 2).
fn bouton(r: (f32, f32, f32, f32), libelle: &str, choisi: bool) {
    cadre(r, choisi);
    let couleur = if choisi { OR_PALE } else { BRUME };
    texte::dessiner_centre_sur(
        libelle,
        r.0 + r.2 / 2.0,
        r.1 + 2.0,
        Style::simple(couleur, 2),
    );
}

pub fn dessiner(banque: &Banque, anim: u32, s: &Sauvegarde, e: &EtatBoutique) {
    panneau(
        4.0,
        4.0,
        LARGEUR_LOGIQUE as f32 - 8.0,
        172.0,
        ARDOISE_NUIT,
        ARDOISE_CLAIRE,
    );
    let ombre = |c, k| Style::simple(c, k).avec_ombre(ARDOISE_OMBRE);
    texte::dessiner_centre(contenu::BOUTIQUE, 8.0, ombre(OR_PALE, 2));
    // Porte-monnaie, toujours visible : la goutte d'or et le nombre en poche.
    banque.dessiner("goutte0", 10.0, 8.0, false);
    texte::dessiner(
        &s.gouttes_en_poche.to_string(),
        23.0,
        8.0,
        ombre(OR_PALE, 2),
    );

    // Onglets (seulement ceux qui ont des objets).
    let noms: Vec<&str> = e.onglets.iter().map(|c| c.nom()).collect();
    for (i, nom) in noms.iter().enumerate() {
        let (x, y, w, h) = rect_onglet(&noms, i);
        let choisi = i == e.onglet;
        panneau(
            x,
            y,
            w,
            h,
            if choisi { ARDOISE } else { ARDOISE_NUIT },
            if choisi { OR } else { ARDOISE_CLAIRE },
        );
        let c = if choisi { OR_PALE } else { BRUME };
        texte::dessiner(nom, x + 6.0, y + 3.0, Style::simple(c, 1));
    }

    // Objets.
    let nb = e.objets.len();
    let debut = premiere_visible(e.menu.selection(), nb);
    for (k, objet) in e.objets.iter().enumerate().skip(debut).take(MAX_VISIBLES) {
        let r @ (x, y, w, _) = rect_ligne(k - debut);
        cadre(r, k == e.menu.selection());
        // Aperçu : petite case sombre, le skin qui court (ou la goutte du factice).
        panneau(x + 4.0, y + 3.0, 24.0, 22.0, ARDOISE_OMBRE, ARDOISE);
        match objet.apercu {
            Apercu::Skin(skin) => banque.dessiner(
                image(&skin.images().course, anim, 5),
                x + 8.0,
                y + 6.0,
                false,
            ),
            #[cfg(any(test, feature = "debug_niveaux"))]
            Apercu::Factice => banque.dessiner("goutte0", x + 11.0, y + 9.0, false),
        }
        texte::dessiner(objet.nom, x + 34.0, y + 3.0, ombre(BLANC_CASSE, 2));
        let prix = if objet.prix == 0 {
            contenu::GRATUIT.to_string()
        } else {
            contenu::gouttes(objet.prix)
        };
        texte::dessiner(&prix, x + 34.0, y + 19.0, ombre(OR, 1));
        let (etat, couleur) = match objet.etat(s) {
            Etat::Equipe => (contenu::ETAT_EQUIPE, OR_PALE),
            Etat::Achete => (contenu::ETAT_ACHETE, BLANC_CASSE),
            Etat::AAcheter => (contenu::ETAT_A_ACHETER, BRUME),
        };
        texte::dessiner_a_droite(etat, x + w - 8.0, y + 11.0, ombre(couleur, 1));
    }
    // Il y a d'autres objets au-dessus / en dessous : une petite flèche.
    if debut > 0 {
        texte::dessiner("^", X + L + 6.0, Y_LIGNES, Style::simple(OR, 1));
    }
    if debut + MAX_VISIBLES < nb {
        texte::dessiner(
            "v",
            X + L + 6.0,
            Y_LIGNES + MAX_VISIBLES as f32 * PAS_LIGNE - 10.0,
            Style::simple(OR, 1),
        );
    }
    bouton((X, Y_RETOUR, L, H_RETOUR), contenu::RETOUR, e.sur_retour());
    texte::dessiner_centre(contenu::AIDE_BOUTIQUE, Y_AIDE, Style::simple(BRUME, 1));

    // Fenêtre de dialogue, sur un fond assombri.
    if let Some(d) = e.dialogue {
        trame();
        let (x, y, w, h) = DIALOGUE;
        panneau(x, y, w, h, ARDOISE_NUIT, OR);
        match d {
            Dialogue::Confirmer { prix, oui } => {
                texte::dessiner_centre(
                    &contenu::acheter_pour(prix),
                    y + 16.0,
                    ombre(BLANC_CASSE, 2),
                );
                bouton(BOUTON_OUI, contenu::OUI, oui);
                bouton(BOUTON_NON, contenu::NON, !oui);
            }
            Dialogue::Manque { n } => {
                texte::dessiner_centre(contenu::IL_TE_MANQUE, y + 8.0, ombre(BLANC_CASSE, 2));
                texte::dessiner_centre(&contenu::gouttes(n), y + 28.0, ombre(OR, 2));
                bouton(BOUTON_OK, contenu::OK, true);
            }
        }
    }
}
