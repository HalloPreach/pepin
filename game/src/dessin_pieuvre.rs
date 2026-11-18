//! Rendu de la Pieuvre du port, boss du Monde 2 (carte 9.12a) : ses tentacules
//! (annonce, dressés, qui redescendent), sa tête (qui remonte, sortie, touchée,
//! qui replonge en lâchant une clé) et l'encre quand elle a eu Pépin.
//!
//! La règle du jeu est visible : AUCUN tentacule ne se dresse sans son annonce. La
//! 1re moitié de l'annonce (0,4 s) : l'ombre de la pieuvre sous la surface et des
//! remous sur toute la largeur de la colonne ; la 2e moitié : le bout du tentacule
//! pointe hors de l'eau. Tout est en sprites de la palette (`sprites::pieuvre`).
//!
//! La saisie (carte 9.12f) : un tentacule monte de l'eau et s'enroule sur la
//! plateforme, qui tremble (l'annonce) ; elle s'enfonce, découpée à la surface ;
//! sous l'eau, des remous marquent sa place ; puis elle remonte.

use pepin_core::aabb::TAILLE_TUILE;
use pepin_core::entities::pieuvre::{
    EtatSaisie, EtatTentacule, PhasePieuvre, Pieuvre, LARGEUR_COLONNE, MARGE_TENTACULE, NB_COLONNES,
};
use pepin_core::entities::Entite;
use pepin_core::palette::ARDOISE_OMBRE;

use crate::monde::{Monde, ENCRE_PAS};
use crate::rendu::couleur_mq;
use crate::sprites::Banque;
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

/// Hauteur (px) d'un tronçon de tentacule (`tentacule0`).
const TRONCON: f32 = 8.0;
/// Hauteur (px) du bout recourbé (`tentacule_bout`).
const BOUT: f32 = 16.0;

