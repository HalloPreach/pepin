//! Le « cerveau » du robot de QA : les plans et la recherche de plus court
//! chemin qui fait finir chaque niveau sans mourir.
//!
//! Principe : Dijkstra (sur la durée, avec `BinaryHeap` + `Reverse` : le tas de
//! Rust sort le PLUS GRAND d'abord) entre des états « posés et immobiles »,
//! ou « lancés » (un auvent vient de relancer Pépin, il choisit de nouveau en
//! l'air, voir `est_lance`). Chaque plan (courir, sauter, planer, attendre) est
//! simulé sur une COPIE du monde (`Clone` d'une structure complète pour explorer
//! des futurs possibles sans toucher au présent) : un plan qui mène à la mort
//! est simplement écarté, donc le parcours trouvé est sûr par construction.
//!
//! Les états s'apparentent aux « cases » du niveau : deux états proches
//! fusionnent en la même case (`CaseParcours`, via `cle`), la mémoire de la
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
//!   (phase, temps) se résume en un ENTIER, 0 dans un niveau sans la
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

use pepin_core::aabb::Aabb;
use pepin_core::entities::contacts::Contact;
use pepin_core::entities::ennemi::{
    pas_du_cycle, EtatEnnemi, ANIMATION_ECRASE_PAS, DORT_PAS, REVEIL_PAS,
};
use pepin_core::entities::{eaux_des_entites, Entite, Maree, TypeEnnemi};
use pepin_core::faisabilite::{eau_sous_les_pieds, pas_max_plan, MARGE_BAS, MARGE_PLANE};
use pepin_core::player::{Direction, Entrees};

use crate::monde::Monde;

/// Pas d'immobilité qui valident un atterrissage stable.
const PAS_STABLE: u16 = 20;
/// Même chose dans le niveau de poursuite (carte 8.2) : le mur de pluie avance,
/// le robot ne peut pas s'arrêter 1/3 s à chaque atterrissage. Il repart dès qu'il
/// est posé (3 pas : ses pieds sont bien au sol).
const PAS_STABLE_POURSUITE: u16 = 3;
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
    /// Entré dans l'eau parapluie retourné (BAS, carte 9.2b) : BARQUE (carte
    /// 9.2), nouveau point de décision à bord (ramer, ou sauter hors de l'eau).
    Barque,
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

/// L'eau est-elle à moins de `marge` px sous les pieds de Pépin (carte 9.2b) ?
/// L'eau du moment : celle du niveau, plus les marées montées. Même règle que le
/// robot du cœur (`faisabilite::eau_sous_les_pieds`) ; toujours faux sans eau.
fn eau_proche(m: &Monde, marge: f32) -> bool {
    if !m.a_de_l_eau() {
        return false;
    }
    let eaux = eaux_des_entites(&m.niveau.eaux, &m.entites);
    eau_sous_les_pieds(&m.joueur.boite, &eaux, marge)
}

/// Les entrées d'un pas avec BAS tenu à l'approche de l'eau (carte 9.2b) : le
/// parapluie retourné, pour entrer dans l'eau en barque. Jamais au sol (carte
/// 9.2d : au sol, BAS assied Pépin et il ne court plus) ; sans eau, jamais tenu
/// (Monde 1 inchangé).
fn entrees_du_pas(m: &Monde) -> Entrees {
    Entrees {
        bas: !m.joueur.au_sol && eau_proche(m, MARGE_BAS),
        ..Entrees::default()
    }
}

