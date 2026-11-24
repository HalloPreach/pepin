//! Tuiles du MONDE 2, par thème (carte 9.6b) : chaque thème du Monde 2 peut avoir
//! son propre jeu de tuiles 16x16 au lieu des toits et des briques du Monde 1. Le
//! choix du dessin, case par case, est dans `crate::tuiles_theme` ; ici, seulement
//! les dessins.
//!
//! Ces tuiles sont dessinées TELLES QUELLES (aucune substitution de couleurs de
//! thème, voir `remplacer`) : chaque jeu sert à un seul thème, ses couleurs sont
//! choisies pour lui. Règles de la palette : contour sombre, deux niveaux d'ombre
//! au plus, aucun dégradé ; le cyan (D, E) reste à l'eau et à la pluie, l'or (B, C)
//! aux gouttes.
//!
//! Carte 9.6c : les jeux des égouts, de la cascade, de la grotte et du port sont
//! dans les sous-modules du même nom (carte 9.10 : la rivière) ; `JEUX` les
//! rassemble tous.

pub mod cascade;
pub mod egouts;
pub mod grotte;
pub mod port;
pub mod riviere;

/// Tous les jeux de tuiles du Monde 2, un par thème (le caniveau, puis carte 9.6c).
pub const JEUX: [&str; 6] = [
    "tuiles_monde2/caniveau",
    "tuiles_monde2/egouts",
    "tuiles_monde2/cascade",
    "tuiles_monde2/grotte",
    "tuiles_monde2/port",
    "tuiles_monde2/riviere",
];

// --- Le caniveau ---------------------------------------------------------------
// La rue sous l'averse : des pavés gris-bleu en surface, la bordure du trottoir au
// bord de l'eau, une terre sombre dessous ; au-dessus de la rue, la maison de la
// gouttière (pierre, fenêtres, chéneau) et l'avant-toit (corniche et store).

// `can_pave.png` : Pavés de la rue (la surface où l'on marche) : contour sombre, arête claire, trois
// rangées de pavés décalées (reflet en haut à gauche, ombre en bas à droite), joints
// sombres, puis le lit de sable. Carte 9.20g : l'arête est le LISERÉ CLAIR (nuage)
// de tout dessus où l'on se pose, comme la bordure et le mobilier de rue.

// `can_pave_g.png` : Pavés au bord GAUCHE de la rue (un creux ou de l'eau à gauche) : la bordure de
// trottoir, une pierre claire en haut et une arête nette (le contour de la colonne 0).

// `can_pave_d.png` : Pavés au bord DROIT de la rue : la même bordure, à droite.

// `can_terre.png` : Sous la rue : une terre sombre et calme, presque sans détail (aucun cyan), pour que
// l'eau et les plateformes ressortent.

// `can_terre_pierre.png` : Variante de la terre : un moellon enfoui (environ une tuile sur cinq).

// `can_rigole.png` : Le fond du caniveau à sec : des dalles de granit, puis la terre.

// `can_bordure.png` : Pierre de taille claire, seule (vide des deux côtés) : la bordure haute du
// trottoir, un rebord de la gouttière. Un bloc cerné, éclairé par le haut.

// `can_bordure_g.png` : Pierre de taille au bout GAUCHE d'une rangée : contour à gauche, joint d'ardoise
// à droite (marches, bordure haute, corniche de l'avant-toit).

// `can_bordure_m.png` : Pierre de taille au milieu d'une rangée : un joint d'ardoise à droite.

// `can_bordure_d.png` : Pierre de taille au bout DROIT d'une rangée : contour à droite.

// `can_soubassement.png` : Soubassement d'une maison ou d'une bordure, au niveau de la rue : gros blocs de
// granit, joints marqués.

// `can_facade.png` : Façade de la maison (au-dessus de la rue) : un enduit de pierre, deux assises.

// `can_fenetre.png` : Façade avec une fenêtre allumée : encadrement sombre, croisée, vitres orange, appui.

// `can_fenetre_noire.png` : Façade avec une fenêtre éteinte (vitres sombres, reflets d'ardoise).

// `can_plaque.png` : Façade avec la plaque de la rue : bleu ardoise cerclé de blanc cassé (le
// « panneau » de la rue, sans lettre de grille : il ne se confond pas avec les
// panneaux d'aide D, S, G, C, W).

// `can_cheneau.png` : Chéneau : sous l'eau de la gouttière, le fond de zinc qui la porte, puis la
// corniche et ses denticules, puis la façade.

// `can_store_g.png` : Store de boutique (le dessous de l'avant-toit, au-dessus du vide), bout GAUCHE :
// toile rayée blanc cassé et orange, cernée de sombre, lambrequin festonné.

// `can_store_m.png` : Store de boutique, milieu de la toile.

// `can_store_d.png` : Store de boutique, bout DROIT.

// --- La bouche d'égout (carte 9.20a) ---------------------------------------------
// La sortie du caniveau : la bordure du trottoir qui arrête le dernier courant (deux
// tuiles de fond) est ouverte. Trois cases l'une sous l'autre : la bordure et sa
// grille (le drapeau y est planté), la gueule où l'eau de surface s'engouffre, la
// gorge où l'eau du fond tombe dans le noir. L'eau garde ses couleurs (bande cyan,
// fond ardoise) jusque dans la gueule.

// `can_egout_grille.png` : La bordure au-dessus de la bouche : sur le dessus, la grille (fentes sombres) ;
// dessous, le linteau puis le haut de la gueule, noir.

// `can_egout_bouche.png` : La gueule de la bouche, à la surface de l'eau : la bande d'eau entre par la
// gauche et plonge dans le noir ; à droite, le granit de la bordure.

// `can_egout_gorge.png` : La gorge, sous la gueule : l'eau du fond se dérobe en biais et tombe dans le noir.

/// Les sprites de ce fichier : (nom, dessin), lus dans `assets/sprites/tuiles_monde2/caniveau/`
/// (carte 10.4a : les dessins sont des fichiers PNG, voir `super::fichiers`).
// Seuls les tests lisent ce thème par son module (le jeu passe par `JEUX`).
#[cfg(test)]
pub fn sprites() -> &'static [(&'static str, crate::sprites::Image)] {
    crate::sprites::fichiers::famille("tuiles_monde2/caniveau")
}
