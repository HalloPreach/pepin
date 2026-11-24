//! Habillage de « La grande cascade » (carte 9.20c) : le niveau ressemble à son nom
//! et se raccorde aux égouts (avant) et à la rivière (après). Rendu seulement : aucune
//! collision, la mécanique ne change pas.
//!
//! 1. Le DÉPART : le bout du GRAND TUYAU des égouts sort de la falaise. Pépin en sort
//!    sur sa passerelle de béton ; sous la passerelle, l'eau des égouts coule dans le
//!    fond du tuyau et se déverse par la bouche dans le bassin A.
//! 2. La GRANDE CHUTE, en fond, sur toute la hauteur du niveau : un large rideau
//!    d'eau derrière les tuiles, plus sombre et moins saturé que les chutes jouables
//!    `|` (cyan vif, au premier plan : ses bras). Pas de collision, aucun danger.
//!    Écume aux paliers, la falaise du pied de part et d'autre, la brume en bas.
//! 3. Le BASSIN D'ARRIVÉE : la chute y tombe dans un bouillon d'écume, puis l'eau part
//!    en rivière vers la droite (le courant `}` de la grille), sous le drapeau.
//!
//! Tout se place d'après la grille : la passerelle sous le départ, la bouche au bout
//! de la passerelle, le bassin d'arrivée (la plus basse étendue d'eau). La grande
//! chute est dans la pile du fond (`Plan`, `dessiner_plan`, repris par `decor::pile`,
//! carte 10.4f) ; la scène appelle `dessiner` devant les tuiles ; rien hors du thème
//! Cascade.

use macroquad::prelude::draw_rectangle;
use pepin_core::aabb::TAILLE_TUILE;
use pepin_core::entities::Entite;
use pepin_core::level::{Niveau, Tuile};
use pepin_core::palette::{
    Couleur, ARDOISE, ARDOISE_CLAIRE, ARDOISE_NUIT, ARDOISE_OMBRE, ARDOISE_PALE, BLANC_CASSE,
    BRUME, NUAGE, PLUIE, PLUIE_SOMBRE, VIOLET_ORAGE,
};
use pepin_core::physics::JOUEUR_H;
use pepin_core::zones::Theme;
use pepin_core::HAUTEUR_LOGIQUE;

use crate::monde::Monde;
use crate::rendu::couleur_mq;

const T: i32 = TAILLE_TUILE as i32;
const H: i32 = HAUTEUR_LOGIQUE as i32;

/// Le rideau de la grande chute : de la colonne 11 à la colonne 21 (160 px), au
/// milieu du niveau, là où il se voit par toutes les ouvertures (au-dessus du bassin
/// A, la salle des chutes, le couloir, le pied de la falaise).
const RIDEAU_X: i32 = 11 * T;
const RIDEAU_L: i32 = 10 * T;
/// Hauteur (px) de la lèvre de roche d'où l'eau bascule, en haut du niveau.
const LEVRE: i32 = 12;
/// Les paliers du rideau (px, dans le niveau) : l'eau y rebondit dans l'écume. Pris
/// là où le rideau se voit (carte 9.21b, le niveau allongé à 94 rangées). Carte
/// 9.26c : chacun à la surface d'un bassin (B, E et la mare aux nénuphars), pour
/// que le rideau tombe dans l'eau et non sur une corniche.
const PALIERS: [i32; 3] = [18 * T, 46 * T, 82 * T];
/// Le pied de la falaise : la roche du fond borde le rideau sur les douze dernières
/// rangées au-dessus du bassin d'arrivée (elle suit la hauteur du niveau).
const PIED_FALAISE: i32 = 12 * T;
/// Vitesse de l'eau du rideau (px par pas) : plus vite que la pluie.
const VITESSE_RIDEAU: i32 = 3;

