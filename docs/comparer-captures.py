#!/usr/bin/env python3
"""Compare deux captures PNG pixel par pixel (carte 10.1b : le départ des niveaux
avant et après la bascule vers les fichiers RON doit être IDENTIQUE).

Usage : python3 docs/comparer-captures.py AVANT.png APRES.png [...paires]
Affiche, pour chaque paire, la taille et le nombre de pixels différents ; code de
sortie 1 si une paire diffère. Sans dépendance (zlib et struct de la bibliothèque
standard ; même lecteur de PNG que `mesure-capture.py`).
"""
import pathlib
import struct
import sys
import zlib


def lire_png(chemin):
    """(largeur, hauteur, lignes de pixels décodées) d'un PNG 8 bits RGB ou RGBA."""
    d = open(chemin, "rb").read()
    i, idat = 8, b""
    while i < len(d):
        n, t = struct.unpack(">I4s", d[i : i + 8])
        c = d[i + 8 : i + 8 + n]
        i += 12 + n
        if t == b"IHDR":
            w, h, bd, ct = struct.unpack(">IIBB", c[:10])
        elif t == b"IDAT":
            idat += c
    assert bd == 8 and ct in (2, 6), (bd, ct)
    bpp = 3 if ct == 2 else 4
    raw = zlib.decompress(idat)
    stride, rows, prev, k = w * bpp, [], bytearray(w * bpp), 0
    for _ in range(h):
        f = raw[k]
        line = bytearray(raw[k + 1 : k + 1 + stride])
        k += 1 + stride
        for x in range(stride):
            a = line[x - bpp] if x >= bpp else 0
            b = prev[x]
            c = prev[x - bpp] if x >= bpp else 0
            if f == 1:
                line[x] = (line[x] + a) & 255
            elif f == 2:
                line[x] = (line[x] + b) & 255
            elif f == 3:
                line[x] = (line[x] + (a + b) // 2) & 255
            elif f == 4:
                pa, pb, pc = abs(b - c), abs(a - c), abs(a + b - 2 * c)
                pr = a if pa <= pb and pa <= pc else (b if pb <= pc else c)
                line[x] = (line[x] + pr) & 255
        rows.append(bytes(line))
        prev = line
    return w, h, bpp, rows


def main():
    args = sys.argv[1:]
    if not args or len(args) % 2:
        print(__doc__)
        sys.exit(2)
    ecart = False
    for a, b in zip(args[::2], args[1::2]):
        la, ha, bpa, ra = lire_png(a)
        lb, hb, bpb, rb = lire_png(b)
        if (la, ha, bpa) != (lb, hb, bpb):
            print(f"{a} / {b} : formats différents")
            ecart = True
            continue
        diff = 0
        for x, y in zip(ra, rb):
            if x != y:
                diff += sum(1 for p in range(la) if x[p * bpa : p * bpa + bpa] != y[p * bpa : p * bpa + bpa])
        print(f"{pathlib.Path(b).name} : {la}x{ha}, {diff} pixel(s) différent(s)")
        ecart |= diff > 0
    sys.exit(1 if ecart else 0)


main()
