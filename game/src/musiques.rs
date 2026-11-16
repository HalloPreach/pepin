//! Catalogue des musiques de PÉPIN : la boucle d'origine (« Orage », gratuite) et
//! trois musiques à débloquer dans la boutique (25 gouttes d'or chacune). C'est
//! purement cosmétique : la musique ne change rien au jeu ni au score.
//!
//! Chaque nouvelle musique est une `Partition` : quelques voix (mélodie, basse,
//! accompagnement, charleston de bruit) écrites en notes MIDI et en croches, puis
//! « jouées » par le même synthétiseur 8 bits que tous les autres sons
//! (`pepin_core::synthese`) : aucun fichier audio, aucune dépendance.
//!
//! Ce module ne fait que le catalogue et la fabrication des échantillons. Le choix
//! du joueur (et sa sauvegarde) sera branché par la carte de la boutique : pour
//! l'instant, la musique jouée est le simple paramètre `musique_active` de
//! `audio::Audio` (par défaut : Orage).
//!
//! Notions Rust illustrées ici : `enum` + tableau constant `TOUTES` pour un
//! catalogue, structures `const` imbriquées (`&'static [Voix]`), itérateurs
//! (`iter().map(..).sum()`), `Option` pour « pas de partition écrite ici ».

use macroquad::input::{is_key_pressed, KeyCode};
use pepin_core::sons;
use pepin_core::synthese::{ajouter, midi_vers_hz, Bruit, Note, Onde, TAUX};

/// Une musique du catalogue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Musique {
    /// La boucle d'origine du jeu (calme, la mineur), toujours débloquée.
    Orage,
    /// Nocturne : arpèges lents en ré mineur, mélodie rêveuse.
    ToitsDeNuit,
    /// Entraînante : do majeur rapide, basse « oum-pa » et charleston.
    Soleil,
    /// Valse aérienne à trois temps, sol majeur.
    ValseDesNuages,
}

// Les identifiants, noms et prix ne servent pas encore dans le jeu : la carte de la
// boutique les affichera et les enregistrera. Seuls les tests les lisent pour l'instant.
#[cfg_attr(not(test), allow(dead_code))]
impl Musique {
    /// Tout le catalogue, dans l'ordre d'affichage (et de la touche de QA).
    pub const TOUTES: [Musique; 4] = [
        Musique::Orage,
        Musique::ToitsDeNuit,
        Musique::Soleil,
        Musique::ValseDesNuages,
    ];

    /// Identifiant texte STABLE (sauvegarde) : ne jamais le changer une fois publié.
    pub fn id(self) -> &'static str {
        match self {
            Musique::Orage => "orage",
            Musique::ToitsDeNuit => "toits_de_nuit",
            Musique::Soleil => "soleil",
            Musique::ValseDesNuages => "valse_des_nuages",
        }
    }

    /// Retrouve une musique par son identifiant (`None` si inconnu).
    pub fn depuis_id(id: &str) -> Option<Musique> {
        Musique::TOUTES.into_iter().find(|m| m.id() == id)
    }

    /// Nom affiché (la police du jeu n'a que des capitales).
    pub fn nom(self) -> &'static str {
        match self {
            Musique::Orage => "ORAGE",
            Musique::ToitsDeNuit => "TOITS DE NUIT",
            Musique::Soleil => "SOLEIL",
            Musique::ValseDesNuages => "VALSE DES NUAGES",
        }
    }

    /// Prix en gouttes d'or (0 = toujours débloquée).
    pub fn prix(self) -> u32 {
        match self {
            Musique::Orage => 0,
            _ => 25,
        }
    }

    /// La musique suivante du catalogue (après la dernière : la première).
    pub fn suivante(self) -> Musique {
        let i = Musique::TOUTES.iter().position(|&m| m == self).unwrap_or(0);
        Musique::TOUTES[(i + 1) % Musique::TOUTES.len()]
    }

    /// La partition écrite dans ce module. `None` pour Orage, dont la partition
    /// d'origine reste dans `pepin_core::sons` (inchangée, au bit près).
    pub fn partition(self) -> Option<&'static Partition> {
        match self {
            Musique::Orage => None,
            Musique::ToitsDeNuit => Some(&TOITS_DE_NUIT),
            Musique::Soleil => Some(&SOLEIL),
            Musique::ValseDesNuages => Some(&VALSE_DES_NUAGES),
        }
    }

    /// Les échantillons d'UN tour de boucle (entre -1 et 1).
    pub fn echantillons(self) -> Vec<f32> {
        match self.partition() {
            Some(p) => p.rendre(),
            None => sons::musique(),
        }
    }
}

