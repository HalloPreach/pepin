//! Monde 2, niveau 1 : « Le caniveau » (carte 9.6, REFAIT à la carte 9.20f). Le
//! premier vrai niveau du Monde 2, « Là où va la pluie » : une RUE DE NUIT SOUS LA
//! PLUIE qui descend par des marches de trottoir. Le caniveau la longe : un mince
//! filet d'abord, un vrai ruisseau à la fin, qui s'engouffre dans la bouche
//! d'égout. On y APPREND LA BARQUE (BAS) en trois temps. Difficulté : celle des
//! deux premiers niveaux du Monde 1, au plus (trois flaques, un rat hors du chemin
//! de la barque, et l'eau qui noie sans BAS).
//!
//! Lecture du niveau (188 x 11 tuiles, un écran de haut), de gauche à droite.
//! Le mobilier de rue (carte 9.20f, lettres `q a t d l`) est SOLIDE : on saute
//! d'un toit de voiture à l'autre, sur une poubelle, un banc, un abribus.
//! 1. la RUE EN HAUT (colonnes 0 à 29, pavés en rangée 6) : une poubelle, deux
//!    voitures garées ; entre elles, une flaque patrouille sous la 1re goutte
//!    (colonne 16, rangée 3), qu'on prend d'un saut ;
//! 2. une marche plus bas (rangée 7), le panneau BAS (`Z`, colonne 31) et une
//!    descente de gouttière (`J`) qui verse le premier FILET : 1er temps de la
//!    barque, cinq cases d'eau calme à l'air libre (colonnes 35 à 39) ;
//! 3. 2e temps : le filet file SOUS UN AUTOBUS garé (colonnes 41 à 47, quatre
//!    cases de haut, rangées 2 à 5) ; entre son bas de caisse et l'eau, une seule
//!    rangée : trop bas à pied (pas de sol, on s'y noie), on ne passe QU'EN
//!    BARQUE, et le toit de l'autobus est hors de portée d'un saut depuis la rue ;
//! 4. la sortie à l'ARRÊT DE BUS : l'abribus (toit en rangée 4, colonnes 52 à
//!    55 ; on passe dessous), un banc. Du toit de l'abribus, un saut en arrière
//!    mène au toit de l'autobus : la 3e goutte y est cachée (colonne 44, rangée
//!    1) ;
//! 5. la RUE COMMERÇANTE (colonnes 62 à 133, toujours en rangée 7) : des voitures
//!    garées, un second filet de cinq cases qu'on saute (colonnes 76 à 80), les
//!    AUVENTS de trois boutiques (rangée 5, ils relancent Pépin comme au Monde 1)
//!    au-dessus d'une flaque, un banc, puis un second autobus garé le long du
//!    trottoir (colonnes 106 à 111, quatre cases de haut) : on y grimpe par la
//!    poubelle d'à côté (une flaque patrouille sur son toit, sous la pluie), on
//!    redescend de l'autre côté ;
//! 6. encore une marche (rangée 8) : un RAT patrouille entre la marche et une
//!    poubelle (il annonce les égouts ; on l'écrase ou on le saute), loin de la
//!    barque ; une dernière descente de gouttière ;
//! 7. 3e temps : le RUISSEAU (colonnes 146 à 183, deux cases de fond), un long
//!    courant `}` où la barque file ; au bout, il passe sous la MARQUISE d'une
//!    boutique (colonnes 170 à 176) : la 2e goutte attend dans la seule rangée
//!    libre entre l'eau et le store (colonne 174), seulement en barque ;
//! 8. la BOUCHE D'ÉGOUT (colonne 184) : le ruisseau s'y engouffre, Pépin saute
//!    sur sa grille, où est planté le drapeau.
//!
//! Quatre lanternes : avant le filet (32), après l'abribus (60), après les
//! auvents (101), avant la dernière marche (129). Aucun bassin n'est suspendu
//! au-dessus d'un passage à pied : l'eau est vérifiée à la hauteur de Pépin
//! (`physics::eau_touchee`).

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    monde: 2,
    numero: 1,
    cle: "caniveau",
    sans_planer: false,
    nom: "Le caniveau",
    theme: Theme::Caniveau,
    genre: GenreNiveau::Normal,
    intro: "La pluie file dans le caniveau, jusqu'à la bouche d'égout. Pépin la suit.",
    mecanique: "La barque : BAS retourne le parapluie, qui flotte en touchant l'eau (sans lui, planer compris, on coule) ; se laisser porter par le courant, sauter pour sortir de l'eau.",
    duree_cible_s: 50,
    ascii: ASCII,
    aiguilles: &[],
};