fn rect(x: i32, y: i32, w: i32, h: i32, c: Couleur) {
    if w > 0 && h > 0 {
        draw_rectangle(x as f32, y as f32, w as f32, h as f32, couleur_mq(c));
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

/// Le grand tuyau du départ, en pixels du niveau : la passerelle sous les pieds de
/// Pépin, de la falaise (bord gauche) jusqu'à la bouche.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Tuyau {
    /// Abscisse de la bouche (le bout de la passerelle).
    pub bouche: i32,
    /// Dessus de la passerelle (sous les pieds de Pépin au départ).
    pub passerelle: i32,
}

/// Le tuyau du départ : la passerelle est la rangée sous les pieds de Pépin, la bouche
/// au bout de cette rangée (la première case libre à droite du départ). `None` si le
/// départ n'est pas posé sur un solide.
pub(crate) fn tuyau(n: &Niveau) -> Option<Tuyau> {
    let (x, y) = n.depart;
    let ty = ((y + JOUEUR_H) / TAILLE_TUILE).floor() as i32;
    let mut tx = (x / TAILLE_TUILE).floor() as i32;
    if !n.tuile_decor(tx, ty).est_solide() {
        return None;
    }
    while n.tuile_decor(tx + 1, ty).est_solide() {
        tx += 1;
    }
    Some(Tuyau {
        bouche: (tx + 1) * T,
        passerelle: ty * T,
    })
}

/// La surface du bassin d'arrivée (px) : la plus basse étendue d'eau du niveau.
pub(crate) fn bassin_d_arrivee(n: &Niveau) -> Option<i32> {
    n.eaux.iter().map(|e| e.surface().round() as i32).max()
}

/// Les plans de la grande chute, du plus loin au plus près (carte 10.4f). Tous
/// CALCULÉS : ils se posent d'après la grille du niveau (le bas de la chute est le
/// bassin d'arrivée, la falaise court sur toute la largeur du niveau, les filets
/// suivent les chutes `|`) ou s'animent (l'eau qui dévale, la brume). `decor::pile`
/// les reprend tels quels pour le thème `Cascade`, après le fond générique.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    /// La falaise du pied, de part et d'autre de la chute.
    Falaise,
    /// Le rideau : la nappe, ses filets qui dévalent, la lèvre, l'écume des paliers.
    Rideau,
    /// La brume du bas de la chute.
    Brume,
    /// Les filets des chutes `|` qui continuent derrière les tuiles jusqu'à l'eau.
    Filets,
}

/// La pile de la grande chute (lue par `decor::pile`).
pub const PILE: [Plan; 4] = [Plan::Falaise, Plan::Rideau, Plan::Brume, Plan::Filets];

impl Plan {
    /// Le nom du plan dans `assets/fonds/LISEZMOI.md` (aucun n'est un fichier).
    #[cfg(test)]
    pub fn nom(self) -> &'static str {
        match self {
            Plan::Falaise => "falaise du pied",
            Plan::Rideau => "rideau de la chute",
            Plan::Brume => "brume de la chute",
            Plan::Filets => "filets des chutes",
        }
    }
}

/// Dessine un plan de la grande chute, derrière les tuiles (rien hors du thème
/// Cascade). `(cx, cy)` : la caméra de la scène.
pub fn dessiner_plan(plan: Plan, monde: &Monde, cx: i32, cy: i32) {
    if monde.infos.theme != Theme::Cascade {
        return;
    }
    let n = &monde.niveau;
    let pas = monde.pas;
    let bas = bassin_d_arrivee(n).unwrap_or(n.hauteur_px() as i32);
    match plan {
        Plan::Falaise => dessiner_falaise_du_pied(bas, n.largeur_px() as i32, (cx, cy)),
        Plan::Rideau => dessiner_rideau(bas, pas, (cx, cy)),
        Plan::Brume => dessiner_brume(bas, pas, (cx, cy)),
        Plan::Filets => {
            for p in pieds_des_chutes(monde) {
                if p.eau > p.haut {
                    dessiner_filet(&p, pas, (cx, cy));
                }
            }
        }
    }
}

/// Le pied d'une chute `|` (px du niveau), carte 9.26c : chaque chute finit dans
/// l'eau. `haut` est le bas de la chute jouable, `eau` la surface de l'eau où elle
/// retombe (sous elle, au plus près). `eau == haut` : elle tombe droit dans un
/// bassin ; plus bas, son eau continue en filet de décor, DERRIÈRE les tuiles (le
/// puits, au-dessus du palier de la salle des bras).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Pied {
    pub g: i32,
    pub d: i32,
    pub haut: i32,
    pub eau: i32,
}

