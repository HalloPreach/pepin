//! La montée de la phase 2 : la Pieuvre GÉANTE grimpe à côté de la tour, et
//! chaque attaque part VISIBLEMENT de son corps (carte 9.13c).
//!
//! Demande d'AGA : « je n'aime pas la montée, les attaques n'ont pas de sens ». Il
//! a choisi : « Garder la montée, mais avec la Pieuvre géante et visible qui
//! grimpe à côté de la tour : chaque attaque part de son corps (son œil te suit,
//! elle lève un tentacule et frappe là où tu es, elle agrippe la tour et la
//! secoue). Plus de trappe, de gardes ni de tentacules qui sortent de nulle
//! part. »
//!
//! ```text
//!   mur │ la tour (13 colonnes)          │ couloir (5) │ mur
//!       │ corniches, caisses, vigie       │  LA GÉANTE  │
//!       │      ◄── ses tentacules ────────│  tête, œil  │
//!   ~~~~~~~~~~~~~~~~~~~~~~~~~ la mer qui monte ~~~~~~~~~~~~~~~~~~~~
//! ```
//!
//! - LA GÉANTE vit dans le COULOIR, contre le mur droit (`Tour::couloir`, posé
//!   par le parseur) : son corps est dans la mer, sa tête et son grand œil juste
//!   au-dessus, et ils montent AVEC la mer (`geante`). Si Pépin grimpe trop vite,
//!   elle GRIMPE hors de l'eau, agrippée à la tour, pour rester à 4 tuiles sous
//!   ses pieds : elle est toujours à l'écran. Toucher son corps tue, comme un
//!   tentacule (`corps` : tout le couloir, de sa tête jusque dans la mer). Son œil suit Pépin et ROUGIT pendant
//!   l'annonce d'une attaque (`annonce_en_cours`) : c'est le repère du joueur.
//! - Ses ATTAQUES (`Attaque`) partent toutes de son corps. Carte 9.13d : plus
//!   aucune n'est attachée à une rangée ; c'est son CERVEAU (`cerveau`) qui
//!   choisit, selon ce qu'elle voit de Pépin, avec un tirage pseudo-aléatoire
//!   dont la graine change à chaque tentative :
//!   - FRAPPE VISÉE : elle lève un tentacule et VISE l'appui où Pépin VA
//!     ATTERRIR (un repère rouge, 1,4 s), puis frappe cette place (0,5 s). Arrêté,
//!     il est visé sur place. Jamais sur la seule place sûre d'un appui
//!     (`viser`).
//!   - BALAYAGE : sur une corniche qui touche son couloir, le tentacule recule et
//!     se tend (1 s), puis balaie de droite à gauche, au ras du sol : on saute
//!     par-dessus son bout.
//!   - SECOUSSE : elle agrippe la tour et la secoue ; 2 ou 3 caisses tombent du
//!     haut, dans la colonne de Pépin et près de lui (jamais deux voisines),
//!     annoncées 1 s par leur ombre sur le sol où elles vont se briser.
//!   - ARRACHAGE (`%`) : son tentacule va jusqu'à la plateforme et s'y enroule
//!     (1,3 s, elle porte encore), puis la tire dans la mer. Il reste toujours un
//!     autre chemin (les tests le vérifient).
//!   - JETÉ, sur une corniche large : elle se CABRE (1,2 s, l'œil fixé sur lui),
//!     puis se jette tête la première sur le bout de la corniche. Si elle le
//!     rate, elle reste SONNÉE 2 s, la tête posée. Un saut sur sa tête : elle
//!     hurle et retombe, la mer RECULE de 3 tuiles et ses attaques s'arrêtent
//!     3 s (une tête en moins). Sinon elle repart, la mer reste haute.
//!
//!   Son HUMEUR monte (calme, énervée, furieuse) avec les coups reçus et la
//!   hauteur de Pépin : recharges plus courtes, et des COMBOS (un balayage ou
//!   une secousse, puis une frappe visée enchaînée).
//! - COUP FINAL : sur la vigie (`&`), elle se jette DEUX fois de suite, la
//!   seconde plus vite (0,8 s), et n'est sonnée qu'après la seconde. Il faut lui
//!   prendre TOUTES les têtes qui restent (3 au début de la phase 2, une de moins
//!   par coup pendant la montée) : autant de doubles jetés que de têtes.
//!
//! La mer s'arrête pendant un jeté (elle a mieux à faire). Une mort ramène à la
//! dernière lanterne (n'importe laquelle) : la mer est remise sous la lanterne,
//! l'humeur et les têtes reviennent à leur état au passage de la lanterne, les
//! plateformes arrachées dessous restent parties, et la graine change. MODE
//! FACILE : annonces et recharges 0,75 fois moins rapides (`lent`).
//!
//! Notions Rust : un `enum` dont chaque variante porte ses données
//! (`Attaque::Secousse { debut, caisses }`) et un `match` qui les déballe ;
//! `Option::take` pour vider une case en récupérant son contenu ; des fonctions
//! PURES du temps : l'état d'une attaque se calcule depuis son pas de départ.

use super::{PhasePieuvre, Pieuvre, ANNONCE_PAS, ANNONCE_SAISIE_PAS, DRESSE_PAS};
use super::{FACTEUR_FACILE, PLONGEE_SAISIE_PAS};
use crate::aabb::{Aabb, Grille, Tuile, TAILLE_TUILE};
use crate::physics::{JOUEUR_H, JOUEUR_L};

mod cerveau;
pub use cerveau::*;

/// La tête de la géante (px) : sa boîte dans le couloir, et quand elle se jette.
pub const TAILLE_GEANTE: (f32, f32) = (64.0, 40.0);
/// Largeur (en tuiles) du couloir de la géante, contre le mur droit de la tour.
pub const COULOIR_TUILES: usize = 5;
/// Sa tête reste au plus 4 tuiles sous les pieds de Pépin (toujours à l'écran)…
pub const SOUS_PEPIN: f32 = 4.0 * TAILLE_TUILE;
/// … et grimpe (ou redescend vers la mer) de 2 px par pas au plus.
pub const GRIMPE_GEANTE: f32 = 2.0;

