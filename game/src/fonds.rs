//! Les couches FIXES des fonds, lues dans les fichiers PNG de `assets/fonds`
//! (carte 10.4c).
//!
//! Chaque couche dessinée à la main (un ciel en bandes, le soleil, l'arc-en-ciel...)
//! est un fichier `assets/fonds/<thème>/<nom>.png`, SEULE source du dessin. Comme les
//! sprites (carte 10.4a) : `build.rs` les embarque tous (`include_bytes!`, aucune
//! liste à la main), le décodeur PNG déjà embarqué par macroquad les lit une fois
//! (`sprites::fichiers::decoder`), et chaque couche devient une texture « au plus
//! proche » posée 1 pixel = 1 pixel.
//!
//! Ce qui est CALCULÉ reste du code, dans `decor` : OÙ poser la couche (parallaxe,
//! remontée du voyage), la répétition horizontale d'un ciel (le fichier n'a qu'une
//! colonne, sa période), et toutes les couches animées ou semées au hasard (pluie,
//! étoiles, silhouettes, îlots, plafond de nuages). L'ordre de la pile de dessin est
//! écrit dans `assets/fonds/LISEZMOI.md` et vérifié par `decor::tests`.
//!
//! Notions Rust : `thread_local!` (une valeur propre au fil d'exécution : les
//! textures, qui ne quittent jamais le fil du rendu), `RefCell` (emprunt mutable
//! vérifié à l'exécution), `OnceLock` (calcul unique, au premier accès).

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::OnceLock;

use macroquad::prelude::{
    draw_texture_ex, vec2, DrawTextureParams, FilterMode, Image as ImageMq, Texture2D, BLANK, WHITE,
};
use pepin_core::palette::Couleur;

use crate::rendu::couleur_mq;
use crate::sprites::{fichiers::decoder, pixel};

// `FICHIERS` : (thème, nom, octets du PNG) de chaque fichier, triés par chemin.
include!(concat!(env!("OUT_DIR"), "/fonds_embarques.rs"));

/// Une couche décodée : ses lignes de pixels (`.` transparent, `0`-`F` palette).
type Grille = Vec<String>;

/// Toutes les couches décodées (au premier appel, une seule fois), dans l'ordre
/// de `FICHIERS`.
fn table() -> &'static [(&'static str, &'static str, Grille)] {
    static TABLE: OnceLock<Vec<(&'static str, &'static str, Grille)>> = OnceLock::new();
    TABLE.get_or_init(|| {
        FICHIERS
            .iter()
            .map(|&(theme, nom, octets)| {
                let grille = decoder(octets)
                    .unwrap_or_else(|e| panic!("assets/fonds/{theme}/{nom}.png : {e}"));
                (theme, nom, grille)
            })
            .collect()
    })
}

/// La couche `assets/fonds/<theme>/<nom>.png`, décodée. Panique si le fichier
/// manque (le test `chaque_couche_demandee_existe` l'attrape avant).
pub fn grille(theme: &str, nom: &str) -> &'static [String] {
    table()
        .iter()
        .find(|(t, n, _)| *t == theme && *n == nom)
        .map(|(_, _, g)| g.as_slice())
        .unwrap_or_else(|| panic!("assets/fonds/{theme}/{nom}.png absent"))
}

/// Couleur du pixel `(x, y)` d'une couche (`None` : transparent). Pure.
pub fn couleur(theme: &str, nom: &str, x: usize, y: usize) -> Option<Couleur> {
    let c = grille(theme, nom)[y].as_bytes()[x] as char;
    pixel(c).expect("caractère vérifié au décodage")
}

/// Largeur et hauteur d'une couche, en pixels.
pub fn taille(theme: &str, nom: &str) -> (i32, i32) {
    let g = grille(theme, nom);
    (g[0].len() as i32, g.len() as i32)
}

