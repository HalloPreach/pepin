//! Les fonds de la carte des niveaux (carte 9.28), un par chapitre, dessinés sous les
//! chemins et les étapes de `plan.rs`.
//!
//! - Chapitre 1, l'ascension : les toits sous la pluie (trois plans de silhouettes,
//!   parallaxe légère), l'orage et ses éclairs lointains, les oiseaux du couchant, la
//!   mer de nuages en volume, le soleil et ses rayons au sommet.
//! - Chapitre 2, la descente de l'eau : des bandes du bas vers le haut, dans l'ordre
//!   des niveaux (rue et caniveau, égouts, cascade, grotte, rivière, port), puis la
//!   mer, l'horizon et la Pieuvre en silhouette tout en haut. Les bandes suivent les
//!   étapes de la carte (`bandes_chapitre_2`) : un niveau s'ajoute, sa bande se range.
//!
//! Palette du jeu seulement, pas de dégradé : les bandes se fondent en damier 2x2.
//! Les fonds restent plus sombres ou plus ternes que le premier plan (étapes, chemins
//! dorés, Pépin), et leurs animations lentes ne bougent que quelques dizaines de pixels.

use std::f32::consts::PI;

use pepin_core::palette::{
    Couleur, ARDOISE, ARDOISE_CLAIRE, ARDOISE_NUIT, ARDOISE_OMBRE, ARDOISE_PALE, BLANC_CASSE,
    BRUME, CORAIL, CORAIL_CLAIR, NUAGE, OR, ORANGE_SOLEIL, OR_PALE, PLUIE, PLUIE_SOMBRE,
    VIOLET_ORAGE,
};
use pepin_core::zones::{InfosNiveau, Theme};

use super::rect;
use crate::plan::{Geometrie, ECART, H_TOITS, H_VUE, Y_SOLEIL};
use crate::sprites::Banque;

/// Où est la vue de la carte à l'écran, et où en est son défilement.
pub struct Vue {
    /// Haut de la vue à l'écran.
    pub y_vue: f32,
    /// Largeur de la carte à l'écran.
    pub l: f32,
    /// Défilement (ordonnée de la carte en haut de la vue).
    pub defil: f32,
}

impl Vue {
    /// Ordonnée écran d'une ordonnée de la carte (premier plan).
    fn ecran(&self, y: f32) -> f32 {
        (self.y_vue + y - self.defil).round()
    }

    /// Ordonnée écran d'un plan lointain (`f` < 1) : la parallaxe resserre ce plan vers
    /// le milieu de la vue, il défile moins vite que le premier plan.
    fn loin(&self, y: f32, f: f32) -> f32 {
        let c = self.y_vue + H_VUE / 2.0;
        (c + (self.y_vue + y - self.defil - c) * f).round()
    }

    /// La bande d'écran `y0..y1` touche-t-elle la vue ?
    fn voit(&self, y0: f32, y1: f32) -> bool {
        y1 >= self.y_vue && y0 <= self.y_vue + H_VUE
    }
}

/// Un petit hachage entier (décor « au hasard », mais le même à chaque image).
fn h(n: u32) -> u32 {
    let mut x = n.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    x
}

fn px(x: f32, y: f32, c: Couleur) {
    rect(x, y, 1.0, 1.0, c);
}

/// Disque plein de rayon `r` (en rangées de pixels).
fn disque(cx: f32, cy: f32, r: i32, c: Couleur) {
    for dy in -r..=r {
        let w = ((r * r - dy * dy) as f32).sqrt() as i32;
        rect(cx - w as f32, cy + dy as f32, (2 * w + 1) as f32, 1.0, c);
    }
}

/// Fondu en damier 2x2 de la couleur `c` sur les 6 rangées au-dessus de `y_bas` :
/// un quart, la moitié, puis trois quarts des cases (pas de dégradé dans la palette).
fn fondu(l: f32, y_bas: f32, c: Couleur) {
    for p in 0..3u32 {
        let y = y_bas - 6.0 + 2.0 * p as f32;
        for k in 0..(l as u32 / 2 + 1) {
            let q = (k + 2 * p) % 4;
            let plein = match p {
                0 => q == 0,
                1 => k % 2 == p % 2,
                _ => q != 0,
            };
            if plein {
                rect(2.0 * k as f32, y, 2.0, 2.0, c);
            }
        }
    }
}

// =====================================================================================
// Chapitre 1 : l'ascension, des toits sous la pluie au soleil
// =====================================================================================

/// Ciel du chapitre 1, de haut en bas : le soleil doré au sommet, l'orage au pied.
const CIEL_1: [Couleur; 9] = [
    OR_PALE,
    OR,
    ORANGE_SOLEIL,
    CORAIL_CLAIR,
    CORAIL,
    VIOLET_ORAGE,
    ARDOISE,
    ARDOISE_NUIT,
    ARDOISE_OMBRE,
];

