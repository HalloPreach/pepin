# Les fonds de PÉPIN (carte 10.4c)

Chaque couche FIXE d'un fond (dessinée une fois pour toutes, toujours pareille) est un
fichier PNG de ce dossier : `assets/fonds/<thème>/<nom>.png`. C'est la SEULE source du
dessin : le jeu n'a plus de code qui la trace. Tout ce qui est CALCULÉ (animé, semé au
hasard, qui suit le temps ou la caméra pixel par pixel, ou placé d'après la grille du
niveau) reste du code, dans `game/src/decor.rs` et ses voisins.

Même chaîne que les sprites (`assets/sprites/LISEZMOI.md`, carte 10.4a) :

- `game/build.rs` embarque TOUS les `.png` de ce dossier (`include_bytes!`, aucune liste
  à la main) : ajouter une couche, c'est ajouter un fichier (et la mettre dans la pile,
  voir plus bas) ;
- PNG indexé, 1 pixel = 1 pixel, couleurs de `pepin_core::palette` seulement, plus le
  transparent ; seuls les blocs IHDR, PLTE, tRNS, IDAT et IEND (aucun texte, aucune
  date, aucun nom) ;
- lire ou réécrire une couche avec l'outil `sprite` :
  `cargo run -p outils --bin sprite -- lire assets/fonds/toits/ciel.png` (la grille de
  texte), puis `... -- ecrire assets/fonds/toits/ciel.png < grille.txt` ;
- les tests (`game/src/fonds.rs`, `game/src/decor.rs`) vérifient : décodage sans pixel
  hors palette, taille attendue, blocs permis, 16 Ko au plus par couche, aucune couche
  manquante ni orpheline, et que la pile écrite ci-dessous est celle du rendu.

## Les couches PNG

| Fichier | Taille | Ce que c'est | Ce qui reste du code |
| --- | --- | --- | --- |
| `<thème>/ciel.png` | 1 x 180 | Les quatre bandes du ciel et leur trame (une ligne sur deux sur les 6 dernières lignes d'une bande). Un ciel ne change pas d'une colonne à l'autre : le fichier n'a qu'UNE colonne, sa période. | La répétition sur les 320 colonnes de l'écran (`fonds::repeter`). Les couleurs des bandes (`Ambiance::ciel`, pour les garde-fous de contraste) sont LUES dans le fichier (`bandes_du_ciel`). |
| `descente/voyage.png` | 1 x 450 | Le ciel de la descente : dix bandes, de l'or du soleil à la nuit des toits. | La remontée selon la hauteur de la caméra (`remontee_voyage`), la répétition sur la largeur. `voyage()` lit les dix couleurs dans le fichier. |
| `soleil/soleil.png` | 45 x 45 | Le soleil : le disque pâle et son cœur blanc. | Sa position (parallaxe lente en x ; au voyage, la remontée en y). |
| `arc_en_ciel/arc.png` | 300 x 150 | L'arc-en-ciel du bonus : huit bandes de 3 px, de son sommet (la ligne 0, posée en y = 36) au centre du cercle (le bas du fichier, sous l'écran). | Sa glissade avec la caméra (`centre_arc`). |

La poursuite reprend le ciel des vents d'altitude (`altitude/ciel.png`) ; la descente,
en bas, a le ciel des toits (`toits/ciel.png`, les quatre dernières bandes du voyage).
Les ciels du Monde 2 qui passent par le même rendu (caniveau, cascade, grotte, rivière,
marée) sont des fichiers aussi : un seul chemin de dessin pour tous les ciels en bandes.

## Inventaire : ce qui reste CALCULÉ, et pourquoi

Monde 1 (toits, gouttières, marché, orage, clocher, cyclone, vents, mer, nuages au
soleil, arc-en-ciel, poursuite, descente) :

