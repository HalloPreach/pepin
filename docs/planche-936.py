#!/usr/bin/env python3
"""Planche de PROPOSITION de la carte 9.36b : le dessin de la nouvelle phase 2 de la
Pieuvre (la montée contre la mer), à montrer à AGA AVANT de rien changer au jeu.

Ce que la planche propose (à l'échelle x3, 1 pixel du jeu = 3 x 3 pixels) :
- la TOUR : un mur derrière les corniches, dans le style du port (pierre, bois,
  fenêtres, une lanterne), en deux variantes : A (mur plein) et B (arcades ouvertes
  sur le crépuscule du port) ;
- la MER qui monte : une vraie masse d'eau (crête d'écume, reflets, profondeur,
  corniches noyées en silhouette) ;
- la PIEUVRE AU LOIN qui lance : trois images (repos, prépare, lance), avec sa tête de
  la phase 1 ;
- les OBJETS qui tombent (caisse, tonneau, épave), leur ombre et le repère d'annonce ;
- la FUITE : quatre images ;
- deux montages d'une scène complète de montée (320 x 180), un par variante.

Les sprites existants (tête de la Pieuvre, tuiles du port, Pépin, lanterne, écume) sont
LUS dans `assets/sprites/` ; les nouveaux dessins sont décrits ici (grilles de texte ou
formes simples avec un contour sombre de 1 px ajouté automatiquement). Seules les 16
couleurs de `pepin_core::palette`, aucun flou.

Le montage part d'une VRAIE capture de la carte 9.36a
(`docs/qa/936/normal-tour-milieu.png`, réduite au pixel près) : tuiles, Pépin,
lanternes, jauge et interface restent les siens ; seuls le ciel vide (remplacé par le
mur), la mer et les ajouts (Pieuvre, objets, repère) sont dessinés ici.

Usage : python3 docs/planche-936.py   (écrit docs/qa/936-proposition-*.png)
Python seul, sans bibliothèque.
"""
import math
import re
import struct
import sys
import zlib
from pathlib import Path

RACINE = Path(__file__).resolve().parent.parent
SPRITES = RACINE / "assets" / "sprites"
FONDS = RACINE / "assets" / "fonds"
SORTIE = RACINE / "docs" / "qa"
ECHELLE = 3


# ---------------------------------------------------------------- palette et PNG

def palette():
    """Les 16 couleurs, dans l'ordre de `TOUTES` (indice = caractère hexadécimal)."""
    texte = (RACINE / "core" / "src" / "palette.rs").read_text()
    couleurs = {
        nom: tuple(int(v, 16) for v in rvb)
        for nom, *rvb in re.findall(
            r"pub const (\w+): Couleur = Couleur::new\(0x(\w\w), 0x(\w\w), 0x(\w\w)\)", texte
        )
    }
    toutes = re.search(r"pub const TOUTES: \[Couleur; 16\] = \[(.*?)\];", texte, re.S).group(1)
    return [couleurs[n] for n in re.findall(r"\w+", toutes)]


PAL = palette()