/// Les pieds des chutes du niveau ; une chute sans eau dessous n'en a pas.
pub(crate) fn pieds_des_chutes(monde: &Monde) -> Vec<Pied> {
    let n = &monde.niveau;
    monde
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Chute(c) => Some(c.zone),
            _ => None,
        })
        .filter_map(|z| {
            let (g, d, haut) = (z.x as i32, z.droite() as i32, z.bas() as i32);
            let tx = ((g + d) / 2).div_euclid(T);
            let ty =
                (haut / T..n.hauteur() as i32).find(|&ty| n.tuile_decor(tx, ty) == Tuile::Eau)?;
            Some(Pied {
                g,
                d,
                haut,
                eau: ty * T,
            })
        })
        .collect()
}

/// Le filet : l'eau d'une chute qui débouche au-dessus d'un vide continue de
/// tomber jusqu'à l'eau du dessous, plus étroite et plus sombre que la chute
/// jouable (elle ne pousse pas), ses traits clairs qui dévalent. Derrière les
/// tuiles : elle passe derrière les corniches.
fn dessiner_filet(p: &Pied, pas: u32, (cx, cy): (i32, i32)) {
    let (y0, y1) = (p.haut.max(cy), p.eau.min(cy + H));
    if y0 >= y1 {
        return;
    }
    let (g, l) = (p.g + 4, p.d - p.g - 8);
    rect(g - cx, y0 - cy, l, y1 - y0, PLUIE_SOMBRE);
    rect(g - cx, y0 - cy, 1, y1 - y0, ARDOISE_NUIT);
    rect(g + l - 1 - cx, y0 - cy, 1, y1 - y0, ARDOISE_NUIT);
    let defile = pas as i32 * VITESSE_RIDEAU;
    for k in 0..(l - 2) / 2 {
        let x = g + 1 + k * 2 + (k % 2);
        let phase = (hache(k + 900 + p.g) % 32) as i32;
        let premier = (y0 - phase - defile - 8).div_euclid(32);
        let dernier = (y1 - phase - defile).div_euclid(32);
        for i in premier..=dernier {
            let ya = (p.haut.max(i * 32 + phase + defile)).max(y0);
            let yb = (i * 32 + phase + defile + 8).min(y1);
            rect(x - cx, ya - cy, 1, yb - ya, PLUIE);
        }
    }
}

/// Au pied de chaque chute, sur l'eau où elle tombe (carte 9.26c) : des
/// éclaboussures blanches et cyan qui sautent, et, au pied d'un filet (qui n'a pas
/// l'écume dessinée des chutes jouables), un liseré d'écume qui bout.
fn dessiner_eclaboussures(p: &Pied, pas: u32, (cx, cy): (i32, i32)) {
    if p.eau + 4 < cy || p.eau - 16 > cy + H {
        return;
    }
    let (g, d) = if p.eau > p.haut {
        (p.g + 2, p.d - 2)
    } else {
        (p.g - 2, p.d + 2)
    };
    if p.eau > p.haut {
        let mut x = g;
        let mut k = 0;
        while x < d {
            let bout = (hache(k * 3 + p.g + (pas / 4) as i32) % 3) as i32;
            let c = if (k + (pas / 8) as i32) % 3 == 0 {
                BLANC_CASSE
            } else {
                BRUME
            };
            rect(x - cx, p.eau - 1 - bout - cy, 2, 1 + bout, c);
            x += 2;
            k += 1;
        }
    }
    let largeur = (d - g).max(1) as u32;
    for i in 0..5i32 {
        let cycle = (pas as i32 + i * 7 + p.g).rem_euclid(20);
        let hauteur = cycle * (20 - cycle) / 10;
        let x = g + (hache(i + 1000 + p.g + (pas as i32 + i * 7 + p.g) / 20) % largeur) as i32;
        let c = if i % 2 == 0 { BLANC_CASSE } else { PLUIE };
        rect(x - cx, p.eau - 2 - hauteur - cy, 1, 2, c);
    }
}

