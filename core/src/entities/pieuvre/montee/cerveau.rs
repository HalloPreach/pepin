//! Le CERVEAU de la géante (carte 9.13d) : ce qu'elle voit de Pépin, son
//! humeur, et l'attaque qu'elle choisit.
//!
//! Critique d'AGA : « Quatre attaques sur cinq sont attachées à un endroit, pas
//! à elle. Après une mort, on connaît tout par cœur. » Plus aucune attaque ne
//! part d'une rangée fixe. À chaque fois qu'elle est libre, elle regarde Pépin
//! et dresse la liste de ce qui est POSSIBLE là où il est :
//!
//! | attaque    | possible quand Pépin…                                   |
//! |------------|---------------------------------------------------------|
//! | jeté       | est sur une corniche LARGE (et qu'il lui reste 2 têtes) |
//! | balayage   | est sur une corniche qui touche son couloir             |
//! | secousse   | est debout sous la tour (au moins 2 caisses tombent)    |
//! | arrachage  | arrive sous une plateforme `%` (qui a une autre route)  |
//! | frappe     | toujours : elle vise l'appui où il VA atterrir          |
//!
//! Chaque attaque a un TEMPS DE RECHARGE ; parmi celles qui sont prêtes, elle
//! tire au sort, avec des poids, grâce à un générateur PSEUDO-ALÉATOIRE
//! DÉTERMINISTE (un « xorshift » : quelques décalages et ou-exclusifs). Sa
//! graine change à chaque tentative (`tentative`, le nombre de reprises) : après
//! une mort, la montée n'est plus la même. Les tests fixent la graine
//! (`fixer_graine`) et retrouvent exactement la même suite.
//!
//! Son HUMEUR monte en trois paliers : CALME au départ, ÉNERVÉE après le 1er
//! coup reçu ou passé 50 % de la tour, FURIEUSE après le 2e coup ou passé 80 %.
//! À chaque palier, les recharges et son souffle entre deux attaques
//! raccourcissent, et dès qu'elle est énervée, un balayage ou une secousse peut
//! s'enchaîner d'une FRAPPE VISÉE (un COMBO) : le repère rouge se pose pendant
//! la première attaque, et la frappe tombe juste APRÈS sa partie mortelle,
//! jamais en même temps (les tests le vérifient).
//!
//! Notions Rust : un sous-module d'un sous-module (`pieuvre::montee::cerveau`)
//! voit les champs privés de `Pieuvre` et de `Tour`, définis chez ses ancêtres ;
//! un générateur pseudo-aléatoire en trois lignes (`u32` et opérations bit à
//! bit, `wrapping_mul` pour multiplier sans paniquer au dépassement).

use super::{Attaque, Balayeur, Pieuvre, TAILLE_GEANTE};
use super::{ANNONCE_BALAI_PAS, ANNONCE_FRAPPE_PAS, ANNONCE_SECOUSSE_PAS};
use super::{ARRACHAGE_COTE, ARRACHAGE_SOUS, RYTHME_FRAPPE_PAS};
use crate::aabb::{Aabb, Grille, Tuile, TAILLE_TUILE};
use crate::physics::{JOUEUR_H, JOUEUR_L};

/// Ses trois humeurs, de la plus douce à la plus dangereuse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Humeur {
    #[default]
    Calme,
    Enervee,
    Furieuse,
}

/// Le genre d'une attaque (le journal du cerveau, les recharges).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Genre {
    Frappe,
    Balayage,
    Secousse,
    Arrachage,
    Jete,
}

impl Genre {
    /// Rang dans les tables (`RECHARGES`, `POIDS`).
    fn rang(self) -> usize {
        match self {
            Genre::Frappe => 0,
            Genre::Balayage => 1,
            Genre::Secousse => 2,
            Genre::Arrachage => 3,
            Genre::Jete => 4,
        }
    }
}

