#!/usr/bin/env bash
# Captures de référence de la carte 10.4d (les fonds du Monde 2 deviennent des fichiers
# PNG, `assets/fonds/egouts` et `assets/fonds/port`) : le jeu doit rester identique AU
# PIXEL. Parcours entièrement déterministes du build de QA, pris avec le pilote de
# `capture-102.mjs` (horloge figée, 1/60 s par image) :
#   PREFIXE-nNN-depart.png (NN = 12 à 18) : le départ des 7 niveaux du Monde 2 (caniveau,
#     égouts, cascade, grotte, rivière, port, Pieuvre du port), 60 images après la carte
#     de titre (touche 9 du build de QA, puis N jusqu'au niveau voulu) ;
#   PREFIXE-nNN-milieu.png : le même niveau après 150 images de course vers la droite (la
#     caméra a avancé : la parallaxe des couches a bougé ; une chute éventuelle ne gêne
#     pas, le parcours reste le même à chaque passage) ;
#   PREFIXE-n18-b.png, -c.png : la Pieuvre, 300 puis 900 images plus tard (les feux des
#     grues, les bouées et les reflets ont changé d'état) ;
#   PREFIXE-montee-bas-a.png, -b.png, PREFIXE-montee-haut-a.png, -b.png : la MONTÉE de
#     la Pieuvre à deux hauteurs. Le niveau de la Pieuvre, Pépin posé dans la tour (en
#     bas, rangée 66 : le fond de l'arène, décalé vers le bas, entre par le bas de
#     l'écran ; en haut, rangée 22 : la nuit et le port lointain), est lancé comme un
#     niveau d'essai (`ESSAI_RON`, `?essai=1`, voir `capture-102.mjs`) ; capture après la
#     carte de titre, puis après 40 images de course et 200 d'attente.
# La planche des fonds (touche V) n'est PAS dans ces captures : son pas compte les images
# réelles du navigateur, deux passages du MÊME build y diffèrent.
# À chaque capture, des couches CALCULÉES passent ENTRE des couches PNG : au port, les
# étoiles et les mouettes entre le crépuscule et la mer, le large (cargo qui glisse,
# faisceau du phare) entre la mer et le port d'en face, les feux de balisage et les
# voiliers entre le port d'en face et les pontons, les bouées devant ; aux égouts, les
# piliers (gouttes qui tombent) entre les arches et l'eau, les tuyaux devant.
#
# Usage : CHROME=chemin/de/chromium docs/capture-104d.sh PORT_HTTP DOSSIER_SORTIE PREFIXE
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

# Les toits (la 4e Entrée passe la carte de titre), touche 9 (indice 8), puis N jusqu'au
# caniveau (indice 12), et ainsi de suite jusqu'à la Pieuvre (indice 18).
etapes=(t:Enter:120 t:Enter:60 t:Enter:120 t:Enter:60 w:60 t:Digit9:60 t:Enter:60)
for i in 1 2 3; do
  etapes+=(t:KeyN:60 t:Enter:60)
done
for i in 12 13 14 15 16 17 18; do
  etapes+=(t:KeyN:60 t:Enter:60 w:60 "c:n$i-depart" m:4:150 "c:n$i-milieu")
done
etapes+=(w:300 c:n18-b w:900 c:n18-c)
timeout 900 "$NODE" "$ici/capture-102.mjs" "$1" "$2" "$3" "${etapes[@]}"


# La montée : le niveau de la Pieuvre, le départ `@` déplacé dans la tour.
for h in bas:66:4 haut:22:6; do
  nom=${h%%:*}; reste=${h#*:}; rangee=${reste%%:*}; colonne=${reste#*:}
  ron="$2/pieuvre-tour-$nom.ron"
  python3 - "$ici/../assets/levels/monde_2/pieuvre.ron" "$ron" "$rangee" "$colonne" <<'FIN'
import sys
source, cible, rangee, colonne = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
texte = open(source, encoding="utf-8").read()
debut = texte.index('ascii: r#"')
lignes = texte[debut:].split("\n")
lignes = [l.replace("@", ".") for l in lignes]
l = lignes[rangee]
assert l[colonne] == ".", l
lignes[rangee] = l[:colonne] + "@" + l[colonne + 1:]
open(cible, "w", encoding="utf-8").write(texte[:debut] + "\n".join(lignes))
FIN
  ESSAI_RON="$ron" timeout 300 "$NODE" "$ici/capture-102.mjs" "$1" "$2" "$3-montee-$nom" \
    w:180 t:Enter:90 c:a m:4:40 w:200 c:b
done
