# Les sprites de PÉPIN

Chaque sprite dessiné à la main est un fichier PNG de ce dossier : c'est la SEULE
source du dessin. Le jeu les embarque tous à la compilation (`game/build.rs` liste le
dossier) ; ajouter un sprite = ajouter un fichier.

## Nommage

`assets/sprites/<famille>/<clé>.png`, par exemple `ennemis_eau/grenouille0.png`.

- La CLÉ (le nom du fichier sans `.png`) est le nom que le code demande
  (`banque.dessiner("grenouille0", ...)`). Les clés sont uniques, toutes familles
  confondues.
- La FAMILLE est le dossier : `pepin`, `pepinette`, `ennemis`, `ennemis_eau`,
  `pieuvre`, `objets`, `objets_decor`, `objets_eau`, `tuiles`, `jetee`, et les jeux de
  tuiles du Monde 2, un par thème : `tuiles_monde2/caniveau`, `egouts`, `cascade`,
  `grotte`, `port`, `riviere`. La liste est dans `game/src/sprites/fichiers.rs`
  (`FAMILLES`) : un nouveau dossier doit y être ajouté (un test refuse un dossier
  inconnu).
- Les commentaires de conception de chaque dessin restent dans le module Rust de sa
  famille (`game/src/sprites/<famille>.rs`), sous la forme `// \`clé.png\` : ...`.

Ce qui est CALCULÉ n'a pas de fichier : les tuiles éclaircies par thème, le miroir,
les motifs de parapluie, les silhouettes blanches ou tramées, et les ALIAS (un même
dessin sous un autre nom, par exemple `pepinette_mort2` = `pepin_mort2`, voir `ALIAS`
dans `fichiers.rs`).

## Format

- Taille réelle : 1 pixel du fichier = 1 pixel du jeu (le jeu agrandit sans flou).
- PNG indexé 8 bits, palette de 17 entrées : les 16 couleurs de
  `pepin_core::palette::TOUTES` dans leur ordre (l'indice 0 est l'ombre de contour,
  8 le corail du parapluie, B l'or, D le cyan de la pluie...), puis le transparent
  (indice 16, rendu invisible par le bloc `tRNS`).
- Blocs permis : `IHDR`, `PLTE`, `tRNS`, `IDAT`, `IEND`, rien d'autre (aucun texte,
  aucune date, aucun nom d'auteur ou de logiciel). Un test le vérifie sur chaque
  fichier.
- Le jeu accepte aussi un PNG retouché dans un logiciel de dessin, tant que chaque
  pixel est transparent ou exactement d'une couleur de la palette. Repasser ensuite le
  fichier par l'outil (`lire` puis `ecrire`) le remet au format canonique et retire les
  blocs en trop.

## L'outil `sprite` : PNG vers texte et retour

Un sprite se lit et se corrige commodément sous forme de GRILLE DE TEXTE : une ligne
par rangée de pixels, un caractère par pixel, `.` pour le transparent, `0`-`9` puis
`A`-`F` pour l'indice de la couleur dans la palette.

    # afficher la grille d'un sprite
    cargo run -p outils --bin sprite -- lire assets/sprites/ennemis/corbeau0.png

    # refaire le PNG depuis une grille (lue sur l'entrée standard)
    cargo run -p outils --bin sprite -- ecrire assets/sprites/ennemis/corbeau0.png < corbeau0.txt

Avec Docker (aucun Rust sur la machine), même chose précédée de `./docker-cargo.sh`.

L'écriture est CANONIQUE : relire un fichier puis le réécrire donne exactement les
mêmes octets (un test le vérifie sur tous les fichiers). Un diff git sur un PNG ne
montre donc que de vrais changements de dessin.

L'outil (et son encodeur PNG, crate `png`) vit dans le crate `outils`, jamais dans le
jeu : le jeu ne fait que DÉCODER, avec le décodeur que macroquad embarque déjà.

## Dans l'éditeur de niveaux

L'éditeur (`editeur.html`, build de QA seulement) montre, dans sa palette et dans la
grille, la VIGNETTE du vrai sprite de chaque lettre qui en a un (première image de
l'animation, ancrée comme dans le jeu), à la place de la case colorée. Le lien
lettre → sprite est la table `VIGNETTES` de `niveaux/src/legende.rs` (une CLÉ par
lettre) ; la famille, donc le chemin du fichier, vient de la liste des PNG faite par
`niveaux/build.rs`, et `legende.json` le donne à l'éditeur (champs `sprite` et
`ancre`). `./build.sh --qa` copie ces PNG dans `dist-qa/sprites/`. Renommer un sprite
d'une lettre sans mettre la table à jour fait échouer le test
`chaque_vignette_est_un_vrai_sprite`. Les tuiles (dessinées par thème, bords
calculés) et les zones gardent leur case colorée ; le bouton « vignettes / cases »
(touche V) revient aux cases pour toute la grille.

## Vérifier

- `cargo test -p game --no-default-features sprites::` : chaque PNG se décode, aucun
  pixel hors palette, blocs permis, aucun fichier orphelin, tailles attendues par le
  code ; le test `ecrit_la_planche_contact_dans_docs` refait
  `docs/planche-sprites.png` (tous les sprites sur une planche) à partir des PNG.
- `cargo test -p outils` : aller-retour de l'outil sur tous les fichiers.