- Les étoiles : elles scintillent (s'éteignent un instant selon le temps).
- La pluie et sa densité, la pluie forte de la nouvelle partie + : animées.
- Les éclairs, le mur de pluie et le soleil qui perce de la poursuite (`mur_pluie`),
  la grêle : animés, et liés au niveau.
- Les silhouettes (immeubles, pics, bancs de nuages, lointaines et proches), les deux
  plafonds de nuages, les îlots : SEMÉS AU HASARD cellule par cellule (`hache`), sans
  période, sur toute la longueur du niveau. Un PNG devrait couvrir chaque niveau sur
  toute sa largeur (des milliers de pixels, et autant de variantes que de hauteurs de
  caméra au clocher), bien au-delà des 16 Ko par couche et contre la règle « une
  période seulement ». Dans le doute, ils restent du code.
- Les fenêtres allumées des immeubles : semées au hasard avec eux.
- Les tuiles éclaircies par thème (`TeinteTuiles`) : des variantes calculées des
  sprites, posées d'après la grille.
- La déchirure, les façades et tout décor rattaché aux tuiles ou aux entités du niveau
  (`scene`) : ils suivent la grille.

Écrans hors niveau :

- Titre, boutique, histoire, écran de fin : ils dessinent le fond d'un thème
  (`decor::dessiner_fond`), donc ses couches PNG et sa pile ci-dessous.
- La carte des niveaux (`ecrans/fond_plan.rs`) : PAS ENCORE traitée (voir QA.md,
  carte 10.4c). Son ciel suit la hauteur de la carte (des bandes de hauteur variable,
  fondues), ses nuages et ses silhouettes sont placés en proportion de cette hauteur,
  ses oiseaux et sa pluie sont animés. Les dessins des nuages (`NUAGES`) sont des
  candidats PNG pour une carte suivante.

Monde 2 (carte 10.4d) : le fond des égouts (`fond_egouts`), celui du port
(`decor_port`, avec son phare posé sur la jetée), la grande chute de la cascade, la
vallée de la rivière, la rue du caniveau et leurs motifs restent du code pour l'instant.

## La pile de dessin

Pour chaque thème, dans l'ordre RÉEL du rendu, du plus loin au plus près. « PNG » : un
fichier de ce dossier ; « CALCULÉ » : du code. NIVEAU : le niveau lui-même (tuiles,
entités, et les décors posés d'après sa grille) ; ce qui le suit passe devant. Les
motifs 0, 1, 2 sont ceux du Monde 2 (carte 9.5 : 0 derrière le plafond, 1 entre les
deux couches de silhouettes, 2 devant la couche proche).

Cette liste est la sortie EXACTE de `decor::pile`, la liste que suivent
`decor::dessiner_fond` et `decor::dessiner_avant` : le test
`la_pile_documentee_est_celle_du_rendu` la compare au texte entre les deux marques.

<!-- pile -->
- `Toits` : ciel PNG toits/ciel.png, étoiles CALCULÉ, silhouettes lointaines CALCULÉ, silhouettes proches CALCULÉ, NIVEAU, pluie CALCULÉ
- `Gouttieres` : ciel PNG gouttieres/ciel.png, plafond lointain CALCULÉ, plafond proche CALCULÉ, silhouettes lointaines CALCULÉ, silhouettes proches CALCULÉ, NIVEAU, pluie CALCULÉ
- `Marche` : ciel PNG marche/ciel.png, étoiles CALCULÉ, silhouettes lointaines CALCULÉ, silhouettes proches CALCULÉ, NIVEAU, pluie CALCULÉ
- `Orage` : ciel PNG orage/ciel.png, plafond lointain CALCULÉ, plafond proche CALCULÉ, silhouettes lointaines CALCULÉ, silhouettes proches CALCULÉ, NIVEAU, pluie CALCULÉ
- `Clocher` : ciel PNG clocher/ciel.png, étoiles CALCULÉ, silhouettes lointaines CALCULÉ, silhouettes proches CALCULÉ, NIVEAU, pluie CALCULÉ
- `Cyclone` : ciel PNG cyclone/ciel.png, plafond lointain CALCULÉ, plafond proche CALCULÉ, silhouettes lointaines CALCULÉ, silhouettes proches CALCULÉ, NIVEAU, pluie CALCULÉ
- `Altitude` : ciel PNG altitude/ciel.png, étoiles CALCULÉ, îlots CALCULÉ, silhouettes lointaines CALCULÉ, silhouettes proches CALCULÉ, NIVEAU, pluie CALCULÉ
- `Mer` : ciel PNG mer/ciel.png, îlots CALCULÉ, silhouettes lointaines CALCULÉ, silhouettes proches CALCULÉ, NIVEAU
- `Soleil` : ciel PNG soleil/ciel.png, soleil PNG soleil/soleil.png, silhouettes lointaines CALCULÉ, silhouettes proches CALCULÉ, NIVEAU
- `ArcEnCiel` : ciel PNG arc_en_ciel/ciel.png, arc-en-ciel PNG arc_en_ciel/arc.png, silhouettes lointaines CALCULÉ, silhouettes proches CALCULÉ, NIVEAU
- `Descente` : ciel du voyage PNG descente/voyage.png, soleil PNG soleil/soleil.png, îlots CALCULÉ, étoiles CALCULÉ, silhouettes lointaines CALCULÉ, silhouettes proches CALCULÉ, NIVEAU
- `Poursuite` : ciel PNG altitude/ciel.png, étoiles CALCULÉ, îlots CALCULÉ, silhouettes lointaines CALCULÉ, silhouettes proches CALCULÉ, NIVEAU
- `Caniveau` : ciel PNG caniveau/ciel.png, silhouettes lointaines CALCULÉ, motifs 1 CALCULÉ, silhouettes proches CALCULÉ, motifs 2 CALCULÉ, NIVEAU, pluie CALCULÉ
- `Egouts` : fond des égouts CALCULÉ, NIVEAU
- `Cascade` : ciel PNG cascade/ciel.png, silhouettes lointaines CALCULÉ, motifs 1 CALCULÉ, silhouettes proches CALCULÉ, NIVEAU, pluie CALCULÉ
- `Grotte` : ciel PNG grotte/ciel.png, plafond lointain CALCULÉ, plafond proche CALCULÉ, silhouettes lointaines CALCULÉ, silhouettes proches CALCULÉ, motifs 2 CALCULÉ, NIVEAU
- `Riviere` : ciel PNG riviere/ciel.png, silhouettes lointaines CALCULÉ, motifs 1 CALCULÉ, silhouettes proches CALCULÉ, motifs 2 CALCULÉ, NIVEAU
- `Port` : fond du port CALCULÉ, NIVEAU, bruine CALCULÉ
- `Maree` : ciel PNG maree/ciel.png, silhouettes lointaines CALCULÉ, silhouettes proches CALCULÉ, motifs 2 CALCULÉ, NIVEAU, pluie CALCULÉ
<!-- pile -->

Après le fond, avant les tuiles, la scène dessine encore ce qui dépend du niveau : les
éclairs, le soleil qui perce et le mur de pluie de la poursuite, la déchirure, les
façades (tout cela reste du code). La pluie passe devant le niveau.
