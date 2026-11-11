#!/usr/bin/env bash
# Aide : exécute une commande cargo dans un conteneur rust:1-slim jetable.
# Usage : ./docker-cargo.sh test -p core
#         ./docker-cargo.sh build -p game --target wasm32-unknown-unknown
# Le conteneur est nommé puis supprimé avec `docker rm` (pas de --rm).
set -u
cd "$(dirname "$0")"

NOM="elsay-cargo-$$"
CARGO_HOME_HOTE="${ELSAY_CARGO_HOME:-$HOME/.hermes/cache/scratch/cargo-home-elsay}"
mkdir -p "$CARGO_HOME_HOTE"

# Dans le conteneur : le RUSTUP_HOME de l'image est en lecture seule pour notre
# uid. On en garde une copie persistante dans le volume cargo-home (copiée une
# seule fois), puis on ajoute la cible wasm32 si elle manque.
SCRIPT_INTERNE='
set -e
if [ ! -d /cargohome/rustup ]; then
  cp -r "$RUSTUP_HOME_IMAGE" /cargohome/rustup
fi
export RUSTUP_HOME=/cargohome/rustup
if ! ls "$RUSTUP_HOME"/toolchains/*/lib/rustlib 2>/dev/null | grep -q wasm32-unknown-unknown; then
  rustup target add wasm32-unknown-unknown >&2
fi
exec cargo "$@"
'

docker run --name "$NOM" --user "$(id -u):$(id -g)" \
  -e CARGO_HOME=/cargohome -e HOME=/tmp \
  -e RUSTUP_HOME_IMAGE=/usr/local/rustup \
  -v "$CARGO_HOME_HOTE":/cargohome \
  -v "$PWD":/work -w /work \
  rust:1-slim bash -c "$SCRIPT_INTERNE" cargo "$@"
CODE=$?
docker rm "$NOM" >/dev/null
exit $CODE
