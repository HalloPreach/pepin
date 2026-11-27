#!/usr/bin/env bash
# OBSOLÈTE depuis la carte 9.36a : la phase 2 qu'il montre (la géante, la vigie, la
# plongée) n'existe plus. Il ne vaut que pour un build d'avant la 9.36a (son script
# normal sert encore de référence pour la PHASE 1, identique jusqu'à son image 2419 :
# voir `docs/capture-936.sh`). Les tests qui écrivaient ses scripts s'appellent
# maintenant `ecrire_script_pieuvre_936[_facile]`.
#
# Captures de la carte 9.33 : la VRAIE phase 2 de la Pieuvre (« Fuite et montée »),
# jouée APRÈS la phase 1 dans le niveau de la Pieuvre, pas dans un niveau d'essai posé
# dans la tour (les captures de « montée » des cartes 10.4 le faisaient). Parcours
# entièrement déterministes du build de QA, pris avec le pilote de `capture-102.mjs`
# (horloge figée, 1/60 s par image), en deux modes :
#
# MODE normal : `docs/script-pieuvre-933.txt` (test ignoré `ecrire_script_pieuvre_933`,
#   `game/src/bot_qa/captures_933.rs` : le robot de QA de partie, phase 1 puis le
#   pilote de la tour, aucune mort), rejoué depuis la carte de titre de la Pieuvre
#   (touche 9, puis N jusqu'à elle) ; son 1er caractère est l'Entrée qui passe la carte.
# MODE facile : `docs/script-pieuvre-933-facile.txt` (test ignoré
#   `ecrire_script_pieuvre_933_facile`), rejoué depuis le TITRE avec une sauvegarde où
#   la Pieuvre a 10 défaites (variable SAUVEGARDE de `capture-102.mjs`, même texte que
#   `SAUVEGARDE_FACILE`) : JOUER, Pépin tombe à l'eau, la bannière « TROP DUR ? »,
#   OUI ; puis la phase 1 en facile (2 têtes), la fuite (le pilote y meurt une fois et
#   repart de la lanterne du bas de la tour), la montée et la vigie.
#   C'est dans ce mode que la géante n'était PAS dessinée (défaut de la carte 9.33).
#
# Captures (images du script) :
#   PREFIXE-fuite-a.png, -fuite-b.png : la fuite (la trombe, la géante qui part) ;
#   PREFIXE-tour-bas.png : le début de la montée, en bas de la tour ;
#   PREFIXE-tour-milieu.png, -tour-haut.png : la montée ;
#   PREFIXE-vigie.png, -vigie-b.png : sur la vigie, la géante en face ;
#   PREFIXE-plongee.png : le coup final, la géante replonge.
# En normal, en plus : PREFIXE-carte.png (la carte de titre de la Pieuvre).
#
# Usage : CHROME=chemin/de/chromium docs/capture-933.sh PORT_HTTP DOSSIER_SORTIE PREFIXE MODE [NB_N]
# (dist-qa/ servi sur 127.0.0.1:PORT_HTTP, Node >= 22, variable NODE pour un autre Node).
# MODE : normal ou facile. NB_N (mode normal) : nombre d'appuis sur N après la touche 9
# pour arriver à la Pieuvre : 11 depuis la carte 9.32 (la marée passe AVANT la Pieuvre),
# 10 avant (commits `c4d715b`, `dd45fb2`).
# Comparer : fait à la fin, avec les références `docs/qa/933-ref/` (voir plus bas).
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
    script="$ici/script-pieuvre-933.txt"
    nb_n="${5:-11}"
    # Le titre (la 4e Entrée passe la carte de titre du 1er niveau), touche 9 (indice
    # 8), Entrée, puis N et Entrée jusqu'au niveau d'avant la Pieuvre, et un dernier
    # N : la carte de titre de la Pieuvre, prête après 60 images.
    etapes=(t:Enter:120 t:Enter:60 t:Enter:120 t:Enter:60 w:60 t:Digit9:60 t:Enter:60)
    for i in $(seq $((nb_n - 1))); do
      etapes+=(t:KeyN:60 t:Enter:60)
    done
    etapes+=(t:KeyN:60 w:60 c:carte)
    moments="fuite-a:2150 fuite-b:2200 tour-bas:2312 tour-milieu:3240 tour-haut:3960 vigie:4174 vigie-b:4700 plongee:6300"
    ;;
  facile)
    script="$ici/script-pieuvre-933-facile.txt"
    # `SAUVEGARDE_FACILE` de `captures_933.rs` : tout fini avant la Pieuvre, 10 défaites.
    sv="version=3"
    for c in toits gouttieres marche clocher poursuite descente vents orage mer nuages \
      cyclone arc caniveau egouts cascade grotte riviere port maree; do
      sv="$sv"$'\n'"niveau.$c=atteint:1 medaille:1"
    done
    export SAUVEGARDE="$sv"$'\n'"niveau.pieuvre=defaites:10"
    moments="fuite-a:1770 fuite-b:1785 tour-bas:1900 tour-milieu:2600 tour-haut:3600 vigie:4190 vigie-b:4700 plongee:5480"
    ;;
  *)
    echo "MODE : normal ou facile" >&2
    exit 2
    ;;
esac
# Les images du script où photographier (cumulées) : `s:` rejoue jusqu'à l'image voulue.
precedent=0
for moment in $moments; do
  nom=${moment%%:*}; image=${moment#*:}
  etapes+=("s:$script:$((image - precedent))" "c:$nom")
  precedent=$image
done
timeout 900 "$NODE" "$ici/capture-102.mjs" "$1" "$2" "$3" "${etapes[@]}"

# Comparaison AU PIXEL avec les références (`docs/qa/933-ref/MODE-*.png`, prises
# après le correctif de la carte 9.33 sur monde-2 `420da59`) : code de sortie 1 si
# une capture diffère. COMPARER=0 pour s'en passer (un commit plus ancien : la marée
# change le compteur de gouttes, le build de `c4d715b` a une autre montée).
if [ "${COMPARER:-1}" != "0" ]; then
  paires=()
  for f in "$ici"/qa/933-ref/"$mode"-*.png; do
    nom=$(basename "$f" .png)
    paires+=("$f" "$2/$3-${nom#"$mode"-}.png")
  done
  python3 "$ici/comparer-captures.py" "${paires[@]}"
fi
