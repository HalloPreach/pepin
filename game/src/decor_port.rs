//! Le FOND DU PORT (carte 9.11b, demande d'AGA : « dans le port le fond est pas
//! assez fini »). Rendu seulement : la grille, la collision et le jeu ne changent
//! pas. Sert au thème `Port` tout entier : « Le port » et « La Pieuvre du port »
//! (la jetée de la Pieuvre peint ensuite sa propre mer sous son horizon,
//! `dessin_port::dessiner_fond`, et garde son phare, ses bateaux et son bassin).
//!
//! Du plus loin au plus près :
//! 1. le CIEL du crépuscule, en bandes de la palette : ardoise de nuit en haut,
//!    violet, puis la lueur corail à l'horizon (pas d'orange : l'or des gouttes
//!    doit se détacher du ciel) ; quelques étoiles qui
//!    scintillent, des mouettes de décor qui volent au loin (pas des ennemis) ;
//! 2. l'HORIZON MARIN : la mer, le reflet de la lueur, deux cargos en silhouette
//!    (l'un glisse très lentement) et le PHARE posé au bout d'une jetée lointaine,
//!    son faisceau qui tourne ;
//! 3. LE PORT, de l'autre côté de la darse : le quai d'en face, ses entrepôts aux
//!    fenêtres allumées, ses piles de conteneurs et ses grandes grues à portique
//!    (câbles tendus jusqu'au palonnier, feu de balisage qui clignote) ;
//! 4. les VOILIERS amarrés : leurs mâts et leurs haubans dépassent des quais ;
//! 5. PLUS PRÈS, sur l'eau de la darse (vue entre les quais jouables) : pontons sur
//!    pieux, filets qui sèchent, bouées, lampadaires et le reflet de leur halo qui
//!    ondule sur l'eau.
//!
//! Tout reste plus terne et plus sombre que le premier plan jouable : de grandes
//! masses d'ardoise de nuit, les couleurs vives en touches de quelques pixels. Les
//! animations sont lentes. La pluie se calme : une bruine fine et clairsemée
//! (`dessiner_bruine`).
//!
//! Sous la TOUR de la Pieuvre (carte 9.13c : « la bande du coucher de soleil du
//! fond se répète au milieu de la tour ») : le fond reste celui de l'arène, il
//! descend avec elle quand Pépin grimpe, et au-dessus de son ciel la nuit continue,
//! étoilée (`dessiner_fond_sous_la_tour`) : un fond CONTINU en hauteur.

use macroquad::prelude::draw_rectangle;
use pepin_core::palette::{
    Couleur, ARDOISE, ARDOISE_CLAIRE, ARDOISE_NUIT, ARDOISE_OMBRE, ARDOISE_PALE, BRUME, CORAIL,
    NUAGE, ORANGE_SOLEIL, OR_PALE, PLUIE_SOMBRE, VIOLET_ORAGE,
};
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use std::cell::Cell;

use crate::decor::{decalage, enfoncement};

thread_local! {
    /// Décalage vertical (px, vers le bas) de tout le fond du port : 0, sauf sous
    /// la tour de la Pieuvre (`dessiner_fond_sous_la_tour`). Une `Cell` dans un
    /// `thread_local!` : une variable globale qu'on peut changer sans `unsafe`.
    static DECALAGE: Cell<i32> = const { Cell::new(0) };
}
use crate::rendu::couleur_mq;

const L: i32 = LARGEUR_LOGIQUE as i32;
const H: i32 = HAUTEUR_LOGIQUE as i32;

/// Un rectangle de couleur : (x, y, largeur, hauteur, couleur), à l'écran.
pub(crate) type Rect = (i32, i32, i32, i32, Couleur);

