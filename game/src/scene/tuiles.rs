//! Tuiles de la grille : quel dessin poser sur chaque case (variantes de brique,
//! ombre sous les toits, cases cachées, thème), puis la pose elle-même.
//! Ne dessine ni le décor posé sur les cases (`habillage`) ni les entités.

use pepin_core::aabb::{Aabb, TAILLE_TUILE};
use pepin_core::level::Tuile;
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use crate::monde::Monde;
use crate::sprites::Banque;
use crate::tuiles_theme;

/// Noms des variantes de brique pleine (`sprites/tuiles.rs`), dans l'ordre du tirage.
const VARIANTES_BRIQUE: [&str; 3] = ["brique_fissure", "brique_manquante", "brique_coulure"];

/// En moyenne, une brique pleine sur `UNE_BRIQUE_SUR` tire une variante (un peu moins au final :
/// on en retire quelques-unes pour ne jamais coller deux variantes identiques).
const UNE_BRIQUE_SUR: u32 = 6;

/// Hachage entier FIXE d'une case : mélange les bits de `tx` et `ty` (multiplications par de
/// grands nombres impairs, puis « xor-décalages », à la manière de murmur3). Le même `(tx, ty)`
/// donne toujours le même nombre : aucun aléatoire, aucun état, donc le mur est identique d'une
/// partie à l'autre et d'une image à l'autre. `wrapping_mul` : on veut le débordement modulo 2³²
/// (en Rust, `*` panique en cas de débordement quand les vérifications sont actives, comme en
/// mode debug et dans les tests).
fn hachage_case(tx: i32, ty: i32) -> u32 {
    let mut h = (tx as u32).wrapping_mul(0x9E37_79B1) ^ (ty as u32).wrapping_mul(0x85EB_CA77);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB_352D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846C_A68B);
    h ^= h >> 16;
    h
}

/// Tirage brut d'une case : `Some(indice de variante)` environ une fois sur `UNE_BRIQUE_SUR`
/// (le reste de `h` divisé par 6 vaut 0 une fois sur 6 ; le quotient choisit le dessin).
fn tirage_brique(tx: i32, ty: i32) -> Option<usize> {
    let h = hachage_case(tx, ty);
    h.is_multiple_of(UNE_BRIQUE_SUR)
        .then_some(((h / UNE_BRIQUE_SUR) % VARIANTES_BRIQUE.len() as u32) as usize)
}

/// Variante de brique pleine à dessiner en `(tx, ty)`, ou `None` pour la brique ordinaire.
/// Fonction PURE de la position. Une case renonce à sa variante si la case de GAUCHE ou celle du
/// DESSUS a tiré la même : comme chaque case regarde ces deux voisines-là, deux cases voisines
/// (côte à côte ou l'une sur l'autre) ne montrent jamais la même variante.
fn variante_brique(tx: i32, ty: i32) -> Option<&'static str> {
    let v = tirage_brique(tx, ty)?;
    if tirage_brique(tx - 1, ty) == Some(v) || tirage_brique(tx, ty - 1) == Some(v) {
        return None;
    }
    Some(VARIANTES_BRIQUE[v])
}

/// La bande d'ombre du rebord se pose sur une brique dont la case du dessus est un toit : c'est
/// la première tuile pleine sous le toit (dans les niveaux, un toit a une seule tuile
/// d'épaisseur). Sous une cheminée posée sur le toit, la règle reste la même : la bande court
/// sans interruption tout le long du rebord.
fn ombre_sous_toit(t: Tuile, dessus: Tuile) -> bool {
    t == Tuile::Brique && dessus == Tuile::Toit
}

/// Tuile VUE en `(tx, ty)` : celle de la grille, ou, pour un faux mur (carte 8.5), le solide
/// qu'il imite (révélé ou non : la forme du mur ne change pas, seul son tramage).
fn tuile_vue(monde: &Monde, tx: i32, ty: i32) -> Tuile {
    match monde.niveau.tuile_decor(tx, ty) {
        Tuile::Vide => monde
            .niveau
            .cache_en(tx, ty)
            .map_or(Tuile::Vide, |(_, t)| t),
        t => t,
    }
}

