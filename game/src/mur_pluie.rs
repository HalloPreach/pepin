//! Dessin du FRONT D'ORAGE du niveau « Le retour du Grand Nuage » et du soleil qui
//! perce à l'arrivée (cartes 8.2 et 8.2b). Le Grand Nuage, vaincu au niveau 6, s'est
//! enfui : il remonte de la mer de nuages et poursuit Pépin. Les règles du front (le
//! « mur de pluie » : vitesse, contact qui tue) sont dans `pepin_core::poursuite` ;
//! ici, rien que du dessin, en pixels entiers de la palette :
//! - le front : un nuage d'orage sombre (ardoise de nuit, liseré violet d'orage : les
//!   couleurs du thème Cyclone et du ventre du boss), du bord gauche de l'écran au
//!   front, au bord en volutes qui roulent ; la pluie et les éclairs tombent DEDANS,
//!   jamais devant (le ciel d'altitude reste clair devant Pépin) ;
//! - le visage du Grand Nuage (le sprite du boss, en colère) collé au bord qui
//!   avance ; essoufflé quand le soleil perce et qu'il recule ;
//! - le soleil : une fois percé (`MurDePluie::vaincu`), un disque d'or grandit en
//!   haut à droite de l'écran et ses rayons s'allongent.
//!
//! Notions Rust illustrées ici : `let … else` pour sortir tôt quand le niveau n'a
//! pas de front, tableaux constants de motifs, arithmétique entière (`rem_euclid`)
//! pour animer sans flottants.

use macroquad::prelude::draw_rectangle;
use pepin_core::entities::nuage::PhaseNuage;
use pepin_core::palette::*;
use pepin_core::HAUTEUR_LOGIQUE;

use crate::monde::Monde;
use crate::rendu::couleur_mq;
use crate::scene::image_nuage;
use crate::sprites::Banque;

fn rect(x: i32, y: i32, w: i32, h: i32, c: Couleur) {
    if w > 0 && h > 0 {
        draw_rectangle(x as f32, y as f32, w as f32, h as f32, couleur_mq(c));
    }
}

/// Hauteur (px) d'une bande du bord en volutes.
const BANDE: i32 = 2;
/// Hauteur (px) d'une volute du bord du nuage (une bosse arrondie).
const VOLUTE: i32 = 24;
/// Épaisseur (px) du liseré violet du bord.
const LISERE: i32 = 3;
/// Centre du soleil qui perce, à l'écran (px logiques).
const SOLEIL_X: i32 = 252;
const SOLEIL_Y: i32 = 44;
/// Un éclair dans le nuage tous les `CYCLE_ECLAIR` pas, visible `DUREE_ECLAIR` pas.
const CYCLE_ECLAIR: u32 = 96;
const DUREE_ECLAIR: u32 = 6;
/// L'éclair part sous le HUD (le compteur de gouttes, en haut à gauche).
const HAUT_ECLAIR: i32 = 16;
/// Zigzag de l'éclair : décalage x à chaque segment de 12 px.
const ZIGZAG: [i32; 9] = [0, -4, 2, -6, -1, -7, -3, -9, -5];
/// Visage du Grand Nuage (sprite du boss, 48 x 32) : haut à l'écran, au-dessus de la
/// ligne de course de Pépin, et son balancement vertical (un pixel tous les 10 pas).
const VISAGE_Y: i32 = 46;
const VISAGE_L: i32 = 48;
const BALANCEMENT: [i32; 4] = [0, 1, 2, 1];

/// Retrait (px) du bord du nuage à la hauteur `y` de l'écran au pas `pas` : des
/// volutes arrondies (une parabole par volute, 0 au milieu, 6 aux creux) qui roulent
/// vers le bas.
fn retrait_bord(y: i32, pas: u32) -> i32 {
    let d = (y + (pas / 3) as i32).rem_euclid(VOLUTE) - VOLUTE / 2;
    d * d / VOLUTE
}

