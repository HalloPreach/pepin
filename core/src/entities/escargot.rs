//! L'escargot (lettre `s`) : « un champignon qui marche ».
//!
//! Il patrouille sur son toit EXACTEMENT comme une flaque (même marche :
//! `marche::patrouiller`, demi-tour contre un mur et juste avant le bord du vide),
//! mais lentement (une tuile par seconde). Ce n'est pas un ennemi :
//! - on ne peut pas l'écraser : lui tomber dessus fait rebondir Pépin aussi haut
//!   qu'un champignon (`VITESSE_CHAMPIGNON`), et sa coquille s'aplatit un instant
//!   (il s'arrête le temps de l'animation) ;
//! - le toucher de côté est sans danger : il passe à travers Pépin (et Pépin à
//!   travers lui), rien ne se passe.
//!
//! On s'en sert comme d'un tremplin MOBILE : il faut l'attendre au bon endroit,
//! puis lui sauter dessus quand il passe.
//!
//! Notions Rust : une structure qui RÉUTILISE une fonction libre d'un module
//! voisin (`super::marche`) au lieu de copier son code ; `Clone` pour simuler
//! l'avenir d'une copie sans toucher l'original (`trajet`).

use super::champignon::{COMPRESSION_PAS, VITESSE_CHAMPIGNON};
use super::contacts::MARGE_ECRASEMENT;
use super::Actualisable;
use crate::aabb::Aabb;
use crate::physics::{Environnement, HZ};
use crate::player::Joueur;

/// Vitesse de marche (px/s) : une tuile par seconde, les deux tiers d'une flaque.
pub const VITESSE_ESCARGOT: f32 = 16.0;
/// Boîte : 16 px de large (comme le champignon), 10 px de haut (la coquille).
pub const LARGEUR_ESCARGOT: f32 = 16.0;
pub const HAUTEUR_ESCARGOT: f32 = 10.0;
/// Durée maximale de la simulation de `trajet` (pas) : 2 minutes de marche, de
/// quoi parcourir 120 tuiles.
const TRAJET_MAX_PAS: u32 = 120 * HZ;

#[derive(Debug, Clone)]
pub struct Escargot {
    pub boite: Aabb,
    /// Regarde (et avance) vers la droite ? Il part vers la gauche, comme la flaque.
    pub vers_la_droite: bool,
    vitesse_y: f32,
    au_sol: bool,
    /// Pas d'animation de rebond restants (0 = il marche).
    compression: u8,
}

