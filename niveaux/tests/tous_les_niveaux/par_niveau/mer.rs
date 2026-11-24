//! Tests propres au niveau « mer » (autrefois le `mod tests` de
//! `core/src/levels/mer.rs` ; la grille et ses commentaires de conception
//! sont dans son fichier `.ron`).

use crate::commun::*;
use pepin_core::aabb::TAILLE_TUILE;
use pepin_core::entities::Entite;
use pepin_core::faisabilite::{analyser, Options};

/// Le niveau de ce module, lu dans son fichier `.ron`.
fn infos() -> &'static InfosNiveau {
    par_cle("mer")
}

/// Les zones de vent latéral du niveau, en tuiles : (colonne gauche, colonne
/// droite, ligne haute, ligne basse, sens), dans l'ordre du parseur.
fn zones_de_vent() -> Vec<(i32, i32, i32, i32, i8)> {
    let t = TAILLE_TUILE;
    let n = infos().charger().unwrap();
    n.vents
        .iter()
        .map(|v| {
            let z = v.zone;
            (
                (z.x / t) as i32,
                ((z.x + z.w) / t) as i32 - 1,
                (z.y / t) as i32,
                ((z.y + z.h) / t) as i32 - 1,
                v.sens,
            )
        })
        .collect()
}

/// Le moment fort : au-dessus du vide des colonnes 110 à 129 (vingt tuiles),
/// trois couches de vent superposées, sans trou entre elles : `>` en haut, `<`
/// au milieu, `>` en bas. En descendant, le vent change DEUX fois de sens.
#[test]
fn le_moment_fort_est_un_plane_de_vingt_tuiles_ou_le_vent_change_deux_fois() {
    let mut couches: Vec<_> = zones_de_vent()
        .into_iter()
        .filter(|&(g, d, _, _, _)| g >= 110 && d <= 129)
        .collect();
    couches.sort_by_key(|&(_, _, haut, _, _)| haut);
    assert_eq!(
        couches,
        vec![
            (110, 129, 2, 3, 1),
            (113, 129, 4, 6, -1),
            (116, 129, 7, 8, 1),
        ]
    );
    // Vingt tuiles de large pour la couche du haut, celle où commence le plané.
    assert_eq!(couches[0].1 - couches[0].0 + 1, 20);
    // Les couches se touchent (la ligne basse de l'une, la ligne haute de la
    // suivante) : le sens change d'un coup, sans accalmie entre deux.
    for paire in couches.windows(2) {
        assert_eq!(paire[0].3 + 1, paire[1].2, "{paire:?}");
        assert_eq!(paire[0].4, -paire[1].4, "{paire:?}");
    }
}

/// Sans le vent latéral, le niveau est bloqué dès le premier vide : c'est le vent
/// qui fait les longs planés (la nouveauté du niveau).
#[test]
fn sans_vent_la_mer_de_nuages_est_infranchissable() {
    let n = infos().charger().unwrap();
    let r = analyser(
        &n,
        n.depart,
        Options {
            vents: false,
            ..Options::default()
        },
    );
    assert!(!r.arrivee, "sans vent le niveau doit être bloqué");
    assert!(r.x_max < 27.0 * TAILLE_TUILE, "x max {}", r.x_max);
}

/// Les trois montgolfières (quai -> étage, quai -> tour, plané -> corniche)
/// montent toutes de 5 tuiles, arrêtées par le petit nuage au-dessus de leur
/// ballon : (x de la nacelle, altitude de départ, altitude en haut).
#[test]
fn trois_montgolfieres_de_cinq_tuiles() {
    let n = infos().charger().unwrap();
    let alt: Vec<(f32, f32, f32)> = n
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Montgolfiere(m) => Some((m.boite.x, m.altitudes().0, m.altitudes().1)),
            _ => None,
        })
        .collect();
    assert_eq!(
        alt,
        vec![
            (1008.0, 128.0, 48.0),
            (1664.0, 128.0, 48.0),
            (2272.0, 144.0, 64.0),
        ]
    );
}

