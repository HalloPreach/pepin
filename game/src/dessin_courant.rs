//! Les stries du courant (carte 9.6b) : sur l'eau qui coule (`{`, `}`), de courtes
//! stries défilent dans le SENS du courant, par-dessus les tuiles d'eau. Elles
//! remplacent les chevrons « > » dessinés dans les anciennes tuiles, qui se
//! lisaient comme du texte.
//!
//! Une strie fait 5 px : une queue de 4 px et une tête plus claire à l'avant (le
//! sens se lit même sur une image fixe, comme les traînées du vent latéral). Deux
//! lignes de stries par case : une dans la bande claire sous la surface, une dans
//! le fond. Les positions sont comptées dans le NIVEAU (pas dans la case) : les
//! stries passent d'une case à la suivante sans saut, et chaque case ne dessine
//! que ses propres pixels. Tout est en couleurs de la palette, au pixel entier.

use macroquad::prelude::draw_rectangle;
use pepin_core::aabb::TAILLE_TUILE;
use pepin_core::level::Tuile;
use pepin_core::palette::{Couleur, BLANC_CASSE, PLUIE, PLUIE_SOMBRE};
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use crate::monde::Monde;
use crate::rendu::couleur_mq;
use crate::scene::PAS_VAGUE;

/// Écart (px) entre deux stries d'une même ligne.
const ECART: i32 = 20;
/// Longueur d'une strie (px), tête comprise.
const LONGUEUR: i32 = 5;

/// Avance des stries (px) au pas `pas` : 45 px/s, plus vite que le courant ne
/// pousse la barque (24 px/s) : l'eau file sous elle, dans son sens.
fn avance(pas: u32) -> i32 {
    (pas.wrapping_mul(3) / 4) as i32
}

/// Les deux lignes de stries d'une case d'eau : `(dy, queue, tête)`, `dy` compté
/// depuis le haut de la case. La surface (case « haut ») a sa bande claire en haut.
fn lignes(surface: bool, vague: bool) -> [(i32, Couleur, Couleur); 2] {
    let decal = i32::from(surface && vague);
    if surface {
        [
            (3 + decal, PLUIE, BLANC_CASSE),
            (11 + decal, PLUIE_SOMBRE, PLUIE),
        ]
    } else {
        [(4, PLUIE_SOMBRE, PLUIE), (12, PLUIE_SOMBRE, PLUIE)]
    }
}

/// Décalage fixe d'une ligne de stries (rangée `ligne` du niveau, en px) : les
/// lignes voisines ne défilent pas en rangs d'oignons.
fn phase(ligne: i32) -> i32 {
    (ligne.wrapping_mul(7) + ligne / 16 * 5).rem_euclid(ECART)
}

/// Les pixels de stries à dessiner dans la case qui commence en `x0` (px, dans le
/// niveau) : `(x, tête ?)` pour la ligne `ligne`, le courant `sens` (-1 ou 1) et
/// l'avance `av`. Fonction pure : chaque pixel reste dans `[x0, x0 + 16)`.
fn pixels(x0: i32, ligne: i32, sens: i32, av: i32) -> Vec<(i32, bool)> {
    let t = TAILLE_TUILE as i32;
    let depart = (phase(ligne) + sens * av).rem_euclid(ECART);
    let mut v = Vec::new();
    // Première strie qui touche la case : la plus à gauche dont la fin atteint `x0`.
    let a = x0 - LONGUEUR + 1;
    let mut p = a + (depart - a).rem_euclid(ECART);
    while p < x0 + t {
        // La tête est à l'avant : à droite pour un courant vers la droite.
        let tete = if sens > 0 { p + LONGUEUR - 1 } else { p };
        v.extend(
            (p..p + LONGUEUR)
                .filter(|x| (x0..x0 + t).contains(x))
                .map(|x| (x, x == tete)),
        );
        p += ECART;
    }
    v
}

/// Dessine les stries de toutes les cases d'eau courante visibles (caméra `cx`, `cy`).
pub fn dessiner(monde: &Monde, cx: i32, cy: i32) {
    let t = TAILLE_TUILE as i32;
    let niveau = &monde.niveau;
    let (l, h) = (LARGEUR_LOGIQUE as i32, HAUTEUR_LOGIQUE as i32);
    let (t0, t1) = (cx.div_euclid(t), (cx + l).div_euclid(t));
    let (u0, u1) = (cy.div_euclid(t), (cy + h).div_euclid(t));
    let av = avance(monde.pas);
    for ty in u0..=u1 {
        for tx in t0..=t1 {
            if niveau.tuile_decor(tx, ty) != Tuile::Eau {
                continue;
            }
            let sens = i32::from(niveau.courant_eau(tx, ty));
            if sens == 0 {
                continue;
            }
            let surface = niveau.tuile_decor(tx, ty - 1) == Tuile::Vide;
            let vague = ((monde.pas / PAS_VAGUE) as i32 + tx).rem_euclid(2) == 1;
            for (dy, queue, tete) in lignes(surface, vague) {
                let y = ty * t + dy;
                for (x, est_tete) in pixels(tx * t, y, sens, av) {
                    let c = if est_tete { tete } else { queue };
                    draw_rectangle((x - cx) as f32, (y - cy) as f32, 1.0, 1.0, couleur_mq(c));
                }
            }
        }
    }
}
