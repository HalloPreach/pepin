//! Résultats des défis (carte 7.3) : la bande sous la carte du niveau suivant et
//! la bande sous l'écran de fin de partie.
//!
//! - `dessiner_bande_niveau` : sur la carte du niveau suivant, « NIVEAU RÉUSSI :
//!   ARGENT, +9 GOUTTES » puis la ligne des 3 défis (cases cochées ou non) ;
//! - `dessiner_bande_fin` : sous l'écran de fin, une seule ligne (la place manque) qui
//!   alterne toutes les 3 s entre les défis du dernier niveau et le détail du bonus
//!   de fin de partie (gagné en or, raté en gris).
//!
//! Les petites icônes (case, médaille) sont des grilles de caractères, 1 caractère
//! = 1 pixel, comme les sprites ; elles sont posées pixel par pixel (`rect`).

use pepin_core::font::largeur_texte;
use pepin_core::palette::{
    Couleur, ARDOISE_CLAIRE, ARDOISE_NUIT, ARDOISE_OMBRE, ARDOISE_PALE, BLANC_CASSE, BRUME, CORAIL,
    CORAIL_CLAIR, NUAGE, OR, OR_PALE,
};
use pepin_core::LARGEUR_LOGIQUE;

use super::{panneau, rect};
use crate::defis::{total_bonus, LigneBonus, ResultatNiveau};
use crate::texte::{self, Style};

/// Case de défi : `#` bord, `.` fond sombre, `x` coche (si réussi).
const CASE: [&str; 7] = [
    "#######", "#.....#", "#....x#", "#x..xx#", "#xxxx.#", "#.xx..#", "#######",
];
/// Médaille : `r` ruban, `#` contour, `o` disque, `+` reflet.
const MEDAILLE: [&str; 8] = [
    "r.....r", ".r...r.", ".#####.", "#oo+oo#", "#o+ooo#", "#ooooo#", "#ooooo#", ".#####.",
];
/// Largeur des icônes, en pixels.
pub const L_ICONE: f32 = 7.0;
/// Écart après une icône, et entre deux défis.
pub const ECART_ICONE: f32 = 2.0;
const ECART_DEFI: f32 = 5.0;
/// Pas de la bande de fin entre ses deux pages (3 s).
const PAGE_FIN: u32 = 180;

fn ombre(c: Couleur) -> Style {
    Style::simple(c, 1).avec_ombre(ARDOISE_OMBRE)
}

/// Pose une grille d'icône en `(x, y)` ; `couleur` dit la couleur de chaque
/// caractère (`None` : transparent).
pub fn grille(lignes: &[&str], x: f32, y: f32, couleur: impl Fn(char) -> Option<Couleur>) {
    for (j, ligne) in lignes.iter().enumerate() {
        for (i, c) in ligne.chars().enumerate() {
            if let Some(coul) = couleur(c) {
                rect(x + i as f32, y + j as f32, 1.0, 1.0, coul);
            }
        }
    }
}

pub fn case(x: f32, y: f32, cochee: bool) {
    grille(&CASE, x, y, |c| match c {
        '#' => Some(BRUME),
        '.' => Some(ARDOISE_OMBRE),
        'x' if cochee => Some(OR),
        'x' => Some(ARDOISE_OMBRE),
        _ => None,
    });
}

/// Couleur et nom de la médaille `n` (0 à 3 défis réussis).
pub fn medaille(n: u8) -> (Couleur, &'static str) {
    match n {
        0 => (ARDOISE_CLAIRE, "AUCUNE MÉDAILLE"),
        1 => (CORAIL_CLAIR, "BRONZE"),
        2 => (NUAGE, "ARGENT"),
        _ => (OR, "OR"),
    }
}

pub fn icone_medaille(x: f32, y: f32, n: u8) {
    let (disque, _) = medaille(n);
    grille(&MEDAILLE, x, y - 1.0, |c| match c {
        'r' => Some(CORAIL),
        '#' => Some(ARDOISE_OMBRE),
        'o' => Some(disque),
        '+' if n > 0 => Some(BLANC_CASSE),
        '+' => Some(disque),
        _ => None,
    });
}

/// Largeur de la ligne des défis (cases + libellés, puis médaille et gain si
/// `avec_gain`).
fn largeur_ligne(r: &ResultatNiveau, avec_gain: bool) -> f32 {
    let defis: f32 = r
        .defis
        .iter()
        .map(|d| L_ICONE + ECART_ICONE + largeur_texte(&d.libelle(), 1) as f32)
        .sum::<f32>()
        + 2.0 * ECART_DEFI;
    if avec_gain {
        defis + ECART_DEFI + L_ICONE + ECART_ICONE + largeur_texte(&gain(r.gain), 1) as f32
    } else {
        defis
    }
}

