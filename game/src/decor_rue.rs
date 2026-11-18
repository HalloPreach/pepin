//! Le fond de la RUE du caniveau (carte 9.6b) : une rangée de maisons vues du
//! trottoir (façades, fenêtres allumées, vitrines sous leur store, descentes de
//! gouttière) et, devant, les lampadaires et leur halo.
//!
//! Deux motifs du Monde 2 (`decor::Detail`) : `Facades` (entre la couche lointaine
//! et la couche proche) et `Lampadaires` (devant). La rue est une suite FIXE de huit
//! maisons qui se répète : chaque maison est décrite à la main dans `RUE` (largeur,
//! étages, couleur, store, gouttière). Tout est en rectangles de la palette, au
//! pixel entier, sans dégradé ni transparence.
//!
//! Carte 9.20g (« c'est dur de savoir qu'est-ce qu'est un obstacle comparé au
//! fond ») : tout ce fond est SOMBRE et TERNE. Il n'emploie que l'ardoise d'ombre,
//! l'ardoise de nuit, l'ardoise et le violet d'orage : aucun
//! contour sombre net, aucune couleur vive, aucun dessus plat clair (ni corniche,
//! ni appui de fenêtre, ni faîte). Les teintes claires restent au premier plan, à
//! ce qu'on touche (pavés, mobilier de rue), cerné de sombre et liseré de clair.

use macroquad::prelude::draw_rectangle;
use pepin_core::palette::{Couleur, ARDOISE, ARDOISE_NUIT, ARDOISE_OMBRE, VIOLET_ORAGE};

use pepin_core::LARGEUR_LOGIQUE;

use crate::rendu::couleur_mq;

const L: i32 = LARGEUR_LOGIQUE as i32;

fn rect(x: i32, y: i32, w: i32, h: i32, c: Couleur) {
    draw_rectangle(x as f32, y as f32, w as f32, h as f32, couleur_mq(c));
}

/// Une maison de la rue, telle qu'on la voit du trottoir.
struct Maison {
    /// Largeur (px), multiple de 16 : une fenêtre par tranche de 16 px.
    largeur: i32,
    /// Étages au-dessus du rez-de-chaussée (16 px chacun).
    etages: i32,
    /// Couleur de la façade.
    corps: Couleur,
    /// Le rez-de-chaussée est une boutique : vitrine sous un store rayé (sinon une
    /// porte et un soupirail).
    boutique: bool,
    /// Une descente de gouttière le long du bord droit.
    gouttiere: bool,
}

/// Raccourci pour la table `RUE` (une maison par ligne) : largeur, étages, couleur
/// de la façade, boutique, gouttière.
const fn m(largeur: i32, etages: i32, corps: Couleur, boutique: bool, gouttiere: bool) -> Maison {
    Maison {
        largeur,
        etages,
        corps,
        boutique,
        gouttiere,
    }
}

/// La rue : huit maisons, de gauche à droite, qui se répètent. Carte 9.20g : deux
/// tons sombres en alternance (ardoise, ardoise de nuit), plus de façade violette ;
/// deux maisons voisines n'ont jamais la même teinte (pas besoin de mur mitoyen
/// cerné pour les séparer).
const RUE: [Maison; 8] = [
    m(64, 3, ARDOISE, true, true),
    m(48, 2, ARDOISE_NUIT, false, false),
    m(80, 4, ARDOISE, true, true),
    m(48, 3, ARDOISE_NUIT, false, true),
    m(64, 2, ARDOISE, true, false),
    m(64, 4, ARDOISE_NUIT, true, true),
    m(48, 3, ARDOISE, false, true),
    m(80, 3, ARDOISE_NUIT, true, false),
];

/// Hauteur du rez-de-chaussée (px) et d'un étage.
const REZ: i32 = 20;
const ETAGE: i32 = 16;
/// Hauteur du toit en mansarde (px).
const TOIT: i32 = 8;

