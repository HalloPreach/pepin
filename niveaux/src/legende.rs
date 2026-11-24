//! La LÉGENDE de la grille, en données (carte 10.3a) : une entrée par lettre,
//! avec ce que l'éditeur de niveaux (`editeur/`) doit savoir pour la proposer
//! dans ses palettes et prévenir des erreurs courantes.
//!
//! C'est la SEULE liste de lettres hors du parseur : l'éditeur la lit dans
//! `legende.json` (écrit par `--bin editeur`, lancé par `./build.sh --qa`), il
//! n'en recopie aucune. Le test `la_legende_couvre_exactement_les_lettres_du_parseur`
//! essaie chaque caractère ASCII avec le parseur de `core` : une lettre ajoutée au
//! jeu sans sa ligne ici (ou une ligne en trop) fait échouer les tests.
//!
//! Notions Rust : tableau `const` de structures, énumération sans données
//! (`Categorie`), écriture d'un JSON à la main (pas de dépendance en plus pour
//! quelques dizaines de lignes de texte).

use pepin_core::zones::Theme;

use crate::format::{GenreFichier, ThemeFichier};

/// Le groupe d'une lettre dans les palettes de l'éditeur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Categorie {
    /// Le vide et les cases solides (toits, briques, mobilier...).
    Tuile,
    /// Départ, arrivées : les repères du parcours.
    Repere,
    /// Ce que Pépin ramasse ou allume : gouttes, lettre, lanternes.
    Collectible,
    Ennemi,
    /// Mécanismes et pièges : champignon, vents, éclairs, plateformes mobiles...
    Objet,
    /// L'eau du Monde 2 (calme ou en courant).
    Eau,
    /// Les boss et leur arène.
    Boss,
    /// Panneaux et décor sans collision.
    Decor,
}

impl Categorie {
    /// Le nom écrit dans `legende.json` (et lu par l'éditeur).
    pub fn nom(self) -> &'static str {
        match self {
            Categorie::Tuile => "tuile",
            Categorie::Repere => "repere",
            Categorie::Collectible => "collectible",
            Categorie::Ennemi => "ennemi",
            Categorie::Objet => "objet",
            Categorie::Eau => "eau",
            Categorie::Boss => "boss",
            Categorie::Decor => "decor",
        }
    }
}

/// Ce qui doit tenir une lettre dans la grille (vérifié par l'éditeur, en
/// avertissement : le juge reste le jeu et les tests de `core/src/zones.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Appui {
    /// Rien : la case peut être en l'air.
    Libre,
    /// Un solide dans la case du dessous (décor, ennemis qui marchent...).
    SolideDessous,
    /// Un solide ou de l'eau dessous (la lanterne : `aucune_lanterne_ne_flotte`).
    SolideOuEauDessous,
    /// Un solide dans la case du dessus (la stalactite `V`).
    SolideDessus,
    /// De l'eau dessous (le poisson `i`).
    EauDessous,
}

impl Appui {
    /// Le nom écrit dans `legende.json`.
    pub fn nom(self) -> &'static str {
        match self {
            Appui::Libre => "libre",
            Appui::SolideDessous => "solide_dessous",
            Appui::SolideOuEauDessous => "solide_ou_eau_dessous",
            Appui::SolideDessus => "solide_dessus",
            Appui::EauDessous => "eau_dessous",
        }
    }
}

/// Combien de fois une lettre peut apparaître dans une grille.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nombre {
    Libre,
    /// Au plus N (le parseur refuse au-delà) : l'éditeur REMPLACE l'ancienne.
    AuPlus(u8),
    /// Exactement N (les 3 gouttes d'or) : l'éditeur avertit tant que le compte
    /// n'y est pas.
    Exactement(u8),
}

/// Une lettre de la grille.
#[derive(Debug, Clone, Copy)]
pub struct Lettre {
    pub lettre: char,
    pub nom: &'static str,
    pub categorie: Categorie,
    /// La case bloque Pépin (pour l'aperçu et les avertissements de l'éditeur).
    pub solide: bool,
    /// Ce qui doit la tenir (« posée sur un solide » ou non) : sinon l'éditeur avertit.
    pub appui: Appui,
    /// Place occupée dans le jeu, en tuiles (le repère est en bas à gauche).
    pub largeur: u8,
    pub hauteur: u8,
    pub nombre: Nombre,
    /// Couleur de la case dans l'éditeur (`#rrggbb`).
    pub couleur: &'static str,
}

