//! Monde 2, niveau 4 : « La grotte aux bulles » (carte 9.9, refaite à la carte
//! 9.20d, allongée à la carte 9.21c). DERRIÈRE LA CASCADE, une grotte : on entre
//! en traversant le rideau d'eau de la chute (au pied, à gauche), puis on y MONTE,
//! porté par des colonnes de bulles qui jaillissent chacune de sa SOURCE (une
//! vasque d'eau bouillonnante ou une fissure dans la roche), sous des stalactites
//! qui tremblent puis tombent. On ressort en haut, au jour, au bord de la rivière.
//! La pluie est restée dehors (thème sans pluie). Demande d'AGA : « ascensions en
//! bulles, stalactites qui tombent » ; (9.20) le niveau ressemble à son nom ;
//! (9.21) « les autres niveaux sont trop courts ».
//!
//! Lecture du niveau (36 x 80 tuiles, sept écrans de haut ; la caméra suit Pépin
//! en hauteur), de bas en haut, en zigzag. Les deux sections du bas (rangées 48
//! à 79) sont celles de la carte 9.21c ; au-dessus (rangées 0 à 47), la grotte de
//! la carte 9.20d, aux mêmes coordonnées :
//! 1. le LAC SOUS LA CASCADE (sol en rangée 78), vers la DROITE : l'ENTRÉE, le
//!    mur de gauche ouvert (rangées 70 à 77) sur le rideau d'eau de la cascade ;
//!    un lac de 16 cases (colonnes 5 à 20) sous une haute voûte. On saute sur le
//!    nénuphar (colonne 7) et l'on tombe à l'eau EN PLANANT : la barque. Un
//!    rideau de roche pendu du plafond (colonnes 10 à 16) descend à deux rangées
//!    de l'eau : seule la barque passe dessous. On rame jusqu'à la rive (lanterne
//!    1, colonne 26), où la colonne CA (33-34) jaillit de sa vasque et monte d'un
//!    trait jusqu'à la fosse ;
//! 2. la FOSSE DU CRABE (rives en rangée 57), vers la GAUCHE : en sortant de CA,
//!    on court sous un surplomb bas d'où pendent trois stalactites en série
//!    (colonnes 31, 28 et 25) ; puis une fosse de 14 cases (colonnes 10 à 23),
//!    trop large pour un saut plané, où patrouille un crabe. Une colonne (18-19)
//!    jaillit d'une vasque au fond : on y saute, elle porte AU-DESSUS DU CRABE, et
//!    l'on plane vers la gauche jusqu'à la rive (lanterne 2, colonne 8). La goutte
//!    du fond de la fosse (colonne 10), près du crabe, se prend en y descendant.
//!    Au bout, C0 (1-2) jaillit de sa vasque contre le mur et perce le sol de la
//!    salle du bassin ;
//! 3. la SALLE DU BASSIN (sol en rangée 46), vers la DROITE : un bassin
//!    (colonnes 8 à 12) et son nénuphar ; un crabe patrouille le sol, une
//!    stalactite (colonne 15) pend sous la corniche P1. Au bout, C1 (30-31)
//!    jaillit de sa vasque et soulève Pépin jusqu'à P1 (rangée 35), à sa gauche ;
//! 4. P1, vers la GAUCHE : la lanterne 3 (colonne 27), puis le COULOIR des
//!    stalactites (colonnes 11 et 16), bas de plafond. Au bout, C2 (4-5) jaillit
//!    d'une fissure du sol de P1 : on la traverse pour se placer contre le mur,
//!    puis on y entre vers la DROITE ;
//! 5. le DESSUS DU ROCHER (rangée 23), vers la DROITE : en sortant des bulles, on
//!    passe sous le surplomb et ses deux stalactites (colonnes 8 et 11), une goutte
//!    entre elles ; la lanterne 4 (colonne 13), un bassin (colonnes 15 à 18) et
//!    une grenouille sur sa rive (colonne 20) ;
//! 6. le MOMENT FORT : C3 (22-23), née de sa vasque au bord de la corniche de
//!    repli (rangée 26), monte jusqu'à la rangée 9 ; C4 (28-29) sort d'une FISSURE
//!    au sommet d'un pilier de roche (rangées 21 à 26) et porte jusqu'au drapeau.
//!    Entre les deux, rien où se poser : on sort du haut de C3 EN PLANANT pour
//!    passer dans C4, EN L'AIR, sous une stalactite (colonne 25). Raté : on
//!    retombe sur la corniche de repli (lanterne 5), d'où l'on reprend C3 ;
//! 7. le DRAPEAU, en haut à droite (colonne 33), sur un rocher accroché au mur :
//!    en sortant de C4, on passe sous une dernière stalactite (colonne 30). La
//!    SORTIE : le mur de droite ouvert au-dessus du rocher (rangées 1 à 3), le
//!    jour qui se lève et les herbes de la rive ;
//! 8. la GOUTTE RISQUÉE, avant le drapeau : depuis le rocher du drapeau, un saut
//!    planant vers la GAUCHE à travers le haut de C4 (elle soulève), sous la
//!    stalactite du passage, puis un long plané au-dessus du vide jusqu'à la
//!    petite corniche P5 (rangée 7) et sa goutte ; retour par un saut planant
//!    dans C4. Raté : la chute finit au pilier ou sur la corniche.
//!
//! Dix stalactites, trois ennemis (deux crabes, la grenouille), cinq lanternes,
//! sept colonnes de bulles (sept sources), deux oiseaux de décor (les
//! hirondelles de la grotte). Aucune lanterne dans la colonne d'une stalactite.
//! Une colonne contre un mur sort d'une vasque : on y attend en barque, puis on
//! saute dans le sens de la sortie (une colonne de fissure laisse la place de se
//! mettre derrière elle, comme C2). Carte 9.21c, première version (quatre
//! sections, 86 rangées, avec un pont de nénuphars et deux poissons) : l'Habitué
//! y mettait 102 s ; elle a été resserrée en deux sections. Rendu des sources, du
//! rideau d'eau et de la sortie : `game/src/decor_grotte.rs`.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    monde: 2,
    // Sa place dans le Monde 2 de SA branche (après le caniveau) : le coordinateur
    // la range à la 4e place à la fusion (recette, point 3).
    numero: 4,
    cle: "grotte",
    sans_planer: false,
    nom: "La grotte aux bulles",
    theme: Theme::Grotte,
    genre: GenreNiveau::Normal,
    intro: "Derrière la cascade, une grotte. Des sources bouillonnent : les bulles te portent.",
    mecanique: "Ascensions en bulles (passer d'une colonne à l'autre en planant) ; stalactites qui tremblent puis tombent : ne pas s'arrêter dessous.",
    duree_cible_s: 60,
    ascii: ASCII,
    aiguilles: &[],
};

