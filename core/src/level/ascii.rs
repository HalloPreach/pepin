//! Parseur ASCII -> `Niveau`.
//!
//! Étapes : 1) découper les lignes et vérifier le rectangle ; 2) lire chaque
//! caractère (tuile, entité, repère) ; 3) assembler les éléments qui dépendent
//! de la grille complète (éclairs, courants, plateformes mobiles) ; 4) valider
//! (un départ, une arrivée, trois gouttes).

use std::collections::BTreeMap;

use super::{ErreurNiveau, GenrePanneau, Niveau, Panneau, Tuile, GOUTTES_PAR_NIVEAU};
use crate::aabb::{Aabb, TAILLE_TUILE};
use crate::entities::{
    Champignon, Eclair, Ennemi, Entite, GoutteDor, Lanterne, PlateformeFriable, PlateformeMobile,
    Pointe, TypeEnnemi,
};
use crate::physics::{CourantAscendant, JOUEUR_H};

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
    let mut departs: Vec<Case> = Vec::new();
    let mut arrivees: Vec<Case> = Vec::new();
    let mut eclairs: Vec<Case> = Vec::new();
    let mut vent = vec![false; largeur * hauteur];
    let mut mobiles: BTreeMap<char, Vec<Case>> = BTreeMap::new();
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
                'F' => entites.push(Entite::Friable(PlateformeFriable::new(Aabb::new(
                    x, y, 16.0, 8.0,
                )))),
                'e' => eclairs.push(case),
                'v' => vent[ty * largeur + tx] = true,
                'D' => panneau(GenrePanneau::Deplacer),
                'S' => panneau(GenrePanneau::Sauter),
                'G' => panneau(GenrePanneau::Planer),
                'C' => panneau(GenrePanneau::Champignon),
                'W' => panneau(GenrePanneau::Vent),
                '1'..='9' => mobiles.entry(c).or_default().push(case),
                _ => {
                    return Err(ErreurNiveau::CaractereInconnu {
                        ligne: ty + 1,
                        colonne: tx + 1,
                        caractere: c,
                    })
                }
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
    // Éclair : colonne depuis le haut de la carte jusqu'au premier solide en dessous.
    for (k, &(tx, ty)) in eclairs.iter().enumerate() {
        let premier_solide = (ty..hauteur)
            .find(|&r| tuiles[r * largeur + tx].est_solide())
            .unwrap_or(hauteur);
        let bas = (premier_solide as f32 * TAILLE_TUILE).max(TAILLE_TUILE);
        let zone = Aabb::new(tx as f32 * TAILLE_TUILE + 2.0, 0.0, 12.0, bas);
        let decalage = 30 + 47 * (k as u16 % 4);
        entites.push(Entite::Eclair(Eclair::new(zone, REPOS_ECLAIR, decalage)));
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

    let courants = fusionner_courants(&vent, largeur, hauteur);
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
        panneaux,
    })
}

/// Regroupe les cases de vent en rectangles : d'abord des tronçons verticaux
/// par colonne, puis fusion des colonnes voisines de même hauteur.
fn fusionner_courants(vent: &[bool], largeur: usize, hauteur: usize) -> Vec<CourantAscendant> {
    // (colonne gauche, colonne droite, ligne haute, ligne basse), bornes incluses.
    let mut rects: Vec<(usize, usize, usize, usize)> = Vec::new();
    for tx in 0..largeur {
        let mut ty = 0;
        while ty < hauteur {
            if !vent[ty * largeur + tx] {
                ty += 1;
                continue;
            }
            let haut = ty;
            while ty < hauteur && vent[ty * largeur + tx] {
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
