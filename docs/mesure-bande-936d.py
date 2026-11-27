#!/usr/bin/env python3
"""Carte 9.36d : mesure la « bande sombre » sur la mer dans des captures (x3).

Une corniche NOYÉE (carte 9.36b) se dessine en silhouette d'ardoise (45, 58, 84),
un cran plus sombre que l'eau. Juste sous l'écume, l'eau est turquoise
(47, 143, 157) : une longue corniche à cette profondeur fait donc une longue
bande d'ardoise dans le turquoise.

Pour chaque PNG : parmi les lignes où le turquoise couvre au moins 64 px du jeu
(la bande claire sous l'écume), la plus longue suite d'ardoise sur une même
ligne (en px du jeu), et le nombre de ces lignes où elle fait au moins 32 px.
Avec --seuil N : code de sortie 1 si une image a une suite de N px ou plus
(une vérification possible des captures, si l'on décide de changer le dessin).

Usage : python3 docs/mesure-bande-936d.py [--seuil N] FICHIER.png [...]
"""
import importlib.util
import sys
from pathlib import Path

ICI = Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location("planche936", ICI / "planche-936.py")
pl = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(pl)
TURQUOISE = (47, 143, 157)
ARDOISE = (45, 58, 84)
ECHELLE = 3


def mesurer(chemin):
    g = pl.decoder_png(chemin)
    plus_longue, lignes = 0, 0
    # Une ligne du jeu sur trois (le centre de chaque carré de 3 x 3 pixels).
    for j in range(1, len(g), ECHELLE):
        ligne = [c[:3] for c in g[j][1::ECHELLE]]
        if sum(1 for c in ligne if c == TURQUOISE) < 64:
            continue
        suite = meilleure = 0
        for c in ligne:
            suite = suite + 1 if c == ARDOISE else 0
            meilleure = max(meilleure, suite)
        if meilleure >= 32:
            lignes += 1
        plus_longue = max(plus_longue, meilleure)
    return plus_longue, lignes


def main():
    args = sys.argv[1:]
    seuil = None
    if args[:1] == ["--seuil"]:
        seuil, args = int(args[1]), args[2:]
    trop = 0
    for f in args:
        m, n = mesurer(f)
        print(f"{Path(f).name:40s} suite d'ardoise {m:4d} px, lignes (>= 32 px) {n:3d}")
        if seuil is not None and m >= seuil:
            trop += 1
    if seuil is not None:
        print(f"{trop} image(s) avec une suite d'ardoise de {seuil} px ou plus")
        sys.exit(1 if trop else 0)


if __name__ == "__main__":
    main()
