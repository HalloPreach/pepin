//! La MER de la montée de la Pieuvre, toujours sensible à l'écran (carte 9.13e).
//! AGA : « On ne voit pas la mer. Sur les 4 captures il n'y a pas d'eau à l'écran,
//! et la géante flotte dans le noir » (la géante de l'époque, remplacée par la
//! Pieuvre au loin à la carte 9.36a). Ce qui presse le joueur est hors champ : il
//! faut de l'écume en bas de l'écran ou une jauge. »
//!
//! Rendu seulement : la mer elle-même (sa hauteur, ce qui tue) ne change pas.
//! - La caméra garde la surface proche dans le cadre, au bas de l'écran
//!   (`camera::cadrer_la_mer`, alimentée par `surface`).
//! - Surface à l'écran : une CRÊTE d'écume et des EMBRUNS, dessinés DEVANT la
//!   Pieuvre au loin (`dessiner_devant`, juste après elle, carte 9.36a) : elle
//!   est à moitié dans l'eau, qui bouillonne plus fort autour d'elle. Carte
//!   9.36b : la crête est un sprite (`pieuvre_crete0/1`) et les remous de la
//!   phase 1 bouillonnent autour d'elle ; la masse d'eau elle-même est dans
//!   `dessin_montee::dessiner_mer`.
//! - Surface sous l'écran : une BANDE D'ÉCUME qui bouillonne au bas de l'écran,
//!   plus haute et plus agitée quand la mer se rapproche (`hauteur_bande`).
//! - La JAUGE, sur le mur gauche (`dessiner_jauge`) : en haut le sommet (une
//!   petite tête rouge), les lanternes de la tour, Pépin (la flèche blanche, qui
//!   clignote en corail quand la mer est à moins de deux tuiles), et la mer qui
//!   monte par le bas.
//!
//! Couleurs de la palette seulement, sprites et rectangles au pixel entier : aucun
//! flou.
//!
//! Notion Rust : `Option::filter` et `and_then` enchaînés pour dire « la pieuvre,
//! si elle est en montée, puis sa mer, si elle en a une » sans `if` imbriqués.

use macroquad::prelude::draw_rectangle;
use pepin_core::aabb::TAILLE_TUILE;
use pepin_core::entities::pieuvre::{PhasePieuvre, Pieuvre};
use pepin_core::entities::Entite;
use pepin_core::palette::{
    Couleur, ARDOISE, ARDOISE_NUIT, ARDOISE_OMBRE, ARDOISE_PALE, BLANC_CASSE, CORAIL, NUAGE,
    OR_PALE, PLUIE, PLUIE_SOMBRE,
};
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use crate::dessin_pieuvre::la_pieuvre;
use crate::monde::Monde;
use crate::rendu::couleur_mq;
use crate::sprites::Banque;

const L: i32 = LARGEUR_LOGIQUE as i32;
const H: i32 = HAUTEUR_LOGIQUE as i32;

/// Au-delà de cette distance sous le bas de l'écran (px, 24 tuiles), la bande
/// d'écume reste à sa plus petite hauteur.
const LOIN: f32 = 24.0 * TAILLE_TUILE;
/// Hauteur de la bande d'écume (px) : de `BANDE_MIN` (mer loin) à `BANDE_MAX`
/// (mer juste sous l'écran).
const BANDE_MIN: i32 = 5;
const BANDE_MAX: i32 = 16;
/// Le rail de la jauge : abscisse de son intérieur, haut et bas (px, à l'écran),
/// sur le mur gauche de la tour, sous le compteur de gouttes.
const JAUGE_X: i32 = 5;
const JAUGE_HAUT: i32 = 32;
const JAUGE_BAS: i32 = 164;

fn rect(x: i32, y: i32, w: i32, h: i32, c: Couleur) {
    if w > 0 && h > 0 {
        draw_rectangle(x as f32, y as f32, w as f32, h as f32, couleur_mq(c));
    }
}

