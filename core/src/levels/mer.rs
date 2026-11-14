//! Niveau 8 : « La mer de nuages ».
//!
//! « Laisse le vent choisir, puis reprends la main. » Après les vents d'altitude,
//! Pépin passe au-dessus d'une mer de nuages, sous un ciel corail. Nouveauté du
//! niveau : le VENT LATÉRAL (`<` et `>`, carte 5.3), qui pousse Pépin à 60 px/s,
//! deux fois plus fort parapluie ouvert ; avec lui, les deux mécaniques que la
//! demande d'AGA lui associe : les montgolfières (`b`, carte 5.7) pour reprendre
//! de la hauteur, et les rideaux de grêle (`g`, carte 5.4) à traverser parapluie
//! ouvert. Aucun ennemi.
//!
//! Chaque zone de vent a sa manche à air (`Q`) à son entrée (le côté par où Pépin
//! y arrive), tournée dans le sens du vent (`scene::manche_vers_la_droite`).
//!
//! Le parcours, de gauche à droite :
//! 1. le départ : un vide de 13 tuiles, trop large pour un plané sans aide ; le
//!    vent `>` au-dessus le fait traverser (premier plané porté) ;
//! 2. un premier rideau de grêle au-dessus d'un vide de 6 tuiles ;
//! 3. une montgolfière monte au premier étage (trop haut pour un saut) ;
//! 4. un long plané sous deux rideaux de grêle, une goutte entre les deux ;
//! 5. une seconde montgolfière monte au sommet d'une tour de nuages ;
//! 6. MOMENT FORT : un plané de vingt tuiles (colonnes 110 à 129) au-dessus du
//!    vide, à travers trois couches de vent superposées : `>` en haut, `<` au
//!    milieu, `>` en bas. En descendant, le vent change deux fois de sens : il
//!    porte, puis repousse (garder la flèche vers l'avant, ou fermer le
//!    parapluie pour traverser vite la couche contraire), puis porte de nouveau.
//!    Chaque couche a sa manche à air sur une marche de la tour. Le vent s'arrête
//!    une tuile avant la rive (colonne 130, vide et sans vent : carte 5.18) : il
//!    faut finir le plané par soi-même, ce qui fait « remonter un peu » la
//!    difficulté au-dessus de celle des vents d'altitude (niveau 7) ;
//! 7. une troisième montgolfière, puis un dernier plané porté sous la grêle
//!    jusqu'au drapeau.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 8,
    nom: "La mer de nuages",
    theme: Theme::Mer,
    genre: GenreNiveau::Normal,
    intro: "Laisse le vent choisir, puis reprends la main.",
    mecanique: "Vent latéral : longs planés portés, une couche contraire à traverser ; \
                montgolfières, rideaux de grêle.",
    duree_cible_s: 50,
    ascii: ASCII,
    aiguilles: &[],
};

