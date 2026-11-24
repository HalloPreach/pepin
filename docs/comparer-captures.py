"""Compare des captures PNG pixel à pixel (carte 10.2 : avant / après une carte de rendu ;
carte 10.1b : départ des niveaux avant / après la bascule vers les fichiers RON).

Usage, deux formes :
  python3 docs/comparer-captures.py A.png B.png [A2.png B2.png ...]
      compare les fichiers deux à deux (A avec B, A2 avec B2...), par exemple les
      captures docs/qa/101b-avant-*.png et 101b-apres-*.png de docs/capture-101b.mjs ;
  python3 docs/comparer-captures.py DOSSIER_A/PREFIXE_A DOSSIER_B/PREFIXE_B
      (sans « .png ») compare les trois captures de docs/capture-102.sh :
      PREFIXE_A-toits.png avec PREFIXE_B-toits.png, puis -port, puis -pieuvre.

Pour chaque paire : le nombre de pixels différents et, s'il y en a, le rectangle qui les
contient (en pixels de la capture). Code de sortie : 0 si toutes les paires sont identiques,
1 sinon (ou si un fichier manque, ou si les tailles diffèrent).

Python seul, sans bibliothèque : le PNG est décodé à la main (zlib + les cinq filtres de
ligne). On compare les PIXELS décodés, pas les octets du fichier : deux PNG de mêmes pixels
peuvent être compressés différemment.
"""
import struct
import sys
import zlib

SCENES = ('toits', 'port', 'pieuvre')
# Octets par pixel selon le type de couleur PNG (en 8 bits par canal) :
# 0 gris, 2 RVB, 4 gris + alpha, 6 RVB + alpha. Les palettes (3) ne sont pas gérées :
# Chromium n'en écrit pas.
CANAUX = {0: 1, 2: 3, 4: 2, 6: 4}


def lire_png(chemin):
    """Renvoie (largeur, hauteur, octets par pixel, lignes de pixels décodées)."""
    with open(chemin, 'rb') as f:
        donnees = f.read()
    if donnees[:8] != b'\x89PNG\r\n\x1a\n':
        raise ValueError(f'{chemin} : pas un PNG')
    pos, idat, entete = 8, [], None
    while pos < len(donnees):
        (longueur,) = struct.unpack('>I', donnees[pos:pos + 4])
        genre = donnees[pos + 4:pos + 8]
        corps = donnees[pos + 8:pos + 8 + longueur]
        pos += 12 + longueur  # longueur + genre + corps + CRC
        if genre == b'IHDR':
            entete = struct.unpack('>IIBBBBB', corps)
        elif genre == b'IDAT':
            idat.append(corps)
        elif genre == b'IEND':
            break
    if entete is None:
        raise ValueError(f'{chemin} : PNG sans en-tête IHDR')
    largeur, hauteur, bits, couleur, _, _, entrelace = entete
    if bits != 8 or couleur not in CANAUX or entrelace:
        raise ValueError(f'{chemin} : format PNG non géré ({bits} bits, couleur {couleur})')
    bpp = CANAUX[couleur]
    brut = zlib.decompress(b''.join(idat))
    pas = largeur * bpp
    lignes, avant = [], bytearray(pas)
    for y in range(hauteur):
        debut = y * (pas + 1)
        filtre = brut[debut]
        ligne = bytearray(brut[debut + 1:debut + 1 + pas])
        # Les filtres PNG : chaque octet est stocké comme une différence avec ses voisins
        # (a : à gauche, b : au-dessus, c : en haut à gauche) ; on les rajoute.
        if filtre == 1:
            for i in range(bpp, pas):
                ligne[i] = (ligne[i] + ligne[i - bpp]) & 255
        elif filtre == 2:
            for i in range(pas):
                ligne[i] = (ligne[i] + avant[i]) & 255
        elif filtre == 3:
            for i in range(pas):
                a = ligne[i - bpp] if i >= bpp else 0
                ligne[i] = (ligne[i] + ((a + avant[i]) >> 1)) & 255
        elif filtre == 4:
            for i in range(pas):
                a = ligne[i - bpp] if i >= bpp else 0
                b = avant[i]
                c = avant[i - bpp] if i >= bpp else 0
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                pred = a if pa <= pb and pa <= pc else (b if pb <= pc else c)
                ligne[i] = (ligne[i] + pred) & 255
        lignes.append(bytes(ligne))
        avant = ligne
    return largeur, hauteur, bpp, lignes


def comparer(a, b):
    """Affiche le résultat d'une paire ; renvoie True si les pixels sont identiques."""
    try:
        la, ha, bpa, lignes_a = lire_png(a)
        lb, hb, bpb, lignes_b = lire_png(b)
    except (OSError, ValueError) as e:
        print(f'ERREUR  {e}')
        return False
    if (la, ha) != (lb, hb):
        print(f'DIFFÉRENT  {a} ({la}x{ha}) / {b} ({lb}x{hb}) : tailles différentes')
        return False
    differents, boite = 0, None
    for y in range(ha):
        ra, rb = lignes_a[y], lignes_b[y]
        if ra == rb and bpa == bpb:
            continue  # ligne identique : le cas courant, comparé d'un bloc
        for x in range(la):
            # On compare les trois canaux de couleur (l'alpha éventuel est ignoré : une
            # capture d'écran est opaque).
            pa = ra[x * bpa:x * bpa + min(bpa, 3)]
            pb = rb[x * bpb:x * bpb + min(bpb, 3)]
            if pa != pb:
                differents += 1
                boite = (x, y, x, y) if boite is None else (
                    min(boite[0], x), min(boite[1], y), max(boite[2], x), max(boite[3], y))
    if differents == 0:
        print(f'IDENTIQUE  {a} = {b} ({la}x{ha}, 0 pixel différent)')
        return True
    print(f'DIFFÉRENT  {a} / {b} : {differents} pixels différents sur {la * ha}, '
          f'dans le rectangle x {boite[0]}..{boite[2]}, y {boite[1]}..{boite[3]}')
    return False


def main(args):
    if len(args) == 2 and not any(x.endswith('.png') for x in args):
        paires = [(f'{args[0]}-{s}.png', f'{args[1]}-{s}.png') for s in SCENES]
    elif len(args) >= 2 and len(args) % 2 == 0:
        paires = list(zip(args[0::2], args[1::2]))
    else:
        print(__doc__)
        return 2
    resultats = [comparer(a, b) for a, b in paires]
    identiques = sum(resultats)
    print(f'{identiques}/{len(resultats)} paires identiques')
    return 0 if identiques == len(resultats) else 1


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