/// Longueur (px) d'un tour de la rue : la somme des largeurs.
fn periode() -> i32 {
    RUE.iter().map(|m| m.largeur).sum()
}

/// Fenêtre allumée ? Fixe (même maison, même fenêtre : même réponse), environ une
/// sur quatre : la rue dort, le fond reste plus sombre que le niveau.
fn allumee(maison: i32, etage: i32, colonne: i32) -> bool {
    let h = (maison as u32).wrapping_mul(73)
        ^ (etage as u32).wrapping_mul(151)
        ^ (colonne as u32).wrapping_mul(37);
    (h.wrapping_mul(0x2C1B_3C6D) >> 7).is_multiple_of(4)
}

/// Le ton « en creux » d'une façade (vitres éteintes, croisées, porte, ombres) :
/// un cran plus sombre que le mur.
fn creux(corps: Couleur) -> Couleur {
    if corps == ARDOISE {
        ARDOISE_NUIT
    } else {
        ARDOISE_OMBRE
    }
}

/// La vitre d'une fenêtre : faiblement allumée (violet d'orage, jamais l'orange
/// vif du premier plan) ou éteinte.
fn vitre(maison: i32, etage: i32, colonne: i32, eteinte: Couleur) -> Couleur {
    if allumee(maison, etage, colonne) {
        VIOLET_ORAGE
    } else {
        eteinte
    }
}

/// Les maisons de la rue, posées sur la ligne `sol` (px, à l'écran), décalées de
/// `dec` px par la parallaxe.
pub(crate) fn dessiner_facades(dec: i32, sol: i32) {
    let p = periode();
    // Le trottoir d'en face, au pied des maisons : l'ombre jusqu'en bas (on la voit
    // dans les creux de la rue). Carte 9.20g : plus de bordure claire, seul le
    // trottoir du niveau a un dessus clair.
    rect(0, sol, L, 62, ARDOISE_OMBRE);
    let tour0 = dec.div_euclid(p) - 1;
    for tour in tour0..=tour0 + 2 {
        let mut x = tour * p - dec;
        for (k, m) in RUE.iter().enumerate() {
            if x < L && x + m.largeur > 0 {
                maison(m, x, sol, tour * 8 + k as i32);
            }
            x += m.largeur;
        }
    }
}

