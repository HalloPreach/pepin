//! Tests propres au niveau « essai_escargot » (autrefois le `mod tests` de
//! `core/src/levels/essai_escargot.rs` ; la grille et ses commentaires de conception
//! sont dans son fichier `.ron`).

use crate::commun::*;
use pepin_core::entities::Entite;

/// Le niveau de ce module, lu dans son fichier `.ron`.
fn infos() -> &'static InfosNiveau {
    par_cle("essai_escargot")
}

/// Les deux escargots sont là où le commentaire le dit, posés sur le sol.
#[test]
fn deux_escargots_sur_le_sol() {
    let n = infos().charger().unwrap();
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
