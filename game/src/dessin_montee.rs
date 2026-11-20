//! Rendu de la montée de la Pieuvre (carte 9.13b, AGA : « la montée est trop
//! simple et trop courte, il faudrait des interactions avec la pieuvre ») :
//! - la TRAPPE d'une rencontre : des tentacules enroulés tendus sur le trou ;
//! - la tête qui SURGIT de la corniche, puis qui replonge, lâchée ;
//! - les deux tentacules de GARDE : le bout qui pointe au ras de la corniche
//!   (l'annonce), puis le tentacule dressé depuis la mer ;
//! - le BALAYEUR : remous au pied du mur, le bout qui sort du mur, puis le long
//!   tentacule couché qui glisse sur la corniche ;
//! - l'ARRACHAGE : l'eau de la saisie (carte 9.12g) à la surface de la mer, le
//!   tentacule enroulé sur la plateforme qui tremble, puis qui l'emporte.
//!
//! Que des sprites existants de la Pieuvre (`sprites::pieuvre`) : aucune couleur
//! ajoutée.

use pepin_core::aabb::TAILLE_TUILE;
use pepin_core::entities::pieuvre::{EtatArrachage, EtatBalai, EtatSaisie, EtatTentacule, Pieuvre};

use crate::dessin_pieuvre::{dessiner_eau_saisie, dessiner_tentacule, la_pieuvre};
use crate::monde::Monde;
use crate::sprites::Banque;

