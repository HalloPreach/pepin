//! Habillage des ÉGOUTS (carte 9.20b) : le niveau ressemble à son nom et se raccorde
//! au caniveau (avant) et à la cascade (après). Rendu seulement : aucune collision,
//! la grille et la mécanique ne changent pas.
//!
//! 1. Le DÉPART : Pépin est tombé par la GRILLE de la bouche d'égout. Elle est au
//!    plafond juste au-dessus de lui, éclairée par le lampadaire de la rue : un rai
//!    de lumière orange (pixels clairsemés, sans transparence) et la pluie qui tombe
//!    par la grille jusqu'au quai.
//! 2. Les BASSINS qui se remplissent (la marée `=` de la carte 9.3, même mécanique) :
//!    de gros TUYAUX sortent du plafond au-dessus de chaque bassin. Ils gargouillent
//!    (gouttes) pendant l'annonce, puis se vident par à-coups (jet d'eau) pendant que
//!    l'eau monte. Les repères qui clignotent (`eau::dessiner_marees`) restent.
//! 3. D'autres GRILLES au plafond le long du niveau, chacune avec son filet de pluie.
//! 4. La SORTIE : les dernières cases sont l'intérieur d'un GRAND TUYAU rond (paroi
//!    de béton, joints des anneaux) ; au bout, sa lèvre, et dehors le ciel de nuit,
//!    le haut d'une falaise et la chute d'eau : le niveau suivant, la cascade.
//!
//! Tout se place d'après la grille (plafond, quai, zones des bassins, drapeau) : les
//! fonctions de placement (`grilles`, `tuyaux`, `grand_tuyau`) sont pures et testées.
//! La scène appelle `dessiner` juste après les tuiles ; rien hors du thème Égouts.

use macroquad::prelude::draw_rectangle;
use pepin_core::aabb::TAILLE_TUILE;
use pepin_core::entities::maree::{
    Maree, ANNONCE_MAREE_PAS, BASSE_MER_PAS, MONTEE_PAS, PERIODE_MAREE,
};
use pepin_core::entities::Entite;
use pepin_core::level::{Niveau, Tuile};
use pepin_core::palette::{
    Couleur, ARDOISE, ARDOISE_CLAIRE, ARDOISE_NUIT, ARDOISE_OMBRE, ARDOISE_PALE, BLANC_CASSE,
    BRUME, ORANGE_SOLEIL, OR_PALE, PLUIE, PLUIE_SOMBRE, VIOLET_ORAGE,
};
use pepin_core::physics::JOUEUR_L;
use pepin_core::zones::Theme;

use crate::monde::Monde;
use crate::rendu::couleur_mq;

const T: i32 = TAILLE_TUILE as i32;

/// Largeur d'une grille du plafond (px) : deux cases.
const LARGEUR_GRILLE: i32 = 32;
/// Décalages (px) essayés pour la grille du départ, depuis le milieu de Pépin.
const DECALAGES_GRILLE_DEPART: [i32; 4] = [48, 32, 16, 0];
/// Écart minimal entre deux grilles (en cases).
const ECART_GRILLES: i32 = 24;
/// Le grand tuyau de la sortie commence à tant de cases du bord droit du niveau...
const DEBUT_TUYAU: i32 = 14;
/// ... et sa lèvre (la bouche) est à tant de cases du bord droit.
const BOUCHE_TUYAU: i32 = 5;
/// Un tuyau de bassin par tranche de tant de cases de bassin (au moins un).
const CASES_PAR_TUYAU: i32 = 5;
/// Diamètre d'un tuyau de bassin (px).
const DIAMETRE_TUYAU: i32 = 12;
/// Hauteur de la bouche évasée d'un tuyau de bassin (px).
const BOUCHE: i32 = 5;
/// Durée d'un à-coup du jet (pas) : le tuyau se vide en 4 à-coups pendant la montée.
const A_COUP_PAS: u32 = 30;

fn rect(x: i32, y: i32, w: i32, h: i32, c: Couleur) {
    if w > 0 && h > 0 {
        draw_rectangle(x as f32, y as f32, w as f32, h as f32, couleur_mq(c));
    }
}

fn solide(n: &Niveau, tx: i32, ty: i32) -> bool {
    n.tuile_decor(tx, ty).est_solide()
}

/// Une grille peut-elle être percée dans le plafond, centrée en `xc` (px) ? Le
/// plafond est la rangée 0 (la rue est juste au-dessus) et les trois rangées du
/// dessous sont libres (on voit tomber la pluie).
fn grille_possible(n: &Niveau, xc: i32) -> bool {
    let (g, d) = (xc - LARGEUR_GRILLE / 2, xc + LARGEUR_GRILLE / 2 - 1);
    if g < 0 || d >= n.largeur_px() as i32 {
        return false;
    }
    (g.div_euclid(T)..=d.div_euclid(T))
        .all(|tx| solide(n, tx, 0) && (1..=3).all(|ty| !solide(n, tx, ty)))
}

