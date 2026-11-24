//! Niveau d'essai VENU DE L'EXTÉRIEUR et erreurs de niveau à l'écran (carte 10.1b),
//! build de QA seulement (`debug_niveaux`) : la porte d'entrée de l'éditeur de
//! niveaux (carte 10.3b).
//!
//! - Ouvrir la page avec `?essai=1` lance directement le niveau écrit en RON dans
//!   le `localStorage` du navigateur, sous la clé `pepin.niveau_essai` (en natif :
//!   le fichier dont le chemin est dans la variable `PEPIN_NIVEAU_ESSAI`).
//! - Un fichier invalide (ce niveau-là, ou un fichier embarqué) ne fait pas
//!   planter le jeu : l'écran affiche l'erreur lisible du crate `niveaux`
//!   (fichier, ligne, colonne) et ce qu'il faut corriger.
//!
//! Le jeu publié n'a rien de tout cela : ses fichiers embarqués sont vérifiés
//! avant le build (`./build.sh`), et il ne lit aucun niveau de l'extérieur.
//!
//! Notions Rust : `OnceLock` (une valeur posée une fois, lue ensuite partout),
//! `extern "C"` vers une fonction JavaScript, `#[cfg(target_arch = ...)]` pour
//! écrire deux versions d'une même fonction (navigateur et natif).

// Les tests natifs n'ont que le niveau externe (pas d'écran ni de lancement).
#![cfg_attr(not(feature = "debug_niveaux"), allow(dead_code))]

use macroquad::prelude::{draw_rectangle, next_frame};
use pepin_core::palette::{ARDOISE_NUIT, BLANC_CASSE, CORAIL, OR};
use pepin_core::zones::InfosNiveau;
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};
use std::sync::OnceLock;

use crate::rendu::{couleur_mq, Cible};
use crate::texte::{self, Style};

/// Nom donné au niveau externe dans les messages d'erreur.
pub const SOURCE: &str = "localStorage pepin.niveau_essai";

/// Le niveau externe, une fois lu et validé (`None` : pas demandé).
static NIVEAU: OnceLock<InfosNiveau> = OnceLock::new();

/// Ce que le jeu fait au lancement.
#[derive(Debug)]
pub enum Lancement {
    /// Comme d'habitude : l'écran titre.
    Normal,
    /// `?essai=1` et un niveau valide : on le lance tout de suite.
    Essai,
    /// Un fichier invalide : on affiche ce message au lieu de jouer.
    Erreur(String),
}

/// Vérifie les fichiers embarqués, puis lit le niveau externe s'il est demandé.
pub fn au_lancement() -> Lancement {
    if let Err(e) = niveaux::table_verifiee() {
        return Lancement::Erreur(e.to_string());
    }
    match texte_demande() {
        None => Lancement::Normal,
        Some(None) => Lancement::Erreur(format!(
            "?essai=1 : aucun niveau dans le navigateur ({SOURCE} est vide)"
        )),
        Some(Some(texte)) => match lire(&texte) {
            Ok(()) => Lancement::Essai,
            Err(message) => Lancement::Erreur(message),
        },
    }
}

/// Lit et valide un niveau RON (même chargeur que les fichiers embarqués) et le
/// garde pour `niveau()`. `Err` : le message lisible.
pub fn lire(texte: &str) -> Result<(), String> {
    let infos = niveaux::charger_fichier(SOURCE, texte).map_err(|e| e.to_string())?;
    // Déjà posé (deuxième appel) : on garde le premier, comme `OnceLock` l'impose.
    let _ = NIVEAU.set(infos);
    Ok(())
}

/// Le niveau externe validé, s'il y en a un (`monde::INDICE_ESSAI_EXTERNE`).
pub fn niveau() -> Option<&'static InfosNiveau> {
    NIVEAU.get()
}

