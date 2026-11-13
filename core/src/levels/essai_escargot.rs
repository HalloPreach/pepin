//! Niveau d'essai « escargot » : deux falaises que l'on gravit en rebondissant
//! sur des escargots (carte 5.8 du plan de capture). Hors de la table `NIVEAUX`,
//! il n'existe QUE dans les tests et dans le build de QA (fonctionnalité
//! `debug_niveaux`, touche F6) ; en production il n'est pas compilé dans le
//! binaire, comme les autres niveaux d'essai.
//!
//! L'escargot est « un champignon qui marche » : chaque falaise est trop haute
//! pour un saut, mais le rebond sur l'escargot y porte Pépin. Il faut l'attendre
//! au bon endroit (au pied de la falaise, où il fait demi-tour), puis lui sauter
//! dessus. Le toucher de côté est sans danger : Pépin le traverse en marchant.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 0,
    nom: "Essai escargot",
    theme: Theme::Orage,
    genre: GenreNiveau::Normal,
    intro: "L'escargot est un champignon qui marche : impossible à écraser, il fait rebondir.",
    mecanique:
        "Attendre l'escargot au pied de chaque falaise, lui sauter dessus et rebondir en haut.",
    duree_cible_s: 40,
    ascii: ASCII,
    aiguilles: &[],
};

/// 35 x 11 tuiles, thème Orage (le ciel violet du futur clocher).
/// - Sol : lignes 9-10 (dessus à y = 144). Départ en colonne 1.
/// - Escargot 1 (colonne 8) : il patrouille sur le sol, du bord gauche de la carte
///   au pied de la falaise 1 (colonnes 14 à 19, lignes 4 à 8, dessus à y = 64 :
///   80 px au-dessus du sol, plus qu'un saut, qui monte de 57 px).
/// - Fosse (colonnes 20 à 29, 160 px de large) : l'escargot 2 (colonne 28) y
///   patrouille, entre les deux falaises.
/// - Falaise 2 (colonnes 30 à 34, lignes 3 à 8, dessus à y = 48 : 96 px au-dessus
///   du fond de la fosse ; le rebond, lui, monte de 103 px), le drapeau au bout. Depuis la falaise 1, même en planant, elle est
///   trop loin : il faut descendre dans la fosse et rebondir sur l'escargot 2.
/// - Gouttes : en l'air au-dessus du sol (colonne 10, ligne 1 : bien plus haut
///   qu'un saut, on l'attrape en rebondissant), falaise 1 (colonne 17), falaise 2
///   (colonne 32), sur le chemin du robot de QA. Lanternes : sol (2), falaise 1
///   (15), fond de la fosse (21).
const ASCII: &str = r"
...................................
..........o........................
................................o.X
...............L.o............#####
..............######..........#####
..............######..........#####
..............######..........#####
..............######..........#####
.@L.....s.....######.L......s.#####
###################################
###################################
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::Entite;

    /// Les deux escargots sont là où le commentaire le dit, posés sur le sol.
    #[test]
    fn deux_escargots_sur_le_sol() {
        let n = INFOS.charger().unwrap();
        let escargots: Vec<(f32, f32)> = n
            .entites
            .iter()
            .filter_map(|e| match e {
                Entite::Escargot(s) => Some((s.boite.x, s.boite.bas())),
                _ => None,
            })
            .collect();
        assert_eq!(escargots, vec![(128.0, 144.0), (448.0, 144.0)]);
        assert_eq!(n.largeur(), 35);
        assert_eq!(n.hauteur(), 11);
    }
}
