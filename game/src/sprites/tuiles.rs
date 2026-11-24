//! Tuiles de décor 16x16 : toits d'ardoise, briques de cheminée, nuages.
//! Chaque matière a une version « haut » (arête éclairée) et une version « plein ».
//! La brique pleine a en plus trois variantes (fissure, brique manquante, coulure), et une bande
//! d'ombre se pose sous le rebord des toits.

// `nuage_bord_g.png` : Bords de plateforme de nuage (arête éclairée) : coins arrondis, choisis quand la case voisine est vide.

// `nuage_ilot.png` : Nuage d'une seule case (voisins vides des deux côtés).

// --- Variantes de brique pleine ---
// Même taille (16 x 16) et mêmes couleurs que `BRIQUE_PLEIN` : seule une petite zone change, le
// reste de l'appareil (joints, assises) est recopié à l'identique pour que les raccords avec les
// tuiles voisines restent parfaits. La scène en pose environ une tuile sur six, à des places
// fixes (voir `variante_brique` dans `scene.rs`).

// `brique_fissure.png` : Fissure : un trait sombre (1 = ombre de nuit) qui descend en biais à travers deux assises.

// `brique_manquante.png` : Brique manquante : le trou laisse voir l'ombre du mur (0 en haut : ombre portée par la brique
// du dessus ; 1 au fond).

// `brique_coulure.png` : Coulure : l'eau de pluie suinte d'un joint et laisse une traînée humide plus sombre (2 =
// ardoise, comme les joints) qui s'affine en descendant et finit par une goutte (1). Pas de cyan :
// le cyan reste réservé à la pluie et aux flaques, qu'on ne doit pas confondre avec le mur.

// `brique_decalee.png` : Indice d'un faux mur (carte 8.5) : la brique du milieu a glissé d'un pixel vers la droite ; le
// jour qu'elle laisse à gauche montre l'ombre derrière le mur (0 en haut, 1 dessous). Le reste est
// la brique pleine : seul un œil attentif voit le joint décalé.

// `brique_saillie_g.png` : Indice d'un faux mur de brique, en plus de `BRIQUE_DECALEE` : le bout d'une brique qui dépasse
// de 3 px de la façade, posé sur la case libre VOISINE (à gauche de la façade : `_G` ; à droite :
// `_D`). Les variantes de brique tirées au hasard restent dans leur case : seul l'indice sort du
// mur.

// `brique_saillie_d.png` : Même chose, la façade à droite de la case libre : le bout de brique sort vers la gauche.

// `nuage_creux.png` : Indice d'un faux mur de nuage (carte 8.5) : un petit creux d'ombre (5) dans le nuage plein,
// comme si le nuage était moins épais à cet endroit.

// `ombre_toit.png` : Ombre sous le rebord d'un toit : bande de 2 px posée PAR-DESSUS le haut de la brique qui est
// juste sous une tuile de toit. 0 (ombre la plus profonde) contre le rebord, puis 1 (ombre de
// nuit) : deux niveaux d'ombre, sans dégradé. Elle donne l'épaisseur de l'avancée du toit.

// --- Eau (carte 9.2, redessinée à la carte 9.6b) ---
// Trois bandes nettes, sans dégradé : la LIGNE de surface claire (D, 1 px), la BANDE
// sous la surface (E, 6 px, quelques reflets D) et le FOND plus sombre (2, l'ardoise,
// quelques traits E) : l'eau tranche sur le sol, même quand il est sombre. Chaque
// matière d'eau a trois dessins : « haut » (le dessus de l'étendue), « vague » (le
// même, 1 px plus bas : `nom_tuile` alterne les deux pour que la surface ondule d'un
// pixel) et « plein » (l'intérieur). Le SENS d'un courant ne se dessine plus dans la
// tuile (les anciens chevrons « > » se lisaient comme du texte) : de courtes stries
// défilent par-dessus, dans le sens du courant (`crate::dessin_courant`).

// `eau_calme_haut.png` : Eau, dessus de l'étendue : la ligne claire, la bande, le fond.

// `eau_calme_vague.png` : Eau, dessus, l'autre temps de la vague : la même surface 1 px plus bas.

// `eau_calme_plein.png` : Eau, intérieur de l'étendue : le fond sombre, quelques traits de reflet.

// Les courants ont les mêmes dessins que l'eau calme (les stries animées disent le
// sens) ; leurs noms restent, pour que le choix de `nom_tuile` ne change pas.
// `eau_courant_g_haut` (alias de `eau_calme_haut.png`) : Courant vers la gauche (`{`), dessus.
// `eau_courant_g_vague` (alias de `eau_calme_vague.png`) : Courant vers la gauche, dessus, l'autre temps de la vague.
// `eau_courant_g_plein` (alias de `eau_calme_plein.png`) : Courant vers la gauche, intérieur.
// `eau_courant_d_haut` (alias de `eau_calme_haut.png`) : Courant vers la droite (`}`), dessus.
// `eau_courant_d_vague` (alias de `eau_calme_vague.png`) : Courant vers la droite, dessus, l'autre temps de la vague.
// `eau_courant_d_plein` (alias de `eau_calme_plein.png`) : Courant vers la droite, intérieur.

/// Les sprites de ce fichier : (nom, dessin), lus dans `assets/sprites/tuiles/`
/// (carte 10.4a : les dessins sont des fichiers PNG, voir `super::fichiers`).
pub fn sprites() -> &'static [(&'static str, crate::sprites::Image)] {
    crate::sprites::fichiers::famille("tuiles")
}
