//! Tests propres au niveau « clocher » (autrefois le `mod tests` de
//! `core/src/levels/clocher.rs` ; la grille et ses commentaires de conception
//! sont dans son fichier `.ron`).

use crate::commun::*;
use pepin_core::entities::Entite;
use pepin_core::faisabilite::{analyser, Options};
use pepin_core::level::Niveau;

/// Le niveau de ce module, lu dans son fichier `.ron`.
fn infos() -> &'static InfosNiveau {
    par_cle("clocher")
}

/// Le clocher avec les lettres `lettre` remplacées par du vide, ligne par ligne
/// (`garder(ligne, colonne)` : `true` pour garder celle-ci). Les aiguilles
/// gardées reçoivent leur réglage, dans l'ordre de lecture.
fn sans(lettre: char, garder: impl Fn(usize, usize) -> bool) -> Niveau {
    let mut reglages = Vec::new();
    let mut k = 0;
    let lignes: Vec<String> = infos()
        .ascii
        .lines()
        .filter(|l| !l.is_empty())
        .enumerate()
        .map(|(r, l)| {
            l.chars()
                .enumerate()
                .map(|(c, ch)| {
                    if ch == 'h' {
                        k += 1;
                    }
                    if ch == lettre && !garder(r, c) {
                        '.'
                    } else {
                        if ch == 'h' {
                            reglages.push(infos().aiguilles[k - 1]);
                        }
                        ch
                    }
                })
                .collect()
        })
        .collect();
    Niveau::depuis_ascii_avec_aiguilles(&lignes.join("\n"), &reglages).unwrap()
}

/// Gouttes dans l'ordre de lecture : 0 sur le pilier de la cage (ligne 6),
/// 1 sur la corniche F1 (ligne 28), 2 en l'air au-dessus du socle (ligne 31).
///
/// Les escargots servent : sans eux, Pépin reste au pied de la tour (le socle
/// est 80 px plus haut, plus qu'un saut) ; sans celui du socle, le niveau se
/// finit mais la goutte en l'air n'est pas prise (seul son rebond y porte).
#[test]
#[ignore = "lot : le niveau exige sa mécanique (recherche complète sans elle)"]
fn le_clocher_exige_ses_escargots() {
    let niv = sans('s', |_, _| false);
    let r = analyser(&niv, niv.depart, Options::default());
    assert!(
        !r.arrivee && r.gouttes == [false; 3],
        "sans escargot : {r:?}"
    );
    let niv = sans('s', |ligne, _| ligne != 37);
    let r = analyser(&niv, niv.depart, Options::default());
    assert!(
        r.arrivee && r.gouttes == [true, true, false],
        "sans l'escargot du socle : {r:?}"
    );
}

/// Sans le souffle du ventilateur, la galerie est un cul-de-sac : la corniche
/// F1 est 128 px plus haut (seule la goutte en l'air, avant, est prise).
#[test]
#[ignore = "lot : le niveau exige sa mécanique (recherche complète sans elle)"]
fn le_clocher_exige_le_ventilateur() {
    let n = infos().charger().unwrap();
    let sans_courant = Options {
        courants: false,
        ..Options::default()
    };
    let r = analyser(&n, n.depart, sans_courant);
    assert!(
        !r.arrivee && r.gouttes == [false, false, true],
        "sans courant : {r:?}"
    );
}

/// Le moment fort, la cage de l'horloge, exige ses deux aiguilles : sans
/// aiguille, le pilier (et sa goutte) est hors de portée ; sans la petite
/// (la 1re en lisant), le pilier l'est, la tour du drapeau non.
#[test]
#[ignore = "lot : le niveau exige sa mécanique (recherche complète sans elle)"]
fn le_clocher_exige_ses_deux_aiguilles() {
    let niv = sans('h', |_, _| false);
    let r = analyser(&niv, niv.depart, Options::default());
    assert!(
        !r.arrivee && r.gouttes == [false, true, true],
        "sans aiguille : {r:?}"
    );
    let niv = sans('h', |ligne, _| ligne != 6);
    let r = analyser(&niv, niv.depart, Options::default());
    assert!(
        !r.arrivee && r.gouttes == [true; 3],
        "sans la petite aiguille : {r:?}"
    );
}

