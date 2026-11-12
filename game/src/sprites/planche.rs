//! Planche contact : rassemble TOUS les sprites dans une image PNG (`docs/planche-sprites.png`)
//! pour pouvoir les relire à l'œil. Compilé uniquement pour les tests.
//!
//! Le PNG est écrit à la main (indexé 4 bits, blocs « stored » non compressés) :
//! aucune dépendance d'image, comme tout le reste du projet.

use super::{hauteur, largeur, pixel, tous, Image};
use pepin_core::palette::{Couleur, ARDOISE_NUIT, TOUTES};

/// CRC-32 d'un bloc (exigé par le format PNG).
fn crc32(octets: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &o in octets {
        crc ^= o as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

/// Somme de contrôle Adler-32 des données (fin du flux zlib).
fn adler32(octets: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &o in octets {
        a = (a + o as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn bloc(sortie: &mut Vec<u8>, nom: &[u8; 4], donnees: &[u8]) {
    sortie.extend_from_slice(&(donnees.len() as u32).to_be_bytes());
    let mut corps = nom.to_vec();
    corps.extend_from_slice(donnees);
    sortie.extend_from_slice(&corps);
    sortie.extend_from_slice(&crc32(&corps).to_be_bytes());
}

/// Encode une image indexée 4 bits (`indices` : un octet 0-15 par pixel, `largeur` x
/// `hauteur`) en PNG avec la palette du jeu. 16 couleurs = 4 bits : le fichier
/// est 6 fois plus léger qu'un PNG RVB non compressé.
pub fn encoder_png(largeur: u32, hauteur: u32, indices: &[u8]) -> Vec<u8> {
    let mut brut = Vec::new();
    for ligne in indices.chunks(largeur as usize) {
        brut.push(0); // filtre « aucun »
        for paire in ligne.chunks(2) {
            // Deux pixels par octet : le premier dans les 4 bits de poids fort.
            brut.push((paire[0] << 4) | paire.get(1).copied().unwrap_or(0));
        }
    }
    // Flux zlib : en-tête, blocs non compressés de 65535 octets au plus, Adler-32.
    let mut zlib = vec![0x78, 0x01];
    let morceaux: Vec<&[u8]> = brut.chunks(65535).collect();
    for (i, m) in morceaux.iter().enumerate() {
        zlib.push(u8::from(i + 1 == morceaux.len()));
        zlib.extend_from_slice(&(m.len() as u16).to_le_bytes());
        zlib.extend_from_slice(&(!(m.len() as u16)).to_le_bytes());
        zlib.extend_from_slice(m);
    }
    zlib.extend_from_slice(&adler32(&brut).to_be_bytes());

    let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&largeur.to_be_bytes());
    ihdr.extend_from_slice(&hauteur.to_be_bytes());
    ihdr.extend_from_slice(&[4, 3, 0, 0, 0]); // 4 bits, palette
    bloc(&mut png, b"IHDR", &ihdr);
    let plte: Vec<u8> = TOUTES.iter().flat_map(|c| [c.r, c.g, c.b]).collect();
    bloc(&mut png, b"PLTE", &plte);
    bloc(&mut png, b"IDAT", &zlib);
    bloc(&mut png, b"IEND", &[]);
    png
}

/// Indice d'une couleur dans la palette du jeu.
fn indice(c: Couleur) -> u8 {
    TOUTES.iter().position(|&t| t == c).unwrap_or(0) as u8
}

/// Dessine tous les sprites en grille sur fond ardoise, agrandis `echelle` fois.
/// Renvoie `(largeur, hauteur, indices de palette)`.
fn planche(sprites: &[(&str, Image)], colonnes: usize, echelle: usize) -> (u32, u32, Vec<u8>) {
    let marge = 3;
    let cellule_l = sprites.iter().map(|(_, d)| largeur(d)).max().unwrap_or(1) + marge;
    let cellule_h = sprites.iter().map(|(_, d)| hauteur(d)).max().unwrap_or(1) + marge;
    let lignes = sprites.len().div_ceil(colonnes);
    let (l, h) = (colonnes * cellule_l + marge, lignes * cellule_h + marge);
    let mut pixels = vec![indice(ARDOISE_NUIT); l * h];
    for (i, (_, dessin)) in sprites.iter().enumerate() {
        let (ox, oy) = (
            marge + (i % colonnes) * cellule_l,
            marge + (i / colonnes) * cellule_h,
        );
        for (y, ligne) in dessin.iter().enumerate() {
            for (x, c) in ligne.chars().enumerate() {
                if let Ok(Some(couleur)) = pixel(c) {
                    pixels[(oy + y) * l + ox + x] = indice(couleur);
                }
            }
        }
    }
    // Agrandissement entier (chaque pixel devient un carré net).
    let mut sortie = Vec::with_capacity(l * h * echelle * echelle);
    for y in 0..h {
        let mut rangee = Vec::with_capacity(l * echelle);
        for x in 0..l {
            rangee.extend(std::iter::repeat(pixels[y * l + x]).take(echelle));
        }
        for _ in 0..echelle {
            sortie.extend_from_slice(&rangee);
        }
    }
    ((l * echelle) as u32, (h * echelle) as u32, sortie)
}

#[test]
fn ecrit_la_planche_contact_dans_docs() {
    let sprites = tous();
    let (l, h, indices) = planche(&sprites, 10, 4);
    let png = encoder_png(l, h, &indices);
    assert_eq!(&png[1..4], b"PNG");
    let dossier = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs");
    std::fs::create_dir_all(&dossier).unwrap();
    std::fs::write(dossier.join("planche-sprites.png"), png).unwrap();
}

#[test]
fn le_codeur_png_connait_les_sommes_de_controle() {
    // Valeurs de référence : CRC-32 de « 123456789 » et Adler-32 de « Wikipedia ».
    assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
}
