//! Tests propres au niveau « essai_horloge » (autrefois le `mod tests` de
//! `core/src/levels/essai_horloge.rs` ; la grille et ses commentaires de conception
//! sont dans son fichier `.ron`).

use crate::commun::*;
use pepin_core::entities::Entite;

/// Le niveau de ce module, lu dans son fichier `.ron`.
fn infos() -> &'static InfosNiveau {
    par_cle("essai_horloge")
}

/// Les deux aiguilles sont là où le commentaire le dit : pivots, rayons, heures
/// de départ.
#[test]
fn deux_aiguilles_a_leur_place() {
    let n = infos().charger().unwrap();
    let aiguilles: Vec<((f32, f32), f32, f32)> = n
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Mobile(m) => m.cercle().map(|c| (c.pivot, c.rayon, c.tour())),
            _ => None,
        })
        .collect();
    assert_eq!(
        aiguilles,
        vec![
            ((456.0, 88.0), 48.0, 0.75),
            ((168.0, 168.0), 64.0, 4.0 / 12.0)
        ]
    );
    assert_eq!((n.largeur(), n.hauteur()), (42, 18));
}