fn gain(n: u32) -> String {
    format!("+{n}")
}

/// La ligne des 3 défis, centrée sur l'écran, en `y`.
pub fn ligne_defis(r: &ResultatNiveau, y: f32, avec_gain: bool) {
    let mut x = ((LARGEUR_LOGIQUE as f32 - largeur_ligne(r, avec_gain)) / 2.0).round();
    for (defi, &ok) in r.defis.iter().zip(&r.reussis) {
        case(x, y, ok);
        x += L_ICONE + ECART_ICONE;
        let libelle = defi.libelle();
        texte::dessiner(
            &libelle,
            x,
            y,
            ombre(if ok { BLANC_CASSE } else { ARDOISE_PALE }),
        );
        x += largeur_texte(&libelle, 1) as f32 + ECART_DEFI;
    }
    if avec_gain {
        icone_medaille(x, y, r.medaille);
        x += L_ICONE + ECART_ICONE;
        texte::dessiner(&gain(r.gain), x, y, ombre(OR_PALE));
    }
}

/// Texte d'en-tête de la bande de niveau : « NIVEAU RÉUSSI : ARGENT, +9 GOUTTES ».
pub fn entete(r: &ResultatNiveau) -> String {
    format!(
        "NIVEAU RÉUSSI : {}, +{} GOUTTE{}",
        medaille(r.medaille).1,
        r.gain,
        if r.gain > 1 { "S" } else { "" }
    )
}

/// Bande sous la carte du niveau suivant (y 148 à 178 : sous le panneau de la carte).
pub fn dessiner_bande_niveau(r: &ResultatNiveau) {
    let l = LARGEUR_LOGIQUE as f32;
    panneau(12.0, 148.0, l - 24.0, 30.0, ARDOISE_NUIT, ARDOISE_CLAIRE);
    let titre = entete(r);
    let x = ((l - largeur_texte(&titre, 1) as f32 - L_ICONE - ECART_ICONE) / 2.0).round();
    icone_medaille(x, 152.0, r.medaille);
    texte::dessiner(&titre, x + L_ICONE + ECART_ICONE, 152.0, ombre(OR_PALE));
    ligne_defis(r, 165.0, false);
}

/// Texte du bonus de fin : `(morceau, gagné ?)`, à poser côte à côte.
pub fn morceaux_bonus(bonus: &[LigneBonus]) -> Vec<(String, bool)> {
    let mut v = vec![(format!("BONUS +{}", total_bonus(bonus)), true)];
    v.extend(
        bonus
            .iter()
            .map(|l| (format!("{} +{}", l.nom, l.gouttes), l.gagne)),
    );
    v
}

/// Bande sous l'écran de fin (y 169 à 180, sur presque toute la largeur : une seule
/// ligne, la place manque) : elle alterne entre le bonus de fin de partie et les
/// défis du dernier niveau (s'il y en a).
pub fn dessiner_bande_fin(dernier: Option<&ResultatNiveau>, bonus: &[LigneBonus], anim: u32) {
    panneau(4.0, 169.0, 312.0, 11.0, ARDOISE_NUIT, ARDOISE_CLAIRE);
    match dernier {
        Some(r) if (anim / PAGE_FIN) % 2 == 1 => ligne_defis(r, 171.0, true),
        _ => {
            let morceaux = morceaux_bonus(bonus);
            let largeur: f32 = morceaux
                .iter()
                .map(|(t, _)| largeur_texte(t, 1) as f32)
                .sum::<f32>()
                + ECART_DEFI * (morceaux.len() - 1) as f32;
            let mut x = ((LARGEUR_LOGIQUE as f32 - largeur) / 2.0).round();
            for (t, gagne) in &morceaux {
                let couleur = if *gagne { OR_PALE } else { ARDOISE_PALE };
                texte::dessiner(t, x, 171.0, ombre(couleur));
                x += largeur_texte(t, 1) as f32 + ECART_DEFI;
            }
        }
    }
}

/// Largeur maximale d'une ligne de bande, à l'intérieur du liseré.
#[cfg(test)]
pub const LARGEUR_BANDE_FIN: f32 = 310.0;

/// Largeurs des deux pages de la bande de fin, au pire (pour les tests de mise en page).
#[cfg(test)]
pub fn largeurs_pire_cas(r: &ResultatNiveau, bonus: &[LigneBonus]) -> (f32, f32) {
    let m = morceaux_bonus(bonus);
    let b = m
        .iter()
        .map(|(t, _)| largeur_texte(t, 1) as f32)
        .sum::<f32>()
        + ECART_DEFI * (m.len() - 1) as f32;
    (largeur_ligne(r, true), b)
}
