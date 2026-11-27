#!/usr/bin/env bash
# Captures de la carte 9.36a : la NOUVELLE phase 2 de la Pieuvre (la montée contre la
# mer, les objets qu'elle lance de loin, sa fuite au sommet), jouée APRÈS la phase 1
# dans le niveau de la Pieuvre, jusqu'au bilan. Parcours entièrement déterministes du
# build de QA, pris avec le pilote de `capture-102.mjs` (horloge figée, 1/60 s par
# image), en deux modes (comme `docs/capture-933.sh`, qu'il remplace) :
#
# MODE normal : `docs/script-pieuvre-936.txt` (test ignoré `ecrire_script_pieuvre_936`,
#   `game/src/bot_qa/captures_936.rs` : le robot de QA de partie, phase 1 puis le
#   pilote de la tour, aucune mort), rejoué depuis la carte de titre de la Pieuvre
#   (touche 9, puis N jusqu'à elle) ; son 1er caractère est l'Entrée qui passe la carte.
#   Jusqu'à son image 2419, il est IDENTIQUE à `docs/script-pieuvre-933.txt` (la même
#   phase 1, jusqu'au début de la montée) : la phase 1 se compare donc au pixel avec
#   l'ancien build (variable MOMENTS, plus bas).
# MODE facile : `docs/script-pieuvre-936-facile.txt` (test ignoré
#   `ecrire_script_pieuvre_936_facile`), rejoué depuis le TITRE avec une sauvegarde où
#   tout est fini avant la Pieuvre : JOUER, Pépin tombe dix fois dans le bassin (carte
#   9.34 : dix défaites de la VISITE), la bannière « TROP DUR ? », OUI ; puis la phase
#   1 en facile (2 têtes), la transition, la montée et la fuite.
#
# Captures (images du script) : PREFIXE-phase1.png (le combat), -fin-phase1.png (juste
# avant le dernier coup), -transition.png (elle se retire au large, la bâche
# déchirée), -tour-bas.png, -chute.png et -chute-b.png (une chute annoncée par son
# ombre, puis l'objet qui tombe), -tour-milieu.png, -tour-haut.png, -fuite-a.png et
# -fuite-b.png (au sommet, elle plonge au large, la mer redescend), -bilan.png. En
# normal, en plus : PREFIXE-carte.png (la carte de titre de la Pieuvre).
#
# Usage : CHROME=chemin/de/chromium docs/capture-936.sh PORT_HTTP DOSSIER_SORTIE PREFIXE MODE [NB_N]
# (dist-qa/ servi sur 127.0.0.1:PORT_HTTP, Node >= 22, variable NODE pour un autre Node).
# MODE : normal ou facile. NB_N (mode normal) : nombre d'appuis sur N après la touche 9
# pour arriver à la Pieuvre (11 depuis la carte 9.32). MOMENTS="nom:image ..." remplace
# la liste des captures (par exemple les seules images de la phase 1, sur l'ancien
# build, avec l'ancien script : SCRIPT=chemin).
set -eu
if [ $# -lt 4 ]; then
  echo "Usage : CHROME=... $0 PORT_HTTP DOSSIER_SORTIE PREFIXE normal|facile [NB_N]" >&2
  exit 2
fi
: "${CHROME:?indiquer le chemin de Chromium dans CHROME}"
export CHROME
ici=$(cd "$(dirname "$0")" && pwd)
NODE="${NODE:-node}"
mode="$4"

etapes=()
case "$mode" in
  normal)
    # Une sauvegarde VIDE (une variable SAUVEGARDE héritée changerait tout le parcours).
    unset SAUVEGARDE
    script="${SCRIPT:-$ici/script-pieuvre-936.txt}"
    nb_n="${5:-11}"
    # Le titre (la 4e Entrée passe la carte de titre du 1er niveau), touche 9 (indice
    # 8), Entrée, puis N et Entrée jusqu'au niveau d'avant la Pieuvre, et un dernier
    # N : la carte de titre de la Pieuvre, prête après 60 images.
    etapes=(t:Enter:120 t:Enter:60 t:Enter:120 t:Enter:60 w:60 t:Digit9:60 t:Enter:60)
    for i in $(seq $((nb_n - 1))); do
      etapes+=(t:KeyN:60 t:Enter:60)
    done
    etapes+=(t:KeyN:60 w:60 c:carte)
    defaut="phase1:1200 fin-phase1:2100 transition:2190 tour-bas:2300 chute:2873 chute-b:2898 tour-milieu:3243 tour-haut:4258 fuite-a:4530 fuite-b:4590 bilan:4791"
    ;;
  facile)
    script="${SCRIPT:-$ici/script-pieuvre-936-facile.txt}"
    # `SAUVEGARDE_FACILE` de `captures_936.rs` : tout fini avant la Pieuvre.
    sv="version=3"
    for c in toits gouttieres marche clocher poursuite descente vents orage mer nuages \
      cyclone arc caniveau egouts cascade grotte riviere port maree; do
      sv="$sv"$'\n'"niveau.$c=atteint:1 medaille:1"
    done
    export SAUVEGARDE="$sv"$'\n'"niveau.pieuvre=defaites:10"
    defaut="phase1:2900 fin-phase1:3400 transition:3500 tour-bas:3610 chute:4093 chute-b:4118 tour-milieu:4784 tour-haut:5657 fuite-a:5897 fuite-b:5957 bilan:6158"
    ;;
  *)
    echo "MODE : normal ou facile" >&2
    exit 2
    ;;
esac
moments="${MOMENTS:-$defaut}"
# Les images du script où photographier (cumulées) : `s:` rejoue jusqu'à l'image voulue.
precedent=0
for moment in $moments; do
  nom=${moment%%:*}; image=${moment#*:}
  etapes+=("s:$script:$((image - precedent))" "c:$nom")
  precedent=$image
done
timeout 900 "$NODE" "$ici/capture-102.mjs" "$1" "$2" "$3" "${etapes[@]}"
