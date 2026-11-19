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

use pepin_core::level::Tuile;
use pepin_core::zones::Theme;

/// Ce que la scène sait d'une case solide et de ses voisines (tuiles VUES : un faux
/// mur compte comme le solide qu'il imite, le mobilier de rue comme du vide).
struct Voisinage {
    /// Colonne et rangée de la case.
    tx: i32,
    ty: i32,
    /// La tuile de la case (le port distingue le bois `#` de la pierre `B`, le
    /// caniveau le sol de la rue `B` de ce qui est bâti `#`).
    tuile: Tuile,
    /// Les tuiles voisines : au-dessus, en dessous, à gauche, à droite.
    dessus: Tuile,
    dessous: Tuile,
    gauche: Tuile,
    droite: Tuile,
    /// Hachage fixe de la case (`scene::hachage_case`) : les variantes d'un dessin
    /// tombent toujours au même endroit.
    hachage: u32,
    /// Le port (carte 9.11b) : la case est-elle à la hauteur de l'eau (de l'eau sur
    /// sa rangée, à moins de `PORTEE_EAU` cases) ? Toujours faux ailleurs.
    rangee_d_eau: bool,
}

/// Distance (cases) à laquelle le port cherche l'eau sur la rangée d'une case.
const PORTEE_EAU: i32 = 24;

/// Le thème a-t-il son propre jeu de tuiles ? (La bande d'ombre sous les toits du
/// Monde 1 ne s'y pose pas.)
pub(crate) fn a_son_jeu(theme: Theme) -> bool {
    matches!(
        theme,
        Theme::Caniveau
            | Theme::Egouts
            | Theme::Cascade
            | Theme::Grotte
            | Theme::Riviere
            | Theme::Port
    )
}

/// Nom de la tuile du jeu du thème pour la case solide `(tx, ty)`, ou `None` (dessin
/// du Monde 1). `vue` : la tuile vue d'une case ; `hachage` : le hachage fixe de la
/// case.
pub(crate) fn nom_du_theme(
    theme: Theme,
    (tx, ty, tuile): (i32, i32, Tuile),
    vue: impl Fn(i32, i32) -> Tuile,
    hachage: u32,
) -> Option<&'static str> {
    let choix: fn(&Voisinage) -> &'static str = match theme {
        Theme::Caniveau => caniveau,
        Theme::Egouts => egouts,
        Theme::Cascade => cascade,
        Theme::Grotte => grotte,
        Theme::Riviere => riviere,
        Theme::Port => port,
        // La marée : son niveau viendra avec son jeu.
        _ => return None,
    };
    if !tuile.est_solide() {
        return None;
    }
    // Carte 9.20a : la bouche d'égout, au bout du dernier courant du caniveau.
    if theme == Theme::Caniveau {
        if let Some(nom) = bouche_d_egout(tx, ty, &vue) {
            return Some(nom);
        }
    }
    Some(choix(&Voisinage {
        tx,
        ty,
        tuile,
        dessus: vue(tx, ty - 1),
        dessous: vue(tx, ty + 1),
        gauche: vue(tx - 1, ty),
        droite: vue(tx + 1, ty),
        hachage,
        rangee_d_eau: theme == Theme::Port
            && (1..=PORTEE_EAU)
                .any(|d| vue(tx - d, ty) == Tuile::Eau || vue(tx + d, ty) == Tuile::Eau),
    }))
}

/// Une case est « ouverte » de ce côté si rien de solide n'y est (du vide ou de l'eau).
fn ouvert(t: Tuile) -> bool {
    !t.est_solide()
}