/// Milieu de Pépin au départ (px).
fn milieu_depart(n: &Niveau) -> i32 {
    (n.depart.0 + JOUEUR_L / 2.0).round() as i32
}

/// La grille du départ (milieu, px) : la bouche d'égout par laquelle Pépin est
/// tombé, au plafond juste au-dessus de lui, un peu sur sa droite (les compteurs
/// du coin haut-gauche de l'écran ne la cachent pas) ; le rai du lampadaire
/// descend en biais jusqu'à lui. `None` sans plafond percé possible au départ.
pub(crate) fn grille_du_depart(n: &Niveau) -> Option<i32> {
    let pepin = milieu_depart(n);
    DECALAGES_GRILLE_DEPART
        .iter()
        .map(|d| pepin + d)
        .find(|&x| grille_possible(n, x))
}

/// Les grilles du plafond : abscisses (px) de leur milieu. La première est celle du
/// départ (`grille_du_depart`) ; les autres, de gauche à droite, à `ECART_GRILLES`
/// cases au moins l'une de l'autre, jamais au-dessus d'un bassin (ses tuyaux y
/// sont) ni d'une lanterne, ni dans le grand tuyau de la sortie.
pub(crate) fn grilles(n: &Niveau, entites: &[Entite]) -> Vec<i32> {
    let mut liste: Vec<i32> = grille_du_depart(n).into_iter().collect();
    let debut_tuyau = n.largeur() as i32 - DEBUT_TUYAU;
    let gene = |tx: i32| {
        entites.iter().any(|e| {
            let b = e.boite();
            let (g, d) = (
                (b.x / TAILLE_TUILE) as i32,
                (b.droite() / TAILLE_TUILE) as i32,
            );
            match e {
                Entite::Maree(_) => tx + 1 >= g - 1 && tx <= d + 1,
                Entite::Lanterne(_) => tx + 1 >= g - 1 && tx <= g + 1,
                _ => false,
            }
        })
    };
    let mut derniere = liste.first().map_or(i32::MIN / 2, |&x| x.div_euclid(T));
    for tx in 0..debut_tuyau - 1 {
        let xc = (tx + 1) * T;
        if tx - derniere >= ECART_GRILLES && grille_possible(n, xc) && !gene(tx) {
            liste.push(xc);
            derniere = tx;
        }
    }
    liste
}

/// Ce qui arrête une goutte qui tombe en `x` (px) depuis `y` : le premier solide ou
/// la première eau du niveau, ou la surface du moment d'un bassin. Ordonnée (px).
fn fond(monde: &Monde, x: i32, y: i32) -> i32 {
    let n = &monde.niveau;
    let tx = x.div_euclid(T);
    let mut bas = n.hauteur_px() as i32;
    for ty in y.div_euclid(T).max(0)..n.hauteur() as i32 {
        let t = n.tuile_decor(tx, ty);
        if t.est_solide() || t == Tuile::Eau {
            bas = ty * T;
            break;
        }
    }
    for e in &monde.entites {
        if let Entite::Maree(m) = e {
            let (g, d) = (m.zone.x as i32, m.zone.droite() as i32);
            if (g..d).contains(&x) {
                if let Some(eau) = m.eau() {
                    bas = bas.min(eau.zone.y.round() as i32);
                }
            }
        }
    }
    bas
}

/// Un tuyau de bassin : abscisse (px) de son milieu, bas du plafond d'où il sort,
/// bas de sa bouche (d'où tombe le jet).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Tuyau {
    pub x: i32,
    pub plafond: i32,
    pub bouche: i32,
}

/// Les tuyaux au-dessus d'un bassin : un par `CASES_PAR_TUYAU` cases de large (au
/// moins un), répartis ; chacun sort du plafond au-dessus du bassin et descend
/// d'autant que la place le permet (jamais à moins de 56 px de l'eau pleine : on
/// garde la place d'un saut et de la barque).
pub(crate) fn tuyaux(n: &Niveau, m: &Maree) -> Vec<Tuyau> {
    let z = m.zone;
    let (x0, w) = (z.x as i32, z.w as i32);
    let nb = (w / T / CASES_PAR_TUYAU).max(1);
    (0..nb)
        .map(|k| {
            let x = x0 + w * (2 * k + 1) / (2 * nb);
            let x = x - x.rem_euclid(2);
            let tx = x.div_euclid(T);
            let mut ty = (z.y as i32).div_euclid(T) - 1;
            while ty >= 0 && !solide(n, tx, ty) {
                ty -= 1;
            }
            let plafond = (ty + 1) * T;
            let longueur = (z.y as i32 - plafond - 56).clamp(0, 24);
            Tuyau {
                x,
                plafond,
                bouche: plafond + longueur + BOUCHE,
            }
        })
        .collect()
}

