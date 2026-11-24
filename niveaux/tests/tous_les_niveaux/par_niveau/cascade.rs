//! Tests propres au niveau « cascade » (autrefois le `mod tests` de
//! `core/src/levels/cascade.rs` ; la grille et ses commentaires de conception
//! sont dans son fichier `.ron`).

use crate::commun::*;
use pepin_core::aabb::TAILLE_TUILE;
use pepin_core::entities::Entite;
use pepin_core::level::{Niveau, Tuile};

/// Le niveau de ce module, lu dans son fichier `.ron`.
fn infos() -> &'static InfosNiveau {
    par_cle("cascade")
}

/// Carte 9.26c (AGA : « des cascades qui finissent sur des plateformes plutôt
/// que dans l'eau ») : sous le pied de chaque chute `|`, jamais un appui. Elle
/// tombe droit dans un bassin, sauf l'eau du puits, qui débouche au-dessus du
/// vide de la salle des bras et retombe (filet de décor, derrière le palier)
/// dans le bassin E.
#[test]
fn chaque_chute_finit_dans_l_eau() {
    let n = Niveau::depuis_ascii(infos().ascii).expect("niveau valide");
    let t = TAILLE_TUILE;
    let mut au_dessus_du_vide = Vec::new();
    let mut chutes = 0;
    for e in &n.entites {
        let Entite::Chute(c) = e else { continue };
        chutes += 1;
        let pied = (c.zone.bas() / t) as i32;
        for tx in (c.zone.x / t) as i32..(c.zone.droite() / t) as i32 {
            match n.tuile_decor(tx, pied) {
                Tuile::Eau => {}
                Tuile::Vide => {
                    assert!(
                        (pied..n.hauteur() as i32).any(|ty| n.tuile_decor(tx, ty) == Tuile::Eau),
                        "la chute ({tx}, {pied}) ne retombe dans aucune eau"
                    );
                    au_dessus_du_vide.push((tx, pied));
                }
                autre => panic!("la chute ({tx}, {pied}) finit sur {autre:?}"),
            }
        }
    }
    // C1, C2, le puits, les 2 bras, la chute du bassin F, les 2 des marches.
    assert_eq!(chutes, 8);
    assert_eq!(
        au_dessus_du_vide,
        vec![(22, 32)],
        "seul le puits débouche sur le vide"
    );
}
