//! Robot de QA : un « joueur automatique » qui finit tous les niveaux avec la
//! VRAIE simulation du jeu (`Monde` : ennemis, éclairs, plateformes mobiles et
//! friables comprises), puis rejoue ce parcours à travers l'`Application`
//! (titre, cartes de niveau, transitions, écran de fin).
//!
//! Différence avec `pepin_core::faisabilite` : le robot du cœur ignore ennemis
//! et plateformes mobiles. Celui-ci les subit : un plan qui mène à la mort est
//! simplement écarté, donc le parcours trouvé est sûr par construction.
//!
//! Principe : recherche de plus court chemin (Dijkstra sur la durée) entre des
//! états « posés et immobiles » (ou « lancés » : un auvent vient de relancer
//! Pépin, et il choisit de nouveau en l'air ; voir `est_lance`). Chaque plan
//! (courir, sauter, planer, attendre) est simulé sur une COPIE du monde. Le test
//! `ignore` d'en bas écrit le script des touches dans `target/qa/` : un
//! navigateur réel le rejoue image par image (voir QA.md).
//!
//! Notions Rust illustrées ici : `BinaryHeap` + `Reverse` (file de priorité
//! « plus petit d'abord »), `Clone` d'une structure complète pour explorer des
//! futurs possibles sans toucher au présent, `HashMap` comme mémoire.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use pepin_core::entities::contacts::Contact;
use pepin_core::entities::ennemi::{
    pas_du_cycle, EtatEnnemi, PhaseChat, PhasePigeon, ANIMATION_ECRASE_PAS, DORT_PAS, REVEIL_PAS,
};
use pepin_core::entities::montgolfiere::EtatMontgolfiere;
use pepin_core::entities::nuage::PhaseNuage;
use pepin_core::entities::ventilateur::PhaseVentilateur;
use pepin_core::entities::{Entite, TypeEnnemi};
use pepin_core::faisabilite::pas_max_plan;
use pepin_core::player::{Direction, Entrees};
use pepin_core::zones::NB_NIVEAUX;

use crate::app::Application;
use crate::ecrans::Ecran;
use crate::entrees::Commandes;
use crate::monde::Monde;
use crate::navigation::Navigation;

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
const MARGE_CHAT_PAS: u16 = 6;

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
pub(crate) type CaseParcours = (i32, i32, bool, i32, i32, i32, i32);

