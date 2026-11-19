//! Le FOND des égouts (carte 9.27b, AGA : « le fond est bizarre toujours dans les
//! égouts il y a une espèce de route et des voûtes bizarres ») : un vrai collecteur
//! d'égout, sobre et plus sombre que les tuiles du premier plan. Plus de bandes de
//! ciel, de tirets de tuyaux ni de bosses au sol : rien qui fasse route ou dehors.
//!
//! Deux plans de parallaxe, comme les autres thèmes :
//! 1. Le MUR du tunnel (plan lointain, `FACTEUR_MUR`) : un mur d'ardoise nuit percé
//!    de grandes arches PLEINES qui se répètent à écart régulier (`PERIODE_ARCHE`),
//!    leurs claveaux, la clé, les pierres d'angle des piédroits ; dans chaque bouche,
//!    une seconde arche plus loin (la galerie qui s'enfonce) ; des coulures
//!    d'humidité sur les piliers (une goutte tombe de temps en temps), une bouche
//!    d'évacuation grillagée sur un pilier sur trois, et l'eau sombre au pied des
//!    arches, où se reflètent les piliers.
//! 2. Les TUYAUX (plan du milieu, `FACTEUR_TUYAUX`) : de gros tuyaux qui sortent du
//!    plafond, longent le haut du tunnel puis plongent dans l'eau, avec leurs colliers.
//!
//! Trois tons d'ardoise (`ARDOISE_OMBRE`, `ARDOISE_NUIT`, `ARDOISE`) et quelques
//! reflets cyan sombre (`PLUIE_SOMBRE`) sur l'eau. Ni ciel, ni pluie. Rendu seul :
//! aucune collision ; la grille et la lumière du départ (`decor_egouts`) restent.
//! La géométrie (`ouverture`, `anneau`, `tuyau`) est pure et testée.

use macroquad::prelude::draw_rectangle;
use pepin_core::palette::{Couleur, ARDOISE, ARDOISE_NUIT, ARDOISE_OMBRE, PLUIE_SOMBRE};
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use crate::decor::{decalage, enfoncement};
use crate::rendu::couleur_mq;

const L: i32 = LARGEUR_LOGIQUE as i32;
const H: i32 = HAUTEUR_LOGIQUE as i32;

/// Parallaxe horizontale et verticale du mur (celles de la couche lointaine).
const FACTEUR_MUR: f32 = 0.15;
const VERT_MUR: f32 = 0.05;
/// Parallaxe des tuyaux (celles des îlots, entre le mur et le niveau).
const FACTEUR_TUYAUX: f32 = 0.25;
const VERT_TUYAUX: f32 = 0.07;

/// Écart entre deux arches (px, centre à centre).
pub(crate) const PERIODE_ARCHE: i32 = 112;
/// Rayon de la bouche d'une arche (demi-largeur de l'ouverture).
pub(crate) const RAYON: i32 = 26;
/// Épaisseur de l'anneau de claveaux autour de la bouche.
pub(crate) const CLAVEAU: i32 = 6;
/// Ordonnée écran de la naissance de l'arc (le centre du plein cintre).
pub(crate) const NAISSANCE: i32 = 64;
/// Ordonnée écran de la surface de l'eau, au pied des arches.
pub(crate) const EAU: i32 = 118;
/// Nombre de claveaux de l'anneau (impair : la clé est au milieu).
const NB_CLAVEAUX: i32 = 9;
/// La seconde arche, au fond de la bouche : rayon et épaisseur.
const RAYON_FOND: i32 = 17;
const ANNEAU_FOND: i32 = 3;

/// Largeur d'une cellule de tuyaux (px) et épaisseur d'un tuyau.
const CELLULE_TUYAU: i32 = 176;
pub(crate) const DIAMETRE: i32 = 6;
/// Écart entre deux colliers le long d'un tuyau.
const PAS_COLLIER: i32 = 24;

fn rect(x: i32, y: i32, w: i32, h: i32, c: Couleur) {
    if w > 0 && h > 0 {
        draw_rectangle(x as f32, y as f32, w as f32, h as f32, couleur_mq(c));
    }
}

/// Nombre pseudo-aléatoire stable dérivé d'un entier.
fn hache(n: i32, graine: u32) -> u32 {
    let mut h = (n as u32).wrapping_mul(0x9E37_79B1) ^ graine.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h
}

/// Demi-largeur d'un disque de rayon `r` à `d` px de son centre (verticalement),
/// ou `None` hors du disque.
fn demi_corde(r: i32, d: i32) -> Option<i32> {
    (d.abs() <= r).then(|| ((r * r - d * d) as f32).sqrt() as i32)
}

