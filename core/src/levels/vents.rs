//! Niveau 7 : « Vents d'altitude ».
//!
//! Colonnes de vent à chevaucher et longs planés entre îlots de nuages ; au
//! milieu de l'escalier de nuages, un îlot friable ; deux corbeaux patrouillent
//! (sur l'îlot d'arrivée du 2e courant et avant le drapeau, `PORTEE_CORBEAU`).
//! Au-dessus du dernier toit, un pigeon guette sur un petit nuage (colonne 167,
//! 4 rangées plus haut) : tourné vers l'arrivée de Pépin, il plonge derrière
//! lui ; il ne touche que celui qui s'arrête ou revient sous lui.
//!
//! Carte 6.4b (« plus d'ennemis ») : deux corbeaux et un pigeon de plus.
//! - Un corbeau au-dessus des nuages friables (colonne 76, rangée 3 : il
//!   patrouille des colonnes 71 à 81), juste au-dessus de la tête de Pépin posé
//!   sur les friables (colonnes 72 à 75) : on passe dessous en marchant, mais il
//!   faut sauter pour en repartir, avant qu'ils ne cèdent, et un saut au mauvais
//!   moment s'y cogne.
//! - Un corbeau à hauteur de Pépin sur l'îlot des colonnes 143 à 151 (colonne 147,
//!   rangée 3 : il patrouille des colonnes 142 à 152, au-delà des deux bords) : on
//!   se laisse tomber dessus depuis l'îlot de la lanterne 136 (il s'écrase), ou on
//!   le saute ; la lanterne 136 est hors de sa patrouille.
//! - Un second pigeon, sur un petit nuage posé pour lui (colonne 38, rangée 4),
//!   4 rangées au-dessus de l'îlot de la lanterne 43, comme celui du dernier toit :
//!   tourné vers l'arrivée de Pépin, qui s'y pose en descendant de l'îlot du
//!   premier courant, il plonge derrière lui (sa plongée finit au ras de l'îlot,
//!   colonne 34, loin de la lanterne).
//!
//! Carte 6.8 (retour d'AGA en jouant : « un courant ascendant qui finit un peu
//! trop tôt et rend le saut compliqué ») : chaque courant lâche maintenant Pépin
//! au moins 32 px (deux tuiles) au-dessus du dessus de son îlot d'arrivée, comme
//! les deux premiers (test `chaque_courant_lache_pepin_au_moins_32_px_au_dessus_de_son_ilot`).
//! - 3e courant (colonnes 97 à 99) : il commençait à la rangée 3, AU NIVEAU de
//!   l'îlot des colonnes 103 à 108 (0 px de marge, trois tuiles de vide à
//!   planer) ; prolongé vers le haut, il commence à la rangée 1.
//! - 4e courant (colonnes 127 à 129) : il commence déjà à la rangée 1 et la
//!   rangée 0 est le haut de l'écran (en le prolongeant, Pépin planerait hors du
//!   cadre). C'est donc son îlot d'arrivée (colonnes 132 à 141) qui descend d'une
//!   rangée (2 → 3), avec sa lanterne (colonne 136, rangée 1 → 2) : 16 → 32 px.
//!   Il est maintenant à la hauteur du corbeau (147, 3), dont la patrouille
//!   commence 2 px après son bord droit : Pépin posé au bord ne le touche pas
//!   (la boîte du corbeau commence 3 px sous le dessus de l'îlot) et, en se
//!   laissant tomber du bord, il l'écrase.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 7,
    nom: "Vents d'altitude",
    theme: Theme::Altitude,
    genre: GenreNiveau::Normal,
    intro: "Le ciel se déchire. Les courants montent : laisse-toi porter, puis plane.",
    mecanique: "Courants ascendants, longs planés, nuage friable, corbeaux, pigeons.",
    duree_cible_s: 55,
    ascii: ASCII,
    aiguilles: &[],
};

