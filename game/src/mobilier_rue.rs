//! Le MOBILIER DE RUE du caniveau (carte 9.20f) : voitures garées, autobus, abribus,
//! poubelles et bancs. Leurs cases sont SOLIDES (`level::Mobilier`, lettres
//! `q a t d l`) : on marche sur le toit d'une voiture, sur une poubelle, sur le
//! toit de l'abribus. La scène ne les dessine pas comme des tuiles
//! (`scene::nom_tuile`) : chaque objet est dessiné ici d'un seul tenant.
//!
//! Règles du dessin :
//! - tout est en rectangles de la palette, au pixel entier, contour sombre, deux
//!   niveaux d'ombre au plus, aucun dégradé ni transparence ;
//! - le DESSUS praticable se lit : il est au pixel près le haut de la case solide,
//!   une ligne de contour sombre puis le LISERÉ CLAIR (`LISERE`, le même que
//!   l'arête des pavés : carte 9.20g) ;
//! - carte 9.20g : chaque objet est cerné de sombre tout autour ; ce qui n'est pas
//!   solide (montants et vitre de l'abribus, affiche) est terne, sans trait
//!   horizontal clair, comme le fond (`decor_rue`) ;
//! - rien ne ressemble à de l'eau : le cyan (`PLUIE`, `PLUIE_SOMBRE`) n'est jamais
//!   employé ici ; les vitres sont d'ardoise, les lumières orange ou or pâle.
//!
//! Deux passes : `dessiner_derriere` (avant les tuiles : les roues de l'autobus,
//! dans l'eau du caniveau, que l'eau recouvre à moitié) et `dessiner` (après les
//! tuiles : les objets eux-mêmes, derrière Pépin et les ennemis).

use macroquad::prelude::draw_rectangle;
use pepin_core::aabb::TAILLE_TUILE;
use pepin_core::level::{GenreMobilier, Mobilier, Niveau};
use pepin_core::palette::{
    Couleur, ARDOISE, ARDOISE_CLAIRE, ARDOISE_NUIT, ARDOISE_OMBRE, ARDOISE_PALE, BRUME, CORAIL,
    CORAIL_CLAIR, NUAGE, ORANGE_SOLEIL, OR_PALE,
};
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use crate::monde::Monde;
use crate::rendu::couleur_mq;

const T: i32 = TAILLE_TUILE as i32;

fn rect(x: i32, y: i32, w: i32, h: i32, c: Couleur) {
    if w > 0 && h > 0 {
        draw_rectangle(x as f32, y as f32, w as f32, h as f32, couleur_mq(c));
    }
}

/// Les couleurs d'une carrosserie : (teinte, reflet).
type Teinte = (Couleur, Couleur);

/// Les teintes des voitures garées, choisies d'après leur colonne (fixes) : des
/// couleurs vives, qui ne se perdent pas sur les façades sombres du fond (carte
/// 9.20g : le fond n'a plus que les tons sombres de la palette).
const TEINTES_VOITURE: [Couleur; 3] = [CORAIL, ARDOISE_PALE, ORANGE_SOLEIL];

/// L'objet est-il (au moins en partie) à l'écran ? `(x, y)` : son coin à l'écran.
fn visible(m: &Mobilier, x: i32, y: i32) -> bool {
    let (l, h) = (LARGEUR_LOGIQUE as i32, HAUTEUR_LOGIQUE as i32);
    // Les roues et les montants débordent un peu : une case de marge.
    x + (m.l + 1) * T > 0 && x - T < l && y + (m.h + 3) * T > 0 && y - T < h
}

/// Le coin haut-gauche de l'objet à l'écran.
fn coin(m: &Mobilier, (cx, cy): (i32, i32)) -> (i32, i32) {
    (m.tx * T - cx, m.ty * T - cy)
}

/// L'autobus repose-t-il sur le trottoir (une case solide sous son milieu) ? Sinon
/// il est garé au-dessus du caniveau : ses roues sont dans l'eau.
fn sur_le_trottoir(n: &Niveau, m: &Mobilier) -> bool {
    n.tuile_decor(m.tx + m.l / 2, m.ty + m.h).est_solide()
}

