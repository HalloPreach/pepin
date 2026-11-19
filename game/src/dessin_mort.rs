//! Dessin du temps de mort de Pépin (carte 9.24, l'état est dans `crate::mort`).
//!
//! Tout se calcule à partir des pas écoulés depuis la mort : rien n'est gardé
//! d'une image à l'autre, et le monde figé ne bouge pas pendant ce temps.
//! - Noyade : Pépin (sans son parapluie) s'enfonce en se débattant, coupé à la
//!   surface ; des bulles cyan remontent ; le parapluie, couché, flotte à la surface.
//! - Dégât : un flash blanc, puis Pépin est éjecté vers le haut et retombe à
//!   travers le décor en tournant sur lui-même, parapluie compris.
//! - Chute dans le vide : rien (la caméra ne l'a pas suivi).
//! - Puis le rideau : deux bandes d'ardoise qui se ferment et se rouvrent.

use macroquad::prelude::draw_rectangle;
use pepin_core::palette::ARDOISE_OMBRE;
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use crate::monde::Monde;
use crate::mort::{hauteur_rideau, CauseDeMort, RIDEAU_FERME, RIDEAU_PAS};
use crate::rendu::couleur_mq;
use crate::sprites::{Banque, JeuImages};

/// Pas que Pépin met à s'enfoncer tout entier sous la surface.
const ENFONCEMENT_PAS: f32 = 48.0;
/// Durée (pas) du flash blanc d'un dégât.
const FLASH_PAS: u16 = 12;
/// Éjection d'un dégât : vitesse de départ (px par pas, vers le haut), recul
/// (px par pas, vers l'arrière) et pesanteur (px par pas²).
const EJECTION_VY: f32 = -3.2;
const EJECTION_VX: f32 = 0.6;
const EJECTION_G: f32 = 0.18;
/// Largeur (px) du corps de Pépin dans ses images 16x16 : le parapluie fermé
/// occupe les colonnes 11 à 15.
const CORPS_L: f32 = 11.0;

/// Pépin est-il caché par le temps de mort ? Oui jusqu'à ce que le rideau soit
/// fermé : il ne réapparaît à la lanterne qu'à l'ouverture.
pub fn pepin_cache(reste: u16) -> bool {
    reste > RIDEAU_FERME
}

/// L'animation de la mort en cours, à sa place dans le niveau (`(cx, cy)` :
/// caméra). Rien hors du temps de mort.
pub fn dessiner(monde: &Monde, banque: &Banque, images: &JeuImages, (cx, cy): (i32, i32)) {
    let reste = monde.reapparition;
    if reste == 0 || !monde.temps_de_mort {
        return;
    }
    let m = &monde.mort;
    let ecoule = m.ecoule(reste);
    let (bx, by, bw, bh) = m.boite;
    // Les images 16x16 de Pépin se posent comme en jeu : centrées, pieds en bas.
    let x0 = bx + bw / 2.0 - 8.0 - cx as f32;
    let y0 = by + bh - 16.0 - cy as f32;
    if ecoule < m.cause.duree() {
        match m.cause {
            CauseDeMort::Noyade => noyade(banque, images, ecoule, (x0, y0), m.surface - cy as f32),
            CauseDeMort::Degat => degat(banque, images, ecoule, (x0, y0), m.regarde_droite),
            CauseDeMort::Vide => {}
        }
    } else if reste <= RIDEAU_FERME {
        // Rideau qui se rouvre : les étincelles se rassemblent sur Pépin replacé.
        let k = usize::from(RIDEAU_FERME - reste) * 3 / usize::from(RIDEAU_FERME);
        let j = &monde.joueur.boite;
        banque.dessiner(
            images.reapparition[k.min(2)],
            j.x - 3.0 - cx as f32,
            j.y - 2.0 - cy as f32,
            false,
        );
    }
}

