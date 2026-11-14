//! Niveau 11 (épilogue) : « La descente ».
//!
//! « Maintenant, on rentre. » Le dernier niveau, joué après le neuvième (ou après
//! le bonus) ; l'écran de fin le suit. C'est le niveau VERTICAL À L'ENVERS : le
//! clocher se gravissait, celui-ci se descend. Départ tout en haut, au soleil, sur
//! un petit nuage ; arrivée tout en bas, la nuit, sur le toit du premier niveau
//! (mêmes tuiles, une cheminée, une antenne et un bloc de briques avant le
//! drapeau). 32 tuiles de large, 45 de haut (quatre écrans) : la caméra suit
//! Pépin en hauteur (carte 5.2).
//!
//! Nouveauté du niveau : la descente elle-même, sous un ciel qui refait tout le
//! voyage en sens inverse (thème `Descente` : des bandes de couleur selon la
//! hauteur, de l'or du soleil à la nuit des toits, voir `decor::VOYAGE`). Rien de
//! nouveau à apprendre : on tombe ou on plane, entre deux pigeons (`p`) puis deux
//! colonnes de grêle (`g`). Le plus calme des onze niveaux : aucune chute ne tue
//! (chaque trou mène à l'étage du dessous), aucune pointe, aucun éclair.
//!
//! La forme : trois couches de nuages, neuf rangées l'une sous l'autre, chacune
//! percée d'un trou de six tuiles, une fois à droite (colonnes 24 à 29), une fois à
//! gauche (2 à 7) ; on court jusqu'au trou, on s'y laisse tomber (ou on y plane),
//! on se pose sur la couche du dessous et on repart dans l'autre sens. Une colonne
//! de grêle tombe du dessous d'une couche, par le bord LOINTAIN du trou suivant
//! (colonnes 2 et 29), jusqu'au coin de la couche (ou du toit) d'en dessous : on
//! descend juste à côté d'elle. Celui qui se laisse tomber du bord ne la touche
//! pas (cinq tuiles plus loin) ; celui qui saute loin dans le trou, parapluie
//! fermé, si.
//!
//! Le parcours, de haut en bas :
//! 1. le départ au soleil ; on tombe sur la première couche (une goutte en l'air,
//!    à attraper d'un saut ou en planant) ;
//! 2. la première couche, vers la droite, sous deux pigeons perchés, tournés vers
//!    Pépin qui arrive : on passe dessous sans s'arrêter, ils plongent derrière
//!    lui ;
//! 3. la deuxième couche, vers la gauche, jusqu'au trou que longe la première
//!    colonne de grêle ; la 2e goutte flotte au-dessus du trou, près d'elle ;
//! 4. la troisième couche, vers la droite : même chose, la 3e goutte près de la
//!    seconde colonne de grêle ;
//! 5. le toit du premier niveau, vers la gauche, jusqu'au drapeau.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 11,
    nom: "La descente",
    theme: Theme::Descente,
    genre: GenreNiveau::Epilogue,
    intro: "Maintenant, on rentre.",
    mecanique: "Épilogue : le niveau vertical à l'envers ; on descend du soleil jusqu'aux \
                toits, en tombant ou en planant, entre les pigeons et la grêle.",
    // Le plus court des onze : une descente tranquille (la mesure de difficulté la
    // joue en 18 à 19 s ; le robot de faisabilité, en course rapide, en 14 s).
    duree_cible_s: 40,
    ascii: ASCII,
    aiguilles: &[],
};

