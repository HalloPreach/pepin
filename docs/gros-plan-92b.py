"""Gros plan AVANT / APRÈS de la barque (carte 9.2b, AGA : « il flotte trop haut par rapport à
l'eau »). Deux captures du jeu (agrandi K fois, image logique 320x180 centrée) : on retrouve la
barque (la tête et le bord du parapluie retourné, rangées 0 à 9 du dessin `PEPIN_BARQUE`, les
seules que l'eau ne recouvre jamais), on découpe un cadre de 48x32 pixels LOGIQUES autour d'elle
et on l'agrandit 8 fois (un pixel logique = un carré de 8x8, aucun mélange de couleurs), les deux
gros plans côte à côte, séparés d'une bande sombre de la palette.

Usage : python3 docs/gros-plan-92b.py PEPIN_RS AVANT.png APRES.png SORTIE.png K
Python pur (aucun module à installer) ; réutilise la lecture PNG de `reperer-pepin.py`.
"""
import os
import re
import sys

ici = os.path.dirname(os.path.abspath(__file__))
# `reperer-pepin.py` lance son `main()` à l'import : on n'en prend que les deux fonctions PNG.
source = open(os.path.join(ici, 'reperer-pepin.py'), encoding='utf-8').read()
source = source.replace('\nmain()', '\n')
outils = {}
exec(compile(source, 'reperer-pepin.py', 'exec'), outils)
lire_png, ecrire_png = outils['lire_png'], outils['ecrire_png']

PAL = ['10141F', '1C2336', '2D3A54', '46587A', '7F93B5', 'A9B8D0', 'C3D1E6', 'F3F1EA',
       'FF5A4F', 'FF9A7A', 'FFB347', 'FFD84A', 'FFF1A8', '62D0E8', '2F8F9D', '5B4A8A']
INDICE = {int(c, 16): format(i, 'X') for i, c in enumerate(PAL)}
L, H, ECHELLE = 48, 32, 8


def barque(chemin):
    src = open(chemin, encoding='utf-8').read()
    m = re.search(r'pub const PEPIN_BARQUE: Image = &\[\n(.*?)\n\];', src, re.S)
    return re.findall(r'"([^"]*)"', m.group(1))


def grille(png, K):
    w, h, bpp, rows = lire_png(png)
    x0, y0 = (w - 320 * K) // 2, (h - 180 * K) // 2
    g = []
    for ly in range(180):
        r = rows[y0 + ly * K]
        ligne = ''
        for lx in range(320):
            o = (x0 + lx * K) * bpp
            ligne += INDICE.get((r[o] << 16) | (r[o + 1] << 8) | r[o + 2], 'x')
        g.append(ligne)
    return g


def trouver(g, dessin):
    haut = dessin[:10]  # la tête et le bord : jamais sous l'eau
    pleins = [(x, y, c) for y, l in enumerate(haut) for x, c in enumerate(l) if c != '.']
    meilleur = (0, (0, 0))
    for miroir in (False, True):
        pp = [(15 - x, y, c) for x, y, c in pleins] if miroir else pleins
        for oy in range(180 - 16):
            for ox in range(320 - 16):
                ok = sum(1 for x, y, c in pp if g[oy + y][ox + x] == c)
                if ok > meilleur[0]:
                    meilleur = (ok, (ox, oy))
    return meilleur[1], meilleur[0] / len(pleins)


def gros_plan(g, ox, oy):
    cx = min(max(ox + 8 - L // 2, 0), 320 - L)
    cy = min(max(oy + 10 - H // 2, 0), 180 - H)
    lignes = []
    for y in range(H):
        ligne = bytearray()
        for x in range(L):
            c = g[cy + y][cx + x]
            rgb = bytes.fromhex(PAL[int(c, 16)]) if c != 'x' else b'\xff\x00\xff'
            ligne += rgb * ECHELLE
        lignes += [ligne] * ECHELLE
    return lignes


def main():
    dessin = barque(sys.argv[1])
    K = int(sys.argv[5])
    plans = []
    for png in sys.argv[2:4]:
        g = grille(png, K)
        (ox, oy), part = trouver(g, dessin)
        print(f'{os.path.basename(png)} : barque en ({ox},{oy}), {part:.0%} de la tête et du bord')
        plans.append(gros_plan(g, ox, oy))
    bande = bytes.fromhex(PAL[0]) * ECHELLE
    sortie = [a + bande + b for a, b in zip(*plans)]
    ecrire_png(sys.argv[4], 2 * L * ECHELLE + ECHELLE, H * ECHELLE, sortie)


main()
