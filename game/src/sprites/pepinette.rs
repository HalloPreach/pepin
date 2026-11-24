//! Sprites de Pépinette : la version fille de Pépin (carte 7.1), premier skin ajouté.
//!
//! Même format que `pepin.rs` (1 caractère = 1 pixel, `.` transparent, `0`-`F` =
//! indice dans `palette::TOUTES`) et MÊMES images, une à une : mêmes tailles (16x16,
//! 20x20 parapluie ouvert, 10x10 pour l'icône des vies), même silhouette, mêmes pieds,
//! même parapluie au même endroit. Le jeu reste donc identique : seule la tenue change.
//!
//! Ce qui la distingue de Pépin, dans chaque image :
//! - un nœud corail (`8`) posé à gauche sur le haut de la tête ;
//! - des cils : un pixel sombre en haut, à l'extérieur de chaque œil ;
//! - une petite robe corail : les rangées d'ombre (`5`) sous le corps blanc ;
//! - un parapluie cyan de la pluie (`D`, plis `6`) au lieu du corail : on la
//!   reconnaît de loin. Les joues corail clair (`9`) restent.
//!
//! Les étincelles de la réapparition (`pepinette_mort2`) sont celles de Pépin.

use super::JeuImages;

// `pepinette_mort2` (pas de fichier : alias de `pepin_mort2.png`, voir `fichiers::ALIAS`) : Les étincelles de la réapparition : les mêmes que pour Pépin.

// `vie_pepinette.png` : Icône des vies du HUD quand Pépinette est active : sa tête, nœud et cils compris.

// `pepinette_retourne.png` : Parapluie RETOURNÉ de Pépinette (carte 9.2b), SOUS elle (carte 9.2c) : comme
// celui de Pépin, debout dans la coque de `PEPINETTE_BARQUE` (toile cyan, bord et
// baleines en 6, crosse dessous), son nœud corail et sa robe au-dessus. 16x16.

// `pepinette_barque.png` : Barque de Pépinette (carte 9.2) : la même coque que celle de Pépin, dans la toile
// cyan de son parapluie (D, bord et baleines en 6), son nœud corail (8) sur la tête.
// Comme la barque de Pépin, la crosse (A) pointe vers le bas.

/// Le jeu d'images de Pépinette, animation par animation (voir `skins`).
pub const IMAGES: JeuImages = JeuImages {
    repos: ["pepinette_idle0", "pepinette_idle1"],
    course: [
        "pepinette_course0",
        "pepinette_course1",
        "pepinette_course2",
        "pepinette_course3",
    ],
    saut: "pepinette_saut",
    chute: "pepinette_chute",
    attente: ["pepinette_cligne", "pepinette_ciel", "pepinette_tapote"],
    planer: ["pepinette_plane0", "pepinette_plane1"],
    retourne: "pepinette_retourne",
    barque: "pepinette_barque",
    suspendu: "pepinette_suspendu",
    reapparition: ["pepinette_mort0", "pepinette_mort1", "pepinette_mort2"],
    vie: "vie_pepinette",
};

/// Les sprites de ce fichier : (nom, dessin), lus dans `assets/sprites/pepinette/`
/// (carte 10.4a : les dessins sont des fichiers PNG, voir `super::fichiers`).
pub fn sprites() -> &'static [(&'static str, crate::sprites::Image)] {
    crate::sprites::fichiers::famille("pepinette")
}
