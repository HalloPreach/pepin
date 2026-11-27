//! La MONTÉE de la phase 2 : la mer qui avance, les objets que la Pieuvre lance
//! de loin, et sa fuite au sommet (carte 9.36a).
//!
//! Décision d'AGA (27/11) : « je pense supprimer la p2 telle qu'elle est
//! actuellement, et juste faire une montée contre l'eau qui avance plus vite
//! avec des objets qui tombent du ciel envoyés par la pieuvre. on ne la tue pas
//! à la fin, comme le nuage juste elle finit par s'enfuir ».
//!
//! ```text
//!   mur │            le SOMMET (`&`)              │ mur
//!       │  corniches, caisses, lanternes          │
//!       │        ▼ caisse, tonneau, épave         │
//!       │        ░ l'ombre annonce la chute       │
//!   ~~~~~~~~~~~~~~ la mer qui monte ~~~~~~~ (Pieuvre au loin) ~~~
//! ```
//!
//! - LA MER est la menace principale (`fuite::surface_mer_a`, une tuile par
//!   seconde). Elle SUIT Pépin : jamais plus de `ECART_MER_MAX` (8 tuiles, 10
//!   en facile) sous le point le plus haut qu'il a atteint ; au-delà, elle
//!   rattrape à vitesse TRIPLE, sans saut brusque, mais jamais à moins de 4
//!   tuiles de ses pieds (`avancer_la_mer`). Elle ne descend jamais, sauf à la
//!   fuite.
//! - LES OBJETS QUI TOMBENT (`Chute`) : caisses, tonneaux, morceaux d'épave,
//!   lancés par la Pieuvre au loin (`lancer`). Chaque chute est ANNONCÉE par
//!   son ombre sur la corniche où elle va se briser (`ANNONCE_CHUTE_PAS`, 1 s,
//!   au moins 0,8 s : `ANNONCE_MIN_PAS`), puis l'objet tombe (mortel au
//!   contact) et se brise (sans danger), puis disparaît. Le premier vise la
//!   colonne de Pépin, le second (passé la moitié de la tour) une colonne
//!   proche ; leur rythme augmente avec la hauteur (`intervalle`). Il y a
//!   toujours une issue : sur une corniche, les objets en cours laissent au
//!   moins `PLACE_LIBRE_MIN` de suite sans rien (`laisse_une_issue`). C'est la
//!   mécanique des caisses de l'ancienne secousse (carte 9.13c), lancées
//!   désormais de loin.
//! - LA PIEUVRE AU LOIN (`au_loin`) : sa tête de la phase 1, à moitié dans la
//!   mer, à droite de la tour. C'est elle qui lance (`lance`). Elle n'est pas
//!   attaquable et ne touche jamais Pépin (aucune boîte mortelle).
//! - LA FIN : Pépin entre dans le SOMMET (`Pieuvre::sommet`, la plateforme du
//!   haut) : elle abandonne et S'ENFUIT (`s_enfuir` : elle plonge au large, la
//!   mer redescend, `PhasePieuvre::Plongee`), puis elle est « vaincue » et le
//!   chapitre est fini comme avant. On ne la tue pas.
//!
//! Une mort ramène à la dernière lanterne : la mer est remise sous elle
//! (`MER_A_LA_REPRISE`), les objets en cours disparaissent et le tirage change
//! (`reprendre_a`). MODE FACILE : la mer monte 0,75 fois moins vite
//! (`fuite::FACTEUR_FACILE`), les objets sont annoncés plus longtemps, plus
//! espacés, et jamais deux à la fois.
//!
//! Notions Rust : un `enum` dont chaque variante porte ses données
//! (`EtatChute::Chute(Aabb)`) et un `match` qui les déballe ; des fonctions
//! PURES du temps (l'état d'une chute se calcule depuis son pas de départ) ;
//! `Vec::retain` pour oublier les objets brisés.

use super::fuite::FACTEUR_FACILE;
use super::{PhasePieuvre, Pieuvre, FURIE_PAS, MARGE_TENTACULE, NB_COLONNES, TAILLE_TETE};
use crate::aabb::{Aabb, Grille, Tuile, TAILLE_TUILE};
use crate::physics::Environnement;

