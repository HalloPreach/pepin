//! Marche au sol des patrouilleurs (la flaque, l'escargot) : gravité, un pas en
//! avant, demi-tour contre un mur ou juste avant le bord du vide.
//!
//! La flaque (`ennemi`) et l'escargot (`escargot`) marchent EXACTEMENT de la même
//! façon : une seule fonction pour les deux, qui reçoit les champs à modifier.
//!
//! Notions Rust : une fonction libre qui emprunte en `&mut` plusieurs champs
//! DIFFÉRENTS d'une même structure (`&mut self.boite`, `&mut self.au_sol`...) ;
//! le compilateur l'accepte, car ces emprunts ne se recouvrent pas.

use crate::aabb::{deplacer, Aabb, TAILLE_TUILE};
use crate::physics::{Environnement, DT, GRAVITE, VITESSE_MAX_CHUTE};

/// Y a-t-il du sol juste devant les « pieds » ? On sonde un point 0,5 px
/// au-delà du bord avant, 0,5 px sous la boîte : tuile solide OU solide mobile.
pub(crate) fn sol_devant(boite: &Aabb, vers_la_droite: bool, env: &Environnement<'_>) -> bool {
    let x = if vers_la_droite {
        boite.droite() + 0.5
    } else {
        boite.x - 0.5
    };
    let y = boite.bas() + 0.5;
    let tx = (x / TAILLE_TUILE).floor() as i32;
    let ty = (y / TAILLE_TUILE).floor() as i32;
    if env.grille.est_solide(tx, ty) {
        return true;
    }
    let sonde = Aabb::new(x - 0.5, y - 0.5, 1.0, 1.0);
    env.solides.iter().any(|s| s.chevauche(&sonde))
}

/// Un pas de marche au sol à `vitesse` px/s : demi-tour avant le vide (si l'on
/// est posé), gravité, déplacement, demi-tour contre un mur.
pub(crate) fn patrouiller(
    boite: &mut Aabb,
    vers_la_droite: &mut bool,
    vitesse_y: &mut f32,
    au_sol: &mut bool,
    vitesse: f32,
    env: &Environnement<'_>,
) {
    if *au_sol && !sol_devant(boite, *vers_la_droite, env) {
        *vers_la_droite = !*vers_la_droite;
    }
    *vitesse_y = (*vitesse_y + GRAVITE * DT).min(VITESSE_MAX_CHUTE);
    let sens = if *vers_la_droite { 1.0 } else { -1.0 };
    let dx = sens * vitesse * DT;
    let contacts = deplacer(boite, dx, *vitesse_y * DT, env.grille, env.solides);
    *au_sol = contacts.sol;
    if contacts.sol || contacts.plafond {
        *vitesse_y = 0.0;
    }
    if contacts.mur() {
        *vers_la_droite = !*vers_la_droite;
    }
}
