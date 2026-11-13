//! Sprites en pixel art, 100 % dans le code (aucun fichier image).
//!
//! Un dessin est une grille de texte : une ligne = une rangée de pixels,
//! 1 caractère = 1 pixel. `.` = transparent ; `0`-`9` puis `A`-`F` = indice
//! dans `pepin_core::palette::TOUTES` (0 = ombre de contour, 8 = corail du
//! parapluie, B = or, D = cyan de la pluie, etc.).
//!
//! Au démarrage, `Banque::charger` transforme chaque grille en `Texture2D`
//! (filtre « nearest » : jamais de flou). Le décodage lui-même (`pixel`,
//! `verifier`) est pur : il ne dépend pas du moteur graphique et se teste.
//!
//! Notions Rust illustrées ici : alias de type (`Image`), `Result` avec un
//! `char` fautif comme erreur, itérateurs, `HashMap`, tests unitaires.

// Les dessins sont des grilles de texte : rustfmt les tasserait sur une seule
// ligne et les rendrait illisibles, donc on l'exclut de ces cinq fichiers.
#[rustfmt::skip]
pub mod ennemis;
#[rustfmt::skip]
pub mod objets;
#[rustfmt::skip]
pub mod objets_decor;
#[rustfmt::skip]
pub mod pepin;
#[cfg(test)]
mod planche;
#[rustfmt::skip]
pub mod tuiles;

use std::collections::HashMap;

use macroquad::prelude::{
    DrawTextureParams, FilterMode, Image as ImageMq, Texture2D, BLANK, WHITE,
};
use pepin_core::palette::{Couleur, TOUTES};
use pepin_core::zones::Theme;

use crate::decor::ambiance;
use crate::rendu::couleur_mq;

/// Un dessin : une liste de lignes de texte, toutes de même longueur.
pub type Image = &'static [&'static str];

/// Décode un caractère de dessin : `Ok(None)` = transparent, `Ok(Some(c))` =
/// couleur de la palette, `Err(caractère)` = caractère interdit.
pub fn pixel(c: char) -> Result<Option<Couleur>, char> {
    if c == '.' {
        return Ok(None);
    }
    match c.to_digit(16) {
        // `to_digit(16)` accepte 0-9 et a-f / A-F ; la palette a 16 entrées.
        Some(i) if c.is_ascii_uppercase() || c.is_ascii_digit() => Ok(Some(TOUTES[i as usize])),
        _ => Err(c),
    }
}

/// Largeur d'un dessin en pixels (longueur de la première ligne).
pub fn largeur(image: Image) -> usize {
    image.first().map_or(0, |l| l.chars().count())
}

/// Hauteur d'un dessin en pixels.
pub fn hauteur(image: Image) -> usize {
    image.len()
}

#[cfg(test)]
/// Vérifie qu'un dessin est rectangulaire et n'utilise que des caractères permis.
pub fn verifier(nom: &str, image: Image) -> Result<(), String> {
    let l = largeur(image);
    if l == 0 || image.is_empty() {
        return Err(format!("{nom} : dessin vide"));
    }
    for (y, ligne) in image.iter().enumerate() {
        if ligne.chars().count() != l {
            return Err(format!("{nom} : la ligne {y} n'a pas {l} pixels"));
        }
        for c in ligne.chars() {
            pixel(c).map_err(|c| format!("{nom} : caractère interdit {c:?} ligne {y}"))?;
        }
    }
    Ok(())
}

#[cfg(test)]
/// Tous les sprites du jeu (nom, dessin), tous fichiers confondus.
pub fn tous() -> Vec<(&'static str, Image)> {
    [
        pepin::SPRITES,
        ennemis::SPRITES,
        objets::SPRITES,
        objets_decor::SPRITES,
        tuiles::SPRITES,
    ]
    .concat()
}

/// Table de substitution de couleurs d'un thème : les tuiles s'éclaircissent
/// d'un niveau à l'autre (l'ardoise gagne `eclaircissement` crans, jusqu'au blanc
/// cassé). Les réglages viennent de la table d'ambiance (`decor::ambiance`).
fn remplacer(c: char, theme: Theme) -> char {
    let teinte = ambiance(theme).tuiles;
    match c {
        // Rampe d'ardoise 1..=4 : on avance sur la rampe sans dépasser 7.
        '1'..='4' => char::from_digit(
            (c.to_digit(10).unwrap_or(1) + teinte.eclaircissement).min(7),
            10,
        )
        .unwrap_or(c),
        // Les briques violettes virent à l'orange sous le soleil.
        'F' if teinte.briques_orangees => 'A',
        _ => c,
    }
}

/// Convertit une grille en texture « nearest ». `theme` : substitution de couleurs.
fn texture(image: Image, theme: Option<Theme>) -> Texture2D {
    let mut img = ImageMq::gen_image_color(largeur(image) as u16, hauteur(image) as u16, BLANK);
    for (y, ligne) in image.iter().enumerate() {
        for (x, c) in ligne.chars().enumerate() {
            let c = theme.map_or(c, |t| remplacer(c, t));
            if let Ok(Some(couleur)) = pixel(c) {
                img.set_pixel(x as u32, y as u32, couleur_mq(couleur));
            }
        }
    }
    let t = Texture2D::from_image(&img);
    t.set_filter(FilterMode::Nearest);
    t
}

