//! Rendu de la montée de la Pieuvre (carte 9.36a, décision d'AGA : « une montée
//! contre l'eau qui avance plus vite avec des objets qui tombent du ciel envoyés
//! par la pieuvre », et à la fin « elle finit par s'enfuir ») :
//! - la PIEUVRE AU LOIN (`Pieuvre::au_loin`) : sa tête de la phase 1, à moitié
//!   dans la mer, à droite de la tour ; pendant la furie, elle s'y retire depuis
//!   sa place dans l'arène ; son BRAS se dresse à sa gauche : au repos, il prépare
//!   (l'objet au bout), puis il lance (l'objet file vers le ciel) (`Pieuvre::lance`) ;
//! - chaque OBJET qui tombe (`Pieuvre::chutes`) : son OMBRE clignote sur le sol où
//!   il va se briser et le REPÈRE « ! » palpite en haut de sa colonne pendant
//!   l'annonce, puis l'objet tombe (l'ombre reste, une traînée au-dessus de lui),
//!   puis ses éclats ;
//! - LA FUITE : surprise (une goutte), l'encre, la tête plonge, une trace d'encre
//!   et des remous au large ; la mer redescend ;
//! - LA MER elle-même (`dessiner_mer`) : une vraie masse d'eau, assemblée de
//!   sprites (`pieuvre_mer0/1` sur 56 px, puis `pieuvre_mer_fond`), et les
//!   corniches noyées en silhouette (`pieuvre_mer_noyee`).
//!
//! Carte 9.36b : le VRAI dessin, d'après la planche acceptée par AGA (sources :
//! `docs/dessins-936.py`). Tout est fait de sprites posés au pixel entier
//! (`assets/sprites/pieuvre/pieuvre_*.png`) : aucune couleur ajoutée, aucun flou.
//! Seuls la hauteur de la mer, la place des objets et le moment de l'annonce
//! viennent de la simulation. En mode facile, le CORPS de la Pieuvre (tête, bras,
//! encre) se dessine pâle, comme en phase 1 ; les objets, leur repère et la mer
//! restent pleins (un danger doit rester lisible).

use pepin_core::aabb::TAILLE_TUILE;
use pepin_core::entities::pieuvre::{EtatChute, Objet, PhasePieuvre, Pieuvre};
use pepin_core::HAUTEUR_LOGIQUE;

use crate::dessin_pieuvre::la_pieuvre;
use crate::monde::Monde;
use crate::sprites::Banque;

/// Image d'une animation à deux images de `duree` pas chacune.
fn image(anim: [&'static str; 2], pas: u32, duree: u32) -> &'static str {
    anim[(pas / duree) as usize % 2]
}

/// Hauteur (px) de la bande d'eau dégradée sous la surface (`pieuvre_mer0/1`,
/// `pieuvre_mer_noyee`) ; plus bas, l'eau de nuit unie (`..._fond`, 16 x 16).
const BANDE_MER: i32 = 56;
/// Largeur (px) d'une image de la mer (elle se répète).
const LARGEUR_MER: i32 = 64;
/// Le bras qui lance (36 x 42) : posé 16 px à gauche de la tête, le bas sur la mer.
const BRAS: (f32, f32) = (16.0, 42.0);
/// Le repère « ! » ne monte jamais plus haut que cette ordonnée de l'écran (sous
/// l'interface : gouttes, vies, chrono).
const Y_REPERE: f32 = 20.0;

/// Tout ce que la phase 2 ajoute au dessin de la pieuvre du niveau (caméra
/// `cx`, `cy`), devant les tuiles et la mer, derrière la crête d'écume
/// (`mer_montee::dessiner_devant`) et Pépin.
pub fn dessiner(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    let Some(p) = la_pieuvre(monde).filter(|p| p.en_fuite()) else {
        return;
    };
    let camera = (cx as f32, cy as f32);
    dessiner_au_loin(p, banque, monde.pas, camera);
    dessiner_chutes(p, banque, monde.pas, camera);
}

/// Le sprite d'un objet qui tombe.
fn nom_objet(objet: Objet) -> &'static str {
    match objet {
        Objet::Caisse => "pieuvre_caisse",
        Objet::Tonneau => "pieuvre_tonneau",
        Objet::Epave => "pieuvre_epave",
    }
}

/// Les éclats d'un objet brisé.
fn nom_eclats(objet: Objet) -> &'static str {
    match objet {
        Objet::Caisse => "pieuvre_eclats_caisse",
        Objet::Tonneau => "pieuvre_eclats_tonneau",
        Objet::Epave => "pieuvre_eclats_epave",
    }
}

