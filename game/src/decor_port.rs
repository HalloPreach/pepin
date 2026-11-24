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
//! Carte 10.4d : le ciel, la mer unie, le port d'en face et les pontons sont des
//! fichiers de `assets/fonds/port` (FIXES) ; tout le reste est CALCULÉ. L'ordre est
//! la pile `PILE` (voir `Plan`), la même que `decor::pile` et le LISEZMOI.
//!
//! Tout reste plus terne et plus sombre que le premier plan jouable : de grandes
//! masses d'ardoise de nuit, les couleurs vives en touches de quelques pixels. Les
//! animations sont lentes. La pluie se calme : une bruine fine et clairsemée
//! (`dessiner_bruine`).
//!
//! Sous la TOUR de la Pieuvre (carte 9.13c : « la bande du coucher de soleil du
//! fond se répète au milieu de la tour ») : le fond reste celui de l'arène, il
//! descend avec elle quand Pépin grimpe, et au-dessus de son ciel la nuit continue,
//! étoilée (`dessiner_fond_sous_la_tour`) : un fond CONTINU en hauteur. Carte
//! 9.13e : au-dessus de lui, le PORT LOINTAIN (phare et faisceau, grues,
//! cargos, lumières), tout petit, qui s'enfonce lentement vers le bas de
//! l'écran pendant la montée, sous un crépuscule qui devient nuit, et la lune
//! tout en haut.

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

/// Le feu de balisage rouge du sommet d'une grue : allumé une seconde sur deux,
/// chaque grue à son rythme.
pub(crate) fn balise_allumee(pas: u32, k: i32) -> bool {
    ((pas + k as u32 * 37) / 60).is_multiple_of(2)
}

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

