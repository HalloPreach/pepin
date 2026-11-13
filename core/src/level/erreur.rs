//! Erreurs typées du parseur de niveaux.
//!
//! Notions Rust : un `enum` d'erreurs à données (chaque variante porte le
//! contexte utile : ligne, colonne...) ; les traits `Display` (message lisible
//! par un humain) et `std::error::Error` (compatibilité avec `?` et `Box<dyn Error>`).

use std::fmt;

/// Tout ce qui peut mal tourner en lisant un niveau ASCII.
///
/// Les numéros de ligne et de colonne commencent à 1 (comme dans un éditeur).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErreurNiveau {
    /// Le texte ne contient aucune ligne de niveau.
    Vide,
    /// Une ligne n'a pas la même longueur que la première.
    LargeurInegale {
        ligne: usize,
        attendu: usize,
        trouve: usize,
    },
    /// Un caractère n'appartient pas à la légende.
    CaractereInconnu {
        ligne: usize,
        colonne: usize,
        caractere: char,
    },
    /// Aucun point de départ `@`.
    DepartManquant,
    /// Plusieurs points de départ.
    DepartMultiple { nombre: usize },
    /// Aucune arrivée `X`.
    ArriveeManquante,
    /// Plusieurs arrivées.
    ArriveeMultiple { nombre: usize },
    /// Un niveau contient exactement 3 gouttes d'or.
    NombreDeGouttes { trouve: usize },
    /// Une plateforme mobile (chiffre 1-9) doit apparaître exactement deux fois
    /// (ses deux extrémités).
    PlateformeMobileIncomplete { marque: char, trouve: usize },
    /// Un objet de décor (`H T R Q K`) doit être posé sur une tuile solide
    /// (`#`, `B` ou `N`) : sinon il flotterait dans le ciel.
    DecorSansAppui {
        ligne: usize,
        colonne: usize,
        caractere: char,
    },
    /// Un pigeon (`p`) doit être perché sur une tuile solide : c'est là qu'il
    /// revient se poser après chaque plongée.
    PigeonSansPerchoir { ligne: usize, colonne: usize },
    /// Un ventilateur (`w`) doit être posé sur une tuile solide : sa casette
    /// (le socle de l'hélice) repose dessus.
    VentilateurSansAppui { ligne: usize, colonne: usize },
}

impl fmt::Display for ErreurNiveau {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErreurNiveau::Vide => write!(f, "le niveau est vide"),
            ErreurNiveau::LargeurInegale {
                ligne,
                attendu,
                trouve,
            } => write!(f, "ligne {ligne} : {trouve} colonnes au lieu de {attendu}"),
            ErreurNiveau::CaractereInconnu {
                ligne,
                colonne,
                caractere,
            } => write!(
                f,
                "ligne {ligne}, colonne {colonne} : caractère inconnu « {caractere} »"
            ),
            ErreurNiveau::DepartManquant => write!(f, "point de départ « @ » manquant"),
            ErreurNiveau::DepartMultiple { nombre } => {
                write!(f, "{nombre} points de départ « @ » (un seul attendu)")
            }
            ErreurNiveau::ArriveeManquante => write!(f, "arrivée « X » manquante"),
            ErreurNiveau::ArriveeMultiple { nombre } => {
                write!(f, "{nombre} arrivées « X » (une seule attendue)")
            }
            ErreurNiveau::NombreDeGouttes { trouve } => {
                write!(f, "{trouve} gouttes d'or « o » au lieu de 3")
            }
            ErreurNiveau::PlateformeMobileIncomplete { marque, trouve } => write!(
                f,
                "plateforme mobile « {marque} » : {trouve} extrémité(s) au lieu de 2"
            ),
            ErreurNiveau::DecorSansAppui {
                ligne,
                colonne,
                caractere,
            } => write!(
                f,
                "ligne {ligne}, colonne {colonne} : le décor « {caractere} » n'est pas posé \
                 sur un solide"
            ),
            ErreurNiveau::PigeonSansPerchoir { ligne, colonne } => write!(
                f,
                "ligne {ligne}, colonne {colonne} : le pigeon « p » n'est pas perché sur un solide"
            ),
            ErreurNiveau::VentilateurSansAppui { ligne, colonne } => write!(
                f,
                "ligne {ligne}, colonne {colonne} : le ventilateur « w » n'est pas posé sur un solide"
            ),
        }
    }
}

impl std::error::Error for ErreurNiveau {}