/// Devant les tuiles : le grand tuyau du départ et le bouillon du bassin d'arrivée.
pub fn dessiner(monde: &Monde, cx: i32, cy: i32) {
    if monde.infos.theme != Theme::Cascade {
        return;
    }
    let n = &monde.niveau;
    let pas = monde.pas;
    if let Some(t) = tuyau(n) {
        dessiner_tuyau(&t, pas, (cx, cy));
    }
    if let Some(surface) = bassin_d_arrivee(n) {
        dessiner_bouillon(surface, pas, (cx, cy));
    }
    for p in pieds_des_chutes(monde) {
        dessiner_eclaboussures(&p, pas, (cx, cy));
    }
}

/// Le bord du rideau à la hauteur `y` (px du niveau) : il frange de 0 à 3 px, et
/// s'élargit un peu sous chaque palier (l'eau s'étale en rebondissant).
fn bords_du_rideau(y: i32) -> (i32, i32) {
    let frange = |k: i32| (hache(y.div_euclid(4) * 7 + k) % 4) as i32;
    let evase = PALIERS.iter().filter(|&&p| y > p).count() as i32 * 4;
    (
        RIDEAU_X - evase - frange(1),
        RIDEAU_X + RIDEAU_L + evase + frange(2),
    )
}

/// Le rideau : la nappe sombre (ardoise), ses filets plus clairs qui dévalent, ses
/// bords frangés, la lèvre de roche en haut et l'écume à chaque palier.
fn dessiner_rideau(bas: i32, pas: u32, (cx, cy): (i32, i32)) {
    let haut = LEVRE;
    let (y0, y1) = (haut.max(cy), bas.min(cy + H));
    // La nappe, rangée par rangée de 4 px (bords frangés).
    let mut y = y0 - y0.rem_euclid(4);
    while y < y1 {
        let (g, d) = bords_du_rideau(y);
        let (ya, yb) = (y.max(y0), (y + 4).min(y1));
        rect(g - cx, ya - cy, d - g, yb - ya, ARDOISE);
        rect(g - cx, ya - cy, 1, yb - ya, ARDOISE_NUIT);
        rect(d - 1 - cx, ya - cy, 1, yb - ya, ARDOISE_NUIT);
        y += 4;
    }
    // Les filets qui dévalent : un tous les 5 px, des traits de 10 à 25 px qui
    // reviennent tous les 48 px, décalés d'un filet à l'autre.
    let defile = pas as i32 * VITESSE_RIDEAU;
    for k in 0..(RIDEAU_L / 5) {
        let x = RIDEAU_X + 2 + k * 5 + (hache(k) % 3) as i32 - 1;
        let longueur = 10 + (hache(k + 100) % 16) as i32;
        let phase = (hache(k + 200) % 48) as i32;
        let couleur = if k % 4 == 1 {
            ARDOISE_PALE
        } else {
            ARDOISE_CLAIRE
        };
        // Les traits visibles : ceux qui chevauchent [y0, y1).
        let premier = (y0 - phase - defile - longueur).div_euclid(48);
        let dernier = (y1 - phase - defile).div_euclid(48);
        for i in premier..=dernier {
            let ya = (i * 48 + phase + defile).max(y0);
            let yb = (i * 48 + phase + defile + longueur).min(y1);
            rect(x - cx, ya - cy, 1, yb - ya, couleur);
        }
    }
    // La lèvre : la roche d'où l'eau bascule (crête déchiquetée, sans dessus plat),
    // et l'eau qui s'y bombe en écume pâle.
    if cy < haut + 8 {
        for k in -2..(RIDEAU_L / 4 + 2) {
            let x = RIDEAU_X + k * 4;
            let creux = (hache(k + 300) % 4) as i32;
            rect(x - cx, -cy, 4, haut - 3 + creux, ARDOISE_NUIT);
            rect(x - cx, haut - 3 + creux - cy, 4, 1, ARDOISE_OMBRE);
        }
        for k in 0..(RIDEAU_L / 3) {
            let x = RIDEAU_X + k * 3;
            let bosse = (hache(k + 400 + (pas / 6) as i32) % 3) as i32;
            rect(x - cx, haut - cy - bosse, 2, 2 + bosse, BRUME);
        }
    }
    // L'écume aux paliers : un nuage d'embruns TRAMÉ (un pixel sur deux, plus
    // clairsemé vers les bords), jamais un trait plein qui se lirait comme un rebord.
    for &p in &PALIERS {
        let (g, d) = bords_du_rideau(p);
        let embruns = Trame {
            centre: ((g + d) / 2, p - 3),
            rayons: ((d - g) / 2 + 10, 12),
            graine: p,
            couleurs: (BRUME, ARDOISE_PALE),
        };
        embruns.dessiner(pas / 10, (cx, cy));
    }
}

