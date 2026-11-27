//! Le MUR de la tour de la Pieuvre (carte 9.36b), derrière les corniches de la
//! phase 2.
//!
//! Décision d'AGA sur la planche de la 9.36b : la variante B (des arcades ouvertes
//! sur le port, au coucher du soleil), « mais avec un peu de A » (le mur de pierre
//! plein) : des PANS DE MUR PLEINS entre les arcades, avec leurs fenêtres éclairées
//! et leurs lanternes, surtout derrière la Pieuvre et les objets ; en montant, plus
//! d'arcades (la sensation de hauteur). Et l'orange vif de la B ne doit plus faire
//! concurrence à la Pieuvre ni aux objets : la vue dans les arcades est un
//! crépuscule FROID et sombre, la baie de droite (où la Pieuvre se tient au loin)
//! est toujours un pan plein sans couleur chaude, et chaque objet se détache du mur
//! avec un contraste d'au moins 3:1 (`tests`).
//!
//! ```text
//!   période (96 px) :  baie 0   baie 1   baie 2
//!   tour_haut         arcade   arcade   plein froid    en haut
//!   tour_milieu       arcade   plein    plein froid
//!   tour_bas          plein    arcade   plein froid    en bas
//! ```
//!
//! Les trois couches sont des fichiers (`assets/fonds/port/tour_*.png`, 288 x 96,
//! une période du mur entre les deux murs de côté ; sources : `docs/dessins-936.py`)
//! ; le code ne fait que les EMPILER au-dessus de la bâche de l'arène, une période
//! par 96 px, la couche choisie selon la hauteur (`couche_de_la_periode`). Toutes
//! partagent leurs rangées du haut (la pierre) et du bas (le bandeau de bois) : elles
//! se raccordent dans n'importe quel ordre. Le mur ne se dessine qu'en phase 2 : la
//! phase 1 (dans l'arène, sous la bâche) reste identique au pixel.
//!
//! Notion Rust : une fonction PURE (`couche_de_la_periode`) partagée par le rendu
//! et les tests, qui mesurent le contraste sur le mur entier ainsi assemblé.

use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use crate::dessin_pieuvre::la_pieuvre;
use crate::monde::Monde;

/// Les trois couches du mur (`assets/fonds/port/<nom>.png`).
pub const COUCHES: [&str; 3] = ["tour_bas", "tour_milieu", "tour_haut"];
/// Une couche : la largeur entre les deux murs de côté de la tour (18 tuiles)…
pub const L_MUR: i32 = 288;
/// … et la hauteur d'une période du mur (6 tuiles).
pub const H_MUR: i32 = 96;
/// Abscisse (px du niveau) du bord gauche du mur : après le mur de côté.
const X_MUR: i32 = 16;

/// La couche de la période `k` (0 = celle qui pose sur la bâche) parmi `n` : en
/// bas, une arcade sur trois (deux couches qui alternent la place de l'arcade) ;
/// au milieu, on alterne une et deux arcades ; en haut (et toujours la dernière
/// période), deux arcades sur trois. Pure.
pub fn couche_de_la_periode(k: i32, n: i32) -> &'static str {
    let [bas, milieu, haut] = COUCHES;
    let f = k as f32 / n.max(1) as f32;
    let pair = k % 2 == 0;
    if k >= n - 1 {
        // La plus haute : toujours ouverte (le sommet, le ciel).
        haut
    } else if f < 0.4 {
        if pair {
            bas
        } else {
            milieu
        }
    } else if f < 0.7 {
        if pair {
            milieu
        } else {
            haut
        }
    } else {
        haut
    }
}

/// Nombre de périodes du mur au-dessus de la bâche (son haut en `bache`, px du
/// niveau) : jusqu'en haut du niveau (la dernière dépasse au-dessus). Pure.
pub fn periodes(bache: i32) -> i32 {
    (bache.max(0) + H_MUR - 1) / H_MUR
}

