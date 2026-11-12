//! Outils de construction des glyphes à la compilation.
//!
//! Chaque glyphe est dessiné sous forme de 7 chaînes de 5 caractères
//! (`#` = pixel allumé, `.` = pixel éteint), ce qui garde la table lisible.
//! Une erreur de dessin (mauvaise longueur, caractère inconnu) fait échouer
//! la compilation plutôt que de produire un glyphe cassé.

/// Un glyphe : 7 rangées, 5 bits utiles par rangée (bit 4 = colonne de gauche).
pub type Glyphe = [u8; 7];

/// Une entrée de table : le caractère Unicode et son dessin.
pub type Entree = (char, Glyphe);

/// Convertit une rangée « #...# » en octet (bit 4 = pixel de gauche).
pub const fn rangee(s: &str) -> u8 {
    let b = s.as_bytes();
    assert!(b.len() == 5, "une rangée de glyphe doit faire 5 pixels");
    let mut v = 0u8;
    let mut i = 0;
    while i < 5 {
        match b[i] {
            b'#' => v |= 1 << (4 - i),
            b'.' => {}
            _ => panic!("rangée de glyphe : seuls '#' et '.' sont permis"),
        }
        i += 1;
    }
    v
}

/// Construit un glyphe complet à partir de ses 7 rangées.
pub const fn g(r: [&str; 7]) -> Glyphe {
    [
        rangee(r[0]),
        rangee(r[1]),
        rangee(r[2]),
        rangee(r[3]),
        rangee(r[4]),
        rangee(r[5]),
        rangee(r[6]),
    ]
}

/// Minuscule accentuée : accent sur 1 rangée (0) + corps sur 6 rangées (1-6).
/// La hauteur d'x des minuscules est de 4 rangées (1-4) : l'accent est donc
/// collé au-dessus de la lettre ; les rangées 5-6 ne servent qu'aux jambages.
pub const fn min_accent(accent: &str, corps: [&str; 6]) -> Glyphe {
    g([
        accent, corps[0], corps[1], corps[2], corps[3], corps[4], corps[5],
    ])
}

/// Majuscule accentuée : accent sur 1 rangée (0) + lettre compressée sur 6 rangées (1-6).
pub const fn maj_accent(accent: &str, corps: [&str; 6]) -> Glyphe {
    g([
        accent, corps[0], corps[1], corps[2], corps[3], corps[4], corps[5],
    ])
}

/// Accent aigu sur une rangée (minuscules).
pub const AIGU: &str = "...#.";
/// Accent grave sur une rangée (minuscules).
pub const GRAVE: &str = ".#...";
/// Accent circonflexe sur une rangée (minuscules).
pub const CIRCONFLEXE: &str = "..#..";
/// Tréma sur une rangée (minuscules).
pub const TREMA: &str = ".#.#.";

/// Accent aigu d'une rangée (majuscules).
pub const AIGU_MAJ: &str = "...#.";
/// Accent grave d'une rangée (majuscules).
pub const GRAVE_MAJ: &str = ".#...";
/// Accent circonflexe d'une rangée (majuscules).
pub const CIRCONFLEXE_MAJ: &str = "..#..";
/// Tréma d'une rangée (majuscules).
pub const TREMA_MAJ: &str = ".#.#.";
