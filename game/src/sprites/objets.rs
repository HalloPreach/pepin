//! Sprites des objets : gouttes d'or, lanternes, champignons, pointes, plateformes,
//! panneaux du tutoriel et drapeau d'arrivée.

use super::Image;

/// Goutte d'or 10x10 : la goutte de 8x8 (boîte de ramassage) cernée de 2 px de contour sombre
/// (le contour d'origine + un anneau), pour rester lisible devant tous les ciels, même dorés.
pub const GOUTTE0: Image = &[
    "....00....",
    "...0000...",
    "..00CB00..",
    "..00BB00..",
    ".00BCBB00.",
    "00BBCBBA00",
    "00BBBBAA00",
    ".00BAAA00.",
    "..000000..",
    "...0000...",
];

pub const GOUTTE1: Image = &[
    "....00....",
    "...0000...",
    "..00BB00..",
    "..00BB00..",
    ".00BBCB00.",
    "00BBBCBA00",
    "00BBBBAA00",
    ".00BAAA00.",
    "..000000..",
    "...0000...",
];

pub const GOUTTE2: Image = &[
    "....00....",
    "...0000.C.",
    "..00BB00..",
    "..00BB00..",
    ".00BBBB00.",
    "00BCBBBA00",
    "00BCBBAA00",
    ".00BAAA00.",
    "..000000..",
    "...0000...",
];

pub const GOUTTE3: Image = &[
    "....00....",
    ".C.0000...",
    "..00BB00..",
    "..00BB00..",
    ".00BBBB00.",
    "00BBBBBA00",
    "00BBBCAA00",
    ".00BACA00.",
    "..000000..",
    "...0000...",
];

/// Lanterne éteinte : vitre claire (ARDOISE_PALE/BRUME) pour se lire devant les immeubles du fond.
pub const LANTERNE0: Image = &[
    "...00...",
    "..0330..",
    ".033330.",
    ".045540.",
    ".045540.",
    ".045540.",
    ".033330.",
    "..0330..",
    "...00...",
    "..0220..",
    "..0220..",
    "..0220..",
    "..0220..",
    "..0220..",
    "..0220..",
    ".000000.",
];

pub const LANTERNE1: Image = &[
    "...00...",
    "..0330..",
    ".033330.",
    ".0BCCB0.",
    ".0BCCB0.",
    ".0BCCB0.",
    ".033330.",
    "..0330..",
    "...00...",
    "..0220..",
    "..0220..",
    "..0220..",
    "..0220..",
    "..0220..",
    "..0220..",
    ".000000.",
];

pub const CHAMPIGNON0: Image = &[
    "................",
    "................",
    "................",
    "....00000000....",
    "..0088889988800.",
    ".088988888888880",
    ".088888898888880",
    ".00000000000000.",
    "...00755570000..",
    ".....075570.....",
    ".....075570.....",
    "....00755700....",
];

pub const CHAMPIGNON1: Image = &[
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "....00000000....",
    "..0088889988800.",
    ".088988888888880",
    ".088888898888880",
    ".00000755570000.",
    "....00755700....",
];

pub const POINTE: Image = &[
    "...00...",
    "..0760..",
    "..0760..",
    ".076540.",
    ".076540.",
    "07765540",
    "07665540",
    "00000000",
];

pub const FRIABLE0: Image = &[
    "................",
    "..000..0000.....",
    ".04770076670....",
    "0477666666660...",
    "0566666666655000",
    ".055555555555550",
    "..05555555550...",
    "...0000000000...",
];

pub const FRIABLE1: Image = &[
    "................",
    "..000..0000.....",
    ".04770076670....",
    "0477060666660...",
    "0566606066655000",
    ".055050555505550",
    "..05500555000...",
    "...0000000000...",
];

