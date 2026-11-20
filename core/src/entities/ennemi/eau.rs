//! Les ennemis d'eau du Monde 2 (carte 9.4) : grenouille, poisson, crabe, rat.
//! La mouette, elle, est un pigeon du port : elle reprend tout le code du
//! sous-module `pigeon` (même annonce, même piqué), seul son dessin change.
//!
//! Chacun a son cycle dans `PhaseEau` (un seul champ pour les quatre) et son
//! ANNONCE visible avant d'attaquer (règle du jeu) :
//! - grenouille (`j`) : posée (sur une berge, un nénuphar ou l'eau), elle guette.
//!   Pépin à portée (5 tuiles en x, 3 en hauteur), elle se RAMASSE 0,5 s
//!   (annonce), puis bondit en arc vers l'endroit où il était à la fin de
//!   l'annonce, et se pose là où elle retombe. Écrasable en sautant dessus ;
//! - poisson (`i`) : sous l'eau, sur une colonne fixe. En cycle : caché 1,5 s,
//!   REMOUS en surface 0,7 s (annonce), puis il saute hors de l'eau à la
//!   verticale (3,5 tuiles) et retombe. Mortel au contact, seulement hors de
//!   l'eau ; jamais écrasable (il est dans l'air trop vite) ;
//! - crabe (`k`) : patrouille au sol comme la flaque, lentement. IMPOSSIBLE À
//!   ÉCRASER : lui sauter dessus fait rebondir Pépin (comme un ennemi écrasé)
//!   sans blesser le crabe, qui claque des pinces et s'arrête un instant. De
//!   côté, il tue. Sa pince levée signale la carapace dure ; il lève les deux
//!   quand Pépin approche par le dessus ;
//! - rat (`r`) : patrouille dans son tunnel. Pépin sur la même ligne (pieds à la
//!   même hauteur, à moins de 6 tuiles, aucun mur entre eux), il s'arrête et se
//!   dresse 0,5 s (annonce), puis FONCE en ligne droite jusqu'à un mur (ou au
//!   bord du vide), où il reste étourdi 0,75 s. Écrasable.
//!
//! Notions Rust : un `enum` d'état partagé par plusieurs espèces, le `match` sur
//! un couple `(espèce, phase)` ; `Option::is_some_and` pour un test sur une
//! valeur facultative.

use super::{Ennemi, EtatEnnemi, TypeEnnemi};
use crate::aabb::{deplacer, Aabb, TAILLE_TUILE};
use crate::physics::{Environnement, DT, GRAVITE, HZ, VITESSE_MAX_CHUTE};

/// Phase d'un ennemi d'eau (toujours `Guet` pour les ennemis du Monde 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseEau {
    /// Au repos : la grenouille guette, le crabe et le rat patrouillent.
    Guet,
    /// Annonce, `n` pas restants : la grenouille se ramasse, les remous du
    /// poisson, le rat dressé ; pour le crabe, les deux pinces levées.
    Annonce(u16),
    /// Attaque, `n`-ième pas : bond de la grenouille, saut du poisson, course du
    /// rat ; pour le crabe, les pinces qui claquent après un rebond de Pépin.
    Attaque(u16),
    /// Après l'attaque, `n` pas restants : grenouille posée, poisson caché sous
    /// l'eau, rat étourdi contre son mur.
    Repos(u16),
}

/// Grenouille : portée de son guet (px, entre les centres en x, entre les pieds en y).
pub const PORTEE_GRENOUILLE: (f32, f32) = (80.0, 48.0);
/// Grenouille : annonce (elle se ramasse), 0,5 s.
pub const ANNONCE_GRENOUILLE_PAS: u16 = 30;
/// Grenouille : vitesse verticale au départ du bond (px/s) : 32 px de haut, 0,53 s de vol.
pub const VITESSE_BOND_GRENOUILLE: f32 = 240.0;
/// Grenouille : repos après un bond avant de guetter de nouveau, 1 s.
pub const REPOS_GRENOUILLE_PAS: u16 = 60;

