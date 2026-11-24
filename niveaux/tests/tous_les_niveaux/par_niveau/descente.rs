//! Tests propres au niveau « descente » (autrefois le `mod tests` de
//! `core/src/levels/descente.rs` ; la grille et ses commentaires de conception
//! sont dans son fichier `.ron`).

use crate::commun::*;
use pepin_core::aabb::TAILLE_TUILE as T;
use pepin_core::entities::contacts::{resoudre_contacts, verifier_vide, Contact};
use pepin_core::entities::{solides_des_entites, Actualisable, Entite, TypeEnnemi};
use pepin_core::physics::{Environnement, JOUEUR_H};
use pepin_core::player::{Direction, Entrees, Joueur};

/// Le niveau de ce module, lu dans son fichier `.ron`.
fn infos() -> &'static InfosNiveau {
    par_cle("descente")
}

/// Le plus calme des douze niveaux : le toit de l'arrivée couvre toute la largeur
/// (aucune chute ne tue : tout ce qui tombe finit sur une couche ou sur le toit),
/// aucune pointe, aucun éclair, aucune plateforme qui bouge ; pour seuls dangers,
/// deux pigeons, deux colonnes de grêle et, depuis la carte 6.4b (« plus
/// d'ennemis » : un au plus pour l'épilogue), un corbeau.
#[test]
fn aucune_chute_ne_tue_et_seuls_deux_pigeons_un_corbeau_et_deux_colonnes_de_grele_menacent() {
    let n = infos().charger().unwrap();
    assert_eq!((n.largeur(), n.hauteur()), (32, 45));
    for tx in 0..n.largeur() as i32 {
        assert!(
            n.tuile_decor(tx, 42).est_solide(),
            "toit percé en colonne {tx}"
        );
    }
    let (mut pigeons, mut corbeaux, mut greles) = (0, 0, 0);
    for e in &n.entites {
        match e {
            Entite::Ennemi(p) if p.type_ennemi == TypeEnnemi::Pigeon => pigeons += 1,
            Entite::Ennemi(c) if c.type_ennemi == TypeEnnemi::Corbeau => corbeaux += 1,
            Entite::Grele(_) => greles += 1,
            Entite::Goutte(_) | Entite::Lanterne(_) => {}
            autre => {
                panic!(
                    "l'épilogue n'a que pigeons, corbeau, grêle, gouttes et lanternes : \
                     {autre:?}"
                )
            }
        }
    }
    assert_eq!((pigeons, corbeaux, greles), (2, 1, 2));
    assert!(n.courants.is_empty() && n.vents.is_empty());
}

/// Se laisser tomber du bord d'un trou en courant (direction gardée, sans saut ni
/// parapluie) pose Pépin sur la couche du dessous (ou sur le toit), sans toucher la
/// colonne de grêle qui longe le bord lointain du trou. Vraie physique, pas à pas.
#[test]
fn se_laisser_tomber_du_bord_d_un_trou_ne_touche_pas_la_grele() {
    let n = infos().charger().unwrap();
    // (colonne de départ, sens de la course, rangée de la couche, rangée d'arrivée)
    for (colonne, sens, de, vers) in [
        (20, Direction::Droite, 15, 24),
        (11, Direction::Gauche, 24, 33),
        (20, Direction::Droite, 33, 42),
    ] {
        let mut entites = n.entites.clone();
        let mut j = Joueur::new(colonne as f32 * T, de as f32 * T - JOUEUR_H);
        let entrees = Entrees {
            direction: sens,
            saut: false,
            bas: false,
        };
        let mut pose = None;
        for _ in 0..600 {
            let solides = solides_des_entites(&entites);
            let env = Environnement {
                grille: &n,
                solides: &solides,
                courants: &n.courants,
                vents: &n.vents,
                eaux: &[],
            };
            for e in entites.iter_mut() {
                e.actualiser(&env);
            }
            j.actualiser(&entrees, &env);
            let contacts = resoudre_contacts(&mut j, &mut entites);
            assert!(
                !contacts.iter().any(|c| matches!(c, Contact::Mort(_))),
                "couche {de} : mort en {:?}",
                (j.boite.x, j.boite.y)
            );
            assert!(verifier_vide(&j, n.hauteur_px()).is_none());
            if j.au_sol && j.boite.bas() > de as f32 * T + 1.0 {
                pose = Some(j.boite.bas());
                break;
            }
        }
        assert_eq!(pose, Some(vers as f32 * T), "couche {de}");
    }
}
