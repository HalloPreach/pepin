//! Niveau d'essai « ennemis d'eau » (carte 9.4) : les cinq ennemis du Monde 2,
//! un par section, de gauche à droite. Hors de la table `NIVEAUX`, il n'existe
//! QUE dans les tests et dans le build de QA (fonctionnalité `debug_niveaux`,
//! touche J), comme les autres niveaux d'essai.
//!
//! Lecture de l'écran (64 x 11 tuiles, sol aux lignes 9 et 10) :
//! - le CRABE (`k`, colonne 8) patrouille dans une cuvette bordée de deux
//!   murets (colonnes 5 et 12). Lui sauter dessus fait rebondir sans le
//!   blesser ; le toucher de côté tue. On passe par-dessus, ou on rebondit ;
//! - la GRENOUILLE (`j`, colonne 19) guette sur la berge, avant le bassin. À
//!   portée (5 tuiles), elle se ramasse 0,5 s puis bondit vers Pépin : on
//!   l'écrase, ou on s'écarte ;
//! - le POISSON (`i`, colonne 23) se cache dans un bassin de trois tuiles
//!   (colonnes 22 à 24) ; les remous annoncent son saut vertical (3,5 tuiles) :
//!   on saute le bassin entre deux sauts ;
//! - le RAT (`r`, colonne 41) patrouille le fond d'un tunnel (colonnes 31 à
//!   43, une tuile plus bas que le sol, sous un bloc de briques) : les deux
//!   marches le retiennent dedans. Pépin descendu sur sa ligne, il se dresse
//!   0,5 s puis fonce. Le tunnel (3 tuiles de haut) laisse sauter par-dessus
//!   la course, ou écraser le rat ;
//! - la MOUETTE (`u`, colonne 50) est perchée au bout gauche du bras d'une grue
//!   (ligne 5, colonnes 50 à 54), tournée vers Pépin : elle pique quand il
//!   passe dessous. On passe sous le bras après son piqué ;
//! - trois gouttes (avant le crabe, après le bassin, au fond du tunnel), trois
//!   lanternes entre les sections, le drapeau au bout (colonne 60).

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 0,
    // Essai hors table : le monde ne sert qu'aux règles de la table.
    monde: 1,
    cle: "essai_ennemis_eau",
    sans_planer: false,
    nom: "Essai ennemis d'eau",
    theme: Theme::Riviere,
    genre: GenreNiveau::Normal,
    intro: "Les ennemis du Monde 2 : un crabe à la carapace dure, une grenouille qui bondit, un poisson qui saute, un rat qui fonce, une mouette qui pique.",
    mecanique: "Rebondir sur le crabe sans le toucher de côté ; écraser la grenouille et le rat ; sauter le bassin entre deux sauts du poisson ; passer sous la grue après le piqué de la mouette.",
    duree_cible_s: 60,
    ascii: ASCII,
    aiguilles: &[],
};

const ASCII: &str = r#"
...............................BBBBBBBBBBBBB....................
...............................BBBBBBBBBBBBB....................
...............................BBBBBBBBBBBBB....................
...............................BBBBBBBBBBBBB....................
...............................BBBBBBBBBBBBB......u.............
...............................BBBBBBBBBBBBB......BBBBB.........
...............................BBBBBBBBBBBBB....................
................................................................
.@.o.#..k...#L.....j...i...oL.................L.............X...
######################~~~######.....o....r..####################
################################################################
"#;
