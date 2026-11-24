# Les fonds de PÉPIN (cartes 10.4c, 10.4d et 10.4f)

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
| `egouts/mur.png` | 1 x 180 | Le mur uni du collecteur (ardoise de nuit). | La répétition sur la largeur. |
| `egouts/arches.png` | 112 x 88 | UNE arche du collecteur (sa période, 112 px) : l'anneau de claveaux et sa clé, les joints, les pierres d'angle, la bouche et la galerie qui s'enfonce. Le haut du fichier est posé en y = 30 (`HAUT_ARCHES`), le bas touche l'eau (y = 118). | La répétition tous les 112 px, la parallaxe (`FACTEUR_MUR`, `VERT_MUR`). |
| `port/crepuscule.png` | 1 x 56 | Le ciel du crépuscule du port : ardoise, nuit, violet, corail, avec la trame de deux rangées entre deux bandes. | La répétition sur la largeur ; le décalage sous la tour de la Pieuvre. |
| `port/mer.png` | 1 x 124 | La mer unie et sa ligne d'horizon (posée en y = 56). | Idem. |
| `port/port_d_en_face.png` | 640 x 60 | Le port d'en face sur UNE période (640 px) : entrepôts et leurs fenêtres, piles de conteneurs, les trois grues à portique, le quai. La colonne x est le point x du motif ; le haut est 56 px au-dessus du quai. | La répétition (`poser_motif`), la parallaxe (`F_PORT`), les feux de balisage qui clignotent. |
| `port/pontons.png` | 640 x 92 | Les pontons sur pieux sur UNE période (640 px), les filets qui sèchent, les lampadaires et leur halo. Les pieux continuent 8 rangées sous l'écran (une secousse ne laisse jamais de trou). | La répétition, la parallaxe (`F_PONTONS`), les bouées qui dansent et le reflet des halos qui ondule. |
| `caniveau/maisons.png` | 496 x 154 | La rue du caniveau sur UNE période (496 px, les huit maisons de `decor_rue::RUE`) : façades d'ardoise, toits en mansarde et lucarnes, vitres ÉTEINTES et leurs croisées, stores et vitrines, portes, descentes de gouttière, puis le trottoir d'en face (62 rangées sous le sol). Le haut du fichier est le faîte de la plus haute maison, 92 px au-dessus du sol. | La répétition (`tour * 496 - dec`), la parallaxe (`FACTEUR_ILOTS`, `VERT_ILOTS`), et les fenêtres ALLUMÉES (plan « fenêtres allumées ») : elles changent d'un tour de la rue à l'autre (`allumee(tour * 8 + maison, ..)`), peintes en violet carreau par carreau sur les vitres éteintes. |
| `caniveau/lampadaire.png` | 15 x 51 | UN lampadaire de la rue : le halo en marches d'escalier, le mât, son socle et sa crosse, la lanterne et ses vitres violettes. Le mât est à la colonne 6, le bas du fichier sur le sol. | Les deux lampadaires (abscisses 60 et 230) posés chacun en UN exemplaire modulo la largeur de l'écran (coupés au bord droit, sans copie à gauche), la parallaxe (`FACTEUR_PROCHE`, `VERT_PROCHE`). |
| `riviere/ciel_vallee.png` | 2 x 89 | Le ciel de la vallée au soleil couchant (carte 10.4f) : six bandes (ardoise, violet, corail, orange, or pâle, brume), chacune ouverte par deux rangées en damier. Sa période est de DEUX colonnes (la trame) ; il va de la première rangée pleine du haut à la première rangée pleine de la brume. | La pose tous les 2 px, la parallaxe verticale (`V_CIEL`), et le prolongement de sa première et de sa dernière rangée au-dessus et au-dessous du fichier (couleurs LUES dans le fichier). |
| `riviere/soleil_couchant.png` | 31 x 31 | Le disque du soleil couchant : l'or pâle (rayon 15) et son cœur blanc (rayon 11). | Sa position (parallaxe lente), son halo orange qui respire et les deux nuages fins qui passent devant (plans calculés autour de lui). |
| `riviere/moulin.png` | 36 x 38 | UN vieux moulin : murs de pierre, toit en gradins, cheminée, fenêtre allumée, porte. Le pied de son mur gauche est à la colonne 3, son sol à la rangée 36. | Sa pose (un moulin tous les 760 px du plan de la forêt), et sa roue qui tourne et l'eau qu'elle verse (plan « roue du moulin »). |

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

Monde 2 (carte 10.4d) :

- Égouts : les piliers (coulures et grilles SEMÉES AU HASARD pilier par pilier, sans
  période, et la goutte qui tombe), l'eau (reflets qui tremblent selon le temps), les
  tuyaux (semés au hasard cellule par cellule). La grille et la lumière du départ
  (`decor_egouts`) suivent le niveau.
