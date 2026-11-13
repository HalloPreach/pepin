//! Module `player` : état du joueur (Pépin) et sa mise à jour à pas fixe.
//!
//! Mécaniques : course (accélération / friction), saut à hauteur variable,
//! coyote time, jump buffer, PLANER (parapluie ouvert) et courants ascendants.
//!
//! Notions Rust illustrées ici (à reprendre dans le README) :
//! - `enum` sans données (`Direction`, `EtatJoueur`) et `match` exhaustif ;
//! - `struct` d'entrées : on passe à la logique des DONNÉES, pas le clavier ;
//!   le cœur reste pur et testable (les tests « appuient » sur des touches virtuelles) ;
//! - `&mut self` : la méthode modifie le joueur ; `&Entrees` / `&Environnement` :
//!   elle ne fait que LIRE le reste du monde (emprunts partagés) ;
//! - `Default` : valeur de départ « tout à faux / à zéro » ;
//! - tests unitaires `#[cfg(test)]`.

use crate::aabb::{deplacer, Aabb};
use crate::physics::*;

/// Protection après une réapparition : 1 s (60 pas), celui de la réapparition compris.
///
/// Pendant ce temps, aucun DANGER (ennemi, pointe, éclair, et tout danger futur
/// traité par `contacts::resoudre_contacts`) ne peut tuer Pépin ; la chute dans le
/// vide, elle, reste mortelle (sinon il pourrait rester coincé en dessous du monde).
/// Sans elle, un ennemi qui passe sur la lanterne tuerait Pépin à chaque pas :
/// il réapparaîtrait dedans et mourrait de nouveau, jusqu'à ce que l'ennemi s'éloigne.
///
/// Durée FIXE (elle ne s'arrête pas quand le joueur bouge) : plus simple et
/// prévisible. Elle ne se recharge qu'à une réapparition : on ne peut donc pas
/// s'en servir pour traverser un danger, sauf juste à côté de la lanterne.
/// 1 s suffit pour qu'une flaque (24 px/s) passe entièrement sur Pépin immobile :
/// 22 px de chevauchement (12 px de flaque + 10 px de Pépin), soit 0,92 s.
pub const PROTECTION_REAPPARITION_PAS: u16 = HZ as u16;

/// Direction horizontale demandée par le joueur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    Gauche,
    #[default]
    Immobile,
    Droite,
}

impl Direction {
    /// Signe numérique : -1, 0 ou +1. Un `match` doit couvrir TOUTES les variantes,
    /// sinon le programme ne compile pas : impossible d'en oublier une.
    pub fn signe(self) -> f32 {
        match self {
            Direction::Gauche => -1.0,
            Direction::Immobile => 0.0,
            Direction::Droite => 1.0,
        }
    }
}

/// Entrées d'UN pas de simulation (clavier, manette ou tactile, peu importe).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Entrees {
    pub direction: Direction,
    /// Touche de saut ENFONCÉE (maintenue). Le front montant (appui) est détecté
    /// par le joueur lui-même ; maintenue en l'air, elle ouvre le parapluie.
    pub saut: bool,
}

/// Phase de mouvement du joueur (pratique pour choisir l'animation).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtatJoueur {
    Sol,
    Montee,
    Chute,
    Plane,
}

/// Ce qui s'est passé pendant un pas : le jeu s'en sert pour les particules et le son.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Evenements {
    pub saut: bool,
    pub atterrissage: bool,
    pub debut_plane: bool,
}

/// État complet du joueur.
#[derive(Debug, Clone)]
pub struct Joueur {
    /// Boîte de collision (position = coin haut-gauche).
    pub boite: Aabb,
    /// Vitesse en px/s.
    pub vitesse: Vec2,
    /// Le joueur est-il posé sur un solide ?
    pub au_sol: bool,
    /// Regarde vers la droite ? (pour l'affichage)
    pub regarde_droite: bool,
    /// Parapluie ouvert ?
    pub plane: bool,
    saut_precedent: bool,
    /// Pas restants pendant lesquels un saut est encore permis après avoir quitté le sol.
    coyote: u8,
    /// Pas restants pendant lesquels un appui sur saut est mémorisé.
    tampon: u8,
    /// Vrai pendant la montée d'un vrai saut (permet la coupure du saut).
    monte_par_saut: bool,
    /// Pas restants de protection après une réapparition (0 = aucune protection).
    /// Voir `PROTECTION_REAPPARITION_PAS`.
    protection: u16,
}

