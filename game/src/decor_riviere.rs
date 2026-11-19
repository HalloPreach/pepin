//! Habillage de « La rivière » (carte 9.10b, demande d'AGA : « le fond est vide et
//! sans âme ») : la VALLÉE au soleil couchant, après l'orage. Rendu seulement :
//! aucune collision, aucune règle de jeu.
//!
//! Le fond (`dessiner_fond`, derrière les tuiles), du plus loin au plus proche, en
//! parallaxe :
//! 1. le CIEL de fin d'après-midi, en dégradé chaud (ardoise, violet, rose, orange,
//!    or pâle à l'horizon) et le soleil bas, son halo ;
//! 2. les MONTAGNES bleutées, leurs flancs éclairés côté soleil ;
//! 3. les COLLINES, la rivière du fond qui y serpente et où le soleil scintille ;
//! 4. la FORÊT (sapins et feuillus, deux tons) et le vieux MOULIN à roue, sa roue
//!    qui tourne lentement, sa fenêtre allumée ;
//! 5. les GRANDS ARBRES tout près, dont les branches dépassent en haut de l'écran ;
//! 6. la vie : des oiseaux qui passent, des lucioles qui flottent.
//!
//! Tout le fond reste plus terne et plus sombre que le premier plan (l'or vif est
//! réservé aux gouttes à ramasser). Devant les tuiles (`dessiner`) : les reflets sur
//! l'eau jouable, l'écume au pied des rochers et des chutes, des roseaux au bord
//! des berges. Le tronc flottant (la plateforme mobile) : `dessiner_tronc`. Rien
//! hors du thème Rivière.

use macroquad::prelude::draw_rectangle;
use pepin_core::aabb::TAILLE_TUILE;
use pepin_core::entities::Entite;
use pepin_core::level::Tuile;
use pepin_core::palette::{
    Couleur, ARDOISE, ARDOISE_CLAIRE, ARDOISE_NUIT, ARDOISE_OMBRE, ARDOISE_PALE, BLANC_CASSE,
    BRUME, CORAIL, CORAIL_CLAIR, NUAGE, ORANGE_SOLEIL, OR_PALE, PLUIE, PLUIE_SOMBRE, VIOLET_ORAGE,
};
use pepin_core::zones::Theme;
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use crate::decor::{camera_fond, decalage, enfoncement};
use crate::monde::Monde;
use crate::rendu::couleur_mq;

const T: i32 = TAILLE_TUILE as i32;
const W: i32 = LARGEUR_LOGIQUE as i32;
const H: i32 = HAUTEUR_LOGIQUE as i32;

/// Facteurs de parallaxe horizontale de chaque plan (0 : fixe ; 1 : les tuiles).
const F_SOLEIL: f32 = 0.02;
const F_MONTAGNES: f32 = 0.08;
const F_COLLINES: f32 = 0.16;
const F_FORET: f32 = 0.3;
const F_ARBRES: f32 = 0.55;
/// Et de parallaxe verticale (le plan s'enfonce quand la caméra monte), comptée
/// depuis le MILIEU de la hauteur du niveau : la vallée se voit en entier du haut
/// au bas de la descente.
const V_CIEL: f32 = 0.02;
const V_MONTAGNES: f32 = 0.04;
const V_COLLINES: f32 = 0.06;
const V_FORET: f32 = 0.08;
/// Lignes de pied (écran, caméra à mi-hauteur) : montagnes, collines, la rivière
/// du fond (au pied du rang clair de la forêt et du moulin), le rang sombre. Au-dessus
/// de la ligne de l'eau jouable (Pépin est le plus souvent vers le bas de l'écran),
/// pour que la vallée se voie au-dessus des berges.
const PIED_MONTAGNES: i32 = 82;
const PIED_COLLINES: i32 = 88;
const RIVIERE_DU_FOND: i32 = 90;
const PIED_RANG_SOMBRE: i32 = 106;
/// Période (px du plan de la forêt) du moulin : un moulin par période.
const PERIODE_MOULIN: i32 = 760;