/// Le fond du chapitre 1 (Monde 1). `anim` : pas écoulés depuis l'ouverture.
pub fn chapitre_1(banque: &Banque, v: &Vue, g: &Geometrie, anim: u32) {
    let hauteur = g.hauteur;
    // Le ciel en bandes, fondues en damier 2x2.
    let h_bande = hauteur / CIEL_1.len() as f32;
    for (i, &c) in CIEL_1.iter().enumerate() {
        let y0 = v.ecran(i as f32 * h_bande);
        let y1 = v.ecran((i + 1) as f32 * h_bande);
        if !v.voit(y0, y1) {
            continue;
        }
        rect(0.0, y0, v.l, y1 - y0 + 1.0, c);
        if let Some(&suivante) = CIEL_1.get(i + 1) {
            fondu(v.l, y1, suivante);
        }
    }
    soleil(v.ecran(Y_SOLEIL), anim);
    // Les oiseaux du couchant, qui passent lentement.
    for i in 0..4u32 {
        let y = v.loin(hauteur * (0.40 + 0.03 * i as f32), 0.85) + (h(i) % 7) as f32;
        let x = ((h(i + 9) % 200 + anim / (6 + i)) % 200) as f32 - 12.0;
        oiseau(x, y, (anim / 14 + i).is_multiple_of(2));
    }
    // La mer de nuages : un plan lointain (plus petit, plus pâle), puis le plan proche.
    let pale = (BRUME, ARDOISE_PALE, NUAGE);
    let clair = (NUAGE, ARDOISE_PALE, BLANC_CASSE);
    for (fx, fy, k) in [(36.0, 0.255, 0usize), (104.0, 0.245, 1), (160.0, 0.26, 2)] {
        nuage(fx, v.loin(hauteur * fy, 0.8), NUAGES_LOIN[k], pale);
    }
    for (fx, fy, k) in [
        (14.0, 0.30, 0usize),
        (74.0, 0.32, 1),
        (138.0, 0.295, 2),
        (176.0, 0.335, 0),
    ] {
        nuage(fx, v.ecran(hauteur * fy), NUAGES[k], clair);
    }
    // L'orage : des éclairs lointains, rares et brefs, sous les nuages d'orage.
    eclair(v, hauteur, anim);
    let orage = (ARDOISE_CLAIRE, ARDOISE, ARDOISE_PALE);
    for (fx, fy, k) in [(40.0, 0.60, 2usize), (120.0, 0.625, 1), (176.0, 0.59, 0)] {
        nuage(fx, v.ecran(hauteur * fy), NUAGES[k], orage);
    }
    // Les toits : deux plans de silhouettes derrière les tuiles, la pluie entre eux.
    let toits = hauteur - H_TOITS;
    let bas = v.y_vue + H_VUE + 16.0;
    silhouette(
        v,
        v.loin(toits, 0.75),
        (14, 38),
        ARDOISE,
        Some(ARDOISE_CLAIRE),
        1,
        bas,
    );
    pluie(v, v.ecran(hauteur * 0.62), v.ecran(toits), 24, anim);
    silhouette(
        v,
        v.loin(toits, 0.9),
        (6, 22),
        ARDOISE_NUIT,
        Some(ORANGE_SOLEIL),
        2,
        bas,
    );
    let y_toits = v.ecran(toits);
    for k in 0..=(v.l as i32 / 16) {
        let x = k as f32 * 16.0;
        banque.dessiner_tuile(Theme::Toits, "toit_haut", x, y_toits);
        banque.dessiner_tuile(Theme::Toits, "toit_plein", x, y_toits + 16.0);
    }
}

/// Le soleil et ses rayons, longs et courts en alternance (ils s'échangent lentement).
fn soleil(cy: f32, anim: u32) {
    let cx = 120.0;
    let phase = (anim / 120) % 2;
    for k in 0..16u32 {
        let a = k as f32 * PI / 8.0;
        let long = if (k + phase).is_multiple_of(2) { 8 } else { 5 };
        for d in 0..long {
            let r = 22.0 + d as f32;
            let c = if d < 3 { OR } else { ORANGE_SOLEIL };
            px((cx + a.cos() * r).round(), (cy + a.sin() * r).round(), c);
        }
    }
    for (r, c) in [(19, ORANGE_SOLEIL), (16, OR), (10, BLANC_CASSE)] {
        disque(cx, cy, r, c);
    }
}

/// Un oiseau en « v », ailes hautes ou basses.
fn oiseau(x: f32, y: f32, haut: bool) {
    let forme: [&str; 3] = if haut {
        ["#...#", ".#.#.", "..#.."]
    } else {
        [".....", "##.##", "..#.."]
    };
    for (j, ligne) in forme.iter().enumerate() {
        for (i, c) in ligne.chars().enumerate() {
            if c == '#' {
                px(x + i as f32, y + j as f32, ARDOISE_NUIT);
            }
        }
    }
}

/// Les bosses d'un nuage : (décalage depuis le centre, rayon).
type Bosses = &'static [(f32, i32)];
const NUAGES: [Bosses; 3] = [
    &[(-14.0, 6), (-4.0, 9), (8.0, 7), (17.0, 4)],
    &[(-18.0, 5), (-8.0, 8), (4.0, 11), (16.0, 7), (25.0, 4)],
    &[(-11.0, 6), (0.0, 8), (11.0, 5)],
];
const NUAGES_LOIN: [Bosses; 3] = [
    &[(-6.0, 4), (2.0, 6), (9.0, 3)],
    &[(-8.0, 3), (-1.0, 5), (7.0, 4)],
    &[(-5.0, 4), (4.0, 5)],
];

/// Un nuage en volume posé sur sa ligne `base` : des bosses en demi-disques (dessus
/// clair, liseré en haut), le bas des bosses et le dessous dans l'ombre.
/// `(clair, ombre, reflet)` : ses trois couleurs.
fn nuage(cx: f32, base: f32, bosses: Bosses, (clair, ombre, reflet): (Couleur, Couleur, Couleur)) {
    let w_de = |r: i32, dy: i32| ((r * r - dy * dy) as f32).sqrt() as i32;
    for &(dx, r) in bosses {
        for dy in -r..=0 {
            let w = w_de(r, dy);
            rect(
                cx + dx - w as f32,
                base + dy as f32,
                (2 * w + 1) as f32,
                1.0,
                clair,
            );
        }
    }
    for &(dx, r) in bosses {
        let w = w_de(r, -r + 1).max(1) - 1;
        rect(
            cx + dx - w as f32,
            base - r as f32 + 1.0,
            (2 * w + 1) as f32,
            1.0,
            reflet,
        );
    }
    for &(dx, r) in bosses {
        for dy in -2..=0 {
            let w = w_de(r, dy);
            rect(
                cx + dx - w as f32,
                base + dy as f32,
                (2 * w + 1) as f32,
                1.0,
                ombre,
            );
        }
    }
    // Le dessous, plat et arrondi aux bouts.
    let g = bosses
        .iter()
        .map(|&(dx, r)| dx - r as f32)
        .fold(f32::MAX, f32::min);
    let d = bosses
        .iter()
        .map(|&(dx, r)| dx + r as f32)
        .fold(f32::MIN, f32::max);
    for k in 1..=2 {
        let retrait = 2.0 * k as f32;
        rect(
            cx + g + retrait,
            base + k as f32,
            d - g - 2.0 * retrait + 1.0,
            1.0,
            ombre,
        );
    }
}

