//! Catalogue des musiques de PÉPIN : la boucle d'origine (« Orage », gratuite) et
//! cinq autres musiques (25 gouttes d'or chacune dans la future boutique). C'est
//! purement cosmétique : la musique ne change rien au jeu ni au score.
//!
//! Une musique par ZONE (carte 8.7) : `musique_du_niveau` choisit la boucle d'après
//! le THÈME du niveau (jamais d'après son numéro : un niveau inséré ou déplacé garde
//! la musique de sa zone), et `musique_de_l_ecran` ajoute les écrans hors niveau
//! (titre, aide, boutique : Orage ; fin : Soleil). Deux niveaux de la même zone
//! jouent la même musique : `Audio::choisir_musique` ne coupe donc rien entre eux.
//!
//! Chaque musique est un VRAI fichier audio (carte 10.4e) : la boucle
//! `assets/musiques/<id>.wav` (WAV mono, voir `assets/musiques/LISEZMOI.md`), servie à
//! côté du jeu et téléchargée en arrière-plan par `audio::Audio::avancer`. Les partitions
//! en notes MIDI qui les fabriquaient jusque-là ont servi une dernière fois à rendre
//! ces fichiers, puis ont été retirées : le fichier est la seule source.
//!
//! Le choix du joueur (et sa sauvegarde) sera branché par la carte de la boutique :
//! elle vendra un « forçage » qui passe PAR-DESSUS la musique de la zone (la boucle
//! principale, `main.rs`, a déjà la place pour ce forçage : la touche F11 du build
//! de QA s'en sert).
//!
//! Notions Rust illustrées ici : `enum` + tableau constant `TOUTES` pour un
//! catalogue, `Option` pour « pas de musique choisie », `match` sur un couple
//! (`musique_jouee`).

use macroquad::input::{is_key_pressed, KeyCode};
use pepin_core::zones::{InfosNiveau, Theme};

use crate::ecrans::Ecran;

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
    /// Le combat contre le Grand Nuage : mi mineur, rapide et tendu (carte 8.7).
    Cyclone,
    /// Le niveau final : le Grand Nuage remonte et poursuit Pépin. Le motif du
    /// Cyclone, furieux et plus rapide, la plus rapide du jeu (carte 8.7).
    Poursuite,
}

// Les identifiants, noms et prix ne servent pas encore dans le jeu : la carte de la
// boutique les affichera et les enregistrera. Seuls les tests les lisent pour l'instant.
#[cfg_attr(not(test), allow(dead_code))]
impl Musique {
    /// Tout le catalogue, dans l'ordre d'affichage (et de la touche de QA).
    pub const TOUTES: [Musique; 6] = [
        Musique::Orage,
        Musique::ToitsDeNuit,
        Musique::Soleil,
        Musique::ValseDesNuages,
        Musique::Cyclone,
        Musique::Poursuite,
    ];

    /// Identifiant texte STABLE (sauvegarde) : ne jamais le changer une fois publié.
    pub fn id(self) -> &'static str {
        match self {
            Musique::Orage => "orage",
            Musique::ToitsDeNuit => "toits_de_nuit",
            Musique::Soleil => "soleil",
            Musique::ValseDesNuages => "valse_des_nuages",
            Musique::Cyclone => "cyclone",
            Musique::Poursuite => "poursuite",
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
            Musique::Cyclone => "CYCLONE",
            Musique::Poursuite => "POURSUITE",
        }
    }

    /// Prix en gouttes d'or (0 = toujours débloquée).
    pub fn prix(self) -> u32 {
        match self {
            Musique::Orage => 0,
            _ => crate::cosmetiques::PRIX_MUSIQUE,
        }
    }

    /// La musique suivante du catalogue (après la dernière : la première).
    pub fn suivante(self) -> Musique {
        let i = Musique::TOUTES.iter().position(|&m| m == self).unwrap_or(0);
        Musique::TOUTES[(i + 1) % Musique::TOUTES.len()]
    }
}

