//! Le fond de la RUE du caniveau (carte 9.6b) : une rangée de maisons vues du
//! trottoir (façades, fenêtres allumées, vitrines sous leur store, descentes de
//! gouttière) et, devant, les lampadaires et leur halo.
//!
//! Carte 10.4d (suite) : ce qui est FIXE est dans des fichiers de
//! `assets/fonds/caniveau`, un PNG par couche :
//! - `maisons.png` : UNE période de la rue (les huit maisons de `RUE`, 496 px de
//!   large, du faîte le plus haut jusqu'au bas du trottoir d'en face), toutes ses
//!   fenêtres ÉTEINTES ; le code la répète ;
//! - `lampadaire.png` : UN lampadaire (halo, mât, crosse, lanterne), que le code
//!   pose deux fois sur la largeur de l'écran.
//!
//! Ce qui est CALCULÉ reste ici : les fenêtres ALLUMÉES (`Plan::Fenetres`). Elles
//! ne se répètent pas avec la rue : chaque tour de la rue a son numéro, et la même
//! fenêtre est allumée à un tour, éteinte au suivant (`allumee`). Elles sont donc
//! peintes par-dessus les vitres éteintes du PNG, carreau par carreau (la croisée
//! du PNG reste visible entre eux).
//!
//! Carte 9.20g (« c'est dur de savoir qu'est-ce qu'est un obstacle comparé au
//! fond ») : tout ce fond est SOMBRE et TERNE. Il n'emploie que l'ardoise d'ombre,
//! l'ardoise de nuit, l'ardoise et le violet d'orage : aucun
//! contour sombre net, aucune couleur vive, aucun dessus plat clair (ni corniche,
//! ni appui de fenêtre, ni faîte). Les teintes claires restent au premier plan, à
//! ce qu'on touche (pavés, mobilier de rue), cerné de sombre et liseré de clair.
//!
//! Carte 9.20h : les descentes de gouttière `J` du niveau sont dessinées ici aussi
//! (`dessiner_gouttieres`, `dessiner_filets`), chacune accrochée à son pan de
//! maison : chéneau, coude, colliers, dauphin qui verse dans le caniveau. Elles
//! dépendent de la grille du niveau : elles restent du code.

use macroquad::prelude::draw_rectangle;
use pepin_core::aabb::TAILLE_TUILE;
use pepin_core::level::GenreDecor;
use pepin_core::palette::{Couleur, ARDOISE, ARDOISE_NUIT, ARDOISE_OMBRE, VIOLET_ORAGE};

use pepin_core::LARGEUR_LOGIQUE;

use crate::monde::Monde;
use crate::rendu::couleur_mq;

const L: i32 = LARGEUR_LOGIQUE as i32;

fn rect(x: i32, y: i32, w: i32, h: i32, c: Couleur) {
    draw_rectangle(x as f32, y as f32, w as f32, h as f32, couleur_mq(c));
}

/// La rue : huit maisons, de gauche à droite, qui se répètent : (largeur en px,
/// multiple de 16, une fenêtre par tranche de 16 px ; étages au-dessus du
/// rez-de-chaussée). Leur dessin est dans `caniveau/maisons.png` ; le code ne garde
/// que ce qu'il faut pour retrouver les fenêtres à allumer.
const RUE: [(i32, i32); 8] = [
    (64, 3),
    (48, 2),
    (80, 4),
    (48, 3),
    (64, 2),
    (64, 4),
    (48, 3),
    (80, 3),
];

/// Hauteur du rez-de-chaussée (px) et d'un étage.
const REZ: i32 = 20;
const ETAGE: i32 = 16;
/// Hauteur du toit en mansarde (px).
const TOIT: i32 = 8;
/// Hauteur de `maisons.png` au-dessus du trottoir (le faîte de la plus haute
/// maison, quatre étages) ; le fichier descend ensuite de `TROTTOIR` px sous le sol.
const HAUT_MAISONS: i32 = REZ + 4 * ETAGE + TOIT;
/// Le trottoir d'en face, sous le sol (vérifié par les tests : le dessin n'en a pas
/// besoin, il est dans le fichier).
#[cfg(test)]
const TROTTOIR: i32 = 62;
/// Le lampadaire du fichier : son mât (abscisse `x` du code) est à la colonne
/// `LAMPADAIRE_X` du PNG (le halo déborde à gauche), et le fichier commence
/// `LAMPADAIRE_HAUT` px au-dessus du sol (le haut du halo).
const LAMPADAIRE_X: i32 = 6;
const LAMPADAIRE_HAUT: i32 = 51;
/// Les deux lampadaires de l'écran (abscisse du mât avant la parallaxe).
const LAMPADAIRES: [i32; 2] = [60, 230];