/// La bouche d'égout du caniveau (carte 9.20a) : la bordure qui arrête un courant
/// PROFOND (deux cases d'eau l'une sur l'autre à sa gauche, sous l'air) est
/// ouverte, de haut en bas : la grille sur le trottoir, la gueule à la surface, la
/// gorge au fond. Les autres bordures (eau d'une tuile de fond) restent pleines.
/// Carte 9.20f : la rue descend, la bouche se cherche à la hauteur de son eau.
fn bouche_d_egout(tx: i32, ty: i32, vue: &impl Fn(i32, i32) -> Tuile) -> Option<&'static str> {
    // La case `(tx, y)` est-elle la gueule : la surface d'une eau profonde à sa gauche ?
    let gueule = |y: i32| {
        vue(tx - 1, y) == Tuile::Eau
            && vue(tx - 1, y + 1) == Tuile::Eau
            && vue(tx - 1, y - 1) != Tuile::Eau
    };
    if gueule(ty) {
        Some("can_egout_bouche")
    } else if gueule(ty + 1) && ouvert(vue(tx - 1, ty)) {
        Some("can_egout_grille")
    } else if gueule(ty - 1) {
        Some("can_egout_gorge")
    } else {
        None
    }
}

/// Le caniveau (refait à la carte 9.20f : la rue descend par des marches) : le sol
/// de la rue `B` (des pavés sous l'air, la bordure du trottoir au bord d'une marche
/// ou de l'eau, la terre dessous) et ce qui est bâti au-dessus `#` (corniches,
/// stores, façades). Le mobilier de rue posé sur les pavés compte comme de l'air.
fn caniveau(v: &Voisinage) -> &'static str {
    let a_l_air = v.dessus == Tuile::Vide;
    let (g, d) = (ouvert(v.gauche), ouvert(v.droite));
    if v.tuile == Tuile::Brique {
        return match (a_l_air, g, d) {
            (true, true, true) => "can_bordure",
            (true, true, false) => "can_pave_g",
            (true, false, true) => "can_pave_d",
            (true, false, false) => "can_pave",
            // Sous une construction posée sur la rue : son soubassement de granit.
            (false, _, _) if v.dessus == Tuile::Toit => "can_soubassement",
            // Sous les pavés ou sous l'eau : la terre, quelques pierres.
            _ if v.hachage.is_multiple_of(5) => "can_terre_pierre",
            _ => "can_terre",
        };
    }
    // Au-dessus de la rue : la maison, l'avant-toit, les balcons. Une rangée de
    // pierres de taille a un contour à ses deux bouts seulement ; sous une pierre,
    // au-dessus du vide, le store d'une boutique.
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

/// La case a-t-elle l'air libre au-dessus (un dessus où l'on marche) ? La rangée 0
/// n'en a jamais : hors carte, le haut est « ouvert », mais la voûte des égouts et
/// le plafond de la grotte sont des plafonds, pas des sols.
fn sol(v: &Voisinage) -> bool {
    v.ty > 0 && v.dessus == Tuile::Vide
}

