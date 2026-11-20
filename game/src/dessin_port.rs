//! « La Pieuvre du port » se lit comme le BOUT D'UNE JETÉE au milieu du port
//! (carte 9.20e). Rendu seulement : la grille, la collision et le combat ne
//! changent pas.
//!
//! - Les deux murs de l'arène (colonnes du bord) restent solides, mais se
//!   dessinent en QUAIS : la pierre du quai, son anneau et le cordage qui pend
//!   jusqu'à l'eau, la pierre mouillée au niveau du bassin ; au-dessus du quai, à
//!   gauche la tour en treillis d'une grue (on voit le ciel et la mer entre ses
//!   barres), à droite une pile de caisses sanglées.
//! - Au fond, le port : l'horizon de la mer, le phare rayé sur son îlot (son feu
//!   clignote et se reflète dans l'eau), un chalutier, une barque et un canot
//!   amarrés, en silhouettes sombres (rien de clair sur le dessus : ce ne sont
//!   pas des plateformes).
//! - Le bassin est l'eau PROFONDE du port d'où sort la Pieuvre : la surface
//!   garde la bande cyan des tuiles d'eau (l'ombre de la pieuvre, son annonce, s'y
//!   lit toujours), l'eau en dessous est d'ardoise de nuit, avec les reflets du
//!   phare.
//!
//! Seul le niveau de clé `pieuvre` est concerné : le niveau d'essai
//! `essai_pieuvre` (build de QA) garde le rendu du thème.

use macroquad::prelude::draw_rectangle;
use pepin_core::aabb::TAILLE_TUILE;
use pepin_core::level::{Niveau, Tuile};
use pepin_core::palette::{
    Couleur, ARDOISE, ARDOISE_NUIT, ARDOISE_OMBRE, ARDOISE_PALE, OR, OR_PALE,
};
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use crate::monde::Monde;
use crate::rendu::couleur_mq;
use crate::scene::PAS_VAGUE;
use crate::sprites::Banque;

/// Clé du niveau dessiné en jetée.
pub const CLE_JETEE: &str = "pieuvre";
/// Ordonnée (px du niveau) de l'horizon de la mer : au-dessus du platelage (le
/// dessus de la rangée 6, y = 96), la mer se voit derrière la jetée.
pub const HORIZON: i32 = 80;
/// Abscisse (px du niveau) de l'axe du phare.
pub const X_PHARE: i32 = 250;
/// Haut du phare (px du niveau) ; son dessin fait 48 px, son pied touche l'îlot.
const HAUT_PHARE: i32 = 30;
/// Le feu du phare : allumé une période sur deux (comme le phare du thème).
const PERIODE_FEU: u32 = 60;
/// Les bateaux amarrés : (sprite, x, ligne de flottaison, hauteur du dessin), du
/// plus proche (le plus bas) au plus lointain. Tous flottent sous l'horizon et
/// au-dessus du platelage, entre les bittes d'amarrage.
pub const BATEAUX: [(&str, i32, i32, i32); 3] = [
    ("jetee_chalutier", 86, 92, 26),
    ("jetee_barque", 178, 88, 14),
    ("jetee_canot", 268, 85, 5),
];

/// Le niveau de `monde` est-il la jetée de la Pieuvre ?
pub fn est_la_jetee(monde: &Monde) -> bool {
    monde.infos.cle == CLE_JETEE
}

