//! Script de construction : sur le web (wasm32), le son vient de fonctions JavaScript
//! (`audio_init`, `audio_play_buffer`...) fournies par `mq_js_bundle.js` au chargement.
//! Le lien doit donc tolérer ces symboles « importés » que Rust ne connaît pas
//! (les versions récentes de rustc ne passent plus `--allow-undefined` d'elles-mêmes).
//! Les fonctions graphiques de miniquad, elles, déclarent leur module d'import.

fn main() {
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32") {
        println!("cargo:rustc-link-arg=--allow-undefined");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
