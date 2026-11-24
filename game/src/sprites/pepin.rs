//! Sprites de Pépin : petite créature ronde avec un grand parapluie corail.
//!
//! 1 caractère = 1 pixel : `.` transparent, `0`-`9` `A`-`F` = indice dans
//! `palette::TOUTES` (voir `sprites/mod.rs`). Pépin fait 16x16 parapluie fermé
//! et 20x20 parapluie ouvert (planer). Boîte de collision : 10x14.
//!
//! Parapluie fermé (colonnes 11 à 15, rangées 0 à 12, le même dans toutes les
//! images où il est fermé, pour qu'il ne tremble pas pendant la course), tenu
//! par la poignée (carte 7.1b : il était à l'envers, crosse en l'air) :
//! - embout de métal (`5`) en haut ;
//! - toile corail repliée (`8`, plis en corail clair `9`) serrée par une sangle
//!   blanche (`7`), fine sous l'embout et plus large en bas, vers la poignée,
//!   comme un vrai parapluie roulé ;
//! - crosse orange (`A`) en bas, à hauteur du flanc, recourbée vers Pépin : la
//!   poignée qu'il tient.
//!
//! Seul le corps de Pépin bouge d'une image à l'autre (respiration, course) : le
//! parapluie reste à sa place, sauf dans `PEPIN_TAPOTE` où il descend d'un pixel
//! (petit coup de crosse vers le sol).

use super::JeuImages;

// --- Attente : après 2 s immobile au sol (voir `scene::image_attente`) ---
// Mêmes pieds et même silhouette que `PEPIN_IDLE0` : on passe de l'une à l'autre
// sans que Pépin saute. Seuls changent les yeux ou la place du parapluie.

// `pepin_cligne.png` : Pépin cligne des yeux : chaque œil (2 px de haut) devient un trait de 2 px.

// `pepin_ciel.png` : Pépin lève les yeux vers le ciel (va-t-il pleuvoir ?) : yeux et joues remontés d'un pixel.

// `pepin_tapote.png` : Pépin tapote le sol avec son parapluie : le parapluie descend d'un pixel et l'embout
// touche le sol (en alternance avec `PEPIN_IDLE0`, ça fait « tac, tac »).

// `pepin_retourne.png` : Parapluie RETOURNÉ (carte 9.2b, BAS tenu), dessiné SOUS Pépin (carte 9.2c, AGA :
// « que le parapluie se mette sous pépin et pas au dessus ») : Pépin debout dans
// la coque, la même que celle de `PEPIN_BARQUE` (bord clair festonné, toile
// corail, baleines, crosse en quille dessous). C'est la barque prête à toucher
// l'eau : posé dessus, Pépin s'y assied (3 px plus bas). Image 16x16, comme la
// barque ; la scène la pose sur les pieds au sol, à la place de la barque en l'air.

// `pepin_barque.png` : Barque (carte 9.2) : le parapluie corail RETOURNÉ devient une coque, et Pépin
// s'assied dedans : la tête (yeux, joues) dépasse du bord, le bord clair (9) est
// festonné comme le bord de la toile, deux baleines (9) descendent vers le fond,
// et la crosse (A) sort dessous comme une quille. Posée comme une image 16x16, le
// coin haut-gauche à (milieu-8, bas-16) : le bas de la coque est à la surface de
// l'eau, la crosse dessous, à hauteur des pieds (3 px sous la surface).

// `vie.png` : Icône des vies dans le HUD (10x10, carte 6.2) : la tête de Pépin, corps blanc,
// yeux, joues corail et ombre de brume, comme le vrai, cernée de sombre pour se lire
// sur tous les ciels.

/// Images de la course (4 images en boucle).
pub const COURSE: [&str; 4] = [
    "pepin_course0",
    "pepin_course1",
    "pepin_course2",
    "pepin_course3",
];
/// Images du repos (respiration).
pub const REPOS: [&str; 2] = ["pepin_idle0", "pepin_idle1"];
/// Images de l'attente (après 2 s immobile) : cligner, regarder le ciel, tapoter le sol.
pub const ATTENTE: [&str; 3] = ["pepin_cligne", "pepin_ciel", "pepin_tapote"];
/// Images du parapluie ouvert (les franges oscillent).
pub const PLANER: [&str; 2] = ["pepin_plane0", "pepin_plane1"];
/// Images de la réapparition (étincelles qui se rassemblent puis Pépin).
pub const REAPPARITION: [&str; 3] = ["pepin_mort0", "pepin_mort1", "pepin_mort2"];

/// Le jeu d'images de Pépin, animation par animation (voir `skins`).
pub const IMAGES: JeuImages = JeuImages {
    repos: REPOS,
    course: COURSE,
    saut: "pepin_saut",
    chute: "pepin_chute",
    attente: ATTENTE,
    planer: PLANER,
    retourne: "pepin_retourne",
    barque: "pepin_barque",
    reapparition: REAPPARITION,
    vie: "vie",
};

/// Les sprites de ce fichier : (nom, dessin), lus dans `assets/sprites/pepin/`
/// (carte 10.4a : les dessins sont des fichiers PNG, voir `super::fichiers`).
pub fn sprites() -> &'static [(&'static str, crate::sprites::Image)] {
    crate::sprites::fichiers::famille("pepin")
}
