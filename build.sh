#!/usr/bin/env bash
# build.sh — construit dist/ : le jeu PÉPIN prêt à être servi par n'importe quel
# serveur de fichiers statiques (index.html + game.wasm + mq_js_bundle.js).
#
# Usage :   ./build.sh
#           WASM_OPT=0 ./build.sh     # saute l'optimisation wasm-opt
#
# Prérequis : Docker (aucun compilateur ni Rust nécessaire sur la machine hôte).
# Le script :
#   1. compile le jeu en release (opt-level "z", LTO, strip : voir [profile.release]
#      du Cargo.toml) pour wasm32-unknown-unknown, dans un conteneur rust:1-slim
#      (via ./docker-cargo.sh) ;
#   2. récupère le chargeur JavaScript de macroquad (mq_js_bundle.js) de la MÊME
#      version que celle du Cargo.lock, et en retire le plugin réseau « quad_net »
#      (inutile ici : le jeu ne fait aucune requête, et ce plugin provoquait une
#      erreur console « register_plugin is not defined ») ;
#   3. réduit encore le wasm avec wasm-opt -Oz (paquet binaryen, installé dans un
#      conteneur jetable). Si cette étape échoue, on garde le wasm non optimisé ;
#   4. copie web/index.html et vérifie que dist/ est complet.
#
# Décision sur dist/ : il est IGNORÉ par git (.gitignore). C'est un produit
# régénérable par ce script ; on évite de versionner un binaire de ~400 Ko qui
# changerait à chaque commit. Pour publier, lancer ./build.sh puis copier le
# contenu de dist/ sur le serveur.
set -euo pipefail
cd "$(dirname "$0")"

DIST="dist"
CARGO_HOME_HOTE="${PEPIN_CARGO_HOME:-${XDG_CACHE_HOME:-$HOME/.cache}/pepin-cargo-home}"
WASM_BRUT="target/wasm32-unknown-unknown/release/game.wasm"

echo "== 1/4 Compilation release (wasm32-unknown-unknown) dans Docker =="
./docker-cargo.sh build -p game --release --locked --target wasm32-unknown-unknown
[ -s "$WASM_BRUT" ] || { echo "ERREUR : $WASM_BRUT absent après la compilation" >&2; exit 1; }

echo "== 2/4 Chargeur JavaScript de macroquad =="
VERSION_MQ="$(awk '/^name = "macroquad"$/ {getline; gsub(/[^0-9.]/, "", $0); print; exit}' Cargo.lock)"
[ -n "$VERSION_MQ" ] || { echo "ERREUR : macroquad introuvable dans Cargo.lock" >&2; exit 1; }
BUNDLE_SRC=""
for f in "$CARGO_HOME_HOTE"/registry/src/*/macroquad-"$VERSION_MQ"/js/mq_js_bundle.js; do
  if [ -f "$f" ]; then BUNDLE_SRC="$f"; break; fi
done
[ -n "$BUNDLE_SRC" ] || { echo "ERREUR : mq_js_bundle.js de macroquad $VERSION_MQ introuvable dans $CARGO_HOME_HOTE" >&2; exit 1; }
echo "macroquad $VERSION_MQ : $BUNDLE_SRC"

rm -rf "$DIST"
mkdir -p "$DIST"

# Le plugin quad_net est le dernier bloc du fichier (il commence par
# « ,function(){function i(){}register_plugin= » et va jusqu'à la fin de la ligne).
if grep -q ',function(){function i(){}register_plugin=' "$BUNDLE_SRC"; then
  sed 's/,function(){function i(){}register_plugin=.*$/;/' "$BUNDLE_SRC" > "$DIST/mq_js_bundle.js"
  if grep -q 'quad_net\|WebSocket' "$DIST/mq_js_bundle.js"; then
    echo "AVERTISSEMENT : du code réseau reste dans mq_js_bundle.js" >&2
  fi
  echo "plugin quad_net retiré"
else
  cp "$BUNDLE_SRC" "$DIST/mq_js_bundle.js"
  echo "AVERTISSEMENT : plugin quad_net non repéré (autre version de macroquad ?), bundle copié tel quel" >&2
fi

echo "== 3/4 Optimisation wasm-opt =="
cp "$WASM_BRUT" "$DIST/game.wasm"
if [ "${WASM_OPT:-1}" = "0" ]; then
  echo "wasm-opt sauté (WASM_OPT=0)"
else
  NOM="pepin-wasmopt-$$"
  # Conteneur jetable en root (apt-get exige root) ; le résultat est rendu à l'uid hôte.
  # Le wasm « strippé » n'a plus la section target_features : on active donc à la
  # main les extensions que rustc utilise par défaut.
  if docker run --name "$NOM" \
      -e DEBIAN_FRONTEND=noninteractive -e HOST_UID="$(id -u)" -e HOST_GID="$(id -g)" \
      -v "$PWD/$DIST":/d -w /d rust:1-slim sh -c '
        set -e
        apt-get update -qq >/dev/null
        apt-get install -y -qq --no-install-recommends binaryen >/dev/null
        wasm-opt -Oz \
          --enable-bulk-memory --enable-sign-ext --enable-nontrapping-float-to-int \
          --enable-mutable-globals --enable-multivalue --enable-reference-types \
          game.wasm -o game.opt.wasm
        mv game.opt.wasm game.wasm
        chown "$HOST_UID:$HOST_GID" game.wasm
        echo "$(wasm-opt --version)"
      '; then
    :
  else
    echo "AVERTISSEMENT : wasm-opt a échoué, on garde le wasm non optimisé" >&2
    rm -f "$DIST/game.opt.wasm"
    cp "$WASM_BRUT" "$DIST/game.wasm"
  fi
  docker rm "$NOM" >/dev/null 2>&1 || true
fi
chmod 644 "$DIST/game.wasm"

echo "== 4/4 Page web et vérifications =="
cp web/index.html "$DIST/index.html"

for f in index.html game.wasm mq_js_bundle.js; do
  [ -s "$DIST/$f" ] || { echo "ERREUR : $DIST/$f manquant ou vide" >&2; exit 1; }
done
# Un wasm valide commence par l'octet nul puis « asm ».
[ "$(head -c 4 "$DIST/game.wasm" | od -An -tx1 | tr -d ' \n')" = "0061736d" ] \
  || { echo "ERREUR : game.wasm n'est pas un module WebAssembly" >&2; exit 1; }

octets() { wc -c < "$1" | tr -d ' '; }
gzip_octets() { gzip -9 -c "$1" | wc -c | tr -d ' '; }
TOTAL=0
echo
echo "dist/ prêt :"
for f in index.html mq_js_bundle.js game.wasm; do
  n="$(octets "$DIST/$f")"; g="$(gzip_octets "$DIST/$f")"
  printf '  %-18s %8s octets   (gzip : %s)\n' "$f" "$n" "$g"
  TOTAL=$((TOTAL + n))
done
WASM_N="$(octets "$DIST/game.wasm")"
printf '  %-18s %8s octets\n' "TOTAL" "$TOTAL"
# Budget : < 2,5 Mo exigé, < 1,2 Mo visé.
if [ "$WASM_N" -ge 2500000 ]; then
  echo "ERREUR : game.wasm dépasse 2,5 Mo" >&2; exit 1
elif [ "$WASM_N" -ge 1200000 ]; then
  echo "AVERTISSEMENT : game.wasm dépasse l'objectif de 1,2 Mo" >&2
fi
echo "Servir en local :  cd $DIST && python3 -m http.server 8080"
