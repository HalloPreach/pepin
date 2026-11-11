//! Majuscules accentuées du français (É È Ê Ë À Â Ç Î Ï Ô Ù Û, plus Ä Ö Ü) + Œ.
//! Accent sur la rangée 0, lettre compressée sur 6 rangées (1-6).

use super::outil::{
    g, maj_accent, Entree, AIGU_MAJ, CIRCONFLEXE_MAJ, GRAVE_MAJ, TREMA_MAJ,
};

/// « A » compressé sur 6 rangées.
const A: [&str; 6] = [".###.", "#...#", "#####", "#...#", "#...#", "#...#"];
/// « E » compressé sur 6 rangées.
const E: [&str; 6] = ["#####", "#....", "####.", "#....", "#....", "#####"];
/// « I » compressé sur 6 rangées.
const I: [&str; 6] = [".###.", "..#..", "..#..", "..#..", "..#..", ".###."];
/// « O » compressé sur 6 rangées.
const O: [&str; 6] = [".###.", "#...#", "#...#", "#...#", "#...#", ".###."];
/// « U » compressé sur 6 rangées.
const U: [&str; 6] = ["#...#", "#...#", "#...#", "#...#", "#...#", ".###."];

pub static ACCENTS_MAJ: &[Entree] = &[
    ('É', maj_accent(AIGU_MAJ, E)),
    ('È', maj_accent(GRAVE_MAJ, E)),
    ('Ê', maj_accent(CIRCONFLEXE_MAJ, E)),
    ('Ë', maj_accent(TREMA_MAJ, E)),
    ('À', maj_accent(GRAVE_MAJ, A)),
    ('Â', maj_accent(CIRCONFLEXE_MAJ, A)),
    ('Ä', maj_accent(TREMA_MAJ, A)),
    ('Î', maj_accent(CIRCONFLEXE_MAJ, I)),
    ('Ï', maj_accent(TREMA_MAJ, I)),
    ('Ô', maj_accent(CIRCONFLEXE_MAJ, O)),
    ('Ö', maj_accent(TREMA_MAJ, O)),
    ('Ù', maj_accent(GRAVE_MAJ, U)),
    ('Û', maj_accent(CIRCONFLEXE_MAJ, U)),
    ('Ü', maj_accent(TREMA_MAJ, U)),
    // Cédille : « C » sur 6 rangées (0-5), cédille en rangée 6.
    ('Ç', g([".###.", "#...#", "#....", "#....", "#...#", ".###.", "..#.."])),
    // Ligature Œ : « O » étroit accolé à un « E » (7 rangées).
    ('Œ', g([".####", "#.#..", "#.#..", "#.###", "#.#..", "#.#..", ".####"])),
];