thread_local! {
    /// Les textures déjà créées, par (thème, nom) : une seule fois chacune, au
    /// premier dessin (il faut le contexte graphique, prêt quand on dessine).
    static TEXTURES: RefCell<HashMap<(&'static str, &'static str), Texture2D>> =
        RefCell::new(HashMap::new());
}

/// La texture d'une couche (créée au premier appel).
fn texture(theme: &'static str, nom: &'static str) -> Texture2D {
    TEXTURES.with(|t| {
        t.borrow_mut()
            .entry((theme, nom))
            .or_insert_with(|| {
                let g = grille(theme, nom);
                let (l, h) = taille(theme, nom);
                let mut img = ImageMq::gen_image_color(l as u16, h as u16, BLANK);
                for (y, ligne) in g.iter().enumerate() {
                    for (x, c) in ligne.chars().enumerate() {
                        if let Ok(Some(couleur)) = pixel(c) {
                            img.set_pixel(x as u32, y as u32, couleur_mq(couleur));
                        }
                    }
                }
                let t = Texture2D::from_image(&img);
                t.set_filter(FilterMode::Nearest);
                t
            })
            .clone()
    })
}

/// Pose la couche, 1 pixel = 1 pixel, son coin haut gauche en `(x, y)` (pixels
/// logiques de l'écran).
pub fn dessiner(theme: &'static str, nom: &'static str, x: i32, y: i32) {
    draw_texture_ex(
        &texture(theme, nom),
        x as f32,
        y as f32,
        WHITE,
        DrawTextureParams::default(),
    );
}

/// Pose une couche d'UNE colonne (un ciel en bandes : sa période horizontale est
/// d'un pixel) répétée sur `largeur` pixels à partir de `(x, y)`. La répétition est
/// un étirement au plus proche : chaque colonne de l'écran reprend la même colonne
/// du fichier, sans aucun mélange de couleurs.
pub fn repeter(theme: &'static str, nom: &'static str, x: i32, y: i32, largeur: i32) {
    let t = texture(theme, nom);
    draw_texture_ex(
        &t,
        x as f32,
        y as f32,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(largeur as f32, t.height())),
            ..Default::default()
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les blocs d'un PNG, dans l'ordre (le même contrôle que pour les sprites).
    fn blocs(octets: &[u8]) -> Vec<String> {
        assert_eq!(&octets[..8], b"\x89PNG\r\n\x1a\n");
        let mut noms = Vec::new();
        let mut i = 8;
        while i + 8 <= octets.len() {
            let taille = u32::from_be_bytes(octets[i..i + 4].try_into().unwrap()) as usize;
            noms.push(String::from_utf8_lossy(&octets[i + 4..i + 8]).into_owned());
            i += 12 + taille;
        }
        assert_eq!(i, octets.len(), "fin de fichier inattendue");
        noms
    }

    #[test]
    fn chaque_couche_se_decode_sans_pixel_hors_palette() {
        assert!(!FICHIERS.is_empty());
        for &(theme, nom, octets) in FICHIERS {
            let g = decoder(octets).unwrap_or_else(|e| panic!("{theme}/{nom}.png : {e}"));
            assert!(!g.is_empty() && !g[0].is_empty(), "{theme}/{nom} vide");
            assert!(g.iter().all(|l| l.len() == g[0].len()), "{theme}/{nom}");
        }
    }

    #[test]
    fn chaque_couche_n_a_que_les_blocs_permis() {
        // Règle « aucun nom » : aucun bloc de texte, de date ni de métadonnées.
        for &(theme, nom, octets) in FICHIERS {
            assert_eq!(
                blocs(octets),
                ["IHDR", "PLTE", "tRNS", "IDAT", "IEND"],
                "{theme}/{nom}.png"
            );
        }
    }

    /// Le plafond de poids de la carte 10.4c : aucune couche de plus de 16 Ko.
    #[test]
    fn aucune_couche_ne_depasse_16_ko() {
        for &(theme, nom, octets) in FICHIERS {
            assert!(
                octets.len() <= 16 * 1024,
                "{theme}/{nom}.png : {} o",
                octets.len()
            );
        }
    }
}
