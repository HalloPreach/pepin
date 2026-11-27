//! Le PILOTE de la phase 2 de la Pieuvre (carte 9.13g, refait pour la montée
//! de la carte 9.36a) : un joueur automatique qui grimpe la tour du port, du bas
//! de la tour jusqu'au sommet (la fuite de la Pieuvre), dans la VRAIE
//! simulation (`Monde::avancer`, sans raccourci : aucune règle, aucun timing,
//! aucune case de la grille ne change pour lui).
//!
//! Pourquoi aucun robot existant ne le fait : `core::faisabilite` et le robot de
//! QA (`parcours.rs`) cherchent un plus court chemin entre des CASES ; la montée
//! a une horloge (la mer) et un tirage (les objets) qui ne sont pas dans la
//! case : les états se confondent ou se multiplient sans fin.
//!
//! Le pilote sépare donc l'ESPACE et le TEMPS :
//! 1. L'espace, une seule fois (`Route`, mise en cache dans un `OnceLock`) : la
//!    tour SANS la mer ni les objets (`Pieuvre::pour_le_robot`), explorée par les
//!    plans du robot de QA (`plans`, `simuler`) depuis la lanterne du bas ;
//!    chaque case posée connaît ses sauts possibles et son coût jusqu'au sommet
//!    (Dijkstra à l'envers, sur la durée). Le temps n'y est pas : quelques
//!    centaines de cases au lieu de millions.
//! 2. Le temps, à chaque décision, dans la vraie partie : il lit l'état PUBLIC
//!    de la montée (les objets annoncés par leur ombre, ceux qui tombent, la mer)
//!    et choisit comme un joueur : le saut de sa route qui le rapproche du
//!    sommet et ne se pose pas sous un objet annoncé ; sinon attendre sur place ;
//!    sinon s'écarter (n'importe quel geste qui survit).
//!
//! Comme le joueur simulé de la mesure, il ESSAIE chaque geste sur une copie du
//! monde avant de le jouer (`simuler`) : il voit donc un objet lancé PENDANT le
//! geste (au plus la durée du geste, environ 1 s ; un humain le voit à son
//! ombre, 1 s avant). C'est sa seule avance sur un humain ; le tirage des
//! objets, lui, il ne le lit pas.
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
use pepin_core::entities::pieuvre::{EtatChute, PhasePieuvre, Pieuvre, TAILLE_OBJET};
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
/// bâche et le sommet) n'existe que dans ce cas.
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
/// phase 2 le ramène). La graine des objets est fixée (`fixer_graine`) ; `None` :
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
/// qui en partent (plan, case d'arrivée, durée) et son coût (pas) jusqu'au
/// sommet.
pub(crate) struct Route {
    gestes: HashMap<Case, Vec<(Plan, Case, u16)>>,
    cout: HashMap<Case, u32>,
    /// Le bas du sommet (px) : les pieds de Pépin y sont à la fin.
    sommet: f32,
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
        // Sans sommet pour la Pieuvre du robot (sinon y entrer serait un coup sur
        // la tête, et Pépin y rebondirait) : on le garde pour les buts.
        let boite_sommet = p.sommet.take().expect("la tour a son sommet");
        let sommet = boite_sommet.bas();
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
            if (m.joueur.boite.bas() - sommet).abs() < 1.0
                && m.joueur.boite.chevauche(&boite_sommet)
            {
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
        // Dijkstra à l'envers : coût (pas) de chaque case jusqu'au sommet.
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
            sommet,
        }
    }

    /// Nombre de cases connues, et de cases qui mènent au sommet.
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

/// Ce que la montée a ANNONCÉ (état public) : la boîte `b` (Pépin posé à
/// l'arrivée d'un geste) est-elle sous un objet annoncé par son ombre, ou qui
/// tombe encore au-dessus d'elle ?
fn zone_annoncee(p: &Pieuvre, b: &Aabb) -> bool {
    let t = p.ecoule();
    p.chutes().iter().any(|c| {
        let colonne = b.x < c.x + TAILLE_OBJET && b.droite() > c.x;
        let a_venir = match c.etat(t) {
            Some(EtatChute::Annonce(_)) => true,
            Some(EtatChute::Chute(o)) => o.y < b.bas(),
            _ => false,
        };
        colonne && a_venir && b.bas() <= c.sol + 1.0 && b.bas() > c.depart
    })
}

/// Ce qui s'est passé pendant une partie (le tableau de QA.md).
#[derive(Debug, Clone, Default)]
pub(crate) struct Bilan {
    pub(crate) victoire: bool,
    pub(crate) morts: u32,
    /// Pas joués (60 par seconde), du bas de la tour à la fin.
    pub(crate) pas: u32,
    /// Objets lancés pendant la partie (toutes tentatives).
    pub(crate) objets: u32,
    /// Le plus petit écart vu entre la mer et les pieds de Pépin (px).
    pub(crate) mer_au_plus_pres: f32,
    /// Où Pépin est mort (les pieds, en rangées) et la cause (la mer ou un
    /// objet).
    pub(crate) lieux_des_morts: Vec<(i32, String)>,
}

/// Compte les objets lancés d'un pas à l'autre (le compteur de la tentative
/// repart à 0 à chaque reprise).
#[derive(Default)]
struct Observateur {
    lances: u32,
    tentative: u32,
}