/// Une voix : une suite de notes jouées l'une après l'autre avec le même timbre.
pub struct Voix {
    pub onde: Onde,
    pub volume: f32,
    /// Exposant de décroissance de chaque note (voir `Note::declin`).
    pub declin: f32,
    /// Part de la durée écrite réellement tenue (< 1 : un petit silence détache
    /// les notes, et la dernière s'éteint AVANT la fin de la boucle : pas de clic).
    pub tenue: f32,
    /// (note MIDI, durée en croches). `0` = silence.
    pub notes: &'static [(u8, u8)],
    /// Nombre de fois que `notes` est joué d'affilée (motif répété).
    pub fois: u32,
}

impl Voix {
    /// Durée totale de la voix, en croches.
    pub fn croches(&self) -> u32 {
        self.notes.iter().map(|&(_, c)| c as u32).sum::<u32>() * self.fois
    }
}

/// Une partition : un tempo et des voix jouées ensemble. Toutes les voix ont la
/// même durée (vérifié par le test) : c'est la durée exacte de la boucle.
pub struct Partition {
    /// Durée d'une croche, en secondes.
    pub croche: f32,
    pub voix: &'static [Voix],
}

impl Partition {
    /// Durée d'un tour de boucle, en croches (celle de la première voix).
    pub fn croches(&self) -> u32 {
        self.voix.first().map_or(0, Voix::croches)
    }

    /// Joue toutes les voix et les mélange. Le départ de chaque note est calculé
    /// à partir de son rang en croches (entier) : aucune erreur d'arrondi ne
    /// s'accumule, la boucle retombe juste.
    pub fn rendre(&self) -> Vec<f32> {
        let mut bruit = Bruit::new();
        let mut son = Vec::new();
        for v in self.voix {
            let mut rang = 0_u32;
            for _ in 0..v.fois {
                for &(midi, croches) in v.notes {
                    if midi != 0 {
                        let duree = croches as f32 * self.croche * v.tenue;
                        let note = Note::fixe(v.onde, midi_vers_hz(midi), duree, v.volume)
                            .declin(v.declin);
                        ajouter(
                            &mut son,
                            &note.rendre(&mut bruit),
                            rang as f32 * self.croche,
                        );
                    }
                    rang += croches as u32;
                }
            }
        }
        son.resize(
            (self.croches() as f32 * self.croche * TAUX as f32) as usize,
            0.0,
        );
        son
    }
}

// --- Les timbres -------------------------------------------------------------

/// Carré creux et plein.
const CARRE: Onde = Onde::Carre { rapport: 0.5 };
/// Carré nasillard (celui de la mélodie d'Orage).
const NASILLARD: Onde = Onde::Carre { rapport: 0.25 };
/// Carré très fin, léger, presque une flûte.
const FIN: Onde = Onde::Carre { rapport: 0.125 };

// --- Toits de nuit : nocturne en ré mineur (4 mesures de 8 croches, ~ 11,5 s) ---

/// Ré mineur, si bémol, sol mineur, la : arpèges égrenés une croche à la fois.
const TOITS_ARPEGES: &[(u8, u8)] = &[
    (62, 1),
    (65, 1),
    (69, 1),
    (65, 1),
    (62, 1),
    (65, 1),
    (69, 1),
    (65, 1),
    (58, 1),
    (62, 1),
    (65, 1),
    (62, 1),
    (58, 1),
    (62, 1),
    (65, 1),
    (62, 1),
    (55, 1),
    (58, 1),
    (62, 1),
    (58, 1),
    (55, 1),
    (58, 1),
    (62, 1),
    (58, 1),
    (57, 1),
    (61, 1),
    (64, 1),
    (61, 1),
    (57, 1),
    (61, 1),
    (64, 1),
    (61, 1),
];

const TOITS_MELODIE: &[(u8, u8)] = &[
    (74, 4),
    (72, 2),
    (69, 2),
    (70, 6),
    (0, 2),
    (67, 2),
    (70, 2),
    (74, 2),
    (72, 2),
    (73, 6),
    (0, 2),
];

const TOITS_BASSE: &[(u8, u8)] = &[(50, 8), (46, 8), (43, 8), (45, 8)];

static TOITS_DE_NUIT: Partition = Partition {
    croche: 0.36,
    voix: &[
        Voix {
            onde: FIN,
            volume: 0.16,
            declin: 0.8,
            tenue: 0.9,
            notes: TOITS_MELODIE,
            fois: 1,
        },
        Voix {
            onde: CARRE,
            volume: 0.06,
            declin: 2.5,
            tenue: 0.9,
            notes: TOITS_ARPEGES,
            fois: 1,
        },
        Voix {
            onde: Onde::Triangle,
            volume: 0.44,
            declin: 0.7,
            tenue: 0.95,
            notes: TOITS_BASSE,
            fois: 1,
        },
    ],
};