/// Poisson : temps caché sous l'eau entre deux sauts, 1,5 s.
pub const SOUS_L_EAU_PAS: u16 = 90;
/// Poisson : annonce (les remous en surface), 0,7 s.
pub const REMOUS_PAS: u16 = 42;
/// Poisson : durée du saut hors de l'eau, 0,8 s.
pub const SAUT_POISSON_PAS: u16 = 48;
/// Poisson : hauteur du saut (px) par défaut : son dos monte 3,5 tuiles au-dessus
/// de la surface. Un niveau peut la changer pour des poissons précis (carte 9.11e :
/// `InfosNiveau::poissons_hauts`, `Ennemi::regler_saut_poisson`).
pub const HAUT_SAUT_POISSON: f32 = 56.0;
/// Poisson : au repos, le haut de sa boîte est 2 px sous la surface (caché).
pub const PROFONDEUR_POISSON: f32 = 2.0;

/// Crabe : vitesse de patrouille (px/s), moins que la flaque.
pub const VITESSE_CRABE: f32 = 20.0;
/// Crabe : il lève les deux pinces quand Pépin est au-dessus, à moins de 3
/// tuiles en x (entre les centres) et de 5 tuiles en hauteur.
pub const ALERTE_CRABE: (f32, f32) = (48.0, 80.0);
/// Crabe : pas d'arrêt (pinces qui claquent) après un rebond de Pépin.
pub const CLAQUE_CRABE_PAS: u16 = 20;

/// Rat : vitesse de patrouille (px/s).
pub const VITESSE_RAT: f32 = 24.0;
/// Rat : vitesse de la course (px/s), plus vite que Pépin.
pub const VITESSE_FONCE_RAT: f32 = 150.0;
/// Rat : portée (px entre les centres) où il voit Pépin sur sa ligne, 6 tuiles.
pub const PORTEE_RAT: f32 = 96.0;
/// Rat : écart maximal (px) entre ses pieds et ceux de Pépin pour « la même ligne ».
pub const LIGNE_RAT: f32 = 4.0;
/// Rat : annonce (il s'arrête et se dresse), 0,5 s.
pub const ANNONCE_RAT_PAS: u16 = 30;
/// Rat : étourdi contre le mur après sa course, 0,75 s.
pub const REPOS_RAT_PAS: u16 = 45;

/// Décalage vertical (px, vers le HAUT) du poisson pendant son saut, au pas `n`
/// (parabole : 0 au départ et à l'arrivée, `haut` au milieu : la hauteur de saut
/// de CE poisson, `HAUT_SAUT_POISSON` par défaut).
pub fn hauteur_saut_poisson(n: u16, haut: f32) -> f32 {
    let f = f32::from(n.min(SAUT_POISSON_PAS)) / f32::from(SAUT_POISSON_PAS);
    haut * 4.0 * f * (1.0 - f)
}

impl TypeEnnemi {
    /// Ennemi d'eau du Monde 2 (carte 9.4), sauf la mouette (un pigeon) ?
    pub fn est_d_eau(self) -> bool {
        matches!(
            self,
            TypeEnnemi::Grenouille | TypeEnnemi::Poisson | TypeEnnemi::Crabe | TypeEnnemi::Rat
        )
    }
}

impl Ennemi {
    /// Grenouille posée dans la case dont le coin haut-gauche est en `(x, y)`
    /// (boîte 12 x 10 au bas de la case) ; elle tombe sur le solide (ou l'eau) dessous.
    pub fn grenouille(x: f32, y: f32) -> Self {
        Ennemi::new(TypeEnnemi::Grenouille, x + 2.0, y + 6.0)
    }

    /// Poisson de la colonne `x` (bord gauche de la case) sous la surface
    /// `surface` (px). Son premier saut est avancé de `decalage_pas` pas : deux
    /// poissons voisins ne sautent pas ensemble.
    pub fn poisson(x: f32, surface: f32, decalage_pas: u16) -> Self {
        let mut p = Ennemi::new(TypeEnnemi::Poisson, x + 3.0, surface + PROFONDEUR_POISSON);
        p.eau = PhaseEau::Repos(SOUS_L_EAU_PAS - decalage_pas.min(SOUS_L_EAU_PAS - 1));
        p.depart.eau = p.eau;
        p
    }

