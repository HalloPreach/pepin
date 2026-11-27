//! Le PILOTE de la phase 2 de la Pieuvre (carte 9.13g) : un joueur automatique
//! qui grimpe la tour du port, du bas de la tour jusqu'à `Vaincue`, dans la
//! VRAIE simulation (`Monde::avancer`, sans raccourci : aucune règle, aucun
//! timing, aucune case de la grille ne change pour lui).
//!
//! Pourquoi aucun robot existant ne le faisait (diagnostic de la carte) :
//! - `core::faisabilite` et le robot de QA (`parcours.rs`) cherchent un plus
//!   court chemin entre des CASES ; une mécanique qui a une horloge met son temps
//!   dans la case (`nuage_de_la_case` : phase, 63 tranches, têtes). La montée
//!   dure plus de 60 s, sa mer, son humeur, son attaque en cours et sa graine ne
//!   sont pas dans la case : les états se confondent ou se multiplient, et la
//!   recherche dépasse 600 s sans parcours ;
//! - la mesure de difficulté (`difficulte.rs`) part des mêmes itinéraires.
//!
//! Le pilote sépare donc l'ESPACE et le TEMPS :
//! 1. L'espace, une seule fois (`Route`, mise en cache dans un `OnceLock`) : la
//!    tour SANS la géante (`Pieuvre::pour_le_robot` : aucune attaque, la mer en
//!    bas), explorée par les plans du robot de QA (`plans`, `simuler`) depuis la
//!    lanterne du bas ; chaque case posée connaît ses sauts possibles et son coût
//!    jusqu'à la vigie (Dijkstra à l'envers, sur la durée). Le temps n'y est
//!    pas : quelques centaines de cases au lieu de millions.
//! 2. Le temps, à chaque décision, dans la vraie partie : il lit l'état PUBLIC
//!    de la géante (frappe visée annoncée, balayeur, caisses, arrachage, jeté,
//!    tête sonnée, mer) et choisit comme un joueur : frapper la tête sonnée si
//!    un saut l'atteint ; sinon le saut de sa route qui le rapproche de la
//!    vigie et ne se pose pas dans une zone annoncée ; sinon attendre sur place ;
//!    sinon s'écarter (n'importe quel geste qui survit).
//!
//! Comme le joueur simulé de la mesure, il ESSAIE chaque geste sur une copie du
//! monde avant de le jouer (`simuler`) : il voit donc une attaque que la géante
//! choisirait PENDANT le geste (au plus la durée du geste, environ 1 s ; un
//! humain la voit à l'annonce, 0,8 s au moins). C'est sa seule avance sur un
//! humain ; la graine de la géante, elle, il ne la lit pas.
//!
//! Pourquoi ici (`bot_qa`, module de test) : il réutilise les plans et la
//! simulation du robot de QA (`parcours.rs`) et la mesure (`difficulte.rs`,
//! elle aussi de test) s'en sert ; rien de tout cela n'entre dans le jeu publié.
//!
//! Notions Rust : `OnceLock` (un calcul partagé fait une seule fois, même entre
//! plusieurs tests), une fermeture passée en paramètre (`&mut dyn FnMut`), un
//! `BinaryHeap` de `Reverse` pour un Dijkstra.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::OnceLock;

use pepin_core::aabb::Aabb;
use pepin_core::entities::pieuvre::{
    Attaque, EtatArrachage, EtatBalai, EtatFrappe, EtatJete, EtatSecousse, PhasePieuvre, Pieuvre,
};
use pepin_core::entities::Entite;
use pepin_core::player::{Direction, Entrees, Joueur};

use super::{est_mort, plans, simuler, Fin, Plan};
use crate::monde::Monde;

/// Une case du pilote : Pépin posé, `x` en huitièmes de tuile (8 px), `y` au
/// pixel (le haut de sa boîte). Comme `parcours::cle`, sans le temps.
type Case = (i32, i32);

/// Plus de cases que cela dans la route : la tour n'est pas finie (garde-fou).
const CASES_MAX: usize = 6_000;
/// Attente quand il n'y a rien de mieux : 1/5 s, puis il regarde de nouveau.
const ATTENTE: u16 = 12;
/// Une partie plus longue que cela (pas) est abandonnée : 6 min de jeu.
const PAS_MAX_PARTIE: u32 = 6 * 60 * 60;
/// Coût d'une case hors de la route, au-dessus de tout coût connu.
const HORS_ROUTE: u32 = 1_000_000;

