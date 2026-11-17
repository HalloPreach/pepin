//! Niveau d'essai « pieuvre » : l'arène de la Pieuvre du port, le boss du Monde 2
//! (carte 9.12a ; son vrai niveau viendra avec la carte 9.12b). Hors de la table
//! `NIVEAUX`, il n'existe QUE dans les tests et dans le build de QA (fonctionnalité
//! `debug_niveaux`, touche L) ; en production il n'est pas compilé dans le binaire,
//! comme les autres niveaux d'essai.
//!
//! La Pieuvre vit sous l'eau du bassin, qui fait toute la largeur de l'arène. Ses
//! tentacules frappent par colonnes annoncées (remous et ombre, puis le bout qui
//! pointe) ; Pépin saute de ponton en caisse pour quitter la colonne à temps (ou
//! rame en barque). Après chaque série, elle sort la tête sur l'eau (au milieu, puis
//! à gauche, puis à droite) : on lui saute dessus. Trois coups, elle replonge et
//! lâche une clé : le niveau est fini. Il n'y a pas de drapeau.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 0,
    // Essai hors table : le monde ne sert qu'aux règles de la table.
    monde: 1,
    cle: "essai_pieuvre",
    sans_planer: false,
    nom: "Essai pieuvre",
    theme: Theme::Port,
    genre: GenreNiveau::Normal,
    intro: "La Pieuvre du port : ses tentacules frappent par colonnes annoncées, on lui saute dessus quand elle sort la tête.",
    mecanique: "Quitter la colonne annoncée par les remous ; sauter sur la tête sortie (jamais la toucher de côté), trois fois.",
    duree_cible_s: 45,
    ascii: ASCII,
    aiguilles: &[],
};

/// 20 x 11 tuiles (un seul écran), thème Port.
/// - Murs : colonnes 0 et 19. Le bassin : ligne 9, colonnes 1 à 18 (surface à
///   y = 144), sur le fond (ligne 10). La Pieuvre : `8` en (9, 9) ; ses 9 colonnes
///   de tentacules (2 cases chacune) couvrent tout le bassin, de la surface au haut
///   de la carte : aucun coin à l'abri.
/// - Ses têtes (48 x 32, posées sur l'eau) : au milieu (x de 136 à 184), à gauche
///   (40 à 88) et à droite (232 à 280). Les pontons (ligne 8) sont entre elles :
///   colonne 1 (le départ, la lanterne), colonnes 6-7 et 12-13 (avec une caisse
///   chacun, ligne 7), colonne 18. De chaque ponton, une tête est à un saut.
/// - Gouttes : au-dessus de l'eau à gauche (colonne 3), au-dessus de la tête du
///   milieu (colonne 10, ligne 6) et sur le ponton de droite (colonne 18) : on les
///   prend en planant (la barque rattrape une chute dans l'eau).
const ASCII: &str = r"
B..................B
B..................B
B..................B
B..................B
B..................B
B..................B
B@........o........B
BL.o...#....#.....oB
B#....##....##....#B
B~~~~~~~~8~~~~~~~~~B
BBBBBBBBBBBBBBBBBBBB
";