/// La case (`CaseParcours`) de Pépin dans `m`.
pub(crate) fn cle(m: &Monde) -> CaseParcours {
    (
        (m.joueur.boite.x / 8.0).floor() as i32,
        m.joueur.boite.y.round() as i32,
        est_lance(m),
        escargot_proche(m),
        chats_de_la_case(m),
        aiguille_proche(m),
        nuage_de_la_case(m),
    )
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
pub(crate) fn chercher(
    depart: &Monde,
    tous: &[Plan],
    est_cible: &dyn Fn(&Monde) -> bool,
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
                // Le Grand Nuage vaincu sans toutes les gouttes : impasse.
                Fin::Arrivee if !gouttes_avant_la_victoire(&monde) => {}
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

/// Ce que le navigateur doit rejouer à chaque image.
#[derive(Clone, Copy, Default)]
pub struct Trame {
    pub valider: bool,
    pub entrees: Entrees,
    /// Numéro du niveau affiché (0 hors jeu) et position de Pépin : pour choisir
    /// les images à photographier.
    pub niveau: u8,
    pub x: f32,
    pub y: f32,
    pub ecran: &'static str,
    /// Position du premier corbeau actif du niveau (pour viser une capture).
    pub corbeau: Option<(f32, f32)>,
    /// Boîte de la première plateforme mobile (x, y), pour viser une capture.
    pub mobile: Option<(f32, f32)>,
    /// Premier pigeon visible du niveau : position et phase (`nom_phase`), pour
    /// viser une capture (perché, annonce, plongée...).
    pub pigeon: Option<(f32, f32, &'static str)>,
    /// État de la partie après l'image : score, gouttes, morts et chrono (pas de
    /// jeu). Ils ne vont pas dans `trace.csv` ; ils nourrissent `empreinte`.
    pub score: u32,
    pub gouttes: u32,
    pub morts: u32,
    pub chrono: u32,
}

impl Trame {
    /// Un chiffre hexadécimal : 1 = Entrée, 2 = gauche, 4 = droite, 8 = saut.
    pub fn code(&self) -> char {
        let mut v = 0u32;
        if self.valider {
            v |= 1;
        }
        match self.entrees.direction {
            Direction::Gauche => v |= 2,
            Direction::Droite => v |= 4,
            Direction::Immobile => {}
        }
        if self.entrees.saut {
            v |= 8;
        }
        char::from_digit(v, 16).unwrap()
    }
}

/// Nom court d'un état de pigeon, pour la trace (`ecrase` : écrasé, quelle que soit
/// la phase où on l'a eu).
fn nom_phase(phase: PhasePigeon, ecrase: bool) -> &'static str {
    if ecrase {
        return "ecrase";
    }
    match phase {
        PhasePigeon::Perche => "perche",
        PhasePigeon::Annonce(_) => "annonce",
        PhasePigeon::Plongee(_) => "plongee",
        PhasePigeon::Remontee(_) => "remontee",
        PhasePigeon::Repos(_) => "repos",
    }
}

fn nom_ecran(e: Ecran) -> &'static str {
    match e {
        Ecran::Titre => "titre",
        Ecran::Aide => "aide",
        Ecran::Carte(_) => "carte",
        Ecran::Jeu => "jeu",
        Ecran::Pause => "pause",
        Ecran::Fin => "fin",
    }
}

/// Résultat d'une partie complète jouée par le robot.
pub struct Partie {
    pub trames: Vec<Trame>,
    pub morts: u32,
    pub gouttes: u32,
    pub score: u32,
    /// Chrono de la partie (pas de jeu, sans menus ni transitions).
    pub chrono: u32,
    pub fin_atteinte: bool,
}

/// Empreinte (hachage FNV-1a sur 64 bits) de la partie, image par image : écran,
/// niveau affiché, score, gouttes, morts et chrono. Deux parties qui ont la même
/// empreinte ont eu le même chrono et le même score à CHAQUE image, pas seulement à
/// la fin : c'est la preuve qu'une carte « sans effet sur le jeu » n'a rien changé.
pub fn empreinte(trames: &[Trame]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut ajouter = |octets: &[u8]| {
        for &o in octets {
            h ^= u64::from(o);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    };
    for t in trames {
        ajouter(t.ecran.as_bytes());
        ajouter(&[t.niveau]);
        for v in [t.score, t.gouttes, t.morts, t.chrono] {
            ajouter(&v.to_le_bytes());
        }
    }
    h
}

/// Joue la partie entière à travers l'`Application`, image par image, comme le
/// ferait la boucle principale (un `reagir` puis un `pas` par image).
pub fn jouer_tout(parcours: &[Vec<Entrees>]) -> Partie {
    let mut app = Application::new(true);
    let mut trames: Vec<Trame> = Vec::new();
    let mut fin_atteinte = false;
    for image in 0..60 * 60 * 15 {
        // Image 0 : « JOUER » au titre. Ensuite les cartes passent toutes seules.
        let mut trame = Trame {
            valider: image == 0,
            ..Trame::default()
        };
        if app.ecran == Ecran::Jeu && app.en_jeu() {
            let n = app.monde.infos.numero as usize;
            if let Some(e) = parcours
                .get(n - 1)
                .and_then(|p| p.get(app.monde.pas as usize))
            {
                trame.entrees = *e;
            }
        }
        let nav = Navigation {
            valider: trame.valider,
            ..Navigation::default()
        };
        app.reagir(&nav, &Commandes::default(), None);
        app.pas(&trame.entrees);
        trame.ecran = nom_ecran(app.ecran);
        trame.niveau = if app.ecran == Ecran::Jeu {
            app.monde.infos.numero
        } else {
            0
        };
        trame.corbeau = app.monde.entites.iter().find_map(|e| match e {
            Entite::Ennemi(en) if en.type_ennemi == TypeEnnemi::Corbeau => {
                Some((en.boite.x, en.boite.y))
            }
            _ => None,
        });
        trame.mobile = app.monde.entites.iter().find_map(|e| match e {
            Entite::Mobile(m) => Some((m.boite.x, m.boite.y)),
            _ => None,
        });
        trame.pigeon = app.monde.entites.iter().find_map(|e| match e {
            Entite::Ennemi(p) if p.type_ennemi == TypeEnnemi::Pigeon && p.est_visible() => {
                let ecrase = !p.est_dangereux();
                Some((p.boite.x, p.boite.y, nom_phase(p.phase(), ecrase)))
            }
            _ => None,
        });
        trame.x = app.monde.joueur.boite.x;
        trame.y = app.monde.joueur.boite.y;
        trame.score = app.monde.etat.score;
        trame.gouttes = app.monde.etat.gouttes;
        trame.morts = app.monde.etat.morts;
        trame.chrono = app.chrono();
        trames.push(trame);
        if app.ecran == Ecran::Fin {
            fin_atteinte = true;
            // Quelques images de confettis, pour la capture.
            for _ in 0..240 {
                app.reagir(&Navigation::default(), &Commandes::default(), None);
                app.pas(&Entrees::default());
                trames.push(Trame {
                    ecran: "fin",
                    ..Trame::default()
                });
            }
            break;
        }
    }
    Partie {
        trames,
        morts: app.monde.etat.morts,
        gouttes: app.monde.etat.gouttes,
        score: app.monde.etat.score,
        chrono: app.chrono(),
        fin_atteinte,
    }
}

/// Cherche le parcours de chaque niveau de la table (robot « maladroit » d'abord,
/// puis précis si besoin).
pub fn tous_les_parcours() -> Vec<Vec<Entrees>> {
    (0..NB_NIVEAUX)
        .map(|i| {
            trouver_parcours(i, false)
                .or_else(|| trouver_parcours(i, true))
                .unwrap_or_else(|| panic!("le robot ne trouve aucun parcours au niveau {}", i + 1))
        })
        .collect()
}

// --- Niveau d'essai vertical (carte 5.2) -------------------------------------------

/// Une image de la partie d'essai : ce que le navigateur rejoue, et ce qu'on veut
/// vérifier de la caméra (position RENDUE, celle que la scène dessine, avec `alpha` = 1).
#[derive(Clone, Copy)]
pub struct ImageEssai {
    pub trame: Trame,
    /// La touche 0 (raccourci du niveau d'essai) est appuyée pendant cette image.
    pub touche_essai: bool,
    /// On joue le niveau d'essai (écran de jeu, monde qui avance).
    pub en_essai: bool,
    /// Caméra entière `(cx, cy)` et ordonnée de Pépin à l'écran (haut de sa boîte).
    pub camera: (i32, i32),
    pub pepin_ecran_y: i32,
    /// La flèche de repère est dessinée (Pépin au-dessus du cadre).
    pub repere: bool,
    /// Pépin plane DANS une colonne de grêle (carte 5.4) : parapluie ouvert, la grêle
    /// se brise sur la toile.
    pub sous_la_grele: bool,
    /// Un auvent vient de relancer Pépin pendant cette image (carte 5.5).
    pub rebond_auvent: bool,
    /// Pépin est posé au sol (sur une tuile ou un solide) à la fin de cette image.
    pub au_sol: bool,
    /// Haut des nacelles des deux premières montgolfières du niveau, de gauche à
    /// droite (carte 5.7 ; -1 s'il n'y en a pas).
    pub nacelles: [f32; 2],
    /// Une montgolfière monte pendant cette image (Pépin à bord, brûleur allumé).
    pub montee: bool,
    /// Phase du premier ventilateur du niveau et pas de souffle qui lui restent
    /// (carte 5.6b ; `None` s'il n'y en a pas) : sert à choisir les images à capturer.
    pub ventilateur: Option<(PhaseVentilateur, u16)>,
    /// Coin haut-gauche des deux premiers escargots du niveau, de gauche à droite
    /// au départ (carte 5.8 ; `(-1, -1)` s'il n'y en a pas), et coquille aplatie.
    pub escargots: [(f32, f32, bool); 2],
    /// Pépin vient de rebondir sur un escargot pendant cette image (carte 5.8).
    pub rebond_escargot: bool,
    /// Pépin touche un escargot sans rebondir (de côté : rien ne se passe).
    pub contre_escargot: bool,
    /// Premier chat du niveau (carte 5.9 ; `None` s'il n'y en a pas) : sa phase,
    /// et `true` s'il est écrasé (ou déjà disparu).
    pub chat: Option<(PhaseChat, bool)>,
    /// Pépin vient d'écraser un chat pendant cette image.
    pub ecrase_chat: bool,
    /// Coin haut-gauche des deux premières aiguilles d'horloge du niveau, dans
    /// l'ordre des entités (carte 5.10 ; `(-1, -1)` s'il n'y en a pas).
    pub aiguilles: [(f32, f32); 2],
    /// L'aiguille qui porte Pépin à la fin de cette image (`aiguille_sous_pepin`).
    pub sur_aiguille: Option<usize>,
    /// Le Grand Nuage (carte 5.11 ; `None` s'il n'y en a pas) : sa phase, le temps
    /// passé dans cette phase et ses points de vie.
    pub nuage: Option<(PhaseNuage, u16, u8)>,
    /// Pépin vient de sauter sur la tête du Grand Nuage pendant cette image.
    pub coup_nuage: bool,
}

/// Résultat de la partie d'essai jouée à travers l'`Application`.
pub struct PartieEssai {
    pub images: Vec<ImageEssai>,
    pub fin_atteinte: bool,
    pub morts: u32,
    pub gouttes: u32,
}

/// Joue un niveau d'essai à travers l'`Application`, comme le ferait le build de
/// QA : « JOUER » au titre (image 0), carte du niveau 1, puis, dès la première
/// image de jeu, la touche du niveau d'essai (`raccourci`) ; la carte du niveau
/// d'essai passe, puis le robot joue `parcours` jusqu'à l'écran de fin.
fn jouer_essai(
    indice_essai: usize,
    raccourci: crate::entrees::RaccourciNiveau,
    parcours: &[Entrees],
) -> PartieEssai {
    let mut app = Application::new(true);
    let mut images = Vec::new();
    let mut appuyee = false;
    let mut fin_atteinte = false;
    for image in 0..60 * 60 * 3 {
        let mut trame = Trame {
            valider: image == 0,
            ..Trame::default()
        };
        let en_essai = app.en_jeu() && app.monde.indice == indice_essai;
        let touche_essai = !appuyee && app.en_jeu() && app.monde.indice == 0;
        appuyee |= touche_essai;
        if en_essai {
            if let Some(e) = parcours.get(app.monde.pas as usize) {
                trame.entrees = *e;
            }
        }
        let nav = Navigation {
            valider: trame.valider,
            ..Navigation::default()
        };
        let raccourci_image = touche_essai.then_some(raccourci);
        app.reagir(&nav, &Commandes::default(), raccourci_image);
        app.pas(&trame.entrees);
        trame.ecran = nom_ecran(app.ecran);
        trame.x = app.monde.joueur.boite.x;
        trame.y = app.monde.joueur.boite.y;
        trame.gouttes = app.monde.etat.gouttes;
        trame.morts = app.monde.etat.morts;
        let j = &app.monde.joueur;
        let camera = app.camera.position_rendue(1.0, (j.boite.x, j.boite.y));
        let pepin_ecran_y = j.boite.y.round() as i32 - camera.1;
        let milieu = j.boite.x.round() as i32 - camera.0 + j.boite.w as i32 / 2;
        let repere = crate::scene::repere_haut(pepin_ecran_y, j.boite.h as i32, milieu).is_some();
        let sous_la_grele = j.plane
            && app.monde.entites.iter().any(|e| {
                matches!(e, pepin_core::entities::Entite::Grele(g) if g.zone.chevauche(&j.boite))
            });
        let rebond_auvent = app.monde.contacts.contains(&Contact::Auvent);
        let au_sol = j.au_sol;
        // Montgolfières rangées de gauche à droite (l'ordre du parcours dans
        // l'essai), et non dans l'ordre de lecture du niveau (de haut en bas).
        let mut montgolfieres: Vec<(f32, f32, bool)> = app
            .monde
            .entites
            .iter()
            .filter_map(|e| match e {
                Entite::Montgolfiere(m) => {
                    Some((m.boite.x, m.boite.y, m.etat() == EtatMontgolfiere::Montee))
                }
                _ => None,
            })
            .collect();
        montgolfieres.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut nacelles = [-1.0f32; 2];
        for (y, m) in nacelles.iter_mut().zip(&montgolfieres) {
            *y = m.1;
        }
        let montee = montgolfieres.iter().any(|m| m.2);
        let ventilateur = app.monde.entites.iter().find_map(|e| match e {
            Entite::Ventilateur(v) => Some((v.phase(), v.souffle_restant())),
            _ => None,
        });
        // Escargots dans l'ordre du niveau (de gauche à droite au départ : l'ordre
        // de lecture d'une même rangée).
        let mut escargots = [(-1.0f32, -1.0f32, false); 2];
        let liste = app.monde.entites.iter().filter_map(|e| match e {
            Entite::Escargot(s) => Some(s),
            _ => None,
        });
        for (place, s) in escargots.iter_mut().zip(liste) {
            *place = (s.boite.x, s.boite.y, s.est_comprime());
        }
        let touche_escargot = app
            .monde
            .entites
            .iter()
            .any(|e| matches!(e, Entite::Escargot(s) if s.boite.chevauche(&j.boite)));
        let rebond_escargot = touche_escargot && app.monde.contacts.contains(&Contact::Rebond);
        let contre_escargot = touche_escargot && !rebond_escargot && j.au_sol;
        let chat = app.monde.entites.iter().find_map(|e| match e {
            Entite::Ennemi(c) if c.type_ennemi == TypeEnnemi::Chat => {
                Some((c.phase_chat(), !c.est_dangereux()))
            }
            _ => None,
        });
        // Un écrasement cette image, et le chat écrasé maintenant, vivant juste avant.
        let ecrase_chat = app.monde.contacts.contains(&Contact::Ecrase)
            && chat.is_some_and(|c| c.1)
            && images
                .last()
                .is_some_and(|i: &ImageEssai| i.chat.is_some_and(|c| !c.1));
        let mut aiguilles = [(-1.0f32, -1.0f32); 2];
        let liste = app.monde.entites.iter().filter_map(|e| match e {
            Entite::Mobile(a) if a.cercle().is_some() => Some(a.boite),
            _ => None,
        });
        for (place, b) in aiguilles.iter_mut().zip(liste) {
            *place = (b.x, b.y);
        }
        let sur_aiguille = aiguille_sous_pepin(&app.monde);
        let nuage = app.monde.entites.iter().find_map(|e| match e {
            Entite::Nuage(g) => Some((g.phase(), g.ecoule(), g.pv())),
            _ => None,
        });
        let coup_nuage = app.monde.contacts.contains(&Contact::Nuage);
        images.push(ImageEssai {
            trame,
            touche_essai,
            en_essai: en_essai && app.en_jeu(),
            camera,
            pepin_ecran_y,
            repere,
            sous_la_grele,
            rebond_auvent,
            au_sol,
            nacelles,
            montee,
            ventilateur,
            escargots,
            rebond_escargot,
            contre_escargot,
            chat,
            ecrase_chat,
            aiguilles,
            sur_aiguille,
            nuage,
            coup_nuage,
        });
        if app.ecran == Ecran::Fin {
            fin_atteinte = true;
            break;
        }
    }
    PartieEssai {
        images,
        fin_atteinte,
        morts: app.monde.etat.morts,
        gouttes: app.monde.etat.gouttes,
    }
}

/// Joue le niveau d'essai vertical (touche 0 du build de QA) — voir `jouer_essai`.
pub fn jouer_essai_vertical(parcours: &[Entrees]) -> PartieEssai {
    use crate::entrees::RaccourciNiveau;
    jouer_essai(crate::monde::INDICE_ESSAI, RaccourciNiveau::Essai, parcours)
}

/// Joue le niveau d'essai « vent » (touche F1 du build de QA) — voir `jouer_essai`.
pub fn jouer_essai_vent(parcours: &[Entrees]) -> PartieEssai {
    use crate::entrees::RaccourciNiveau;
    jouer_essai(
        crate::monde::INDICE_ESSAI_VENT,
        RaccourciNiveau::Essai2,
        parcours,
    )
}

/// Parcours du robot dans le niveau d'essai vertical (maladroit, puis précis).
pub fn parcours_essai_vertical() -> Vec<Entrees> {
    let i = crate::monde::INDICE_ESSAI;
    trouver_parcours(i, false)
        .or_else(|| trouver_parcours(i, true))
        .expect("le robot ne trouve aucun parcours au niveau d'essai vertical")
}

/// Parcours du robot dans le niveau d'essai « vent » (maladroit, puis précis) : le
/// plané qui traverse le vide est porté par le vent latéral du niveau.
pub fn parcours_essai_vent() -> Vec<Entrees> {
    let i = crate::monde::INDICE_ESSAI_VENT;
    trouver_parcours(i, false)
        .or_else(|| trouver_parcours(i, true))
        .expect("le robot ne trouve aucun parcours au niveau d'essai vent")
}

/// Joue le niveau d'essai « grêle » (touche F2 du build de QA) — voir `jouer_essai`.
pub fn jouer_essai_grele(parcours: &[Entrees]) -> PartieEssai {
    use crate::entrees::RaccourciNiveau;
    jouer_essai(
        crate::monde::INDICE_ESSAI_GRELE,
        RaccourciNiveau::Essai3,
        parcours,
    )
}

/// Joue le niveau d'essai « auvent » (touche F3 du build de QA) — voir `jouer_essai`.
pub fn jouer_essai_auvent(parcours: &[Entrees]) -> PartieEssai {
    use crate::entrees::RaccourciNiveau;
    jouer_essai(
        crate::monde::INDICE_ESSAI_AUVENT,
        RaccourciNiveau::Essai4,
        parcours,
    )
}

/// Joue le niveau d'essai « ventilateur » (touche F4 du build de QA) — voir
/// `jouer_essai`.
pub fn jouer_essai_ventilateur(parcours: &[Entrees]) -> PartieEssai {
    use crate::entrees::RaccourciNiveau;
    jouer_essai(
        crate::monde::INDICE_ESSAI_VENTILATEUR,
        RaccourciNiveau::Essai5,
        parcours,
    )
}

/// Parcours de démonstration du niveau d'essai « ventilateur » (carte 5.6b, pour les
/// captures) : Pépin attend au départ que le PREMIER souffle soit passé, monte sur
/// la casette pendant l'arrêt, puis ne touche plus à rien. Il y est donc posé
/// pendant toute l'annonce suivante, le souffle le porte jusqu'en haut de la
/// colonne, et il retombe à l'arrêt. Le saut (élan, appui, durée) est cherché
/// sur la vraie simulation : `None` si aucun ne pose Pépin sur la casette.
pub fn parcours_essai_ventilateur_caisse() -> Option<Vec<Entrees>> {
    use crate::monde::{Monde, INDICE_ESSAI_VENTILATEUR};
    // Casette du ventilateur, sa phase.
    let casette = |m: &Monde| {
        m.entites.iter().find_map(|e| match e {
            Entite::Ventilateur(v) => Some((v.boite, v.phase())),
            _ => None,
        })
    };
    // Les pas du cycle, mesurés sur la simulation (sans entrée) : fin du premier
    // souffle, puis début de l'annonce et du souffle suivants.
    let mut m = Monde::new(INDICE_ESSAI_VENTILATEUR);
    let mut pas = 0u32;
    let mut vu_souffle = false;
    let (mut fin_souffle, mut annonce, mut souffle) = (None, None, None);
    while souffle.is_none() && pas < 1000 {
        m.avancer(&Entrees::default());
        pas += 1;
        match casette(&m)?.1 {
            PhaseVentilateur::Souffle if annonce.is_some() => souffle = Some(pas),
            PhaseVentilateur::Souffle => vu_souffle = true,
            PhaseVentilateur::Arret if vu_souffle && fin_souffle.is_none() => {
                fin_souffle = Some(pas)
            }
            PhaseVentilateur::Annonce if fin_souffle.is_some() && annonce.is_none() => {
                annonce = Some(pas)
            }
            _ => {}
        }
    }
    let (fin_souffle, annonce, souffle) = (fin_souffle?, annonce?, souffle?);
    // Préfixe : immobile jusqu'à la fin du premier souffle (quelques pas de marge).
    let attente = fin_souffle as usize + 4;
    let mut depart = Monde::new(INDICE_ESSAI_VENTILATEUR);
    for _ in 0..attente {
        depart.avancer(&Entrees::default());
    }
    for course in 1..40usize {
        for debut_saut in 0..course {
            for duree_saut in 1..16usize {
                let elan: Vec<Entrees> = (0..course)
                    .map(|k| Entrees {
                        direction: Direction::Droite,
                        saut: k >= debut_saut && k < debut_saut + duree_saut,
                    })
                    .collect();
                let mut essai = depart.clone();
                let mut pas = attente as u32;
                for e in &elan {
                    essai.avancer(e);
                    pas += 1;
                }
                if pas >= annonce {
                    continue;
                }
                // Immobile jusqu'au souffle : Pépin doit être posé sur la casette
                // pendant TOUTE l'annonce.
                let mut pose = true;
                while pas < souffle {
                    essai.avancer(&Entrees::default());
                    pas += 1;
                    let boite = casette(&essai)?.0;
                    let j = &essai.joueur;
                    if (annonce..souffle).contains(&pas)
                        && !(j.au_sol
                            && (j.boite.bas() - boite.y).abs() < 0.5
                            && j.boite.x >= boite.x
                            && j.boite.x + j.boite.w <= boite.x + boite.w)
                    {
                        pose = false;
                        break;
                    }
                }
                if !pose || essai.etat.morts > 0 {
                    continue;
                }
                let mut parcours = vec![Entrees::default(); attente];
                parcours.extend(elan);
                return Some(parcours);
            }
        }
    }
    None
}

/// Parcours du robot dans le niveau d'essai « auvent » (maladroit, puis précis) :
/// chaque rebond d'auvent est un point de décision (`Fin::Lance`), d'où il essaie
/// les plans en vol (`plans_en_vol`). Il cherche le plus RAPIDE : en gardant la
/// droite, il saute des auvents (voir `parcours_essai_auvent_rebond_par_rebond`).
pub fn parcours_essai_auvent() -> Vec<Entrees> {
    let i = crate::monde::INDICE_ESSAI_AUVENT;
    trouver_parcours(i, false)
        .or_else(|| trouver_parcours(i, true))
        .expect("le robot ne trouve aucun parcours au niveau d'essai auvent")
}

/// Parcours « rebond par rebond » du niveau d'essai « auvent » : le robot vise
/// chaque auvent à son tour (de gauche à droite), sans en sauter aucun, puis le
/// drapeau. Chaque morceau est un plus court chemin (`chercher`) dont la cible est
/// « vient d'être relancé par CET auvent ». C'est le moment fort du marché couvert
/// (six auvents à la suite sans toucher le sol), joué par le robot pour les captures.
pub fn parcours_essai_auvent_rebond_par_rebond() -> Vec<Entrees> {
    let (mut monde, mut entrees) = depart_pose(crate::monde::INDICE_ESSAI_AUVENT);
    let mut auvents: Vec<pepin_core::aabb::Aabb> = monde
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Auvent(a) => Some(a.boite),
            _ => None,
        })
        .collect();
    auvents.sort_by(|a, b| a.x.total_cmp(&b.x));
    let tous = plans(false);
    let mut jouer = |monde: &mut Monde, etapes: Vec<Etape>| {
        for e in etapes {
            for x in &e.entrees {
                monde.avancer(x);
            }
            entrees.extend(e.entrees);
        }
    };
    for cible in auvents {
        // Relancé par CET auvent : posé sur sa toile, en vol, élan tout neuf.
        let relance = |m: &Monde| {
            let b = m.joueur.boite;
            est_lance(m)
                && (b.bas() - cible.y).abs() < 0.01
                && b.x < cible.droite()
                && cible.x < b.droite()
        };
        let etapes = chercher(&monde, &tous, &relance, usize::MAX)
            .expect("le robot ne sait pas atteindre l'auvent suivant");
        jouer(&mut monde, etapes);
    }
    let etapes = chercher(&monde, &tous, &|_| false, usize::MAX)
        .expect("le robot ne sait pas finir après le dernier auvent");
    jouer(&mut monde, etapes);
    entrees
}