/// Simule `plan` depuis `depart` (copie). Renvoie la fin, le monde final et les
/// entrées de chaque pas. Un chat écrasé de justesse compte comme une mort
/// (`chat_ecrase_de_justesse`).
pub(crate) fn simuler(depart: &Monde, plan: Plan) -> (Fin, Monde, Vec<Entrees>) {
    let mut m = depart.clone();
    let mut script = Vec::new();

    if plan.attente > 0 {
        for _ in 0..plan.attente {
            let e = entrees_du_pas(&m);
            m.avancer(&e);
            script.push(e);
            if est_mort(&m) || chat_ecrase_de_justesse(&m) {
                return (Fin::Mort, m, script);
            }
            if m.termine {
                return (Fin::Arrivee, m, script);
            }
        }
        let fin = if m.joueur.en_barque() {
            Fin::Barque
        } else if m.joueur.au_sol {
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
        let mut e = entrees_du_pas(&m);
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
                // Au ras de l'eau, il lâche le saut : le parapluie se retourne
                // (BAS est tenu) et Pépin entre en barque (carte 9.2b).
                e.saut = !eau_proche(&m, MARGE_PLANE);
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
        // Barque (carte 9.2) : le plan s'arrête quand Pépin entre dans l'eau
        // (parapluie retourné, carte 9.2b) : à bord est un point de décision, comme un atterrissage.
        if m.joueur.en_barque() {
            return (Fin::Barque, m, script);
        }
        if !m.joueur.au_sol {
            a_decolle = true;
            en_l_air += 1;
            stable = 0;
        } else if a_decolle || marche_finie {
            stable += 1;
            let pas_stable = if m.mur.is_some() {
                PAS_STABLE_POURSUITE
            } else {
                PAS_STABLE
            };
            if stable >= pas_stable {
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
/// plus tard. Et la marée (`Maree::decaler`, carte 9.3) : l'eau monte plus tôt ou
/// plus tard.
pub(crate) fn aiguilles_decalees(m: &Monde, pas: i32) -> Monde {
    let sous = aiguille_sous_pepin(m);
    let mut d = m.clone();
    let mut k = 0;
    for e in &mut d.entites {
        // Le boss (Grand Nuage, ou Pieuvre du port : carte 9.12a) a son horloge.
        if e.horloge_du_boss().is_some() {
            e.decaler_boss(pas);
            continue;
        }
        // La marée (carte 9.3) a son horloge, elle aussi.
        if let Entite::Maree(r) = e {
            r.decaler(pas);
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

/// Décalages essayés dans l'arène du Grand Nuage (carte 5.18) : une image de marge
/// de plus que pour les aiguilles (voir `tient_au_decalage`).
const DECALAGES_NUAGE: [i32; 6] = [-3, -2, -1, 1, 2, 3];

/// Écarts tolérés dans l'arène du Grand Nuage (carte 5.18) : rejouées avec le nuage
/// décalé, les mêmes touches doivent laisser Pépin presque au même endroit (1 px) et
/// presque à la même vitesse (5 px/s), pour que le plan suivant parte du même état.
const ECART_DECALAGE_NUAGE: f32 = 1.0;
const ECART_VITESSE_NUAGE: f32 = 5.0;

/// Marge (px) que Pépin garde de chaque côté de la plaque quand un plan le pose
/// sur une aiguille : jamais posé sur le bord, d'où un saut de travers le ferait
/// tomber au moindre écart.
const MARGE_PLAQUE: f32 = 3.0;

/// Le plan `plan`, joué depuis `depart` jusqu'à `fin` / `arrivee`, tient-il si les
/// aiguilles d'horloge (et le Grand Nuage) ont une ou deux images d'avance ou de
/// retard (`DECALAGES_AIGUILLE` ; trois dans l'arène du nuage, `DECALAGES_NUAGE`) ?
/// Il faut la même fin (jamais une mort), les mêmes
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
///
/// Dans l'arène du Grand Nuage (carte 5.18), on rejoue les MÊMES TOUCHES (`entrees`,
/// en « boucle ouverte », `rejouer`) et non le même plan : le plan s'adapte (il garde
/// la direction « en descendant », par exemple, et descend plus tard si le courant du
/// nuage souffle plus tard), alors que le navigateur et le test
/// `le_parcours_du_grand_nuage_tient_a_une_image_pres` rejouent les touches telles
/// qu'elles ont été enregistrées. Avec les motifs élargis de la carte 5.18, le
/// parcours trouvé avec le même plan tenait plan par plan, mais pas une fois rejoué
/// en entier avec deux images d'avance : Pépin, soulevé deux images plus tard,
/// frappait le nuage plus tard, rebondissait 2,4 px plus loin et mourait dans le
/// dernier motif. On y exige aussi une image de marge de plus (`DECALAGES_NUAGE`),
/// et la même vitesse à l'arrivée (`ECART_VITESSE_NUAGE`).
pub(crate) fn tient_au_decalage(
    depart: &Monde,
    plan: Plan,
    fin: Fin,
    arrivee: &Monde,
    entrees: &[Entrees],
) -> bool {
    // Le rectangle qui va du départ à l'arrivée du plan (pour la marée).
    let (de, vers) = (&depart.joueur.boite, &arrivee.joueur.boite);
    let (gauche, haut) = (de.x.min(vers.x), de.y.min(vers.y));
    let trajet = Aabb::new(
        gauche,
        haut,
        de.droite().max(vers.droite()) - gauche,
        de.bas().max(vers.bas()) - haut,
    );
    let a_des_aiguilles = depart.entites.iter().any(|e| match e {
        Entite::Mobile(a) => a.cercle().is_some(),
        Entite::Nuage(_) | Entite::Pieuvre(_) => true,
        // La marée (carte 9.3) : une barque qui monte à l'image près, un fond
        // de fosse touché juste quand l'eau s'en va ne laissent aucune marge.
        // Seulement si le plan passe PRÈS d'elle (carte 9.7, `pres_de_la_maree`).
        Entite::Maree(r) => pres_de_la_maree(r, &trajet),
        _ => false,
    });
    if !a_des_aiguilles {
        return true;
    }
    // Un boss (le Grand Nuage, ou la Pieuvre du port : carte 9.12a) : mêmes règles.
    let nuage = depart.pv_nuage().is_some();
    let (decalages, ecart): (&[i32], f32) = if nuage {
        (&DECALAGES_NUAGE, ECART_DECALAGE_NUAGE)
    } else {
        (&DECALAGES_AIGUILLE, ECART_DECALAGE)
    };
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
    decalages.iter().all(|&pas| {
        let decale = aiguilles_decalees(depart, pas);
        let (fin_d, m_d) = if nuage {
            rejouer(&decale, entrees)
        } else {
            let (f, m, _) = simuler(&decale, plan);
            (f, m)
        };
        let (a, b) = (&m_d.joueur.boite, &arrivee.joueur.boite);
        fin_d == fin
            && m_d.gouttes_niveau == arrivee.gouttes_niveau
            && m_d.lettre_prise == arrivee.lettre_prise
            && m_d.pv_nuage() == arrivee.pv_nuage()
            && (!coup || horloge_du_nuage(&m_d) == horloge_du_nuage(arrivee))
            && (fin == Fin::Arrivee || ((a.x - b.x).abs() <= ecart && (a.y - b.y).abs() <= ecart))
            && (!nuage
                || fin == Fin::Arrivee
                || ((m_d.joueur.vitesse.x - arrivee.joueur.vitesse.x).abs() <= ECART_VITESSE_NUAGE
                    && (m_d.joueur.vitesse.y - arrivee.joueur.vitesse.y).abs()
                        <= ECART_VITESSE_NUAGE))
    })
}

/// Rejoue les touches `entrees` depuis `depart` (copie), sans rien adapter (comme le
/// navigateur qui rejoue un script) ; s'arrête à une mort ou à l'arrivée. Renvoie la
/// fin, comme `simuler` : posé ou non à la fin des touches.
fn rejouer(depart: &Monde, entrees: &[Entrees]) -> (Fin, Monde) {
    let mut m = depart.clone();
    for e in entrees {
        m.avancer(e);
        if est_mort(&m) || chat_ecrase_de_justesse(&m) {
            return (Fin::Mort, m);
        }
        if m.termine {
            return (Fin::Arrivee, m);
        }
    }
    let fin = if m.joueur.en_barque() {
        Fin::Barque
    } else if m.joueur.au_sol {
        Fin::Pose
    } else {
        Fin::Abandon
    };
    (fin, m)
}

/// Une arrivée compte-t-elle ? Toujours, sauf dans l'arène du Grand Nuage : là, le
/// dernier coup finit le niveau, et le robot doit avoir pris toutes les gouttes
/// avant (comme un joueur qui veut les trois gouttes du niveau). La sortie secrète
/// (carte 8.5) ne compte jamais : le robot suit le chemin normal, jusqu'au drapeau.
pub(crate) fn gouttes_avant_la_victoire(m: &Monde) -> bool {
    !m.sortie_secrete && (m.pv_nuage().is_none() || m.gouttes_niveau.iter().all(|&g| g))
}

/// La phase du boss (Grand Nuage ou Pieuvre) et le temps passé dans cette phase
/// (`None` sans boss).
fn horloge_du_nuage(m: &Monde) -> Option<(u8, u16)> {
    m.entites.iter().find_map(Entite::horloge_du_boss)
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
/// `difficulte`), et `Ord` (lexicographique, dans l'ordre des champs, comme
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
    /// Barque (carte 9.2) : à bord de l'eau, le prochain choix se fait en
    /// ramant ou en sautant hors de l'eau : un état distinct de « posé » au
    /// même endroit (là où on peut marcher ou planer sans noyade).
    pub(crate) barque: bool,
    /// Escargot proche, résumé en entier (`escargot_proche`) ; 0 s'il n'y en a pas.
    pub(crate) escargot: i32,
    /// Chats du niveau, résumés en entier (`chats_de_la_case`) ; 0 sans chat.
    pub(crate) chats: i32,
    /// Heure de l'aiguille d'horloge proche (`aiguille_proche`) ; 0 s'il n'y en a pas.
    pub(crate) aiguille: i32,
    /// Le Grand Nuage, résumé en entier (`nuage_de_la_case`) ; 0 sans nuage.
    pub(crate) nuage: i32,
    /// L'heure de la marée (`maree_de_la_case`, carte 9.3) ; 0 sans marée.
    pub(crate) maree: i32,
    /// Ennemis d'eau du Monde 2 (carte 9.4), résumés en entier
    /// (`ennemis_d_eau_de_la_case`) ; 0 sans eux.
    pub(crate) eau: i64,
    /// Un secret déjà révélé (carte 8.5) : le même endroit atteint par un raccourci
    /// ne remplace pas celui du chemin normal (`trouver_parcours` refuse l'arrivée
    /// après un secret).
    pub(crate) secret: bool,
}

/// La case (`CaseParcours`) de Pépin dans `m`.
pub(crate) fn cle(m: &Monde) -> CaseParcours {
    CaseParcours {
        x: (m.joueur.boite.x / 8.0).floor() as i32,
        y: m.joueur.boite.y.round() as i32,
        lance: est_lance(m),
        barque: m.joueur.en_barque(),
        escargot: escargot_proche(m),
        chats: chats_de_la_case(m),
        aiguille: aiguille_proche(m),
        nuage: nuage_de_la_case(m),
        maree: maree_de_la_case(m),
        eau: ennemis_d_eau_de_la_case(m),
        secret: m.caches_reveles.iter().any(|&r| r),
    }
}

/// Tranche du temps de la marée (pas) retenue dans la case : 30 pas (1/2 s).
const TRANCHE_MAREE_PAS: u32 = 30;

/// Portée (px) de la marée pour le robot (carte 9.7) : 8 tuiles autour de sa zone.
const PORTEE_MAREE: f32 = 128.0;

/// `boite` est-elle à moins de `PORTEE_MAREE` px de la zone de la marée `r` (en
/// x comme en y) ?
///
/// Pourquoi (carte 9.7, « Les égouts ») : l'heure de la marée multiplie par 20
/// les cases du robot (`maree_de_la_case`) et `tient_au_decalage` rejoue chaque
/// plan quatre fois de plus. Dans TOUT le niveau (146 tuiles), la recherche
/// tournait encore après 75 minutes ; marée comptée seulement près de ses
/// bassins, elle trouve en 17 minutes (machine chargée). Loin de toute marée,
/// son heure ne change rien au plan ; arrivé près d'elle, le robot l'attend
/// (plans « attendre ») comme avant.
fn pres_de_la_maree(r: &Maree, boite: &Aabb) -> bool {
    let (z, p) = (r.zone, PORTEE_MAREE);
    Aabb::new(z.x - p, z.y - p, z.w + 2.0 * p, z.h + 2.0 * p).chevauche(boite)
}

/// L'heure de la marée (carte 9.3), résumée en un entier : `1 + tranche`, où
/// `tranche` est le temps écoulé dans son cycle par tranches de
/// `TRANCHE_MAREE_PAS` ; 0 sans marée, et 0 loin de toute marée
/// (`pres_de_la_maree`, carte 9.7). Comme le nuage, la marée s'ATTEND : sans
/// son heure dans la case, attendre au bord ramènerait à une case déjà connue.
pub(crate) fn maree_de_la_case(m: &Monde) -> i32 {
    m.entites
        .iter()
        .find_map(|e| match e {
            Entite::Maree(r) if pres_de_la_maree(r, &m.joueur.boite) => {
                Some(1 + (r.pas() / TRANCHE_MAREE_PAS) as i32)
            }
            _ => None,
        })
        .unwrap_or(0)
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
///
/// La Pieuvre du port (carte 9.12a) se résume de la même façon : sa phase (le rang
/// de `PhasePieuvre`, de 0 à 6), son temps, ses points de vie (`Entite::horloge_du_boss`).
pub(crate) fn nuage_de_la_case(m: &Monde) -> i32 {
    m.entites
        .iter()
        .find_map(|e| {
            let (phase, ecoule) = e.horloge_du_boss()?;
            let (pv, _) = e.points_du_boss()?;
            let tranche = i32::from(ecoule / TRANCHE_NUAGE_PAS).min(63);
            let gouttes = m
                .gouttes_niveau
                .iter()
                .enumerate()
                .fold(0, |b, (k, &g)| b | (i32::from(g) << k));
            Some(1 + tranche + 64 * (i32::from(phase) + 8 * i32::from(pv)) + 2048 * gouttes)
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
/// 8 tranches pour un cycle de 3,7 s (12 pour 6 s avant la carte 6.3).
const TRANCHE_CHAT_PAS: u16 = 30;

/// Fenêtre (px, en x entre les centres et en y entre les pieds) dans laquelle un
/// ennemi ÉCRASÉ (chat, ennemi d'eau) compte dans la case de Pépin : un écran de
/// chaque côté.
///
/// Pourquoi (allègement des outils) : avec un bit par ennemi écrasé de TOUT le
/// niveau, chaque ennemi écrasable doublait le nombre de cases de toute la suite
/// (6 rats au caniveau : 64 fois plus de cases, une recherche de plusieurs
/// heures), alors qu'un rat écrasé trois écrans plus tôt ne change rien à ce que
/// Pépin peut faire ici. Chaque noeud de la recherche garde son VRAI monde (tous
/// ses ennemis, écrasés ou non) : le parcours trouvé reste exactement jouable ;
/// seules deux cases qui ne diffèrent que par un ennemi lointain fusionnent (la
/// moins coûteuse reste, comme pour deux positions voisines).
const FENETRE_ECRASES: (f32, f32) = (320.0, 180.0);

/// `(x du centre, y des pieds)` est-il dans `FENETRE_ECRASES` autour de Pépin ?
fn dans_la_fenetre_des_ecrases(joueur: &Aabb, centre_x: f32, pieds: f32) -> bool {
    (centre_x - (joueur.x + joueur.w / 2.0)).abs() <= FENETRE_ECRASES.0
        && (pieds - joueur.bas()).abs() <= FENETRE_ECRASES.1
}

/// Les chats du niveau, résumés en un entier : `32 × écrasés + 2 × tranche + bond`.
/// - `écrasés` : un bit par chat du niveau (dans l'ordre des entités), à 1 s'il
///   est écrasé ET proche de Pépin (`FENETRE_ECRASES`) ;
/// - `tranche` : où en est le chat vivant le plus proche (dans `PORTEE_CHAT`,
///   mesurée depuis sa PLACE : un chat en plein bond reste le même chat) dans son
///   cycle, `1 + pas_du_cycle / TRANCHE_CHAT_PAS` (de 1 à 8) ; 0 si aucun chat
///   vivant n'est proche ;
/// - `bond` (carte 6.3) : 1 si ce chat est en train de bondir (son éveil avec
///   bond : il n'est plus à sa place, sa zone dangereuse s'étend à tout son bond),
///   0 sinon.
///
/// Pourquoi : le chat garde un passage, il faut ATTENDRE qu'il dorme. Sans le
/// moment de son cycle dans la case, attendre sur place ramène à une case déjà
/// connue et le plan « attendre » est jeté : le robot ne pourrait jamais laisser
/// le chat se rendormir. Et sans les chats écrasés, un état « chat écrasé »
/// fusionnerait avec le même endroit « chat vivant ». Le bond, lui, se joue
/// dans la VRAIE simulation (le chat voit Pépin dans `resoudre_contacts`) : un
/// plan qui finit sous le bond est une mort, et il est écarté ; le bit `bond`
/// garde seulement distincts « même moment, chat à sa place » et « même moment,
/// chat en l'air ». Toujours 0 dans un niveau sans chat : la case est alors
/// exactement celle d'avant la carte 5.9.
pub(crate) fn chats_de_la_case(m: &Monde) -> i32 {
    let j = &m.joueur.boite;
    let centre = j.x + j.w / 2.0;
    let mut ecrases = 0i32;
    let mut proche: Option<(f32, u16, bool)> = None;
    let chats = m.entites.iter().filter_map(|e| match e {
        Entite::Ennemi(c) if c.type_ennemi == TypeEnnemi::Chat => Some(c),
        _ => None,
    });
    for (k, c) in chats.enumerate() {
        assert!(k < 25, "trop de chats pour la case du robot");
        if !c.est_dangereux() {
            let (px, py) = c.place();
            if dans_la_fenetre_des_ecrases(j, px + c.boite.w / 2.0, py + c.boite.h) {
                ecrases |= 1 << k;
            }
            continue;
        }
        let (px, py) = c.place();
        let dx = (px + c.boite.w / 2.0 - centre).abs();
        let dy = (py + c.boite.h - j.bas()).abs();
        if dx <= PORTEE_CHAT.0 && dy <= PORTEE_CHAT.1 && proche.is_none_or(|(d, ..)| dx < d) {
            proche = Some((dx, pas_du_cycle(c.phase_chat()), c.bond().is_some()));
        }
    }
    let (tranche, bond) = proche.map_or((0, 0), |(_, p, b)| {
        (1 + i32::from(p / TRANCHE_CHAT_PAS), i32::from(b))
    });
    32 * ecrases + 2 * tranche + bond
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

/// Portée (px) à laquelle un ennemi d'eau compte dans la case de Pépin : 7 tuiles
/// en x (entre les centres), 5 tuiles en y (entre les pieds).
const PORTEE_EAU: (f32, f32) = (112.0, 80.0);
/// Tranche du temps d'une phase d'ennemi d'eau (pas) retenue dans la case : 1/4 s.
const TRANCHE_EAU_PAS: u16 = 15;

/// Les ennemis d'eau du Monde 2 (carte 9.4 : grenouille, poisson, crabe, rat,
/// mouette), résumés en un entier : un bit par ennemi déjà écrasé et proche de
/// Pépin (`FENETRE_ECRASES`), puis, pour le
/// plus proche dans `PORTEE_EAU` encore vivant, son espèce, sa phase, le temps
/// de sa phase (tranches de `TRANCHE_EAU_PAS`) et sa case de 16 px en x. 0 dans
/// un niveau sans eux : la case est alors exactement celle d'avant la carte 9.4.
///
/// Pourquoi : comme le chat, ces ennemis s'ATTENDENT (le poisson saute à heure
/// fixe, le rat fonce puis reste étourdi, la mouette pique puis remonte, le
/// crabe passe). Sans leur moment dans la case, attendre sur place ramènerait à
/// une case déjà connue et le plan « attendre » serait jeté.
pub(crate) fn ennemis_d_eau_de_la_case(m: &Monde) -> i64 {
    use pepin_core::entities::ennemi::{PhaseEau, PhasePigeon};
    let j = &m.joueur.boite;
    let centre = j.x + j.w / 2.0;
    let mut ecrases = 0i64;
    let mut proche: Option<(f32, i64)> = None;
    let ennemis = m.entites.iter().filter_map(|e| match e {
        Entite::Ennemi(c) if c.type_ennemi.est_d_eau() || c.type_ennemi == TypeEnnemi::Mouette => {
            Some(c)
        }
        _ => None,
    });
    for (k, c) in ennemis.enumerate() {
        assert!(k < 24, "trop d'ennemis d'eau pour la case du robot");
        if c.etat != EtatEnnemi::Actif {
            // Écrasé : il ne compte que proche de Pépin (`FENETRE_ECRASES`).
            if dans_la_fenetre_des_ecrases(j, c.boite.x + c.boite.w / 2.0, c.boite.bas()) {
                ecrases |= 1 << k;
            }
            continue;
        }
        let dx = (c.boite.x + c.boite.w / 2.0 - centre).abs();
        let dy = (c.boite.bas() - j.bas()).abs();
        if dx > PORTEE_EAU.0 || dy > PORTEE_EAU.1 || proche.is_some_and(|(d, _)| dx >= d) {
            continue;
        }
        let (code, n) = if c.type_ennemi == TypeEnnemi::Mouette {
            match c.phase() {
                PhasePigeon::Perche => (0, 0),
                PhasePigeon::Annonce(n) => (1, u16::from(n)),
                PhasePigeon::Plongee(n) => (2, n),
                PhasePigeon::Remontee(n) => (3, n),
                PhasePigeon::Repos(n) => (4, n),
            }
        } else {
            match c.phase_eau() {
                PhaseEau::Guet => (0, 0),
                PhaseEau::Annonce(n) => (1, n),
                PhaseEau::Attaque(n) => (2, n),
                PhaseEau::Repos(n) => (3, n),
            }
        };
        let espece = c.type_ennemi as i64;
        let tranche = i64::from((n / TRANCHE_EAU_PAS).min(31));
        let case_x = (c.boite.x / 16.0).floor() as i64;
        let resume = 1 + espece + 16 * (code + 8 * (tranche + 32 * (case_x + 1)));
        proche = Some((dx, resume));
    }
    let resume = proche.map_or(0, |(_, r)| r);
    resume + (ecrases << 40)
}

/// Un morceau de parcours : le plan joué, les entrées exactes qu'il a produites
/// et la case (`cle`) où Pépin s'est posé, a été relancé, ou a touché l'arrivée.
#[derive(Clone)]
pub(crate) struct Etape {
    // Lu par la mesure de difficulté, absente du build de QA (carte 9.1).
    #[cfg_attr(feature = "debug_niveaux", allow(dead_code))]
    pub(crate) plan: Plan,
    pub(crate) entrees: Vec<Entrees>,
    pub(crate) cle: CaseParcours,
}

/// Le monde du niveau `indice` après `PAS_DEPART` pas sans entrée (Pépin posé),
/// et ces entrées vides.
pub(crate) fn depart_pose(indice: usize) -> (Monde, Vec<Entrees>) {
    depart_pose_mode(indice, false)
}

/// Comme `depart_pose`, en nouvelle partie + si `ng_plus` (carte 8.8).
fn depart_pose_mode(indice: usize, ng_plus: bool) -> (Monde, Vec<Entrees>) {
    let mut depart = Monde::charger(indice, ng_plus);
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
        // Chaque plan est simulé à part, sur les cœurs libres (`parallele`), puis
        // dépouillé ci-dessous dans l'ordre des plans : le robot trouve exactement
        // le même parcours qu'en série (mêmes empreintes). `None` : rien à
        // dépouiller (mort, abandon, ou plan écarté).
        let depart_des_plans = &noeuds[i].monde;
        let essais = pepin_core::parallele::appliquer(a_essayer, |&plan| {
            let (fin, monde, entrees) = simuler(depart_des_plans, plan);
            if matches!(fin, Fin::Mort | Fin::Abandon) {
                return None;
            }
            // Dans un niveau à aiguilles d'horloge : un plan qui ne tient qu'à
            // l'image près est écarté, comme une mort (`tient_au_decalage`).
            if !tient_au_decalage(depart_des_plans, plan, fin, &monde, &entrees) {
                return None;
            }
            Some((plan, fin, monde, entrees))
        });
        for (plan, fin, monde, entrees) in essais.into_iter().flatten() {
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
                Fin::Pose | Fin::Lance | Fin::Barque if est_cible(&monde) => {
                    if arrivee.as_ref().is_none_or(|(c, _, _)| cout_plan < *c) {
                        arrivee = Some((cout_plan, i, etape));
                    }
                }
                Fin::Pose | Fin::Lance | Fin::Barque => {
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
/// l'arrivée sans jamais mourir. `None` si rien n'est trouvé. Le chemin NORMAL
/// (carte 8.5) : une arrivée après avoir révélé un secret (salle, raccourci) ne
/// compte pas ; seule la partie complète entre dans les salles secrètes.
pub fn trouver_parcours(indice: usize, precis: bool) -> Option<Vec<Entrees>> {
    trouver_parcours_mode(indice, precis, false)
}

/// Genre de parcours mis en mémoire (`en_memoire`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GenreParcours {
    Normal,
    NouvellePartiePlus,
    Complet,
}

/// Mémoire des parcours déjà cherchés dans CE processus de test (carte 9.23, tri
/// des tests). La recherche ne dépend que de ses arguments (niveaux embarqués,
/// simulation déterministe) : plusieurs tests qui demandent le même parcours (la
/// partie du Monde 1 et le clocher, le Grand Nuage et sa tenue à une image près...)
/// le cherchent UNE fois. Chaque clé a sa `OnceLock` : un deuxième fil qui la
/// demande pendant la recherche attend son résultat au lieu de la refaire.
/// Notions Rust : `OnceLock` (valeur calculée une seule fois, partagée entre fils),
/// `Mutex<HashMap<..>>` (table partagée), `Arc` (la cellule survit au verrou).
fn en_memoire(
    cle: (usize, bool, GenreParcours),
    chercher: impl FnOnce() -> Option<Vec<Entrees>>,
) -> Option<Vec<Entrees>> {
    use std::sync::{Arc, Mutex, OnceLock};
    type Cellule = Arc<OnceLock<Option<Vec<Entrees>>>>;
    type Table = HashMap<(usize, bool, GenreParcours), Cellule>;
    static MEMOIRE: OnceLock<Mutex<Table>> = OnceLock::new();
    let cellule: Cellule = {
        let mut table = MEMOIRE
            .get_or_init(Default::default)
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        Arc::clone(table.entry(cle).or_default())
    };
    cellule.get_or_init(|| sur_disque(cle, chercher)).clone()
}

/// Mémoire SUR DISQUE des parcours (carte 9.23), d'une commande de test à l'autre.
/// Seulement si la variable `PEPIN_MEMOIRE_PARCOURS` donne un dossier (les
/// commandes `./docker-cargo.sh carte <clé>` et `lot` la posent ; un simple
/// `cargo test` cherche toujours). Chaque fichier porte l'EMPREINTE de ce dont la
/// recherche dépend (`empreinte_parcours`) : le code du jeu et du robot, et les
/// données du niveau. Un niveau retouché, ou une ligne de code changée, et l'empreinte
/// change : on cherche de nouveau. Un parcours relu est en plus REJOUÉ dans la vraie
/// simulation (`parcours_encore_bon`) avant d'être rendu ; s'il ne tient plus, on
/// cherche. « Aucun parcours » est gardé aussi (le robot maladroit qui échoue fait
/// une recherche complète, la plus longue) : la recherche est déterministe, et
/// l'empreinte couvre tout ce qu'elle lit.
fn sur_disque(
    cle: (usize, bool, GenreParcours),
    chercher: impl FnOnce() -> Option<Vec<Entrees>>,
) -> Option<Vec<Entrees>> {
    let Some(dossier) = std::env::var_os("PEPIN_MEMOIRE_PARCOURS") else {
        return chercher();
    };
    let dossier = std::path::PathBuf::from(dossier);
    let chemin = dossier.join(format!("{:016x}.txt", empreinte_parcours(cle)));
    if let Ok(texte) = std::fs::read_to_string(&chemin) {
        match lire_parcours(&texte) {
            Some(None) => return None,
            Some(Some(p)) if parcours_encore_bon(cle, &p) => return Some(p),
            _ => {}
        }
    }
    let trouve = chercher();
    // Écrit à côté puis renommé : un autre processus ne lit jamais un fichier à moitié écrit.
    let provisoire = chemin.with_extension(format!("{}.provisoire", std::process::id()));
    let _ = std::fs::create_dir_all(&dossier);
    if std::fs::write(&provisoire, ecrire_parcours(trouve.as_deref())).is_ok() {
        let _ = std::fs::rename(&provisoire, &chemin);
    }
    trouve
}

/// Une entrée par caractère : direction (0 immobile, 1 gauche, 2 droite) + 3 si
/// SAUT tenu + 6 si BAS tenu, écrite en base 12 (`0`..`9`, `a`, `b`). « AUCUN » :
/// la recherche n'a rien trouvé.
fn ecrire_parcours(parcours: Option<&[Entrees]>) -> String {
    use pepin_core::player::Direction;
    let Some(p) = parcours else {
        return "AUCUN\n".to_string();
    };
    let mut texte: String = p
        .iter()
        .map(|e| {
            let d = match e.direction {
                Direction::Immobile => 0,
                Direction::Gauche => 1,
                Direction::Droite => 2,
            };
            let n = d + 3 * u32::from(e.saut) + 6 * u32::from(e.bas);
            char::from_digit(n, 12).expect("moins de 12")
        })
        .collect();
    texte.push('\n');
    texte
}

/// L'inverse de `ecrire_parcours` : `None` si le fichier est illisible.
fn lire_parcours(texte: &str) -> Option<Option<Vec<Entrees>>> {
    use pepin_core::player::Direction;
    let texte = texte.trim();
    if texte == "AUCUN" {
        return Some(None);
    }
    let mut p = Vec::with_capacity(texte.len());
    for c in texte.chars() {
        let n = c.to_digit(12)?;
        p.push(Entrees {
            direction: match n % 3 {
                0 => Direction::Immobile,
                1 => Direction::Gauche,
                _ => Direction::Droite,
            },
            saut: (n / 3) % 2 == 1,
            bas: n / 6 == 1,
        });
    }
    Some(Some(p))
}

/// Le parcours relu, rejoué depuis le début du niveau dans la vraie simulation :
/// jamais de mort, le niveau fini, et la même condition d'arrivée que la recherche
/// (chemin normal : `gouttes_avant_la_victoire`, aucun secret révélé ; partie
/// complète : tous les objets du niveau, pas la sortie secrète).
fn parcours_encore_bon(cle: (usize, bool, GenreParcours), parcours: &[Entrees]) -> bool {
    let (indice, _, genre) = cle;
    let mut m = Monde::charger(indice, genre == GenreParcours::NouvellePartiePlus);
    let total = objets_du_niveau(&m);
    for e in parcours {
        m.avancer(e);
        if m.derniere_mort.is_some() {
            return false;
        }
    }
    m.termine
        && match genre {
            GenreParcours::Complet => objets_pris(&m) == total && !m.sortie_secrete,
            _ => gouttes_avant_la_victoire(&m) && !m.caches_reveles.iter().any(|&r| r),
        }
}

/// Empreinte de ce dont dépend la recherche d'un parcours : le genre, le niveau
/// (indice, robot précis ou maladroit, et TOUTES ses données : `InfosNiveau` en
/// `Debug`, grille comprise), et le code (`empreinte_du_code`).
fn empreinte_parcours(cle: (usize, bool, GenreParcours)) -> u64 {
    use std::hash::{Hash, Hasher};
    let (indice, precis, genre) = cle;
    let infos = crate::monde::infos_niveau(indice).expect("indice de niveau valide");
    let mut h = std::collections::hash_map::DefaultHasher::new();
    empreinte_du_code().hash(&mut h);
    (indice, precis, genre).hash(&mut h);
    format!("{infos:?}").hash(&mut h);
    h.finish()
}

/// Empreinte du CODE que lit la recherche : `core/src` (la simulation), SAUF les
/// fichiers des niveaux (`core/src/levels/<clé>.rs` : leurs données sont dans
/// l'empreinte de chaque niveau ; `variantes.rs`, la nouvelle partie +, est du code
/// et compte), plus `game/src/monde.rs` (le monde simulé, qui ne dépend que de
/// `pepin_core`) et ce fichier (le robot). Une carte de rendu (décor, sprites,
/// écrans) ne la change donc pas. Calculée une fois par processus.
fn empreinte_du_code() -> u64 {
    use std::hash::{Hash, Hasher};
    use std::path::{Path, PathBuf};
    use std::sync::OnceLock;
    static EMPREINTE: OnceLock<u64> = OnceLock::new();
    fn fichiers(dossier: &Path, liste: &mut Vec<PathBuf>) {
        let Ok(entrees) = std::fs::read_dir(dossier) else {
            return;
        };
        for entree in entrees.flatten() {
            let chemin = entree.path();
            if chemin.is_dir() {
                fichiers(&chemin, liste);
            } else {
                liste.push(chemin);
            }
        }
    }
    *EMPREINTE.get_or_init(|| {
        let racine = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let niveaux = racine.join("core/src/levels");
        let mut liste = Vec::new();
        fichiers(&racine.join("core/src"), &mut liste);
        liste.retain(|f| !f.starts_with(&niveaux) || f.ends_with("variantes.rs"));
        liste.push(racine.join("game/src/monde.rs"));
        liste.push(racine.join("game/src/bot_qa/parcours.rs"));
        // Carte 10.7 : le pilote de la phase 2 de la Pieuvre fait partie du robot.
        liste.push(racine.join("game/src/bot_qa/pilote_pieuvre.rs"));
        liste.sort();
        let mut h = std::collections::hash_map::DefaultHasher::new();
        for f in &liste {
            let contenu = std::fs::read(f)
                .unwrap_or_else(|e| panic!("source illisible {} : {e}", f.display()));
            f.strip_prefix(&racine).unwrap_or(f).hash(&mut h);
            contenu.hash(&mut h);
        }
        h.finish()
    })
}

/// Comme `trouver_parcours`, dans la variante de la nouvelle partie + si `ng_plus`
/// (carte 8.8 : ennemis déplacés, éclairs plus fréquents, vent de face...).
pub fn trouver_parcours_mode(indice: usize, precis: bool, ng_plus: bool) -> Option<Vec<Entrees>> {
    let genre = if ng_plus {
        GenreParcours::NouvellePartiePlus
    } else {
        GenreParcours::Normal
    };
    en_memoire((indice, precis, genre), || {
        chercher_parcours_mode(indice, precis, ng_plus)
    })
}

/// La recherche de `trouver_parcours_mode`, sans la mémoire.
fn chercher_parcours_mode(indice: usize, precis: bool, ng_plus: bool) -> Option<Vec<Entrees>> {
    let (depart, mut entrees) = depart_pose_mode(indice, ng_plus);
    let sans_secret =
        |m: &Monde| gouttes_avant_la_victoire(m) && !m.caches_reveles.iter().any(|&r| r);
    // Carte 10.7 : la Pieuvre du port À TOUR (sa phase 2, « fuite et montée »).
    // La recherche d'un seul tenant n'y finit jamais : après le dernier coup de la
    // phase 1, la montée (plus de 60 s, la mer, l'humeur et le cerveau de la
    // géante hors de la case) multiplie les états sans fin (voir la tête de
    // `pilote_pieuvre`). On va donc tout de suite au point par point, qui passe la
    // main au pilote pour la phase 2.
    if super::pilote_pieuvre::a_une_phase_2(&depart) {
        return parcours_du_boss_par_points(depart, entrees, &sans_secret, precis);
    }
    let Some(etapes) = chercher_avec(
        &depart,
        &plans(precis),
        &|_| false,
        &sans_secret,
        usize::MAX,
    ) else {
        // Repli au DERNIER essai (`precis`), dans l'arène d'un boss seulement.
        return if precis && depart.pv_nuage().is_some() {
            parcours_du_boss_par_points(depart, entrees, &sans_secret, true)
        } else {
            None
        };
    };
    for e in etapes {
        entrees.extend(e.entrees);
    }
    Some(entrees)
}

/// Arène d'un boss (carte 9.12c, la Pieuvre du port sur son platelage) : le parcours
/// cherché POINT PAR POINT, quand la recherche d'un seul tenant échoue. Une case
/// (`cle`) résume le boss à 15 pas près : dans la recherche d'un seul tenant, un
/// monde arrivé le premier sur une case la garde, même si les salves serrées ne lui
/// laissent plus d'issue, alors qu'un autre, quelques pas plus tard, en avait une.
/// Par étapes, chaque recherche repart d'un monde exact, jusqu'au point suivant (un
/// atterrissage où le boss a un point de moins), toujours sans mourir (`chercher`
/// écarte les morts). Après le dernier coup, Pépin attend sans rien toucher que la
/// plongée finisse le niveau : rien n'y est dangereux, mais un plan « attendre » n'y
/// tiendrait pas au décalage du boss (`tient_au_decalage` rejoue les seules touches
/// du plan, qui s'arrêtent à la fin ; le coup, lui, remet l'horloge du boss à zéro).
/// Repli seulement : là où la recherche d'un seul tenant réussit (tout le Monde 1,
/// le Grand Nuage compris), rien ne change.
///
/// Carte 10.7 : la Pieuvre à tour y vient directement, robot maladroit compris
/// (`precis` choisit les plans). Le dernier coup de la phase 1 ne finit pas le
/// niveau : il lance la phase 2. Le PILOTE de la 9.13g (`pilote_pieuvre::jouer`, le
/// même, ni recopié ni modifié) la joue alors dans la vraie partie, de l'arène
/// jusqu'à la plongée ; chacun de ses pas est noté puis rejoué ici. Le parcours ne
/// garde la phase 2 que si le pilote l'a gagnée SANS MOURIR (le rejeu le vérifie
/// encore, pas à pas).
fn parcours_du_boss_par_points(
    mut monde: Monde,
    mut entrees: Vec<Entrees>,
    arrivee_admise: &dyn Fn(&Monde) -> bool,
    precis: bool,
) -> Option<Vec<Entrees>> {
    let tous = plans(precis);
    // Carte 10.7 : le dernier coup de la phase 1 d'une Pieuvre à tour ne fait PAS
    // baisser ses points (elle les retrouve aussitôt pour la montée) : il compte
    // donc comme un point quand la phase 2 commence, et la boucle s'arrête là.
    let en_phase_2 = super::pilote_pieuvre::en_phase_2;
    while let Some(pv) = monde.pv_nuage().filter(|&pv| pv > 0 && !en_phase_2(&monde)) {
        let point = |m: &Monde| m.pv_nuage().is_some_and(|p| p < pv) || en_phase_2(m);
        for e in chercher_avec(&monde, &tous, &point, arrivee_admise, usize::MAX)? {
            for x in &e.entrees {
                monde.avancer(x);
            }
            entrees.extend(e.entrees);
        }
    }
    if en_phase_2(&monde) {
        let mut pas_du_pilote: Vec<Entrees> = Vec::new();
        let bilan = super::pilote_pieuvre::jouer(monde.clone(), &mut |plan| plan, |_, e| {
            pas_du_pilote.push(*e);
            true
        });
        if !bilan.victoire || bilan.morts > 0 {
            return None;
        }
        for e in &pas_du_pilote {
            monde.avancer(e);
            if est_mort(&monde) {
                return None;
            }
        }
        entrees.extend(pas_du_pilote);
    }
    // La plongée (120 pas) : au plus 600 pas (10 s) d'attente.
    for _ in 0..600 {
        if monde.termine {
            break;
        }
        let e = entrees_du_pas(&monde);
        monde.avancer(&e);
        entrees.push(e);
        if est_mort(&monde) {
            return None;
        }
    }
    (monde.termine && arrivee_admise(&monde)).then_some(entrees)
}

/// Nombre de gouttes d'or du niveau déjà prises dans `m`.
fn gouttes_prises(m: &Monde) -> usize {
    m.gouttes_niveau.iter().filter(|&&g| g).count()
}

/// Objets de la partie complète pris dans `m` (gouttes d'or, et la lettre cachée du
/// niveau, carte 8.6), et combien le niveau en a en tout.
fn objets_pris(m: &Monde) -> usize {
    gouttes_prises(m) + usize::from(m.lettre_prise)
}

fn objets_du_niveau(m: &Monde) -> usize {
    let lettre = m.entites.iter().any(|e| matches!(e, Entite::Lettre(_)));
    m.gouttes_niveau.len() + usize::from(lettre)
}

/// Comme `trouver_parcours`, mais en ramassant les TROIS gouttes d'or du niveau
/// avant l'arrivée (carte 5.16 : la partie « complète », celle qui débloque le
/// bonus), et sa lettre cachée s'il en a une (carte 8.6 : la fin secrète). Recherche
/// par étapes, comme pour les auvents de l'essai : l'objet le plus proche (en temps),
/// puis le suivant, puis l'arrivée ; une arrivée avec un objet en moins est une
/// impasse (`chercher_avec`). Chaque étape est rejouée sur
/// le vrai monde (la simulation est déterministe : on retrouve exactement l'état
/// trouvé par la recherche). Le parcours n'est pas forcément le plus court qui
/// ramasse tout (on prend les gouttes dans l'ordre où elles sont les plus proches),
/// mais il est sûr par construction, comme celui du robot.
///
/// Arène d'un boss sans lettre (carte 9.12b, la Pieuvre du port) : si cette recherche
/// par étapes échoue (la goutte la plus proche, prise au mauvais moment, laisse Pépin
/// là où les salves serrées ne pardonnent plus), le parcours du robot convient : il
/// prend déjà les trois gouttes avant le dernier coup (`gouttes_avant_la_victoire`),
/// c'est donc un parcours complet. Repli seulement, et au DERNIER essai (`precis` :
/// les appelants essaient le maladroit, puis le précis) : là où la recherche par
/// étapes réussit (tout le Monde 1, le Grand Nuage compris), rien ne change.
pub fn trouver_parcours_complet(indice: usize, precis: bool) -> Option<Vec<Entrees>> {
    en_memoire((indice, precis, GenreParcours::Complet), || {
        chercher_parcours_complet(indice, precis)
    })
}

/// La recherche de `trouver_parcours_complet`, sans la mémoire.
fn chercher_parcours_complet(indice: usize, precis: bool) -> Option<Vec<Entrees>> {
    parcours_complet_par_etapes(indice, precis).or_else(|| {
        let (depart, _) = depart_pose(indice);
        let boss_sans_lettre =
            depart.pv_nuage().is_some() && objets_du_niveau(&depart) == depart.gouttes_niveau.len();
        if precis && boss_sans_lettre {
            trouver_parcours(indice, false).or_else(|| trouver_parcours(indice, true))
        } else {
            None
        }
    })
}

/// La recherche par étapes de `trouver_parcours_complet` (voir sa documentation).
fn parcours_complet_par_etapes(indice: usize, precis: bool) -> Option<Vec<Entrees>> {
    let (mut monde, mut entrees) = depart_pose(indice);
    let tous = plans(precis);
    let total = objets_du_niveau(&monde);
    let toutes = |m: &Monde| objets_pris(m) == total && !m.sortie_secrete;
    while !monde.termine {
        let prises = objets_pris(&monde);
        let etapes = if toutes(&monde) {
            chercher(&monde, &tous, &|_| false, usize::MAX)?
        } else {
            let une_de_plus = |m: &Monde| objets_pris(m) > prises;
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

/// Parcours qui prend le SECRET du niveau `indice` (carte 8.5) : une arrivée ne
/// compte qu'après avoir révélé une zone cachée (le raccourci), et, si le niveau a
/// une sortie secrète, que par elle. Sert aux captures de QA.md ; `None` si rien
/// n'est trouvé.
pub fn trouver_parcours_secret(indice: usize, precis: bool) -> Option<Vec<Entrees>> {
    let (depart, mut entrees) = depart_pose(indice);
    let par_le_secret = |m: &Monde| {
        m.caches_reveles.iter().any(|&r| r)
            && (m.niveau.sortie_secrete.is_none() || m.sortie_secrete)
    };
    let etapes = chercher_avec(
        &depart,
        &plans(precis),
        &|_| false,
        &par_le_secret,
        usize::MAX,
    )?;
    for e in etapes {
        entrees.extend(e.entrees);
    }
    Some(entrees)
}