/// Recharge de chaque genre quand elle est calme (pas, depuis le DÉBUT de
/// l'attaque) : frappe 2,5 s, balayage 6 s, secousse 8 s, arrachage 4 s,
/// jeté 6 s.
pub const RECHARGES: [u16; 5] = [RYTHME_FRAPPE_PAS, 360, 480, 240, 360];
/// Les poids du tirage : un jeté possible est souvent choisi (c'est la
/// respiration de la montée), la frappe reste la plus fréquente sinon.
const POIDS: [u32; 5] = [3, 3, 2, 4, 6];
/// Les recharges selon l'humeur (en centièmes) : énervée 0,7, furieuse 0,5.
const FACTEUR_HUMEUR: [u32; 3] = [100, 70, 50];
/// Son souffle entre deux attaques (pas), selon l'humeur, plus un tirage d'au
/// plus la moitié en plus.
const SOUFFLE: [u16; 3] = [30, 18, 8];
/// Graine par défaut (la tentative la fait varier).
const GRAINE: u32 = 0x5EED_9013;
/// Largeur (cases) d'une corniche où elle peut se jeter : sa tête (4 cases) et
/// 2 cases pour l'éviter.
pub const JETE_LARGE: i32 = 6;
/// Le plus haut qu'un appui visé peut être au-dessus des pieds (px) : un saut
/// (320² / (2 x 900) = 57 px), un peu plus.
const SAUT_MAX: f32 = 4.0 * TAILLE_TUILE;
/// Le plus loin, de côté, qu'un appui visé peut être de Pépin (px).
const PORTEE_VISEE: f32 = 6.0 * TAILLE_TUILE;
/// Pas d'avance de la prévision : où sera-t-il dans 0,5 s ?
const AVANCE_PAS: f32 = 30.0;

/// Ce qu'elle voit, ce qu'elle ressent et ce qu'elle décide. Une partie dure
/// toute la phase 2 (la graine, la tentative, les places de jeté), le reste est
/// remis à neuf à chaque tentative (`oublier`).
#[derive(Debug, Clone, Default)]
pub(super) struct Cerveau {
    /// La graine choisie (`None` : `GRAINE`).
    graine: Option<u32>,
    /// Nombre de reprises depuis le début de la phase 2.
    pub(super) tentative: u32,
    /// L'état du générateur.
    tirage: u32,
    /// Les places de jeté et les balayeurs sont tirés de la grille (une fois).
    pret: bool,
    /// Pas où chaque genre est de nouveau prêt.
    pub(super) recharges: [u16; 5],
    /// La frappe visée d'un combo : son début et la place visée…
    pub(super) combo: Option<(u16, Aabb)>,
    /// … ou le pas où elle partira.
    pub(super) combo_prevu: Option<u16>,
    /// Pépin au pas d'avant (sa vitesse) et sa vitesse (px par pas).
    avant: Option<Aabb>,
    pub(super) vitesse: (f32, f32),
    /// La part de la tour gravie par Pépin (de 0 à 1), son plus haut.
    pub(super) progres: f32,
    /// À chaque nouvelle hauteur : (pieds de Pépin, têtes qu'il lui restait).
    /// Une reprise y retrouve l'état au passage de sa lanterne.
    pub(super) marques: Vec<(f32, u8)>,
    /// Les places de jeté déjà tentées pendant cette tentative.
    pub(super) jetes_tentes: Vec<usize>,
    /// Les attaques choisies, dans l'ordre (pas, genre) : pour les tests.
    journal: Vec<(u16, Genre)>,
    /// Carte 9.13f : les étapes qui restent du cycle de la vigie en cours.
    pub(super) suite_vigie: Vec<EtapeVigie>,
    /// Le balayeur de la vigie (dans `Tour::balayeurs`), réservé à ses cycles.
    balai_vigie: Option<usize>,
}

/// Une étape d'un cycle de la vigie (carte 9.13f).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtapeVigie {
    /// Un jeté lent (cabre 1,2 s) qui ne la laisse pas sonnée.
    Jete,
    /// Le jeté vif (cabre 0,8 s) qui la laisse sonnée : la fin de chaque cycle.
    JeteVif,
    /// Une frappe visée sur l'appui de Pépin.
    Frappe,
    /// Le balayage de la vigie.
    Balayage,
}

