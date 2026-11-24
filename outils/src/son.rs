//! Le format des sons de PÉPIN (carte 10.4e) : un fichier WAV PCM 16 bits MONO à
//! 22 050 Hz par son, dans `assets/musiques/<clé>.wav` (les boucles musicales) et
//! `assets/sons/<clé>.wav` (les bruitages et la pluie). Voir
//! `assets/musiques/LISEZMOI.md`.
//!
//! Pourquoi du WAV et pas de l'OGG Vorbis (essayé d'abord, 6 fois plus léger) :
//! Chromium ne coupe pas le remplissage de fin du Vorbis (un blanc de 7 à 22 ms à
//! chaque tour de boucle), et Safari sur iPhone lit mal l'OGG. Le WAV est lu
//! partout, à l'échantillon près. Le poids (3 Mo) reste HORS du wasm : les fichiers
//! sont servis à côté du jeu et téléchargés après l'écran titre.
//!
//! Ce module CONTRÔLE un fichier, il ne l'écrit pas :
//! - `lire_wav` lit un WAV et refuse tout autre format que celui du jeu ;
//! - `raccord` mesure la jonction d'une boucle (fin collée au début).
//!
//! Notions Rust : `as_chunks` (des paires d'octets), `Result` et `?` pour refuser un
//! fichier dès la première anomalie.

/// Fréquence d'échantillonnage de tous les sons du jeu (celle de l'ancien synthé).
pub const TAUX: u32 = 22_050;

/// Lit un WAV du jeu : PCM 16 bits, mono, `TAUX` Hz, et RIEN d'autre que les blocs
/// `fmt ` (de 16 octets) puis `data` (un en-tête de 44 octets en tout : aucune
/// métadonnée, ni bloc `LIST`, ni octet après les données). Rend les échantillons.
pub fn lire_wav(octets: &[u8]) -> Result<Vec<i16>, String> {
    if octets.len() < 44 || &octets[0..4] != b"RIFF" || &octets[8..16] != b"WAVEfmt " {
        return Err("pas un WAV simple (RIFF, WAVE, fmt en tête)".into());
    }
    let u16_a = |i: usize| u16::from_le_bytes([octets[i], octets[i + 1]]);
    let u32_a = |i: usize| u32::from_le_bytes(octets[i..i + 4].try_into().unwrap());
    let (taille_fmt, format, canaux, taux, bits) =
        (u32_a(16), u16_a(20), u16_a(22), u32_a(24), u16_a(34));
    if taille_fmt != 16 || format != 1 {
        return Err(format!(
            "bloc fmt de {taille_fmt} octets, format {format} (attendu : PCM simple)"
        ));
    }
    if canaux != 1 || bits != 16 || taux != TAUX {
        return Err(format!(
            "{canaux} canal(aux), {bits} bits, {taux} Hz (attendu : mono, 16 bits, {TAUX} Hz)"
        ));
    }
    if &octets[36..40] != b"data" {
        return Err(format!(
            "bloc {:?} avant les données (aucune métadonnée admise)",
            String::from_utf8_lossy(&octets[36..40])
        ));
    }
    let donnees = u32_a(40) as usize;
    if octets.len() != 44 + donnees || u32_a(4) as usize != octets.len() - 8 {
        return Err(format!(
            "{} octets pour {donnees} octets de données (attendu : 44 de plus, rien après)",
            octets.len()
        ));
    }
    // `as_chunks::<2>` : le contenu vu comme des paires d'octets.
    Ok(octets[44..]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&paire| i16::from_le_bytes(paire))
        .collect())
}

/// La jonction d'une boucle : en jouant le son en boucle, le dernier échantillon est
/// suivi du premier.
#[derive(Debug, Clone, Copy)]
pub struct Raccord {
    /// Saut au point de boucle : |premier - dernier|.
    pub saut: i32,
    /// Plus grand saut entre deux échantillons voisins À L'INTÉRIEUR du son.
    pub saut_max_interieur: i32,
    /// |premier échantillon| et plus grande valeur absolue des 10 derniers.
    pub debut: i32,
    pub fin: i32,
}

impl Raccord {
    /// Sans couture : le saut au point de boucle n'est pas plus fort que les sauts
    /// que le son fait déjà de lui-même (aucun clic ajouté par la boucle).
    pub fn sans_couture(&self) -> bool {
        self.saut <= self.saut_max_interieur
    }
}

