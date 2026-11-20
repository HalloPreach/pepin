//! Monde 2, niveau du boss : « La Pieuvre du port » (carte 9.12b). Le boss du
//! Monde 2, « Là où va la pluie » : à l'entrée du port, la Pieuvre garde le
//! bassin. Il n'y a pas de drapeau : le niveau est fini quand la Pieuvre
//! (lettre `8`, carte 9.12a) est vaincue.
//!
//! Le combat (voir `entities::pieuvre`) :
//! 1. la pieuvre est calme (prélude), puis frappe par COLONNES de tentacules
//!    (remous et ombre, annonce, puis le bout qui se dresse, mortel, puis
//!    redescend) : on quitte la colonne annoncée en courant sur le platelage
//!    (carte 9.12c), en sautant ses trous d'une case ;
//! 2. après chaque série, elle sort la TÊTE sur l'eau (au milieu, puis à
//!    gauche, puis à droite) : on tombe dans le trou qui la borde, sur elle (un
//!    point) ;
//! 3. trois motifs de plus en plus rapides, trois coups, elle replonge et
//!    lâche la clé : le niveau est fini.
//!
//! Une mort ramène à la lanterne (au début de l'arène, la seule) ; la pieuvre
//! reprend son calme mais GARDE les points perdus.
//!
//! L'arène reprend la géométrie du niveau d'essai « pieuvre » (`essai_pieuvre`),
//! sur laquelle le robot de QA a appris à battre la pieuvre : mêmes murs, même
//! bassin, même place de la pieuvre, des pontons, des gouttes et de la
//! lanterne. Seules changent les INFOS (un vrai niveau du Monde 2, pas un
//! essai hors table).
//!
//! Le rendu (carte 9.20e, `game/src/dessin_port.rs`) en fait le BOUT D'UNE JETÉE
//! au milieu du port : les murs `B` des bords se dessinent en quais (grue à
//! gauche, pile de caisses à droite), le port au fond (mer, phare rayé, bateaux),
//! le bassin en eau profonde. La grille n'a pas changé.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    monde: 2,
    // Sa place dans le Monde 2 de SA branche (recette de l'étape 9) : après le
    // caniveau (numéro 1), donc 2 ici. Sa place finale est la 7e (avant la marée) :
    // à la fusion, le coordinateur range la ligne et corrige les numéros
    // (`verifier_ordre` refuse un 7 tant que les niveaux 2 à 6 manquent).
    numero: 7,
    cle: "pieuvre",
    sans_planer: false,
    nom: "La Pieuvre du port",
    theme: Theme::Port,
    genre: GenreNiveau::Normal,
    intro: "La Pieuvre garde le port. Elle ne te laissera pas passer.",
    mecanique: "La Pieuvre du port (boss) : quitter la colonne du tentacule annoncé, sauter sur \
                la tête sortie trois fois (jamais la toucher de côté).",
    duree_cible_s: 45,
    ascii: ASCII,
    aiguilles: &[],
    poissons_hauts: &[],
};

/// 20 x 11 tuiles (un seul écran), thème Port. Géométrie du niveau d'essai
/// « pieuvre » : bassin de toute l'arène, la pieuvre `8` (ses 9 colonnes de
/// tentacules de 2 cases couvrent tout le bassin, aucun coin à l'abri), le
/// platelage percé d'un trou au bord de chaque tête (carte 9.12c ; carte 9.12e :
/// plus de petit ponton au-dessus de la tête du milieu), les trois gouttes sous le
/// départ (prises en tombant) et la lanterne.
const ASCII: &str = r"
B@.................B
Bo.................B
Bo.................B
Bo.................B
B..................B
B.L................B
B####.##....##.####B
B#....##....##....#B
B#....##....##....#B
B~~~~~~~~8~~~~~~~~~B
BBBBBBBBBBBBBBBBBBBB
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::Aabb;
    use crate::entities::Entite;
    use crate::levels::essai_pieuvre;

    /// L'arène du boss : un écran, pas de drapeau, une seule pieuvre, une seule
    /// lanterne (au départ), trois gouttes ; et EXACTEMENT la géométrie du niveau
    /// d'essai « pieuvre » (mêmes cases vides ou pleines, même pieuvre, même
    /// bassin, mêmes pontons, mêmes gouttes, même départ), sur laquelle le combat a
    /// été réglé et le robot de QA entraîné.
    #[test]
    fn l_arene_de_la_pieuvre_reprend_celle_du_niveau_d_essai() {
        let n = INFOS.charger().unwrap();
        let e = essai_pieuvre::INFOS.charger().unwrap();
        assert_eq!((n.largeur(), n.hauteur()), (20, 11));
        assert!(n.arrivee.is_none(), "l'arène n'a pas de drapeau");
        assert_eq!(n.nb_lanternes(), 1);
        assert_eq!(n.depart, e.depart);
        for ty in 0..11 {
            for tx in 0..20 {
                assert_eq!(
                    n.tuile_decor(tx, ty).est_solide(),
                    e.tuile_decor(tx, ty).est_solide(),
                    "case ({tx}, {ty})"
                );
            }
        }
        let boites =
            |entites: &[Entite]| -> Vec<Aabb> { entites.iter().map(|e| e.boite()).collect() };
        assert_eq!(boites(&n.entites), boites(&e.entites));
        let pieuvres = |entites: &[Entite]| -> Vec<_> {
            entites
                .iter()
                .filter_map(|e| match e {
                    Entite::Pieuvre(p) => Some((p.colonnes, p.places)),
                    _ => None,
                })
                .collect()
        };
        assert_eq!(pieuvres(&n.entites).len(), 1);
        assert_eq!(pieuvres(&n.entites), pieuvres(&e.entites));
        // Carte 9.12f : les 4 appuis de l'arène (les 2 grosses plateformes des
        // bords, les 2 petites sur piliers) sont ceux qu'elle peut saisir.
        let appuis = n.entites.iter().find_map(|e| match e {
            Entite::Pieuvre(p) => Some(p.plateformes.len()),
            _ => None,
        });
        assert_eq!(appuis, Some(4));
    }
}
