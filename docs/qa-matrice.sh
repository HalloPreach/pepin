#!/usr/bin/env bash
# Matrice de QA navigateur « clavier AZERTY / raccourcis 1-5 / écran haute densité ».
# Prérequis : Chromium (variable CHROME), Node >= 22 (variable NODE, défaut : node), et trois
# serveurs statiques sur 127.0.0.1 :
#   8101 = ancien dist/ (avant correction)   8102 = dist/ actuel   8103 = dist-qa/ (./build.sh --qa)
# Ex. : (cd dist && python3 -m http.server 8102 --bind 127.0.0.1) &  etc.
# Résultats : captures docs/qa/91-*.png et mesures JSON sur la sortie standard.
set -u
cd "$(dirname "$0")/.."
: "${CHROME:?indiquer le chemin de Chromium dans CHROME}"
NODE="${NODE:-node}"; export CHROME
run(){ echo "== $*"; timeout 200 "$NODE" docs/qa-clavier-ecran.mjs "$@" 2>&1 | head -14; }
Q=docs/qa/91
MOBILE=1 run ecran 8101 $Q-tel844x340-avant 844 340 3
MOBILE=1 run ecran 8102 $Q-tel844x340-apres 844 340 3
MOBILE=1 run ecran 8101 $Q-tel844x390-avant 844 390 3
MOBILE=1 run ecran 8102 $Q-tel844x390-apres 844 390 3
run ecran 8102 $Q-bureau1280x720-dpr1-apres 1280 720 1
run ecran 8102 $Q-bureau1280x720-dpr125-apres 1280 720 1.25
run ecran 8102 $Q-bureau1280x720-dpr150-apres 1280 720 1.5
run ecran 8101 $Q-bureau1280x600-dpr150-avant 1280 600 1.5
run ecran 8102 $Q-bureau1280x600-dpr150-apres 1280 600 1.5
run ecran 8101 $Q-bureau1000x560-dpr125-avant 1000 560 1.25
run ecran 8102 $Q-bureau1000x560-dpr125-apres 1000 560 1.25
LIGNE=1 run ecran 8102 $Q-souris-dpr150-ligne2 1280 720 1.5
run azerty 8101 $Q-azerty-avant
run azerty 8102 $Q-azerty-apres
run chiffres 8101 $Q-chiffres-avant
run chiffres 8102 $Q-chiffres-production
run chiffres 8103 $Q-chiffres-qa