/// Longueur (px) d'un tour de la rue : la somme des largeurs (la largeur de
/// `maisons.png`).
fn periode() -> i32 {
    RUE.iter().map(|&(largeur, _)| largeur).sum()
}

/// Fenêtre allumée ? Fixe (même maison, même fenêtre : même réponse), environ une
/// sur quatre : la rue dort, le fond reste plus sombre que le niveau.
fn allumee(maison: i32, etage: i32, colonne: i32) -> bool {
    let h = (maison as u32).wrapping_mul(73)
        ^ (etage as u32).wrapping_mul(151)
        ^ (colonne as u32).wrapping_mul(37);
    (h.wrapping_mul(0x2C1B_3C6D) >> 7).is_multiple_of(4)
}

/// La vitre d'une fenêtre : faiblement allumée (violet d'orage, jamais l'orange
/// vif du premier plan) ou éteinte.
fn vitre(maison: i32, etage: i32, colonne: i32, eteinte: Couleur) -> Couleur {
    if allumee(maison, etage, colonne) {
        VIOLET_ORAGE
    } else {
        eteinte
    }
}

/// Les plans de la rue, du plus loin au plus près (carte 10.4d, suite). Les
/// maisons et le lampadaire sont des fichiers de `assets/fonds/caniveau` (FIXES) ;
/// les fenêtres allumées sont CALCULÉES. `decor::pile` les place : les maisons et
/// leurs fenêtres entre les deux couches de silhouettes, les lampadaires devant la
/// couche proche.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    /// Les maisons, fenêtres éteintes, et le trottoir d'en face :
    /// `caniveau/maisons.png`, UNE période de la rue, répétée.
    Maisons,
    /// Les fenêtres allumées, qui changent d'un tour de la rue à l'autre.
    Fenetres,
    /// Les lampadaires : `caniveau/lampadaire.png`, posé deux fois.
    Lampadaires,
}

impl Plan {
    /// Le nom du fichier de `assets/fonds/caniveau` (`None` : plan calculé).
    #[cfg(test)]
    pub fn fichier(self) -> Option<&'static str> {
        match self {
            Plan::Maisons => Some("maisons"),
            Plan::Lampadaires => Some("lampadaire"),
            Plan::Fenetres => None,
        }
    }

    /// Le nom du plan dans `assets/fonds/LISEZMOI.md`.
    #[cfg(test)]
    pub fn nom(self) -> &'static str {
        match self {
            Plan::Maisons => "maisons",
            Plan::Fenetres => "fenêtres allumées",
            Plan::Lampadaires => "lampadaires",
        }
    }
}

/// Dessine un plan de la rue posé sur la ligne `sol` (px, à l'écran : le haut du
/// trottoir d'en face), décalé de `dec` px par la parallaxe.
pub(crate) fn dessiner_plan(plan: Plan, dec: i32, sol: i32) {
    match plan {
        Plan::Maisons => {
            let p = periode();
            let tour0 = dec.div_euclid(p) - 1;
            for tour in tour0..=tour0 + 2 {
                let x = tour * p - dec;
                crate::fonds::dessiner("caniveau", "maisons", x, sol - HAUT_MAISONS);
            }
        }
        Plan::Fenetres => dessiner_fenetres(dec, sol),
        // Chaque lampadaire en UN exemplaire, modulo la largeur de l'écran : coupé
        // au bord droit, sans copie à gauche.
        Plan::Lampadaires => {
            for lx in LAMPADAIRES {
                let x = (lx - dec).rem_euclid(L);
                crate::fonds::dessiner(
                    "caniveau",
                    "lampadaire",
                    x - LAMPADAIRE_X,
                    sol - LAMPADAIRE_HAUT,
                );
            }
        }
    }
}

