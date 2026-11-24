//! Variantes de la NOUVELLE PARTIE + (carte 8.8) : les mêmes niveaux, plus durs.
//!
//! Après une première partie finie, le titre propose « NOUVELLE PARTIE+ » : les
//! mêmes niveaux, dans le même ordre, avec :
//! - des ennemis DÉPLACÉS : chaque ennemi qui le peut change de côté sur son toit
//!   (ou de place dans son couloir de vol), et un ennemi de plus par niveau
//!   (table `VARIANTES`, écrite à la main, en cases de la grille) ;
//! - des éclairs plus fréquents (`REPOS_ECLAIR_NG_PLUS` au lieu de 150 pas) ;
//! - le Grand Nuage plus serré (`GrandNuage::serrer` : à chaque colère, le motif
//!   suivant) ;
//! - un léger vent de face sur les niveaux de toits (`NIVEAUX_DE_TOITS`,
//!   `physics::VENT_DE_FACE`) ;
//! - une pluie plus forte (plus dense, plus oblique) : dessin seulement, côté jeu.
//!
//! Le reste ne bouge pas : géométrie, gouttes d'or, lanternes, secrets. Les grilles
//! ASCII des niveaux ne sont PAS modifiées : la variante est appliquée au TEXTE de la
//! grille au moment du chargement (`texte_ng_plus`), puis lue par le même parseur
//! (ses vérifications valent donc aussi pour la variante). Une case attendue qui ne
//! correspond plus (grille retouchée depuis) est une erreur typée, que le test
//! `chaque_variante_ng_plus_est_finissable` (`niveaux/tests/tous_les_niveaux/`)
//! attrape avant le jeu.
//!
//! Les ennemis restent à au moins 6 cases de chaque lanterne et du départ (pas de
//! mort en boucle à la reprise) et loin des secrets ; ce même test vérifie la
//! distance et la faisabilité de chaque variante (robot de faisabilité, depuis le
//! départ et depuis chaque lanterne). Le robot de QA, lui, finit la nouvelle partie
//! + complète sans mourir (`game/src/bot_qa`).
//!
//! Notions Rust illustrées ici : tableaux constants de tuples, `Vec<Vec<char>>`
//! pour modifier un texte case par case, erreur typée qui ENVELOPPE une autre erreur
//! (`From<ErreurNiveau>` pour l'opérateur `?`).

use std::fmt;

use crate::aabb::Aabb;
use crate::entities::Entite;
use crate::level::{ErreurNiveau, Niveau};
use crate::physics::VentLateral;
use crate::zones::InfosNiveau;

/// Pause (pas) entre deux coups d'éclair en nouvelle partie + : 110 au lieu de 150.
/// Le cycle (annonce, frappe, pause) passe de 3,5 s à 2,8 s ; l'annonce, elle, ne
/// change pas (0,8 s).
pub const REPOS_ECLAIR_NG_PLUS: u16 = 110;

/// Les niveaux de toits (clés) : le léger vent de face y souffle en nouvelle partie +.
pub const NIVEAUX_DE_TOITS: [&str; 4] = ["toits", "gouttieres", "marche", "orage"];

/// Une case de la grille : `(colonne, rangée)`, à partir de 0, comme dans le dessin.
pub type Case = (usize, usize);

/// La variante d'un niveau, désigné par sa CLÉ.
#[derive(Debug, Clone, Copy)]
pub struct Variante {
    pub cle: &'static str,
    /// Ennemis déplacés : `(case du dessin, case en nouvelle partie +)`. La lettre
    /// (flaque, corbeau, pigeon, chat) suit l'ennemi ; la case d'arrivée doit être
    /// vide (`.`).
    pub deplaces: &'static [(Case, Case)],
    /// Ennemis ajoutés : `(lettre de la légende, case vide)`.
    pub ajouts: &'static [(char, Case)],
}