fn dessiner_etoiles(cam_x: i32, pas: u32) {
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

fn dessiner_reflets(cam_x: i32, pas: u32) {
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

/// Le feu de balisage de chaque grue du port d'en face (posée dans
/// `port/port_d_en_face.png`) : un point corail au-dessus de sa cabine.
fn dessiner_balises(cam_x: i32, cam_y: i32, pas: u32) {
    let dec = decalage(cam_x, F_PORT);
    let base = QUAI_FACE + enfoncement(cam_y, F_PORT);
    for (k, &x0) in PORTIQUES.iter().enumerate() {
        let x = dans_motif(x0, dec);
        if balise_allumee(pas, k as i32) {
            rect(x + 8, base - 46 - 11, 2, 1, CORAIL);
        }
    }
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

/// Les bouées amarrées devant les pontons (`port/pontons.png`), qui dansent sur
/// l'eau, et le reflet du halo des lampadaires, qui ondule.
fn dessiner_bouees(cam_x: i32, cam_y: i32, pas: u32) {
    let dec = decalage(cam_x, F_PONTONS);
    let pont = PONTON + enfoncement(cam_y, F_PONTONS);
    for (k, &x0) in LAMPADAIRES.iter().enumerate() {
        let x = dans_motif(x0, dec);
        // La bouée amarrée devant le ponton : elle danse sur l'eau.
        let bx = x + 30;
        let by = pont + 12 + ((pas / 25 + k as u32) % 2) as i32;
        rect(bx, by, 5, 4, VIOLET_ORAGE);
        rect(bx, by + 1, 5, 1, ARDOISE_PALE);
        rect(bx + 2, by - 2, 1, 2, ARDOISE_OMBRE);
        let lx = x + 18;
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
    for plan in PILE {
        dessiner_plan(plan, cam_x, cam_y, pas, avec_phare);
    }
}

/// Les plans du fond du port, du plus loin au plus près (carte 10.4d). Quatre sont
/// des fichiers de `assets/fonds/port` (FIXES : seul leur placement est calculé) ;
/// les autres sont CALCULÉS (animés, ou qui suivent le temps). `decor::pile` les
/// reprend tels quels pour le thème `Port`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    /// Le ciel du crépuscule, `port/crepuscule.png` : une colonne étirée.
    Crepuscule,
    /// Les étoiles qui s'éteignent chacune à son tour.
    Etoiles,
    /// Les mouettes de décor qui volent au loin.
    Mouettes,
    /// La mer unie et sa ligne d'horizon, `port/mer.png` : une colonne étirée.
    Mer,
    /// Le reflet de la lueur et la houle de la darse, qui dérivent.
    Reflets,
    /// Le large : les deux cargos (l'un glisse), le phare et son faisceau.
    Large,
    /// Le port d'en face (entrepôts, conteneurs, grues, quai),
    /// `port/port_d_en_face.png` : UNE période de `MOTIF` px, répétée par le code.
    PortDEnFace,
    /// Les feux de balisage des grues, qui clignotent.
    Balises,
    /// Les voiliers amarrés, bercés par la houle.
    Voiliers,
    /// Les pontons, leurs filets et leurs lampadaires, `port/pontons.png` : UNE
    /// période de `MOTIF` px.
    Pontons,
    /// Les bouées qui dansent et le reflet des halos qui ondule.
    Bouees,
}

/// La pile du fond du port (lue par `dessiner_fond` et par `decor::pile`).
pub const PILE: [Plan; 11] = [
    Plan::Crepuscule,
    Plan::Etoiles,
    Plan::Mouettes,
    Plan::Mer,
    Plan::Reflets,
    Plan::Large,
    Plan::PortDEnFace,
    Plan::Balises,
    Plan::Voiliers,
    Plan::Pontons,
    Plan::Bouees,
];

/// Haut de `port/port_d_en_face.png` au-dessus du dessus du quai d'en face : le
/// haubanage des grues.
const HAUT_PORT_D_EN_FACE: i32 = 56;
/// Haut de `port/pontons.png` au-dessus du platelage : le halo des lampadaires.
const HAUT_PONTONS: i32 = 30;

impl Plan {
    /// Le nom du fichier de `assets/fonds/port` (`None` : plan calculé).
    #[cfg(test)]
    pub fn fichier(self) -> Option<&'static str> {
        match self {
            Plan::Crepuscule => Some("crepuscule"),
            Plan::Mer => Some("mer"),
            Plan::PortDEnFace => Some("port_d_en_face"),
            Plan::Pontons => Some("pontons"),
            _ => None,
        }
    }

    /// Le nom du plan dans `assets/fonds/LISEZMOI.md`.
    #[cfg(test)]
    pub fn nom(self) -> &'static str {
        match self {
            Plan::Crepuscule => "crépuscule",
            Plan::Etoiles => "étoiles du port",
            Plan::Mouettes => "mouettes",
            Plan::Mer => "mer",
            Plan::Reflets => "reflets",
            Plan::Large => "large",
            Plan::PortDEnFace => "port d'en face",
            Plan::Balises => "balises",
            Plan::Voiliers => "voiliers",
            Plan::Pontons => "pontons",
            Plan::Bouees => "bouées",
        }
    }
}

/// Pose une couche FIXE qui se répète tous les `MOTIF` px (`port_d_en_face` ou
/// `pontons`), décalée de `dec` : la colonne `c` du fichier est le point `c` du
/// motif, que `dans_motif` place à l'écran en `c - 96 - dec` (à `MOTIF` près).
fn poser_motif(nom: &'static str, dec: i32, y: i32) {
    let y = y + DECALAGE.with(Cell::get);
    let mut x = (-96 - dec).rem_euclid(MOTIF) - MOTIF;
    while x < L {
        crate::fonds::dessiner("port", nom, x, y);
        x += MOTIF;
    }
}

/// Dessine un plan du fond du port (voir `dessiner_fond`).
pub fn dessiner_plan(plan: Plan, cam_x: i32, cam_y: i32, pas: u32, avec_phare: bool) {
    let decale = DECALAGE.with(Cell::get);
    match plan {
        Plan::Crepuscule => crate::fonds::repeter("port", "crepuscule", 0, decale, L),
        Plan::Etoiles => dessiner_etoiles(cam_x, pas),
        Plan::Mouettes => dessiner_mouettes(cam_x, pas),
        Plan::Mer => crate::fonds::repeter("port", "mer", 0, HORIZON + decale, L),
        Plan::Reflets => dessiner_reflets(cam_x, pas),
        Plan::Large => dessiner_large(cam_x, pas, avec_phare),
        Plan::PortDEnFace => {
            let base = QUAI_FACE + enfoncement(cam_y, F_PORT);
            let dec = decalage(cam_x, F_PORT);
            poser_motif("port_d_en_face", dec, base - HAUT_PORT_D_EN_FACE);
        }
        Plan::Balises => dessiner_balises(cam_x, cam_y, pas),
        Plan::Voiliers => dessiner_voiliers(cam_x, cam_y, pas),
        Plan::Pontons => {
            let pont = PONTON + enfoncement(cam_y, F_PONTONS);
            poser_motif("pontons", decalage(cam_x, F_PONTONS), pont - HAUT_PONTONS);
        }
        Plan::Bouees => dessiner_bouees(cam_x, cam_y, pas),
    }
}

/// Le fond du port sous la tour de la Pieuvre (carte 9.13c) : `monte` px de
/// caméra au-dessus du bas du niveau (0 dans l'arène). Le port de l'arène descend
/// d'autant, et au-dessus de son ciel (dont le haut est déjà l'ardoise la plus
/// sombre), la NUIT continue, de la même ardoise, avec des étoiles qui
/// descendent avec lui : aucune bande ne se répète, le fond est continu.
pub fn dessiner_fond_sous_la_tour(cam_x: i32, pas: u32, monte: i32) {
    let monte = monte.max(0);
    // Carte 9.13e (AGA : « le fond est vide […] le phare ou la grue visibles
    // pendant la montée donneraient la sensation de hauteur ») : au-dessus du
    // port de l'arène (qui descend toujours avec elle), le PORT LOINTAIN, en
    // petit : son horizon suit d'abord le haut du ciel de l'arène, puis, posé
    // à `HORIZON_LOIN`, ne descend plus que très lentement (`F_LOIN`) jusqu'en
    // bas de l'écran ; le crépuscule au-dessus de lui s'éteint en montant
    // (`nuit`), les étoiles s'allument, la lune se lève tout en haut.
    let horizon = horizon_lointain(monte);
    let nuit = (monte as f32 / MONTEE_TOUR as f32).clamp(0.0, 1.0);
    rect(0, 0, L, horizon.min(H), ARDOISE_OMBRE);
    // La lueur du crépuscule, de plus en plus mince : ardoise, violet, corail.
    let reste = |n: f32| ((1.0 - nuit) * n).round() as i32;
    let lueur = [
        (ARDOISE, 2 + reste(10.0)),
        (VIOLET_ORAGE, 1 + reste(6.0)),
        (CORAIL, reste(3.0)),
    ];
    let mut y = horizon;
    for (c, e) in lueur.iter().rev() {
        y -= e;
        rect(0, y, L, *e, *c);
    }
    let ciel_noir = y;
    // Les étoiles, au-dessus de la lueur, plus nombreuses à mesure qu'elle
    // s'éteint ; elles glissent un peu avec la montée (`F_ETOILES`).
    let dec = decalage(cam_x, 0.02);
    let n = 30 + (60.0 * nuit) as i32;
    for k in 0..n {
        let h = hache(k * 13 + 5);
        let ecran = ((h >> 9) as i32 + (monte as f32 * F_ETOILES) as i32).rem_euclid(H + 40) - 20;
        if !(0..ciel_noir - 2).contains(&ecran) || (pas / 30 + k as u32).is_multiple_of(9) {
            continue;
        }
        let x = ((h % 640) as i32 - dec).rem_euclid(L);
        let c = if k % 4 == 0 { NUAGE } else { ARDOISE_PALE };
        rect(x, ecran, 1, 1, c);
    }
    dessiner_lune(cam_x, monte);
    dessiner_port_lointain(cam_x, horizon, pas);
    // Le port de l'arène, tant qu'il en reste à l'écran (son ciel compris).
    if monte < H {
        DECALAGE.with(|d| d.set(monte));
        dessiner_fond(cam_x, 0, pas, false);
        DECALAGE.with(|d| d.set(0));
    }
}

/// Horizon du port lointain (px à l'écran) quand la caméra a monté de `monte`
/// px dans la tour : juste au-dessus du ciel de l'arène qui descend, puis posé
/// à `HORIZON_LOIN`, il ne descend plus que de `F_LOIN` px par px de montée.
/// Continu (aucun saut). Pure.
pub(crate) fn horizon_lointain(monte: i32) -> i32 {
    let colle = monte - 4;
    if colle <= HORIZON_LOIN {
        colle
    } else {
        HORIZON_LOIN + ((colle - HORIZON_LOIN) as f32 * F_LOIN) as i32
    }
}

/// Le port lointain sur son horizon `horizon` (px à l'écran) : la mer en
/// dessous, le phare et son faisceau qui tourne, les grues, les cargos (l'un
/// glisse), les lumières du quai et leur reflet. Tout petit, terne : ardoise de
/// nuit, des touches de lumière d'un pixel.
fn dessiner_port_lointain(cam_x: i32, horizon: i32, pas: u32) {
    if horizon >= H {
        return;
    }
    let dec = decalage(cam_x, 0.01);
    let y = horizon;
    rect(0, y, L, H - y, ARDOISE_NUIT);
    rect(0, y, L, 1, ARDOISE);
    // Le quai lointain, une ligne d'entrepôts bas.
    for &(x0, w, h) in &[(96, 40, 3), (150, 24, 5), (226, 52, 4), (296, 30, 3)] {
        rect(x0 - dec, y - h, w, h, ARDOISE_OMBRE);
    }
    // Les grues : mât, flèche vers la gauche, câble, feu rouge qui clignote.
    for (k, &x0) in [140, 206, 250].iter().enumerate() {
        let x = x0 - dec;
        let haut = 13 + 3 * (k as i32 % 2);
        rect(x, y - haut, 2, haut, ARDOISE_OMBRE);
        rect(x - 14, y - haut, 20, 1, ARDOISE_OMBRE);
        rect(x - 11, y - haut + 1, 1, haut - 6, ARDOISE);
        if (pas / 40 + k as u32).is_multiple_of(2) {
            rect(x, y - haut - 1, 1, 1, CORAIL);
        }
    }
    // Deux cargos au mouillage : coque, château, cheminée ; le second glisse.
    let cargo = |x: i32, long: i32| {
        rect(x, y - 2, long, 2, ARDOISE_OMBRE);
        rect(x + long - 7, y - 5, 4, 3, ARDOISE_OMBRE);
        rect(x + 2, y - 3, long - 11, 1, ARDOISE_OMBRE);
        rect(x + long - 6, y - 4, 1, 1, ORANGE_SOLEIL);
    };
    cargo(26 - dec, 22);
    cargo(
        (300 - (pas / (2 * LENTEUR_CARGO)) as i32 - dec).rem_euclid(L + 40) - 20,
        16,
    );
    // Les lumières du quai : des points or pâle, quelques-uns qui vacillent,
    // leur reflet d'un pixel dans la mer.
    for k in 0..14 {
        let h = hache(k * 7 + 3);
        let x = 90 + (h % 220) as i32 - dec;
        if (pas / 20 + h).is_multiple_of(7) {
            continue;
        }
        let c = if k % 3 == 0 { ORANGE_SOLEIL } else { OR_PALE };
        rect(x, y - 2 - (h >> 8) as i32 % 2, 1, 1, c);
        rect(x, y + 2 + (k % 2), 1, 1, ARDOISE_CLAIRE);
    }
    // Le phare, sur sa jetée lointaine, et son faisceau qui tourne (deux fois
    // plus court que celui de l'arène).
    let xp = X_PHARE - dec;
    rect(xp - 4, y - 1, 26, 1, ARDOISE_OMBRE);
    rect(xp - 1, y - 11, 3, 10, ARDOISE_OMBRE);
    rect(xp - 1, y - 8, 3, 1, ARDOISE_PALE);
    rect(xp - 1, y - 5, 3, 1, ARDOISE_PALE);
    let (sens, long) = faisceau(pas);
    let yl = y - 13;
    for d in 2..long / 2 {
        let px = if sens > 0 { xp + 1 + d } else { xp - d };
        rect(px, yl, 1, 1 + d / 12, ARDOISE_CLAIRE);
    }
    rect(xp - 1, yl - 1, 3, 2, OR_PALE);
}

/// Parallaxe des étoiles au-dessus du port lointain.
const F_ETOILES: f32 = 0.30;
/// Le port lointain se pose à 150 px du haut de l'écran…
const HORIZON_LOIN: i32 = 150;
/// … puis ne descend plus que de 1 px pour 60 px de montée (16 px pour toute
/// la tour : il finit au ras du bas de l'écran, sous la bande d'écume).
const F_LOIN: f32 = 1.0 / 60.0;
/// La montée de la caméra dans la tour (px) : 80 rangées moins un écran.
const MONTEE_TOUR: i32 = 80 * 16 - H;

/// La lune, tout en haut de la tour : un disque de nuage, son ombre et deux
/// mers de brume, qui descend dans l'écran pendant le dernier tiers de la
/// montée.
fn dessiner_lune(cam_x: i32, monte: i32) {
    let y = 18 - (MONTEE_TOUR - monte) / 8;
    let x = 250 - decalage(cam_x, 0.02);
    if y < -8 {
        return;
    }
    for dy in -6..=6i32 {
        let demi = (((36 - dy * dy) as f32).sqrt() + 0.3) as i32;
        rect(x - demi, y + dy, 2 * demi, 1, NUAGE);
        let ombre = (demi - 2).max(0) / 2;
        rect(x + demi - ombre, y + dy, ombre, 1, BRUME);
    }
    rect(x - 3, y - 2, 2, 2, BRUME);
    rect(x, y + 2, 3, 2, BRUME);
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

    /// Le ciel du crépuscule (`port/crepuscule.png`, carte 10.4d) : sombre en haut,
    /// la lueur chaude à l'horizon ; la mer (`port/mer.png`) commence par la ligne
    /// d'horizon et descend jusqu'en bas de l'écran.
    #[test]
    fn le_ciel_va_de_la_nuit_a_la_lueur_de_l_horizon() {
        use crate::fonds::{couleur, taille};
        assert_eq!(taille("port", "crepuscule"), (1, HORIZON));
        assert_eq!(couleur("port", "crepuscule", 0, 0), Some(ARDOISE_OMBRE));
        let bas = HORIZON as usize - 1;
        assert_eq!(couleur("port", "crepuscule", 0, bas), Some(CORAIL));
        assert_eq!(taille("port", "mer"), (1, H - HORIZON));
        assert_eq!(couleur("port", "mer", 0, 0), Some(ARDOISE));
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
    /// flèche jusqu'au palonnier, et les jambes touchent le quai. Carte 10.4d : vu
    /// dans `port/port_d_en_face.png`, où la colonne `x` est le point `x` du motif
    /// et la rangée `HAUT_PORT_D_EN_FACE` le dessus du quai.
    #[test]
    fn les_cables_de_la_grue_vont_de_la_fleche_au_palonnier() {
        let (x, base) = (PORTIQUES[0], HAUT_PORT_D_EN_FACE);
        let (l, h) = crate::fonds::taille("port", "port_d_en_face");
        assert_eq!(l, MOTIF);
        let mut p = std::collections::HashMap::new();
        for py in 0..h {
            for px in 0..l {
                if let Some(c) =
                    crate::fonds::couleur("port", "port_d_en_face", px as usize, py as usize)
                {
                    p.insert((px, py), c);
                }
            }
        }
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

    /// Carte 9.13e : l'horizon du port lointain descend sans jamais sauter ni
    /// remonter, d'abord avec le ciel de l'arène, puis très lentement ; en haut
    /// de la tour, il est encore à l'écran, dans son bas.
    #[test]
    fn le_port_lointain_descend_sans_sauter() {
        let mut avant = horizon_lointain(0);
        for m in 1..=MONTEE_TOUR {
            let h = horizon_lointain(m);
            assert!(h >= avant && h - avant <= 1, "{m} : {avant} puis {h}");
            avant = h;
        }
        assert!((HORIZON_LOIN..H).contains(&avant), "{avant}");
    }
}
