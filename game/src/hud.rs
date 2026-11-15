//! Interface en jeu : gouttes d'or ramassées et vies (en haut à gauche), chrono (en
//! haut à droite) et, dans l'arène du Grand Nuage, ses points de vie (en haut au
//! milieu).

use pepin_core::entities::nuage::PV_NUAGE;
use pepin_core::font::largeur_texte;
use pepin_core::level::GOUTTES_PAR_NIVEAU;
use pepin_core::levels::{gouttes_du_parcours, NIVEAUX};
use pepin_core::palette::{ARDOISE_OMBRE, BLANC_CASSE, OR};
use pepin_core::LARGEUR_LOGIQUE;

use crate::chrono;
use crate::sprites::Banque;
use crate::texte::{self, Style};

/// Largeur (px) d'un point de vie du Grand Nuage dans le HUD, et écart entre deux.
const LARGEUR_PV: i32 = 10;
const ECART_PV: i32 = 3;

/// Abscisse du texte du compteur de gouttes (après l'icône de la goutte).
const X_GOUTTES: f32 = 17.0;
/// Abscisse de l'icône des vies (10 px de large) et du chiffre qui la suit. Fixe :
/// le compteur ne bouge pas quand le nombre de gouttes passe de 9 à 10. Le texte des
/// gouttes le plus large, « 30/30 », finit en x = 46 (test
/// `les_vies_ne_chevauchent_ni_les_gouttes_ni_le_nuage`).
const X_ICONE_VIE: f32 = 53.0;
const X_CHIFFRE_VIE: f32 = 65.0;
// Le chiffre suit l'icône (10 px) à 2 px (vérifié à la compilation).
const _: () = assert!(X_CHIFFRE_VIE >= X_ICONE_VIE + 10.0 + 2.0);
/// Durée (pas) du petit signe au gain d'une vie : 1 s.
pub const DUREE_ECLAT_VIE: u16 = 60;

/// Le compteur de vies tel qu'il est dessiné : le chiffre, sa couleur, et le « +1 »
/// d'une vie qui vient d'être gagnée (`None` sinon), avec son abscisse et son
/// ordonnée (il monte de 3 px pendant la première moitié de l'éclat).
#[derive(Debug, Clone, PartialEq)]
pub struct AffichageVies {
    pub chiffre: String,
    pub couleur: pepin_core::palette::Couleur,
    pub plus_un: Option<(f32, f32)>,
}

/// Le compteur de vies pour `restantes` vies ; `eclat` : pas restants du petit signe
/// d'une vie gagnée (0 : aucun). Pendant l'éclat, le chiffre passe en or et un « +1 »
/// doré monte à côté. Pure.
pub fn vies(restantes: u8, eclat: u16) -> AffichageVies {
    let chiffre = restantes.to_string();
    if eclat == 0 {
        return AffichageVies {
            chiffre,
            couleur: BLANC_CASSE,
            plus_un: None,
        };
    }
    let x = X_CHIFFRE_VIE + largeur_texte(&chiffre, 1) as f32 + 3.0;
    // Écoulé : de 0 à DUREE_ECLAT_VIE ; il monte de 3 px pendant la première
    // moitié, puis reste en place.
    let ecoule = DUREE_ECLAT_VIE.saturating_sub(eclat);
    let montee = (u32::from(ecoule) * 6 / u32::from(DUREE_ECLAT_VIE)).min(3) as f32;
    AffichageVies {
        chiffre,
        couleur: OR,
        plus_un: Some((x, 6.0 - montee)),
    }
}

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
/// `vies` : le compteur de vies (`vies`) ; `pas` : durée de la partie en pas ;
/// `pv_nuage` : points de vie du Grand Nuage s'il y en a un dans le niveau.
pub fn dessiner(
    banque: &Banque,
    (gouttes, total): (u32, u32),
    vies: &AffichageVies,
    pas: u32,
    pv_nuage: Option<u8>,
) {
    let style = Style::simple(BLANC_CASSE, 1).avec_ombre(ARDOISE_OMBRE);
    // Sprite 10x10 (contour de 2 px) : décalé d'un pixel pour garder la goutte à la même place.
    banque.dessiner("goutte0", 5.0, 4.0, false);
    texte::dessiner(&format!("{gouttes}/{total}"), X_GOUTTES, 6.0, style);
    banque.dessiner("vie", X_ICONE_VIE, 4.0, false);
    texte::dessiner(
        &vies.chiffre,
        X_CHIFFRE_VIE,
        6.0,
        Style::simple(vies.couleur, 1).avec_ombre(ARDOISE_OMBRE),
    );
    if let Some((x, y)) = vies.plus_un {
        texte::dessiner("+1", x, y, Style::simple(OR, 1).avec_ombre(ARDOISE_OMBRE));
    }
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

    /// Le compteur de vies (carte 6.2) tient entre le compteur de gouttes le plus
    /// large (« 30/30 », et « 3/3 » dans le bonus) et les points de vie du Grand
    /// Nuage, « +1 » compris, avec au moins 4 px d'écart (ombres comprises), et ne
    /// bouge pas d'un pixel quand les gouttes changent.
    #[test]
    fn les_vies_ne_chevauchent_ni_les_gouttes_ni_le_nuage() {
        let fin_gouttes = X_GOUTTES + largeur_texte("30/30", 1) as f32 + 1.0;
        assert!(X_ICONE_VIE >= fin_gouttes + 4.0, "{fin_gouttes}");
        let debut_nuage = points_de_vie(3)[0].1 as f32;
        for n in 0..=pepin_core::vies::VIES_MAX {
            for eclat in [0, 1, DUREE_ECLAT_VIE / 2, DUREE_ECLAT_VIE] {
                let a = vies(n, eclat);
                let mut fin = X_CHIFFRE_VIE + largeur_texte(&a.chiffre, 1) as f32 + 1.0;
                if let Some((x, y)) = a.plus_un {
                    assert!(x >= fin + 2.0, "« +1 » après le chiffre");
                    assert!((3.0..=6.0).contains(&y), "{y}");
                    fin = x + largeur_texte("+1", 1) as f32 + 1.0;
                }
                assert!(fin + 4.0 <= debut_nuage, "{n} vies, éclat {eclat} : {fin}");
            }
        }
        // Un seul chiffre jusqu'au plafond.
        assert_eq!(vies(pepin_core::vies::VIES_MAX, 0).chiffre.len(), 1);
    }

    /// Le petit signe d'une vie gagnée : chiffre en or et « +1 » qui monte de 3 px
    /// pendant la première demi-seconde ; rien sans éclat.
    #[test]
    fn une_vie_gagnee_se_voit_une_seconde() {
        let calme = vies(4, 0);
        assert_eq!(
            (calme.chiffre.as_str(), calme.couleur, calme.plus_un),
            ("4", BLANC_CASSE, None)
        );
        let debut = vies(5, DUREE_ECLAT_VIE);
        assert_eq!(debut.couleur, OR);
        assert_eq!(debut.plus_un.unwrap().1, 6.0);
        assert_eq!(
            vies(5, DUREE_ECLAT_VIE / 2).plus_un.unwrap().1,
            3.0,
            "en haut"
        );
        assert_eq!(vies(5, 1).plus_un.unwrap().1, 3.0, "il y reste");
    }
}