    /// Crabe dans la case dont le coin haut-gauche est en `(x, y)` (boîte 14 x 9
    /// au bas de la case).
    pub fn crabe(x: f32, y: f32) -> Self {
        Ennemi::new(TypeEnnemi::Crabe, x + 1.0, y + 7.0)
    }

    /// Rat dans la case dont le coin haut-gauche est en `(x, y)` (boîte 14 x 8 au
    /// bas de la case).
    pub fn rat(x: f32, y: f32) -> Self {
        Ennemi::new(TypeEnnemi::Rat, x + 1.0, y + 8.0)
    }

    /// Mouette perchée (Monde 2, le port) : un pigeon (même annonce, même piqué,
    /// voir le sous-module `pigeon`) ; seul son dessin change.
    pub fn mouette(x: f32, y: f32, vers_la_droite: bool) -> Self {
        let mut m = Ennemi::pigeon(x, y, vers_la_droite);
        m.type_ennemi = TypeEnnemi::Mouette;
        m
    }

    /// Phase de l'ennemi d'eau (`Guet` pour les autres).
    pub fn phase_eau(&self) -> PhaseEau {
        self.eau
    }

    /// Le poisson est-il hors de l'eau (en plein saut) ? Vrai pour tous les
    /// autres ennemis : seul le poisson caché sous l'eau est inoffensif.
    pub fn hors_de_l_eau(&self) -> bool {
        self.type_ennemi != TypeEnnemi::Poisson
            || matches!(self.eau, PhaseEau::Attaque(n) if n < SAUT_POISSON_PAS)
    }

    /// Hauteur (px) du saut du poisson (`HAUT_SAUT_POISSON` sauf réglage du niveau).
    pub fn saut_poisson(&self) -> f32 {
        self.saut_poisson
    }

    /// Règle la hauteur du saut (px) du poisson, gardée après une mort de Pépin
    /// (carte 9.11e : les poissons de la fin du port sautent plus haut).
    pub fn regler_saut_poisson(&mut self, haut: f32) {
        self.saut_poisson = haut;
        self.depart.saut_poisson = haut;
    }

    /// Case (colonne, rangée) de la lettre `i` du poisson : la case d'air juste
    /// au-dessus de la surface (voir `level::ascii`).
    pub fn case_du_poisson(&self) -> (usize, usize) {
        let colonne = ((self.origine_x - 3.0) / TAILLE_TUILE).round() as usize;
        let rangee = (self.surface_du_poisson() / TAILLE_TUILE).round() as usize - 1;
        (colonne, rangee)
    }

    /// Ordonnée (px) de la surface au-dessus du poisson (`origine_y` moins sa profondeur).
    pub fn surface_du_poisson(&self) -> f32 {
        self.origine_y - PROFONDEUR_POISSON
    }

    /// Carapace dure (le crabe) : sauté dessus, il fait rebondir Pépin sans être
    /// blessé (`contacts::resoudre_contacts`).
    pub fn a_une_carapace(&self) -> bool {
        self.type_ennemi == TypeEnnemi::Crabe && self.etat == EtatEnnemi::Actif
    }

    /// Pépin vient de rebondir sur la carapace : les pinces claquent, il s'arrête.
    pub fn claquer(&mut self) {
        self.eau = PhaseEau::Attaque(CLAQUE_CRABE_PAS);
    }

    /// La grenouille voit-elle Pépin (`joueur`) à portée de bond ?
    pub fn grenouille_voit(&self, joueur: &Aabb) -> bool {
        let dx = (joueur.centre().0 - self.boite.centre().0).abs();
        let dy = (joueur.bas() - self.boite.bas()).abs();
        dx <= PORTEE_GRENOUILLE.0 && dy <= PORTEE_GRENOUILLE.1
    }

    /// Le rat voit-il Pépin sur sa ligne : pieds à la même hauteur (à
    /// `LIGNE_RAT` près), à moins de `PORTEE_RAT`, sans tuile solide entre eux ?
    pub fn rat_voit(&self, joueur: &Aabb, env: &Environnement<'_>) -> bool {
        let (cx, jx) = (self.boite.centre().0, joueur.centre().0);
        if (joueur.bas() - self.boite.bas()).abs() > LIGNE_RAT || (jx - cx).abs() > PORTEE_RAT {
            return false;
        }
        let rangee = ((self.boite.bas() - 1.0) / TAILLE_TUILE).floor() as i32;
        let (a, b) = (cx.min(jx), cx.max(jx));
        let (ca, cb) = (
            (a / TAILLE_TUILE).floor() as i32,
            (b / TAILLE_TUILE).floor() as i32,
        );
        !(ca..=cb).any(|c| env.grille.est_solide(c, rangee))
    }

