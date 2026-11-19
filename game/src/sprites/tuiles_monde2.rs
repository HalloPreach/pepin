//! Tuiles du MONDE 2, par thème (carte 9.6b) : chaque thème du Monde 2 peut avoir
//! son propre jeu de tuiles 16x16 au lieu des toits et des briques du Monde 1. Le
//! choix du dessin, case par case, est dans `crate::tuiles_theme` ; ici, seulement
//! les dessins.
//!
//! Ces tuiles sont dessinées TELLES QUELLES (aucune substitution de couleurs de
//! thème, voir `remplacer`) : chaque jeu sert à un seul thème, ses couleurs sont
//! choisies pour lui. Règles de la palette : contour sombre, deux niveaux d'ombre
//! au plus, aucun dégradé ; le cyan (D, E) reste à l'eau et à la pluie, l'or (B, C)
//! aux gouttes.
//!
//! Carte 9.6c : les jeux des égouts, de la cascade, de la grotte et du port sont
//! dans les sous-modules du même nom (carte 9.10 : la rivière) ; `JEUX` les
//! rassemble tous.

use super::Image;

pub mod cascade;
pub mod egouts;
pub mod grotte;
pub mod port;
pub mod riviere;

/// Tous les jeux de tuiles du Monde 2, un par thème (le caniveau, puis carte 9.6c).
pub const JEUX: [&[(&str, Image)]; 6] = [
    SPRITES,
    egouts::SPRITES,
    cascade::SPRITES,
    grotte::SPRITES,
    port::SPRITES,
    riviere::SPRITES,
];

// --- Le caniveau ---------------------------------------------------------------
// La rue sous l'averse : des pavés gris-bleu en surface, la bordure du trottoir au
// bord de l'eau, une terre sombre dessous ; au-dessus de la rue, la maison de la
// gouttière (pierre, fenêtres, chéneau) et l'avant-toit (corniche et store).

/// Pavés de la rue (la surface où l'on marche) : contour sombre, arête claire, trois
/// rangées de pavés décalées (reflet en haut à gauche, ombre en bas à droite), joints
/// sombres, puis le lit de sable. Carte 9.20g : l'arête est le LISERÉ CLAIR (nuage)
/// de tout dessus où l'on se pose, comme la bordure et le mobilier de rue.
pub const PAVE: Image = &[
    "0000000000000000",
    "6666666666666666",
    "4443333144433331",
    "4333332143333321",
    "3333322133333221",
    "1111111111111111",
    "3331444333314443",
    "3321433333214333",
    "3221333332213333",
    "1111111111111111",
    "4443333144433331",
    "4333332143333321",
    "3333322133333221",
    "1111111111111111",
    "1210111112101111",
    "1111111111111111",
];

/// Pavés au bord GAUCHE de la rue (un creux ou de l'eau à gauche) : la bordure de
/// trottoir, une pierre claire en haut et une arête nette (le contour de la colonne 0).
pub const PAVE_G: Image = &[
    "0000000000000000",
    "0666666666666666",
    "0555513144433331",
    "0444412143333321",
    "0443412133333221",
    "0444411111111111",
    "0434414333314443",
    "0444413333214333",
    "0444313332213333",
    "0444411111111111",
    "0443413144433331",
    "0444412143333321",
    "0333312133333221",
    "0222211111111111",
    "0000001112101111",
    "0111111111111111",
];

/// Pavés au bord DROIT de la rue : la même bordure, à droite.
pub const PAVE_D: Image = &[
    "0000000000000000",
    "6666666666666660",
    "4443333144155550",
    "4333332143144430",
    "3333322133143430",
    "1111111111144430",
    "3331444333144340",
    "3321433333144430",
    "3221333332134430",
    "1111111111144430",
    "4443333144143430",
    "4333332143144430",
    "3333322133133330",
    "1111111111122220",
    "1210111112000000",
    "1111111111111110",
];

/// Sous la rue : une terre sombre et calme, presque sans détail (aucun cyan), pour que
/// l'eau et les plateformes ressortent.
pub const TERRE: Image = &[
    "1111111111111111",
    "1111111111111111",
    "1111121111111111",
    "1111100111111111",
    "1111111111111111",
    "1111111111112111",
    "1111111111110011",
    "1111111111111111",
    "1211111111111111",
    "1001111111111111",
    "1111111111111111",
    "1111111111211111",
    "1111111111001111",
    "1111111111111111",
    "1111111111111111",
    "1111111111111111",
];

