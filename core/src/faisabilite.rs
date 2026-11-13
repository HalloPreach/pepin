//! Module `faisabilite` : un « robot joueur » qui explore un niveau avec la
//! VRAIE physique (`Joueur::actualiser`) pour prouver qu'il se termine.
//!
//! Principe : on part d'un état stable (le joueur posé, immobile). Pour chaque
//! état, on essaie une famille de « plans » d'entrées (courir, sauter plus ou
//! moins longtemps, planer, lâcher la direction en l'air...) et on simule
//! chaque plan pas à pas. Un plan qui se termine posé, vivant, donne un nouvel
//! état à explorer (recherche en largeur). On note au passage ce qui a été
//! touché : arrivée, gouttes d'or, lanternes.
//!
//! Simplifications volontaires (documentées pour rester honnêtes) :
//! - ennemis, corbeaux et éclairs sont ignorés : ils sont évitables en
//!   attendant ou en écrasant, et leur timing ne change pas l'atteignabilité ;
//! - SAUF le chat endormi : il ne bouge pas et garde un passage, on ne peut
//!   donc pas l'éviter. Le robot le garde, TOUJOURS endormi (comme le
//!   ventilateur toujours actif : un joueur patient attend son sommeil). Le
//!   heurter de côté tue ; il faut lui sauter dessus. Un chat écrasé le reste :
//!   chaque état du robot retient les chats déjà écrasés (`Noeud::chats`) ;
//! - la grêle, elle, est GARDÉE : sa colonne est fixe et tombe sans arrêt, donc
//!   attendre ne sert à rien. Un plan qui la touche parapluie fermé finit par une
//!   mort (`resoudre_contacts`) et il est écarté ; le robot la traverse avec ses
//!   plans « planer », ou passe dessous quand un solide l'arrête ;
//! - les plateformes friables restent solides (on peut toujours s'en
//!   détacher avant 0,5 s) ;
//! - un rebond d'auvent n'est PAS une simplification : la vraie physique relance
//!   le robot, et chaque rebond devient un nouveau point de décision (noeud
//!   « lancé », voir `Noeud`) d'où il essaie les plans « en vol » ;
//! - une plateforme mobile est remplacée par DEUX plateformes immobiles à ses
//!   extrémités, et un joueur posé sur un bout est aussi considéré posé sur
//!   l'autre (il suffit d'attendre d'être emporté) ;
//! - un ventilateur est traité comme un courant ascendant TOUJOURS actif : le
//!   robot ne connaît pas le temps, et son cycle (2 s de souffle, 2 s d'arrêt)
//!   ne change pas l'atteignabilité, un joueur patient attendant le souffle ;
//! - une montgolfière est remplacée, comme une plateforme mobile, par DEUX
//!   nacelles immobiles : au repos (altitude de départ) et en haut (`altitudes`).
//!   Un joueur posé sur la nacelle au repos est aussi considéré posé sur la
//!   nacelle en haut (il suffit d'attendre la montée), si sa boîte monte sans
//!   rien toucher (sinon la vraie nacelle s'arrêterait plus bas : le robot
//!   renonce, c'est prudent). La redescente n'est pas simulée : la nacelle en
//!   haut reste là, comme les deux bouts d'une plateforme mobile ;
//! - une aiguille d'horloge est remplacée par DOUZE plateformes immobiles, une
//!   par heure de son cadran. Un joueur posé sur l'une est aussi considéré posé
//!   sur chacune des autres (il suffit d'attendre que l'aiguille l'y emmène, dans
//!   le sens horaire), avec le temps du trajet comme coût, si sa boîte ne touche
//!   aucune tuile solide en chemin (`Cadran::trajet`). Les douze places, toutes
//!   présentes à la fois, gênent parfois un saut qui passerait en vrai : c'est
//!   prudent. Le robot de QA, lui, attend vraiment l'aiguille ;
//! - le Grand Nuage (boss) est gardé, TOUJOURS essoufflé, ses DEUX courants
//!   ascendants actifs (un joueur patient attend la fin d'un motif d'éclairs, et
//!   le courant du bon côté) ; ses éclairs sont donc ignorés, comme les éclairs
//!   `e`. Un plan qui lui retombe sur la tête « arrive » (`Fin::Arrivee`) : un
//!   coup se refait trois fois de la même façon, Pépin retombant chaque fois dans
//!   l'arène. Les tests du niveau d'essai vérifient la tête depuis CHACUN des deux
//!   courants (ils changent de côté d'un motif à l'autre) ;
//! - un escargot est remplacé par un tremplin IMMOBILE qui couvre tout son
//!   trajet (`Escargot::etendu_au_trajet`) : partout où il peut passer, un
//!   joueur patient qui l'attend au bon endroit lui tombe dessus et rebondit.
//!   Le temps n'existe pas pour le robot : il ne sait pas qu'il faut attendre,
//!   ni qu'en sautant au-dessus de ce trajet on peut aussi retomber à côté de
//!   l'escargot (il rebondit toujours). Le robot de QA, sur la vraie
//!   simulation, attend vraiment l'escargot.
//!
//! Notions Rust illustrées ici (à reprendre dans le README) :
//! - `struct` d'options + `Default` ; `VecDeque` et `HashSet` (collections) ;
//! - boucles imbriquées sur des tableaux constants de paramètres ;
//! - emprunts : la grille et les courants sont PRÊTÉS à `Environnement`.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use crate::aabb::{touche_un_obstacle, Aabb};
use crate::entities::contacts::{resoudre_contacts, verifier_vide, Contact};
use crate::entities::montgolfiere::VITESSE_MONTEE;
use crate::entities::{solides_des_entites, Entite, TypeEnnemi};
use crate::level::{Niveau, VITESSE_PLATEFORMES};
use crate::physics::{
    CourantAscendant, Environnement, VentLateral, HZ, JOUEUR_H, JOUEUR_L, VITESSE_PLANER_CHUTE,
};
use crate::player::{Direction, Entrees, Joueur};

/// Ce que le robot a le droit d'utiliser.
#[derive(Debug, Clone, Copy)]
pub struct Options {
    /// Peut-il ouvrir le parapluie (maintenir SAUT en tombant) ?
    pub planer: bool,
    /// Les courants ascendants soufflent-ils ?
    pub courants: bool,
    /// Le vent latéral souffle-t-il ?
    pub vents: bool,
    /// Robot précis (beaucoup de plans) ou « maladroit » (peu de choix : élans
    /// et durées de saut grossiers). Un niveau que le robot maladroit termine
    /// n'exige pas de précision au pixel près.
    pub precis: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            planer: true,
            courants: true,
            vents: true,
            precis: true,
        }
    }
}

/// Résultat d'une exploration.
#[derive(Debug, Clone)]
pub struct Rapport {
    /// L'arrivée est-elle atteignable ?
    pub arrivee: bool,
    /// Gouttes d'or touchées par un plan qui survit (indice 0 à 2).
    pub gouttes: [bool; 3],
    /// Pour chaque lanterne du niveau (ordre des entités) : a-t-elle été allumée ?
    pub lanternes: Vec<bool>,
    /// Durée (en pas de 1/60 s) du trajet le plus rapide trouvé jusqu'à l'arrivée,
    /// attentes de plateformes mobiles comprises (course « speedrun »).
    pub pas_arrivee: Option<u32>,
    /// Nombre d'états stables explorés.
    pub etats: usize,
    /// Abscisse (px) la plus à droite atteinte, utile pour déboguer un échec.
    pub x_max: f32,
    /// Position `(x, y)` de l'état stable le plus à droite.
    pub dernier_etat: (f32, f32),
}

/// Paramètres d'un plan d'entrées.
#[derive(Clone, Copy)]
struct Plan {
    dir: Direction,
    /// Pas passés à courir au sol avant le saut.
    elan: u16,
    /// Pas de maintien du saut (0 = pas de saut : on marche dans le vide).
    saut: u16,
    planer: bool,
    /// Direction conservée pendant la montée ?
    dir_montee: bool,
    /// Direction conservée pendant la descente ?
    dir_descente: bool,
}