/// La musique d'un niveau, d'après sa ZONE (son thème). Fonction pure.
///
/// On regarde le thème et non le numéro : insérer un niveau ou en déplacer un ne
/// change pas la musique des autres. Un thème qui n'est rangé dans aucune zone
/// (par exemple un thème tout neuf) retombe sur la boucle d'origine, Orage.
///
/// Le niveau final « Le retour du Grand Nuage » a son thème à lui, `Poursuite` (le
/// décor d'altitude, carte 8.11) : il reçoit donc sa propre musique, Poursuite.
pub fn musique_du_niveau(niveau: &InfosNiveau) -> Musique {
    match niveau.theme {
        // La ville la nuit : les toits, les gouttières, le marché couvert.
        Theme::Toits | Theme::Gouttieres | Theme::Marche => Musique::ToitsDeNuit,
        // Le boss : le Grand Nuage, dans l'œil du cyclone.
        Theme::Cyclone => Musique::Cyclone,
        // Le retour du Grand Nuage (carte 8.2) : son thème à lui (carte 8.11).
        Theme::Poursuite => Musique::Poursuite,
        // L'altitude : les vents, la mer de nuages, au-dessus des nuages (dont le
        // thème s'appelle « Soleil » : le soleil doré du ciel, pas encore la fin).
        Theme::Altitude | Theme::Mer | Theme::Soleil => Musique::ValseDesNuages,
        // Le bonus et l'épilogue : l'orage est fini, le soleil est réveillé.
        Theme::ArcEnCiel | Theme::Descente => Musique::Soleil,
        // Monde 2, « Là où va la pluie » (carte 9.5) : les sept thèmes ont chacun
        // un REPLI sur une musique existante ; leurs musiques propres (eau, port,
        // pieuvre) arriveront avec la carte 9.14.
        Theme::Caniveau => Musique::ToitsDeNuit,
        Theme::Egouts => Musique::Orage,
        Theme::Cascade => Musique::ValseDesNuages,
        Theme::Grotte => Musique::Orage,
        Theme::Riviere => Musique::ValseDesNuages,
        Theme::Port => Musique::ValseDesNuages,
        Theme::Maree => Musique::Poursuite,
        // L'orage (le grand orage, le clocher) et tout thème non rangé : repli sur la
        // boucle d'origine.
        _ => Musique::Orage,
    }
}

/// La musique d'un écran. `niveau` : le niveau chargé (celui de la carte de
/// présentation, de la partie, de la pause...). Fonction pure.
///
/// La carte de présentation joue déjà la musique du niveau qu'elle annonce : entre
/// deux niveaux de la même zone, la boucle continue sans coupure.
pub fn musique_de_l_ecran(ecran: Ecran, niveau: &InfosNiveau) -> Musique {
    match ecran {
        // La carte des niveaux (`Plan`, carte 7.4) se range avec les menus.
        Ecran::Titre | Ecran::Aide | Ecran::Boutique | Ecran::Plan | Ecran::ChoixForce => {
            Musique::Orage
        }
        // Le bilan d'un niveau joué seul (ou d'une course) garde la musique du niveau ;
        // la démo du titre (carte 8.1) a celle du niveau que joue le robot.
        Ecran::Carte(_)
        | Ecran::Jeu
        | Ecran::Pause
        | Ecran::PlusDeVies(_)
        | Ecran::BilanNiveau
        | Ecran::BilanCourse
        | Ecran::Demo => musique_du_niveau(niveau),
        // L'histoire (carte 8.4) : le soleil réveillé a sa musique, le reste garde
        // celle du titre.
        Ecran::Histoire(crate::ecrans::histoire::Chapitre::Soleil) => Musique::Soleil,
        Ecran::Histoire(_) => Musique::Orage,
        Ecran::Fin => Musique::Soleil,
    }
}

