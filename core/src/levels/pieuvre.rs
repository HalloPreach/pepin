//! Monde 2, niveau du boss : « La Pieuvre du port » (carte 9.12b). Le boss du
//! Monde 2, « Là où va la pluie » : à l'entrée du port, la Pieuvre garde le
//! bassin. Il n'y a pas de drapeau : le niveau est fini quand la Pieuvre
//! (lettre `8`, carte 9.12a) est vaincue.
//!
//! Le combat (voir `entities::pieuvre`) :
//! 1. la pieuvre est calme (prélude), puis frappe par COLONNES de tentacules
//!    (remous et ombre, annonce, puis le bout qui se dresse, mortel, puis
//!    redescend) : on quitte la colonne annoncée en courant sur le platelage
//!    (carte 9.12c), en sautant ses trous d'une case ;
//! 2. après chaque série, elle sort la TÊTE sur l'eau (au milieu, puis à
//!    gauche, puis à droite) : on tombe dans le trou qui la borde, sur elle (un
//!    point) ;
//! 3. trois motifs de plus en plus rapides, trois coups, elle replonge et
//!    lâche la clé : le niveau est fini.
//!
//! Une mort ramène à la lanterne (au début de l'arène) ; la pieuvre
//! reprend son calme mais GARDE les points perdus.
//!
//! PHASE 2 (carte 9.13a, demande d'AGA : « J'aime pas la pieuvre, elle manque
//! toujours de quelque chose pour un boss final… Une phase deux ? », il a choisi
//! « fuite et montée ») : au 3e coup, elle entre en furie, la bâche du plafond se
//! déchire, une trombe monte au-dessus du trou du milieu et le quai casse. La mer
//! monte (elle tue, barque comprise) ; on grimpe la tour du port en fuyant les
//! tentacules qui jaillissent de dessous ; en haut, sur la vigie, un saut sur la
//! tête l'achève. Une mort en phase 2 ramène à la lanterne du bas de la tour, au
//! début de la phase 2 (voir `entities::pieuvre::fuite`).
//!
//! LA MONTÉE (carte 9.13b, AGA : « la montée est trop simple et trop courte, il
//! faudrait des interactions avec la pieuvre ») : une tour deux fois plus haute,
//! deux RENCONTRES avec la tête (au tiers et aux deux tiers : elle bloque le
//! passage jusqu'à ce qu'on lui saute dessus, la mer recule alors de 2 tuiles),
//! deux BALAYEURS qui glissent sur une corniche, deux plateformes ARRACHÉES sous
//! le nez de Pépin, et une lanterne de mi-hauteur.
//!
//! LA GÉANTE (carte 9.13c, AGA : « je n'aime pas la montée, les attaques n'ont
//! pas de sens », il a choisi « la Pieuvre géante et visible qui grimpe à côté
//! de la tour ») : la tour n'a plus que 13 colonnes ; dans le couloir de 5
//! colonnes à sa droite, la géante monte avec la mer, son œil suit Pépin, et
//! chaque attaque part de son corps : frappe visée, balayage, secousse (des
//! caisses tombent), arrachage, et deux JETÉS où, sonnée, elle se laisse frapper
//! (la mer recule). Plus de trappe ni de gardes (voir
//! `entities::pieuvre::montee`).
//!
//! L'arène reprend la géométrie du niveau d'essai « pieuvre » (`essai_pieuvre`),
//! sur laquelle le robot de QA a appris à battre la pieuvre : mêmes murs, même
//! bassin, même place de la pieuvre, des pontons, des gouttes et de la
//! lanterne. Seules changent les INFOS (un vrai niveau du Monde 2, pas un
//! essai hors table) et, au-dessus de l'arène, la tour (carte 9.13a).
//!
//! Le rendu (carte 9.20e, `game/src/dessin_port.rs`) en fait le BOUT D'UNE JETÉE
//! au milieu du port : les murs `B` des bords se dessinent en quais (grue à
//! gauche, pile de caisses à droite), le port au fond (mer, phare rayé, bateaux),
//! le bassin en eau profonde. La grille n'a pas changé.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    monde: 2,
    // Sa place dans le Monde 2 de SA branche (recette de l'étape 9) : après le
    // caniveau (numéro 1), donc 2 ici. Sa place finale est la 7e (avant la marée) :
    // à la fusion, le coordinateur range la ligne et corrige les numéros
    // (`verifier_ordre` refuse un 7 tant que les niveaux 2 à 6 manquent).
    numero: 7,
    cle: "pieuvre",
    sans_planer: false,
    nom: "La Pieuvre du port",
    theme: Theme::Port,
    genre: GenreNiveau::Normal,
    intro: "La Pieuvre garde le port. Elle ne te laissera pas passer.",
    mecanique: "La Pieuvre du port (boss) : quitter la colonne du tentacule annoncé, sauter sur \
                la tête sortie trois fois (jamais la toucher de côté) ; puis fuir la mer qui \
                monte en haut de la tour, et l'achever là-haut.",
    // Carte 9.13a : le boss final dure deux niveaux (la phase 1 et la montée).
    duree_cible_s: 75,
    ascii: ASCII,
    aiguilles: &[],
    poissons_hauts: &[],
};

