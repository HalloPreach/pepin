//! Parseur ASCII -> `Niveau`.
//!
//! Étapes : 1) découper les lignes et vérifier le rectangle ; 2) lire chaque
//! caractère (tuile, entité, repère, décor) ; 3) assembler les éléments qui dépendent
//! de la grille complète (éclairs, grêle, courants, plateformes mobiles, appui du décor) ;
//! 4) valider (un départ, une arrivée, trois gouttes).

use std::collections::BTreeMap;

use super::{
    ErreurNiveau, GenreDecor, GenrePanneau, Niveau, ObjetDecor, Panneau, Tuile, GOUTTES_PAR_NIVEAU,
};
use crate::aabb::{Aabb, TAILLE_TUILE};
use crate::entities::{
    Auvent, Champignon, Eclair, Ennemi, Entite, GoutteDor, Grele, Lanterne, PlateformeFriable,
    PlateformeMobile, Pointe, TypeEnnemi,
};
use crate::physics::{CourantAscendant, VentLateral, JOUEUR_H};

/// Vitesse (px/s) des plateformes mobiles.
pub(super) const VITESSE_MOBILE: f32 = 36.0;
/// Largeur (px) d'une plateforme mobile : deux tuiles.
const LARGEUR_MOBILE: f32 = 32.0;
/// Repos (pas) entre deux coups d'éclair.
const REPOS_ECLAIR: u16 = 150;

/// Position d'une case : colonne `tx`, ligne `ty` (0 = ligne du haut).
type Case = (usize, usize);

/// Pixels du coin haut-gauche d'une case.
fn px(case: Case) -> (f32, f32) {
    (case.0 as f32 * TAILLE_TUILE, case.1 as f32 * TAILLE_TUILE)
}

