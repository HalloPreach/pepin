//! Niveau 4 : « Le grand orage ».
//!
//! Éclairs temporisés, plateformes mobiles (qu'on attend), nuages friables, ascenseur.
//! Entre les deux plateformes mobiles, une cheminée d'une tuile (colonne 51)
//! sert de point de repos : on y attend la seconde plateforme.
//!
//! Un second corbeau (carte 6.4a), en plein ciel au-dessus des deux pointes du
//! deuxième toit (colonnes 33-34) : il vole deux rangées au-dessus du toit
//! (rangée 5, colonnes 31 à 41), on saute les pointes d'un bond court, ou on
//! attend qu'il soit passé. Pas de flaque de plus : chaque toit libre du niveau
//! porte une lanterne (une flaque y passerait sur Pépin réapparu), un objet de
//! décor (aucune patrouille devant le décor), des pointes, ou finit sur
//! l'ascenseur (il passe au ras du toit : une flaque marcherait dessus). Et
//! le juge de la courbe garde l'orage sous le clocher (voir QA.md, carte 6.4a).
//!
//! Secret (carte 8.5), un RACCOURCI : le grand bloc de l'ascenseur (colonnes 113
//! à 135) est percé à sa base d'un tunnel caché (rangée 7, faux murs `m` ;
//! l'indice est la brique décalée de l'entrée, à droite de l'ascenseur). Au lieu
//! de monter avec l'ascenseur puis de redescendre, on traverse le bloc et on
//! saute le trou de la sortie. Au milieu du tunnel, la goutte qui flottait
//! au-dessus des nuages friables, suivie du second P de « P-É-P-I-N »
//! (lettre cachée `*`, colonne 125, carte 8.6).
//!
//! Carte 8.15 (AGA : « impossible d'avoir la goutte sans mourir ») : l'ascenseur
//! descend une rangée PLUS BAS que le toit (rangée 9) ; il passe au niveau du
//! tunnel en descendant et continue sous Pépin, qui entre sans être repoussé.
//! L'entrée (colonne 113) et la sortie (colonne 135) du tunnel sont plus hautes
//! (faux murs au-dessus, invisibles de dehors) : en remontant, l'ascenseur laisse
//! le temps d'entrer, et on saute le trou de la sortie sans se cogner au plafond.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 4,
    cle: "orage",
    sans_planer: true,
    nom: "Le grand orage",
    theme: Theme::Orage,
    genre: GenreNiveau::Normal,
    intro: "L'orage éclate. Chaque éclair est annoncé : garde la tête froide !",
    mecanique: "Éclairs annoncés, plateformes mobiles et friables, ascenseur, corbeaux.",
    duree_cible_s: 60,
    ascii: ASCII,
    aiguilles: &[],
};

/// Dessin du niveau (200 x 11 tuiles). Légende : voir `crate::level`.
/// - Corbeaux : colonne 36 (rangée 5, au-dessus des pointes 33-34, carte 6.4a) et
///   colonne 130 (rangée 3, enfermé entre les murs 127 et 133). Flaque : 154.
const ASCII: &str = r"
..............e........................e..........................e.......e..............................e..............e.............................e.........e...........e...........................
...............................................................................................................................B.....B...........................................#......................
...............................................................................................................................B.....B..................................................................
.......................................................o............................................................L.......^^.B..c..B..................................................................
...............................................................................................................3.#######################................................................................
....................................c............................................................................BBBBBBBBBBBBBBBBBBBBBBm................................................................
.........o.......................................................................................................mBBBBBBBBBBBBBBBBBBBBBm................................................................
..@..R..............L........R...^^...........................L.......R.......^^^................................mmmmmmmmmmmo*mmmmmmmmmm.........L........f..........^^.........M.............X.........
#######################...##################1....1.#2.....2.#########################.FF.FF.FF.FF.F..##########..BBBBBBBBBBBBBBBBBBBBBBB...#############################################################
BBBBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBB.......B........BBBBBBBBBBBBBBBBBBBBBBBBB................BBBBBBBBBB3.BBBBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBB.......B........BBBBBBBBBBBBBBBBBBBBBBBBB................BBBBBBBBBB..BBBBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";
