//! Le « cerveau » du robot de QA : les plans et la recherche de plus court
//! chemin qui fait finir chaque niveau sans mourir.
//!
//! Principe : Dijkstra (sur la durée, avec `BinaryHeap` + `Reverse` : le tas de
//! Rust sort le PLUS GRAND d'abord) entre des états « posés et immobiles » —
//! ou « lancés » (un auvent vient de relancer Pépin, il choisit de nouveau en
//! l'air, voir `est_lance`). Chaque plan (courir, sauter, planer, attendre) est
//! simulé sur une COPIE du monde (`Clone` d'une structure complète pour explorer
//! des futurs possibles sans toucher au présent) : un plan qui mène à la mort
//! est simplement écarté, donc le parcours trouvé est sûr par construction.
//!
//! Les états s'apparentent aux « cases » du niveau : deux états proches
//! fusionnent en la même case (`CaseParcours`, via `cle`) — la mémoire de la
//! recherche est une `HashMap` (`chercher`).
//!
//! ## Ce qui va où
//!
//! - Les plans et leur simulation : `Plan`, `Fin`, `plans`, `plans_en_vol`,
//!   `simuler` (+ `est_lance`, `est_mort`, `chat_ecrase_de_justesse`).
//! - La tenue des parcours quand le niveau bouge (`aiguilles_decalees`,
//!   `tient_au_decalage` : une aiguille d'horloge ou le Grand Nuage rejoués
//!   avec une ou deux images d'avance ou de retard doivent donner la même fin).
//! - Les cases : `CaseParcours`, `cle`, et les résumés par mécanique
//!   (`nuage_de_la_case`, `aiguille_proche`, `aiguille_sous_pepin`,
//!   `chats_de_la_case`, `escargot_proche`) : une mécanique qui a de l'état
//!   (phase, temps) se résume en un ENTIER — 0 dans un niveau sans la
//!   mécanique, pour ne pas changer la case ailleurs.
//! - La recherche : `Etape`, `depart_pose`, `chercher`, `trouver_parcours`.
//!
//! ## Ajouter une nouvelle mécanique
//!
//! 1. Si la mécanique a un état dont le robot doit tenir compte (une phase, un
//!    temps), écrire sa fonction de résumé `X_de_la_case(m) -> i32` (0 sans la
//!    mécanique) et ajouter un champ à `CaseParcours` + `cle`. La simulation
//!    rejette déjà toute plan qui meurt ; si la mécanique impose d'attendre
//!    (un courant qui souffle à heure fixe), le temps doit être dans la case,
//!    sinon le plan « attendre » aboutit dans une case déjà connue et la
//!    recherche l'abandonne.
//! 2. Si la mécanique change la FIN (un dernier coup qui termine le niveau),
//!    l'ajouter à `chercher` (ex. `gouttes_avant_la_victoire` pour le Grand
//!    Nuage). Si elle a un horloge, la faire tenir à `tient_au_decalage`
//!    (même nombre de coups, même horloge du nuage, quel que soit le décalage).
//! 3. Un test dans `essais_tests.rs` (le robot finit le niveau d'essai ; le
//!    parcours tient à une image près) et un script dans `scripts.rs`
//!    (`ecrire_script_*`, pour les captures du navigateur).
//!
//! Notions Rust illustrées ici : `BinaryHeap` + `Reverse` (file de priorité
//! « plus petit d'abord »), `Clone` d'une structure complète, `HashMap` comme
//! mémoire, et un `struct` de champs nommés (`CaseParcours`) : un champ s'ajoute
//! avec un nom et un commentaire, sans compter les éléments d'un septuor.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use pepin_core::entities::contacts::Contact;
use pepin_core::entities::ennemi::{
    pas_du_cycle, EtatEnnemi, ANIMATION_ECRASE_PAS, DORT_PAS, REVEIL_PAS,
};
use pepin_core::entities::nuage::PhaseNuage;
use pepin_core::entities::{Entite, TypeEnnemi};
use pepin_core::faisabilite::pas_max_plan;
use pepin_core::player::{Direction, Entrees};

use crate::monde::Monde;

/// Pas d'immobilité qui valident un atterrissage stable.
const PAS_STABLE: u16 = 20;
/// Pas d'attente au départ : Pépin tombe au sol et se pose.
pub(crate) const PAS_DEPART: u16 = 40;
/// Pas passés en l'air avant que le plan `planer` n'ouvre le parapluie.
const DEBUT_PLANE: u16 = 7;

/// Un plan d'entrées, simulé d'un seul bloc. `pub(crate)` : la mesure de
/// difficulté rejoue les mêmes plans, avec des erreurs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Plan {
    pub(crate) dir: Direction,
    /// Pas de course au sol avant le saut.
    pub(crate) elan: u16,
    /// Pas de maintien du saut (0 = pas de saut, on marche dans le vide).
    pub(crate) saut: u16,
    pub(crate) planer: bool,
    /// Pas de chute avant d'ouvrir le parapluie (si `planer`). Toujours 0 pour le
    /// robot (il ouvre dès le sommet du saut) ; la mesure de difficulté
    /// (`difficulte.rs`) l'allonge pour imiter un « plané tardif ».
    pub(crate) retard_plane: u16,
    pub(crate) dir_montee: bool,
    pub(crate) dir_descente: bool,
    /// Plan « attendre » : aucune entrée pendant ce nombre de pas (0 = non).
    pub(crate) attente: u16,
}

