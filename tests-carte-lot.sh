#!/usr/bin/env bash
# tests-carte-lot.sh : les deux commandes de tests du chantier (carte 9.23, tri des
# tests). Lancé DANS le conteneur par `./docker-cargo.sh carte <clé>` ou
# `./docker-cargo.sh lot` (jamais directement sur la machine).
#
#   carte <clé> : ce qu'une carte de NIVEAU lance (cible : moins de 3 minutes).
#                 Tous les tests rapides (core et game : grilles, décor, physique...),
#                 plus, pour SON niveau seulement : faisabilité (3 gouttes, lanternes,
#                 robot maladroit, durée), reprise depuis chaque lanterne et tests de
#                 lot de son module (`levels::<clé>::`). PAS de robot de QA (décision
#                 d'AGA du 18/11 : aucune carte ne lance de robot).
#   carte <clé> robot [sans-memoire] : la même chose, plus le robot de QA qui finit
#                 SON niveau sans mourir (sur demande seulement : sa recherche dure
#                 9 s au caniveau, 25 min aux égouts). Le parcours trouvé entre dans
#                 la mémoire (plus bas) : le `lot` suivant le relit.
#   lot         : la suite complète du coordinateur, une fois par lot (cible : moins
#                 de 12 minutes). Tous les tests rapides, plus tous les tests marqués
#                 `#[ignore = "lot : ..."]` (robots, faisabilité de chaque niveau...).
#                 Les autres tests ignorés (écriture de scripts, mesures) n'y sont pas.
#                 `lot sans-memoire` : sans la mémoire des parcours du robot (plus bas),
#                 toutes ses recherches refaites (34 minutes).
#
# Les binaires de test sont compilés une fois (profil `tests-rapides`, voir
# Cargo.toml), puis lancés EN MÊME TEMPS (un processus par groupe). Chaque
# processus écrit son compte rendu dans target/<commande>/ ; on n'affiche que les
# résultats, les échecs et le chrono de chaque test lent (> 5 s).
set -u

COMMANDE="$1"
shift
PROFIL=(--profile tests-rapides)
SORTIE="target/$COMMANDE"
mkdir -p "$SORTIE"
rm -f "$SORTIE"/*.txt
# --report-time (durée de chaque test) est une option instable de libtest : elle
# demande RUSTC_BOOTSTRAP au LANCEMENT des binaires (pas à la compilation).
export RUSTC_BOOTSTRAP=1
TEMPS=(-Z unstable-options --report-time)
# Mémoire des parcours du robot de QA (`game/src/bot_qa/parcours.rs`, `sur_disque`),
# partagée par tous les worktrees (le volume cargo de docker-cargo.sh) : un parcours
# déjà cherché pour le MÊME code et le MÊME niveau est relu (puis rejoué dans la
# vraie simulation) au lieu d'être cherché de nouveau (le Monde 2 : 40 min). On peut
# la vider à tout moment : ~/.cache/pepin-cargo-home/parcours-du-robot/.
export PEPIN_MEMOIRE_PARCOURS="$CARGO_HOME/parcours-du-robot"

debut=$(date +%s)
echo "== compilation (profil tests-rapides)"
if ! cargo test -p core -p game "${PROFIL[@]}" --no-default-features --no-run \
  >"$SORTIE/compilation.txt" 2>&1; then
  cat "$SORTIE/compilation.txt"
  exit 1
fi
# « Executable unittests src/lib.rs (target/.../pepin_core-xxxx) » : le chemin.
CORE=$(sed -n 's/.*Executable unittests src\/lib\.rs (\(.*\))$/\1/p' "$SORTIE/compilation.txt")
GAME=$(sed -n 's/.*Executable unittests src\/main\.rs (\(.*\))$/\1/p' "$SORTIE/compilation.txt")
if [ -z "$CORE" ] || [ -z "$GAME" ]; then
  echo "binaires de test introuvables" >&2
  cat "$SORTIE/compilation.txt"
  exit 1
fi
echo "compilé en $(($(date +%s) - debut)) s"

# Noms des tests ignorés dont la raison commence par « lot » (liste JSON de libtest).
tests_de_lot() {
  "$1" --list --ignored --format json -Z unstable-options |
    grep '"ignore_message": "lot' |
    sed 's/.*"name": "\([^"]*\)".*/\1/'
}