/// Ce que la jetée pose dans une case du bord de l'arène.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    /// Une tuile du jeu du port (`por_quai`, `por_anneau`, `por_pierre`).
    Tuile(&'static str),
    /// Un sprite de la jetée (`sprites::jetee`), en miroir sur le bord droit.
    Sprite(&'static str),
}

/// La pièce du bord (`gauche` ou droit) à la rangée `ty`. `quai` : la rangée du
/// dessus du quai (celle du platelage) ; `eau` : la case voisine, dans l'arène,
/// est de l'eau (le niveau du bassin). Pure.
pub fn piece_du_bord(gauche: bool, ty: i32, quai: i32, eau: bool) -> Piece {
    if ty < quai {
        // Au-dessus du quai : ce qui monte haut pour fermer l'arène.
        if !gauche {
            return if (quai - 1 - ty) % 2 == 0 {
                Piece::Sprite("jetee_caisse_ocre")
            } else {
                Piece::Sprite("jetee_caisse_violette")
            };
        }
        return Piece::Sprite(if ty == quai - 1 {
            "jetee_grue_pied"
        } else if ty == quai - 5 {
            "jetee_grue_cabine"
        } else {
            "jetee_grue"
        });
    }
    if eau {
        Piece::Sprite("jetee_quai_mouille")
    } else if ty == quai {
        Piece::Tuile("por_quai")
    } else if ty == quai + 1 {
        Piece::Tuile("por_anneau")
    } else {
        Piece::Tuile("por_pierre")
    }
}

/// La rangée du dessus du quai au bord dont la colonne voisine (dans l'arène) est
/// `voisin` : la première rangée où cette colonne est solide (le platelage s'y
/// raccorde). Pure.
pub fn rangee_du_quai(niveau: &Niveau, voisin: i32) -> i32 {
    let h = niveau.hauteur() as i32;
    (0..h)
        .find(|&ty| niveau.tuile_decor(voisin, ty).est_solide())
        .unwrap_or(h)
}

/// La case `(tx, ty)` (tuile `t`) est-elle dessinée par la jetée plutôt que par
/// le jeu de tuiles du thème : l'eau, et les bords de l'arène au-dessus de la
/// dernière rangée (le fond, qui reste au thème). Pure.
pub fn case_de_la_jetee(niveau: &Niveau, tx: i32, ty: i32, t: Tuile) -> bool {
    let (l, h) = (niveau.largeur() as i32, niveau.hauteur() as i32);
    t == Tuile::Eau || ((tx == 0 || tx == l - 1) && ty < h - 1 && t != Tuile::Vide)
}

/// Pour `scene::nom_tuile` : la jetée dessine-t-elle elle-même cette case ?
pub fn dessine_la_case(monde: &Monde, tx: i32, ty: i32, t: Tuile) -> bool {
    est_la_jetee(monde) && case_de_la_jetee(&monde.niveau, tx, ty, t)
}

/// Le feu du phare est-il allumé au pas `pas` ? Pure.
pub fn feu_allume(pas: u32) -> bool {
    (pas / PERIODE_FEU).is_multiple_of(2)
}

/// Clé du niveau « Le port » (carte 9.11) : ses plateformes mobiles sont des
/// bateaux et une caisse de grue.
pub const CLE_PORT: &str = "port";

/// Sprite d'une plateforme mobile en aller-retour du port, et son décalage
/// vertical (px) au-dessus de sa boîte : un trajet horizontal est un BATEAU (le
/// pont sur la boîte, la coque dessous), un trajet vertical la CAISSE de la grue
/// (ses élingues au-dessus). `None` hors du port. Pure.
pub fn sprite_du_port(cle: &str, a: (f32, f32), b: (f32, f32)) -> Option<(&'static str, i32)> {
    if cle != CLE_PORT {
        return None;
    }
    Some(if a.0 == b.0 {
        ("port_caisse_grue", 8)
    } else {
        ("port_bateau", 0)
    })
}

/// Dessine une plateforme mobile du port (boîte dessinée en `(sx, sy)`, écran) :
/// le bateau, ou la caisse de la grue et son câble qui monte jusqu'en haut de
/// l'écran (la flèche de la grue est au-dessus). Faux hors du port.
pub fn dessiner_mobile(
    monde: &Monde,
    banque: &Banque,
    extremites: Option<((f32, f32), (f32, f32))>,
    sx: f32,
    sy: f32,
) -> bool {
    let Some((nom, haut)) = extremites.and_then(|(a, b)| sprite_du_port(monde.infos.cle, a, b))
    else {
        return false;
    };
    let y = sy - haut as f32;
    if haut > 0 {
        // Le câble (2 px, ardoise d'ombre) du crochet jusqu'au haut de l'écran.
        draw_rectangle(sx + 15.0, 0.0, 2.0, y.max(0.0), couleur_mq(ARDOISE_OMBRE));
    }
    banque.dessiner(nom, sx, y, false);
    true
}

fn rect(x: i32, y: i32, w: i32, h: i32, c: Couleur) {
    draw_rectangle(x as f32, y as f32, w as f32, h as f32, couleur_mq(c));
}

/// Le port au fond, par-dessus le fond du thème et derrière les tuiles : la mer
/// et son horizon, le phare sur son îlot, les bateaux amarrés.
pub fn dessiner_fond(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    if !est_la_jetee(monde) {
        return;
    }
    let pas = monde.pas;
    let (l, hh) = (LARGEUR_LOGIQUE as i32, HAUTEUR_LOGIQUE as i32);
    let horizon = HORIZON - cy;
    // La mer, jusqu'au bas de l'écran (on la voit sous la jetée, entre les piles).
    rect(0, horizon, l, hh - horizon, ARDOISE_NUIT);
    // La houle : des traits d'ardoise qui dérivent, plus serrés près de l'horizon.
    for k in 0..20i32 {
        let y = horizon + 2 + k * (k + 4) / 5;
        let sens = if k % 2 == 0 { 1 } else { -1 };
        let x = (k * 53 + sens * (pas / (9 + (k % 3) as u32 * 4)) as i32 - cx).rem_euclid(l);
        rect(x, y, 3 + k % 4, 1, ARDOISE);
    }
    rect(0, horizon, l, 1, ARDOISE_PALE);
    // Le phare sur son îlot de rochers.
    let xp = X_PHARE - cx;
    banque.dessiner(
        "jetee_phare",
        (xp - 6) as f32,
        (HAUT_PHARE - cy) as f32,
        false,
    );
    rect(xp - 13, horizon - 3, 26, 5, ARDOISE_OMBRE);
    rect(xp - 9, horizon - 5, 16, 2, ARDOISE_OMBRE);
    let allume = feu_allume(pas);
    if allume {
        let yl = HAUT_PHARE + 3 - cy;
        rect(xp - 2, yl, 4, 4, OR);
        rect(xp - 1, yl + 1, 2, 2, OR_PALE);
        rect(xp - 34, yl + 2, 30, 1, OR_PALE);
        rect(xp + 4, yl + 2, 30, 1, OR_PALE);
    }
    // Son reflet sur la mer, entre l'îlot et la jetée : trois traits qui tremblent,
    // or pâle seulement (l'or franc reste aux gouttes).
    for k in 0..3i32 {
        let y = horizon + 4 + k * 4;
        let w = 7 - k * 2;
        let dx = [0, 1, 0, -1][((pas / 10) as i32 + k).rem_euclid(4) as usize];
        let c = if allume { OR_PALE } else { ARDOISE };
        rect(xp - w / 2 + dx, y, w, 1, c);
    }
    // Les bateaux amarrés, du plus lointain au plus proche ; ils se balancent d'un
    // pixel sur la houle, chacun à son rythme.
    for (k, &(nom, x, flottaison, h)) in BATEAUX.iter().enumerate().rev() {
        let houle = [0, 0, 1, 1][((pas / (24 + k as u32 * 7)) % 4) as usize];
        let y = flottaison - h + 1 + houle - cy;
        banque.dessiner(nom, (x - cx) as f32, y as f32, false);
    }
}

/// La jetée elle-même, juste après les tuiles du thème : le bassin profond et ses
/// reflets, puis les quais des deux bords (grue, caisses, pierre, cordages).
pub fn dessiner_jetee(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    if !est_la_jetee(monde) {
        return;
    }
    let niveau = &monde.niveau;
    let t = TAILLE_TUILE as i32;
    let (l, h) = (niveau.largeur() as i32, niveau.hauteur() as i32);
    let ecran = |tx: i32, ty: i32| ((tx * t - cx) as f32, (ty * t - cy) as f32);
    // Le bassin : la surface ondule comme l'eau du jeu (même rythme).
    for ty in 0..h {
        for tx in 0..l {
            if niveau.tuile_decor(tx, ty) != Tuile::Eau {
                continue;
            }
            let surface = niveau.tuile_decor(tx, ty - 1) == Tuile::Vide;
            let vague = ((monde.pas / PAS_VAGUE) as i32 + tx).rem_euclid(2) == 1;
            let nom = match (surface, vague) {
                (false, _) => "jetee_eau_plein",
                (true, false) => "jetee_eau_haut",
                (true, true) => "jetee_eau_vague",
            };
            let (x, y) = ecran(tx, ty);
            banque.dessiner(nom, x, y, false);
            // Les reflets du phare, sous son axe, quand le feu est allumé : dans l'eau
            // profonde, sous la bande où s'annoncent les tentacules ; or pâle seulement.
            if surface && tx == X_PHARE / t && feu_allume(monde.pas) {
                for k in 0..4i32 {
                    let w = 8 - 2 * k;
                    let dx = [0, 1, 0, -1][((monde.pas / 8) as i32 + k).rem_euclid(4) as usize];
                    rect(
                        X_PHARE - cx - w / 2 + dx,
                        y as i32 + 7 + 2 * k,
                        w,
                        1,
                        OR_PALE,
                    );
                }
            }
        }
    }
    // Les quais des deux bords.
    for (gauche, tx, voisin) in [(true, 0, 1), (false, l - 1, l - 2)] {
        let quai = rangee_du_quai(niveau, voisin);
        let mut anneau = None;
        for ty in 0..h - 1 {
            let tuile = niveau.tuile_decor(tx, ty);
            if !case_de_la_jetee(niveau, tx, ty, tuile) || tuile == Tuile::Eau {
                continue;
            }
            let eau = niveau.tuile_decor(voisin, ty) == Tuile::Eau;
            let (x, y) = ecran(tx, ty);
            match piece_du_bord(gauche, ty, quai, eau) {
                Piece::Tuile(nom) => banque.dessiner_tuile(monde.infos.theme, nom, x, y),
                Piece::Sprite(nom) => banque.dessiner(nom, x, y, !gauche),
            }
            if piece_du_bord(gauche, ty, quai, eau) == Piece::Tuile("por_anneau") {
                anneau = Some((x, y));
            }
        }
        // Le cordage pend de l'anneau jusque dans l'eau (dessiné après la pierre
        // mouillée, qu'il traverse).
        if let Some((x, y)) = anneau {
            banque.dessiner("jetee_cordage", x, y + 11.0, !gauche);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sprites::jetee;
    use crate::sprites::tuiles_monde2::port;
    use pepin_core::levels::{essai_pieuvre, pieuvre};

    fn nom(p: Piece) -> &'static str {
        match p {
            Piece::Tuile(n) | Piece::Sprite(n) => n,
        }
    }

    /// Les bords de l'arène de la Pieuvre : le quai s'aligne sur le platelage
    /// (rangée 6), la grue à gauche (pied sur le quai, cabine en haut), la pile de
    /// caisses à droite (couleurs alternées), la pierre du quai, son anneau, la
    /// pierre mouillée au niveau du bassin.
    #[test]
    fn les_bords_de_l_arene_sont_des_quais_avec_une_grue_et_des_caisses() {
        let n = pieuvre::INFOS.charger().unwrap();
        let (l, h) = (n.largeur() as i32, n.hauteur() as i32);
        assert_eq!(rangee_du_quai(&n, 1), 6);
        assert_eq!(rangee_du_quai(&n, l - 2), 6);
        let gauche: Vec<_> = (0..h - 1)
            .map(|ty| {
                nom(piece_du_bord(
                    true,
                    ty,
                    6,
                    n.tuile_decor(1, ty) == Tuile::Eau,
                ))
            })
            .collect();
        assert_eq!(
            gauche,
            [
                "jetee_grue",
                "jetee_grue_cabine",
                "jetee_grue",
                "jetee_grue",
                "jetee_grue",
                "jetee_grue_pied",
                "por_quai",
                "por_anneau",
                "por_pierre",
                "jetee_quai_mouille",
            ]
        );
        let droite: Vec<_> = (0..6)
            .map(|ty| nom(piece_du_bord(false, ty, 6, false)))
            .collect();
        assert_eq!(droite[5], "jetee_caisse_ocre");
        assert_eq!(droite[4], "jetee_caisse_violette");
        assert!(droite.windows(2).all(|p| p[0] != p[1]));
    }

    /// La jetée ne change que le RENDU : chaque case qu'elle dessine au bord est
    /// solide dans la grille (les murs restent des limites), toute l'eau du bassin
    /// est à elle, et la rangée du fond reste au thème.
    #[test]
    fn la_jetee_dessine_les_murs_solides_et_le_bassin_sans_toucher_la_grille() {
        let n = pieuvre::INFOS.charger().unwrap();
        let (l, h) = (n.largeur() as i32, n.hauteur() as i32);
        for tx in [0, l - 1] {
            for ty in 0..h - 1 {
                let t = n.tuile_decor(tx, ty);
                assert!(t.est_solide(), "case ({tx}, {ty})");
                assert!(case_de_la_jetee(&n, tx, ty, t));
            }
            assert!(!case_de_la_jetee(&n, tx, h - 1, n.tuile_decor(tx, h - 1)));
        }
        let eaux = (1..l - 1)
            .filter(|&tx| n.tuile_decor(tx, 9) == Tuile::Eau)
            .count();
        assert_eq!(eaux, (l - 2) as usize, "le bassin couvre toute la rangée 9");
        assert!(
            !case_de_la_jetee(&n, 1, 6, n.tuile_decor(1, 6)),
            "le platelage reste au thème"
        );
        // Le niveau d'essai garde l'ancien rendu.
        assert_ne!(essai_pieuvre::INFOS.cle, CLE_JETEE);
        assert_eq!(pieuvre::INFOS.cle, CLE_JETEE);
    }

    /// Chaque pièce existe : les sprites dans `sprites::jetee`, les tuiles dans le
    /// jeu du port.
    #[test]
    fn chaque_piece_du_bord_existe() {
        let sprites: Vec<_> = jetee::SPRITES.iter().map(|(n, _)| *n).collect();
        let tuiles: Vec<_> = port::SPRITES.iter().map(|(n, _)| *n).collect();
        for gauche in [true, false] {
            for ty in 0..11 {
                for eau in [false, true] {
                    match piece_du_bord(gauche, ty, 6, eau) {
                        Piece::Sprite(n) => assert!(sprites.contains(&n), "{n}"),
                        Piece::Tuile(n) => assert!(tuiles.contains(&n), "{n}"),
                    }
                }
            }
        }
        for (n, ..) in BATEAUX {
            assert!(sprites.contains(&n), "{n}");
        }
        for n in [
            "jetee_cordage",
            "jetee_phare",
            "jetee_eau_haut",
            "jetee_eau_vague",
        ] {
            assert!(sprites.contains(&n), "{n}");
        }
    }

    /// Le bassin est plus SOMBRE que l'eau du thème, mais l'annonce d'un tentacule
    /// s'y lit toujours : l'ombre de la pieuvre (4 rangées sous la surface) tombe
    /// sur la bande cyan, et l'eau profonde n'a que de l'ardoise de nuit (1, 2).
    #[test]
    fn le_bassin_est_profond_et_garde_la_bande_ou_se_lit_l_annonce() {
        let haut = jetee::EAU_HAUT;
        assert!(
            haut[0].chars().all(|c| c == 'D'),
            "la ligne claire de la surface"
        );
        for rangee in &haut[1..=4] {
            assert!(rangee.chars().all(|c| c == 'D' || c == 'E'), "{rangee}");
        }
        for rangee in haut[6..].iter().chain(jetee::EAU_PLEIN.iter()) {
            assert!(rangee.chars().all(|c| c == '1' || c == '2'), "{rangee}");
        }
        // L'eau du thème (Monde 1, éclaircie d'un cran au port) avait un fond d'ardoise
        // claire (3) : le bassin est plus sombre.
        let theme = crate::sprites::tuiles::EAU_CALME_PLEIN;
        assert!(theme.iter().any(|r| r.contains('2')));
    }

    /// Le fond du port : l'horizon est au-dessus du platelage (on voit la mer
    /// derrière la jetée), les bateaux flottent entre l'horizon et le platelage,
    /// le phare tient sur son îlot à l'horizon.
    #[test]
    fn la_mer_et_les_bateaux_se_voient_au_dessus_du_platelage() {
        let platelage = 6 * TAILLE_TUILE as i32;
        assert!(HORIZON < platelage);
        for (n, x, flottaison, h) in BATEAUX {
            assert!(flottaison > HORIZON && flottaison < platelage, "{n}");
            let dessin = jetee::SPRITES.iter().find(|(m, _)| *m == n).unwrap().1;
            assert_eq!(dessin.len() as i32, h, "{n}");
            assert!(
                x >= 16 && x + crate::sprites::largeur(dessin) as i32 <= 304,
                "{n}"
            );
        }
        assert_eq!(HAUT_PHARE + jetee::PHARE.len() as i32, HORIZON - 2);
    }

    /// Les bateaux sont des SILHOUETTES sombres (ombre de contour, quelques
    /// hublots) : rien de clair sur leur dessus, ils ne se lisent pas comme des
    /// plateformes.
    #[test]
    fn les_bateaux_sont_des_silhouettes_sombres() {
        for (n, ..) in BATEAUX {
            let dessin = jetee::SPRITES.iter().find(|(m, _)| *m == n).unwrap().1;
            let pixels: Vec<char> = dessin
                .iter()
                .flat_map(|r| r.chars())
                .filter(|&c| c != '.')
                .collect();
            // L'ombre de contour, et l'ardoise de la voile ferlée.
            let sombres = pixels.iter().filter(|&&c| c == '0' || c == '2').count();
            assert!(
                sombres * 10 >= pixels.len() * 9,
                "{n} : {sombres} / {}",
                pixels.len()
            );
            assert!(pixels.iter().all(|c| "028C".contains(*c)), "{n}");
        }
    }

    #[test]
    fn le_feu_du_phare_clignote_une_periode_sur_deux() {
        assert!(feu_allume(0) && feu_allume(PERIODE_FEU - 1));
        assert!(!feu_allume(PERIODE_FEU) && feu_allume(2 * PERIODE_FEU));
    }

    /// Carte 9.11 : au port, chaque plateforme mobile horizontale est un bateau,
    /// la verticale la caisse de la grue ; leur dessin couvre toute la collision
    /// (32 x 8, sous le décalage), clair sur le dessus. Ailleurs, rien ne change.
    #[test]
    fn au_port_les_plateformes_mobiles_sont_des_bateaux_et_une_caisse_de_grue() {
        use pepin_core::entities::Entite;
        use pepin_core::levels::port as niveau_port;
        let n = niveau_port::INFOS.charger().unwrap();
        let mut noms = Vec::new();
        for e in &n.entites {
            let Entite::Mobile(m) = e else { continue };
            let (a, b) = m.extremites().unwrap();
            let (nom, haut) = sprite_du_port(CLE_PORT, a, b).unwrap();
            let dessin = jetee::SPRITES.iter().find(|(m, _)| *m == nom).unwrap().1;
            assert_eq!(dessin[0].len() as f32, m.boite.w, "{nom}");
            let collision = dessin.iter().skip(haut as usize).take(m.boite.h as usize);
            for (y, rangee) in collision.enumerate() {
                assert!(!rangee.contains('.'), "{nom} : rangée {y} percée");
            }
            assert!(
                dessin[haut as usize][1..31].chars().all(|c| c == 'C'),
                "{nom}"
            );
            noms.push(nom);
        }
        let bateaux = noms.iter().filter(|n| **n == "port_bateau").count();
        let caisses = noms.iter().filter(|n| **n == "port_caisse_grue").count();
        assert_eq!((bateaux, caisses), (6, 2), "carte 9.11d");
        assert_eq!(sprite_du_port("pieuvre", (0.0, 0.0), (16.0, 0.0)), None);
    }
}
