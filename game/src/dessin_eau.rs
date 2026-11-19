//! Dessin des ennemis d'eau du Monde 2 (carte 9.4) : grenouille, poisson (et ses
//! remous), crabe, rat et mouette. Chaque annonce se voit (règle du jeu) : la
//! grenouille se ramasse et le rat se dresse, avec le « ! » du chat au-dessus de
//! la tête ; le poisson caché fait des remous à la surface ; le crabe lève les
//! deux pinces quand Pépin est au-dessus de lui. La mouette reprend les images du
//! pigeon (`scene::image_pigeon`), redessinées en blanc et gris.
//!
//! Les fonctions `image_*` sont pures (testables sans fenêtre) ; `dessiner` pose
//! les sprites, en coordonnées d'écran (`sx`, `sy` : coin haut-gauche de la boîte).

use pepin_core::entities::ennemi::{Ennemi, EtatEnnemi, PhaseEau, REMOUS_PAS, SAUT_POISSON_PAS};
use pepin_core::entities::TypeEnnemi;

use crate::ecrans::image;
use crate::scene::image_pigeon;
use crate::sprites::Banque;

/// Image de la grenouille (16x12, tournée vers la gauche).
pub(crate) fn image_grenouille(phase: PhaseEau, ecrase: bool, pas: u32) -> &'static str {
    if ecrase {
        return "grenouille_ecrasee";
    }
    match phase {
        PhaseEau::Annonce(_) => "grenouille_annonce",
        PhaseEau::Attaque(_) => "grenouille_saut",
        PhaseEau::Guet | PhaseEau::Repos(_) => image(&["grenouille0", "grenouille1"], pas, 20),
    }
}

/// Image du poisson (12x16) hors de l'eau, `None` quand il est caché (le dessin
/// de la phase cachée, ce sont les remous, `image_remous`).
pub(crate) fn image_poisson(phase: PhaseEau, pas: u32) -> Option<&'static str> {
    match phase {
        PhaseEau::Attaque(n) if n <= SAUT_POISSON_PAS / 2 => {
            Some(image(&["poisson0", "poisson1"], pas, 4))
        }
        PhaseEau::Attaque(_) => Some("poisson_chute"),
        _ => None,
    }
}

/// Remous (16x3) au-dessus du poisson caché pendant l'annonce : des petits cercles,
/// puis de grands à la fin de l'annonce. `None` hors de l'annonce.
pub(crate) fn image_remous(phase: PhaseEau) -> Option<&'static str> {
    match phase {
        PhaseEau::Annonce(n) if n > REMOUS_PAS / 2 => Some("remous0"),
        PhaseEau::Annonce(_) => Some("remous1"),
        _ => None,
    }
}

/// Image du crabe (18x12, tourné vers la gauche) : une pince toujours levée (la
/// carapace dure), les deux en alerte, qui claquent après un rebond.
pub(crate) fn image_crabe(phase: PhaseEau, pas: u32) -> &'static str {
    match phase {
        PhaseEau::Attaque(_) => "crabe_claque",
        PhaseEau::Annonce(_) => "crabe_alerte",
        PhaseEau::Guet | PhaseEau::Repos(_) => image(&["crabe0", "crabe1"], pas, 10),
    }
}

/// Image du rat (18x14, tourné vers la droite).
pub(crate) fn image_rat(phase: PhaseEau, ecrase: bool, pas: u32) -> &'static str {
    if ecrase {
        return "rat_ecrase";
    }
    match phase {
        PhaseEau::Guet => image(&["rat0", "rat1"], pas, 10),
        PhaseEau::Annonce(_) => "rat_annonce",
        PhaseEau::Attaque(_) => image(&["rat_fonce0", "rat_fonce1"], pas, 4),
        PhaseEau::Repos(_) => "rat_etourdi",
    }
}

/// Le nom d'image de la mouette qui correspond à une image du pigeon.
pub(crate) fn image_mouette(pigeon: &str) -> &'static str {
    match pigeon {
        "pigeon1" => "mouette1",
        "pigeon_annonce" => "mouette_annonce",
        "pigeon_plongee0" => "mouette_plongee0",
        "pigeon_plongee1" => "mouette_plongee1",
        "pigeon_remontee0" => "mouette_remontee0",
        "pigeon_remontee1" => "mouette_remontee1",
        "pigeon_ecrase" => "mouette_ecrasee",
        _ => "mouette0",
    }
}

/// Dessine l'ennemi d'eau `e` dont la boîte est en `(sx, sy)` à l'écran.
pub(crate) fn dessiner(e: &Ennemi, banque: &Banque, pas: u32, sx: f32, sy: f32) {
    let ecrase = matches!(e.etat, EtatEnnemi::Ecrase(_));
    let phase = e.phase_eau();
    match e.type_ennemi {
        TypeEnnemi::Grenouille => {
            // Sprite 16x12 posé sur le bas de la boîte 12x10, centré.
            let (x, y) = (sx - 2.0, sy - 2.0);
            banque.dessiner(image_grenouille(phase, ecrase, pas), x, y, e.vers_la_droite);
            if matches!(phase, PhaseEau::Annonce(_)) && !ecrase {
                // Le « ! » au-dessus de l'oeil (à gauche du dessin, à droite en miroir).
                let bx = if e.vers_la_droite { x + 9.0 } else { x + 3.0 };
                banque.dessiner("chat_alerte", bx, y - 7.0, false);
            }
        }
        TypeEnnemi::Poisson => {
            // Sprite 12x16 centré sur la boîte 10x12. La surface est 2 px au-dessus
            // de la boîte du poisson caché (il ne bouge pas avant de sauter).
            if let Some(nom) = image_poisson(phase, pas) {
                banque.dessiner(nom, sx - 1.0, sy - 2.0, false);
            } else if let Some(nom) = image_remous(phase) {
                banque.dessiner(nom, sx - 3.0, sy - 4.0, false);
            }
        }
        TypeEnnemi::Crabe => {
            // Sprite 18x12 posé sur le bas de la boîte 14x9, centré.
            banque.dessiner(
                image_crabe(phase, pas),
                sx - 2.0,
                sy - 3.0,
                e.vers_la_droite,
            );
        }
        TypeEnnemi::Rat => {
            // Sprite 18x14 posé sur le bas de la boîte 14x8, centré ; il regarde à droite.
            let (x, y) = (sx - 2.0, sy - 6.0);
            banque.dessiner(image_rat(phase, ecrase, pas), x, y, !e.vers_la_droite);
            if matches!(phase, PhaseEau::Annonce(_)) && !ecrase {
                let bx = if e.vers_la_droite { x + 13.0 } else { x + 1.0 };
                banque.dessiner("chat_alerte", bx, y - 8.0, false);
            }
        }
        _ => {
            // La mouette : les images du pigeon (sprite 18x14 sur la boîte 12x10).
            let (nom, tremble, retourne) = image_pigeon(e.phase(), ecrase, pas);
            banque.dessiner(
                image_mouette(nom),
                sx - 3.0 + tremble,
                sy - 4.0,
                e.vers_la_droite == retourne,
            );
        }
    }
}