pub(super) fn analyser(texte: &str) -> Result<Niveau, ErreurNiveau> {
    let mut lignes: Vec<&str> = texte.lines().map(str::trim_end).collect();
    while lignes.first().is_some_and(|l| l.is_empty()) {
        lignes.remove(0);
    }
    while lignes.last().is_some_and(|l| l.is_empty()) {
        lignes.pop();
    }
    if lignes.is_empty() {
        return Err(ErreurNiveau::Vide);
    }
    let largeur = lignes[0].chars().count();
    for (i, l) in lignes.iter().enumerate() {
        let n = l.chars().count();
        if n != largeur {
            return Err(ErreurNiveau::LargeurInegale {
                ligne: i + 1,
                attendu: largeur,
                trouve: n,
            });
        }
    }
    let hauteur = lignes.len();
    let mut tuiles = vec![Tuile::Vide; largeur * hauteur];
    let mut entites: Vec<Entite> = Vec::new();
    let mut panneaux = Vec::new();
    // Décor : (case, genre), vérifié une fois la grille entière lue (il faut la
    // case du dessous).
    let mut decors: Vec<(Case, GenreDecor)> = Vec::new();
    let mut departs: Vec<Case> = Vec::new();
    let mut arrivees: Vec<Case> = Vec::new();
    let mut eclairs: Vec<Case> = Vec::new();
    // Grêle : il faut la grille entière pour trouver le solide sous la colonne.
    let mut greles: Vec<Case> = Vec::new();
    let mut vent = vec![false; largeur * hauteur];
    // Vents latéraux : `<` pousse vers la gauche (-1), `>` vers la droite (+1).
    let mut vents: Vec<(Case, i8)> = Vec::new();
    let mut mobiles: BTreeMap<char, Vec<Case>> = BTreeMap::new();
    // Pigeons : il faut la grille entière pour trouver leur perchoir.
    let mut pigeons: Vec<Case> = Vec::new();
    let mut gouttes = 0usize;

    for (ty, ligne) in lignes.iter().enumerate() {
        for (tx, c) in ligne.chars().enumerate() {
            let case = (tx, ty);
            let (x, y) = px(case);
            let mut panneau = |genre| panneaux.push(Panneau { x, y, genre });
            match c {
                '.' => {}
                '#' => tuiles[ty * largeur + tx] = Tuile::Toit,
                'B' => tuiles[ty * largeur + tx] = Tuile::Brique,
                'N' => tuiles[ty * largeur + tx] = Tuile::Nuage,
                '@' => departs.push(case),
                'X' => arrivees.push(case),
                'L' => entites.push(Entite::Lanterne(Lanterne::new(x + 4.0, y))),
                'o' => {
                    entites.push(Entite::Goutte(GoutteDor::new(
                        gouttes as u8,
                        x + 4.0,
                        y + 4.0,
                    )));
                    gouttes += 1;
                }
                '^' => entites.push(Entite::Pointe(Pointe::new(x, y + 8.0))),
                'M' => entites.push(Entite::Champignon(Champignon::new(x, y + 8.0))),
                'A' => entites.push(Entite::Auvent(Auvent::dans_la_case(x, y))),
                'f' => entites.push(Entite::Ennemi(Ennemi::new(
                    TypeEnnemi::Flaque,
                    x + 2.0,
                    y + 8.0,
                ))),
                'c' => entites.push(Entite::Ennemi(Ennemi::new(
                    TypeEnnemi::Corbeau,
                    x + 2.0,
                    y + 3.0,
                ))),
                'p' => pigeons.push(case),
                'F' => entites.push(Entite::Friable(PlateformeFriable::new(Aabb::new(
                    x, y, 16.0, 8.0,
                )))),
                'e' => eclairs.push(case),
                'g' => greles.push(case),
                'v' => vent[ty * largeur + tx] = true,
                '<' => vents.push((case, -1)),
                '>' => vents.push((case, 1)),
                'D' => panneau(GenrePanneau::Deplacer),
                'S' => panneau(GenrePanneau::Sauter),
                'G' => panneau(GenrePanneau::Planer),
                'C' => panneau(GenrePanneau::Champignon),
                'W' => panneau(GenrePanneau::Vent),
                '1'..='9' => mobiles.entry(c).or_default().push(case),
                // Le reste : une lettre de décor, ou un caractère hors légende.
                _ => match GenreDecor::depuis_lettre(c) {
                    Some(genre) => decors.push((case, genre)),
                    None => {
                        return Err(ErreurNiveau::CaractereInconnu {
                            ligne: ty + 1,
                            colonne: tx + 1,
                            caractere: c,
                        })
                    }
                },
            }
        }
    }

    // --- Validation des repères ------------------------------------------
    let depart_case = match departs.as_slice() {
        [] => return Err(ErreurNiveau::DepartManquant),
        [d] => *d,
        autres => {
            return Err(ErreurNiveau::DepartMultiple {
                nombre: autres.len(),
            })
        }
    };
    let arrivee_case = match arrivees.as_slice() {
        [] => return Err(ErreurNiveau::ArriveeManquante),
        [a] => *a,
        autres => {
            return Err(ErreurNiveau::ArriveeMultiple {
                nombre: autres.len(),
            })
        }
    };
    if gouttes != GOUTTES_PAR_NIVEAU {
        return Err(ErreurNiveau::NombreDeGouttes { trouve: gouttes });
    }

    // --- Éléments qui dépendent de la grille complète ----------------------
    // Éclair : colonne depuis le premier solide AU-DESSUS du `e` (le haut de la carte s'il
    // n'y en a pas) jusqu'au premier solide en dessous. Dans un niveau d'un écran, le `e` est
    // posé sur la rangée du haut : la colonne part du haut de la carte, comme toujours. Dans
    // une grille haute, elle ne traverse pas les corniches au-dessus d'elle.
    for (k, &(tx, ty)) in eclairs.iter().enumerate() {
        let solide = |r: usize| tuiles[r * largeur + tx].est_solide();
        let premier_solide = (ty..hauteur).find(|&r| solide(r)).unwrap_or(hauteur);
        let plafond = (0..ty).rev().find(|&r| solide(r)).map_or(0, |r| r + 1);
        let haut = plafond as f32 * TAILLE_TUILE;
        let bas = (premier_solide as f32 * TAILLE_TUILE).max(haut + TAILLE_TUILE);
        let zone = Aabb::new(tx as f32 * TAILLE_TUILE + 2.0, haut, 12.0, bas - haut);
        let decalage = 30 + 47 * (k as u16 % 4);
        entites.push(Entite::Eclair(Eclair::new(zone, REPOS_ECLAIR, decalage)));
    }
    // Grêle : colonne de la case `g` elle-même (pas du haut de la carte, à la
    // différence de l'éclair) jusqu'au premier solide en dessous, ou jusqu'au bas
    // de la carte s'il n'y en a pas (rideau de grêle au-dessus d'un vide). Même
    // largeur que l'éclair : 12 px centrés dans la case (2 px de pardon par côté).
    for &(tx, ty) in &greles {
        let premier_solide = (ty..hauteur)
            .find(|&r| tuiles[r * largeur + tx].est_solide())
            .unwrap_or(hauteur);
        let haut = ty as f32 * TAILLE_TUILE;
        let bas = premier_solide as f32 * TAILLE_TUILE;
        let zone = Aabb::new(tx as f32 * TAILLE_TUILE + 2.0, haut, 12.0, bas - haut);
        entites.push(Entite::Grele(Grele::new(zone)));
    }
    // Plateformes mobiles : un chiffre = deux extrémités (ordre de lecture).
    for (marque, cases) in &mobiles {
        let [a, b] = cases.as_slice() else {
            return Err(ErreurNiveau::PlateformeMobileIncomplete {
                marque: *marque,
                trouve: cases.len(),
            });
        };
        let (pa, pb) = (px(*a), px(*b));
        let boite = Aabb::new(pa.0, pa.1, LARGEUR_MOBILE, 8.0);
        entites.push(Entite::Mobile(PlateformeMobile::new(
            boite,
            pa,
            pb,
            VITESSE_MOBILE,
        )));
    }

    // Pigeons : perchés sur la tuile du dessous (obligatoire), boîte 12 x 10
    // centrée dans la case, pattes sur le perchoir. Ils regardent du côté où le
    // perchoir s'arrête : à droite seulement si le perchoir continue à gauche et
    // s'arrête à droite (sinon à gauche, vers le départ de Pépin).
    let solide = |tx: usize, ty: usize| tuiles[ty * largeur + tx].est_solide();
    for (tx, ty) in pigeons {
        if ty + 1 >= hauteur || !solide(tx, ty + 1) {
            return Err(ErreurNiveau::PigeonSansPerchoir {
                ligne: ty + 1,
                colonne: tx + 1,
            });
        }
        let gauche = tx > 0 && solide(tx - 1, ty + 1);
        let droite = tx + 1 < largeur && solide(tx + 1, ty + 1);
        let (x, y) = px((tx, ty));
        entites.push(Entite::Ennemi(Ennemi::pigeon(
            x + 2.0,
            y + 6.0,
            gauche && !droite,
        )));
    }

    // Vents latéraux : les cases `<` et `>` forment des rectangles de même
    // sens (deux zones voisines de sens opposés restent distinctes).
    let grille_vent = |sens: i8| {
        let mut grille = vec![false; largeur * hauteur];
        for ((tx, ty), s) in &vents {
            if *s == sens {
                grille[ty * largeur + tx] = true;
            }
        }
        grille
    };
    let mut vents_laterales: Vec<VentLateral> = Vec::new();
    for sens in [-1i8, 1] {
        let grille = grille_vent(sens);
        for (g, d, h, b) in fusionner_zones(&grille, largeur, hauteur) {
            vents_laterales.push(VentLateral::new(
                Aabb::new(
                    g as f32 * TAILLE_TUILE,
                    h as f32 * TAILLE_TUILE,
                    (d - g + 1) as f32 * TAILLE_TUILE,
                    (b - h + 1) as f32 * TAILLE_TUILE,
                ),
                sens,
            ));
        }
    }

    let courants = fusionner_courants(&vent, largeur, hauteur);
    // Décor : posé sur la tuile du dessous, qui doit être solide (la dernière
    // ligne n'a pas de dessous : le décor y serait au-dessus du vide).
    let mut objets_decor = Vec::with_capacity(decors.len());
    for ((tx, ty), genre) in decors {
        let appui = ty + 1 < hauteur && tuiles[(ty + 1) * largeur + tx].est_solide();
        if !appui {
            return Err(ErreurNiveau::DecorSansAppui {
                ligne: ty + 1,
                colonne: tx + 1,
                caractere: genre.lettre(),
            });
        }
        let (x, y) = px((tx, ty));
        objets_decor.push(ObjetDecor { x, y, genre });
    }
    let (dx, dy) = px(depart_case);
    let (ax, ay) = px(arrivee_case);
    Ok(Niveau {
        largeur,
        hauteur,
        tuiles,
        // Pépin est posé au centre de la case, les pieds au bas de la case.
        depart: (
            dx + (TAILLE_TUILE - 10.0) / 2.0,
            dy + TAILLE_TUILE - JOUEUR_H,
        ),
        // Drapeau : la case `X` et celle du dessus.
        arrivee: Aabb::new(ax, ay - TAILLE_TUILE, TAILLE_TUILE, 2.0 * TAILLE_TUILE),
        entites,
        courants,
        vents: vents_laterales,
        panneaux,
        decors: objets_decor,
    })
}

