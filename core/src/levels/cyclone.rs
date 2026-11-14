//! Niveau 6 : « L'œil du cyclone » (le boss).
//!
//! « Le Grand Nuage ne veut pas te laisser passer. » Au sommet du clocher, Pépin
//! est monté dans l'œil du cyclone : une arène d'un seul écran, bordée de murs
//! de nuages, sous une trouée d'or (le ciel, tout là-haut) et des tours de nuages
//! violettes. Il n'y a pas de drapeau : le niveau est fini quand le Grand Nuage
//! (lettre `O`, carte 5.11) est vaincu.
//!
//! Le combat :
//! 1. le nuage gronde (1,5 s), puis lance un MOTIF d'éclairs sur les 17 colonnes
//!    de l'arène, chacune annoncée comme un éclair `e` (48 pas d'annonce, 12 de
//!    frappe) : on court entre les salves ;
//! 2. le motif fini, il s'essouffle (5 s) et laisse échapper un courant ascendant
//!    à côté de lui (à gauche après les motifs 1 et 3, à droite après le 2) : on y
//!    monte, et on lui retombe sur la tête (un point) ;
//! 3. trois motifs de plus en plus serrés, trois coups, et le ciel se déchire.
//!
//! Une mort ramène à la lanterne, au début de l'arène (la seule) ; le nuage
//! reprend son grondement mais GARDE les points perdus.
//!
//! L'arène reprend la géométrie du niveau d'essai « nuage » (`essai_nuage`), sur
//! laquelle le robot de QA a appris à battre le Grand Nuage : mêmes murs, même
//! place du nuage, des gouttes et de la lanterne. Seuls changent les tuiles (des
//! nuages `N` au lieu des toits : on est dans le ciel) et l'ambiance (thème
//! `Cyclone`, violet et or).

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 6,
    nom: "L'œil du cyclone",
    theme: Theme::Cyclone,
    genre: GenreNiveau::Normal,
    intro: "Le Grand Nuage ne veut pas te laisser passer.",
    mecanique: "Le Grand Nuage (boss) : éviter ses éclairs, monter dans son courant quand il \
                s'essouffle, lui sauter sur la tête trois fois.",
    duree_cible_s: 40,
    ascii: ASCII,
    aiguilles: &[],
};

/// 20 x 11 tuiles (un seul écran), murs et sol de nuages.
/// - Sol : lignes 9-10 (dessus à y = 144). Murs : colonnes 0-1 et 19, toute la
///   hauteur : l'arène fait les 17 colonnes (2 à 18) que les éclairs du nuage
///   couvrent, sans coin à l'abri.
/// - Le Grand Nuage : `O` en (10, 2) ; son corps (48 x 32) couvre les colonnes 9 à
///   11, lignes 2-3 (tête à y = 32, dessous à y = 64, 80 px au-dessus du sol :
///   hors de portée d'un saut). Ses colonnes d'éclairs vont de y = 64 au sol.
/// - Ses courants : colonnes 6-7 (à gauche, après les motifs 1 et 3) et 13-14 (à
///   droite, après le motif 2), de y = 24 (une demi-case au-dessus de sa tête) au
///   sol.
/// - Gouttes : au sol à gauche (colonne 5) et, à la hauteur de sa tête, entre
///   chaque courant et le nuage (colonnes 8 et 12, ligne 1) : on y passe forcément
///   pour aller du courant à sa tête ; celle de droite attend le deuxième motif.
///   Lanterne : au départ (le début de l'arène, où l'on reprend après une mort).
const ASCII: &str = r"
NN.................N
NN......o...o......N
NN........O........N
NN.................N
NN.................N
NN.................N
NN.................N
NN.................N
NN@L.o.............N
NNNNNNNNNNNNNNNNNNNN
NNNNNNNNNNNNNNNNNNNN
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::Aabb;
    use crate::entities::Entite;
    use crate::levels::essai_nuage;

    /// L'arène du boss : un écran, pas de drapeau, un seul Grand Nuage, une seule
    /// lanterne (au départ), trois gouttes ; et EXACTEMENT la géométrie du niveau
    /// d'essai « nuage » (mêmes cases vides ou pleines, même nuage, mêmes
    /// colonnes, mêmes courants, mêmes gouttes, même départ), sur laquelle le
    /// combat a été réglé et le robot de QA entraîné : seules les tuiles changent
    /// de dessin.
    #[test]
    fn l_arene_du_cyclone_reprend_celle_du_niveau_d_essai() {
        let n = INFOS.charger().unwrap();
        let e = essai_nuage::INFOS.charger().unwrap();
        assert_eq!((n.largeur(), n.hauteur()), (20, 11));
        assert!(n.arrivee.is_none(), "l'arène n'a pas de drapeau");
        assert_eq!(n.nb_lanternes(), 1);
        assert_eq!(n.depart, e.depart);
        for ty in 0..11 {
            for tx in 0..20 {
                assert_eq!(
                    n.tuile_decor(tx, ty).est_solide(),
                    e.tuile_decor(tx, ty).est_solide(),
                    "case ({tx}, {ty})"
                );
            }
        }
        let boites =
            |entites: &[Entite]| -> Vec<Aabb> { entites.iter().map(|e| e.boite()).collect() };
        assert_eq!(boites(&n.entites), boites(&e.entites));
        let nuages = |entites: &[Entite]| -> Vec<_> {
            entites
                .iter()
                .filter_map(|e| match e {
                    Entite::Nuage(g) => Some((g.boite, g.colonnes, g.courants)),
                    _ => None,
                })
                .collect()
        };
        assert_eq!(nuages(&n.entites).len(), 1);
        assert_eq!(nuages(&n.entites), nuages(&e.entites));
    }
}