/// Plateforme friable des niveaux 1 à 3 : dalle d'ardoise claire à angles écornés, avec une fine fêlure.
pub const FRIABLE_ARDOISE0: Image = &[
    ".00000000000000.",
    "0777777777777770",
    "0666666066666660",
    "0666660066666650",
    "0555550555555550",
    "0555555055555450",
    "0444444444444440",
    ".00000000000000.",
];

/// Dalle d'ardoise qui s'effrite : la fêlure s'ouvre de part en part.
pub const FRIABLE_ARDOISE1: Image = &[
    ".00000000000000.",
    "0777777007777770",
    "0666660000666660",
    "0666600..0066650",
    "0555500..0055550",
    "055500....005550",
    "0440000..0000440",
    ".00000....00000.",
];

/// Plateforme mobile 32x8 : le dessin couvre toute la boîte de collision (8 px de haut).
pub const MOBILE: Image = &[
    "00000000000000000000000000000000",
    "04444444444444444444444444444440",
    "03333333333333333333333333333330",
    "032222A222222222222222A222222230",
    "03222222222222222222222222222230",
    "02222222222222222222222222222220",
    "01111111111111111111111111111110",
    "00000000000000000000000000000000",
];

/// Aiguille d'horloge 32x8 (carte 5.10) : une plaque de laiton qui tourne autour
/// de son cadran. Même taille que la plateforme mobile, le dessin couvre toute la
/// boîte de collision ; dessus or pâle (le bord où l'on se tient), ombre orange
/// dessous, trois rivets (celui du milieu tient la tige des maillons).
pub const AIGUILLE: Image = &[
    "00000000000000000000000000000000",
    "0CCCCCCCCCCCCCCCCCCCCCCCCCCCCCC0",
    "0BBBBBBBBBBBBBBBBBBBBBBBBBBBBBB0",
    "0BB00BBBBBBBBBB00BBBBBBBBBB00BB0",
    "0BB00BBBBBBBBBB00BBBBBBBBBB00BB0",
    "0BBBBBBBBBBBBBBBBBBBBBBBBBBBBBB0",
    "0AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA0",
    "00000000000000000000000000000000",
];

/// Cadran d'horloge 32x32, centré sur le pivot `h` d'une aiguille : contour sombre,
/// bord doré (ombré orange en bas à droite), filet sombre, fond blanc cassé,
/// douze graduations (12, 3, 6 et 9 plus grosses) et le moyeu doré du pivot.
/// Dessin seulement : aucune collision.
pub const CADRAN: Image = &[
    "..............0000..............",
    "..........0000BBBB0000..........",
    "........00BBBBBBBBBBBB00........",
    ".......0BBBBBB0000BBBBBB0.......",
    ".....00BBBB0007777000BBBB00.....",
    "....00BBB00777700777700BBB00....",
    "....0BBB0770777007770770BBB0....",
    "...0BBB077777777777777770BBB0...",
    "..0BBB07777777777777777770BBA0..",
    "..0BB0777777777777777777770AA0..",
    ".0BBB0707777777777777777070AAA0.",
    ".0BB077777777777777777777770AA0.",
    ".0BB077777777777777777777770AA0.",
    ".0BB077777777777777777777770AA0.",
    "0BB07777777777700777777777770AA0",
    "0BB070077777770BB077777770070AA0",
    "0BB070077777770BB077777770070AA0",
    "0BB07777777777700777777777770AA0",
    ".0BB077777777777777777777770AA0.",
    ".0BB077777777777777777777770AA0.",
    ".0BB077777777777777777777770AA0.",
    ".0BBB0707777777777777777070AAA0.",
    "..0BB0777777777777777777770AA0..",
    "..0BBB07777777777777777770AAA0..",
    "...0BBB077777777777777770AAA0...",
    "....0BBB0770777007770770AAA0....",
    "....00BBB00777700777700AAA00....",
    ".....00BBAA0007777000AAAA00.....",
    ".......0AAAAAA0000AAAAAA0.......",
    "........00AAAAAAAAAAAA00........",
    "..........0000AAAA0000..........",
    "..............0000..............",
];

