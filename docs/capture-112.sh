#!/usr/bin/env bash
# Captures de référence de la carte 11.2 (« plus aucun monde 2 écrit en dur ») : le
# corbeau se choisit par le THÈME du niveau et le fond de la carte des niveaux par les
# thèmes du monde, plus par le numéro du monde. Les Mondes 1 et 2 doivent rester
# identiques AU PIXEL. Parcours déterministes du build de QA, pris avec le pilote de
# `capture-102.mjs` (horloge figée, 1/60 s par image) :
#   PREFIXE-nNN-depart.png (NN = 00 à 18) : le départ de chaque niveau des Mondes 1 et 2,
#     60 images après la carte de titre (N du build de QA pour passer au suivant) ;
#   PREFIXE-nNN-milieu.png : le même niveau après 150 images de course vers la droite
#     (corbeaux et chauves-souris en vol ; une chute éventuelle ne gêne pas, le parcours
#     reste le même à chaque passage) ;
#   PREFIXE-carte-*.png : la carte des niveaux (tout ouverte au build de QA) : elle
#     s'ouvre au bout du chapitre 2 ; Bas parcourt ses étapes jusqu'à la rue, puis passe
#     au chapitre 1 (son dernier niveau) et le descend jusqu'aux toits : les deux fonds,
#     descente et ascension.
#
# Usage : CHROME=chemin/de/chromium docs/capture-112.sh PORT_HTTP DOSSIER_SORTIE PREFIXE
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

# Les 19 niveaux : JOUER (les toits), puis N jusqu'à la Pieuvre (indice 18).
etapes=(t:Enter:120 t:Enter:60 t:Enter:120 t:Enter:60 w:60 c:n00-depart m:4:150 c:n00-milieu)
for i in $(seq 1 18); do
  nn=$(printf '%02d' "$i")
  etapes+=(t:KeyN:60 t:Enter:60 w:60 "c:n$nn-depart" m:4:150 "c:n$nn-milieu")
done
timeout 900 "$NODE" "$ici/capture-102.mjs" "$1" "$2" "$3" "${etapes[@]}"

# La carte des niveaux : Bas (CARTE au titre), Entrée ; puis Bas, étape après étape.
etapes=(w:60 t:ArrowDown:10 t:Enter:120 c:carte-a)
for k in 1 2 3; do etapes+=(t:ArrowDown:40); done
etapes+=(w:60 c:carte-b)
for k in 1 2 3; do etapes+=(t:ArrowDown:40); done
etapes+=(w:60 c:carte-c)
etapes+=(t:ArrowDown:40 w:60 c:carte-d)
for k in 1 2 3 4 5 6; do etapes+=(t:ArrowDown:40); done
etapes+=(w:60 c:carte-e)
for k in 1 2 3 4 5; do etapes+=(t:ArrowDown:40); done
etapes+=(w:60 c:carte-f)
timeout 300 "$NODE" "$ici/capture-102.mjs" "$1" "$2" "$3" "${etapes[@]}"