/// Variante de la terre : un moellon enfoui (environ une tuile sur cinq).
pub const TERRE_PIERRE: Image = &[
    "1111111111111111",
    "1111111111111111",
    "1111121111111111",
    "1111100111111111",
    "1111111111111111",
    "1111111111111111",
    "1111111112221111",
    "1111111123322111",
    "1211111122222011",
    "1001111110000111",
    "1111111111111111",
    "1111111111111111",
    "1111111111111111",
    "1112111111111111",
    "1110011111111111",
    "1111111111111111",
];

/// Le fond du caniveau à sec : des dalles de granit, puis la terre.
pub const RIGOLE: Image = &[
    "0000000000000000",
    "4444444144444441",
    "3333333133333331",
    "3332333133323331",
    "2222222122222221",
    "0000000000000000",
    "1111111111111111",
    "1111121111111111",
    "1111100111111111",
    "1111111111111111",
    "1111111111112111",
    "1111111111110011",
    "1111111111111111",
    "1211111111111111",
    "1001111111111111",
    "1111111111111111",
];

/// Pierre de taille claire, seule (vide des deux côtés) : la bordure haute du
/// trottoir, un rebord de la gouttière. Un bloc cerné, éclairé par le haut.
pub const BORDURE: Image = &[
    "0000000000000000",
    "0666666666666660",
    "0555555555555550",
    "0444444444444430",
    "0444443444444430",
    "0444444444444430",
    "0434444444434430",
    "0444444444444430",
    "0444444434444430",
    "0444444444444430",
    "0444344444444430",
    "0444444444443430",
    "0444444444444430",
    "0333333333333330",
    "0222222222222220",
    "0000000000000000",
];

/// Pierre de taille au bout GAUCHE d'une rangée : contour à gauche, joint d'ardoise
/// à droite (marches, bordure haute, corniche de l'avant-toit).
pub const BORDURE_G: Image = &[
    "0000000000000000",
    "0666666666666662",
    "0555555555555552",
    "0444444444444432",
    "0444443444444432",
    "0444444444444432",
    "0434444444434432",
    "0444444444444432",
    "0444444434444432",
    "0444444444444432",
    "0444344444444432",
    "0444444444443432",
    "0444444444444432",
    "0333333333333332",
    "0222222222222222",
    "0000000000000000",
];

/// Pierre de taille au milieu d'une rangée : un joint d'ardoise à droite.
pub const BORDURE_M: Image = &[
    "0000000000000000",
    "6666666666666662",
    "5555555555555552",
    "4444444444444432",
    "4444443444444432",
    "4444444444444432",
    "4434444444434432",
    "4444444444444432",
    "4444444434444432",
    "4444444444444432",
    "4444344444444432",
    "4444444444443432",
    "4444444444444432",
    "3333333333333332",
    "2222222222222222",
    "0000000000000000",
];

/// Pierre de taille au bout DROIT d'une rangée : contour à droite.
pub const BORDURE_D: Image = &[
    "0000000000000000",
    "6666666666666660",
    "5555555555555550",
    "4444444444444430",
    "4444443444444430",
    "4444444444444430",
    "4434444444434430",
    "4444444444444430",
    "4444444434444430",
    "4444444444444430",
    "4444344444444430",
    "4444444444443430",
    "4444444444444430",
    "3333333333333330",
    "2222222222222220",
    "0000000000000000",
];

/// Soubassement d'une maison ou d'une bordure, au niveau de la rue : gros blocs de
/// granit, joints marqués.
pub const SOUBASSEMENT: Image = &[
    "4444444144444441",
    "3333333133333331",
    "3333233133333331",
    "3333333133332331",
    "3333333133333331",
    "3233333133333331",
    "2222222122222221",
    "1111111111111111",
    "4441444444414444",
    "3331333333313333",
    "3331333323313333",
    "3331332333313333",
    "3331333333313323",
    "3331333333313333",
    "2221222222212222",
    "1111111111111111",
];

