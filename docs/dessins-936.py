#!/usr/bin/env python3
"""Les DESSINS de la nouvelle phase 2 de la Pieuvre (carte 9.36b, étape 2) : la source
des nouveaux PNG du jeu, d'après la planche acceptée par AGA (variante B, « avec un peu
de A »).

Ce script écrit :
- les couches de la TOUR (`assets/fonds/port/tour_bas.png`, `tour_milieu.png`,
  `tour_haut.png`, 288 x 96 chacune, une période du mur) : des arcades ouvertes sur le
  port (B) entre des PANS DE MUR PLEINS (A) avec leurs fenêtres éclairées et leurs
  lanternes ; plus de mur plein en bas, plus d'arcades en haut ; la baie de droite
  (là où la Pieuvre se tient au loin) est toujours un pan plein, sans couleur chaude ;
  la vue dans les arcades est un crépuscule FROID et sombre (nuit, violet, un fil de
  corail à l'horizon), et l'horizon descend dans l'arcade à mesure qu'on monte ;
- les SPRITES (`assets/sprites/pieuvre/pieuvre_*.png`) : la mer (masse d'eau, crête,
  corniches noyées), les trois gestes du bras qui lance, la caisse, le tonneau,
  l'épave, leurs éclats, la traînée de la chute, le repère « ! », l'encre, les bulles
  et la goutte de la fuite ;
- les images de QA : `docs/qa/936-melange-scene.png` (le mélange B + A monté sur une
  VRAIE capture de la 9.36a) et `docs/qa/936b-planche-sprites.png` (tous les nouveaux
  dessins, x3).

Les PNG du jeu passent par l'outil `sprite` du dépôt (`target/release/sprite ecrire`,
le format canonique) : construire l'outil d'abord
(`./docker-cargo.sh build -p outils --bin sprite --release`).

Usage : python3 docs/dessins-936.py            (tout)
        python3 docs/dessins-936.py --scene    (la scène du mélange seulement)
Python seul, sans bibliothèque. Seules les 16 couleurs de la palette, aucun flou.
"""
import importlib.util
import subprocess
import sys
from pathlib import Path

ICI = Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location("planche936", ICI / "planche-936.py")
pl = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(pl)

RACINE = pl.RACINE
OUTIL = RACINE / "target" / "release" / "sprite"
toile, grille, rect, poser, contour = pl.toile, pl.grille, pl.rect, pl.poser, pl.contour

L_MUR, H_MUR = 288, 96


# ---------------------------------------------------------------- la TOUR

def maconnerie(t, x0, x1, y0, y1):
    """Le mur de pierre de la A : assises de 8 px, pierres de 16 décalées d'une assise
    à l'autre, joints d'ombre, arête d'ardoise en haut de chaque pierre. Le motif ne
    dépend que de (x, y) : deux couches se raccordent toujours."""
    for j in range(y0, y1):
        assise, dans = divmod(j, 8)
        for i in range(x0, x1):
            decal = (i + (8 if assise % 2 else 0)) % 16
            if dans == 7 or decal == 15:
                t[j][i] = 0
            elif dans == 0 and decal < 14:
                t[j][i] = 2
            else:
                t[j][i] = 1


def bandeau(t, x0, x1, y):
    """Une poutre de bois sur le mur (5 rangées), chevilles d'ardoise pâle."""
    rect(t, x0, y, x1 - x0, 1, 0)
    rect(t, x0, y + 1, x1 - x0, 1, 15)
    rect(t, x0, y + 2, x1 - x0, 2, 2)
    rect(t, x0, y + 4, x1 - x0, 1, 0)
    for i in range(x0 + 6, x1, 24):
        t[y + 2][i] = 4


def fenetre(allumee):
    """Fenêtre en plein cintre 8 x 14 : cadre sombre, croisée de bois violet, appui de
    pierre. Allumée : une lueur corail et orange, un seul pixel d'or (petite : elle
    ne doit pas manger le contraste des objets qui tombent devant). Éteinte : vitre de
    nuit, un reflet de lune cyan sombre (froide : derrière la Pieuvre)."""
    lignes = [
        "..0000..",
        ".0aaaa0.",
        "0abbbba0",
        "0bcbFbb0",
        "0bbbFbb0",
        "0FFFFFF0",
        "0bbbFbb0",
        "0bbbFbb0",
        "0bbbFbb0",
        "0ddbFbb0",
        "0ddbFbb0",
        "00000000",
        "33333333",
        "22222222",
    ]
    if allumee:
        # corail, un pixel d'or en haut, un coin orange en bas (la lampe)
        couleurs = {"a": "8", "b": "8", "c": "B", "d": "A"}
    else:
        couleurs = {"a": "1", "b": "1", "c": "E", "d": "1"}
    return grille(*["".join(couleurs.get(c, c) for c in l) for l in lignes])