/// Demi-largeur de la bouche d'une arche à l'ordonnée écran `y` : un plein cintre
/// au-dessus de la naissance, puis les piédroits verticaux jusqu'à l'eau. `None`
/// au-dessus de la bouche ou sous l'eau. Pure.
pub(crate) fn ouverture(y: i32) -> Option<i32> {
    if y >= EAU {
        None
    } else if y >= NAISSANCE {
        Some(RAYON)
    } else {
        demi_corde(RAYON, NAISSANCE - y)
    }
}

/// Demi-largeur EXTÉRIEURE de l'anneau de claveaux à l'ordonnée `y` (au-dessus de
/// la naissance seulement), ou `None`. L'anneau va de `ouverture(y)` (ou 0 au-dessus
/// de la bouche) à cette valeur : une bande pleine, pas un simple contour. Pure.
pub(crate) fn anneau(y: i32) -> Option<i32> {
    if y >= NAISSANCE {
        None
    } else {
        demi_corde(RAYON + CLAVEAU, NAISSANCE - y)
    }
}

/// Un tuyau de la cellule `cellule` : abscisses (repère du plan des tuyaux) de la
/// colonne qui descend du plafond et de celle qui plonge dans l'eau, ordonnée du
/// haut de la longueur horizontale. `None` : une cellule sur quatre n'en a pas. Pure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Tuyau {
    pub depart: i32,
    pub plongee: i32,
    pub y: i32,
}

pub(crate) fn tuyau(cellule: i32) -> Option<Tuyau> {
    let h = hache(cellule, 0xE60);
    if h.is_multiple_of(4) {
        return None;
    }
    let depart = cellule * CELLULE_TUYAU + 8 + (h >> 3) as i32 % 24;
    let longueur = 64 + (h >> 9) as i32 % 64;
    Some(Tuyau {
        depart,
        plongee: depart + longueur,
        y: if (h >> 16).is_multiple_of(2) { 14 } else { 20 },
    })
}

/// Dessine le fond des égouts. `cam_y` : la caméra vue par le fond
/// (`decor::camera_fond`), pour la parallaxe verticale (nulle dans un niveau d'un
/// écran de haut, comme les égouts).
pub fn dessiner(cam_x: i32, cam_y: i32, pas: u32) {
    dessiner_mur(cam_x, enfoncement(cam_y, VERT_MUR), pas);
    dessiner_tuyaux(cam_x, enfoncement(cam_y, VERT_TUYAUX));
}

/// Le mur du tunnel, ses arches, ses coulures, ses grilles et l'eau à son pied.
fn dessiner_mur(cam_x: i32, vert: i32, pas: u32) {
    rect(0, 0, L, H, ARDOISE_NUIT);
    let dec = decalage(cam_x, FACTEUR_MUR);
    let premiere = dec.div_euclid(PERIODE_ARCHE) - 1;
    let derniere = (dec + L).div_euclid(PERIODE_ARCHE) + 1;
    let eau = EAU + vert;
    for k in premiere..=derniere {
        let cx = k * PERIODE_ARCHE + PERIODE_ARCHE / 2 - dec;
        dessiner_arche(cx, vert);
        dessiner_pilier(k, cx + PERIODE_ARCHE / 2, vert, pas);
    }
    // L'eau sombre au pied des arches : le reflet tremblé de chaque pilier (des
    // traits courts d'ardoise nuit, décalés d'une rangée à l'autre : jamais des
    // marches), quelques reflets cyan sombre.
    rect(0, eau, L, H - eau, ARDOISE_OMBRE);
    for k in premiere..=derniere {
        let cx = k * PERIODE_ARCHE + PERIODE_ARCHE / 2 - dec;
        let px = cx + PERIODE_ARCHE / 2;
        for (i, y) in (eau + 2..eau + 26).step_by(4).enumerate() {
            let h = hache(k * 16 + i as i32, 0x0DE);
            let onde = ((pas as i32 / 10 + i as i32) % 3) - 1;
            let w = 6 + (h % 10) as i32;
            let x = px - 14 + ((h >> 5) % 16) as i32 + onde;
            rect(x, y, w, 1, ARDOISE_NUIT);
        }
        for r in 0..2 {
            let h = hache(k * 2 + r, 0xEA0);
            let x = cx - RAYON + (h % (2 * RAYON as u32)) as i32;
            let y = eau + 3 + ((h >> 8) % 8) as i32;
            if (pas / 12 + h % 5).is_multiple_of(3) {
                rect(x, y, 3, 1, PLUIE_SOMBRE);
            }
        }
    }
}