/// Un éclair lointain toutes les ~6 s, bref (deux éclats), à l'une de trois places.
fn eclair(v: &Vue, hauteur: f32, anim: u32) {
    let cycle = anim % 370;
    if !(cycle < 4 || (8..11).contains(&cycle)) {
        return;
    }
    let n = anim / 370;
    let x0 = [28.0, 96.0, 150.0][(n % 3) as usize];
    let mut x = x0;
    let mut y = v.loin(hauteur * 0.61, 0.9);
    let c = if cycle < 4 { BLANC_CASSE } else { NUAGE };
    for k in 0..7u32 {
        let pas = 3.0 + (h(n * 7 + k) % 3) as f32;
        let dx = if h(n + k).is_multiple_of(2) {
            -1.0
        } else {
            1.0
        };
        for _ in 0..pas as u32 {
            px(x, y, c);
            y += 1.0;
        }
        x += dx * 2.0;
        px(x - dx, y - 1.0, c);
    }
}

/// Une rangée de bâtiments en silhouette posée sur `base` (ordonnée écran), remplie
/// jusqu'à `bas` : toits plats, pointus ou en mansarde, cheminées, quelques fenêtres
/// (dont de rares allumées, `lumiere`). `graine` change la rangée.
fn silhouette(
    v: &Vue,
    base: f32,
    (h_min, h_max): (u32, u32),
    c: Couleur,
    lumiere: Option<Couleur>,
    graine: u32,
    bas: f32,
) {
    if base - h_max as f32 - 8.0 > v.y_vue + H_VUE {
        return;
    }
    let mut x = -6.0;
    let mut i = graine * 101;
    while x < v.l {
        let a = h(i);
        let w = (10 + a % 14) as f32;
        let haut = (h_min + (a >> 8) % (h_max - h_min)) as f32;
        let top = base - haut;
        rect(x, top, w, bas - top, c);
        match (a >> 16) % 3 {
            // Toit pointu.
            0 => {
                let m = (w / 2.0).floor() as i32;
                for r in 0..m {
                    let wr = (2 * r + 1) as f32;
                    rect(x + w / 2.0 - wr / 2.0, top - (m - r) as f32, wr, 1.0, c);
                }
            }
            // Mansarde : deux rangées en retrait.
            1 => {
                rect(x + 1.0, top - 1.0, w - 2.0, 1.0, c);
                rect(x + 3.0, top - 2.0, w - 6.0, 1.0, c);
            }
            // Toit plat et sa cheminée.
            _ => {
                let cx = x + 2.0 + ((a >> 20) % (w as u32 - 4)) as f32;
                rect(cx, top - 5.0, 3.0, 5.0, c);
                rect(cx - 1.0, top - 6.0, 5.0, 1.0, c);
            }
        }
        // Fenêtres : une grille clairsemée de 2x2, de rares allumées.
        if let Some(l) = lumiere {
            let mut k = 0;
            let mut fy = top + 3.0;
            while fy + 2.0 < base {
                let mut fx = x + 2.0;
                while fx + 2.0 < x + w - 1.0 {
                    let b = h(i * 31 + k);
                    if b.is_multiple_of(5) {
                        let couleur = if b.is_multiple_of(4) {
                            l
                        } else {
                            ARDOISE_OMBRE
                        };
                        rect(fx, fy, 2.0, 2.0, couleur);
                    }
                    k += 1;
                    fx += 4.0;
                }
                fy += 5.0;
            }
        }
        x += w + (a >> 24) as f32 % 3.0;
        i += 1;
    }
}

/// La pluie : `n` courts traits qui tombent entre `haut` et `bas` (ordonnées écran),
/// les lointains (plus sombres) plus lents.
fn pluie(v: &Vue, haut: f32, bas: f32, n: u32, anim: u32) {
    if bas <= haut || !v.voit(haut, bas) {
        return;
    }
    let span = (bas - haut) as u32;
    for i in 0..n {
        let loin = i.is_multiple_of(3);
        let vitesse = if loin { 2 } else { 3 };
        let x = (h(i + 40) % v.l as u32) as f32 + ((anim / 3 + i * 7) % 4) as f32;
        let y = haut + ((h(i) % span + anim * vitesse) % span) as f32;
        let c = if loin { PLUIE_SOMBRE } else { PLUIE };
        rect(x, y, 1.0, if loin { 2.0 } else { 3.0 }, c);
    }
}

// =====================================================================================
// Chapitre 2 : la descente de l'eau, de la rue jusqu'à la mer
// =====================================================================================

/// Les bandes du fond du chapitre 2, du bas vers le haut (l'ordre des niveaux).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bande {
    Rue,
    Egouts,
    Cascade,
    Grotte,
    Riviere,
    Port,
    /// La mer et l'horizon, au-dessus du port, avec la Pieuvre : tout en haut.
    Mer,
}

/// Les bandes qui suivent les niveaux, du bas vers le haut (sans la mer).
const ORDRE_2: [Bande; 6] = [
    Bande::Rue,
    Bande::Egouts,
    Bande::Cascade,
    Bande::Grotte,
    Bande::Riviere,
    Bande::Port,
];

/// La bande du fond d'un thème du Monde 2 (la marée et la Pieuvre sont au port).
pub fn bande_du_theme(t: Theme) -> Option<Bande> {
    match t {
        Theme::Caniveau => Some(Bande::Rue),
        Theme::Egouts => Some(Bande::Egouts),
        Theme::Cascade => Some(Bande::Cascade),
        Theme::Grotte => Some(Bande::Grotte),
        Theme::Riviere => Some(Bande::Riviere),
        Theme::Port | Theme::Maree => Some(Bande::Port),
        _ => None,
    }
}

