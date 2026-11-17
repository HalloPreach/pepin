//! Monde 2, niveau 1 : « Le caniveau » (carte 9.6). Le premier vrai niveau du
//! Monde 2, « Là où va la pluie » : on y APPREND LA BARQUE en suivant l'eau des
//! gouttières, de la rue sèche jusqu'à la bouche d'égout. Difficulté : celle des
//! deux premiers niveaux du Monde 1, au plus (une seule flaque, aucun ennemi
//! d'eau : ils arrivent dans les égouts et à la rivière).
//!
//! Lecture du niveau (128 x 11 tuiles, un écran de haut), de gauche à droite :
//! 1. le CANIVEAU À SEC (colonnes 9 à 24) : un creux d'une tuile dans la rue où
//!    l'on marche ; une flaque y patrouille, la 1re goutte flotte au-dessus d'elle
//!    (rangée 6) : on la prend d'un saut, en écrasant la flaque ou en l'évitant ;
//! 2. le PREMIER FILET D'EAU (colonnes 33 à 46), une tuile de fond, sans aucun
//!    danger : le panneau « planer » (colonne 28) le précède. Trop large pour un
//!    saut planant (14 tuiles), on y entre en planant : le parapluie retourné
//!    devient une barque (eau calme `~`, puis un courant `}` qui la pousse). La
//!    2e goutte est posée sur l'eau (colonne 44) ; la BORDURE (colonnes 47 à 49,
//!    une tuile plus haut que la rue) arrête la barque : on en sort d'un saut ;
//! 3. la GOUTTIÈRE (colonnes 61 à 93) : deux marches, puis le rebord gauche
//!    (colonne 61, sa lanterne dessus). L'eau y coule en rangée 5, sur la maison
//!    (des briques jusqu'à la rue : aucun passage à pied dessous). Le courant `}`
//!    emporte la barque sous un avant-toit (colonnes 75 à 81) : la 3e goutte y
//!    attend (colonne 79, rangée 4), dans le seul espace libre entre l'eau et le
//!    toit. On ne l'atteint qu'EN BARQUE. Au bout, le rebord droit (colonne 93) :
//!    on saute, on redescend dans la rue ;
//! 4. le COURANT DE LA RUE (colonnes 103 à 114) : la gouttière se vide dans la
//!    rue ; un dernier tour de barque jusqu'à la bouche d'égout (la bordure des
//!    colonnes 115 à 117), puis le drapeau (colonne 122).
//!
//! Quatre lanternes : avant le filet d'eau (30), après sa bordure (52), sur le
//! rebord de la gouttière (61, rangée 3) et avant le courant de la rue (99).
//! Aucun bassin n'est suspendu au-dessus d'un passage à pied : l'eau est vérifiée
//! à la hauteur de Pépin (`physics::eau_touchee`).

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    monde: 2,
    numero: 1,
    cle: "caniveau",
    sans_planer: false,
    nom: "Le caniveau",
    theme: Theme::Caniveau,
    genre: GenreNiveau::Normal,
    intro: "La pluie descend des toits et file dans le caniveau. Pépin la suit.",
    mecanique: "La barque : planer dans l'eau pour flotter (parapluie fermé, on coule), se laisser porter par le courant, sauter pour sortir de l'eau.",
    duree_cible_s: 50,
    ascii: ASCII,
    aiguilles: &[],
};

/// Dessin du niveau (128 x 11 tuiles). Légende : voir `crate::level`.
/// Décor (sans collision) : une cheminée au départ, une cheminée et une antenne
/// sur l'avant-toit de la gouttière, une antenne sur la bouche d'égout.
const ASCII: &str = r"
................................................................................................................................
............................................................................H...T...............................................
...........................................................................#######..............................................
.............................................................L.............BBBBBBB..............................................
.............................................................#.................o.............#..................................
.............................................................B~~~}}}}}}}}}}}}}}}}}}}}}}}}}}}}B..................................
....................o......................................##BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB......................T...........
..@..D.H....................G.L.............o..###..L....##BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB.....L...............###....X.....
#########.......f........########~~~}}}}}}}}}}}BBB#####################################################~~}}}}}}}}}}BBB##########
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";

#[cfg(test)]
mod tests {
    use super::*;

    /// Les rangées du dessin (rangée 0 en haut), en caractères.
    fn grille() -> Vec<Vec<char>> {
        ASCII.trim().lines().map(|l| l.chars().collect()).collect()
    }

    /// La goutte de la gouttière ne s'atteint qu'EN BARQUE : elle est dans la seule
    /// rangée libre entre l'eau (dessous) et l'avant-toit (dessus), et toute cette
    /// rangée sous l'avant-toit est au-dessus de l'eau : nulle part où se tenir.
    #[test]
    fn la_goutte_de_la_gouttiere_ne_s_atteint_qu_en_barque() {
        let g = grille();
        assert_eq!(g[4][79], 'o');
        // Les colonnes 75 à 81 (l'avant-toit) des rangées 3, 4 et 5.
        let sous_l_avant_toit = |rangee: usize| &g[rangee][75..=81];
        let colonnes = sous_l_avant_toit(3)
            .iter()
            .zip(sous_l_avant_toit(4))
            .zip(sous_l_avant_toit(5));
        for (k, ((toit, air), eau)) in colonnes.enumerate() {
            let x = 75 + k;
            assert_eq!(*toit, 'B', "avant-toit, colonne {x}");
            assert!(matches!(air, '.' | 'o'), "rangée libre, colonne {x}");
            assert_eq!(*eau, '}', "courant d'eau, colonne {x}");
        }
    }

    /// Aucun bassin suspendu au-dessus d'un passage à pied (l'eau est vérifiée à
    /// la hauteur de Pépin) : sous chaque case d'eau, du solide jusqu'en bas.
    #[test]
    fn sous_l_eau_rien_que_du_solide() {
        let g = grille();
        for (y, ligne) in g.iter().enumerate() {
            for (x, &c) in ligne.iter().enumerate() {
                if matches!(c, '~' | '{' | '}') {
                    for (dessous, rangee) in g.iter().enumerate().skip(y + 1) {
                        assert!(
                            matches!(rangee[x], '#' | 'B'),
                            "colonne {x} : pas de solide en rangée {dessous} sous l'eau"
                        );
                    }
                }
            }
        }
    }

    /// Le niveau n'utilise que les lettres permises pour le premier niveau du
    /// Monde 2 : eau, courants d'eau, un nénuphar au plus, aucun ennemi d'eau.
    #[test]
    fn seulement_les_mecaniques_du_premier_niveau() {
        let lettres: String = ASCII.chars().filter(|c| !c.is_whitespace()).collect();
        for interdite in ['j', 'i', 'k', 'r', 'u', 'Y', '=', 'V', '|'] {
            assert!(!lettres.contains(interdite), "{interdite}");
        }
        assert!(lettres.matches('n').count() <= 1);
        assert_eq!(lettres.matches('o').count(), 3);
        assert_eq!(lettres.matches('L').count(), 4);
    }
}