/// Les trois cycles de la vigie (carte 9.13f, AGA : « la vigie demande six
/// jetés identiques »). Le cycle dépend des têtes qui lui restent : avec 3
/// têtes, deux jetés ; avec 2, jeté, frappe visée, jeté ; avec la dernière,
/// balayage puis jeté vif. Chaque tête demande donc un enchaînement différent
/// du précédent. Toutes les annonces durent au moins 0,8 s (le jeté vif).
pub fn cycle_de_la_vigie(tetes: u8) -> &'static [EtapeVigie] {
    use EtapeVigie::*;
    match tetes {
        3.. => &[Jete, JeteVif],
        2 => &[Jete, Frappe, JeteVif],
        _ => &[Balayage, JeteVif],
    }
}

impl Cerveau {
    /// Tout ce qui ne dure qu'une tentative repart à neuf ; le générateur
    /// repart de la graine mêlée au numéro de la tentative.
    pub(super) fn oublier(&mut self) {
        self.recharges = [0; 5];
        self.combo = None;
        self.combo_prevu = None;
        self.avant = None;
        self.vitesse = (0.0, 0.0);
        self.progres = 0.0;
        self.marques.clear();
        self.jetes_tentes.clear();
        self.journal.clear();
        self.suite_vigie.clear();
        let g = self.graine.unwrap_or(GRAINE) ^ self.tentative.wrapping_mul(0x9E37_79B9);
        // Le xorshift reste bloqué à 0 : jamais de graine nulle.
        self.tirage = g.max(1);
        for _ in 0..4 {
            self.suivant();
        }
    }

    /// Le nombre pseudo-aléatoire suivant (xorshift32 de Marsaglia).
    fn suivant(&mut self) -> u32 {
        let mut x = self.tirage;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.tirage = x;
        x
    }

    /// Un nombre de 0 à `n - 1`.
    fn tirer(&mut self, n: u32) -> u32 {
        self.suivant() % n.max(1)
    }
}

impl Pieuvre {
    /// Les tests choisissent la graine : la même graine, la même montée.
    pub fn fixer_graine(&mut self, graine: u32) {
        self.tour.cerveau.graine = Some(graine);
        self.tour.cerveau.oublier();
    }

    /// Les attaques choisies depuis le début de la tentative : (pas, genre).
    pub fn journal(&self) -> &[(u16, Genre)] {
        &self.tour.cerveau.journal
    }

    /// Le numéro de la tentative (0 au début de la phase 2, +1 à chaque reprise).
    pub fn tentative(&self) -> u32 {
        self.tour.cerveau.tentative
    }

    /// Son humeur : énervée après le 1er coup reçu ou passé 50 % de la tour,
    /// furieuse après le 2e ou passé 80 %.
    pub fn humeur(&self) -> Humeur {
        let coups = self.pv_de_la_fuite().saturating_sub(self.pv);
        let p = self.tour.cerveau.progres;
        if coups >= 2 || p >= 0.8 {
            Humeur::Furieuse
        } else if coups >= 1 || p >= 0.5 {
            Humeur::Enervee
        } else {
            Humeur::Calme
        }
    }

    /// La part de la tour sous des pieds en `bas` (px) : 0 sur la bâche, 1 sur
    /// la vigie.
    pub(super) fn progres_a(&self, bas: f32) -> f32 {
        let (Some(bache), Some(vigie)) = (self.bache, self.tete_finale) else {
            return 0.0;
        };
        let h = (bache.y - vigie.bas()).max(1.0);
        ((bache.y - bas) / h).clamp(0.0, 1.0)
    }

    /// Une recharge selon l'humeur et le mode facile (pas).
    pub(super) fn recharge(&self, g: Genre) -> u16 {
        let base = u32::from(RECHARGES[g.rang()]);
        let r = base * FACTEUR_HUMEUR[self.humeur() as usize] / 100;
        self.lent(r.min(u32::from(u16::MAX)) as u16)
    }

    /// Le souffle après une attaque (pas) : selon l'humeur, et un peu de hasard.
    pub(super) fn souffle(&mut self) -> u16 {
        let s = SOUFFLE[self.humeur() as usize];
        let hasard = self.tour.cerveau.tirer(u32::from(s / 2) + 1) as u16;
        self.lent(s + hasard)
    }