/// Le chapitre se dessine-t-il en DESCENTE de l'eau (le fond `chapitre_2`) plutôt
/// qu'en ascension (`chapitre_1`) ? Oui si un de ses niveaux a un thème qui a sa
/// bande (`bande_du_theme`) : une donnée portée par les niveaux (carte 11.2), plus
/// le numéro du monde. Monde 1 : non ; Monde 2 : oui. Pure.
pub fn fond_en_descente(tranche: &[InfosNiveau]) -> bool {
    tranche.iter().any(|n| bande_du_theme(n.theme).is_some())
}

/// Hauteur de la bande de la mer au-dessus du centre du port, au moins.
const AU_DESSUS_DU_PORT: f32 = 22.0;

/// Les bandes du chapitre 2 : `(bande, haut, bas)` en ordonnées de la carte, du bas
/// (la rue, jusqu'au pied de la carte) vers le haut (la mer, jusqu'à 0). Chaque bande
/// est centrée sur les étapes de son thème ; une bande sans niveau (la rivière et le
/// port, tant qu'ils n'existent pas) prend sa place entre ses voisines.
pub fn bandes_chapitre_2(tranche: &[InfosNiveau], g: &Geometrie) -> Vec<(Bande, f32, f32)> {
    let mut centres: Vec<Option<f32>> = ORDRE_2
        .iter()
        .map(|&b| {
            let ys: Vec<f32> = tranche
                .iter()
                .zip(&g.etapes)
                .filter(|(n, _)| bande_du_theme(n.theme) == Some(b))
                .map(|(_, &(_, y))| y)
                .collect();
            (!ys.is_empty()).then(|| ys.iter().sum::<f32>() / ys.len() as f32)
        })
        .collect();
    let connus: Vec<usize> = (0..centres.len())
        .filter(|&i| centres[i].is_some())
        .collect();
    if connus.is_empty() {
        // Aucun niveau reconnu : des bandes égales.
        let pas = g.hauteur / (ORDRE_2.len() + 1) as f32;
        for (i, c) in centres.iter_mut().enumerate() {
            *c = Some(g.hauteur - (i as f32 + 1.0) * pas);
        }
    }
    // Les trous : entre deux voisines connues, à parts égales ; aux bouts, un écart.
    let connu = |c: &[Option<f32>], i: usize| c[i];
    for i in 0..centres.len() {
        if centres[i].is_some() {
            continue;
        }
        let avant = (0..i).rev().find(|&k| centres[k].is_some());
        let apres = (i + 1..centres.len()).find(|&k| centres[k].is_some());
        centres[i] = Some(match (avant, apres) {
            (Some(a), Some(b)) => {
                let (ya, yb) = (connu(&centres, a).unwrap(), connu(&centres, b).unwrap());
                ya + (yb - ya) * (i - a) as f32 / (b - a) as f32
            }
            (Some(a), None) => connu(&centres, a).unwrap() - (i - a) as f32 * ECART / 2.0,
            (None, Some(b)) => connu(&centres, b).unwrap() + (b - i) as f32 * ECART / 2.0,
            (None, None) => g.hauteur / 2.0,
        });
    }
    let c: Vec<f32> = centres.into_iter().map(|c| c.unwrap_or(0.0)).collect();
    let mut bandes = Vec::new();
    let mut bas = g.hauteur;
    for (i, &b) in ORDRE_2.iter().enumerate() {
        let haut = match c.get(i + 1) {
            Some(&suivant) => (c[i] + suivant) / 2.0,
            None => c[i] - AU_DESSUS_DU_PORT,
        }
        .clamp(0.0, bas)
        .round();
        bandes.push((b, haut, bas));
        bas = haut;
    }
    bandes.push((Bande::Mer, 0.0, bas));
    bandes
}

/// Le fond du chapitre 2 (Monde 2) : ses bandes, dessinées du haut (la mer) vers le
/// bas, chacune fondue en damier dans celle du dessus.
pub fn chapitre_2(banque: &Banque, v: &Vue, g: &Geometrie, tranche: &[InfosNiveau], anim: u32) {
    let bandes = bandes_chapitre_2(tranche, g);
    for &(b, haut, bas) in bandes.iter().rev() {
        let (y0, y1) = (v.ecran(haut), v.ecran(bas));
        // Une marge : les détails d'une bande débordent un peu sur celle du dessus.
        if !v.voit(y0 - 24.0, y1 + 8.0) || y1 <= y0 {
            continue;
        }
        match b {
            Bande::Mer => mer(v, y0, y1, anim),
            Bande::Port => port(v, y0, y1, anim),
            Bande::Riviere => riviere(v, y0, y1, anim),
            Bande::Grotte => grotte(v, y0, y1, anim),
            Bande::Cascade => cascade(v, y0, y1, anim),
            Bande::Egouts => egouts(v, y0, y1, anim),
            Bande::Rue => rue(banque, v, y0, g, anim),
        }
    }
}