/// Nombre pseudo-aléatoire stable dérivé d'un entier (même entrée, même sortie).
fn hache(n: i32) -> u32 {
    let mut h = (n as u32).wrapping_mul(0x9E37_79B1);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h
}

/// La pieuvre de la phase 2, tant que la mer monte (furie ou montée).
fn en_montee(monde: &Monde) -> Option<&Pieuvre> {
    la_pieuvre(monde)
        .filter(|p| p.en_fuite() && matches!(p.phase(), PhasePieuvre::Furie | PhasePieuvre::Montee))
}

/// La surface de la mer qui monte (px du niveau), pendant la montée seulement :
/// la caméra la garde dans le cadre quand elle est proche (`Camera::mer`).
pub fn surface(monde: &Monde) -> Option<f32> {
    en_montee(monde)
        .filter(|p| p.phase() == PhasePieuvre::Montee)
        .and_then(Pieuvre::surface_mer)
}

/// Proximité de la mer sous le bas de l'écran (`d` px plus bas) : 1 au bord de
/// l'écran, 0 à `LOIN` ou plus. Pure.
pub fn proximite(d: f32) -> f32 {
    (1.0 - d / LOIN).clamp(0.0, 1.0)
}

/// Hauteur (px) de la bande d'écume quand la mer est `d` px sous l'écran. Pure.
pub fn hauteur_bande(d: f32) -> i32 {
    BANDE_MIN + ((BANDE_MAX - BANDE_MIN) as f32 * proximite(d)).round() as i32
}

/// Devant la Pieuvre au loin et les tuiles, derrière Pépin : la crête et les embruns si la
/// surface est à l'écran, sinon (pendant la montée) la bande d'écume du bas de
/// l'écran. Carte 9.36b : la crête est faite de sprites (`pieuvre_crete0/1`) et se
/// dessine aussi pendant la fuite (la mer redescend, la Pieuvre plonge).
pub fn dessiner_devant(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    let Some(p) = la_pieuvre(monde).filter(|p| p.en_fuite()) else {
        return;
    };
    let Some(mer) = p.surface_mer() else {
        return;
    };
    let montee = en_montee(monde).is_some();
    let surface = mer.round() as i32 - cy;
    if surface >= H - 1 {
        if montee {
            dessiner_bande(banque, mer - (cy + H) as f32, monde.pas);
        }
        return;
    }
    // Carte 9.36a : l'eau bouillonne autour de la Pieuvre au loin.
    let couloir = p
        .au_loin()
        .filter(|_| montee)
        .map(|b| (b.x.round() as i32 - cx, b.droite().round() as i32 - cx));
    let largeur = monde.niveau.largeur_px() as i32;
    dessiner_crete(
        banque,
        surface,
        (TAILLE_TUILE as i32 - cx, largeur - 16 - cx),
        couloir,
        monde.pas,
    );
}