fn case(m: &Monde) -> Case {
    (
        (m.joueur.boite.x / 8.0).floor() as i32,
        m.joueur.boite.y.round() as i32,
    )
}

fn plan_attente(pas: u16) -> Plan {
    Plan {
        dir: Direction::Immobile,
        elan: 0,
        saut: 0,
        planer: false,
        retard_plane: 0,
        dir_montee: false,
        dir_descente: false,
        attente: pas,
    }
}

/// L'indice du niveau du boss dans la table.
pub(crate) fn indice_pieuvre() -> usize {
    niveaux::niveaux()
        .iter()
        .position(|n| n.cle == "pieuvre")
        .expect("le niveau « pieuvre » est dans la table")
}

pub(crate) fn pieuvre(m: &Monde) -> &Pieuvre {
    m.entites
        .iter()
        .find_map(|e| match e {
            Entite::Pieuvre(p) => Some(&**p),
            _ => None,
        })
        .expect("le niveau a sa pieuvre")
}

/// Le niveau a-t-il une Pieuvre À TOUR, donc une phase 2 (carte 10.7 : le robot de
/// QA de partie passe alors la main au pilote après la phase 1) ? Sa trombe (la
/// bâche et la vigie) n'existe que dans ce cas.
pub(crate) fn a_une_phase_2(m: &Monde) -> bool {
    m.entites
        .iter()
        .any(|e| matches!(e, Entite::Pieuvre(p) if p.zone_trombe().is_some()))
}

/// La phase 2 est-elle commencée et pas encore gagnée (furie ou montée) ? C'est
/// là que le pilote prend la main.
pub(crate) fn en_phase_2(m: &Monde) -> bool {
    m.entites.iter().any(|e| {
        matches!(e, Entite::Pieuvre(p)
            if p.en_fuite() && matches!(p.phase(), PhasePieuvre::Furie | PhasePieuvre::Montee))
    })
}

fn pieuvre_mut(m: &mut Monde) -> &mut Pieuvre {
    m.entites
        .iter_mut()
        .find_map(|e| match e {
            Entite::Pieuvre(p) => Some(&mut **p),
            _ => None,
        })
        .expect("le niveau a sa pieuvre")
}

/// Le DÉBUT DE LA PHASE 2, comme après le dernier coup de la phase 1 : la phase 1
/// est sautée par les deux appels publics qu'utilisent déjà le robot de
/// faisabilité et les tests (`sortir_la_tete`, `toucher`), autant de fois que la
/// pieuvre a de points (3, ou 2 en facile : `passer_nuage_en_facile`, comme le
/// jeu). Pépin est posé à la lanterne du bas de la tour (celle où une mort de la
/// phase 2 le ramène). La graine du cerveau est fixée (`fixer_graine`) ; `None` :
/// celle du jeu, comme pour un joueur.
pub(crate) fn debut_phase_2(graine: Option<u32>, facile: bool) -> Monde {
    let mut m = Monde::new(indice_pieuvre());
    if facile {
        m.passer_nuage_en_facile();
    }
    let p = pieuvre_mut(&mut m);
    for _ in 0..p.pv_max() {
        p.sortir_la_tete(0);
        p.toucher();
    }
    if let Some(g) = graine {
        p.fixer_graine(g);
    }
    let bache = p.bache.expect("la pieuvre du port a sa bâche");
    let (x, y) = m
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Lanterne(l) if l.boite.y < bache.y => Some(l),
            _ => None,
        })
        .max_by(|a, b| a.boite.y.total_cmp(&b.boite.y))
        .map(|l| l.point_de_reapparition())
        .expect("la tour a une lanterne");
    m.joueur = Joueur::new(x, y);
    m
}

/// L'ESPACE de la tour, calculé une fois : pour chaque case posée, les gestes
/// qui en partent (plan, case d'arrivée, durée) et son coût (pas) jusqu'à la
/// vigie.
pub(crate) struct Route {
    gestes: HashMap<Case, Vec<(Plan, Case, u16)>>,
    cout: HashMap<Case, u32>,
    /// Le dessus de la vigie (px) : les pieds de Pépin y sont à la fin.
    vigie: f32,
}