/// Les fenêtres allumées de la rue : `(x, y, largeur, hauteur)` de chaque carreau
/// violet, pour un décalage `dec` et un sol `sol`. Chaque tour de la rue a son
/// numéro (`tour * 8 + maison`) : c'est lui qui choisit les fenêtres allumées.
/// Une vitre allumée est peinte en quatre carreaux, autour de la croisée du PNG ;
/// la lucarne du toit, d'un seul carreau. Pure.
fn carreaux_allumes(dec: i32, sol: i32) -> Vec<(i32, i32, i32, i32)> {
    let p = periode();
    let tour0 = dec.div_euclid(p) - 1;
    let mut r = Vec::new();
    for tour in tour0..=tour0 + 2 {
        let mut x = tour * p - dec;
        for (k, &(w, etages)) in RUE.iter().enumerate() {
            let numero = tour * 8 + k as i32;
            if x < L && x + w > 0 {
                let haut = sol - REZ - etages * ETAGE;
                if allumee(numero, 9, 0) {
                    r.push((x + w / 2 - 2, haut - TOIT + 3, 4, 4));
                }
                for e in 0..etages {
                    let y = haut + 4 + e * ETAGE;
                    for c in 0..w / 16 {
                        if allumee(numero, e, c) {
                            let fx = x + c * 16 + 5;
                            r.push((fx, y + 2, 2, 3));
                            r.push((fx + 3, y + 2, 3, 3));
                            r.push((fx, y + 6, 2, 5));
                            r.push((fx + 3, y + 6, 3, 5));
                        }
                    }
                }
            }
            x += w;
        }
    }
    r
}

fn dessiner_fenetres(dec: i32, sol: i32) {
    for (x, y, w, h) in carreaux_allumes(dec, sol) {
        rect(x, y, w, h, VIOLET_ORAGE);
    }
}

/// Un rectangle à dessiner : `(x, y, largeur, hauteur, couleur)`.
type Rect = (i32, i32, i32, i32, Couleur);

/// Largeur (px) du pan de mur derrière une descente de gouttière (trois fenêtres).
const PAN: i32 = 48;
/// Bord gauche du pan de mur, depuis le bord gauche de la case `J`.
const PAN_X: i32 = -18;
/// Le tuyau (3 px de large), depuis le bord gauche de la case `J`.
const TUYAU_X: i32 = 26;
/// Le bec du dauphin (dernier pixel), depuis le bord gauche de la case `J` : au-dessus
/// de la 1re case d'eau (la lettre `J` se pose deux cases avant l'eau).
const BEC_X: i32 = 39;
/// Le dessous du dauphin, au-dessus du trottoir (px) : l'eau tombe de là.
const DAUPHIN_BAS: i32 = 8;

