//! Niveau d'essai « eau 2 » (carte 9.3) : les mécaniques d'eau du Monde 2, une
//! par étape, de gauche à droite. Hors de la table `NIVEAUX`, il n'existe QUE
//! dans les tests et dans le build de QA (fonctionnalité `debug_niveaux`,
//! touche F12), comme les autres niveaux d'essai.
//!
//! Lecture de l'écran (64 x 16 tuiles, thème de la grotte) :
//! - le couloir de la STALACTITE `V` (colonne 7, sous le plafond de briques) :
//!   elle tremble quand Pépin passe dessous, puis tombe ; on passe en courant ;
//! - le bassin des NÉNUPHARS `n` (colonnes 14 et 18) : trop large pour un saut
//!   planant, on y saute de feuille en feuille avant qu'elles ne mettent Pépin
//!   à l'eau ; au bout, la colonne de BULLES `Y` (colonne 21, prolongée jusqu'au
//!   fond du bassin) le soulève jusqu'à la plateforme haute (à sa gauche) et sa
//!   goutte, puis il plane jusqu'à la rive ;
//! - la fosse de la MARÉE `=` (colonnes 31 à 40, trois rangées) : sèche à basse
//!   mer, pleine à pleine mer. Le mur d'en face (colonnes 41 à 48) est trop haut
//!   depuis le fond, et trop loin depuis la rive ou le haut des bulles : on y
//!   monte en BARQUE, portée par la marée, puis on saute sur le mur (sa goutte) ;
//! - le puits de la CHUTE D'EAU `|` (colonnes 49 et 50) : elle pousse vers le
//!   bas, sur des pointes. Parapluie ouvert on y descend lentement et l'on
//!   glisse à droite, vers la 3e goutte ; parapluie fermé on tombe dessus ;
//! - deux lanternes : au départ et sur la rive de la marée.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 0,
    // Essai hors table : le monde ne sert qu'aux règles de la table.
    monde: 2,
    cle: "essai_eau2",
    sans_planer: false,
    nom: "Essai eau 2",
    theme: Theme::Grotte,
    genre: GenreNiveau::Normal,
    intro: "Les mécaniques d'eau : la stalactite qui tombe, les nénuphars qui s'enfoncent, les bulles qui soulèvent, la marée qui monte et descend, la chute d'eau qui pousse vers le bas.",
    mecanique: "Passer sous la stalactite sans s'arrêter ; sauter de feuille en feuille ; planer dans les bulles ; attendre la marée haute en barque ; planer dans la chute d'eau.",
    duree_cible_s: 60,
    ascii: ASCII,
    aiguilles: &[],
    poissons_hauts: &[],
};

const ASCII: &str = r#"
.................................................||.............
.................................................||.............
.................................................||.............
.................................................||.............
.................................................||.............
.................................................||.............
..................o..Y...........................||.............
................#####Y...........................||.............
.....BBBBB...........Y...........................||.............
.......V.............Y.....................o.....||.............
.....................Y...................########||.............
.@.L..........n...n..Y....L..............########||.............
###########~~~~~~~~~~~~~~######==========########||.........X...
###########~~~~~~~~~~~~~~######==========########||..###########
###############################==========########^^o.###########
################################################################
"#;