fn rect(x: i32, y: i32, w: i32, h: i32, c: Couleur) {
    if w > 0 && h > 0 {
        draw_rectangle(x as f32, y as f32, w as f32, h as f32, couleur_mq(c));
    }
}

/// Nombre pseudo-aléatoire stable dérivé d'un entier (même entrée, même sortie).
fn hache(n: i32, graine: u32) -> u32 {
    let mut h = (n as u32).wrapping_mul(0x9E37_79B1) ^ graine.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h
}

fn est_la_riviere(monde: &Monde) -> bool {
    monde.infos.theme == Theme::Riviere
}

/// Le dégradé du ciel : (couleur, rangée de l'écran où elle commence), du haut vers
/// le bas. L'horizon (or pâle) est derrière les montagnes. Fonction pure.
pub(crate) fn bandes_du_ciel(dy: i32) -> [(Couleur, i32); 6] {
    [
        (ARDOISE_CLAIRE, i32::MIN),
        (VIOLET_ORAGE, 14 + dy),
        (CORAIL_CLAIR, 36 + dy),
        (ORANGE_SOLEIL, 54 + dy),
        (OR_PALE, 70 + dy),
        (BRUME, 86 + dy),
    ]
}

/// Ciel : chaque bande commence par deux rangées en damier (un pixel sur deux de
/// la couleur d'avant) pour un passage doux sans transparence.
fn dessiner_ciel(dy: i32) {
    let bandes = bandes_du_ciel(dy);
    for (k, &(c, debut)) in bandes.iter().enumerate() {
        let haut = debut.max(0);
        let bas = bandes.get(k + 1).map_or(H, |b| b.1).min(H);
        rect(0, haut, W, bas - haut, c);
        if k > 0 {
            let avant = bandes[k - 1].0;
            for r in 0..2 {
                for x in (r % 2..W).step_by(2) {
                    rect(x, debut + r, 1, 1, avant);
                }
            }
        }
    }
}

/// Un disque plein, rangée par rangée.
fn disque(cx: i32, cy: i32, r: i32, c: Couleur) {
    for dy in -r..=r {
        let l = (((r * r - dy * dy) as f32).sqrt()).round() as i32;
        rect(cx - l, cy + dy, 2 * l + 1, 1, c);
    }
}

fn dessiner_soleil(cam_x: i32, dy: i32, pas: u32) {
    let x = 236 - decalage(cam_x, F_SOLEIL);
    let y = 68 + dy;
    // Le halo respire très lentement.
    let souffle = ((pas as f32 / 90.0).sin() * 1.5).round() as i32;
    disque(x, y, 22 + souffle, ORANGE_SOLEIL);
    disque(x, y, 15, OR_PALE);
    disque(x, y, 11, BLANC_CASSE);
    // Deux longs nuages fins qui passent devant le soleil.
    for (k, (ly, lw)) in [(60, 70), (75, 48)].into_iter().enumerate() {
        let lx = x - 40 + k as i32 * 30 + ((pas / 12) as i32 % 400) / 40;
        rect(lx, ly + dy, lw, 2, CORAIL_CLAIR);
        rect(lx + 6, ly + dy + 2, lw - 14, 1, VIOLET_ORAGE);
    }
}

/// Hauteur (px au-dessus du pied) d'une ligne de crête à l'abscisse `x` du plan.
fn crete(x: i32, base: f32, ampl: f32, periode: f32, graine: u32) -> i32 {
    let xf = x as f32;
    let v = (xf / periode).sin() * ampl
        + (xf / (periode * 0.37) + graine as f32).sin() * ampl * 0.45
        + (hache(x / 24, graine) % 5) as f32 * 0.6;
    (base + v).round() as i32
}

