//! Les ennemis : « flaques » qui patrouillent au sol, corbeaux qui volent en ligne
//! droite et font demi-tour au bout de leur portée (ou contre un mur).
//!
//! Notions Rust : un `enum` pour le TYPE (`TypeEnnemi`) et un autre pour l'ÉTAT
//! (`EtatEnnemi`) ; l'état `Ecrase(u8)` PORTE une donnée (les pas d'animation restants) ;
//! `Option<f32>` pour une portée qui peut ne pas exister (`None` : sans limite).

use super::Actualisable;
use crate::aabb::{deplacer, Aabb, TAILLE_TUILE};
use crate::physics::{Environnement, DT, GRAVITE, VITESSE_MAX_CHUTE};

/// Nombre de pas pendant lesquels une flaque écrasée reste affichée (aplatie).
pub const ANIMATION_ECRASE_PAS: u8 = 20;

/// Portée de patrouille du corbeau (px) : distance maximale entre sa position et
/// son point d'apparition, de chaque côté. Il fait demi-tour en l'atteignant, ou
/// plus tôt s'il rencontre un mur : le premier des deux atteint.
///
/// 5 tuiles de chaque côté = une patrouille de 10 tuiles (160 px, la moitié de
/// l'écran logique de 320 px), soit 4 s pour un aller à 40 px/s : on le voit
/// revenir, on peut donc l'attendre. Sans portée, dans un niveau ouvert (niveau 4,
/// sans murs en altitude), il traverserait toute la carte.
pub const PORTEE_CORBEAU: f32 = 5.0 * TAILLE_TUILE;

