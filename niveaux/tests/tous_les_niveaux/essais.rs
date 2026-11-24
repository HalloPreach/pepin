//! Les niveaux d'ESSAI (un par mécanique nouvelle) : chacun est faisable, et
//! seulement grâce à SA mécanique (autrefois dans `core/src/zones.rs`).

use crate::commun::*;
use crate::faisabilite::verifier_infos;

/// Le niveau d'essai vertical (45 rangées) passe les mêmes contrôles qu'un vrai
/// niveau : arrivée, 3 gouttes, lanternes, robot maladroit compris.
#[test]
#[ignore = "lot : recherche complète du robot de faisabilité"]
fn niveau_d_essai_vertical_faisable() {
    verifier_infos(par_cle("essai_vertical"));
}

/// Le niveau d'essai « vent » passe les mêmes contrôles : la traversée du vide
/// n'est faisable que parce que le robot simule les vents latéraux.
#[test]
fn niveau_d_essai_vent_faisable() {
    verifier_infos(par_cle("essai_vent"));
}

/// Sans vents latéraux, le vide du niveau d'essai est trop large : le niveau
/// doit être bloqué (c'est ce qui prouve que le vent fait la traversée).
#[test]
fn niveau_d_essai_vent_exige_les_vents() {
    let niv = par_cle("essai_vent").charger().unwrap();
    assert!(!niv.vents.is_empty());
    let r = analyser(
        &niv,
        niv.depart,
        Options {
            vents: false,
            ..Options::default()
        },
    );
    assert!(!r.arrivee, "sans vent le vide doit rester infranchissable");
}

/// Le niveau d'essai « grêle » passe les mêmes contrôles (arrivée, 3 gouttes,
/// lanternes, robot maladroit) : le robot traverse les trois rideaux.
#[test]
fn niveau_d_essai_grele_faisable() {
    verifier_infos(par_cle("essai_grele"));
}

/// Les quatre colonnes du niveau d'essai « grêle » sont bien là où le dessin le
/// dit : du ciel au toit (colonne 9), du ciel au bas de la carte au-dessus du
/// vide (colonnes 21 et 22), et arrêtée par la corniche (colonne 28, ligne 5).
/// Et sans parapluie, le niveau est bloqué.
#[test]
fn niveau_d_essai_grele_a_ses_trois_rideaux() {
    use pepin_core::entities::Entite;
    let niv = par_cle("essai_grele").charger().unwrap();
    let t = pepin_core::aabb::TAILLE_TUILE;
    let mut colonnes: Vec<(i32, f32, f32)> = niv
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Grele(g) => Some(((g.zone.x / t) as i32, g.zone.y / t, g.zone.bas() / t)),
            _ => None,
        })
        .collect();
    colonnes.sort_by_key(|c| c.0);
    assert_eq!(
        colonnes,
        vec![
            (9, 1.0, 8.0),
            (21, 2.0, 11.0),
            (22, 2.0, 11.0),
            (28, 1.0, 5.0)
        ]
    );
    let r = analyser(
        &niv,
        niv.depart,
        Options {
            planer: false,
            ..Options::default()
        },
    );
    assert!(!r.arrivee, "sans parapluie, la grêle et le vide bloquent");
}

/// Le niveau d'essai « auvent » passe les mêmes contrôles (arrivée, 3 gouttes,
/// lanternes, robot maladroit) : le robot enchaîne les six auvents.
#[test]
fn niveau_d_essai_auvent_faisable() {
    verifier_infos(par_cle("essai_auvent"));
}

