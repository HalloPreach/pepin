//! Script de construction du crate `niveaux` : il parcourt `assets/levels` et
//! écrit, dans le dossier de sortie de cargo (`OUT_DIR`), la liste de TOUS les
//! fichiers `.ron` trouvés, chacun embarqué par `include_str!`.
//!
//! Ajouter un niveau = ajouter un fichier : aucune liste à tenir à la main. Le
//! binaire contient le texte des fichiers (la version web n'a rien à télécharger).
//! L'ORDRE du parcours n'est pas décidé ici : c'est le chargeur qui range les
//! niveaux d'après leurs champs `monde` et `numero`.
//!
//! Notions Rust : un `build.rs` est un petit programme lancé par cargo AVANT la
//! compilation du crate ; ce qu'il écrit sur la sortie standard (`cargo:...`)
//! parle à cargo, et le fichier généré est inclus par `include!` dans `lib.rs`.

use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifeste = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let racine = manifeste.join("../assets/levels");
    // Un dossier en `rerun-if-changed` : cargo relance ce script si un fichier y
    // est ajouté, retiré ou modifié (il regarde tout le dossier).
    println!("cargo:rerun-if-changed=../assets/levels");
    println!("cargo:rerun-if-changed=build.rs");

    let mut chemins = Vec::new();
    lister(&racine, &racine, &mut chemins);
    // Ordre STABLE (alphabétique) : la même arborescence donne le même code.
    chemins.sort();

    // Deux listes : le PARCOURS (et tout dossier autre que `essais/`, que le
    // chargeur refusera avec un message clair) et les niveaux d'ESSAI. Celle des
    // essais n'est lue que par le build de QA et les tests (fonctionnalité
    // `essais`) : une constante jamais utilisée n'est pas mise dans le binaire,
    // donc le jeu publié ne contient pas les niveaux d'essai (comme avant 10.1b).
    let (essais, parcours): (Vec<&String>, Vec<&String>) =
        chemins.iter().partition(|c| c.starts_with("essais/"));
    let mut code = String::new();
    for (nom, liste, doc) in [
        ("FICHIERS", &parcours, "du parcours"),
        ("FICHIERS_ESSAIS", &essais, "d'essai (`essais/`)"),
    ] {
        code.push_str(&format!(
            "/// Fichiers `.ron` {doc} de `assets/levels`, embarqués à la compilation (généré par `build.rs`).\n\
             pub const {nom}: &[Fichier] = &[\n"
        ));
        for relatif in liste {
            code.push_str(&format!(
                "    Fichier {{ chemin: {relatif:?}, texte: include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/../assets/levels/{relatif}\")) }},\n"
            ));
        }
        code.push_str("];\n");
    }

    // Les NOMBRES de niveaux, comptés ici, à la compilation, d'après les DOSSIERS
    // (le chargeur vérifie qu'un fichier de `monde_1/` dit bien `monde: 1`). Ce
    // sont de vraies constantes : le jeu s'en sert comme taille de tableau
    // (`[Intro; NB_NIVEAUX_MONDE_1]`), donc un niveau ajouté sans sa ligne dans un
    // tel tableau empêche de compiler, comme avant la carte 10.1b.
    let compter = |prefixe: &str| chemins.iter().filter(|c| c.starts_with(prefixe)).count();
    // Les CLÉS des niveaux du parcours hors Monde 1 (le nom du fichier : le
    // chargeur vérifie qu'il est égal au champ `cle`), pour les vérifications
    // `const` du jeu (chaque niveau du Monde 2 a son intro).
    let cles_hors_monde_1: Vec<&str> = chemins
        .iter()
        .filter(|c| c.starts_with("monde_") && !c.starts_with("monde_1/"))
        .filter_map(|c| c.rsplit('/').next()?.strip_suffix(".ron"))
        .collect();
    code.push_str(&format!(
        "/// Clés des niveaux du parcours hors Monde 1 (ordre alphabétique des chemins).\n\
         pub const CLES_HORS_MONDE_1: &[&str] = &{cles_hors_monde_1:?};\n"
    ));
    code.push_str(&format!(
        "/// Nombre de niveaux du parcours (fichiers sous `monde_*/`), tous mondes.\n\
         pub const NB_NIVEAUX: usize = {};\n\
         /// Nombre de niveaux du Monde 1 (fichiers sous `monde_1/`) : ils sont en tête du parcours.\n\
         pub const NB_NIVEAUX_MONDE_1: usize = {};\n\
         /// Nombre de niveaux d'essai (fichiers sous `essais/`).\n\
         pub const NB_ESSAIS: usize = {};\n",
        compter("monde_"),
        compter("monde_1/"),
        compter("essais/"),
    ));
    let sortie = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("fichiers.rs");
    fs::write(sortie, code).unwrap();
}

/// Ajoute à `chemins` chaque fichier `.ron` sous `dossier`, en chemin relatif à
/// `racine` avec des `/` (le même sur tous les systèmes).
fn lister(racine: &Path, dossier: &Path, chemins: &mut Vec<String>) {
    let Ok(entrees) = fs::read_dir(dossier) else {
        return;
    };
    for entree in entrees.flatten() {
        let chemin = entree.path();
        if chemin.is_dir() {
            lister(racine, &chemin, chemins);
        } else if chemin.extension().is_some_and(|e| e == "ron") {
            let relatif = chemin.strip_prefix(racine).unwrap();
            let morceaux: Vec<String> = relatif
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect();
            chemins.push(morceaux.join("/"));
        }
    }
}
