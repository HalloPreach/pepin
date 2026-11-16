#!/usr/bin/env python3
"""Planche « Pépin / Pépinette » côte à côte (carte 7.1) : toutes les images des deux skins,
Pépin en haut, Pépinette en dessous, dans l'ordre de leurs constantes `SPRITES`.

Lit les grilles directement dans `game/src/sprites/pepin.rs` et `pepinette.rs` (aucune image
externe) et les peint avec la palette de `core/src/palette.rs`, agrandies d'un facteur entier
(pas de flou). Fond ardoise nuit, comme le ciel le plus sombre.

Usage : python3 docs/planche-skins.py SORTIE.png [ECHELLE]   (défaut : 8)
"""
import re
import struct
import sys
import zlib
from pathlib import Path

RACINE = Path(__file__).resolve().parent.parent
SPRITES = RACINE / "game" / "src" / "sprites"


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


def images(fichier, autres=None):
    """(nom, grille) dans l'ordre de `SPRITES`. Une constante qui en reprend une autre
    (`= super::pepin::PEPIN_MORT2`) est cherchée dans `autres`."""
    texte = (SPRITES / fichier).read_text()
    grilles = {
        nom: re.findall(r'"([^"]*)"', corps)
        for nom, corps in re.findall(r"pub const (\w+): Image = &\[(.*?)\];", texte, re.S)
    }
    for nom, cible in re.findall(r"pub const (\w+): Image = super::\w+::(\w+);", texte):
        grilles[nom] = autres[cible]
    liste = re.search(r"pub const SPRITES: .*? = &\[(.*?)\];", texte, re.S).group(1)
    return [(n, grilles[c]) for n, c in re.findall(r'\("(\w+)", (\w+)\)', liste)], grilles


def main():
    sortie = sys.argv[1]
    echelle = int(sys.argv[2]) if len(sys.argv) > 2 else 8
    pal = palette()
    pepin, grilles_pepin = images("pepin.rs")
    pepinette, _ = images("pepinette.rs", grilles_pepin)
    assert len(pepin) == len(pepinette), "les deux skins n'ont pas le même nombre d'images"
    case_l = max(len(g[0]) for _, g in pepin + pepinette) + 4
    case_h = max(len(g) for _, g in pepin + pepinette) + 4
    largeur, hauteur = case_l * len(pepin) + 4, case_h * 2 + 4
    fond = 1  # ARDOISE_NUIT
    pixels = [[fond] * largeur for _ in range(hauteur)]
    for rangee, liste in enumerate((pepin, pepinette)):
        for i, (_, grille) in enumerate(liste):
            # Chaque image en bas de sa case (les pieds alignés), centrée en largeur.
            x0 = 4 + i * case_l + (case_l - 4 - len(grille[0])) // 2
            y0 = 4 + rangee * case_h + (case_h - 4 - len(grille))
            for y, ligne in enumerate(grille):
                for x, c in enumerate(ligne):
                    if c != ".":
                        pixels[y0 + y][x0 + x] = int(c, 16)
    brut = b"".join(
        (b"\x00" + b"".join(bytes(pal[i]) * echelle for i in ligne)) * echelle for ligne in pixels
    )

    def bloc(type_, donnees):
        return (
            struct.pack(">I", len(donnees))
            + type_
            + donnees
            + struct.pack(">I", zlib.crc32(type_ + donnees) & 0xFFFFFFFF)
        )

    entete = struct.pack(">IIBBBBB", largeur * echelle, hauteur * echelle, 8, 2, 0, 0, 0)
    Path(sortie).write_bytes(
        b"\x89PNG\r\n\x1a\n"
        + bloc(b"IHDR", entete)
        + bloc(b"IDAT", zlib.compress(brut, 9))
        + bloc(b"IEND", b"")
    )
    print(f"{sortie} : {len(pepin)} images par skin, {largeur * echelle}x{hauteur * echelle}")


if __name__ == "__main__":
    main()
