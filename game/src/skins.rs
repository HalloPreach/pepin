//! Catalogue des skins (carte 7.1) : les tenues du personnage joué.
//!
//! Un skin ne change QUE le dessin : la physique, la boîte de collision (10x14) et les
//! animations restent celles de Pépin. Chaque skin fournit donc un jeu d'images
//! complet (`sprites::JeuImages`, 18 images + l'icône des vies), aux mêmes tailles.
//!
//! Ajouter un skin = un fichier de sprites dans `sprites/` (sur le modèle de
//! `pepinette.rs`) + une variante ici, avec sa ligne dans chaque `match` et dans
//! `TOUS`. Le compilateur signale tout `match` oublié : c'est l'intérêt d'un `enum`
//! plutôt que d'une liste de chaînes.
//!
//! Le skin actif est un simple paramètre du rendu (`scene::dessiner`, `hud::dessiner`).
//! Par défaut c'est Pépin. Le joueur le choisit dans la boutique (carte 7.5) ; la
//! sauvegarde (carte 7.2) le garde sous son identifiant (`id`).

use crate::sprites::{pepin, pepinette, Image, JeuImages};

/// Un skin du catalogue.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Skin {
    /// Pépin, le personnage d'origine (parapluie corail).
    #[default]
    Pepin,
    /// Pépinette, la version fille de Pépin : nœud, cils, robe, parapluie cyan.
    Pepinette,
}

impl Skin {
    /// Tout le catalogue, dans l'ordre d'affichage.
    pub const TOUS: [Skin; 2] = [Skin::Pepin, Skin::Pepinette];

    /// Identifiant enregistré dans la sauvegarde (`skins_debloques`, `skin_choisi`).
    pub fn id(self) -> &'static str {
        match self {
            Skin::Pepin => "pepin",
            Skin::Pepinette => "pepinette",
        }
    }

    /// Le skin d'un identifiant de la sauvegarde (`None` : inconnu de cette version).
    pub fn depuis_id(id: &str) -> Option<Skin> {
        Skin::TOUS.into_iter().find(|s| s.id() == id)
    }

    /// Nom affiché (police du jeu : majuscules accentuées).
    pub fn nom(self) -> &'static str {
        match self {
            Skin::Pepin => "PÉPIN",
            Skin::Pepinette => "PÉPINETTE",
        }
    }

    /// Prix en gouttes. Les deux premiers sont gratuits : Pépinette est débloquée dès
    /// le départ.
    pub fn prix(self) -> u32 {
        match self {
            Skin::Pepin => 0,
            Skin::Pepinette => 0,
        }
    }

    /// Le jeu d'images (noms des sprites, par animation) que le rendu utilise.
    pub fn images(self) -> &'static JeuImages {
        match self {
            Skin::Pepin => &pepin::IMAGES,
            Skin::Pepinette => &pepinette::IMAGES,
        }
    }

    /// Les dessins du skin (nom, grille), chargés dans la `Banque` au démarrage.
    pub fn sprites(self) -> &'static [(&'static str, Image)] {
        match self {
            Skin::Pepin => pepin::sprites(),
            Skin::Pepinette => pepinette::sprites(),
        }
    }

    /// Skin suivant du catalogue (le dernier repart au premier). Sert à la touche de
    /// mise au point du build de QA.
    pub fn suivant(self) -> Skin {
        let i = Skin::TOUS.iter().position(|&s| s == self).unwrap_or(0);
        Skin::TOUS[(i + 1) % Skin::TOUS.len()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sprites::{hauteur, largeur, verifier};

    /// Le seul test de la carte : chaque skin du catalogue fournit TOUTES les images de
    /// Pépin, chacune présente dans ses sprites, valide (palette), et à la même taille
    /// que l'image de Pépin qu'elle remplace. Le jeu reste donc le même quel que soit le
    /// skin (mêmes ancrages au dessin, même icône des vies).
    #[test]
    fn chaque_skin_fournit_toutes_les_images_aux_tailles_de_pepin() {
        let dessin = |skin: Skin, nom: &str| {
            skin.sprites()
                .iter()
                .find(|(n, _)| *n == nom)
                .unwrap_or_else(|| panic!("{skin:?} : image {nom} absente"))
                .1
        };
        let modele = Skin::Pepin.images().noms();
        assert_eq!(
            modele.len(),
            20,
            "19 images (suspendu compris, carte 11.1) + l'icône des vies"
        );
        let mut noms_vus = std::collections::HashSet::new();
        for skin in Skin::TOUS {
            assert!(!skin.nom().is_empty());
            let noms = skin.images().noms();
            assert_eq!(noms.len(), modele.len(), "{skin:?}");
            assert_eq!(
                skin.sprites().len(),
                noms.len(),
                "{skin:?} : sprites en trop"
            );
            for (nom, nom_pepin) in noms.iter().zip(&modele) {
                // Un nom ne sert qu'à un skin (la Banque les range tous ensemble).
                assert!(noms_vus.insert(*nom), "{nom} en double");
                let (image, reference) = (dessin(skin, nom), dessin(Skin::Pepin, nom_pepin));
                verifier(nom, image).unwrap();
                assert_eq!(
                    (largeur(image), hauteur(image)),
                    (largeur(reference), hauteur(reference)),
                    "{skin:?} : {nom} n'a pas la taille de {nom_pepin}"
                );
            }
        }
        // Pépinette est débloquée dès le départ (demande d'AGA).
        assert_eq!(Skin::Pepinette.prix(), 0);
        assert_eq!(Skin::default(), Skin::Pepin);
    }
}
