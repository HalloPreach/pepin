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
    let arrivee = n.arrivee.expect("un drapeau");
    assert_eq!(arrivee.x, 12.0 * 16.0);
    assert_eq!((arrivee.y, arrivee.h), (16.0, 32.0));
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

/// Grille haute (carte 5.2) : un éclair posé sous une corniche part du DESSOUS de la
/// corniche, pas du haut de la carte ; sans rien au-dessus, il part du haut de la carte.
#[test]
fn eclair_d_une_grille_haute_part_du_solide_au_dessus() {
    let mut lignes = vec![".............."; 30];
    lignes[0] = ".o.o.o...e....";
    lignes[11] = "....####......";
    lignes[13] = "......e.......";
    lignes[28] = ".@..........X.";
    lignes[29] = "##############";
    let n = Niveau::depuis_ascii(&lignes.join("\n")).unwrap();
    assert_eq!(n.hauteur(), 30);
    let mut zones: Vec<_> = n
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Eclair(c) => Some(c.zone),
            _ => None,
        })
        .collect();
    zones.sort_by(|a, b| a.x.total_cmp(&b.x));
    // Colonne 6 : sous la corniche de la ligne 11 (y = 12 x 16) jusqu'au sol (ligne 29).
    assert_eq!(
        (zones[0].x, zones[0].y, zones[0].bas()),
        (98.0, 192.0, 464.0)
    );
    // Colonne 9 : rien au-dessus, du haut de la carte jusqu'au sol.
    assert_eq!(
        (zones[1].x, zones[1].y, zones[1].bas()),
        (146.0, 0.0, 464.0)
    );
}

/// Zones de grêle d'un niveau, triées de gauche à droite.
fn zones_de_grele(n: &Niveau) -> Vec<crate::aabb::Aabb> {
    let mut zones: Vec<_> = n
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Grele(g) => Some(g.zone),
            _ => None,
        })
        .collect();
    zones.sort_by(|a, b| a.x.total_cmp(&b.x));
    zones
}

/// Grêle (carte 5.4) : la colonne part de la case `g` elle-même (pas du haut de
/// la carte) et s'arrête au premier solide en dessous ; sans solide dessous, elle
/// descend jusqu'au bas de la carte (rideau au-dessus d'un vide). Une corniche
/// AU-DESSUS du `g` ne change rien.
#[test]
fn grele_tombe_de_sa_case_au_premier_solide() {
    let mut lignes = ["..............."; 8];
    lignes[0] = ".o.o.o.........";
    lignes[1] = "..g......###...";
    lignes[2] = "..........g....";
    lignes[3] = ".......####....";
    lignes[4] = ".......g.......";
    lignes[5] = ".....g.........";
    lignes[6] = ".@..........X..";
    // Un trou dans le sol à la colonne 5, sous le `g` de la ligne 5.
    lignes[7] = "#####.#########";
    let n = Niveau::depuis_ascii(&lignes.join("\n")).unwrap();
    let z = zones_de_grele(&n);
    assert_eq!(z.len(), 4);
    let t = |c: usize| c as f32 * 16.0;
    // Colonne 2 : de la ligne 1 au sol (ligne 7).
    assert_eq!((z[0].x, z[0].y, z[0].bas()), (t(2) + 2.0, t(1), t(7)));
    assert_eq!(z[0].w, 12.0);
    // Colonne 5 : de la ligne 5, à travers le trou du sol, jusqu'au bas de la carte.
    assert_eq!((z[1].x, z[1].y, z[1].bas()), (t(5) + 2.0, t(5), t(8)));
    // Colonne 7 : de la ligne 4 au sol, sous la corniche de la ligne 3.
    assert_eq!((z[2].x, z[2].y, z[2].bas()), (t(7) + 2.0, t(4), t(7)));
    // Colonne 10 : de la ligne 2, arrêtée par la corniche de la ligne 3.
    assert_eq!((z[3].x, z[3].y, z[3].bas()), (t(10) + 2.0, t(2), t(3)));
}

