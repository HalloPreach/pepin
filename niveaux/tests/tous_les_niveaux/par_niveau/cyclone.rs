//! Tests propres au niveau « cyclone » (autrefois le `mod tests` de
//! `core/src/levels/cyclone.rs` ; la grille et ses commentaires de conception
//! sont dans son fichier `.ron`).

use crate::commun::*;
use pepin_core::aabb::Aabb;
use pepin_core::entities::Entite;

/// Le niveau de ce module, lu dans son fichier `.ron`.
fn infos() -> &'static InfosNiveau {
    par_cle("cyclone")
}

/// L'arène du boss : un écran, pas de drapeau, un seul Grand Nuage, une seule
/// lanterne (au départ), trois gouttes ; et EXACTEMENT la géométrie du niveau
/// d'essai « nuage » (mêmes cases vides ou pleines, même nuage, mêmes
/// colonnes, mêmes courants, mêmes gouttes, même départ), sur laquelle le
/// combat a été réglé et le robot de QA entraîné : seules les tuiles changent
/// de dessin.
#[test]
fn l_arene_du_cyclone_reprend_celle_du_niveau_d_essai() {
    let n = infos().charger().unwrap();
    let e = par_cle("essai_nuage").charger().unwrap();
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
    let boites = |entites: &[Entite]| -> Vec<Aabb> { entites.iter().map(|e| e.boite()).collect() };
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