/// Un nuage TRAMÉ (embruns, brume) : des pixels isolés en damier dans une ellipse,
/// serrés au cœur, clairsemés vers le bord ; aucun aplat (rien ne s'y lit comme un
/// rebord où se poser). Le tirage change tous les `instant` pour que ça bouillonne.
struct Trame {
    centre: (i32, i32),
    rayons: (i32, i32),
    graine: i32,
    /// Couleur du cœur, couleur du bord.
    couleurs: (Couleur, Couleur),
}

impl Trame {
    fn dessiner(&self, instant: u32, (cx, cy): (i32, i32)) {
        let ((xc, yc), (rx, ry)) = (self.centre, self.rayons);
        if yc + ry < cy || yc - ry > cy + H || rx <= 0 || ry <= 0 {
            return;
        }
        for dy in -ry..=ry {
            for dx in -rx..=rx {
                let (x, y) = (xc + dx, yc + dy);
                if (x + y).rem_euclid(2) != 0 {
                    continue;
                }
                let (u, v) = (dx as f32 / rx as f32, dy as f32 / ry as f32);
                let d = u * u + v * v;
                if d > 1.0 {
                    continue;
                }
                let tirage = hache(x * 31 + y * 17 + self.graine + instant as i32 * 7) % 100;
                if tirage as f32 >= (1.0 - d) * 110.0 {
                    continue;
                }
                let c = if d < 0.3 {
                    self.couleurs.0
                } else {
                    self.couleurs.1
                };
                rect(x - cx, y - cy, 1, 1, c);
            }
        }
    }
}

/// La falaise du pied, de part et d'autre du rideau, sur `PIED_FALAISE` au-dessus du
/// bassin d'arrivée : des pans de roche du fond (ardoise de nuit, fissures en biais),
/// leur crête déchiquetée,
/// jamais de dessus clair ni de trait horizontal (rien à atteindre). À droite, la
/// crête plonge avant le bord : la gorge s'ouvre sur le ciel, là où part la rivière.
fn dessiner_falaise_du_pied(bas: i32, largeur: i32, (cx, cy): (i32, i32)) {
    let pied = bas - PIED_FALAISE;
    if pied > cy + H || bas < cy {
        return;
    }
    let (g, d) = (RIDEAU_X - 10, RIDEAU_X + RIDEAU_L + 10);
    // La crête : plus haute contre le rideau (la falaise d'où il tombe), qui
    // descend vers les bords du niveau, et s'effondre près du bord droit (la
    // brèche de la rivière).
    let breche = largeur - 7 * T;
    let crete = |x: i32| {
        let distance = if x < g { g - x } else { x - d };
        let effondrement = if x > breche { (x - breche) * 2 } else { 0 };
        pied + distance / 3 + effondrement + (hache(x.div_euclid(4) + 500) % 6) as i32
    };
    for x in (0..largeur).step_by(4) {
        if (g..d).contains(&x) {
            continue;
        }
        let y = crete(x);
        if y >= bas {
            continue;
        }
        let ya = y.max(cy);
        rect(x - cx, ya - cy, 4, bas - ya, ARDOISE_NUIT);
        rect(x - cx, y - cy, 4, 1, ARDOISE_OMBRE);
    }
    // Les fissures, en biais (un trait vertical puis un pas de côté), et quelques
    // éclats de roche violette, isolés.
    for k in 0..28 {
        let x = (hache(k + 600) % largeur.max(1) as u32) as i32;
        if (g - 6..d + 6).contains(&x) {
            continue;
        }
        let y = crete(x) + 10 + (hache(k + 700) % 160) as i32;
        if y + 12 >= bas || crete(x + 4) + 6 > y {
            continue;
        }
        let c = if k % 3 == 0 { VIOLET_ORAGE } else { ARDOISE };
        rect(x - cx, y - cy, 1, 6, c);
        rect(x + 1 - cx, y + 6 - cy, 1, 6, c);
    }
}

