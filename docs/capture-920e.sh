#!/usr/bin/env bash
# Captures de la carte 9.20e (la jetée de « La Pieuvre du port ») sur le build de QA, avec
# `capture-96.mjs` en pilotage à la main : JOUER, N jusqu'au niveau de la Pieuvre (le 16e
# niveau de la table), puis la partie du robot sur ce niveau rejouée masque par masque.
# Le script vient du test ignoré `ecrire_script_pieuvre_912b` (`target/qa/script-912b.txt`),
# qui affiche la 1re image de jeu du niveau (DEBUT) ; DECALAGE : images à attendre après le
# dernier N (fin des volets et de la carte de titre ; 46 au build du 2026-11-17, vérifié sur
# la trace : Pépin à x 93 et tentacules « Xaa..XXXX » au pas 1092).
# Usage : CHROME=... docs/capture-920e.sh PORT_HTTP PREFIXE [DECALAGE] [DEBUT] [FIN]
set -eu
cd "$(dirname "$0")/.."
PORT=$1; PRE=$2; DEC=${3:-46}; DEBUT=${4:-7918}; FIN=${5:-9424}
SORTIE=${SORTIE:-${TMPDIR:-/tmp}/cap920e}
ARGS=$(python3 - "$PRE" "$DEC" "$DEBUT" "$FIN" <<'EOF'
import sys, itertools
pre, dec, debut, fin = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4])
seg = open('target/qa/script-912b.txt').read()[debut:fin]
caps = {120: 'depart', 1062: 'annonce', 1092: 'milieu', 1440: 'sortie'}
args = ['t:Enter:300'] + ['t:KeyN:60'] * 15 + ['t:KeyN:%d' % (1 + dec)]
pas = 0
for k, g in itertools.groupby(seg):
    m, n = int(k, 32), len(list(g))
    while n > 0:
        prochaine = min([c for c in caps if c > pas] or [10**9])
        pris = min(n, prochaine - pas)
        args.append(('m:%d:%d' % (m, pris)) if m else ('w:%d' % pris))
        pas += pris
        n -= pris
        if pas in caps:
            args.append('c:%s-%s' % (pre, caps[pas]))
print(' '.join(args))
EOF
)
timeout 400 node docs/capture-96.mjs "$PORT" "$SORTIE" - $ARGS 2>&1 | tail -7