/// Comment se termine la simulation d'un plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Fin {
    Mort,
    Arrivee,
    Pose,
    /// Un auvent vient de relancer Pépin : nouveau point de décision, en l'air
    /// (voir `est_lance`).
    Lance,
    Abandon,
}

/// Tous les plans à essayer. `precis` ajoute des élans et des durées plus fins.
pub(crate) fn plans(precis: bool) -> Vec<Plan> {
    let (elans, sauts): (&[u16], &[u16]) = if precis {
        (&[0, 4, 8, 14, 22, 32, 50, 80, 120], &[0, 5, 9, 13, 40])
    } else {
        (&[0, 8, 22, 50, 120], &[0, 13, 40])
    };
    let mut v = Vec::new();
    for dir in [Direction::Gauche, Direction::Droite] {
        for &elan in elans {
            for &saut in sauts {
                for planer in [false, true] {
                    for dir_montee in [true, false] {
                        for dir_descente in [true, false] {
                            if saut == 0 && !dir_montee {
                                continue; // sans saut, « montée » n'a pas de sens
                            }
                            v.push(Plan {
                                dir,
                                elan,
                                saut,
                                planer,
                                retard_plane: 0,
                                dir_montee,
                                dir_descente,
                                attente: 0,
                            });
                        }
                    }
                }
            }
        }
    }
    // Attendre : laisse passer une plateforme mobile, un ennemi ou un éclair.
    for attente in [20, 45, 90, 180] {
        v.push(Plan {
            dir: Direction::Immobile,
            elan: 0,
            saut: 0,
            planer: false,
            retard_plane: 0,
            dir_montee: false,
            dir_descente: false,
            attente,
        });
    }
    v
}

/// Les plans « en vol », essayés juste après un rebond d'auvent (`Fin::Lance`) :
/// une direction (gauche, droite, ou aucune) gardée pendant la montée et/ou la
/// descente, parapluie ouvert ou non. Ni élan au sol, ni saut, ni attente : on
/// est en l'air. Même liste que le robot du cœur (`faisabilite::plans_en_vol`).
pub(crate) fn plans_en_vol() -> Vec<Plan> {
    let mut v = Vec::new();
    for planer in [false, true] {
        for dir in [Direction::Gauche, Direction::Droite] {
            for dir_montee in [true, false] {
                for dir_descente in [true, false] {
                    v.push(Plan {
                        dir,
                        elan: 0,
                        saut: 0,
                        planer,
                        retard_plane: 0,
                        dir_montee,
                        dir_descente,
                        attente: 0,
                    });
                }
            }
        }
        v.push(Plan {
            dir: Direction::Immobile,
            elan: 0,
            saut: 0,
            planer,
            retard_plane: 0,
            dir_montee: false,
            dir_descente: false,
            attente: 0,
        });
    }
    v
}

/// Pépin est-il en vol, lancé par un auvent (élan d'auvent pas encore éteint) ?
/// Juste après un rebond (`Fin::Lance`), c'est un point de décision en l'air :
/// on y essaie les `plans_en_vol`, pas les plans du sol.
pub(crate) fn est_lance(m: &Monde) -> bool {
    m.joueur.elan_auvent() > 0.0 && !m.joueur.au_sol
}

/// Pépin est-il mort (ou en train de réapparaître) ? `derniere_mort` n'est jamais
/// effacée par le jeu : après une mort, l'appelant doit la remettre à `None`
/// avant de simuler de nouveaux plans (voir `difficulte.rs`).
pub(crate) fn est_mort(m: &Monde) -> bool {
    m.reapparition > 0 || m.derniere_mort.is_some()
}

/// Marge (pas) que le robot garde autour de l'éveil d'un chat : 6 pas (0,1 s).
pub(crate) const MARGE_CHAT_PAS: u16 = 6;

/// Pépin vient-il d'écraser un chat DE JUSTESSE, à moins de `MARGE_CHAT_PAS` de
/// son éveil (juste avant qu'il ne se réveille, ou juste après qu'il s'est
/// rendormi) ? Le jeu l'accepte, mais le robot le compte comme une mort : un plan
/// réglé au pas près ne laisse aucune marge à un vrai joueur, et le moindre
/// décalage d'une image (rejeu dans le navigateur) le ferait mourir.
///
/// Un chat écrasé ne change plus de phase : on lit celle du moment du contact,
/// au pas même de l'écrasement (son aplatissement vient de commencer).
pub(crate) fn chat_ecrase_de_justesse(m: &Monde) -> bool {
    let eveil = DORT_PAS + REVEIL_PAS;
    m.entites.iter().any(|e| match e {
        Entite::Ennemi(c) if c.type_ennemi == TypeEnnemi::Chat => {
            let p = pas_du_cycle(c.phase_chat());
            c.etat == EtatEnnemi::Ecrase(ANIMATION_ECRASE_PAS)
                && (p + MARGE_CHAT_PAS >= eveil || p < MARGE_CHAT_PAS)
        }
        _ => false,
    })
}

