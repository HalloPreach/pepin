//! Tests propres au niveau « essai_montgolfiere » (autrefois le `mod tests` de
//! `core/src/levels/essai_montgolfiere.rs` ; la grille et ses commentaires de conception
//! sont dans son fichier `.ron`).

use crate::commun::*;
use pepin_core::entities::Entite;

/// Le niveau de ce module, lu dans son fichier `.ron`.
fn infos() -> &'static InfosNiveau {
    par_cle("essai_montgolfiere")
}

/// Les deux montgolfières sont là où le commentaire le dit : la première
/// monte de 8 tuiles, la seconde s'arrête sous le nuage de la ligne 1.
#[test]
fn deux_montgolfieres_et_leurs_sommets() {
    let n = infos().charger().unwrap();
    let alt: Vec<(f32, f32, f32)> = n
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Montgolfiere(m) => Some((m.boite.x, m.altitudes().0, m.altitudes().1)),
            _ => None,
        })
        .collect();
    assert_eq!(alt, vec![(256.0, 144.0, 64.0), (96.0, 256.0, 128.0)]);
}
