//! Module `zones` : les cinq niveaux de PÉPIN et leurs métadonnées.
//!
//! Chaque niveau est une « zone » du monde : un thème de décor (le ciel
//! s'éclaircit d'un niveau à l'autre, de l'orage au soleil), un nom, un court
//! texte d'introduction et une idée de mécanique. Le dessin ASCII lui-même
//! vit dans `levels/` (un petit fichier par niveau).
//!
//! Notions Rust illustrées ici (à reprendre dans le README) :
//! - `const` et `&'static str` : des données connues à la compilation, qui
//!   vivent pendant tout le programme (donc pas d'ownership à gérer) ;
//! - `enum` sans données pour le thème ; tableau de taille fixe `[T; 5]` ;
//! - `Result` : charger un niveau peut échouer, l'appelant décide quoi faire.

use crate::level::{ErreurNiveau, Niveau};
use crate::levels::{gouttieres, nuages, orage, toits, vents};

/// Ambiance visuelle d'un niveau. Le rendu choisit ciel, parallaxe et pluie
/// selon cette valeur ; `luminosite` donne l'avancement de l'orage vers le soleil.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    /// Nuit d'orage sur les toits.
    Toits,
    /// Averse, ciel d'ardoise.
    Gouttieres,
    /// Orage au plus fort : éclairs, ciel violet.
    Orage,
    /// Le ciel se déchire, vent d'altitude.
    Altitude,
    /// Au-dessus des nuages : le soleil doré.
    Soleil,
}

impl Theme {
    /// Clarté du ciel de 0 (orage noir) à 4 (soleil) ; l'orage du niveau 3
    /// reste volontairement sombre, c'est le sommet de la tension.
    pub fn luminosite(self) -> u8 {
        match self {
            Theme::Toits => 0,
            Theme::Gouttieres => 1,
            Theme::Orage => 0,
            Theme::Altitude => 3,
            Theme::Soleil => 4,
        }
    }

    /// Pleut-il dans ce thème ?
    pub fn pluie(self) -> bool {
        !matches!(self, Theme::Soleil)
    }
}

/// Description d'un niveau : tout sauf le niveau chargé.
#[derive(Debug, Clone, Copy)]
pub struct InfosNiveau {
    /// Numéro de 1 à 5.
    pub numero: u8,
    /// Titre affiché.
    pub nom: &'static str,
    pub theme: Theme,
    /// Texte d'introduction (placeholder), une ou deux phrases courtes.
    pub intro: &'static str,
    /// Mécanique introduite (documentation et écran d'aide).
    pub mecanique: &'static str,
    /// Durée visée en secondes (de jeu normal, sans mort).
    pub duree_cible_s: u16,
    /// Dessin ASCII du niveau (voir la légende dans `level`).
    pub ascii: &'static str,
}

impl InfosNiveau {
    /// Lit le dessin ASCII. Ne peut échouer que si le niveau embarqué est
    /// invalide, ce que les tests de ce module interdisent.
    pub fn charger(&self) -> Result<Niveau, ErreurNiveau> {
        Niveau::depuis_ascii(self.ascii)
    }
}

/// Nombre de niveaux du jeu.
pub const NB_NIVEAUX: usize = 5;

/// Les cinq niveaux, dans l'ordre du parcours.
pub const NIVEAUX: [InfosNiveau; NB_NIVEAUX] = [
    toits::INFOS,
    gouttieres::INFOS,
    orage::INFOS,
    vents::INFOS,
    nuages::INFOS,
];

/// Nombre total de gouttes d'or du jeu.
pub const GOUTTES_TOTAL: usize = NB_NIVEAUX * crate::level::GOUTTES_PAR_NIVEAU;