- Port : les étoiles et les mouettes (animées), le reflet de la lueur et la houle
  (animés), le large (le second cargo glisse, le faisceau du phare tourne ; le premier
  cargo et le phare lointain sont dessinés avec lui, dans le doute ils restent du code
  pour l'instant), les feux de balisage, les voiliers (bercés par la houle), les bouées
  et le reflet des halos. La bruine passe devant le niveau.
- Sous la tour de la Pieuvre (`dessiner_fond_sous_la_tour`) : le crépuscule qui
  s'éteint selon la montée, les étoiles, la lune, le port lointain qui s'enfonce :
  tout dépend de la hauteur de la caméra. Le fond de l'arène, en dessous, est la même
  pile que le port (sans le phare), décalée vers le bas.
- Rue du caniveau (carte 10.4d, suite) : les fenêtres allumées (elles changent d'un
  tour de la rue à l'autre, donc ne sont pas dans la période de `maisons.png`). Les
  descentes de gouttière `J` et leur pan de façade (`decor_rue::dessiner_gouttieres`,
  `dessiner_filets`) et le mobilier du trottoir (`mobilier_rue`) suivent la grille du
  niveau.
- Cascade (carte 10.4f) : la grande chute est entrée dans la pile (plans de
  `decor_cascade::PILE`), mais reste TOUT ENTIÈRE du code : elle suit la grille (son
  bas est le bassin d'arrivée du niveau, la falaise du pied court sur toute sa
  largeur, les filets continuent les chutes `|`) et elle s'anime (l'eau qui dévale,
  l'écume, la brume). Les couches de la pile qui en ont besoin reçoivent le niveau
  (`decor::dessiner_fond_du_niveau`) ; sans niveau, elles ne dessinent rien.
- Rivière (carte 10.4f) : la vallée est entrée dans la pile (plans de
  `decor_riviere::PILE`), DEVANT le fond générique du thème, qu'elle recouvre en
  entier (son ciel remplit l'écran). Ce fond générique reste dans la pile tel quel :
  le ciel `riviere/ciel.png` nourrit aussi les garde-fous de contraste
  (`Ambiance::ciel`). Restent du code : le halo et les nuages fins (animés), les
  oiseaux, la rivière du fond et les lucioles (animés), la roue du moulin (elle
  tourne), et les montagnes, les collines, les deux rangs d'arbres, le versant et
  les grands arbres : SEMÉS AU HASARD (`hache`, crêtes en sinus) sans période, comme
  les silhouettes du Monde 1 ; les grands arbres ondulent en plus au vent.
- PAS ENCORE traités (voir QA.md, carte 10.4f) : la grotte
  (`decor_grotte`), la Pieuvre
  (`dessin_port`) et la montée (`dessin_montee`), les motifs du Monde 2 (`decor`).
  Tout ce qui y est posé d'après la grille (phare de la jetée de la Pieuvre, habillage
  des quais, gouttières, bords de toits, chutes jouables, mobilier du trottoir) restera
  du code.

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
- `Caniveau` : ciel PNG caniveau/ciel.png, silhouettes lointaines CALCULÉ, maisons PNG caniveau/maisons.png, fenêtres allumées CALCULÉ, silhouettes proches CALCULÉ, lampadaires PNG caniveau/lampadaire.png, NIVEAU, pluie CALCULÉ
- `Egouts` : mur PNG egouts/mur.png, arches PNG egouts/arches.png, piliers CALCULÉ, eau CALCULÉ, tuyaux CALCULÉ, NIVEAU
- `Cascade` : ciel PNG cascade/ciel.png, silhouettes lointaines CALCULÉ, motifs 1 CALCULÉ, silhouettes proches CALCULÉ, falaise du pied CALCULÉ, rideau de la chute CALCULÉ, brume de la chute CALCULÉ, filets des chutes CALCULÉ, NIVEAU, pluie CALCULÉ
- `Grotte` : ciel PNG grotte/ciel.png, plafond lointain CALCULÉ, plafond proche CALCULÉ, silhouettes lointaines CALCULÉ, silhouettes proches CALCULÉ, motifs 2 CALCULÉ, NIVEAU
- `Riviere` : ciel PNG riviere/ciel.png, silhouettes lointaines CALCULÉ, motifs 1 CALCULÉ, silhouettes proches CALCULÉ, motifs 2 CALCULÉ, ciel de la vallée PNG riviere/ciel_vallee.png, halo du soleil CALCULÉ, soleil couchant PNG riviere/soleil_couchant.png, nuages fins CALCULÉ, oiseaux CALCULÉ, montagnes CALCULÉ, collines CALCULÉ, rang clair CALCULÉ, rivière du fond CALCULÉ, moulin PNG riviere/moulin.png, roue du moulin CALCULÉ, rang sombre CALCULÉ, versant CALCULÉ, grands arbres CALCULÉ, lucioles CALCULÉ, NIVEAU
- `Port` : crépuscule PNG port/crepuscule.png, étoiles du port CALCULÉ, mouettes CALCULÉ, mer PNG port/mer.png, reflets CALCULÉ, large CALCULÉ, port d'en face PNG port/port_d_en_face.png, balises CALCULÉ, voiliers CALCULÉ, pontons PNG port/pontons.png, bouées CALCULÉ, NIVEAU, bruine CALCULÉ
- `Maree` : ciel PNG maree/ciel.png, silhouettes lointaines CALCULÉ, silhouettes proches CALCULÉ, motifs 2 CALCULÉ, NIVEAU, pluie CALCULÉ
<!-- pile -->

Après le fond, avant les tuiles, la scène dessine encore ce qui dépend du niveau : les
éclairs, le soleil qui perce et le mur de pluie de la poursuite, la déchirure, les
façades (tout cela reste du code). La pluie passe devant le niveau.