/// Première passe, avant les tuiles : les roues d'un autobus garé au-dessus du
/// caniveau, posées dans l'eau sous son bas de caisse (l'eau, dessinée ensuite, en
/// couvre le bas).
pub fn dessiner_derriere(monde: &Monde, cx: i32, cy: i32) {
    let n = &monde.niveau;
    for m in &n.mobilier {
        let (x, y) = coin(m, (cx, cy));
        if m.genre == GenreMobilier::Autobus && visible(m, x, y) && !sur_le_trottoir(n, m) {
            let bas = y + m.h * T;
            let w = m.l * T;
            for rx in [x + 12, x + w - 36] {
                roue(rx, bas - 4, 24, ARDOISE_NUIT, ARDOISE);
            }
        }
    }
}

/// Seconde passe, après les tuiles : chaque objet du mobilier de rue.
pub fn dessiner(monde: &Monde, cx: i32, cy: i32) {
    let n = &monde.niveau;
    for m in &n.mobilier {
        let (x, y) = coin(m, (cx, cy));
        if !visible(m, x, y) {
            continue;
        }
        match m.genre {
            GenreMobilier::Voiture => {
                let teinte = TEINTES_VOITURE[m.tx.rem_euclid(3) as usize];
                // Le toit est sur les deux cases du milieu : capot à droite.
                voiture(x, y, m.l * T, teinte);
            }
            GenreMobilier::Autobus => {
                // Deux livrées, d'après la colonne : corail, ou orange.
                let teinte = if m.tx.rem_euclid(2) == 1 {
                    (CORAIL, CORAIL_CLAIR)
                } else {
                    (ORANGE_SOLEIL, OR_PALE)
                };
                let (w, h) = (m.l * T, m.h * T);
                autobus(x, y, (w, h), teinte, monde.pas);
                if sur_le_trottoir(n, m) {
                    // Garé sur le trottoir : ses roues, devant le bas de caisse.
                    for rx in [x + 12, x + w - 28] {
                        rect(rx - 2, y + h - 18, 24, 2, ARDOISE_OMBRE);
                        roue(rx, y + h - 20, 20, ARDOISE_NUIT, ARDOISE);
                    }
                }
            }
            GenreMobilier::Abribus => abribus(x, y, m.l * T, pied_de(n, m) * T - cy),
            GenreMobilier::Poubelle => poubelle(x, y),
            GenreMobilier::Banc => banc(x, y, m.l * T),
        }
    }
}

/// La rangée (en px de niveau, divisée par une case) du sol sous un objet : la
/// première case solide sous le milieu de son bord bas (le bas de la carte sinon).
fn pied_de(n: &Niveau, m: &Mobilier) -> i32 {
    let milieu = m.tx + m.l / 2;
    (m.ty + m.h..n.hauteur() as i32)
        .find(|&y| n.tuile_decor(milieu, y).est_solide())
        .unwrap_or(n.hauteur() as i32)
}

/// Une roue de `d` px de diamètre, en rangées de pixels (un disque au pixel près,
/// sans forme vectorielle) : contour sombre, pneu, jante, moyeu.
fn roue(x: i32, y: i32, d: i32, pneu: Couleur, jante: Couleur) {
    // Demi-largeur de la rangée `k` d'un disque de rayon `r` centré dans `d`.
    let disque = |r: f32, c: Couleur| {
        let centre = d as f32 / 2.0;
        for k in 0..d {
            let dy = k as f32 + 0.5 - centre;
            if dy.abs() < r {
                let demi = (r * r - dy * dy).sqrt().round() as i32;
                rect(x + d / 2 - demi, y + k, 2 * demi, 1, c);
            }
        }
    };
    let r = d as f32 / 2.0;
    disque(r, ARDOISE_OMBRE);
    disque(r - 1.5, pneu);
    disque(r * 0.45, jante);
    rect(x + d / 2 - 1, y + d / 2 - 1, 2, 2, ARDOISE_OMBRE);
}

