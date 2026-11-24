//! Faisabilité de chaque niveau du parcours (autrefois dans `core/src/zones.rs`) :
//! robot de faisabilité (arrivée, 3 gouttes, lanternes, robot maladroit, durée),
//! reprise depuis chaque lanterne, et ce qu'une carte de niveau lance
//! (`le_niveau_de_la_carte`, `./docker-cargo.sh carte <clé>`).

use crate::commun::*;

/// Un test de faisabilité PAR niveau (ils tournent en parallèle). La macro écrit
/// les fonctions de test et la liste des indices couverts : un niveau ajouté sans
/// sa ligne ici fait échouer `chaque_niveau_a_son_test_de_faisabilite`.
///
/// Le Monde 2 (carte 9.6) a ses huit places RÉSERVÉES d'avance (ses huit clés
/// sont décidées, voir `pepin-notes/ETAPE-9.md`) : ajouter un niveau du Monde 2
/// n'oblige à rien toucher ici. Une place encore vide (pas encore de niveau à cet
/// indice) passe sans rien vérifier.
///
/// Carte 9.23 (tri des tests) : chaque niveau a aussi SON test de reprise
/// (`reprise`, l'arrivée depuis chacune de ses lanternes), au lieu d'un seul
/// test qui faisait tous les niveaux l'un après l'autre (13 min) : ils tournent
/// en parallèle. Tous sont des tests de LOT (`./docker-cargo.sh lot`) ; une
/// carte de niveau lance les mêmes vérifications sur le sien seulement
/// (`le_niveau_de_la_carte`, `./docker-cargo.sh carte <clé>`).
macro_rules! tests_de_faisabilite {
    ($($nom:ident, $reprise:ident => $indice:expr),* $(,)?) => {
        $(
            #[test]
            #[ignore = "lot : recherche complète du robot de faisabilité (carte : le_niveau_de_la_carte)"]
            fn $nom() {
                if $indice < NB_NIVEAUX {
                    verifier($indice);
                }
            }

            #[test]
            #[ignore = "lot : reprise depuis chaque lanterne (carte : le_niveau_de_la_carte)"]
            fn $reprise() {
                if $indice < NB_NIVEAUX {
                    reprise_depuis_chaque_lanterne(&niveaux()[$indice]);
                }
            }
        )*
        const INDICES_VERIFIES: &[usize] = &[$($indice),*];
    };
}

/// Indice dans la table du 1er niveau du Monde 2.
const M2: usize = NB_NIVEAUX_MONDE_1;

tests_de_faisabilite! {
    niveau_1_faisable, niveau_1_reprise => 0,
    niveau_2_faisable, niveau_2_reprise => 1,
    niveau_3_faisable, niveau_3_reprise => 2,
    niveau_4_faisable, niveau_4_reprise => 3,
    niveau_5_faisable, niveau_5_reprise => 4,
    niveau_6_faisable, niveau_6_reprise => 5,
    niveau_7_faisable, niveau_7_reprise => 6,
    niveau_8_faisable, niveau_8_reprise => 7,
    niveau_9_faisable, niveau_9_reprise => 8,
    niveau_10_faisable, niveau_10_reprise => 9,
    niveau_11_faisable, niveau_11_reprise => 10,
    niveau_12_faisable, niveau_12_reprise => 11,
    monde_2_niveau_1_faisable, monde_2_niveau_1_reprise => M2,
    monde_2_niveau_2_faisable, monde_2_niveau_2_reprise => M2 + 1,
    monde_2_niveau_3_faisable, monde_2_niveau_3_reprise => M2 + 2,
    monde_2_niveau_4_faisable, monde_2_niveau_4_reprise => M2 + 3,
    monde_2_niveau_5_faisable, monde_2_niveau_5_reprise => M2 + 4,
    monde_2_niveau_6_faisable, monde_2_niveau_6_reprise => M2 + 5,
    monde_2_niveau_7_faisable, monde_2_niveau_7_reprise => M2 + 6,
    monde_2_niveau_8_faisable, monde_2_niveau_8_reprise => M2 + 7,
}