/// Le sommet (carte 9.36a) : la zone d'arrivée de 4 x 3 cases (px), son coin
/// bas-gauche sur le `&`.
pub const TAILLE_SOMMET: (f32, f32) = (64.0, 48.0);

/// LA MER SUIT PÉPIN (carte 9.13f, resserrée par la 9.36a) : jamais plus de 8
/// tuiles sous le point le plus haut qu'il a atteint (10 en mode facile)…
pub const ECART_MER_MAX: f32 = 8.0 * TAILLE_TUILE;
pub const ECART_MER_MAX_FACILE: f32 = 10.0 * TAILLE_TUILE;
/// … au-delà, elle rattrape à vitesse triple…
pub const RATTRAPAGE_MER: f32 = 3.0;
/// … mais jamais à moins de 4 tuiles des pieds de Pépin (le rattrapage ne tue
/// pas : s'il est retombé, elle reprend sa vitesse normale).
pub const RATTRAPAGE_PAS_SOUS: f32 = 4.0 * TAILLE_TUILE;
/// À la reprise à une lanterne, la mer est remise 4 tuiles sous elle.
pub const MER_A_LA_REPRISE: f32 = 4.0 * TAILLE_TUILE;

/// Une chute s'annonce 1 s (l'ombre sur la corniche)…
pub const ANNONCE_CHUTE_PAS: u16 = 60;
/// … jamais moins de 0,8 s (la règle de la carte, vérifiée par les tests).
pub const ANNONCE_MIN_PAS: u16 = 48;
/// L'objet tombe de 5 px par pas…
pub const VITESSE_CHUTE: f32 = 5.0;
/// … depuis 9 tuiles au-dessus des pieds de Pépin (le haut de l'écran), ou de
/// dessous la corniche qui le couvre.
pub const HAUT_CHUTE: f32 = 9.0 * TAILLE_TUILE;
/// Un objet fait une tuile.
pub const TAILLE_OBJET: f32 = TAILLE_TUILE;
/// Brisé, il reste 0,3 s en éclats (sans danger), puis disparaît.
pub const BRIS_PAS: u16 = 18;
/// Le geste de la Pieuvre qui lance : 0,5 s.
pub const LANCER_PAS: u16 = 30;
/// Le premier lancer, 2 s après le début de la montée (Pépin a quitté le bas).
pub const PREMIER_LANCER_PAS: u16 = 120;
/// Un lancer toutes les 2,8 s en bas de la tour, toutes les 1,8 s en haut (à
/// 20 % près, tiré au sort).
pub const INTERVALLE_BAS: u16 = 168;
pub const INTERVALLE_HAUT: u16 = 108;
/// Passé 60 % de la tour, deux objets par lancer (pas en facile).
pub const DOUBLE_DES: f32 = 0.6;
/// Le second objet tombe entre 2 et 5 colonnes de Pépin…
pub const ECART_OBJETS: i32 = 5;
/// … sur un sol au plus 3 tuiles sous ses pieds (son ombre se voit).
pub const SOL_OBJETS: f32 = 3.0 * TAILLE_TUILE;
/// Au plus 3 objets en même temps.
pub const MAX_EN_COURS: usize = 3;
/// L'issue : sur la corniche d'arrivée, toujours 2 cases de suite sans objet.
pub const PLACE_LIBRE_MIN: usize = 2;

/// La graine du tirage (les colonnes et les objets), mêlée au numéro de la
/// tentative : après une mort, la montée n'est plus tout à fait la même.
const GRAINE: u32 = 0x5EED_9036;

/// Ce qui tombe : le dessin seulement (même taille, même danger).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Objet {
    Caisse,
    Tonneau,
    Epave,
}

/// Une chute : l'objet, sa colonne (`x`, px), d'où il part, le sol où il se
/// brise (le dessus d'une case solide, ou la mer), le pas de son lancer et la
/// durée de son annonce.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Chute {
    pub objet: Objet,
    pub x: f32,
    pub depart: f32,
    pub sol: f32,
    pub debut: u16,
    pub annonce: u16,
}