/// Raccourci de lecture pour le tableau ci-dessous (1 x 1, nombre libre).
const fn l(
    lettre: char,
    nom: &'static str,
    categorie: Categorie,
    solide: bool,
    appui: Appui,
    couleur: &'static str,
) -> Lettre {
    Lettre {
        lettre,
        nom,
        categorie,
        solide,
        appui,
        largeur: 1,
        hauteur: 1,
        nombre: Nombre::Libre,
        couleur,
    }
}

/// La même lettre, avec une taille.
const fn taille(mut x: Lettre, largeur: u8, hauteur: u8) -> Lettre {
    x.largeur = largeur;
    x.hauteur = hauteur;
    x
}

/// La même lettre, avec un nombre limité.
const fn nombre(mut x: Lettre, nombre: Nombre) -> Lettre {
    x.nombre = nombre;
    x
}

use Appui::*;
use Categorie::*;

/// Toutes les lettres de la grille (le sens détaillé de chacune : la légende de
/// `assets/levels/LISEZMOI.md`, recopiée de `core/src/level.rs`).
pub const LETTRES: &[Lettre] = &[
    // --- Tuiles ---
    l('.', "vide", Tuile, false, Libre, "#20263a"),
    l('#', "toit", Tuile, true, Libre, "#8a5a44"),
    l('B', "brique", Tuile, true, Libre, "#a04a3a"),
    l('N', "nuage (solide)", Tuile, true, Libre, "#c8d0e0"),
    l('F', "plateforme friable", Tuile, true, Libre, "#b08a5a"),
    l('m', "faux mur (case libre)", Tuile, false, Libre, "#7a4a3a"),
    l('%', "plateforme arrachable", Tuile, true, Libre, "#9a7a5a"),
    l('_', "bâche de l'arène", Tuile, true, Libre, "#5a7a9a"),
    taille(l('q', "voiture garée", Tuile, true, Libre, "#c04848"), 4, 2),
    l('a', "autobus garé", Tuile, true, Libre, "#d8a030"),
    l('t', "toit d'abribus", Tuile, true, Libre, "#6a8aa0"),
    l('d', "poubelle de rue", Tuile, true, Libre, "#4a6a4a"),
    l('l', "banc", Tuile, true, Libre, "#8a6a3a"),
    // --- Repères ---
    nombre(
        l('@', "départ de Pépin", Repere, false, Libre, "#40e070"),
        Nombre::AuPlus(1),
    ),
    nombre(
        l('X', "arrivée (drapeau)", Repere, false, Libre, "#f0f0f0"),
        Nombre::AuPlus(1),
    ),
    nombre(
        l('x', "sortie secrète", Repere, false, Libre, "#c0c0ff"),
        Nombre::AuPlus(1),
    ),
    // --- Collectibles ---
    nombre(
        l('o', "goutte d'or", Collectible, false, Libre, "#f0c020"),
        Nombre::Exactement(3),
    ),
    nombre(
        l('*', "lettre cachée", Collectible, false, Libre, "#ff80c0"),
        Nombre::AuPlus(1),
    ),
    l(
        'L',
        "lanterne (checkpoint)",
        Collectible,
        false,
        SolideOuEauDessous,
        "#ffb040",
    ),
    // --- Ennemis ---
    l('f', "flaque", Ennemi, false, SolideDessous, "#4080e0"),
    l('c', "corbeau", Ennemi, false, Libre, "#303040"),
    l('s', "escargot", Ennemi, false, SolideDessous, "#a0c060"),
    l('p', "pigeon", Ennemi, false, SolideDessous, "#9090a8"),
    l('z', "chat endormi", Ennemi, false, SolideDessous, "#e09050"),
    l('j', "grenouille", Ennemi, false, Libre, "#50c050"),
    l(
        'i',
        "poisson (au-dessus de l'eau)",
        Ennemi,
        false,
        EauDessous,
        "#e07050",
    ),
    l('k', "crabe", Ennemi, false, SolideDessous, "#e05040"),
    l('r', "rat", Ennemi, false, SolideDessous, "#807060"),
    l('u', "mouette", Ennemi, false, SolideDessous, "#e8e8f0"),
    l(
        'V',
        "stalactite (sous un solide)",
        Ennemi,
        false,
        SolideDessus,
        "#7080a0",
    ),
    // --- Objets et mécanismes ---
    l('^', "pointes", Objet, false, Libre, "#d0d0d0"),
    l('M', "champignon-trampoline", Objet, false, Libre, "#e04060"),
    l('A', "auvent", Objet, false, Libre, "#e0a0a0"),
    l('e', "éclair", Objet, false, Libre, "#ffff60"),
    l('g', "grêle", Objet, false, Libre, "#d0f0ff"),
    l('v', "courant ascendant", Objet, false, Libre, "#80c0e0"),
    l('<', "vent vers la gauche", Objet, false, Libre, "#a0d0e0"),
    l('>', "vent vers la droite", Objet, false, Libre, "#a0d0e0"),
    l('w', "ventilateur", Objet, false, SolideDessous, "#a0a0b0"),
    taille(l('b', "montgolfière", Objet, false, Libre, "#e06040"), 1, 3),
    l(
        'h',
        "pivot d'aiguille d'horloge",
        Objet,
        false,
        Libre,
        "#c0a060",
    ),
    nombre(
        l('P', "mur de pluie", Objet, false, Libre, "#6060a0"),
        Nombre::AuPlus(1),
    ),
    l(
        '1',
        "plateforme mobile 1 (2 extrémités)",
        Objet,
        false,
        Libre,
        "#b0b060",
    ),
    l('2', "plateforme mobile 2", Objet, false, Libre, "#b0b060"),
    l('3', "plateforme mobile 3", Objet, false, Libre, "#b0b060"),
    l('4', "plateforme mobile 4", Objet, false, Libre, "#b0b060"),
    l('5', "plateforme mobile 5", Objet, false, Libre, "#b0b060"),
    l('6', "plateforme mobile 6", Objet, false, Libre, "#b0b060"),
    l('7', "plateforme mobile 7", Objet, false, Libre, "#b0b060"),
    l('9', "plateforme mobile 9", Objet, false, Libre, "#b0b060"),
    l('Y', "bulles", Objet, false, Libre, "#80e0f0"),
    l('n', "nénuphar", Objet, false, Libre, "#60b060"),
    l('=', "marée", Objet, false, Libre, "#3060a0"),
    l('|', "chute d'eau", Objet, false, Libre, "#6090d0"),
    // Prototype du Monde 3 (carte 11.1) : le crochet du balancement.
    l('y', "crochet (balancement)", Objet, false, Libre, "#e0c040"),
    // --- Eau ---
    l('~', "eau calme", Eau, false, Libre, "#2050a0"),
    l(
        '{',
        "courant d'eau vers la gauche",
        Eau,
        false,
        Libre,
        "#2a5ab0",
    ),
    l(
        '}',
        "courant d'eau vers la droite",
        Eau,
        false,
        Libre,
        "#2a5ab0",
    ),
    // --- Boss ---
    nombre(
        taille(
            l('O', "le Grand Nuage", Boss, false, Libre, "#707088"),
            3,
            2,
        ),
        Nombre::AuPlus(1),
    ),
    nombre(
        l('8', "la Pieuvre du port", Boss, false, Libre, "#a040a0"),
        Nombre::AuPlus(1),
    ),
    taille(
        l('&', "vigie de la Pieuvre", Boss, false, Libre, "#c060c0"),
        4,
        3,
    ),
    taille(
        l('$', "jeté de la Pieuvre", Boss, false, Libre, "#b050b0"),
        4,
        3,
    ),
    l('!', "balayage de la Pieuvre", Boss, false, Libre, "#b050b0"),
    l('+', "secousse de la tour", Boss, false, Libre, "#b050b0"),
    // --- Panneaux et décor ---
    l('D', "panneau « déplacer »", Decor, false, Libre, "#c0a080"),
    l('S', "panneau « sauter »", Decor, false, Libre, "#c0a080"),
    l('G', "panneau « planer »", Decor, false, Libre, "#c0a080"),
    l(
        'C',
        "panneau « champignon »",
        Decor,
        false,
        Libre,
        "#c0a080",
    ),
    l('W', "panneau « vent »", Decor, false, Libre, "#c0a080"),
    l('Z', "panneau « barque »", Decor, false, Libre, "#c0a080"),
    l('H', "cheminée", Decor, false, SolideDessous, "#906050"),
    l('T', "antenne", Decor, false, SolideDessous, "#909090"),
    l('R', "paratonnerre", Decor, false, SolideDessous, "#909090"),
    l('Q', "manche à air", Decor, false, SolideDessous, "#e08040"),
    l('K', "oiseau posé", Decor, false, SolideDessous, "#606070"),
    l(
        'E',
        "bouche d'égout",
        Decor,
        false,
        SolideDessous,
        "#505060",
    ),
    l('I', "borne", Decor, false, SolideDessous, "#707080"),
    l('U', "poubelle", Decor, false, SolideDessous, "#507050"),
    l(
        'J',
        "descente de gouttière",
        Decor,
        false,
        SolideDessous,
        "#708090",
    ),
];

