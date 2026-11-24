//! Habillage fixe du niveau : façades des boutiques derrière les auvents, panneaux,
//! objets de décor (oiseaux, manches à air), cadrans des aiguilles d'horloge, drapeau
//! d'arrivée. Rien ici ne bouge selon les règles du jeu : seulement selon `monde.pas`.

use pepin_core::aabb::{Aabb, TAILLE_TUILE};
use pepin_core::entities::auvent::HAUT_TOILE;
use pepin_core::entities::Entite;
use pepin_core::level::{GenreDecor, GenrePanneau, Niveau, Tuile};
use pepin_core::physics::{CourantAscendant, VentLateral};

use super::tuiles::boite_cachee;
use super::{entier, image};
use crate::monde::Monde;
use crate::sprites::{hauteur, largeur, objets, objets_decor, Banque};

/// Pixels entre le haut de la case d'un auvent et le sommet de son mât : le sprite de
/// l'auvent est posé 1 px sous le haut de sa case (`dessiner_entites`). La façade de
/// boutique part de la même hauteur (carte 5.12b) : le faîte de son toit est au sommet
/// du mât, jamais plus haut, 1 px sous le dessus des toits voisins.
pub(super) const SOMMET_DU_MAT: i32 = 1;

/// Façade de boutique derrière l'auvent de la case `(ax, ay)` (carte 5.12) : les cases
/// `(tx, ty, sprite)` à dessiner. Elle occupe la colonne à GAUCHE du mât : la vitrine
/// dans la rangée de l'auvent, le mur en dessous, jusqu'au premier solide ou au bas du
/// niveau. Si la case à gauche est déjà un solide (l'auvent est au bord d'un toit), le
/// toit est sa façade : rien à dessiner. Dessin seulement, aucune collision : la toile
/// ne flotte plus au-dessus du vide (on ne la confond pas avec une manche à air).
/// Fonction pure, testable.
fn facade_d_auvent(niveau: &Niveau, ax: i32, ay: i32) -> Vec<(i32, i32, &'static str)> {
    let x = ax - 1;
    if x < 0 {
        return Vec::new();
    }
    (ay..niveau.hauteur() as i32)
        .take_while(|&ty| niveau.tuile_decor(x, ty) == Tuile::Vide)
        .map(|ty| {
            let nom = if ty == ay {
                objets::FACADE[0]
            } else {
                objets::FACADE[1]
            };
            (x, ty, nom)
        })
        .collect()
}

/// Case `(tx, ty)` d'un auvent : sa zone de contact commence `HAUT_TOILE` px sous le
/// haut de la case.
fn case_auvent(boite: &Aabb) -> (i32, i32) {
    (
        (boite.x / TAILLE_TUILE) as i32,
        ((boite.y - HAUT_TOILE) / TAILLE_TUILE) as i32,
    )
}

/// Les façades de boutique, derrière les tuiles (voir `facade_d_auvent`).
pub(super) fn dessiner_facades(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    let t = TAILLE_TUILE as i32;
    for e in &monde.entites {
        let Entite::Auvent(a) = e else {
            continue;
        };
        let (ax, ay) = case_auvent(&a.boite);
        for (tx, ty, nom) in facade_d_auvent(&monde.niveau, ax, ay) {
            // La vitrine part du sommet du mât (elle remplit sa case jusqu'en bas) ;
            // le mur, en dessous, remplit ses cases entières.
            let retrait = if ty == ay { SOMMET_DU_MAT } else { 0 };
            banque.dessiner(
                nom,
                (tx * t - cx) as f32,
                (ty * t + retrait - cy) as f32,
                false,
            );
        }
    }
}

pub(super) fn dessiner_panneaux(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    for p in &monde.niveau.panneaux {
        let nom = match p.genre {
            GenrePanneau::Deplacer => "panneau_deplacer",
            GenrePanneau::Sauter => "panneau_sauter",
            GenrePanneau::Planer => "panneau_planer",
            GenrePanneau::Champignon => "panneau_champignon",
            GenrePanneau::Vent => "panneau_vent",
            GenrePanneau::Barque => "panneau_barque",
        };
        banque.dessiner(nom, p.x.round() - cx as f32, p.y.round() - cy as f32, false);
    }
}