    /// Regarde Pépin (appelé par `guetter`, dans `resoudre_contacts`) : la
    /// grenouille et le rat le retiennent pour le pas suivant ; le crabe lève
    /// les deux pinces s'il est au-dessus de lui.
    pub(super) fn guetter_eau(&mut self, joueur: &Aabb) {
        self.proie = Some(*joueur);
        if self.type_ennemi == TypeEnnemi::Crabe && !matches!(self.eau, PhaseEau::Attaque(_)) {
            let dx = (joueur.centre().0 - self.boite.centre().0).abs();
            let dessus = self.boite.y - joueur.bas();
            let alerte = dx <= ALERTE_CRABE.0 && (-2.0..=ALERTE_CRABE.1).contains(&dessus);
            self.eau = if alerte {
                PhaseEau::Annonce(1)
            } else {
                PhaseEau::Guet
            };
        }
    }

    /// Un pas des ennemis d'eau (appelé par `actualiser`).
    pub(super) fn actualiser_eau(&mut self, env: &Environnement<'_>) {
        match self.type_ennemi {
            TypeEnnemi::Grenouille => self.pas_grenouille(env),
            TypeEnnemi::Poisson => self.pas_poisson(),
            TypeEnnemi::Crabe => self.pas_crabe(env),
            _ => self.pas_rat(env),
        }
    }

    /// Chute (gravité) avec une vitesse horizontale `vx` ; un mur l'annule. La
    /// grenouille se pose aussi sur la SURFACE de l'eau (elle y flotte).
    fn chute(&mut self, vx: f32, env: &Environnement<'_>) {
        let bas_avant = self.boite.bas();
        self.vitesse_y = (self.vitesse_y + GRAVITE * DT).min(VITESSE_MAX_CHUTE);
        let c = deplacer(
            &mut self.boite,
            vx * DT,
            self.vitesse_y * DT,
            env.grille,
            env.solides,
        );
        if c.mur() {
            self.elan = 0.0;
        }
        self.au_sol = c.sol;
        if c.sol || c.plafond {
            self.vitesse_y = 0.0;
        }
        if !self.au_sol && self.vitesse_y > 0.0 {
            // La surface que la grenouille vient de FRANCHIR en tombant, sous
            // son centre (pas celle d'un bassin plus haut, correctif d'AGA).
            let cx = self.boite.centre().0;
            let bas = self.boite.bas();
            let surface = env
                .eaux
                .iter()
                .map(|e| (e, e.surface()))
                .find(|&(e, s)| e.couvre_x(cx) && bas_avant <= s + 0.01 && bas >= s)
                .map(|(_, s)| s);
            if let Some(s) = surface {
                self.boite.y = s - self.boite.h;
                self.vitesse_y = 0.0;
                self.au_sol = true;
            }
        }
    }

    fn pas_grenouille(&mut self, env: &Environnement<'_>) {
        self.eau = match self.eau {
            PhaseEau::Guet => {
                self.chute(0.0, env);
                match self.proie {
                    Some(j) if self.au_sol && self.grenouille_voit(&j) => {
                        self.vers_la_droite = j.centre().0 > self.boite.centre().0;
                        PhaseEau::Annonce(ANNONCE_GRENOUILLE_PAS)
                    }
                    _ => PhaseEau::Guet,
                }
            }
            PhaseEau::Annonce(n) if n > 1 => PhaseEau::Annonce(n - 1),
            PhaseEau::Annonce(_) => {
                // Le bond vise l'endroit où est Pépin à la fin de l'annonce.
                let vol = 2.0 * VITESSE_BOND_GRENOUILLE / GRAVITE;
                let dx = self
                    .proie
                    .map_or(0.0, |j| j.centre().0 - self.boite.centre().0);
                self.elan = dx.clamp(-PORTEE_GRENOUILLE.0, PORTEE_GRENOUILLE.0) / vol;
                self.vers_la_droite = self.elan > 0.0;
                self.vitesse_y = -VITESSE_BOND_GRENOUILLE;
                self.au_sol = false;
                self.chute(self.elan, env);
                PhaseEau::Attaque(1)
            }
            PhaseEau::Attaque(n) => {
                self.chute(self.elan, env);
                if self.au_sol {
                    self.elan = 0.0;
                    PhaseEau::Repos(REPOS_GRENOUILLE_PAS)
                } else {
                    PhaseEau::Attaque(n.saturating_add(1))
                }
            }
            PhaseEau::Repos(n) => {
                self.chute(0.0, env);
                if n > 1 {
                    PhaseEau::Repos(n - 1)
                } else {
                    PhaseEau::Guet
                }
            }
        };
    }

