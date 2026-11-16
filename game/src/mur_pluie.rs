//! Dessin du MUR DE PLUIE du niveau de poursuite et du soleil qui perce à
//! l'arrivée (carte 8.2). Les règles du mur sont dans `pepin_core::poursuite` ;
//! ici, rien que du dessin, en pixels entiers de la palette :
//! - le rideau : un bloc d'ardoise très sombre, du bord gauche de l'écran au front,
//!   avec un bord déchiqueté qui bouge, des traînées de pluie qui tombent dedans,
//!   une frange de gouttes cyan devant lui et, de temps en temps, un éclair ;
//! - le soleil : une fois percé (`MurDePluie::vaincu`), un disque d'or grandit en
//!   haut à droite de l'écran et ses rayons s'allongent ; la pluie du thème
//!   s'arrête (`scene::dessiner`).
//!
//! Notions Rust illustrées ici : `let … else` pour sortir tôt quand le niveau n'a
//! pas de mur, tableaux constants de motifs, arithmétique entière (`rem_euclid`)
//! pour animer sans flottants.

use macroquad::prelude::draw_rectangle;
use pepin_core::palette::*;
use pepin_core::HAUTEUR_LOGIQUE;

use crate::monde::Monde;
use crate::rendu::couleur_mq;

fn rect(x: i32, y: i32, w: i32, h: i32, c: Couleur) {
    if w > 0 && h > 0 {
        draw_rectangle(x as f32, y as f32, w as f32, h as f32, couleur_mq(c));
    }
}

/// Hauteur (px) d'une bande du bord déchiqueté.
const BANDE: i32 = 4;
/// Centre du soleil qui perce, à l'écran (px logiques).
const SOLEIL_X: i32 = 252;
const SOLEIL_Y: i32 = 44;
/// Retrait (px) du bord de chaque bande, selon son rang : le bord dentelé.
const DENTS: [i32; 6] = [0, 3, 1, 5, 2, 4];
/// Un éclair dans le rideau tous les `CYCLE_ECLAIR` pas, visible `DUREE_ECLAIR` pas.
const CYCLE_ECLAIR: u32 = 96;
const DUREE_ECLAIR: u32 = 6;
/// L'éclair part sous le HUD (le compteur de gouttes, en haut à gauche).
const HAUT_ECLAIR: i32 = 16;
/// Zigzag de l'éclair : décalage x à chaque segment de 12 px.
const ZIGZAG: [i32; 9] = [0, -4, 2, -6, -1, -7, -3, -9, -5];

/// Retrait du bord du rideau pour la bande `k` au pas `pas` (le bord ondule).
fn retrait_bord(k: i32, pas: u32) -> i32 {
    DENTS[(k + (pas / 4) as i32).rem_euclid(DENTS.len() as i32) as usize]
}

/// Dessine le rideau de pluie (devant le niveau). `cx` : caméra entière.
pub fn dessiner_mur(monde: &Monde, cx: i32) {
    let Some(mur) = &monde.mur else {
        return;
    };
    let front = mur.front().round() as i32 - cx;
    if front < -8 {
        return;
    }
    let h = HAUTEUR_LOGIQUE as i32;
    let pas = monde.pas;
    // 1. Le corps du rideau, bande par bande, bord dentelé et liseré.
    for k in 0..(h + BANDE - 1) / BANDE {
        let y = k * BANDE;
        let bord = front - retrait_bord(k, pas);
        rect(0, y, bord, BANDE, ARDOISE_OMBRE);
        rect(bord - 2, y, 2, BANDE, ARDOISE_NUIT);
    }
    // 2. Pluie battante dans le rideau : une colonne tous les 4 px, quatre traînées
    //    de 6 px par colonne (un quart d'écran entre elles), trois vitesses et deux
    //    tons, pour qu'on lise « de la pluie » et pas un simple mur noir.
    let mut x = front - 7;
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
    // 3. La frange : des gouttes cyan projetées devant le front.
    for j in 0..14 {
        let y = ((j * 29) as u32 + pas * 4) % h as u32;
        let dx = 1 + (j * 5 + (pas / 3) as i32) % 7;
        rect(front + dx, y as i32, 1, 2, PLUIE);
    }
    // 4. L'éclair : un zigzag blanc dans le rideau, quelques pas tous les 1,6 s.
    if pas % CYCLE_ECLAIR < DUREE_ECLAIR && front > 24 {
        let x0 = front - 18;
        for (s, d) in ZIGZAG.iter().enumerate() {
            let y = HAUT_ECLAIR + s as i32 * 12;
            let suivant = ZIGZAG.get(s + 1).copied().unwrap_or(*d);
            let (a, b) = ((*d).min(suivant), (*d).max(suivant));
            rect(x0 + d, y, 2, 12, BLANC_CASSE);
            rect(x0 + a, y + 11, b - a + 2, 2, BLANC_CASSE);
            rect(x0 + d - 1, y + 3, 1, 4, OR_PALE);
        }
    }
}

/// Le soleil qui perce au-dessus de l'arrivée, une fois le mur arrêté : le disque
/// grandit pendant `pas_soleil` pas, puis ses rayons pointillés s'allongent.
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
