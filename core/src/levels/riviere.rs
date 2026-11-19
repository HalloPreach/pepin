//! Monde 2, niveau 5 : « La rivière » (carte 9.10). Au pied de la cascade, l'eau
//! part en rivière vers la droite : une DESCENTE EN BARQUE (parapluie retourné,
//! flèche du BAS), de bief en bief, chacun un peu plus bas que le précédent.
//! Difficulté : entre la grotte et la Pieuvre du port.
//!
//! Lecture du niveau (244 x 11 tuiles, un écran de haut ; l'eau descend de la
//! rangée 5 à la rangée 8), de gauche à droite :
//! 1. la berge du départ (colonnes 0 à 9), le panneau « barque » : on plane
//!    dans le BIEF 1 (colonnes 10 à 41, courant `}`). Deux poissons y sautent
//!    (colonnes 18 et 36) ; la 1re goutte flotte au-dessus de l'eau (colonne 24),
//!    près d'une chauve-souris (colonne 32) : on la prend d'un saut depuis la
//!    barque, et l'on replane pour retomber à bord ;
//! 2. un ROCHER (colonnes 42 et 43) arrête la barque, une grenouille dessus : on
//!    saute hors de la barque, on l'écrase ou on la laisse bondir, on franchit le
//!    rocher, et l'on retrouve la berge de la 1re lanterne (colonne 47) ;
//! 3. le BIEF 2 (colonnes 52 à 84), une rangée plus bas : deux poissons et une
//!    chauve-souris au-dessus de l'eau. Au bout, l'eau tombe par une petite
//!    CHUTE (colonnes 85 et 86) : la barque bascule et retombe dans le bief 3 ;
//! 4. le BIEF 3 (colonnes 85 à 118), deux rangées plus bas : deux poissons, un
//!    rocher au milieu du courant (colonnes 101 et 102) : on en saute, et l'on
//!    replane dans l'eau d'après, sous une chauve-souris ;
//! 5. la rive de la 2e lanterne (colonne 123), puis la MARE (colonnes 129 à 150,
//!    eau calme) : on la traverse de nénuphar en nénuphar (chacun s'enfonce sous
//!    Pépin). Une grenouille guette sur celui du milieu, la 2e goutte au-dessus ;
//! 6. la rive de la 3e lanterne (colonne 153), le BIEF 4 (colonnes 157 à 176), son
//!    poisson sous une chauve-souris, puis une 2e CHUTE (colonnes 177 et 178) ;
//! 7. le BIEF 5 (colonnes 177 à 229), le plus long : trois poissons, deux
//!    chauves-souris, la 3e goutte au-dessus de l'eau (colonne 194) et un rocher
//!    (colonnes 203 et 204) gardé par une grenouille ; puis la berge du drapeau
//!    (colonne 240), gardée par une grenouille.
//!
//! Chaque chute finit dans l'eau. Les poissons sautent au-dessus d'un courant,
//! jamais d'une eau calme ; les chauves-souris (`c`) volent au-dessus de l'eau,
//! loin des lanternes. Décor : des oiseaux posés (`K`) sur les berges. Tuiles du
//! thème : `game/src/sprites/tuiles_monde2/riviere.rs`.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    monde: 2,
    // Sa place dans le Monde 2 de SA branche : la 5e, avant la Pieuvre (le
    // coordinateur range l'ordre final à la fusion).
    numero: 5,
    cle: "riviere",
    sans_planer: false,
    nom: "La rivière",
    theme: Theme::Riviere,
    genre: GenreNiveau::Normal,
    intro: "Au pied de la cascade, l'eau part en rivière. Pépin la descend en barque.",
    mecanique: "Descendre la rivière en barque, de bief en bief : sauter hors de la barque \
                devant un rocher, replaner dans l'eau d'après ; traverser la mare de nénuphar \
                en nénuphar ; laisser retomber les poissons.",
    duree_cible_s: 60,
    ascii: ASCII,
    aiguilles: &[],
};

/// Dessin du niveau (186 x 11 tuiles). Légende : voir `crate::level`.
const ASCII: &str = r"
....................................................................................................................................................................................................................................................
................................c...............................................................................................................................................................c...................................................
........................o.................j.......................c...................................................................................................................................................c.............................
..........................................##.................................................................................................o............................c.........................................................................
..@...Z.K.........i.................i.....##..................................................................c...................................................................................o.................................................
BBBBBBBBBB}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}##...L............i...............i...........................................K..L...............j.............L..................................................j...............................j...X..K
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}||..............##................BBBBBBBBBB..n...n...n...n...n...BBBBBB.........i....................................##.........................BBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB||......i.......##.........i......BBBBBBBBBB~~~~~~~~~~~~~~~~~~~~~~BBBBBB}}}}}}}}}}}}}}}}}}}}||.......i................##.......i.......i.........BBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB}}}}}}}}}}}}}}}}##}}}}}}}}}}}}}}}}BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB}}}}}}}}}}}}}}}}}}}}}}}}}}##}}}}}}}}}}}}}}}}}}}}}}}}}BBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::TAILLE_TUILE;
    use crate::entities::Entite;
    use crate::level::{Niveau, Tuile};

    /// Comme à la cascade (carte 9.26c) : les deux chutes (bief 2 vers 3, bief 4
    /// vers 5) finissent dans l'eau, jamais sur un appui.
    #[test]
    fn la_chute_finit_dans_l_eau() {
        let n = Niveau::depuis_ascii(ASCII).expect("niveau valide");
        let t = TAILLE_TUILE;
        let mut chutes = 0;
        for e in &n.entites {
            let Entite::Chute(c) = e else { continue };
            chutes += 1;
            let pied = (c.zone.bas() / t) as i32;
            for tx in (c.zone.x / t) as i32..(c.zone.droite() / t) as i32 {
                assert_eq!(n.tuile_decor(tx, pied), Tuile::Eau, "chute ({tx}, {pied})");
            }
        }
        assert_eq!(chutes, 2);
    }
}