/// Variante d'un dessus selon ses bords ouverts (`seul`, `g`, `d`, plein).
fn dessus_borde(v: &Voisinage, noms: [&'static str; 4]) -> &'static str {
    let [seul, g, d, plein] = noms;
    match (ouvert(v.gauche), ouvert(v.droite)) {
        (true, true) => seul,
        (true, false) => g,
        (false, true) => d,
        (false, false) => plein,
    }
}

/// Les égouts (carte 9.6c) : des briques gris-bleu sombre partout, un quai de pierre
/// claire sur chaque passage, des voûtes en arches aveugles sous les plafonds. Le
/// cyan reste à l'eau.
fn egouts(v: &Voisinage) -> &'static str {
    let (g, d) = (ouvert(v.gauche), ouvert(v.droite));
    if sol(v) {
        return dessus_borde(v, ["ego_quai_seul", "ego_quai_g", "ego_quai_d", "ego_quai"]);
    }
    if v.dessus == Tuile::Eau {
        return "ego_fond";
    }
    if v.dessous == Tuile::Vide {
        // Sous un plafond : la voûte, une arche de deux cases (colonne paire, puis
        // impaire) ; aux bouts, l'angle du mur.
        return match (g, d) {
            (true, _) => "ego_angle_g",
            (false, true) => "ego_angle_d",
            _ if v.tx.rem_euclid(2) == 0 => "ego_voute_g",
            _ => "ego_voute_d",
        };
    }
    match (g, d) {
        (true, _) => "ego_mur_g",
        (false, true) => "ego_mur_d",
        _ if v.hachage.is_multiple_of(4) => "ego_brique_sombre",
        _ => "ego_brique",
    }
}

/// La cascade (carte 9.6c) : la roche grise en strates (une rangée brune sur
/// trois), la mousse sur les dessus, la margelle de pierre au bord des bassins, des
/// galets au fond de l'eau.
fn cascade(v: &Voisinage) -> &'static str {
    let (eau_g, eau_d) = (v.gauche == Tuile::Eau, v.droite == Tuile::Eau);
    if sol(v) {
        // Au bord d'un bassin : la margelle de pierre (pas de mousse au bord de l'eau).
        return match (eau_g, eau_d) {
            (true, _) => "cas_margelle_g",
            (false, true) => "cas_margelle_d",
            _ => dessus_borde(
                v,
                [
                    "cas_mousse_seul",
                    "cas_mousse_g",
                    "cas_mousse_d",
                    "cas_mousse",
                ],
            ),
        };
    }
    if v.dessus == Tuile::Eau {
        return "cas_galets";
    }
    if eau_g || eau_d {
        return "cas_pierre";
    }
    if v.dessous == Tuile::Vide {
        return "cas_plafond";
    }
    if v.ty.rem_euclid(3) == 1 {
        "cas_brune"
    } else {
        "cas_roche"
    }
}

/// La grotte (carte 9.6c) : la roche bleu nuit, un rebord éclairé sur chaque sol, et
/// des cristaux cyan pâles sur les PAROIS (une case de paroi sur trois), jamais sur
/// un dessus où l'on marche.
fn grotte(v: &Voisinage) -> &'static str {
    let (g, d) = (ouvert(v.gauche), ouvert(v.droite));
    if sol(v) {
        return dessus_borde(v, ["gro_sol_seul", "gro_sol_g", "gro_sol_d", "gro_sol"]);
    }
    if v.dessus == Tuile::Eau {
        return "gro_fond";
    }
    if v.dessous == Tuile::Vide {
        return "gro_plafond";
    }
    let cristal = v.hachage.is_multiple_of(3) && v.dessous.est_solide();
    match (g, d) {
        (true, false) if cristal => "gro_cristal_g",
        (false, true) if cristal => "gro_cristal_d",
        _ if v.hachage.is_multiple_of(2) => "gro_roche_b",
        _ => "gro_roche",
    }
}

/// La rivière (carte 9.10) : la terre (`B`) fait les berges, herbe rousse sur les
/// dessus, galets au bord et au fond de l'eau ; le `#` fait les rochers du courant,
/// gris, coiffés de mousse.
fn riviere(v: &Voisinage) -> &'static str {
    if v.tuile == Tuile::Toit {
        return if sol(v) {
            "riv_rocher_haut"
        } else {
            "riv_rocher"
        };
    }
    if sol(v) {
        return dessus_borde(
            v,
            ["riv_herbe_seul", "riv_herbe_g", "riv_herbe_d", "riv_herbe"],
        );
    }
    if v.dessus == Tuile::Eau {
        return "riv_galets";
    }
    if v.gauche == Tuile::Eau || v.droite == Tuile::Eau {
        return "riv_berge";
    }
    if v.hachage.is_multiple_of(3) {
        "riv_terre_b"
    } else {
        "riv_terre"
    }
}

