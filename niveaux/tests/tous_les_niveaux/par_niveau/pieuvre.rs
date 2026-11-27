//! Tests propres au niveau « pieuvre » (autrefois le `mod tests` de
//! `core/src/levels/pieuvre.rs` ; la grille et ses commentaires de conception
//! sont dans son fichier `.ron`).

use crate::commun::*;
use pepin_core::aabb::{Aabb, TAILLE_TUILE};
use pepin_core::entities::contacts::{resoudre_contacts, Contact};
use pepin_core::entities::pieuvre::{PhasePieuvre, Pieuvre, FURIE_PAS, TAILLE_SOMMET};
use pepin_core::entities::{Actualisable, Entite};
use pepin_core::level::Niveau;
use pepin_core::physics::Environnement;
use pepin_core::player::Joueur;

/// Le niveau de ce module, lu dans son fichier `.ron`.
fn infos() -> &'static InfosNiveau {
    par_cle("pieuvre")
}

/// Première rangée de l'arène : la tour et la bâche sont au-dessus.
const HAUT_ARENE: i32 = 69;

/// L'arène du boss : pas de drapeau, une seule pieuvre, six lanternes (au
/// départ, au début de la tour : carte 9.13a, puis tous les 20 à 25 % de la
/// tour : carte 9.36a), trois gouttes ; et, dans ses 11 dernières rangées,
/// EXACTEMENT la géométrie du niveau d'essai « pieuvre » (mêmes cases vides ou
/// pleines, même pieuvre, même bassin, mêmes pontons, mêmes gouttes, même
/// départ), sur laquelle le combat a été réglé et le robot de QA entraîné.
#[test]
fn l_arene_de_la_pieuvre_reprend_celle_du_niveau_d_essai() {
    let n = infos().charger().unwrap();
    let e = par_cle("essai_pieuvre").charger().unwrap();
    let dy = HAUT_ARENE as f32 * TAILLE_TUILE;
    assert_eq!((n.largeur(), n.hauteur()), (20, 80));
    assert!(n.arrivee.is_none(), "l'arène n'a pas de drapeau");
    assert_eq!(n.nb_lanternes(), 6);
    assert_eq!(n.depart, (e.depart.0, e.depart.1 + dy));
    for ty in 0..11 {
        for tx in 0..20 {
            assert_eq!(
                n.tuile_decor(tx, ty + HAUT_ARENE).est_solide(),
                e.tuile_decor(tx, ty).est_solide(),
                "case ({tx}, {ty})"
            );
        }
    }
    let decale = |b: Aabb| Aabb::new(b.x, b.y - dy, b.w, b.h);
    let arene: Vec<Aabb> = n
        .entites
        .iter()
        .map(|e| e.boite())
        .filter(|b| b.y >= dy)
        .map(decale)
        .collect();
    let essai: Vec<Aabb> = e
        .entites
        .iter()
        .filter(|e| !matches!(e, Entite::Pieuvre(_)))
        .map(|e| e.boite())
        .collect();
    assert_eq!(
        arene.len(),
        essai.len() + 1,
        "les entités de l'arène, et la pieuvre"
    );
    for b in &essai {
        assert!(arene.contains(b), "{b:?}");
    }
    let (p, q) = (pieuvre(&n), pieuvre(&e));
    assert_eq!(p.colonnes.map(decale), q.colonnes);
    assert_eq!(p.places.map(decale), q.places);
    // Carte 9.12f : les 4 appuis de l'arène (les 2 grosses plateformes des
    // bords, les 2 petites sur piliers) sont ceux qu'elle peut saisir ; la
    // tour, au-dessus de la bâche, n'en fait pas partie.
    assert_eq!(p.plateformes.len(), 4);
    assert_eq!(
        p.plateformes,
        q.plateformes
            .iter()
            .map(|c| c
                .iter()
                .map(|&(x, y)| (x, y + HAUT_ARENE))
                .collect::<Vec<_>>())
            .collect::<Vec<_>>()
    );
    // Carte 9.13a : la tour a sa bâche ; carte 9.36a : son SOMMET (4 x 3
    // cases), posé sur la plateforme du haut, et la mer s'arrête dessous.
    assert!(p.sommet.is_some() && p.bache.is_some());
    let sommet = p.sommet.unwrap();
    assert_eq!((sommet.w, sommet.h), TAILLE_SOMMET);
    let (sx, sy) = (
        (sommet.x / TAILLE_TUILE) as i32,
        (sommet.bas() / TAILLE_TUILE) as i32,
    );
    assert!(
        (sx..sx + 4).all(|x| n.tuile_decor(x, sy).est_solide()),
        "posé sur la plateforme"
    );
    assert!(p.arret_mer > sommet.bas());
    assert!(q.sommet.is_none(), "le niveau d'essai reste en une phase");
}

fn pieuvre(n: &Niveau) -> Pieuvre {
    n.entites
        .iter()
        .find_map(|e| match e {
            Entite::Pieuvre(p) => Some((**p).clone()),
            _ => None,
        })
        .unwrap()
}

/// Les trois coups de la phase 1 (la tête sortie tout de suite à chaque fois).
fn trois_coups(p: &mut Pieuvre) {
    for _ in 0..3 {
        p.sortir_la_tete(0);
        p.toucher();
    }
}

