//! Tuiles du PORT (carte 9.6c, préfixe `por_`) : quai de pierre, pontons en
//! planches, caisses, bittes d'amarrage. Choix case par case :
//! `crate::tuiles_theme`.

// `por_pierre.png` : Pierre du quai (le plein) : grands blocs de granit, joints sombres.

// `por_quai.png` : Dessus du quai (où l'on marche) : dalles claires, arête, bordure.

// `por_quai_g.png` : Quai au bord GAUCHE.

// `por_quai_d.png` : Quai au bord DROIT.

// `por_quai_seul.png` : Quai seul (vide des deux côtés).

// `por_anneau.png` : Mur du quai avec un anneau d'amarrage (variante).

// `por_fond.png` : Fond du port (l'eau au-dessus) : pierre noyée, sombre.

// `por_ponton.png` : Ponton (`#` du port sur l'eau) : platelage de planches clouées, chevêtre,
// pieux sombres.

// `por_ponton_g.png` : Ponton au bout GAUCHE.

// `por_ponton_d.png` : Ponton au bout DROIT.

// `por_ponton_seul.png` : Ponton seul (vide des deux côtés).

// `por_caisse.png` : Caisse (un `#` du port posé sur un solide) : caisse à claire-voie, trois
// lattes clouées, l'ombre entre elles.

// `por_bitte.png` : Bitte d'amarrage : posée SUR la case au-dessus d'un bout de quai ou de
// ponton (`crate::tuiles_theme::objet_pose`), sans collision.

// `por_taille.png` : Pierre de taille (carte 9.11b, variante du plein) : un long bloc à bossage en
// haut (arête claire, panneau en relief), la rangée du dessous en joints décalés.

// `por_barbacane.png` : Barbacane (carte 9.11b) : la bouche d'écoulement du quai, ronde et noire, et la
// traînée mouillée qu'elle laisse sur la pierre en dessous.

// `por_algues.png` : Pierre au ras de l'eau (carte 9.11b) : la pierre noircie par la marée, les
// algues violettes qui pendent de la laisse de mer, la mousse dans les joints.

// `por_echelle.png` : Échelle de fer scellée dans le quai (carte 9.11b) : deux montants sombres, un
// barreau toutes les quatre rangées (et son ombre), d'une case à l'autre.

/// Les sprites de ce fichier : (nom, dessin), lus dans `assets/sprites/tuiles_monde2/port/`
/// (carte 10.4a : les dessins sont des fichiers PNG, voir `super::fichiers`).
// Seuls les tests lisent ce thème par son module (le jeu passe par `JEUX`).
#[cfg(test)]
pub fn sprites() -> &'static [(&'static str, crate::sprites::Image)] {
    crate::sprites::fichiers::famille("tuiles_monde2/port")
}