/// La Pieuvre au loin : la tête, coupée à la surface de la mer (le reste est
/// sous l'eau), et son bras. Furie et montée : en colère ; la fuite : touchée,
/// elle plonge dans son encre.
fn dessiner_au_loin(p: &Pieuvre, banque: &Banque, pas: u32, (cx, cy): (f32, f32)) {
    let (Some(tete), Some(mer)) = (p.au_loin(), p.surface_mer()) else {
        return;
    };
    let facile = p.est_facile();
    let (x, y) = ((tete.x - cx).round(), (tete.y - cy).round());
    let surface = (mer - cy).round();
    let fuite = p.phase() == PhasePieuvre::Plongee;
    if fuite {
        dessiner_encre(p, banque, x, surface);
    }
    let visible = (mer - tete.y).clamp(0.0, tete.h).round();
    if visible > 0.0 {
        let nom = match p.phase() {
            PhasePieuvre::Plongee => "pieuvre_touchee",
            PhasePieuvre::Furie => image(["pieuvre_colere0", "pieuvre_colere1"], pas, 4),
            _ => image(["pieuvre_colere0", "pieuvre_colere1"], pas, 20),
        };
        if facile {
            banque.dessiner_pale_haut(nom, x, y, visible);
        } else {
            banque.dessiner_haut(nom, x, y, visible);
        }
    }
    if p.phase() == PhasePieuvre::Montee {
        dessiner_bras(p, banque, x, surface);
    }
    if fuite {
        dessiner_fuite(p, banque, pas, x, surface);
    }
}

/// Le bras dressé à gauche de la tête (montée seulement) : au repos, puis, à
/// chaque lancer (`Pieuvre::lance`, de 0 à 1), il PRÉPARE (l'objet au bout) et
/// LANCE (l'objet file vers le ciel ; il retombera, annoncé, sur la tour).
fn dessiner_bras(p: &Pieuvre, banque: &Banque, x: f32, surface: f32) {
    let geste = p.lance();
    let (bx, by) = (x - BRAS.0, surface - BRAS.1);
    let nom = match geste {
        Some(k) if k < 0.5 => "pieuvre_bras_prepare",
        Some(_) => "pieuvre_bras_lance",
        None => "pieuvre_bras_repos",
    };
    if p.est_facile() {
        banque.dessiner_pale(nom, bx, by);
    } else {
        banque.dessiner(nom, bx, by, false);
    }
    // L'objet du dernier lancer : tenu au bout du bras, puis lancé.
    let dernier = p.chutes().iter().max_by_key(|c| c.debut).map(|c| c.objet);
    if let (Some(k), Some(objet)) = (geste, dernier) {
        let (ox, oy) = if k < 0.5 {
            (bx + 14.0, by - 2.0)
        } else {
            (bx + 36.0, by - 18.0 - (k - 0.5) * 320.0)
        };
        banque.dessiner(nom_objet(objet), ox.round(), oy.round(), false);
    }
}

/// L'encre de la fuite, DERRIÈRE la tête qui plonge : un nuage, puis un plus
/// grand, puis une trace sur l'eau (selon `Pieuvre::progression_plongee`).
fn dessiner_encre(p: &Pieuvre, banque: &Banque, x: f32, surface: f32) {
    let k = p.progression_plongee();
    if k < 0.25 {
        return;
    }
    // Les nuages ont 16 rangées : la dernière sur la surface.
    let etape = if k < 0.55 {
        0
    } else if k < 0.85 {
        1
    } else {
        2
    };
    let (dx, dy) = [(-31.0, -15.0), (-34.0, -15.0), (-4.0, 1.0)][etape];
    let nom = ["pieuvre_encre0", "pieuvre_encre1", "pieuvre_encre2"][etape];
    if p.est_facile() {
        banque.dessiner_pale(nom, x + dx, surface + dy);
    } else {
        banque.dessiner(nom, x + dx, surface + dy, false);
    }
}

