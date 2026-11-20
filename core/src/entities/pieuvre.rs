//! La Pieuvre du port (lettre `8`) : le boss du Monde 2 (carte 9.12a).
//!
//! Demande d'AGA : « ses tentacules frappent par colonnes annoncées, on lui saute
//! dessus quand elle sort la tête. » Elle vit SOUS l'eau d'un bassin qui fait toute
//! la largeur de l'arène ; Pépin se tient sur des pontons et des caisses (ou en
//! barque, carte 9.2).
//!
//! Même modèle que le Grand Nuage (`super::nuage`) : une machine à états PURE, en
//! entiers (pas de 1/60 s), le même pas donne toujours la même pieuvre.
//!
//! ```text
//!   prélude (90) ──► colère (son motif) ──► remous (36) ──► tête sortie (120) ──► colère...
//!                                                                │ Pépin lui saute sur la tête
//!                                                                ▼
//!                                                            touchée (60) ──► colère (motif suivant)
//!                                                                │ c'était son dernier point
//!                                                                ▼
//!                                                            plongée (120) ──► vaincue (niveau fini)
//! ```
//!
//! - colère : un MOTIF de tentacules sur les 9 colonnes du bassin (2 tuiles de
//!   large chacune). Chaque tentacule est d'abord ANNONCÉ (84 pas, 1,4 s : remous
//!   et ombre sur l'eau, puis le bout du tentacule qui pointe), sans danger ; puis
//!   il se DRESSE de toute la hauteur (30 pas, 0,5 s) : mortel ; puis il redescend
//!   (12 pas), sans danger. Un motif par point perdu (`MOTIFS`), de plus en plus
//!   gros et rapide : 5 colonnes sur 9 par salve, puis 7, puis 7 ou 8 en VAGUE qui
//!   balaie le bassin, aller, un souffle, puis retour (carte 9.12d : plus lente
//!   et lisible pour un humain, nettement sous l'orage à la mesure ; carte 9.12h :
//!   encore environ 25 % plus lente).
//! - remous : la tête remonte (l'eau bouillonne à sa place), sans danger.
//! - tête sortie : la tête est posée sur l'eau, au milieu ou d'un côté
//!   (`PLACES_TETE`), 2 s. Lui tomber dessus la touche (`recoit`, même règle que
//!   l'écrasement d'un ennemi) ; la toucher AUTREMENT est mortel (`blesse`).
//! - touchée : elle grimace, sans danger, puis replonge et reprend, plus vite.
//! - plongée : à 0 point de vie, elle replonge pour de bon et lâche une clé ; à la
//!   fin, elle est vaincue et le niveau est fini (`est_vaincue`), comme le cyclone.
//!
//! Si Pépin meurt, elle reprend au prélude (`reprendre`) mais GARDE ses points
//! perdus. MODE FACILE (carte 8.14, même mécanisme que le nuage) : `passer_en_facile`
//! lui donne `PV_PIEUVRE_FACILE` points de vie et seulement les deux premiers motifs.
//!
//! LA SAISIE (carte 9.12f, demande d'AGA : « ajouter une mécanique » plutôt
//! qu'accélérer) : PAR-DESSUS les motifs, qui ne changent pas, un tentacule
//! attrape la plateforme de Pépin (`Pieuvre::plateformes`, les 4 appuis de
//! l'arène) et l'emporte sous l'eau. Annonce de 1,3 s (le tentacule s'enroule, la
//! plateforme tremble, elle porte encore), plongée de 0,5 s (elle ne porte plus :
//! Pépin tombe à l'eau, la barque le sauve), 3 s sous l'eau, puis elle remonte,
//! jamais dans Pépin. Une seule à la fois ; toutes les 9, 7 puis 5 s selon le
//! motif ; quand la tête remonte, elle lâche la plateforme. La grille du niveau ne change pas : le jeu
//! l'enveloppe d'une `GrilleSaisie` qui perce les cases englouties.
//!
//! LA PHASE 2, « fuite et montée » (carte 9.13a, demande d'AGA : « J'aime pas la
//! pieuvre, elle manque toujours de quelque chose pour un boss final… Une phase
//! deux ? »). Seulement dans une arène qui a une TOUR (une tête finale `&` dans
//! la grille, `Pieuvre::tete_finale`) : au dernier coup de la phase 1, au lieu de
//! plonger, elle entre en FURIE, casse le quai et la mer monte. Tout est dans le
//! sous-module `fuite` ; sans tour (le niveau d'essai), rien ne change.
//!
//! Notions Rust : on RÉUTILISE la `Salve` du Grand Nuage (`use super::nuage::Salve`)
//! au lieu de la recopier ; un `enum` à données (`EtatTentacule::Annonce(f32)`) ; des
//! tableaux de taille fixe (`[Aabb; NB_COLONNES]`) construits par le parseur ; un
//! SOUS-MODULE (`pieuvre/fuite.rs`) qui voit les champs privés de son parent.

mod fuite;
mod montee;
pub use fuite::*;
pub use montee::*;

use super::nuage::Salve;
use super::{Actualisable, Entite};
use crate::aabb::{Aabb, Grille, Tuile, TAILLE_TUILE};
use crate::entities::contacts::MARGE_ECRASEMENT;
use crate::physics::Environnement;
use crate::player::Joueur;

/// Points de vie de la Pieuvre : trois sauts sur la tête.
pub const PV_PIEUVRE: u8 = 3;
/// Points de vie en MODE FACILE (carte 8.14) : deux sauts suffisent.
pub const PV_PIEUVRE_FACILE: u8 = 2;
/// Prélude : 1,5 s de calme avant le premier motif (et après une mort).
pub const PRELUDE_PAS: u16 = 90;
/// Annonce d'un tentacule : 1,4 s (remous et ombre, puis le bout qui pointe).
/// Carte 9.12d (AGA : « la pieuvre est trop rapide, c'est plus dur que l'orage
/// pour un humain ») : 0,8 s auparavant, le robot réagit à l'image près, un humain
/// non (voir `un_humain_a_le_temps_de_changer_de_colonne`).
/// Carte 9.12h (AGA : « la pieuvre tape encore un peu trop vite pour un
/// humain ») : 1,2 s (72 pas) auparavant. La frappe (`DRESSE_PAS`) et la
/// redescente ne changent pas : on voit plus longtemps venir le coup, il ne dure
/// pas plus.
pub const ANNONCE_PAS: u16 = 84;
/// Moitié de l'annonce : avant, remous et ombre ; après, le bout du tentacule pointe.
pub const POINTE_PAS: u16 = ANNONCE_PAS / 2;
/// Tentacule dressé de toute la hauteur : 0,5 s, mortel.
pub const DRESSE_PAS: u16 = 30;
/// Tentacule qui redescend : 0,2 s, sans danger.
pub const REDESCENTE_PAS: u16 = 12;
/// Durée d'une salve : annonce, frappe, redescente.
pub const SALVE_PAS: u16 = ANNONCE_PAS + DRESSE_PAS + REDESCENTE_PAS;
/// La tête remonte : 0,6 s de remous à sa place, sans danger (c'est son annonce).
pub const REMOUS_PAS: u16 = 36;
/// La tête est sortie, posée sur l'eau : 2 s pour lui sauter dessus.
pub const TETE_PAS: u16 = 120;
/// Touchée : 1,5 s de grimace, sans danger (carte 9.12d : un vrai souffle après
/// chaque coup reçu, 1 s auparavant).
pub const TOUCHEE_PAS: u16 = 90;
/// Vaincue : elle replonge pendant 2 s, puis le niveau est fini.
pub const PLONGEE_PAS: u16 = 120;
/// Taille de la tête (px) : trois cases de large, deux de haut.
pub const TAILLE_TETE: (f32, f32) = (48.0, 32.0);
/// Colonnes de tentacules : de 4 colonnes à gauche de celle du `8` à 4 à droite.
pub const PORTEE_COLONNES: i8 = 4;
/// Nombre de colonnes de tentacules (9).
pub const NB_COLONNES: usize = 2 * PORTEE_COLONNES as usize + 1;
/// Largeur (px) d'une colonne : deux tuiles.
pub const LARGEUR_COLONNE: f32 = 32.0;
/// Marge (px) de chaque côté du tentacule dans sa colonne : son corps fait 24 px.
/// Entre deux colonnes voisines, il reste 8 px : moins que Pépin (10 px), on ne
/// s'abrite donc pas « entre deux » : la colonne annoncée est la colonne qui tue.
pub const MARGE_TENTACULE: f32 = 4.0;
/// Places de la tête, en colonnes depuis celle du `8` : au milieu après le motif 1,
/// à gauche après le 2, à droite après le 3 (il faut traverser l'arène).
pub const PLACES_TETE: [i8; 3] = [0, -3, 3];