/// Les variantes, niveau par niveau. L'arène du Grand Nuage (son motif suffit) et
/// l'arc-en-ciel (le bonus reste « sans ennemi ») n'en ont pas. Les pigeons sur un
/// perchoir d'une case, les ennemis gardiens d'un secret et le chat du clocher
/// restent à leur place.
pub const VARIANTES: &[Variante] = &[
    Variante {
        cle: "toits",
        deplaces: &[((76, 7), (79, 7)), ((118, 7), (116, 7))],
        ajouts: &[('f', (25, 7))],
    },
    Variante {
        cle: "gouttieres",
        deplaces: &[
            ((70, 3), (74, 3)),
            ((50, 5), (44, 5)),
            ((20, 7), (14, 7)),
            ((46, 7), (42, 7)),
            ((52, 7), (49, 7)),
            ((90, 7), (87, 7)),
            ((97, 7), (94, 7)),
            ((108, 7), (118, 7)),
            ((150, 7), (146, 7)),
            ((160, 7), (162, 7)),
        ],
        ajouts: &[('f', (127, 6))],
    },
    Variante {
        cle: "marche",
        deplaces: &[
            ((68, 4), (65, 4)),
            ((12, 7), (17, 7)),
            ((34, 7), (29, 7)),
            ((128, 7), (122, 7)),
        ],
        ajouts: &[('c', (92, 4))],
    },
    Variante {
        cle: "orage",
        deplaces: &[((36, 5), (42, 5)), ((154, 7), (184, 7))],
        ajouts: &[('f', (105, 7))],
    },
    // Le clocher : le chat de la galerie reste deux tuiles avant le ventilateur (sa
    // place va avec le souffle, carte 6.4a) ; le corbeau passe au bout de la
    // corniche F4 ; une flaque garde la corniche F2, où l'on saute en sortant du
    // courant du ventilateur.
    Variante {
        cle: "clocher",
        deplaces: &[((26, 18), (29, 18))],
        ajouts: &[('f', (28, 25))],
    },
    Variante {
        cle: "vents",
        deplaces: &[((60, 2), (66, 2)), ((76, 3), (82, 3)), ((147, 3), (153, 3))],
        ajouts: &[('f', (147, 3))],
    },
    Variante {
        cle: "mer",
        deplaces: &[
            ((65, 3), (77, 3)),
            ((85, 4), (91, 4)),
            ((162, 6), (168, 6)),
            ((97, 7), (91, 7)),
        ],
        ajouts: &[('f', (56, 7))],
    },
    Variante {
        cle: "nuages",
        deplaces: &[((120, 2), (126, 2)), ((63, 3), (69, 3)), ((37, 5), (40, 5))],
        ajouts: &[('f', (29, 6))],
    },
    Variante {
        cle: "poursuite",
        deplaces: &[((146, 3), (152, 3))],
        ajouts: &[('f', (79, 7))],
    },
    Variante {
        cle: "descente",
        deplaces: &[((10, 12), (16, 12))],
        ajouts: &[('c', (20, 28))],
    },
];

/// Lettres d'ennemis qu'une variante peut déplacer ou ajouter.
const LETTRES_ENNEMIS: [char; 4] = ['f', 'c', 'p', 'z'];

/// Ce qui empêche d'appliquer une variante.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErreurVariante {
    /// La case `(colonne, rangée)` du niveau `cle` ne contient pas ce qui est
    /// attendu (un ennemi à déplacer, ou une case vide où le poser).
    CaseInattendue {
        cle: &'static str,
        case: Case,
        attendu: &'static str,
        trouve: char,
    },
    /// La grille modifiée ne se lit plus (erreur du parseur).
    Niveau(ErreurNiveau),
}

impl fmt::Display for ErreurVariante {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErreurVariante::CaseInattendue {
                cle,
                case,
                attendu,
                trouve,
            } => write!(
                f,
                "variante « {cle} », case {case:?} : {attendu} attendu, « {trouve} » trouvé"
            ),
            ErreurVariante::Niveau(e) => write!(f, "variante illisible : {e}"),
        }
    }
}