/// Les bandes du ciel, de haut en bas : (bas de la bande en px, couleur). La
/// dernière s'arrête à l'horizon.
pub(crate) const CIEL: [(i32, Couleur); 5] = [
    (14, ARDOISE_OMBRE),
    (26, ARDOISE_NUIT),
    (36, ARDOISE),
    (46, VIOLET_ORAGE),
    (HORIZON, CORAIL),
];
/// L'horizon de la mer (px à l'écran, au pied du niveau) : sous lui la mer.
pub(crate) const HORIZON: i32 = 56;
/// Le dessus du quai d'en face (px à l'écran) : les entrepôts, les conteneurs et
/// les grues y sont posés, juste au-dessus du dessus des quais jouables (y = 80 au
/// « Port », l'horizon de la jetée de la Pieuvre) : tout le port se voit.
pub(crate) const QUAI_FACE: i32 = 76;
/// Le platelage des pontons du premier plan (px à l'écran), sur l'eau de la darse,
/// sous les pontons jouables (rangée 7) : on les voit entre les quais du niveau.
pub(crate) const PONTON: i32 = 126;
/// Facteurs de parallaxe : le large, le port d'en face, les voiliers, les pontons.
const F_LARGE: f32 = 0.05;
const F_PORT: f32 = 0.15;
const F_VOILIERS: f32 = 0.25;
const F_PONTONS: f32 = 0.35;
/// Largeur du motif qui se répète (port d'en face, voiliers, pontons).
const MOTIF: i32 = 640;
/// Le phare, au bout de sa jetée lointaine (abscisse de l'axe à l'écran, caméra 0).
pub(crate) const X_PHARE: i32 = 70;
/// Hauteur de la tour du phare (px), de la jetée à la galerie.
const TOUR: i32 = 22;
/// Portée du faisceau du phare (px) quand il passe de profil.
pub(crate) const PORTEE: i32 = 64;
/// Un tour de faisceau, en pas (6 s).
pub(crate) const TOUR_FAISCEAU: u32 = 360;
/// Le cargo qui glisse à l'horizon : un pixel tous les `LENTEUR_CARGO` pas.
pub(crate) const LENTEUR_CARGO: u32 = 24;

fn rect(x: i32, y: i32, w: i32, h: i32, c: Couleur) {
    let y = y + DECALAGE.with(Cell::get);
    if w > 0 && h > 0 && y < H && y + h > 0 {
        draw_rectangle(x as f32, y as f32, w as f32, h as f32, couleur_mq(c));
    }
}

fn poser(r: &[Rect]) {
    for &(x, y, w, h, c) in r {
        rect(x, y, w, h, c);
    }
}

/// Nombre pseudo-aléatoire stable dérivé d'un entier (même entrée, même sortie).
fn hache(n: i32) -> u32 {
    let mut h = (n as u32).wrapping_mul(0x9E37_79B1);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h
}

/// Couleur du ciel à l'ordonnée `y` (au-dessus de l'horizon). Les bandes se
/// fondent l'une dans l'autre sur deux rangées alternées (pas de demi-teinte).
pub(crate) fn ciel_a(y: i32) -> Couleur {
    let mut haut = 0;
    for (k, &(bas, c)) in CIEL.iter().enumerate() {
        if y < bas {
            // Les deux dernières rangées d'une bande : une sur deux prend la
            // couleur de la bande suivante.
            if k + 1 < CIEL.len() && y >= bas - 2 && (y - haut) % 2 == 1 {
                return CIEL[k + 1].1;
            }
            return c;
        }
        haut = bas;
    }
    CIEL[CIEL.len() - 1].1
}

/// Le faisceau du phare au pas `pas` : (sens, longueur en px). Il tourne : vu de
/// profil il porte loin, d'un côté puis de l'autre ; tourné vers nous ou de dos il
/// se réduit à la lampe.
pub(crate) fn faisceau(pas: u32) -> (i32, i32) {
    let a = (pas % TOUR_FAISCEAU) as f32 / TOUR_FAISCEAU as f32 * std::f32::consts::TAU;
    let c = a.cos();
    let sens = if c >= 0.0 { 1 } else { -1 };
    (sens, (c.abs() * PORTEE as f32) as i32)
}