/// Durée (pas) d'un cycle de l'oiseau posé, et durée de la tête tournée en fin de cycle.
const CYCLE_OISEAU: u32 = 300;
const TETE_TOURNEE: u32 = 40;

/// Image de l'oiseau posé n° `rang` au pas `pas` : il regarde devant, puis tourne la tête
/// 2/3 s toutes les 5 s. Chaque oiseau a son décalage (97 pas de plus que le précédent) :
/// ils ne tournent jamais la tête tous ensemble. Fonction pure, testable.
fn image_oiseau(pas: u32, rang: usize) -> &'static str {
    let phase = (pas + 97 * rang as u32) % CYCLE_OISEAU;
    objets_decor::OISEAU[usize::from(phase >= CYCLE_OISEAU - TETE_TOURNEE)]
}

/// La manche à air de la case `(x, y)` (coin haut-gauche, px) montre où va l'air :
/// - à l'entrée d'une zone de vent latéral (à une tuile au plus de son bord, à une
///   hauteur où il souffle ; carte 5.15), elle s'étire dans le SENS DU VENT ;
/// - sinon, elle s'étire vers le courant ascendant le plus proche (l'air y est
///   aspiré) : vers la droite si son centre est à droite de la manche (ou s'il n'y a
///   aucun courant).
fn manche_vers_la_droite(
    x: f32,
    y: f32,
    courants: &[CourantAscendant],
    vents: &[VentLateral],
) -> bool {
    let entree_de_vent = vents.iter().find(|v| {
        let z = v.zone;
        let ecart = (z.x - (x + TAILLE_TUILE)).max(x - (z.x + z.w));
        (0.0..=TAILLE_TUILE).contains(&ecart) && z.y <= y && y < z.y + z.h
    });
    if let Some(v) = entree_de_vent {
        return v.sens > 0;
    }
    let centre_manche = x + TAILLE_TUILE / 2.0;
    courants
        .iter()
        .map(|c| c.zone.x + c.zone.w / 2.0 - centre_manche)
        .min_by(|a, b| a.abs().total_cmp(&b.abs()))
        .is_none_or(|ecart| ecart >= 0.0)
}

/// Coin haut-gauche (au pixel) d'un sprite de décor de `largeur` x `hauteur` posé sur sa
/// case `(x, y)` : centré dans la largeur de la case, le bas sur le bas de la case (donc
/// sur la tuile solide du dessous).
fn coin_decor(x: f32, y: f32, largeur: usize, hauteur: usize) -> (i32, i32) {
    let t = TAILLE_TUILE as i32;
    (
        x.round() as i32 + (t - largeur as i32) / 2,
        y.round() as i32 + t - hauteur as i32,
    )
}

/// Objets de décor : derrière les panneaux, les objets, les ennemis et Pépin (il passe
/// devant une cheminée), sans aucune collision.
pub(super) fn dessiner_decors(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    let mut oiseaux = 0;
    for d in &monde.niveau.decors {
        let (nom, miroir) = match d.genre {
            GenreDecor::Cheminee => ("cheminee", false),
            GenreDecor::Antenne => ("antenne", false),
            GenreDecor::Paratonnerre => ("paratonnerre", false),
            GenreDecor::MancheAAir => (
                "manche_a_air",
                !manche_vers_la_droite(d.x, d.y, &monde.niveau.courants, &monde.niveau.vents),
            ),
            GenreDecor::Oiseau => {
                oiseaux += 1;
                (image_oiseau(monde.pas, oiseaux - 1), false)
            }
            // La rue du Monde 2 (carte 9.6b).
            GenreDecor::BoucheEgout => ("bouche_egout", false),
            GenreDecor::Borne => ("borne", false),
            GenreDecor::Poubelle => ("poubelle", false),
            // Carte 9.20h : la descente de gouttière est dessinée avec sa maison
            // (`decor_rue::dessiner_gouttieres`, avant les tuiles).
            GenreDecor::Gouttiere => continue,
        };
        let Some(dessin) = objets_decor::sprites()
            .iter()
            .find(|(n, _)| *n == nom)
            .map(|&(_, dessin)| dessin)
        else {
            continue;
        };
        let (x, y) = coin_decor(d.x, d.y, largeur(dessin), hauteur(dessin));
        banque.dessiner(nom, (x - cx) as f32, (y - cy) as f32, miroir);
    }
}

