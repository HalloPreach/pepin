//! Décor : ciel qui s'éclaircit d'un niveau à l'autre (orage -> soleil doré),
//! étoiles, silhouettes de toits ou de nuages en parallaxe sur deux couches,
//! soleil, et pluie diagonale. Tout est dessiné par rectangles alignés sur la
//! grille de pixels logiques (320x180) ; les décalages de parallaxe sont ARRONDIS
//! à l'entier : zéro sous-pixel, zéro scintillement.
//!
//! Les fonctions de calcul (`ciel`, `goutte_de_pluie`, `decalage`, `immeuble`) sont
//! pures : elles ne dessinent rien et se testent sans fenêtre.
//!
//! Notions Rust illustrées ici : struct de configuration par thème, `match`
//! exhaustif sur un `enum`, itérateurs sur des intervalles, arithmétique entière
//! qui boucle (`rem_euclid`).

use macroquad::prelude::draw_rectangle;
use pepin_core::palette::*;
use pepin_core::zones::Theme;
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use crate::rendu::couleur_mq;

const L: i32 = LARGEUR_LOGIQUE as i32;
const H: i32 = HAUTEUR_LOGIQUE as i32;

/// Ce qui distingue l'ambiance d'un niveau.
#[derive(Debug, Clone, Copy)]
pub struct Ambiance {
    /// Quatre bandes de ciel, de haut en bas.
    pub ciel: [Couleur; 4],
    /// Silhouettes lointaines et proches.
    pub lointain: Couleur,
    pub proche: Couleur,
    /// Couleur des traînées de pluie (lointaines, proches).
    pub pluie: (Couleur, Couleur),
    /// Étoiles visibles ?
    pub etoiles: bool,
    /// Soleil visible ?
    pub soleil: bool,
    /// Silhouettes en nuages arrondis plutôt qu'en toits ?
    pub nuageux: bool,
}

/// Ambiance d'un thème : l'orage noir s'éclaircit jusqu'au soleil doré.
pub fn ambiance(theme: Theme) -> Ambiance {
    match theme {
        Theme::Toits => Ambiance {
            ciel: [ARDOISE_OMBRE, ARDOISE_NUIT, ARDOISE, ARDOISE],
            lointain: ARDOISE_NUIT,
            proche: ARDOISE_OMBRE,
            pluie: (PLUIE_SOMBRE, PLUIE),
            etoiles: true,
            soleil: false,
            nuageux: false,
        },
        Theme::Gouttieres => Ambiance {
            ciel: [ARDOISE_NUIT, ARDOISE, ARDOISE_CLAIRE, ARDOISE_CLAIRE],
            lointain: ARDOISE,
            proche: ARDOISE_NUIT,
            pluie: (PLUIE_SOMBRE, PLUIE),
            etoiles: false,
            soleil: false,
            nuageux: false,
        },
        Theme::Orage => Ambiance {
            ciel: [ARDOISE_OMBRE, VIOLET_ORAGE, VIOLET_ORAGE, ARDOISE_NUIT],
            lointain: ARDOISE_NUIT,
            proche: ARDOISE_OMBRE,
            pluie: (PLUIE_SOMBRE, PLUIE),
            etoiles: false,
            soleil: false,
            nuageux: true,
        },
        Theme::Altitude => Ambiance {
            ciel: [VIOLET_ORAGE, CORAIL, CORAIL_CLAIR, ORANGE_SOLEIL],
            lointain: VIOLET_ORAGE,
            proche: ARDOISE_NUIT,
            pluie: (CORAIL_CLAIR, BLANC_CASSE),
            etoiles: true,
            soleil: false,
            nuageux: true,
        },
        Theme::Soleil => Ambiance {
            ciel: [CORAIL_CLAIR, ORANGE_SOLEIL, OR, OR_PALE],
            lointain: NUAGE,
            proche: BLANC_CASSE,
            pluie: (OR_PALE, BLANC_CASSE),
            etoiles: false,
            soleil: true,
            nuageux: true,
        },
    }
}

/// Décalage de parallaxe entier d'une couche : plus `facteur` est petit, plus la
/// couche semble lointaine (elle bouge moins vite que la caméra).
pub fn decalage(camera: i32, facteur: f32) -> i32 {
    (camera as f32 * facteur).round() as i32
}

/// Nombre pseudo-aléatoire stable dérivé d'un entier (même entrée, même sortie).
fn hache(n: i32, graine: u32) -> u32 {
    let mut h = (n as u32).wrapping_mul(0x9E37_79B1) ^ graine.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h
}

