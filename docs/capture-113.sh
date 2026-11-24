#!/usr/bin/env bash
# Captures de référence de la carte 11.3 (« le texte d'intro et l'astuce dans le
# fichier du niveau ») : la carte de présentation de CHAQUE niveau des Mondes 1 et 2,
# dont la phrase et le conseil viennent maintenant de `intro_texte` et `intro_astuce`
# du fichier `.ron` au lieu des listes `INTROS` et `INTROS_MONDE_2` du code. Elles
# doivent rester identiques AU PIXEL. Parcours déterministe du build de QA, pris avec
# le pilote de `capture-102.mjs` (horloge figée, 1/60 s par image) :
#   PREFIXE-nNN-carte.png (NN = 00 à 18) : la carte du niveau, 60 images après son
#     ouverture (l'invite « continuer » déjà affichée) ; Entrée la passe, N (build de
#     QA) ouvre la carte du niveau suivant.
#
# Usage : CHROME=chemin/de/chromium docs/capture-113.sh PORT_HTTP DOSSIER_SORTIE PREFIXE
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

# JOUER, l'écran d'avant la partie, puis la carte des toits ; ensuite N
# jusqu'à la Pieuvre (indice 18), une carte par niveau.
etapes=(t:Enter:120 t:Enter:60 w:120 c:n00-carte t:Enter:60)
for i in $(seq 1 18); do
  nn=$(printf '%02d' "$i")
  etapes+=(t:KeyN:60 w:60 "c:n$nn-carte" t:Enter:60)
done
timeout 900 "$NODE" "$ici/capture-102.mjs" "$1" "$2" "$3" "${etapes[@]}"