/// La mer à l'aube, l'horizon, le soleil qui se lève et la Pieuvre en silhouette.
fn mer(v: &Vue, y0: f32, y1: f32, anim: u32) {
    let horizon = (y0 + (y1 - y0) * 0.62).round();
    let h_ciel = ((horizon - y0) / 3.0).max(2.0);
    for (k, &c) in [OR_PALE, CORAIL_CLAIR, ARDOISE_PALE].iter().enumerate() {
        let ya = (y0 + k as f32 * h_ciel).round();
        rect(0.0, ya, v.l, horizon - ya, c);
        if k > 0 {
            fondu(v.l, ya, c);
        }
    }
    // Le soleil qui se lève, à moitié sous l'horizon.
    for (r, c) in [(9, OR), (5, OR_PALE)] {
        for dy in -r..=0 {
            let w = ((r * r - dy * dy) as f32).sqrt() as i32;
            rect(
                36.0 - w as f32,
                horizon + dy as f32,
                (2 * w + 1) as f32,
                1.0,
                c,
            );
        }
    }
    // La mer : ses vagues, et le chemin de lumière du soleil qui scintille.
    rect(0.0, horizon, v.l, y1 - horizon, ARDOISE);
    rect(0.0, horizon, v.l, 1.0, ARDOISE_CLAIRE);
    for i in 0..10u32 {
        let x = (h(i + 300) % v.l as u32) as f32;
        let y = horizon + 2.0 + (h(i + 310) % (y1 - horizon - 2.0).max(1.0) as u32) as f32;
        rect(x, y, 3.0, 1.0, ARDOISE_CLAIRE);
    }
    let scintille = (anim / 40) % 2;
    let mut y = horizon + 2.0;
    let mut k = 0u32;
    while y < y1 {
        let w = 5.0 + k as f32 * 2.0;
        let dx = if (k + scintille).is_multiple_of(2) {
            0.0
        } else {
            2.0
        };
        rect(36.0 - w / 2.0 + dx, y, w / 2.0, 1.0, OR_PALE);
        y += 2.0;
        k += 1;
    }
    pieuvre(140.0, horizon, anim);
}

/// La Pieuvre en silhouette, la tête hors de l'eau à l'horizon, trois tentacules
/// levés qui ondulent lentement.
fn pieuvre(cx: f32, horizon: f32, anim: u32) {
    let c = ARDOISE_NUIT;
    // La tête : un dôme un peu allongé.
    for dy in -14..=0i32 {
        let t = (dy + 14) as f32 / 14.0;
        let w = (11.0 * (1.0 - (1.0 - t) * (1.0 - t)).sqrt()).round();
        rect(cx - w, horizon + dy as f32, 2.0 * w + 1.0, 1.0, c);
    }
    // Les yeux, qui luisent faiblement.
    px(cx - 4.0, horizon - 4.0, CORAIL);
    px(cx + 4.0, horizon - 4.0, CORAIL);
    // Les tentacules : (pied, sens de la boucle, hauteur).
    let phase = (anim / 45) % 4;
    for (k, &(x0, sens, haut)) in [(-20.0f32, -1.0f32, 16i32), (19.0, 1.0, 20), (31.0, 1.0, 11)]
        .iter()
        .enumerate()
    {
        let ondule = [0.0, 1.0, 0.0, -1.0][((phase + k as u32) % 4) as usize];
        for i in 0..haut {
            let t = i as f32 / haut as f32;
            let x = cx + x0 + sens * (t * PI * 0.9).sin() * 3.0 + ondule * t;
            let w: f32 = if t < 0.4 {
                3.0
            } else if t < 0.75 {
                2.0
            } else {
                1.0
            };
            rect(x.round() - (w / 2.0).floor(), horizon - i as f32, w, 1.0, c);
        }
        // La boucle du bout.
        let xb = (cx + x0 + sens * (PI * 0.9).sin() * 3.0 + ondule).round();
        let yb = horizon - haut as f32;
        px(xb + sens, yb - 1.0, c);
        px(xb + 2.0 * sens, yb, c);
    }
}

/// Le port : l'eau du bassin, des bateaux à quai, une grue, le quai de pierre.
fn port(v: &Vue, y0: f32, y1: f32, anim: u32) {
    let quai = y1 - 7.0;
    rect(0.0, y0, v.l, quai - y0, ARDOISE);
    fondu(v.l, y0, ARDOISE);
    // Reflets sur l'eau du bassin.
    for i in 0..6u32 {
        let x = ((h(i + 400) % v.l as u32) + (anim / 50 + i) % 3) as f32;
        let y = y0 + 2.0 + (h(i + 410) % ((quai - y0 - 3.0).max(1.0) as u32)) as f32;
        rect(x, y, 4.0, 1.0, ARDOISE_CLAIRE);
    }
    // La grue, à droite, qui monte au-dessus de la mer (plan lointain).
    let c = ARDOISE_NUIT;
    rect(158.0, quai - 30.0, 3.0, 30.0, c);
    rect(140.0, quai - 30.0, 30.0, 2.0, c);
    for k in 0..7 {
        px(141.0 + 4.0 * k as f32, quai - 28.0, c);
    }
    rect(146.0, quai - 28.0, 1.0, 8.0, c);
    rect(145.0, quai - 20.0, 3.0, 2.0, ARDOISE_OMBRE);
    // Deux bateaux à quai, leur coque, leur cabine et leur mât.
    for &(bx, w, coque) in &[(14.0f32, 24.0f32, CORAIL), (92.0, 30.0, ARDOISE_NUIT)] {
        let ligne = quai - 1.0;
        for r in 0..4 {
            let retrait = r as f32;
            rect(
                bx + retrait,
                ligne - 3.0 + r as f32,
                w - 2.0 * retrait,
                1.0,
                ARDOISE_NUIT,
            );
        }
        rect(bx, ligne - 4.0, w, 1.0, coque);
        rect(bx + w / 2.0 - 4.0, ligne - 8.0, 8.0, 4.0, ARDOISE_CLAIRE);
        rect(bx + w / 2.0 - 3.0, ligne - 7.0, 2.0, 2.0, ARDOISE_NUIT);
        rect(bx + w / 2.0 + 6.0, ligne - 18.0, 1.0, 14.0, ARDOISE_NUIT);
    }
    // Le quai : des blocs de pierre, des bittes d'amarrage.
    rect(0.0, quai, v.l, y1 - quai, ARDOISE_CLAIRE);
    rect(0.0, quai, v.l, 1.0, ARDOISE_PALE);
    for r in 0..2 {
        let y = quai + 3.0 + 3.0 * r as f32;
        rect(0.0, y, v.l, 1.0, ARDOISE);
        let mut x = if r == 0 { 5.0 } else { 0.0 };
        while x < v.l {
            rect(x, y - 2.0, 1.0, 2.0, ARDOISE);
            x += 10.0;
        }
    }
    for bx in [60.0, 130.0] {
        rect(bx, quai - 3.0, 3.0, 3.0, ARDOISE_OMBRE);
        rect(bx - 1.0, quai - 4.0, 5.0, 1.0, ARDOISE_OMBRE);
    }
}

