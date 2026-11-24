//! Catalogue des bruitages de PÉPIN : la liste des effets ponctuels (`Effet`), le
//! nom du fichier de chacun et son volume de lecture.
//!
//! Le SON lui-même est un vrai fichier audio (carte 10.4e) : `assets/sons/<clé>.ogg`,
//! embarqué et chargé par le module `audio` du crate `game` (voir
//! `assets/musiques/LISEZMOI.md`). Les réglages de synthèse qui les fabriquaient
//! jusque-là ont servi une dernière fois à rendre ces fichiers, puis ont été retirés,
//! avec le synthétiseur `synthese` qui ne servait plus qu'à eux.
//!
//! Notions Rust illustrées ici : `enum` + `match` exhaustif pour un catalogue, tableau
//! constant `TOUS` pour le parcourir.

/// Les effets sonores ponctuels du jeu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Effet {
    Saut,
    /// Le parapluie s'ouvre.
    Plane,
    Atterrissage,
    /// Goutte d'or ramassée.
    Goutte,
    /// Ennemi écrasé.
    Ecrase,
    /// Champignon trampoline.
    Rebond,
    /// Lanterne allumée (point de reprise).
    Lanterne,
    Mort,
    /// Coup de tonnerre d'un éclair.
    Eclair,
    /// Drapeau d'arrivée atteint.
    Victoire,
    /// Déplacement / validation dans les menus.
    Menu,
    /// Une zone secrète se révèle (carte 8.5).
    Secret,
}

impl Effet {
    /// Tous les effets, pour tout charger d'un coup.
    pub const TOUS: [Effet; 12] = [
        Effet::Saut,
        Effet::Plane,
        Effet::Atterrissage,
        Effet::Goutte,
        Effet::Ecrase,
        Effet::Rebond,
        Effet::Lanterne,
        Effet::Mort,
        Effet::Eclair,
        Effet::Victoire,
        Effet::Menu,
        Effet::Secret,
    ];

    /// Nom du fichier du son, sans `.ogg` (`assets/sons/<clé>.ogg`, carte 10.4e).
    /// STABLE : renommer une clé, c'est renommer le fichier.
    pub fn cle(self) -> &'static str {
        match self {
            Effet::Saut => "saut",
            Effet::Plane => "plane",
            Effet::Atterrissage => "atterrissage",
            Effet::Goutte => "goutte",
            Effet::Ecrase => "ecrase",
            Effet::Rebond => "rebond",
            Effet::Lanterne => "lanterne",
            Effet::Mort => "mort",
            Effet::Eclair => "eclair",
            Effet::Victoire => "victoire",
            Effet::Menu => "menu",
            Effet::Secret => "secret",
        }
    }

    /// Volume de lecture conseillé (de 0.0 à 1.0) : les bruits forts sont baissés.
    pub fn volume(self) -> f32 {
        match self {
            Effet::Eclair => 0.55,
            Effet::Atterrissage => 0.5,
            Effet::Plane => 0.6,
            _ => 0.8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chaque_effet_a_une_cle_unique_et_un_volume() {
        for (i, e) in Effet::TOUS.iter().enumerate() {
            let cle = e.cle();
            // Nom de fichier simple : minuscules ASCII et `_`.
            assert!(
                cle.bytes().all(|b| b.is_ascii_lowercase() || b == b'_'),
                "{cle}"
            );
            assert!(Effet::TOUS[i + 1..].iter().all(|f| f.cle() != cle), "{cle}");
            assert!((0.0..=1.0).contains(&e.volume()), "{e:?}");
        }
    }
}