// ---------------------------------------------------------------------------
// La SAISIE des plateformes (carte 9.12f)
// ---------------------------------------------------------------------------

/// Annonce d'une saisie : 1,3 s (au moins 1,2 s, demande d'AGA). Un tentacule sort
/// de l'eau et s'enroule autour de la plateforme, qui tremble. Sans danger.
pub const ANNONCE_SAISIE_PAS: u16 = 78;
/// La plateforme s'enfonce sous l'eau : 0,5 s. Ses cases ne portent PLUS dès le
/// début de la plongée (Pépin, s'il est dessus, tombe à l'eau).
pub const PLONGEE_SAISIE_PAS: u16 = 30;
/// La plateforme reste sous l'eau 3 s (des remous marquent sa place).
pub const SOUS_L_EAU_PAS: u16 = 180;
/// Elle remonte : 0,4 s, toujours sans porter ; solide à la fin seulement.
pub const REMONTEE_SAISIE_PAS: u16 = 24;
/// Durée totale d'une saisie (si rien ne retient la remontée).
pub const SAISIE_PAS: u16 =
    ANNONCE_SAISIE_PAS + PLONGEE_SAISIE_PAS + SOUS_L_EAU_PAS + REMONTEE_SAISIE_PAS;
/// Une saisie toutes les 9 s, puis 7 s, puis 5 s (par motif, donc par point
/// perdu) : plus elle perd de vie, plus elle insiste. L'annonce, elle, ne
/// raccourcit jamais.
pub const INTERVALLES_SAISIE: [u16; PV_PIEUVRE as usize] = [540, 420, 300];
/// Première saisie (et après une mort) : 3 s après le début du prélude, soit
/// 1,5 s après le début du premier motif.
pub const PREMIERE_SAISIE_PAS: u16 = 180;

/// Ce que fait la plateforme saisie pendant un pas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EtatSaisie {
    /// Le tentacule s'enroule, la plateforme tremble (avancement de 0.0 à 1.0) :
    /// elle porte encore.
    Annonce(f32),
    /// Elle s'enfonce (avancement de 0.0 à 1.0) : elle ne porte plus.
    Plongee(f32),
    /// Sous l'eau : invisible, des remous à sa place.
    SousLEau,
    /// Elle remonte (avancement de 0.0 à 1.0) : solide à la fin seulement.
    Remontee(f32),
}

/// Une saisie en cours : la plateforme visée (indice dans `Pieuvre::plateformes`)
/// et les pas écoulés depuis son début. UNE SEULE à la fois : la pieuvre n'en
/// garde qu'une (`Option<Saisie>`), il reste donc toujours les autres appuis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Saisie {
    pub cible: usize,
    ecoule: u16,
}

impl Saisie {
    /// L'état de la plateforme au pas en cours (voir `EtatSaisie`).
    pub fn etat(&self) -> EtatSaisie {
        let t = self.ecoule;
        let a = ANNONCE_SAISIE_PAS;
        let p = a + PLONGEE_SAISIE_PAS;
        let s = p + SOUS_L_EAU_PAS;
        if t < a {
            EtatSaisie::Annonce(f32::from(t) / f32::from(a))
        } else if t < p {
            EtatSaisie::Plongee(f32::from(t - a) / f32::from(PLONGEE_SAISIE_PAS))
        } else if t < s {
            EtatSaisie::SousLEau
        } else {
            EtatSaisie::Remontee(f32::from(t - s) / f32::from(REMONTEE_SAISIE_PAS))
        }
    }

    /// La plateforme porte-t-elle encore (seulement pendant l'annonce) ?
    pub fn porte(&self) -> bool {
        matches!(self.etat(), EtatSaisie::Annonce(_))
    }
}

/// Une grille dont quelques cases solides sont percées le temps d'une saisie : la
/// plateforme engloutie. Elle « enveloppe » la vraie grille (un EMPRUNT, `&dyn
/// Grille`) au lieu de la modifier : la grille du niveau ne change jamais, et le
/// trou disparaît tout seul quand le jeu cesse de l'envelopper.
pub struct GrilleSaisie<'a> {
    pub grille: &'a dyn Grille,
    /// Les cases `(tx, ty)` qui ne portent plus.
    pub trous: &'a [(i32, i32)],
}

impl Grille for GrilleSaisie<'_> {
    fn tuile(&self, tx: i32, ty: i32) -> Tuile {
        if self.trous.contains(&(tx, ty)) {
            Tuile::Vide
        } else {
            self.grille.tuile(tx, ty)
        }
    }
}

/// Boîte (px) d'une case de la grille.
fn boite_case((tx, ty): (i32, i32)) -> Aabb {
    let t = TAILLE_TUILE;
    Aabb::new(tx as f32 * t, ty as f32 * t, t, t)
}

/// Étendue horizontale (px, de gauche à droite) des cases d'une plateforme.
fn etendue(cases: &[(i32, i32)]) -> (f32, f32) {
    let x0 = cases.iter().map(|c| c.0).min().unwrap_or(0);
    let x1 = cases.iter().map(|c| c.0).max().unwrap_or(0);
    (x0 as f32 * TAILLE_TUILE, (x1 + 1) as f32 * TAILLE_TUILE)
}

/// Les cases englouties de toutes les pieuvres du niveau (au plus une) : le jeu
/// en perce la grille à chaque pas (`GrilleSaisie`). Vide sans pieuvre.
pub fn cases_englouties(entites: &[Entite]) -> Vec<(i32, i32)> {
    entites
        .iter()
        .filter_map(|e| match e {
            Entite::Pieuvre(p) => Some(p.cases_englouties()),
            _ => None,
        })
        .flatten()
        .collect()
}

