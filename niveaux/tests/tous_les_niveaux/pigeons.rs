//! Les pigeons (autrefois dans `core/src/zones.rs`) : où ils sont, loin des
//! oiseaux de décor, et leur plongée.

use crate::commun::*;
use crate::decor::case_decor;
use pepin_core::level::GenreDecor;

use pepin_core::entities::ennemi::{decalage_vol, PhasePigeon, PLONGEE_PAS};
use pepin_core::entities::{Ennemi, Entite as E, TypeEnnemi as T};

/// Écart minimal (en tuiles, horizontal OU vertical) entre un pigeon et un
/// oiseau de décor : on ne doit pas confondre l'oiseau inoffensif avec celui
/// qui plonge (exigence du coordinateur, carte 4.4).
const ECART_PIGEON_OISEAU: i32 = 4;

/// Les pigeons d'un niveau.
fn pigeons(niv: &Niveau) -> Vec<&Ennemi> {
    niv.entites
        .iter()
        .filter_map(|e| match e {
            E::Ennemi(p) if p.type_ennemi == T::Pigeon => Some(p),
            _ => None,
        })
        .collect()
}

/// Case `(tx, ty)` d'un pigeon perché.
fn case_pigeon(p: &Ennemi) -> (i32, i32) {
    let t = pepin_core::aabb::TAILLE_TUILE;
    let (x, y) = p.perchoir();
    ((x / t) as i32, (y / t) as i32)
}

/// Un à trois pigeons dans le Clocher (sur les corniches), Vents d'altitude, La mer de
/// nuages (carte 6.4b : au-dessus du premier quai), Au-dessus des nuages et La
/// descente (l'épilogue, au-dessus de la première couche de nuages), aucun ailleurs ;
/// jamais à moins de 4 tuiles d'un oiseau de décor.
#[test]
fn les_pigeons_sont_dans_le_clocher_vents_mer_nuages_et_descente_loin_des_oiseaux_de_decor() {
    let avec_pigeons = [
        par_cle("clocher").nom,
        par_cle("vents").nom,
        par_cle("mer").nom,
        par_cle("nuages").nom,
        par_cle("descente").nom,
    ];
    for infos in niveaux() {
        let niv = infos.charger().unwrap();
        let n = infos.numero;
        let ps = pigeons(&niv);
        if avec_pigeons.contains(&infos.nom) {
            assert!(
                (1..=3).contains(&ps.len()),
                "niveau {n} : {} pigeons",
                ps.len()
            );
        } else {
            assert!(ps.is_empty(), "niveau {n} : un pigeon");
        }
        for p in &ps {
            let (px, py) = case_pigeon(p);
            for d in niv.decors.iter().filter(|d| d.genre == GenreDecor::Oiseau) {
                let (ox, oy) = case_decor(d);
                let ecart = (px - ox).abs().max((py - oy).abs());
                assert!(
                    ecart >= ECART_PIGEON_OISEAU,
                    "niveau {n} : pigeon en ({px}, {py}) à {ecart} tuile(s) de l'oiseau \
                     en ({ox}, {oy})"
                );
            }
        }
    }
}

/// La plongée de chaque pigeon (toutes les positions de sa diagonale, dans le sens
/// où il regarde) reste dans le ciel : elle ne traverse aucune tuile solide, sauf le
/// coin de son propre perchoir au départ, et ne passe ni sur une lanterne (là où
/// Pépin réapparaît) ni sur le départ.
#[test]
fn la_plongee_des_pigeons_reste_en_l_air_et_loin_des_lanternes() {
    use pepin_core::aabb::{Aabb, TAILLE_TUILE as T16};
    use pepin_core::physics::{JOUEUR_H, JOUEUR_L};
    for infos in niveaux() {
        let niv = infos.charger().unwrap();
        let n = infos.numero;
        let mut reperes: Vec<(Aabb, &str)> = niv
            .entites
            .iter()
            .filter_map(|e| match e {
                E::Lanterne(l) => {
                    let (x, y) = l.point_de_reapparition();
                    Some((Aabb::new(x, y, JOUEUR_L, JOUEUR_H), "lanterne"))
                }
                _ => None,
            })
            .collect();
        let (dx, dy) = niv.depart;
        reperes.push((Aabb::new(dx, dy, JOUEUR_L, JOUEUR_H), "départ"));
        for p in pigeons(&niv) {
            let (cx, cy) = case_pigeon(p);
            let perchoir = (cx, cy + 1);
            let sens = if p.vers_la_droite { 1.0 } else { -1.0 };
            for k in 1..=PLONGEE_PAS {
                let (ex, ey) = decalage_vol(PhasePigeon::Plongee(k), sens);
                let b = p.boite.decale(ex, ey);
                let (tx0, ty0) = ((b.x / T16).floor() as i32, (b.y / T16).floor() as i32);
                for ty in ty0..=((b.bas() / T16).floor() as i32) {
                    for tx in tx0..=((b.droite() / T16).floor() as i32) {
                        let tuile = Aabb::new(tx as f32 * T16, ty as f32 * T16, T16, T16);
                        let solide = niv.tuile_decor(tx, ty).est_solide();
                        assert!(
                            !(solide && tuile.chevauche(&b)) || (tx, ty) == perchoir,
                            "niveau {n} : le pigeon de ({cx}, {cy}) plonge dans la tuile \
                             ({tx}, {ty}) au pas {k}"
                        );
                    }
                }
                for (r, nom) in &reperes {
                    assert!(
                        !r.chevauche(&b),
                        "niveau {n} : le pigeon de ({cx}, {cy}) plonge sur {nom} au pas {k}"
                    );
                }
            }
        }
    }
}