impl Route {
    /// La route, partagée par tous les tests du processus.
    pub(crate) fn de_la_tour() -> &'static Route {
        static ROUTE: OnceLock<Route> = OnceLock::new();
        ROUTE.get_or_init(Route::calculer)
    }

    fn calculer() -> Route {
        let mut m = debut_phase_2(Some(1), false);
        let p = pieuvre_mut(&mut m);
        p.pour_le_robot();
        let vigie = p.tete_finale.expect("la tour a sa vigie").bas();
        for _ in 0..40 {
            m.avancer(&Entrees::default());
        }
        let tous = plans(false);
        let mut gestes: HashMap<Case, Vec<(Plan, Case, u16)>> = HashMap::new();
        let mut vus: HashSet<Case> = HashSet::new();
        let mut file = vec![m];
        vus.insert(case(&file[0]));
        let mut buts: HashSet<Case> = HashSet::new();
        while let Some(m) = file.pop() {
            let ici = case(&m);
            if (m.joueur.boite.bas() - vigie).abs() < 1.0 {
                buts.insert(ici);
            }
            if vus.len() > CASES_MAX {
                break;
            }
            let mut sortants = Vec::new();
            for &plan in &tous {
                let (fin, suite, script) = simuler(&m, plan);
                if fin != Fin::Pose || est_mort(&suite) {
                    continue;
                }
                // La tour seulement : jamais sous la corniche du bas.
                if suite.joueur.boite.bas() > m.joueur.boite.bas() + 80.0 {
                    continue;
                }
                let la = case(&suite);
                if la == ici {
                    continue;
                }
                sortants.push((plan, la, script.len() as u16));
                if vus.insert(la) {
                    file.push(suite);
                }
            }
            gestes.insert(ici, sortants);
        }
        // Dijkstra à l'envers : coût (pas) de chaque case jusqu'à la vigie.
        let mut entrants: HashMap<Case, Vec<(Case, u16)>> = HashMap::new();
        for (&de, sortants) in &gestes {
            for &(_, vers, duree) in sortants {
                entrants.entry(vers).or_default().push((de, duree));
            }
        }
        let mut cout: HashMap<Case, u32> = HashMap::new();
        let mut tas: BinaryHeap<Reverse<(u32, Case)>> = BinaryHeap::new();
        for &b in &buts {
            cout.insert(b, 0);
            tas.push(Reverse((0, b)));
        }
        while let Some(Reverse((c, ici))) = tas.pop() {
            if cout.get(&ici).is_some_and(|&k| k < c) {
                continue;
            }
            for &(de, duree) in entrants.get(&ici).map_or(&[][..], |v| v.as_slice()) {
                let n = c + u32::from(duree);
                if cout.get(&de).is_none_or(|&k| n < k) {
                    cout.insert(de, n);
                    tas.push(Reverse((n, de)));
                }
            }
        }
        Route {
            gestes,
            cout,
            vigie,
        }
    }

    /// Nombre de cases connues, et de cases qui mènent à la vigie.
    pub(crate) fn taille(&self) -> (usize, usize) {
        (self.gestes.len(), self.cout.len())
    }

    /// Le coût d'une case ; hors de la route, celui de la case connue la plus
    /// proche à la même hauteur (une arrivée un peu décalée), sinon d'autant
    /// plus grand qu'elle est basse.
    fn cout(&self, c: Case) -> u32 {
        if let Some(&k) = self.cout.get(&c) {
            return k;
        }
        let voisin = (1..=4)
            .flat_map(|dx| [(c.0 - dx, dx), (c.0 + dx, dx)])
            .flat_map(|(x, dx)| (-2..=2).map(move |dy| ((x, c.1 + dy), dx)))
            .filter_map(|(v, dx)| self.cout.get(&v).map(|&k| k + 4 * dx as u32))
            .min();
        voisin.unwrap_or(HORS_ROUTE + c.1.max(0) as u32)
    }

    /// Les gestes de la case `c` (ou de la plus proche à la même hauteur).
    fn gestes(&self, c: Case) -> &[(Plan, Case, u16)] {
        if let Some(g) = self.gestes.get(&c) {
            return g;
        }
        (1..=4)
            .flat_map(|dx| [c.0 - dx, c.0 + dx])
            .flat_map(|x| (-2..=2).map(move |dy| (x, c.1 + dy)))
            .find_map(|v| self.gestes.get(&v))
            .map_or(&[], |g| g.as_slice())
    }
}