/// Le phare et sa jetée lointaine, l'axe de la tour en `x`, la jetée posée sur
/// l'horizon : la tour (silhouette, galerie, lanterne) se dresse SUR le musoir.
pub(crate) fn phare(x: i32) -> Vec<Rect> {
    let jetee = HORIZON - 3;
    let pied = jetee;
    let galerie = pied - TOUR;
    let mut r = vec![
        // La jetée : une longue digue basse vers la droite, ses blocs, le musoir.
        (x - 6, jetee, 76, 3, ARDOISE_OMBRE),
        (x - 7, jetee + 1, 1, 2, ARDOISE_OMBRE),
        (x - 5, jetee - 1, 12, 1, ARDOISE_OMBRE),
    ];
    for k in 0..6 {
        r.push((x + 10 + k * 11, jetee - 1, 4, 1, ARDOISE_OMBRE));
    }
    // La tour, plus large au pied, posée sur le musoir.
    r.push((x - 3, pied - 6, 6, 6, ARDOISE_OMBRE));
    r.push((x - 2, galerie, 4, TOUR - 6, ARDOISE_OMBRE));
    // La galerie, la lanterne, le toit.
    r.push((x - 3, galerie - 1, 6, 1, ARDOISE_OMBRE));
    r.push((x - 1, galerie - 5, 2, 4, ARDOISE_OMBRE));
    r.push((x - 2, galerie - 6, 4, 1, ARDOISE_OMBRE));
    r
}

/// Le faisceau du phare dont la lampe est en `(x, yl)`, tourné du côté `sens`, long
/// de `long` px : un CÔNE de lumière terne qui s'évase en s'éloignant (une colonne
/// de pixels par pas), plus clair au cœur, jamais un trait seul ; l'or pâle reste à
/// la lampe.
pub(crate) fn cone(x: i32, yl: i32, sens: i32, long: i32) -> Vec<Rect> {
    let mut r = Vec::new();
    for d in 3..long {
        let px = if sens > 0 { x + d } else { x - 1 - d };
        let demi = 1 + d / 14;
        r.push((px, yl + 1 - demi, 1, 2 * demi + 1, ARDOISE_CLAIRE));
        if d < long * 2 / 3 {
            r.push((px, yl + 1, 1, 1, ARDOISE_PALE));
        }
    }
    r
}

/// Ordonnée de la lampe du phare (px à l'écran).
pub(crate) fn y_lampe() -> i32 {
    HORIZON - 3 - TOUR - 4
}

/// Une grande grue à portique posée sur le quai d'en face en `base` (le pied des
/// jambes), sa flèche au-dessus de l'eau vers la gauche. Le chariot, ses deux
/// câbles tendus et le palonnier qui tient un conteneur au-dessus du quai.
pub(crate) fn portique(x: i32, base: i32) -> Vec<Rect> {
    let haut = base - 46;
    let c = ARDOISE_OMBRE;
    let mut r = vec![
        // Les deux jambes et leur traverse.
        (x, haut, 2, base - haut, c),
        (x + 16, haut, 2, base - haut, c),
        (x, base - 14, 18, 2, c),
        // La poutre de la flèche (au-dessus de l'eau à gauche, contre-flèche à
        // droite), en treillis : deux membrures, des montants.
        (x - 34, haut - 1, 62, 1, c),
        (x - 34, haut + 2, 62, 1, c),
        // La cabine de la machinerie, le haubanage au sommet.
        (x + 18, haut - 5, 8, 5, c),
        (x + 4, haut - 9, 2, 8, c),
        (x + 12, haut - 9, 2, 8, c),
        (x + 4, haut - 10, 10, 1, c),
    ];
    for k in 0..16 {
        r.push((x - 34 + k * 4, haut, 1, 2, c));
    }
    // Le chariot, ses deux câbles tendus, le palonnier et son conteneur.
    let chariot = x - 22;
    let palonnier = base - 26;
    r.push((chariot, haut + 3, 6, 2, c));
    r.push((chariot + 1, haut + 5, 1, palonnier - haut - 5, c));
    r.push((chariot + 4, haut + 5, 1, palonnier - haut - 5, c));
    r.push((chariot - 2, palonnier, 10, 1, c));
    r.push((chariot - 3, palonnier + 1, 12, 5, ARDOISE));
    r.push((chariot - 3, palonnier + 5, 12, 1, ARDOISE_OMBRE));
    r
}

/// Le feu de balisage rouge du sommet d'une grue : allumé une seconde sur deux,
/// chaque grue à son rythme.
pub(crate) fn balise_allumee(pas: u32, k: i32) -> bool {
    ((pas + k as u32 * 37) / 60).is_multiple_of(2)
}