def decoder_png(chemin):
    """Grille de couleurs RVB (None = transparent) d'un PNG 8 bits indexé ou RVB(A)."""
    d = Path(chemin).read_bytes()
    p, idat, plte, trns = 8, b"", None, None
    while p < len(d):
        n, = struct.unpack(">I", d[p:p + 4])
        t, c = d[p + 4:p + 8], d[p + 8:p + 8 + n]
        p += 12 + n
        if t == b"IHDR":
            w, h, _, ct = struct.unpack(">IIBB", c[:10])
        elif t == b"PLTE":
            plte = [tuple(c[i:i + 3]) for i in range(0, len(c), 3)]
        elif t == b"tRNS":
            trns = c
        elif t == b"IDAT":
            idat += c
    bpp = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}[ct]
    brut, st, i = zlib.decompress(idat), w * bpp, 0
    prec, lignes = bytearray(st), []
    for _ in range(h):
        f = brut[i]
        ligne = bytearray(brut[i + 1:i + 1 + st])
        i += 1 + st
        for x in range(st):
            a = ligne[x - bpp] if x >= bpp else 0
            b = prec[x]
            c = prec[x - bpp] if x >= bpp else 0
            if f == 1:
                ligne[x] = (ligne[x] + a) & 255
            elif f == 2:
                ligne[x] = (ligne[x] + b) & 255
            elif f == 3:
                ligne[x] = (ligne[x] + (a + b) // 2) & 255
            elif f == 4:
                pa, pb, pc = abs(b - c), abs(a - c), abs(a + b - 2 * c)
                ligne[x] = (ligne[x] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        lignes.append(ligne)
        prec = ligne
    g = []
    for ligne in lignes:
        if ct == 3:
            g.append([None if trns and v < len(trns) and trns[v] == 0 else plte[v] for v in ligne])
        else:
            g.append([tuple(ligne[x * bpp:x * bpp + 3]) for x in range(w)])
    return g


def en_indices(g):
    """Grille RVB vers grille d'indices de palette (None = transparent)."""
    return [[None if c is None else PAL.index(c) for c in r] for r in g]


def sprite(nom):
    """Un sprite du jeu, par sa clé (cherché dans toutes les familles)."""
    chemin = next(SPRITES.rglob(nom + ".png"))
    return en_indices(decoder_png(chemin))


def ecrire_png(chemin, g, echelle):
    """PNG RVB, chaque pixel agrandi en carré `echelle` x `echelle` (aucun flou)."""
    h, w = len(g), len(g[0])
    brut = bytearray()
    for r in g:
        ligne = bytearray([0])
        for v in r:
            ligne += bytes(PAL[v]) * echelle
        brut += bytes(ligne) * echelle
    def bloc(t, c):
        return struct.pack(">I", len(c)) + t + c + struct.pack(">I", zlib.crc32(t + c) & 0xFFFFFFFF)
    Path(chemin).write_bytes(
        b"\x89PNG\r\n\x1a\n"
        + bloc(b"IHDR", struct.pack(">IIBBBBB", w * echelle, h * echelle, 8, 2, 0, 0, 0))
        + bloc(b"IDAT", zlib.compress(bytes(brut), 9))
        + bloc(b"IEND", b"")
    )


# ---------------------------------------------------------------- dessin

def toile(w, h, fond=None):
    return [[fond] * w for _ in range(h)]


def grille(*lignes):
    """Grille de texte (`.` transparent, `0`-`F` indice) vers grille d'indices."""
    largeur = len(lignes[0])
    for l in lignes:
        assert len(l) == largeur, (l, len(l), largeur)
    return [[None if ch == "." else int(ch, 16) for ch in l] for l in lignes]


def poser(t, s, x, y, miroir=False, haut=None, coupe=None):
    """Pose le sprite `s` en (x, y). `haut` : n'en garder que les `haut` premières
    rangées ; `coupe` : ne rien poser sous cette ordonnée de la toile."""
    for j, r in enumerate(s):
        if haut is not None and j >= haut:
            break
        if coupe is not None and y + j >= coupe:
            break
        for i, v in enumerate(reversed(r) if miroir else r):
            if v is not None and 0 <= y + j < len(t) and 0 <= x + i < len(t[0]):
                t[y + j][x + i] = v


def rect(t, x, y, w, h, c):
    for j in range(max(0, y), min(len(t), y + h)):
        for i in range(max(0, x), min(len(t[0]), x + w)):
            t[j][i] = c


def disque(t, cx, cy, r, c):
    for j in range(int(cy - r) - 1, int(cy + r) + 2):
        for i in range(int(cx - r) - 1, int(cx + r) + 2):
            if (i - cx) ** 2 + (j - cy) ** 2 <= r * r and 0 <= j < len(t) and 0 <= i < len(t[0]):
                t[j][i] = c


def contour(s, c=0):
    """Ajoute un contour de 1 px (4 voisins) autour de tout ce qui est plein."""
    h, w = len(s), len(s[0])
    out = [r[:] for r in s]
    for j in range(h):
        for i in range(w):
            if s[j][i] is None and any(
                0 <= j + dj < h and 0 <= i + di < w and s[j + dj][i + di] is not None
                for dj, di in ((0, 1), (0, -1), (1, 0), (-1, 0))
            ):
                out[j][i] = c
    return out


def pale(s):
    """Aperçu de la version pâle du FACILE (comme `dessiner_pale` : une couleur sur
    deux, en damier, remplacée par le brouillard) : ici, juste pour la planche."""
    return [[v if v is None or (i + j) % 2 else 5 for i, v in enumerate(r)] for j, r in enumerate(s)]


# ---------------------------------------------------------------- texte (3 x 5)

POLICE = {
    "A": "010101111101101", "B": "110101110101110", "C": "011100100100011", "D": "110101101101110",
    "E": "111100110100111", "F": "111100110100100", "G": "011100101101011", "H": "101101111101101",
    "I": "111010010010111", "J": "001001001101010", "K": "101101110101101", "L": "100100100100111",
    "M": "101111111101101", "N": "110101101101101", "O": "010101101101010", "P": "110101110100100",
    "Q": "010101101110011", "R": "110101110101101", "S": "011100010001110", "T": "111010010010010",
    "U": "101101101101111", "V": "101101101101010", "W": "101101111111101", "X": "101101010101101",
    "Y": "101101010010010", "Z": "111001010100111", "0": "111101101101111", "1": "010110010010111",
    "2": "110001010100111", "3": "110001010001110", "4": "101101111001001", "5": "111100110001110",
    "6": "011100111101111", "7": "111001010010010", "8": "111101111101111", "9": "111101111001110",
    ":": "000010000010000", ",": "000000000010100", ".": "000000000000010", " ": "000000000000000",
    "(": "010100100100010", ")": "010001001001010", "-": "000000111000000", "+": "000010111010000",
    "/": "001001010100100", "=": "000111000111000", "'": "010010000000000",
}


def ecrire(t, x, y, texte, c=7):
    for k, ch in enumerate(texte.upper()):
        motif = POLICE.get(ch, POLICE[" "])
        for n, b in enumerate(motif):
            if b == "1":
                rect(t, x + k * 4 + n % 3, y + n // 3, 1, 1, c)


# ---------------------------------------------------------------- la TOUR (fond)

L_MUR = 288  # largeur du mur entre les deux bords de la tour (18 tuiles)
H_MUR = 96   # période verticale du mur (6 tuiles) : il se répète en hauteur


def fenetre(lit=True):
    """Fenêtre en plein cintre 10 x 16 : appui de pierre, croisée de bois."""
    s = toile(10, 16)
    for j in range(16):
        for i in range(10):
            dx = i - 4.5
            if j >= 4 or dx * dx + (j - 4.5) ** 2 <= 20:
                s[j][i] = 0
    for j in range(1, 14):
        for i in range(1, 9):
            dx = i - 4.5
            if j >= 4 or dx * dx + (j - 4.5) ** 2 <= 12:
                s[j][i] = (11 if i < 4 else 10) if lit else 1
    if lit:
        for j in range(3, 7):
            s[j][2] = 12
    for j in range(1, 14):
        s[j][4] = s[j][5] = 15 if lit else 0
    for i in range(1, 9):
        s[8][i] = 15 if lit else 0
    rect(s, 0, 14, 10, 1, 3)
    rect(s, 0, 15, 10, 1, 2)
    return s


def lanterne_murale():
    """Lanterne de mur 9 x 14 : la potence de bois, le chapeau, la vitre allumée."""
    return grille(
        "0000.....",
        "0FF0.....",
        "0000000..",
        "...0230..",
        "..022220.",
        "..0CCBB0.",
        "..0CBBA0.",
        "..0CBBA0.",
        "..0BBAA0.",
        "..022220.",
        "...0220..",
        "....00...",
        ".........",
        ".........",
    )


def mur_a():
    """Variante A : mur plein de pierre (assises décalées de 16 x 8, joints sombres,
    arête claire en haut de chaque pierre), deux bandeaux de bois par période, deux
    fenêtres éclairées, une éteinte, une lanterne."""
    t = toile(L_MUR, H_MUR, 1)
    for j in range(H_MUR):
        assise, dans = divmod(j, 8)
        for i in range(L_MUR):
            decal = (i + (8 if assise % 2 else 0)) % 16
            if dans == 7 or decal == 15:
                t[j][i] = 0
            elif dans == 0 and decal < 14:
                t[j][i] = 2
    for y in (40, 88):  # bandeaux de bois (poutres violettes, chevilles)
        rect(t, 0, y, L_MUR, 1, 0)
        rect(t, 0, y + 1, L_MUR, 1, 15)
        rect(t, 0, y + 2, L_MUR, 2, 2)
        rect(t, 0, y + 4, L_MUR, 1, 0)
        for i in range(6, L_MUR, 24):
            t[y + 2][i] = 4
    poser(t, fenetre(True), 40, 12)
    poser(t, fenetre(False), 184, 20)
    poser(t, fenetre(True), 232, 12)
    # la lanterne
    lx, ly = 100, 26
    poser(t, lanterne_murale(), lx, ly)
    return t


def mur_b():
    """Variante B : arcades de pierre ouvertes sur le crépuscule du port (le ciel de
    `port/crepuscule.png`, la mer au loin), piliers et cintres, garde-corps de bois."""
    t = toile(L_MUR, H_MUR, 1)
    ciel = [PAL.index(r[0]) for r in decoder_png(FONDS / "port" / "crepuscule.png")]
    # baies : 3 par période, 72 px de large, 6 px de pilier entre elles
    for k in range(3):
        bx = 12 + k * 96
        for j in range(8, 84):
            for i in range(72):
                dx = i - 35.5
                dedans = j >= 36 or dx * dx + (j - 36) ** 2 <= 36 * 36
                if not dedans:
                    continue
                # le ciel (décalé pour ne garder que nuit, violet et corail), puis la mer
                yc = j - 8 + 4
                if j < 74:
                    t[j][bx + i] = ciel[min(len(ciel) - 1, yc)]
                elif j == 74:
                    t[j][bx + i] = 4
                else:
                    t[j][bx + i] = 2 if (i + j) % 2 else 1
        # cintre (claveaux) et appui
        for j in range(4, 86):
            for i in range(-3, 75):
                dx = i - 35.5
                d2 = dx * dx + (j - 36) ** 2
                bord = (j < 36 and 36 * 36 < d2 <= 39 * 39) or (j >= 36 and i in (-3, -2, -1, 72, 73, 74))
                if bord and 0 <= bx + i < L_MUR:
                    t[j][bx + i] = 3 if (j + i) % 6 else 0
        rect(t, bx - 3, 84, 78, 1, 4)
        rect(t, bx - 3, 85, 78, 2, 3)
        rect(t, bx - 3, 87, 78, 1, 0)
    # garde-corps de bois devant les baies
    for y in (64,):
        rect(t, 0, y, L_MUR, 1, 0)
        rect(t, 0, y + 1, L_MUR, 2, 15)
        rect(t, 0, y + 3, L_MUR, 1, 0)
    for i in range(0, L_MUR, 12):
        rect(t, i, y + 4, 2, 16, 15)
        rect(t, i + 2, y + 4, 1, 16, 0)
    # la lanterne sur le pilier du milieu
    poser(t, lanterne_murale(), 104, 40)
    return t


# ---------------------------------------------------------------- la MER

def ecume_crete(frame):
    return sprite("ecume0" if frame % 2 == 0 else "ecume1")


def dessiner_mer(t, x0, x1, surface, frame, plein):
    """La masse d'eau sous `surface` (ordonnée de la toile), de x0 à x1 (exclu).
    `plein[j][i]` dit si une tuile est dessinée là : sous l'eau, elle passe en
    SILHOUETTE (une teinte sous l'eau qui l'entoure)."""
    crete = ecume_crete(frame)
    for j in range(max(0, surface), len(t)):
        p = j - surface  # profondeur en px
        for i in range(x0, x1):
            damier = (i + j) % 2 == 0
            if p < 3:
                c = 13
            elif p < 5:
                c = 13 if damier else 14
            elif p < 24:
                c = 14
            elif p < 27:
                c = 14 if damier else 2
            elif p < 50:
                c = 2
            elif p < 53:
                c = 2 if damier else 1
            else:
                c = 1
            if plein and plein[j][i]:
                c = {13: 14, 14: 2, 2: 1, 1: 0}[c]
            t[j][i] = c
        # reflets : tirets clairs, plus rares et plus sombres en profondeur
        if 6 <= p < 48 and p % 5 == 1:
            graine = (j * 7919 + frame * 13) % 97
            for i in range(x0 + graine % 23, x1, 29 + p % 7):
                lg = 3 + (i + p) % 5
                c = 13 if p < 24 else 14
                for k in range(lg):
                    if i + k < x1 and not (plein and plein[j][i + k]):
                        t[j][i + k] = c
    # la crête d'écume (4 px) qui dépasse au-dessus de la surface
    for i in range(x0, x1, 16):
        poser(t, crete, i + (frame * 3) % 16 - 16 if False else i, surface - 2)
    # embruns : quelques gouttes blanches au-dessus de la crête
    for i in range(x0 + 5, x1, 37):
        y = surface - 4 - ((i // 37 + frame) % 3) * 2
        if 0 <= y < len(t):
            t[y][i] = 7
            if y - 1 >= 0:
                t[y - 1][i + 1 if i + 1 < x1 else i] = 13


# ---------------------------------------------------------------- la PIEUVRE au loin

def tentacule(points, rayon0, rayon1, w, h, ventouses=True):
    """Un tentacule d'un seul tenant (pas un chapelet) : des disques serrés le long
    de la ligne `points`, du rayon `rayon0` (la base) à `rayon1` (la pointe), puis
    le volume (côté éclairé corail clair à gauche, ombre violette à droite), les
    ventouses or pâle du côté intérieur et le contour."""
    s = toile(w, h)
    pas = []
    for (ax, ay), (bx, by) in zip(points, points[1:]):
        n = max(1, int(math.hypot(bx - ax, by - ay) * 2))
        for k in range(n):
            pas.append((ax + (bx - ax) * k / n, ay + (by - ay) * k / n))
    pas.append(points[-1])
    for k, (x, y) in enumerate(pas):
        r = rayon0 + (rayon1 - rayon0) * k / max(1, len(pas) - 1)
        disque(s, x, y, r, 8)
    vol = [r[:] for r in s]
    for j in range(h):
        for i in range(w):
            if s[j][i] is None:
                continue
            if i + 2 >= w or s[j][i + 2] is None or (i + 1 < w and s[j][i + 1] is None):
                vol[j][i] = 15
            elif i - 1 < 0 or s[j][i - 1] is None:
                vol[j][i] = 9
    if ventouses:
        for k in range(6, len(pas) - 8, 9):
            x, y = pas[k]
            xi, yi = int(round(x)) + 1, int(round(y))
            if 0 <= yi < h and 0 <= xi < w and vol[yi][xi] == 8:
                vol[yi][xi] = 12
    return contour(vol)


def caisse():
    """Caisse de quai 16 x 16, comme `por_caisse` : cadre de bois clair, planches,
    écharpe en diagonale."""
    s = toile(16, 16)
    rect(s, 1, 1, 14, 14, 10)
    for y in (5, 10):
        rect(s, 3, y, 10, 1, 8)
    for k in range(10):
        s[3 + k][3 + k] = 9
        if 4 + k < 13:
            s[3 + k][4 + k] = 8
    rect(s, 1, 1, 14, 2, 9)
    rect(s, 1, 13, 14, 2, 8)
    rect(s, 1, 1, 2, 14, 9)
    rect(s, 13, 1, 2, 14, 8)
    for x, y in ((2, 2), (13, 2), (2, 13), (13, 13)):
        s[y][x] = 4
    return contour(s)[0:16]


def tonneau():
    return grille(
        "...0000000000...",
        "..0444444444430.",
        ".0A9AAA8AAA8A80.",
        ".0A9AAA8AAA8A80.",
        "0444444444444430",
        "0A9AAAA8AAAA8A80",
        "0A9AAAA8AAAA8A80",
        "0A9AA0A8AAAA8A80",
        "0A9AAAA8AAAA8A80",
        "0A9AAAA8AAAA8A80",
        "0333333333333320",
        ".0A9AAA8AAA8A80.",
        ".0A9AAA8AAA8A80.",
        "..0333333333320.",
        "...0000000000...",
        "................",
    )


def epave():
    """Un morceau d'épave : trois bordés cassés en dents de scie, une membrure de
    bois sombre, un hublot de laiton et sa vitre, un bout de cordage."""
    return grille(
        "..0.....0.......",
        ".0A0...0A0..0...",
        ".0AA0.0AA0.0A0..",
        "0A9AA0A9A00AA0..",
        "0A9AAAA9AAAA9A0.",
        "0A9AA8A9A8AA9A80",
        "0FFFFFFFFFFFFFF0",
        "0A9A0000A8AA9A80",
        "0A90BBBB0A8A9A80",
        "0A90BDDB0AA89A80",
        "0A90BDDB0A8A9A80",
        "0A9A0BB0AA8A9A80",
        "0FFFF00FFFFFFFF0",
        "0A8A8A8A8A8A8A80",
        ".000000000000000",
        "................",
    )


def repere_annonce(frame):
    """Le repère d'annonce, dans le style de l'interface (comme les icônes du
    compteur : contour sombre, or, blanc) : un écriteau « ! » dont la pointe montre
    la colonne où l'objet tombera. Deux images (il clignote, plus vite à la fin)."""
    a = grille(
        ".000000.",
        "0BBBBBB0",
        "0BB00BB0",
        "0BB00BB0",
        "0BB00BB0",
        "0BBBBBB0",
        "0BB00BB0",
        "0BBBBBB0",
        ".0BBBB0.",
        "..0BB0..",
        "...00...",
    )
    if frame % 2:
        a = [[{11: 12}.get(v, v) for v in r] for r in a]
    return a


# ---------------------------------------------------------------- la FUITE

def encre(rayons, w, h):
    """Nuage d'encre violette (des bulles d'un seul tenant, contour sombre)."""
    s = toile(w, h)
    for x, y, r in rayons:
        disque(s, x, y, r, 15)
    for x, y, r in rayons:
        if r >= 3:
            s[int(y - r / 2)][int(x - r / 2)] = 3
    return contour(s)


def bulles(t, x, y, frame):
    for k, (dx, dy) in enumerate(((0, 0), (6, -5), (12, -2), (3, -10))):
        yy = y + dy - (frame * 2 + k) % 4
        if 0 <= yy < len(t) - 2:
            poser(t, grille(".0.", "0D0", ".0."), x + dx, yy)


# ---------------------------------------------------------------- montage

def capture_reduite(chemin):
    """La capture x3 réduite à 320 x 180 (un pixel sur trois : le jeu agrandit sans
    flou, chaque carré 3 x 3 est d'une seule couleur)."""
    g = decoder_png(chemin)
    return [[PAL.index(g[j * 3 + 1][i * 3 + 1]) for i in range(320)] for j in range(180)]


NIVEAU = (RACINE / "assets" / "levels" / "monde_2" / "pieuvre.ron").read_text()
ASCII = re.search(r'ascii: r#"\n(.*?)"#', NIVEAU, re.S).group(1).splitlines()


PLEINS = set("#B_F")


def plein_du_niveau(x, y):
    """La case de la grille au point (x, y) du niveau est-elle une tuile ?"""
    r, c = int(y // 16), int(x // 16)
    return 0 <= r < len(ASCII) and 0 <= c < 20 and ASCII[r][c] in PLEINS


def caler_camera(base):
    """Le haut de la caméra (px du niveau) de la capture : celui qui fait le mieux
    coïncider les tuiles de la grille avec ce qui n'est pas du ciel
    (indice 0) dans la capture (le niveau fait 320 px de large : la caméra ne bouge qu'en y)."""
    points = [(x, y) for y in range(16, 140, 4) for x in range(18, 302, 4)]
    meilleur, cy_ok = -1, 0
    for cy in range(0, len(ASCII) * 16 - 180):
        score = sum(plein_du_niveau(x, y + cy) == (base[y][x] != 0) for x, y in points)
        if score > meilleur:
            meilleur, cy_ok = score, cy
    return cy_ok


def est_du_ciel(base, x, y):
    """Le ciel de nuit de la capture (l'ombre d'ardoise, indice 0), et ses étoiles
    et gouttes de pluie isolées (un pixel clair entouré de nuit). Un pixel sombre
    qui touche un dessin (le contour de Pépin, d'une lanterne, d'une bitte) n'est
    pas du ciel : on le garde."""
    def isole(xx, yy):
        voisins = [base[yy + dy][xx + dx] for dy in (-1, 0, 1) for dx in (-1, 0, 1)
                   if (dx or dy) and 0 <= yy + dy < 180 and 0 <= xx + dx < 320]
        zeros = sum(n == 0 for n in voisins)
        # une étoile ou une goutte (2 px) : au moins 7 voisins de nuit ; la trame
        # d'ardoise de nuit sur la nuit : au moins 4 (les lettres de l'interface,
        # traits de 1 px, en ont moins : elles restent)
        return (base[yy][xx] in (3, 4, 5, 6, 7, 13, 14) and zeros >= 7) or (
            base[yy][xx] == 1 and zeros >= 4
        )
    v = base[y][x]
    if v != 0:
        return isole(x, y)
    for dy in (-1, 0, 1):
        for dx in (-1, 0, 1):
            xx, yy = x + dx, y + dy
            if 0 <= yy < 180 and 0 <= xx < 320 and base[yy][xx] != 0 and not isole(xx, yy):
                return False
    return True


def surface_de_la_capture(base):
    """La surface de la mer dans la capture : la première rangée du bas de l'écran
    où la crête d'écume (cyan sombre, nuage, blanc) couvre beaucoup de colonnes."""
    for y in range(120, 180):
        if sum(base[y][x] in (6, 7, 13, 14) for x in range(16, 304)) >= 40:
            return y
    return 170


def montage(mur, variante, frame=0):
    """Une VRAIE capture de la carte 9.36a (la montée, `normal-tour-milieu.png`),
    où l'on remplace : le ciel de nuit vide par le mur de la tour ; la mer par la
    nouvelle masse d'eau ; et où l'on ajoute la Pieuvre au loin qui lance, un objet
    annoncé (repère + ombre) et un tonneau qui tombe. Tuiles, Pépin, lanternes,
    jauge et interface sont ceux de la capture, au pixel près."""
    base = capture_reduite(SORTIE / "936" / "normal-tour-milieu.png")
    cy = caler_camera(base)
    surface = surface_de_la_capture(base)
    t = [r[:] for r in base]
    plein = [[plein_du_niveau(x, y + cy) for x in range(320)] for y in range(180)]
    # 1. le mur, qui défile avec la caméra, partout où l'on voyait le ciel ; et, au
    #    ras de la mer, à la place du port d'en face (le mur le cache)
    for y in range(0, surface):
        for x in range(16, 304):
            if plein[y][x]:
                continue
            if y < 16 and (x < 90 or x > 270) and base[y][x] != 0:
                continue  # l'interface (gouttes, vies, chrono) : celle de la capture
            bas_libre = y >= surface - 24 and not any(plein[k][x] for k in range(surface - 24, surface))
            if est_du_ciel(base, x, y) or bas_libre:
                t[y][x] = mur[(cy + y) % H_MUR][x - 16]
    # 2. la mer : la nouvelle masse d'eau depuis la surface de la capture
    dessiner_mer(t, 16, 304, surface + 2, frame, plein)
    # 3. la Pieuvre au loin, à droite, à moitié dans l'eau, qui lance une caisse
    tete = sprite("pieuvre_colere0")
    hx, hy = 252, surface + 2 - 16
    poser(t, tete, hx, hy, coupe=surface + 3)
    bras = tentacule([(10, 40), (8, 28), (11, 16), (20, 8), (30, 5)], 4.5, 2, 36, 46)
    poser(t, bras, hx - 16, hy - 26, coupe=surface + 2)
    poser(t, caisse(), hx + 22, hy - 72)
    poser(t, grille("7", ".", "7", "7"), hx + 22, hy - 54)
    poser(t, grille("7", "7", ".", "7"), hx + 30, hy - 50)
    poser(t, sprite("remous_pieuvre0"), hx + 8, surface - 1)
    poser(t, sprite("remous_pieuvre0"), hx - 18, surface - 1)
    # 4. un objet ANNONCÉ : son ombre sur la corniche de Pépin et le repère en haut
    #    de sa colonne (le sol est la première tuile sous le haut de l'écran)
    def sol(x, depuis):
        return next(y for y in range(depuis, 180) if plein[y][x] and plein[y][x + 15])
    xa = 150
    poser(t, repere_annonce(frame), xa + 4, 16)
    poser(t, sprite("ombre_caisse"), xa, sol(xa, 20) - 2)
    # 5. un TONNEAU qui tombe (traits de vitesse) au-dessus de son ombre
    xb = 96
    yb = 62
    poser(t, tonneau(), xb, yb)
    poser(t, grille("7", ".", "7", "7", ".", "7"), xb + 3, yb - 11)
    poser(t, grille("7", "7", ".", "7"), xb + 11, yb - 8)
    poser(t, sprite("ombre_caisse"), xb, sol(xb, yb + 16) - 2)
    ecrire(t, 122, 172, "VARIANTE " + variante, 7)
    return t


# ---------------------------------------------------------------- planche

def cadre(t, x, y, w, h, titre):
    rect(t, x, y, w, h, 2)
    rect(t, x, y, w, 1, 4)
    ecrire(t, x + 3, y + 3, titre, 7)


def planche_elements():
    t = toile(560, 470, 1)
    ecrire(t, 4, 4, "CARTE 9.36B : PROPOSITION DU DESSIN DE LA PHASE 2 (PLANCHE X3)", 11)

    # 1. la tour, deux variantes (un morceau de 144 x 96 de chaque)
    cadre(t, 4, 14, 552, 112, "1. LA TOUR : MUR DERRIERE LES CORNICHES (A : MUR PLEIN, B : ARCADES SUR LE PORT)")
    for k, mur in enumerate((mur_a(), mur_b())):
        for j in range(H_MUR):
            for i in range(264):
                t[24 + j][8 + k * 276 + i] = mur[j][i]
        poser(t, sprite("por_quai_g"), 8 + k * 276 + 48, 24 + 50)
        for n in range(1, 6):
            poser(t, sprite("por_quai"), 8 + k * 276 + 48 + n * 16, 24 + 50)
        poser(t, sprite("por_quai_d"), 8 + k * 276 + 48 + 96, 24 + 50)
        poser(t, sprite("pepin_idle0"), 8 + k * 276 + 90, 24 + 34)
        ecrire(t, 8 + k * 276, 121, "A" if k == 0 else "B", 11)

    # 2. la mer
    cadre(t, 4, 130, 552, 84, "2. LA MER QUI MONTE : CRETE, REFLETS, PROFONDEUR, CORNICHES NOYEES EN SILHOUETTE (2 IMAGES)")
    for k in range(2):
        z = toile(268, 66, None)
        plein = [[False] * 268 for _ in range(66)]
        for n in range(6):
            poser(z, sprite("por_quai" if 0 < n < 5 else "por_quai_g" if n == 0 else "por_quai_d"), 40 + n * 16, 30)
            for j in range(16):
                for i in range(16):
                    plein[30 + j][40 + n * 16 + i] = True
        rect_fond = [[1 if v is None else v for v in r] for r in z]
        dessiner_mer(rect_fond, 0, 268, 18, k, plein)
        poser(t, rect_fond, 8 + k * 276, 142)

    # 3. la pieuvre au loin qui lance (repos, prépare, lance) + pâle (facile)
    cadre(t, 4, 218, 360, 100, "3. LA PIEUVRE AU LOIN : REPOS, PREPARE, LANCE  (+ FACILE)")
    tete = sprite("pieuvre_colere0")
    gestes = [
        ("REPOS", tentacule([(10, 40), (12, 34), (16, 31)], 4.5, 2.5, 36, 46), None),
        ("PREPARE", tentacule([(10, 40), (6, 28), (8, 16), (18, 9), (26, 12)], 4.5, 2, 36, 46), (14, -2)),
        ("LANCE", tentacule([(10, 40), (8, 28), (11, 16), (20, 8), (30, 5)], 4.5, 2, 36, 46), (36, -14)),
    ]
    for k, (nom, bras, objet) in enumerate(gestes):
        z = toile(84, 80, 1)
        surface = 62
        dessiner_mer(z, 0, 84, surface, k, None)
        poser(z, tete, 30, surface - 16, coupe=surface + 1)
        poser(z, bras, 14, surface - 42, coupe=surface)
        if objet:
            poser(z, caisse(), 14 + objet[0], surface - 42 + objet[1] if objet[1] > -10 else 2)
        poser(z, sprite("remous_pieuvre0"), 36, surface - 3)
        poser(t, z, 8 + k * 88, 230)
        ecrire(t, 8 + k * 88, 312, nom, 7)
    z = toile(84, 80, 1)
    dessiner_mer(z, 0, 84, 62, 0, None)
    poser(z, pale(tete), 30, 46, coupe=63)
    poser(z, pale(gestes[2][1]), 14, 20, coupe=62)
    poser(t, z, 272, 230)
    ecrire(t, 272, 312, "FACILE (PALE)", 7)

    # 4. les objets, l'ombre, le repère d'annonce
    cadre(t, 368, 218, 188, 100, "4. OBJETS ET ANNONCE")
    objets = [("CAISSE", caisse()), ("TONNEAU", tonneau()), ("EPAVE", epave())]
    for k, (nom, s) in enumerate(objets):
        poser(t, s, 376 + k * 40, 232)
        poser(t, pale(s), 376 + k * 40, 252)
        ecrire(t, 374 + k * 40, 270, nom[:7], 7)
    ecrire(t, 496, 232, "PALE", 5)
    ecrire(t, 496, 256, "(FACILE)", 5)
    for k in range(2):
        poser(t, repere_annonce(k), 376 + k * 14, 282)
    rect(t, 410, 296, 48, 16, 1)
    poser(t, sprite("por_quai"), 410, 300)
    poser(t, sprite("por_quai"), 426, 300)
    poser(t, sprite("por_quai"), 442, 300)
    poser(t, sprite("ombre_caisse"), 426, 298)
    ecrire(t, 466, 286, "REPERE (2 IM.)", 7)
    ecrire(t, 466, 296, "+ OMBRE SUR", 7)
    ecrire(t, 466, 303, "LE SOL", 7)

    # 5. la fuite, 4 images
    cadre(t, 4, 322, 552, 104, "5. LA FUITE : SURPRISE, ENCRE, PLONGEE, PARTIE (4 IMAGES)")
    touchee = sprite("pieuvre_touchee")
    for k in range(4):
        z = toile(128, 80, 1)
        surface = 52
        dessiner_mer(z, 0, 128, surface, k, None)
        if k == 0:
            poser(z, touchee, 40, surface - 18, coupe=surface + 1)
            poser(z, grille(".0.", "0D0", "0D0", ".0."), 86, surface - 24)
        elif k == 1:
            poser(z, encre([(30, surface - 4, 7), (20, surface - 7, 5), (38, surface - 10, 4)], 128, 80), 0, 0, coupe=surface + 1)
            poser(z, touchee, 46, surface - 10, coupe=surface + 1)
            poser(z, tentacule([(8, 30), (10, 20), (6, 10), (10, 4)], 3.5, 1.8, 20, 36), 96, surface - 30, coupe=surface)
        elif k == 2:
            poser(z, encre([(34, surface - 2, 9), (22, surface - 6, 6), (48, surface - 6, 6), (60, surface - 3, 5)], 128, 80), 0, 0, coupe=surface + 1)
            poser(z, touchee, 52, surface - 3, coupe=surface + 1)
            bulles(z, 64, surface - 10, k)
        else:
            poser(z, sprite("remous_pieuvre1"), 48, surface - 3)
            poser(z, sprite("remous_pieuvre0"), 76, surface - 3)
            for i in range(30, 70):
                if i % 2 == 0:
                    z[surface + 2][i] = 15
            bulles(z, 70, surface - 8, k)
        poser(t, z, 8 + k * 136, 334)
        ecrire(t, 8 + k * 136, 416, str(k + 1), 7)

    # 6. légende
    ecrire(t, 4, 432, "SEULES LES 16 COULEURS DU JEU, CONTOUR SOMBRE DE 1 PX, AUCUN FLOU.", 5)
    ecrire(t, 4, 440, "EXISTANTS REPRIS : TETE DE LA PHASE 1, TUILES DU PORT, PEPIN, ECUME, OMBRE, REMOUS.", 5)
    ecrire(t, 4, 448, "NOUVEAUX : MUR (A OU B), MASSE D'EAU, TENTACULE QUI LANCE, CAISSE, TONNEAU, EPAVE,", 5)
    ecrire(t, 4, 456, "REPERE, ENCRE. MONTAGES : 936-PROPOSITION-SCENE-A.PNG ET -B.PNG", 5)
    return t


def main():
    SORTIE.mkdir(parents=True, exist_ok=True)
    sorties = {
        "936-proposition-planche.png": planche_elements(),
        "936-proposition-scene-a.png": montage(mur_a(), "A"),
        "936-proposition-scene-b.png": montage(mur_b(), "B"),
    }
    for nom, g in sorties.items():
        ecrire_png(SORTIE / nom, g, ECHELLE)
        print(SORTIE / nom, len(g[0]) * ECHELLE, "x", len(g) * ECHELLE)


if __name__ == "__main__":
    sys.exit(main())
