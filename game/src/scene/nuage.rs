//! Le Grand Nuage (boss du Monde 1) : ses images selon sa phase, ses éclairs annoncés
//! et la déchirure du ciel quand il est vaincu. Les règles du combat sont dans
//! `pepin_core::entities::nuage` : ce fichier ne fait que les montrer.

use pepin_core::aabb::Aabb;
use pepin_core::entities::nuage::{EtatColonne, GrandNuage, PhaseNuage, NB_COLONNES};
use pepin_core::entities::Entite;
use pepin_core::palette::*;
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use super::{image, rect};
use crate::monde::Monde;
use crate::sprites::{ennemis, Banque};

/// Annonce de l'éclair : le pointillé est-il allumé à cet instant ? `progression` va de 0.0
/// (début de l'annonce) à 1.0 (frappe imminente) : le clignotement ACCÉLÈRE, de 2,5 Hz au début
/// à 15 Hz à la fin (7 clignotements en 0,8 s). Fonction pure, testable.
pub(crate) fn annonce_allumee(progression: f32) -> bool {
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
pub(super) fn dessiner_eclair(
    zone: &Aabb,
    annonce: bool,
    progression: f32,
    frappe: bool,
    cx: i32,
    cy: i32,
) {
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

/// Image du Grand Nuage (fonction pure) selon sa phase, le temps passé dans cette
/// phase (`ecoule`) et le pas du jeu ; `None` : rien à dessiner à cette image.
/// - prélude et colère : il fronce les sourcils (2 images de 0,2 s) ;
/// - essoufflé : il halète (2 images de 0,25 s) ;
/// - touché : il clignote (4 images visibles, 4 cachées) ;
/// - le ciel se déchire : touché, visible la 1re seconde, puis il clignote de plus en
///   plus (2 images sur 3, puis 1 sur 3) et disparaît ;
/// - vaincu : plus rien.
pub(crate) fn image_nuage(phase: PhaseNuage, ecoule: u16, pas: u32) -> Option<&'static str> {
    match phase {
        PhaseNuage::Prelude | PhaseNuage::Colere => Some(image(&ennemis::NUAGE_COLERE, pas, 12)),
        PhaseNuage::Essouffle => Some(image(&ennemis::NUAGE_ESSOUFFLE, pas, 15)),
        PhaseNuage::Touche => (ecoule / 4).is_multiple_of(2).then_some("nuage_touche"),
        PhaseNuage::Dechirure => {
            let visible = match ecoule {
                0..60 => true,
                60..90 => !ecoule.is_multiple_of(3),
                _ => ecoule.is_multiple_of(3),
            };
            visible.then_some("nuage_touche")
        }
        PhaseNuage::Vaincu => None,
    }
}

/// Décalage horizontal (px) du Grand Nuage : il gronde pendant le prélude (un pixel
/// de chaque côté, toutes les 3 images) et tremble pendant la déchirure du ciel (deux
/// pixels, chaque image). Le dessin bouge, jamais sa boîte. Fonction pure.
pub(crate) fn tremblement_nuage(phase: PhaseNuage, pas: u32) -> i32 {
    match phase {
        PhaseNuage::Prelude => [0, 1, 0, -1][(pas / 3) as usize % 4],
        PhaseNuage::Dechirure => [-2, 2][pas as usize % 2],
        _ => 0,
    }
}

/// Le Grand Nuage : d'abord ses 17 colonnes d'éclairs, dessinées EXACTEMENT comme un
/// éclair `e` (même annonce : barre dorée au pied, pointillé qui clignote de plus en
/// plus vite ; même frappe), puis son corps par-dessus. `(sx, sy)` : coin de sa boîte
/// à l'écran.
pub(super) fn dessiner_nuage(
    g: &GrandNuage,
    banque: &Banque,
    pas: u32,
    sx: f32,
    sy: f32,
    cx: i32,
    cy: i32,
) {
    for (i, zone) in g.colonnes.iter().enumerate().take(NB_COLONNES) {
        match g.etat_colonne(i) {
            EtatColonne::Calme => {}
            EtatColonne::Annonce(p) => dessiner_eclair(zone, true, p, false, cx, cy),
            EtatColonne::Frappe => dessiner_eclair(zone, false, 0.0, true, cx, cy),
        }
    }
    if let Some(nom) = image_nuage(g.phase(), g.ecoule(), pas) {
        let dx = tremblement_nuage(g.phase(), pas) as f32;
        // Mode facile (carte 8.14) : le nuage est plus pâle (à demi transparent).
        if g.est_facile() {
            banque.dessiner_pale(nom, sx + dx, sy);
        } else {
            banque.dessiner(nom, sx + dx, sy, false);
        }
    }
}

/// Hauteur (px) d'une bande de la déchirure du ciel.
const BANDE_DECHIRURE: i32 = 4;
/// Zigzag des bords de la déchirure (px, d'une bande à l'autre).
const ZIGZAG_DECHIRURE: [i32; 6] = [0, 3, -2, 4, -3, 1];

/// La déchirure du ciel (fonction pure) : pour chaque bande de 4 px de l'écran, de haut
/// en bas, les abscisses (à l'écran) de ses bords gauche et droit. Elle s'ouvre autour
/// de `milieu` (abscisse à l'écran du milieu du nuage) : vide au début
/// (`progression` 0), plus large que l'écran à la fin (1). Ses bords sont en zigzag,
/// comme une étoffe qui craque.
pub(crate) fn bandes_dechirure(progression: f32, milieu: i32) -> Vec<(i32, i32)> {
    let p = progression.clamp(0.0, 1.0);
    if p <= 0.0 {
        return Vec::new();
    }
    let demi = (p * p * (LARGEUR_LOGIQUE as f32 * 0.75)).round() as i32 + 1;
    (0..HAUTEUR_LOGIQUE as i32 / BANDE_DECHIRURE)
        .map(|k| {
            // Le zigzag ne referme jamais la fente : au plus `demi - 1` de chaque côté.
            let z = ZIGZAG_DECHIRURE[k as usize % ZIGZAG_DECHIRURE.len()].min(demi - 1);
            let w = ZIGZAG_DECHIRURE[(k as usize + 2) % ZIGZAG_DECHIRURE.len()].min(demi - 1);
            (milieu - demi + z, milieu + demi - w)
        })
        .collect()
}

/// Le ciel se déchire derrière le Grand Nuage vaincu : une fente de lumière d'or pâle,
/// bordée d'or, qui s'ouvre jusqu'à remplir l'écran. Dessinée juste après le fond (les
/// tuiles, Pépin et le nuage passent devant).
pub(super) fn dessiner_dechirure(monde: &Monde, cx: i32) {
    let Some(g) = monde.entites.iter().find_map(|e| match e {
        Entite::Nuage(g) => Some(g),
        _ => None,
    }) else {
        return;
    };
    let milieu = (g.boite.x + g.boite.w / 2.0).round() as i32 - cx;
    for (k, (x0, x1)) in bandes_dechirure(g.progression_dechirure(), milieu)
        .into_iter()
        .enumerate()
    {
        let y = k as i32 * BANDE_DECHIRURE;
        rect(x0 - 2, y, x1 - x0 + 4, BANDE_DECHIRURE, OR);
        rect(x0, y, x1 - x0, BANDE_DECHIRURE, OR_PALE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Grand Nuage (carte 5.11) ---

    /// Chaque phase a son image : colère (prélude compris) et essoufflé s'animent,
    /// touché clignote, la déchirure le fait disparaître, vaincu il n'est plus là.
    #[test]
    fn le_grand_nuage_change_d_image_selon_sa_phase() {
        use PhaseNuage::*;
        assert_eq!(image_nuage(Prelude, 0, 0), Some("nuage_colere0"));
        assert_eq!(image_nuage(Colere, 5, 12), Some("nuage_colere1"));
        assert_eq!(image_nuage(Essouffle, 0, 0), Some("nuage_essouffle0"));
        assert_eq!(image_nuage(Essouffle, 0, 15), Some("nuage_essouffle1"));
        // Touché : 4 images visibles, 4 cachées.
        let vues: Vec<bool> = (0..16)
            .map(|e| image_nuage(Touche, e, 0).is_some())
            .collect();
        assert_eq!(vues.iter().filter(|&&v| v).count(), 8);
        assert!(vues[..4].iter().all(|&v| v) && vues[4..8].iter().all(|&v| !v));
        assert_eq!(image_nuage(Touche, 0, 0), Some("nuage_touche"));
        // Déchirure : visible la 1re seconde, de moins en moins ensuite.
        let compte = |r: std::ops::Range<u16>| {
            r.filter(|&e| image_nuage(Dechirure, e, 0).is_some())
                .count()
        };
        assert_eq!(compte(0..60), 60);
        assert_eq!(compte(60..90), 20);
        assert_eq!(compte(90..120), 10);
        assert_eq!(image_nuage(Vaincu, 0, 0), None);
    }

    /// Il gronde au prélude (au plus 1 px), tremble à la déchirure (2 px), et reste
    /// immobile le reste du combat.
    #[test]
    fn le_grand_nuage_gronde_puis_tremble() {
        let prelude: Vec<i32> = (0..12)
            .map(|p| tremblement_nuage(PhaseNuage::Prelude, p))
            .collect();
        assert_eq!(prelude, vec![0, 0, 0, 1, 1, 1, 0, 0, 0, -1, -1, -1]);
        assert_eq!(tremblement_nuage(PhaseNuage::Dechirure, 0), -2);
        assert_eq!(tremblement_nuage(PhaseNuage::Dechirure, 1), 2);
        for phase in [
            PhaseNuage::Colere,
            PhaseNuage::Essouffle,
            PhaseNuage::Touche,
        ] {
            assert!((0..20).all(|p| tremblement_nuage(phase, p) == 0));
        }
    }

    /// La déchirure : rien avant, une fente autour du nuage qui s'élargit, plus large
    /// que l'écran à la fin ; une bande de 4 px pour chaque rangée de l'écran (45),
    /// bords en zigzag (pas tous alignés).
    #[test]
    fn le_ciel_se_dechire_autour_du_nuage() {
        assert!(bandes_dechirure(0.0, 160).is_empty());
        let debut = bandes_dechirure(0.1, 160);
        assert_eq!(debut.len(), 45);
        assert!(debut.iter().all(|&(a, b)| a < 160 && b > 160 && b - a < 20));
        let milieu = bandes_dechirure(0.5, 160);
        let fin = bandes_dechirure(1.0, 160);
        let largeur = |v: &[(i32, i32)]| v.iter().map(|&(a, b)| b - a).min().unwrap();
        assert!(largeur(&debut) < largeur(&milieu) && largeur(&milieu) < largeur(&fin));
        assert!(fin
            .iter()
            .all(|&(a, b)| a <= 0 && b >= LARGEUR_LOGIQUE as i32));
        let gauches: std::collections::BTreeSet<i32> = milieu.iter().map(|b| b.0).collect();
        assert!(gauches.len() >= 4, "bord en zigzag");
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
}
