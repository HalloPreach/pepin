//! Le chat endormi (lettre `z`) : couché sur un toit, il dort, se réveille
//! brièvement, puis se rendort, en boucle. Le réveil est ANNONCÉ, comme un éclair.
//! Depuis la carte 6.3, il est plus menaçant : il dort moins longtemps, et s'il
//! voit Pépin à la fin de l'annonce, il lui BONDIT dessus.
//!
//! ```text
//!   phase :  Dort (1,7 s) ──> Réveil (0,8 s, l'annonce) ──> Éveillé (1,2 s) ──┐
//!              ^                                                               │
//!              └───────────────────────────────────────────────────────────────┘
//!
//!   éveil avec bond :  aller (0,5 s) ─> posé (0,2 s) ─> retour (0,5 s) ─> il se rendort
//! ```
//!
//! - Endormi (`Dort`) ou en train de se réveiller (`Reveil`) : c'est un ennemi
//!   comme les autres. On l'écrase en lui sautant dessus ; le heurter de côté tue
//!   (il se réveille en sursaut et griffe).
//! - Pendant l'annonce, s'il voit Pépin à portée de bond (`voit_pepin` : 5 tuiles
//!   en x, 2 en hauteur, aucun mur entre eux), il se ramasse, prêt à bondir
//!   (`est_pret_a_bondir`, une posture à part à l'écran).
//! - À la fin de l'annonce, s'il le voit encore, il bondit vers l'endroit où se
//!   tient Pépin (`cible_du_bond`) : un saut en arc, avec la gravité de Pépin,
//!   qui ne quitte jamais son toit (il s'arrête au bord du vide) et ne traverse
//!   aucun mur ; il s'y pose un instant, puis revient à sa place d'un second
//!   bond, le même à l'envers, et se rendort. Sinon, il se réveille sur place.
//! - Éveillé (`Eveille`), qu'il bondisse ou non : on ne peut PAS l'écraser. Le
//!   toucher, même par le dessus, tue Pépin. Rendormi, il redevient écrasable.
//!
//! Sa trajectoire est une fonction PURE du temps (`decalage_chat`), comme la
//! plongée du pigeon : elle est vérifiée en entier au moment du départ, et il
//! revient EXACTEMENT à sa place, sans accumuler d'arrondis.
//!
//! Comme le pigeon, ce fichier est un SOUS-MODULE de `ennemi` : il lit et modifie
//! les champs privés de `Ennemi` sans les rendre publics.
//!
//! Notions Rust : un `enum` à données (`PhaseChat`) dont chaque variante porte
//! son compteur ; des fonctions pures (`phase_chat_suivante`, `decalage_bond`,
//! `decalage_chat`) qui contiennent toute la machine à états et la trajectoire ;
//! `Option<f32>` pour un bond qui peut ne pas avoir lieu ; un objet-trait
//! (`&dyn Grille`) pour lire les tuiles sans connaître le niveau ; des durées en
//! ENTIERS (pas de 1/60 s), exactement reproductibles.

use super::{Ennemi, EtatEnnemi, TypeEnnemi};
use crate::aabb::{touche_un_obstacle, Aabb, Grille, TAILLE_TUILE};
use crate::physics::{Environnement, GRAVITE, HZ};

/// Durée du sommeil : 100 pas, 1,7 s (4 s avant la carte 6.3). Juste le temps de
/// lui sauter dessus quand on l'a attendu hors de sa portée. Le coordinateur
/// proposait « environ 2 s » (120 pas) : avec 120, et aussi avec 110, la mesure de
/// difficulté officielle place le marché couvert SOUS les gouttières pour
/// l'Habitué, et le juge de la courbe (`game/src/difficulte.rs`) refuse ; 100 est
/// la valeur la plus proche (de 10 en 10) qui le garde vert (voir QA.md, carte 6.3).
pub const DORT_PAS: u16 = 100;
/// Durée de l'annonce du réveil : 0,8 s, comme l'annonce d'un éclair. Gardée
/// telle quelle : c'est elle qui rend le bond juste (on le voit venir).
pub const REVEIL_PAS: u16 = 48;
/// Durée de l'éveil : 1,2 s (« il se réveille brièvement »). Avec un bond, elle
/// contient l'aller, la pose et le retour (`BOND_PAS`, `POSE_BOND_PAS`).
pub const EVEILLE_PAS: u16 = 72;
/// Période complète du cycle : 3,7 s (220 pas).
pub const PERIODE_CHAT_PAS: u16 = DORT_PAS + REVEIL_PAS + EVEILLE_PAS;

/// Portée du bond en x (px), entre le centre du chat (à sa place) et celui de
/// Pépin : 5 tuiles.
pub const PORTEE_BOND_X: f32 = 5.0 * TAILLE_TUILE;
/// Portée du bond en hauteur (px), entre les pieds du chat et ceux de Pépin :
/// 2 tuiles, au-dessus comme au-dessous.
pub const PORTEE_BOND_Y: f32 = 2.0 * TAILLE_TUILE;
/// Durée d'un bond (l'aller, ou le retour) : 0,5 s.
pub const BOND_PAS: u16 = 30;
/// Pause, posé au bout du bond, avant le bond de retour : le reste de l'éveil
/// (0,2 s), pour que l'aller et le retour se lisent comme deux sauts.
pub const POSE_BOND_PAS: u16 = EVEILLE_PAS - 2 * BOND_PAS;
/// Vitesse verticale au départ d'un bond (px/s, vers le haut) : avec la gravité
/// de Pépin (`GRAVITE`), le chat retombe à sa hauteur de départ au bout de
/// `BOND_PAS` pile (225 px/s, un arc de 28 px de haut).
pub const VITESSE_BOND_Y: f32 = GRAVITE * BOND_PAS as f32 / HZ as f32 / 2.0;
/// Pas (px) entre deux longueurs de bond essayées par `cible_du_bond`, quand
/// l'arc complet vers Pépin toucherait un mur ou un plafond.
const PAS_RACCOURCI_BOND: f32 = 2.0;
/// Pas (px) entre deux points examinés sur la ligne de vue (`voit_pepin`).
const PAS_LIGNE_DE_VUE: f32 = 4.0;