/// Ce que crache un tuyau de bassin en ce moment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Jet {
    /// Rien (basse mer, descente, pleine mer installée).
    Rien,
    /// Il gargouille : des gouttes (l'annonce de la montée).
    Gouttes,
    /// Il se vide : un jet de cette largeur (px), par à-coups pendant la montée,
    /// puis un dernier filet au début de la pleine mer.
    Plein(i32),
}

/// Le jet du tuyau au pas `pas` du cycle de la marée (pure).
pub(crate) fn jet_a(pas: u32) -> Jet {
    let t = pas % PERIODE_MAREE;
    let montee = BASSE_MER_PAS;
    if t < montee {
        return if t >= montee - ANNONCE_MAREE_PAS {
            Jet::Gouttes
        } else {
            Jet::Rien
        };
    }
    let e = t - montee;
    if e < MONTEE_PAS {
        // Un à-coup : le jet enfle, coule plein, faiblit, puis presque rien.
        return Jet::Plein(match e % A_COUP_PAS {
            0..=3 => 4,
            4..=17 => 6,
            18..=23 => 3,
            _ => 1,
        });
    }
    if e < MONTEE_PAS + 20 {
        Jet::Plein(1)
    } else {
        Jet::Rien
    }
}

/// Le grand tuyau de la sortie : abscisses (px) de son début et de sa lèvre, haut et
/// bas de l'intérieur (le plafond et le quai). `None` si le drapeau n'est pas au
/// bout du niveau, contre le bord droit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GrandTuyau {
    pub debut: i32,
    pub levre: i32,
    pub haut: i32,
    pub bas: i32,
}

pub(crate) fn grand_tuyau(n: &Niveau) -> Option<GrandTuyau> {
    let a = n.arrivee?;
    let l = n.largeur() as i32;
    let (tx, ty) = (
        (a.x / TAILLE_TUILE) as i32,
        (a.bas() / TAILLE_TUILE) as i32 - 1,
    );
    if tx < l - 10 {
        return None;
    }
    let colonne = l - 2;
    let mut haut = ty;
    while haut > 0 && !solide(n, colonne, haut - 1) {
        haut -= 1;
    }
    let mut bas = ty;
    while bas < n.hauteur() as i32 - 1 && !solide(n, colonne, bas + 1) {
        bas += 1;
    }
    Some(GrandTuyau {
        debut: (l - DEBUT_TUYAU) * T,
        levre: (l - BOUCHE_TUYAU) * T,
        haut: haut * T,
        bas: (bas + 1) * T,
    })
}

/// Dessine l'habillage des égouts (après les tuiles, avant les objets et Pépin).
pub fn dessiner(monde: &Monde, cx: i32, cy: i32) {
    if monde.infos.theme != Theme::Egouts {
        return;
    }
    let pas = monde.pas;
    if let Some(g) = grand_tuyau(&monde.niveau) {
        dessiner_grand_tuyau(&g, monde.niveau.largeur_px() as i32, pas, (cx, cy));
    }
    let depart = grille_du_depart(&monde.niveau);
    for (k, xc) in grilles(&monde.niveau, &monde.entites)
        .into_iter()
        .enumerate()
    {
        if Some(xc) == depart {
            let pepin = milieu_depart(&monde.niveau);
            let bas = fond(monde, pepin, T);
            dessiner_grille(xc, OR_PALE, ORANGE_SOLEIL, (cx, cy));
            dessiner_rai((xc, pepin), bas, pas, (cx, cy));
        } else {
            let bas = fond(monde, xc, T);
            dessiner_grille(xc, ARDOISE_PALE, ARDOISE_CLAIRE, (cx, cy));
            dessiner_filet(xc, bas, pas + k as u32 * 17, (cx, cy));
        }
    }
    for (k, e) in monde.entites.iter().enumerate() {
        let Entite::Maree(m) = e else { continue };
        for (i, t) in tuyaux(&monde.niveau, m).into_iter().enumerate() {
            let decale = (k * 7 + i * 11) as u32;
            dessiner_jet(monde, &t, jet_a(m.pas()), pas + decale, (cx, cy));
            dessiner_tuyau(&t, (cx, cy));
        }
    }
}

