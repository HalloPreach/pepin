//! Le format des sons de PÉPIN (carte 10.4e) : un fichier OGG Vorbis MONO par son,
//! dans `assets/musiques/<clé>.ogg` (les boucles musicales) et `assets/sons/<clé>.ogg`
//! (les bruitages et la pluie). Voir `assets/musiques/LISEZMOI.md`.
//!
//! Ce module CONTRÔLE un fichier, il ne l'écrit pas (l'encodage est fait par ffmpeg,
//! script `outils/sons.sh`) :
//! - `lire_ogg` décode le fichier (crate `lewton`, le décodeur que macroquad utilise
//!   déjà hors du web) et rend ses en-têtes : canaux, fréquence, commentaires ;
//! - `lire_wav` lit un WAV 16 bits mono (le WAV de RÉFÉRENCE d'un son) ;
//! - `signal_bruit_db` mesure l'écart entre le son décodé et sa référence ;
//! - `raccord` mesure la jonction d'une boucle (fin collée au début).
//!
//! Notions Rust : `std::io::Cursor` (lire des octets en mémoire comme un fichier),
//! `while let Some(..)` pour vider un décodeur paquet par paquet, `i64` pour sommer
//! des carrés sans débordement.

use std::io::Cursor;

use lewton::inside_ogg::OggStreamReader;

/// Fréquence d'échantillonnage de tous les sons du jeu (celle de l'ancien synthé).
pub const TAUX: u32 = 22_050;

/// Un OGG Vorbis décodé.
pub struct Ogg {
    pub canaux: u8,
    pub taux: u32,
    /// Chaîne d'identification de la bibliothèque d'encodage, OBLIGATOIRE dans le
    /// format Vorbis (« Xiph.Org libVorbis... ») : ce n'est pas une métadonnée ajoutée.
    pub fabricant: String,
    /// Commentaires Vorbis (titre, ENCODER...) : doivent être VIDES.
    pub commentaires: Vec<(String, String)>,
    /// Échantillons décodés (entrelacés si plusieurs canaux).
    pub echantillons: Vec<i16>,
}

/// Décode un fichier OGG Vorbis entier.
pub fn lire_ogg(octets: &[u8]) -> Result<Ogg, String> {
    let mut lecteur =
        OggStreamReader::new(Cursor::new(octets)).map_err(|e| format!("OGG illisible : {e:?}"))?;
    let mut echantillons = Vec::new();
    while let Some(paquet) = lecteur
        .read_dec_packet_itl()
        .map_err(|e| format!("paquet illisible : {e:?}"))?
    {
        echantillons.extend_from_slice(&paquet);
    }
    Ok(Ogg {
        canaux: lecteur.ident_hdr.audio_channels,
        taux: lecteur.ident_hdr.audio_sample_rate,
        fabricant: lecteur.comment_hdr.vendor.clone(),
        commentaires: lecteur.comment_hdr.comment_list.clone(),
        echantillons,
    })
}

/// Lit un WAV PCM 16 bits mono : (fréquence, échantillons).
pub fn lire_wav(octets: &[u8]) -> Result<(u32, Vec<i16>), String> {
    if octets.len() < 12 || &octets[0..4] != b"RIFF" || &octets[8..12] != b"WAVE" {
        return Err("pas un fichier WAV".into());
    }
    let mut taux = 0;
    let mut i = 12;
    // Un WAV est une suite de blocs (identifiant de 4 lettres, taille, contenu).
    while i + 8 <= octets.len() {
        let id = &octets[i..i + 4];
        let taille = u32::from_le_bytes(octets[i + 4..i + 8].try_into().unwrap()) as usize;
        let contenu = octets
            .get(i + 8..i + 8 + taille)
            .ok_or("bloc WAV tronqué")?;
        if id == b"fmt " {
            let canaux = u16::from_le_bytes(contenu[2..4].try_into().unwrap());
            let bits = u16::from_le_bytes(contenu[14..16].try_into().unwrap());
            if canaux != 1 || bits != 16 {
                return Err(format!(
                    "WAV {canaux} canaux, {bits} bits (attendu : mono 16 bits)"
                ));
            }
            taux = u32::from_le_bytes(contenu[4..8].try_into().unwrap());
        } else if id == b"data" {
            let ech = contenu
                .chunks_exact(2)
                .map(|c| i16::from_le_bytes([c[0], c[1]]))
                .collect();
            return Ok((taux, ech));
        }
        i += 8 + taille + (taille & 1);
    }
    Err("WAV sans bloc de données".into())
}