/// Mesure le raccord de la boucle `s` (non vide).
pub fn raccord(s: &[i16]) -> Raccord {
    let premier = s[0] as i32;
    let dernier = s[s.len() - 1] as i32;
    Raccord {
        saut: (premier - dernier).abs(),
        saut_max_interieur: s
            .windows(2)
            .map(|w| (w[1] as i32 - w[0] as i32).abs())
            .max()
            .unwrap_or(0),
        debut: premier.abs(),
        fin: s[s.len().saturating_sub(10)..]
            .iter()
            .map(|&v| (v as i32).abs())
            .max()
            .unwrap_or(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    /// Les fichiers `.wav` de `assets/<dossier>`, triés.
    fn fichiers(dossier: &str) -> Vec<PathBuf> {
        let racine = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../assets")
            .join(dossier);
        let mut v: Vec<PathBuf> = std::fs::read_dir(&racine)
            .unwrap_or_else(|e| panic!("{}: {e}", racine.display()))
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "wav"))
            .collect();
        v.sort();
        v
    }

    /// Tous les sons du jeu : WAV mono 16 bits à 22 050 Hz sans métadonnées (`lire_wav`
    /// refuse le reste), non vides, 600 Ko au plus par musique (une boucle de 13,6 s)
    /// et 100 Ko par bruitage. Aucun autre fichier son (`.ogg`, `.mp3`) ne traîne.
    #[test]
    fn chaque_son_est_un_wav_mono_sans_metadonnees() {
        let tous: Vec<PathBuf> = [fichiers("musiques"), fichiers("sons")].concat();
        assert!(tous.len() >= 19, "{} fichiers", tous.len());
        for f in &tous {
            let octets = std::fs::read(f).unwrap();
            let ech = lire_wav(&octets).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
            assert!(!ech.is_empty(), "{}", f.display());
            let plafond = if f.parent().unwrap().ends_with("musiques") {
                600_000
            } else {
                100_000
            };
            assert!(
                octets.len() <= plafond,
                "{} : {} octets",
                f.display(),
                octets.len()
            );
        }
        for dossier in ["musiques", "sons"] {
            let racine = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../assets")
                .join(dossier);
            for e in std::fs::read_dir(&racine).unwrap().flatten() {
                let p = e.path();
                let ext = p.extension().and_then(|x| x.to_str()).unwrap_or("");
                assert!(
                    ext == "wav" || ext == "md",
                    "fichier inattendu : {}",
                    p.display()
                );
            }
        }
    }

    /// Les boucles (les musiques et la pluie) se raccordent sans clic : le saut au
    /// point de boucle ne dépasse pas les sauts du son lui-même ; une musique part et
    /// finit presque dans le silence (moins de 2 % du plein volume, comme le
    /// demandait déjà le test de l'ancienne partition).
    #[test]
    fn chaque_boucle_se_raccorde_sans_couture() {
        let pluie = Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/sons/pluie.wav");
        for f in fichiers("musiques").iter().chain([&pluie]) {
            let ech = lire_wav(&std::fs::read(f).unwrap()).unwrap();
            let r = raccord(&ech);
            assert!(r.sans_couture(), "{} : {r:?}", f.display());
            if f != &pluie {
                let deux_pour_cent = i16::MAX as i32 / 50;
                assert!(
                    r.debut < deux_pour_cent && r.fin < deux_pour_cent,
                    "{} : {r:?}",
                    f.display()
                );
            }
        }
    }

    /// Empreinte FNV-1a 64 bits des octets (assez pour reconnaître un fichier, sans
    /// dépendance : ce n'est pas une protection, seulement une comparaison).
    fn fnv1a(octets: &[u8]) -> u64 {
        octets.iter().fold(0xcbf2_9ce4_8422_2325, |h, &o| {
            (h ^ o as u64).wrapping_mul(0x0100_0000_01b3)
        })
    }

    /// ÉQUIVALENCE (carte 10.4e) : chaque fichier est, à l'échantillon près, le rendu
    /// du synthé 8 bits d'origine (commit ef0283c, test ignoré
    /// `audio::tests::rendre_les_wav_de_reference`, avant le retrait du synthé) :
    /// même nombre d'échantillons et même empreinte des données. Le synthé n'existe
    /// plus : ces empreintes sont la trace de ce rendu. Un son REMPLACÉ exprès change
    /// sa ligne ici (voir `assets/musiques/LISEZMOI.md`).
    #[test]
    fn chaque_son_est_le_rendu_du_synthe_d_origine() {
        const RENDU: [(&str, usize, u64); 19] = [
            ("musiques/cyclone", 211680, 0x0a51_4282_3b00_fe27),
            ("musiques/orage", 211680, 0x5f34_97cd_ea52_23d0),
            ("musiques/poursuite", 254015, 0x9667_acdd_5da6_971d),
            ("musiques/soleil", 254016, 0x54b5_3465_3b71_c6d9),
            ("musiques/toits_de_nuit", 254016, 0x261f_815b_d13b_f9b5),
            ("musiques/valse_des_nuages", 232847, 0xefc8_728e_98cc_98c3),
            ("sons/atterrissage", 1543, 0x09e7_7b97_563e_6cea),
            ("sons/eclair", 16537, 0x2386_0d2c_fdbf_c8ef),
            ("sons/ecrase", 2646, 0x15e5_daf8_7aa9_af76),
            ("sons/goutte", 5071, 0xf19a_51be_b79c_7032),
            ("sons/lanterne", 8820, 0xf93e_dd8f_3cc8_c650),
            ("sons/menu", 1102, 0x6988_9010_45d4_5e99),
            ("sons/mort", 12127, 0x20a6_2a84_4b88_a3a2),
            ("sons/plane", 3528, 0x131b_3bf6_afaa_dbb6),
            ("sons/pluie", 44100, 0x7db8_bee1_9d93_2fc8),
            ("sons/rebond", 4851, 0x25ed_7574_afde_3cff),
            ("sons/saut", 3087, 0xb011_814f_b8e2_f8aa),
            ("sons/secret", 7937, 0xc6a0_cf1c_5516_b8ef),
            ("sons/victoire", 18301, 0x5088_c099_44af_27b2),
        ];
        let racine = Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets");
        let presents: Vec<PathBuf> = [fichiers("musiques"), fichiers("sons")].concat();
        assert_eq!(presents.len(), RENDU.len());
        for (nom, longueur, empreinte) in RENDU {
            let octets = std::fs::read(racine.join(format!("{nom}.wav"))).unwrap();
            let ech = lire_wav(&octets).unwrap();
            assert_eq!(ech.len(), longueur, "{nom} : longueur");
            let fnv = fnv1a(&octets[44..]);
            assert!(
                fnv == empreinte,
                "{nom} : échantillons différents (empreinte {fnv:#018x})"
            );
        }
    }

    #[test]
    fn le_raccord_se_calcule() {
        let r = raccord(&[0, 500, -500, 10]);
        assert_eq!(
            (r.saut, r.saut_max_interieur, r.debut, r.fin),
            (10, 1000, 0, 500)
        );
        assert!(r.sans_couture());
    }

    /// En-tête minimal : RIFF, WAVE, fmt (mono 16 bits, 22 050 Hz), data (2 éch.).
    fn wav(canaux: u16, taux: u32, donnees: &[u8]) -> Vec<u8> {
        let mut w = b"RIFF".to_vec();
        w.extend_from_slice(&(36 + donnees.len() as u32).to_le_bytes());
        w.extend_from_slice(b"WAVEfmt ");
        w.extend_from_slice(&16u32.to_le_bytes());
        w.extend_from_slice(&1u16.to_le_bytes());
        w.extend_from_slice(&canaux.to_le_bytes());
        w.extend_from_slice(&taux.to_le_bytes());
        w.extend_from_slice(&(taux * 2 * canaux as u32).to_le_bytes());
        w.extend_from_slice(&[2 * canaux as u8, 0, 16, 0]);
        w.extend_from_slice(b"data");
        w.extend_from_slice(&(donnees.len() as u32).to_le_bytes());
        w.extend_from_slice(donnees);
        w
    }

    #[test]
    fn le_wav_se_relit_et_le_reste_est_refuse() {
        let bon = wav(1, TAUX, &[0x10, 0x00, 0xF0, 0xFF]);
        assert_eq!(lire_wav(&bon).unwrap(), vec![16, -16]);
        assert!(lire_wav(&wav(2, TAUX, &[0; 4])).is_err(), "stéréo");
        assert!(
            lire_wav(&wav(1, 44_100, &[0; 4])).is_err(),
            "autre fréquence"
        );
        let mut traine = bon.clone();
        traine.extend_from_slice(b"LIST\x04\0\0\0INFO");
        assert!(lire_wav(&traine).is_err(), "bloc après les données");
        assert!(lire_wav(b"OggS").is_err());
    }
}
