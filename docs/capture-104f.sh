#!/usr/bin/env bash
# Captures de référence de la carte 10.4f (la grande chute de la cascade et la vallée
# de la rivière entrent dans la pile du fond ; le ciel, le soleil et le moulin de la
# vallée deviennent des fichiers PNG, `assets/fonds/riviere`) : le jeu doit rester
# identique AU PIXEL. Même pilote que `capture-104d.sh` (`capture-102.mjs`, horloge
# figée, 1/60 s par image), qui reste à lancer pour le départ et le milieu des 7
# niveaux du Monde 2. Celui-ci va PLUS LOIN dans les deux niveaux concernés :
#   PREFIXE-cascadeK.png, PREFIXE-grotteK.png (K = 1 à 6) et PREFIXE-riviereK.png
#     (K = 1 à 8) : Pépin court
#     et saute (4 sauts entre deux captures) ; la caméra avance et monte ou descend
#     (parallaxe verticale de la vallée, bas du rideau de la chute).
# À chaque capture de la rivière, des couches CALCULÉES passent ENTRE des couches
# PNG : le halo qui respire entre le ciel et le disque du soleil, les nuages fins, les
# oiseaux, les montagnes, les collines et la rivière du fond entre le soleil et le
# moulin, la roue qui tourne devant le moulin ; à la cascade, le rideau (filets qui
# dévalent) et la brume entre le fond générique et le niveau ; à la grotte, les
# plafonds et les silhouettes (semés au hasard) entre le ciel et les cristaux PNG.
#
# Usage : CHROME=chemin/de/chromium docs/capture-104f.sh PORT_HTTP DOSSIER_SORTIE PREFIXE
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

# `nom:paires:captures` : touche 9 (indice 8), puis `paires` fois N et Entrée jusqu'au
# niveau voulu (6 : la cascade, indice 14 ; 7 : la grotte, 15 ; 8 : la rivière, 16).
for niveau in cascade:6:6 grotte:7:6 riviere:8:8; do
  nom=${niveau%%:*}; reste=${niveau#*:}; paires=${reste%%:*}; captures=${reste#*:}
  etapes=(t:Enter:120 t:Enter:60 t:Enter:120 t:Enter:60 w:60 t:Digit9:60 t:Enter:60)
  for i in $(seq "$paires"); do
    etapes+=(t:KeyN:60 t:Enter:60)
  done
  etapes+=(w:60)
  for k in $(seq "$captures"); do
    for j in 1 2 3 4; do etapes+=(m:12:24 m:4:36); done
    etapes+=("c:$nom$k")
  done
  timeout 900 "$NODE" "$ici/capture-102.mjs" "$1" "$2" "$3" "${etapes[@]}"
done