impl std::error::Error for ErreurVariante {}

/// `?` transforme une erreur du parseur en erreur de variante.
impl From<ErreurNiveau> for ErreurVariante {
    fn from(e: ErreurNiveau) -> Self {
        ErreurVariante::Niveau(e)
    }
}

/// La variante du niveau de clé `cle` (`None` : aucun ennemi ne change).
pub fn variante(cle: &str) -> Option<&'static Variante> {
    VARIANTES.iter().find(|v| v.cle == cle)
}

/// Le dessin ASCII du niveau avec les ennemis de sa variante déplacés et ajoutés.
/// Les lignes vides du début et de la fin sont retirées (comme le fait le parseur :
/// la rangée 0 est la première ligne du dessin).
pub fn texte_ng_plus(infos: &InfosNiveau) -> Result<String, ErreurVariante> {
    let mut grille: Vec<Vec<char>> = infos
        .ascii
        .lines()
        .map(|l| l.trim_end().chars().collect())
        .skip_while(|l: &Vec<char>| l.is_empty())
        .collect();
    while grille.last().is_some_and(|l| l.is_empty()) {
        grille.pop();
    }
    let Some(v) = variante(infos.cle) else {
        return Ok(joindre(&grille));
    };
    // Lit (ou écrit) une case ; hors du dessin : `None`.
    let erreur = |case: Case, attendu, trouve| ErreurVariante::CaseInattendue {
        cle: v.cle,
        case,
        attendu,
        trouve,
    };
    let lire = |g: &Vec<Vec<char>>, (x, y): Case| g.get(y).and_then(|l| l.get(x)).copied();
    for &(depuis, vers) in v.deplaces {
        let lettre = lire(&grille, depuis).unwrap_or(' ');
        if !LETTRES_ENNEMIS.contains(&lettre) {
            return Err(erreur(depuis, "un ennemi", lettre));
        }
        let cible = lire(&grille, vers).unwrap_or(' ');
        if cible != '.' {
            return Err(erreur(vers, "une case vide", cible));
        }
        grille[depuis.1][depuis.0] = '.';
        grille[vers.1][vers.0] = lettre;
    }
    for &(lettre, case) in v.ajouts {
        if !LETTRES_ENNEMIS.contains(&lettre) {
            return Err(erreur(case, "une lettre d'ennemi", lettre));
        }
        let cible = lire(&grille, case).unwrap_or(' ');
        if cible != '.' {
            return Err(erreur(case, "une case vide", cible));
        }
        grille[case.1][case.0] = lettre;
    }
    Ok(joindre(&grille))
}

fn joindre(grille: &[Vec<char>]) -> String {
    grille
        .iter()
        .map(|l| l.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Charge le niveau en nouvelle partie + : la grille de sa variante, puis les
/// réglages communs (éclairs plus fréquents, Grand Nuage serré, vent de face sur
/// les toits).
pub fn charger_ng_plus(infos: &InfosNiveau) -> Result<Niveau, ErreurVariante> {
    let mut niveau = Niveau::depuis_ascii_avec_aiguilles(&texte_ng_plus(infos)?, infos.aiguilles)?;
    niveau.regler_sauts_poissons(infos.poissons_hauts)?;
    for e in &mut niveau.entites {
        match e {
            Entite::Eclair(eclair) => eclair.changer_repos(REPOS_ECLAIR_NG_PLUS),
            Entite::Nuage(nuage) => nuage.serrer(),
            _ => {}
        }
    }
    if NIVEAUX_DE_TOITS.contains(&infos.cle) {
        // Tout le niveau, et un peu au-dessus (un saut peut sortir du haut de la carte).
        let zone = Aabb::new(
            0.0,
            -niveau.hauteur_px(),
            niveau.largeur_px(),
            2.0 * niveau.hauteur_px(),
        );
        niveau.vents.push(VentLateral::de_face(zone));
    }
    Ok(niveau)
}
