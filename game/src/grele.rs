//! Dessin de la grêle (carte 5.4) : dans chaque colonne `Grele` du niveau, des
//! grêlons tombent sans fin, en deux files, et éclatent sur ce qui les arrête : le
//! toit au bas de la colonne, ou le parapluie OUVERT de Pépin quand il plane
//! dessous (on voit la grêle rebondir sur la toile : c'est la preuve visuelle
//! qu'il est protégé). Au-dessus d'un vide, rien ne les arrête : ils sortent de
//! la carte.
//!
//! Aucune mémoire : la place de chaque grêlon se CALCULE à partir du numéro du pas
//! (`Monde::pas`). Le même pas donne toujours la même image : le rendu est
//! reproductible (captures, rejeu du robot) et `disposer` se teste sans fenêtre.
//!
//! Notions Rust illustrées ici : une fonction pure qui renvoie un `Vec` de petits
//! `enum` (ce qu'il faut dessiner, sans le dessiner), puis une fonction séparée qui
//! dessine ; `rem_euclid` (reste toujours positif) ; `Option<Aabb>` pour « un
//! parapluie ouvert, ou aucun ».

use pepin_core::aabb::Aabb;

use crate::sprites::{objets, Banque};

/// Vitesse de chute d'un grêlon : 3 px par pas (180 px/s), bien plus vite que la
/// pluie du décor : on ne les confond pas.
pub const VITESSE: i32 = 3;
/// Écart vertical entre deux grêlons d'une même file (px).
pub const ECART: i32 = 20;
/// Côté d'un grêlon (px) : le sprite `grelon*` fait 4 x 4.
pub const TAILLE: i32 = 4;
/// Un grêlon arrêté reste visible sous forme d'éclat pendant ses `ECLAT` derniers
/// pixels de chute (2 pas).
const ECLAT: i32 = 2 * VITESSE;
/// Les deux files d'une colonne de 12 px : décalage horizontal dans la colonne et
/// déphasage vertical (la seconde file tombe entre les grêlons de la première).
const FILES: [(i32, i32); 2] = [(1, 0), (7, ECART / 2)];
/// Un pas sur 4, le reflet du grêlon avance d'un coin (il roule).
const PAS_PAR_IMAGE: u32 = 4;

/// Un morceau de grêle à dessiner, coin haut-gauche en pixels du MONDE.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Morceau {
    /// Un grêlon qui tombe, avec son image (0 à 3 : `objets::GRELON`).
    Grelon { x: i32, y: i32, image: usize },
    /// Un grêlon qui se brise sur ce qui l'arrête (`grele_eclat`, 7 x 3, centré sur
    /// sa file : les miettes débordent un peu de chaque côté).
    Eclat { x: i32, y: i32 },
}

/// Où est le parapluie ouvert de Pépin : la boîte de son sprite de plané
/// (20 x 20, toile en haut), à partir du coin haut-gauche de la boîte de Pépin
/// (10 x 14) — même calcul que `scene::dessiner_pepin`.
pub fn parapluie(x: f32, y: f32) -> Aabb {
    Aabb::new(x - 5.0, y - 6.0, 20.0, 20.0)
}

