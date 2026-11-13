//! Niveau d'essai « auvent » : deux écrans de boutiques qui mettent en scène
//! les auvents (carte 5.5 du plan de capture). Hors de la table `NIVEAUX`, il
//! n'existe QUE dans les tests et dans le build de QA (fonctionnalité
//! `debug_niveaux`, touche F3) ; en production il n'est pas compilé dans le
//! binaire, comme les autres niveaux d'essai.
//!
//! Le « moment fort » du futur marché couvert, en petit : six auvents à la suite
//! au-dessus d'un vide de 23 tuiles, sans toucher le sol. Ils sont espacés de 4,
//! 4, 5, 3 et 4 tuiles : sans rien toucher, un rebond retombe 4 tuiles plus loin ;
//! il faut garder la droite pour le 5 et retenir (gauche) pour le 3. Le toit
//! d'arrivée commence 3 tuiles après le dernier auvent. (Un joueur pressé peut
//! aussi en sauter : en gardant la droite, un rebond porte à 7,5 tuiles.)

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 0,
    nom: "Essai auvent",
    theme: Theme::Gouttieres,
    genre: GenreNiveau::Normal,
    intro: "Un auvent relance Pépin en diagonale, vers le haut et vers l'avant : quatre tuiles plus loin sans rien toucher.",
    mecanique: "Enchaîner six auvents au-dessus du vide : garder la droite pour aller plus loin, la gauche pour retomber plus près.",
    duree_cible_s: 30,
    ascii: ASCII,
    aiguilles: &[],
};

/// 44 x 11 tuiles, thème Gouttières (nuit bleue, comme le futur marché couvert).
/// - Toit de départ : colonnes 0 à 8 ; toit d'arrivée : colonnes 32 à 43 (dessus
///   en ligne 7, Pépin court sur la ligne 6).
/// - Auvents (ligne 7, toile 4 px sous le dessus des toits) aux colonnes 9, 13,
///   17, 22, 25 et 29 : on se laisse tomber du toit sur le premier.
/// - Gouttes : sur le toit de départ, en l'air entre les auvents 17 et 22 (ligne
///   4, sur la trajectoire d'un rebond), sur le toit d'arrivée.
/// - Lanternes aux colonnes 6 (toit de départ), 34 et 39 (toit d'arrivée).
const ASCII: &str = r"
............................................
............................................
............................................
............................................
...................o........................
............................................
..@o..L...........................L..o.L.X..
#########A...A...A....A..A...A..############
#########.......................############
#########.......................############
#########.......................############
";
