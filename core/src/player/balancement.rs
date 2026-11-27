//! Le balancement au parapluie (carte 11.1, PROTOTYPE du Monde 3) : en l'air,
//! SAUT tenu près d'un CROCHET, la crosse du parapluie s'y accroche au lieu de
//! planer. Pépin pend alors au bout d'une corde, comme un pendule ; GAUCHE et
//! DROITE lui donnent de l'élan ; lâcher SAUT le lance avec la vitesse du
//! balancement, augmentée de 10 %. Aucun bouton nouveau.
//!
//! Les règles de départ d'AGA sont des constantes nommées, à régler en jouant :
//! portée de l'accroche (`PORTEE_CROCHET`), corde bornée (`CORDE_MIN`,
//! `CORDE_MAX`), angle maximal (`ANGLE_MAX`), bonus du lâcher (`BONUS_LACHER`),
//! délai avant de se raccrocher au même crochet (`DELAI_RACCROCHE_PAS`).
//!
//! Le pendule : un angle `theta` (0 = à la verticale sous le crochet, positif
//! vers la droite) et une vitesse angulaire `omega` (rad/s). À chaque pas, la
//! pesanteur le ramène vers le bas (`-g/R sin theta`), l'élan du joueur pousse
//! le long du cercle (`POUSSEE_BALANCEMENT`), puis on avance avec la méthode
//! d'Euler « semi-implicite » : la vitesse d'abord, l'angle ensuite avec la
//! NOUVELLE vitesse. Cet ordre-là (et non l'inverse) ne fabrique pas d'énergie :
//! sans appui, le balancement ne monte jamais plus haut qu'au départ. Un léger
//! frottement (`FROTTEMENT_BALANCEMENT`) le fait même lentement retomber.
//!
//! Hors du prototype (documenté pour rester honnête) : pendant le balancement,
//! ni courant ascendant ni vent ne poussent Pépin.
//!
//! Notions Rust : sous-module d'un fichier (`player.rs` + `player/`), qui voit
//! les champs PRIVÉS de `Joueur` (un module enfant voit tout ce qui est dans ses
//! parents) ; `Option<T>` pour un état qui n'existe que pendant le balancement ;
//! `f32::atan2`, `sin`, `cos`, `hypot`.

use super::{Direction, Entrees, Evenements, Joueur};
use crate::aabb::{deplacer, Aabb};
use crate::physics::{eau_touchee, Environnement, DT, GRAVITE, JOUEUR_L, VITESSE_COURSE};

/// Portée de l'accroche : 2,5 cases (40 px), du crochet à la crosse.
pub const PORTEE_CROCHET: f32 = 2.5 * 16.0;
/// Corde la plus courte : 1,5 case. Accroché plus près, Pépin glisse au bout.
pub const CORDE_MIN: f32 = 1.5 * 16.0;
/// Corde la plus longue : 3 cases.
pub const CORDE_MAX: f32 = 3.0 * 16.0;
/// Angle maximal du balancement : 80° de chaque côté de la verticale. Arrivé
/// là, Pépin s'arrête net et redescend.
pub const ANGLE_MAX: f32 = 80.0 * std::f32::consts::PI / 180.0;
/// Accélération (px/s², le long du cercle) que donnent GAUCHE et DROITE.
pub const POUSSEE_BALANCEMENT: f32 = 250.0;
/// Frottement de l'air sur le balancement (par seconde) : la vitesse angulaire
/// perd 2 % par seconde. Assez faible pour ne pas se sentir, assez pour que
/// le pendule laissé à lui-même retombe.
pub const FROTTEMENT_BALANCEMENT: f32 = 0.02;
/// Au lâcher, la vitesse du balancement est augmentée de 10 %.
pub const BONUS_LACHER: f32 = 1.10;
/// Délai (pas) avant de pouvoir se raccrocher au même crochet : 0,3 s.
pub const DELAI_RACCROCHE_PAS: u8 = 18;
/// La crosse (là où s'attache la corde) est 8 px au-dessus de la tête de
/// Pépin : il tient son parapluie fermé à bout de bras, crosse en haut.
pub const HAUTEUR_CROSSE: f32 = 8.0;

