//! Niveau 9 : « Au-dessus des nuages ».
//!
//! Final tout en douceur sur un tapis de nuages, jusqu'au soleil. Les îlots
//! d'atterrissage sont larges et les nuages friables se touchent ; les gouttes
//! d'or restent des détours facultatifs, plus exigeants. Sur le long toit de
//! nuages de la fin, un pigeon guette depuis un petit nuage (colonne 108,
//! 4 rangées plus haut), à 13 tuiles de l'oiseau de décor (colonne 95) : on
//! les voit ensemble, on ne les confond pas.
//!
//! Carte 6.4b (« plus d'ennemis », sans quitter le calme : le niveau reste sous
//! les vents d'altitude dans la mesure de difficulté) : deux corbeaux et une
//! flaque.
//! - Une flaque sur le petit nuage des colonnes 35 à 40 (rangée 6), entre deux
//!   sauts de l'escalier du début : on lui saute dessus, ou par-dessus.
//! - Un corbeau au-dessus de la traversée friable (colonne 63, rangée 3 : il
//!   patrouille des colonnes 58 à 68), juste au-dessus de la tête de Pépin posé
//!   sur les friables : on passe dessous en marchant ; un saut au mauvais moment
//!   s'y cogne.
//! - Un corbeau au-dessus du nuage des colonnes 121 à 127 (colonne 120, rangée 2 :
//!   colonnes 115 à 125), juste au-dessus de la tête de Pépin posé sur ce nuage,
//!   avant le saut vers le dernier toit : on attend qu'il s'éloigne pour sauter.
//!
//! Une seconde flaque, sous ce dernier corbeau (colonne 124), a été retirée : sur
//! six graines de la mesure, elle faisait passer le niveau au-dessus des vents
//! pour l'Habitué une fois sur six, et hors de sa bande trois fois sur six (QA.md,
//! carte 6.4b).

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    monde: 1,
    numero: 9,
    cle: "nuages",
    sans_planer: true,
    nom: "Au-dessus des nuages",
    theme: Theme::Soleil,
    genre: GenreNiveau::Normal,
    intro: "Là-haut, le soleil. Prends ton temps : l'orage gronde encore en bas.",
    mecanique: "Final calme : nuages, trampoline, petite traversée de nuages friables, pigeon, \
                corbeaux, flaque.",
    duree_cible_s: 45,
    ascii: ASCII,
    aiguilles: &[],
    poissons_hauts: &[],
};

/// Dessin du niveau (150 x 11 tuiles). Légende : voir `crate::level`.
/// - Ennemis (carte 6.4b) : flaque (37, 5) ; corbeaux (63, 3) et (120, 2).
const ASCII: &str = r"
......................................................................................................................................................
............................................................................o.........................................................................
........................................................................................................................c..........K....L.......X.....
......................................o........................c............................................p...................NNNNNNNNNNNNNNNNNNNNNN
..................................................L...K.....................................................N............NNNNNNN......................
.....................................f.....NNNNNNNNNNNNNNN..FFFFFFF................................................K..................................
.........o....................K....NNNNNN...................................M....................................NNNNNN...............................
..@...K..............L.....NNNNNN...................................NNNNNNNNNNNNNNN............K......L........NN.....................................
NNNNNNNNNNNNNNN..NNNNNNNN...............................................................NNNNNNNNNNNNNNNNNNNNNNN.......................................
......................................................................................................................................................
......................................................................................................................................................
";
