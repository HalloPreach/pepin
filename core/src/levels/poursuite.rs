//! Niveau 10 : « Le mur de pluie » (carte 8.2), le SOMMET du jeu.
//!
//! L'orage n'a pas dit son dernier mot : un rideau de pluie avance depuis la
//! gauche (`P`, voir `crate::poursuite`) et l'écran défile avec lui. Pépin court
//! vers le soleil en enchaînant, sous pression, les mécaniques déjà connues :
//! - départ sur les toits de nuages, une marche, un pont FRIABLE (colonnes 41 à
//!   50) ;
//! - un long VENT ARRIÈRE (colonnes 58 à 73) : parapluie ouvert, il emporte Pépin
//!   par-dessus le vide ;
//! - un COURANT ASCENDANT (colonnes 87 à 89) devant une falaise de nuages : on y
//!   monte en planant, puis on plane jusqu'au plateau (rangée 3) ;
//! - deux colonnes de GRÊLE (colonnes 117 et 125) : on les traverse en planant ;
//! - un second pont friable sous un corbeau (colonne 146, rangée 3) : on passe
//!   dessous en marchant ;
//! - la dernière ligne droite : à 24 tuiles du drapeau (`PERCEE_AVANT_ARRIVEE`,
//!   en arrivant sur le long toit de nuages), le soleil perce et la pluie recule.
//!
//! Trois gouttes d'or presque sur le chemin (un saut au bon moment : colonnes 19,
//! 98 et 160) et trois lanternes (colonnes 54, 102 et 134).

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 10,
    nom: "Le mur de pluie",
    theme: Theme::Altitude,
    genre: GenreNiveau::Normal,
    intro: "L'orage revient en force : cours vers le soleil, ne te retourne pas.",
    mecanique: "Poursuite : un mur de pluie avance et fait défiler l'écran ; vent arrière, \
                courant, grêle, ponts friables, corbeau.",
    duree_cible_s: 45,
    ascii: ASCII,
    aiguilles: &[],
};

/// Dessin du niveau (200 x 11 tuiles). Légende : voir `crate::level` (`P` : le
/// départ du mur de pluie, tout à gauche).
const ASCII: &str = r"
.....................................................................................................................g.......g..........................................................................
.......................................................................................vvv........o.....................................................................................................
..........................................................>>>>>>>>>>>>>>>>.............vvv............L.................................................................................................
..........................................................>>>>>>>>>>>>>>>>.............vvv..NNNNNNNNNNNNN.........................................c.....................................................
..........................................................>>>>>>>>>>>>>>>>.............vvv..NN..................................................................o.......................................
...................o..................................L.Q.>>>>>>>>>>>>>>>>.............vvv..NN..........................................................................................................
...............................NNNNNNNN..FFFFFFFFFF.NNNNNN>>>>>>>>>>>>>>>>.............vvv..NN......................................................................................K.....K.......X.....
P...K....@................................................>>>>>>>>>>>>>>>>.........Q...vvv..NN........................................L.....FFFFFFFFFF.NNNNNNNN...NNNNNNN...NNNNNNNNNNNNNNNNNNNNNNNNNNNN
NNNNNNNNNNNNNNNNN...NNNNNNNNN..............................................NNNNNNNNNN..vvv..NN.............NNNNNNN......NNNNNNNNNNNNNNNNNN..............................................................
.......................................................................................vvv..............................................................................................................
........................................................................................................................................................................................................
";