/// Façade de la maison (au-dessus de la rue) : un enduit de pierre, deux assises.
pub const FACADE: Image = &[
    "3333333333333333",
    "3333333333333333",
    "3333332333333333",
    "3333333333333333",
    "3333333333333323",
    "3333333333333333",
    "3233333333333333",
    "2222222222222222",
    "3333333333333333",
    "3333333333233333",
    "3333333333333333",
    "3333233333333333",
    "3333333333333333",
    "3333333333333233",
    "3333333333333333",
    "2222222222222222",
];

/// Façade avec une fenêtre allumée : encadrement sombre, croisée, vitres orange, appui.
pub const FENETRE: Image = &[
    "3333333333333333",
    "3333444444443333",
    "3333000000003333",
    "333307A007A03333",
    "33330AA00AA03323",
    "33330AA00AA03333",
    "3233000000003333",
    "222207A007A02222",
    "33330AA00AA03333",
    "33330AA00AA03333",
    "33330AA00AA03333",
    "33330AA00AA03333",
    "3333000000003333",
    "3335555555555233",
    "3332222222222333",
    "2222222222222222",
];

/// Façade avec une fenêtre éteinte (vitres sombres, reflets d'ardoise).
pub const FENETRE_NOIRE: Image = &[
    "3333333333333333",
    "3333444444443333",
    "3333000000003333",
    "3333031003103333",
    "3333011001103323",
    "3333011001103333",
    "3233000000003333",
    "2222031003102222",
    "3333011001103333",
    "3333011001103333",
    "3333011001103333",
    "3333011001103333",
    "3333000000003333",
    "3335555555555233",
    "3332222222222333",
    "2222222222222222",
];

/// Façade avec la plaque de la rue : bleu ardoise cerclé de blanc cassé (le
/// « panneau » de la rue, sans lettre de grille : il ne se confond pas avec les
/// panneaux d'aide D, S, G, C, W).
pub const PLAQUE: Image = &[
    "3333333333333333",
    "3333333333333333",
    "3333332333333333",
    "3333333333333333",
    "3300000000000023",
    "3307777777777033",
    "3207222222227033",
    "2207277277727022",
    "3307222222227033",
    "3307777777777033",
    "3300000000000033",
    "3333233333333333",
    "3333333333333333",
    "3333333333333233",
    "3333333333333333",
    "2222222222222222",
];

/// Chéneau : sous l'eau de la gouttière, le fond de zinc qui la porte, puis la
/// corniche et ses denticules, puis la façade.
pub const CHENEAU: Image = &[
    "0000000000000000",
    "3333333333333333",
    "2222222222222222",
    "0000000000000000",
    "5555555555555555",
    "4444444444444444",
    "0440044004400440",
    "0000000000000000",
    "3333333333333333",
    "3333333333333333",
    "3333233333333333",
    "3333333333332333",
    "3333333333333333",
    "3333333333333333",
    "3333333233333333",
    "2222222222222222",
];

/// Store de boutique (le dessous de l'avant-toit, au-dessus du vide), bout GAUCHE :
/// toile rayée blanc cassé et orange, cernée de sombre, lambrequin festonné.
pub const STORE_G: Image = &[
    "0000000000000000",
    "0111111111111111",
    "0000000000000000",
    "0777AAAA7777AAAA",
    "0777AAAA7777AAAA",
    "0777AAAA7777AAAA",
    "0777AAAA7777AAAA",
    "0777AAAA7777AAAA",
    "0777AAAA7777AAAA",
    "0777AAAA7777AAAA",
    "0777AAAA7777AAAA",
    "0777AAAA7777AAAA",
    "0000000000000000",
    "07700AA007700AA0",
    ".00..00..00..00.",
    "................",
];

/// Store de boutique, milieu de la toile.
pub const STORE_M: Image = &[
    "0000000000000000",
    "1111111111111111",
    "0000000000000000",
    "7777AAAA7777AAAA",
    "7777AAAA7777AAAA",
    "7777AAAA7777AAAA",
    "7777AAAA7777AAAA",
    "7777AAAA7777AAAA",
    "7777AAAA7777AAAA",
    "7777AAAA7777AAAA",
    "7777AAAA7777AAAA",
    "7777AAAA7777AAAA",
    "0000000000000000",
    "07700AA007700AA0",
    ".00..00..00..00.",
    "................",
];