def lanterne():
    """La lanterne de mur de la A (9 x 12) : potence de bois, chapeau, vitre allumée
    (or, orange, corail : pas d'or pâle, elle ne doit pas éblouir les objets)."""
    return grille(
        "0000.....",
        "0FF0.....",
        "0000000..",
        "...0230..",
        "..022220.",
        "..0BAA80.",
        "..0AA880.",
        "..0A8880.",
        "..022220.",
        "...0220..",
        "....00...",
        ".........",
    )


def vue_froide(t, bx, horizon, port, graine):
    """La vue dans une arcade (72 px de large, de y = 8 à 84) : un crépuscule FROID,
    un cran plus sombre que le ciel du port (ombre, nuit, violet, un fil de corail à
    l'horizon), quelques étoiles, puis la mer de nuit. `port` : la silhouette du port
    d'en face sur l'horizon (en bas de la tour seulement)."""
    for j in range(8, 84):
        for i in range(72):
            dx = i - 35.5
            if not (j >= 36 or dx * dx + (j - 36) ** 2 <= 36 * 36):
                continue
            d = horizon - j  # hauteur au-dessus de l'horizon
            damier = (i + j) % 2 == 0
            if d > 30:
                c = 0
            elif d > 27:
                c = 0 if damier else 1
            elif d > 14:
                c = 1
            elif d > 11:
                c = 1 if damier else 15
            elif d > 1:
                c = 15
            elif d == 1:
                c = 8 if i % 4 else 15  # le fil de corail, pointillé
            elif d == 0:
                c = 2
            elif d > -3:
                c = 2 if damier else 1
            else:
                c = 1
            t[j][bx + i] = c
    # étoiles (un pixel), seulement dans la nuit haute
    h = graine
    for _ in range(9):
        h = (h * 1103515245 + 12345) & 0x7FFFFFFF
        i, j = 4 + h % 64, 10 + (h >> 8) % max(1, horizon - 34)
        dx = i - 35.5
        if (j >= 36 or dx * dx + (j - 36) ** 2 <= 33 * 33) and t[j][bx + i] in (0, 1):
            t[j][bx + i] = 4 if (h >> 3) % 3 else 6
    if port:
        # entrepôts bas, deux grues, sur l'horizon : ombre d'ardoise
        y = horizon - 1
        for x0, w, hh in ((4, 14, 3), (22, 10, 5), (44, 18, 4), (64, 6, 2)):
            rect(t, bx + x0, y - hh + 1, w, hh, 0)
        for x0, hh in ((34, 12), (56, 9)):
            rect(t, bx + x0, y - hh, 1, hh + 1, 0)
            rect(t, bx + x0 - 9, y - hh, 13, 1, 0)
            rect(t, bx + x0 - 7, y - hh + 1, 1, 4, 2)
        for x0 in (8, 26, 48):
            t[y - 1][bx + x0] = 3  # fenêtres éteintes du quai lointain


def arcade(t, bx, horizon, port, graine):
    """Une arcade de la B : la vue, le cintre de claveaux, l'appui, le garde-corps."""
    vue_froide(t, bx, horizon, port, graine)
    for j in range(4, 86):
        for i in range(-3, 75):
            dx = i - 35.5
            d2 = dx * dx + (j - 36) ** 2
            bord = (j < 36 and 36 * 36 < d2 <= 39 * 39) or (j >= 36 and i in (-3, -2, -1, 72, 73, 74))
            if bord and 0 <= bx + i < L_MUR:
                t[j][bx + i] = 3 if (j + i) % 6 else 0
    # l'appui : arête d'ardoise claire (pas d'ardoise pâle : trop clair derrière
    # un objet qui tombe), pierre d'ardoise, joint d'ombre
    rect(t, bx - 3, 84, 78, 1, 3)
    rect(t, bx - 3, 85, 78, 2, 2)
    rect(t, bx - 3, 87, 78, 1, 0)
    # garde-corps de bois devant la baie : main courante et balustres
    y = 66
    rect(t, bx, y, 72, 1, 0)
    rect(t, bx, y + 1, 72, 2, 15)
    rect(t, bx, y + 3, 72, 1, 0)
    for i in range(bx + 4, bx + 72, 12):
        rect(t, i, y + 4, 2, 14, 15)
        rect(t, i + 2, y + 4, 1, 14, 0)