/// Maillon 4x4 de la tige qui relie le moyeu du cadran à l'aiguille : posés en
/// ligne du pivot jusqu'au milieu de la plaque, ils tournent avec elle comme une
/// aiguille de montre (chaque maillon est posé à une position entière, jamais
/// tourné ni agrandi).
pub const MAILLON: Image = &[
    ".00.",
    "0BB0",
    "0BA0",
    ".00.",
];

pub const PANNEAU_DEPLACER: Image = &[
    "..000000000000..",
    "..044444444440..",
    "..032222222230..",
    "..032722227230..",
    "..037777777730..",
    "..032722227230..",
    "..032222222230..",
    "..032222222230..",
    "..032222222230..",
    "..033333333330..",
    "..000000000000..",
    ".......33.......",
    ".......33.......",
    ".......33.......",
    ".......33.......",
    "......0000......",
];

pub const PANNEAU_SAUTER: Image = &[
    "..000000000000..",
    "..044444444440..",
    "..032227722230..",
    "..032277772230..",
    "..032727727230..",
    "..032227722230..",
    "..032227722230..",
    "..032288882230..",
    "..032222222230..",
    "..033333333330..",
    "..000000000000..",
    ".......33.......",
    ".......33.......",
    ".......33.......",
    ".......33.......",
    "......0000......",
];

pub const PANNEAU_PLANER: Image = &[
    "..000000000000..",
    "..044444444440..",
    "..032288882230..",
    "..032889988230..",
    "..038888888830..",
    "..032227722230..",
    "..032227722230..",
    "..032272272230..",
    "..032222222230..",
    "..033333333330..",
    "..000000000000..",
    ".......33.......",
    ".......33.......",
    ".......33.......",
    ".......33.......",
    "......0000......",
];

pub const PANNEAU_CHAMPIGNON: Image = &[
    "..000000000000..",
    "..044444444440..",
    "..032227722230..",
    "..032277772230..",
    "..032227722230..",
    "..032888888230..",
    "..038899888830..",
    "..032275572230..",
    "..032222222230..",
    "..033333333330..",
    "..000000000000..",
    ".......33.......",
    ".......33.......",
    ".......33.......",
    ".......33.......",
    "......0000......",
];

pub const PANNEAU_VENT: Image = &[
    "..000000000000..",
    "..044444444440..",
    "..032D2222D230..",
    "..032D2DD2D230..",
    "..032D2D22D230..",
    "..032D2DD2D230..",
    "..032D22D2D230..",
    "..032D2DD2D230..",
    "..032227722230..",
    "..033333333330..",
    "..000000000000..",
    ".......33.......",
    ".......33.......",
    ".......33.......",
    ".......33.......",
    "......0000......",
];

pub const DRAPEAU0: Image = &[
    ".0BB0...........",
    ".0BB0...........",
    ".00000000000....",
    "..54CBBBBBB0....",
    "..54CBBBBBBB0...",
    "..54CBBBBBBBB0..",
    "..54CBBBBBBBBB0.",
    "..54CBBBBBBBBBB0",
    "..54CBBBBBBBBB0.",
    "..54CBBBBBBBB0..",
    "..54CBBBBBBB0...",
    "..54CBBBBBB0....",
    "..54CBBBBB0.....",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "03333330........",
    "03333330........",
    "00000000........",
];

pub const DRAPEAU1: Image = &[
    ".0BB0...........",
    ".0BB0...........",
    ".0000000000.....",
    "..54CBBBBB0.....",
    "..54CBBBBBBBB0..",
    "..54CBBBBBBB0...",
    "..54CBBBBBBBBBB0",
    "..54CBBBBBBBBB0.",
    "..54CBBBBBBBBBB0",
    "..54CBBBBBBB0...",
    "..54CBBBBBBBB0..",
    "..54CBBBBB0.....",
    "..54CBBBBBB0....",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "03333330........",
    "03333330........",
    "00000000........",
];

