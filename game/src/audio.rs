//! Audio : joue les sons fabriqués par `pepin_core::sons` via l'API audio de
//! macroquad. Aucun fichier : les WAV sont construits en mémoire au démarrage.
//!
//! Le son est DÉSACTIVÉ par défaut (et un navigateur n'autorise de toute façon
//! l'audio qu'après une première interaction : clic, touche ou toucher). Tant
//! que `regler(false)` est en vigueur, rien ne joue et les boucles sont arrêtées.
//!
//! La partie « que doit-on entendre ? » (`effets_du_pas`, `Detecteur`) est pure
//! et testée ; seule `Audio` parle à macroquad.
//!
//! Notions Rust illustrées ici : `async fn` (le chargement attend le navigateur),
//! `HashMap` pour retrouver un son par son effet, détection de « front montant »
//! par comparaison avec la valeur de l'image précédente.

use std::collections::HashMap;

use macroquad::audio::{
    load_sound_from_bytes, play_sound, play_sound_once, set_sound_volume, stop_sound,
    PlaySoundParams, Sound,
};
use pepin_core::entities::contacts::Contact;
use pepin_core::entities::Entite;
use pepin_core::player::Evenements;
use pepin_core::sons::{echantillons, musique, pluie, vers_wav, Effet};
use pepin_core::LARGEUR_LOGIQUE;

use crate::monde::Monde;

/// Volume de la boucle musicale et de la pluie douce (discrets).
const VOLUME_MUSIQUE: f32 = 0.30;
const VOLUME_PLUIE: f32 = 0.22;

/// Quels effets déclencher après un pas de simulation ? (fonction pure)
pub fn effets_du_pas(evenements: &Evenements, contacts: &[Contact]) -> Vec<Effet> {
    let mut v = Vec::new();
    if evenements.saut {
        v.push(Effet::Saut);
    }
    if evenements.debut_plane {
        v.push(Effet::Plane);
    }
    if evenements.atterrissage {
        v.push(Effet::Atterrissage);
    }
    for c in contacts {
        match c {
            Contact::Goutte(_) => v.push(Effet::Goutte),
            Contact::Lanterne(..) => v.push(Effet::Lanterne),
            // Un coup sur la tête du Grand Nuage sonne comme un ennemi écrasé.
            Contact::Ecrase | Contact::Nuage => v.push(Effet::Ecrase),
            // L'auvent fait le même « boing » que le champignon.
            Contact::Rebond | Contact::Auvent => v.push(Effet::Rebond),
            // (Le tonnerre d'un éclair est déjà déclenché par `Detecteur`.)
            Contact::Mort(_) => v.push(Effet::Mort),
        }
    }
    v
}

/// Détecte les changements d'une image à l'autre (front montant) : un éclair qui
/// vient de frapper dans la zone visible, le drapeau qui vient d'être atteint.
#[derive(Debug, Default)]
pub struct Detecteur {
    eclair_visible: bool,
    termine: bool,
}

impl Detecteur {
    /// `eclair_visible` : un éclair frappe-t-il en ce moment dans l'écran ?
    /// `termine` : le niveau est-il fini ? Renvoie les effets « de transition ».
    pub fn comparer(&mut self, eclair_visible: bool, termine: bool) -> Vec<Effet> {
        let mut v = Vec::new();
        if eclair_visible && !self.eclair_visible {
            v.push(Effet::Eclair);
        }
        if termine && !self.termine {
            v.push(Effet::Victoire);
        }
        self.eclair_visible = eclair_visible;
        self.termine = termine;
        v
    }
}

/// Un éclair frappe-t-il dans la zone visible (bord gauche `cam_x`) ? Ceux du Grand
/// Nuage comptent aussi : même tonnerre.
fn eclair_en_vue(monde: &Monde, cam_x: f32) -> bool {
    let en_vue =
        |z: &pepin_core::aabb::Aabb| z.droite() >= cam_x && z.x <= cam_x + LARGEUR_LOGIQUE as f32;
    monde.entites.iter().any(|e| match e {
        Entite::Eclair(ec) => ec.frappe() && en_vue(&ec.zone),
        Entite::Nuage(g) => g.colonnes.iter().enumerate().any(|(i, z)| {
            g.etat_colonne(i) == pepin_core::entities::nuage::EtatColonne::Frappe && en_vue(z)
        }),
        _ => false,
    })
}

/// Ce qu'il faut faire aux sons quand le réglage change (voir `EtatSon::regler`).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Consignes {
    pub demarrer_boucles: bool,
    pub arreter_boucles: bool,
    pub arreter_effets: bool,
}

/// État du son, sans macroquad (donc testable) : le son est-il activé, et la
/// musique et la pluie (les « boucles ») tournent-elles ?
#[derive(Debug, Default)]
pub struct EtatSon {
    actif: bool,
    boucles: bool,
}

impl EtatSon {
    /// `actif` : le joueur veut du son. `boucles_voulues` : faux pendant la pause.
    /// Les boucles ne tournent que si les deux sont vrais ; renvoie les changements.
    pub fn regler(&mut self, actif: bool, boucles_voulues: bool) -> Consignes {
        let boucles = actif && boucles_voulues;
        let consignes = Consignes {
            demarrer_boucles: boucles && !self.boucles,
            arreter_boucles: !boucles && self.boucles,
            arreter_effets: !actif && self.actif,
        };
        self.actif = actif;
        self.boucles = boucles;
        consignes
    }
}

/// Tous les sons chargés, prêts à jouer.
pub struct Audio {
    effets: HashMap<Effet, Sound>,
    musique: Sound,
    pluie: Sound,
    etat: EtatSon,
    detecteur: Detecteur,
}