/// Écart (px) entre deux maillons de la tige d'une aiguille d'horloge.
const PAS_MAILLON: f32 = 6.0;

/// Coins haut-gauche (entiers) des maillons 4x4 de la tige d'une aiguille, du
/// pivot `pivot` jusqu'au centre `bout` de sa plaque : un tous les `PAS_MAILLON`
/// px, le premier à 6 px du pivot (le moyeu du cadran est au centre), le dernier
/// au moins 4 px avant le bout (caché sous la plaque sinon). La tige tourne avec
/// la plaque comme l'aiguille d'une montre : seules les POSITIONS changent, aucun
/// sprite n'est tourné. Fonction pure.
fn maillons(pivot: (f32, f32), bout: (f32, f32)) -> Vec<(i32, i32)> {
    let (dx, dy) = (bout.0 - pivot.0, bout.1 - pivot.1);
    let longueur = (dx * dx + dy * dy).sqrt();
    if longueur < 1.0 {
        return Vec::new();
    }
    let n = ((longueur - 4.0) / PAS_MAILLON).floor().max(0.0) as usize;
    (1..=n)
        .map(|k| {
            let t = k as f32 * PAS_MAILLON / longueur;
            let (x, y) = (pivot.0 + dx * t, pivot.1 + dy * t);
            ((x - 2.0).round() as i32, (y - 2.0).round() as i32)
        })
        .collect()
}

/// Les cadrans des aiguilles d'horloge (32x32, centrés sur leur pivot) et la tige
/// de maillons qui va du moyeu au milieu de la plaque. Dessinés AVANT les entités :
/// la plaque, Pépin, les gouttes passent devant.
pub(super) fn dessiner_cadrans(monde: &Monde, banque: &Banque, alpha: f32, cx: i32, cy: i32) {
    for (i, entite) in monde.entites.iter().enumerate() {
        let Entite::Mobile(m) = entite else {
            continue;
        };
        let Some(cercle) = m.cercle() else {
            continue;
        };
        let (px, py) = (cercle.pivot.0.round() as i32, cercle.pivot.1.round() as i32);
        banque.dessiner(
            "cadran",
            (px - 16 - cx) as f32,
            (py - 16 - cy) as f32,
            false,
        );
        // Bout de la tige : le milieu de la plaque, à sa position DESSINÉE (arrondie).
        let (x, y) = entier(&monde.boite_interpolee(i, alpha));
        let bout = (x as f32 + m.boite.w / 2.0, y as f32 + m.boite.h / 2.0);
        for (mx, my) in maillons(cercle.pivot, bout) {
            banque.dessiner("maillon", (mx - cx) as f32, (my - cy) as f32, false);
        }
    }
}

pub(super) fn dessiner_arrivee(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    // La sortie secrète (carte 8.5) : le même drapeau, caché tant que sa salle l'est.
    let secrete = monde
        .niveau
        .sortie_secrete
        .filter(|a| !boite_cachee(monde, a));
    for a in monde.niveau.arrivee.iter().chain(secrete.iter()) {
        dessiner_drapeau(a, monde, banque, cx, cy);
    }
}

