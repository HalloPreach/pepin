//! Scène : dessine un `Monde` dans la cible 320x180, couche par couche.
//!
//! Ordre (arrière -> avant) : fond (ciel, silhouettes) -> tuiles -> panneaux ->
//! plateformes -> objets -> ennemis -> Pépin -> particules -> pluie -> interface.
//! Chaque sprite est posé à une position ENTIÈRE (positions interpolées puis
//! arrondies) : aucun sous-pixel.

use macroquad::prelude::draw_rectangle;
use pepin_core::aabb::{Aabb, TAILLE_TUILE};
use pepin_core::entities::ennemi::EtatEnnemi;
use pepin_core::entities::friable::EtatFriable;
use pepin_core::entities::{Entite, TypeEnnemi};
use pepin_core::level::{GenrePanneau, Tuile};
use pepin_core::palette::*;
use pepin_core::player::EtatJoueur;
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use crate::decor;
use crate::monde::{Monde, REAPPARITION_PAS};
use crate::particules::Particules;
use crate::rendu::couleur_mq;
use crate::sprites::{ennemis, objets, pepin, Banque};
use crate::texte::{self, Style};

fn rect(x: i32, y: i32, w: i32, h: i32, c: Couleur) {
    draw_rectangle(x as f32, y as f32, w as f32, h as f32, couleur_mq(c));
}

/// Image d'une animation : `pas` / `duree` modulo le nombre d'images.
fn image<'a>(anim: &'a [&'a str], pas: u32, duree: u32) -> &'a str {
    anim[(pas / duree) as usize % anim.len()]
}

/// Arrondit une boîte interpolée au pixel : `(x, y)` entiers.
fn entier(b: &Aabb) -> (i32, i32) {
    (b.x.round() as i32, b.y.round() as i32)
}

/// Nom de la tuile à dessiner : version « haut » si rien de solide au-dessus.
fn nom_tuile(monde: &Monde, tx: i32, ty: i32, t: Tuile) -> &'static str {
    let haut = monde.niveau.tuile_decor(tx, ty - 1) == Tuile::Vide;
    match (t, haut) {
        (Tuile::Toit, true) => "toit_haut",
        (Tuile::Toit, false) => "toit_plein",
        (Tuile::Brique, true) => "brique_haut",
        (Tuile::Brique, false) => "brique_plein",
        (Tuile::Nuage, true) => "nuage_haut",
        (Tuile::Nuage, false) => "nuage_plein",
        (Tuile::Vide, _) => "",
    }
}

fn dessiner_tuiles(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    let taille = TAILLE_TUILE as i32;
    let (t0, t1) = (cx.div_euclid(taille), (cx + LARGEUR_LOGIQUE as i32).div_euclid(taille));
    let (u0, u1) = (cy.div_euclid(taille), (cy + HAUTEUR_LOGIQUE as i32).div_euclid(taille));
    for ty in u0..=u1 {
        for tx in t0..=t1 {
            // Hors carte : `tuile_decor` renvoie des murs à gauche/droite ; on ne les dessine pas.
            if tx < 0 || tx >= monde.niveau.largeur() as i32 {
                continue;
            }
            let t = monde.niveau.tuile_decor(tx, ty);
            if t != Tuile::Vide {
                let nom = nom_tuile(monde, tx, ty, t);
                banque.dessiner_tuile(monde.infos.theme, nom, (tx * taille - cx) as f32, (ty * taille - cy) as f32);
            }
        }
    }
}

fn dessiner_panneaux(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    for p in &monde.niveau.panneaux {
        let nom = match p.genre {
            GenrePanneau::Deplacer => "panneau_deplacer",
            GenrePanneau::Sauter => "panneau_sauter",
            GenrePanneau::Planer => "panneau_planer",
            GenrePanneau::Champignon => "panneau_champignon",
            GenrePanneau::Vent => "panneau_vent",
        };
        banque.dessiner(nom, p.x.round() - cx as f32, p.y.round() - cy as f32, false);
    }
}

/// Éclair : une colonne annoncée par des pointillés, puis un zigzag éblouissant.
fn dessiner_eclair(zone: &Aabb, annonce: bool, frappe: bool, pas: u32, cx: i32, cy: i32) {
    let (x0, y0) = (zone.x.round() as i32 - cx, zone.y.round() as i32 - cy);
    let (w, h) = (zone.w.round() as i32, zone.h.round() as i32);
    let milieu = x0 + w / 2;
    if annonce && (pas / 4) % 2 == 0 {
        for y in (y0..y0 + h).step_by(4) {
            rect(milieu, y, 1, 2, OR);
        }
    }
    if frappe {
        rect(x0, y0, w, h, OR_PALE);
        const ZIGZAG: [i32; 6] = [0, 2, -1, 2, -2, 1];
        for (k, y) in (y0..y0 + h).step_by(4).enumerate() {
            let dx = ZIGZAG[k % ZIGZAG.len()];
            rect(milieu - 1 + dx, y, 3, 4, BLANC_CASSE);
        }
    }
}