/// Une grille au plafond (rangée 0), centrée en `xc` : son cadre, ses barreaux, et
/// entre eux la lumière de la rue (`clair` en haut, `bas` sur les 3 dernières rangées).
fn dessiner_grille(xc: i32, clair: Couleur, bas: Couleur, (cx, cy): (i32, i32)) {
    let (x, y) = (xc - LARGEUR_GRILLE / 2 - cx, -cy);
    rect(x - 1, y, LARGEUR_GRILLE + 2, T, ARDOISE_OMBRE);
    rect(x + 1, y, LARGEUR_GRILLE - 2, 10, clair);
    rect(x + 1, y + 10, LARGEUR_GRILLE - 2, 3, bas);
    for b in 0..8 {
        rect(x + 1 + b * 4, y, 2, 13, ARDOISE_NUIT);
    }
    // La traverse et le cadre de fonte, sa lèvre éclairée par en dessous.
    rect(x + 1, y + 5, LARGEUR_GRILLE - 2, 1, ARDOISE_NUIT);
    rect(x - 1, y + 13, LARGEUR_GRILLE + 2, 2, ARDOISE);
    rect(x - 1, y + 15, LARGEUR_GRILLE + 2, 1, ARDOISE_PALE);
}

/// Le rai du lampadaire sous la grille du départ : une bande oblique de pixels
/// orange clairsemés (plus serrés en haut), qui descend de la grille (milieu `xc`)
/// en s'élargissant jusqu'au quai, là où Pépin est tombé (milieu `cible`), et la
/// pluie qui tombe dedans.
fn dessiner_rai((xc, cible): (i32, i32), bas: i32, pas: u32, (cx, cy): (i32, i32)) {
    let haut = T;
    let largeur = LARGEUR_GRILLE / 2 - 1;
    let hauteur = (bas - haut).max(1);
    let bornes = |y: i32| {
        let d = y - haut;
        let g = xc - largeur + (cible - 22 - (xc - largeur)) * d / hauteur;
        let dr = xc + largeur + (cible + 26 - (xc + largeur)) * d / hauteur;
        (g, dr)
    };
    for y in (haut + 1..bas - 1).step_by(2) {
        let (g, d) = bornes(y);
        let rang = (y - haut) / 2;
        let pas_motif = if y - haut < 24 { 3 } else { 5 };
        for x in g..d {
            if (x + rang * 2).rem_euclid(pas_motif) == 0 {
                let c = if y - haut < 24 {
                    OR_PALE
                } else {
                    ORANGE_SOLEIL
                };
                rect(x - cx, y - cy, 1, 1, c);
            }
        }
    }
    // La flaque de lumière sur le quai : un pixel sur deux de la margelle.
    let (g, d) = bornes(bas);
    for x in (g..d).step_by(2) {
        rect(x - cx, bas + 1 - cy, 1, 1, OR_PALE);
    }
    // La pluie qui tombe par la grille, et ses éclaboussures.
    let chute = (bas - haut - 4).max(1);
    for i in 0..12i32 {
        let y = haut + ((pas as i32) * 4 + i * 23).rem_euclid(chute);
        let (g, d) = bornes(y);
        let x = g + (i * 7).rem_euclid((d - g).max(1));
        rect(x - cx, y - cy, 1, 4, PLUIE);
        if y + 4 >= bas - 3 {
            rect(x - 1 - cx, bas - 2 - cy, 1, 1, BLANC_CASSE);
            rect(x + 1 - cx, bas - 3 - cy, 1, 1, BLANC_CASSE);
        }
    }
}

/// Le filet de pluie sous une grille : un mince ruisseau haché qui descend jusqu'à
/// ce qui l'arrête (quai ou eau), quelques gouttes à côté, l'éclaboussure en bas.
fn dessiner_filet(xc: i32, bas: i32, pas: u32, (cx, cy): (i32, i32)) {
    let haut = T;
    let x = xc - 1 - cx;
    for y in haut..bas {
        if ((y - pas as i32 * 3).div_euclid(6)).rem_euclid(3) != 0 {
            rect(x, y - cy, 1, 1, PLUIE);
            rect(x + 1, y - cy, 1, 1, PLUIE_SOMBRE);
        }
    }
    let chute = (bas - haut - 3).max(1);
    for i in 0..4i32 {
        let y = haut + ((pas as i32) * 3 + i * 31).rem_euclid(chute);
        let dx = [-9, 6, -4, 10][i as usize];
        rect(x + dx, y - cy, 1, 3, PLUIE);
    }
    let e = (pas / 4 % 2) as i32;
    rect(x - 2 - e, bas - 2 - cy, 1, 1, BLANC_CASSE);
    rect(x + 3 + e, bas - 2 - cy, 1, 1, BLANC_CASSE);
    rect(x - 1, bas - 1 - cy, 4, 1, BLANC_CASSE);
}