/// Les trois motifs, un par point perdu, de plus en plus gros et rapides. Les
/// bandes sont en COLONNES depuis celle du `8` (-4 tout à gauche, 4 tout à droite).
///
/// Carte 9.12d (AGA, après avoir joué : « la pieuvre est trop rapide, c'est plus
/// dur que l'orage pour un humain ») : même principe que la 9.12c (« un peu moins
/// dure que le grand nuage, qui accélère au fur et à mesure qu'elle perd de la
/// vie et qui fait des attaques plus grosses »), mais partant de plus bas : une
/// salve toutes les 100, 78, puis 60 pas (66, 48, 30 auparavant), annonce de
/// 1,2 s, une salve de moins par motif, et un souffle de 140 pas entre l'aller et
/// le retour de la vague.
///
/// Carte 9.12h (AGA : « la pieuvre tape encore un peu trop vite pour un
/// humain ») : une salve dure 114 pas (1,9 s) ; annoncées toutes les 78 ou 60
/// pas, la suivante s'annonçait pendant que la précédente frappait encore. Les
/// débuts de chaque motif sont donc ÉTIRÉS, tous du même facteur : une salve
/// toutes les 120, 104, puis 84 pas (100, 78, 60 auparavant), soit x 1,2, x 4/3
/// et x 1,4 ; au motif 3, l'écart avant l'aller passe de 100 à 140 pas et le
/// souffle de 140 à 196. La fin de chaque motif (donc la sortie de la tête) suit
/// toute seule : `duree_motif` part du dernier début. Colonnes frappées, saisie,
/// tête et points de vie : inchangés.
///
/// Carte 9.12e (AGA : la Pieuvre doit rester le boss au-dessus des niveaux 1 à 4
/// du chapitre 2 ; sans le petit ponton du milieu, l'Habitué n'y mourait plus que
/// 0,44 fois) : des attaques PLUS LARGES, jamais plus rapides. Chaque salve des
/// motifs 1 et 2 frappe une colonne de plus ; au motif 3, 4 salves sur 6 (la 1re
/// et la 3e restent à 7 colonnes : à 8, l'aller de la vague n'a plus d'issue
/// qu'un humain rejoigne à temps). Débuts des salves, annonce, frappe : inchangés.
/// Part du bassin frappée par salve : 56 %, 78 %, 85 % (44, 67, 78 % avant).
/// Réglés à la MESURE (`difficulte::tests::mesure_d_un_niveau_du_monde_2`,
/// CLE=pieuvre).
/// - Motif 1 : 5 colonnes sur 9, une salve toutes les 120 pas : les bords et un
///   peu du milieu, puis le milieu, puis les bords... on fait la navette.
/// - Motif 2 : 7 colonnes sur 9, toutes les 104 pas : trois bandes, deux trous
///   d'une colonne qui sautent d'un côté à l'autre, d'une colonne à la fois.
/// - Motif 3 : 7 ou 8 colonnes sur 9, toutes les 84 pas : deux trous d'une
///   colonne se rejoignent au milieu ; ce trou glisse vers la droite (l'ALLER de
///   la VAGUE), elle souffle, puis il revient au milieu (le RETOUR), et la tête
///   sort à droite : on retraverse l'arène.
///
/// Toujours une issue : le test `chaque_motif_se_traverse` (le raisonnement du
/// nuage, plus strict) vérifie qu'on peut toujours s'en sortir en courant, et
/// `un_humain_a_le_temps_de_changer_de_colonne` qu'un humain a le temps de voir
/// l'annonce avant de partir.
pub const MOTIFS: [&[Salve]; PV_PIEUVRE as usize] = [
    &[
        // Le bord gauche, et trois colonnes isolées : au milieu ou à droite.
        Salve {
            debut: 0,
            bandes: &[(-4, -3), (-1, -1), (2, 2), (4, 4)],
        },
        // Le bord gauche et le milieu : à gauche ou à droite.
        Salve {
            debut: 120,
            bandes: &[(-4, -4), (-2, 1)],
        },
        // Les bords : au milieu.
        Salve {
            debut: 240,
            bandes: &[(-4, -2), (3, 4)],
        },
        // Le milieu et le bord droit : à gauche (la tête sort ensuite au milieu).
        Salve {
            debut: 360,
            bandes: &[(-1, 2), (4, 4)],
        },
    ],
    &[
        Salve {
            debut: 0,
            bandes: &[(-4, -3), (-1, 1), (3, 4)],
        },
        Salve {
            debut: 104,
            bandes: &[(-4, -2), (0, 2), (4, 4)],
        },
        Salve {
            debut: 208,
            bandes: &[(-4, -3), (-1, 2), (4, 4)],
        },
        Salve {
            debut: 312,
            bandes: &[(-4, -2), (0, 2), (4, 4)],
        },
        // La gauche et la droite : au milieu, puis à gauche (la tête y sort).
        Salve {
            debut: 416,
            bandes: &[(-4, -2), (0, 1), (3, 4)],
        },
    ],
    &[
        // Deux trous d'une colonne, à 2 colonnes au plus de chaque tentacule...
        Salve {
            debut: 0,
            bandes: &[(-4, -3), (-1, 1), (3, 4)],
        },
        // ... qui se rejoignent au milieu (140 pas : le temps d'y courir).
        Salve {
            debut: 140,
            bandes: &[(-4, -1), (1, 4)],
        },
        // L'aller : le trou glisse vers la droite, une colonne toutes les 84 pas...
        Salve {
            debut: 224,
            bandes: &[(-4, -1), (2, 4)],
        },
        Salve {
            debut: 308,
            bandes: &[(-4, 0), (2, 4)],
        },
        // ... elle souffle (196 pas), puis le retour, au milieu (la tête sort
        // ensuite à droite).
        Salve {
            debut: 504,
            bandes: &[(-4, -1), (1, 4)],
        },
        Salve {
            debut: 588,
            bandes: &[(-4, -1), (1, 4)],
        },
    ],
];

/// Durée (pas) du motif `motif` : le début de sa dernière salve, plus une salve.
pub fn duree_motif(motif: usize) -> u16 {
    MOTIFS[motif].iter().map(|s| s.debut).max().unwrap_or(0) + SALVE_PAS
}

/// Ce que fait le tentacule `decalage` (de -4 à 4) au pas `t` du motif `motif`.
/// Si deux salves se chevauchent, le tentacule dressé l'emporte. Fonction pure.
pub fn etat_tentacule(motif: usize, t: u16, decalage: i8) -> EtatTentacule {
    let mut etat = EtatTentacule::Calme;
    for s in MOTIFS[motif].iter().filter(|s| s.couvre(decalage)) {
        if t < s.debut || t >= s.debut + SALVE_PAS {
            continue;
        }
        let dans = t - s.debut;
        if dans < ANNONCE_PAS {
            etat = EtatTentacule::Annonce(dans as f32 / ANNONCE_PAS as f32);
        } else if dans < ANNONCE_PAS + DRESSE_PAS {
            return EtatTentacule::Dresse;
        } else {
            let fin = dans - ANNONCE_PAS - DRESSE_PAS;
            etat = EtatTentacule::Redescente(fin as f32 / REDESCENTE_PAS as f32);
        }
    }
    etat
}