/// Un entrepôt du quai d'en face : (x, largeur, hauteur) dans le motif.
const ENTREPOTS: [(i32, i32, i32); 4] = [(96, 50, 20), (271, 60, 16), (470, 84, 22), (20, 40, 26)];
/// Les piles de conteneurs : (x, nombre de piles, hauteur de la plus haute).
const CONTENEURS: [(i32, i32, i32); 4] = [(156, 3, 4), (341, 3, 3), (566, 2, 4), (70, 2, 2)];
/// Les grues à portique : x dans le motif.
const PORTIQUES: [i32; 3] = [236, 400, 620];
/// Les voiliers amarrés : (x dans le motif, hauteur du mât).
const VOILIERS: [(i32, i32); 6] = [
    (20, 40),
    (130, 34),
    (235, 44),
    (345, 36),
    (450, 42),
    (555, 32),
];
/// Les lampadaires des pontons : x dans le motif.
const LAMPADAIRES: [i32; 3] = [60, 270, 480];

/// Abscisse à l'écran d'un élément placé en `x` dans un motif répété de `MOTIF`
/// px, la couche décalée de `dec` : la copie la plus proche du bord gauche.
fn dans_motif(x: i32, dec: i32) -> i32 {
    (x - dec).rem_euclid(MOTIF) - 96
}

fn dessiner_ciel(cam_x: i32, pas: u32) {
    for y in 0..HORIZON {
        rect(0, y, L, 1, ciel_a(y));
    }
    // Les étoiles, dans le haut du ciel seulement ; une sur sept s'éteint un
    // instant, chacune à son tour.
    let dec = decalage(cam_x, 0.02);
    for k in 0..18 {
        let h = hache(k * 7 + 3);
        let x = ((h % 640) as i32 - dec).rem_euclid(L);
        let y = 2 + (h >> 10) as i32 % 32;
        if (pas / 30 + k as u32).is_multiple_of(7) {
            continue;
        }
        let c = if k % 3 == 0 { NUAGE } else { ARDOISE_PALE };
        rect(x, y, 1, 1, c);
    }
}

fn dessiner_mouettes(cam_x: i32, pas: u32) {
    let dec = decalage(cam_x, 0.04);
    for k in 0..4i32 {
        let x = (k * 89 + 40 - (pas / (9 + k as u32 * 2)) as i32 - dec).rem_euclid(L + 40) - 20;
        let y = 10 + k * 5 + ((pas / 40 + k as u32) % 3) as i32;
        // Deux images de battement : ailes en V, ailes basses.
        let ailes = if ((pas / 14) + k as u32).is_multiple_of(2) {
            -1
        } else {
            1
        };
        rect(x - 2, y + ailes, 1, 1, ARDOISE_PALE);
        rect(x - 1, y, 3, 1, ARDOISE_PALE);
        rect(x + 2, y + ailes, 1, 1, ARDOISE_PALE);
    }
}

fn dessiner_mer(cam_x: i32, pas: u32) {
    rect(0, HORIZON, L, H - HORIZON, ARDOISE_NUIT);
    rect(0, HORIZON, L, 1, ARDOISE);
    // Le reflet de la lueur : de courts traits sur les premières rangées de la mer,
    // collés à l'horizon (corail sur la première, violets dessous).
    let dec = decalage(cam_x, F_LARGE);
    for k in 0..14i32 {
        let rang = k % 3;
        let y = HORIZON + 1 + rang;
        let w = 5 - rang;
        let x = (k * 47 + ((pas / 20) as i32 + k) % 3 - dec).rem_euclid(L);
        let c = if rang == 0 { CORAIL } else { VIOLET_ORAGE };
        rect(x, y, w, 1, c);
    }
    // La houle de la darse, plus bas : des traits d'ardoise qui dérivent.
    let dec = decalage(cam_x, F_PONTONS);
    for k in 0..16i32 {
        let y = QUAI_FACE + 8 + k * 4;
        let sens = if k % 2 == 0 { 1 } else { -1 };
        let x = (k * 53 + sens * (pas / 11) as i32 - dec).rem_euclid(L);
        rect(x, y, 4 + k % 3, 1, ARDOISE);
    }
}

