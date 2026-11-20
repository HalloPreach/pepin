//! Monde 2, niveau 5 : « La rivière » (cartes 9.10 et 9.10b). Au pied de la
//! cascade, l'eau DESCEND une vallée par paliers : une descente en barque
//! (parapluie retourné, flèche du BAS) de bief en bief, deux grandes chutes, et
//! deux endroits où l'on CHOISIT entre le chemin du haut, à pied (plus sûr, une
//! goutte), et la barque en bas (plus rapide, plus risquée). Difficulté : entre la
//! grotte et la Pieuvre du port.
//!
//! Lecture du niveau (206 x 34 tuiles ; l'eau descend des rangées 9, 16, 23 puis
//! 29), de gauche à droite :
//! 1. la berge du départ (colonnes 0 à 9), le panneau « barque » ; le BIEF 1
//!    (colonnes 10 à 33), deux poissons sous une chauve-souris ; la berge de la
//!    1re lanterne (colonne 37) ;
//! 2. la GORGE (colonnes 42 à 93), 1er CHOIX : en haut, le chemin des PONTS de bois
//!    et des corniches, à sauter de l'un à l'autre (la 1re goutte au-dessus de la
//!    2e brèche, une grenouille sur le pont du milieu) ; en bas, on se laisse
//!    tomber par une brèche dans le BIEF 2, sept rangées plus bas : deux poissons,
//!    une chauve-souris sous le pont, un rocher au milieu du courant (colonnes 70
//!    et 71) à sauter. Les deux chemins se rejoignent à la 2e lanterne (colonne 96) ;
//! 3. le BIEF 3 (colonnes 102 à 117), son poisson, puis la 1re GRANDE CHUTE
//!    (colonnes 118 et 119, sept rangées) : on la descend en planant jusqu'au
//!    BIEF 4 ;
//! 4. le BIEF 4 (colonnes 118 à 145), coupé par l'ÎLOT de la 3e lanterne (colonnes
//!    130 à 134) : un poisson et une chauve-souris avant, un poisson et la 2e goutte
//!    après ;
//! 5. la 2e GRANDE CHUTE (colonnes 146 et 147, six rangées), 2e CHOIX : on saute
//!    par-dessus vers le PLATEAU (en haut : la MARE, ses nénuphars qui s'enfoncent,
//!    une grenouille, un TRONC FLOTTANT qui va et vient, puis un rocher dont le pied
//!    cache la 3e goutte derrière un faux mur) ; ou l'on tombe dans la chute vers
//!    le BIEF 5, qui passe SOUS le plateau par la PETITE GROTTE (deux
//!    chauves-souris) ;
//! 6. la berge du drapeau (colonne 199), gardée par une grenouille.
//!
//! Chaque chute finit dans l'eau. Les poissons sautent au-dessus d'un courant,
//! jamais d'une eau calme ni sous un plafond ; les chauves-souris (`c`) volent
//! au-dessus de l'eau, loin des lanternes. Décor : des oiseaux posés (`K`) sur les
//! berges. Tuiles du thème : `game/src/sprites/tuiles_monde2/riviere.rs` ; le fond
//! (vallée au soleil couchant) : `game/src/decor_riviere.rs`.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    monde: 2,
    // Sa place dans le Monde 2 de SA branche : la 5e, avant la Pieuvre (le
    // coordinateur range l'ordre final à la fusion).
    numero: 5,
    cle: "riviere",
    sans_planer: false,
    nom: "La rivière",
    theme: Theme::Riviere,
    genre: GenreNiveau::Normal,
    intro: "L'orage est passé. La rivière descend la vallée : par les ponts, ou en barque.",
    mecanique: "Choisir : le chemin du haut, à pied, de pont en pont ; ou la barque en bas, \
                plus rapide. Descendre les grandes chutes en planant ; traverser la mare \
                de nénuphar en tronc ; laisser retomber les poissons.",
    duree_cible_s: 60,
    ascii: ASCII,
    aiguilles: &[],
    poissons_hauts: &[],
};

/// Dessin du niveau (206 x 34 tuiles). Légende : voir `crate::level`.
const ASCII: &str = r"
..............................................................................................................................................................................................................
..............................................................................................................................................................................................................
..............................................................................................................................................................................................................
..............................................................................................................................................................................................................
..............................................................................................................................................................................................................
......................c.......................................................................................................................................................................................
............................................................o.................................................................................................................................................
..............................................................................................................................................................................................................
..@...Z.K.......i...........i........L..............................j.........................................................................................................................................
BBBBBBBBBB}}}}}}}}}}}}}}}}}}}}}}}}BBBBBBBB#########...#####...###########...#####...##########................................................................................................................
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB............#####.................#####.............................................................................................................................
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB....................................................................................................................................................................
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB..................c.................................................................................................................................................
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB....................................................................................................................................................................
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB............................##......................................................................................................................................
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB................i...........##............i...........L............i................................................................................................
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB}}}}}}}}}}}}}}}}}}}}}}}}}}}}##}}}}}}}}}}}}}}}}}}}}BBBBBBBBBB}}}}}}}}}}}}}}}}||......................................................................................
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB||......................................................................................
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB||......................................................................................
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB||....c..............o..................................................................
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB||......................................................................................
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB||....................................................j...###...........................
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB||....i.......L..........i......K.....n..n..1.....1...n...mo#...........................
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB}}}}}}}}}}}}BBBBB}}}}}}}}}}}||BBBBBB~~~~~~~~~~~~~~~~~~~~~BBBBBBBBBB.....................
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB||BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB.....................
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB||BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB.....................
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB||..........c............c..................................
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB||..........................................................
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB||..........................................j........X...K..
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}BBBBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::TAILLE_TUILE;
    use crate::entities::Entite;
    use crate::level::{Niveau, Tuile};

    /// Comme à la cascade (carte 9.26c) : les deux chutes (bief 2 vers 3, bief 4
    /// vers 5) finissent dans l'eau, jamais sur un appui.
    #[test]
    fn la_chute_finit_dans_l_eau() {
        let n = Niveau::depuis_ascii(ASCII).expect("niveau valide");
        let t = TAILLE_TUILE;
        let mut chutes = 0;
        for e in &n.entites {
            let Entite::Chute(c) = e else { continue };
            chutes += 1;
            let pied = (c.zone.bas() / t) as i32;
            for tx in (c.zone.x / t) as i32..(c.zone.droite() / t) as i32 {
                assert_eq!(n.tuile_decor(tx, pied), Tuile::Eau, "chute ({tx}, {pied})");
            }
        }
        assert_eq!(chutes, 2);
    }
}