/// Un gros tuyau de bassin : sa bride au plafond, son corps de fonte (reflet à
/// gauche, ombre à droite, un collier), sa bouche évasée ouverte par en dessous. Pas
/// de dessus clair : rien n'y ressemble à une plate-forme.
fn dessiner_tuyau(t: &Tuyau, (cx, cy): (i32, i32)) {
    let x0 = t.x - DIAMETRE_TUYAU / 2 - cx;
    let (haut, bouche) = (t.plafond - cy, t.bouche - BOUCHE - cy);
    // Le corps, qui entre dans le plafond.
    let corps = bouche - (haut - 4);
    rect(x0 - 1, haut - 4, DIAMETRE_TUYAU + 2, corps, ARDOISE_OMBRE);
    rect(x0, haut - 4, DIAMETRE_TUYAU, corps, ARDOISE);
    rect(x0 + 2, haut - 4, 2, corps, ARDOISE_CLAIRE);
    rect(x0 + 9, haut - 4, 3, corps, ARDOISE_NUIT);
    // La bride contre le plafond, ses rivets.
    rect(x0 - 3, haut, DIAMETRE_TUYAU + 6, 3, ARDOISE_CLAIRE);
    rect(x0 - 3, haut + 2, DIAMETRE_TUYAU + 6, 1, ARDOISE_NUIT);
    rect(x0 - 2, haut + 1, 1, 1, ARDOISE_OMBRE);
    rect(x0 + DIAMETRE_TUYAU + 1, haut + 1, 1, 1, ARDOISE_OMBRE);
    // Un collier au milieu d'un long tuyau.
    if bouche - haut >= 16 {
        let y = haut + (bouche - haut) / 2;
        rect(x0 - 1, y, DIAMETRE_TUYAU + 2, 2, ARDOISE_CLAIRE);
        rect(x0 - 1, y + 2, DIAMETRE_TUYAU + 2, 1, ARDOISE_OMBRE);
    }
    // La bouche évasée : la lèvre claire, l'ouverture noire au milieu.
    rect(x0 - 3, bouche, DIAMETRE_TUYAU + 6, BOUCHE, ARDOISE_OMBRE);
    rect(x0 - 2, bouche, DIAMETRE_TUYAU + 4, BOUCHE - 1, ARDOISE);
    rect(x0 - 2, bouche, 2, BOUCHE - 1, ARDOISE_CLAIRE);
    rect(
        x0 - 2,
        bouche + BOUCHE - 1,
        DIAMETRE_TUYAU + 4,
        1,
        ARDOISE_PALE,
    );
    rect(
        x0 + 1,
        bouche + BOUCHE - 2,
        DIAMETRE_TUYAU - 2,
        2,
        ARDOISE_OMBRE,
    );
}

/// Ce qui sort d'un tuyau de bassin : des gouttes (il gargouille), ou le jet d'eau
/// qui tombe jusqu'à la surface (ou au fond, à sec), son écume en bas.
fn dessiner_jet(monde: &Monde, t: &Tuyau, jet: Jet, pas: u32, (cx, cy): (i32, i32)) {
    let bas = fond(monde, t.x, t.bouche);
    let haut = t.bouche;
    match jet {
        Jet::Rien => {}
        Jet::Gouttes => {
            // Une goutte qui pend à la lèvre, une autre qui tombe.
            rect(t.x - 1 - cx, haut - cy, 2, 1 + (pas / 6 % 2) as i32, PLUIE);
            let chute = (bas - haut).max(1);
            let y = haut + ((pas as i32) * 3).rem_euclid(chute);
            rect(t.x - cx, y - cy, 1, 3, PLUIE);
        }
        Jet::Plein(largeur) => {
            let x = t.x - largeur / 2 - cx;
            rect(x, haut - cy, largeur, bas - haut, PLUIE);
            if largeur >= 3 {
                rect(x, haut - cy, 1, bas - haut, PLUIE_SOMBRE);
                rect(x + largeur - 1, haut - cy, 1, bas - haut, PLUIE_SOMBRE);
                // Les traits clairs qui descendent dans le jet.
                let mut y = haut + (pas as i32 * 4).rem_euclid(9);
                while y < bas - 3 {
                    rect(
                        x + 1 + (y / 9).rem_euclid(largeur - 2),
                        y - cy,
                        1,
                        3,
                        BLANC_CASSE,
                    );
                    y += 9;
                }
            }
            // L'écume où le jet frappe : une couronne qui bat.
            let e = (pas / 3 % 2) as i32;
            let l = largeur + 4 + 2 * e;
            rect(t.x - l / 2 - cx, bas - 2 - cy, l, 2, BLANC_CASSE);
            if largeur >= 3 {
                rect(t.x - l / 2 - 2 - cx, bas - 4 - e - cy, 1, 2, BLANC_CASSE);
                rect(t.x + l / 2 + 1 - cx, bas - 4 - e - cy, 1, 2, BLANC_CASSE);
            }
        }
    }
}