fn dessiner_large(cam_x: i32, pas: u32, avec_phare: bool) {
    let dec = decalage(cam_x, F_LARGE);
    // Le cargo au mouillage : coque, piles de conteneurs, château, cheminée.
    let cargo = |x: i32, long: i32| {
        let w = HORIZON;
        rect(x, w - 3, long, 3, ARDOISE_OMBRE);
        rect(x + 1, w - 4, long - 2, 1, ARDOISE_OMBRE);
        rect(x + 4, w - 7, long - 16, 3, ARDOISE_OMBRE);
        rect(x + long - 10, w - 11, 6, 7, ARDOISE_OMBRE);
        rect(x + long - 9, w - 14, 2, 3, ARDOISE_OMBRE);
        rect(x + long - 9, w - 9, 1, 1, ORANGE_SOLEIL);
    };
    cargo(196 - dec, 38);
    // Le second cargo glisse très lentement vers la gauche, puis revient au large.
    let x = (330 - (pas / LENTEUR_CARGO) as i32 - dec).rem_euclid(L + 80) - 40;
    cargo(x, 28);
    if !avec_phare {
        return;
    }
    // Le phare sur sa jetée, et son faisceau qui tourne.
    let xp = X_PHARE - dec;
    poser(&phare(xp));
    let yl = y_lampe();
    let (sens, long) = faisceau(pas);
    poser(&cone(xp, yl, sens, long));
    // La lampe : plus grosse quand le faisceau est tourné vers nous.
    if long < PORTEE / 4 {
        rect(xp - 2, yl, 4, 3, OR_PALE);
    } else {
        rect(xp - 1, yl, 2, 3, OR_PALE);
    }
}

fn dessiner_port(cam_x: i32, cam_y: i32, pas: u32) {
    let dec = decalage(cam_x, F_PORT);
    let base = QUAI_FACE + enfoncement(cam_y, F_PORT);
    for (i, &(x0, w, h)) in ENTREPOTS.iter().enumerate() {
        let x = dans_motif(x0, dec);
        // Le corps, le toit en dents de scie, les fenêtres (quelques-unes allumées).
        rect(x, base - h, w, h, ARDOISE_NUIT);
        for k in 0..w / 10 {
            for d in 0..4 {
                rect(
                    x + k * 10 + d * 2,
                    base - h - 4 + d,
                    10 - d * 2,
                    1,
                    ARDOISE_NUIT,
                );
            }
        }
        for fx in 0..(w - 4) / 6 {
            for fy in 0..(h - 8) / 6 {
                let allume = hache(i as i32 * 101 + fx * 13 + fy * 7).is_multiple_of(4);
                let c = if allume { ORANGE_SOLEIL } else { ARDOISE_OMBRE };
                rect(x + 3 + fx * 6, base - h + 4 + fy * 6, 3, 2, c);
            }
        }
        // La grande porte du quai.
        rect(x + w / 2 - 5, base - 8, 10, 8, ARDOISE_OMBRE);
    }
    const TEINTES: [Couleur; 4] = [ARDOISE, VIOLET_ORAGE, ARDOISE_NUIT, ARDOISE];
    for (i, &(x0, piles, plus_haute)) in CONTENEURS.iter().enumerate() {
        let x = dans_motif(x0, dec);
        for p in 0..piles {
            let n = 1 + (hache(i as i32 * 17 + p) % plus_haute as u32) as i32;
            for e in 0..n {
                let (cx, cy) = (x + p * 15, base - 5 * (e + 1));
                let c = TEINTES[(hache(i as i32 * 31 + p * 5 + e) % 4) as usize];
                rect(cx, cy, 14, 5, c);
                // Les nervures et le liseré sombre.
                rect(cx, cy + 4, 14, 1, ARDOISE_OMBRE);
                for n in 0..4 {
                    rect(cx + 2 + n * 3, cy + 1, 1, 3, ARDOISE_OMBRE);
                }
            }
        }
    }
    for (k, &x0) in PORTIQUES.iter().enumerate() {
        let x = dans_motif(x0, dec);
        poser(&portique(x, base));
        if balise_allumee(pas, k as i32) {
            rect(x + 8, base - 46 - 11, 2, 1, CORAIL);
        }
    }
    // Le quai d'en face : son arête, son mur.
    rect(0, base, L, 4, ARDOISE_OMBRE);
    rect(0, base, L, 1, ARDOISE);
}

