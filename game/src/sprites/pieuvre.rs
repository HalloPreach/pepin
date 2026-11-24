//! Sprites de la Pieuvre du port, boss du Monde 2 (carte 9.12a) : sa tête (48x32,
//! en colère, touchée, vaincue), ses tentacules en tronçons (24x8) avec leur bout
//! recourbé (24x16), l'annonce (remous, ombre, bout qui pointe), la clé de la
//! victoire et ses points de vie dans le HUD. Carte 9.13c : la GÉANTE de la montée (sa
//! tête de 64x40, son œil, ses tentacules en tronçons ronds), le repère de la frappe
//! visée et les caisses de la secousse ; carte 9.13d : ses humeurs (tête furieuse, veines, œil
//! injecté, vapeur). Corail et violet de la palette, contour sombre,
//! aucune couleur ajoutée.

// `pieuvre_colere0.png` : Tête de la Pieuvre (48x32), en colère, 1re image : corps corail, reflet corail clair,
// ombre violette en croissant à droite, bras recourbés au pied ; grands yeux blancs sous
// des sourcils froncés, pupilles à gauche ; écume au ras de l'eau (dernière rangée).

// `pieuvre_colere1.png` : En colère, 2e image : les pupilles regardent à droite, l'écume a bougé.

// `pieuvre_touchee.png` : Touchée : yeux plissés en chevrons, bouche ouverte, joues marquées d'or pâle.

// `pieuvre_vaincue.png` : Vaincue : yeux en croix, bouche ondulée (elle replonge avec cette image).

// `tentacule_bout.png` : Bout du tentacule (24x16) : la tige s'affine et se recourbe en crosse ; son pied se
// raccorde au tronçon (`tentacule0`).

// `tentacule0.png` : Tronçon de tentacule (24x8), répété de la surface jusqu'au bout : corps corail, reflet
// à gauche, ombre violette à droite, deux ventouses d'or pâle. 1re image.

// `tentacule1.png` : Tronçon, 2e image : les ventouses ont glissé (le tentacule ondule).

// `tentacule_pointe0.png` : Annonce, 2e moitié (24x12) : le bout du tentacule pointe hors de l'eau, dans l'écume.

// `tentacule_pointe1.png` : Le bout qui pointe, 2e image (l'écume bouge).

// `remous_pieuvre0.png` : Remous sur l'eau (32x6) : annonce d'un tentacule (1re moitié) et de la tête qui
// remonte. 1re image.

// `remous_pieuvre1.png` : Remous, 2e image.

// `ombre_pieuvre.png` : L'ombre de la pieuvre sous la surface (32x4), avec les remous.

// `cle_pieuvre.png` : La clé que la Pieuvre vaincue lâche en replongeant (14x8), or.

// `pv_pieuvre.png` : Point de vie de la Pieuvre dans le HUD (10x6) : une petite pieuvre corail.

// `pv_pieuvre_perdu.png` : Point de vie perdu (10x6) : la même, éteinte (ardoise).

// `tentacule_enroule0.png` : Tentacule ENROULÉ sur le dessus d'une plateforme saisie (16x6, carte 9.12f),
// posé case par case : corps corail, reflet corail clair, ventouses d'or pâle,
// ombre violette dessous. 1re image.

// `tentacule_enroule1.png` : Tentacule enroulé, 2e image : les ventouses ont glissé (il serre).

// `bulle_saisie.png` : Une bulle qui remonte sous une plateforme saisie (4x4, carte 9.12g) : contour
// sombre, reflet blanc cassé, cyan. Elle crève à la surface.

// `onde_saisie.png` : Bout gauche du cercle de remous de la saisie (4x5, carte 9.12g ; le bout droit
// est son miroir) : l'anneau d'écume posé à plat sur la surface, la 3e rangée au
// ras de l'eau. Entre les deux bouts, deux traits d'écume (`dessiner_saisie`).

