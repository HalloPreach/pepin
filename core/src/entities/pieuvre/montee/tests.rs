//! Tests de la carte 9.13d : le cerveau de la géante (graine, visée anticipée,
//! humeurs et combos), le coup final en deux jetés, les têtes et la reprise à
//! n'importe quelle lanterne. Sur le vrai niveau du boss (`levels::pieuvre`).
//!
//! Notion Rust : un module de tests rangé dans un sous-module voit les champs
//! privés de ses ancêtres (ici `Pieuvre::pv` et `Tour::cerveau`) : les tests
//! peuvent préparer un état précis sans ouvrir l'interface publique.

use super::*;
use crate::entities::{Actualisable, Entite};
use crate::level::Niveau;
use crate::levels::pieuvre::INFOS;
use crate::physics::Environnement;

fn pieuvre(n: &Niveau) -> Pieuvre {
    n.entites
        .iter()
        .find_map(|e| match e {
            Entite::Pieuvre(p) => Some((**p).clone()),
            _ => None,
        })
        .unwrap()
}

fn pas(p: &mut Pieuvre, n: &Niveau, k: u32) {
    let env = Environnement::simple(n);
    for _ in 0..k {
        p.actualiser(&env);
    }
}

/// Avance pas à pas (au plus `max`) jusqu'à ce que `fini` soit vrai.
#[track_caller]
fn jusqu_a(p: &mut Pieuvre, n: &Niveau, max: u32, fini: impl Fn(&Pieuvre) -> bool) {
    for _ in 0..max {
        if fini(p) {
            return;
        }
        pas(p, n, 1);
    }
    assert!(fini(p), "jamais en {max} pas");
}

/// La pieuvre de la tour au début de la montée (les trois coups de la phase 1,
/// puis la furie), graine `graine`.
fn en_montee(n: &Niveau, graine: u32) -> Pieuvre {
    let mut p = pieuvre(n);
    for _ in 0..3 {
        p.sortir_la_tete(0);
        p.toucher();
    }
    p.fixer_graine(graine);
    pas(&mut p, n, u32::from(super::super::FURIE_PAS));
    p
}

/// Pépin debout, le milieu à `x`, les pieds en `bas`.
fn debout(x: f32, bas: f32) -> Aabb {
    Aabb::new(x - JOUEUR_L / 2.0, bas - JOUEUR_H, JOUEUR_L, JOUEUR_H)
}

/// Un saut qui retombe sur `tete`.
fn saut_sur(tete: Aabb) -> crate::player::Joueur {
    let mut j = crate::player::Joueur::new(tete.x + 19.0, tete.y - 12.0);
    j.vitesse.y = 100.0;
    j
}

/// Les genres choisis par une géante de graine `graine`, Pépin immobile sur la
/// corniche balayée de la rangée 40 (frappe, balayage, secousse et jeté y sont
/// tous possibles), pendant 1500 pas.
fn suite(n: &Niveau, graine: u32) -> Vec<(u16, Genre)> {
    let mut p = en_montee(n, graine);
    p.voir_pepin(debout(9.0 * TAILLE_TUILE, 40.0 * TAILLE_TUILE));
    pas(&mut p, n, 1500);
    p.journal().to_vec()
}

/// Deux graines, deux montées ; la même graine, la même montée. Une reprise
/// change la graine (la tentative), donc la suite.
#[test]
fn deux_graines_donnent_deux_suites_d_attaques() {
    let n = INFOS.charger().unwrap();
    let (a, b) = (suite(&n, 1), suite(&n, 2));
    assert!(a.len() >= 6, "{a:?}");
    assert_ne!(a, b, "deux graines");
    assert_eq!(a, suite(&n, 1), "la même graine");
    // Plusieurs genres, pas toujours le même.
    let genres = |s: &[(u16, Genre)]| {
        let mut g: Vec<Genre> = Vec::new();
        for &(_, x) in s {
            if !g.contains(&x) {
                g.push(x);
            }
        }
        g.len()
    };
    assert!(genres(&a) >= 3, "{a:?}");
    // La tentative suivante (une reprise depuis la lanterne du bas).
    let mut p = en_montee(&n, 1);
    let bache = p.bache.unwrap();
    p.reprendre_a(&debout(40.0, bache.y - 2.0 * TAILLE_TUILE));
    assert_eq!(p.tentative(), 1);
    p.voir_pepin(debout(9.0 * TAILLE_TUILE, 40.0 * TAILLE_TUILE));
    pas(&mut p, &n, 1500 + u32::from(super::super::FURIE_PAS));
    assert_ne!(p.journal(), &a[..], "une autre tentative, une autre montée");
}

