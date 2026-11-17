//! Tuiles PAR THÈME (carte 9.6b) : un thème du Monde 2 peut remplacer les toits et
//! les briques du Monde 1 par son propre jeu de tuiles (`sprites::tuiles_monde2`).
//!
//! Le principe : `scene::nom_tuile` demande d'abord ici le nom de la tuile à poser.
//! `None` (tous les thèmes du Monde 1, l'eau, et tout thème sans jeu à lui) : la scène
//! garde son choix habituel, au pixel près. Pour un thème qui a son jeu, une fonction
//! (ici `caniveau`) choisit le dessin d'après le VOISINAGE de la case : ce qu'il y a
//! au-dessus, dessous, à gauche et à droite, et la hauteur de la case par rapport à la
//! rue. Rendu seulement : la collision reste celle de la grille.
//!
//! Ajouter le jeu d'un thème : ses dessins dans `sprites/tuiles_monde2.rs` (préfixe
//! du thème), une fonction de choix ici, et une ligne dans `nom_du_theme`.

use pepin_core::aabb::TAILLE_TUILE;
use pepin_core::level::Tuile;
use pepin_core::physics::JOUEUR_H;
use pepin_core::zones::Theme;

/// Ce que la scène sait d'une case solide et de ses voisines (tuiles VUES : un faux
/// mur compte comme le solide qu'il imite).
struct Voisinage {
    /// Colonne et rangée de la case.
    tx: i32,
    ty: i32,
    /// Les tuiles voisines : au-dessus, en dessous, à gauche, à droite.
    dessus: Tuile,
    dessous: Tuile,
    gauche: Tuile,
    droite: Tuile,
    /// Rangée de la RUE : celle du sol sous le point de départ de Pépin.
    rue: i32,
    /// Hachage fixe de la case (`scene::hachage_case`) : les variantes d'un dessin
    /// tombent toujours au même endroit.
    hachage: u32,
}

/// Le thème a-t-il son propre jeu de tuiles ? (La bande d'ombre sous les toits du
/// Monde 1 ne s'y pose pas.)
pub(crate) fn a_son_jeu(theme: Theme) -> bool {
    matches!(theme, Theme::Caniveau)
}

/// La rangée de la rue : celle de la case solide sous les pieds de Pépin au départ
/// (`depart` : coin haut-gauche de sa boîte, en pixels).
fn rangee_de_la_rue(depart: (f32, f32)) -> i32 {
    ((depart.1 + JOUEUR_H) / TAILLE_TUILE).floor() as i32
}

/// Nom de la tuile du jeu du thème pour la case solide `(tx, ty)`, ou `None` (dessin
/// du Monde 1). `vue` : la tuile vue d'une case ; `depart` : le départ du niveau ;
/// `hachage` : le hachage fixe de la case.
pub(crate) fn nom_du_theme(
    theme: Theme,
    (tx, ty, tuile): (i32, i32, Tuile),
    vue: impl Fn(i32, i32) -> Tuile,
    depart: (f32, f32),
    hachage: u32,
) -> Option<&'static str> {
    let choix: fn(&Voisinage) -> &'static str = match theme {
        Theme::Caniveau => caniveau,
        _ => return None,
    };
    if !tuile.est_solide() {
        return None;
    }
    Some(choix(&Voisinage {
        tx,
        ty,
        dessus: vue(tx, ty - 1),
        dessous: vue(tx, ty + 1),
        gauche: vue(tx - 1, ty),
        droite: vue(tx + 1, ty),
        rue: rangee_de_la_rue(depart),
        hachage,
    }))
}

/// Une case est « ouverte » de ce côté si rien de solide n'y est (du vide ou de l'eau).
fn ouvert(t: Tuile) -> bool {
    !t.est_solide()
}

/// Le caniveau : la rue, la terre dessous, et ce qui est bâti au-dessus de la rue.
fn caniveau(v: &Voisinage) -> &'static str {
    let a_l_air = v.dessus == Tuile::Vide;
    if v.ty > v.rue {
        // Sous la rue : le fond du caniveau à sec (rien au-dessus), sinon la terre.
        return if a_l_air {
            "can_rigole"
        } else if v.hachage.is_multiple_of(5) {
            "can_terre_pierre"
        } else {
            "can_terre"
        };
    }
    if v.ty == v.rue {
        // La rue : des pavés, la bordure du trottoir au bord d'un creux ou de
        // l'eau ; sous une construction, son soubassement de granit.
        return match (a_l_air, ouvert(v.gauche), ouvert(v.droite)) {
            (false, _, _) => "can_soubassement",
            (true, true, true) => "can_bordure",
            (true, true, false) => "can_pave_g",
            (true, false, true) => "can_pave_d",
            (true, false, false) => "can_pave",
        };
    }
    // Au-dessus de la rue : la maison, ses marches, l'avant-toit. Une rangée de
    // pierres de taille a un contour à ses deux bouts seulement ; sous une pierre,
    // au-dessus du vide, le store d'une boutique.
    let (g, d) = (ouvert(v.gauche), ouvert(v.droite));
    if a_l_air {
        return match (g, d) {
            (true, true) => "can_bordure",
            (true, false) => "can_bordure_g",
            (false, true) => "can_bordure_d",
            (false, false) => "can_bordure_m",
        };
    }
    if v.dessous == Tuile::Vide {
        return match (g, d) {
            (true, _) => "can_store_g",
            (false, true) => "can_store_d",
            (false, false) => "can_store_m",
        };
    }
    if v.dessus == Tuile::Eau {
        return "can_cheneau";
    }
    // Le mur : une fenêtre une colonne sur trois (jamais au bord d'une eau ni d'un
    // vide), allumée ou non ; la plaque de la rue, rare, sur le mur plein.
    let au_bord = g || d;
    if v.tx.rem_euclid(3) == 1 && !au_bord {
        return if v.hachage.is_multiple_of(3) {
            "can_fenetre_noire"
        } else {
            "can_fenetre"
        };
    }
    if !au_bord && v.hachage.is_multiple_of(7) {
        return "can_plaque";
    }
    "can_facade"
}