/// Simule `plan` depuis `depart` (copie). Renvoie la fin, le monde final et les
/// entrées de chaque pas. Un chat écrasé de justesse compte comme une mort
/// (`chat_ecrase_de_justesse`).
pub(crate) fn simuler(depart: &Monde, plan: Plan) -> (Fin, Monde, Vec<Entrees>) {
    let mut m = depart.clone();
    let mut script = Vec::new();

    if plan.attente > 0 {
        for _ in 0..plan.attente {
            m.avancer(&Entrees::default());
            script.push(Entrees::default());
            if est_mort(&m) || chat_ecrase_de_justesse(&m) {
                return (Fin::Mort, m, script);
            }
            if m.termine {
                return (Fin::Arrivee, m, script);
            }
        }
        let fin = if m.joueur.au_sol {
            Fin::Pose
        } else {
            Fin::Abandon
        };
        return (fin, m, script);
    }

    let mut en_l_air: u16 = 0;
    // Lancé par un auvent : déjà en l'air, les commandes « en vol » s'appliquent
    // dès le premier pas. (Toujours faux sans auvent : rien ne change ailleurs.)
    let mut a_decolle = est_lance(&m);
    let mut stable: u16 = 0;
    let mut saut_envoye: u16 = 0;
    let mut chute: u16 = 0;
    // Durée maximale d'un plan : 10 s dans un niveau d'un écran, le temps d'un plané sur
    // toute la hauteur dans un niveau haut (même règle que le robot du cœur).
    for pas in 0..pas_max_plan(m.niveau.hauteur_px()) {
        let mut e = Entrees::default();
        let marche_finie = plan.saut == 0 && pas >= plan.elan;
        if !a_decolle && stable == 0 && !marche_finie {
            e.direction = plan.dir;
            if pas >= plan.elan {
                e.saut = true;
                saut_envoye += 1;
            }
        } else if a_decolle && stable == 0 {
            let montee = m.joueur.vitesse.y < 0.0;
            if !montee {
                chute += 1;
            }
            let garde = if montee {
                plan.dir_montee
            } else {
                plan.dir_descente
            };
            e.direction = if garde { plan.dir } else { Direction::Immobile };
            if saut_envoye > 0 && saut_envoye < plan.saut && montee {
                e.saut = true;
                saut_envoye += 1;
            } else if plan.planer && !montee && en_l_air >= DEBUT_PLANE && chute > plan.retard_plane
            {
                e.saut = true;
            }
        }
        m.avancer(&e);
        script.push(e);
        if est_mort(&m) || chat_ecrase_de_justesse(&m) {
            return (Fin::Mort, m, script);
        }
        if m.termine {
            return (Fin::Arrivee, m, script);
        }
        if m.contacts.contains(&Contact::Auvent) {
            return (Fin::Lance, m, script);
        }
        if !m.joueur.au_sol {
            a_decolle = true;
            en_l_air += 1;
            stable = 0;
        } else if a_decolle || marche_finie {
            stable += 1;
            if stable >= PAS_STABLE {
                return (Fin::Pose, m, script);
            }
        }
    }
    (Fin::Abandon, m, script)
}

/// Le monde `m` où chaque aiguille d'horloge a `pas` pas d'avance sur Pépin (de
/// retard si `pas` est négatif) : `PlateformeMobile::tourner`. Pépin debout sur
/// une aiguille la suit (pieds posés pile sur son dessus), comme s'il avait été
/// porté. Le Grand Nuage aussi a `pas` pas d'avance ou de retard
/// (`GrandNuage::decaler`) : ses éclairs et son essoufflement arrivent plus tôt ou
/// plus tard.
pub(crate) fn aiguilles_decalees(m: &Monde, pas: i32) -> Monde {
    let sous = aiguille_sous_pepin(m);
    let mut d = m.clone();
    let mut k = 0;
    for e in &mut d.entites {
        if let Entite::Nuage(g) = e {
            g.decaler(pas);
            continue;
        }
        let Entite::Mobile(a) = e else {
            continue;
        };
        if a.cercle().is_none() {
            continue;
        }
        let avant = a.boite;
        a.tourner(pas);
        if sous == Some(k) {
            d.joueur.boite.x += a.boite.x - avant.x;
            d.joueur.boite.y = a.boite.y - d.joueur.boite.h;
        }
        k += 1;
    }
    d
}

/// Écart toléré (px, en x comme en y) entre l'arrivée d'un plan et celle du même
/// plan, aiguilles décalées.
const ECART_DECALAGE: f32 = 4.0;