/// Le drapeau de la boîte d'arrivée `a`. (L'arène du Grand Nuage n'a pas de drapeau : le
/// vaincre finit le niveau.)
fn dessiner_drapeau(a: &Aabb, monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    let x = (a.x + a.w / 2.0 - 8.0).round() as i32 - cx;
    let y = a.bas().round() as i32 - 32 - cy;
    banque.dessiner(
        image(&objets::DRAPEAU, monde.pas, 12),
        x as f32,
        y as f32,
        false,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Aiguilles d'horloge (carte 5.10) ---

    /// La tige d'une aiguille de 48 px de rayon : 7 maillons tous les 6 px, du
    /// moyeu jusque sous la plaque, alignés sur le rayon, à midi comme à 3 h.
    #[test]
    fn les_maillons_vont_du_pivot_a_la_plaque() {
        let midi = maillons((120.0, 88.0), (120.0, 40.0));
        assert_eq!(midi.len(), 7);
        assert_eq!(midi[0], (118, 80));
        assert_eq!(midi[6], (118, 44));
        assert!(midi
            .windows(2)
            .all(|w| w[0].0 == w[1].0 && w[0].1 - w[1].1 == 6));
        let trois_heures = maillons((120.0, 88.0), (168.0, 88.0));
        let attendu: Vec<(i32, i32)> = (1..=7).map(|k| (118 + 6 * k, 86)).collect();
        assert_eq!(trois_heures, attendu);
        // En diagonale : chaque maillon reste à moins d'un pixel du rayon.
        let diag = maillons((0.0, 0.0), (30.0, 40.0));
        assert_eq!(diag.len(), 7);
        for (x, y) in diag {
            let (cx, cy) = (x as f32 + 2.0, y as f32 + 2.0);
            let ecart = (cx * 40.0 - cy * 30.0).abs() / 50.0;
            assert!(ecart <= 0.75, "({x}, {y}) : {ecart}");
        }
        assert!(maillons((5.0, 5.0), (5.0, 5.0)).is_empty());
    }

    // --- Objets de décor (carte 4.2) ---

    /// Chaque oiseau tourne la tête 40 pas sur 300 (2/3 s toutes les 5 s), jamais tous
    /// ensemble : deux oiseaux voisins ne changent pas d'image au même pas.
    #[test]
    fn l_oiseau_tourne_la_tete_de_temps_en_temps() {
        for rang in 0..6 {
            let tete: Vec<bool> = (0..CYCLE_OISEAU)
                .map(|pas| image_oiseau(pas, rang) == "oiseau1")
                .collect();
            assert_eq!(tete.iter().filter(|&&t| t).count(), TETE_TOURNEE as usize);
            // Une seule série de tête tournée par cycle (pas de clignotement).
            let changements = (0..CYCLE_OISEAU as usize)
                .filter(|&i| tete[i] != tete[(i + 1) % CYCLE_OISEAU as usize])
                .count();
            assert_eq!(changements, 2, "oiseau {rang}");
        }
        for pas in 0..3 * CYCLE_OISEAU {
            let ensemble = (0..6).all(|rang| image_oiseau(pas, rang) == "oiseau1");
            assert!(!ensemble, "pas {pas} : tous les oiseaux tournent la tête");
        }
    }

    /// Au niveau 4, chaque manche à air est à gauche de son courant : elle s'étire vers la
    /// droite. Retournée si le courant le plus proche est à gauche.
    #[test]
    fn la_manche_a_air_s_etire_vers_son_courant() {
        let niv = niveaux::par_cle("vents").charger().unwrap();
        let manches: Vec<f32> = niv
            .decors
            .iter()
            .filter(|d| d.genre == GenreDecor::MancheAAir)
            .map(|d| d.x)
            .collect();
        assert_eq!(manches.len(), 4);
        assert!(manches
            .iter()
            .all(|&x| manche_vers_la_droite(x, 0.0, &niv.courants, &niv.vents)));
        let a_gauche = CourantAscendant::new(Aabb::new(0.0, 0.0, 48.0, 64.0));
        let a_droite = CourantAscendant::new(Aabb::new(300.0, 0.0, 48.0, 64.0));
        assert!(!manche_vers_la_droite(80.0, 0.0, &[a_gauche], &[]));
        assert!(manche_vers_la_droite(80.0, 0.0, &[a_droite], &[]));
        // Le plus proche l'emporte.
        assert!(!manche_vers_la_droite(
            80.0,
            0.0,
            &[a_droite, a_gauche],
            &[]
        ));
        assert!(manche_vers_la_droite(
            250.0,
            0.0,
            &[a_gauche, a_droite],
            &[]
        ));
        assert!(
            manche_vers_la_droite(80.0, 0.0, &[], &[]),
            "sans courant : vers la droite"
        );
    }

    /// Carte 5.15 : à l'entrée d'une zone de vent latéral, la manche à air
    /// s'étire dans le sens du vent (et non plus vers le courant le plus proche).
    /// Dans la mer de nuages, une manche par zone : toutes vers la droite, sauf
    /// celle de la couche du milieu du moment fort (`<`).
    #[test]
    fn la_manche_a_air_montre_le_sens_du_vent_lateral() {
        let niv = niveaux::par_cle("mer").charger().unwrap();
        let t = TAILLE_TUILE;
        let manches: Vec<((i32, i32), bool)> = niv
            .decors
            .iter()
            .filter(|d| d.genre == GenreDecor::MancheAAir)
            .map(|d| {
                (
                    ((d.x / t) as i32, (d.y / t) as i32),
                    manche_vers_la_droite(d.x, d.y, &niv.courants, &niv.vents),
                )
            })
            .collect();
        assert_eq!(
            manches,
            vec![
                ((108, 3), true),
                ((149, 4), true),
                ((111, 5), false),
                ((12, 7), true),
                ((114, 7), true),
            ]
        );
        // Le vent l'emporte sur le courant : même un courant à gauche ne la
        // retourne pas à l'entrée d'un vent vers la droite (et inversement).
        let courant_a_gauche = CourantAscendant::new(Aabb::new(0.0, 0.0, 48.0, 64.0));
        let vers_la_droite = VentLateral::new(Aabb::new(96.0, 0.0, 64.0, 64.0), 1);
        let vers_la_gauche = VentLateral::new(Aabb::new(96.0, 0.0, 64.0, 64.0), -1);
        assert!(manche_vers_la_droite(
            80.0,
            16.0,
            &[courant_a_gauche],
            &[vers_la_droite]
        ));
        assert!(!manche_vers_la_droite(80.0, 16.0, &[], &[vers_la_gauche]));
        // Une tuile vide entre la manche et le vent : c'est encore son entrée ; deux
        // tuiles (trop loin), ou trop bas : le vent ne compte pas.
        assert!(manche_vers_la_droite(
            64.0,
            16.0,
            &[courant_a_gauche],
            &[vers_la_droite]
        ));
        assert!(!manche_vers_la_droite(
            48.0,
            16.0,
            &[courant_a_gauche],
            &[vers_la_droite]
        ));
        assert!(!manche_vers_la_droite(
            80.0,
            64.0,
            &[courant_a_gauche],
            &[vers_la_droite]
        ));
    }

    /// Un sprite de décor est centré dans sa case et posé sur la tuile du dessous ; chaque
    /// genre de décor a son dessin.
    #[test]
    fn le_decor_est_pose_sur_sa_case() {
        assert_eq!(coin_decor(48.0, 112.0, 12, 20), (50, 108));
        assert_eq!(coin_decor(48.0, 112.0, 16, 22), (48, 106));
        assert_eq!(coin_decor(48.0, 112.0, 7, 24), (52, 104));
        let noms = [
            "cheminee",
            "antenne",
            "paratonnerre",
            "manche_a_air",
            "bouche_egout",
            "borne",
            "poubelle",
        ];
        for nom in noms.iter().chain(objets_decor::OISEAU.iter()) {
            assert!(
                objets_decor::sprites().iter().any(|(n, _)| n == nom),
                "{nom} : pas de dessin"
            );
        }
        // L'oiseau a un jeu d'images ; la gouttière (carte 9.20h) est dessinée avec
        // sa maison par `decor_rue`, sans sprite.
        assert_eq!(GenreDecor::TOUS.len(), noms.len() + 2);
    }

    // --- Façades de boutique derrière les auvents (carte 5.12) ---

    /// Chaque auvent du marché couvert et du niveau d'essai est accroché à une
    /// boutique : au bord d'un toit (la case à gauche est solide), ou devant une
    /// façade dessinée dans la case à gauche (la vitrine dans sa rangée, le mur
    /// dessous jusqu'au premier solide ou au bas du niveau). Aucune façade ne se
    /// dessine sur un solide.
    #[test]
    fn chaque_auvent_est_accroche_a_une_boutique() {
        let mut dessinees = 0;
        for infos in [
            *niveaux::par_cle("marche"),
            *niveaux::par_cle("essai_auvent"),
        ] {
            let niv = infos.charger().unwrap();
            for e in &niv.entites {
                let Entite::Auvent(a) = e else {
                    continue;
                };
                let (ax, ay) = case_auvent(&a.boite);
                let f = facade_d_auvent(&niv, ax, ay);
                if niv.tuile_decor(ax - 1, ay) != Tuile::Vide {
                    assert!(
                        f.is_empty(),
                        "{} : auvent {ax} au bord d'un toit",
                        infos.nom
                    );
                    continue;
                }
                dessinees += 1;
                assert_eq!(
                    f[0],
                    (ax - 1, ay, "facade_vitrine"),
                    "{} : auvent {ax}",
                    infos.nom
                );
                for (i, &(tx, ty, nom)) in f.iter().enumerate() {
                    assert_eq!((tx, ty), (ax - 1, ay + i as i32));
                    assert_eq!(nom, objets::FACADE[usize::from(i > 0)]);
                    assert_eq!(niv.tuile_decor(tx, ty), Tuile::Vide);
                }
                // Le mur descend jusqu'au bas du niveau ou jusqu'à un solide.
                let bas = ay + f.len() as i32;
                assert!(
                    bas == niv.hauteur() as i32 || niv.tuile_decor(ax - 1, bas) != Tuile::Vide,
                    "{} : façade de l'auvent {ax} coupée en l'air",
                    infos.nom
                );
            }
        }
        // Marché : 7 façades dessinées (auvents 44, 89, 93, 97, 101, 105 et 139 ; 20,
        // 40, 85 et 135 sont au bord d'un toit) ; essai : 5 (le premier est au bord).
        assert_eq!(dessinees, 7 + 5);
    }

    /// La façade se dessine dans la colonne à gauche du mât, la vitrine à partir du
    /// sommet du mât (`SOMMET_DU_MAT` sous le haut de sa case) jusqu'au bas de sa case,
    /// et ne garde de la palette que des couleurs sans rôle dans le jeu : ni or
    /// (gouttes, éclairs), ni cyan (flaques, pluie), ni corail (parapluie, toile de
    /// l'auvent).
    #[test]
    fn la_facade_de_boutique_a_sa_taille_et_ses_couleurs() {
        let (l, h, vitrine) = dessin_facade("facade_vitrine");
        assert_eq!((l, h as i32), (16, 16 - SOMMET_DU_MAT));
        let (l, h, mur) = dessin_facade("facade_mur");
        assert_eq!((l, h), (16, 16));
        for d in [vitrine, mur] {
            for ligne in d {
                assert!(!ligne.contains(['B', 'C', 'D', 'E', '8', '9']), "{ligne}");
                // Transparent seulement à GAUCHE (au-dessus de la pente du toit) :
                // contre le mât, chaque rangée finit par le contour sombre, et son
                // premier pixel opaque est sombre lui aussi.
                let opaques = ligne.trim_start_matches('.');
                assert!(!opaques.contains('.'), "{ligne}");
                assert!(
                    opaques.starts_with('0') && opaques.ends_with('0'),
                    "{ligne}"
                );
            }
        }
        // Le mur remplit toute sa case (il descend jusqu'au premier solide).
        assert!(mur.iter().all(|l| !l.contains('.')));
        // La vitrine est allumée (orange), le mur n'a que des fenêtres éteintes.
        assert!(vitrine.iter().any(|l| l.contains('A')));
        assert!(mur.iter().all(|l| !l.contains('A')));
    }

    /// Carte 5.12b : la façade n'offre aucun « dessus » qui ressemble à un sol.
    /// Ce qui porte Pépin a une arête CLAIRE et PLATE (le haut des tuiles de toit et
    /// de brique) ; la façade, elle, monte en pente jusqu'au mât (toit en appentis :
    /// avec la toile, un seul toit dont le haut est l'auvent) et n'a que les
    /// teintes sombres du fond, plus l'orange de sa vitrine.
    #[test]
    fn la_facade_n_a_pas_de_dessus_ou_se_poser() {
        let (toit_haut, brique_haut) = (
            crate::sprites::dessin("toit_haut"),
            crate::sprites::dessin("brique_haut"),
        );
        let (_, _, vitrine) = dessin_facade("facade_vitrine");
        let (_, _, mur) = dessin_facade("facade_mur");

        // 1. Couleurs. L'arête d'un dessus de tuile (rangées 1 et 2) est un ton
        //    clair, de l'ardoise claire (3) à la brume (5) ; les thèmes ne font que
        //    l'éclaircir (jusqu'au blanc cassé, 7). La façade n'en a aucun : ombre,
        //    nuit, ardoise (0, 1, 2) et l'orange de la vitrine (A), rien d'autre.
        for tuile in [toit_haut, brique_haut] {
            for ligne in &tuile[1..=2] {
                assert!(ligne.chars().all(|c| ('3'..='7').contains(&c)), "{ligne}");
            }
        }
        for d in [vitrine, mur] {
            for ligne in d {
                assert!(
                    ligne.chars().all(|c| ".012A".contains(c)),
                    "ton clair sur la façade : {ligne}"
                );
            }
        }
        // Sa rangée supérieure : le seul contour sombre du faîte, contre le mât.
        assert_eq!(vitrine[0].trim_start_matches('.'), "00");

        // 2. Silhouette. Le haut de chaque colonne (premier pixel opaque) monte de
        //    la gauche vers le mât, d'un pixel toutes les deux colonnes au plus :
        //    aucun palier où un Pépin de 10 px croirait tenir debout.
        let haut = |x: usize| -> usize {
            vitrine
                .iter()
                .position(|l| l.as_bytes()[x] != b'.')
                .unwrap()
        };
        let hauts: Vec<usize> = (0..16).map(haut).collect();
        assert_eq!(hauts[15], 0, "le faîte est contre le mât : {hauts:?}");
        for x in 0..15 {
            assert!(hauts[x] >= hauts[x + 1], "le toit redescend : {hauts:?}");
            assert!(
                hauts[x] <= hauts[x + 1] + 1,
                "le toit fait une marche : {hauts:?}"
            );
        }
        let palier_max = hauts
            .chunk_by(|a, b| a == b)
            .map(<[usize]>::len)
            .max()
            .unwrap();
        assert!(palier_max <= 2, "palier de {palier_max} px : {hauts:?}");
        // Le bord du toit est du contour sombre, jamais une arête claire.
        for (x, &y) in hauts.iter().enumerate() {
            assert_eq!(vitrine[y].as_bytes()[x], b'0', "colonne {x}");
        }

        // 3. Hauteur. La façade et l'auvent sont posés à partir du même
        //    `SOMMET_DU_MAT` (1 px sous le haut de leur case, donc sous le dessus des
        //    toits voisins) : la rangée 0 de la vitrine (le faîte) est à la hauteur
        //    de la rangée 0 de l'auvent (le haut du mât), jamais plus haut.
        let (_, _, auvent) = dessin_facade(objets::AUVENT[0]);
        assert!(auvent[0].starts_with("000"), "{}", auvent[0]);
    }

    /// Dessin d'un sprite par son nom, avec sa largeur et sa hauteur.
    fn dessin_facade(nom: &str) -> (usize, usize, crate::sprites::Image) {
        let d = crate::sprites::tous()
            .into_iter()
            .find(|(n, _)| *n == nom)
            .unwrap()
            .1;
        (largeur(d), hauteur(d), d)
    }
}
