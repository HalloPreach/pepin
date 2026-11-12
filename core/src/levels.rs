//! Module `levels` : un petit fichier par niveau, chacun exporte `INFOS`.
//!
//! Les dessins ASCII sont des `&str` embarqués dans le binaire : aucun fichier
//! à charger, donc aucune erreur d'entrée/sortie possible au lancement.

pub mod gouttieres;
pub mod nuages;
pub mod orage;
pub mod toits;
pub mod vents;