/// Où en est le chat dans son cycle. Chaque variante porte le nombre de pas
/// qui lui restent (celui-ci compris) : `Dort(1)` est le dernier pas de sommeil.
/// Les autres ennemis gardent toujours `Dort(DORT_PAS)`, qui ne leur sert à rien.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseChat {
    /// Il dort (« zzz ») : écrasable.
    Dort(u16),
    /// Il se réveille (l'annonce) : encore écrasable.
    Reveil(u16),
    /// Il est réveillé (sur place, ou en plein bond) : intouchable, le toucher tue.
    Eveille(u16),
}

/// Phase suivante (un pas de 1/60 s). Fonction pure : toute la machine à états
/// tient dans ce `match`.
pub fn phase_chat_suivante(phase: PhaseChat) -> PhaseChat {
    match phase {
        PhaseChat::Dort(n) if n > 1 => PhaseChat::Dort(n - 1),
        PhaseChat::Dort(_) => PhaseChat::Reveil(REVEIL_PAS),
        PhaseChat::Reveil(n) if n > 1 => PhaseChat::Reveil(n - 1),
        PhaseChat::Reveil(_) => PhaseChat::Eveille(EVEILLE_PAS),
        PhaseChat::Eveille(n) if n > 1 => PhaseChat::Eveille(n - 1),
        PhaseChat::Eveille(_) => PhaseChat::Dort(DORT_PAS),
    }
}

/// Pas écoulés depuis le début du sommeil en cours : où en est le chat dans son
/// cycle, de 0 (il vient de s'endormir) à `PERIODE_CHAT_PAS - 1` (il va se
/// rendormir). Fonction pure : le robot de QA s'en sert pour savoir combien de
/// temps attendre (voir `game/src/bot_qa/parcours.rs`).
pub fn pas_du_cycle(phase: PhaseChat) -> u16 {
    match phase {
        PhaseChat::Dort(n) => DORT_PAS - n,
        PhaseChat::Reveil(n) => DORT_PAS + REVEIL_PAS - n,
        PhaseChat::Eveille(n) => PERIODE_CHAT_PAS - n,
    }
}

/// Décalage `(dx, dy)` du chat par rapport à sa place, `k` pas après le départ
/// d'un bond de longueur `longueur` (px, négative vers la gauche). En x, il
/// avance d'un même pas à chaque image ; en y, c'est la chute d'un objet lancé
/// vers le haut à `VITESSE_BOND_Y` avec la gravité de Pépin :
/// `y = g t² / 2 - v t` (y vers le bas). L'arc est symétrique, et au pas
/// `BOND_PAS` le chat est pile à la hauteur de départ. Fonction pure.
pub fn decalage_bond(k: u16, longueur: f32) -> (f32, f32) {
    let k = k.min(BOND_PAS);
    if k == BOND_PAS {
        return (longueur, 0.0); // exact : pas d'arrondi à l'arrivée
    }
    let t = f32::from(k) / HZ as f32;
    let dy = GRAVITE * t * t / 2.0 - VITESSE_BOND_Y * t;
    (longueur * f32::from(k) / f32::from(BOND_PAS), dy)
}

/// Décalage `(dx, dy)` du chat par rapport à sa place, selon sa phase et son
/// bond (`None` : pas de bond, il reste en place). Pendant l'éveil avec bond :
/// l'aller (`BOND_PAS`), la pose au bout (`POSE_BOND_PAS`), puis le retour, qui
/// est l'aller à l'envers ; au dernier pas de l'éveil il est revenu PILE à sa
/// place. Fonction pure (le rendu et les tests s'en servent aussi).
pub fn decalage_chat(phase: PhaseChat, bond: Option<f32>) -> (f32, f32) {
    match (phase, bond) {
        (PhaseChat::Eveille(n), Some(longueur)) => {
            // Pas écoulés dans l'éveil : 1 au premier pas, `EVEILLE_PAS` au dernier.
            let k = EVEILLE_PAS - n.min(EVEILLE_PAS) + 1;
            if k <= BOND_PAS {
                decalage_bond(k, longueur)
            } else if k <= BOND_PAS + POSE_BOND_PAS {
                (longueur, 0.0)
            } else {
                let (dx, dy) = decalage_bond(k - BOND_PAS - POSE_BOND_PAS, -longueur);
                (longueur + dx, dy)
            }
        }
        _ => (0.0, 0.0),
    }
}

impl Ennemi {
    /// Pose un chat endormi dont le coin haut-gauche est en `(x, y)`. Son premier
    /// sommeil est raccourci de `decalage_pas` (borné : il dort au moins 1 pas) :
    /// deux chats d'un même niveau ne se réveillent pas en même temps.
    pub fn chat(x: f32, y: f32, decalage_pas: u16) -> Self {
        let mut c = Ennemi::new(TypeEnnemi::Chat, x, y);
        c.sommeil = PhaseChat::Dort(DORT_PAS - decalage_pas.min(DORT_PAS - 1));
        c
    }

    /// Phase du cycle du chat (toujours `Dort(DORT_PAS)` pour un autre ennemi).
    pub fn phase_chat(&self) -> PhaseChat {
        self.sommeil
    }

    /// Le chat est-il réveillé (intouchable) ? Toujours faux pour un autre ennemi.
    pub fn est_eveille(&self) -> bool {
        self.type_ennemi == TypeEnnemi::Chat && matches!(self.sommeil, PhaseChat::Eveille(_))
    }

    /// Avancement de l'annonce du réveil entre 0.0 (début) et 1.0 (réveil
    /// imminent), pour faire clignoter plus vite à l'écran (comme l'éclair).
    /// 0.0 hors de l'annonce.
    pub fn progression_reveil(&self) -> f32 {
        match self.sommeil {
            PhaseChat::Reveil(n) => 1.0 - f32::from(n) / f32::from(REVEIL_PAS),
            _ => 0.0,
        }
    }