/// Dessin du niveau (32 x 45 tuiles). Légende : voir `crate::level`.
/// - Départ : nuage des colonnes 0 à 5 (ligne 6), Pépin colonne 2, deux oiseaux.
/// - Couches de nuages : lignes 15 et 33 (trou colonnes 24 à 29), ligne 24 (trou
///   colonnes 2 à 7).
/// - Pigeons : (15, 10) et (21, 10), chacun sur deux nuages (ligne 11), tournés
///   vers la gauche (vers Pépin qui arrive de la gauche).
/// - Grêle : colonne 2 de la ligne 16 au nuage de la ligne 33 ; colonne 29 de la
///   ligne 25 au toit (ligne 42).
/// - Gouttes : (9, 10), (4, 21), (27, 30) ; lanternes là où Pépin se pose : (8,
///   14), (26, 23), (6, 32), (26, 41).
/// - Arrivée : le toit (ligne 42, briques dessous), drapeau (4, 41).
const ASCII: &str = r"
................................
................................
................................
................................
................................
K.@.K...........................
NNNNNN..........................
................................
................................
................................
.........o.....p.....p..........
...............NN....NN.........
................................
................................
........L.......................
NNNNNNNNNNNNNNNNNNNNNNNN......NN
..g.............................
................................
................................
................................
................................
....o...........................
................................
..........................L.....
NN......NNNNNNNNNNNNNNNNNNNNNNNN
.............................g..
................................
................................
................................
................................
...........................o....
................................
......L.........................
NNNNNNNNNNNNNNNNNNNNNNNN......NN
................................
................................
................................
................................
................................
................................
.........BB.....................
....X....BB...T.....H.....L.....
################################
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::TAILLE_TUILE as T;
    use crate::entities::contacts::{resoudre_contacts, verifier_vide, Contact};
    use crate::entities::{solides_des_entites, Actualisable, Entite, TypeEnnemi};
    use crate::physics::{Environnement, JOUEUR_H};
    use crate::player::{Direction, Entrees, Joueur};

    /// Le plus calme des onze niveaux : le toit de l'arrivée couvre toute la largeur
    /// (aucune chute ne tue : tout ce qui tombe finit sur une couche ou sur le toit),
    /// aucune pointe, aucun éclair ni rien qui bouge ; pour seuls dangers, deux
    /// pigeons et deux colonnes de grêle.
    #[test]
    fn aucune_chute_ne_tue_et_seuls_deux_pigeons_et_deux_colonnes_de_grele_menacent() {
        let n = INFOS.charger().unwrap();
        assert_eq!((n.largeur(), n.hauteur()), (32, 45));
        for tx in 0..n.largeur() as i32 {
            assert!(
                n.tuile_decor(tx, 42).est_solide(),
                "toit percé en colonne {tx}"
            );
        }
        let (mut pigeons, mut greles) = (0, 0);
        for e in &n.entites {
            match e {
                Entite::Ennemi(p) if p.type_ennemi == TypeEnnemi::Pigeon => pigeons += 1,
                Entite::Grele(_) => greles += 1,
                Entite::Goutte(_) | Entite::Lanterne(_) => {}
                autre => {
                    panic!("l'épilogue n'a que pigeons, grêle, gouttes et lanternes : {autre:?}")
                }
            }
        }
        assert_eq!((pigeons, greles), (2, 2));
        assert!(n.courants.is_empty() && n.vents.is_empty());
    }

    /// Se laisser tomber du bord d'un trou en courant (direction gardée, sans saut ni
    /// parapluie) pose Pépin sur la couche du dessous (ou sur le toit), sans toucher la
    /// colonne de grêle qui longe le bord lointain du trou. Vraie physique, pas à pas.
    #[test]
    fn se_laisser_tomber_du_bord_d_un_trou_ne_touche_pas_la_grele() {
        let n = INFOS.charger().unwrap();
        // (colonne de départ, sens de la course, rangée de la couche, rangée d'arrivée)
        for (colonne, sens, de, vers) in [
            (20, Direction::Droite, 15, 24),
            (11, Direction::Gauche, 24, 33),
            (20, Direction::Droite, 33, 42),
        ] {
            let mut entites = n.entites.clone();
            let mut j = Joueur::new(colonne as f32 * T, de as f32 * T - JOUEUR_H);
            let entrees = Entrees {
                direction: sens,
                saut: false,
            };
            let mut pose = None;
            for _ in 0..600 {
                let solides = solides_des_entites(&entites);
                let env = Environnement {
                    grille: &n,
                    solides: &solides,
                    courants: &n.courants,
                    vents: &n.vents,
                };
                for e in entites.iter_mut() {
                    e.actualiser(&env);
                }
                j.actualiser(&entrees, &env);
                let contacts = resoudre_contacts(&mut j, &mut entites);
                assert!(
                    !contacts.iter().any(|c| matches!(c, Contact::Mort(_))),
                    "couche {de} : mort en {:?}",
                    (j.boite.x, j.boite.y)
                );
                assert!(verifier_vide(&j, n.hauteur_px()).is_none());
                if j.au_sol && j.boite.bas() > de as f32 * T + 1.0 {
                    pose = Some(j.boite.bas());
                    break;
                }
            }
            assert_eq!(pose, Some(vers as f32 * T), "couche {de}");
        }
    }
}
