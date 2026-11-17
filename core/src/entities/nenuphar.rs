//! Nénuphar qui s'enfonce (carte 9.3, lettre `n`) : une plateforme posée sur
//! l'eau. Sous Pépin, elle s'enfonce doucement : en 1 s environ, ses pieds
//! passent sous la surface et, parapluie fermé, il coule (la règle de la
//! barque). Il faut donc repartir à temps : sauter plus loin, ou sauter et
//! retomber dans l'eau en planant pour devenir une barque. Sans personne
//! dessus, elle remonte à sa place, deux fois plus vite.
//!
//! Le modèle est la plateforme friable (`friable.rs`) : un solide que le
//! contact du joueur (`contacts::resoudre_contacts`) met en route, et qui se
//! remet en place après une mort (`entities::reprendre_apres_une_mort`).
//!
//! La lettre `n` se place dans la case JUSTE AU-DESSUS de l'eau (comme une
//! goutte posée sur l'eau) : la feuille (16 x 4) dépasse la surface de
//! `HAUT_NENUPHAR` px (toute son épaisseur).

use super::Actualisable;
use crate::aabb::Aabb;
use crate::physics::{Environnement, DT};

/// Hauteur (px) dont la feuille dépasse la surface au repos.
pub const HAUT_NENUPHAR: f32 = 4.0;
/// Épaisseur (px) de la feuille (sa boîte solide).
pub const EPAISSEUR_NENUPHAR: f32 = 4.0;
/// Vitesse (px/s) à laquelle la feuille s'enfonce sous Pépin : elle met
/// `HAUT_NENUPHAR` / 4 = 1 s à amener ses pieds au ras de l'eau.
pub const VITESSE_ENFONCEMENT: f32 = 4.0;
/// Profondeur (px) maximale sous la position de repos.
pub const PROFONDEUR_NENUPHAR: f32 = 8.0;
/// Vitesse (px/s) à laquelle la feuille remonte quand personne n'est dessus.
pub const VITESSE_REMONTEE: f32 = 6.0;

#[derive(Debug, Clone)]
pub struct Nenuphar {
    /// Boîte solide de la feuille (16 x 4), à sa hauteur du moment.
    pub boite: Aabb,
    /// Haut de la feuille au repos.
    pub repos: f32,
    /// Pépin est-il posé dessus (noté par `resoudre_contacts`) ?
    porte: bool,
}

impl Nenuphar {
    /// Feuille de la case `(x, y)` (coin haut-gauche) : la surface de l'eau est
    /// le bas de la case.
    pub fn dans_la_case(x: f32, y: f32) -> Self {
        let repos = y + crate::aabb::TAILLE_TUILE - HAUT_NENUPHAR;
        Self {
            boite: Aabb::new(x, repos, 16.0, EPAISSEUR_NENUPHAR),
            repos,
            porte: false,
        }
    }

    /// Pépin est (ou n'est pas) posé dessus ce pas-ci.
    pub fn porter(&mut self, porte: bool) {
        self.porte = porte;
    }

    /// Profondeur (px) sous la position de repos : 0 au repos.
    pub fn enfoncement(&self) -> f32 {
        self.boite.y - self.repos
    }

    /// Après une mort de Pépin : la feuille revient TOUT DE SUITE à sa place.
    pub fn remettre(&mut self) {
        self.boite.y = self.repos;
        self.porte = false;
    }
}

impl Actualisable for Nenuphar {
    fn actualiser(&mut self, _env: &Environnement<'_>) {
        let y = if self.porte {
            (self.boite.y + VITESSE_ENFONCEMENT * DT).min(self.repos + PROFONDEUR_NENUPHAR)
        } else {
            (self.boite.y - VITESSE_REMONTEE * DT).max(self.repos)
        };
        self.boite.y = y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::GrilleTuiles;
    use crate::entities::contacts::{resoudre_contacts, CauseMort, Contact};
    use crate::entities::{solides_des_entites, Entite};
    use crate::physics::{Eau, JOUEUR_H};
    use crate::player::{Entrees, Joueur};

    /// La règle centrale : debout sur la feuille, Pépin tient environ une
    /// seconde, puis la feuille l'a mis à l'eau et il coule.
    #[test]
    fn la_feuille_met_pepin_a_l_eau_en_une_seconde_environ() {
        let g = GrilleTuiles::vide(10, 10);
        // L'eau : surface à y = 64 ; la feuille dans la case du dessus (48..64).
        let eaux = [Eau {
            zone: Aabb::new(0.0, 64.0, 160.0, 32.0),
            courant: 0,
        }];
        let mut ents = vec![Entite::Nenuphar(Nenuphar::dans_la_case(32.0, 48.0))];
        let mut j = Joueur::new(35.0, 64.0 - HAUT_NENUPHAR - JOUEUR_H);
        let mut mort = None;
        for pas in 0..240 {
            for e in &mut ents {
                e.actualiser(&Environnement::simple(&g));
            }
            let solides = solides_des_entites(&ents);
            let env = Environnement {
                solides: &solides,
                eaux: &eaux,
                ..Environnement::simple(&g)
            };
            j.actualiser(&Entrees::default(), &env);
            if resoudre_contacts(&mut j, &mut ents).contains(&Contact::Mort(CauseMort::Noyade)) {
                mort = Some(pas);
                break;
            }
        }
        let pas = mort.expect("Pépin finit à l'eau");
        assert!((55..=75).contains(&pas), "noyé au pas {pas}");
    }
}
