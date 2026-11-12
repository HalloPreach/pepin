//! Les ennemis : « flaques » qui patrouillent au sol, corbeaux qui volent en ligne droite.
//!
//! Notions Rust : un `enum` pour le TYPE (`TypeEnnemi`) et un autre pour l'ÉTAT
//! (`EtatEnnemi`) ; l'état `Ecrase(u8)` PORTE une donnée (les pas d'animation restants).

use super::Actualisable;
use crate::aabb::{deplacer, Aabb, TAILLE_TUILE};
use crate::physics::{Environnement, DT, GRAVITE, VITESSE_MAX_CHUTE};

/// Nombre de pas pendant lesquels une flaque écrasée reste affichée (aplatie).
pub const ANIMATION_ECRASE_PAS: u8 = 20;

/// Espèce d'ennemi. `Copy` : un simple petit « tag », on peut le dupliquer sans souci.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeEnnemi {
    /// Flaque : glisse au sol, fait demi-tour aux murs et au bord du vide.
    Flaque,
    /// Corbeau : vole droit, fait demi-tour aux murs (ignore la gravité et le vide).
    Corbeau,
}

impl TypeEnnemi {
    /// Vitesse de déplacement horizontale (px/s).
    pub fn vitesse(self) -> f32 {
        match self {
            TypeEnnemi::Flaque => 24.0,
            TypeEnnemi::Corbeau => 40.0,
        }
    }

    /// Dimensions `(largeur, hauteur)` de la boîte de collision.
    pub fn taille(self) -> (f32, f32) {
        match self {
            TypeEnnemi::Flaque => (12.0, 8.0),
            TypeEnnemi::Corbeau => (12.0, 10.0),
        }
    }
}

/// Cycle de vie d'un ennemi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtatEnnemi {
    /// Vivant et dangereux.
    Actif,
    /// Écrasé : inoffensif, encore visible `n` pas.
    Ecrase(u8),
    /// Terminé : à ne plus afficher.
    Disparu,
}

#[derive(Debug, Clone)]
pub struct Ennemi {
    pub type_ennemi: TypeEnnemi,
    pub boite: Aabb,
    pub etat: EtatEnnemi,
    /// Regarde (et avance) vers la droite ?
    pub vers_la_droite: bool,
    vitesse_y: f32,
    au_sol: bool,
}

impl Ennemi {
    /// Crée un ennemi dont le coin haut-gauche est en `(x, y)`, qui part vers la gauche.
    pub fn new(type_ennemi: TypeEnnemi, x: f32, y: f32) -> Self {
        let (w, h) = type_ennemi.taille();
        Self {
            type_ennemi,
            boite: Aabb::new(x, y, w, h),
            etat: EtatEnnemi::Actif,
            vers_la_droite: false,
            vitesse_y: 0.0,
            au_sol: false,
        }
    }

    /// Peut-il blesser le joueur (ou être écrasé) ?
    pub fn est_dangereux(&self) -> bool {
        self.etat == EtatEnnemi::Actif
    }

    /// À afficher ? (actif ou en train d'être aplati)
    pub fn est_visible(&self) -> bool {
        self.etat != EtatEnnemi::Disparu
    }

    /// Écrase l'ennemi. Sans effet s'il l'est déjà : on ne peut pas l'écraser deux fois.
    pub fn ecraser(&mut self) {
        if self.etat == EtatEnnemi::Actif {
            self.etat = EtatEnnemi::Ecrase(ANIMATION_ECRASE_PAS);
        }
    }

    /// Demi-tour.
    pub fn retourner(&mut self) {
        self.vers_la_droite = !self.vers_la_droite;
    }

    fn sens(&self) -> f32 {
        if self.vers_la_droite {
            1.0
        } else {
            -1.0
        }
    }

    /// Y a-t-il du sol juste devant les « pieds » ? On sonde un point 0,5 px
    /// au-delà du bord avant, 0,5 px sous la boîte : tuile solide OU solide mobile.
    fn sol_devant(&self, env: &Environnement<'_>) -> bool {
        let x = if self.vers_la_droite {
            self.boite.droite() + 0.5
        } else {
            self.boite.x - 0.5
        };
        let y = self.boite.bas() + 0.5;
        let tx = (x / TAILLE_TUILE).floor() as i32;
        let ty = (y / TAILLE_TUILE).floor() as i32;
        if env.grille.est_solide(tx, ty) {
            return true;
        }
        let sonde = Aabb::new(x - 0.5, y - 0.5, 1.0, 1.0);
        env.solides.iter().any(|s| s.chevauche(&sonde))
    }

    /// Patrouille de la flaque : gravité, marche, demi-tour au mur ou avant le vide.
    fn patrouiller(&mut self, env: &Environnement<'_>) {
        if self.au_sol && !self.sol_devant(env) {
            self.retourner();
        }
        self.vitesse_y = (self.vitesse_y + GRAVITE * DT).min(VITESSE_MAX_CHUTE);
        let dx = self.sens() * self.type_ennemi.vitesse() * DT;
        let contacts = deplacer(
            &mut self.boite,
            dx,
            self.vitesse_y * DT,
            env.grille,
            env.solides,
        );
        self.au_sol = contacts.sol;
        if contacts.sol || contacts.plafond {
            self.vitesse_y = 0.0;
        }
        if contacts.mur() {
            self.retourner();
        }
    }