/// Carte 9.20h (« les gouttières sont rattachées à rien ») : la descente de
/// gouttière `J` du caniveau, ACCROCHÉE à une maison. Le fond défile plus lentement
/// que le niveau, ses maisons ne sont jamais au même endroit derrière le tuyau :
/// chaque `J` a donc son propre pan de façade, posé dans le niveau, qui prolonge la
/// rue du fond jusqu'au caniveau (mêmes tons sombres et ternes, toit en pente, sans
/// dessus plat clair). Sur ce mur : le CHÉNEAU au bord du toit, son COUDE, le tuyau
/// fixé par trois COLLIERS, et en bas le DAUPHIN, coudé vers la rue, au-dessus de la
/// 1re case d'eau. Rien de tout cela n'a de collision. `(x, sol)` : bord gauche de
/// la case `J` et haut du trottoir sous elle (px, à l'écran) ; `numero` choisit les
/// fenêtres allumées.
fn gouttiere_accrochee(x: i32, sol: i32, numero: i32) -> Vec<Rect> {
    let x0 = x + PAN_X;
    let haut = sol - REZ - 2 * ETAGE;
    let mut r: Vec<Rect> = vec![(x0, haut, PAN, sol - haut, ARDOISE_NUIT)];
    // Le toit en mansarde, en pente (aucun dessus plat), et sa lucarne.
    for k in 0..TOIT {
        r.push((x0 + 1 + k, haut - 1 - k, PAN - 2 - 2 * k, 1, ARDOISE_OMBRE));
    }
    let lucarne = vitre(numero, 9, 0, ARDOISE_NUIT);
    r.push((x0 + PAN / 2 - 4, haut - TOIT + 3, 4, 4, lucarne));
    // Deux étages de fenêtres (comme les maisons du fond), une porte cochère.
    for e in 0..2 {
        let y = haut + 4 + e * ETAGE;
        for c in 0..PAN / 16 {
            let fx = x0 + c * 16 + 5;
            r.push((fx, y + 2, 6, 9, vitre(numero, e, c, ARDOISE_OMBRE)));
            r.push((fx + 2, y + 2, 1, 9, ARDOISE_NUIT));
            r.push((fx, y + 5, 6, 1, ARDOISE_NUIT));
        }
    }
    r.push((x0, sol - REZ - 1, PAN, 1, ARDOISE_OMBRE));
    r.push((x0 + 6, sol - 18, 12, 18, ARDOISE_OMBRE));
    r.push((x0 + 12, sol - 14, 1, 14, ARDOISE_NUIT));
    // Le chéneau : la gouttière pendue au bord du toit, d'un bout à l'autre du pan,
    // qui dépasse à droite. Lèvre sombre DESSUS : rien de clair où se poser.
    // Mince (un seul pixel de zinc entre deux ombres) : pas une corniche.
    r.push((x0, haut, PAN + 2, 1, ARDOISE_OMBRE));
    r.push((x0, haut + 1, PAN + 2, 1, ARDOISE));
    r.push((x0, haut + 2, PAN + 2, 1, ARDOISE_OMBRE));
    // Le coude (col de cygne) : sous le bout du chéneau, il ramène l'eau vers le mur.
    let t = x + TUYAU_X;
    r.push((t + 2, haut + 3, 2, 2, ARDOISE));
    r.push((t + 4, haut + 3, 1, 2, ARDOISE_OMBRE));
    r.push((t + 1, haut + 5, 2, 2, ARDOISE));
    r.push((t + 3, haut + 5, 1, 2, ARDOISE_OMBRE));
    // Le tuyau, contre le mur (ombre portée à droite) jusqu'au dauphin.
    let bas = sol - DAUPHIN_BAS - 3;
    r.push((t, haut + 7, 2, bas - haut - 7, ARDOISE));
    r.push((t + 2, haut + 7, 1, bas - haut - 7, ARDOISE_OMBRE));
    // Trois colliers sombres (une bande courte, pas des barreaux d'échelle).
    for y in [haut + 12, haut + 24, haut + 36] {
        r.push((t - 1, y, 5, 2, ARDOISE_OMBRE));
    }
    // Le dauphin : le pied du tuyau se coude vers la rue, au-dessus du caniveau.
    // Son dessus est une ombre, comme le chéneau : pas une marche claire.
    r.push((t, bas, 2, 1, ARDOISE));
    r.push((t + 2, bas, x + BEC_X - 1 - t, 1, ARDOISE_OMBRE));
    r.push((t, bas + 1, x + BEC_X + 1 - t, 1, ARDOISE));
    r.push((t, bas + 2, x + BEC_X + 1 - t, 1, ARDOISE_OMBRE));
    r.push((x + BEC_X, bas, 1, 3, ARDOISE_OMBRE));
    r
}

/// Le filet d'eau sous le bec du dauphin, deux images : de `x + FILET_X`, sur
/// `DAUPHIN_BAS` rangées jusqu'à la surface de l'eau. Le filet blanc d'écume cerné
/// de sombre (celui de la carte 9.20a) ; au point d'arrivée, sur l'eau, de petites
/// éclaboussures CYAN (`D`, la pluie) qui sautent de part et d'autre. Chiffres : la
/// palette (`sprites::pixel`).
const FILET: [[&str; DAUPHIN_BAS as usize]; 2] = [
    [
        "..0760..", "..0670..", "..0760..", "..0760..", "..0670..", "..0760..", ".D0760D.",
        "D.0DD0.D",
    ],
    [
        "..0670..", "..0760..", "..0670..", "..0670..", "..0760..", "..0670..", "D.0670.D",
        ".D0DD0D.",
    ],
];
/// Bord gauche du dessin du filet, depuis le bord gauche de la case `J`.
const FILET_X: i32 = 34;

