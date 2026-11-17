//! Chute d'eau (carte 9.3, lettre `|`) : une colonne d'eau qui tombe et pousse
//! Pépin vers le BAS, pour la grande cascade (un niveau vertical : on descend
//! entre les chutes). Parapluie OUVERT, il y descend lentement, en sécurité ;
//! parapluie FERMÉ, il est emporté vite. Ce n'est pas mortel en soi : la chute
//! peut l'être (le vide, les pointes, l'eau parapluie fermé, en bas).
//!
//! La règle est celle du courant ascendant retournée : un `CourantAscendant` à
//! l'accélération NÉGATIVE (`-ACCEL_CHUTE`). Rien de nouveau dans le joueur
//! (`Joueur::actualiser`, étape 6) : en planant, le parapluie freine à
//! `VITESSE_PLANER_CHUTE` et la chute ajoute `ACCEL_CHUTE * DT` à chaque pas, soit
//! un équilibre à 36 + 20 = 56 px/s ; parapluie fermé, Pépin file à la vitesse
//! maximale de chute plus la poussée (360 + 20 = 380 px/s).
//!
//! La colonne : les cases `|` de la grille, assemblées en rectangles (comme les
//! courants `v`). L'eau de la chute n'est PAS une étendue d'eau : on la traverse
//! sans barque ni noyade.

use super::Actualisable;
use crate::aabb::Aabb;
use crate::physics::{CourantAscendant, Environnement};

/// Poussée (px/s²) d'une chute d'eau vers le bas.
pub const ACCEL_CHUTE: f32 = 1200.0;

#[derive(Debug, Clone)]
pub struct Chute {
    /// Colonne d'eau qui tombe.
    pub zone: Aabb,
}

impl Chute {
    pub fn new(zone: Aabb) -> Self {
        Self { zone }
    }

    /// Le « courant » de la chute : vers le BAS (accélération négative).
    pub fn courant(&self) -> CourantAscendant {
        CourantAscendant {
            zone: self.zone,
            acceleration: -ACCEL_CHUTE,
        }
    }
}

/// La colonne ne bouge pas : son eau qui coule n'est qu'un dessin.
impl Actualisable for Chute {
    fn actualiser(&mut self, _env: &Environnement<'_>) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::GrilleTuiles;
    use crate::physics::{DT, VITESSE_MAX_CHUTE, VITESSE_PLANER_CHUTE};
    use crate::player::{Direction, Entrees, Joueur};

    /// La règle centrale : dans la chute, parapluie ouvert on descend lentement
    /// (un peu plus vite qu'un plané libre), parapluie fermé on est emporté plus
    /// vite que la chute libre.
    #[test]
    fn la_chute_emporte_vite_parapluie_ferme_et_lentement_parapluie_ouvert() {
        let g = GrilleTuiles::vide(10, 200);
        let chute = Chute::new(Aabb::new(64.0, 0.0, 16.0, 3200.0));
        let courants = [chute.courant()];
        let env = Environnement {
            courants: &courants,
            ..Environnement::simple(&g)
        };
        let vitesse_finale = |saut: bool| {
            let mut j = Joueur::new(67.0, 16.0);
            let e = Entrees {
                direction: Direction::Immobile,
                saut,
            };
            for _ in 0..120 {
                j.actualiser(&e, &env);
            }
            let y = j.boite.y;
            j.actualiser(&e, &env);
            (j.boite.y - y) / DT
        };
        let ouvert = vitesse_finale(true);
        let ferme = vitesse_finale(false);
        assert!(ouvert > VITESSE_PLANER_CHUTE && ouvert < 70.0, "ouvert : {ouvert}");
        assert!(ferme > VITESSE_MAX_CHUTE, "fermé : {ferme}");
    }
}
