//! Niveau d'essai « nuage » : l'arène du Grand Nuage, le boss (carte 5.11 du plan
//! de capture), avant-goût de « L'œil du cyclone ». Hors de la table `NIVEAUX`, il
//! n'existe QUE dans les tests et dans le build de QA (fonctionnalité
//! `debug_niveaux`, touche F10) ; en production il n'est pas compilé dans le
//! binaire, comme les autres niveaux d'essai.
//!
//! Le Grand Nuage gronde, puis lance un motif d'éclairs sur l'arène (chaque
//! colonne annoncée comme un éclair `e`). Le motif fini, il s'essouffle et laisse
//! échapper un courant ascendant à côté de lui : on y monte, et on lui retombe sur
//! la tête. Trois coups (trois motifs de plus en plus serrés, le courant change de
//! côté), et le ciel se déchire : le niveau est fini. Il n'y a pas de drapeau.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 0,
    nom: "Essai nuage",
    theme: Theme::Orage,
    genre: GenreNiveau::Normal,
    intro: "Le Grand Nuage ne veut pas te laisser passer.",
    mecanique: "Éviter ses éclairs, monter dans son courant quand il s'essouffle, lui sauter \
                sur la tête trois fois.",
    duree_cible_s: 40,
    ascii: ASCII,
    aiguilles: &[],
};

/// 20 x 11 tuiles (un seul écran), thème Orage (le violet du futur œil du cyclone).
/// - Sol : lignes 9-10 (dessus à y = 144). Murs : colonnes 0-1 et 19, toute la
///   hauteur : l'arène fait les 17 colonnes (2 à 18) que les éclairs du nuage
///   couvrent, sans coin à l'abri.
/// - Le Grand Nuage : `O` en (10, 2) ; son corps (48 x 32) couvre les colonnes 9 à
///   11, lignes 2-3 (tête à y = 32, dessous à y = 64, 80 px au-dessus du sol :
///   hors de portée d'un saut). Ses colonnes d'éclairs vont de y = 64 au sol.
/// - Ses courants : colonnes 6-7 (à gauche, après les motifs 1 et 3) et 13-14 (à
///   droite, après le motif 2), de y = 24 (une demi-case au-dessus de sa tête) au
///   sol.
/// - Gouttes : au sol à gauche (colonne 5), en haut du courant de gauche (colonne
///   7, ligne 1 : on la prend en montant vers le premier coup), en haut du courant
///   de droite (colonne 13, ligne 1 : au deuxième coup). Lanterne : au départ (le
///   début de l'arène, où l'on reprend après une mort).
const ASCII: &str = r"
##.................#
##.....o.....o.....#
##........O........#
##.................#
##.................#
##.................#
##.................#
##.................#
##@L.o.............#
####################
####################
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::Aabb;
    use crate::entities::nuage::{NB_COLONNES, TAILLE_NUAGE};
    use crate::entities::Entite;

    /// Le nuage, ses colonnes et ses courants sont là où le commentaire le dit ;
    /// pas de drapeau.
    #[test]
    fn le_grand_nuage_a_sa_place() {
        let n = INFOS.charger().unwrap();
        assert_eq!((n.largeur(), n.hauteur()), (20, 11));
        assert!(n.arrivee.is_none(), "l'arène n'a pas de drapeau");
        let nuages: Vec<_> = n
            .entites
            .iter()
            .filter_map(|e| match e {
                Entite::Nuage(g) => Some(g.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(nuages.len(), 1);
        let g = &nuages[0];
        assert_eq!(
            g.boite,
            Aabb::new(144.0, 32.0, TAILLE_NUAGE.0, TAILLE_NUAGE.1)
        );
        // 17 colonnes, de la colonne 2 à la 18, de 64 au sol (144).
        assert_eq!(g.colonnes.len(), NB_COLONNES);
        assert_eq!(g.colonnes[0], Aabb::new(34.0, 64.0, 12.0, 80.0));
        assert_eq!(g.colonnes[16], Aabb::new(290.0, 64.0, 12.0, 80.0));
        assert_eq!(g.courants[0], Aabb::new(96.0, 24.0, 32.0, 120.0));
        assert_eq!(g.courants[1], Aabb::new(208.0, 24.0, 32.0, 120.0));
    }
}