/// La case `(tx, ty)` est-elle libre pour un objet de décor posé (carte 9.6c) : ni
/// tuile, ni entité du niveau (lanterne, goutte, ennemi...) qui la touche ?
fn case_libre(monde: &Monde, tx: i32, ty: i32) -> bool {
    let t = TAILLE_TUILE;
    let case = Aabb::new(tx as f32 * t, ty as f32 * t, t, t);
    tuile_vue(monde, tx, ty) == Tuile::Vide
        && !monde
            .niveau
            .entites
            .iter()
            .any(|e| e.boite().chevauche(&case))
}

/// La case `(tx, ty)` est-elle dans une zone cachée PAS ENCORE révélée (carte 8.5) ?
fn case_cachee(monde: &Monde, tx: i32, ty: i32) -> bool {
    monde
        .niveau
        .cache_en(tx, ty)
        .is_some_and(|(k, _)| !monde.caches_reveles[k])
}

/// Le centre de la boîte `b` est-il dans une zone cachée pas encore révélée ? (La goutte
/// d'une salle secrète reste cachée derrière le faux mur.)
pub(super) fn boite_cachee(monde: &Monde, b: &Aabb) -> bool {
    let (x, y) = (b.x + b.w / 2.0, b.y + b.h / 2.0);
    case_cachee(
        monde,
        (x / TAILLE_TUILE).floor() as i32,
        (y / TAILLE_TUILE).floor() as i32,
    )
}

/// La case `(tx, ty)` est-elle l'indice d'une zone cachée (carte 8.5) ?
fn case_d_indice(monde: &Monde, tx: i32, ty: i32) -> bool {
    monde
        .niveau
        .caches
        .iter()
        .any(|z| z.indice == Some((tx, ty)))
}

/// Pas (1/60 s) entre deux temps de la vague de l'eau (carte 9.2) : la surface
/// monte et descend d'un pixel toutes les 0,4 s.
pub(crate) const PAS_VAGUE: u32 = 24;

/// Nom de la tuile à dessiner : version « haut » si rien de solide au-dessus ; un nuage dont la
/// case voisine est vide (à gauche et/ou à droite) prend un bord arrondi. Dessin seulement :
/// la collision reste celle de la case. Les faux murs comptent comme le solide qu'ils imitent
/// (`tuile_vue`) ; la case d'indice d'une zone cachée est une brique décalée.
pub(crate) fn nom_tuile(monde: &Monde, tx: i32, ty: i32, t: Tuile) -> &'static str {
    // Carte 9.12f : la plateforme que la Pieuvre saisit se dessine à part
    // (`dessin_pieuvre::dessiner_saisie`) : elle tremble, coule, remonte.
    if crate::dessin_pieuvre::case_saisie(monde, tx, ty) {
        return "";
    }
    nom_tuile_brut(monde, tx, ty, t)
}

/// Pour la plateforme saisie par la Pieuvre (carte 9.12f) : le nom de la tuile de
/// la case `(tx, ty)` comme si de rien n'était, et l'objet posé dessus (la bitte
/// d'amarrage), qui coule avec elle.
pub(crate) fn noms_de_la_case(
    monde: &Monde,
    tx: i32,
    ty: i32,
) -> (&'static str, Option<&'static str>) {
    let nom = nom_tuile_brut(monde, tx, ty, tuile_vue(monde, tx, ty));
    let objet = tuiles_theme::objet_pose(nom).filter(|_| case_libre(monde, tx, ty - 1));
    (nom, objet)
}