/// La frappe visée s'annonce 1,4 s (le tentacule levé, le repère rouge)…
pub const ANNONCE_FRAPPE_PAS: u16 = ANNONCE_PAS;
/// … frappe 0,5 s (mortelle) …
pub const FRAPPE_PAS: u16 = DRESSE_PAS;
/// … puis se retire en 0,2 s, sans danger.
pub const RETRAIT_FRAPPE_PAS: u16 = 12;
/// Une frappe visée toutes les 2,5 s quand elle est calme (voir `RECHARGES`).
pub const RYTHME_FRAPPE_PAS: u16 = 150;
/// La place frappée : 24 x 40 px, posée sur les pieds de Pépin.
pub const TAILLE_FRAPPE: (f32, f32) = (24.0, 40.0);
/// Si la place de Pépin est la seule sûre de son appui, elle vise à côté, au
/// plus à 3 tuiles.
pub const VISEE_MAX: f32 = 3.0 * TAILLE_TUILE;

/// Le balayage s'annonce 1 s (le tentacule recule et se tend).
pub const ANNONCE_BALAI_PAS: u16 = 60;
/// Il glisse de 3 px par pas (180 px/s : Pépin court à 96 px/s, il faut sauter).
pub const VITESSE_BALAI: f32 = 3.0;
/// Il se retire en 0,5 s, sans danger.
pub const RETRAIT_BALAI_PAS: u16 = 30;
/// Repos d'un balayage avant qu'il puisse repartir.
pub const REPOS_BALAI_PAS: u16 = 180;
/// Le bout du balayage : 24 x 12, au ras de la corniche.
pub const BOUT_BALAI: (f32, f32) = (24.0, 12.0);

/// La secousse s'annonce 1 s (elle agrippe la tour, l'ombre des caisses).
pub const ANNONCE_SECOUSSE_PAS: u16 = 60;
/// Une caisse tombe de 5 px par pas…
pub const CHUTE_CAISSE: f32 = 5.0;
/// … depuis 9 tuiles au-dessus des pieds de Pépin (hors de l'écran).
pub const HAUT_CHUTE: f32 = 9.0 * TAILLE_TUILE;
/// Les caisses : dans la colonne de Pépin, puis dans les colonnes au moins à 2
/// colonnes de lui et de la caisse voisine, jusqu'à 5 colonnes de lui…
pub const ECART_CAISSES: i32 = 5;
/// … là où elles se brisent au plus 3 tuiles sous ses pieds (leur ombre se voit).
pub const SOL_CAISSES: f32 = 3.0 * TAILLE_TUILE;
/// Une caisse fait une tuile.
pub const TAILLE_CAISSE: f32 = TAILLE_TUILE;

/// Un arrachage commence quand Pépin est 1 à 4 tuiles sous la plateforme…
pub const ARRACHAGE_SOUS: (f32, f32) = (TAILLE_TUILE, 4.0 * TAILLE_TUILE);
/// … et à moins de 5 tuiles d'elle de côté.
pub const ARRACHAGE_COTE: f32 = 5.0 * TAILLE_TUILE;
/// L'arrachée part en 0,5 s (elle ne porte plus dès ce moment).
pub const EMPORTEE_PAS: u16 = PLONGEE_SAISIE_PAS;

/// Le jeté : elle se cabre 1,2 s (de 24 px), l'œil fixé sur Pépin…
pub const CABRE_PAS: u16 = 72;
/// … 0,8 s seulement pour le second jeté de la vigie (carte 9.13d) …
pub const CABRE_VIF_PAS: u16 = 48;
pub const CABRE_HAUT: f32 = 24.0;
/// … se jette en 1/6 s (mortelle) …
pub const CHUTE_JETE_PAS: u16 = 10;
/// … reste sonnée 2 s, la tête sur la corniche (on peut lui sauter dessus) …
pub const SONNEE_PAS: u16 = 120;
/// … puis, ratée, elle repart en 0,5 s.
pub const RETOUR_PAS: u16 = 30;
/// Après un jeté raté, 1 s sans attaque.
pub const APRES_JETE_PAS: u16 = 60;
/// Frappée, elle hurle et retombe dans la mer en 0,7 s…
pub const LACHEE_PAS: u16 = 42;
/// … la mer recule de 3 tuiles…
pub const RECUL_MER: f32 = 3.0 * TAILLE_TUILE;
/// … en remontant son horloge de 4 pas par pas …
pub const RECUL_PAR_PAS: u16 = 4;
/// … et ses attaques s'arrêtent 3 s.
pub const REPIT_PAS: u16 = 180;
/// Ses têtes au début de la phase 2 (au plus les cases du HUD) : une par coup
/// pendant la montée, les autres à la vigie.
pub const TETES_FUITE: u8 = 3;

/// Un balayage : la corniche qu'il balaie (son dessus `y`), le bord du couloir
/// d'où part le bout (`depart`) et le bout gauche de la corniche (`arrivee`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Balayeur {
    pub y: f32,
    pub depart: f32,
    pub arrivee: f32,
}

impl Balayeur {
    /// Pas de glissade, du couloir au bout de la corniche.
    pub fn course_pas(&self) -> u16 {
        ((self.depart - self.arrivee - BOUT_BALAI.0).max(0.0) / VITESSE_BALAI).ceil() as u16
    }

    /// La boîte du bout après `s` pas de glissade (de droite à gauche).
    pub fn bout(&self, s: u16) -> Aabb {
        let x = (self.depart - BOUT_BALAI.0 - f32::from(s) * VITESSE_BALAI).max(self.arrivee);
        Aabb::new(x, self.y - BOUT_BALAI.1, BOUT_BALAI.0, BOUT_BALAI.1)
    }
}

