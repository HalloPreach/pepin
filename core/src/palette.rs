//! Palette de PÉPIN : « orage → soleil ». Ardoise et bleu nuit qui
//! s'éclaircissent jusqu'à l'or du soleil, parapluie corail, pluie cyan.
//! 16 couleurs maximum, figée.

/// Couleur RGB opaque, 8 bits par canal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Couleur {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Couleur {
    /// Construit une couleur à partir de ses trois canaux.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Convertit en flottants 0.0..=1.0 (pratique pour le moteur graphique).
    pub fn en_f32(self) -> [f32; 3] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        ]
    }
}

/// Ombre la plus profonde : contours, ombres portées, noir du ciel d'orage.
pub const ARDOISE_OMBRE: Couleur = Couleur::new(0x10, 0x14, 0x1F);
/// Fond de l'orage : ciel nocturne chargé de pluie.
pub const ARDOISE_NUIT: Couleur = Couleur::new(0x1C, 0x23, 0x36);
/// Ardoise : grands nuages sombres et plateformes de pierre.
pub const ARDOISE: Couleur = Couleur::new(0x2D, 0x3A, 0x54);
/// Ardoise éclaircie : relief des nuages et du décor lointain.
pub const ARDOISE_CLAIRE: Couleur = Couleur::new(0x46, 0x58, 0x7A);
/// Bleu-gris moyen : le ciel commence à se dégager.
pub const ARDOISE_PALE: Couleur = Couleur::new(0x7F, 0x93, 0xB5);
/// Brume : voiles légers entre les nuages et l'horizon.
pub const BRUME: Couleur = Couleur::new(0xA9, 0xB8, 0xD0);
/// Nuage clair : bords éclairés par la lumière qui revient.
pub const NUAGE: Couleur = Couleur::new(0xC3, 0xD1, 0xE6);
/// Blanc cassé : texte, éclats d'écume et reflets.
pub const BLANC_CASSE: Couleur = Couleur::new(0xF3, 0xF1, 0xEA);
/// Corail vif : le parapluie de Pépin, couleur du héros.
pub const CORAIL: Couleur = Couleur::new(0xFF, 0x5A, 0x4F);
/// Corail clair : reflets sur le parapluie.
pub const CORAIL_CLAIR: Couleur = Couleur::new(0xFF, 0x9A, 0x7A);
/// Orange soleil : lueur à l'horizon quand l'orage s'éloigne.
pub const ORANGE_SOLEIL: Couleur = Couleur::new(0xFF, 0xB3, 0x47);
/// Or : soleil, pièces et points forts.
pub const OR: Couleur = Couleur::new(0xFF, 0xD8, 0x4A);
/// Or pâle : halo du soleil et lumière douce.
pub const OR_PALE: Couleur = Couleur::new(0xFF, 0xF1, 0xA8);
/// Cyan de la pluie : gouttes et flaques éclairées.
pub const PLUIE: Couleur = Couleur::new(0x62, 0xD0, 0xE8);
/// Cyan sombre : eau profonde et gouttes au second plan.
pub const PLUIE_SOMBRE: Couleur = Couleur::new(0x2F, 0x8F, 0x9D);
/// Violet d'orage : nuages électriques et éclairs lointains.
pub const VIOLET_ORAGE: Couleur = Couleur::new(0x5B, 0x4A, 0x8A);

/// Liste complète des couleurs de la palette (16 au maximum), de l'orage vers le soleil.
pub const TOUTES: [Couleur; 16] = [
    ARDOISE_OMBRE,
    ARDOISE_NUIT,
    ARDOISE,
    ARDOISE_CLAIRE,
    ARDOISE_PALE,
    BRUME,
    NUAGE,
    BLANC_CASSE,
    CORAIL,
    CORAIL_CLAIR,
    ORANGE_SOLEIL,
    OR,
    OR_PALE,
    PLUIE,
    PLUIE_SOMBRE,
    VIOLET_ORAGE,
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn au_plus_seize_couleurs_distinctes() {
        let distinctes: HashSet<Couleur> = TOUTES.iter().copied().collect();
        assert!(distinctes.len() <= 16, "{} couleurs", distinctes.len());
        // Aucun doublon : chaque constante est bien unique.
        assert_eq!(distinctes.len(), TOUTES.len());
    }

    #[test]
    fn l_ombre_est_la_couleur_la_plus_sombre() {
        let lum = |c: Couleur| c.r as u32 + c.g as u32 + c.b as u32;
        assert!(TOUTES.iter().all(|&c| lum(c) >= lum(ARDOISE_OMBRE)));
    }

    #[test]
    fn conversion_flottante_bornee() {
        for c in TOUTES {
            assert!(c.en_f32().iter().all(|v| (0.0..=1.0).contains(v)));
        }
    }
}
