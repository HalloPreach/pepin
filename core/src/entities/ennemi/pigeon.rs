//! Le pigeon (lettre `p`) : perché au bord d'un solide, il guette. Quand Pépin
//! passe DESSOUS, dans un cône devant lui, il frémit un court instant (l'annonce),
//! puis plonge en diagonale, remonte se poser sur son perchoir et s'y repose
//! avant de pouvoir replonger.
//!
//! ```text
//!   phase :  Perché ──(Pépin dans le cône)──> Annonce (0,3 s)
//!              ^                                  │
//!              │                                  v
//!            Repos (1,5 s) <── Remontée (1,07 s) <── Plongée (0,53 s)
//! ```
//!
//! Sa trajectoire ne dépend que de son perchoir et du sens où il regarde, fixé
//! au déclenchement : une diagonale à 45° vers le bas et vers l'avant, longue de
//! `PROFONDEUR_PLONGEE` px dans chaque axe, puis le même trait en sens inverse.
//! Il vole : les tuiles ne l'arrêtent pas. C'est au niveau de lui laisser le
//! passage : un test le vérifie sur les vrais niveaux (`core/src/zones.rs`), et
//! sa plongée ne passe jamais sur une lanterne.
//!
//! Ce fichier est un SOUS-MODULE de `ennemi` : il peut lire et modifier les
//! champs privés de `Ennemi` (en Rust, un module voit ce qui est privé dans ses
//! modules parents), sans les rendre publics pour le reste du programme.

use super::{Ennemi, EtatEnnemi, TypeEnnemi};
use crate::aabb::Aabb;

/// Durée (pas) du frémissement d'annonce avant la plongée : 0,3 s.
pub const ANNONCE_PIGEON_PAS: u8 = 18;
/// Profondeur (px) de la plongée, dans chaque axe : 4 tuiles vers le bas et
/// 4 tuiles vers l'avant (une diagonale à 45°). Perché sur un nuage 4 rangées
/// au-dessus d'un toit, le pigeon finit sa plongée au ras de ce toit : il
/// atteint Pépin qui y marche (le haut de Pépin est 50 px sous ses pattes).
pub const PROFONDEUR_PLONGEE: f32 = 64.0;
/// Durée (pas) de la plongée : 32 pas, soit 2 px par pas dans chaque axe
/// (120 px/s, plus vite que la course de Pépin, 96 px/s).
pub const PLONGEE_PAS: u16 = 32;
/// Durée (pas) de la remontée vers le perchoir : deux fois plus lente (60 px/s).
pub const REMONTEE_PAS: u16 = 64;
/// Repos (pas) sur le perchoir après la remontée, avant de pouvoir replonger : 1,5 s.
pub const REPOS_PIGEON_PAS: u16 = 90;
/// Tolérance (px) du cône vers l'arrière : Pépin juste sous le bec compte encore
/// comme « dessous ».
pub const MARGE_CONE: f32 = 6.0;

/// Où en est le pigeon. Les flaques et les corbeaux restent toujours `Perche`.
///
/// Chaque variante porte le compteur dont elle a besoin (un `enum` à données) :
/// impossible d'avoir un compte de plongée pendant le repos, par exemple.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhasePigeon {
    /// Posé sur son perchoir, il guette.
    Perche,
    /// Il frémit : plus que `n` pas avant de plonger.
    Annonce(u8),
    /// Il plonge depuis `n` pas (1 à `PLONGEE_PAS`).
    Plongee(u16),
    /// Il remonte depuis `n` pas (1 à `REMONTEE_PAS`).
    Remontee(u16),
    /// Revenu sur son perchoir, il ne guette pas encore : plus que `n` pas.
    Repos(u16),
}

/// Décalage `(dx, dy)` du pigeon par rapport à son perchoir pendant le vol, pour
/// un sens `sens` (+1 : vers la droite, -1 : vers la gauche). Fonction pure : la
/// position se recalcule à chaque pas depuis le perchoir, sans accumuler
/// d'arrondis, et il revient donc EXACTEMENT à son point de départ.
pub fn decalage_vol(phase: PhasePigeon, sens: f32) -> (f32, f32) {
    let d = match phase {
        PhasePigeon::Plongee(n) => PROFONDEUR_PLONGEE * f32::from(n) / f32::from(PLONGEE_PAS),
        PhasePigeon::Remontee(n) => {
            PROFONDEUR_PLONGEE * (1.0 - f32::from(n) / f32::from(REMONTEE_PAS))
        }
        PhasePigeon::Perche | PhasePigeon::Annonce(_) | PhasePigeon::Repos(_) => 0.0,
    };
    (sens * d, d)
}