/// Le grand tuyau de la sortie : l'intérieur (un cylindre de béton vu du dedans,
/// les joints de ses anneaux), sa voûte, son entrée, sa bouche ronde (un anneau
/// en ellipse, éclairé du dehors), et dehors le ciel de nuit, la falaise d'en face
/// et sa chute d'eau, le rebord de rocher où débouche le tuyau.
fn dessiner_grand_tuyau(g: &GrandTuyau, fin: i32, pas: u32, (cx, cy): (i32, i32)) {
    let (debut, levre, haut, bas) = (g.debut - cx, g.levre - cx, g.haut - cy, g.bas - cy);
    let h = bas - haut;
    let w = levre - debut;
    // La paroi du fond : un cylindre vu de l'intérieur, sombre en haut et en bas,
    // un reflet le long de sa courbe au tiers de la hauteur.
    rect(debut, haut, w, h, ARDOISE_NUIT);
    rect(debut, haut, w, 5, ARDOISE_OMBRE);
    rect(debut, haut + h / 3, w, 2, ARDOISE);
    rect(debut, haut + h / 3 + 2, w, 1, ARDOISE_OMBRE);
    rect(debut, bas - 7, w, 7, ARDOISE_OMBRE);
    // Les joints des anneaux, tous les 32 px : un trait clair qui se courbe en haut
    // et en bas (le cercle de l'anneau vu de biais).
    let mut x = debut + 16;
    while x < levre - 12 {
        rect(x, haut + 7, 1, h - 16, ARDOISE);
        rect(x + 1, haut + 7, 1, h - 16, ARDOISE_OMBRE);
        rect(x + 1, haut + 5, 1, 2, ARDOISE);
        rect(x + 2, haut + 3, 1, 2, ARDOISE);
        rect(x + 1, bas - 9, 1, 2, ARDOISE);
        rect(x + 2, bas - 7, 1, 2, ARDOISE);
        x += 32;
    }
    // La voûte du tuyau, dans les tuiles du plafond : un anneau de béton.
    rect(debut, haut - 10, w, 10, ARDOISE_CLAIRE);
    rect(debut, haut - 11, w, 1, ARDOISE_OMBRE);
    rect(debut, haut - 3, w, 3, ARDOISE);
    let mut x = debut + 16;
    while x < levre {
        rect(x, haut - 10, 1, 7, ARDOISE);
        x += 32;
    }
    // L'entrée du tuyau (l'anneau vu par la tranche), sombre : un seuil, pas un mur.
    rect(debut - 3, haut - 11, 5, h + 11, ARDOISE);
    rect(debut - 3, haut - 11, 1, h + 11, ARDOISE_OMBRE);
    rect(debut, haut - 11, 1, h + 11, ARDOISE_CLAIRE);
    // Dehors (à droite de la bouche) : le ciel de nuit de la cascade, la falaise
    // d'en face et sa chute, le rebord où l'on débouche.
    let dehors = levre;
    let l = fin - cx - dehors;
    dessiner_dehors((dehors, l), (haut, bas), pas, cy);
    // La bouche ronde : un anneau en ellipse (demi-largeur `A`, demi-hauteur `b`),
    // centré sur la lèvre. Sa moitié gauche est l'épaisseur du béton vue du dedans
    // (sombre), sa moitié droite la tranche éclairée par le ciel. Rien sous le quai.
    const A: i32 = 7;
    let b = h / 2 + 9;
    let yc = haut + h / 2 - 4;
    for y in (yc - b)..bas {
        let dy = (y - yc) as f32 / b as f32;
        if dy.abs() > 1.0 {
            continue;
        }
        let dx = (A as f32 * (1.0 - dy * dy).sqrt()).round() as i32;
        // La moitié gauche : le béton du dedans, d'épaisseur 3.
        rect(levre - dx - 3, y, 3, 1, ARDOISE);
        rect(levre - dx - 3, y, 1, 1, ARDOISE_OMBRE);
        // Entre les deux arcs : l'épaisseur de la lèvre, vue de face.
        rect(levre - dx, y, 2 * dx, 1, ARDOISE_CLAIRE);
        // La moitié droite : la tranche éclairée.
        rect(levre + dx, y, 2, 1, ARDOISE_PALE);
        rect(levre + dx + 2, y, 1, 1, ARDOISE_OMBRE);
    }
    // L'ouverture : le ciel vu à travers l'anneau (une ellipse plus petite).
    let (a2, b2) = (A - 3, b - 4);
    for y in (yc - b2)..bas {
        let dy = (y - yc) as f32 / b2 as f32;
        if dy.abs() > 1.0 {
            continue;
        }
        let dx = (a2 as f32 * (1.0 - dy * dy).sqrt()).round() as i32;
        let c = couleur_du_ciel(y, haut, h);
        rect(levre - dx, y, 2 * dx, 1, c);
    }
}

