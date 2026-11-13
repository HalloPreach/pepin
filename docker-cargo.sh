#!/usr/bin/env bash
# Aide : exécute une commande cargo dans un conteneur rust:1-slim jetable.
# Usage : ./docker-cargo.sh test -p core
#         ./docker-cargo.sh build -p game --target wasm32-unknown-unknown
#         ./docker-cargo.sh test -p game --no-default-features   (natif sans libasound : sans son réel)
#         ./docker-cargo.sh fmt -p core         (applique rustfmt)
#         ./docker-cargo.sh lint                (fmt --check + clippy -D warnings, core puis game wasm puis game tests)
# Le conteneur est nommé puis supprimé avec `docker rm` (pas de --rm).
set -u
cd "$(dirname "$0")"

NOM="pepin-cargo-$$"
CARGO_HOME_HOTE="${PEPIN_CARGO_HOME:-${XDG_CACHE_HOME:-$HOME/.cache}/pepin-cargo-home}"
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
# rustfmt et clippy manquent dans l'"'"'image : on les installe (une seule fois)
# dans la copie persistante de rustup.
if [ "$1" = fmt ] || [ "$1" = clippy ] || [ "$1" = lint ]; then
  if ! cargo fmt --version >/dev/null 2>&1 || ! cargo clippy --version >/dev/null 2>&1; then
    rustup component add clippy rustfmt >&2
  fi
fi
if [ "$1" = lint ]; then
  echo "== cargo fmt --check =="
  cargo fmt --all --check
  echo "FMT-OK"
  echo "== clippy core (natif, tests inclus) =="
  cargo clippy -p core --all-targets -- -D warnings
  echo "== clippy game (wasm32) =="
  cargo clippy -p game --target wasm32-unknown-unknown -- -D warnings
  echo "== clippy game (natif, tests inclus, sans son) =="
  cargo clippy -p game --all-targets --no-default-features -- -D warnings
  echo "== clippy game (natif, tests inclus, sans son, avec debug_niveaux) =="
  cargo clippy -p game --all-targets --no-default-features --features debug_niveaux -- -D warnings
  echo "== clippy game (wasm32, avec debug_niveaux) =="
  cargo clippy -p game --target wasm32-unknown-unknown --features debug_niveaux -- -D warnings
  echo "LINT-OK"
  exit 0
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