    /// Une fois au début de la montée : les places de jeté (le bout droit de
    /// chaque corniche LARGE de la tour, hors la vigie et la corniche du bas) et
    /// les balayeurs (chaque corniche qui touche son couloir), en plus de ceux
    /// que la grille marque (`$`, `!`).
    pub(super) fn preparer_la_tour(&mut self, grille: &dyn Grille) {
        if self.tour.cerveau.pret {
            return;
        }
        self.tour.cerveau.pret = true;
        let (Some((bord, _)), Some(bache)) = (self.tour.couloir, self.bache) else {
            return;
        };
        let t = TAILLE_TUILE;
        let (w, h) = TAILLE_GEANTE;
        let cases_h = (h / t).ceil() as i32;
        let cases_l = (w / t).ceil() as i32;
        let solide = |x: i32, y: i32| grille.tuile(x, y) == Tuile::Solide;
        let fin = (bord / t) as i32;
        let bas = (bache.y / t) as i32;
        let vigie = self.tete_finale.map(|v| v.bas());
        for y in cases_h..bas {
            let mut x = 1;
            while x < fin {
                if !(solide(x, y) && !solide(x, y - 1)) {
                    x += 1;
                    continue;
                }
                let a = x;
                while x < fin && solide(x, y) && !solide(x, y - 1) {
                    x += 1;
                }
                let (dessus, gauche, droite) = (y as f32 * t, a as f32 * t, x as f32 * t);
                let libre = (y - cases_h..y).all(|yy| (x - cases_l..x).all(|xx| !solide(xx, yy)));
                let jete = x - a >= JETE_LARGE
                    && libre
                    && y + 3 < bas
                    && vigie.is_none_or(|v| (v - dessus).abs() > 1.0);
                let place = Aabb::new(droite - w, dessus - h, w, h);
                if jete && !self.tour.jetes.iter().any(|j| j.chevauche(&place)) {
                    self.tour.jetes.push(place);
                }
                let touche =
                    x == fin && x - a >= 3 && vigie.is_none_or(|v| (v - dessus).abs() > 1.0);
                if touche
                    && !self
                        .tour
                        .balayeurs
                        .iter()
                        .any(|b| (b.y - dessus).abs() < 1.0)
                {
                    self.tour.balayeurs.push(Balayeur {
                        y: dessus,
                        depart: droite,
                        arrivee: gauche,
                    });
                }
            }
        }
        // Carte 9.13f : le balayeur de la vigie (le bas de sa tête posée est le
        // dessus de la corniche), du couloir jusqu'au bout gauche de la vigie.
        if let Some(v) = vigie {
            let y = (v / t).round() as i32;
            let mut a = fin;
            while a > 1 && solide(a - 1, y) && !solide(a - 1, y - 1) {
                a -= 1;
            }
            if fin - a >= 3 {
                self.tour.cerveau.balai_vigie = Some(self.tour.balayeurs.len());
                self.tour.balayeurs.push(Balayeur {
                    y: v,
                    depart: fin as f32 * t,
                    arrivee: a as f32 * t,
                });
            }
        }
        // Du bas vers le haut, comme le parseur.
        self.tour.jetes.sort_by(|a, b| b.y.total_cmp(&a.y));
    }

    /// Ce qu'elle voit à chaque pas : la vitesse de Pépin, son plus haut, et
    /// une marque à chaque nouvelle hauteur.
    pub(super) fn observer(&mut self, pepin: &Aabb) {
        let progres = self.progres_a(pepin.bas());
        let pv = self.pv;
        let c = &mut self.tour.cerveau;
        c.vitesse = c
            .avant
            .map_or((0.0, 0.0), |a| (pepin.x - a.x, pepin.bas() - a.bas()));
        c.avant = Some(*pepin);
        c.progres = c.progres.max(progres);
        // Les marques montent seulement (le plus haut atteint), et une de plus
        // quand elle perd une tête.
        let plus_haut = c.marques.iter().map(|m| m.0).fold(f32::MAX, f32::min);
        let tete_perdue = c.marques.last().is_some_and(|m| m.1 != pv);
        if c.marques.is_empty() || pepin.bas() < plus_haut - 4.0 || tete_perdue {
            c.marques.push((pepin.bas().min(plus_haut), pv));
        }
    }

