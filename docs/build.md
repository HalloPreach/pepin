# Construire et publier PÉPIN

## Une seule commande

    ./build.sh

Prérequis : Docker (rien d'autre : ni Rust, ni compilateur C sur la machine).
Le script produit `dist/`, un dossier AUTONOME et statique :

    dist/index.html        page (canvas plein écran, écran de chargement, messages d'erreur)
    dist/mq_js_bundle.js   chargeur JavaScript de macroquad (même version que Cargo.lock)
    dist/game.wasm         le jeu

Aucune requête externe : ni CDN, ni police web, ni lien. N'importe quel serveur de
fichiers statiques convient (le type MIME `application/wasm` pour `.wasm` est
reconnu par tous les serveurs courants ; sinon le chargement continue à marcher,
seul le démarrage en streaming est désactivé par certains navigateurs).

Essai en local :

    cd dist && python3 -m http.server 8080      # puis http://127.0.0.1:8080/

## Build de mise au point

    ./build.sh --qa

produit `dist-qa/` (ignoré par git) avec les fonctionnalités Cargo `debug_niveaux` et
`qa_plantage`. Avec `debug_niveaux`, les touches 1 à 5 sautent au niveau demandé. Elles
n'existent PAS dans `dist/` : sur AZERTY ce sont les touches & é " ' (, et un appui
accidentel faisait perdre la partie (et permettait de finir avec un temps truqué).
Avec `qa_plantage`, F9 provoque une vraie panique Rust (« RuntimeError: unreachable »
dans le navigateur) pour vérifier le message de plantage de la page
(`docs/qa-focus-plantage.mjs plantage`) ; le texte de la panique est absent du wasm de
production. `docs/capture-fond.mjs` et `docs/qa-clavier-ecran.mjs` utilisent ce build ;
`./docker-cargo.sh lint` vérifie les deux variantes.

## Ce que fait build.sh

1. `./docker-cargo.sh build -p game --release --locked --target wasm32-unknown-unknown`
   dans un conteneur `rust:1-slim`. Le profil release du `Cargo.toml` vise la
   taille : `opt-level = "z"`, LTO, `strip`, `codegen-units = 1`, `panic = "abort"`.
2. Copie `mq_js_bundle.js` depuis la bibliothèque macroquad téléchargée par cargo
   (donc toujours de la bonne version) et en retire le plugin réseau `quad_net`,
   inutile : le jeu n'envoie ni ne reçoit rien. Cela supprime aussi une erreur
   console (`register_plugin is not defined`) et le code WebSocket/XHR.
   Le son (plugin `macroquad_audio`) est conservé.
3. `wasm-opt -Oz` (paquet binaryen, installé dans un conteneur jetable). Si cette
   étape échoue, le wasm non optimisé est gardé ; `WASM_OPT=0 ./build.sh` la saute.
4. Copie `web/index.html` (la SOURCE de la page est `web/index.html`, pas `dist/`)
   et vérifie que les trois fichiers existent, que le wasm est valide, et que sa
   taille respecte le budget (échec > 2,5 Mo, avertissement > 1,2 Mo).

## Poids obtenu (mesuré le 12/11/2026)

    fichier              octets     gzip
    index.html             7 397    3 140
    mq_js_bundle.js       36 148    9 839
    game.wasm            422 556  172 990
    TOTAL                466 101  ~186 000

Le wasm pèse environ 0,42 Mo (objectif < 1,2 Mo, limite 2,5 Mo). Un serveur qui
compresse en gzip/brotli transfère donc environ 190 Ko.

## dist/ n'est pas versionné

`dist/` est dans `.gitignore` : c'est un produit régénérable par `./build.sh`, et
un binaire de 400 Ko qui change à chaque commit alourdirait inutilement l'historique.
Pour publier : `./build.sh`, puis copier le contenu de `dist/` sur le serveur.
