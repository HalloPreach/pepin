#!/usr/bin/env bash
# build.sh : construit dist/ : le jeu PÉPIN prêt à être servi par n'importe quel
# serveur de fichiers statiques (index.html + game.wasm + mq_js_bundle.js, et les
# sons dans musiques/ et sons/ depuis la carte 10.4e).
#
# Usage :   ./build.sh
#           WASM_OPT=0 ./build.sh     # saute l'optimisation wasm-opt
#           ./build.sh --qa           # build de MISE AU POINT dans dist-qa/ (jamais à publier) :
#                                     # active les fonctionnalités « debug_niveaux » (touches 1 à 9
#                                     # pour sauter à un niveau, 0 pour le niveau
#                                     # d'essai vertical) et « qa_plantage » (F9 provoque un
#                                     # vrai plantage), utilisées par les captures et la QA ;
#                                     # y ajoute l'éditeur de niveaux (carte 10.3a) :
#                                     # editeur.html, editeur/, legende.json, niveaux/
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
#   4. copie web/index.html et les sons (assets/musiques, assets/sons : WAV
#      canoniques, sans nom interdit), et vérifie que dist/ est complet.
#
# Décision sur dist/ : il est IGNORÉ par git (.gitignore). C'est un produit
# régénérable par ce script ; on évite de versionner un binaire de ~400 Ko qui
# changerait à chaque commit. Pour publier, lancer ./build.sh puis copier TOUT le
# contenu de dist/ sur le serveur (sous-dossiers musiques/ et sons/ compris).
set -euo pipefail
cd "$(dirname "$0")"

DIST="dist"
FONCTIONNALITES=""
case "${1:-}" in
  "") ;;
  --qa) DIST="dist-qa"; FONCTIONNALITES="--features debug_niveaux,qa_plantage" ;;
  *) echo "Usage : ./build.sh [--qa]" >&2; exit 2 ;;
esac
CARGO_HOME_HOTE="${PEPIN_CARGO_HOME:-${XDG_CACHE_HOME:-$HOME/.cache}/pepin-cargo-home}"
WASM_BRUT="target/wasm32-unknown-unknown/release/game.wasm"

echo "== 1/4 Compilation release (wasm32-unknown-unknown) dans Docker =="
# Carte 10.1b : les niveaux sont des fichiers `.ron` embarqués. Avant de construire,
# on vérifie qu'ils sont TOUS valides : un fichier invalide arrête le build, le jeu
# publié ne peut donc jamais tomber sur une erreur de niveau.
echo "== Vérification des fichiers de niveau (assets/levels) =="
./docker-cargo.sh test -p niveaux --locked --lib -q tous_les_fichiers_embarques_sont_valides \
  || { echo "ERREUR : un fichier de niveau est invalide (voir le message ci-dessus)" >&2; exit 1; }
# shellcheck disable=SC2086  # FONCTIONNALITES est volontairement vide ou « --features x »
./docker-cargo.sh build -p game --release --locked --target wasm32-unknown-unknown $FONCTIONNALITES
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

echo "== 4/4 Page web, sons et vérifications =="
cp web/index.html "$DIST/index.html"