/// Un immeuble (ou un banc de nuages) de la couche : `(x, largeur, hauteur, cheminee)`
/// dans le repère de la couche, pour la cellule d'indice `cellule` (48 px de large).
pub fn immeuble(cellule: i32, graine: u32) -> (i32, i32, i32, i32) {
    let h = hache(cellule, graine);
    let largeur = 28 + (h % 17) as i32;
    let hauteur = 22 + ((h >> 5) % 40) as i32;
    let cheminee = 6 + ((h >> 11) % (largeur as u32 - 12)) as i32;
    (cellule * 48 + ((h >> 17) % 6) as i32, largeur, hauteur, cheminee)
}

/// Position `(x, y)` écran de la goutte de pluie `i` au pas `pas`, pour une couche
/// de vitesse `vitesse` (px par pas vers le bas). Chute en diagonale (1 px de
/// dérive vers la gauche pour 2 de chute) et boucle sur l'écran.
pub fn goutte_de_pluie(i: i32, pas: u32, vitesse: i32, graine: u32) -> (i32, i32) {
    let h = hache(i, graine);
    let x0 = (h % L as u32) as i32;
    let y0 = ((h >> 9) % H as u32) as i32;
    let t = pas as i32 * vitesse;
    ((x0 - t / 2).rem_euclid(L), (y0 + t).rem_euclid(H))
}

/// Nombre de gouttes par couche de pluie.
const GOUTTES_PAR_COUCHE: i32 = 36;

fn rect(x: i32, y: i32, w: i32, h: i32, c: Couleur) {
    draw_rectangle(x as f32, y as f32, w as f32, h as f32, couleur_mq(c));
}

/// Le ciel : quatre bandes séparées par une petite zone « tramée » (lignes alternées).
fn dessiner_ciel(a: &Ambiance) {
    let bande = H / 4;
    for (i, &c) in a.ciel.iter().enumerate() {
        let y = i as i32 * bande;
        rect(0, y, L, bande + 1, c);
        // Trame : sur les 6 dernières lignes de la bande, une ligne sur deux
        // reprend la couleur de la bande suivante.
        if let Some(&suivante) = a.ciel.get(i + 1) {
            for k in 0..3 {
                rect(0, y + bande - 6 + k * 2 + 1, L, 1, suivante);
            }
        }
    }
}

fn dessiner_etoiles(pas: u32, cam_x: i32) {
    let dec = decalage(cam_x, 0.03);
    for i in 0..40 {
        let h = hache(i, 77);
        let x = ((h % (L as u32 + 64)) as i32 - dec).rem_euclid(L);
        let y = ((h >> 10) % (H as u32 / 2)) as i32;
        // Scintillement : certaines étoiles s'éteignent un instant.
        let eteinte = (pas / 20 + h % 7) % 9 == 0;
        let couleur = if h % 5 == 0 { OR_PALE } else { ARDOISE_PALE };
        if !eteinte {
            rect(x, y, 1, 1, couleur);
        }
    }
}

fn dessiner_soleil(cam_x: i32) {
    let (cx, cy, r) = (236 - decalage(cam_x, 0.02) % 40, 44, 22);
    for dy in -r..=r {
        let w = ((r * r - dy * dy) as f32).sqrt() as i32;
        rect(cx - w, cy + dy, 2 * w + 1, 1, OR_PALE);
    }
    let r2 = 14;
    for dy in -r2..=r2 {
        let w = ((r2 * r2 - dy * dy) as f32).sqrt() as i32;
        rect(cx - w, cy + dy, 2 * w + 1, 1, BLANC_CASSE);
    }
}

/// Une couche de silhouettes. `base` : ordonnée du sol de la couche.
fn dessiner_couche(a: &Ambiance, cam_x: i32, base: i32, facteur: f32, couleur: Couleur, graine: u32) {
    let dec = decalage(cam_x, facteur);
    let premiere = dec.div_euclid(48) - 1;
    let derniere = (dec + L).div_euclid(48) + 1;
    for cellule in premiere..=derniere {
        let (x, l, h, chem) = immeuble(cellule, graine);
        let x = x - dec;
        let y = base - h;
        if a.nuageux {
            // Banc de nuages : trois étages de plus en plus étroits (un dôme),
            // bas et large plutôt qu'un immeuble.
            let hn = 8 + h / 3;
            rect(x, base - hn / 2, l, hn / 2, couleur);
            rect(x + l / 8, base - hn * 3 / 4, l * 3 / 4, hn / 4, couleur);
            rect(x + l / 3, base - hn, l / 3, hn / 4, couleur);
        } else {
            // Immeuble : corps, toit en gradins, cheminée.
            rect(x, y, l, h, couleur);
            rect(x + 3, y - 2, l - 6, 2, couleur);
            rect(x + 7, y - 4, l - 14, 2, couleur);
            rect(x + chem, y - 10, 5, 8, couleur);
        }
    }
}