/// La couleur du ciel de nuit, dehors, à la hauteur `y` (écran) : nuit en haut,
/// violet d'orage au milieu, brume sur l'horizon (le ciel de la cascade).
fn couleur_du_ciel(y: i32, haut: i32, h: i32) -> Couleur {
    if y < haut + h * 2 / 5 {
        ARDOISE_NUIT
    } else if y < haut + h * 7 / 10 {
        VIOLET_ORAGE
    } else {
        BRUME
    }
}

/// Dehors, de `x0` sur `l` px : le ciel (bandes), les étoiles, la falaise d'en face
/// plus bas que le quai (on la domine : rien à atteindre), sa chute d'eau, la pluie,
/// et le rebord de rocher sous la bouche (le dessus de la falaise où l'on est).
fn dessiner_dehors((x0, l): (i32, i32), (haut, bas): (i32, i32), pas: u32, cy: i32) {
    if l <= 0 {
        return;
    }
    let h = bas - haut;
    for y in -cy..bas {
        rect(x0, y, l, 1, couleur_du_ciel(y, haut, h));
    }
    // Les marches entre deux bandes : un pixel sur deux, sans demi-teinte.
    for k in (0..l).step_by(2) {
        rect(x0 + k, haut + h * 2 / 5 - 1, 1, 1, VIOLET_ORAGE);
        rect(x0 + k, haut + h * 7 / 10 - 1, 1, 1, BRUME);
    }
    for (sx, sy) in [(12, 6), (25, 20), (41, 3), (57, 13), (70, 25)] {
        if sx < l {
            rect(x0 + sx, haut - T + sy, 1, 1, OR_PALE);
        }
    }
    // La falaise d'en face, au loin : une silhouette sombre sur la brume, sa crête
    // déchiquetée (aucun dessus plat : rien à atteindre), l'arête à peine éclairée.
    let crete =
        |k: i32| bas - 24 + [0, 2, 1, 4, 3, 5, 2, 0, 3, 6, 4, 1][(k / 4).rem_euclid(12) as usize];
    for k in 0..l {
        let y = crete(k);
        rect(x0 + k, y, 1, bas - y, ARDOISE_NUIT);
        rect(x0 + k, y, 1, 1, ARDOISE);
    }
    // La chute d'eau de la falaise : une large nappe qui tombe de la brèche de la
    // crête, l'écume en haut, des éclats qui dévalent (la grande cascade).
    let xc = x0 + l / 2;
    let y0 = crete(xc - x0) - 1;
    rect(xc - 3, y0, 8, bas - y0, PLUIE_SOMBRE);
    rect(xc - 2, y0, 6, bas - y0, PLUIE);
    rect(xc - 4, y0 - 1, 10, 2, BLANC_CASSE);
    for k in 0..2 {
        let y = y0 + 2 + (pas as i32 + k * 3).rem_euclid((bas - y0 - 2).max(1));
        rect(xc - 1 + 3 * k, y, 1, 2, BLANC_CASSE);
    }
    // La pluie, dehors.
    for i in 0..7i32 {
        let y = -cy + ((pas as i32) * 5 + i * 37).rem_euclid((bas + cy).max(1));
        rect(x0 + 4 + (i * 13).rem_euclid((l - 4).max(1)), y, 1, 3, PLUIE);
    }
    // Le rebord : le rocher sous la bouche, son dessus clair (on y marche), un
    // bord qui casse vers le vide à droite.
    rect(x0, bas, l, 3, ARDOISE_PALE);
    rect(x0, bas + 3, l, 2, ARDOISE_CLAIRE);
    rect(x0, bas + 5, l, 60, ARDOISE);
    for (fx, fy) in [(9, 9), (22, 14), (37, 8), (51, 17), (66, 11)] {
        if fx < l {
            rect(x0 + fx, bas + fy, 5, 1, ARDOISE_NUIT);
            rect(x0 + fx + 4, bas + fy + 1, 1, 3, ARDOISE_NUIT);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pepin_core::entities::maree::PLEINE_MER_PAS;
    use pepin_core::levels::egouts;

    fn niveau() -> Niveau {
        Niveau::depuis_ascii(egouts::INFOS.ascii).expect("égouts")
    }

    /// La première grille est au-dessus du départ (à trois cases au plus de Pépin,
    /// sur sa droite) : on est tombé par là.
    #[test]
    fn la_premiere_grille_est_au_dessus_du_depart() {
        let n = niveau();
        let g = grilles(&n, &n.entites);
        assert_eq!(grille_du_depart(&n), Some(g[0]));
        let ecart = g[0] - milieu_depart(&n);
        assert!((0..=3 * T).contains(&ecart), "{g:?}");
        assert!(g.len() >= 4, "quelques grilles le long du niveau : {g:?}");
    }

    /// Les grilles sont percées dans un plafond (rangée 0 pleine), au-dessus d'un
    /// passage libre, assez espacées, jamais au-dessus d'un bassin ni dans le
    /// grand tuyau.
    #[test]
    fn les_grilles_sont_au_plafond_loin_des_bassins() {
        let n = niveau();
        let g = grilles(&n, &n.entites);
        let tuyau = grand_tuyau(&n).expect("grand tuyau");
        for paire in g.windows(2) {
            assert!(paire[1] - paire[0] >= ECART_GRILLES * T - T, "{g:?}");
        }
        for &x in &g {
            assert!(grille_possible(&n, x), "grille en {x}");
            assert!(
                x + LARGEUR_GRILLE / 2 <= tuyau.debut,
                "grille en {x} dans le tuyau"
            );
            for e in &n.entites {
                if let Entite::Maree(m) = e {
                    let (a, b) = (m.zone.x as i32, m.zone.droite() as i32);
                    assert!(x + LARGEUR_GRILLE / 2 <= a || x - LARGEUR_GRILLE / 2 >= b);
                }
            }
        }
    }

    /// Chaque bassin a au moins un tuyau, au-dessus de lui, sorti du plafond, dont
    /// la bouche laisse au moins 56 px au-dessus de l'eau pleine (sous une voûte
    /// basse, seule la bouche dépasse du plafond).
    #[test]
    fn chaque_bassin_a_ses_tuyaux() {
        let n = niveau();
        let mut bassins = 0;
        for e in &n.entites {
            let Entite::Maree(m) = e else { continue };
            bassins += 1;
            let t = tuyaux(&n, m);
            assert!(!t.is_empty());
            for t in t {
                assert!(m.zone.x as i32 <= t.x && t.x < m.zone.droite() as i32);
                assert!(t.plafond < t.bouche);
                assert!(
                    t.bouche - t.plafond == BOUCHE || m.zone.y as i32 - t.bouche >= 56 - BOUCHE,
                    "{t:?}"
                );
                let (tx, ty) = (t.x.div_euclid(T), t.plafond / T - 1);
                assert!(solide(&n, tx, ty), "le tuyau sort du plafond : {t:?}");
            }
        }
        assert_eq!(bassins, 4, "les quatre bassins des égouts (carte 9.21a)");
    }

    /// Le tuyau gargouille pendant l'annonce, se vide par à-coups pendant toute la
    /// montée (le jet enfle et faiblit), et se tait à la descente.
    #[test]
    fn le_tuyau_se_vide_quand_l_eau_monte() {
        assert_eq!(jet_a(0), Jet::Rien);
        assert_eq!(jet_a(BASSE_MER_PAS - 1), Jet::Gouttes);
        let montee: Vec<Jet> = (BASSE_MER_PAS..BASSE_MER_PAS + MONTEE_PAS)
            .map(jet_a)
            .collect();
        assert!(montee.iter().all(|j| matches!(j, Jet::Plein(_))));
        let pleins = montee.iter().filter(|&&j| j == Jet::Plein(6)).count();
        assert!(pleins > 0 && pleins < montee.len(), "par à-coups");
        let descente = BASSE_MER_PAS + MONTEE_PAS + PLEINE_MER_PAS;
        assert_eq!(jet_a(descente + 1), Jet::Rien);
        assert_eq!(jet_a(PERIODE_MAREE), Jet::Rien);
    }

    /// Le grand tuyau de la sortie entoure le drapeau, va du plafond au quai, et sa
    /// lèvre est au bout du niveau.
    #[test]
    fn la_sortie_est_le_grand_tuyau() {
        let n = niveau();
        let g = grand_tuyau(&n).expect("grand tuyau");
        let a = n.arrivee.expect("drapeau");
        assert!(g.debut < a.x as i32 && (a.droite() as i32) <= g.levre);
        assert_eq!((g.haut, g.bas), (T, 7 * T));
        assert!(g.levre < n.largeur_px() as i32);
        // Le quai continue sous la lèvre : on n'y voit pas de trou.
        let tx = g.levre / T;
        assert!(solide(&n, tx, g.bas / T) && solide(&n, tx + 1, g.bas / T));
    }
}