/// Tous les morceaux de la colonne `zone` au pas `pas`. `sol` : un solide arrête la
/// colonne en bas (sinon elle tombe dans le vide, sans éclat). `abri` : le parapluie
/// ouvert de Pépin s'il plane (voir `parapluie`), qui arrête les grêlons des files
/// qu'il couvre ; ceux qui sont déjà passés sous lui continuent de tomber.
pub fn disposer(zone: Aabb, pas: u32, sol: bool, abri: Option<Aabb>) -> Vec<Morceau> {
    let (x0, haut, hauteur) = (zone.x as i32, zone.y as i32, zone.h as i32);
    // Chute cumulée, modulo l'écart : (pas x VITESSE) mod ECART, sans jamais
    // déborder d'un entier même après des heures de jeu.
    let chute = (pas % ECART as u32) as i32 * VITESSE;
    let mut morceaux = Vec::new();
    for (f, &(dx, dephasage)) in FILES.iter().enumerate() {
        let x = x0 + dx;
        // Le parapluie couvre-t-il cette file, à une hauteur où il y a de la grêle ?
        // Alors la toile (`toile`) et le corps de Pépin dessous (jusqu'à `pieds`)
        // forment un trou dans la file.
        let ombre = abri.filter(|p| {
            (p.x as i32) < x + TAILLE
                && x < p.droite() as i32
                && (p.y as i32) < haut + hauteur
                && (p.bas() as i32) > haut
        });
        // Premier grêlon de la file : entre 0 et ECART px sous le haut de la colonne.
        let premier = (chute + dephasage).rem_euclid(ECART);
        for (k, y_rel) in (premier..).step_by(ECART as usize).enumerate() {
            let y = haut + y_rel;
            let image = ((pas / PAS_PAR_IMAGE) as usize + k + f) % objets::GRELON.len();
            if let Some(p) = ombre {
                let (toile, pieds) = (p.y as i32, p.bas() as i32);
                let enfonce = y + TAILLE - toile;
                if enfonce > 0 && y < pieds {
                    // Arrêté par la toile : éclat posé dessus, puis plus rien.
                    if enfonce <= ECLAT && toile >= haut {
                        morceaux.push(Morceau::Eclat {
                            x: x - 2,
                            y: toile - 3,
                        });
                    }
                    continue;
                }
            }
            let enfonce = y_rel + TAILLE - hauteur;
            if enfonce <= 0 {
                morceaux.push(Morceau::Grelon { x, y, image });
            } else {
                if sol && enfonce <= ECLAT {
                    morceaux.push(Morceau::Eclat {
                        x: x - 2,
                        y: haut + hauteur - 3,
                    });
                }
                break;
            }
        }
    }
    morceaux
}