/// Décalages (en images) des aiguilles essayés par `tient_au_decalage` : une et
/// deux images d'avance ou de retard. Deux : de la marge, car les petits écarts
/// d'un plan à l'autre s'ajoutent tout au long du parcours.
const DECALAGES_AIGUILLE: [i32; 4] = [-2, -1, 1, 2];

/// Marge (px) que Pépin garde de chaque côté de la plaque quand un plan le pose
/// sur une aiguille : jamais posé sur le bord, d'où un saut de travers le ferait
/// tomber au moindre écart.
const MARGE_PLAQUE: f32 = 3.0;

/// Le plan `plan`, joué depuis `depart` jusqu'à `fin` / `arrivee`, tient-il si les
/// aiguilles d'horloge (et le Grand Nuage) ont une ou deux images d'avance ou de
/// retard (`DECALAGES_AIGUILLE`) ? Il faut la même fin (jamais une mort), les mêmes
/// gouttes, et Pépin à moins de `ECART_DECALAGE` px du même endroit ; posé sur une
/// aiguille, il doit avoir `MARGE_PLAQUE` px de plaque de chaque côté. Toujours
/// vrai dans un niveau sans aiguille ni Grand Nuage.
///
/// Le Grand Nuage (carte 5.11) : un éclair évité d'une image, un saut sur sa tête au
/// tout dernier pas de son essoufflement ne laissent aucune marge non plus. Avec le
/// nuage décalé, le même plan doit aussi donner le même nombre de coups
/// (`pv_nuage`). Et un plan qui le frappe doit le frapper au MÊME pas du plan, quel
/// que soit le décalage (même phase, même temps écoulé dans la phase à la fin du
/// plan : `horloge_du_nuage`) : un coup donne le départ du combat suivant (touché,
/// puis le motif suivant), et un coup qui dépendrait de l'heure exacte du nuage (un
/// saut lancé avant que son courant ne souffle) décalerait tout le reste du
/// parcours. Le premier parcours trouvé, sans cette règle, frappait le nuage une
/// image plus tard une fois rejoué avec deux images d'avance, et mourait ensuite.
///
/// Pourquoi : une aiguille tourne sans arrêt, et un plan réglé à l'image près (un
/// saut qui touche le bord de la plaque, un atterrissage au tout dernier moment)
/// ne laisse aucune marge à un vrai joueur. Le rejeu dans le navigateur, qui peut
/// démarrer le niveau d'essai une image plus tôt ou plus tard (voir QA.md), le
/// raterait aussi : c'est arrivé au premier parcours de la carte 5.10.
pub(crate) fn tient_au_decalage(depart: &Monde, plan: Plan, fin: Fin, arrivee: &Monde) -> bool {
    let a_des_aiguilles = depart.entites.iter().any(|e| match e {
        Entite::Mobile(a) => a.cercle().is_some(),
        Entite::Nuage(_) => true,
        _ => false,
    });
    if !a_des_aiguilles {
        return true;
    }
    let coup = arrivee.pv_nuage() != depart.pv_nuage();
    if let Some(k) = aiguille_sous_pepin(arrivee) {
        let plaque = arrivee
            .entites
            .iter()
            .filter_map(|e| match e {
                Entite::Mobile(a) if a.cercle().is_some() => Some(a.boite),
                _ => None,
            })
            .nth(k);
        let j = &arrivee.joueur.boite;
        if plaque
            .is_some_and(|p| j.x < p.x + MARGE_PLAQUE || j.droite() > p.droite() - MARGE_PLAQUE)
        {
            return false;
        }
    }
    DECALAGES_AIGUILLE.into_iter().all(|pas| {
        let (fin_d, m_d, _) = simuler(&aiguilles_decalees(depart, pas), plan);
        let (a, b) = (&m_d.joueur.boite, &arrivee.joueur.boite);
        fin_d == fin
            && m_d.gouttes_niveau == arrivee.gouttes_niveau
            && m_d.pv_nuage() == arrivee.pv_nuage()
            && (!coup || horloge_du_nuage(&m_d) == horloge_du_nuage(arrivee))
            && (fin == Fin::Arrivee
                || ((a.x - b.x).abs() <= ECART_DECALAGE && (a.y - b.y).abs() <= ECART_DECALAGE))
    })
}

/// Une arrivée compte-t-elle ? Toujours, sauf dans l'arène du Grand Nuage : là, le
/// dernier coup finit le niveau, et le robot doit avoir pris toutes les gouttes
/// avant (comme un joueur qui veut les trois gouttes du niveau).
pub(crate) fn gouttes_avant_la_victoire(m: &Monde) -> bool {
    m.pv_nuage().is_none() || m.gouttes_niveau.iter().all(|&g| g)
}

/// La phase du Grand Nuage et le temps passé dans cette phase (`None` sans nuage).
fn horloge_du_nuage(m: &Monde) -> Option<(PhaseNuage, u16)> {
    m.entites.iter().find_map(|e| match e {
        Entite::Nuage(g) => Some((g.phase(), g.ecoule())),
        _ => None,
    })
}

