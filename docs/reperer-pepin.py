"""Repère Pépin dans une grille logique 320x180 (capture-visibilite.mjs, GRILLE=1) et recadre
la capture PNG agrandie autour de lui (carte 4.3 : captures 43-zoom-*).

Usage : python3 docs/reperer-pepin.py PEPIN_RS GRILLE.txt [CAPTURE.png SORTIE.png K]
- PEPIN_RS : le fichier game/src/sprites/pepin.rs dont on prend les dessins (avant ou après,
  par exemple `git show COMMIT:game/src/sprites/pepin.rs > ancien.rs`) ;
- affiche le sprite reconnu (nom, miroir, position, part de pixels identiques) : on sait ainsi
  QUELLE image de Pépin le jeu affichait au moment de la capture ;
- avec CAPTURE.png : écrit SORTIE.png, un cadre de 64x48 pixels logiques autour de Pépin,
  à l'agrandissement K de la capture (aucun rééchantillonnage : on découpe).
Recherche exhaustive en Python pur : une dizaine de secondes par grille.
"""
import re
import struct
import sys
import zlib


def sprites(chemin):
    src = open(chemin, encoding='utf-8').read()
    out = {}
    for nom, const in re.findall(r'\("(pepin_[a-z0-9]+)", (PEPIN_[A-Z0-9]+)\)', src):
        m = re.search(r'pub const %s: Image = &\[\n(.*?)\n\];' % const, src, re.S)
        out[nom] = re.findall(r'"([^"]*)"', m.group(1))
    return out


def reperer(grille, dessins):
    meilleur = (0, None)
    H, W = len(grille), len(grille[0])
    for nom, d in dessins.items():
        if 'mort' in nom or 'repar' in nom or 'plane' in nom:
            continue
        for miroir in (False, True):
            dd = [l[::-1] for l in d] if miroir else d
            pleins = [(x, y, c) for y, l in enumerate(dd) for x, c in enumerate(l) if c != '.']
            for oy in range(H - len(dd) + 1):
                for ox in range(W - len(dd[0]) + 1):
                    ok = 0
                    for k, (x, y, c) in enumerate(pleins):
                        if grille[oy + y][ox + x] == c:
                            ok += 1
                        elif k - ok > 8:
                            break
                    score = ok / len(pleins)
                    if score > meilleur[0]:
                        meilleur = (score, (nom, miroir, ox, oy))
    return meilleur


def lire_png(p):
    d = open(p, 'rb').read()
    i = 8
    idat = b''
    while i < len(d):
        n, t = struct.unpack('>I4s', d[i:i + 8])
        c = d[i + 8:i + 8 + n]
        i += 12 + n
        if t == b'IHDR':
            w, h, bd, ct = struct.unpack('>IIBB', c[:10])
        elif t == b'IDAT':
            idat += c
    bpp = 3 if ct == 2 else 4
    raw = zlib.decompress(idat)
    stride = w * bpp
    rows = []
    prev = bytearray(stride)
    k = 0
    for _ in range(h):
        f = raw[k]
        line = bytearray(raw[k + 1:k + 1 + stride])
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
                line[x] = (line[x] + (a if pa <= pb and pa <= pc else (b if pb <= pc else c))) & 255
        rows.append(line)
        prev = line
    return w, h, bpp, rows


def ecrire_png(p, w, h, rows3):
    brut = b''.join(b'\x00' + bytes(r) for r in rows3)

    def bloc(t, d):
        return struct.pack('>I', len(d)) + t + d + struct.pack('>I', zlib.crc32(t + d) & 0xffffffff)
    open(p, 'wb').write(b'\x89PNG\r\n\x1a\n' + bloc(b'IHDR', struct.pack('>IIBBBBB', w, h, 8, 2, 0, 0, 0))
                        + bloc(b'IDAT', zlib.compress(brut, 9)) + bloc(b'IEND', b''))


def main():
    dessins = sprites(sys.argv[1])
    grille = open(sys.argv[2]).read().split()
    score, trouve = reperer(grille, dessins)
    nom, miroir, ox, oy = trouve
    print(f'{sys.argv[2]} : {nom}{" (miroir)" if miroir else ""} en ({ox},{oy}), {score:.0%} des pixels')
    if len(sys.argv) > 3:
        w, h, bpp, rows = lire_png(sys.argv[3])
        K = int(sys.argv[5])
        x0, y0 = (w - 320 * K) // 2, (h - 180 * K) // 2
        cx = min(max(ox + 8 - 32, 0), 320 - 64)
        cy = min(max(oy + 8 - 28, 0), 180 - 48)
        sortie = []
        for y in range(48 * K):
            r = rows[y0 + (cy * K) + y]
            ligne = bytearray()
            for x in range(64 * K):
                o = (x0 + cx * K + x) * bpp
                ligne += r[o:o + 3]
            sortie.append(ligne)
        ecrire_png(sys.argv[4], 64 * K, 48 * K, sortie)


main()
