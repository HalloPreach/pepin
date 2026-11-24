//! Tests propres au niveau « riviere » (autrefois le `mod tests` de
//! `core/src/levels/riviere.rs` ; la grille et ses commentaires de conception
//! sont dans son fichier `.ron`).

use crate::commun::*;
use pepin_core::aabb::TAILLE_TUILE;
use pepin_core::entities::Entite;
use pepin_core::level::{Niveau, Tuile};

/// Le niveau de ce module, lu dans son fichier `.ron`.
fn infos() -> &'static InfosNiveau {
    par_cle("riviere")
}

/// Comme à la cascade (carte 9.26c) : les deux chutes (bief 2 vers 3, bief 4
/// vers 5) finissent dans l'eau, jamais sur un appui.
#[test]
fn la_chute_finit_dans_l_eau() {
    let n = Niveau::depuis_ascii(infos().ascii).expect("niveau valide");
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