/// Ce que la géante a ANNONCÉ (état public) : la boîte `b` (Pépin posé à
/// l'arrivée d'un geste) est-elle dans une zone qui va devenir mortelle ? Frappe
/// visée annoncée, colonne d'une caisse annoncée, corniche d'un balayeur
/// annoncé, plateforme qui va être arrachée, place d'un jeté qui se cabre.
fn zone_annoncee(p: &Pieuvre, b: &Aabb) -> bool {
    let frappe = matches!(p.etat_frappe(), Some((c, EtatFrappe::Annonce(_))) if c.chevauche(b));
    let caisse = match p.etat_secousse() {
        Some((caisses, EtatSecousse::Annonce(_))) => caisses
            .iter()
            .any(|c| b.x < c.x + 16.0 && b.droite() > c.x && b.bas() <= c.sol + 1.0),
        _ => false,
    };
    let balai = p.tour.balayeurs.iter().enumerate().any(|(k, bal)| {
        matches!(p.etat_balayeur(k), Some(EtatBalai::Annonce(_)))
            && (b.bas() - bal.y).abs() < 2.0
            && b.droite() > bal.arrivee
            && b.x < bal.depart
    });
    let arrachage = p.tour.arrachables.iter().enumerate().any(|(k, cases)| {
        matches!(p.etat_arrachage(k), Some(EtatArrachage::Annonce(_)))
            && cases.iter().any(|&(tx, ty)| {
                (b.bas() - ty as f32 * 16.0).abs() < 2.0
                    && b.droite() > tx as f32 * 16.0
                    && b.x < (tx + 1) as f32 * 16.0
            })
    });
    let jete = matches!(p.etat_jete(), Some((t, EtatJete::Cabre(_))) if t.chevauche(b));
    frappe || caisse || balai || arrachage || jete
}

/// Pépin est-il debout sur la vigie (ses pieds au niveau de son dessus) ?
pub(crate) fn sur_la_vigie(m: &Monde) -> bool {
    let v = pieuvre(m).tete_finale.map_or(f32::MAX, |v| v.bas());
    (m.joueur.boite.bas() - v).abs() < 2.0
}

/// Le nom de l'attaque en cours (pour dire ce qui a tué Pépin).
fn nom_attaque(p: &Pieuvre) -> &'static str {
    let combo = matches!(p.etat_frappe(), Some((_, EtatFrappe::Frappe)))
        && !matches!(p.attaque(), Some(Attaque::Frappe { .. }));
    match p.attaque() {
        _ if combo => "frappe de combo",
        Some(Attaque::Frappe { .. }) => "frappe",
        Some(Attaque::Balayage { .. }) => "balayage",
        Some(Attaque::Secousse { .. }) => "secousse",
        Some(Attaque::Arrachage { .. }) => "arrachage",
        Some(Attaque::Jete { .. }) => "jeté",
        None => "aucune attaque",
    }
}

/// Ce qui s'est passé pendant une partie (le tableau de QA.md).
#[derive(Debug, Clone, Default)]
pub(crate) struct Bilan {
    pub(crate) victoire: bool,
    pub(crate) morts: u32,
    /// Pas joués (60 par seconde), du bas de la tour à la fin.
    pub(crate) pas: u32,
    /// Attaques vues, par genre : frappe (seule), frappe de combo, balayage,
    /// secousse, arrachage, jeté (hors vigie), étape d'un cycle de la vigie.
    pub(crate) attaques: [u32; 7],
    /// Les cycles de vigie joués jusqu'à leur jeté vif : têtes qui lui
    /// restaient à chacun.
    pub(crate) cycles: Vec<u8>,
    /// Têtes prises pendant la montée (jetés) et sur la vigie.
    pub(crate) coups_montee: u32,
    pub(crate) coups_vigie: u32,
    /// Où Pépin est mort (les pieds, en rangées), la cause et l'attaque en
    /// cours (ou la mer).
    pub(crate) lieux_des_morts: Vec<(i32, String)>,
}

/// Les noms des colonnes de `Bilan::attaques`.
pub(crate) const NOMS_ATTAQUES: [&str; 7] = [
    "frappe",
    "combo",
    "balayage",
    "secousse",
    "arrachage",
    "jete",
    "vigie",
];

