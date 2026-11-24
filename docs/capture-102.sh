#!/usr/bin/env bash
# Captures de référence de la carte 10.2 (avant / après une carte de rendu) : trois scènes
# du build de QA, reproductibles au pixel près (détails et parcours : capture-102.mjs).
#   PREFIXE-toits.png   : départ du 1er niveau (les toits, Monde 1), carte de titre passée ;
#   PREFIXE-port.png    : le port (Monde 2), Pépin sur le 2e quai, une mouette perchée ;
#   PREFIXE-pieuvre.png : la Pieuvre du port, les tentacules sortent de l'eau.
#
# Usage : CHROME=chemin/de/chromium docs/capture-102.sh PORT_HTTP DOSSIER_SORTIE PREFIXE
# où dist-qa/ (le build de QA à photographier) est servi sur 127.0.0.1:PORT_HTTP, par
# exemple : (cd dist-qa && python3 -m http.server PORT_HTTP --bind 127.0.0.1)
# Node >= 22 (WebSocket intégré) ; variable NODE pour un autre Node (défaut : node).
#
# Comparer deux séries (avant, après) :
#   python3 docs/comparer-captures.py DOSSIER_AVANT/PREFIXE_AVANT DOSSIER_APRES/PREFIXE_APRES
set -eu
if [ $# -ne 3 ]; then
  echo "Usage : CHROME=... $0 PORT_HTTP DOSSIER_SORTIE PREFIXE" >&2
  exit 2
fi
: "${CHROME:?indiquer le chemin de Chromium dans CHROME}"
export CHROME
ici=$(cd "$(dirname "$0")" && pwd)
# Une capture tient en une minute environ ; la limite évite un navigateur bloqué pour toujours.
timeout 400 "${NODE:-node}" "$ici/capture-102.mjs" "$1" "$2" "$3"