/// La crête d'écume sur la surface (à l'écran, `surface`), de `bords.0` à
/// `bords.1` : le sprite `pieuvre_crete0/1` (64 px, ses trois dernières rangées
/// sur l'eau) répété, et les embruns ; autour de la Pieuvre au loin (`couloir`,
/// carte 9.36a), l'eau bouillonne (les remous de la phase 1) et éclabousse plus
/// haut (elle y est à moitié immergée).
fn dessiner_crete(
    banque: &Banque,
    surface: i32,
    (x0, x1): (i32, i32),
    couloir: Option<(i32, i32)>,
    pas: u32,
) {
    let nom = if (pas / 10).is_multiple_of(2) {
        "pieuvre_crete0"
    } else {
        "pieuvre_crete1"
    };
    let mut x = x0;
    while x < x1 {
        let l = (x1 - x).min(64) as f32;
        let ecran = (x as f32, (surface - 3) as f32);
        banque.dessiner_partie(nom, ecran, (0.0, 0.0, l, 6.0), 0, false);
        x += 64;
    }
    if let Some((g, d)) = couloir {
        let remous = if (pas / 6).is_multiple_of(2) {
            "remous_pieuvre0"
        } else {
            "remous_pieuvre1"
        };
        for rx in [g - 18, g + 8, d - 24] {
            banque.dessiner(remous, rx as f32, (surface - 4) as f32, false);
        }
    }
    // Les embruns : des gouttes qui jaillissent et retombent (une parabole),
    // nombreuses dans le couloir, quelques-unes le long de la tour.
    for k in 0..14i32 {
        let h = hache(k * 31 + 7);
        let x = match couloir {
            Some((g, d)) if k < 9 => g + (h % (d - g).max(1) as u32) as i32,
            _ => x0 + (h % (x1 - x0).max(1) as u32) as i32,
        };
        let vie = 36 + (h >> 8) as i32 % 20;
        let age = (pas as i32 + (h >> 4) as i32).rem_euclid(vie + 30);
        if age >= vie {
            continue;
        }
        // Montée puis chute : hauteur max 10 px (couloir) ou 6 px.
        let sommet = if k < 9 { 10 } else { 6 };
        let y = 4 * sommet * age * (vie - age) / (vie * vie);
        let dx = (age / 6) * if h.is_multiple_of(2) { 1 } else { -1 };
        let c = if age < vie / 2 { BLANC_CASSE } else { NUAGE };
        rect(x + dx, surface - 2 - y, 1 + (h % 2) as i32, 1, c);
    }
}

/// La bande d'écume qui bouillonne au bas de l'écran, la mer étant `d` px plus
/// bas : plus haute et plus agitée (houle plus ample, bulles plus nombreuses et
/// plus rapides) quand elle se rapproche. Carte 9.36b : c'est le HAUT de la vraie
/// mer (la crête `pieuvre_crete0/1` et les premières rangées de `pieuvre_mer0/1`)
/// qui dépasse du bas de l'écran, plus une frise calculée.
fn dessiner_bande(banque: &Banque, d: f32, pas: u32) {
    let k = proximite(d);
    let lenteur = (8.0 - 5.0 * k).round() as u32;
    let t = (pas / lenteur) as i32;
    let ample = 1 + (2.0 * k).round() as i32;
    // La houle soulève toute la bande d'un ou deux pixels.
    let houle = [0, 1, 2, 2, 1, 0, -1, -1][(t as usize / 4) % 8] * ample / 2;
    let haut = (hauteur_bande(d) + houle).max(2);
    let mer = if (pas / 20).is_multiple_of(2) {
        "pieuvre_mer0"
    } else {
        "pieuvre_mer1"
    };
    let crete = if (pas / 10).is_multiple_of(2) {
        "pieuvre_crete0"
    } else {
        "pieuvre_crete1"
    };
    for x in (0..L).step_by(64) {
        let y = (H - haut) as f32;
        banque.dessiner_partie(mer, (x as f32, y), (0.0, 0.0, 64.0, haut as f32), 0, false);
        banque.dessiner(crete, x as f32, y - 3.0, false);
    }
    // Les bulles qui montent de l'écume et éclatent.
    let n = 6 + (14.0 * k).round() as i32;
    for i in 0..n {
        let h = hache(i * 17 + 3);
        let x = (h % L as u32) as i32;
        let vie = 24 + (h >> 10) as i32 % 16;
        let age = (pas as i32 * (1 + ample) / 2 + (h >> 3) as i32).rem_euclid(vie);
        let y = H - haut - 4 - age * (2 + ample) / 8;
        let c = if age + 3 >= vie { BLANC_CASSE } else { NUAGE };
        rect(x, y, 1 + (age + 3 >= vie) as i32, 1, c);
    }
}