/// Niveau `indice` (0 à 4) ou `None` hors bornes.
pub fn niveau(indice: usize) -> Option<&'static InfosNiveau> {
    NIVEAUX.get(indice)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faisabilite::{analyser, Options};

    #[test]
    fn cinq_niveaux_dans_l_ordre() {
        assert_eq!(NIVEAUX.len(), 5);
        for (i, n) in NIVEAUX.iter().enumerate() {
            assert_eq!(n.numero as usize, i + 1);
            assert!(!n.nom.is_empty() && !n.intro.is_empty() && !n.mecanique.is_empty());
        }
        assert_eq!(niveau(0).unwrap().nom, "Les toits");
        assert!(niveau(5).is_none());
    }

    #[test]
    fn tous_les_niveaux_se_chargent() {
        for n in &NIVEAUX {
            let niv = n
                .charger()
                .unwrap_or_else(|e| panic!("niveau {} : {e}", n.numero));
            assert_eq!(niv.hauteur(), 11, "niveau {} : 11 lignes", n.numero);
        }
    }

    #[test]
    fn niveau_1_faisable() {
        verifier(0);
    }
    #[test]
    fn niveau_2_faisable() {
        verifier(1);
    }
    #[test]
    fn niveau_3_faisable() {
        verifier(2);
    }
    #[test]
    fn niveau_4_faisable() {
        verifier(3);
    }
    #[test]
    fn niveau_5_faisable() {
        verifier(4);
    }

    #[test]
    fn niveau_1_exige_de_planer() {
        let niv = NIVEAUX[0].charger().unwrap();
        let r = analyser(
            &niv,
            niv.depart,
            Options {
                planer: false,
                ..Options::default()
            },
        );
        println!("sans planer : {:?}", r);
        assert!(!r.arrivee, "le grand vide doit exiger le parapluie");
    }

    #[test]
    fn niveau_4_exige_les_courants_ascendants() {
        let niv = NIVEAUX[3].charger().unwrap();
        assert!(!niv.courants.is_empty());
        let r = analyser(
            &niv,
            niv.depart,
            Options {
                courants: false,
                ..Options::default()
            },
        );
        assert!(!r.arrivee, "sans vent le niveau 4 doit être bloqué");
    }

    /// Vérifie un niveau : arrivée, 3 gouttes, toutes les lanternes atteignables,
    /// y compris par le robot « maladroit » (pas de précision au pixel près).
    fn verifier(i: usize) {
        let infos = &NIVEAUX[i];
        let niv = infos.charger().unwrap();
        let r = analyser(&niv, niv.depart, Options::default());
        let n = infos.numero;
        println!(
            "niveau {n} : {:.1} s en course rapide, {} états",
            r.pas_arrivee.unwrap_or(0) as f32 / 60.0,
            r.etats
        );
        assert!(
            r.arrivee,
            "niveau {n} : arrivée inatteignable (x max {} en {:?})",
            r.x_max, r.dernier_etat
        );
        assert_eq!(r.gouttes, [true; 3], "niveau {n} : gouttes {:?}", r.gouttes);
        assert!(
            r.lanternes.iter().all(|&l| l),
            "niveau {n} : lanternes {:?}",
            r.lanternes
        );
        assert_eq!(r.lanternes.len(), niv.nb_lanternes());
        assert!(
            niv.nb_lanternes() >= 3,
            "niveau {n} : au moins 3 checkpoints"
        );

        let maladroit = analyser(
            &niv,
            niv.depart,
            Options {
                precis: false,
                ..Options::default()
            },
        );
        assert!(
            maladroit.arrivee,
            "niveau {n} : trop exigeant (robot maladroit bloqué à x = {})",
            maladroit.x_max
        );
    }

    /// Chaque lanterne est aussi un point de reprise : le niveau doit se
    /// terminer DEPUIS elle (sinon un respawn pourrait coincer le joueur).
    #[test]
    fn reprise_depuis_chaque_lanterne() {
        for infos in &NIVEAUX {
            let niv = infos.charger().unwrap();
            for e in &niv.entites {
                if let crate::entities::Entite::Lanterne(l) = e {
                    let r = analyser(&niv, l.point_de_reapparition(), Options::default());
                    assert!(
                        r.arrivee,
                        "niveau {} : bloqué depuis la lanterne en {:?}",
                        infos.numero,
                        l.point_de_reapparition()
                    );
                }
            }
        }
    }

    /// Un niveau dure 40 à 60 s pour un joueur normal. Le robot « course
    /// rapide » est ~2 fois plus rapide qu'un humain découvrant le niveau.
    #[test]
    fn durees_plausibles() {
        for infos in &NIVEAUX {
            let niv = infos.charger().unwrap();
            let r = analyser(&niv, niv.depart, Options::default());
            let s = r.pas_arrivee.unwrap() as f32 / 60.0;
            println!(
                "niveau {} : course rapide {s:.1} s (cible {} s)",
                infos.numero, infos.duree_cible_s
            );
            assert!(
                (12.0..=45.0).contains(&s),
                "niveau {} : {s:.1} s",
                infos.numero
            );
            assert!((40..=60).contains(&infos.duree_cible_s));
        }
    }

    #[test]
    fn premier_defi_visible_des_le_premier_ecran() {
        // Fun immédiat : à l'écran de départ (320 px de large) on voit déjà une
        // goutte, un ennemi, un éclair, un courant ou un trampoline.
        for infos in &NIVEAUX {
            let niv = infos.charger().unwrap();
            let ecran = crate::LARGEUR_LOGIQUE as f32;
            let proche = niv.courants.iter().any(|c| c.zone.x < ecran)
                || niv.entites.iter().any(|e| match e {
                    crate::entities::Entite::Goutte(g) => g.boite.x < ecran,
                    crate::entities::Entite::Ennemi(en) => en.boite.x < ecran,
                    crate::entities::Entite::Eclair(ec) => ec.zone.x < ecran,
                    crate::entities::Entite::Champignon(c) => c.boite.x < ecran,
                    _ => false,
                });
            assert!(
                proche,
                "niveau {} : rien d'intéressant à l'écran de départ",
                infos.numero
            );
        }
    }
}