    /// Peut-on l'écraser en lui tombant dessus ? Un ennemi actif, sauf un chat
    /// réveillé (sur place ou en plein bond).
    pub fn est_ecrasable(&self) -> bool {
        self.etat == EtatEnnemi::Actif && !self.est_eveille()
    }

    /// Pendant l'annonce, le chat voit-il Pépin à portée de bond ? (La posture
    /// « prêt à bondir » à l'écran.) Toujours faux hors de l'annonce.
    pub fn est_pret_a_bondir(&self) -> bool {
        self.pret
    }

    /// Le bond en cours : sa longueur en px (négative vers la gauche), ou `None`
    /// s'il n'y en a pas (endormi, en annonce, ou éveillé sur place).
    pub fn bond(&self) -> Option<f32> {
        self.bond
    }

    /// Le chat est-il en l'air, en plein bond (l'aller ou le retour) ? Faux
    /// quand il est posé, même pendant un bond (au bout de l'aller). Pour l'image
    /// à l'écran : pattes tendues en l'air, debout une fois posé.
    pub fn en_plein_bond(&self) -> bool {
        decalage_chat(self.sommeil, self.bond).1 < 0.0
    }

    /// Coin haut-gauche de la place du chat (là où il dort, et où il revient).
    pub fn place(&self) -> (f32, f32) {
        (self.origine_x, self.origine_y)
    }

    /// De quel côté le chat regarde-t-il à l'écran ? Vers Pépin quand il est prêt
    /// à bondir, dans le sens du saut pendant un bond (l'aller, la pose, puis le
    /// retour vers sa place), sinon de son côté habituel (`vers_la_droite`).
    pub fn regarde_a_droite(&self) -> bool {
        if let (PhaseChat::Eveille(n), Some(longueur)) = (self.sommeil, self.bond) {
            let retour = EVEILLE_PAS - n.min(EVEILLE_PAS) + 1 > BOND_PAS + POSE_BOND_PAS;
            return (longueur > 0.0) != retour;
        }
        match (self.pret, self.proie) {
            (true, Some(j)) => j.centre().0 > self.origine_x + self.boite.w / 2.0,
            _ => self.vers_la_droite,
        }
    }

    /// Le chat, couché à sa place, voit-il Pépin (sa boîte `joueur`) à portée de
    /// bond ? Il faut :
    /// - au plus `PORTEE_BOND_X` entre leurs centres, en x ;
    /// - au plus `PORTEE_BOND_Y` entre leurs pieds, en hauteur ;
    /// - une ligne de vue libre : le segment entre leurs centres ne traverse
    ///   aucune tuile solide de `grille`, ni aucune boîte de `solides` (un point
    ///   tous les `PAS_LIGNE_DE_VUE` px).
    ///
    /// Fonction pure (lecture seule), publique : le robot de faisabilité s'en sert
    /// pour savoir où il ne faut pas s'attarder (`crate::faisabilite`).
    pub fn voit_pepin(&self, joueur: &Aabb, grille: &dyn Grille, solides: &[Aabb]) -> bool {
        let (cx, cy) = (
            self.origine_x + self.boite.w / 2.0,
            self.origine_y + self.boite.h / 2.0,
        );
        let (jx, jy) = joueur.centre();
        let pieds = self.origine_y + self.boite.h;
        if (jx - cx).abs() > PORTEE_BOND_X || (joueur.bas() - pieds).abs() > PORTEE_BOND_Y {
            return false;
        }
        let longueur = (jx - cx).hypot(jy - cy);
        let points = (longueur / PAS_LIGNE_DE_VUE).ceil().max(1.0) as u32;
        (0..=points).all(|i| {
            let f = i as f32 / points as f32;
            let (x, y) = (cx + (jx - cx) * f, cy + (jy - cy) * f);
            let tuile = (
                (x / TAILLE_TUILE).floor() as i32,
                (y / TAILLE_TUILE).floor() as i32,
            );
            !grille.est_solide(tuile.0, tuile.1)
                && !solides
                    .iter()
                    .any(|s| x > s.x && x < s.droite() && y > s.y && y < s.bas())
        })
    }

    /// Longueur (px, négative vers la gauche) du bond vers Pépin (`joueur`), ou
    /// `None` si aucun bond n'est possible. Le chat vise l'endroit où se tient
    /// Pépin (son centre sous le centre de Pépin), puis raccourcit le bond :
    /// 1. jamais hors de son toit : toutes les colonnes de tuiles survolées
    ///    doivent avoir un solide sous les pieds du chat (il s'arrête au bord du
    ///    vide, il n'en saute jamais un) ;
    /// 2. jamais à travers un mur ou un plafond : à chaque pas de l'arc, sa boîte
    ///    ne touche aucun obstacle (tuile solide ou boîte de `solides`) ; sinon,
    ///    il essaie un bond plus court, de `PAS_RACCOURCI_BOND` en
    ///    `PAS_RACCOURCI_BOND` px, jusqu'au saut sur place (longueur 0).
    ///
    /// Le retour, l'aller à l'envers, passe par les mêmes boîtes : il est donc
    /// libre lui aussi. Fonction pure (lecture seule).
    pub fn cible_du_bond(
        &self,
        joueur: &Aabb,
        grille: &dyn Grille,
        solides: &[Aabb],
    ) -> Option<f32> {
        let (x0, y0, w, h) = (self.origine_x, self.origine_y, self.boite.w, self.boite.h);
        let voulu = (joueur.centre().0 - w / 2.0 - x0).clamp(-PORTEE_BOND_X, PORTEE_BOND_X);
        // 1. Le toit : la rangée de tuiles sous les pieds du chat.
        let rangee_sol = ((y0 + h + 0.5) / TAILLE_TUILE).floor() as i32;
        let colonne = |x: f32| (x / TAILLE_TUILE).floor() as i32;
        let sol_sous = |de: f32, a: f32| {
            (colonne(de)..=colonne(a - 0.01)).all(|c| grille.est_solide(c, rangee_sol))
        };
        let sens = if voulu < 0.0 { -1.0 } else { 1.0 };
        let mut longueur = voulu.abs();
        while longueur > 0.0
            && !sol_sous(
                x0.min(x0 + sens * longueur),
                x0.max(x0 + sens * longueur) + w,
            )
        {
            longueur = (longueur - 1.0).max(0.0);
        }
        // 2. L'arc : aucun obstacle, à aucun pas.
        loop {
            let arc_libre = (1..=BOND_PAS).all(|k| {
                let (dx, dy) = decalage_bond(k, sens * longueur);
                !touche_un_obstacle(&Aabb::new(x0 + dx, y0 + dy, w, h), grille, solides)
            });
            if arc_libre {
                return Some(sens * longueur);
            }
            if longueur <= 0.0 {
                return None;
            }
            longueur = (longueur - PAS_RACCOURCI_BOND).max(0.0);
        }
    }