impl Escargot {
    /// Escargot dont le coin haut-gauche est en `(x, y)`, tourné vers la gauche.
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            boite: Aabb::new(x, y, LARGEUR_ESCARGOT, HAUTEUR_ESCARGOT),
            vers_la_droite: false,
            vitesse_y: 0.0,
            au_sol: false,
            compression: 0,
        }
    }

    /// Pépin lui tombe-t-il dessus en ce moment ? Il descend, chevauche la coquille,
    /// et ses pieds n'y sont pas enfoncés de plus de `MARGE_ECRASEMENT` (même règle
    /// que pour écraser un ennemi). De côté, ou en montant : non.
    pub fn recoit(&self, joueur: &Joueur) -> bool {
        joueur.vitesse.y > 0.0
            && joueur.boite.chevauche(&self.boite)
            && joueur.boite.bas() - self.boite.y <= MARGE_ECRASEMENT
    }

    /// Fait rebondir Pépin comme un champignon ; la coquille s'aplatit et
    /// l'escargot s'arrête pendant `COMPRESSION_PAS` pas.
    pub fn lancer(&mut self, joueur: &mut Joueur) {
        joueur.rebondir(VITESSE_CHAMPIGNON);
        self.compression = COMPRESSION_PAS;
    }

    /// La coquille est-elle aplatie en ce moment (juste après un rebond) ?
    pub fn est_comprime(&self) -> bool {
        self.compression > 0
    }

    /// Abscisses extrêmes `(x_min, x_max)` du coin gauche de sa boîte pendant sa
    /// patrouille, sur une COPIE simulée dans `env` (sans Pépin) jusqu'à son
    /// deuxième demi-tour (le toit parcouru d'un bout à l'autre), ou 2 minutes au
    /// plus. Sert au robot de faisabilité, qui ne connaît pas le temps.
    pub fn trajet(&self, env: &Environnement<'_>) -> (f32, f32) {
        let mut copie = self.clone();
        let (mut x_min, mut x_max) = (copie.boite.x, copie.boite.x);
        let mut demi_tours = 0;
        for _ in 0..TRAJET_MAX_PAS {
            let sens = copie.vers_la_droite;
            copie.actualiser(env);
            x_min = x_min.min(copie.boite.x);
            x_max = x_max.max(copie.boite.x);
            if copie.vers_la_droite != sens {
                demi_tours += 1;
                if demi_tours == 2 {
                    break;
                }
            }
        }
        (x_min, x_max)
    }

    /// Copie immobile dont la boîte couvre TOUT son trajet (voir `trajet`), à la
    /// hauteur où il marche : partout où l'escargot peut passer, Pépin qui tombe
    /// dessus rebondit. Pour le robot de faisabilité seulement.
    pub fn etendu_au_trajet(&self, env: &Environnement<'_>) -> Self {
        let (x_min, x_max) = self.trajet(env);
        let mut pose = self.clone();
        // Un pas pour qu'il se pose (s'il est placé en l'air, il tombe d'abord).
        for _ in 0..HZ {
            pose.actualiser(env);
            if pose.au_sol {
                break;
            }
        }
        let mut etendu = self.clone();
        etendu.boite = Aabb::new(
            x_min,
            pose.boite.y,
            x_max - x_min + LARGEUR_ESCARGOT,
            HAUTEUR_ESCARGOT,
        );
        etendu
    }
}