    fn pas_poisson(&mut self) {
        self.eau = match self.eau {
            PhaseEau::Guet | PhaseEau::Repos(1) => PhaseEau::Annonce(REMOUS_PAS),
            PhaseEau::Repos(n) => PhaseEau::Repos(n - 1),
            PhaseEau::Annonce(n) if n > 1 => PhaseEau::Annonce(n - 1),
            PhaseEau::Annonce(_) => PhaseEau::Attaque(1),
            PhaseEau::Attaque(n) if n < SAUT_POISSON_PAS => PhaseEau::Attaque(n + 1),
            PhaseEau::Attaque(_) => PhaseEau::Repos(SOUS_L_EAU_PAS),
        };
        let haut = match self.eau {
            PhaseEau::Attaque(n) => hauteur_saut_poisson(n, self.saut_poisson),
            _ => 0.0,
        };
        self.boite.y = self.origine_y - haut;
    }

    fn pas_crabe(&mut self, env: &Environnement<'_>) {
        if let PhaseEau::Attaque(n) = self.eau {
            // Pinces qui claquent : il ne bouge pas.
            self.eau = if n > 1 {
                PhaseEau::Attaque(n - 1)
            } else {
                PhaseEau::Guet
            };
            return;
        }
        super::super::marche::patrouiller(
            &mut self.boite,
            &mut self.vers_la_droite,
            &mut self.vitesse_y,
            &mut self.au_sol,
            VITESSE_CRABE,
            env,
        );
    }

    fn pas_rat(&mut self, env: &Environnement<'_>) {
        let vitesse = match self.eau {
            PhaseEau::Guet => VITESSE_RAT,
            PhaseEau::Attaque(_) => VITESSE_FONCE_RAT,
            PhaseEau::Annonce(_) | PhaseEau::Repos(_) => 0.0,
        };
        let sens = self.vers_la_droite;
        super::super::marche::patrouiller(
            &mut self.boite,
            &mut self.vers_la_droite,
            &mut self.vitesse_y,
            &mut self.au_sol,
            vitesse,
            env,
        );
        self.eau = match self.eau {
            PhaseEau::Guet => match self.proie {
                Some(j) if self.au_sol && self.rat_voit(&j, env) => {
                    self.vers_la_droite = j.centre().0 > self.boite.centre().0;
                    PhaseEau::Annonce(ANNONCE_RAT_PAS)
                }
                _ => PhaseEau::Guet,
            },
            PhaseEau::Annonce(n) if n > 1 => PhaseEau::Annonce(n - 1),
            PhaseEau::Annonce(_) => PhaseEau::Attaque(1),
            // Demi-tour (un mur, ou le bord du vide) : la course s'arrête là.
            PhaseEau::Attaque(_) if self.vers_la_droite != sens => PhaseEau::Repos(REPOS_RAT_PAS),
            PhaseEau::Attaque(n) => PhaseEau::Attaque(n.saturating_add(1)),
            PhaseEau::Repos(n) if n > 1 => PhaseEau::Repos(n - 1),
            PhaseEau::Repos(_) => PhaseEau::Guet,
        };
    }
}

/// Durée (pas) d'un cycle complet du poisson : caché, remous, saut (3 s).
pub const PERIODE_POISSON_PAS: u32 =
    SOUS_L_EAU_PAS as u32 + REMOUS_PAS as u32 + SAUT_POISSON_PAS as u32;