impl Observateur {
    fn regarder(&mut self, m: &Monde, bilan: &mut Bilan) {
        let p = pieuvre(m);
        if p.phase() != PhasePieuvre::Montee {
            return;
        }
        if p.tentative() != self.tentative {
            self.tentative = p.tentative();
            self.lances = 0;
        }
        let n = p.objets_lances();
        bilan.objets += n.saturating_sub(self.lances);
        self.lances = n;
        if let Some(mer) = p.surface_mer() {
            if m.reapparition == 0 {
                let ecart = mer - m.joueur.boite.bas();
                if bilan.mer_au_plus_pres == 0.0 || ecart < bilan.mer_au_plus_pres {
                    bilan.mer_au_plus_pres = ecart;
                }
            }
        }
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
        let ici = case(m);
        self.bloque = if self.bloque.0 == ici {
            (ici, self.bloque.1 + 1)
        } else {
            (ici, 0)
        };
        let bloque = self.bloque.1;
        let cout_ici = self.route.cout(ici);
        // 1. Un geste de sa route qui le rapproche du sommet, sans se poser
        //    sous un objet annoncé, et sûr encore un instant après.
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
        // 1 bis. Aucun geste de la route ne marche (un nuage friable tombé :
        //    la grille n'est plus celle de la route ; un objet annoncé) : tous
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
        // 2. Attendre sur place, si c'est sûr.
        let attente = plan_attente(ATTENTE);
        if let Some((_, _, true)) = essayer(m, attente) {
            return attente;
        }
        // 3. S'écarter : n'importe quel geste qui survit ; d'abord sûr ensuite,
        //    puis hors des zones annoncées, puis le plus près du sommet.
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
/// joué pas à pas dans la vraie partie. Gagnée dès que la Pieuvre s'enfuit.
pub(crate) fn jouer(
    mut m: Monde,
    executer: &mut dyn FnMut(Plan) -> Plan,
    mut avant_chaque_pas: impl FnMut(&Monde, &Entrees) -> bool,
) -> Bilan {
    let mut pilote = Pilote::new();
    let mut bilan = Bilan::default();
    let mut obs = Observateur::default();
    let depart = m.pas;
    let morts_avant = m.etat.morts;
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
            let mer = pieuvre(&m).surface_mer();
            m.avancer(&Entrees::default());
            obs.regarder(&m, &mut bilan);
            // Une mort pendant l'attente (la mer, un objet sur la lanterne...).
            if let Some(cause) = m.derniere_mort.take() {
                bilan
                    .lieux_des_morts
                    .push((rangee(&avant), format!("{cause:?}, {}", quoi(mer, &avant))));
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
            m.avancer(e);
            obs.regarder(&m, &mut bilan);
            if let Some(cause) = m.derniere_mort.take() {
                bilan
                    .lieux_des_morts
                    .push((rangee(&avant), format!("{cause:?}, {}", quoi(mer, &avant))));
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

/// La rangée des pieds de `b`.
fn rangee(b: &Aabb) -> i32 {
    (b.bas() / 16.0).round() as i32
}

/// Ce qui a tué Pépin, pieds en `b` juste avant : la mer (`mer`) si ses pieds
/// y étaient presque, sinon un objet.
fn quoi(mer: Option<f32>, b: &Aabb) -> &'static str {
    if mer.is_some_and(|y| b.bas() + 4.0 > y) {
        "mer"
    } else {
        "objet"
    }
}

/// Le dessin du tableau de QA.md : une ligne par partie.
pub(crate) fn ligne(mode: &str, graine: u32, b: &Bilan) -> String {
    let morts: Vec<String> = b
        .lieux_des_morts
        .iter()
        .map(|(r, c)| format!("rangée {r} ({c})"))
        .collect();
    format!(
        "| {mode} | {graine} | {} | {} | {:.1} s | {} | {:.1} | {} |",
        if b.victoire { "OUI" } else { "NON" },
        b.morts,
        b.pas as f32 / 60.0,
        b.objets,
        b.mer_au_plus_pres / 16.0,
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

    /// Les graines de la preuve (carte 9.36a : 10 graines, en normal et en
    /// facile).
    const GRAINES: [u32; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    /// La route de la tour mène du bas de la tour au sommet.
    #[test]
    #[ignore = "pilote de la Pieuvre (cartes 9.13g et 9.36a) : se lance à la main"]
    fn la_route_mene_au_sommet() {
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
            "route : {cases} cases, {menent} mènent au sommet (bas {}), départ {depart:?} coût {} ; {:.1} s",
            route.sommet,
            route.cout(depart),
            debut.elapsed().as_secs_f32()
        );
        assert!(
            route.cout(depart) < HORS_ROUTE,
            "le bas de la tour ne mène pas au sommet"
        );
    }

    /// LA PREUVE (cartes 9.13g et 9.36a) : la phase 2 complète, du bas de la
    /// tour à la fuite de la Pieuvre, pour 10 graines en normal et 10 en facile.
    /// Lancer avec :
    /// `timeout 600 ./docker-cargo.sh test -p game --no-default-features --profile tests-rapides le_pilote_finit_la_phase_2 -- --ignored --nocapture`
    #[test]
    #[ignore = "pilote de la Pieuvre (cartes 9.13g et 9.36a) : se lance à la main"]
    fn le_pilote_finit_la_phase_2() {
        let debut = std::time::Instant::now();
        // Les 20 parties sur les fils libres (`PEPIN_FILS`), dans l'ordre.
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
        println!("| mode | graine | victoire | morts | durée | objets lancés | mer au plus près (tuiles) | morts (lieu, cause) |");
        for l in &lignes {
            println!("{l}");
        }
        println!(
            "{victoires}/{} victoires ({:.0} s)",
            parties.len(),
            debut.elapsed().as_secs_f32()
        );
        assert_eq!(
            victoires,
            parties.len(),
            "le pilote ne finit pas toutes les parties"
        );
    }
}