/// Le filet d'eau qui sort du dauphin et tombe dans le caniveau, image du pas `pas`
/// (tous les 10 pas, comme à la carte 9.20a).
fn filet_de_gouttiere(x: i32, sol: i32, pas: u32) -> Vec<Rect> {
    let image = &FILET[((pas / 10) % 2) as usize];
    let mut r: Vec<Rect> = Vec::new();
    for (dy, ligne) in image.iter().enumerate() {
        for (dx, c) in ligne.chars().enumerate() {
            if let Ok(Some(couleur)) = crate::sprites::pixel(c) {
                let y = sol - DAUPHIN_BAS + dy as i32;
                r.push((x + FILET_X + dx as i32, y, 1, 1, couleur));
            }
        }
    }
    r
}

/// Les descentes de gouttière `J` du niveau, à l'écran : `(x, sol)` de chacune
/// (bord gauche de sa case, haut du trottoir sous elle), et un numéro fixe.
fn gouttieres_a_l_ecran(monde: &Monde, cx: i32, cy: i32) -> Vec<(i32, i32, i32)> {
    let t = TAILLE_TUILE as i32;
    monde
        .niveau
        .decors
        .iter()
        .filter(|d| d.genre == GenreDecor::Gouttiere)
        .map(|d| {
            let (x, y) = (d.x.round() as i32, d.y.round() as i32);
            (x - cx, y + t - cy, x / t)
        })
        .filter(|&(x, _, _)| x + PAN_X + PAN + 16 > 0 && x + PAN_X < L)
        .collect()
}

/// Première passe, avant les tuiles et le mobilier (une voiture garée passe
/// devant) : chaque descente de gouttière et le pan de maison qui la porte.
pub(crate) fn dessiner_gouttieres(monde: &Monde, cx: i32, cy: i32) {
    for (x, sol, numero) in gouttieres_a_l_ecran(monde, cx, cy) {
        for (rx, ry, w, h, c) in gouttiere_accrochee(x, sol, numero) {
            rect(rx, ry, w, h, c);
        }
    }
}

