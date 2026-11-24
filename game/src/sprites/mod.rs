//! Sprites en pixel art. Depuis la carte 10.4a, chaque dessin fait à la main est un
//! fichier PNG du dépôt (`assets/sprites/<famille>/<clé>.png`, voir
//! `assets/sprites/LISEZMOI.md`), embarqué dans le jeu à la compilation et décodé par
//! `fichiers`. Les modules `pepin`, `ennemis`... gardent les NOMS (animations, jeux
//! d'images) et les commentaires de conception de chaque dessin.
//!
//! Une fois décodé, un dessin est une grille de texte : une ligne = une rangée de
//! pixels, 1 caractère = 1 pixel. `.` = transparent ; `0`-`9` puis `A`-`F` = indice
//! dans `pepin_core::palette::TOUTES` (0 = ombre de contour, 8 = corail du
//! parapluie, B = or, D = cyan de la pluie, etc.).
//!
//! Au démarrage, `Banque::charger` transforme chaque grille en `Texture2D`
//! (filtre « nearest » : jamais de flou). Le décodage lui-même (`pixel`,
//! `verifier`) est pur : il ne dépend pas du moteur graphique et se teste.
//!
//! Notions Rust illustrées ici : alias de type (`Image`), `Result` avec un
//! `char` fautif comme erreur, itérateurs, `HashMap`, tests unitaires.

pub mod ennemis;
pub mod ennemis_eau;
pub mod fichiers;
pub mod jetee;
pub mod objets;
pub mod objets_decor;
pub mod objets_eau;
pub mod pepin;
pub mod pepinette;
pub mod pieuvre;
#[cfg(test)]
mod planche;
pub mod tuiles;
pub mod tuiles_monde2;

use std::cell::Cell;
use std::collections::HashMap;

use macroquad::prelude::{
    DrawTextureParams, FilterMode, Image as ImageMq, Texture2D, BLANK, WHITE,
};
use pepin_core::palette::{Couleur, TOUTES};
use pepin_core::zones::Theme;

use crate::cosmetiques::{appliquer_motif, Motif};
use crate::decor::ambiance;
use crate::rendu::couleur_mq;
use crate::skins::Skin;
pub use fichiers::dessin;

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