fn dessiner_montagnes(cam_x: i32, pied: i32) {
    let dec = decalage(cam_x, F_MONTAGNES);
    for sx in (0..W).step_by(2) {
        let x = sx + dec;
        let h = crete(x, 36.0, 12.0, 46.0, 3);
        let haut = pied - h;
        rect(sx, haut, 2, H - haut, ARDOISE_PALE);
        // Le flanc tourné vers le soleil (la crête monte vers la droite) s'éclaire.
        if crete(x + 2, 36.0, 12.0, 46.0, 3) > h {
            rect(sx, haut, 2, 3, BRUME);
        }
        // Neiges : les plus hauts sommets.
        if h > 44 {
            rect(sx, haut, 2, (h - 44).min(5), NUAGE);
        }
    }
}

/// Les collines, entre les montagnes et la forêt.
fn dessiner_collines(cam_x: i32, pied: i32) {
    let dec = decalage(cam_x, F_COLLINES);
    for sx in (0..W).step_by(2) {
        let x = sx + dec;
        let haut = pied - crete(x, 22.0, 7.0, 70.0, 11);
        rect(sx, haut, 2, H - haut, ARDOISE_CLAIRE);
    }
}

/// La rivière du fond, au pied du rang clair de la forêt : une bande d'eau où le
/// soleil couchant scintille (des éclats or pâle qui s'allument lentement).
fn dessiner_riviere_du_fond(cam_x: i32, y0: i32, pas: u32) {
    let dec = decalage(cam_x, F_FORET);
    rect(0, y0, W, 7, PLUIE_SOMBRE);
    rect(0, y0, W, 1, ARDOISE_PALE);
    for sx in (0..W).step_by(4) {
        let x = sx + dec;
        let h = hache(x / 4, 29);
        if h.wrapping_add(pas / 7).is_multiple_of(9) {
            rect(sx + 1, y0 + 2 + (h % 4) as i32, 2, 1, OR_PALE);
        } else if h % 7 == 1 {
            rect(sx, y0 + 3 + (h % 3) as i32, 3, 1, ARDOISE_PALE);
        }
    }
    rect(0, y0 + 7, W, H, ARDOISE);
}

/// Un sapin : des gradins de plus en plus larges, du sommet au pied.
fn sapin(x: i32, sommet: i32, hauteur: i32, c: Couleur) {
    let gradins = (hauteur / 6).max(2);
    for g in 0..gradins {
        let y = sommet + g * hauteur / gradins;
        let l = 2 + g * 3;
        rect(x - l / 2, y, l, hauteur / gradins + 2, c);
    }
    rect(x - 1, sommet + hauteur, 2, H, c);
}

/// Un feuillu : un tronc et une couronne ronde, éclairée côté soleil.
fn feuillu(x: i32, sommet: i32, r: i32, c: Couleur, clair: Couleur) {
    disque(x, sommet + r, r, c);
    rect(x + r / 3, sommet + 2, r / 2, 2, clair);
    rect(x - 1, sommet + 2 * r, 3, H, c);
}

