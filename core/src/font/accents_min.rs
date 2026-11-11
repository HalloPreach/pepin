//! Minuscules accentuées du français (et quelques voisines : ä ö ü ÿ) + œ.
//! Accent sur les rangées 0-1, corps de la lettre sur les rangées 2-6.

use super::outil::{
    g, min_accent, Entree, AIGU, CIRCONFLEXE, GRAVE, TREMA,
};

/// Corps du « a » (rangées 2-6).
const A: [&str; 5] = [".###.", "....#", ".####", "#...#", ".####"];
/// Corps du « e ».
const E: [&str; 5] = [".###.", "#...#", "#####", "#....", ".###."];
/// Corps du « i » sans point.
const I: [&str; 5] = [".##..", "..#..", "..#..", "..#..", ".###."];
/// Corps du « o ».
const O: [&str; 5] = [".###.", "#...#", "#...#", "#...#", ".###."];
/// Corps du « u ».
const U: [&str; 5] = ["#...#", "#...#", "#...#", "#..##", ".##.#"];
/// Corps du « y » (jambage intégré aux rangées 5-6).
const Y: [&str; 5] = ["#...#", "#...#", ".####", "....#", ".###."];

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
    // Cédille : pas d'accent au-dessus, la cédille occupe la rangée 6.
    ('ç', g([".....", ".....", ".###.", "#...#", "#....", ".###.", "..#.."])),
    // Ligature œ : deux voyelles accolées sur 5 colonnes.
    ('œ', g([".....", ".....", ".####", "#.#.#", "#.###", "#.#..", ".####"])),
];
