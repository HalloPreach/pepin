//! Tests propres au niveau « arc » (autrefois le `mod tests` de
//! `core/src/levels/arc.rs` ; la grille et ses commentaires de conception
//! sont dans son fichier `.ron`).

use crate::commun::*;
use pepin_core::aabb::TAILLE_TUILE;
use pepin_core::entities::Entite;

/// Le niveau de ce module, lu dans son fichier `.ron`.
fn infos() -> &'static InfosNiveau {
    par_cle("arc")
}

/// Un pont de nuages friables d'un bout à l'autre : entre le nuage du départ
/// (colonnes 0 à 5) et celui de l'arrivée (230 à 239), aucune tuile solide, rien
/// que des dalles friables ; aucun ennemi, aucune lanterne, ni rien qui porte
/// (plateforme mobile, montgolfière, champignon...).
#[test]
fn un_pont_friable_d_un_bout_a_l_autre_sans_ennemi_ni_lanterne() {
    let n = infos().charger().unwrap();
    assert_eq!((n.largeur(), n.hauteur()), (240, 11));
    for ty in 0..n.hauteur() as i32 {
        for tx in 6..230 {
            assert!(
                !n.tuile_decor(tx, ty).est_solide(),
                "solide en ({tx}, {ty})"
            );
        }
    }
    let mut friables = 0;
    for e in &n.entites {
        match e {
            Entite::Friable(f) => {
                friables += 1;
                let colonne = (f.boite.x / TAILLE_TUILE) as i32;
                assert!((6..228).contains(&colonne), "dalle en colonne {colonne}");
            }
            Entite::Goutte(_) => {}
            autre => panic!("le bonus n'a que des dalles et des gouttes : {autre:?}"),
        }
    }
    assert!(friables >= 150, "{friables} dalles");
    assert_eq!(n.nb_lanternes(), 0);
    assert!(n.courants.is_empty() && n.vents.is_empty());
}