/// Six auvents au-dessus du vide, aux colonnes 9, 13, 17, 22, 25 et 29 (écarts
/// de 4, 4, 5, 3 et 4 tuiles) ; sans eux, le vide est infranchissable : c'est
/// bien la chaîne d'auvents qui fait la traversée.
#[test]
fn niveau_d_essai_auvent_exige_les_auvents() {
    use pepin_core::entities::Entite;
    let niv = par_cle("essai_auvent").charger().unwrap();
    let t = pepin_core::aabb::TAILLE_TUILE;
    let colonnes: Vec<i32> = niv
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Auvent(a) => Some((a.boite.x / t) as i32),
            _ => None,
        })
        .collect();
    assert_eq!(colonnes, vec![9, 13, 17, 22, 25, 29]);
    let mut sans = niv.clone();
    sans.entites.retain(|e| !matches!(e, Entite::Auvent(_)));
    let r = analyser(&sans, sans.depart, Options::default());
    assert!(
        !r.arrivee,
        "sans auvents, le vide doit rester infranchissable"
    );
}

/// Le niveau d'essai « ventilateur » passe les mêmes contrôles (arrivée, 3
/// gouttes, lanternes, robot maladroit) : le robot monte par le souffle.
#[test]
#[ignore = "lot : recherche complète du robot de faisabilité"]
fn niveau_d_essai_ventilateur_faisable() {
    verifier_infos(par_cle("essai_ventilateur"));
}

/// Sans les courants ascendants (le souffle du ventilateur), le balcon du
/// drapeau est hors de portée : c'est bien le ventilateur seul qui fait la
/// montée (le saut ne porte pas à 18 tuiles).
#[test]
fn niveau_d_essai_ventilateur_exige_les_courants() {
    let niv = par_cle("essai_ventilateur").charger().unwrap();
    let r = analyser(
        &niv,
        niv.depart,
        Options {
            courants: false,
            ..Options::default()
        },
    );
    assert!(
        !r.arrivee,
        "sans le souffle du ventilateur, le balcon doit rester hors de portée"
    );
}

/// Le niveau d'essai « montgolfière » passe les mêmes contrôles (arrivée, 3
/// gouttes, lanternes, robot maladroit) : le robot gravit les deux étages en
/// montgolfière.
#[test]
fn niveau_d_essai_montgolfiere_faisable() {
    verifier_infos(par_cle("essai_montgolfiere"));
}

/// Sans les montgolfières (les `b` remplacés par du vide), aucun étage n'est à
/// portée : ni l'étage B depuis le quai (112 px), ni l'étage du drapeau depuis
/// l'étage B (64 px, plus qu'un saut). Avec la PREMIÈRE seulement, l'étage B est
/// atteint mais pas le drapeau : chacune des deux sert.
#[test]
fn niveau_d_essai_montgolfiere_exige_les_deux_montgolfieres() {
    let ascii = par_cle("essai_montgolfiere").ascii;
    let sans = Niveau::depuis_ascii(&ascii.replace('b', ".")).unwrap();
    let r = analyser(&sans, sans.depart, Options::default());
    assert!(
        !r.arrivee,
        "sans montgolfière, le drapeau doit rester hors de portée"
    );
    // La seconde (ligne 9) retirée, la première (ligne 16) gardée.
    let lignes: Vec<String> = ascii
        .lines()
        .enumerate()
        .map(|(i, l)| {
            if i == 10 {
                l.replace('b', ".")
            } else {
                l.to_string()
            }
        })
        .collect();
    let une = Niveau::depuis_ascii(&lignes.join("\n")).unwrap();
    assert_eq!(
        une.entites
            .iter()
            .filter(|e| matches!(e, pepin_core::entities::Entite::Montgolfiere(_)))
            .count(),
        1
    );
    let r = analyser(&une, une.depart, Options::default());
    assert!(
        !r.arrivee,
        "sans la seconde, le drapeau doit rester hors de portée"
    );
    assert!(
        r.gouttes[1],
        "mais la première porte bien à l'étage B : {r:?}"
    );
}

/// Le niveau d'essai « escargot » passe les mêmes contrôles (arrivée, 3 gouttes,
/// lanternes, robot maladroit) : le robot gravit les deux falaises en
/// rebondissant sur les escargots.
#[test]
fn niveau_d_essai_escargot_faisable() {
    verifier_infos(par_cle("essai_escargot"));
}