impl Actualisable for Escargot {
    fn actualiser(&mut self, env: &Environnement<'_>) {
        if self.compression > 0 {
            // Rentré dans sa coquille aplatie : il ne bouge pas.
            self.compression -= 1;
            return;
        }
        super::marche::patrouiller(
            &mut self.boite,
            &mut self.vers_la_droite,
            &mut self.vitesse_y,
            &mut self.au_sol,
            VITESSE_ESCARGOT,
            env,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::contacts::{resoudre_contacts, Contact};
    use crate::entities::{solides_des_entites, Entite};
    use crate::level::Niveau;
    use crate::physics::JOUEUR_H;
    use crate::player::{Direction, Entrees};

    /// Mini-niveau de test, 16 x 12 : un toit de 9 tuiles (colonnes 1 à 9, dessus
    /// en y = 160) bordé d'un mur à gauche (colonne 1, rangées 8 et 9) et d'un vide
    /// à droite (colonnes 10 à 13). L'escargot part de la colonne 6, rangée 9. Huit
    /// rangées de ciel au-dessus : le rebond (≈ 103 px) ne touche pas le haut.
    const TOIT: &str = "
............@...
................
................
................
................
................
................
.o.o.o..........
.B..............
.B....s........X
.#########....##
.#########....##
";

    /// Le mini-niveau, ses entités et Pépin, avancés pas à pas dans l'ordre du
    /// jeu (`Monde::avancer`) : entités, joueur, contacts.
    struct Banc {
        niveau: Niveau,
        entites: Vec<Entite>,
        joueur: Joueur,
    }

    impl Banc {
        fn new(ascii: &str) -> Self {
            let niveau = Niveau::depuis_ascii(ascii).expect("mini-niveau valide");
            let entites = niveau.entites.clone();
            let joueur = Joueur::new(niveau.depart.0, niveau.depart.1);
            Self {
                niveau,
                entites,
                joueur,
            }
        }

        fn escargot(&self) -> &Escargot {
            self.entites
                .iter()
                .find_map(|e| match e {
                    Entite::Escargot(s) => Some(s),
                    _ => None,
                })
                .expect("un escargot dans le mini-niveau")
        }

        /// Un pas complet ; renvoie les contacts.
        fn pas(&mut self, entrees: &Entrees) -> Vec<Contact> {
            let solides = solides_des_entites(&self.entites);
            let env = Environnement {
                grille: &self.niveau,
                solides: &solides,
                courants: &[],
                vents: &[],
            };
            for e in &mut self.entites {
                e.actualiser(&env);
            }
            self.joueur.actualiser(entrees, &env);
            resoudre_contacts(&mut self.joueur, &mut self.entites)
        }

        /// L'escargot seul avance de `n` pas (Pépin ne bouge pas, loin de lui).
        fn escargot_seul(&mut self, n: u32) {
            let env = Environnement::simple(&self.niveau);
            for _ in 0..n {
                for e in &mut self.entites {
                    e.actualiser(&env);
                }
            }
        }
    }

    #[test]
    fn le_parseur_le_pose_au_bas_de_sa_case() {
        let b = Banc::new(TOIT);
        let s = b.escargot();
        assert_eq!(
            (s.boite.x, s.boite.y, s.boite.w, s.boite.h),
            (96.0, 150.0, 16.0, 10.0)
        );
        assert!(!s.vers_la_droite, "il part vers la gauche, comme la flaque");
        assert!(!s.est_comprime());
    }

    #[test]
    fn il_patrouille_lentement_entre_le_mur_et_le_bord_du_vide() {
        let mut b = Banc::new(TOIT);
        b.escargot_seul(30);
        // Posé sur le toit (bas de la boîte en y = 160), il a marché vers la gauche
        // à 16 px/s : 8 px en 30 pas (une demi-seconde).
        let s = b.escargot();
        assert_eq!(s.boite.bas(), 160.0);
        assert!((s.boite.x - 88.0).abs() < 0.01, "x = {}", s.boite.x);
        // Puis il fait les allers-retours : jamais dans le mur (colonne 1), jamais
        // au-dessus du vide (colonne 10 et plus).
        let (mut x_min, mut x_max) = (f32::MAX, f32::MIN);
        let mut demi_tours = 0;
        for _ in 0..60 * 30 {
            let sens = b.escargot().vers_la_droite;
            b.escargot_seul(1);
            let s = b.escargot();
            x_min = x_min.min(s.boite.x);
            x_max = x_max.max(s.boite.x);
            demi_tours += u32::from(s.vers_la_droite != sens);
            assert_eq!(s.boite.bas(), 160.0, "il ne tombe jamais");
        }
        assert!(
            (x_min - 32.0).abs() < 1.0,
            "contre le mur : x_min = {x_min}"
        );
        assert!(
            x_max + LARGEUR_ESCARGOT <= 160.0 && x_max + LARGEUR_ESCARGOT > 158.0,
            "au bord du vide : x_max = {x_max}"
        );
        assert!(demi_tours >= 3, "{demi_tours} demi-tours en 30 s");
    }

    #[test]
    fn le_trajet_couvre_tout_le_toit() {
        let b = Banc::new(TOIT);
        let s = b.escargot().clone();
        let env = Environnement::simple(&b.niveau);
        let (x_min, x_max) = s.trajet(&env);
        assert!((x_min - 32.0).abs() < 1.0, "x_min = {x_min}");
        assert!(x_max + LARGEUR_ESCARGOT > 158.0, "x_max = {x_max}");
        let e = s.etendu_au_trajet(&env);
        assert_eq!((e.boite.y, e.boite.h), (150.0, HAUTEUR_ESCARGOT));
        assert!((e.boite.x - x_min).abs() < 0.01);
        assert!((e.boite.droite() - (x_max + LARGEUR_ESCARGOT)).abs() < 0.01);
    }

    /// Pépin lâché au-dessus de l'escargot : il rebondit aussi haut que sur un
    /// champignon ; l'escargot n'est pas écrasé : sa coquille s'aplatit, il
    /// s'arrête `COMPRESSION_PAS` pas, puis il se remet à marcher.
    #[test]
    fn lui_tomber_dessus_fait_rebondir_comme_un_champignon() {
        let mut b = Banc::new(TOIT);
        let s = b.escargot().boite;
        b.joueur = Joueur::new(s.x + 3.0, s.y - 40.0);
        let mut rebond: Option<(u32, f32)> = None;
        for pas in 0..40 {
            let contacts = b.pas(&Entrees::default());
            assert!(!contacts.iter().any(|c| matches!(c, Contact::Mort(_))));
            if contacts.contains(&Contact::Rebond) {
                assert!(rebond.is_none(), "un seul rebond");
                assert_eq!(b.joueur.vitesse.y, -VITESSE_CHAMPIGNON);
                rebond = Some((pas, b.escargot().boite.x));
            }
            if let Some((p, x)) = rebond {
                let ecoule = pas - p;
                let e = b.escargot();
                if ecoule <= u32::from(COMPRESSION_PAS) {
                    assert_eq!(e.boite.x, x, "aplati, il ne bouge pas ({ecoule})");
                    assert_eq!(e.est_comprime(), ecoule < u32::from(COMPRESSION_PAS));
                } else {
                    assert!(e.boite.x < x, "il remarche ({ecoule})");
                }
            }
        }
        assert!(rebond.is_some(), "Pépin n'a pas rebondi");
    }

    /// Le rebond porte bien plus haut qu'un saut (≈ 57 px) : environ 103 px.
    #[test]
    fn le_rebond_monte_plus_haut_qu_un_saut() {
        let mut b = Banc::new(TOIT);
        let s = b.escargot().boite;
        b.joueur = Joueur::new(s.x + 3.0, s.y - 30.0);
        let mut y_min = f32::MAX;
        for _ in 0..90 {
            b.pas(&Entrees::default());
            y_min = y_min.min(b.joueur.boite.y);
        }
        // Un saut depuis le toit amène la tête en y ≈ 160 - 14 - 57 = 89.
        assert!(y_min < 160.0 - JOUEUR_H - 90.0, "y_min = {y_min}");
    }

    /// Le toucher de côté est sans danger : Pépin marche vers l'escargot, le
    /// traverse, et aucun contact n'est signalé (ni mort, ni rebond).
    #[test]
    fn le_toucher_de_cote_est_sans_danger() {
        let mut b = Banc::new(TOIT);
        // Pépin posé sur le toit, à gauche de l'escargot.
        b.joueur = Joueur::new(40.0, 160.0 - JOUEUR_H);
        let droite = Entrees {
            direction: Direction::Droite,
            saut: false,
        };
        let mut traverse = false;
        for _ in 0..60 {
            let contacts = b.pas(&droite);
            assert!(contacts.is_empty(), "{contacts:?}");
            traverse |= b.joueur.boite.chevauche(&b.escargot().boite);
        }
        assert!(traverse, "Pépin est passé à travers l'escargot");
        assert!(
            b.joueur.boite.x > b.escargot().boite.droite(),
            "de l'autre côté"
        );
        assert!(b.joueur.au_sol);
    }

    /// En montant, ou les pieds déjà enfoncés dans la coquille (arrivé de côté) :
    /// pas de rebond.
    #[test]
    fn en_montant_ou_de_cote_il_ne_rebondit_pas() {
        let b = Banc::new(TOIT);
        let s = b.escargot().boite;
        let mut j = Joueur::new(s.x, s.y + 2.0 - JOUEUR_H);
        j.vitesse.y = -200.0;
        assert!(!b.escargot().recoit(&j), "en montant");
        j.vitesse.y = 100.0;
        j.boite.y = s.y + MARGE_ECRASEMENT + 1.0 - JOUEUR_H;
        assert!(!b.escargot().recoit(&j), "pieds trop enfoncés");
        j.boite.y = s.y + MARGE_ECRASEMENT - JOUEUR_H;
        assert!(b.escargot().recoit(&j), "pieds juste dans la marge");
    }
}
