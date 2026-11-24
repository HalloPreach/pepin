//! Tests propres au niveau « grotte » (autrefois le `mod tests` de
//! `core/src/levels/grotte.rs` ; la grille et ses commentaires de conception
//! sont dans son fichier `.ron`).

use crate::commun::*;
use pepin_core::faisabilite::{analyser, Options};

/// Le niveau de ce module, lu dans son fichier `.ron`.
fn infos() -> &'static InfosNiveau {
    par_cle("grotte")
}

/// Les bulles sont LA mécanique du niveau (demande d'AGA). Sans elles
/// (courants coupés), le robot de faisabilité ne quitte pas le bas de la
/// grotte : ni drapeau, ni goutte risquée (la 1re dans l'ordre de lecture).
/// Et le passage de la 3e colonne (22-23) à la 4e (28-29) se fait EN L'AIR :
/// rien où se poser entre elles (colonnes 24 à 27) au-dessus de la lanterne
/// de la corniche de repli (rangée 25), sauf la stalactite qui y pend.
#[test]
fn les_bulles_font_le_niveau() {
    let niv = infos().charger().unwrap();
    let sans_bulles = Options {
        courants: false,
        ..Options::default()
    };
    let r = analyser(&niv, niv.depart, sans_bulles);
    assert!(!r.arrivee, "le drapeau sans les bulles");
    assert!(!r.gouttes[0], "la goutte risquée sans les bulles");

    let g: Vec<Vec<char>> = infos()
        .ascii
        .trim()
        .lines()
        .map(|l| l.chars().collect())
        .collect();
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
    use pepin_core::physics::{VITESSE_PLANER_CHUTE, VITESSE_PLANER_X};
    let g: Vec<Vec<char>> = infos()
        .ascii
        .trim()
        .lines()
        .map(|l| l.chars().collect())
        .collect();
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
    // celle de `faisabilite::monde_2_niveau_4_faisable`, qui exige TOUTES les
    // lanternes (carte 9.23 : elle n'est plus refaite ici).
    assert_eq!(g[77][26], 'L');
}