/// Dessine une maison dont le coin bas-gauche est `(x, sol)`. Carte 9.20g : sans
/// contour, sans rebord clair ; le toit en mansarde est en PENTE (un pixel de
/// retrait par rangée de chaque côté) : aucun dessus plat où l'on croirait se poser.
fn maison(m: &Maison, x: i32, sol: i32, numero: i32) {
    let w = m.largeur;
    let haut = sol - REZ - m.etages * ETAGE;
    let creux = creux(m.corps);
    rect(x, haut, w, sol - haut, m.corps);
    // Le toit en mansarde, et sa lucarne éteinte ou faiblement allumée.
    for k in 0..TOIT {
        rect(x + 1 + k, haut - 1 - k, w - 2 - 2 * k, 1, ARDOISE_OMBRE);
    }
    let lucarne = x + w / 2 - 2;
    rect(
        lucarne,
        haut - TOIT + 3,
        4,
        4,
        vitre(numero, 9, 0, ARDOISE_NUIT),
    );
    // Le bandeau entre le rez-de-chaussée et l'étage : une ombre, pas un rebord.
    rect(x, sol - REZ - 1, w, 1, creux);
    // Les étages : une fenêtre par tranche de 16 px (vitre et croisée, sans cadre
    // ni appui).
    for e in 0..m.etages {
        let y = haut + 4 + e * ETAGE;
        for c in 0..w / 16 {
            let fx = x + c * 16 + 5;
            rect(fx, y + 2, 6, 9, vitre(numero, e, c, creux));
            rect(fx + 2, y + 2, 1, 9, m.corps);
            rect(fx, y + 5, 6, 1, m.corps);
        }
    }
    // Le rez-de-chaussée.
    let rez = sol - REZ;
    if m.boutique {
        // Le store rayé, en deux tons sombres (le store du niveau, lui, est un
        // solide où l'on se cogne), son lambrequin festonné, puis la vitrine : une
        // vitre SOMBRE (la boutique est fermée la nuit), ses montants, ses reflets
        // et une veilleuse violette au fond.
        for s in 0..(w - 4) / 4 {
            let c = if s % 2 == 0 { ARDOISE_OMBRE } else { creux };
            rect(x + 2 + s * 4, rez, 4, 4, c);
            rect(x + 3 + s * 4, rez + 4, 2, 1, c);
        }
        rect(x + 4, rez + 6, w - 8, 14, creux);
        for mx in (x + 5 + 11..x + w - 6).step_by(12) {
            rect(mx, rez + 7, 1, 13, m.corps);
        }
        for k in 0..2 {
            rect(x + 7 + k * 3, rez + 15 - k * 3, 1, 3, m.corps);
        }
        let lampe = x + w / 2 + 3;
        rect(lampe - 1, rez + 10, 3, 2, VIOLET_ORAGE);
    } else {
        // Une porte cochère sombre et son battant, un soupirail.
        let px = x + w / 2 - 6;
        rect(px, rez + 2, 12, 18, creux);
        rect(px + 6, rez + 6, 1, 14, m.corps);
        rect(x + 4, sol - 7, 8, 3, creux);
    }
    // La descente de gouttière : un tuyau de zinc terne (un cran plus sombre que
    // le mur) le long du bord droit, du toit jusqu'au trottoir, sans colliers.
    if m.gouttiere {
        rect(x + w - 4, haut, 2, sol - haut, creux);
    }
}

/// Un disque plein de rayon `r` centré en `(cx, cy)`, rangée par rangée : la
/// demi-largeur de chaque rangée est arrondie au pixel.
fn disque(cx: i32, cy: i32, r: i32, c: Couleur) {
    for dy in -r..=r {
        let demi = (((r * r - dy * dy) as f32).sqrt() + 0.3) as i32;
        rect(cx - demi, cy + dy, 2 * demi + 1, 1, c);
    }
}

/// Les lampadaires de la rue, au pied `sol` (px, à l'écran), décalés de `dec` px :
/// un mât de fonte sombre, une crosse, la lanterne et son halo. Carte 9.20g : du
/// FOND, donc ternes : un halo d'ardoise, une lanterne violette (l'orange vif reste
/// aux lanternes du niveau, devant), un chapeau pointu.
pub(crate) fn dessiner_lampadaires(dec: i32, sol: i32) {
    for lx in [60i32, 230] {
        let x = (lx - dec).rem_euclid(L);
        let tete = sol - 46;
        // Le halo : un disque de lumière autour de la lanterne, en marches
        // d'escalier (un cercle de pixels, sans aucune demi-teinte).
        disque(x + 1, tete + 2, 7, ARDOISE);
        // Le mât, son socle évasé, sa crosse.
        rect(x, tete + 4, 2, sol - tete - 4, ARDOISE_OMBRE);
        rect(x - 2, sol - 10, 6, 10, ARDOISE_OMBRE);
        rect(x - 1, sol - 14, 4, 4, ARDOISE_OMBRE);
        rect(x - 1, tete + 9, 4, 1, ARDOISE_OMBRE);
        // La lanterne : chapeau pointu, vitres, fond.
        rect(x, tete - 2, 2, 1, ARDOISE_OMBRE);
        rect(x - 1, tete - 1, 4, 1, ARDOISE_OMBRE);
        rect(x - 2, tete, 6, 5, ARDOISE_OMBRE);
        rect(x - 1, tete + 1, 4, 3, VIOLET_ORAGE);
    }
}