/// Noyade : Pépin s'enfonce sous la surface (`surface`, à l'écran) en se débattant.
fn noyade(banque: &Banque, images: &JeuImages, e: u16, (x0, y0): (f32, f32), surface: f32) {
    let t = f32::from(e);
    // Au départ, on le voit encore presque en entier au-dessus de l'eau.
    let depart = y0.min(surface - 14.0);
    let y = depart + (t / ENFONCEMENT_PAS).min(1.0) * (surface + 2.0 - depart);
    // Il se débat : il se retourne et tressaute d'un pixel tous les 5 pas.
    let phase = !(e / 5).is_multiple_of(2);
    let x = x0 + if phase { 1.0 } else { 0.0 };
    let visible = (surface - y).min(16.0);
    if visible > 0.0 {
        banque.dessiner_partie(images.chute, (x, y), (0.0, 0.0, CORPS_L, visible), 0, phase);
    }
    // Les bulles : une tous les 9 pas, partie sous l'eau, de plus en plus bas à mesure
    // que Pépin coule ; elle remonte et éclate juste au-dessus de la surface.
    for k in 0..7u16 {
        let naissance = 4 + 9 * k;
        let Some(age) = e.checked_sub(naissance) else {
            continue;
        };
        let fond = surface + 4.0 + (f32::from(naissance) / ENFONCEMENT_PAS).min(1.0) * 8.0;
        let by = fond - f32::from(age) * 0.5;
        if by < surface - 6.0 {
            continue;
        }
        let bx = x0
            + 3.0
            + f32::from(k % 3) * 2.0
            + if (age / 6).is_multiple_of(2) {
                0.0
            } else {
                1.0
            };
        banque.dessiner(if k % 2 == 0 { "bulle0" } else { "bulle1" }, bx, by, false);
    }
    // Le parapluie lâché : couché sur l'eau (un quart de tour), il dérive un peu et
    // tangue d'un pixel. Ses colonnes 11 à 15 (5x13), tournées : 13x5 ; le coin de
    // l'image tournée est à (+4, +4) de celui de l'image d'origine.
    let derive = (t * 0.1).min(6.0);
    let tangue = if (e / 12).is_multiple_of(2) { 0.0 } else { 1.0 };
    banque.dessiner_partie(
        images.chute,
        (x0 + 6.0 + derive - 4.0, surface - 3.0 + tangue - 4.0),
        (CORPS_L, 0.0, 16.0 - CORPS_L, 13.0),
        1,
        false,
    );
}

/// Dégât : un flash blanc à l'endroit du coup, puis Pépin éjecté vers le haut (en
/// arrière) qui retombe à travers le décor en tournant d'un quart de tour tous les 5 pas.
fn degat(banque: &Banque, images: &JeuImages, e: u16, (x0, y0): (f32, f32), droite: bool) {
    let miroir = !droite;
    if e < FLASH_PAS {
        if (e / 3).is_multiple_of(2) {
            banque.dessiner_blanc(images.saut, x0, y0, miroir);
        } else {
            banque.dessiner(images.saut, x0, y0, miroir);
        }
        return;
    }
    let t = f32::from(e - FLASH_PAS);
    let recul = if droite { -EJECTION_VX } else { EJECTION_VX };
    let x = x0 + recul * t;
    let y = y0 + EJECTION_VY * t + EJECTION_G * t * t / 2.0;
    let quarts = ((e - FLASH_PAS) / 5 % 4) as u8;
    banque.dessiner_partie(images.saut, (x, y), (0.0, 0.0, 16.0, 16.0), quarts, miroir);
}

/// Le rideau du temps de mort : deux bandes d'ardoise, du haut et du bas de l'écran,
/// qui se ferment sur l'endroit de la mort puis se rouvrent sur la lanterne.
pub fn dessiner_rideau(reste: u16) {
    if reste == 0 || reste > RIDEAU_PAS {
        return;
    }
    let demi = (HAUTEUR_LOGIQUE as i32 + 1) / 2;
    let h = hauteur_rideau(reste, demi);
    if h <= 0 {
        return;
    }
    let couleur = couleur_mq(ARDOISE_OMBRE);
    let l = LARGEUR_LOGIQUE as f32;
    draw_rectangle(0.0, 0.0, l, h as f32, couleur);
    draw_rectangle(
        0.0,
        (HAUTEUR_LOGIQUE as i32 - h) as f32,
        l,
        h as f32,
        couleur,
    );
}
