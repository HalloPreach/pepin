//! Point d'entrée macroquad : fenêtre, fond de palette, résolution logique 320x180.

use elsay_core::palette::NUIT_PROFONDE;
use elsay_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};
use macroquad::prelude::*;

/// Configuration de la fenêtre (échelle x4 de la résolution logique).
fn conf() -> Conf {
    Conf {
        window_title: "elsay-game".to_owned(),
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
        let [r, g, b] = NUIT_PROFONDE.en_f32();
        clear_background(Color::new(r, g, b, 1.0));
        next_frame().await;
    }
}