fn dessiner_pluie(a: &Ambiance, pas: u32) {
    for (couche, (vitesse, couleur, longueur)) in [(2, a.pluie.0, 1), (3, a.pluie.1, 2)]
        .into_iter()
        .enumerate()
    {
        for i in 0..GOUTTES_PAR_COUCHE {
            let (x, y) = goutte_de_pluie(i, pas, vitesse, 100 + couche as u32);
            // Un trait diagonal de 1 à 2 pixels, calé sur la grille.
            for k in 0..longueur + 1 {
                rect(x + k, y - k * 2, 1, 1, couleur);
            }
        }
    }
}

/// Dessine tout ce qui est DERRIÈRE le niveau (ciel, étoiles, soleil, silhouettes).
pub fn dessiner_fond(theme: Theme, cam_x: i32, cam_y: i32, pas: u32) {
    let a = ambiance(theme);
    dessiner_ciel(&a);
    if a.etoiles {
        dessiner_etoiles(pas, cam_x);
    }
    if a.soleil {
        dessiner_soleil(cam_x);
    }
    // Les silhouettes se décalent aussi un peu à la verticale (parallaxe verticale).
    let vert = |f: f32| -decalage(cam_y, f);
    dessiner_couche(&a, cam_x, H - 18 + vert(0.05), 0.15, a.lointain, 11);
    dessiner_couche(&a, cam_x, H + 4 + vert(0.10), 0.35, a.proche, 23);
}

/// Dessine la pluie PAR-DESSUS le niveau (devant tout).
pub fn dessiner_avant(theme: Theme, pas: u32) {
    if theme.pluie() {
        dessiner_pluie(&ambiance(theme), pas);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const THEMES: [Theme; 5] = [
        Theme::Toits,
        Theme::Gouttieres,
        Theme::Orage,
        Theme::Altitude,
        Theme::Soleil,
    ];

    #[test]
    fn toutes_les_couleurs_du_decor_sont_dans_la_palette() {
        for t in THEMES {
            let a = ambiance(t);
            let mut couleurs = a.ciel.to_vec();
            couleurs.extend([a.lointain, a.proche, a.pluie.0, a.pluie.1]);
            for c in couleurs {
                assert!(TOUTES.contains(&c), "{t:?} : couleur hors palette {c:?}");
            }
        }
    }

    #[test]
    fn le_ciel_s_eclaircit_de_l_orage_au_soleil() {
        let clarte = |t: Theme| {
            let c = ambiance(t).ciel[3];
            c.r as u32 + c.g as u32 + c.b as u32
        };
        assert!(clarte(Theme::Toits) < clarte(Theme::Gouttieres));
        assert!(clarte(Theme::Gouttieres) < clarte(Theme::Altitude));
        assert!(clarte(Theme::Altitude) < clarte(Theme::Soleil));
    }

    #[test]
    fn la_parallaxe_est_entiere_et_proportionnelle() {
        assert_eq!(decalage(0, 0.35), 0);
        assert_eq!(decalage(100, 0.5), 50);
        assert_eq!(decalage(1000, 0.15), 150);
        // La couche lointaine bouge moins que la proche.
        assert!(decalage(777, 0.15) < decalage(777, 0.35));
    }

    #[test]
    fn la_pluie_reste_a_l_ecran_et_tombe_en_diagonale() {
        for pas in [0, 1, 59, 600, 123_456] {
            for i in 0..GOUTTES_PAR_COUCHE {
                let (x, y) = goutte_de_pluie(i, pas, 3, 101);
                assert!((0..L).contains(&x) && (0..H).contains(&y), "({x},{y})");
            }
        }
        // Après 2 pas à vitesse 2 : 4 px de chute, 2 px de dérive à gauche.
        let (x0, y0) = goutte_de_pluie(5, 0, 2, 100);
        let (x1, y1) = goutte_de_pluie(5, 2, 2, 100);
        assert_eq!((y1 - y0).rem_euclid(H), 4);
        assert_eq!((x0 - x1).rem_euclid(L), 2);
    }

    #[test]
    fn les_immeubles_sont_stables_et_bornes() {
        for c in -50..50 {
            assert_eq!(immeuble(c, 11), immeuble(c, 11));
            let (_, l, h, chem) = immeuble(c, 23);
            assert!((28..=44).contains(&l) && (22..62).contains(&h));
            assert!(chem >= 6 && chem + 5 <= l, "cheminée {chem} hors de {l}");
        }
    }
}