/// Le vieux moulin : murs de pierre, toit, fenêtre allumée, la roue qui tourne et
/// l'eau qu'elle verse. `x` : le pied du mur gauche (écran), `pied` : le sol.
fn moulin(x: i32, pied: i32, pas: u32) {
    let (l, h) = (30, 22);
    rect(x, pied - h, l, h + 2, ARDOISE);
    rect(x + 2, pied - h + 2, l - 4, 1, ARDOISE_CLAIRE);
    // Le toit, en gradins (violet sombre), et sa cheminée.
    for k in 0..6 {
        rect(
            x - 3 + k * 3,
            pied - h - 2 - k * 2,
            l + 6 - k * 6,
            2,
            VIOLET_ORAGE,
        );
    }
    rect(x + 20, pied - h - 14, 4, 8, ARDOISE_NUIT);
    // La fenêtre allumée (or pâle : une lueur, pas une goutte d'or).
    rect(x + 8, pied - h + 7, 5, 5, OR_PALE);
    rect(x + 10, pied - h + 7, 1, 5, ARDOISE_NUIT);
    rect(x + 18, pied - 9, 6, 9, ARDOISE_NUIT);
    // La roue, contre le mur droit : jante, moyeu, huit rayons qui tournent.
    let (rx, ry, r) = (x + l + 9, pied - 6, 11);
    let angle = pas as f32 / 120.0;
    for k in 0..8 {
        let a = angle + k as f32 * std::f32::consts::FRAC_PI_4;
        for d in 2..=r {
            let px = rx + (a.cos() * d as f32).round() as i32;
            let py = ry + (a.sin() * d as f32).round() as i32;
            rect(px, py, 1, 1, ARDOISE_NUIT);
        }
        // Les palettes, au bout des rayons.
        let px = rx + (a.cos() * r as f32).round() as i32;
        let py = ry + (a.sin() * r as f32).round() as i32;
        rect(px - 1, py - 1, 3, 3, ARDOISE_OMBRE);
    }
    // La jante, un cercle de pixels sous les palettes.
    for k in 0..32 {
        let a = k as f32 * std::f32::consts::PI / 16.0;
        let px = rx + (a.cos() * (r - 2) as f32).round() as i32;
        let py = ry + (a.sin() * (r - 2) as f32).round() as i32;
        rect(px, py, 1, 1, ARDOISE_NUIT);
    }
    rect(rx - 1, ry - 1, 3, 3, ARDOISE_CLAIRE);
    // L'eau que verse la roue, qui tombe en filets dans le bief du moulin.
    for k in 0..3 {
        let fy = ((pas as i32 / 2) + k * 5) % 14;
        rect(rx + r - 2 + k, ry - 4 + fy, 1, 2, PLUIE);
    }
}

/// Un rang d'arbres (sapins et feuillus mêlés) posé sur la ligne `pied`.
fn rang_d_arbres(dec: i32, pied: i32, rang: u32, (c, clair): (Couleur, Couleur), base: i32) {
    let ecart = if rang == 0 { 8 } else { 11 };
    let premier = (dec - 40).div_euclid(ecart);
    for k in premier..premier + W / ecart + 8 {
        let h = hache(k, 50 + rang);
        let x = k * ecart - dec + (h % 5) as i32;
        // Le rang sombre s'ouvre devant le moulin, et en clairières.
        let dans_moulin = (k * ecart).rem_euclid(PERIODE_MOULIN) < 100;
        if rang == 1 && (dans_moulin || h.is_multiple_of(5)) {
            continue;
        }
        let haut = base + (h % 11) as i32;
        if h.is_multiple_of(3) {
            feuillu(x, pied - haut, 6 + (h % 3) as i32, c, clair);
        } else {
            sapin(x, pied - haut, haut - 4, c);
        }
    }
}

/// La forêt : le rang clair, la rivière du fond et son moulin, le rang sombre et le
/// sous-bois (plein jusqu'au bas de l'écran).
fn dessiner_foret(cam_x: i32, dy: i32, pas: u32) {
    let dec = decalage(cam_x, F_FORET);
    let riviere = RIVIERE_DU_FOND + dy;
    rang_d_arbres(dec, riviere + 2, 0, (ARDOISE_CLAIRE, ARDOISE_PALE), 26);
    dessiner_riviere_du_fond(cam_x, riviere, pas);
    let premier = dec.div_euclid(PERIODE_MOULIN) - 1;
    for k in premier..premier + 3 {
        let x = k * PERIODE_MOULIN + 30 - dec;
        if (-80..W + 20).contains(&x) {
            moulin(x, riviere, pas);
        }
    }
    let sombre = PIED_RANG_SOMBRE + dy;
    rang_d_arbres(dec, sombre, 1, (ARDOISE, ARDOISE_CLAIRE), 24);
    dessiner_versant(dec, sombre);
}

