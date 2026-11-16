//! Niveau 1 : « Les toits ».
//!
//! Tutoriel en plein air : déplacer, sauter, puis planer pour franchir un grand vide.
//! Après le dernier panneau (planer), les deux premières flaques du jeu (carte
//! 6.4a) : on apprend ici à les écraser en leur sautant dessus. Chacune garde un
//! bout de toit fermé, sans lanterne ni décor sur sa patrouille :
//! - au fond du creux entre les deux cheminées de briques (colonnes 74 à 79) ;
//!   on lui tombe dessus, ou on saute d'une cheminée à l'autre en planant ;
//! - sur le petit toit du pied de l'immeuble (colonnes 116 à 120), juste avant
//!   le trou de deux tuiles : on lui tombe dessus depuis le grand toit, ou on
//!   plane par-dessus jusqu'à l'autre rive.
//!
//! Secret (carte 8.5) : la cheminée de gauche du creux (colonnes 72-73) est
//! creuse. Sa face gauche (colonne 72, rangée 7 : une brique décalée) est un faux
//! mur `m`, le toit dessous aussi : on y entre en marchant, on tombe dans une
//! salle sous le toit (rangée 9, colonnes 71 à 73) où attend une goutte d'or ; on
//! en ressort d'un saut. La goutte qui était au-dessus de l'immeuble y est
//! descendue. Le premier P de « PÉPIN » (carte 8.6, case `*`) l'y attend aussi, au
//! bout gauche de la salle (colonne 71).

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 1,
    cle: "toits",
    nom: "Les toits",
    theme: Theme::Toits,
    genre: GenreNiveau::Normal,
    intro:
        "La pluie tambourine sur les toits. Pépin ouvre son parapluie : en route vers le soleil !",
    mecanique: "Course, saut, planer (maintenir SAUT en l'air), premières flaques à écraser.",
    duree_cible_s: 50,
    ascii: ASCII,
    aiguilles: &[],
};

/// Dessin du niveau (176 x 11 tuiles). Légende : voir `crate::level`.
/// - Flaques : colonne 76 (creux 74-79, entre les cheminées 72-73 et 80) et
///   colonne 118 (toit 116-120). L'antenne qui était au fond du creux (colonne
///   77) est passée sur le toit d'arrivée du grand plané (colonne 68) : aucune
///   patrouille ne passe devant un objet de décor.
const ASCII: &str = r"
................................................................................................................................................................................
................................................................................................................................................................................
................................................................................................................................................................................
................................................................................................................................................................................
................................................................................................L...H.........T.................................................................
...............................................................o............................########################............................................................
.............o..........................................................BB......B.......####BBBBBBBBBBBBBBBBBBBBBBBB........................BB..................................
..@..D....S.....######.....S........T...L......H.......G............T...mB..f...B...####BBBBBBBBBBBBBBBBBBBBBBBBBBBB..f......H....L.........BB........T.........H.......X.......
################BBBBBB########...##########################.......######m###########BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB#####..#####################################################
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBBBBBBBBBB.......BBBBB*.oBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB..BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBBBBBBBBBB.......BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB..BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";