/// Auvent (carte 5.5) : la lettre `A` pose une zone de contact de 16 x 8 px dont
/// le dessus est `HAUT_TOILE` px sous le haut de la case. Ce n'est ni une tuile ni
/// un solide : la case reste vide pour les collisions.
#[test]
fn auvent_pose_dans_sa_case_et_traversable() {
    use crate::entities::auvent::HAUT_TOILE;
    let texte = "
.o.o.o..........
.....A.....A....
.@.........B..X.
################
";
    let n = Niveau::depuis_ascii(texte).unwrap();
    let auvents: Vec<_> = n
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Auvent(a) => Some(a.boite),
            _ => None,
        })
        .collect();
    assert_eq!(auvents.len(), 2);
    assert_eq!(
        (auvents[0].x, auvents[0].y, auvents[0].w, auvents[0].h),
        (5.0 * 16.0, 16.0 + HAUT_TOILE, 16.0, 8.0)
    );
    assert_eq!(auvents[1].x, 11.0 * 16.0);
    assert!(!n.est_solide(5, 1), "la case de l'auvent reste vide");
    assert!(n.entites.iter().all(|e| match e {
        Entite::Auvent(_) => e.solide().is_none(),
        _ => true,
    }));
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

// --- Pigeon (carte 4.4) ------------------------------------------------------

#[test]
fn pigeon_perche_et_tourne_du_cote_ou_son_perchoir_s_arrete() {
    use crate::entities::ennemi::PhasePigeon;
    use crate::entities::TypeEnnemi;
    // Trois pigeons : sur un perchoir d'une tuile (colonne 3), au bout droit d'un
    // perchoir de trois tuiles (colonne 10), au bout gauche du même (colonne 8).
    let niveau = "
.o.o.o..........
...p....p.p.....
...N....NNN.....
.@............X.
################
";
    let n = Niveau::depuis_ascii(niveau).expect("niveau valide");
    let pigeons: Vec<_> = n
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Ennemi(p) if p.type_ennemi == TypeEnnemi::Pigeon => Some(p),
            _ => None,
        })
        .collect();
    assert_eq!(pigeons.len(), 3);
    // Ordre de lecture ; boîte 12 x 10 centrée, pattes sur le perchoir (y = 32).
    let attendus = [(3.0, false), (8.0, false), (10.0, true)];
    for (p, (col, droite)) in pigeons.iter().zip(attendus) {
        assert_eq!((p.boite.x, p.boite.y), (col * 16.0 + 2.0, 22.0));
        assert_eq!((p.boite.w, p.boite.h), (12.0, 10.0));
        assert_eq!(p.boite.bas(), 32.0, "posé sur la tuile du dessous");
        assert_eq!(p.vers_la_droite, droite, "colonne {col}");
        assert_eq!(p.perchoir(), (p.boite.x, p.boite.y));
        assert_eq!(p.phase(), PhasePigeon::Perche);
    }
}

#[test]
fn pigeon_sans_perchoir_refuse() {
    let en_l_air = "
.o.o.o.......
.......p.....
.@........X..
#############
";
    assert_eq!(
        Niveau::depuis_ascii(en_l_air).unwrap_err(),
        ErreurNiveau::PigeonSansPerchoir {
            ligne: 2,
            colonne: 8
        }
    );
    let tout_en_bas = "
.o.o.o.......
.@........X..
######p######
";
    assert!(matches!(
        Niveau::depuis_ascii(tout_en_bas),
        Err(ErreurNiveau::PigeonSansPerchoir { ligne: 3, .. })
    ));
    let msg = ErreurNiveau::PigeonSansPerchoir {
        ligne: 2,
        colonne: 8,
    }
    .to_string();
    assert!(msg.contains("pigeon") && msg.contains("solide"), "{msg}");
}

