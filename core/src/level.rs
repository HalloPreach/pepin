//! Module `level` : format de niveau, grille de tuiles et entités instanciées.
//!
//! Un niveau s'écrit en ASCII dans une constante `&str` (voir `levels/`) et se
//! lit avec `Niveau::depuis_ascii`, qui renvoie un `Result` : soit le niveau,
//! soit une `ErreurNiveau` précise (ligne, colonne...). Aucun fichier externe.
//!
//! Légende (1 caractère = 1 tuile de 16 x 16 px ; la ligne 0 est en HAUT) :
//!
//! ```text
//!   .  vide                    #  toit (solide)      B  brique (solide)
//!   N  nuage (solide)          @  départ de Pépin     X  arrivée (drapeau)
//!   L  lanterne (checkpoint)   o  goutte d'or (3 par niveau)
//!   ^  pointes                 M  champignon-trampoline
//!   f  flaque (patrouille)     c  corbeau (vol)
//!   F  plateforme friable      e  éclair (colonne jusqu'au 1er solide dessous)
//!   v  courant ascendant       1-9 extrémités d'une plateforme mobile (2 par chiffre)
//!   D S G C W  panneaux : déplacer, sauter, planer, champignon, vent
//! ```
//!
//! Notions Rust illustrées ici (à reprendre dans le README) :
//! - `enum` (`Tuile`, `GenrePanneau`) et `match` exhaustif ;
//! - `Result<T, E>` avec une erreur typée (`ErreurNiveau`) et l'opérateur `?` ;
//! - `trait` : `Niveau` implémente `Grille`, donc le moteur de collision le
//!   comprend sans rien savoir de son format ;
//! - modules découpés en petits fichiers (`level/erreur.rs`, `level/ascii.rs`).

mod ascii;
mod erreur;
#[cfg(test)]
mod tests;

pub use erreur::ErreurNiveau;

/// Vitesse (px/s) des plateformes mobiles posées par le parseur.
pub const VITESSE_PLATEFORMES: f32 = ascii::VITESSE_MOBILE;

use crate::aabb::{self, Aabb, Grille, TAILLE_TUILE};
use crate::entities::Entite;
use crate::physics::CourantAscendant;

/// Nombre de gouttes d'or dans chaque niveau.
pub const GOUTTES_PAR_NIVEAU: usize = 3;

/// Nature d'une tuile du décor. Tout sauf `Vide` est solide ; la variante ne
/// sert qu'à choisir le dessin (toit en tuiles, brique de cheminée, nuage).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tuile {
    Vide,
    Toit,
    Brique,
    Nuage,
}

impl Tuile {
    /// La tuile bloque-t-elle le joueur ?
    pub fn est_solide(self) -> bool {
        !matches!(self, Tuile::Vide)
    }
}

/// Sujet d'un panneau dessiné (tutoriel sans texte).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenrePanneau {
    Deplacer,
    Sauter,
    Planer,
    Champignon,
    Vent,
}

/// Panneau décoratif ; `(x, y)` est le coin haut-gauche de sa tuile.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Panneau {
    pub x: f32,
    pub y: f32,
    pub genre: GenrePanneau,
}

/// Un niveau entièrement chargé : tuiles, entités, courants, repères.
#[derive(Debug, Clone)]
pub struct Niveau {
    pub(crate) largeur: usize,
    pub(crate) hauteur: usize,
    pub(crate) tuiles: Vec<Tuile>,
    /// Coin haut-gauche du joueur au départ.
    pub depart: (f32, f32),
    /// Zone d'arrivée : le toucher termine le niveau.
    pub arrivee: Aabb,
    /// Entités à l'état initial (à cloner pour recommencer).
    pub entites: Vec<Entite>,
    /// Colonnes de vent ascendant.
    pub courants: Vec<CourantAscendant>,
    /// Panneaux du tutoriel.
    pub panneaux: Vec<Panneau>,
}

impl Niveau {
    /// Lit un niveau ASCII (voir la légende en tête de module).
    pub fn depuis_ascii(texte: &str) -> Result<Niveau, ErreurNiveau> {
        ascii::analyser(texte)
    }

    /// Largeur en tuiles.
    pub fn largeur(&self) -> usize {
        self.largeur
    }

    /// Hauteur en tuiles.
    pub fn hauteur(&self) -> usize {
        self.hauteur
    }

    /// Largeur en pixels.
    pub fn largeur_px(&self) -> f32 {
        self.largeur as f32 * TAILLE_TUILE
    }

    /// Hauteur en pixels ; sert aussi de « bas du monde » pour détecter le vide.
    pub fn hauteur_px(&self) -> f32 {
        self.hauteur as f32 * TAILLE_TUILE
    }

    /// Tuile de décor en `(tx, ty)`. Mêmes règles hors carte que `GrilleTuiles` :
    /// murs à gauche et à droite, ciel ouvert en haut, vide en bas.
    pub fn tuile_decor(&self, tx: i32, ty: i32) -> Tuile {
        if tx < 0 || tx >= self.largeur as i32 {
            return Tuile::Brique;
        }
        if ty < 0 || ty >= self.hauteur as i32 {
            return Tuile::Vide;
        }
        self.tuiles[ty as usize * self.largeur + tx as usize]
    }

    /// Nombre de lanternes (checkpoints).
    pub fn nb_lanternes(&self) -> usize {
        self.entites
            .iter()
            .filter(|e| matches!(e, Entite::Lanterne(_)))
            .count()
    }
}

/// Le moteur de collision ne connaît que ce contrat : un niveau en est un.
impl Grille for Niveau {
    fn tuile(&self, tx: i32, ty: i32) -> aabb::Tuile {
        if self.tuile_decor(tx, ty).est_solide() {
            aabb::Tuile::Solide
        } else {
            aabb::Tuile::Vide
        }
    }
}