/// Devant la tête qui plonge : la goutte de la surprise, puis les bulles, et à la
/// fin les remous là où elle a disparu.
fn dessiner_fuite(p: &Pieuvre, banque: &Banque, pas: u32, x: f32, surface: f32) {
    let k = p.progression_plongee();
    if k < 0.25 {
        let saut = ((pas / 6) % 2) as f32;
        let nom = "pieuvre_goutte";
        if p.est_facile() {
            banque.dessiner_pale(nom, x + 46.0, surface - 24.0 - saut);
        } else {
            banque.dessiner(nom, x + 46.0, surface - 24.0 - saut, false);
        }
        return;
    }
    let nom = image(["pieuvre_bulles0", "pieuvre_bulles1"], pas, 8);
    if p.est_facile() {
        banque.dessiner_pale(nom, x + 16.0, surface - 14.0);
    } else {
        banque.dessiner(nom, x + 16.0, surface - 14.0, false);
    }
    if k >= 0.85 {
        let remous = image(["remous_pieuvre0", "remous_pieuvre1"], pas, 6);
        banque.dessiner(remous, x + 8.0, surface - 4.0, false);
    }
}

/// Les objets lancés : l'ombre qui clignote et le repère « ! » qui palpite
/// pendant l'annonce (plus vite à la fin), l'objet qui tombe au-dessus de son
/// ombre avec sa traînée, puis ses éclats.
fn dessiner_chutes(p: &Pieuvre, banque: &Banque, pas: u32, (cx, cy): (f32, f32)) {
    let t = p.ecoule();
    for c in p.chutes() {
        let (x, sol) = ((c.x - cx).round(), (c.sol - cy).round());
        match c.etat(t) {
            Some(EtatChute::Annonce(a)) => {
                let rythme = if a < 0.5 { 6 } else { 3 };
                let allume = (pas / rythme).is_multiple_of(2);
                if allume {
                    banque.dessiner("ombre_caisse", x, sol, false);
                }
                // Le repère : en haut de la colonne, là où l'objet apparaîtra.
                let y = (c.depart - cy).round().max(Y_REPERE);
                let repere = if allume {
                    "pieuvre_repere0"
                } else {
                    "pieuvre_repere1"
                };
                banque.dessiner(repere, x + 4.0, y, false);
            }
            Some(EtatChute::Chute(b)) => {
                banque.dessiner("ombre_caisse", x, sol, false);
                let y = (b.y - cy).round();
                banque.dessiner("pieuvre_trainee", x, y - 10.0, false);
                banque.dessiner(nom_objet(c.objet), x, y, false);
            }
            Some(EtatChute::Bris(_)) => banque.dessiner(nom_eclats(c.objet), x, sol - 6.0, false),
            None => {}
        }
    }
}