/// Le versant boisé sous la forêt, jusqu'au bas de l'écran (vu dans les gorges,
/// entre les ponts et l'eau d'en bas) : l'ardoise du sous-bois, des strates de
/// roche en tirets et des touffes plus claires. Plus clair que la terre des berges.
fn dessiner_versant(dec: i32, haut: i32) {
    rect(0, haut, W, H, ARDOISE);
    let mut y = haut + 6;
    let mut strate = 0;
    while y < H {
        let premier = dec.div_euclid(12) - 1;
        for k in premier..premier + W / 12 + 3 {
            let h = hache(k * 31 + strate, 97);
            let x = k * 12 - dec;
            if !h.is_multiple_of(3) {
                rect(x, y + (h % 3) as i32, 4 + (h % 7) as i32, 1, ARDOISE_NUIT);
            }
            if h.is_multiple_of(5) {
                rect(x + 6, y - 3, 3, 2, ARDOISE_CLAIRE);
            }
        }
        y += 9;
        strate += 1;
    }
}

/// Les grands arbres tout près : leurs troncs sombres, et leurs branches feuillues
/// qui dépassent en haut de l'écran.
fn dessiner_grands_arbres(cam_x: i32, pas: u32) {
    let dec = decalage(cam_x, F_ARBRES);
    let ecart = 230;
    let premier = (dec - 120).div_euclid(ecart);
    for k in premier..premier + W / ecart + 3 {
        let h = hache(k, 77);
        let x = k * ecart - dec + (h % 60) as i32;
        // Le tronc (ardoise : plus clair que les tuiles, plus sombre que le ciel).
        rect(x, 0, 9, H, ARDOISE);
        rect(x + 6, 0, 2, H, ARDOISE_NUIT);
        // Les branches : des masses de feuilles qui ondulent au vent.
        let vent = ((pas as f32 / 70.0 + k as f32).sin() * 2.0).round() as i32;
        for (bx, by, r) in [
            (-26, 4, 11),
            (-10, -2, 13),
            (12, 2, 12),
            (30, 8, 9),
            (-40, 12, 7),
        ] {
            disque(x + bx + vent, by, r, ARDOISE_NUIT);
            rect(x + bx + vent - r / 2, by + r - 3, r, 2, VIOLET_ORAGE);
        }
    }
}

/// Des oiseaux qui passent au loin, en petits vols, de temps en temps.
fn dessiner_oiseaux(pas: u32) {
    let periode = 1500;
    let t = (pas % periode) as i32;
    for k in 0..3 {
        let x = -20 + (t * 2) / 5 - k * 14;
        let y = 30 + k * 6 + ((pas as f32 / 25.0 + k as f32).sin() * 2.0).round() as i32;
        if !(-10..W + 10).contains(&x) {
            continue;
        }
        let haut = (pas / 10 + k as u32).is_multiple_of(2);
        let dy = if haut { -1 } else { 1 };
        rect(x - 2, y + dy, 2, 1, ARDOISE_NUIT);
        rect(x, y, 1, 1, ARDOISE_NUIT);
        rect(x + 1, y + dy, 2, 1, ARDOISE_NUIT);
    }
}

/// Des lucioles (ou des pollens) qui flottent et clignotent doucement.
fn dessiner_lucioles(cam_x: i32, dy: i32, pas: u32) {
    let dec = decalage(cam_x, 0.7);
    for k in 0..18 {
        let h = hache(k, 91);
        let base_x = (h % 640) as i32;
        let t = pas as f32 / 60.0;
        let x = (base_x - dec + ((t * 0.3 + k as f32).sin() * 10.0) as i32).rem_euclid(W + 40) - 20;
        let y = 50 + ((h >> 10) % 90) as i32 + ((t * 0.8 + k as f32 * 1.7).sin() * 6.0) as i32;
        let allume = (pas / 6 + (h >> 4) % 23) % 23 < 15;
        if allume {
            rect(x, y + dy, 1, 1, OR_PALE);
        }
    }
}