/// « b » : la nacelle occupe la moitié HAUTE de sa case (16 x 8, dessus au ras
/// d'un toit de la même rangée) ; elle monte au plus jusqu'au 1er solide
/// au-dessus du ballon, ou de 8 tuiles. Elle est solide (on s'y pose).
#[test]
fn montgolfiere_dans_sa_case_et_son_sommet() {
    use crate::entities::montgolfiere::{HAUT_BALLON, MONTEE_MAX};
    let texte = "
..........
..........
..........
..........
..........
..........
..........
..........
..........
..........
.....#....
..........
..........
.o.o.o....
.@b..b..X.
##########
";
    let n = Niveau::depuis_ascii(texte).expect("niveau valide");
    let nacelles: Vec<_> = n
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Montgolfiere(m) => Some(m),
            _ => None,
        })
        .collect();
    assert_eq!(nacelles.len(), 2);
    // Colonne 2 : rien au-dessus, montée de 8 tuiles (la carte est assez haute).
    let m = nacelles[0];
    assert_eq!(m.boite, crate::aabb::Aabb::new(32.0, 224.0, 16.0, 8.0));
    assert_eq!(m.altitudes(), (224.0, 224.0 - MONTEE_MAX));
    // Colonne 5 : un toit en ligne 10 (bas à y = 176) : le haut du ballon s'y arrête.
    assert_eq!(nacelles[1].altitudes(), (224.0, 176.0 + HAUT_BALLON));
    // La nacelle est un solide (pour le joueur et les robots).
    assert_eq!(Entite::Montgolfiere(m.clone()).solide(), Some(m.boite));
}

/// Le ballon occupe les deux cases au-dessus de la nacelle : un solide ou le bord
/// haut de la carte à cet endroit est refusé, avec la ligne et la colonne.
#[test]
fn montgolfiere_sans_place_pour_le_ballon_refusee() {
    let sous_un_toit = "
..........
..#.......
..........
..b.......
.o.o.o....
.@......X.
##########
";
    assert_eq!(
        Niveau::depuis_ascii(sous_un_toit).unwrap_err(),
        ErreurNiveau::MontgolfiereSansPlace {
            ligne: 4,
            colonne: 3
        }
    );
    let trop_haut = "
..........
..b.......
..........
.o.o.o....
.@......X.
##########
";
    assert!(matches!(
        Niveau::depuis_ascii(trop_haut),
        Err(ErreurNiveau::MontgolfiereSansPlace { ligne: 2, .. })
    ));
    let msg = ErreurNiveau::MontgolfiereSansPlace {
        ligne: 4,
        colonne: 3,
    }
    .to_string();
    assert!(
        msg.contains("montgolfière") && msg.contains("ballon"),
        "{msg}"
    );
}

// --- Chat endormi (carte 5.9) ------------------------------------------------

#[test]
fn chat_couche_sur_son_toit_et_tourne_vers_le_depart() {
    use crate::entities::ennemi::{PhaseChat, DORT_PAS};
    use crate::entities::TypeEnnemi;
    // Trois chats : en plein toit (colonne 3), contre un mur à sa gauche
    // (colonne 7 : il regarde à droite), et un troisième (colonne 12).
    let niveau = "
.o.o.o..........
................
................
.@.z..Bz....z.X.
################
";
    let n = Niveau::depuis_ascii(niveau).expect("niveau valide");
    let chats: Vec<_> = n
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Ennemi(c) if c.type_ennemi == TypeEnnemi::Chat => Some(c),
            _ => None,
        })
        .collect();
    assert_eq!(chats.len(), 3);
    // Boîte 14 x 8 au bas de la case, centrée : le dos 8 px au-dessus du toit.
    for (c, (colonne, droite)) in chats.iter().zip([(3.0, false), (7.0, true), (12.0, false)]) {
        assert_eq!(
            (c.boite.x, c.boite.y, c.boite.w, c.boite.h),
            (colonne * 16.0 + 1.0, 56.0, 14.0, 8.0)
        );
        assert_eq!(c.vers_la_droite, droite, "chat de la colonne {colonne}");
    }
    // Ils dorment tous au départ, avec des sommeils décalés (0, 40, 80 pas).
    let phases: Vec<PhaseChat> = chats.iter().map(|c| c.phase_chat()).collect();
    assert_eq!(
        phases,
        vec![
            PhaseChat::Dort(DORT_PAS),
            PhaseChat::Dort(DORT_PAS - 40),
            PhaseChat::Dort(DORT_PAS - 80)
        ]
    );
}