impl Audio {
    /// Fabrique et charge tous les sons (sur le web, attend leur décodage).
    pub async fn charger() -> Self {
        let mut effets = HashMap::new();
        for e in Effet::TOUS {
            effets.insert(e, charger_son(&echantillons(e)).await);
        }
        Self {
            effets,
            musique: charger_son(&musique()).await,
            pluie: charger_son(&pluie()).await,
            etat: EtatSon::default(),
            detecteur: Detecteur::default(),
        }
    }

    /// Le son est-il activé ?
    pub fn est_actif(&self) -> bool {
        self.etat.actif
    }

    /// Active ou coupe le son, et dit si la musique et la pluie doivent tourner
    /// (`boucles` : faux pendant la pause). Appelé à chaque image ; n'agit que sur
    /// un changement. Une boucle arrêtée puis relancée repart de son début.
    pub fn regler(&mut self, actif: bool, boucles: bool) {
        let c = self.etat.regler(actif, boucles);
        if c.demarrer_boucles {
            play_sound(
                &self.musique,
                PlaySoundParams {
                    looped: true,
                    volume: VOLUME_MUSIQUE,
                },
            );
            play_sound(
                &self.pluie,
                PlaySoundParams {
                    looped: true,
                    volume: VOLUME_PLUIE,
                },
            );
        }
        if c.arreter_boucles {
            stop_sound(&self.musique);
            stop_sound(&self.pluie);
        }
        if c.arreter_effets {
            for s in self.effets.values() {
                stop_sound(s);
            }
        }
    }

    /// Joue un effet (ignoré si le son est coupé).
    pub fn jouer(&self, effet: Effet) {
        if !self.etat.actif {
            return;
        }
        if let Some(s) = self.effets.get(&effet) {
            set_sound_volume(s, effet.volume());
            play_sound_once(s);
        }
    }

    /// À appeler après chaque pas de simulation du monde.
    /// `cam_x` : bord gauche de la zone visible (pour n'entendre que les éclairs vus).
    pub fn apres_pas(&mut self, monde: &Monde, cam_x: f32) {
        let mut a_jouer = effets_du_pas(&monde.evenements, &monde.contacts);
        let vue = eclair_en_vue(monde, cam_x);
        a_jouer.extend(self.detecteur.comparer(vue, monde.termine));
        for e in a_jouer {
            self.jouer(e);
        }
    }
}

/// Encode en WAV puis confie à macroquad. Les échantillons sont fabriqués par
/// le code du cœur : un échec ici serait une erreur de programmation.
async fn charger_son(donnees: &[f32]) -> Sound {
    load_sound_from_bytes(&vers_wav(donnees))
        .await
        .expect("WAV fabriqué en mémoire valide")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pepin_core::entities::contacts::CauseMort;

    #[test]
    fn rien_ne_se_passe_rien_ne_sonne() {
        assert!(effets_du_pas(&Evenements::default(), &[]).is_empty());
    }

    #[test]
    fn les_evenements_et_contacts_donnent_les_bons_effets() {
        let ev = Evenements {
            saut: true,
            atterrissage: false,
            debut_plane: true,
        };
        let contacts = [
            Contact::Goutte(1),
            Contact::Rebond,
            Contact::Ecrase,
            Contact::Lanterne(10.0, 20.0),
            Contact::Mort(CauseMort::Vide),
        ];
        assert_eq!(
            effets_du_pas(&ev, &contacts),
            vec![
                Effet::Saut,
                Effet::Plane,
                Effet::Goutte,
                Effet::Rebond,
                Effet::Ecrase,
                Effet::Lanterne,
                Effet::Mort,
            ]
        );
    }

    #[test]
    fn le_detecteur_ne_reagit_qu_au_front_montant() {
        let mut d = Detecteur::default();
        assert!(d.comparer(false, false).is_empty());
        assert_eq!(d.comparer(true, false), vec![Effet::Eclair]);
        assert!(d.comparer(true, false).is_empty(), "éclair déjà en cours");
        assert!(d.comparer(false, false).is_empty());
        assert_eq!(d.comparer(true, true), vec![Effet::Eclair, Effet::Victoire]);
        assert!(d.comparer(true, true).is_empty(), "victoire déjà annoncée");
    }

    #[test]
    fn la_pause_coupe_les_boucles_et_la_sortie_les_relance() {
        let mut s = EtatSon::default();
        // Son coupé au départ : rien ne tourne, pause ou pas.
        assert_eq!(s.regler(false, true), Consignes::default());
        assert_eq!(s.regler(false, false), Consignes::default());
        // Le joueur active le son en pause : les boucles attendent la sortie de pause.
        assert_eq!(s.regler(true, false), Consignes::default());
        // Sortie de pause : musique et pluie démarrent.
        let c = s.regler(true, true);
        assert!(c.demarrer_boucles && !c.arreter_boucles && !c.arreter_effets);
        // Image suivante, rien ne change : aucune consigne (pas de redémarrage en boucle).
        assert_eq!(s.regler(true, true), Consignes::default());
        // Pause (manuelle ou perte de focus) : les boucles s'arrêtent, le son reste actif.
        let c = s.regler(true, false);
        assert!(c.arreter_boucles && !c.arreter_effets && !c.demarrer_boucles);
        assert!(s.actif);
        assert_eq!(s.regler(true, false), Consignes::default());
        // Reprise : elles repartent.
        assert!(s.regler(true, true).demarrer_boucles);
        // Couper le son : tout s'arrête.
        let c = s.regler(false, true);
        assert!(c.arreter_boucles && c.arreter_effets);
        // Couper le son pendant la pause : les boucles étaient déjà arrêtées.
        s.regler(true, true);
        s.regler(true, false);
        let c = s.regler(false, false);
        assert!(!c.arreter_boucles && c.arreter_effets);
    }
}