    /// Où Pépin VA ATTERRIR (une boîte de sa taille, les pieds sur l'appui) :
    /// arrêté sur un appui, là où il est (on punit qui s'arrête) ; sinon
    /// l'appui le plus proche AU-DESSUS de ses pieds (à portée de saut), du
    /// côté où il va, ou, s'il tombe, le premier appui dessous ; à défaut, la
    /// suite de sa course. Pure (sauf la vitesse vue au pas d'avant).
    pub fn arrivee_prevue(&self, grille: &dyn Grille, pepin: &Aabb) -> Aabb {
        let (vx, vy) = self.tour.cerveau.vitesse;
        let appuis = self.appuis(grille);
        let milieu = pepin.x + pepin.w / 2.0;
        let sur = |&(y, a, b): &(f32, f32, f32)| {
            (pepin.bas() - y).abs() < 1.5 && a < pepin.droite() && b > pepin.x
        };
        let au_sol = vy.abs() < 0.01 && appuis.iter().any(sur);
        if au_sol && vx.abs() < 0.1 {
            return *pepin;
        }
        let sens = if vx > 0.1 {
            1.0
        } else if vx < -0.1 {
            -1.0
        } else {
            0.0
        };
        let de_cote = |a: f32, b: f32| (a - milieu).max(milieu - b).max(0.0);
        let devant =
            |a: f32, b: f32| sens == 0.0 || (b - milieu) * sens > 0.0 || (a - milieu) * sens > 0.0;
        let tombe = vy > 0.5;
        let candidat = |&&(y, a, b): &&(f32, f32, f32)| {
            let monte = y < pepin.bas() - 2.0 && y >= pepin.bas() - SAUT_MAX;
            let dessous = tombe && y >= pepin.bas() - 2.0 && y <= pepin.bas() + SAUT_MAX;
            (monte || dessous) && de_cote(a, b) <= PORTEE_VISEE && devant(a, b)
        };
        let cout = |&(y, a, b): &(f32, f32, f32)| (y - pepin.bas()).abs() + de_cote(a, b);
        let vise = appuis
            .iter()
            .filter(candidat)
            .min_by(|p, q| cout(p).total_cmp(&cout(q)))
            .copied();
        let projete = milieu + vx * AVANCE_PAS;
        let Some((y, a, b)) = vise else {
            // Pas d'appui en vue : la suite de sa course, sur son appui.
            let x = projete - pepin.w / 2.0;
            return Aabb::new(x.round(), pepin.y, pepin.w, pepin.h);
        };
        let (g, d) = (a + JOUEUR_L / 2.0 + 1.0, b - JOUEUR_L / 2.0 - 1.0);
        let x = if g <= d {
            projete.clamp(g, d)
        } else {
            (a + b) / 2.0
        };
        Aabb::new(
            (x - JOUEUR_L / 2.0).round(),
            y - JOUEUR_H,
            JOUEUR_L,
            JOUEUR_H,
        )
    }