/// Store de boutique, bout DROIT.
pub const STORE_D: Image = &[
    "0000000000000000",
    "1111111111111110",
    "0000000000000000",
    "7777AAAA7777AAA0",
    "7777AAAA7777AAA0",
    "7777AAAA7777AAA0",
    "7777AAAA7777AAA0",
    "7777AAAA7777AAA0",
    "7777AAAA7777AAA0",
    "7777AAAA7777AAA0",
    "7777AAAA7777AAA0",
    "7777AAAA7777AAA0",
    "0000000000000000",
    "07700AA007700AA0",
    ".00..00..00..00.",
    "................",
];

// --- La bouche d'égout (carte 9.20a) ---------------------------------------------
// La sortie du caniveau : la bordure du trottoir qui arrête le dernier courant (deux
// tuiles de fond) est ouverte. Trois cases l'une sous l'autre : la bordure et sa
// grille (le drapeau y est planté), la gueule où l'eau de surface s'engouffre, la
// gorge où l'eau du fond tombe dans le noir. L'eau garde ses couleurs (bande cyan,
// fond ardoise) jusque dans la gueule.

/// La bordure au-dessus de la bouche : sur le dessus, la grille (fentes sombres) ;
/// dessous, le linteau puis le haut de la gueule, noir.
pub const EGOUT_GRILLE: Image = &[
    "0000000000000000",
    "0666666666666662",
    "0606060606060602",
    "0505050505050502",
    "0000000000000002",
    "0444444444444432",
    "0444443444444432",
    "0434444444434432",
    "0444444444444432",
    "0333333333333332",
    "0222222222222222",
    "0000000000000000",
    "0000000000000444",
    "0000000000000333",
    "0000000000000333",
    "0000000000000323",
];

/// La gueule de la bouche, à la surface de l'eau : la bande d'eau entre par la
/// gauche et plonge dans le noir ; à droite, le granit de la bordure.
pub const EGOUT_BOUCHE: Image = &[
    "DDDD770000000333",
    "EEEEED7000000333",
    "EEEEEED700000222",
    "EEEEEEED00000111",
    "EEEEEEEED0000444",
    "EEEEEEEED0000333",
    "EEEEEEEEED000323",
    "22222EEEED000333",
    "222222EEED000333",
    "2222222EED000333",
    "22222222ED000222",
    "222222222E000111",
    "222222222E000444",
    "2222222222E00333",
    "2222222222E00333",
    "2222222222E00323",
];

/// La gorge, sous la gueule : l'eau du fond se dérobe en biais et tombe dans le noir.
pub const EGOUT_GORGE: Image = &[
    "2222222222E00333",
    "222E222222E00333",
    "222222222E000222",
    "22222222E0000111",
    "2222222E00000444",
    "222222E000000333",
    "22222E0000000323",
    "2222E00000000333",
    "222E000000000333",
    "22E0000000000333",
    "2E00000000000222",
    "E000000000000111",
    "0000000000000444",
    "0000000000000333",
    "0000000000000222",
    "0000000000000111",
];

/// Tous les sprites de ce fichier : (nom, dessin). Préfixe du nom = le thème
/// (`can_` : le caniveau).
pub const SPRITES: &[(&str, Image)] = &[
    ("can_pave", PAVE),
    ("can_pave_g", PAVE_G),
    ("can_pave_d", PAVE_D),
    ("can_terre", TERRE),
    ("can_terre_pierre", TERRE_PIERRE),
    ("can_rigole", RIGOLE),
    ("can_bordure", BORDURE),
    ("can_bordure_g", BORDURE_G),
    ("can_bordure_m", BORDURE_M),
    ("can_bordure_d", BORDURE_D),
    ("can_soubassement", SOUBASSEMENT),
    ("can_facade", FACADE),
    ("can_fenetre", FENETRE),
    ("can_fenetre_noire", FENETRE_NOIRE),
    ("can_plaque", PLAQUE),
    ("can_cheneau", CHENEAU),
    ("can_store_g", STORE_G),
    ("can_store_m", STORE_M),
    ("can_store_d", STORE_D),
    ("can_egout_grille", EGOUT_GRILLE),
    ("can_egout_bouche", EGOUT_BOUCHE),
    ("can_egout_gorge", EGOUT_GORGE),
];