/// Dessin du niveau (176 x 11 tuiles, un écran de haut). Légende : voir `crate::level`.
/// - Départ : colonnes 0 à 13 (dessus à y = 128), manche à air (12, 7) ; vent `>`
///   des colonnes 14 à 26, lignes 4 à 7, au-dessus du vide ; goutte au sommet du
///   saut (17, 3), juste au-dessus du vent.
/// - Lanterne (29, 7) ; grêle colonnes 48-49 au-dessus du vide (45 à 50).
/// - Quai 51-62, montgolfière (63, 8) arrêtée par le nuage de la ligne 0 (elle
///   monte de 5 tuiles) ; étage 65-77 (dessus à y = 64, 64 px au-dessus du quai :
///   hors de portée d'un saut), lanterne (66, 3).
/// - Vide 78-90 : grêle colonnes 81-82 et 86-87 depuis la ligne 0, goutte (84, 2)
///   entre les deux rideaux ; quai 91-103.
/// - Montgolfière (104, 8) jusqu'au sommet de la tour 106-109 (dessus à y = 64),
///   lanterne (106, 3) ; deux marches de 3 tuiles, 110-112 (dessus à y = 96) et
///   113-115 (dessus à y = 128), 32 px l'une sous l'autre : une manche à air sur
///   chaque palier, à une tuile de l'entrée de sa couche de vent, (108, 3),
///   (111, 5) et (114, 7). Les marches rattrapent qui se laisse repousser par la
///   couche du milieu, et l'on remonte de l'une à l'autre jusqu'à la tour : aucun
///   cul-de-sac (test `on_repart_de_chaque_marche`).
/// - Moment fort : `>` colonnes 110-129 lignes 2-3, `<` 113-129 lignes 4-6, `>`
///   116-129 lignes 7-8 ; goutte (112, 1) au sommet du saut depuis la tour ;
///   colonne 130 vide et sans vent (carte 5.18) ; arrivée du plané : colonnes
///   131-141 (dessus à y = 144), lanterne (132, 8).
/// - Montgolfière (142, 9) jusqu'à la corniche 144-150 (dessus à y = 80), manche à
///   air (149, 4) ; vent `>` 151-166 lignes 3-5, grêle colonnes 158-159 ; drapeau
///   (172, 7).
const ASCII: &str = r"
..............................................................NNN................gg...gg...............NNN......................................................................
................................................gg..............................................................o............................NNN..............gg................
....................................................................................o.........................>>>>>>>>>>>>>>>>>>>>..............................................
.................o................................................L.......................................L.Q.>>>>>>>>>>>>>>>>>>>>.....................>>>>>>>>>>>>>>>>.........
..............>>>>>>>>>>>>>......................................NNNNNNNNNNNNN............................NNNN...<<<<<<<<<<<<<<<<<...................Q.>>>>>>>>>>>>>>>>.........
..............>>>>>>>>>>>>>......................................NNNNNNNNNNNNN............................NNNN.Q.<<<<<<<<<<<<<<<<<..............NNNNNNN>>>>>>>>>>>>>>>>.........
..............>>>>>>>>>>>>>...............................................................................NNNNNNN<<<<<<<<<<<<<<<<<..............NNNNNNN.........................
..@.........Q.>>>>>>>>>>>>>..L............................................................................NNNNNNN.Q.>>>>>>>>>>>>>>..........................................X...
NNNNNNNNNNNNNN.............NNNNNNNNNNNNNNNNNN......NNNNNNNNNNNNb...........................NNNNNNNNNNNNNb.NNNNNNNNNN>>>>>>>>>>>>>>..L..................................NNNNNNNNN
NNNNNNNNNNNNNN.............NNNNNNNNNNNNNNNNNN......NNNNNNNNNNNN............................NNNNNNNNNNNNN..NNNNNNNNNN...............NNNNNNNNNNNb........................NNNNNNNNN
NNNNNNNNNNNNNN.............NNNNNNNNNNNNNNNNNN......NNNNNNNNNNNN............................NNNNNNNNNNNNN..NNNNNNNNNN...............NNNNNNNNNNN.........................NNNNNNNNN
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::TAILLE_TUILE;
    use crate::entities::Entite;
    use crate::faisabilite::{analyser, Options};

    /// Les zones de vent latéral du niveau, en tuiles : (colonne gauche, colonne
    /// droite, ligne haute, ligne basse, sens), dans l'ordre du parseur.
    fn zones_de_vent() -> Vec<(i32, i32, i32, i32, i8)> {
        let t = TAILLE_TUILE;
        let n = INFOS.charger().unwrap();
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
        let n = INFOS.charger().unwrap();
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
        let n = INFOS.charger().unwrap();
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
    fn on_repart_de_chaque_marche() {
        let n = INFOS.charger().unwrap();
        let t = TAILLE_TUILE;
        for (colonne, dessus) in [(111.0, 6.0), (114.0, 8.0)] {
            let pose = (colonne * t + 3.0, dessus * t - crate::physics::JOUEUR_H);
            assert!(n.tuile_decor(colonne as i32, dessus as i32).est_solide());
            let r = analyser(&n, pose, Options::default());
            assert!(r.arrivee, "bloqué sur la marche de la colonne {colonne}");
        }
    }

    /// L'arrivée du moment fort (carte 5.18) : la rive commence à la colonne 131, une
    /// tuile APRÈS la fin du vent (colonne 129) ; la colonne 130 est vide de haut en
    /// bas. C'est ce qui place la mer « un peu au-dessus » des vents d'altitude dans
    /// la mesure de difficulté (voir `ecarts_de_la_courbe`, côté jeu).
    #[test]
    fn la_rive_du_moment_fort_commence_une_tuile_apres_le_vent() {
        let n = INFOS.charger().unwrap();
        for ty in 0..n.hauteur() as i32 {
            assert!(!n.tuile_decor(130, ty).est_solide(), "colonne 130, ligne {ty}");
        }
        assert!(n.tuile_decor(131, 9).est_solide());
        assert!(!n.tuile_decor(131, 8).est_solide());
    }

    /// Aucun ennemi : le vent, la grêle et le vide suffisent.
    #[test]
    fn aucun_ennemi() {
        let n = INFOS.charger().unwrap();
        assert!(!n.entites.iter().any(|e| matches!(e, Entite::Ennemi(_))));
    }
}