fn dessiner_voiliers(cam_x: i32, cam_y: i32, pas: u32) {
    let dec = decalage(cam_x, F_VOILIERS);
    let eau = QUAI_FACE + 14 + enfoncement(cam_y, F_VOILIERS);
    for (k, &(x0, mat)) in VOILIERS.iter().enumerate() {
        let x = dans_motif(x0, dec);
        // Le voilier se balance d'un pixel sur la houle.
        let houle = ((pas / (30 + k as u32 * 9)) % 2) as i32;
        let w = eau + houle;
        // La coque, sombre et effilée (aucun liseré clair : ce n'est pas une
        // plateforme), sa ligne de flottaison.
        rect(x - 12, w - 4, 26, 2, ARDOISE_NUIT);
        rect(x - 10, w - 2, 22, 2, ARDOISE_NUIT);
        rect(x - 10, w, 22, 1, ARDOISE_OMBRE);
        // La voile ferlée sur sa bôme.
        rect(x + 1, w - 12, 8, 2, ARDOISE);
        // Le mât, la bôme, et les deux haubans jusqu'à l'étrave et la poupe.
        let tete = w - 5 - mat;
        rect(x, tete, 1, mat, ARDOISE_OMBRE);
        rect(x, w - 10, 10, 1, ARDOISE_OMBRE);
        for d in 1..mat {
            let pente = d * 12 / mat;
            rect(x - pente, tete + d, 1, 1, ARDOISE_NUIT);
            rect(x + pente, tete + d, 1, 1, ARDOISE_NUIT);
        }
    }
}

fn dessiner_pontons(cam_x: i32, cam_y: i32, pas: u32) {
    let dec = decalage(cam_x, F_PONTONS);
    let pont = PONTON + enfoncement(cam_y, F_PONTONS);
    for (k, &x0) in LAMPADAIRES.iter().enumerate() {
        let x = dans_motif(x0, dec);
        // Le ponton : platelage sur ses pieux.
        // Sombre et sans arête claire : ce n'est pas une plateforme.
        rect(x - 40, pont, 90, 2, ARDOISE_NUIT);
        rect(x - 40, pont + 2, 90, 2, ARDOISE_OMBRE);
        for p in 0..8 {
            let px = x - 38 + p * 12;
            let haut = if p % 4 == 0 { pont - 10 } else { pont };
            rect(px, haut, 3, H - haut, ARDOISE_OMBRE);
        }
        // Le filet qui sèche entre les deux grands pieux, en mailles qui pendent.
        let (g, d) = (x - 38 + 3, x + 10);
        for m in 0..(d - g) / 3 {
            let creux = (m * (d - g) / 3 - m * m) / 12;
            for r in 0..3 {
                rect(g + m * 3 + r % 2, pont - 9 + creux + r * 2, 1, 1, ARDOISE);
            }
        }
        // La bouée amarrée devant le ponton : elle danse sur l'eau.
        let bx = x + 30;
        let by = pont + 12 + ((pas / 25 + k as u32) % 2) as i32;
        rect(bx, by, 5, 4, VIOLET_ORAGE);
        rect(bx, by + 1, 5, 1, ARDOISE_PALE);
        rect(bx + 2, by - 2, 1, 2, ARDOISE_OMBRE);
        // Le lampadaire, son halo, et le reflet du halo qui ondule sur l'eau.
        let lx = x + 18;
        let tete = pont - 26;
        for (r, c) in [(7, ARDOISE), (4, ARDOISE_CLAIRE)] {
            for dy in -r..=r {
                let demi = (((r * r - dy * dy) as f32).sqrt() + 0.3) as i32;
                rect(lx + 1 - demi, tete + 2 + dy, 2 * demi, 1, c);
            }
        }
        rect(lx, tete + 4, 2, pont - tete - 4, ARDOISE_OMBRE);
        rect(lx - 1, tete - 1, 4, 1, ARDOISE_OMBRE);
        rect(lx - 1, tete, 4, 4, ARDOISE_OMBRE);
        rect(lx, tete + 1, 2, 2, BRUME);
        for r in 0..5i32 {
            let ondule = [0, 1, 0, -1][((pas / 12) as i32 + r).rem_euclid(4) as usize];
            let w = 6 - r;
            rect(
                lx + 1 - w / 2 + ondule,
                pont + 6 + r * 3,
                w,
                1,
                ARDOISE_CLAIRE,
            );
        }
    }
}