/// Dessin du niveau (188 x 11 tuiles). Légende : voir `crate::level`.
/// `B` : le sol de la rue (pavés, terre) ; `#` : ce qui est bâti au-dessus (la
/// marquise). Mobilier de rue (carte 9.20f, solide) : `q` voiture, `a` autobus,
/// `t` toit d'abribus, `d` poubelle, `l` banc. Décor sans collision : des bornes
/// `I`, des poubelles `U`, trois descentes de gouttière `J` (leur eau tombe dans
/// le caniveau), les auvents `A` (ils relancent Pépin). La seule bouche d'égout
/// est la sortie, dessinée dans la bordure.
const ASCII: &str = r"
............................................................................................................................................................................................
............................................o...............................................................................................................................................
.........................................aaaaaaa.............................................................f..............................................................................
................o........................aaaaaaa..........................................................aaaaaa............................................................................
..........qq........qq...................aaaaaaa....tttt..................................................aaaaaa............................................................................
..@...d..qqqq..f...qqqq..I.U.............aaaaaaa...............qq......qq...........A...A...A.............aaaaaa...qq.......qq............................................#######...........
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBB.ZLJ.......................ll.L.qqqq....qqqqJ.......f...............ll.L..d.aaaaaa..qqqq.I.U.qqqq..L........................................#######.......X...
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB~~~~~}}}}}}}}}}}BBBBBBBBBBBBBBBBBBBBBBBBB~~~~~BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB..r.....d.J.............................o.........BBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}BBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}BBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";

#[cfg(test)]
mod tests {
    use super::*;

    /// Les rangées du dessin (rangée 0 en haut), en caractères.
    fn grille() -> Vec<Vec<char>> {
        ASCII.trim().lines().map(|l| l.chars().collect()).collect()
    }

    /// Les passages bas ne se passent qu'EN BARQUE. Sous la marquise, la goutte est
    /// dans la seule rangée libre entre l'eau (dessous) et le store (dessus), et
    /// toute cette rangée est au-dessus de l'eau : nulle part où se tenir. Sous
    /// l'autobus (carte 9.20f), une seule rangée libre au-dessus de l'eau sur toute
    /// sa longueur, et son toit est plus haut qu'un saut depuis la rue (4 cases).
    #[test]
    fn la_goutte_de_la_marquise_ne_s_atteint_qu_en_barque() {
        let g = grille();
        assert_eq!(g[7][174], 'o');
        // Les colonnes 170 à 176 (la marquise) des rangées 6, 7 et 8.
        let sous_la_marquise = |rangee: usize| &g[rangee][170..=176];
        let colonnes = sous_la_marquise(6)
            .iter()
            .zip(sous_la_marquise(7))
            .zip(sous_la_marquise(8));
        for (k, ((store, air), eau)) in colonnes.enumerate() {
            let x = 170 + k;
            assert_eq!(*store, '#', "store de la marquise, colonne {x}");
            assert!(matches!(air, '.' | 'o'), "rangée libre, colonne {x}");
            assert_eq!(*eau, '}', "courant d'eau, colonne {x}");
        }
        // L'autobus : colonnes 41 à 47, rangées 2 à 5 ; la rue est en rangée 7.
        for x in 41..=47 {
            let colonne: String = (1..=7).map(|y| g[y][x]).collect();
            assert!(
                colonne.ends_with("aaaa.}") || colonne.ends_with("aaaa.~"),
                "colonne {x} : {colonne}"
            );
        }
        assert!(g[7][40..=48].iter().all(|&c| matches!(c, '~' | '}')));
    }

