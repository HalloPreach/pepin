//! Niveau 10 (bonus) : « L'arc-en-ciel ».
//!
//! « Il ne tient que le temps de le traverser. » Le niveau BONUS : on n'y entre
//! qu'avec TOUTES les gouttes d'or des neuf niveaux normaux, ramassées pendant la
//! partie en cours (`levels::apres`) ; sinon on passe directement au suivant.
//!
//! Un pont de nuages friables (`F`) d'un bout à l'autre, sans ennemi ni lanterne :
//! tout s'effrite derrière Pépin (une dalle touchée tombe une demi-seconde plus
//! tard, `entities::friable`), donc on ne s'arrête jamais. Un sprint d'une minute,
//! fait pour le chronomètre. Seuls le nuage du départ et celui de l'arrivée sont
//! solides ; une chute ramène au départ (aucune lanterne), où le pont s'est déjà
//! reformé (`entities::reprendre_apres_une_mort`).
//!
//! Nouveauté du niveau : le pont qui ne pardonne pas l'arrêt (aucun appui sûr
//! entre le départ et l'arrivée), sous un grand arc-en-ciel qui glisse lentement
//! derrière Pépin (thème `ArcEnCiel` : toute la palette).
//!
//! Le parcours, de gauche à droite, comme un arc-en-ciel qu'on enjambe :
//! 1. le départ : le pont au ras de la mer de nuages, un premier vide de deux
//!    tuiles (goutte au sommet du saut) ;
//! 2. le premier arc : quatre marches de six tuiles, une rangée plus haut chaque
//!    fois, puis le sommet (rangée 3), coupé de deux vides (goutte au-dessus du
//!    premier) ;
//! 3. un long plané de dix tuiles vers le bas (goutte juste avant l'arrivée du
//!    plané), puis le retour au ras de la mer ;
//! 4. le deuxième arc : trois marches de deux rangées, puis trois pas japonais de
//!    deux dalles qui redescendent ;
//! 5. le troisième arc : cinq marches qui remontent, et un second long plané ;
//! 6. le sprint final au ras de la mer, des vides d'une tuile, jusqu'au drapeau.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 10,
    nom: "L'arc-en-ciel",
    theme: Theme::ArcEnCiel,
    genre: GenreNiveau::Bonus,
    intro: "Il ne tient que le temps de le traverser.",
    mecanique: "Bonus : un pont de nuages friables d'un bout à l'autre, sans ennemi ni \
                lanterne ; on ne s'arrête jamais. Un sprint pour le chronomètre.",
    duree_cible_s: 60,
    ascii: ASCII,
    aiguilles: &[],
};

/// Dessin du niveau (240 x 11 tuiles, un écran de haut). Légende : voir `crate::level`.
/// - Départ : nuage solide, colonnes 0 à 5 ; arrivée : nuage solide, colonnes 230 à
///   239, drapeau (235, 7). Deux oiseaux posés (décor) sur chacun.
/// - Tout le reste est un pont de dalles friables : colonnes 6 à 227, rangées 3 à 8.
/// - Gouttes : (16, 4) au-dessus du premier vide, (64, 1) au-dessus du sommet du
///   premier arc, (87, 4) à la fin du premier long plané.
const ASCII: &str = r"
................................................................................................................................................................................................................................................
................................................................o...............................................................................................................................................................................
................................................................................................................................................................................................................................................
.........................................................FFFFFF..FFFFFF..FFFFF..................................................................................................................................................................
................o................................FFFFFF................................o.........................FFFF...FFFFFFFF..........................................FFFFFFFFF.............................................................
.........................................FFFFFF...................................................................................FF................................FFFF........................................................................
.................................FFFFFF.................................................FFFFFFF............FFFF.......................FF......................FFFF..............................................................................
K.@.K....................FFFFFF...........................................................................................................FF............FFFF...................................FFFFFFF..................................K..X..K.
NNNNNNFFFFFFFFF..FFFFFF..........................................................................FFFFFFFF.....................................FFFFFFFF..................................................FFFFFFFF.FFFFFF.FFFFF.FFFFFF..NNNNNNNNNN
NNNNNN................................................................................................................................................................................................................................NNNNNNNNNN
NNNNNN................................................................................................................................................................................................................................NNNNNNNNNN
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::TAILLE_TUILE;
    use crate::entities::Entite;

    /// Un pont de nuages friables d'un bout à l'autre : entre le nuage du départ
    /// (colonnes 0 à 5) et celui de l'arrivée (230 à 239), aucune tuile solide, rien
    /// que des dalles friables ; aucun ennemi, aucune lanterne, ni rien qui porte
    /// (plateforme mobile, montgolfière, champignon...).
    #[test]
    fn un_pont_friable_d_un_bout_a_l_autre_sans_ennemi_ni_lanterne() {
        let n = INFOS.charger().unwrap();
        assert_eq!((n.largeur(), n.hauteur()), (240, 11));
        for ty in 0..n.hauteur() as i32 {
            for tx in 6..230 {
                assert!(
                    !n.tuile_decor(tx, ty).est_solide(),
                    "solide en ({tx}, {ty})"
                );
            }
        }
        let mut friables = 0;
        for e in &n.entites {
            match e {
                Entite::Friable(f) => {
                    friables += 1;
                    let colonne = (f.boite.x / TAILLE_TUILE) as i32;
                    assert!((6..228).contains(&colonne), "dalle en colonne {colonne}");
                }
                Entite::Goutte(_) => {}
                autre => panic!("le bonus n'a que des dalles et des gouttes : {autre:?}"),
            }
        }
        assert!(friables >= 150, "{friables} dalles");
        assert_eq!(n.nb_lanternes(), 0);
        assert!(n.courants.is_empty() && n.vents.is_empty());
    }
}
