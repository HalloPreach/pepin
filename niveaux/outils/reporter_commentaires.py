#!/usr/bin/env python3
"""Reporte les commentaires de conception des niveaux Rust dans les fichiers RON.

Carte 10.1c : les niveaux ne sont plus écrits en Rust (`core/src/levels/<cle>.rs`)
mais seulement en RON (`assets/levels/<dossier>/<cle>.ron`). Les `.rs` portaient
des commentaires précieux (pourquoi telle colonne, quelle recette, quel piège) :
ce script, lancé UNE fois avant leur suppression, les recopie en commentaires
`//` dans le `.ron` du même niveau, à la place que l'éditeur sait garder :

- la documentation du module (`//!`, en haut du `.rs`) va dans l'en-tête du
  fichier, sous la ligne « Niveau de PÉPIN. Format : ... », avant la parenthèse ;
- un commentaire posé dans `INFOS` juste avant un champ (ou à l'intérieur de sa
  valeur) va juste avant le même champ du `.ron` ;
- la documentation (`///`) de `const ASCII` va avant `ascii:`, celle de
  `const AIGUILLES` (et les commentaires de son contenu) avant `aiguilles:`, celle
  d'une constante de hauteur de poissons avant `poissons_hauts:`.

Les tests (`mod tests`) ne sont pas repris : ils partent dans `niveaux/tests/`
avec leurs propres commentaires.

    python3 niveaux/outils/reporter_commentaires.py      (depuis la racine du dépôt)

Le script refuse de toucher un `.ron` qui contient déjà des commentaires de plus
que l'en-tête (il ne s'applique qu'une fois). Il vérifie à la fin que chaque
fichier garde exactement les mêmes lignes une fois les commentaires retirés.
"""

import re
import sys
from pathlib import Path

RACINE = Path(__file__).resolve().parents[2]
LEVELS = RACINE / "core" / "src" / "levels"
ASSETS = RACINE / "assets" / "levels"
EN_TETE = "// Niveau de PÉPIN. Format : voir assets/levels/LISEZMOI.md."
CHAMPS = [
    "monde", "numero", "cle", "nom", "theme", "genre", "sans_planer",
    "duree_cible_s", "intro_texte", "intro_astuce", "mecanique", "aiguilles", "poissons_hauts", "ascii",
]


def en_commentaire_ron(ligne):
    """`//! texte`, `/// texte` ou `// texte` (indentés ou non) -> `// texte`."""
    texte = ligne.strip()
    for prefixe in ("//!", "///", "//"):
        if texte.startswith(prefixe):
            reste = texte[len(prefixe):]
            return ("//" + reste).rstrip()
    raise ValueError(f"pas un commentaire : {ligne!r}")