const _: () = assert!(PERIODE_POISSON_PAS == 3 * HZ);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::GrilleTuiles;
    use crate::entities::contacts::{resoudre_contacts, Contact};
    use crate::entities::{Actualisable, Entite};
    use crate::physics::{Eau, JOUEUR_H, JOUEUR_L};
    use crate::player::Joueur;

    /// Couloir plat : sol à la rangée 5 (y = 80), murs aux colonnes 0 et 19.
    fn couloir() -> GrilleTuiles {
        GrilleTuiles::depuis_texte(&[
            "#..................#",
            "#..................#",
            "#..................#",
            "#..................#",
            "#..................#",
            "####################",
        ])
    }

    fn pepin(x: f32, bas: f32) -> Aabb {
        Aabb::new(x, bas - JOUEUR_H, JOUEUR_L, JOUEUR_H)
    }

    fn avancer(e: &mut Ennemi, env: &Environnement<'_>, pepin: &Aabb, pas: u32) {
        for _ in 0..pas {
            e.actualiser(env);
            e.guetter(pepin);
        }
    }

    /// La grenouille : Pépin à portée, elle se ramasse 0,5 s sur place, puis
    /// bondit (elle quitte le sol) et retombe près de lui. On l'écrase par le dessus.
    #[test]
    fn la_grenouille_s_annonce_puis_bondit_vers_pepin() {
        let g = couloir();
        let env = Environnement::simple(&g);
        let mut j = Ennemi::grenouille(48.0, 64.0);
        let loin = pepin(250.0, 80.0);
        avancer(&mut j, &env, &loin, 30);
        assert_eq!(j.phase_eau(), PhaseEau::Guet, "Pépin hors de portée");
        assert!((j.boite.bas() - 80.0).abs() < 1e-3, "posée sur le sol");
        let proche = pepin(110.0, 80.0);
        avancer(&mut j, &env, &proche, 2);
        assert!(matches!(j.phase_eau(), PhaseEau::Annonce(_)));
        let x0 = j.boite.x;
        avancer(&mut j, &env, &proche, u32::from(ANNONCE_GRENOUILLE_PAS) - 2);
        assert_eq!(j.boite.x, x0, "pendant l'annonce, elle ne bouge pas");
        avancer(&mut j, &env, &proche, 10);
        assert!(matches!(j.phase_eau(), PhaseEau::Attaque(_)) && j.boite.bas() < 79.0);
        avancer(&mut j, &env, &proche, 40);
        assert!(
            matches!(j.phase_eau(), PhaseEau::Repos(_)),
            "{:?}",
            j.phase_eau()
        );
        let ecart = (j.boite.centre().0 - proche.centre().0).abs();
        assert!(ecart < 4.0, "retombée sur Pépin : écart {ecart}");

        // Écrasée par le dessus : Pépin rebondit, elle s'aplatit.
        let mut joueur = Joueur::new(j.boite.x, j.boite.y - JOUEUR_H + 3.0);
        joueur.vitesse.y = 100.0;
        let mut ents = vec![Entite::Ennemi(j)];
        let contacts = resoudre_contacts(&mut joueur, &mut ents);
        assert_eq!(contacts, vec![Contact::Ecrase]);
    }

    /// Le poisson : caché sous l'eau, il est inoffensif ; remous, puis il saute
    /// hors de l'eau sur sa colonne, et là il tue (même par le dessus).
    #[test]
    fn le_poisson_ne_tue_que_hors_de_l_eau() {
        let g = couloir();
        let eaux = [Eau {
            zone: Aabb::new(16.0, 64.0, 288.0, 16.0),
            courant: 0,
        }];
        let env = Environnement {
            eaux: &eaux,
            ..Environnement::simple(&g)
        };
        let mut p = Ennemi::poisson(96.0, 64.0, SOUS_L_EAU_PAS - 1);
        let mut vu_annonce = false;
        let mut haut_max = 0.0f32;
        for _ in 0..PERIODE_POISSON_PAS {
            p.actualiser(&env);
            vu_annonce |= matches!(p.phase_eau(), PhaseEau::Annonce(_));
            assert_eq!(p.est_dangereux(), p.boite.y < 64.0, "{:?}", p.phase_eau());
            assert_eq!(p.boite.x, 99.0, "colonne fixe");
            haut_max = haut_max.max(64.0 - p.boite.y);
        }
        assert!(vu_annonce && haut_max > HAUT_SAUT_POISSON - 4.0);

        // En plein saut : Pépin qui lui tombe dessus meurt (pas écrasable).
        while p.phase_eau() != PhaseEau::Attaque(SAUT_POISSON_PAS / 2) {
            p.actualiser(&env);
        }
        let mut joueur = Joueur::new(p.boite.x, p.boite.y - JOUEUR_H + 3.0);
        joueur.vitesse.y = 100.0;
        let mut ents = vec![Entite::Ennemi(p)];
        let contacts = resoudre_contacts(&mut joueur, &mut ents);
        assert_eq!(
            contacts,
            vec![Contact::Mort(crate::entities::contacts::CauseMort::Ennemi)]
        );
    }

    /// Le crabe fait rebondir Pépin sans mourir, et reste entier ; de côté, il tue.
    #[test]
    fn le_crabe_fait_rebondir_sans_mourir() {
        let crabe = Ennemi::crabe(96.0, 64.0);
        let mut joueur = Joueur::new(crabe.boite.x, crabe.boite.y - JOUEUR_H + 3.0);
        joueur.vitesse.y = 120.0;
        let mut ents = vec![Entite::Ennemi(crabe.clone())];
        let contacts = resoudre_contacts(&mut joueur, &mut ents);
        assert_eq!(contacts, vec![Contact::Rebond]);
        assert!(joueur.vitesse.y < 0.0, "Pépin rebondit");
        let Entite::Ennemi(c) = &ents[0] else {
            unreachable!()
        };
        assert!(
            c.est_dangereux() && c.etat == EtatEnnemi::Actif,
            "crabe intact"
        );
        assert!(
            matches!(c.phase_eau(), PhaseEau::Attaque(_)),
            "pinces qui claquent"
        );

        let mut joueur = Joueur::new(crabe.boite.x - 6.0, crabe.boite.bas() - JOUEUR_H);
        let mut ents = vec![Entite::Ennemi(crabe)];
        let contacts = resoudre_contacts(&mut joueur, &mut ents);
        assert_eq!(
            contacts,
            vec![Contact::Mort(crate::entities::contacts::CauseMort::Ennemi)]
        );
    }

    /// Le rat : Pépin sur sa ligne, il s'arrête 0,5 s (annonce), puis fonce
    /// jusqu'au mur, où il reste étourdi. Un mur entre eux : il ne le voit pas.
    #[test]
    fn le_rat_s_annonce_puis_fonce_jusqu_au_mur() {
        let g = couloir();
        let env = Environnement::simple(&g);
        let mut r = Ennemi::rat(160.0, 64.0);
        r.vers_la_droite = true;
        let pepin = pepin(80.0, 80.0);
        avancer(&mut r, &env, &pepin, 2);
        assert!(matches!(r.phase_eau(), PhaseEau::Annonce(_)) && !r.vers_la_droite);
        let x0 = r.boite.x;
        avancer(&mut r, &env, &pepin, u32::from(ANNONCE_RAT_PAS) - 1);
        assert!((r.boite.x - x0).abs() < 1e-3, "immobile pendant l'annonce");
        // Pépin a sauté par-dessus : le rat file quand même jusqu'au mur gauche.
        let ailleurs = pepin.decale(100.0, -40.0);
        avancer(&mut r, &env, &ailleurs, 80);
        assert!(
            matches!(r.phase_eau(), PhaseEau::Repos(_)),
            "{:?}",
            r.phase_eau()
        );
        assert!(r.boite.x < 16.5, "au mur : x = {}", r.boite.x);

        let mure = GrilleTuiles::depuis_texte(&[
            "#..................#",
            "#.......#..........#",
            "####################",
        ]);
        let env = Environnement::simple(&mure);
        let r = Ennemi::rat(160.0, 16.0);
        assert!(!r.rat_voit(&self::pepin(90.0, 32.0), &env));
        assert!(r.rat_voit(&self::pepin(150.0, 32.0), &env));
    }
}