/// Aucune aiguille ne passe là où Pépin se tient debout sur une tuile : il
/// serait « ramassé » par la plaque (le parseur ne le refuse pas, voir la note
/// de la carte 5.10). Sur un tour complet, image par image, la plaque ne touche
/// jamais les 14 px au-dessus d'un dessus de tuile.
#[test]
fn aucune_aiguille_ne_ramasse_pepin_debout() {
    use pepin_core::aabb::{Aabb, TAILLE_TUILE as T};
    use pepin_core::physics::JOUEUR_H;
    let n = infos().charger().unwrap();
    let solide = |tx: usize, ty: usize| n.tuile_decor(tx as i32, ty as i32).est_solide();
    let mut aiguilles = 0;
    for e in &n.entites {
        let Entite::Mobile(m) = e else { continue };
        let Some(tour) = m.cercle().map(|c| c.pas_par_tour().ceil() as u32) else {
            continue;
        };
        aiguilles += 1;
        let mut m = m.clone();
        for pas in 0..=tour {
            m.tourner(1);
            for ty in 1..n.hauteur() {
                for tx in 0..n.largeur() {
                    if !solide(tx, ty) || solide(tx, ty - 1) {
                        continue;
                    }
                    let debout = Aabb::new(tx as f32 * T, ty as f32 * T - JOUEUR_H, T, JOUEUR_H);
                    assert!(
                        !debout.chevauche(&m.boite),
                        "au pas {pas}, l'aiguille {:?} passe sur la tuile ({tx}, {ty})",
                        m.boite
                    );
                }
            }
        }
    }
    assert_eq!(aiguilles, 2);
}

/// Les pièces du clocher sont là où le commentaire le dit : deux escargots, un
/// ventilateur, deux aiguilles (pivots, rayons, heures), les dix pointes de la
/// poutre, sous la petite aiguille, et celle du sol de la cage.
#[test]
fn les_pieces_du_clocher_a_leur_place() {
    let n = infos().charger().unwrap();
    assert_eq!((n.largeur(), n.hauteur()), (34, 45));
    let t = pepin_core::aabb::TAILLE_TUILE;
    let case = |x: f32, y: f32| ((x / t) as i32, (y / t) as i32);
    let mut escargots = Vec::new();
    let mut ventilateurs = Vec::new();
    let mut aiguilles = Vec::new();
    let mut pointes = Vec::new();
    for e in &n.entites {
        match e {
            Entite::Pointe(p) => pointes.push(case(p.boite.x, p.boite.y)),
            Entite::Escargot(s) => escargots.push(case(s.boite.x, s.boite.y)),
            Entite::Ventilateur(v) => ventilateurs.push(case(v.boite.x, v.boite.y)),
            Entite::Mobile(m) => {
                if let Some(c) = m.cercle() {
                    aiguilles.push((c.pivot, c.rayon, c.tour()));
                }
            }
            _ => {}
        }
    }
    assert_eq!(escargots, vec![(10, 37), (5, 42)]);
    assert_eq!(ventilateurs, vec![(30, 36)]);
    // Les dix de la poutre (ligne 10), puis celle du sol de la cage (ligne 16,
    // carte 5.18), dans l'ordre de lecture.
    let attendues: Vec<_> = (19..=28).map(|c| (c, 10)).chain([(13, 16)]).collect();
    assert_eq!(pointes, attendues);
    assert_eq!(
        aiguilles,
        vec![
            ((392.0, 104.0), 48.0, 0.75),
            ((136.0, 184.0), 64.0, 4.0 / 12.0)
        ]
    );
}
