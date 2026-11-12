//! Point d'entrée macroquad : fenêtre, fond ardoise de la palette, résolution logique 320x180.

mod texte;

use macroquad::prelude::*;
use pepin_core::palette::{ARDOISE_NUIT, ARDOISE_OMBRE, BLANC_CASSE, OR_PALE};
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

/// Configuration de la fenêtre (échelle x4 de la résolution logique).
fn conf() -> Conf {
    Conf {
        window_title: "PÉPIN — un jeu de plateformes".to_owned(),
        window_width: (LARGEUR_LOGIQUE * 4) as i32,
        window_height: (HAUTEUR_LOGIQUE * 4) as i32,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    loop {
        // Caméra 2D fixe : on dessine toujours en 320x180 logiques.
        set_camera(&Camera2D::from_display_rect(Rect::new(
            0.0,
            0.0,
            LARGEUR_LOGIQUE as f32,
            HAUTEUR_LOGIQUE as f32,
        )));
        let [r, g, b] = ARDOISE_NUIT.en_f32();
        clear_background(Color::new(r, g, b, 1.0));
        // Phrase de recette de la police (accents français), avec et sans ombre.
        let titre = texte::Style::simple(OR_PALE, 2).avec_ombre(ARDOISE_OMBRE);
        let corps = texte::Style::simple(BLANC_CASSE, 1);
        texte::dessiner_centre("Où êtes-vous allé, Pépin ?", 70.0, titre);
        texte::dessiner_centre("Œuvre, façade, été.", 90.0, titre);
        texte::dessiner_centre("ÉÈÊËÀÂÇÙÛÔÎÏ « àâçéèêëîïôùûœ » 0123456789", 120.0, corps);
        next_frame().await;
    }
}