/// Seconde passe, après les tuiles (sur l'eau) : le filet de chaque dauphin.
pub(crate) fn dessiner_filets(monde: &Monde, cx: i32, cy: i32) {
    for (x, sol, _) in gouttieres_a_l_ecran(monde, cx, cy) {
        for (rx, ry, w, h, c) in filet_de_gouttiere(x, sol, monde.pas) {
            rect(rx, ry, w, h, c);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pepin_core::palette::{BLANC_CASSE, NUAGE, PLUIE};

    /// Carte 10.4d (suite) : `maisons.png` tient UNE période de la rue (la somme
    /// des largeurs de `RUE`), et chaque carreau que le code allume tombe sur une
    /// vitre ÉTEINTE du fichier : les carreaux d'une même fenêtre sont d'une seule
    /// teinte sombre (le « creux » de la façade), la croisée entre eux est d'un
    /// cran plus claire (la couleur du mur), et la lucarne est d'ardoise de nuit.
    #[test]
    fn les_fenetres_allumees_tombent_sur_les_vitres_du_fichier() {
        let (l, h) = crate::fonds::taille("caniveau", "maisons");
        assert_eq!((l, h), (periode(), HAUT_MAISONS + TROTTOIR));
        let px =
            |x: i32, y: i32| crate::fonds::couleur("caniveau", "maisons", x as usize, y as usize);
        let sol = HAUT_MAISONS;
        let mut x = 0;
        let mut fenetres = 0;
        for &(w, etages) in &RUE {
            let haut = sol - REZ - etages * ETAGE;
            for dy in 0..4 {
                for dx in 0..4 {
                    let lucarne = px(x + w / 2 - 2 + dx, haut - TOIT + 3 + dy);
                    assert_eq!(lucarne, Some(ARDOISE_NUIT));
                }
            }
            for e in 0..etages {
                let y = haut + 4 + e * ETAGE;
                for c in 0..w / 16 {
                    let fx = x + c * 16 + 5;
                    let creux = px(fx, y + 2);
                    assert!(creux == Some(ARDOISE_NUIT) || creux == Some(ARDOISE_OMBRE));
                    for (cx, cy, cw, ch) in [
                        (fx, y + 2, 2, 3),
                        (fx + 3, y + 2, 3, 3),
                        (fx, y + 6, 2, 5),
                        (fx + 3, y + 6, 3, 5),
                    ] {
                        for yy in cy..cy + ch {
                            for xx in cx..cx + cw {
                                assert_eq!(px(xx, yy), creux, "({xx}, {yy})");
                            }
                        }
                    }
                    assert_ne!(px(fx + 2, y + 2), creux);
                    assert_ne!(px(fx, y + 5), creux);
                    fenetres += 1;
                }
            }
            x += w;
        }
        assert_eq!(fenetres, 95);
        // Le trottoir d'en face, sous les maisons, d'un bout à l'autre.
        assert!((0..l).all(|x| px(x, sol) == Some(ARDOISE_OMBRE)));
    }

    /// Les fenêtres allumées changent d'un tour de la rue à l'autre (sinon elles
    /// seraient dans le PNG) : environ une sur quatre, jamais deux tours pareils
    /// sur un long trajet.
    #[test]
    fn les_fenetres_allumees_changent_d_un_tour_a_l_autre() {
        let p = periode();
        let tours: Vec<Vec<(i32, i32, i32, i32)>> = (0..6)
            .map(|t| {
                carreaux_allumes(t * p, 92)
                    .into_iter()
                    .map(|(x, y, w, h)| (x.rem_euclid(p), y, w, h))
                    .collect()
            })
            .collect();
        assert!(tours.windows(2).all(|d| d[0] != d[1]));
    }

    /// Le lampadaire du fichier : le mât (ardoise d'ombre, deux pixels) est à la
    /// colonne `LAMPADAIRE_X`, jusqu'au sol ; le halo d'ardoise est au-dessus ;
    /// la lanterne violette au milieu du halo.
    #[test]
    fn le_lampadaire_du_fichier_est_pose_sur_son_mat() {
        let (l, h) = crate::fonds::taille("caniveau", "lampadaire");
        assert_eq!((l, h), (15, LAMPADAIRE_HAUT));
        let px = |x: i32, y: i32| {
            crate::fonds::couleur("caniveau", "lampadaire", x as usize, y as usize)
        };
        for y in 20..h {
            assert_eq!(px(LAMPADAIRE_X, y), Some(ARDOISE_OMBRE), "y={y}");
            assert_eq!(px(LAMPADAIRE_X + 1, y), Some(ARDOISE_OMBRE), "y={y}");
        }
        assert_eq!(px(LAMPADAIRE_X + 1, 0), Some(ARDOISE));
        assert_eq!(px(0, 0), None);
        assert_eq!(px(LAMPADAIRE_X, 6), Some(VIOLET_ORAGE));
    }

    /// Les pixels couverts par une liste de rectangles (le dernier posé l'emporte).
    fn pixels(r: &[Rect]) -> std::collections::HashMap<(i32, i32), Couleur> {
        let mut p = std::collections::HashMap::new();
        for &(x, y, w, h, c) in r {
            for py in y..y + h {
                for px in x..x + w {
                    p.insert((px, py), c);
                }
            }
        }
        p
    }

    /// Carte 9.20h : la descente de gouttière est ACCROCHÉE. Du chéneau au dauphin,
    /// le tuyau est d'un seul tenant (chaque rangée touche la suivante), tout contre
    /// le mur de son pan de maison ; le chéneau court au bord du toit ; le bec du
    /// dauphin est au-dessus de la 1re case d'eau (deux cases après la lettre `J`).
    #[test]
    fn la_gouttiere_est_accrochee_a_sa_maison() {
        let (x, sol) = (160, 112);
        let p = pixels(&gouttiere_accrochee(x, sol, 7));
        let haut = sol - REZ - 2 * ETAGE;
        // Le mur derrière tout le tuyau, du toit au trottoir.
        assert!((haut..sol).all(|y| p.contains_key(&(x + PAN_X, y))));
        // Le chéneau : du bord gauche du pan jusqu'au-dessus du tuyau.
        assert!((x + PAN_X..=x + TUYAU_X + 4).all(|px| p[&(px, haut + 1)] == ARDOISE));
        // Le zinc (ardoise) du tuyau, rangée par rangée, du chéneau au dauphin :
        // chaque rangée chevauche la précédente (coude compris), sans trou.
        let zinc = |y: i32| -> Vec<i32> {
            (x + TUYAU_X - 1..=x + BEC_X)
                .filter(|&px| p.get(&(px, y)) == Some(&ARDOISE))
                .collect()
        };
        let bas = sol - DAUPHIN_BAS - 3;
        let mut avant = zinc(haut + 1);
        for y in haut + 3..bas {
            let ici: Vec<i32> = zinc(y)
                .into_iter()
                .filter(|&px| px <= x + TUYAU_X + 4)
                .collect();
            // Un collier couvre le tuyau d'ombre : la rangée sous lui reprend.
            if ici.is_empty() {
                assert_eq!(p[&(x + TUYAU_X, y)], ARDOISE_OMBRE, "rangée {y}");
                continue;
            }
            assert!(
                ici.iter().any(|px| avant.contains(px)),
                "tuyau coupé en {y}"
            );
            avant = ici;
        }
        // Un collier : le tuyau passe à l'ombre (de part et d'autre), après du zinc.
        let colliers = (haut + 8..bas)
            .filter(|&y| {
                let t = x + TUYAU_X;
                p[&(t - 1, y)] == ARDOISE_OMBRE
                    && p[&(t + 1, y)] == ARDOISE_OMBRE
                    && p[&(t + 1, y - 1)] == ARDOISE
            })
            .count();
        assert!((2..=3).contains(&colliers), "{colliers} colliers");
        // Le dauphin : du tuyau au bec, au-dessus de l'eau (case `J` + 2).
        assert!((x + TUYAU_X..=x + BEC_X - 1).all(|px| p[&(px, bas + 1)] == ARDOISE));
        assert!((x + 32..x + 48).contains(&(x + BEC_X)));
        // Rien sous le haut du trottoir (l'eau et les pavés sont devant).
        assert!(p.keys().all(|&(_, y)| y < sol));
    }

    /// Le pan de maison et la gouttière sont du FOND (carte 9.20g) : les seuls tons
    /// sombres du fond, et aucun dessus plat clair : au-dessus de tout pixel
    /// d'ardoise (le plus clair), sur une rangée horizontale de plus de 12 px, il y
    /// a de l'ombre, jamais le ciel. Seul le filet est clair (blanc d'écume, et ses
    /// éclaboussures cyan) : il tombe sous le bec jusqu'à la surface de l'eau.
    #[test]
    fn la_gouttiere_et_sa_maison_sont_du_fond_terne() {
        let (x, sol) = (160, 112);
        let p = pixels(&gouttiere_accrochee(x, sol, 3));
        let fond = [ARDOISE_OMBRE, ARDOISE_NUIT, ARDOISE, VIOLET_ORAGE];
        assert!(p.values().all(|c| fond.contains(c)));
        for (&(px, py), &c) in &p {
            if c == ARDOISE && !p.contains_key(&(px, py - 1)) {
                panic!("dessus d'ardoise à l'air libre en ({px}, {py})");
            }
        }
        for pas in [0, 10] {
            let f = pixels(&filet_de_gouttiere(x, sol, pas));
            assert!(f
                .values()
                .all(|c| [ARDOISE_OMBRE, NUAGE, BLANC_CASSE, PLUIE].contains(c)));
            // Les éclaboussures cyan, juste au-dessus de la surface.
            assert!(f.iter().any(|(&(_, y), &c)| c == PLUIE && y == sol - 1));
            // Sous le bec, jusqu'à la surface (rien dessous), au-dessus de l'eau.
            assert!((sol - DAUPHIN_BAS..sol).all(|y| f.contains_key(&(x + BEC_X - 1, y))));
            assert!(f
                .keys()
                .all(|&(px, y)| y < sol && px >= x + 32 && px < x + 48));
        }
    }
}