#[test]
fn chaque_niveau_a_son_test_de_faisabilite() {
    for (i, n) in niveaux().iter().enumerate() {
        assert!(
            INDICES_VERIFIES.contains(&i),
            "niveau d'indice {i} ({}) : aucun test de faisabilité",
            n.nom
        );
    }
}

#[test]
fn niveau_1_exige_de_planer() {
    let niv = par_cle("toits").charger().unwrap();
    let r = analyser(
        &niv,
        niv.depart,
        Options {
            planer: false,
            ..Options::default()
        },
    );
    println!("sans planer : {:?}", r);
    assert!(!r.arrivee, "le grand vide doit exiger le parapluie");
}

#[test]
fn niveau_7_exige_les_courants_ascendants() {
    let niv = par_cle("vents").charger().unwrap();
    assert!(!niv.courants.is_empty());
    let r = analyser(
        &niv,
        niv.depart,
        Options {
            courants: false,
            ..Options::default()
        },
    );
    assert!(!r.arrivee, "sans vent le niveau 7 doit être bloqué");
}

/// Le marché couvert (niveau 3) : son moment fort, six auvents au-dessus d'un
/// vide de 23 tuiles (colonnes 85 à 105, tous espacés de 4 tuiles), ne se
/// franchit pas sans eux.
#[test]
#[ignore = "lot : le niveau exige sa mécanique (recherche complète sans elle)"]
fn niveau_3_le_marche_exige_ses_six_auvents() {
    use pepin_core::entities::Entite;
    let niv = par_cle("marche").charger().unwrap();
    let t = pepin_core::aabb::TAILLE_TUILE;
    let colonnes: Vec<i32> = niv
        .entites
        .iter()
        .filter_map(|e| match e {
            Entite::Auvent(a) => Some((a.boite.x / t) as i32),
            _ => None,
        })
        .collect();
    assert_eq!(
        colonnes,
        vec![20, 40, 44, 85, 89, 93, 97, 101, 105, 135, 139]
    );
    // Sans les six auvents du moment fort (les autres restent), on bute sur le vide.
    let mut sans = niv.clone();
    sans.entites
        .retain(|e| !matches!(e, Entite::Auvent(a) if (85.0 * t..=105.0 * t).contains(&a.boite.x)));
    let r = analyser(&sans, sans.depart, Options::default());
    assert!(
        !r.arrivee,
        "sans les six auvents, le vide du moment fort doit rester infranchissable"
    );
    assert!(r.x_max < 108.0 * t, "x max {}", r.x_max);
}