/// La rivière : la berge verte d'en face et ses roseaux, l'eau qui coule (des reflets
/// qui filent lentement vers la droite), la berge de devant.
fn riviere(v: &Vue, y0: f32, y1: f32, anim: u32) {
    rect(0.0, y0, v.l, y1 - y0, ARDOISE);
    fondu(v.l, y0, ARDOISE);
    let berge = (y0 + 5.0).min(y1);
    rect(0.0, y0, v.l, berge - y0, PLUIE_SOMBRE);
    // Les touffes de la berge d'en face.
    let mut x = 0.0;
    let mut i = 0u32;
    while x < v.l {
        let r = 2 + (h(i + 500) % 3) as i32;
        for dy in -r..=0 {
            let w = ((r * r - dy * dy) as f32).sqrt();
            rect(x - w, y0 + dy as f32, 2.0 * w + 1.0, 1.0, PLUIE_SOMBRE);
        }
        x += 7.0 + (h(i + 520) % 9) as f32;
        i += 1;
    }
    // Les roseaux, et leurs massettes.
    for k in 0..9u32 {
        let rx = (h(k + 540) % v.l as u32) as f32;
        let haut = 4.0 + (h(k + 550) % 5) as f32;
        rect(rx, berge - haut, 1.0, haut, ARDOISE_NUIT);
        if k.is_multiple_of(3) {
            rect(rx, berge - haut - 1.0, 1.0, 2.0, CORAIL);
        }
    }
    rect(0.0, berge, v.l, 1.0, ARDOISE_NUIT);
    // L'eau qui coule : des reflets qui filent.
    let fond = y1 - 3.0;
    for k in 0..7u32 {
        let y = berge + 2.0 + (h(k + 560) % ((fond - berge - 2.0).max(1.0) as u32)) as f32;
        let x = ((h(k + 570) + anim / 8) % (v.l as u32 + 10)) as f32 - 5.0;
        rect(
            x,
            y,
            4.0,
            1.0,
            if k.is_multiple_of(3) {
                BRUME
            } else {
                ARDOISE_CLAIRE
            },
        );
    }
    // La berge de devant.
    rect(0.0, fond, v.l, y1 - fond, PLUIE_SOMBRE);
    rect(0.0, fond, v.l, 1.0, ARDOISE_CLAIRE);
}

/// La grotte : roche sombre, stalactites et stalagmites, cristaux cyan, et des bulles
/// qui montent des sources.
fn grotte(v: &Vue, y0: f32, y1: f32, anim: u32) {
    rect(0.0, y0, v.l, y1 - y0, ARDOISE_OMBRE);
    fondu(v.l, y0, ARDOISE_OMBRE);
    // Plan lointain : la roche, en plaques.
    for k in 0..14u32 {
        let x = (h(k + 600) % v.l as u32) as f32;
        let y = y0 + (h(k + 610) % ((y1 - y0).max(1.0) as u32)) as f32;
        rect(x, y, 6.0 + (k % 3) as f32 * 3.0, 2.0, ARDOISE_NUIT);
    }
    // Les stalactites (et leur goutte qui tombe), les stalagmites.
    let mut x = 3.0;
    let mut i = 0u32;
    while x < v.l {
        let a = h(i + 620);
        let w = (5 + a % 5) as i32;
        let long = (6 + (a >> 8) % 9) as i32;
        for r in 0..long {
            let wr = (w as f32 * (1.0 - r as f32 / long as f32)).ceil().max(1.0);
            rect(x - (wr / 2.0).floor(), y0 + r as f32, wr, 1.0, ARDOISE);
            px(x - (wr / 2.0).floor(), y0 + r as f32, ARDOISE_CLAIRE);
        }
        if i % 4 == 1 {
            let chute = ((y1 - y0) as u32).saturating_sub(long as u32 + 2).max(1);
            let dy = ((anim / 2 + h(i) % 90) % 90).min(chute);
            if dy < chute {
                px(x, y0 + long as f32 + 1.0 + dy as f32, PLUIE);
            }
        }
        let haut = (2 + (a >> 16) % 5) as i32;
        for r in 0..haut {
            let wr = (2 * (haut - r) - 1) as f32;
            rect(
                x + 6.0 - (wr / 2.0).floor(),
                y1 - 1.0 - r as f32,
                wr,
                1.0,
                ARDOISE_NUIT,
            );
        }
        x += 12.0 + ((a >> 20) % 10) as f32;
        i += 1;
    }
    // Les cristaux cyan, au pied des parois.
    for &(cx, k) in &[(18.0f32, 0u32), (84.0, 1), (156.0, 2)] {
        let cy = y1 - 4.0 - (h(k + 650) % 6) as f32;
        for (dx, haut) in [(-2.0, 3.0), (0.0, 5.0), (2.0, 3.0)] {
            rect(cx + dx, cy - haut, 1.0, haut, PLUIE_SOMBRE);
        }
        px(cx, cy - 4.0, PLUIE);
    }
    // Les bulles qui montent, en oscillant.
    let span = (y1 - y0 - 4.0).max(1.0) as u32;
    for k in 0..4u32 {
        let bx = [30.0, 70.0, 110.0, 146.0][k as usize];
        let monte = (anim / 4 + h(k + 660) % span) % span;
        let by = y1 - 3.0 - monte as f32;
        let ox = if (anim / 30 + k).is_multiple_of(2) {
            0.0
        } else {
            1.0
        };
        let r = 1.0 + (k % 2) as f32;
        rect(bx + ox - r + 1.0, by - r, 2.0 * r - 1.0, 1.0, PLUIE_SOMBRE);
        rect(bx + ox - r + 1.0, by + r, 2.0 * r - 1.0, 1.0, PLUIE_SOMBRE);
        rect(bx + ox - r, by - r + 1.0, 1.0, 2.0 * r - 1.0, PLUIE_SOMBRE);
        rect(bx + ox + r, by - r + 1.0, 1.0, 2.0 * r - 1.0, PLUIE_SOMBRE);
        px(bx + ox - r + 1.0, by - r + 1.0, PLUIE);
    }
}

