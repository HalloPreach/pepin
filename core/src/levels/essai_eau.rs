//! Niveau d'essai « eau » (carte 9.2) : l'écran de la mécanique du Monde 2,
//! le parapluie retourné devient une barque. Hors de la table `NIVEAUX`, il
//! n'existe QUE dans les tests et dans le build de QA (fonctionnalité
//! `debug_niveaux`, touche F11) ; en production il n'est pas compilé dans le
//! binaire, comme les autres niveaux d'essai.
//!
//! Lecture de l'écran (un écran plat, 36 x 11 tuiles) :
//! - le sol (lignes 9 et 10) porte la course ; le bassin est creusé dans la
//!   ligne 9 (colonnes 6 à 25), sa surface au ras du sol (y = 144), avec le
//!   fond solide dessous (ligne 10). On marche au BORD sans rien toucher ; un
//!   pas de plus, les pieds passent sous la surface : Pépin COULE
//!   (`CauseMort::Noyade`), une vie perdue ;
//! - on y entre parapluie RETOURNÉ (BAS tenu, carte 9.2b ; en planant, on
//!   coule) : Pépin devient une BARQUE (eau calme `~`,
//!   colonnes 6 à 8), puis le courant d'eau `}` (colonnes 9 à 25) l'emporte
//!   vers la droite, jusqu'au QUAI (colonnes 26 à 35, une tuile plus haut que
//!   le sol) ; on en sort d'un SAUT depuis la barque ;
//! - le bassin (320 px) est trop large pour un saut planant depuis le sol (il
//!   retombe dans l'eau vers 140 px) : la barque est le SEUL passage ;
//! - posées SUR l'eau (la ligne au-dessus de la surface, comme une goutte est
//!   posée sur le sol) : une lanterne (colonne 14) et la goutte de la barque
//!   (colonne 21), qu'on n'atteint qu'en passant par la barque.
//!   La lanterne reste un point de réapparition jouable : Pépin, protégé à sa
//!   réapparition, retombe en barque au lieu de couler ;
//! - les deux autres gouttes : au sol avant le bassin, et sur le quai ;
//! - le robot de QA ramasse les trois gouttes SANS MOURIR (les plans « à
//!   bord » sont gérés par la recherche : `Fin::Barque` est un point de
//!   décision comme un atterrissage).

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 0,
    // Essai hors table : le monde ne sert qu'aux règles de la table (champ ajouté par la carte 9.1).
    monde: 1,
    cle: "essai_eau",
    sans_planer: false,
    nom: "Essai eau",
    // Décor du caniveau (carte 9.5) : le premier niveau du Monde 2 y apprendra la barque.
    theme: Theme::Caniveau,
    genre: GenreNiveau::Normal,
    intro: "BAS retourne le parapluie : tombé ainsi dans l'eau, Pépin flotte dans une barque. Parapluie fermé ou ouvert, il coule. Le courant d'eau l'emporte vers le quai.",
    mecanique: "Tenir BAS en entrant dans l'eau pour tenir la barque ; ramer (lentement) et laisser le courant porter ; sauter sur le quai depuis la barque.",
    duree_cible_s: 45,
    ascii: ASCII,
    aiguilles: &[],
};

/// Écran plat (36 x 11 tuiles), thème Orage.
/// - Départ à la colonne 1, goutte au sol (colonne 3), lanterne (colonne 4).
/// - Bassin : ligne 9, colonnes 6 à 25 (x de 96 à 416). La surface est le haut
///   de la ligne 9 (y = 144), au ras du sol : la barque y flotte, les pieds de
///   Pépin 3 px sous la surface (`ENFONCEMENT_BARQUE`). Eau calme `~` aux
///   colonnes 6 à 8 (on y entre), courant vers la droite `}` aux colonnes 9
///   à 25 (24 px/s, plus lent que la rame : on peut ramer contre).
/// - Sur l'eau (ligne 8) : une lanterne (colonne 14) et une goutte (colonne
///   21, x de 340 à 348, au ras de l'eau) qu'on n'atteint QU'EN PASSANT PAR
///   LA BARQUE : à bord, ou en planant après un saut depuis la barque (puis
///   BAS avant de retoucher l'eau, carte 9.2b). Un saut
///   planant depuis le sol retombe dans l'eau vers x = 200, et depuis le quai
///   on la survole bien trop haut.
/// - Quai : colonnes 26 à 35, lignes 8 et 9 (le dessus à y = 128, 16 px
///   au-dessus de l'eau). La barque bute contre son mur ; le saut depuis la
///   barque (pieds à 147, sommet vers 90) passe largement par-dessus. Ce saut
///   vaut un saut du sol (il part 3 px plus bas) : le robot de QA s'en sert
///   pour « ricocher » (toucher l'eau parapluie retourné, ressauter aussitôt de
///   la barque, planer de nouveau).
/// - Sur le quai : la 3e goutte (colonne 30) et le drapeau (colonne 33).
const ASCII: &str = r#"
....................................
....................................
....................................
....................................
....................................
....................................
....................................
..............................o..X..
.@.oL.........L......o....##########
######~~~}}}}}}}}}}}}}}}}}##########
####################################
"#;
