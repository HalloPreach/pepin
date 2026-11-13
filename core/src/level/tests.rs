//! Tests du parseur : niveaux valides, erreurs typées, dimensions, entités.

use super::*;
use crate::entities::Entite;

/// Petit niveau valide : 3 gouttes, un départ, une arrivée, quelques entités.
const PETIT: &str = "
..............
.o.o.o........
.@..L.f.M.^.X.
##############
";

#[test]
fn niveau_valide_dimensions_et_reperes() {
    let n = Niveau::depuis_ascii(PETIT).expect("niveau valide");
    assert_eq!((n.largeur(), n.hauteur()), (14, 4));
    assert_eq!(n.largeur_px(), 14.0 * 16.0);
    assert_eq!(n.hauteur_px(), 64.0);
    // Départ : case (1, 2) -> pieds au bas de la case (y = 3 * 16).
    assert_eq!(n.depart.0, 16.0 + 3.0);
    assert_eq!(n.depart.1 + crate::physics::JOUEUR_H, 48.0);
    // Arrivée : 16 x 32, la case X et celle du dessus.
    assert_eq!(n.arrivee.x, 12.0 * 16.0);
    assert_eq!((n.arrivee.y, n.arrivee.h), (16.0, 32.0));
    assert_eq!(n.nb_lanternes(), 1);
    // Le sol est solide, le ciel non, hors carte : murs latéraux, ciel ouvert.
    assert!(n.est_solide(0, 3));
    assert!(!n.est_solide(0, 2));
    assert!(n.est_solide(-1, 0));
    assert!(n.est_solide(14, 0));
    assert!(!n.est_solide(3, -1));
    assert!(!n.est_solide(3, 4));
}

#[test]
fn entites_instanciees_avec_les_bons_indices() {
    let n = Niveau::depuis_ascii(PETIT).unwrap();
    let indices: Vec<u8> = n
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Goutte(g) => Some(g.index),
            _ => None,
        })
        .collect();
    assert_eq!(indices, vec![0, 1, 2], "indices dans l'ordre de lecture");
    let pointes = n
        .entites
        .iter()
        .filter(|e| matches!(e, Entite::Pointe(_)))
        .count();
    let champis = n
        .entites
        .iter()
        .filter(|e| matches!(e, Entite::Champignon(_)))
        .count();
    let ennemis = n
        .entites
        .iter()
        .filter(|e| matches!(e, Entite::Ennemi(_)))
        .count();
    assert_eq!((pointes, champis, ennemis), (1, 1, 1));
}

#[test]
fn erreurs_typees() {
    let sans_depart = PETIT.replace('@', ".");
    assert_eq!(
        Niveau::depuis_ascii(&sans_depart).unwrap_err(),
        ErreurNiveau::DepartManquant
    );
    let sans_arrivee = PETIT.replace('X', ".");
    assert_eq!(
        Niveau::depuis_ascii(&sans_arrivee).unwrap_err(),
        ErreurNiveau::ArriveeManquante
    );
    let deux_departs = PETIT.replace('X', "@");
    assert_eq!(
        Niveau::depuis_ascii(&deux_departs).unwrap_err(),
        ErreurNiveau::DepartMultiple { nombre: 2 }
    );
    let deux_arrivees = PETIT.replace('L', "X");
    assert_eq!(
        Niveau::depuis_ascii(&deux_arrivees).unwrap_err(),
        ErreurNiveau::ArriveeMultiple { nombre: 2 }
    );
    let deux_gouttes = PETIT.replacen('o', ".", 1);
    assert_eq!(
        Niveau::depuis_ascii(&deux_gouttes).unwrap_err(),
        ErreurNiveau::NombreDeGouttes { trouve: 2 }
    );
    assert_eq!(Niveau::depuis_ascii("").unwrap_err(), ErreurNiveau::Vide);
    assert_eq!(
        Niveau::depuis_ascii("\n\n  \n").unwrap_err(),
        ErreurNiveau::Vide
    );
}

#[test]
fn caractere_inconnu_et_lignes_inegales() {
    let inconnu = PETIT.replace('M', "?");
    assert_eq!(
        Niveau::depuis_ascii(&inconnu).unwrap_err(),
        ErreurNiveau::CaractereInconnu {
            ligne: 3,
            colonne: 9,
            caractere: '?'
        }
    );
    let inegal = PETIT.replace("##############", "#############");
    assert_eq!(
        Niveau::depuis_ascii(&inegal).unwrap_err(),
        ErreurNiveau::LargeurInegale {
            ligne: 4,
            attendu: 14,
            trouve: 13
        }
    );
}

#[test]
fn messages_d_erreur_en_francais() {
    let msg = ErreurNiveau::CaractereInconnu {
        ligne: 3,
        colonne: 9,
        caractere: '?',
    }
    .to_string();
    assert!(
        msg.contains("ligne 3") && msg.contains("caractère inconnu"),
        "{msg}"
    );
    // Compatible avec `Box<dyn Error>` grâce au trait `Error`.
    let _: Box<dyn std::error::Error> = Box::new(ErreurNiveau::Vide);
}