/// Phase suivante (un pas de 1/60 s). Fonction pure : toute la machine à états
/// tient dans ce `match`. `Perche` ne change pas tout seul : c'est `guetter` qui
/// déclenche l'annonce.
pub fn phase_suivante(phase: PhasePigeon) -> PhasePigeon {
    match phase {
        PhasePigeon::Perche => PhasePigeon::Perche,
        PhasePigeon::Annonce(n) if n > 1 => PhasePigeon::Annonce(n - 1),
        PhasePigeon::Annonce(_) => PhasePigeon::Plongee(1),
        PhasePigeon::Plongee(n) if n < PLONGEE_PAS => PhasePigeon::Plongee(n + 1),
        PhasePigeon::Plongee(_) => PhasePigeon::Remontee(1),
        PhasePigeon::Remontee(n) if n < REMONTEE_PAS => PhasePigeon::Remontee(n + 1),
        PhasePigeon::Remontee(_) => PhasePigeon::Repos(REPOS_PIGEON_PAS),
        PhasePigeon::Repos(n) if n > 1 => PhasePigeon::Repos(n - 1),
        PhasePigeon::Repos(_) => PhasePigeon::Perche,
    }
}

impl Ennemi {
    /// Pose un pigeon dont le coin haut-gauche est en `(x, y)` (sur son perchoir),
    /// tourné vers la droite ou vers la gauche : c'est de ce côté qu'il plongera.
    pub fn pigeon(x: f32, y: f32, vers_la_droite: bool) -> Self {
        let mut p = Ennemi::new(TypeEnnemi::Pigeon, x, y);
        p.vers_la_droite = vers_la_droite;
        p
    }

    /// Phase de vol (toujours `Perche` pour un autre ennemi que le pigeon).
    pub fn phase(&self) -> PhasePigeon {
        self.phase
    }

    /// Coin haut-gauche du perchoir (point d'apparition).
    pub fn perchoir(&self) -> (f32, f32) {
        (self.origine_x, self.origine_y)
    }

    /// Pépin (sa boîte `joueur`) est-il dans le cône que le pigeon surveille ?
    ///
    /// Le cône part des pattes du pigeon et s'ouvre vers le bas, entre la
    /// verticale (à `MARGE_CONE` près vers l'arrière) et la diagonale à 45° de
    /// sa plongée, vers l'avant ; il s'arrête à la profondeur de la plongée.
    /// Il faut donc que Pépin soit tout entier SOUS le pigeon : à la même
    /// hauteur, ou au-dessus, il ne déclenche rien. Fonction pure (lecture seule).
    pub fn dans_le_cone(&self, joueur: &Aabb) -> bool {
        // Profondeur : du bas du pigeon au haut de Pépin.
        let dessous = joueur.y - self.boite.bas();
        // Avance : de son centre à celui de Pépin, comptée vers l'avant.
        let avance = (joueur.centre().0 - self.boite.centre().0) * self.sens();
        (0.0..=PROFONDEUR_PLONGEE).contains(&dessous)
            && avance >= -MARGE_CONE
            && avance <= dessous + MARGE_CONE
    }

    /// Le pigeon regarde Pépin : s'il est perché (ni en vol, ni au repos), actif,
    /// et que Pépin est dans son cône, il commence son annonce. Le sens de la
    /// plongée est celui où il regarde : fixé ici, il ne suivra pas Pépin.
    /// Sans effet pour les autres ennemis.
    pub fn guetter(&mut self, joueur: &Aabb) {
        if self.type_ennemi == TypeEnnemi::Pigeon
            && self.etat == EtatEnnemi::Actif
            && self.phase == PhasePigeon::Perche
            && self.dans_le_cone(joueur)
        {
            self.phase = PhasePigeon::Annonce(ANNONCE_PIGEON_PAS);
        }
    }