/// Une caisse qui tombe pendant une secousse : sa colonne (`x`), d'où elle part
/// et le sol où elle se brise (le dessus d'une case solide, ou la mer).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Caisse {
    pub x: f32,
    pub depart: f32,
    pub sol: f32,
}

impl Caisse {
    /// Pas de chute jusqu'au sol.
    pub fn chute_pas(&self) -> u16 {
        ((self.sol - TAILLE_CAISSE - self.depart).max(0.0) / CHUTE_CAISSE).ceil() as u16
    }

    /// La caisse après `s` pas de chute ; `None` une fois brisée sur son sol.
    pub fn boite(&self, s: u16) -> Option<Aabb> {
        let y = self.depart + f32::from(s) * CHUTE_CAISSE;
        (y + TAILLE_CAISSE < self.sol).then(|| Aabb::new(self.x, y, TAILLE_CAISSE, TAILLE_CAISSE))
    }
}

/// L'attaque en cours (une seule à la fois, plus la frappe d'un combo), avec le
/// pas où elle a commencé.
#[derive(Debug, Clone, PartialEq)]
pub enum Attaque {
    /// La place visée.
    Frappe { debut: u16, cible: Aabb },
    /// Le balayeur `k`.
    Balayage { k: usize, debut: u16 },
    /// Les caisses qui vont tomber.
    Secousse { debut: u16, caisses: Vec<Caisse> },
    /// La plateforme arrachable `k` (carte 9.13d : une attaque choisie).
    Arrachage { k: usize, debut: u16 },
    /// Le jeté sur la place `k` (`k == jetes.len()` : la vigie, le coup final).
    /// `vif` : la cabre courte ; `sonne` : ratée, elle reste sonnée (pas après le
    /// premier jeté de la vigie, qui enchaîne sur le second).
    Jete {
        k: usize,
        debut: u16,
        vif: bool,
        sonne: bool,
    },
}

impl Attaque {
    /// Le pas où elle a commencé.
    pub fn debut(&self) -> u16 {
        match self {
            Attaque::Frappe { debut, .. }
            | Attaque::Balayage { debut, .. }
            | Attaque::Secousse { debut, .. }
            | Attaque::Arrachage { debut, .. }
            | Attaque::Jete { debut, .. } => *debut,
        }
    }
}

/// Où en est une frappe visée.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EtatFrappe {
    /// Le tentacule se lève et vise (de 0 à 1) : sans danger.
    Annonce(f32),
    /// Il frappe la place visée : mortel.
    Frappe,
    /// Il se retire (de 0 à 1) : sans danger.
    Retrait(f32),
}

/// Où en est un balayage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EtatBalai {
    /// Le tentacule recule et se tend (de 0 à 1) : sans danger.
    Annonce(f32),
    /// Il glisse : son bout (la boîte) tue.
    Balaie(Aabb),
    /// Il se retire (de 0 à 1) : sans danger.
    Retrait(f32),
}

/// Où en est une secousse.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EtatSecousse {
    /// Elle agrippe la tour, l'ombre des caisses (de 0 à 1) : sans danger.
    Annonce(f32),
    /// Les caisses tombent depuis ce nombre de pas.
    Chute(u16),
}

/// Où en est un jeté (la tête de la géante).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EtatJete {
    /// Elle se cabre, l'œil fixé sur Pépin (de 0 à 1) : sans danger.
    Cabre(f32),
    /// Elle se jette sur la corniche (de 0 à 1) : mortelle.
    Chute(f32),
    /// Sonnée, la tête posée (de 0 à 1) : on lui saute dessus.
    Sonnee(f32),
    /// Ratée, elle repart (de 0 à 1) : sans danger.
    Retour(f32),
}

/// Ce que devient une plateforme arrachée.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EtatArrachage {
    /// Le tentacule s'enroule, elle tremble (de 0 à 1) : elle porte encore.
    Annonce(f32),
    /// Il l'emporte (de 0 à 1) : elle ne porte plus.
    Emportee(f32),
    /// Partie pour de bon.
    Partie,
}

/// Tout ce que la montée ajoute à la Pieuvre. Posé par le parseur (`jetes`,
/// `balayeurs`, `arrachables`, `couloir` ; carte 9.13d : complété au début de
/// la montée par les corniches larges et celles qui touchent le couloir), le
/// reste est l'état du combat.
#[derive(Debug, Clone, Default)]
pub struct Tour {
    /// Les places du jeté : la tête posée sur le bout droit d'une corniche large
    /// (64 x 40), du bas vers le haut.
    pub jetes: Vec<Aabb>,
    pub balayeurs: Vec<Balayeur>,
    /// Les anciennes rangées de secousse (`+`, carte 9.13c) : plus lues par le
    /// cerveau (carte 9.13d), gardées pour le parseur.
    pub secousses: Vec<f32>,
    /// Les plateformes arrachables : leurs cases `(tx, ty)`.
    pub arrachables: Vec<Vec<(i32, i32)>>,
    /// Le couloir de la géante (gauche, droite en px) ; `None` : pas de géante.
    pub couloir: Option<(f32, f32)>,
    /// Le bas de sa tête au repos (px), quand elle grimpe hors de l'eau ; `None` :
    /// posée sur la mer.
    bas_geante: Option<f32>,
    /// Pour le robot de faisabilité : aucune attaque, la tête de la vigie offerte.
    robot: bool,
    attaque: Option<Attaque>,
    /// Jetés lancés pendant cette tentative.
    faits: usize,
    /// Pas avant lequel aucune attaque ne part (le répit, son souffle).
    repit: u16,
    /// La tête frappée qui retombe et le pas du coup.
    lachee: Option<(Aabb, u16)>,
    /// L'horloge de la mer (pas de `surface_mer_a`), arrêtée pendant un jeté.
    horloge_mer: u16,
    /// Pas d'horloge de la mer qu'il reste à reculer.
    recul: u16,
    /// Début de chaque arrachage (ou rien).
    arrachages: Vec<Option<u16>>,
    /// Ce qu'elle voit, ce qu'elle ressent et ce qu'elle décide (carte 9.13d).
    cerveau: Cerveau,
}