/// Dessine les morceaux d'une colonne ; `(cx, cy)` : caméra entière.
pub fn dessiner(morceaux: &[Morceau], banque: &Banque, cx: i32, cy: i32) {
    for m in morceaux {
        match *m {
            Morceau::Grelon { x, y, image } => banque.dessiner(
                objets::GRELON[image],
                (x - cx) as f32,
                (y - cy) as f32,
                false,
            ),
            Morceau::Eclat { x, y } => {
                banque.dessiner("grele_eclat", (x - cx) as f32, (y - cy) as f32, false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Colonne de 12 px de large, de y = 16 à y = 128 (7 tuiles), comme la colonne 9
    /// du niveau d'essai.
    fn zone() -> Aabb {
        Aabb::new(146.0, 16.0, 12.0, 112.0)
    }

    fn grelons(m: &[Morceau]) -> Vec<(i32, i32)> {
        m.iter()
            .filter_map(|m| match *m {
                Morceau::Grelon { x, y, .. } => Some((x, y)),
                _ => None,
            })
            .collect()
    }

    fn eclats(m: &[Morceau]) -> Vec<(i32, i32)> {
        m.iter()
            .filter_map(|m| match *m {
                Morceau::Eclat { x, y } => Some((x, y)),
                _ => None,
            })
            .collect()
    }

    /// Les grêlons restent DANS la colonne (bornés en largeur et en hauteur), à tout
    /// pas ; il y en a toujours (la grêle tombe en continu, deux files pleines).
    #[test]
    fn les_grelons_restent_dans_la_colonne_et_ne_s_arretent_jamais() {
        let z = zone();
        for pas in 0..600 {
            let m = disposer(z, pas, true, None);
            let g = grelons(&m);
            // 112 px de haut, un grêlon tous les 20 px, deux files : 10 à 12.
            assert!(
                (10..=12).contains(&g.len()),
                "pas {pas} : {} grêlons",
                g.len()
            );
            for (x, y) in g {
                assert!(
                    x >= z.x as i32 && x + TAILLE <= z.droite() as i32,
                    "x = {x}"
                );
                assert!(y >= z.y as i32 && y + TAILLE <= z.bas() as i32, "y = {y}");
            }
        }
    }

    /// D'un pas à l'autre, chaque grêlon descend de `VITESSE` px (on retrouve le même
    /// grêlon 3 px plus bas), sauf celui qui vient d'apparaître en haut.
    #[test]
    fn les_grelons_tombent_de_trois_pixels_par_pas() {
        let z = zone();
        for pas in 0..100 {
            let avant = grelons(&disposer(z, pas, true, None));
            let apres = grelons(&disposer(z, pas + 1, true, None));
            for &(x, y) in &apres {
                let vient_d_apparaitre = y < z.y as i32 + VITESSE;
                assert!(
                    vient_d_apparaitre || avant.contains(&(x, y - VITESSE)),
                    "pas {pas} : ({x}, {y}) sans prédécesseur"
                );
            }
        }
    }

    /// Les éclats n'apparaissent qu'au bas de la colonne, et seulement si un solide
    /// l'arrête ; au-dessus d'un vide, jamais d'éclat.
    #[test]
    fn les_eclats_sont_au_pied_de_la_colonne_sur_un_solide() {
        let z = zone();
        let mut vus = 0;
        for pas in 0..120 {
            for (_, y) in eclats(&disposer(z, pas, true, None)) {
                assert_eq!(y, z.bas() as i32 - 3);
                vus += 1;
            }
            assert!(eclats(&disposer(z, pas, false, None)).is_empty());
        }
        assert!(vus > 0, "des grêlons se brisent sur le toit");
    }

    /// Parapluie ouvert dans la colonne : aucun grêlon dans la toile ni dans le corps
    /// de Pépin ; ils se brisent SUR la toile (éclats juste au-dessus) ; au-dessus de
    /// la toile et sous ses pieds, la grêle continue.
    #[test]
    fn le_parapluie_ouvert_arrete_la_grele() {
        let z = zone();
        // Pépin plane au milieu de la colonne (boîte 10 x 14 en x = 148, y = 70).
        let p = parapluie(148.0, 70.0);
        let mut eclats_sur_la_toile = 0;
        let mut dessus = 0;
        let mut dessous = 0;
        for pas in 0..120 {
            let m = disposer(z, pas, true, Some(p));
            for (x, y) in grelons(&m) {
                let boite = Aabb::new(x as f32, y as f32, TAILLE as f32, TAILLE as f32);
                assert!(
                    !boite.chevauche(&p),
                    "pas {pas} : grêlon ({x}, {y}) sous la toile"
                );
                if y < p.y as i32 {
                    dessus += 1;
                } else {
                    dessous += 1;
                }
            }
            for (_, y) in eclats(&m) {
                if y == p.y as i32 - 3 {
                    eclats_sur_la_toile += 1;
                }
            }
        }
        assert!(eclats_sur_la_toile > 0, "la grêle rebondit sur la toile");
        assert!(dessus > 0 && dessous > 0);
    }

    /// Un parapluie hors de la colonne (à côté) ne change rien.
    #[test]
    fn un_parapluie_a_cote_ne_change_rien() {
        let z = zone();
        let loin = parapluie(200.0, 70.0);
        for pas in 0..60 {
            assert_eq!(
                disposer(z, pas, true, None),
                disposer(z, pas, true, Some(loin))
            );
        }
    }

    /// Même pas, même image (rendu reproductible) ; et le reflet tourne : les quatre
    /// images servent.
    #[test]
    fn le_rendu_est_reproductible_et_anime() {
        let z = zone();
        assert_eq!(disposer(z, 37, true, None), disposer(z, 37, true, None));
        let mut images = std::collections::BTreeSet::new();
        for pas in 0..16 {
            for m in disposer(z, pas, true, None) {
                if let Morceau::Grelon { image, .. } = m {
                    images.insert(image);
                }
            }
        }
        assert_eq!(images.len(), objets::GRELON.len());
    }
}