/// La frappe visée vise l'appui où Pépin VA ATTERRIR : en plein saut sous la
/// corniche 40, le repère se pose sur la corniche 40 (pas sur lui) ; s'il
/// s'arrête, sur lui. Et la place visée laisse une issue sur l'appui.
#[test]
fn la_frappe_vise_l_appui_d_arrivee() {
    let n = INFOS.charger().unwrap();
    let t = TAILLE_TUILE;
    let mut p = en_montee(&n, 7);
    // Après la crue, sans Pépin : aucune attaque.
    pas(&mut p, &n, 280);
    assert!(p.attaque().is_none());
    // En l'air, entre la caisse 43 (cases 3 à 5) et la corniche 40 (7 à 13),
    // qui monte vers la droite.
    let en_l_air = |k: f32| Aabb::new(6.0 * t + k, 41.0 * t - JOUEUR_H - k, JOUEUR_L, JOUEUR_H);
    p.tour.cerveau.recharges = [u16::MAX; 5];
    p.voir_pepin(en_l_air(0.0));
    pas(&mut p, &n, 1);
    // Seule la frappe est prête : elle part au pas suivant, sur son arrivée.
    let mut q = p.clone();
    p.voir_pepin(en_l_air(1.5));
    pas(&mut p, &n, 1);
    let arrivee = p.arrivee_prevue(&n, &en_l_air(1.5));
    assert!((arrivee.bas() - 40.0 * t).abs() < 0.5, "{arrivee:?}");
    assert!(
        arrivee.x >= 7.0 * t && arrivee.droite() <= 14.0 * t,
        "{arrivee:?}"
    );
    q.tour.cerveau.recharges = [0, u16::MAX, u16::MAX, u16::MAX, u16::MAX];
    q.voir_pepin(en_l_air(1.5));
    pas(&mut q, &n, 1);
    let Some((cible, EtatFrappe::Annonce(_))) = q.etat_frappe() else {
        panic!("une frappe annoncée : {:?}", q.attaque());
    };
    assert!(
        (cible.bas() - 40.0 * t).abs() < 0.5,
        "sur la corniche : {cible:?}"
    );
    assert!(!cible.chevauche(&en_l_air(1.5)), "pas sur sa place passée");
    // L'issue : une place de la corniche 40 que la frappe ne touche pas.
    let issue = (7..13).any(|c| !cible.chevauche(&debout(c as f32 * t + 8.0, 40.0 * t)));
    assert!(issue, "une autre place sur la corniche");
    // Arrêté : visé sur place.
    let arret = debout(10.0 * t, 40.0 * t);
    p.voir_pepin(arret);
    pas(&mut p, &n, 2);
    assert_eq!(p.arrivee_prevue(&n, &arret), arret);
}

/// L'humeur : calme au départ, énervée après le 1er coup reçu (ou passé 50 %
/// de la tour), furieuse après le 2e (ou passé 80 %) ; les recharges
/// raccourcissent à chaque palier.
#[test]
fn l_humeur_monte_apres_un_coup() {
    let n = INFOS.charger().unwrap();
    let mut p = en_montee(&n, 3);
    assert_eq!(p.humeur(), Humeur::Calme);
    let calme = p.recharge(Genre::Frappe);
    // Un jeté sur la corniche 52, frappé.
    let place = p.tour.jetes[0];
    p.voir_pepin(debout(place.x - 2.0 * TAILLE_TUILE, place.bas()));
    jusqu_a(&mut p, &n, 1500, |p| {
        matches!(p.etat_jete(), Some((_, EtatJete::Sonnee(_))))
    });
    assert!(p.recoit(&saut_sur(place)));
    p.toucher();
    assert_eq!(p.humeur(), Humeur::Enervee, "après le 1er coup");
    assert!(p.recharge(Genre::Frappe) < calme);
    p.pv -= 1;
    assert_eq!(p.humeur(), Humeur::Furieuse, "après le 2e coup");
    // Par la hauteur seule.
    let mut q = en_montee(&n, 3);
    let vigie = q.tete_finale.unwrap().bas();
    let bache = q.bache.unwrap().y;
    for (part, humeur) in [
        (0.3, Humeur::Calme),
        (0.55, Humeur::Enervee),
        (0.85, Humeur::Furieuse),
    ] {
        q.voir_pepin(debout(40.0, bache - part * (bache - vigie)));
        pas(&mut q, &n, 1);
        assert_eq!(q.humeur(), humeur, "à {part}");
    }
}