/// Le LISERÉ CLAIR (carte 9.20g) : la ligne de 1 px, juste sous le contour du
/// haut, de TOUT dessus où l'on se pose (mobilier de rue, pavés, bordures,
/// marquise). Toujours la même teinte : au premier coup d'œil, « ici on marche ».
const LISERE: Couleur = NUAGE;

/// Un dessus praticable de `w` px en `(x, y)` : le contour sombre (le haut exact
/// de la case solide), puis le liseré clair (`LISERE`).
fn dessus(x: i32, y: i32, w: i32) {
    rect(x, y, w, 1, ARDOISE_OMBRE);
    rect(x + 1, y + 1, w - 2, 1, LISERE);
}

/// Voiture garée, vue de profil, capot à droite (`w` = 4 cases, 2 de haut) : le
/// TOIT (cases du milieu de la rangée du haut), le capot et le coffre (rangée du
/// bas), deux roues dans leurs passages, phare or pâle devant, feu corail derrière.
fn voiture(x: i32, y: i32, w: i32, teinte: Couleur) {
    // L'habitacle : de la 2e à la 3e case, avec ses montants inclinés.
    let (h0, h1) = (x + T, x + w - T);
    dessus(h0 + 2, y, h1 - h0 - 4);
    rect(h0 + 1, y + 1, 1, 15, ARDOISE_OMBRE);
    rect(h1 - 2, y + 1, 1, 15, ARDOISE_OMBRE);
    rect(h0 + 2, y + 2, h1 - h0 - 4, 14, teinte);
    // Les vitres : arrière et avant, séparées par le montant du milieu ; un
    // reflet en biais sur chacune.
    let milieu = (h0 + h1) / 2;
    for (v0, v1) in [(h0 + 4, milieu - 1), (milieu + 1, h1 - 4)] {
        rect(v0, y + 4, v1 - v0, 9, ARDOISE_OMBRE);
        rect(v0 + 1, y + 5, v1 - v0 - 2, 7, ARDOISE_NUIT);
        for k in 0..4 {
            rect(v0 + 3 + k, y + 10 - 2 * k, 1, 2, ARDOISE_CLAIRE);
        }
    }
    // La caisse : capot et coffre au ras de la rangée du bas.
    let yb = y + T;
    dessus(x + 1, yb, w - 2);
    // Sous l'habitacle, la caisse continue : le liseré n'y passe pas (on ne se
    // pose que sur le capot et le coffre, de part et d'autre).
    rect(h0 + 1, yb, h1 - h0 - 2, 2, teinte);
    rect(x, yb + 1, 1, 10, ARDOISE_OMBRE);
    rect(x + w - 1, yb + 1, 1, 10, ARDOISE_OMBRE);
    rect(x + 1, yb + 2, w - 2, 9, teinte);
    // La ligne de caisse (ombre) et le pare-chocs.
    rect(x + 1, yb + 6, w - 2, 1, ARDOISE);
    rect(x, yb + 11, w, 2, ARDOISE_OMBRE);
    rect(x + 2, yb + 11, 6, 1, ARDOISE_PALE);
    rect(x + w - 8, yb + 11, 6, 1, ARDOISE_PALE);
    // Les portières : deux joints verticaux et deux poignées.
    for px in [h0 + 2, milieu] {
        rect(px, yb + 2, 1, 9, ARDOISE);
        rect(px + 4, yb + 4, 3, 1, ARDOISE_OMBRE);
    }
    // Phare devant (à droite), feu derrière.
    rect(x + w - 4, yb + 3, 3, 3, OR_PALE);
    rect(x + 1, yb + 3, 2, 3, CORAIL);
    // Les roues dans leurs passages (l'arc sombre au-dessus de chaque roue).
    for rx in [x + 6, x + w - 18] {
        rect(rx - 1, yb + 3, 14, 2, ARDOISE_OMBRE);
        roue(rx, yb + 4, 12, ARDOISE_NUIT, ARDOISE_PALE);
    }
}

