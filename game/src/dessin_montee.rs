//! Rendu de la montée de la Pieuvre (carte 9.36a, décision d'AGA : « une montée
//! contre l'eau qui avance plus vite avec des objets qui tombent du ciel envoyés
//! par la pieuvre », et à la fin « elle finit par s'enfuir ») :
//! - la PIEUVRE AU LOIN (`Pieuvre::au_loin`) : sa tête de la phase 1, à moitié
//!   dans la mer, à droite de la tour ; pendant la furie, elle s'y retire depuis
//!   sa place dans l'arène ; quand elle lance, le bout d'un tentacule se lève à
//!   côté d'elle (`Pieuvre::lance`) ;
//! - chaque OBJET qui tombe (`Pieuvre::chutes`) : son OMBRE clignote sur le sol où
//!   il va se briser pendant l'annonce, puis l'objet tombe (l'ombre reste), puis
//!   ses éclats ;
//! - LA FUITE : la tête plonge et disparaît au large, la mer redescend.
//!
//! Dessin PROVISOIRE (le vrai vient dans la carte 9.36b, après l'avis d'AGA sur
//! une planche) : que des sprites existants de la Pieuvre (`sprites::pieuvre` :
//! la tête de la phase 1, le bout de tentacule, la caisse, son ombre, ses éclats),
//! placés au pixel entier : aucune couleur ajoutée, aucun flou. Le tonneau et
//! l'épave n'ont pas encore leur image : la caisse, retournée pour le tonneau,
//! avec des éclats pour l'épave. En mode facile, la tête se dessine pâle, comme
//! en phase 1.

use pepin_core::entities::pieuvre::{EtatChute, Objet, PhasePieuvre, Pieuvre};

use crate::dessin_pieuvre::la_pieuvre;
use crate::monde::Monde;
use crate::sprites::Banque;

/// Image d'une animation à deux images de `duree` pas chacune.
fn image(anim: [&'static str; 2], pas: u32, duree: u32) -> &'static str {
    anim[(pas / duree) as usize % 2]
}

/// Tout ce que la phase 2 ajoute au dessin de la pieuvre du niveau (caméra
/// `cx`, `cy`), devant les tuiles et la mer, derrière la crête d'écume
/// (`mer_montee::dessiner_devant`) et Pépin.
pub fn dessiner(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    let Some(p) = la_pieuvre(monde).filter(|p| p.en_fuite()) else {
        return;
    };
    let camera = (cx as f32, cy as f32);
    dessiner_au_loin(p, banque, monde.pas, camera);
    dessiner_chutes(p, banque, monde.pas, camera);
}

/// La Pieuvre au loin : la tête, coupée à la surface de la mer (le reste est
/// sous l'eau). Furie et montée : en colère ; la fuite : touchée, elle plonge.
fn dessiner_au_loin(p: &Pieuvre, banque: &Banque, pas: u32, (cx, cy): (f32, f32)) {
    let (Some(tete), Some(mer)) = (p.au_loin(), p.surface_mer()) else {
        return;
    };
    let nom = match p.phase() {
        PhasePieuvre::Plongee => "pieuvre_touchee",
        PhasePieuvre::Furie => image(["pieuvre_colere0", "pieuvre_colere1"], pas, 4),
        _ => image(["pieuvre_colere0", "pieuvre_colere1"], pas, 20),
    };
    let visible = (mer - tete.y).clamp(0.0, tete.h).round();
    if visible <= 0.0 {
        return;
    }
    let (x, y) = ((tete.x - cx).round(), (tete.y - cy).round());
    if p.est_facile() && visible >= tete.h {
        banque.dessiner_pale(nom, x, y);
    } else {
        banque.dessiner_haut(nom, x, y, visible);
    }
    // Le lancer : le bout d'un tentacule se lève à sa gauche, puis retombe.
    if let Some(k) = p.lance() {
        let leve = (16.0 * (k * std::f32::consts::PI).sin()).round();
        let surface = (mer - cy).round();
        banque.dessiner_haut("tentacule_bout", x - 14.0, surface - leve, leve);
    }
}

/// Les objets lancés : l'ombre qui clignote pendant l'annonce (plus vite à la
/// fin), l'objet qui tombe au-dessus de son ombre, puis ses éclats.
fn dessiner_chutes(p: &Pieuvre, banque: &Banque, pas: u32, (cx, cy): (f32, f32)) {
    let t = p.ecoule();
    for c in p.chutes() {
        let (x, sol) = ((c.x - cx).round(), (c.sol - cy).round());
        match c.etat(t) {
            Some(EtatChute::Annonce(a)) => {
                let rythme = if a < 0.5 { 6 } else { 3 };
                if (pas / rythme).is_multiple_of(2) {
                    banque.dessiner("ombre_caisse", x, sol, false);
                }
            }
            Some(EtatChute::Chute(b)) => {
                banque.dessiner("ombre_caisse", x, sol, false);
                let y = (b.y - cy).round();
                banque.dessiner("caisse_chute", x, y, c.objet == Objet::Tonneau);
                if c.objet == Objet::Epave {
                    banque.dessiner("eclats_caisse", x, y + 5.0, false);
                }
            }
            Some(EtatChute::Bris(_)) => banque.dessiner("eclats_caisse", x, sol - 6.0, false),
            None => {}
        }
    }
}
