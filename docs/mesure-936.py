#!/usr/bin/env python3
"""Mesures de la carte 9.36b : palette, absence de flou, poids.

Pour chaque PNG passé : couleurs hors de la palette du jeu, carrés 3 x 3 non unis (un
flou ou un pixel hors grille), poids. Les PNG du jeu (assets) sont vérifiés à part
par les tests ; ici, les captures et planches agrandies x3.

Usage : python3 docs/mesure-936.py FICHIER.png [...]
"""
import importlib.util
import sys
from pathlib import Path

ICI = Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location("planche936", ICI / "planche-936.py")
pl = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(pl)


def mesurer(chemin):
    g = pl.decoder_png(chemin)
    h, w = len(g), len(g[0])
    palette = set(pl.PAL)
    couleurs = {c for r in g for c in r}
    hors = [c for c in couleurs if c not in palette]
    flous = 0
    for j in range(0, h - h % 3, 3):
        for i in range(0, w - w % 3, 3):
            c = g[j][i]
            if any(g[j + dj][i + di] != c for dj in range(3) for di in range(3)):
                flous += 1
    poids = Path(chemin).stat().st_size
    return len(couleurs), len(hors), flous, poids, (w, h)


def main():
    total_hors = total_flous = 0
    for f in sys.argv[1:]:
        n, hors, flous, poids, (w, h) = mesurer(f)
        total_hors += hors
        total_flous += flous
        print(f"{Path(f).name:42} {w}x{h} {n:2} couleurs, {hors} hors palette, {flous} carrés non unis, {poids / 1024:.1f} Ko")
    print(f"TOTAL : {total_hors} couleur(s) hors palette, {total_flous} carré(s) non uni(s)")
    return 1 if total_hors or total_flous else 0


if __name__ == "__main__":
    sys.exit(main())