/// L'état d'un tentacule pendant un pas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EtatTentacule {
    /// Rien ne sort de l'eau.
    Calme,
    /// Annoncé (sans danger) : avancement de l'annonce, de 0.0 à 1.0 (remous et
    /// ombre jusqu'à 0.5, le bout du tentacule ensuite).
    Annonce(f32),
    /// Dressé de toute la hauteur : mortel.
    Dresse,
    /// Il redescend (sans danger) : avancement, de 0.0 à 1.0.
    Redescente(f32),
}

/// Phase de la Pieuvre (voir le schéma en tête de module).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhasePieuvre {
    Prelude,
    Colere,
    Remous,
    Tete,
    Touchee,
    Plongee,
    Vaincue,
    /// Phase 2 (carte 9.13a) : elle hurle, l'écran tremble, la bâche se déchire
    /// et un jet d'eau emporte Pépin dans la tour ; le quai casse à la fin.
    Furie,
    /// Phase 2 : la mer monte, les tentacules jaillissent de dessous, la tête
    /// attend en haut de la tour.
    Montee,
}

#[derive(Debug, Clone)]
pub struct Pieuvre {
    /// Les 9 colonnes de tentacules (le corps du tentacule, 24 px de large), de la
    /// surface de l'eau jusqu'en haut de la carte, de -4 à 4.
    pub colonnes: [Aabb; NB_COLONNES],
    /// Les trois places de la tête (48 x 32, posée sur l'eau), dans l'ordre de
    /// `PLACES_TETE` : milieu, gauche, droite.
    pub places: [Aabb; 3],
    phase: PhasePieuvre,
    /// Pas écoulés depuis le début de la phase.
    ecoule: u16,
    pv: u8,
    pv_max: u8,
    /// Place de la tête (indice dans `places`), choisie quand elle remonte.
    place: usize,
    /// Les plateformes qu'elle peut saisir (carte 9.12f) : chacune est la liste
    /// de ses cases `(tx, ty)`, posée par le parseur (`level::ascii`).
    pub plateformes: Vec<Vec<(i32, i32)>>,
    /// La saisie en cours, s'il y en a une (jamais deux : c'est un `Option`).
    saisie: Option<Saisie>,
    /// Pas avant la prochaine saisie (le compte s'arrête quand la tête sort).
    avant_saisie: u16,
    /// Où était Pépin au dernier pas (`voir_pepin`) : pour viser sa plateforme
    /// et ne jamais la faire remonter dans lui.
    pepin: Option<Aabb>,
    /// Phase 2 (carte 9.13a) : la BÂCHE (`_`), le plafond de l'arène : une boîte
    /// solide (`Entite::solide`) tant que la phase 1 dure, déchirée dès la furie.
    pub bache: Option<Aabb>,
    /// La tête en haut de la tour (`&`, 48 x 32) ; `None` : pas de phase 2.
    pub tete_finale: Option<Aabb>,
    /// Ordonnées (px) où la mer marque un palier de répit (sous les corniches
    /// larges de la tour, posées par le parseur), de la plus basse à la plus haute.
    pub paliers: Vec<f32>,
    /// Ordonnée (px) où la mer s'arrête pour de bon, sous la vigie.
    pub arret_mer: f32,
    /// La phase 2 a commencé (la bâche est déchirée, puis le quai cassé).
    fuite: bool,
    /// Le dernier tentacule de la montée : sa colonne et son pas de départ.
    tentacule_montee: Option<(usize, u16)>,
    /// La montée de la carte 9.13b : rencontres, balayeurs, arrachages
    /// (`pieuvre::montee`).
    pub tour: Tour,
}

impl Pieuvre {
    /// Une pieuvre neuve, au prélude, avec ses 3 points de vie (et aucune
    /// plateforme à saisir : le parseur les ajoute).
    pub fn new(colonnes: [Aabb; NB_COLONNES], places: [Aabb; 3]) -> Self {
        Self {
            colonnes,
            places,
            phase: PhasePieuvre::Prelude,
            ecoule: 0,
            pv: PV_PIEUVRE,
            pv_max: PV_PIEUVRE,
            place: 0,
            plateformes: Vec::new(),
            saisie: None,
            avant_saisie: PREMIERE_SAISIE_PAS,
            pepin: None,
            bache: None,
            tete_finale: None,
            paliers: Vec::new(),
            arret_mer: 0.0,
            fuite: false,
            tentacule_montee: None,
            tour: Tour::default(),
        }
    }

    /// La saisie en cours (pour le dessin et les tests).
    pub fn saisie(&self) -> Option<Saisie> {
        self.saisie
    }

    /// Les cases qui ne portent plus en ce moment : celles de la plateforme
    /// saisie, de sa plongée jusqu'à la fin de sa remontée. Le jeu en perce la
    /// grille (`GrilleSaisie`). En phase 2 (carte 9.13a) : la bâche déchirée, et,
    /// après la furie, tout le quai de l'arène (`fuite::cases_de_la_fuite`).
    pub fn cases_englouties(&self) -> Vec<(i32, i32)> {
        if self.fuite {
            return self.cases_de_la_fuite();
        }
        match self.saisie {
            Some(s) if !s.porte() => self.plateformes[s.cible].clone(),
            _ => Vec::new(),
        }
    }

    /// Le jeu dit à la pieuvre où est Pépin, à chaque pas, avant de la faire
    /// avancer : elle vise sa plateforme, et attend qu'il en sorte pour la
    /// faire remonter.
    pub fn voir_pepin(&mut self, boite: Aabb) {
        self.pepin = Some(boite);
    }

    /// La plateforme à saisir : celle sous Pépin, ou la plus proche de lui (il
    /// faut changer d'appui) ; sans Pépin connu, chacune son tour.
    fn choisir_cible(&self, tour: usize) -> usize {
        let Some(pepin) = self.pepin else {
            return tour % self.plateformes.len();
        };
        let milieu = pepin.x + pepin.w / 2.0;
        let distance = |cases: &Vec<(i32, i32)>| {
            let (g, d) = etendue(cases);
            (g - milieu).max(milieu - d).max(0.0)
        };
        (0..self.plateformes.len())
            .min_by(|&a, &b| {
                distance(&self.plateformes[a]).total_cmp(&distance(&self.plateformes[b]))
            })
            .unwrap_or(0)
    }