/// Le port (carte 9.6c) : le bois (`#`) fait des pontons en planches sur leurs
/// pieux, ou une caisse quand il est posé sur un solide ; la pierre (`B`) fait le
/// quai (dalles claires sur le dessus, anneaux d'amarrage aux parois, fond noyé).
/// Carte 9.11b : le mur du quai varie. À la hauteur de l'eau, la pierre noircie et
/// ses algues ; aux parois d'une colonne sur trois, une échelle de fer d'un seul
/// tenant jusqu'à l'eau ; dans le plein, des pierres de taille et des barbacanes.
fn port(v: &Voisinage) -> &'static str {
    if v.tuile == Tuile::Toit {
        if v.dessous.est_solide() {
            return "por_caisse";
        }
        return dessus_borde(
            v,
            [
                "por_ponton_seul",
                "por_ponton_g",
                "por_ponton_d",
                "por_ponton",
            ],
        );
    }
    if sol(v) {
        return dessus_borde(v, ["por_quai_seul", "por_quai_g", "por_quai_d", "por_quai"]);
    }
    if v.dessus == Tuile::Eau {
        return "por_fond";
    }
    if v.gauche == Tuile::Eau || v.droite == Tuile::Eau || v.rangee_d_eau {
        return "por_algues";
    }
    // La paroi d'un quai : de l'air, de l'eau ou un ponton (qui s'y amarre) à côté.
    let bord = |t: Tuile| ouvert(t) || t == Tuile::Toit;
    if (bord(v.gauche) || bord(v.droite)) && v.tx.rem_euclid(3) == 1 {
        return "por_echelle";
    }
    let paroi = ouvert(v.gauche) || ouvert(v.droite);
    if paroi && v.hachage.is_multiple_of(5) {
        return "por_anneau";
    }
    match v.hachage % 9 {
        0 | 5 => "por_taille",
        3 => "por_barbacane",
        _ => "por_pierre",
    }
}

/// Objet de décor posé SUR une tuile (dessiné dans la case du dessus, sans
/// collision) : la bitte d'amarrage au bout d'un quai ou d'un ponton du port. La
/// scène ne le pose que si la case du dessus est libre (ni tuile ni entité).
pub(crate) fn objet_pose(nom_tuile: &str) -> Option<&'static str> {
    matches!(
        nom_tuile,
        "por_quai_g" | "por_quai_d" | "por_ponton_g" | "por_ponton_d"
    )
    .then_some("por_bitte")
}

#[cfg(test)]
mod tests_bouche_d_egout {
    use super::*;
    use pepin_core::aabb::TAILLE_TUILE;
    use pepin_core::levels::caniveau;

    /// Carte 9.20a : dans le caniveau, la bordure au bout du dernier courant (deux
    /// tuiles de fond) est la bouche d'égout, et le drapeau est planté sur sa grille ;
    /// les bordures des bassins d'une tuile de fond restent pleines. Chaque nom
    /// choisi pour le niveau a son dessin. (Carte 9.20f : la rue descend, la bouche
    /// est à la hauteur de son eau, plus à celle du départ.)
    #[test]
    fn la_sortie_du_caniveau_est_la_bouche_d_egout() {
        let niveau = caniveau::INFOS.charger().expect("caniveau");
        let bas = niveau.hauteur() as i32;
        let vue = |x: i32, y: i32| {
            if y >= bas {
                Tuile::Brique
            } else if niveau.est_mobilier(x, y) {
                Tuile::Vide
            } else {
                niveau.tuile_decor(x, y)
            }
        };
        let nom = |tx: i32, ty: i32| {
            let t = vue(tx, ty);
            nom_du_theme(Theme::Caniveau, (tx, ty, t), vue, 0)
        };
        let mut bouches = Vec::new();
        for ty in 0..bas {
            for tx in 0..niveau.largeur() as i32 {
                if let Some(n) = nom(tx, ty) {
                    assert!(
                        crate::sprites::tuiles_monde2::SPRITES
                            .iter()
                            .any(|(dessin, _)| *dessin == n),
                        "{n} ({tx}, {ty}) n'a pas de dessin"
                    );
                    if n.starts_with("can_egout") {
                        bouches.push((tx, ty, n));
                    }
                }
            }
        }
        let (x, y) = (bouches[0].0, bouches[0].1);
        assert_eq!(
            bouches,
            vec![
                (x, y, "can_egout_grille"),
                (x, y + 1, "can_egout_bouche"),
                (x, y + 2, "can_egout_gorge"),
            ]
        );
        // Le drapeau est sur la grille.
        let drapeau = niveau.arrivee.expect("drapeau");
        assert_eq!(
            (
                (drapeau.x / TAILLE_TUILE) as i32,
                ((drapeau.y + drapeau.h) / TAILLE_TUILE).ceil() as i32
            ),
            (x, y)
        );
    }
}

