//! Les crochets du balancement (carte 11.1, prototype du Monde 3) : chaque
//! crochet `y` du niveau, éteint, ou ALLUMÉ quand la crosse de Pépin est à sa
//! portée (ou quand il y est suspendu) ; et la corde, du crochet à la crosse,
//! pendant le balancement. La physique est dans `pepin_core::player`
//! (sous-module `balancement`) : ici, seulement le dessin.
//!
//! La corde est une ligne de pixels pleins (algorithme de Bresenham : à chaque
//! pas, on avance d'un pixel sur l'axe le plus long et, quand l'erreur
//! accumulée le demande, d'un pixel sur l'autre), d'une couleur de la palette :
//! aucun trait lissé, aucune transparence.

use pepin_core::palette::{Couleur, ARDOISE_OMBRE};
use pepin_core::physics::JOUEUR_L;
use pepin_core::player::HAUTEUR_CROSSE;

use super::rect;
use crate::monde::Monde;
use crate::sprites::Banque;

/// Taille du dessin d'un crochet (px) : le point d'accroche est en son milieu.
const TAILLE_CROCHET: (i32, i32) = (10, 8);
/// Couleur de la corde : l'ombre de contour, lisible sur tous les ciels clairs.
const COULEUR_CORDE: Couleur = ARDOISE_OMBRE;

/// Les pixels d'une ligne de `a` à `b` (extrémités comprises), sans trou ni
/// doublon. Fonction pure (testée sans fenêtre).
pub(crate) fn pixels_de_ligne(a: (i32, i32), b: (i32, i32)) -> Vec<(i32, i32)> {
    let (dx, dy) = ((b.0 - a.0).abs(), -(b.1 - a.1).abs());
    let (sx, sy) = ((b.0 - a.0).signum(), (b.1 - a.1).signum());
    let (mut x, mut y, mut erreur) = (a.0, a.1, dx + dy);
    let mut pixels = Vec::with_capacity((dx.max(-dy) + 1) as usize);
    loop {
        pixels.push((x, y));
        if (x, y) == b {
            return pixels;
        }
        let double = 2 * erreur;
        if double >= dy {
            erreur += dy;
            x += sx;
        }
        if double <= dx {
            erreur += dx;
            y += sy;
        }
    }
}

/// Dessine la corde (derrière) puis les crochets. Rien dans un niveau sans crochet.
pub(super) fn dessiner_crochets(monde: &Monde, banque: &Banque, alpha: f32, cx: i32, cy: i32) {
    let crochets = &monde.niveau.crochets;
    if crochets.is_empty() {
        return;
    }
    let j = &monde.joueur;
    let suspendu = j.balancement();
    if let Some(b) = suspendu {
        // La crosse à la position DESSINÉE de Pépin (interpolée, arrondie).
        let (px, py) = monde.joueur_interpole(alpha);
        let crosse = (
            (px + JOUEUR_L / 2.0).round() as i32,
            (py - HAUTEUR_CROSSE).round() as i32,
        );
        let point = (b.point.0.round() as i32, b.point.1.round() as i32);
        for (x, y) in pixels_de_ligne(point, crosse) {
            rect(x - cx, y - cy, 1, 1, COULEUR_CORDE);
        }
    }
    let allume = suspendu
        .map(|b| b.crochet)
        .or_else(|| j.crochet_a_portee(crochets));
    for (i, c) in crochets.iter().enumerate() {
        let nom = if allume == Some(i) {
            "crochet_allume"
        } else {
            "crochet"
        };
        let (x, y) = (
            c.x.round() as i32 - TAILLE_CROCHET.0 / 2,
            c.y.round() as i32 - TAILLE_CROCHET.1 / 2,
        );
        banque.dessiner(nom, (x - cx) as f32, (y - cy) as f32, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn une_ligne_va_d_un_bout_a_l_autre_sans_trou() {
        for b in [(0, 0), (5, 0), (0, -7), (12, 30), (-9, 4), (-3, -3)] {
            let p = pixels_de_ligne((0, 0), b);
            assert_eq!(p.first(), Some(&(0, 0)));
            assert_eq!(p.last(), Some(&b));
            // Autant de pixels que la plus grande des deux distances, plus un.
            let n = (b.0.abs().max(b.1.abs()) + 1) as usize;
            assert_eq!(p.len(), n, "{b:?}");
            // Deux pixels voisins se touchent (par un côté ou un coin).
            for w in p.windows(2) {
                let (d0, d1) = ((w[1].0 - w[0].0).abs(), (w[1].1 - w[0].1).abs());
                assert!(d0 <= 1 && d1 <= 1 && (d0, d1) != (0, 0), "{b:?}");
            }
        }
    }
}