/// Espèce d'ennemi. `Copy` : un simple petit « tag », on peut le dupliquer sans souci.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeEnnemi {
    /// Flaque : glisse au sol, fait demi-tour aux murs et au bord du vide.
    Flaque,
    /// Corbeau : vole droit (ignore la gravité et le vide), fait demi-tour au bout
    /// de sa portée (`PORTEE_CORBEAU`) ou contre un mur.
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

    /// Portée de patrouille par défaut (px de chaque côté du point d'apparition).
    ///
    /// `Option<f32>` : `None` veut dire « pas de limite ». La flaque n'en a pas
    /// besoin : le bord du vide et les murs la retiennent déjà sur son toit.
    pub fn portee(self) -> Option<f32> {
        match self {
            TypeEnnemi::Flaque => None,
            TypeEnnemi::Corbeau => Some(PORTEE_CORBEAU),
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
    /// Abscisse (px) du coin haut-gauche à l'apparition : le centre de la patrouille.
    origine_x: f32,
    /// Distance maximale (px) à `origine_x`, de chaque côté (`None` : sans limite).
    portee: Option<f32>,
}

impl Ennemi {
    /// Crée un ennemi dont le coin haut-gauche est en `(x, y)`, qui part vers la gauche.
    /// Sa patrouille est centrée sur ce point, avec la portée par défaut de son type.
    pub fn new(type_ennemi: TypeEnnemi, x: f32, y: f32) -> Self {
        let (w, h) = type_ennemi.taille();
        Self {
            type_ennemi,
            boite: Aabb::new(x, y, w, h),
            etat: EtatEnnemi::Actif,
            vers_la_droite: false,
            vitesse_y: 0.0,
            au_sol: false,
            origine_x: x,
            portee: type_ennemi.portee(),
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

    /// Distance (px) qui reste avant la limite de portée, dans le sens de la marche.
    /// `None` si l'ennemi n'a pas de portée.
    fn reste_avant_limite(&self) -> Option<f32> {
        // `map` transforme l'intérieur d'une `Option` : `None` reste `None`.
        self.portee.map(|portee| {
            if self.vers_la_droite {
                self.origine_x + portee - self.boite.x
            } else {
                self.boite.x - (self.origine_x - portee)
            }
        })
    }

    /// Vol du corbeau : ligne droite horizontale, demi-tour au mur OU à la limite
    /// de sa portée, le premier des deux atteint.
    fn voler(&mut self, env: &Environnement<'_>) {
        let mut dx = self.sens() * self.type_ennemi.vitesse() * DT;
        // Si le pas dépasse la limite, on le raccourcit pour s'arrêter PILE dessus :
        // la patrouille reste exactement symétrique, sans dérive au fil des allers.
        let mut limite_atteinte = false;
        if let Some(reste) = self.reste_avant_limite() {
            if dx.abs() >= reste {
                dx = self.sens() * reste.max(0.0);
                limite_atteinte = true;
            }
        }
        let contacts = deplacer(&mut self.boite, dx, 0.0, env.grille, env.solides);
        if contacts.mur() || limite_atteinte {
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
        GrilleTuiles::depuis_texte(&["##########", "#........#", "#........#", "##########"])
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
        assert!(
            min_x >= TAILLE_TUILE - 1e-3,
            "ne traverse pas le mur gauche"
        );
        assert!(
            max_x <= 9.0 * TAILLE_TUILE + 1e-3,
            "ne traverse pas le mur droit"
        );
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

    /// Fait voler (ou marcher) `e` pendant `pas` pas ; renvoie `(x min, x max,
    /// nombre de demi-tours)` de son coin haut-gauche.
    fn patrouille(e: &mut Ennemi, env: &Environnement<'_>, pas: usize) -> (f32, f32, usize) {
        let (mut min_x, mut max_x, mut demi_tours) = (f32::MAX, f32::MIN, 0);
        for _ in 0..pas {
            let avant = e.vers_la_droite;
            e.actualiser(env);
            min_x = min_x.min(e.boite.x);
            max_x = max_x.max(e.boite.x);
            if avant != e.vers_la_droite {
                demi_tours += 1;
            }
        }
        (min_x, max_x, demi_tours)
    }

    #[test]
    fn en_monde_ouvert_le_corbeau_oscille_autour_de_son_point() {
        // 40 tuiles de large, aucun mur à moins de 5 tuiles du corbeau.
        let g = GrilleTuiles::vide(40, 10);
        let env = Environnement::simple(&g);
        let origine = 300.0;
        let mut c = Ennemi::new(TypeEnnemi::Corbeau, origine, 50.0);
        let (min_x, max_x, demi_tours) = patrouille(&mut c, &env, 1500);
        // Exactement la portée de chaque côté, jamais au-delà.
        assert!(
            (min_x - (origine - PORTEE_CORBEAU)).abs() < 1e-3,
            "min = {min_x}"
        );
        assert!(
            (max_x - (origine + PORTEE_CORBEAU)).abs() < 1e-3,
            "max = {max_x}"
        );
        // 1500 pas à 40 px/s = 1000 px parcourus : plusieurs allers-retours.
        assert!(demi_tours >= 5, "demi-tours : {demi_tours}");
        assert_eq!(c.boite.y, 50.0, "reste à la même altitude");

        // Il repasse régulièrement par son point d'apparition : la patrouille
        // est centrée dessus, elle ne dérive pas au fil des allers-retours.
        let mut c = Ennemi::new(TypeEnnemi::Corbeau, origine, 50.0);
        let mut passages = 0;
        let mut cote = c.boite.x < origine;
        for _ in 0..1500 {
            c.actualiser(&env);
            let nouveau_cote = c.boite.x < origine;
            if nouveau_cote != cote {
                passages += 1;
                cote = nouveau_cote;
            }
        }
        // 1000 px parcourus, un passage tous les 160 px environ.
        assert!(passages >= 5, "passages par l'origine : {passages}");
    }

    #[test]
    fn le_mur_fait_faire_demi_tour_au_corbeau_avant_sa_portee() {
        // Mur à gauche (colonne 0, x < 16), rien à droite sur 19 tuiles.
        let g = GrilleTuiles::depuis_texte(&[
            "#...................",
            "#...................",
            "#...................",
            "#...................",
        ]);
        let env = Environnement::simple(&g);
        // Le mur est à 32 px (2 tuiles) : il arrive AVANT la portée de 80 px.
        let mut c = Ennemi::new(TypeEnnemi::Corbeau, 48.0, 24.0);
        let (min_x, max_x, demi_tours) = patrouille(&mut c, &env, 1500);
        assert!(
            (min_x - TAILLE_TUILE).abs() < 1e-3,
            "s'arrête au mur : {min_x}"
        );
        // À droite, pas de mur : c'est la portée qui joue.
        assert!(
            (max_x - (48.0 + PORTEE_CORBEAU)).abs() < 1e-3,
            "max = {max_x}"
        );
        assert!(demi_tours >= 5, "demi-tours : {demi_tours}");
    }

    #[test]
    fn la_flaque_n_a_pas_de_portee() {
        assert_eq!(TypeEnnemi::Flaque.portee(), None);
        assert_eq!(TypeEnnemi::Corbeau.portee(), Some(PORTEE_CORBEAU));
        // Toit de 28 tuiles entre deux murs : la flaque va d'un mur à l'autre,
        // bien plus loin que la portée d'un corbeau.
        let g = GrilleTuiles::depuis_texte(&[
            "##############################",
            "#............................#",
            "#............................#",
            "##############################",
        ]);
        let env = Environnement::simple(&g);
        let mut f = Ennemi::new(TypeEnnemi::Flaque, 240.0, 40.0);
        let (min_x, max_x, _) = patrouille(&mut f, &env, 2400);
        assert!(
            min_x < TAILLE_TUILE + 1.0,
            "atteint le mur gauche : {min_x}"
        );
        assert!(
            max_x + 12.0 > 29.0 * TAILLE_TUILE - 1.0,
            "atteint le mur droit : {max_x}"
        );
        assert!(240.0 - min_x > PORTEE_CORBEAU && max_x - 240.0 > PORTEE_CORBEAU);
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
