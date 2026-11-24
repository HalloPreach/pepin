//! La nouvelle partie + : chaque variante de chaque niveau reste finissable
//! (autrefois dans `core/src/levels/variantes.rs`).

use crate::commun::*;
use pepin_core::aabb::Aabb;
use pepin_core::entities::Entite;
use pepin_core::levels::variantes::*;

/// Toutes les variantes s'appliquent (cases attendues trouvées) sans toucher à la
/// géométrie, aux gouttes ni aux lanternes ; un ennemi de plus par niveau qui a
/// une variante ; aucun ennemi placé par une variante à moins de 6 cases d'une
/// reprise (lanterne ou départ) à hauteur de Pépin ; et chaque niveau de la
/// nouvelle partie + reste FINISSABLE, vent de face compris : arrivée, trois
/// gouttes et lanternes (robot précis), arrivée (robot maladroit), et arrivée
/// depuis chaque lanterne. Un fil d'exécution par niveau (`thread::scope`).
#[test]
#[ignore = "lot : faisabilité de chaque niveau de la nouvelle partie +"]
fn chaque_variante_ng_plus_est_finissable() {
    for v in VARIANTES {
        assert!(niveaux().iter().any(|n| n.cle == v.cle), "clé : {}", v.cle);
    }
    std::thread::scope(|portee| {
        for infos in niveaux() {
            portee.spawn(move || verifier_variante(infos));
        }
    });
}

fn verifier_variante(infos: &InfosNiveau) {
    let cle = infos.cle;
    let normal = infos.charger().unwrap();
    let ng = charger_ng_plus(infos).unwrap_or_else(|e| panic!("{e}"));
    let compter = |n: &Niveau, f: fn(&Entite) -> bool| n.entites.iter().filter(|e| f(e)).count();
    let gouttes = |n: &Niveau| -> Vec<Aabb> {
        n.entites
            .iter()
            .filter_map(|e| match e {
                Entite::Goutte(g) => Some(g.boite),
                _ => None,
            })
            .collect()
    };
    assert_eq!(geometrie(&ng), geometrie(&normal), "{cle} : géométrie");
    assert_eq!(ng.depart, normal.depart, "{cle} : départ");
    assert_eq!(gouttes(&ng), gouttes(&normal), "{cle} : gouttes");
    let est_lanterne: fn(&Entite) -> bool = |e| matches!(e, Entite::Lanterne(_));
    let est_ennemi: fn(&Entite) -> bool = |e| matches!(e, Entite::Ennemi(_));
    assert_eq!(
        compter(&ng, est_lanterne),
        compter(&normal, est_lanterne),
        "{cle}"
    );
    let ajouts = variante(cle).map_or(0, |v| v.ajouts.len());
    assert_eq!(
        compter(&ng, est_ennemi),
        compter(&normal, est_ennemi) + ajouts,
        "{cle}"
    );
    if let Some(v) = variante(cle) {
        assert!(!v.ajouts.is_empty(), "{cle} : un ennemi de plus");
    }

    // Pas de mort en boucle à une reprise : aucun ennemi placé par la variante
    // près d'une lanterne ou du départ.
    let reprises: Vec<(f32, f32)> = ng
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Lanterne(l) => Some(l.point_de_reapparition()),
            _ => None,
        })
        .collect();
    let t = pepin_core::aabb::TAILLE_TUILE;
    let places = variante(cle).into_iter().flat_map(|v| {
        v.deplaces
            .iter()
            .map(|&(_, vers)| vers)
            .chain(v.ajouts.iter().map(|&(_, c)| c))
    });
    for (cx, cy) in places {
        let (ex, ey) = (cx as f32 * t, cy as f32 * t);
        for &(rx, ry) in reprises.iter().chain([ng.depart].iter()) {
            let loin = (ex - rx).abs() >= 6.0 * t || (ey - ry).abs() > 2.0 * t;
            assert!(loin, "{cle} : ennemi en ({cx}, {cy}) près de ({rx}, {ry})");
        }
    }

    // Finissable (robot de faisabilité : vent de face compris). Un niveau que la
    // nouvelle partie + ne change pas du tout (aucune variante, ni éclair, ni
    // nuage, ni vent de face : le Monde 2) est déjà vérifié tel quel par
    // `zones` (faisabilité et reprise depuis chaque lanterne) : les mêmes
    // recherches ne sont pas refaites (carte 9.23, tri des tests).
    if format!("{ng:?}") == format!("{normal:?}") {
        return;
    }
    let r = analyser(&ng, ng.depart, Options::default());
    assert!(
        r.arrivee,
        "{cle} : arrivée inatteignable (x max {})",
        r.x_max
    );
    assert_eq!(r.gouttes, [true; 3], "{cle} : gouttes");
    assert!(r.lanternes.iter().all(|&l| l), "{cle} : lanternes");
    // Les autres recherches ne lisent que l'arrivée : arrêt dès qu'elle est
    // touchée (`Options::arrivee_seule`, même réponse).
    let maladroit = Options {
        precis: false,
        ..Options::arrivee_seule()
    };
    assert!(
        analyser(&ng, ng.depart, maladroit).arrivee,
        "{cle} : robot maladroit"
    );
    for &p in &reprises {
        assert!(
            analyser(&ng, p, Options::arrivee_seule()).arrivee,
            "{cle} : depuis {p:?}"
        );
    }
}

/// Toutes les tuiles de la grille, rangée par rangée (la grille elle-même est
/// privée à `core` : on la relit case par case, par `tuile_decor`).
fn geometrie(n: &Niveau) -> Vec<pepin_core::level::Tuile> {
    let (l, h) = (n.largeur() as i32, n.hauteur() as i32);
    (0..h)
        .flat_map(|ty| (0..l).map(move |tx| n.tuile_decor(tx, ty)))
        .collect()
}