/// La cascade : la falaise, une chute lointaine, la grande chute (de l'eau claire
/// qui tombe, des filets qui descendent lentement) et son écume.
fn cascade(v: &Vue, y0: f32, y1: f32, anim: u32) {
    rect(0.0, y0, v.l, y1 - y0, ARDOISE_NUIT);
    fondu(v.l, y0, ARDOISE_NUIT);
    // La roche de la falaise.
    for k in 0..16u32 {
        let x = (h(k + 700) % v.l as u32) as f32;
        let y = y0 + (h(k + 710) % ((y1 - y0).max(1.0) as u32)) as f32;
        rect(x, y, 3.0 + (k % 4) as f32, 2.0, ARDOISE);
        px(x, y, ARDOISE_CLAIRE);
    }
    // Une chute lointaine, mince et sombre.
    rect(22.0, y0, 3.0, y1 - y0, PLUIE_SOMBRE);
    // La grande chute, entre deux parois de roche.
    let (x0, w) = (136.0, 26.0);
    rect(x0 - 3.0, y0, 3.0, y1 - y0, ARDOISE);
    rect(x0 + w, y0, 3.0, y1 - y0, ARDOISE);
    rect(x0, y0, w, y1 - y0, PLUIE_SOMBRE);
    for k in 0..6 {
        rect(x0 + 2.0 + 4.0 * k as f32, y0, 1.0, y1 - y0, PLUIE);
    }
    // Les filets clairs qui descendent.
    let span = (y1 - y0).max(1.0) as u32;
    for k in 0..8u32 {
        let fx = x0 + 1.0 + (h(k + 720) % (w as u32 - 2)) as f32;
        let fy = y0 + ((h(k + 730) % span + anim / 2) % span) as f32;
        rect(fx, fy, 1.0, 4.0_f32.min(y1 - fy), NUAGE);
    }
    // L'écume au pied.
    for k in 0..7 {
        let ex = x0 - 2.0 + 4.5 * k as f32;
        let r = 2 + (k % 2);
        for dy in -r..=0 {
            let ww = ((r * r - dy * dy) as f32).sqrt();
            rect(
                ex - ww,
                y1 - 1.0 + dy as f32,
                2.0 * ww + 1.0,
                1.0,
                if dy == -r { BLANC_CASSE } else { NUAGE },
            );
        }
    }
}

/// Les égouts : des briques violettes, deux tuyaux (l'un goutte), une arche à
/// barreaux et le canal où l'eau file.
fn egouts(v: &Vue, y0: f32, y1: f32, anim: u32) {
    rect(0.0, y0, v.l, y1 - y0, ARDOISE_OMBRE);
    fondu(v.l, y0, ARDOISE_OMBRE);
    // Les briques : leur face sombre, l'arête violette.
    let canal = y1 - 7.0;
    let mut y = y0 + 1.0;
    let mut rangee = 0u32;
    while y + 4.0 <= canal {
        let mut x = if rangee.is_multiple_of(2) { 0.0 } else { -5.0 };
        while x < v.l {
            rect(x, y, 9.0, 4.0, ARDOISE_NUIT);
            rect(x, y, 9.0, 1.0, VIOLET_ORAGE);
            x += 10.0;
        }
        y += 5.0;
        rangee += 1;
    }
    // L'arche à barreaux, à droite.
    let ax = 154.0;
    for dy in -11..=0i32 {
        let w = ((121 - dy * dy) as f32).sqrt() as i32;
        rect(
            ax - w as f32,
            canal + dy as f32,
            (2 * w + 1) as f32,
            1.0,
            ARDOISE_OMBRE,
        );
    }
    for k in 0..6 {
        rect(ax - 9.0 + 3.5 * k as f32, canal - 9.0, 1.0, 9.0, ARDOISE);
    }
    // Le tuyau du haut, ses colliers, et le tuyau qui en descend.
    let ty = y0 + 6.0;
    rect(0.0, ty, v.l - 30.0, 1.0, ARDOISE_PALE);
    rect(0.0, ty + 1.0, v.l - 30.0, 1.0, ARDOISE_CLAIRE);
    rect(0.0, ty + 2.0, v.l - 30.0, 1.0, ARDOISE);
    for k in 0..4 {
        rect(10.0 + 34.0 * k as f32, ty - 1.0, 2.0, 5.0, ARDOISE_CLAIRE);
    }
    let vx = 112.0;
    let bout = canal - 9.0;
    rect(vx, ty + 3.0, 1.0, bout - ty - 3.0, ARDOISE_PALE);
    rect(vx + 1.0, ty + 3.0, 2.0, bout - ty - 3.0, ARDOISE);
    rect(vx - 1.0, bout, 5.0, 2.0, ARDOISE_CLAIRE);
    // La goutte qui tombe du tuyau.
    let chute = (canal - bout - 2.0).max(1.0) as u32;
    let t = (anim / 3) % (chute + 20);
    if t < chute {
        px(vx + 1.0, bout + 2.0 + t as f32, PLUIE);
    }
    // Le canal, et ses reflets qui filent.
    rect(0.0, canal, v.l, y1 - canal, PLUIE_SOMBRE);
    rect(0.0, canal, v.l, 1.0, ARDOISE_CLAIRE);
    for k in 0..5u32 {
        let x = ((h(k + 800) + anim / 7) % (v.l as u32 + 8)) as f32 - 4.0;
        let y = canal + 2.0 + (k % 3) as f32 * 1.5;
        rect(x, y.round(), 3.0, 1.0, PLUIE);
    }
}

