//! Effets visuels déclenchés par ce qui s'est passé pendant un pas de simulation :
//! éclaboussures, poussière, étincelles, vent visible, secousse de mort.

use pepin_core::entities::contacts::Contact;
use pepin_core::palette::{BLANC_CASSE, CORAIL_CLAIR, OR, OR_PALE, PLUIE, PLUIE_SOMBRE};
use pepin_core::LARGEUR_LOGIQUE;

use crate::monde::Monde;
use crate::particules::Particules;
use crate::scene::borner_mort;
use crate::secousse::Secousse;

/// Effets visuels déclenchés par ce qui s'est passé pendant un pas.
/// `(cam_x, cam_y)` : coin haut-gauche de la zone visible, pour n'animer que le vent qu'on voit
/// et garder les éclats de mort dans le cadre.
pub fn effets(
    monde: &Monde,
    particules: &mut Particules,
    secousse: &mut Secousse,
    cam_x: f32,
    cam_y: f32,
) {
    let (x, y) = (monde.joueur.boite.centre().0, monde.joueur.boite.bas());
    let sens = if monde.joueur.regarde_droite {
        1.0
    } else {
        -1.0
    };
    if monde.evenements.atterrissage {
        particules.eclabousser(x, y, 7, PLUIE, 1.6);
        // Poussière des deux côtés des pieds.
        particules.poussiere(x - 3.0, y - 1.0, 3, BLANC_CASSE, -1.0);
        particules.poussiere(x + 3.0, y - 1.0, 3, BLANC_CASSE, 1.0);
    }
    if monde.evenements.saut {
        particules.eclabousser(x, y, 4, PLUIE_SOMBRE, 1.2);
        particules.poussiere(x, y - 1.0, 3, BLANC_CASSE, -sens);
    }
    // En courant : un filet d'eau et un peu de poussière derrière les pieds.
    if monde.joueur.au_sol && monde.joueur.vitesse.x.abs() > 8.0 {
        if monde.pas.is_multiple_of(6) {
            particules.eclabousser(x, y, 1, PLUIE_SOMBRE, 0.8);
        }
        if monde.pas.is_multiple_of(5) {
            particules.poussiere(x - sens * 4.0, y - 1.0, 1, BLANC_CASSE, -sens);
        }
    }
    // Le vent des courants ascendants devient visible (brins qui montent).
    if monde.pas.is_multiple_of(3) {
        for (k, c) in monde.niveau.courants.iter().enumerate() {
            let z = c.zone;
            if z.x + z.w < cam_x || z.x > cam_x + LARGEUR_LOGIQUE as f32 {
                continue;
            }
            let decal = (monde.pas as usize * 7 + k * 13) as f32 % z.w.max(1.0);
            // Le brin naît à une hauteur tirée sur toute la zone (et reste dedans) : la colonne
            // se voit de bas en haut, pas seulement sous les pieds de Pépin.
            particules.souffle(z.x + decal, z.y, z.bas(), BLANC_CASSE);
        }
        // Le vent latéral devient visible aussi (tirets horizontaux qui traversent
        // la zone, dans son sens).
        for v in &monde.niveau.vents {
            let z = v.zone;
            if z.x + z.w < cam_x || z.x > cam_x + LARGEUR_LOGIQUE as f32 {
                continue;
            }
            particules.souffle_lateral(z.x, z.x + z.w, z.y, z.bas(), v.sens, BLANC_CASSE);
        }
    }
    for c in &monde.contacts {
        match c {
            Contact::Goutte(_) => {
                particules.eclabousser(x, y - 6.0, 10, OR, 2.0);
                particules.etincelles(x, y - 8.0, 8, OR_PALE);
            }
            Contact::Ecrase => particules.eclabousser(x, y, 10, PLUIE, 2.0),
            Contact::Rebond | Contact::Auvent => particules.eclabousser(x, y, 6, CORAIL_CLAIR, 2.0),
            Contact::Lanterne(..) => particules.etincelles(x, y - 8.0, 10, OR),
            Contact::Mort(_) => {
                // Une chute finit sous l'écran : les éclats naissent dans le cadre.
                let (mx, my) = borner_mort(monde.pos_mort, (cam_x, cam_y));
                particules.eclabousser(mx + 5.0, my + 7.0, 14, CORAIL_CLAIR, 2.4);
                secousse.declencher(2, 14);
            }
        }
    }
}