/// La boîte à mi-chemin (`k` de 0 à 1) entre `a` et `b`, au pixel près.
fn entre(a: Aabb, b: Aabb, k: f32) -> Aabb {
    let k = k.clamp(0.0, 1.0);
    Aabb::new(
        (a.x + (b.x - a.x) * k).round(),
        (a.y + (b.y - a.y) * k).round(),
        a.w,
        a.h,
    )
}

/// Quatre réglages pour les tests du VRAI niveau du boss, qui vivent dans le
/// crate `niveaux` (`tests/tous_les_niveaux/pieuvre_montee.rs`, carte 10.1c) :
/// ils y préparent un état précis (la géante à 1 point de vie, les recharges
/// vides...) sans voir les champs privés. Compilés seulement avec la
/// fonctionnalité `pour_les_tests` de `core` (que `niveaux` n'active que pour ses
/// tests) : le jeu ne les voit jamais.
#[cfg(feature = "pour_les_tests")]
impl Pieuvre {
    /// Fixe ses points de vie.
    pub fn fixer_pv_pour_les_tests(&mut self, pv: u8) {
        self.pv = pv;
    }

    /// Les recharges de son cerveau (pas où chaque genre d'attaque est prêt).
    pub fn recharges_pour_les_tests(&mut self) -> &mut [u16; 5] {
        &mut self.tour.cerveau.recharges
    }

    /// La frappe visée d'un combo en cours (son début, la place visée).
    pub fn combo_pour_les_tests(&self) -> Option<(u16, Aabb)> {
        self.tour.cerveau.combo
    }

    /// La recharge d'un genre d'attaque selon son humeur (pas).
    pub fn recharge_pour_les_tests(&self, g: Genre) -> u16 {
        self.recharge(g)
    }
}

impl Pieuvre {
    /// L'horloge de la mer : `surface_mer_a(horloge_mer)` est la mer du moment.
    pub fn horloge_mer(&self) -> u16 {
        self.tour.horloge_mer
    }

    /// Jetés lancés pendant cette tentative (frappés ou non).
    pub fn jetes_faits(&self) -> usize {
        self.tour.faits
    }

    /// Coups reçus pendant la phase 2 (une tête chacun).
    pub fn jetes_frappes(&self) -> usize {
        usize::from(self.pv_de_la_fuite().saturating_sub(self.pv))
    }

    fn en_montee(&self) -> bool {
        self.en_fuite() && self.phase() == PhasePieuvre::Montee
    }

    /// Mode facile : les annonces et le rythme des attaques 0,75 fois moins
    /// rapides.
    fn lent(&self, pas: u16) -> u16 {
        if self.est_facile() {
            (f32::from(pas) / FACTEUR_FACILE) as u16
        } else {
            pas
        }
    }

    /// Pas écoulés depuis `debut` (`None` avant).
    fn depuis(&self, debut: u16) -> Option<u16> {
        self.ecoule().checked_sub(debut)
    }

    /// L'attaque en cours, pendant la montée.
    pub fn attaque(&self) -> Option<&Attaque> {
        self.tour.attaque.as_ref().filter(|_| self.en_montee())
    }

    /// La boîte solide de la pieuvre : la bâche, tant que dure la phase 1.
    pub fn solide(&self) -> Option<Aabb> {
        self.bache_solide()
    }

    /// La place de la tête au jeté `k` (`k == jetes.len()` : la vigie).
    fn place_du_jete(&self, k: usize) -> Option<Aabb> {
        match self.tour.jetes.get(k) {
            Some(&t) => Some(t),
            None if k == self.tour.jetes.len() => self.tete_finale,
            None => None,
        }
    }

    /// Durée totale d'une attaque (pas).
    fn duree_attaque(&self, a: &Attaque) -> u16 {
        match a {
            Attaque::Frappe { .. } => self.duree_frappe(),
            Attaque::Balayage { k, .. } => {
                let course = self.tour.balayeurs.get(*k).map_or(0, |b| b.course_pas());
                self.lent(ANNONCE_BALAI_PAS) + course + RETRAIT_BALAI_PAS
            }
            Attaque::Secousse { caisses, .. } => {
                let chute = caisses.iter().map(|c| c.chute_pas()).max().unwrap_or(0);
                // Et 1/6 s pour que les débris retombent.
                self.lent(ANNONCE_SECOUSSE_PAS) + chute + 10
            }
            Attaque::Arrachage { .. } => ANNONCE_SAISIE_PAS + EMPORTEE_PAS,
            &Attaque::Jete { vif, sonne, .. } => {
                let (cabre, sonnee) = self.durees_du_jete(vif, sonne);
                cabre + CHUTE_JETE_PAS + sonnee + RETOUR_PAS
            }
        }
    }

    /// Durée d'une frappe visée (pas) : annonce, frappe, retrait.
    fn duree_frappe(&self) -> u16 {
        self.lent(ANNONCE_FRAPPE_PAS) + FRAPPE_PAS + RETRAIT_FRAPPE_PAS
    }

    /// La cabre et la sonnée d'un jeté (pas) : la cabre courte s'il est `vif`,
    /// aucune sonnée s'il ne `sonne` pas.
    fn durees_du_jete(&self, vif: bool, sonne: bool) -> (u16, u16) {
        let cabre = self.lent(if vif { CABRE_VIF_PAS } else { CABRE_PAS });
        (cabre, if sonne { self.lent(SONNEE_PAS) } else { 0 })
    }