/// Regroupe les cases d'un vent en rectangles : d'abord des tronçons
/// verticaux par colonne, puis fusion des colonnes voisines de même hauteur.
fn fusionner_courants(vent: &[bool], largeur: usize, hauteur: usize) -> Vec<CourantAscendant> {
    fusionner_zones(vent, largeur, hauteur)
        .into_iter()
        .map(|(g, d, h, b)| {
            CourantAscendant::new(Aabb::new(
                g as f32 * TAILLE_TUILE,
                h as f32 * TAILLE_TUILE,
                (d - g + 1) as f32 * TAILLE_TUILE,
                (b - h + 1) as f32 * TAILLE_TUILE,
            ))
        })
        .collect()
}

/// Regroupe les cases marquées `true` d'une grille en rectangles : d'abord
/// des tronçons verticaux par colonne, puis fusion des colonnes voisines de
/// même hauteur. Bornes incluses : (colonne gauche, colonne droite, ligne
/// haute, ligne basse).
fn fusionner_zones(
    zone: &[bool],
    largeur: usize,
    hauteur: usize,
) -> Vec<(usize, usize, usize, usize)> {
    let mut rects: Vec<(usize, usize, usize, usize)> = Vec::new();
    for tx in 0..largeur {
        let mut ty = 0;
        while ty < hauteur {
            if !zone[ty * largeur + tx] {
                ty += 1;
                continue;
            }
            let haut = ty;
            while ty < hauteur && zone[ty * largeur + tx] {
                ty += 1;
            }
            let bas = ty - 1;
            match rects
                .iter_mut()
                .find(|r| r.1 + 1 == tx && r.2 == haut && r.3 == bas)
            {
                Some(r) => r.1 = tx,
                None => rects.push((tx, tx, haut, bas)),
            }
        }
    }
    rects
}
