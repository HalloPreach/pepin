//! Minuscules accentuées du français (et quelques voisines : ä ö ü ÿ) + œ.
//! Accent sur la rangée 0, corps de la lettre sur les rangées 1-4 (hauteur
//! d'x) ; la cédille et le jambage du ÿ descendent dans les rangées 5-6.

use super::outil::{
    g, min_accent, Entree, AIGU, CIRCONFLEXE, GRAVE, TREMA,
};

/// Corps du « a » (rangées 1-6 ; les rangées 5-6 restent vides).
const A: [&str; 6] = [".###.", "....#", "#####", "#...#", ".....", "....."];
/// Corps du « e ».
const E: [&str; 6] = [".###.", "#####", "#....", ".###.", ".....", "....."];
/// Corps du « i » sans point.
const I: [&str; 6] = [".##..", "..#..", "..#..", ".###.", ".....", "....."];
/// Corps du « o ».
const O: [&str; 6] = [".###.", "#...#", "#...#", ".###.", ".....", "....."];
/// Corps du « u ».
const U: [&str; 6] = ["#...#", "#...#", "#..##", ".##.#", ".....", "....."];
/// Corps du « y » (jambage sur les rangées 5-6).
const Y: [&str; 6] = ["#...#", "#...#", ".#.#.", "..#..", ".#...", "#...."];

pub static ACCENTS_MIN: &[Entree] = &[
    ('é', min_accent(AIGU, E)),
    ('è', min_accent(GRAVE, E)),
    ('ê', min_accent(CIRCONFLEXE, E)),
    ('ë', min_accent(TREMA, E)),
    ('à', min_accent(GRAVE, A)),
    ('â', min_accent(CIRCONFLEXE, A)),
    ('ä', min_accent(TREMA, A)),
    ('î', min_accent(CIRCONFLEXE, I)),
    ('ï', min_accent(TREMA, I)),
    ('ô', min_accent(CIRCONFLEXE, O)),
    ('ö', min_accent(TREMA, O)),
    ('ù', min_accent(GRAVE, U)),
    ('û', min_accent(CIRCONFLEXE, U)),
    ('ü', min_accent(TREMA, U)),
    ('ÿ', min_accent(TREMA, Y)),
    // Cédille : pas d'accent au-dessus, la cédille occupe les rangées 5-6.
    ('ç', g([".....", ".###.", "#....", "#....", ".###.", "..#..", ".##.."])),
    // Ligature œ : deux voyelles accolées sur 5 colonnes.
    ('œ', g([".....", ".####", "#.#.#", "#.###", ".####", ".....", "....."])),
];