// --- Soleil : entraînante en do majeur (8 mesures de 8 croches, ~ 11,5 s) -------

const SOLEIL_MELODIE: &[(u8, u8)] = &[
    (72, 1),
    (76, 1),
    (79, 2),
    (76, 1),
    (79, 1),
    (84, 2), // do
    (83, 2),
    (79, 2),
    (74, 2),
    (79, 2), //                     sol
    (81, 1),
    (79, 1),
    (76, 2),
    (72, 1),
    (76, 1),
    (81, 2), // la mineur
    (77, 2),
    (76, 2),
    (74, 2),
    (0, 2), //                      fa
    (72, 1),
    (76, 1),
    (79, 2),
    (76, 1),
    (79, 1),
    (84, 2), // do
    (86, 2),
    (83, 2),
    (79, 2),
    (83, 2), //                     sol
    (81, 1),
    (79, 1),
    (77, 2),
    (76, 2),
    (74, 2), //            fa
    (72, 4),
    (0, 4), //                                        do
];

/// Basse « oum-pa » : fondamentale et quinte, une noire chacune.
const SOLEIL_BASSE: &[(u8, u8)] = &[
    (48, 2),
    (55, 2),
    (48, 2),
    (55, 2),
    (43, 2),
    (50, 2),
    (43, 2),
    (50, 2),
    (45, 2),
    (52, 2),
    (45, 2),
    (52, 2),
    (41, 2),
    (48, 2),
    (41, 2),
    (48, 2),
    (48, 2),
    (55, 2),
    (48, 2),
    (55, 2),
    (43, 2),
    (50, 2),
    (43, 2),
    (50, 2),
    (41, 2),
    (48, 2),
    (41, 2),
    (48, 2),
    (48, 2),
    (55, 2),
    (48, 2),
    (43, 2),
];

/// Charleston : un souffle de bruit aigu sur chaque contretemps.
const SOLEIL_CHARLESTON: &[(u8, u8)] = &[(0, 1), (120, 1)];

static SOLEIL: Partition = Partition {
    croche: 0.18,
    voix: &[
        Voix {
            onde: NASILLARD,
            volume: 0.18,
            declin: 0.9,
            tenue: 0.85,
            notes: SOLEIL_MELODIE,
            fois: 1,
        },
        Voix {
            onde: Onde::Triangle,
            volume: 0.52,
            declin: 1.2,
            tenue: 0.9,
            notes: SOLEIL_BASSE,
            fois: 1,
        },
        Voix {
            onde: Onde::Bruit,
            volume: 0.05,
            declin: 3.0,
            tenue: 0.5,
            notes: SOLEIL_CHARLESTON,
            fois: 32,
        },
    ],
};

// --- Valse des nuages : trois temps en sol majeur (8 mesures de 6 croches, ~ 10,6 s)

const VALSE_MELODIE: &[(u8, u8)] = &[
    (74, 3),
    (76, 1),
    (74, 2), // sol
    (71, 4),
    (67, 2), //          mi mineur
    (72, 3),
    (74, 1),
    (76, 2), // do
    (78, 4),
    (74, 2), //          ré
    (79, 3),
    (78, 1),
    (76, 2), // sol
    (74, 4),
    (71, 2), //          mi mineur
    (69, 2),
    (72, 2),
    (76, 2), // la mineur
    (74, 6), //                   ré
];

/// Le « oum » du premier temps, laissé résonner toute la mesure.
const VALSE_BASSE: &[(u8, u8)] = &[
    (43, 6),
    (40, 6),
    (48, 6),
    (50, 6),
    (43, 6),
    (40, 6),
    (45, 6),
    (50, 6),
];

/// Les « pa-pa » des 2e et 3e temps : deux notes de l'accord, deux voix.
const VALSE_PA_GRAVE: &[(u8, u8)] = &[
    (0, 2),
    (59, 2),
    (59, 2),
    (0, 2),
    (59, 2),
    (59, 2),
    (0, 2),
    (60, 2),
    (60, 2),
    (0, 2),
    (57, 2),
    (57, 2),
    (0, 2),
    (59, 2),
    (59, 2),
    (0, 2),
    (59, 2),
    (59, 2),
    (0, 2),
    (60, 2),
    (60, 2),
    (0, 2),
    (57, 2),
    (57, 2),
];