/// LA MER de la phase 2, de sa surface `mer` (px du niveau) au bas de l'écran,
/// entre les deux murs de la tour : la bande dégradée (`pieuvre_mer0/1`, 56 px,
/// ses reflets bougent), puis l'eau de nuit ; chaque case solide sous l'eau est
/// redessinée en SILHOUETTE (la tranche de `pieuvre_mer_noyee` à sa profondeur),
/// un cran plus sombre que l'eau qui l'entoure. Devant les tuiles.
pub fn dessiner_mer(monde: &Monde, banque: &Banque, mer: f32, cx: i32, cy: i32) {
    let t = TAILLE_TUILE as i32;
    let s = mer.round() as i32;
    let bas = cy + HAUTEUR_LOGIQUE as i32;
    if s >= bas {
        return;
    }
    let (x0, x1) = (t, monde.niveau.largeur_px() as i32 - t);
    let nom = image(["pieuvre_mer0", "pieuvre_mer1"], monde.pas, 20);
    let mut x = x0;
    while x < x1 {
        let l = (x1 - x).min(LARGEUR_MER) as f32;
        let ecran = ((x - cx) as f32, (s - cy) as f32);
        banque.dessiner_partie(nom, ecran, (0.0, 0.0, l, BANDE_MER as f32), 0, false);
        x += LARGEUR_MER;
    }
    let mut y = s + BANDE_MER;
    while y < bas {
        for x in (x0..x1).step_by(t as usize) {
            let ecran = ((x - cx) as f32, (y - cy) as f32);
            banque.dessiner("pieuvre_mer_fond", ecran.0, ecran.1, false);
        }
        y += t;
    }
    // Les corniches noyées.
    for ty in s.div_euclid(t)..=bas.div_euclid(t) {
        for tx in x0 / t..x1 / t {
            if !monde.niveau.tuile_decor(tx, ty).est_solide() {
                continue;
            }
            let (haut, fin) = ((ty * t).max(s), (ty + 1) * t);
            let ex = (tx * t - cx) as f32;
            let limite = s + BANDE_MER;
            if haut < limite {
                let h = (fin.min(limite) - haut) as f32;
                let source = (0.0, (haut - s) as f32, t as f32, h);
                banque.dessiner_partie(
                    "pieuvre_mer_noyee",
                    (ex, (haut - cy) as f32),
                    source,
                    0,
                    false,
                );
            }
            if fin > limite {
                let debut = haut.max(limite);
                let source = (0.0, 0.0, t as f32, (fin - debut) as f32);
                let ecran = (ex, (debut - cy) as f32);
                banque.dessiner_partie("pieuvre_mer_noyee_fond", ecran, source, 0, false);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BANDE_MER;
    use crate::sprites::fichiers::dessin;

    /// Profondeur (px sous la surface) où finit la bande TURQUOISE de la mer : le
    /// cyan sombre (`E`) de `pieuvre_mer0/1`, juste sous l'écume.
    const FIN_TURQUOISE: usize = 24;

    /// Carte 9.36d (décision B d'AGA) : une corniche noyée juste sous l'écume ne
    /// fait plus de RECTANGLE UNI SOMBRE sur la mer (la « bande sombre » de la
    /// plongée de la Pieuvre). Dans la bande turquoise, sa silhouette est un
    /// turquoise plus sombre fait en damier (cyan sombre `E` et ardoise `2`) : sur
    /// aucune ligne, deux pixels voisins d'ardoise ou plus sombres (`0`, `1`, `2`).
    /// Elle reste VISIBLE : chaque ligne sous l'écume a de l'ardoise, que l'eau
    /// n'a pas. Plus bas, rien ne change (le dégradé un cran plus sombre).
    #[test]
    fn la_corniche_noyee_ne_fait_pas_de_bande_sombre_sous_l_ecume() {
        let noyee = dessin("pieuvre_mer_noyee");
        assert_eq!(
            noyee.len(),
            BANDE_MER as usize,
            "la silhouette suit la bande"
        );
        let sombre = |c: char| matches!(c, '0' | '1' | '2');
        for (j, ligne) in noyee.iter().take(FIN_TURQUOISE).enumerate() {
            let suite = ligne
                .chars()
                .fold((0, 0), |(s, m), c| {
                    let s = if sombre(c) { s + 1 } else { 0 };
                    (s, m.max(s))
                })
                .1;
            assert!(
                suite <= 1,
                "ligne {j} : {suite} px sombres de suite ({ligne})"
            );
            assert!(
                ligne.chars().all(|c| c == 'E' || c == '2'),
                "ligne {j} : turquoise sombre ou ardoise seulement ({ligne})"
            );
            if j >= 3 {
                assert!(
                    ligne.contains('2'),
                    "ligne {j} : la corniche se devine ({ligne})"
                );
            }
        }
        // L'eau autour, dans la même bande : turquoise (et ses reflets), jamais
        // d'ardoise ; la silhouette s'en distingue donc toujours.
        for nom in ["pieuvre_mer0", "pieuvre_mer1"] {
            for (j, ligne) in dessin(nom).iter().take(FIN_TURQUOISE).enumerate() {
                assert!(!ligne.chars().any(sombre), "{nom} ligne {j} : {ligne}");
            }
        }
        // Plus bas que la bande turquoise : la silhouette d'avant, un cran plus
        // sombre que l'eau (ardoise sur la nuit, puis l'ombre).
        assert_eq!(noyee[30], "1111111111111111");
        assert_eq!(noyee[BANDE_MER as usize - 1], "0000000000000000");
    }
}