/// Grêlon 4x4 : une bille de glace de 2x2 px (reflet blanc cassé, nuage, ombre de
/// brume) cernée d'un contour sombre. Sur les ciels sombres la glace claire se
/// détache, sur les ciels dorés c'est le contour. Quatre images : le reflet fait le
/// tour de la bille (elle roule en tombant).
pub const GRELON0: Image = &[
    ".00.",
    "0760",
    "0650",
    ".00.",
];

pub const GRELON1: Image = &[
    ".00.",
    "0670",
    "0560",
    ".00.",
];

pub const GRELON2: Image = &[
    ".00.",
    "0560",
    "0670",
    ".00.",
];

pub const GRELON3: Image = &[
    ".00.",
    "0650",
    "0760",
    ".00.",
];

/// Éclat d'un grêlon qui se brise sur ce qui l'arrête (toit, parapluie ouvert) :
/// deux miettes de glace d'un pixel qui s'écartent, chacune cernée de sombre.
pub const GRELE_ECLAT: Image = &[
    ".0...0.",
    "070.070",
    ".0...0.",
];

/// Auvent de boutique (lettre `A`, 16x15), vu de profil : un mât de métal sombre
/// (« 030 »), la toile en pente haute à gauche (rayée corail / blanc cassé /
/// corail), le lambrequin au bout (rayures verticales, bas festonné) et une jambe
/// de force. La pente dit d'où part le rebond : vers le haut et vers la droite.
pub const AUVENT0: Image = &[
    "000.............",
    "03000...........",
    "0308800.........",
    "030778800.......",
    "03088778800.....",
    "0300088778800000",
    "030..00887787870",
    "030....008887870",
    "030......0087870",
    "030.......087870",
    "030....00.080800",
    "030..00....0.0..",
    "03000...........",
    "030.............",
    "000.............",
];

/// Auvent enfoncé : la toile plie au milieu sous le poids de Pépin et le
/// lambrequin descend d'un pixel (image du rebond).
pub const AUVENT1: Image = &[
    "000.............",
    "03000...........",
    "0308800.........",
    "03077880........",
    "030887700.......",
    "03000888800.....",
    "030..00778800000",
    "030...0887787870",
    "030....008887870",
    "030......0087870",
    "030....00.087870",
    "030..00...080800",
    "03000......0.0..",
    "030.............",
    "000.............",
];

/// Façade de boutique (16x20) : la vitrine allumée derrière le mât d'un auvent
/// (carte 5.12). Dessin SEUL, aucune collision : posée dans la case à gauche de
/// l'auvent, 4 px plus haut que la case (la corniche arrive au sommet du mât),
/// elle accroche la toile à une boutique. Corniche claire, vitrine orange (deux
/// carreaux, des étalages en silhouette), soubassement d'ardoise. Ni or (gouttes,
/// éclairs), ni corail (parapluie, toile de l'auvent), ni cyan (flaques, pluie).
pub const FACADE_VITRINE: Image = &[
    "0000000000000000",
    "0333333333333330",
    "0222222222222220",
    "0000000000000000",
    "0111111111111110",
    "0100000000000010",
    "010AAAA00AAAA010",
    "010AAAA00AAAA010",
    "010AAAA00AAAA010",
    "010A11A00A11A010",
    "0101111001111010",
    "0100000000000010",
    "0111111111111110",
    "0110000000000110",
    "0110222222220110",
    "0110222222220110",
    "0110222222220110",
    "0110222222220110",
    "0110222222220110",
    "0110222222220110",
];

/// Mur de la façade de boutique (16x16), répété sous la vitrine jusqu'au premier
/// solide (ou au bas du niveau) : bleu nuit, fenêtres éteintes. Plus sombre que
/// les tuiles, sans briques : on voit que c'est un fond, pas un mur où se poser.
pub const FACADE_MUR: Image = &[
    "0111111111111110",
    "0111111111111110",
    "0110000110000110",
    "0110220110220110",
    "0110220110220110",
    "0110220110220110",
    "0110000110000110",
    "0111111111111110",
    "0111111111111110",
    "0111111111111110",
    "0110000110000110",
    "0110220110220110",
    "0110220110220110",
    "0110220110220110",
    "0110000110000110",
    "0111111111111110",
];

