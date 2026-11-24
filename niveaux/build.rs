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

    let mut code = String::from(
        "/// Fichiers `.ron` de `assets/levels`, embarqués à la compilation (généré par `build.rs`).\n\
         pub const FICHIERS: &[Fichier] = &[\n",
    );
    for relatif in &chemins {
        code.push_str(&format!(
            "    Fichier {{ chemin: {relatif:?}, texte: include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/../assets/levels/{relatif}\")) }},\n"
        ));
    }
    code.push_str("];\n");
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