    /// Un pas des saisies. Le compte à rebours ne tourne qu'au prélude et en
    /// colère ; une saisie ne COMMENCE qu'en colère (jamais tête sortie), et
    /// jamais pendant une autre.
    fn avancer_saisie(&mut self) {
        if let Some(s) = &mut self.saisie {
            let fin = s.ecoule + 1 >= SAISIE_PAS;
            // La plateforme redevient solide au dernier pas : jamais dans Pépin.
            // S'il est dans son volume, elle attend sous la surface.
            let gene = self.pepin.is_some_and(|p| {
                self.plateformes[s.cible]
                    .iter()
                    .any(|&c| boite_case(c).chevauche(&p))
            });
            if fin && gene {
                return;
            }
            s.ecoule += 1;
            if s.ecoule >= SAISIE_PAS {
                self.saisie = None;
            }
        }
        if !matches!(self.phase, PhasePieuvre::Prelude | PhasePieuvre::Colere) {
            // La tête remonte : la pieuvre a besoin de ses bras, elle LÂCHE la
            // plateforme (règle d'équité : la tête se rejoint toujours depuis
            // les 4 appuis). Annoncée, elle ne coule pas ; sous l'eau, elle
            // remonte tout de suite, avant que la tête sorte (36 pas de remous
            // contre 24 de remontée).
            if let Some(s) = &mut self.saisie {
                match s.etat() {
                    EtatSaisie::Annonce(_) => self.saisie = None,
                    EtatSaisie::Plongee(_) | EtatSaisie::SousLEau => {
                        s.ecoule = SAISIE_PAS - REMONTEE_SAISIE_PAS;
                    }
                    EtatSaisie::Remontee(_) => {}
                }
            }
            return;
        }
        self.avant_saisie = self.avant_saisie.saturating_sub(1);
        let peut = self.phase == PhasePieuvre::Colere
            && self.saisie.is_none()
            && !self.plateformes.is_empty();
        if self.avant_saisie == 0 && peut {
            let tour = usize::from(self.ecoule);
            self.saisie = Some(Saisie {
                cible: self.choisir_cible(tour),
                ecoule: 0,
            });
            self.avant_saisie = INTERVALLES_SAISIE[self.motif()];
        }
    }

    /// Mode facile (carte 8.14) : `PV_PIEUVRE_FACILE` points de vie, donc les deux
    /// premiers motifs seulement. À appeler sur une pieuvre neuve.
    pub fn passer_en_facile(&mut self) {
        self.pv_max = PV_PIEUVRE_FACILE;
        self.pv = self.pv.min(PV_PIEUVRE_FACILE);
    }

    pub fn pv(&self) -> u8 {
        self.pv
    }

    pub fn pv_max(&self) -> u8 {
        self.pv_max
    }

    /// Le combat est-il en mode facile ? (Le dessin la pâlit, comme le nuage.)
    pub fn est_facile(&self) -> bool {
        self.pv_max < PV_PIEUVRE
    }

    pub fn phase(&self) -> PhasePieuvre {
        self.phase
    }

    /// Pas écoulés depuis le début de la phase en cours.
    pub fn ecoule(&self) -> u16 {
        self.ecoule
    }

    /// Le motif joué en colère : le nombre de points déjà perdus depuis `pv_max`.
    pub fn motif(&self) -> usize {
        usize::from(self.pv_max - self.pv.clamp(1, self.pv_max))
    }

    /// Durée de la phase en cours (`None` : vaincue, pour toujours).
    pub fn duree_phase(&self) -> Option<u16> {
        match self.phase {
            PhasePieuvre::Prelude => Some(PRELUDE_PAS),
            PhasePieuvre::Colere => Some(duree_motif(self.motif())),
            PhasePieuvre::Remous => Some(REMOUS_PAS),
            PhasePieuvre::Tete => Some(TETE_PAS),
            PhasePieuvre::Touchee => Some(TOUCHEE_PAS),
            PhasePieuvre::Plongee => Some(PLONGEE_PAS),
            PhasePieuvre::Furie => Some(FURIE_PAS),
            PhasePieuvre::Vaincue | PhasePieuvre::Montee => None,
        }
    }

    /// La place de la tête (sa boîte), de la remontée à la plongée ; `None` quand
    /// elle est sous l'eau.
    pub fn tete(&self) -> Option<Aabb> {
        match self.phase {
            PhasePieuvre::Remous
            | PhasePieuvre::Tete
            | PhasePieuvre::Touchee
            | PhasePieuvre::Furie => Some(self.places[self.place]),
            // Phase 2 (carte 9.13a) : la tête attend en haut de la tour, et y
            // replonge. Carte 9.13b : pendant une rencontre, la tête agrippée.
            PhasePieuvre::Montee => self.rencontre().map(|r| r.tete).or(self.tete_finale),
            PhasePieuvre::Plongee if self.fuite => self.tete_finale,
            PhasePieuvre::Plongee => Some(self.places[self.place]),
            _ => None,
        }
    }

    /// L'état du tentacule `i` (0 = le plus à gauche) : toujours `Calme` hors de la
    /// colère.
    pub fn etat_tentacule(&self, i: usize) -> EtatTentacule {
        if self.phase != PhasePieuvre::Colere || i >= NB_COLONNES {
            return EtatTentacule::Calme;
        }
        etat_tentacule(self.motif(), self.ecoule, i as i8 - PORTEE_COLONNES)
    }

    /// Pépin lui tombe-t-il sur la tête, sortie ? Même règle que l'écrasement d'un
    /// ennemi : il descend, et ses pieds ne dépassent pas de plus de
    /// `MARGE_ECRASEMENT` le dessus de la tête.
    pub fn recoit(&self, joueur: &Joueur) -> bool {
        let tete = match self.phase {
            PhasePieuvre::Tete => self.places[self.place],
            // Le coup final (carte 9.13a) : la tête du haut de la tour ; carte
            // 9.13b : avant elle, la tête de chaque rencontre, une fois agrippée.
            PhasePieuvre::Montee => match self.tete_a_frapper() {
                Some(t) => t,
                None => return false,
            },
            _ => return false,
        };
        joueur.vitesse.y > 0.0
            && joueur.boite.chevauche(&tete)
            && joueur.boite.bas() - tete.y <= MARGE_ECRASEMENT
    }

    /// `boite` meurt-elle ? Un tentacule dressé la touche, ou la tête sortie la
    /// touche (autrement qu'en tombant dessus : `recoit` est vérifié avant). En
    /// phase 2 : la mer en furie, ou un tentacule de la montée (`fuite`).
    pub fn blesse(&self, boite: &Aabb) -> bool {
        if self.blesse_en_fuite(boite) || self.blesse_en_montee(boite) {
            return true;
        }
        let tete = self.phase == PhasePieuvre::Tete && boite.chevauche(&self.places[self.place]);
        tete || (0..NB_COLONNES).any(|i| {
            self.etat_tentacule(i) == EtatTentacule::Dresse && self.colonnes[i].chevauche(boite)
        })
    }

    /// Un saut sur la tête : un point en moins ; touchée, ou, au dernier point, la
    /// plongée. Sans effet si la tête n'est pas sortie. Carte 9.13a : dans une
    /// arène à tour, le dernier point de la phase 1 donne la FURIE (la phase 2) ;
    /// le coup sur la tête du haut de la tour, la plongée.
    pub fn toucher(&mut self) {
        if self.phase == PhasePieuvre::Montee && self.gagner_la_rencontre() {
            return;
        }
        if self.phase == PhasePieuvre::Montee && self.tete_finale.is_some() {
            self.pv = 0;
            self.figer_la_mer();
            self.entrer(PhasePieuvre::Plongee);
            return;
        }
        if self.phase != PhasePieuvre::Tete {
            return;
        }
        self.pv = self.pv.saturating_sub(1);
        let phase = if self.pv > 0 {
            PhasePieuvre::Touchee
        } else if self.tete_finale.is_some() {
            self.commencer_la_fuite();
            PhasePieuvre::Furie
        } else {
            PhasePieuvre::Plongee
        };
        self.entrer(phase);
    }