/// Ventilateur : un ventilateur à pied sur sa casette (socle 16x8, en bas de la
/// case). Trois états : à l'arrêt (pale dorée, rien au-dessus), à l'annonce
/// (la lampe orange clignote au-dessus de la cage, comme l'annonce d'un éclair)
/// et en souffle (le courant d'air cyan remonte, les pales tournent). Le
/// courant ne porte qu'en phase de souffle : la boîte SOLIDE est la casette.
pub const VENTILATEUR_ARRET: Image = &[
    "................",
    "................",
    "....00000000....",
    "...0333333330...",
    "...03BB00BB30...",
    "...03BB77BB30...",
    "...03BB00BB30...",
    "...0333333330...",
    "....04444440....",
    "......0440......",
    "......0440......",
    "......0440......",
    ".00000000000000.",
    ".04444444444440.",
    ".04444444444440.",
    "0000000000000000",
];

/// Ventilateur à l'annonce : la lampe orange clignote (c'est le signal, comme
/// pour un éclair : « le souffle va reprendre »).
pub const VENTILATEUR_AVEUGLE: Image = &[
    "......0AA0......",
    "......0AA0......",
    "....00000000....",
    "...0333333330...",
    "...03BB00BB30...",
    "...03BB77BB30...",
    "...03BB00BB30...",
    "...0333333330...",
    "....04444440....",
    "......0440......",
    "......0440......",
    "......0440......",
    ".00000000000000.",
    ".04444444444440.",
    ".04444444444440.",
    "0000000000000000",
];

/// Ventilateur en souffle : le courant d'air (cyan) remonte au-dessus de la
/// cage, les pales sont en mouvement (image du courant actif).
pub const VENTILATEUR_SOUFFLE: Image = &[
    "....D..D..D.....",
    "...D...D...D....",
    "....D..D..D.....",
    "...D...D...D....",
    "....00000000....",
    "...0333333330...",
    "...03BB77BB30...",
    "...0333333330...",
    "....04444440....",
    "......0440......",
    "......0440......",
    "......0440......",
    ".00000000000000.",
    ".04444444444440.",
    ".04444444444440.",
    "0000000000000000",
];

/// Montgolfière (20 x 40) : l'enveloppe à fuseaux cyan et blancs (le fuseau de
/// droite dans l'ombre), les cordes, puis la NACELLE (16 x 8, rangées 32 à 39 : sa
/// boîte solide ; le dessus est la rangée 32). Brûleur éteint : au repos, en
/// redescente, ou arrêtée en haut. On la dessine 2 px à gauche de la nacelle et
/// 32 px au-dessus (le ballon déborde de 2 px de chaque côté).
pub const MONTGOLFIERE_REPOS: Image = &[
    "......00000000......",
    "....00D77DD77E00....",
    "..00DD777DD777EE00..",
    ".0DDD777DDDD777EEE0.",
    "0DDD777DDDDDD777EEE0",
    "0DDD777DDDDDD777EEE0",
    "0DDD777DDDDDD777EEE0",
    "0DDD777DDDDDD777EEE0",
    "0DDD777DDDDDD777EEE0",
    "0DDD777DDDDDD777EEE0",
    ".0DDD777DDDD777EEE0.",
    "..0DD777DDDD777EE0..",
    "...0DD77DDDD77EE0...",
    "....0D77DDDD77E0....",
    ".....0D7DDDD7E0.....",
    "......0D7DD7E0......",
    ".......000000.......",
    ".......4.00.4.......",
    ".......4....4.......",
    "......4......4......",
    "......4......4......",
    "......4......4......",
    ".....4........4.....",
    ".....4........4.....",
    ".....4........4.....",
    "....4..........4....",
    "....4..........4....",
    "...4............4...",
    "...4............4...",
    "...4............4...",
    "..4..............4..",
    "..4..............4..",
    "..0000000000000000..",
    "..0777777777777770..",
    "..0AABAABAABAABAA0..",
    "..0AABAABAABAABAA0..",
    "..0AABAABAABAABAA0..",
    "..0AABAABAABAABAA0..",
    "..0AABAABAABAABAA0..",
    "...00000000000000...",
];