/// Autobus garé (`w` x `h` px), avant à droite : toit praticable, la girouette
/// allumée, la rangée des vitres éclairées de l'intérieur, la porte, une bande
/// claire, le bas de caisse sombre. Le clignotant avant s'allume une demi-seconde
/// sur deux (il attend à l'arrêt).
fn autobus(x: i32, y: i32, (w, h): (i32, i32), (teinte, reflet): Teinte, pas: u32) {
    dessus(x + 1, y, w - 2);
    rect(x, y + 1, 1, h - 1, ARDOISE_OMBRE);
    rect(x + w - 1, y + 1, 1, h - 1, ARDOISE_OMBRE);
    rect(x + 1, y + 2, w - 2, h - 6, teinte);
    rect(x + 1, y + 2, w - 2, 2, reflet);
    // Le bas de caisse, sombre, jusqu'au bas de la dernière case.
    rect(x, y + h - 4, w, 4, ARDOISE_OMBRE);
    rect(x + 1, y + h - 4, w - 2, 1, ARDOISE);
    // La rangée des vitres (éclairées : ardoise pâle, sièges en ombre) et le
    // pare-brise à l'avant, plus haut.
    let (v0, v1) = (y + 7, y + 27);
    let avant = x + w - 18;
    let mut vx = x + 4;
    while vx + 12 <= avant - 2 {
        rect(vx, v0, 13, v1 - v0, ARDOISE_OMBRE);
        rect(vx + 1, v0 + 1, 11, v1 - v0 - 2, ARDOISE_PALE);
        // Le dossier d'un siège en bas de la vitre, un reflet en biais en haut.
        rect(vx + 3, v1 - 7, 4, 6, ARDOISE);
        rect(vx + 3, v1 - 8, 4, 1, ARDOISE_CLAIRE);
        for k in 0..3 {
            rect(vx + 7 + k, v0 + 4 - k, 1, 2, BRUME);
        }
        vx += 16;
    }
    rect(avant, y + 4, 16, v1 - y, ARDOISE_OMBRE);
    rect(avant + 1, y + 11, 14, v1 - y - 8, ARDOISE_NUIT);
    for k in 0..5 {
        rect(avant + 4 + k, y + 22 - 2 * k, 1, 2, ARDOISE_CLAIRE);
    }
    // La girouette (le numéro de la ligne, allumé) au-dessus du pare-brise.
    rect(avant + 1, y + 5, 14, 5, OR_PALE);
    for k in [3, 6, 10] {
        rect(avant + k, y + 6, 2, 3, ORANGE_SOLEIL);
    }
    // La bande claire sous les vitres, la porte avant (deux vantaux vitrés).
    rect(x + 1, v1 + 2, w - 2, 2, reflet);
    let porte = avant - 36;
    rect(porte, v1 + 4, 18, h - 4 - (v1 + 4 - y), ARDOISE_OMBRE);
    for k in [porte + 1, porte + 9] {
        rect(k, v1 + 5, 8, h - 4 - (v1 + 6 - y), ARDOISE_NUIT);
        rect(k + 2, v1 + 7, 1, 8, ARDOISE_CLAIRE);
    }
    // Phare et clignotant devant, feu derrière.
    rect(x + w - 4, y + h - 13, 3, 4, OR_PALE);
    if (pas / 30).is_multiple_of(2) {
        rect(x + w - 4, y + h - 8, 3, 2, ORANGE_SOLEIL);
    }
    rect(x + 1, y + h - 14, 2, 6, ORANGE_SOLEIL);
}