/// Une arche centrée en `cx` : l'anneau de claveaux (plein) et sa clé, les pierres
/// d'angle des piédroits, la bouche sombre et, au fond, la seconde arche.
fn dessiner_arche(cx: i32, vert: i32) {
    let haut = NAISSANCE - RAYON - CLAVEAU;
    for y in haut..NAISSANCE {
        let Some(ext) = anneau(y) else { continue };
        match ouverture(y) {
            Some(o) => {
                rect(cx - ext, y + vert, ext - o, 1, ARDOISE);
                rect(cx + o, y + vert, ext - o, 1, ARDOISE);
            }
            None => rect(cx - ext, y + vert, 2 * ext, 1, ARDOISE),
        }
    }
    // Les joints rayonnants entre les claveaux, et la clé qui dépasse en haut.
    for j in 1..NB_CLAVEAUX {
        let a = std::f32::consts::PI * j as f32 / NB_CLAVEAUX as f32;
        for r in RAYON..RAYON + CLAVEAU {
            let x = cx + (r as f32 * a.cos()).round() as i32;
            let y = NAISSANCE - (r as f32 * a.sin()).round() as i32;
            rect(x, y + vert, 1, 1, ARDOISE_NUIT);
        }
    }
    rect(cx - 3, haut - 2 + vert, 6, 2, ARDOISE);
    // Les pierres d'angle des piédroits, alternées (5 px, puis 3 px).
    for (i, y) in (NAISSANCE..EAU).step_by(6).enumerate() {
        let w = if i % 2 == 0 { 5 } else { 3 };
        let h = 5.min(EAU - y);
        rect(cx - RAYON - w, y + vert, w, h, ARDOISE);
        rect(cx + RAYON, y + vert, w, h, ARDOISE);
    }
    // La bouche, et la galerie qui s'enfonce : une seconde arche plus petite.
    for y in NAISSANCE - RAYON..EAU {
        if let Some(o) = ouverture(y) {
            rect(cx - o, y + vert, 2 * o, 1, ARDOISE_OMBRE);
        }
        let d = NAISSANCE + 6 - y;
        let ext = if d > 0 {
            demi_corde(RAYON_FOND + ANNEAU_FOND, d)
        } else {
            Some(RAYON_FOND + ANNEAU_FOND)
        };
        let int = if d > 0 {
            demi_corde(RAYON_FOND, d)
        } else {
            Some(RAYON_FOND)
        };
        match (ext, int) {
            (Some(e), Some(i)) => {
                rect(cx - e, y + vert, e - i, 1, ARDOISE_NUIT);
                rect(cx + i, y + vert, e - i, 1, ARDOISE_NUIT);
            }
            (Some(e), None) => rect(cx - e, y + vert, 2 * e, 1, ARDOISE_NUIT),
            _ => {}
        }
    }
}

/// Le pilier `k` (entre l'arche `k` et la suivante), centré en `px` : deux coulures
/// d'humidité (une goutte en tombe de temps en temps) et, un pilier sur trois, une
/// bouche d'évacuation grillagée au ras de l'eau, d'où coule un filet.
fn dessiner_pilier(k: i32, px: i32, vert: i32, pas: u32) {
    let large = PERIODE_ARCHE - 2 * (RAYON + CLAVEAU) - 12;
    for j in 0..2 {
        let h = hache(k * 2 + j, 0xC01);
        let x = px - large / 2 + (h % large as u32) as i32;
        let y = 36 + ((h >> 6) % 24) as i32;
        let long = 16 + ((h >> 11) % 36) as i32;
        let long = long.min(EAU - y - 2);
        rect(x - 1, y + vert, 3, 2, ARDOISE_OMBRE);
        rect(x, y + vert, 1, long / 2, ARDOISE_OMBRE);
        rect(
            x + 1,
            y + long / 2 + vert,
            1,
            long - long / 2,
            ARDOISE_OMBRE,
        );
        // La goutte : elle tombe du bas de la coulure à l'eau, une fois par cycle.
        let cycle = 150 + (h >> 20) % 60;
        let t = ((pas + h % cycle) % cycle) as i32;
        let gy = y + long + t * 2;
        if gy < EAU {
            rect(x + 1, gy + vert, 1, 2, ARDOISE);
        }
    }
    if hache(k, 0x6B1).is_multiple_of(3) {
        let (gx, gy) = (px - 7, EAU - 14);
        rect(gx, gy + vert, 14, 10, ARDOISE_OMBRE);
        rect(gx, gy + vert, 14, 1, ARDOISE);
        for b in 0..3 {
            rect(gx + 3 + b * 4, gy + 2 + vert, 1, 8, ARDOISE);
        }
        // Le filet qui s'en échappe : des pixels qui descendent jusqu'à l'eau.
        for i in 0..2 {
            let y = gy + 10 + ((pas as i32 / 3 + i * 2) % 4);
            rect(gx + 6, y + vert, 2, 1, ARDOISE);
        }
    }
}