def pan_plein(t, bx, allumee, avec_lanterne):
    """Un pan de mur plein de la A sur une baie (96 px) : un bandeau de bois au milieu,
    deux fenêtres (allumées ou éteintes) et une lanterne."""
    bandeau(t, bx, bx + 96, 40)
    poser(t, fenetre(allumee), bx + 14, 14)
    poser(t, fenetre(False), bx + 70, 54)
    if avec_lanterne:
        poser(t, lanterne(), bx + 48, 52)


def couche(baies, horizon, graine):
    """Une période du mur (288 x 96) : trois baies de 96 px, chacune « arcade » ou
    « plein »/« sombre » ; le bandeau du bas (y = 88) est commun à toutes les couches
    (elles se raccordent dans n'importe quel ordre)."""
    t = toile(L_MUR, H_MUR, 1)
    maconnerie(t, 0, L_MUR, 0, H_MUR)
    for k, b in enumerate(baies):
        bx = k * 96
        if b == "arcade":
            arcade(t, bx + 12, horizon, horizon < 64, graine + 31 * k)
        elif b == "plein":
            pan_plein(t, bx, True, True)
        elif b == "plein_sans_lanterne":
            pan_plein(t, bx, True, False)
        elif b == "sombre":
            # la baie de la Pieuvre : pierre, fenêtres éteintes, rien de chaud
            bandeau(t, bx, bx + 96, 40)
            poser(t, fenetre(False), bx + 30, 14)
            poser(t, fenetre(False), bx + 62, 54)
    bandeau(t, 0, L_MUR, 88)
    return t


COUCHES = {
    # en bas : une arcade sur trois (celle du milieu), l'horizon haut dans la baie
    "tour_bas": (["plein", "arcade", "sombre"], 58, 7),
    # au milieu : une arcade sur trois, à gauche (elles alternent avec la couche du bas)
    "tour_milieu": (["arcade", "plein", "sombre"], 68, 19),
    # en haut : deux arcades sur trois, l'horizon tout en bas (on est haut)
    "tour_haut": (["arcade", "arcade", "sombre"], 80, 43),
}


def couches():
    return {nom: couche(*v) for nom, v in COUCHES.items()}


def couche_de_la_periode(k, n):
    """La couche de la période `k` (0 = la plus basse) sur `n` : le MÊME choix que le
    jeu (`dessin_tour::couche_de_la_periode`)."""
    f = k / max(1, n)
    if k >= n - 1:
        return "tour_haut"
    if f < 0.4:
        return "tour_bas" if k % 2 == 0 else "tour_milieu"
    if f < 0.7:
        return "tour_milieu" if k % 2 == 0 else "tour_haut"
    return "tour_haut"


# ---------------------------------------------------------------- la MER

def bande_mer(p, damier):
    """Couleur de l'eau à `p` px sous la surface (le dégradé de la planche)."""
    if p < 3:
        return 13
    if p < 5:
        return 13 if damier else 14
    if p < 24:
        return 14
    if p < 27:
        return 14 if damier else 2
    if p < 50:
        return 2
    if p < 53:
        return 2 if damier else 1
    return 1


def mer(frame):
    """La masse d'eau sous la surface (64 x 56, se répète en largeur) : le dégradé
    cyan, cyan sombre, ardoise, nuit, et des reflets en tirets (deux images)."""
    w, h = 64, 56
    s = [[bande_mer(j, (i + j) % 2 == 0) for i in range(w)] for j in range(h)]
    for j in range(6, 48):
        if j % 5 != 1:
            continue
        graine = (j * 7919 + frame * 13) % 97
        pas = 29 + j % 7
        for depart in range(graine % 23, w, pas):
            lg = 3 + (depart + j) % 5
            for k in range(lg):
                s[j][(depart + k) % w] = 13 if j < 24 else 14
    return s