    /// Vol du corbeau : ligne droite horizontale, demi-tour au mur.
    fn voler(&mut self, env: &Environnement<'_>) {
        let dx = self.sens() * self.type_ennemi.vitesse() * DT;
        let contacts = deplacer(&mut self.boite, dx, 0.0, env.grille, env.solides);
        if contacts.mur() {
            self.retourner();
        }
    }
}

impl Actualisable for Ennemi {
    fn actualiser(&mut self, env: &Environnement<'_>) {
        // `EtatEnnemi` est `Copy` : on le copie dans le `match`, ce qui libère
        // `self` pour pouvoir l'emprunter en `&mut` dans les branches (sinon le
        // borrow checker refuserait : `self.etat` serait emprunté pendant l'appel).
        match self.etat {
            EtatEnnemi::Actif => match self.type_ennemi {
                TypeEnnemi::Flaque => self.patrouiller(env),
                TypeEnnemi::Corbeau => self.voler(env),
            },
            EtatEnnemi::Ecrase(restant) => {
                self.etat = if restant <= 1 {
                    EtatEnnemi::Disparu
                } else {
                    EtatEnnemi::Ecrase(restant - 1)
                };
            }
            EtatEnnemi::Disparu => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::GrilleTuiles;

    fn couloir() -> GrilleTuiles {
        GrilleTuiles::depuis_texte(&[
            "##########",
            "#........#",
            "#........#",
            "##########",
        ])
    }

    #[test]
    fn la_flaque_fait_demi_tour_aux_murs() {
        let g = couloir();
        let env = Environnement::simple(&g);
        // Sol = rangée 3 (y = 48) ; flaque posée dessus (bas = 48).
        let mut f = Ennemi::new(TypeEnnemi::Flaque, 64.0, 40.0);
        let (mut min_x, mut max_x) = (f32::MAX, f32::MIN);
        for _ in 0..1500 {
            f.actualiser(&env);
            min_x = min_x.min(f.boite.x);
            max_x = max_x.max(f.boite.droite());
        }
        assert!(min_x >= TAILLE_TUILE - 1e-3, "ne traverse pas le mur gauche");
        assert!(max_x <= 9.0 * TAILLE_TUILE + 1e-3, "ne traverse pas le mur droit");
        assert!(min_x < 17.0, "atteint bien le mur gauche : min = {min_x}");
        assert!(max_x > 143.0, "atteint bien le mur droit : max = {max_x}");
    }

    #[test]
    fn la_flaque_fait_demi_tour_avant_le_vide() {
        // Une île de 6 tuiles (x de 64 à 160) au-dessus du vide.
        let g = GrilleTuiles::depuis_texte(&[
            "..............",
            "..............",
            "....######....",
            "..............",
        ]);
        let env = Environnement::simple(&g);
        let mut f = Ennemi::new(TypeEnnemi::Flaque, 80.0, 24.0);
        let (mut min_x, mut max_x) = (f32::MAX, f32::MIN);
        for _ in 0..1500 {
            f.actualiser(&env);
            min_x = min_x.min(f.boite.x);
            max_x = max_x.max(f.boite.droite());
            assert!((f.boite.bas() - 32.0).abs() < 1e-3, "reste sur l'île");
        }
        assert!(min_x >= 64.0 - 1e-3 && max_x <= 160.0 + 1e-3);
        assert!(min_x < 66.0 && max_x > 158.0, "va jusqu'aux deux bords");
    }

    #[test]
    fn la_flaque_tombe_puis_se_pose() {
        let g = couloir();
        let env = Environnement::simple(&g);
        let mut f = Ennemi::new(TypeEnnemi::Flaque, 64.0, 16.0);
        for _ in 0..60 {
            f.actualiser(&env);
        }
        assert!((f.boite.bas() - 3.0 * TAILLE_TUILE).abs() < 1e-3);
    }

    #[test]
    fn le_corbeau_vole_sans_tomber_et_rebondit_sur_les_murs() {
        let g = couloir();
        let env = Environnement::simple(&g);
        let mut c = Ennemi::new(TypeEnnemi::Corbeau, 64.0, 24.0);
        let mut a_change_de_sens = false;
        for _ in 0..600 {
            let avant = c.vers_la_droite;
            c.actualiser(&env);
            a_change_de_sens |= avant != c.vers_la_droite;
            assert_eq!(c.boite.y, 24.0, "reste à la même altitude");
        }
        assert!(a_change_de_sens);
    }

    #[test]
    fn ecraser_aplatit_puis_fait_disparaitre() {
        let g = couloir();
        let env = Environnement::simple(&g);
        let mut f = Ennemi::new(TypeEnnemi::Flaque, 64.0, 40.0);
        assert!(f.est_dangereux());
        f.ecraser();
        assert!(!f.est_dangereux() && f.est_visible());
        assert_eq!(f.etat, EtatEnnemi::Ecrase(ANIMATION_ECRASE_PAS));

        // Écraser deux fois ne relance pas l'animation.
        f.actualiser(&env);
        f.ecraser();
        assert_eq!(f.etat, EtatEnnemi::Ecrase(ANIMATION_ECRASE_PAS - 1));

        let x = f.boite.x;
        for _ in 0..ANIMATION_ECRASE_PAS {
            f.actualiser(&env);
        }
        assert_eq!(f.etat, EtatEnnemi::Disparu);
        assert!(!f.est_visible());
        assert_eq!(f.boite.x, x, "une flaque écrasée ne bouge plus");
    }
}
