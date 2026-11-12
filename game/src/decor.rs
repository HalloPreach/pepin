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

/// Forme des silhouettes d'une couche.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Forme {
    /// Immeubles : corps, toit en gradins, cheminée.
    Immeubles,
    /// Bancs de nuages arrondis (trois étages en dôme).
    Nuages,
    /// Tours ou pics de nuages en quatre étages de plus en plus étroits.
    Pics,
}

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
    /// Forme des silhouettes (immeubles, bancs de nuages ou pics).
    pub forme: Forme,
    /// Pixels ajoutés à la hauteur des silhouettes de la couche LOINTAINE :
    /// un fond qui monte haut au lieu d'un simple liseré en bas d'écran.
    pub relief: i32,
    /// La couche lointaine descend jusqu'au bas de l'écran (pas de bande de ciel
    /// sous les silhouettes dans les trouées de la couche proche).
    pub socle: bool,
    /// Fenêtres allumées (lointaines, proches) sur les immeubles, s'il y en a.
    pub fenetres: Option<(Couleur, Couleur)>,
    /// Petites îles de nuages flottantes (corps, dessous), s'il y en a.
    pub ilots: Option<(Couleur, Couleur)>,
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
            forme: Forme::Immeubles,
            relief: 0,
            socle: false,
            fenetres: None,
            ilots: None,
        },
        Theme::Gouttieres => Ambiance {
            ciel: [ARDOISE_NUIT, ARDOISE, ARDOISE_CLAIRE, ARDOISE_CLAIRE],
            lointain: ARDOISE,
            proche: ARDOISE_NUIT,
            pluie: (PLUIE_SOMBRE, PLUIE),
            etoiles: false,
            soleil: false,
            forme: Forme::Immeubles,
            relief: 0,
            socle: false,
            fenetres: None,
            ilots: None,
        },
        // Orage : le ciel est violet à l'horizon ; les immeubles, plus hauts
        // qu'aux niveaux 1 et 2, s'y découpent en noir et bleu nuit.
        Theme::Orage => Ambiance {
            ciel: [ARDOISE_OMBRE, ARDOISE, VIOLET_ORAGE, VIOLET_ORAGE],
            lointain: ARDOISE_NUIT,
            proche: ARDOISE_OMBRE,
            pluie: (PLUIE_SOMBRE, PLUIE),
            etoiles: false,
            soleil: false,
            forme: Forme::Immeubles,
            relief: 22,
            socle: true,
            fenetres: Some((ORANGE_SOLEIL, OR)),
            ilots: None,
        },
        // Altitude : tours de nuages violettes sur le ciel corail, tours proches
        // plus sombres, petites îles flottantes dans le tiers médian.
        Theme::Altitude => Ambiance {
            ciel: [VIOLET_ORAGE, CORAIL, CORAIL_CLAIR, ORANGE_SOLEIL],
            lointain: VIOLET_ORAGE,
            proche: ARDOISE_NUIT,
            pluie: (CORAIL_CLAIR, BLANC_CASSE),
            etoiles: true,
            soleil: false,
            forme: Forme::Pics,
            relief: 14,
            socle: true,
            fenetres: None,
            ilots: Some((VIOLET_ORAGE, ARDOISE_NUIT)),
        },
        Theme::Soleil => Ambiance {
            ciel: [CORAIL_CLAIR, ORANGE_SOLEIL, OR, OR_PALE],
            lointain: NUAGE,
            proche: BLANC_CASSE,
            pluie: (OR_PALE, BLANC_CASSE),
            etoiles: false,
            soleil: true,
            forme: Forme::Nuages,
            relief: 14,
            socle: false,
            fenetres: None,
            ilots: None,
        },
    }
}

/// Ordonnée du sol de la couche lointaine et de la couche proche (avant parallaxe verticale).
const BASE_LOINTAINE: i32 = H - 18;
const BASE_PROCHE: i32 = H + 4;
/// Facteurs de parallaxe horizontale des couches (lointaine, îlots, proche).
const FACTEUR_LOINTAINE: f32 = 0.15;
const FACTEUR_ILOTS: f32 = 0.25;
const FACTEUR_PROCHE: f32 = 0.35;
/// Graines de hasard des deux couches de silhouettes.
const GRAINE_LOINTAINE: u32 = 11;
const GRAINE_PROCHE: u32 = 23;
/// Largeur d'une cellule d'îlots et fourchette d'ordonnées des îlots (tiers médian).
const CELLULE_ILOT: i32 = 72;
const ILOT_Y_MIN: i32 = 58;
const ILOT_Y_ALEA: u32 = 20;
const ILOT_HAUTEUR: i32 = 8;