    /// Un pas de vol du pigeon : la phase avance, la boîte suit la trajectoire.
    pub(super) fn voler_pigeon(&mut self) {
        self.phase = phase_suivante(self.phase);
        let (dx, dy) = decalage_vol(self.phase, self.sens());
        self.boite.x = self.origine_x + dx;
        self.boite.y = self.origine_y + dy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::GrilleTuiles;
    use crate::entities::ennemi::ANIMATION_ECRASE_PAS;
    use crate::entities::Actualisable;
    use crate::physics::{Environnement, JOUEUR_H, JOUEUR_L};

    /// Pigeon tourné vers la gauche, pattes en y = 64 (posé sur une tuile en
    /// rangée 4), boîte de 12 x 10 en (100, 54).
    fn pigeon_gauche() -> Ennemi {
        Ennemi::pigeon(100.0, 54.0, false)
    }

    /// Boîte de Pépin dont le centre est en `(cx, cy)`.
    fn pepin_centre(cx: f32, cy: f32) -> Aabb {
        Aabb::new(cx - JOUEUR_L / 2.0, cy - JOUEUR_H / 2.0, JOUEUR_L, JOUEUR_H)
    }

    /// Fait avancer `p` de `pas` pas, sans Pépin (ciel vide).
    fn avancer(p: &mut Ennemi, pas: u32) {
        let g = GrilleTuiles::vide(40, 12);
        let env = Environnement::simple(&g);
        for _ in 0..pas {
            p.actualiser(&env);
        }
    }

    #[test]
    fn le_pigeon_ne_se_declenche_que_si_pepin_passe_dessous_devant_lui() {
        let p = pigeon_gauche();
        let (cx, _) = p.boite.centre(); // 106
                                        // Sous lui, 40 px plus bas (haut de Pépin en y = 104).
        let sous = |avance: f32| pepin_centre(cx - avance, 64.0 + 40.0 + JOUEUR_H / 2.0);
        assert!(p.dans_le_cone(&sous(0.0)), "juste dessous");
        assert!(p.dans_le_cone(&sous(30.0)), "dessous, devant");
        assert!(p.dans_le_cone(&sous(40.0 + MARGE_CONE)), "sur la diagonale");
        assert!(
            !p.dans_le_cone(&sous(50.0)),
            "trop loin devant (hors du cône)"
        );
        assert!(!p.dans_le_cone(&sous(-12.0)), "derrière lui");
        // À la même hauteur que lui, ou au-dessus : jamais, même tout près.
        assert!(!p.dans_le_cone(&pepin_centre(cx - 10.0, 59.0)), "à côté");
        assert!(!p.dans_le_cone(&pepin_centre(cx - 4.0, 30.0)), "au-dessus");
        // Plus bas que la plongée ne descend : il ne se donne pas la peine.
        let tres_bas = pepin_centre(cx - 10.0, 64.0 + PROFONDEUR_PLONGEE + 20.0);
        assert!(!p.dans_le_cone(&tres_bas), "hors de portée");
        // Retourné vers la droite, le cône change de côté.
        let d = Ennemi::pigeon(100.0, 54.0, true);
        assert!(d.dans_le_cone(&sous(-30.0)) && !d.dans_le_cone(&sous(30.0)));

        // `guetter` ne lance l'annonce que dans le cône.
        let mut q = pigeon_gauche();
        q.guetter(&pepin_centre(cx - 10.0, 59.0));
        assert_eq!(q.phase(), PhasePigeon::Perche, "Pépin à côté : rien");
        q.guetter(&sous(20.0));
        assert_eq!(q.phase(), PhasePigeon::Annonce(ANNONCE_PIGEON_PAS));
        assert_eq!(q.boite.x, 100.0, "pendant l'annonce, il reste perché");
    }

    #[test]
    fn la_flaque_et_le_corbeau_ne_guettent_pas() {
        let pepin = pepin_centre(100.0, 120.0);
        for t in [TypeEnnemi::Flaque, TypeEnnemi::Corbeau] {
            let mut e = Ennemi::new(t, 100.0, 54.0);
            e.guetter(&pepin);
            assert_eq!(e.phase(), PhasePigeon::Perche, "{t:?}");
        }
    }

    #[test]
    fn la_plongee_suit_une_diagonale_a_45_degres_vers_l_avant() {
        for (vers_la_droite, sens) in [(false, -1.0), (true, 1.0)] {
            let mut p = Ennemi::pigeon(100.0, 54.0, vers_la_droite);
            p.phase = PhasePigeon::Annonce(ANNONCE_PIGEON_PAS);
            // Annonce : immobile, puis plongée au pas suivant.
            avancer(&mut p, ANNONCE_PIGEON_PAS as u32);
            assert_eq!(p.phase(), PhasePigeon::Plongee(1));
            for n in 1..PLONGEE_PAS {
                let (dx, dy) = (p.boite.x - 100.0, p.boite.y - 54.0);
                assert!(dy > 0.0, "il descend (pas {n})");
                assert!(
                    (dx - sens * dy).abs() < 1e-4,
                    "45° vers l'avant : {dx}, {dy}"
                );
                avancer(&mut p, 1);
            }
            // Au fond : 4 tuiles plus bas, 4 tuiles devant.
            assert_eq!(p.phase(), PhasePigeon::Plongee(PLONGEE_PAS));
            assert_eq!(
                (p.boite.x, p.boite.y),
                (100.0 + sens * PROFONDEUR_PLONGEE, 54.0 + PROFONDEUR_PLONGEE)
            );
        }
    }

    #[test]
    fn la_direction_est_fixee_au_declenchement() {
        // Pépin passe ensuite derrière le pigeon : la plongée ne le suit pas.
        let mut p = pigeon_gauche();
        p.guetter(&pepin_centre(90.0, 100.0));
        avancer(&mut p, ANNONCE_PIGEON_PAS as u32 + 10);
        p.guetter(&pepin_centre(200.0, 100.0));
        avancer(&mut p, 10);
        assert!(p.boite.x < 100.0, "il continue vers la gauche");
        assert!(!p.vers_la_droite);
    }

    #[test]
    fn il_remonte_se_poser_exactement_sur_son_perchoir_puis_se_repose() {
        let mut p = pigeon_gauche();
        p.guetter(&pepin_centre(100.0, 100.0));
        // Le dernier pas de l'annonce est déjà le premier de la plongée.
        avancer(&mut p, ANNONCE_PIGEON_PAS as u32 + PLONGEE_PAS as u32 - 1);
        assert_eq!(p.phase(), PhasePigeon::Plongee(PLONGEE_PAS), "au fond");
        // Remontée : le même trait, en sens inverse, deux fois plus lentement.
        let mut avant = p.boite.y;
        for _ in 0..REMONTEE_PAS {
            avancer(&mut p, 1);
            assert!(p.boite.y < avant, "il remonte");
            assert!((avant - p.boite.y - 1.0).abs() < 1e-4, "1 px par pas");
            let (dx, dy) = (p.boite.x - 100.0, p.boite.y - 54.0);
            assert!((dx + dy).abs() < 1e-4, "sur la même diagonale : {dx}, {dy}");
            avant = p.boite.y;
        }
        assert_eq!(p.perchoir(), (100.0, 54.0));
        assert_eq!(
            (p.boite.x, p.boite.y),
            p.perchoir(),
            "pile sur son perchoir"
        );
        avancer(&mut p, 1);
        assert_eq!(p.phase(), PhasePigeon::Repos(REPOS_PIGEON_PAS));
        assert_eq!((p.boite.x, p.boite.y), p.perchoir(), "il s'y repose");
        assert!(!p.vers_la_droite, "toujours tourné du même côté");
    }

    #[test]
    fn apres_la_remontee_il_ne_replonge_qu_apres_son_repos() {
        let mut p = pigeon_gauche();
        let pepin = pepin_centre(100.0, 100.0); // toujours dans le cône
        p.guetter(&pepin);
        let vol = ANNONCE_PIGEON_PAS as u32 + PLONGEE_PAS as u32 + REMONTEE_PAS as u32;
        for _ in 0..vol {
            avancer(&mut p, 1);
            p.guetter(&pepin); // en vol : sans effet
        }
        assert_eq!(p.phase(), PhasePigeon::Repos(REPOS_PIGEON_PAS));
        // Pendant le repos, Pépin a beau rester dessous : rien.
        for _ in 0..REPOS_PIGEON_PAS - 1 {
            avancer(&mut p, 1);
            p.guetter(&pepin);
            assert!(matches!(p.phase(), PhasePigeon::Repos(_)));
        }
        avancer(&mut p, 1);
        assert_eq!(p.phase(), PhasePigeon::Perche);
        p.guetter(&pepin);
        assert_eq!(p.phase(), PhasePigeon::Annonce(ANNONCE_PIGEON_PAS));
    }

    #[test]
    fn le_cycle_complet_dure_ce_qu_annoncent_les_constantes() {
        let mut phase = PhasePigeon::Annonce(ANNONCE_PIGEON_PAS);
        let mut pas = 0;
        while phase != PhasePigeon::Perche {
            phase = phase_suivante(phase);
            pas += 1;
        }
        let attendu = ANNONCE_PIGEON_PAS as u32
            + PLONGEE_PAS as u32
            + REMONTEE_PAS as u32
            + REPOS_PIGEON_PAS as u32;
        assert_eq!(pas, attendu);
        // Perché, sans Pépin : il ne bouge pas.
        assert_eq!(phase_suivante(PhasePigeon::Perche), PhasePigeon::Perche);
        assert_eq!(decalage_vol(PhasePigeon::Repos(3), -1.0), (0.0, 0.0));
    }

    #[test]
    fn un_pigeon_ecrase_ne_vole_plus_et_ne_guette_plus() {
        let mut p = pigeon_gauche();
        p.guetter(&pepin_centre(100.0, 100.0));
        avancer(&mut p, ANNONCE_PIGEON_PAS as u32 + 10);
        let ici = (p.boite.x, p.boite.y);
        p.ecraser();
        assert!(!p.est_dangereux() && p.est_visible());
        avancer(&mut p, 5);
        assert_eq!((p.boite.x, p.boite.y), ici, "écrasé en vol : il reste là");
        avancer(&mut p, ANIMATION_ECRASE_PAS as u32);
        assert!(!p.est_visible());
        // Un pigeon écrasé ne se déclenche plus.
        let mut q = pigeon_gauche();
        q.ecraser();
        q.guetter(&pepin_centre(100.0, 100.0));
        assert_eq!(q.phase(), PhasePigeon::Perche);
    }
}