fn dessiner_entites(monde: &Monde, banque: &Banque, alpha: f32, cx: i32, cy: i32) {
    let pas = monde.pas;
    for (i, entite) in monde.entites.iter().enumerate() {
        let b = monde.boite_interpolee(i, alpha);
        let (x, y) = entier(&b);
        let (sx, sy) = ((x - cx) as f32, (y - cy) as f32);
        match entite {
            Entite::Goutte(g) => {
                if !g.est_ramassee() {
                    // Petit flottement d'un pixel, en 4 temps.
                    let flotte = [0.0, -1.0, -1.0, 0.0][(pas / 15) as usize % 4];
                    banque.dessiner(image(&objets::GOUTTE, pas, 8), sx, sy + flotte, false);
                }
            }
            Entite::Lanterne(l) => {
                let nom = if l.est_active() { objets::LANTERNE[1] } else { objets::LANTERNE[0] };
                banque.dessiner(nom, sx, sy, false);
            }
            Entite::Champignon(c) => {
                // Sprite 16x12 : le chapeau commence à la hauteur de la boîte.
                let nom = if c.est_comprime() { objets::CHAMPIGNON[1] } else { objets::CHAMPIGNON[0] };
                banque.dessiner(nom, sx, sy - 3.0, false);
            }
            Entite::Pointe(p) => {
                // La boîte fait 16 px de large : deux pointes de 8 px.
                let n = (p.boite.w / 8.0) as i32;
                for k in 0..n {
                    banque.dessiner("pointe", sx + (k * 8) as f32, sy, false);
                }
            }
            Entite::Friable(f) => {
                let tremble = match f.etat {
                    EtatFriable::Effritement(_) => [0.0, 1.0, 0.0, -1.0][(pas / 2) as usize % 4],
                    EtatFriable::Absente(_) => continue,
                    EtatFriable::Stable => 0.0,
                };
                let nom = if matches!(f.etat, EtatFriable::Effritement(_)) { objets::FRIABLE[1] } else { objets::FRIABLE[0] };
                for k in 0..(f.boite.w / 16.0).ceil() as i32 {
                    banque.dessiner(nom, sx + (k * 16) as f32 + tremble, sy, false);
                }
            }
            Entite::Mobile(_) => banque.dessiner("mobile", sx, sy, false),
            Entite::Eclair(e) => dessiner_eclair(&e.zone, e.en_annonce(), e.frappe(), pas, cx, cy),
            Entite::Ennemi(e) => {
                if !e.est_visible() {
                    continue;
                }
                let ecrase = matches!(e.etat, EtatEnnemi::Ecrase(_));
                match e.type_ennemi {
                    TypeEnnemi::Flaque => {
                        let nom = if ecrase { "flaque_ecrasee" } else { image(&ennemis::FLAQUE, pas, 12) };
                        // Sprite 16x10 posé sur le bas de la boîte 12x8, centré.
                        banque.dessiner(nom, sx - 2.0, sy - 2.0, e.vers_la_droite);
                    }
                    TypeEnnemi::Corbeau => {
                        // Le corbeau du dessin regarde à droite.
                        banque.dessiner(image(&ennemis::CORBEAU, pas, 10), sx - 2.0, sy - 1.0, !e.vers_la_droite);
                    }
                }
            }
        }
    }
}

fn dessiner_arrivee(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    let a = monde.niveau.arrivee;
    let x = (a.x + a.w / 2.0 - 8.0).round() as i32 - cx;
    let y = a.bas().round() as i32 - 32 - cy;
    banque.dessiner(image(&objets::DRAPEAU, monde.pas, 12), x as f32, y as f32, false);
}

