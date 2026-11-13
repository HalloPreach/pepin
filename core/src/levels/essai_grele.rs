//! Niveau d'essai « grêle » : un écran et demi de toits qui met en scène la
//! grêle (carte 5.4 du plan de capture). Hors de la table `NIVEAUX`, il n'existe
//! QUE dans les tests et dans le build de QA (fonctionnalité `debug_niveaux`,
//! touche F2) ; en production il n'est pas compilé dans le binaire, comme les
//! deux autres niveaux d'essai.
//!
//! Trois rideaux de grêle, trois façons de faire :
//! - colonne 9, du ciel au toit : on saute tôt et on plane dessous ;
//! - colonnes 21 et 22, au-dessus d'un vide de 6 tuiles : la planée qui traverse
//!   le vide traverse aussi la grêle (parapluie ouvert d'un bout à l'autre) ;
//! - colonne 28, arrêtée par une corniche (ligne 5) : on passe dessous à pied,
//!   et la troisième goutte attend sous l'abri, juste sous la grêle.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 0,
    nom: "Essai grêle",
    theme: Theme::Altitude,
    genre: GenreNiveau::Normal,
    intro: "La grêle tombe sans arrêt. Parapluie fermé, elle est mortelle ; parapluie ouvert, on passe dessous.",
    mecanique: "Sauter tôt et planer sous la grêle ; un toit l'arrête, on passe dessous à pied.",
    duree_cible_s: 30,
    ascii: ASCII,
    aiguilles: &[],
};

/// 36 x 11 tuiles, thème Altitude (ciel corail : le plus clair des ciels où la
/// grêle tombera dans le jeu).
/// - Colonne 9 : grêle de la ligne 1 jusqu'au toit (ligne 8).
/// - Vide des colonnes 18 à 23, rideau de grêle double (colonnes 21 et 22) de la
///   ligne 2 jusqu'au bas de la carte ; une goutte au sommet de la planée.
/// - Corniche (colonnes 27 à 31, ligne 5) : la grêle de la colonne 28 s'y arrête.
/// - Lanternes aux colonnes 16, 26 et 30, toutes hors de la grêle.
const ASCII: &str = r"
....................................
.........g..................g.......
.....................gg.............
....................................
...................o................
...........................#####....
....................................
..@...........o.L.........L.o.L..X..
##################......############
##################......############
##################......############
";