const VALSE_PA_AIGU: &[(u8, u8)] = &[
    (0, 2),
    (62, 2),
    (62, 2),
    (0, 2),
    (64, 2),
    (64, 2),
    (0, 2),
    (64, 2),
    (64, 2),
    (0, 2),
    (62, 2),
    (62, 2),
    (0, 2),
    (62, 2),
    (62, 2),
    (0, 2),
    (64, 2),
    (64, 2),
    (0, 2),
    (64, 2),
    (64, 2),
    (0, 2),
    (66, 2),
    (66, 2),
];

static VALSE_DES_NUAGES: Partition = Partition {
    croche: 0.22,
    voix: &[
        Voix {
            onde: FIN,
            volume: 0.18,
            declin: 0.7,
            tenue: 0.9,
            notes: VALSE_MELODIE,
            fois: 1,
        },
        Voix {
            onde: Onde::Triangle,
            volume: 0.5,
            declin: 1.5,
            tenue: 0.95,
            notes: VALSE_BASSE,
            fois: 1,
        },
        Voix {
            onde: CARRE,
            volume: 0.06,
            declin: 2.0,
            tenue: 0.7,
            notes: VALSE_PA_GRAVE,
            fois: 1,
        },
        Voix {
            onde: CARRE,
            volume: 0.06,
            declin: 2.0,
            tenue: 0.7,
            notes: VALSE_PA_AIGU,
            fois: 1,
        },
    ],
};

/// Build de QA seulement (fonctionnalité `debug_niveaux`, `./build.sh --qa`) : la
/// touche F11 passe à la musique suivante du catalogue (F1 à F10 sont déjà prises par
/// les niveaux d'essai et le plantage volontaire). Toujours faux en production : la
/// touche n'est même pas lue.
pub fn musique_suivante_qa() -> bool {
    cfg!(feature = "debug_niveaux") && is_key_pressed(KeyCode::F11)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chaque_musique_a_une_partition_non_vide_qui_boucle() {
        for m in Musique::TOUTES {
            // Catalogue : identifiant stable et retrouvable, prix 0 pour Orage seule.
            assert_eq!(Musique::depuis_id(m.id()), Some(m));
            assert!(!m.nom().is_empty() && m.nom() == m.nom().to_uppercase());
            assert_eq!(m.prix() == 0, m == Musique::Orage, "{m:?}");
            if let Some(p) = m.partition() {
                // Toutes les voix ont la durée de la boucle : elles retombent ensemble.
                assert!(p.croches() > 0, "{m:?}");
                for v in p.voix {
                    assert_eq!(
                        v.croches(),
                        p.croches(),
                        "{m:?} : voix de longueur différente"
                    );
                }
            }
            let s = m.echantillons();
            let duree = s.len() as f32 / TAUX as f32;
            assert!((8.0..14.0).contains(&duree), "{m:?} dure {duree} s");
            // Audible mais sans écrêtage.
            let pic = s.iter().fold(0.0_f32, |a, v| a.max(v.abs()));
            assert!((0.2..1.0).contains(&pic), "{m:?} : pic {pic}");
            // Boucle propre : la dernière note s'éteint avant la fin et la première
            // part de zéro (attaque de 2 ms) : la jonction ne fait pas de clic.
            let fin = s[s.len() - 10..].iter().fold(0.0_f32, |a, v| a.max(v.abs()));
            assert!(s[0].abs() < 0.02 && fin < 0.02, "{m:?} : début {}, fin {fin}", s[0]);
        }
    }

    /// Pas un test : régénère les extraits à écouter `docs/qa/77-*.wav` (10 s de
    /// chaque nouvelle musique, boucle comprise) et affiche leur volume efficace à
    /// côté de celui d'Orage. Lancé à la main :
    /// `./docker-cargo.sh test -p game --no-default-features ecrire_les_extraits -- --ignored`
    #[test]
    #[ignore]
    fn ecrire_les_extraits_wav_de_qa() {
        let efficace = |s: &[f32]| (s.iter().map(|v| v * v).sum::<f32>() / s.len() as f32).sqrt();
        for m in Musique::TOUTES {
            let boucle = m.echantillons();
            let dix_s: Vec<f32> = boucle
                .iter()
                .cycle()
                .take(10 * TAUX as usize)
                .copied()
                .collect();
            println!("{:<18} efficace {:.3}", m.id(), efficace(&boucle));
            if m != Musique::Orage {
                let chemin = format!("../docs/qa/77-musique-{}.wav", m.id());
                std::fs::write(&chemin, sons::vers_wav(&dix_s)).expect("écriture du WAV");
            }
        }
    }
}
