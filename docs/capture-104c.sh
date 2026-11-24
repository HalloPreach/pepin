#!/usr/bin/env bash
# Captures de référence de la carte 10.4c (les fonds du Monde 1 deviennent des fichiers
# PNG, `assets/fonds/`) : le jeu doit rester identique AU PIXEL. Parcours entièrement
# déterministes du build de QA, pris avec le pilote de `capture-102.mjs` (horloge figée,
# 1/60 s par image) :
#   PREFIXE-titre.png, PREFIXE-carte.png, PREFIXE-boutique.png : l'écran titre (le ciel
#     des toits qui défile), la carte des niveaux et la boutique ;
#   PREFIXE-mNN-depart.png (NN = 00 à 11) : le départ des 12 niveaux du Monde 1 (touches
#     1 à 9 du build de QA, puis N ; 09 la poursuite, 10 l'arc-en-ciel, 11 la descente),
#     60 images après la carte de titre ;
#   PREFIXE-mNN-milieu.png (NN = 00 à 10) : le même niveau après 150 images de course vers la droite
#     (la caméra a avancé : la parallaxe des couches a bougé ; une chute éventuelle ne
#     gêne pas, le parcours reste le même à chaque passage) ;
#   PREFIXE-m11-a.png à -d.png : la descente (niveau vertical) vue à quatre moments, Pépin
#     courant d'un trou à l'autre : le ciel du voyage remonté, du corail au violet (le
#     parcours s'arrête au 2e étage, avant la ville du bas ; peu importe, il est le même
#     à chaque passage).
# À chaque capture, des couches CALCULÉES (étoiles, pluie, plafond, silhouettes) passent
# devant les couches PNG (ciel, soleil, arc-en-ciel) et derrière d'autres.
#
# Usage : CHROME=chemin/de/chromium docs/capture-104c.sh PORT_HTTP DOSSIER_SORTIE PREFIXE
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

# 2. Départ et milieu des 11 premiers niveaux du Monde 1 (la 4e Entrée passe la carte de
#    titre des toits), puis la descente à quatre hauteurs.
etapes=(t:Enter:120 t:Enter:60 t:Enter:120 t:Enter:60 w:60 c:m00-depart m:4:150 c:m00-milieu)
for i in 2 3 4 5 6 7 8 9; do
  etapes+=("t:Digit$i:60" t:Enter:60 w:60 "c:m0$((i - 1))-depart" m:4:150 "c:m0$((i - 1))-milieu")
done
for i in 09 10; do
  etapes+=(t:KeyN:60 t:Enter:60 w:60 "c:m$i-depart" m:4:150 "c:m$i-milieu")
done
etapes+=(t:KeyN:60 t:Enter:60 w:60 c:m11-depart
  m:4:240 c:m11-a m:2:300 c:m11-b m:4:300 c:m11-c m:2:400 c:m11-d)
timeout 600 "$NODE" "$ici/capture-102.mjs" "$1" "$2" "$3" "${etapes[@]}"