    /// La frappe visée en cours (seule, ou enchaînée dans un combo) : son début
    /// et la place visée.
    fn frappe_en_cours(&self) -> Option<(u16, Aabb)> {
        if !self.en_montee() {
            return None;
        }
        match self.tour.attaque {
            Some(Attaque::Frappe { debut, cible }) => Some((debut, cible)),
            _ => self.tour.cerveau.combo,
        }
    }

    /// La frappe visée en cours : la place visée et son état.
    pub fn etat_frappe(&self) -> Option<(Aabb, EtatFrappe)> {
        let (debut, cible) = self.frappe_en_cours()?;
        let d = self.depuis(debut)?;
        let annonce = self.lent(ANNONCE_FRAPPE_PAS);
        let etat = if d < annonce {
            EtatFrappe::Annonce(f32::from(d) / f32::from(annonce))
        } else if d < annonce + FRAPPE_PAS {
            EtatFrappe::Frappe
        } else if d < annonce + FRAPPE_PAS + RETRAIT_FRAPPE_PAS {
            let r = d - annonce - FRAPPE_PAS;
            EtatFrappe::Retrait(f32::from(r) / f32::from(RETRAIT_FRAPPE_PAS))
        } else {
            return None;
        };
        Some((cible, etat))
    }

    /// L'état du balayeur `k` (`None` : au repos).
    pub fn etat_balayeur(&self, k: usize) -> Option<EtatBalai> {
        let Some(&Attaque::Balayage { k: kk, debut }) = self.attaque() else {
            return None;
        };
        if kk != k {
            return None;
        }
        let b = self.tour.balayeurs.get(k)?;
        let d = self.depuis(debut)?;
        let annonce = self.lent(ANNONCE_BALAI_PAS);
        let course = b.course_pas();
        Some(if d < annonce {
            EtatBalai::Annonce(f32::from(d) / f32::from(annonce))
        } else if d < annonce + course {
            EtatBalai::Balaie(b.bout(d - annonce))
        } else if d < annonce + course + RETRAIT_BALAI_PAS {
            let r = d - annonce - course;
            EtatBalai::Retrait(f32::from(r) / f32::from(RETRAIT_BALAI_PAS))
        } else {
            return None;
        })
    }

    /// La secousse en cours : ses caisses et son état.
    pub fn etat_secousse(&self) -> Option<(&[Caisse], EtatSecousse)> {
        let Some(Attaque::Secousse { debut, caisses }) = self.attaque() else {
            return None;
        };
        let d = self.depuis(*debut)?;
        let annonce = self.lent(ANNONCE_SECOUSSE_PAS);
        let etat = if d < annonce {
            EtatSecousse::Annonce(f32::from(d) / f32::from(annonce))
        } else {
            EtatSecousse::Chute(d - annonce)
        };
        Some((caisses.as_slice(), etat))
    }

    /// Le jeté en cours : la place de la tête sur la corniche et son état.
    pub fn etat_jete(&self) -> Option<(Aabb, EtatJete)> {
        let Some(&Attaque::Jete {
            k,
            debut,
            vif,
            sonne,
        }) = self.attaque()
        else {
            return None;
        };
        let tete = self.place_du_jete(k)?;
        let d = self.depuis(debut)?;
        let (cabre, sonnee) = self.durees_du_jete(vif, sonne);
        let part = |x: u16, duree: u16| f32::from(x) / f32::from(duree);
        let etat = if d < cabre {
            EtatJete::Cabre(part(d, cabre))
        } else if d < cabre + CHUTE_JETE_PAS {
            EtatJete::Chute(part(d - cabre, CHUTE_JETE_PAS))
        } else if d < cabre + CHUTE_JETE_PAS + sonnee {
            EtatJete::Sonnee(part(d - cabre - CHUTE_JETE_PAS, sonnee))
        } else if d < cabre + CHUTE_JETE_PAS + sonnee + RETOUR_PAS {
            EtatJete::Retour(part(d - cabre - CHUTE_JETE_PAS - sonnee, RETOUR_PAS))
        } else {
            return None;
        };
        Some((tete, etat))
    }

    /// La tête frappée qui retombe dans la mer : d'où elle part et l'avancement
    /// (de 0 à 1).
    pub fn tete_lachee(&self) -> Option<(Aabb, f32)> {
        let (tete, coup) = self.tour.lachee?;
        let d = self.depuis(coup)?;
        (d < LACHEE_PAS && self.en_montee()).then(|| (tete, f32::from(d) / f32::from(LACHEE_PAS)))
    }

    /// Une annonce est-elle en cours (une attaque se prépare, ou une plateforme
    /// va être arrachée) ? L'œil de la géante rougit.
    pub fn annonce_en_cours(&self) -> bool {
        let arrachage = (0..self.tour.arrachables.len())
            .any(|k| matches!(self.etat_arrachage(k), Some(EtatArrachage::Annonce(_))));
        arrachage
            || matches!(self.etat_frappe(), Some((_, EtatFrappe::Annonce(_))))
            || (0..self.tour.balayeurs.len())
                .any(|k| matches!(self.etat_balayeur(k), Some(EtatBalai::Annonce(_))))
            || matches!(self.etat_secousse(), Some((_, EtatSecousse::Annonce(_))))
            || matches!(self.etat_jete(), Some((_, EtatJete::Cabre(_))))
    }

    /// La tête de la géante dans le couloir, au repos : posée sur la mer (ou
    /// plus haut quand elle grimpe), au milieu du couloir. `None` sans géante ou
    /// hors de la montée.
    fn tete_au_repos(&self) -> Option<Aabb> {
        let (g, d) = self.tour.couloir?;
        if !self.en_montee() {
            return None;
        }
        let mer = self.surface_mer()?;
        let bas = self.tour.bas_geante.unwrap_or(mer).min(mer);
        let (w, h) = TAILLE_GEANTE;
        Some(Aabb::new(
            ((g + d - w) / 2.0).round(),
            bas.round() - h,
            w,
            h,
        ))
    }