/// La brume du bas : des nappes tramées (jamais un aplat qui se lirait comme une
/// plate-forme) qui montent du bassin d'arrivée et dérivent.
fn dessiner_brume(bas: i32, pas: u32, (cx, cy): (i32, i32)) {
    if bas - 64 > cy + H || bas < cy {
        return;
    }
    for (k, (dy, x0)) in [(10, 150), (26, 250), (44, 190), (14, 330), (58, 290)]
        .into_iter()
        .enumerate()
    {
        let derive = ((pas as i32 / 7 + k as i32 * 13) % 40) - 20;
        let nappe = Trame {
            centre: (x0 + derive, bas - dy),
            rayons: (34, 9),
            graine: k as i32 * 1000,
            couleurs: (BRUME, ARDOISE_PALE),
        };
        nappe.dessiner(pas / 16, (cx, cy));
    }
}

/// Le bouillon : là où le rideau tombe dans le bassin d'arrivée, l'eau blanchit
/// (écume qui bout, éclaboussures qui sautent).
fn dessiner_bouillon(surface: i32, pas: u32, (cx, cy): (i32, i32)) {
    if surface + 8 < cy || surface - 16 > cy + H {
        return;
    }
    let (g, d) = bords_du_rideau(surface - 1);
    let mut x = g - 4;
    let mut k = 0;
    while x < d + 4 {
        let bout = (hache(k * 3 + (pas / 4) as i32) % 4) as i32;
        let c = if (k + (pas / 8) as i32) % 4 == 0 {
            BLANC_CASSE
        } else {
            BRUME
        };
        rect(x - cx, surface - 1 - bout - cy, 2, 2 + bout, c);
        rect(x - cx, surface + 2 + (k % 3) - cy, 2, 1, NUAGE);
        x += 2;
        k += 1;
    }
    // Les éclaboussures : des gouttes qui sautent au-dessus du bouillon.
    for i in 0..8i32 {
        let cycle = (pas as i32 + i * 11).rem_euclid(24);
        let hauteur = cycle * (24 - cycle) / 12;
        let x = g + (hache(i + 800 + (pas as i32 + i * 11) / 24) % (d - g) as u32) as i32;
        rect(x - cx, surface - 2 - hauteur - cy, 1, 2, NUAGE);
    }
}

