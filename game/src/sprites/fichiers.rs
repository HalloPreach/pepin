//! Les sprites lus dans les fichiers PNG de `assets/sprites` (carte 10.4a).
//!
//! Chaque sprite dessiné à la main est un fichier `assets/sprites/<famille>/<clé>.png`,
//! SEULE source du dessin. `build.rs` les embarque tous dans le jeu à la compilation
//! (`include_bytes!`) ; au premier besoin, `table` les décode UNE fois avec le décodeur
//! PNG que macroquad embarque déjà (crate `image`) et range chaque dessin sous sa forme
//! de grille de texte (`.` transparent, `0`-`F` couleur de la palette) : tout le reste
//! du jeu (textures, éclaircissement des tuiles, motifs, tests) continue de lire des
//! grilles, exactement comme quand elles étaient écrites dans le code.
//!
//! Ce qui est CALCULÉ reste du code : les ALIAS ci-dessous (un même dessin sous deux
//! noms) et, ailleurs, les variantes (tuiles éclaircies par thème, miroir, motifs).
//!
//! Notions Rust : `include!` d'un fichier écrit par `build.rs`, `OnceLock` (une valeur
//! calculée une seule fois, au premier accès), `Box::leak` (donner une durée de vie
//! `'static` à une valeur construite à l'exécution : la table vit jusqu'à la fin du jeu).

use std::sync::OnceLock;

use macroquad::prelude::{Image as ImageMq, ImageFormat};
use pepin_core::palette::TOUTES;

use super::Image;

// `FICHIERS` : (famille, clé, octets du PNG) de chaque fichier, triés par chemin.
include!(concat!(env!("OUT_DIR"), "/sprites_embarques.rs"));

/// Les familles, dans l'ordre du jeu (celui de la planche contact) : une famille = un
/// dossier de `assets/sprites`. Un dossier absent de cette liste serait un oubli (un
/// test le refuse).
pub const FAMILLES: [&str; 16] = [
    "pepin",
    "pepinette",
    "ennemis",
    "ennemis_eau",
    "pieuvre",
    "objets",
    "objets_decor",
    "objets_eau",
    "tuiles",
    "jetee",
    "tuiles_monde2/caniveau",
    "tuiles_monde2/egouts",
    "tuiles_monde2/cascade",
    "tuiles_monde2/grotte",
    "tuiles_monde2/port",
    "tuiles_monde2/riviere",
];

/// Les dessins RÉUTILISÉS sous un autre nom : (famille, nom, nom du dessin d'origine).
/// Aucun fichier pour eux : c'est le même dessin.
const ALIAS: [(&str, &str, &str); 7] = [
    // La réapparition finit par les mêmes étincelles pour les deux personnages.
    ("pepinette", "pepinette_mort2", "pepin_mort2"),
    // L'eau des courants (carte 9.3) se dessine comme l'eau calme : seul le
    // mouvement (les particules) change.
    ("tuiles", "eau_courant_g_haut", "eau_calme_haut"),
    ("tuiles", "eau_courant_g_vague", "eau_calme_vague"),
    ("tuiles", "eau_courant_g_plein", "eau_calme_plein"),
    ("tuiles", "eau_courant_d_haut", "eau_calme_haut"),
    ("tuiles", "eau_courant_d_vague", "eau_calme_vague"),
    ("tuiles", "eau_courant_d_plein", "eau_calme_plein"),
];

/// Les caractères d'une grille, dans l'ordre de la palette.
const CHIFFRES: &[u8; 16] = b"0123456789ABCDEF";

/// Décode un PNG en grille de texte. Erreur si un pixel n'est ni transparent ni
/// d'une couleur de la palette.
pub fn decoder(octets: &[u8]) -> Result<Vec<String>, String> {
    let image = ImageMq::from_file_with_format(octets, Some(ImageFormat::Png))
        .map_err(|e| e.to_string())?;
    let largeur = image.width as usize;
    let mut grille = Vec::with_capacity(image.height as usize);
    for (y, rangee) in image.bytes.chunks(largeur * 4).enumerate() {
        let mut ligne = String::with_capacity(largeur);
        for (x, p) in rangee.chunks(4).enumerate() {
            if p[3] == 0 {
                ligne.push('.');
                continue;
            }
            let i = TOUTES
                .iter()
                .position(|c| p[3] == 255 && (c.r, c.g, c.b) == (p[0], p[1], p[2]))
                .ok_or_else(|| format!("pixel ({x}, {y}) hors palette : {p:?}"))?;
            ligne.push(CHIFFRES[i] as char);
        }
        grille.push(ligne);
    }
    Ok(grille)
}