    /// Carte 9.20a, refaite à la 9.20f : l'eau reste DANS LE CANIVEAU, au ras de
    /// la rue (la surface de chaque bassin est à la hauteur des pavés qui le
    /// bordent à gauche), chaque bassin est au moins aussi profond que le
    /// précédent (deux minces filets, puis un ruisseau de deux tuiles de fond), et
    /// une descente de gouttière `J` se vide dans chacun (posée deux cases avant
    /// l'eau : sa gargouille traverse le trottoir).
    #[test]
    fn l_eau_reste_au_caniveau_et_grossit() {
        let g = grille();
        let eau = |c: char| matches!(c, '~' | '{' | '}');
        // Les bassins, de gauche à droite : (première colonne, surface, profondeur).
        let mut bassins: Vec<(usize, usize, usize)> = Vec::new();
        for x in 0..g[0].len() {
            let Some(surface) = (0..g.len()).find(|&y| eau(g[y][x])) else {
                continue;
            };
            let fond = (surface..g.len()).take_while(|&y| eau(g[y][x])).count();
            match bassins.last_mut() {
                Some((_, s, n)) if x > 0 && eau(g[*s][x - 1]) => *n = (*n).max(fond),
                _ => bassins.push((x, surface, fond)),
            }
        }
        assert_eq!(bassins.len(), 3, "{bassins:?}");
        assert!(bassins.windows(2).all(|p| p[0].2 <= p[1].2), "{bassins:?}");
        assert_eq!(bassins[2].2, 2, "le ruisseau a deux tuiles de fond");
        for &(debut, surface, _) in &bassins {
            assert_eq!(g[surface][debut - 1], 'B', "pavés au bord, colonne {debut}");
            assert_eq!(
                g[surface - 1][debut - 1],
                '.',
                "au ras de la rue, colonne {debut}"
            );
            assert_eq!(
                g[surface - 1][debut - 2],
                'J',
                "descente de gouttière avant la colonne {debut}"
            );
        }
    }

    /// La sortie est la bouche d'égout : le drapeau est planté sur la bordure qui
    /// arrête le ruisseau, celle dont `tuiles_theme` ouvre la bouche.
    #[test]
    fn le_drapeau_est_sur_la_bouche_d_egout() {
        let g = grille();
        let y = g.iter().position(|l| l.contains(&'X')).expect("drapeau");
        let x = g[y].iter().position(|&c| c == 'X').unwrap();
        assert_eq!(g[y + 1][x], 'B', "le drapeau est sur la bordure");
        assert_eq!((g[y + 2][x], g[y + 3][x]), ('B', 'B'));
        assert_eq!(
            (g[y + 2][x - 1], g[y + 3][x - 1]),
            ('}', '}'),
            "l'eau s'y engouffre"
        );
        assert_eq!(g[y + 1][x - 1], '.', "le ruisseau est au ras de la rue");
        assert!(!ASCII.contains('E'), "une seule bouche d'égout : la sortie");
    }

    /// Aucun bassin suspendu au-dessus d'un passage à pied (l'eau est vérifiée à
    /// la hauteur de Pépin) : sous chaque case d'eau, de l'eau (le ruisseau a
    /// deux tuiles de fond) puis du solide jusqu'en bas.
    #[test]
    fn sous_l_eau_rien_que_du_solide() {
        let g = grille();
        for (y, ligne) in g.iter().enumerate() {
            for (x, &c) in ligne.iter().enumerate() {
                if matches!(c, '~' | '{' | '}') {
                    let mut fond = false;
                    for (dessous, rangee) in g.iter().enumerate().skip(y + 1) {
                        fond |= matches!(rangee[x], '#' | 'B');
                        let attendu = if fond { "#B" } else { "#B~{}" };
                        assert!(
                            attendu.contains(rangee[x]),
                            "colonne {x} : pas de solide en rangée {dessous} sous l'eau"
                        );
                    }
                    assert!(fond, "colonne {x} : l'eau touche le bas de la carte");
                }
            }
        }
    }

    /// Le niveau n'utilise que les lettres permises pour le premier niveau du
    /// Monde 2 : eau, courants d'eau, un nénuphar au plus, un rat au plus (carte
    /// 9.20f : il annonce les égouts), aucun autre ennemi d'eau.
    #[test]
    fn seulement_les_mecaniques_du_premier_niveau() {
        let lettres: String = ASCII.chars().filter(|c| !c.is_whitespace()).collect();
        for interdite in ['j', 'i', 'k', 'u', 'Y', '=', 'V', '|'] {
            assert!(!lettres.contains(interdite), "{interdite}");
        }
        assert!(lettres.matches('n').count() <= 1);
        assert!(lettres.matches('r').count() <= 1);
        assert_eq!(lettres.matches('o').count(), 3);
        assert_eq!(lettres.matches('L').count(), 4);
    }
}