# Lance un groupe en arrière-plan : lancer <nom> <binaire> [arguments...]
PIDS=()
NOMS=()
lancer() {
  local nom="$1"
  shift
  "$@" "${TEMPS[@]}" >"$SORTIE/$nom.txt" 2>&1 &
  PIDS+=($!)
  NOMS+=("$nom")
}

case "$COMMANDE" in
  carte)
    CLE="${1:-}"
    if [ -z "$CLE" ] || [ ! -f "core/src/levels/$CLE.rs" ]; then
      echo "usage : ./docker-cargo.sh carte <clé> [robot] [sans-memoire] (clé d'un fichier core/src/levels/<clé>.rs)" >&2
      exit 2
    fi
    shift
    ROBOT=0
    for option in "$@"; do
      case "$option" in
        robot) ROBOT=1 ;;
        sans-memoire)
          unset PEPIN_MEMOIRE_PARCOURS
          echo "mémoire des parcours : NON (recherche du robot refaite)"
          ;;
        *)
          echo "option inconnue : $option (robot, sans-memoire)" >&2
          exit 2
          ;;
      esac
    done
    export CLE
    lancer core-rapides "$CORE"
    lancer game-rapides "$GAME"
    lancer core-niveau "$CORE" --ignored --exact zones::tests::le_niveau_de_la_carte
    lancer core-module "$CORE" --ignored "levels::$CLE::"
    if [ $ROBOT = 1 ]; then
      lancer game-robot "$GAME" --ignored --exact bot_qa::tests::le_robot_finit_le_niveau_de_la_carte
    fi
    ;;
  lot)
    # `lot sans-memoire` : toutes les recherches du robot refaites (voir plus bas).
    if [ "${1:-}" = sans-memoire ]; then
      unset PEPIN_MEMOIRE_PARCOURS
      echo "mémoire des parcours : NON (toutes les recherches du robot refaites)"
    fi
    # Les tests de lot les plus longs d'abord n'est pas possible (libtest les prend
    # dans l'ordre alphabétique) : on lance donc les quatre groupes ensemble.
    mapfile -t LOT_CORE < <(tests_de_lot "$CORE")
    mapfile -t LOT_GAME < <(tests_de_lot "$GAME")
    echo "tests de lot : ${#LOT_CORE[@]} (core) + ${#LOT_GAME[@]} (game)"
    lancer game-lot "$GAME" --ignored --exact "${LOT_GAME[@]}"
    lancer core-lot "$CORE" --ignored --exact "${LOT_CORE[@]}"
    lancer core-rapides "$CORE"
    lancer game-rapides "$GAME"
    ;;
  *)
    echo "commande inconnue : $COMMANDE (carte <clé> ou lot)" >&2
    exit 2
    ;;
esac

ECHEC=0
for k in "${!PIDS[@]}"; do
  if ! wait "${PIDS[$k]}"; then
    ECHEC=1
  fi
  f="$SORTIE/${NOMS[$k]}.txt"
  echo "== ${NOMS[$k]} : $(grep -h '^test result' "$f" | tail -1)"
  # Échecs (avec leur message) et tests lents (> 5 s, chrono de --report-time).
  grep -h -E '^test .* (FAILED|ok) <([5-9]|[1-9][0-9]+)\.[0-9]+s>' "$f" | sed 's/^/   /'
  if grep -q -E '^test .* FAILED' "$f"; then
    sed -n '/^failures:$/,/^test result/p' "$f"
  fi
done
echo "== $COMMANDE : $([ $ECHEC = 0 ] && echo VERT || echo ROUGE), $(($(date +%s) - debut)) s en tout (comptes rendus : $SORTIE/)"
exit $ECHEC
