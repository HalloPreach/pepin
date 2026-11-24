#!/usr/bin/env bash
# Captures de référence de la carte 10.4a (les sprites deviennent des fichiers PNG) :
# le jeu doit rester identique AU PIXEL. Deux parcours entièrement déterministes du build
# de QA, pris avec le pilote de `capture-102.mjs` (horloge figée, 1/60 s par image) :
#   PREFIXE-titre.png, PREFIXE-carte.png, PREFIXE-boutique.png : l'écran titre, la carte
#     des niveaux et la boutique ;
#   PREFIXE-n00.png à PREFIXE-n17.png : le départ de chaque niveau de la table (les
#     touches 1 à 9 du build de QA, puis N : un niveau par thème, Monde 1 et Monde 2),
#     carte de titre passée, 60 images au départ ;
#   PREFIXE-pieuvre-a.png à -c.png : la Pieuvre du port (indice 18) à 151 images (les bouts de
#     tentacules sortent de l'eau, phase 1), puis 300 et 900 images plus tard (la
#     montée de la bête et ses frappes ; Pépin reste immobile sur le quai).
#
# Usage : CHROME=chemin/de/chromium docs/capture-104a.sh PORT_HTTP DOSSIER_SORTIE PREFIXE
# (dist-qa/ servi sur 127.0.0.1:PORT_HTTP, Node >= 22, variable NODE pour un autre Node).
# Comparer : python3 docs/comparer-captures.py DOSSIER/AVANT DOSSIER/APRES
set -eu
if [ $# -ne 3 ]; then
  echo "Usage : CHROME=... $0 PORT_HTTP DOSSIER_SORTIE PREFIXE" >&2
  exit 2
fi
: "${CHROME:?indiquer le chemin de Chromium dans CHROME}"
export CHROME
ici=$(cd "$(dirname "$0")" && pwd)
NODE="${NODE:-node}"

# 1. Titre, carte des niveaux (2e ligne du menu) et boutique (3e ligne).
timeout 400 "$NODE" "$ici/capture-102.mjs" "$1" "$2" "$3" \
  w:60 c:titre t:ArrowDown:30 t:Enter:90 c:carte
timeout 400 "$NODE" "$ici/capture-102.mjs" "$1" "$2" "$3" \
  w:60 t:ArrowDown:30 t:ArrowDown:30 t:Enter:90 c:boutique

# 2. Le départ de chaque niveau, puis la Pieuvre.
etapes=(t:Enter:120 t:Enter:60 t:Enter:120 w:60 c:n00)
for i in 2 3 4 5 6 7 8 9; do
  etapes+=("t:Digit$i:60" t:Enter:60 w:60 "c:n0$((i - 1))")
done
for i in 09 10 11 12 13 14 15 16 17; do
  etapes+=(t:KeyN:60 t:Enter:60 w:60 "c:n$i")
done
etapes+=(t:KeyN:60 t:Enter:151 c:pieuvre-a w:300 c:pieuvre-b w:600 c:pieuvre-c)
timeout 600 "$NODE" "$ici/capture-102.mjs" "$1" "$2" "$3" "${etapes[@]}"