    /// Grimpe-t-elle hors de l'eau (le bas de sa tête au-dessus de la mer) ?
    pub fn grimpe(&self) -> bool {
        match (self.tour.bas_geante, self.surface_mer()) {
            (Some(b), Some(m)) => b < m - 1.0,
            _ => false,
        }
    }

    /// Où est la tête de la géante en ce moment (le dessin) : au repos sur la
    /// mer, cabrée, jetée sur une corniche, ou qui retombe.
    pub fn geante(&self) -> Option<Aabb> {
        let repos = self.tete_au_repos()?;
        if let Some((tete, a)) = self.tete_lachee() {
            return Some(entre(tete, repos, a * a));
        }
        let cabre = Aabb::new(repos.x, repos.y - CABRE_HAUT, repos.w, repos.h);
        Some(match self.etat_jete() {
            Some((_, EtatJete::Cabre(a))) => entre(repos, cabre, a),
            Some((tete, EtatJete::Chute(a))) => entre(cabre, tete, a),
            Some((tete, EtatJete::Sonnee(_))) => tete,
            Some((tete, EtatJete::Retour(a))) => entre(tete, repos, a),
            None => repos,
        })
    }

    /// Le corps de la géante, qui tue au contact : tout le couloir, du haut de sa
    /// tête jusque sous la mer. Seulement quand la mer est dans la tour (pendant
    /// la crue, elle sort à peine de l'eau du bassin) ; jamais pour le robot.
    pub fn corps(&self) -> Option<Aabb> {
        if self.tour.robot || self.horloge_mer() < self.fin_de_crue() {
            return None;
        }
        let (g, d) = self.tour.couloir?;
        let repos = self.tete_au_repos()?;
        let mer = self.surface_mer()?;
        let h = mer - repos.y + TAILLE_TUILE;
        Some(Aabb::new(g + 4.0, repos.y, d - g - 4.0, h))
    }

    /// La mer est-elle arrêtée (pendant un jeté, jusqu'à ce qu'elle reparte) ?
    fn mer_arretee(&self) -> bool {
        matches!(
            self.etat_jete(),
            Some((
                _,
                EtatJete::Cabre(_) | EtatJete::Chute(_) | EtatJete::Sonnee(_)
            ))
        )
    }

    /// La place que vise une frappe lancée sur Pépin (`pepin`) : la sienne, ou,
    /// si c'est la seule place sûre de son appui, la plus proche qui laisse sur
    /// CHAQUE appui touché une place sûre d'au moins la largeur de Pépin (plus
    /// 2 px). Pure.
    pub fn viser(&self, grille: &dyn Grille, pepin: &Aabb) -> Aabb {
        let (w, h) = TAILLE_FRAPPE;
        let x0 = (pepin.x + pepin.w / 2.0 - w / 2.0).round();
        let boite = |x: f32| Aabb::new(x, pepin.bas() - h, w, h);
        let appuis = self.appuis(grille);
        let laisse_une_issue = |c: &Aabb| {
            appuis
                .iter()
                .filter(|&&(y, a, b)| {
                    y > c.y && y - JOUEUR_H < c.bas() && a < c.droite() && b > c.x
                })
                .all(|&(_, a, b)| c.x - a >= JOUEUR_L + 2.0 || b - c.droite() >= JOUEUR_L + 2.0)
        };
        (0..=(VISEE_MAX / 4.0) as i32)
            .flat_map(|k| [k, -k])
            .map(|k| boite(x0 + 4.0 * k as f32))
            .find(laisse_une_issue)
            .unwrap_or_else(|| boite(x0))
    }

    /// Les caisses d'une secousse lancée sur Pépin : au plus 3, du haut jusqu'au
    /// premier sol sous elles (ou la mer `mer`). La 1re dans sa colonne ; les
    /// autres dans les colonnes de la tour les plus proches de lui (à
    /// `ECART_CAISSES` au plus) dont le sol est au plus `SOL_CAISSES` sous ses
    /// pieds (leur ombre se voit), jamais à moins de 2 colonnes d'une autre
    /// caisse : à côté de chaque caisse, une place sûre. Pure.
    pub fn caisses_pour(&self, grille: &dyn Grille, pepin: &Aabb, mer: f32) -> Vec<Caisse> {
        let t = TAILLE_TUILE;
        let bord = self.tour.couloir.map_or(f32::MAX, |c| c.0);
        let colonne = ((pepin.x + pepin.w / 2.0) / t).floor() as i32;
        let rangee = (pepin.bas() / t).floor() as i32 - 1;
        let depart = pepin.bas() - HAUT_CHUTE;
        let sol = |tx: i32| {
            (rangee..rangee + 40)
                .find(|&ty| grille.tuile(tx, ty) == Tuile::Solide)
                .map_or(mer, |ty| ty as f32 * t)
                .min(mer)
        };
        // Le plus bas solide au-dessus des pieds de Pépin dans la colonne `tx`
        // (une caisse n'y passe pas) : le bas de sa case, ou rien.
        let haut = (depart / t).floor() as i32;
        let plafond = |tx: i32| {
            (haut..=rangee)
                .rev()
                .find(|&ty| grille.tuile(tx, ty) == Tuile::Solide)
                .map(|ty| (ty + 1) as f32 * t)
        };
        let mut caisses: Vec<Caisse> = Vec::new();
        let ecarts = std::iter::once(0).chain((2..=ECART_CAISSES).flat_map(|k| [-k, k]));
        for tx in ecarts.map(|dx| colonne + dx) {
            let dans_la_tour = tx >= 1 && (tx + 1) as f32 * t <= bord;
            let loin = caisses.iter().all(|c| ((c.x / t) as i32 - tx).abs() >= 2);
            let proche = sol(tx) <= pepin.bas() + SOL_CAISSES && plafond(tx).is_none();
            if caisses.len() < 3 && dans_la_tour && loin && (tx == colonne || proche) {
                // Sous un plafond, celle de Pépin part de dessous lui.
                caisses.push(Caisse {
                    x: tx as f32 * t,
                    depart: plafond(tx).unwrap_or(depart),
                    sol: sol(tx),
                });
            }
        }
        caisses
    }

