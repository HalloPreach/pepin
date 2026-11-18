//! Monde 2, niveau 2 : « Les égouts » (carte 9.7, allongés par la carte 9.21a).
//! La pluie du caniveau est tombée par la bouche d'égout : des tunnels de
//! briques, des rats qui foncent, et l'eau qui monte et descend dans les
//! bassins. Difficulté : un cran au-dessus du caniveau, celle des Gouttières ou
//! du Marché du Monde 1.
//!
//! Lecture du niveau (239 x 11 tuiles, un écran de haut, voûte de briques en
//! rangée 0), de gauche à droite :
//! 1. le COULOIR DU PREMIER RAT (colonnes 10 à 27) : un tunnel bas (3 tuiles),
//!    une marche plus bas que l'entrée ; le rat y patrouille. Pépin sur sa
//!    ligne, il se dresse puis fonce : on lui saute dessus ou par-dessus. Les
//!    marches des deux bouts le gardent dans son couloir ;
//! 2. le BASSIN de la goutte (colonnes 34 à 41, 3 tuiles de fond), qu'un gros
//!    tuyau remplit et qui se vide (la mécanique `=` de la carte 9.3) : on le
//!    saute sans peine, mais la 1re goutte attend au FOND, dans une niche du mur
//!    de droite (colonne 42, rangée 9). On y descend quand il est vide et on
//!    remonte avant que le tuyau ne crache (annoncé par le repère qui clignote et
//!    le tuyau qui gargouille) ;
//! 3. le BASSIN DU COLLECTEUR (colonnes 53 à 62), sous une voûte basse : pas
//!    de place pour planer par-dessus. On le traverse à pied quand il est vide
//!    (vite : l'eau qui monte noie), ou en barque quand il est plein ;
//! 4. le GRAND COLLECTEUR (colonnes 76 à 120, carte 9.21a) : une salle haute,
//!    deux bassins à marée en escalier (A, colonnes 80 à 93 ; B, colonnes 98 à
//!    111, une marche plus bas), chacun trop large pour être survolé d'un seul
//!    plané. Vides, on les traverse au fond et on remonte avant l'eau ; pleins,
//!    on s'y pose en barque. Entre les deux, un palier à lanterne où attendre
//!    la bonne marée ;
//! 5. le CANAL (colonnes 121 à 164) : un courant `}` qui pousse à droite. Une
//!    voûte descend jusqu'à une tuile de l'eau (colonnes 129 à 135) : impossible
//!    de la survoler, on y passe EN BARQUE, et la 2e goutte (colonne 132) attend
//!    dans ce seul espace libre. Plus loin (carte 9.21a), un poisson saute du
//!    canal (colonne 147) : on rame à contre-courant pour le laisser retomber,
//!    puis une seconde voûte basse (colonnes 153 à 162), toujours en barque. La
//!    bordure (colonnes 165 à 167) arrête la barque ;
//! 6. la GALERIE DES RATS (colonnes 169 à 192, carte 9.21a) : le tunnel bas
//!    reprend, une marche plus bas que la lanterne (un rat, colonne 175), puis
//!    une marche plus haut (un rat, colonne 186). Le plafond bas empêche de
//!    sauter haut : chaque rat se saute de près, ou s'écrase ;
//! 7. deux ÉTAGES au-dessus du couloir des rats (colonnes 197 à 220) : en
//!    haut, la GALERIE (rangées 1 à 3), en cul-de-sac, où un rat garde la 3e
//!    goutte (colonne 207) ; on y monte par la marche des colonnes 194 et 195.
//!    En bas, le COULOIR DES RATS : deux rats et une goutte du plafond
//!    (stalactite, colonne 211), qui tombe quand on passe dessous ;
//! 8. la sortie : le drapeau (colonne 233).
//!
//! Cinq lanternes (30, 66, 95, 168, 224), chacune sur un palier sans rat (les
//! marches arrêtent leur patrouille) et hors de l'eau qui monte. Aucune eau
//! suspendue au-dessus d'un passage à pied. Décor : deux souches d'aération
//! (`H`, au départ et à l'arrivée) ; les tuyaux et la grille du fond viennent du
//! thème.
//!
//! Habillage (carte 9.20b, `game/src/decor_egouts.rs`, rendu seulement) : on
//! arrive en tombant par la GRILLE de la bouche d'égout (au plafond au-dessus du
//! départ, le rai du lampadaire et la pluie qui passent) ; d'autres grilles au
//! plafond et leur filet de pluie ; les gros tuyaux des bassins, qui se vident
//! par à-coups quand l'eau monte ; la sortie au bout d'un GRAND TUYAU qui
//! débouche sur la nuit, en haut d'une falaise : la cascade est en face.

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
    mecanique: "Sauter par-dessus les rats qui foncent (ou les écraser) ; traverser les bassins à pied quand ils sont vides, ou en barque quand les tuyaux les ont remplis ; passer sous les voûtes du canal en barque.",
    duree_cible_s: 55,
    ascii: ASCII,
    aiguilles: &[],
};

/// Dessin du niveau (239 x 11 tuiles). Légende : voir `crate::level`.
const ASCII: &str = r"
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
..........BBBBBBBBBBBBBBBBBB.......................BBBBBBBBBBBBBB................................................................BBBBBBB.................BBBBBBBBBB......BBBBBBBBBBBBBBBBBBBBBBBB................BBBBBBBBBBBB.................B
..........BBBBBBBBBBBBBBBBBB.......................BBBBBBBBBBBBBB................................................................BBBBBBB.................BBBBBBBBBB......BBBBBBBBBBBBBBBBBBBBBBBB................BBBBBBBBBBBB.................B
..........BBBBBBBBBBBBBBBBBB.......................BBBBBBBBBBBBBB................................................................BBBBBBB.................BBBBBBBBBB......BBBBBBBBBBBBBBBBBBBBBBBB........r.....o.BBBBBBBBBBBB.................B
..........BBBBBBBBBBBBBBBBBB.......................BBBBBBBBBBBBBB................................................................BBBBBBB.................BBBBBBBBBB......BBBBBBBBBBBB................############BBBBBBBBBBBB.................B
.................................................................................................................................BBBBBBB.................BBBBBBBBBB................................................V..........................B
..@..H........................L...................................L.........####...............L....................................o..............i.................###L.................r.......##............................L....H...X....B
##########.........r........######========###########==========#############BBBB==============####.................######~~~~}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}BBB#......r.....#############BB#......r...........r.....##################
BBBBBBBBBB##################BBBBBB========BBBBBBBBBBB==========BBBBBBBBBBBBBBBBB==============BBBB==============###BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB############BBBBBBBBBBBBBBBB########################BBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB========oBBBBBBBBBB==========BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB==============BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";