/// Dessin du niveau (190 x 11 tuiles). Légende : voir `crate::level`.
/// - Corbeaux (carte 6.4b) : (76, 3) et (147, 3) ; pigeon (38, 3) sur le nuage (38, 4).
/// - Courants (carte 6.8) : colonnes 16-18 (dès la rangée 2), 50-52, 97-99 et
///   127-129 (dès la rangée 1) ; îlot de la lanterne 136 à la rangée 3.
const ASCII: &str = r"
..............................................................................................................................................................................................
............................o.....................vvv............................................vvv.....o.....................vvv............................o...............................
................vvv...............................vvv.......c....................................vvv...........................vvv......L.....................................................
................vvv....L..............p...........vvv...NNNNNNNN............c....................vvv...NNNNNN..................vvv..NNNNNNNNNN.....c...................p......................
................vvv..NNNNNNNNNNN......N...........vvv.............NNNN...........................vvv...................L....Q..vvv.............NNNNNNNNN...............N......................
................vvv...............................vvv...................FFFF.....................vvv.............NNNNNNNNNNNNN.vvv............................M...............................
................vvv...............................vvv.........................NNNN...............vvv...........................vvv.......................NNNNNNNNN............................
..@........W.Q..vvv........................L..Q...vvv...................................L...Q....vvv...........................vvv........................................L.......c.....X.....
###############.vvv...............NNNNNNNNNNNNNN..vvv...............................NNNNNNNNNN...vvv...........................vvv.................................NNNNNNNNNNNNNNNNNNNNNNNNNNN
BBBBBBBBBBBBBBB.vvv...............................vvv............................................vvv...........................vvv............................................................
BBBBBBBBBBBBBBB.vvv...............................vvv............................................vvv...........................vvv............................................................
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::TAILLE_TUILE;
    use crate::entities::{Entite, TypeEnnemi};

    /// Carte 6.8. Un courant ascendant porte Pépin tant que sa boîte touche la
    /// zone du courant ; au-dessus de son HAUT, plus rien ne le porte : il plane
    /// en descendant (36 px/s au plus avec le parapluie). Pour que le saut vers
    /// l'îlot d'arrivée reste simple, le haut du courant doit être au moins 32 px
    /// (deux tuiles) au-dessus du dessus de cet îlot.
    ///
    /// L'îlot d'arrivée d'un courant : dans la première colonne à sa droite qui
    /// contient un solide, le solide le plus haut. Avant la carte 6.8, les marges
    /// étaient 32, 32, 0 et 16 px : le 3e courant lâchait Pépin à la hauteur de
    /// son îlot, trois tuiles plus loin (le saut « compliqué » d'AGA).
    #[test]
    fn chaque_courant_lache_pepin_au_moins_32_px_au_dessus_de_son_ilot() {
        let n = INFOS.charger().unwrap();
        let t = TAILLE_TUILE;
        let mut ilots = Vec::new();
        for c in &n.courants {
            // `find_map` : la première colonne pour laquelle la fermeture rend
            // `Some(...)`, c'est-à-dire la première qui a un solide.
            let premiere = ((c.zone.x + c.zone.w) / t) as i32;
            let (colonne, ligne) = (premiere..n.largeur() as i32)
                .find_map(|tx| {
                    (0..n.hauteur() as i32)
                        .find(|&ty| n.tuile_decor(tx, ty).est_solide())
                        .map(|ty| (tx, ty))
                })
                .expect("un îlot à droite de chaque courant");
            let marge = ligne as f32 * t - c.zone.y;
            assert!(
                marge >= 32.0,
                "courant de la colonne {} : son haut n'est que {marge} px au-dessus de l'îlot \
                 de la colonne {colonne}",
                c.zone.x / t
            );
            ilots.push((colonne, ligne, marge));
        }
        // Les quatre courants, de gauche à droite : (colonne et rangée de l'îlot, marge).
        assert_eq!(
            ilots,
            vec![(21, 4, 32.0), (56, 3, 32.0), (103, 3, 32.0), (132, 3, 32.0)]
        );
    }

    /// Ce que la carte 6.8 a déplacé ne gêne rien : aucune entité (lanterne,
    /// goutte, ennemi, nuage friable) dans un courant ; la lanterne 136 reste posée
    /// sur son îlot descendu ; la patrouille du corbeau (147, 3), à la même hauteur
    /// que cet îlot, commence après son bord droit.
    #[test]
    fn rien_dans_les_courants_et_le_corbeau_reste_hors_de_l_ilot_descendu() {
        use crate::entities::ennemi::PORTEE_CORBEAU;
        let n = INFOS.charger().unwrap();
        let t = TAILLE_TUILE;
        for e in &n.entites {
            for c in &n.courants {
                assert!(!c.zone.chevauche(&e.boite()), "{e:?} dans un courant");
            }
        }
        let lanterne = n
            .entites
            .iter()
            .find_map(|e| match e {
                Entite::Lanterne(l) if (l.boite.x / t) as i32 == 136 => Some(l.boite),
                _ => None,
            })
            .expect("la lanterne 136");
        assert_eq!(lanterne.bas(), 3.0 * t, "posée sur l'îlot de la rangée 3");
        let corbeau = n
            .entites
            .iter()
            .find_map(|e| match e {
                Entite::Ennemi(en)
                    if en.type_ennemi == TypeEnnemi::Corbeau
                        && (en.perchoir().0 / t) as i32 == 147 =>
                {
                    Some(en.perchoir().0 - PORTEE_CORBEAU)
                }
                _ => None,
            })
            .expect("le corbeau (147, 3)");
        assert!(corbeau >= 142.0 * t, "patrouille dès x = {corbeau}");
        assert!(n.tuile_decor(141, 3).est_solide() && !n.tuile_decor(142, 3).est_solide());
    }
}