// `bord_creux_saisie.png` : Lèvre d'écume au bord gauche du creux (6x4, carte 9.12g ; le bord droit est son
// miroir) : l'eau bourrelée qui tourne, contour sombre au dehors, la dernière
// rangée au ras de la surface. Pas de gerbe : 3 px au-dessus de l'eau, pas plus.

// `geante0.png` : La tête de la Pieuvre GÉANTE de la montée (64x40, carte 9.13c), 1re image : le manteau
// corail (reflet corail clair en haut à gauche, ombre violette en croissant à droite, taches
// claires), le grand œil blanc cerné sous un sourcil froncé (la pupille est posée à part :
// elle suit Pépin), et cinq racines de tentacules au ras de l'eau, ventouses d'or pâle.

// `geante1.png` : La tête de la géante, 2e image : les racines ondulent d'un pixel.

// `paupiere_geante.png` : La paupière de la géante (18x14, posée sur l'œil) : elle PLISSE l'œil pendant
// l'annonce d'une attaque (le haut de l'œil fermé, bord sombre).

// `oeil_sonne.png` : L'œil SONNÉ de la géante (18x14, posé sur l'œil) : une spirale sombre sur le blanc.

// `pupille_geante.png` : La pupille de la géante (6x6) : elle suit Pépin dans le blanc de l'œil.

// `pupille_rouge.png` : La pupille ROUGE (8x8) : l'œil rougit pendant l'annonce d'une attaque.

// `bouche_geante.png` : La bouche de la géante qui HURLE (12x8), quand on la frappe ou qu'elle se jette.

// `bras_gros.png` : Un tronçon rond de tentacule de la géante, près du corps (10x10) : corail, reflet,
// ombre violette, ventouse d'or pâle. Les tentacules sont des chapelets de tronçons, du
// corps jusqu'au bout, de plus en plus fins (`dessin_montee`).

// `bras_moyen.png` : Tronçon du milieu (8x8).

// `bras_fin.png` : Tronçon fin, vers le bout (6x6).

// `bras_pointe.png` : Le bout du tentacule (4x4).

// `repere_frappe0.png` : Le repère ROUGE de la frappe visée (16x16) : quatre coins et un centre corail,
// cernés de sombre, sur la place visée. 1re image.

// `repere_frappe1.png` : Le repère, 2e image : les coins se resserrent (il palpite).

// `caisse_chute.png` : Une caisse qui tombe du haut de la tour pendant une secousse (16x16) : planches
// orange, croisillon d'or, contour sombre.

// `ombre_caisse.png` : L'ombre d'une caisse qui va tomber (16x4), posée sur le sol où elle se brisera.

// `eclats_caisse.png` : Les éclats d'une caisse brisée (16x6).

// `geante_furieuse0.png` : La géante FURIEUSE (carte 9.13d), 1re image : le reflet corail clair vire à
// l'orange et des taches orange percent le manteau (elle chauffe).

// `geante_furieuse1.png` : La géante furieuse, 2e image.

// `veines_geante.png` : Les veines gonflées de son front, violettes (énervée et furieuse), posées sur la tête.

// `oeil_injecte1.png` : L'œil injecté d'une géante énervée (18x14, posé comme la paupière) : quatre
// vaisseaux corail partent du bord du blanc.

// `oeil_injecte2.png` : L'œil injecté d'une géante furieuse : neuf vaisseaux.

// `pupille_enervee.png` : La pupille d'une géante énervée : cerclée d'orange (rouge : furieuse, ou une annonce).

// `vapeur_geante0.png` : La vapeur qui fume de sa tête quand elle est furieuse, 1re image (10x8).

// `vapeur_geante1.png` : La vapeur, 2e image.

/// Les sprites de ce fichier : (nom, dessin), lus dans `assets/sprites/pieuvre/`
/// (carte 10.4a : les dessins sont des fichiers PNG, voir `super::fichiers`).
pub fn sprites() -> &'static [(&'static str, crate::sprites::Image)] {
    crate::sprites::fichiers::famille("pieuvre")
}
