//! Le fond de la RUE du caniveau (carte 9.6b) : une rangée de maisons vues du
//! trottoir (façades, fenêtres allumées, vitrines sous leur store, descentes de
//! gouttière) et, devant, les lampadaires et leur halo.
//!
//! Deux motifs du Monde 2 (`decor::Detail`) : `Facades` (entre la couche lointaine
//! et la couche proche) et `Lampadaires` (devant). La rue est une suite FIXE de huit
//! maisons qui se répète : chaque maison est décrite à la main dans `RUE` (largeur,
//! étages, couleur, store, gouttière). Tout est en rectangles de la palette, au
//! pixel entier, sans dégradé ni transparence.

use macroquad::prelude::draw_rectangle;
use pepin_core::palette::{
    Couleur, ARDOISE, ARDOISE_CLAIRE, ARDOISE_NUIT, ARDOISE_OMBRE, ARDOISE_PALE, NUAGE,
    ORANGE_SOLEIL, VIOLET_ORAGE,
};

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
    /// Le rez-de-chaussée est une boutique : vitrine éclairée sous un store rayé
    /// (sinon une porte et un soupirail).
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

/// La rue : huit maisons, de gauche à droite, qui se répètent.
const RUE: [Maison; 8] = [
    m(64, 3, ARDOISE, true, true),
    m(48, 2, ARDOISE_NUIT, false, false),
    m(80, 4, ARDOISE, true, true),
    m(48, 3, VIOLET_ORAGE, false, true),
    m(64, 2, ARDOISE_NUIT, true, false),
    m(64, 4, ARDOISE, true, true),
    m(48, 3, ARDOISE_NUIT, false, true),
    m(80, 3, ARDOISE, true, false),
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

/// Les maisons de la rue, posées sur la ligne `sol` (px, à l'écran), décalées de
/// `dec` px par la parallaxe.
pub(crate) fn dessiner_facades(dec: i32, sol: i32) {
    let p = periode();
    // Le trottoir d'en face, au pied des maisons : sa bordure claire, puis l'ombre
    // jusqu'en bas (on la voit dans les creux de la rue).
    rect(0, sol, L, 2, ARDOISE);
    rect(0, sol + 2, L, 60, ARDOISE_OMBRE);
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

/// Dessine une maison dont le coin bas-gauche est `(x, sol)`.
fn maison(m: &Maison, x: i32, sol: i32, numero: i32) {
    let w = m.largeur;
    let haut = sol - REZ - m.etages * ETAGE;
    // La façade, cernée d'un trait sombre de chaque côté (les murs mitoyens) :
    // elle se détache du ciel même quand elle en a la couleur.
    rect(x, haut, w, sol - haut, m.corps);
    rect(x, haut, 1, sol - haut, ARDOISE_OMBRE);
    rect(x + w - 1, haut, 1, sol - haut, ARDOISE_OMBRE);
    // Le toit en mansarde : ardoise sombre, faîte clair, une lucarne allumée ou non.
    rect(x + 2, haut - TOIT, w - 4, TOIT, ARDOISE_OMBRE);
    rect(x + 4, haut - TOIT - 1, w - 8, 1, ARDOISE_CLAIRE);
    let lucarne = x + w / 2 - 2;
    rect(lucarne - 1, haut - TOIT + 1, 6, 6, ARDOISE_CLAIRE);
    let vitre = if allumee(numero, 9, 0) {
        ORANGE_SOLEIL
    } else {
        ARDOISE_NUIT
    };
    rect(lucarne, haut - TOIT + 2, 4, 4, vitre);
    // La corniche sous le toit, et le bandeau entre le rez-de-chaussée et l'étage.
    rect(x, haut, w, 2, ARDOISE_CLAIRE);
    rect(x, haut + 2, w, 1, ARDOISE_OMBRE);
    rect(x, sol - REZ - 2, w, 2, ARDOISE_CLAIRE);
    // Les étages : une fenêtre par tranche de 16 px (cadre sombre, croisée, appui).
    for e in 0..m.etages {
        let y = haut + 4 + e * ETAGE;
        for c in 0..w / 16 {
            let fx = x + c * 16 + 5;
            rect(fx - 1, y + 1, 8, 11, ARDOISE_OMBRE);
            let vitre = if allumee(numero, e, c) {
                ORANGE_SOLEIL
            } else {
                ARDOISE_NUIT
            };
            rect(fx, y + 2, 6, 9, vitre);
            rect(fx + 2, y + 2, 1, 9, ARDOISE_OMBRE);
            rect(fx, y + 5, 6, 1, ARDOISE_OMBRE);
            rect(fx - 1, y + 12, 8, 1, ARDOISE_PALE);
        }
    }
    // Le rez-de-chaussée.
    let rez = sol - REZ;
    if m.boutique {
        // Le store rayé (ardoise et violet, plus sourd que le store du niveau, qui
        // est un solide où l'on se cogne) et son lambrequin festonné, puis la
        // vitrine : une vitre SOMBRE (la boutique est fermée la nuit : rien de clair
        // là où passent Pépin et les gouttes), ses reflets, ses montants, et une
        // seule lampe orange au fond.
        for s in 0..(w - 4) / 4 {
            let c = if s % 2 == 0 {
                ARDOISE_PALE
            } else {
                VIOLET_ORAGE
            };
            rect(x + 2 + s * 4, rez, 4, 4, c);
            rect(x + 3 + s * 4, rez + 4, 2, 1, c);
        }
        rect(x + 2, rez - 1, w - 4, 1, ARDOISE_OMBRE);
        rect(x + 4, rez + 6, w - 8, 14, ARDOISE_OMBRE);
        rect(x + 5, rez + 7, w - 10, 13, ARDOISE_NUIT);
        for mx in (x + 5 + 11..x + w - 6).step_by(12) {
            rect(mx, rez + 7, 1, 13, ARDOISE_OMBRE);
        }
        for k in 0..2 {
            rect(x + 7 + k * 3, rez + 15 - k * 3, 1, 3, ARDOISE);
        }
        let lampe = x + w / 2 + 3;
        rect(lampe, rez + 7, 1, 3, ARDOISE_OMBRE);
        rect(lampe - 1, rez + 10, 3, 2, ORANGE_SOLEIL);
    } else {
        // Une porte cochère sombre et son imposte, un soupirail.
        let px = x + w / 2 - 6;
        rect(px - 1, rez + 1, 14, 19, ARDOISE_OMBRE);
        rect(px, rez + 2, 12, 18, ARDOISE_NUIT);
        rect(px + 6, rez + 6, 1, 14, ARDOISE_OMBRE);
        rect(px + 1, rez + 3, 10, 2, ARDOISE_CLAIRE);
        rect(x + 4, sol - 7, 8, 3, ARDOISE_OMBRE);
    }
    // La descente de gouttière : un tuyau de zinc le long du bord droit, ses
    // colliers sombres, de la corniche jusqu'au trottoir.
    if m.gouttiere {
        let gx = x + w - 4;
        rect(gx, haut + 3, 2, sol - haut - 3, ARDOISE_PALE);
        rect(gx + 1, haut + 3, 1, sol - haut - 3, ARDOISE_CLAIRE);
        let mut y = haut + 8;
        while y < sol {
            rect(gx - 1, y, 4, 1, ARDOISE_OMBRE);
            y += 12;
        }
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
/// un mât de fonte sombre, une crosse, la lanterne orange et son halo.
pub(crate) fn dessiner_lampadaires(dec: i32, sol: i32) {
    for lx in [60i32, 230] {
        let x = (lx - dec).rem_euclid(L);
        let tete = sol - 46;
        // Le halo : deux disques de lumière autour de la lanterne, en marches
        // d'escalier (un cercle de pixels, sans aucune demi-teinte).
        for (r, c) in [(9, ARDOISE_CLAIRE), (5, ARDOISE_PALE)] {
            disque(x + 1, tete + 2, r, c);
        }
        // Le mât, son socle évasé, sa crosse.
        rect(x, tete + 4, 2, sol - tete - 4, ARDOISE_OMBRE);
        rect(x - 2, sol - 10, 6, 10, ARDOISE_OMBRE);
        rect(x - 1, sol - 14, 4, 4, ARDOISE_OMBRE);
        rect(x - 1, tete + 9, 4, 1, ARDOISE_OMBRE);
        // La lanterne : chapeau, vitres orange, cœur clair, fond.
        rect(x - 2, tete - 2, 6, 2, ARDOISE_OMBRE);
        rect(x - 3, tete, 8, 1, ARDOISE_OMBRE);
        rect(x - 2, tete + 1, 6, 4, ARDOISE_OMBRE);
        rect(x - 1, tete + 1, 4, 3, ORANGE_SOLEIL);
        rect(x, tete + 2, 2, 1, NUAGE);
    }
}
