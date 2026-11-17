//! Rendu des mécaniques d'eau du Monde 2 (carte 9.3) : bulles, nénuphars,
//! marées (l'eau qui monte, sa graduation et sa flèche d'annonce), stalactites
//! et chutes d'eau. La scène (`scene.rs`) appelle `dessiner_marees` derrière les
//! tuiles, et `dessiner` pour chacune de ces entités, à sa place parmi les objets.
//!
//! Tout est en sprites de la palette (`sprites::objets_eau`, et les tuiles
//! d'eau du thème pour la marée) ; les mouvements (bulles qui montent, eau qui
//! coule) ne sont que des positions calculées à partir du numéro du pas.

use pepin_core::aabb::TAILLE_TUILE;
use pepin_core::entities::maree::PhaseMaree;
use pepin_core::entities::nenuphar::HAUT_NENUPHAR;
use pepin_core::entities::stalactite::EtatStalactite;
use pepin_core::entities::Entite;
use pepin_core::level::Tuile;

use crate::monde::Monde;
use crate::scene::{annonce_allumee, nom_tuile, PAS_VAGUE};
use crate::sprites::{objets_eau, Banque};

/// Écart (px) entre deux bulles d'une même colonne.
const ECART_BULLES: i32 = 12;
/// Vitesse de montée des bulles : 1 px par pas (60 px/s), comme le courant.
const MONTEE_BULLE_PAS: u32 = 1;