/// Derrière les tuiles : tout le fond de la vallée, par-dessus le fond générique.
pub fn dessiner_fond(monde: &Monde, cx: i32, cy: i32) {
    if !est_la_riviere(monde) {
        return;
    }
    let pas = monde.pas;
    let hauteur = monde.niveau.hauteur_px() as i32;
    // La caméra du fond comptée depuis le milieu de sa course (0 au milieu).
    let milieu = camera_fond(cy, hauteur) + (hauteur - H).max(0) / 2;
    let v = |f: f32| enfoncement(milieu, f);
    dessiner_ciel(v(V_CIEL));
    dessiner_soleil(cx, v(V_CIEL), pas);
    dessiner_oiseaux(pas);
    dessiner_montagnes(cx, PIED_MONTAGNES + v(V_MONTAGNES));
    dessiner_collines(cx, PIED_COLLINES + v(V_COLLINES));
    dessiner_foret(cx, v(V_FORET), pas);
    dessiner_grands_arbres(cx, pas);
    dessiner_lucioles(cx, v(V_FORET), pas);
}

/// Devant les tuiles : reflets sur l'eau jouable, écume des rochers et des chutes,
/// roseaux au bord des berges.
pub fn dessiner(monde: &Monde, cx: i32, cy: i32) {
    if !est_la_riviere(monde) {
        return;
    }
    let n = &monde.niveau;
    let pas = monde.pas;
    let (tx0, tx1) = (cx.div_euclid(T) - 1, (cx + W).div_euclid(T) + 1);
    // Reflets : de courts éclats or pâle et blancs qui s'allument sur la surface.
    for e in &n.eaux {
        let y = e.surface().round() as i32 - cy;
        if !(-4..H + 4).contains(&y) {
            continue;
        }
        let (g, d) = (e.zone.x as i32, e.zone.droite() as i32);
        let mut x = g.max(cx - 8) / 6 * 6;
        while x < d.min(cx + W + 8) {
            let h = hache(x, 13);
            let phase = (h / 3).wrapping_add(pas / 9) % 14;
            if phase < 3 {
                let c = if h.is_multiple_of(2) {
                    OR_PALE
                } else {
                    BLANC_CASSE
                };
                rect(x - cx, y + 3 + (h % 3) as i32 * 2, 2 + (h % 2) as i32, 1, c);
            }
            x += 6;
        }
    }
    // Écume au pied des rochers dans l'eau et au bord des berges qui l'arrêtent.
    for ty in 0..n.hauteur_px() as i32 / T {
        let y = ty * T - cy;
        if !(-T..H + T).contains(&y) {
            continue;
        }
        for tx in tx0..=tx1 {
            if n.tuile_decor(tx, ty) != Tuile::Eau || n.tuile_decor(tx, ty - 1) == Tuile::Eau {
                continue;
            }
            for (cote, voisin) in [(-1, tx - 1), (1, tx + 1)] {
                let t = n.tuile_decor(voisin, ty);
                if t.est_solide() {
                    let bord = if cote < 0 { tx * T } else { tx * T + T };
                    ecume(bord - cx, y, -cote, pas, tx);
                }
            }
        }
    }
    // Écume au pied de chaque chute : un bouillon qui gonfle sur l'eau d'arrivée.
    for e in &n.entites {
        let Entite::Chute(c) = e else { continue };
        let pied = c.zone.bas() as i32 - cy;
        let (g, l) = (c.zone.x as i32 - cx, c.zone.w as i32);
        for k in 0..(l + 12) / 3 {
            let h = hache(k + c.zone.x as i32, 41);
            let hauteur = ((pas / 4 + h % 7) % 5) as i32;
            rect(
                g - 6 + k * 3,
                pied - hauteur,
                2,
                2 + hauteur / 2,
                BLANC_CASSE,
            );
        }
        rect(g - 4, pied + 2, l + 8, 1, NUAGE);
    }
    // Roseaux au bord des berges qui touchent l'eau (tiges fines, épi rouille).
    for ty in 1..n.hauteur_px() as i32 / T {
        let y = ty * T - cy;
        if !(-T..H + T).contains(&y) {
            continue;
        }
        for tx in tx0..=tx1 {
            if n.tuile_decor(tx, ty) != Tuile::Brique || n.tuile_decor(tx, ty - 1) != Tuile::Vide {
                continue;
            }
            for (cote, voisin) in [(-1, tx - 1), (1, tx + 1)] {
                if n.tuile_decor(voisin, ty) == Tuile::Eau {
                    let x = if cote < 0 { tx * T + 1 } else { tx * T + T - 6 } - cx;
                    roseaux(x, y, pas, tx);
                }
            }
        }
    }
}

