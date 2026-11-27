//! Tests de la nouvelle montée de la Pieuvre (carte 9.36a) : la mer qui suit
//! Pépin, les objets qui tombent (annoncés, toujours une issue), la Pieuvre au
//! loin qui ne blesse jamais, la fuite au sommet, la reprise à une lanterne, le
//! mode facile. Sur le vrai niveau du boss (`monde_2/pieuvre.ron`).
//!
//! Décision d'AGA (27/11) : « je pense supprimer la p2 telle qu'elle est
//! actuellement, et juste faire une montée contre l'eau qui avance plus vite
//! avec des objets qui tombent du ciel envoyés par la pieuvre. on ne la tue pas
//! à la fin, comme le nuage juste elle finit par s'enfuir ».

use crate::commun::*;
use pepin_core::aabb::{Aabb, Grille, TAILLE_TUILE};
use pepin_core::entities::pieuvre::*;
use pepin_core::entities::{Actualisable, Entite};
use pepin_core::physics::{Environnement, JOUEUR_H, JOUEUR_L};
use pepin_core::player::Joueur;

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

/// La pieuvre de la tour au début de la montée (les coups de la phase 1, puis
/// la furie), graine `graine`, en mode facile ou non.
fn en_montee_mode(n: &Niveau, graine: u32, facile: bool) -> Pieuvre {
    let mut p = pieuvre(n);
    if facile {
        p.passer_en_facile();
    }
    for _ in 0..PV_PIEUVRE {
        if !p.en_fuite() {
            p.sortir_la_tete(0);
            p.toucher();
        }
    }
    assert!(p.en_fuite());
    p.fixer_graine(graine);
    pas(&mut p, n, u32::from(FURIE_PAS));
    assert_eq!(p.phase(), PhasePieuvre::Montee);
    p
}

fn en_montee(n: &Niveau, graine: u32) -> Pieuvre {
    en_montee_mode(n, graine, false)
}

/// Pépin debout, le milieu à `x`, les pieds en `bas`.
fn debout(x: f32, bas: f32) -> Aabb {
    Aabb::new(x - JOUEUR_L / 2.0, bas - JOUEUR_H, JOUEUR_L, JOUEUR_H)
}

/// Les appuis de la tour, du bas vers le haut : (dessus, gauche, droite) en px.
fn appuis_de_la_tour(p: &Pieuvre, n: &Niveau) -> Vec<(f32, f32, f32)> {
    let mut a = p.appuis(n);
    a.sort_by(|x, y| y.0.total_cmp(&x.0));
    a
}

