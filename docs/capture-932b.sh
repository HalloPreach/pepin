#!/usr/bin/env bash
# Captures de la carte 9.32b (« la carte des niveaux accepte plusieurs niveaux du même
# thème ») : la carte des niveaux du build de QA (tout ouverte, `tout_ouvert_qa`), étape
# par étape, avec le pilote de `capture-102.mjs` (horloge figée, 1/60 s par image).
# La carte s'ouvre au bout du chapitre 2 (la Pieuvre) ; Bas descend d'une étape à la
# fois jusqu'à la rue, puis passe au chapitre 1 (le Monde 1) et le descend jusqu'aux
# toits. Une capture par étape :
#   PREFIXE-carte-NN.png (NN = 00 à 22) : 00 à l'ouverture, puis après chaque Bas.
# Avec la marée (8 niveaux au chapitre 2), le Monde 1 commence à la capture 08 ; sans
# elle (7 niveaux), à la capture 07.
#
# Usage : CHROME=chemin/de/chromium docs/capture-932b.sh PORT_HTTP DOSSIER_SORTIE PREFIXE
# (dist-qa/ servi sur 127.0.0.1:PORT_HTTP, Node >= 22, variable NODE pour un autre Node).
# Comparer : python3 docs/comparer-captures.py AVANT-x.png APRES-x.png ...
set -eu
if [ $# -ne 3 ]; then
  echo "Usage : CHROME=... $0 PORT_HTTP DOSSIER_SORTIE PREFIXE" >&2
  exit 2
fi
: "${CHROME:?indiquer le chemin de Chromium dans CHROME}"
export CHROME
ici=$(cd "$(dirname "$0")" && pwd)
NODE="${NODE:-node}"

# Bas (CARTE au titre), Entrée : la carte s'ouvre ; puis Bas, étape après étape.
etapes=(w:60 t:ArrowDown:10 t:Enter:120 w:60 c:carte-00)
for k in $(seq 1 22); do
  etapes+=(t:ArrowDown:40 w:60 "c:carte-$(printf '%02d' "$k")")
done
timeout 600 "$NODE" "$ici/capture-102.mjs" "$1" "$2" "$3" "${etapes[@]}"
