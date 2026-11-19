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
//!
//! Carte 9.20h : les descentes de gouttière `J` du niveau sont dessinées ici aussi
//! (`dessiner_gouttieres`, `dessiner_filets`), chacune accrochée à son pan de
//! maison : chéneau, coude, colliers, dauphin qui verse dans le caniveau.

use macroquad::prelude::draw_rectangle;
use pepin_core::aabb::TAILLE_TUILE;
use pepin_core::level::GenreDecor;
use pepin_core::palette::{Couleur, ARDOISE, ARDOISE_NUIT, ARDOISE_OMBRE, VIOLET_ORAGE};

use pepin_core::LARGEUR_LOGIQUE;

use crate::monde::Monde;
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

/// Un rectangle à dessiner : `(x, y, largeur, hauteur, couleur)`.
type Rect = (i32, i32, i32, i32, Couleur);

/// Largeur (px) du pan de mur derrière une descente de gouttière (trois fenêtres).
const PAN: i32 = 48;
/// Bord gauche du pan de mur, depuis le bord gauche de la case `J`.
const PAN_X: i32 = -18;
/// Le tuyau (3 px de large), depuis le bord gauche de la case `J`.
const TUYAU_X: i32 = 26;
/// Le bec du dauphin (dernier pixel), depuis le bord gauche de la case `J` : au-dessus
/// de la 1re case d'eau (la lettre `J` se pose deux cases avant l'eau).
const BEC_X: i32 = 39;
/// Le dessous du dauphin, au-dessus du trottoir (px) : l'eau tombe de là.
const DAUPHIN_BAS: i32 = 8;

/// Carte 9.20h (« les gouttières sont rattachées à rien ») : la descente de
/// gouttière `J` du caniveau, ACCROCHÉE à une maison. Le fond défile plus lentement
/// que le niveau, ses maisons ne sont jamais au même endroit derrière le tuyau :
/// chaque `J` a donc son propre pan de façade, posé dans le niveau, qui prolonge la
/// rue du fond jusqu'au caniveau (mêmes tons sombres et ternes, toit en pente, sans
/// dessus plat clair). Sur ce mur : le CHÉNEAU au bord du toit, son COUDE, le tuyau
/// fixé par trois COLLIERS, et en bas le DAUPHIN, coudé vers la rue, au-dessus de la
/// 1re case d'eau. Rien de tout cela n'a de collision. `(x, sol)` : bord gauche de
/// la case `J` et haut du trottoir sous elle (px, à l'écran) ; `numero` choisit les
/// fenêtres allumées.
fn gouttiere_accrochee(x: i32, sol: i32, numero: i32) -> Vec<Rect> {
    let x0 = x + PAN_X;
    let haut = sol - REZ - 2 * ETAGE;
    let mut r: Vec<Rect> = vec![(x0, haut, PAN, sol - haut, ARDOISE_NUIT)];
    // Le toit en mansarde, en pente (aucun dessus plat), et sa lucarne.
    for k in 0..TOIT {
        r.push((x0 + 1 + k, haut - 1 - k, PAN - 2 - 2 * k, 1, ARDOISE_OMBRE));
    }
    let lucarne = vitre(numero, 9, 0, ARDOISE_NUIT);
    r.push((x0 + PAN / 2 - 4, haut - TOIT + 3, 4, 4, lucarne));
    // Deux étages de fenêtres (comme les maisons du fond), une porte cochère.
    for e in 0..2 {
        let y = haut + 4 + e * ETAGE;
        for c in 0..PAN / 16 {
            let fx = x0 + c * 16 + 5;
            r.push((fx, y + 2, 6, 9, vitre(numero, e, c, ARDOISE_OMBRE)));
            r.push((fx + 2, y + 2, 1, 9, ARDOISE_NUIT));
            r.push((fx, y + 5, 6, 1, ARDOISE_NUIT));
        }
    }
    r.push((x0, sol - REZ - 1, PAN, 1, ARDOISE_OMBRE));
    r.push((x0 + 6, sol - 18, 12, 18, ARDOISE_OMBRE));
    r.push((x0 + 12, sol - 14, 1, 14, ARDOISE_NUIT));
    // Le chéneau : la gouttière pendue au bord du toit, d'un bout à l'autre du pan,
    // qui dépasse à droite. Lèvre sombre DESSUS : rien de clair où se poser.
    // Mince (un seul pixel de zinc entre deux ombres) : pas une corniche.
    r.push((x0, haut, PAN + 2, 1, ARDOISE_OMBRE));
    r.push((x0, haut + 1, PAN + 2, 1, ARDOISE));
    r.push((x0, haut + 2, PAN + 2, 1, ARDOISE_OMBRE));
    // Le coude (col de cygne) : sous le bout du chéneau, il ramène l'eau vers le mur.
    let t = x + TUYAU_X;
    r.push((t + 2, haut + 3, 2, 2, ARDOISE));
    r.push((t + 4, haut + 3, 1, 2, ARDOISE_OMBRE));
    r.push((t + 1, haut + 5, 2, 2, ARDOISE));
    r.push((t + 3, haut + 5, 1, 2, ARDOISE_OMBRE));
    // Le tuyau, contre le mur (ombre portée à droite) jusqu'au dauphin.
    let bas = sol - DAUPHIN_BAS - 3;
    r.push((t, haut + 7, 2, bas - haut - 7, ARDOISE));
    r.push((t + 2, haut + 7, 1, bas - haut - 7, ARDOISE_OMBRE));
    // Trois colliers sombres (une bande courte, pas des barreaux d'échelle).
    for y in [haut + 12, haut + 24, haut + 36] {
        r.push((t - 1, y, 5, 2, ARDOISE_OMBRE));
    }
    // Le dauphin : le pied du tuyau se coude vers la rue, au-dessus du caniveau.
    // Son dessus est une ombre, comme le chéneau : pas une marche claire.
    r.push((t, bas, 2, 1, ARDOISE));
    r.push((t + 2, bas, x + BEC_X - 1 - t, 1, ARDOISE_OMBRE));
    r.push((t, bas + 1, x + BEC_X + 1 - t, 1, ARDOISE));
    r.push((t, bas + 2, x + BEC_X + 1 - t, 1, ARDOISE_OMBRE));
    r.push((x + BEC_X, bas, 1, 3, ARDOISE_OMBRE));
    r
}