/// Un combo (énervée ou furieuse) : un balayage ou une secousse, puis une
/// frappe visée enchaînée. Jamais deux parties mortelles en même temps, et à
/// chaque pas il reste une place sûre à portée de Pépin (sur sa corniche, ou
/// au-dessus du bout du balayage en sautant).
#[test]
fn un_combo_laisse_une_issue() {
    let n = INFOS.charger().unwrap();
    let t = TAILLE_TUILE;
    let mut vus = (0, 0);
    for graine in 1..40 {
        let mut p = en_montee(&n, graine);
        p.pv = 1;
        assert_eq!(p.humeur(), Humeur::Furieuse);
        // Sur la corniche 22, balayée, sous la tour : tout est possible.
        let (sol, x) = (22.0 * t, 8.0 * t);
        let pepin = debout(x, sol);
        p.voir_pepin(pepin);
        let mut combo = false;
        for _ in 0..1200 {
            pas(&mut p, &n, 1);
            let frappe = matches!(p.etat_frappe(), Some((_, EtatFrappe::Frappe)));
            let balai = (0..p.tour.balayeurs.len())
                .any(|k| matches!(p.etat_balayeur(k), Some(EtatBalai::Balaie(_))));
            let caisse = match p.etat_secousse() {
                Some((caisses, EtatSecousse::Chute(s))) => {
                    caisses.iter().any(|c| c.boite(s).is_some())
                }
                _ => false,
            };
            assert!(
                !(frappe && (balai || caisse)),
                "graine {graine} : deux à la fois au pas {} : {:?} {:?} {:?}",
                p.ecoule(),
                p.tour.attaque,
                p.tour.cerveau.combo,
                p.journal()
            );
            if p.tour.cerveau.combo.is_some() {
                combo = true;
            }
            if combo && p.tour.cerveau.combo.is_some() {
                // Une place sûre sur la corniche (5 à 13), à pied ou en l'air
                // au-dessus du bout (12 px).
                let sure = (5..13).any(|c| {
                    let cx = c as f32 * t + 8.0;
                    !p.blesse(&debout(cx, sol)) || !p.blesse(&debout(cx, sol - 16.0))
                });
                assert!(sure, "graine {graine} : aucune issue au pas {}", p.ecoule());
            }
            if combo && p.tour.cerveau.combo.is_none() && p.tour.attaque.is_none() {
                break;
            }
        }
        if combo {
            let avant = p.journal().iter().rev().nth(1).map(|j| j.1);
            match avant {
                Some(Genre::Balayage) => vus.0 += 1,
                Some(Genre::Secousse) => vus.1 += 1,
                _ => {}
            }
        }
        if vus.0 > 0 && vus.1 > 0 {
            break;
        }
    }
    assert!(vus.0 > 0 && vus.1 > 0, "combos vus : {vus:?}");
}

/// Sur la vigie, elle se jette DEUX fois : la 1re (cabre de 1,2 s) ne la
/// laisse pas sonnée, la 2e enchaîne, plus vive (0,8 s), et seulement là elle
/// reste sonnée. À la dernière tête, le coup donne la plongée.
#[test]
fn la_vigie_demande_deux_jetes_avant_sonnee() {
    let n = INFOS.charger().unwrap();
    let mut p = en_montee(&n, 5);
    p.pv = 1;
    let tete = p.tete_finale.unwrap();
    p.voir_pepin(debout(tete.x - 3.0 * TAILLE_TUILE, tete.bas()));
    jusqu_a(&mut p, &n, 600, |p| p.etat_jete().is_some());
    let mut cabre = 0;
    while matches!(p.etat_jete(), Some((_, EtatJete::Cabre(_)))) {
        cabre += 1;
        pas(&mut p, &n, 1);
    }
    assert_eq!(cabre, CABRE_PAS, "1er jeté : 1,2 s");
    let mut sonnee = false;
    while !matches!(p.etat_jete(), Some((_, EtatJete::Cabre(_)))) {
        sonnee |= matches!(p.etat_jete(), Some((_, EtatJete::Sonnee(_))));
        assert!(p.etat_jete().is_some(), "elle enchaîne");
        pas(&mut p, &n, 1);
    }
    assert!(!sonnee, "pas sonnée après le 1er");
    let mut vif = 0;
    while matches!(p.etat_jete(), Some((_, EtatJete::Cabre(_)))) {
        vif += 1;
        pas(&mut p, &n, 1);
    }
    assert_eq!(vif, CABRE_VIF_PAS, "2e jeté : 0,8 s");
    jusqu_a(&mut p, &n, 30, |p| {
        matches!(p.etat_jete(), Some((_, EtatJete::Sonnee(_))))
    });
    assert!(p.recoit(&saut_sur(tete)));
    p.toucher();
    assert_eq!(p.phase(), PhasePieuvre::Plongee);
}