/// Dessine tout le fond du port (derrière le niveau). `cam_y` : la caméra vue par
/// le fond (`decor::camera_fond`). `avec_phare` : faux sur la jetée de la Pieuvre,
/// qui dessine son propre phare (`dessin_port`).
pub fn dessiner_fond(cam_x: i32, cam_y: i32, pas: u32, avec_phare: bool) {
    dessiner_ciel(cam_x, pas);
    dessiner_mouettes(cam_x, pas);
    dessiner_mer(cam_x, pas);
    dessiner_large(cam_x, pas, avec_phare);
    dessiner_port(cam_x, cam_y, pas);
    dessiner_voiliers(cam_x, cam_y, pas);
    dessiner_pontons(cam_x, cam_y, pas);
}

/// Le fond du port sous la tour de la Pieuvre (carte 9.13c) : `monte` px de
/// caméra au-dessus du bas du niveau (0 dans l'arène). Le port de l'arène descend
/// d'autant, et au-dessus de son ciel (dont le haut est déjà l'ardoise la plus
/// sombre), la NUIT continue, de la même ardoise, avec des étoiles qui
/// descendent avec lui : aucune bande ne se répète, le fond est continu.
pub fn dessiner_fond_sous_la_tour(cam_x: i32, pas: u32, monte: i32) {
    let monte = monte.max(0);
    rect(0, 0, L, monte.min(H), ARDOISE_OMBRE);
    // Les étoiles de la nuit, posées au-dessus du ciel du port (y < 0 dans son
    // repère), plus serrées en montant.
    let dec = decalage(cam_x, 0.02);
    for k in 0..90 {
        let h = hache(k * 13 + 5);
        let y = -2 - (h >> 9) as i32 % 1100;
        let ecran = y + monte;
        if !(0..H).contains(&ecran) || (h % 5) as i32 * 300 > -y {
            continue;
        }
        if (pas / 30 + k as u32).is_multiple_of(9) {
            continue;
        }
        let x = ((h % 640) as i32 - dec).rem_euclid(L);
        let c = if k % 4 == 0 { NUAGE } else { ARDOISE_PALE };
        rect(x, ecran, 1, 1, c);
    }
    DECALAGE.with(|d| d.set(monte));
    dessiner_fond(cam_x, 0, pas, false);
    DECALAGE.with(|d| d.set(0));
}