/// Rapport signal sur bruit (en dB) du son `decode` par rapport à sa `reference` :
/// énergie de la référence divisée par l'énergie de la DIFFÉRENCE, échantillon par
/// échantillon. 30 dB : l'écart est 1 000 fois moins fort que le son.
pub fn signal_bruit_db(reference: &[i16], decode: &[i16]) -> f64 {
    let n = reference.len().min(decode.len());
    let carre = |v: i64| v * v;
    let signal: i64 = reference[..n].iter().map(|&v| carre(v as i64)).sum();
    let bruit: i64 = (0..n)
        .map(|k| carre(reference[k] as i64 - decode[k] as i64))
        .sum();
    if bruit == 0 {
        return f64::INFINITY;
    }
    10.0 * (signal as f64 / bruit as f64).log10()
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

/// Mots qui ne doivent JAMAIS paraître dans un fichier son publié : les marques que
/// ffmpeg ajoute sans `-fflags +bitexact` (« Lavf », « Lavc »), les commentaires
/// usuels (« encoder=Lavc libvorbis » que ffmpeg ajoute), un chemin de la machine.
pub const MOTS_INTERDITS: [&[u8]; 7] = [
    b"Lavf", b"Lavc", b"encoder", b"ENCODER", b"TITLE", b"ARTIST", b"/home",
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    /// Les fichiers `.ogg` de `assets/<dossier>`, triés.
    fn fichiers(dossier: &str) -> Vec<PathBuf> {
        let racine = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../assets")
            .join(dossier);
        let mut v: Vec<PathBuf> = std::fs::read_dir(&racine)
            .unwrap_or_else(|e| panic!("{}: {e}", racine.display()))
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "ogg"))
            .collect();
        v.sort();
        v
    }

    /// Tous les sons du jeu : OGG Vorbis mono à 22 050 Hz, aucun commentaire, aucun
    /// mot interdit dans les octets, décodables, 64 Ko au plus chacun (une boucle
    /// de 12 s en pèse environ 80 : les musiques ont leur plafond plus bas).
    #[test]
    fn chaque_son_est_un_ogg_mono_sans_metadonnees() {
        let tous: Vec<PathBuf> = [fichiers("musiques"), fichiers("sons")].concat();
        assert!(tous.len() >= 19, "{} fichiers", tous.len());
        for f in &tous {
            let octets = std::fs::read(f).unwrap();
            let ogg = lire_ogg(&octets).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
            assert_eq!(ogg.canaux, 1, "{} : mono", f.display());
            assert_eq!(ogg.taux, TAUX, "{}", f.display());
            assert!(
                ogg.commentaires.is_empty(),
                "{} : {:?}",
                f.display(),
                ogg.commentaires
            );
            assert!(!ogg.echantillons.is_empty());
            for mot in MOTS_INTERDITS {
                assert!(
                    !octets.windows(mot.len()).any(|w| w == mot),
                    "{} contient {:?}",
                    f.display(),
                    String::from_utf8_lossy(mot)
                );
            }
            let plafond = if f.parent().unwrap().ends_with("musiques") {
                100_000
            } else {
                32_000
            };
            assert!(
                octets.len() <= plafond,
                "{} : {} octets",
                f.display(),
                octets.len()
            );
        }
    }

    /// Les boucles (les musiques et la pluie) se raccordent sans clic : le saut au
    /// point de boucle ne dépasse pas les sauts du son lui-même ; une musique part et
    /// finit presque dans le silence (moins de 2 % du plein volume, comme le
    /// demandait déjà le test de l'ancienne partition).
    #[test]
    fn chaque_boucle_se_raccorde_sans_couture() {
        let pluie = Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/sons/pluie.ogg");
        for f in fichiers("musiques").iter().chain([&pluie]) {
            let ogg = lire_ogg(&std::fs::read(f).unwrap()).unwrap();
            let r = raccord(&ogg.echantillons);
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

    #[test]
    fn le_signal_bruit_et_le_raccord_se_calculent() {
        let a = [1000_i16, -1000, 1000, -1000];
        assert!(signal_bruit_db(&a, &a).is_infinite());
        // Écart de 10 % en amplitude : 20 dB.
        let b = [900_i16, -900, 900, -900];
        assert!((signal_bruit_db(&a, &b) - 20.0).abs() < 1e-9);
        let r = raccord(&[0, 500, -500, 10]);
        assert_eq!(
            (r.saut, r.saut_max_interieur, r.debut, r.fin),
            (10, 1000, 0, 500)
        );
        assert!(r.sans_couture());
    }

    #[test]
    fn le_wav_se_relit() {
        // En-tête minimal : RIFF, WAVE, fmt (mono 16 bits, 22 050 Hz), data (2 éch.).
        let mut w = b"RIFF\0\0\0\0WAVEfmt ".to_vec();
        w.extend_from_slice(&16u32.to_le_bytes());
        w.extend_from_slice(&[1, 0, 1, 0]);
        w.extend_from_slice(&TAUX.to_le_bytes());
        w.extend_from_slice(&(TAUX * 2).to_le_bytes());
        w.extend_from_slice(&[2, 0, 16, 0]);
        w.extend_from_slice(b"data");
        w.extend_from_slice(&4u32.to_le_bytes());
        w.extend_from_slice(&[0x10, 0x00, 0xF0, 0xFF]);
        assert_eq!(lire_wav(&w).unwrap(), (TAUX, vec![16, -16]));
    }
}