/// Où l'éditeur pose la vignette d'un sprite par rapport à la place de sa lettre
/// (carte 10.4b), comme le jeu le dessine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ancre {
    /// Centrée en largeur, posée sur le BAS de la place : ennemis, décors (la règle
    /// `coin_decor` du jeu), drapeau, montgolfière...
    Bas,
    /// Centrée en largeur, accrochée au HAUT de la case : la stalactite.
    Haut,
    /// Coin haut-gauche sur celui de la case : les plateformes (friable, mobile,
    /// nénuphar), qui partent de leur case vers la droite.
    Plateforme,
    /// Centrée sur la case : petits objets qui flottent (goutte, lettre, bulles),
    /// cadran d'horloge autour de son pivot.
    Centre,
}

impl Ancre {
    /// Nom écrit dans `legende.json`.
    pub fn nom(self) -> &'static str {
        match self {
            Ancre::Bas => "bas",
            Ancre::Haut => "haut",
            Ancre::Plateforme => "plateforme",
            Ancre::Centre => "centre",
        }
    }
}

/// La VIGNETTE de chaque lettre qui a un vrai sprite (carte 10.4b) : la lettre, la
/// CLÉ du sprite (le nom que le jeu demande, `assets/sprites/<famille>/<clé>.png` :
/// la famille est retrouvée dans la table des sprites, `crate::SPRITES`, écrite
/// d'après les fichiers) et son ancrage. On prend la première image de
/// l'animation, celle du repos.
///
/// Une lettre ABSENTE d'ici garde sa case colorée dans l'éditeur : les tuiles
/// (dessinées par thème, avec leurs bords calculés d'après les voisines), les zones
/// (vent, courant, pluie, marée...), le mobilier de rue dessiné par code, et les
/// repères du boss sans dessin propre. Le test
/// `chaque_vignette_est_un_vrai_sprite` vérifie que chaque clé a son fichier.
pub const VIGNETTES: &[(char, &str, Ancre)] = &[
    // Tuile dessinée par un sprite à elle (et non par le jeu de tuiles du thème).
    ('F', "friable0", Ancre::Plateforme),
    // Repères.
    ('@', "pepin_idle0", Ancre::Bas),
    ('X', "drapeau0", Ancre::Bas),
    // Collectibles.
    ('o', "goutte0", Ancre::Centre),
    ('*', "lettre_p", Ancre::Centre),
    ('L', "lanterne0", Ancre::Bas),
    // Ennemis.
    ('f', "flaque0", Ancre::Bas),
    ('c', "corbeau0", Ancre::Bas),
    ('s', "escargot0", Ancre::Bas),
    ('p', "pigeon0", Ancre::Bas),
    ('z', "chat_dort0", Ancre::Bas),
    ('j', "grenouille0", Ancre::Bas),
    ('i', "poisson0", Ancre::Bas),
    ('k', "crabe0", Ancre::Bas),
    ('r', "rat0", Ancre::Bas),
    ('u', "mouette0", Ancre::Bas),
    ('V', "stalactite", Ancre::Haut),
    // Objets et mécanismes.
    ('^', "pointe", Ancre::Bas),
    ('M', "champignon0", Ancre::Bas),
    ('A', "auvent0", Ancre::Bas),
    ('g', "grele_nuage0", Ancre::Centre),
    ('w', "ventilateur_souffle", Ancre::Bas),
    ('b', "montgolfiere_repos", Ancre::Bas),
    ('h', "cadran", Ancre::Centre),
    ('1', "mobile", Ancre::Plateforme),
    ('2', "mobile", Ancre::Plateforme),
    ('3', "mobile", Ancre::Plateforme),
    ('4', "mobile", Ancre::Plateforme),
    ('5', "mobile", Ancre::Plateforme),
    ('6', "mobile", Ancre::Plateforme),
    ('7', "mobile", Ancre::Plateforme),
    ('9', "mobile", Ancre::Plateforme),
    ('Y', "bulle0", Ancre::Centre),
    ('n', "nenuphar", Ancre::Plateforme),
    ('|', "chute0", Ancre::Bas),
    ('y', "crochet", Ancre::Centre),
    // Boss.
    ('O', "nuage_colere0", Ancre::Bas),
    ('8', "pieuvre_colere0", Ancre::Bas),
    // Panneaux et décor.
    ('D', "panneau_deplacer", Ancre::Bas),
    ('S', "panneau_sauter", Ancre::Bas),
    ('G', "panneau_planer", Ancre::Bas),
    ('C', "panneau_champignon", Ancre::Bas),
    ('W', "panneau_vent", Ancre::Bas),
    ('Z', "panneau_barque", Ancre::Bas),
    ('H', "cheminee", Ancre::Bas),
    ('T', "antenne", Ancre::Bas),
    ('R', "paratonnerre", Ancre::Bas),
    ('Q', "manche_a_air", Ancre::Bas),
    ('K', "oiseau0", Ancre::Bas),
    ('E', "bouche_egout", Ancre::Bas),
    ('I', "borne", Ancre::Bas),
    ('U', "poubelle", Ancre::Bas),
];

