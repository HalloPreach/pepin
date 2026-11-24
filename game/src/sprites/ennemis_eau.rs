//! Sprites des ennemis d'eau du Monde 2 (carte 9.4) : grenouille (16x12),
//! poisson (12x16) et ses remous (16x6), crabe (18x12), rat (18x14) et
//! mouette (18x14, le pigeon du port : mêmes images que le pigeon, en blanc et
//! gris, bec jaune, pattes orange). Le crabe et la grenouille regardent à
//! gauche, comme le chat ; le rat et la mouette à droite, comme le pigeon.

// `grenouille0.png` : Grenouille (16x12, tournée vers la gauche) qui guette : corps cyan sombre,
// reflets cyan, ventre or pâle, oeil blanc au-dessus de la tête. 1re image.

// `grenouille1.png` : Grenouille qui guette, 2e image : la gorge se gonfle (le ventre avance d'un pixel).

// `grenouille_annonce.png` : Annonce de la grenouille : elle se ramasse (2 px plus bas, plus large) avant
// de bondir.

// `grenouille_saut.png` : Bond de la grenouille : pattes avant tendues, pattes arrière allongées.

// `grenouille_ecrasee.png` : Grenouille écrasée : aplatie, les yeux encore visibles.

// `poisson0.png` : Poisson (12x16) qui monte hors de l'eau, la tête en haut : orange, reflets
// or, ventre or pâle, nageoires corail. 1re image.

// `poisson1.png` : Poisson qui monte, 2e image : la queue bat d'un pixel.

// `poisson_chute.png` : Poisson qui retombe, la tête en bas (la 1re image retournée haut-bas).

// `remous0.png` : Remous (16x3) à la surface, au-dessus du poisson caché : l'annonce de son saut.
// 1re image, petits cercles.

// `remous1.png` : Remous, 2e image (fin de l'annonce) : les cercles s'élargissent.

// `crabe0.png` : Crabe (18x12, tourné vers la gauche) : carapace corail, une pince LEVÉE (la
// carapace dure), yeux sur pédoncules. Marche, 1re image.

// `crabe1.png` : Crabe, marche, 2e image : les pattes alternent.

// `crabe_alerte.png` : Crabe en alerte (Pépin au-dessus de lui) : les deux pinces levées.

// `crabe_claque.png` : Crabe qui claque des pinces, juste après un rebond de Pépin sur sa carapace.

// `rat0.png` : Rat (18x14, tourné vers la droite) qui patrouille : gris, ventre brume,
// oreille, museau et queue corail clair. 1re image.

// `rat1.png` : Rat qui patrouille, 2e image : les pattes et la queue bougent.

// `rat_annonce.png` : Annonce du rat : il se dresse sur ses pattes arrière avant de foncer.

// `rat_fonce0.png` : Rat qui fonce : allongé, oreille couchée, traits de vitesse. 1re image.

// `rat_fonce1.png` : Rat qui fonce, 2e image.

// `rat_etourdi.png` : Rat étourdi contre le mur : étoiles d'or, yeux en croix.

// `rat_ecrase.png` : Rat écrasé : aplati.

// `mouette0.png` : Mouette : image0 du pigeon, recolorée.

// `mouette1.png` : Mouette : image1 du pigeon, recolorée.

// `mouette_annonce.png` : Mouette : image_annonce du pigeon, recolorée.

// `mouette_plongee0.png` : Mouette : image_plongee0 du pigeon, recolorée.

// `mouette_plongee1.png` : Mouette : image_plongee1 du pigeon, recolorée.

// `mouette_remontee0.png` : Mouette : image_remontee0 du pigeon, recolorée.

// `mouette_remontee1.png` : Mouette : image_remontee1 du pigeon, recolorée.

// `mouette_ecrasee.png` : Mouette : image_ecrasee du pigeon, recolorée.

/// Les sprites de ce fichier : (nom, dessin), lus dans `assets/sprites/ennemis_eau/`
/// (carte 10.4a : les dessins sont des fichiers PNG, voir `super::fichiers`).
pub fn sprites() -> &'static [(&'static str, crate::sprites::Image)] {
    crate::sprites::fichiers::famille("ennemis_eau")
}
