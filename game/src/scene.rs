//! Scène : dessine un `Monde` dans la cible 320x180, couche par couche.
//!
//! Ordre (arrière -> avant) : fond (ciel, silhouettes) -> tuiles -> panneaux ->
//! plateformes -> objets -> ennemis -> Pépin -> particules -> pluie.
//! L'interface (compteurs, chrono) est dessinée par `hud`, les menus par `ecrans`.
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
use pepin_core::zones::Theme;
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use crate::decor;
use crate::monde::{Monde, REAPPARITION_PAS};
use crate::particules::Particules;
use crate::rendu::couleur_mq;
use crate::sprites::{ennemis, objets, pepin, Banque};

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

/// Nom de la tuile à dessiner : version « haut » si rien de solide au-dessus ; un nuage dont la
/// case voisine est vide (à gauche et/ou à droite) prend un bord arrondi. Dessin seulement :
/// la collision reste celle de la case.
fn nom_tuile(monde: &Monde, tx: i32, ty: i32, t: Tuile) -> &'static str {
    let haut = monde.niveau.tuile_decor(tx, ty - 1) == Tuile::Vide;
    if t == Tuile::Nuage && haut {
        let (gauche, droite) = (
            monde.niveau.tuile_decor(tx - 1, ty) == Tuile::Vide,
            monde.niveau.tuile_decor(tx + 1, ty) == Tuile::Vide,
        );
        match (gauche, droite) {
            (true, true) => return "nuage_ilot",
            (true, false) => return "nuage_bord_g",
            (false, true) => return "nuage_bord_d",
            (false, false) => {}
        }
    }
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
    let (t0, t1) = (
        cx.div_euclid(taille),
        (cx + LARGEUR_LOGIQUE as i32).div_euclid(taille),
    );
    let (u0, u1) = (
        cy.div_euclid(taille),
        (cy + HAUTEUR_LOGIQUE as i32).div_euclid(taille),
    );
    for ty in u0..=u1 {
        for tx in t0..=t1 {
            // Hors carte : `tuile_decor` renvoie des murs à gauche/droite ; on ne les dessine pas.
            if tx < 0 || tx >= monde.niveau.largeur() as i32 {
                continue;
            }
            let t = monde.niveau.tuile_decor(tx, ty);
            if t != Tuile::Vide {
                let nom = nom_tuile(monde, tx, ty, t);
                banque.dessiner_tuile(
                    monde.infos.theme,
                    nom,
                    (tx * taille - cx) as f32,
                    (ty * taille - cy) as f32,
                );
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

/// Annonce de l'éclair : le pointillé est-il allumé à cet instant ? `progression` va de 0.0
/// (début de l'annonce) à 1.0 (frappe imminente) : le clignotement ACCÉLÈRE, de 2,5 Hz au début
/// à 15 Hz à la fin (7 clignotements en 0,8 s). Fonction pure, testable.
fn annonce_allumee(progression: f32) -> bool {
    let p = progression.clamp(0.0, 1.0);
    let phase = 2.0 * p + 5.0 * p * p; // nombre de cycles écoulés
    phase.fract() < 0.5
}

/// Colonne de l'éclair à l'écran : `(y, hauteur)`. `zone_y`, `hauteur` : la zone dans le niveau ;
/// `cy` : ordonnée de la caméra. Quand la zone part du haut du niveau (y = 0) et que le niveau
/// est plus bas que l'écran, quelques lignes de ciel resteraient au-dessus : la colonne est alors
/// prolongée jusqu'au haut de l'écran. Dessin seulement : la zone dangereuse ne change pas.
fn colonne_eclair(zone_y: i32, hauteur: i32, cy: i32) -> (i32, i32) {
    let y0 = zone_y - cy;
    if zone_y <= 0 && y0 > 0 {
        (0, hauteur + y0)
    } else {
        (y0, hauteur)
    }
}

/// Éclair : une colonne annoncée par un pointillé clignotant sur ses deux bords et une barre
/// dorée à son pied, puis un zigzag éblouissant. `progression` : avancement de l'annonce (0 à 1).
fn dessiner_eclair(zone: &Aabb, annonce: bool, progression: f32, frappe: bool, cx: i32, cy: i32) {
    let x0 = zone.x.round() as i32 - cx;
    let (y0, h) = colonne_eclair(zone.y.round() as i32, zone.h.round() as i32, cy);
    let w = zone.w.round() as i32;
    let milieu = x0 + w / 2;
    if annonce {
        // Le pied de la colonne s'allume tout de suite et reste allumé : on voit OÙ ça va tomber.
        rect(x0, y0 + h - 2, w, 2, OR);
        if annonce_allumee(progression) {
            for y in (y0..y0 + h - 2).step_by(4) {
                rect(x0, y, 1, 2, OR);
                rect(x0 + w - 1, y, 1, 2, OR);
            }
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
                    // Petit flottement d'un pixel, en 4 temps. Le dessin (10x10) déborde d'un
                    // pixel de chaque côté de la boîte de ramassage (8x8).
                    let flotte = [0.0, -1.0, -1.0, 0.0][(pas / 15) as usize % 4];
                    banque.dessiner(
                        image(&objets::GOUTTE, pas, 8),
                        sx - 1.0,
                        sy + flotte - 1.0,
                        false,
                    );
                }
            }
            Entite::Lanterne(l) => {
                let nom = if l.est_active() {
                    objets::LANTERNE[1]
                } else {
                    objets::LANTERNE[0]
                };
                banque.dessiner(nom, sx, sy, false);
            }
            Entite::Champignon(c) => {
                // Sprite 16x12 : le chapeau commence à la hauteur de la boîte.
                let nom = if c.est_comprime() {
                    objets::CHAMPIGNON[1]
                } else {
                    objets::CHAMPIGNON[0]
                };
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
                // Dalle d'ardoise sur les toits (niveaux 1 à 3), nuage en altitude (4 et 5).
                let anim = match monde.infos.theme {
                    Theme::Altitude | Theme::Soleil => &objets::FRIABLE,
                    Theme::Toits | Theme::Gouttieres | Theme::Orage => &objets::FRIABLE_ARDOISE,
                };
                let nom = if matches!(f.etat, EtatFriable::Effritement(_)) {
                    anim[1]
                } else {
                    anim[0]
                };
                for k in 0..(f.boite.w / 16.0).ceil() as i32 {
                    banque.dessiner(nom, sx + (k * 16) as f32 + tremble, sy, false);
                }
            }
            Entite::Mobile(_) => banque.dessiner("mobile", sx, sy, false),
            Entite::Eclair(e) => dessiner_eclair(
                &e.zone,
                e.en_annonce(),
                e.progression_annonce(),
                e.frappe(),
                cx,
                cy,
            ),
            Entite::Ennemi(e) => {
                if !e.est_visible() {
                    continue;
                }
                let ecrase = matches!(e.etat, EtatEnnemi::Ecrase(_));
                match e.type_ennemi {
                    TypeEnnemi::Flaque => {
                        let nom = if ecrase {
                            "flaque_ecrasee"
                        } else {
                            image(&ennemis::FLAQUE, pas, 12)
                        };
                        // Sprite 16x10 posé sur le bas de la boîte 12x8, centré.
                        banque.dessiner(nom, sx - 2.0, sy - 2.0, e.vers_la_droite);
                    }
                    TypeEnnemi::Corbeau => {
                        // Le corbeau du dessin regarde à droite.
                        banque.dessiner(
                            image(&ennemis::CORBEAU, pas, 10),
                            sx - 2.0,
                            sy - 1.0,
                            !e.vers_la_droite,
                        );
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
    banque.dessiner(
        image(&objets::DRAPEAU, monde.pas, 12),
        x as f32,
        y as f32,
        false,
    );
}

/// Marge (px) gardée entre le bas de l'écran et l'animation de mort d'une chute.
const MARGE_MORT_BAS: f32 = 16.0;
/// Largeur (px) réservée à l'animation de mort pour qu'elle reste entière à l'écran.
const LARGEUR_MORT: f32 = 16.0;

/// Place l'animation de mort (et les éclats) dans le cadre. `pos` : où Pépin est mort, dans le
/// niveau (coin haut-gauche) ; `camera` : coin haut-gauche de l'écran dans le niveau. Une chute
/// dans un trou finit sous l'écran, et la caméra repart vers le point de reprise : sans cette
/// limite, l'animation se jouerait hors champ. L'ordonnée est bornée à 16 px au-dessus du bord
/// bas, l'abscisse à la largeur de l'écran. Fonction pure.
pub fn borner_mort(pos: (f32, f32), camera: (f32, f32)) -> (f32, f32) {
    let x_max = camera.0 + LARGEUR_LOGIQUE as f32 - LARGEUR_MORT;
    let y_max = camera.1 + HAUTEUR_LOGIQUE as f32 - MARGE_MORT_BAS;
    (pos.0.clamp(camera.0, x_max), pos.1.min(y_max))
}

/// Taille (largeur, hauteur) du repère « Pépin est au-dessus de l'écran ».
const REPERE: (i32, i32) = (5, 3);

/// Où poser le repère en haut de l'écran ? `boite_y`, `boite_h` : ordonnée écran du haut de la
/// boîte de Pépin et sa hauteur ; `milieu_x` : abscisse écran de son milieu. Renvoie l'abscisse
/// écran (bord gauche) du repère, bornée pour qu'il reste entier dans l'écran, ou `None` tant
/// que Pépin est visible (au moins une ligne de sa boîte dans le cadre). Fonction pure.
fn repere_haut(boite_y: i32, boite_h: i32, milieu_x: i32) -> Option<i32> {
    if boite_y + boite_h > 0 {
        return None;
    }
    let max = LARGEUR_LOGIQUE as i32 - REPERE.0;
    Some((milieu_x - REPERE.0 / 2).clamp(0, max))
}

/// Petite flèche corail collée en haut de l'écran, à l'abscisse de Pépin, quand il saute
/// au-dessus du cadre (les petits niveaux ont une caméra verticale figée).
fn dessiner_repere(x: i32) {
    const FLECHE: [(i32, i32); 3] = [(2, 1), (1, 3), (0, 5)]; // (décalage, largeur) par rangée
    for (k, &(dx, w)) in FLECHE.iter().enumerate() {
        rect(x + dx, k as i32, w, 1, CORAIL);
    }
}

fn dessiner_pepin(monde: &Monde, banque: &Banque, alpha: f32, cx: i32, cy: i32) {
    let pas = monde.pas;
    let j = &monde.joueur;
    let miroir = !j.regarde_droite;

    // Mort : trois images jouées à l'endroit de la mort, puis réapparition clignotante.
    if monde.reapparition > 0 {
        let ecoule = REAPPARITION_PAS - monde.reapparition;
        let k = (ecoule as usize * 3 / REAPPARITION_PAS as usize).min(2);
        // Chute dans le vide ou réapparition lointaine : l'animation reste dans le cadre.
        let (mort_x, mort_y) = borner_mort(monde.pos_mort, (cx as f32, cy as f32));
        let (mx, my) = (
            mort_x.round() as i32 - cx - 3,
            mort_y.round() as i32 - cy - 2,
        );
        banque.dessiner(pepin::REAPPARITION[k], mx as f32, my as f32, false);
        if (pas / 3).is_multiple_of(2) {
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

/// Flèche de repère par-dessus tout le monde (pluie comprise) quand Pépin est au-dessus du cadre.
fn dessiner_repere_pepin(monde: &Monde, alpha: f32, cx: i32, cy: i32) {
    let j = &monde.joueur;
    let (px, py) = monde.joueur_interpole(alpha);
    let (bx, by) = (px.round() as i32 - cx, py.round() as i32 - cy);
    if let Some(x) = repere_haut(by, j.boite.h as i32, bx + j.boite.w as i32 / 2) {
        dessiner_repere(x);
    }
}

/// Dessine tout le monde dans la cible logique. `(cx, cy)` : caméra entière.
pub fn dessiner(
    monde: &Monde,
    banque: &Banque,
    particules: &Particules,
    alpha: f32,
    cx: i32,
    cy: i32,
) {
    decor::dessiner_fond(monde.infos.theme, cx, cy, monde.pas);
    dessiner_tuiles(monde, banque, cx, cy);
    dessiner_panneaux(monde, banque, cx, cy);
    dessiner_arrivee(monde, banque, cx, cy);
    dessiner_entites(monde, banque, alpha, cx, cy);
    dessiner_pepin(monde, banque, alpha, cx, cy);
    particules.dessiner(cx, cy);
    decor::dessiner_avant(monde.infos.theme, monde.pas);
    dessiner_repere_pepin(monde, alpha, cx, cy);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_repere_n_apparait_que_pepin_hors_du_cadre() {
        // Boîte de 14 px de haut : visible tant que sa dernière ligne est dans l'écran.
        assert_eq!(repere_haut(100, 14, 160), None);
        assert_eq!(repere_haut(-13, 14, 160), None, "une ligne encore visible");
        assert!(repere_haut(-14, 14, 160).is_some(), "entièrement au-dessus");
        assert!(repere_haut(-60, 14, 160).is_some());
    }

    #[test]
    fn le_repere_est_centre_sur_pepin_et_borne_a_l_ecran() {
        assert_eq!(repere_haut(-20, 14, 160), Some(158));
        assert_eq!(repere_haut(-20, 14, 0), Some(0));
        assert_eq!(repere_haut(-20, 14, -50), Some(0));
        assert_eq!(repere_haut(-20, 14, 319), Some(315));
        assert_eq!(repere_haut(-20, 14, 900), Some(315));
    }

    #[test]
    fn l_annonce_clignote_de_plus_en_plus_vite() {
        let transitions = |a: f32, b: f32| {
            let n = 480;
            let mut t = 0;
            let mut avant = annonce_allumee(a);
            for k in 1..=n {
                let p = a + (b - a) * k as f32 / n as f32;
                let maintenant = annonce_allumee(p);
                if maintenant != avant {
                    t += 1;
                }
                avant = maintenant;
            }
            t
        };
        let debut = transitions(0.0, 0.5);
        let fin = transitions(0.5, 1.0);
        assert!(fin > debut, "début : {debut}, fin : {fin}");
        assert!(annonce_allumee(0.0), "allumée dès la première image");
        // 7 clignotements en tout : 14 changements d'état.
        assert_eq!(transitions(0.0, 1.0), 14);
    }

    #[test]
    fn le_clignotement_le_plus_rapide_reste_lisible_a_60_images_par_seconde() {
        // 48 images d'annonce : jamais plus de 2 changements d'état à la suite sans plateau.
        let etats: Vec<bool> = (0..48).map(|k| annonce_allumee(k as f32 / 48.0)).collect();
        let mut plus_court = usize::MAX;
        let mut serie = 1;
        for w in etats.windows(2) {
            if w[0] == w[1] {
                serie += 1;
            } else {
                plus_court = plus_court.min(serie);
                serie = 1;
            }
        }
        assert!(plus_court >= 2, "plus petite série : {plus_court} image(s)");
    }

    #[test]
    fn l_eclair_part_du_haut_de_l_ecran() {
        // Niveau de 176 px, caméra à -4 : la zone (y = 0) commence 4 px sous le haut.
        assert_eq!(colonne_eclair(0, 80, -4), (0, 84));
        // Caméra à 0 : rien à prolonger.
        assert_eq!(colonne_eclair(0, 80, 0), (0, 80));
        // Zone qui ne part pas du haut du niveau : inchangée.
        assert_eq!(colonne_eclair(32, 80, -4), (36, 80));
        // Niveau défilé : la zone remonte au-dessus de l'écran, rien à prolonger.
        assert_eq!(colonne_eclair(0, 80, 20), (-20, 80));
    }

    #[test]
    fn la_mort_dans_le_vide_reste_dans_le_cadre() {
        let cam = (200.0, -4.0);
        // Chute sous le niveau : ordonnée bornée à 16 px au-dessus du bas de l'écran.
        let (x, y) = borner_mort((300.0, 400.0), cam);
        assert_eq!((x, y), (300.0, -4.0 + 180.0 - 16.0));
        // Déjà dans le cadre : inchangé.
        assert_eq!(borner_mort((250.0, 100.0), cam), (250.0, 100.0));
        // Mort hors champ (réapparition lointaine) : abscisse ramenée dans l'écran.
        assert_eq!(borner_mort((20.0, 100.0), cam).0, 200.0);
        assert_eq!(borner_mort((900.0, 100.0), cam).0, 200.0 + 320.0 - 16.0);
    }
}