def mer_noyee():
    """Une corniche sous l'eau, en SILHOUETTE (16 x 56) : le même dégradé, un cran
    plus sombre (le jeu en découpe la tranche à la profondeur de la case)."""
    plus_sombre = {13: 14, 14: 2, 2: 1, 1: 0}
    return [[plus_sombre[bande_mer(j, (i + j) % 2 == 0)] for i in range(16)] for j in range(56)]


def crete(frame):
    """La crête d'écume (64 x 6) posée sur la surface (ses 3 dernières rangées sur
    l'eau), DEVANT la Pieuvre : écume blanche et nuage, la vague d'une période de
    16 px, décalée de 4 px d'une image à l'autre."""
    w = 64
    s = toile(w, 6)
    profil = [0, 1, 1, 2, 2, 2, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0]
    for i in range(w):
        haut = profil[(i + frame * 4) % 16]
        for j in range(3 - haut, 6):
            s[j][i] = 6
        s[3 - haut][i] = 7
        if (i * 7 + frame * 3) % 11 == 0:
            s[min(5, 4 - haut)][i] = 7
        s[5][i] = 13 if (i + frame) % 3 else 7
    return s


# ---------------------------------------------------------------- la PIEUVRE

def bras(points, rayon0=4.5, rayon1=2.0):
    """Le bras qui lance (36 x 42, le bas sur la surface) : le tentacule d'un seul
    tenant de la planche."""
    t = pl.tentacule(points, rayon0, rayon1, 36, 46)
    return t[:42]


BRAS = {
    "pieuvre_bras_repos": [(10, 40), (12, 34), (16, 31)],
    "pieuvre_bras_prepare": [(10, 40), (6, 28), (8, 16), (18, 9), (26, 12)],
    "pieuvre_bras_lance": [(10, 40), (8, 28), (11, 16), (20, 8), (30, 5)],
}


def recadrer(s):
    """Retire les rangées et colonnes vides du bord (rend la grille et son décalage)."""
    lignes = [j for j, r in enumerate(s) if any(v is not None for v in r)]
    cols = [i for i in range(len(s[0])) if any(r[i] is not None for r in s)]
    return [r[cols[0]:cols[-1] + 1] for r in s[lignes[0]:lignes[-1] + 1]], cols[0], lignes[0]


def encre(rayons, w, h):
    """L'encre de la fuite (violet, contour sombre), coupée à la surface : sa
    dernière rangée (`h - 1`) est posée sur l'eau."""
    s = pl.encre(rayons, w, h + 12)
    return s[:h]


def eclats(base):
    """Les éclats d'un objet brisé (16 x 6) : des morceaux de ses couleurs."""
    s = toile(16, 6)
    morceaux = [(1, 3, 3, 2), (5, 1, 2, 2), (8, 4, 3, 1), (11, 2, 2, 2), (13, 4, 2, 1), (3, 5, 2, 1)]
    for k, (x, y, w, h) in enumerate(morceaux):
        rect(s, x, y, w, h, base[k % len(base)])
    return contour(s)


def trainee():
    """La traînée d'un objet qui tombe (16 x 10, au-dessus de lui) : deux filets
    blancs (les traits de vitesse de la planche)."""
    s = toile(16, 10)
    for j in (0, 2, 3, 5, 6, 8, 9):
        s[j][3] = 7
    for j in (3, 4, 6, 8, 9):
        s[j][11] = 7
    for j in (6, 7, 9):
        s[j][7] = 6
    return s


def bulles(frame):
    s = toile(16, 14)
    for k, (dx, dy) in enumerate(((0, 10), (6, 5), (12, 8), (3, 0))):
        y = dy + 2 - (frame * 2 + k) % 3
        poser(s, grille(".0.", "0D0", ".0."), dx, max(0, y))
    return s


