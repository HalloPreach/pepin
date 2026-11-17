//! Monde 2, niveau 1 : « Le caniveau » (carte 9.6). Le premier vrai niveau du
//! Monde 2, « Là où va la pluie » : on y APPREND LA BARQUE en suivant l'eau des
//! gouttières, de la rue sèche jusqu'à la bouche d'égout. Difficulté : celle des
//! deux premiers niveaux du Monde 1, au plus (une seule flaque, aucun ennemi
//! d'eau : ils arrivent dans les égouts et à la rivière).
//!
//! Lecture du niveau (134 x 11 tuiles, un écran de haut), de gauche à droite.
//! Carte 9.20a (le nom et le dessin d'accord) : TOUTE l'eau coule dans le
//! caniveau, au ras de la rue (rangée 8), et grossit d'un bassin à l'autre ; plus
//! de gouttière sur un toit, la sortie est la bouche d'égout.
//! 1. le CANIVEAU À SEC (colonnes 9 à 24) : un creux d'une tuile dans la rue où
//!    l'on marche ; une flaque y patrouille, la 1re goutte flotte au-dessus d'elle
//!    (rangée 6) : on la prend d'un saut, en écrasant la flaque ou en l'évitant ;
//! 2. le PREMIER FILET D'EAU (colonnes 33 à 46), une tuile de fond, sans aucun
//!    danger : le panneau BAS (colonne 28, `Z` : la flèche plonge dans le parapluie
//!    retourné) le précède. Trop large pour un
//!    saut planant (14 tuiles), on y entre parapluie retourné (BAS, carte
//!    9.2b) : il devient une barque (eau calme `~`, puis un courant `}` qui la pousse). La
//!    2e goutte est posée sur l'eau (colonne 44) ; la BORDURE (colonnes 47 à 49,
//!    une tuile plus haut que la rue) arrête la barque : on en sort d'un saut ;
//! 3. la DESCENTE DE GOUTTIÈRE (`J`, colonne 59) : le tuyau descend la façade, sa
//!    gargouille traverse le trottoir et se vide dans le caniveau, qui devient un COURANT (colonnes 61 à 96, 36
//!    tuiles). Il passe sous la MARQUISE d'une boutique (colonnes 77 à 83 : une
//!    corniche de pierre, son store dessous) : la 3e goutte y attend (colonne 81,
//!    rangée 7), dans le seul espace libre entre l'eau et le store. On ne
//!    l'atteint qu'EN BARQUE. Au bout, la bordure (colonnes 97 à 99) ;
//! 4. le GRAND COURANT (colonnes 107 à 126, deux tuiles de fond) : une seconde
//!    descente (colonne 105) le grossit ; il file droit dans la BOUCHE D'ÉGOUT,
//!    ouverte dans la bordure du trottoir (colonne 127, `tuiles_theme`) : le
//!    drapeau est planté sur sa grille. Sortir de la barque, c'est arriver.
//!
//! Quatre lanternes : avant le filet d'eau (30), après sa bordure (52), au pied de
//! la descente de gouttière (57) et avant le grand courant (103).
//! Aucun bassin n'est suspendu au-dessus d'un passage à pied : l'eau est vérifiée
//! à la hauteur de Pépin (`physics::eau_touchee`).

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

/// Dessin du niveau (134 x 11 tuiles). Légende : voir `crate::level`.
/// Décor de rue (sans collision, carte 9.6b) : une poubelle `U` au départ, une borne
/// `I` au bout du caniveau à sec et une après le premier filet, deux descentes de
/// gouttière `J` (carte 9.20a : leur eau tombe dans le caniveau). Plus de bouche
/// d'égout `E` en décor : la seule est la sortie, dessinée dans la bordure.
const ASCII: &str = r"
......................................................................................................................................
......................................................................................................................................
......................................................................................................................................
......................................................................................................................................
......................................................................................................................................
.............................................................................#######..................................................
....................o........................................................BBBBBBB...........................................X......
..@..D.U..................I.Z.L.............o..###..L..I.L.J.....................o...............###...L.J.....................###....
#########.......f........########~~~}}}}}}}}}}}BBB###########~~}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}BBB#######~~}}}}}}}}}}}}}}}}}}BBB####
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB~~}}}}}}}}}}}}}}}}}}BBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";

#[cfg(test)]
mod tests {
    use super::*;

    /// Les rangées du dessin (rangée 0 en haut), en caractères.
    fn grille() -> Vec<Vec<char>> {
        ASCII.trim().lines().map(|l| l.chars().collect()).collect()
    }