#[test]
fn chat_sans_appui_ou_sans_place_refuse() {
    // Dans le vide : rien sous lui.
    let dans_le_vide = "
.o.o.o....
..z.......
..........
.@......X.
##########
";
    assert_eq!(
        Niveau::depuis_ascii(dans_le_vide).unwrap_err(),
        ErreurNiveau::ChatSansPlace {
            ligne: 2,
            colonne: 3
        }
    );
    // Sous un toit : impossible de lui sauter dessus.
    let sous_un_toit = "
.o.o.o....
..#.......
..z.......
.@......X.
##########
";
    assert_eq!(
        Niveau::depuis_ascii(sous_un_toit).unwrap_err(),
        ErreurNiveau::ChatSansPlace {
            ligne: 3,
            colonne: 3
        }
    );
    // Sur la première ligne : le bord haut de la carte le coince aussi.
    let tout_en_haut = "
..z.......
.o#o.o....
.@......X.
##########
";
    assert!(matches!(
        Niveau::depuis_ascii(tout_en_haut),
        Err(ErreurNiveau::ChatSansPlace { ligne: 1, .. })
    ));
    let msg = ErreurNiveau::ChatSansPlace {
        ligne: 3,
        colonne: 3,
    }
    .to_string();
    assert!(
        msg.contains("chat « z »") && msg.contains("solide"),
        "{msg}"
    );
}

// --- Aiguilles d'horloge (carte 5.10) ----------------------------------------

/// Deux pivots `h` : le 1er rencontré en lisant (ligne 2, colonne 4) prend la
/// 1re ligne de la table, le 2e (ligne 2, colonne 12) la 2e.
const DEUX_HORLOGES: &str = "
................
...h.......h....
................
................
................
.o.o.o........X.
.@..............
################
";

const DEUX_REGLAGES: [ReglageAiguille; 2] = [
    ReglageAiguille {
        rayon: 2.0,
        vitesse: 30.0,
        phase: 3.0,
    },
    ReglageAiguille {
        rayon: 1.5,
        vitesse: 24.0,
        phase: 6.0,
    },
];

#[test]
fn aiguilles_posees_depuis_leur_table_dans_l_ordre_de_lecture() {
    let n = Niveau::depuis_ascii_avec_aiguilles(DEUX_HORLOGES, &DEUX_REGLAGES).unwrap();
    let aiguilles: Vec<_> = n
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Mobile(m) => m.cercle().map(|c| (m.boite, c.pivot, c.rayon, m.vitesse())),
            _ => None,
        })
        .collect();
    assert_eq!(aiguilles.len(), 2);
    // 1re : pivot au centre de la case (3, 1), rayon 2 tuiles, partie de 3 h
    // (à droite du pivot) ; plateforme 32 x 8 centrée sur le cercle.
    let (b, pivot, rayon, vitesse) = aiguilles[0];
    assert_eq!((pivot, rayon, vitesse), ((56.0, 24.0), 32.0, 30.0));
    assert_eq!((b.x, b.y, b.w, b.h), (72.0, 20.0, 32.0, 8.0));
    // 2e : pivot (11, 1), rayon 1,5 tuile, partie de 6 h (sous le pivot).
    let (b, pivot, rayon, _) = aiguilles[1];
    assert_eq!((pivot, rayon), ((184.0, 24.0), 24.0));
    assert_eq!((b.x, b.y), (168.0, 44.0));
    assert_eq!(TAILLE_AIGUILLE, (32.0, 8.0));
}