def sprites():
    """Tous les nouveaux sprites : nom -> grille d'indices (None : transparent)."""
    d = {
        "pieuvre_mer0": mer(0),
        "pieuvre_mer1": mer(1),
        "pieuvre_mer_fond": [[1] * 16 for _ in range(16)],
        "pieuvre_mer_noyee": mer_noyee(),
        "pieuvre_mer_noyee_fond": [[0] * 16 for _ in range(16)],
        "pieuvre_crete0": crete(0),
        "pieuvre_crete1": crete(1),
        "pieuvre_caisse": pl.caisse(),
        "pieuvre_tonneau": pl.tonneau(),
        "pieuvre_epave": pl.epave(),
        "pieuvre_eclats_caisse": eclats([10, 9, 8]),
        "pieuvre_eclats_tonneau": eclats([10, 4, 9, 3]),
        "pieuvre_eclats_epave": eclats([10, 15, 9, 11]),
        "pieuvre_trainee": trainee(),
        "pieuvre_repere0": pl.repere_annonce(0),
        "pieuvre_repere1": pl.repere_annonce(1),
        # la fuite : l'encre à gauche de la tête (le bas sur la surface, rangée 15)
        "pieuvre_encre0": encre([(15, 11, 7), (7, 8, 5), (23, 5, 4)], 34, 16),
        "pieuvre_encre1": encre([(16, 13, 9), (8, 9, 6), (30, 9, 6), (42, 12, 5)], 50, 16),
        "pieuvre_encre2": grille(
            "F.F.F.FF.F..F.F.FF.F.F..F.F.F.FF",
            ".3F3F3.F3F33F3F3.F3F3F33F3F.F3F.",
        ),
        "pieuvre_bulles0": bulles(0),
        "pieuvre_bulles1": bulles(1),
        "pieuvre_goutte": grille(".0.", "0D0", "0D0", ".0."),
    }
    for nom, pts in BRAS.items():
        d[nom] = bras(pts)
    return d


# ---------------------------------------------------------------- écriture

def texte(g):
    return "\n".join("".join("." if v is None else "0123456789ABCDEF"[v] for v in r) for r in g) + "\n"


def ecrire_asset(chemin, g):
    chemin.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run([str(OUTIL), "ecrire", str(chemin)], input=texte(g).encode(), check=True)


