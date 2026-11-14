//! Interface en jeu : gouttes d'or ramassées (en haut à gauche), chrono (en haut à droite)
//! et, dans l'arène du Grand Nuage, ses points de vie (en haut au milieu).

use pepin_core::entities::nuage::PV_NUAGE;
use pepin_core::level::GOUTTES_PAR_NIVEAU;
use pepin_core::levels::{gouttes_du_parcours, NIVEAUX};
use pepin_core::palette::{ARDOISE_OMBRE, BLANC_CASSE};
use pepin_core::LARGEUR_LOGIQUE;

use crate::chrono;
use crate::sprites::Banque;
use crate::texte::{self, Style};

/// Largeur (px) d'un point de vie du Grand Nuage dans le HUD, et écart entre deux.
const LARGEUR_PV: i32 = 10;
const ECART_PV: i32 = 3;

/// Points de vie du Grand Nuage : `(nom du sprite, x)` de chacun, de gauche à droite,
/// centrés en haut de l'écran : pleins d'abord (`pv` restants), puis perdus. Pure.
pub fn points_de_vie(pv: u8) -> Vec<(&'static str, i32)> {
    let n = i32::from(PV_NUAGE);
    let total = n * LARGEUR_PV + (n - 1) * ECART_PV;
    let x0 = (LARGEUR_LOGIQUE as i32 - total) / 2;
    (0..n)
        .map(|k| {
            let nom = if k < i32::from(pv) {
                "pv_nuage"
            } else {
                "pv_nuage_perdu"
            };
            (nom, x0 + k * (LARGEUR_PV + ECART_PV))
        })
        .collect()
}

/// Compteur de gouttes d'or du HUD, `(ramassées, total)` (carte 5.17). Dans le
/// niveau bonus : celles du bonus (`gouttes_niveau`), sur ses trois. Partout
/// ailleurs : celles du PARCOURS (tous les niveaux sauf le bonus, épilogue compris),
/// c'est-à-dire les gouttes de la partie moins celles du bonus s'il a été joué
/// (`gouttes_bonus`) ; c'est aussi le total de l'écran de fin. Pure.
pub fn compteur(
    dans_le_bonus: bool,
    gouttes_partie: u32,
    gouttes_bonus: Option<u32>,
    gouttes_niveau: u32,
) -> (u32, u32) {
    if dans_le_bonus {
        (gouttes_niveau, GOUTTES_PAR_NIVEAU as u32)
    } else {
        (
            gouttes_partie.saturating_sub(gouttes_bonus.unwrap_or(0)),
            gouttes_du_parcours(NIVEAUX),
        )
    }
}

/// Dessine le HUD. `(gouttes, total)` : le compteur de gouttes d'or (`compteur`) ;
/// `pas` : durée de la partie en pas ; `pv_nuage` : points de vie du Grand Nuage
/// s'il y en a un dans le niveau.
pub fn dessiner(banque: &Banque, (gouttes, total): (u32, u32), pas: u32, pv_nuage: Option<u8>) {
    let style = Style::simple(BLANC_CASSE, 1).avec_ombre(ARDOISE_OMBRE);
    // Sprite 10x10 (contour de 2 px) : décalé d'un pixel pour garder la goutte à la même place.
    banque.dessiner("goutte0", 5.0, 4.0, false);
    texte::dessiner(&format!("{gouttes}/{total}"), 17.0, 6.0, style);
    texte::dessiner_a_droite(
        &chrono::temps_court(pas),
        LARGEUR_LOGIQUE as f32 - 6.0,
        6.0,
        style,
    );
    if let Some(pv) = pv_nuage {
        for (nom, x) in points_de_vie(pv) {
            banque.dessiner(nom, x as f32, 6.0, false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le compteur : le parcours (30 : neuf niveaux normaux et l'épilogue), sans les
    /// gouttes du bonus ; dans le bonus, celles du bonus sur trois.
    #[test]
    fn le_compteur_compte_le_parcours_et_le_bonus_a_part() {
        // Au 9e niveau, toutes les gouttes : 27/30 (le bonus s'ouvre).
        assert_eq!(compteur(false, 27, None, 3), (27, 30));
        // Dans le bonus : ses gouttes, sur 3 (la partie en compte 27 + 2).
        assert_eq!(compteur(true, 29, None, 2), (2, 3));
        // Dans l'épilogue, après un bonus à 2 gouttes : 27 + 1 du parcours.
        assert_eq!(compteur(false, 30, Some(2), 1), (28, 30));
        // Tout : 30/30 (et le bonus, compté à part).
        assert_eq!(compteur(false, 33, Some(3), 3), (30, 30));
        // Bonus sauté : rien à retrancher.
        assert_eq!(compteur(false, 22, None, 0), (22, 30));
    }

    /// Trois points, centrés sur l'écran (320 px : de 142 à 178), pleins puis perdus.
    #[test]
    fn les_points_de_vie_du_nuage_sont_centres_pleins_puis_perdus() {
        assert_eq!(
            points_de_vie(3),
            vec![("pv_nuage", 142), ("pv_nuage", 155), ("pv_nuage", 168)]
        );
        assert_eq!(
            points_de_vie(1),
            vec![
                ("pv_nuage", 142),
                ("pv_nuage_perdu", 155),
                ("pv_nuage_perdu", 168)
            ]
        );
        assert!(points_de_vie(0).iter().all(|(n, _)| *n == "pv_nuage_perdu"));
        let (premier, dernier) = (points_de_vie(3)[0].1, points_de_vie(3)[2].1 + LARGEUR_PV);
        assert_eq!(premier + dernier, LARGEUR_LOGIQUE as i32, "centrés");
    }
}