/// Dessin du niveau (36 x 80 tuiles). Légende : voir `crate::level`.
/// - Murs (colonnes 0 et 35) et plafond (rangée 0) de briques ; sol du bas en
///   rangée 78 (le lac), départ en colonne 2. Deux ouvertures dans les murs (le
///   bord de la carte reste un mur pour la collision) : l'entrée (colonne 0,
///   rangées 70 à 77, le rideau de la cascade) et la sortie (colonne 35, rangées
///   1 à 3, le jour).
/// - Bulles `Y`, de bas en haut : CA (33-34, rangées 56 à 77), la colonne de la
///   fosse (18-19, 50 à 58), C0 (1-2, 45 à 56), C1 (30-31, 34 à 45), C2 (4-5, 22
///   à 34), C3 (22-23, 9 à 25), C4 (28-29, 3 à 20). Le haut d'une colonne est une
///   rangée au-dessus du sol où l'on en sort.
/// - Sources (cartes 9.20d et 9.21c) : une vasque d'eau `~` de deux cases au pied
///   de CA (rangée 78), de la colonne de la fosse (59), de C0 (57), de C1 (46) et
///   de C3 (26) : les bulles en partent (la colonne descend jusqu'au fond de
///   l'eau, `level::ascii`). Une fissure dans la roche sous C2 (sol de P1) et C4
///   (sommet d'un pilier de roche, 28-29, rangées 21 à 26).
/// - Stalactites `V` : (25, 54), (28, 54), (31, 54) sous le surplomb de la
///   galerie (colonnes 24 à 34, rangées 48 à 53), puis (15, 36), (11, 31), (16,
///   31), (8, 19), (11, 19), (25, 1), (30, 1), chacune sous un solide.
/// - Eau : le lac (rangée 78, colonnes 5 à 20) sous la voûte (rangées 60 à 75,
///   colonnes 10 à 16) ; ceux de la salle du bassin et du rocher. Nénuphars `n` :
///   (7, 77), (10, 45).
/// - Corniches : P1 (rangée 35, 1 à 29), rocher (dessus en rangée 23, 6 à 21),
///   repli (rangée 26, 24 à 27), P5 (rangée 7, 16 à 19), drapeau (rangée 4, 30 à 34).
/// - Décor : deux oiseaux `K` (colonne 4 dans la salle du bassin, colonne 20 sur P1).
const ASCII: &str = r"
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
B........................V....V.....
B...................................
B...........................YY...X..
B...........................YY#####B
B...........................YYBBBBBB
B...............o...........YYBBBBBB
B...............####........YY.....B
B...........................YY.....B
B.....................YY....YY.....B
B.....................YY....YY.....B
B.....................YY....YY.....B
B.....................YY....YY.....B
B.....................YY....YY.....B
B.....................YY....YY.....B
B.....................YY....YY.....B
B.....................YY....YY.....B
B......BBBBBB.........YY....YY.....B
B......BBBBBB.........YY....YY.....B
B.......V..V..........YY....YY.....B
B.....................YY....YY.....B
B.....................YY....BB.....B
B...YY....o..L......j.YY....BB.....B
B...YY#########~~~~###YY....BB.....B
B...YYBBBBBBBBBBBBBBBBYY....BB.....B
B...YYBBBBBBBBBBBBBBBBYY...LBB.....B
B...YYBBBBBBBBBBBBBBBB~~####BB.....B
B...YYBBBBBBBBBBBBBBBBBB...........B
B...YYBBBBBBBBBBBBBBBBBB...........B
B...YYBBBBBBBBBBBBBBBBBB...........B
B...YYBBBBBBBBBBBBBBBBBB...........B
B...YY.....V....V..................B
B...YY.............................B
B...YY.............................B
B...YY..............K......L..YY...B
B#############################YY...B
B..............V..............YY...B
B.............................YY...B
B.............................YY...B
B.............................YY...B
B.............................YY...B
B.............................YY...B
B.............................YY...B
B.............................YY...B
B.............................YY...B
BYY.K.....n...........k.......YY...B
BYY#####~~~~~#################~~###B
BYYBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
BYY.....................BBBBBBBBBBBB
BYY.....................BBBBBBBBBBBB
BYY...............YY....BBBBBBBBBBBB
BYY...............YY....BBBBBBBBBBBB
BYY...............YY....BBBBBBBBBBBB
BYY...............YY....BBBBBBBBBBBB
BYY...............YY.....V..V..V...B
BYY...............YY...............B
BYY.....L.........YY.............YYB
B~~#######........YY....#########YYB
BBBBBBBBBBo...k...YY....BBBBBBBBBYYB
BBBBBBBBBB########~~####BBBBBBBBBYYB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBYYB
B.........BBBBBBB................YYB
B.........BBBBBBB................YYB
B.........BBBBBBB................YYB
B.........BBBBBBB................YYB
B.........BBBBBBB................YYB
B.........BBBBBBB................YYB
B.........BBBBBBB................YYB
B.........BBBBBBB................YYB
B.........BBBBBBB................YYB
..........BBBBBBB................YYB
..........BBBBBBB................YYB
..........BBBBBBB................YYB
..........BBBBBBB................YYB
..........BBBBBBB................YYB
..........BBBBBBB................YYB
.................................YYB
..@....n..................L......YYB
B####~~~~~~~~~~~~~~~~############~~B
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faisabilite::{analyser, Options};

    /// Les bulles sont LA mécanique du niveau (demande d'AGA). Sans elles
    /// (courants coupés), le robot de faisabilité ne quitte pas le bas de la
    /// grotte : ni drapeau, ni goutte risquée (la 1re dans l'ordre de lecture).
    /// Et le passage de la 3e colonne (22-23) à la 4e (28-29) se fait EN L'AIR :
    /// rien où se poser entre elles (colonnes 24 à 27) au-dessus de la lanterne
    /// de la corniche de repli (rangée 25), sauf la stalactite qui y pend.
    #[test]
    fn les_bulles_font_le_niveau() {
        let niv = INFOS.charger().unwrap();
        let sans_bulles = Options {
            courants: false,
            ..Options::default()
        };
        let r = analyser(&niv, niv.depart, sans_bulles);
        assert!(!r.arrivee, "le drapeau sans les bulles");
        assert!(!r.gouttes[0], "la goutte risquée sans les bulles");

        let g: Vec<Vec<char>> = ASCII.trim().lines().map(|l| l.chars().collect()).collect();
        for (rangee, ligne) in g.iter().enumerate().take(25).skip(1) {
            for (x, c) in ligne.iter().enumerate().take(28).skip(24) {
                assert!(
                    matches!(c, '.' | 'V'),
                    "colonne {x}, rangée {rangee} : un appui entre les deux colonnes"
                );
            }
        }
        assert!(g[9][22] == 'Y' && g[8][22] == '.', "haut de la 3e colonne");
        assert!(g[3][28] == 'Y' && g[20][28] == 'Y', "4e colonne");
    }

    /// Carte 9.21c : le lac du pied ne se traverse qu'en barque. Sous la voûte
    /// (colonnes 10 à 16, pendue du plafond), rien que l'eau et deux rangées
    /// d'air : aucun appui, et un plané entré sous la voûte (au plus 32 px
    /// au-dessus de l'eau) touche l'eau avant d'en ressortir. Toucher l'eau
    /// parapluie fermé, c'est couler : seule la barque passe.
    #[test]
    fn le_lac_se_traverse_en_barque() {
        use crate::physics::{VITESSE_PLANER_CHUTE, VITESSE_PLANER_X};
        let g: Vec<Vec<char>> = ASCII.trim().lines().map(|l| l.chars().collect()).collect();
        let voute = 10..=16;
        for x in voute.clone() {
            assert!((60..=75).all(|y| g[y][x] == 'B'), "voûte, colonne {x}");
            assert!(
                g[76][x] == '.' && g[77][x] == '.',
                "air sous la voûte ({x})"
            );
            assert_eq!(g[78][x], '~', "le lac sous la voûte ({x})");
        }
        assert!(g[78][5..=20].iter().all(|&c| c == '~'), "le lac (5 à 20)");
        let longueur = voute.count() as f32 * 16.0;
        let descente = longueur / VITESSE_PLANER_X * VITESSE_PLANER_CHUTE;
        assert!(
            descente > 32.0,
            "un plané passerait sous la voûte : {descente}"
        );
        // Le robot de faisabilité, lui, la passe (en barque) : la rive d'en face
        // et sa lanterne (la dernière dans l'ordre de lecture). Sa recherche est
        // celle de `zones::tests::monde_2_niveau_4_faisable`, qui exige TOUTES les
        // lanternes (carte 9.23 : elle n'est plus refaite ici).
        assert_eq!(g[77][26], 'L');
    }
}
