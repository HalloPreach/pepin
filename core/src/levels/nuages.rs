//! Niveau 8 : « Au-dessus des nuages ».
//!
//! Final tout en douceur sur un tapis de nuages, jusqu'au soleil. Les îlots
//! d'atterrissage sont larges et les nuages friables se touchent ; les gouttes
//! d'or restent des détours facultatifs, plus exigeants. Sur le long toit de
//! nuages de la fin, un pigeon guette depuis un petit nuage (colonne 108,
//! 4 rangées plus haut), à 13 tuiles de l'oiseau de décor (colonne 95) : on
//! les voit ensemble, on ne les confond pas.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 8,
    nom: "Au-dessus des nuages",
    theme: Theme::Soleil,
    genre: GenreNiveau::Normal,
    intro: "Là-haut, le soleil. Prends ton temps : il n'y a plus d'orage.",
    mecanique: "Final calme : nuages, trampoline, petite traversée de nuages friables, pigeon.",
    duree_cible_s: 45,
    ascii: ASCII,
    aiguilles: &[],
};

/// Dessin du niveau (150 x 11 tuiles). Légende : voir `crate::level`.
const ASCII: &str = r"
......................................................................................................................................................
............................................................................o.........................................................................
...................................................................................................................................K....L.......X.....
......................................o.....................................................................p...................NNNNNNNNNNNNNNNNNNNNNN
..................................................L...K.....................................................N............NNNNNNN......................
...........................................NNNNNNNNNNNNNNN..FFFFFFF................................................K..................................
.........o....................K....NNNNNN...................................M....................................NNNNNN...............................
..@...K..............L.....NNNNNN...................................NNNNNNNNNNNNNNN............K......L........NN.....................................
NNNNNNNNNNNNNNN..NNNNNNNN...............................................................NNNNNNNNNNNNNNNNNNNNNNN.......................................
......................................................................................................................................................
......................................................................................................................................................
";
