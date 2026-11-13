//! Perte de focus : onglet masqué, autre fenêtre ou application au premier plan,
//! téléphone verrouillé. Pendant une partie, le jeu ouvre alors l'écran de pause.
//!
//! Faisabilité (vérifiée dans les sources de miniquad 0.4.11 et macroquad 0.4.16) :
//! - sur le web, le chargeur JavaScript de macroquad (`mq_js_bundle.js`) écoute
//!   `visibilitychange`, `focus` et `blur`, et appelle la fonction exportée
//!   `focus(vrai/faux)` du wasm (« a le focus » = `document.hasFocus()` ET page visible) ;
//! - sur ordinateur (X11, Windows, macOS) et Android, la fenêtre envoie les mêmes signaux ;
//! - miniquad les transmet comme `window_minimized_event` / `window_restored_event`, et
//!   macroquad les range dans la file d'événements de chaque « abonné » (`input::utils`).
//!
//! On s'abonne donc à cette file : ni touche simulée ni code JavaScript en plus pour
//! détecter la perte de focus. (La page web, elle, suspend le son : voir `index.html`.)
//!
//! À savoir : un onglet masqué ne dessine plus (le navigateur suspend
//! `requestAnimationFrame`). Le jeu ne lit donc l'événement qu'au RETOUR, à la
//! première image, AVANT tout pas de simulation : la pause s'ouvre avant que le monde
//! ne bouge, et le chrono n'a rien compté pendant l'absence.
//!
//! Notions Rust illustrées ici : implémenter un trait d'une autre bibliothèque
//! (`EventHandler`) en ne gardant que les méthodes utiles, `std::mem::take`.

use macroquad::input::utils::{register_input_subscriber, repeat_all_miniquad_input};
use macroquad::miniquad::EventHandler;

/// Abonnement aux événements de fenêtre de macroquad.
pub struct Focus {
    abonne: usize,
    releve: Releve,
}

/// Reçoit les événements rejoués par macroquad ; ne garde que ceux du focus.
#[derive(Default)]
struct Releve {
    /// Changements dans l'ordre d'arrivée : `false` = perdu, `true` = retrouvé.
    changements: Vec<bool>,
}

impl EventHandler for Releve {
    // `update` et `draw` sont obligatoires dans le trait ; inutiles ici.
    fn update(&mut self) {}
    fn draw(&mut self) {}

    fn window_minimized_event(&mut self) {
        self.changements.push(false);
    }

    fn window_restored_event(&mut self) {
        self.changements.push(true);
    }
}

impl Focus {
    /// À créer une seule fois, dans la boucle principale (le contexte macroquad doit exister).
    pub fn new() -> Self {
        Self {
            abonne: register_input_subscriber(),
            releve: Releve::default(),
        }
    }

    /// Changements de focus depuis l'image précédente, dans l'ordre (`false` = perdu).
    /// À appeler à CHAQUE image : sinon la file de l'abonné grossirait sans fin.
    pub fn lire(&mut self) -> Vec<bool> {
        repeat_all_miniquad_input(&mut self.releve, self.abonne);
        std::mem::take(&mut self.releve.changements)
    }
}