/// Le filet d'eau sous le bec du dauphin, deux images : de `x + FILET_X`, sur
/// `DAUPHIN_BAS` rangées jusqu'à la surface de l'eau. Le filet blanc d'écume cerné
/// de sombre (celui de la carte 9.20a) ; au point d'arrivée, sur l'eau, de petites
/// éclaboussures CYAN (`D`, la pluie) qui sautent de part et d'autre. Chiffres : la
/// palette (`sprites::pixel`).
const FILET: [[&str; DAUPHIN_BAS as usize]; 2] = [
    [
        "..0760..", "..0670..", "..0760..", "..0760..", "..0670..", "..0760..", ".D0760D.",
        "D.0DD0.D",
    ],
    [
        "..0670..", "..0760..", "..0670..", "..0670..", "..0760..", "..0670..", "D.0670.D",
        ".D0DD0D.",
    ],
];
/// Bord gauche du dessin du filet, depuis le bord gauche de la case `J`.
const FILET_X: i32 = 34;

/// Le filet d'eau qui sort du dauphin et tombe dans le caniveau, image du pas `pas`
/// (tous les 10 pas, comme à la carte 9.20a).
fn filet_de_gouttiere(x: i32, sol: i32, pas: u32) -> Vec<Rect> {
    let image = &FILET[((pas / 10) % 2) as usize];
    let mut r: Vec<Rect> = Vec::new();
    for (dy, ligne) in image.iter().enumerate() {
        for (dx, c) in ligne.chars().enumerate() {
            if let Ok(Some(couleur)) = crate::sprites::pixel(c) {
                let y = sol - DAUPHIN_BAS + dy as i32;
                r.push((x + FILET_X + dx as i32, y, 1, 1, couleur));
            }
        }
    }
    r
}

/// Les descentes de gouttière `J` du niveau, à l'écran : `(x, sol)` de chacune
/// (bord gauche de sa case, haut du trottoir sous elle), et un numéro fixe.
fn gouttieres_a_l_ecran(monde: &Monde, cx: i32, cy: i32) -> Vec<(i32, i32, i32)> {
    let t = TAILLE_TUILE as i32;
    monde
        .niveau
        .decors
        .iter()
        .filter(|d| d.genre == GenreDecor::Gouttiere)
        .map(|d| {
            let (x, y) = (d.x.round() as i32, d.y.round() as i32);
            (x - cx, y + t - cy, x / t)
        })
        .filter(|&(x, _, _)| x + PAN_X + PAN + 16 > 0 && x + PAN_X < L)
        .collect()
}

/// Première passe, avant les tuiles et le mobilier (une voiture garée passe
/// devant) : chaque descente de gouttière et le pan de maison qui la porte.
pub(crate) fn dessiner_gouttieres(monde: &Monde, cx: i32, cy: i32) {
    for (x, sol, numero) in gouttieres_a_l_ecran(monde, cx, cy) {
        for (rx, ry, w, h, c) in gouttiere_accrochee(x, sol, numero) {
            rect(rx, ry, w, h, c);
        }
    }
}