/// La rue sous l'averse : les façades au loin et leurs fenêtres, un lampadaire, la
/// pluie, puis la chaussée du caniveau (pavés, terre), ses flaques qui reflètent.
fn rue(banque: &Banque, v: &Vue, y0: f32, g: &Geometrie, anim: u32) {
    let sol = v.ecran(g.hauteur - H_TOITS);
    let bas = v.y_vue + H_VUE + 16.0;
    rect(0.0, y0, v.l, sol - y0, ARDOISE_NUIT);
    fondu(v.l, y0, ARDOISE_NUIT);
    silhouette(v, sol, (16, 34), ARDOISE, Some(ORANGE_SOLEIL), 3, bas);
    // Le lampadaire.
    let lx = 150.0;
    rect(lx, sol - 30.0, 1.0, 30.0, ARDOISE_OMBRE);
    rect(lx - 3.0, sol - 32.0, 7.0, 2.0, ARDOISE_OMBRE);
    rect(lx - 2.0, sol - 30.0, 5.0, 1.0, OR_PALE);
    pluie(v, y0, sol, 18, anim);
    for k in 0..=(v.l as i32 / 16) {
        let x = k as f32 * 16.0;
        banque.dessiner_tuile(Theme::Caniveau, "can_pave", x, sol);
        banque.dessiner_tuile(Theme::Caniveau, "can_terre", x, sol + 16.0);
    }
    // Une bouche d'égout dans la chaussée.
    rect(98.0, sol + 6.0, 12.0, 4.0, ARDOISE_OMBRE);
    for k in 0..4 {
        rect(99.0 + 3.0 * k as f32, sol + 7.0, 1.0, 2.0, ARDOISE_CLAIRE);
    }
    // Les flaques, et leur reflet qui tremble.
    for (k, &(fx, fw)) in [(18.0f32, 14.0f32), (126.0, 18.0)].iter().enumerate() {
        rect(fx, sol + 2.0, fw, 2.0, PLUIE_SOMBRE);
        let d = ((anim / 30 + k as u32) % 3) as f32;
        rect(fx + 2.0 + d, sol + 2.0, 4.0, 1.0, PLUIE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pepin_core::levels::{niveaux_du_monde, GenreNiveau};

    /// Les bandes du chapitre 2 sont rangées du bas vers le haut, se touchent, couvrent
    /// toute la carte, et chaque étape du Monde 2 est dans la bande de son thème.
    #[test]
    fn les_bandes_du_chapitre_2_suivent_les_niveaux() {
        let tranche = niveaux_du_monde(niveaux::niveaux(), 2);
        let g = Geometrie::new(tranche);
        let bandes = bandes_chapitre_2(tranche, &g);
        assert_eq!(bandes.len(), 7);
        assert_eq!(bandes[0].0, Bande::Rue);
        assert_eq!(
            bandes[0].2, g.hauteur,
            "la rue va jusqu'au pied de la carte"
        );
        assert_eq!(bandes.last().unwrap(), &(Bande::Mer, 0.0, bandes[5].1));
        for w in bandes.windows(2) {
            assert_eq!(w[0].1, w[1].2, "{:?} et {:?} se touchent", w[0].0, w[1].0);
        }
        for &(b, haut, bas) in &bandes {
            assert!(haut < bas, "{b:?} a une hauteur");
        }
        for (n, &(_, y)) in tranche.iter().zip(&g.etapes) {
            if n.genre != GenreNiveau::Normal {
                continue;
            }
            let b = bande_du_theme(n.theme).expect("thème du Monde 2");
            let &(_, haut, bas) = bandes.iter().find(|x| x.0 == b).unwrap();
            assert!(
                (haut..=bas).contains(&y),
                "{} ({b:?}) : {y} hors de {haut}..{bas}",
                n.cle
            );
        }
        // La rivière, sans niveau pour l'instant, garde sa place sous le port.
        let riviere = bandes.iter().find(|x| x.0 == Bande::Riviere).unwrap();
        assert!(riviere.2 - riviere.1 >= 10.0, "{riviere:?}");
    }

    /// Le Monde 1 n'a aucun thème du chapitre 2. Carte 11.2 : le fond choisi par les
    /// thèmes redonne celui d'avant (choisi par le numéro du monde).
    #[test]
    fn aucun_theme_du_monde_1_n_a_de_bande() {
        for n in niveaux_du_monde(niveaux::niveaux(), 1) {
            assert_eq!(bande_du_theme(n.theme), None, "{}", n.cle);
        }
        assert!(!fond_en_descente(niveaux_du_monde(niveaux::niveaux(), 1)));
        assert!(fond_en_descente(niveaux_du_monde(niveaux::niveaux(), 2)));
    }

    /// Le faux Monde 3 (carte 11.2) a un niveau du caniveau parmi ses thèmes : sa
    /// carte se dessine en descente, avec des bandes qui couvrent toute la carte.
    #[test]
    fn le_faux_monde_3_a_un_fond_de_carte() {
        let t = crate::faux_monde_3::table_trois_mondes();
        let tranche = crate::faux_monde_3::faux_monde_3(&t);
        assert!(fond_en_descente(tranche));
        let g = Geometrie::new(tranche);
        let bandes = bandes_chapitre_2(tranche, &g);
        assert_eq!(bandes[0].2, g.hauteur, "la première bande touche le pied");
        assert_eq!(
            bandes.last().map(|b| b.1),
            Some(0.0),
            "la mer touche le haut"
        );
        for w in bandes.windows(2) {
            assert_eq!(w[0].1, w[1].2, "{:?} et {:?} se touchent", w[0].0, w[1].0);
        }
        // Sans thème du chapitre 2, un monde garde le fond de l'ascension.
        assert!(!fond_en_descente(&tranche[..2]));
    }

    #[test]
    fn le_hachage_est_stable() {
        assert_eq!(h(1), h(1));
        assert_ne!(h(1), h(2));
    }
}