const ELANS: [u16; 9] = [0, 4, 8, 14, 22, 32, 50, 80, 120];
const SAUTS: [u16; 5] = [0, 5, 9, 13, 40];
/// Choix grossiers du robot maladroit.
const ELANS_GROSSIERS: [u16; 5] = [0, 8, 22, 50, 120];
const SAUTS_GROSSIERS: [u16; 3] = [0, 13, 40];
/// Durée maximale d'un plan (pas) dans un niveau d'un écran de haut : 10 s.
const PAS_MAX: u16 = 600;
/// Pas d'immobilité qui valident un atterrissage « stable ».
const PAS_STABILISATION: u16 = 20;

/// Durée maximale d'un plan (pas) dans un niveau de `hauteur_px` pixels de haut.
///
/// Un plan se termine posé : dans un niveau haut (quatre écrans, carte 5.2), un plané
/// d'un bout à l'autre du niveau dure plus de 10 s (le parapluie ouvert descend à
/// `VITESSE_PLANER_CHUTE`, 36 px/s : 720 px en 20 s). La limite est donc le temps de
/// planer sur TOUTE la hauteur, plus 2 s ; jamais moins de 10 s (`PAS_MAX`). Un niveau
/// d'un écran (176 px : 5 s de plané) garde exactement les 10 s d'avant. Le robot de QA
/// du jeu (`game/src/bot_qa.rs`) applique la même règle.
pub fn pas_max_plan(hauteur_px: f32) -> u16 {
    let plane_complet = (hauteur_px / VITESSE_PLANER_CHUTE * HZ as f32).ceil() as u32;
    (plane_complet + 2 * HZ).clamp(u32::from(PAS_MAX), u32::from(u16::MAX)) as u16
}

fn direction(sens: i8) -> Direction {
    match sens {
        s if s < 0 => Direction::Gauche,
        0 => Direction::Immobile,
        _ => Direction::Droite,
    }
}

/// Fin d'un plan.
enum Fin {
    /// Mort ou chute dans le vide.
    Mort,
    /// Arrivée touchée.
    Arrivee,
    /// Posé et immobile en `(x, y)`.
    Pose(f32, f32),
    /// Relancé par un auvent en `(x, y)` : un nouveau choix commence en l'air
    /// (voir `Noeud::lance`).
    Lance(f32, f32),
    /// Temps écoulé ou état instable : on abandonne ce plan.
    Abandon,
}

/// Un état d'où le robot choisit son prochain plan.
///
/// - posé (`lance` faux) : Pépin est immobile au sol en `(x, y)` ;
/// - lancé (`lance` vrai) : un auvent vient de le relancer depuis `(x, y)`.
///   L'état du joueur ne dépend alors QUE de cette position (vitesse et élan
///   sont remis à neuf par `Joueur::lancer_par_auvent`) : c'est donc un vrai
///   point de décision, comme un atterrissage. Chaque rebond d'une chaîne
///   d'auvents est ainsi un nouveau choix (garder la droite, la gauche, rien).
///
/// `chats` : les chats déjà écrasés (bit `k` = `k`-ième chat du niveau). Le même
/// endroit, chat écrasé ou non, donne deux états différents : derrière un chat
/// vivant on ne repasse pas, derrière un chat écrasé si.
#[derive(Clone, Copy)]
struct Noeud {
    x: f32,
    y: f32,
    lance: bool,
    chats: u32,
}

/// Clé de fusion d'un noeud (voir `Noeud::cle`).
type CleNoeud = (i32, i32, bool, u32);

/// Entrée de la file de priorité : `(coût en pas, x, y, lancé, chats écrasés)`.
/// `x` et `y` sont rangés en bits (`f32::to_bits`) : un `f32` n'est pas `Ord`,
/// ses bits si (et ils redonnent exactement le même `f32`).
type EntreeTas = (u32, u32, u32, bool, u32);

impl Noeud {
    fn pose(x: f32, y: f32, chats: u32) -> Self {
        Self {
            x,
            y,
            lance: false,
            chats,
        }
    }

    /// Clé de fusion : une case de 8 px en x, y arrondi, le genre du noeud et
    /// les chats écrasés.
    fn cle(self) -> CleNoeud {
        let (kx, ky) = cle((self.x, self.y));
        (kx, ky, self.lance, self.chats)
    }
}

/// Au plus 32 chats par niveau (un bit chacun dans `Noeud::chats`).
const CHATS_MAX: usize = 32;

/// Trajet d'une plateforme mobile : (extrémité a, extrémité b, largeur).
type Trajet = ((f32, f32), (f32, f32), f32);

/// Montée d'une montgolfière : (x de la nacelle, haut de la nacelle au repos,
/// haut de la nacelle en haut, largeur).
type Montee = (f32, f32, f32, f32);

/// Nombre de places d'une aiguille d'horloge vue par le robot : une par heure.
const HEURES: usize = 12;

/// Une aiguille d'horloge vue par le robot : sa plateforme (pour retrouver sa
/// place à n'importe quel moment du tour) et ses 12 places, une par heure du
/// cadran (coin haut-gauche ; 0 = midi).
struct Cadran {
    aiguille: crate::entities::PlateformeMobile,
    places: [(f32, f32); HEURES],
}

impl Cadran {
    /// Debout sur la place `heure` à `(x, y)`, Pépin attend que l'aiguille l'emmène
    /// à la place `heure + n` (sens horaire). Renvoie où il arrive et l'attente (en
    /// pas), ou `None` si, en chemin, sa boîte toucherait une tuile solide (on
    /// regarde 8 fois par heure). Prudent : le vrai portage le retiendrait.
    fn trajet(
        &self,
        niveau: &Niveau,
        (x, y): (f32, f32),
        heure: usize,
        n: usize,
    ) -> Option<((f32, f32), u32)> {
        let depart = self.places[heure];
        for k in 1..=8 * n {
            let tour = (heure as f32 + k as f32 / 8.0) / HEURES as f32;
            let p = self.aiguille.coin_a(tour)?;
            let boite = Aabb::new(x + p.0 - depart.0, y + p.1 - depart.1, JOUEUR_L, JOUEUR_H);
            if touche_un_obstacle(&boite, niveau, &[]) {
                return None;
            }
        }
        let arrivee = self.places[(heure + n) % HEURES];
        let pas_par_tour = self.aiguille.cercle()?.pas_par_tour();
        let attente = (n as f32 / HEURES as f32 * pas_par_tour).ceil() as u32;
        Some((
            (x + arrivee.0 - depart.0, y + arrivee.1 - depart.1),
            attente,
        ))
    }
}