/// Suit les attaques publiques d'un pas à l'autre (une nouvelle attaque : un
/// nouveau début).
#[derive(Default)]
struct Observateur {
    derniere: Option<(u32, u16)>,
    /// Une frappe de combo était-elle annoncée au pas d'avant ?
    combo: bool,
    /// Les têtes qu'il lui restait au pas d'avant.
    pv: u8,
    /// L'attaque du pas d'avant était-elle le jeté de la vigie ?
    jete_vigie: bool,
}

impl Observateur {
    fn regarder(&mut self, m: &Monde, bilan: &mut Bilan) {
        let p = pieuvre(m);
        // Une tête prise (même la dernière, qui la fait plonger) : sur la
        // vigie si le jeté du pas d'avant était celui de la vigie.
        if p.pv() < self.pv {
            if self.jete_vigie {
                bilan.coups_vigie += 1;
            } else {
                bilan.coups_montee += 1;
            }
        }
        self.pv = p.pv();
        self.jete_vigie =
            matches!(p.attaque(), Some(Attaque::Jete { k, .. }) if *k == p.tour.jetes.len());
        if p.phase() != PhasePieuvre::Montee {
            return;
        }
        let t = p.tentative();
        let vigie = p.tour.jetes.len();
        if let Some(a) = p.attaque() {
            let cle = (t, a.debut());
            if self.derniere != Some(cle) {
                self.derniere = Some(cle);
                // Sur la vigie, une frappe ou un balayage est une étape de son
                // cycle : compté en « vigie ».
                let rang = match a {
                    Attaque::Frappe { .. } | Attaque::Balayage { .. } if sur_la_vigie(m) => 6,
                    Attaque::Frappe { .. } => 0,
                    Attaque::Balayage { .. } => 2,
                    Attaque::Secousse { .. } => 3,
                    Attaque::Arrachage { .. } => 4,
                    Attaque::Jete { k, .. } if *k < vigie => 5,
                    Attaque::Jete { .. } => 6,
                };
                bilan.attaques[rang] += 1;
            }
        }
        // Une frappe visée de combo (pas l'attaque en cours) : comptée au pas
        // où son annonce commence.
        let annonce = matches!(p.etat_frappe(), Some((_, EtatFrappe::Annonce(_))))
            && !matches!(p.attaque(), Some(Attaque::Frappe { .. }));
        if annonce && !self.combo {
            bilan.attaques[1] += 1;
        }
        self.combo = annonce;
    }
}

/// Le pilote : la route, et ce qu'il a déjà décidé.
pub(crate) struct Pilote {
    route: &'static Route,
    /// Les plans du robot de QA, grossiers et précis (calculés une fois).
    grossiers: Vec<Plan>,
    precis: Vec<Plan>,
    /// La case de la décision d'avant et le nombre de décisions de suite prises
    /// là : l'essai de tous les plans (2 bis) ne se fait que s'il y reste.
    bloque: (Case, u32),
}

impl Pilote {
    pub(crate) fn new() -> Self {
        Pilote {
            route: Route::de_la_tour(),
            grossiers: plans(false),
            precis: plans(true),
            bloque: ((i32::MIN, i32::MIN), 0),
        }
    }

    fn plans(&self, precis: bool) -> &[Plan] {
        if precis {
            &self.precis
        } else {
            &self.grossiers
        }
    }

