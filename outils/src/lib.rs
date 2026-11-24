//! Le format des sprites de PÉPIN (carte 10.4a) : un fichier PNG par sprite dans
//! `assets/sprites/<famille>/<nom>.png`, et sa forme TEXTE, la grille que l'on lit et
//! que l'on corrige à la main (voir `assets/sprites/LISEZMOI.md`).
//!
//! La grille : une ligne = une rangée de pixels, un caractère = un pixel. `.` =
//! transparent ; `0`-`9` puis `A`-`F` = indice dans `pepin_core::palette::TOUTES`.
//!
//! Le PNG ÉCRIT par ce crate est CANONIQUE : PNG indexé 8 bits, toujours la même
//! palette de 17 entrées (les 16 couleurs du jeu dans leur ordre, l'indice du PNG est
//! donc le chiffre de la grille, puis le transparent en 16), et seulement les blocs
//! IHDR, PLTE, tRNS, IDAT, IEND (aucun texte, aucune date, aucun nom). Relire puis
//! réécrire un fichier donne donc exactement les mêmes octets.
//!
//! La LECTURE accepte tout PNG dont les couleurs sont celles de la palette (un fichier
//! retouché dans un logiciel de dessin, par exemple) : chaque pixel est ramené à sa
//! couleur, puis à son caractère.
//!
//! Notions Rust : `Result<_, String>` pour des erreurs lisibles, l'opérateur `?`,
//! `map_err` pour traduire l'erreur d'une autre crate en message.

use pepin_core::palette::TOUTES;

/// Les sons en WAV (carte 10.4e) : lecture et contrôle des fichiers.
pub mod son;

/// Indice du transparent dans la palette du PNG (après les 16 couleurs du jeu).
pub const INDICE_TRANSPARENT: u8 = 16;

/// Les caractères d'une grille, dans l'ordre de la palette (`0` à `F`).
const CHIFFRES: &[u8; 16] = b"0123456789ABCDEF";

/// Indice dans la palette du PNG d'un caractère de grille (`.` : le transparent).
pub fn indice(c: char) -> Result<u8, String> {
    if c == '.' {
        return Ok(INDICE_TRANSPARENT);
    }
    CHIFFRES
        .iter()
        .position(|&ch| ch as char == c)
        .map(|i| i as u8)
        .ok_or_else(|| format!("caractère interdit {c:?} (permis : . 0-9 A-F)"))
}

/// Vérifie qu'une grille est un rectangle non vide de caractères permis, et rend ses
/// indices de palette, rangée par rangée.
fn indices(grille: &[String]) -> Result<(u32, u32, Vec<u8>), String> {
    let largeur = grille.first().map_or(0, |l| l.chars().count());
    if largeur == 0 {
        return Err("grille vide".into());
    }
    let mut sortie = Vec::with_capacity(largeur * grille.len());
    for (y, ligne) in grille.iter().enumerate() {
        if ligne.chars().count() != largeur {
            return Err(format!("la ligne {y} n'a pas {largeur} pixels"));
        }
        for c in ligne.chars() {
            sortie.push(indice(c).map_err(|e| format!("ligne {y} : {e}"))?);
        }
    }
    Ok((largeur as u32, grille.len() as u32, sortie))
}

/// Encode une grille en PNG canonique (voir l'en-tête du module).
pub fn ecrire_png(grille: &[String]) -> Result<Vec<u8>, String> {
    let (largeur, hauteur, donnees) = indices(grille)?;
    // PLTE : les 16 couleurs, puis le transparent (noir, mais invisible).
    let mut palette: Vec<u8> = TOUTES.iter().flat_map(|c| [c.r, c.g, c.b]).collect();
    palette.extend_from_slice(&[0, 0, 0]);
    // tRNS : opacité de chaque entrée (255 = opaque, 0 = transparent).
    let mut opacites = vec![255u8; 16];
    opacites.push(0);
    let mut octets = Vec::new();
    {
        let mut codeur = png::Encoder::new(&mut octets, largeur, hauteur);
        codeur.set_color(png::ColorType::Indexed);
        codeur.set_depth(png::BitDepth::Eight);
        codeur.set_palette(palette);
        codeur.set_trns(opacites);
        // Réglages FIXES : la même grille donne toujours les mêmes octets. Les filtres
        // ne servent à rien sur une image à palette : on n'en met aucun.
        codeur.set_compression(png::Compression::Best);
        codeur.set_filter(png::FilterType::NoFilter);
        codeur.set_adaptive_filter(png::AdaptiveFilterType::NonAdaptive);
        let mut ecrivain = codeur.write_header().map_err(|e| e.to_string())?;
        ecrivain
            .write_image_data(&donnees)
            .map_err(|e| e.to_string())?;
        ecrivain.finish().map_err(|e| e.to_string())?;
    }
    Ok(octets)
}

