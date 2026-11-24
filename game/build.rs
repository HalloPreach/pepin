//! Script de construction du jeu.
//!
//! 1. Sur le web (wasm32), le son vient de fonctions JavaScript (`audio_init`,
//!    `audio_play_buffer`...) fournies par `mq_js_bundle.js` au chargement. Le lien
//!    doit donc tolérer ces symboles « importés » que Rust ne connaît pas (les versions
//!    récentes de rustc ne passent plus `--allow-undefined` d'elles-mêmes). Les
//!    fonctions graphiques de miniquad, elles, déclarent leur module d'import.
//! 2. Les sprites (carte 10.4a) : on parcourt `assets/sprites` et on écrit, dans le
//!    dossier de sortie de cargo (`OUT_DIR`), la liste de TOUS les fichiers `.png`
//!    trouvés, chacun embarqué par `include_bytes!` (comme les niveaux du crate
//!    `niveaux`) : ajouter un sprite = ajouter un fichier. Le dossier du fichier est
//!    sa FAMILLE (`ennemis`, `tuiles_monde2/port`...), son nom sans `.png` sa CLÉ.
//! 3. Les fonds (carte 10.4c) : de même pour `assets/fonds/<thème>/<nom>.png` (les
//!    couches FIXES des fonds : ciels, soleil, arc-en-ciel), dans `fonds_embarques.rs`.

use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32") {
        println!("cargo:rustc-link-arg=--allow-undefined");
    }
    println!("cargo:rerun-if-changed=build.rs");

    let manifeste = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let sortie = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    // Un dossier en `rerun-if-changed` : cargo relance ce script si un fichier y est
    // ajouté, retiré ou modifié.
    println!("cargo:rerun-if-changed=../assets/sprites");
    println!("cargo:rerun-if-changed=../assets/fonds");
    embarquer(
        &manifeste.join("../assets/sprites"),
        "/// Sprites de `assets/sprites` : (famille, clé, octets du PNG), embarqués à la\n\
         /// compilation (généré par `build.rs`).\n\
         pub const FICHIERS: &[(&str, &str, &[u8])] = &[\n",
        &sortie.join("sprites_embarques.rs"),
    );
    embarquer(
        &manifeste.join("../assets/fonds"),
        "/// Couches de fond de `assets/fonds` : (thème, nom, octets du PNG), embarquées à\n\
         /// la compilation (généré par `build.rs`).\n\
         pub const FICHIERS: &[(&str, &str, &[u8])] = &[\n",
        &sortie.join("fonds_embarques.rs"),
    );
}

/// Écrit dans `fichier` la constante dont `entete` ouvre le tableau : une ligne
/// (dossier, nom sans `.png`, `include_bytes!`) par PNG trouvé sous `racine`.
fn embarquer(racine: &Path, entete: &str, fichier: &Path) {
    let mut chemins = Vec::new();
    lister(racine, racine, &mut chemins);
    // Ordre STABLE (alphabétique) : la même arborescence donne le même code.
    chemins.sort();
    let mut code = String::from(entete);
    for relatif in &chemins {
        let (famille, nom) = relatif.rsplit_once('/').unwrap_or(("", relatif));
        let cle = nom.trim_end_matches(".png");
        let absolu = racine.join(relatif);
        code.push_str(&format!(
            "    ({famille:?}, {cle:?}, include_bytes!({:?})),\n",
            absolu.to_str().unwrap()
        ));
    }
    code.push_str("];\n");
    fs::write(fichier, code).unwrap();
}

/// Ajoute à `chemins` les fichiers `.png` de `dossier` (récursivement), en chemins
/// relatifs à `racine` séparés par `/`.
fn lister(racine: &Path, dossier: &Path, chemins: &mut Vec<String>) {
    let Ok(entrees) = fs::read_dir(dossier) else {
        return;
    };
    for entree in entrees.flatten() {
        let chemin = entree.path();
        if chemin.is_dir() {
            lister(racine, &chemin, chemins);
        } else if chemin.extension().is_some_and(|e| e == "png") {
            let relatif = chemin.strip_prefix(racine).unwrap();
            let morceaux: Vec<_> = relatif.iter().map(|m| m.to_string_lossy()).collect();
            chemins.push(morceaux.join("/"));
        }
    }
}
