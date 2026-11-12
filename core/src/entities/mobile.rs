//! Plateforme mobile : fait des allers-retours entre deux points et emporte le joueur.

use super::Actualisable;
use crate::aabb::Aabb;
use crate::physics::{Environnement, Vec2, DT};

#[derive(Debug, Clone)]
pub struct PlateformeMobile {
    /// Boîte actuelle (solide).
    pub boite: Aabb,
    /// Les deux extrémités du trajet : position du coin haut-gauche de la boîte.
    a: (f32, f32),
    b: (f32, f32),
    /// Vitesse en px/s.
    vitesse: f32,
    /// Va-t-elle vers `b` (sinon vers `a`) ?
    vers_b: bool,
    /// Déplacement effectué pendant le DERNIER pas (sert à porter le joueur).
    pub deplacement: Vec2,
}

impl PlateformeMobile {
    /// `boite` donne la taille ; elle est placée en `a` immédiatement.
    pub fn new(boite: Aabb, a: (f32, f32), b: (f32, f32), vitesse: f32) -> Self {
        Self {
            boite: Aabb::new(a.0, a.1, boite.w, boite.h),
            a,
            b,
            vitesse,
            vers_b: true,
            deplacement: Vec2::default(),
        }
    }

    /// Les deux extrémités du trajet (positions du coin haut-gauche), pour
    /// l'analyse de faisabilité des niveaux.
    pub fn extremites(&self) -> ((f32, f32), (f32, f32)) {
        (self.a, self.b)
    }

    fn cible(&self) -> (f32, f32) {
        if self.vers_b {
            self.b
        } else {
            self.a
        }
    }
}

impl Actualisable for PlateformeMobile {
    fn actualiser(&mut self, _env: &Environnement<'_>) {
        let (cx, cy) = self.cible();
        let (dx, dy) = (cx - self.boite.x, cy - self.boite.y);
        let distance = (dx * dx + dy * dy).sqrt();
        let pas = self.vitesse * DT;
        let (mx, my) = if distance <= pas {
            // Arrivée : on se pose pile sur la cible et on repart dans l'autre sens.
            self.vers_b = !self.vers_b;
            (dx, dy)
        } else {
            (dx / distance * pas, dy / distance * pas)
        };
        self.boite.x += mx;
        self.boite.y += my;
        self.deplacement = Vec2::new(mx, my);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::GrilleTuiles;

    #[test]
    fn va_et_vient_entre_les_deux_points() {
        let g = GrilleTuiles::vide(20, 5);
        let env = Environnement::simple(&g);
        let mut p = PlateformeMobile::new(
            Aabb::new(0.0, 0.0, 32.0, 8.0),
            (16.0, 40.0),
            (96.0, 40.0),
            60.0,
        );
        assert_eq!((p.boite.x, p.boite.y), (16.0, 40.0));
        let (mut min_x, mut max_x) = (f32::MAX, f32::MIN);
        for _ in 0..600 {
            p.actualiser(&env);
            min_x = min_x.min(p.boite.x);
            max_x = max_x.max(p.boite.x);
            assert_eq!(p.boite.y, 40.0);
        }
        assert!((min_x - 16.0).abs() < 1e-3, "min = {min_x}");
        assert!((max_x - 96.0).abs() < 1e-3, "max = {max_x}");
    }

    #[test]
    fn le_deplacement_memorise_correspond_a_la_vitesse() {
        let g = GrilleTuiles::vide(20, 5);
        let env = Environnement::simple(&g);
        let mut p = PlateformeMobile::new(
            Aabb::new(0.0, 0.0, 32.0, 8.0),
            (0.0, 80.0),
            (0.0, 0.0),
            30.0,
        );
        p.actualiser(&env);
        assert_eq!(p.deplacement.x, 0.0);
        assert!((p.deplacement.y - (-30.0 * DT)).abs() < 1e-4, "monte");
    }
}