    /// L'état de la plateforme arrachable `k` (`None` : intacte).
    pub fn etat_arrachage(&self, k: usize) -> Option<EtatArrachage> {
        let d = self.ecoule().checked_sub((*self.tour.arrachages.get(k)?)?);
        let Some(d) = d.filter(|_| self.phase() == PhasePieuvre::Montee) else {
            // Après le coup final (ou une horloge remise à zéro) : partie.
            return Some(EtatArrachage::Partie);
        };
        Some(if d < ANNONCE_SAISIE_PAS {
            EtatArrachage::Annonce(f32::from(d) / f32::from(ANNONCE_SAISIE_PAS))
        } else if d < ANNONCE_SAISIE_PAS + EMPORTEE_PAS {
            EtatArrachage::Emportee(f32::from(d - ANNONCE_SAISIE_PAS) / f32::from(EMPORTEE_PAS))
        } else {
            EtatArrachage::Partie
        })
    }

    /// Les cases arrachées qui ne portent plus.
    pub(super) fn cases_arrachees(&self) -> Vec<(i32, i32)> {
        (0..self.tour.arrachables.len())
            .filter(|&k| {
                matches!(
                    self.etat_arrachage(k),
                    Some(EtatArrachage::Emportee(_) | EtatArrachage::Partie)
                )
            })
            .flat_map(|k| self.tour.arrachables[k].iter().copied())
            .collect()
    }

    /// Phase 2 : le corps de la géante, ou une attaque mortelle en ce moment,
    /// touche-t-il `boite` ?
    pub(super) fn blesse_en_montee(&self, boite: &Aabb) -> bool {
        if !self.en_montee() {
            return false;
        }
        let corps = self.corps().is_some_and(|c| c.chevauche(boite));
        let frappe = matches!(
            self.etat_frappe(),
            Some((c, EtatFrappe::Frappe)) if c.chevauche(boite)
        );
        let balai = (0..self.tour.balayeurs.len()).any(
            |k| matches!(self.etat_balayeur(k), Some(EtatBalai::Balaie(b)) if b.chevauche(boite)),
        );
        let caisse = match self.etat_secousse() {
            Some((caisses, EtatSecousse::Chute(s))) => caisses
                .iter()
                .any(|c| c.boite(s).is_some_and(|b| b.chevauche(boite))),
            _ => false,
        };
        let jete = matches!(
            self.etat_jete(),
            Some((t, EtatJete::Chute(_) | EtatJete::Sonnee(_))) if t.chevauche(boite)
        );
        corps || frappe || balai || caisse || jete
    }

    /// La tête qu'on peut frapper en montée : celle d'un jeté, sonnée (pour le
    /// robot : celle de la vigie, toujours).
    pub(super) fn tete_a_frapper(&self) -> Option<Aabb> {
        if self.tour.robot {
            return self.tete_finale;
        }
        match self.etat_jete() {
            Some((t, EtatJete::Sonnee(_))) => Some(t),
            _ => None,
        }
    }

    /// Un saut sur la tête sonnée (`toucher`, en montée) : une tête en moins.
    /// Sur la vigie, à la dernière tête (ou pour le robot) : le coup final, la
    /// plongée. Sinon elle hurle et retombe, la mer recule de 3 tuiles, 3 s sans
    /// attaque ; sur la vigie, elle se jettera encore deux fois.
    pub(super) fn frapper_en_montee(&mut self) {
        if self.tete_a_frapper().is_none() {
            return;
        }
        let vigie = self.tour.jetes.len();
        let k = match self.tour.attaque {
            Some(Attaque::Jete { k, .. }) => k,
            _ => vigie,
        };
        self.pv = self.pv.saturating_sub(1);
        if self.tour.robot || (k >= vigie && self.pv == 0) {
            self.pv = 0;
            self.tour.attaque = None;
            self.annuler_le_combo();
            self.figer_la_mer();
            self.entrer(PhasePieuvre::Plongee);
            return;
        }
        // Pendant la montée, jamais la dernière tête (le choix du jeté le garantit).
        self.pv = self.pv.max(1);
        let t = self.ecoule();
        self.tour.attaque = None;
        self.annuler_le_combo();
        self.tour.lachee = self.place_du_jete(k).map(|tete| (tete, t));
        self.tour.repit = t.saturating_add(REPIT_PAS);
        // Le recul se compte en HAUTEUR : on cherche, en remontant l'horloge, le
        // dernier pas où la mer était 3 tuiles plus bas (la crue et les paliers
        // n'ont pas la vitesse de la montée).
        let h = self.tour.horloge_mer;
        let cible = self.surface_mer_a(h) + RECUL_MER;
        let avant = (0..=h)
            .rev()
            .find(|&t| self.surface_mer_a(t) >= cible - 0.01);
        self.tour.recul = h - avant.unwrap_or(0);
    }

    /// Ses têtes au début de la phase 2 : `TETES_FUITE`, au plus les cases du
    /// HUD (`pv_max`, 2 en mode facile).
    pub(super) fn pv_de_la_fuite(&self) -> u8 {
        TETES_FUITE.min(self.pv_max)
    }

    /// Pour le robot de faisabilité : aucune attaque, la tête de la vigie offerte.
    pub(super) fn offrir_la_vigie(&mut self) {
        self.tour.robot = true;
    }