/// Montgolfière qui MONTE : la flamme du brûleur éclaire la jupe de l'enveloppe
/// (image 1 sur 2 ; la flamme vacille d'une image à l'autre).
pub const MONTGOLFIERE_FEU0: Image = &[
    "......00000000......",
    "....00D77DD77E00....",
    "..00DD777DD777EE00..",
    ".0DDD777DDDD777EEE0.",
    "0DDD777DDDDDD777EEE0",
    "0DDD777DDDDDD777EEE0",
    "0DDD777DDDDDD777EEE0",
    "0DDD777DDDDDD777EEE0",
    "0DDD777DDDDDD777EEE0",
    "0DDD777DDDDDD777EEE0",
    ".0DDD777DDDD777EEE0.",
    "..0DD777DDDD777EE0..",
    "...0DD77DDDD77EE0...",
    "....0D77DDDD77E0....",
    ".....0D7DADD7E0.....",
    "......0D7BA7E0......",
    ".......0BCCB0.......",
    ".......4.00.4.......",
    ".......4....4.......",
    "......4......4......",
    "......4......4......",
    "......4......4......",
    ".....4........4.....",
    ".....4........4.....",
    ".....4........4.....",
    "....4..........4....",
    "....4..........4....",
    "...4............4...",
    "...4............4...",
    "...4............4...",
    "..4..............4..",
    "..4..............4..",
    "..0000000000000000..",
    "..0777777777777770..",
    "..0AABAABAABAABAA0..",
    "..0AABAABAABAABAA0..",
    "..0AABAABAABAABAA0..",
    "..0AABAABAABAABAA0..",
    "..0AABAABAABAABAA0..",
    "...00000000000000...",
];

/// Montgolfière qui monte : image 2 sur 2 de la flamme.
pub const MONTGOLFIERE_FEU1: Image = &[
    "......00000000......",
    "....00D77DD77E00....",
    "..00DD777DD777EE00..",
    ".0DDD777DDDD777EEE0.",
    "0DDD777DDDDDD777EEE0",
    "0DDD777DDDDDD777EEE0",
    "0DDD777DDDDDD777EEE0",
    "0DDD777DDDDDD777EEE0",
    "0DDD777DDDDDD777EEE0",
    "0DDD777DDDDDD777EEE0",
    ".0DDD777DDDD777EEE0.",
    "..0DD777DDDD777EE0..",
    "...0DD77DDDD77EE0...",
    "....0D77DDDD77E0....",
    ".....0D7DDAD7E0.....",
    "......0D7AB7E0......",
    ".......0CBBC0.......",
    ".......4.00.4.......",
    ".......4....4.......",
    "......4......4......",
    "......4......4......",
    "......4......4......",
    ".....4........4.....",
    ".....4........4.....",
    ".....4........4.....",
    "....4..........4....",
    "....4..........4....",
    "...4............4...",
    "...4............4...",
    "...4............4...",
    "..4..............4..",
    "..4..............4..",
    "..0000000000000000..",
    "..0777777777777770..",
    "..0AABAABAABAABAA0..",
    "..0AABAABAABAABAA0..",
    "..0AABAABAABAABAA0..",
    "..0AABAABAABAABAA0..",
    "..0AABAABAABAABAA0..",
    "...00000000000000...",
];