    /// Pépin est mort : elle reprend au prélude, avec ses points perdus. Sans effet
    /// quand elle plonge déjà pour de bon (ou une fois vaincue). En phase 2 (carte
    /// 9.13a) : au DÉBUT DE LA PHASE 2 (la mer en bas, aucun tentacule).
    pub fn reprendre(&mut self) {
        if self.fuite {
            if matches!(self.phase, PhasePieuvre::Furie | PhasePieuvre::Montee) {
                self.entrer(PhasePieuvre::Montee);
                self.tentacule_montee = None;
                self.remettre_la_tour();
            }
            return;
        }
        if !matches!(self.phase, PhasePieuvre::Plongee | PhasePieuvre::Vaincue) {
            self.entrer(PhasePieuvre::Prelude);
        }
        // La plateforme saisie lâchée remonte d'un coup, et le compte repart.
        self.saisie = None;
        self.avant_saisie = PREMIERE_SAISIE_PAS;
    }

    /// La tête sort tout de suite, à la place `place` (robot de faisabilité, qui ne
    /// connaît pas le temps : un joueur patient attend qu'elle sorte ; tests).
    pub fn sortir_la_tete(&mut self, place: usize) {
        if self.pv > 0 {
            self.entrer(PhasePieuvre::Tete);
            self.place = place.min(self.places.len() - 1);
        }
    }

    /// Est-elle vaincue (le niveau est fini) ?
    pub fn est_vaincue(&self) -> bool {
        self.phase == PhasePieuvre::Vaincue
    }

    /// Avancement de la plongée finale, de 0.0 à 1.0 (1.0 une fois vaincue).
    pub fn progression_plongee(&self) -> f32 {
        match self.phase {
            PhasePieuvre::Plongee => self.ecoule as f32 / PLONGEE_PAS as f32,
            PhasePieuvre::Vaincue => 1.0,
            _ => 0.0,
        }
    }

    /// Décale la pieuvre de `pas` pas dans le temps (en avance si positif, en retard
    /// dans la phase en cours si négatif), comme `GrandNuage::decaler` : le robot de
    /// QA vérifie qu'un plan tient à une ou deux images près.
    pub fn decaler(&mut self, pas: i32) {
        if pas >= 0 {
            for _ in 0..pas {
                self.avancer();
            }
        } else {
            self.ecoule = self.ecoule.saturating_sub(pas.unsigned_abs() as u16);
        }
    }

    fn avancer(&mut self) {
        // La montée (carte 9.13a) n'a pas de fin fixée : elle dure jusqu'au coup
        // final. Son temps fait monter la mer (`fuite::surface_mer`).
        if self.phase == PhasePieuvre::Montee {
            self.ecoule = self.ecoule.saturating_add(1);
            return;
        }
        let Some(duree) = self.duree_phase() else {
            return; // vaincue : plus rien ne bouge
        };
        self.ecoule = self.ecoule.saturating_add(1);
        if self.ecoule < duree {
            return;
        }
        let suivante = match self.phase {
            PhasePieuvre::Prelude | PhasePieuvre::Tete | PhasePieuvre::Touchee => {
                PhasePieuvre::Colere
            }
            PhasePieuvre::Colere => {
                // La tête remonte à la place du motif qui vient de finir.
                self.place = self.motif() % PLACES_TETE.len();
                PhasePieuvre::Remous
            }
            PhasePieuvre::Remous => PhasePieuvre::Tete,
            PhasePieuvre::Furie | PhasePieuvre::Montee => PhasePieuvre::Montee,
            PhasePieuvre::Plongee | PhasePieuvre::Vaincue => PhasePieuvre::Vaincue,
        };
        self.entrer(suivante);
    }

    fn entrer(&mut self, phase: PhasePieuvre) {
        self.phase = phase;
        self.ecoule = 0;
    }
}