/// Image d'une animation à deux images de `duree` pas chacune.
fn image(anim: [&'static str; 2], pas: u32, duree: u32) -> &'static str {
    anim[(pas / duree) as usize % 2]
}

/// Hauteur (px) du tentacule hors de l'eau, de 0 à `pleine` : rien au calme ni
/// pendant l'annonce (le bout qui pointe a son sprite), tout dressé, puis il
/// redescend en 12 pas. Pure.
fn hauteur_tentacule(etat: EtatTentacule, pleine: f32) -> f32 {
    match etat {
        EtatTentacule::Dresse => pleine,
        EtatTentacule::Redescente(p) => (pleine * (1.0 - p)).round(),
        EtatTentacule::Calme | EtatTentacule::Annonce(_) => 0.0,
    }
}

/// Dessine la pieuvre `p` (caméra `cx`, `cy`), au pas `pas` du monde.
pub fn dessiner(p: &Pieuvre, banque: &Banque, pas: u32, cx: i32, cy: i32) {
    let (cx, cy) = (cx as f32, cy as f32);
    for i in 0..NB_COLONNES {
        let corps = p.colonnes[i];
        let surface = corps.bas() - cy;
        let gauche = corps.x - MARGE_TENTACULE - cx;
        match p.etat_tentacule(i) {
            EtatTentacule::Calme => {}
            EtatTentacule::Annonce(a) => {
                banque.dessiner("ombre_pieuvre", gauche, surface + 1.0, false);
                banque.dessiner(
                    image(["remous_pieuvre0", "remous_pieuvre1"], pas, 6),
                    gauche,
                    surface - 5.0,
                    false,
                );
                if a >= 0.5 {
                    let nom = image(["tentacule_pointe0", "tentacule_pointe1"], pas, 8);
                    banque.dessiner(nom, corps.x - cx, surface - 11.0, false);
                }
            }
            etat => {
                let h = hauteur_tentacule(etat, corps.h);
                dessiner_tentacule(banque, corps.x - cx, surface, h, pas);
                banque.dessiner(
                    image(["remous_pieuvre0", "remous_pieuvre1"], pas, 6),
                    gauche,
                    surface - 5.0,
                    false,
                );
            }
        }
    }
    dessiner_tete(p, banque, pas, cx, cy);
}

/// Un tentacule de `h` px dressé au-dessus de `surface` (à l'écran) : des tronçons
/// empilés depuis l'eau, puis le bout recourbé en haut (dessiné en dernier, il couvre
/// le tronçon qui dépasse).
fn dessiner_tentacule(banque: &Banque, x: f32, surface: f32, h: f32, pas: u32) {
    if h <= 0.0 {
        return;
    }
    if h < BOUT {
        // Il finit de replonger : seul le haut du bout dépasse encore.
        banque.dessiner_haut("tentacule_bout", x, surface - h, h);
        return;
    }
    let haut = surface - h;
    let n = ((h - BOUT) / TRONCON).ceil() as u32;
    for k in 1..=n {
        // Les ventouses glissent d'un tronçon à l'autre : le tentacule ondule.
        let nom = image(["tentacule0", "tentacule1"], pas / 4 + k, 1);
        banque.dessiner(nom, x, surface - TRONCON * k as f32, false);
    }
    banque.dessiner("tentacule_bout", x, haut, false);
}

/// La tête : remous à sa place quand elle remonte, puis la tête posée sur l'eau ;
/// touchée, elle clignote ; vaincue, elle replonge et la clé monte de l'eau.
fn dessiner_tete(p: &Pieuvre, banque: &Banque, pas: u32, cx: f32, cy: f32) {
    let Some(tete) = p.tete() else {
        return;
    };
    let (x, y) = (tete.x - cx, tete.y - cy);
    let surface = tete.bas() - cy;
    let remous = image(["remous_pieuvre0", "remous_pieuvre1"], pas, 6);
    match p.phase() {
        PhasePieuvre::Remous => {
            banque.dessiner("ombre_pieuvre", x + 8.0, surface + 1.0, false);
            for dx in [0.0, LARGEUR_COLONNE / 2.0] {
                banque.dessiner(remous, x + dx, surface - 5.0, false);
            }
        }
        PhasePieuvre::Tete => {
            let nom = image(["pieuvre_colere0", "pieuvre_colere1"], pas, 20);
            if p.est_facile() {
                banque.dessiner_pale(nom, x, y);
            } else {
                banque.dessiner(nom, x, y, false);
            }
        }
        // Touchée : 4 images visibles, 4 cachées, comme le Grand Nuage.
        PhasePieuvre::Touchee if (p.ecoule() / 4).is_multiple_of(2) => {
            banque.dessiner("pieuvre_touchee", x, y, false);
        }
        PhasePieuvre::Plongee => {
            let k = p.progression_plongee();
            let descente = (tete.h * k).round();
            banque.dessiner_haut("pieuvre_vaincue", x, y + descente, tete.h - descente);
            for dx in [0.0, LARGEUR_COLONNE / 2.0] {
                banque.dessiner(remous, x + dx, surface - 5.0, false);
            }
            // La clé sort de l'eau et monte de 40 px au-dessus d'elle.
            let montee = (40.0 * (k * 2.0).min(1.0)).round();
            banque.dessiner("cle_pieuvre", x + 17.0, surface - 8.0 - montee, false);
        }
        _ => {}
    }
}

/// L'encre de la Pieuvre (carte 9.12a) : l'écran s'assombrit brièvement quand elle a
/// eu Pépin. Sans transparence (palette fixe) : des rangées d'ardoise sombre, une
/// sur quatre au début et à la fin, une sur deux au milieu.
pub fn dessiner_encre(monde: &Monde) {
    if monde.encre == 0 {
        return;
    }
    let ecoule = ENCRE_PAS - monde.encre;
    let pas_rangee = if ecoule < ENCRE_PAS / 3 || monde.encre < ENCRE_PAS / 3 {
        4
    } else {
        2
    };
    let c = couleur_mq(ARDOISE_OMBRE);
    for y in (0..HAUTEUR_LOGIQUE).step_by(pas_rangee) {
        macroquad::prelude::draw_rectangle(0.0, y as f32, LARGEUR_LOGIQUE as f32, 1.0, c);
    }
}

/// La pieuvre du niveau, s'il y en a une.
fn la_pieuvre(monde: &Monde) -> Option<&Pieuvre> {
    monde.entites.iter().find_map(|e| match e {
        Entite::Pieuvre(p) => Some(p.as_ref()),
        _ => None,
    })
}

/// La case `(tx, ty)` appartient-elle à la plateforme saisie en ce moment ? Le
/// jeu de tuiles ne la dessine pas : `dessiner_saisie` s'en charge.
pub fn case_saisie(monde: &Monde, tx: i32, ty: i32) -> bool {
    la_pieuvre(monde)
        .and_then(|p| {
            p.saisie()
                .map(|s| p.plateformes[s.cible].contains(&(tx, ty)))
        })
        .unwrap_or(false)
}

/// Enfoncement (px) de la plateforme saisie sous sa place, de 0 à `fond` (elle
/// est alors tout entière sous la surface), et son tremblement (px). Pure.
fn enfoncement(etat: EtatSaisie, fond: f32, pas: u32) -> (f32, f32) {
    match etat {
        // Le tremblement grandit avec l'annonce : 1 px, puis 2 px à la fin.
        EtatSaisie::Annonce(a) => {
            let ampleur = if a < 0.5 { 1.0 } else { 2.0 };
            (0.0, [0.0, ampleur, 0.0, -ampleur][(pas / 2 % 4) as usize])
        }
        EtatSaisie::Plongee(p) => ((fond * p).round(), 0.0),
        EtatSaisie::SousLEau => (fond, 0.0),
        EtatSaisie::Remontee(p) => ((fond * (1.0 - p)).round(), 0.0),
    }
}

/// La plateforme que la Pieuvre emporte (carte 9.12f), après l'eau du bassin :
/// ses tuiles (qui tremblent, puis coulent, découpées à la surface), le
/// tentacule qui monte de l'eau et s'enroule sur son dessus, les remous.
pub fn dessiner_saisie(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    let Some(p) = la_pieuvre(monde) else {
        return;
    };
    let Some(saisie) = p.saisie() else {
        return;
    };
    let t = TAILLE_TUILE;
    let (cx, cy) = (cx as f32, cy as f32);
    let cases = &p.plateformes[saisie.cible];
    // La surface du bassin : le bas des colonnes de tentacules.
    let surface = p.colonnes[0].bas();
    let haut = cases.iter().map(|c| c.1).min().unwrap_or(0) as f32 * t;
    let (g, d) = (
        cases.iter().map(|c| c.0).min().unwrap_or(0) as f32 * t,
        (cases.iter().map(|c| c.0).max().unwrap_or(0) + 1) as f32 * t,
    );
    let etat = saisie.etat();
    let (dy, dx) = enfoncement(etat, surface - haut, monde.pas);
    // Les tuiles, et l'objet posé dessus : seulement ce qui dépasse de l'eau.
    for &(tx, ty) in cases {
        let (nom, objet) = crate::scene::noms_de_la_case(monde, tx, ty);
        let (x, y) = (tx as f32 * t + dx, ty as f32 * t + dy);
        let reste = surface - y;
        banque.dessiner_tuile_haut(monde.infos.theme, nom, x - cx, y - cy, reste);
        if let Some(objet) = objet {
            banque.dessiner_tuile_haut(monde.infos.theme, objet, x - cx, y - t - cy, reste + t);
        }
    }
    // Le tentacule : il sort au milieu de la plateforme, monte pendant la 1re
    // moitié de l'annonce, s'enroule sur le dessus ensuite et l'entraîne.
    let milieu = ((g + d) / 2.0 - 12.0).round();
    let remous = image(["remous_pieuvre0", "remous_pieuvre1"], monde.pas, 6);
    let tirage = match etat {
        EtatSaisie::Annonce(a) => (a * 2.0).min(1.0),
        EtatSaisie::Plongee(_) => 1.0,
        _ => 0.0,
    };
    if tirage > 0.0 {
        let plein = surface - haut + 4.0;
        let h = (plein * tirage).round() - dy;
        dessiner_tentacule(banque, milieu - cx, surface - cy, h, monde.pas);
        if tirage >= 1.0 {
            let nom = image(["tentacule_enroule0", "tentacule_enroule1"], monde.pas, 8);
            let mut x = g;
            while x < d {
                let y = haut - 3.0 + dy;
                banque.dessiner_haut(nom, x + dx - cx, y - cy, surface - y);
                x += t;
            }
        }
        banque.dessiner(
            "ombre_pieuvre",
            milieu - 4.0 - cx,
            surface + 1.0 - cy,
            false,
        );
    }
    // Des remous sur toute la largeur de la plateforme, de la plongée à la
    // remontée (sa place sous l'eau), et au pied du tentacule pendant l'annonce.
    let (r0, r1) = if matches!(etat, EtatSaisie::Annonce(_)) {
        (milieu - 4.0, milieu + 28.0)
    } else {
        (g, d)
    };
    let mut x = r0;
    while x < r1 {
        banque.dessiner(remous, x - cx, surface - 5.0 - cy, false);
        x += LARGEUR_COLONNE;
    }
}