    /// Un pas de la montée (après `avancer`) : la mer, l'attaque en cours et son
    /// combo, puis, si elle est libre, ce que son cerveau choisit.
    pub(super) fn avancer_tour(&mut self, grille: &dyn Grille) {
        if !self.en_montee() || self.tour.robot {
            return;
        }
        self.preparer_la_tour(grille);
        let t = self.ecoule();
        // La mer : elle recule après un coup, s'arrête pendant un jeté.
        if self.tour.recul > 0 {
            let k = self.tour.recul.min(RECUL_PAR_PAS);
            self.tour.recul -= k;
            self.tour.horloge_mer = self.tour.horloge_mer.saturating_sub(k);
        } else if !self.mer_arretee() {
            self.tour.horloge_mer = self.tour.horloge_mer.saturating_add(1);
        }
        let combo = self.avancer_le_combo(grille, t);
        // L'attaque finie s'en va. Le 1er jeté de la vigie enchaîne aussitôt sur
        // le second, plus vif, qui la laisse sonnée.
        let fin = self
            .tour
            .attaque
            .as_ref()
            .map(|a| a.debut().saturating_add(self.duree_attaque(a)));
        if fin.is_some_and(|f| t >= f) {
            match self.tour.attaque.take() {
                Some(Attaque::Jete {
                    k, sonne: false, ..
                }) => {
                    self.tour.faits += 1;
                    self.tour.attaque = Some(Attaque::Jete {
                        k,
                        debut: t,
                        vif: true,
                        sonne: true,
                    });
                }
                Some(Attaque::Jete { .. }) => {
                    self.tour.repit = t.saturating_add(APRES_JETE_PAS);
                }
                _ => {
                    let s = self.souffle();
                    self.tour.repit = self.tour.repit.max(t.saturating_add(s));
                }
            }
        }
        let Some(pepin) = self.pepin else {
            return;
        };
        // Elle grimpe pour rester sous Pépin (jamais sous la mer).
        if let Some(mer) = self.surface_mer() {
            let cible = mer.min(pepin.bas() + SOUS_PEPIN);
            let bas = self.tour.bas_geante.unwrap_or(mer);
            let bas = if cible < bas {
                (bas - GRIMPE_GEANTE).max(cible)
            } else {
                (bas + GRIMPE_GEANTE).min(cible)
            };
            self.tour.bas_geante = Some(bas.min(mer));
        }
        self.observer(&pepin);
        if self.tour.attaque.is_some()
            || combo
            || t < self.tour.repit
            || self.horloge_mer() < self.fin_de_crue()
        {
            return;
        }
        self.tour.attaque = self.choisir(grille, &pepin, t);
    }

    /// Remet la montée à neuf (début de la phase 2) ; la graine, la tentative
    /// et les places tirées de la grille restent.
    pub(super) fn remettre_la_tour(&mut self) {
        self.tour.attaque = None;
        self.tour.faits = 0;
        self.tour.repit = 0;
        self.tour.lachee = None;
        self.tour.bas_geante = None;
        self.tour.horloge_mer = 0;
        self.tour.recul = 0;
        self.tour.arrachages.clear();
        self.tour.cerveau.oublier();
        self.pv = self.pv_de_la_fuite();
    }

    /// Pépin est mort en phase 2 et réapparaît dans `occupant` (sa lanterne,
    /// n'importe laquelle) : la graine change (une tentative de plus) ; depuis
    /// la lanterne du bas, tout repart à neuf ; plus haut, ses têtes et son
    /// humeur reviennent à leur état au passage de la lanterne, les plateformes
    /// arrachées dessous restent parties et la mer revient 3 tuiles sous ses
    /// pieds.
    pub fn reprendre_a(&mut self, occupant: &Aabb) {
        if !self.en_fuite() || !matches!(self.phase(), PhasePieuvre::Furie | PhasePieuvre::Montee) {
            self.reprendre();
            return;
        }
        let marques = std::mem::take(&mut self.tour.cerveau.marques);
        let jetes_tentes = std::mem::take(&mut self.tour.cerveau.jetes_tentes);
        let arrachages = std::mem::take(&mut self.tour.arrachages);
        self.tour.cerveau.tentative = self.tour.cerveau.tentative.wrapping_add(1);
        self.reprendre();
        let t = TAILLE_TUILE;
        let au_bas = self.bache.is_none_or(|b| occupant.bas() >= b.y - 3.0 * t);
        if au_bas {
            return;
        }
        let pieds = occupant.bas();
        // Son état la première fois que Pépin est arrivé à cette hauteur (les
        // marques suivent l'ordre du temps) ; ce qui a suivi est oublié. S'il
        // n'y est jamais monté pendant cette tentative, tout repart du bas.
        let mut marques = marques;
        let Some(ici) = marques.iter().position(|m| m.0 <= pieds + 2.0) else {
            return;
        };
        self.pv = marques[ici].1.max(1);
        marques.truncate(ici + 1);
        self.tour.cerveau.marques = marques;
        self.tour.cerveau.progres = self.progres_a(pieds);
        let dessous = |y: f32| y > pieds;
        // Les jetés tentés sous la lanterne restent faits (elle ne s'y jette
        // plus) ; ceux du dessus repartent à neuf.
        let tentes: Vec<usize> = jetes_tentes
            .into_iter()
            .filter(|&k| self.tour.jetes.get(k).is_some_and(|j| dessous(j.bas())))
            .collect();
        self.tour.faits = tentes.len();
        self.tour.cerveau.jetes_tentes = tentes;
        // Les plateformes arrachées sous Pépin restent parties (horloge 0 : partie).
        self.tour.arrachages = arrachages
            .iter()
            .enumerate()
            .map(|(k, a)| {
                let haut = self.tour.arrachables[k]
                    .iter()
                    .map(|c| c.1)
                    .min()
                    .unwrap_or(0);
                a.filter(|_| dessous(haut as f32 * t)).map(|_| u16::MAX)
            })
            .collect();
        let cible = pieds + 3.0 * t;
        self.tour.horloge_mer = (0..u16::MAX)
            .find(|&h| self.surface_mer_a(h) <= cible)
            .unwrap_or(0);
    }
}
