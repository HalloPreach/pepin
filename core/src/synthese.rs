//! Synthèse sonore « 8 bits » : ondes carrée, triangle et bruit, construites
//! échantillon par échantillon, sans aucun fichier audio.
//!
//! Tout est pur (pas de périphérique audio) : on fabrique des tableaux de
//! `f32` entre -1.0 et 1.0, que `wav` encode ensuite et que le jeu joue.
//!
//! Notions Rust illustrées ici : `enum` avec données (`Onde::Carre { rapport }`),
//! constructeur « à la chaîne » (`Note::new(..).declin(2.0)`), structure d'état
//! minuscule (`Bruit`, un générateur pseudo-aléatoire déterministe).

/// Fréquence d'échantillonnage : 22 050 Hz suffit pour des bips et divise le
/// poids en mémoire par deux par rapport à 44 100 Hz.
pub const TAUX: u32 = 22_050;

/// Forme de l'onde d'une note.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Onde {
    /// Carrée ; `rapport` = part du cycle à l'état haut (0.5 = creuse et pleine,
    /// 0.25 = plus nasillarde).
    Carre { rapport: f32 },
    /// Triangle : douce, idéale pour les basses.
    Triangle,
    /// Bruit « échantillonné-bloqué » : la valeur change `freq` fois par seconde
    /// (grave = grondement, aigu = souffle).
    Bruit,
}

/// Petit générateur pseudo-aléatoire (xorshift32), déterministe : le même bruit
/// est produit à chaque lancement, ce qui rend les sons testables.
pub struct Bruit(u32);

impl Bruit {
    pub fn new() -> Self {
        Self(0x2545_F491)
    }

    /// Valeur suivante, entre -1.0 et 1.0.
    pub fn suivant(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

impl Default for Bruit {
    fn default() -> Self {
        Self::new()
    }
}

/// Une note : une onde dont la fréquence glisse de `debut` à `fin` (en Hz).
#[derive(Debug, Clone, Copy)]
pub struct Note {
    pub onde: Onde,
    pub debut: f32,
    pub fin: f32,
    pub duree: f32,
    pub volume: f32,
    /// Exposant de la décroissance : 1.0 = linéaire, 2.0 = plus « percussif ».
    pub declin: f32,
}

impl Note {
    /// Note à fréquence variable (glissando de `debut` à `fin`).
    pub fn new(onde: Onde, debut: f32, fin: f32, duree: f32, volume: f32) -> Self {
        Self {
            onde,
            debut,
            fin,
            duree,
            volume,
            declin: 1.0,
        }
    }

    /// Note à fréquence fixe.
    pub fn fixe(onde: Onde, freq: f32, duree: f32, volume: f32) -> Self {
        Self::new(onde, freq, freq, duree, volume)
    }

    /// Change l'exposant de décroissance (style « à la chaîne »).
    pub fn declin(mut self, exposant: f32) -> Self {
        self.declin = exposant;
        self
    }

    /// Fabrique les échantillons de la note.
    pub fn rendre(&self, bruit: &mut Bruit) -> Vec<f32> {
        let n = (self.duree * TAUX as f32) as usize;
        // Attaque de 2 ms : évite le « clic » d'un départ brutal.
        let attaque = (0.002 * TAUX as f32).max(1.0);
        let mut phase = 0.0_f32;
        let mut tenue = bruit.suivant();
        let mut sortie = Vec::with_capacity(n);
        for i in 0..n {
            let p = i as f32 / n as f32;
            let freq = self.debut + (self.fin - self.debut) * p;
            phase += freq / TAUX as f32;
            if phase >= 1.0 {
                phase -= phase.floor();
                tenue = bruit.suivant();
            }
            let brut = match self.onde {
                Onde::Carre { rapport } => {
                    if phase < rapport {
                        1.0
                    } else {
                        -1.0
                    }
                }
                Onde::Triangle => 4.0 * (phase - 0.5).abs() - 1.0,
                Onde::Bruit => tenue,
            };
            let enveloppe = (i as f32 / attaque).min(1.0) * (1.0 - p).powf(self.declin);
            sortie.push(brut * enveloppe * self.volume);
        }
        sortie
    }
}

/// Fréquence (Hz) d'une note MIDI (69 = la 440 Hz). `0` n'est pas une note : silence.
pub fn midi_vers_hz(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

/// Ajoute `source` à `dest` à partir de `decalage_s` secondes (mélange par somme).
/// `dest` s'agrandit au besoin.
pub fn ajouter(dest: &mut Vec<f32>, source: &[f32], decalage_s: f32) {
    let debut = (decalage_s * TAUX as f32) as usize;
    if dest.len() < debut + source.len() {
        dest.resize(debut + source.len(), 0.0);
    }
    for (i, s) in source.iter().enumerate() {
        dest[debut + i] += s;
    }
}

/// Convertit en PCM 16 bits signé, en écrêtant ce qui dépasse [-1, 1].
pub fn vers_pcm16(echantillons: &[f32]) -> Vec<i16> {
    echantillons
        .iter()
        .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CARRE: Onde = Onde::Carre { rapport: 0.5 };

    #[test]
    fn la_duree_donne_le_bon_nombre_d_echantillons() {
        let s = Note::fixe(CARRE, 440.0, 0.1, 0.5).rendre(&mut Bruit::new());
        assert_eq!(s.len(), 2205);
    }

    #[test]
    fn le_volume_borne_l_amplitude_et_le_son_s_eteint() {
        for onde in [CARRE, Onde::Triangle, Onde::Bruit] {
            let s = Note::fixe(onde, 300.0, 0.2, 0.4).rendre(&mut Bruit::new());
            let max = s.iter().fold(0.0_f32, |m, v| m.max(v.abs()));
            assert!(max <= 0.4 + 1e-6 && max > 0.1, "{onde:?} : {max}");
            assert!(s[s.len() - 1].abs() < 0.01, "fin proche du silence");
            assert_eq!(s[0], 0.0, "départ à zéro (pas de clic)");
        }
    }

    #[test]
    fn la_carree_est_plus_haute_avec_une_frequence_plus_haute() {
        // Compte des passages par zéro : plus la fréquence est haute, plus il y en a.
        let passages = |f: f32| {
            let s = Note::fixe(CARRE, f, 0.2, 0.5).rendre(&mut Bruit::new());
            s.windows(2).filter(|w| w[0] * w[1] < 0.0).count()
        };
        assert!(passages(880.0) > passages(220.0) * 3);
    }

    #[test]
    fn le_bruit_est_deterministe_et_borne() {
        let mut a = Bruit::new();
        let mut b = Bruit::new();
        for _ in 0..1000 {
            let v = a.suivant();
            assert_eq!(v, b.suivant());
            assert!((-1.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn midi_la_440_et_octave() {
        assert!((midi_vers_hz(69) - 440.0).abs() < 1e-3);
        assert!((midi_vers_hz(81) - 880.0).abs() < 1e-2);
    }

    #[test]
    fn le_melange_additionne_et_agrandit() {
        let mut d = vec![0.5; 4];
        ajouter(&mut d, &[0.25; 4], 2.0 / TAUX as f32);
        assert_eq!(d, vec![0.5, 0.5, 0.75, 0.75, 0.25, 0.25]);
    }

    #[test]
    fn le_pcm_est_ecrete() {
        assert_eq!(vers_pcm16(&[2.0, -2.0, 0.0]), vec![i16::MAX, -i16::MAX, 0]);
    }
}