# Carte 10.4e : les musiques et les bruitages sont des fichiers WAV servis À CÔTÉ du
# jeu (`musiques/<id>.wav`, `sons/<clé>.wav`), téléchargés par le jeu après l'écran
# titre. Copiés TELS QUELS de `assets` ; chacun doit être un WAV canonique (en-tête
# de 44 octets : RIFF, fmt, data, rien d'autre, donc aucune métadonnée) et ne
# contenir aucun nom interdit.
for genre in musiques sons; do
  mkdir -p "$DIST/$genre"
  cp assets/"$genre"/*.wav "$DIST/$genre/"
done
SONS_N=0
SONS_OCTETS=0
SONS_GZIP=0
for f in "$DIST"/musiques/*.wav "$DIST"/sons/*.wav; do
  [ "$(head -c 4 "$f")" = "RIFF" ] && [ "$(dd if="$f" bs=1 skip=8 count=8 2>/dev/null)" = "WAVEfmt " ] \
    && [ "$(dd if="$f" bs=1 skip=36 count=4 2>/dev/null)" = "data" ] \
    || { echo "ERREUR : $f n'est pas un WAV canonique (RIFF, fmt, data)" >&2; exit 1; }
  if grep -qaiE 'lucas|tixier|elsay|georges|jarvis|/home/' "$f"; then
    echo "ERREUR : nom interdit dans $f" >&2; exit 1
  fi
  chmod 644 "$f"
  SONS_N=$((SONS_N + 1))
  SONS_OCTETS=$((SONS_OCTETS + $(wc -c < "$f")))
  SONS_GZIP=$((SONS_GZIP + $(gzip -9 -c "$f" | wc -c)))
done
[ "$SONS_N" -ge 19 ] || { echo "ERREUR : $SONS_N fichiers son dans $DIST (19 attendus au moins)" >&2; exit 1; }
# Plafond de la carte 10.4e : 3,5 Mo de sons au plus.
[ "$SONS_OCTETS" -le 3500000 ] || { echo "ERREUR : $SONS_OCTETS octets de sons (plafond 3,5 Mo)" >&2; exit 1; }
echo "sons : $SONS_N WAV canoniques, aucun nom interdit : OK"

# Carte 10.3a : l'éditeur de niveaux (editeur/) n'existe QUE dans le build de QA
# (site de test). Il y trouve `legende.json` (les lettres de la grille, écrites par
# le crate `niveaux` : la seule source) et une copie des `.ron` dans `niveaux/`.
# Carte 10.4b : et les PNG des vignettes de la palette, copiés de `assets/sprites`
# dans `sprites/<famille>/<clé>.png` (`--bin editeur`, d'après `legende.json`).
if [ "$DIST" = "dist-qa" ]; then
  cp editeur/editeur.html "$DIST/editeur.html"
  mkdir -p "$DIST/editeur"
  cp editeur/editeur.js editeur/editeur.css editeur/ron.js editeur/robot-worker.js "$DIST/editeur/"
  # Carte 10.3c : le robot de faisabilité de `core`, compilé en wasm (crate
  # `robot`, profil `robot-web` : rapide plutôt que petit), pour le bouton ROBOT.
  ./docker-cargo.sh build -p robot --locked --profile robot-web --target wasm32-unknown-unknown
  cp target/wasm32-unknown-unknown/robot-web/robot.wasm "$DIST/robot.wasm"
  chmod 644 "$DIST/robot.wasm"
  ./docker-cargo.sh run -p niveaux --locked -q --bin editeur -- "$DIST"
  if command -v node >/dev/null 2>&1; then
    node editeur/test-aller-retour.mjs
  else
    echo "AVERTISSEMENT : node absent, test aller-retour de l'éditeur sauté" >&2
  fi
fi
# Le jeu publié ne contient JAMAIS l'éditeur ni ses données.
if [ "$DIST" = "dist" ]; then
  for f in editeur.html editeur legende.json niveaux robot.wasm sprites; do
    [ ! -e "$DIST/$f" ] || { echo "ERREUR : $DIST/$f présent : l'éditeur ne va jamais en prod" >&2; exit 1; }
  done
  echo "éditeur absent de dist/ : OK"
fi

for f in index.html game.wasm mq_js_bundle.js; do
  [ -s "$DIST/$f" ] || { echo "ERREUR : $DIST/$f manquant ou vide" >&2; exit 1; }
done
# Un wasm valide commence par l'octet nul puis « asm ».
[ "$(head -c 4 "$DIST/game.wasm" | od -An -tx1 | tr -d ' \n')" = "0061736d" ] \
  || { echo "ERREUR : game.wasm n'est pas un module WebAssembly" >&2; exit 1; }

# Carte portfolio : même contrôle des noms interdits que pour les sons, sur la page, le
# chargeur et le wasm. Seule exception, et seulement dans index.html : le lien voulu par
# l'auteur, texte exact « Portfolio de Lucas Tixier » et URL exacte
# « https://portfolio.elsay.dev/ ». On retire ces deux chaînes exactes avant de
# chercher : toute autre occurrence (autre texte, autre URL, chemin /home/…) échoue.
LIEN_TEXTE="Portfolio de Lucas Tixier"
LIEN_URL="https://portfolio.elsay.dev/"
for f in index.html mq_js_bundle.js game.wasm; do
  # grep -c lit tout (pas de -q : avec pipefail, un SIGPIPE masquerait une détection).
  if [ "$f" = index.html ]; then
    N_INTERDITS="$(LC_ALL=C sed -e "s#$LIEN_TEXTE##g" -e "s#https://portfolio\.elsay\.dev/##g" "$DIST/$f" \
      | LC_ALL=C grep -caiE 'lucas|tixier|elsay|georges|jarvis|/home/' || true)"
  else
    N_INTERDITS="$(LC_ALL=C grep -caiE 'lucas|tixier|elsay|georges|jarvis|/home/' "$DIST/$f" || true)"
  fi
  [ "$N_INTERDITS" = 0 ] || { echo "ERREUR : nom interdit dans $DIST/$f" >&2; exit 1; }
done
grep -qF ">$LIEN_TEXTE</a>" "$DIST/index.html" && grep -qF "href=\"$LIEN_URL\"" "$DIST/index.html" \
  || { echo "ERREUR : lien « $LIEN_TEXTE » vers $LIEN_URL absent de $DIST/index.html" >&2; exit 1; }
echo "page, chargeur et wasm : aucun nom interdit hors du lien portfolio : OK"

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
printf '  %-18s %8s octets   (gzip : %s, %s fichiers)\n' "musiques/ sons/" "$SONS_OCTETS" "$SONS_GZIP" "$SONS_N"
TOTAL=$((TOTAL + SONS_OCTETS))
WASM_N="$(octets "$DIST/game.wasm")"
printf '  %-18s %8s octets\n' "TOTAL" "$TOTAL"
# Budget : < 2,5 Mo exigé, < 1,2 Mo visé.
if [ "$WASM_N" -ge 2500000 ]; then
  echo "ERREUR : game.wasm dépasse 2,5 Mo" >&2; exit 1
elif [ "$WASM_N" -ge 1200000 ]; then
  echo "AVERTISSEMENT : game.wasm dépasse l'objectif de 1,2 Mo" >&2
fi
echo "Servir en local :  cd $DIST && python3 -m http.server 8080"
