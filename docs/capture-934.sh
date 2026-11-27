#!/usr/bin/env bash
# Captures de la carte 9.34 (la bannière « TROP DUR ? » nomme le bon boss) : sur le build
# de QA (carte des niveaux tout ouverte), avec le pilote de `capture-102.mjs` (horloge
# figée, 1/60 s par image) et la sauvegarde `docs/qa/934-sauvegarde.txt` : la Pieuvre et
# le Grand Nuage vaincus en FACILE seulement, donc JOUER propose le facile d'emblée.
# La carte s'ouvre sur la Pieuvre (le bout du chapitre 2) :
#   PREFIXE-carte-pieuvre.png, puis Entrée : PREFIXE-banniere-pieuvre.png ; Échap ;
#   14 fois Bas (8 étapes du chapitre 2, puis descente, arc, poursuite, nuages, mer,
#   vents) : PREFIXE-carte-cyclone.png, puis Entrée : PREFIXE-banniere-cyclone.png.
#
# Usage : CHROME=chemin/de/chromium docs/capture-934.sh PORT_HTTP DOSSIER_SORTIE PREFIXE
# (dist-qa/ servi sur 127.0.0.1:PORT_HTTP). Comparer : python3 docs/comparer-captures.py.
set -eu
if [ $# -ne 3 ]; then
  echo "Usage : CHROME=... $0 PORT_HTTP DOSSIER_SORTIE PREFIXE" >&2
  exit 2
fi
: "${CHROME:?indiquer le chemin de Chromium dans CHROME}"
export CHROME
ici=$(cd "$(dirname "$0")" && pwd)
NODE="${NODE:-node}"
export SAUVEGARDE="$ici/qa/934-sauvegarde.txt"
etapes=(w:60 t:ArrowDown:10 t:Enter:120 w:60 c:carte-pieuvre t:Enter:30 c:banniere-pieuvre t:Escape:30)
for _ in $(seq 1 14); do
  etapes+=(t:ArrowDown:40)
done
etapes+=(w:30 c:carte-cyclone t:Enter:30 c:banniere-cyclone t:Escape:30)
timeout 600 "$NODE" "$ici/capture-102.mjs" "$1" "$2" "$3" "${etapes[@]}"