/// Le mur, derrière les tuiles, pendant la phase 2 seulement (caméra `cx`, `cy`).
pub fn dessiner(monde: &Monde, cx: i32, cy: i32) {
    let Some(p) = la_pieuvre(monde).filter(|p| p.en_fuite()) else {
        return;
    };
    let Some(bache) = p.bache else {
        return;
    };
    // Le mur hors de l'écran (sur les côtés) : rien à poser.
    if X_MUR - cx + L_MUR <= 0 || X_MUR - cx >= LARGEUR_LOGIQUE as i32 {
        return;
    }
    let bas = bache.y.round() as i32;
    let n = periodes(bas);
    let h = HAUTEUR_LOGIQUE as i32;
    for k in 0..n {
        let y = bas - (k + 1) * H_MUR;
        if y + H_MUR <= cy || y >= cy + h {
            continue;
        }
        crate::fonds::dessiner("port", couche_de_la_periode(k, n), X_MUR - cx, y - cy);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pepin_core::palette::{Couleur, CORAIL, CORAIL_CLAIR, OR, ORANGE_SOLEIL, OR_PALE};

    /// Luminance relative (WCAG) d'une couleur de la palette.
    fn lum(c: Couleur) -> f64 {
        let lin = |v: u8| {
            let v = f64::from(v) / 255.0;
            if v <= 0.03928 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
    }

    fn contraste(a: f64, b: f64) -> f64 {
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    /// Le mur entier d'une tour de `n` périodes, comme le jeu l'empile (la
    /// période du haut en premier) : la luminance de chaque pixel.
    fn mur_entier(n: i32) -> Vec<Vec<f64>> {
        let mut rangs = Vec::new();
        for k in (0..n).rev() {
            let nom = couche_de_la_periode(k, n);
            for y in 0..H_MUR as usize {
                let rang = (0..L_MUR as usize)
                    .map(|x| {
                        let c = crate::fonds::couleur("port", nom, x, y);
                        lum(c.unwrap_or_else(|| panic!("{nom} ({x}, {y}) : transparent")))
                    })
                    .collect();
                rangs.push(rang);
            }
        }
        rangs
    }

    /// Luminance moyenne du REMPLISSAGE d'un sprite (sans son contour d'ombre `0`
    /// ni le transparent), et sa taille.
    fn remplissage(nom: &str) -> (f64, usize, usize) {
        let image = crate::sprites::tous()
            .into_iter()
            .find(|(n, _)| *n == nom)
            .unwrap_or_else(|| panic!("{nom} : aucun sprite"))
            .1;
        let (mut somme, mut nb) = (0.0, 0);
        for ligne in image {
            for c in ligne.chars().filter(|&c| c != '.' && c != '0') {
                somme += lum(crate::sprites::pixel(c).unwrap().unwrap());
                nb += 1;
            }
        }
        (somme / f64::from(nb), image[0].len(), image.len())
    }

    /// La demande d'AGA (27/11, sur la planche) : « contraste d'au moins 3:1 entre
    /// chaque objet, son repère et le fond qui est derrière ». Les objets tombent
    /// dans TOUTES les colonnes de la tour, à toutes les hauteurs : on mesure donc
    /// partout. Pour chaque place possible de l'objet sur le mur assemblé (pixel par
    /// pixel, raccords compris), le fond derrière lui (luminance moyenne du mur sous
    /// sa boîte) contre son remplissage : au moins 3:1. Les lumières (fenêtres,
    /// lanternes, le fil de corail de l'horizon) sont assez petites pour ne jamais
    /// faire passer une place sous ce seuil.
    #[test]
    fn chaque_objet_et_son_repere_se_detachent_du_mur() {
        let objets = [
            "pieuvre_caisse",
            "pieuvre_tonneau",
            "pieuvre_epave",
            "pieuvre_repere0",
            "pieuvre_repere1",
        ];
        let mut pire = f64::MAX;
        for n in 3..=16 {
            let mur = mur_entier(n);
            let (lh, lw) = (mur.len(), L_MUR as usize);
            // Table des sommes (sommes cumulées) : la moyenne de n'importe quel
            // rectangle en quatre lectures.
            let mut cumul = vec![vec![0.0f64; lw + 1]; lh + 1];
            for y in 0..lh {
                for x in 0..lw {
                    cumul[y + 1][x + 1] =
                        mur[y][x] + cumul[y][x + 1] + cumul[y + 1][x] - cumul[y][x];
                }
            }
            for nom in objets {
                let (objet, w, h) = remplissage(nom);
                for y in 0..=lh - h {
                    for x in 0..=lw - w {
                        let somme =
                            cumul[y + h][x + w] - cumul[y][x + w] - cumul[y + h][x] + cumul[y][x];
                        let fond = somme / (w * h) as f64;
                        let c = contraste(objet, fond);
                        pire = pire.min(c);
                        assert!(
                            c >= 3.0,
                            "{nom} sur le mur de {n} périodes en ({x}, {y}) : {c:.2}:1"
                        );
                    }
                }
            }
        }
        eprintln!("contraste le plus faible objet/mur : {pire:.2}:1");
    }

    /// « Pas d'orange vif juste derrière la Pieuvre » : la Pieuvre au loin (sa tête
    /// en x = 252 dans la tour de 20 tuiles, son bras 16 px à gauche) ne passe
    /// que devant la baie de droite, qui n'a AUCUNE couleur chaude, sur chaque
    /// couche (et une marge de 16 px).
    #[test]
    fn rien_de_chaud_derriere_la_pieuvre() {
        let debut = (252 - 16 - 16 - X_MUR) as usize;
        for nom in COUCHES {
            for y in 0..H_MUR as usize {
                for x in debut..L_MUR as usize {
                    let c = crate::fonds::couleur("port", nom, x, y).unwrap();
                    assert!(
                        ![CORAIL, CORAIL_CLAIR, ORANGE_SOLEIL, OR, OR_PALE].contains(&c),
                        "{nom} ({x}, {y}) : {c:?} derrière la Pieuvre"
                    );
                }
            }
        }
    }

    /// Les couches se raccordent dans n'importe quel ordre : mêmes rangées du haut
    /// (la pierre, jusqu'au cintre des arcades) et du bas (le bandeau de bois).
    #[test]
    fn les_couches_du_mur_se_raccordent() {
        let rang = |nom: &str, y: usize| crate::fonds::grille("port", nom)[y].clone();
        for nom in COUCHES {
            assert_eq!(crate::fonds::taille("port", nom), (L_MUR, H_MUR), "{nom}");
            for y in (0..4).chain(88..H_MUR as usize) {
                assert_eq!(rang(nom, y), rang(COUCHES[0], y), "{nom}, rangée {y}");
            }
        }
    }

    /// Plus de mur plein en bas, plus d'arcades en haut ; les trois couches servent
    /// dans la vraie tour (la bâche en y = 1 088 : 12 périodes).
    #[test]
    fn plus_d_arcades_en_montant() {
        let arcades = |nom: &str| match nom {
            "tour_bas" | "tour_milieu" => 1,
            _ => 2,
        };
        assert_eq!(periodes(1088), 12);
        for n in 3..=16 {
            assert_eq!(couche_de_la_periode(0, n), "tour_bas");
            assert_eq!(couche_de_la_periode(n - 1, n), "tour_haut");
            let moitie = |r: std::ops::Range<i32>| {
                r.map(|k| arcades(couche_de_la_periode(k, n))).sum::<i32>()
            };
            assert!(moitie(0..n / 2) < moitie(n - n / 2..n), "{n} périodes");
        }
        for nom in COUCHES {
            assert!((0..12).any(|k| couche_de_la_periode(k, 12) == nom), "{nom}");
        }
    }
}