/// La bruine du port, devant tout : des gouttes fines et clairsemées qui tombent
/// lentement. `forte` (nouvelle partie +) : deux fois plus de gouttes.
pub fn dessiner_bruine(pas: u32, forte: bool) {
    let n = if forte { 24 } else { 12 };
    for i in 0..n {
        let h = hache(i * 11 + 5);
        let x = ((h % L as u32) as i32 - (pas as i32 + (h >> 8) as i32 % 300) / 4).rem_euclid(L);
        let y = ((h >> 12) as i32 + pas as i32 * 2).rem_euclid(H);
        rect(x, y, 1, 2, PLUIE_SOMBRE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    /// Le ciel du crépuscule : sombre en haut, la lueur chaude à l'horizon, et les
    /// bandes descendent dans l'ordre.
    #[test]
    fn le_ciel_va_de_la_nuit_a_la_lueur_de_l_horizon() {
        assert_eq!(ciel_a(0), ARDOISE_OMBRE);
        assert_eq!(ciel_a(HORIZON - 1), CORAIL);
        assert!(CIEL.windows(2).all(|w| w[0].0 < w[1].0));
        assert_eq!(CIEL[CIEL.len() - 1].0, HORIZON);
        // Le ciel tient dans le haut de l'écran, au-dessus des quais jouables.
        const { assert!(HORIZON < QUAI_FACE && QUAI_FACE < PONTON && PONTON < H) };
    }

    /// Le phare est POSÉ : sa tour touche le musoir de sa jetée, d'un seul tenant
    /// de la lanterne au pied, et la jetée est sur l'horizon.
    #[test]
    fn le_phare_est_pose_au_bout_de_sa_jetee() {
        let x = 100;
        let p = pixels(&phare(x));
        let jetee = HORIZON - 3;
        assert!((x - 5..x + 60).all(|px| p.contains_key(&(px, jetee))));
        assert!((jetee..HORIZON).all(|y| p.contains_key(&(x, y))));
        // La tour, sans trou, de la lampe à la jetée.
        assert!(
            (y_lampe() - 2..jetee).all(|y| p.contains_key(&(x, y))),
            "tour"
        );
    }

    /// Le faisceau tourne : il porte d'un côté puis de l'autre, jamais plus loin que
    /// sa portée, et se réduit à la lampe quand il nous fait face.
    #[test]
    fn le_faisceau_du_phare_tourne() {
        let tous: Vec<_> = (0..TOUR_FAISCEAU).map(faisceau).collect();
        assert!(tous.iter().all(|&(_, l)| (0..=PORTEE).contains(&l)));
        assert!(tous.iter().any(|&(s, l)| s > 0 && l == PORTEE));
        assert!(tous.iter().any(|&(s, l)| s < 0 && l >= PORTEE - 1));
        assert!(tous.iter().any(|&(_, l)| l < 2));
        // Lentement : d'un pas au suivant, le faisceau ne saute pas de plus de 2 px.
        assert!(tous.windows(2).all(|w| {
            let a = w[0].0 * w[0].1;
            let b = w[1].0 * w[1].1;
            (a - b).abs() <= 2
        }));
    }

    /// Les câbles de la grue sont TENDUS : d'un seul tenant du chariot sous la
    /// flèche jusqu'au palonnier, et les jambes touchent le quai.
    #[test]
    fn les_cables_de_la_grue_vont_de_la_fleche_au_palonnier() {
        let (x, base) = (100, 84);
        let p = pixels(&portique(x, base));
        let haut = base - 46;
        let chariot = x - 22;
        for cable in [chariot + 1, chariot + 4] {
            assert!(
                (haut + 2..base - 26).all(|y| p.contains_key(&(cable, y))),
                "câble"
            );
        }
        assert!(p.contains_key(&(x, base - 1)) && p.contains_key(&(x + 16, base - 1)));
        // La flèche est d'un seul tenant au-dessus de l'eau.
        assert!((x - 34..x + 28).all(|px| p.contains_key(&(px, haut - 1))));
    }

    /// Le faisceau est un cône qui s'évase (pas un trait jaune qui flotte) : plus
    /// haut au bout qu'à la lampe, d'un seul tenant, sans or (l'or pâle reste à la
    /// lampe).
    #[test]
    fn le_faisceau_est_un_cone_terne() {
        for sens in [-1, 1] {
            let p = pixels(&cone(100, 30, sens, PORTEE));
            assert!(p
                .values()
                .all(|&c| c == ARDOISE_CLAIRE || c == ARDOISE_PALE));
            let hauteur = |x: i32| p.keys().filter(|k| k.0 == x).count();
            let (pres, loin) = if sens > 0 {
                (103, 100 + PORTEE - 1)
            } else {
                (96, 99 - PORTEE + 1)
            };
            assert!(hauteur(loin) > hauteur(pres), "sens {sens}");
            let xs: std::collections::BTreeSet<i32> = p.keys().map(|k| k.0).collect();
            assert_eq!(xs.len() as i32, PORTEE - 3, "sens {sens}");
        }
    }

    /// Le cargo glisse très lentement ; les feux de balisage clignotent chacun à son
    /// rythme.
    #[test]
    fn la_vie_du_port_est_lente() {
        const { assert!(LENTEUR_CARGO >= 20) };
        assert_ne!(balise_allumee(0, 0), balise_allumee(60, 0));
        assert!((0..120).any(|p| balise_allumee(p, 0) != balise_allumee(p, 1)));
    }
}