/// Image d'une animation à deux images de `duree` pas chacune.
fn image(anim: [&'static str; 2], pas: u32, duree: u32) -> &'static str {
    anim[(pas / duree) as usize % 2]
}

/// Les cases arrachables qui ne se dessinent plus comme des tuiles (le jeu de
/// tuiles les saute : `dessiner` les dessine, tremblantes ou emportées).
pub fn case_arrachee(p: &Pieuvre, tx: i32, ty: i32) -> bool {
    (0..p.tour.arrachables.len())
        .any(|k| p.etat_arrachage(k).is_some() && p.tour.arrachables[k].contains(&(tx, ty)))
}

/// Tout ce que la montée ajoute au dessin de la pieuvre du niveau (caméra `cx`,
/// `cy`), devant les tuiles.
pub fn dessiner(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    let Some(p) = la_pieuvre(monde).filter(|p| p.en_fuite()) else {
        return;
    };
    let (cx, cy) = (cx as f32, cy as f32);
    let pas = monde.pas;
    let enroule = image(["tentacule_enroule0", "tentacule_enroule1"], pas, 8);
    let remous = image(["remous_pieuvre0", "remous_pieuvre1"], pas, 6);
    let mer = p.surface_mer();
    // La trappe de la rencontre suivante, fermée : les tentacules tendus.
    if let Some(trappe) = p.solide() {
        let mut x = trappe.x;
        while x < trappe.droite() {
            banque.dessiner(enroule, x - cx, trappe.y + 5.0 - cy, false);
            x += TAILLE_TUILE;
        }
    }
    // La tête qui surgit de la corniche (elle monte, coupée au ras du sol).
    if let (Some(a), Some(r)) = (p.surgissement(), p.rencontre()) {
        let monte = (r.tete.h * a).round();
        let y = r.tete.bas() - monte;
        let nom = image(["pieuvre_colere0", "pieuvre_colere1"], pas, 4);
        banque.dessiner_haut(nom, r.tete.x - cx, y - cy, monte);
        banque.dessiner(remous, r.tete.x + 12.0 - cx, r.tete.bas() - 5.0 - cy, false);
    }
    // La tête lâchée : touchée, elle hurle et redescend sous la corniche.
    if let Some((tete, a)) = p.tete_lachee() {
        let descente = (tete.h * a).round();
        let nom = if (pas / 4).is_multiple_of(2) {
            "pieuvre_touchee"
        } else {
            "pieuvre_colere0"
        };
        banque.dessiner_haut(nom, tete.x - cx, tete.y + descente - cy, tete.h - descente);
    }
    // Le tentacule de garde : la pointe qui sort au ras de la corniche, puis
    // tout le tentacule, dressé depuis la mer.
    if let (Some((corps, etat)), Some(mer)) = (p.garde_en_cours(), mer) {
        let sol = corps.y + 4.0 * TAILLE_TUILE;
        match etat {
            EtatTentacule::Annonce(a) if a >= 0.5 => {
                let nom = image(["tentacule_pointe0", "tentacule_pointe1"], pas, 8);
                banque.dessiner(nom, corps.x - cx, sol - 11.0 - cy, false);
            }
            EtatTentacule::Annonce(_) => {
                banque.dessiner(remous, corps.x - 4.0 - cx, sol - 5.0 - cy, false);
            }
            etat => {
                let plein = mer - corps.y;
                let h = match etat {
                    EtatTentacule::Redescente(r) => (plein * (1.0 - r)).round(),
                    _ => plein,
                };
                dessiner_tentacule(banque, corps.x - cx, mer - cy, h, pas);
            }
        }
    }
    // Les balayeurs.
    for (k, b) in p.tour.balayeurs.iter().enumerate() {
        let Some(etat) = p.etat_balayeur(k) else {
            continue;
        };
        let pointe = image(["tentacule_pointe0", "tentacule_pointe1"], pas, 8);
        // Le mur d'où il sort, et l'avancée du bout.
        let (mur, bout_x) = match etat {
            EtatBalai::Annonce(a) => {
                if let Some(mer) = mer.filter(|_| a < 0.5) {
                    let x = if b.de_gauche() {
                        b.depart
                    } else {
                        b.depart - 24.0
                    };
                    banque.dessiner(remous, x - cx, mer - 5.0 - cy, false);
                    continue;
                }
                let sort = 8.0 * (a - 0.5) * 2.0;
                let x = if b.de_gauche() {
                    b.depart - 24.0 + sort + 8.0
                } else {
                    b.depart - sort - 8.0
                };
                (b.depart, x)
            }
            EtatBalai::Balaie(bout) => (b.depart, bout.x),
            EtatBalai::Retrait(r) => {
                let fin = b.bout(b.course_pas());
                let x = if b.de_gauche() {
                    fin.x + (b.depart - fin.x) * r
                } else {
                    fin.x + (b.depart - fin.droite()) * r
                };
                (b.depart, x)
            }
        };
        // Le corps couché, du mur au bout, au ras de la corniche.
        let y = b.y - 7.0;
        let (g, d) = if b.de_gauche() {
            (mur, bout_x)
        } else {
            (bout_x + 24.0, mur)
        };
        let mut x = g;
        while x < d {
            banque.dessiner_haut(enroule, x - cx, y - cy, 6.0);
            x += 16.0;
        }
        banque.dessiner(pointe, bout_x - cx, b.y - 12.0 - cy, !b.de_gauche());
    }
    // Les arrachages.
    for (k, cases) in p.tour.arrachables.iter().enumerate() {
        let Some(etat) = p.etat_arrachage(k) else {
            continue;
        };
        let t = TAILLE_TUILE;
        let haut = cases.iter().map(|c| c.1).min().unwrap_or(0) as f32 * t;
        let g = cases.iter().map(|c| c.0).min().unwrap_or(0) as f32 * t;
        let d = (cases.iter().map(|c| c.0).max().unwrap_or(0) + 1) as f32 * t;
        let (dx, dy, saisie) = match etat {
            EtatArrachage::Annonce(a) => {
                let ampleur = if a < 0.5 { 1.0 } else { 2.0 };
                let dx = [0.0, ampleur, 0.0, -ampleur][(pas / 2 % 4) as usize];
                (dx, 0.0, EtatSaisie::Annonce(a))
            }
            // Emportée : elle tombe vers la mer en 0,5 s, coupée à la surface.
            EtatArrachage::Emportee(e) => {
                let fond = mer.map_or(5.0 * t, |m| m - haut);
                (0.0, (fond * e * e).round(), EtatSaisie::Plongee(e))
            }
            EtatArrachage::Partie => continue,
        };
        if let Some(mer) = mer {
            dessiner_eau_saisie(
                banque,
                saisie,
                (g, d),
                (t, monde.niveau.largeur_px() - t),
                (-cx, mer - cy),
                pas,
            );
        }
        let reste = mer.map_or(t, |m| m - haut - dy).max(0.0);
        for &(tx, ty) in cases {
            let (nom, _) = crate::scene::noms_de_la_case(monde, tx, ty);
            let (x, y) = (tx as f32 * t + dx, ty as f32 * t + dy);
            banque.dessiner_tuile_haut(monde.infos.theme, nom, x - cx, y - cy, reste.min(t));
        }
        // Le tentacule enroulé sur le dessus (à la 2e moitié de l'annonce).
        if !matches!(etat, EtatArrachage::Annonce(a) if a < 0.5) {
            let mut x = g;
            while x < d {
                banque.dessiner_haut(enroule, x + dx - cx, haut - 3.0 + dy - cy, reste.min(6.0));
                x += t;
            }
        }
    }
}