/// Les gros tuyaux : chacun sort du plafond, longe le haut du tunnel, puis plonge
/// dans l'eau ; des colliers à intervalle régulier, une ombre sur un bord.
fn dessiner_tuyaux(cam_x: i32, vert: i32) {
    let dec = decalage(cam_x, FACTEUR_TUYAUX);
    let premiere = dec.div_euclid(CELLULE_TUYAU) - 1;
    let derniere = (dec + L).div_euclid(CELLULE_TUYAU) + 1;
    for c in premiere..=derniere {
        let Some(t) = tuyau(c) else { continue };
        let (a, b, y) = (t.depart - dec, t.plongee - dec, t.y + vert);
        let d = DIAMETRE;
        // La descente du plafond, la longueur horizontale, la plongée.
        rect(a, 0, d, y, ARDOISE);
        rect(a + d - 1, 0, 1, y, ARDOISE_OMBRE);
        rect(a, y, b - a + d, d, ARDOISE);
        rect(a, y + d - 1, b - a + d, 1, ARDOISE_OMBRE);
        rect(b, y + d, d, EAU + vert - y - d + 2, ARDOISE);
        rect(
            b + d - 1,
            y + d - 1,
            1,
            EAU + vert - y - d + 3,
            ARDOISE_OMBRE,
        );
        // Les colliers (plus larges que le tuyau d'un pixel de chaque côté).
        for yc in (y - PAS_COLLIER / 2..y).rev().step_by(PAS_COLLIER as usize) {
            rect(a - 1, yc, d + 2, 2, ARDOISE_OMBRE);
        }
        for xc in (a + PAS_COLLIER / 2..b).step_by(PAS_COLLIER as usize) {
            rect(xc, y - 1, 2, d + 2, ARDOISE_OMBRE);
        }
        for yc in (y + d + PAS_COLLIER / 2..EAU + vert).step_by(PAS_COLLIER as usize) {
            rect(b - 1, yc, d + 2, 2, ARDOISE_OMBRE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les arches sont PLEINES : la bouche est ouverte sur toute sa hauteur (du
    /// sommet du cintre jusqu'à l'eau, sans jamais se refermer en descendant), et
    /// l'anneau de claveaux l'entoure d'une bande d'au moins 3 px de large au-dessus
    /// de la naissance (pas un contour d'un pixel).
    #[test]
    fn les_arches_sont_pleines() {
        let haut = NAISSANCE - RAYON;
        let o: Vec<i32> = (haut..EAU).map(|y| ouverture(y).unwrap()).collect();
        assert!(o.windows(2).all(|p| p[0] <= p[1]), "{o:?}");
        assert_eq!(*o.last().unwrap(), RAYON);
        assert!(ouverture(haut - 1).is_none() && ouverture(EAU).is_none());
        for y in haut + 4..NAISSANCE {
            let (e, i) = (anneau(y).unwrap(), ouverture(y).unwrap());
            assert!(e - i >= 3, "y={y} : anneau de {} px", e - i);
        }
    }

    // Un pilier plein entre deux arches voisines (écart régulier), et les arches
    // descendent jusqu'à l'eau, sous la ligne du sol du niveau.
    const _: () = assert!(PERIODE_ARCHE - 2 * (RAYON + CLAVEAU + 5) >= 30);
    const _: () = assert!(NAISSANCE - RAYON - CLAVEAU > 24 && EAU > 112);

    /// Les tuyaux sont stables, gros, longent le haut du tunnel au-dessus des clés
    /// des arches et ne se chevauchent pas d'une cellule à l'autre.
    #[test]
    fn les_tuyaux_restent_en_haut_et_ne_se_chevauchent_pas() {
        let mut avec = 0;
        for c in -60..60 {
            assert_eq!(tuyau(c), tuyau(c));
            if let Some(t) = tuyau(c) {
                avec += 1;
                assert!(t.y + DIAMETRE < NAISSANCE - RAYON - CLAVEAU - 2, "{t:?}");
                assert!(t.plongee - t.depart >= 64, "{t:?}");
                assert!(t.depart >= c * CELLULE_TUYAU);
                assert!(t.plongee + DIAMETRE + 2 < (c + 1) * CELLULE_TUYAU, "{t:?}");
            }
        }
        assert!((60..=110).contains(&avec), "{avec} tuyaux sur 120 cellules");
    }
}