#[cfg(test)]
/// Couleur de la bande de ciel située à l'ordonnée `y` (fonction pure : le test
/// vérifie par là qu'aucune silhouette n'a la couleur du ciel derrière elle).
pub(crate) fn ciel_a(a: &Ambiance, y: i32) -> Couleur {
    let bande = (H / 4) as usize;
    a.ciel[(y.max(0) as usize / bande).min(3)]
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
    (
        cellule * 48 + ((h >> 17) % 6) as i32,
        largeur,
        hauteur,
        cheminee,
    )
}

/// Hauteur totale (en pixels) de la silhouette de la cellule `cellule`, pour la
/// forme de l'ambiance et `relief` pixels de plus. Pure : sert au dessin ET aux tests.
pub fn hauteur_silhouette(a: &Ambiance, cellule: i32, graine: u32, relief: i32) -> i32 {
    let (_, _, h, _) = immeuble(cellule, graine);
    match a.forme {
        // Corps + cheminée qui dépasse de 10 px.
        Forme::Immeubles => h + relief + 10,
        Forme::Nuages => 8 + h / 3 + relief,
        // Quatre étages égaux (arrondis vers le haut).
        Forme::Pics => 4 * ((h + relief + 3) / 4),
    }
}

/// Fenêtre allumée `k` (0 à 2) de l'immeuble de la cellule `cellule` : décalage
/// `(dx, dy)` depuis le coin haut-gauche du corps, ou `None` (environ une fenêtre
/// sur trois seulement existe). Toujours à l'intérieur du corps du bâtiment.
pub fn fenetre(
    cellule: i32,
    graine: u32,
    k: i32,
    largeur: i32,
    hauteur: i32,
) -> Option<(i32, i32)> {
    let h = hache(cellule * 3 + k, graine ^ 0xF3);
    if !h.is_multiple_of(3) {
        return None;
    }
    let dx = 3 + ((h >> 4) % (largeur as u32 - 7)) as i32;
    let dy = 5 + ((h >> 12) % (hauteur as u32 - 10)) as i32;
    Some((dx, dy))
}

/// Îlot de nuages de la cellule `cellule` (72 px de large) : `(x, y, largeur)` dans le
/// repère de la couche, ou `None` (une cellule sur trois est vide).
pub fn ilot(cellule: i32) -> Option<(i32, i32, i32)> {
    let h = hache(cellule, 41);
    if h.is_multiple_of(3) {
        return None;
    }
    Some((
        cellule * CELLULE_ILOT + ((h >> 3) % 24) as i32,
        ILOT_Y_MIN + ((h >> 8) % ILOT_Y_ALEA) as i32,
        18 + ((h >> 14) % 15) as i32,
    ))
}

#[cfg(test)]
/// Une zone d'écran occupée par une couche de silhouettes : couleur et ordonnées
/// `[haut, bas)` occupées (avant parallaxe verticale), pour le contrôle de contraste.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Zone {
    pub nom: &'static str,
    pub couleur: Couleur,
    pub haut: i32,
    pub bas: i32,
}