fn pas(p: &mut Pieuvre, n: &Niveau, k: u32) {
    let env = Environnement::simple(n);
    for _ in 0..k {
        p.actualiser(&env);
    }
}

/// La pieuvre de la tour au pas `t` de la montée.
fn en_montee(n: &Niveau, t: u32) -> Pieuvre {
    let mut p = pieuvre(n);
    trois_coups(&mut p);
    pas(&mut p, n, u32::from(FURIE_PAS) + t);
    p
}

/// Carte 9.13a : au 3e coup, elle entre en FURIE (la phase 2), pas en
/// plongée ; la bâche se déchire ; 2 s plus tard, le quai casse et la mer
/// monte. Le niveau d'essai, sans tour, plonge comme avant.
#[test]
fn au_troisieme_coup_elle_entre_en_furie() {
    let n = infos().charger().unwrap();
    let mut p = pieuvre(&n);
    assert!(p.bache_solide().is_some(), "la bâche tient en phase 1");
    trois_coups(&mut p);
    assert_eq!(p.phase(), PhasePieuvre::Furie);
    assert!(!p.est_vaincue());
    assert!(p.bache_solide().is_none(), "la bâche se déchire");
    assert!(p.trombe().is_some(), "la trombe monte vers la tour");
    assert!(
        p.cases_englouties().is_empty(),
        "le quai tient pendant la furie"
    );
    pas(&mut p, &n, u32::from(FURIE_PAS));
    assert_eq!(p.phase(), PhasePieuvre::Montee);
    assert_eq!(
        p.cases_englouties().len(),
        p.plateformes.iter().flatten().count()
    );
    let e = par_cle("essai_pieuvre").charger().unwrap();
    let mut q = pieuvre(&e);
    trois_coups(&mut q);
    assert_eq!(q.phase(), PhasePieuvre::Plongee);
}

/// La mer monte (vite dans l'arène, puis d'une tuile en 1,17 s, carte 9.36a :
/// plus vite qu'avant, sans palier), jusque sous le sommet ; elle TUE au
/// contact, barque comprise (des pieds 3 px sous la surface) ; en mode facile,
/// elle monte moins vite.
#[test]
fn la_mer_monte_et_tue() {
    let n = infos().charger().unwrap();
    let p = en_montee(&n, 0);
    let bache = p.bache.unwrap();
    let mut avant = p.surface_mer_a(0);
    for t in 1..6000 {
        let s = p.surface_mer_a(t);
        assert!(s <= avant, "elle ne redescend jamais (t = {t})");
        avant = s;
    }
    assert!((p.surface_mer_a(5999) - p.arret_mer).abs() < 0.01);
    let crue = p.fin_de_crue();
    assert!((p.surface_mer_a(crue) - bache.bas()).abs() < 1.0);
    // Une tuile en 70 pas (1,17 s ; la 9.13f : en 1,3 s).
    let une_tuile = p.surface_mer_a(crue) - p.surface_mer_a(crue + 70);
    assert!((une_tuile - TAILLE_TUILE).abs() < 0.5, "{une_tuile}");
    let s = p.surface_mer().unwrap();
    let pied = |bas: f32| Aabb::new(100.0, bas - 14.0, 10.0, 14.0);
    assert!(p.blesse(&pied(s + 3.0)), "la barque n'y résiste pas");
    assert!(!p.blesse(&pied(s - 40.0)));
    let mut f = pieuvre(&n);
    f.passer_en_facile();
    for _ in 0..2 {
        f.sortir_la_tete(0);
        f.toucher();
    }
    assert_eq!(f.phase(), PhasePieuvre::Furie);
    let lent = f.surface_mer_a(crue) - f.surface_mer_a(crue + 70);
    assert!((lent - 0.75 * TAILLE_TUILE).abs() < 0.5, "{lent}");
}

/// Une mort pendant la montée ramène au début de la phase 2 : la lanterne de la
/// tour s'est allumée d'elle-même (le point de reprise est au-dessus de la
/// bâche), et la mer est revenue en bas.
#[test]
fn reprise_au_debut_de_la_phase_2() {
    let n = infos().charger().unwrap();
    let p = en_montee(&n, 600);
    let bache = p.bache.unwrap();
    assert!(p.surface_mer().unwrap() < bache.y);
    let mut entites = n.entites.clone();
    for e in &mut entites {
        if let Entite::Pieuvre(q) = e {
            **q = p.clone();
        }
    }
    let mut j = Joueur::new(40.0, 100.0);
    let contacts = resoudre_contacts(&mut j, &mut entites);
    let reprise = contacts.iter().find_map(|c| match *c {
        Contact::Lanterne(x, y) => Some((x, y)),
        _ => None,
    });
    assert!(reprise.is_some_and(|(_, y)| y < bache.y), "{reprise:?}");
    pepin_core::entities::reprendre_apres_une_mort(&mut entites, &j.boite);
    let q = pieuvre_de(&entites);
    assert_eq!((q.phase(), q.ecoule()), (PhasePieuvre::Montee, 0));
    assert_eq!(q.surface_mer(), Some(q.surface_mer_a(0)));
    assert!(q.bache_solide().is_none(), "la bâche reste déchirée");
}

fn pieuvre_de(entites: &[Entite]) -> &Pieuvre {
    entites
        .iter()
        .find_map(|e| match e {
            Entite::Pieuvre(p) => Some(p.as_ref()),
            _ => None,
        })
        .unwrap()
}