    /// Le chat, s'il se réveillait maintenant, bondirait-il SUR Pépin (sa boîte
    /// `joueur`, immobile) ? Il faut qu'il le voie (`voit_pepin`), qu'un bond
    /// soit possible (`cible_du_bond`), et que sa boîte touche celle de Pépin à
    /// un pas au moins de l'éveil (l'aller, la pose ou le retour). Un Pépin vu
    /// mais hors d'atteinte (de l'autre côté d'un trou, sur une marche contre
    /// laquelle le chat s'arrête) ne risque rien.
    ///
    /// Fonction pure, publique : le robot de faisabilité ne s'arrête jamais là
    /// où un chat vivant l'atteindrait (`crate::faisabilite`), et les tests des
    /// niveaux l'utilisent pour les lanternes (`crate::zones`).
    pub fn bond_atteint(&self, joueur: &Aabb, grille: &dyn Grille, solides: &[Aabb]) -> bool {
        if !self.voit_pepin(joueur, grille, solides) {
            return false;
        }
        let Some(longueur) = self.cible_du_bond(joueur, grille, solides) else {
            return false;
        };
        (1..=EVEILLE_PAS).any(|n| {
            let (dx, dy) = decalage_chat(PhaseChat::Eveille(n), Some(longueur));
            let boite = Aabb::new(
                self.origine_x + dx,
                self.origine_y + dy,
                self.boite.w,
                self.boite.h,
            );
            boite.chevauche(joueur)
        })
    }

    /// Le chat regarde Pépin (appelé par `guetter`, à chaque pas, depuis
    /// `resoudre_contacts`) : il retient sa boîte pour le pas suivant.
    pub(super) fn guetter_chat(&mut self, joueur: &Aabb) {
        self.proie = Some(*joueur);
    }