/// Carte 9.13f, resserrée par la 9.36a : LA MER SUIT PÉPIN. Il attend en bas de
/// la tour que la mer arrive à N tuiles de lui (8, 10 en facile), puis grimpe
/// d'un trait jusqu'au sommet, sans pause : vite (juste moins vite que la mer
/// qui rattrape, 3 fois sa vitesse) ou tranquillement (moitié moins). Elle n'est
/// jamais plus de N tuiles sous le plus haut qu'il a atteint, ne descend
/// jamais, ne monte jamais de plus de 3 fois sa vitesse en un pas (plus au plus
/// un pas de crue : pas de saut brusque).
#[test]
fn la_mer_suit_pepin_sans_jamais_descendre() {
    let n = par_cle("pieuvre").charger().unwrap();
    let t = TAILLE_TUILE;
    const { assert!(ECART_MER_MAX == 8.0 * TAILLE_TUILE) };
    const { assert!(ECART_MER_MAX_FACILE == 10.0 * TAILLE_TUILE) };
    for (facile, ecart) in [(false, ECART_MER_MAX), (true, ECART_MER_MAX_FACILE)] {
        for allure in [0.95, 0.5] {
            let mut p = en_montee_mode(&n, 1, facile);
            let v = VITESSE_MER * if facile { FACTEUR_FACILE } else { 1.0 };
            let grimpe = allure * RATTRAPAGE_MER * v;
            let sommet = p.sommet.unwrap().bas() + 4.0 * t;
            let mut pieds = p.bache.unwrap().y - 2.0 * t;
            // En l'air à gauche de la tour.
            p.voir_pepin(debout(2.5 * t, pieds));
            jusqu_a(&mut p, &n, 3000, |p| {
                p.plus_haut_atteint()
                    .is_some_and(|h| p.surface_mer().unwrap() - h <= ecart)
            });
            let mut avant = p.surface_mer().unwrap();
            let mut rattrape = 0;
            while pieds > sommet {
                pieds = (pieds - grimpe).max(sommet);
                p.voir_pepin(debout(2.5 * t, pieds));
                pas(&mut p, &n, 1);
                let mer = p.surface_mer().unwrap();
                let quoi = format!("facile {facile}, allure {allure}");
                assert!(
                    mer <= avant + 0.01,
                    "{quoi} : elle descend ({avant} -> {mer})"
                );
                assert!(
                    avant - mer <= RATTRAPAGE_MER * v + VITESSE_CRUE + 0.01,
                    "{quoi} : un saut de {} px",
                    avant - mer
                );
                avant = mer;
                let haut = p.plus_haut_atteint().unwrap();
                assert!(
                    mer - haut <= ecart + 0.01,
                    "{quoi} : {} tuiles sous le plus haut",
                    (mer - haut) / t
                );
                if p.mer_rattrape() {
                    rattrape += 1;
                    assert!(mer - pieds > RATTRAPAGE_PAS_SOUS, "{quoi} : il ne tue pas");
                }
            }
            assert!(
                rattrape > 100,
                "facile {facile} : rattrapage {rattrape} pas"
            );
        }
    }
}

/// Carte 9.13f : le rattrapage ne tue pas. Pépin saute très haut, puis
/// redescend se poser 6 tuiles au-dessus de la mer : elle rattrape à vitesse
/// triple, mais jamais à moins de 4 tuiles de ses pieds ; là, elle reprend sa
/// vitesse normale.
#[test]
fn le_rattrapage_ne_tue_pas() {
    let n = par_cle("pieuvre").charger().unwrap();
    let t = TAILLE_TUILE;
    let mut p = en_montee(&n, 2);
    let crue = u32::from(p.fin_de_crue());
    pas(&mut p, &n, crue);
    let bas = p.surface_mer().unwrap();
    // Son plus haut : 20 tuiles au-dessus de la mer (à gauche, en l'air)…
    p.voir_pepin(debout(2.5 * t, bas - 20.0 * t));
    pas(&mut p, &n, 1);
    // … puis il se pose 6 tuiles au-dessus d'elle.
    let pieds = bas - 6.0 * t;
    p.voir_pepin(debout(2.5 * t, pieds));
    let mut avant = p.surface_mer().unwrap();
    let (mut vite, mut normal) = (0, 0);
    for _ in 0..600 {
        pas(&mut p, &n, 1);
        let mer = p.surface_mer().unwrap();
        assert!(
            mer <= avant + 0.01
                && avant - mer <= RATTRAPAGE_MER * VITESSE_MER + VITESSE_CRUE + 0.01
        );
        if p.mer_rattrape() {
            vite += 1;
            assert!(mer - pieds > RATTRAPAGE_PAS_SOUS, "jamais sur lui");
        } else if vite > 0 {
            normal += 1;
        }
        avant = mer;
        if mer < pieds - 0.01 {
            break;
        }
    }
    assert!(vite > 20 && normal > 30, "vite {vite}, normal {normal}");
}

/// Les colonnes de la corniche (sa rangée `ty`) qui contient `tx` : de sa case
/// la plus à gauche à la plus à droite.
fn corniche(n: &Niveau, tx: i32, ty: i32) -> (i32, i32) {
    let dessus = |x: i32| n.est_solide(x, ty) && !n.est_solide(x, ty - 1);
    let (mut g, mut d) = (tx, tx);
    while dessus(g - 1) {
        g -= 1;
    }
    while dessus(d + 1) {
        d += 1;
    }
    (g, d)
}