#[cfg(test)]
mod tests_quai_du_port {
    use super::*;
    use pepin_core::levels::port;

    /// Carte 9.11b : le mur du quai du port n'est plus uniforme. Dans « Le port »,
    /// chaque nom choisi a son dessin ; la rangée de l'eau est en pierre aux algues
    /// d'un bout à l'autre ; chaque échelle descend d'un seul tenant de sous le
    /// dessus du quai jusqu'aux algues ; le plein mêle pierres de taille,
    /// barbacanes et pierres ordinaires.
    #[test]
    fn le_mur_du_quai_varie_et_ses_echelles_descendent_a_l_eau() {
        let niveau = port::INFOS.charger().expect("port");
        let (larg, bas) = (niveau.largeur() as i32, niveau.hauteur() as i32);
        let vue = |x: i32, y: i32| {
            if y >= bas {
                Tuile::Brique
            } else {
                niveau.tuile_decor(x, y)
            }
        };
        let nom = |tx: i32, ty: i32| {
            let h = (tx as u32).wrapping_mul(0x9E37_79B1) ^ (ty as u32).wrapping_mul(0x85EB_CA77);
            nom_du_theme(Theme::Port, (tx, ty, vue(tx, ty)), vue, h)
        };
        let mut vus = std::collections::BTreeSet::new();
        for ty in 0..bas {
            for tx in 0..larg {
                if let Some(n) = nom(tx, ty) {
                    assert!(
                        crate::sprites::tuiles_monde2::port::SPRITES
                            .iter()
                            .any(|(dessin, _)| *dessin == n),
                        "{n} ({tx}, {ty}) n'a pas de dessin"
                    );
                    vus.insert(n);
                }
            }
        }
        for n in [
            "por_taille",
            "por_barbacane",
            "por_algues",
            "por_echelle",
            "por_pierre",
        ] {
            assert!(vus.contains(n), "{n} absent du port");
        }
        // La rangée de l'eau : chaque pierre (sous le dessus du quai) y est aux algues.
        let eau = (0..bas)
            .find(|&y| (0..larg).any(|x| vue(x, y) == Tuile::Eau))
            .expect("eau");
        for tx in 0..larg {
            if vue(tx, eau) == Tuile::Brique && vue(tx, eau - 1).est_solide() {
                assert_eq!(nom(tx, eau), Some("por_algues"), "colonne {tx}");
            }
        }
        // Chaque échelle : sous le dessus du quai, d'un seul tenant jusqu'aux algues.
        let mut echelles = 0;
        for tx in 0..larg {
            let cases: Vec<i32> = (0..bas)
                .filter(|&y| nom(tx, y) == Some("por_echelle"))
                .collect();
            if let (Some(&haut), Some(&fin)) = (cases.first(), cases.last()) {
                echelles += 1;
                assert_eq!(cases.len() as i32, fin - haut + 1, "colonne {tx}");
                assert!(nom(tx, haut - 1).is_some_and(|n| n.starts_with("por_quai")));
                assert_eq!(nom(tx, fin + 1), Some("por_algues"), "colonne {tx}");
            }
        }
        assert!(echelles >= 2, "{echelles} échelles");
    }
}