/// Image d'une animation de `n` images de `duree` pas chacune.
fn image(anim: &[&'static str], pas: u32, duree: u32) -> &'static str {
    anim[(pas / duree) as usize % anim.len()]
}

/// L'eau de chaque marée, derrière les tuiles : la graduation au bord gauche de
/// sa zone, puis l'eau jusqu'à sa surface du moment (tuiles d'eau du thème, la
/// surface ondule comme celle de l'eau du niveau). Ce qui déborde sous la zone
/// est recouvert par les tuiles du dessous (solide ou eau), dessinées ensuite.
pub fn dessiner_marees(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    let t = TAILLE_TUILE as i32;
    let theme = monde.infos.theme;
    for e in &monde.entites {
        let Entite::Maree(m) = e else { continue };
        let z = m.zone;
        let (x0, y0, bas) = (z.x as i32, z.y as i32, z.bas() as i32);
        for y in (y0..bas).step_by(t as usize) {
            banque.dessiner("graduation", (x0 - cx) as f32, (y - cy) as f32, false);
        }
        let Some(eau) = m.eau() else { continue };
        let surface = eau.zone.y.round() as i32;
        for x in (x0..z.droite() as i32).step_by(t as usize) {
            let mut y = bas - t;
            while y > surface {
                banque.dessiner_tuile(theme, "eau_calme_plein", (x - cx) as f32, (y - cy) as f32);
                y -= t;
            }
            let vague = (monde.pas / PAS_VAGUE) as i32 + x / t;
            let nom = if vague.rem_euclid(2) == 1 {
                "eau_calme_vague"
            } else {
                "eau_calme_haut"
            };
            banque.dessiner_tuile(theme, nom, (x - cx) as f32, (surface - cy) as f32);
        }
    }
}

/// La case d'eau `(tx, ty)` du niveau est-elle SOUS l'eau d'une marée (sa
/// surface n'est plus la surface) ? Alors elle se dessine « pleine ».
pub fn sous_une_maree(monde: &Monde, tx: i32, ty: i32) -> bool {
    let (x, y) = ((tx as f32 + 0.5) * TAILLE_TUILE, ty as f32 * TAILLE_TUILE);
    monde.entites.iter().any(|e| match e {
        Entite::Maree(m) => {
            m.eau().is_some()
                && m.zone.x < x
                && x < m.zone.droite()
                && (m.zone.bas() - y).abs() < 0.5
        }
        _ => false,
    })
}

/// Dessine l'entité d'eau `e` (sa boîte à l'écran : `(sx, sy)`) ; rien si `e`
/// n'est pas une mécanique d'eau.
pub fn dessiner(
    monde: &Monde,
    banque: &Banque,
    e: &Entite,
    (sx, sy): (f32, f32),
    cx: i32,
    cy: i32,
) {
    let pas = monde.pas;
    let t = TAILLE_TUILE as i32;
    match e {
        Entite::Bulles(b) => {
            let z = b.zone;
            let h = z.h as i32;
            for (k, x) in (z.x as i32..z.droite() as i32)
                .step_by(t as usize)
                .enumerate()
            {
                for i in 0..(h / ECART_BULLES).max(1) {
                    let monte = (pas * MONTEE_BULLE_PAS) as i32 + i * ECART_BULLES + k as i32 * 5;
                    let y = z.bas() as i32 - 5 - monte.rem_euclid(h);
                    let oscille = [0, 1, 1, 0][((pas / 8) as i32 + i).rem_euclid(4) as usize];
                    let (nom, dx) = if i % 2 == 0 {
                        (objets_eau::BULLES[0], 3)
                    } else {
                        (objets_eau::BULLES[1], 9)
                    };
                    banque.dessiner(nom, (x + dx + oscille - cx) as f32, (y - cy) as f32, false);
                }
            }
        }
        Entite::Chute(c) => {
            let z = c.zone;
            for x in (z.x as i32..z.droite() as i32).step_by(t as usize) {
                for y in (z.y as i32..z.bas() as i32).step_by(t as usize) {
                    banque.dessiner(
                        image(&objets_eau::CHUTE, pas, 4),
                        (x - cx) as f32,
                        (y - cy) as f32,
                        false,
                    );
                }
                let pied = z.bas() as i32 - 4;
                banque.dessiner(
                    image(&objets_eau::ECUME, pas, 8),
                    (x - cx) as f32,
                    (pied - cy) as f32,
                    false,
                );
            }
        }
        Entite::Nenuphar(n) => {
            // La fleur (4 rangées) dépasse au-dessus de la feuille.
            banque.dessiner("nenuphar", sx, sy - 4.0, false);
            // La partie sous l'eau disparaît : on redessine par-dessus la case
            // d'eau qui est sous la feuille (sa surface coupe la feuille).
            let tx = (n.boite.x / TAILLE_TUILE) as i32;
            let ty = ((n.repos + HAUT_NENUPHAR) / TAILLE_TUILE).round() as i32;
            let nom = nom_tuile(monde, tx, ty, Tuile::Eau);
            banque.dessiner_tuile(
                monde.infos.theme,
                nom,
                (tx * t - cx) as f32,
                (ty * t - cy) as f32,
            );
        }
        Entite::Stalactite(s) => match s.etat {
            EtatStalactite::Accrochee | EtatStalactite::Chute(_) => {
                banque.dessiner("stalactite", sx, sy, false);
            }
            EtatStalactite::Tremble(_) => {
                let dx = [0.0, 1.0, 0.0, -1.0][(pas / 2) as usize % 4];
                banque.dessiner("stalactite", sx + dx, sy, false);
            }
            EtatStalactite::Brisee(n) if n > 0 => {
                let sol = s.colonne.bas() as i32 - 4;
                banque.dessiner(
                    "stalactite_eclats",
                    (s.colonne.x as i32 - cx) as f32,
                    (sol - cy) as f32,
                    false,
                );
            }
            EtatStalactite::Brisee(_) => {}
        },
        Entite::Maree(m) => {
            // La flèche : fixe pendant le mouvement, clignotante pendant
            // l'annonce, rien sinon. Juste au-dessus de l'eau, aux deux bords.
            let fleche = match (m.phase(), m.annonce()) {
                (PhaseMaree::Montee, _) => Some(true),
                (PhaseMaree::Descente, _) => Some(false),
                (_, Some((monte, p))) if annonce_allumee(p) => Some(monte),
                _ => None,
            };
            if let Some(monte) = fleche {
                let nom = if monte {
                    "maree_monte"
                } else {
                    "maree_descend"
                };
                let y = (m.surface().round() as i32 - 9 - cy) as f32;
                for x in [m.zone.x as i32 + 5, m.zone.droite() as i32 - 12] {
                    banque.dessiner(nom, (x - cx) as f32, y, false);
                }
            }
        }
        _ => {}
    }
}