fn nom_tuile_brut(monde: &Monde, tx: i32, ty: i32, t: Tuile) -> &'static str {
    // Monde 2 (carte 9.6b) : un thème qui a son propre jeu de tuiles choisit d'abord.
    // Sous la dernière rangée, le jeu voit du solide (le sol continue hors carte) :
    // la rangée du bas n'est ni un plafond ni une voûte (carte 9.6c).
    let bas = monde.niveau.hauteur() as i32;
    // Carte 9.20f : le mobilier de rue se dessine d'un seul tenant
    // (`mobilier_rue`) ; ses cases ne sont pas des tuiles, et le jeu du thème le
    // voit comme de l'air (les pavés restent des pavés sous une voiture).
    if monde.niveau.est_mobilier(tx, ty) {
        return "";
    }
    let vue = |x, y| {
        if y >= bas {
            Tuile::Brique
        } else if monde.niveau.est_mobilier(x, y) {
            Tuile::Vide
        } else {
            tuile_vue(monde, x, y)
        }
    };
    // Carte 9.20e : la jetée de la Pieuvre dessine elle-même ses quais et son
    // bassin (`dessin_port::dessiner_jetee`) ; ici, rien.
    if crate::dessin_port::dessine_la_case(monde, tx, ty, t) {
        return "";
    }
    let theme = monde.infos.theme;
    if let Some(nom) = tuiles_theme::nom_du_theme(theme, (tx, ty, t), vue, hachage_case(tx, ty)) {
        return nom;
    }
    let haut = tuile_vue(monde, tx, ty - 1) == Tuile::Vide;
    if t == Tuile::Nuage && haut {
        let (gauche, droite) = (
            tuile_vue(monde, tx - 1, ty) == Tuile::Vide,
            tuile_vue(monde, tx + 1, ty) == Tuile::Vide,
        );
        match (gauche, droite) {
            (true, true) => return "nuage_ilot",
            (true, false) => return "nuage_bord_g",
            (false, true) => return "nuage_bord_d",
            (false, false) => {}
        }
    }
    match (t, haut) {
        (Tuile::Toit, true) => "toit_haut",
        (Tuile::Toit, false) => "toit_plein",
        (Tuile::Brique, true) => "brique_haut",
        (Tuile::Brique, false) if case_d_indice(monde, tx, ty) => "brique_decalee",
        (Tuile::Nuage, false) if case_d_indice(monde, tx, ty) => "nuage_creux",
        (Tuile::Brique, false) => variante_brique(tx, ty).unwrap_or("brique_plein"),
        (Tuile::Nuage, true) => "nuage_haut",
        (Tuile::Nuage, false) => "nuage_plein",
        // Eau (carte 9.2) : la version « haut » (la surface) est posée quand la
        // case au-dessus est vide ; le sens du courant donne les chevrons. La
        // surface ONDULE d'un pixel : « haut » et « vague » (le même dessin 1 px
        // plus bas) alternent tous les `PAS_VAGUE` pas, à contretemps d'une
        // colonne à sa voisine. `rem_euclid` garde un reste positif même pour
        // une colonne négative.
        (Tuile::Eau, _) => {
            // Sous l'eau d'une marée montée (carte 9.3), ce n'est plus la surface.
            let haut = haut && !crate::eau::sous_une_maree(monde, tx, ty);
            let vague = (monde.pas / PAS_VAGUE) as i32 + tx;
            let (surface, creux, plein) = match monde.niveau.courant_eau(tx, ty) {
                -1 => (
                    "eau_courant_g_haut",
                    "eau_courant_g_vague",
                    "eau_courant_g_plein",
                ),
                1 => (
                    "eau_courant_d_haut",
                    "eau_courant_d_vague",
                    "eau_courant_d_plein",
                ),
                _ => ("eau_calme_haut", "eau_calme_vague", "eau_calme_plein"),
            };
            match (haut, vague.rem_euclid(2) == 1) {
                (false, _) => plein,
                (true, false) => surface,
                (true, true) => creux,
            }
        }
        (Tuile::Vide, _) => "",
    }
}