/// 20 x 80 tuiles (7,5 écrans de haut, caméra verticale), thème Port.
///
/// En BAS (les 11 dernières rangées), l'arène de la phase 1, qui reprend la
/// géométrie du niveau d'essai « pieuvre » : bassin de toute l'arène, la pieuvre
/// `8` (ses 9 colonnes de tentacules de 2 cases couvrent tout le bassin, aucun
/// coin à l'abri), le platelage percé d'un trou au bord de chaque tête (carte
/// 9.12c ; carte 9.12e : plus de petit ponton au-dessus de la tête du milieu), les
/// trois gouttes sous le départ (prises en tombant) et la lanterne.
///
/// Au-dessus, la phase 2 : la BÂCHE `_` (le plafond de l'arène, carte 9.13a),
/// puis la TOUR du port (66 rangées, carte 9.13b), resserrée par la carte 9.13c
/// sur les 13 colonnes de gauche : les 5 colonnes de droite sont le COULOIR de
/// la géante, toujours vide. Carte 9.13e (AGA : « la tour est monotone, c'est
/// le même zigzag environ huit fois ») : des SECTIONS différentes, aucune plus
/// de deux fois. Du bas vers le haut :
/// - la corniche du bas (rangée 66) et sa lanterne, la reprise de la phase 2 ;
/// - PILE DE CONTENEURS en escalier : un ponton (64), deux piles de caisses de
///   deux étages (61, 55) et, entre elles, un NUAGE FRIABLE `F` (58) ;
/// - QUAI DE PIERRE : JETÉ 1 (`$`) sur la corniche large 52 (des pierres `B`),
///   puis la LANTERNE de 29 % (49) sur son môle ;
/// - MÂT ET VERGUE : deux caisses (46) dont une ARRACHABLE `%` à droite (l'autre
///   chemin passe à gauche), la vergue (43) sur son mât d'une case ;
/// - la corniche large 40, que BALAIE un tentacule (`!`, contre le couloir) ;
/// - CHEMINÉE DE CARGO : SECOUSSE 1 (`+`) en arrivant sur la caisse 37, puis le
///   haut d'une cheminée de pierre (34), un NUAGE FRIABLE (31) ;
/// - JETÉ 2 sur la corniche large 28, puis la LANTERNE de 69 % (carte 9.13e :
///   « j'en ajouterais une juste après le deuxième jeté ») sur la corniche
///   large et CALME 25 (aucune attaque n'y est liée), la corniche large 22
///   BALAYÉE ;
/// - SECOUSSE 2 en arrivant sur la caisse 19, deux caisses (16) dont une
///   ARRACHABLE à droite, un NUAGE FRIABLE (13), une caisse (10), puis la
///   VIGIE (7), où elle se jette pour le coup final (`&`).
///
/// Six corniches larges (52, 40, 28, 25, 22 et la vigie) : un palier de la mer
/// sous chacune, et la place d'un jeté.
///
/// Chaque saut monte de 2 ou 3 rangées et passe par une colonne libre à côté de
/// l'appui suivant (le robot de faisabilité le vérifie, avec et sans les
/// plateformes arrachables).
const ASCII: &str = r"
B..................B
B..................B
B..................B
B..................B
B..................B
B..................B
B.........&........B
B....#########.....B
B..................B
B..................B
B###...............B
B..................B
B..................B
B...FFF............B
B..................B
B..................B
B##....%%%.........B
B..................B
B.+................B
B####..............B
B..................B
B............!.....B
B.....########.....B
B..................B
B.L................B
B########..........B
B..................B
B.........$........B
B.....########.....B
B..................B
B..................B
B.FFF..............B
B..................B
B..................B
B.....BBB..........B
B.....BBB..........B
B.+................B
B####..............B
B..................B
B............!.....B
B.....########.....B
B..................B
B..................B
B..###.............B
B...#..............B
B...#..............B
B##...%%%..........B
B..................B
B.L................B
BBBBB..............B
B..................B
B.........$........B
B.....BBBBBBBB.....B
B..................B
B..................B
B..###.............B
B..###.............B
B..................B
BFF................B
B..................B
B..................B
B....###...........B
B....###...........B
B..................B
B.........###......B
B.L................B
B#######....##.....B
B..................B
B__________________B
B@.................B
Bo.................B
Bo.................B
Bo.................B
B..................B
B.L................B
B####.##....##.####B
B#....##....##....#B
B#....##....##....#B
B~~~~~~~~8~~~~~~~~~B
BBBBBBBBBBBBBBBBBBBB
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::{Aabb, TAILLE_TUILE};
    use crate::entities::contacts::{resoudre_contacts, Contact};
    use crate::entities::pieuvre::{
        EtatArrachage, EtatBalai, EtatFrappe, EtatJete, EtatSecousse, PhasePieuvre, Pieuvre,
        ANNONCE_BALAI_PAS, ANNONCE_FRAPPE_PAS, ANNONCE_SAISIE_PAS, ANNONCE_SECOUSSE_PAS,
        BOUT_BALAI, CABRE_PAS, CHUTE_JETE_PAS, FURIE_PAS, PLONGEE_PAS, RECUL_MER, REPIT_PAS,
        RYTHME_FRAPPE_PAS, SOUS_PEPIN, TAILLE_GEANTE, VISEE_MAX, VITESSE_BALAI,
    };
    use crate::entities::{Actualisable, Entite};
    use crate::level::Niveau;
    use crate::levels::essai_pieuvre;
    use crate::physics::{Environnement, JOUEUR_H, JOUEUR_L};
    use crate::player::Joueur;

    /// Première rangée de l'arène : la tour et la bâche sont au-dessus.
    const HAUT_ARENE: i32 = 69;

    /// L'arène du boss : pas de drapeau, une seule pieuvre, quatre lanternes (au
    /// départ, au début de la tour : carte 9.13a, à 29 % : carte 9.13b, à 69 % :
    /// carte 9.13e),
    /// trois gouttes ; et, dans ses
    /// 11 dernières rangées, EXACTEMENT la géométrie du niveau d'essai « pieuvre »
    /// (mêmes cases vides ou pleines, même pieuvre, même bassin, mêmes pontons,
    /// mêmes gouttes, même départ), sur laquelle le combat a été réglé et le robot
    /// de QA entraîné.
    #[test]
    fn l_arene_de_la_pieuvre_reprend_celle_du_niveau_d_essai() {
        let n = INFOS.charger().unwrap();
        let e = essai_pieuvre::INFOS.charger().unwrap();
        let dy = HAUT_ARENE as f32 * TAILLE_TUILE;
        assert_eq!((n.largeur(), n.hauteur()), (20, 80));
        assert!(n.arrivee.is_none(), "l'arène n'a pas de drapeau");
        assert_eq!(n.nb_lanternes(), 4);
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
        // Carte 9.13a : la tour a sa tête finale et sa bâche ; carte 9.13c : deux
        // jetés, deux balayages, deux secousses, deux plateformes arrachables, et
        // le couloir de la géante (5 colonnes contre le mur droit), toujours
        // vide ; carte 9.13e : cinq paliers (sous ses cinq corniches larges sous
        // la vigie).
        assert!(p.tete_finale.is_some() && p.bache.is_some());
        assert_eq!(p.paliers.len(), 5, "{:?}", p.paliers);
        assert_eq!(p.tour.jetes.len(), 2);
        assert_eq!(p.tour.balayeurs.len(), 2);
        assert_eq!(p.tour.secousses.len(), 2);
        assert_eq!(p.tour.arrachables.len(), 2);
        assert_eq!(
            p.tour.couloir,
            Some((14.0 * TAILLE_TUILE, 19.0 * TAILLE_TUILE))
        );
        for ty in 0..HAUT_ARENE - 1 {
            for tx in 14..19 {
                assert!(!n.tuile_decor(tx, ty).est_solide(), "couloir ({tx}, {ty})");
            }
        }
        assert!(
            q.tete_finale.is_none(),
            "le niveau d'essai reste en une phase"
        );
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
        let n = INFOS.charger().unwrap();
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
        let e = essai_pieuvre::INFOS.charger().unwrap();
        let mut q = pieuvre(&e);
        trois_coups(&mut q);
        assert_eq!(q.phase(), PhasePieuvre::Plongee);
    }

    /// La mer monte (vite dans l'arène, puis d'une tuile toutes les 1,3 s, avec
    /// ses paliers), jusque sous la vigie ; elle TUE au contact, barque comprise
    /// (des pieds 3 px sous la surface) ; en mode facile, elle monte moins vite.
    #[test]
    fn la_mer_monte_et_tue() {
        let n = INFOS.charger().unwrap();
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
        // Une tuile en 1,3 s (78 pas) hors palier.
        let une_tuile = p.surface_mer_a(crue) - p.surface_mer_a(crue + 78);
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
        let lent = f.surface_mer_a(crue) - f.surface_mer_a(crue + 78);
        assert!((lent - 0.75 * TAILLE_TUILE).abs() < 0.5, "{lent}");
    }

    /// Une mort pendant la montée ramène au début de la phase 2 : la lanterne de la
    /// tour s'est allumée d'elle-même (le point de reprise est au-dessus de la
    /// bâche), et la mer est revenue en bas.
    #[test]
    fn reprise_au_debut_de_la_phase_2() {
        let n = INFOS.charger().unwrap();
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
        crate::entities::reprendre_apres_une_mort(&mut entites, &j.boite);
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

    /// Pépin debout à `x` (milieu), les pieds en `bas`.
    fn pepin_debout(x: f32, bas: f32) -> Aabb {
        Aabb::new(x - JOUEUR_L / 2.0, bas - JOUEUR_H, JOUEUR_L, JOUEUR_H)
    }

    /// Un saut qui retombe sur `tete`.
    fn saut_sur(tete: Aabb) -> Joueur {
        let mut j = Joueur::new(tete.x + 19.0, tete.y - 12.0);
        j.vitesse.y = 100.0;
        j
    }

    /// Avance pas à pas (au plus `max`) jusqu'à ce que `fini` soit vrai.
    fn jusqu_a(p: &mut Pieuvre, n: &Niveau, max: u32, fini: impl Fn(&Pieuvre) -> bool) {
        for _ in 0..max {
            if fini(p) {
                return;
            }
            pas(p, n, 1);
        }
        assert!(fini(p), "jamais en {max} pas");
    }

    /// Carte 9.13c : la géante monte AVEC la mer dans son couloir ; son corps
    /// tue (pas la tour à côté), dès que la mer est dans la tour.
    #[test]
    fn la_geante_monte_avec_la_mer() {
        let n = INFOS.charger().unwrap();
        let p = en_montee(&n, 1);
        assert!(p.corps().is_none(), "pendant la crue, elle sort du bassin");
        let (g, d) = p.tour.couloir.unwrap();
        for t in [p.fin_de_crue() + 5, 900, 1500] {
            let mut q = en_montee(&n, 0);
            pas(&mut q, &n, u32::from(t));
            let mer = q.surface_mer().unwrap();
            let (corps, tete) = (q.corps().unwrap(), q.geante().unwrap());
            assert!(corps.x >= g && corps.droite() <= d, "dans le couloir");
            assert!((tete.bas() - mer).abs() <= 1.0 && tete.h == TAILLE_GEANTE.1);
            assert!(
                q.blesse(&pepin_debout(g + 30.0, mer - 20.0)),
                "son corps tue"
            );
            assert!(
                !q.blesse(&pepin_debout(g - 20.0, mer - 20.0)),
                "la tour, non"
            );
        }
        // Pépin loin devant la mer : elle grimpe hors de l'eau, à 4 tuiles sous
        // ses pieds (toujours à l'écran), et tout le couloir sous sa tête tue.
        let mut q = en_montee(&n, 0);
        let pieds = 31.0 * TAILLE_TUILE;
        q.voir_pepin(pepin_debout(3.0 * TAILLE_TUILE, pieds));
        let crue = u32::from(q.fin_de_crue());
        pas(&mut q, &n, crue + 400);
        let tete = q.geante().unwrap();
        assert!(q.grimpe(), "{tete:?}");
        assert!((tete.bas() - (pieds + SOUS_PEPIN)).abs() <= 1.0, "{tete:?}");
        let mer = q.surface_mer().unwrap();
        assert!(q.blesse(&pepin_debout(g + 30.0, (tete.bas() + mer) / 2.0)));
    }

    /// Chaque frappe visée laisse une issue : pour CHAQUE place de Pépin (au px
    /// près) sur chaque appui de la tour, la place visée est à moins de 3 tuiles
    /// de lui (le plus souvent sur lui), et sur chaque appui qu'elle touche il
    /// reste une place épargnée, que Pépin rejoint en courant pendant l'annonce
    /// (1,6 px par pas). En jeu : annoncée 1,4 s sans danger, puis elle frappe
    /// la place visée, toutes les 2,5 s.
    #[test]
    fn chaque_frappe_visee_laisse_une_issue() {
        let n = INFOS.charger().unwrap();
        let p = en_montee(&n, 0);
        let appuis = p.appuis(&n);
        let (mut places, mut sur_lui) = (0, 0);
        for &(y, a, b) in &appuis {
            for q in (a as i32..=(b - JOUEUR_L) as i32).map(|q| q as f32) {
                let pepin = Aabb::new(q, y - JOUEUR_H, JOUEUR_L, JOUEUR_H);
                let cible = p.viser(&n, &pepin);
                places += 1;
                sur_lui += usize::from(cible.chevauche(&pepin));
                let milieu = |c: &Aabb| c.x + c.w / 2.0;
                assert!((milieu(&cible) - milieu(&pepin)).abs() <= VISEE_MAX + 0.01);
                for &(y2, a2, b2) in &appuis {
                    let debout = |r: f32| Aabb::new(r, y2 - JOUEUR_H, JOUEUR_L, JOUEUR_H);
                    let touchees: Vec<f32> = (a2 as i32..=(b2 - JOUEUR_L) as i32)
                        .map(|r| r as f32)
                        .collect();
                    for &r in touchees.iter().filter(|&&r| cible.chevauche(&debout(r))) {
                        let issue = touchees.iter().any(|&s| {
                            !cible.chevauche(&debout(s))
                                && (s - r).abs() <= 1.6 * f32::from(ANNONCE_FRAPPE_PAS)
                        });
                        assert!(
                            issue,
                            "Pépin en {q} sur {:?} : appui {:?}",
                            (y, a, b),
                            (y2, a2, b2)
                        );
                    }
                }
            }
        }
        assert!(
            sur_lui * 10 >= places * 8,
            "{sur_lui} visées sur lui pour {places}"
        );
        // En jeu, sur la caisse de la rangée 64.
        let mut p = en_montee(&n, 0);
        let pepin = pepin_debout(11.0 * TAILLE_TUILE, 64.0 * TAILLE_TUILE);
        p.voir_pepin(pepin);
        jusqu_a(&mut p, &n, 900, |p| p.etat_frappe().is_some());
        let (cible, _) = p.etat_frappe().unwrap();
        assert_eq!(cible, p.viser(&n, &pepin));
        let debut = p.ecoule();
        for _ in 1..ANNONCE_FRAPPE_PAS {
            assert!(matches!(p.etat_frappe(), Some((_, EtatFrappe::Annonce(_)))));
            assert!(
                p.annonce_en_cours() && !p.blesse(&pepin),
                "l'annonce ne tue pas"
            );
            pas(&mut p, &n, 1);
        }
        pas(&mut p, &n, 1);
        assert_eq!(p.etat_frappe(), Some((cible, EtatFrappe::Frappe)));
        assert!(p.blesse(&pepin), "elle frappe la place visée");
        jusqu_a(&mut p, &n, 900, |p| {
            matches!(p.etat_frappe(), Some((_, EtatFrappe::Annonce(_))))
        });
        // Carte 9.13d : au moins sa recharge (le cerveau tire au sort ensuite).
        assert!(p.ecoule() - debut >= RYTHME_FRAPPE_PAS);
    }

    /// Les balayages et les caisses sont annoncés. Un balayage : 1 s sans danger
    /// (le tentacule recule et se tend), puis il glisse sur toute sa corniche, de
    /// droite à gauche, au ras du sol ; un saut le passe. Une secousse : 1 s sans
    /// danger (l'ombre des caisses), puis les caisses tombent ; l'une sur Pépin,
    /// jamais sur les places à une colonne de lui.
    #[test]
    fn balayages_et_caisses_sont_annonces() {
        use crate::physics::{GRAVITE, VITESSE_SAUT};
        let n = INFOS.charger().unwrap();
        let p0 = en_montee(&n, 0);
        let t = TAILLE_TUILE;
        for (k, b) in p0.tour.balayeurs.iter().enumerate() {
            assert!(
                (b.depart - p0.tour.couloir.unwrap().0).abs() < 0.01,
                "du couloir"
            );
            let mut p = p0.clone();
            let x = b.arrivee + 2.0 * t;
            p.voir_pepin(pepin_debout(x, b.y));
            jusqu_a(&mut p, &n, 1500, |p| p.etat_balayeur(k).is_some());
            for _ in 1..ANNONCE_BALAI_PAS {
                assert!(matches!(p.etat_balayeur(k), Some(EtatBalai::Annonce(_))));
                assert!(!p.blesse(&pepin_debout(x, b.y)), "l'annonce ne tue pas");
                pas(&mut p, &n, 1);
            }
            let mut touche = false;
            let mut avant = f32::MAX;
            for _ in 0..b.course_pas() {
                pas(&mut p, &n, 1);
                let Some(EtatBalai::Balaie(bout)) = p.etat_balayeur(k) else {
                    panic!("balayage {k} : il glisse");
                };
                assert!(bout.bas() <= b.y + 0.01 && bout.y >= b.y - 12.01);
                assert!(bout.x <= avant, "de droite à gauche");
                avant = bout.x;
                touche |= p.blesse(&pepin_debout(x, b.y));
            }
            assert!(touche, "balayage {k} : il balaie toute la corniche");
        }
        // Le saut : les pieds au-dessus du bout (12 px) pendant plus longtemps
        // que le bout ne met à passer sous Pépin.
        let (v, g) = (VITESSE_SAUT / 60.0, GRAVITE / 3600.0);
        let (mut y, mut vy, mut au_dessus) = (0.0, v, 0);
        while y >= 0.0 {
            y += vy;
            vy -= g;
            if y > BOUT_BALAI.1 {
                au_dessus += 1;
            }
        }
        let passage = ((BOUT_BALAI.0 + JOUEUR_L) / VITESSE_BALAI).ceil() as i32;
        assert!(au_dessus > 2 * passage, "{au_dessus} pas contre {passage}");
        // La secousse 1 : Pépin arrive sur la caisse de la rangée 37.
        let mut p = p0.clone();
        let sol = 37.0 * t;
        let x = 2.5 * t;
        p.voir_pepin(pepin_debout(x, sol));
        jusqu_a(&mut p, &n, 1500, |p| p.etat_secousse().is_some());
        let caisses = p.etat_secousse().unwrap().0.to_vec();
        assert!((2..=3).contains(&caisses.len()), "{caisses:?}");
        for _ in 1..ANNONCE_SECOUSSE_PAS {
            assert!(matches!(
                p.etat_secousse(),
                Some((_, EtatSecousse::Annonce(_)))
            ));
            assert!(p.annonce_en_cours() && !p.blesse(&pepin_debout(x, sol)));
            pas(&mut p, &n, 1);
        }
        let (mut sur_lui, mut a_cote) = (false, false);
        while p.etat_secousse().is_some() {
            pas(&mut p, &n, 1);
            sur_lui |= p.blesse(&pepin_debout(x, sol));
            a_cote |= p.blesse(&pepin_debout(x + t, sol)) || p.blesse(&pepin_debout(x - t, sol));
        }
        assert!(sur_lui && !a_cote, "sur lui : {sur_lui}, à côté : {a_cote}");
    }

    /// Les arrachages : annoncés 1,3 s (elle porte encore), puis emportés pour de
    /// bon ; et après CHAQUE arrachage (et tous à la fois), le robot de
    /// faisabilité trouve encore un chemin jusqu'à la vigie.
    #[test]
    fn apres_chaque_arrachage_un_chemin_existe() {
        use crate::faisabilite::{analyser, Options};
        let n = INFOS.charger().unwrap();
        let mut p = en_montee(&n, 300);
        assert!((1..=2).contains(&p.tour.arrachables.len()));
        let cases = p.tour.arrachables[0].clone();
        let (x, y) = cases[0];
        let dessous = pepin_debout(x as f32 * TAILLE_TUILE, (y + 3) as f32 * TAILLE_TUILE);
        p.voir_pepin(dessous);
        // Carte 9.13d : quand son cerveau la choisit.
        jusqu_a(&mut p, &n, 900, |p| p.etat_arrachage(0).is_some());
        assert!(matches!(
            p.etat_arrachage(0),
            Some(EtatArrachage::Annonce(_))
        ));
        assert!(p.annonce_en_cours());
        assert!(
            !p.cases_englouties().contains(&cases[0]),
            "elle porte encore"
        );
        pas(&mut p, &n, u32::from(ANNONCE_SAISIE_PAS));
        assert!(p.cases_englouties().contains(&cases[0]), "emportée");
        pas(&mut p, &n, 600);
        assert_eq!(p.etat_arrachage(0), Some(EtatArrachage::Partie));
        assert!(
            p.cases_englouties().contains(&cases[0]),
            "elle ne revient pas"
        );
        // L'autre chemin : la grille sans la plateforme arrachée.
        let lignes: Vec<&str> = ASCII.lines().skip(1).collect();
        let sans = |garde: &dyn Fn(usize, usize) -> bool| {
            let texte: String = lignes
                .iter()
                .enumerate()
                .map(|(ty, l)| {
                    let l: String = l
                        .chars()
                        .enumerate()
                        .map(|(tx, c)| if c == '%' && !garde(tx, ty) { '.' } else { c })
                        .collect();
                    l + "\n"
                })
                .collect();
            crate::level::Niveau::depuis_ascii(&texte).unwrap()
        };
        let mut grilles = vec![sans(&|_, _| false)];
        for cases in &p.tour.arrachables {
            let cases = cases.clone();
            grilles.push(sans(&move |tx, ty| {
                !cases.contains(&(tx as i32, ty as i32))
            }));
        }
        let depart = p.bache.unwrap();
        for (k, g) in grilles.iter().enumerate() {
            let r = analyser(
                g,
                (depart.x + 40.0, depart.y - 40.0),
                Options::arrivee_seule(),
            );
            assert!(r.arrivee, "grille {k} : plus de chemin");
        }
    }

    /// Le jeté `k` gagné comme en jeu : Pépin arrive sur sa corniche (loin du
    /// bout), elle se cabre, se jette, le rate, reste sonnée, et il lui saute
    /// dessus. Vérifie au passage : l'annonce sans danger, la mer arrêtée, la
    /// chute mortelle au bout de la corniche, la mer qui RECULE de 3 tuiles et
    /// 3 s sans attaque.
    fn jeter_et_frapper(p: &mut Pieuvre, n: &Niveau, k: usize) {
        let tete = p.tour.jetes[k];
        let pepin = pepin_debout(tete.x - 3.0 * TAILLE_TUILE, tete.bas());
        p.voir_pepin(pepin);
        jusqu_a(p, n, 900, |p| p.etat_jete().is_some());
        let mer = p.surface_mer().unwrap();
        for _ in 1..CABRE_PAS {
            assert!(matches!(p.etat_jete(), Some((_, EtatJete::Cabre(_)))));
            assert!(p.annonce_en_cours() && !p.blesse(&pepin), "elle se cabre");
            assert!(!p.recoit(&saut_sur(tete)), "pas frappable cabrée");
            pas(p, n, 1);
        }
        pas(p, n, 1);
        assert!(matches!(p.etat_jete(), Some((_, EtatJete::Chute(_)))));
        assert!(
            p.blesse(&pepin_debout(tete.x + 30.0, tete.bas())),
            "au bout : mortelle"
        );
        assert!(!p.blesse(&pepin), "elle le rate");
        pas(p, n, u32::from(CHUTE_JETE_PAS));
        assert!(matches!(p.etat_jete(), Some((_, EtatJete::Sonnee(_)))));
        assert_eq!(p.geante(), Some(tete), "la tête posée sur la corniche");
        assert_eq!(p.surface_mer().unwrap(), mer, "la mer s'arrête");
        let pv = p.pv();
        assert!(p.recoit(&saut_sur(tete)));
        p.toucher();
        assert_eq!(p.pv(), pv - 1);
        assert!(p.etat_jete().is_none() && p.tete_lachee().is_some());
        // Le plus bas où elle redescend (puis elle remonte) ; aucune attaque.
        let mut plus_bas = mer;
        for _ in 0..REPIT_PAS - 1 {
            pas(p, n, 1);
            plus_bas = plus_bas.max(p.surface_mer().unwrap());
            assert!(p.attaque().is_none(), "3 s sans attaque");
        }
        let recul = plus_bas - mer;
        assert!(
            (recul - RECUL_MER).abs() < 2.0,
            "jeté {k} : la mer recule ({recul})"
        );
        assert!(p.surface_mer().unwrap() < plus_bas, "puis elle remonte");
    }

    /// Carte 9.13c : à chaque jeté, sonnée, un saut sur sa tête la fait retomber
    /// et la mer recule (un point de vie chacun, puis le coup final). Si on ne la
    /// frappe pas, elle repart et la mer reste haute.
    #[test]
    fn jete_sonnee_coup_sur_la_tete_la_mer_recule() {
        let n = INFOS.charger().unwrap();
        let mut p = en_montee(&n, 0);
        assert_eq!(p.pv(), 3);
        let mut q = p.clone();
        // Carte 9.13d : deux coups pendant la montée (la 3e tête est pour la vigie).
        for k in 0..2 {
            jeter_et_frapper(&mut p, &n, k);
        }
        assert_eq!((p.pv(), p.jetes_frappes()), (1, 2));
        // Ratée et pas frappée : elle repart, le jeté est passé.
        let tete = q.tour.jetes[0];
        q.voir_pepin(pepin_debout(tete.x - 3.0 * TAILLE_TUILE, tete.bas()));
        jusqu_a(&mut q, &n, 900, |q| q.etat_jete().is_some());
        let mer = q.surface_mer().unwrap();
        jusqu_a(&mut q, &n, 900, |q| q.etat_jete().is_none());
        assert_eq!((q.jetes_faits(), q.jetes_frappes(), q.pv()), (1, 0, 3));
        assert!(q.surface_mer().unwrap() <= mer, "la mer reste haute");
    }

    /// Le coup final : sur la vigie, elle se jette DEUX fois (carte 9.13d), et
    /// recommence si on la rate ; sonnée, un saut sur sa tête lui prend une
    /// tête ; à la dernière (ici les 3, aucune prise pendant la montée), elle
    /// plonge, puis elle est vaincue : le niveau est fini.
    #[test]
    fn le_dernier_coup_finit_le_niveau() {
        let n = INFOS.charger().unwrap();
        let mut p = en_montee(&n, 300);
        let tete = p.tete_finale.unwrap();
        p.voir_pepin(pepin_debout(tete.x - 3.0 * TAILLE_TUILE, tete.bas()));
        jusqu_a(&mut p, &n, 400, |p| p.etat_jete().is_some());
        assert_eq!(p.etat_jete().unwrap().0, tete);
        // Ratée : elle repart, puis se jette de nouveau.
        let mut q = p.clone();
        jusqu_a(&mut q, &n, 900, |q| q.etat_jete().is_none());
        jusqu_a(&mut q, &n, 900, |q| q.etat_jete().is_some());
        for reste in (1..=3).rev() {
            jusqu_a(&mut p, &n, 900, |p| {
                matches!(p.etat_jete(), Some((_, EtatJete::Sonnee(_))))
            });
            assert_eq!(p.tete(), Some(tete));
            assert!(p.recoit(&saut_sur(tete)));
            p.toucher();
            let phase = if reste > 1 {
                PhasePieuvre::Montee
            } else {
                PhasePieuvre::Plongee
            };
            assert_eq!(p.phase(), phase, "{reste} têtes");
        }
        pas(&mut p, &n, u32::from(PLONGEE_PAS));
        assert!(p.est_vaincue());
        assert!(Entite::Pieuvre(Box::new(p)).boss_vaincu());
        // Jamais d'attaque sans la montée.
        let q = pieuvre(&n);
        assert!(q.attaque().is_none() && q.geante().is_none());
    }

    /// Une mort après la lanterne de mi-hauteur y ramène ; le 1er jeté reste
    /// fait (et frappé), le 2e repart à neuf, la mer revient sous la lanterne
    /// (pas tout en bas).
    #[test]
    fn reprise_a_la_lanterne_de_mi_hauteur() {
        let n = INFOS.charger().unwrap();
        let mut p = en_montee(&n, 0);
        jeter_et_frapper(&mut p, &n, 0);
        let bache = p.bache.unwrap();
        // Les lanternes de la tour, du haut vers le bas : 69 %, 29 %, le bas.
        let mut de_la_tour: Vec<(f32, f32)> = n
            .entites
            .iter()
            .filter_map(|e| match e {
                Entite::Lanterne(l) if l.boite.y < bache.y => Some(l.point_de_reapparition()),
                _ => None,
            })
            .collect();
        de_la_tour.sort_by(|a, b| a.1.total_cmp(&b.1));
        let (haut, mi) = (de_la_tour[0], de_la_tour[1]);
        let jetes = p.tour.jetes.clone();
        assert!(
            mi.1 < jetes[0].y && mi.1 > jetes[1].y,
            "entre les deux jetés"
        );
        // Carte 9.13e : la 2e lanterne, juste après le 2e jeté, vers 65 % de la
        // montée (de la lanterne du bas à la vigie).
        assert!(haut.1 < jetes[1].y, "après le 2e jeté");
        let (bas, sommet) = (de_la_tour[2].1, p.tete_finale.unwrap().bas());
        let part = (bas - haut.1) / (bas - sommet);
        assert!((0.6..0.75).contains(&part), "{part}");
        let mut entites = n.entites.clone();
        // Carte 9.13d : Pépin est passé par la lanterne (elle garde l'état de
        // ce moment-là).
        p.voir_pepin(Joueur::new(mi.0, mi.1).boite);
        pas(&mut p, &n, 1);
        for e in &mut entites {
            if let Entite::Pieuvre(q) = e {
                **q = p.clone();
            }
        }
        let j = Joueur::new(mi.0, mi.1);
        crate::entities::reprendre_apres_une_mort(&mut entites, &j.boite);
        let q = pieuvre_de(&entites);
        assert_eq!((q.jetes_faits(), q.jetes_frappes(), q.pv()), (1, 1, 2));
        let mer = q.surface_mer().unwrap();
        assert!(mer > j.boite.bas() && mer <= j.boite.bas() + 3.0 * TAILLE_TUILE + 1.0);
        // Depuis la lanterne du bas : tout repart à neuf.
        let mut entites2 = entites.clone();
        let bas = Joueur::new(40.0, bache.y - 40.0);
        crate::entities::reprendre_apres_une_mort(&mut entites2, &bas.boite);
        let q = pieuvre_de(&entites2);
        assert_eq!((q.jetes_faits(), q.horloge_mer(), q.pv()), (0, 0, 3));
        // La lanterne de la tour s'allume d'office une seule fois : celle de
        // mi-hauteur, allumée à la main, ne s'éteint pas au pas suivant.
        let mut j = Joueur::new(mi.0, mi.1);
        resoudre_contacts(&mut j, &mut entites);
        resoudre_contacts(&mut j, &mut entites);
        let allumee = entites.iter().find_map(|e| match e {
            Entite::Lanterne(l) if l.est_active() => Some(l.point_de_reapparition()),
            _ => None,
        });
        assert_eq!(allumee, Some(mi));
    }

    /// Carte 9.13e : une mort après la lanterne de 69 % (juste après le 2e jeté)
    /// y ramène : les deux jetés restent faits et frappés, la mer revient SOUS la
    /// lanterne (3 tuiles au plus sous ses pieds), et ne la noie pas d'emblée :
    /// la seconde suivante, elle est encore sous ses pieds.
    #[test]
    fn reprise_a_la_lanterne_de_69_pour_cent() {
        let n = INFOS.charger().unwrap();
        let mut p = en_montee(&n, 0);
        jeter_et_frapper(&mut p, &n, 0);
        jeter_et_frapper(&mut p, &n, 1);
        let bache = p.bache.unwrap();
        let haut = n
            .entites
            .iter()
            .filter_map(|e| match e {
                Entite::Lanterne(l) if l.boite.y < bache.y => Some(l.point_de_reapparition()),
                _ => None,
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap();
        // Carte 9.13d : la reprise rend l'état de la Pieuvre « au passage de la
        // lanterne », noté quand elle VOIT Pépin à cette hauteur (`observer`).
        // Il faut donc qu'elle l'y ait vu au moins une fois avant la mort.
        let j = Joueur::new(haut.0, haut.1);
        p.voir_pepin(j.boite);
        pas(&mut p, &n, 1);
        let mut entites = n.entites.clone();
        for e in &mut entites {
            if let Entite::Pieuvre(q) = e {
                **q = p.clone();
            }
        }
        crate::entities::reprendre_apres_une_mort(&mut entites, &j.boite);
        let mut q = pieuvre_de(&entites).clone();
        assert_eq!((q.jetes_faits(), q.jetes_frappes(), q.pv()), (2, 2, 1));
        let mer = q.surface_mer().unwrap();
        assert!(
            mer > j.boite.bas() && mer <= j.boite.bas() + 3.0 * TAILLE_TUILE + 1.0,
            "mer {mer}, pieds {}",
            j.boite.bas()
        );
        q.voir_pepin(j.boite);
        pas(&mut q, &n, 60);
        assert!(
            q.surface_mer().unwrap() > j.boite.bas(),
            "pas noyé d'emblée"
        );
    }
}