#[test]
fn aiguille_sans_reglage_ou_hors_des_bornes_refusee() {
    // Sans table (lecture simple) : les deux pivots n'ont pas de réglage.
    assert_eq!(
        Niveau::depuis_ascii(DEUX_HORLOGES).unwrap_err(),
        ErreurNiveau::AiguillesSansReglage {
            pivots: 2,
            reglages: 0
        }
    );
    // Une ligne de trop dans la table d'un niveau sans pivot.
    assert_eq!(
        Niveau::depuis_ascii_avec_aiguilles(PETIT, &DEUX_REGLAGES[..1]).unwrap_err(),
        ErreurNiveau::AiguillesSansReglage {
            pivots: 0,
            reglages: 1
        }
    );
    // Hors des bornes : rayon de moins d'une tuile, vitesse nulle ou trop grande.
    for faux in [
        ReglageAiguille {
            rayon: 0.5,
            ..DEUX_REGLAGES[1]
        },
        ReglageAiguille {
            vitesse: 0.0,
            ..DEUX_REGLAGES[1]
        },
        ReglageAiguille {
            vitesse: 121.0,
            ..DEUX_REGLAGES[1]
        },
        ReglageAiguille {
            phase: f32::NAN,
            ..DEUX_REGLAGES[1]
        },
    ] {
        assert_eq!(
            Niveau::depuis_ascii_avec_aiguilles(DEUX_HORLOGES, &[DEUX_REGLAGES[0], faux])
                .unwrap_err(),
            ErreurNiveau::AiguilleInvalide {
                ligne: 2,
                colonne: 12
            },
            "{faux:?}"
        );
    }
    let msg = ErreurNiveau::AiguillesSansReglage {
        pivots: 2,
        reglages: 0,
    }
    .to_string();
    assert!(msg.contains("« h »") && msg.contains("réglage"), "{msg}");
}

#[test]
fn aiguille_dont_le_cercle_touche_un_solide_refusee() {
    // Un toit sur le chemin de la 1re aiguille : ligne 4, colonne 4, juste sous
    // le pivot, là où elle passe à 6 h (bas de la plateforme à y = 60).
    let mut lignes: Vec<String> = DEUX_HORLOGES.lines().map(String::from).collect();
    lignes[4].replace_range(3..4, "#");
    assert_eq!(
        Niveau::depuis_ascii_avec_aiguilles(&lignes.join("\n"), &DEUX_REGLAGES).unwrap_err(),
        ErreurNiveau::AiguilleBloquee {
            ligne: 2,
            colonne: 4
        }
    );
    // Pépin debout sur l'aiguille compte aussi. Deux lignes vides de plus en haut :
    // à midi, le dessus de la 1re plateforme est à y = 20 et la tête de Pépin à
    // y = 6. Une 1re ligne vide : tout va bien. Une 1re ligne de toit (y de 0 à
    // 16) : la plateforme passe dessous sans la toucher, mais pas Pépin : refusé.
    let vide = "................";
    let grand = format!("\n{vide}\n{vide}{DEUX_HORLOGES}");
    assert!(Niveau::depuis_ascii_avec_aiguilles(&grand, &DEUX_REGLAGES).is_ok());
    let toit = format!("\n{}\n{vide}{DEUX_HORLOGES}", "#".repeat(16));
    assert_eq!(
        Niveau::depuis_ascii_avec_aiguilles(&toit, &DEUX_REGLAGES).unwrap_err(),
        ErreurNiveau::AiguilleBloquee {
            ligne: 4,
            colonne: 4
        }
    );
    let msg = ErreurNiveau::AiguilleBloquee {
        ligne: 4,
        colonne: 4,
    }
    .to_string();
    assert!(msg.contains("touche un solide"), "{msg}");
}

// --- Grand Nuage (carte 5.11) ---

/// Une petite arène : 19 colonnes (le `O` au milieu, en colonne 9), un toit à
/// droite qui coupe la colonne d'éclair la plus à droite, aucun drapeau.
const ARENE: &str = r"
...................
.........O.........
...................
...................
...................
.@..o.o.o.......##.
###################
";

