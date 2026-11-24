//! Les erreurs du chargeur, LISIBLES par un humain ou par un agent : chacune dit
//! QUEL fichier, et où (ligne et colonne) quand c'est possible.
//!
//! Notions Rust : une structure d'erreur qui ENVELOPPE les erreurs typées de
//! `core` (`ErreurNiveau` de la grille, `ErreurOrdre` de la table) au lieu de les
//! recopier ; `Display` écrit le message, `std::error::Error` le rend compatible
//! avec `?` et `Box<dyn Error>`.

use pepin_core::level::ErreurNiveau;
use pepin_core::levels::ErreurOrdre;
use std::fmt;

/// Une erreur de chargement : le fichier en cause et ce qui ne va pas.
#[derive(Debug, Clone, PartialEq)]
pub struct ErreurChargement {
    /// Chemin du fichier sous `assets/levels` (`monde_1/toits.ron`), ou
    /// `(texte)` pour un texte chargé sans nom (`crate::charger`).
    pub fichier: String,
    pub genre: GenreErreur,
}

/// Ce qui ne va pas dans un fichier, ou dans la table entière.
#[derive(Debug, Clone, PartialEq)]
pub enum GenreErreur {
    /// Le RON est mal écrit (parenthèse, virgule, champ inconnu ou manquant,
    /// thème inconnu...). `ligne` et `colonne` sont celles du FICHIER.
    Syntaxe {
        ligne: usize,
        colonne: usize,
        message: String,
    },
    /// La grille est refusée par le parseur de `core` (lettre inconnue, lignes de
    /// longueurs différentes, `@` manquant ou en double, pas de sortie `X`,
    /// aiguilles ou poissons qui ne correspondent à aucune case...). Les lignes de
    /// l'erreur sont celles de la GRILLE ; `premiere_ligne` est la ligne du
    /// fichier où commence la grille (pour retrouver la case dans l'éditeur).
    Grille {
        erreur: ErreurNiveau,
        premiere_ligne: usize,
    },
    /// Le fichier n'est pas dans un dossier connu (`monde_<N>` ou `essais`).
    DossierInconnu,
    /// Le champ `monde` ne correspond pas au dossier `monde_<N>`.
    MondeHorsDeSonDossier { dossier: u8, monde: u8 },
    /// Un niveau d'essai doit avoir `numero: 0` (il n'est pas dans le parcours).
    EssaiNumerote { numero: u8 },
    /// La clé doit être le nom du fichier (`cle: "toits"` dans `toits.ron`).
    CleDifferenteDuFichier { cle: String },
    /// Deux fichiers portent la même clé.
    CleEnDouble { cle: String, autre_fichier: String },
    /// La table rangée par (`monde`, `numero`) viole une règle d'ordre de `core`
    /// (numérotation trouée ou en double, monde manquant, bonus mal placé...).
    Ordre(ErreurOrdre),
}

impl fmt::Display for ErreurChargement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} : ", self.fichier)?;
        match &self.genre {
            GenreErreur::Syntaxe {
                ligne,
                colonne,
                message,
            } => write!(f, "ligne {ligne}, colonne {colonne} du fichier : {message}"),
            GenreErreur::Grille {
                erreur,
                premiere_ligne,
            } => write!(
                f,
                "grille (sa ligne 1 est la ligne {premiere_ligne} du fichier) : {erreur}"
            ),
            GenreErreur::DossierInconnu => write!(
                f,
                "dossier inconnu (attendu : monde_1/, monde_2/... ou essais/)"
            ),
            GenreErreur::MondeHorsDeSonDossier { dossier, monde } => write!(
                f,
                "« monde: {monde} » dans le dossier monde_{dossier}/ (ils doivent être égaux)"
            ),
            GenreErreur::EssaiNumerote { numero } => write!(
                f,
                "niveau d'essai avec « numero: {numero} » (un essai a toujours le numéro 0)"
            ),
            GenreErreur::CleDifferenteDuFichier { cle } => write!(
                f,
                "la clé « {cle} » n'est pas le nom du fichier (attendu : {cle}.ron)"
            ),
            GenreErreur::CleEnDouble { cle, autre_fichier } => {
                write!(f, "clé « {cle} » déjà prise par {autre_fichier}")
            }
            GenreErreur::Ordre(e) => write!(f, "ordre des niveaux : {}", ordre_lisible(e)),
        }
    }
}

impl std::error::Error for ErreurChargement {}

/// Une phrase pour chaque règle d'ordre de `core::levels::verifier_ordre`.
fn ordre_lisible(e: &ErreurOrdre) -> String {
    match e {
        ErreurOrdre::AucunNiveauNormal => "aucun niveau normal".into(),
        ErreurOrdre::NumeroFaux { numero, .. } => format!(
            "« numero: {numero} » n'est pas sa place dans son monde (numéros 1, 2, 3... sans trou ni doublon)"
        ),
        ErreurOrdre::MondeDesordonne { monde, .. } => {
            format!("monde {monde} inattendu (mondes 1, 2, 3... sans trou)")
        }
        ErreurOrdre::NormalApresLaFin { .. } => {
            "niveau normal après le bonus ou l'épilogue de son monde".into()
        }
        ErreurOrdre::BonusMultiple => "plus d'un bonus dans ce monde".into(),
        ErreurOrdre::EpilogueAvantLaFin { .. } => {
            "épilogue qui n'est pas le dernier niveau de son monde".into()
        }
    }
}