/// Explore le niveau depuis `depart` (coin haut-gauche du joueur).
pub fn analyser(niveau: &Niveau, depart: (f32, f32), options: Options) -> Rapport {
    // Solides immobiles : friables + extrémités des plateformes mobiles + nacelles
    // des montgolfières (au repos et en haut).
    let mut solides: Vec<Aabb> = solides_des_entites(&niveau.entites);
    let mut entites: Vec<Entite> = Vec::new();
    let mut lanternes: Vec<(f32, f32)> = Vec::new();
    // (extrémité a, extrémité b, largeur) de chaque plateforme mobile.
    let mut trajets: Vec<Trajet> = Vec::new();
    let mut montees: Vec<Montee> = Vec::new();
    let mut cadrans: Vec<Cadran> = Vec::new();
    for e in &niveau.entites {
        match e {
            Entite::Mobile(m) => {
                // Plateforme en aller-retour : ses deux extrémités.
                if let Some((a, b)) = m.extremites() {
                    solides.push(Aabb::new(a.0, a.1, m.boite.w, m.boite.h));
                    solides.push(Aabb::new(b.0, b.1, m.boite.w, m.boite.h));
                    trajets.push((a, b, m.boite.w));
                }
                // Aiguille d'horloge : ses 12 places, une par heure du cadran.
                if m.cercle().is_some() {
                    let places: [(f32, f32); HEURES] = std::array::from_fn(|h| {
                        m.coin_a(h as f32 / HEURES as f32)
                            .expect("une aiguille a un cercle")
                    });
                    for &(x, y) in &places {
                        solides.push(Aabb::new(x, y, m.boite.w, m.boite.h));
                    }
                    cadrans.push(Cadran {
                        aiguille: m.clone(),
                        places,
                    });
                }
            }
            Entite::Montgolfiere(m) => {
                // La nacelle au repos est déjà dans `solides` (niveau tout juste lu :
                // elle est à son altitude de départ). On ajoute la nacelle en haut.
                let (bas, haut) = m.altitudes();
                if haut < bas {
                    solides.push(Aabb::new(m.boite.x, haut, m.boite.w, m.boite.h));
                    montees.push((m.boite.x, bas, haut, m.boite.w));
                }
            }
            // Le chat endormi est gardé : il garde un passage (voir l'en-tête). Ses
            // entités ne sont jamais actualisées pendant la recherche : il reste
            // dans son premier sommeil.
            Entite::Ennemi(c) if c.type_ennemi == TypeEnnemi::Chat => entites.push(e.clone()),
            Entite::Ennemi(_) | Entite::Eclair(_) => {}
            // Le Grand Nuage : toujours essoufflé (aucun éclair), voir l'en-tête.
            Entite::Nuage(n) => {
                let mut essouffle = n.clone();
                essouffle.essouffler();
                entites.push(Entite::Nuage(essouffle));
            }
            Entite::Escargot(s) => {
                // Son trajet sur la grille et les solides immobiles du niveau.
                let fixes = solides_des_entites(&niveau.entites);
                let env = Environnement {
                    grille: niveau,
                    solides: &fixes,
                    courants: &[],
                    vents: &[],
                };
                entites.push(Entite::Escargot(s.etendu_au_trajet(&env)));
            }
            // La grêle reste : elle tue les plans qui la touchent parapluie fermé.
            Entite::Grele(_) => entites.push(e.clone()),
            Entite::Lanterne(l) => {
                lanternes.push(l.point_de_reapparition());
                entites.push(e.clone());
            }
            autre => entites.push(autre.clone()),
        }
    }
    // Les friables sont déjà dans `solides` : on les retire des entités pour
    // qu'elles ne s'effritent jamais pendant la recherche.
    entites.retain(|e| !matches!(e, Entite::Friable(_)));
    // Place des chats dans `entites` : le bit `k` de `Noeud::chats` est le chat
    // `chats[k]`.
    let chats: Vec<usize> = entites
        .iter()
        .enumerate()
        .filter(|(_, e)| matches!(e, Entite::Ennemi(c) if c.type_ennemi == TypeEnnemi::Chat))
        .map(|(i, _)| i)
        .collect();
    assert!(chats.len() <= CHATS_MAX, "trop de chats : {}", chats.len());

    let mut courants: Vec<CourantAscendant> = if options.courants {
        niveau.courants.clone()
    } else {
        Vec::new()
    };
    // Ventilateurs : le robot est INSENSIBLE au temps, donc on considère leur
    // courant comme TOUJOURS actif (un joueur patient attend simplement le
    // bon moment du cycle de 4 s). C'est une simplification documentée, à
    // l'image de l'ignorance des éclairs : le timing ne change pas
    // l'atteignabilité, on peut attendre la phase de souffle.
    // Même chose pour les deux courants du Grand Nuage : un joueur patient
    // attend son essoufflement, du bon côté.
    if options.courants {
        for e in &niveau.entites {
            match e {
                Entite::Ventilateur(v) => courants.push(CourantAscendant::new(v.zone)),
                Entite::Nuage(n) => {
                    courants.extend(n.courants.iter().map(|&z| CourantAscendant::new(z)));
                }
                _ => {}
            }
        }
    }
    // Le robot joue avec la VRAIE physique : le vent latéral pousse ses plans
    // (ou les fait échouer) comme à un joueur. Ses plans « planer » s'en
    // servent pour allonger la portée d'un plané dans le sens du vent, et il
    // les évite quand le vent le porterait vers le danger.
    let vents: Vec<VentLateral> = if options.vents {
        niveau.vents.clone()
    } else {
        Vec::new()
    };
    let env = Environnement {
        grille: niveau,
        solides: &solides,
        courants: &courants,
        vents: &vents,
    };

    let mut rapport = Rapport {
        arrivee: false,
        pas_arrivee: None,
        gouttes: [false; 3],
        lanternes: vec![false; lanternes.len()],
        etats: 0,
        x_max: depart.0,
        dernier_etat: depart,
    };
    // Dijkstra sur la durée : `meilleur` retient le coût (en pas) le plus bas
    // connu pour chaque case de 8 px (et chaque genre de noeud : posé ou lancé) ;
    // `tas` sert les états du moins coûteux au plus coûteux (`Reverse`
    // transforme le tas max en tas min).
    let mut meilleur: HashMap<CleNoeud, u32> = HashMap::new();
    let mut tas: BinaryHeap<Reverse<EntreeTas>> = BinaryHeap::new();
    let n0 = Noeud::pose(depart.0, depart.1, 0);
    meilleur.insert(n0.cle(), 0);
    tas.push(Reverse((
        0,
        depart.0.to_bits(),
        depart.1.to_bits(),
        false,
        0,
    )));
    let tous_les_plans: Vec<Plan> = plans(options).collect();
    // Après un rebond d'auvent, on est en l'air : seuls les choix « en vol »
    // ont un sens (ni élan au sol ni saut).
    let plans_lance: Vec<Plan> = plans_en_vol(options).collect();

    while let Some(Reverse((cout, bx, by, lance, ecrases))) = tas.pop() {
        let noeud = Noeud {
            x: f32::from_bits(bx),
            y: f32::from_bits(by),
            lance,
            chats: ecrases,
        };
        let etat = (noeud.x, noeud.y);
        if meilleur.get(&noeud.cle()).is_some_and(|&c| c < cout) {
            continue; // version périmée : un chemin plus court existe
        }
        rapport.etats += 1;
        if etat.0 > rapport.x_max {
            rapport.x_max = etat.0;
            rapport.dernier_etat = etat;
        }
        let a_essayer = if lance { &plans_lance } else { &tous_les_plans };
        for plan in a_essayer {
            let mut ents = entites.clone();
            // Les chats déjà écrasés dans cet état le restent.
            for (k, &i) in chats.iter().enumerate() {
                if noeud.chats & (1 << k) != 0 {
                    if let Entite::Ennemi(c) = &mut ents[i] {
                        c.ecraser();
                    }
                }
            }
            let mut touches: Vec<Contact> = Vec::new();
            let (fin, duree) = simuler(niveau, &env, noeud, *plan, &mut ents, &mut touches);
            if matches!(fin, Fin::Mort | Fin::Abandon) {
                continue;
            }
            // Chats écrasés à la fin du plan (ceux d'avant, plus ceux de ce plan).
            let ecrases = chats
                .iter()
                .enumerate()
                .fold(0u32, |m, (k, &i)| match &ents[i] {
                    Entite::Ennemi(c) if !c.est_dangereux() => m | (1 << k),
                    _ => m,
                });
            for c in &touches {
                match *c {
                    Contact::Goutte(i) if (i as usize) < 3 => rapport.gouttes[i as usize] = true,
                    Contact::Lanterne(x, y) => {
                        if let Some(k) = lanternes.iter().position(|&p| p == (x, y)) {
                            rapport.lanternes[k] = true;
                        }
                    }
                    _ => {}
                }
            }
            let cout_plan = cout + u32::from(duree);
            let mut ajouter = |arrive: Noeud, cout: u32| {
                let k = arrive.cle();
                if meilleur.get(&k).is_none_or(|&c| cout < c) {
                    meilleur.insert(k, cout);
                    tas.push(Reverse((
                        cout,
                        arrive.x.to_bits(),
                        arrive.y.to_bits(),
                        arrive.lance,
                        arrive.chats,
                    )));
                }
            };
            match fin {
                Fin::Arrivee => {
                    rapport.arrivee = true;
                    let mieux = rapport.pas_arrivee.is_none_or(|p| cout_plan < p);
                    if mieux {
                        rapport.pas_arrivee = Some(cout_plan);
                    }
                }
                Fin::Lance(x, y) => ajouter(
                    Noeud {
                        x,
                        y,
                        lance: true,
                        chats: ecrases,
                    },
                    cout_plan,
                ),
                Fin::Pose(x, y) => {
                    ajouter(Noeud::pose(x, y, ecrases), cout_plan);
                    // Debout sur un bout de plateforme mobile ? On peut attendre
                    // d'être emporté à l'autre bout : on ajoute cet état-là aussi,
                    // avec le temps de trajet comme coût.
                    for &(a, b, w) in &trajets {
                        for (de, vers) in [(a, b), (b, a)] {
                            let dessus = (y + JOUEUR_H - de.1).abs() < 0.01
                                && x + JOUEUR_L > de.0
                                && x < de.0 + w;
                            if dessus {
                                let (dx, dy) = (vers.0 - de.0, vers.1 - de.1);
                                let trajet = (dx * dx + dy * dy).sqrt() / VITESSE_PLATEFORMES;
                                let attente = (trajet * HZ as f32) as u32;
                                ajouter(Noeud::pose(x + dx, y + dy, ecrases), cout_plan + attente);
                            }
                        }
                    }
                    // Debout sur une nacelle de montgolfière au repos ? On attend la
                    // montée : posé sur la nacelle en haut, au même x, avec le temps
                    // de la montée comme coût — si la boîte de Pépin monte sans rien
                    // toucher (une corniche au-dessus de lui arrêterait la vraie
                    // nacelle plus bas : le robot n'en tient pas compte, prudent).
                    for &(nx, bas, haut, w) in &montees {
                        let dessus =
                            (y + JOUEUR_H - bas).abs() < 0.01 && x + JOUEUR_L > nx && x < nx + w;
                        if !dessus {
                            continue;
                        }
                        let dy = haut - bas; // négatif : vers le haut
                        let balayage = Aabb::new(x, y + dy, JOUEUR_L, JOUEUR_H - dy);
                        if !touche_un_obstacle(&balayage, niveau, &[]) {
                            let attente = (-dy / VITESSE_MONTEE * HZ as f32).ceil() as u32;
                            ajouter(Noeud::pose(x, y + dy, ecrases), cout_plan + attente);
                        }
                    }
                    // Debout sur une place d'aiguille d'horloge ? On attend qu'elle
                    // nous emmène à chacune des 11 autres heures, dans le sens
                    // horaire, avec le temps du trajet comme coût.
                    for cadran in &cadrans {
                        let w = cadran.aiguille.boite.w;
                        for (heure, place) in cadran.places.iter().enumerate() {
                            let dessus = (y + JOUEUR_H - place.1).abs() < 0.01
                                && x + JOUEUR_L > place.0
                                && x < place.0 + w;
                            if !dessus {
                                continue;
                            }
                            for n in 1..HEURES {
                                if let Some(((ax, ay), attente)) =
                                    cadran.trajet(niveau, (x, y), heure, n)
                                {
                                    ajouter(Noeud::pose(ax, ay, ecrases), cout_plan + attente);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    rapport
}

/// Deux états très proches sont fusionnés : on garde une case de 8 px en x.
fn cle(p: (f32, f32)) -> (i32, i32) {
    ((p.0 / 8.0).floor() as i32, p.1.round() as i32)
}

/// Tous les plans à essayer depuis un état.
fn plans(options: Options) -> impl Iterator<Item = Plan> {
    let sens = [-1i8, 1];
    let planer = if options.planer {
        &[false, true][..]
    } else {
        &[false][..]
    };
    let (elans, sauts): (&[u16], &[u16]) = if options.precis {
        (&ELANS, &SAUTS)
    } else {
        (&ELANS_GROSSIERS, &SAUTS_GROSSIERS)
    };
    let mut v = Vec::new();
    for &s in &sens {
        for &elan in elans {
            for &saut in sauts {
                for &p in planer {
                    for &montee in &[true, false] {
                        for &descente in &[true, false] {
                            // Sans saut, « montée » n'a aucun sens : on évite les doublons.
                            if saut == 0 && !montee {
                                continue;
                            }
                            v.push(Plan {
                                dir: direction(s),
                                elan,
                                saut,
                                planer: p,
                                dir_montee: montee,
                                dir_descente: descente,
                            });
                        }
                    }
                }
            }
        }
    }
    v.into_iter()
}

/// Les plans « en vol », à essayer juste après un rebond d'auvent (noeud lancé) :
/// une direction (gauche, droite, ou aucune), gardée pendant la montée et/ou la
/// descente, parapluie ouvert ou non en descente. Ni élan au sol ni saut.
fn plans_en_vol(options: Options) -> impl Iterator<Item = Plan> {
    let planer = if options.planer {
        &[false, true][..]
    } else {
        &[false][..]
    };
    let mut v = Vec::new();
    for &p in planer {
        for &s in &[-1i8, 1] {
            for &montee in &[true, false] {
                for &descente in &[true, false] {
                    v.push(Plan {
                        dir: direction(s),
                        elan: 0,
                        saut: 0,
                        planer: p,
                        dir_montee: montee,
                        dir_descente: descente,
                    });
                }
            }
        }
        // Ne rien toucher (le vol tout seul : 4 tuiles plus loin).
        v.push(Plan {
            dir: Direction::Immobile,
            elan: 0,
            saut: 0,
            planer: p,
            dir_montee: false,
            dir_descente: false,
        });
    }
    v.into_iter()
}

/// Simule un plan complet depuis un noeud : posé (état stable) ou lancé (un
/// auvent vient de relancer Pépin). Le plan s'arrête posé, mort, à l'arrivée,
/// ou au prochain rebond d'auvent (`Fin::Lance` : un nouveau choix commence).
fn simuler(
    niveau: &Niveau,
    env: &Environnement<'_>,
    depart: Noeud,
    plan: Plan,
    ents: &mut [Entite],
    touches: &mut Vec<Contact>,
) -> (Fin, u16) {
    let mut j = Joueur::new(depart.x, depart.y);
    if depart.lance {
        // Exactement l'état laissé par l'auvent : seule la position compte.
        j.lancer_par_auvent();
    } else {
        // Un pas à vide pour que `au_sol` soit vrai dès le début.
        j.actualiser(&Entrees::default(), env);
    }
    let bas_monde = niveau.hauteur_px();
    let pas_max = pas_max_plan(bas_monde);
    let mut en_l_air: u16 = 0;
    // Lancé : on est déjà en l'air, les commandes « en vol » s'appliquent dès le 1er pas.
    let mut a_decolle = depart.lance;
    let mut stable: u16 = 0;
    let mut saut_envoye: u16 = 0;

    for pas in 0..pas_max {
        let mut entrees = Entrees::default();
        let marche_finie = plan.saut == 0 && pas >= plan.elan;
        if !a_decolle && stable == 0 && !marche_finie {
            // --- Au sol : élan, puis saut ---
            entrees.direction = plan.dir;
            if pas >= plan.elan {
                entrees.saut = true;
                saut_envoye += 1;
            }
        } else if a_decolle && stable == 0 {
            // --- En l'air ---
            let montee = j.vitesse.y < 0.0;
            let garde = if montee {
                plan.dir_montee
            } else {
                plan.dir_descente
            };
            entrees.direction = if garde { plan.dir } else { Direction::Immobile };
            if saut_envoye > 0 && saut_envoye < plan.saut && montee {
                entrees.saut = true; // maintien du saut pendant la montée
                saut_envoye += 1;
            } else if plan.planer && !montee && en_l_air >= 7 {
                entrees.saut = true; // parapluie ouvert en descente
            }
        }
        // Après l'atterrissage : aucune entrée (on laisse la vitesse s'éteindre).

        j.actualiser(&entrees, env);
        let contacts = resoudre_contacts(&mut j, ents);
        let mut relance = false;
        for c in contacts {
            match c {
                Contact::Mort(_) => return (Fin::Mort, pas),
                Contact::Auvent => relance = true,
                // Un saut sur la tête du Grand Nuage : le combat se gagne ainsi.
                Contact::Nuage => return (Fin::Arrivee, pas),
                autre => touches.push(autre),
            }
        }
        if verifier_vide(&j, bas_monde).is_some() {
            return (Fin::Mort, pas);
        }
        if niveau.arrivee.is_some_and(|a| j.boite.chevauche(&a)) {
            return (Fin::Arrivee, pas);
        }
        if relance {
            return (Fin::Lance(j.boite.x, j.boite.y), pas + 1);
        }

        if !j.au_sol {
            a_decolle = true;
            en_l_air += 1;
            stable = 0;
        } else if a_decolle || marche_finie {
            // Posé : on attend que la vitesse s'éteigne.
            stable += 1;
            if stable >= PAS_STABILISATION {
                // Les pas d'immobilité ne comptent pas : un vrai joueur enchaîne.
                return (Fin::Pose(j.boite.x, j.boite.y), pas + 1 - PAS_STABILISATION);
            }
        }
    }
    (Fin::Abandon, pas_max)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Niveau plat : sol de 8 tuiles, vide de `trou` tuiles (à `dh` tuiles plus
    /// bas si `dh > 0`), sol d'arrivée.
    fn niveau_a_trou(trou: usize, marche_haute: usize) -> Niveau {
        let largeur = 10 + trou + 10;
        let mut lignes = vec![".".repeat(largeur); 11];
        // Départ à gauche, arrivée à droite : toujours trois gouttes (rangée du haut).
        for (i, c) in [(2usize, 'o'), (4, 'o'), (6, 'o')] {
            lignes[0].replace_range(i..i + 1, &c.to_string());
        }
        lignes[7].replace_range(2..3, "@");
        lignes[7].replace_range(largeur - 3..largeur - 2, "X");
        for x in 0..largeur {
            let sol = x < 10 || x >= 10 + trou;
            if !sol {
                continue;
            }
            let haut = if x >= 10 + trou { 8 - marche_haute } else { 8 };
            for ligne in &mut lignes[haut..11] {
                ligne.replace_range(x..x + 1, "#");
            }
        }
        // Le X doit être posé au-dessus du sol d'arrivée.
        let ax = largeur - 3;
        lignes[7].replace_range(ax..ax + 1, ".");
        lignes[7 - marche_haute].replace_range(ax..ax + 1, "X");
        let texte = lignes.join("\n");
        Niveau::depuis_ascii(&texte).expect("niveau de test")
    }

    fn passe(trou: usize, marche: usize, options: Options) -> bool {
        let n = niveau_a_trou(trou, marche);
        analyser(&n, n.depart, options).arrivee
    }

    #[test]
    fn portee_du_saut_sans_parapluie() {
        let sans = Options {
            planer: false,
            ..Options::default()
        };
        let max = (1..14).take_while(|&t| passe(t, 0, sans)).last().unwrap();
        println!("écart max sans planer : {max} tuiles");
        assert!((3..=5).contains(&max), "écart max = {max}");
        assert!(!passe(max + 1, 0, sans));
    }

    #[test]
    fn le_parapluie_allonge_nettement_la_portee() {
        let sans = Options {
            planer: false,
            ..Options::default()
        };
        let avec = Options::default();
        let max_sans = (1..30).take_while(|&t| passe(t, 0, sans)).last().unwrap();
        let max_avec = (1..30).take_while(|&t| passe(t, 0, avec)).last().unwrap();
        println!("écart max avec planer : {max_avec} tuiles (sans : {max_sans})");
        assert!(
            max_avec >= max_sans + 3,
            "avec = {max_avec}, sans = {max_sans}"
        );
    }

    #[test]
    fn hauteur_de_marche_maximale() {
        let sans = Options {
            planer: false,
            ..Options::default()
        };
        let max = (1..7).take_while(|&h| passe(1, h, sans)).last().unwrap();
        println!("marche max : {max} tuiles");
        assert!((3..=4).contains(&max), "marche max = {max}");
        assert!(!passe(1, max + 1, sans));
    }

    #[test]
    fn un_mur_infranchissable_est_detecte() {
        // Témoin négatif : sans ce test, un robot trop permissif passerait inaperçu.
        assert!(!passe(
            40,
            0,
            Options {
                planer: false,
                courants: false,
                precis: true,
                ..Options::default()
            }
        ));
    }

    /// Mini-niveau pour les vents latéraux (54 x 11) : un vide de `trou` tuiles
    /// (colonne 10) entre deux sols, et un vent latéral de `sens` (1 = `>` vers
    /// la droite, -1 = `<` vers la gauche). Si `bande_basse` est fixé, la bande
    /// de vent couvre les lignes 5 à 7 (elle touche aussi la course au sol) ;
    /// sinon les lignes 3 à 7 (elle ne touche que les sauts et les planés).
    fn niveau_vent(trou: usize, sens: i8, bande_basse: bool) -> Niveau {
        let largeur = 54;
        let mut lignes = vec![".".repeat(largeur); 11];
        // Départ à gauche, arrivée à droite : toujours trois gouttes (rangée du haut).
        for (i, c) in [(2usize, 'o'), (8, 'o'), (33, 'o')] {
            lignes[0].replace_range(i..i + 1, &c.to_string());
        }
        lignes[7].replace_range(2..3, "@");
        lignes[7].replace_range(largeur - 3..largeur - 2, "X");
        let signe = if sens > 0 { '>' } else { '<' };
        let (hauteur_bande, (debut, fin)) = if bande_basse {
            (5usize, (14usize, 28))
        } else {
            (3, (10, 10 + trou))
        };
        for ligne in &mut lignes[hauteur_bande..=7] {
            ligne.replace_range(debut..fin, &signe.to_string().repeat(fin - debut));
        }
        for x in 0..largeur {
            let sol = x < 10 || x >= 10 + trou;
            if !sol {
                continue;
            }
            for ligne in &mut lignes[8..11] {
                ligne.replace_range(x..x + 1, "#");
            }
        }
        Niveau::depuis_ascii(&lignes.join("\n")).expect("niveau de test")
    }

    /// Un vide plus large que la portée du plané : seule la planée portée par le
    /// vent de dos (`>`) le franchit ; sans vent, le robot reste sur le bord.
    #[test]
    fn le_vent_porteur_prolonge_la_traversee_du_vide() {
        let n = niveau_vent(14, 1, false);
        assert!(!n.vents.is_empty());
        let avec = analyser(&n, n.depart, Options::default());
        assert!(
            avec.arrivee,
            "la planée portée par le vent doit franchir le vide : {avec:?}"
        );
        let sans = analyser(
            &n,
            n.depart,
            Options {
                vents: false,
                ..Options::default()
            },
        );
        assert!(
            !sans.arrivee,
            "sans vent le vide doit rester infranchissable"
        );
    }

    /// Le même principe, mais vent de face (`<`) au-dessus du vide : la planée est
    /// poussée vers l'arrière et le saut est freiné — la traversée devient
    /// impossible, alors que le vide SANS vent est franchissable (témoin).
    #[test]
    fn le_vent_de_face_bloque_la_traversee_du_vide() {
        let n = niveau_vent(8, -1, false);
        assert!(!n.vents.is_empty());
        let avec = analyser(&n, n.depart, Options::default());
        assert!(!avec.arrivee, "le vent de face doit bloquer la traversée");
        let sans = analyser(
            &n,
            n.depart,
            Options {
                vents: false,
                ..Options::default()
            },
        );
        assert!(
            sans.arrivee,
            "sans vent, le vide est franchissable : {sans:?}"
        );
    }

    /// Bande de vent de face au niveau du sol : le robot la résiste (course à
    /// 36 px/s nets) et arrive quand même — mais le vent ralentit la traversée.
    #[test]
    fn le_robot_resiste_au_vent_de_face_au_sol() {
        let n = niveau_vent(0, -1, true);
        assert!(!n.vents.is_empty());
        let sans = analyser(
            &n,
            n.depart,
            Options {
                vents: false,
                ..Options::default()
            },
        );
        let avec = analyser(&n, n.depart, Options::default());
        assert!(
            sans.arrivee && avec.arrivee,
            "le vent au sol freine mais n'arrête pas : sans {sans:?}, avec {avec:?}"
        );
        assert!(
            avec.pas_arrivee.unwrap() > sans.pas_arrivee.unwrap(),
            "le vent de face doit ralentir la traversée : avec {} pas, sans {} pas",
            avec.pas_arrivee.unwrap(),
            sans.pas_arrivee.unwrap()
        );
    }

    /// Mini-niveau pour la grêle (54 x 11) : un toit plat, départ à gauche, drapeau à
    /// droite. `grele` : une colonne `g` posée à la colonne 20 (ligne 1), ou rien
    /// (témoin). `abri` : une corniche (colonnes 18 à 23, ligne 5) arrête la colonne
    /// au-dessus de la course. `trou` : un vide de `trou` tuiles à partir de la
    /// colonne 18 (la grêle tombe alors dans le vide, jusqu'au bas de la carte).
    fn niveau_grele(grele: bool, abri: bool, trou: usize) -> Niveau {
        let largeur = 54;
        let mut lignes = vec![".".repeat(largeur); 11];
        for (i, c) in [(2usize, 'o'), (8, 'o'), (33, 'o')] {
            lignes[0].replace_range(i..i + 1, &c.to_string());
        }
        lignes[7].replace_range(2..3, "@");
        lignes[7].replace_range(largeur - 3..largeur - 2, "X");
        if grele {
            lignes[1].replace_range(20..21, "g");
        }
        if abri {
            lignes[5].replace_range(18..24, "######");
        }
        for x in 0..largeur {
            if (18..18 + trou).contains(&x) {
                continue;
            }
            for ligne in &mut lignes[8..11] {
                ligne.replace_range(x..x + 1, "#");
            }
        }
        Niveau::depuis_ascii(&lignes.join("\n")).expect("niveau de test")
    }

    /// Sans parapluie (options du robot).
    fn sans_planer() -> Options {
        Options {
            planer: false,
            ..Options::default()
        }
    }

    /// Un rideau de grêle du ciel jusqu'au toit : impossible à franchir parapluie
    /// fermé (ni à pied, ni d'un saut : la colonne est plus haute que le saut), alors
    /// que le même toit sans grêle se traverse à pied (témoin). En planant dessous,
    /// le robot passe — même le robot maladroit.
    #[test]
    fn la_grele_se_traverse_en_planant_dessous() {
        let temoin = niveau_grele(false, false, 0);
        assert!(analyser(&temoin, temoin.depart, sans_planer()).arrivee);
        let n = niveau_grele(true, false, 0);
        let sans = analyser(&n, n.depart, sans_planer());
        assert!(!sans.arrivee, "parapluie fermé, la grêle bloque : {sans:?}");
        let avec = analyser(&n, n.depart, Options::default());
        assert!(avec.arrivee, "en planant, la grêle se traverse : {avec:?}");
        let maladroit = analyser(
            &n,
            n.depart,
            Options {
                precis: false,
                ..Options::default()
            },
        );
        assert!(
            maladroit.arrivee,
            "le robot maladroit aussi : {maladroit:?}"
        );
    }

    /// Une corniche arrête la colonne au-dessus de la course : on passe dessous à
    /// pied, parapluie fermé (limite basse de la colonne).
    #[test]
    fn sous_un_abri_la_grele_ne_tombe_plus() {
        let n = niveau_grele(true, true, 0);
        let sans = analyser(&n, n.depart, sans_planer());
        assert!(sans.arrivee, "l'abri arrête la grêle : {sans:?}");
    }

    /// Un vide de 4 tuiles que l'on saute sans parapluie (témoin) ; avec un rideau
    /// de grêle au-dessus, le même saut tue : il faut sauter plus tôt et planer.
    #[test]
    fn un_rideau_de_grele_au_dessus_du_vide_oblige_a_planer() {
        let temoin = niveau_grele(false, false, 4);
        assert!(analyser(&temoin, temoin.depart, sans_planer()).arrivee);
        let n = niveau_grele(true, false, 4);
        assert!(!analyser(&n, n.depart, sans_planer()).arrivee);
        let avec = analyser(&n, n.depart, Options::default());
        assert!(avec.arrivee, "en planant : {avec:?}");
    }

    /// Mini-niveau HAUT (45 rangées, quatre écrans) : départ sur une corniche en haut à
    /// gauche, puis un vide de 40 rangées ; le seul sol est en bas, loin à droite. Sans
    /// parapluie, la chute est trop courte en largeur (on tombe dans le vide) ; avec, le
    /// plané dure environ 18 s, plus que les 10 s d'un plan dans un niveau d'un écran.
    fn niveau_haut_a_long_plane() -> Niveau {
        let (largeur, hauteur) = (60, 45);
        let mut lignes = vec![".".repeat(largeur); hauteur];
        lignes[0].replace_range(10..13, "ooo");
        lignes[2].replace_range(2..3, "@");
        lignes[3].replace_range(0..6, "######");
        lignes[43].replace_range(56..57, "X");
        lignes[44].replace_range(30..60, &"#".repeat(30));
        Niveau::depuis_ascii(&lignes.join("\n")).expect("niveau de test")
    }

    #[test]
    fn la_duree_d_un_plan_suit_la_hauteur_du_niveau() {
        // Un écran (11 rangées) : exactement les 10 s d'avant.
        assert_eq!(pas_max_plan(11.0 * 16.0), PAS_MAX);
        assert_eq!(pas_max_plan(0.0), PAS_MAX);
        // Quatre écrans (45 rangées) : un plané de 720 px (20 s) tient dans un plan.
        let quatre_ecrans = pas_max_plan(45.0 * 16.0);
        assert!(quatre_ecrans >= 20 * HZ as u16, "{quatre_ecrans}");
        assert!(pas_max_plan(1.0e9) == u16::MAX, "borné, pas de débordement");
    }

    #[test]
    fn un_long_plane_d_une_grille_haute_est_trouve() {
        let n = niveau_haut_a_long_plane();
        assert_eq!(n.hauteur(), 45);
        let avec = analyser(&n, n.depart, Options::default());
        assert!(avec.arrivee, "le long plané doit être trouvé : {avec:?}");
        // Le même niveau sans parapluie : bloqué (le vide est trop large).
        let sans = analyser(
            &n,
            n.depart,
            Options {
                planer: false,
                ..Options::default()
            },
        );
        assert!(!sans.arrivee, "sans parapluie, la chute est trop courte");
    }

    /// Mini-niveau des auvents : un toit de départ (colonnes 0 à 8), puis un vide
    /// au-dessus duquel six auvents (`A`, ligne 7) se suivent, le premier en colonne
    /// 10, chacun `ecarts[k]` tuiles après le précédent ; le toit d'arrivée commence
    /// 4 tuiles après le dernier. `avec_auvents` faux : les mêmes cases restent vides
    /// (témoin). Le vide (au moins 20 tuiles) est bien plus large qu'un saut plané.
    fn niveau_auvents(ecarts: [usize; 5], avec_auvents: bool) -> Niveau {
        let mut colonnes = vec![10];
        for e in ecarts {
            colonnes.push(colonnes[colonnes.len() - 1] + e);
        }
        let arrivee_x = colonnes[5] + 4;
        let largeur = arrivee_x + 8;
        let mut lignes = vec![".".repeat(largeur); 11];
        lignes[0].replace_range(1..4, "ooo");
        lignes[7].replace_range(2..3, "@");
        lignes[7].replace_range(largeur - 3..largeur - 2, "X");
        if avec_auvents {
            for x in colonnes {
                lignes[7].replace_range(x..x + 1, "A");
            }
        }
        for x in (0..9).chain(arrivee_x..largeur) {
            for ligne in &mut lignes[8..11] {
                ligne.replace_range(x..x + 1, "#");
            }
        }
        Niveau::depuis_ascii(&lignes.join("\n")).expect("niveau de test")
    }

    /// Écarts essayés : 3, 4 ou 5 tuiles partout, puis un rang irrégulier (3 et 5
    /// alternés).
    const ECARTS: [[usize; 5]; 4] = [[3; 5], [4; 5], [5; 5], [3, 5, 3, 5, 4]];

    /// Le robot sait se servir des auvents : six auvents espacés de 3 à 5 tuiles
    /// au-dessus d'un vide qu'aucun saut plané ne franchit (témoin sans auvents).
    /// Chaque rebond est un nouveau point de décision (noeud « lancé ») ; le robot
    /// peut aussi sauter un auvent en planant, s'il le trouve plus rapide.
    #[test]
    fn le_robot_traverse_le_vide_par_les_auvents() {
        for ecarts in ECARTS {
            let temoin = niveau_auvents(ecarts, false);
            let sans = analyser(&temoin, temoin.depart, Options::default());
            assert!(
                !sans.arrivee,
                "{ecarts:?} : le vide se franchit sans auvent ?"
            );
            let n = niveau_auvents(ecarts, true);
            let avec = analyser(&n, n.depart, Options::default());
            assert!(
                avec.arrivee,
                "{ecarts:?} : chaîne d'auvents non trouvée : {avec:?}"
            );
        }
    }

    /// Le robot maladroit (choix grossiers au sol) y arrive aussi : en vol, après
    /// un rebond, il a les mêmes choix que le robot précis.
    #[test]
    fn le_robot_maladroit_traverse_aussi_par_les_auvents() {
        for ecarts in ECARTS {
            let n = niveau_auvents(ecarts, true);
            let maladroit = analyser(
                &n,
                n.depart,
                Options {
                    precis: false,
                    ..Options::default()
                },
            );
            assert!(maladroit.arrivee, "{ecarts:?} : {maladroit:?}");
        }
    }

    /// Mini-niveau de la montgolfière, 16 x 16 : un quai (ligne 13, colonnes 0 à 3)
    /// où Pépin part, la nacelle (`b`) au ras du quai en colonne 4, et un balcon
    /// (colonnes 5 à 11) dont le dessus est en ligne `rangee_balcon`, avec deux
    /// gouttes et le drapeau. La nacelle monte de 8 tuiles au plus : du haut à
    /// y = 208 jusqu'à y = 80, au ras d'un balcon en ligne 5. Le balcon touche la
    /// colonne de la nacelle : Pépin doit se tenir sur sa partie gauche, sinon sa
    /// tête cogne le dessous du balcon. `avec_nacelle` faux : la case reste vide.
    fn niveau_montgolfiere(rangee_balcon: usize, avec_nacelle: bool) -> Niveau {
        let mut lignes = vec![".".repeat(16); 16];
        lignes[rangee_balcon - 1].replace_range(6..12, "o.o..X");
        lignes[rangee_balcon].replace_range(5..12, "#######");
        lignes[12].replace_range(1..3, "@o");
        lignes[13].replace_range(0..5, if avec_nacelle { "####b" } else { "####." });
        Niveau::depuis_ascii(&lignes.join("\n")).expect("niveau de test")
    }

    /// Le robot monte au balcon par la montgolfière : 128 px au-dessus du quai,
    /// hors de portée d'un saut (témoin sans nacelle). Le parcours le plus rapide
    /// compte le temps de la montée (128 px à 48 px/s : 160 pas au moins).
    #[test]
    fn le_robot_monte_au_balcon_par_la_montgolfiere() {
        let temoin = niveau_montgolfiere(5, false);
        let sans = analyser(&temoin, temoin.depart, Options::default());
        assert!(!sans.arrivee, "le balcon s'atteint sans montgolfière ?");
        let n = niveau_montgolfiere(5, true);
        let avec = analyser(&n, n.depart, Options::default());
        assert!(avec.arrivee, "montée non trouvée : {avec:?}");
        assert_eq!(avec.gouttes, [true; 3]);
        let pas = avec.pas_arrivee.unwrap();
        assert!(pas >= 160, "la montée dure au moins 160 pas ({pas})");
        let maladroit = analyser(
            &n,
            n.depart,
            Options {
                precis: false,
                ..Options::default()
            },
        );
        assert!(maladroit.arrivee, "robot maladroit : {maladroit:?}");
    }

    /// La montée s'arrête à 8 tuiles : un balcon en ligne 1 (64 px au-dessus de la
    /// nacelle en haut, plus qu'un saut) reste hors de portée, même avec elle.
    #[test]
    fn le_robot_ne_monte_pas_plus_haut_que_la_montgolfiere() {
        let n = niveau_montgolfiere(1, true);
        let r = analyser(&n, n.depart, Options::default());
        assert!(!r.arrivee, "le balcon est trop haut : {r:?}");
    }

    /// Mini-niveau de l'escargot, 16 x 10 : un sol (rangée 9) entre un mur à gauche
    /// et une falaise à droite (colonnes 11 à 15, dessus en rangée 4 : 80 px
    /// au-dessus du sol, plus qu'un saut). L'escargot (`s`, colonne 8) patrouille sur
    /// le sol, entre le mur et la falaise. Le drapeau et deux gouttes sont en haut
    /// de la falaise. `avec_escargot` faux : la case reste vide.
    fn niveau_escargot(avec_escargot: bool) -> Niveau {
        let sol = if avec_escargot {
            "B@.o....s..#####"
        } else {
            "B@.o.......#####"
        };
        let lignes = [
            "................",
            "................",
            "................",
            "...........o.o.X",
            "...........#####",
            "...........#####",
            "...........#####",
            "...........#####",
            sol,
            "################",
        ];
        Niveau::depuis_ascii(&lignes.join("\n")).expect("niveau de test")
    }

    /// Le robot monte sur la falaise en rebondissant sur l'escargot (témoin : sans
    /// lui, la falaise est hors de portée). Robot maladroit compris.
    #[test]
    fn le_robot_rebondit_sur_l_escargot_pour_monter() {
        let temoin = niveau_escargot(false);
        let sans = analyser(&temoin, temoin.depart, Options::default());
        assert!(!sans.arrivee, "la falaise s'atteint sans escargot ?");
        let n = niveau_escargot(true);
        let avec = analyser(&n, n.depart, Options::default());
        assert!(avec.arrivee, "rebond non trouvé : {avec:?}");
        assert_eq!(avec.gouttes, [true; 3]);
        let maladroit = analyser(
            &n,
            n.depart,
            Options {
                precis: false,
                ..Options::default()
            },
        );
        assert!(maladroit.arrivee, "robot maladroit : {maladroit:?}");
    }

    /// Le tremplin du robot couvre tout le trajet de l'escargot : du mur (x = 16)
    /// jusqu'au pied de la falaise (x = 176), au ras du sol (y = 134).
    #[test]
    fn le_robot_voit_l_escargot_sur_tout_son_trajet() {
        use crate::entities::escargot::HAUTEUR_ESCARGOT;
        let n = niveau_escargot(true);
        let s = n
            .entites
            .iter()
            .find_map(|e| match e {
                Entite::Escargot(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap();
        let env = Environnement::simple(&n);
        let b = s.etendu_au_trajet(&env).boite;
        assert!((b.x - 16.0).abs() < 0.01, "{b:?}");
        assert!((b.droite() - 176.0).abs() < 0.01, "{b:?}");
        assert_eq!((b.y, b.h), (144.0 - HAUTEUR_ESCARGOT, HAUTEUR_ESCARGOT));
    }

    /// Mini-niveau du chat, 16 x 5 : une niche d'une tuile de haut (colonnes 6 à
    /// 11, sous un toit de nuages, fermée à droite par un mur de briques) cache
    /// une goutte ; un chat endormi (`z`, colonne 5) dort devant son entrée. Le
    /// drapeau est derrière le mur (on passe par-dessus le toit). Une goutte
    /// avant le chat, une dans la niche, une derrière le mur. `avec_chat` faux :
    /// la case du chat reste vide ; `dans_la_niche` : Pépin part du fond de la
    /// niche (colonne 10) au lieu de la colonne 1.
    fn niveau_chat(avec_chat: bool, dans_la_niche: bool) -> Niveau {
        let mut rangee: Vec<char> = "...o.z...o..Bo.X".chars().collect();
        if !avec_chat {
            rangee[5] = '.';
        }
        rangee[if dans_la_niche { 10 } else { 1 }] = '@';
        let rangee: String = rangee.into_iter().collect();
        let lignes = [
            "................",
            "................",
            "......NNNNNNB...",
            &rangee,
            "################",
        ];
        Niveau::depuis_ascii(&lignes.join("\n")).expect("niveau de test")
    }

    /// Le robot prend la goutte cachée : il saute sur le chat (endormi), l'écrase,
    /// entre dans la niche puis en ressort (le chat écrasé le reste), et finit.
    #[test]
    fn le_robot_ecrase_le_chat_pour_prendre_la_goutte_cachee() {
        let n = niveau_chat(true, false);
        let r = analyser(&n, n.depart, Options::default());
        assert!(r.arrivee, "drapeau non atteint : {r:?}");
        assert_eq!(r.gouttes, [true; 3], "{r:?}");
        let maladroit = analyser(
            &n,
            n.depart,
            Options {
                precis: false,
                ..Options::default()
            },
        );
        assert!(maladroit.arrivee, "robot maladroit : {maladroit:?}");
        assert_eq!(
            maladroit.gouttes, [true; 3],
            "robot maladroit : {maladroit:?}"
        );
    }

    /// Le robot ne traverse pas un chat vivant : parti du fond de la niche, il
    /// marche vers la sortie et heurte le chat de côté (mort). Le même plan, chat
    /// déjà écrasé (bit de `Noeud::chats`), le fait sortir ; sans chat aussi.
    #[test]
    fn marcher_dans_un_chat_vivant_tue_le_robot() {
        let sortir = Plan {
            dir: Direction::Gauche,
            elan: 120,
            saut: 0,
            planer: false,
            dir_montee: true,
            dir_descente: true,
        };
        let essayer = |n: &Niveau, ecrase: bool| {
            let solides = solides_des_entites(&n.entites);
            let env = Environnement {
                grille: n,
                solides: &solides,
                courants: &[],
                vents: &[],
            };
            let mut ents = n.entites.clone();
            for e in &mut ents {
                if let Entite::Ennemi(c) = e {
                    if ecrase {
                        c.ecraser();
                    }
                }
            }
            let depart = Noeud::pose(n.depart.0, n.depart.1, 0);
            let mut touches = Vec::new();
            let (fin, _) = simuler(n, &env, depart, sortir, &mut ents, &mut touches);
            (fin, touches)
        };
        let (fin, _) = essayer(&niveau_chat(true, true), false);
        assert!(matches!(fin, Fin::Mort), "le chat vivant est traversé");
        let (fin, touches) = essayer(&niveau_chat(true, true), true);
        assert!(matches!(fin, Fin::Pose(..)), "chat écrasé : il sort");
        assert!(touches.contains(&Contact::Goutte(0)));
        let (fin, _) = essayer(&niveau_chat(false, true), false);
        assert!(matches!(fin, Fin::Pose(..)), "sans chat : il sort");
    }

    /// Enfermé au fond de la niche, le robot en sort quand même : une petite
    /// détente contre le plafond le fait retomber sur le dos du chat (il l'écrase
    /// par le dessus, comme le ferait un joueur). Le vrai jeu aussi : le chat
    /// garde la niche, il ne la mure pas.
    #[test]
    fn du_fond_de_la_niche_le_robot_ecrase_le_chat_pour_sortir() {
        let n = niveau_chat(true, true);
        let r = analyser(&n, n.depart, Options::default());
        assert!(r.arrivee && r.gouttes == [true; 3], "{r:?}");
    }

    // --- Aiguilles d'horloge (carte 5.10) ---

    /// Mini-niveau de l'horloge, 28 x 20 : un balcon (lignes 2-3, colonnes 0 à 4,
    /// dessus à y = 48) porte le drapeau ; une grande aiguille (pivot en colonne 14,
    /// ligne 9, rayon 8 tuiles) tourne à côté. Depuis le sol on saute sur ses
    /// places basses (6 h, 7 h, 8 h), mais de 8 h à 9 h il y a 64 px à monter :
    /// plus qu'un saut. Seul le trajet SUR l'aiguille mène au-dessus (10 h, d'où
    /// le balcon est à portée). Sans aiguille (`h` vidé), le balcon est hors d'atteinte.
    fn niveau_horloge(avec_aiguille: bool) -> Niveau {
        let texte = lignes_horloge().join("\n");
        if avec_aiguille {
            let reglage = crate::level::ReglageAiguille {
                rayon: 8.0,
                vitesse: 40.0,
                phase: 0.0,
            };
            Niveau::depuis_ascii_avec_aiguilles(&texte, &[reglage]).expect("niveau de test")
        } else {
            Niveau::depuis_ascii(&texte.replace('h', ".")).expect("niveau de test")
        }
    }

    /// Le robot monte au balcon en se laissant porter par l'aiguille (robot précis
    /// et maladroit) ; sans aiguille, il n'y arrive pas.
    #[test]
    fn le_robot_monte_au_balcon_par_l_aiguille() {
        let n = niveau_horloge(true);
        let r = analyser(&n, n.depart, Options::default());
        assert!(r.arrivee, "drapeau non atteint : {r:?}");
        assert_eq!(r.gouttes, [true; 3]);
        let maladroit = analyser(
            &n,
            n.depart,
            Options {
                precis: false,
                ..Options::default()
            },
        );
        assert!(maladroit.arrivee, "robot maladroit : {maladroit:?}");
        let sans = niveau_horloge(false);
        let r = analyser(&sans, sans.depart, Options::default());
        assert!(
            !r.arrivee,
            "sans aiguille, le balcon doit rester hors de portée"
        );
    }

    /// Le trajet sur l'aiguille : de 6 h à midi, un demi-tour d'attente, et Pépin arrive décalé comme la place (même endroit de la
    /// plateforme). Une tuile sur le chemin de sa tête : trajet refusé.
    #[test]
    fn le_trajet_sur_l_aiguille_suit_le_cadran() {
        let n = niveau_horloge(true);
        let m = n
            .entites
            .iter()
            .find_map(|e| match e {
                Entite::Mobile(m) => Some(m.clone()),
                _ => None,
            })
            .unwrap();
        let places = std::array::from_fn(|h| m.coin_a(h as f32 / 12.0).unwrap());
        let cadran = Cadran {
            aiguille: m.clone(),
            places,
        };
        // Midi au-dessus du pivot (232, 152), 6 h en dessous : 256 px plus bas.
        assert_eq!(places[0], (216.0, 20.0));
        assert_eq!(places[6], (216.0, 276.0));
        let debout = (226.0, 276.0 - JOUEUR_H);
        let ((x, y), attente) = cadran.trajet(&n, debout, 6, 6).unwrap();
        assert_eq!((x, y), (226.0, 20.0 - JOUEUR_H));
        let demi_tour = m.cercle().unwrap().pas_par_tour() / 2.0;
        assert_eq!(attente, demi_tour.ceil() as u32);
        // Un toit à 9 h, là où passe la tête de Pépin : refusé (dans l'autre
        // sens, de 6 h à 8 h, rien ne gêne).
        let mut lignes = lignes_horloge();
        lignes[8].replace_range(6..7, "#");
        let bloque = Niveau::depuis_ascii(&lignes.join("\n").replace('h', ".")).unwrap();
        assert!(cadran.trajet(&bloque, debout, 6, 6).is_none());
        assert!(cadran.trajet(&bloque, debout, 6, 2).is_some());
    }

    /// Les lignes du mini-niveau de l'horloge (voir `niveau_horloge`).
    fn lignes_horloge() -> Vec<String> {
        let mut lignes = vec!["............................".to_string(); 20];
        lignes[2] = ".X..........................".into();
        lignes[3] = "#####.......................".into();
        lignes[9] = "..............h.............".into();
        lignes[18] = ".@.o.o.o....................".into();
        lignes[19] = "############################".into();
        lignes
    }
}