/// La musique jouée (carte 7.6) : celle de l'écran (`musique_de_l_ecran`), sauf en
/// jeu si le joueur a équipé une musique dans la boutique (`choisie`) : elle remplace
/// alors la musique de la zone, dans tous les niveaux. Les menus, la démo du titre,
/// l'histoire et la fin gardent la leur. Fonction pure.
pub fn musique_jouee(ecran: Ecran, niveau: &InfosNiveau, choisie: Option<Musique>) -> Musique {
    match (choisie, ecran) {
        (
            Some(m),
            Ecran::Carte(_) | Ecran::Jeu | Ecran::Pause | Ecran::PlusDeVies(_) | Ecran::BilanNiveau,
        ) => m,
        _ => musique_de_l_ecran(ecran, niveau),
    }
}

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
    fn chaque_musique_a_un_identifiant_stable_et_son_fichier() {
        for m in Musique::TOUTES {
            // Catalogue : identifiant stable et retrouvable, prix 0 pour Orage seule.
            assert_eq!(Musique::depuis_id(m.id()), Some(m));
            assert!(!m.nom().is_empty() && m.nom() == m.nom().to_uppercase());
            assert_eq!(m.prix() == 0, m == Musique::Orage, "{m:?}");
            // L'identifiant est aussi le nom du fichier (carte 10.4e) : la boucle
            // `assets/musiques/<id>.wav` existe (son format est contrôlé par
            // `audio::tests`, son raccord par le crate `outils`).
            assert!(
                crate::audio::FICHIERS.contains(&("musiques", m.id())),
                "{m:?}"
            );
        }
    }

    /// Une musique par zone (carte 8.7) : chaque niveau du jeu reçoit celle de sa
    /// zone (d'après son thème), le boss et l'épilogue ont la leur, et les écrans hors
    /// niveau suivent la règle (titre : Orage ; fin : Soleil ; pause : celle du niveau).
    #[test]
    fn chaque_niveau_a_la_musique_de_sa_zone() {
        use Musique::*;
        // Dans l'ordre de la table : la ville, l'orage, le boss, l'altitude, le retour
        // du Grand Nuage (carte 8.2 ; son thème `Poursuite` : carte 8.11),
        // puis le bonus et l'épilogue.
        let attendu = [
            ToitsDeNuit,    // les toits
            ToitsDeNuit,    // gouttières et cheminées
            ToitsDeNuit,    // le marché couvert
            Orage,          // le grand orage
            Orage,          // le clocher
            Cyclone,        // l'œil du cyclone (boss)
            ValseDesNuages, // vents d'altitude
            ValseDesNuages, // la mer de nuages
            ValseDesNuages, // au-dessus des nuages
            Poursuite,      // le retour du Grand Nuage
            Soleil,         // l'arc-en-ciel (bonus)
            Soleil,         // la descente (épilogue)
        ];
        let obtenu: Vec<Musique> = niveaux::niveaux_monde_1()
            .iter()
            .map(musique_du_niveau)
            .collect();
        assert_eq!(obtenu, attendu);
        // Monde 2 (carte 9.6) : le caniveau garde la musique de repli de son thème
        // (`ToitsDeNuit`) ; ses vraies musiques viendront avec la carte 9.14.
        assert_eq!(musique_du_niveau(niveaux::par_cle("caniveau")), ToitsDeNuit);
        // Le boss a une musique à lui seul.
        assert_eq!(obtenu.iter().filter(|&&m| m == Cyclone).count(), 1);
        let cyclone = &niveaux::niveaux()[5];
        assert_eq!(musique_de_l_ecran(Ecran::Titre, cyclone), Orage);
        assert_eq!(musique_de_l_ecran(Ecran::Boutique, cyclone), Orage);
        assert_eq!(musique_de_l_ecran(Ecran::Carte(10), cyclone), Cyclone);
        assert_eq!(musique_de_l_ecran(Ecran::Pause, cyclone), Cyclone);
        assert_eq!(musique_de_l_ecran(Ecran::Fin, cyclone), Soleil);
        // Carte 7.6 : une musique équipée remplace celle de la zone en jeu seulement.
        assert_eq!(musique_jouee(Ecran::Jeu, cyclone, Some(Soleil)), Soleil);
        assert_eq!(musique_jouee(Ecran::Pause, cyclone, Some(Soleil)), Soleil);
        assert_eq!(musique_jouee(Ecran::Titre, cyclone, Some(Soleil)), Orage);
        assert_eq!(musique_jouee(Ecran::Jeu, cyclone, None), Cyclone);
    }
}