def lire_rs(chemin):
    """(en-tête, {champ: [commentaires]}) tirés du `.rs` d'un niveau."""
    lignes = chemin.read_text(encoding="utf-8").split("\n")
    # Le module de tests n'est pas repris.
    for i, l in enumerate(lignes):
        if l.startswith("#[cfg(test)]") or l.startswith("mod tests"):
            lignes = lignes[:i]
            break
    entete = []
    par_champ = {c: [] for c in CHAMPS}
    i = 0
    while i < len(lignes) and lignes[i].startswith("//!"):
        entete.append(en_commentaire_ron(lignes[i]))
        i += 1
    en_attente = []  # commentaires lus, pas encore rattachés à un champ
    bloc = None  # `INFOS`, `aiguilles`... : le bloc ouvert en ce moment
    champ_courant = None  # dans INFOS : le champ dont la valeur est en cours
    profondeur = 0
    while i < len(lignes):
        l = lignes[i]
        s = l.strip()
        i += 1
        if bloc is None:
            if s.startswith("//"):
                en_attente.append(en_commentaire_ron(s))
                continue
            if s.startswith("pub const INFOS"):
                bloc, profondeur = "INFOS", 1
                continue
            m = re.match(r"(pub )?const (\w+)", s)
            if m:
                nom = m.group(2)
                if nom == "ASCII":
                    cible = "ascii"
                elif nom == "AIGUILLES":
                    cible = "aiguilles"
                elif "POISSONS" in nom:
                    cible = "poissons_hauts"
                    # La valeur de la constante est déjà écrite dans le `.ron` :
                    # on garde son nom pour que le commentaire reste lisible.
                    en_attente.append(f"// (Ancienne constante `{nom}` du code Rust : {s.split('=')[1].strip().rstrip(';')}.)")
                else:
                    raise ValueError(f"{chemin.name} : constante inattendue {nom}")
                par_champ[cible] += en_attente
                en_attente = []
                if nom == "ASCII":
                    # La grille (chaîne brute) se termine par `";` seul sur sa ligne.
                    while i < len(lignes) and lignes[i].strip() != '";':
                        i += 1
                    i += 1
                elif s.endswith("["):
                    bloc, profondeur = cible, 1
                continue
            if s.startswith("use ") or s == "":
                continue
            raise ValueError(f"{chemin.name} : ligne inattendue {l!r}")
        # Dans un bloc (INFOS ou le contenu d'AIGUILLES).
        if bloc != "INFOS":
            if s.startswith("//"):
                par_champ[bloc].append(en_commentaire_ron(s))
            profondeur += s.count("[") + s.count("{") - s.count("]") - s.count("}")
            if profondeur <= 0:
                bloc = None
            continue
        if s == "};" and profondeur == 1:
            bloc = None
            continue
        if s.startswith("//"):
            if champ_courant is not None and profondeur > 1:
                par_champ[champ_courant].append(en_commentaire_ron(s))
            else:
                en_attente.append(en_commentaire_ron(s))
            continue
        if profondeur == 1:
            m = re.match(r"(\w+)\s*:", s)
            if not m or m.group(1) not in CHAMPS:
                # Suite d'une valeur sur plusieurs lignes (`mecanique:` puis le texte).
                continue
            champ_courant = m.group(1)
            par_champ[champ_courant] += en_attente
            en_attente = []
        profondeur += s.count("[") + s.count("(") - s.count("]") - s.count(")")
    if en_attente:
        raise ValueError(f"{chemin.name} : commentaires sans champ {en_attente}")
    return entete, par_champ


def sans_commentaires(texte):
    return [l for l in texte.split("\n") if not l.lstrip().startswith("//")]


def reporter(rs):
    cle = rs.stem
    trouves = list(ASSETS.glob(f"*/{cle}.ron"))
    if len(trouves) != 1:
        raise ValueError(f"{cle} : {len(trouves)} fichier(s) .ron")
    ron = trouves[0]
    texte = ron.read_text(encoding="utf-8")
    lignes = texte.split("\n")
    if sum(1 for l in lignes if l.lstrip().startswith("//")) != 1 or lignes[0] != EN_TETE:
        raise ValueError(f"{ron} : déjà commenté (le script ne s'applique qu'une fois)")
    entete, par_champ = lire_rs(rs)
    sortie = [EN_TETE]
    if entete:
        sortie.append("//")
        sortie += entete
    for l in lignes[1:]:
        m = re.match(r"    (\w+): ", l)
        if m and m.group(1) in CHAMPS:
            sortie += ["    " + c for c in par_champ[m.group(1)]]
        sortie.append(l)
    nouveau = "\n".join(sortie)
    assert sans_commentaires(nouveau) == sans_commentaires(texte), ron
    ron.write_text(nouveau, encoding="utf-8")
    n = len(entete) + sum(len(v) for v in par_champ.values())
    print(f"{ron.relative_to(RACINE)} : {n} lignes de commentaire")


def main():
    fichiers = sorted(p for p in LEVELS.glob("*.rs") if p.stem != "variantes")
    if not fichiers:
        sys.exit("aucun niveau Rust dans core/src/levels (déjà supprimés ?)")
    for rs in fichiers:
        reporter(rs)


if __name__ == "__main__":
    main()
