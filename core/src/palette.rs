//! Palette originale du jeu : crépuscule violet / corail / jaune pâle
//! sur une nuit profonde. 16 couleurs maximum, figée.

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
        [self.r as f32 / 255.0, self.g as f32 / 255.0, self.b as f32 / 255.0]
    }
}

/// Fond du ciel, le plus sombre de la palette.
pub const NUIT_PROFONDE: Couleur = Couleur::new(0x16, 0x0F, 0x2A);
/// Ciel nocturne un peu plus clair, pour les aplats lointains.
pub const NUIT_MOYENNE: Couleur = Couleur::new(0x2A, 0x1B, 0x4A);
/// Silhouettes lointaines, relief du second plan.
pub const PRUNE_SOMBRE: Couleur = Couleur::new(0x3D, 0x24, 0x5E);
/// Violet principal : décor et plateformes.
pub const VIOLET: Couleur = Couleur::new(0x6B, 0x3F, 0xA0);
/// Violet clair pour les arêtes et les reflets de plateformes.
pub const VIOLET_CLAIR: Couleur = Couleur::new(0x9A, 0x6F, 0xD1);
/// Lavande douce : brume et horizon.
pub const LAVANDE: Couleur = Couleur::new(0xC9, 0xA8, 0xE8);
/// Magenta profond : transition entre violet et corail.
pub const MAGENTA: Couleur = Couleur::new(0xA8, 0x3A, 0x7E);
/// Rose crépusculaire : lueur basse à l'horizon.
pub const ROSE_CREPUSCULE: Couleur = Couleur::new(0xE0, 0x5A, 0x8C);
/// Corail vif : héros, éléments dangereux ou à saisir.
pub const CORAIL: Couleur = Couleur::new(0xFF, 0x6B, 0x5E);
/// Corail clair : reflets sur le corail.
pub const CORAIL_CLAIR: Couleur = Couleur::new(0xFF, 0x9A, 0x82);
/// Orange brûlé : ombres chaudes et braises.
pub const ORANGE_BRULE: Couleur = Couleur::new(0xD9, 0x5B, 0x2E);
/// Jaune pâle : lumière, pièces, interface.
pub const JAUNE_PALE: Couleur = Couleur::new(0xFF, 0xE9, 0x9A);
/// Jaune doré : étincelles et points forts.
pub const JAUNE_DORE: Couleur = Couleur::new(0xF5, 0xC2, 0x4B);
/// Blanc chaud lunaire : texte et étoiles.
pub const BLANC_LUNE: Couleur = Couleur::new(0xFF, 0xF6, 0xE0);
/// Pourpre nuit : ombres portées sur le violet.
pub const POURPRE_NUIT: Couleur = Couleur::new(0x4A, 0x1F, 0x4F);
/// Brume rosée : voiles translucides de fond.
pub const BRUME_ROSEE: Couleur = Couleur::new(0x7A, 0x4A, 0x8C);

/// Liste complète des couleurs de la palette (16 au maximum).
pub const TOUTES: [Couleur; 16] = [
    NUIT_PROFONDE,
    NUIT_MOYENNE,
    PRUNE_SOMBRE,
    VIOLET,
    VIOLET_CLAIR,
    LAVANDE,
    MAGENTA,
    ROSE_CREPUSCULE,
    CORAIL,
    CORAIL_CLAIR,
    ORANGE_BRULE,
    JAUNE_PALE,
    JAUNE_DORE,
    BLANC_LUNE,
    POURPRE_NUIT,
    BRUME_ROSEE,
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
    fn la_nuit_est_la_couleur_la_plus_sombre() {
        let lum = |c: Couleur| c.r as u32 + c.g as u32 + c.b as u32;
        assert!(TOUTES.iter().all(|&c| lum(c) >= lum(NUIT_PROFONDE)));
    }

    #[test]
    fn conversion_flottante_bornee() {
        for c in TOUTES {
            assert!(c.en_f32().iter().all(|v| (0.0..=1.0).contains(v)));
        }
    }
}
