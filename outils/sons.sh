#!/usr/bin/env bash
# Encode les sons de PÉPIN en OGG Vorbis (carte 10.4e) : chaque WAV 16 bits mono d'un
# dossier source devient `assets/musiques/<clé>.ogg` ou `assets/sons/<clé>.ogg`.
#
# Usage : FFMPEG=chemin/de/ffmpeg outils/sons.sh SOURCE
#   SOURCE/musiques/<clé>.wav -> assets/musiques/<clé>.ogg  (qualité Vorbis 4)
#   SOURCE/sons/<clé>.wav     -> assets/sons/<clé>.ogg      (qualité Vorbis 6)
# Le chemin de ffmpeg n'est PAS écrit dans le dépôt : il vient de la variable FFMPEG
# (un ffmpeg construit avec libvorbis).
#
# Réglages (voir `assets/musiques/LISEZMOI.md`) :
#   -ac 1 -ar 22050            mono, 22 050 Hz (la fréquence du synthé d'origine) ;
#   -q:a                       qualité Vorbis : 4 pour les musiques (les boucles
#                              longues : le total reste sous 600 Ko), 6 pour les
#                              bruitages (courts : la qualité coûte peu) ;
#   -page_duration             durée d'une page OGG : 1 s pour les musiques (la valeur
#                              de ffmpeg), 10 ms pour les bruitages. Un son court
#                              tenait sinon dans UNE seule page, et ffmpeg 9 en coupait
#                              mal le remplissage du Vorbis (256 à 500 échantillons de
#                              trop ou de moins), et le décodeur `lewton` (celui de
#                              macroquad hors du web) se trompait encore avec des
#                              pages de 20 à 200 ms (pluie, rebond, plane...). Avec
#                              10 ms, ffmpeg (donc Chromium) ET lewton décodent
#                              exactement la longueur du WAV, pour les 13 bruitages ;
#   -map_metadata -1, -metadata:s:a:0 encoder=, -fflags +bitexact, -flags:a +bitexact
#                              AUCUNE métadonnée : ni titre, ni commentaire
#                              « encoder=Lavc libvorbis » (que ffmpeg ajoute même en
#                              bitexact sans le `encoder=` vide), ni marque « Lavf »
#                              (règle « aucun nom »). Reste la chaîne « ffmpeg » du
#                              champ FABRICANT, obligatoire dans un en-tête Vorbis.
# Ensuite, contrôler : `./docker-cargo.sh run -p outils --bin son -- controler ...`.
set -eu
if [ $# -ne 1 ]; then
  echo "Usage : FFMPEG=... $0 SOURCE" >&2
  exit 2
fi
: "${FFMPEG:?indiquer le chemin de ffmpeg dans FFMPEG}"
ici=$(cd "$(dirname "$0")/.." && pwd)
for genre in musiques sons; do
  case $genre in
    musiques) qualite=4 page=1000000 ;;
    sons) qualite=6 page=10000 ;;
  esac
  mkdir -p "$ici/assets/$genre"
  for wav in "$1/$genre"/*.wav; do
    cle=$(basename "$wav" .wav)
    "$FFMPEG" -v error -y -i "$wav" -ac 1 -ar 22050 -c:a libvorbis -q:a "$qualite" \
      -page_duration "$page" -map_metadata -1 -metadata:s:a:0 encoder= \
      -fflags +bitexact -flags:a +bitexact \
      "$ici/assets/$genre/$cle.ogg"
    echo "$genre/$cle.ogg $(wc -c < "$ici/assets/$genre/$cle.ogg") octets"
  done
done