#[test]
fn plateformes_mobiles_par_paires() {
    let avec = "
.o.o.o.......
.@.1...1...X.
#############
";
    let n = Niveau::depuis_ascii(avec).unwrap();
    let mobiles: Vec<_> = n
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Mobile(m) => Some(m.boite),
            _ => None,
        })
        .collect();
    assert_eq!(mobiles.len(), 1);
    assert_eq!(
        (mobiles[0].x, mobiles[0].y, mobiles[0].w),
        (48.0, 16.0, 32.0)
    );
    let seule = avec.replacen('1', ".", 1);
    assert_eq!(
        Niveau::depuis_ascii(&seule).unwrap_err(),
        ErreurNiveau::PlateformeMobileIncomplete {
            marque: '1',
            trouve: 1
        }
    );
}

#[test]
fn courants_fusionnes_en_rectangles() {
    let avec = "
.o.o.o..vv....
.@.....vvv..X.
##############
";
    // Colonnes 8-9 sur la ligne 0 ; colonnes 7-9 sur la ligne 1.
    let n = Niveau::depuis_ascii(avec).unwrap();
    // Colonnes 8 et 9 : tronçon lignes 0-1 fusionné ; colonne 7 : ligne 1 seule.
    assert_eq!(n.courants.len(), 2);
    let grand = n.courants.iter().find(|c| c.zone.h == 32.0).unwrap();
    assert_eq!((grand.zone.x, grand.zone.w), (8.0 * 16.0, 32.0));
}

#[test]
fn eclair_tombe_jusqu_au_premier_solide() {
    let avec = "
.o.o.o...e....
.@..........X
##########.###
";
    // La ligne 2 est plus courte d'une colonne : erreur de largeur attendue.
    assert!(matches!(
        Niveau::depuis_ascii(avec),
        Err(ErreurNiveau::LargeurInegale { .. })
    ));
    let bon = avec.replace(".@..........X", ".@..........X.");
    let n = Niveau::depuis_ascii(&bon).unwrap();
    let zone = n
        .entites
        .iter()
        .find_map(|e| match e {
            Entite::Eclair(c) => Some(c.zone),
            _ => None,
        })
        .unwrap();
    assert_eq!(zone.y, 0.0);
    assert_eq!(zone.bas(), 32.0, "s'arrête au sommet du sol (ligne 2)");
}

#[test]
fn panneaux_et_tuiles_decor() {
    let avec = "
.o.o.o.......
.@DSGCW...X..
.B.N.........
#############
";
    let n = Niveau::depuis_ascii(avec).unwrap();
    assert_eq!(n.panneaux.len(), 5);
    assert_eq!(n.panneaux[0].genre, GenrePanneau::Deplacer);
    assert_eq!(n.panneaux[4].genre, GenrePanneau::Vent);
    assert_eq!(n.tuile_decor(1, 2), Tuile::Brique);
    assert_eq!(n.tuile_decor(3, 2), Tuile::Nuage);
    assert_eq!(n.tuile_decor(0, 3), Tuile::Toit);
    assert!(Tuile::Toit.est_solide() && !Tuile::Vide.est_solide());
}

#[test]
fn objets_de_decor_lus_et_poses_sur_leur_appui() {
    let avec = "
.o.o.o.......
.@.HTRQK..X..
#####B#N#####
";
    let n = Niveau::depuis_ascii(avec).unwrap();
    let genres: Vec<GenreDecor> = n.decors.iter().map(|d| d.genre).collect();
    assert_eq!(
        genres,
        GenreDecor::TOUS.to_vec(),
        "ordre de lecture = ordre de la légende"
    );
    // Coin haut-gauche de la case : H en colonne 3, ligne 1.
    assert_eq!((n.decors[0].x, n.decors[0].y), (48.0, 16.0));
    // Aucune collision : les cases du décor restent vides, rien d'autre n'est créé.
    for d in &n.decors {
        let (tx, ty) = ((d.x / 16.0) as i32, (d.y / 16.0) as i32);
        assert_eq!(n.tuile_decor(tx, ty), Tuile::Vide);
        assert!(n.tuile_decor(tx, ty + 1).est_solide());
    }
    assert!(n.panneaux.is_empty());
    assert_eq!(n.entites.len(), 3, "les 3 gouttes seulement");
    // Chaque lettre se relit dans les deux sens.
    for g in GenreDecor::TOUS {
        assert_eq!(GenreDecor::depuis_lettre(g.lettre()), Some(g));
    }
    assert_eq!(GenreDecor::depuis_lettre('.'), None);
    assert_eq!(
        GenreDecor::depuis_lettre('h'),
        None,
        "lettres majuscules seulement"
    );
}

#[test]
fn decor_en_l_air_refuse() {
    // Oiseau au-dessus du vide (ligne 1, colonne 9 : sous lui, la ligne 2 est vide) : erreur typée.
    let en_l_air = "
.o.o.o..K....
.@........X..
#############
";
    assert_eq!(
        Niveau::depuis_ascii(en_l_air).unwrap_err(),
        ErreurNiveau::DecorSansAppui {
            ligne: 1,
            colonne: 9,
            caractere: 'K'
        }
    );
    // Sur la dernière ligne, il n'y a pas de dessous du tout.
    let tout_en_bas = "
.o.o.o.......
.@........X..
######H######
";
    assert!(matches!(
        Niveau::depuis_ascii(tout_en_bas),
        Err(ErreurNiveau::DecorSansAppui { caractere: 'H', .. })
    ));
    let msg = ErreurNiveau::DecorSansAppui {
        ligne: 2,
        colonne: 9,
        caractere: 'K',
    }
    .to_string();
    assert!(
        msg.contains("décor « K »") && msg.contains("solide"),
        "{msg}"
    );
}