/// Pépin monte la tour appui par appui (2 s sur chacun, au milieu, puis à
/// gauche, puis à droite) ; on regarde à chaque pas tous les objets en cours.
/// Rend, pour chaque pas, (le pas, les objets en cours).
fn montee_par_appuis(n: &Niveau, graine: u32, facile: bool) -> Vec<(u16, Vec<Chute>)> {
    let mut p = en_montee_mode(n, graine, facile);
    let mut vus = Vec::new();
    for (y, g, d) in appuis_de_la_tour(&p, n) {
        for x in [(g + d) / 2.0, g + 6.0, d - 6.0] {
            p.voir_pepin(debout(x, y));
            for _ in 0..40 {
                pas(&mut p, n, 1);
                if p.phase() != PhasePieuvre::Montee {
                    return vus;
                }
                vus.push((p.ecoule(), p.chutes().to_vec()));
            }
        }
    }
    vus
}

/// TOUTE CHUTE EST ANNONCÉE au moins 0,8 s avant de devenir dangereuse, et il
/// y a TOUJOURS UNE ISSUE : sur une corniche, les objets annoncés ou qui
/// tombent laissent au moins 2 cases de suite libres. Pour 12 graines, en
/// normal et en facile, Pépin sur chaque appui de la tour. Le rythme augmente
/// avec la hauteur, et passé la moitié de la tour, deux objets par lancer
/// (jamais en facile).
#[test]
fn toute_chute_est_annoncee_et_laisse_une_issue() {
    let n = par_cle("pieuvre").charger().unwrap();
    let t = TAILLE_TUILE;
    let mut total = [0usize; 2];
    let mut doubles = [0usize; 2];
    for facile in [false, true] {
        for graine in 1..=12 {
            let mut lancers: std::collections::HashMap<u16, usize> = Default::default();
            for (k, chutes) in montee_par_appuis(&n, graine, facile) {
                for c in &chutes {
                    assert!(
                        c.lache() - c.debut >= ANNONCE_MIN_PAS,
                        "annonce de {} pas",
                        c.lache() - c.debut
                    );
                    assert!(c
                        .etat(c.debut)
                        .is_some_and(|e| matches!(e, EtatChute::Annonce(_))));
                    if c.debut == k {
                        *lancers.entry(k).or_default() += 1;
                    }
                }
                // Les objets dangereux (annoncés ou qui tombent), par corniche.
                let actifs: Vec<&Chute> = chutes
                    .iter()
                    .filter(|c| {
                        matches!(c.etat(k), Some(EtatChute::Annonce(_) | EtatChute::Chute(_)))
                    })
                    .collect();
                for c in &actifs {
                    let ty = (c.sol / t).round() as i32;
                    if !n.est_solide(c.colonne(), ty) {
                        continue; // dans la mer
                    }
                    let (g, d) = corniche(&n, c.colonne(), ty);
                    let prises: Vec<i32> = actifs
                        .iter()
                        .filter(|o| (o.sol - c.sol).abs() < 0.01)
                        .map(|o| o.colonne())
                        .collect();
                    let mut suite = 0;
                    let issue = (g..=d).any(|x| {
                        suite = if prises.contains(&x) { 0 } else { suite + 1 };
                        suite >= PLACE_LIBRE_MIN
                    });
                    assert!(
                        issue,
                        "graine {graine}, facile {facile}, pas {k} : corniche {g}..{d} rangée {ty}, objets {prises:?}"
                    );
                }
            }
            let i = usize::from(facile);
            total[i] += lancers.values().sum::<usize>();
            doubles[i] += lancers.values().filter(|&&v| v >= 2).count();
        }
    }
    eprintln!(
        "objets lancés : normal {} ({} doubles), facile {} ({} doubles)",
        total[0], doubles[0], total[1], doubles[1]
    );
    assert!(total[0] > 12 * 10, "des objets tombent : {total:?}");
    assert!(doubles[0] > 0, "des doubles en normal");
    assert_eq!(doubles[1], 0, "jamais deux à la fois en facile");
    assert!(total[1] < total[0], "moins d'objets en facile : {total:?}");
    let p = pieuvre(&n);
    assert!(p.intervalle(1.0) < p.intervalle(0.5) && p.intervalle(0.5) < p.intervalle(0.0));
    const { assert!(ANNONCE_CHUTE_PAS >= ANNONCE_MIN_PAS && ANNONCE_MIN_PAS >= 48) };
}

