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
    poissons_hauts: &[],
};

/// 20 x 11 tuiles (un seul écran), thème Port.
/// - Murs : colonnes 0 et 19. Le bassin : ligne 9, colonnes 1 à 18 (surface à
///   y = 144), sur le fond (ligne 10). La Pieuvre : `8` en (9, 9) ; ses 9 colonnes
///   de tentacules (2 cases chacune) couvrent tout le bassin, de la surface au haut
///   de la carte : aucun coin à l'abri.
/// - Ses têtes (48 x 32, posées sur l'eau) : au milieu (x de 136 à 184), à gauche
///   (40 à 88) et à droite (232 à 280).
/// - Carte 9.12c (arène moins noyante) : un PLATELAGE continu (ligne 6, sur des
///   piliers aux colonnes 1, 6-7, 12-13 et 18), percé de quatre trous d'une case
///   (colonnes 5, 8, 11 et 14) : chaque trou est au bord d'une tête, qui y dépasse
///   d'une demi-case. On esquive en courant, en sautant un trou d'une case ; on
///   tombe dans un trou pour atterrir sur la tête sortie. Sous le platelage,
///   l'eau (la barque, BAS tenu, rattrape une chute).
/// - Carte 9.12e (AGA : « enlever les petites plateformes au-dessus des endroits
///   où se positionnait la pieuvre ») : le petit ponton flottant des colonnes
///   9-10, au-dessus de la tête du milieu (tuiles 8,5 à 11,5), est retiré : le
///   trou du milieu fait 4 cases (colonnes 8 à 11). Les petits pontons sur piliers
///   (6-7, 12-13) ne surplombent aucune tête et restent ; les têtes de gauche et
///   de droite sont sous les grands pontons des bords, gardés.
/// - Gouttes : en colonne sous le départ (colonne 1, lignes 1 à 3) : Pépin les
///   prend en tombant au début, avant la colère. Sinon chaque goutte prise ou non
///   multiplie les cas que la recherche du robot et la mesure de difficulté
///   explorent (8 fois plus) : sur ce platelage, la recherche n'aboutissait pas.
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