/// Où en est une chute.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EtatChute {
    /// L'ombre sur le sol, de 0 à 1 (elle grandit).
    Annonce(f32),
    /// L'objet tombe : sa boîte, mortelle.
    Chute(Aabb),
    /// Brisé sur son sol, de 0 à 1 (les éclats), sans danger.
    Bris(f32),
}

impl Chute {
    /// Pas de chute, du départ jusqu'au sol.
    pub fn chute_pas(&self) -> u16 {
        ((self.sol - TAILLE_OBJET - self.depart).max(0.0) / VITESSE_CHUTE).ceil() as u16
    }

    /// Le pas où il commence à tomber (la fin de l'annonce).
    pub fn lache(&self) -> u16 {
        self.debut.saturating_add(self.annonce)
    }

    /// Le pas où il se brise sur son sol.
    pub fn arrivee(&self) -> u16 {
        self.lache().saturating_add(self.chute_pas())
    }

    /// Le pas où ses éclats disparaissent.
    pub fn fin(&self) -> u16 {
        self.arrivee().saturating_add(BRIS_PAS)
    }

    /// La case (colonne) où il tombe.
    pub fn colonne(&self) -> i32 {
        (self.x / TAILLE_TUILE).round() as i32
    }

    /// L'ombre : la place de l'objet posé sur son sol.
    pub fn ombre(&self) -> Aabb {
        Aabb::new(self.x, self.sol - TAILLE_OBJET, TAILLE_OBJET, TAILLE_OBJET)
    }

    /// Son état au pas `t` (`None` avant son lancer ou après ses éclats).
    /// Fonction pure du temps.
    pub fn etat(&self, t: u16) -> Option<EtatChute> {
        if t < self.debut || t >= self.fin() {
            return None;
        }
        if t < self.lache() {
            return Some(EtatChute::Annonce(
                f32::from(t - self.debut) / f32::from(self.annonce.max(1)),
            ));
        }
        if t < self.arrivee() {
            let y = self.depart + f32::from(t - self.lache()) * VITESSE_CHUTE;
            return Some(EtatChute::Chute(Aabb::new(
                self.x,
                y,
                TAILLE_OBJET,
                TAILLE_OBJET,
            )));
        }
        Some(EtatChute::Bris(
            f32::from(t - self.arrivee()) / f32::from(BRIS_PAS),
        ))
    }

    /// Sa boîte mortelle au pas `t`, pendant qu'il tombe.
    pub fn boite(&self, t: u16) -> Option<Aabb> {
        match self.etat(t) {
            Some(EtatChute::Chute(b)) => Some(b),
            _ => None,
        }
    }
}

/// La grille de la tour, avec les solides des entités (les nuages friables) :
/// un objet se brise aussi sur eux, et leur dessus compte comme une corniche.
pub struct GrilleDeLaTour<'a> {
    pub grille: &'a dyn Grille,
    pub solides: &'a [Aabb],
}

impl Grille for GrilleDeLaTour<'_> {
    fn tuile(&self, tx: i32, ty: i32) -> Tuile {
        let t = TAILLE_TUILE;
        let (cx, cy) = ((tx as f32 + 0.5) * t, (ty as f32 + 0.5) * t);
        let dedans = |b: &Aabb| cx > b.x && cx < b.droite() && cy > b.y && cy < b.bas();
        if self.grille.est_solide(tx, ty) || self.solides.iter().any(dedans) {
            Tuile::Solide
        } else {
            Tuile::Vide
        }
    }
}

/// Tout ce que la montée ajoute à la Pieuvre : l'état de la mer et des objets.
/// Remis à neuf au début de la phase 2 et à chaque reprise.
#[derive(Debug, Clone, Default)]
pub struct Tour {
    /// Pour le robot de faisabilité : rien ne tombe, la mer ne monte pas.
    robot: bool,
    /// L'horloge de la mer (pas de `surface_mer_a`).
    horloge_mer: u16,
    /// Les pieds de Pépin au plus haut pendant cette tentative.
    plus_haut: Option<f32>,
    /// La mer rattrape-t-elle en ce moment (vitesse triple) ?
    rattrape: bool,
    /// Les objets lancés, pas encore disparus.
    chutes: Vec<Chute>,
    /// Le pas du prochain lancer.
    prochain: u16,
    /// Le pas du dernier lancer (son geste, au loin).
    dernier_lancer: Option<u16>,
    /// Objets lancés pendant cette tentative.
    lances: u32,
    /// La graine choisie (`None` : `GRAINE`).
    graine: Option<u32>,
    /// Nombre de reprises depuis le début de la phase 2.
    tentative: u32,
    /// L'état du générateur (xorshift32).
    tirage: u32,
}

