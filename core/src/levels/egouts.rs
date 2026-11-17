//! Monde 2, niveau 2 : « Les égouts » (carte 9.7). La pluie du caniveau est
//! tombée par la bouche d'égout : des tunnels de briques, des rats qui foncent,
//! et l'eau qui monte et descend dans les bassins. Difficulté : un cran au-dessus
//! du caniveau, celle des Gouttières ou du Marché du Monde 1.
//!
//! Lecture du niveau (146 x 11 tuiles, un écran de haut, voûte de briques en
//! rangée 0), de gauche à droite :
//! 1. le COULOIR DU PREMIER RAT (colonnes 10 à 27) : un tunnel bas (3 tuiles),
//!    une marche plus bas que l'entrée ; le rat y patrouille. Pépin sur sa
//!    ligne, il se dresse puis fonce : on lui saute dessus ou par-dessus. Les
//!    marches des deux bouts le gardent dans son couloir ;
//! 2. le BASSIN À MARÉE de la goutte (colonnes 34 à 41, 3 tuiles de fond) :
//!    on le saute sans peine, mais la 1re goutte attend au FOND, dans une niche
//!    du mur de droite (colonne 42, rangée 9). On y descend à marée basse et on
//!    remonte avant que l'eau ne revienne (annoncée par le repère qui clignote) ;
//! 3. le BASSIN DU COLLECTEUR (colonnes 53 à 62), sous une voûte basse : pas
//!    de place pour planer par-dessus. On le traverse à pied à marée basse (vite :
//!    l'eau qui monte noie), ou en barque à marée haute ;
//! 4. le CANAL (colonnes 76 à 95) : un courant `}` qui pousse à droite. Une
//!    voûte descend jusqu'à une tuile de l'eau (colonnes 84 à 90) : impossible
//!    de la survoler, on y passe EN BARQUE, et la 2e goutte (colonne 87) attend
//!    dans ce seul espace libre. La bordure (colonnes 96 à 98) arrête la barque ;
//! 5. deux ÉTAGES au-dessus du couloir des rats (colonnes 104 à 127) : en
//!    haut, la GALERIE (rangées 1 à 3), en cul-de-sac, où un rat garde la 3e
//!    goutte (colonne 114) ; on y monte par la marche des colonnes 101 et 102.
//!    En bas, le COULOIR DES RATS : deux rats et une goutte du plafond
//!    (stalactite, colonne 118), qui tombe quand on passe dessous ;
//! 6. la sortie : le drapeau (colonne 140).
//!
//! Quatre lanternes (30, 66, 99, 131), chacune sur un palier sans rat (les
//! marches arrêtent leur patrouille). Aucune eau suspendue au-dessus d'un
//! passage à pied. Décor : deux souches d'aération (`H`, au départ et à
//! l'arrivée) ; les tuyaux et la grille du fond viennent du thème.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    monde: 2,
    numero: 2,
    cle: "egouts",
    sans_planer: false,
    nom: "Les égouts",
    theme: Theme::Egouts,
    genre: GenreNiveau::Normal,
    intro: "Sous la rue, l'eau de pluie file dans les tunnels. Les rats aussi.",
    mecanique: "Sauter par-dessus les rats qui foncent (ou les écraser) ; traverser les bassins à marée basse à pied, ou à marée haute en barque ; passer sous la voûte du canal en barque.",
    duree_cible_s: 55,
    ascii: ASCII,
    aiguilles: &[],
};

/// Dessin du niveau (146 x 11 tuiles). Légende : voir `crate::level`.
const ASCII: &str = r"
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
..........BBBBBBBBBBBBBBBBBB.......................BBBBBBBBBBBBBB...................BBBBBBB.........................BBBBBBBBBBBB.................B
..........BBBBBBBBBBBBBBBBBB.......................BBBBBBBBBBBBBB...................BBBBBBB.........................BBBBBBBBBBBB.................B
..........BBBBBBBBBBBBBBBBBB.......................BBBBBBBBBBBBBB...................BBBBBBB.................r.....o.BBBBBBBBBBBB.................B
..........BBBBBBBBBBBBBBBBBB.......................BBBBBBBBBBBBBB...................BBBBBBB.............############BBBBBBBBBBBB.................B
....................................................................................BBBBBBB...........................V..........................B
..@..H........................L...................................L....................o........###L.##............................L....H...X....B
##########.........r........######========###########==========#############~~~~}}}}}}}}}}}}}}}}BBB##BB#......r...........r.....##################
BBBBBBBBBB##################BBBBBB========BBBBBBBBBBB==========BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB########################BBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB========oBBBBBBBBBB==========BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";
