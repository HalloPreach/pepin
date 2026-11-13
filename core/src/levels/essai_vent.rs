//! Niveau d'essai « vent » : un écran plat qui met en scène le vent latéral
//! (carte 5.3 du plan de capture). Hors de la table `NIVEAUX`, il n'existe QUE
//! dans les tests et dans le build de QA (fonctionnalité `debug_niveaux`,
//! touche F1) ; en production il n'est pas compilé dans le binaire, comme le
//! niveau d'essai vertical.
//!
//! Le `>` au-dessus du vide (lignes 3 à 5) : sans vent, le saut est trop court
//! pour traverser les 12 tuiles du vide ; en planant avec le vent, la planée
//! est prolongée et le vide devient franchissable. Le contre-vent (`<`) est,
//! lui, démontré par les mini-niveaux du robot (`faisabilite`).
//! Les trois gouttes sont posées au sol sur le chemin du robot.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 0,
    nom: "Essai vent",
    theme: Theme::Soleil,
    genre: GenreNiveau::Normal,
    intro: "Le vent latéral vous pousse de 60 px par seconde. Parapluie ouvert, il devient deux fois plus fort : 120 px par seconde.",
    mecanique: "Planer avec le vent pour prolonger la traversée du vide. Parapluie fermé pour le résister.",
    duree_cible_s: 30,
    ascii: ASCII,
    aiguilles: &[],
};

/// Écran plat (36 x 11 tuiles), thème Soleil.
/// - Vide de 12 tuiles (colonnes 12 à 23), sans plancher : le saut seul est
///   trop court ; le vent `>` au-dessus (lignes 3 à 5) prolonge la planée et
///   rend la traversée possible. Sans vent, le niveau est bloqué.
/// - Les trois gouttes sont au sol (ligne 7) sur le chemin du robot : une avant
///   le vide, deux sur l'approche du drapeau. Les lanternes sont aux colonnes
///   9, 26 et 30.
const ASCII: &str = r"
....................................
....................................
....................................
............>>>>>>>>>>>>............
............>>>>>>>>>>>>............
............>>>>>>>>>>>>............
....................................
..@o.....L................L...L.ooX.
############............############
############............############
############............############
";
