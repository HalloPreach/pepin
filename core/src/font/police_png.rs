//! Preuve visuelle de la police : rend des phrases de recette en PNG, avec un
//! encodeur PNG minimal écrit à la main (pas de dépendance : blocs deflate
//! « stockés », donc non compressés).
//!
//! Le test `ecrit_docs_police_test_png` est ignoré par défaut (il écrit dans
//! le dépôt). Pour régénérer l'image :
//! `./docker-cargo.sh test -p core -- --ignored ecrit_docs_police_test_png`

use super::*;

/// Lignes de la planche : (texte, échelle).
const LIGNES: &[(&str, u32)] = &[
    ("Au-dessus des nuages", 4),
    ("Prends ton temps : il n'y a plus d'orage.", 4),
    ("flaques rôdent entre les cheminées", 4),
    ("Pépin plane, glisse et gagne", 4),
    ("abcdefghijklmnopqrstuvwxyz", 4),
    ("g p q y j / o a e : nuages temps d'orage flaques", 3),
    ("çà éèêë ùûü ôö îï œ ÿ", 4),
];

const MARGE: u32 = 16;
/// Espace vertical entre deux lignes (en pixels de l'image).
const INTERLIGNE: u32 = 14;
const FOND: [u8; 3] = [0x1b, 0x1f, 0x2e];
const ENCRE: [u8; 3] = [0xf2, 0xea, 0xd3];

/// Image RGB 8 bits : largeur, hauteur, pixels (3 octets par pixel).
struct Image {
    largeur: u32,
    hauteur: u32,
    rvb: Vec<u8>,
}

/// Dessine toutes les lignes de la planche.
fn dessiner_planche() -> Image {
    let largeur = LIGNES
        .iter()
        .map(|(t, e)| largeur_texte(t, *e))
        .max()
        .unwrap()
        + 2 * MARGE;
    let hauteur = LIGNES
        .iter()
        .map(|(_, e)| hauteur_texte(*e) + INTERLIGNE)
        .sum::<u32>()
        + 2 * MARGE
        - INTERLIGNE;
    let mut rvb = FOND.repeat((largeur * hauteur) as usize);
    let mut y0 = MARGE;
    for (texte, echelle) in LIGNES {
        for (x, y) in pixels_texte(texte) {
            for dy in 0..*echelle {
                for dx in 0..*echelle {
                    let px = MARGE + x * echelle + dx;
                    let py = y0 + y * echelle + dy;
                    let i = ((py * largeur + px) * 3) as usize;
                    rvb[i..i + 3].copy_from_slice(&ENCRE);
                }
            }
        }
        y0 += hauteur_texte(*echelle) + INTERLIGNE;
    }
    Image {
        largeur,
        hauteur,
        rvb,
    }
}

/// CRC-32 (polynôme PNG/zlib).
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

/// Somme de contrôle Adler-32 (fin du flux zlib).
fn adler32(octets: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &o in octets {
        a = (a + o as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

/// Ajoute un bloc PNG (longueur, type, données, CRC).
fn ajouter_bloc(sortie: &mut Vec<u8>, type_bloc: &[u8; 4], donnees: &[u8]) {
    sortie.extend_from_slice(&(donnees.len() as u32).to_be_bytes());
    let debut = sortie.len();
    sortie.extend_from_slice(type_bloc);
    sortie.extend_from_slice(donnees);
    let crc = crc32(&sortie[debut..]);
    sortie.extend_from_slice(&crc.to_be_bytes());
}

/// Encode une image RVB en PNG (sans compression).
fn encoder_png(image: &Image) -> Vec<u8> {
    // Chaque rangée est précédée de son octet de filtre (0 = aucun).
    let mut brut = Vec::with_capacity(image.rvb.len() + image.hauteur as usize);
    for rangee in image.rvb.chunks((image.largeur * 3) as usize) {
        brut.push(0);
        brut.extend_from_slice(rangee);
    }
    // Flux zlib : en-tête, blocs « stockés » de 65535 octets au plus, Adler-32.
    let mut zlib = vec![0x78, 0x01];
    let mut morceaux = brut.chunks(65535).peekable();
    while let Some(morceau) = morceaux.next() {
        zlib.push(u8::from(morceaux.peek().is_none())); // BFINAL, BTYPE = 00
        zlib.extend_from_slice(&(morceau.len() as u16).to_le_bytes());
        zlib.extend_from_slice(&(!(morceau.len() as u16)).to_le_bytes());
        zlib.extend_from_slice(morceau);
    }
    zlib.extend_from_slice(&adler32(&brut).to_be_bytes());

    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&image.largeur.to_be_bytes());
    ihdr.extend_from_slice(&image.hauteur.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]); // 8 bits, RVB, sans entrelacement

    let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    ajouter_bloc(&mut png, b"IHDR", &ihdr);
    ajouter_bloc(&mut png, b"IDAT", &zlib);
    ajouter_bloc(&mut png, b"IEND", &[]);
    png
}

#[test]
fn crc32_et_adler32_donnent_les_valeurs_de_reference() {
    assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
}

#[test]
fn le_png_de_la_planche_est_bien_forme() {
    let image = dessiner_planche();
    let png = encoder_png(&image);
    assert_eq!(&png[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
    assert_eq!(&png[12..16], b"IHDR");
    assert_eq!(
        u32::from_be_bytes(png[16..20].try_into().unwrap()),
        image.largeur
    );
    assert_eq!(
        u32::from_be_bytes(png[20..24].try_into().unwrap()),
        image.hauteur
    );
    assert_eq!(&png[png.len() - 8..png.len() - 4], b"IEND");
    // Il y a bien de l'encre sur la planche.
    assert!(image.rvb.chunks(3).any(|p| p == ENCRE));
}

#[test]
#[ignore = "écrit docs/police-test.png dans le dépôt"]
fn ecrit_docs_police_test_png() {
    let chemin = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/police-test.png");
    std::fs::write(&chemin, encoder_png(&dessiner_planche())).expect("écriture du PNG");
    println!("PNG écrit : {}", chemin.display());
}