/// Les têtes : 3 au début de la phase 2 ; à la vigie, il faut prendre TOUTES
/// celles qui restent, chacune après deux jetés. Un seul coup n'en vide jamais
/// trois ; un coup pendant la montée en retire une, et la vigie en demande
/// une de moins.
#[test]
fn les_tetes_restantes_sont_exigees_a_la_vigie() {
    let n = INFOS.charger().unwrap();
    let vigie = |p: &mut Pieuvre| {
        let tete = p.tete_finale.unwrap();
        p.voir_pepin(debout(tete.x - 3.0 * TAILLE_TUILE, tete.bas()));
        let mut jetes = 0;
        let mut coups = 0;
        while p.phase() == PhasePieuvre::Montee {
            jusqu_a(p, &n, 900, |p| {
                matches!(p.etat_jete(), Some((_, EtatJete::Sonnee(_))))
            });
            jetes = p.jetes_faits();
            assert!(p.recoit(&saut_sur(tete)));
            p.toucher();
            coups += 1;
            assert!(coups <= 3, "trop de coups");
        }
        (coups, jetes)
    };
    let mut p = en_montee(&n, 11);
    assert_eq!(p.pv(), TETES_FUITE);
    let (coups, jetes) = vigie(&mut p);
    assert_eq!(coups, 3, "les 3 têtes à la vigie");
    assert_eq!(jetes, 6, "deux jetés par tête");
    // Une tête prise pendant la montée : deux à la vigie.
    let mut q = en_montee(&n, 11);
    let place = q.tour.jetes[0];
    q.voir_pepin(debout(place.x - 2.0 * TAILLE_TUILE, place.bas()));
    jusqu_a(&mut q, &n, 1500, |q| {
        matches!(q.etat_jete(), Some((_, EtatJete::Sonnee(_))))
    });
    assert!(q.recoit(&saut_sur(place)));
    q.toucher();
    assert_eq!((q.pv(), q.phase()), (2, PhasePieuvre::Montee));
    pas(&mut q, &n, u32::from(REPIT_PAS));
    assert_eq!(vigie(&mut q).0, 2);
}

/// La reprise à une lanterne QUELCONQUE de la tour (la carte 9.13e en ajoute
/// une) : la mer revient sous la lanterne, les têtes et l'humeur reviennent à
/// leur état au passage de la lanterne, la graine change. Essayée à deux
/// hauteurs : avant et après un coup.
#[test]
fn reprise_a_une_lanterne_quelconque() {
    let n = INFOS.charger().unwrap();
    let t = TAILLE_TUILE;
    let mut p = en_montee(&n, 9);
    let bache = p.bache.unwrap().y;
    let vigie = p.tete_finale.unwrap().bas();
    // Pépin monte : rangée 49 (une lanterne), puis le jeté de la rangée 52
    // n'est pas sur son chemin ; on le fait passer par 40 (frappé), puis 25.
    for rangee in [60.0, 49.0, 43.0] {
        p.voir_pepin(debout(3.0 * t, rangee * t));
        pas(&mut p, &n, 1);
    }
    let place = p.tour.jetes[1];
    p.voir_pepin(debout(place.x - 2.0 * t, place.bas()));
    jusqu_a(&mut p, &n, 2000, |p| {
        matches!(p.etat_jete(), Some((_, EtatJete::Sonnee(_))))
    });
    assert!(p.recoit(&saut_sur(place)));
    p.toucher();
    for rangee in [34.0, 25.0] {
        p.voir_pepin(debout(3.0 * t, rangee * t));
        pas(&mut p, &n, 1);
    }
    assert_eq!(p.pv(), 2);
    for (rangee, pv) in [(49.0, 3), (34.0, 2), (25.0, 2)] {
        let mut q = p.clone();
        let tentative = q.tentative();
        let lanterne = debout(3.0 * t, rangee * t);
        q.reprendre_a(&lanterne);
        assert_eq!(q.tentative(), tentative + 1, "la graine change");
        assert_eq!(q.pv(), pv, "rangée {rangee}");
        let attendue = if pv == 3 && rangee * t > bache - 0.5 * (bache - vigie) {
            Humeur::Calme
        } else {
            Humeur::Enervee
        };
        assert_eq!(q.humeur(), attendue, "rangée {rangee}");
        let mer = q.surface_mer().unwrap();
        assert!(
            mer > lanterne.bas() && mer <= lanterne.bas() + 3.0 * t + 1.0,
            "{mer}"
        );
        assert!(q.journal().is_empty() && q.attaque().is_none());
    }
    // Depuis la lanterne du bas : tout repart à neuf.
    let mut q = p.clone();
    q.reprendre_a(&debout(40.0, bache - 2.0 * t));
    assert_eq!((q.pv(), q.horloge_mer(), q.humeur()), (3, 0, Humeur::Calme));
}
