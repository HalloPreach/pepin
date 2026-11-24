//! Repères d'interface dessinés DANS la scène : la flèche qui montre Pépin quand il
//! est au-dessus de l'écran. Les compteurs et le chrono sont dans `crate::hud`, les
//! menus dans `crate::ecrans`.

use pepin_core::palette::*;
use pepin_core::LARGEUR_LOGIQUE;

use super::rect;
use crate::monde::Monde;

/// Taille (largeur, hauteur) du repère « Pépin est au-dessus de l'écran ».
const REPERE: (i32, i32) = (5, 3);

/// Où poser le repère en haut de l'écran ? `boite_y`, `boite_h` : ordonnée écran du haut de la
/// boîte de Pépin et sa hauteur ; `milieu_x` : abscisse écran de son milieu. Renvoie l'abscisse
/// écran (bord gauche) du repère, bornée pour qu'il reste entier dans l'écran, ou `None` tant
/// que Pépin est visible (au moins une ligne de sa boîte dans le cadre). Fonction pure.
pub(crate) fn repere_haut(boite_y: i32, boite_h: i32, milieu_x: i32) -> Option<i32> {
    if boite_y + boite_h > 0 {
        return None;
    }
    let max = LARGEUR_LOGIQUE as i32 - REPERE.0;
    Some((milieu_x - REPERE.0 / 2).clamp(0, max))
}

/// Petite flèche corail collée en haut de l'écran, à l'abscisse de Pépin, quand il saute
/// au-dessus du cadre : au-dessus d'un niveau d'un écran (caméra verticale figée), ou
/// au-dessus du HAUT d'un niveau haut (la caméra s'arrête à sa borne haute).
fn dessiner_repere(x: i32) {
    const FLECHE: [(i32, i32); 3] = [(2, 1), (1, 3), (0, 5)]; // (décalage, largeur) par rangée
    for (k, &(dx, w)) in FLECHE.iter().enumerate() {
        rect(x + dx, k as i32, w, 1, CORAIL);
    }
}

/// Flèche de repère par-dessus tout le monde (pluie comprise) quand Pépin est au-dessus du cadre.
pub(super) fn dessiner_repere_pepin(monde: &Monde, alpha: f32, cx: i32, cy: i32) {
    let j = &monde.joueur;
    let (px, py) = monde.joueur_interpole(alpha);
    let (bx, by) = (px.round() as i32 - cx, py.round() as i32 - cy);
    if let Some(x) = repere_haut(by, j.boite.h as i32, bx + j.boite.w as i32 / 2) {
        dessiner_repere(x);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_repere_n_apparait_que_pepin_hors_du_cadre() {
        // Boîte de 14 px de haut : visible tant que sa dernière ligne est dans l'écran.
        assert_eq!(repere_haut(100, 14, 160), None);
        assert_eq!(repere_haut(-13, 14, 160), None, "une ligne encore visible");
        assert!(repere_haut(-14, 14, 160).is_some(), "entièrement au-dessus");
        assert!(repere_haut(-60, 14, 160).is_some());
    }

    #[test]
    fn le_repere_est_centre_sur_pepin_et_borne_a_l_ecran() {
        assert_eq!(repere_haut(-20, 14, 160), Some(158));
        assert_eq!(repere_haut(-20, 14, 0), Some(0));
        assert_eq!(repere_haut(-20, 14, -50), Some(0));
        assert_eq!(repere_haut(-20, 14, 319), Some(315));
        assert_eq!(repere_haut(-20, 14, 900), Some(315));
    }
}
