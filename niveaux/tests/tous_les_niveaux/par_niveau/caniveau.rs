//! Tests propres au niveau « caniveau » (autrefois le `mod tests` de
//! `core/src/levels/caniveau.rs` ; la grille et ses commentaires de conception
//! sont dans son fichier `.ron`).

use crate::commun::*;

/// Le niveau de ce module, lu dans son fichier `.ron`.
fn infos() -> &'static InfosNiveau {
    par_cle("caniveau")
}

/// Les rangées du dessin (rangée 0 en haut), en caractères.
fn grille() -> Vec<Vec<char>> {
    infos()
        .ascii
        .trim()
        .lines()
        .map(|l| l.chars().collect())
        .collect()
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
    for (x, &dessous) in g[6].iter().enumerate().take(48).skip(41) {
        let colonne: String = (2..=5).map(|y| g[y][x]).collect();
        assert_eq!(colonne, "aaaa", "autobus, colonne {x}");
        assert_eq!(dessous, '.', "une rangée libre sous l'autobus, colonne {x}");
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
    assert!(
        !infos().ascii.contains('E'),
        "une seule bouche d'égout : la sortie"
    );
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
/// Monde 2 : eau, courants d'eau, un nénuphar au plus, six rats au plus
/// (cartes 9.20f, 9.22a et 9.22e : ils annoncent les égouts), aucun autre
/// ennemi d'eau.
#[test]
fn seulement_les_mecaniques_du_premier_niveau() {
    let lettres: String = infos()
        .ascii
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    for interdite in ['j', 'i', 'k', 'u', 'Y', '=', 'V', '|'] {
        assert!(!lettres.contains(interdite), "{interdite}");
    }
    assert!(lettres.matches('n').count() <= 1);
    assert!(lettres.matches('r').count() <= 6);
    assert_eq!(lettres.matches('o').count(), 3);
    assert_eq!(lettres.matches('L').count(), 4);
}
