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
//! le nez de Pépin, et une lanterne de mi-hauteur (voir
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
/// puis la TOUR du port, doublée par la carte 9.13b (AGA : « la montée est trop
/// simple et trop courte, il faudrait des interactions avec la pieuvre ») : 66
/// rangées en zigzag, aucun motif plus de deux fois. Du bas vers le haut :
/// - la corniche du bas (rangée 66) et sa lanterne, la reprise de la phase 2 ;
/// - deux caisses (63, 60), une PLATEFORME MOBILE (`1`, rangée 57) qui mène à
///   gauche, une caisse contre le mur (54), un saut plongeant par le trou de la
///   corniche large 49 ;
/// - RENCONTRE 1 (`$`, rangée 48) : la tête agrippée à la corniche 49 tient la
///   TRAPPE `-` de la corniche 44 ; frappée, elle ouvre le passage ;
/// - la LANTERNE de mi-hauteur (rangée 43), sur la corniche 44 ;
/// - une VERGUE sur son mât (41-43), deux caisses (38) dont une ARRACHABLE `%`, à
///   droite (l'autre chemin passe à gauche), deux corniches de mur (35) ;
/// - le quai de la grue (32), que BALAIE un tentacule sorti du mur gauche (`!`) ;
/// - une passerelle FRIABLE `F` (29), un passage étroit par le trou de la
///   corniche 27 ;
/// - RENCONTRE 2 (`$`, rangée 26), sur la corniche 27 : sa trappe est sur la
///   corniche 22, au-dessus de la caisse 24 ;
/// - la corniche 22, que balaie un tentacule sorti du mur droit (`!`) ;
/// - deux caisses (19) dont une ARRACHABLE à gauche (l'autre chemin passe à
///   droite), deux autres (16), la corniche large 13, une caisse (10), puis la
///   VIGIE du dernier mât (7), où la tête attend (`&`), gardée par deux
///   tentacules.
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
B........&.........B
B........#####.....B
B..........#.......B
B..........#.......B
B..............###.B
B..................B
B..................B
B....#########.....B
B..................B
B..................B
B..###........###..B
B..................B
B..................B
B.....%%%..###.....B
B..................B
B.................!B
B#---##############B
B..................B
B.###..............B
B..................B
B.....$............B
B############...###B
B..................B
B............FFF...B
B..................B
B!.................B
B...############...B
B..................B
B..................B
B##.............###B
B..................B
B..................B
B..###.......%%%...B
B..................B
B..................B
B......#####.......B
B........#.........B
B..L.....#.........B
B##############---#B
B..................B
B..............###.B
B..................B
B...........$......B
B###...############B
B..................B
B...###............B
B..................B
B..................B
B##................B
B..................B
B..................B
B...1....1.........B
B..................B
B..................B
B..........###.....B
B..................B
B..................B
B..............###.B
B..................B
B............L.....B
B#######....#######B
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
        EtatArrachage, EtatBalai, EtatTentacule, PhasePieuvre, Pieuvre, ANNONCE_BALAI_PAS,
        ANNONCE_PAS, ANNONCE_SAISIE_PAS, BOUT_BALAI, FURIE_PAS, GARDE_PAS, PLONGEE_PAS, RECUL_MER,
        RYTHME_TENTACULE_PAS, SURGIT_PAS, VITESSE_BALAI,
    };
    use crate::entities::{Actualisable, Entite};
    use crate::level::Niveau;
    use crate::levels::essai_pieuvre;
    use crate::physics::{Environnement, JOUEUR_L};
    use crate::player::Joueur;

    /// Première rangée de l'arène : la tour et la bâche sont au-dessus.
    const HAUT_ARENE: i32 = 69;

    /// L'arène du boss : pas de drapeau, une seule pieuvre, trois lanternes (au
    /// départ, au début de la tour : carte 9.13a, à mi-hauteur : carte 9.13b),
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
        assert_eq!(n.nb_lanternes(), 3);
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
        // Carte 9.13a : la tour a sa tête finale et sa bâche ; carte 9.13b : six
        // paliers (sous ses six corniches larges), deux rencontres, deux
        // balayeurs, deux plateformes arrachables.
        assert!(p.tete_finale.is_some() && p.bache.is_some());
        assert_eq!(p.paliers.len(), 6, "{:?}", p.paliers);
        assert_eq!(p.tour.rencontres.len(), 2);
        assert_eq!(p.tour.balayeurs.len(), 2);
        assert_eq!(p.tour.arrachables.len(), 2);
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

    /// Une issue à chaque tentacule de la montée (le raisonnement de
    /// `chaque_motif_se_traverse`, sur la vraie tour) : à chaque lancer possible,
    /// pour chaque place de Pépin (au px près) sur un appui que le tentacule dressé
    /// frappera, il existe sur le MÊME appui une place qu'il épargne, que Pépin
    /// rejoint en courant pendant l'annonce (1,6 px par pas). Et le rythme : un
    /// tentacule toutes les 2 à 2,5 s.
    #[test]
    fn une_issue_a_chaque_tentacule() {
        let n = INFOS.charger().unwrap();
        let p = en_montee(&n, 0);
        let rythme = f32::from(RYTHME_TENTACULE_PAS) / 60.0;
        assert!((2.0..=2.5).contains(&rythme));
        let fin = (0..u16::MAX)
            .find(|&t| (p.surface_mer_a(t) - p.arret_mer).abs() < 0.01)
            .unwrap();
        let appuis = p.appuis(&n);
        let mut lancers = 0;
        for t in (p.fin_de_crue()..fin).step_by(3) {
            for x in (16..304).step_by(8) {
                let Some(i) = p.colonne_de_montee(&n, x as f32, t) else {
                    continue;
                };
                lancers += 1;
                for k in 0..30 {
                    let mer = p.surface_mer_a(t + ANNONCE_PAS + k);
                    let corps = p.boite_tentacule_montee(i, mer);
                    for &(y, a, b) in &appuis {
                        let pepin = |q: f32| Aabb::new(q, y - 14.0, JOUEUR_L, 14.0);
                        let frappe = |q: f32| corps.chevauche(&pepin(q));
                        let places: Vec<f32> = (a as i32..=(b - JOUEUR_L) as i32)
                            .map(|q| q as f32)
                            .collect();
                        for &q in places.iter().filter(|&&q| frappe(q)) {
                            let issue = places.iter().any(|&r| {
                                !frappe(r) && (r - q).abs() <= 1.6 * f32::from(ANNONCE_PAS)
                            });
                            assert!(
                                issue,
                                "t = {t}, colonne {i}, appui {:?}, Pépin en {q}",
                                (y, a, b)
                            );
                        }
                    }
                }
            }
        }
        assert!(lancers > 1000, "{lancers}");
    }

    /// Le dernier coup : Pépin tombe sur la tête du haut de la tour, elle plonge,
    /// puis elle est vaincue : le niveau est fini.
    #[test]
    fn le_dernier_coup_finit_le_niveau() {
        let n = INFOS.charger().unwrap();
        let mut p = en_montee(&n, 300);
        gagner_les_rencontres(&mut p, &n);
        let tete = p.tete_finale.unwrap();
        assert_eq!(p.tete(), Some(tete));
        let mut j = Joueur::new(tete.x + 19.0, tete.y - 12.0);
        j.vitesse.y = 100.0;
        assert!(p.recoit(&j));
        p.toucher();
        assert_eq!(p.phase(), PhasePieuvre::Plongee);
        pas(&mut p, &n, u32::from(PLONGEE_PAS));
        assert!(p.est_vaincue());
        assert!(Entite::Pieuvre(Box::new(p)).boss_vaincu());
        // Jamais de tentacule de la montée sans la montée.
        let q = pieuvre(&n);
        assert!(q.tentacule_de_montee().is_none());
    }

    /// Pépin debout à `x` (milieu), les pieds en `bas`.
    fn pepin_debout(x: f32, bas: f32) -> Aabb {
        Aabb::new(x - JOUEUR_L / 2.0, bas - 14.0, JOUEUR_L, 14.0)
    }

    /// Un saut qui retombe sur `tete`.
    fn saut_sur(tete: Aabb) -> Joueur {
        let mut j = Joueur::new(tete.x + 19.0, tete.y - 12.0);
        j.vitesse.y = 100.0;
        j
    }

    /// Les deux rencontres, gagnées dans l'ordre comme en jeu (Pépin approche,
    /// attend que la tête s'agrippe, lui saute dessus). Vérifie au passage que
    /// chacune BLOQUE (sa trappe solide, la mer arrêtée) puis se DÉBLOQUE (la
    /// trappe ouverte, la mer qui RECULE de 2 tuiles).
    fn gagner_les_rencontres(p: &mut Pieuvre, n: &Niveau) {
        for k in 0..p.tour.rencontres.len() {
            let r = p.tour.rencontres[k];
            assert_eq!(p.solide(), Some(r.trappe), "rencontre {k} : trappe fermée");
            assert!(p.rencontre().is_none());
            p.voir_pepin(pepin_debout(r.tete.x - 40.0, r.tete.bas()));
            pas(p, n, 1);
            assert_eq!(p.rencontre(), Some(&r), "rencontre {k} : la tête surgit");
            assert!(!p.recoit(&saut_sur(r.tete)), "pas frappable en surgissant");
            let mer = p.surface_mer().unwrap();
            pas(p, n, u32::from(SURGIT_PAS) + 200);
            assert_eq!(p.surface_mer().unwrap(), mer, "la mer s'arrête");
            assert_eq!(p.solide(), Some(r.trappe), "elle bloque toujours");
            let pv = p.pv();
            assert!(p.recoit(&saut_sur(r.tete)));
            p.toucher();
            assert_eq!(p.pv(), pv - 1);
            assert!(p.rencontre().is_none() && p.tete_lachee().is_some());
            assert_ne!(p.solide(), Some(r.trappe), "la trappe s'ouvre");
            // Le plus bas où elle redescend (puis elle remonte).
            let mut plus_bas = mer;
            for _ in 0..120 {
                pas(p, n, 1);
                plus_bas = plus_bas.max(p.surface_mer().unwrap());
            }
            let recul = plus_bas - mer;
            assert!(
                (recul - RECUL_MER).abs() < 2.0,
                "rencontre {k} : la mer recule de 2 tuiles ({recul})"
            );
            // Puis elle reprend sa montée.
            pas(p, n, 200);
            assert!(p.surface_mer().unwrap() < plus_bas);
        }
        assert_eq!(p.solide(), None);
        assert_eq!(p.rencontres_gagnees(), 2);
    }

    /// Carte 9.13b : chaque rencontre bloque le passage et se débloque d'un saut
    /// sur la tête ; deux points de vie, puis le coup final (3 en tout).
    #[test]
    fn chaque_rencontre_bloque_puis_se_debloque() {
        let n = INFOS.charger().unwrap();
        let mut p = en_montee(&n, 300);
        assert_eq!(p.pv(), 3);
        // Les rencontres sont au tiers et aux deux tiers de la tour.
        let bas = p.bache.unwrap().y;
        let haut = p.tete_finale.unwrap().bas();
        for (r, part) in p.tour.rencontres.iter().zip([1.0 / 3.0, 2.0 / 3.0]) {
            let ou = (bas - r.tete.bas()) / (bas - haut);
            assert!((ou - part).abs() < 0.12, "{ou}");
            assert!(
                r.trappe.bas() <= r.tete.y,
                "la trappe est au-dessus de la tête"
            );
        }
        gagner_les_rencontres(&mut p, &n);
        assert_eq!(p.pv(), 1);
    }

    /// Les tentacules de garde d'une rencontre et du coup final : annoncés comme en
    /// phase 1, un seul à la fois, à gauche puis à droite de la tête ; il reste
    /// toujours une place sûre sur la corniche, et la tête à frapper.
    #[test]
    fn les_gardes_laissent_une_issue() {
        let n = INFOS.charger().unwrap();
        let mut p = en_montee(&n, 300);
        let r = p.tour.rencontres[0];
        p.voir_pepin(pepin_debout(r.tete.x - 40.0, r.tete.bas()));
        let mut cotes = Vec::new();
        for _ in 0..(SURGIT_PAS + 4 * GARDE_PAS) {
            pas(&mut p, &n, 1);
            if let Some((corps, etat)) = p.garde_en_cours() {
                assert!(!corps.chevauche(&r.tete), "jamais sur la tête");
                if etat == EtatTentacule::Dresse {
                    if cotes.last() != Some(&corps.x) {
                        cotes.push(corps.x);
                    }
                    // Une place sûre sur la corniche, de chaque côté possible.
                    let surs = (16..304)
                        .filter(|&x| !p.blesse(&pepin_debout(x as f32, r.tete.bas())))
                        .count();
                    assert!(surs > 200, "{surs}");
                }
            }
        }
        assert_eq!(cotes.len(), 4, "gauche, droite, gauche, droite : {cotes:?}");
        assert!(cotes[0] < r.tete.x && cotes[1] > r.tete.x && cotes[0] == cotes[2]);
    }

    /// Les balayeurs : au moins deux, chacun annoncé 1 s puis qui glisse d'un mur
    /// à l'autre au ras de sa corniche ; un saut le passe (son bout est plus bas
    /// qu'un saut, et passe sous Pépin bien avant qu'il retombe).
    #[test]
    fn le_balayeur_s_annonce_et_se_saute() {
        use crate::physics::{GRAVITE, VITESSE_SAUT};
        let n = INFOS.charger().unwrap();
        let p0 = en_montee(&n, 300);
        assert!(p0.tour.balayeurs.len() >= 2);
        for (k, b) in p0.tour.balayeurs.iter().enumerate() {
            let mut p = p0.clone();
            let x = (b.depart + b.arrivee) / 2.0;
            p.voir_pepin(pepin_debout(x, b.y));
            pas(&mut p, &n, 1);
            assert!(matches!(p.etat_balayeur(k), Some(EtatBalai::Annonce(_))));
            let mut touche = false;
            for _ in 0..ANNONCE_BALAI_PAS - 1 {
                pas(&mut p, &n, 1);
                assert!(!p.blesse(&pepin_debout(x, b.y)), "l'annonce ne tue pas");
            }
            for _ in 0..b.course_pas() {
                pas(&mut p, &n, 1);
                let Some(EtatBalai::Balaie(bout)) = p.etat_balayeur(k) else {
                    panic!("balayeur {k} : il glisse");
                };
                assert!(bout.bas() <= b.y + 0.01 && bout.y >= b.y - 12.01);
                touche |= p.blesse(&pepin_debout(x, b.y));
            }
            assert!(touche, "balayeur {k} : il balaie toute la corniche");
        }
        // Le saut : les pieds au-dessus du bout (12 px) pendant plus longtemps
        // que le bout ne met à passer sous Pépin.
        let (v, g) = (VITESSE_SAUT / 60.0, GRAVITE / 3600.0);
        let mut y = 0.0;
        let mut vy = v;
        let mut au_dessus = 0;
        while y >= 0.0 {
            y += vy;
            vy -= g;
            if y > BOUT_BALAI.1 {
                au_dessus += 1;
            }
        }
        let passage = ((BOUT_BALAI.0 + JOUEUR_L) / VITESSE_BALAI).ceil() as i32;
        assert!(au_dessus > 2 * passage, "{au_dessus} pas contre {passage}");
    }

    /// Les arrachages : annoncés 1,3 s (elle porte encore), puis emportés pour de
    /// bon ; et après CHAQUE arrachage (et tous à la fois), le robot de
    /// faisabilité trouve encore un chemin jusqu'à la vigie.
    #[test]
    fn apres_chaque_arrachage_un_chemin_existe() {
        use crate::faisabilite::{analyser, Options};
        let n = INFOS.charger().unwrap();
        let mut p = en_montee(&n, 300);
        assert!((2..=3).contains(&p.tour.arrachables.len()));
        let cases = p.tour.arrachables[0].clone();
        let (x, y) = cases[0];
        let dessous = pepin_debout(x as f32 * TAILLE_TUILE, (y + 3) as f32 * TAILLE_TUILE);
        p.voir_pepin(dessous);
        pas(&mut p, &n, 1);
        assert!(matches!(
            p.etat_arrachage(0),
            Some(EtatArrachage::Annonce(_))
        ));
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

    /// Carte 9.13b : une mort après la lanterne de mi-hauteur y ramène ; la 1re
    /// rencontre reste gagnée, la 2e repart à neuf, la mer revient sous la
    /// lanterne (pas tout en bas).
    #[test]
    fn reprise_a_la_lanterne_de_mi_hauteur() {
        let n = INFOS.charger().unwrap();
        let mut p = en_montee(&n, 300);
        gagner_les_rencontres(&mut p, &n);
        let bache = p.bache.unwrap();
        let mi = n
            .entites
            .iter()
            .filter_map(|e| match e {
                Entite::Lanterne(l) if l.boite.y < bache.y => Some(l.point_de_reapparition()),
                _ => None,
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap();
        let r = p.tour.rencontres.clone();
        assert!(
            mi.1 < r[0].tete.y && mi.1 > r[1].tete.y,
            "entre les deux rencontres"
        );
        let mut entites = n.entites.clone();
        for e in &mut entites {
            if let Entite::Pieuvre(q) = e {
                **q = p.clone();
            }
        }
        let j = Joueur::new(mi.0, mi.1);
        crate::entities::reprendre_apres_une_mort(&mut entites, &j.boite);
        let q = pieuvre_de(&entites);
        assert_eq!(q.rencontres_gagnees(), 1);
        assert_eq!(q.pv(), 2);
        assert_eq!(q.solide(), Some(r[1].trappe));
        let mer = q.surface_mer().unwrap();
        assert!(mer > j.boite.bas() && mer <= j.boite.bas() + 3.0 * TAILLE_TUILE + 1.0);
        // Depuis la lanterne du bas : tout repart à neuf.
        let mut entites2 = entites.clone();
        let bas = Joueur::new(40.0, bache.y - 40.0);
        crate::entities::reprendre_apres_une_mort(&mut entites2, &bas.boite);
        let q = pieuvre_de(&entites2);
        assert_eq!((q.rencontres_gagnees(), q.horloge_mer()), (0, 0));
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
}