/// L'écume contre un bord (`x`, écran) sur la surface `y` ; `vers` : le côté de
/// l'eau (1 : à droite du bord).
fn ecume(x: i32, y: i32, vers: i32, pas: u32, graine: i32) {
    for k in 0..4 {
        let h = hache(graine * 7 + k, 23);
        let monte = ((pas / 5 + h % 6) % 4) as i32;
        let dx = if vers > 0 { k * 2 } else { -k * 2 - 2 };
        rect(x + dx, y - 1 - monte / 2 + (k % 2), 2, 2, BLANC_CASSE);
    }
    rect(if vers > 0 { x } else { x - 9 }, y + 2, 9, 1, NUAGE);
}

/// Une touffe de roseaux posée sur la berge (`x`, `y` : le dessus de la case).
fn roseaux(x: i32, y: i32, pas: u32, graine: i32) {
    for k in 0..3 {
        let h = hache(graine * 3 + k, 61);
        let haut = 7 + (h % 5) as i32;
        let pench = i32::from((pas / 40 + h % 3).is_multiple_of(3));
        let bx = x + k * 2;
        rect(bx, y - haut, 1, haut, ARDOISE_CLAIRE);
        rect(bx + pench, y - haut - 3, 2, 3, CORAIL);
    }
}

/// Le tronc flottant de la mare (plateforme mobile `1` de la rivière), à la place
/// du dessin générique. Faux hors du thème Rivière.
pub fn dessiner_tronc(monde: &Monde, sx: f32, sy: f32) -> bool {
    if !est_la_riviere(monde) {
        return false;
    }
    let (x, y) = (sx.round() as i32, sy.round() as i32);
    // L'écorce : contour, bois corail, une ligne claire dessus, les nœuds.
    rect(x, y + 1, 32, 8, ARDOISE_OMBRE);
    rect(x + 1, y + 2, 30, 6, CORAIL);
    rect(x + 2, y + 2, 26, 1, ORANGE_SOLEIL);
    rect(x + 1, y + 6, 30, 1, VIOLET_ORAGE);
    for k in [8, 19] {
        rect(x + k, y + 4, 3, 1, ARDOISE_NUIT);
    }
    // Le bout coupé, ses cernes.
    rect(x + 27, y + 2, 4, 6, CORAIL_CLAIR);
    rect(x + 28, y + 4, 2, 2, ORANGE_SOLEIL);
    // Les rides de l'eau autour.
    let r = ((monde.pas / 8) % 3) as i32;
    rect(x - 2 - r, y + 9, 3, 1, BLANC_CASSE);
    rect(x + 31 + r, y + 9, 3, 1, BLANC_CASSE);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le ciel descend du sombre (en haut) vers le clair (l'horizon), et l'or vif
    /// des gouttes n'y paraît jamais.
    #[test]
    fn le_ciel_va_du_sombre_au_clair_sans_or_vif() {
        let b = bandes_du_ciel(0);
        for paire in b.windows(2) {
            assert!(paire[0].1 < paire[1].1);
        }
        assert!(b.iter().all(|(c, _)| *c != pepin_core::palette::OR));
        let lum = |c: Couleur| c.r as u32 + c.g as u32 + c.b as u32;
        assert!(lum(b[0].0) < lum(b[4].0));
    }
}