fn dessiner_pepin(monde: &Monde, banque: &Banque, alpha: f32, cx: i32, cy: i32) {
    let pas = monde.pas;
    let j = &monde.joueur;
    let miroir = !j.regarde_droite;

    // Mort : trois images jouées à l'endroit de la mort, puis réapparition clignotante.
    if monde.reapparition > 0 {
        let ecoule = REAPPARITION_PAS - monde.reapparition;
        let k = (ecoule as usize * 3 / REAPPARITION_PAS as usize).min(2);
        let (mx, my) = (monde.pos_mort.0.round() as i32 - cx - 3, monde.pos_mort.1.round() as i32 - cy - 2);
        banque.dessiner(pepin::REAPPARITION[k], mx as f32, my as f32, false);
        if (pas / 3) % 2 == 0 {
            return; // clignotement de Pépin tout juste réapparu
        }
    }

    let (px, py) = monde.joueur_interpole(alpha);
    let (bx, by) = (px.round() as i32 - cx, py.round() as i32 - cy);
    let (bas, milieu) = (by + j.boite.h as i32, bx + j.boite.w as i32 / 2);
    match j.etat() {
        EtatJoueur::Plane => {
            let nom = image(&pepin::PLANER, pas, 8);
            banque.dessiner(nom, (milieu - 10) as f32, (bas - 20) as f32, miroir);
        }
        etat => {
            let nom = match etat {
                EtatJoueur::Montee => "pepin_saut",
                EtatJoueur::Chute => "pepin_chute",
                _ if j.vitesse.x.abs() > 8.0 => image(&pepin::COURSE, pas, 5),
                _ => image(&pepin::REPOS, pas, 30),
            };
            banque.dessiner(nom, (milieu - 8) as f32, (bas - 16) as f32, miroir);
        }
    }
}

/// Largeur maximale d'une ligne de texte d'introduction, en caractères
/// (320 px / 6 px par caractère = 53 ; on garde une marge).
const LIGNE_MAX: usize = 44;

/// Coupe `texte` en lignes de `max` caractères au plus, en ne coupant qu'entre les mots.
/// Un mot plus long que `max` reste seul sur sa ligne.
pub fn couper_lignes(texte: &str, max: usize) -> Vec<String> {
    let mut lignes: Vec<String> = Vec::new();
    for mot in texte.split_whitespace() {
        match lignes.last_mut() {
            Some(ligne) if ligne.chars().count() + 1 + mot.chars().count() <= max => {
                ligne.push(' ');
                ligne.push_str(mot);
            }
            _ => lignes.push(mot.to_string()),
        }
    }
    lignes
}

/// Interface : gouttes d'or ramassées et nom du niveau au début.
fn dessiner_interface(monde: &Monde, banque: &Banque) {
    let style = Style::simple(BLANC_CASSE, 1).avec_ombre(ARDOISE_OMBRE);
    banque.dessiner("goutte0", 6.0, 5.0, false);
    texte::dessiner(&format!("{}", monde.etat.gouttes), 17.0, 6.0, style);
    if monde.pas < 240 {
        let titre = format!("{} — {}", monde.infos.numero, monde.infos.nom);
        texte::dessiner_centre(&titre, 20.0, Style::simple(OR_PALE, 1).avec_ombre(ARDOISE_OMBRE));
        for (i, ligne) in couper_lignes(monde.infos.intro, LIGNE_MAX).iter().enumerate() {
            texte::dessiner_centre(ligne, 32.0 + i as f32 * 10.0, style);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pepin_core::zones::NIVEAUX;

    #[test]
    fn le_texte_est_coupe_entre_les_mots() {
        assert_eq!(couper_lignes("un deux trois", 7), vec!["un deux", "trois"]);
        assert_eq!(couper_lignes("", 10), Vec::<String>::new());
        assert_eq!(couper_lignes("immensément", 4), vec!["immensément"]);
    }

    #[test]
    fn toutes_les_introductions_tiennent_dans_l_ecran() {
        for n in NIVEAUX {
            let lignes = couper_lignes(n.intro, LIGNE_MAX);
            assert!(lignes.len() <= 4, "{} : {} lignes", n.nom, lignes.len());
            for l in lignes {
                let largeur = pepin_core::font::largeur_texte(&l, 1);
                assert!(largeur < LARGEUR_LOGIQUE - 8, "{} : {largeur} px", n.nom);
            }
        }
    }
}

/// Dessine tout le monde dans la cible logique. `(cx, cy)` : caméra entière.
pub fn dessiner(monde: &Monde, banque: &Banque, particules: &Particules, alpha: f32, cx: i32, cy: i32) {
    decor::dessiner_fond(monde.infos.theme, cx, cy, monde.pas);
    dessiner_tuiles(monde, banque, cx, cy);
    dessiner_panneaux(monde, banque, cx, cy);
    dessiner_arrivee(monde, banque, cx, cy);
    dessiner_entites(monde, banque, alpha, cx, cy);
    dessiner_pepin(monde, banque, alpha, cx, cy);
    particules.dessiner(cx, cy);
    decor::dessiner_avant(monde.infos.theme, monde.pas);
    dessiner_interface(monde, banque);
}