/// Une famille décodée : ses sprites (nom, dessin), fichiers puis alias.
type Famille = Vec<(&'static str, Image)>;

/// Toutes les familles décodées, dans l'ordre de `FAMILLES` (décodées au premier
/// appel, une seule fois).
fn table() -> &'static [Famille; 16] {
    static TABLE: OnceLock<[Famille; 16]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut familles: [Famille; 16] = Default::default();
        for &(famille, cle, octets) in FICHIERS {
            let grille = decoder(octets)
                .unwrap_or_else(|e| panic!("assets/sprites/{famille}/{cle}.png : {e}"));
            let lignes: Vec<&'static str> = grille
                .into_iter()
                .map(|l| &*Box::leak(l.into_boxed_str()))
                .collect();
            let dessin: Image = Box::leak(lignes.into_boxed_slice());
            // Un dossier inconnu est rangé nulle part (le test `aucun_fichier_orphelin`
            // le signale).
            if let Some(i) = FAMILLES.iter().position(|&f| f == famille) {
                familles[i].push((cle, dessin));
            }
        }
        for (famille, nom, origine) in ALIAS {
            let dessin = familles
                .iter()
                .flatten()
                .find(|(n, _)| *n == origine)
                .map(|&(_, d)| d)
                .unwrap_or_else(|| panic!("alias {nom} : dessin {origine} absent"));
            let i = FAMILLES.iter().position(|&f| f == famille).unwrap();
            familles[i].push((nom, dessin));
        }
        familles
    })
}

/// Les sprites d'une famille (nom, dessin). Panique si la famille n'existe pas
/// (erreur de programmation, attrapée par les tests).
pub fn famille(nom: &str) -> &'static [(&'static str, Image)] {
    let i = FAMILLES
        .iter()
        .position(|&f| f == nom)
        .unwrap_or_else(|| panic!("famille de sprites inconnue : {nom}"));
    &table()[i]
}

/// Le dessin d'un sprite, d'après son nom (les noms sont uniques, toutes familles
/// confondues). Panique s'il manque : un test vérifie que chaque nom demandé existe.
pub fn dessin(nom: &str) -> Image {
    table()
        .iter()
        .flatten()
        .find(|(n, _)| *n == nom)
        .map(|&(_, d)| d)
        .unwrap_or_else(|| panic!("sprite introuvable : {nom}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les blocs d'un PNG, dans l'ordre (le fichier doit commencer par la signature).
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
    fn chaque_png_se_decode_sans_pixel_hors_palette() {
        assert!(!FICHIERS.is_empty());
        for &(famille, cle, octets) in FICHIERS {
            let grille =
                decoder(octets).unwrap_or_else(|e| panic!("{famille}/{cle}.png : {e}"));
            assert!(!grille.is_empty() && !grille[0].is_empty(), "{famille}/{cle} vide");
        }
    }

    #[test]
    fn chaque_png_n_a_que_les_blocs_permis() {
        // Règle « aucun nom » : aucun bloc de texte, de date ni de métadonnées.
        for &(famille, cle, octets) in FICHIERS {
            assert_eq!(
                blocs(octets),
                ["IHDR", "PLTE", "tRNS", "IDAT", "IEND"],
                "{famille}/{cle}.png"
            );
        }
    }

    #[test]
    fn aucun_fichier_orphelin() {
        // Chaque fichier est dans une famille connue...
        for &(famille, cle, _) in FICHIERS {
            assert!(
                FAMILLES.contains(&famille),
                "assets/sprites/{famille}/{cle}.png : dossier inconnu (voir FAMILLES)"
            );
        }
        // ... chaque famille a des fichiers, et aucun alias ne cache un fichier.
        for f in FAMILLES {
            assert!(FICHIERS.iter().any(|&(g, _, _)| g == f), "famille {f} vide");
        }
        for (_, nom, _) in ALIAS {
            assert!(!FICHIERS.iter().any(|&(_, c, _)| c == nom), "{nom} : fichier ET alias");
        }
    }
}