/// Sans les escargots (les `s` remplacés par du vide), la falaise 1 est hors de
/// portée (80 px, plus qu'un saut). Avec le PREMIER seulement, la falaise 1 est
/// atteinte mais pas la falaise 2 (trop loin, même en planant depuis la
/// falaise 1) : chacun des deux sert.
#[test]
#[ignore = "lot : le niveau exige sa mécanique (recherche complète sans elle)"]
fn niveau_d_essai_escargot_exige_les_deux_escargots() {
    let ascii = par_cle("essai_escargot").ascii;
    let sans = Niveau::depuis_ascii(&ascii.replace('s', ".")).unwrap();
    let r = analyser(&sans, sans.depart, Options::default());
    // Gouttes dans l'ordre de lecture : 0 en l'air (ligne 1), 1 sur la falaise 2
    // (ligne 2), 2 sur la falaise 1 (ligne 3).
    assert!(
        !r.arrivee && r.gouttes == [false; 3],
        "sans escargot : {r:?}"
    );
    // Le second (colonne 28) retiré, le premier gardé.
    let lignes: Vec<String> = ascii
        .lines()
        .map(|l| {
            let mut c: Vec<char> = l.chars().collect();
            if c.len() > 28 && c[28] == 's' {
                c[28] = '.';
            }
            c.into_iter().collect()
        })
        .collect();
    let un = Niveau::depuis_ascii(&lignes.join("\n")).unwrap();
    assert_eq!(
        un.entites
            .iter()
            .filter(|e| matches!(e, pepin_core::entities::Entite::Escargot(_)))
            .count(),
        1
    );
    let r = analyser(&un, un.depart, Options::default());
    assert!(
        !r.arrivee,
        "sans le second, le drapeau doit rester hors de portée"
    );
    assert!(
        r.gouttes[0] && r.gouttes[2] && !r.gouttes[1],
        "mais le premier porte bien sur la falaise 1 (et à la goutte en l'air) : {r:?}"
    );
}

/// Le niveau d'essai « chat » passe les mêmes contrôles (arrivée, 3 gouttes,
/// lanternes, robot maladroit) : le robot tombe du toit bas sur le chat
/// endormi, l'écrase, et passe dessous le grand mur.
#[test]
#[ignore = "lot : recherche complète du robot de faisabilité"]
fn niveau_d_essai_chat_faisable() {
    verifier_infos(par_cle("essai_chat"));
}

/// La case du chat est le SEUL passage : bouchée par un toit (`#` à la place
/// du `z`), ni la goutte cachée ni le drapeau ne sont atteints ; seule la
/// goutte du haut de la fente l'est. (Le chat vivant, lui, ne se traverse pas : voir
/// `faisabilite::tests::marcher_dans_un_chat_vivant_tue_le_robot`.)
#[test]
fn niveau_d_essai_chat_le_chat_garde_le_seul_passage() {
    let bouche = Niveau::depuis_ascii(&par_cle("essai_chat").ascii.replace('z', "#")).unwrap();
    let r = analyser(&bouche, bouche.depart, Options::default());
    // Gouttes dans l'ordre de lecture : 0 en haut de la fente (ligne 6, encore
    // atteinte : on se pose sur le toit qui bouche la case), 1 cachée derrière
    // le chat, 2 avant le drapeau (ligne 8).
    assert!(
        !r.arrivee && r.gouttes == [true, false, false],
        "passage bouché : {r:?}"
    );
}

/// Le niveau d'essai « horloge » passe les mêmes contrôles (arrivée, 3 gouttes,
/// lanternes, robot maladroit) : le robot se laisse porter par les deux
/// aiguilles d'une tour à l'autre.
#[test]
#[ignore = "lot : recherche complète du robot de faisabilité"]
fn niveau_d_essai_horloge_faisable() {
    verifier_infos(par_cle("essai_horloge"));
}