    /// Le geste à jouer maintenant (Pépin posé, `m` la vraie partie).
    pub(crate) fn choisir(&mut self, m: &Monde) -> Plan {
        let p = pieuvre(m);
        let ici = case(m);
        self.bloque = if self.bloque.0 == ici {
            (ici, self.bloque.1 + 1)
        } else {
            (ici, 0)
        };
        let bloque = self.bloque.1;
        let cout_ici = self.route.cout(ici);
        let pv = p.pv();
        // 1. Une tête sonnée : un saut qui la prend.
        if matches!(p.etat_jete(), Some((_, EtatJete::Sonnee(_)))) {
            let mut meilleur: Option<(usize, Plan)> = None;
            for &plan in self.plans(true) {
                let (fin, suite, script) = simuler(m, plan);
                if fin == Fin::Mort || est_mort(&suite) {
                    continue;
                }
                let q = pieuvre(&suite);
                let pris = q.pv() < pv || q.phase() != PhasePieuvre::Montee;
                if pris && meilleur.is_none_or(|(d, _)| script.len() < d) {
                    meilleur = Some((script.len(), plan));
                }
            }
            if let Some((_, plan)) = meilleur {
                return plan;
            }
        }
        // 2. Un geste de sa route qui le rapproche de la vigie, sans se poser
        //    dans une zone annoncée, et sûr encore un instant après.
        let mut gestes: Vec<&(Plan, Case, u16)> = self
            .route
            .gestes(ici)
            .iter()
            .filter(|g| self.route.cout(g.1) < cout_ici)
            .collect();
        gestes.sort_by_key(|g| self.route.cout(g.1) + u32::from(g.2));
        for &(plan, _, _) in gestes.iter().take(16) {
            if let Some((suite, _, true)) = essayer(m, *plan) {
                if self.route.cout(case(&suite)) < cout_ici {
                    return *plan;
                }
            }
        }
        // 2 bis. Aucun geste de la route ne marche (une plateforme arrachée,
        //    un nuage tombé : la grille n'est plus celle de la route) : tous
        //    les plans, essayés dans la vraie partie (les grossiers, puis les
        //    précis s'il le faut : c'est le plus coûteux du pilote, donc
        //    seulement quand il reste sur la même case : les grossiers deux
        //    décisions sur six, les précis une sur six).
        let essais: &[bool] = match bloque % 6 {
            3 => &[false, true],
            1 | 5 => &[false],
            _ => &[],
        };
        for &precis in essais {
            let mut meilleur: Option<(u32, Plan)> = None;
            for &plan in self.plans(precis) {
                if let Some((suite, duree, true)) = essayer(m, plan) {
                    let c = self.route.cout(case(&suite));
                    let note = c + duree as u32;
                    if c < cout_ici && meilleur.is_none_or(|(n, _)| note < n) {
                        meilleur = Some((note, plan));
                    }
                }
            }
            if let Some((_, plan)) = meilleur {
                return plan;
            }
        }
        // 3. Attendre sur place, si c'est sûr.
        let attente = plan_attente(ATTENTE);
        if let Some((_, _, true)) = essayer(m, attente) {
            return attente;
        }
        // 4. S'écarter : n'importe quel geste qui survit ; d'abord sûr ensuite,
        //    puis hors des zones annoncées, puis le plus près de la vigie.
        let mut meilleur: Option<((bool, bool, u32), Plan)> = None;
        for &plan in self.plans(false) {
            let Some((suite, duree, sur)) = essayer(m, plan) else {
                continue;
            };
            let note = (
                !sur,
                zone_annoncee(pieuvre(&suite), &suite.joueur.boite),
                self.route.cout(case(&suite)) + duree as u32,
            );
            if meilleur.is_none_or(|(n, _)| note < n) {
                meilleur = Some((note, plan));
            }
        }
        meilleur.map_or(attente, |(_, plan)| plan)
    }
}

/// Essaie `plan` sur une copie de la partie : `None` s'il tue Pépin ; sinon le
/// monde d'après, la durée du geste, et s'il est SÛR ENSUITE (posé hors d'une
/// zone annoncée, et vivant après `ATTENTE` pas de plus sans bouger : un coup
/// d'avance, pour ne pas se poser là où l'on ne peut que mourir).
fn essayer(m: &Monde, plan: Plan) -> Option<(Monde, usize, bool)> {
    let (fin, suite, script) = simuler(m, plan);
    if fin == Fin::Mort || est_mort(&suite) {
        return None;
    }
    let pose = matches!(fin, Fin::Pose | Fin::Arrivee);
    let annonce = zone_annoncee(pieuvre(&suite), &suite.joueur.boite)
        || zone_annoncee(pieuvre(m), &suite.joueur.boite);
    let (fin2, apres, _) = simuler(&suite, plan_attente(ATTENTE));
    let sur = pose && !annonce && fin2 != Fin::Mort && !est_mort(&apres);
    Some((suite, script.len(), sur))
}