/// Seconde passe, après les tuiles (sur l'eau) : le filet de chaque dauphin.
pub(crate) fn dessiner_filets(monde: &Monde, cx: i32, cy: i32) {
    for (x, sol, _) in gouttieres_a_l_ecran(monde, cx, cy) {
        for (rx, ry, w, h, c) in filet_de_gouttiere(x, sol, monde.pas) {
            rect(rx, ry, w, h, c);
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

#[cfg(test)]
mod tests {
    use super::*;
    use pepin_core::palette::{BLANC_CASSE, NUAGE, PLUIE};

    /// Les pixels couverts par une liste de rectangles (le dernier posé l'emporte).
    fn pixels(r: &[Rect]) -> std::collections::HashMap<(i32, i32), Couleur> {
        let mut p = std::collections::HashMap::new();
        for &(x, y, w, h, c) in r {
            for py in y..y + h {
                for px in x..x + w {
                    p.insert((px, py), c);
                }
            }
        }
        p
    }

    /// Carte 9.20h : la descente de gouttière est ACCROCHÉE. Du chéneau au dauphin,
    /// le tuyau est d'un seul tenant (chaque rangée touche la suivante), tout contre
    /// le mur de son pan de maison ; le chéneau court au bord du toit ; le bec du
    /// dauphin est au-dessus de la 1re case d'eau (deux cases après la lettre `J`).
    #[test]
    fn la_gouttiere_est_accrochee_a_sa_maison() {
        let (x, sol) = (160, 112);
        let p = pixels(&gouttiere_accrochee(x, sol, 7));
        let haut = sol - REZ - 2 * ETAGE;
        // Le mur derrière tout le tuyau, du toit au trottoir.
        assert!((haut..sol).all(|y| p.contains_key(&(x + PAN_X, y))));
        // Le chéneau : du bord gauche du pan jusqu'au-dessus du tuyau.
        assert!((x + PAN_X..=x + TUYAU_X + 4).all(|px| p[&(px, haut + 1)] == ARDOISE));
        // Le zinc (ardoise) du tuyau, rangée par rangée, du chéneau au dauphin :
        // chaque rangée chevauche la précédente (coude compris), sans trou.
        let zinc = |y: i32| -> Vec<i32> {
            (x + TUYAU_X - 1..=x + BEC_X)
                .filter(|&px| p.get(&(px, y)) == Some(&ARDOISE))
                .collect()
        };
        let bas = sol - DAUPHIN_BAS - 3;
        let mut avant = zinc(haut + 1);
        for y in haut + 3..bas {
            let ici: Vec<i32> = zinc(y)
                .into_iter()
                .filter(|&px| px <= x + TUYAU_X + 4)
                .collect();
            // Un collier couvre le tuyau d'ombre : la rangée sous lui reprend.
            if ici.is_empty() {
                assert_eq!(p[&(x + TUYAU_X, y)], ARDOISE_OMBRE, "rangée {y}");
                continue;
            }
            assert!(
                ici.iter().any(|px| avant.contains(px)),
                "tuyau coupé en {y}"
            );
            avant = ici;
        }
        // Un collier : le tuyau passe à l'ombre (de part et d'autre), après du zinc.
        let colliers = (haut + 8..bas)
            .filter(|&y| {
                let t = x + TUYAU_X;
                p[&(t - 1, y)] == ARDOISE_OMBRE
                    && p[&(t + 1, y)] == ARDOISE_OMBRE
                    && p[&(t + 1, y - 1)] == ARDOISE
            })
            .count();
        assert!((2..=3).contains(&colliers), "{colliers} colliers");
        // Le dauphin : du tuyau au bec, au-dessus de l'eau (case `J` + 2).
        assert!((x + TUYAU_X..=x + BEC_X - 1).all(|px| p[&(px, bas + 1)] == ARDOISE));
        assert!((x + 32..x + 48).contains(&(x + BEC_X)));
        // Rien sous le haut du trottoir (l'eau et les pavés sont devant).
        assert!(p.keys().all(|&(_, y)| y < sol));
    }

    /// Le pan de maison et la gouttière sont du FOND (carte 9.20g) : les seuls tons
    /// sombres du fond, et aucun dessus plat clair : au-dessus de tout pixel
    /// d'ardoise (le plus clair), sur une rangée horizontale de plus de 12 px, il y
    /// a de l'ombre, jamais le ciel. Seul le filet est clair (blanc d'écume, et ses
    /// éclaboussures cyan) : il tombe sous le bec jusqu'à la surface de l'eau.
    #[test]
    fn la_gouttiere_et_sa_maison_sont_du_fond_terne() {
        let (x, sol) = (160, 112);
        let p = pixels(&gouttiere_accrochee(x, sol, 3));
        let fond = [ARDOISE_OMBRE, ARDOISE_NUIT, ARDOISE, VIOLET_ORAGE];
        assert!(p.values().all(|c| fond.contains(c)));
        for (&(px, py), &c) in &p {
            if c == ARDOISE && !p.contains_key(&(px, py - 1)) {
                panic!("dessus d'ardoise à l'air libre en ({px}, {py})");
            }
        }
        for pas in [0, 10] {
            let f = pixels(&filet_de_gouttiere(x, sol, pas));
            assert!(f
                .values()
                .all(|c| [ARDOISE_OMBRE, NUAGE, BLANC_CASSE, PLUIE].contains(c)));
            // Les éclaboussures cyan, juste au-dessus de la surface.
            assert!(f.iter().any(|(&(_, y), &c)| c == PLUIE && y == sol - 1));
            // Sous le bec, jusqu'à la surface (rien dessous), au-dessus de l'eau.
            assert!((sol - DAUPHIN_BAS..sol).all(|y| f.contains_key(&(x + BEC_X - 1, y))));
            assert!(f
                .keys()
                .all(|&(px, y)| y < sol && px >= x + 32 && px < x + 48));
        }
    }
}