    /// L'attaque qu'elle choisit au pas `t`, Pépin en `pepin` (`None` : rien de
    /// possible ou de prêt). Sur la vigie, toujours le jeté (le coup final).
    pub(super) fn choisir(&mut self, grille: &dyn Grille, pepin: &Aabb, t: u16) -> Option<Attaque> {
        let mer = self.surface_mer().unwrap_or(f32::MAX);
        let debout_sur = |y: f32| pepin.bas() <= y + 1.0 && pepin.bas() >= y - 2.0;
        let vigie = self.tour.jetes.len();
        if let Some(place) = self.place_du_jete(vigie) {
            if debout_sur(place.bas()) && pepin.x < place.droite() && place.bas() < mer {
                // Carte 9.13f : un cycle par tête, chacun différent.
                let mut suite = cycle_de_la_vigie(self.pv).to_vec();
                suite.reverse();
                self.tour.cerveau.suite_vigie = suite;
                return self.etape_suivante_de_la_vigie(grille, t);
            }
        }
        let pret = |g: Genre| t >= self.tour.cerveau.recharges[g.rang()];
        let mut options: Vec<(Genre, Attaque)> = Vec::new();
        // Le jeté : il lui faut 2 têtes (la dernière est pour la vigie).
        if pret(Genre::Jete) && self.pv > 1 {
            for (k, place) in self.tour.jetes.iter().enumerate() {
                let sur = debout_sur(place.bas())
                    && pepin.x < place.droite()
                    && pepin.droite() > place.x - 6.0 * TAILLE_TUILE;
                if sur && place.bas() < mer && !self.tour.cerveau.jetes_tentes.contains(&k) {
                    let a = Attaque::Jete {
                        k,
                        debut: t,
                        vif: false,
                        sonne: true,
                    };
                    options.push((Genre::Jete, a));
                }
            }
        }
        if pret(Genre::Balayage) {
            for (k, b) in self.tour.balayeurs.iter().enumerate() {
                if Some(k) == self.tour.cerveau.balai_vigie {
                    continue;
                }
                let sur = pepin.bas() <= b.y + 1.0 && pepin.bas() >= b.y - 2.0 * TAILLE_TUILE;
                if sur && pepin.x < b.depart && pepin.droite() > b.arrivee && b.y < mer {
                    options.push((Genre::Balayage, Attaque::Balayage { k, debut: t }));
                }
            }
        }
        let au_sol = self.tour.cerveau.vitesse.1.abs() < 0.01
            && self.appuis(grille).iter().any(|&(y, a, b)| {
                (pepin.bas() - y).abs() < 1.5 && a < pepin.droite() && b > pepin.x
            });
        if pret(Genre::Secousse) && au_sol {
            let caisses = self.caisses_pour(grille, pepin, mer);
            if caisses.len() >= 2 {
                options.push((Genre::Secousse, Attaque::Secousse { debut: t, caisses }));
            }
        }
        if pret(Genre::Arrachage) {
            self.tour
                .arrachages
                .resize(self.tour.arrachables.len(), None);
            let milieu = pepin.x + pepin.w / 2.0;
            for (k, cases) in self.tour.arrachables.iter().enumerate() {
                if self.tour.arrachages[k].is_some() {
                    continue;
                }
                let (g, d) = super::super::etendue(cases);
                let haut = cases.iter().map(|c| c.1).min().unwrap_or(0) as f32 * TAILLE_TUILE;
                let sous = pepin.bas() - haut;
                let cote = (g - milieu).max(milieu - d).max(0.0);
                if sous >= ARRACHAGE_SOUS.0 && sous <= ARRACHAGE_SOUS.1 && cote < ARRACHAGE_COTE {
                    options.push((Genre::Arrachage, Attaque::Arrachage { k, debut: t }));
                }
            }
        }
        if pret(Genre::Frappe) {
            let arrivee = self.arrivee_prevue(grille, pepin);
            let cible = self.viser(grille, &arrivee);
            options.push((Genre::Frappe, Attaque::Frappe { debut: t, cible }));
        }
        // Le poids de chaque genre grandit avec le temps où il attend, prêt : une
        // attaque possible finit toujours par venir (une « faim » d'au plus x8).
        let faim = |g: Genre| {
            let attente = t.saturating_sub(self.tour.cerveau.recharges[g.rang()]) / 60;
            POIDS[g.rang()] * (1 + u32::from(attente).min(7))
        };
        let poids: Vec<u32> = options.iter().map(|(g, _)| faim(*g)).collect();
        let total: u32 = poids.iter().sum();
        if total == 0 {
            return None;
        }
        let mut n = self.tour.cerveau.tirer(total);
        let mut rang = 0;
        while rang + 1 < poids.len() && n >= poids[rang] {
            n -= poids[rang];
            rang += 1;
        }
        let (genre, attaque) = options.swap_remove(rang);
        let recharge = self.recharge(genre);
        self.tour.cerveau.recharges[genre.rang()] = t.saturating_add(recharge);
        self.tour.cerveau.journal.push((t, genre));
        match &attaque {
            Attaque::Jete { k, .. } => {
                self.tour.cerveau.jetes_tentes.push(*k);
                self.tour.faits += 1;
            }
            Attaque::Arrachage { k, .. } => self.tour.arrachages[*k] = Some(t),
            _ => {}
        }
        self.preparer_le_combo(&attaque, t);
        Some(attaque)
    }