/// Le sprite de la lettre `c` : son chemin `famille/clé` sous `assets/sprites`
/// (sans `.png`) et son ancrage, ou `None` (case colorée). La famille vient de la
/// table des sprites : une clé sans fichier donne `None` (et fait échouer le test).
pub fn vignette(c: char) -> Option<(String, Ancre)> {
    let &(_, cle, ancre) = VIGNETTES.iter().find(|(l, _, _)| *l == c)?;
    let &(famille, _) = crate::SPRITES.iter().find(|(_, k)| *k == cle)?;
    Some((format!("{famille}/{cle}"), ancre))
}

/// Les noms des genres de niveau, tels qu'on les écrit dans un `.ron`.
const GENRES: [GenreFichier; 3] = [
    GenreFichier::Normal,
    GenreFichier::Bonus,
    GenreFichier::Epilogue,
];

/// Une chaîne JSON entre guillemets (seuls `\` et `"` peuvent apparaître ici).
fn json(texte: &str) -> String {
    format!("\"{}\"", texte.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Le contenu de `legende.json` : les lettres, les thèmes et les genres
/// (l'éditeur en fait ses palettes et les listes de son formulaire).
pub fn legende_json() -> String {
    let mut s = String::from("{\n  \"lettres\": [\n");
    for (i, x) in LETTRES.iter().enumerate() {
        let (au_plus, exactement) = match x.nombre {
            Nombre::Libre => ("null".to_string(), "null".to_string()),
            Nombre::AuPlus(n) => (n.to_string(), "null".to_string()),
            Nombre::Exactement(n) => ("null".to_string(), n.to_string()),
        };
        // Carte 10.4b : la vignette (`null` : la case colorée reste).
        let (sprite, ancre) = match vignette(x.lettre) {
            Some((chemin, ancre)) => (json(&chemin), json(ancre.nom())),
            None => ("null".to_string(), "null".to_string()),
        };
        s.push_str(&format!(
            "    {{\"lettre\": {}, \"nom\": {}, \"categorie\": {}, \"solide\": {}, \
             \"appui\": {}, \"largeur\": {}, \"hauteur\": {}, \
             \"au_plus\": {au_plus}, \"exactement\": {exactement}, \"couleur\": {}, \
             \"sprite\": {sprite}, \"ancre\": {ancre}}}{}\n",
            json(&x.lettre.to_string()),
            json(x.nom),
            json(x.categorie.nom()),
            x.solide,
            json(x.appui.nom()),
            x.largeur,
            x.hauteur,
            json(x.couleur),
            if i + 1 < LETTRES.len() { "," } else { "" },
        ));
    }
    let themes: Vec<String> = Theme::TOUS
        .iter()
        .map(|t| json(&format!("{:?}", ThemeFichier::from(*t))))
        .collect();
    let genres: Vec<String> = GENRES.iter().map(|g| json(&format!("{g:?}"))).collect();
    s.push_str(&format!(
        "  ],\n  \"themes\": [{}],\n  \"genres\": [{}]\n}}\n",
        themes.join(", "),
        genres.join(", ")
    ));
    s
}