/// Les deux aiguilles servent. Sans aiguille (les `h` vidés), ni la tour B ni
/// le drapeau ne sont atteints : seule la goutte du sol l'est. Sans la petite
/// (la 1re, en haut), la tour B l'est (goutte 1) mais pas la tour C.
#[test]
#[ignore = "lot : le niveau exige sa mécanique (recherche complète sans elle)"]
fn niveau_d_essai_horloge_exige_les_deux_aiguilles() {
    let ascii = par_cle("essai_horloge").ascii;
    let sans = Niveau::depuis_ascii(&ascii.replace('h', ".")).unwrap();
    let r = analyser(&sans, sans.depart, Options::default());
    // Gouttes dans l'ordre de lecture : 0 sur la tour C, 1 sur la tour B, 2 au sol.
    assert!(
        !r.arrivee && r.gouttes == [false, false, true],
        "sans aiguille : {r:?}"
    );
    let grande = Niveau::depuis_ascii_avec_aiguilles(
        &ascii.replacen('h', ".", 1),
        &par_cle("essai_horloge").aiguilles[1..],
    )
    .unwrap();
    let r = analyser(&grande, grande.depart, Options::default());
    assert!(
        !r.arrivee && r.gouttes == [false, true, true],
        "sans la petite aiguille : {r:?}"
    );
}

/// L'arène du Grand Nuage (essai) : pas de drapeau, l'« arrivée » du robot est
/// la tête du nuage (essoufflé, ses deux courants actifs : voir l'en-tête de
/// `faisabilite`). Il l'atteint, précis comme maladroit, prend les 3 gouttes ;
/// une seule lanterne (le début de l'arène : on y reprend après une mort), d'où
/// il l'atteint aussi.
#[test]
#[ignore = "lot : recherche complète du robot de faisabilité"]
fn niveau_d_essai_nuage_faisable() {
    let niv = par_cle("essai_nuage").charger().unwrap();
    let r = analyser(&niv, niv.depart, Options::default());
    assert!(r.arrivee && r.gouttes == [true; 3], "{r:?}");
    assert!(r.lanternes.iter().all(|&l| l) && r.lanternes.len() == 1);
    let maladroit = Options {
        precis: false,
        ..Options::default()
    };
    assert!(analyser(&niv, niv.depart, maladroit).arrivee);
}

/// Chacun des deux courants suffit pour atteindre sa tête (ils alternent d'un
/// motif à l'autre). Sans courant, la tête est hors de portée (80 px au-dessus
/// du sol), et seule la goutte du sol est prise.
#[test]
#[ignore = "lot : recherche complète du robot de faisabilité"]
fn niveau_d_essai_nuage_chaque_courant_mene_a_la_tete() {
    use pepin_core::entities::Entite;
    let niv = par_cle("essai_nuage").charger().unwrap();
    // Un courant « éteint » : réduit à une zone vide, sous le sol.
    let eteint = pepin_core::aabb::Aabb::new(0.0, 400.0, 0.0, 0.0);
    let avec = |gauche: bool, droite: bool| {
        let mut n = niv.clone();
        for e in &mut n.entites {
            if let Entite::Nuage(g) = e {
                if !gauche {
                    g.courants[0] = eteint;
                }
                if !droite {
                    g.courants[1] = eteint;
                }
            }
        }
        analyser(&n, n.depart, Options::default())
    };
    // Gouttes dans l'ordre de lecture : 0 en haut à gauche, 1 en haut à droite,
    // 2 au sol.
    let gauche = avec(true, false);
    assert!(
        gauche.arrivee && gauche.gouttes == [true, false, true],
        "{gauche:?}"
    );
    let droite = avec(false, true);
    assert!(
        droite.arrivee && droite.gouttes == [false, true, true],
        "{droite:?}"
    );
    let aucun = avec(false, false);
    assert!(
        !aucun.arrivee && aucun.gouttes == [false, false, true],
        "{aucun:?}"
    );
}