/// Toutes les textures du jeu, rangées par nom.
pub struct Banque {
    /// Sprites (Pépin, ennemis, objets, décor) : les mêmes pour tous les niveaux.
    sprites: HashMap<&'static str, Texture2D>,
    /// Tuiles : un jeu complet par thème (couleurs éclaircies progressivement).
    tuiles: Vec<HashMap<&'static str, Texture2D>>,
}

impl Banque {
    /// Génère toutes les textures. À appeler une seule fois, après l'ouverture de la fenêtre.
    pub fn charger() -> Self {
        let sprites = [
            pepin::SPRITES,
            ennemis::SPRITES,
            objets::SPRITES,
            objets_decor::SPRITES,
        ]
        .concat()
        .into_iter()
        .map(|(nom, dessin)| (nom, texture(dessin, None)))
        .collect();
        let tuiles = Theme::TOUS
            .iter()
            .map(|&t| {
                tuiles::SPRITES
                    .iter()
                    .map(|&(nom, dessin)| (nom, texture(dessin, Some(t))))
                    .collect()
            })
            .collect();
        Self { sprites, tuiles }
    }

    /// Dessine le sprite `nom` avec son coin haut-gauche en `(x, y)` (arrondi au pixel).
    /// `miroir` : retourne l'image horizontalement (Pépin qui regarde à gauche).
    pub fn dessiner(&self, nom: &str, x: f32, y: f32, miroir: bool) {
        if let Some(t) = self.sprites.get(nom) {
            Self::poser(t, x, y, miroir);
        }
    }

    /// Dessine la tuile `nom` du thème donné.
    pub fn dessiner_tuile(&self, theme: Theme, nom: &str, x: f32, y: f32) {
        if let Some(t) = self.tuiles[theme.indice()].get(nom) {
            Self::poser(t, x, y, false);
        }
    }