pub(super) fn dessiner_tuiles(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    let taille = TAILLE_TUILE as i32;
    let (t0, t1) = (
        cx.div_euclid(taille),
        (cx + LARGEUR_LOGIQUE as i32).div_euclid(taille),
    );
    let (u0, u1) = (
        cy.div_euclid(taille),
        (cy + HAUTEUR_LOGIQUE as i32).div_euclid(taille),
    );
    for ty in u0..=u1 {
        for tx in t0..=t1 {
            // Hors carte : `tuile_decor` renvoie des murs à gauche/droite ; on ne les dessine pas.
            if tx < 0 || tx >= monde.niveau.largeur() as i32 {
                continue;
            }
            let t = tuile_vue(monde, tx, ty);
            if t != Tuile::Vide {
                let nom = nom_tuile(monde, tx, ty, t);
                let (x, y) = ((tx * taille - cx) as f32, (ty * taille - cy) as f32);
                // Faux mur révélé (carte 8.5) : tramé en damier, on voit la salle derrière.
                let revele =
                    monde.niveau.tuile_decor(tx, ty) == Tuile::Vide && !case_cachee(monde, tx, ty);
                let poser = |nom: &str| {
                    if revele {
                        banque.dessiner_tuile_tramee(monde.infos.theme, nom, x, y);
                    } else {
                        banque.dessiner_tuile(monde.infos.theme, nom, x, y);
                    }
                };
                poser(nom);
                // Carte 9.6c : un objet posé sur la tuile (la bitte d'amarrage du
                // port), dans la case du dessus quand elle est libre.
                if let Some(objet) = tuiles_theme::objet_pose(nom) {
                    if !revele && case_libre(monde, tx, ty - 1) {
                        banque.dessiner_tuile(monde.infos.theme, objet, x, y - taille as f32);
                    }
                }
                // L'indice de brique d'une zone cachée (carte 8.5) dépasse sur la case libre.
                if nom == "brique_decalee" && !revele {
                    let (saillie, dx) = if tuile_vue(monde, tx - 1, ty) == Tuile::Vide {
                        ("brique_saillie_g", -taille)
                    } else {
                        ("brique_saillie_d", taille)
                    };
                    banque.dessiner_tuile(monde.infos.theme, saillie, x + dx as f32, y);
                }
                // Bande d'ombre du rebord, par-dessus le haut de la brique.
                if ombre_sous_toit(t, tuile_vue(monde, tx, ty - 1))
                    && !tuiles_theme::a_son_jeu(monde.infos.theme)
                {
                    poser("ombre_toit");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Variantes de brique et ombre sous les toits ---

    #[test]
    fn le_choix_de_la_variante_de_brique_est_fixe() {
        // Même case, même réponse, à chaque appel (fonction pure, sans aléatoire).
        for ty in -3..15 {
            for tx in -3..250 {
                assert_eq!(variante_brique(tx, ty), variante_brique(tx, ty));
            }
        }
        // Valeurs de référence (relevées une fois avec ce hachage) : si elles changent,
        // tous les murs du jeu changent d'aspect ; il faut alors le vouloir.
        assert_eq!(hachage_case(0, 0), 0);
        assert_eq!(hachage_case(1, 0), 1_834_104_592);
        assert_eq!(hachage_case(-1, -1), 857_305_552);
        let rangee_9: Vec<(i32, &str)> = (0..40)
            .filter_map(|tx| variante_brique(tx, 9).map(|v| (tx, v)))
            .collect();
        assert_eq!(
            rangee_9,
            [
                (0, "brique_manquante"),
                (4, "brique_manquante"),
                (11, "brique_coulure"),
                (14, "brique_coulure"),
                (16, "brique_fissure"),
                (32, "brique_coulure"),
                (36, "brique_fissure"),
                (37, "brique_manquante"),
                (39, "brique_manquante"),
            ]
        );
    }

    #[test]
    fn environ_une_brique_sur_six_est_une_variante() {
        let mut compte = [0usize; 3];
        let mut total = 0;
        for ty in 0..40 {
            for tx in 0..400 {
                total += 1;
                if let Some(v) = variante_brique(tx, ty) {
                    compte[VARIANTES_BRIQUE.iter().position(|&n| n == v).unwrap()] += 1;
                }
            }
        }
        let part = compte.iter().sum::<usize>() as f32 / total as f32;
        // Demande : une tuile sur 5 à 8.
        assert!((1.0 / 8.0..=1.0 / 5.0).contains(&part), "part : {part}");
        // Les trois dessins sortent à peu près autant les uns que les autres.
        for (i, &n) in compte.iter().enumerate() {
            let p = n as f32 / compte.iter().sum::<usize>() as f32;
            assert!((0.25..=0.42).contains(&p), "{} : {p}", VARIANTES_BRIQUE[i]);
        }
    }

    #[test]
    fn deux_variantes_identiques_ne_se_touchent_jamais() {
        for ty in -5..40 {
            for tx in -50..400 {
                if let Some(v) = variante_brique(tx, ty) {
                    assert_ne!(variante_brique(tx + 1, ty), Some(v), "({tx},{ty}) à droite");
                    assert_ne!(variante_brique(tx, ty + 1), Some(v), "({tx},{ty}) dessous");
                }
            }
        }
    }

    /// Dans les vrais niveaux : part des briques pleines dessinées en variante, et nombre de
    /// briques qui reçoivent la bande d'ombre d'un toit.
    #[test]
    fn variantes_et_ombres_dans_les_vrais_niveaux() {
        use pepin_core::zones::{Theme, NIVEAUX};
        let (mut pleines, mut variantes) = (0, 0);
        for infos in NIVEAUX {
            let n = infos.charger().unwrap();
            let mut ombres = 0;
            for ty in 0..n.hauteur() as i32 {
                for tx in 0..n.largeur() as i32 {
                    let (t, dessus) = (n.tuile_decor(tx, ty), n.tuile_decor(tx, ty - 1));
                    if t == Tuile::Brique && dessus.est_solide() {
                        pleines += 1;
                        variantes += usize::from(variante_brique(tx, ty).is_some());
                    }
                    if ombre_sous_toit(t, dessus) {
                        ombres += 1;
                    }
                }
            }
            // Les niveaux de toits ont un long rebord ; les nuages n'en ont pas. Le
            // clocher, haut mais étroit (34 colonnes), a des dalles et des corniches
            // plus courtes : la moitié suffit.
            match infos.theme {
                Theme::Toits | Theme::Gouttieres | Theme::Marche | Theme::Orage => {
                    assert!(ombres > 100, "{} : {ombres} ombres", infos.nom)
                }
                Theme::Clocher => assert!(ombres > 50, "{} : {ombres} ombres", infos.nom),
                // La descente : des couches de nuages, puis le toit du premier niveau
                // (32 tuiles de large, briques dessous).
                Theme::Descente => assert!(ombres >= 30, "{} : {ombres} ombres", infos.nom),
                // L'œil du cyclone, la mer de nuages et l'arc-en-ciel : des nuages,
                // sans toit ni brique.
                Theme::Soleil | Theme::Cyclone | Theme::Mer | Theme::ArcEnCiel => {
                    assert_eq!(ombres, 0, "{}", infos.nom)
                }
                // L'altitude, le retour du Grand Nuage et le Monde 2 (carte 9.5,
                // niveaux à venir) n'ont pas encore de contrôle d'ombre propre.
                Theme::Altitude
                | Theme::Poursuite
                | Theme::Caniveau
                | Theme::Egouts
                | Theme::Cascade
                | Theme::Grotte
                | Theme::Riviere
                | Theme::Port
                | Theme::Maree => {}
            }
        }
        let part = variantes as f32 / pleines as f32;
        assert!(
            (1.0 / 8.0..=1.0 / 5.0).contains(&part),
            "{variantes} / {pleines}"
        );
    }

    #[test]
    fn l_ombre_ne_tombe_que_d_un_toit_sur_une_brique() {
        assert!(ombre_sous_toit(Tuile::Brique, Tuile::Toit));
        assert!(!ombre_sous_toit(Tuile::Brique, Tuile::Brique));
        assert!(!ombre_sous_toit(Tuile::Brique, Tuile::Vide));
        assert!(!ombre_sous_toit(Tuile::Toit, Tuile::Toit));
        assert!(!ombre_sous_toit(Tuile::Nuage, Tuile::Toit));
        assert!(!ombre_sous_toit(Tuile::Vide, Tuile::Toit));
    }
}