/// Parcours du robot dans le niveau d'essai « grêle » (maladroit, puis précis). La
/// grêle n'a rien de spécial pour lui : la VRAIE simulation (`Monde`) tue un plan qui
/// la touche parapluie fermé, ce plan est écarté, et il reste ceux qui la traversent
/// en planant (ou qui passent sous la corniche).
pub fn parcours_essai_grele() -> Vec<Entrees> {
    let i = crate::monde::INDICE_ESSAI_GRELE;
    trouver_parcours(i, false)
        .or_else(|| trouver_parcours(i, true))
        .expect("le robot ne trouve aucun parcours au niveau d'essai grêle")
}

/// Parcours du robot dans le niveau d'essai « ventilateur » (maladroit, puis précis).
/// Le ventilateur porte Pépin d'un étage entier ; le robot apprend à attendre le
/// souffle sur la casette, puis à marcher sur le balcon qu'il atteint.
pub fn parcours_essai_ventilateur() -> Vec<Entrees> {
    let i = crate::monde::INDICE_ESSAI_VENTILATEUR;
    trouver_parcours(i, false)
        .or_else(|| trouver_parcours(i, true))
        .expect("le robot ne trouve aucun parcours au niveau d'essai ventilateur")
}

/// Joue le niveau d'essai « montgolfière » (touche F5 du build de QA) — voir
/// `jouer_essai`.
pub fn jouer_essai_montgolfiere(parcours: &[Entrees]) -> PartieEssai {
    use crate::entrees::RaccourciNiveau;
    jouer_essai(
        crate::monde::INDICE_ESSAI_MONTGOLFIERE,
        RaccourciNiveau::Essai6,
        parcours,
    )
}

/// Parcours du robot dans le niveau d'essai « montgolfière » (maladroit, puis
/// précis). Rien de spécial pour lui : sur la VRAIE simulation, un plan qui finit
/// dans la nacelle la fait monter ; il apprend qu'il faut y attendre (plans
/// « attendre ») jusqu'à ce qu'elle s'arrête en haut, puis avancer vers l'étage.
/// Un Pépin trop au bord de la nacelle se cognerait la tête et l'arrêterait plus
/// bas : ces plans-là ne mènent nulle part et sont écartés.
pub fn parcours_essai_montgolfiere() -> Vec<Entrees> {
    let i = crate::monde::INDICE_ESSAI_MONTGOLFIERE;
    trouver_parcours(i, false)
        .or_else(|| trouver_parcours(i, true))
        .expect("le robot ne trouve aucun parcours au niveau d'essai montgolfière")
}

/// Parcours « jusqu'en haut » du niveau d'essai « montgolfière » : le robot monte
/// dans chaque montgolfière à son tour (de gauche à droite) et y reste jusqu'à ce
/// qu'elle s'arrête EN HAUT (8 tuiles pour la première, sous le nuage pour la
/// seconde), puis va au drapeau. Chaque morceau est un plus court chemin
/// (`chercher`) dont la cible est « posé dans CETTE nacelle, arrivée en haut ». Le
/// parcours le plus rapide, lui, saute de la nacelle avant le haut : celui-ci montre
/// toute la mécanique (montée, arrêt en haut, redescente à vide) pour les captures.
pub fn parcours_essai_montgolfiere_jusqu_en_haut() -> Vec<Entrees> {
    let (mut monde, mut entrees) = depart_pose(crate::monde::INDICE_ESSAI_MONTGOLFIERE);
    let mut nacelles: Vec<(f32, f32)> = monde
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Montgolfiere(m) => Some((m.boite.x, m.altitudes().1)),
            _ => None,
        })
        .collect();
    nacelles.sort_by(|a, b| a.0.total_cmp(&b.0));
    let tous = plans(false);
    let mut jouer = |monde: &mut Monde, etapes: Vec<Etape>| {
        for e in etapes {
            for x in &e.entrees {
                monde.avancer(x);
            }
            entrees.extend(e.entrees);
        }
    };
    for (x, haut) in nacelles {
        // Posé dans CETTE nacelle, arrivée tout en haut.
        let en_haut = |m: &Monde| {
            m.joueur.au_sol
                && m.entites.iter().any(|e| {
                    matches!(e, Entite::Montgolfiere(n)
                        if n.boite.x == x && n.boite.y == haut && n.a_un_passager())
                })
        };
        let etapes = chercher(&monde, &tous, &en_haut, usize::MAX)
            .expect("le robot ne sait pas monter en haut de la montgolfière suivante");
        jouer(&mut monde, etapes);
    }
    let etapes = chercher(&monde, &tous, &|_| false, usize::MAX)
        .expect("le robot ne sait pas finir après la dernière montgolfière");
    jouer(&mut monde, etapes);
    entrees
}

/// Joue le niveau d'essai « escargot » (touche F6 du build de QA) — voir
/// `jouer_essai`.
pub fn jouer_essai_escargot(parcours: &[Entrees]) -> PartieEssai {
    use crate::entrees::RaccourciNiveau;
    jouer_essai(
        crate::monde::INDICE_ESSAI_ESCARGOT,
        RaccourciNiveau::Essai7,
        parcours,
    )
}