/// Chaque objet vise près de Pépin : le premier d'un lancer dans sa colonne ou
/// une voisine, les autres à au plus `ECART_OBJETS` colonnes, et leur ombre se
/// voit (le sol au plus 3 tuiles sous ses pieds, sauf le premier qui tombe là
/// où il est).
#[test]
fn les_objets_tombent_pres_de_pepin() {
    let n = par_cle("pieuvre").charger().unwrap();
    let t = TAILLE_TUILE;
    let mut p = en_montee(&n, 3);
    let mut vus = 0;
    for (y, g, d) in appuis_de_la_tour(&p, &n) {
        let x = (g + d) / 2.0;
        p.voir_pepin(debout(x, y));
        for _ in 0..120 {
            pas(&mut p, &n, 1);
            if p.phase() != PhasePieuvre::Montee {
                break;
            }
            let k = p.ecoule();
            for c in p.chutes().iter().filter(|c| c.debut == k) {
                vus += 1;
                let dx = (c.colonne() - (x / t).floor() as i32).abs();
                assert!(dx <= ECART_OBJETS, "à {dx} colonnes");
                assert!(c.depart < c.sol, "il tombe vers son sol");
            }
        }
    }
    assert!(vus > 20, "{vus} objets");
}

/// LA PIEUVRE AU LOIN ne blesse jamais : pendant toute une montée, sa tête
/// (au-dessus de l'eau) ne tue pas Pépin et ne se laisse pas frapper ; elle
/// est toujours là, à moitié dans la mer.
#[test]
fn la_pieuvre_au_loin_ne_blesse_jamais() {
    let n = par_cle("pieuvre").charger().unwrap();
    let mut p = en_montee(&n, 4);
    let appuis = appuis_de_la_tour(&p, &n);
    let mut vus = 0;
    for (y, g, d) in appuis {
        p.voir_pepin(debout((g + d) / 2.0, y));
        for _ in 0..60 {
            pas(&mut p, &n, 1);
            if p.phase() != PhasePieuvre::Montee {
                break;
            }
            let tete = p
                .au_loin()
                .expect("elle est au loin pendant toute la montée");
            let mer = p.surface_mer().unwrap();
            assert!(tete.y < mer && tete.bas() > mer, "à moitié dans la mer");
            // Le haut de sa tête, hors de l'eau.
            let hors_de_l_eau = Aabb::new(tete.x, tete.y, tete.w, mer - tete.y - 1.0);
            let k = p.ecoule();
            let objet = p
                .chutes()
                .iter()
                .any(|c| c.boite(k).is_some_and(|b| b.chevauche(&hors_de_l_eau)));
            if !objet {
                vus += 1;
                assert!(!p.blesse(&hors_de_l_eau), "elle ne touche jamais Pépin");
            }
            let mut j = Joueur::new(tete.x + 19.0, tete.y - 12.0);
            j.vitesse.y = 100.0;
            assert!(!p.recoit(&j), "elle n'est pas attaquable");
        }
    }
    assert!(vus > 200);
}

