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
//! - les plateformes friables restent solides (on peut toujours s'en
//!   détacher avant 0,5 s) ;
//! - une plateforme mobile est remplacée par DEUX plateformes immobiles à ses
//!   extrémités, et un joueur posé sur un bout est aussi considéré posé sur
//!   l'autre (il suffit d'attendre d'être emporté).
//!
//! Notions Rust illustrées ici (à reprendre dans le README) :
//! - `struct` d'options + `Default` ; `VecDeque` et `HashSet` (collections) ;
//! - boucles imbriquées sur des tableaux constants de paramètres ;
//! - emprunts : la grille et les courants sont PRÊTÉS à `Environnement`.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use crate::aabb::Aabb;
use crate::entities::contacts::{resoudre_contacts, verifier_vide, Contact};
use crate::entities::{solides_des_entites, Entite};
use crate::level::{Niveau, VITESSE_PLATEFORMES};
use crate::physics::{
    CourantAscendant, Environnement, HZ, JOUEUR_H, JOUEUR_L, VITESSE_PLANER_CHUTE,
};
use crate::player::{Direction, Entrees, Joueur};

/// Ce que le robot a le droit d'utiliser.
#[derive(Debug, Clone, Copy)]
pub struct Options {
    /// Peut-il ouvrir le parapluie (maintenir SAUT en tombant) ?
    pub planer: bool,
    /// Les courants ascendants soufflent-ils ?
    pub courants: bool,
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
    /// Temps écoulé ou état instable : on abandonne ce plan.
    Abandon,
}

/// Trajet d'une plateforme mobile : (extrémité a, extrémité b, largeur).
type Trajet = ((f32, f32), (f32, f32), f32);

/// Explore le niveau depuis `depart` (coin haut-gauche du joueur).
pub fn analyser(niveau: &Niveau, depart: (f32, f32), options: Options) -> Rapport {
    // Solides immobiles : friables + extrémités des plateformes mobiles.
    let mut solides: Vec<Aabb> = solides_des_entites(&niveau.entites);
    let mut entites: Vec<Entite> = Vec::new();
    let mut lanternes: Vec<(f32, f32)> = Vec::new();
    // (extrémité a, extrémité b, largeur) de chaque plateforme mobile.
    let mut trajets: Vec<Trajet> = Vec::new();
    for e in &niveau.entites {
        match e {
            Entite::Mobile(m) => {
                let (a, b) = m.extremites();
                solides.push(Aabb::new(a.0, a.1, m.boite.w, m.boite.h));
                solides.push(Aabb::new(b.0, b.1, m.boite.w, m.boite.h));
                trajets.push((a, b, m.boite.w));
            }
            Entite::Ennemi(_) | Entite::Eclair(_) => {}
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

    let courants: Vec<CourantAscendant> = if options.courants {
        niveau.courants.clone()
    } else {
        Vec::new()
    };
    let env = Environnement {
        grille: niveau,
        solides: &solides,
        courants: &courants,
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
    // connu pour chaque case de 8 px ; `tas` sert les états du moins coûteux
    // au plus coûteux (`Reverse` transforme le tas max en tas min).
    let mut meilleur: HashMap<(i32, i32), u32> = HashMap::new();
    let mut tas: BinaryHeap<Reverse<(u32, u32, u32)>> = BinaryHeap::new();
    meilleur.insert(cle(depart), 0);
    tas.push(Reverse((0, depart.0.to_bits(), depart.1.to_bits())));
    let tous_les_plans: Vec<Plan> = plans(options).collect();

    while let Some(Reverse((cout, bx, by))) = tas.pop() {
        let etat = (f32::from_bits(bx), f32::from_bits(by));
        if meilleur.get(&cle(etat)).is_some_and(|&c| c < cout) {
            continue; // version périmée : un chemin plus court existe
        }
        rapport.etats += 1;
        if etat.0 > rapport.x_max {
            rapport.x_max = etat.0;
            rapport.dernier_etat = etat;
        }
        for plan in &tous_les_plans {
            let mut ents = entites.clone();
            let mut touches: Vec<Contact> = Vec::new();
            let (fin, duree) = simuler(niveau, &env, etat, *plan, &mut ents, &mut touches);
            if matches!(fin, Fin::Mort | Fin::Abandon) {
                continue;
            }
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
            let mut ajouter = |arrive: (f32, f32), cout: u32| {
                let k = cle(arrive);
                if meilleur.get(&k).is_none_or(|&c| cout < c) {
                    meilleur.insert(k, cout);
                    tas.push(Reverse((cout, arrive.0.to_bits(), arrive.1.to_bits())));
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
                Fin::Pose(x, y) => {
                    ajouter((x, y), cout_plan);
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
                                ajouter((x + dx, y + dy), cout_plan + attente);
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

/// Simule un plan complet depuis un état stable.
fn simuler(
    niveau: &Niveau,
    env: &Environnement<'_>,
    depart: (f32, f32),
    plan: Plan,
    ents: &mut [Entite],
    touches: &mut Vec<Contact>,
) -> (Fin, u16) {
    let mut j = Joueur::new(depart.0, depart.1);
    // Un pas à vide pour que `au_sol` soit vrai dès le début.
    j.actualiser(&Entrees::default(), env);
    let bas_monde = niveau.hauteur_px();
    let pas_max = pas_max_plan(bas_monde);
    let mut en_l_air: u16 = 0;
    let mut a_decolle = false;
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
        for c in contacts {
            match c {
                Contact::Mort(_) => return (Fin::Mort, pas),
                autre => touches.push(autre),
            }
        }
        if verifier_vide(&j, bas_monde).is_some() {
            return (Fin::Mort, pas);
        }
        if j.boite.chevauche(&niveau.arrivee) {
            return (Fin::Arrivee, pas);
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
                precis: true
            }
        ));
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
}