impl Tour {
    /// Le robot de faisabilité (`Pieuvre::pour_le_robot`).
    pub(super) fn pour_le_robot(&mut self) {
        self.robot = true;
    }

    /// Le générateur repart de la graine mêlée au numéro de la tentative.
    fn semer(&mut self) {
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
    /// L'horloge de la mer : `surface_mer_a(horloge_mer)` est la mer du moment.
    pub fn horloge_mer(&self) -> u16 {
        self.tour.horloge_mer
    }

    fn en_montee(&self) -> bool {
        self.en_fuite() && self.phase() == PhasePieuvre::Montee
    }

    /// Mode facile : une durée 0,75 fois moins rapide (plus longue).
    fn lent(&self, pas: u16) -> u16 {
        if self.est_facile() {
            (f32::from(pas) / FACTEUR_FACILE) as u16
        } else {
            pas
        }
    }

    /// La boîte solide de la pieuvre : la bâche, tant que dure la phase 1.
    pub fn solide(&self) -> Option<Aabb> {
        self.bache_solide()
    }

    /// Les tests choisissent la graine : la même graine, la même montée.
    pub fn fixer_graine(&mut self, graine: u32) {
        self.tour.graine = Some(graine);
        self.tour.semer();
    }

    /// Le numéro de la tentative (0 au début de la phase 2, +1 à chaque reprise).
    pub fn tentative(&self) -> u32 {
        self.tour.tentative
    }

    /// Les objets lancés pendant la montée, pas encore disparus.
    pub fn chutes(&self) -> &[Chute] {
        if self.en_montee() {
            &self.tour.chutes
        } else {
            &[]
        }
    }

    /// Objets lancés depuis le début de la tentative.
    pub fn objets_lances(&self) -> u32 {
        self.tour.lances
    }

    /// La part de la tour sous des pieds en `bas` (px) : 0 sur la bâche, 1 au
    /// sommet.
    pub fn progres_a(&self, bas: f32) -> f32 {
        let (Some(bache), Some(sommet)) = (self.bache, self.sommet) else {
            return 0.0;
        };
        ((bache.y - bas) / (bache.y - sommet.bas()).max(1.0)).clamp(0.0, 1.0)
    }

    /// Pas entre deux lancers quand Pépin est à la hauteur `progres` (de 0 à
    /// 1) : de `INTERVALLE_BAS` à `INTERVALLE_HAUT`, plus long en facile.
    pub fn intervalle(&self, progres: f32) -> u16 {
        let (bas, haut) = (f32::from(INTERVALLE_BAS), f32::from(INTERVALLE_HAUT));
        let i = (bas + (haut - bas) * progres.clamp(0.0, 1.0)).round() as u16;
        self.lent(i)
    }

    /// LA PIEUVRE AU LOIN (pour le dessin) : sa tête de la phase 1 (48 x 32), à
    /// moitié dans la mer, à droite de la tour. Pendant la furie, elle y va
    /// depuis sa place dans l'arène ; à la fuite, elle s'enfonce. `None` hors de
    /// la phase 2 et une fois partie. Aucune boîte mortelle : elle ne touche
    /// jamais Pépin.
    pub fn au_loin(&self) -> Option<Aabb> {
        self.sommet?;
        if !self.en_fuite() {
            return None;
        }
        let mer = self.surface_mer()?;
        let (w, h) = TAILLE_TETE;
        let x_loin = (self.colonnes[NB_COLONNES - 1].droite() + MARGE_TENTACULE - w - 8.0).round();
        let y_loin = (mer - h / 2.0).round();
        let ici = |k: f32| {
            let depart = self.places[self.place];
            let k = k.clamp(0.0, 1.0);
            Aabb::new(
                (depart.x + (x_loin - depart.x) * k).round(),
                (depart.y + (y_loin - depart.y) * k).round(),
                w,
                h,
            )
        };
        match self.phase() {
            PhasePieuvre::Furie => Some(ici(f32::from(self.ecoule()) / f32::from(FURIE_PAS))),
            PhasePieuvre::Montee => Some(ici(1.0)),
            PhasePieuvre::Plongee => {
                let k = self.progression_plongee();
                Some(Aabb::new(
                    x_loin,
                    (y_loin + k * (h / 2.0 + 2.0)).round(),
                    w,
                    h,
                ))
            }
            _ => None,
        }
    }

    /// Le geste du lancer, de 0 à 1, pendant `LANCER_PAS` après chaque lancer
    /// (`None` sinon) : le dessin lève un tentacule.
    pub fn lance(&self) -> Option<f32> {
        let d = self.ecoule().checked_sub(self.tour.dernier_lancer?)?;
        (self.en_montee() && d < LANCER_PAS).then(|| f32::from(d) / f32::from(LANCER_PAS))
    }

    /// Phase 2 : un objet qui tombe touche-t-il `boite` ? (La mer : `fuite`.)
    pub(super) fn blesse_en_montee(&self, boite: &Aabb) -> bool {
        let t = self.ecoule();
        self.chutes()
            .iter()
            .any(|c| c.boite(t).is_some_and(|b| b.chevauche(boite)))
    }

    /// Le sommet, pour le robot de faisabilité seulement (y entrer compte comme
    /// un coup sur la tête : `Pieuvre::recoit`).
    pub(super) fn sommet_pour_le_robot(&self) -> Option<Aabb> {
        // La pieuvre du robot n'est pas « en fuite » (la bâche est retirée à
        // part, la mer ne monte pas) : seule sa phase compte.
        self.sommet
            .filter(|_| self.tour.robot && self.phase() == PhasePieuvre::Montee)
    }

    /// LA FUITE : elle abandonne, plonge au large, la mer redescend ; puis elle
    /// est « vaincue » (le niveau est fini). Plus rien ne tombe.
    pub(super) fn s_enfuir(&mut self) {
        self.tour.chutes.clear();
        self.figer_la_mer();
        self.entrer(PhasePieuvre::Plongee);
    }

    /// Les colonnes (cases) de la tour : celles du bassin, entre les murs.
    fn colonnes_de_la_tour(&self) -> (i32, i32) {
        let t = TAILLE_TUILE;
        let x0 = ((self.colonnes[0].x - MARGE_TENTACULE) / t).round() as i32;
        let x1 = ((self.colonnes[NB_COLONNES - 1].droite() + MARGE_TENTACULE) / t).round() as i32;
        (x0, x1 - 1)
    }

    /// Un pas de la montée (après `avancer`) : la mer, les objets, le sommet.
    pub(super) fn avancer_tour(&mut self, env: &Environnement<'_>) {
        let grille = GrilleDeLaTour {
            grille: env.grille,
            solides: env.solides,
        };
        let grille = &grille;
        if !self.en_montee() || self.tour.robot {
            return;
        }
        let t = self.ecoule();
        if let Some(pepin) = self.pepin {
            let haut = self
                .tour
                .plus_haut
                .map_or(pepin.bas(), |y| y.min(pepin.bas()));
            self.tour.plus_haut = Some(haut);
        }
        self.tour.rattrape = false;
        self.avancer_la_mer();
        self.tour.chutes.retain(|c| t < c.fin());
        let Some(pepin) = self.pepin else {
            return;
        };
        // Le sommet atteint : elle s'enfuit.
        if self.sommet.is_some_and(|s| s.chevauche(&pepin)) {
            self.s_enfuir();
            return;
        }
        if t >= self.tour.prochain && self.horloge_mer() >= self.fin_de_crue() {
            self.lancer(grille, &pepin, t);
        }
    }

    /// Un lancer : un objet sur la colonne de Pépin (ou juste à côté) et, passé
    /// la moitié de la tour, un second sur une colonne proche ; jamais sans
    /// issue. Le prochain lancer suit l'intervalle de sa hauteur.
    fn lancer(&mut self, grille: &dyn Grille, pepin: &Aabb, t: u16) {
        let progres = self.progres_a(pepin.bas());
        // L'intervalle de sa hauteur, à 20 % près (le tirage) : un rythme qu'on
        // ne peut pas compter.
        let i = self.intervalle(progres);
        let ecart = u32::from(i / 5);
        let i = i - i / 5 + self.tour.tirer(2 * ecart + 1) as u16;
        self.tour.prochain = t.saturating_add(i);
        let Some(mer) = self.surface_mer() else {
            return;
        };
        let double = !self.est_facile() && progres >= DOUBLE_DES;
        let nouvelles = self.objets_pour(grille, pepin, mer, t, if double { 2 } else { 1 });
        if !nouvelles.is_empty() {
            self.tour.dernier_lancer = Some(t);
            self.tour.lances += nouvelles.len() as u32;
            self.tour.chutes.extend(nouvelles);
        }
    }

    /// Les objets d'un lancer au pas `t` (au plus `n`), sans les ajouter. Le
    /// premier sur la colonne de Pépin (ou une voisine, si la sienne ne laisse
    /// pas d'issue), les autres de 2 à `ECART_OBJETS` colonnes de lui, là où
    /// leur sol est au plus `SOL_OBJETS` sous ses pieds, jamais à moins de 2
    /// colonnes d'un autre objet en cours. Chacun laisse une issue sur sa
    /// corniche (`laisse_une_issue`).
    pub fn objets_pour(
        &mut self,
        grille: &dyn Grille,
        pepin: &Aabb,
        mer: f32,
        t: u16,
        n: usize,
    ) -> Vec<Chute> {
        let tt = TAILLE_TUILE;
        let (x0, x1) = self.colonnes_de_la_tour();
        let colonne = ((pepin.x + pepin.w / 2.0) / tt).floor() as i32;
        let rangee = (pepin.bas() / tt).floor() as i32 - 1;
        let haut_chute = pepin.bas() - HAUT_CHUTE;
        let haut = (haut_chute / tt).floor() as i32;
        let libre = |tx: i32, ty: i32| !grille.est_solide(tx, ty);
        // Le premier solide sous la rangée de Pépin (son dessus), ou la mer.
        let sol = |tx: i32| {
            (rangee..rangee + 40)
                .find(|&ty| grille.est_solide(tx, ty))
                .map_or(mer, |ty| ty as f32 * tt)
                .min(mer)
        };
        // Le plus bas solide au-dessus de la rangée de Pépin : le bas de sa case.
        let plafond = |tx: i32| {
            (haut..rangee)
                .rev()
                .find(|&ty| grille.est_solide(tx, ty))
                .map(|ty| (ty + 1) as f32 * tt)
        };
        let annonce = self.lent(ANNONCE_CHUTE_PAS);
        let mut objets: Vec<Chute> = Vec::new();
        let premiers = [0, -1, 1];
        let mut autres: Vec<i32> = (2..=ECART_OBJETS).flat_map(|k| [-k, k]).collect();
        if self.tour.tirer(2) == 1 {
            for paire in autres.chunks_mut(2) {
                paire.swap(0, 1);
            }
        }
        for (rang, dx) in premiers
            .iter()
            .map(|&d| (0, d))
            .chain(autres.iter().map(|&d| (1, d)))
        {
            if objets.len() >= n || self.tour.chutes.len() + objets.len() >= MAX_EN_COURS {
                break;
            }
            // Le premier objet n'a qu'une place (sa colonne ou une voisine).
            if rang == 0 && !objets.is_empty() {
                continue;
            }
            if rang == 1 && objets.is_empty() {
                break;
            }
            let tx = colonne + dx;
            if tx < x0 || tx > x1 || !libre(tx, rangee) {
                continue;
            }
            let loin = self
                .tour
                .chutes
                .iter()
                .chain(&objets)
                .all(|c| (c.colonne() - tx).abs() >= 2);
            let s = sol(tx);
            let proche = rang == 0 || (s <= pepin.bas() + SOL_OBJETS && plafond(tx).is_none());
            if !loin || !proche {
                continue;
            }
            let objet = match self.tour.tirer(3) {
                0 => Objet::Caisse,
                1 => Objet::Tonneau,
                _ => Objet::Epave,
            };
            let c = Chute {
                objet,
                x: tx as f32 * tt,
                depart: plafond(tx).unwrap_or(haut_chute).min(s - TAILLE_OBJET),
                sol: s,
                debut: t,
                annonce,
            };
            if self.laisse_une_issue(grille, &objets, &c) {
                objets.push(c);
            }
        }
        objets
    }

    /// Avec `c` en plus des objets en cours (et de `nouvelles`), reste-t-il, sur
    /// la corniche où `c` se brise, `PLACE_LIBRE_MIN` cases de suite sans
    /// objet ? (Une chute dans la mer en laisse toujours.)
    pub fn laisse_une_issue(&self, grille: &dyn Grille, nouvelles: &[Chute], c: &Chute) -> bool {
        let tt = TAILLE_TUILE;
        let ty = (c.sol / tt).round() as i32;
        if (c.sol - ty as f32 * tt).abs() > 0.01 || !grille.est_solide(c.colonne(), ty) {
            return true;
        }
        let corniche = |x: i32| grille.est_solide(x, ty) && !grille.est_solide(x, ty - 1);
        let (mut g, mut d) = (c.colonne(), c.colonne());
        while corniche(g - 1) {
            g -= 1;
        }
        while corniche(d + 1) {
            d += 1;
        }
        let prises: Vec<i32> = self
            .tour
            .chutes
            .iter()
            .chain(nouvelles)
            .chain(std::iter::once(c))
            .filter(|o| (o.sol - c.sol).abs() < 0.01)
            .map(Chute::colonne)
            .collect();
        let mut suite = 0;
        (g..=d).any(|x| {
            suite = if prises.contains(&x) { 0 } else { suite + 1 };
            suite >= PLACE_LIBRE_MIN
        })
    }

    /// Remet la montée à neuf (début de la phase 2, reprise) ; la graine et la
    /// tentative restent. Plus aucune tête à prendre : ses points restent à 0.
    pub(super) fn remettre_la_tour(&mut self) {
        let tour = &mut self.tour;
        tour.horloge_mer = 0;
        tour.plus_haut = None;
        tour.rattrape = false;
        tour.chutes.clear();
        tour.prochain = PREMIER_LANCER_PAS;
        tour.dernier_lancer = None;
        tour.lances = 0;
        tour.semer();
        self.pv = 0;
    }

    /// Pépin est mort en phase 2 et réapparaît dans `occupant` (sa lanterne,
    /// n'importe laquelle) : le tirage change (une tentative de plus) ; depuis
    /// la lanterne du bas, tout repart à neuf ; plus haut, la mer revient
    /// `MER_A_LA_REPRISE` sous ses pieds. S'il n'était jamais monté jusque-là
    /// pendant cette tentative (une lanterne allumée d'avant), tout repart du
    /// bas aussi.
    pub fn reprendre_a(&mut self, occupant: &Aabb) {
        if !self.en_fuite() || !matches!(self.phase(), PhasePieuvre::Furie | PhasePieuvre::Montee) {
            self.reprendre();
            return;
        }
        let pieds = occupant.bas();
        let jamais_la = self.tour.plus_haut.is_none_or(|h| h > pieds + 2.0);
        self.tour.tentative = self.tour.tentative.wrapping_add(1);
        self.reprendre();
        let t = TAILLE_TUILE;
        let au_bas = self.bache.is_none_or(|b| pieds >= b.y - 3.0 * t);
        if au_bas || jamais_la {
            return;
        }
        let cible = pieds + MER_A_LA_REPRISE;
        self.tour.horloge_mer = (0..u16::MAX)
            .find(|&h| self.surface_mer_a(h) <= cible)
            .unwrap_or(0);
        self.tour.plus_haut = Some(pieds);
    }

    /// L'écart le plus grand permis entre la mer et le plus haut de Pépin.
    pub fn ecart_mer_max(&self) -> f32 {
        if self.est_facile() {
            ECART_MER_MAX_FACILE
        } else {
            ECART_MER_MAX
        }
    }

    /// Les pieds de Pépin au plus haut pendant cette tentative.
    pub fn plus_haut_atteint(&self) -> Option<f32> {
        self.tour.plus_haut
    }

    /// La mer rattrape-t-elle Pépin (vitesse triple) en ce moment ?
    pub fn mer_rattrape(&self) -> bool {
        self.tour.rattrape && self.en_montee()
    }

    /// La mer est-elle trop loin sous le plus haut de Pépin ? Vrai dès qu'elle
    /// est à 2 px de l'écart permis (`ecart_mer_max`), pour qu'elle ne le
    /// dépasse jamais, et seulement si elle reste à plus de
    /// `RATTRAPAGE_PAS_SOUS` de ses pieds après un pas rapide (s'il est retombé
    /// près d'elle, elle reprend sa vitesse normale).
    fn mer_trop_loin(&self) -> bool {
        let (Some(mer), Some(haut), Some(pepin)) =
            (self.surface_mer(), self.tour.plus_haut, self.pepin)
        else {
            return false;
        };
        // Un pas rapide, plus un pas d'horloge au plus (l'horloge avance d'un
        // pas entier : elle peut dépasser la cible d'un pas normal).
        let pas_vite = (RATTRAPAGE_MER + 1.0) * self.vitesse_mer();
        mer - haut > self.ecart_mer_max() - 2.0
            && mer - pepin.bas() > RATTRAPAGE_PAS_SOUS + pas_vite
    }

    /// Un pas de la mer : l'horloge avance d'un pas ; si la mer est trop loin
    /// sous Pépin (`mer_trop_loin`), elle monte TROIS fois plus vite : on avance
    /// l'horloge jusqu'au pas où elle a monté de trois fois sa vitesse. Jamais
    /// de saut : au plus 3 vitesses par pas (un pas de crue de plus au plus) ;
    /// jamais de descente (l'horloge ne fait qu'avancer).
    fn avancer_la_mer(&mut self) {
        let suivant = self.tour.horloge_mer.saturating_add(1);
        let Some(mer) = self.surface_mer().filter(|_| self.mer_trop_loin()) else {
            self.tour.horloge_mer = suivant;
            return;
        };
        self.tour.rattrape = true;
        let cible = mer - RATTRAPAGE_MER * self.vitesse_mer();
        let limite = suivant.saturating_add(8);
        let mut n = suivant;
        while n + 1 < limite && self.surface_mer_a(n) > cible + 0.01 {
            n += 1;
        }
        self.tour.horloge_mer = n;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chute() -> Chute {
        Chute {
            objet: Objet::Caisse,
            x: 32.0,
            depart: 0.0,
            sol: 160.0,
            debut: 10,
            annonce: ANNONCE_CHUTE_PAS,
        }
    }

    /// Une chute : l'annonce (au moins 0,8 s), puis la chute (mortelle), puis
    /// les éclats (sans danger), puis plus rien. Elle ne passe jamais sous son
    /// sol.
    #[test]
    fn une_chute_s_annonce_tombe_puis_se_brise() {
        const { assert!(ANNONCE_CHUTE_PAS >= ANNONCE_MIN_PAS) };
        let c = chute();
        assert_eq!(c.etat(9), None);
        assert_eq!(c.etat(10), Some(EtatChute::Annonce(0.0)));
        for t in 10..c.lache() {
            assert!(c.boite(t).is_none(), "pas de danger pendant l'annonce");
        }
        assert!(c.lache() - c.debut >= ANNONCE_MIN_PAS);
        for t in c.lache()..c.arrivee() {
            let b = c.boite(t).expect("elle tombe");
            assert!(b.bas() <= c.sol, "{t} : {b:?}");
        }
        assert!(matches!(c.etat(c.arrivee()), Some(EtatChute::Bris(_))));
        assert!(c.boite(c.arrivee()).is_none());
        assert_eq!(c.etat(c.fin()), None);
    }
}
