//! Bibliothèque pure du jeu : aucune dépendance graphique,
//! entièrement testable avec `cargo test -p core`.

pub mod aabb;
pub mod entities;
pub mod font;
pub mod level;
pub mod palette;
pub mod physics;
pub mod player;
pub mod state;
pub mod timestep;

/// Largeur logique de l'écran en pixels.
pub const LARGEUR_LOGIQUE: u32 = 320;
/// Hauteur logique de l'écran en pixels.
pub const HAUTEUR_LOGIQUE: u32 = 180;