#[cfg(test)]
/// Les couches de silhouettes d'une ambiance (lointaine, proche, îlots) avec la zone
/// d'écran où elles se dessinent. Hauteur maximale mesurée sur 200 cellules.
pub(crate) fn zones(a: &Ambiance) -> Vec<Zone> {
    let max_h = |graine: u32, relief: i32| {
        (-100..100)
            .map(|c| hauteur_silhouette(a, c, graine, relief))
            .max()
            .unwrap_or(0)
    };
    let mut v = vec![
        Zone {
            nom: "lointaine",
            couleur: a.lointain,
            haut: BASE_LOINTAINE - max_h(GRAINE_LOINTAINE, a.relief),
            bas: if a.socle { H } else { BASE_LOINTAINE },
        },
        Zone {
            nom: "proche",
            couleur: a.proche,
            haut: BASE_PROCHE - max_h(GRAINE_PROCHE, 0),
            bas: H,
        },
    ];
    if let Some((corps, dessous)) = a.ilots {
        for (nom, couleur) in [("ilots", corps), ("ilots-dessous", dessous)] {
            v.push(Zone {
                nom,
                couleur,
                haut: ILOT_Y_MIN,
                bas: ILOT_Y_MIN + ILOT_Y_ALEA as i32 + ILOT_HAUTEUR,
            });
        }
    }
    v
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
        let eteinte = (pas / 20 + h % 7).is_multiple_of(9);
        let couleur = if h.is_multiple_of(5) {
            OR_PALE
        } else {
            ARDOISE_PALE
        };
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

/// Une couche de silhouettes. `base` : ordonnée du sol de la couche. `relief` :
/// pixels ajoutés à la hauteur. `fenetres` : couleur des fenêtres allumées (immeubles).
#[allow(clippy::too_many_arguments)]
fn dessiner_couche(
    a: &Ambiance,
    cam_x: i32,
    base: i32,
    facteur: f32,
    couleur: Couleur,
    graine: u32,
    relief: i32,
    fenetres: Option<Couleur>,
) {
    let dec = decalage(cam_x, facteur);
    let premiere = dec.div_euclid(48) - 1;
    let derniere = (dec + L).div_euclid(48) + 1;
    for cellule in premiere..=derniere {
        let (x, l, h, chem) = immeuble(cellule, graine);
        let x = x - dec;
        let h = h + relief;
        let y = base - h;
        match a.forme {
            Forme::Nuages => {
                // Banc de nuages : trois étages de plus en plus étroits (un dôme).
                let hn = hauteur_silhouette(a, cellule, graine, relief);
                rect(x, base - hn / 2, l, hn / 2, couleur);
                rect(x + l / 8, base - hn * 3 / 4, l * 3 / 4, hn / 4, couleur);
                rect(x + l / 3, base - hn, l / 3, hn / 4, couleur);
            }
            Forme::Pics => {
                // Tour de nuages : quatre étages égaux, de plus en plus étroits ; la
                // position du sommet (`chem`) change d'une cellule à l'autre.
                let etage = hauteur_silhouette(a, cellule, graine, relief) / 4;
                for s in 0..4 {
                    let w = l - s * l / 5;
                    let decal = (l - w) * (chem - 6) / (l - 12);
                    rect(x + decal, base - (s + 1) * etage, w, etage, couleur);
                }
            }
            Forme::Immeubles => {
                // Immeuble : corps, toit en gradins, cheminée.
                rect(x, y, l, h, couleur);
                rect(x + 3, y - 2, l - 6, 2, couleur);
                rect(x + 7, y - 4, l - 14, 2, couleur);
                rect(x + chem, y - 10, 5, 8, couleur);
                if let Some(c) = fenetres {
                    for k in 0..3 {
                        if let Some((dx, dy)) = fenetre(cellule, graine, k, l, h) {
                            rect(x + dx, y + dy, 1, 2, c);
                        }
                    }
                }
            }
        }
    }
    // Socle : la couche lointaine descend jusqu'au bas de l'écran, sans trouée.
    if a.socle && base < H {
        rect(0, base, L, H - base, couleur);
    }
}

/// Îlots de nuages flottants (couche intermédiaire), dessinés en blocs entiers.
fn dessiner_ilots(corps: Couleur, dessous: Couleur, cam_x: i32, vert: i32) {
    let dec = decalage(cam_x, FACTEUR_ILOTS);
    let premiere = dec.div_euclid(CELLULE_ILOT) - 1;
    let derniere = (dec + L).div_euclid(CELLULE_ILOT) + 1;
    for cellule in premiere..=derniere {
        if let Some((x, y, l)) = ilot(cellule) {
            let (x, y) = (x - dec, y + vert);
            rect(x + l / 4, y, l / 2, 2, corps);
            rect(x, y + 2, l, 4, corps);
            rect(x + 3, y + 6, l - 6, ILOT_HAUTEUR - 6, dessous);
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
    if let Some((corps, dessous)) = a.ilots {
        dessiner_ilots(corps, dessous, cam_x, vert(0.07));
    }
    let (fen_loin, fen_proche) = match a.fenetres {
        Some((l, p)) => (Some(l), Some(p)),
        None => (None, None),
    };
    dessiner_couche(
        &a,
        cam_x,
        BASE_LOINTAINE + vert(0.05),
        FACTEUR_LOINTAINE,
        a.lointain,
        GRAINE_LOINTAINE,
        a.relief,
        fen_loin,
    );
    dessiner_couche(
        &a,
        cam_x,
        BASE_PROCHE + vert(0.10),
        FACTEUR_PROCHE,
        a.proche,
        GRAINE_PROCHE,
        0,
        fen_proche,
    );
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
            for (c1, c2) in a.fenetres.into_iter().chain(a.ilots) {
                couleurs.extend([c1, c2]);
            }
            for c in couleurs {
                assert!(TOUTES.contains(&c), "{t:?} : couleur hors palette {c:?}");
            }
        }
    }

    #[test]
    fn aucune_silhouette_n_a_la_couleur_du_ciel_derriere_elle() {
        for t in THEMES {
            let a = ambiance(t);
            for z in zones(&a) {
                assert!(z.haut < z.bas, "{t:?} {} : zone vide", z.nom);
                for y in z.haut..z.bas {
                    assert_ne!(
                        z.couleur,
                        ciel_a(&a, y),
                        "{t:?} : la couche {} a la couleur du ciel à y={y}",
                        z.nom
                    );
                }
            }
        }
    }

    #[test]
    fn la_couche_lointaine_monte_a_40_px_au_moins_dans_chaque_theme() {
        for t in THEMES {
            let a = ambiance(t);
            let max = (-100..100)
                .map(|c| hauteur_silhouette(&a, c, GRAINE_LOINTAINE, a.relief))
                .max()
                .unwrap();
            assert!(max >= 40, "{t:?} : fond « aplat », silhouette de {max} px");
        }
        // Les deux niveaux signalés ont de vraies silhouettes hautes.
        for t in [Theme::Orage, Theme::Altitude] {
            let a = ambiance(t);
            let z = zones(&a)[0];
            assert!(z.bas - z.haut >= 60, "{t:?} : couche lointaine trop basse");
        }
    }

    #[test]
    fn le_niveau_3_a_des_immeubles_avec_quelques_fenetres_allumees() {
        let a = ambiance(Theme::Orage);
        assert_eq!(a.forme, Forme::Immeubles);
        assert!(a.fenetres.is_some());
        let (mut total, mut cellules) = (0, 0);
        for c in -100..100 {
            let (_, l, h, _) = immeuble(c, GRAINE_LOINTAINE);
            let h = h + a.relief;
            cellules += 1;
            for k in 0..3 {
                if let Some((dx, dy)) = fenetre(c, GRAINE_LOINTAINE, k, l, h) {
                    total += 1;
                    // Une fenêtre de 1x2 reste dans le corps du bâtiment.
                    assert!((0..l).contains(&dx) && dy >= 0 && dy + 2 <= h, "{dx},{dy}");
                }
            }
        }
        // Peu nombreuses : entre 0,3 et 1,5 par immeuble en moyenne.
        assert!(
            total > cellules * 3 / 10 && total < cellules * 3 / 2,
            "{total}"
        );
    }

    #[test]
    fn les_ilots_du_niveau_4_restent_dans_le_tiers_median() {
        assert!(ambiance(Theme::Altitude).ilots.is_some());
        let mut vus = 0;
        for c in -100..100 {
            if let Some((_, y, l)) = ilot(c) {
                vus += 1;
                assert!(
                    y >= ILOT_Y_MIN
                        && y + ILOT_HAUTEUR <= ILOT_Y_MIN + ILOT_Y_ALEA as i32 + ILOT_HAUTEUR
                );
                assert!((18..33).contains(&l));
                assert!(y >= H / 4 && y + ILOT_HAUTEUR <= 3 * H / 4);
            }
        }
        assert!(vus > 100, "trop peu d'îlots : {vus}");
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
