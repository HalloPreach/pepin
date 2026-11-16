//! Niveau 2 : « Gouttières et cheminées ».
//!
//! Flaques patrouilleuses (on les écrase en sautant dessus), trampolines et gouttières suspendues.
//!
//! Deux corbeaux (carte 6.4a), qui volent DEUX rangées au-dessus des toits
//! (rangée 5) : on passe dessous en marchant, on saute par-dessus une flaque d'un
//! petit bond, mais un grand saut au mauvais moment s'y cogne. Le premier patrouille
//! au-dessus des deux flaques du milieu (colonnes 45 à 55, avant le champignon de la
//! colonne 57, hors de son rebond) ; le second au-dessus de la lanterne 104 et de la
//! flaque 108, entre la cheminée haute (100-101, où il fait demi-tour) et le
//! champignon de la colonne 112 (hors de son rebond, colonne 111 au plus). À la
//! lanterne, Pépin réapparu debout passe dessous : rien ne le touche s'il ne saute pas.
//!
//! Secret (carte 8.5) : la façade gauche du grand immeuble (colonnes 60-61,
//! rangées 5 et 6, au-dessus du toit où marchent les flaques) est un faux mur
//! `m` ; l'indice est la brique décalée du bas. On y saute depuis le pied du
//! champignon : derrière, une salle de deux cases (colonne 62) garde la goutte
//! qui flottait au-dessus de l'immeuble.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 2,
    nom: "Gouttières et cheminées",
    theme: Theme::Gouttieres,
    genre: GenreNiveau::Normal,
    intro: "L'averse redouble. Les flaques rôdent entre les cheminées : saute-leur dessus !",
    mecanique: "Flaques à écraser, champignons-trampolines, gouttières suspendues, corbeaux.",
    duree_cible_s: 55,
    ascii: ASCII,
    aiguilles: &[],
};

/// Dessin du niveau (190 x 11 tuiles). Légende : voir `crate::level`.
/// - Corbeaux (rangée 5) : colonnes 50 et 106.
const ASCII: &str = r"
..............................................................................................................................................................................................
................................................................................................................o.............................................................................
..............................................................................................................................................................................................
............................................................BB....L...f.......................................................................................................................
............................................................BB#####################.................................................F.........................................................
..................................................c.........mm.BBBBBBBBBBBBBBBBBBBB.................BB....c...................................................................................
.........o..............BB..............BB..................mmoBBBBBBBBBBBBBBBBBBBB.........BB......BB.............................###.....................BB.........BB......................
..@.................f...BB........L..H..BB....f.....f....M..BBBBBBBBBBBBBBBBBBBBBBB.......f.BB...f..BB..L...f...M.............###.......###...........f....BB...f.....BB.T..L....H......X..T..
############################...###############################BBBBBBBBBBBBBBBBBBBBB...#################################..###.................###..############################################
BBBBBBBBBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB...........................BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB...........................BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";