/// Caractère de grille d'un pixel RVBA : `.` s'il est transparent, sinon le chiffre
/// de sa couleur dans la palette (erreur si la couleur n'y est pas).
pub fn caractere(r: u8, g: u8, b: u8, a: u8) -> Result<char, String> {
    if a == 0 {
        return Ok('.');
    }
    if a != 255 {
        return Err(format!("pixel à demi transparent (alpha {a})"));
    }
    TOUTES
        .iter()
        .position(|c| (c.r, c.g, c.b) == (r, g, b))
        .map(|i| CHIFFRES[i] as char)
        .ok_or_else(|| format!("couleur hors palette #{r:02X}{g:02X}{b:02X}"))
}

/// Décode un PNG (n'importe quel PNG aux couleurs de la palette) en grille.
pub fn lire_png(octets: &[u8]) -> Result<Vec<String>, String> {
    let mut decodeur = png::Decoder::new(octets);
    // EXPAND : la palette devient du RVB (RVBA avec tRNS), les gris passent en 8 bits ;
    // STRIP_16 : les images 16 bits passent en 8 bits.
    decodeur.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut lecteur = decodeur.read_info().map_err(|e| e.to_string())?;
    let mut tampon = vec![0; lecteur.output_buffer_size()];
    let image = lecteur.next_frame(&mut tampon).map_err(|e| e.to_string())?;
    let canaux = match image.color_type {
        png::ColorType::Grayscale => 1,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        png::ColorType::Indexed => return Err("palette non développée".into()),
    };
    let mut grille = Vec::with_capacity(image.height as usize);
    for y in 0..image.height as usize {
        let rangee = &tampon[y * image.line_size..][..image.width as usize * canaux];
        let mut ligne = String::with_capacity(image.width as usize);
        for (x, p) in rangee.chunks(canaux).enumerate() {
            let (r, g, b, a) = match canaux {
                1 => (p[0], p[0], p[0], 255),
                2 => (p[0], p[0], p[0], p[1]),
                3 => (p[0], p[1], p[2], 255),
                _ => (p[0], p[1], p[2], p[3]),
            };
            ligne.push(caractere(r, g, b, a).map_err(|e| format!("pixel ({x}, {y}) : {e}"))?);
        }
        grille.push(ligne);
    }
    Ok(grille)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grille(lignes: &[&str]) -> Vec<String> {
        lignes.iter().map(|l| l.to_string()).collect()
    }

    #[test]
    fn une_grille_fait_l_aller_retour() {
        let g = grille(&[".0F.", "1234", "ABCD"]);
        let png = ecrire_png(&g).unwrap();
        assert_eq!(lire_png(&png).unwrap(), g);
        // Canonique : réécrire la grille relue redonne les mêmes octets.
        assert_eq!(ecrire_png(&lire_png(&png).unwrap()).unwrap(), png);
    }

    /// Tous les fichiers `.png` d'un dossier, récursivement.
    fn pngs(dossier: &std::path::Path, sortie: &mut Vec<std::path::PathBuf>) {
        for entree in std::fs::read_dir(dossier).unwrap().flatten() {
            let chemin = entree.path();
            if chemin.is_dir() {
                pngs(&chemin, sortie);
            } else if chemin.extension().is_some_and(|e| e == "png") {
                sortie.push(chemin);
            }
        }
    }

    #[test]
    fn chaque_sprite_du_depot_fait_l_aller_retour_aux_memes_octets() {
        // L'outil `sprite` est canonique sur TOUS les fichiers de assets/sprites :
        // lire puis réécrire redonne exactement les mêmes octets.
        let racine = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/sprites");
        let mut fichiers = Vec::new();
        pngs(&racine, &mut fichiers);
        assert!(
            fichiers.len() > 300,
            "{} fichiers seulement",
            fichiers.len()
        );
        for f in fichiers {
            let octets = std::fs::read(&f).unwrap();
            let grille = lire_png(&octets).unwrap_or_else(|e| panic!("{f:?} : {e}"));
            assert_eq!(ecrire_png(&grille).unwrap(), octets, "{f:?}");
        }
    }

    #[test]
    fn les_grilles_fautives_sont_refusees() {
        assert!(ecrire_png(&grille(&["00", "0"])).is_err());
        assert!(ecrire_png(&grille(&["0a"])).is_err());
        assert!(ecrire_png(&[]).is_err());
    }

    #[test]
    fn une_couleur_hors_palette_est_refusee() {
        assert_eq!(caractere(1, 2, 3, 0), Ok('.'));
        assert!(caractere(1, 2, 3, 255).is_err());
        let c = TOUTES[11];
        assert_eq!(caractere(c.r, c.g, c.b, 255), Ok('B'));
        assert!(caractere(c.r, c.g, c.b, 128).is_err());
    }
}