def scene_melange():
    """Le mélange B + A dans UNE scène de montée (x3) : la vraie capture de la 9.36a
    (`normal-tour-milieu.png`), son ciel remplacé par les couches de la tour posées
    comme dans le jeu, la nouvelle mer, la Pieuvre au loin qui lance, un objet annoncé
    et une caisse qui tombe devant une arcade."""
    base = pl.capture_reduite(pl.SORTIE / "936" / "normal-tour-milieu.png")
    cy = pl.caler_camera(base)
    surface = pl.surface_de_la_capture(base)
    t = [r[:] for r in base]
    plein = [[pl.plein_du_niveau(x, y + cy) for x in range(320)] for y in range(180)]
    murs = couches()
    bache = 68 * 16
    n = -(-bache // H_MUR)
    for y in range(0, surface):
        yn = y + cy
        if yn >= bache:
            continue
        k = (bache - 1 - yn) // H_MUR
        mur = murs[couche_de_la_periode(k, n)]
        ym = H_MUR - 1 - ((bache - 1 - yn) % H_MUR)
        for x in range(16, 304):
            if plein[y][x]:
                continue
            if y < 16 and (x < 90 or x > 270) and base[y][x] != 0:
                continue
            bas_libre = y >= surface - 24 and not any(plein[k2][x] for k2 in range(surface - 24, surface))
            if pl.est_du_ciel(base, x, y) or bas_libre:
                t[y][x] = mur[ym][x - 16]
    sp = sprites()
    s = surface + 2
    # la mer : la masse d'eau, les corniches noyées en silhouette
    for y in range(s, 180):
        for x in range(16, 304):
            p = y - s
            if p < 56:
                c = sp["pieuvre_mer0"][p][(x - 16) % 64]
                if plein[y][x]:
                    c = sp["pieuvre_mer_noyee"][p][x % 16]
            else:
                c = 0 if plein[y][x] else 1
            t[y][x] = c
    # la Pieuvre au loin qui lance, la crête devant elle
    hx = 252
    poser(t, pl.sprite("pieuvre_colere0"), hx, s - 16, coupe=s)
    poser(t, sp["pieuvre_bras_lance"], hx - 16, s - 42)
    poser(t, sp["pieuvre_tonneau"], hx - 16 + 36, s - 42 - 22)
    for x in range(16, 304, 64):
        poser(t, sp["pieuvre_crete0"], x, s - 3)
    for x in range(304, 320):
        for y in range(s - 3, 180):
            t[y][x] = base[y][x]
    poser(t, pl.sprite("remous_pieuvre0"), hx + 8, s - 4)
    # un objet annoncé (le repère sous l'interface, l'ombre sur la corniche)
    def sol(x, depuis):
        return next(y for y in range(depuis, 180) if plein[y][x] and plein[y][x + 15])
    xa = 228
    poser(t, sp["pieuvre_repere0"], xa + 4, 20)
    poser(t, pl.sprite("ombre_caisse"), xa, sol(xa, 30) - 2)
    # une CAISSE qui tombe devant une arcade (la colonne de gauche)
    xb, yb = 140, 72
    poser(t, sp["pieuvre_caisse"], xb, yb)
    poser(t, sp["pieuvre_trainee"], xb, yb - 10)
    poser(t, pl.sprite("ombre_caisse"), xb, sol(xb, yb + 16) - 2)
    return t


def planche_sprites():
    """Tous les nouveaux dessins à l'échelle x3 : les trois couches de la tour (un
    morceau), puis chaque sprite et sa version pâle."""
    sp = sprites()
    murs = couches()
    t = toile(600, 470, 1)
    pl.ecrire(t, 4, 4, "CARTE 9.36B : LES NOUVEAUX DESSINS DE LA PHASE 2 (X3)", 11)
    for k, nom in enumerate(("tour_bas", "tour_milieu", "tour_haut")):
        poser(t, murs[nom], 4 + 0, 14 + k * 100)
        pl.ecrire(t, 296, 14 + k * 100 + 44, nom.upper().replace("_", " "), 7)
    x, y, haut = 360, 14, 0
    for nom in sorted(sp):
        g = sp[nom]
        w, h = len(g[0]), len(g)
        if x + w > 596:
            x, y, haut = 360, y + haut + 12, 0
        fond = [[1 if v is None else v for v in r] for r in g]
        poser(t, fond, x, y)
        x += w + 6
        haut = max(haut, h)
    y += haut + 14
    pl.ecrire(t, 360, y, "VERSIONS PALES (FACILE) :", 5)
    x, y, haut = 360, y + 8, 0
    for nom in ("pieuvre_bras_repos", "pieuvre_bras_prepare", "pieuvre_bras_lance", "pieuvre_encre0", "pieuvre_encre1", "pieuvre_goutte", "pieuvre_bulles0"):
        g = sp[nom]
        p = [[v if v is None or (i + j) % 2 == 0 else None for i, v in enumerate(r)] for j, r in enumerate(g)]
        poser(t, [[1 if v is None else v for v in r] for r in p], x, y)
        x += len(g[0]) + 6
        haut = max(haut, len(g))
    pl.ecrire(t, 4, 316, "16 COULEURS DU JEU, CONTOUR DE 1 PX, AUCUN FLOU. EN BAS : UNE ARCADE SUR TROIS ;", 5)
    pl.ecrire(t, 4, 324, "EN HAUT : DEUX SUR TROIS. LA BAIE DE DROITE (LA PIEUVRE) RESTE UN PAN PLEIN, FROID.", 5)
    return t


def main():
    pl.SORTIE.mkdir(parents=True, exist_ok=True)
    pl.ecrire_png(pl.SORTIE / "936-melange-scene.png", scene_melange(), 3)
    print(pl.SORTIE / "936-melange-scene.png")
    if "--scene" in sys.argv:
        return 0
    if not OUTIL.exists():
        print("construire l'outil sprite d'abord :", OUTIL, file=sys.stderr)
        return 1
    for nom, g in couches().items():
        ecrire_asset(RACINE / "assets" / "fonds" / "port" / f"{nom}.png", g)
    for nom, g in sprites().items():
        ecrire_asset(RACINE / "assets" / "sprites" / "pieuvre" / f"{nom}.png", g)
    pl.ecrire_png(pl.SORTIE / "936b-planche-sprites.png", planche_sprites(), 3)
    print(pl.SORTIE / "936b-planche-sprites.png")
    return 0


if __name__ == "__main__":
    sys.exit(main())