/// Une case du niveau où Pépin choisit son prochain plan : case de 8 px en x, y
/// arrondi, « lancé par un auvent » (`est_lance`) ou posé, le Grand Nuage
/// (`nuage_de_la_case`, 0 sans nuage), l'escargot proche
/// (`escargot_proche`, 0 s'il n'y en a pas), les chats (`chats_de_la_case`, 0
/// dans un niveau sans chat) et l'heure de l'aiguille d'horloge proche
/// (`aiguille_proche`, 0 s'il n'y en a pas). Deux états proches fusionnent.
///
/// Un `struct` de champs nommés (et non un 7-uple) : chaque champ se lit seul,
/// et une mécanique nouvelle s'ajoute avec un nom et un commentaire, sans
/// compter les éléments d'un 7-uple. `Copy` + `Eq` + `Hash` : une case sert de
/// clé dans les `HashMap` / `HashSet` de la recherche (voir `chercher` et
/// `difficulte`), et `Ord` (lexicographique, dans l'ordre des champs — comme
/// l'ancien 7-uple) : la mesure de difficulté trie ses cases dans un tas, et
/// deux mesures comparables exigent le MÊME ordre.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct CaseParcours {
    /// Case de 8 px en x (position de Pépin).
    pub(crate) x: i32,
    /// Y de Pépin, arrondi.
    pub(crate) y: i32,
    /// Lancé par un auvent : on n'essaie que des plans « en vol ».
    pub(crate) lance: bool,
    /// Escargot proche, résumé en entier (`escargot_proche`) ; 0 s'il n'y en a pas.
    pub(crate) escargot: i32,
    /// Chats du niveau, résumés en entier (`chats_de_la_case`) ; 0 sans chat.
    pub(crate) chats: i32,
    /// Heure de l'aiguille d'horloge proche (`aiguille_proche`) ; 0 s'il n'y en a pas.
    pub(crate) aiguille: i32,
    /// Le Grand Nuage, résumé en entier (`nuage_de_la_case`) ; 0 sans nuage.
    pub(crate) nuage: i32,
}

/// La case (`CaseParcours`) de Pépin dans `m`.
pub(crate) fn cle(m: &Monde) -> CaseParcours {
    CaseParcours {
        x: (m.joueur.boite.x / 8.0).floor() as i32,
        y: m.joueur.boite.y.round() as i32,
        lance: est_lance(m),
        escargot: escargot_proche(m),
        chats: chats_de_la_case(m),
        aiguille: aiguille_proche(m),
        nuage: nuage_de_la_case(m),
    }
}

/// Tranche du temps du Grand Nuage (pas) retenue dans la case : 15 pas (1/4 s).
const TRANCHE_NUAGE_PAS: u16 = 15;

/// Le Grand Nuage, résumé en un entier : `1 + tranche + 64 × (phase + 8 × points de
/// vie) + 2048 × gouttes`, où `tranche` est le temps passé dans sa phase, par
/// tranches de `TRANCHE_NUAGE_PAS` (au plus 63), et `gouttes` un bit par goutte d'or
/// déjà prise dans le niveau ; 0 sans nuage (la case est alors exactement celle
/// d'avant la carte 5.11).
///
/// Pourquoi : comme le chat, le nuage s'ATTEND. Sans son temps dans la case, attendre
/// à l'abri d'un éclair ramènerait à une case déjà connue et le plan « attendre »
/// serait jeté : le robot ne pourrait ni laisser passer une salve, ni attendre
/// l'essoufflement. Ses points de vie et sa phase séparent les étapes du combat. Les
/// gouttes : le dernier coup finit le niveau, le robot doit donc les prendre AVANT
/// (`gouttes_avant_la_victoire`) ; sans elles dans la case, le même endroit avec et
/// sans une goutte fusionnerait.
pub(crate) fn nuage_de_la_case(m: &Monde) -> i32 {
    m.entites
        .iter()
        .find_map(|e| match e {
            Entite::Nuage(g) => {
                let phase = match g.phase() {
                    PhaseNuage::Prelude => 0,
                    PhaseNuage::Colere => 1,
                    PhaseNuage::Essouffle => 2,
                    PhaseNuage::Touche => 3,
                    PhaseNuage::Dechirure => 4,
                    PhaseNuage::Vaincu => 5,
                };
                let tranche = i32::from(g.ecoule() / TRANCHE_NUAGE_PAS).min(63);
                let gouttes = m
                    .gouttes_niveau
                    .iter()
                    .enumerate()
                    .fold(0, |b, (k, &g)| b | (i32::from(g) << k));
                Some(1 + tranche + 64 * (phase + 8 * i32::from(g.pv())) + 2048 * gouttes)
            }
            _ => None,
        })
        .unwrap_or(0)
}

/// Portée (px) à laquelle une aiguille d'horloge compte dans la case de Pépin :
/// son rayon plus 6 tuiles, en x comme en y, entre le centre de Pépin et le pivot.
const MARGE_AIGUILLE: f32 = 96.0;

/// Nombre de tranches d'un tour d'aiguille retenues dans la case : une par heure.
const TRANCHES_AIGUILLE: f32 = 12.0;

