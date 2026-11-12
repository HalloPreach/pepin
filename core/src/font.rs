//! Police bitmap 5x7 intégrée au code (aucune webfont, aucun asset externe).
//!
//! Un glyphe est un tableau de 7 octets (une rangée par octet, du haut vers
//! le bas) ; dans chaque octet, seuls les 5 bits de poids faible servent et
//! le bit 4 est la colonne de gauche. Les données sont pures : la couche
//! `game` se contente de dessiner les pixels allumés.
//!
//! Mise en page : les capitales et chiffres occupent les rangées 0-6, les
//! minuscules ont leur hauteur d'x sur les rangées 2-6 (les accents
//! minuscules logent dans les rangées 0-1). Les capitales accentuées sont
//! compressées sur 6 rangées pour laisser une rangée à l'accent.

#[rustfmt::skip] // blocs de glyphes lisibles : une ligne par glyphe
mod accents_maj;
#[rustfmt::skip] // blocs de glyphes lisibles : une ligne par glyphe
mod accents_min;
#[rustfmt::skip] // blocs de glyphes lisibles : une ligne par glyphe
mod chiffres;
#[rustfmt::skip] // blocs de glyphes lisibles : une ligne par glyphe
mod majuscules;
#[rustfmt::skip] // blocs de glyphes lisibles : une ligne par glyphe
mod minuscules;
mod outil;
#[rustfmt::skip] // blocs de glyphes lisibles : une ligne par glyphe
mod ponctuation;

pub use outil::Glyphe;

/// Largeur d'un glyphe en pixels.
pub const LARGEUR_GLYPHE: u32 = 5;
/// Hauteur d'un glyphe en pixels.
pub const HAUTEUR_GLYPHE: u32 = 7;
/// Espace vide entre deux glyphes consécutifs, en pixels.
pub const ESPACEMENT: u32 = 1;

/// Glyphe « inconnu » (cadre vide) affiché pour tout caractère absent de la table.
pub static GLYPHE_INCONNU: Glyphe = outil::g([
    "#####", "#...#", "#...#", "#...#", "#...#", "#...#", "#####",
]);

/// Tous les blocs de la table, parcourus dans l'ordre par [`glyphe`].
fn blocs() -> [&'static [outil::Entree]; 6] {
    [
        majuscules::MAJUSCULES,
        minuscules::MINUSCULES,
        chiffres::CHIFFRES,
        ponctuation::PONCTUATION,
        accents_min::ACCENTS_MIN,
        accents_maj::ACCENTS_MAJ,
    ]
}

/// Renvoie le glyphe d'un caractère, ou `None` s'il n'est pas dans la table.
pub fn glyphe(c: char) -> Option<&'static Glyphe> {
    blocs()
        .into_iter()
        .flat_map(|bloc| bloc.iter())
        .find(|(car, _)| *car == c)
        .map(|(_, dessin)| dessin)
}

/// Comme [`glyphe`], mais se replie sur [`GLYPHE_INCONNU`] au lieu de `None`.
pub fn glyphe_ou_inconnu(c: char) -> &'static Glyphe {
    glyphe(c).unwrap_or(&GLYPHE_INCONNU)
}

/// Indique si le pixel (`x`, `y`) du glyphe est allumé (`x` de 0 à 4, `y` de 0 à 6).
pub fn pixel_allume(dessin: &Glyphe, x: u32, y: u32) -> bool {
    x < LARGEUR_GLYPHE
        && y < HAUTEUR_GLYPHE
        && dessin[y as usize] & (1 << (LARGEUR_GLYPHE - 1 - x)) != 0
}

/// Largeur en pixels d'un texte à l'échelle donnée (espacement compris,
/// sans espace superflu après le dernier caractère). Vaut 0 pour un texte vide.
pub fn largeur_texte(texte: &str, echelle: u32) -> u32 {
    let nb = texte.chars().count() as u32;
    if nb == 0 {
        return 0;
    }
    (nb * LARGEUR_GLYPHE + (nb - 1) * ESPACEMENT) * echelle
}

/// Hauteur en pixels d'une ligne de texte à l'échelle donnée.
pub fn hauteur_texte(echelle: u32) -> u32 {
    HAUTEUR_GLYPHE * echelle
}

/// Liste les pixels allumés d'un texte (coordonnées `(x, y)` à l'échelle 1,
/// origine en haut à gauche). Les caractères inconnus donnent le glyphe de repli.
pub fn pixels_texte(texte: &str) -> Vec<(u32, u32)> {
    let mut pixels = Vec::new();
    for (i, c) in texte.chars().enumerate() {
        let dessin = glyphe_ou_inconnu(c);
        let x0 = i as u32 * (LARGEUR_GLYPHE + ESPACEMENT);
        for y in 0..HAUTEUR_GLYPHE {
            for x in 0..LARGEUR_GLYPHE {
                if pixel_allume(dessin, x, y) {
                    pixels.push((x0 + x, y));
                }
            }
        }
    }
    pixels
}

#[cfg(test)]
mod tests;