/// Abribus : le toit (la case solide, `w` px de large en `(x, y)`), ses montants
/// jusqu'au trottoir (`sol`, ordonnée à l'écran du dessus des pavés), la vitre du
/// fond (un cadre et des reflets, sans remplissage : on voit la rue derrière) et un
/// panneau d'affichage (un parapluie dessiné dessus). On passe dessous.
fn abribus(x: i32, y: i32, w: i32, sol: i32) {
    // Le toit : la dalle, son bandeau allumé, son ombre dessous.
    dessus(x, y, w);
    rect(x, y + 1, 1, T - 1, ARDOISE_OMBRE);
    rect(x + w - 1, y + 1, 1, T - 1, ARDOISE_OMBRE);
    rect(x + 1, y + 2, w - 2, 6, ARDOISE_CLAIRE);
    rect(x + 1, y + 8, w - 2, 5, OR_PALE);
    for k in (x + 4..x + w - 6).step_by(6) {
        rect(k, y + 9, 3, 3, ARDOISE);
    }
    rect(x, y + 13, w, 3, ARDOISE_OMBRE);
    // Sous le toit, rien n'est solide (on passe dessous) : carte 9.20g, les
    // montants, la vitre et l'affiche sont ternes, sans trait horizontal clair.
    // Les montants, aux deux bouts.
    let haut = y + T;
    for mx in [x + 1, x + w - 4] {
        rect(mx, haut, 3, sol - haut, ARDOISE_OMBRE);
        rect(mx + 1, haut, 1, sol - haut, ARDOISE);
    }
    // La vitre du fond : un cadre fin et deux reflets en biais.
    let (v0, v1) = (x + 4, x + w - 4);
    rect(v0, haut + 3, v1 - v0, 1, ARDOISE);
    rect(v0, sol - 3, v1 - v0, 1, ARDOISE);
    for (k, r) in [(v0 + 8, 6), (v0 + 14, 4)] {
        for i in 0..r {
            rect(k + i, haut + 4 + r * 2 - 2 * i, 1, 2, ARDOISE);
        }
    }
    // Le panneau d'affichage, contre le montant de droite : éteint la nuit.
    let (p0, p1) = (x + w - 18, sol - 26);
    rect(p0, p1, 13, 23, ARDOISE_OMBRE);
    rect(p0 + 1, p1 + 1, 11, 21, ARDOISE_NUIT);
    // Le parapluie de l'affiche : sa toile corail, son manche.
    rect(p0 + 3, p1 + 6, 7, 2, CORAIL);
    rect(p0 + 4, p1 + 5, 5, 1, CORAIL);
    rect(p0 + 6, p1 + 8, 1, 7, ARDOISE);
    rect(p0 + 5, p1 + 14, 1, 1, ARDOISE);
}

/// Poubelle de rue (une case) : couvercle praticable, cuve cannelée, cernée de
/// sombre tout autour (carte 9.20g).
fn poubelle(x: i32, y: i32) {
    dessus(x, y, 16);
    rect(x, y + 1, 1, 4, ARDOISE_OMBRE);
    rect(x + 15, y + 1, 1, 4, ARDOISE_OMBRE);
    rect(x + 1, y + 2, 14, 2, ARDOISE_CLAIRE);
    rect(x, y + 4, 16, 1, ARDOISE_OMBRE);
    rect(x + 2, y + 5, 12, 11, ARDOISE_OMBRE);
    rect(x + 3, y + 5, 10, 10, ARDOISE_CLAIRE);
    for k in [5, 8, 11] {
        rect(x + k, y + 6, 1, 8, ARDOISE);
    }
    rect(x + 3, y + 5, 10, 1, ARDOISE_PALE);
}

/// Banc public (une rangée de `w` px) : le haut du dossier est le dessus
/// praticable ; dessous, deux lattes de dossier, l'assise claire et les pieds de
/// fonte aux deux bouts. Carte 9.20g : cerné de sombre aux deux bouts.
fn banc(x: i32, y: i32, w: i32) {
    dessus(x, y, w);
    rect(x, y + 2, w, 2, ARDOISE_PALE);
    rect(x, y + 4, w, 1, ARDOISE_OMBRE);
    rect(x + 1, y + 5, w - 2, 2, ARDOISE_CLAIRE);
    rect(x, y + 7, w, 1, ARDOISE_OMBRE);
    // L'assise, un peu en avant.
    rect(x, y + 9, w, 1, ARDOISE_OMBRE);
    rect(x, y + 10, w, 2, ARDOISE_PALE);
    rect(x, y + 12, w, 1, ARDOISE_OMBRE);
    // Le contour des deux bouts (dossier et assise).
    for bx in [x, x + w - 1] {
        rect(bx, y + 1, 1, 7, ARDOISE_OMBRE);
        rect(bx, y + 9, 1, 4, ARDOISE_OMBRE);
    }
    // Les pieds de fonte, et les montants du dossier au-dessus.
    for px in [x + 2, x + w - 5] {
        rect(px, y + 2, 3, 14, ARDOISE_OMBRE);
        rect(px + 1, y + 13, 1, 3, ARDOISE);
    }
}