/// La jauge de la montée, sur le mur gauche, en coordonnées d'écran : le sommet
/// (le sommet), les lanternes de la tour, Pépin et la mer.
pub fn dessiner_jauge(monde: &Monde) {
    let Some(p) = en_montee(monde).filter(|p| p.phase() == PhasePieuvre::Montee) else {
        return;
    };
    let (Some(mer), Some(sommet), Some(bache)) = (p.surface_mer(), p.sommet, p.bache) else {
        return;
    };
    let (haut, bas) = (sommet.bas(), bache.y);
    let vers = |y: f32| {
        let part = ((y - haut) / (bas - haut)).clamp(0.0, 1.0);
        JAUGE_HAUT + (part * (JAUGE_BAS - JAUGE_HAUT) as f32).round() as i32
    };
    let (x, h) = (JAUGE_X, JAUGE_BAS - JAUGE_HAUT);
    // Le cadre : ardoise d'ombre, un liseré d'ardoise, l'intérieur de nuit.
    rect(x - 3, JAUGE_HAUT - 3, 9, h + 7, ARDOISE_OMBRE);
    rect(x - 1, JAUGE_HAUT - 1, 5, h + 3, ARDOISE);
    rect(x, JAUGE_HAUT, 3, h + 1, ARDOISE_NUIT);
    // La mer, par le bas : l'eau, sa surface claire.
    let ym = vers(mer);
    rect(x, ym, 3, JAUGE_BAS + 1 - ym, PLUIE_SOMBRE);
    rect(x, ym, 3, 1, PLUIE);
    // Les lanternes de la tour : un cran à gauche du rail, or pâle si allumée.
    for e in &monde.entites {
        if let Entite::Lanterne(l) = e {
            if l.boite.y < bache.y {
                let c = if l.est_active() {
                    OR_PALE
                } else {
                    ARDOISE_PALE
                };
                rect(x - 3, vers(l.boite.bas()), 3, 1, c);
            }
        }
    }
    // Le sommet (carte 9.36a : là où elle abandonne) : la petite tête rouge.
    rect(x - 2, JAUGE_HAUT - 6, 7, 4, ARDOISE_OMBRE);
    rect(x - 1, JAUGE_HAUT - 5, 5, 3, CORAIL);
    rect(x + 1, JAUGE_HAUT - 4, 1, 1, BLANC_CASSE);
    // Pépin : une flèche blanche à droite du rail, pointée vers lui ; en corail,
    // clignotante, quand la mer est à moins de deux tuiles de ses pieds.
    let pieds = monde.joueur.boite.bas();
    let yp = vers(pieds);
    let danger = mer - pieds < 2.0 * TAILLE_TUILE;
    let c = if danger && (monde.pas / 6).is_multiple_of(2) {
        CORAIL
    } else {
        BLANC_CASSE
    };
    rect(x + 3, yp - 2, 6, 5, ARDOISE_OMBRE);
    rect(x + 5, yp - 1, 3, 3, c);
    rect(x + 4, yp, 1, 1, c);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// La bande d'écume grandit quand la mer se rapproche, sans saut : de
    /// `BANDE_MIN` (loin) à `BANDE_MAX` (au bord de l'écran), d'un pixel au plus
    /// pour une tuile de mer en plus.
    #[test]
    fn la_bande_d_ecume_grandit_quand_la_mer_approche() {
        assert_eq!(hauteur_bande(0.0), BANDE_MAX);
        assert_eq!(hauteur_bande(LOIN), BANDE_MIN);
        assert_eq!(hauteur_bande(10.0 * LOIN), BANDE_MIN);
        let mut avant = hauteur_bande(0.0);
        for d in (0..400).map(|d| d as f32) {
            let h = hauteur_bande(d);
            assert!(h <= avant && avant - h <= 1, "{d} : {avant} puis {h}");
            avant = h;
        }
        // La jauge tient sur le mur gauche, sous le compteur de gouttes.
        const { assert!(JAUGE_X + 9 <= 16 && JAUGE_HAUT >= 24 && JAUGE_BAS < H - 8) };
    }
}