    /// Dès qu'elle est énervée (une fois sur deux ; furieuse, toujours), un
    /// balayage ou une secousse s'enchaîne d'une frappe visée : son annonce
    /// (1,4 s) se finit juste quand la partie mortelle de la 1re attaque est
    /// passée, pour que les deux ne tuent jamais en même temps.
    fn preparer_le_combo(&mut self, attaque: &Attaque, t: u16) {
        let mortelle = match attaque {
            Attaque::Balayage { k, .. } => {
                let course = self.tour.balayeurs[*k].course_pas();
                self.lent(ANNONCE_BALAI_PAS) + course
            }
            Attaque::Secousse { caisses, .. } => {
                let chute = caisses.iter().map(|c| c.chute_pas()).max().unwrap_or(0);
                self.lent(ANNONCE_SECOUSSE_PAS) + chute
            }
            _ => return,
        };
        let enchaine = match self.humeur() {
            Humeur::Calme => false,
            Humeur::Enervee => self.tour.cerveau.tirer(2) == 0,
            Humeur::Furieuse => true,
        };
        if enchaine {
            let debut = (mortelle + 1)
                .saturating_sub(self.lent(ANNONCE_FRAPPE_PAS))
                .max(1);
            self.tour.cerveau.combo_prevu = Some(t.saturating_add(debut));
        }
    }

    /// Un pas du combo : la frappe prévue part (elle vise l'arrivée de Pépin à
    /// ce moment-là), puis s'en va. Vrai tant qu'un combo occupe la géante.
    pub(super) fn avancer_le_combo(&mut self, grille: &dyn Grille, t: u16) -> bool {
        if let Some(d) = self.tour.cerveau.combo_prevu {
            if t >= d {
                self.tour.cerveau.combo_prevu = None;
                if let Some(pepin) = self.pepin {
                    let arrivee = self.arrivee_prevue(grille, &pepin);
                    let cible = self.viser(grille, &arrivee);
                    self.tour.cerveau.combo = Some((t, cible));
                    self.tour.cerveau.journal.push((t, Genre::Frappe));
                }
            }
        }
        if let Some((d, _)) = self.tour.cerveau.combo {
            if t >= d.saturating_add(self.duree_frappe()) {
                self.tour.cerveau.combo = None;
                let s = self.souffle();
                self.tour.repit = self.tour.repit.max(t.saturating_add(s));
            }
        }
        self.tour.cerveau.combo.is_some() || self.tour.cerveau.combo_prevu.is_some()
    }

    /// Plus de combo (un coup sur sa tête, une reprise) ; le cycle de la vigie
    /// en cours s'arrête aussi.
    pub(super) fn annuler_le_combo(&mut self) {
        self.tour.cerveau.combo = None;
        self.tour.cerveau.combo_prevu = None;
        self.tour.cerveau.suite_vigie.clear();
    }

    /// La prochaine étape du cycle de la vigie, lancée au pas `t` (`None` : le
    /// cycle est fini). Pépin est sur la vigie ; la frappe vise son appui.
    pub(super) fn etape_suivante_de_la_vigie(
        &mut self,
        grille: &dyn Grille,
        t: u16,
    ) -> Option<Attaque> {
        let etape = self.tour.cerveau.suite_vigie.pop()?;
        let k = self.tour.jetes.len();
        let (genre, attaque) = match (etape, self.tour.cerveau.balai_vigie) {
            (EtapeVigie::Frappe, _) => {
                let pepin = self.pepin?;
                let arrivee = self.arrivee_prevue(grille, &pepin);
                let cible = self.viser(grille, &arrivee);
                (Genre::Frappe, Attaque::Frappe { debut: t, cible })
            }
            (EtapeVigie::Balayage, Some(b)) => {
                (Genre::Balayage, Attaque::Balayage { k: b, debut: t })
            }
            (EtapeVigie::JeteVif, _) => (
                Genre::Jete,
                Attaque::Jete {
                    k,
                    debut: t,
                    vif: true,
                    sonne: true,
                },
            ),
            // Le jeté lent (ou, sans balayeur de vigie, à la place du balayage).
            _ => (
                Genre::Jete,
                Attaque::Jete {
                    k,
                    debut: t,
                    vif: false,
                    sonne: false,
                },
            ),
        };
        if genre == Genre::Jete {
            self.tour.faits += 1;
        }
        self.tour.cerveau.journal.push((t, genre));
        Some(attaque)
    }
}
