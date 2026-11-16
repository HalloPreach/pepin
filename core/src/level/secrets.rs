//! Secrets (carte 8.5) : zones cachées derrière des faux murs (`m`).
//!
//! Un faux mur est une case LIBRE (aucune collision) que le jeu dessine comme le
//! solide qu'elle prolonge. Les faux murs d'un seul tenant forment une zone
//! cachée ; une case libre dont les quatre voisines sont des faux murs ou des
//! solides (au moins un faux mur) est cachée avec eux : c'est ainsi qu'une goutte
//! d'or (`o`) se cache au fond d'une salle secrète.
//!
//! Tout est calculé ici, une fois, à la lecture du niveau : le jeu n'a plus qu'à
//! savoir si Pépin touche la zone (elle se révèle) et quoi dessiner.
//!
//! Notions Rust illustrées ici : parcours en largeur avec une file
//! (`VecDeque`), tableau de booléens « déjà vu », tri par clé (`min_by_key`).

use std::collections::VecDeque;

use super::{Tuile, ZoneCachee};

/// Cases cachées d'un niveau, rangées par zone (4-connexité), dans l'ordre de
/// lecture de leur première case. `faux` : les cases `m` ; `tuiles` : la grille
/// (les faux murs y sont `Vide`).
pub(super) fn zones_cachees(
    tuiles: &[Tuile],
    largeur: usize,
    hauteur: usize,
    faux: &[(usize, usize)],
) -> Vec<ZoneCachee> {
    let i = |x: usize, y: usize| y * largeur + x;
    let mut cache = vec![false; largeur * hauteur];
    for &(x, y) in faux {
        cache[i(x, y)] = true;
    }
    // Hors carte : murs à gauche et à droite, ouvert en haut et en bas.
    let bouche = |x: i32, y: i32, cache: &[bool]| -> bool {
        if x < 0 || x >= largeur as i32 {
            return true;
        }
        if y < 0 || y >= hauteur as i32 {
            return false;
        }
        let k = i(x as usize, y as usize);
        cache[k] || tuiles[k].est_solide()
    };
    // Cases libres enfermées par des faux murs : cachées aussi (une seule passe).
    let mut enfermees = Vec::new();
    for y in 0..hauteur {
        for x in 0..largeur {
            let (xi, yi) = (x as i32, y as i32);
            let voisins = [(xi - 1, yi), (xi + 1, yi), (xi, yi - 1), (xi, yi + 1)];
            let contre_un_faux = voisins.iter().any(|&(vx, vy)| {
                vx >= 0
                    && vy >= 0
                    && (vx as usize) < largeur
                    && (vy as usize) < hauteur
                    && cache[i(vx as usize, vy as usize)]
            });
            if !cache[i(x, y)]
                && !tuiles[i(x, y)].est_solide()
                && contre_un_faux
                && voisins.iter().all(|&(vx, vy)| bouche(vx, vy, &cache))
            {
                enfermees.push(i(x, y));
            }
        }
    }
    for k in enfermees {
        cache[k] = true;
    }

    // Zones : parcours en largeur depuis chaque case cachée pas encore vue.
    let mut vue = vec![false; largeur * hauteur];
    let mut zones = Vec::new();
    for depart in 0..largeur * hauteur {
        if !cache[depart] || vue[depart] {
            continue;
        }
        vue[depart] = true;
        let mut file = VecDeque::from([depart]);
        let mut cases = Vec::new();
        while let Some(k) = file.pop_front() {
            let (x, y) = (k % largeur, k / largeur);
            cases.push((x, y));
            let voisins = [
                (x.wrapping_sub(1), y),
                (x + 1, y),
                (x, y.wrapping_sub(1)),
                (x, y + 1),
            ];
            for (vx, vy) in voisins {
                if vx < largeur && vy < hauteur && cache[i(vx, vy)] && !vue[i(vx, vy)] {
                    vue[i(vx, vy)] = true;
                    file.push_back(i(vx, vy));
                }
            }
        }
        cases.sort_by_key(|&(x, y)| (y, x));
        zones.push(zone(tuiles, largeur, hauteur, &cache, cases));
    }
    zones
}

/// Assemble une zone : l'apparence de chaque case et la case de l'indice.
fn zone(
    tuiles: &[Tuile],
    largeur: usize,
    hauteur: usize,
    cache: &[bool],
    cases: Vec<(usize, usize)>,
) -> ZoneCachee {
    let solide = |x: i32, y: i32| -> Option<Tuile> {
        let dans = x >= 0 && y >= 0 && (x as usize) < largeur && (y as usize) < hauteur;
        let t = dans.then(|| tuiles[y as usize * largeur + x as usize])?;
        t.est_solide().then_some(t)
    };
    let est_cache = |x: i32, y: i32| {
        x >= 0
            && y >= 0
            && (x as usize) < largeur
            && (y as usize) < hauteur
            && cache[y as usize * largeur + x as usize]
    };
    // Apparence : le solide qui prolonge la rangée de cases cachées (à droite,
    // sinon à gauche), sinon celui du dessous, sinon la brique.
    let apparence = |x: i32, y: i32| -> Tuile {
        let mut d = x;
        while est_cache(d + 1, y) {
            d += 1;
        }
        let mut g = x;
        while est_cache(g - 1, y) {
            g -= 1;
        }
        solide(d + 1, y)
            .or_else(|| solide(g - 1, y))
            .or_else(|| solide(x, y + 1))
            .unwrap_or(Tuile::Brique)
    };
    let cases: Vec<(i32, i32, Tuile)> = cases
        .into_iter()
        .map(|(x, y)| {
            let (x, y) = (x as i32, y as i32);
            (x, y, apparence(x, y))
        })
        .collect();
    // Indice : une case d'entrée (voisine, à gauche ou à droite, d'une case libre
    // non cachée) qui imite une brique coiffée d'une autre case pleine ; la plus
    // basse, puis la plus à gauche.
    let indice = cases
        .iter()
        .filter(|&&(x, y, t)| {
            let libre = |vx: i32| solide(vx, y).is_none() && !est_cache(vx, y) && vx >= 0;
            t == Tuile::Brique
                && (libre(x - 1) || libre(x + 1))
                && (solide(x, y - 1).is_some() || est_cache(x, y - 1))
        })
        .min_by_key(|&&(x, y, _)| (-y, x))
        .map(|&(x, y, _)| (x, y));
    ZoneCachee { cases, indice }
}