/// Boucle sans fin de l'écran d'erreur : fond uni, titre, message coupé en lignes.
pub async fn afficher_erreur(cible: &Cible, message: &str) -> ! {
    let lignes = texte::couper_lignes(message, 50);
    loop {
        cible.commencer();
        draw_rectangle(
            0.0,
            0.0,
            LARGEUR_LOGIQUE as f32,
            HAUTEUR_LOGIQUE as f32,
            couleur_mq(ARDOISE_NUIT),
        );
        texte::dessiner_centre("NIVEAU INVALIDE", 14.0, Style::simple(CORAIL, 2));
        for (i, ligne) in lignes.iter().take(14).enumerate() {
            texte::dessiner(
                ligne,
                10.0,
                40.0 + 9.0 * i as f32,
                Style::simple(BLANC_CASSE, 1),
            );
        }
        texte::dessiner_centre(
            "corrige le fichier puis recharge la page",
            168.0,
            Style::simple(OR, 1),
        );
        cible.afficher();
        next_frame().await;
    }
}

/// `None` : pas demandé ; `Some(None)` : demandé mais rien à lire ;
/// `Some(Some(texte))` : le texte RON à charger.
#[cfg(target_arch = "wasm32")]
fn texte_demande() -> Option<Option<String>> {
    navigateur::texte_demande()
}

/// Natif : la variable d'environnement `PEPIN_NIVEAU_ESSAI` donne le chemin du
/// fichier (pratique pour essayer un niveau sans navigateur).
#[cfg(not(target_arch = "wasm32"))]
fn texte_demande() -> Option<Option<String>> {
    let chemin = std::env::var("PEPIN_NIVEAU_ESSAI").ok()?;
    Some(std::fs::read_to_string(chemin).ok())
}

/// Navigateur : le plugin JavaScript de `web/index.html` (avec la sauvegarde).
#[cfg(target_arch = "wasm32")]
mod navigateur {
    extern "C" {
        /// 1 si l'adresse de la page contient `?essai=1`, 0 sinon.
        fn pepin_niveau_essai_demande() -> u32;
        /// Copie le niveau (UTF-8) en `ptr` s'il tient dans `max` octets ; renvoie
        /// sa longueur en octets (0 : rien, ou `localStorage` refusé).
        fn pepin_niveau_essai_lire(ptr: *mut u8, max: u32) -> u32;
    }

    pub fn texte_demande() -> Option<Option<String>> {
        // SAFETY : fonction JavaScript sans argument.
        if unsafe { pepin_niveau_essai_demande() } == 0 {
            return None;
        }
        let mut tampon = vec![0u8; 64 * 1024];
        // SAFETY : le plugin écrit au plus `max` octets dans le tampon fourni.
        let mut n = unsafe { pepin_niveau_essai_lire(tampon.as_mut_ptr(), tampon.len() as u32) };
        if n as usize > tampon.len() {
            // Plus long que prévu : un second essai avec la bonne taille.
            tampon.resize(n as usize, 0);
            // SAFETY : idem.
            n = unsafe { pepin_niveau_essai_lire(tampon.as_mut_ptr(), tampon.len() as u32) };
        }
        let n = n as usize;
        if n == 0 || n > tampon.len() {
            return Some(None);
        }
        tampon.truncate(n);
        Some(String::from_utf8(tampon).ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un niveau externe valide est gardé et devient le niveau de
    /// `monde::INDICE_ESSAI_EXTERNE` ; une faute donne un message avec sa ligne.
    #[test]
    fn un_niveau_externe_se_charge_et_une_faute_s_affiche() {
        let bon = niveaux::ecrire(niveaux::par_cle("essai_vent"));
        let faux = bon.replace("theme: Soleil", "theme: Lune");
        assert_ne!(faux, bon);
        let message = lire(&faux).unwrap_err();
        assert!(
            message.contains(SOURCE) && message.contains("ligne"),
            "{message}"
        );
        lire(&bon).unwrap();
        let infos = crate::monde::infos_niveau(crate::monde::INDICE_ESSAI_EXTERNE).unwrap();
        assert_eq!(infos.cle, "essai_vent");
    }
}