    fn poser(t: &Texture2D, x: f32, y: f32, miroir: bool) {
        macroquad::prelude::draw_texture_ex(
            t,
            x.round(),
            y.round(),
            WHITE,
            DrawTextureParams {
                flip_x: miroir,
                ..Default::default()
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tous_les_sprites_sont_valides() {
        let liste = tous();
        assert!(liste.len() > 40, "{} sprites", liste.len());
        for (nom, dessin) in liste {
            verifier(nom, dessin).unwrap();
        }
    }

    #[test]
    fn les_noms_de_sprites_sont_uniques() {
        let liste = tous();
        let mut noms: Vec<&str> = liste.iter().map(|(n, _)| *n).collect();
        noms.sort_unstable();
        let total = noms.len();
        noms.dedup();
        assert_eq!(noms.len(), total, "nom de sprite en double");
    }

    #[test]
    fn le_decodeur_refuse_les_caracteres_inconnus() {
        assert_eq!(pixel('.'), Ok(None));
        assert_eq!(pixel('0'), Ok(Some(TOUTES[0])));
        assert_eq!(pixel('F'), Ok(Some(TOUTES[15])));
        assert_eq!(pixel('f'), Err('f'), "minuscules interdites");
        assert_eq!(pixel('z'), Err('z'));
        assert_eq!(pixel('#'), Err('#'));
    }

    #[test]
    fn verifier_detecte_une_grille_irreguliere() {
        const MAUVAISE: Image = &["000", "00"];
        assert!(verifier("x", MAUVAISE).is_err());
        const VIDE: Image = &[];
        assert!(verifier("x", VIDE).is_err());
    }

    /// Dimensions attendues (largeur, hauteur) selon la famille de sprites.
    fn taille(nom: &str) -> (usize, usize) {
        let dessin = tous().into_iter().find(|(n, _)| *n == nom).unwrap().1;
        (largeur(dessin), hauteur(dessin))
    }

    #[test]
    fn dimensions_constantes_des_sprites_de_reference() {
        assert_eq!(taille("pepin_idle0"), (16, 16));
        assert_eq!(taille("pepin_plane0"), (20, 20));
        assert_eq!(taille("flaque0"), (16, 10));
        assert_eq!(taille("goutte0"), (10, 10));
        assert_eq!(taille("lanterne0"), (8, 16));
        assert_eq!(taille("champignon0"), (16, 12));
        assert_eq!(taille("pointe"), (8, 8));
        assert_eq!(taille("toit_plein"), (16, 16));
    }

    /// Toutes les images d'une animation doivent avoir la même taille,
    /// sinon le sprite « sauterait » d'une image à l'autre.
    #[test]
    fn les_animations_ont_des_images_de_meme_taille() {
        let animations: Vec<&[&str]> = vec![
            &pepin::COURSE,
            &pepin::REPOS,
            &pepin::ATTENTE,
            &pepin::PLANER,
            &pepin::REAPPARITION,
            &ennemis::FLAQUE,
            &ennemis::CORBEAU,
            &ennemis::PIGEON,
            &objets::GOUTTE,
            &objets::LANTERNE,
            &objets::CHAMPIGNON,
            &objets::FRIABLE,
            &objets::FRIABLE_ARDOISE,
            &objets::DRAPEAU,
            &objets::GRELON,
            &objets::MONTGOLFIERE,
        ];
        for anim in animations {
            let tailles: Vec<_> = anim.iter().map(|n| taille(n)).collect();
            assert!(
                tailles.windows(2).all(|p| p[0] == p[1]),
                "{anim:?} : tailles différentes {tailles:?}"
            );
        }
    }

    #[test]
    fn la_substitution_de_theme_reste_dans_la_palette() {
        for theme in Theme::TOUS {
            for c in "0123456789ABCDEF.".chars() {
                assert!(pixel(remplacer(c, theme)).is_ok());
            }
        }
        // Le thème du soleil éclaircit l'ardoise : '2' devient '5'.
        assert_eq!(remplacer('2', Theme::Soleil), '5');
        assert_eq!(remplacer('2', Theme::Toits), '2');
    }

    // --- Corrections de lisibilité (audit des sprites) ---

    fn dessin(nom: &str) -> Image {
        tous().into_iter().find(|(n, _)| *n == nom).unwrap().1
    }

    /// Contraste WCAG entre deux couleurs de la palette.
    fn contraste(a: Couleur, b: Couleur) -> f32 {
        fn lum(c: Couleur) -> f32 {
            let lin = |v: u8| {
                let v = v as f32 / 255.0;
                if v <= 0.03928 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
        }
        let (la, lb) = (lum(a), lum(b));
        (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
    }

    fn couleur_de(c: char) -> Couleur {
        pixel(c).unwrap().unwrap()
    }

    /// Le caractère du dessin en `(x, y)`, `.` hors du dessin.
    fn car(image: Image, x: i32, y: i32) -> char {
        if x < 0 || y < 0 {
            return '.';
        }
        image
            .get(y as usize)
            .and_then(|l| l.chars().nth(x as usize))
            .unwrap_or('.')
    }

    #[test]
    fn la_goutte_est_cernee_de_deux_pixels_de_contour_sombre() {
        for nom in objets::GOUTTE {
            let g = dessin(nom);
            assert_eq!((largeur(g), hauteur(g)), (10, 10), "{nom}");
            for y in 0..10 {
                for x in 0..10 {
                    let c = car(g, x, y);
                    let voisins = [(1, 0), (-1, 0), (0, 1), (0, -1)];
                    // Tout pixel qui touche le vide est du contour sombre (anneau extérieur).
                    if c != '.'
                        && c != 'C'
                        && voisins.iter().any(|(dx, dy)| car(g, x + dx, y + dy) == '.')
                    {
                        assert_eq!(c, '0', "{nom} : le bord ({x},{y}) n'est pas sombre");
                    }
                    // Un pixel doré est séparé du vide par au moins 2 pixels de contour.
                    if c == 'B' || c == 'A' {
                        for (dx, dy) in voisins {
                            let (nx, ny) = (x + dx, y + dy);
                            assert_ne!(car(g, nx, ny), '.', "{nom} : or au bord ({x},{y})");
                            assert!(
                                voisins
                                    .iter()
                                    .all(|(ex, ey)| car(g, nx + ex, ny + ey) != '.'),
                                "{nom} : un seul pixel de contour près de ({x},{y})"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn la_vitre_de_la_lanterne_eteinte_est_claire() {
        let l = dessin("lanterne0");
        // Les trois rangées de vitre : ARDOISE_PALE (4) et BRUME (5), plus de bleu nuit.
        for (y, ligne) in l.iter().enumerate().skip(3).take(3) {
            assert_eq!(*ligne, ".045540.", "rangée {y}");
        }
        assert!(contraste(couleur_de('5'), pepin_core::palette::ARDOISE_CLAIRE) >= 3.0);
        assert!(contraste(couleur_de('4'), pepin_core::palette::ARDOISE) >= 3.0);
    }

    #[test]
    fn le_corbeau_se_detache_du_ciel_d_ardoise() {
        for nom in ennemis::CORBEAU {
            let c = dessin(nom);
            assert!(
                c.iter().all(|l| !l.contains('2')),
                "{nom} : il reste de l'ardoise (couleur du ciel)"
            );
            assert!(
                c.iter().any(|l| l.contains('A')),
                "{nom} : le bec orange a disparu"
            );
            assert!(
                c.iter().any(|l| l.contains('0')),
                "{nom} : le contour a disparu"
            );
        }
        assert!(contraste(couleur_de('4'), pepin_core::palette::ARDOISE) >= 3.0);
        assert!(contraste(couleur_de('3'), pepin_core::palette::ARDOISE) < 3.0);
    }

    // --- Pigeon (carte 4.4) ---

    /// Les six images du pigeon : perché (2), annonce, plongée, remontée, écrasé.
    const PIGEON_TOUT: [&str; 6] = [
        "pigeon0",
        "pigeon1",
        "pigeon_annonce",
        "pigeon_plongee",
        "pigeon_remontee",
        "pigeon_ecrase",
    ];

    /// Nombre de pixels opaques d'un dessin.
    fn opaques(d: Image) -> usize {
        d.iter()
            .map(|l| l.chars().filter(|&c| c != '.').count())
            .sum()
    }

    /// Couleur la plus fréquente d'un dessin, contour sombre (`0`) et vide exclus.
    fn couleur_principale(d: Image) -> char {
        let mut compte: HashMap<char, usize> = HashMap::new();
        for c in d.iter().flat_map(|l| l.chars()) {
            if c != '.' && c != '0' {
                *compte.entry(c).or_default() += 1;
            }
        }
        // À égalité, le plus petit caractère : résultat stable d'une exécution à l'autre.
        compte
            .into_iter()
            .max_by_key(|&(c, n)| (n, std::cmp::Reverse(c)))
            .unwrap()
            .0
    }

    #[test]
    fn le_pigeon_a_six_images_de_meme_taille_et_un_contour_sombre() {
        for nom in PIGEON_TOUT {
            assert_eq!(taille(nom), (16, 12), "{nom}");
            let d = dessin(nom);
            for y in 0..12 {
                for x in 0..16 {
                    let c = car(d, x, y);
                    if c == '.' || c == '0' {
                        continue;
                    }
                    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        assert_ne!(
                            car(d, x + dx, y + dy),
                            '.',
                            "{nom} : ({x},{y}) « {c} » touche le vide sans contour"
                        );
                        assert!((0..16).contains(&(x + dx)) && (0..12).contains(&(y + dy)));
                    }
                }
            }
        }
        // Perché, il ne fait que hocher la tête : le dessous du corps ne bouge pas.
        let (a, b) = (dessin("pigeon0"), dessin("pigeon1"));
        assert_eq!(&a[6..], &b[6..]);
        assert_ne!(a, b);
    }

    /// Exigence du coordinateur (carte 4.4) : on distingue d'un coup d'œil l'oiseau de
    /// décor, inoffensif, du pigeon qui plonge.
    #[test]
    fn le_pigeon_ne_ressemble_pas_a_l_oiseau_de_decor() {
        let oiseau = dessin("oiseau0");
        for nom in PIGEON_TOUT {
            let d = dessin(nom);
            // Pas blanc : aucune des couleurs claires de l'oiseau (brume, nuage, blanc).
            assert!(
                d.iter().all(|l| !l.contains(['5', '6', '7'])),
                "{nom} : couleur claire de l'oiseau de décor"
            );
            // Cou irisé : sarcelle (E) et cyan (D), toujours visibles.
            assert!(d.iter().any(|l| l.contains('E')) && d.iter().any(|l| l.contains('D')));
        }
        // Plus gros : 16 x 12 contre 12 x 8, et au moins 1,5 fois plus de pixels pleins.
        assert_eq!(taille("oiseau0"), (12, 8));
        let (p, o) = (opaques(dessin("pigeon0")), opaques(oiseau));
        assert!(2 * p >= 3 * o, "pigeon {p} px, oiseau {o} px");
        // Corps sombre (ardoise) contre corps blanc : contraste fort entre les deux.
        assert_eq!(couleur_principale(dessin("pigeon0")), '2');
        assert_eq!(couleur_principale(oiseau), '7');
        assert!(contraste(couleur_de('2'), couleur_de('7')) >= 7.0);
        // Regard menaçant : l'œil (orange, ou corail pendant l'annonce) a un pixel sombre
        // juste au-dessus (le sourcil froncé).
        for nom in ["pigeon0", "pigeon1", "pigeon_annonce"] {
            let d = dessin(nom);
            let oeil = (0..12)
                .flat_map(|y| (0..16).map(move |x| (x, y)))
                .find(|&(x, y)| matches!(car(d, x, y), 'A' | '8'))
                .unwrap_or_else(|| panic!("{nom} : pas d'œil"));
            assert_eq!(car(d, oeil.0, oeil.1 - 1), '0', "{nom} : pas de sourcil");
        }
    }

    /// Contraste d'au moins 3:1 entre le pigeon et le ciel derrière lui, dans les vrais
    /// niveaux 4 et 5 : pour chaque pigeon posé, toutes les bandes de ciel qu'il traverse
    /// (perché, puis jusqu'au fond de sa plongée) contrastent avec son corps (ardoise) et
    /// son contour. Le haut du ciel du niveau 4 est violet (ARDOISE et violet : 1,5:1) :
    /// ce test interdit d'y percher un pigeon.
    #[test]
    fn le_pigeon_se_detache_du_ciel_des_niveaux_4_et_5() {
        use crate::decor::{ambiance, ciel_a};
        use pepin_core::entities::ennemi::PROFONDEUR_PLONGEE;
        use pepin_core::entities::{Entite, TypeEnnemi};
        use pepin_core::levels::{nuages, vents};
        use pepin_core::HAUTEUR_LOGIQUE;
        let mut vus = 0;
        for infos in [vents::INFOS, nuages::INFOS] {
            let niveau = infos.charger().unwrap();
            // Niveau pas plus haut que l'écran : la caméra aligne son bas sur celui de
            // l'écran (`Camera::borner`) et ne bouge pas en hauteur.
            assert!(niveau.hauteur_px() <= HAUTEUR_LOGIQUE as f32);
            let cam_y = niveau.hauteur_px() - HAUTEUR_LOGIQUE as f32;
            let a = ambiance(infos.theme);
            for e in &niveau.entites {
                let Entite::Ennemi(p) = e else { continue };
                if p.type_ennemi != TypeEnnemi::Pigeon {
                    continue;
                }
                vus += 1;
                // Sprite 16x12 posé 2 px au-dessus de la boîte (voir `scene.rs`).
                let haut = (p.boite.y - 2.0 - cam_y) as i32;
                let bas = (p.boite.bas() + PROFONDEUR_PLONGEE - cam_y) as i32;
                // Les 6 dernières lignes d'une bande sont tramées avec la suivante.
                for y in haut..bas.min(HAUTEUR_LOGIQUE as i32) {
                    for ciel in [ciel_a(&a, y), ciel_a(&a, y + 6)] {
                        for c in ['2', '0'] {
                            let k = contraste(couleur_de(c), ciel);
                            assert!(
                                k >= 3.0,
                                "niveau {} : pigeon en {:?}, « {c} » sur {ciel:?} à y = {y} : {k:.2}",
                                infos.numero,
                                p.perchoir()
                            );
                        }
                    }
                }
            }
        }
        assert!(vus >= 2, "{vus} pigeon(s) dans les niveaux 4 et 5");
    }

    /// Grêle (carte 5.4) : grêlons de 4x4 (bille de glace de 2x2), éclat de 7x3 ; chaque
    /// pixel de glace est entouré de contour sombre (aucun pixel clair ne touche le
    /// vide), et 3 couleurs de glace au plus (reflet, teinte, une ombre).
    #[test]
    fn les_grelons_sont_petits_et_cernes_de_sombre() {
        for nom in objets::GRELON {
            assert_eq!(taille(nom), (4, 4), "{nom}");
        }
        assert_eq!(taille("grele_eclat"), (7, 3));
        for nom in objets::GRELON.iter().chain(&["grele_eclat"]) {
            let d = dessin(nom);
            let mut glace = std::collections::BTreeSet::new();
            for (y, ligne) in d.iter().enumerate() {
                for (x, c) in ligne.chars().enumerate() {
                    if c == '.' || c == '0' {
                        continue;
                    }
                    glace.insert(c);
                    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        let v = car(d, x as i32 + dx, y as i32 + dy);
                        assert_ne!(v, '.', "{nom} : glace ({x}, {y}) au bord du vide");
                    }
                }
            }
            assert!(glace.len() <= 3, "{nom} : {glace:?}");
        }
    }

    /// La grêle se lit sur TOUS les ciels : sur chaque bande de ciel de chaque thème,
    /// la glace (teinte principale, NUAGE) ou le contour sombre contraste d'au moins
    /// 4,5:1. Ciels de nuit : la glace claire ; ciels corail et dorés : le contour.
    #[test]
    fn la_grele_se_detache_de_tous_les_ciels() {
        use crate::decor::ambiance;
        let glace = couleur_principale(dessin("grelon0"));
        assert_eq!(glace, '6');
        for theme in Theme::TOUS {
            for ciel in ambiance(theme).ciel {
                let k = contraste(couleur_de(glace), ciel).max(contraste(couleur_de('0'), ciel));
                assert!(k >= 4.5, "{theme:?} : grêle sur {ciel:?} : {k:.2}");
            }
        }
    }

    /// Auvent (carte 5.5) : deux images de 16x15 ; la toile n'a que deux couleurs
    /// (corail 8 et blanc cassé 7, les rayures) et chacun de ses pixels est cerné de
    /// sombre (aucun ne touche le vide) ; le reste est l'armature (contour 0, métal 3).
    /// L'image enfoncée (la toile plie, le lambrequin descend) garde le même mât.
    #[test]
    fn l_auvent_a_une_toile_rayee_cernee_de_sombre() {
        for nom in objets::AUVENT {
            assert_eq!(taille(nom), (16, 15), "{nom}");
            let d = dessin(nom);
            let mut toile = std::collections::BTreeSet::new();
            for (y, ligne) in d.iter().enumerate() {
                for (x, c) in ligne.chars().enumerate() {
                    match c {
                        '.' | '0' | '3' => {}
                        '7' | '8' => {
                            toile.insert(c);
                            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                                let v = car(d, x as i32 + dx, y as i32 + dy);
                                assert_ne!(v, '.', "{nom} : toile ({x}, {y}) au bord du vide");
                            }
                        }
                        autre => panic!("{nom} : couleur {autre:?} en ({x}, {y})"),
                    }
                }
            }
            assert_eq!(toile.len(), 2, "{nom} : rayures {toile:?}");
        }
        // Le rebond se voit : les deux images diffèrent, le mât (colonnes 0 à 2) non.
        let (a, b) = (dessin("auvent0"), dessin("auvent1"));
        assert_ne!(a, b);
        for y in 0..15 {
            assert_eq!(&a[y][..3], &b[y][..3], "mât, rangée {y}");
        }
    }

    /// L'auvent se lit sur TOUS les ciels : sur chaque bande de ciel de chaque thème,
    /// la rayure claire (blanc cassé) ou le contour sombre contraste d'au moins 4,5:1.
    #[test]
    fn l_auvent_se_detache_de_tous_les_ciels() {
        use crate::decor::ambiance;
        for theme in Theme::TOUS {
            for ciel in ambiance(theme).ciel {
                let k = contraste(couleur_de('7'), ciel).max(contraste(couleur_de('0'), ciel));
                assert!(k >= 4.5, "{theme:?} : auvent sur {ciel:?} : {k:.2}");
            }
        }
    }

    /// Le ventilateur se lit sur TOUS les ciels : sur chaque bande de ciel de chaque
    /// thème, l'une des couleurs du corps (le contour sombre, le moyeu blanc, le
    /// courant d'air cyan) contraste d'au moins 4,5:1.
    #[test]
    fn le_ventilateur_se_detache_de_tous_les_ciels() {
        use crate::decor::ambiance;
        for theme in Theme::TOUS {
            for ciel in ambiance(theme).ciel {
                let k = contraste(couleur_de('0'), ciel)
                    .max(contraste(couleur_de('7'), ciel))
                    .max(contraste(couleur_de('D'), ciel));
                assert!(k >= 4.5, "{theme:?} : ventilateur sur {ciel:?} : {k:.2}");
            }
        }
    }

    /// La montgolfière (20 x 40) : sa nacelle couvre EXACTEMENT sa boîte solide
    /// (16 x 8 : colonnes 2 à 17, rangées 32 à 39, dessus en rangée 32), cernée de
    /// sombre ; le ballon occupe les 32 rangées du dessus. Les images de la flamme
    /// ne changent que la jupe de l'enveloppe (rangées 14 à 16) : le reste ne
    /// tremble pas.
    #[test]
    fn la_montgolfiere_a_une_nacelle_qui_couvre_sa_boite() {
        let repos = dessin(objets::MONTGOLFIERE[0]);
        for nom in objets::MONTGOLFIERE {
            let m = dessin(nom);
            assert_eq!((largeur(m), hauteur(m)), (20, 40), "{nom}");
            for (y, ligne) in m.iter().enumerate() {
                let r: Vec<char> = repos[y].chars().collect();
                for (x, c) in ligne.chars().enumerate() {
                    if c != r[x] {
                        assert!((14..=16).contains(&y), "{nom} : ({x}, {y}) change");
                    }
                }
            }
        }
        // Nacelle : rangées 32 à 39, colonnes 2 à 17, coins du bas arrondis.
        for y in 32..40 {
            for x in 2..18 {
                let c = car(repos, x, y);
                let coin = y == 39 && (x == 2 || x == 17);
                assert_eq!(c == '.', coin, "nacelle en ({x}, {y}) : « {c} »");
            }
            assert_eq!(car(repos, 1, y), '.');
            assert_eq!(car(repos, 18, y), '.');
        }
        // Le dessus de la nacelle (là où Pépin se tient) est une ligne de contour
        // sombre, et ses bords aussi.
        assert!((2..18).all(|x| car(repos, x, 32) == '0'));
        assert!((32..39).all(|y| car(repos, 2, y) == '0' && car(repos, 17, y) == '0'));
    }

    /// La montgolfière se lit sur TOUS les ciels (ceux de la nuit comme le corail
    /// et l'or des niveaux du haut) : sur chaque bande de ciel de chaque thème, l'une
    /// des couleurs de l'enveloppe (contour sombre, blanc, cyan) contraste d'au moins
    /// 4,5:1. Ni l'orange ni l'or ne servent à l'enveloppe (ils se fondraient dans les
    /// ciels du soleil).
    #[test]
    fn la_montgolfiere_se_detache_de_tous_les_ciels() {
        use crate::decor::ambiance;
        for theme in Theme::TOUS {
            for ciel in ambiance(theme).ciel {
                let k = contraste(couleur_de('0'), ciel)
                    .max(contraste(couleur_de('7'), ciel))
                    .max(contraste(couleur_de('D'), ciel));
                assert!(k >= 4.5, "{theme:?} : montgolfière sur {ciel:?} : {k:.2}");
            }
        }
        let repos = dessin(objets::MONTGOLFIERE[0]);
        for ligne in &repos[..17] {
            assert!(
                !ligne.contains(['A', 'B', 'C']),
                "enveloppe sans orange ni or : {ligne}"
            );
        }
    }

    #[test]
    fn la_plateforme_mobile_couvre_toute_sa_collision() {
        let m = dessin("mobile");
        // Collision : 32 x 8 px (`LARGEUR_MOBILE` x 8 dans `core`).
        assert_eq!((largeur(m), hauteur(m)), (32, 8));
        for (y, ligne) in m.iter().enumerate() {
            assert!(
                !ligne.contains('.'),
                "rangée {y} : du vide sous la plateforme"
            );
        }
    }

    #[test]
    fn les_friables_d_ardoise_ont_la_taille_des_nuages() {
        for k in 0..2 {
            assert_eq!(
                taille(objets::FRIABLE_ARDOISE[k]),
                taille(objets::FRIABLE[k]),
                "image {k}"
            );
        }
        assert_eq!(taille("friable0"), (16, 8));
        // Clair sur sombre : le dessus de la dalle est aussi clair que celui du nuage.
        assert_eq!(dessin("friable_ardoise0")[1].chars().nth(5), Some('7'));
        assert!(dessin("friable_ardoise0")
            .iter()
            .all(|l| !l.contains(|c| "123".contains(c))));
    }

    #[test]
    fn les_bords_de_nuage_ont_la_taille_d_une_tuile_et_des_coins_arrondis() {
        for nom in ["nuage_bord_g", "nuage_bord_d", "nuage_ilot"] {
            assert_eq!(taille(nom), (16, 16), "{nom}");
        }
        let (g, d, i) = (
            dessin("nuage_bord_g"),
            dessin("nuage_bord_d"),
            dessin("nuage_ilot"),
        );
        let plein = dessin("nuage_haut");
        // Bord gauche : coins gauches creusés, côté droit inchangé.
        assert_eq!(car(g, 0, 15), '.');
        assert_eq!(car(g, 0, 3), '6');
        assert_eq!(car(g, 15, 15), car(plein, 15, 15));
        // Bord droit : l'inverse.
        assert_eq!(car(d, 15, 15), '.');
        assert_eq!(car(d, 0, 15), car(plein, 0, 15));
        // Îlot : les deux côtés.
        assert_eq!((car(i, 0, 15), car(i, 15, 15)), ('.', '.'));
    }

    /// Les variantes de brique se raccordent exactement comme la brique pleine : mêmes bords
    /// (première et dernière rangée, première et dernière colonne), seul l'intérieur change, et
    /// sur une petite zone (moins d'un quart des pixels). Aucun pixel transparent : c'est un mur.
    #[test]
    fn les_variantes_de_brique_se_raccordent_comme_la_brique_pleine() {
        let plein = dessin("brique_plein");
        for nom in ["brique_fissure", "brique_manquante", "brique_coulure"] {
            let v = dessin(nom);
            assert_eq!(taille(nom), (16, 16), "{nom}");
            let mut differences = 0;
            for y in 0..16 {
                for x in 0..16 {
                    let (a, b) = (car(v, x, y), car(plein, x, y));
                    assert_ne!(a, '.', "{nom} : trou transparent en ({x},{y})");
                    if x == 0 || x == 15 || y == 0 || y == 15 {
                        assert_eq!(a, b, "{nom} : le bord change en ({x},{y})");
                    }
                    differences += usize::from(a != b);
                }
            }
            assert!(
                (4..64).contains(&differences),
                "{nom} : {differences} pixels changés"
            );
        }
    }

    /// L'ombre du rebord : une bande pleine de la largeur d'une tuile, 1 ou 2 px de haut, faite
    /// des deux couleurs d'ombre (0 et 1) seulement.
    #[test]
    fn l_ombre_du_toit_est_une_bande_d_ombre_de_deux_pixels_au_plus() {
        let (l, h) = taille("ombre_toit");
        assert_eq!(l, 16);
        assert!((1..=2).contains(&h), "{h} px de haut");
        assert!(dessin("ombre_toit")
            .iter()
            .all(|ligne| ligne.chars().all(|c| c == '0' || c == '1')));
    }

    // --- Objets de décor (carte 4.2) ---

    /// Taille d'un objet de décor : une tuile de large au plus (il tient sur sa case) et
    /// moins de deux tuiles de haut (il ne déborde que sur la case du dessus, que les
    /// tests des niveaux gardent libre de tout danger et de tout objet).
    #[test]
    fn les_objets_de_decor_tiennent_dans_leur_case_et_celle_du_dessus() {
        for &(nom, d) in objets_decor::SPRITES {
            let (l, h) = (largeur(d), hauteur(d));
            assert!((6..=16).contains(&l), "{nom} : {l} px de large");
            assert!((8..=28).contains(&h), "{nom} : {h} px de haut");
            // Posé : la dernière rangée touche le toit (au moins un pixel plein).
            assert!(d[h - 1].chars().any(|c| c != '.'), "{nom} : pas posé");
        }
    }

    /// Contour sombre de 1 px : tout pixel coloré (autre que le contour `0`) a ses quatre
    /// voisins pleins ; seul le dessous de la dernière rangée compte comme plein (le toit).
    #[test]
    fn les_objets_de_decor_ont_un_contour_sombre() {
        for &(nom, d) in objets_decor::SPRITES {
            let (l, h) = (largeur(d) as i32, hauteur(d) as i32);
            for y in 0..h {
                for x in 0..l {
                    let c = car(d, x, y);
                    if c == '.' || c == '0' {
                        continue;
                    }
                    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        let (xx, yy) = (x + dx, y + dy);
                        let plein = if yy == h {
                            true
                        } else {
                            (0..l).contains(&xx) && (0..h).contains(&yy) && car(d, xx, yy) != '.'
                        };
                        assert!(
                            plein,
                            "{nom} : ({x},{y}) « {c} » touche le vide sans contour"
                        );
                    }
                }
            }
        }
    }

    /// Le décor ne vole aucune couleur qui a un sens dans le jeu : ni l'or des gouttes (B, C),
    /// ni le cyan des flaques et de la pluie (D, E), ni le corail du parapluie (8, 9).
    #[test]
    fn le_decor_n_utilise_pas_les_couleurs_du_jeu() {
        for &(nom, d) in objets_decor::SPRITES {
            for ligne in d {
                for c in ligne.chars() {
                    assert!(!"BCDE89".contains(c), "{nom} : couleur réservée « {c} »");
                }
            }
        }
    }

    /// Les deux images de l'oiseau gardent la même silhouette (seule la tête bouge) :
    /// au plus 8 pixels diffèrent, et aucun dans le corps (rangées 3 et suivantes).
    #[test]
    fn l_oiseau_ne_bouge_que_la_tete() {
        let (a, b) = (dessin("oiseau0"), dessin("oiseau1"));
        assert_eq!(taille("oiseau0"), taille("oiseau1"));
        let mut differences = 0;
        for y in 0..hauteur(a) as i32 {
            for x in 0..largeur(a) as i32 {
                if car(a, x, y) != car(b, x, y) {
                    differences += 1;
                    assert!(y < 4, "le corps change en ({x},{y})");
                }
            }
        }
        assert!(
            (1..=8).contains(&differences),
            "{differences} pixels changés"
        );
    }

    // --- Parapluie fermé et attente de Pépin (carte 4.3) ---

    /// Toutes les images où Pépin tient son parapluie fermé.
    const PARAPLUIE_FERME: [&str; 11] = [
        "pepin_idle0",
        "pepin_idle1",
        "pepin_course0",
        "pepin_course1",
        "pepin_course2",
        "pepin_course3",
        "pepin_saut",
        "pepin_chute",
        "pepin_cligne",
        "pepin_ciel",
        "pepin_tapote",
    ];

    /// Décalage vertical du parapluie fermé : il descend d'un pixel quand Pépin tapote le sol.
    fn decalage_parapluie(nom: &str) -> i32 {
        i32::from(nom == "pepin_tapote")
    }

    /// Le parapluie (colonnes 12 à 15, rangées 0 à 14) est le MÊME dessin dans toutes les
    /// images : sinon il tremblerait pendant la course pendant que le corps rebondit.
    #[test]
    fn le_parapluie_ferme_est_le_meme_dans_toutes_les_images() {
        let reference = dessin("pepin_idle0");
        for nom in PARAPLUIE_FERME {
            let (d, dy) = (dessin(nom), decalage_parapluie(nom));
            assert_eq!(taille(nom), (16, 16), "{nom}");
            for y in 0..15 {
                for x in 12..16 {
                    assert_eq!(
                        car(d, x, y + dy),
                        car(reference, x, y),
                        "{nom} : parapluie différent en ({x},{y})"
                    );
                }
            }
        }
    }

    /// Le parapluie fermé se lit : crosse orange en haut (crochet ouvert vers Pépin), toile
    /// corail repliée (plis en corail clair, sangle blanche), plus large en haut qu'en bas,
    /// embout de métal en bas.
    #[test]
    fn le_parapluie_ferme_a_une_crosse_une_toile_repliee_et_un_embout() {
        let d = dessin("pepin_idle0");
        // Pixels de couleur `c` du parapluie (colonnes 11 à 15) dans les rangées données.
        let compter = |c: char, rangees: std::ops::Range<i32>| {
            rangees
                .flat_map(|y| (11..16).map(move |x| (x, y)))
                .filter(|&(x, y)| car(d, x, y) == c)
                .count()
        };
        // Crosse : l'orange n'est que dans les 5 rangées du haut.
        assert!(compter('A', 0..5) >= 6, "crosse trop petite");
        assert_eq!(compter('A', 5..16), 0, "de l'orange hors de la crosse");
        // C'est un crochet : une rangée a deux bouts d'orange séparés (le bec et la tige).
        let crochet = (0..5).any(|y| {
            let oranges: Vec<i32> = (11..16).filter(|&x| car(d, x, y) == 'A').collect();
            oranges.windows(2).any(|p| p[1] - p[0] > 1)
        });
        assert!(crochet, "la crosse n'est pas recourbée");
        // Toile repliée : corail, plis en corail clair, sangle blanche.
        assert!(compter('8', 5..13) >= 8, "toile trop petite");
        assert!(compter('9', 5..13) >= 3, "pas de plis");
        let sangle = (5..13).any(|y| (12..15).all(|x| car(d, x, y) == '7'));
        assert!(sangle, "pas de sangle");
        // Plus large en haut (rangée 5) qu'en bas (rangée 12).
        let pleins = |y: i32| (12..16).filter(|&x| !".0".contains(car(d, x, y))).count();
        assert!(pleins(5) > pleins(12), "la toile ne s'affine pas");
        // Embout de métal (gris `5`) sous la toile.
        assert_eq!(compter('5', 12..16), 1, "embout");
    }

    /// Contour sombre de 1 px autour du parapluie dans toutes les images : tout pixel coloré
    /// des colonnes 11 à 15 a ses quatre voisins pleins.
    #[test]
    fn le_parapluie_ferme_a_un_contour_sombre() {
        for nom in PARAPLUIE_FERME {
            let d = dessin(nom);
            for y in 0..16 {
                for x in 11..16 {
                    if ".0".contains(car(d, x, y)) {
                        continue;
                    }
                    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        assert_ne!(
                            car(d, x + dx, y + dy),
                            '.',
                            "{nom} : ({x},{y}) touche le vide sans contour"
                        );
                    }
                }
            }
        }
    }

    /// Les images d'attente gardent la silhouette du repos (`pepin_idle0`) : on passe de l'une
    /// à l'autre sans que Pépin saute. Cligner et regarder le ciel ne changent que les yeux et
    /// les joues (rangées 4 à 8, aucun pixel ajouté ni retiré) ; tapoter ne bouge que le
    /// parapluie (colonnes 10 et suivantes).
    #[test]
    fn l_attente_ne_change_que_les_yeux_ou_le_parapluie() {
        let repos = dessin("pepin_idle0");
        for nom in pepin::ATTENTE {
            let d = dessin(nom);
            assert_eq!(taille(nom), taille("pepin_idle0"), "{nom}");
            let mut differences = 0;
            for y in 0..16 {
                for x in 0..16 {
                    let (a, b) = (car(repos, x, y), car(d, x, y));
                    if a == b {
                        continue;
                    }
                    differences += 1;
                    if nom == "pepin_tapote" {
                        assert!(x >= 10, "{nom} : le corps change en ({x},{y})");
                    } else {
                        assert!(
                            (4..9).contains(&y) && a != '.' && b != '.',
                            "{nom} : changement hors du visage en ({x},{y})"
                        );
                    }
                }
            }
            assert!(differences > 0, "{nom} : identique au repos");
        }
    }
}