/// Test de lisibilité (carte 9.20g), build de QA seulement (touche O, voir
/// `entrees::solides_qa`) : chaque case SOLIDE de la grille à l'écran est repassée
/// en or, une rangée de pixels sur deux (on voit encore le dessin dessous). Sur une
/// capture, l'or doit tomber exactement sur ce qui PARAÎT solide.
#[cfg(feature = "debug_niveaux")]
pub fn dessiner_solides_qa(monde: &Monde, cx: i32, cy: i32) {
    let n = &monde.niveau;
    let (l, h) = (LARGEUR_LOGIQUE as i32, HAUTEUR_LOGIQUE as i32);
    for ty in cy.div_euclid(T)..=(cy + h).div_euclid(T) {
        for tx in cx.div_euclid(T)..=(cx + l).div_euclid(T) {
            if (0..n.largeur() as i32).contains(&tx) && n.tuile_decor(tx, ty).est_solide() {
                for k in (0..T).step_by(2) {
                    rect(tx * T - cx, ty * T - cy + k, T, 1, pepin_core::palette::OR);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sprites::{objets_decor, pixel, tuiles_monde2};

    /// Carte 9.20g : tout dessus où l'on se pose dans le caniveau (pavés, bordures,
    /// grille de la bouche d'égout) a, sous son contour, le MÊME liseré clair que
    /// le mobilier de rue (`LISERE`), d'un bout à l'autre de la tuile.
    #[test]
    fn les_dessus_de_la_rue_ont_le_lisere_du_mobilier() {
        let dessus = [
            "can_pave",
            "can_pave_g",
            "can_pave_d",
            "can_bordure",
            "can_bordure_g",
            "can_bordure_m",
            "can_bordure_d",
            "can_egout_grille",
        ];
        for nom in dessus {
            let (_, d) = tuiles_monde2::SPRITES
                .iter()
                .find(|(n, _)| *n == nom)
                .unwrap_or_else(|| panic!("{nom} introuvable"));
            assert!(d[0].chars().all(|c| c == '0'), "{nom} : contour du haut");
            let clairs = d[1]
                .chars()
                .filter(|&c| pixel(c) == Ok(Some(LISERE)))
                .count();
            assert!(clairs >= 14, "{nom} : liseré de {clairs} px seulement");
        }
    }

    /// Carte 9.20g : le décor de la rue SANS collision (poubelle `U`, borne `I` ;
    /// la gouttière `J` dans `decor_rue`) n'a aucune teinte claire
    /// (ardoise claire et au-dessus) : rien de clair qui se prendrait pour un dessus.
    #[test]
    fn le_decor_sans_collision_de_la_rue_est_terne() {
        let ternes = [ARDOISE_OMBRE, ARDOISE_NUIT, ARDOISE];
        for (nom, d) in objets_decor::SPRITES {
            let rangees = match *nom {
                "borne" | "poubelle" => d.len(),
                // La descente de gouttière `J` : `decor_rue` (carte 9.20h), testée là.
                _ => continue,
            };
            for ligne in &d[..rangees] {
                for c in ligne.chars() {
                    if let Ok(Some(couleur)) = pixel(c) {
                        assert!(ternes.contains(&couleur), "{nom} : « {c} » trop clair");
                    }
                }
            }
        }
    }
}