#[test]
fn grand_nuage_pose_avec_ses_colonnes_et_ses_courants() {
    use crate::entities::nuage::{PhaseNuage, NB_COLONNES, PV_NUAGE};
    let n = Niveau::depuis_ascii(ARENE).expect("arène valide");
    assert!(n.arrivee.is_none(), "pas de drapeau dans l'arène");
    let g = n
        .entites
        .iter()
        .find_map(|e| match e {
            Entite::Nuage(g) => Some(g.clone()),
            _ => None,
        })
        .expect("un nuage");
    assert_eq!((g.phase(), g.pv()), (PhaseNuage::Prelude, PV_NUAGE));
    // Corps : colonnes 8 à 10, lignes 1-2.
    assert_eq!(g.boite, Aabb::new(128.0, 16.0, 48.0, 32.0));
    // Colonnes : de 1 (colonne 9 - 8) à 17 (9 + 8), du dessous du nuage (y = 48)
    // au sol (y = 96) ; la dernière s'arrête sur le toit (y = 80).
    assert_eq!(g.colonnes[0], Aabb::new(18.0, 48.0, 12.0, 48.0));
    assert_eq!(g.colonnes[8], Aabb::new(146.0, 48.0, 12.0, 48.0));
    assert_eq!(
        g.colonnes[NB_COLONNES - 1],
        Aabb::new(274.0, 48.0, 12.0, 32.0)
    );
    // Courants : colonnes 5-6 et 12-13, d'une demi-case au-dessus de la tête au sol.
    assert_eq!(g.courants[0], Aabb::new(80.0, 8.0, 32.0, 88.0));
    assert_eq!(g.courants[1], Aabb::new(192.0, 8.0, 32.0, 88.0));
}

#[test]
fn arene_du_nuage_sans_drapeau_mais_drapeau_ailleurs_obligatoire() {
    // Un drapeau dans l'arène : refusé (le vaincre finit le niveau).
    let avec_drapeau = ARENE.replacen(".......##.", "......X##.", 1);
    assert_eq!(
        Niveau::depuis_ascii(&avec_drapeau).unwrap_err(),
        ErreurNiveau::ArriveeEtNuage
    );
    // Sans nuage, le drapeau reste obligatoire.
    let sans_rien = ARENE.replace('O', ".");
    assert_eq!(
        Niveau::depuis_ascii(&sans_rien).unwrap_err(),
        ErreurNiveau::ArriveeManquante
    );
    // Deux nuages : refusé.
    let deux = ARENE.replacen("...................", ".........O.........", 1);
    assert_eq!(
        Niveau::depuis_ascii(&deux).unwrap_err(),
        ErreurNiveau::NuageMultiple { nombre: 2 }
    );
    let msg = ErreurNiveau::ArriveeEtNuage.to_string();
    assert!(msg.contains("« X »") && msg.contains("« O »"), "{msg}");
}

#[test]
fn grand_nuage_sans_place_refuse() {
    let refuse = |texte: &str, ligne: usize, colonne: usize| {
        assert_eq!(
            Niveau::depuis_ascii(texte).unwrap_err(),
            ErreurNiveau::NuageSansPlace { ligne, colonne },
            "{texte}"
        );
    };
    // Trop près du bord gauche : ses colonnes d'éclairs sortiraient de la carte.
    let mut lignes: Vec<String> = ARENE.trim().lines().map(String::from).collect();
    lignes[1] = ".......O...........".into();
    refuse(&lignes.join("\n"), 2, 8);
    // Un toit dans son corps (ligne 3, colonne 11).
    let mut lignes: Vec<String> = ARENE.trim().lines().map(String::from).collect();
    lignes[2].replace_range(10..11, "#");
    refuse(&lignes.join("\n"), 2, 10);
    // Un toit au-dessus de sa tête (ligne 1, colonne 9).
    let mut lignes: Vec<String> = ARENE.trim().lines().map(String::from).collect();
    lignes[0].replace_range(8..9, "#");
    refuse(&lignes.join("\n"), 2, 10);
    // Collé au haut de la carte : pas de place pour lui sauter sur la tête.
    let haut = ARENE.trim().lines().skip(1).collect::<Vec<_>>().join("\n");
    refuse(&haut, 1, 10);
    let msg = ErreurNiveau::NuageSansPlace {
        ligne: 2,
        colonne: 10,
    }
    .to_string();
    assert!(msg.contains("« O »") && msg.contains("sa place"), "{msg}");
}