impl Joueur {
    /// Crée un joueur dont le coin haut-gauche est en `(x, y)`.
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            boite: Aabb::new(x, y, JOUEUR_L, JOUEUR_H),
            vitesse: Vec2::default(),
            au_sol: false,
            regarde_droite: true,
            plane: false,
            saut_precedent: false,
            coyote: 0,
            tampon: 0,
            monte_par_saut: false,
            protection: 0,
        }
    }

    /// Replace le joueur (respawn) en remettant vitesse et minuteries à zéro.
    /// La protection aussi : c'est à l'appelant de la donner (`proteger`), seulement
    /// pour une vraie réapparition (`EtatJeu::traiter`).
    pub fn placer(&mut self, x: f32, y: f32) {
        *self = Joueur::new(x, y);
    }

    /// Donne la protection de réapparition (durée `PROTECTION_REAPPARITION_PAS`).
    /// On REMPLACE le reste éventuel, on n'additionne pas : impossible de cumuler.
    pub fn proteger(&mut self) {
        self.protection = PROTECTION_REAPPARITION_PAS;
    }

    /// Pépin est-il protégé des dangers (juste après une réapparition) ?
    pub fn est_protege(&self) -> bool {
        self.protection > 0
    }

    /// Pas de protection restants (pour l'affichage et les tests).
    pub fn protection_restante(&self) -> u16 {
        self.protection
    }

    /// Phase de mouvement courante, déduite des champs.
    pub fn etat(&self) -> EtatJoueur {
        if self.au_sol {
            EtatJoueur::Sol
        } else if self.plane {
            EtatJoueur::Plane
        } else if self.vitesse.y < 0.0 {
            EtatJoueur::Montee
        } else {
            EtatJoueur::Chute
        }
    }

    /// Rebond forcé (trampoline, écrasement d'ennemi) : propulse vers le haut
    /// à `vitesse` px/s. Ce n'est pas un « saut » : il ne peut pas être coupé.
    pub fn rebondir(&mut self, vitesse: f32) {
        self.vitesse.y = -vitesse;
        self.au_sol = false;
        self.plane = false;
        self.monte_par_saut = false;
        self.coyote = 0;
    }

    /// Déplace le joueur d'une plateforme mobile (sans entrée, sans gravité) en
    /// respectant les collisions. À appeler avec le déplacement de la plateforme.
    pub fn porter(&mut self, dx: f32, dy: f32, env: &Environnement<'_>) {
        deplacer(&mut self.boite, dx, dy, env.grille, env.solides);
    }

    /// Un pas de simulation de 1/60 s.
    pub fn actualiser(&mut self, entrees: &Entrees, env: &Environnement<'_>) -> Evenements {
        let mut ev = Evenements::default();
        let etait_au_sol = self.au_sol;
        let etait_plane = self.plane;

        // --- 0. Protection de réapparition : un pas de moins ----------------
        // `saturating_sub` : 0 - 1 reste 0 (un `u16` ne peut pas être négatif).
        self.protection = self.protection.saturating_sub(1);

        // --- 1. Minuteries : coyote time et jump buffer ---------------------
        let appui = entrees.saut && !self.saut_precedent; // front montant
        self.saut_precedent = entrees.saut;
        if appui {
            self.tampon = TAMPON_SAUT_PAS;
        }
        self.coyote = if self.au_sol {
            COYOTE_PAS
        } else {
            self.coyote.saturating_sub(1)
        };

        // --- 2. Saut --------------------------------------------------------
        if self.tampon > 0 && (self.au_sol || self.coyote > 0) {
            self.vitesse.y = -VITESSE_SAUT;
            self.au_sol = false;
            self.monte_par_saut = true;
            self.tampon = 0;
            self.coyote = 0;
            ev.saut = true;
        } else {
            self.tampon = self.tampon.saturating_sub(1);
        }

        // Saut à hauteur variable : relâcher pendant la montée la raccourcit.
        if self.monte_par_saut && !entrees.saut && self.vitesse.y < -VITESSE_COUPE_SAUT {
            self.vitesse.y = -VITESSE_COUPE_SAUT;
        }
        if self.vitesse.y >= 0.0 {
            self.monte_par_saut = false;
        }

        // --- 3. Planer : saut maintenu, en l'air, en train de descendre -----
        self.plane = !self.au_sol && entrees.saut && self.vitesse.y >= 0.0;
        if self.plane && !etait_plane {
            ev.debut_plane = true;
        }

        // --- 4. Mouvement horizontal ---------------------------------------
        let signe = entrees.direction.signe();
        let (vmax, accel, friction) = if self.au_sol {
            (VITESSE_COURSE, ACCEL_SOL, FRICTION_SOL)
        } else if self.plane {
            (VITESSE_PLANER_X, ACCEL_PLANER, FRICTION_PLANER)
        } else {
            (VITESSE_COURSE, ACCEL_AIR, FRICTION_AIR)
        };
        self.vitesse.x = if signe != 0.0 {
            self.regarde_droite = signe > 0.0;
            approcher(self.vitesse.x, signe * vmax, accel * DT)
        } else {
            approcher(self.vitesse.x, 0.0, friction * DT)
        };

        // --- 5. Gravité (ou parapluie) -------------------------------------
        if self.plane {
            if self.vitesse.y > VITESSE_PLANER_CHUTE {
                // Chute rapide : le parapluie freine progressivement.
                self.vitesse.y = (self.vitesse.y - FREIN_PLANER * DT).max(VITESSE_PLANER_CHUTE);
            } else {
                self.vitesse.y = (self.vitesse.y + GRAVITE * DT).min(VITESSE_PLANER_CHUTE);
            }
        } else {
            self.vitesse.y = (self.vitesse.y + GRAVITE * DT).min(VITESSE_MAX_CHUTE);
        }

        // --- 6. Courants ascendants ----------------------------------------
        // `iter().find` parcourt la tranche en EMPRUNTANT chaque courant.
        if let Some(courant) = env.courants.iter().find(|c| c.zone.chevauche(&self.boite)) {
            if self.vitesse.y > -VITESSE_MAX_COURANT {
                self.vitesse.y =
                    (self.vitesse.y - courant.acceleration * DT).max(-VITESSE_MAX_COURANT);
            }
        }

        // --- 7. Déplacement + collisions, axe par axe ----------------------
        let contacts = deplacer(
            &mut self.boite,
            self.vitesse.x * DT,
            self.vitesse.y * DT,
            env.grille,
            env.solides,
        );
        if contacts.mur() {
            self.vitesse.x = 0.0;
        }
        if contacts.plafond && self.vitesse.y < 0.0 {
            self.vitesse.y = 0.0;
            self.monte_par_saut = false;
        }
        if contacts.sol {
            self.vitesse.y = 0.0;
            self.au_sol = true;
            self.plane = false;
            self.monte_par_saut = false;
            if !etait_au_sol {
                ev.atterrissage = true;
            }
        } else {
            self.au_sol = false;
        }
        ev
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Planer est plus lent que la course. Vérifié à la compilation.
    const _: () = assert!(VITESSE_PLANER_X < VITESSE_COURSE);
    use crate::aabb::{GrilleTuiles, TAILLE_TUILE};

    /// Salle 20 × 12 tuiles : murs latéraux, sol sur la ligne 11 (haut du sol : y = 176),
    /// plafond sur la ligne 0 (bas du plafond : y = 16).
    fn salle() -> GrilleTuiles {
        let mut lignes = vec!["#..................#"; 12];
        lignes[0] = "####################";
        lignes[11] = "####################";
        GrilleTuiles::depuis_texte(&lignes)
    }

    const SOL_Y: f32 = 176.0;
    const AUCUNE: Entrees = Entrees {
        direction: Direction::Immobile,
        saut: false,
    };
    const SAUT: Entrees = Entrees {
        direction: Direction::Immobile,
        saut: true,
    };

    fn droite() -> Entrees {
        Entrees {
            direction: Direction::Droite,
            saut: false,
        }
    }

    /// Joueur posé sur le sol de la salle, à x donné, après quelques pas pour se stabiliser.
    fn au_sol(grille: &GrilleTuiles, x: f32) -> Joueur {
        let env = Environnement::simple(grille);
        let mut j = Joueur::new(x, SOL_Y - JOUEUR_H - 4.0);
        for _ in 0..10 {
            j.actualiser(&AUCUNE, &env);
        }
        assert!(j.au_sol);
        j
    }

    /// Hauteur maximale atteinte (en px au-dessus du point de départ) pendant `n` pas.
    fn hauteur_atteinte(j: &mut Joueur, env: &Environnement<'_>, entrees: &[Entrees]) -> f32 {
        let depart = j.boite.y;
        let mut haut = depart;
        for e in entrees {
            j.actualiser(e, env);
            haut = haut.min(j.boite.y);
        }
        depart - haut
    }

    #[test]
    fn gravite_acceleration_puis_chute() {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut j = Joueur::new(100.0, 40.0);
        j.actualiser(&AUCUNE, &env);
        assert!((j.vitesse.y - GRAVITE * DT).abs() < 1e-3);
        let mut y_prec = j.boite.y;
        for _ in 0..10 {
            j.actualiser(&AUCUNE, &env);
            assert!(j.boite.y > y_prec, "le joueur descend");
            y_prec = j.boite.y;
        }
        assert_eq!(j.etat(), EtatJoueur::Chute);
        // Chute plus longue : la vitesse ne dépasse jamais le plafond.
        let mut j = Joueur::new(100.0, -2000.0);
        for _ in 0..200 {
            j.actualiser(&AUCUNE, &env);
            assert!(j.vitesse.y <= VITESSE_MAX_CHUTE);
        }
        assert_eq!(j.vitesse.y, VITESSE_MAX_CHUTE);
    }

    #[test]
    fn atterrissage_pile_sur_le_sol() {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut j = Joueur::new(100.0, 60.0);
        let mut atterri = false;
        for _ in 0..120 {
            let ev = j.actualiser(&AUCUNE, &env);
            atterri |= ev.atterrissage;
        }
        assert!(atterri, "l'événement d'atterrissage a bien été émis");
        assert!(j.au_sol);
        assert_eq!(j.etat(), EtatJoueur::Sol);
        assert_eq!(j.boite.bas(), SOL_Y);
        assert_eq!(j.vitesse.y, 0.0);
    }

    #[test]
    fn course_acceleration_puis_friction() {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut j = au_sol(&g, 100.0);
        j.actualiser(&droite(), &env);
        assert!(
            (j.vitesse.x - ACCEL_SOL * DT).abs() < 1e-3,
            "accélère progressivement"
        );
        for _ in 0..30 {
            j.actualiser(&droite(), &env);
        }
        assert_eq!(
            j.vitesse.x, VITESSE_COURSE,
            "plafonnée à la vitesse de course"
        );
        assert!(j.regarde_droite);
        let mut dernier = j.vitesse.x;
        for _ in 0..30 {
            j.actualiser(&AUCUNE, &env);
            assert!(j.vitesse.x <= dernier);
            dernier = j.vitesse.x;
        }
        assert_eq!(j.vitesse.x, 0.0, "la friction arrête le joueur");
    }

    #[test]
    fn collision_mur() {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut j = au_sol(&g, 100.0);
        for _ in 0..400 {
            j.actualiser(&droite(), &env);
        }
        assert_eq!(j.boite.droite(), 19.0 * TAILLE_TUILE);
        assert_eq!(j.vitesse.x, 0.0);
        assert!(j.au_sol);
    }

    #[test]
    fn collision_plafond() {
        // Plafond bas : sa face inférieure est à y = 144, soit 18 px au-dessus de la tête.
        let mut lignes = vec!["#..................#"; 12];
        lignes[8] = "####################";
        lignes[11] = "####################";
        let g = GrilleTuiles::depuis_texte(&lignes);
        let env = Environnement::simple(&g);
        let mut j = au_sol(&g, 100.0);
        let mut touche = false;
        for _ in 0..30 {
            j.actualiser(&SAUT, &env);
            if j.boite.y == 144.0 {
                touche = true;
                assert_eq!(j.vitesse.y, 0.0, "la montée est coupée net");
                break;
            }
        }
        assert!(touche, "la tête a touché le plafond");
        j.actualiser(&SAUT, &env);
        assert!(j.vitesse.y > 0.0, "puis le joueur retombe");
    }

    #[test]
    fn saut_complet_atteint_la_hauteur_prevue() {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut j = au_sol(&g, 100.0);
        let h = hauteur_atteinte(&mut j, &env, &[SAUT; 40]);
        let theorique = hauteur_saut_theorique();
        assert!(
            (h - theorique).abs() < 6.0,
            "h = {h}, théorique = {theorique}"
        );
    }

    #[test]
    fn saut_variable_relacher_raccourcit() {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut long = au_sol(&g, 100.0);
        let mut court = au_sol(&g, 100.0);
        let mut touches_court = vec![SAUT];
        touches_court.extend([AUCUNE; 39]);
        let h_long = hauteur_atteinte(&mut long, &env, &[SAUT; 40]);
        let h_court = hauteur_atteinte(&mut court, &env, &touches_court);
        assert!(h_court > 5.0, "même un appui bref décolle ({h_court})");
        assert!(h_court < 20.0, "saut court = {h_court}");
        assert!(
            h_long - h_court > 30.0,
            "long = {h_long}, court = {h_court}"
        );
    }

    /// Fait marcher le joueur vers la droite jusqu'au bord d'une corniche (x = 112).
    fn joueur_au_bord(g: &GrilleTuiles) -> Joueur {
        let env = Environnement::simple(g);
        let mut j = au_sol(g, 60.0);
        for _ in 0..200 {
            j.actualiser(&droite(), &env);
            if !j.au_sol {
                return j;
            }
        }
        panic!("le joueur n'a jamais quitté la corniche");
    }

    fn corniche() -> GrilleTuiles {
        let mut lignes = vec!["#..................#"; 12];
        lignes[0] = "####################";
        lignes[11] = "#######.............";
        GrilleTuiles::depuis_texte(&lignes)
    }

    #[test]
    fn coyote_time_saut_juste_apres_le_bord() {
        let g = corniche();
        let env = Environnement::simple(&g);
        let mut j = joueur_au_bord(&g);
        for _ in 0..3 {
            j.actualiser(&AUCUNE, &env);
        }
        assert!(!j.au_sol);
        let ev = j.actualiser(&SAUT, &env);
        assert!(ev.saut, "saut accepté dans la fenêtre de coyote time");
        assert!(j.vitesse.y < 0.0);
    }

    #[test]
    fn coyote_time_expire() {
        let g = corniche();
        let env = Environnement::simple(&g);
        let mut j = joueur_au_bord(&g);
        for _ in 0..(COYOTE_PAS as usize + 4) {
            j.actualiser(&AUCUNE, &env);
        }
        let ev = j.actualiser(&SAUT, &env);
        assert!(!ev.saut, "trop tard : plus de saut en l'air");
        assert!(j.vitesse.y >= 0.0);
    }

    /// Lâche un joueur de haut ; appuie sur SAUT (un seul pas) quand il est à moins de
    /// `distance` px du sol ; renvoie `true` si un saut a eu lieu à l'atterrissage.
    fn appui_avant_atterrissage(distance: f32) -> bool {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut j = Joueur::new(100.0, 40.0);
        let mut deja_appuye = false;
        let mut a_saute = false;
        for _ in 0..150 {
            let doit_appuyer = !deja_appuye && j.boite.bas() >= SOL_Y - distance;
            deja_appuye |= doit_appuyer;
            let e = if doit_appuyer { SAUT } else { AUCUNE };
            a_saute |= j.actualiser(&e, &env).saut;
        }
        a_saute
    }

    #[test]
    fn jump_buffer_appui_juste_avant_le_sol() {
        assert!(
            appui_avant_atterrissage(12.0),
            "appui ~3 pas avant : mémorisé"
        );
    }

    #[test]
    fn jump_buffer_expire() {
        assert!(!appui_avant_atterrissage(90.0), "appui trop tôt : oublié");
    }

    #[test]
    fn planer_plafonne_la_chute_et_relacher_la_retablit() {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut j = Joueur::new(100.0, 24.0);
        // Chute libre puis parapluie ouvert.
        for _ in 0..8 {
            j.actualiser(&AUCUNE, &env);
        }
        assert!(j.vitesse.y > VITESSE_PLANER_CHUTE);
        for _ in 0..20 {
            j.actualiser(&SAUT, &env);
            assert!(j.vitesse.y <= VITESSE_MAX_CHUTE);
        }
        assert_eq!(j.etat(), EtatJoueur::Plane);
        assert_eq!(j.vitesse.y, VITESSE_PLANER_CHUTE, "chute plafonnée");
        for _ in 0..10 {
            j.actualiser(&SAUT, &env);
            assert_eq!(j.vitesse.y, VITESSE_PLANER_CHUTE, "reste plafonnée");
        }
        // Parapluie fermé : la gravité reprend.
        for _ in 0..10 {
            j.actualiser(&AUCUNE, &env);
        }
        assert_eq!(j.etat(), EtatJoueur::Chute);
        assert!(
            j.vitesse.y > VITESSE_PLANER_CHUTE + 50.0,
            "vy = {}",
            j.vitesse.y
        );
    }

    #[test]
    fn planer_ne_demarre_pas_pendant_la_montee() {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut j = au_sol(&g, 100.0);
        let ev = j.actualiser(&SAUT, &env);
        assert!(ev.saut && !j.plane);
        assert_eq!(j.etat(), EtatJoueur::Montee);
        // Bouton maintenu : après le sommet, le parapluie s'ouvre tout seul.
        let mut a_plane = false;
        for _ in 0..60 {
            a_plane |= j.actualiser(&SAUT, &env).debut_plane;
        }
        assert!(a_plane);
    }

    #[test]
    fn planer_controle_horizontal_leger() {
        let g = salle();
        let env = Environnement::simple(&g);
        let planer_droite = Entrees {
            direction: Direction::Droite,
            saut: true,
        };
        let mut j = Joueur::new(60.0, 24.0);
        for _ in 0..50 {
            j.actualiser(&planer_droite, &env);
        }
        assert_eq!(j.etat(), EtatJoueur::Plane);
        assert_eq!(j.vitesse.x, VITESSE_PLANER_X, "plus lent que la course");
        // Sans parapluie, la même chute permet de dépasser cette vitesse.
        let mut k = Joueur::new(60.0, 24.0);
        for _ in 0..25 {
            k.actualiser(&droite(), &env);
        }
        assert!(k.vitesse.x > VITESSE_PLANER_X);
    }

    #[test]
    fn courant_ascendant_fait_monter() {
        let g = salle();
        let courants = [CourantAscendant::new(Aabb::new(48.0, 64.0, 32.0, 112.0))];
        let env = Environnement {
            grille: &g,
            solides: &[],
            courants: &courants,
        };
        let mut j = au_sol(&g, 56.0);
        let depart = j.boite.y;
        let mut vy_min = 0.0_f32;
        for _ in 0..90 {
            j.actualiser(&AUCUNE, &env);
            vy_min = vy_min.min(j.vitesse.y);
        }
        assert_eq!(vy_min, -VITESSE_MAX_COURANT, "montée plafonnée");
        assert!(j.boite.y < depart - 60.0, "le vent a soulevé le joueur");
        // Hors du courant : aucun effet, le joueur reste au sol.
        let mut hors = au_sol(&g, 200.0);
        for _ in 0..30 {
            hors.actualiser(&AUCUNE, &env);
        }
        assert!(hors.au_sol);
    }

    #[test]
    fn planer_dans_un_courant_ne_tombe_pas() {
        let g = salle();
        let courants = [CourantAscendant::new(Aabb::new(48.0, 32.0, 32.0, 144.0))];
        let env = Environnement {
            grille: &g,
            solides: &[],
            courants: &courants,
        };
        let mut j = Joueur::new(56.0, 120.0);
        for _ in 0..90 {
            j.actualiser(&SAUT, &env);
        }
        assert!(j.boite.y < 100.0, "y = {}", j.boite.y);
    }

    #[test]
    fn rebond_et_placer() {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut j = au_sol(&g, 100.0);
        j.rebondir(400.0);
        assert_eq!(j.vitesse.y, -400.0);
        assert!(!j.au_sol);
        j.actualiser(&AUCUNE, &env);
        assert!(j.vitesse.y < 0.0);
        j.placer(30.0, 30.0);
        assert_eq!(
            (j.boite.x, j.boite.y, j.vitesse),
            (30.0, 30.0, Vec2::default())
        );
    }

    /// La protection de réapparition dure exactement `PROTECTION_REAPPARITION_PAS`
    /// pas (celui de la réapparition compris, donc `- 1` pas de simulation ensuite),
    /// puis expire ; un nouveau `placer` l'efface, `proteger` la remet au maximum
    /// sans jamais la cumuler.
    #[test]
    fn la_protection_de_reapparition_expire() {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut j = au_sol(&g, 100.0);
        assert!(!j.est_protege(), "pas de protection au départ");
        j.proteger();
        assert_eq!(j.protection_restante(), PROTECTION_REAPPARITION_PAS);
        for k in 1..PROTECTION_REAPPARITION_PAS {
            j.actualiser(&droite(), &env);
            assert!(j.est_protege(), "encore protégé au pas {k}");
        }
        j.actualiser(&droite(), &env);
        assert!(!j.est_protege(), "protection expirée");
        j.actualiser(&droite(), &env);
        assert_eq!(j.protection_restante(), 0, "reste à 0");

        j.proteger();
        j.actualiser(&AUCUNE, &env);
        j.proteger();
        assert_eq!(
            j.protection_restante(),
            PROTECTION_REAPPARITION_PAS,
            "pas de cumul"
        );
        j.placer(30.0, 30.0);
        assert!(!j.est_protege(), "placer seul ne protège pas");
        assert_eq!(PROTECTION_REAPPARITION_PAS, 60, "1 s à 60 Hz");
    }

    #[test]
    fn porter_deplace_avec_collisions() {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut j = au_sol(&g, 100.0);
        j.porter(10.0, 0.0, &env);
        assert_eq!(j.boite.x, 110.0);
        j.porter(500.0, 0.0, &env);
        assert_eq!(j.boite.droite(), 19.0 * TAILLE_TUILE);
    }

    #[test]
    fn simulation_deterministe() {
        // Deux exécutions identiques donnent EXACTEMENT les mêmes nombres.
        let g = salle();
        let env = Environnement::simple(&g);
        let scenario = |n: usize| {
            let mut j = Joueur::new(40.0, 100.0);
            for i in 0..n {
                let e = Entrees {
                    direction: if i % 50 < 30 {
                        Direction::Droite
                    } else {
                        Direction::Gauche
                    },
                    saut: i % 40 < 15,
                };
                j.actualiser(&e, &env);
            }
            (j.boite, j.vitesse)
        };
        assert_eq!(scenario(500), scenario(500));
    }
}