/// Un crochet du niveau (lettre `y` de la grille) : le point où s'accroche la
/// crosse, au milieu de sa case.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Crochet {
    pub x: f32,
    pub y: f32,
}

/// L'état du balancement : à quel crochet, la longueur de la corde, l'angle et
/// la vitesse angulaire.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Balancement {
    /// Indice du crochet dans la liste du niveau.
    pub crochet: usize,
    /// Le point du crochet (px).
    pub point: (f32, f32),
    /// Longueur de la corde (px), fixée à l'accroche.
    pub corde: f32,
    /// Angle (rad) : 0 = à la verticale sous le crochet, positif vers la droite.
    pub theta: f32,
    /// Vitesse angulaire (rad/s), positive vers la droite (en bas du cercle).
    pub omega: f32,
}

impl Balancement {
    /// La crosse au bout de la corde (px).
    pub fn crosse(&self) -> (f32, f32) {
        (
            self.point.0 + self.corde * self.theta.sin(),
            self.point.1 + self.corde * self.theta.cos(),
        )
    }

    /// Vitesse (px/s) de la crosse le long du cercle : `R omega` sur la
    /// tangente `(cos theta, -sin theta)`.
    pub fn vitesse(&self) -> (f32, f32) {
        let v = self.corde * self.omega;
        (v * self.theta.cos(), -v * self.theta.sin())
    }

    /// Énergie du pendule (par unité de masse) : cinétique plus potentielle,
    /// le zéro en bas du cercle. Pour les tests.
    pub fn energie(&self) -> f32 {
        let v = self.corde * self.omega;
        0.5 * v * v + GRAVITE * self.corde * (1.0 - self.theta.cos())
    }
}

impl Joueur {
    /// La crosse du parapluie, là où s'attache la corde : au-dessus du milieu
    /// de la tête (`HAUTEUR_CROSSE`).
    pub fn crosse(&self) -> (f32, f32) {
        (self.boite.x + JOUEUR_L / 2.0, self.boite.y - HAUTEUR_CROSSE)
    }

    /// Le balancement en cours (`None` : Pépin n'est pas suspendu).
    pub fn balancement(&self) -> Option<Balancement> {
        self.balancement
    }

    /// Le crochet le plus proche à PORTÉE de la crosse (2,5 cases au plus) et
    /// PLUS HAUT qu'elle, sans tenir compte du délai de raccroche : celui que
    /// le jeu allume.
    pub fn crochet_a_portee(&self, crochets: &[Crochet]) -> Option<usize> {
        self.crochet_proche(crochets, None)
    }