/// LA FIN : Pépin entre dans le sommet : elle s'enfuit (la plongée), plus rien
/// ne tombe, la mer redescend et ne tue plus ; puis elle est « vaincue » : le
/// niveau est fini comme avant (`Entite::boss_vaincu`).
#[test]
fn au_sommet_elle_s_enfuit_puis_le_niveau_finit() {
    let n = par_cle("pieuvre").charger().unwrap();
    let mut p = en_montee(&n, 5);
    let sommet = p.sommet.unwrap();
    // Un peu de montée d'abord (des objets en cours).
    p.voir_pepin(debout(
        sommet.x + 20.0,
        p.bache.unwrap().y - 2.0 * TAILLE_TUILE,
    ));
    pas(&mut p, &n, 600);
    let mer = p.surface_mer().unwrap();
    p.voir_pepin(debout(sommet.x + 20.0, sommet.bas()));
    pas(&mut p, &n, 1);
    assert_eq!(p.phase(), PhasePieuvre::Plongee, "elle s'enfuit");
    assert!(p.chutes().is_empty(), "plus rien ne tombe");
    assert!(!p.est_vaincue());
    let mut avant = p.surface_mer().unwrap();
    assert!((avant - mer).abs() < 2.0 * VITESSE_MER * RATTRAPAGE_MER + 0.01);
    let figee = avant;
    let mut gauche = Vec::new();
    for _ in 0..PLONGEE_PAS {
        let s = p.surface_mer().unwrap();
        assert!(s >= avant - 0.01, "la mer redescend");
        assert!(
            !p.blesse(&debout(100.0, s + 3.0)),
            "la mer calmée ne tue plus"
        );
        gauche.push(p.au_loin());
        avant = s;
        pas(&mut p, &n, 1);
    }
    assert!(p.surface_mer().unwrap() >= figee + DESCENTE_MER - 0.01);
    assert!(p.est_vaincue(), "puis le chapitre est fini");
    assert!(Entite::Pieuvre(Box::new(p.clone())).boss_vaincu());
    // Elle plonge : sa tête descend, puis disparaît.
    let ys: Vec<f32> = gauche.iter().flatten().map(|b| b.y).collect();
    assert!(ys.windows(2).all(|w| w[1] >= w[0]) && ys.last() > ys.first());
    assert!(p.au_loin().is_none(), "partie au large");
}

/// MÊME SUITE D'ENTRÉES, MÊME RÉSULTAT : deux montées de même graine, où Pépin
/// fait la même chose, sont identiques au pas près (la mer, chaque objet) ;
/// une autre graine en donne une autre.
#[test]
fn meme_suite_meme_resultat() {
    let n = par_cle("pieuvre").charger().unwrap();
    let a = montee_par_appuis(&n, 7, false);
    let b = montee_par_appuis(&n, 7, false);
    assert_eq!(a, b);
    let c = montee_par_appuis(&n, 8, false);
    assert_ne!(a, c, "une autre graine, d'autres objets");
}

/// LE FACILE EST PLUS LENT : la mer monte moins vite, les objets sont annoncés
/// plus longtemps et plus espacés, et l'écart permis est plus grand.
#[test]
fn le_facile_est_plus_lent() {
    let n = par_cle("pieuvre").charger().unwrap();
    let (normal, facile) = (en_montee_mode(&n, 1, false), en_montee_mode(&n, 1, true));
    let crue = normal.fin_de_crue();
    assert_eq!(crue, facile.fin_de_crue());
    let monte = |p: &Pieuvre| p.surface_mer_a(crue) - p.surface_mer_a(crue + 600);
    assert!(monte(&facile) < monte(&normal));
    assert!((monte(&facile) / monte(&normal) - FACTEUR_FACILE).abs() < 0.01);
    for h in [0.0, 0.5, 1.0] {
        assert!(facile.intervalle(h) > normal.intervalle(h));
    }
    assert!(facile.ecart_mer_max() > normal.ecart_mer_max());
    let annonce = |v: Vec<(u16, Vec<Chute>)>| {
        v.into_iter()
            .flat_map(|(_, c)| c)
            .map(|c| c.annonce)
            .max()
            .unwrap()
    };
    assert!(annonce(montee_par_appuis(&n, 1, true)) > annonce(montee_par_appuis(&n, 1, false)));
}