/// Parcours du robot dans le niveau d'essai « escargot » (maladroit, puis précis).
/// Rien de spécial pour lui : sur la VRAIE simulation, un plan qui retombe sur la
/// coquille de l'escargot le fait rebondir. Il apprend où et quand sauter (plans
/// « attendre », élans) pour que l'escargot soit sous lui au bon moment, au pied
/// de chaque falaise ; un plan où l'escargot est déjà passé retombe au sol et ne
/// mène nulle part.
pub fn parcours_essai_escargot() -> Vec<Entrees> {
    let i = crate::monde::INDICE_ESSAI_ESCARGOT;
    trouver_parcours(i, false)
        .or_else(|| trouver_parcours(i, true))
        .expect("le robot ne trouve aucun parcours au niveau d'essai escargot")
}

/// Joue le niveau d'essai « chat » (touche F7 du build de QA) — voir `jouer_essai`.
pub fn jouer_essai_chat(parcours: &[Entrees]) -> PartieEssai {
    use crate::entrees::RaccourciNiveau;
    jouer_essai(
        crate::monde::INDICE_ESSAI_CHAT,
        RaccourciNiveau::Essai8,
        parcours,
    )
}

/// Parcours du robot dans le niveau d'essai « chat » (maladroit, puis précis).
/// Sur la VRAIE simulation, un plan qui heurte le chat de côté, ou qui lui tombe
/// dessus pendant qu'il est réveillé, tue Pépin : il est écarté. Le robot apprend
/// donc à attendre que le chat dorme (plans « attendre » : le moment du cycle est
/// dans la case, voir `chats_de_la_case`), puis à se laisser tomber sur lui.
pub fn parcours_essai_chat() -> Vec<Entrees> {
    let i = crate::monde::INDICE_ESSAI_CHAT;
    trouver_parcours(i, false)
        .or_else(|| trouver_parcours(i, true))
        .expect("le robot ne trouve aucun parcours au niveau d'essai chat")
}

/// Joue l'arène du Grand Nuage (touche F10 du build de QA) — voir `jouer_essai`.
pub fn jouer_essai_nuage(parcours: &[Entrees]) -> PartieEssai {
    use crate::entrees::RaccourciNiveau;
    jouer_essai(
        crate::monde::INDICE_ESSAI_NUAGE,
        RaccourciNiveau::Essai10,
        parcours,
    )
}

/// Parcours du robot dans l'arène du Grand Nuage (maladroit, puis précis). Sur la
/// VRAIE simulation, le nuage lance ses éclairs : le robot apprend à les esquiver
/// (attendre à l'abri, filer entre deux salves : le temps du nuage est dans la case,
/// voir `nuage_de_la_case`), puis, quand il s'essouffle, à monter dans son courant
/// et à lui retomber sur la tête, trois fois. Chaque plan doit tenir avec le nuage
/// décalé d'une ou deux images (`tient_au_decalage`).
pub fn parcours_essai_nuage() -> Vec<Entrees> {
    let i = crate::monde::INDICE_ESSAI_NUAGE;
    trouver_parcours(i, false)
        .or_else(|| trouver_parcours(i, true))
        .expect("le robot ne trouve aucun parcours dans l'arène du Grand Nuage")
}

/// Joue le niveau d'essai « horloge » (touche F8 du build de QA) — voir `jouer_essai`.
pub fn jouer_essai_horloge(parcours: &[Entrees]) -> PartieEssai {
    use crate::entrees::RaccourciNiveau;
    jouer_essai(
        crate::monde::INDICE_ESSAI_HORLOGE,
        RaccourciNiveau::Essai9,
        parcours,
    )
}