/// Une partie de la phase 2, du bas de la tour à la fin : chaque geste choisi
/// par le pilote est d'abord passé à `executer` (la mesure de difficulté y
/// glisse les erreurs de son profil ; le pilote seul le rend tel quel), puis
/// joué pas à pas dans la vraie partie.
pub(crate) fn jouer(
    mut m: Monde,
    executer: &mut dyn FnMut(Plan) -> Plan,
    mut avant_chaque_pas: impl FnMut(&Monde, &Entrees) -> bool,
) -> Bilan {
    let mut pilote = Pilote::new();
    let mut bilan = Bilan::default();
    let mut obs = Observateur {
        pv: pieuvre(&m).pv(),
        ..Observateur::default()
    };
    let depart = m.pas;
    let morts_avant = m.etat.morts;
    let mut vif_vu: Option<(u32, u16)> = None;
    while m.pas - depart < PAS_MAX_PARTIE {
        let p = pieuvre(&m);
        if p.est_vaincue() || m.termine || p.phase() == PhasePieuvre::Plongee {
            bilan.victoire = true;
            break;
        }
        // Une réapparition en cours : on attend qu'elle finisse.
        if m.reapparition > 0 || !m.joueur.au_sol {
            if !avant_chaque_pas(&m, &Entrees::default()) {
                break;
            }
            let avant = m.joueur.boite;
            m.avancer(&Entrees::default());
            obs.regarder(&m, &mut bilan);
            // Une mort pendant l'attente (la mer, une caisse sur la lanterne...).
            if let Some(cause) = m.derniere_mort.take() {
                bilan.lieux_des_morts.push((
                    (avant.bas() / 16.0).round() as i32,
                    format!("{cause:?}, en attente"),
                ));
            }
            continue;
        }
        let plan = executer(pilote.choisir(&m));
        let (_, _, script) = simuler(&m, plan);
        let mut arret = false;
        for e in &script {
            if !avant_chaque_pas(&m, e) {
                arret = true;
                break;
            }
            let avant = m.joueur.boite;
            let mer = pieuvre(&m).surface_mer();
            let attaque = nom_attaque(pieuvre(&m));
            m.avancer(e);
            obs.regarder(&m, &mut bilan);
            // Chaque cycle de la vigie finit par un seul jeté VIF : on le
            // compte à son début, avec les têtes qui lui restent.
            let p = pieuvre(&m);
            if let Some(&Attaque::Jete {
                k,
                debut,
                vif: true,
                ..
            }) = p.attaque()
            {
                let cle = (p.tentative(), debut);
                if k == p.tour.jetes.len() && vif_vu != Some(cle) {
                    vif_vu = Some(cle);
                    bilan.cycles.push(p.pv());
                }
            }
            if let Some(cause) = m.derniere_mort.take() {
                let noye = mer.is_some_and(|y| avant.bas() + 4.0 > y);
                let quoi = if noye { "mer" } else { attaque };
                bilan.lieux_des_morts.push((
                    (avant.bas() / 16.0).round() as i32,
                    format!("{cause:?}, {quoi}"),
                ));
                break;
            }
        }
        if arret {
            break;
        }
    }
    let p = pieuvre(&m);
    bilan.victoire |= p.est_vaincue() || p.phase() == PhasePieuvre::Plongee;
    bilan.pas = m.pas - depart;
    // Le compteur du jeu : toutes les morts, même pendant une attente.
    bilan.morts = m.etat.morts - morts_avant;
    bilan
}

