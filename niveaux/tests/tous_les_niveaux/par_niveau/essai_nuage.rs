//! Tests propres au niveau « essai_nuage » (autrefois le `mod tests` de
//! `core/src/levels/essai_nuage.rs` ; la grille et ses commentaires de conception
//! sont dans son fichier `.ron`).

use crate::commun::*;
use pepin_core::aabb::Aabb;
use pepin_core::entities::nuage::{NB_COLONNES, TAILLE_NUAGE};
use pepin_core::entities::Entite;

/// Le niveau de ce module, lu dans son fichier `.ron`.
fn infos() -> &'static InfosNiveau {
    par_cle("essai_nuage")
}

/// Le nuage, ses colonnes et ses courants sont là où le commentaire le dit ;
/// pas de drapeau.
#[test]
fn le_grand_nuage_a_sa_place() {
    let n = infos().charger().unwrap();
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