/// Le grand tuyau : il sort de la falaise (bord gauche du niveau) et s'arrête à sa
/// bouche, au bout de la passerelle. Au-dessus de la passerelle, l'intérieur sombre
/// où se tient Pépin ; en dessous, l'eau des égouts qui coule et se déverse par la
/// bouche dans le bassin du dessous.
fn dessiner_tuyau(t: &Tuyau, pas: u32, (cx, cy): (i32, i32)) {
    let p = t.passerelle;
    // Les cotes du tuyau (px du niveau) : voûte, plafond, passerelle, rigole, fond.
    let voute = p - 4 * T;
    let plafond = p - 3 * T;
    let surface = p + T;
    let fond = p + 2 * T;
    let dessous = fond + 14;
    let (x0, b) = (-cx, t.bouche - cx);
    let w = b - x0;
    let y = |v: i32| v - cy;
    // La voûte : un anneau de béton, ses joints tous les 32 px.
    rect(x0, y(voute), w, plafond - voute, ARDOISE_CLAIRE);
    rect(x0, y(voute), w, 1, ARDOISE_OMBRE);
    rect(x0, y(plafond) - 3, w, 3, ARDOISE);
    // L'intérieur, au-dessus de la passerelle : sombre en haut, un reflet au tiers.
    rect(x0, y(plafond), w, p - plafond, ARDOISE_NUIT);
    rect(x0, y(plafond), w, 4, ARDOISE_OMBRE);
    rect(x0, y(plafond) + (p - plafond) / 3, w, 2, ARDOISE);
    // La passerelle : un dessus clair où l'on marche, la tranche du béton.
    rect(x0, y(p), w, 1, ARDOISE_PALE);
    rect(x0, y(p) + 1, w, 6, ARDOISE_CLAIRE);
    rect(x0, y(p) + 7, w, 1, ARDOISE_OMBRE);
    // La rigole sous la passerelle : l'ombre, puis l'eau des égouts qui file vers
    // la bouche (des rides claires qui avancent).
    rect(x0, y(p) + 8, w, surface - p - 8, ARDOISE_OMBRE);
    rect(x0, y(surface), w, fond - surface, PLUIE_SOMBRE);
    rect(x0, y(surface), w, 1, PLUIE);
    for k in 0..(w / 12 + 1) {
        let x = (k * 12 + pas as i32 / 2).rem_euclid(w.max(1));
        rect(x0 + x, y(surface) + 3 + (k % 3) * 3, 4, 1, PLUIE);
    }
    // Le fond du tuyau.
    rect(x0, y(fond), w, dessous - fond, ARDOISE_CLAIRE);
    rect(x0, y(dessous) - 1, w, 1, ARDOISE_OMBRE);
    // Les joints des anneaux (au-dessus et au-dessous de la passerelle).
    let mut x = b - 24;
    while x > x0 {
        rect(x, y(voute) + 1, 1, plafond - voute - 4, ARDOISE);
        rect(x, y(plafond) + 4, 1, p - plafond - 6, ARDOISE_OMBRE);
        rect(x, y(fond) + 1, 1, dessous - fond - 2, ARDOISE);
        x -= 32;
    }
    // La bouche : un anneau en ellipse, un collet de béton plus large que le tube.
    // Sa moitié gauche est l'épaisseur vue du dedans (sombre), sa moitié droite la
    // tranche éclairée par le ciel.
    const A: i32 = 7;
    let (haut, bas) = (voute - 6, dessous + 4);
    let demi = (bas - haut) / 2;
    let yc = haut + demi;
    for yy in haut..bas {
        let dy = (yy - yc) as f32 / demi as f32;
        let dx = (A as f32 * (1.0 - dy * dy).max(0.0).sqrt()).round() as i32;
        rect(b - dx - 3, y(yy), 3, 1, ARDOISE_OMBRE);
        rect(b - dx, y(yy), 2 * dx, 1, ARDOISE_CLAIRE);
        rect(b + dx, y(yy), 2, 1, ARDOISE_PALE);
        rect(b + dx + 2, y(yy), 1, 1, ARDOISE_OMBRE);
    }
    // La passerelle reste lisible jusqu'à la bouche : son dessus clair passe devant
    // l'anneau.
    rect(b - A - 3, y(p), A + 3, 1, ARDOISE_PALE);
    // Le déversoir : l'eau de la rigole bascule par la bouche et tombe en nappe
    // courbe dans le bassin du dessous.
    let chute = T;
    for dy in 0..(fond - surface + chute) {
        let haut_x = (3.0 * (2.0 * dy as f32).sqrt()).round() as i32;
        let bas_x = if dy > fond - surface {
            (1.5 * (2.0 * (dy - (fond - surface)) as f32).sqrt()).round() as i32
        } else {
            0
        };
        let yy = y(surface) + dy;
        rect(b + bas_x, yy, haut_x - bas_x + 1, 1, PLUIE_SOMBRE);
        rect(b + haut_x, yy, 1, 1, PLUIE);
    }
    for i in 0..3i32 {
        let dy = (pas as i32 * 2 + i * 9).rem_euclid(fond - surface + chute);
        let x = b + (2.0 * (2.0 * dy as f32).sqrt()).round() as i32;
        rect(x, y(surface) + dy, 1, 3, BLANC_CASSE);
    }
    // L'éclaboussure en bas du déversoir, sur l'eau du bassin.
    let pied = surface + chute + (fond - surface);
    for k in 0..6i32 {
        let bond = (hache(k + (pas / 5) as i32) % 3) as i32;
        rect(b + 2 + k * 4, y(pied) - 1 - bond, 3, 1 + bond, BRUME);
    }
}