/// L'aiguille d'horloge la plus proche de Pépin (pivot à moins de son rayon plus
/// `MARGE_AIGUILLE`), résumée en un entier : `1 + l'heure où elle se trouve`
/// (de 1 à 12) ; 0 si aucune n'est proche (toujours 0 dans un niveau sans
/// aiguille : la case est alors exactement celle d'avant la carte 5.10).
///
/// Pourquoi : comme l'escargot et le chat, l'aiguille se rattrape au bon MOMENT.
/// Sans son heure dans la case, attendre au pied de l'aiguille ramène à une case
/// déjà connue et le plan « attendre » est jeté : le robot ne pourrait jamais la
/// laisser descendre jusqu'à lui. Posé sur l'aiguille, sa position suffirait ;
/// l'heure ne gêne pas.
pub(crate) fn aiguille_proche(m: &Monde) -> i32 {
    let j = &m.joueur.boite;
    let (px, py) = (j.x + j.w / 2.0, j.y + j.h / 2.0);
    m.entites
        .iter()
        .filter_map(|e| match e {
            Entite::Mobile(a) => a.cercle(),
            _ => None,
        })
        .filter_map(|c| {
            let (dx, dy) = ((c.pivot.0 - px).abs(), (c.pivot.1 - py).abs());
            let portee = c.rayon + MARGE_AIGUILLE;
            (dx <= portee && dy <= portee).then_some((dx * dx + dy * dy, c.tour()))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map_or(0, |(_, tour)| 1 + (tour * TRANCHES_AIGUILLE) as i32)
}

/// Pépin est-il debout sur une aiguille d'horloge ? Son indice parmi les aiguilles
/// du niveau (ordre des entités), ou `None`.
pub(crate) fn aiguille_sous_pepin(m: &Monde) -> Option<usize> {
    if !m.joueur.au_sol {
        return None;
    }
    let j = &m.joueur.boite;
    m.entites
        .iter()
        .filter_map(|e| match e {
            Entite::Mobile(a) if a.cercle().is_some() => Some(a.boite),
            _ => None,
        })
        .position(|b| (j.bas() - b.y).abs() < 0.01 && j.x < b.droite() && b.x < j.droite())
}

/// Portée (px) à laquelle un chat compte dans la case de Pépin : la même que
/// celle de l'escargot (6 tuiles en x entre les centres, 4 en y entre les pieds).
const PORTEE_CHAT: (f32, f32) = PORTEE_ESCARGOT;

/// Tranche du cycle du chat (pas) retenue dans la case : 30 pas (0,5 s), donc
/// 12 tranches pour un cycle de 6 s.
const TRANCHE_CHAT_PAS: u16 = 30;

/// Les chats du niveau, résumés en un entier : `16 × écrasés + tranche`.
/// - `écrasés` : un bit par chat du niveau (dans l'ordre des entités), à 1 s'il
///   est écrasé ;
/// - `tranche` : où en est le chat vivant le plus proche (dans `PORTEE_CHAT`)
///   dans son cycle, `1 + pas_du_cycle / TRANCHE_CHAT_PAS` (de 1 à 12) ; 0 si
///   aucun chat vivant n'est proche.
///
/// Pourquoi : le chat garde un passage, il faut ATTENDRE qu'il dorme. Sans le
/// moment de son cycle dans la case, attendre sur place ramène à une case déjà
/// connue et le plan « attendre » est jeté : le robot ne pourrait jamais laisser
/// le chat se rendormir. Et sans les chats écrasés, un état « chat écrasé »
/// fusionnerait avec le même endroit « chat vivant ». Toujours 0 dans un niveau
/// sans chat : la case est alors exactement celle d'avant la carte 5.9.
pub(crate) fn chats_de_la_case(m: &Monde) -> i32 {
    let j = &m.joueur.boite;
    let centre = j.x + j.w / 2.0;
    let mut ecrases = 0i32;
    let mut proche: Option<(f32, u16)> = None;
    let chats = m.entites.iter().filter_map(|e| match e {
        Entite::Ennemi(c) if c.type_ennemi == TypeEnnemi::Chat => Some(c),
        _ => None,
    });
    for (k, c) in chats.enumerate() {
        assert!(k < 26, "trop de chats pour la case du robot");
        if !c.est_dangereux() {
            ecrases |= 1 << k;
            continue;
        }
        let dx = (c.boite.x + c.boite.w / 2.0 - centre).abs();
        let dy = (c.boite.bas() - j.bas()).abs();
        if dx <= PORTEE_CHAT.0 && dy <= PORTEE_CHAT.1 && proche.is_none_or(|(d, _)| dx < d) {
            proche = Some((dx, pas_du_cycle(c.phase_chat())));
        }
    }
    let tranche = proche.map_or(0, |(_, p)| 1 + i32::from(p / TRANCHE_CHAT_PAS));
    16 * ecrases + tranche
}

/// Portée (px) à laquelle un escargot compte dans la case de Pépin : 6 tuiles en x
/// (entre les centres), 4 tuiles en y (entre les pieds).
const PORTEE_ESCARGOT: (f32, f32) = (96.0, 64.0);

/// L'escargot le plus proche de Pépin (dans `PORTEE_ESCARGOT`), résumé en un
/// entier : `1 + 2 × (case de 16 px de son x) + (1 s'il va à droite)` ; 0 si aucun
/// escargot n'est proche (toujours 0 dans un niveau sans escargot : la case est
/// alors exactement celle d'avant la carte 5.8).
///
/// Pourquoi : l'escargot est un tremplin MOBILE, il faut l'ATTENDRE au bon endroit.
/// Sans lui dans la case, attendre sur place ramène à une case déjà connue et le
/// plan « attendre » est jeté : le robot ne pourrait jamais laisser l'escargot
/// venir sous lui. Avec lui, chaque attente où l'escargot a avancé d'une case est
/// un nouvel état. Loin des escargots, rien ne change (pas d'explosion du nombre
/// d'états).
pub(crate) fn escargot_proche(m: &Monde) -> i32 {
    let j = &m.joueur.boite;
    let centre = j.x + j.w / 2.0;
    m.entites
        .iter()
        .filter_map(|e| match e {
            Entite::Escargot(s) => {
                let dx = (s.boite.x + s.boite.w / 2.0 - centre).abs();
                let dy = (s.boite.bas() - j.bas()).abs();
                (dx <= PORTEE_ESCARGOT.0 && dy <= PORTEE_ESCARGOT.1).then_some((dx, s))
            }
            _ => None,
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map_or(0, |(_, s)| {
            1 + 2 * (s.boite.x / 16.0).floor() as i32 + i32::from(s.vers_la_droite)
        })
}

/// Un morceau de parcours : le plan joué, les entrées exactes qu'il a produites
/// et la case (`cle`) où Pépin s'est posé, a été relancé, ou a touché l'arrivée.
#[derive(Clone)]
pub(crate) struct Etape {
    pub(crate) plan: Plan,
    pub(crate) entrees: Vec<Entrees>,
    pub(crate) cle: CaseParcours,
}

/// Le monde du niveau `indice` après `PAS_DEPART` pas sans entrée (Pépin posé),
/// et ces entrées vides.
pub(crate) fn depart_pose(indice: usize) -> (Monde, Vec<Entrees>) {
    let mut depart = Monde::new(indice);
    let prefixe = vec![Entrees::default(); PAS_DEPART as usize];
    for e in &prefixe {
        depart.avancer(e);
    }
    (depart, prefixe)
}

/// Plus court chemin (en pas) depuis `depart` jusqu'à l'arrivée, ou jusqu'à un
/// atterrissage que `est_cible` accepte. `limite` : nombre maximal de noeuds
/// développés (`usize::MAX` : aucune). `None` si rien n'est trouvé.
///
/// Le robot cherche l'arrivée seule (`est_cible` toujours faux) ; la mesure de
/// difficulté s'en sert aussi pour « rejoindre le parcours » après une erreur.
/// Une arrivée compte si `gouttes_avant_la_victoire` l'admet (voir `chercher_avec`).
pub(crate) fn chercher(
    depart: &Monde,
    tous: &[Plan],
    est_cible: &dyn Fn(&Monde) -> bool,
    limite: usize,
) -> Option<Vec<Etape>> {
    chercher_avec(depart, tous, est_cible, &gouttes_avant_la_victoire, limite)
}

/// Comme `chercher`, mais une arrivée ne compte que si `arrivee_admise` l'accepte
/// (sinon c'est une impasse, comme une mort). La partie « complète » de la carte
/// 5.16 (`trouver_parcours_complet`) refuse ainsi de finir un niveau tant qu'une
/// goutte manque.
pub(crate) fn chercher_avec(
    depart: &Monde,
    tous: &[Plan],
    est_cible: &dyn Fn(&Monde) -> bool,
    arrivee_admise: &dyn Fn(&Monde) -> bool,
    limite: usize,
) -> Option<Vec<Etape>> {
    // Chaque noeud : le monde posé, son parent et l'étape qui y a mené. La file
    // de priorité ne contient que (coût, indice).
    struct Noeud {
        monde: Monde,
        parent: Option<usize>,
        etape: Option<Etape>,
    }
    let mut noeuds = vec![Noeud {
        monde: depart.clone(),
        parent: None,
        etape: None,
    }];
    let mut meilleur: HashMap<CaseParcours, u32> = HashMap::new();
    meilleur.insert(cle(&noeuds[0].monde), 0);
    let mut tas = BinaryHeap::new();
    tas.push(Reverse((0u32, 0usize)));
    let mut arrivee: Option<(u32, usize, Etape)> = None;
    let mut developpes = 0usize;
    // Après un rebond d'auvent, Pépin est en l'air : on n'essaie que les plans en vol.
    let en_vol = plans_en_vol();

    while let Some(Reverse((cout, i))) = tas.pop() {
        if let Some((cout_arrivee, _, _)) = &arrivee {
            if cout >= *cout_arrivee {
                break; // plus rien de moins cher à trouver
            }
        }
        if meilleur
            .get(&cle(&noeuds[i].monde))
            .is_some_and(|&c| c < cout)
        {
            continue;
        }
        if developpes >= limite {
            break;
        }
        developpes += 1;
        let a_essayer = if est_lance(&noeuds[i].monde) {
            &en_vol[..]
        } else {
            tous
        };
        for &plan in a_essayer {
            let (fin, monde, entrees) = simuler(&noeuds[i].monde, plan);
            // Dans un niveau à aiguilles d'horloge : un plan qui ne tient qu'à
            // l'image près est écarté, comme une mort (`tient_au_decalage`).
            if !matches!(fin, Fin::Mort | Fin::Abandon)
                && !tient_au_decalage(&noeuds[i].monde, plan, fin, &monde)
            {
                continue;
            }
            let cout_plan = cout + entrees.len() as u32;
            let etape = Etape {
                plan,
                entrees,
                cle: cle(&monde),
            };
            match fin {
                // Une arrivée refusée (le Grand Nuage vaincu sans toutes les
                // gouttes, ou une goutte qui manque dans la partie complète) : impasse.
                Fin::Arrivee if !arrivee_admise(&monde) => {}
                Fin::Arrivee => {
                    if arrivee.as_ref().is_none_or(|(c, _, _)| cout_plan < *c) {
                        arrivee = Some((cout_plan, i, etape));
                    }
                }
                Fin::Pose | Fin::Lance if est_cible(&monde) => {
                    if arrivee.as_ref().is_none_or(|(c, _, _)| cout_plan < *c) {
                        arrivee = Some((cout_plan, i, etape));
                    }
                }
                Fin::Pose | Fin::Lance => {
                    let k = etape.cle;
                    if meilleur.get(&k).is_none_or(|&c| cout_plan < c) {
                        meilleur.insert(k, cout_plan);
                        noeuds.push(Noeud {
                            monde,
                            parent: Some(i),
                            etape: Some(etape),
                        });
                        tas.push(Reverse((cout_plan, noeuds.len() - 1)));
                    }
                }
                Fin::Mort | Fin::Abandon => {}
            }
        }
    }

    let (_, dernier, etape_finale) = arrivee?;
    // Remonte la chaîne des parents, puis remet les étapes à l'endroit.
    let mut etapes = vec![etape_finale];
    let mut courant = Some(dernier);
    while let Some(i) = courant {
        if let Some(e) = &noeuds[i].etape {
            etapes.push(e.clone());
        }
        courant = noeuds[i].parent;
    }
    etapes.reverse();
    Some(etapes)
}

/// Cherche les entrées (une par pas) qui mènent du début du niveau `indice` à
/// l'arrivée sans jamais mourir. `None` si rien n'est trouvé.
pub fn trouver_parcours(indice: usize, precis: bool) -> Option<Vec<Entrees>> {
    let (depart, mut entrees) = depart_pose(indice);
    let etapes = chercher(&depart, &plans(precis), &|_| false, usize::MAX)?;
    for e in etapes {
        entrees.extend(e.entrees);
    }
    Some(entrees)
}

/// Nombre de gouttes d'or du niveau déjà prises dans `m`.
fn gouttes_prises(m: &Monde) -> usize {
    m.gouttes_niveau.iter().filter(|&&g| g).count()
}

/// Comme `trouver_parcours`, mais en ramassant les TROIS gouttes d'or du niveau
/// avant l'arrivée (carte 5.16 : la partie « complète », celle qui débloque le
/// bonus). Recherche par étapes, comme pour les auvents de l'essai : la goutte la
/// plus proche (en temps), puis la suivante, puis l'arrivée ; une arrivée avec une
/// goutte en moins est une impasse (`chercher_avec`). Chaque étape est rejouée sur
/// le vrai monde (la simulation est déterministe : on retrouve exactement l'état
/// trouvé par la recherche). Le parcours n'est pas forcément le plus court qui
/// ramasse tout (on prend les gouttes dans l'ordre où elles sont les plus proches),
/// mais il est sûr par construction, comme celui du robot.
pub fn trouver_parcours_complet(indice: usize, precis: bool) -> Option<Vec<Entrees>> {
    let (mut monde, mut entrees) = depart_pose(indice);
    let tous = plans(precis);
    let toutes = |m: &Monde| gouttes_prises(m) == m.gouttes_niveau.len();
    while !monde.termine {
        let prises = gouttes_prises(&monde);
        let etapes = if toutes(&monde) {
            chercher(&monde, &tous, &|_| false, usize::MAX)?
        } else {
            let une_de_plus = |m: &Monde| gouttes_prises(m) > prises;
            chercher_avec(&monde, &tous, &une_de_plus, &toutes, usize::MAX)?
        };
        for e in etapes {
            for x in &e.entrees {
                monde.avancer(x);
            }
            entrees.extend(e.entrees);
        }
    }
    Some(entrees)
}