/// Parcours du robot dans le niveau d'essai « horloge » (maladroit, puis précis).
/// Sur la VRAIE simulation, l'aiguille tourne : le robot apprend à l'attendre
/// (son heure est dans la case, voir `aiguille_proche`), à sauter dessus, à se
/// laisser porter (plans « attendre » posé sur elle), puis à sauter plus haut.
pub fn parcours_essai_horloge() -> Vec<Entrees> {
    let i = crate::monde::INDICE_ESSAI_HORLOGE;
    trouver_parcours(i, false)
        .or_else(|| trouver_parcours(i, true))
        .expect("le robot ne trouve aucun parcours au niveau d'essai horloge")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le robot finit aussi le niveau d'essai « vent » (un écran, carte 5.3) sans
    /// mourir, à travers l'`Application` (touche F1 du build de QA). Le plané qui
    /// traverse le vide est porté par le vent latéral du niveau ; la caméra reste
    /// au sol (niveau d'un écran) et les trois gouttes sont ramassées.
    #[test]
    fn le_robot_finit_le_niveau_d_essai_vent() {
        let parcours = parcours_essai_vent();
        let partie = jouer_essai_vent(&parcours);
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
        assert_eq!(
            partie.gouttes, 3,
            "les trois gouttes du niveau d'essai vent"
        );
        assert_eq!(
            partie.images.iter().filter(|i| i.touche_essai).count(),
            1,
            "la touche F1, une seule fois"
        );
        let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
        assert!(essai.len() > 100, "{} images de jeu", essai.len());
        // Niveau d'un écran : la caméra reste au bas du niveau (y = -4 : le niveau de
        // 176 px est aligné au bas de l'écran de 180 px, le ciel remplit le reste).
        assert!(
            essai.iter().all(|i| i.camera.1 == -4),
            "la caméra d'un niveau plat reste au sol"
        );
        // Le robot a bel et bien traversé le vide : il arrive à droite de la zone de
        // vent (x > 400), où se trouve le drapeau.
        let dernier = essai.last().unwrap();
        assert!(
            dernier.trame.x > 400.0,
            "arrivée à x = {} (le drapeau est à droite du vide)",
            dernier.trame.x
        );
        println!(
            "essai vent : {} images de jeu, gouttes {}, fin à x = {:.1}",
            essai.len(),
            partie.gouttes,
            dernier.trame.x
        );
    }

    /// Le robot finit le niveau d'essai « grêle » (carte 5.4) sans mourir, à travers
    /// l'`Application` (touche F2 du build de QA) : trois gouttes (dont celle de la
    /// corniche, SOUS la grêle arrêtée), et il a bel et bien plané DANS la grêle.
    #[test]
    fn le_robot_finit_le_niveau_d_essai_grele() {
        let parcours = parcours_essai_grele();
        let partie = jouer_essai_grele(&parcours);
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
        assert_eq!(
            partie.gouttes, 3,
            "les trois gouttes du niveau d'essai grêle"
        );
        assert_eq!(
            partie.images.iter().filter(|i| i.touche_essai).count(),
            1,
            "la touche F2, une seule fois"
        );
        let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
        assert!(essai.len() > 100, "{} images de jeu", essai.len());
        assert!(
            essai.iter().all(|i| i.camera.1 == -4),
            "la caméra d'un niveau plat reste au sol"
        );
        let sous_la_grele = essai.iter().filter(|i| i.sous_la_grele).count();
        assert!(
            sous_la_grele > 20,
            "{sous_la_grele} images à planer dans la grêle"
        );
        let dernier = essai.last().unwrap();
        assert!(dernier.trame.x > 500.0, "arrivée à x = {}", dernier.trame.x);
        println!(
            "essai grêle : {} images de jeu, {sous_la_grele} sous la grêle, gouttes {}, fin à x = {:.1}",
            essai.len(),
            partie.gouttes,
            dernier.trame.x
        );
    }

    /// Rebonds d'auvent d'une partie d'essai : indices (dans les images de jeu) des
    /// images où un auvent relance Pépin.
    fn rebonds(essai: &[&ImageEssai]) -> Vec<usize> {
        (0..essai.len())
            .filter(|&k| essai[k].rebond_auvent)
            .collect()
    }

    /// Le robot (le plus rapide) finit le niveau d'essai « auvent » (carte 5.5) sans
    /// mourir, à travers l'`Application` (touche F3 du build de QA) : il traverse le
    /// vide par les auvents sans toucher le sol entre son premier et son dernier
    /// rebond (en gardant la droite, il en saute certains : c'est permis).
    #[test]
    fn le_robot_finit_le_niveau_d_essai_auvent() {
        let partie = jouer_essai_auvent(&parcours_essai_auvent());
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
        assert_eq!(
            partie.images.iter().filter(|i| i.touche_essai).count(),
            1,
            "la touche F3, une seule fois"
        );
        let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
        let r = rebonds(&essai);
        assert!(r.len() >= 3, "{} rebonds d'auvent", r.len());
        let (premier, dernier) = (r[0], *r.last().unwrap());
        assert!(
            essai[premier..=dernier].iter().all(|i| !i.au_sol),
            "Pépin a touché le sol entre deux auvents"
        );
        println!(
            "essai auvent (le plus rapide) : {} images de jeu, {} rebonds, gouttes {}",
            essai.len(),
            r.len(),
            partie.gouttes
        );
    }

    /// Rebond par rebond (le parcours des captures) : les SIX auvents, chacun à son
    /// tour, sans toucher le sol du premier au dernier ; trois gouttes (dont celles
    /// en l'air, entre deux rebonds), aucune mort.
    #[test]
    fn le_robot_enchaine_les_six_auvents_de_l_essai() {
        let partie = jouer_essai_auvent(&parcours_essai_auvent_rebond_par_rebond());
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
        assert_eq!(
            partie.gouttes, 3,
            "les trois gouttes du niveau d'essai auvent"
        );
        let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
        let r = rebonds(&essai);
        // Les six auvents (colonnes 9, 13, 17, 22, 25, 29), un rebond chacun, dans
        // l'ordre : l'auvent relanceur est celui que la boîte de Pépin (10 px) touche.
        let colonnes: Vec<i32> = r
            .iter()
            .filter_map(|&k| {
                let x = essai[k].trame.x;
                [9, 13, 17, 22, 25, 29]
                    .into_iter()
                    .find(|&c| x < (c * 16 + 16) as f32 && ((c * 16) as f32) < x + 10.0)
            })
            .collect();
        assert_eq!(
            colonnes,
            vec![9, 13, 17, 22, 25, 29],
            "rebonds aux images {r:?}"
        );
        assert!(
            essai[r[0]..=r[5]].iter().all(|i| !i.au_sol),
            "Pépin a touché le sol entre deux auvents"
        );
        println!(
            "essai auvent (rebond par rebond) : {} images de jeu, rebonds aux images {r:?}",
            essai.len()
        );
    }

    /// Le robot finit le niveau d'essai « ventilateur » (carte 5.6) sans mourir, à
    /// travers l'`Application` (touche F4 du build de QA) : trois gouttes, et il
    /// monte réellement au balcon du drapeau — le seul chemin passe par le souffle
    /// du ventilateur, car le balcon est trop haut pour un saut direct.
    #[test]
    fn le_robot_finit_le_niveau_d_essai_ventilateur() {
        let partie = jouer_essai_ventilateur(&parcours_essai_ventilateur());
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
        assert_eq!(
            partie.gouttes, 3,
            "les trois gouttes du niveau d'essai ventilateur"
        );
        assert_eq!(
            partie.images.iter().filter(|i| i.touche_essai).count(),
            1,
            "la touche F4, une seule fois"
        );
        let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
        assert!(essai.len() > 100, "{} images de jeu", essai.len());
        // Le balcon du drapeau est en HAUT du niveau (y petit). Le robot y est bien
        // arrivé : sa trame finale est nettement plus haute que le sol.
        let dernier = essai.last().unwrap();
        assert!(
            dernier.trame.y < 100.0,
            "arrivée en haut du niveau (y = {:.1}) : le balcon a été atteint",
            dernier.trame.y
        );
        println!(
            "essai ventilateur : {} images de jeu, gouttes {}, fin à x = {:.1}, y = {:.1}",
            essai.len(),
            partie.gouttes,
            dernier.trame.x,
            dernier.trame.y
        );
    }

    /// Le robot finit le niveau d'essai « montgolfière » (carte 5.7) sans mourir, à
    /// travers l'`Application` (touche F5 du build de QA), par le parcours le plus
    /// RAPIDE : trois gouttes, l'arrivée tout en haut (étage du drapeau). Il saute de
    /// la nacelle dès que l'étage suivant est à portée de saut, mais il a dû y monter :
    /// la première l'a porté de plus d'un saut (56 px). Pendant toute montée où il est
    /// posé, ses pieds sont PILE sur le dessus d'une nacelle (porté, sans glisser).
    #[test]
    fn le_robot_finit_le_niveau_d_essai_montgolfiere() {
        let partie = jouer_essai_montgolfiere(&parcours_essai_montgolfiere());
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
        assert_eq!(
            partie.gouttes, 3,
            "les trois gouttes de l'essai montgolfière"
        );
        assert_eq!(
            partie.images.iter().filter(|i| i.touche_essai).count(),
            1,
            "la touche F5, une seule fois"
        );
        let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
        let dernier = essai.last().unwrap();
        assert!(
            dernier.trame.y < 80.0,
            "arrivée à l'étage du drapeau (y = {:.1})",
            dernier.trame.y
        );
        let plus_haut = |k: usize| essai.iter().map(|i| i.nacelles[k]).fold(f32::MAX, f32::min);
        assert!(plus_haut(0) < 256.0 - 48.0, "nacelle 1 : {}", plus_haut(0));
        let portees: Vec<&&ImageEssai> = essai.iter().filter(|i| i.montee && i.au_sol).collect();
        assert!(portees.len() > 60, "{} images portées", portees.len());
        for i in &portees {
            let pieds = i.trame.y + pepin_core::physics::JOUEUR_H;
            assert!(
                i.nacelles.iter().any(|&n| (pieds - n).abs() < 0.01),
                "pieds en y = {pieds}, nacelles en y = {:?}",
                i.nacelles
            );
        }
        println!(
            "essai montgolfière (le plus rapide) : {} images de jeu, {} portées, plus haut : \
             {:.1} et {:.1}",
            essai.len(),
            portees.len(),
            plus_haut(0),
            plus_haut(1)
        );
    }

    /// Le parcours « jusqu'en haut » (celui des captures) : chaque montgolfière monte
    /// JUSQU'EN HAUT puis s'arrête — la première de 8 tuiles (nacelle de y = 256 à
    /// y = 128), la seconde sous le nuage (de y = 144 à y = 64) —, Pépin y est
    /// toujours posé pendant la montée, et la première, quittée, redescend vers son
    /// quai pendant que Pépin monte dans la seconde. Trois gouttes, aucune mort.
    #[test]
    fn le_robot_monte_jusqu_en_haut_des_deux_montgolfieres() {
        let partie = jouer_essai_montgolfiere(&parcours_essai_montgolfiere_jusqu_en_haut());
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
        assert_eq!(partie.gouttes, 3);
        let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
        let dernier = essai.last().unwrap();
        let plus_haut = |k: usize| essai.iter().map(|i| i.nacelles[k]).fold(f32::MAX, f32::min);
        assert_eq!(plus_haut(0), 128.0, "la première monte de 8 tuiles");
        assert_eq!(plus_haut(1), 64.0, "la seconde s'arrête sous le nuage");
        let montees = essai.iter().filter(|i| i.montee).count();
        assert!(
            montees >= 260,
            "{montees} images de montée (128 + 80 px à 0,8 px : 260)"
        );
        // Une montée où Pépin n'est pas posé ne dure qu'une image : celle où il
        // saute de la nacelle (elle montait encore au début du pas). Dès l'image
        // suivante, plus aucune nacelle ne monte.
        for (k, i) in essai.iter().enumerate() {
            if i.montee && !i.au_sol {
                assert!(
                    essai.get(k + 1).is_none_or(|s| !s.montee),
                    "image {k} : Pépin en l'air pendant une montée"
                );
            }
        }
        assert!(
            dernier.nacelles[0] > 128.0 + 20.0,
            "quittée, la première redescend (nacelle en y = {:.1})",
            dernier.nacelles[0]
        );
        println!(
            "essai montgolfière (jusqu'en haut) : {} images de jeu, {montees} de montée, \
             nacelles à la fin en y = {:?}",
            essai.len(),
            dernier.nacelles
        );
    }

    /// Le robot finit le niveau d'essai « escargot » (touche F6 du build de QA) sans
    /// mourir : trois gouttes, l'arrivée en haut de la falaise 2 (y < 48). Il a
    /// rebondi au moins deux fois sur un escargot (une par falaise) ; chaque rebond
    /// relance Pépin vers le haut et aplatit la coquille de l'escargot touché.
    #[test]
    fn le_robot_finit_le_niveau_d_essai_escargot() {
        let partie = jouer_essai_escargot(&parcours_essai_escargot());
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
        assert_eq!(partie.gouttes, 3, "les trois gouttes de l'essai escargot");
        assert_eq!(
            partie.images.iter().filter(|i| i.touche_essai).count(),
            1,
            "la touche F6, une seule fois"
        );
        let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
        let dernier = essai.last().unwrap();
        assert!(
            dernier.trame.y < 48.0,
            "arrivée en haut de la falaise 2 (y = {:.1})",
            dernier.trame.y
        );
        let rebonds: Vec<usize> = (0..essai.len())
            .filter(|&k| essai[k].rebond_escargot)
            .collect();
        assert!(rebonds.len() >= 2, "{} rebond(s)", rebonds.len());
        for &k in &rebonds {
            assert!(
                essai[k].escargots.iter().any(|e| e.2),
                "image {k} : rebond sans coquille aplatie"
            );
        }
        let contre = essai.iter().filter(|i| i.contre_escargot).count();
        println!(
            "essai escargot : {} images de jeu, rebonds aux images {rebonds:?}, {contre} \
             image(s) au sol contre un escargot",
            essai.len()
        );
    }

    /// Le robot finit le niveau d'essai « chat » (touche F7 du build de QA) sans
    /// mourir : trois gouttes (dont la goutte cachée derrière le chat), l'arrivée.
    /// Il a écrasé le chat une fois, PENDANT son sommeil ou son annonce (jamais
    /// réveillé), et il n'est passé sous le grand mur (x > 432) qu'après.
    #[test]
    fn le_robot_finit_le_niveau_d_essai_chat() {
        let partie = jouer_essai_chat(&parcours_essai_chat());
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
        assert_eq!(partie.gouttes, 3, "les trois gouttes de l'essai chat");
        assert_eq!(
            partie.images.iter().filter(|i| i.touche_essai).count(),
            1,
            "la touche F7, une seule fois"
        );
        let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
        let ecrasements: Vec<usize> = (0..essai.len()).filter(|&k| essai[k].ecrase_chat).collect();
        assert_eq!(ecrasements.len(), 1, "un seul écrasement : {ecrasements:?}");
        let k = ecrasements[0];
        // La phase au moment du contact : celle de l'image de l'écrasement (le
        // chat avance d'un pas AVANT les contacts, voir `Monde::avancer`), et un
        // chat écrasé ne change plus de phase. Endormi ou en train de se réveiller.
        let (phase, ecrase) = essai[k].chat.unwrap();
        assert!(
            ecrase && !matches!(phase, PhaseChat::Eveille(_)),
            "{phase:?}"
        );
        // … et jamais de justesse : au moins `MARGE_CHAT_PAS` après qu'il s'est
        // endormi, et avant son réveil.
        let p = pas_du_cycle(phase);
        assert!(
            p >= MARGE_CHAT_PAS && p + MARGE_CHAT_PAS < DORT_PAS + REVEIL_PAS,
            "écrasé de justesse : {phase:?}"
        );
        assert!(
            essai[..k].iter().all(|i| i.trame.x < 432.0),
            "le robot passe sous le mur avant d'avoir écrasé le chat"
        );
        assert!(essai[k..].iter().any(|i| i.trame.x > 432.0));
        // Il est arrivé sur le bloc (dessus à y = 112) pendant l'éveil du chat, et
        // l'y a attendu : posé là-haut, chat réveillé.
        assert!(
            essai[..k].iter().any(|i| {
                i.au_sol
                    && (i.trame.y - 98.0).abs() < 1.0
                    && matches!(i.chat, Some((PhaseChat::Eveille(_), false)))
            }),
            "le robot n'a pas attendu le chat sur le bloc"
        );
        // Le chat s'est-il réveillé pendant que le robot l'approchait ?
        let eveils = essai[..k]
            .windows(2)
            .filter(|w| {
                matches!(w[1].chat, Some((PhaseChat::Eveille(_), false)))
                    && !matches!(w[0].chat, Some((PhaseChat::Eveille(_), _)))
            })
            .count();
        println!(
            "essai chat : {} images de jeu, chat écrasé à l'image {k} ({phase:?}), \
             {eveils} réveil(s) avant",
            essai.len()
        );
    }

    /// Le robot finit le niveau d'essai « horloge » (touche F8) sans mourir, avec
    /// les trois gouttes, porté par les DEUX aiguilles : il monte sur chacune (au
    /// moins une tuile de montée pendant qu'elle le porte) avant de sauter plus haut.
    #[test]
    fn le_robot_finit_le_niveau_d_essai_horloge() {
        let partie = jouer_essai_horloge(&parcours_essai_horloge());
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
        assert_eq!(partie.gouttes, 3, "les trois gouttes de l'essai horloge");
        assert_eq!(
            partie.images.iter().filter(|i| i.touche_essai).count(),
            1,
            "la touche F8, une seule fois"
        );
        let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
        for k in 0..2 {
            let portees: Vec<f32> = essai
                .iter()
                .filter(|i| i.sur_aiguille == Some(k))
                .map(|i| i.trame.y)
                .collect();
            assert!(!portees.is_empty(), "jamais porté par l'aiguille {k}");
            let montee = portees.iter().fold(f32::MIN, |a, &y| a.max(y))
                - portees.iter().fold(f32::MAX, |a, &y| a.min(y));
            println!(
                "essai horloge : aiguille {k}, {} images portées, {montee:.1} px de montée",
                portees.len()
            );
            assert!(montee >= 16.0, "aiguille {k} : {montee} px");
        }
        // Porté, Pépin a les pieds pile sur le dessus de son aiguille.
        for i in essai.iter().filter(|i| i.sur_aiguille.is_some()) {
            let k = i.sur_aiguille.unwrap();
            assert!((i.trame.y + 14.0 - i.aiguilles[k].1).abs() < 0.01);
        }
        println!("essai horloge : {} images de jeu", essai.len());
    }

    /// Le robot bat le Grand Nuage (arène, touche F10) sans mourir : trois coups sur
    /// sa tête, chacun PENDANT un essoufflement, le ciel qui se déchire, puis l'écran
    /// de fin ; les trois gouttes. Ses points de vie descendent de 3 à 0, un coup à la
    /// fois, et il a traversé ses trois motifs d'éclairs.
    #[test]
    fn le_robot_bat_le_grand_nuage() {
        let partie = jouer_essai_nuage(&parcours_essai_nuage());
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
        assert_eq!(partie.gouttes, 3, "les trois gouttes de l'arène");
        assert_eq!(
            partie.images.iter().filter(|i| i.touche_essai).count(),
            1,
            "la touche F10, une seule fois"
        );
        let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
        let coups: Vec<usize> = (0..essai.len()).filter(|&k| essai[k].coup_nuage).collect();
        assert_eq!(coups.len(), 3, "trois coups : {coups:?}");
        for (n, &k) in coups.iter().enumerate() {
            // Juste avant le coup, le nuage était essoufflé, avec `3 - n` points.
            let (phase, _, pv) = essai[k - 1].nuage.unwrap();
            assert_eq!(
                (phase, pv),
                (PhaseNuage::Essouffle, 3 - n as u8),
                "coup {n}"
            );
            let (apres, _, pv) = essai[k].nuage.unwrap();
            assert_eq!(pv, 2 - n as u8);
            let attendu = if n == 2 {
                PhaseNuage::Dechirure
            } else {
                PhaseNuage::Touche
            };
            assert_eq!(apres, attendu);
        }
        // Les trois motifs ont été joués (une colère à 3, à 2, puis à 1 point).
        for pv in 1..=3 {
            assert!(
                essai
                    .iter()
                    .any(|i| matches!(i.nuage, Some((PhaseNuage::Colere, _, p)) if p == pv)),
                "aucune colère à {pv} point(s)"
            );
        }
        // Le ciel s'est déchiré jusqu'au bout (120 images après le dernier coup),
        // puis le nuage est vaincu et la partie passe à l'écran de fin.
        let dernier = coups[2];
        assert!(essai[dernier..]
            .iter()
            .all(|i| matches!(i.nuage, Some((PhaseNuage::Dechirure, _, 0)))));
        assert_eq!(essai.len() - dernier, 120, "déchirure : 2 s");
        assert!(partie
            .images
            .iter()
            .any(|i| matches!(i.nuage, Some((PhaseNuage::Vaincu, _, 0)))));
        println!(
            "essai nuage : {} images de jeu, coups aux images {coups:?}",
            essai.len()
        );
    }

    /// Le parcours de l'arène tient si le monde a une ou deux images d'avance ou de
    /// retard sur les touches (comme celui de l'horloge) : nuage vaincu, 0 mort, 3
    /// gouttes à chaque fois.
    #[test]
    fn le_parcours_du_grand_nuage_tient_a_une_image_pres() {
        let parcours = parcours_essai_nuage();
        for decalage in [-2, -1, 0, 1, 2] {
            let (fin, morts, gouttes) =
                rejouer_decale(crate::monde::INDICE_ESSAI_NUAGE, &parcours, decalage);
            assert!(
                fin && morts == 0 && gouttes == 3,
                "décalage {decalage} : fin {fin}, {morts} mort(s), {gouttes} goutte(s)"
            );
        }
    }

    /// Rejoue `parcours` dans le niveau `indice` avec les entrées décalées de
    /// `decalage` images par rapport au monde (positif : Pépin agit plus tard ;
    /// négatif : plus tôt), puis 60 images sans rien toucher (comme le navigateur à
    /// la fin du script). Renvoie (drapeau atteint, morts, gouttes du niveau).
    fn rejouer_decale(indice: usize, parcours: &[Entrees], decalage: i32) -> (bool, u32, usize) {
        let mut p = parcours.to_vec();
        if decalage > 0 {
            p.splice(
                0..0,
                std::iter::repeat_n(Entrees::default(), decalage as usize),
            );
        } else {
            // Le parcours commence par l'attente du départ : on n'y retire que du vide.
            assert!(p[..(-decalage) as usize]
                .iter()
                .all(|e| *e == Entrees::default()));
            p.drain(0..(-decalage) as usize);
        }
        p.extend(std::iter::repeat_n(Entrees::default(), 60));
        let mut m = Monde::new(indice);
        for e in &p {
            m.avancer(e);
            if m.termine {
                break;
            }
        }
        let gouttes = m.gouttes_niveau.iter().filter(|&&g| g).count();
        (m.termine, m.etat.morts, gouttes)
    }

    /// Le parcours de l'essai horloge tient si le monde a une ou deux images
    /// d'avance ou de retard sur les touches (le rejeu dans le navigateur peut
    /// démarrer le niveau d'essai une image plus tôt ou plus tard) : drapeau, 0
    /// mort, 3 gouttes à chaque fois. C'est ce que garantit `tient_au_decalage`.
    #[test]
    fn le_parcours_de_l_horloge_tient_a_une_image_pres() {
        let parcours = parcours_essai_horloge();
        for decalage in [-2, -1, 0, 1, 2] {
            let (fin, morts, gouttes) =
                rejouer_decale(crate::monde::INDICE_ESSAI_HORLOGE, &parcours, decalage);
            assert!(
                fin && morts == 0 && gouttes == 3,
                "décalage {decalage} : drapeau {fin}, {morts} mort(s), {gouttes} goutte(s)"
            );
        }
    }

    /// Le robot finit la partie sans mourir, et joue les niveaux dans l'ordre que
    /// donne `levels::apres` avec SES gouttes (bonus sauté s'il en manque).
    #[test]
    fn le_robot_finit_tous_les_niveaux_sans_mourir() {
        use pepin_core::levels::{apres, Apres, NIVEAUX};
        let parcours = tous_les_parcours();
        let partie = jouer_tout(&parcours);
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
        // Niveaux joués (numéros), dans l'ordre, et gouttes à la fin de chacun.
        let mut joues: Vec<(u8, u32)> = Vec::new();
        for t in partie.trames.iter().filter(|t| t.niveau > 0) {
            match joues.last_mut() {
                Some((n, g)) if *n == t.niveau => *g = t.gouttes,
                _ => joues.push((t.niveau, t.gouttes)),
            }
        }
        let mut attendu = vec![1u8];
        for &(n, g) in &joues {
            if let Apres::Niveau(j) = apres(NIVEAUX, n as usize - 1, g) {
                attendu.push(j as u8 + 1);
            }
        }
        let numeros: Vec<u8> = joues.iter().map(|&(n, _)| n).collect();
        assert_eq!(numeros, attendu);
    }

    /// Le robot finit aussi le niveau d'essai vertical (45 rangées) sans mourir, à travers
    /// l'`Application` (touche 0 du build de QA). Pendant tout le parcours, la caméra suit
    /// Pépin en hauteur : elle part de sa borne basse, monte jusqu'en haut du niveau et
    /// redescend ; Pépin est TOUJOURS entier à l'écran, sauf quand il est lancé au-dessus
    /// du niveau, et alors la flèche de repère est là (et seulement alors).
    #[test]
    fn le_robot_finit_le_niveau_d_essai_vertical() {
        let parcours = parcours_essai_vertical();
        let partie = jouer_essai_vertical(&parcours);
        assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
        assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
        assert_eq!(
            partie.images.iter().filter(|i| i.touche_essai).count(),
            1,
            "la touche 0, une seule fois"
        );
        let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
        assert!(essai.len() > 300, "{} images de jeu", essai.len());
        let ys: Vec<i32> = essai.iter().map(|i| i.camera.1).collect();
        let (haut, bas) = (*ys.iter().min().unwrap(), *ys.iter().max().unwrap());
        assert_eq!((haut, bas), (0, 540), "la caméra parcourt toute la hauteur");
        assert_eq!(ys[0], 540, "départ en bas");
        let sommet = ys.iter().position(|&y| y == 0).unwrap();
        assert!(ys[sommet..].contains(&540), "puis redescend jusqu'en bas");
        let mut reperes = 0;
        for (k, i) in essai.iter().enumerate() {
            let (y, h) = (i.pepin_ecran_y, 14);
            assert!(y + h <= 180, "image {k} : Pépin sous l'écran (y = {y})");
            if i.camera.1 > 0 {
                // Caméra sous sa borne haute : Pépin est entier à l'écran, sans flèche.
                assert!(
                    y >= 0 && !i.repere,
                    "image {k} : Pépin hors de l'écran (y = {y}, caméra {:?})",
                    i.camera
                );
            } else {
                // Caméra en haut du niveau : la flèche apparaît dès que Pépin est
                // entièrement au-dessus du cadre, jamais avant.
                assert_eq!(i.repere, y + h <= 0, "image {k} : y = {y}");
            }
            reperes += usize::from(i.repere);
        }
        assert!(reperes > 0, "le champignon lance Pépin au-dessus du niveau");
        println!(
            "essai vertical : {} images de jeu, {reperes} avec la flèche, gouttes {}",
            essai.len(),
            partie.gouttes
        );
    }

    /// Écrit le script du niveau d'essai vertical pour le navigateur (build de QA) :
    /// `target/qa/script-52.txt` (un chiffre hexadécimal par image, comme `script.txt`),
    /// `target/qa/touches-52.txt` (image du pilote où appuyer sur la touche 0) et
    /// `target/qa/trace-52.csv` (position de Pépin et caméra rendue, image par image).
    /// Dans le navigateur, le jeu a un pas de retard sur la numérotation du pilote (déjà
    /// noté à la carte 3.1b, voir QA.md) : sans effet sur des touches tenues plusieurs
    /// images, mais une touche d'UNE image doit tomber une image plus tard que dans la
    /// trace, sinon elle arrive pendant la dernière image de la carte du niveau 1 et le jeu
    /// l'ignore (vérifié : à l'image 228, rien ; à 229, la carte « NIVEAU D'ESSAI »).
    /// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai
    #[test]
    #[ignore]
    fn ecrire_script_essai_vertical() {
        let partie = jouer_essai_vertical(&parcours_essai_vertical());
        assert!(partie.fin_atteinte && partie.morts == 0);
        std::fs::create_dir_all("../target/qa").unwrap();
        let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
        std::fs::write("../target/qa/script-52.txt", &script).unwrap();
        let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
        let pilote = touche + 1;
        std::fs::write("../target/qa/touches-52.txt", format!("{pilote}:Digit0\n")).unwrap();
        let mut trace = String::from("image,ecran,essai,x,y,cam_x,cam_y,pepin_ecran_y,repere\n");
        for (k, i) in partie.images.iter().enumerate() {
            trace.push_str(&format!(
                "{k},{},{},{:.1},{:.1},{},{},{},{}\n",
                i.trame.ecran,
                u8::from(i.en_essai),
                i.trame.x,
                i.trame.y,
                i.camera.0,
                i.camera.1,
                i.pepin_ecran_y,
                u8::from(i.repere)
            ));
        }
        std::fs::write("../target/qa/trace-52.csv", trace).unwrap();
        println!(
            "images : {} ; touche 0 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
            partie.images.len(),
            partie.gouttes
        );
    }

    /// Écrit le script du niveau d'essai « vent » pour le navigateur (build de QA) :
    /// `target/qa/script-53.txt`, `target/qa/touches-53.txt` (image du pilote où
    /// appuyer sur F1) et `target/qa/trace-53.csv` (position de Pépin et caméra
    /// rendue, image par image). Comme la carte 5.2, la touche du pilote tombe une
    /// image plus tard que dans la trace (le navigateur a un pas de retard).
    /// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_vent
    #[test]
    #[ignore]
    fn ecrire_script_essai_vent() {
        let partie = jouer_essai_vent(&parcours_essai_vent());
        assert!(partie.fin_atteinte && partie.morts == 0);
        std::fs::create_dir_all("../target/qa").unwrap();
        let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
        std::fs::write("../target/qa/script-53.txt", &script).unwrap();
        let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
        let pilote = touche + 1;
        std::fs::write("../target/qa/touches-53.txt", format!("{pilote}:F1\n")).unwrap();
        let mut trace = String::from("image,ecran,essai,x,y,cam_x,cam_y\n");
        for (k, i) in partie.images.iter().enumerate() {
            trace.push_str(&format!(
                "{k},{},{},{:.1},{:.1},{},{}\n",
                i.trame.ecran,
                u8::from(i.en_essai),
                i.trame.x,
                i.trame.y,
                i.camera.0,
                i.camera.1
            ));
        }
        std::fs::write("../target/qa/trace-53.csv", trace).unwrap();
        println!(
            "images : {} ; F1 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
            partie.images.len(),
            partie.gouttes
        );
    }

    /// Écrit le script du niveau d'essai « grêle » pour le navigateur (build de QA) :
    /// `target/qa/script-54.txt`, `target/qa/touches-54.txt` (image du pilote où
    /// appuyer sur F2, une image après la trace : le navigateur a un pas de retard) et
    /// `target/qa/trace-54.csv` (position de Pépin, caméra, plané dans la grêle).
    /// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_grele
    #[test]
    #[ignore]
    fn ecrire_script_essai_grele() {
        let partie = jouer_essai_grele(&parcours_essai_grele());
        assert!(partie.fin_atteinte && partie.morts == 0);
        std::fs::create_dir_all("../target/qa").unwrap();
        let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
        std::fs::write("../target/qa/script-54.txt", &script).unwrap();
        let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
        let pilote = touche + 1;
        std::fs::write("../target/qa/touches-54.txt", format!("{pilote}:F2\n")).unwrap();
        let mut trace = String::from("image,ecran,essai,x,y,cam_x,cam_y,sous_la_grele\n");
        for (k, i) in partie.images.iter().enumerate() {
            trace.push_str(&format!(
                "{k},{},{},{:.1},{:.1},{},{},{}\n",
                i.trame.ecran,
                u8::from(i.en_essai),
                i.trame.x,
                i.trame.y,
                i.camera.0,
                i.camera.1,
                u8::from(i.sous_la_grele)
            ));
        }
        std::fs::write("../target/qa/trace-54.csv", trace).unwrap();
        println!(
            "images : {} ; F2 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
            partie.images.len(),
            partie.gouttes
        );
    }

    /// Écrit le script du niveau d'essai « auvent » (parcours rebond par rebond : les
    /// six auvents à la suite) pour le navigateur :
    /// `target/qa/script-55.txt`, `target/qa/touches-55.txt` (image du pilote où
    /// appuyer sur F3, une image après la trace : le navigateur a un pas de retard) et
    /// `target/qa/trace-55.csv` (position de Pépin, caméra, rebonds d'auvent, sol).
    /// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_auvent
    #[test]
    #[ignore]
    fn ecrire_script_essai_auvent() {
        let partie = jouer_essai_auvent(&parcours_essai_auvent_rebond_par_rebond());
        assert!(partie.fin_atteinte && partie.morts == 0);
        std::fs::create_dir_all("../target/qa").unwrap();
        let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
        std::fs::write("../target/qa/script-55.txt", &script).unwrap();
        let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
        let pilote = touche + 1;
        std::fs::write("../target/qa/touches-55.txt", format!("{pilote}:F3\n")).unwrap();
        let mut trace = String::from("image,ecran,essai,x,y,cam_x,cam_y,rebond_auvent,au_sol\n");
        for (k, i) in partie.images.iter().enumerate() {
            trace.push_str(&format!(
                "{k},{},{},{:.1},{:.1},{},{},{},{}\n",
                i.trame.ecran,
                u8::from(i.en_essai),
                i.trame.x,
                i.trame.y,
                i.camera.0,
                i.camera.1,
                u8::from(i.rebond_auvent),
                u8::from(i.au_sol)
            ));
        }
        std::fs::write("../target/qa/trace-55.csv", trace).unwrap();
        println!(
            "images : {} ; F3 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
            partie.images.len(),
            partie.gouttes
        );
    }

    /// Écrit le script du niveau d'essai « ventilateur » (parcours du robot) pour le
    /// navigateur : `target/qa/script-56b.txt`, `target/qa/touches-56b.txt` (image du
    /// pilote où appuyer sur F4, une image après la trace : le navigateur a un pas de
    /// retard) et `target/qa/trace-56b.csv` (position de Pépin, caméra, sol, phase du
    /// ventilateur et pas de souffle restants) : de quoi choisir les images à capturer
    /// pendant le souffle, juste après l'arrêt et pendant l'annonce (carte 5.6b).
    /// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_ventilateur
    #[test]
    #[ignore]
    fn ecrire_script_essai_ventilateur() {
        let partie = jouer_essai_ventilateur(&parcours_essai_ventilateur());
        assert!(partie.fin_atteinte && partie.morts == 0);
        std::fs::create_dir_all("../target/qa").unwrap();
        let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
        std::fs::write("../target/qa/script-56b.txt", &script).unwrap();
        let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
        let pilote = touche + 1;
        std::fs::write("../target/qa/touches-56b.txt", format!("{pilote}:F4\n")).unwrap();
        let mut trace =
            String::from("image,ecran,essai,x,y,cam_x,cam_y,au_sol,phase,souffle_restant\n");
        for (k, i) in partie.images.iter().enumerate() {
            let (phase, restant) = match i.ventilateur {
                Some((PhaseVentilateur::Arret, _)) => ("arret", 0),
                Some((PhaseVentilateur::Annonce, _)) => ("annonce", 0),
                Some((PhaseVentilateur::Souffle, r)) => ("souffle", r),
                None => ("-", 0),
            };
            trace.push_str(&format!(
                "{k},{},{},{:.1},{:.1},{},{},{},{phase},{restant}\n",
                i.trame.ecran,
                u8::from(i.en_essai),
                i.trame.x,
                i.trame.y,
                i.camera.0,
                i.camera.1,
                u8::from(i.au_sol)
            ));
        }
        std::fs::write("../target/qa/trace-56b.csv", trace).unwrap();
        println!(
            "images : {} ; F4 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
            partie.images.len(),
            partie.gouttes
        );
    }

    /// Démonstration de la carte 5.6b : Pépin posé sur la casette pendant toute
    /// l'annonce, porté par le souffle suivant jusqu'en haut de la colonne, puis
    /// redescendu à l'arrêt, sans mourir.
    #[test]
    fn pepin_attend_sur_la_casette_puis_le_souffle_le_porte() {
        let parcours =
            parcours_essai_ventilateur_caisse().expect("un saut pose Pépin sur la casette");
        let partie = jouer_essai_ventilateur(&parcours);
        let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
        let phase = |i: &ImageEssai| i.ventilateur.map(|v| v.0);
        // L'annonce qui suit la montée sur la casette : Pépin y est posé.
        let fin_elan = parcours.len();
        let annonce: Vec<&&ImageEssai> = essai
            .iter()
            .skip(fin_elan)
            .take_while(|i| phase(i) != Some(PhaseVentilateur::Souffle))
            .filter(|i| phase(i) == Some(PhaseVentilateur::Annonce))
            .collect();
        assert!(!annonce.is_empty(), "une annonce après l'élan");
        assert!(annonce
            .iter()
            .all(|i| i.au_sol && (i.trame.y - 298.0).abs() < 0.5));
        // Le souffle suivant le porte haut (au moins 10 tuiles), puis, à l'arrêt, il
        // retombe sur la casette.
        let (k_haut, haut) = essai
            .iter()
            .enumerate()
            .skip(fin_elan)
            .map(|(k, i)| (k, i.trame.y))
            .fold((0, f32::MAX), |a, b| if b.1 < a.1 { b } else { a });
        assert!(haut < 298.0 - 160.0, "porté jusqu'à y = {haut}");
        assert!(
            essai[k_haut..]
                .iter()
                .any(|i| i.au_sol && (i.trame.y - 298.0).abs() < 0.5),
            "Pépin retombe sur la casette"
        );
        assert_eq!(partie.morts, 0);
    }

    /// Écrit le script de la démonstration « Pépin sur la casette » (carte 5.6b) :
    /// `target/qa/script-56b-caisse.txt` (1 200 images : la montée sur la casette
    /// puis deux cycles du ventilateur) et `target/qa/trace-56b-caisse.csv` (même
    /// format que `trace-56b.csv`). Touche F4 : la même image que
    /// `touches-56b.txt` (les images avant le niveau d'essai sont les mêmes).
    /// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_ventilateur_caisse
    #[test]
    #[ignore]
    fn ecrire_script_essai_ventilateur_caisse() {
        let parcours =
            parcours_essai_ventilateur_caisse().expect("un saut pose Pépin sur la casette");
        let partie = jouer_essai_ventilateur(&parcours);
        let images = &partie.images[..partie.images.len().min(1200)];
        std::fs::create_dir_all("../target/qa").unwrap();
        let script: String = images.iter().map(|i| i.trame.code()).collect();
        std::fs::write("../target/qa/script-56b-caisse.txt", &script).unwrap();
        let mut trace =
            String::from("image,ecran,essai,x,y,cam_x,cam_y,au_sol,phase,souffle_restant\n");
        for (k, i) in images.iter().enumerate() {
            let (phase, restant) = match i.ventilateur {
                Some((PhaseVentilateur::Arret, _)) => ("arret", 0),
                Some((PhaseVentilateur::Annonce, _)) => ("annonce", 0),
                Some((PhaseVentilateur::Souffle, r)) => ("souffle", r),
                None => ("-", 0),
            };
            trace.push_str(&format!(
                "{k},{},{},{:.1},{:.1},{},{},{},{phase},{restant}\n",
                i.trame.ecran,
                u8::from(i.en_essai),
                i.trame.x,
                i.trame.y,
                i.camera.0,
                i.camera.1,
                u8::from(i.au_sol)
            ));
        }
        std::fs::write("../target/qa/trace-56b-caisse.csv", trace).unwrap();
        let touche = images.iter().position(|i| i.touche_essai).unwrap();
        println!(
            "images : {} ; élan : {} pas ; F4 à l'image {touche} (pilote : {})",
            images.len(),
            parcours.len(),
            touche + 1
        );
    }

    /// Écrit le script du niveau d'essai « montgolfière » (parcours jusqu'en haut :
    /// les deux montées complètes) pour le navigateur :
    /// `target/qa/script-57.txt`, `target/qa/touches-57.txt` (image du pilote où
    /// appuyer sur F5, une image après la trace : le navigateur a un pas de retard) et
    /// `target/qa/trace-57.csv` (position de Pépin, caméra, sol, nacelles, montée).
    /// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_montgolfiere
    #[test]
    #[ignore]
    fn ecrire_script_essai_montgolfiere() {
        let partie = jouer_essai_montgolfiere(&parcours_essai_montgolfiere_jusqu_en_haut());
        assert!(partie.fin_atteinte && partie.morts == 0);
        std::fs::create_dir_all("../target/qa").unwrap();
        let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
        std::fs::write("../target/qa/script-57.txt", &script).unwrap();
        let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
        let pilote = touche + 1;
        std::fs::write("../target/qa/touches-57.txt", format!("{pilote}:F5\n")).unwrap();
        let mut trace =
            String::from("image,ecran,essai,x,y,cam_x,cam_y,au_sol,nacelle1,nacelle2,montee\n");
        for (k, i) in partie.images.iter().enumerate() {
            trace.push_str(&format!(
                "{k},{},{},{:.1},{:.1},{},{},{},{:.1},{:.1},{}\n",
                i.trame.ecran,
                u8::from(i.en_essai),
                i.trame.x,
                i.trame.y,
                i.camera.0,
                i.camera.1,
                u8::from(i.au_sol),
                i.nacelles[0],
                i.nacelles[1],
                u8::from(i.montee)
            ));
        }
        std::fs::write("../target/qa/trace-57.csv", trace).unwrap();
        println!(
            "images : {} ; F5 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
            partie.images.len(),
            partie.gouttes
        );
    }

    /// Écrit le script du niveau d'essai « escargot » pour les captures de la carte
    /// 5.8 : `target/qa/script-58.txt`, `target/qa/touches-58.txt` (l'image où
    /// appuyer sur F6, une image après la trace : le navigateur a un pas de retard) et
    /// `target/qa/trace-58.csv` (position de Pépin, caméra, sol, escargots, rebond,
    /// contact de côté).
    /// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_escargot
    #[test]
    #[ignore]
    fn ecrire_script_essai_escargot() {
        let partie = jouer_essai_escargot(&parcours_essai_escargot());
        assert!(partie.fin_atteinte && partie.morts == 0);
        std::fs::create_dir_all("../target/qa").unwrap();
        let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
        std::fs::write("../target/qa/script-58.txt", &script).unwrap();
        let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
        let pilote = touche + 1;
        std::fs::write("../target/qa/touches-58.txt", format!("{pilote}:F6\n")).unwrap();
        let mut trace = String::from(
            "image,ecran,essai,x,y,cam_x,cam_y,au_sol,escargot1_x,escargot1_aplati,\
             escargot2_x,escargot2_aplati,rebond,contre\n",
        );
        for (k, i) in partie.images.iter().enumerate() {
            let [e1, e2] = i.escargots;
            trace.push_str(&format!(
                "{k},{},{},{:.1},{:.1},{},{},{},{:.1},{},{:.1},{},{},{}\n",
                i.trame.ecran,
                u8::from(i.en_essai),
                i.trame.x,
                i.trame.y,
                i.camera.0,
                i.camera.1,
                u8::from(i.au_sol),
                e1.0,
                u8::from(e1.2),
                e2.0,
                u8::from(e2.2),
                u8::from(i.rebond_escargot),
                u8::from(i.contre_escargot)
            ));
        }
        std::fs::write("../target/qa/trace-58.csv", trace).unwrap();
        println!(
            "images : {} ; F6 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
            partie.images.len(),
            partie.gouttes
        );
    }

    /// Écrit le script du niveau d'essai « chat » pour les captures de la carte 5.9 :
    /// `target/qa/script-59.txt`, `target/qa/touches-59.txt` (l'image où appuyer
    /// sur F7, une image après la trace : le navigateur a un pas de retard) et
    /// `target/qa/trace-59.csv` (position de Pépin, caméra, sol, phase du chat,
    /// pas restants dans la phase, chat écrasé, écrasement).
    /// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_chat
    #[test]
    #[ignore]
    fn ecrire_script_essai_chat() {
        let partie = jouer_essai_chat(&parcours_essai_chat());
        assert!(partie.fin_atteinte && partie.morts == 0);
        std::fs::create_dir_all("../target/qa").unwrap();
        let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
        std::fs::write("../target/qa/script-59.txt", &script).unwrap();
        let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
        let pilote = touche + 1;
        std::fs::write("../target/qa/touches-59.txt", format!("{pilote}:F7\n")).unwrap();
        let mut trace = String::from(
            "image,ecran,essai,x,y,cam_x,cam_y,au_sol,gouttes,chat_phase,chat_reste,\
             chat_ecrase,ecrasement\n",
        );
        for (k, i) in partie.images.iter().enumerate() {
            let (phase, reste, ecrase) = match i.chat {
                Some((PhaseChat::Dort(n), e)) => ("dort", n, e),
                Some((PhaseChat::Reveil(n), e)) => ("reveil", n, e),
                Some((PhaseChat::Eveille(n), e)) => ("eveille", n, e),
                None => ("-", 0, false),
            };
            trace.push_str(&format!(
                "{k},{},{},{:.1},{:.1},{},{},{},{},{phase},{reste},{},{}\n",
                i.trame.ecran,
                u8::from(i.en_essai),
                i.trame.x,
                i.trame.y,
                i.camera.0,
                i.camera.1,
                u8::from(i.au_sol),
                i.trame.gouttes,
                u8::from(ecrase),
                u8::from(i.ecrase_chat)
            ));
        }
        std::fs::write("../target/qa/trace-59.csv", trace).unwrap();
        println!(
            "images : {} ; F7 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
            partie.images.len(),
            partie.gouttes
        );
    }

    /// Écrit le script du niveau d'essai « horloge » pour le navigateur :
    /// `target/qa/script-510.txt` (un chiffre hexadécimal par image),
    /// `target/qa/touches-510.txt` (F8 à l'image où le robot l'appuie) et
    /// `target/qa/trace-510.csv` (position de Pépin, des deux aiguilles, et
    /// l'aiguille qui le porte), pour choisir les images à photographier.
    /// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_horloge
    #[test]
    #[ignore]
    fn ecrire_script_essai_horloge() {
        let partie = jouer_essai_horloge(&parcours_essai_horloge());
        assert!(partie.fin_atteinte && partie.morts == 0);
        std::fs::create_dir_all("../target/qa").unwrap();
        let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
        std::fs::write("../target/qa/script-510.txt", &script).unwrap();
        let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
        let pilote = touche + 1;
        std::fs::write("../target/qa/touches-510.txt", format!("{pilote}:F8\n")).unwrap();
        let mut trace = String::from(
            "image,ecran,essai,x,y,cam_x,cam_y,au_sol,gouttes,a0_x,a0_y,a1_x,a1_y,porte\n",
        );
        for (k, i) in partie.images.iter().enumerate() {
            let porte = i.sur_aiguille.map_or(-1, |a| a as i32);
            trace.push_str(&format!(
                "{k},{},{},{:.1},{:.1},{},{},{},{},{:.1},{:.1},{:.1},{:.1},{porte}\n",
                i.trame.ecran,
                u8::from(i.en_essai),
                i.trame.x,
                i.trame.y,
                i.camera.0,
                i.camera.1,
                u8::from(i.au_sol),
                i.trame.gouttes,
                i.aiguilles[0].0,
                i.aiguilles[0].1,
                i.aiguilles[1].0,
                i.aiguilles[1].1,
            ));
        }
        std::fs::write("../target/qa/trace-510.csv", trace).unwrap();
        println!(
            "images : {} ; F8 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
            partie.images.len(),
            partie.gouttes
        );
    }

    /// Arène du Grand Nuage (carte 5.11) : écrit `target/qa/script-511.txt` (touches
    /// du robot, une par image), `target/qa/touches-511.txt` (F10 à l'image du
    /// pilote) et `target/qa/trace-511.csv` (position de Pépin, phase, temps et
    /// points de vie du nuage, coups), pour rejouer la partie dans un navigateur et
    /// viser les captures. Lancer avec : cargo test -p game --no-default-features --
    /// --ignored ecrire_script_essai_nuage
    #[test]
    #[ignore]
    fn ecrire_script_essai_nuage() {
        let partie = jouer_essai_nuage(&parcours_essai_nuage());
        assert!(partie.fin_atteinte && partie.morts == 0 && partie.gouttes == 3);
        std::fs::create_dir_all("../target/qa").unwrap();
        let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
        std::fs::write("../target/qa/script-511.txt", &script).unwrap();
        let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
        let pilote = touche + 1;
        std::fs::write("../target/qa/touches-511.txt", format!("{pilote}:F10\n")).unwrap();
        let mut trace =
            String::from("image,ecran,essai,x,y,cam_x,cam_y,au_sol,gouttes,phase,ecoule,pv,coup\n");
        for (k, i) in partie.images.iter().enumerate() {
            let (phase, ecoule, pv) = i.nuage.map_or(("-".to_string(), 0, 0), |(p, e, v)| {
                (format!("{p:?}"), e, v)
            });
            trace.push_str(&format!(
                "{k},{},{},{:.1},{:.1},{},{},{},{},{phase},{ecoule},{pv},{}\n",
                i.trame.ecran,
                u8::from(i.en_essai),
                i.trame.x,
                i.trame.y,
                i.camera.0,
                i.camera.1,
                u8::from(i.au_sol),
                i.trame.gouttes,
                u8::from(i.coup_nuage),
            ));
        }
        std::fs::write("../target/qa/trace-511.csv", trace).unwrap();
        println!(
            "images : {} ; F10 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
            partie.images.len(),
            partie.gouttes
        );
    }

    /// Écrit le script des touches pour le navigateur : `target/qa/script.txt`
    /// (un chiffre hexadécimal par image) et `target/qa/trace.csv`.
    /// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script
    #[test]
    #[ignore]
    fn ecrire_script_navigateur() {
        let parcours = tous_les_parcours();
        let partie = jouer_tout(&parcours);
        assert!(partie.fin_atteinte);
        std::fs::create_dir_all("../target/qa").unwrap();
        let script: String = partie.trames.iter().map(Trame::code).collect();
        std::fs::write("../target/qa/script.txt", &script).unwrap();
        let mut trace = String::from(
            "image,ecran,niveau,x,y,corbeau_x,corbeau_y,mobile_x,mobile_y,\
             pigeon_x,pigeon_y,pigeon_phase\n",
        );
        for (i, t) in partie.trames.iter().enumerate() {
            let (cx, cy) = t.corbeau.unwrap_or((-1.0, -1.0));
            let (mx, my) = t.mobile.unwrap_or((-1.0, -1.0));
            let (px, py, phase) = t.pigeon.unwrap_or((-1.0, -1.0, "-"));
            trace.push_str(&format!(
                "{},{},{},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{}\n",
                i, t.ecran, t.niveau, t.x, t.y, cx, cy, mx, my, px, py, phase
            ));
        }
        std::fs::write("../target/qa/trace.csv", trace).unwrap();
        println!(
            "images : {} ({:.1} s) ; morts : {} ; gouttes : {} ; score : {} ; chrono : {} pas ; \
             empreinte : {:016x}",
            partie.trames.len(),
            partie.trames.len() as f32 / 60.0,
            partie.morts,
            partie.gouttes,
            partie.score,
            partie.chrono,
            empreinte(&partie.trames)
        );
    }
}