/// Le dessin du tableau de QA.md : une ligne par partie.
pub(crate) fn ligne(mode: &str, graine: u32, b: &Bilan) -> String {
    let attaques: Vec<String> = NOMS_ATTAQUES
        .iter()
        .zip(b.attaques)
        .map(|(n, k)| format!("{n} {k}"))
        .collect();
    let cycles: Vec<String> = b.cycles.iter().map(|t| format!("{t} t")).collect();
    let morts: Vec<String> = b
        .lieux_des_morts
        .iter()
        .map(|(r, c)| format!("rangée {r} ({c})"))
        .collect();
    format!(
        "| {mode} | {graine} | {} | {} | {}+{} | {:.1} s | {} | {} | {} |",
        if b.victoire { "OUI" } else { "NON" },
        b.morts,
        b.coups_montee,
        b.coups_vigie,
        b.pas as f32 / 60.0,
        attaques.join(", "),
        cycles.join(", "),
        if morts.is_empty() {
            "-".to_string()
        } else {
            morts.join(", ")
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les graines de la preuve (carte 9.13g).
    const GRAINES: [u32; 5] = [1, 2, 3, 4, 5];

    /// La route de la tour mène du bas de la tour à la vigie.
    #[test]
    #[ignore = "pilote de la Pieuvre (carte 9.13g) : se lance à la main"]
    fn la_route_mene_a_la_vigie() {
        println!(
            "plans : {} grossiers, {} précis",
            plans(false).len(),
            plans(true).len()
        );
        let debut = std::time::Instant::now();
        let route = Route::de_la_tour();
        let (cases, menent) = route.taille();
        let depart = case(&{
            let mut m = debut_phase_2(Some(1), false);
            for _ in 0..40 {
                m.avancer(&Entrees::default());
            }
            m
        });
        println!(
            "route : {cases} cases, {menent} mènent à la vigie (dessus {}), départ {depart:?} coût {} ; {:.1} s",
            route.vigie,
            route.cout(depart),
            debut.elapsed().as_secs_f32()
        );
        assert!(
            route.cout(depart) < HORS_ROUTE,
            "le bas de la tour ne mène pas à la vigie"
        );
    }

    /// Le script des entrées du pilote (graine 1, normal), du début du niveau
    /// jusqu'à un jeté de la vigie qui se cabre, Pépin sur la vigie : un
    /// caractère par pas (`code_entrees`), dans `target/qa/913g-pilote-vigie.txt`.
    /// Pour la capture `docs/qa/913g-pilote-vigie.png` (build jetable qui
    /// prépare la phase 2 comme `debut_phase_2` et rejoue ce script).
    #[test]
    #[ignore = "pilote de la Pieuvre (carte 9.13g) : se lance à la main"]
    fn ecrire_script_pilote_vigie() {
        let mut script = String::new();
        let mut fin = None;
        jouer(debut_phase_2(Some(1), false), &mut |p| p, |m, e| {
            let cabre = matches!(pieuvre(m).etat_jete(), Some((_, EtatJete::Cabre(k))) if k > 0.6);
            if sur_la_vigie(m) && cabre {
                fin = Some((m.pas, m.joueur.boite, pieuvre(m).pv()));
                return false;
            }
            script.push(super::super::code_entrees(e, false));
            true
        });
        let (pas, boite, pv) = fin.expect("le pilote arrive sur la vigie");
        std::fs::create_dir_all("../target/qa").expect("dossier target/qa");
        std::fs::write("../target/qa/913g-pilote-vigie.txt", &script).expect("écriture du script");
        println!(
            "script : {} pas (pas du monde {pas}), Pépin {boite:?}, {pv} têtes",
            script.len()
        );
        assert_eq!(script.len() as u32, pas);
    }

    /// LA PREUVE (carte 9.13g) : la phase 2 complète, du bas de la tour à la
    /// victoire, pour 5 graines en normal et 5 en facile. Lancer avec :
    /// `timeout 600 ./docker-cargo.sh test -p game --no-default-features --profile tests-rapides le_pilote_finit_la_phase_2 -- --ignored --nocapture`
    #[test]
    #[ignore = "pilote de la Pieuvre (carte 9.13g) : se lance à la main"]
    fn le_pilote_finit_la_phase_2() {
        let debut = std::time::Instant::now();
        // Les 10 parties sur les fils libres (`PEPIN_FILS`), dans l'ordre.
        let parties: Vec<(bool, u32)> = [false, true]
            .iter()
            .flat_map(|&f| GRAINES.iter().map(move |&g| (f, g)))
            .collect();
        let bilans = pepin_core::parallele::appliquer(&parties, |&(facile, g)| {
            jouer(debut_phase_2(Some(g), facile), &mut |p| p, |_, _| true)
        });
        let mut lignes = Vec::new();
        let mut victoires = 0;
        for (&(facile, g), b) in parties.iter().zip(&bilans) {
            let mode = if facile { "facile" } else { "normal" };
            victoires += usize::from(b.victoire);
            lignes.push(ligne(mode, g, b));
        }
        println!("| mode | graine | victoire | morts | têtes montée+vigie | durée | attaques vues | cycles de vigie (têtes) | morts (lieu, cause) |");
        for l in &lignes {
            println!("{l}");
        }
        println!(
            "{victoires}/10 victoires ({:.0} s)",
            debut.elapsed().as_secs_f32()
        );
        assert_eq!(victoires, 10, "le pilote ne finit pas toutes les parties");
    }
}