    /// Un pas du chat : son cycle avance, et sa boîte suit sa trajectoire.
    /// - Pendant l'annonce : il regarde si Pépin (vu au pas précédent) est à
    ///   portée (`voit_pepin`) et s'il a un bond possible vers lui
    ///   (`cible_du_bond`) : c'est la posture « prêt à bondir ».
    /// - Au premier pas de l'éveil (l'annonce est finie) : même question ; si
    ///   oui, il bondit (le bond choisi ne changera plus jusqu'au retour).
    /// - Rendormi : plus de bond, il est à sa place.
    pub(super) fn dormir(&mut self, env: &Environnement<'_>) {
        let avant = self.sommeil;
        self.sommeil = phase_chat_suivante(avant);
        let bond_vers_pepin = |c: &Ennemi| match c.proie {
            Some(j) if c.voit_pepin(&j, env.grille, env.solides) => {
                c.cible_du_bond(&j, env.grille, env.solides)
            }
            _ => None,
        };
        match (avant, self.sommeil) {
            (PhaseChat::Reveil(_), PhaseChat::Eveille(_)) => {
                self.bond = bond_vers_pepin(self);
                self.pret = false;
            }
            (_, PhaseChat::Reveil(_)) => self.pret = bond_vers_pepin(self).is_some(),
            (_, PhaseChat::Dort(_)) => {
                self.bond = None;
                self.pret = false;
            }
            _ => {}
        }
        let (dx, dy) = decalage_chat(self.sommeil, self.bond);
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
    use crate::physics::Environnement;

    fn avancer(c: &mut Ennemi, pas: u32) {
        let g = GrilleTuiles::vide(8, 8);
        let env = Environnement::simple(&g);
        for _ in 0..pas {
            c.actualiser(&env);
        }
    }

    #[test]
    fn le_cycle_dure_ce_qu_annoncent_les_constantes() {
        assert_eq!(
            (DORT_PAS, REVEIL_PAS, EVEILLE_PAS, PERIODE_CHAT_PAS),
            (100, 48, 72, 220),
            "carte 6.3 : 1,7 s de sommeil au lieu de 4, annonce et éveil inchangés"
        );
        let mut c = Ennemi::chat(32.0, 40.0, 0);
        assert_eq!(c.phase_chat(), PhaseChat::Dort(DORT_PAS));
        // Compte des pas passés dans chaque phase sur trois périodes.
        let (mut dort, mut reveil, mut eveille) = (0u32, 0u32, 0u32);
        for _ in 0..3 * u32::from(PERIODE_CHAT_PAS) {
            match c.phase_chat() {
                PhaseChat::Dort(_) => dort += 1,
                PhaseChat::Reveil(_) => reveil += 1,
                PhaseChat::Eveille(_) => eveille += 1,
            }
            avancer(&mut c, 1);
        }
        assert_eq!((dort, reveil, eveille), (3 * 100, 3 * 48, 3 * 72));
        // Retour exact au point de départ après un nombre entier de périodes.
        assert_eq!(c.phase_chat(), PhaseChat::Dort(DORT_PAS));
    }

    #[test]
    fn l_ordre_des_phases_est_dort_reveil_eveille() {
        let mut c = Ennemi::chat(0.0, 0.0, 0);
        avancer(&mut c, u32::from(DORT_PAS) - 1);
        assert_eq!(c.phase_chat(), PhaseChat::Dort(1));
        avancer(&mut c, 1);
        assert_eq!(c.phase_chat(), PhaseChat::Reveil(REVEIL_PAS));
        avancer(&mut c, u32::from(REVEIL_PAS));
        assert_eq!(c.phase_chat(), PhaseChat::Eveille(EVEILLE_PAS));
        avancer(&mut c, u32::from(EVEILLE_PAS));
        assert_eq!(c.phase_chat(), PhaseChat::Dort(DORT_PAS));
    }

    /// `pas_du_cycle` compte les pas depuis l'endormissement : 0, 1, 2… jusqu'à
    /// `PERIODE_CHAT_PAS - 1`, puis retour à 0 ; un chat décalé part plus loin.
    #[test]
    fn le_pas_du_cycle_avance_d_un_a_chaque_pas() {
        let mut c = Ennemi::chat(0.0, 0.0, 0);
        for attendu in (0..PERIODE_CHAT_PAS).chain(0..3) {
            assert_eq!(
                pas_du_cycle(c.phase_chat()),
                attendu,
                "{:?}",
                c.phase_chat()
            );
            avancer(&mut c, 1);
        }
        assert_eq!(pas_du_cycle(PhaseChat::Reveil(REVEIL_PAS)), DORT_PAS);
        assert_eq!(
            pas_du_cycle(PhaseChat::Eveille(EVEILLE_PAS)),
            DORT_PAS + REVEIL_PAS
        );
        assert_eq!(pas_du_cycle(Ennemi::chat(0.0, 0.0, 40).phase_chat()), 40);
    }

    #[test]
    fn ecrasable_endormi_et_pendant_l_annonce_jamais_eveille() {
        let mut c = Ennemi::chat(0.0, 0.0, 0);
        for _ in 0..PERIODE_CHAT_PAS {
            let eveille = matches!(c.phase_chat(), PhaseChat::Eveille(_));
            assert_eq!(c.est_eveille(), eveille);
            assert_eq!(c.est_ecrasable(), !eveille, "{:?}", c.phase_chat());
            // Éveillé ou non, il reste dangereux au toucher (de côté).
            assert!(c.est_dangereux());
            avancer(&mut c, 1);
        }
    }

    #[test]
    fn sans_pepin_le_chat_ne_bouge_jamais() {
        let mut c = Ennemi::chat(48.0, 40.0, 0);
        for _ in 0..2 * PERIODE_CHAT_PAS {
            avancer(&mut c, 1);
            assert_eq!((c.boite.x, c.boite.y), (48.0, 40.0));
        }
    }

    #[test]
    fn le_decalage_raccourcit_le_premier_sommeil() {
        let c = Ennemi::chat(0.0, 0.0, 90);
        assert_eq!(c.phase_chat(), PhaseChat::Dort(DORT_PAS - 90));
        // Borné : un décalage trop grand laisse quand même 1 pas de sommeil.
        let c = Ennemi::chat(0.0, 0.0, 1000);
        assert_eq!(c.phase_chat(), PhaseChat::Dort(1));
    }

    #[test]
    fn la_progression_du_reveil_va_de_0_a_1() {
        let mut c = Ennemi::chat(0.0, 0.0, 0);
        assert_eq!(c.progression_reveil(), 0.0);
        avancer(&mut c, u32::from(DORT_PAS));
        assert!(c.progression_reveil() < 0.05);
        avancer(&mut c, u32::from(REVEIL_PAS) - 1);
        assert!(c.progression_reveil() > 0.95);
        avancer(&mut c, 1);
        assert_eq!(c.progression_reveil(), 0.0, "éveillé : plus d'annonce");
    }

    #[test]
    fn un_chat_ecrase_ne_se_reveille_plus_et_disparait() {
        let mut c = Ennemi::chat(0.0, 0.0, 0);
        c.ecraser();
        let phase = c.phase_chat();
        assert!(!c.est_ecrasable() && !c.est_dangereux());
        avancer(&mut c, u32::from(ANIMATION_ECRASE_PAS) - 1);
        assert_eq!(c.phase_chat(), phase, "écrasé, son cycle s'arrête");
        assert!(c.est_visible());
        avancer(&mut c, 1);
        assert!(!c.est_visible());
    }

    // --- Mini-niveau de test : le chat qui garde une goutte cachée ---

    /// Le chat (colonne 5) dort devant une niche basse (colonnes 6 à 12, sous un
    /// plafond de nuages) où attend une goutte (colonne 10). Le dessus du chat
    /// est libre : on peut lui sauter dessus. Par la niche, impossible de passer
    /// au-dessus de lui (la tête cognerait le plafond) : il faut l'écraser.
    const NICHE: &str = "
o.o...........
..............
..............
......NNNNNNN.
.@...z....o..X
##############
";

    /// Le mini-niveau, ses entités et Pépin, avancés pas à pas dans l'ordre du
    /// jeu (`Monde::avancer`) : entités, joueur, contacts.
    struct Banc {
        niveau: crate::level::Niveau,
        entites: Vec<crate::entities::Entite>,
        joueur: crate::player::Joueur,
    }

    impl Banc {
        /// Pépin en `(x, y)` (coin haut-gauche) ; le chat a déjà vécu `pas_chat` pas.
        fn new(x: f32, y: f32, pas_chat: u32) -> Self {
            Self::sur(NICHE, x, y, pas_chat)
        }

        /// Le même banc sur le mini-niveau `ascii` (un seul chat). Pendant les
        /// `pas_chat` premiers pas, le chat ne voit pas Pépin (personne ne le fait
        /// guetter) : il ne peut pas bondir avant le début du banc.
        fn sur(ascii: &str, x: f32, y: f32, pas_chat: u32) -> Self {
            let niveau = crate::level::Niveau::depuis_ascii(ascii).expect("mini-niveau valide");
            let entites = niveau.entites.clone();
            let mut banc = Self {
                niveau,
                entites,
                joueur: crate::player::Joueur::new(x, y),
            };
            let g = GrilleTuiles::vide(4, 4);
            let env = Environnement::simple(&g);
            for _ in 0..pas_chat {
                for e in &mut banc.entites {
                    e.actualiser(&env);
                }
            }
            banc
        }

        fn chat(&self) -> &Ennemi {
            self.entites
                .iter()
                .find_map(|e| match e {
                    crate::entities::Entite::Ennemi(c) => Some(c),
                    _ => None,
                })
                .expect("un chat dans le mini-niveau")
        }

        /// Un pas ; renvoie les contacts.
        fn pas(
            &mut self,
            entrees: &crate::player::Entrees,
        ) -> Vec<crate::entities::contacts::Contact> {
            let solides = crate::entities::solides_des_entites(&self.entites);
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
            crate::entities::contacts::resoudre_contacts(&mut self.joueur, &mut self.entites)
        }

        /// `n` pas avec `entrees` ; renvoie tous les contacts, dans l'ordre.
        fn jouer(
            &mut self,
            entrees: &crate::player::Entrees,
            n: u32,
        ) -> Vec<crate::entities::contacts::Contact> {
            (0..n).flat_map(|_| self.pas(entrees)).collect()
        }
    }

    fn droite() -> crate::player::Entrees {
        crate::player::Entrees {
            direction: crate::player::Direction::Droite,
            ..Default::default()
        }
    }

    fn mort(contacts: &[crate::entities::contacts::Contact]) -> bool {
        use crate::entities::contacts::{CauseMort, Contact};
        contacts.contains(&Contact::Mort(CauseMort::Ennemi))
    }

    #[test]
    fn la_niche_est_gardee_par_le_chat() {
        let b = Banc::new(24.0, 50.0, 0);
        let c = b.chat();
        assert_eq!((c.boite.x, c.boite.y), (81.0, 72.0));
        assert_eq!(c.phase_chat(), PhaseChat::Dort(DORT_PAS));
    }

    #[test]
    fn foncer_dans_le_chat_endormi_tue() {
        // Pépin part du départ et court vers la niche : il heurte le chat de côté.
        let mut b = Banc::new(24.0, 66.0, 0);
        let contacts = b.jouer(&droite(), 60);
        assert!(mort(&contacts), "{contacts:?}");
        assert!(b.chat().est_dangereux(), "le chat n'est pas écrasé");
    }

    #[test]
    fn tomber_sur_le_chat_endormi_l_ecrase_et_ouvre_la_niche() {
        use crate::entities::contacts::Contact;
        // Pépin lâché au-dessus du dos du chat endormi : il tombe dessus.
        let mut b = Banc::new(83.0, 30.0, 0);
        let contacts = b.jouer(&Default::default(), 40);
        assert!(contacts.contains(&Contact::Ecrase), "{contacts:?}");
        assert!(!mort(&contacts));
        assert!(!b.chat().est_dangereux());
        // Le chat disparu, Pépin entre dans la niche et prend la goutte cachée.
        b.jouer(&Default::default(), u32::from(ANIMATION_ECRASE_PAS));
        let contacts = b.jouer(&droite(), 60);
        assert!(contacts.contains(&Contact::Goutte(2)), "{contacts:?}");
        assert!(!mort(&contacts));
    }

    #[test]
    fn tomber_sur_le_chat_eveille_tue() {
        use crate::entities::contacts::Contact;
        // Même chute, mais le chat est réveillé (dernier pas de l'annonce passé).
        let reveil = u32::from(DORT_PAS + REVEIL_PAS);
        let mut b = Banc::new(83.0, 30.0, reveil - 10);
        let contacts = b.jouer(&Default::default(), 40);
        assert!(mort(&contacts), "{contacts:?}");
        assert!(!contacts.contains(&Contact::Ecrase));
        assert!(b.chat().est_dangereux(), "un chat éveillé ne s'écrase pas");
    }

    #[test]
    fn les_autres_ennemis_ne_sont_jamais_eveilles() {
        for t in [TypeEnnemi::Flaque, TypeEnnemi::Corbeau, TypeEnnemi::Pigeon] {
            let mut e = Ennemi::new(t, 32.0, 32.0);
            for _ in 0..PERIODE_CHAT_PAS {
                assert!(!e.est_eveille() && e.est_ecrasable(), "{t:?}");
                avancer(&mut e, 1);
            }
        }
    }

    // --- Le bond sur Pépin (carte 6.3) ---

    /// Toit plat (colonnes 0 à 13), un trou de 2 tuiles, puis un second toit à la
    /// même hauteur (16 à 21). Le chat dort en colonne 11 : boîte 14 x 8 en
    /// (177, 72), centre en x = 184, pieds en y = 80. Le bord de son toit est en
    /// x = 224. Pépin debout sur un toit : y = 66.
    const TOIT: &str = "
o.o.o.................
......................
......................
......................
.@.........z........X.
##############..######
";

    /// Le même chat, un mur (colonne 13, rangées 1 à 4) entre lui et la droite.
    const MUR: &str = "
o.o.o.................
.............#........
.............#........
.............#........
.@.........z.#......X.
######################
";

    /// Le même chat, une marche d'une tuile de haut (colonnes 15 à 18, dessus en
    /// y = 64, bord gauche en x = 240) devant lui.
    const MARCHE: &str = "
o.o.o.................
......................
......................
......................
.@.........z...BBBB.X.
######################
";

    /// Le même chat sous un plafond de nuages (colonnes 9 à 13, dessous en
    /// y = 48) : trop bas pour un bond (l'arc monte à 28 px au-dessus de lui).
    const PLAFOND: &str = "
o.o.o.................
......................
.........NNNNN........
......................
.@.........z........X.
######################
";

    /// Pépin immobile, quel que soit le pas.
    fn rien() -> crate::player::Entrees {
        Default::default()
    }

    /// Pas du chat juste avant la fin de l'annonce : `n` pas avant l'éveil.
    fn avant_l_eveil(n: u16) -> u32 {
        u32::from(DORT_PAS + REVEIL_PAS - n)
    }

    /// Le chat (et ses solides) du mini-niveau `ascii`, et le niveau.
    fn chat_de(ascii: &str) -> (crate::level::Niveau, Ennemi) {
        let n = crate::level::Niveau::depuis_ascii(ascii).expect("mini-niveau valide");
        let c = n
            .entites
            .iter()
            .find_map(|e| match e {
                crate::entities::Entite::Ennemi(c) => Some(c.clone()),
                _ => None,
            })
            .expect("un chat");
        (n, c)
    }

    /// Boîte de Pépin debout, coin haut-gauche en `(x, y)`.
    fn pepin(x: f32, y: f32) -> Aabb {
        Aabb::new(x, y, crate::physics::JOUEUR_L, crate::physics::JOUEUR_H)
    }

    #[test]
    fn la_trajectoire_du_bond_est_un_arc_qui_revient_pile_a_sa_place() {
        let mut sommet = 0.0f32;
        for k in 1..=EVEILLE_PAS {
            let n = EVEILLE_PAS - k + 1; // Eveille(n) : k-ième pas de l'éveil
            let (dx, dy) = decalage_chat(PhaseChat::Eveille(n), Some(48.0));
            assert!(dy <= 0.0, "jamais sous son toit : pas {k}, dy = {dy}");
            assert!((0.0..=48.0).contains(&dx), "pas {k}, dx = {dx}");
            sommet = sommet.min(dy);
            if k == BOND_PAS || k == BOND_PAS + POSE_BOND_PAS {
                assert_eq!((dx, dy), (48.0, 0.0), "posé au bout du bond (pas {k})");
            }
        }
        // Un arc de 28 px de haut, et le dernier pas de l'éveil le ramène PILE à sa place.
        assert!((sommet + 28.1).abs() < 0.2, "sommet de l'arc : {sommet}");
        assert_eq!(decalage_chat(PhaseChat::Eveille(1), Some(48.0)), (0.0, 0.0));
        // Sans bond, ou hors de l'éveil : à sa place.
        assert_eq!(decalage_chat(PhaseChat::Eveille(30), None), (0.0, 0.0));
        assert_eq!(decalage_chat(PhaseChat::Dort(5), Some(48.0)), (0.0, 0.0));
        assert_eq!(decalage_bond(BOND_PAS, -33.0), (-33.0, 0.0));
    }

    #[test]
    fn il_ne_voit_pepin_qu_a_portee_et_sans_mur_entre_eux() {
        let (n, c) = chat_de(TOIT);
        assert_eq!((c.boite.x, c.boite.y), (177.0, 72.0));
        let voit = |x: f32, y: f32| c.voit_pepin(&pepin(x, y), &n, &[]);
        // Sur son toit : jusqu'à 5 tuiles (80 px) entre les centres, des deux côtés.
        assert!(voit(210.0, 66.0) && voit(99.0, 66.0), "à portée");
        assert!(voit(259.0, 66.0), "centre à 80 px : encore à portée");
        assert!(
            !voit(260.0, 66.0) && !voit(90.0, 66.0),
            "au-delà de 5 tuiles"
        );
        // En hauteur : jusqu'à 2 tuiles (32 px) entre les pieds, pas plus.
        assert!(voit(200.0, 34.0), "pieds 32 px plus haut");
        assert!(
            !voit(200.0, 33.0) && !voit(200.0, 18.0),
            "plus de 2 tuiles plus haut"
        );
        // Derrière un mur : rien, même tout près.
        let (n, c) = chat_de(MUR);
        assert!(
            !c.voit_pepin(&pepin(227.0, 66.0), &n, &[]),
            "derrière le mur"
        );
        assert!(c.voit_pepin(&pepin(190.0, 66.0), &n, &[]), "devant le mur");
        // Une boîte solide (plateforme) entre eux bloque aussi la vue.
        let caisse = Aabb::new(200.0, 60.0, 8.0, 20.0);
        assert!(!c.voit_pepin(&pepin(212.0, 66.0), &n, &[caisse]));
    }

    #[test]
    fn a_portee_il_se_prepare_puis_bondit_et_tue_pepin() {
        let mut b = Banc::sur(TOIT, 210.0, 66.0, avant_l_eveil(5));
        b.jouer(&rien(), 3);
        assert!(matches!(b.chat().phase_chat(), PhaseChat::Reveil(_)));
        assert!(b.chat().est_pret_a_bondir(), "posture « prêt à bondir »");
        assert!(
            b.chat().regarde_a_droite(),
            "il regarde Pépin (à sa droite)"
        );
        b.jouer(&rien(), 2);
        assert_eq!(b.chat().phase_chat(), PhaseChat::Eveille(EVEILLE_PAS));
        // Il vise l'endroit où se tient Pépin : son centre sous celui de Pépin.
        assert_eq!(b.chat().bond(), Some(31.0));
        assert!(!b.chat().est_pret_a_bondir());
        let mut tue_a = None;
        for k in 1..EVEILLE_PAS {
            assert!(
                !b.chat().est_ecrasable(),
                "en plein bond, il ne s'écrase pas"
            );
            if mort(&b.pas(&rien())) && tue_a.is_none() {
                tue_a = Some(k);
            }
        }
        let tue_a = tue_a.expect("le chat qui bondit tue Pépin");
        assert!(tue_a <= BOND_PAS + 1, "dès l'aller : pas {tue_a}");
    }

    #[test]
    fn hors_de_portee_il_se_reveille_sur_place() {
        for x in [40.0, 270.0] {
            let mut b = Banc::sur(TOIT, x, 66.0, 0);
            for _ in 0..2 * PERIODE_CHAT_PAS {
                let contacts = b.pas(&rien());
                let c = b.chat();
                assert!(!mort(&contacts), "Pépin en x = {x}");
                assert_eq!((c.boite.x, c.boite.y), (177.0, 72.0), "Pépin en x = {x}");
                assert!(c.bond().is_none() && !c.est_pret_a_bondir());
            }
        }
    }

    /// Vérifie pas à pas tout l'éveil d'un chat qui bondit : sa boîte ne touche
    /// jamais un solide, et il ne quitte jamais son toit (une tuile solide sous
    /// chaque colonne qu'il couvre, rangée 5). Renvoie la longueur du bond.
    fn bond_sans_chute_ni_mur(ascii: &str, x: f32, y: f32) -> Option<f32> {
        let mut b = Banc::sur(ascii, x, y, avant_l_eveil(3));
        b.jouer(&rien(), 3);
        assert!(b.chat().est_eveille());
        let bond = b.chat().bond();
        for _ in 0..EVEILLE_PAS {
            let c = b.chat().boite;
            assert!(
                !touche_un_obstacle(&c, &b.niveau, &[]),
                "dans un solide : {c:?}"
            );
            let (g, d) = ((c.x / 16.0) as i32, ((c.droite() - 0.01) / 16.0) as i32);
            assert!(
                (g..=d).all(|col| b.niveau.est_solide(col, 5)),
                "au-dessus du vide : {c:?}"
            );
            b.pas(&rien());
        }
        let c = b.chat();
        assert_eq!(
            (c.boite.x, c.boite.y),
            (177.0, 72.0),
            "revenu PILE à sa place"
        );
        assert_eq!(c.phase_chat(), PhaseChat::Dort(DORT_PAS), "et rendormi");
        assert!(c.bond().is_none() && c.est_ecrasable());
        bond
    }

    #[test]
    fn le_bond_s_arrete_au_bord_du_toit() {
        // Pépin de l'autre côté du trou, à portée : le chat bondit, mais s'arrête
        // au bord (x = 224) au lieu de sauter le trou.
        assert_eq!(bond_sans_chute_ni_mur(TOIT, 257.0, 66.0), Some(33.0));
    }

    #[test]
    fn le_bond_ne_traverse_pas_un_mur() {
        // Pépin debout sur la marche : il est vu (par-dessus le coin), mais le
        // chat s'arrête contre la marche (bord gauche en x = 240) au lieu d'y entrer.
        let (n, c) = chat_de(MARCHE);
        assert!(c.voit_pepin(&pepin(243.0, 50.0), &n, &[]));
        assert_eq!(bond_sans_chute_ni_mur(MARCHE, 243.0, 50.0), Some(48.0));
        // Un plafond trop bas : aucun bond possible, il ne se prépare même pas.
        let mut b = Banc::sur(PLAFOND, 200.0, 66.0, avant_l_eveil(10));
        for _ in 0..u32::from(10 + EVEILLE_PAS) {
            b.pas(&rien());
            let c = b.chat();
            assert!(!c.est_pret_a_bondir() && c.bond().is_none());
            assert_eq!((c.boite.x, c.boite.y), (177.0, 72.0));
        }
    }

    /// Partout où Pépin peut se tenir (au pixel près, sur les deux mini-niveaux à
    /// obstacles), le bond choisi ne traverse rien et ne quitte pas le toit.
    #[test]
    fn aucun_bond_ne_traverse_un_solide_ni_ne_saute_dans_le_vide() {
        for (ascii, y) in [(TOIT, 66.0), (MARCHE, 66.0), (MARCHE, 50.0), (MUR, 66.0)] {
            let (n, c) = chat_de(ascii);
            for x in (16..330).map(|x| x as f32) {
                let j = pepin(x, y);
                if touche_un_obstacle(&j, &n, &[]) {
                    continue;
                }
                let Some(longueur) = c.cible_du_bond(&j, &n, &[]) else {
                    continue;
                };
                assert!(longueur.abs() <= PORTEE_BOND_X);
                for k in 1..=BOND_PAS {
                    let (dx, dy) = decalage_bond(k, longueur);
                    let boite = Aabb::new(177.0 + dx, 72.0 + dy, 14.0, 8.0);
                    assert!(!touche_un_obstacle(&boite, &n, &[]), "x = {x}, pas {k}");
                    let d = ((boite.droite() - 0.01) / 16.0) as i32;
                    assert!(((boite.x / 16.0) as i32..=d).all(|col| n.est_solide(col, 5)));
                }
            }
        }
    }

    #[test]
    fn en_plein_bond_il_tue_par_le_dessus_rendormi_il_s_ecrase() {
        use crate::entities::contacts::{resoudre_contacts, CauseMort, Contact};
        use crate::entities::Entite;
        let (n, mut c) = chat_de(TOIT);
        let env = Environnement::simple(&n);
        // Pépin de l'autre côté du trou : le chat bondit jusqu'au bord et s'y pose.
        let loin = pepin(257.0, 66.0);
        for _ in 0..avant_l_eveil(0) + u32::from(BOND_PAS) + 1 {
            c.guetter(&loin);
            c.actualiser(&env);
        }
        assert_eq!(
            (c.boite.x, c.bond()),
            (210.0, Some(33.0)),
            "posé au bout du bond"
        );
        // Un Pépin qui lui tombe sur le dos : tué, pas d'écrasement.
        let sur_le_dos = |c: &Ennemi| {
            let mut j = crate::player::Joueur::new(c.boite.x + 2.0, c.boite.y - 12.0);
            j.vitesse.y = 120.0;
            j
        };
        let mut ents = vec![Entite::Ennemi(c.clone())];
        let contacts = resoudre_contacts(&mut sur_le_dos(&c), &mut ents);
        assert_eq!(contacts, vec![Contact::Mort(CauseMort::Ennemi)]);
        // Rendormi, revenu à sa place : la même chute l'écrase.
        while c.est_eveille() {
            c.actualiser(&env);
        }
        assert_eq!(c.boite.x, 177.0);
        let mut ents = vec![Entite::Ennemi(c.clone())];
        let contacts = resoudre_contacts(&mut sur_le_dos(&c), &mut ents);
        assert_eq!(contacts, vec![Contact::Ecrase]);
    }
}