/// Les deux marches de la tour ne sont pas des culs-de-sac : de chacune, on
/// finit le niveau (en remontant à la tour, ou en planant de là). Dans une
/// première version, les marches étaient 16 px plus bas (48 px sous la tour,
/// puis 32 px plus bas) : le joueur simulé de la mesure de difficulté, repoussé
/// par la couche du milieu, y restait jusqu'à la fin de l'essai (aucun chemin
/// connu depuis ces cases).
#[test]
#[ignore = "lot : recherche complète du robot de faisabilité"]
fn on_repart_de_chaque_marche() {
    let n = infos().charger().unwrap();
    let t = TAILLE_TUILE;
    for (colonne, dessus) in [(111.0, 6.0), (114.0, 8.0)] {
        let pose = (
            colonne * t + 3.0,
            dessus * t - pepin_core::physics::JOUEUR_H,
        );
        assert!(n.tuile_decor(colonne as i32, dessus as i32).est_solide());
        let r = analyser(&n, pose, Options::arrivee_seule());
        assert!(r.arrivee, "bloqué sur la marche de la colonne {colonne}");
    }
}

/// L'arrivée du moment fort (carte 5.18) : la rive commence à la colonne 131, une
/// tuile APRÈS la fin du vent (colonne 129) ; la colonne 130 est vide de haut en
/// bas. C'est ce qui place la mer « un peu au-dessus » des vents d'altitude dans
/// la mesure de difficulté (voir `ecarts_de_la_courbe`, côté jeu).
#[test]
fn la_rive_du_moment_fort_commence_une_tuile_apres_le_vent() {
    let n = infos().charger().unwrap();
    for ty in 0..n.hauteur() as i32 {
        assert!(
            !n.tuile_decor(130, ty).est_solide(),
            "colonne 130, ligne {ty}"
        );
    }
    assert!(n.tuile_decor(131, 9).est_solide());
    assert!(!n.tuile_decor(131, 8).est_solide());
}

/// Carte 6.4b : trois corbeaux et un pigeon (jusque-là, aucun ennemi), et aucun
/// dans le moment fort. La patrouille d'un corbeau va de sa case de départ moins
/// `PORTEE_CORBEAU` à sa case de départ plus `PORTEE_CORBEAU` (plus sa largeur) :
/// elle doit rester hors des colonnes 110 à 130 (le plané de vingt tuiles et la
/// colonne sans vent qui le termine). La demande admettait un corbeau au plus sur
/// ce plané ; il n'y en a aucun.
#[test]
fn trois_corbeaux_un_pigeon_et_aucun_dans_le_moment_fort() {
    use pepin_core::entities::ennemi::PORTEE_CORBEAU;
    use pepin_core::entities::TypeEnnemi;
    let n = infos().charger().unwrap();
    let (debut, fin) = (110.0 * TAILLE_TUILE, 131.0 * TAILLE_TUILE);
    let (mut corbeaux, mut pigeons) = (0, 0);
    for e in &n.entites {
        let Entite::Ennemi(en) = e else { continue };
        match en.type_ennemi {
            TypeEnnemi::Corbeau => {
                corbeaux += 1;
                let (x, _) = en.perchoir();
                let (gauche, droite) = (x - PORTEE_CORBEAU, x + PORTEE_CORBEAU + en.boite.w);
                assert!(
                    droite <= debut || gauche >= fin,
                    "corbeau parti de x = {x} : sa patrouille ({gauche} à {droite}) croise \
                     le moment fort"
                );
            }
            TypeEnnemi::Pigeon => pigeons += 1,
            autre => panic!("la mer n'a que des corbeaux et un pigeon : {autre:?}"),
        }
    }
    assert_eq!((corbeaux, pigeons), (3, 1));
}
