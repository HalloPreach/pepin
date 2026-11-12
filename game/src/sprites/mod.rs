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
// ligne et les rendrait illisibles, donc on l'exclut de ces quatre fichiers.
#[rustfmt::skip]
pub mod ennemis;
#[rustfmt::skip]
pub mod objets;
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
        tuiles::SPRITES,
    ]
    .concat()
}

/// Table de substitution de couleurs d'un thème : les tuiles s'éclaircissent
/// d'un niveau à l'autre (l'ardoise gagne `decalage` crans, jusqu'au blanc cassé).
fn remplacer(c: char, theme: Theme) -> char {
    let decalage = match theme {
        Theme::Toits | Theme::Orage => 0,
        Theme::Gouttieres => 1,
        Theme::Altitude => 2,
        Theme::Soleil => 3,
    };
    match c {
        // Rampe d'ardoise 1..=4 : on avance sur la rampe sans dépasser 7.
        '1'..='4' => {
            char::from_digit((c.to_digit(10).unwrap_or(1) + decalage).min(7), 10).unwrap_or(c)
        }
        // Les briques violettes virent à l'orange sous le soleil.
        'F' if theme == Theme::Soleil => 'A',
        _ => c,
    }
}

/// Indice d'un thème (0 à 4), pour ranger ses tuiles dans un tableau.
pub fn indice_theme(theme: Theme) -> usize {
    match theme {
        Theme::Toits => 0,
        Theme::Gouttieres => 1,
        Theme::Orage => 2,
        Theme::Altitude => 3,
        Theme::Soleil => 4,
    }
}

const THEMES: [Theme; 5] = [
    Theme::Toits,
    Theme::Gouttieres,
    Theme::Orage,
    Theme::Altitude,
    Theme::Soleil,
];

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
    /// Sprites (Pépin, ennemis, objets) : les mêmes pour tous les niveaux.
    sprites: HashMap<&'static str, Texture2D>,
    /// Tuiles : un jeu complet par thème (couleurs éclaircies progressivement).
    tuiles: Vec<HashMap<&'static str, Texture2D>>,
}

impl Banque {
    /// Génère toutes les textures. À appeler une seule fois, après l'ouverture de la fenêtre.
    pub fn charger() -> Self {
        let sprites = [pepin::SPRITES, ennemis::SPRITES, objets::SPRITES]
            .concat()
            .into_iter()
            .map(|(nom, dessin)| (nom, texture(dessin, None)))
            .collect();
        let tuiles = THEMES
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
        if let Some(t) = self.tuiles[indice_theme(theme)].get(nom) {
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
        assert_eq!(taille("goutte0"), (8, 8));
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
            &pepin::PLANER,
            &pepin::REAPPARITION,
            &ennemis::FLAQUE,
            &ennemis::CORBEAU,
            &objets::GOUTTE,
            &objets::LANTERNE,
            &objets::CHAMPIGNON,
            &objets::FRIABLE,
            &objets::DRAPEAU,
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
        for theme in THEMES {
            for c in "0123456789ABCDEF.".chars() {
                assert!(pixel(remplacer(c, theme)).is_ok());
            }
        }
        // Le thème du soleil éclaircit l'ardoise : '2' devient '5'.
        assert_eq!(remplacer('2', Theme::Soleil), '5');
        assert_eq!(remplacer('2', Theme::Toits), '2');
    }
}