impl Actualisable for Pieuvre {
    fn actualiser(&mut self, env: &Environnement<'_>) {
        self.avancer_saisie();
        self.avancer();
        self.avancer_tour();
        self.avancer_montee(env.grille);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::GrilleTuiles;

    /// Une pieuvre dont le `8` est en (9, 9) : surface de l'eau à y = 144.
    fn pieuvre() -> Pieuvre {
        let colonnes = std::array::from_fn(|i| {
            Aabb::new(16.0 + 32.0 * i as f32 + MARGE_TENTACULE, 0.0, 24.0, 144.0)
        });
        let places = PLACES_TETE.map(|k| {
            let milieu = 160.0 + 32.0 * f32::from(k);
            Aabb::new(milieu - 24.0, 112.0, TAILLE_TETE.0, TAILLE_TETE.1)
        });
        Pieuvre::new(colonnes, places)
    }

    fn pas(p: &mut Pieuvre, k: u32) {
        let g = GrilleTuiles::vide(4, 4);
        let env = Environnement::simple(&g);
        for _ in 0..k {
            p.actualiser(&env);
        }
    }

    /// La règle centrale : tentacules annoncés puis mortels, et la tête sortie ne se
    /// touche qu'en lui tombant dessus (un point de moins à chaque fois, puis la
    /// plongée) ; la toucher de côté est mortel, et rien ne la touche sous l'eau.
    #[test]
    fn la_tete_ne_se_touche_qu_en_lui_tombant_dessus() {
        let mut p = pieuvre();
        // Prélude, puis le 1er motif : sa 1re salve frappe le bord gauche et trois
        // colonnes isolées, dont la colonne 2 (corps de x = 212 à 236), annoncée 84
        // pas (sans danger), dressée 30 (mortelle), puis elle redescend. La colonne
        // 0 (x = 148 à 172) est libre.
        let sur_la_colonne = Aabb::new(218.0, 114.0, 10.0, 14.0);
        pas(&mut p, PRELUDE_PAS as u32);
        assert_eq!(p.phase(), PhasePieuvre::Colere);
        assert!(matches!(p.etat_tentacule(6), EtatTentacule::Annonce(_)));
        assert!(!p.blesse(&sur_la_colonne));
        pas(&mut p, ANNONCE_PAS as u32);
        assert_eq!(p.etat_tentacule(6), EtatTentacule::Dresse);
        assert!(p.blesse(&sur_la_colonne));
        assert!(!p.blesse(&Aabb::new(154.0, 114.0, 10.0, 14.0)));
        pas(&mut p, DRESSE_PAS as u32);
        assert!(matches!(p.etat_tentacule(6), EtatTentacule::Redescente(_)));
        assert!(!p.blesse(&sur_la_colonne));
        // Sous l'eau, la tête ne se touche pas.
        let mut j = Joueur::new(150.0, 100.0);
        j.vitesse.y = 60.0;
        assert!(!p.recoit(&j));
        // Fin du motif : remous (sans danger), puis la tête sort au milieu.
        pas(&mut p, (duree_motif(0) - ANNONCE_PAS - DRESSE_PAS) as u32);
        assert_eq!(
            (p.phase(), p.tete()),
            (PhasePieuvre::Remous, Some(p.places[0]))
        );
        let tete = p.places[0];
        assert!(!p.blesse(&tete));
        pas(&mut p, REMOUS_PAS as u32);
        assert_eq!(p.phase(), PhasePieuvre::Tete);
        // De côté (Pépin qui marche dedans), ou par-dessous : mortel, pas de point.
        let mut de_cote = Joueur::new(tete.x - 6.0, tete.y + 10.0);
        de_cote.vitesse.x = 90.0;
        assert!(!p.recoit(&de_cote) && p.blesse(&de_cote.boite));
        let mut monte = Joueur::new(tete.x + 19.0, tete.y - 12.0);
        monte.vitesse.y = -100.0;
        assert!(!p.recoit(&monte) && p.blesse(&monte.boite));
        // En lui tombant dessus (pieds 2 px dans le dessus) : touchée.
        let mut tombe = Joueur::new(tete.x + 19.0, tete.y - 12.0);
        tombe.vitesse.y = 120.0;
        assert!(p.recoit(&tombe));
        p.toucher();
        assert_eq!((p.phase(), p.pv()), (PhasePieuvre::Touchee, 2));
        assert!(!p.blesse(&tombe.boite), "touchée, elle ne blesse plus");
        // Une mort de Pépin : elle reprend au prélude mais GARDE le point perdu.
        p.reprendre();
        assert_eq!(
            (p.phase(), p.pv(), p.motif()),
            (PhasePieuvre::Prelude, 2, 1)
        );
        // Les deux derniers coups (la tête sort à gauche, puis à droite), puis la
        // plongée : vaincue, le niveau est fini.
        for (place, pv) in [(1, 1), (2, 0)] {
            pas(&mut p, 1);
            while p.phase() != PhasePieuvre::Tete {
                pas(&mut p, 1);
            }
            assert_eq!(p.tete(), Some(p.places[place]));
            p.toucher();
            assert_eq!(p.pv(), pv);
        }
        assert_eq!(p.phase(), PhasePieuvre::Plongee);
        p.reprendre();
        assert_eq!(
            p.phase(),
            PhasePieuvre::Plongee,
            "la plongée ne s'annule pas"
        );
        pas(&mut p, PLONGEE_PAS as u32);
        assert!(p.est_vaincue());
        // Chaque coup la rend plus rapide : `chaque_point_perdu_frappe_plus_gros_et_plus_vite`
        // (depuis la carte 9.12d, le 3e motif dure plus longtemps : il a un souffle
        // entre l'aller et le retour de la vague).
        // Mode facile : 2 points de vie, seulement les deux premiers motifs.
        let mut f = pieuvre();
        f.passer_en_facile();
        assert_eq!((f.pv(), f.pv_max(), f.motif()), (2, 2, 0));
    }

    /// Part des colonnes frappées par une salve, en moyenne sur le motif (de 0 à 1).
    fn part_frappee(salves: &[Salve], nb_colonnes: usize, portee: i8) -> f32 {
        let frappees: usize = salves
            .iter()
            .map(|s| (-portee..=portee).filter(|&d| s.couvre(d)).count())
            .sum();
        frappees as f32 / (salves.len() * nb_colonnes) as f32
    }

    /// Carte 9.12c (AGA : « un peu moins dure que le grand nuage mais sur le même
    /// principe, qui accélère au fur et à mesure qu'elle perd de la vie et qui fait
    /// des attaques plus grosses ») : à chaque point perdu, la pieuvre frappe PLUS
    /// GROS (part du bassin frappée par salve) et PLUS VITE (écart entre salves).
    /// La difficulté elle-même se MESURE (`difficulte::tests::mesure_des_deux_boss`) :
    /// la part frappée du nuage est affichée pour comparaison, pas imposée. Le
    /// rythme : l'écart le PLUS COURT entre deux salves (carte 9.12d : le souffle
    /// entre l'aller et le retour de la vague du 3e motif n'est pas son rythme).
    #[test]
    fn chaque_point_perdu_frappe_plus_gros_et_plus_vite() {
        use super::super::nuage;
        let mut avant: Option<(f32, u16)> = None;
        for (m, salves) in MOTIFS.iter().enumerate() {
            let pieuvre = part_frappee(salves, NB_COLONNES, PORTEE_COLONNES);
            let grand_nuage =
                part_frappee(nuage::MOTIFS[m], nuage::NB_COLONNES, nuage::PORTEE_COLONNES);
            let ecart = salves
                .windows(2)
                .map(|p| p[1].debut - p[0].debut)
                .min()
                .unwrap();
            println!(
                "motif {} : pieuvre {:.0} % toutes les {ecart} pas, Grand Nuage {:.0} %",
                m + 1,
                100.0 * pieuvre,
                100.0 * grand_nuage
            );
            if let Some((part, ecart_avant)) = avant {
                assert!(pieuvre > part, "motif {} : pas plus gros", m + 1);
                assert!(ecart < ecart_avant, "motif {} : pas plus vite", m + 1);
            }
            avant = Some((pieuvre, ecart));
        }
    }

    /// Toujours une issue (le raisonnement du Grand Nuage, `nuage::tests::
    /// chaque_motif_se_traverse`, en plus strict). Calcul à rebours : `sur[q]` dit
    /// si, à la place `q` et au pas `t`, on peut tenir jusqu'à la fin du motif en se
    /// déplaçant d'au plus 1,5 px par pas (un peu moins que la course, 1,6 px par
    /// pas). Places au demi-pixel ; Pépin fait 10 px de large, chaque tentacule
    /// 24 px au milieu de sa colonne de 32. Deux garanties :
    /// - d'où qu'on parte au début du calme qui précède le motif (le plus court :
    ///   « touchée », 90 pas), on s'en sort ;
    /// - au DÉBUT de chaque frappe, de TOUTE place que cette frappe épargne, on
    ///   tient jusqu'à la fin du motif : la salve suivante a toujours une colonne
    ///   sûre qu'on atteint en courant (aucune impasse, salve après salve).
    ///
    /// Le modèle est celui du nuage (un sol plat où l'on court) : depuis la carte
    /// 9.12c, l'arène a justement un platelage où l'on court, et ses trous d'une
    /// case se sautent à la même vitesse (1,6 px par pas en l'air) ; pas en barque
    /// (lente). La preuve sur la vraie physique est le
    /// robot de QA (`le_robot_bat_la_pieuvre_sans_mourir`), et la mesure de
    /// difficulté (tous les profils finissent).
    #[test]
    fn chaque_motif_se_traverse() {
        const DEMI: usize = 2;
        let pepin = 10 * DEMI;
        let largeur = LARGEUR_COLONNE as usize;
        let marge = MARGE_TENTACULE as usize;
        let corps = largeur - 2 * marge;
        let places = NB_COLONNES * largeur * DEMI - pepin + 1;
        let calme = TOUCHEE_PAS;
        for (motif, salves) in MOTIFS.iter().enumerate() {
            let debuts: Vec<u16> = salves
                .iter()
                .map(|s| calme + s.debut + ANNONCE_PAS)
                .collect();
            let mut sur = vec![true; places];
            for t in (0..calme + duree_motif(motif)).rev() {
                let precedent = sur.clone();
                for (q, s) in sur.iter_mut().enumerate() {
                    let (de, a) = (q.saturating_sub(3), (q + 3).min(places - 1));
                    *s = precedent[de..=a].iter().any(|&p| p);
                }
                let mut frappee = vec![false; places];
                if t >= calme {
                    for i in 0..NB_COLONNES {
                        let etat = etat_tentacule(motif, t - calme, i as i8 - PORTEE_COLONNES);
                        if etat != EtatTentacule::Dresse {
                            continue;
                        }
                        let gauche = (largeur * i + marge) * DEMI;
                        let droite = gauche + corps * DEMI;
                        // Pépin (de q à q + 10 px) chevauche le tentacule.
                        for (q, s) in sur.iter_mut().enumerate() {
                            if q < droite && q + pepin > gauche {
                                *s = false;
                                frappee[q] = true;
                            }
                        }
                    }
                }
                if debuts.contains(&t) {
                    if let Some(q) = (0..places).find(|&q| !frappee[q] && !sur[q]) {
                        panic!(
                            "motif {}, frappe au pas {} : impasse en x = {}",
                            motif + 1,
                            t - calme,
                            q as f32 / DEMI as f32
                        );
                    }
                }
            }
            if let Some(q) = sur.iter().position(|&s| !s) {
                panic!(
                    "motif {} : impossible de s'en sortir en partant de x = {}",
                    motif + 1,
                    q as f32 / DEMI as f32
                );
            }
        }
    }

    /// Carte 9.12d (AGA : « la pieuvre est trop rapide, c'est plus dur que l'orage
    /// pour un humain ») : le robot réagit à l'image près, un humain non. Aucune
    /// salve n'oblige à partir moins de 0,6 s (36 pas) après le début de son
    /// annonce. Pour chaque salve, Pépin est au milieu d'une colonne que la salve
    /// d'avant épargnait (n'importe laquelle pour la première) ; si celle-ci la
    /// frappe, il court (1,6 px par pas) vers une colonne qu'elle épargne : il part
    /// au plus tôt au début de l'annonce, ou quand le dernier tentacule dressé sur
    /// son chemin redescend ; il doit être sorti du tentacule (le bord de sa boîte
    /// passe celui du corps, 15 px de moins que la distance des milieux) quand la
    /// colonne tue. Le temps entre le départ possible et le dernier moment pour
    /// partir est sa marge : au moins 36 pas, vers la meilleure colonne.
    #[test]
    fn un_humain_a_le_temps_de_changer_de_colonne() {
        const REACTION_PAS: f32 = 36.0;
        const COURSE: f32 = 1.6;
        let colonnes = || -PORTEE_COLONNES..=PORTEE_COLONNES;
        for (motif, salves) in MOTIFS.iter().enumerate() {
            for (j, salve) in salves.iter().enumerate() {
                let tue = salve.debut + ANNONCE_PAS;
                let departs = colonnes().filter(|&c| j == 0 || !salves[j - 1].couvre(c));
                for c in departs.filter(|&c| salve.couvre(c)) {
                    let marge = colonnes()
                        .filter(|&d| !salve.couvre(d))
                        .map(|d| {
                            // Le chemin, de la colonne voisine de `c` jusqu'à `d` : un
                            // tentacule d'une autre salve dressé dessus pendant
                            // l'annonce retarde le départ.
                            let (de, a) = if d > c { (c + 1, d) } else { (d, c - 1) };
                            let depart = salves
                                .iter()
                                .filter(|k| (de..=a).any(|e| k.couvre(e)))
                                .map(|k| {
                                    (k.debut + ANNONCE_PAS, k.debut + ANNONCE_PAS + DRESSE_PAS)
                                })
                                .filter(|&(dresse, fin)| dresse < tue && fin > salve.debut)
                                .map(|(_, fin)| fin)
                                .fold(salve.debut, u16::max);
                            let trajet =
                                (LARGEUR_COLONNE * f32::from((d - c).abs()) - 15.0) / COURSE;
                            f32::from(tue) - trajet - f32::from(depart)
                        })
                        .fold(f32::MIN, f32::max);
                    assert!(
                        marge >= REACTION_PAS,
                        "motif {}, salve {} (pas {}) : depuis la colonne {c}, {marge:.0} pas \
                         pour réagir",
                        motif + 1,
                        j + 1,
                        salve.debut
                    );
                }
            }
        }
    }

    // --- La saisie des plateformes (carte 9.12f) ---

    /// Une pieuvre et deux plateformes de 2 cases (rangée 6) ; Pépin debout sur
    /// la première.
    fn pieuvre_et_plateformes() -> Pieuvre {
        let mut p = pieuvre();
        p.plateformes = vec![vec![(2, 6), (3, 6)], vec![(12, 6), (13, 6)]];
        p.voir_pepin(Aabb::new(36.0, 82.0, 10.0, 14.0));
        p
    }

    /// L'annonce dure au moins 1,2 s ; jamais deux saisies à la fois ; et la
    /// plateforme redevient solide (sur 1 min de combat, Pépin sur une plateforme).
    #[test]
    fn une_saisie_s_annonce_coule_puis_remonte_une_a_la_fois() {
        let mut p = pieuvre_et_plateformes();
        let (mut saisies, mut remontees) = (0, 0);
        let mut avant: Option<Saisie> = None;
        for _ in 0..3600 {
            pas(&mut p, 1);
            let s = p.saisie();
            // Une seule saisie : les cases englouties sont celles d'UNE plateforme.
            let trous = p.cases_englouties();
            assert!(trous.is_empty() || p.plateformes.contains(&trous));
            match (avant, s) {
                (None, Some(s)) => {
                    saisies += 1;
                    assert!(s.porte());
                }
                // Elle cesse de porter après au moins 1,2 s (72 pas) d'annonce.
                (Some(a), Some(b)) if a.porte() && !b.porte() => {
                    assert!(b.ecoule >= 72, "{} pas d'annonce", b.ecoule);
                }
                // Elle finit remontée, ou lâchée encore annoncée (la tête sort).
                (Some(a), None) => match a.etat() {
                    EtatSaisie::Remontee(_) => remontees += 1,
                    EtatSaisie::Annonce(_) => assert_ne!(p.phase(), PhasePieuvre::Colere),
                    e => panic!("finie en {e:?}"),
                },
                _ => {}
            }
            avant = s;
        }
        assert!(saisies >= 3, "{saisies} saisies en 1 min");
        assert!(remontees >= 2, "{remontees} remontées");
    }

    /// La plateforme ne remonte jamais dans Pépin : s'il est dans son volume (en
    /// barque juste dessous, par exemple), elle attend qu'il en sorte.
    #[test]
    fn la_plateforme_ne_remonte_pas_dans_pepin() {
        let mut p = pieuvre_et_plateformes();
        while p.saisie().is_none() {
            pas(&mut p, 1);
        }
        let dans = Aabb::new(36.0, 98.0, 10.0, 14.0);
        p.voir_pepin(dans);
        pas(&mut p, u32::from(SAISIE_PAS) + 60);
        assert_eq!(p.cases_englouties(), &[(2, 6), (3, 6)], "elle attend");
        p.voir_pepin(Aabb::new(100.0, 82.0, 10.0, 14.0));
        pas(&mut p, 1);
        assert!(
            p.cases_englouties().is_empty(),
            "Pépin sorti : elle remonte"
        );
    }
}