    /// La goutte de la marquise ne s'atteint qu'EN BARQUE : elle est dans la seule
    /// rangée libre entre l'eau (dessous) et le store (dessus), et toute cette
    /// rangée sous la marquise est au-dessus de l'eau : nulle part où se tenir.
    #[test]
    fn la_goutte_de_la_marquise_ne_s_atteint_qu_en_barque() {
        let g = grille();
        assert_eq!(g[7][81], 'o');
        // Les colonnes 77 à 83 (la marquise) des rangées 6, 7 et 8.
        let sous_la_marquise = |rangee: usize| &g[rangee][77..=83];
        let colonnes = sous_la_marquise(6)
            .iter()
            .zip(sous_la_marquise(7))
            .zip(sous_la_marquise(8));
        for (k, ((store, air), eau)) in colonnes.enumerate() {
            let x = 77 + k;
            assert_eq!(*store, 'B', "store de la marquise, colonne {x}");
            assert!(matches!(air, '.' | 'o'), "rangée libre, colonne {x}");
            assert_eq!(*eau, '}', "courant d'eau, colonne {x}");
        }
    }

    /// Carte 9.20a : l'eau reste DANS LE CANIVEAU (aucune case d'eau au-dessus de
    /// la rue, rangée 8), chaque bassin est au moins aussi grand que le précédent
    /// (le dernier a deux tuiles de fond), et une descente de gouttière `J` se vide
    /// dans les deux derniers (posée deux cases avant l'eau : sa gargouille traverse
    /// le trottoir).
    #[test]
    fn l_eau_reste_au_caniveau_et_grossit() {
        let g = grille();
        let eau = |c: char| matches!(c, '~' | '{' | '}');
        for (y, ligne) in g.iter().enumerate().take(8) {
            assert!(
                !ligne.iter().any(|&c| eau(c)),
                "eau au-dessus de la rue, rangée {y}"
            );
        }
        // Les bassins de la rue, de gauche à droite : (première colonne, cases d'eau).
        let mut bassins: Vec<(usize, usize)> = Vec::new();
        for x in 0..g[8].len() {
            let cases = (8..g.len()).filter(|&y| eau(g[y][x])).count();
            if cases == 0 {
                continue;
            }
            match bassins.last_mut() {
                Some((_, n)) if x > 0 && eau(g[8][x - 1]) => *n += cases,
                _ => bassins.push((x, cases)),
            }
        }
        assert_eq!(bassins.len(), 3, "{bassins:?}");
        assert!(bassins.windows(2).all(|p| p[0].1 <= p[1].1), "{bassins:?}");
        assert!(
            eau(g[9][bassins[2].0]),
            "le dernier courant a deux tuiles de fond"
        );
        for &(debut, _) in &bassins[1..] {
            assert_eq!(
                g[7][debut - 2],
                'J',
                "descente de gouttière avant la colonne {debut}"
            );
        }
    }

    /// La sortie est la bouche d'égout : le drapeau est planté sur la bordure qui
    /// arrête le dernier courant, celle dont `tuiles_theme` ouvre la bouche.
    #[test]
    fn le_drapeau_est_sur_la_bouche_d_egout() {
        let g = grille();
        let x = g[6].iter().position(|&c| c == 'X').expect("drapeau");
        assert_eq!(g[7][x], '#', "le drapeau est sur la bordure");
        assert_eq!((g[8][x], g[9][x]), ('B', 'B'));
        assert_eq!(
            (g[8][x - 1], g[9][x - 1]),
            ('}', '}'),
            "l'eau s'y engouffre"
        );
        assert!(!ASCII.contains('E'), "une seule bouche d'égout : la sortie");
    }

    /// Aucun bassin suspendu au-dessus d'un passage à pied (l'eau est vérifiée à
    /// la hauteur de Pépin) : sous chaque case d'eau, de l'eau (le grand courant a
    /// deux tuiles de fond, carte 9.20a) puis du solide jusqu'en bas.
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
    /// Monde 2 : eau, courants d'eau, un nénuphar au plus, aucun ennemi d'eau.
    #[test]
    fn seulement_les_mecaniques_du_premier_niveau() {
        let lettres: String = ASCII.chars().filter(|c| !c.is_whitespace()).collect();
        for interdite in ['j', 'i', 'k', 'r', 'u', 'Y', '=', 'V', '|'] {
            assert!(!lettres.contains(interdite), "{interdite}");
        }
        assert!(lettres.matches('n').count() <= 1);
        assert_eq!(lettres.matches('o').count(), 3);
        assert_eq!(lettres.matches('L').count(), 4);
    }
}