/// Le jeu d'images d'un personnage (un skin, voir `crate::skins`) : les NOMS de ses
/// sprites, rangés par animation. Le rendu (`scene`, `hud`) ne connaît que ces champs :
/// changer de skin = changer de jeu d'images, la logique d'animation reste la même.
pub struct JeuImages {
    /// Repos au sol (respiration, 2 images).
    pub repos: [&'static str; 2],
    /// Course (4 images en boucle).
    pub course: [&'static str; 4],
    /// Montée d'un saut.
    pub saut: &'static str,
    /// Chute.
    pub chute: &'static str,
    /// Attente après 2 s immobile : cligner, regarder le ciel, tapoter le sol.
    pub attente: [&'static str; 3],
    /// Parapluie ouvert (20x20, les franges oscillent).
    pub planer: [&'static str; 2],
    /// Parapluie retourné SOUS le personnage, debout dans la coque de la barque
    /// (carte 9.2b, BAS tenu ; dessin sous les pieds depuis la 9.2c ; 16x16).
    pub retourne: &'static str,
    /// Barque (carte 9.2) : le parapluie retourné avec le personnage assis dedans.
    pub barque: &'static str,
    /// Réapparition après une mort (étincelles qui se rassemblent).
    pub reapparition: [&'static str; 3],
    /// Icône des vies du HUD (10x10).
    pub vie: &'static str,
}

impl JeuImages {
    #[cfg(test)]
    /// Tous les noms du jeu (18 images du personnage, puis l'icône des vies).
    pub fn noms(&self) -> Vec<&'static str> {
        let mut noms = Vec::new();
        noms.extend(self.repos);
        noms.extend(self.course);
        noms.extend([self.saut, self.chute]);
        noms.extend(self.attente);
        noms.extend(self.planer);
        noms.push(self.retourne);
        noms.push(self.barque);
        noms.extend(self.reapparition);
        noms.push(self.vie);
        noms
    }
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
    fichiers::FAMILLES
        .iter()
        .flat_map(|f| fichiers::famille(f).iter().copied())
        .collect()
}

/// Les sprites de tous les skins du catalogue (Pépin, Pépinette…), dans son ordre.
fn personnages() -> Vec<(&'static str, Image)> {
    Skin::TOUS
        .iter()
        .flat_map(|s| s.sprites().iter().copied())
        .collect()
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
        // Monde 2 (carte 9.5) : les briques mouillées virent au cyan sombre.
        'F' if teinte.briques_mouillees => 'E',
        _ => c,
    }
}

/// Convertit une grille en texture « nearest ». `theme` : substitution de couleurs.
fn texture(image: &[&str], theme: Option<Theme>) -> Texture2D {
    texture_tramee(image, theme, false)
}

/// Comme `texture` ; `trame` : un pixel sur deux en damier reste transparent (le faux mur
/// révélé de la carte 8.5 : « translucide » sans aucune demi-teinte).
fn texture_tramee(image: &[&str], theme: Option<Theme>, trame: bool) -> Texture2D {
    let l = image.first().map_or(0, |l| l.chars().count());
    let mut img = ImageMq::gen_image_color(l as u16, image.len() as u16, BLANK);
    for (y, ligne) in image.iter().enumerate() {
        for (x, c) in ligne.chars().enumerate() {
            if trame && (x + y) % 2 == 1 {
                continue;
            }
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
    /// Les mêmes tuiles tramées en damier (faux murs révélés, carte 8.5).
    tuiles_tramees: Vec<HashMap<&'static str, Texture2D>>,
    /// Les images des skins avec un motif sur le parapluie (carte 7.6), par motif.
    motifs: HashMap<(Motif, &'static str), Texture2D>,
    /// Le motif en service : `dessiner` prend alors ces images-là pour Pépin, sur
    /// tous les écrans (jeu, titre, carte des niveaux, histoire).
    motif: Cell<Option<Motif>>,
    /// Les images du Grand Nuage tramées en damier : le nuage « pâle » du mode facile
    /// (carte 8.14), sans aucune demi-teinte hors palette.
    nuages_trames: HashMap<&'static str, Texture2D>,
    /// Les personnages tramés en damier : le fantôme de la course (carte 8.7).
    personnages_trames: HashMap<&'static str, Texture2D>,
    /// Les personnages tout en blanc cassé : le flash d'un dégât (carte 9.24).
    personnages_blancs: HashMap<&'static str, Texture2D>,
}

impl Banque {
    /// Génère toutes les textures. À appeler une seule fois, après l'ouverture de la fenêtre.
    pub fn charger() -> Self {
        let sprites = personnages()
            .into_iter()
            .chain(
                [
                    ennemis::sprites(),
                    ennemis_eau::sprites(),
                    pieuvre::sprites(),
                    objets::sprites(),
                    objets_decor::sprites(),
                    objets_eau::sprites(),
                    jetee::sprites(),
                ]
                .concat(),
            )
            .map(|(nom, dessin)| (nom, texture(dessin, None)))
            .collect();
        let jeu = |trame: bool| -> Vec<HashMap<&'static str, Texture2D>> {
            Theme::TOUS
                .iter()
                .map(|&t| {
                    // Les tuiles du Monde 1 prennent la teinte du thème ; celles d'un
                    // jeu du Monde 2 (carte 9.6b) se dessinent telles quelles.
                    let monde_1 = tuiles::sprites()
                        .iter()
                        .map(|&(nom, dessin)| (nom, texture_tramee(dessin, Some(t), trame)));
                    let monde_2 = tuiles_monde2::JEUX
                        .iter()
                        .flat_map(|jeu| fichiers::famille(jeu).iter())
                        .map(|&(nom, dessin)| (nom, texture_tramee(dessin, None, trame)));
                    monde_1.chain(monde_2).collect()
                })
                .collect()
        };
        let mut motifs = HashMap::new();
        for skin in Skin::TOUS {
            for motif in Motif::TOUS {
                for &(nom, dessin) in skin.sprites() {
                    let avec = appliquer_motif(dessin, skin, motif);
                    let lignes: Vec<&str> = avec.iter().map(String::as_str).collect();
                    motifs.insert((motif, nom), texture(&lignes, None));
                }
            }
        }
        // Le boss du mode facile, plus pâle : le Grand Nuage et la tête de la Pieuvre.
        let nuages_trames = [ennemis::sprites(), pieuvre::sprites()]
            .concat()
            .iter()
            .filter(|(nom, _)| nom.starts_with("nuage_") || nom.starts_with("pieuvre_"))
            .map(|&(nom, dessin)| (nom, texture_tramee(dessin, None, true)))
            .collect();
        let personnages_trames = personnages()
            .into_iter()
            .map(|(nom, dessin)| (nom, texture_tramee(dessin, None, true)))
            .collect();
        // Silhouettes blanches : chaque pixel du dessin passe au blanc cassé (`7`).
        let personnages_blancs = personnages()
            .into_iter()
            .map(|(nom, dessin)| {
                let blanc: Vec<String> = dessin
                    .iter()
                    .map(|l| l.chars().map(|c| if c == '.' { c } else { '7' }).collect())
                    .collect();
                let lignes: Vec<&str> = blanc.iter().map(String::as_str).collect();
                (nom, texture(&lignes, None))
            })
            .collect();
        Self {
            sprites,
            tuiles: jeu(false),
            tuiles_tramees: jeu(true),
            motifs,
            motif: Cell::new(None),
            nuages_trames,
            personnages_trames,
            personnages_blancs,
        }
    }

    /// Le motif de parapluie en service (`None` : parapluie uni).
    pub fn choisir_motif(&self, motif: Option<Motif>) {
        self.motif.set(motif);
    }

    /// Comme `dessiner`, avec un motif donné (aperçus de la boutique).
    pub fn dessiner_avec_motif(&self, nom: &str, motif: Option<Motif>, x: f32, y: f32) {
        let t = motif.and_then(|m| self.motifs.get(&(m, nom)));
        if let Some(t) = t.or_else(|| self.sprites.get(nom)) {
            Self::poser(t, x, y, false);
        }
    }

    /// Dessine le sprite `nom` avec son coin haut-gauche en `(x, y)` (arrondi au pixel).
    /// `miroir` : retourne l'image horizontalement (Pépin qui regarde à gauche).
    pub fn dessiner(&self, nom: &str, x: f32, y: f32, miroir: bool) {
        let avec_motif = self.motif.get().and_then(|m| self.motifs.get(&(m, nom)));
        if let Some(t) = avec_motif.or_else(|| self.sprites.get(nom)) {
            Self::poser(t, x, y, miroir);
        }
    }

    /// Dessine l'image `nom` du Grand Nuage tramée en damier (un pixel sur deux) :
    /// le nuage du mode facile, plus pâle sur le ciel (carte 8.14).
    pub fn dessiner_pale(&self, nom: &str, x: f32, y: f32) {
        if let Some(t) = self.nuages_trames.get(nom) {
            Self::poser(t, x, y, false);
        }
    }

    /// Dessine la tuile `nom` du thème donné.
    pub fn dessiner_tuile(&self, theme: Theme, nom: &str, x: f32, y: f32) {
        if let Some(t) = self.tuiles[theme.indice()].get(nom) {
            Self::poser(t, x, y, false);
        }
    }

    /// Dessine la tuile `nom` tramée en damier (faux mur révélé, carte 8.5).
    pub fn dessiner_tuile_tramee(&self, theme: Theme, nom: &str, x: f32, y: f32) {
        if let Some(t) = self.tuiles_tramees[theme.indice()].get(nom) {
            Self::poser(t, x, y, false);
        }
    }

    /// Dessine le personnage `nom` tramé en damier : le fantôme de la course contre le
    /// robot (carte 8.7), « translucide » sans demi-teinte.
    pub fn dessiner_trame(&self, nom: &str, x: f32, y: f32, miroir: bool) {
        if let Some(t) = self.personnages_trames.get(nom) {
            Self::poser(t, x, y, miroir);
        }
    }

    /// Le personnage `nom` en silhouette blanche : le flash d'un dégât (carte 9.24).
    pub fn dessiner_blanc(&self, nom: &str, x: f32, y: f32, miroir: bool) {
        if let Some(t) = self.personnages_blancs.get(nom) {
            Self::poser(t, x, y, miroir);
        }
    }

    /// Animation de mort (carte 9.24) : le rectangle `source` (x, y, largeur, hauteur,
    /// en pixels de l'image) du sprite `nom`, avec le motif en service, posé en
    /// `(x, y)` et tourné de `quarts` quarts de tour autour de son centre (des
    /// quarts seulement : les pixels restent sur la grille).
    pub fn dessiner_partie(
        &self,
        nom: &str,
        (x, y): (f32, f32),
        source: (f32, f32, f32, f32),
        quarts: u8,
        miroir: bool,
    ) {
        let avec_motif = self.motif.get().and_then(|m| self.motifs.get(&(m, nom)));
        let Some(t) = avec_motif.or_else(|| self.sprites.get(nom)) else {
            return;
        };
        let (sx, sy, l, h) = source;
        if l <= 0.0 || h <= 0.0 {
            return;
        }
        macroquad::prelude::draw_texture_ex(
            t,
            x.round(),
            y.round(),
            WHITE,
            DrawTextureParams {
                source: Some(macroquad::prelude::Rect::new(sx, sy, l, h.round())),
                rotation: f32::from(quarts % 4) * std::f32::consts::FRAC_PI_2,
                flip_x: miroir,
                ..Default::default()
            },
        );
    }

    /// Dessine seulement les `hauteur` premières rangées de la tuile `nom` du thème
    /// (le reste est sous l'eau) : la plateforme que la Pieuvre emporte (carte 9.12f).
    pub fn dessiner_tuile_haut(&self, theme: Theme, nom: &str, x: f32, y: f32, hauteur: f32) {
        if let Some(t) = self.tuiles[theme.indice()].get(nom) {
            Self::poser_haut(t, x, y, hauteur);
        }
    }

    /// Dessine seulement les `hauteur` premières rangées du sprite `nom` (le reste est
    /// sous l'eau) : la tête de la Pieuvre qui replonge, un tentacule qui redescend
    /// (carte 9.12a). Un découpage de l'image, pas une mise à l'échelle.
    pub fn dessiner_haut(&self, nom: &str, x: f32, y: f32, hauteur: f32) {
        let Some(t) = self.sprites.get(nom) else {
            return;
        };
        Self::poser_haut(t, x, y, hauteur);
    }

    fn poser_haut(t: &Texture2D, x: f32, y: f32, hauteur: f32) {
        let h = hauteur.round().clamp(0.0, t.height());
        if h <= 0.0 {
            return;
        }
        macroquad::prelude::draw_texture_ex(
            t,
            x.round(),
            y.round(),
            WHITE,
            DrawTextureParams {
                source: Some(macroquad::prelude::Rect::new(0.0, 0.0, t.width(), h)),
                ..Default::default()
            },
        );
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
            &ennemis::CHAUVE_SOURIS,
            &ennemis::PIGEON,
            &ennemis::ESCARGOT,
            &ennemis::CHAT_DORT,
            &objets::GOUTTE,
            &objets::LANTERNE,
            &objets::CHAMPIGNON,
            &objets::FRIABLE,
            &objets::FRIABLE_ARDOISE,
            &objets::DRAPEAU,
            &objets::GRELON,
            &objets::GRELE_NUAGE,
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
        // Monde 2 (carte 9.5) : les briques mouillées virent au cyan sombre ('F'
        // devient 'E'), aux égouts et au port ; ailleurs elles restent violettes.
        assert_eq!(remplacer('F', Theme::Egouts), 'E');
        assert_eq!(remplacer('F', Theme::Port), 'E');
        assert_eq!(remplacer('F', Theme::Toits), 'F');
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
                "{nom} : l'orange (l'œil, le bec avant la carte 6.1) a disparu"
            );
            assert!(
                c.iter().any(|l| l.contains('0')),
                "{nom} : le contour a disparu"
            );
        }
        assert!(contraste(couleur_de('4'), pepin_core::palette::ARDOISE) >= 3.0);
        assert!(contraste(couleur_de('3'), pepin_core::palette::ARDOISE) < 3.0);
    }

    /// Carte 9.26b : la chauve-souris (le corbeau du Monde 2) a la taille du corbeau
    /// (même boîte, même placement), un contour sombre de 1 px (aucun pixel coloré ne
    /// touche le vide), des yeux rouges (`8`), un corps violet (`F`, couleur la plus
    /// fréquente), un liseré clair (`4`) qui se détache des fonds sombres de la grotte
    /// et des égouts, des ailes qui battent (les deux images diffèrent) ; symétrique.
    #[test]
    fn la_chauve_souris_du_monde_2_se_detache_de_la_grotte() {
        for nom in ennemis::CHAUVE_SOURIS {
            assert_eq!(taille(nom), (18, 14), "{nom}");
            let d = dessin(nom);
            for (x, y, c) in pixels_colores(d) {
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    assert_ne!(
                        car(d, x + dx, y + dy),
                        '.',
                        "{nom} : ({x}, {y}) `{c}` touche le vide (contour)"
                    );
                }
            }
            let compte = |c: char| d.iter().map(|l| l.matches(c).count()).sum::<usize>();
            assert_eq!(compte('8'), 2, "{nom} : deux yeux rouges");
            assert!(compte('4') >= 8, "{nom} : le liseré clair des ailes");
            assert!(compte('F') > compte('1'), "{nom} : corps violet");
            assert!(
                d.iter().all(|l| l.chars().all(|c| ".0148F".contains(c))),
                "{nom} : couleurs inattendues"
            );
            // Symétrique : le miroir du rendu ne la change pas.
            for l in d.iter() {
                assert_eq!(l.chars().rev().collect::<String>(), *l, "{nom}");
            }
        }
        let (a, b) = (dessin("chauve_souris0"), dessin("chauve_souris1"));
        assert_ne!(a, b, "les ailes battent");
        // Le liseré clair se voit sur l'ombre de la grotte et le fond des égouts.
        for fond in [
            pepin_core::palette::ARDOISE_OMBRE,
            pepin_core::palette::ARDOISE_NUIT,
        ] {
            assert!(contraste(couleur_de('4'), fond) >= 3.0);
        }
    }

    /// Carte 6.1 : le corbeau se lit DE PROFIL, jamais comme une chauve-souris (corps
    /// rond entre deux ailes en V, rien qui dise où est l'avant). Ce qui est vérifiable
    /// simplement, sur les deux images :
    /// - contour sombre de 1 px : aucun pixel coloré ne touche le vide ;
    /// - le bec (gris clair `5`/`6`, au moins 5 px) n'existe QUE devant (x >= 13), et
    ///   c'est lui qui pointe le plus loin : le pixel coloré le plus à droite est du bec ;
    /// - un seul œil (`A`, orange), dans la tête, devant (x >= 12), au-dessus de la
    ///   pointe du bec ;
    /// - plumage sombre : le bleu nuit `1` est la couleur la plus fréquente, avec des
    ///   reflets `4` (ceux qui se voient sur le ciel d'ardoise, test précédent) ;
    /// - assez de pixels clairs (reflets, bec, œil) pour se détacher du ciel d'ardoise
    ///   de l'orage, et un plumage qui se détache des bandes claires des vents ;
    /// - rien du pigeon (cou irisé `D`/`E`) ni de l'oiseau de décor (blanc `7`) ;
    /// - la tête et le bec (colonnes 12 et suivantes) sont les mêmes sur les deux
    ///   images : seule l'aile bat.
    #[test]
    fn le_corbeau_est_un_oiseau_de_profil_et_pas_une_chauve_souris() {
        for nom in ennemis::CORBEAU {
            assert_eq!(taille(nom), (18, 14), "{nom}");
            let d = dessin(nom);
            let mut bec = 0;
            let mut yeux = Vec::new();
            // Pixel coloré le plus à droite : (x, caractère, y).
            let mut avant = (0, '.', 0);
            let mut compte = std::collections::BTreeMap::new();
            for y in 0..14 {
                for x in 0..18 {
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
                    }
                    assert!(!"27DE".contains(c), "{nom} : « {c} » en ({x},{y})");
                    *compte.entry(c).or_insert(0) += 1;
                    if c == '5' || c == '6' {
                        bec += 1;
                        assert!(x >= 13, "{nom} : du bec en ({x},{y}), derrière la tête");
                    }
                    if c == 'A' {
                        yeux.push((x, y));
                    }
                    if x > avant.0 {
                        avant = (x, c, y);
                    }
                }
            }
            assert!(bec >= 5, "{nom} : bec de {bec} px seulement");
            assert_eq!(
                avant.1, '5',
                "{nom} : le bec ne pointe pas devant ({avant:?})"
            );
            assert_eq!(yeux.len(), 1, "{nom} : yeux {yeux:?}");
            let (ox, oy) = yeux[0];
            assert!(ox >= 12 && oy < avant.2, "{nom} : œil en ({ox},{oy})");
            let plus_frequente = compte.iter().max_by_key(|(_, n)| **n).map(|(c, _)| *c);
            assert_eq!(plus_frequente, Some('1'), "{nom} : plumage {compte:?}");
            assert!(compte.contains_key(&'4'), "{nom} : aucun reflet clair");
            // Sur le ciel d'ardoise du grand orage, le plumage sombre ne se détache pas
            // (contraste 1,4) : ce sont les reflets, le bec et l'œil qui le font
            // (contraste 3,7 à 7,4). Il en faut assez pour dessiner la silhouette.
            let clairs: usize = compte
                .iter()
                .filter(|(c, _)| contraste(couleur_de(**c), pepin_core::palette::ARDOISE) >= 3.0)
                .map(|(_, n)| *n)
                .sum();
            assert!(clairs >= 15, "{nom} : {clairs} px clairs seulement");
        }
        // Sur les bandes claires des vents d'altitude (corail, corail clair, orange),
        // c'est le plumage sombre qui se détache (l'ancien corbeau clair s'y perdait :
        // contraste 1,0 sur le corail).
        for ciel in [
            pepin_core::palette::CORAIL,
            pepin_core::palette::CORAIL_CLAIR,
            pepin_core::palette::ORANGE_SOLEIL,
        ] {
            assert!(contraste(couleur_de('1'), ciel) >= 5.0, "{ciel:?}");
        }
        let (a, b) = (dessin("corbeau0"), dessin("corbeau1"));
        assert_ne!(a, b);
        for (y, (la, lb)) in a.iter().zip(b.iter()).enumerate() {
            assert_eq!(
                &la[12..],
                &lb[12..],
                "rangée {y} : la tête doit rester immobile"
            );
        }
    }

    // --- Escargot (carte 5.8) ---

    /// Les trois images de l'escargot : marche (2), rebond.
    const ESCARGOT_TOUT: [&str; 3] = ["escargot0", "escargot1", "escargot_rebond"];

    /// L'escargot (16 x 12) : cerné d'un contour sombre (aucun pixel coloré ne
    /// touche le vide), posé sur sa boîte 16 x 10 (dernière rangée : contour, sans
    /// rien dessous). La marche ne change que les cornes (2 rangées du haut).
    #[test]
    fn l_escargot_a_trois_images_et_un_contour_sombre() {
        for nom in ESCARGOT_TOUT {
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
                    }
                }
            }
            assert!(d[11].chars().all(|c| c == '0' || c == '.'), "{nom}");
        }
        let (a, b) = (dessin("escargot0"), dessin("escargot1"));
        assert_ne!(a, b);
        assert_eq!(&a[2..], &b[2..], "seules les cornes bougent");
    }

    /// Sa coquille est celle du champignon : mêmes couleurs (corail 8, corail clair
    /// 9, pois blancs 7), et le corail domine : on devine qu'il fait rebondir.
    #[test]
    fn la_coquille_de_l_escargot_est_un_chapeau_de_champignon() {
        let couleurs = |nom: &str| -> std::collections::BTreeSet<char> {
            dessin(nom).iter().flat_map(|l| l.chars()).collect()
        };
        let chapeau = couleurs("champignon0");
        for c in ['7', '8', '9'] {
            assert!(chapeau.contains(&c), "champignon : {c}");
            for nom in ESCARGOT_TOUT {
                assert!(couleurs(nom).contains(&c), "{nom} : {c}");
            }
        }
        // Dans la coquille (et le chapeau), le corail domine nettement.
        for nom in ["escargot0", "champignon0"] {
            let compte = |c: char| {
                dessin(nom)
                    .iter()
                    .flat_map(|l| l.chars())
                    .filter(|&x| x == c)
                    .count()
            };
            assert!(compte('8') > 2 * (compte('9') + compte('7')), "{nom}");
        }
    }

    /// L'escargot se lit sur TOUS les ciels : sur chaque bande de ciel de chaque
    /// thème, le contour sombre ou le corps clair (nuage 6) contraste d'au moins
    /// 4,5:1.
    #[test]
    fn l_escargot_se_detache_de_tous_les_ciels() {
        use crate::decor::ambiance;
        for theme in Theme::TOUS {
            for ciel in ambiance(theme).ciel {
                let k = contraste(couleur_de('0'), ciel).max(contraste(couleur_de('6'), ciel));
                assert!(k >= 4.5, "{theme:?} : escargot sur {ciel:?} : {k:.2}");
            }
        }
    }

    // --- Chat endormi (carte 5.9) ---

    /// Les cinq images du chat : dort (2), réveil, éveillé, écrasé.
    const CHAT_TOUT: [&str; 7] = [
        "chat_dort0",
        "chat_dort1",
        "chat_reveil",
        "chat_eveille",
        "chat_ecrase",
        "chat_pret",
        "chat_bond",
    ];

    /// Tout pixel coloré (ni vide ni contour) a ses quatre voisins dans le dessin
    /// et opaques : le dessin est cerné d'un contour sombre.
    fn verifier_contour(nom: &str) {
        let d = dessin(nom);
        let (l, h) = (largeur(d) as i32, hauteur(d) as i32);
        for y in 0..h {
            for x in 0..l {
                let c = car(d, x, y);
                if c == '.' || c == '0' {
                    continue;
                }
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    assert!((0..l).contains(&(x + dx)) && (0..h).contains(&(y + dy)));
                    assert_ne!(
                        car(d, x + dx, y + dy),
                        '.',
                        "{nom} : ({x},{y}) « {c} » touche le vide sans contour"
                    );
                }
            }
        }
    }

    #[test]
    fn le_chat_a_sept_images_de_meme_taille_et_un_contour_sombre() {
        for nom in CHAT_TOUT {
            assert_eq!(taille(nom), (16, 14), "{nom}");
            verifier_contour(nom);
            // Posé sur sa boîte 14 x 8 (colonnes 1 à 14, bas du dessin) : la
            // dernière rangée est une ligne de contour sombre, sans rien dessous.
            let d = dessin(nom);
            assert!(d[13].chars().all(|c| c == '0' || c == '.'), "{nom}");
        }
        // Endormi, il respire : seul le dos bouge, le bas du corps (rangées 10 à 13) non.
        let (a, b) = (dessin("chat_dort0"), dessin("chat_dort1"));
        assert_ne!(a, b);
        assert_eq!(&a[10..], &b[10..]);
        // Les indices (« z » du sommeil, « ! » du réveil) aussi sont cernés.
        assert_eq!(taille("chat_z"), (6, 6));
        assert_eq!(taille("chat_alerte"), (4, 8));
        verifier_contour("chat_z");
        verifier_contour("chat_alerte");
    }

    /// Endormi, il a les yeux fermés (aucun pixel d'or) ; éveillé, ses yeux sont
    /// d'or : la différence se voit d'un coup d'œil.
    #[test]
    fn le_chat_eveille_a_les_yeux_ouverts() {
        let or = |nom: &str| {
            dessin(nom)
                .iter()
                .flat_map(|l| l.chars())
                .filter(|&c| c == 'B')
                .count()
        };
        assert_eq!(or("chat_dort0"), 0);
        assert_eq!(or("chat_dort1"), 0);
        assert_eq!(
            or("chat_ecrase"),
            0,
            "écrasé, il ferme les yeux (il reste mignon)"
        );
        assert_eq!(or("chat_reveil"), 1, "un œil ouvert");
        assert_eq!(or("chat_eveille"), 2, "les deux yeux");
        // Carte 6.3 : prêt à bondir et en plein bond, les deux yeux grands ouverts.
        assert_eq!(or("chat_pret"), 2);
        assert_eq!(or("chat_bond"), 2);
    }

    /// Carte 6.3 : les deux nouvelles postures se distinguent au premier coup
    /// d'œil. Prêt à bondir : l'arrière-train (colonnes 9 à 15) monte plus haut
    /// que celui du chat qui s'annonce simplement, et la tête est plus basse. En
    /// plein bond : les pattes tendues atteignent les deux bords du dessin (à
    /// gauche les pattes avant, à droite les pattes arrière) dans le bas.
    #[test]
    fn pret_a_bondir_et_en_plein_bond_se_reconnaissent() {
        // Première rangée où la zone (colonnes `de..a`) a un pixel opaque.
        let haut = |nom: &str, de: usize, a: usize| {
            dessin(nom)
                .iter()
                .position(|l| l.chars().skip(de).take(a - de).any(|c| c != '.'))
                .unwrap()
        };
        assert!(
            haut("chat_pret", 9, 16) + 3 <= haut("chat_reveil", 9, 16),
            "croupion levé"
        );
        assert!(
            haut("chat_pret", 0, 9) > haut("chat_reveil", 0, 9),
            "tête basse"
        );
        assert_ne!(dessin("chat_pret"), dessin("chat_reveil"));
        let bas = &dessin("chat_bond")[10..];
        let colonnes = |l: &&str| {
            l.char_indices()
                .filter(|&(_, c)| c != '.')
                .map(|(i, _)| i)
                .collect::<Vec<_>>()
        };
        let tout: Vec<usize> = bas.iter().flat_map(colonnes).collect();
        assert!(tout.iter().any(|&i| i <= 2), "pattes avant tendues");
        assert!(tout.iter().any(|&i| i >= 14), "pattes arrière tendues");
    }

    /// Le chat et ses indices se lisent sur TOUS les ciels : sur chaque bande de ciel
    /// de chaque thème, le contour sombre ou le blanc (museau, pattes, « z »)
    /// contraste d'au moins 4,5:1.
    #[test]
    fn le_chat_se_detache_de_tous_les_ciels() {
        use crate::decor::ambiance;
        for theme in Theme::TOUS {
            for ciel in ambiance(theme).ciel {
                let k = contraste(couleur_de('0'), ciel).max(contraste(couleur_de('7'), ciel));
                assert!(k >= 4.5, "{theme:?} : chat sur {ciel:?} : {k:.2}");
            }
        }
    }

    // --- Pigeon (carte 4.4, redessiné carte 6.6) ---

    /// Les huit images du pigeon : perché (2), annonce, plongée (2 : l'élan, le
    /// piqué), remontée (2 : ailes levées, ailes basses), écrasé.
    const PIGEON_TOUT: [&str; 8] = [
        "pigeon0",
        "pigeon1",
        "pigeon_annonce",
        "pigeon_plongee0",
        "pigeon_plongee1",
        "pigeon_remontee0",
        "pigeon_remontee1",
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

    /// Pixels colorés d'un dessin (contour `0` et vide exclus) : `(x, y, caractère)`.
    fn pixels_colores(d: Image) -> Vec<(i32, i32, char)> {
        let mut v = Vec::new();
        for (y, ligne) in d.iter().enumerate() {
            for (x, c) in ligne.chars().enumerate() {
                if c != '.' && c != '0' {
                    v.push((x as i32, y as i32, c));
                }
            }
        }
        v
    }

    /// Carte 6.6 : 18 x 14 (au lieu de 16 x 12), même taille pour les huit images
    /// (sinon il « sauterait » d'une image à l'autre), contour sombre de 1 px
    /// (aucun pixel coloré ne touche le vide ni le bord du dessin), cou irisé
    /// (sarcelle `E` et cyan `D`) sur chaque image.
    #[test]
    fn le_pigeon_a_huit_images_de_meme_taille_et_un_contour_sombre() {
        for nom in PIGEON_TOUT {
            assert_eq!(taille(nom), (18, 14), "{nom}");
            let d = dessin(nom);
            for (x, y, c) in pixels_colores(d) {
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    assert_ne!(
                        car(d, x + dx, y + dy),
                        '.',
                        "{nom} : ({x},{y}) « {c} » touche le vide sans contour"
                    );
                    assert!((0..18).contains(&(x + dx)) && (0..14).contains(&(y + dy)));
                }
            }
            assert!(
                d.iter().any(|l| l.contains('E')) && d.iter().any(|l| l.contains('D')),
                "{nom} : pas de cou irisé"
            );
        }
        // Perché, il ne fait que hocher la tête : sous la tête (rangées 5 et
        // suivantes), le corps, l'aile et les pattes ne bougent pas.
        let (a, b) = (dessin("pigeon0"), dessin("pigeon1"));
        assert_eq!(&a[5..], &b[5..]);
        assert_ne!(a, b);
        // Remontée : deux images différentes (le battement d'ailes) ; ailes levées
        // (de l'aile claire `5` dans les 3 rangées du haut), puis rabattues (de l'aile
        // claire sous le ventre, rangées 10 et suivantes).
        let aile = |nom: &str, rangees: std::ops::Range<i32>| {
            pixels_colores(dessin(nom))
                .iter()
                .filter(|&&(_, y, c)| c == '5' && rangees.contains(&y))
                .count()
        };
        assert!(aile("pigeon_remontee0", 0..3) >= 3);
        assert_eq!(aile("pigeon_remontee1", 0..3), 0);
        assert!(aile("pigeon_remontee1", 10..14) >= 6);
        assert_eq!(aile("pigeon_remontee0", 10..14), 0);
        // Écrasé : aplati, rien dans les 6 rangées du haut.
        assert!(pixels_colores(dessin("pigeon_ecrase"))
            .iter()
            .all(|&(_, y, _)| y >= 6));
    }

    /// Carte 6.6, priorité d'AGA (« surtout quand ils piquent ») : en piqué, le
    /// pigeon se lit comme un oiseau qui fond sur Pépin le long de sa diagonale
    /// (vers le bas et vers l'avant, à 45°). Le dessin regarde à droite (la scène le
    /// retourne) ; sur les deux images de la plongée :
    /// - la masse des pixels colorés s'allonge vers le bas à droite : son axe
    ///   principal (covariance des positions) penche entre 15° et 75° sous
    ///   l'horizontale ;
    /// - devant, le long de la diagonale (le plus grand `x + y`), c'est le bec
    ///   (gris clair `5`), à 4 px au plus de l'œil ;
    /// - un seul œil, rouge corail (`8`, l'œil vif de l'annonce), dans le quart en bas
    ///   à droite ;
    /// - derrière (le plus petit `x + y`), ce sont les rémiges sombres (`1`), dans
    ///   le coin en haut à gauche.
    #[test]
    fn le_pigeon_en_pique_fond_sur_pepin_le_long_de_sa_diagonale() {
        for nom in ["pigeon_plongee0", "pigeon_plongee1"] {
            let px = pixels_colores(dessin(nom));
            let n = px.len() as f32;
            let mx = px.iter().map(|p| p.0 as f32).sum::<f32>() / n;
            let my = px.iter().map(|p| p.1 as f32).sum::<f32>() / n;
            let (mut sxx, mut syy, mut sxy) = (0.0, 0.0, 0.0);
            for &(x, y, _) in &px {
                let (dx, dy) = (x as f32 - mx, y as f32 - my);
                sxx += dx * dx;
                syy += dy * dy;
                sxy += dx * dy;
            }
            // Angle de l'axe principal (y vers le bas : positif = vers le bas à droite).
            let angle = 0.5 * (2.0 * sxy).atan2(sxx - syy).to_degrees();
            assert!((15.0..=75.0).contains(&angle), "{nom} : axe à {angle:.1}°");

            let yeux: Vec<_> = px.iter().filter(|p| matches!(p.2, 'A' | '8')).collect();
            assert_eq!(yeux.len(), 1, "{nom} : yeux {yeux:?}");
            let &&(ox, oy, oc) = yeux.first().unwrap();
            assert_eq!(oc, '8', "{nom} : l'œil du piqué est rouge");
            assert!(ox >= 9 && oy >= 7, "{nom} : œil en ({ox},{oy})");

            let avant = px.iter().max_by_key(|p| (p.0 + p.1, p.0)).unwrap();
            assert_eq!(avant.2, '5', "{nom} : devant, {avant:?} n'est pas le bec");
            assert!(
                (avant.0 - ox).abs() <= 4 && (avant.1 - oy).abs() <= 4,
                "{nom} : bec {avant:?} loin de l'œil ({ox},{oy})"
            );
            let arriere = px.iter().min_by_key(|p| (p.0 + p.1, p.0)).unwrap();
            assert_eq!(arriere.2, '1', "{nom} : derrière, {arriere:?}");
            assert!(arriere.0 + arriere.1 <= 6, "{nom} : derrière, {arriere:?}");
        }
        // L'élan (ailes ouvertes) est plus étalé que le piqué (ailes en flèche).
        assert_ne!(dessin("pigeon_plongee0"), dessin("pigeon_plongee1"));
    }

    /// Exigence du coordinateur (carte 4.4), reprise carte 6.6 : on distingue d'un
    /// coup d'œil le pigeon de l'oiseau de décor (inoffensif) et du corbeau.
    /// - Couleur principale : le pigeon perché est gris-bleu MOYEN (`4`), entre le
    ///   corbeau bleu nuit (`1`) et l'oiseau blanc (`7`), avec un écart de clarté
    ///   d'au moins 2,5:1 de chaque côté ;
    /// - taille : plus gros que l'oiseau de décor (18 x 14 contre 12 x 8, au moins
    ///   2 fois plus de pixels pleins) ;
    /// - rien du blanc ni du nuage de l'oiseau (`7`, `6`) dans le pigeon ; le cou
    ///   irisé (`D`, `E`) et les pattes corail (`9`) n'existent que chez lui ;
    /// - l'œil : orange (`A`) quand il est perché, rouge corail (`8`) sous un sourcil
    ///   sombre pendant l'annonce (il va piquer).
    #[test]
    fn le_pigeon_ne_ressemble_ni_a_l_oiseau_de_decor_ni_au_corbeau() {
        let oiseau = dessin("oiseau0");
        for nom in PIGEON_TOUT {
            let d = dessin(nom);
            assert!(
                d.iter().all(|l| !l.contains(['6', '7'])),
                "{nom} : couleur claire de l'oiseau de décor"
            );
        }
        for nom in ennemis::CORBEAU.iter().chain(&objets_decor::OISEAU) {
            let d = dessin(nom);
            assert!(
                d.iter().all(|l| !l.contains(['D', 'E', '9'])),
                "{nom} : couleur du pigeon"
            );
        }
        assert_eq!(taille("oiseau0"), (12, 8));
        let (p, o) = (opaques(dessin("pigeon0")), opaques(oiseau));
        assert!(p >= 2 * o, "pigeon {p} px, oiseau {o} px");
        for nom in ennemis::PIGEON {
            assert_eq!(couleur_principale(dessin(nom)), '4', "{nom}");
            assert!(
                dessin(nom).iter().any(|l| l.contains('9')),
                "{nom} : pattes"
            );
        }
        assert_eq!(couleur_principale(oiseau), '7');
        assert_eq!(couleur_principale(dessin("corbeau0")), '1');
        assert!(contraste(couleur_de('4'), couleur_de('1')) >= 2.5);
        assert!(contraste(couleur_de('4'), couleur_de('7')) >= 2.5);
        for (nom, couleur) in [("pigeon0", 'A'), ("pigeon1", 'A'), ("pigeon_annonce", '8')] {
            let yeux: Vec<_> = pixels_colores(dessin(nom))
                .into_iter()
                .filter(|p| matches!(p.2, 'A' | '8'))
                .collect();
            assert_eq!(yeux.len(), 1, "{nom} : yeux {yeux:?}");
            let (x, y, c) = yeux[0];
            assert_eq!(c, couleur, "{nom} : couleur de l'œil");
            if nom == "pigeon_annonce" {
                assert!(
                    matches!(car(dessin(nom), x, y - 1), '0' | '1'),
                    "{nom} : pas de sourcil"
                );
            }
        }
    }

    /// Le pigeon se détache-t-il de ce ciel ? (Carte 6.6.) Deux conditions :
    /// - son contour sombre (`0`) y contraste d'au moins 3:1 : c'est lui qui dessine
    ///   la silhouette sur les ciels clairs ;
    /// - son plumage (`4`, gris-bleu : plus de bleu que de rouge) tranche sur le ciel
    ///   par la TEINTE : le ciel est chaud (plus de rouge que de bleu) et son rouge
    ///   dépasse celui du plumage d'au moins 100.
    ///
    /// L'ancien pigeon (ardoise `2`, carte 4.4) contrastait par la clarté (3,7:1 sur
    /// le corail), mais n'était qu'une masse sombre. Le nouveau, gris-bleu moyen, a
    /// la même clarté que le corail (1,0:1) : c'est la teinte, froide sur un ciel
    /// chaud, qui l'en sépare, et le contour qui le cerne.
    fn pigeon_se_detache(ciel: Couleur) -> Result<(), String> {
        let k = contraste(couleur_de('0'), ciel);
        if k < 3.0 {
            return Err(format!("contour à {k:.2}:1"));
        }
        let plumage = couleur_de('4');
        let chaud = ciel.r > ciel.b && i32::from(ciel.r) - i32::from(plumage.r) >= 100;
        if plumage.b <= plumage.r || !chaud {
            return Err(format!("plumage {plumage:?} de même teinte que le ciel"));
        }
        Ok(())
    }

    /// Le pigeon se détache du ciel derrière lui (`pigeon_se_detache`), dans les vrais
    /// niveaux 4 et 5 (aujourd'hui les vents d'altitude et Au-dessus des nuages) et, depuis
    /// la carte 6.4b, dans la mer de nuages (son ciel corail, qui se dore vers le bas) :
    /// pour chaque pigeon posé, toutes les bandes de ciel qu'il traverse
    /// (perché, puis jusqu'au fond de sa plongée). Le haut du ciel du niveau 4 est
    /// violet (contour et violet : 2,4:1, et le violet est froid) : ce test interdit
    /// d'y percher un pigeon.
    #[test]
    fn le_pigeon_se_detache_du_ciel_des_niveaux_4_et_5() {
        use crate::decor::{ambiance, ciel_a};
        use pepin_core::entities::ennemi::PROFONDEUR_PLONGEE;
        use pepin_core::entities::{Entite, TypeEnnemi};
        use pepin_core::palette::{
            ARDOISE_NUIT, CORAIL, CORAIL_CLAIR, OR, ORANGE_SOLEIL, OR_PALE, VIOLET_ORAGE,
        };
        use pepin_core::HAUTEUR_LOGIQUE;
        let mut vus = 0;
        // Le critère lui-même : les ciels chauds passent, le violet et la nuit non.
        for ciel in [CORAIL, CORAIL_CLAIR, ORANGE_SOLEIL, OR, OR_PALE] {
            assert_eq!(pigeon_se_detache(ciel), Ok(()), "{ciel:?}");
        }
        for ciel in [VIOLET_ORAGE, ARDOISE_NUIT] {
            assert!(pigeon_se_detache(ciel).is_err(), "{ciel:?}");
        }
        for infos in [
            *niveaux::par_cle("vents"),
            *niveaux::par_cle("mer"),
            *niveaux::par_cle("nuages"),
        ] {
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
                // Sprite 18x14 : 4 px dépassent au-dessus de la boîte (voir `scene.rs`).
                let haut = (p.boite.y - 4.0 - cam_y) as i32;
                let bas = (p.boite.bas() + PROFONDEUR_PLONGEE - cam_y) as i32;
                // Les 6 dernières lignes d'une bande sont tramées avec la suivante.
                for y in haut..bas.min(HAUTEUR_LOGIQUE as i32) {
                    for ciel in [ciel_a(&a, y), ciel_a(&a, y + 6)] {
                        if let Err(e) = pigeon_se_detache(ciel) {
                            panic!(
                                "niveau {} : pigeon en {:?}, sur {ciel:?} à y = {y} : {e}",
                                infos.numero,
                                p.perchoir()
                            );
                        }
                    }
                }
            }
        }
        assert!(
            vus >= 3,
            "{vus} pigeon(s) dans les vents, la mer et Au-dessus des nuages"
        );
    }

    /// Même contrôle dans la descente (carte 5.17), dont le ciel défile avec la
    /// caméra (`decor::VOYAGE`) : pour chaque pigeon, chaque ligne qu'il occupe
    /// (perché, puis jusqu'au fond de sa plongée) et CHAQUE position de la caméra où
    /// cette ligne est à l'écran, il se détache de la bande de ciel derrière lui (et
    /// de celle de sa trame) : voir `pigeon_se_detache`. Les pigeons sont
    /// perchés assez haut pour n'avoir derrière eux que l'or, l'orange et le corail,
    /// jamais le violet ni la nuit.
    #[test]
    fn le_pigeon_se_detache_du_ciel_de_la_descente() {
        use crate::decor::{bande_voyage, camera_fond, remontee_voyage, VOYAGE};
        use pepin_core::entities::ennemi::PROFONDEUR_PLONGEE;
        use pepin_core::entities::{Entite, TypeEnnemi};
        use pepin_core::HAUTEUR_LOGIQUE;
        let niveau = niveaux::par_cle("descente").charger().unwrap();
        let (h_ecran, h_niveau) = (HAUTEUR_LOGIQUE as i32, niveau.hauteur_px() as i32);
        let mut vus = 0;
        for e in &niveau.entites {
            let Entite::Ennemi(p) = e else { continue };
            if p.type_ennemi != TypeEnnemi::Pigeon {
                continue;
            }
            vus += 1;
            // Sprite 18x14 : 4 px dépassent au-dessus de la boîte (voir `scene.rs`).
            let haut = (p.boite.y - 4.0) as i32;
            let bas = (p.boite.bas() + PROFONDEUR_PLONGEE) as i32;
            for y_niveau in haut..bas {
                let cameras = (y_niveau - h_ecran + 1).max(0)..=y_niveau.min(h_niveau - h_ecran);
                for cam_y in cameras {
                    let r = remontee_voyage(camera_fond(cam_y, h_niveau));
                    let y = y_niveau - cam_y;
                    for ciel in [VOYAGE[bande_voyage(y, r)], VOYAGE[bande_voyage(y + 6, r)]] {
                        if let Err(e) = pigeon_se_detache(ciel) {
                            panic!(
                                "pigeon en {:?}, sur {ciel:?} (caméra {cam_y}) : {e}",
                                p.perchoir()
                            );
                        }
                    }
                }
            }
        }
        assert_eq!(vus, 2);
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

    // --- Aiguilles d'horloge (carte 5.10) ---

    /// La plaque de l'aiguille couvre EXACTEMENT sa boîte de collision
    /// (`level::TAILLE_AIGUILLE`), cernée de sombre ; son dessus (rangée 1) est
    /// clair : on voit où poser les pieds.
    #[test]
    fn l_aiguille_couvre_toute_sa_collision() {
        let a = dessin("aiguille");
        let (w, h) = pepin_core::level::TAILLE_AIGUILLE;
        assert_eq!((largeur(a), hauteur(a)), (w as usize, h as usize));
        for (y, ligne) in a.iter().enumerate() {
            assert!(!ligne.contains('.'), "rangée {y} : du vide dans la plaque");
            assert!(ligne.starts_with('0') && ligne.ends_with('0'), "rangée {y}");
        }
        assert!(a[0].chars().all(|c| c == '0') && a[7].chars().all(|c| c == '0'));
        assert!(a[1][1..31].chars().all(|c| c == 'C'), "dessus or pâle");
    }

    /// Le cadran (32 x 32) est rond et symétrique : chaque rangée a la même
    /// silhouette à gauche et à droite, et en haut et en bas ; son premier et son
    /// dernier pixel sont le contour sombre. Le moyeu (le pivot) est au centre.
    #[test]
    fn le_cadran_est_rond_et_cerne_de_sombre() {
        let c = dessin("cadran");
        assert_eq!((largeur(c), hauteur(c)), (32, 32));
        let plein = |x: usize, y: usize| c[y].as_bytes()[x] != b'.';
        for (y, ligne) in c.iter().enumerate() {
            for x in 0..32 {
                assert_eq!(plein(x, y), plein(31 - x, y), "({x}, {y})");
                assert_eq!(plein(x, y), plein(x, 31 - y), "({x}, {y})");
                assert_eq!(plein(x, y), plein(y, x), "({x}, {y})");
            }
            let l = ligne.trim_matches('.');
            assert!(l.starts_with('0') && l.ends_with('0'), "rangée {y}");
        }
        assert_eq!(&c[15][14..18], "0BB0", "moyeu");
        assert_eq!(dessin("maillon").len(), 4);
    }

    /// Le cadran, la tige et l'aiguille se lisent sur TOUS les ciels : sur chaque
    /// bande de ciel de chaque thème, l'une de leurs couleurs (contour sombre, or,
    /// blanc cassé du cadran) contraste d'au moins 4,5:1.
    #[test]
    fn l_horloge_se_detache_de_tous_les_ciels() {
        use crate::decor::ambiance;
        for theme in Theme::TOUS {
            for ciel in ambiance(theme).ciel {
                let k = contraste(couleur_de('0'), ciel)
                    .max(contraste(couleur_de('B'), ciel))
                    .max(contraste(couleur_de('7'), ciel));
                assert!(k >= 4.5, "{theme:?} : horloge sur {ciel:?} : {k:.2}");
            }
        }
    }

    // --- Grand Nuage (carte 5.11) ---

    /// Les cinq images du Grand Nuage : colère (2), essoufflé (2), touché.
    const NUAGE_TOUT: [&str; 5] = [
        "nuage_colere0",
        "nuage_colere1",
        "nuage_essouffle0",
        "nuage_essouffle1",
        "nuage_touche",
    ];

    /// 48 x 32 (sa boîte), contour sombre de 1 px ; la même silhouette dans les cinq
    /// images (seul le visage change). Il remplit sa boîte : la tête, sur laquelle on
    /// saute, touche le haut du dessin (rangée 0), ses flancs les bords gauche et
    /// droit, ses festons descendent jusqu'à l'avant-dernière rangée (ses éclairs
    /// partent du bas de la boîte).
    #[test]
    fn le_grand_nuage_a_cinq_images_de_sa_taille_et_un_contour_sombre() {
        let reference = dessin("nuage_colere0");
        for nom in NUAGE_TOUT {
            assert_eq!(taille(nom), (48, 32), "{nom}");
            verifier_contour(nom);
            let d = dessin(nom);
            for y in 0..32 {
                for x in 0..48 {
                    assert_eq!(
                        car(d, x, y) == '.',
                        car(reference, x, y) == '.',
                        "{nom} : silhouette différente en ({x},{y})"
                    );
                }
            }
        }
        let d = reference;
        assert!(d[0].contains('0') && d[30].contains('0'));
        assert!((0..32).any(|y| car(d, 0, y) != '.') && (0..32).any(|y| car(d, 47, y) != '.'));
        // Une ombre (ventre violet) et un reflet (nuage clair) seulement, en plus du corps.
        let couleurs: std::collections::BTreeSet<char> = d
            .iter()
            .flat_map(|l| l.chars())
            .filter(|&c| c != '.')
            .collect();
        assert_eq!(
            couleurs.into_iter().collect::<String>(),
            "0467BF",
            "contour, corps, reflet, dents, ventre, yeux d'or"
        );
    }

    /// L'état se lit au visage : en colère, des yeux d'or (aucun ailleurs) ; essoufflé,
    /// une langue corail et une goutte de sueur cyan ; touché, ni or, ni blanc, ni
    /// corail (les yeux plissés et la bouche sont des traits sombres).
    #[test]
    fn le_visage_du_grand_nuage_dit_son_etat() {
        let compte = |nom: &str, c: char| {
            dessin(nom)
                .iter()
                .flat_map(|l| l.chars())
                .filter(|&x| x == c)
                .count()
        };
        for nom in ["nuage_colere0", "nuage_colere1"] {
            assert!(compte(nom, 'B') >= 8, "{nom} : yeux d'or");
            assert_eq!(compte(nom, '8') + compte(nom, 'D'), 0, "{nom}");
        }
        for nom in ["nuage_essouffle0", "nuage_essouffle1"] {
            assert_eq!(compte(nom, 'B'), 0, "{nom} : plus de colère");
            assert!(compte(nom, '8') >= 4 && compte(nom, 'D') >= 2, "{nom}");
        }
        for c in ['B', '7', '8', 'D'] {
            assert_eq!(compte("nuage_touche", c), 0, "touché : « {c} »");
        }
        // Les deux images d'une animation diffèrent.
        assert_ne!(dessin("nuage_colere0"), dessin("nuage_colere1"));
        assert_ne!(dessin("nuage_essouffle0"), dessin("nuage_essouffle1"));
    }

    /// Le Grand Nuage se lit sur TOUS les ciels : son contour sombre ou son corps
    /// clair contraste d'au moins 4,5:1 avec chaque bande de ciel de chaque thème.
    /// Ses points de vie du HUD aussi (même contour, même corps).
    #[test]
    fn le_grand_nuage_et_ses_points_de_vie_se_detachent_de_tous_les_ciels() {
        use crate::decor::ambiance;
        for theme in Theme::TOUS {
            for ciel in ambiance(theme).ciel {
                let k = contraste(couleur_de('0'), ciel).max(contraste(couleur_de('6'), ciel));
                assert!(k >= 4.5, "{theme:?} : nuage sur {ciel:?} : {k:.2}");
            }
        }
        for nom in ["pv_nuage", "pv_nuage_perdu"] {
            assert_eq!(taille(nom), (10, 6), "{nom}");
            verifier_contour(nom);
        }
        // Plein : le corps clair du nuage ; perdu : éteint (aucune couleur claire).
        let clairs = |nom: &str| {
            dessin(nom)
                .iter()
                .flat_map(|l| l.chars())
                .filter(|c| "4567".contains(*c))
                .count()
        };
        assert!(clairs("pv_nuage") >= 12);
        assert_eq!(clairs("pv_nuage_perdu"), 0);
    }

    /// L'icône des vies (carte 6.2) : 10 x 10, contour sombre complet, deux yeux, deux
    /// joues, et lisible sur TOUS les ciels : sur chaque bande de chaque thème, son
    /// contour ou son corps blanc contraste d'au moins 4,5:1.
    #[test]
    fn l_icone_des_vies_est_pepin_et_se_lit_sur_tous_les_ciels() {
        use crate::decor::ambiance;
        assert_eq!(taille("vie"), (10, 10));
        verifier_contour("vie");
        let d = dessin("vie");
        let compter = |c: char| d.iter().flat_map(|l| l.chars()).filter(|&x| x == c).count();
        // Corps blanc (la couleur la plus fréquente), deux joues corail clair.
        assert!(compter('7') >= 25, "corps : {}", compter('7'));
        assert_eq!(compter('9'), 2, "deux joues");
        // Deux yeux : les seuls pixels sombres À L'INTÉRIEUR du contour (rangées 3, 4).
        let yeux: Vec<(usize, usize)> = (1..9)
            .flat_map(|y| (1..9).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                let ligne: Vec<char> = d[y].chars().collect();
                let premier = ligne
                    .iter()
                    .position(|&c| c != '.' && c != '0')
                    .unwrap_or(0);
                let dernier = ligne
                    .iter()
                    .rposition(|&c| c != '.' && c != '0')
                    .unwrap_or(0);
                ligne[x] == '0' && x > premier && x < dernier
            })
            .collect();
        assert_eq!(yeux, [(4, 3), (7, 3), (4, 4), (7, 4)], "deux yeux de 2 px");
        for theme in Theme::TOUS {
            for ciel in ambiance(theme).ciel {
                let k = contraste(couleur_de('0'), ciel).max(contraste(couleur_de('7'), ciel));
                assert!(k >= 4.5, "{theme:?} : icône des vies sur {ciel:?} : {k:.2}");
            }
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
    /// (La descente de gouttière de la rue, plus grande, n'est plus un sprite depuis
    /// la carte 9.20h : `decor_rue` la dessine avec sa maison.)
    #[test]
    fn les_objets_de_decor_tiennent_dans_leur_case_et_celle_du_dessus() {
        for &(nom, d) in objets_decor::sprites() {
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
        for &(nom, d) in objets_decor::sprites() {
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
        for &(nom, d) in objets_decor::sprites() {
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

    /// Le parapluie fermé se lit, tenu par la poignée (carte 7.1b) : embout de métal en
    /// haut, toile corail repliée (plis en corail clair, sangle blanche) plus large en bas
    /// qu'en haut, crosse orange en bas, à hauteur du flanc (crochet ouvert vers Pépin).
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
        // Crosse : l'orange n'est que dans les rangées 8 à 12 (le flanc), pas en l'air.
        assert!(compter('A', 8..13) >= 6, "crosse trop petite");
        assert_eq!(compter('A', 0..8), 0, "de l'orange au-dessus de la crosse");
        assert_eq!(compter('A', 13..16), 0, "de l'orange sous la crosse");
        // C'est un crochet : une rangée a deux bouts d'orange séparés (le bec et la tige).
        let crochet = (8..13).any(|y| {
            let oranges: Vec<i32> = (11..16).filter(|&x| car(d, x, y) == 'A').collect();
            oranges.windows(2).any(|p| p[1] - p[0] > 1)
        });
        assert!(crochet, "la crosse n'est pas recourbée");
        // Toile repliée : corail, plis en corail clair, sangle blanche.
        assert!(compter('8', 2..8) >= 8, "toile trop petite");
        assert!(compter('9', 2..8) >= 3, "pas de plis");
        let sangle = (2..8).any(|y| (12..15).all(|x| car(d, x, y) == '7'));
        assert!(sangle, "pas de sangle");
        // Plus large en bas (rangée 7, vers la poignée) qu'en haut (rangée 2, sous l'embout).
        let pleins = |y: i32| (12..16).filter(|&x| !".0".contains(car(d, x, y))).count();
        assert!(pleins(7) > pleins(2), "la toile ne s'affine pas");
        // Embout de métal (gris `5`) au-dessus de la toile.
        assert_eq!(compter('5', 0..2), 1, "embout");
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
