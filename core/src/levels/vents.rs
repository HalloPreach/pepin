//! Niveau 7 : « Vents d'altitude ».
//!
//! Colonnes de vent à chevaucher et longs planés entre îlots de nuages ; au
//! milieu de l'escalier de nuages, un îlot friable ; deux corbeaux patrouillent
//! (sur l'îlot d'arrivée du 2e courant et avant le drapeau, `PORTEE_CORBEAU`).
//! Au-dessus du dernier toit, un pigeon guette sur un petit nuage (colonne 167,
//! 4 rangées plus haut) : tourné vers l'arrivée de Pépin, il plonge derrière
//! lui ; il ne touche que celui qui s'arrête ou revient sous lui.
//!
//! Carte 6.4b (« plus d'ennemis ») : deux corbeaux et un pigeon de plus.
//! - Un corbeau au-dessus des nuages friables (colonne 76, rangée 3 : il
//!   patrouille des colonnes 71 à 81), juste au-dessus de la tête de Pépin posé
//!   sur les friables (colonnes 72 à 75) : on passe dessous en marchant, mais il
//!   faut sauter pour en repartir, avant qu'ils ne cèdent, et un saut au mauvais
//!   moment s'y cogne.
//! - Un corbeau à hauteur de Pépin sur l'îlot des colonnes 143 à 151 (colonne 147,
//!   rangée 3 : il patrouille des colonnes 142 à 152, au-delà des deux bords) : on
//!   se laisse tomber dessus depuis l'îlot de la lanterne 136 (il s'écrase), ou on
//!   le saute ; la lanterne 136 est hors de sa patrouille.
//! - Un second pigeon, sur un petit nuage posé pour lui (colonne 38, rangée 4),
//!   4 rangées au-dessus de l'îlot de la lanterne 43, comme celui du dernier toit :
//!   tourné vers l'arrivée de Pépin, qui s'y pose en descendant de l'îlot du
//!   premier courant, il plonge derrière lui (sa plongée finit au ras de l'îlot,
//!   colonne 34, loin de la lanterne).
//!
//! Carte 6.8 (retour d'AGA en jouant : « un courant ascendant qui finit un peu
//! trop tôt et rend le saut compliqué ») : chaque courant lâche maintenant Pépin
//! au moins 32 px (deux tuiles) au-dessus du dessus de son îlot d'arrivée, comme
//! les deux premiers : au-dessus du haut d'un courant, plus rien ne porte Pépin,
//! il plane en descendant.
//! - 3e courant (colonnes 97 à 99) : il commençait à la rangée 3, AU NIVEAU de
//!   l'îlot des colonnes 103 à 108 (0 px de marge, trois tuiles de vide à
//!   planer) ; prolongé vers le haut, il commence à la rangée 1.
//! - 4e courant (colonnes 127 à 129) : il commence déjà à la rangée 1 et la
//!   rangée 0 est le haut de l'écran (en le prolongeant, Pépin planerait hors du
//!   cadre). C'est donc son îlot d'arrivée (colonnes 132 à 141) qui descend d'une
//!   rangée (2 → 3), avec sa lanterne (colonne 136, rangée 1 → 2) : 16 → 32 px.
//!   Il est maintenant à la hauteur du corbeau (147, 3), dont la patrouille
//!   commence 2 px après son bord droit : il ne vient pas sur l'îlot, et sa
//!   boîte commence 3 px sous le dessus de l'îlot (Pépin posé au bord ne le
//!   touche pas).

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    monde: 1,
    numero: 7,
    cle: "vents",
    sans_planer: true,
    nom: "Vents d'altitude",
    theme: Theme::Altitude,
    genre: GenreNiveau::Normal,
    intro: "Le Grand Nuage s'est enfui. Les courants montent : laisse-toi porter, puis plane.",
    mecanique: "Courants ascendants, longs planés, nuage friable, corbeaux, pigeons.",
    duree_cible_s: 55,
    ascii: ASCII,
    aiguilles: &[],
};

/// Dessin du niveau (190 x 11 tuiles). Légende : voir `crate::level`.
/// - Corbeaux (carte 6.4b) : (76, 3) et (147, 3) ; pigeon (38, 3) sur le nuage (38, 4).
/// - Courants (carte 6.8) : colonnes 16-18 (dès la rangée 2), 50-52, 97-99 et
///   127-129 (dès la rangée 1) ; îlot de la lanterne 136 à la rangée 3.
const ASCII: &str = r"
..............................................................................................................................................................................................
............................o.....................vvv............................................vvv.....o.....................vvv............................o...............................
................vvv...............................vvv.......c....................................vvv...........................vvv......L.....................................................
................vvv....L..............p...........vvv...NNNNNNNN............c....................vvv...NNNNNN..................vvv..NNNNNNNNNN.....c...................p......................
................vvv..NNNNNNNNNNN......N...........vvv.............NNNN...........................vvv...................L....Q..vvv.............NNNNNNNNN...............N......................
................vvv...............................vvv...................FFFF.....................vvv.............NNNNNNNNNNNNN.vvv............................M...............................
................vvv...............................vvv.........................NNNN...............vvv...........................vvv.......................NNNNNNNNN............................
..@........W.Q..vvv........................L..Q...vvv...................................L...Q....vvv...........................vvv........................................L.......c.....X.....
###############.vvv...............NNNNNNNNNNNNNN..vvv...............................NNNNNNNNNN...vvv...........................vvv.................................NNNNNNNNNNNNNNNNNNNNNNNNNNN
BBBBBBBBBBBBBBB.vvv...............................vvv............................................vvv...........................vvv............................................................
BBBBBBBBBBBBBBB.vvv...............................vvv............................................vvv...........................vvv............................................................
";
