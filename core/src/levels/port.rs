//! Monde 2, niveau 6 : « Le port » (carte 9.11). Juste avant le repaire de la
//! Pieuvre : la pluie arrive au port, et Pépin le traverse de quai en quai, porté
//! par les bateaux qui vont et viennent sur l'eau et par la caisse d'une grue,
//! jusqu'au BOUT DE LA JETÉE, là où commence « La Pieuvre du port ». Plan d'AGA :
//! « Le port : bateaux en plateformes mobiles, grues, mouettes ».
//!
//! Lecture du niveau (97 x 11 tuiles, un écran de haut), de gauche à droite :
//! 1. le QUAI DU DÉPART (colonnes 0 à 13) : une caisse et sa goutte, un crabe qui
//!    patrouille jusqu'au bord ;
//! 2. le BASSIN 1 (colonnes 14 à 27, 14 cases : trop large pour un saut plané) :
//!    le BATEAU 1 va et vient au milieu ; la 2e goutte au-dessus de son passage,
//!    un poisson dans l'eau libre entre le bateau et le quai 2 ;
//! 3. le QUAI 2 (colonnes 28 à 39) : une MOUETTE sur deux caisses près du bord du
//!    quai pique sur l'eau, là où l'on saute du bateau ; la lanterne 1 ; une caisse
//!    et un crabe entre elle et le bord ; puis la DARSE de la GRUE (colonnes 40 à
//!    44) : sa caisse (colonnes 43-44) monte de l'eau jusqu'au toit de l'entrepôt,
//!    trop haut pour un saut (quatre cases). On saute dessus quand elle passe ;
//! 4. le TOIT DE L'ENTREPÔT (rangée 1 : une mouette au repos en décor, la
//!    lanterne 2), puis la marche (rangée 3) où patrouille un crabe, et le quai
//!    bas (lanterne 3) ;
//! 5. le BASSIN 2, coupé par un DUC D'ALBE (le pilier d'amarrage, colonnes 76-77) :
//!    deux bateaux, chacun obligé (12 cases d'eau). La 3e goutte au-dessus du
//!    premier. Comme au quai 2, à chaque arrivée de bateau (sur le duc, puis
//!    sur la jetée) une mouette perchée sur deux caisses pique sur l'eau, et un
//!    poisson guette dans l'eau libre avant le quai ;
//! 6. le BOUT DE LA JETÉE (colonnes 90 à 96) et le drapeau.
//!
//! Tombé à l'eau en planant : la barque. Contre chaque quai de départ d'un bateau,
//! un ponton (`#` rangée 7) fait l'échelle : barque, ponton, quai ; dans la
//! darse, la caisse de la grue, quand elle descend.
//!
//! Le rendu (`game/src/scene.rs`) dessine les plateformes mobiles du port en
//! bateaux (va-et-vient horizontal) ou en caisse pendue à son câble (vertical).

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    monde: 2,
    // Recette de l'étape 9 : 5 dans SA branche (la Pieuvre passe à 6) ; place
    // finale 6 (après la rivière), rangée par le coordinateur à la fusion.
    numero: 5,
    cle: "port",
    sans_planer: false,
    nom: "Le port",
    theme: Theme::Port,
    genre: GenreNiveau::Normal,
    intro: "La pluie arrive au port. Au bout de la jetée, quelque chose remue dans l'eau.",
    mecanique: "Bateaux en va-et-vient d'un quai à l'autre, caisse de grue qui monte au \
                toit de l'entrepôt, mouettes perchées au-dessus de l'eau.",
    duree_cible_s: 60,
    ascii: ASCII,
    aiguilles: &[],
};

/// 97 x 11 tuiles, thème Port. `1`, `3`, `4` : les bateaux (rangée 8, posés
/// sur l'eau) ; `2` : la caisse de la grue (colonne 43, rangées 8 à 1).
const ASCII: &str = r"
...............................................K..L..............................................
...........................................2.BBBBBBB.............................................
.............................u...............BBBBBBB..k......................u.............u.....
......o......................#...............BBBBBBBBBBBBB...................#.............#.....
..@...#...k.........o........#.L..#..k.......BBBBBBBBBBBBB..L.........o......#.............#..X..
BBBBBBBBBBBBBB..............BBBBBBBBBBBB.....BBBBBBBBBBBBBBBBBBB............BB............BBBBBBB
BBBBBBBBBBBBBB..............BBBBBBBBBBBB.....BBBBBBBBBBBBBBBBBBB............BB............BBBBBBB
BBBBBBBBBBBBBB#.............BBBBBBBBBBBB.....BBBBBBBBBBBBBBBBBBB#...........BB#...........BBBBBBB
BBBBBBBBBBBBBB.1........1.i.BBBBBBBBBBBB...2.BBBBBBBBBBBBBBBBBBB.3......3.i.BB.4......4.i.BBBBBBB
BBBBBBBBBBBBBB~~~~~~~~~~~~~~BBBBBBBBBBBB~~~~~BBBBBBBBBBBBBBBBBBB~~~~~~~~~~~~BB~~~~~~~~~~~~BBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::Entite;

    /// Trois bateaux horizontaux et une caisse de grue verticale, trois gouttes,
    /// trois lanternes, un drapeau au bout de la jetée.
    #[test]
    fn le_port_a_ses_bateaux_sa_grue_et_ses_gouttes() {
        let n = INFOS.charger().unwrap();
        let mobiles: Vec<_> = n
            .entites
            .iter()
            .filter_map(|e| match e {
                Entite::Mobile(m) => m.extremites(),
                _ => None,
            })
            .collect();
        let horizontaux = mobiles.iter().filter(|(a, b)| a.1 == b.1).count();
        let verticaux = mobiles.iter().filter(|(a, b)| a.0 == b.0).count();
        assert_eq!((horizontaux, verticaux), (3, 1));
        let gouttes = n
            .entites
            .iter()
            .filter(|e| matches!(e, Entite::Goutte(_)))
            .count();
        assert_eq!(gouttes, 3);
        assert_eq!(n.nb_lanternes(), 3);
        assert!(n.arrivee.is_some());
    }
}