/// Tous les sprites de ce fichier : (nom, dessin).
pub const SPRITES: &[(&str, Image)] = &[
    ("goutte0", GOUTTE0),
    ("goutte1", GOUTTE1),
    ("goutte2", GOUTTE2),
    ("goutte3", GOUTTE3),
    ("lanterne0", LANTERNE0),
    ("lanterne1", LANTERNE1),
    ("champignon0", CHAMPIGNON0),
    ("champignon1", CHAMPIGNON1),
    ("auvent0", AUVENT0),
    ("auvent1", AUVENT1),
    ("facade_vitrine", FACADE_VITRINE),
    ("facade_mur", FACADE_MUR),
    ("ventilateur_arret", VENTILATEUR_ARRET),
    ("ventilateur_aveugle", VENTILATEUR_AVEUGLE),
    ("ventilateur_souffle", VENTILATEUR_SOUFFLE),
    ("montgolfiere_repos", MONTGOLFIERE_REPOS),
    ("montgolfiere_feu0", MONTGOLFIERE_FEU0),
    ("montgolfiere_feu1", MONTGOLFIERE_FEU1),
    ("pointe", POINTE),
    ("friable0", FRIABLE0),
    ("friable1", FRIABLE1),
    ("friable_ardoise0", FRIABLE_ARDOISE0),
    ("friable_ardoise1", FRIABLE_ARDOISE1),
    ("mobile", MOBILE),
    ("aiguille", AIGUILLE),
    ("cadran", CADRAN),
    ("maillon", MAILLON),
    ("panneau_deplacer", PANNEAU_DEPLACER),
    ("panneau_sauter", PANNEAU_SAUTER),
    ("panneau_planer", PANNEAU_PLANER),
    ("panneau_champignon", PANNEAU_CHAMPIGNON),
    ("panneau_vent", PANNEAU_VENT),
    ("drapeau0", DRAPEAU0),
    ("drapeau1", DRAPEAU1),
    ("grelon0", GRELON0),
    ("grelon1", GRELON1),
    ("grelon2", GRELON2),
    ("grelon3", GRELON3),
    ("grele_eclat", GRELE_ECLAT),
];

/// Goutte d'or : 4 images de brillance.
pub const GOUTTE: [&str; 4] = ["goutte0", "goutte1", "goutte2", "goutte3"];
/// Lanterne : éteinte puis allumée.
pub const LANTERNE: [&str; 2] = ["lanterne0", "lanterne1"];
/// Champignon-trampoline : détendu puis comprimé.
pub const CHAMPIGNON: [&str; 2] = ["champignon0", "champignon1"];
/// Auvent : toile tendue puis enfoncée (rebond).
pub const AUVENT: [&str; 2] = ["auvent0", "auvent1"];
/// Façade de boutique derrière un auvent : la vitrine, puis le mur en dessous.
pub const FACADE: [&str; 2] = ["facade_vitrine", "facade_mur"];
/// Ventilateur : arrêt, annonce (lampe clignotante), souffle (courant actif).
pub const VENTILATEUR: [&str; 3] = [
    "ventilateur_arret",
    "ventilateur_aveugle",
    "ventilateur_souffle",
];
/// Montgolfière : brûleur éteint, puis les deux images de la flamme (montée).
pub const MONTGOLFIERE: [&str; 3] = ["montgolfiere_repos", "montgolfiere_feu0", "montgolfiere_feu1"];
/// Plateforme friable (nuages, niveaux 4 et 5) : intacte puis fissurée.
pub const FRIABLE: [&str; 2] = ["friable0", "friable1"];
/// Plateforme friable (dalle d'ardoise, niveaux 1 à 3) : intacte puis fendue.
pub const FRIABLE_ARDOISE: [&str; 2] = ["friable_ardoise0", "friable_ardoise1"];
/// Drapeau d'arrivée (2 images).
pub const DRAPEAU: [&str; 2] = ["drapeau0", "drapeau1"];
/// Grêlon qui roule en tombant (4 images, le reflet fait le tour).
pub const GRELON: [&str; 4] = ["grelon0", "grelon1", "grelon2", "grelon3"];
