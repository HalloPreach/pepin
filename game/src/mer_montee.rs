//! La MER de la montée de la Pieuvre, toujours sensible à l'écran (carte 9.13e).
//! AGA : « On ne voit pas la mer. Sur les 4 captures il n'y a pas d'eau à l'écran,
//! et la géante flotte dans le noir. Ce qui presse le joueur est hors champ : il
//! faut de l'écume en bas de l'écran ou une jauge. »
//!
//! Rendu seulement : la mer elle-même (sa hauteur, ce qui tue) ne change pas.
//! - La caméra garde la surface proche dans le cadre, au bas de l'écran
//!   (`camera::cadrer_la_mer`, alimentée par `surface`).
//! - Surface à l'écran : une CRÊTE d'écume et des EMBRUNS, dessinés DEVANT la
//!   géante (`dessiner_devant`, juste après elle) : son corps plonge dans l'eau.
//!   Autour d'elle, dans son couloir, l'eau bouillonne plus fort.
//! - Surface sous l'écran : une BANDE D'ÉCUME qui bouillonne au bas de l'écran,
//!   plus haute et plus agitée quand la mer se rapproche (`hauteur_bande`) ; le
//!   cou de la géante, quand elle grimpe, y plonge.
//! - La JAUGE, sur le mur gauche (`dessiner_jauge`) : en haut la vigie (la tête
//!   rouge de la géante), les lanternes de la tour, Pépin (la flèche blanche, qui
//!   clignote en corail quand la mer est à moins de deux tuiles), et la mer qui
//!   monte par le bas.
//!
//! Couleurs de la palette seulement, rectangles au pixel entier : aucun flou.
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

/// Devant la géante et les tuiles, derrière Pépin : la crête et les embruns si la
/// surface est à l'écran, sinon la bande d'écume du bas de l'écran.
pub fn dessiner_devant(monde: &Monde, cx: i32, cy: i32) {
    let Some(p) = en_montee(monde) else {
        return;
    };
    let Some(mer) = p.surface_mer() else {
        return;
    };
    let surface = mer.round() as i32 - cy;
    if surface >= H - 1 {
        dessiner_bande(mer - (cy + H) as f32, monde.pas);
    } else {
        let couloir = p
            .tour
            .couloir
            .map(|(g, d)| (g.round() as i32 - cx, d.round() as i32 - cx));
        let largeur = monde.niveau.largeur_px() as i32;
        dessiner_crete(
            surface,
            (TAILLE_TUILE as i32 - cx, largeur - 16 - cx),
            couloir,
            monde.pas,
        );
    }
}

/// La crête d'écume sur la surface (à l'écran, `surface`), de `bords.0` à
/// `bords.1`, et les embruns ; dans le couloir de la géante, l'eau bouillonne et
/// éclabousse plus haut (son corps y entre dans la mer).
fn dessiner_crete(surface: i32, (x0, x1): (i32, i32), couloir: Option<(i32, i32)>, pas: u32) {
    let dans_le_couloir = |x: i32| couloir.is_some_and(|(g, d)| x >= g && x < d);
    let t = (pas / 6) as i32;
    for x in (x0.max(0)..x1.min(L)).step_by(2) {
        let fort = dans_le_couloir(x);
        let vague = [0, 1, 1, 0, 0, -1][((x / 2 + t) as usize) % 6];
        let haut = if fort {
            2 + vague + (hache(x / 2 + t) % 2) as i32
        } else {
            1 + vague
        };
        rect(x, surface - haut, 2, haut + 2, NUAGE);
        rect(x, surface - haut, 2, 1, BLANC_CASSE);
        rect(x, surface + 2, 2, 1, PLUIE);
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
/// plus rapides) quand elle se rapproche.
fn dessiner_bande(d: f32, pas: u32) {
    let k = proximite(d);
    let haut = hauteur_bande(d);
    let lenteur = (8.0 - 5.0 * k).round() as u32;
    let t = (pas / lenteur) as i32;
    let ample = 1 + (2.0 * k).round() as i32;
    for x in (0..L).step_by(2) {
        let houle = [0, 1, 2, 2, 1, 0, -1, -1][((x / 2 + t) as usize) % 8];
        let bouillon = (hache(x / 2 * 3 + t) % 2) as i32;
        let h = (haut + houle * ample / 2 + bouillon).max(2);
        let y = H - h;
        rect(x, y, 2, h, ARDOISE_NUIT);
        rect(x, y + 1, 2, 2.min(h - 1), PLUIE_SOMBRE);
        rect(x, y, 2, 1, if bouillon == 1 { BLANC_CASSE } else { NUAGE });
    }
    // Les bulles qui montent de l'écume et éclatent.
    let n = 6 + (14.0 * k).round() as i32;
    for i in 0..n {
        let h = hache(i * 17 + 3);
        let x = (h % L as u32) as i32;
        let vie = 24 + (h >> 10) as i32 % 16;
        let age = (pas as i32 * (1 + ample) / 2 + (h >> 3) as i32).rem_euclid(vie);
        let y = H - haut - 1 - age * (2 + ample) / 8;
        let c = if age + 3 >= vie { BLANC_CASSE } else { NUAGE };
        rect(x, y, 1 + (age + 3 >= vie) as i32, 1, c);
    }
}

/// La jauge de la montée, sur le mur gauche, en coordonnées d'écran : le sommet
/// (la vigie), les lanternes de la tour, Pépin et la mer.
pub fn dessiner_jauge(monde: &Monde) {
    let Some(p) = en_montee(monde).filter(|p| p.phase() == PhasePieuvre::Montee) else {
        return;
    };
    let (Some(mer), Some(sommet), Some(bache)) = (p.surface_mer(), p.tete_finale, p.bache) else {
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
    // Le sommet : la tête rouge de la géante, son œil blanc.
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
