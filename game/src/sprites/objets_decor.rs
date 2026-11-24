//! Sprites des objets de décor (carte 4.2) : cheminée, antenne, paratonnerre,
//! manche à air, oiseau posé ; et ceux de la rue du Monde 2 (carte 9.6b) : bouche
//! d'égout, borne, poubelle. Aucun n'a de collision : ce
//! sont des dessins posés sur le toit ou la rue (leur dernière rangée touche la
//! tuile du dessous), d'une tuile de large au plus et de moins de deux tuiles de
//! haut. (La descente de gouttière de la rue n'est plus un sprite depuis la carte
//! 9.20h : `decor_rue` la dessine accrochée à sa maison.)
//!
//! Ils sont dessinés tels quels (sans la substitution de couleurs des tuiles) : chacun
//! ne sert que dans les niveaux de son thème (voir `level::GenreDecor`). Couleurs
//! réservées au jeu, donc absentes d'ici : l'or des gouttes (B, C), le cyan des
//! flaques et de la pluie (D, E), le corail du parapluie de Pépin (8, 9).

// `cheminee.png` : Souche de cheminée : chapeau de pierre, deux mitrons, corps en brique violette
// comme les murs, ombre sur le côté droit (12x20).

// `antenne.png` : Antenne de télévision : mât et trois barres, de plus en plus larges vers le bas (15x24).

// `paratonnerre.png` : Paratonnerre : pointe claire, boule, mât et pied (7x24).

// `manche_a_air.png` : Manche à air rayée orange et blanc, gonflée vers la droite (16x22). Dans le jeu,
// elle s'étire vers le courant ascendant qu'elle signale (retournée s'il est à gauche).

// `oiseau0.png` : Oiseau posé, petit et blanc, regard vers la droite (12x8).

// `oiseau1.png` : Oiseau posé : il tourne la tête (bec vers la gauche), même silhouette.

// `bouche_egout.png` : Bouche d'égout du caniveau (carte 9.6b) : la pierre de la bordure, la bouche
// sombre et sa grille de fonte (16x9).

// `borne.png` : Borne de trottoir en fonte : tête ronde, bague, fût, pied évasé (6x14). Carte
// 9.20g : du décor sans collision, donc terne (ardoise sombre, aucune tête claire).

// `poubelle.png` : Poubelle de rue : couvercle et sa poignée, cuve cannelée (12x14). Carte 9.20g :
// du décor sans collision, donc terne (plus de couvercle clair : elle ne se
// confond plus avec la poubelle SOLIDE du mobilier de rue, `mobilier_rue`).

/// Oiseau posé (2 images) : regard devant, puis tête tournée.
pub const OISEAU: [&str; 2] = ["oiseau0", "oiseau1"];

/// Les sprites de ce fichier : (nom, dessin), lus dans `assets/sprites/objets_decor/`
/// (carte 10.4a : les dessins sont des fichiers PNG, voir `super::fichiers`).
pub fn sprites() -> &'static [(&'static str, crate::sprites::Image)] {
    crate::sprites::fichiers::famille("objets_decor")
}