/// Une mort ramène à la dernière lanterne : la mer revient sous elle (au plus
/// `MER_A_LA_REPRISE`), et ne la noie pas d'emblée ; les objets en cours
/// disparaissent, le tirage change. Depuis la lanterne du bas, tout repart à
/// neuf.
#[test]
fn reprise_a_une_lanterne() {
    let n = par_cle("pieuvre").charger().unwrap();
    let mut p = en_montee(&n, 6);
    let bache = p.bache.unwrap();
    let mut lanternes: Vec<(f32, f32)> = n
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Lanterne(l) if l.boite.y < bache.y => Some(l.point_de_reapparition()),
            _ => None,
        })
        .collect();
    lanternes.sort_by(|a, b| a.1.total_cmp(&b.1));
    // La 2e lanterne en partant du haut, où Pépin est passé.
    let l = lanternes[1];
    let j = Joueur::new(l.0, l.1);
    p.voir_pepin(j.boite);
    pas(&mut p, &n, 200);
    assert!(!p.chutes().is_empty(), "des objets en cours");
    let tentative = p.tentative();
    let mut entites = n.entites.clone();
    for e in &mut entites {
        if let Entite::Pieuvre(q) = e {
            **q = p.clone();
        }
    }
    pepin_core::entities::reprendre_apres_une_mort(&mut entites, &j.boite);
    let mut q = entites
        .iter()
        .find_map(|e| match e {
            Entite::Pieuvre(p) => Some((**p).clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(q.phase(), PhasePieuvre::Montee);
    assert_eq!(q.tentative(), tentative + 1);
    assert!(q.chutes().is_empty());
    let mer = q.surface_mer().unwrap();
    assert!(
        mer > j.boite.bas() && mer <= j.boite.bas() + MER_A_LA_REPRISE + 1.0,
        "mer {mer}, pieds {}",
        j.boite.bas()
    );
    q.voir_pepin(j.boite);
    pas(&mut q, &n, 60);
    assert!(
        q.surface_mer().unwrap() > j.boite.bas(),
        "pas noyé d'emblée"
    );
    // Depuis la lanterne du bas : tout repart à neuf.
    let bas = Joueur::new(40.0, bache.y - 40.0);
    pepin_core::entities::reprendre_apres_une_mort(&mut entites, &bas.boite);
    let r = entites
        .iter()
        .find_map(|e| match e {
            Entite::Pieuvre(p) => Some((**p).clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!((r.horloge_mer(), r.objets_lances()), (0, 0));
}

/// Les lanternes de la tour : une tous les 20 à 25 % de la hauteur, de la
/// lanterne du bas jusqu'au sommet (et la dernière à moins de 25 % de lui).
#[test]
fn les_lanternes_de_la_tour_sont_regulieres() {
    let n = par_cle("pieuvre").charger().unwrap();
    let p = pieuvre(&n);
    let bache = p.bache.unwrap();
    let mut bas: Vec<f32> = n
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Lanterne(l) if l.boite.y < bache.y => Some(l.boite.bas()),
            _ => None,
        })
        .collect();
    bas.sort_by(|a, b| b.total_cmp(a));
    let sommet = p.sommet.unwrap().bas();
    let hauteur = bas[0] - sommet;
    for w in bas.windows(2) {
        let part = (w[0] - w[1]) / hauteur;
        assert!(
            (0.2..=0.25).contains(&part),
            "{part} entre {} et {}",
            w[0],
            w[1]
        );
    }
    let dernier = (bas[bas.len() - 1] - sommet) / hauteur;
    assert!(dernier <= 0.25, "{dernier}");
}