    /// Comme `crochet_a_portee`, sans le crochet `exclu` (celui qu'on vient de
    /// quitter, pendant le délai de raccroche).
    fn crochet_proche(&self, crochets: &[Crochet], exclu: Option<usize>) -> Option<usize> {
        let (ax, ay) = self.crosse();
        crochets
            .iter()
            .enumerate()
            .filter(|&(i, c)| c.y < ay && Some(i) != exclu)
            .map(|(i, c)| (i, (ax - c.x).hypot(ay - c.y)))
            .filter(|&(_, d)| d <= PORTEE_CROCHET)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)
    }

    /// Un pas de simulation AVEC les crochets du niveau. Sans crochet (et sans
    /// balancement en cours), c'est exactement `actualiser`, au bit près : les
    /// niveaux d'avant ne changent pas.
    pub fn actualiser_avec_crochets(
        &mut self,
        entrees: &Entrees,
        env: &Environnement<'_>,
        crochets: &[Crochet],
    ) -> Evenements {
        if crochets.is_empty() && self.balancement.is_none() {
            return self.actualiser(entrees, env);
        }
        self.delai_crochet = self.delai_crochet.saturating_sub(1);
        if let Some(b) = self.balancement {
            return self.balancer(b, entrees, env);
        }
        let mut ev = self.actualiser(entrees, env);
        // « La crosse s'accroche au lieu de planer » : au moment exact où le
        // parapluie s'ouvrirait (en l'air, SAUT tenu, en descente), un crochet
        // à portée le prend.
        if self.plane && !self.en_barque && !self.noyade {
            let exclu = if self.delai_crochet > 0 {
                self.crochet_quitte
            } else {
                None
            };
            if let Some(i) = self.crochet_proche(crochets, exclu) {
                if self.accrocher(crochets, i, env) {
                    ev.debut_plane = false;
                }
            }
        }
        ev
    }

    /// Accroche la crosse au crochet `i`, depuis la position et la vitesse
    /// actuelles. La corde est la distance du moment, bornée entre `CORDE_MIN`
    /// et `CORDE_MAX` ; la vitesse de Pépin est gardée le long du cercle
    /// (projetée sur la tangente). Faux si la place au bout de la corde est
    /// prise par un solide : pas d'accroche.
    fn accrocher(&mut self, crochets: &[Crochet], i: usize, env: &Environnement<'_>) -> bool {
        let c = crochets[i];
        let (ax, ay) = self.crosse();
        let corde = (ax - c.x).hypot(ay - c.y).clamp(CORDE_MIN, CORDE_MAX);
        let theta = (ax - c.x).atan2(ay - c.y).clamp(-ANGLE_MAX, ANGLE_MAX);
        let (vx, vy) = (self.vitesse.x + self.elan_auvent, self.vitesse.y);
        let omega = (vx * theta.cos() - vy * theta.sin()) / corde;
        let b = Balancement {
            crochet: i,
            point: (c.x, c.y),
            corde,
            theta,
            omega,
        };
        self.suspendre_a(b, env)
    }

    /// Met Pépin AU BOUT DE LA CORDE dans l'état `b` (les robots le
    /// reconstituent depuis un état ; le jeu l'obtient par `accrocher`). Faux,
    /// et rien ne change, si un solide occupe la place.
    pub fn suspendre_a(&mut self, b: Balancement, env: &Environnement<'_>) -> bool {
        let (cx, cy) = b.crosse();
        let cible = Aabb::new(
            cx - JOUEUR_L / 2.0,
            cy + HAUTEUR_CROSSE,
            self.boite.w,
            self.boite.h,
        );
        if crate::aabb::touche_un_obstacle(&cible, env.grille, env.solides) {
            return false;
        }
        self.boite = cible;
        let (vx, vy) = b.vitesse();
        self.vitesse.x = vx;
        self.vitesse.y = vy;
        self.balancement = Some(b);
        self.au_sol = false;
        self.plane = false;
        self.retourne = false;
        self.bateau_herite = false;
        self.monte_par_saut = false;
        self.elan_auvent = 0.0;
        self.coyote = 0;
        self.tampon = 0;
        true
    }

    /// Fin du balancement (lâcher ou décrochage) : le même crochet est
    /// interdit pendant `DELAI_RACCROCHE_PAS`.
    pub(super) fn quitter_le_crochet(&mut self) {
        if let Some(b) = self.balancement.take() {
            self.crochet_quitte = Some(b.crochet);
            self.delai_crochet = DELAI_RACCROCHE_PAS;
        }
    }

    /// Un pas suspendu au crochet. Le pas finit toujours ici : lâché ou
    /// décroché, Pépin reprend la physique normale au pas suivant.
    fn balancer(
        &mut self,
        mut b: Balancement,
        entrees: &Entrees,
        env: &Environnement<'_>,
    ) -> Evenements {
        self.protection = self.protection.saturating_sub(1);
        self.saut_precedent = entrees.saut;

        // --- SAUT lâché : Pépin part avec la vitesse du balancement + 10 % ---
        if !entrees.saut {
            let (vx, vy) = b.vitesse();
            let (vx, vy) = (vx * BONUS_LACHER, vy * BONUS_LACHER);
            self.quitter_le_crochet();
            // La part horizontale au-delà de la course passe dans l'élan (comme
            // celui des auvents) : les commandes en l'air s'y AJOUTENT au lieu
            // de le freiner, et il s'éteint en ligne droite.
            self.vitesse.x = vx.clamp(-VITESSE_COURSE, VITESSE_COURSE);
            self.elan_auvent = vx - self.vitesse.x;
            self.vitesse.y = vy;
            if vx != 0.0 {
                self.regarde_droite = vx > 0.0;
            }
            let contacts = deplacer(&mut self.boite, vx * DT, vy * DT, env.grille, env.solides);
            if contacts.mur() {
                self.vitesse.x = 0.0;
                self.elan_auvent = 0.0;
            }
            if contacts.plafond || contacts.sol {
                self.vitesse.y = 0.0;
            }
            return Evenements::default();
        }

        // --- Le pendule : vitesse d'abord, angle ensuite (Euler semi-implicite) ---
        let poussee = entrees.direction.signe() * POUSSEE_BALANCEMENT;
        let alpha = -GRAVITE / b.corde * b.theta.sin() + poussee / b.corde;
        b.omega = (b.omega + alpha * DT) * (1.0 - FROTTEMENT_BALANCEMENT * DT);
        b.theta += b.omega * DT;
        if b.theta.abs() > ANGLE_MAX {
            // Angle maximal atteint : arrêt net, la pesanteur le ramène.
            b.theta = b.theta.clamp(-ANGLE_MAX, ANGLE_MAX);
            b.omega = 0.0;
        }
        if entrees.direction != Direction::Immobile {
            self.regarde_droite = entrees.direction == Direction::Droite;
        }

        // --- Déplacement : un solide touché décroche Pépin ---
        let (cx, cy) = b.crosse();
        let (dx, dy) = (
            cx - JOUEUR_L / 2.0 - self.boite.x,
            cy + HAUTEUR_CROSSE - self.boite.y,
        );
        let contacts = deplacer(&mut self.boite, dx, dy, env.grille, env.solides);
        let (vx, vy) = b.vitesse();
        self.vitesse.x = vx;
        self.vitesse.y = vy;
        let touche = contacts.mur() || contacts.sol || contacts.plafond;
        let mouille =
            eau_touchee(&self.boite, env.eaux).is_some_and(|e| self.boite.bas() > e.surface());
        if touche || mouille {
            // Décroché : il garde sa vitesse le long du cercle (sans bonus) ;
            // au pas suivant, la physique normale reprend (chute, atterrissage,
            // eau : la noyade ou la barque, comme d'habitude).
            self.quitter_le_crochet();
            if contacts.mur() {
                self.vitesse.x = 0.0;
            }
            if contacts.sol || contacts.plafond {
                self.vitesse.y = 0.0;
            }
            self.au_sol = contacts.sol;
            return Evenements {
                atterrissage: contacts.sol,
                ..Evenements::default()
            };
        }
        self.balancement = Some(b);
        Evenements::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::GrilleTuiles;
    use crate::physics::JOUEUR_H;
    use crate::player::EtatJoueur;

    /// Salle 20 × 12 tuiles, sol en bas (haut du sol : y = 176) ; `mur` ajoute
    /// un mur plein à la colonne 12 (x de 192 à 208).
    fn salle(mur: bool) -> GrilleTuiles {
        let mut lignes = vec!["#..................#".to_string(); 12];
        lignes[0] = "####################".into();
        lignes[11] = "####################".into();
        if mur {
            for l in lignes.iter_mut().take(11).skip(1) {
                l.replace_range(12..13, "#");
            }
        }
        let refs: Vec<&str> = lignes.iter().map(String::as_str).collect();
        GrilleTuiles::depuis_texte(&refs)
    }

    /// Le crochet des tests : (160, 40).
    const CROCHET: Crochet = Crochet { x: 160.0, y: 40.0 };

    /// Pépin en l'air, la crosse en `(cx, cy)`, immobile.
    fn pepin_crosse_en(cx: f32, cy: f32) -> Joueur {
        Joueur::new(cx - JOUEUR_L / 2.0, cy + HAUTEUR_CROSSE)
    }

    fn saut_tenu() -> Entrees {
        Entrees {
            saut: true,
            ..Entrees::default()
        }
    }

    /// Pépin suspendu, corde de 32 px, à l'angle `theta` (degrés), immobile.
    fn suspendu(grille: &GrilleTuiles, theta_deg: f32) -> Joueur {
        let mut j = pepin_crosse_en(160.0, 72.0);
        let b = Balancement {
            crochet: 0,
            point: (CROCHET.x, CROCHET.y),
            corde: 32.0,
            theta: theta_deg.to_radians(),
            omega: 0.0,
        };
        assert!(j.suspendre_a(b, &Environnement::simple(grille)));
        j
    }

    #[test]
    fn en_l_air_saut_tenu_pres_d_un_crochet_la_crosse_s_accroche() {
        let g = salle(false);
        let env = Environnement::simple(&g);
        // La crosse 2 cases sous le crochet : à portée.
        let mut j = pepin_crosse_en(160.0, 72.0);
        j.actualiser_avec_crochets(&saut_tenu(), &env, &[CROCHET]);
        assert_eq!(j.etat(), EtatJoueur::Suspendu);
        let b = j.balancement().unwrap();
        // La corde : la distance du moment (32 px, plus un pas de chute).
        assert!((32.0..34.0).contains(&b.corde), "corde = {}", b.corde);
        assert!(!j.plane, "accroché au lieu de planer");
        // La crosse est bien au bout de la corde.
        let (cx, cy) = j.crosse();
        let (bx, by) = b.crosse();
        assert!((cx - bx).abs() < 1e-3 && (cy - by).abs() < 1e-3);
    }

    #[test]
    fn pas_d_accroche_sans_saut_trop_loin_plus_bas_ou_au_sol() {
        let g = salle(false);
        let env = Environnement::simple(&g);
        // SAUT non tenu : il tombe.
        let mut j = pepin_crosse_en(160.0, 72.0);
        j.actualiser_avec_crochets(&Entrees::default(), &env, &[CROCHET]);
        assert!(j.balancement().is_none());
        // Trop loin : 2,6 cases.
        let mut j = pepin_crosse_en(160.0, 40.0 + 2.6 * 16.0);
        j.actualiser_avec_crochets(&saut_tenu(), &env, &[CROCHET]);
        assert!(j.balancement().is_none());
        assert!(j.plane, "il plane, comme sans crochet");
        // Juste à portée (2,4 cases, avec le pas de chute) : accroché.
        let mut j = pepin_crosse_en(160.0, 40.0 + 2.4 * 16.0);
        j.actualiser_avec_crochets(&saut_tenu(), &env, &[CROCHET]);
        assert!(j.balancement().is_some());
        // Crochet PLUS BAS que la crosse : rien.
        let bas = Crochet { x: 160.0, y: 100.0 };
        let mut j = pepin_crosse_en(160.0, 90.0);
        j.actualiser_avec_crochets(&saut_tenu(), &env, &[bas]);
        assert!(j.balancement().is_none());
        // Au sol, SAUT tenu : c'est un saut, pas une accroche (le crochet est
        // à portée dès le premier pas, mais Pépin monte : il ne planerait pas).
        let mut j = Joueur::new(155.0, 176.0 - JOUEUR_H);
        let haut = Crochet { x: 160.0, y: 140.0 };
        j.actualiser_avec_crochets(&Entrees::default(), &env, &[haut]);
        assert!(j.au_sol);
        j.actualiser_avec_crochets(&saut_tenu(), &env, &[haut]);
        assert!(j.balancement().is_none());
        assert!(j.vitesse.y < 0.0, "il saute");
    }

    #[test]
    fn la_corde_est_bornee_entre_une_case_et_demie_et_trois() {
        let g = salle(false);
        let env = Environnement::simple(&g);
        // Accroché à 10 px du crochet : la corde fait 1,5 case.
        let mut j = pepin_crosse_en(160.0, 50.0);
        j.actualiser_avec_crochets(&saut_tenu(), &env, &[CROCHET]);
        let b = j.balancement().unwrap();
        assert_eq!(b.corde, CORDE_MIN);
        assert!((j.crosse().1 - (40.0 + CORDE_MIN)).abs() < 1e-3);
    }

    #[test]
    fn sans_appui_le_balancement_ne_gagne_pas_d_energie() {
        let g = salle(false);
        let env = Environnement::simple(&g);
        let mut j = suspendu(&g, 60.0);
        let e0 = j.balancement().unwrap().energie();
        // L'amplitude de chaque demi-balancement (l'angle au point de
        // retournement, là où toute l'énergie est en hauteur) ne grandit
        // jamais : 10 s, SAUT tenu, aucune direction.
        let mut amplitudes: Vec<f32> = vec![60f32.to_radians()];
        let mut omega_avant = 0.0;
        let mut theta_max: f32 = 0.0;
        for _ in 0..600 {
            j.actualiser_avec_crochets(&saut_tenu(), &env, &[CROCHET]);
            let b = j.balancement().expect("toujours suspendu");
            theta_max = theta_max.max(b.theta.abs());
            if omega_avant * b.omega < 0.0 {
                amplitudes.push(b.theta.abs());
            }
            omega_avant = b.omega;
        }
        assert!(amplitudes.len() >= 10, "{amplitudes:?}");
        for paire in amplitudes.windows(2) {
            assert!(paire[1] <= paire[0] + 1e-3, "{amplitudes:?}");
        }
        assert!(
            theta_max <= 60f32.to_radians() + 1e-4,
            "theta max = {theta_max}"
        );
        // Le frottement le fait doucement retomber.
        assert!(j.balancement().unwrap().energie() < e0);
    }

    #[test]
    fn gauche_et_droite_donnent_de_l_elan_jusqu_a_80_degres() {
        let g = salle(false);
        let env = Environnement::simple(&g);
        let mut j = suspendu(&g, 0.0);
        let mut theta_max: f32 = 0.0;
        for _ in 0..600 {
            // Pousser dans le sens du mouvement (à droite au départ).
            let b = j.balancement().unwrap();
            let direction = if b.omega >= 0.0 {
                Direction::Droite
            } else {
                Direction::Gauche
            };
            let e = Entrees {
                direction,
                saut: true,
                ..Entrees::default()
            };
            j.actualiser_avec_crochets(&e, &env, &[CROCHET]);
            theta_max = theta_max.max(j.balancement().unwrap().theta.abs());
        }
        assert!(theta_max > 70f32.to_radians(), "theta max = {theta_max}");
        assert!(theta_max <= ANGLE_MAX + 1e-6);
    }

    #[test]
    fn au_lacher_pepin_part_avec_la_vitesse_du_balancement_plus_dix_pour_cent() {
        let g = salle(false);
        let env = Environnement::simple(&g);
        let mut j = suspendu(&g, -60.0);
        // Il descend vers la droite ; on lâche juste après le bas du cercle.
        while j.balancement().unwrap().theta < 0.05 {
            j.actualiser_avec_crochets(&saut_tenu(), &env, &[CROCHET]);
        }
        let (vx, vy) = j.balancement().unwrap().vitesse();
        assert!(vx > 100.0 && vy < 0.0, "vers la droite et vers le haut");
        let avant = j.boite;
        j.actualiser_avec_crochets(&Entrees::default(), &env, &[CROCHET]);
        assert!(j.balancement().is_none());
        let (lx, ly) = (j.vitesse.x + j.elan_auvent(), j.vitesse.y);
        assert!((lx - vx * BONUS_LACHER).abs() < 1e-3, "{lx} / {vx}");
        assert!((ly - vy * BONUS_LACHER).abs() < 1e-3, "{ly} / {vy}");
        assert!(j.regarde_droite);
        // Ce pas-là, il a bougé exactement de cette vitesse.
        assert!((j.boite.x - avant.x - lx * DT).abs() < 1e-3);
        assert!((j.boite.y - avant.y - ly * DT).abs() < 1e-3);
        // Il peut planer aussitôt : SAUT de nouveau tenu, en descente.
        let mut pas = 0;
        while j.vitesse.y < 0.0 && pas < 120 {
            j.actualiser_avec_crochets(&Entrees::default(), &env, &[CROCHET]);
            pas += 1;
        }
        j.actualiser_avec_crochets(&saut_tenu(), &env, &[]);
        assert!(j.plane);
    }

    #[test]
    fn pas_de_raccroche_au_meme_crochet_avant_trois_dixiemes_de_seconde() {
        let g = salle(false);
        let env = Environnement::simple(&g);
        let autre = Crochet { x: 150.0, y: 36.0 };
        for crochets in [vec![CROCHET], vec![CROCHET, autre]] {
            let mut j = suspendu(&g, 0.0);
            // Lâché en bas, immobile.
            j.actualiser_avec_crochets(&Entrees::default(), &env, &crochets);
            assert!(j.balancement().is_none());
            // Chaque pas, on le remet à la même place, immobile, SAUT tenu : le
            // crochet est à portée.
            let mut pas = 0;
            while j.balancement().is_none() && pas < 60 {
                j.boite.x = 155.0;
                j.boite.y = 72.0 + HAUTEUR_CROSSE;
                j.vitesse.y = 0.0;
                j.actualiser_avec_crochets(&saut_tenu(), &env, &crochets);
                pas += 1;
            }
            let b = j.balancement().expect("raccroché");
            if crochets.len() == 1 {
                // Le même crochet : au 18e pas après le lâcher (0,3 s).
                assert_eq!((pas, b.crochet), (usize::from(DELAI_RACCROCHE_PAS), 0));
            } else {
                // Un AUTRE crochet à portée prend tout de suite.
                assert_eq!((pas, b.crochet), (1, 1));
            }
        }
    }

    #[test]
    fn un_solide_touche_decroche_pepin() {
        let g = salle(true);
        let env = Environnement::simple(&g);
        // Corde de 32 px, crochet à x = 160, mur à x = 192 : vers 60° à
        // droite, Pépin (crosse à 160 + 27,7, boîte jusqu'à + 5) touche le mur.
        let mut j = suspendu(&g, -70.0);
        let mut decroche = false;
        for _ in 0..120 {
            j.actualiser_avec_crochets(&saut_tenu(), &env, &[CROCHET]);
            if j.balancement().is_none() {
                decroche = true;
                break;
            }
        }
        assert!(decroche, "le mur l'a décroché");
        assert!(j.boite.droite() <= 192.0 + 1e-3);
        assert_eq!(j.vitesse.x, 0.0, "arrêté par le mur");
    }

    #[test]
    fn meme_suite_d_entrees_meme_resultat() {
        let g = salle(true);
        let env = Environnement::simple(&g);
        let suite: Vec<Entrees> = (0..400)
            .map(|i| Entrees {
                direction: match (i / 23) % 3 {
                    0 => Direction::Droite,
                    1 => Direction::Gauche,
                    _ => Direction::Immobile,
                },
                saut: (i / 61) % 4 != 3,
                ..Entrees::default()
            })
            .collect();
        let jouer = || {
            let mut j = pepin_crosse_en(150.0, 70.0);
            let mut trace = Vec::new();
            for e in &suite {
                j.actualiser_avec_crochets(e, &env, &[CROCHET]);
                trace.push((j.boite.x.to_bits(), j.boite.y.to_bits(), j.etat()));
            }
            trace
        };
        let (a, b) = (jouer(), jouer());
        assert_eq!(a, b);
        assert!(a.iter().any(|t| t.2 == EtatJoueur::Suspendu));
    }

    #[test]
    fn sans_crochet_rien_ne_change() {
        let g = salle(false);
        let env = Environnement::simple(&g);
        let mut a = pepin_crosse_en(160.0, 72.0);
        let mut b = a.clone();
        for i in 0..200 {
            let e = Entrees {
                saut: i % 50 < 30,
                direction: Direction::Droite,
                ..Entrees::default()
            };
            a.actualiser(&e, &env);
            b.actualiser_avec_crochets(&e, &env, &[]);
            assert_eq!(a.boite, b.boite);
        }
    }
}