/// Dessine le front d'orage et le visage du Grand Nuage (devant le niveau).
/// `cx` : caméra entière.
pub fn dessiner_mur(monde: &Monde, banque: &Banque, cx: i32) {
    let Some(mur) = &monde.mur else {
        return;
    };
    let front = mur.front().round() as i32 - cx;
    if front < -8 {
        return;
    }
    let h = HAUTEUR_LOGIQUE as i32;
    let pas = monde.pas;
    // 1. Le corps du nuage, bande par bande : liseré violet d'orage au bord en
    //    volutes, ardoise de nuit dedans, et un cœur plus sombre qui roule à
    //    contretemps.
    for k in 0..(h + BANDE - 1) / BANDE {
        let y = k * BANDE;
        let bord = front - retrait_bord(y, pas);
        rect(0, y, bord, BANDE, VIOLET_ORAGE);
        rect(0, y, bord - LISERE, BANDE, ARDOISE_NUIT);
        let coeur = front - 26 - retrait_bord(y + VOLUTE / 2, pas + 36);
        rect(0, y, coeur, BANDE, ARDOISE_OMBRE);
    }
    // 2. La pluie DANS le nuage : une colonne tous les 4 px, derrière le liseré,
    //    quatre traînées de 6 px par colonne, trois vitesses et deux tons.
    let mut x = front - 10;
    let mut i = 0;
    while x > -4 {
        let vitesse = 4 + (i % 3) as u32;
        let couleur = if i % 2 == 0 {
            ARDOISE_CLAIRE
        } else {
            PLUIE_SOMBRE
        };
        for q in 0..4 {
            let y = ((i * 37 + q * 45) as u32 + pas * vitesse) % (h as u32 + 8);
            rect(x, y as i32 - 8, 1, 6, couleur);
        }
        x -= 4;
        i += 1;
    }
    // 3. L'éclair : un zigzag blanc dans le nuage, quelques pas tous les 1,6 s
    //    (derrière le visage).
    if pas % CYCLE_ECLAIR < DUREE_ECLAIR && front > 24 {
        let x0 = front - 14;
        for (s, d) in ZIGZAG.iter().enumerate() {
            let y = HAUT_ECLAIR + s as i32 * 12;
            let suivant = ZIGZAG.get(s + 1).copied().unwrap_or(*d);
            let (a, b) = ((*d).min(suivant), (*d).max(suivant));
            rect(x0 + d, y, 2, 12, BLANC_CASSE);
            rect(x0 + a, y + 11, b - a + 2, 2, BLANC_CASSE);
            rect(x0 + d - 1, y + 3, 1, 4, OR_PALE);
        }
    }
    // 4. Le visage du Grand Nuage, collé au bord qui avance (jamais devant : le
    //    sprite finit au front, là où le nuage tue). En colère tant qu'il poursuit
    //    Pépin, essoufflé quand le soleil perce et qu'il recule.
    let phase = if mur.vaincu() {
        PhaseNuage::Essouffle
    } else {
        PhaseNuage::Colere
    };
    if let Some(nom) = image_nuage(phase, 0, pas) {
        let y = VISAGE_Y + BALANCEMENT[(pas / 10) as usize % BALANCEMENT.len()];
        banque.dessiner(nom, (front - VISAGE_L) as f32, y as f32, false);
    }
}

/// Le soleil qui perce au-dessus de l'arrivée, une fois le Grand Nuage arrêté : le
/// disque grandit pendant `pas_soleil` pas, puis ses rayons pointillés s'allongent.
/// Derrière le niveau (juste après le ciel).
pub fn dessiner_soleil(monde: &Monde) {
    let Some(mur) = &monde.mur else {
        return;
    };
    let Some(t) = mur.pas_soleil() else {
        return;
    };
    let t = t as i32;
    // Fixe à l'écran, en haut à droite : il perce devant Pépin, au-dessus de la
    // dernière ligne droite (le drapeau est encore hors de l'écran).
    let (sx, sy) = (SOLEIL_X, SOLEIL_Y);
    // Les rayons : huit directions, des points 2x2 tous les 6 px.
    let longueur = (t * 2).min(120);
    for (dx, dy) in [
        (1, 0),
        (-1, 0),
        (0, 1),
        (0, -1),
        (1, 1),
        (-1, 1),
        (1, -1),
        (-1, -1),
    ] {
        let mut r = 30 + (t / 2) % 6;
        while r < 30 + longueur {
            let (ux, uy) = if dx != 0 && dy != 0 {
                (dx * r * 7 / 10, dy * r * 7 / 10)
            } else {
                (dx * r, dy * r)
            };
            rect(sx + ux - 1, sy + uy - 1, 2, 2, OR_PALE);
            r += 6;
        }
    }
    // Le disque : grandit jusqu'à 22 px de rayon, cœur blanc.
    let rayon = (4 + t / 2).min(22);
    disque(sx, sy, rayon + 2, OR);
    disque(sx, sy, rayon, OR_PALE);
    disque(sx, sy, rayon * 2 / 3, BLANC_CASSE);
}

/// Disque plein en lignes horizontales (pixel art : ni cercle vectoriel ni flou).
fn disque(cx: i32, cy: i32, r: i32, c: Couleur) {
    for dy in -r..=r {
        let w = ((r * r - dy * dy) as f32).sqrt() as i32;
        rect(cx - w, cy + dy, 2 * w + 1, 1, c);
    }
}