/// Le marché couvert : la goutte cachée (colonne 67, au fond du passage) est
/// gardée par le chat de la fente (colonne 64). Fente bouchée (un toit à la
/// place du chat), la goutte n'est plus atteinte ; l'arrivée, si (le chemin
/// principal passe par-dessus le mur).
#[test]
#[ignore = "lot : le niveau exige sa mécanique (recherche complète sans elle)"]
fn niveau_3_le_chat_de_la_fente_garde_la_goutte_cachee() {
    let grille: Vec<&str> = par_cle("marche")
        .ascii
        .lines()
        .filter(|l| !l.is_empty())
        .collect();
    let fente: String = grille
        .iter()
        .enumerate()
        .map(|(r, l)| {
            if r == 8 {
                assert_eq!(&l[64..68], "z..o", "la fente et la goutte cachée");
                format!("{}#{}", &l[..64], &l[65..])
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let bouche = Niveau::depuis_ascii(&fente).unwrap();
    let r = analyser(&bouche, bouche.depart, Options::default());
    // Gouttes dans l'ordre de lecture : 0 et 1 en rangée 5 (premier auvent,
    // moment fort), 2 cachée (rangée 8).
    assert!(
        r.arrivee && r.gouttes == [true, true, false],
        "fente bouchée : {r:?}"
    );
}

/// Le marché couvert : on RESSORT de la cachette. Pépin posé sur la goutte
/// cachée (colonne 67, rangée 8), le chat de la fente déjà écrasé (on n'entre
/// pas autrement) : le drapeau reste atteignable (on remonte la fente d'un
/// saut, sur le bloc de boutique). Aucun piège sans retour au fond du passage.
#[test]
#[ignore = "lot : recherche complète du robot de faisabilité"]
fn niveau_3_on_ressort_de_la_cachette_du_chat() {
    let lignes: Vec<String> = par_cle("marche")
        .ascii
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| {
            let mut l = l.to_string();
            if l.as_bytes().get(64) == Some(&b'z') {
                l.replace_range(64..65, "."); // le chat de la fente, écrasé
            }
            l
        })
        .collect();
    let ecrase = Niveau::depuis_ascii(&lignes.join("\n")).unwrap();
    let t = pepin_core::aabb::TAILLE_TUILE;
    let cachette = (67.0 * t + 3.0, 9.0 * t - pepin_core::physics::JOUEUR_H);
    let r = analyser(&ecrase, cachette, Options::arrivee_seule());
    assert!(r.arrivee, "coincé dans la cachette : {r:?}");
}

/// Vérifie un niveau : arrivée, 3 gouttes, toutes les lanternes atteignables,
/// y compris par le robot « maladroit » (pas de précision au pixel près). Au
/// moins 3 lanternes, sauf dans l'arène du Grand Nuage (l'« arrivée » y est sa
/// tête, voir `faisabilite`) : une seule, au début de l'arène.
fn verifier(i: usize) {
    let r = verifier_infos(&niveaux()[i]);
    verifier_duree(&niveaux()[i], &r);
}

pub(crate) fn verifier_infos(infos: &InfosNiveau) -> pepin_core::faisabilite::Rapport {
    let niv = infos.charger().unwrap();
    let r = analyser(&niv, niv.depart, Options::default());
    let n = infos.numero;
    println!(
        "niveau {n} : {:.1} s en course rapide, {} états",
        r.pas_arrivee.unwrap_or(0) as f32 / 60.0,
        r.etats
    );
    assert!(
        r.arrivee,
        "niveau {n} : arrivée inatteignable (x max {} en {:?})",
        r.x_max, r.dernier_etat
    );
    assert_eq!(r.gouttes, [true; 3], "niveau {n} : gouttes {:?}", r.gouttes);
    assert!(
        r.lanternes.iter().all(|&l| l),
        "niveau {n} : lanternes {:?}",
        r.lanternes
    );
    assert_eq!(r.lanternes.len(), niv.nb_lanternes());
    if infos.genre == GenreNiveau::Bonus {
        // Le bonus (« L'arc-en-ciel », carte 5.16) : « sans ennemi ni lanterne »
        // (demande d'AGA) ; une chute ramène au départ.
        assert_eq!(
            niv.nb_lanternes(),
            0,
            "niveau {n} : le bonus n'a aucune lanterne"
        );
    } else if niv.arrivee.is_some() {
        assert!(
            niv.nb_lanternes() >= 3,
            "niveau {n} : au moins 3 checkpoints"
        );
    } else {
        // L'arène du Grand Nuage (pas de drapeau) : un seul écran, une seule
        // lanterne, au début de l'arène (décision du coordinateur : une mort
        // ramène au début de l'arène).
        // Carte 9.13a : l'arène à tour de la Pieuvre a une seconde lanterne,
        // au début de la tour (la reprise de la phase 2).
        let tour = niv.entites.iter().any(
            |e| matches!(e, pepin_core::entities::Entite::Pieuvre(p) if p.tete_finale.is_some()),
        );
        // Carte 9.13b : et une troisième à mi-hauteur de la tour ; carte
        // 9.13e : une quatrième juste après le 2e jeté (69 % de la montée) ;
        // carte 9.13f : une cinquième juste sous la vigie.
        let attendues = if tour { 5 } else { 1 };
        assert_eq!(niv.nb_lanternes(), attendues, "niveau {n} : arène du boss");
    }

    // Le robot maladroit ne sert qu'à l'arrivée : il s'arrête dès qu'il l'a
    // touchée (même réponse, carte 9.23).
    let maladroit = analyser(
        &niv,
        niv.depart,
        Options {
            precis: false,
            ..Options::arrivee_seule()
        },
    );
    assert!(
        maladroit.arrivee,
        "niveau {n} : trop exigeant (robot maladroit bloqué à x = {})",
        maladroit.x_max
    );
    r
}

/// Chaque lanterne est aussi un point de reprise : le niveau doit se
/// terminer DEPUIS elle (sinon un respawn pourrait coincer le joueur). Un test
/// par niveau (`tests_de_faisabilite!`), un pour les niveaux d'essai ; la
/// recherche s'arrête dès l'arrivée touchée (`Options::arrivee_seule`).
fn reprise_depuis_chaque_lanterne(infos: &InfosNiveau) {
    let niv = infos.charger().unwrap();
    for e in &niv.entites {
        if let pepin_core::entities::Entite::Lanterne(l) = e {
            let r = analyser(&niv, l.point_de_reapparition(), Options::arrivee_seule());
            assert!(
                r.arrivee,
                "niveau {} : bloqué depuis la lanterne en {:?}",
                infos.numero,
                l.point_de_reapparition()
            );
        }
    }
}

#[test]
#[ignore = "lot : reprise depuis chaque lanterne des niveaux d'essai"]
fn reprise_depuis_chaque_lanterne_des_niveaux_d_essai() {
    for infos in niveaux_et_essai().skip(NB_NIVEAUX) {
        reprise_depuis_chaque_lanterne(infos);
    }
}

/// Ce qu'une carte de NIVEAU vérifie de son niveau (carte 9.23, commande
/// `./docker-cargo.sh carte <clé>`) : la faisabilité (arrivée, 3 gouttes,
/// lanternes, robot maladroit, durée) et la reprise depuis chaque lanterne, les
/// deux en parallèle. Les mêmes vérifications que ses deux tests de lot
/// (`tests_de_faisabilite!`), choisies par la clé (variable `CLE`).
#[test]
#[ignore = "carte : un seul niveau, CLE=<clé> (./docker-cargo.sh carte <clé>)"]
fn le_niveau_de_la_carte() {
    let cle = std::env::var("CLE").expect("CLE=<clé du niveau>");
    let i = niveaux()
        .iter()
        .position(|n| n.cle == cle)
        .unwrap_or_else(|| panic!("aucun niveau de clé {cle:?}"));
    std::thread::scope(|portee| {
        portee.spawn(|| reprise_depuis_chaque_lanterne(&niveaux()[i]));
        verifier(i);
    });
}

/// Un niveau dure 40 à 60 s pour un joueur normal. Le robot « course
/// rapide » est ~2 fois plus rapide qu'un humain découvrant le niveau (au plus
/// 45 s, 60 s au chapitre 2 : carte 9.21b).
///
/// L'arène du Grand Nuage : le robot de faisabilité le juge toujours essoufflé
/// et atteint sa tête en une seconde ; ce qui fait la durée, c'est le combat.
/// On compte donc le combat le plus court (grondement, les trois motifs, deux
/// fois « touché », la déchirure du ciel), sans même les montées dans le
/// courant. L'arène de la Pieuvre du port (carte 9.12b) : de même, avec SES
/// durées (prélude, les trois motifs, trois remontées de la tête, deux fois
/// « touchée », la plongée).
///
/// Carte 9.23 : vérifiée dans le test de faisabilité de chaque niveau, avec SA
/// recherche (l'ancien test `durees_plausibles` refaisait la même recherche
/// complète de tous les niveaux, 6 min).
fn verifier_duree(infos: &InfosNiveau, r: &pepin_core::faisabilite::Rapport) {
    use pepin_core::entities::nuage::{
        duree_motif, DECHIRURE_PAS, PRELUDE_PAS, PV_NUAGE, TOUCHE_PAS,
    };
    use pepin_core::entities::pieuvre;
    let niv = infos.charger().unwrap();
    let a_une_pieuvre = niv
        .entites
        .iter()
        .any(|e| matches!(e, pepin_core::entities::Entite::Pieuvre(_)));
    let tour = niv
        .entites
        .iter()
        .any(|e| matches!(e, pepin_core::entities::Entite::Pieuvre(p) if p.tete_finale.is_some()));
    let pas = if niv.arrivee.is_some() {
        r.pas_arrivee.unwrap()
    } else if tour {
        // Carte 9.13a : la phase 1 (le combat le plus court, sans la plongée :
        // le 3e coup donne la furie) et la phase 2 (la furie, la course du
        // robot jusqu'à la tête du haut, la plongée) se vérifient CHACUNE
        // comme un niveau : le boss final dure deux niveaux.
        let pv = pieuvre::PV_PIEUVRE;
        let motifs: u32 = (0..pv as usize)
            .map(|m| u32::from(pieuvre::duree_motif(m)))
            .sum();
        let phase1 = u32::from(pieuvre::PRELUDE_PAS)
            + motifs
            + u32::from(pv) * u32::from(pieuvre::REMOUS_PAS)
            + u32::from(pv - 1) * u32::from(pieuvre::TOUCHEE_PAS);
        let phase2 = u32::from(pieuvre::FURIE_PAS)
            + r.pas_arrivee.unwrap()
            + u32::from(pieuvre::PLONGEE_PAS);
        println!(
            "niveau {} : phase 1 {:.1} s, phase 2 {:.1} s",
            infos.numero,
            phase1 as f32 / 60.0,
            phase2 as f32 / 60.0
        );
        assert!((12.0..=60.0).contains(&(phase1 as f32 / 60.0)));
        assert!((5.0..=60.0).contains(&(phase2 as f32 / 60.0)));
        phase1 + phase2
    } else if a_une_pieuvre {
        let pv = pieuvre::PV_PIEUVRE;
        let motifs: u32 = (0..pv as usize)
            .map(|m| u32::from(pieuvre::duree_motif(m)))
            .sum();
        u32::from(pieuvre::PRELUDE_PAS)
            + motifs
            + u32::from(pv) * u32::from(pieuvre::REMOUS_PAS)
            + u32::from(pv - 1) * u32::from(pieuvre::TOUCHEE_PAS)
            + u32::from(pieuvre::PLONGEE_PAS)
    } else {
        let motifs: u32 = (0..PV_NUAGE as usize)
            .map(|m| u32::from(duree_motif(m)))
            .sum();
        u32::from(PRELUDE_PAS)
            + motifs
            + u32::from(PV_NUAGE - 1) * u32::from(TOUCHE_PAS)
            + u32::from(DECHIRURE_PAS)
    };
    let s = pas as f32 / 60.0;
    println!(
        "niveau {} : course rapide {s:.1} s (cible {} s)",
        infos.numero, infos.duree_cible_s
    );
    // Le chapitre 2 (carte 9.21b, demande d'AGA « les autres niveaux sont trop
    // courts ») vise 50 à 70 s pour l'Habitué de la mesure : ses longues
    // traversées en barque comptent autant pour le robot que pour l'Habitué
    // (peu d'attente), d'où une course rapide permise jusqu'à 60 s.
    let max = if infos.monde >= 2 { 60.0 } else { 45.0 };
    assert!(
        (12.0..=max).contains(&s),
        "niveau {} : {s:.1} s",
        infos.numero
    );
    // Chrono cible : 40 à 60 s au Monde 1, jusqu'à 75 s au chapitre 2, aux
    // niveaux plus longs (les égouts allongés de la carte 9.21a visent 65 s).
    let cible_max = if infos.monde >= 2 { 75 } else { 60 };
    assert!((40..=cible_max).contains(&infos.duree_cible_s));
}

#[test]
fn premier_defi_visible_des_le_premier_ecran() {
    // Fun immédiat : à l'écran de départ (320 px de large) on voit déjà une
    // goutte, un ennemi, un éclair, un courant ou un trampoline.
    for infos in niveaux() {
        let niv = infos.charger().unwrap();
        let ecran = pepin_core::LARGEUR_LOGIQUE as f32;
        let proche = niv.courants.iter().any(|c| c.zone.x < ecran)
            || niv.entites.iter().any(|e| match e {
                pepin_core::entities::Entite::Goutte(g) => g.boite.x < ecran,
                pepin_core::entities::Entite::Ennemi(en) => en.boite.x < ecran,
                pepin_core::entities::Entite::Eclair(ec) => ec.zone.x < ecran,
                pepin_core::entities::Entite::Champignon(c) => c.boite.x < ecran,
                _ => false,
            });
        assert!(
            proche,
            "niveau {} : rien d'intéressant à l'écran de départ",
            infos.numero
        );
    }
}
