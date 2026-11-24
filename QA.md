# QA finale de PÉPIN

Date : 12/11/2026. Tout ce qui suit a été mesuré ou observé pendant cette passe ;
ce qui n'a pas pu être vérifié est dit explicitement (section « Limites »).

Méthode : le dernier commit (`1acb71f`) a été extrait avec `git archive` dans un
dossier vierge, puis `./docker-cargo.sh lint`, les tests et `./build.sh` y ont été
lancés. Les empreintes SHA-256 de `dist/` de cette extraction et de `dist/` du
dépôt sont identiques (build reproductible). Le jeu a ensuite été joué dans un
vrai navigateur (Chromium piloté par le protocole CDP) servi par `python3 -m http.server`.

Cette première passe portait sur les cinq niveaux du 12/11/2026. La synthèse À JOUR, sur les onze
niveaux de l'étape 5 (courbe de difficulté finale, robot, textes, poids, images par seconde,
captures de chaque niveau, liste de ce qui n'est pas testé), est la section
« QA finale des onze niveaux (carte 5.18) ». Retouches suivantes, à la fin du fichier : « Corbeau
redessiné (carte 6.1) », « Système de vies (carte 6.2) ».

## Résultat par point

| # | Point | Résultat | Chiffres / preuve |
|---|-------|----------|-------------------|
| 1 | Tests `core` | RÉUSSI | 143 réussis, 0 échec, 1 ignoré (régénère `police-test.png`) |
| 1 | Tests `game` (natif, sans son) | RÉUSSI | 112 réussis, 0 échec, 1 ignoré (écrit le script du robot) |
| 1 | `cargo fmt --check` | RÉUSSI | `FMT-OK` |
| 1 | `cargo clippy -D warnings` | RÉUSSI après correction | `LINT-OK` sur core, game wasm32 et game natif (tests inclus). Voir « Corrections » |
| 2 | `./build.sh` depuis propre | RÉUSSI | `dist/` = index.html 7 397 o, mq_js_bundle.js 36 148 o, game.wasm 422 556 o ; total 466 101 o (gzip : 3 140 + 9 839 + 172 990 = 185 969 o) ; le wasm seul fait 0,42 Mo pour une limite de 2,5 Mo |
| 2 | Aucune requête externe | RÉUSSI | le journal du serveur ne contient que `/`, `/game.wasm`, `/mq_js_bundle.js` ; aucune URL `http(s)` dans la page hors l'espace de noms SVG de l'icône ; 0 URL dans le chargeur JS |
| 3 | Commentaires en français | RÉUSSI | 1 601 lignes de commentaire dans 72 fichiers `.rs`, 0 ligne détectée comme anglaise (heuristique par mots-outils) |
| 3 | Aucun asset externe | RÉUSSI | aucun `include_bytes!`, `load_texture`, `load_ttf`, requête réseau ; les seuls PNG versionnés sont dans `docs/` ; sons et sprites sont générés par le code |
| 3 | Palette ≤ 16 couleurs | RÉUSSI | `core/src/palette.rs` : 16 constantes ; sur les captures : 0 pixel hors palette (voir point pixel art) |
| 3 | Accents dans la police | RÉUSSI | tous les accents français et `«»…` présents (test `tous_les_caracteres_francais_sont_presents`) ; lus à l'écran sur le titre, l'aide, les cartes de niveau et la pause (`Échap`, `Écrase`, `Dernière`, `Là-haut`, `PÉPIN`) |
| 3 | Résolution 320x180, échelle entière | RÉUSSI | captures à 640x360 (x2), 1280x577 (x3), 1366x768 en dpr 2 (x8), 1920x1080 (x6), 480x270 (x1) : zone de jeu toujours centrée, bandes sombres autour |
| 3 | Pas de nom d'auteur, pas de lien portfolio | RÉUSSI | noms propres, `portfolio`, `github` : 0 occurrence dans `game.wasm` et `index.html` ; un test (`aucune_mention_hors_sujet`) interdit `http`, `www.`, `@` et des noms propres dans tous les textes |
| 4 | Un robot finit les 5 niveaux | RÉUSSI | `game/src/bot_qa.rs` : recherche du plus court chemin avec la vraie simulation (ennemis, éclairs, plateformes mobiles et friables). Niveaux 1 à 5 finis, 0 mort, 3 gouttes ramassées au bilan, écran de fin atteint. Partie complète : 12 267 images = 204,4 s |
| 4 | Le même parcours dans le vrai navigateur | RÉUSSI | le script des touches du robot est rejoué dans le wasm : titre, 5 cartes, 5 niveaux, écran de fin « LE SOLEIL ! » : TEMPS 3:01,0, GOUTTES D'OR 3/15, MORTS 0, SCORE 500 ; 0 erreur JS, pas de panique |
| 4 | 60 images par seconde | RÉUSSI | en temps réel : 6 500 images en 108,3 s = 60,0 i/s ; plus grand intervalle entre deux images 16,8 ms (aucune image perdue) |
| 4 | Chargement < 2 s | RÉUSSI | cache désactivé, réseau limité à 10 Mbit/s + 40 ms de latence, SANS compression : wasm instancié en 555–649 ms, page prête en 571–661 ms (3 essais). En local sans limite : < 90 ms |
| 5 | Pixel art net (couleurs, flou, semi-transparence) | RÉUSSI sauf l'écran tactile | voir ci-dessous |
| 5 | Mobile 390x844 + contrôles tactiles | RÉUSSI avec réserves | voir ci-dessous |

## Pixel art : mesure sur captures réelles

`docs/mesure-capture.py capture.png K` (K = facteur d'échelle ; j'ai ajouté ce 2e argument,
il valait 3 en dur). Résultat identique sur les 11 captures de bureau mesurées :

- 10 à 16 couleurs distinctes dans la zone de jeu, TOUTES dans la palette ;
- 0 pixel hors palette, 0 pixel semi-transparent ;
- 0 bloc KxK non uniforme sur 57 600 (aucun flou) à x1 (480x270), x2 (640x360),
  x3 (1280x577), x6 (1920x1080) et x8 (1366x768 en dpr 2).

Captures contrôlées à la main : héros, flaque, corbeau, éclair, champignon, lanternes,
gouttes d'or, soleil et nuages sont de vrais sprites à gros pixels (aucune forme vectorielle
lissée). Planche contact des sprites : `docs/planche-sprites.png`.

Exception documentée : les boutons tactiles (point suivant) sont dessinés à la résolution
de l'écran, avec 55 % d'opacité, par-dessus le jeu. Sur la capture mobile paysage la mesure
donne 44 couleurs (29 hors palette) et 461 blocs non uniformes : ce sont uniquement les
pixels sous les boutons. Ce n'est pas corrigé : un bouton opaque cacherait le décor sous le
pouce. Si l'exigence « zéro semi-transparence » doit aussi couvrir les boutons tactiles, il
faut les rendre opaques (contour seul au repos) : petit changement dans
`game/src/tactile.rs`, à décider.

## Mobile (émulation CDP : écran tactile, dpr 2)

- Paysage 844x390 : titre lisible, un toucher sur JOUER lance la partie ; boutons
  gauche, droite (bleu), saut (or) et pause en haut à droite, tous visibles, sans recouvrir
  le HUD ni Pépin ; zone de jeu x2 centrée. Capture `22-mobile-paysage-jeu.png`.
- Multitouch : droite + saut maintenus ensemble : Pépin avance et saute, les deux boutons
  s'éclairent (`24-mobile-saut-multitouch.png`).
- Pause tactile : menu PAUSE / REPRENDRE / RECOMMENCER / SON : non / QUITTER AU TITRE
  (`26-mobile-pause.png`).
- Portrait 390x844 (`20-mobile-portrait-titre.png`, capture d'AVANT la correction) : le jeu
  s'affichait en x2 au milieu de grandes bandes sombres, sans message. Corrigé : voir
  « Portrait téléphone » ci-dessous.

## Portrait téléphone (écran « Tourne ton téléphone »)

Code : `game/src/orientation.rs` (décision, sprites, dessin, 10 tests), branché dans `main.rs`
(une condition) ; `tactile.rs` expose seulement `est_actif()`. Les boutons tactiles n'ont pas été touchés.

- Règle : fenêtre plus haute que large ET un toucher déjà reçu. Un navigateur de bureau
  étroit sans toucher garde le jeu normal (`34-portrait-sans-toucher.png`, titre affiché).
- Tactile en portrait 390x844, dpr 2 : écran plein cadre en pixel art dans la cible 320x180
  (fond ARDOISE_NUIT, téléphone 12x20 / 20x12 qui alterne toutes les 0,9 s, flèche or, texte
  « Tourne ton téléphone » / « pour jouer en paysage » en police 5x7 à l'échelle 2). Zone x2,
  texte lisible (`35-...-a.png`, `36-...-b.png` : les deux images de l'animation).
- Pause : le jeu (simulation, chrono, entrées de menu) ne bouge plus tant que l'écran est
  en portrait. Test : partie lancée en paysage, chrono 0:03 (`38-jeu-avant-rotation.png`),
  5 s en portrait (`39-jeu-en-portrait.png`), retour en paysage : chrono toujours 0:03 et
  Pépin au même endroit (`40-jeu-apres-rotation.png`). Reprise automatique, sans appui.
  Un toucher fait en portrait ne lance rien (la réaction aux appuis est suspendue).
- Titre en paysage après retour : `37-retour-paysage.png`.
- Pixel art mesuré (`docs/mesure-capture.py`, zone de jeu) : 0 pixel hors palette, 0 pixel
  semi-transparent, 0 bloc flou, sur 35 et 36 (x2) et sur 37 (x4).
- Poids : game.wasm 424 350 o (avant 422 556), dist/ 467 895 o (avant 466 101), bien sous 1,2 Mo.
- Lint vert ; tests : 143 core, 112 game (102 + 10 nouveaux), 0 échec.
- Limites : émulation CDP seulement (la rotation est simulée en changeant la taille de la
  fenêtre), pas de vrai téléphone. Entre « hauteur > largeur » et un clavier virtuel ouvert,
  un téléphone en paysage pourrait brièvement paraître en portrait : non observé ici.

## Points demandés par le coordinateur

1. Jambages g/j/p/q/y : lisibles aux échelles 1 et 3 (test `les_lettres_a_jambage_restent_distinctes_a_l_echelle_1_et_3`, et « gouttes », « plein », « ou » lus à l'écran : `14-fin.png`, `01b-aide.png`).
2. Texte d'intro du niveau 5 : c'est un écran de carte dédié (180 pas = 3 s, passé à l'Entrée ou au toucher), PAS une superposition en jeu. Il couvre environ 90 % de la largeur et 60 % de la hauteur et cache en grande partie le soleil pendant ces 3 s (`17-carte-niveau5.png`) ; le texte est lisible. Dès que le niveau commence, plus aucun texte ne recouvre la scène et le soleil est entier (`18-niveau5-apres-carte.png`). Jugé acceptable.
3. Captures réelles en jeu : corbeau (`10b-niveau3-corbeau-proche.png`), éclair (`07-niveau3-eclair.png`, colonne claire pleine hauteur), plateforme mobile (`08-niveau3-plateformes-mobiles.png`, dalle bleutée à droite de la falaise), flaque (`05-niveau2-flaque.png`), champignon (même capture, rouge), nuages du niveau 5 (`06-niveau5-nuages-friables.png`), vents du niveau 4 (`11-niveau4-vents.png`), planer (`04-niveau1-planer.png`).
4. Mesure des couleurs à x1 et x≥3 : faite (section pixel art).

## Corrections apportées (commitées)

- `clippy --all-targets` sur `game` natif signalait 2 erreurs : une dans mon nouveau code de test, une ancienne dans `game/src/sprites/planche.rs` (`repeat().take()` remplacé par `repeat_n`). Le lint existant ne vérifiait que la cible wasm32, donc ne voyait pas les tests de `game` ; `./docker-cargo.sh lint` couvre maintenant aussi `game` natif tests inclus.
- `Monde` dérive `Clone` (le robot explore des futurs possibles sur des copies).
- `docs/mesure-capture.py` : facteur d'échelle en 2e argument.
- README : nombre de tests à jour (102) et description du lint.

## Ce qui a été ajouté pour la QA

- `game/src/bot_qa/` (module, compilé uniquement pour les tests) : le robot (voir
  « Refactor du robot de QA (carte 5.11b) »). `cargo test -p game --no-default-features -- --ignored ecrire_script_navigateur` écrit `target/qa/script.txt` (un chiffre hexadécimal par image : 1 Entrée, 2 gauche, 4 droite, 8 saut) et `trace.csv`. Un petit pilote JavaScript, collé dans la console, rejoue ce script avec une horloge virtuelle.
- `docs/qa/` : 29 captures réelles du navigateur.

## Limites (non vérifié ou à relativiser)

- Les 60 i/s ont été mesurés dans un Chromium piloté, sur cette machine ; pas sur un vrai téléphone ni sur Firefox/Safari. Seul Chromium a été utilisé.
- Le chargement « < 2 s » a été mesuré avec un réseau simulé (10 Mbit/s, 40 ms) et sans compression ; avec gzip il serait plus court, mais ce n'est pas mesuré.
- Le tactile a été émulé (événements tactiles CDP), pas testé au doigt. Le portrait affiche désormais l'écran « Tourne ton téléphone ».
- Le son n'a pas été écouté : aucune sortie audio dans cette session. Les tests de synthèse passent, et aucune erreur JavaScript n'a été capturée pendant toute la partie ; rien de plus n'est affirmé.
- Les plateformes friables du niveau 5 sont franchies par le robot (images 10 555 à 10 619), mais je n'ai pas obtenu de capture qui distingue nettement leur état d'effritement. Le friable du niveau 2 et du niveau 3 non plus. Le comportement est couvert par les tests du cœur, pas par une capture.
- Le robot est une preuve de faisabilité, pas de confort de jeu : il joue sans erreur, un humain peut trouver certains passages difficiles. Le jeu n'a pas été évalué du point de vue du plaisir de jeu.

## Restes à décider (aucun n'est bloquant)

1. Boutons tactiles semi-transparents : les garder ou les rendre opaques (voir plus haut).

## Fonds des niveaux 3 et 4 (décor.rs)

Avant : à l'orage (niveau 3) et en altitude (niveau 4), la couche lointaine ne dépassait que d'environ
20 px du bas de l'écran : le fond était un aplat de ciel. Maintenant (`game/src/decor.rs`) :

- niveau 3 : immeubles hauts (relief +22 px) en bleu nuit sur ciel violet, quelques fenêtres allumées
  (1x2 px, orange/or, environ une fenêtre sur trois par immeuble au plus), couche lointaine prolongée
  jusqu'au bas de l'écran (pas de bande de ciel dans les trouées) ;
- niveau 4 : tours de nuages violettes en 4 étages sur le ciel corail, tours proches sombres, petits
  îlots flottants dans le tiers médian de l'écran (y 58 à 86), parallaxe propre (0,25).

Captures réelles avant/après : `docs/qa/41` à `44` (pilote `docs/capture-fond.mjs`, CDP, Chromium 1280x720).
Rien d'autre ne change : mêmes 16 couleurs de la palette, 0 semi-transparence, wasm 425 510 octets.
Nouveaux tests (decor.rs) : couleurs dans la palette, aucune couche de la couleur du ciel derrière
elle (4 bandes, tous thèmes), silhouette lointaine >= 40 px, fenêtres dans le bâtiment et peu
nombreuses, îlots dans le tiers médian. Résultat : 143 + 116 tests OK, `./docker-cargo.sh lint` OK.
Limite : le contraste est contrôlé par la palette et à l'œil sur captures ; pas de mesure de
luminance entre le héros et le fond.

## Corrections du décor (audit)

Source : RAPPORT d'audit des 5 niveaux (I-03, 04, 06, 16, 17, 18, 19, 20, 21, 22). Seul
`game/src/decor.rs` change. Captures réelles avant/après (Chromium piloté par CDP, 1280x720, début,
milieu et fin de chaque niveau) : `docs/qa/50-niveauN-{debut,milieu,fin}-{avant,apres}.png`
(30 images, pilote `docs/capture-fond.mjs`). Mesure `docs/mesure-capture.py` sur les 30 : 0 pixel hors
palette, 0 pixel semi-transparent, 0 bloc flou.

Fait :

- I-04 : la couche proche descend (32 px pour les immeubles, 24 px pour les tours du niveau 4). Son
  sommet reste sous la ligne du sol (y=132) même avec la parallaxe verticale maximale. Plus de
  silhouette noire posée sur le toit. Test `la_couche_proche_ne_depasse_pas_de_la_ligne_du_sol`.
- I-03 : niveau 5, bancs de nuages du fond en CORAIL_CLAIR (plus la couleur des plateformes), sans
  relief : leurs sommets restent sous la ligne du sol. Test dédié.
- I-22 : niveau 5, nappe proche en BRUME au lieu de BLANC_CASSE (nette sur le ciel OR_PALE).
- I-06 : `socle: true` aux niveaux 1 et 2 : plus de ciel clair sous les immeubles du fond.
- I-17 : fenêtres du niveau 3 en ORANGE_SOLEIL / CORAIL_CLAIR ; l'or est réservé aux gouttes et aux
  éclairs. Test `aucun_decor_n_utilise_l_or`.
- I-18 : dessus des îlots du niveau 4 arrondi (1 px sur un quart de la largeur, puis 1 px sur la moitié).
- I-19 : étoiles du niveau 4 limitées à la bande violette du haut (y < 45) ; niveau 1 inchangé
  (moitié haute). Test sur la couleur du ciel derrière chaque étoile.
- I-20 : densité de pluie par ambiance : 24, 48, 60, 18 gouttes aux niveaux 1 à 4, 0 au niveau 5
  (inchangé). Pluie lointaine du niveau 4 en cyan (PLUIE) : visible sur les quatre bandes.
- I-21 : plafond de nuages d'orage en dômes renversés, deux couches de parallaxe (0,10 et 0,20),
  au niveau 3 (ampleur 4, ardoise nuit / ardoise) et au niveau 2 (ampleur 2, nuages discrets).
  Il tient dans la bande du haut (profondeur max 34 px < 45).

Non fait : I-16 (quatrième bande de ciel du niveau 2 en ARDOISE_PALE), facultatif. Le plafond change
déjà le haut du ciel du niveau 2 ; je n'ai pas testé cette variante, et je ne prétends pas qu'elle
améliorerait le contraste sol/ciel.

Mesures : wasm 426 213 octets (< 1,2 Mo). Tests : 123 (jeu) + 143 (noyau) OK, dont 7 nouveaux tests
du décor ; `./docker-cargo.sh lint` OK.
Limites : jugé à l'œil sur captures, sans mesure de luminance. Toutes les captures sont prises au
début, au milieu et à la fin de chaque niveau avec `docs/capture-fond.mjs` ; je n'ai pas rejoué une
partie complète du robot pour cette carte.

## Corrections des sprites (audit)

Source : RAPPORT d'audit des 5 niveaux (I-05, 10, 11, 23, 27, 28). Fichiers : `game/src/sprites/`
(`objets.rs`, `ennemis.rs`, `tuiles.rs`, `mod.rs` pour les tests), `game/src/scene.rs`, `game/src/hud.rs`.
Aucune hitbox, aucune physique, aucun niveau ASCII ne change. Captures réelles avant/après (Chromium CDP,
onglet au premier plan, 960x540, mêmes images du robot de QA rejouées sur l'ancien et le nouveau `dist/`) :
`docs/qa/60-*-{avant,apres}.png` (40 images). Pilote : `docs/capture-sprites.mjs` (+ `docs/pilote-qa.js`).
Mesure `docs/mesure-capture.py` (K=3) sur les 38 captures : 0 pixel hors palette, 0 bloc flou.

Fait :

- I-05 : goutte d'or redessinée en 10x10 : la goutte de 8x8 cernée de 2 px de contour sombre (les 4
  images). `scene.rs` la dessine décalée de (-1, -1) : la boîte de ramassage (8x8) n'a pas bougé. Le HUD
  (`hud.rs`) la décale aussi, pour qu'elle reste à la même place. Vue sur chaque ciel : bande OR, bande
  ORANGE_SOLEIL, soleil (OR_PALE) au niveau 5 (`60-goutte-n5-*`, zoom `60-zoom-goutte-n5-*`), et niveaux 1
  à 4. Elle se lit partout : contour sombre sur 2 px, contraste 8,9 à 16,1 avec les ciels dorés.
- I-10 : vitre de la lanterne éteinte en ARDOISE_PALE/BRUME (`.045540.`). Contraste calculé : 3,66
  (ARDOISE_PALE) sur l'ardoise, 3,56 (BRUME) sur ARDOISE_CLAIRE, contre 1,18 à 2,19 avant. La lanterne
  allumée ne change pas.
- I-11 : corbeau éclairci d'un cran dans ses deux images (3 devient 4, 2 devient 3). Le contour sombre
  et le bec orange restent ; contraste 3,66 du corps (ARDOISE_PALE) avec la bande ARDOISE du niveau 3.
- I-27 : plateforme mobile : les 3 lignes manquantes sont dessinées (corps, ombre, contour du bas).
  Le dessin fait 32x8, comme la collision.
- I-28 : nouvelle dalle d'ardoise fêlée (2 images, 16x8 comme le nuage) pour les niveaux 1 à 3 ; le
  nuage reste aux niveaux 4 et 5. Choix dans `scene.rs` selon le thème du niveau. Clair sur sombre
  conservé (dessus BLANC_CASSE, corps NUAGE/BRUME).
- I-23 : trois tuiles de nuage (`nuage_bord_g`, `nuage_bord_d`, `nuage_ilot`) à coins arrondis, choisies
  par `scene.rs` quand le voisin de gauche/de droite est vide (nuages à arête éclairée seulement). Les
  tuiles « plein » et la collision ne changent pas.

Tests ajoutés (`sprites/mod.rs`) : goutte 10x10 avec contour de 2 px, vitre de la lanterne éteinte et
contraste, corbeau sans ardoise et avec bec et contour, plateforme mobile sans vide sous le dessin,
friable d'ardoise de la taille du nuage, bords de nuage (tuile 16x16, coins creusés). Les tests de
dimensions et d'indices de palette existants couvrent les nouveaux sprites (banque, planche).

Mesures : wasm 427 740 octets (< 1,2 Mo). Tests : 143 (noyau) + 129 (jeu) OK ; `./docker-cargo.sh lint` OK.
`docs/planche-sprites.png` régénérée par le test de la planche.
Limites : lisibilité jugée à l'œil sur captures, plus les contrastes calculés ci-dessus ; la mesure
automatique par gabarit des gouttes sur capture (ancien et nouveau) a retrouvé les 15 gouttes. Le
contraste de l'or avec le fond (1,0 devant la bande OR) ne change pas : c'est le contour de 2 px qui
rend la goutte lisible.

## Corrections de visibilité (audit)

Carte 3/4 : affichage, effets et calcul d'affichage de la caméra. Aucun changement de physique, de
vitesse, de collision ni de mécanique. Captures `docs/qa/70-*.png` (« avant » = ancien `dist/`, servi
sur un autre port, mêmes images du parcours du robot). Pilote : `docs/capture-visibilite.mjs`
(rejeu du script du robot, séries de captures consécutives, chute libre dans un trou).

- I-01 : petite flèche corail 5x3 px collée en haut de l'écran, à l'abscisse de Pépin (bornée à
  l'écran), tant que sa boîte est entièrement au-dessus du cadre. Dessinée après le monde et la
  pluie, avant le HUD ; pas dessinée quand Pépin est visible. Fonction pure `repere_haut` (6 cas
  testés). Captures : `70-repere-n3-avant.png` (rien) et `70-repere-n3-apres.png` (flèche en haut,
  niveau 3, image 6760). Le mur de la colonne 133 reste pour la carte 4/4.
- I-02 : les brins de vent naissent à une hauteur tirée au hasard sur toute la zone du courant et
  sont des tirets verticaux de 3 px (BLANC_CASSE, contraste fort sur les bandes corail/orange).
  Durée de vie bornée pour ne jamais sortir de la zone ; la force du vent ne change pas. Test :
  200 brins, tous dans la zone et tous de 3 px de haut ; les autres particules restent des points
  de 1 px. Captures : `70-vent-n4-avant.png`, `70-vent-n4-apres.png`, `70-vent-n4-apres-b.png`
  (image 8393 : colonne de tirets de bas en haut).
- I-12 : l'annonce de l'éclair clignote sur les deux bords de la colonne (x0 et x0+w-1), de plus
  en plus vite (de 2,5 Hz à 15 Hz, 7 clignotements en 0,8 s) selon `progression_annonce()` du
  cœur ; barre dorée de 12x2 px au pied, allumée dès le début. Pas de flash plein écran. Tests :
  accélération, 14 changements d'état, aucune série d'une seule image à 60 i/s. Captures :
  `70-eclair-n3-annonce-debut.png`, `70-eclair-n3-annonce-fin.png`, `70-eclair-n3-frappe.png`.
- I-26 : quand la zone de l'éclair commence en haut du niveau (y = 0) et que le niveau est plus bas
  que l'écran, la colonne dessinée est prolongée jusqu'au haut de l'écran (annonce et frappe).
  La zone dangereuse du cœur ne change pas. Fonction pure `colonne_eclair` testée.
- I-15 : l'animation de mort et les éclats sont bornés au cadre (`borner_mort` : 16 px au-dessus
  du bord bas, abscisse dans l'écran) ; la mort par le vide est visible. Captures :
  `70-chute-n1-avant.png` (rien, niveau 1, trou colonnes 30-32) et `70-chute-n1-apres.png`
  (étincelles de mort en bas d'écran) ; la caméra repart tout de suite vers la reprise, donc
  l'animation se joue en bas à droite de l'écran quand la reprise est à gauche.
- I-09 : l'écart Pépin-caméra est arrondi une seule fois, la caméra entière en est déduite
  (`camera_entiere`) ; `app.rs` passe l'abscisse interpolée de Pépin à `position_rendue`. La
  simulation n'est pas touchée. Mesure sur 12 images consécutives du niveau 1 (images 2125 à
  2136, abscisse écran du bord gauche du corps, lue dans la grille 320x180) :
  avant 147 147 147 147 146 147 147 146 147 146 146 146 (4 allers-retours) ;
  après 147 147 147 147 147 147 146 146 146 146 146 146 (aucun). Captures `70-course-n1-00.png`
  à `70-course-n1-11.png`. Tests : l'écart écran reste à moins d'un demi-pixel de l'écart réel ;
  sur course à vitesse constante (pas de 0,4 à 1,6 px) aucun aller-retour ; le décor (caméra
  entière) ne recule jamais ; l'ancien calcul fait échouer ces tests (vérifié). Caméra verticale
  inchangée (test : figée à -4 sur un niveau de 11 rangées, arrondi de y identique sinon).

Mesures : `docs/mesure-capture.py capture.png 3` sur les 24 captures `70-*` : 0 couleur hors
palette, 0 pixel semi-transparent, 0 bloc flou. wasm 428 538 octets (< 1,2 Mo). Tests : 143 (noyau)
+ 142 (jeu) OK ; `./docker-cargo.sh lint` OK.
Limites : mesure à 60 images par seconde (pas testé à 144 Hz) ; la flèche de repère est petite
(15x9 px à l'écran, comme demandé) ; le robot ne tombe jamais, la chute est obtenue en lui faisant
courir tout droit dans le trou de la colonne 30.

## Corrections des niveaux et textes (audit), carte 4/4

Points I-07, I-08, I-12 (texte), I-13, I-14, I-24, I-25 et la partie « niveau » de I-01. Aucune
physique ni mécanique touchée.

- I-07 / I-12 : carte du niveau 3 : « L'orage éclate. Chaque éclair est annoncé : garde la tête
  froide ! » coupée en « L'orage éclate. Chaque éclair est annoncé : » / « garde la tête
  froide ! » (plus de ligne commençant par « : »). Le mot « flash » a disparu du texte.
- I-24 : conseil du niveau 5 : « Dernière ligne droite : le champignon t'aide. » (une ligne de
  45 caractères) ; écran de fin : largeur de coupe 40 -> 42 (`LIGNE_MAX_MOT`), le mot « Il reste
  plein de gouttes d'or à dénicher. » tient sur une ligne. Toutes les autres cartes vérifiées par
  test (aucune ligne ne commence par « : », « ! », « ? », « ; » ou « . » ; chaque ligne <= largeur).
- I-14 : noms de niveau en capitales (`titre_niveau`, accents compris : « GOUTTIÈRES ET
  CHEMINÉES », test : un glyphe existe pour chaque caractère) ; ligne du menu « SON : OUI » /
  « SON : NON ». Titre, pause et fin n'avaient déjà que des capitales ou des textes sans
  descendantes gênantes : rien d'autre à changer.
- I-08 : gouttes d'or descendues d'une rangée (rangée 0 -> 1) : niveau 2 colonne 76, niveau 4
  colonnes 105 et 158, niveau 5 colonne 76. Capture : la goutte est entièrement sous la bande
  de l'interface (`80-niveau4-goutte-col105.png`).
- I-13 : niveau 3, colonne 177, rangée 1 : `N` -> `#` (bout de toit suspendu, plus de nuage isolé).
- I-01 (niveau) : niveau 3, colonne 133 : la brique de la rangée 0 est retirée (mur de 3 rangées
  comme à la colonne 127). Mesure sur le parcours du robot, colonnes 120 à 142 (307 images) :
  images où Pépin est entièrement hors de l'écran (bas du corps au-dessus de y = 0) : 151 ->
  70 ; images où il dépasse en partie : 171 -> 107. Ce n'est pas nul : un saut sur un mur de
  3 rangées (48 px) pour une hauteur de saut de 57 px fait forcément dépasser la tête de Pépin
  au sommet, et le robot monte plus haut que nécessaire (parapluie). Essayé aussi : abaisser
  encore les murs 127 et 133 d'une rangée : 27 images hors écran, mais le niveau devient
  nettement plus facile ; annulé pour ne pas dénaturer le niveau. La flèche de repère de la carte
  3/4 reste le secours.
- I-25 : trame des cartes et de la pause en damier (un pixel sur deux en quinconce) au lieu de
  lignes pleines ; texte lisible sur les cinq ciels (captures `80-carte-niveau1..5.png`).
- Tests ajoutés : noms en capitales avec glyphes, aucune ligne de carte ne commence par une
  ponctuation, coupures du niveau 3 et du niveau 5, mot de fin sur une ligne.

Mesures : `docs/mesure-capture.py capture.png 3` sur les 15 captures `80-*` : 0 couleur hors
palette, 0 flou. wasm 433 771 octets (< 1,2 Mo). Tests : 143 (noyau) + 146 (jeu) OK, robot
de QA : les 5 niveaux finis sans mort (12 254 images) ; `./docker-cargo.sh lint` OK.

## Niveau 2 : lanterne et poche close

Signalement : « niveau 2 : un trou est bouché par un mur bizarrement, et il y a un checkpoint dans
les airs ». Deux défauts confirmés dans `core/src/levels/gouttieres.rs`. Aucune physique ni
mécanique touchée.

- Lanterne flottante : `L` colonne 66, rangée 2, au-dessus d'une rangée 3 vide (toit en rangée 4).
  Descendue en rangée 3 : elle repose sur le toit.
- Poche de vide close : colonne 61, rangées 8 à 10, coiffée par le bloc `BB` des colonnes 60-61 et
  bordée de plein des deux côtés (inaccessible, visible comme un trou sombre sous le mur). Remplie :
  rangée 8 -> `#`, rangées 9 et 10 -> `B`. Les trous accessibles des colonnes 28-30 et 83-85 sont
  intacts.
- Tests ajoutés (`core/src/zones.rs`, 5 niveaux) : `aucune_lanterne_ne_flotte` (case juste dessous
  = `#`, `B` ou `N`) et `aucune_poche_de_vide_close` (aucune colonne de vide coiffée d'un solide et
  bordée de solide à chaque rangée) ; messages en français avec niveau, colonne et rangée. Vérifié
  rouge sur l'ancien niveau 2 (lanterne col. 66 rangée 2 ; poche col. 61 rangées 8 à 10), vert
  ensuite ; les 4 autres niveaux passent sans changement.
- Captures (même image 3301 du parcours du robot, 960x540) : `docs/qa/90-niveau2-lanterne-poche-avant.png`
  (lanterne en l'air) et `docs/qa/90-niveau2-lanterne-poche-apres.png` (lanterne posée sur le toit,
  plus de poche sombre sous le mur, colonnes 58-64). `mesure-capture.py ... 3` : 0 couleur hors
  palette, 0 flou.
- Vérifications : tests 145 (noyau) + 146 (jeu) OK ; faisabilité inchangée ; le robot de QA finit
  les 5 niveaux, 0 mort, script identique à l'ancien (12 254 images) ; `./docker-cargo.sh lint`
  OK ; wasm 433 771 octets.

## Revue du 13 novembre : échelle téléphone, raccourcis, AZERTY, pause automatique, plantage

Trois défauts constatés, corrigés ; aucun niveau, aucune physique, aucune hitbox ne change.

1. Touche M sur AZERTY. Le moteur lit la position physique des touches ; M d'un clavier AZERTY est à la
   position du point-virgule QWERTY. Le son ne s'activait donc pas. `game/src/entrees.rs` porte
   maintenant une table unique de listes de touches (`TOUCHES_*`), lue par `Clavier` et par
   `navigation.rs` : M et point-virgule coupent/activent le son, A et Q vont à gauche, W et Z sautent,
   avec toujours une touche indépendante de la disposition (flèches, Espace, Entrée, Échap).
   Tests : aucune touche en double dans une même situation (en jeu, en menu), au moins une touche fixe
   par action, et un test « AZERTY » qui traduit les lettres imprimées en position physique.
2. Touches 1 à 5. Sur AZERTY ce sont & é " ' ( ; un appui accidentel renvoyait au début d'un niveau,
   remettait le score à zéro et permettait de finir avec un temps truqué. Elles sont retirées de la
   production : fonctionnalité Cargo `debug_niveaux`, activée par `./build.sh --qa` (dossier `dist-qa/`,
   ignoré par git). `./docker-cargo.sh lint` vérifie les deux variantes ; des tests couvrent « aucune
   touche sans la fonctionnalité, même toutes enfoncées » et « chaque touche donne son niveau avec ».
3. Écrans haute densité. `Conf::high_dpi` est maintenant vrai : le canevas a la résolution physique, et
   le facteur entier d'agrandissement se calcule en pixels physiques (le rendu, la souris et le tactile
   convertissaient déjà avec `screen_dpi_scale()`). Avant, un téléphone en paysage de 844x340 pixels CSS
   (DPR 3) donnait un facteur 1 : le jeu occupait 19 % de l'écran ; maintenant facteur 5, 54 %.

Mesures sur Chromium 1208 piloté par CDP (`docs/qa-clavier-ecran.mjs`, matrice `docs/qa-matrice.sh`,
sorties JSON de l'exécution ; images `docs/qa/91-*.png`, « avant » = `dist/` construit depuis le commit
précédent). Les mesures sont des pourcentages de pixels différents ou des rectangles de jeu, vérifiés
à l'œil sur les captures :

| Cas | Avant | Après |
|-----|-------|-------|
| AZERTY : M (position point-virgule) sur le titre | texte « SON » inchangé (6 784 px clairs) | change (6 848 -> 6 336), le 2e appui revient (6 848) |
| QWERTY : M | change | change |
| Touches 1 à 5 en plein jeu, production | les touches 1, 3, 4, 5 rechargent un niveau (écart 43 à 64 % ; la 2 donne 0,5 % : son début ressemble au niveau 1) | aucune réaction (0,4 à 0,5 %, la pluie) |
| Touches 1 à 5, `dist-qa/` | idem | idem (mêmes écarts à 0,01 % près) |
| Téléphone 844x340, DPR 3 : rectangle de jeu | 960x525 px physiques sur 2532x1020 = 320x175 px CSS (19 % de l'écran, facteur 1 CSS) | 1600x875 = 533x292 px CSS (54 %, facteur 5 physique = 1,67 CSS) |
| Téléphone 844x390, DPR 3 | 1920x1050 px physiques = 640x350 px CSS (facteur 6) | 1920x1032 : identique (facteur 6) |
| Bureau 1280x600, DPR 1,5 | facteur 4,5 physique : pixels irréguliers (960x525 px CSS) | facteur 5 entier (1067x573 px CSS) |
| Bureau 1000x560, DPR 1,25 | facteur 3,75 : blocs de 3 et 4 px mélangés (960x525 px CSS) | facteur 3 : blocs uniformes de 3 px (768x418 px CSS) |
| Bureau 1280x720, DPR 1 / 1,25 / 1,5 / 2 | 4 / 5 / 6 / 8 (calculé, non mesuré) | 4 / 5 / 6 mesurés ; 8 mesuré en essai préliminaire |

Netteté (`docs/mesure-capture.py` sur la capture du jeu en cours, bureau 1280x720) : DPR 1 (facteur 4),
DPR 1,25 (5) et DPR 1,5 (6) donnent 0 pixel hors palette et 0 bloc flou. Sur la capture du téléphone, les
boutons tactiles translucides recouvrent la zone mesurée (35 835 px hors palette sur 1 440 000, 205 blocs
« non uniformes » sur 57 600) : ce sont les boutons, pas un défaut d'échelle.

Souris et tactile après correction (DPR 1,5 et 3) : clic sur « JOUER » et sur la 2e ligne, doigt sur
droite, saut, gauche, bouton de pause en haut à droite, doigt sur « REPRENDRE » du menu de pause : chaque
geste change bien l'écran (`docs/qa/91-tel844x340-apres-*`).

Limites : un seul navigateur (Chromium), émulation CDP (pas un vrai téléphone ni un vrai Windows à 125 %) ;
le son n'a pas été écouté, seul l'état affiché de la ligne « SON » a été mesuré. Contrepartie de
`high_dpi` : quand le facteur physique tombe juste sous un entier (1000x560 à 125 % : 3,88 -> 3), le jeu
est un peu plus petit qu'avant (3 contre 3,75), mais net. Sur QWERTY, la touche point-virgule bascule aussi
le son (effet de bord inévitable de la table à deux positions).

### Pause automatique à la perte de focus

Défaut constaté : onglet masqué, autre fenêtre au premier plan ou téléphone verrouillé, le jeu se figeait
mais la musique continuait, puis l'action reprenait sans prévenir au retour.

Faisabilité, vérifiée dans les sources (miniquad 0.4.11, macroquad 0.4.16, cache cargo) : le chargeur
`mq_js_bundle.js` écoute déjà `visibilitychange`, `focus` et `blur` et appelle la fonction exportée
`focus(vrai/faux)` du wasm (« focus » = `document.hasFocus()` ET page visible) ; X11, Windows, macOS et
Android envoient le même signal. miniquad en fait `window_minimized_event` / `window_restored_event`,
que macroquad range dans la file de chaque abonné (`input::utils`). Solution retenue, la plus simple :
`game/src/focus.rs` s'abonne à cette file ; ni touche simulée, ni fonction exportée en plus.

- `Application::focus` (`game/src/app.rs`) : en pleine partie, ouvre l'écran Pause (curseur sur
  REPRENDRE) ; déjà en pause, ne fait rien (pas de seconde ouverture, curseur conservé) ; titre, aide,
  fin : rien. Pendant la carte de présentation ou les volets qui mènent au jeu, la pause est différée
  au premier pas de jeu (avant que le monde ne bouge), et annulée si le focus revient avant.
- La musique et la pluie s'arrêtent pendant toute pause (manuelle ou automatique) et ne repartent qu'à
  la sortie de la pause, par le joueur (`EtatSon` dans `game/src/audio.rs`, testé sans son réel). Les
  sons du menu restent audibles.
- Un onglet masqué ne dessine plus : le jeu ne voit l'événement qu'au RETOUR, à la première image,
  avant tout pas de simulation. La page (`web/index.html`) suspend donc elle-même les AudioContext
  pendant l'absence (sinon la musique continuerait en fond) et les reprend deux images après le retour,
  quand la pause a coupé la musique. Elle les connaît en enveloppant le constructeur `AudioContext`
  avant de charger `mq_js_bundle.js` (le chargeur garde le sien dans une variable privée).
- Aucun saut du temps : le chrono ne compte que les pas de jeu (rien en pause) ; le `PasFixe` plafonne
  déjà une image à 0,25 s et 5 pas (`core/src/timestep.rs`), pas qui tombent dans l'écran de pause ;
  l'accumulateur est en plus remis à zéro à chaque changement de focus.

Tests unitaires (9) : pause ouverte en partie et temps figé (monde et chrono identiques après 600 pas,
reprise à +1 pas exactement), jamais deux fois, rien sur titre/aide/fin ni après des volets vers le
titre ou la fin, carte (pause différée, puis une seule fois), focus revenu pendant la carte (aucune
pause), volets de « recommencer » qui se ferment ou s'ouvrent, boucles sonores coupées en pause.

Preuves dans Chromium 1208 piloté par CDP (`docs/qa-focus-plantage.mjs`, `dist/` de production ;
« avant » = `dist/` du commit précédent). Chromium sans écran ne masque jamais vraiment un onglet : un
script injecté avant la page force `visibilityState` à « hidden », envoie `visibilitychange` et retient
les rappels de `requestAnimationFrame` comme un vrai onglet masqué ; la perte de focus « fenêtre
visible » force `document.hasFocus()` à faux et envoie `blur`. Les boucles sonores sont comptées en
observant `AudioBufferSourceNode.start` (en boucle) et leur arrêt (`disconnect`).

| Cas (son activé, niveau 1 en cours) | Avant | Après |
|-----|-------|-------|
| Onglet masqué 6 s : AudioContext | « running » (la musique continue) | « suspended » |
| Retour sur l'onglet | jeu relancé aussitôt (0,4 % d'écart avec l'image d'avant : la pluie) | écran PAUSE, REPRENDRE en surbrillance ; 0 % d'écart 3 s plus tard (tout est figé) |
| Boucles musique + pluie au retour | 2 | 0 (contextes « running » : les sons du menu marchent) |
| Échap (reprise) | ouvre la pause | jeu à la même place, chrono 0:04 avant le masquage et 0:04 après (6 s masqué + 3 s de pause), 2 boucles relancées |
| Fenêtre visible sans focus (`blur`) | rien | écran PAUSE, contextes suspendus, 0 boucle |
| Titre : onglet masqué, retour, puis `blur` | rien | rien (écran titre, 2 boucles : la musique du titre continue au retour) ; Entrée lance bien la carte du niveau 1 |

Captures : `docs/qa/92-focus-jeu-apres-*` (1 avant le masquage, 2 retour = PAUSE, 4 après Échap,
5 fenêtre sans focus = PAUSE, 6 focus revenu = toujours PAUSE), `92-focus-jeu-avant-1/2` (ancien build :
au retour, le jeu tourne), `92-focus-titre-apres-*` (titre intact, puis carte du niveau 1).

Non testé : un vrai téléphone verrouillé, Safari iOS (qui peut suspendre l'audio de lui-même),
Firefox, un vrai changement d'onglet ou d'application (simulé par script, voir plus haut) ; la version
de bureau native (X11/Windows) n'a pas été lancée ; le son n'a pas été écouté (seuls l'état des
AudioContext et le nombre de boucles sont mesurés).

### Plantage en cours de partie : message au lieu d'une image figée

Défaut constaté : dans `web/index.html`, le message d'erreur ne s'affichait que PENDANT le chargement.
Une panique Rust en pleine partie (« RuntimeError: unreachable ») ou un affichage WebGL perdu figeait
l'image sans explication, musique comprise.

- Après le démarrage, une erreur qui vient VRAIMENT du jeu affiche maintenant un message plein écran
  « PÉPIN s'est arrêté », la cause entre parenthèses et un bouton « Recharger » (qui recharge la page ;
  aucun lien externe), qui reçoit le focus clavier. Les AudioContext sont fermés : plus aucun son.
  Le premier message est gardé (les erreurs suivantes ne l'écrasent pas).
- « Vient du jeu » : fichier `mq_js_bundle.js` ou `game.wasm`, ou message WebAssembly / RuntimeError /
  unreachable / WebGL (le filtre d'origine, inchangé), plus `unhandledrejection` (même tri, ou erreur
  de type `WebAssembly.RuntimeError` / `CompileError` / `LinkError`) et l'événement `webglcontextlost`
  du canevas (macroquad ne sait pas recréer ses textures).
- Filtre conservé : les erreurs des scripts injectés par le navigateur (portefeuille de Brave :
  `window.ethereum`, `selectedAddress` ; « Script error. » sans fichier) ne déclenchent JAMAIS de
  message ; elles sont même écartées avant tout autre test. Écart volontaire avec la demande : le nom
  de fichier `index.html` n'est PAS retenu comme critère, car un script injecté dans la page porte
  souvent l'adresse de la page elle-même (c'est ce qui bloquait l'iPhone sous Brave) ; le script de la
  page protège ses propres appels risqués (suspension du son) par des `try/catch`.
- Au chargement, rien ne change (mêmes messages qu'avant), avec en plus le bouton « Recharger ».
- Build de QA seulement : fonctionnalité Cargo `qa_plantage` (activée par `./build.sh --qa`, vérifiée
  par le lint) : F9 provoque une vraie panique. Le texte de la panique est absent du wasm de production
  (`grep -c 'plantage volontaire'` : 0 dans `dist/`, 1 dans `dist-qa/`). Tests : F9 seule le déclenche,
  aucune touche de jeu ou de menu, et jamais sans la fonctionnalité.

Preuves dans Chromium 1208 piloté par CDP (`docs/qa-focus-plantage.mjs`, scénarios `brave`,
`plantage`, `webgl`, `wasm-bloque`). « Avant » = page du commit précédent.

| Cas | Avant | Après |
|-----|-------|-------|
| Partie en cours, 4 erreurs « Brave » injectées (lecture de `window.ethereum.selectedAddress` par un script en ligne, « Script error. » sans fichier, promesse rejetée « reading 'selectedAddress' », `web3` absent dans une minuterie), `dist/` | pas de message | pas de message ; après chaque erreur Pépin court encore (4,7 à 9,6 % de l'image change), musique et pluie continuent |
| F9 en pleine partie, son activé (`dist-qa/`) | image noire et figée, aucun message, AudioContext « running » | « PÉPIN s'est arrêté » + « (Uncaught RuntimeError: unreachable) » + bouton Recharger (focus clavier dessus), AudioContext « closed » ; même état 2 s plus tard |
| Clic sur « Recharger » | (pas de bouton) | la page se recharge, écran titre, jeu chargé |
| Perte du contexte WebGL en pleine partie (`WEBGL_lose_context`), `dist/` | image figée, aucun message, son qui continue | message « (l'affichage WebGL a été perdu) » + bouton, AudioContext « closed » |
| `game.wasm` bloqué (`Network.setBlockedURLs`) | écran de chargement à 4 s, message « n'a pas démarré après 25 secondes » à 27 s | identique, avec le bouton Recharger en plus |

Captures : `docs/qa/92-brave-apres-*`, `92-plantage-apres-*` (1 jeu, 2 après F9, 3 après Recharger),
`92-plantage-avant-2-plantage-f9.png` (écran noir figé), `92-webgl-avant/apres-2-contexte-perdu.png`,
`92-wasm-bloque-apres-1-apres-27s.png`.

Non testé : un vrai plantage du jeu publié (aucun connu : il a été provoqué) ; Safari iOS (message de
panique différent, « Unreachable code should not be executed », couvert par « unreachable » mais non
essayé) ; un vrai iPhone sous Brave (ses erreurs ont été imitées d'après l'historique : script en
ligne, « Script error. », promesse) ; Firefox ; une vraie perte de contexte par le pilote graphique
(provoquée par l'extension `WEBGL_lose_context`). Une erreur de notre propre script de page qui ne
serait pas protégée ne déclencherait pas de message (choix assumé, voir plus haut).

## Mesure de difficulté

Outil ajouté par la carte 3.1 (`game/src/difficulte.rs`, compilé pour les tests seulement). Il répond à
« combien de fois meurt-on dans chaque niveau ? » par un chiffre reproductible, à comparer avant/après
chaque carte qui touche un niveau, un ennemi ou une mécanique.

**Facultative depuis la carte 6.5** (décision d'AGA du 2026-11-16) : AGA juge lui-même la difficulté en
jouant. Les cartes ne relancent plus la mesure, ne régénèrent plus `docs/mesure-difficulte.txt` (il garde
la mesure de la carte 6.4b) et ne sont plus bloquées par le juge de la courbe, qui est désormais un test
ignoré. L'outil reste entier, à lancer À LA MAIN quand AGA le demande (commandes ci-dessous ; durées
mesurées dans « Mesure de difficulté facultative (carte 6.5) », en fin de fichier). Restent obligatoires
pour toute carte : faisabilité, lint de grille, robot de QA qui finit les onze niveaux sans mourir (lancés
avec tous les tests).

### Commande

    ./docker-cargo.sh test -p game --no-default-features mesure_de_difficulte -- --ignored --nocapture

Elle affiche les tableaux et réécrit `docs/mesure-difficulte.txt` (fichier versionné : `git diff` montre
l'effet d'une carte). Durée : 30 à 90 s sur la machine de développement à la carte 3.1 (un fil d'exécution
par niveau puis par niveau et profil ; la durée dépend de la charge) ; à la carte 6.5, la même mesure
(par le juge de la courbe) a pris 536 s, machine chargée (charge moyenne de 7 à 19). Le test échoue si un
profil finit moins de la moitié de ses essais sur un niveau, si l'ordre Débutant > Habitué > Expert n'est
plus respecté sur le total, ou si la forme de la courbe sort de ses bandes (voir ci-dessous).

Depuis la carte 3.3, le fichier a une 3e partie, « Lieux de mort » : pour chaque niveau et chaque profil,
les endroits où l'on meurt (colonne de tuile du centre de Pépin, cause : chute, ennemi, pointes, éclair),
colonnes voisines de même cause regroupées, du plus meurtrier au moins meurtrier. C'est là qu'il faut
agir pour adoucir ou corser un niveau.

Depuis la carte 6.2 (vies), une 4e partie, « Game over », ajoutée À LA FIN (les trois premières ne
changent pas d'un octet) : pour chaque niveau et chaque profil, les essais où Pépin perd sa dernière vie
au moins une fois, le niveau commencé avec 5 vies ; une estimation sur une partie enchaînée ; et les
niveaux normaux où le Débutant dépasse 10 % de game over, SIGNALÉS (voir « Système de vies (carte 6.2) »).

Deux autres tests :
- `courbe_de_difficulte_dans_les_bandes` (IGNORÉ depuis la carte 6.5 ; avant, il était lancé avec tous
  les tests) : le juge de la forme de la courbe sur la mesure officielle (`ecarts_de_la_courbe`,
  `game/src/difficulte.rs`), sans réécrire aucun fichier. À la main :

      ./docker-cargo.sh test -p game --no-default-features courbe_de_difficulte_dans_les_bandes -- --ignored

  Depuis la carte 5.18 (courbe finale sur onze niveaux) : Débutant et Habitué, le boss (6) est le plus
  meurtrier de tous sauf le bonus, la mer (8) l'est plus que les vents (7), et les vents plus que le
  final (9) ; Habitué, la courbe ne redescend jamais avant le boss (niveaux 1 à 6), chaque niveau est dans
  sa bande (`BANDES_HABITUE`) et les vents au moins 0,15 sous le boss ; Expert, le bonus dure de 45 à 75 s ;
  Débutant et Habitué, l'épilogue n'est jamais plus meurtrier qu'un autre niveau. En cas d'échec, le
  message liste tous les écarts et le tableau. (Avant la carte 5.18 : le sommet était le grand orage, voir
  les sections des cartes 3.3 à 5.17.) Le test synthétique `la_forme_de_la_courbe_est_jugee` (mesures
  inventées, instantané) reste dans la suite normale : il vérifie le juge lui-même.
- `bruit_de_la_mesure` (ignoré, environ 2 min 30) : la même mesure avec 4 graines (l'officielle et ses 3
  voisines) et les lieux de mort cumulés sur toutes. Sert à savoir si un écart est réel ou dû au hasard :
  une différence plus petite que l'écart entre graines ne prouve rien. (Une variable `GRAINES=a,b,c`
  choisit d'autres graines, mais `docker-cargo.sh` ne la transmet pas au conteneur : seulement avec un
  `cargo` local.)

      ./docker-cargo.sh test -p game --no-default-features bruit_de_la_mesure -- --ignored --nocapture

Quand AGA demande un tableau avant/après : lancer la commande sur une extraction propre du commit de départ
(`git archive <commit> | tar -x -C <nouveau dossier>`), puis sur le commit final, et recopier les deux
tableaux « Morts moyennes » dans la section de la carte. Ne jamais changer `GRAINE`, `ESSAIS` ni `PROFILS`
d'une carte à l'autre (sinon les chiffres ne sont plus comparables) ; si c'est indispensable, refaire la
mesure « avant » avec les nouvelles valeurs.

### Méthode : intention parfaite, exécution imparfaite

1. Itinéraires : pour chaque niveau, toutes les cases où Pépin peut se poser sont explorées avec les plans
   du robot de QA (courir, sauter, planer, attendre ; case = 8 px en x, y au pixel), puis une passe à
   rebours depuis le drapeau (Dijkstra) donne, pour chaque case, le plan le plus rapide vers le drapeau.
   243 à 685 cases par niveau. Si les plans grossiers ne suffisent pas, les plans précis du robot prennent
   le relais (même repli que le robot).
2. Le joueur simulé joue dans la vraie simulation (`Monde`, comme le robot). Avant chaque plan, il
   l'essaie sur une copie du monde : il « voit » l'ennemi arriver et ne choisit qu'un plan sûr qui le
   rapproche du drapeau ; posé hors des cases connues (après une erreur), il essaie les conseils des cases
   voisines, puis un court détour calculé, puis il attend 1/3 s.
3. Il exécute ensuite ce plan avec, selon son profil, une erreur tirée au hasard parmi : élan trop court ou
   trop long (saut parti trop tôt / trop tard), touche de saut lâchée trop tôt / trop tard, parapluie ouvert
   en retard, direction gardée ou lâchée à tort en redescendant.
4. Une mort le renvoie à la lanterne comme dans le jeu (animation de réapparition comprise, le monde
   continue d'avancer). Les morts comptées sont celles du compteur du jeu (`etat.morts`).

Hasard : petit générateur maison (xorshift64*, graine mélangée par splitmix64), graine fixe 20261113 ;
chaque essai a sa propre graine (profil, niveau, numéro d'essai), donc le résultat ne dépend pas de l'ordre
des fils d'exécution. 48 essais par profil et par niveau.

| Profil   | Gestes ratés | Élan        | Durée du saut | Parapluie en retard |
|----------|--------------|-------------|---------------|---------------------|
| Débutant | 40 %         | ± 12 pas    | ± 8 pas       | jusqu'à 30 pas      |
| Habitué  | 20 %         | ± 8 pas     | ± 5 pas       | jusqu'à 15 pas      |
| Expert   | 8 %          | ± 4 pas     | ± 3 pas       | jusqu'à 6 pas       |

(1 pas = 1/60 s ; un « geste » = un plan entier, de l'élan jusqu'à l'atterrissage.)

### Mesure de référence (commit de la carte 3.1, aucun niveau modifié)

Morts moyennes par niveau ; entre parenthèses : sans les morts « en série » (moins d'1 s après la
précédente, voir le constat plus bas).

| Niveau                      | Débutant     | Habitué      | Expert       |
| --------------------------- | ------------ | ------------ | ------------ |
| 1. Les toits                | 0,4 (0,4)    | 0,1 (0,1)    | 0,0 (0,0)    |
| 2. Gouttières et cheminées  | 6,9 (1,4)    | 1,2 (0,4)    | 0,2 (0,2)    |
| 3. Le grand orage           | 3,6 (3,6)    | 1,0 (1,0)    | 0,2 (0,2)    |
| 4. Vents d'altitude         | 2,9 (2,9)    | 0,9 (0,9)    | 0,3 (0,3)    |
| 5. Au-dessus des nuages     | 2,5 (2,5)    | 0,4 (0,4)    | 0,2 (0,2)    |
| Total                       | 16,3 (10,8)  | 3,6 (2,8)    | 0,9 (0,9)    |

Les 720 essais finissent tous (détail, durées et pire essai : `docs/mesure-difficulte.txt`). Lecture :
le niveau 1 est presque sans danger, le niveau 3 (éclairs, corbeaux) est le plus meurtrier hors morts en
série, la difficulté redescend aux niveaux 4 et 5.

Vérifié :
- Reproductible : trois exécutions (dont une sur extraction propre du commit final) donnent un fichier
  `docs/mesure-difficulte.txt` identique à l'octet ; test `niveau_1_sans_erreur_aucune_mort_et_essais_reproductibles`
  (même graine = même essai, mort pour mort ; sans erreur = aucune mort au niveau 1).
- Robot de QA inchangé : sa recherche a été rendue réutilisable (`bot_qa::chercher`), mais
  `target/qa/script.txt` et `trace.csv` sont identiques à l'octet avant/après ; il finit toujours les
  cinq niveaux sans mourir.
- Tests du générateur (bornes, même graine même suite) et des erreurs (la direction ne change jamais,
  « attendre » n'est jamais perturbé, écarts bornés par le profil).

### Constat : morts en série à la lanterne du niveau 2

La mesure a révélé un défaut du jeu (non corrigé ici : cette carte ne touche ni niveau ni mécanique).
Niveau 2, lanterne de la colonne 104 : deux flaques (colonnes 98 et 108) patrouillent sur le même toit et
passent sur la lanterne. Si Pépin meurt après l'avoir allumée et qu'une flaque passe sur la lanterne,
il y réapparaît sur elle et meurt de nouveau à CHAQUE pas (aucune protection pendant la réapparition :
`Monde::avancer` traite les contacts même quand `reapparition > 0`), jusqu'à ce que la flaque s'éloigne.
Débutant, essai 0 : 46 morts en 46 images (0,8 s), Pépin en (1667, 114) ; pire essai : 57 morts. C'est
presque tout l'écart entre 6,9 et 1,4 au niveau 2. Piste pour une carte à venir (décision du
coordinateur) : une courte invulnérabilité à la réapparition, ou une lanterne hors de la patrouille.
Corrigé par la carte 3.1b : voir « Protection après réapparition (carte 3.1b) ».

Preuve en vrai navigateur : `ecrire_script_morts_en_serie` (test ignoré) écrit `target/qa/script-31.txt`
(le robot finit le niveau 1, puis le niveau 2 est joué par cet essai, entrées exactes) ; rejoué dans
Chromium 1208 piloté par CDP sur le build de QA servi en local (`SCRIPT=target/qa/script-31.txt node
docs/capture-visibilite.mjs PORT DOSSIER ...`, onglet au premier plan, canevas focalisé).
Captures `docs/qa/31-essai-debutant-*` : 1 avant (deux flaques sur le toit), 2 Pépin réapparu sur la
lanterne, la flaque collée à lui, 3 éclats de mort, 4 toujours au même endroit après 25 morts, 5 la flaque
partie, Pépin repart (0 pixel hors palette, 0 flou, `docs/mesure-capture.py`, K = 3).

### Limites

- Un modèle, pas des joueurs : réflexes parfaits (il voit chaque danger avant d'agir), erreurs tirées
  uniformément, ni fatigue ni apprentissage. Les chiffres absolus sont indicatifs ; seule la comparaison
  avant/après à graine égale a un sens.
- Le joueur simulé ne ramasse pas les gouttes d'or (il va droit au drapeau) : les détours vers les gouttes
  ne sont pas mesurés.
- Pas de tactile, pas de manette : les erreurs ne dépendent pas de l'appareil.
- Le son n'intervient pas. Aucune vérification sur un vrai téléphone ni sur Safari/Firefox (rien d'affiché
  ne change dans cette carte).

## Portée de patrouille du corbeau (carte 3.2)

Ce qui change (`core/src/entities/ennemi.rs`) : le corbeau mémorise son point d'apparition et fait
demi-tour quand il s'en éloigne de `PORTEE_CORBEAU` (5 tuiles = 80 px de chaque côté, soit une patrouille
de 10 tuiles, la moitié de l'écran logique, 4 s pour un aller à 40 px/s), OU quand il touche un mur : le
premier des deux atteint. Le pas qui dépasserait la limite est raccourci pour s'arrêter pile dessus (pas de
dérive au fil des allers-retours). La portée est une `Option<f32>` par type d'ennemi : `None` pour la
flaque, dont la patrouille ne change pas. Sans portée, un corbeau posé dans un niveau ouvert (niveau 4,
aucun mur en altitude) aurait traversé toute la carte : il pourra y être placé à la carte 3.3.

Aucun niveau modifié. Le seul corbeau existant (niveau 3, colonne 130) vole entre deux murs de briques
distants de 5 tuiles (colonnes 127 et 133) : son couloir fait 68 px, moins que sa portée, les murs restent
donc la limite et son vol est identique (abscisse de 2048 à 2116 px avant comme après).

Tests ajoutés : `en_monde_ouvert_le_corbeau_oscille_autour_de_son_point` (grille vide de 40 tuiles :
x min et x max exactement à 80 px de part et d'autre du point, au moins 5 demi-tours et 5 passages par le
point en 1500 pas, altitude constante), `le_mur_fait_faire_demi_tour_au_corbeau_avant_sa_portee` (mur à
2 tuiles à gauche : demi-tour contre le mur ; à droite, rien : demi-tour à la portée),
`la_flaque_n_a_pas_de_portee` (`None`, et sur un toit de 28 tuiles entre deux murs elle va d'un mur à
l'autre, bien au-delà de 80 px). Les tests existants du corbeau (rebond aux murs du couloir) et des flaques
passent sans changement.

Vérifié sur une extraction propre du commit (`git archive`) :
- `./docker-cargo.sh test --workspace --no-default-features` : 148 + 171 tests, tous verts ;
  `./docker-cargo.sh lint` : `LINT-OK` ; `./build.sh` : `dist/` produit, 0 occurrence des noms interdits,
  seule URL `http://www.w3.org/2000/svg`.
- Robot de QA : finit les cinq niveaux sans mourir ; `target/qa/script.txt` et `trace.csv` identiques à
  l'octet avant/après (même position du corbeau à chaque image).
- Faisabilité : tests verts (les corbeaux n'y entrent pas, voir `core/src/faisabilite.rs`).
- Mesure de difficulté relancée : `docs/mesure-difficulte.txt` identique à l'octet. Morts moyennes par
  niveau, avant → après :

| Niveau                      | Débutant            | Habitué            | Expert             |
| --------------------------- | ------------------- | ------------------ | ------------------ |
| 1. Les toits                | 0,4 → 0,4           | 0,1 → 0,1          | 0,0 → 0,0          |
| 2. Gouttières et cheminées  | 6,9 → 6,9           | 1,2 → 1,2          | 0,2 → 0,2          |
| 3. Le grand orage           | 3,6 → 3,6           | 1,0 → 1,0          | 0,2 → 0,2          |
| 4. Vents d'altitude         | 2,9 → 2,9           | 0,9 → 0,9          | 0,3 → 0,3          |
| 5. Au-dessus des nuages     | 2,5 → 2,5           | 0,4 → 0,4          | 0,2 → 0,2          |

Captures (Chromium 1208 piloté par CDP, build de QA servi en local sur 127.0.0.1, onglet au premier plan,
canevas focalisé, serveur arrêté ensuite ; script du robot rejoué avec `docs/capture-visibilite.mjs`) :
`docs/qa/32-niveau3-corbeau-approche.png` (image 6689, Pépin arrive, corbeau dans son couloir),
`32-niveau3-corbeau-mur-gauche.png` (6760, corbeau contre le mur gauche, demi-tour),
`32-niveau3-corbeau-pepin-dans-la-cage.png` (6794, Pépin entre les deux murs, corbeau derrière lui),
`32-niveau3-corbeau-mur-droit.png` (6862, corbeau contre le mur droit). Chaque capture regardée ; 0 pixel
hors palette, 0 flou (`docs/mesure-capture.py`, K = 3).

Pas testé : un corbeau en monde ouvert DANS un niveau (aucun n'existe encore ; couvert par le test unitaire,
à voir en jeu à la carte 3.3), vrai téléphone, Safari/Firefox, son (rien de sonore ne change).

## Protection après réapparition (carte 3.1b)

Le défaut trouvé par la mesure 3.1 (morts en série à la lanterne de la colonne 104 du niveau 2, jusqu'à 57
d'affilée) est corrigé pour tous les niveaux et tous les dangers, sans toucher aux niveaux.

Ce qui change :
- `core/src/player.rs` : `PROTECTION_REAPPARITION_PAS` = 60 pas (1 s), compteur `protection` du joueur
  (`proteger`, `est_protege`, `protection_restante`), décompté à chaque pas. Durée FIXE (bouger ne l'arrête
  pas), jamais cumulée (`proteger` remet à 60, n'ajoute pas), `placer` seul ne protège pas. 1 s parce
  qu'une flaque (24 px/s) met 0,92 s à passer entièrement sur Pépin immobile (22 px de chevauchement).
- `core/src/state.rs` : `EtatJeu::traiter` donne la protection à chaque mort, en replaçant Pépin au
  checkpoint (lanterne ou départ). C'est la SEULE source de protection : ni lanterne, ni écrasement, ni
  champignon ne la donnent ; elle ne se recharge qu'à une réapparition.
- `core/src/entities/contacts.rs` : protégé, aucun danger ne tue (ennemi touché de côté ou par-dessous,
  pointe, éclair ; tout danger futur ajouté à `resoudre_contacts` doit suivre la même règle, écrit en tête
  de la fonction). Le reste continue : gouttes, lanternes, champignons, écrasement d'un ennemi par le dessus.
  La chute dans le vide (`verifier_vide`) reste mortelle.
- `game/src/scene.rs` : Pépin clignote pendant toute la protection (`pepin_cache` : 3 images dessiné,
  3 images non dessiné, 10 fois par seconde). Image absente plutôt que transparente : rien hors palette.
  Avant, le même clignotement ne durait que les 18 pas de l'animation de mort.

Tests ajoutés (tous verts) :
- `player::la_protection_de_reapparition_expire` : protégé pendant exactement 60 pas, puis plus ; pas de
  cumul ; `placer` ne protège pas.
- `contacts::protege_aucun_danger_ne_tue_mais_le_reste_continue` : flaque de côté + pointe + éclair qui
  frappe + goutte : sans protection, mort ; protégé, la goutte seulement ; protégé, l'écrasement marche.
- `contacts::le_vide_tue_meme_protege` et `state::la_protection_ne_vient_que_d_une_reapparition`
  (lanterne, écrasement, rebond : pas de protection ; une mort : protection ; le vide tue même protégé ;
  rien en pause).
- `state::une_flaque_qui_passe_sur_la_lanterne_ne_tue_qu_une_fois` : le scénario sur un petit toit, pas
  de jeu complet (entités, joueur, contacts, état) : 1 mort. Contrôle : la même ligne `joueur.proteger()`
  retirée, ce test échoue avec 55 morts (pas 227 à 281), comme la mesure 3.1.
- `monde::la_flaque_sur_la_lanterne_104_du_niveau_2_ne_tue_plus_en_boucle` : le vrai niveau 2, Pépin
  immobile à la lanterne 104 pendant 6 s : au plus 2 morts (un passage de la flaque à l'aller, un au
  retour), jamais deux à moins de 60 pas d'intervalle, Pépin toujours à la lanterne.
- `scene::pepin_clignote_pendant_toute_la_protection` : 30 images cachées sur 60, par séries de 3.
- `zones::lanternes_sur_le_trajet_d_un_danger` : inventaire ci-dessous (à mettre à jour avec les niveaux).

Lanternes sur le trajet d'un danger (les entités des 5 niveaux avancent seules pendant 2 min ; on note
chaque lanterne dont le point de réapparition est touché par une flaque, un corbeau, une pointe ou un
éclair qui frappe) : 3 lanternes, toutes sur la patrouille d'une flaque qui longe son toit d'un bout à
l'autre. Niveau 2, colonne 66 (flaque de la colonne 70, toit des colonnes 62 à 82) ; niveau 2, colonne 104
(flaque de la colonne 108, toit des colonnes 102 à 118 : le cas de la mesure 3.1) ; niveau 3, colonne 145
(flaque de la colonne 154, toit de 61 tuiles). Aucun corbeau, aucune pointe, aucun éclair sur une
lanterne. Aucune lanterne déplacée : la protection suffit (au plus une mort par passage de la flaque, comme
n'importe où ailleurs sur le toit). (Carte 5.12 : le grand orage est devenu le niveau 4, la liste est
désormais (2, 66), (2, 104), (4, 145) ; le marché couvert n'ajoute aucune lanterne sur le trajet d'un
danger, chats compris.) (Carte 6.3 : + (3, 50) et (3, 110), deux lanternes du marché couvert à portée
du BOND d'un chat ; voir « Chats plus menaçants (carte 6.3) ».)

Vérifié sur une extraction propre du commit 8d2a66a (`git archive`) :
- `./docker-cargo.sh test --workspace --no-default-features` : 154 + 173 tests, tous verts ;
  `./docker-cargo.sh lint` : `LINT-OK` ; `./build.sh` : `dist/` produit, 0 occurrence des noms interdits,
  seule URL `http://www.w3.org/2000/svg` ; `./build.sh --qa` : `dist-qa/` produit.
- Robot de QA : finit les cinq niveaux sans mourir ; `target/qa/script.txt` et `trace.csv` identiques à
  l'octet avant (96d043e) et après (le robot ne meurt jamais, la protection ne joue donc jamais).
- Faisabilité : tests verts (aucun niveau modifié).
- Mesure de difficulté relancée (`docs/mesure-difficulte.txt`, identique sur l'extraction propre et dans le
  dépôt). Morts moyennes par niveau, avant → après (entre parenthèses : hors morts en série) :

| Niveau                      | Débutant              | Habitué               | Expert                |
| --------------------------- | --------------------- | --------------------- | --------------------- |
| 1. Les toits                | 0,4 → 0,4             | 0,1 → 0,1             | 0,0 → 0,0             |
| 2. Gouttières et cheminées  | 6,9 (1,4) → 1,5 (1,5) | 1,2 (0,4) → 0,5 (0,5) | 0,2 → 0,2             |
| 3. Le grand orage           | 3,6 → 3,6             | 1,0 → 1,0             | 0,2 → 0,2             |
| 4. Vents d'altitude         | 2,9 → 2,9             | 0,9 → 0,9             | 0,3 → 0,3             |
| 5. Au-dessus des nuages     | 2,5 → 2,5             | 0,4 → 0,4             | 0,2 → 0,2             |
| Total                       | 16,3 → 10,9           | 3,6 → 2,9             | 0,9 → 0,9             |

  Niveau 2, pire essai : Débutant 57 → 5 morts, Habitué 36 → 3, Expert 1 → 2 ; morts en série : 5,5 → 0,
  0,8 → 0, 0 → 0 (plus aucune dans tout le jeu). Niveaux 1, 3, 4 et 5 : lignes identiques à l'octet. Les
  petits écarts du niveau 2 hors série (1,4 → 1,5 ; un essai Expert à 2 morts au lieu de 1 ; durée moyenne
  Habitué et Expert + 1 s) viennent de ce qu'après une mort l'essai suit un autre cours (Pépin repart plus
  tôt, protégé, à un autre moment de la patrouille) : bruit de la mesure, pas un effet du niveau.

Preuve en vrai navigateur (Chromium 1208 piloté par CDP, build de QA `dist-qa/` de l'extraction propre servi
sur 127.0.0.1, onglet au premier plan, canevas focalisé, serveur arrêté ensuite). `ecrire_script_lanterne_104`
(test ignoré, remplace `ecrire_script_morts_en_serie` qui ne trouve plus aucune mort en série ; pour
régénérer `script-31.txt`, extraire 96d043e) écrit `target/qa/script-31b.txt` : le robot finit le niveau 1,
puis le niveau 2 est joué par le MÊME essai que les captures 31- (Débutant n° 0). Avant : 46 morts en 46
images à la lanterne 104. Après : 1 seule mort dans tout l'essai, à l'image 3932, Pépin réapparu en
(1667, 114) DANS la flaque ; la flaque le recouvre pendant 43 images (3932 à 3974), toutes protégées, puis
Pépin repart vers la droite. Rejoué avec `SCRIPT=target/qa/script-31b.txt node docs/capture-visibilite.mjs`.
Captures `docs/qa/31b-*` (demande = image de la trace) :
- `31b-1-saut-vers-la-lanterne` (3925) : Pépin saute vers la lanterne, la flaque est collée à elle ;
- `31b-2-mort-et-reapparition` (3932) : éclats de mort sur la lanterne allumée, la flaque dessus, Pépin
  caché (clignotement) ;
- `31b-3-protege-visible-dans-la-flaque` (3936) : Pépin dessiné, debout dans la flaque, vivant ;
- `31b-4-protege-cache-flaque-sur-la-lanterne` (3951) : Pépin caché, la flaque sur la lanterne ;
- `31b-clignote-00` à `05` (3953 à 3958) : dessiné, dessiné, caché, caché, caché, dessiné ;
- `31b-5-protege-visible-flaque-qui-passe` (3966) : Pépin dessiné, la flaque sort par sa gauche ;
- `31b-6-protection-finie-pepin-repart` (3995) : protection finie, Pépin saute vers la droite, la flaque
  repart vers le mur de gauche.
Chaque capture regardée ; 0 pixel hors palette, 0 flou (`docs/mesure-capture.py`, K = 3). Remarque : dans
le navigateur, le motif du clignotement est décalé d'une image par rapport au tableau du test (cachées :
demandes 3955 à 3957 au lieu de 3956 à 3958 ; même rythme 3/3). Décalage de numérotation entre le pilote et
la trace, cause non recherchée ; les autres captures sont prises au milieu d'une phase de 3 images, donc
insensibles à ce décalage.

Pas testé : vrai téléphone, Safari/Firefox, ressenti d'un vrai joueur sur la durée de 1 s (assez longue
pour laisser passer une flaque ; elle permet aussi de traverser une flaque juste à côté de la lanterne
pendant cette seconde, voulu), son (rien de sonore ne change), grêle (pas encore écrite : elle devra passer
par `resoudre_contacts` pour être couverte).

## Retouches de difficulté des niveaux 3, 4 et 5 (carte 3.3)

Cible (addendum du coordinateur, profil Habitué = joueur moyen, même ordre pour le Débutant) : le niveau 3
le plus meurtrier des cinq ; le 4 nettement en dessous du 3 mais au-dessus du 5 ; le 5 sous 0,2 ; les
niveaux 1 et 2 quasi inchangés. Toutes les retouches demandées par AGA sont faites, puis le niveau 4 est
réglé sur les lieux de mort mesurés.

Outil d'abord (commit a304115) : la mesure note maintenant OÙ l'on meurt (colonne, cause ; 3e partie de
`docs/mesure-difficulte.txt`, voir « Mesure de difficulté »). Morts moyennes inchangées à l'octet.

Retouches (coordonnées : rangée 0 en haut, colonnes comptées depuis 0) :
- Niveau 3 (`orage.rs`) : cheminée d'une tuile en colonne 51 (toit en rangée 8, briques en dessous),
  point de repos entre les deux plateformes mobiles. La plateforme 1 s'arrête en colonne 49 au lieu de 50 :
  large de 2 tuiles, elle serait sinon entrée dans la cheminée.
- Niveau 4 (`vents.rs`) :
  - îlot du milieu de l'escalier de nuages (rangée 5, colonnes 72-75) en nuage friable ; sur ce niveau le
    friable se dessine déjà en nuage (`objets::FRIABLE`, thème Altitude), aucun sprite à changer ;
  - îlots d'atterrissage des courants 2 et 3 raccourcis d'une tuile (colonnes 55 et 102 retirées) ;
  - 2 corbeaux (portée 5 tuiles) : colonne 60 rangée 2, à hauteur de Pépin au-dessus de l'îlot d'arrivée
    du courant 2 ; colonne 178 rangée 7, sur le dernier toit de nuages, avant le drapeau ;
  - réglage : après les retouches ci-dessus, le niveau 4 montait à 1,2 (plus dur que le 3, 0,6). Les lieux
    de mort (4 graines cumulées) désignaient les atterrissages : toit de départ (colonne 14), îlot du
    courant 1 (colonnes 34 à 38), îlots après le courant 4 (113-114, 124-125, 143-144, 153-154) et
    dernier toit (163-164) élargis. Aucun îlot demandé « raccourci » n'est rallongé.
- Niveau 5 (`nuages.rs`) : adouci là où l'on mourait (lieux de mort, 4 graines cumulées, Débutant :
  colonnes 59-77, 26-36, 111-115, 83-96, 17-18, 120-123, 127-131). Îlots élargis d'une ou deux tuiles
  (colonnes 17, 27, 35, 43, 57, 68-69, 88-90, 113, 121, 127-129), traversée friable continue (colonnes
  60 à 66 au lieu de 4 dalles espacées), petit nuage d'appui en colonnes 111-112. Gouttes d'or non
  déplacées : elles restent des détours facultatifs (en hauteur, par le trampoline), plus exigeants que le
  chemin principal.
- README : mécanique du niveau 4 (nuage friable, corbeaux).

Mesure de difficulté (graine officielle), morts moyennes avant (d1efab7) → après :

| Niveau                      | Débutant      | Habitué      | Expert       |
| --------------------------- | ------------- | ------------ | ------------ |
| 1. Les toits                | 0,4 → 0,4     | 0,1 → 0,1    | 0,0 → 0,0    |
| 2. Gouttières et cheminées  | 1,5 → 1,5     | 0,5 → 0,5    | 0,2 → 0,2    |
| 3. Le grand orage           | 3,6 → 3,0     | 1,0 → 0,6    | 0,2 → 0,4    |
| 4. Vents d'altitude         | 2,9 → 1,3     | 0,9 → 0,4    | 0,3 → 0,1    |
| 5. Au-dessus des nuages     | 2,5 → 0,5     | 0,4 → 0,1    | 0,2 → 0,0    |
| Total                       | 10,9 → 6,8    | 2,9 → 1,7    | 0,9 → 0,7    |

Bruit (test `bruit_de_la_mesure`, 4 graines : 20261113 à 20261116), Habitué, avant → après :
N3 1,0 / 1,1 / 1,3 / 1,0 → 0,6 / 1,0 / 1,0 / 0,8 ; N4 0,9 / 0,9 / 0,9 / 1,1 → 0,4 / 0,4 / 0,5 / 0,5 ;
N5 0,4 / 0,5 / 0,6 / 0,9 → 0,1 / 0,1 / 0,1 / 0,1. Débutant après : N3 3,0 / 2,4 / 3,5 / 3,4 ;
N4 1,3 / 1,1 / 1,6 / 1,3 ; N5 0,5 / 0,5 / 0,6 / 0,4. Sur les 4 graines, l'ordre 3 > 4 > 5 tient pour les
deux profils, le 4 reste dans [0,35 ; 0,65] et le 5 sous 0,2.

Lecture : la courbe Habitué devient 0,1 · 0,5 · 0,6 · 0,4 · 0,1 (monte jusqu'au 3, redescend au 4, finit
calme). Le niveau 3 baisse aussi (1,0 → 0,6 sur la graine officielle, 1,1 → 0,85 en moyenne des 4) : en
partie le point de repos de la colonne 51, en partie le hasard (une retouche change la suite des tirages de
chaque essai après elle : les chutes des colonnes 136-138, loin de la retouche, passent de 9 à 3). L'Expert
du niveau 3 passe de 0,2 à 0,4 sur la graine officielle (avant 0,2 à 0,3 selon la graine, après 0,2 à
0,4) : bruit. Niveaux 1 et 2 : lignes identiques.

Test de non-régression de la courbe (commit 4bdb074) : `courbe_de_difficulte_dans_les_bandes`, lancé avec
tous les tests (environ 30 s), vérifie sur la mesure officielle la forme ci-dessus et les bandes Habitué
(détail dans « Mesure de difficulté ») ; `la_forme_de_la_courbe_est_jugee` vérifie le juge lui-même
(la courbe d'avant la carte est refusée : niveau 4 à 0,90, niveau 5 à 0,40, 4 pas nettement sous le 3).

Autres vérifications (sur une extraction propre du commit 7b11de2, `git archive`) :
- `./docker-cargo.sh test --workspace --no-default-features` : 154 + 176 tests, tous verts ;
  `./docker-cargo.sh lint` : `LINT-OK` ; `./build.sh` : `dist/` produit, 0 occurrence des noms interdits,
  seule URL `http://www.w3.org/2000/svg` ; `docs/mesure-difficulte.txt` régénéré identique à l'octet.
- Faisabilité (`zones.rs`, `faisabilite.rs`) : les 5 niveaux finissables, toutes gouttes et lanternes
  atteignables, reprise depuis chaque lanterne, robot « maladroit » ; lint de grille vert (lanternes
  posées, aucune poche de vide close).
- Lanternes sur le trajet d'un danger : inventaire inchangé [(2, 66, flaque), (2, 104, flaque),
  (3, 145, flaque)] ; aucun des deux corbeaux du niveau 4 n'atteint une lanterne (le plus proche,
  colonne 178, patrouille de 173 à 183 ; lanterne en 170).
- Robot de QA : finit les cinq niveaux sans mourir (12130 images au lieu de 12254, 5 gouttes au lieu
  de 4 : son parcours change avec les niveaux) ; script identique sur l'extraction propre et pour les
  captures.

Captures (Chromium 1208 piloté par CDP, build de QA servi sur 127.0.0.1, onglet au premier plan, canevas
focalisé, serveur arrêté ensuite ; script du robot rejoué par `docs/capture-visibilite.mjs`). « Avant » :
build de QA d'une extraction propre de d1efab7 et son propre script ; « après » : HEAD. Même endroit du
parcours du robot (même colonne de Pépin), donc pas forcément la même image. Toutes regardées ;
0 pixel hors palette, 0 flou (`docs/mesure-capture.py`, K = 3) sur les 14.
- `33-niveau3-colonne51-avant` / `-apres` : avant, un trou de 16 tuiles sous les deux plateformes
  mobiles ; après, la cheminée (toit sur briques) entre elles, Pépin saute de la plateforme 1 vers elle.
- `33-niveau4-ilot-courant2-avant` / `33-niveau4-ilot-courant2-corbeau-apres` : îlot d'arrivée du
  courant 2 ; après, le corbeau vole au ras de l'îlot pendant que Pépin plane au-dessus.
- `33-niveau4-escalier-avant` / `33-niveau4-escalier-friable-apres` : l'escalier de nuages ; après,
  l'îlot du milieu est fait de 4 petits nuages friables, entre deux îlots pleins.
- `33-niveau4-ilot-courant3-avant` / `-apres` : îlot d'arrivée du courant 3, une tuile plus court.
- `33-niveau4-drapeau-avant` / `33-niveau4-drapeau-corbeau-apres` : dernier toit de nuages ; après, le
  corbeau arrive vers Pépin, à sa hauteur, juste après la dernière lanterne.
- `33-niveau5-friables-avant` / `-apres` : avant, 4 dalles friables espacées ; après, une traversée de
  7 nuages friables jointifs, les deux premiers en train de s'effriter derrière Pépin.
- `33-niveau5-montee-avant` / `-apres` : la montée vers le soleil, îlots plus larges.

Limites : la mesure est un joueur simulé (intention parfaite, exécution imparfaite), pas un vrai joueur ;
le niveau 4 Habitué (0,4) est un peu sous le niveau 2 (0,5), ce que la cible n'interdit pas (elle ne compare
le 4 qu'au 3 et au 5) ; le niveau 3 a baissé lui aussi (point de repos et hasard ; toujours le plus dur,
au-dessus du bas de sa bande, 0,5, mais de peu sur la graine officielle : 0,6). Pas testé : vrai téléphone,
Safari/Firefox, ressenti d'un vrai joueur face aux corbeaux du niveau 4, son (rien de sonore ne change).

## Variantes de brique et ombre sous les toits (carte 4.1)

Habillage des tuiles, dessin seulement : aucune collision, aucun niveau, aucun ennemi ni aucune
mécanique ne change (la mesure de difficulté n'est donc pas relancée ; ses tests de non-régression
restent verts).

- `game/src/sprites/tuiles.rs` : trois variantes de `BRIQUE_PLEIN`, même taille 16 x 16, mêmes
  couleurs, mêmes bords (le raccord avec les tuiles voisines est intact) :
  `BRIQUE_FISSURE` (trait sombre en biais sur deux assises), `BRIQUE_MANQUANTE` (trou d'une brique,
  ombre portée en haut, fond sombre), `BRIQUE_COULURE` (traînée humide sombre qui part d'un joint et
  finit en goutte). Une première version de la coulure était cyan : sur capture, elle se confondait
  avec les gouttes de pluie qui tombent devant le mur ; refaite en ardoise (le cyan reste à la pluie
  et aux flaques). Et `OMBRE_TOIT` : bande de 16 x 2 px (0 contre le rebord, puis 1), deux niveaux
  d'ombre, sans dégradé.
- `game/src/scene.rs` : `variante_brique(tx, ty)`, fonction pure de la position (hachage entier fixe
  `hachage_case`, aucun aléatoire) : une brique pleine sur 6 tire une variante ; une case y renonce si
  sa voisine de gauche ou du dessus a tiré la même, donc deux variantes identiques ne se touchent
  jamais (ni côte à côte, ni l'une sur l'autre). La bande d'ombre se pose par-dessus la brique dont
  la case du dessus est un toit (`ombre_sous_toit`) : c'est la première tuile pleine sous le rebord.
- Dans les vrais niveaux : 229 variantes sur 1291 briques pleines (17,7 %, une sur 5,6) ;
  niveau 1 : 70 sur 423 (fissure 22, manquante 25, coulure 23), niveau 2 : 79 sur 420, niveau 3 :
  76 sur 418, niveau 4 : 4 sur 30, niveau 5 : aucune brique. Bandes d'ombre : 164, 157, 161, 15, 0.
- Tests (`scene.rs`) : choix fixe (valeurs de référence du hachage et de la rangée 9), répartition
  sur 16 000 cases (entre 1/8 et 1/5, chacun des trois dessins entre 25 et 42 % des variantes),
  voisines jamais identiques, vrais niveaux (part des variantes entre 1/8 et 1/5 ; plus de 100 bandes
  d'ombre aux niveaux 1 à 3, aucune au 5), règle de l'ombre (toit sur brique seulement).
  (`sprites/mod.rs`) : chaque variante a les bords de la brique pleine, 4 à 63 pixels changés, aucun
  trou transparent ; l'ombre fait 16 px de large, 1 à 2 px de haut, en couleurs 0 et 1 seulement.
  Planche de sprites `docs/planche-sprites.png` régénérée.

Captures (Chromium 1208 piloté par CDP, build de QA servi sur 127.0.0.1, onglet au premier plan, canevas
focalisé, serveur arrêté ensuite ; script du robot `target/qa/script.txt` rejoué par
`docs/capture-visibilite.mjs`, mêmes images du parcours avant et après). « Avant » : build de QA de
b5da413 ; « après » : build de QA avec ces commits. Toutes regardées ; 0 pixel hors palette, 0 flou
(`docs/mesure-capture.py`, K = 3) sur les 12.
- `41-niveau1-debut-avant` / `-apres` (image 350) : toit bas puis toit surélevé ; après, la bande
  d'ombre court sous les deux rebords, deux briques manquantes et quelques fissures et coulures fines
  dans le mur.
- `41-niveau1-briques-avant` / `-apres` (image 1348) : l'escalier de toits ; une bande d'ombre sous
  chaque marche, variantes clairsemées dans les grands murs.
- `41-niveau2-debut-avant` / `-apres` (image 2615) et `41-niveau2-briques-avant` / `-apres`
  (image 3365) : thème des gouttières (briques plus claires) ; la flaque (ennemi cyan) ne se confond
  avec aucune variante.
- `41-niveau3-debut-avant` / `-apres` (image 5201) et `41-niveau3-briques-avant` / `-apres`
  (image 6619) : l'orage, le grand mur sous la cheminée.

Vérification sur une extraction propre de 76b3e0f (`git archive`, nouveau dossier daté) :
- `./docker-cargo.sh test --workspace --no-default-features` : 154 + 183 tests, tous verts (7 de
  plus qu'à la carte 3.3) ; `./docker-cargo.sh lint` : `LINT-OK` ; `./build.sh` : `dist/` produit,
  0 occurrence des noms interdits, seule URL `http://www.w3.org/2000/svg`.
- `docs/planche-sprites.png` régénérée par les tests de l'extraction : identique à l'octet à celle du
  commit. Robot de QA : `target/qa/script.txt` et `trace.csv` identiques à l'octet à ceux de la carte
  3.3 (le dessin ne change rien au jeu).
- Les 6 captures « après » refaites sur le build de QA de l'extraction : 5 identiques à l'octet aux
  captures commitées ; la 6e (`41-niveau3-debut-apres`) diffère de 666 pixels, tous dans un carré de
  38 x 50 px autour des pieds de Pépin, au-dessus du toit (gouttes de pluie et éclaboussure) : murs et
  ombres identiques.

Limites : jugé sur captures à l'échelle 3 (960 x 540) ; la fissure est plus discrète que le trou à
cette taille (voulu : variantes rares et calmes, le mur ne doit pas attirer l'œil plus que les
dangers). Pas testé : vrai téléphone, Safari/Firefox ; aucun son ne change.

## Objets de décor sans collision (carte 4.2)

Sur le modèle des panneaux : une lettre dans la grille, un sprite, aucune physique.

- `core/src/level.rs` : `GenreDecor` (`H` cheminée, `T` antenne, `R` paratonnerre, `Q` manche à air,
  `K` oiseau posé ; `lettre` / `depuis_lettre`, une seule table) et `ObjetDecor` dans `Niveau::decors`.
  Ni tuile ni entité : la grille de collision, les entités et les courants ne les voient pas. Légende
  en tête du fichier mise à jour.
- `core/src/level/ascii.rs` : le parseur lit ces lettres et refuse un décor qui n'est pas posé sur un
  solide (`ErreurNiveau::DecorSansAppui`, ligne et colonne comme les autres erreurs).
- Placement (`core/src/levels/*.rs`), 25 objets, tous au milieu d'un toit, loin des dangers et des
  repères : niveau 1, 4 cheminées et 4 antennes ; niveau 2, 2 et 2 (les toits libres y sont rares :
  flaques partout) ; niveau 3, 3 paratonnerres (pas sous une colonne d'éclair : on ne doit pas croire
  que l'éclair y tombe) ; niveau 4, une manche à air au bord gauche de chacun des 4 courants
  ascendants (colonnes 13, 46, 92 et 124) ; niveau 5, 6 oiseaux.
- `game/src/sprites/objets_decor.rs` : cheminée 12 x 20 (chapeau de pierre, deux mitrons, corps en
  brique violette des murs), antenne 15 x 24 (trois barres), paratonnerre 7 x 24, manche à air
  16 x 22 (rayures orange et blanc, contraste fort sur le ciel corail du niveau 4), oiseau 12 x 8
  en 2 images (tête tournée). Contour sombre de 1 px ; ni or, ni cyan, ni corail (réservés aux
  gouttes, à l'eau et à Pépin). Planche de sprites régénérée.
- `game/src/scene.rs` : dessinés juste après les tuiles (Pépin, ennemis, panneaux et objets passent
  devant), centrés dans leur case, posés sur la tuile du dessous. La manche à air s'étire vers le
  courant le plus proche (retournée s'il est à gauche ; aujourd'hui les 4 sont à gauche de leur
  courant). Chaque oiseau tourne la tête 2/3 s toutes les 5 s, décalé de 97 pas d'un oiseau à
  l'autre (jamais tous ensemble).

Tests :
- `core/src/level/tests.rs` : lecture des 5 lettres (positions, cases restées vides, aucune entité
  créée), aller-retour lettre <-> genre, décor en l'air ou sur la dernière ligne refusé, message.
- `core/src/zones.rs`, sur les vrais niveaux : chaque objet a 3 solides sous lui (jamais au bord d'un
  toit, là où l'on prend son élan) ; en remplaçant les lettres de décor par `.`, le niveau est
  identique (toutes les tuiles, entités, courants, panneaux, départ et arrivée) ; chaque niveau a
  son décor (au moins 3 objets, les bons genres) ; chaque manche à air est à 4 tuiles au plus d'un
  courant, à une hauteur qu'il traverse ; pendant 2 min de jeu (plus qu'un aller-retour de chaque
  ennemi), aucune entité (ennemi, pointe, colonne d'éclair, plateforme, lanterne, goutte,
  champignon) ni aucun panneau, ni le départ, ni l'arrivée ne touche la place d'un objet (sa case et
  celle du dessus, élargies de 4 px). Contrôle de ce dernier test : une cheminée posée exprès en
  colonne 12 du niveau 2 le fait échouer (flaque au pas 276), puis retirée.
- `game/src/sprites/mod.rs` : taille (une case de large au plus, moins de deux de haut, posé),
  contour sombre, aucune couleur réservée, l'oiseau ne bouge que la tête.
- `game/src/scene.rs` : coin du sprite, sens de la manche sur le vrai niveau 4, rythme de l'oiseau.

Jeu inchangé (dessin seulement) :
- Faisabilité, lint de grille (lanternes posées, aucune poche close) et lanternes sur le trajet d'un
  danger : verts, sans retouche.
- Robot de QA : `target/qa/script.txt` et `trace.csv` régénérés, identiques à l'octet à ceux de la
  carte 4.1 (12 130 images, 0 mort, 5 gouttes).
- Mesure de difficulté relancée : `docs/mesure-difficulte.txt` identique à l'octet au fichier commité
  (aucune différence avant/après) :

| Niveau                      | Débutant     | Habitué      | Expert       |
| --------------------------- | ------------ | ------------ | ------------ |
| 1. Les toits                | 0,4 (0,4)    | 0,1 (0,1)    | 0,0 (0,0)    |
| 2. Gouttières et cheminées  | 1,5 (1,5)    | 0,5 (0,5)    | 0,2 (0,2)    |
| 3. Le grand orage           | 3,0 (3,0)    | 0,6 (0,6)    | 0,4 (0,4)    |
| 4. Vents d'altitude         | 1,3 (1,3)    | 0,4 (0,4)    | 0,1 (0,1)    |
| 5. Au-dessus des nuages     | 0,5 (0,5)    | 0,1 (0,1)    | 0,0 (0,0)    |
| Total                       | 6,8 (6,8)    | 1,7 (1,7)    | 0,7 (0,7)    |

Captures (Chromium 1208 piloté par CDP, build de QA servi sur 127.0.0.1, port libre, onglet au
premier plan, canevas focalisé, serveur arrêté ensuite ; script du robot rejoué par
`docs/capture-visibilite.mjs`, mêmes images avant et après). « Avant » : build de QA de 5edb6c7 ;
« après » : build de QA de ces commits. Toutes regardées ; 0 pixel hors palette, 0 flou
(`docs/mesure-capture.py`, K = 3) sur les 14.
- `42-niveau1-debut-avant` / `-apres` (image 717) : une antenne à gauche de la lanterne, une
  cheminée à droite, sur le toit du début.
- `42-niveau1-toit-haut-avant` / `-apres` (image 1524) : le toit surélevé, cheminée et antenne de
  part et d'autre de Pépin, la goutte d'or au-dessus, dégagée.
- `42-niveau2-fin-avant` / `-apres` (image 4774) : antenne, cheminée, puis le drapeau, sans
  chevauchement.
- `42-niveau3-paratonnerres-avant` / `-apres` (image 5906) : un paratonnerre sur le grand toit, hors
  de la colonne d'éclair annoncée (pointillé or) à droite ; clair sur le ciel violet, il ne se
  confond pas avec l'éclair.
- `42-niveau4-manche-avant` / `-apres` (image 7824) : la manche à air au bout du premier toit, tendue
  vers le courant (brins blancs qui montent) ; `42-niveau4-manche-nuage-avant` / `-apres`
  (image 9164) : celle du nuage haut, devant la bande corail.
- `42-niveau5-oiseaux-avant` / `-apres` (image 11652) : deux oiseaux posés sur les nuages (l'un tête
  tournée).

Limites : jugé sur captures à l'échelle 3 ; l'oiseau (12 x 8) est petit, voulu (il ne doit pas
passer pour un ennemi). Les objets sont devant les silhouettes du fond mais derrière Pépin : il passe
devant une cheminée sans la heurter, ce qui est le comportement attendu d'un décor. Pas testé : vrai
téléphone, Safari/Firefox ; aucun son ne change.

Vérification sur une extraction propre de 7e9868a (`git archive`, nouveau dossier daté) :
- `./docker-cargo.sh test --workspace --no-default-features` : 161 + 190 tests, tous verts (14 de
  plus qu'à la carte 4.1) ; `./docker-cargo.sh lint` : `LINT-OK` ; `./build.sh` : `dist/` produit,
  0 occurrence des noms interdits, seule URL `http://www.w3.org/2000/svg`.
- `docs/planche-sprites.png` régénérée par les tests de l'extraction : identique à l'octet à celle du
  commit.
- Les 7 captures « après » refaites sur le build de QA de l'extraction (mêmes images du rejeu) et
  regardées : même scène, décor au même endroit. Une est identique à l'octet
  (`42-niveau1-debut-apres`) ; les 6 autres sont décalées d'un pixel logique en x (la caméra suit la
  position interpolée de Pépin, et le rejeu ne fige pas toujours au même instant entre deux pas) ;
  une fois ce décalage compensé, il ne reste que des différences éparses (pluie, parallaxe du fond).
  Le même décalage d'un pixel existe entre certaines paires avant/après : il vient du rejeu, pas du
  décor.

## Parapluie fermé et attente de Pépin (carte 4.3)

- `game/src/sprites/pepin.rs` : le parapluie fermé des 8 images (repos 2, course 4, saut, chute)
  est redessiné. Avant : un bâton corail de 2 px, sans poignée, qui montait et descendait avec le
  corps. Maintenant : crosse orange recourbée vers Pépin (la poignée), toile corail repliée avec des
  plis en corail clair et une sangle blanche, plus large en haut qu'en bas, embout de métal ; contour
  sombre de 1 px. C'est le MÊME dessin, à la même place, dans les 8 images : il ne tremble plus
  pendant la course. Le corps de Pépin n'a pas changé (même boîte, mêmes pieds).
- 3 images d'attente, même silhouette que le repos : `pepin_cligne` (les yeux deviennent un trait),
  `pepin_ciel` (yeux et joues remontés d'un pixel : il regarde s'il pleut), `pepin_tapote` (le
  parapluie descend d'un pixel, l'embout touche le sol). Planche de sprites régénérée.
- `game/src/monde.rs` : `Monde::immobile`, tenu par la fonction pure `compter_immobile` : +1 à
  chaque pas où Pépin est au sol, à vitesse nulle, sans touche de direction ni de saut ; 0 sinon.
  Elle ne fait que LIRE le joueur : aucun changement de physique (`core/` n'a pas bougé).
- `game/src/scene.rs` : fonction pure `image_attente` : rien avant 2 s (120 pas) ; ensuite un cycle
  de 5 s qui recommence : clignement (8 pas), nez en l'air (1 s), clignement, puis 4 coups de
  parapluie (« tac » 8 pas, parapluie relevé 8 pas) ; le reste du temps (plus de la moitié), la
  respiration habituelle. Au premier mouvement la minuterie retombe à 0 : l'attente cesse net.

Tests (13 de plus côté `game`) :
- `sprites/mod.rs` : même parapluie dans les 8 images ; crosse en crochet, toile, plis, sangle,
  embout, plus large en haut ; contour sombre ; les images d'attente ne changent que les yeux et les
  joues (ou le parapluie seul pour le tapotement) ; même taille que le repos.
- `monde.rs` : la minuterie compte au sol immobile (sans débordement) ; repart de 0 sur une touche
  (même sans bouger, contre un mur), une vitesse, en l'air ; dans le vrai niveau 1, 3 s sans toucher
  à rien puis une touche ; deux mondes aux mêmes touches restent identiques quand l'un oublie sa
  minuterie à chaque pas (elle ne change rien au jeu).
- `scene.rs` : rien avant 2 s ; contenu du cycle (durées) ; les 4 coups ; répétition (même après
  1000 cycles) ; arrêt au premier mouvement avec la vraie minuterie.

Jeu inchangé (dessin seulement) :
- Robot de QA : `target/qa/script.txt` et `trace.csv` régénérés, identiques à l'octet à ceux de la
  carte 4.2 (12 130 images).
- Mesure de difficulté relancée : `docs/mesure-difficulte.txt` identique à l'octet au fichier
  commité ; mêmes morts moyennes qu'à la carte 4.2 (total 6,8 / 1,7 / 0,7).

Captures (Chromium 1208 piloté par CDP, fenêtre 1280 x 720 donc image agrandie x4, build de QA servi
sur 127.0.0.1, port libre, onglet au premier plan, canevas focalisé, serveurs arrêtés ensuite).
Script du robot rejoué par `docs/capture-visibilite.mjs` (nouveau mode `attente:` : à l'image 250,
au début du niveau 1, le robot lâche son script et Pépin reste immobile), mêmes images avant et
après. « Avant » : build de QA de 06caa93 ; « après » : build de QA de ces commits. L'image de Pépin
affichée est reconnue dans la grille logique par `docs/reperer-pepin.py`, qui découpe aussi les
vues rapprochées `43-zoom-*` (64 x 48 pixels logiques autour de Pépin, x4, sans rééchantillonnage).
Toutes regardées ; 0 pixel hors palette, 0 flou (`docs/mesure-capture.py`, K = 4) sur les 12
captures pleines.
- `43-repos-avant` / `-apres` (image 255, `pepin_idle0`) ;
- `43-course-avant` / `-apres` (image 330, `pepin_course0`) ;
- `43-saut-avant` / `-apres` (image 395, `pepin_saut`) ; `43-chute-avant` / `-apres` (image 419,
  `pepin_chute`) : la crosse et la sangle se lisent devant le ciel d'ardoise ;
- attente, après : `43-attente-cligne-apres` (image 354, `pepin_cligne`), `43-attente-ciel-apres`
  (image 410, `pepin_ciel`), `43-attente-tapote-apres` (image 530, `pepin_tapote`) ;
  `43-attente-avant` (image 410 du build d'avant) : Pépin respire, rien d'autre ;
- une vue `43-zoom-…` pour chacune des 12.
Exploration préalable (une capture toutes les 8 images de 258 à 570, non versionnée) : respiration
jusqu'à l'image 346, clignement à 354, nez en l'air de 410 à 466, clignement à 474, tapotements
alternés avec le repos de 530 à 570, comme prévu par `image_attente`.

Limites : jugé sur captures x4 et sur la planche ; le tapotement n'est qu'un pixel de déplacement
du parapluie (lisible en mouvement, discret à l'arrêt). Pas testé : vrai téléphone, Safari/Firefox ;
aucun son (le tapotement est muet).

Vérification sur une extraction propre de aea406c (`git archive`, nouveau dossier daté) :
- `./docker-cargo.sh test --workspace --no-default-features` : 161 + 203 tests, tous verts (13 de
  plus qu'à la carte 4.2) ; `./docker-cargo.sh lint` : `LINT-OK` ; `./build.sh` : `dist/` produit,
  0 occurrence des noms interdits, seule URL `http://www.w3.org/2000/svg`.
- `docs/planche-sprites.png` régénérée par les tests de l'extraction : identique à l'octet à celle du
  commit. Script du robot régénéré dans l'extraction : identique à l'octet.
- Les 7 captures « après » refaites sur le build de QA de l'extraction (mêmes images du rejeu) et
  regardées : même image de Pépin reconnue dans chacune. Quatre sont identiques à l'octet (repos,
  clignement, nez en l'air, tapotement : Pépin immobile) ; course, saut et chute sont décalées d'un
  pixel logique en x (caméra) et Pépin d'un ou deux pixels en y, comme à la carte 4.2 : le rejeu ne
  fige pas toujours au même instant entre deux pas (position interpolée).

## Pigeon, deuxième ennemi des niveaux 4 et 5 (carte 4.4)

- `core/src/entities/ennemi/pigeon.rs` (sous-module de `ennemi.rs`) : `TypeEnnemi::Pigeon`, lettre
  `p`. Perché, il guette un cône sous lui et devant lui : Pépin entièrement plus bas que ses pattes
  (de 0 à 64 px), entre la verticale (6 px de tolérance vers l'arrière) et la diagonale à 45° vers
  l'avant. Pépin dedans : annonce 0,3 s (il tremble, ailes dressées, œil corail), plongée en
  diagonale à 45° (4 tuiles vers le bas et vers l'avant, 120 px/s, sens fixé au déclenchement),
  remontée deux fois plus lente sur le même trait jusqu'à son perchoir exact, repos 1,5 s avant de
  pouvoir replonger. Machine à états pure (`PhasePigeon`, `phase_suivante`, `decalage_vol` : la
  position est recalculée depuis le perchoir à chaque pas, sans dérive). `guetter` est appelé par
  `resoudre_contacts` : le jeu, le robot de QA et la mesure ont les mêmes pigeons sans rien ajouter.
  Écrasable par le dessus (perché ou en vol), tout autre contact tue ; protection de réapparition
  respectée (même règle que les autres ennemis).
- Grille : `p` est perché sur la tuile du dessous (sinon erreur typée `PigeonSansPerchoir`). Il
  regarde du côté où son perchoir s'arrête : à droite seulement si le perchoir continue à gauche
  et s'arrête à droite, sinon à gauche (vers le départ). Légende de `core/src/level.rs` à jour.
- 6 dessins 16x12 (`game/src/sprites/ennemis.rs`) : perché (2 images, il hoche la tête), annonce,
  plongée, remontée (la scène le retourne : il regarde son perchoir), écrasé. Ardoise sombre, deux
  barres noires sur l'aile, cou irisé (sarcelle et cyan), œil orange sous un sourcil sombre ; rien
  de blanc. Planche de sprites régénérée.
- Placement (2 pigeons, chacun sur un petit nuage d'une tuile ajouté 4 rangées au-dessus d'un
  toit) : niveau 4, colonne 167, au-dessus du dernier toit (avant le corbeau et le drapeau) ;
  niveau 5, colonne 108, au-dessus du long toit de la fin, à 13 tuiles de l'oiseau de décor de la
  colonne 95 et à 8 de celui de la colonne 116. Tous deux regardent vers l'arrivée de Pépin : ils
  plongent derrière quelqu'un qui court, et ne touchent que celui qui s'arrête, hésite ou revient
  sous eux. Premier essai refusé : un pigeon au-dessus du toit qui suit le 1er courant du niveau 4
  (colonne 40, sur le chemin des réapparitions) donnait Habitué 0,52 au niveau 4, refusé par
  `courbe_de_difficulte_dans_les_bandes` (le 4 doit rester 0,15 sous le 3, à 0,65).
- Exigences du coordinateur (oiseau de décor contre pigeon) : silhouette plus grosse (16x12 contre
  12x8, 116 pixels pleins contre 53), corps ardoise contre corps blanc (contraste 10:1
  entre les deux couleurs principales), aucune couleur claire de l'oiseau, regard froncé ; aucun
  oiseau de décor à moins de 4 tuiles d'un pigeon (testé sur les vrais niveaux) ; aucun oiseau
  déplacé ni retiré.
- Robot de QA : `trace.csv` donne aussi la position et la phase du premier pigeon (pour viser les
  captures). Il n'a rien eu à apprendre : il subit la vraie simulation et écarte les plans mortels.

Tests (14 de plus côté `core`, 4 côté `game`) :
- `pigeon.rs` : déclenchement seulement dessous et devant (pas à côté, pas au-dessus, pas trop
  loin, pas derrière ; cône retourné avec le pigeon) ; plongée à 45° vers l'avant, dans les deux
  sens ; sens fixé au déclenchement ; retour exact au perchoir à 1 px par pas ; pas de nouvelle
  plongée pendant le repos même si Pépin reste dessous ; durée du cycle ; écrasé (en vol aussi) il
  ne bouge plus et ne guette plus ; flaque et corbeau ne guettent pas.
- `contacts.rs` : écrasé par le dessus, tue de côté ou en montant ; la plongée tue Pépin immobile
  sous lui, pas avant la fin de l'annonce ; écrasé en pleine plongée.
- `level/tests.rs` : position, boîte, sens selon le perchoir ; pigeon sans perchoir refusé.
- `zones.rs` (vrais niveaux) : 1 à 3 pigeons aux niveaux 4 et 5, aucun ailleurs ; jamais à moins
  de 4 tuiles d'un oiseau de décor ; plongée entièrement dans le ciel (aucune tuile solide, sauf
  le coin de son perchoir), ni sur une lanterne ni sur le départ. Vérifié en les cassant exprès
  (copie jetable) : plongée de 5 tuiles (elle entre dans le dernier toit du niveau 4), pigeon à 3
  tuiles d'un oiseau, pigeon dont la plongée passe sur la lanterne 102 du niveau 5 : 3 échecs.
- `sprites/mod.rs` : 6 images 16x12 au contour sombre ; perché, seule la tête bouge ; différences
  avec l'oiseau de décor (ci-dessus) ; contraste d'au moins 3:1 entre le corps (ardoise) et le
  contour du pigeon et chaque bande de ciel qu'il traverse, perché puis jusqu'au fond de sa plongée
  (niveau 4 : corail, corail clair, orange ; niveau 5 : les quatre bandes). Le violet du haut du
  ciel du niveau 4 (1,5:1 contre l'ardoise) y est interdit : le test refuserait un pigeon perché
  sur les îlots des rangées 2 et 3.
- `scene.rs` : une image par phase, tremblement seulement pendant l'annonce, retourné à la
  remontée, écrasé quelle que soit la phase.

Mesure de difficulté (graine officielle), morts moyennes avant (755df73) → après :

| Niveau                      | Débutant      | Habitué      | Expert       |
| --------------------------- | ------------- | ------------ | ------------ |
| 1. Les toits                | 0,4 → 0,4     | 0,1 → 0,1    | 0,0 → 0,0    |
| 2. Gouttières et cheminées  | 1,5 → 1,5     | 0,5 → 0,5    | 0,2 → 0,2    |
| 3. Le grand orage           | 3,0 → 3,0     | 0,6 → 0,6    | 0,4 → 0,4    |
| 4. Vents d'altitude         | 1,3 → 1,3     | 0,4 → 0,4    | 0,1 → 0,1    |
| 5. Au-dessus des nuages     | 0,5 → 0,5     | 0,1 → 0,1    | 0,0 → 0,0    |
| Total                       | 6,8 → 6,8     | 1,7 → 1,8    | 0,7 → 0,7    |

Le niveau 4 reste dans [0,35 ; 0,65] et nettement sous le 3, le niveau 5 sous 0,2. Le pigeon du
niveau 5 tue une fois (Habitué et Débutant, colonne 107, « ennemi ») sur 48 essais ; celui du
niveau 4 aucune fois (les 4 morts « ennemi » colonnes 180-182 chez le Débutant sont le corbeau de la
colonne 178 : la suite des tirages change dès que le pigeon modifie un choix). Bruit (4 graines,
`bruit_de_la_mesure`), Habitué : N4 0,4 / 0,4 / 0,5 / 0,5 et N5 0,1 / 0,1 / 0,1 / 0,1, exactement
les chiffres de la carte 3.3 ; une seule mort de pigeon sur les 4 graines (N5, colonne 107).

Robot de QA : finit les cinq niveaux sans mourir (12 131 images au lieu de 12 130, 5 gouttes). Son
script est identique à celui de 755df73 jusqu'à l'image 11 400 : au niveau 4 le pigeon plonge
derrière lui (annonce à l'image 9685) sans rien changer à son parcours ; au niveau 5 (annonce à
l'image 11 400) son chemin s'en écarte de 0,2 px.

Captures (Chromium 1208 piloté par CDP, fenêtre 1280 x 720 donc image agrandie x4, build de QA
servi sur 127.0.0.1, port libre, onglet au premier plan, canevas focalisé, serveurs arrêtés
ensuite ; script du robot rejoué par `docs/capture-visibilite.mjs`). « Avant » : build de QA et
script d'une extraction propre de 755df73, même image du parcours (même position de Pépin).
Toutes regardées ; 0 pixel hors palette, 0 flou (`docs/mesure-capture.py`, K = 4) sur les 12
captures pleines.
- `44-niveau5-oiseau-et-pigeon-apres` (image 11 341) : capture côte à côte demandée. À gauche
  sur le toit, l'oiseau de décor (petit, blanc, bec orange) ; à droite, le pigeon perché sur son
  petit nuage (plus gros, ardoise, cou cyan) ; Pépin et la lanterne entre eux. On ne les confond
  pas : couleur, taille, place (l'oiseau au sol sur le chemin, le pigeon en hauteur).
  `44-zoom-niveau5-oiseau-et-pigeon-apres` : les deux découpés à la même échelle x4 et posés
  côte à côte (aucun rééchantillonnage). `-avant` : même endroit, ni nuage ni pigeon.
- `44-niveau5-pigeon-annonce-apres` (11 406) : ailes dressées, il tremble ; Pépin passe dessous ;
  un second oiseau de décor est visible à droite, à 8 tuiles. `-plongee-apres` (11 433) : en
  diagonale, tête en avant, derrière Pépin qui court. `-remontee-apres` (11 471) : ailes levées,
  tourné vers son perchoir.
- `44-niveau4-pigeon-perche-apres` (9661) / `-avant` : le petit nuage et le pigeon sur le ciel
  corail, au-dessus du dernier toit ; Pépin arrive par le haut (repère en haut de l'écran).
  `44-niveau4-pigeon-annonce-apres` (9693), `-plongee-apres` (9716), `-fond-apres` (9734, au
  ras du toit, Pépin déjà passé), `-remontee-apres` (9761).

Limites : le contraste n'est testé que contre les bandes de ciel, pas contre les silhouettes du
fond : au niveau 4, en plongée ou en remontée, le pigeon passe devant les tours violettes (ardoise
sur violet 1,5:1) ; son contour, l'aile claire et le cou cyan le gardent lisible (vu sur
`44-niveau4-pigeon-remontee-apres`). La plongée est muette (aucun son ajouté). Pas testé : vrai
téléphone, Safari/Firefox.

Vérification sur une extraction propre de 15e36c2 (`git archive`, nouveau dossier daté) :
- `./docker-cargo.sh test --workspace --no-default-features` : 175 + 207 tests, tous verts (14 + 4
  de plus qu'à la carte 4.3) ; `./docker-cargo.sh lint` : `LINT-OK` ; `./build.sh` : `dist/`
  produit, 0 occurrence des noms interdits, seule URL `http://www.w3.org/2000/svg`.
- Régénérés dans l'extraction, identiques à l'octet : `docs/planche-sprites.png`,
  `docs/mesure-difficulte.txt`, `target/qa/script.txt` et `trace.csv` (12 131 images, 0 mort).
- Les 9 captures « après » refaites sur le build de QA de l'extraction (mêmes images du rejeu) :
  identiques à l'octet à celles du dépôt.

## Nombre et ordre de niveaux variables, bonus et épilogue (carte 5.1)

Socle de l'étape 5 (le jeu passera de 5 à 11 niveaux). Aucun niveau, ennemi ni mécanique
ne change : avec les 5 niveaux actuels, le jeu se comporte exactement comme avant.

Ce qui a été fait :
- Table ORDONNÉE des niveaux dans `core/src/levels.rs` : `NIVEAUX` est une tranche
  (`&[InfosNiveau]`), `NB_NIVEAUX` sa longueur ; `zones.rs` les ré-exporte. Chaque niveau a
  un genre (`GenreNiveau` : `Normal`, `Bonus`, `Epilogue`, champ `genre` de `InfosNiveau`) ;
  les 5 niveaux actuels sont `Normal`.
- Fonctions pures, testées sur de FAUSSES listes de niveaux (`levels::tests`) :
  `apres(niveaux, indice, gouttes)` (niveau suivant ou écran de fin), `bonus_debloque`,
  `gouttes_pour_le_bonus` (3 par niveau normal), `verifier_ordre`. Règles : le bonus est joué
  si TOUTES les gouttes des niveaux normaux ont été ramassées pendant la partie, sinon on
  passe directement au niveau suivant (l'épilogue) ; l'écran de fin vient après l'épilogue,
  ou après le dernier niveau s'il n'y a pas d'épilogue. `verifier_ordre` impose : numéros =
  places (1, 2, 3...), au moins un niveau normal, tous les normaux d'abord, au plus un bonus,
  l'épilogue en dernier. Cet ordre garantit qu'au moment de choisir le bonus, les gouttes
  de la partie sont exactement celles des niveaux normaux (aucun compteur à part).
  Tests : 11 niveaux (9 normaux, bonus, épilogue) avec 27 gouttes, 26 et 0 ; sans bonus ;
  sans épilogue ; indices hors table ; 6 tables mal rangées refusées ; et la vraie table :
  1, 2, 3, 4, 5 puis la fin, quelles que soient les gouttes.
- Le jeu (`app.rs`) enchaîne par `levels::apres` ; `Monde::indice` remplace les calculs
  `numero - 1`. Test `la_suite_de_chaque_niveau_suit_la_table` : pour chaque vrai niveau,
  avec 0 goutte et avec toutes, l'écran suivant est celui que donne `apres`.
- Carte de présentation : « NIVEAU n/N » où N = nombre de niveaux NORMAUX (« NIVEAU 2/5 »
  aujourd'hui, « NIVEAU 9/9 » demain) ; « NIVEAU BONUS » et « ÉPILOGUE » pour les deux
  autres genres (glyphes vérifiés dans la police).
- Table d'ambiance par thème (`decor::ambiance`) : la teinte des tuiles (éclaircissement de
  l'ardoise, briques orangées) et le dessin des dalles friables (nuage ou ardoise), écrits
  jusqu'ici dans deux `match` séparés (`sprites::remplacer`, `scene.rs`), la rejoignent ;
  valeurs inchangées (test `teinte_des_tuiles_et_friables_par_theme`). `Theme::TOUS` et
  `Theme::indice` remplacent les deux listes de 5 thèmes. Ambiance inédite pour un nouveau
  niveau = un `Theme` de plus (avec `Theme::TOUS`, test `tous_les_themes_dans_l_ordre`) et
  une entrée dans `ambiance` : le compilateur refuse d'oublier l'une ou l'autre.
- Build de QA seulement (`debug_niveaux`) : touches 1 à 9 (une touche au-delà du nombre de
  niveaux ne fait rien), niveau SUIVANT (Page suivante ou N) et PRÉCÉDENT (Page précédente
  ou B) dans la table, bonus compris et sans condition : tous les niveaux restent
  atteignables au-delà de 9. Fonction pure `entrees::cible_raccourci` testée avec 11
  niveaux (aller-retour complet) et avec 5 (touches 6 à 9 sans effet, « suivant » au dernier
  et « précédent » au premier sans effet). N et B sont à la même place en QWERTY et en
  AZERTY et ne servent à rien d'autre (test `les_raccourcis_de_niveau_ne_touchent_a_aucune_commande`).
  Production : test inchangé, aucune touche ne change de niveau.
- Robot de QA : `empreinte` (FNV-1a sur 64 bits, image par image, de l'écran, du niveau, du
  score, des gouttes, des morts et du chrono) affichée par `ecrire_script_navigateur` ; le
  test du robot (`le_robot_finit_tous_les_niveaux_sans_mourir`) vérifie aussi que les
  niveaux joués suivent `apres` avec ses gouttes.

Hypothèses « 5 niveaux » retirées (`grep -rn` sur `cinq`, `/5`, `1 à 5`, `0 à 4`, `; 5]`,
`NIVEAUX[`, `[3..5]`) :
- `core/src/zones.rs` : `NB_NIVEAUX = 5` et `[InfosNiveau; 5]` (devenus la longueur de la
  table) ; test `cinq_niveaux_dans_l_ordre` (devenu `les_niveaux_sont_numerotes_dans_l_ordre`,
  `niveau(NB_NIVEAUX)` hors table) ; 5 tests de faisabilité écrits à la main (désormais une
  macro, et `chaque_niveau_a_son_test_de_faisabilite` échoue si un niveau n'a pas le sien) ;
  `NIVEAUX[0]`, `NIVEAUX[3]` et le tableau de 5 décors attendus rangés par place (désormais
  par `INFOS` de niveau : insérer un niveau ne décale plus rien).
- `game/src/app.rs` : `numero >= NB_NIVEAUX` (fin) et `numero - 1` (2 endroits).
- `game/src/contenu.rs` : « NIVEAU {numero}/{NB_NIVEAUX} » (comptait bonus et épilogue).
- `game/src/ecrans/carte.rs` : signature `(indice, numero, nom)`, commentaire « 0 à 4 ».
- `game/src/entrees.rs` et `main.rs` : touches 1 à 5 seulement (`[KeyCode; 5]`).
- `game/src/sprites/mod.rs` : `THEMES: [Theme; 5]`, `indice_theme` ; test du pigeon sur
  `NIVEAUX[3..5]` (désormais `vents::INFOS`, `nuages::INFOS`).
- `game/src/decor.rs` : `THEMES: [Theme; 5]` des tests.
- `game/src/scene.rs` : test des ombres par `numero` 1 à 3 et 5 (désormais par thème),
  `NIVEAUX[3]`.
- `game/src/bot_qa.rs` : « cinq niveaux », « 1 à 5 » (commentaires, nom du test).
- `game/src/difficulte.rs` : `fausse_mesure([f32; 5], ...)` (désormais `[f32; NB_NIVEAUX]`).
- `game/src/monde.rs` : commentaire « 0 à 4 ».
- `README.md`, `docs/build.md` : touches 1 à 5, marche à suivre pour ajouter un niveau.

Laissés tels quels, volontairement :
- HUD et écran de fin : « x/N » avec N = `GOUTTES_TOTAL` = 3 × `NB_NIVEAUX` (déjà calculé
  d'après la table ; avec 11 niveaux : 33, bonus et épilogue compris). Le message « collection
  complète » demande donc aussi les gouttes du bonus et de l'épilogue.
- Tableaux typés `[_; NB_NIVEAUX]` (`contenu::INTROS`, `difficulte::BANDES_HABITUE`) : un
  niveau ajouté sans son texte ou sa bande de difficulté ne compile pas. Les juges de la
  courbe de difficulté (`ecarts_de_la_courbe` : « niveau 4 sous le niveau 3 »...) parlent des
  niveaux actuels par leur place : à reprendre par la carte 5.18 (courbe sur 11 niveaux).
- Tests qui nomment un niveau par son numéro (lanternes sur le trajet d'un danger, pigeons
  aux niveaux 4 et 5) : insérer un niveau avant eux les fait ÉCHOUER (pas de faux vert) ; la
  carte qui insère le niveau les renumérote.
- `README.md` (« en cinq niveaux », liste des niveaux) : décrit le jeu d'aujourd'hui.

Même partie qu'avant (preuve demandée) :
- Robot de QA, partie complète à travers l'`Application`, avant (8076d65 : seule
  l'empreinte ajoutée) et après (8c8eac0) : 12 131 images, 0 mort, 5 gouttes, score 800,
  chrono 10 727 pas (2:58,7), empreinte `e03e19a27c17c283` des deux côtés ;
  `target/qa/script.txt` et `trace.csv` identiques à l'octet.
- Mesure de difficulté : aucune entrée ne change (ni niveau, ni ennemi, ni mécanique) ; la
  mesure et la planche de sprites (aucun sprite ne change) sont régénérées sur l'extraction
  propre, résultat dans la vérification ci-dessous.

Captures (Chromium 1208 piloté par CDP, fenêtre 1280 x 720 donc image agrandie x4, build de
QA servi sur 127.0.0.1, onglet au premier plan, canevas focalisé, serveurs arrêtés ensuite ;
`docs/capture-visibilite.mjs`, qui gagne la demande `touche:` pour appuyer sur une touche).
Toutes regardées ; 0 pixel hors palette, 0 flou (`docs/mesure-capture.py`, K = 4) sur les 13.
- Rejeu du robot, AVANT (build de QA d'une extraction propre de 73f844b) et APRÈS, mêmes
  images : `51-carte-niveau1` (image 143), `51-carte-niveau2` (2408), `51-jeu-niveau4-hud`
  (8500, HUD 3/15 et 2:06), `51-carte-niveau5` (10 063), `51-fin` (11 950 : 2:58,7, 5/15,
  0 mort, 800 points). Les 5 paires sont identiques à l'octet ; seules les « après » sont
  versionnées.
- Raccourcis du build de QA (après seulement ; le robot lâche son script à l'image 1000, en
  plein niveau 1) : `51-qa-pagedown-vers-niveau2` (Page suivante : NIVEAU 2/5 GOUTTIÈRES ET
  CHEMINÉES), puis Entrée ; `51-qa-n-vers-niveau3` (N : NIVEAU 3/5 LE GRAND ORAGE), Entrée ;
  `51-qa-b-retour-niveau2` (B : NIVEAU 2/5), Entrée ; `51-qa-pageup-vers-niveau1` (Page
  précédente : NIVEAU 1/5 LES TOITS), Entrée ; `51-qa-pageup-au-premier-sans-effet` (Page
  précédente au niveau 1 : on reste en jeu au niveau 1, chrono 0:02) ;
  `51-qa-touche5-vers-niveau5` (5 : NIVEAU 5/5 AU-DESSUS DES NUAGES), Entrée ;
  `51-qa-pagedown-au-dernier-sans-effet` (Page suivante au dernier niveau : on reste en jeu)
  et `51-qa-touche7-sans-effet` (7 avec 5 niveaux : rien, le chrono passe de 0:02 à 0:03).

Limites : aucune fausse liste ne passe par l'`Application` ni par le robot (la table du jeu
est une constante) : bonus et épilogue ne sont vérifiés que par les fonctions pures et par
`la_suite_de_chaque_niveau_suit_la_table` sur la vraie table ; le premier vrai bonus (carte
de l'arc-en-ciel) devra le vérifier en jeu. Pas de capture « NIVEAU BONUS » ni « ÉPILOGUE »
(aucun niveau de ce genre aujourd'hui ; l'étiquette est testée, glyphes compris). Pas testé :
vrai téléphone, Safari/Firefox, raccourcis sur un vrai clavier AZERTY (touches envoyées par
CDP avec leur code physique).

Vérification sur une extraction propre de 024c06d (`git archive`, nouveau dossier daté) :
- `./docker-cargo.sh test --workspace --no-default-features` : 185 + 212 tests, tous verts (10 + 5
  de plus qu'à la carte 4.4) ; `./docker-cargo.sh lint` : `LINT-OK` ; `./build.sh` : `dist/`
  produit, 0 occurrence des noms interdits, seule URL `http://www.w3.org/2000/svg`.
- Régénérés dans l'extraction, identiques à l'octet : `docs/planche-sprites.png`,
  `docs/mesure-difficulte.txt` (tableau avant/après inchangé : aucun niveau, ennemi ni
  mécanique ne bouge), `target/qa/script.txt` et `trace.csv` (12 131 images, 0 mort, score
  800, chrono 10 727 pas, empreinte `e03e19a27c17c283`, la même qu'avant la carte).
- Les 13 captures 51- refaites sur le build de QA de l'extraction : identiques à l'octet à
  celles du dépôt.
- Build de PRODUCTION de l'extraction (`dist/`), mêmes touches à l'image 1000 en plein
  niveau 1 : `51-prod-pagedown-sans-effet` (Page suivante, 100 images plus tard : toujours le
  niveau 1, chrono 0:14 qui continue, 1/15) et `51-prod-touche3-sans-effet` (N puis 3 : toujours
  le niveau 1, chrono 0:16) ; un changement de niveau aurait remis le chrono à zéro. 0 pixel
 hors palette, 0 flou.

 ## Caméra verticale des niveaux hauts, niveau d'essai (carte 5.2)

 Socle de l'étape 5 : un niveau peut dépasser un écran (11 rangées / 176 px). La caméra suit
 Pépin en hauteur (niveau d'essai de 24 x 45, soit quatre écrans, 720 px). Aucun niveau,
 ennemi ni mécanique ne change : les 5 niveaux actuels (un écran chacun) se comportent
 exactement comme avant, prouvé au bit près.

 Ce qui a été fait :
 - Caméra (`game/src/camera.rs`) : au sol, Pépin est recadré à 0,58 de l'écran ; en l'air,
 zone morte verticale (0,20-0,70 de l'écran, 0,50 parapluie ouvert), lissage 0,12, Pépin
 toujours à 8 px des bords, bornes du niveau en haut et en bas, arrondi vertical par
 l'écart Pépin-caméra. Le bas de la zone remonte de 0,70 à 0,40 sur les 96 px sous le point
 de décollage (regard vers le bas en descente, sans à-coup) : dans le puits du niveau
 d'essai, Pépin restait collé en bas de l'écran (y = 158). Jamais plus de 9,6 px de caméra
 par pas ; plus de 100 px de vue sous Pépin en chute. Niveaux d'un écran : ancien calcul
 inchangé, test bit à bit contre une copie VERBATIM de l'ancienne caméra sur les 5 niveaux
 (5000 pas chacun, 68 morts/reparutions) ; 7 tests sur un mini-niveau vertical (saut sans
 bouger l'écran, recadrage doux, longue chute visible, plané, au-dessus du niveau, arrondi,
 pas de tremblement).
 - Parallaxe verticale du fond (`game/src/decor.rs`, `scene.rs`) : le fond compte la caméra
 depuis le bas du niveau (`decor::camera_fond`) ; les silhouettes descendent quand la caméra
 monte (`enfoncement` : 27, 38 et 54 px en haut d'un niveau de quatre écrans, proche > îlots
 > lointaine, toutes moindres que les tuiles), facteurs nommés `VERT_*`. Niveau d'un écran :
 même fond qu'avant, secousse comprise (test) ; contraste couches/ciel vérifié jusqu'à
 64 rangées.
 - Cœur (`core/src`) : éclair d'une grille haute, la colonne part du premier solide au-dessus
 du « e » (ou du haut de la carte s'il n'y en a pas), niveaux d'un écran inchangés (« e » sur
 la rangée du haut), test sur une grille de 30 rangées ; durée d'un plan selon la hauteur du
 niveau (`faisabilite::pas_max_plan` : plané sur toute la hauteur + 2 s, jamais moins des
 10 s d'avant ; mini-niveau de 45 rangées où seul un plané de ~18 s mène à l'arrivée, trouvé
 avec parapluie, bloqué sans, échoue avec l'ancienne limite) ; niveau d'essai vertical
 24 x 45 HORS de la table `NIVEAUX` (échelle de corniches en zigzag, champignon qui lance
 au-dessus du niveau, puits de droite, 3 gouttes, 3 lanternes, thème Altitude) ; faisable
 (robot précis et maladroit, 3 gouttes, 3 lanternes, reprise depuis chaque lanterne),
 lanternes posées, aucune poche close ; un niveau du jeu peut avoir plus de 11 rangées.
 - Jeu (`game/src`) : le niveau d'essai n'est chargé que dans le build de QA
 (`debug_niveaux`), touche 0 (`entrees::RaccourciNiveau::Essai`), indice juste après la
 table (`monde::INDICE_ESSAI`, `monde::infos_niveau`), carte « NIVEAU D'ESSAI » et son intro
 (`contenu::INTRO_ESSAI`, « L'altitude et le puits vous attendent »). En PRODUCTION l'indice
 reste hors table et le niveau n'est référencé nulle part (test). Tests : raccourci, cible,
 carte qui tient dans son cadre, glyphes, chargement par l'`Application` avec la caméra en
 bas du niveau. README, docs/build.md, build.sh, Cargo.toml.
 - Robot de QA (`game/src/bot_qa.rs`) : `pas_max_plan` du cœur ; le robot FINIT le niveau
 d'essai vertical à travers l'`Application` (touche 0), sans mourir : la caméra va de 540 à
 0 et revient, Pépin est toujours entier à l'écran sauf quand le champignon le lance au-
 dessus du niveau, et la flèche de repère (`scene::repere_haut`) apparaît exactement alors.
 Test ignoré `ecrire_script_essai_vertical` : `target/qa/script-52.txt`, `touches-52.txt` et
 `trace-52.csv` pour le navigateur.
 - Navigateur (`docs/pilote-qa.js`, `docs/capture-visibilite.mjs`) : variable `TOUCHES`
 (« IMAGE:CODE » ou fichier) pour appuyer une touche pendant le rejeu sans lâcher le script
 (la touche 0 du niveau d'essai).

 Même partie qu'avant (preuve demandée) :
 - Robot de QA, partie complète à travers l'`Application`, AVANT (extraction propre de
 abd1483, parent de la carte) et APRÈS (extraction propre de 36a3e5f) : `target/qa/script.txt`
 et `trace.csv` IDENTIQUES À L'OKTET (12 131 images, 0 mort, 5 gouttes, score 800, chrono
 10 727 pas, les 5 niveaux d'un écran n'ayant ni caméra ni durée de plan modifiées).
 - Mesure de difficulté et planche de sprites : aucune entrée ne change (ni niveau, ni ennemi,
 ni mécanique, ni sprite) ; régénérés sur l'extraction propre, IDENTIQUES À L'OKTET.
 - Rejeu du robot, AVANT (build de QA de l'extraction abd1483) et APRÈS : les 11 captures des
 5 niveaux sont identiques à l'octet ; seules les « après » sont versionnées.

 Captures (Chromium 1208 piloté par CDP, fenêtre 1280 x 720 donc image agrandie x4, build de
 QA servi sur 127.0.0.1, onglet au premier plan, canevas focalisé, serveurs arrêtés ensuite ;
 `docs/capture-visibilite.mjs` avec sa variable `TOUCHES` ; `docs/mesure-capture.py`, K = 3).
 Toutes regardées ; 0 pixel hors palette, 0 pixel semi-transparent sur les 19.
 - Rejeu du robot sur les 5 niveaux, AVANT et APRÈS, mêmes images : `52-carte-niveau1` (143),
 `52-niveau1` (1500), `52-carte-niveau2` (2408), `52-niveau2` (4000), `52-niveau3-a` (6000),
 `52-niveau3-b` (7000), `52-niveau4-a` (8500), `52-niveau4-b` (9500), `52-carte-niveau5`
 (10 063), `52-niveau5` (11 000), `52-fin` (11 950 : 2:58,7, 5/15, 0 mort, 800 points).
 Les 11 paires avant/après sont identiques à l'octet ; seules les « après » sont versionnées.
 - Niveau d'essai vertical (après seulement ; le robot joue `script-52.txt` et la touche 0 est
 appuyée à l'image 229 du pilote, `TOUCHES=229:Digit0`) : `52-essai-carte` (300 : carte
 « NIVEAU D'ESSAI », intro, NIVEAU 1/15 0/15 gouttes 15 s), `52-essai-depart` (470 : bas du
 niveau, caméra en bas, silhouettes hautes), `52-essai-montee` (806 : montée sur les
 corniches, la caméra suit), `52-essai-haut` (1156 : sommet, champignon, goutte, puits à
 droite), `52-essai-repere` (1181 : Pépin lancé au-dessus du niveau, flèche de repère en haut
 de l'écran, caméra en haut), `52-essai-chute` (1256 : chute dans le puits, la caméra suit,
 plus de 100 px de vue sous Pépin), `52-essai-fin-de-chute` (1306 : fin de chute, sol,
 drapeau), `52-essai-arrivee` (1381 : arrivée).

 Limites :
 - Le niveau d'essai est dans le build de QA seulement (`debug_niveaux`) ; en production
 l'indice est hors table et le niveau n'est référencé nulle part (test). Aucune capture
 « NIVEAU D'ESSAI » en build de production (la touche 0 n'y fait rien, comme les touches de
 raccourci de la carte 5.1).
 - Dans le navigateur, le jeu a un PAS de retard sur la numérotation du pilote (déjà noté à la
 carte 3.1b) : sans effet sur des touches tenues plusieurs images, mais une touche d'UNE
 image doit tomber une image plus tard que dans la trace, d'où `touches-52.txt` = 229
 (pilote) et non 228 (trace) ; à l'image 228 la touche arrive sur la dernière image de la
 carte du niveau 1 et le jeu l'ignore.
 - Captures du niveau d'essai : les frames en MOUVEMENT (montée, haut, chute, fin de chute,
 arrivée) peuvent décaler d'UNE image d'une exécution à l'autre (horloge temps réel du
 navigateur, événement de touche `TOUCHES` à la frontière d'image) ; les moments statiques
 (carte, départ, repère) et tout le rejeu sans `TOUCHES` sont déterministes, identiques à
 l'octet. La preuve FORMALISÉE du niveau d'essai est le test Rust `jouer_essai_vertical` à
 travers l'`Application` (caméra 540 -> 0 -> 540, Pépin toujours entier à l'écran sauf lancé
 au-dessus, flèche de repère exactement alors) ; les captures sont la preuve VISUELLE.
 - Pas testé : vrai téléphone, Safari/Firefox, ressenti d'un vrai joueur sur la durée (la
 caméra verticale n'a de sens que sur un niveau > 4 écrans, l'essai en fait exactement
 quatre), niveau réel de plus de quatre écrans.

## Vent latéral, niveau d'essai « vent » (carte 5.3)

 Nouvelle mécanique de l'étape 5 : un vent latéral qui pousse Pépin horizontalement
 (`<` = vers la gauche, `>` = vers la droite dans les ASCII), plus fort quand il plane
 (parapluie ouvert) que quand il est au sol ou en chute avec le parapluie fermé. Le socle
 de l'étape 5 reste intact : aucun niveau, ennemi ni mécanique existante ne change, prouvé
 au bit près (le rejeu des 5 niveaux est inchangé, voir Vérification).

 Ce qui a été fait :
 - Cœur (`core/src/physics.rs`) : `VentLateral { zone: Aabb, sens: i8 }` (sens -1 gauche,
 +1 droite) ; `VENT_LAT = 60` px/s (parapluie fermé, 1 px par pas) et
 `VENT_LAT_PLANE = 2 * VENT_LAT = 120` px/s (en planant) ; champ `vents` ajouté à
 `Environnement` (vide par défaut dans `simple()`).
 - Le vent pousse en DÉPLACEMENT, pas dans `vitesse.x` (`core/src/player.rs`,
 `actualiser`, bloc « 6bis ») : la vitesse du joueur reste propre (l'animation de course et
 la friction ne la voient pas), le vent s'ajoute au déplacement du pas. On somme les sens
 des zones qui touchent la boîte : deux vents opposés qui se chevauchent s'annulent,
 plusieurs vents de même sens ne se cumulent pas au-delà d'un vent. 6 tests physiques
 (poussée au sol vers la droite et vers la gauche, double en planant, plus faible en
 chute avec le parapluie fermé, annulation de deux vents opposés, résistance au vent de
 face au sol).
 - Analyse des ASCII (`core/src/level/ascii.rs`) : les cases `<` (sens -1) et `>` (sens
 +1) forment des rectangles de même sens, les tuiles voisines de même sens sont fusionnées
 en un seul AABB (`fusionner_zones`, le même algorithme que les courants ascendants `v`),
 les zones de sens opposés restent distinctes. `Niveau.vents` (`core/src/level.rs`) et
 légende de la doc.
 - Robot de faisabilité (`core/src/faisabilite.rs`) : le robot compte les vents
 (`Options.vents`, activé par défaut) dans sa simulation du jeu réel ; 3 tests sur un
 mini-niveau de 54 x 11 : un vide de 14 tuiles que seul un vent de dos (plané) laisse
 passer, un vent de face qui bloque la traversée d'un vide que le saut seul passe (le
 vent rend le niveau IMPOSSIBLE sans lui), et au sol un vent de face qui ralentit sans
 arrêter (le robot finit toujours).
 - Niveau d'essai « ESSAI VENT » (`core/src/levels/essai_vent.rs`) : 36 x 11 (un écran),
 thème Soleil, `GenreNiveau::Normal` (comme l'essai vertical ; l'essai est hors de la
 table `NIVEAUX`, `numero: 0`), builds de QA et tests uniquement. Un vide de 12 tuiles
 (192 px) sous une zone de vent `>` (3 rangées) : infranchissable sans vent (test
 `niveau_d_essai_vent_exige_les_vents`), franchissable avec (test
 `niveau_d_essai_vent_faisable`) ; 3 gouttes (posées sur le sol, ramassées en marchant)
 et 3 lanternes ; faisable robot précis ET maladroit, 3 gouttes, 3 lanternes,
 reprise depuis chaque lanterne, lanternes posées, aucune poche close (suite de tests de
 `core/src/zones.rs` sur `niveaux_et_essai`, comme l'essai vertical).
 - Jeu (`game/src`) : la touche F1 charge le niveau (build de QA seulement :
 `entrees::RaccourciNiveau::Essai2`, et en build de production le raccourci est inactif,
 `niveau_demande` reçoit `actif = false` et renvoie toujours `None`, comme les touches de
 la carte 5.1, et l'indice est hors table : `INDICE_ESSAI_VENT` n'est compilé que dans
 les tests et le build de QA). L'indice est juste après les essais
 (`monde::INDICE_ESSAI_VENT = NB_NIVEAUX + 1`), carte « NIVEAU D'ESSAI » et son intro
 (INTRO_ESSAI_VENT : le vent porte, planer le double). Effet visuel : les zones de vent
 soufflent des traits horizontaux (`game/src/particules.rs::souffle_lateral`, largeur 3,
 hauteur 1) qui traversent la zone dans le sens du vent (bornés au rectangle de la zone,
 ils s'éteignent à l'arête opposée) et `game/src/effets.rs` les dépose d'après
 `monde.niveau.vents`, en production il n'y a aucune zone de vent, donc aucun trait.
 - Bot de QA (`game/src/bot_qa.rs`) : `jouer_essai` généralisé (les deux essais se jouent
 au même endroit, avec leur parcours et leur raccourci) ; `le_robot_finit_le_niveau_d_essai_vent`
 : le robot finit le niveau d'essai « vent » SANS mourir à travers l'`Application`,
 l'écran de fin est atteint, 3 gouttes ramassées, F1 appuyée une seule fois, la caméra
 reste au sol (niveau d'un écran, y toujours -4), Pépin traverse le vide et arrive à
 x = 533 (le drapeau est à droite de la zone de vent, 343 images de jeu) ;
 `ecrire_script_essai_vent` (test ignoré, comme les autres scripts) écrit
 `target/qa/script-53.txt` (824 images), `target/qa/touches-53.txt` (`229:F1`, image du
 PILOTE, la touche du navigateur tombe une image plus tard que dans la trace, déjà noté
 à la carte 3.1b) et `target/qa/trace-53.csv` (position de Pépin et caméra rendue, image
 par image).

 Limites :
 - Le niveau d'essai « vent » est dans le build de QA seulement (`debug_niveaux`) ; en
 production l'indice est hors table et la touche F1 n'y fait rien (comme la touche 0 de la
 carte 5.2 et les touches de raccourci de la carte 5.1). Aucune capture « ESSAI VENT » en
 build de production.
 - Le niveau d'essai est d'UN écran (11 rangées) : il valide la mécanique du vent, pas une
 combinaison avec la caméra verticale de la carte 5.2 (les deux essais restent séparés).
 - Le vent ne modifie que le DÉPLACEMENT de Pépin : il ne peut pas le faire coller à un
 mur plus vite qu'au déplacement suivant, et une zone de vent ne l'empêche jamais
 d'atterrir (les collisions de `deplacer` restent maîtresses, l'axe x d'abord).

 Vérification :
 - `./docker-cargo.sh test --workspace --no-default-features` : 200 + 227 tests verts ;
 `./docker-cargo.sh lint` : `LINT-OK` ; `./build.sh --qa` : `dist-qa/` produit.
 - Mêmes jeux qu'avant : sur une extraction propre de cb4a8cb (`git archive`, nouveau
 dossier), régénérés et identiques à l'octet à l'arbre courant : `target/qa/script.txt`
 et `trace.csv` (robot principal sur les 5 niveaux, aucune zone de vent n'y existe) et
 `target/qa/script-52.txt` et `trace-52.csv` (essai vertical, le parcours n'utilise pas
 de vent) ; `docs/mesure-difficulte.txt` et `docs/planche-sprites.png` régénérés dans
 l'arbre courant, identiques à l'octet à l'extraction. Les nouveaux artefacts 53
 (script-53.txt, touches-53.txt, trace-53.csv) sont générés par le bot.
 - Les 7 captures 53- sont dans `docs/qa/` (build de QA servi en local, pilote de la
 carte 3.1b avec `TOUCHES` = `target/qa/touches-53.txt`, F1 au pilote image 229) :
 `53-carte-niveau1` (100 : carte du niveau 1), `53-carte-essai` (350 : carte « ESSAI
 VENT » avec son intro), `53-essai-depart` (460 : départ au sol), `53-essai-vent` (636 :
 plané dans le vide, parapluie ouvert, les traits de vent latéral se voient),
 `53-essai-chute` (680 : fin de la planée, retombe à droite du vide),
 `53-essai-approche` (780 : approche du drapeau) et `53-fin` (823 : écran de fin,
 3 gouttes sur 3). 0 pixel hors palette, 0 semi-transparent (mesure-capture.py,
 K = 3). Comme la carte 5.2, les moments statiques (cartes, fin) sont déterministes et
 les frames en mouvement (départ, vent, chute, approche) peuvent décaler d'une image
 d'une exécution à l'autre (horloge temps réel du navigateur) ; la preuve FORMALISÉE du
 niveau est le test Rust `jouer_essai_vent` à travers l'`Application` (3 gouttes,
 0 mort, F1 une fois, caméra au sol, arrivée à x = 533) ; les captures sont la preuve
 VISUELLE.

 Vérification sur une extraction propre de 36a3e5f (`git archive`, nouveau dossier daté) :
 - `./docker-cargo.sh test --workspace --no-default-features` : 189 + 225 tests, tous verts
 (6 ignorés : les scripts) ; `./docker-cargo.sh lint` : `LINT-OK` ; `./build.sh` : `dist/`
 produit, 0 occurrence des noms interdits, seule URL `http://www.w3.org/2000/svg`.
 - Régénérés dans l'extraction, identiques à l'octet : `docs/planche-sprites.png`,
 `docs/mesure-difficulte.txt` (tableau avant/après inchangé : aucun niveau, ennemi ni
 mécanique ne bouge), `target/qa/script.txt` et `trace.csv` (robot principal, mêmes 12 131
 images que l'avant de la carte), `target/qa/script-52.txt` et `trace-52.csv` (1 429 images,
 1 goutte, caméra 540 -> 0 -> 540) ; `touches-52.txt` = `229:Digit0` (image du pilote).
 - Les 19 captures 52- refaites sur le build de QA de l'extraction : les 11 du rejeu et les 3
 moments statiques du niveau d'essai (carte, départ, repère) sont identiques à l'octet à
 celles du dépôt ; les 5 frames en mouvement du niveau d'essai décalent d'une image (wasm
 identique à l'octet, jitter de l'horloge navigateur, voir Limites). 0 pixel hors palette,
 0 semi-transparent.

## Grêle, niveau d'essai « grêle » (carte 5.4)

Lettre `g` : une colonne de grêlons qui tombe EN CONTINU (ni pause ni annonce, à la
différence de l'éclair), de la case `g` elle-même jusqu'au premier solide en dessous (ou
jusqu'au bas de la carte au-dessus d'un vide). Elle tue Pépin parapluie FERMÉ ; il la
traverse sans danger en planant dessous (parapluie ouvert). Aucune touche nouvelle. Aucun
niveau du jeu n'est modifié (les niveaux viennent avec les cartes 5.12 à 5.17).

Ce qui a été fait :
- Cœur (`core/src/entities/grele.rs`) : `Grele { zone }` (colonne fixe de 12 px centrée dans
 la case, comme l'éclair) ; `Grele::blesse(&Joueur)` = « touche la colonne ET ne plane
 pas ». Parapluie fermé = tout ce qui n'est pas planer : au sol, pendant la MONTÉE d'un saut
 (on ne plane qu'en descendant, SAUT maintenu), en chute sans SAUT. `Entite::Grele`,
 `CauseMort::Grele` (« grêle » dans la mesure de difficulté) ; la protection de
 réapparition s'applique comme aux autres dangers. Tests : mort parapluie fermé, survie en
 plané, limites de la colonne au pixel près (collé à gauche, à droite, dessous : rien ;
 1 px de chevauchement : mort), protection, et un MINI-NIVEAU joué avec la vraie physique
 (marcher dedans tue au sol ; sauter juste devant tue pendant la montée, SAUT pourtant
 enfoncé ; sauter tôt et planer dessous mène au drapeau, en planant DANS la colonne).
- Analyse ASCII (`core/src/level/ascii.rs`, légende de `core/src/level.rs`) : test
 `grele_tombe_de_sa_case_au_premier_solide` (sol, trou dans le sol jusqu'au bas de la
 carte, corniche au-dessus sans effet, corniche dessous qui arrête la colonne).
- Robot de faisabilité (`core/src/faisabilite.rs`) : la grêle est GARDÉE dans sa simulation
 (fixe et continue : attendre ne sert à rien), un plan qui la touche parapluie fermé est
 écarté. 3 mini-niveaux 54 x 11 : un rideau du ciel au toit, infranchissable sans parapluie
 (témoin sans grêle : traversé à pied), traversé en planant par le robot précis ET le
 maladroit ; une corniche qui arrête la colonne (on passe dessous à pied, sans parapluie) ;
 un vide de 4 tuiles qu'un saut simple franchit sans grêle mais plus avec (il faut sauter
 plus tôt et planer).
- Niveau d'essai « ESSAI GRÊLE » (`core/src/levels/essai_grele.rs`, 36 x 11, thème Altitude :
 ciel corail), hors de la table `NIVEAUX`, touche F2 du build de QA seulement (en production
 le raccourci est inactif et le niveau n'est pas dans le binaire : « Essai grêle » 0 fois
 dans `dist/game.wasm`, 1 fois dans `dist-qa/`). Trois rideaux : colonne 9 du ciel au toit
 (sauter tôt, planer dessous), double rideau des colonnes 21-22 au-dessus d'un vide de
 6 tuiles (la planée traverse vide et grêle), colonne 28 arrêtée par une corniche (on passe
 dessous à pied, la 3e goutte y attend). 3 gouttes, 3 lanternes. Contrôles : faisable robot
 précis et maladroit, reprise depuis chaque lanterne, contrôles de grille, positions des
 4 colonnes, bloqué sans parapluie ; nouveau contrôle `aucun_point_de_reprise_sous_la_grele`
 (départ et lanternes de TOUS les niveaux et essais : Pépin y réapparaît au sol, parapluie
 fermé, il y mourrait dès la fin de sa protection).
- Jeu : `RaccourciNiveau::Essai3` (F2), `monde::INDICE_ESSAI_GRELE`, `INTRO_ESSAI_GRELE` ; les
 tests des cartes couvrent les trois essais (et vérifient désormais aussi les glyphes de
 leurs noms) ; README : touches F1 et F2 du build de QA.
- Rendu (`game/src/grele.rs`, sprites `grelon0-3` et `grele_eclat` dans
 `game/src/sprites/objets.rs`) : grêlon = bille de glace 2 x 2 cernée de sombre (4 x 4), le
 reflet fait le tour en 4 images ; deux files par colonne, 3 px par pas, un tous les 20 px,
 calculés depuis le numéro du pas (aucune mémoire, rendu reproductible), bornés à la
 colonne ; les grêlons se brisent (éclat 7 x 3) au pied de la colonne sur un solide (jamais
 au-dessus d'un vide) et SUR la toile du parapluie quand Pépin plane dessous : la
 protection se voit. Lisibilité sur tous les ciels : sur chaque bande de ciel de chaque
 thème, la glace ou le contour contraste d'au moins 4,5:1 (test
 `la_grele_se_detache_de_tous_les_ciels`). Planche de sprites régénérée.
- Robot de QA (`game/src/bot_qa.rs`) : `le_robot_finit_le_niveau_d_essai_grele`, à travers
 l'`Application`, fin atteinte, 0 mort, 3 gouttes (dont celle sous la corniche), F2 une
 seule fois, caméra au sol, 430 images de jeu dont 50 à planer DANS la grêle, arrivée à
 x = 516,6. Rien de spécial à lui apprendre : la vraie simulation tue les plans qui
 touchent la grêle parapluie fermé, il garde ceux qui planent. Script du navigateur :
 `ecrire_script_essai_grele` (ignoré) écrit `target/qa/script-54.txt` (911 images),
 `touches-54.txt` (`229:F2`, image du pilote) et `trace-54.csv` (colonne `sous_la_grele`).

Mesure de difficulté : inchangée (aucun niveau, ennemi ni mécanique d'un niveau du jeu ne
bouge). Avant (extraction propre de 5660143) et après (extraction propre de bd53e36) :
`docs/mesure-difficulte.txt` identique à l'octet.

| Niveau                      | Débutant (avant = après) | Habitué (avant = après) | Expert (avant = après) |
| --------------------------- | ------------------------ | ----------------------- | ---------------------- |
| 1. Les toits                | 0,4                      | 0,1                     | 0,0                    |
| 2. Gouttières et cheminées  | 1,5                      | 0,5                     | 0,2                    |
| 3. Le grand orage           | 3,0                      | 0,6                     | 0,4                    |
| 4. Vents d'altitude         | 1,3                      | 0,4                     | 0,1                    |
| 5. Au-dessus des nuages     | 0,5                      | 0,1                     | 0,0                    |
| Total                       | 6,8                      | 1,8                     | 0,7                    |

Même jeu qu'avant, à l'octet (régénérés sur les deux extractions propres) :
`target/qa/script.txt` et `trace.csv` (robot principal, 12 131 images, empreinte
`e03e19a27c17c283`), `script-52.txt`/`trace-52.csv`/`touches-52.txt` (essai vertical),
`script-53.txt`/`trace-53.csv`/`touches-53.txt` (essai vent).

Captures (`docs/qa/54-*`, 960 x 540, build de QA d'une extraction propre de bc25be3 servi
en local sur 127.0.0.1, Chromium 145 piloté par CDP, onglet au premier plan et canvas
focalisé, pilote `docs/capture-visibilite.mjs` avec `SCRIPT=target/qa/script-54.txt` et
`TOUCHES=target/qa/touches-54.txt`, serveur arrêté à la fin) :
- `54-carte-niveau1` (image 100) puis `54-carte-essai` (350) : carte « ESSAI GRÊLE », texte et
 astuce dans le cadre ;
- `54-essai-depart` (470) : la colonne 9 tombe du ciel au toit, lisible sur le ciel corail
 (glace claire en haut sur le violet, contour sombre plus bas sur le corail) ;
- `54-essai-plane-grele` (577) : Pépin plane DANS la colonne 9, les grêlons se brisent sur
 la toile du parapluie et continuent sous ses pieds ;
- `54-essai-rideau-vide` (750) : planée au-dessus du vide à travers le double rideau, qui
 tombe sans fin dans le vide ; éclats sur la toile, et à droite, un éclat sur la corniche
 où s'arrête la colonne 28 ;
- `54-essai-corniche` (845) : la colonne 28 s'arrête sur la corniche, Pépin passe dessous à
 pied (3e goutte ramassée), le drapeau en vue ;
- `54-essai-ferme` (560) et `54-essai-mort` (572) : à l'image 500 le robot lâche son script
 et Pépin court tout droit, parapluie fermé : il arrive au bord de la colonne 9 (560), y
 meurt (animation de mort au pied de la grêle, 572), puis réapparaît au départ ;
- `54-fin` (970, 60 images après la fin du script) : « LE SOLEIL ! », 0:07,1, 3 gouttes
 (3/15 : celles de l'essai), 0 mort.
Chaque capture regardée ; `docs/mesure-capture.py <capture> 3` : 0 pixel hors palette,
0 bloc flou, sur les 9.

Vérification sur une extraction propre de bc25be3 (`git archive`, nouveau dossier daté) :
`./docker-cargo.sh test --workspace --no-default-features` : 214 + 237 tests, tous verts
(9 ignorés : écritures de fichiers, scripts du robot, mesure, police, et diagnostics de la
mesure) ; `./docker-cargo.sh lint` : `LINT-OK` ; `./build.sh` :
`dist/` produit, 0 occurrence des noms interdits, seule URL `http://www.w3.org/2000/svg` ;
`./build.sh --qa` : `dist-qa/` produit.

Limites (non testé ou à relativiser) :
- Le niveau d'essai n'existe que dans le build de QA ; aucun niveau du jeu n'a encore de
 grêle (« La mer de nuages » et « La descente » viendront avec leurs cartes).
- La colonne s'arrête sur les TUILES solides seulement : une plateforme mobile ou friable
 sous la grêle ne l'arrête pas (comme l'éclair). À éviter en dessinant un niveau, ou à
 traiter dans la carte qui en aurait besoin.
- La grêle ne fait aucun son (ni bruit continu, ni éclat) : non demandé, non entendu.
- Les éclats sur la toile suivent la position DESSINÉE de Pépin ; pendant le clignotement
 de la protection de réapparition, ils se posent sur un parapluie invisible (sans effet :
 Pépin réapparaît au sol, il ne plane pas encore).
- Le « Ê » du titre de la carte est le circonflexe à un pixel centré de la police existante
 (accents d'une rangée sur les capitales) : lisible de près, discret à 1x ; la police n'a
 pas été retouchée.
- Captures faites dans Chromium seulement, à 960 x 540 ; pas de vrai téléphone, pas de
 Safari ni Firefox. Les images en mouvement peuvent décaler d'une image d'un rejeu à
 l'autre (horloge du navigateur) ; la preuve formelle est le test Rust du robot.

## Auvent, niveau d'essai « auvent » (carte 5.5)

Lettre `A` : une toile de boutique inclinée (haute à gauche, rayée, sur son armature) qui
relance Pépin en DIAGONALE, vers le haut et vers l'avant (vers la droite), quand il tombe
dessus. Ce n'est pas un solide : par-dessous ou par le côté, on passe à travers. Aucune
touche nouvelle. Aucun niveau du jeu n'est modifié (le marché couvert viendra avec sa carte).

Réglage retenu (prototypes en Python avant d'écrire le Rust, puis tests) :
- vers le haut : `VITESSE_AUVENT` = 300 px/s, soit 50 px de haut et 40 pas de vol pour
  retomber à la même hauteur (un peu moins qu'un saut : l'auvent sert à avancer) ;
- vers l'avant : un ÉLAN rangé à part de `vitesse.x` (comme la poussée du vent latéral),
  `ELAN_AUVENT` = 192 px/s, qui s'éteint en ligne droite en 40 pas. Sans rien toucher,
  Pépin retombe 4 tuiles plus loin (65,5 px mesurés) ; les commandes s'AJOUTENT à l'élan :
  garder la droite porte à 119,5 px (7,5 tuiles), garder la gauche à 11,6 px seulement ;
- la vitesse horizontale propre repart de 0 à chaque rebond : le vol ne dépend que de
  l'endroit où Pépin touche la toile, pas de la façon dont il arrive. C'est ce qui rend
  les enchaînements prévisibles pour le joueur, et ce qui fait de chaque rebond un vrai
  point de décision pour les robots. L'élan tombe à 0 contre un mur et à l'atterrissage.

Ce qui a été fait :
- Cœur (`core/src/entities/auvent.rs`, `core/src/player.rs`, `core/src/physics.rs`) :
  `Auvent { boite }` (zone de contact 16 x 8, dessus 4 px sous le haut de la case :
  `HAUT_TOILE`), `Auvent::recoit` (Pépin descend et ses pieds sont à moins de
  `MARGE_AUVENT` = 7 px sous le dessus), `Auvent::lancer` (posé sur la toile puis
  `Joueur::lancer_par_auvent`), toile enfoncée 8 pas ; `Entite::Auvent`, `Contact::Auvent`
  (même son et mêmes éclaboussures que le champignon). Sans auvent, le déplacement du
  joueur est calculé exactement comme avant (l'élan n'est ajouté que s'il existe).
  Tests sur un MINI-NIVEAU de six auvents au-dessus du vide (grille vide 60 x 12) :
  trajectoire d'un rebond (diagonale haut-droite dès le premier pas, 45 à 52 px de haut,
  40 pas ± 1, 60 à 70 px d'avance), élan qui baisse de 4,8 px/s par pas et commandes qui
  allongent ou raccourcissent le vol (rien 65,5 px, droite 119,5 px, gauche 11,6 px :
  affiché par le test), rien par-dessous ni par le côté, élan coupé par un
  mur et à l'atterrissage, six auvents espacés de 3, 4, 5 et 6 tuiles enchaînés SANS
  TOUCHER LE SOL en dosant la direction (5 points de départ sur le premier), à 4 tuiles
  sans rien toucher (même point de contact à chaque rebond, à moins de 3 px), et à 3 ou
  5 tuiles un joueur qui ne touche à rien finit par tomber (il faut doser).
- Analyse ASCII (`core/src/level/ascii.rs`, légende de `core/src/level.rs`) : test
  `auvent_pose_dans_sa_case_et_traversable` (position de la zone, case vide, pas solide).
- Robot de faisabilité (`core/src/faisabilite.rs`) : il joue l'auvent avec la vraie
  physique et chaque rebond devient un noeud « lancé » (`Noeud { x, y, lance }`,
  `Fin::Lance`) d'où il essaie 18 plans en vol (gauche, droite ou rien ; gardés en montée
  et/ou en descente ; parapluie ou non). Mini-niveaux : six auvents au-dessus d'un vide
  qu'aucun saut plané ne franchit (témoin sans auvents : bloqué), écarts 3-3-3-3-3,
  4-4-4-4-4, 5-5-5-5-5 et 3-5-3-5-4 : traversés par le robot précis ET le maladroit.
- Robot de QA et mesure de difficulté (`game/src/bot_qa.rs`, `game/src/difficulte.rs`) :
  même idée sur la vraie simulation du jeu (`Fin::Lance` quand `m.contacts` contient
  `Contact::Auvent`, `est_lance`, `plans_en_vol`, clé `CaseParcours` = (x/8, y, lancé)).
- Niveau d'essai « ESSAI AUVENT » (`core/src/levels/essai_auvent.rs`, 44 x 11, thème
  Gouttières : nuit bleue comme le futur marché), hors de la table `NIVEAUX`, touche F3 du
  build de QA seulement (« Essai auvent » 0 fois dans `dist/game.wasm`, 1 fois dans
  `dist-qa/`). Six auvents au-dessus d'un vide de 23 tuiles, colonnes 9, 13, 17, 22, 25,
  29 (écarts 4, 4, 5, 3, 4 : rien à toucher pour 4, la droite pour 5, la gauche pour 3),
  une goutte en l'air entre deux rebonds, 3 lanternes. Contrôles : faisable robot précis
  et maladroit (4,3 s en course rapide), reprise depuis chaque lanterne, contrôles de
  grille, et sans auvents le vide est infranchissable. Jeu : `RaccourciNiveau::Essai4`
  (F3), `monde::INDICE_ESSAI_AUVENT`, `INTRO_ESSAI_AUVENT` ; README : touche F3.
- Rendu (`game/src/sprites/objets.rs`, `game/src/scene.rs`) : `auvent0` et `auvent1`
  (16 x 15, de profil : mât de métal sombre, toile en pente rayée corail / blanc cassé,
  lambrequin festonné au bout, jambe de force ; image 1 : la toile plie au milieu et le
  lambrequin descend, pendant les 8 pas qui suivent un rebond). Tests : toile de deux
  couleurs cernée de sombre, mât identique d'une image à l'autre, rayure claire ou contour
  à au moins 4,5:1 sur toutes les bandes de ciel de tous les thèmes. Planche régénérée.
- Robot de QA, niveau d'essai : `le_robot_finit_le_niveau_d_essai_auvent` (le plus rapide :
  326 images de jeu, 0 mort, 4 rebonds sans toucher le sol entre le premier et le dernier :
  en gardant la droite il saute deux auvents) et `le_robot_enchaine_les_six_auvents_de_l_essai`
  (parcours « rebond par rebond » : `chercher` vise chaque auvent à son tour ; les six, dans
  l'ordre, un rebond toutes les 40 images, jamais au sol entre le premier et le sixième,
  3 gouttes, 0 mort, 418 images de jeu). Script du navigateur : `ecrire_script_essai_auvent`
  (ignoré) écrit `target/qa/script-55.txt` (899 images), `touches-55.txt` (`229:F3`) et
  `trace-55.csv` (colonnes `rebond_auvent` et `au_sol`).

Mesure de difficulté : inchangée (aucun niveau, ennemi ni mécanique d'un niveau du jeu ne
bouge). Avant (extraction propre de 77d6611) et après (extraction propre de d606ab4) :
`docs/mesure-difficulte.txt` identique à l'octet.

| Niveau                      | Débutant (avant = après) | Habitué (avant = après) | Expert (avant = après) |
| --------------------------- | ------------------------ | ----------------------- | ---------------------- |
| 1. Les toits                | 0,4                      | 0,1                     | 0,0                    |
| 2. Gouttières et cheminées  | 1,5                      | 0,5                     | 0,2                    |
| 3. Le grand orage           | 3,0                      | 0,6                     | 0,4                    |
| 4. Vents d'altitude         | 1,3                      | 0,4                     | 0,1                    |
| 5. Au-dessus des nuages     | 0,5                      | 0,1                     | 0,0                    |
| Total                       | 6,8                      | 1,8                     | 0,7                    |

Même jeu qu'avant, à l'octet (régénérés sur les deux extractions propres) :
`target/qa/script.txt` et `trace.csv` (robot principal, 12 131 images, empreinte
`e03e19a27c17c283`), et les scripts, traces et touches des essais vertical (52), vent (53)
et grêle (54).

Captures (`docs/qa/55-*`, 960 x 540, build de QA d'une extraction propre de d606ab4 servi
en local sur 127.0.0.1, Chromium 145 piloté par CDP, onglet au premier plan et canvas
focalisé, pilote `docs/capture-visibilite.mjs` avec `SCRIPT=target/qa/script-55.txt` et
`TOUCHES=target/qa/touches-55.txt`, serveur arrêté à la fin) :
- `55-carte-niveau1` (image 100) puis `55-carte-essai` (350) : carte « ESSAI AUVENT »,
  texte et astuce (« Droite : plus loin. Gauche : plus près. ») dans le cadre ;
- `55-essai-depart` (470) : le toit de départ, et au bord du toit les premiers auvents en
  rang au-dessus du vide, lisibles sur la nuit bleue ;
- `55-essai-rebond` (574) : juste après le premier rebond, Pépin part en diagonale, les
  éclaboussures corail sur la toile. Vérifié au pixel sur la grille logique (`GRILLE=1`) :
  à cette image le premier auvent est dessiné enfoncé (`auvent1`), les autres tendus ; aux
  images 570 et 590, tous tendus ;
- `55-essai-diagonale` (590) : au sommet du premier vol, entre les deux premiers auvents ;
- `55-essai-goutte` (660) : la goutte attrapée en l'air après le 3e rebond (2/15) ;
- `55-essai-retenir` (726) : avant l'auvent de la colonne 25 (3 tuiles seulement après le
  précédent), Pépin regarde à GAUCHE : il retient pour retomber plus près ;
- `55-essai-dernier` (774) : juste après le 6e rebond, le toit d'arrivée en vue ;
- `55-essai-arrivee` (830) : posé sur le toit d'arrivée, la dernière goutte devant lui ;
- `55-fin` (959, 60 images après la fin du script) : « LE SOLEIL ! », 0:06,9, 3 gouttes
  (3/15 : celles de l'essai), 0 mort.
Chaque capture regardée ; `docs/mesure-capture.py <capture> 3` : 0 pixel hors palette,
0 bloc flou, sur les 10.

Vérification sur une extraction propre de c78bdd4 (dernier commit de code ; `git archive`,
nouveau dossier daté) : `./docker-cargo.sh test --workspace --no-default-features` :
227 + 242 tests, tous verts (1 + 9 ignorés : écritures de fichiers et diagnostics de la
mesure) ; `./docker-cargo.sh lint` : `LINT-OK` ; `./build.sh` : `dist/` produit, 0
occurrence des noms interdits, seule URL `http://www.w3.org/2000/svg`, « Essai auvent »
absent ; `./build.sh --qa` : `dist-qa/` produit. Les deux `game.wasm` (production et QA)
sont identiques à l'octet à ceux de d606ab4 qui ont servi aux captures (c78bdd4 ne change
que des commentaires, un affichage de test et le README).

Limites (non testé ou à relativiser) :
- Le niveau d'essai n'existe que dans le build de QA ; aucun niveau du jeu n'a encore
  d'auvent (le marché couvert viendra avec sa carte).
- Le dessus de la zone de contact est plat (4 px sous le haut de la case) alors que la
  toile dessinée est en pente (de 2 à 6 px sous le haut de la case) : au moment du rebond,
  les pieds de Pépin peuvent paraître 2 px dans la toile ou au-dessus, pendant une image.
- Un seul sens : l'auvent relance toujours vers la DROITE (demande de la carte). Un auvent
  « vers la gauche » demanderait une autre lettre.
- En gardant la droite, un rebond porte à 7,5 tuiles : un joueur pressé peut sauter
  des auvents (le robot le plus rapide le fait). La carte du marché couvert devra en tenir
  compte si elle veut imposer les six rebonds (par exemple un plafond, ou des écarts qui
  ne laissent pas de raccourci).
- L'auvent fait le même « boing » que le champignon : pas de son propre, rien entendu.
- 16 px de large (une case) : à petite taille, l'auvent peut faire penser à un fanion ;
  un auvent plus large (deux cases) n'a pas été fait.
- Captures faites dans Chromium seulement, à 960 x 540 ; pas de vrai téléphone, pas de
  Safari ni Firefox. Les images en mouvement peuvent décaler d'une image d'un rejeu à
  l'autre (horloge du navigateur) ; la preuve formelle est le test Rust du robot.

## Ventilateur, niveau d'essai « ventilateur » (carte 5.6)

Lettre `w` : un ventilateur à pied posé sur sa casette (socle 16x8, bas de la case, SOLIDE :
on peut s'y poser). Son courant ascendant (16 px de large, comme un vent « v ») souffle 2 s
(120 pas), puis s'arrête 2 s (72 pas d'arrêt + 48 pas d'annonce), période de 4 s (240 pas à
60 Hz). La reprise est annoncée comme un éclair : la lampe orange clignote de plus en plus
vite (`progression_annonce`, de 2,5 Hz à 15 Hz) et un repère au sol (barre orange au pied de
la casette) indique la colonne d'où le souffle va reprendre. Le courant ne porte que pendant
la phase de souffle. Aucune touche nouvelle. Aucun niveau du jeu n'est modifié (le
ventilateur viendra dans un niveau avec sa propre carte).

Ce qui a été fait :
- Cœur (`core/src/entities/ventilateur.rs`) : `Ventilateur { boite, zone, ... }`, machine à
  états `PhaseVentilateur` (Arret 72 / Annonce 48 / Souffle 120, période 240) avec durées
  entières à 60 Hz ; `courant_actif()` (un `CourantAscendant`, uniquement pendant le souffle),
  `en_annonce()`, `progression_annonce()` (0..1, pour accélérer le clignotement, comme
  l'éclair) ; `solide()` renvoie la casette (boîte 16x8, on peut s'y poser). Tests sur un
  MINI-NIVEAU (grille 8 x 8) : cycle exact (72+48+120 = 240), annonce avant la reprise,
  poussée seulement pendant le souffle, montée portée pendant le souffle, arrêt au plafond,
  redescente à l'arrêt, pas de glissement latéral, décalage qui raccourcit le premier arrêt.
- Analyse ASCII (`core/src/level/ascii.rs`, légende de `core/src/level.rs`) : « w » place un
  ventilateur sur sa case ; il exige une tuile SOLIDE juste en dessous (casette posée), sinon
  erreur `VentilateurSansAppui`. Le courant part du dessus de la casette (ty*16+8) jusqu'au
  premier solide au-dessus (plafond) ou le haut de la carte ; désynchronisation des
  ventilateurs (décalage 12 + 18*(k%4), comme pour les éclairs).
- Robot de faisabilité (`core/src/faisabilite.rs`) : le robot est aveugle au temps (il ne
  simule pas les phases) : il traite le courant d'un ventilateur comme un courant ascendant
  TOUJOURS actif (`options.courants`), car le joueur peut attendre la phase de souffle. Les
  courants des ventilateurs (`courants_des_ventilateurs`) s'ajoutent aux courants statiques du
  niveau.
- Robot de QA (`game/src/bot_qa.rs`) : `jouer_essai_ventilateur` /
  `parcours_essai_ventilateur` sur la vraie simulation du jeu (le souffle n'existe que pendant
  sa phase : le robot attend le bon moment). Test `le_robot_finit_le_niveau_d_essai_ventilateur`
  : 291 images de jeu, 3 gouttes, 0 mort, F4 une seule fois, arrivée au balcon (y = 18).
- Niveau d'essai « ESSAI VENTILATEUR » (`core/src/levels/essai_ventilateur.rs`, 24 x 22, thème
  Gouttières : nuit bleue), hors de la table `NIVEAUX`, touche F4 du build de QA seulement
  (« Essai ventilateur » 0 fois dans `dist/game.wasm`, 1 fois dans `dist-qa/`). Un balcon du
  drapeau à 18 tuiles au-dessus du sol (lignes 2-3 contre 20-21), inaccessible par un saut
  direct : seul le souffle du ventilateur (colonne 4) porte Pépin au balcon. 3 gouttes et
  3 lanternes sur le chemin du robot (1 goutte au sol entre le départ et le ventilateur ;
  2 gouttes et 3 lanternes sur le balcon, drapeau à la droite). Contrôles (`core/src/zones.rs`) :
  faisable par le robot précis ET le maladroit (3,8 s en course rapide, 48 états), 3 gouttes,
  3 lanternes, et SANS les courants le balcon reste hors de portée (preuve que c'est le
  ventilateur seul qui fait la montée).
- Jeu (`game/src/monde.rs`, `game/src/entrees.rs`, `game/src/contenu.rs`,
  `game/src/ecrans/carte.rs`) : `Monde::avancer` ajoute à l'`Environnement` les courants des
  ventilateurs actifs (phase de souffle uniquement, en temps réel) ; raccourci F4
  (`RaccourciNiveau::Essai5`, `monde::INDICE_ESSAI_VENTILATEUR` = NB_NIVEAUX + 4), intro
  `INTRO_ESSAI_VENTILATEUR`, carte de présentation. README : touche F4.
- Rendu (`game/src/sprites/objets.rs`, `game/src/scene.rs`) : trois sprites 16x16 (arrêt :
  pale dorée, rien au-dessus ; annonce : la lampe orange clignote au-dessus de la cage ;
  souffle : le courant d'air cyan remonte). Clignotement qui ACCÉLÈRE (même
  `annonce_allumee` et `progression_annonce` que l'éclair) ; repère au sol (barre orange de
  2 px au pied de la casette pendant l'annonce). Test : le ventilateur se lit sur tous les
  ciels (contour sombre, moyeu blanc ou courant cyan, contraste ≥ 4,5:1 sur chaque bande de
  ciel de chaque thème). Planche régénérée (`docs/planche-sprites.png`).

Mesure de difficulté : inchangée (aucun niveau, ennemi ni mécanique d'un niveau du jeu ne
bouge : le ventilateur n'apparaît dans aucun niveau du jeu). `docs/mesure-difficulte.txt`
identique à l'octet avant et après la carte.

Preuve visuelle : la planche de sprites (`docs/planche-sprites.png`, régénérée) montre les
trois images du ventilateur, distinctes et lisibles (cage cernée de sombre, pale dorée +
moyeu blanc, lampe orange, courant d'air cyan). Vérifiée à l'œil.

Vérification sur une extraction propre de 05187a0 (`git archive`, nouveau dossier daté) :
`./docker-cargo.sh test --workspace --no-default-features` : 237 + 244 tests, tous verts
(1 + 9 ignorés : écritures de fichiers et diagnostics) ; `./docker-cargo.sh lint` : `LINT-OK` ;
`./build.sh` : `dist/` produit, 0 occurrence des noms interdits, seule URL
`http://www.w3.org/2000/svg`, « Essai ventilateur » absent de `dist/game.wasm` (0) et présent
dans `dist-qa/game.wasm` (1, build de QA).

Limites (non testé ou à relativiser) :
- Le niveau d'essai n'existe que dans le build de QA ; aucun niveau du jeu n'a encore de
  ventilateur (il viendra avec sa carte).
- Le robot de faisabilité traite le courant comme TOUJOURS actif (aveugle au temps) : c'est
  une approximation assumée (le joueur peut attendre la phase de souffle) ; le robot de QA
  (vraie simulation, phases réelles) prouve que le niveau est réellement jouable.
- Le rendu EN JEU (clignotement qui accélère, repère au sol) n'a PAS été capturé dans un
  navigateur : le clignotement réutilise le code éprouvé de l'éclair
  (`annonce_allumee` + `progression_annonce`) et le repère au sol est une simple barre de
  2 px ; la preuve visuelle est la planche de sprites. Complété par la carte 5.6b : courant
  visible sur toute la colonne pendant le souffle et 15 captures `docs/qa/56-*` en jeu
  (annonce, souffle, arrêt, Pépin sur la casette, porté, au balcon), voir la section
  « Courant visible du ventilateur (carte 5.6b) ».
- Une seule colonne de souffle (16 px, une case) ; un ventilateur plus large (deux cases) n'a
  pas été fait.
- Le souffle n'a pas de son : rien n'est entendu (comme la plupart des mécaniques).

## Montgolfière, niveau d'essai « montgolfière » (carte 5.7)

Lettre `b` : une montgolfière. Sa nacelle (16 x 8, moitié HAUTE de la case `b`, SOLIDE :
son dessus est au ras d'un toit de la même rangée) monte quand Pépin s'y tient, à 48 px/s
(0,8 px par pas), jusqu'au premier solide au-dessus du ballon ou 8 tuiles au plus, puis
s'arrête ; quand il la quitte, elle redescend doucement (24 px/s) jusqu'à son altitude de
départ. Le ballon (deux cases au-dessus de la nacelle) n'est qu'un dessin : on le traverse,
mais il ne traverse rien (la montée s'arrête quand il toucherait un solide). Aucune touche
nouvelle. Aucun niveau du jeu n'est modifié.

Ce qui a été fait :
- Cœur (`core/src/entities/montgolfiere.rs`) : `Montgolfiere { boite, ... }`, états
  `EtatMontgolfiere` (Repos, Montee, EnHaut, Descente). À chaque pas : `actualiser` fait
  monter la nacelle si Pépin était à bord au pas précédent (sans dépasser le sommet, et
  seulement si ni le ballon ni Pépin debout dedans n'entreraient dans un solide : une
  corniche au-dessus du bord de la nacelle arrête la montée), sinon redescendre (jamais sur
  Pépin juste en dessous) ; `contacts::porter_par_plateformes` pose les pieds de Pépin PILE
  sur la nacelle (pas d'addition de déplacement : aucun cheveu d'arrondi, aucun glissement) ;
  `resoudre_contacts` appelle `embarquer` (Pépin posé dessus à la fin du pas ?). Nouvelle
  fonction `aabb::touche_un_obstacle`. `Montgolfiere::sommet` (pure) : le plafond ou 8
  tuiles. Tests sur des MINI-NIVEAUX (8 x 16) : montée portée sans glisser (100 pas : pieds
  sur la nacelle, x inchangé, 0,8 px par pas), arrêt pile à 8 tuiles, arrêt sous un plafond
  (le haut du ballon le touche), arrêt avant que la tête de Pépin ne cogne une corniche puis
  reprise quand il recule, redescente douce jusqu'à l'altitude de départ exacte, saut puis
  reprise de la montée, pas de redescente sur Pépin, repos sans passager.
- Analyse ASCII (`core/src/level/ascii.rs`, légende de `core/src/level.rs`) : « b » ; les
  deux cases au-dessus doivent être libres (ni solide, ni bord haut de la carte), sinon
  erreur `MontgolfiereSansPlace` (ligne, colonne). Tests du parseur (place de la nacelle,
  sommet sous un toit ou à 8 tuiles, refus).
- Robot de faisabilité (`core/src/faisabilite.rs`) : comme une plateforme mobile, deux
  nacelles immobiles (au repos et en haut) ; posé sur la nacelle au repos, il peut attendre la
  montée (coût : 128 px à 48 px/s = 160 pas pour 8 tuiles) si sa boîte monte sans rien
  toucher. Tests sur un mini-niveau : balcon atteint par la seule montgolfière (témoin sans
  nacelle), robot maladroit compris ; balcon hors de portée au-delà des 8 tuiles.
- Niveau d'essai « ESSAI MONTGOLFIÈRE » (`core/src/levels/essai_montgolfiere.rs`, 28 x 18,
  thème Altitude), hors de la table `NIVEAUX`, touche F5 du build de QA seulement
  (« Essai montgolfi » 0 fois dans `dist/game.wasm`, 1 fois dans `dist-qa/game.wasm`). Une
  tour de nuages en deux montgolfières : la première monte de 8 tuiles (rien au-dessus),
  la seconde s'arrête sous un nuage. Contrôles (`core/src/zones.rs`) : faisable par le robot
  précis et le maladroit (8,0 s en course rapide, 57 états), 3 gouttes, 3 lanternes ; sans
  montgolfière rien n'est atteint, et sans la seconde le drapeau reste hors de portée.
- Jeu (`game/src/monde.rs`, `game/src/entrees.rs`, `game/src/contenu.rs`,
  `game/src/ecrans/carte.rs`) : raccourci F5 (`RaccourciNiveau::Essai6`,
  `monde::INDICE_ESSAI_MONTGOLFIERE` = NB_NIVEAUX + 5), intro, carte de présentation ; test
  `le_raccourci_d_essai_charge_le_niveau_des_montgolfieres`. README : touche F5,
  `montgolfiere.rs`.
- Robot de QA (`game/src/bot_qa.rs`, vraie simulation) : `parcours_essai_montgolfiere` (le
  plus rapide : 399 images de jeu, il saute de la nacelle dès que l'étage est à portée,
  104 images portées, pieds pile sur la nacelle à chacune) et
  `parcours_essai_montgolfiere_jusqu_en_haut` (563 images de jeu, 262 de montée : la
  première monte de 8 tuiles, nacelle de y = 256 à y = 128, la seconde jusque sous le nuage,
  de y = 144 à y = 64 ; la première, quittée, redescend : y = 251,6 à la fin). Les deux :
  3 gouttes, 0 mort, F5 une fois. `ecrire_script_essai_montgolfiere` (ignoré) écrit
  `target/qa/script-57.txt`, `touches-57.txt` (229:F5) et `trace-57.csv`.
- Rendu (`game/src/sprites/objets.rs`, `game/src/scene.rs`) : trois sprites 20 x 40
  (enveloppe à fuseaux cyan et blancs, fuseau de droite dans l'ombre, cordes, nacelle
  tressée orange et or de 16 x 8 qui couvre exactement la boîte solide). Indice visuel : la
  flamme du brûleur éclaire la jupe de l'enveloppe et vacille (une image sur 4 pas)
  seulement pendant la MONTÉE. Tests : nacelle = boîte, seules les rangées de la jupe
  changent d'une image à l'autre, lisible sur tous les ciels (contour sombre, blanc ou cyan :
  contraste ≥ 4,5:1 sur chaque bande de ciel de chaque thème ; ni orange ni or dans
  l'enveloppe, qui se fondraient dans les ciels du soleil). Planche régénérée
  (`docs/planche-sprites.png`), vérifiée à l'œil.

Mesure de difficulté : inchangée (aucun niveau du jeu n'a de montgolfière ; le portage par
les plateformes mobiles n'a pas changé). `docs/mesure-difficulte.txt` régénéré sur
l'extraction propre de b524dcf : identique à l'octet au fichier versionné (celui de la
carte 5.6). Morts moyennes (avant = après) :

| Niveau                      | Débutant (avant = après) | Habitué (avant = après) | Expert (avant = après) |
| --------------------------- | ------------------------ | ----------------------- | ---------------------- |
| 1. Les toits                | 0,4                      | 0,1                     | 0,0                    |
| 2. Gouttières et cheminées  | 1,5                      | 0,5                     | 0,2                    |
| 3. Le grand orage           | 3,0                      | 0,6                     | 0,4                    |
| 4. Vents d'altitude         | 1,3                      | 0,4                     | 0,1                    |
| 5. Au-dessus des nuages     | 0,5                      | 0,1                     | 0,0                    |
| Total                       | 6,8                      | 1,8                     | 0,7                    |

Même jeu qu'avant, à l'octet (extraction propre de b524dcf contre les fichiers de
`target/qa/` écrits par les cartes précédentes) : `script.txt` et `trace.csv` du robot
principal (12 131 images, empreinte `e03e19a27c17c283`), et les scripts des essais vertical
(52), vent (53), grêle (54) et auvent (55).

Captures (`docs/qa/57-*`, 960 x 540, build de QA d'une extraction propre de b524dcf servi
en local sur 127.0.0.1, Chromium 145 piloté par CDP, onglet au premier plan et canvas
focalisé, pilote `docs/capture-visibilite.mjs` avec `SCRIPT=target/qa/script-57.txt` et
`TOUCHES=target/qa/touches-57.txt`, serveur arrêté à la fin) :
- `57-carte-niveau1` (image 101) puis `57-carte-essai` (351) : carte « ESSAI MONTGOLFIÈRE »,
  texte et astuce dans le cadre ;
- `57-essai-depart` (481) : Pépin sur le quai, la première nacelle au ras du quai, la
  seconde au ras de l'étage du milieu ; les ballons cyan et blanc se lisent sur le ciel
  corail et violet ;
- `57-essai-montee` (601) : Pépin debout dans la nacelle qui monte, la flamme jaune et
  orange dans la jupe (vérifié au pixel sur la grille logique, `GRILLE=1`) ;
- `57-essai-en-haut` (705) : arrêtée 8 tuiles plus haut, flamme éteinte ;
- `57-essai-redescente` (801) : Pépin sur l'étage du milieu, la première montgolfière
  redescend à vide sous l'étage ;
- `57-essai-seconde-montee` (871) : seconde montée, flamme allumée, le nuage au-dessus ;
- `57-essai-sous-le-nuage` (916) : le haut du ballon touche le nuage, Pépin saute ;
- `57-essai-arrivee` (1019) : Pépin attrape le drapeau, la seconde montgolfière redescend ;
- `57-fin` (60 images après la fin du script) : « LE SOLEIL ! », 0:09,3, 3 gouttes (3/15 :
  celles de l'essai), 0 mort.
Chaque capture regardée ; `docs/mesure-capture.py <capture> 3` : 0 pixel hors palette,
0 bloc flou, sur les 10.

Vérification sur une extraction propre de b524dcf (dernier commit de code ; `git archive`,
nouveau dossier daté) : `./docker-cargo.sh test --workspace --no-default-features` :
254 + 249 tests, tous verts (1 + 10 ignorés : écritures de fichiers et diagnostics) ;
`./docker-cargo.sh lint` : `LINT-OK` ; `./build.sh` : `dist/` produit, 0 occurrence des noms
interdits, seule URL `http://www.w3.org/2000/svg` ; `./build.sh --qa` : `dist-qa/` produit.

Limites (non testé ou à relativiser) :
- Le niveau d'essai n'existe que dans le build de QA ; aucun niveau du jeu n'a encore de
  montgolfière (la mer de nuages viendra avec sa carte).
- Le robot de faisabilité garde la nacelle « en haut » comme un solide permanent (comme
  les deux bouts d'une plateforme mobile) : il pourrait la croire atteignable sans y être
  monté. Il renonce à la montée quand la boîte de Pépin toucherait un solide (la vraie
  nacelle s'arrêterait plus bas). Le robot de QA, sur la vraie simulation, fait foi.
- Pépin peut se tenir sur le BORD de la nacelle (quelques pixels) : il est alors porté, mais
  paraît à côté des cordes (capture `57-essai-seconde-montee`). Comme pour les autres
  plateformes.
- Le parcours le plus rapide saute de la nacelle avant le haut dès que l'étage suivant est à
  portée : un niveau qui veut imposer la montée complète doit mettre l'étage hors de portée
  d'un saut depuis toute hauteur intermédiaire.
- Le brûleur n'a pas de son : rien n'est entendu. Captures dans Chromium seulement, à
  960 x 540 ; pas de vrai téléphone, pas de Safari ni Firefox.

## Courant visible du ventilateur (carte 5.6b)

Avant : le courant d'un ventilateur portait Pépin sans que rien ne le montre, alors que les
courants `v` du niveau se voient (brins blancs qui montent). Maintenant, pendant le souffle,
la colonne du ventilateur se voit de la même façon ; à l'arrêt et pendant l'annonce, rien.
Aucune règle de jeu ne change (simulation identique à l'octet, voir plus bas).

Ce qui a été fait :
- Cœur (`core/src/entities/ventilateur.rs`) : `souffle_restant()` en lecture seule (pas de
  courant restants, celui-ci compris ; 0 hors du souffle). Test
  `le_souffle_restant_compte_les_pas_de_courant` : 120 au premier pas du souffle, décompte,
  le courant cesse quand il est épuisé.
- Particules (`game/src/particules.rs`) : `souffle_borne(.., vie_max)`, le brin de `souffle`
  dont la vie est bornée ; `souffle` l'appelle avec `u8::MAX` (mêmes tirages, même vie : les
  courants `v` ne changent pas). Test `le_souffle_borne_ne_vit_pas_plus_que_demande`.
- Effets (`game/src/effets.rs`) : à côté des courants du niveau, un brin tous les 3 pas dans
  la colonne de chaque ventilateur qui souffle (`courant_actif`), même tirage de position.
  Sa vie est bornée à `souffle_restant + 1` : un brin né à un pas est dessiné après ce pas
  et les `vie - 2` suivants, donc le dernier s'efface pile à la première image où l'hélice
  est arrêtée. Choix : borner la vie plutôt que purger, ce qui garde `Particules` sans
  notion d'entité. Test `le_courant_du_ventilateur_se_voit_pendant_le_souffle_seulement`
  (ordre de `Application::pas`, 750 pas, trois souffles) : 0 brin à toute image hors
  souffle ; des brins sur au moins 90 % des images de souffle ; brins dans la colonne et
  dans la zone ; brins vus en haut ET en bas de la colonne. Il échoue (« 7 brin(s) de vent
  survivent hors du souffle », pas 180) si la vie n'est pas bornée, et aussi avec
  `souffle_restant + 2` : la borne est exacte.
- Robot de QA (`game/src/bot_qa.rs`) : la phase du ventilateur entre dans `ImageEssai` ;
  test ignoré `ecrire_script_essai_ventilateur` (`target/qa/script-56b.txt`,
  `touches-56b.txt` : F4 à l'image 229 du pilote, `trace-56b.csv` : phase et souffle
  restant à chaque image). 772 images, 3 gouttes, 0 mort. Le robot, lui, n'attend jamais
  posé sur la casette (il saute dans le souffle en cours) : d'où une démonstration à part,
  `parcours_essai_ventilateur_caisse` (Pépin attend la fin du premier souffle, saute sur la
  casette pendant l'arrêt, puis ne touche plus à rien ; saut cherché sur la vraie
  simulation). Test `pepin_attend_sur_la_casette_puis_le_souffle_le_porte` : posé sur la
  casette à toutes les images de l'annonce, porté à plus de 10 tuiles, retombé sur la
  casette, 0 mort. Test ignoré `ecrire_script_essai_ventilateur_caisse`
  (`target/qa/script-56b-caisse.txt`, 1 200 images, et `trace-56b-caisse.csv` ; même F4).

Mesure de difficulté : inchangée. `docs/mesure-difficulte.txt` régénéré sur l'extraction
propre de 971e528 (le dernier commit qui touche au jeu : 7afe2dc n'ajoute que du code de
test, `bot_qa` étant `#[cfg(test)]`) : identique à l'octet (Débutant 6,8, Habitué 1,8,
Expert 0,7 au total, avant = après). Script du robot principal (`script.txt`, `trace.csv`)
identique à l'octet (12 131 images, empreinte `e03e19a27c17c283`).

Captures (`docs/qa/56-*` : le préfixe 56- est celui que demande la carte, ce sont les
captures que la carte 5.6 n'avait pas faites ; 960 x 540, build de QA d'une extraction
propre de 971e528 servi en local sur 127.0.0.1, Chromium 145 piloté par CDP, onglet au
premier plan et canvas focalisé, pilote `docs/capture-visibilite.mjs` avec
`SCRIPT=target/qa/script-56b.txt`, `TOUCHES=target/qa/touches-56b.txt` et `GRILLE=1`,
serveur arrêté à la fin). Numéros d'image : ceux de `trace-56b.csv`.
- `56-carte-essai` (351) : carte « ESSAI VENTILATEUR ».
- Pépin laissé au départ (demande `attente:d=470:300:1`, une capture par image pendant
  300 images, un cycle complet) : `56-annonce` (497 : lampe orange allumée, barre au pied,
  aucun brin) ; `56-souffle-colonne` (614 : courant cyan sur la cage, brins dans la colonne
  du sol au haut de l'écran) ; `56-dernier-souffle` (634 : dernière image de souffle, brins
  encore là) ; `56-arret` (635 : première image d'arrêt, plus aucun brin).
- Parcours du robot : `56-montee` (544 : Pépin porté, brins sous lui dans la colonne) ;
  `56-haut-dernier-souffle` (634 : Pépin au-dessus du cadre, flèche de repère, brins sur
  toute la colonne) ; `56-haut-arret` (635 : colonne vide) ; `56-balcon` (680 : Pépin sur
  le balcon, colonne vide).
- Contrôle au pixel sur les 300 grilles logiques du cycle : sprite de souffle sur 120 images
  consécutives, d'arrêt sur 72, d'annonce sur 48 (le cycle du cœur) ; 0 pixel blanc cassé
  dans la colonne sur les 165 images d'arrêt et d'annonce ; brins visibles sur 131
  des 135 images de souffle vues (4 images sans brin au début d'un souffle : les brins
  naissent sur toute la hauteur de la colonne, 312 px, dont l'écran ne montre ici que le bas,
  140 px ; les premiers nés étaient au-dessus du cadre). Chaque capture regardée ;
  `docs/mesure-capture.py <capture> 3` : 0 pixel hors palette, 0 bloc flou, sur les 9.
- Démonstration « Pépin sur la casette » (`docs/qa/56-caisse-*`, même méthode, build de QA
  d'une extraction propre de 7afe2dc, `SCRIPT=target/qa/script-56b-caisse.txt`, même
  `TOUCHES`, serveur arrêté à la fin ; numéros d'image de `trace-56b-caisse.csv`) :
  `56-caisse-annonce` (734 : Pépin posé sur la casette, barre orange au pied, aucun brin) ;
  `56-caisse-portee` (840 : Pépin porté 12 tuiles plus haut, la colonne de brins sous lui
  le relie au ventilateur) ; `56-caisse-dernier-souffle` (874 : Pépin en haut, brins
  encore là) ; `56-caisse-arret` (875 : première image d'arrêt, plus aucun brin) ;
  `56-caisse-chute` (915 : Pépin retombe, colonne vide) ; `56-caisse-retombe` (945 : de
  nouveau sur la casette, hélice arrêtée). Au pixel (grilles logiques, boîte de Pépin
  écartée car il est blanc cassé lui aussi) : 0 brin aux images 731 à 736 (annonce), 875,
  915 et 945 ; 9 brins sous Pépin à l'image 840, 3 à l'image 874. Chaque capture regardée ;
  0 pixel hors palette, 0 bloc flou, sur les 6.

Vérification sur une extraction propre de 7afe2dc (dernier commit de code ; `git archive`,
nouveau dossier daté) : `./docker-cargo.sh test --workspace --no-default-features` :
255 + 252 tests, tous verts (1 + 12 ignorés : écritures de fichiers et diagnostics) ;
`./docker-cargo.sh lint` : `LINT-OK` ; `./build.sh` : `dist/` produit, 0 occurrence des noms
interdits, seule URL `http://www.w3.org/2000/svg` ; `./build.sh --qa` : `dist-qa/` produit
(« Essai ventilateur » : 0 fois dans `dist/game.wasm`, 1 fois dans `dist-qa/game.wasm`) ;
les scripts `script-56b`, `trace-56b`, `touches-56b`, `script-56b-caisse` et
`trace-56b-caisse` réécrits là-bas sont identiques à l'octet à ceux qui ont servi aux
captures. (Une première extraction, de 971e528, avait donné les mêmes résultats, avec
255 + 251 tests.)

Limites (non testé ou à relativiser) :
- Aucun niveau du jeu n'a encore de ventilateur : l'effet ne se voit que dans le niveau
  d'essai (touche F4 du build de QA).
- Brins un sur trois pas, à une hauteur tirée sur toute la colonne, comme les courants `v` :
  quand la caméra ne montre que le bas d'une colonne haute, quelques images de souffle
  peuvent n'avoir aucun brin dans le cadre (4 sur 135 mesurées ; le sprite cyan du
  ventilateur, lui, est là dès la première image).
- Les brins de la colonne sont blanc cassé comme ceux des courants `v`, et Pépin aussi : au
  pixel, un brin qui passe sur Pépin se confond avec lui.
- Pas de son. Captures dans Chromium seulement, à 960 x 540 ; pas de vrai téléphone, pas de
  Safari ni Firefox.

## Escargot, niveau d'essai « escargot » (carte 5.8)

Demande : l'escargot (`s`), « un champignon qui marche : impossible à écraser, il fait
rebondir. Il faut l'attendre au bon endroit. »

Ce qui a été fait :
- Cœur, `core/src/entities/escargot.rs` : entité `Escargot` (boîte 16 x 10 au bas de sa case).
  Il patrouille EXACTEMENT comme la flaque (demi-tour contre un mur et juste avant le bord
  du vide) : la marche au sol est sortie dans `core/src/entities/marche.rs`
  (`patrouiller`, `sol_devant`), que la flaque appelle désormais (même calcul). Il marche
  lentement : 16 px/s (une tuile par seconde, les deux tiers d'une flaque). Ce n'est pas un
  ennemi : lui tomber dessus (même règle que l'écrasement : en descendant, pieds enfoncés
  de 7 px au plus) fait rebondir Pépin à `VITESSE_CHAMPIGNON` (≈ 103 px, contre 57 pour un
  saut) avec le contact `Rebond` (même son, mêmes éclats que le champignon) ; sa coquille
  s'aplatit et il s'arrête 8 pas ; le toucher de côté ou en montant ne fait rien (Pépin le
  traverse). Lettre `s` dans le parseur, légende de `core/src/level.rs` à jour.
- Rendu : 3 sprites 16 x 12 (`game/src/sprites/ennemis.rs`, planche
  `docs/planche-sprites.png` régénérée) : marche (2 images, seules les cornes bougent) et
  coquille aplatie, œil fermé, au rebond. L'indice visuel est la coquille elle-même : c'est
  le chapeau du champignon (corail, reflet corail clair, pois blancs), on devine qu'il fait
  rebondir. Corps nuage, ventre brume, contour sombre : contraste d'au moins 4,5:1 sur toutes
  les bandes de ciel de tous les thèmes (test).
- Robot de faisabilité (`core/src/faisabilite.rs`) : il ne connaît pas le temps ; chaque
  escargot est remplacé par un tremplin immobile qui couvre TOUT son trajet
  (`Escargot::etendu_au_trajet`, trajet simulé sur la grille). Simplification documentée
  dans l'en-tête du module : le robot y rebondit partout, alors qu'un joueur doit attendre
  que l'escargot passe ; le robot de QA, lui, joue la vraie simulation.
- Robot de QA (`game/src/bot_qa.rs`) : l'escargot est un tremplin MOBILE, il faut
  l'attendre. La case de parcours (`CaseParcours`) ne retenait que la place de Pépin :
  attendre sur place ramenait à une case connue, le plan « attendre » était jeté et le
  robot ne trouvait aucun parcours. Elle gagne un 4e terme, `escargot_proche` (case de
  16 px et sens de l'escargot le plus proche, à 6 tuiles en x et 4 en y ; 0 sinon). Dans un
  niveau sans escargot il vaut toujours 0 : rien ne change pour les niveaux du jeu
  (`difficulte.rs` suit le nouveau type ; mesure identique, voir plus bas).
- Niveau d'essai `core/src/levels/essai_escargot.rs` (35 x 11, thème Orage), hors de la
  table `NIVEAUX`, touche F6 du build de QA seulement (`RaccourciNiveau::Essai7`,
  `INDICE_ESSAI_ESCARGOT`, intro et carte ; absent de `dist/`). Deux falaises : la première
  (80 px) est trop haute pour un saut, la seconde (96 px au-dessus d'une fosse de 160 px)
  trop loin de la première même en planant ; un escargot patrouille au pied de chacune.
  Une goutte en l'air (ligne 1, bien plus haut qu'un saut) sur la trajectoire du premier
  rebond. Aucun niveau du jeu n'est modifié.

Tests (tous sur la vraie physique) :
- `entities::escargot` (mini-niveau : un toit entre un mur et le vide) : place dans la case,
  patrouille lente entre le mur et le bord du vide sans jamais tomber, trajet, un seul
  rebond de champignon (coquille aplatie 8 pas, immobile, puis il remarche), rebond plus
  haut qu'un saut, traversée de côté sans aucun contact, pas de rebond en montant ni pieds
  trop enfoncés.
- `faisabilite` (mini-niveau : falaise de 80 px) : sans escargot la falaise est hors de
  portée ; avec lui le robot, précis et maladroit, y monte et prend les 3 gouttes ; le
  tremplin couvre bien du mur au pied de la falaise.
- `zones` : l'essai passe les contrôles d'un vrai niveau (arrivée, 3 gouttes, lanternes,
  robot maladroit, reprise depuis chaque lanterne, lint de grille) ; sans escargot rien
  n'est atteint, sans le second le drapeau reste hors de portée (chacun sert).
- Jeu : sprites (taille, contour, coquille de champignon, contraste), images choisies par la
  scène, touche F6 et niveau chargé avec ses deux escargots, carte dans son cadre ; robot de
  QA `le_robot_finit_le_niveau_d_essai_escargot` : fin atteinte, 0 mort, 3 gouttes, F6 une
  fois, arrivée en haut de la falaise 2, 2 rebonds sur un escargot (coquille aplatie à
  chaque fois), 137 images où l'escargot traverse Pépin posé au sol sans rien faire.
  `le_robot_finit_tous_les_niveaux_sans_mourir` reste vert.

Mesure de difficulté : inchangée (aucun niveau du jeu n'a d'escargot ; la marche de la
flaque est le même calcul, déplacé ; la case de parcours ne change que près d'un escargot).
`docs/mesure-difficulte.txt` régénéré sur l'extraction propre de 35a6654 : identique à
l'octet au fichier versionné. Morts moyennes (avant = après) :

| Niveau                      | Débutant (avant = après) | Habitué (avant = après) | Expert (avant = après) |
| --------------------------- | ------------------------ | ----------------------- | ---------------------- |
| 1. Les toits                | 0,4                      | 0,1                     | 0,0                    |
| 2. Gouttières et cheminées  | 1,5                      | 0,5                     | 0,2                    |
| 3. Le grand orage           | 3,0                      | 0,6                     | 0,4                    |
| 4. Vents d'altitude         | 1,3                      | 0,4                     | 0,1                    |
| 5. Au-dessus des nuages     | 0,5                      | 0,1                     | 0,0                    |
| Total                       | 6,8                      | 1,8                     | 0,7                    |

Captures (`docs/qa/58-*`, 960 x 540, build de QA d'une extraction propre de 35a6654 servi en
local sur 127.0.0.1, Chromium (révision 1208) piloté par CDP, onglet au premier plan et
canvas focalisé, pilote `docs/capture-visibilite.mjs` avec `SCRIPT=target/qa/script-58.txt`
et `TOUCHES=target/qa/touches-58.txt` (F6 à l'image 229), serveur arrêté à la fin ;
numéros d'image de `trace-58.csv`, écrits par le test ignoré `ecrire_script_essai_escargot`) :
- `58-carte-niveau1` (101) puis `58-carte-essai` (351) : carte « ESSAI ESCARGOT », texte et
  astuce dans le cadre ;
- `58-essai-depart` (481) : Pépin au départ, l'escargot sur le sol, la falaise à droite ;
- `58-essai-de-cote` (860) : l'escargot passe à travers Pépin posé au sol : rien ne se
  passe (ni mort, ni rebond) ;
- `58-essai-attente` (1200) : Pépin attend au bord gauche, l'escargot repart vers la falaise ;
- `58-essai-saut` (1250) : Pépin saute et plane vers lui ;
- `58-essai-rebond` (1332) et `58-essai-rebond-suite` (1336) : Pépin retombe sur la coquille,
  qui s'aplatit (œil fermé), éclats corail, Pépin repart vers le haut ;
- `58-essai-goutte-en-l-air` (1347) puis `58-essai-haut` (1356) : porté bien plus haut que la
  falaise, il prend la goutte en l'air (1/15) ;
- `58-essai-falaise1` (1450) : posé sur la falaise 1, lanterne allumée ;
- `58-essai-fosse` (1560) puis `58-essai-rebond2` (1583) : au fond de la fosse, second rebond
  au pied de la falaise 2 (coquille aplatie) ;
- `58-essai-arrivee` (1660) : Pépin au drapeau, 3/15 ;
- `58-fin` (60 images après la fin du script) : « LE SOLEIL ! », 0:20,1, 3 gouttes (3/15 :
  celles de l'essai), 0 mort.
Chaque capture regardée ; `docs/mesure-capture.py <capture> 3` : 0 pixel hors palette,
0 bloc flou, sur les 15.

Vérification sur une extraction propre de 35a6654 (dernier commit de code ; `git archive`,
nouveau dossier daté) : `./docker-cargo.sh test --workspace --no-default-features` :
267 + 258 tests, tous verts (1 + 13 ignorés : écritures de fichiers et diagnostics) ;
`./docker-cargo.sh lint` : `LINT-OK` ; `./build.sh` : `dist/` produit, 0 occurrence des noms
interdits, seule URL `http://www.w3.org/2000/svg` ; `./build.sh --qa` : `dist-qa/` produit
(« Essai escargot » : 0 fois dans `dist/game.wasm`, 1 fois dans `dist-qa/game.wasm`).

Limites (non testé ou à relativiser) :
- Aucun niveau du jeu n'a encore d'escargot (le clocher viendra ensuite) : il ne se voit
  que dans le niveau d'essai (touche F6 du build de QA).
- Le robot de faisabilité voit l'escargot comme un tremplin immobile sur tout son trajet :
  il peut juger faisable un passage où le timing serait très serré. Seul le robot de QA
  (vraie simulation) prouve qu'on peut l'attendre ; il faudra garder les deux verts sur
  chaque niveau qui en aura.
- Un escargot ne pousse pas Pépin et ne réagit pas à lui (il le traverse) ; il ne monte pas
  sur les plateformes mobiles en route (il suit les solides comme la flaque).
- Pas de son propre (le « boing » du champignon). Captures dans Chromium seulement, à
  960 x 540 ; pas de vrai téléphone, pas de Safari ni Firefox.

## Chat endormi, niveau d'essai « chat » (carte 5.9)

Demande : le chat endormi (`z`, pour le marché couvert) : immobile ; cycle annoncé : il dort
(« zzz », plusieurs secondes) puis se réveille brièvement. Endormi, on l'écrase en lui sautant
dessus ; éveillé, le toucher tue Pépin. Il bloque l'accès à une goutte cachée.

Reprise : le cœur et la faisabilité avaient été écrits par erreur pendant la carte 5.8 et mis
de côté sur la branche locale `brouillon-5.9-chat` (voir le commentaire de la carte). Ils sont
repris par cherry-pick sur le HEAD de la 5.8 (7790fe5, b9494d3 : un conflit d'ajout de tests
résolu à la main dans `faisabilite.rs`), puis vérifiés ; le rendu (brouillon non compilé) a
été réappliqué à la main, relu et corrigé. La branche de brouillon n'est plus utile (non
supprimée : à faire par le coordinateur s'il le souhaite).

Ce qui a été fait :
- Cœur, `core/src/entities/ennemi/chat.rs` : `TypeEnnemi::Chat` (boîte 14 x 8, immobile),
  cycle `PhaseChat` en pas entiers : `Dort` 240 pas (4 s) → `Reveil` 48 pas (0,8 s,
  l'annonce, comme l'éclair) → `Eveille` 72 pas (1,2 s) → `Dort`… Endormi ou en train de se
  réveiller, c'est un ennemi comme les autres (écrasé par le dessus, mortel de côté) ; éveillé,
  `est_ecrasable` est faux : le toucher tue, même par le dessus (`contacts.rs`). La protection
  de réapparition s'applique aussi au chat. Lettre `z` : posé sur le solide du dessous, la case
  du dessus libre (sinon `ErreurNiveau::ChatSansPlace`), tourné vers le départ sauf s'il est
  adossé à un mur ; deux chats d'un niveau ne se réveillent pas ensemble (premiers sommeils
  raccourcis de 0, 40, 80 ou 120 pas). Légende de `core/src/level.rs` à jour. Ajout de cette
  carte : `pas_du_cycle` (où en est le chat dans son cycle de 360 pas), fonction pure testée.
- Rendu : 5 sprites 16 x 14 (`game/src/sprites/ennemis.rs`) : dort (2 images, le dos respire),
  réveil (tête levée, un œil d'or ouvert), éveillé (assis, hérissé, queue dressée, deux yeux
  d'or), écrasé (redessiné par rapport au brouillon : aplati, oreilles rentrées, yeux plissés,
  petit nez rose, il reste mignon). Indices : deux « z » blancs cernés de sombre qui montent
  pendant le sommeil, puis un « ! » doré qui clignote de plus en plus vite pendant l'annonce
  (même accélération que l'éclair). Contour sombre ou blanc : contraste d'au moins 4,5:1 sur
  toutes les bandes de ciel de tous les thèmes (test). Planche `docs/planche-sprites.png`
  régénérée.
- Robot de faisabilité (`core/src/faisabilite.rs`) : il ne connaît pas le temps ; le chat y
  est toujours endormi (le joueur patient attend son sommeil) : il le heurte de côté (mort) ou
  l'écrase par le dessus, et chaque état retient les chats écrasés (`Noeud::chats`, un bit par
  chat), sinon « derrière le chat vivant » et « derrière le chat écrasé » fusionneraient. Le
  lint a demandé un alias de type pour la file de priorité (`EntreeTas`, 4be9968).
- Robot de QA (`game/src/bot_qa.rs`) : la case de parcours gagne un 5e terme,
  `chats_de_la_case` = 16 × (chats écrasés, un bit chacun) + tranche de 0,5 s du cycle du chat
  vivant le plus proche (à 6 tuiles en x et 4 en y ; 0 sinon). Sans la tranche, attendre sur
  place ramènerait à une case connue et le robot ne pourrait jamais laisser le chat se
  rendormir. Dans un niveau sans chat le terme vaut toujours 0 (`difficulte.rs` suit le type).
  De plus, un chat écrasé à moins de 6 pas (0,1 s) de son éveil compte comme une mort pour le
  robot (`chat_ecrase_de_justesse`) : son premier parcours tombait sur le chat au tout premier
  pas de son sommeil, et le rejeu dans le navigateur, décalé d'une image, tombait sur le chat
  encore éveillé (mort vue sur capture). Avec la marge, le chat est écrasé 6 pas après son
  endormissement.
- Niveau d'essai `core/src/levels/essai_chat.rs` (42 x 11, thème Toits), hors de la table
  `NIVEAUX`, touche F7 du build de QA seulement (`RaccourciNiveau::Essai8`,
  `INDICE_ESSAI_CHAT`, intro et carte ; absent de `dist/`). Le chat dort au fond d'une fente
  d'une tuile entre un bloc de toit (plein jusqu'au sol) et un grand mur qui monte jusqu'au
  haut de la carte ; le seul passage vers la goutte cachée et le drapeau part de sa case, sous
  le mur. La course jusqu'au bloc est assez longue pour que Pépin y arrive pendant l'éveil :
  il faut l'y attendre. Gouttes : en haut de la fente (prise en se laissant tomber), cachée
  derrière le chat, avant le drapeau. Aucun niveau du jeu n'est modifié.

Tests (tous sur la vraie physique) :
- `entities::ennemi::chat` (mini-niveau « niche » : une goutte au fond d'un renfoncement
  gardé par le chat) : durées du cycle, ordre des phases, décalage du premier sommeil,
  écrasable en dormant et pendant l'annonce jamais éveillé, immobile, foncer dedans tue,
  tomber dessus endormi l'écrase et ouvre la niche, tomber dessus éveillé tue, un chat écrasé
  ne se réveille plus et disparaît, `pas_du_cycle`. `contacts` : écrasé par le dessus endormi,
  mortel de côté, éveillé mortel même par le dessus, protection de réapparition. `level` :
  couché sur son toit et tourné vers le départ, refusé sans appui ou sans place.
- `faisabilite` (mini-niveau) : le robot, précis et maladroit, écrase le chat pour prendre la
  goutte cachée ; marcher dans un chat vivant le tue (le même plan passe si le chat est déjà
  écrasé) ; enfermé au fond de la niche, il en sort en retombant sur le chat.
- `zones` : l'essai passe les contrôles d'un vrai niveau (arrivée, 3 gouttes, lanternes,
  robot maladroit, reprise depuis chaque lanterne, lint de grille) ; la case du chat bouchée
  par un toit, ni la goutte cachée ni le drapeau ne sont atteints (c'est le seul passage).
- Jeu : sprites (5 images de même taille, contour, yeux fermés endormi et écrasé, un œil
  ouvert au réveil, deux éveillé, contraste sur tous les ciels), image choisie par phase,
  « z » qui montent et « ! » qui clignote plus vite à la fin, touche F7 et niveau chargé avec
  son chat, carte dans son cadre ; robot de QA `le_robot_finit_le_niveau_d_essai_chat` : fin
  atteinte, 0 mort, 3 gouttes, F7 une fois, un seul écrasement, chat endormi au moment du
  contact et pas de justesse (Dort(234) : 6 pas après l'endormissement), passage sous le mur
  seulement après, attente sur le bloc pendant l'éveil (1 réveil vu avant l'écrasement).
  `le_robot_finit_tous_les_niveaux_sans_mourir` reste vert.

Mesure de difficulté : inchangée (aucun niveau du jeu n'a de chat ; le terme ajouté à la case
de parcours y vaut toujours 0). `docs/mesure-difficulte.txt` régénéré sur l'extraction propre
de 75724e8 : identique à l'octet au fichier versionné. Morts moyennes (avant = après) :

| Niveau                      | Débutant (avant = après) | Habitué (avant = après) | Expert (avant = après) |
| --------------------------- | ------------------------ | ----------------------- | ---------------------- |
| 1. Les toits                | 0,4                      | 0,1                     | 0,0                    |
| 2. Gouttières et cheminées  | 1,5                      | 0,5                     | 0,2                    |
| 3. Le grand orage           | 3,0                      | 0,6                     | 0,4                    |
| 4. Vents d'altitude         | 1,3                      | 0,4                     | 0,1                    |
| 5. Au-dessus des nuages     | 0,5                      | 0,1                     | 0,0                    |
| Total                       | 6,8                      | 1,8                     | 0,7                    |

Captures (`docs/qa/59-*`, 960 x 540, build de QA d'une extraction propre de 75724e8 servi en
local sur 127.0.0.1, Chromium (révision 1208) piloté par CDP, onglet au premier plan et
canvas focalisé, pilote `docs/capture-visibilite.mjs` avec `SCRIPT=target/qa/script-59.txt`
et `TOUCHES=target/qa/touches-59.txt` (F7 à l'image 229), serveur arrêté à la fin ;
numéros d'image de `trace-59.csv`, écrits par le test ignoré `ecrire_script_essai_chat`) :
- `59-carte-niveau1` (101) puis `59-carte-essai` (351) : carte « ESSAI CHAT », texte et
  astuce (« Attends les « z », puis tombe-lui dessus. ») dans le cadre ;
- `59-essai-depart` (481) : Pépin au départ, le bloc et le grand mur à droite ;
- `59-essai-dort` (685) : le chat roulé en boule au fond de la fente, deux « z » au-dessus ;
- `59-essai-reveil` (700) et `59-essai-reveil-fin` (730) : il lève la tête, le « ! » doré
  au-dessus ; Pépin plane vers le bloc, puis arrive au-dessus ;
- `59-essai-eveille` (760) : le chat assis, hérissé, yeux d'or ; Pépin attend sur le bloc ;
- `59-essai-attente` (800) : Pépin au bord du bloc, le chat toujours éveillé ;
- `59-essai-chute` (816) : le chat rendormi (« z »), Pépin tombe dans la fente et prend la
  goutte du haut (1/15) ;
- `59-essai-ecrase` (821) et `59-essai-ecrase-suite` (830) : le chat aplati sous Pépin
  (éclaboussures), puis Pépin rebondit au-dessus de lui ;
- `59-essai-passage` (870) puis `59-essai-goutte-cachee` (899) : Pépin passe sous le grand
  mur et prend la goutte cachée (2/15) ;
- `59-essai-arrivee` (1028) : Pépin au drapeau, 3/15 ;
- `59-fin` (60 images après la fin du script) : « LE SOLEIL ! », 0:09,5, 3 gouttes (3/15 :
  celles de l'essai), 0 mort ;
- `59-mort-contact` (808) : série « chute » du pilote, le robot lâché à l'image 744 (chat
  éveillé, Pépin sur le bloc) court tout droit : Pépin tombe du bloc dans la fente sur le chat
  ÉVEILLÉ ; `59-mort-suite` (812) : éclats sur le chat resté assis (il ne s'écrase pas),
  Pépin réapparaît à la lanterne du bloc.
Chaque capture regardée ; `docs/mesure-capture.py <capture> 3` : 0 pixel hors palette,
0 bloc flou, sur les 17.

Vérification sur une extraction propre de 75724e8 (dernier commit de code ; `git archive`,
nouveau dossier daté) : `./docker-cargo.sh test --workspace --no-default-features` :
291 + 265 tests, tous verts (1 + 14 ignorés : écritures de fichiers et diagnostics) ;
`./docker-cargo.sh lint` : `LINT-OK` ; `./build.sh` : `dist/` produit, 0 occurrence des noms
interdits, seule URL `http://www.w3.org/2000/svg` ; `./build.sh --qa` : `dist-qa/` produit
(« Essai chat » : 0 fois dans `dist/game.wasm`, 1 fois dans `dist-qa/game.wasm`) ; planche
de sprites régénérée identique à l'octet.

Limites (non testé ou à relativiser) :
- Aucun niveau du jeu n'a encore de chat (le marché couvert viendra ensuite) : il ne se voit
  que dans le niveau d'essai (touche F7 du build de QA).
- Le robot de faisabilité voit le chat toujours endormi : il ne juge pas le timing. Seul le
  robot de QA (vraie simulation, avec la marge de 0,1 s) prouve qu'on peut attendre son
  sommeil ; il faudra garder les deux verts sur chaque niveau qui en aura.
- La marge de 0,1 s est une règle du robot, pas du jeu : un joueur peut écraser le chat au
  tout dernier pas de son annonce.
- Pas de son propre (l'écrasement réutilise celui des autres ennemis) ; pas de miaulement au
  réveil. Captures dans Chromium seulement, à 960 x 540 ; pas de vrai téléphone, pas de Safari
  ni Firefox.

## Aiguilles d'horloge, niveau d'essai « horloge » (carte 5.10)

Demande : les aiguilles de la cage de l'horloge du futur clocher (`h`) : extension de
`core/src/entities/mobile.rs`, une plateforme qui tourne autour d'un pivot (trajectoire
circulaire, rayon et vitesse réglables, sens horaire), dont le dessus reste horizontal pour
qu'on s'y tienne ; elle transporte Pépin. Lettre `h` = le pivot ; rayon, vitesse et phase dans
une petite table du fichier de niveau, sur le modèle des plateformes mobiles à chiffres.
Sprite aiguille + cadran visible. Tests : position sur le cercle, transport, pas de traversée.

Ce qui a été fait :
- Cœur, `core/src/entities/mobile.rs` : `PlateformeMobile` garde sa structure publique ; son
  trajet devient un `enum` privé, `AllerRetour` (inchangé : les plateformes du niveau 3 font
  exactement les mêmes calculs) ou `Cercle`. `PlateformeMobile::aiguille(taille, pivot, rayon,
  vitesse, tour)` : le CENTRE de la plaque suit le cercle dans le sens horaire (midi au-dessus
  du pivot, puis 3 h à droite…), la boîte reste un rectangle droit (dessus horizontal, comme
  la nacelle d'une grande roue). La place se recalcule à chaque pas depuis la fraction de tour
  (aucune erreur qui s'accumule). Le sinus est « maison » (`cos_sin_tour` : série de Taylor sur
  un quart de tour, puis symétries ; seulement + - × ÷, donc le même résultat au bit près en
  natif, où le robot calcule, et en wasm, où le navigateur rejoue) ; testé contre la
  bibliothèque standard (écart < 3e-7). `boite_avant` retient EXACTEMENT la place d'avant le
  pas (reculer du déplacement pouvait tomber un cheveu à côté : Pépin n'aurait plus été « posé
  dessus ») ; `tourner(pas)` avance l'aiguille de plusieurs pas d'un coup (pour le robot de QA).
- Format : `h` = pivot (centre de la case). Réglages : `level::ReglageAiguille { rayon (en
  tuiles), vitesse (px/s), phase (heure de départ, 0 à 12) }`, dans une table
  `InfosNiveau::aiguilles` du fichier de niveau ; la 1re ligne règle le 1er `h` dans l'ordre de
  lecture, etc. (comme un chiffre désigne une plateforme mobile). `Niveau::depuis_ascii` lit un
  niveau sans aiguille ; `depuis_ascii_avec_aiguilles` avec. Erreurs typées : pas autant de
  réglages que de pivots (`AiguillesSansReglage`), réglage hors des bornes (rayon < 1 tuile,
  vitesse nulle ou > 120 px/s, phase non finie : `AiguilleInvalide`), cercle qui touche une
  tuile solide, la plaque OU Pépin debout dessus (`AiguilleBloquee` : vérifié pas à pas sur un
  tour complet, comme dans le jeu). Plaque 32 x 8 (`level::TAILLE_AIGUILLE`). Légende de
  `core/src/level.rs` à jour. Le nouveau champ `aiguilles` a obligé à ajouter une ligne
  `aiguilles: &[],` dans chaque fichier de niveau (5 du jeu, 8 d'essai) : aucune grille ni
  aucun autre champ n'a changé (mesure de difficulté identique à l'octet, ci-dessous).
- Portage (`contacts::porter_par_plateformes`) : l'aiguille bouge en x ET en y à chaque pas ;
  porter Pépin des deux d'un coup échouait (collision traitée axe X d'abord : quand l'aiguille
  monte, ses pieds sont déjà un peu dedans, il était repoussé de côté). `porter_sur_aiguille` :
  en x, Pépin suit l'aiguille sans buter sur elle (les murs l'arrêtent), puis ses pieds sont
  posés pile sur son dessus (comme dans la nacelle de la montgolfière) si la place est libre et
  s'il est encore au-dessus d'elle.
- Robot de faisabilité (`core/src/faisabilite.rs`) : il ne connaît pas le temps ; une aiguille
  y devient DOUZE plateformes immobiles, une par heure du cadran. Posé sur l'une, il est aussi
  considéré posé sur chacune des 11 autres (il attend que l'aiguille l'y emmène, sens horaire),
  avec la durée du trajet comme coût, si sa boîte ne touche aucune tuile en chemin
  (`Cadran::trajet`, 8 regards par heure). Les douze places présentes à la fois gênent parfois
  un saut qui passerait en vrai : c'est prudent (documenté en tête du module).
- Niveau d'essai `core/src/levels/essai_horloge.rs` (42 x 18, thème Orage, le violet du futur
  clocher), hors de la table `NIVEAUX`, touche F8 du build de QA seulement
  (`RaccourciNiveau::Essai9`, `INDICE_ESSAI_HORLOGE`, intro et carte ; absent de `dist/`). Deux
  aiguilles : la grande (4 tuiles, 40 px/s, un tour en 10 s, partie de 4 h) part du sol du
  départ et monte assez haut pour sauter sur la tour B (160 px au-dessus du sol, hors de portée
  d'un saut) ; la petite (3 tuiles, 36 px/s, partie de 9 h) mène de la tour B à la tour C du
  drapeau (64 px plus haut). Entre les deux tours, un gouffre. Gouttes : sol, tour B, tour C
  (au bord où l'on atterrit) ; lanternes : départ, tour B, tour C.
- Rendu (`game/src/sprites/objets.rs`, `game/src/scene.rs`) : plaque de laiton 32 x 8 qui
  couvre toute sa boîte de collision (contour sombre, dessus or pâle où l'on pose les pieds,
  ombre orange, trois rivets) ; cadran 32 x 32 centré sur le pivot (contour sombre, bord doré
  ombré en bas à droite, filet, fond blanc cassé, douze graduations dont 12, 3, 6 et 9 plus
  grosses, moyeu doré) ; tige de maillons 4 x 4 tous les 6 px, du moyeu au milieu de la plaque :
  elle tourne comme l'aiguille d'une montre, mais seules les POSITIONS changent (entières),
  aucun sprite n'est tourné ni agrandi (`maillons`, fonction pure testée). Les cadrans et leurs
  tiges sont dessinés avant les entités (la plaque, Pépin, les gouttes passent devant).
  Contour sombre, or ou blanc cassé : contraste d'au moins 4,5:1 sur toutes les bandes de ciel
  de tous les thèmes (test). Planche `docs/planche-sprites.png` régénérée.
- Robot de QA (`game/src/bot_qa.rs`) : la case de parcours gagne un 6e terme,
  `aiguille_proche` = 1 + l'heure (0 à 11) de l'aiguille la plus proche (pivot à moins de son
  rayon + 6 tuiles ; 0 sinon, donc toujours 0 dans les niveaux du jeu ; `difficulte.rs` suit
  le type). Sans l'heure, attendre au pied de l'aiguille ramènerait à une case connue : le robot
  ne pourrait jamais la laisser descendre jusqu'à lui. De plus, dans un niveau à aiguilles, un
  plan n'est gardé que s'il TIENT avec les aiguilles décalées de 1 ou 2 images d'avance ou de
  retard (même fin, mêmes gouttes, Pépin à 4 px près : `tient_au_decalage`) et s'il ne pose
  jamais Pépin à moins de 3 px du bord d'une plaque. Raison : le premier parcours trouvé était
  réglé à l'image près ; rejoué avec les touches décalées d'une image (test temporaire de
  diagnostic, non gardé), il ratait la petite aiguille et tombait deux fois dans le gouffre
  (pas 561 et 706), alors que le rejeu dans le navigateur peut être décalé d'une image (déjà
  noté aux cartes 3.1b, 5.2 et 5.9). La première série de captures, prise avec ce parcours, a
  été jetée. Avec la règle, le parcours finit avec les touches décalées de -2 à +2 images
  (test `le_parcours_de_l_horloge_tient_a_une_image_pres`).

Tests ajoutés (sur des mini-niveaux de test, dans les tests seulement) :
- `entities::mobile` : sinus « maison » contre la bibliothèque standard (1000 positions,
  valeurs exactes aux quatre quarts) ; place sur le cadran (midi au-dessus du pivot, 3 h à
  droite, 6 h dessous, 9 h à gauche) ; deux tours complets : centre toujours sur le cercle
  (1e-3 près), taille de la boîte inchangée (dessus horizontal), vitesse 36 px/s le long du
  cercle, quarts franchis dans l'ordre 1, 2, 3, 0 (sens horaire), durée d'un tour 2πr/v ; partie
  de 3 h, elle descend aussitôt ; `boite_avant` exacte ; `tourner` (d'un coup = pas à pas, aller
  et retour, sans effet sur un aller-retour).
- `entities::contacts` (mini-niveau d'une aiguille de 2 tuiles) : Pépin immobile sur l'aiguille
  est porté deux tours complets, les pieds pile sur le dessus et au même endroit de la plaque à
  CHAQUE pas (64 px de montée et de descente) ; lâché à pleine vitesse de chute 40 px au-dessus,
  aux 12 heures du cadran (elle monte, descend, file de côté), il ne la chevauche jamais et
  finit posé dessus ; sautant dessous, il s'y cogne la tête sans jamais la chevaucher et
  retombe au sol.
- `level` : deux pivots réglés dans l'ordre de lecture (position, rayon, vitesse, heure de
  départ) ; réglage manquant, en trop, ou hors des bornes refusé ; cercle bloqué par un toit
  refusé ; Pépin debout dessus compte (un toit que la plaque frôle sans le toucher, mais où sa
  tête cognerait, est refusé).
- `faisabilite` (mini-niveau d'un balcon et d'une grande aiguille) : le robot, précis et
  maladroit, monte au balcon en se laissant porter ; sans aiguille, il n'y arrive pas ; le
  trajet de 6 h à midi (place, durée d'un demi-tour) et un trajet refusé par une tuile sur le
  chemin de la tête.
- `zones` : l'essai passe les contrôles d'un vrai niveau (arrivée, 3 gouttes, lanternes, robot
  maladroit, reprise depuis chaque lanterne, lint de grille) ; sans aiguille, seule la goutte
  du sol est prise ; sans la petite, ni la tour C ni le drapeau ne sont atteints.
- Jeu : plaque qui couvre sa collision, cadran rond et symétrique cerné de sombre, contraste sur
  tous les ciels, tige de maillons (nombre, écart, alignement, à midi, à 3 h, en diagonale),
  touche F8 et niveau chargé avec ses deux aiguilles, carte dans son cadre ; robot de QA
  `le_robot_finit_le_niveau_d_essai_horloge` : fin atteinte, 0 mort, 3 gouttes, F8 une fois,
  porté par CHACUNE des deux aiguilles (grande : 178 images, 114,9 px de montée ; petite : 56
  images, 18,2 px), les pieds pile sur la plaque à chaque image portée.
  `le_robot_finit_tous_les_niveaux_sans_mourir` reste vert.

Mesure de difficulté : inchangée (aucun niveau du jeu n'a d'aiguille ; le terme ajouté à la
case de parcours y vaut toujours 0, la règle du décalage ne s'applique qu'avec des aiguilles).
`docs/mesure-difficulte.txt` régénéré sur l'extraction propre de fe90c64 : identique à l'octet
au fichier versionné. Morts moyennes (avant = après) :

| Niveau                      | Débutant (avant = après) | Habitué (avant = après) | Expert (avant = après) |
| --------------------------- | ------------------------ | ----------------------- | ---------------------- |
| 1. Les toits                | 0,4                      | 0,1                     | 0,0                    |
| 2. Gouttières et cheminées  | 1,5                      | 0,5                     | 0,2                    |
| 3. Le grand orage           | 3,0                      | 0,6                     | 0,4                    |
| 4. Vents d'altitude         | 1,3                      | 0,4                     | 0,1                    |
| 5. Au-dessus des nuages     | 0,5                      | 0,1                     | 0,0                    |
| Total                       | 6,8                      | 1,8                     | 0,7                    |

Captures (`docs/qa/510-*`, 960 x 540, build de QA d'une extraction propre de fe90c64 servi en
local sur 127.0.0.1, Chromium (révision 1208) piloté par CDP, onglet au premier plan et canvas
focalisé, pilote `docs/capture-visibilite.mjs` avec `SCRIPT=target/qa/script-510.txt` et
`TOUCHES=target/qa/touches-510.txt` (F8 à l'image 229), serveur arrêté à la fin ; numéros
d'image de `trace-510.csv`, écrits par le test ignoré `ecrire_script_essai_horloge`) :
- `510-carte-niveau1` (101) puis `510-carte-essai` (351) : carte « ESSAI HORLOGE », texte et
  astuce (« Saute dessus quand elle passe en bas, saute en haut. ») dans le cadre ;
- `510-essai-depart` (481) : Pépin au départ, la lanterne et la goutte du sol ; le cadran de la
  grande aiguille, sa tige et sa plaque vers 4 h ;
- `510-essai-saut` (591) : Pépin saute vers la plaque qui passe au plus bas (vers 7 h) ;
- `510-essai-porte-bas` (621), `510-essai-porte-monte` (726), `510-essai-porte-haut` (801) :
  Pépin debout sur la plaque, porté de 7 h à 9 h puis vers 11 h ; la caméra le suit vers le
  haut ; la tour B apparaît à droite ;
- `510-essai-saut-tour` (846) : Pépin plane vers la tour B ;
- `510-essai-tour-b` (936) : sur la tour B, la goutte devant lui, la petite aiguille à droite ;
- `510-essai-saut-petite` (996) puis `510-essai-petite` (1041) : saut sur la plaque de la
  petite aiguille (vers 10 h), puis porté vers 11 h, la tour C à droite (2/15) ;
- `510-essai-saut-c` (1086) : Pépin au-dessus du cadre (repère en haut de l'écran), la plaque
  à midi ; `510-essai-goutte-c` (1107) : posé au bord de la tour C, 3/15 ;
- `510-essai-arrivee` (1191) : Pépin au drapeau ;
- `510-fin` (60 images après la fin du script) : « LE SOLEIL ! », 0:12,3, 3 gouttes (3/15 :
  celles de l'essai), 0 mort ;
- `510-tour-00` à `05` (811 à 1311, toutes les 100 images) : série « attente » du pilote, le
  robot lâché à l'image 711 sur la grande aiguille : Pépin ne touche plus à rien et l'aiguille
  l'emporte sur cinq sixièmes de tour (11 h, 1 h, 3 h, 5 h, 7 h, 9 h) ; il est debout sur la
  plaque sur les six images, la tige suit la plaque autour du cadran.
Chaque capture regardée ; `docs/mesure-capture.py <capture> 3` : 0 pixel hors palette,
0 bloc flou, sur les 21.

Vérification sur une extraction propre de fe90c64 (dernier commit de code ; `git archive`,
nouveau dossier daté) : `./docker-cargo.sh test --workspace --no-default-features` :
307 + 272 tests, tous verts (1 + 15 ignorés : écritures de fichiers et diagnostics) ;
`./docker-cargo.sh lint` : `LINT-OK` ; `./build.sh` : `dist/` produit, 0 occurrence des noms
interdits, seule URL `http://www.w3.org/2000/svg` ; `./build.sh --qa` : `dist-qa/` produit
(« Essai horloge » : 0 fois dans `dist/game.wasm`, 1 fois dans `dist-qa/game.wasm`) ; planche
de sprites régénérée identique à l'octet.

Limites (non testé ou à relativiser) :
- Aucun niveau du jeu n'a encore d'aiguille (le clocher viendra ensuite) : elles ne se voient
  que dans le niveau d'essai (touche F8 du build de QA).
- Le parseur vérifie le cercle contre les TUILES seulement. Une aiguille qui descend sur Pépin
  debout ailleurs le « ramasse » (la collision le pose sur la plaque), comme les plateformes
  mobiles existantes : au dessin du clocher, garder le dessous des cercles loin des endroits où
  Pépin se tient. Le cadran (dessin seul) recouvre les tuiles à moins d'une tuile du pivot :
  voulu pour une horloge posée sur un mur de tour, à garder en tête.
- Le robot de faisabilité ne juge pas le timing (douze places immobiles). Seul le robot de QA
  (vraie simulation, plans tenus à 2 images près) prouve qu'on peut attraper les aiguilles ; il
  faudra garder les deux verts sur le clocher. La règle du décalage allonge la recherche dans
  un niveau à aiguilles (chaque plan gardé est rejoué 4 fois de plus) : sur l'essai, le script
  du robot passe d'environ 16 s à 37 s.
- Le sinus « maison » est identique au bit près en natif et en wasm par construction (opérations
  de base IEEE seulement) ; pas vérifié bit à bit dans le navigateur, mais le rejeu suit la
  trace du robot (Pépin porté par les deux aiguilles, 3 gouttes, 0 mort).
- Pas de son propre (pas de tic-tac). Captures dans Chromium seulement, à 960 x 540 ; pas de
  vrai téléphone, pas de Safari ni Firefox.

## Grand Nuage, le boss (carte 5.11)

Le Grand Nuage (`O`, lettre réservée de la carte), l'avant-goût de l'œil du cyclone : un boss
d'arène, un écran, sans drapeau. Trois points de vie ; trois motifs d'éclairs de plus en plus
serrés sur 17 colonnes (8 de part et d'autre de lui), chaque colonne annoncée comme un éclair
`e` existant (48 pas d'annonce, 12 de frappe) ; après chaque motif il s'essouffle (5 s) et
libère un courant ascendant temporaire, à gauche après les motifs 1 et 3, à droite après le
motif 2 ; Pépin y monte et lui saute sur la tête (un saut = 1 point) ; à 0 point, le ciel se
déchire (2 s) et le niveau est fini. Mort de Pépin : retour au prélude (1,5 s de grondement),
le nuage garde ses points perdus.

Logique PURE dans `core/src/entities/nuage.rs` : machine à états en entiers (pas, phases
`Prélude`, `Colere`, `Essouffle`, `Touche`, `Dechirure`, `Vaincu`), testée pas à pas
(`le_combat_pas_a_pas` : motifs de 4 à 5 s, de plus en plus serrés, annonce/frappe de 48+12
pas, essoufflement manqué = le même motif recommence, la tête compte seulement pendant
l'essoufflement, une mort rend au prélude sans rendre les points) ; la tête est un cercle
(collision comme les plateformes), les éclairs tuent pendant la frappe, la protection après
une mort porte aussi le nuage (`apres_une_mort_le_nuage_reprend_avec_ses_points_perdus`).
Contacts, score (éclat et secousse du coup, tonnerre du grondement), `Niveau::arrivee` devenu
une `Option` (l'arène n'a pas d'`X`), erreurs typées (une `O` sans sa place de 3 x 2, deux
`O`, un `X` dans l'arène du boss).

Le niveau d'essai « nuage » (`core/src/levels/essai_nuage.rs`) : 20 x 11 tuiles, un écran,
thème Orage (le violet de l'œil du cyclone), murs à gauche et à droite (les 17 colonnes
d'éclairs n'ont pas de coin à l'abri), le nuage au milieu (tête 80 px au-dessus du sol, hors
de portée d'un saut), trois gouttes (au sol, et une en l'air entre chaque courant et le
nuage), une lanterne au départ. Hors de la table `NIVEAUX`, il n'existe que dans les tests et
le build de QA : touche **F10** (`F9` reste au plantage volontaire), carte et intro à part.

Le robot de faisabilité (`core/src/zones.rs`) : `niveau_d_essai_nuage_faisable` et
`niveau_d_essai_nuage_chaque_courant_mene_a_la_tete`, le nuage est jugé essoufflé, ses deux
courants actifs, la tête compte comme l'arrivée ; chacun des deux courants mène à la tête,
aucun sans courant. Le robot de QA (`game/src/bot_qa.rs`) a appris à le battre :
`le_robot_bat_le_grand_nuage` (éclairement du combat : fin atteinte, 0 mort, 3 gouttes,
trois coups pendant l'essoufflement à 3, 2 puis 1 point, les trois motifs joués, déchirure de
2 s, vaincu) et `le_parcours_du_grand_nuage_tient_a_une_image_pres` (le monde une ou deux
images en avance ou en retard : nuage vaincu, 0 mort, 3 gouttes à chaque fois). Le script
ignoré `ecrire_script_essai_nuage` écrit `target/qa/script-511.txt`, `touches-511.txt`
(F10) et `trace-511.csv` (phase du nuage, essoufflement restant, points, coups).

Rendu (`game/src/`) : sprites 48 x 32 (colère x2 : sourcils froncés, yeux d'or, dents
serrées ; essoufflé x2 : paupières lourdes, bouche qui halète, goutte de sueur ; touché :
yeux plissés, il clignote ; vaincu : affaissé, yeux fermés), ventre violet d'orage, contour
sombre, lisible sur tous les ciels ; ses 17 colonnes d'éclairs sont des éclairs `e` (même
annonce, même frappe) ; il gronde au prélude (tremblement), le ciel se déchire derrière lui
(fente d'or pâle qui s'ouvre jusqu'à remplir l'écran, bande au milieu) ; ses points de vie
dans le HUD (trois petits nuages, en haut au milieu : éteints un à un au fil des coups).

Mesure de difficulté : inchangée, aucun niveau du jeu, aucun ennemi, aucune mécanique de
niveau n'a été modifié (l'arène du nuage n'est pas un niveau du jeu, elle n'est compilée que
dans le build de QA) ; `docs/mesure-difficulte.txt` régénéré sur l'extraction propre est
identique à l'octet à la version sous dépôt (même tableau que la carte 5.10 : toits 0,4 /
0,1 / 0,0, gouttières 1,5 / 0,5 / 0,2, grand orage 3,0 / 0,6 / 0,4, vents d'altitude
1,3 / 0,4 / 0,1, au-dessus des nuages 0,5 / 0,1 / 0,0).

Captures (`docs/qa/511-*`, 960 x 540, build de QA d'une extraction propre de b482998 servi en
local sur 127.0.0.1, Chromium piloté par CDP, onglet au premier plan et canvas focalisé,
pilote `docs/capture-visibilite.mjs` (protocole `docs/pilote-qa.js`) avec
`SCRIPT=target/qa/script-511.txt` et `TOUCHES=target/qa/touches-511.txt` (F10 au pilote,
image 229), serveur arrêté à la fin ; numéros d'image de `trace-511.csv` (1877 images,
écrits par le test ignoré `ecrire_script_essai_nuage`) :
- `511-carte` (330) : carte « ESSAI NUAGE », le texte et l'astuce dans le cadre ;
- `511-essai-depart` (470) : l'arène au prélude, Pépin au départ près de la lanterne, la
  goutte du sol, le nuage qui gronde (sourcils froncés, yeux d'or, dents serrées), 3
  points de vie en haut ;
- `511-colere-a` (545), `511-colere-b` (557) : colère, les deux images de l'animation, les
  colonnes d'éclairs annoncées ;
- `511-eclair-frappe-1` (596) : la frappe des éclairs (colonnes d'éclairs `e`), Pépin au
  sol ;
- `511-essouffle-a` (785), `511-essouffle-b` (800) : essoufflé (deux images), le courant
  ascendant de gauche visible (particules), la goutte en l'air, Pépin porté ;
- `511-saut-tete` (838) : Pépin au-dessus de la tête, avant le coup ;
- `511-coup-1` (845) : touché (yeux plissés), 2 points de vie ;
- `511-motif-2` (1000) : deuxième motif, 2 points ;
- `511-coup-2` (1226) : deuxième coup, 1 point, le courant à droite ;
- `511-motif-3` (1370) : troisième motif (le plus serré), 1 point ;
- `511-essouffle-3` (1595) : troisième essoufflement, le courant à gauche, la 3e goutte
  prise ;
- `511-coup-3` (1734) : troisième coup, 0 point, la déchirure commence (fente d'or pâle) ;
- `511-dechirure` (1800) : le ciel déchiré, la bande dorée au milieu ;
- `511-vaincu` (1860) : le nuage vaincu (affaissé, yeux fermés, éteint) ;
- `511-fin` (1876) : écran de fin, 0:36,6, 3 gouttes, 0 mort.
Chaque capture regardée ; `docs/mesure-capture.py <capture> 3` : 0 pixel hors palette,
0 bloc flou, sur les 17.

Vérification sur une extraction propre de b482998 (dernier commit de code ; `git archive`,
nouveau dossier daté) : `./docker-cargo.sh test --workspace --no-default-features` :
326 + 282 tests, tous verts (1 + 16 ignorés : écritures de fichiers, scripts, diagnostics) ;
`./docker-cargo.sh lint` : `LINT-OK` ; `./build.sh` : `dist/` produit (473 Ko de wasm),
0 occurrence des noms interdits, seule URL `http://www.w3.org/2000/svg` ; `./build.sh --qa` :
`dist-qa/` produit (« Grand Nuage » : 0 fois dans `dist/game.wasm`, 1 fois dans
`dist-qa/game.wasm` ; idem « Essai nuage ») ; planche de sprites et mesure de difficulté
régénérées, identiques à l'octet.

Limites (non testé ou à relativiser) :
- Le Grand Nuage n'est encore dans aucun niveau du jeu (l'œil du cyclone, niveau 6, viendra
  avec la suite de l'étape 5) : il ne se voit que dans l'arène d'essai (touche F10 du build
  de QA).
- Le robot de faisabilité ne juge pas le timing du combat (la tête compte comme l'arrivée
  pendant un essoufflement statique) : seul le robot de QA (vraie simulation, coups pendant
  l'essoufflement, plans tenus à 2 images près) prouve qu'on peut le battre ; à garder vert
  sur l'œil du cyclone.
- Captures dans un Chromium piloté par CDP seulement, à 960 x 540 ; pas de vrai téléphone,
  pas de Safari ni Firefox. Pas de son entendu à l'oreille (le grondement et les éclats ne
  sont vérifiés que par les tests d'audio).

## Refactor du robot de QA (carte 5.11b)

Refactorisation PURE de `game/src/bot_qa.rs` (2 748 lignes) : aucun niveau,
aucun ennemi, aucune mécanique modifiés, seuls des fichiers de code ont bougé.
Le module est découpé en `game/src/bot_qa/` (cinq fichiers, aucun ne dépasse
~800 lignes, chacun avec une documentation de module en français : rôle, ce qui
va où, comment ajouter une nouvelle mécanique ou un niveau d'essai). La carte
proposait quatre fichiers ; les tests des essais (676 lignes) ne pouvaient pas
rester dans `essais.rs` sous la limite de ~800 lignes, d'où `essais_tests.rs`.

- `mod.rs` (335) : le rejouage complet à travers l'`Application` (`Trame`,
  `Partie`, `jouer_tout`, `tous_les_parcours`, `empreinte` FNV-1a), le test
  « le robot finit tous les niveaux sans mourir » et le test ignore
  `ecrire_script_navigateur` (script de tous les niveaux, `script.txt`/`trace.csv`).
- `parcours.rs` (798) : le « cerveau » du robot, plans et simulation sur copie
  du monde (`Plan`, `plans`, `simuler`), les cases du niveau et la recherche
  (`cle`, `chercher`, Dijkstra), la tenue au décalage (`tient_au_decalage`).
  **`CaseParcours` est maintenant un `struct` à champs nommés et commentés**
  (au lieu d'un 7-uple) : `x`, `y`, `lance`, `escargot`, `chats`, `aiguille`,
  `nuage` ; `Ord` dérivé dans l'ordre des champs, soit l'ordre EXACT de
  l'ancien 7-uple (la mesure de difficulté trie ses cases dans un tas, et un
  ordre changé fausserait les comparaisons). `game/src/difficulte.rs` est
  adapté (quatre lignes) sans changer de comportement ; `MARGE_CHAT_PAS` passe
  `pub(crate)` (les tests y vont par `bot_qa::parcours`).
- `essais.rs` (625) : les parties des niveaux d'essai, la fonction GÉNÉRIQUE
  `jouer_essai(indice, raccourci, parcours)` est paramétrée par le niveau et sa
  touche raccourci ; les dix `jouer_essai_*` sont chacun une ligne (doc
  comprise), `ImageEssai`/`PartieEssai` et les parcours de référence
  (`parcours_essai_*`) sont inchangés.
- `essais_tests.rs` (676) : les tests des niveaux d'essai (le robot finit sans
  mourir ; les parcours à horloge tiennent à une image près ; la démo casette).
- `scripts.rs` (517) : les onze tests ignore `ecrire_script_essai_*` qui
  écrivent `target/qa/*` (script, touches, trace) pour le rejeu navigateur.

Preuves (sur l'extraction propre du commit final, `git archive`, nouveau
dossier daté) :
- `./docker-cargo.sh test --workspace --no-default-features` : 326 (core) +
  282 (game) tests, tous verts, MEME nombre de tests qu'avant (1 + 16 ignorés :
  écritures de scripts, mesure, diagnostics).
- `./docker-cargo.sh lint` : `LINT-OK`.
- Mesure de difficulté : `docs/mesure-difficulte.txt` régénéré, identique à
  l'octet au fichier commité (même tableau, même graine).
- Les 35 scripts/touches/traces régénérés par la commande `-- --ignored ecrire_script`
  (les onze essais `52-`…`511-`, `script.txt`/`trace.csv` de tous les niveaux,
  `script-31b.txt` de la mesure) sont tous `cmp`-identiques à la régénération
  faite sur l'extraction AVANT la refactorisation ; l'empreinte du robot
  principal est inchangée : `e03e19a27c17c283` (`ecrire_script_navigateur`,
  12 131 images, 0 mort, 5 gouttes). Deux autres fichiers de `target/qa/`
  (`script-31.txt`, `trace-57-rapide.csv`) viennent de cartes antérieures, ne
  sont plus régénérés par un test courant (fichiers de travail de cartes
  antérieures) et ne sont donc pas dans une extraction propre, sans
  importance : aucun code de production n'a changé.
- Wasm de production : IDENTIQUE à l'octet (473 390 octets, SHA-256
  `505b3a4c89e0c…`, delta 0 %, bien dans ±1 %), le robot et `difficulte.rs`
  sont compilés seulement sous `#[cfg(test)]`, ils n'entrent pas dans le
  binaire de production ; `dist/` : 0 occurrence des noms interdits, seule URL
  `http://www.w3.org/2000/svg`.

Limites (non testé ou à relativiser) :
- Aucun comportement n'a changé (preuve ci-dessus) : pas de captures à
  refaire, pas de vrai téléphone, pas de Safari, pas de son à l'oreille.
- La mesure « identique à l'octet » couvre les cinq niveaux officiels ; les
  niveaux d'essai ne font pas partie de la mesure officielle.

## Le marché couvert, niveau 3 (carte 5.12)

« Sous les toiles, tout rebondit. » Premier des nouveaux niveaux de l'étape 5, inséré à sa
place (3) entre les Gouttières et l'Orage : le grand orage devient le niveau 4, Vents d'altitude
le 5, Au-dessus des nuages le 6 (« NIVEAU 3/6 »). Grille `core/src/levels/marche.rs`
(153 x 11 tuiles, lettres `A` et `z` des cartes 5.5 et 5.9, aucune lettre nouvelle). Nouveautés
du niveau : l'auvent (et le chat endormi, qui garde la goutte cachée).

Le parcours :
- départ sur un toit avec un chat endormi (on l'écrase ou on le saute) ; un premier auvent au
  BORD du toit : en marchant droit devant, Pépin tombe dessus et rebondit, la première goutte
  est sur la trajectoire du rebond ;
- un toit gardé par une flaque, puis une paire d'auvents au-dessus de la RUE du marché
  (rangée 10, deux tuiles sous les toits : on y tombe sans mourir, on remonte d'un saut) ;
- la goutte CACHÉE : un bloc de boutique, une fente d'une tuile où dort un chat, un mur dont le
  pied est creusé d'un passage en cul-de-sac ; le chemin principal passe par-dessus (bloc puis
  mur) ; la goutte est au fond du passage, derrière le chat : on se laisse tomber dans la fente
  pendant qu'il dort, on l'écrase, on prend la goutte et on remonte la fente d'un saut ;
- le MOMENT FORT : six auvents à la suite au-dessus d'un vide de 23 tuiles (colonnes 85 à 105),
  tous espacés de 4 tuiles (sans rien toucher, chaque rebond retombe pile sur le suivant), la
  deuxième goutte en l'air sur le rebond du troisième ;
- un chat au pied d'un bloc (on le saute en montant sur le bloc), une flaque, une dernière
  paire d'auvents au-dessus de la rue, le drapeau.
4 lanternes (3, 50, 73, 110), toutes posées sur un toit et loin des chats et des flaques ;
décor : cheminées et antennes, AUCUNE manche à air.

Note du coordinateur (l'auvent seul au-dessus du vide se lisait comme une manche à air) :
- chaque auvent est accroché à une boutique. Au bord d'un toit, le toit est sa façade ; au-dessus
  du vide, une FAÇADE DE BOUTIQUE est dessinée dans la colonne à gauche du mât
  (`scene::facade_d_auvent`, dessin seulement, aucune collision, physique de l'auvent
  inchangée) : la vitrine (16 x 20, corniche au sommet du mât, deux carreaux orange, étalages en
  silhouette) puis le mur (16 x 16, bleu nuit, fenêtres éteintes, sans briques : un fond, pas un
  mur où se poser) jusqu'au premier solide ou au bas du niveau. Elle vaut pour TOUS les auvents,
  donc aussi pour ceux du niveau d'essai « auvent » (F3). Tests :
  `chaque_auvent_est_accroche_a_une_boutique` (12 façades : 7 au marché, 5 à l'essai ; les
  autres auvents sont au bord d'un toit) et `la_facade_de_boutique_a_sa_taille_et_ses_couleurs`
  (ni or, ni cyan, ni corail : aucune couleur qui a un sens dans le jeu) ;
- aucune manche à air dans le niveau (test `chaque_niveau_a_son_decor` : cheminées et antennes
  seulement) ;
- capture côte à côte `512-auvent-ou-manche-a-air` (ci-dessous).

Ambiance : un thème de plus, `Theme::Marche` (`decor::ambiance`) : nuit bleue qui s'éclaircit
vers l'horizon (ardoise ombre, nuit, ardoise, ardoise claire), étoiles en haut, immeubles bleu
nuit en parallaxe avec quelques fenêtres allumées orange et corail (l'or reste réservé aux
gouttes), pluie de 36 gouttes (entre l'averse et l'orage), tuiles des gouttières. Les tests du
décor passent sur les six thèmes (dont : aucune silhouette de la couleur du ciel derrière elle,
fond jamais plat). Carte de titre : « LE MARCHÉ COUVERT », la phrase d'AGA, le conseil
« Tombe sur un auvent : il te relance. Un chat ne s'écrase qu'endormi. »

Robots :
- faisabilité (`core/src/zones.rs`) : `niveau_3_faisable` (arrivée, 3 gouttes, 4 lanternes,
  robot maladroit), `reprise_depuis_chaque_lanterne`, `niveau_3_le_marche_exige_ses_six_auvents`
  (sans les six auvents du moment fort, le vide est infranchissable),
  `niveau_3_le_chat_de_la_fente_garde_la_goutte_cachee` (fente bouchée : arrivée atteinte, goutte
  cachée non) et `niveau_3_on_ressort_de_la_cachette_du_chat` (posé sur la goutte cachée, chat
  écrasé : le drapeau reste atteignable, aucun piège sans retour) ; lint de grille vert (aucune
  lanterne qui flotte, aucune poche de vide close) ;
- robot de QA : `le_robot_finit_tous_les_niveaux_sans_mourir` vert sur les six niveaux. Nouvelle
  empreinte de la partie complète (un niveau de plus, donc attendue) : `5594ca471d004d32`,
  13 878 images, 0 mort, 7 gouttes, score 1000, chrono 12 246 pas. Le marché : images 5076 à
  6618 du script. Le robot ne va pas chercher la goutte cachée (il ne vise pas toutes les gouttes) :
  la preuve qu'on l'atteint et qu'on en ressort est celle du robot de faisabilité.
Aucune mécanique ne s'est mal comportée : rien n'a été corrigé dans l'auvent, le chat ni la flaque.

Renumérotation (le niveau inséré décale les suivants) : tests qui nommaient un niveau par sa
place, lanterne de l'orage sur la patrouille d'une flaque désormais (4, 145), pigeons rangés par
NOM de niveau (Vents d'altitude, Au-dessus des nuages), `niveau_5_exige_les_courants_ascendants`,
étiquette « NIVEAU 2/6 », coupures de la carte de l'orage et du conseil final, juge de la courbe
de difficulté (constantes `SOMMET`, `APRES_SOMMET`, `FINAL`, l'orage, les vents, les nuages,
vérifiées par leur nom) ; README : six niveaux.

Mesure de difficulté (`docs/mesure-difficulte.txt`, graine 20261113, 48 essais) :

| Niveau                      | Avant (5.11) D / H / E | Après (5.12) D / H / E |
| --------------------------- | ---------------------- | ---------------------- |
| 1. Les toits                | 0,4 / 0,1 / 0,0        | 0,4 / 0,1 / 0,0        |
| 2. Gouttières et cheminées  | 1,5 / 0,5 / 0,2        | 1,5 / 0,5 / 0,2        |
| 3. Le marché couvert        |,                      | 0,8 / 0,5 / 0,2        |
| 4. Le grand orage (ex-3)    | 3,0 / 0,6 / 0,4        | 3,0 / 0,6 / 0,4        |
| 5. Vents d'altitude (ex-4)  | 1,3 / 0,4 / 0,1        | 1,3 / 0,4 / 0,1        |
| 6. Au-dessus des nuages     | 0,5 / 0,1 / 0,0        | 0,5 / 0,1 / 0,0        |
| Total                       | 6,8 / 1,8 / 0,7        | 7,6 / 2,3 / 0,9        |

Le marché se place entre les Gouttières et l'Orage (Débutant 0,8 < 3,0 ; Habitué 0,5, bande
admise [0,3 ; 1,2[), l'orage reste le sommet. Ses lieux de mort : les chutes du moment fort
(colonnes 87 à 104) et les chats (colonnes 11 et 113 : réveillés, ils tuent). Les cinq autres niveaux sont
identiques LIGNE À LIGNE à la mesure d'avant (renumérotés ; seule la ligne Total change).
Deux retouches pour y arriver :
- première version (auvents espacés de 4, 4, 5, 3 et 4 tuiles comme l'essai, vide sous toutes les
  paires) : Habitué 1,0, au-dessus de l'orage (0,7), chutes au 5 et au 3 du moment fort et sous
  les paires d'auvents. D'où la chaîne régulière (tout de 4) et la rue sous les paires ;
- la graine d'un essai dépendait de la PLACE du niveau dans la table : insérer le marché changeait
  les tirages (donc les chiffres, au bruit près) des trois niveaux suivants sans qu'aucun ne change
  (orage mesuré 0,7 au lieu de 0,6). `difficulte::RANGS_DE_GRAINE` donne désormais à chaque niveau
  un rang d'ARRIVÉE (les cinq anciens gardent leur ancienne place, le marché est le 6e ; test
  `chaque_niveau_a_un_rang_de_graine`) : un niveau inséré plus tard ne déplacera plus la mesure des
  autres. `LIMITE_CASES` (garde-fou des itinéraires de la mesure) passe de 6 000 à 20 000 : le
  marché (quatre chats, dont l'état compte dans la case) en explore plus que les autres.

Captures (`docs/qa/512-*`, 960 x 540, build de QA d'une extraction propre de 22c6955 servi en
local sur 127.0.0.1, navigateur Chromium sans tête (CDP) du poste, onglet au premier plan
(`Page.bringToFront`) et canvas focalisé, protocole `docs/pilote-qa.js` (horloge virtuelle) avec
le script du robot `target/qa/script.txt` régénéré sur la même extraction, serveur arrêté à la
fin ; numéros d'image de `trace.csv`) :
- `512-carte` (5000) : « NIVEAU 3/6 », « LE MARCHÉ COUVERT », la phrase d'AGA, le conseil ;
- `512-depart` (5110) : Pépin au départ, lanterne, cheminée, chat endormi (« z »), ciel étoilé ;
- `512-premier-auvent` (5352) : le rebond sur le premier auvent (toile enfoncée) au bord du toit,
  la goutte sur la trajectoire, la flaque du toit suivant, le chat du départ qui se réveille (« ! ») ;
- `512-fente` (5792) : le bloc, la fente où dort le chat, le passage et la goutte cachée au fond,
  Pépin passe au-dessus ;
- `512-moment-fort-a` (6068), `-b` (6112), `-c` (6150) : la chaîne des six auvents, chacun devant
  sa façade de boutique (le premier au bord du toit, sous l'antenne), Pépin en l'air de rebond en
  rebond, la goutte en l'air puis prise (3/18 puis 4/18), le toit suivant et sa lanterne en vue ;
- `512-arrivee` (6588), `512-arrivee-drapeau` (6600) : la dernière paire d'auvents au-dessus de
  la rue (façade posée sur la rue), Pépin plane jusqu'au drapeau, la transition commence ;
- `512-vents-manche-a-air` (9480) : la manche à air de Vents d'altitude (niveau 5) ;
- `512-auvent-ou-manche-a-air` : côte à côte, SANS AUCUN pixel recalculé, la moitié gauche de
  `512-moment-fort-a` (colonnes 201 à 680) et la moitié droite de `512-vents-manche-a-air`
  (colonnes 330 à 806), séparées d'une colonne de 3 px d'ardoise ombre, découpes alignées sur la
  grille de 3 px. On ne les confond pas : l'auvent est une toile rayée corail et blanc, EN PENTE
  vers le bas, sur un mât court fixé au bord d'un toit ou à une vitrine allumée ; la manche à air
  est un tube orange et blanc, à l'HORIZONTALE, au bout d'un grand mât planté seul sur un toit,
  sans boutique.
Chaque capture regardée ; `docs/mesure-capture.py <capture> 3` : 0 pixel hors palette, 0 bloc
flou, sur les 11.

Vérification sur une extraction propre de 9626e51 (dernier commit de code ; `git archive`,
nouveau dossier daté) : `./docker-cargo.sh test --workspace --no-default-features` :
330 + 285 tests, tous verts (1 + 16 ignorés : écritures de fichiers, scripts, diagnostics) ;
`./docker-cargo.sh lint` : `LINT-OK` ; script du robot régénéré (empreinte `5594ca471d004d32`,
0 mort) ; mesure de difficulté et planche de sprites régénérées : identiques à l'octet aux
versions sous dépôt ; `./build.sh` : `dist/` produit (477 110 octets de wasm, + 3 720 octets,
+0,8 %, pour le niveau, son thème et la façade), 0 occurrence des noms interdits, seule URL
`http://www.w3.org/2000/svg` ; `./build.sh --qa` : `dist-qa/` produit ; les deux wasm sont
identiques à l'octet à ceux de l'extraction de 22c6955 qui a servi aux captures (9626e51 n'ajoute
qu'un test).

Limites (non testé ou à relativiser) :
- Pas de vrai téléphone, ni Safari ni Firefox : Chromium sans tête seulement, à 960 x 540. Pas de
  son entendu à l'oreille (rebonds, chats : vérifiés par les tests d'audio existants).
- La goutte cachée n'est prise par aucun robot qui joue vraiment (le robot de QA l'ignore) : seul
  le robot de faisabilité prouve qu'on l'atteint, qu'il faut écraser le chat endormi et qu'on en
  ressort ; le minutage réel de la descente dans la fente (chat endormi 4 s sur 6) n'est pas
  mesuré par un humain.
- La mesure n'a qu'une graine (bruit de l'ordre de 0,1 à 0,2) ; le marché à 0,5 (Habitué) est
  proche des Gouttières : la courbe complète sur 11 niveaux reste à juger (carte 5.18).
- La façade se dessine pour TOUS les auvents posés au-dessus du vide : le niveau d'essai
  « auvent » (F3) en a gagné cinq (vérifié par test, pas recapturé).

## Le clocher, niveau 5 (carte 5.13)

« Une seule direction : en haut. » Deuxième des nouveaux niveaux de l'étape 5, inséré à sa place
(5) après l'Orage : Vents d'altitude devient le niveau 6, Au-dessus des nuages le 7 (« NIVEAU
5/7 »). Grille `core/src/levels/clocher.rs` (34 x 45 tuiles : le premier niveau VERTICAL du jeu,
quatre écrans de haut, la caméra suit en hauteur depuis la carte 5.2). Lettres `p`, `w`, `s`, `h`
des cartes 4.4, 5.6, 5.8 et 5.10, et `^` ; aucune lettre nouvelle. Nouveauté du niveau : la
verticale (et la cage de l'horloge, son moment fort).

Le parcours, de bas en haut, autour de la tour du clocher (colonnes 14 à 18) :
- au pied de la tour, un escargot patrouille le sol : son rebond seul porte sur le socle (80 px plus
  haut, plus qu'un saut) ; sur le socle, un second escargot, dont le rebond porte à la première
  goutte, suspendue en l'air ;
- une marche, l'arche du bas à travers la tour, la galerie de droite : un pigeon guette sur une
  corniche isolée, tourné vers l'arche ; on passe dessous sans s'arrêter, il plonge derrière Pépin ;
- au bout de la galerie, le ventilateur (2 s de souffle, 2 s d'arrêt annoncées) porte Pépin huit
  rangées plus haut, jusqu'à la première corniche du zigzag ;
- le zigzag : quatre corniches à trois rangées l'une de l'autre, alternées contre la tour et contre
  le mur ; au bout de la première, la deuxième goutte, sous un second pigeon tourné vers Pépin qui
  arrive (s'arrêter sur sa diagonale pour préparer le saut suivant est le vrai danger) ;
- la porte de la cage, puis le MOMENT FORT, la cage de l'horloge : on saute sur la grande aiguille
  quand elle passe au plus bas, elle emporte Pépin jusqu'au sommet du pilier (troisième goutte) ;
  du bord du pilier, la petite aiguille, puis la tour du drapeau. Entre le pilier et la tour, sous
  la petite aiguille, une poutre hérissée de pointes anti-pigeons : là, et là seulement, tomber tue.
La cage reprend la géométrie du niveau d'essai « horloge » (mêmes rayons, vitesses, heures de
départ, mêmes écarts entre pivots, pilier et tour). Ailleurs, le bas de la carte est plein : une
chute ne coûte que de la hauteur. 3 lanternes, toutes là où l'on passe forcément (après l'arche du
bas, à la sortie de la porte de la cage, au bord du pilier où l'on attend la petite aiguille) ;
décor : trois paratonnerres (cage, galerie, tour du drapeau).

Ambiance : un thème de plus, `Theme::Clocher` (`decor::ambiance`) : nuit violette après l'orage
(bandes ardoise nuit, violet, violet, et une lueur pâle à l'horizon), étoiles en haut, immeubles
aux fenêtres orange et corail, petite pluie (16 gouttes), briques violettes. Note du coordinateur
(sentir la hauteur au sommet) : nouveau champ `Ambiance::vertige`, multiplicateur de la parallaxe
verticale des silhouettes, à 2,5 pour le clocher et 1,0 pour tous les autres thèmes (rendu des
niveaux d'un écran inchangé ; test `au_sommet_du_clocher_la_ville_s_est_enfoncee` : au sommet, la
couche proche s'est enfoncée de 135 px et la lointaine de 68 px, contre 54 et 27 au niveau d'essai
vertical). Sur les captures : au départ, la ville monte jusqu'au milieu de l'écran ; au zigzag, la
lueur pâle apparaît derrière ; dans la cage et à l'arrivée, il ne reste qu'une bande d'immeubles
au bas de l'écran sous un grand ciel violet. Carte de titre : « LE CLOCHER », la phrase d'AGA, le
conseil « L'escargot est un tremplin. Sur une aiguille, laisse-toi porter. »

Robots et tests :
- faisabilité : `niveau_5_faisable` (arrivée, 3 gouttes, 3 lanternes, robot maladroit ; course
  rapide 16,9 s), `reprise_depuis_chaque_lanterne`, et dans `clocher.rs` :
  `le_clocher_exige_ses_escargots` (sans escargot, Pépin reste au pied de la tour ; sans celui du
  socle, la goutte en l'air n'est pas prise), `le_clocher_exige_le_ventilateur` (sans courant, la
  galerie est un cul-de-sac), `le_clocher_exige_ses_deux_aiguilles` (sans aiguille, ni pilier ni
  drapeau ; sans la petite, le pilier mais pas le drapeau), `aucune_aiguille_ne_ramasse_pepin_debout`
  (note de la carte 5.10 : sur un tour complet, image par image, aucune plaque ne passe dans les
  14 px au-dessus d'un dessus de tuile) et `les_pieces_du_clocher_a_leur_place` ; lint de grille
  vert (aucune lanterne qui flotte, aucune poche de vide close, décor posé sur un solide) ;
- robot de QA : `le_robot_finit_tous_les_niveaux_sans_mourir` vert sur les sept niveaux (aucune
  mécanique à lui apprendre : escargot, ventilateur, pigeon et aiguilles le sont déjà) ; nouveau test
  `au_clocher_le_parcours_allume_les_trois_lanternes`. Nouvelle empreinte de la partie complète (un
  niveau de plus, donc attendue) : `38e5db5b62bbed02`, 16 255 images, 0 mort, 9 gouttes, score 1200,
  chrono 14 395 pas ; le clocher : images 9449 à 11621 du script.
Aucune mécanique ne s'est mal comportée : rien n'a été corrigé dans l'escargot, le ventilateur, le
pigeon ni les aiguilles.

Renumérotation : faisabilité `niveau_7_faisable`, `niveau_6_exige_les_courants_ascendants`, pigeons
admis dans le clocher (sur les corniches), paratonnerres pour le clocher dans
`chaque_niveau_a_son_decor`, étiquette « NIVEAU 2/7 », conseil final `intro(6)`, densité de pluie
par thème ; la comparaison bit à bit des caméras (`les_niveaux_d_un_ecran_gardent_la_meme_camera`)
saute le clocher, seul niveau que la caméra suit en hauteur ; ombres sous les toits : plus de 50 au
clocher (34 colonnes seulement), plus de 100 ailleurs ; légende du décor (`GenreDecor`) par nom de
niveau ; README : sept niveaux.

Mesure de difficulté (`docs/mesure-difficulte.txt`, graine 20261113, 48 essais) :

| Niveau                      | Avant (5.12) D / H / E | Après (5.13) D / H / E |
| --------------------------- | ---------------------- | ---------------------- |
| 1. Les toits                | 0,4 / 0,1 / 0,0        | 0,4 / 0,1 / 0,0        |
| 2. Gouttières et cheminées  | 1,5 / 0,5 / 0,2        | 1,5 / 0,5 / 0,2        |
| 3. Le marché couvert        | 0,8 / 0,5 / 0,2        | 0,8 / 0,5 / 0,2        |
| 4. Le grand orage           | 3,0 / 0,6 / 0,4        | 3,0 / 0,6 / 0,4        |
| 5. Le clocher               |,                      | 0,6 / 0,3 / 0,0        |
| 6. Vents d'altitude (ex-5)  | 1,3 / 0,4 / 0,1        | 1,3 / 0,4 / 0,1        |
| 7. Au-dessus des nuages     | 0,5 / 0,1 / 0,0        | 0,5 / 0,1 / 0,0        |
| Total                       | 7,6 / 2,3 / 0,9        | 8,2 / 2,6 / 0,9        |

Les six autres niveaux sont identiques LIGNE À LIGNE à la mesure d'avant (rang de graine 7 pour le
clocher, `RANGS_DE_GRAINE`). Lieux de mort du clocher : les pointes de la poutre (colonnes 19 à 28)
et une fois le pigeon de la première corniche (colonne 23). Durée moyenne : 66 s (Débutant), 51 s
(Habitué), 39 s (Expert) : la difficulté du bas de la tour est du TEMPS perdu à retomber, que la
mesure (des morts) ne voit pas. Trois retouches en chemin :
- première version sans pointes : 0,0 / 0,0 / 0,0 (une seule mort en 144 essais, un pigeon) ; d'où
  la poutre aux pointes sous la petite aiguille (comme le gouffre du niveau d'essai horloge) :
  0,8 / 0,1 / 0,0 ;
- vu sur les captures : une chute sur les pointes renvoyait à la galerie, vingt rangées plus bas
  (les lanternes de la cage et du pilier étaient à côté du chemin, jamais touchées) ; lanternes
  déplacées sur le chemin forcé, celle du socle retirée (le rebond du premier escargot porte
  par-dessus, et rien n'y tue) : 0,7 / 0,1 / 0,0 ;
- vu sur les captures : depuis le pilier, la caméra ne montrait pas les pointes (trois rangées sous
  le bas de l'écran) ; poutre remontée de la ligne 14 à la ligne 11, pointes visibles pendant tout le
  passage : 0,6 / 0,3 / 0,0.
Juge de la courbe : bande Habitué du clocher [0 ; 0,5[ ; l'orage reste le sommet (le plus meurtrier
pour le Débutant et l'Habitué), Vents d'altitude reste le niveau jugé « après le sommet » (nettement
plus facile que lui) ; le clocher n'est jugé que par sa bande. À SIGNALER : la décision « monte
jusqu'au boss (6) » voudrait un clocher au moins aussi dur que l'orage ; il se mesure entre les
gouttières et l'orage pour l'Habitué (0,3) et sous le marché pour le Débutant (0,6). Leviers si la
carte 5.18 veut le durcir : des pointes plus bas dans la tour (une chute coûterait une vie, plus
seulement de la hauteur), un troisième pigeon au zigzag, une petite aiguille plus rapide.

Captures (`docs/qa/513-*`, 960 x 540, build de QA d'une extraction propre de 7b65766 servi en local
sur 127.0.0.1, Chromium sans tête (CDP) du poste, onglet au premier plan (`Page.bringToFront`) et
canvas focalisé, pilote `docs/capture-visibilite.mjs` (protocole `docs/pilote-qa.js`, horloge
virtuelle) avec le script du robot régénéré sur la même extraction, serveur arrêté à la fin ;
numéros d'image de `trace.csv`) :
- `513-carte` (9440) : « NIVEAU 5/7 », « LE CLOCHER », la phrase d'AGA, le conseil ;
- `513-depart` (9462) : Pépin au départ à côté du premier escargot, le socle et le second escargot,
  la marche de l'arche, la ville haute derrière ;
- `513-escargot` (9596) : Pépin au sommet du rebond sur l'escargot du socle, la goutte en l'air
  vient d'être prise (6/21), la lanterne de la galerie au loin ;
- `513-galerie` (9790) : Pépin passe sous la corniche, le pigeon plonge DERRIÈRE lui ; la lanterne
  allumée, un paratonnerre, le ventilateur qui clignote (reprise annoncée) ;
- `513-ventilateur` (10060) : Pépin porté par le souffle jusqu'à la corniche du zigzag, la goutte de
  la corniche et le second pigeon en vue ;
- `513-zigzag` (10300) : les corniches alternées, le second pigeon perché sous l'une d'elles ;
- `513-porte` (10600) : sortie de la porte de la cage, la lanterne de la cage, la grande aiguille,
  les pointes de la poutre en haut ;
- `513-moment-fort-a` (10740) : Pépin porté par la grande aiguille (cadran, tige, plaque) ;
- `513-moment-fort-b` (11200) : au bord du pilier, à côté de sa lanterne allumée, Pépin attend la
  petite aiguille ; les pointes au bas de l'écran ;
- `513-moment-fort-c` (11340) : Pépin plane depuis la petite aiguille vers la tour du drapeau,
  au-dessus des pointes ;
- `513-arrivee` (11600) : Pépin au drapeau, paratonnerre, la transition commence ; il ne reste de la
  ville qu'une bande au bas de l'écran ;
- `513-chute-a` (11220), `513-chute-b` (11244) : à l'image 11190 le script est lâché et Pépin court
  tout droit : il tombe du pilier, meurt sur les pointes (étincelles) et réapparaît à la lanterne du
  pilier (étincelles de réapparition, la caméra y revient).
Chaque capture regardée ; `docs/mesure-capture.py <capture> 3` : 0 pixel hors palette, 0 bloc flou,
sur les 13.

Vérification sur une extraction propre de 7b65766 (dernier commit de code ; `git archive`, nouveau
dossier daté) : `./docker-cargo.sh test --workspace --no-default-features` : 336 + 287 tests, tous verts (1 + 16
ignorés : écritures de fichiers, scripts, diagnostics) ; `./docker-cargo.sh lint` : `LINT-OK` ; script du robot régénéré (empreinte `38e5db5b62bbed02`,
0 mort) ; `./build.sh` : `dist/` produit (479 412 octets de wasm, + 2 302 octets, +0,5 %, pour le
niveau et son thème), 0 occurrence des noms interdits, seule URL `http://www.w3.org/2000/svg` ;
`./build.sh --qa` : `dist-qa/` produit (celui des captures). Planche de sprites : aucun sprite
nouveau ni modifié, non régénérée.

Limites (non testé ou à relativiser) :
- Pas de vrai téléphone, ni Safari ni Firefox : Chromium sans tête seulement, à 960 x 540. Pas de
  son entendu à l'oreille.
- La mesure ne compte que les morts : le temps perdu à retomber dans le bas de la tour (66 s en
  moyenne pour le Débutant, la cible est 55 s) n'y apparaît pas. Une seule graine (bruit de l'ordre
  de 0,1 à 0,2).
- Le robot de QA ne prend pas la goutte de la première corniche (il ne vise pas toutes les gouttes)
  et ne passe jamais sur la diagonale du second pigeon : le danger de ce pigeon n'est montré que par
  la mesure (une mort, Débutant) ; la goutte, par le robot de faisabilité.
- Le ressenti de la hauteur (parallaxe ×2,5) n'est jugé que sur captures fixes, pas en mouvement ni
  par un humain.

## Façades du marché couvert sans « dessus » (carte 5.12b, rendu seulement)

Constat du coordinateur sur `512-moment-fort-*` : la façade de boutique dessinée derrière chaque auvent
au-dessus du vide (`scene::facade_d_auvent`) finissait par une corniche CLAIRE et PLATE, 4 px au-dessus
des toits voisins : une tour d'une tuile qui se lisait comme un pilier où atterrir, alors que sa case
est du vide (on traverse, on tombe de 23 tuiles). La physique était juste, la lecture mentait.

Ce qui change (dessin seulement : aucune physique, aucune carte ASCII, aucun auvent modifié) :
- la vitrine (`objets::FACADE_VITRINE`, 16x19 -> 16x15) finit par un toit en APPENTIS adossé au mât :
  le faîte est au sommet du mât, la pente sombre descend vers la gauche d'un pixel toutes les deux
  colonnes, comme la toile descend vers la droite. Avec la toile, c'est un seul toit dont le haut est
  l'auvent, le seul endroit où l'on se pose. Choix : dans ce jeu, tout ce qui porte Pépin est plat avec
  une arête claire ; une pente sombre n'en a ni l'un ni l'autre ;
- plus aucun ton clair : ardoise ombre, ardoise nuit, ardoise (les teintes des immeubles du fond) et
  l'orange de la vitrine (deux carreaux, étalages), gardée pour la lecture « boutique » ; le mur sous
  la vitrine (`FACADE_MUR`) est inchangé ;
- le faîte est posé au VRAI sommet du mât : constante `SOMMET_DU_MAT` (1 px sous le haut de la case),
  partagée par le dessin de l'auvent (image inchangée) et celui de la façade, à la place de
  `DEBORD_VITRINE` (4 px au-dessus). Le faîte est donc 1 px SOUS le dessus des toits voisins. Vu en
  chemin sur les captures avant/après : la première version (7261f85) supposait le mât 3 px au-dessus
  de la case et dressait encore le faîte 4 px au-dessus des toits ; corrigé par 9f938ca ;
- la distinction auvent / manche à air de la carte 5.12 ne bouge pas (toile, mât et manche inchangés).

Tests : `la_facade_n_a_pas_de_dessus_ou_se_poser` (nouveau), l'arête d'un dessus de tuile (rangées 1
et 2 de `toit_haut` et `brique_haut`) n'a que des tons de 3 à 7, et la façade n'en a aucun (seulement
`0 1 2 A`) ; sa rangée supérieure se réduit aux 2 px sombres du faîte contre le mât ; le haut de chaque
colonne monte vers le mât sans redescendre, sans marche, sans palier de plus de 2 px (Pépin en fait
10) et c'est toujours du contour sombre. Vérifié sur l'ancienne vitrine : elle échoue à trois de ces
contrôles (ardoise claire, 3, sur sa corniche ; rangée supérieure de 16 px ; palier de 16 px).
`la_facade_de_boutique_a_sa_taille_et_ses_couleurs`
adapté (16 x (16 - SOMMET_DU_MAT), transparence seulement au-dessus de la pente) ;
`chaque_auvent_est_accroche_a_une_boutique` inchangé et vert (12 façades).

Aucun changement de jeu, prouvé sur une extraction propre de 9f938ca (dernier commit de code) :
- mesure de difficulté régénérée : identique À L'OCTET à `docs/mesure-difficulte.txt` sous dépôt (sha256
  `1cf20d01…`) ; par rapport à celle de 400cad8 (carte 5.12), seules diffèrent les lignes ajoutées par
  la carte 5.13 (le clocher, renumérotation) ; le marché reste 0,8 / 0,5 / 0,2 ;
- script du robot de QA régénéré : 16 255 images, 0 mort, empreinte `38e5db5b62bbed02`, la même qu'à la
  carte 5.13 ; le script est identique à l'octet à celui de 7261f85. (L'empreinte `5594ca471d004d32`
  citée par la carte est celle de 400cad8, à six niveaux : le clocher l'a changée à la carte 5.13.)

Captures (`docs/qa/512b-*`, 960 x 540 ; build de QA d'une extraction propre de ac6372b pour « avant »
et de 9f938ca pour « après », servis sur 127.0.0.1 (ports libres, serveurs arrêtés à la fin), Chromium
sans tête (CDP), onglet au premier plan et canvas focalisé, `docs/capture-visibilite.mjs` avec le MÊME
script du robot ; numéros d'image de `trace.csv`) :
- `512b-moment-fort-a-avant` / `-apres` (6068) : le toit D et l'auvent 85 à son bord, puis les façades
  des auvents 89, 93 et 97 (et le bord de celle de 101) ; Pépin en l'air, la goutte devant lui ;
- `512b-moment-fort-b-avant` / `-apres` (6112) : les cinq façades des auvents au-dessus du vide (89 à
  105) ; avec la précédente, les six auvents du moment fort ;
- `512b-zoom-facade` : découpe 80 x 45 de `-a-apres` agrandie x12 (le bout du toit D, plat à arête
  claire, l'auvent 85, le vide, la façade de l'auvent 89 et sa toile) ;
- `512b-zoom-avant-apres` : la même façade, avant | après, découpes 39 x 45 agrandies x12, séparées de
  24 px d'ardoise ombre.
Zooms : copies de pixels réels, aucun pixel recalculé. Chaque paire avant/après vient de sessions de
rejeu au même cadrage : seuls diffèrent les pixels des façades (746 et 1 117 pixels logiques, dans les
colonnes de 16 px des façades, rangées 128 à 147 de l'écran). `docs/mesure-capture.py <capture> 3` : 0
pixel hors palette, 0 bloc flou, sur les 6. Chaque capture regardée.

Un joueur peut-il encore croire se poser sur la façade ? D'après les captures, c'est peu probable :
avant, chaque façade dressait au-dessus des toits un rebord clair et plat d'une tuile de large, la
même lecture que les toits ; après, il n'y a plus ni plat ni clair, le haut de la façade est plus bas
que les toits voisins et c'est la toile rayée qui le surmonte. La façade reste une colonne sombre d'une
tuile qui descend dans le vide (c'est voulu : la boutique porte l'auvent), mais rien en haut ne dit
« sol ».

Vérification sur l'extraction propre de 9f938ca : `./docker-cargo.sh test --workspace
--no-default-features` : 336 + 288 tests, tous verts (1 + 16 ignorés) ; `./docker-cargo.sh lint` :
`LINT-OK` (la première extraction, 7261f85, avait révélé une liste de commentaire sans ligne vide :
2fe4376) ; `./build.sh` : `dist/` produit (479 431 octets de wasm, + 19 octets), 0 occurrence des noms
interdits, seule URL `http://www.w3.org/2000/svg` ; `./build.sh --qa` : `dist-qa/` produit (celui des
captures « après »). Planche de sprites régénérée par les tests : identique à l'octet à celle committée
(f4e9aa1).

Limites (non testé ou à relativiser) :
- Aucun joueur humain : la lecture « pas un sol » est jugée sur captures fixes, pas en mouvement.
- Si un joueur visait malgré tout la pente : d'après la géométrie (boîte de Pépin de 10 px, zone de
  l'auvent qui commence au mât), si le centre de Pépin arrive sur les 5 px de droite de la façade, sa
  boîte touche déjà l'auvent et il rebondit ; plus à gauche, il tombe. Raisonnement, pas essai en jeu.
- Le rejeu par le pilote n'a pas exactement le même cadrage d'une session à l'autre (Pépin et la
  caméra décalés de 1 à 2 px pour le même build et le même script) : les paires avant/après ont été
  choisies parmi des sessions au même cadrage, vérifié pixel par pixel.
- Pas de vrai téléphone, ni Safari ni Firefox : Chromium sans tête seulement, à 960 x 540. Le niveau
  d'essai « auvent » (F3), qui a aussi cinq façades, n'est pas recapturé (couvert par les tests).

## L'œil du cyclone, niveau 6, le boss (carte 5.14)

« Le Grand Nuage ne veut pas te laisser passer. » Troisième des nouveaux niveaux de l'étape 5, inséré à sa
place (6) après le clocher : Vents d'altitude devient le niveau 7, Au-dessus des nuages le 8 (« NIVEAU
6/8 »). Grille `core/src/levels/cyclone.rs` : 20 x 11 tuiles, un seul écran, murs et sol de nuages (`N`).
Lettre `O` de la carte 5.11 ; aucune lettre nouvelle. Nouveauté du niveau : le Grand Nuage, le boss.

Le combat, celui de la carte 5.11 sans rien changer : le nuage gronde (1,5 s), lance un motif d'éclairs
sur les 17 colonnes de l'arène (chacune annoncée comme un éclair `e`), s'essouffle (5 s) et laisse
échapper un courant à côté de lui (à gauche après les motifs 1 et 3, à droite après le 2) ; Pépin y monte
et lui retombe sur la tête ; trois motifs de plus en plus serrés, trois coups, et le ciel se déchire : le
niveau est fini (pas de drapeau). Une seule lanterne, au début de l'arène (décision du coordinateur : une
mort ramène au début de l'arène, le nuage garde ses points perdus). Trois gouttes : au sol, et deux à la
hauteur de sa tête, entre chaque courant et lui (celle de droite attend le deuxième motif). L'arène
reprend EXACTEMENT la géométrie du niveau d'essai « nuage » (sur laquelle le combat a été réglé et le
robot de QA entraîné) : test `l_arene_du_cyclone_reprend_celle_du_niveau_d_essai` (mêmes cases pleines,
même départ, mêmes entités, même nuage, mêmes colonnes et courants) ; seules les tuiles changent de
dessin. Pas de décor (`H T R Q K`) : ses éclairs couvrent tout le sol, et un objet de décor n'est jamais
devant un danger (carte 4.2).

Ambiance, violet et or : un thème de plus, `Theme::Cyclone` (`decor::ambiance`). En haut, la trouée d'or
(bande orange soleil : le ciel au-dessus de l'œil), bordée d'un plafond de nuages violets et bleu nuit qui
tournent autour ; plus bas, le mur de l'œil, ardoise puis bleu nuit ; des tours de nuages violettes
montent du fond, les proches presque noires ; un peu de pluie (32 gouttes). Choisi sur maquettes avant le
code (six essais : trouée d'or ou ciel violet uni, tours plus ou moins hautes, plafond plus ou moins
épais) : les colonnes annoncées (pointillés d'or) se lisent sur le bleu nuit, la frappe (or pâle et blanc)
sur tout le fond, le nuage (contour sombre, corps clair) sur la trouée. Les tuiles sont les nuages clairs
des niveaux d'altitude (aucun sprite nouveau ni modifié). Carte de titre : « L'ŒIL DU CYCLONE » (ligature
Œ de la police), la phrase d'AGA, le conseil « Quand il s'essouffle, monte dans son vent et saute-lui sur
la tête. » (celui de l'arène d'essai).

Robots et tests :
- faisabilité : `niveau_6_faisable` (l'« arrivée » du robot de faisabilité est la tête du nuage, toujours
  essoufflé, voir `faisabilite` ; 3 gouttes, la lanterne, robot maladroit compris),
  `reprise_depuis_chaque_lanterne`, et ceux du niveau d'essai
  (`niveau_d_essai_nuage_chaque_courant_mene_a_la_tete`) valent pour la même géométrie. Trois règles
  communes ont une exception pour l'arène du boss (niveau sans drapeau), écrite et commentée dans le
  test : une seule lanterne au lieu de 3 au moins (`verifier_infos`) ; la durée (`durees_plausibles`) est
  celle du combat le plus court (grondement, trois motifs, deux « touché », la déchirure : 19,3 s), pas le
  1,2 s que met le robot de faisabilité à atteindre la tête ; pas de décor imposé
  (`le_decor_ne_change_rien_a_la_collision`). Lint de grille vert ;
- robot de QA : `le_robot_finit_tous_les_niveaux_sans_mourir` vert sur les huit niveaux, sans rien lui
  apprendre (il bat déjà le Grand Nuage depuis la carte 5.11). Son parcours du cyclone est identique à
  celui de l'arène d'essai (1 397 entrées) et tient les entrées décalées de -2 à +2 images (vérifié par un
  test de diagnostic temporaire, non gardé). Nouvelle empreinte de la partie complète (un niveau de plus,
  donc attendue) : `31055a8413c4561b`, 17 880 images, 0 mort, 12 gouttes, score 2 100, chrono 15 792 pas ;
  le cyclone : images 11826 à 13246 du script.
Aucune mécanique ne s'est mal comportée : rien n'a été corrigé dans le Grand Nuage.

Le joueur simulé de la mesure, lui, ne savait pas se battre (consigne : lui apprendre la mécanique dans la
même carte). Première mesure : 126,0 / 131,1 / 124,8 morts, 5, 4 et 7 essais finis sur 48, toutes les
morts sous les éclairs. Raisons : la case du robot ne connaît le nuage qu'à 15 pas près, et le conseil
d'une case (calculé pour l'heure exacte où l'exploration y est passée) peut mener sous une colonne à une
autre heure de la même tranche ; après une mort, le joueur reprenait dans des cases que l'exploration
n'avait jamais vues (le nuage au prélude avec des points perdus) ; et, perdu, il attendait sur place, sous
la colonne. Corrigé dans `difficulte.rs`, seulement dans un niveau à Grand Nuage (`Itineraires::nuage` ;
les sept autres niveaux sont identiques ligne à ligne, et la mesure du commit 6b958a5, apprentissage seul,
est identique À L'OCTET à celle d'avant la carte) : les cases de reprise après une mort sont ajoutées à
l'exploration (sans arête : mourir n'est pas un chemin conseillé) ; les tranches de temps voisines
comptent parmi les cases voisines ; un candidat n'est gardé que si la suite des conseils, huit plans plus
loin, ne fait pas mourir (`chaine_sure`), le joueur VOIT les colonnes annoncées ; détour plus long
(`LIMITE_RECHERCHE_NUAGE`) ; et le secours ne reste plus sous une colonne qui va frapper
(`secours_sous_les_eclairs`). Le joueur sans erreur bat le nuage sans mourir (22 s).

Mesure de difficulté (`docs/mesure-difficulte.txt`, graine 20261113, 48 essais) :

| Niveau                      | Avant (5.13) D / H / E | Après (5.14) D / H / E |
| --------------------------- | ---------------------- | ---------------------- |
| 1. Les toits                | 0,4 / 0,1 / 0,0        | 0,4 / 0,1 / 0,0        |
| 2. Gouttières et cheminées  | 1,5 / 0,5 / 0,2        | 1,5 / 0,5 / 0,2        |
| 3. Le marché couvert        | 0,8 / 0,5 / 0,2        | 0,8 / 0,5 / 0,2        |
| 4. Le grand orage           | 3,0 / 0,6 / 0,4        | 3,0 / 0,6 / 0,4        |
| 5. Le clocher               | 0,6 / 0,3 / 0,0        | 0,6 / 0,3 / 0,0        |
| 6. L'œil du cyclone         |,                      | 2,9 / 0,6 / 0,3        |
| 7. Vents d'altitude (ex-6)  | 1,3 / 0,4 / 0,1        | 1,3 / 0,4 / 0,1        |
| 8. Au-dessus des nuages     | 0,5 / 0,1 / 0,0        | 0,5 / 0,1 / 0,0        |
| Total                       | 8,2 / 2,6 / 0,9        | 11,1 / 3,2 / 1,2       |

Les sept autres niveaux sont identiques LIGNE À LIGNE (rang de graine 8 pour le cyclone,
`RANGS_DE_GRAINE`). Dans l'arène : 48/48 essais finis pour chaque profil, en 42 s (Débutant), 27 s
(Habitué), 25 s (Expert) ; toutes les morts sous les éclairs (colonnes 4 à 18), le pire essai à 11 morts
(Débutant). Exactement (morts cumulées / 48) : 2,88 / 0,58 / 0,29, contre 3,04 / 0,65 / 0,35 pour l'orage.
Juge de la courbe : bande Habitué du cyclone [0,3 ; 1,2[ ; l'orage reste le sommet (le plus meurtrier pour
le Débutant et l'Habitué) ; les vents, qui suivent désormais le boss, restent le niveau jugé « après le
sommet » (nettement sous l'orage) ; places jugées déplacées (vents 7, final 8). À SIGNALER pour la carte
5.18 : la décision « monte jusqu'au boss (6) » voudrait un boss au moins aussi dur que l'orage ; il se
mesure juste en dessous (Débutant 2,88 contre 3,04, Habitué 0,58 contre 0,65 : 8 et 3 morts de moins sur
48 essais ; bruit de la mesure de l'ordre de 0,1 à 0,2). Leviers possibles : des motifs plus serrés (table
`MOTIFS` de `nuage.rs`, partagée avec l'arène d'essai), ou un quatrième motif ; à décider avec la courbe
sur onze niveaux.

Captures (`docs/qa/514-*`, 960 x 540, build de QA d'une extraction propre de f6206f5 servi en local sur
127.0.0.1 (port libre, serveur arrêté à la fin), Chromium sans tête (CDP) du poste, onglet au premier plan
(`Page.bringToFront`) et canvas focalisé, pilote `docs/capture-visibilite.mjs` (protocole
`docs/pilote-qa.js`, horloge virtuelle) avec le script du robot régénéré sur la même extraction ; numéros
d'image de `trace.csv`) :
- `514-carte` (11815) : « NIVEAU 6/8 », « L'ŒIL DU CYCLONE », la phrase d'AGA, le conseil ;
- `514-depart` (11850) : Pépin au départ à côté de la lanterne, la goutte du sol, les deux gouttes en
  l'air dans la trouée d'or, le nuage qui gronde (sourcils froncés, yeux d'or), trois points de vie en
  haut ;
- `514-motif-1-a` (11950) : premier motif, les cinq colonnes de gauche annoncées (pointillés, pieds d'or),
  Pépin saute pour en sortir ;
- `514-eclair` (11968) : leur frappe, Pépin juste à droite ;
- `514-essouffle-1` (12180) : le nuage essoufflé (paupières lourdes, goutte de sueur), Pépin monte dans le
  courant de gauche (traits blancs) vers la goutte ;
- `514-saut-tete` (12200) : MOMENT FORT, Pépin au sommet du courant, au-dessus de la tête, la goutte
  prise ;
- `514-coup-1` (12216) : le coup, nuage touché (yeux plissés), 2 points de vie, étincelles ;
- `514-motif-2` (12480) : deuxième motif, des bandes frappées et d'autres annoncées, Pépin au sol entre
  les colonnes frappées ;
- `514-coup-2` (12598) : deuxième coup (courant de droite), le nuage clignote (image où il est caché), 1
  point de vie ;
- `514-motif-3-a` (12800) : troisième motif, le plus serré, sept colonnes frappées et d'autres annoncées,
  Pépin au sol juste à côté d'une bande frappée ;
- `514-essouffle-3` (13060) : troisième essoufflement, Pépin saute vers le courant de gauche ;
- `514-coup-3` (13105) : troisième coup, 0 point de vie, la déchirure commence (fente d'or) ;
- `514-dechirure` (13170) : le ciel déchiré, la bande d'or pâle au milieu, Pépin plane ;
- `514-arrivee` (13232) : le nuage vaincu, l'or remplit l'écran, la transition commence ;
- `514-carte-suivante` (13440) : « NIVEAU 7/8 », « VENTS D'ALTITUDE ».
Chaque capture regardée ; `docs/mesure-capture.py <capture> 3` : 0 pixel hors palette, 0 bloc flou, sur
les 15. Comme noté à la carte 5.12b, deux sessions de rejeu ne donnent pas toujours la même image à
l'octet pour une même image du script : sur quatre sessions, certaines captures sont identiques (départ,
attente au sol entre deux salves), d'autres diffèrent autour de Pépin en mouvement (et de ses
étincelles) ; aucune session n'a montré de mort, et les deux sessions menées jusqu'au bout sont arrivées à
la carte des vents. Les 15 captures viennent d'une seule session, et Pépin y est reconnu
(`docs/reperer-pepin.py`) à quelques pixels de la place que donne `trace.csv`.

Vérification sur une extraction propre de f6206f5 (dernier commit de code ; `git archive`, nouveau dossier
daté) : `./docker-cargo.sh test --workspace --no-default-features` : 338 + 288 tests, tous verts (1 + 16
ignorés : écritures de fichiers, scripts, diagnostics) ; `./docker-cargo.sh lint` : `LINT-OK` ; script du
robot régénéré (empreinte `31055a8413c4561b`, 0 mort) ; `./build.sh` : `dist/` produit (480 350 octets de
wasm, + 919 octets, +0,2 %, pour le niveau et son thème), 0 occurrence des noms interdits, seule URL
`http://www.w3.org/2000/svg` ; le Grand Nuage est maintenant dans `dist/` (son niveau est dans le jeu),
l'arène d'essai toujours pas (« Essai nuage » : 0 fois dans `dist/game.wasm`, 1 fois dans
`dist-qa/game.wasm`) ; `./build.sh --qa` : `dist-qa/` produit (celui des captures). Planche de sprites :
aucun sprite nouveau ni modifié, non régénérée.

Limites (non testé ou à relativiser) :
- Pas de vrai téléphone, ni Safari ni Firefox : Chromium sans tête seulement, à 960 x 540. Pas de son
  entendu à l'oreille (grondement, tonnerre des éclairs, éclats).
- Le boss n'est jugé difficile que par la mesure (un joueur simulé qui voit les annonces) et par les
  captures ; aucun joueur humain. Une seule graine.
- Le joueur simulé a appris le combat dans `difficulte.rs` : ses chiffres dépendent de cet apprentissage
  (huit plans d'avance, voisinage d'une tranche de 15 pas). Un joueur moins prévoyant mourrait plus.
- Le robot de QA prend les trois gouttes avant le dernier coup (règle de la carte 5.11) ; un joueur qui
  frappe trop tôt finit le niveau sans elles (voulu : le niveau finit au dernier coup).

## La mer de nuages, niveau 8 (carte 5.15)

« Laisse le vent choisir, puis reprends la main. » Quatrième des nouveaux niveaux de l'étape 5, inséré à
sa place (8) après les vents d'altitude : Au-dessus des nuages devient le niveau 9 (« NIVEAU 8/9 »).
Grille `core/src/levels/mer.rs` : 176 x 11 tuiles, un écran de haut, des nuages (`N`) seulement. Lettres
`<` `>` (carte 5.3), `b` (carte 5.7), `g` (carte 5.4) et la manche à air `Q` (étape 4) ; aucune lettre
nouvelle. Nouveauté du niveau : le vent latéral ; les montgolfières et la grêle sont les deux mécaniques
que la demande d'AGA lui associe. Aucun ennemi (test `aucun_ennemi`).

Le parcours (détail en tête de `mer.rs`) :
1. le départ : un vide de 13 tuiles, trop large pour un plané sans aide ; le vent `>` au-dessus le fait
   traverser (sans vent, le niveau est bloqué dès ce vide : test `sans_vent_la_mer_de_nuages_est_infranchissable`) ;
2. un rideau de grêle au-dessus d'un vide de 6 tuiles, à traverser en planant ;
3. une montgolfière monte au premier étage (64 px plus haut : hors de portée d'un saut) ;
4. un long plané (13 tuiles) sous deux rideaux de grêle, une goutte entre les deux ;
5. une seconde montgolfière monte au sommet d'une tour de nuages, avec deux marches de trois tuiles ;
6. MOMENT FORT : depuis la tour, un plané de vingt tuiles (colonnes 110 à 129) au-dessus du vide, à
   travers trois couches de vent superposées et jointives : `>` en haut, `<` au milieu, `>` en bas. En
   descendant, le vent change deux fois de sens : il porte, il repousse (garder la flèche vers l'avant,
   ou fermer le parapluie pour traverser vite la couche contraire), il porte de nouveau (test
   `le_moment_fort_est_un_plane_de_vingt_tuiles_ou_le_vent_change_deux_fois`). Qui se laisse repousser
   retombe sur une marche et remonte à la tour : aucun cul-de-sac (test `on_repart_de_chaque_marche`) ;
7. une troisième montgolfière, puis un dernier plané porté, sous la grêle, jusqu'au drapeau.
Trois montgolfières de 5 tuiles chacune (arrêtées par un petit nuage au-dessus du ballon : test
`trois_montgolfieres_de_cinq_tuiles`), quatre rideaux de grêle, cinq zones de vent, trois gouttes (au
sommet du premier saut, entre les deux rideaux, au sommet du saut depuis la tour), quatre lanternes
(aucune sous la grêle).

Vent lisible (note du coordinateur, vérification de la carte 5.3 : 3 ou 4 tirets blancs de 1 px,
clairsemés, illisibles sur le ciel jaune du niveau d'essai) :
- les traînées : 8 px au lieu de 3, une queue et une TÊTE claire de 2 px à l'avant (côté où souffle le
  vent) : sur une image fixe, la tête dit le sens, comme une comète ; une traînée naît pour 16 tuiles de
  zone tous les 3 pas (au lieu d'une par zone), et vit 18 à 36 pas sans sortir de sa zone
  (`particules::souffle_lateral`, `effets::trainees_par_zone`) ;
- leurs couleurs viennent de la table d'ambiance (`decor::ambiance`, champ `vent`) : queue ardoise pâle et
  tête blanc cassé sur les ciels de nuit, queue violette (bleu nuit sous le ciel violet de l'altitude) et
  tête blanc cassé sur les ciels de jour ; aucune semi-transparence. Test
  `le_vent_lateral_se_voit_sur_chaque_bande_de_ciel` : sur chaque bande du ciel de chaque thème, la queue
  ou la tête tranche d'au moins 50 de luminance ; dans la mer de nuages, la queue seule tranche sur les
  quatre bandes et sur la mer de nuages du fond ;
- une manche à air (`Q`) à l'entrée de chaque zone de vent du jeu, à une tuile au plus de son bord, à une
  hauteur où il souffle (tests `chaque_zone_de_vent_lateral_a_sa_manche_a_air` et
  `chaque_manche_a_air_est_au_bord_d_un_courant_ou_d_un_vent`) ; elle s'étire dans le SENS DU VENT
  (`scene::manche_vers_la_droite`, test `la_manche_a_air_montre_le_sens_du_vent_lateral`) ; une manche
  sans vent latéral pointe toujours vers son courant ascendant (vents d'altitude inchangés) ;
- la physique du vent n'a pas changé (`VENT_LAT`, `VENT_LAT_PLANE`), ni le rendu des niveaux sans vent
  latéral (aucun des sept autres niveaux du jeu n'en a).

Ambiance, corail : un thème de plus, `Theme::Mer`. Un couchant corail qui se dore vers l'horizon (corail,
corail clair, orange, or : entre les vents d'altitude et le soleil, test
`le_ciel_s_eclaircit_de_l_orage_au_soleil`) ; tout en bas, la mer de nuages, rose et sans trouée, avec
ses bancs en dôme ; devant, des bancs violets dans l'ombre ; au milieu du ciel, des îlots corail au ventre
violet ; pas de pluie (seule la grêle tombe). Les nuages du niveau (clairs, bleutés) ne se confondent avec
aucune de ces couleurs ; les gouttes d'or se détachent sur le corail. Aucun sprite nouveau ni modifié.
Carte de titre : « LA MER DE NUAGES », la phrase d'AGA, le conseil « Parapluie ouvert, le vent te porte ;
fermé, tu lui résistes. ».

Robots et tests :
- faisabilité : `niveau_8_faisable` (31,9 s en course rapide, 3 gouttes, 4 lanternes, robot maladroit
  compris ; cible 50 s), `reprise_depuis_chaque_lanterne`, `durees_plausibles`, lint de grille (lanternes
  posées, aucune poche close), décor posé sur un solide et jamais devant un danger ; `niveau_9_faisable`
  pour les nuages renumérotés ;
- robot de QA : `le_robot_finit_tous_les_niveaux_sans_mourir` vert sur les neuf niveaux, sans rien lui
  apprendre (vent, grêle et montgolfières sont connus depuis les cartes 5.3, 5.4, 5.7). Nouvelle
  empreinte de la partie complète (un niveau de plus, donc attendue) : `5cfdacb1518e039e`, 20 054 images,
  0 mort, 15 gouttes, score 2 400, chrono 17 738 pas ; la mer de nuages : images 15896 à 17865 du script.

À SIGNALER, réglé dans la carte : la première version de la tour avait des marches d'UNE tuile, 48 px
puis 32 px sous la tour (la manche à air exige trois tuiles de solide sous elle : elles sont passées à
trois tuiles, aux mêmes hauteurs). La mesure de difficulté a alors montré 11 essais Débutant et 4 Habitué
NON FINIS (6 minutes de jeu) : le joueur simulé repoussé par la couche du milieu retombait sur une
marche (presque toujours la marche haute, colonne 112) et y restait jusqu'à la fin de l'essai, ses
itinéraires ne connaissant aucun chemin depuis ces cases (diagnostic : environ 370 « secours » de suite,
aucun détour trouvé). Marches remontées de 16 px (dessus à 96 et 128 : 32 px de marche à marche, comme de
la marche haute à la tour), test `on_repart_de_chaque_marche` (robot de faisabilité, depuis chaque
marche) ; mesure refaite : 48/48 partout. Aucune mécanique ne s'est mal comportée ; la physique n'a pas
été touchée.

Mesure de difficulté (`docs/mesure-difficulte.txt`, graine 20261113, 48 essais) :

| Niveau                         | Avant (5.14) D / H / E | Après (5.15) D / H / E |
| ------------------------------ | ---------------------- | ---------------------- |
| 1. Les toits                   | 0,4 / 0,1 / 0,0        | 0,4 / 0,1 / 0,0        |
| 2. Gouttières et cheminées     | 1,5 / 0,5 / 0,2        | 1,5 / 0,5 / 0,2        |
| 3. Le marché couvert           | 0,8 / 0,5 / 0,2        | 0,8 / 0,5 / 0,2        |
| 4. Le grand orage              | 3,0 / 0,6 / 0,4        | 3,0 / 0,6 / 0,4        |
| 5. Le clocher                  | 0,6 / 0,3 / 0,0        | 0,6 / 0,3 / 0,0        |
| 6. L'œil du cyclone            | 2,9 / 0,6 / 0,3        | 2,9 / 0,6 / 0,3        |
| 7. Vents d'altitude            | 1,3 / 0,4 / 0,1        | 1,3 / 0,4 / 0,1        |
| 8. La mer de nuages            |,                      | 1,3 / 0,3 / 0,1        |
| 9. Au-dessus des nuages (ex-8) | 0,5 / 0,1 / 0,0        | 0,5 / 0,1 / 0,0        |
| Total                          | 11,1 / 3,2 / 1,2       | 12,4 / 3,5 / 1,3       |

Les huit autres niveaux sont identiques LIGNE À LIGNE (rang de graine 9 pour la mer, `RANGS_DE_GRAINE`).
Dans la mer de nuages : 48/48 essais finis pour chaque profil, en 42 s (Débutant), 35 s (Habitué), 33 s
(Expert) ; surtout des chutes (le vide sous le premier rideau de grêle, le premier plané, les sauts depuis
les montgolfières, la tour) et deux morts sous la grêle (Habitué). Exactement (morts cumulées / 48) :
1,27 / 0,33 / 0,10, contre 1,31 / 0,40 / 0,10 pour les vents. Juge de la courbe : bande Habitué de la mer
[0,2 ; 1,0[ ; le final (au-dessus des nuages) jugé à la 9e place ; l'orage reste le sommet. À SIGNALER pour
la carte 5.18 : la décision « remonte un peu au 8 » voudrait la mer un peu AU-DESSUS des vents ; elle se
mesure au même niveau (un peu en dessous pour l'Habitué : 16 morts contre 19 sur 48 essais, dans le bruit
de la mesure). Leviers possibles, à décider avec la courbe sur onze niveaux : un plané du moment fort
plus long ou une couche contraire plus épaisse (la plupart des joueurs simulés le passent en gardant la
flèche vers l'avant), un rideau de grêle sur la fin du premier plané.

Captures (`docs/qa/515-*`, 960 x 540, build de QA d'une extraction propre de 5b1da36 servi en local sur
127.0.0.1 (port libre, serveur arrêté à la fin), Chromium sans tête (CDP) du poste, onglet au premier plan
(`Page.bringToFront`) et canvas focalisé, pilote `docs/capture-visibilite.mjs` (protocole
`docs/pilote-qa.js`, horloge virtuelle) avec le script du robot régénéré sur la même extraction ; numéros
d'image de `trace.csv`) :
- `515-carte` (15885) : « NIVEAU 8/9 », « LA MER DE NUAGES », la phrase d'AGA, le conseil ;
- `515-depart` (15925) : Pépin au départ, la manche à air au bord du nuage tournée vers la droite, les
  premières traînées au-dessus du vide, la goutte au sommet du saut, la mer de nuages rose et ses bancs
  violets, les îlots corail ;
- `515-plane-porte` (16100) : le premier plané, porté (traînées têtes à droite tout autour), la goutte
  prise (étincelles) ;
- `515-grele` (16395) : Pépin plane sous le premier rideau, la grêle éclate sur la toile ;
- `515-montgolfiere` (16590) : Pépin dans la nacelle qui monte vers l'étage et sa lanterne ;
- `515-entre-deux-rideaux` (16850) : le long plané entre les deux rideaux, la goutte prise ;
- `515-tour` (17160) : Pépin au sommet de la tour, à côté de la lanterne, la montgolfière vide qui
  redescend, la goutte au-dessus du vide ; les trois manches à air, tournées à droite (tour), à GAUCHE
  (marche du haut), à droite (marche du bas) : les trois couches se lisent avant d'y entrer ;
- `515-inversion-1` (17270), `515-inversion-2` (17350), `515-inversion-3` (17405) : MOMENT FORT, Pépin
  plane dans la couche du haut, puis dans celle du milieu (repoussé), puis dans celle du bas, au-dessus
  de l'arrivée ; sur chaque image, les traînées des trois couches ;
- `515-inversions-zoom` : découpe de `515-inversion-1` (x 0 à 480, y 90 à 450 de la capture) agrandie
  deux fois (plus proche voisin, aucun pixel recalculé), 960 x 720 : têtes claires à droite en haut, à
  gauche au milieu, à droite en bas, et les manches à air des deux marches (gauche, droite) ;
- `515-arrivee` (17838) : Pépin se pose, parapluie ouvert, au pied du drapeau, le dernier rideau de grêle
  et les dernières traînées derrière lui (14 gouttes sur 27 depuis le début de la partie) ;
- `515-carte-suivante` (18059) : « NIVEAU 9/9 », « AU-DESSUS DES NUAGES » ;
- `515-essai-vent` (image 636 du script 53, touche F1) : le niveau d'essai « vent » de la carte 5.3 avec
  les nouvelles traînées, lisibles sur le ciel jaune du soleil (à comparer à `53-essai-vent`).
Chaque capture regardée. Le sens des traînées a aussi été compté sur les pixels des trois captures
d'inversion (trait violet de 6 px avec tête blanc cassé de 2 px à droite ou à gauche), rangée de tuiles
par rangée de tuiles : couche du haut (écran y 36 à 68) toutes têtes à droite, couche du milieu (68 à
116) têtes à gauche, couche du bas (116 à 148) têtes à droite, sauf aux deux rangées de jonction où les
deux sens se côtoient. `docs/mesure-capture.py <capture> 3` : 0 pixel hors palette, 0 bloc flou, sur les
13 captures de 960 x 540 (le zoom n'a que les couleurs de sa capture). `515-montgolfiere` et
`515-arrivee` viennent d'une seconde session de rejeu (même build, même script), les autres d'une seule
session ; aucune mort dans aucune.

Vérification sur une extraction propre de 14727dd (dernier commit de code ; `git archive`, nouveau
dossier daté) : `./docker-cargo.sh test --workspace --no-default-features` : 345 + 294 tests, tous verts
(1 + 16 ignorés : écritures de fichiers, scripts, diagnostics, mesure) ; `./docker-cargo.sh lint` :
`LINT-OK` ; `./build.sh` : `dist/` produit (483 621 octets de wasm, + 3 271 octets, +0,7 %, pour le niveau,
son thème et les traînées), 0 occurrence des noms interdits, seule URL `http://www.w3.org/2000/svg` ; la
mer de nuages est dans `dist/game.wasm`, le niveau d'essai vent toujours pas (« Essai vent » : 0 fois).
Script du robot et build de QA (`./build.sh --qa`, celui des captures) : extraction propre de 5b1da36
(même jeu : les commits suivants ne changent qu'un test, un commentaire et les captures). Planche de
sprites : aucun sprite nouveau ni modifié, non régénérée.

Limites (non testé ou à relativiser) :
- Pas de vrai téléphone, ni Safari ni Firefox : Chromium sans tête seulement, à 960 x 540. Pas de son.
- Lisibilité du vent jugée sur captures et par un seuil de luminance, sans joueur humain ; la tête de
  2 px se voit nettement en x3, moins sur un petit écran de téléphone (x2 ou moins) : la manche à air
  reste alors le repère principal.
- La difficulté n'est jugée que par la mesure (une graine, un joueur simulé) ; la mer ne « remonte » pas
  encore au-dessus des vents (voir plus haut, pour la carte 5.18).
- Dans un niveau à vent, les traînées tirent plus de nombres au générateur des particules : les
  éclaboussures et la poussière de ce niveau ne sont plus dessinées à l'identique d'avant (les
  particules repartent de zéro à chaque niveau : les autres niveaux ne changent pas) ; aucun effet sur
  le jeu ni sur l'empreinte du robot (elle ne lit pas les particules).

## L'arc-en-ciel, niveau bonus (carte 5.16)

« Il ne tient que le temps de le traverser. » Dixième place de la table, genre `Bonus`
(`core/src/levels/arc.rs`, 240 x 11 tuiles, un écran de haut). Il n'est joué que si les 27 gouttes
d'or des neuf niveaux normaux ont été ramassées pendant la partie en cours (`levels::apres`, carte
5.1) ; sinon l'écran de fin suit directement le 9e niveau (l'épilogue viendra à la carte 5.17).

Ce qui a été fait :
- Le niveau : un pont de 154 dalles friables (`F`) d'un bout à l'autre ; seuls le nuage du départ
  (colonnes 0 à 5) et celui de l'arrivée (230 à 239) sont solides. Aucun ennemi, aucune lanterne
  (exigence d'AGA) : une chute ramène au départ. Une dalle touchée tombe une demi-seconde plus tard :
  on ne s'arrête jamais. Le parcours enjambe trois arcs, comme l'arc-en-ciel : marches de six tuiles
  qui montent d'une rangée, un sommet coupé de deux vides, un long plané de dix tuiles, des marches de
  deux rangées, trois pas japonais de deux dalles, cinq marches qui remontent, un second long plané,
  puis un sprint au ras de la mer de nuages avec des vides d'une tuile. 3 gouttes (au-dessus du
  premier vide, du sommet du premier arc, à la fin du premier plané), 4 oiseaux posés (décor) sur les
  deux nuages solides. Test `un_pont_friable_d_un_bout_a_l_autre_sans_ennemi_ni_lanterne` (aucune
  tuile solide entre les colonnes 6 et 229, rien que des dalles et des gouttes).
- Nouveauté du niveau : le pont qui ne pardonne pas l'arrêt, aucun appui sûr entre le départ et
  l'arrivée. Aucune lettre de grille nouvelle (`F`, `K` existaient).
- Thème `ArcEnCiel`, toute la palette : ciel d'ardoise après l'averse (sombre en haut, pâle vers
  l'horizon), un grand arc-en-ciel de huit bandes de 3 px (corail, corail clair, orange, or, or pâle,
  cyan, cyan sombre, violet : la palette n'a pas de vert, les deux cyans en tiennent lieu) dessiné en
  colonnes de pixels entiers (`decor::bande_arc`, pure : bandes jointives sans trou ni
  recouvrement, symétriques), qui glisse d'un écran de droite à gauche sur tout le niveau
  (parallaxe 0,10 : au départ on voit son pied gauche, au milieu du pont l'arc entier, à l'arrivée
  son pied droit) ; bancs de nuages pâles au loin, sombres devant ; pas de pluie, ni étoiles, ni
  îlots (un îlot clair se lirait comme une dalle). Tests `l_arc_en_ciel_a_huit_bandes_jointives`,
  `l_arc_glisse_d_un_ecran_sur_le_bonus`, `l_arc_en_ciel_montre_toute_la_palette` (ciel, bancs,
  arc et dalles du pont : les 16 couleurs) ; les 16 couleurs ont aussi été comptées sur les pixels
  des captures `516-depart`, `-plane`, `-arc-entier` et `-arrivee`.
- Carte de titre : « NIVEAU BONUS », « L'ARC-EN-CIEL », la phrase d'AGA, le conseil « Ne t'arrête
  pas : tout s'effrite derrière toi. » (une première version, « Ne t'arrête jamais : ... », laissait
  « toi. » seul sur une seconde ligne : vu sur la capture, corrigé, et un test vérifie que les deux
  phrases du bonus tiennent chacune sur une ligne).
- Faisabilité : `niveau_10_faisable` (39,6 s en course rapide, robot maladroit compris) ; le bonus
  est le seul niveau sans lanterne (`verifier_infos` l'exige) ; `durees_plausibles` (39,6 s, cible
  60 s : « un sprint d'une minute »). Lint de grille, décor, reprise : verts. Le robot du cœur
  (`faisabilite`) voit les dalles friables comme des solides fixes ; c'est le robot de QA, dans la
  vraie simulation, qui prouve qu'on traverse un pont qui s'effondre.

Correctif a minima d'une mécanique (signalé) : les dalles friables se reforment à la mort de Pépin.
Avant, une dalle tombée ne revenait que 3 s après sa chute (si Pépin n'était pas dessus). Dans le
bonus, sans lanterne, une chute aux premiers vides ramenait au départ devant un pont encore effondré :
il fallait attendre, ou l'on retombait aussitôt. Désormais `entities::reprendre_apres_une_mort` (qui
remettait déjà le Grand Nuage au début de son combat) reforme aussi chaque dalle friable
(`PlateformeFriable::reformer`), sauf une qui chevaucherait Pépin déjà replacé. Tests
`se_reforme_aussitot_apres_une_mort_sauf_dans_le_joueur` (core) et
`dans_le_bonus_une_chute_ramene_au_depart_sur_un_pont_reforme` (game). Effet ailleurs : aucun sur la
mesure (les neuf autres niveaux identiques ligne à ligne), aucun sur le robot (il ne meurt jamais,
empreinte inchangée).

Les DEUX chemins, prouvés à trois niveaux :
- Règles pures (`core/src/levels.rs`) : `la_vraie_table_joue_le_bonus_avec_toutes_les_gouttes_et_le_saute_sinon`
  (27 gouttes : 1 à 9, le bonus, la fin ; 26 ou 0 : 1 à 9, la fin ; après le bonus : la fin).
- Application (`game/src/app.rs`) : `apres_le_neuvieme_le_bonus_seulement_avec_toutes_les_gouttes`
  (le 9e fini avec 27 gouttes : carte « NIVEAU BONUS » de l'arc-en-ciel, puis l'écran de fin avec 30
  gouttes ; avec 26 ou 0 : directement l'écran de fin).
- Partie entière, robot de QA : la partie habituelle du robot ramasse 15 gouttes et SAUTE le bonus
  (le test `le_robot_finit_tous_les_niveaux_sans_mourir` le vérifie désormais : niveaux joués 1 à
  9) ; une partie « complète » nouvelle (`trouver_parcours_complet` : recherche par étapes, la goutte
  la plus proche, puis la suivante, puis l'arrivée ; `chercher_avec` refuse une arrivée tant qu'une
  goutte manque) ramasse les trois gouttes de chaque niveau et JOUE le bonus : test
  `la_partie_complete_joue_le_bonus` (niveaux 1 à 10, 3, 6, ..., 30 gouttes, aucune mort). Le
  parcours du bonus est aussi rejoué seul (`le_robot_traverse_l_arc_en_ciel_sans_mourir`). Rien à
  apprendre au robot : il joue dans la vraie simulation, où la dalle tombe sous lui s'il s'attarde
  (le plan finit en chute et il est écarté) ; posé, il ne reste que 20 pas immobile, la dalle en
  tient 30.
- Navigateur : les deux scripts rejoués dans le build de QA (captures plus bas).

Robot de QA (extraction propre de 41f68d8) : partie habituelle 20 054 images, 0 mort, 15 gouttes,
score 2400, empreinte 5cfdacb1518e039e, IDENTIQUE à celle de la carte 5.15 (les neuf niveaux et leur
enchaînement n'ont pas bougé ; le bonus est sauté). Partie complète (`ecrire_script_partie_complete`,
`target/qa/script-516.txt`) : 24 084 images (401,4 s), 0 mort, 30 gouttes, score 3900, chrono
21 540 pas, empreinte ec81270b03b82c66 ; le bonus y dure 3 006 images (50,1 s, détours pour les
gouttes compris).

Mesure de difficulté (même commande, graine 20261113, 48 essais), morts moyennes, avant (2fb73cd,
carte 5.15) et après (d77cba3) :

| Niveau                      | Débutant avant | Débutant après | Habitué avant | Habitué après | Expert avant | Expert après |
| --------------------------- | -------------- | -------------- | ------------- | ------------- | ------------ | ------------ |
| 1. Les toits                | 0,4            | 0,4            | 0,1           | 0,1           | 0,0          | 0,0          |
| 2. Gouttières et cheminées  | 1,5            | 1,5            | 0,5           | 0,5           | 0,2          | 0,2          |
| 3. Le marché couvert        | 0,8            | 0,8            | 0,5           | 0,5           | 0,2          | 0,2          |
| 4. Le grand orage           | 3,0            | 3,0            | 0,6           | 0,6           | 0,4          | 0,4          |
| 5. Le clocher               | 0,6            | 0,6            | 0,3           | 0,3           | 0,0          | 0,0          |
| 6. L'œil du cyclone         | 2,9            | 2,9            | 0,6           | 0,6           | 0,3          | 0,3          |
| 7. Vents d'altitude         | 1,3            | 1,3            | 0,4           | 0,4           | 0,1          | 0,1          |
| 8. La mer de nuages         | 1,3            | 1,3            | 0,3           | 0,3           | 0,1          | 0,1          |
| 9. Au-dessus des nuages     | 0,5            | 0,5            | 0,1           | 0,1           | 0,0          | 0,0          |
| 10. L'arc-en-ciel (bonus)   |,              | 8,8            |,             | 1,2           |,            | 0,3          |
| Total                       | 12,4           | 21,2           | 3,5           | 4,7           | 1,3          | 1,6          |

Détail du bonus : Débutant 39/48 essais finis (6 minutes au plus par essai), 215 s en moyenne, pire
essai 22 morts ; Habitué 48/48, 80 s, pire 6 ; Expert 48/48, 59 s, pire 1. Lieux : chutes partout,
surtout aux pas japonais (col. 127-131), au sommet du premier arc (63-67) et dans le deuxième arc
(117-121). Les neuf autres niveaux : identiques ligne à ligne (lieux de mort compris).

Lecture : le bonus est, en morts, le niveau le plus dur du jeu (Habitué 1,2 contre 0,6 à l'orage) :
sans lanterne, chaque chute recommence le sprint, et le joueur simulé, qui se pose et s'arrête 1/3 s
avant chaque plan, est plus exposé qu'un joueur qui court sans s'arrêter (une dalle tient 1/2 s). Une
première version (marches de cinq tuiles, pas japonais d'une dalle, plané de douze tuiles), mesurée
avant le correctif des dalles, donnait presque la même chose (8,8 / 1,0 / 0,3) : élargir les marches
n'a fait que déplacer les chutes. C'est voulu
pour un niveau réservé à qui a ramassé les 27 gouttes, et conforme à la décision du coordinateur (« le
bonus se juge au chrono, pas aux morts ») : `ecarts_de_la_courbe` sort le bonus de la règle « sous
le sommet », lui donne une bande Habitué de simple garde-fou ([0 ; 2[) et juge son chrono : la durée
moyenne d'un essai Expert doit rester dans [45 ; 75[ s (mesurée : 59 s, « un sprint d'une minute »).
Test `la_forme_de_la_courbe_est_jugee` complété (un bonus à 8,8 morts passe, un bonus de 80 s pour
l'Expert est refusé). À revoir avec la courbe sur onze niveaux (carte 5.18).

Durée des tests : la mesure de difficulté prend maintenant 4 min 50 s (au lieu de 30 à 90 s) : le
Débutant y rejoue le bonus jusqu'à 6 minutes par essai. `courbe_de_difficulte_dans_les_bandes`
(lancé avec tous les tests) refait cette mesure : la suite `game` dure environ 7 min, `core` 3 min ;
`la_partie_complete_joue_le_bonus` dure 2 min 30 s à elle seule (un fil par niveau).

Captures (`docs/qa/516-*`, 960 x 540, build de QA d'une extraction propre de 41f68d8 servi sur
127.0.0.1 (port libre, serveur arrêté à la fin), Chromium sans tête (CDP) du poste, onglet au premier
plan (`Page.bringToFront`) et canvas focalisé, pilote `docs/capture-visibilite.mjs` avec
`SCRIPT=target/qa/script-516.txt` (partie complète) puis `script.txt` (partie du robot), générés sur
la même extraction ; numéros d'image de `trace-516.csv` et `trace.csv`) :
- partie complète, le bonus JOUÉ :
  - `516-fin-du-9` (20545) : la fin du 9e niveau, Pépin court vers le drapeau, 27/30 au compteur ;
  - `516-carte-bonus` (20760) : « NIVEAU BONUS », « L'ARC-EN-CIEL », « Il ne tient que le temps de
    le traverser. », le conseil sur une ligne ;
  - `516-depart` (20850) : Pépin sur le nuage du départ entre ses deux oiseaux, le pont qui part au
    ras de la mer de nuages, le pied gauche de l'arc-en-ciel qui se lève à droite, la première goutte
    devant l'arc (son contour sombre la détache des bandes) ;
  - `516-premier-arc` (21300) : Pépin saute de marche en marche ; derrière lui, la marche qu'il
    vient de quitter s'effrite (dalle fendue) et celles d'avant ont disparu ;
  - `516-plane` (21880) : le long plané vers le bas, parapluie ouvert, sous l'arc entier ; devant
    Pépin, la goutte au-dessus du bout du pont où il va se poser (29/30) ;
  - `516-arc-entier` (22380) : MOMENT FORT, l'arc-en-ciel entier au-dessus des bancs de nuages, Pépin
    en haut du deuxième arc, la dalle sous lui déjà fendue, le vide derrière ;
  - `516-pas-japonais` (22500) : les pas japonais qui redescendent ;
  - `516-sprint` (23500) : le sprint final, la dalle quittée qui s'effrite, le pied droit de l'arc ;
  - `516-arrivee` (23814) : Pépin descend en planant vers le drapeau, sur le nuage de l'arrivée
    entre ses deux oiseaux, le pied droit de l'arc à gauche, 30/30 ;
  - `516-fin-complete` (23990) : écran de fin, 30 gouttes sur 30, « Collection complète » ;
- partie du robot, le bonus SAUTÉ :
  - `516-sans-bonus-fin-du-9` (19745) : la fin du 9e niveau, Pépin court vers le drapeau, 15/30 au
    compteur ;
  - `516-sans-bonus-fin` (19960) : l'écran de fin, 15 gouttes sur 30, juste après le 9e (aucune
    carte de bonus entre les deux : la trace passe de « jeu 9 » à « fin » à l'image 19813).
Chaque capture regardée. `docs/mesure-capture.py <capture> 3` : 0 pixel hors palette, 0 bloc flou
sur les 12. Les captures ont été faites deux fois : la première série (build d'une extraction de
d77cba3) a montré le conseil coupé sur deux lignes (voir plus haut) ; la série versionnée vient de
41f68d8, après correction ; aucune mort dans aucune. Remarque hors carte : le message de fin à 15
gouttes (« Pas mal ! Il reste des gouttes d'or ici et là. », inchangé) laisse « là. » seul sur sa
seconde ligne (`516-sans-bonus-fin`) ; à reprendre avec les textes, pas dans cette carte.

Vérification sur une extraction propre de 41f68d8 (dernier commit de code ; `git archive`, nouveau
dossier daté) : `./docker-cargo.sh test --workspace --no-default-features` : 348 + 301 tests, tous
verts (1 + 17 ignorés : écritures de fichiers, scripts, diagnostics, mesure), en 3 min et 7 min ;
`./docker-cargo.sh lint` : `LINT-OK` ; `./build.sh` : `dist/` produit (487 490 octets de wasm,
+ 3 869 octets, +0,8 %, pour le niveau, son thème et l'arc), 0 occurrence des noms interdits, seule
URL `http://www.w3.org/2000/svg` ; « L'arc-en-ciel » est dans `dist/game.wasm`, le niveau d'essai vent
toujours pas (« Essai vent » : 0 fois). Build de QA (`./build.sh --qa`) et scripts du robot
(`ecrire_script_navigateur`, `ecrire_script_partie_complete`) : même extraction. Planche de sprites :
aucun sprite nouveau ni modifié, non régénérée. Les commits suivants (captures, QA.md) ne touchent pas
au code.

Limites (non testé ou à relativiser) :
- Pas de vrai téléphone, ni Safari ni Firefox : Chromium sans tête seulement, à 960 x 540. Pas de son.
- Le bonus n'a été joué que par le robot (qui sautille de dalle en dalle) et par le joueur simulé de
  la mesure, pas par un humain : la sensation de « sprint » (courir sans s'arrêter) n'est pas jugée.
- La partie complète du robot n'est pas la plus courte qui ramasse tout (gouttes prises dans l'ordre
  où elles sont les plus proches) : son chrono n'est pas un record.
- La difficulté n'est jugée que par la mesure (une graine) ; le bonus y est plus meurtrier que le
  sommet, assumé (jugé au chrono) mais à confirmer avec la courbe finale (carte 5.18).
- L'or et l'or pâle de l'arc-en-ciel sont des couleurs que le décor évitait jusqu'ici (réservées aux
  gouttes et aux éclairs) : « toute la palette » l'impose ; les bandes ne font que 3 px et les gouttes
  gardent leur contour sombre, aucune confusion vue sur les captures.

## La descente, l'épilogue (carte 5.17)

« Maintenant, on rentre. » Onzième et dernière place de la table, genre `Epilogue`
(`core/src/levels/descente.rs`, 32 x 45 tuiles, quatre écrans de haut) : il suit le 9e niveau, ou le
bonus quand celui-ci a été débloqué ; l'écran de fin vient juste après.

Ce qui a été fait :
- Le niveau, vertical à l'envers : départ tout en haut, sur un petit nuage au soleil (deux oiseaux
  posés) ; arrivée tout en bas, sur le toit du premier niveau (mêmes tuiles `#` et `B`, une cheminée,
  une antenne, un bloc de briques avant le drapeau, comme à la fin des toits). Entre les deux, trois
  couches de nuages, neuf rangées l'une sous l'autre, chacune percée d'un trou de six tuiles, une fois
  à droite (colonnes 24 à 29), une fois à gauche (2 à 7) : on court jusqu'au trou, on s'y laisse tomber
  ou on y plane, et l'on repart dans l'autre sens sur la couche du dessous. Deux pigeons (`p`) perchés
  au-dessus de la première couche, tournés vers Pépin qui arrive : on passe dessous sans s'arrêter, ils
  plongent derrière lui. Deux colonnes de grêle (`g`) tombent du dessous d'une couche, le long du bord
  LOINTAIN du trou suivant (colonnes 2 et 29), jusqu'à la couche ou au toit d'en dessous : on descend à
  côté d'elles, et les 2e et 3e gouttes flottent dans les trous, près d'elles (planer pour aller les
  chercher). 3 gouttes, 4 lanternes (une à chaque atterrissage). Aucune lettre de grille nouvelle.
- Le plus calme des onze : le toit de l'arrivée couvre toute la largeur, donc AUCUNE chute ne tue
  (tout trou mène à la couche du dessous), aucune pointe, aucun éclair, rien qui bouge ; seuls dangers,
  les deux pigeons et les deux colonnes de grêle. Tests `aucune_chute_ne_tue_et_seuls_deux_pigeons_et_deux_colonnes_de_grele_menacent`
  et `se_laisser_tomber_du_bord_d_un_trou_ne_touche_pas_la_grele` (vraie physique, pas à pas : courir et
  tomber du bord de chaque trou pose Pépin sur la couche du dessous sans toucher la grêle).
- Nouveauté du niveau : la descente elle-même, sous un ciel qui refait le voyage en sens inverse. Thème
  `Descente` : au lieu des quatre bandes fixes, dix bandes de 45 px (`decor::VOYAGE`) défilent avec la
  caméra (parallaxe verticale 0,5, `remontee_voyage`) : or pâle, or, orange, corail clair, corail
  (« Au-dessus des nuages », la mer de nuages, les vents), violet (le clocher, le cyclone), puis
  l'ombre, la nuit et l'ardoise : en bas, EXACTEMENT le ciel du premier niveau. Le soleil au haut de la
  descente (dans la bande d'or), des îlots violets devant le corail (ceux des vents d'altitude), les
  étoiles et la ville du premier niveau en bas (mêmes immeubles, quelques fenêtres allumées : on rentre
  à la maison) ; tout défile ensemble, chaque élément reste devant sa bande. Pas de pluie : l'orage est
  fini. Tuiles et dalles teintées comme aux toits. Tests : `le_voyage_va_de_l_or_a_la_nuit`,
  `la_remontee_du_voyage_suit_la_hauteur_de_la_camera` (en bas : le ciel des toits pixel pour pixel ;
  en haut : les quatre premières bandes ; entière, monotone), `chaque_couleur_du_voyage_est_deja_le_ciel_d_un_autre_niveau`
  (donc tous les tests de contraste des sprites sur « tous les ciels » couvrent la descente),
  `soleil_ilots_etoiles_et_ville_restent_devant_leur_bande`, `seule_la_descente_refait_le_voyage`, et
  `le_pigeon_se_detache_du_ciel_de_la_descente` (sprites : pour chaque ligne qu'occupe un pigeon,
  perché puis jusqu'au fond de sa plongée, et CHAQUE hauteur de caméra où elle est à l'écran, contraste
  3:1 au moins avec la bande derrière lui).
- Carte de titre : « ÉPILOGUE », « LA DESCENTE », « Maintenant, on rentre. », conseil « Parapluie
  ouvert, la grêle ne te touche pas. ».
- Faisabilité : `niveau_11_faisable` (14,1 s en course rapide, robot maladroit compris),
  `reprise_depuis_chaque_lanterne`, lint de grille (lanternes posées, aucune poche close), décor
  (`chaque_niveau_a_son_decor` : oiseaux, cheminées, antenne), pigeons (au moins 4 tuiles des oiseaux,
  plongée en l'air et loin des lanternes), `aucun_point_de_reprise_sous_la_grele`, `durees_plausibles`
  (cible 40 s, la plus courte des onze : la mesure le joue en 18 à 19 s).

Écran de fin et compteur (addendum du coordinateur) :
- Partie finie par l'épilogue : « DE RETOUR ! », « Pépin replie son parapluie. » sous le titre, sur la
  nuit tranquille de la descente (le ciel et la ville du premier niveau, là où Pépin vient d'arriver).
  Sans épilogue (un jeu qui n'en aurait pas) : « LE SOLEIL ! » sur l'or, mise en page d'avant
  inchangée (test `sans_epilogue_l_ecran_de_fin_reste_le_soleil` : lignes en 42, 55, 68, 81, mot en
  98). La mise en page est une fonction pure (`fin::mise_en_page`) ; `rien_ne_se_chevauche_sur_l_ecran_de_fin`
  la vérifie pour tous les cas (avec ou sans épilogue, bonus joué ou non, 0 à 30 gouttes).
- Compteur : avant cette carte, le HUD et l'écran de fin comptaient sur `GOUTTES_TOTAL`, toutes les
  gouttes du jeu (10 niveaux : 30 ; avec l'épilogue, ce serait devenu 33, bonus compris, alors qu'une
  partie sans bonus ne peut en ramasser que 30). Désormais le total est celui du PARCOURS : les neuf
  niveaux normaux et l'épilogue, 30 (`levels::gouttes_du_parcours`) ; le bonus est compté à part : dans
  le bonus, le compteur montre ses gouttes sur 3 (`hud::compteur`) ; à la fin, une ligne « NIVEAU
  SECRET x/3 » s'il a été joué (`Application::gouttes_bonus`, remis à zéro par une nouvelle partie).
  Tests `le_compteur_compte_le_parcours_et_le_bonus_a_part`, `les_gouttes_du_parcours_et_du_bonus_sont_comptees_a_part`,
  `les_gouttes_du_parcours_et_celles_du_bonus`, `rejouer_oublie_les_gouttes_du_bonus`.
- Mot de fin : quand le bonus a été manqué, il annonce le niveau secret : « Il reste plein de gouttes
  d'or à dénicher. Toutes ouvrent un niveau secret. » (ou « Pas mal ! Il reste quelques gouttes d'or.
  Toutes ouvrent un niveau secret. ») ; bonus joué mais pas tout ramassé : « Bravo, tu as trouvé le
  niveau secret ! Quelques gouttes d'or manquent encore. » ; tout ramassé : « Collection complète :
  rien n'est oublié ! ». Chaque phrase tient sur sa ligne (test). L'ancien « Collection complète : pas
  une goutte oubliée ! » laissait « oubliée ! » seul sur une seconde ligne (vu sur la capture
  `517-complet-fin` de la première série) : raccourci, et un test le garde sur une ligne.

Les DEUX chemins vers l'épilogue :
- Règles pures : `la_vraie_table_joue_le_bonus_avec_toutes_les_gouttes_et_finit_par_l_epilogue`
  (27 gouttes : 1 à 9, le bonus, l'épilogue ; 26 ou 0 : 1 à 9, l'épilogue ; après l'épilogue : la fin).
- Application : `apres_le_neuvieme_le_bonus_seulement_avec_toutes_les_gouttes_puis_l_epilogue`
  (cartes « NIVEAU BONUS » puis « ÉPILOGUE », ou « ÉPILOGUE » seule ; puis l'écran de fin du retour,
  30 gouttes du parcours et le bonus à part).
- Robot de QA : la partie habituelle joue 1 à 9 puis 11 (`le_robot_finit_tous_les_niveaux_sans_mourir`) ;
  la partie complète joue 1 à 11, 3 gouttes de plus à chaque niveau, 33 en tout
  (`la_partie_complete_joue_le_bonus`). Aucune mort. Rien à apprendre au robot (pigeons et grêle
  existaient). Les deux scripts sont identiques à ceux de la carte 5.16 sur toute leur longueur
  d'alors (20 054 et 24 084 images) : les dix premiers niveaux se jouent exactement pareil.
  Partie habituelle : 21 339 images (355,6 s), 16 gouttes, score 2500, empreinte 0cfd185bb30e4abf ;
  partie complète : 25 511 images (425,2 s), 33 gouttes, score 4200, empreinte 2475461594d0de53.
- Navigateur : les deux scripts rejoués dans le build de QA (captures plus bas).

Mesure de difficulté (même commande, graine 20261113, 48 essais), morts moyennes, avant (d286fee,
carte 5.16) et après (133612b) :

| Niveau                      | Débutant avant | Débutant après | Habitué avant | Habitué après | Expert avant | Expert après |
| --------------------------- | -------------- | -------------- | ------------- | ------------- | ------------ | ------------ |
| 1. Les toits                | 0,4            | 0,4            | 0,1           | 0,1           | 0,0          | 0,0          |
| 2. Gouttières et cheminées  | 1,5            | 1,5            | 0,5           | 0,5           | 0,2          | 0,2          |
| 3. Le marché couvert        | 0,8            | 0,8            | 0,5           | 0,5           | 0,2          | 0,2          |
| 4. Le grand orage           | 3,0            | 3,0            | 0,6           | 0,6           | 0,4          | 0,4          |
| 5. Le clocher               | 0,6            | 0,6            | 0,3           | 0,3           | 0,0          | 0,0          |
| 6. L'œil du cyclone         | 2,9            | 2,9            | 0,6           | 0,6           | 0,3          | 0,3          |
| 7. Vents d'altitude         | 1,3            | 1,3            | 0,4           | 0,4           | 0,1          | 0,1          |
| 8. La mer de nuages         | 1,3            | 1,3            | 0,3           | 0,3           | 0,1          | 0,1          |
| 9. Au-dessus des nuages     | 0,5            | 0,5            | 0,1           | 0,1           | 0,0          | 0,0          |
| 10. L'arc-en-ciel (bonus)   | 8,8            | 8,8            | 1,2           | 1,2           | 0,3          | 0,3          |
| 11. La descente (épilogue)  |,              | 0,0            |,             | 0,0           |,            | 0,0          |
| Total                       | 21,2           | 21,2           | 4,7           | 4,7           | 1,6          | 1,6          |

Détail de l'épilogue : une seule mort en 144 essais (un Débutant, dans la grêle de la colonne 3 :
0,02 en moyenne) ; 48/48 essais finis pour chaque profil, 19 s (Débutant), 18 s (Habitué et Expert).
Les dix autres niveaux : identiques ligne à ligne (lieux de mort compris). C'est le plus calme des onze,
comme décidé : le juge de la courbe l'exige désormais (`ecarts_de_la_courbe` : pour le Débutant et
l'Habitué, l'épilogue ne meurt jamais plus que dans aucun autre niveau ; bande Habitué [0 ; 0,2[ ;
`la_forme_de_la_courbe_est_jugee` vérifie qu'un épilogue à 0,45 / 0,15 est refusé). Outil ajouté pour
régler un niveau sans relancer toute la mesure : `mesure_d_un_niveau` (test ignoré, variable NIVEAU,
défaut le dernier) donne la ligne d'un niveau, mêmes graines, en une seconde pour la descente.

Lecture honnête : la descente est calme parce que rien n'y tue en tombant ; le joueur simulé y meurt
moins qu'aux toits. Le parapluie y sert surtout à approcher sans risque les deux gouttes voisines de
la grêle (en planant, on est à l'abri) ; le reste se descend en se laissant tomber de trou en trou (le
robot, la mesure et sans doute un joueur pressé le font). C'est une descente « à la promenade », pas un long plané continu. Si AGA
veut plus de plané, la suite logique serait d'élargir les couches (moins de trous droits) : à décider
à la carte 5.18 avec la courbe entière.

Captures (`docs/qa/517-*`, 960 x 540, build de QA d'une extraction propre de 097df8b servi sur
127.0.0.1 (port libre, serveur arrêté à la fin), Chromium sans tête (CDP) du poste, onglet au premier
plan (`Page.bringToFront`) et canvas focalisé, pilote `docs/capture-visibilite.mjs`, scripts
`target/qa/script.txt` (partie du robot) et `script-516.txt` (partie complète) ; numéros d'image de
`trace.csv` et `trace-516.csv`) :
- partie du robot, bonus SAUTÉ :
  - `517-fin-du-9` (19745) : la fin du 9e niveau, 15/30 au compteur ;
  - `517-carte` (19960) : « ÉPILOGUE », « LA DESCENTE », « Maintenant, on rentre. », le conseil sur
    une ligne, sur le haut du voyage (or et orange) ;
  - `517-depart` (20030) : Pépin sur le nuage du départ, le soleil, la bande d'or, la première goutte
    en l'air ;
  - `517-pigeon` (20227) : le premier pigeon plonge derrière Pépin qui court sur la première couche ;
    le second, perché plus loin ; le ciel passe de l'or à l'orange puis au corail ;
  - `517-corail` (20330) : Pépin vient de passer le trou de droite de la première couche et descend
    devant le corail et les îlots violets ; au-dessus, le second pigeon en plongée ;
  - `517-grele-1` (20587) : Pépin descend par le trou de gauche de la deuxième couche, la colonne de
    grêle à côté de lui ; en dessous, la troisième couche et sa lanterne ; au-dessus le violet,
    derrière lui la nuit qui commence ;
  - `517-grele-2` (20827) : Pépin a passé le trou de droite de la troisième couche, la seconde colonne
    de grêle à sa droite ; les immeubles montent du bas (fenêtres allumées) ;
  - `517-ville` (20900) : Pépin posé sur le toit du premier niveau, cheminée et antenne, la ville, les
    étoiles, le ciel des toits ;
  - `517-arrivee` (21067) : Pépin descend en planant vers le drapeau, devant le bloc de briques ;
  - `517-fin` (21200) : l'écran de fin du retour : « DE RETOUR ! », « Pépin replie son parapluie. »,
    16/30, pas de ligne de bonus, « Il reste plein de gouttes d'or à dénicher. / Toutes ouvrent un
    niveau secret. », sur la nuit de la ville ;
- partie complète, bonus JOUÉ :
  - `517-complet-bonus` (21900) : dans le bonus, le compteur montre ses gouttes : 2/3 ;
  - `517-complet-carte` (23990) : la carte « ÉPILOGUE » juste après le bonus ;
  - `517-complet-descente` (24600) : dans l'épilogue, le compteur du parcours : 28/30 (27 + 1 ; les
    3 gouttes du bonus sont à part) ;
  - `517-complet-fin` (25400) : « DE RETOUR ! », 30/30, « NIVEAU SECRET 3/3 », 0 mort, « Collection
    complète : rien n'est oublié ! » sur une ligne.
Chaque capture regardée. `docs/mesure-capture.py <capture> 3` : 0 pixel hors palette, 0 bloc flou sur
les 14. Une première série (build de 357e5fa) a montré le mot de fin « Collection complète : pas une
goutte oubliée ! » coupé, « oubliée ! » seul sur la seconde ligne : corrigé (097df8b), série refaite.
Entre les deux séries, les images identiques à l'octet sont celles où rien ne bouge (cartes, départ,
écran de fin du robot, bonus) ; les autres diffèrent d'un pixel de caméra ou de Pépin (l'interpolation
de rendu dépend de l'instant réel de l'image, pas de la simulation, identique).

Vérification sur une extraction propre de 097df8b (dernier commit de code ; `git archive`, nouveau
dossier daté) : `./docker-cargo.sh test --workspace --no-default-features` : 352 + 313 tests, tous
verts (1 + 18 ignorés : écritures de fichiers, scripts, diagnostics, mesure, et le nouvel outil
`mesure_d_un_niveau`), en 4 min 30 s et 7 min 30 s ; `./docker-cargo.sh lint` : `LINT-OK` (une
première extraction, de 133612b, avait montré un avertissement clippy `int_plus_one` dans
`dessiner_voyage` : corrigé en 357e5fa) ; `./build.sh` : `dist/` produit (491 283 octets de wasm,
+ 3 793 octets, +0,8 %, pour le niveau, le ciel du voyage et l'écran de fin), 0 occurrence des noms
interdits, seule URL `http://www.w3.org/2000/svg` ; « La descente » est dans `dist/game.wasm`, le
niveau d'essai vent toujours pas (« Essai vent » : 0 fois). Build de QA (`./build.sh --qa`) et scripts
du robot (`ecrire_script_navigateur`, `ecrire_script_partie_complete`) : même extraction ; scripts et
traces identiques à l'octet à ceux de 357e5fa (rejoués pour les captures). Planche de sprites : aucun
sprite nouveau ni modifié, non régénérée. Le commit suivant (captures, QA.md) ne touche pas au code.

Limites (non testé ou à relativiser) :
- Pas de vrai téléphone, ni Safari ni Firefox : Chromium sans tête seulement, à 960 x 540. Pas de son.
- L'épilogue n'a été joué que par le robot et par le joueur simulé de la mesure, pas par un humain.
  Lecture plus haut : on peut le descendre presque sans planer ; le « long plané entre les pigeons et
  la grêle » de la demande d'AGA n'est qu'une des façons de le jouer.
- Le ciel du voyage est jugé sur les captures et par des tests de couleurs (pas de mesure de
  luminance sur image) ; ses bandes défilent à la moitié de la vitesse de la caméra : la sensation de
  « refaire le voyage » n'a pas été jugée par un humain.
- La difficulté n'est jugée que par la mesure (une graine) ; la courbe sur onze niveaux reste à faire
  (carte 5.18).

## QA finale des onze niveaux (carte 5.18)

Synthèse de l'étape 5 : onze niveaux (neuf du parcours, le bonus, l'épilogue), une nouveauté par
niveau, toutes jouées avec les mêmes touches. Cette carte règle la courbe de difficulté finale,
vérifie le robot sur les deux chemins (avec et sans bonus), met à jour les textes, mesure le poids et
les images par seconde, et capture chacun des onze niveaux.

### Courbe de difficulté : retouches et mesure avant/après

Décision du coordinateur : la courbe « monte jusqu'au boss (6), redescend au 7, remonte un peu au 8,
calme au 9 ; le bonus se juge au chrono ; l'épilogue est le plus calme de tous ». La mesure de départ
(`docs/mesure-difficulte.txt` de la carte 5.17) ne la suivait pas sur trois points (Habitué : orage
0,65, clocher 0,29, boss 0,58, vents 0,40, mer 0,33) : l'orage (4) restait le sommet, le clocher (5)
REDESCENDAIT, et la mer (8) restait sous les vents (7).

Chaque levier a été essayé sur une extraction à part (outil `mesure_d_un_niveau`, mêmes graines que la
mesure officielle, donc les mêmes chiffres que sa ligne du tableau), puis gardé ou écarté :

| Niveau | Levier essayé | Habitué (Débutant) | Gardé ? |
| ------ | ------------- | ------------------ | ------- |
| 5. Le clocher | petite aiguille plus rapide (36 -> 46 px/s) | 0,29 -> 0,23 (0,60 -> 0,56) | non : plus facile |
| 5. Le clocher | 4 pointes au sol de la cage, sous la grande aiguille | 0,65 (2,44) | non : le robot de faisabilité du cœur ne trouve plus le drapeau (`niveau_5_faisable` rouge ; avec 2 ou 3 pointes non plus) |
| 5. Le clocher | 1 pointe au sol de la cage, juste avant la lanterne de la porte | 0,69 (2,83) | OUI |
| 5. Le clocher | la même, plus un trou à pointe au pied de la tour du drapeau | 0,71 (3,19) | non : à peine plus, et la tour change de forme |
| 6. Le boss | motif 1 élargi d'une colonne par bande | 1,33 (3,12) | non : le Débutant à peine au-dessus de l'orage (3,04), dans le bruit |
| 6. Le boss | motifs 1 ET 2 élargis (une colonne, parfois deux) | 1,00 (3,60), puis 1,19 (4,06) | OUI |
| 8. La mer | un rideau de grêle de plus au-dessus du départ | 0,33 (1,27) | non : aucun effet |
| 8. La mer | quai creusé d'une colonne avant la première grêle | 0,29 (1,29) | non : aucun effet |
| 8. La mer | premier vide d'une tuile plus large | 0,33 (1,60) | non : Débutant seulement |
| 8. La mer | rive du moment fort une tuile plus loin (le vent s'arrête avant) | 0,46 (1,46) | OUI |
| 8. La mer | les deux derniers ensemble | 0,46 (1,73) | non : le Débutant seul en profite |

Pour le boss, les deux chiffres : avant, puis après la correction du robot de QA décrite plus bas (les
détours que le joueur simulé de la mesure calcule sur place passent par la recherche du robot,
`chercher`, et donc par son contrôle de décalage ; l'essai « motif 1 seul » a été mesuré avec le robot
d'avant).

Retouches gardées (le reste du niveau est identique, tests des pièces mis à jour) :
- Le clocher (`core/src/levels/clocher.rs`) : une pointe au sol de la cage (colonne 13, ligne 16), la
  lanterne de la porte décalée d'une colonne (14). On la saute en allant à la grande aiguille ; un saut
  manqué du haut de l'aiguille vers le pilier peut y finir : une chute coûte une vie, plus seulement de
  la hauteur. Robot de faisabilité et « reprise depuis chaque lanterne » verts.
- Le boss (`core/src/entities/nuage.rs`, table `MOTIFS`) : bandes des motifs 1 et 2 élargies d'une
  colonne (parfois deux) ; motif 3 et écarts entre salves (60, 40, 30 pas) inchangés ; le test
  `chaque_motif_se_traverse` (on peut toujours s'en sortir en courant) reste vert. L'arène du niveau
  d'essai (F10) partage la table : même combat.
- La mer (`core/src/levels/mer.rs`) : la rive qui suit le moment fort commence à la colonne 131 au lieu
  de 130 ; le vent garde ses vingt colonnes (110 à 129) : la dernière tuile se fait sans lui. Test
  `la_rive_du_moment_fort_commence_une_tuile_apres_le_vent`.

Morts moyennes (graine officielle, 48 essais par profil et par niveau), avant (carte 5.17) et après :

| Niveau | Débutant avant | après | Habitué avant | après | Expert avant | après |
| ------ | -------------- | ----- | ------------- | ----- | ------------ | ----- |
| 1. Les toits | 0,4 | 0,4 | 0,1 | 0,1 | 0,0 | 0,0 |
| 2. Gouttières et cheminées | 1,5 | 1,5 | 0,5 | 0,5 | 0,2 | 0,2 |
| 3. Le marché couvert | 0,8 | 0,8 | 0,5 | 0,5 | 0,2 | 0,2 |
| 4. Le grand orage | 3,0 | 3,0 | 0,6 | 0,6 | 0,4 | 0,4 |
| 5. Le clocher | 0,6 | **2,8** | 0,3 | **0,7** | 0,0 | 0,1 |
| 6. L'œil du cyclone (boss) | 2,9 | **4,1** | 0,6 | **1,2** | 0,3 | 0,4 |
| 7. Vents d'altitude | 1,3 | 1,3 | 0,4 | 0,4 | 0,1 | 0,1 |
| 8. La mer de nuages | 1,3 | **1,5** | 0,3 | **0,5** | 0,1 | 0,1 |
| 9. Au-dessus des nuages | 0,5 | 0,5 | 0,1 | 0,1 | 0,0 | 0,0 |
| 10. L'arc-en-ciel (bonus) | 8,8 | 8,8 | 1,2 | 1,2 | 0,3 | 0,3 |
| 11. La descente (épilogue) | 0,0 | 0,0 | 0,0 | 0,0 | 0,0 | 0,0 |
| Total | 21,2 | 24,8 | 4,7 | 5,9 | 1,6 | 1,8 |

En centièmes (Habitué) : toits 0,12, gouttières 0,50, marché 0,54, orage 0,65, clocher 0,69, boss
1,19, vents 0,40, mer 0,46, au-dessus des nuages 0,08 ; bonus 1,23 (au chrono : Expert 58,5 s, dans
[45 ; 75[ s) ; épilogue 0,00. La courbe Habitué MONTE donc du 1 au 6 (0,12 · 0,50 · 0,54 · 0,65 ·
0,69 · 1,19 : jamais de descente), redescend au 7 (0,40), remonte un peu au 8 (0,46) et se calme au 9
(0,08) ; l'épilogue est le plus calme de tous pour les trois profils. Débutant : 0,4 · 1,5 · 0,8 · 3,0
· 2,8 · 4,1 · 1,3 · 1,5 · 0,5 (le marché, à 0,8, reste sous les gouttières : il se joue « en
rebondissant », voir la carte 5.12) ; Expert : le boss (0,38) et l'orage (0,35) en tête. Lieux de mort
nouveaux : clocher, col. 12-14 « pointes » (la pointe de la cage : 29 des 33 morts Habitué) ; boss,
éclairs seulement ; mer, col. 132 « chute » (le bord de la nouvelle rive).

Non-régression : le juge de la courbe (`ecarts_de_la_courbe`, `game/src/difficulte.rs`) a été réécrit
pour la courbe finale. Sommet = le boss (avant : l'orage) ; nouvelles règles : la mer au-dessus des
vents (Débutant et Habitué), et une courbe Habitué qui ne redescend jamais avant le boss (un palier est
admis) ; bandes Habitué du clocher [0,5 ; 1,2[ et du boss [0,7 ; 1,6[. Le test
`la_forme_de_la_courbe_est_jugee` vérifie que les places désignent bien le boss, les vents, la mer et
le final, qu'une bonne courbe passe (avec ou sans palier), et que la mesure d'AVANT cette carte est
refusée avec ses cinq écarts (orage au-dessus du boss pour les deux profils, mer pas au-dessus des
vents pour les deux, clocher qui redescend). `courbe_de_difficulte_dans_les_bandes` passe sur la
mesure officielle.

Robot de QA dans l'arène du boss : avec les motifs élargis, le parcours trouvé tenait plan par plan
(`tient_au_decalage` : chaque plan rejoué avec le nuage décalé d'une ou deux images) mais pas rejoué en
entier avec deux images d'avance (test `le_parcours_du_grand_nuage_tient_a_une_image_pres` : 3 à 5
morts). Diagnostic (trace image par image du rejeu décalé) : le plan s'adapte (« garder la direction en
descendant » descend plus tard si le courant du nuage souffle plus tard) alors que le navigateur et le
test rejouent les TOUCHES enregistrées ; soulevé deux images plus tard, Pépin frappait le nuage plus
tard, rebondissait 2,4 px plus loin, et mourait dans le dernier motif. Dans l'arène,
`tient_au_decalage` rejoue maintenant les mêmes touches (boucle ouverte, `rejouer`) avec le nuage
décalé de 1 à 3 images, et exige la même place (1 px) et la même vitesse (5 px/s) à l'arrivée. Le test
est vert ; ailleurs (aiguilles du clocher et de l'horloge), rien ne change.

### Robot : les onze niveaux, avec et sans bonus

Sur l'extraction propre de ea83e42 :
- `le_robot_finit_tous_les_niveaux_sans_mourir` et `la_partie_complete_joue_le_bonus` : verts ;
- `ecrire_script_navigateur` (chemin SANS bonus : il manque des gouttes, on passe de 9 à 11) : 21 399
  images (356,6 s), 0 mort, 16 gouttes, score 2 500, empreinte 9fcab605c6e802a7 ; niveaux 1 à 9 puis
  11, écran de fin « DE RETOUR ! » à l'image 21 158 ;
- `ecrire_script_partie_complete` (chemin AVEC bonus : toutes les gouttes, le bonus après le 9) :
  26 173 images (436,2 s), 0 mort, 33 gouttes, score 4 200, empreinte e226eb7090319702 ; niveaux 1 à
  11, écran de fin à l'image 25 932 ;
- ces deux scripts ont été rejoués dans le navigateur (build de QA) pour les captures ci-dessous : les
  écrans de fin affichent bien 16/30 sans ligne de bonus, et 30/30 avec « NIVEAU SECRET 3/3 ».

Par rapport à la carte 5.17 (chemin sans bonus) : le clocher dure 3 images de plus (la pointe à
sauter), le boss 112 de plus (motifs plus larges, attentes plus longues) ; la mer (malgré la rive plus
loin), les vents, le final et l'épilogue durent exactement autant d'images qu'avant.

### Textes

- Écran titre : sous-titre « De l'orage jusqu'au soleil, et retour » (avant : « Grimpe de l'orage
  jusqu'au soleil », qui ignorait l'épilogue).
- Aide (« COMMENT JOUER ») : six astuces au lieu de quatre, dont « Sous la grêle, garde le parapluie
  ouvert ! » (la seule règle des nouveautés à savoir d'avance) et « Vent, auvent, horloge : aucune
  touche en plus ! » ; commandes resserrées (12 px au lieu de 13), astuces tous les 10 px : la dernière
  finit avant la ligne « retour ». Test `les_astuces_tiennent` (48 caractères au plus, une ligne).
- Écran de fin : inchangé (vérifié sur les captures des deux chemins ; il a été revu par la carte 5.17).
- README : les onze niveaux et la courbe de difficulté (section 1), « Notions apparues avec les onze
  niveaux » (section 7 : tranche constante dont la longueur sert de taille de tableau, données
  constantes imbriquées `MOTIFS`, `enum` privé derrière une structure publique, machines à états en
  entiers, `std::thread::scope`, fonctionnalités Cargo et `#[cfg(feature)]`), poids du wasm (section 10).

### Fonds : aucune silhouette « où se poser » dans une trouée

Note du coordinateur (captures 515-plane-porte et 515-grele) : dans la mer de nuages, les bancs roses
du fond avaient un sommet plat à la hauteur des quais, dans les vides mortels. Corrigé dans la table
d'ambiance (`decor::ambiance`, `Theme::Mer`, relief 10 -> -16), sans toucher à la physique ni à la
mesure : leurs sommets sont à y = 150 au plus haut, sous le quai le plus bas du niveau (y = 144). Une
première valeur (0, comme au niveau 9) laissait encore les bancs à la hauteur du quai bas de l'arrivée
du moment fort (capture de vérification), d'où -16. Test
`les_bancs_de_nuages_du_fond_restent_sous_la_ligne_du_sol` : pour chaque thème à bancs de nuages, sous
la ligne du sol ; et pour chaque niveau d'un écran de haut à bancs de nuages (au-dessus des nuages, la
mer, l'arc-en-ciel), au moins 4 px sous le plus bas des dessus où l'on se pose (tuiles, friables,
nacelles). Captures avant/après : `518-mer-plane-avant` / `518-niveau08-mer-plane`,
`518-mer-grele-avant` / `518-niveau08-mer-grele`, `518-mer-rive-avant` / `518-niveau08-mer-rive`.

Les autres thèmes, regardés sur les captures : toits, gouttières, marché et clocher (immeubles sombres,
toits en gradins et cheminées) ; orage et vents d'altitude (tours hautes, VOULUES par l'audit de
visibilité : test `la_couche_lointaine_monte_a_40_px_au_moins_dans_chaque_theme`) ; le boss (aucun
vide) ; l'épilogue (ciel du voyage, ville en bas). Dans l'orage et les vents, des tours sombres montent
derrière les vides à la hauteur des plateformes : elles sont nettement plus sombres que les nuages et
les dalles du niveau (blancs, cernés de noir) et défilent en parallaxe ; je ne les ai pas changées (ce
serait revenir sur la décision de l'audit), mais c'est à juger par un humain (`518-niveau04-orage`,
`518-niveau07-vents`).

### Poids

`./build.sh` sur l'extraction propre (octets ; gzip -9 entre parenthèses) :

| Fichier | Avant (56b07c4, carte 5.17) | Après (ea83e42) | Écart |
| ------- | --------------------------- | ----------------- | ----- |
| game.wasm | 491 283 (197 501) | 491 395 (197 595) | + 112 |
| index.html | 14 693 (5 617) | 14 693 (5 617) | 0 |
| mq_js_bundle.js | 36 148 (9 839) | 36 148 (9 839) | 0 |
| dist/ (total) | 542 124 | 542 236 | + 112 |

Build de QA (`dist-qa/`, raccourcis de niveaux) : wasm 501 226 -> 501 338 octets. Sur toute l'étape 5
(depuis la fin de l'étape 4, 73f844b, cinq niveaux : wasm 442 789 octets, dist/ 493 630) : + 48 606
octets de wasm (+ 11 %) pour six niveaux, neuf mécaniques, leurs sprites et leurs thèmes ; dist/ fait
0,54 Mo, sous l'avertissement de `build.sh` (1,2 Mo).

### Images par seconde

Outil `docs/mesure-images-par-seconde.mjs` (nouveau) : rejoue le script du robot jusqu'à un point du
parcours, puis joue la suite EN TEMPS RÉEL, une image du jeu par rappel de `requestAnimationFrame`
(comme la boucle normale), pendant 10 s ; il compte les images par seconde réelle et chronomètre
chaque image du jeu (simulation + dessin, `wasm_exports.frame()`, avec `performance.now()`).
Chromium 1208 sans tête, 960 x 540, rendu WebGL logiciel (SwiftShader : pas de carte graphique), build
de QA de l'extraction propre, servi sur 127.0.0.1.

Chaque point : 10 s de jeu réel (600 images attendues à 60 Hz) ; avant = build de QA de 56b07c4 (fin de
la carte 5.17), après = ea83e42, aux mêmes endroits du parcours (après le boss, 115 images plus tard).

| Point du parcours | Avant : i/s, coût moyen / pire (ms) | Après : i/s, coût moyen / pire (ms) |
| ----------------- | ----------------------------------- | ----------------------------------- |
| 1. Les toits | 60,0 ; 0,57 / 1,2 | 60,0 ; 0,56 / 1,0 |
| 2. Gouttières | 60,0 ; 0,61 / 1,6 | 60,0 ; 0,60 / 2,1 |
| 3. Le marché | 60,0 ; 0,59 / 1,0 | 60,0 ; 0,39 / 0,8 |
| 4. L'orage | 60,1 ; 0,53 / 4,3 | 60,1 ; 0,54 / 1,0 |
| 5. Le clocher | 60,0 ; 0,60 / 1,1 | 60,1 ; 0,68 / 2,5 |
| 6. Le boss | 60,0 ; 0,33 / 0,5 | 60,0 ; 0,34 / 0,8 |
| 7. Vents d'altitude | 60,1 ; 0,35 / 0,7 | 60,0 ; 0,40 / 1,7 |
| 8. La mer (premier plané, grêle) | 60,1 ; 0,92 / 1,8 | 60,0 ; 0,87 / 2,7 |
| 9. Au-dessus des nuages | 60,0 ; 0,30 / 0,5 | 60,0 ; 0,31 / 0,5 |
| 10. L'arc-en-ciel (partie complète) | non mesuré | 60,0 ; 0,55 / 1,0 |
| 11. La descente | 60,0 ; 0,54 / 0,8 | 60,1 ; 0,55 / 1,9 |

Le niveau le plus chargé est la mer de nuages (traînées de vent, grêle, montgolfières), et dans la mer
son moment fort (trois couches de vent superposées) : 20 s de jeu réel à cet endroit, 60,0 i/s avant
comme après (coût moyen 1,26 ms, 95e centile 3,7 à 4,2 ms, pire image 5,0 ms après, 7,5 avant). Aussi
mesurés 20 s : la cage du clocher (60,0 i/s ; 0,63 / 5,8 ms) et le motif 2 du boss (60,0 ; 0,95 /
7,5 ms). Le pire coût d'une image, partout, reste sous la moitié des 16,7 ms d'une image à 60 Hz.

Honnêteté sur le bruit : la machine est un serveur partagé (charge de 7 à 25 pendant les mesures). Une
première série, lancée pendant que les tests tournaient, a donné 56,0 i/s au marché et 58 au clocher, et
une mesure du moment fort de la mer, la charge montant à 25, 55,0 i/s (coût moyen 2,1 ms, pire 14,9) :
refaites machine plus calme, elles donnent 60,0. Les images perdues venaient donc de la machine, pas du
jeu (son coût par image n'a jamais dépassé 15 ms) ; mais je n'ai pas de mesure sur une machine dédiée.

### Captures de chacun des onze niveaux

Build de QA (`./build.sh --qa`) d'une extraction propre de ea83e42, servi sur 127.0.0.1 (port
libre, serveur arrêté à la fin), Chromium 1208 sans tête piloté par CDP, onglet au premier plan
(`Page.bringToFront`) et canvas focalisé, pilote `docs/capture-visibilite.mjs`, scripts
`target/qa/script.txt` (sans bonus) et `script-516.txt` (partie complète) de la même extraction ;
numéros d'image de `trace.csv` et `trace-516.csv`. 960 x 540 (agrandissement x3). Chaque capture
regardée ; `docs/mesure-capture.py <capture> 3` : 0 pixel hors palette, 0 pixel semi-transparent,
0 bloc flou sur les 23.

- `518-titre` : « PÉPIN », « De l'orage jusqu'au soleil, et retour » sur une ligne, le menu ;
- `518-aide` : les cinq commandes et les six astuces, la dernière au-dessus de « Entrée ou Échap :
  retour », rien ne se chevauche ;
- `518-niveau01-toits` (1200), `518-niveau02-gouttieres` (3600), `518-niveau03-marche` (5800),
  `518-niveau04-orage` (8000) : inchangés par cette carte (contrôle) ;
- `518-niveau05-clocher-pointe` (10677) : la cage de l'horloge, la pointe au sol juste avant la
  lanterne de la porte, Pépin qui l'a sautée, la grande aiguille ;
- `518-niveau06-cyclone-motif` (11990), `518-niveau06-cyclone-coup` (12223 : Pépin retombe sur la
  tête du nuage, premier coup), `518-niveau06-cyclone-dernier-coup` (13150),
  `518-niveau06-cyclone-dechirure` (13200 : le ciel se déchire) ;
- `518-niveau07-vents` (14500) ;
- `518-niveau08-mer-plane` (16195), `518-niveau08-mer-grele` (16510), `518-niveau08-mer-rive`
  (17475 : la fin du moment fort, la rive) ; les AVANT, du build de QA de 56b07c4 aux mêmes endroits du
  parcours (115 images plus tôt : le boss était plus court) : `518-mer-plane-avant` (16080),
  `518-mer-grele-avant` (16395), `518-mer-rive-avant` (17360) ; avant, les bancs roses du fond
  montent au-dessus des quais, dans les vides ; après, ils restent sous le quai le plus bas ;
- `518-niveau09-nuages` (19000), `518-niveau11-descente` (20500) ;
- `518-fin-sans-bonus` (21300) : « DE RETOUR ! », 16/30, pas de ligne de bonus, le mot de fin qui
  annonce le niveau secret ;
- partie complète : `518-niveau10-arc-en-ciel` (22500), `518-fin-complete` (26100) : 30/30,
  « NIVEAU SECRET 3/3 », « Collection complète : rien n'est oublié ! ».

### Vérification sur extraction propre

Sur une extraction propre de ea83e42 (dernier commit de code ; `git archive`, nouveau dossier daté) :
- `./docker-cargo.sh test --workspace --no-default-features` : 353 (core) + 315 (game) tests, tous
  verts (1 + 18 ignorés : écritures de fichiers, scripts, diagnostics, mesure), en 4 min 17 s et
  10 min 57 s (le robot joue les onze niveaux ; la courbe est mesurée en entier) ;
- `./docker-cargo.sh lint` : `LINT-OK` ;
- `./build.sh` : `dist/` produit (491 395 octets de wasm), 0 occurrence des noms interdits, seule URL
  celle de l'espace de noms SVG de l'icône ; « La mer de
  nuages » est dans `dist/game.wasm`, les niveaux d'essai non (« Essai vent » : 0 fois ; 2 dans
  `dist-qa/`) ;
- `./build.sh --qa` et les deux scripts du robot (`ecrire_script_navigateur`,
  `ecrire_script_partie_complete`) : même extraction ; scripts identiques à l'octet à ceux de b8fb043 et
  75173be (les retouches de fond suivantes ne touchent que le rendu) ; rejoués pour les captures et
  pour les images par seconde.
- Planche de sprites : aucun sprite nouveau ni modifié, non régénérée.
- `docs/mesure-difficulte.txt` : régénéré par `mesure_de_difficulte` sur le code de la carte (graine
  officielle) ; aucun commit de niveau, d'ennemi ou de mécanique après lui (les commits suivants :
  fonds, captures, outil d'images par seconde, README, QA.md).

Commits de la carte (dans l'ordre) : 1887e6b (mer : rive plus loin), 5fe21b3 (clocher : pointe de la
cage), a1ad88a (mer : bancs du fond, relief 0), 12a792c (textes : titre, aide), d27f440 (format),
9975831 (boss : motifs élargis ; robot : décalage en boucle ouverte), b8fb043 (juge de la courbe et
mesure), 75173be et ea83e42 (mer : bancs du fond, relief -8 puis -16), captures 518-, outil d'images par
seconde, README et QA.md.

### Ce qui n'est PAS testé (liste honnête)

- Aucun vrai téléphone ni tablette, ni Safari ni Firefox : Chromium 1208 sans tête seulement, à
  960 x 540, rendu WebGL logiciel. Le tactile n'a pas été rejoué sur les nouveaux niveaux.
- Les images par seconde sont mesurées dans ce Chromium sans tête, sur cette machine ; pas sur un
  téléphone d'entrée de gamme ni avec une vraie carte graphique (qui ferait sans doute mieux que le rendu
  logiciel). L'outil mesure le coût d'une image, pas le temps d'affichage réel à l'écran.
- Le son n'a été écouté par personne (aucun son nouveau dans cette carte).
- Aucun humain n'a joué les onze niveaux à la suite. La difficulté n'est jugée que par la mesure (un
  joueur simulé, une graine ; bruit de l'ordre de 0,1 à 0,2 mort) : l'écart entre la mer (0,46) et les
  vents (0,40), entre le clocher (0,69) et l'orage (0,65), est du même ordre que ce bruit ; seul le boss
  (1,19) est nettement au sommet. `bruit_de_la_mesure` (4 graines) n'a pas été relancé.
- La pointe du clocher et les motifs élargis du boss sont jugés « plus durs » par la mesure ; s'ils sont
  « justes » ou « injustes » pour un enfant ou un débutant (le Débutant simulé meurt 2,8 fois au clocher,
  4,1 fois au boss), seul un humain peut le dire.
- Le bonus reste plus meurtrier que le boss (Habitué 1,23 contre 1,19), voulu : il se juge au chrono
  (Expert 59 s), et il est réservé à ceux qui ont tout ramassé.
- Les tours sombres de l'orage et des vents d'altitude (voir « Fonds ») n'ont pas été jugées par un
  humain.
- La planche de sprites n'a pas été régénérée : aucun sprite nouveau ni modifié dans cette carte.
- Le niveau d'essai F10 (arène du Grand Nuage) a changé avec la table `MOTIFS` : son robot et son test de
  décalage sont verts, mais il n'a pas été recapturé.

## Corbeau redessiné (carte 6.1)

Demande (mot pour mot) : « j'aime pas le skin de la chauve souris ». C'est le CORBEAU, l'ennemi
volant des niveaux 4 (« Le grand orage ») et 7 (« Vents d'altitude ») : un corps rond et pâle entre
deux ailes en V, rien qui dise où est l'avant, se lisait comme une chauve-souris. Rendu seulement :
aucune boîte de collision, aucune physique, aucun niveau ne change.

Ce qui a été fait (`game/src/sprites/ennemis.rs`, `game/src/scene.rs`, `game/src/sprites/mod.rs`) :

- `CORBEAU0` / `CORBEAU1` redessinés en 18 x 14 (au lieu de 16 x 12), de profil, tournés vers la
  droite : gros bec gris clair (BRUME, NUAGE) devant, œil orange juste derrière, tête ronde au
  crâne éclairé, queue en éventail derrière, UNE aile qui bat depuis le dos : levée (image 0),
  puis rabattue sous le ventre (image 1), rémiges écartées en « doigts » au bout. Plumage bleu
  nuit (ARDOISE_NUIT), reflets ARDOISE_CLAIRE sur le dos et ARDOISE_PALE sur le bord de l'aile et
  le crâne ; contour sombre de 1 px. Le corps et la tête ne bougent pas d'une image à l'autre,
  seule l'aile bat. Rien du pigeon (cou irisé) ni de l'oiseau de décor (blanc).
- Plus grand de 2 px dans chaque sens : `scene.rs` le décale de (-3, -2) au lieu de (-2, -1), il
  reste centré sur la même boîte 12 x 10 (le miroir aussi).
- Contraste (WCAG, calculé) : sur le ciel d'ardoise de l'orage, le plumage seul ne se détache pas
  (1,4) ; ce sont ses 16 à 21 pixels clairs (reflets 3,7, bec 5,7 et 7,4, œil 6,4) qui le
  dessinent. Sur les bandes claires des vents (corail, corail clair, orange), c'est l'inverse : le
  plumage se détache à 5,1, 7,6 et 8,8 (l'ancien corbeau, gris pâle, n'y était qu'à 1,0 à 1,8).
- Test ajouté `le_corbeau_est_un_oiseau_de_profil_et_pas_une_chauve_souris` : taille 18 x 14 ;
  aucun pixel coloré ne touche le vide (contour) ; le bec (au moins 5 px) n'existe que devant
  (x >= 13) et c'est lui qui pointe le plus loin ; un seul œil, devant, au-dessus de la pointe du
  bec ; le bleu nuit est la couleur la plus fréquente, avec des reflets ARDOISE_PALE ; au moins 15
  px clairs sur l'ardoise ; plumage à 5:1 au moins sur les trois bandes claires des vents ; aucune
  couleur du pigeon ni de l'oiseau de décor ; colonnes 12 et suivantes (tête, bec) identiques sur
  les deux images. Le test existant `le_corbeau_se_detache_du_ciel_d_ardoise` reste vert sans
  changement de fond (son message parle maintenant de l'œil orange).
- `docs/planche-sprites.png` régénérée (seul le corbeau change).

Captures `docs/qa/61-*` (13 images, 960 x 540) : builds de QA (`./build.sh --qa`) d'une extraction
propre de a30f4ac (AVANT) et de 243592e (APRÈS, le dessin), servis sur 127.0.0.1 (ports libres,
serveurs arrêtés), Chromium 1208 sans tête piloté par CDP, onglet au premier plan
(`Page.bringToFront`) et canvas focalisé, pilote `docs/capture-visibilite.mjs`, même script du robot
(`script.txt`, partie sans bonus) pour les deux builds, mêmes images :

- niveau 4, orage (images 8506 et 8516 : le corbeau de la cage, entre les deux tours, à côté de
  Pépin) : `61-niveau04-orage-avant-aile-a`, `-avant-aile-b`,
  `61-niveau04-orage-apres-aile-haute`, `-apres-aile-basse` ;
- niveau 7, vents, premier corbeau (images 14161 et 14171, Pépin dans le courant juste dessous) :
  `61-niveau07-vents-courant-avant-a`, `-avant-b`, `-apres-a`, `-apres-b` ;
- niveau 7, vents, second corbeau avant le drapeau, avec le pigeon perché sur son petit nuage
  (images 15601 et 15611) : `61-niveau07-vents-pigeon-avant-a`, `-avant-b`, `-apres-aile-haute`,
  `-apres-aile-basse` ;
- `61-zoom-corbeau-pigeon-oiseau` : morceaux des VRAIES captures (prises en 1920 x 1080, pixels
  logiques relus un par un), agrandis x3. En haut : corbeau aile haute et aile basse (vents),
  pigeon (vents), oiseau de décor (la descente, image 20161). En bas, sur le ciel de l'orage :
  corbeau après (deux images), corbeau avant (deux images).

`docs/mesure-capture.py <capture> 3` sur les 13 : 0 pixel hors palette, 0 pixel semi-transparent, 0
bloc flou. Entre AVANT et APRÈS (prises en 1920 x 1080, pixels logiques comparés un par un), aux
images 8516, 15601 et 15611, seule la case du corbeau change (106 à 147 pixels logiques). À l'image
8506 de l'orage, deux passages du MÊME build diffèrent déjà (décor décalé d'un pixel, pluie) : ces
écarts-là ne viennent pas de cette carte.

Chaque capture regardée. Peut-on encore le prendre pour une chauve-souris ? Je ne le pense plus : de
profil, le bec clair désigne l'avant, l'œil est sur la tête, la queue en éventail dépasse derrière,
et une seule aile bat, au-dessus puis au-dessous du corps. Rien n'est symétrique : ni V, ni ailes
membraneuses, ni oreilles. À côté, le pigeon reste gris-bleu au cou turquoise, perché ; l'oiseau de
décor reste petit et blanc. AVANT, sur les mêmes images, c'était une tache gris pâle entre deux
ailes pointues, qu'on ne savait pas orienter.

Vérification sur une extraction propre de 086f853 (dernier commit de code ; `git archive`, nouveau
dossier daté) :

- `./docker-cargo.sh test --workspace --no-default-features` : 353 (core) + 316 (game, un de plus)
  tests, tous verts (1 + 18 ignorés), en 10 min 56 s ;
- `./docker-cargo.sh lint` : `LINT-OK` ;
- `mesure_de_difficulte` : `docs/mesure-difficulte.txt` régénéré, IDENTIQUE à l'octet à celui de
  a30f4ac (SHA-256 d5283ace…0ebe0e) : le dessin ne touche ni au jeu ni au robot ;
- scripts du robot (`ecrire_script_navigateur`, `ecrire_script_partie_complete`), empreintes
  inchangées : 9fcab605c6e802a7 (sans bonus, 21 399 images, 0 mort, 16 gouttes) et
  e226eb7090319702 (partie complète, 26 173 images, 0 mort, 33 gouttes) ;
- `./build.sh` : `dist/` produit, wasm 491 619 octets (491 395 à la carte 5.18 : +224 octets, deux
  grilles un peu plus grandes), 0 occurrence des noms interdits, seule URL celle de l'espace de
  noms SVG.

Commits : 243592e (dessin, décalage, test, planche), 086f853 (test : contrastes), captures 61-,
cette section.

Ce qui n'est PAS testé :

- Aucun humain n'a regardé le nouveau corbeau ; « on ne le prend plus pour une chauve-souris » est
  mon jugement sur les captures, à confirmer par celui qui l'a demandé.
- Sur le ciel d'ardoise de l'orage, le corbeau est maintenant SOMBRE sur sombre : il se lit par
  ses reflets, son bec et son œil, pas par sa masse (l'ancien, gris pâle, se voyait plus comme une
  tache, mais pas comme un oiseau). Il reste visible sur les captures ; un humain dira si c'est
  assez, pendant la pluie et en mouvement. Sur les ciels clairs des vents, il se voit mieux
  qu'avant.
- Chromium sans tête seulement, à 960 x 540 (et 1920 x 1080 pour le zoom) : ni vrai téléphone, ni
  Safari, ni Firefox.
- La boîte de collision (12 x 10) n'a pas changé ; le dessin la dépasse un peu plus (3 px de
  chaque côté au lieu de 2, 2 px en haut et en bas au lieu de 1) : le bout de l'aile ou de la
  queue peut sembler « frôler » Pépin sans le toucher. Rien n'a été mesuré là-dessus (la collision
  est celle d'avant, la mesure de difficulté est identique).

## Système de vies (carte 6.2)

Demande (mot pour mot) : « j'aimerai aussi que tu rajoutes un système de vies ». Avant, il n'y en avait
pas : on réapparaissait à la dernière lanterne et on comptait seulement les morts.

### Les règles (choix par défaut du coordinateur, réglables en une ligne)

Toutes dans `core/src/vies.rs`, en constantes nommées et commentées :

- 5 vies au début d'une partie (`VIES_DEPART`) ; une mort en retire une ; la réapparition à la lanterne
  (et sa seconde de protection) ne change pas ;
- la troisième goutte d'or d'un même niveau (toutes ses gouttes) donne une vie, UNE fois par niveau,
  jusqu'à 9 (`VIES_MAX`, un seul chiffre dans le HUD) ;
- à 0 : écran « PLUS DE VIES », puis le niveau EN COURS recommence depuis son début avec 5 vies ; les
  gouttes ramassées dans ce niveau sont perdues, celles des niveaux d'avant restent ; le chrono continue ;
  le compteur de morts garde tout ; le Grand Nuage repart avec ses trois points de vie (le niveau est
  rechargé tout neuf) ; la vie des trois gouttes déjà donnée dans ce niveau ne se regagne pas en les
  reprenant. Pas de sauvegarde ;
- le bonus « L'arc-en-ciel » ne coûte aucune vie (`coute_des_vies`) : sa carte de titre ajoute « Ici,
  tomber ne coûte aucune vie. » ; ses trois gouttes donnent quand même la vie. L'épilogue en coûte ;
- choix ajouté (non demandé, à valider) : « RECOMMENCER » du menu de pause ne rend PAS les vies perdues
  dans le niveau (sinon ce serait un « PLUS DE VIES » évité d'un geste) ; la vie des trois gouttes y
  reste donnée.

### Ce qui a été fait

- `core/src/vies.rs` (nouveau) : `Vies`, champs privés, méthodes `perdre`, `trois_gouttes`,
  `nouveau_niveau`, `repartir` ; 6 tests pas à pas (perte, dernière vie, gain une fois par niveau,
  plafond, après un game over, bonus).
- `core/src/state.rs` : `EtatJeu` tient les vies et les gouttes du niveau par index (la 3e donne la vie) ;
  `nouveau_niveau(depart, genre)` ; `recommencer_niveau` et `apres_plus_de_vies` construisent l'état du
  niveau recommencé. À 0 vie, `traiter` replace Pépin comme avant : c'est le jeu qui ouvre l'écran. La
  SIMULATION ne change donc pas (mesure de difficulté et robot identiques, voir plus bas). 6 tests.
- `game/src/app.rs` : nouvel écran `Ecran::PlusDeVies(pas restants)` et `Suite::PlusDeVies`. L'écran
  s'ouvre au pas où la dernière vie est perdue (avant la fin du niveau si les deux tombent ensemble), se
  passe d'un appui (Entrée, toucher) ou seul au bout de 4 s, puis les volets rechargent le niveau. Focus
  perdu ou touche de pause pendant l'écran : la pause est DIFFÉRÉE et s'ouvre sur le niveau recommencé,
  avant son premier pas (focus revenu entre-temps : pas de pause). 9 tests de la machine à états
  (cinq vies et une de moins par mort ; PLUS DE VIES puis le niveau 2 recommencé avec les bons compteurs,
  gouttes du niveau 1 gardées, chrono et morts conservés, 3 gouttes reprises sans 2e vie ; passer
  l'écran au clavier et au toucher ; focus perdu / pause / focus revenu ; Grand Nuage remis à 3 PV après
  un game over mais pas après une simple mort ; bonus sans perte et épilogue avec ; « +1 » au HUD et
  plafond à 9 ; RECOMMENCER ; vies du bilan).
- `game/src/ecrans/plus_de_vies.rs` (nouveau) : le niveau figé et tramé, un panneau « PLUS DE VIES »
  (corail, échelle 3), Pépin qui lève les yeux au ciel, « Le niveau recommence, avec 5 vies. » (le
  chiffre vient de `VIES_DEPART`) et « Ses gouttes d'or sont à reprendre. », puis « Entrée ou toucher :
  continuer » qui clignote. Mise en page vérifiée à la compilation (`const _`) et largeurs par un test.
- HUD (`game/src/hud.rs`) : après les gouttes, une icône 10 x 10 (tête de Pépin, `vie` dans
  `sprites/pepin.rs`, contour sombre) en x = 53 et le chiffre en x = 65, à place fixe (le compteur de
  gouttes le plus large, « 30/30 », finit en x = 46). Au gain d'une vie : chiffre en or et « +1 » doré
  qui monte de 3 px pendant une seconde. Tests : rien ne chevauche les gouttes ni les points de vie du
  Grand Nuage (de 0 à 9 vies, « +1 » compris) ; icône lisible sur TOUS les ciels (contour ou corps
  blanc à 4,5:1 au moins sur chaque bande de chaque thème) ; sur téléphone (844 x 340 et 844 x 390 à
  DPR 3, 667 x 375 à DPR 2, portrait, tablette), aucun bouton tactile ne recouvre les vies ni les
  gouttes.
- Aide : septième astuce « Trois gouttes d'un niveau : une vie de plus ! » ; commandes tous les 11 px
  (au lieu de 12), astuces à partir de y = 92 : la dernière finit en y = 160, « retour » en 163
  (vérifié à la compilation). Écran de fin : ligne « VIES » entre MORTS et SCORE quand elle tient avec
  2 px au-dessus du menu, lignes resserrées d'un pixel s'il le faut (11 -> 10 px) ; elle tient dans la
  partie normale (bonus manqué, bonus complet), pas quand le bonus est joué mais incomplet (mot de fin
  sur deux lignes) : la ligne est alors omise, comme demandé (« seulement si ça tient proprement »).
- `docs/planche-sprites.png` régénérée (l'icône `vie` s'ajoute).

### Mesure de difficulté : taux de game over

`game/src/difficulte.rs` note, pendant chaque essai, ce qui change les vies (morts, troisième goutte),
puis rejoue ces événements avec les règles du jeu (`rejouer_vies`). Les colonnes de morts ne changent
pas : `docs/mesure-difficulte.txt` régénéré commence par les 6 819 octets de celui de a30f4ac, IDENTIQUES
(SHA-256 d5283ace…0ebe0e pour l'ancien) ; seule une 4e partie s'ajoute à la fin. Essais où Pépin perd sa
dernière vie au moins une fois, niveau commencé avec 5 vies (48 essais par case) :

| Niveau                     | Débutant     | Habitué    | Expert     |
|----------------------------|--------------|------------|------------|
| 1. Les toits               | 0/48 (0 %)   | 0/48       | 0/48       |
| 2. Gouttières et cheminées | 3/48 (6 %)   | 0/48       | 0/48       |
| 3. Le marché couvert       | 0/48 (0 %)   | 0/48       | 0/48       |
| 4. Le grand orage          | 12/48 (25 %) | 0/48       | 0/48       |
| 5. Le clocher              | 13/48 (27 %) | 0/48       | 0/48       |
| 6. L'œil du cyclone        | 16/48 (33 %) | 1/48 (2 %) | 0/48       |
| 7. Vents d'altitude        | 2/48 (4 %)   | 0/48       | 0/48       |
| 8. La mer de nuages        | 0/48 (0 %)   | 0/48       | 0/48       |
| 9. Au-dessus des nuages    | 1/48 (2 %)   | 0/48       | 0/48       |
| 10. L'arc-en-ciel (bonus)  | 0/48 (aucune vie perdue) | 0/48 | 0/48 |
| 11. La descente            | 0/48 (0 %)   | 0/48       | 0/48       |

Partie enchaînée (estimation : les essais de même rang des dix niveaux du parcours bout à bout, vies
reportées d'un niveau au suivant ; après un game over, 5 vies et la suite de l'essai) : Débutant 47/48
parties avec au moins un game over (2,5 en moyenne, 5 au plus) ; Habitué 13/48 (0,3 en moyenne, 2 au
plus) ; Expert 1/48.

**À SIGNALER (rien n'a été retouché, décision d'AGA)** : trois niveaux normaux dépassent 10 % de game over
pour le Débutant : 4. Le grand orage (12/48, 25 %), 5. Le clocher (13/48, 27 %), 6. L'œil du cyclone
(16/48, 33 %). Ce sont les trois niveaux où le Débutant meurt le plus (3,0, 2,8 et 4,1 morts en moyenne,
pire essai 11, 11 et 14). Pistes possibles, non essayées : 6 vies au départ (une ligne), une vie rendue
à chaque lanterne, ou adoucir ces trois niveaux. À noter : le joueur simulé va au drapeau au plus court
et ne cherche pas les gouttes ; un joueur qui les ramasse gagne des vies, ces taux sont donc plutôt
pessimistes.

### Robot de QA

Les deux parties du robot finissent sans mourir, empreintes INCHANGÉES (comme à la carte 6.1) :
9fcab605c6e802a7 (sans bonus, 21 399 images, 0 mort, 16 gouttes) et e226eb7090319702 (partie complète,
26 173 images, 0 mort, 33 gouttes). Nouveau test ignoré `ecrire_script_vies` (`bot_qa/scripts.rs`) : la
partie « vies » à travers la vraie `Application` (niveau 1 avec ses trois gouttes : 6 vies ; niveau 2,
un joueur têtu qui court tout droit et saute quand il bute : six chutes, PLUS DE VIES à l'image 3331 ;
le niveau 2 recommence à l'image 3595 avec 5 vies et 3 gouttes ; puis le robot jusqu'à la fin) :
22 526 images, 6 morts, 7 vies à la fin (deux autres « +1 », au boss et à la mer), 18 gouttes.

### Captures `docs/qa/62-*` (25 images)

Build de QA (`./build.sh --qa`) d'une extraction propre de 8b203ce, servi sur 127.0.0.1 (port libre,
serveur arrêté), Chromium 1208 sans tête piloté par CDP, onglet au premier plan (`Page.bringToFront`) et
canvas focalisé, pilote `docs/capture-visibilite.mjs` (scripts `script-62.txt` et, pour la carte du
bonus, `script-516.txt`). Le pilote accepte maintenant `DPR`, `MOBILE=1` et une demande `doigt=IMAGE`
(un doigt posé au milieu de l'écran, où il ne commande rien : le jeu montre ses boutons tactiles).

- HUD sur neuf ciels : toits (`62-niveau01-toits-cinq-vies`), orage, clocher, cyclone (avec les points de
  vie du Grand Nuage), vents, mer, soleil, descente (`62-niveau04-…` à `62-niveau11-…`), et le zoom
  `62-zoom-hud-sept-ciels` (bandes du haut de sept captures, pixels d'origine) ;
- gain d'une vie : `62-niveau01-gain-vie-a` (2 images après la 3e goutte : « 6 » en or et « +1 » encore
  bas), `-b` (« +1 » monté), `-fin` (fini : « 6 » blanc), et au boss et à la mer ;
- vies qui baissent : `62-niveau02-deux-vies`, `62-niveau02-derniere-vie` ; le zoom
  `62-zoom-hud-vies-qui-changent` (5, 6+1, 6+1, 6, 2, 1, puis 5 après le recommencement) ;
- `62-plus-de-vies` ; `62-niveau02-recommence` (Pépin au départ du niveau 2, 3/30, 5 vies, chrono qui a
  continué) ;
- `62-aide`, `62-carte-bonus` (la ligne « Ici, tomber ne coûte aucune vie. »), `62-fin` (TEMPS, GOUTTES
  D'OR 18/30, MORTS 6, VIES 7, SCORE) ;
- téléphone en paysage 844 x 340, DPR 3 (captures de 2532 x 1020, facteur 5) avec les boutons tactiles :
  `62-tel844x340-niveau01-gain-vie`, `-plus-de-vies`, `-niveau02-recommence`, `-niveau06-cyclone`. Les
  boutons (flèches en bas à gauche, saut en bas à droite, pause en haut à droite, hors du jeu) ne
  touchent pas le HUD.

`docs/mesure-capture.py <capture> K` (K = 3, 5 pour le téléphone) : 0 pixel hors palette, 0 pixel
semi-transparent, 0 bloc flou sur les 21 captures de bureau et sur `62-tel844x340-plus-de-vies`. Sur les
trois autres captures du téléphone, 35 835 pixels hors palette et 205 blocs « flous » : ce sont les
boutons tactiles translucides qui débordent sur la zone de jeu (même chiffre exact qu'à la carte
« clavier, écran haute densité ») ; recompté en masquant les quatre boutons (leur rectangle calculé comme
`tactile::Disposition`, en pixels CSS x 3) : 35 835 sous les boutons, 0 ailleurs. Les deux zooms : 0
pixel hors palette.

Chaque capture regardée. Le compteur se lit sur tous les ciels (la tête blanche cernée de sombre ; le
chiffre blanc à ombre). Au clocher, une lanterne du décor passe juste au-dessus de l'icône sans gêner la
lecture. Sur les ciels chauds, le « +1 » et le chiffre DORÉS du gain se détachent surtout par leur ombre
sombre : or sur corail 2,2:1, sur corail clair 1,5:1, sur orange 1,3:1 (l'ombre, elle, y est à 6,0, 8,9
et 10,3:1) ; sur la capture de la mer (`62-niveau08-mer-gain-vie`, « 7 +1 » sur le corail), on les lit,
mais c'est le cas le moins contrasté. À part : au cyclone, le compteur de GOUTTES (inchangé) passe devant
la colonne de neige blanche à gauche et s'y lit mal ; l'icône des vies, plus à droite, est dégagée.

### Vérification sur une extraction propre de 8b203ce (dernier commit de code)

`git archive`, nouveau dossier daté sous `$TMPDIR` :

- `./docker-cargo.sh test --workspace --no-default-features` : 365 (core, + 12) et 335 (game, + 19)
  tests, tous verts (1 + 19 ignorés), en 10 min 50 s ;
- `./docker-cargo.sh lint` : `FMT-OK`, `LINT-OK` ;
- `mesure_de_difficulte` : fichier identique à l'octet à celui du dépôt (SHA-256 57e47b14…ccad531), dont
  les 6 819 premiers octets sont ceux de a30f4ac ;
- `ecrire_script_navigateur`, `ecrire_script_partie_complete`, `ecrire_script_vies` : empreintes et
  chiffres ci-dessus ; `script-62.txt` identique à celui des captures ;
- `./build.sh` : `dist/` produit, wasm 494 519 octets (491 619 à la carte 6.1 : + 2 900), 0 occurrence des
  noms interdits, seule URL celle de l'espace de noms SVG.

Commits : 3e21f3c (cœur), 39ccdbb (jeu : écran, HUD, aide, carte, fin), 2dd129f (mesure), 8b203ce
(script du robot, test des boutons tactiles), 2c88764 (captures, pilote), cette section et le README.

### Ce qui n'est PAS testé

- Aucun humain n'a joué avec les vies : le réglage (5 vies, +1 aux trois gouttes) est un choix par
  défaut, et trois niveaux dépassent 10 % de game over pour le Débutant simulé (ci-dessus).
- Le taux de game over vient du joueur simulé de la mesure, qui ne recommence pas le niveau : le
  « game over » est compté au premier passage (exact) ; la partie enchaînée est une estimation.
- Chromium sans tête seulement (bureau 960 x 540 et téléphone émulé 844 x 340 à DPR 3) : ni vrai
  téléphone, ni Safari, ni Firefox ; le son n'a pas été écouté (aucun son nouveau n'a été ajouté).
- Défaut ANTÉRIEUR trouvé par le nouveau test des boutons tactiles, non corrigé (hors de cette carte) :
  sur un 667 x 375 à DPR 2 (jeu agrandi x4, presque tout l'écran), le bouton de pause recouvre le chrono
  en haut à droite. Les vies et les gouttes, elles, restent dégagées partout.

## Chats plus menaçants : réveil plus rapide, bond sur Pépin (carte 6.3)

Demande d'AGA : « Garde les chats mais rends les plus impactant, ils se réveillent plus vite et saute
sur pépin ». Les règles (`core/src/entities/ennemi/chat.rs`, chaque durée et chaque portée est une
constante nommée et commentée) :

- cycle : il dort 100 pas (1,7 s, `DORT_PAS` ; 240 avant), s'annonce 48 pas (0,8 s, le « ! » doré qui
  clignote de plus en plus vite, `REVEIL_PAS`, inchangé), reste éveillé 72 pas (1,2 s, `EVEILLE_PAS`,
  inchangé) : 3,7 s au lieu de 6 ; les premiers sommeils des chats d'un niveau sont raccourcis de 0, 30,
  60 ou 90 pas (0, 40, 80, 120 avant) ;
- pendant l'annonce, s'il voit Pépin (`voit_pepin` : au plus 5 tuiles entre leurs centres en x,
  `PORTEE_BOND_X` ; au plus 2 tuiles entre leurs pieds en hauteur, `PORTEE_BOND_Y` ; le segment entre
  leurs centres ne traverse aucune tuile solide ni aucun solide d'entité) ET qu'un bond est possible, il
  se ramasse : posture « prêt à bondir » (image `chat_pret`), tourné vers Pépin ;
- au premier pas de l'éveil, même question avec la position de Pépin à ce moment : s'il le voit, il
  bondit vers l'endroit où se tient Pépin (`cible_du_bond`, son centre sous celui de Pépin). Le bond est
  un arc à la gravité de Pépin (`decalage_bond` : vitesse verticale `VITESSE_BOND_Y` = 225 px/s, 28 px de
  haut, 30 pas pour l'aller, `BOND_PAS`), une pose de 12 pas au bout (`POSE_BOND_PAS`), puis le même arc
  à l'envers : au dernier pas de l'éveil il est PILE à sa place, et il se rendort. Hors de portée (ou
  sans bond possible), il se réveille sur place comme avant ;
- le bond ne quitte jamais son toit : il est raccourci (au pixel) jusqu'à ce que toutes les colonnes
  survolées aient un solide sous les pieds du chat (il s'arrête au bord du vide) ; puis, de 2 px en 2 px,
  jusqu'à ce qu'aucun pas de l'arc ne touche une tuile solide ou un solide d'entité (il s'arrête contre un
  mur, un plafond trop bas l'empêche de bondir) ; le retour repasse par les mêmes boîtes ;
- éveillé (sur place, en l'air, posé au bout du bond, en revenant), le chat ne s'écrase pas : le toucher
  tue, même par le dessus ; rendormi, il redevient écrasable (sommeil et annonce, comme avant) ;
- la trajectoire est une fonction PURE du temps (`decalage_chat(phase, bond)`), comme la plongée du
  pigeon : rien ne s'accumule, le chat revient exactement à sa place. Le chat retient la boîte de Pépin à
  chaque pas (`Ennemi::guetter`, appelé par `resoudre_contacts` comme pour le pigeon) : le jeu, le robot
  de QA et la mesure de difficulté ont tous le même chat, sans rien ajouter.

### Pourquoi 1,7 s de sommeil et non 2 s

La carte proposait `DORT_PAS` ≈ 120 (2 s). Avec 120, la mesure officielle (graine 20261113) donne pour
l'Habitué 0,33 mort au marché couvert, sous les gouttières (0,50) : le juge de la courbe
(`ecarts_de_la_courbe`, « la courbe ne redescend jamais avant le boss ») refuse. Avec 110 : 0,46, refusé
aussi. Avec 100 : 0,54, vert. Consigne de la carte : ne pas toucher au juge ; le sommeil a donc été
raccourci à 100 pas (dans le sens de la demande : « ils se réveillent plus vite »), pas allongé.

À savoir : cet écart est du même ordre que le bruit de la mesure. Sur 6 graines (20261113 à 20261118, 48
essais chacune, test `mesure_d_un_niveau` avec une variable `GRAINES` ajoutée dans une copie jetable,
rien de commité), morts moyennes au marché couvert :

| Sommeil (`DORT_PAS`)          | Débutant | Habitué | Expert | Game over Débutant (6 x 48) |
| ----------------------------- | -------- | ------- | ------ | --------------------------- |
| 240, sans bond (avant)        | 0,94     | 0,36    | 0,11   | 7                           |
| 120                           | 0,89     | 0,32    | 0,10   | 4                           |
| 110                           | 0,92     | 0,37    | 0,13   | 6                           |
| 100 (retenu)                  | 0,99     | 0,37    | 0,06   | 8                           |
| 90                            | 0,97     | 0,33    | 0,10   | 9                           |

Gouttières (niveau 2), Habitué, mêmes graines : 0,53 en moyenne. Sur 6 graines, le marché couvert
était donc DÉJÀ sous les gouttières pour l'Habitué avant cette carte ; le juge ne regarde que la graine
officielle, où il passait de justesse (0,5 contre 0,5, arrondis au dixième). À signaler au coordinateur pour la carte 6.4a
(plus d'ennemis aux niveaux 1 à 5), qui touchera aussi ces deux niveaux.

Pourquoi le chat plus menaçant pèse si peu dans la mesure : le joueur simulé a une intention PARFAITE
(il connaît le cycle du chat et l'attend hors de portée, comme le robot de QA) et ne rate que des gestes
(élan, durée du saut, plané). Le bond punit surtout le joueur qui s'arrête près d'un chat sans regarder
le « ! » : c'est exactement ce que la mesure ne simule pas. Les morts « ennemi » (flaques et chats) du
marché couvert montent quand même pour le Débutant (64 sur les 6 graines avant, 85 avec 100 pas ; sur la
graine officielle, colonnes 112 à 115, le chat du pied du bloc 116, en tête des morts « ennemi »).

### Mesure de difficulté : avant / après

Commande officielle (`mesure_de_difficulte`), graine 20261113, 48 essais. Avant : 88b2bce (version
publiée) ; après : d65c9dc. Seul le marché couvert change ; les dix autres niveaux sont identiques ligne
à ligne dans `docs/mesure-difficulte.txt` (morts, détail, lieux de mort, game over), sauf les lignes qui
les additionnent (Total, partie enchaînée du Débutant).

| 3. Le marché couvert   | Débutant           | Habitué            | Expert             |
| ---------------------- | ------------------ | ------------------ | ------------------ |
| Morts moyennes, avant  | 0,8                | 0,5                | 0,2                |
| Morts moyennes, après  | 1,1                | 0,5 (0,54)         | 0,1                |
| Game over, avant       | 0/48 (0 %)         | 0/48 (0 %)         | 0/48 (0 %)         |
| Game over, après       | 3/48 (6 %)         | 0/48 (0 %)         | 0/48 (0 %)         |
| Pire essai avant/après | 4 / 7 morts        | 4 / 3              | 2 / 2              |
| Durée avant/après      | 33 s / 33 s        | 32 s / 30 s        | 28 s / 26 s        |

Total des morts : Débutant 24,8 -> 25,1, Habitué 5,9 -> 5,9, Expert 1,8 -> 1,7. Partie enchaînée,
Débutant : 47/48 parties avec au moins un game over (inchangé), 2,5 -> 2,6 en moyenne. Le marché couvert
reste sous les 10 % de game over pour le Débutant (rien de nouveau à signaler). Le juge de la courbe
(`courbe_de_difficulte_dans_les_bandes`) est vert : Habitué, marché couvert 0,54 dans sa bande (0,3 à
1,2) et au-dessus des gouttières (0,50).

### Robots

- Robot de faisabilité (`core/src/faisabilite.rs`) : il garde le chat TOUJOURS endormi (un joueur patient
  attend son sommeil), mais connaît maintenant le bond : il ne retient aucun état posé là où un chat
  vivant bondirait sur lui (`chat_bondirait`, d'après `Ennemi::bond_atteint` : le chat le voit, un bond
  est possible, et sa boîte touche celle de Pépin à un pas au moins de l'éveil). Il passe, ou il écrase le
  chat, d'un seul plan. C'est prudent : un vrai joueur peut s'y poser un instant pendant le sommeil. Test
  `le_robot_ne_s_arrete_pas_la_ou_le_chat_bondirait`. Le marché couvert et l'essai chat restent faisables
  (robot précis et maladroit, depuis le départ et depuis chaque lanterne).
- Robot de QA (`game/src/bot_qa/parcours.rs`) : vraie simulation, le bond y est donc joué tel quel (un plan
  qui finit sous le chat est une mort, écartée). La case retient en plus si le chat proche est en plein bond
  (`chats_de_la_case` : 32 x écrasés + 2 x tranche + bond ; proximité mesurée depuis sa PLACE) ; tranche du
  cycle de 30 pas (8 tranches). Les deux parties (sans et avec bonus) finissent les onze niveaux sans
  mourir (`le_robot_finit_tous_les_niveaux_sans_mourir`, `la_partie_complete_joue_le_bonus`). Essai chat
  (F7) : fini sans mourir, 3 gouttes ; le robot arrive sur le bloc pendant le 2e sommeil et saute aussitôt
  (il n'a plus à y attendre l'éveil) ; chat écrasé pendant son annonce (`Reveil(31)`), jamais de justesse.
- Lanternes sur le trajet d'un danger (`zones::lanternes_sur_le_trajet_d_un_danger`) : le test calcule
  aussi, lanterne par lanterne, si un chat bondirait sur Pépin réapparu immobile. Deux de plus au marché
  couvert : colonne 50 (chat de la colonne 54) et colonne 110 (chat de la colonne 114). Pas un piège : la
  protection de réapparition (1 s) couvre tout bond déjà parti, le bond suivant vient au plus tôt 3,7 s
  plus tard, annoncé 0,8 s par le « ! » ; au plus une mort par réveil pour un joueur qui reste planté à la
  lanterne. Aucune lanterne déplacée. Liste : (2, 66, flaque), (2, 104, flaque), (3, 50, bond de chat),
  (3, 110, bond de chat), (4, 145, flaque).

### Tests du cœur (mini-niveaux pas à pas, `ennemi/chat.rs`)

- `le_cycle_dure_ce_qu_annoncent_les_constantes` (100 / 48 / 72 / 220) ;
- `la_trajectoire_du_bond_est_un_arc_qui_revient_pile_a_sa_place` ;
- `il_ne_voit_pepin_qu_a_portee_et_sans_mur_entre_eux` (5 tuiles, 2 tuiles, mur, solide d'entité) ;
- `a_portee_il_se_prepare_puis_bondit_et_tue_pepin` (posture, cible au pixel, mort pendant l'aller) ;
- `hors_de_portee_il_se_reveille_sur_place` (deux cycles, boîte immobile) ;
- `le_bond_s_arrete_au_bord_du_toit` (33 px au lieu de 78 : bord du toit), `le_bond_ne_traverse_pas_un_mur`
  (48 px contre une marche ; plafond trop bas : ni posture ni bond) ;
- `aucun_bond_ne_traverse_un_solide_ni_ne_saute_dans_le_vide` (Pépin à chaque pixel de quatre mini-niveaux) ;
- `en_plein_bond_il_tue_par_le_dessus_rendormi_il_s_ecrase`.
- Côté jeu : `le_chat_a_une_image_par_phase` (posture et vol), `le_chat_a_sept_images_de_meme_taille_et_un_contour_sombre`,
  `le_chat_eveille_a_les_yeux_ouverts`, `pret_a_bondir_et_en_plein_bond_se_reconnaissent`.

### Lisibilité

Deux images de plus (16 x 14, palette, contour sombre d'1 px, mêmes couleurs que le chat : orange, corail,
blanc, or) : `chat_pret` (tête basse posée sur les pattes avant, yeux d'or grand ouverts, croupion levé et
queue dressée, au lieu de l'annonce simple couchée, un œil ouvert) et `chat_bond` (corps allongé, pattes
avant tendues vers l'avant, pattes arrière vers l'arrière). Posé au bout du bond : l'image éveillée
(assis, hérissé). Le chat se tourne vers Pépin quand il est prêt, puis dans le sens du saut (aller,
retour). Planche de sprites régénérée. Astuce de la carte du marché couvert : « Tombe sur un auvent : il te
relance. Écrase le chat endormi : au « ! », il bondit. » (deux lignes, dans le cadre) ; intro de l'essai
chat : « … réveillé, il bondit sur toi. »

### Captures `docs/qa/63-*` (14 images)

Build de QA d'une extraction propre de d65c9dc (`./build.sh --qa`), servi en local sur 127.0.0.1 (port
libre, serveur arrêté à la fin), Chromium 1208 sans tête piloté par CDP, onglet au premier plan et canvas
focalisé, 960 x 540 (x3), pilote `docs/capture-visibilite.mjs` avec `SCRIPT=target/qa/script-63.txt` et
`TOUCHES=target/qa/touches-63.txt` (touche 3 à l'image 229 du pilote). Script écrit par le test ignoré
`ecrire_script_chats` (numéros d'image de `trace-63.csv`) : le premier chat du marché couvert (colonne 12),
Pépin d'abord au départ, à 11 tuiles :
- `63-carte-marche` (400) : la carte du niveau 3, astuce sur deux lignes dans le cadre ;
- `63-hors-portee-annonce` (571) : hors de portée, l'annonce simple (couché, un œil ouvert, « ! ») ;
- `63-hors-portee-eveille` (641) : il se réveille SUR PLACE (assis, hérissé), Pépin toujours au départ ;
- `63-approche-dort` (761) : rendormi (« z »), Pépin s'est approché à moins de 5 tuiles et s'arrête ;
- `63-pret-a-bondir` (791) et `63-pret-a-bondir-fin` (816) : il se ramasse, tête basse, croupion et queue
  levés, « ! » allumé, tourné vers Pépin ;
- `63-bond-aller` (833) : en l'air, pattes tendues, vers l'endroit où se tenait Pépin, qui s'enfuit ;
- `63-bond-pose` (859) : posé au bout du bond, assis, là où était Pépin (qui a fui à gauche) ;
- `63-bond-retour` (879) : le bond de retour, tourné vers sa place ;
- `63-rendormi` (901) : revenu à sa place, endormi ; 0 mort ;
- `63-reste-sur-place-00` à `03` (série « attente » du pilote, lâché à l'image 822 : Pépin ne bouge plus,
  une capture toutes les 8 images) : le chat bondit, lui retombe dessus ; dernière image : éclats, vies 5
  -> 4, secousse de l'écran.
Chaque capture regardée. `docs/mesure-capture.py <capture> 3` : 0 pixel hors palette, 0 bloc flou, sur les 14.

### Vérification sur une extraction propre de d65c9dc (dernier commit de code)

`git archive`, nouveau dossier daté sous `$TMPDIR` :

- `./docker-cargo.sh test --workspace --no-default-features` : 374 (core, + 9) et 336 (game, + 1) tests,
  tous verts (1 + 20 ignorés), dont `courbe_de_difficulte_dans_les_bandes`, les deux parties du robot et
  l'essai chat ;
- `./docker-cargo.sh lint` : `FMT-OK`, `LINT-OK` ;
- `./build.sh` : `dist/` produit, wasm 497 647 octets (494 519 à la carte 6.2 : + 3 128), 0 occurrence des
  noms interdits, seule URL `http://www.w3.org/2000/svg`, « Essai chat » absent de `dist/` ;
  `./build.sh --qa` (build des captures) : `dist-qa/` produit.
- Mesure de difficulté (`mesure_de_difficulte`, sur le dépôt au même contenu) : verte, fichier
  `docs/mesure-difficulte.txt` commité avec d65c9dc.

Commits : 6f79537 (cœur), 7b1bb25 (robots), 36e24ff (rendu, sprites, textes), 5eaf025 (script du
navigateur), d65c9dc (sommeil 100 pas, mesure), a59ef9b (captures), cette section et le README.

### Ce qui n'est PAS testé

- Aucun humain n'a joué le nouveau chat : la portée (5 tuiles), la durée du sommeil (1,7 s) et la pose au
  bout du bond (0,2 s) sont des choix ; la mesure, à intention parfaite, ne juge pas la surprise d'un
  joueur qui ne regarde pas le « ! » (voir plus haut).
- Le juge de la courbe passe sur la graine officielle seulement ; sur 6 graines, le marché couvert reste
  sous les gouttières pour l'Habitué, avant comme après cette carte.
- Pas de son propre au bond (aucun son ajouté) ; captures dans Chromium seulement, à 960 x 540 ; pas de
  vrai téléphone, ni Safari, ni Firefox.
- Le chat ne bondit que sur son toit (même rangée de sol) : un Pépin au-dessus d'un trou, ou sur un autre
  toit, est vu mais pas atteint (voulu : il ne tombe jamais dans le vide).

## Plus d'ennemis dans les niveaux 1 à 5 (carte 6.4a)

Demande d'AGA : « tu peux rajouter des ennemis sur les niveau, il y a beaucoup de niveaux avec peu
d'ennemis ». Le but est un CHALLENGE. Cette carte traite les niveaux 1 à 5 (la 6.4b fera 7, 8, 9 et 11) ;
le boss (6), le bonus (10), les mécaniques et la physique ne changent pas. Types existants seulement
(flaque `f`, corbeau `c`, pigeon `p`, chat `z`), aucune lettre nouvelle.

### Inventaire avant / après

| Niveau                        | Avant         | Après                 | Ajouts (colonne, rangée de la lettre)                       |
| ----------------------------- | ------------- | --------------------- | ----------------------------------------------------------- |
| 1. Les toits (176 col.)       | aucun         | 2 flaques             | flaques (76, 7) et (118, 7)                                 |
| 2. Gouttières (190 col.)      | 9 flaques     | 9 flaques, 2 corbeaux | corbeaux (50, 5) et (106, 5)                                |
| 3. Le marché couvert (153)    | 2 f + 4 chats | 2 f + 5 chats         | chat (68, 4), sur le mur de la goutte cachée                |
| 4. Le grand orage (200)       | 1 f + 1 c     | 1 f + 2 corbeaux      | corbeau (36, 5), au-dessus des pointes du deuxième toit     |
| 5. Le clocher (34 x 45)       | 2 pigeons     | 2 p + 1 chat + 1 c    | chat (28, 36) sur la galerie, corbeau (26, 18) sous la porte |

Soit 19 ennemis avant, 27 après (+ 8). Une retouche de niveau = un commit avec sa mesure (700b5f3
clocher, e1752dc grand orage, 2c87bf3 marché couvert, c1c9266 gouttières, fc0a97c toits, dans cet ordre :
du sommet vers le début, pour que chaque niveau soit réglé sous le suivant déjà retouché).

- Les toits : les deux premières flaques du jeu, APRÈS le dernier panneau (planer, colonne 55). Chacune
  garde un bout de toit FERMÉ, sans lanterne ni décor sur sa patrouille : le creux entre les deux
  cheminées de briques (colonnes 74 à 79 : on lui tombe dessus, ou on saute d'une cheminée à l'autre en
  planant) et le petit toit du pied de l'immeuble (colonnes 116 à 120, juste avant le trou de deux
  tuiles : on lui tombe dessus depuis le grand toit, ou on plane par-dessus). L'antenne qui était au fond
  du creux (colonne 77) passe à la colonne 68, sur le toit d'arrivée du grand plané : aucune patrouille
  ne passe devant un objet de décor (test `aucun_decor_devant_un_danger_ni_un_objet`). Les autres toits
  plats portent une lanterne (une flaque y passerait sur Pépin réapparu), le départ, ou des objets de
  décor (le dernier toit : antenne et cheminée). Texte de mécanique : « …, premières flaques à écraser. »
  (l'aide dit déjà « Écrase les flaques en leur sautant dessus »).
- Gouttières : deux corbeaux DEUX rangées au-dessus des toits (rangée 5, `PORTEE_CORBEAU` de 5 tuiles de
  chaque côté) : on passe dessous en marchant, on saute une flaque d'un petit bond, mais un grand saut au
  mauvais moment s'y cogne (on peut aussi les écraser par le dessus : le robot de QA le fait). Le premier
  vole au-dessus des deux flaques du milieu (colonnes 45 à 55, hors du rebond du champignon 57) ; le
  second au-dessus de la lanterne 104 et de la flaque 108 (de la cheminée haute 100-101, où il fait
  demi-tour, à la colonne 111, hors du rebond du champignon 112). Pépin réapparu debout à la lanterne
  passe dessous : la liste des lanternes sur le trajet d'un danger ne change pas.
- Le marché couvert : un cinquième chat, sur le mur de la goutte cachée (colonne 68, dessus du mur en
  rangée 5), sur le passage OBLIGÉ du chemin principal (bloc de boutique, puis mur). Avec la règle du
  bond (carte 6.3) : posé sur le bloc, Pépin est hors d'atteinte (le bond ne quitte pas le mur, et le
  coin du mur coupe la ligne de vue) ; il guette le sommeil, saute sur le mur et écrase le chat (ou le
  saute). Posé à côté de lui sur le mur, on est à portée. Rien au-dessus des six auvents du moment fort.
  Pas de pigeon : le test
  `les_pigeons_sont_dans_le_clocher_vents_nuages_et_descente_loin_des_oiseaux_de_decor` les réserve à
  quatre niveaux, et un seul chat suffisait à la courbe (voir plus bas). Ce chat est le premier dans
  l'ordre de lecture : il prend le premier sommeil non raccourci, et les quatre autres chats changent de
  décalage (ils sont décalés de 0, 30, 60 ou 90 pas selon leur rang : colonne 12 : 0 -> 30, 54 : 30 ->
  60, 114 : 60 -> 90, 64 : 90 -> 0). Le script des captures de la carte 6.3 (`ecrire_script_chats`) suit
  désormais le chat de la colonne 12 par sa place, et non « le premier chat du niveau ».
- Le grand orage : un second corbeau en plein ciel, rangée 5 (colonnes 31 à 41), au-dessus des deux
  pointes du deuxième toit : on saute les pointes d'un bond court, ou après son passage. Pas de flaque :
  chaque toit libre du niveau porte une lanterne, un objet de décor (paratonnerres), des pointes, ou
  finit sur l'ascenseur (en bas, sa plateforme est au ras du toit : une flaque pourrait y monter) ; le
  dernier toit a déjà sa flaque, qui passe sur la lanterne 145 (une seconde y passerait aussi). Un second
  corbeau de plus a été essayé et retiré (voir « Réglage »).
- Le clocher : un chat sur la galerie, deux tuiles avant le ventilateur (colonne 28) : on y attend le
  souffle (2 s de souffle, 2 s d'arrêt) à portée de son bond, il faut donc l'écraser pendant son sommeil
  ou monter dès que le ventilateur souffle ; à 8 tuiles de la lanterne 20, hors de portée. Et un corbeau
  sous la porte de la cage (rangée 18, colonnes 21 à 31), juste au-dessus de la tête de Pépin debout sur
  la dernière corniche du zigzag : on saute vers la porte quand il est passé.

### Réglage : ce qui a été essayé, et pourquoi c'est réduit

Chaque essai mesuré niveau par niveau (`mesure_d_un_niveau`, mêmes graines et mêmes essais que la mesure
officielle ; Habitué, morts moyennes ; avant la carte : toits 0,12, gouttières 0,50, marché 0,54, orage
0,65, clocher 0,69, boss 1,19). Le juge de la courbe (`ecarts_de_la_courbe`) n'a pas été touché.

| Essai                                                      | Habitué | Verdict                                         |
| ---------------------------------------------------------- | ------- | ----------------------------------------------- |
| Clocher : chat de la galerie seul                          | 0,71    | trop peu                                        |
| Clocher : chat + corbeau entre F3 et F4 (rangée 21)        | 0,69    | trop peu                                        |
| Clocher : chat + corbeau sous la porte (rangée 18)         | 0,79    | RETENU                                          |
| Clocher : chat + les deux corbeaux                         | 0,79    | pas mieux                                       |
| Clocher : + une flaque sur une corniche du zigzag (F1, F3 ou F4) | 0,52 à 0,71 | moins de morts (la flaque change l'itinéraire) |
| Clocher : + un chat au sol de la cage (colonne 6)          | 0,50    | moins de morts                                  |
| Clocher : + un pigeon sur un perchoir ajouté (19, 32)      | 0,71 à 0,85 | Débutant 14 à 15/48 game over (29 à 31 %) : écarté |
| Orage : corbeaux en 36 ET 166 (au-dessus des pointes 164-165) | 0,83 | au-dessus du clocher (0,79) : juge rouge        |
| Orage : corbeau en 166 seul                                | 0,81    | au-dessus du clocher : juge rouge               |
| Orage : corbeau en 36 seul                                 | 0,75    | RETENU                                          |
| Marché : chats en 79 (toit D) et 146 (toit d'arrivée)      | 0,31    | sous les gouttières : juge rouge                |
| Marché : chat en 79 seul / en 146 seul                     | 0,25 / 0,50 | sous les gouttières                         |
| Marché : chats en 17 et 146 / chat en 17 seul              | 0,56 / 0,62 | possible, mais 17 est sur le toit de départ, à 5 tuiles du chat 12 |
| Marché : chats en 61 (bloc de boutique) et 146             | 0,75    | piège la sortie de la cachette : faisabilité rouge (`niveau_3_on_ressort_de_la_cachette_du_chat`) |
| Marché : chats en 68 (mur) et 146                          | 0,81    | au-dessus de l'orage (0,75) : juge rouge        |
| Marché : chat en 68 (mur) seul                             | 0,67    | RETENU                                          |
| Gouttières : corbeau en 50 seul / en 106 seul              | 0,46 / 0,42 | sous la valeur d'avant                      |
| Gouttières : corbeaux en 50 et 106                         | 0,56    | RETENU                                          |
| Toits : flaques en 76 et 118                               | 0,19    | RETENU (bande des toits : sous 0,3)             |

Cibles de la carte, donc : toits 2 flaques (cible 2-3) ; gouttières 2 corbeaux (1-2) ; marché 1 chat (1-2
chats et/ou un pigeon) ; grand orage 1 corbeau, aucune flaque (cible 2-3 flaques et 1-2 corbeaux :
RÉDUIT, voir plus haut) ; clocher 1 chat et 1 corbeau (cible : 1 pigeon et/ou 1 chat). La courbe est
serrée : l'Habitué ne doit jamais redescendre de 1 à 6, et le boss reste le sommet ; chaque niveau ne
peut donc monter que jusqu'au niveau suivant. Un ennemi ajouté ne fait pas toujours monter la mesure : le
joueur simulé a une intention parfaite (il attend le chat hors de portée, le corbeau qui passe), et
l'ennemi change son itinéraire, donc l'heure où il arrive aux passages difficiles (aiguilles, auvents) et
les tirages de ses erreurs ; au marché, les chats en 79 et 146 FONT BAISSER les morts aux auvents du
moment fort (colonnes 87 et 104 : 21 morts de l'Habitué avant, 6 avec eux).

### Mesure de difficulté : avant / après

Commande officielle (`mesure_de_difficulte`), graine 20261113, 48 essais. Avant : 5ebbc78 (fin de la
carte 6.3) ; après : fc0a97c. Centièmes : `mesure_d_un_niveau` (mêmes chiffres que la mesure officielle,
qui arrondit au dixième). Les niveaux 6 à 11 sont identiques ligne à ligne dans
`docs/mesure-difficulte.txt` (morts, détail, lieux de mort, game over) ; seules changent, en plus des
lignes des niveaux 1 à 5, celles qui les additionnent (Total, partie enchaînée, liste « À SIGNALER »).

| Morts moyennes        | Débutant      | Habitué       | Expert        |
| --------------------- | ------------- | ------------- | ------------- |
| 1. Les toits          | 0,42 -> 0,54  | 0,12 -> 0,19  | 0,04 -> 0,06  |
| 2. Gouttières         | 1,52 -> 1,50  | 0,50 -> 0,56  | 0,19 -> 0,15  |
| 3. Le marché couvert  | 1,06 -> 1,27  | 0,54 -> 0,67  | 0,08 -> 0,08  |
| 4. Le grand orage     | 3,04 -> 3,48  | 0,65 -> 0,75  | 0,35 -> 0,40  |
| 5. Le clocher         | 2,83 -> 2,58  | 0,69 -> 0,79  | 0,10 -> 0,15  |
| 6. L'œil du cyclone   | 4,06 (inchangé) | 1,19 (inchangé) | 0,38 (inchangé) |

| Game over (niveau commencé avec 5 vies) | Débutant        | Habitué       | Expert   |
| --------------------------------------- | --------------- | ------------- | -------- |
| 1. Les toits                            | 0/48 -> 0/48    | 0 -> 0        | 0 -> 0   |
| 2. Gouttières                           | 3/48 -> 2/48    | 0 -> 0        | 0 -> 0   |
| 3. Le marché couvert                    | 3/48 -> 2/48    | 0 -> 0        | 0 -> 0   |
| 4. Le grand orage                       | 12/48 -> 13/48  | 0 -> 1/48     | 0 -> 0   |
| 5. Le clocher                           | 13/48 -> 8/48   | 0 -> 0        | 0 -> 0   |

Pire essai (morts) avant -> après : toits 4 -> 4, 1 -> 2, 1 -> 2 ; gouttières 5 -> 7, 3 -> 3, 2 -> 2 ;
marché 7 -> 6, 3 -> 3, 2 -> 1 ; orage 11 -> 16, 4 -> 5, 3 -> 3 ; clocher 11 -> 8, 4 -> 4, 1 -> 1. Durée
moyenne d'un essai : toits 38 -> 39 s, gouttières 55 -> 57 s, marché 33 -> 36 s, orage 67 -> 70 s,
clocher 78 -> 79 s pour le Débutant ; l'Expert met 44 -> 56 s au clocher (56 s dès le chat seul, sans le
corbeau : il attend son sommeil au pied du ventilateur). Total des morts : Débutant 25,1 -> 25,6, Habitué
5,9 -> 6,3, Expert 1,7 -> 1,7. Partie enchaînée : Débutant 47/48 parties avec au moins un game over
(inchangé), 2,6 en moyenne (inchangé), 7 au plus (5 avant) ; Habitué 13/48 -> 17/48, 0,3 -> 0,4 en
moyenne.

Le juge de la courbe est vert (`courbe_de_difficulte_dans_les_bandes`, et la mesure officielle
elle-même) : Habitué 0,19 / 0,56 / 0,67 / 0,75 / 0,79 / 1,19 du niveau 1 au boss, chaque niveau dans sa
bande (`BANDES_HABITUE`). Le marché couvert passe nettement au-dessus des gouttières (0,67 contre 0,56),
ce que la carte 6.3 signalait comme fragile (0,54 contre 0,50).

Le clocher BAISSE pour le Débutant (2,83 -> 2,58 morts, game over 13/48 -> 8/48) : ses morts aux pointes
de la cage passent de 107 à 89 (et les morts « ennemi » de 1 à 17). Explication probable, non prouvée :
le chat et le corbeau changent l'heure à laquelle il arrive dans la cage, donc la place des aiguilles
qu'il y trouve, et les tirages de ses erreurs. Ce n'est pas voulu, et la baisse se retrouve sur six
graines (2,60 -> 2,37, voir ci-dessous). Le clocher reste au-dessus de 10 % de game over pour le Débutant
(17 %), comme le grand orage (27 %) et le boss (33 %) : déjà signalés à la carte 6.2, toujours sans
retouche.

#### Bruit de la mesure : six graines

Le juge ne regarde que la graine officielle. Sur six graines (20261113 à 20261118, 48 essais chacune,
test `bruit_de_la_mesure` avec `GRAINES=...` ; la seule retouche, dans une copie jetable de 5ebbc78 et de
fc0a97c, rien de commité : la ligne par graine affichée au centième au lieu du dixième), morts moyennes :

| Moyenne de 6 graines  | Débutant      | Habitué       | Expert        | Habitué, graine par graine (après)  |
| --------------------- | ------------- | ------------- | ------------- | ----------------------------------- |
| 1. Les toits          | 0,45 -> 0,68  | 0,11 -> 0,17  | 0,02 -> 0,03  | 0,19 0,10 0,08 0,15 0,33 0,19       |
| 2. Gouttières         | 1,36 -> 1,54  | 0,53 -> 0,55  | 0,14 -> 0,15  | 0,56 0,75 0,21 0,38 0,67 0,71       |
| 3. Le marché couvert  | 0,99 -> 1,11  | 0,37 -> 0,34  | 0,06 -> 0,10  | 0,67 0,31 0,27 0,27 0,33 0,19       |
| 4. Le grand orage     | 3,09 -> 2,99  | 0,81 -> 0,91  | 0,29 -> 0,30  | 0,75 1,10 1,17 0,77 0,79 0,88       |
| 5. Le clocher         | 2,60 -> 2,37  | 0,86 -> 0,81  | 0,11 -> 0,25  | 0,79 0,83 0,94 0,56 0,98 0,73       |
| 6. L'œil du cyclone   | 4,06          | 1,18          | 0,33          | 1,19 1,29 1,21 0,92 1,15 1,29       |

Ce qu'il faut en retenir (À SIGNALER au coordinateur) :
- la graine officielle est la seule des six où l'Habitué monte sans jamais redescendre du niveau 1 au
  boss ; c'était déjà le cas avant cette carte (cinq graines sur six non monotones avant comme après) ;
- en moyenne, le marché couvert reste SOUS les gouttières pour l'Habitué (0,34 contre 0,55 ; 0,37 contre
  0,53 avant) : le chat du mur ne lui ajoute rien en moyenne (il ajoute des morts au Débutant et à
  l'Expert) ; sa hausse sur la graine officielle (0,54 -> 0,67) est en partie de la chance ;
- en moyenne, le grand orage passe AU-DESSUS du clocher pour l'Habitué (0,91 contre 0,81 ; 0,81 contre
  0,86 avant) : c'est nouveau. Retirer le corbeau de l'orage ne règle rien (sur la graine officielle, le
  marché, 0,67, passerait au-dessus de l'orage, 0,65 : juge rouge) ; le juge à une graine et ce bruit (un
  niveau bouge de 0,2 mort d'une graine à l'autre pour l'Habitué) ne permettent pas de régler plus fin
  niveau par niveau. Une piste, hors de cette carte : juger la courbe sur la moyenne de plusieurs
  graines ;
- le Débutant meurt plus aux toits, aux gouttières et au marché, un peu moins à l'orage et au clocher ;
  l'Expert, plus au marché et au clocher (0,11 -> 0,25).

### Robots et tests

- Robot de faisabilité : vert sur les cinq niveaux, depuis le départ et depuis chaque lanterne, robot
  précis et maladroit (il ignore les flaques et les corbeaux, évitables en attendant ou en écrasant ; il
  connaît le chat et son bond depuis la carte 6.3 : le chat du mur et celui de la galerie sont gardés
  endormis, et le robot ne s'arrête jamais là où ils bondiraient). Tests propres aux niveaux inchangés et
  verts (`niveau_3_le_chat_de_la_fente_garde_la_goutte_cachee`,
  `niveau_3_on_ressort_de_la_cachette_du_chat`, `le_clocher_exige_le_ventilateur`, etc.).
- Robot de QA (vraie simulation : flaques, corbeaux et chats y sont joués tels quels, un plan qui meurt
  est écarté) : les deux parties finissent les onze niveaux sans mourir
  (`le_robot_finit_tous_les_niveaux_sans_mourir`, `la_partie_complete_joue_le_bonus`) ;
  `au_clocher_le_parcours_allume_les_trois_lanternes` vert. Partie du robot
  (`ecrire_script_navigateur`) : 21 348 images, 0 mort, 16 gouttes, empreinte `9139f4114520258f`. Aucun
  changement de robot nécessaire : aucune mécanique nouvelle.
- Lanternes sur le trajet d'un danger (`lanternes_sur_le_trajet_d_un_danger`) : liste INCHANGÉE, (2, 66,
  flaque), (2, 104, flaque), (3, 50, bond de chat), (3, 110, bond de chat), (4, 145, flaque) ; aucun
  nouvel ennemi ne passe sur Pépin réapparu, aucun chat ajouté ne bondit sur une lanterne.
- Pigeons : aucun ajouté ; décor : aucun objet sur une patrouille
  (`aucun_decor_devant_un_danger_ni_un_objet`).
- Premier défi visible dès le premier écran, durées plausibles, lint de grille (lanternes posées, aucune
  poche de vide close) : verts.

### Captures `docs/qa/64-*` (11 images)

Build de QA d'une extraction propre de fc0a97c (`./build.sh --qa`), servi en local sur 127.0.0.1 (port
libre, serveur arrêté à la fin), Chromium 1208 (Chrome for Testing 145) sans tête piloté par CDP
(`docs/capture-visibilite.mjs`, onglet au premier plan et canvas focalisé), 960 x 540 (x3). Un script par
niveau, écrit par le test ignoré `ecrire_script_ennemis` (`SCRIPT=target/qa/script-64-N.txt`,
`TOUCHES=target/qa/touches-64-N.txt` : touche N à l'image 229 du pilote, aucune au niveau 1) : le
parcours du robot dans le niveau, joué jusqu'au drapeau ; numéros d'image de `trace-64-N.csv`, qui suit
chaque ennemi ajouté.

- `64-toits-creux-saut` (1175) : Pépin plane au-dessus du creux entre les cheminées, la flaque au fond ;
- `64-toits-pied-chute` (1650) : Pépin saute du grand toit vers le petit toit du pied de l'immeuble, la
  flaque dessus, le trou de deux tuiles plus loin ;
- `64-gouttieres-corbeau-1` (975) : Pépin sur une cheminée, le premier corbeau devant lui, deux rangées
  au-dessus du toit, une flaque dessous ;
- `64-gouttieres-corbeau-2-lanterne` (1725) : le second corbeau au-dessus de la lanterne 104 et de la
  flaque 108, Pépin sur la cheminée haute ;
- `64-marche-chat-mur-eveille` (1075) : Pépin sur le bloc de boutique, hors d'atteinte ; le chat du mur
  ÉVEILLÉ (assis) ne bondit pas ; le chat de la fente éveillé aussi, la goutte cachée au fond du passage,
  le chat de la colonne 54 endormi à gauche ;
- `64-marche-chat-mur-saut` (1175) : le chat du mur rendormi (« z »), Pépin lui tombe dessus ;
- `64-orage-corbeau-pointes` (875) et `64-orage-corbeau-passe` (950) : le corbeau au-dessus des pointes,
  Pépin les saute, puis le corbeau passé derrière lui ;
- `64-clocher-chat-pret` (810) : sur la galerie, le chat de la colonne 28 « prêt à bondir » (« ! », queue
  dressée), Pépin à portée, le ventilateur derrière le chat, le pigeon de la corniche en plein vol ;
- `64-clocher-chat-bond` (850) : le chat en plein bond (pattes tendues) vers l'endroit où se tenait
  Pépin, qui lui a sauté par-dessus, vers le ventilateur ;
- `64-clocher-corbeau-tete` (1425) : le corbeau juste devant et au-dessus de la tête de Pépin, debout sur
  la dernière corniche ; en haut à gauche, le sol de la porte de la cage.

Le robot écrase les deux corbeaux des gouttières (ils sont écrasables, comme celui du grand orage) ; les
captures montrent donc chaque corbeau avant. Chaque capture regardée ;
`docs/mesure-capture.py <capture> 3` : 0 pixel hors palette, 0 bloc flou, sur les 11 (et sur les 5 autres
prises et non gardées).

### Vérification sur une extraction propre de 62af160 (dernier commit de code et des captures)

`git archive`, nouveau dossier daté sous `$TMPDIR` :

- `./docker-cargo.sh test --workspace --no-default-features` : 374 (core) et 336 (game) tests, tous verts
  (1 + 21 ignorés : `ecrire_script_ennemis` s'ajoute aux tests ignorés), dont les tests de faisabilité
  des onze niveaux, `lanternes_sur_le_trajet_d_un_danger`, `aucun_decor_devant_un_danger_ni_un_objet`,
  `courbe_de_difficulte_dans_les_bandes` et les deux parties du robot ;
- `./docker-cargo.sh lint` : `FMT-OK`, `LINT-OK` ;
- `./build.sh` : `dist/` produit, wasm 497 695 octets (497 647 à la carte 6.3 : + 48), 0 occurrence des
  noms interdits, seule URL `http://www.w3.org/2000/svg`, « Essai chat » absent de `dist/` ;
  `./build.sh --qa` (build des captures, extraction de fc0a97c) : `dist-qa/` produit.
- Mesure de difficulté (`mesure_de_difficulte`) : verte sur chacun des cinq états intermédiaires (un par
  commit de niveau : extraction de 5ebbc78 plus les grilles des niveaux déjà retouchés) ; le fichier
  `docs/mesure-difficulte.txt` de chaque commit est celui de son état. Relancée sur l'extraction propre
  de 62af160 : verte, et le fichier qu'elle écrit est identique à l'octet à celui du dépôt.
- `ecrire_script_chats` (script des captures 6.3, retouché) : vert, Pépin échappe toujours au bond (926
  images au lieu de 956 : le chat de la colonne 12 a maintenant un premier sommeil raccourci de 30 pas).

Commits : 700b5f3 (clocher), e1752dc (grand orage), 2c87bf3 (marché couvert, et script 6.3), c1c9266
(gouttières), fc0a97c (toits), a31e835 (script du navigateur), 62af160 (captures), cette section et le
README.

### Ce qui n'est PAS testé

- Aucun humain n'a joué ces niveaux : l'emplacement des ennemis est jugé sur la mesure (joueur simulé à
  intention parfaite, qui ne rate que des gestes) et sur les captures fixes, pas en mouvement.
- Le grand orage n'a pas ses flaques (cible de la carte) : aucun toit libre sans lanterne, décor, pointes
  ou ascenseur ; en ajouter demandait de déplacer une lanterne ou un décor, hors du périmètre choisi.
- La courbe est serrée (chaque niveau entre ses voisins pour l'Habitué) et la mesure est bruitée : le
  juge passe sur la graine officielle seulement ; sur six graines, le marché reste sous les gouttières et
  l'orage passe au-dessus du clocher (voir « Bruit de la mesure »). Un réglage de plus sur un niveau 1 à
  5 risque de retourner la graine officielle.
- Pas de son propre aux nouveaux ennemis (aucun son ajouté) ; captures dans Chromium seulement, à 960 x
  540 ; pas de vrai téléphone, ni Safari, ni Firefox.

## Plus d'ennemis dans les niveaux 7, 8, 9 et 11 (carte 6.4b)

Demande d'AGA (mot pour mot) : « tu peux rajouter des ennemis sur les niveau, il y a beaucoup de niveaux
avec peu d'ennemis ». Le but est un défi. Cette carte traite les niveaux 7, 8, 9 et 11, avec la démarche
de la carte 6.4a (niveaux 1 à 5) : types existants seulement, aucune physique ni mécanique modifiée, un
commit par niveau avec sa mesure. Non touchés : 6 (L'œil du cyclone, le boss) et 10 (L'arc-en-ciel, le
bonus sans ennemi) ; leurs lignes de `docs/mesure-difficulte.txt`, comme celles des niveaux 1 à 5, sont
identiques ligne à ligne à la fin de la carte 6.4a.

### Inventaire avant / après

f flaque, c corbeau, p pigeon (case de la lettre dans la grille : colonne, rangée).

| Niveau                  | Avant (6.4a)  | Après               | Ajouts                                      |
| ----------------------- | ------------- | ------------------- | ------------------------------------------- |
| 7. Vents d'altitude     | 2 c + 1 p (3) | 4 c + 2 p (6)       | c (76, 3), c (147, 3), p (38, 3)            |
| 8. La mer de nuages     | aucun (0)     | 3 c + 1 p (4)       | p (65, 3), c (85, 4), c (97, 7), c (162, 6) |
| 9. Au-dessus des nuages | 1 p (1)       | 1 p + 2 c + 1 f (4) | f (37, 5), c (63, 3), c (120, 2)            |
| 11. La descente         | 2 p (2)       | 2 p + 1 c (3)       | c (10, 12)                                  |
| Total des quatre        | 6             | 17                  | + 11                                        |

Cibles de la carte : vents +2 c +1 p (fait) ; mer +2-3 c et +1 p (fait : 3 c et 1 p ; le plané de vingt
tuiles reste SANS ennemi, la demande en admettait un au plus) ; nuages +2 c et +1-2 f ou chats (fait : 2 c
et 1 f, la seconde flaque retirée, voir plus bas) ; descente +1 au plus (fait : 1 c, et elle reste le
niveau le plus calme).

Emplacements et raisons (détail dans le commentaire en tête de chaque fichier de niveau) :

- Vents d'altitude (`core/src/levels/vents.rs`) :
  - corbeau (76, 3), patrouille des colonnes 71 à 81, juste au-dessus de la tête de Pépin posé sur les
    nuages friables (72 à 75) : on passe dessous en marchant, mais il faut repartir d'un saut avant que
    les friables ne cèdent, et un saut au mauvais moment s'y cogne ;
  - corbeau (147, 3), à hauteur de Pépin sur l'îlot des colonnes 143 à 151 (patrouille 142 à 152) : on
    lui tombe dessus en descendant de l'îlot de la lanterne 136, ou on le saute ; la lanterne 136 est
    hors de sa patrouille ;
  - pigeon (38, 3) sur un petit nuage posé pour lui (38, 4), quatre rangées au-dessus de l'îlot de la
    lanterne 43, comme celui du dernier toit : tourné vers l'arrivée de Pépin, il plonge derrière lui,
    au ras de l'îlot (colonne 34), loin de la lanterne et à plus de 4 tuiles de tout oiseau de décor.
- La mer de nuages (`core/src/levels/mer.rs`, jusque-là sans ennemi) :
  - pigeon (65, 3) au bord gauche de l'étage, tourné vers le premier quai : il plonge sur qui passe
    dessous, au bout du quai ou dans la nacelle de la première montgolfière (63), et finit au ras du
    quai. On le fait plonger d'un pas au bout du quai, on recule, on embarque pendant son repos ; en
    haut, on peut lui tomber dessus. La demande disait « un pigeon sur un quai » : perché SUR un quai,
    il ne guetterait que le vide ; perché au-dessus du quai, il le garde ;
  - corbeau (85, 4), patrouille 80 à 90, entre les deux rideaux de grêle, sous le long plané vers le
    second quai : on part de l'étage quand il s'éloigne ; fermer le parapluie pour plonger sous lui ne
    se fait qu'entre les rideaux (sous la grêle, parapluie fermé, on meurt) ;
  - corbeau (97, 7), patrouille 92 à 102, à hauteur de Pépin sur le second quai, avant la montgolfière
    104 : on atterrit dessus ou on le saute ;
  - corbeau (162, 6), patrouille 157 à 167, une rangée sous le vent du dernier plané : on plane haut
    dans le vent, ou l'on part de la corniche quand il s'éloigne ;
  - le moment fort (colonnes 110 à 130) : aucune patrouille ne le croise (test).
- Au-dessus des nuages (`core/src/levels/nuages.rs`) :
  - flaque (37, 5) sur le petit nuage des colonnes 35 à 40, entre deux sauts de l'escalier du début ;
  - corbeau (63, 3), patrouille 58 à 68, juste au-dessus de la tête de Pépin posé sur la traversée
    friable ;
  - corbeau (120, 2), patrouille 115 à 125, juste au-dessus de la tête de Pépin posé sur le nuage
    d'avant le dernier toit : on attend qu'il s'éloigne pour sauter.
- La descente (`core/src/levels/descente.rs`) : corbeau (10, 12), patrouille 5 à 15, sous la première
  goutte, 21 px au-dessus de la tête de Pépin posé sur la première couche : on passe dessous en marchant,
  on lui tombe dessus en descendant du départ, un saut vers la goutte au mauvais moment s'y cogne.

### Réglage : ce qui a été essayé, et ce qui est réduit

Chaque placement essayé avec `mesure_d_un_niveau` (mêmes graines que la mesure officielle) dans une
copie jetable du dépôt ; Habitué en morts moyennes, avant -> après.

- Vents (0,40) : corbeau au-dessus de l'îlot de la lanterne 43 (40, 5) -> 0,69, hors de la bande des
  vents ([0,3 ; 0,65[), écarté ; corbeaux (88, 4) et (116, 2) -> 0,42 et 0,40 (jamais sur le chemin,
  écartés) ; corbeau (152, 2) -> 0,44, (147, 3) gardé (même effet, mais à hauteur de Pépin sur l'îlot :
  il se voit et se saute). Retenu : 0,52.
- Mer (0,46) : corbeau au-dessus du premier vent (22, 3) et après le moment fort (137, 7) -> 0,46 (aucun
  effet, écartés) ; pigeon, corbeaux (84, 3) et (57, 7) -> Expert 0,27, écarté ; corbeau du premier quai
  (57, 7) à la place de celui du second -> 0,58. Retenu : pigeon, (85, 4), (97, 7), (162, 6) -> 0,73.
- Au-dessus des nuages (0,08) : corbeau (85, 5) au-dessus du trampoline -> Expert 0,19 avec les autres
  ajouts, écarté ; chat (124, 3) seul -> 0,21, hors de la bande du niveau ([0 ; 0,2[), écarté ; chat
  (37, 5) -> 0,06 ; flaques (37, 5) et (124, 3) avec les deux corbeaux -> 0,17, commité (a1492c7),
  puis réduit (9befa46) : sur six graines, cette version sortait de la bande trois fois sur six (0,21,
  0,21, 0,27) et passait au-dessus des vents une fois (graine 20261118 : 0,27 contre 0,25). Sans la
  flaque (124, 3) : une fois hors de la bande (0,23), jamais au-dessus des vents ; sans celle de
  l'escalier (37, 5) à la place : deux fois hors de la bande, une fois au-dessus des vents. Retenu : la
  flaque de l'escalier et les deux corbeaux -> 0,15.
- Descente : corbeaux (16, 20) et (16, 22), au-dessus de la deuxième couche -> 0,00 partout (la seule
  mort d'avant, à la grêle, disparaît) ; (7, 13), (12, 13), (10, 12) -> Débutant 0,02 comme avant.
  Retenu (10, 12) : sous la goutte, là où l'on saute. La mesure de la descente est identique à
  l'octet : aucun joueur simulé n'y meurt du corbeau (il le voit venir et passe dessous ou à côté).

### Mesure de difficulté : avant / après

Commande officielle (`mesure_de_difficulte`), graine 20261113, 48 essais. Avant : 97067bc (fin de la
carte 6.4a) ; après : 9befa46. Centièmes : `mesure_d_un_niveau` (mêmes chiffres que la mesure
officielle, qui arrondit au dixième). Dans `docs/mesure-difficulte.txt`, seules changent les lignes des
niveaux 7, 8 et 9 et celles qui les additionnent (Total, partie enchaînée, liste « À SIGNALER ») ; les
niveaux 1 à 6, 10 et 11 sont identiques ligne à ligne.

| Morts moyennes           | Débutant      | Habitué       | Expert        |
| ------------------------ | ------------- | ------------- | ------------- |
| 7. Vents d'altitude      | 1,31 -> 1,54  | 0,40 -> 0,52  | 0,10 -> 0,10  |
| 8. La mer de nuages      | 1,46 -> 2,23  | 0,46 -> 0,73  | 0,12 -> 0,17  |
| 9. Au-dessus des nuages  | 0,50 -> 0,67  | 0,08 -> 0,15  | 0,00 -> 0,06  |
| 11. La descente          | 0,02 (inchangé) | 0,00 (inchangé) | 0,00 (inchangé) |

| Game over (niveau commencé avec 5 vies) | Débutant      | Habitué | Expert |
| --------------------------------------- | ------------- | ------- | ------ |
| 7. Vents d'altitude                     | 2/48 -> 4/48  | 0 -> 0  | 0 -> 0 |
| 8. La mer de nuages                     | 0/48 -> 5/48  | 0 -> 0  | 0 -> 0 |
| 9. Au-dessus des nuages                 | 1/48 -> 0/48  | 0 -> 0  | 0 -> 0 |
| 11. La descente                         | 0/48 -> 0/48  | 0 -> 0  | 0 -> 0 |

Pire essai (morts, Débutant / Habitué / Expert) avant -> après : vents 6/2/1 -> 8/4/1 ; mer 4/2/1 ->
7/4/1 ; nuages 5/2/0 -> 3/1/1 ; descente 1/0/0 inchangé. Durée moyenne d'un essai (mêmes profils) :
vents 48/41/38 s -> 50/42/38 s ; mer 44/36/34 s -> 51/41/35 s ; nuages 32/29/29 s -> 35/31/30 s ;
descente 19/18/18 s inchangé. Total des morts : Débutant 25,6 -> 26,7, Habitué 6,3 -> 6,8, Expert
1,7 -> 1,8. Partie enchaînée : Débutant 47/48 parties avec au moins un game over (inchangé), 2,6 -> 2,9
en moyenne, 7 -> 6 au plus ; Habitué 17/48 -> 23/48, 0,4 -> 0,5 en moyenne ; Expert 1/48 -> 2/48.

Où l'on meurt des nouveaux ennemis (lieux de mort de la mesure) : à la mer, le pigeon du quai (colonne
63 : 8 des 35 morts de l'Habitué, 3 des 8 de l'Expert) ; aux nuages, la flaque de l'escalier (colonnes
35-36) et le corbeau d'avant le dernier toit (colonne 119) ; aux vents, presque aucune mort d'un nouvel
ennemi (le Débutant, une fois, le corbeau de l'îlot, colonne 149) : la hausse vient de chutes nouvelles
aux colonnes 49 (après l'îlot du pigeon) et 82 à 84 (après les friables, sous le corbeau). Les ennemis
pressent le saut ; c'est une explication probable, pas une preuve.

À SIGNALER : la mer de nuages entre dans la liste des niveaux normaux au-dessus de 10 % de game over
pour le Débutant, de justesse (5/48, 10,4 %) ; sur six graines, 26/288 (9,0 %). Si le coordinateur la
veut sous le seuil, le levier est le pigeon du quai (le plus meurtrier) ; cette carte ne l'a pas retiré
(c'est le seul ennemi de la mer qui demande d'attendre, le défi voulu).

#### Courbe finale des onze niveaux (graine officielle, morts moyennes)

| Niveau                      | Débutant | Habitué | Expert |
| --------------------------- | -------- | ------- | ------ |
| 1. Les toits                | 0,54     | 0,19    | 0,06   |
| 2. Gouttières et cheminées  | 1,50     | 0,56    | 0,15   |
| 3. Le marché couvert        | 1,27     | 0,67    | 0,08   |
| 4. Le grand orage           | 3,48     | 0,75    | 0,40   |
| 5. Le clocher               | 2,58     | 0,79    | 0,15   |
| 6. L'œil du cyclone (boss)  | 4,06     | 1,19    | 0,38   |
| 7. Vents d'altitude         | 1,54     | 0,52    | 0,10   |
| 8. La mer de nuages         | 2,23     | 0,73    | 0,17   |
| 9. Au-dessus des nuages     | 0,67     | 0,15    | 0,06   |
| 10. L'arc-en-ciel (bonus)   | 8,83     | 1,23    | 0,29   |
| 11. La descente (épilogue)  | 0,02     | 0,00    | 0,00   |

Le juge de la courbe est vert (`courbe_de_difficulte_dans_les_bandes`, et la mesure officielle
elle-même), sans retouche du juge : le boss reste le sommet (Débutant et Habitué) ; la mer au-dessus
des vents (Habitué 0,73 contre 0,52 ; Débutant 2,23 contre 1,54) ; les vents au-dessus du niveau 9 et
nettement sous le boss (1,19 - 0,52 = 0,67, il en faut 0,15) ; chaque niveau dans sa bande Habitué
(vents [0,3 ; 0,65[, mer [0,2 ; 1,0[, nuages [0 ; 0,2[, descente [0 ; 0,2[) ; l'épilogue le plus calme
de tous ; le bonus en 58,5 s pour l'Expert (borne [45 ; 75[ s).

#### Bruit de la mesure : six graines

Même méthode qu'à la carte 6.4a (graines 20261113 à 20261118, 48 essais chacune, `mesure_d_un_niveau`
dans une copie jetable où la graine se choisit par GRAINES ; rien de commité). Morts moyennes :

| Moyenne de 6 graines     | Débutant      | Habitué       | Expert        | Habitué, graine par graine (après) |
| ------------------------ | ------------- | ------------- | ------------- | ---------------------------------- |
| 7. Vents d'altitude      | 1,31 -> 1,61  | 0,43 -> 0,45  | 0,08 -> 0,08  | 0,52 0,27 0,52 0,58 0,56 0,25      |
| 8. La mer de nuages      | 1,33 -> 1,95  | 0,46 -> 0,59  | 0,09 -> 0,19  | 0,73 0,67 0,48 0,67 0,65 0,33      |
| 9. Au-dessus des nuages  | 0,45 -> 0,69  | 0,06 -> 0,16  | 0,01 -> 0,02  | 0,15 0,23 0,19 0,15 0,10 0,15      |
| 11. La descente          | 0,00 -> 0,00  | 0,00 -> 0,00  | 0,00 -> 0,00  | 0,00 sur les six                   |

Ce qu'il faut en retenir :
- la mer passe au-dessus des vents pour l'Habitué sur 5 graines sur 6 (3 sur 6 avant), pour le
  Débutant sur 6 sur 6 (4 sur 6 avant) : la « remontée » de la courbe, fragile avant, est plus nette ;
- les vents restent au-dessus du niveau 9 sur les six graines (Habitué et Débutant), et nettement sous
  le boss (au moins 0,34 d'écart) ;
- le niveau 9 sort de sa bande Habitué ([0 ; 0,2[) sur une graine (20261114 : 0,23) : c'est ce qui
  reste de fragile, après le retrait d'une flaque (voir « Réglage ») ;
- les vents sortent PAR LE BAS de leur bande ([0,3 ; 0,65[) sur deux graines (0,27 et 0,25) ; c'était
  déjà le cas d'une graine avant (0,25) ;
- game over du Débutant sur les six graines : vents 10/288 -> 17/288, mer 1/288 -> 26/288, nuages
  1/288 -> 1/288, descente 0 -> 0.

### Robots et tests

- Robot de faisabilité : vert sur les onze niveaux, depuis le départ et depuis chaque lanterne, robot
  précis et maladroit (il ignore flaques, corbeaux et pigeons, évitables en attendant ou en écrasant).
  Aucun robot à modifier : aucune mécanique nouvelle.
- Robot de QA (vraie simulation : un plan qui meurt est écarté) : les deux parties finissent les onze
  niveaux sans mourir (`le_robot_finit_tous_les_niveaux_sans_mourir`, `la_partie_complete_joue_le_bonus`).
  Il écrase le corbeau du second quai de la mer et le corbeau des friables du niveau 9 ; il passe sous
  le corbeau d'avant le dernier toit du niveau 9 à 0 px près (voir les captures).
- Lanternes sur le trajet d'un danger (`lanternes_sur_le_trajet_d_un_danger`) : liste INCHANGÉE, (2, 66,
  flaque), (2, 104, flaque), (3, 50, bond de chat), (3, 110, bond de chat), (4, 145, flaque) : aucun
  ennemi ajouté ne passe sur Pépin réapparu à une lanterne.
- Pigeons : `les_pigeons_sont_dans_le_clocher_vents_mer_nuages_et_descente_loin_des_oiseaux_de_decor`
  (renommé : la mer est admise, 1 à 3 pigeons par niveau, à au moins 4 tuiles de tout oiseau de décor) ;
  `la_plongee_des_pigeons_reste_en_l_air_et_loin_des_lanternes` vert (les deux nouvelles plongées ne
  traversent aucun solide et ne passent ni sur une lanterne ni sur le départ) ;
  `le_pigeon_se_detache_du_ciel_des_niveaux_4_et_5` couvre aussi la mer (contraste d'au moins 3:1 sur le
  ciel corail, du perchoir au fond de la plongée).
- Mer : `aucun_ennemi` remplacé par `trois_corbeaux_un_pigeon_et_aucun_dans_le_moment_fort` (aucune
  patrouille de corbeau ne croise les colonnes 110 à 130). Descente :
  `aucune_chute_ne_tue_et_seuls_deux_pigeons_un_corbeau_et_deux_colonnes_de_grele_menacent` (renommé,
  compte le corbeau) ; `se_laisser_tomber_du_bord_d_un_trou_ne_touche_pas_la_grele` vert.
- Décor : aucun objet sur une patrouille (`aucun_decor_devant_un_danger_ni_un_objet`), lint de grille
  (lanternes posées, aucune poche de vide close), premier défi visible, durées plausibles : verts.

### Captures `docs/qa/64b-*` (15 images)

Build de QA (`./build.sh --qa`) d'une copie dont `core/`, `game/`, `web/` et les scripts de build sont
identiques à ab45cdb pour les niveaux 7, 8 et 11, et à 9befa46 pour le niveau 9 (repris après le retrait
de la flaque), servi en local sur 127.0.0.1 (port libre, serveur arrêté à la fin), Chromium 1208 (Chrome
for Testing 145) sans tête piloté par CDP (`docs/capture-visibilite.mjs`, onglet au premier plan et
canvas focalisé), 960 x 540 (x3). Un script par niveau, écrit par le test ignoré
`ecrire_script_ennemis_64b` (`SCRIPT=target/qa/script-64b-N.txt`, `TOUCHES=target/qa/touches-64b-N.txt` :
touche 7, 8 ou 9 à l'image 229 du pilote ; pour la descente, touche 9 puis N aux images 457 et 685) ;
numéros d'image de `trace-64b-N.csv`, qui suit chaque ennemi ajouté (et la phase de vol des pigeons).

- `64b-vents-pigeon-plongee` (907) : le pigeon du nuage (38, 4) en pleine plongée, derrière Pépin qui
  file vers la droite sur l'îlot de la lanterne 43 ;
- `64b-vents-corbeau-friables` (1350) : Pépin posé sur les nuages friables, le corbeau qui arrive vers
  lui, juste au-dessus de sa tête ; `64b-vents-corbeau-friables-saut` (1378) : Pépin quitte les
  friables d'un saut, le corbeau sous lui ;
- `64b-vents-corbeau-ilot` (2262) : Pépin sur l'îlot des colonnes 143 à 151, le corbeau qui vient à sa
  hauteur ; `64b-vents-corbeau-ilot-saut` (2284) : Pépin le saute ;
- `64b-mer-pigeon-nacelle` (1171) : le pigeon plonge pendant que la montgolfière hisse Pépin (il le
  manque de peu) ;
- `64b-mer-corbeau-grele` (1431) : Pépin plane entre les deux rideaux de grêle, le corbeau sous lui ;
- `64b-mer-corbeau-quai` (1521) : Pépin tombe sur le corbeau du second quai (écrasé à l'image 1528) ;
- `64b-mer-corbeau-vent` (2359) : Pépin plane dans le vent du dernier plané, le corbeau une rangée plus
  bas, la grêle derrière (6 vies : la troisième goutte en rend une) ;
- `64b-nuages-flaque-escalier` (890) : la flaque sur le petit nuage de l'escalier, Pépin au-dessus ;
- `64b-nuages-corbeau-friables` (1200) : Pépin sur la traversée friable, le corbeau juste au-dessus de
  sa tête, un peu devant ;
  `64b-nuages-corbeau-ecrase` (1259) : le corbeau écrasé sous Pépin ;
- `64b-nuages-corbeau-attente` (1912) : Pépin attend sur le nuage d'avant le dernier toit, le corbeau
  arrive au-dessus ; `64b-nuages-corbeau-saut` (1927) : Pépin saute sous le corbeau qui arrive ;
- `64b-descente-corbeau` (1046) : Pépin tombe du départ vers la première couche, à droite du corbeau
  qui patrouille sous la goutte.

Chaque capture regardée ; `docs/mesure-capture.py <capture> 3` : 0 pixel hors palette, 0 bloc flou, sur
les 15. Compteur de vies vérifié sur chacune (même chiffre que la capture de référence, sauf la mer à 6
vies après la troisième goutte). Une capture écartée : au niveau 9, à l'image 1934, le rejeu du
navigateur a touché le corbeau d'avant le dernier toit (4 vies, Pépin à la lanterne) ; dans la
simulation, le robot passe dessous à 0 px près (haut de Pépin à y = 45, bas du corbeau à y = 45) et le
rejeu glisse d'un pas, comme noté à la carte 6.4a. Ce n'est pas un défaut du niveau : un joueur passe
avec de la marge en attendant que le corbeau s'éloigne.

### Vérification sur une extraction propre de 5250968 (dernier commit de code et des captures)

`git archive`, nouveau dossier daté sous `$TMPDIR` :

- `./docker-cargo.sh test --workspace --no-default-features` : 374 (core) et 336 (game) tests, tous verts
  (1 + 22 ignorés : `ecrire_script_ennemis_64b` s'ajoute aux tests ignorés), dont les tests de
  faisabilité des onze niveaux, `reprise_depuis_chaque_lanterne`, `lanternes_sur_le_trajet_d_un_danger`,
  `aucun_decor_devant_un_danger_ni_un_objet`, les tests des pigeons, `courbe_de_difficulte_dans_les_bandes`
  et les deux parties du robot ;
- `./docker-cargo.sh lint` : `FMT-OK`, `LINT-OK` ;
- `./build.sh` : `dist/` produit, wasm 497 743 octets (497 695 à la carte 6.4a : + 48), 0 occurrence des
  noms interdits, seule URL `http://www.w3.org/2000/svg`, « Essai chat » absent de `dist/` ;
- mesure de difficulté (`mesure_de_difficulte`) : verte sur chacun des états intermédiaires (un par
  commit de niveau) ; le fichier `docs/mesure-difficulte.txt` de chaque commit est celui de son état ;
  relancée sur l'extraction propre de 5250968 : verte, et le fichier qu'elle écrit est identique à
  l'octet à celui du dépôt ;
- partie du robot (`ecrire_script_navigateur`) : 21 414 images (21 348 à la carte 6.4a), 0 mort, 15
  gouttes (16 à la carte 6.4a : son parcours change avec les nouveaux ennemis et passe par une goutte de
  moins), empreinte `1f3190119f6bd9b3` ;
- `ecrire_script_ennemis` (scripts des captures 6.4a, dont le travail passe dans
  `ecrire_scripts_des_ajouts`) : ses quinze fichiers `script-64-N`, `touches-64-N`, `trace-64-N`
  identiques à l'octet à ceux de la carte 6.4a.

Commits : fc9ffcb (mer de nuages), 73dc73c (vents d'altitude), a1492c7 (au-dessus des nuages), 8fbfdb1
(descente), ab45cdb (script du navigateur), b4442cd (captures), 9befa46 (au-dessus des nuages : une
flaque retirée), 5250968 (captures du niveau 9 refaites), cette section et le README (nombre de tests,
poids du wasm, ennemis des niveaux 7, 8, 9 et 11).

### Ce qui n'est PAS testé

- Aucun humain n'a joué ces niveaux : l'emplacement des ennemis est jugé sur la mesure (joueur simulé à
  intention parfaite, qui ne rate que des gestes) et sur des captures fixes, pas en mouvement.
- La descente : le corbeau ne tue aucun joueur simulé (mesure identique à l'octet) ; son effet sur un
  humain (un saut raté vers la goutte) n'est pas mesuré.
- Le juge passe sur la graine officielle ; sur six graines, le niveau 9 sort une fois de sa bande et les
  vents deux fois par le bas de la leur (voir « Bruit de la mesure »).
- Pas de son propre aux nouveaux ennemis ; captures dans Chromium seulement, à 960 x 540 ; pas de vrai
  téléphone, ni Safari, ni Firefox.

## Mesure de difficulté facultative (carte 6.5)

Décision d'AGA (2026-11-16) : AGA juge lui-même la difficulté en jouant. Les cartes ne relancent plus la
mesure de difficulté, ne régénèrent plus `docs/mesure-difficulte.txt` et ne sont plus bloquées par le juge de
la courbe. L'outil reste entier dans le dépôt, utilisable à la demande d'AGA. Aucun niveau, aucune règle du
jeu, aucun autre test touché ; aucune capture (rien ne change à l'écran).

### Ce qui a changé

- `game/src/difficulte.rs` : `courbe_de_difficulte_dans_les_bandes` porte `#[ignore = "mesure de difficulté
  facultative (décision d'AGA, carte 6.5) : se lance à la main"]` ; son commentaire dit pourquoi et comment
  le lancer. Commentaires du module et de `mesure_de_difficulte` mis à jour. Restent tels quels : la mesure
  (`mesure_de_difficulte`, `bruit_de_la_mesure`, tous deux déjà ignorés), le juge (`ecarts_de_la_courbe`,
  `BANDES_HABITUE`) et son test synthétique `la_forme_de_la_courbe_est_jugee` (mesures inventées,
  instantané), qui reste dans la suite normale.
- `docs/mesure-difficulte.txt` : non régénéré, il garde la mesure de la carte 6.4b ; une ligne en tête le
  dit (« Dernière mesure faite à la carte 6.4b, plus mise à jour automatiquement »). Relancer la mesure à
  la main réécrit le fichier entier, sans cette ligne : c'est alors la nouvelle mesure.
- Autres tests cherchés : seuls `mesure_de_difficulte`, `bruit_de_la_mesure` (déjà ignorés) et le juge
  appellent `mesurer` (la mesure complète), et aucun test ne lit `docs/mesure-difficulte.txt`. Les tests
  restés actifs de `difficulte.rs` sont courts : hasard, profils, rejeu des vies, game over, niveau 1 sans
  erreur, regroupement des lieux de mort, juge sur des courbes fabriquées.
- Ce fichier, section « Mesure de difficulté » : statut facultatif, commande du juge à la main, « quand AGA
  demande un tableau avant/après » au lieu de « pour chaque carte ». README : nombre de tests, durée.

### À lancer à la main, sur demande d'AGA

    ./docker-cargo.sh test -p game --no-default-features mesure_de_difficulte -- --ignored --nocapture
    ./docker-cargo.sh test -p game --no-default-features courbe_de_difficulte_dans_les_bandes -- --ignored
    ./docker-cargo.sh test -p game --no-default-features bruit_de_la_mesure -- --ignored --nocapture

La première réécrit `docs/mesure-difficulte.txt` ; les deux autres n'écrivent rien. Le juge, lancé ainsi sur
l'extraction propre de 09f28de : vert (1 test, 357 filtrés), 536 s (charge moyenne de 7 à 19), et
`docs/mesure-difficulte.txt` inchangé (`sha256sum -c` avant/après : OK).

### Restent obligatoires, dans la suite normale

Faisabilité (`*_faisable` des onze niveaux et des niveaux d'essai, 21 tests ;
`reprise_depuis_chaque_lanterne`), lint de grille (`aucune_lanterne_ne_flotte`, `aucune_poche_de_vide_close`),
robot de QA (`le_robot_finit_tous_les_niveaux_sans_mourir`, `la_partie_complete_joue_le_bonus`), tous verts.
Les noms et résultats des 733 tests listés par la suite, avant (c185e24) et après (09f28de), ne diffèrent
que d'une ligne : `courbe_de_difficulte_dans_les_bandes ... ok` devient `... ignored`. `game` : 336
réussis + 22 ignorés avant, 335 + 23 après ; `core` : 374 + 1, inchangé.

### Durée de la suite, avant / après

`./docker-cargo.sh test --workspace --no-default-features`, temps total mesuré de bout en bout (lancement
du conteneur compris, tests déjà compilés), sur des extractions propres de c185e24 (avant) et 09f28de
(après). Entre parenthèses : `core` / `game`, temps affichés par cargo.

| Passe                   | Avant (c185e24)             | Après (09f28de)             |
| ----------------------- | --------------------------- | --------------------------- |
| 1                       | 1 178,6 s (322,0 / 855,3)   | 856,8 s (260,5 / 595,3)     |
| 2                       | 1 138,9 s (376,2 / 761,6)   | 863,2 s (256,7 / 605,4)     |
| 3 (avant puis après)    | 1 313,2 s (303,9 / 1 007,9) | 1 110,9 s (435,1 / 674,7)   |
| Moyenne                 | 1 210,3 s (game : 874,9)    | 943,6 s (game : 625,1)      |

Gain moyen : 267 s sur la suite (- 22 %), 250 s sur `game` (- 29 %), soit environ 4 min 30 par
vérification complète. Les chiffres sont bruités : d'autres cartes compilaient et testaient sur la même
machine (charge moyenne relevée de 2 à 42 ; `core`, inchangé, varie à lui seul de 257 à 435 s). La passe 3
lance avant puis après à la suite, pour les comparer dans des conditions proches ; le gain y est le plus
faible (202 s) mais reste net. Le test le plus long de la suite reste le robot de QA
(`le_robot_finit_tous_les_niveaux_sans_mourir`, dernier à finir avant comme après).

### Vérification sur une extraction propre de 09f28de (dernier commit de code)

`git archive`, nouveau dossier daté sous `$TMPDIR` :

- `./docker-cargo.sh test --workspace --no-default-features` : vert, trois fois (tableau ci-dessus) ;
- `./docker-cargo.sh lint` : `LINT-OK` ;
- `./build.sh` : `dist/` produit, 0 occurrence des noms interdits, seule URL `http://www.w3.org/2000/svg` ;
  `dist/` identique à l'octet à celui de c185e24, construit de la même façon (`cmp` des trois fichiers :
  `index.html`, `mq_js_bundle.js`, `game.wasm` de 497 743 octets) : aucun changement de jeu ;
- robot de QA, les deux parties à 0 mort : `le_robot_finit_tous_les_niveaux_sans_mourir` et
  `la_partie_complete_joue_le_bonus` (`assert_eq!(partie.morts, 0)`), verts à chaque passe.

Les commits de cette section ne touchent que `QA.md`, `README.md` et la ligne d'en-tête de
`docs/mesure-difficulte.txt` (lus par aucun test ni par le build).

Commits : 09f28de (le juge de la courbe ignoré, commentaires), 7c76c90 (cette section et le README), puis
l'en-tête de `docs/mesure-difficulte.txt` avec les compléments de cette section.

### Ce qui n'est PAS testé

- La difficulté n'est plus surveillée automatiquement : une retouche de niveau qui casse la forme de la
  courbe passe désormais la suite sans le signaler. C'est voulu (AGA juge en jouant) ; pour la vérifier,
  lancer le juge à la main.
- Durées mesurées sur une machine partagée et chargée : l'ordre de grandeur du gain est sûr, pas sa valeur
  exacte.
- Aucune partie jouée, aucune capture : rien ne change dans le jeu.

## Pigeon redessiné (carte 6.6)

Demande (mot pour mot) : « les pigeons sont pas beaux, tu pourras les refaire, surtout quand ils
piquent ». Constat : perché, l'ancien pigeon (16 x 12, ardoise `2`) était une masse sombre où l'on ne
lisait ni tête ni queue ; en plongée, un fuseau sombre en diagonale (un poisson, une feuille). Rendu
seulement : aucune boîte de collision, aucune physique, aucun comportement ne change (`core/` intact).

### Ce qui a été fait

`game/src/sprites/ennemis.rs` : huit images de 18 x 14 au lieu de six de 16 x 12, toutes tournées vers
la droite (la scène les retourne), contour sombre de 1 px :

- un pigeon des villes : corps rond et dodu gris-bleu MOYEN (`4`, ARDOISE_PALE, la couleur principale),
  aile repliée plus claire (`5`, BRUME) barrée de deux traits sombres, ventre et dessous de la queue
  ombrés (`3`), queue courte au bout sombre ; petite tête ronde ardoise au front éclairé, œil orange,
  bec court ; cou irisé sarcelle et cyan avec une touche violette ; pattes corail clair ;
- perché (`pigeon0`, `pigeon1`) : il hoche la tête, le corps ne bouge pas ;
- annonce (`pigeon_annonce`) : il se ramasse, corps bas et tendu vers l'avant, tête rentrée, ailes
  entrouvertes (rémiges dressées au-dessus du dos), œil rouge corail sous un sourcil sombre ; la scène
  le fait toujours trembler d'un pixel ;
- plongée, deux images : l'élan (`pigeon_plongee0`, ailes grandes ouvertes, corps déjà couché sur la
  diagonale) pendant les 8 premiers pas (0,13 s), puis le piqué (`pigeon_plongee1`) pour les 24 autres :
  tête et bec devant, en bas vers Pépin, œil rouge, ailes relevées et repliées en flèche vers l'arrière,
  rémiges sombres en « doigts », queue derrière, en haut ;
- remontée, deux images (`pigeon_remontee0`, ailes levées ; `pigeon_remontee1`, ailes rabattues sous le
  ventre) : quatre battements pendant les 64 pas de la remontée ; il regarde toujours son perchoir ;
- écrasé (`pigeon_ecrase`) : aplati, plumes ébouriffées qui dépassent du dos.

`game/src/scene.rs` : sprite décalé de (-3, -4) au lieu de (-2, -2) : centré sur la même boîte 12 x 10
(le miroir aussi), le contour sous les pattes sur la dernière rangée de la boîte, 4 px dépassent en
haut. `image_pigeon` choisit l'image de la plongée et de la remontée d'après le compteur de la phase
(pas l'horloge) : chaque piqué commence par l'élan.

Distinction avec les deux autres oiseaux : le corbeau (carte 6.1) est bleu nuit presque noir (`1`),
gros bec gris, queue en éventail, il vole ; l'oiseau de décor `K` est petit (12 x 8) et blanc (`7`). Le
pigeon est entre les deux : gris-bleu moyen (`4`), à 5,0:1 du bleu nuit et 2,8:1 du blanc.

### Tests

- `le_pigeon_a_huit_images_de_meme_taille_et_un_contour_sombre` (remplace la version à six images) :
  18 x 14 pour les huit, aucun pixel coloré ne touche le vide ni le bord, cou irisé sur chaque image ;
  perché, seule la tête bouge (rangées 5 et suivantes identiques) ; remontée : de l'aile claire en haut
  sur l'une, sous le ventre sur l'autre ; écrasé : rien dans les 6 rangées du haut.
- `le_pigeon_en_pique_fond_sur_pepin_le_long_de_sa_diagonale` (nouveau, la priorité de la demande) : sur
  les deux images de la plongée, l'axe principal des pixels colorés penche entre 15° et 75° vers le bas
  et l'avant (mesuré : 23,5° pour l'élan, 27,2° pour le piqué) ; devant, le long de la diagonale (le plus
  grand x + y), c'est le bec clair, à 4 px au plus de l'œil ; un seul œil, rouge, en bas à droite ;
  derrière (le plus petit x + y), des rémiges sombres, dans le coin en haut à gauche.
- `le_pigeon_ne_ressemble_ni_a_l_oiseau_de_decor_ni_au_corbeau` (remplace
  `le_pigeon_ne_ressemble_pas_a_l_oiseau_de_decor`) : couleur principale `4` pour le pigeon perché, `1`
  pour le corbeau, `7` pour l'oiseau ; écarts de clarté d'au moins 2,5:1 ; aucun blanc ni nuage
  (`6`, `7`) dans le pigeon, ni cou irisé ni pattes corail (`D`, `E`, `9`) chez le corbeau et l'oiseau ;
  au moins deux fois plus de pixels pleins que l'oiseau ; œil orange perché, rouge sous un sourcil
  pendant l'annonce.
- `le_pigeon_se_detache_du_ciel_des_niveaux_4_et_5` et `le_pigeon_se_detache_du_ciel_de_la_descente` :
  mêmes pigeons, mêmes bandes de ciel (du perchoir au fond de la plongée, trame comprise, et dans la
  descente pour chaque position de la caméra), boîte du sprite passée à 4 px au-dessus de la boîte. LE
  CRITÈRE CHANGE, et c'est à signaler : l'ancien vérifiait que les couleurs `2` (le corps de l'ancien
  pigeon) et `0` contrastent d'au moins 3:1 avec le ciel. Gardé tel quel, il serait resté vert sans rien
  dire du nouveau pigeon, qui n'a plus de `2`. Or un plumage gris-bleu « clair à moyen » ne peut pas
  contraster par la clarté avec le corail : dans la palette, le gris-bleu `4` y est à 1,0:1, le `5` à
  1,5:1, et seul l'ardoise `2` (la masse sombre refusée) dépasse 3:1. Le nouveau critère
  (`pigeon_se_detache`) : le contour `0` à 3:1 au moins (inchangé ; mesuré de 6,0:1 sur le corail à
  16,1:1 sur l'or pâle), ET un plumage froid (plus de bleu que de rouge) sur un ciel chaud dont le rouge
  dépasse celui du plumage d'au moins 100. Le test vérifie aussi que le critère refuse le violet et la
  nuit.
- `scene::le_pigeon_a_une_image_par_phase` : l'élan pendant les pas 1 à 8 de la plongée et le piqué de
  9 à 32, quelle que soit l'horloge ; remontée retournée, qui commence ailes levées et change d'image
  7 fois (quatre battements).
- Planche de sprites `docs/planche-sprites.png` régénérée (seuls les pigeons changent).

Contrastes (WCAG, calculés) du nouveau pigeon et, entre parenthèses, de l'ancien corps `2` :

    ciel            contour 0   corps 4   aile 5   tête 3   (ancien 2)
    corail            5,98       1,01      1,53     2,32      (3,70)
    corail clair      8,89       1,51      1,03     3,46      (5,50)
    orange           10,33       1,75      1,13     4,01      (6,39)
    or               13,29       2,25      1,45     5,17      (8,22)
    violet            2,44       2,42      3,75     1,05      (1,51)
    nuit              1,18       5,02      7,78     2,19      (1,37)

Sur les ciels chauds, le pigeon se lit par son contour et par sa teinte froide, plus par sa masse ; sur
le violet et la nuit du clocher (que les tests de contraste ne couvrent pas, ni avant ni maintenant), il
se voit mieux qu'avant : le corps clair et l'aile y tranchent, là où l'ancienne ardoise s'y perdait.

### Captures `docs/qa/66-*` (8 images)

Build de QA (`./build.sh --qa`) d'une extraction propre de bfbd8d8 (dessin et scripts des captures ;
les commits suivants n'ajoutent qu'un test ignoré et les images), servi sur 127.0.0.1 (port libre,
serveur arrêté à la fin), Chromium 1208 sans tête piloté par CDP (`docs/capture-visibilite.mjs`, onglet
au premier plan et canvas focalisé, `GRILLE=1` : grille logique 320 x 180 de chaque capture), 960 x 540
(x3). Scripts écrits par le test ignoré `ecrire_script_pigeons_66` (`script-66-N`, `touches-66-N`,
`trace-66-N` : niveaux 5, 7, 8, 9 et 11, tous les pigeons du jeu suivis, avec leur phase) ; numéros
d'image de ces traces.

- `66-clocher-annonce` (image 795) : le pigeon de la corniche du clocher dans son annonce, sur le ciel de
  nuit, Pépin dessous sur la galerie ;
- `66-clocher-pique` (822) : le même en piqué sur le violet, juste derrière Pépin qu'il manque ;
- `66-vents-pique` (915) : le pigeon du nuage (38, 3) des vents d'altitude en piqué sur le corail ;
- `66-mer-pique` (1172) : le pigeon de la mer de nuages en piqué sur le corail clair, Pépin hissé par la
  montgolfière ;
- `66-descente-pique` (1125) : dans la descente, le premier pigeon en piqué, le second perché sur son
  nuage, le corbeau à gauche : les deux ennemis volants côte à côte, en vrai ;
- `66-descente-ecrase` (1114, script `script-66-ecrase`, voir plus bas) : le premier pigeon écrasé sur
  son nuage, Pépin juste au-dessus (il vient de rebondir), le second perché sur le nuage voisin, le
  corbeau en bas à gauche ;
- `66-planche-etats` : planche zoomée (x3 de plus, soit x9) des HUIT images, découpées dans les grilles
  logiques de vraies captures : perché (images 760 et 775), annonce (795), élan (809), piqué (822),
  remontée ailes levées (841) et basses (849), au clocher ; écrasé (1114) dans la descente ;
- `66-zoom-pigeon-corbeau-oiseau` : même procédé, x4 de plus (x12) ; en haut, Au-dessus des nuages :
  pigeon perché (1700), corbeau (1200), oiseau de décor (1700, même image que le pigeon) ; en bas, la
  descente (1125) : pigeon en piqué, corbeau, pigeon perché.

Chaque morceau des planches a été retrouvé pixel à pixel dans sa grille (le sprite attendu, avec ou
sans miroir, 100 % de ses pixels à leur place). Le robot ne tombe jamais sur un pigeon : pour l'image
« écrasé », le test ignoré `ecrire_parcours_pigeon_ecrase_66` cherche dans la descente une suite simple
qui l'écrase (courir 46 pas, planer 89 pas, tomber : pigeon écrasé au pas 197 du niveau), et le script
`script-66-ecrase` est celui du robot jusqu'au début de la descente, suivi de ce parcours.

`docs/mesure-capture.py <capture> 3` sur les 8 : 0 pixel hors palette, 0 pixel semi-transparent, 0 bloc
flou. Compteur de vies : 5 sur les six captures entières.

Chaque capture regardée. Le piqué se lit-il comme un oiseau ? Oui, à mon avis : sur les quatre ciels
(violet du clocher, corail des vents, corail clair de la mer, corail et orange de la descente), on voit
un oiseau ailes relevées en V, rémiges sombres écartées, le corps sur la diagonale, la tête devant et en
bas ; on ne le prend plus pour un poisson ni pour une feuille. L'œil rouge et le bec clair ne font que 1
et 2 pixels : à l'échelle du jeu, c'est la silhouette qui dit où il va, pas le bec. Perché, c'est un
pigeon (corps rond, petite tête, cou turquoise, pattes corail). L'annonce est l'image la moins parlante :
on voit les ailes se dresser et le corps s'aplatir, à 0,3 s du piqué. Côte à côte, impossible de
confondre le pigeon gris-bleu au cou turquoise avec le corbeau presque noir ni avec le petit oiseau
blanc.

### Vérification sur une extraction propre de c1e9fd9 (dernier commit de code et des captures)

`git archive`, nouveau dossier daté sous `$TMPDIR` :

- `./docker-cargo.sh test --workspace --no-default-features` : 374 (core) et 337 (game : un test de plus,
  `le_pigeon_en_pique_fond_sur_pepin_le_long_de_sa_diagonale`) tests, tous verts (1 + 24 ignorés :
  `ecrire_script_pigeons_66` et `ecrire_parcours_pigeon_ecrase_66` s'ajoutent), dont les tests de
  faisabilité des onze niveaux, les tests des pigeons, `courbe_de_difficulte_dans_les_bandes` (encore
  actif sur cette branche) et les deux parties du robot ;
- `./docker-cargo.sh lint` : `FMT-OK`, `LINT-OK` ;
- `./build.sh` : `dist/` produit, wasm 498 972 octets (497 743 à la carte 6.4b : + 1 229, deux images de
  plus et des grilles plus grandes), 0 occurrence des noms interdits, seule URL
  `http://www.w3.org/2000/svg`, « Essai chat » absent de `dist/` ;
- parties du robot : `ecrire_script_navigateur` : 21 414 images, 0 mort, 15 gouttes, empreinte
  `1f3190119f6bd9b3`, IDENTIQUE à celle de la carte 6.4b (le dessin ne touche pas au jeu) ;
  `ecrire_script_partie_complete` : 25 636 images, 0 mort, 33 gouttes, empreinte `2add196d123e37e1`
  (aucune valeur de référence relevée depuis la carte 6.4a, pas de comparaison) ;
- `ecrire_script_pigeons_66` : ses quinze fichiers `script-66-N`, `touches-66-N`, `trace-66-N` identiques
  à l'octet à ceux qui ont servi aux captures. `script-66-ecrase` n'est pas écrit par un test : ce sont
  les 912 premières images de `script-66-11.txt` (jusqu'au premier pas de la descente) suivies de
  `parcours-66-ecrase.txt`.
- Pas de mesure de difficulté (consigne allégée du 2026-11-16) : rendu seulement, rien à mesurer.

Commits : 8ca5e33 (dessin, scène, tests, planche), bfbd8d8 (scripts des captures), b518829 (parcours qui
écrase un pigeon), c1e9fd9 (captures), cette section. Le README n'est pas retouché (nombre de tests, poids
du wasm) : la carte 6.5 travaille en parallèle sur `main`, le coordinateur reportera les chiffres à la
fusion.

### Ce qui n'est PAS testé

- Aucun humain n'a regardé le nouveau pigeon : « le piqué se lit comme un oiseau » est mon jugement sur
  des captures fixes, à confirmer par celle qui l'a demandé, en jouant (le piqué dure 0,53 s, dont
  0,13 s d'élan : l'élan se voit-il seulement en mouvement ?).
- Le changement de critère du test de contraste (teinte froide sur ciel chaud plutôt que clarté) est un
  choix : sur le corail, le corps du pigeon a la même clarté que le ciel (1,0:1), seuls le contour,
  la tête et la teinte l'en séparent. Sur les captures, il se voit ; un écran délavé (téléphone en plein
  soleil) pourrait réduire la différence de teinte.
- L'annonce reste l'image la moins nette (ailes dressées, corps aplati) ; elle n'a pas été comparée à
  l'ancienne en mouvement.
- Chromium sans tête seulement, à 960 x 540 : ni vrai téléphone, ni Safari, ni Firefox.

## Vents d'altitude : courant ascendant qui finit trop tôt (carte 6.8)

Retour d'AGA en jouant (mot pour mot) : « dans le niveau précédent la mer de nuage il y a un courant
ascendant qui finit un peu trop tôt et rend le saut compliqué » (niveau 7, `core/src/levels/vents.rs`).
Au-dessus du haut d'un courant, plus rien ne porte Pépin : il plane en descendant. Marge entre le haut de
chaque courant et le dessus de son îlot d'arrivée (premier solide à droite) :

| Courant (colonnes) | Îlot d'arrivée      | Avant          | Après          | Retouche                                  |
| ------------------ | ------------------- | -------------- | -------------- | ----------------------------------------- |
| 16 à 18            | 21 à 31, rangée 4   | 32 px          | 32 px          | aucune                                    |
| 50 à 52            | 56 à 63, rangée 3   | 32 px          | 32 px          | aucune                                    |
| 97 à 99            | 103 à 108, rangée 3 | 0 px           | 32 px          | courant prolongé : rangée 3 → rangée 1    |
| 127 à 129          | 132 à 141           | 16 px (rang 2) | 32 px (rang 3) | îlot et lanterne 136 descendus d'un rang  |

- Le 3e courant était celui du saut « compliqué » : il lâchait Pépin à la hauteur de son îlot, trois tuiles
  de vide plus loin.
- Le 4e commence déjà à la rangée 1 ; la rangée 0 est le haut de l'écran (le prolonger ferait planer Pépin
  hors du cadre). C'est donc son îlot (colonnes 132 à 141) qui descend d'une rangée, avec sa lanterne
  (colonne 136, rangée 1 → 2, toujours posée dessus). Il est maintenant à la hauteur du corbeau (147, 3),
  dont la patrouille commence 2 px après le bord droit de l'îlot.
- Rien d'autre ne bouge : corbeaux (76, 3) et (147, 3), pigeons, nuage friable, gouttes à leur place ;
  aucune entité dans un courant. Aucun test ajouté (consigne d'AGA) ; aucun test existant n'a cassé.

### Preuves (retouche d'un niveau : tests ciblés, pas de suite complète ni de lint)

Extraction propre de 6085bd5 (`git archive`, nouveau dossier daté sous `$TMPDIR`) :

- `./docker-cargo.sh test -p core -- levels::vents zones faisabilite` : 78 tests verts (dont la
  faisabilité des onze niveaux, `niveau_7_exige_les_courants_ascendants`, chargement et lint des grilles) ;
- `./build.sh` : `dist/` produit, wasm 498 972 octets, 0 occurrence des noms interdits, seule URL
  `http://www.w3.org/2000/svg` ; `./build.sh --qa` : `dist-qa/` pour la capture ;
- `ecrire_script_ennemis_64b` (test ignoré existant, lancé à la main pour la capture) : niveau 7 rejoué par
  le robot sans mort (`assert_eq!(morts, 0)`), 2 667 images.

Dans le worktree, même grille (seuls les deux tests retirés ensuite diffèrent) : les deux parties du robot,
`le_robot_finit_tous_les_niveaux_sans_mourir` et `la_partie_complete_joue_le_bonus` (toutes les gouttes,
bonus joué), vertes à 0 mort ; `la_manche_a_air_s_etire_vers_son_courant` et les tests du pigeon verts. Le
parcours du robot est calculé à chaque partie : aucun fichier de parcours à régénérer.

Capture `docs/qa/68-vents-courant-prolonge.png` : build de QA servi sur 127.0.0.1 (port libre, serveur
arrêté à la fin), Chromium 1208 sans tête piloté par CDP (`docs/capture-visibilite.mjs`, onglet au premier
plan et canvas focalisé), 960 x 540, script `script-64b-7.txt` (touche 7 puis le parcours du robot), image
1671. Pépin sort du haut du courant prolongé (x = 1 600, y = 28 dans la trace), pieds 6 px au-dessus du
dessus de l'îlot 103-108 visible en dessous à droite ; il monte encore jusqu'à y = 16,7 (pieds 17 px
au-dessus de l'îlot) avant de planer. `docs/mesure-capture.py` : 0 pixel hors palette, 0 bloc flou.
Regardée.

Commits : 7db8c8c (grille, doc du niveau, deux tests), 6085bd5 (les deux tests retirés), cette section et
la capture.

### Ce qui n'est PAS testé

- Aucun humain n'a rejoué le saut : « assez tôt maintenant » est à confirmer par AGA en jouant.
- La capture `64b-vents-corbeau-ilot.png` (carte 6.4b) montre l'îlot 132-141 à son ancienne hauteur : non
  refaite (une seule capture demandée).
- Le corbeau (147, 3) est maintenant à la hauteur de l'îlot de la lanterne 136 : la façon de l'écraser en
  partant de cet îlot (se laisser tomber dessus, carte 6.4b) n'a été vérifiée que par le robot qui passe
  sans mourir, pas geste par geste.
- Chromium sans tête seulement, 960 x 540 : ni vrai téléphone, ni Safari, ni Firefox.

## Mer de nuages : première goutte prenable (carte 6.7)

Retour d'AGA en jouant : « dans la mer de nuage la première goutte est trop haute et donc impossible à
avoir ».

### Ce qui a été fait

- `core/src/levels/mer.rs` seulement : la goutte passe de (17, 3) à (16, 4), une rangée plus bas et une
  colonne plus près du bord du départ ; le vent `>` du départ perd sa ligne 4 (lignes 5 à 7, colonnes
  14 à 26, inchangées) pour lui laisser la place. Doc du niveau mise à jour. Rien d'autre dans `core/`,
  ni dans la physique, ni dans le robot.
- Aucun nouveau test (consigne d'AGA) ; aucun test existant n'a cassé. Le parcours du robot de QA n'est
  pas enregistré dans le dépôt (il est recalculé à chaque partie) : rien à régénérer.

### Pourquoi « faisable » n'était pas « prenable »

Mesuré avec une sonde jetable (Pépin court vers la droite, saute près du bord en tenant la touche N
images, puis rouvre le parapluie en redescendant ; vraie physique, image par image ; sonde retirée,
non versionnée). Un saut complet monte de 54 px (57 px pour la formule v²/2g).

- Ancienne place (17, 3) : seul un saut complet, parti des 6 derniers pixels du bord, la touchait. Le
  robot de faisabilité, qui essaie tout à l'image près, la trouvait ; un humain, non.
- Nouvelle place (16, 4) : un saut à 79 % de la hauteur maximale (touche tenue 9 images) la prend depuis
  n'importe où dans les 33 derniers pixels avant le bord (2 tuiles) ; un saut complet, depuis les 57
  derniers. Le bas de la goutte (y = 76) est 16 px sous le sommet d'un saut complet depuis le départ.
  Un sautillement (touche tenue 6 images ou moins) passe dessous : il faut un vrai saut.

Les deux autres gouttes, revues avec la même sonde, ne sont pas limites en hauteur, donc pas déplacées :

- (84, 2), entre les deux rideaux de grêle : 48 px sous le sommet d'un saut depuis l'étage. Sa
  difficulté est la distance (100 px au-delà du bord, en planant sous la grêle) : un saut tenu, parapluie
  ouvert, la prend depuis les 35 derniers pixels du bord ; un saut à 79 %, depuis les 11 derniers
  seulement. C'est le défi prévu ; à surveiller si AGA la trouve trop dure.
- (112, 1), au-dessus du moment fort : 32 px sous le sommet d'un saut depuis la tour ; un saut à 79 %
  la prend depuis n'importe où sur la tour, un saut complet aussi, sauf depuis les 7 derniers pixels du
  bord (il passe au-delà).

### Preuves

Lancées dans le worktree, avec le contenu exact du commit eb1ad2d :

- `./docker-cargo.sh test -p core levels::mer` : 6 verts ; `-p core zones` : 54 verts (tous les niveaux
  se chargent, 3 gouttes, robot maladroit compris) ; `-p core faisabilite` : 25 verts ;
- `./docker-cargo.sh test -p game --no-default-features le_robot_finit_tous_les_niveaux_sans_mourir` :
  vert (0 mort, fin atteinte) ;
- `./build.sh` : `dist/` produit, 0 nom interdit, seule URL `http://www.w3.org/2000/svg`.

Capture `docs/qa/67-mer-goutte.png` : build de QA (`./build.sh --qa`) servi sur 127.0.0.1 (port libre,
serveur arrêté à la fin), Chromium 1208 sans tête par CDP (`docs/capture-visibilite.mjs`, onglet au
premier plan, canvas focalisé), 960 x 540 (x3). Script : Entrée au titre, touche 8 du build de QA à
l'image 229, puis 300 images sans rien (`touche:67-mer-goutte=229:Digit8:300`), capture à l'image 530 :
Pépin au départ du niveau 8, la goutte au-dessus des traînées du vent, juste après la manche à air du
bord. `docs/mesure-capture.py`, K = 3 : 0 pixel hors palette, 0 semi-transparent, 0 bloc flou.
Regardée.

### Ce qui n'est PAS testé

- Personne n'a repris la goutte en jouant : « un vrai saut depuis le bord la prend » vient de la sonde,
  pas d'une main humaine (AGA rejugera).
- Seuls les cinq contrôles de la retouche d'un niveau ont tourné (consigne allégée) : ni la suite
  complète, ni le lint, ni la partie complète du robot (`la_partie_complete_joue_le_bonus`, qui ramasse
  toutes les gouttes) ; `zones` vérifie seulement que le robot du cœur atteint les trois gouttes.
- Chromium sans tête seulement : ni vrai téléphone, ni Safari, ni Firefox.

## Robot de QA plus rapide : un niveau par fil (carte 6.9)

`tous_les_parcours()` (game/src/bot_qa/mod.rs) cherchait le parcours des 11 niveaux l'un après l'autre.
Il en cherche maintenant un par fil (`std::thread::scope`, comme `tous_les_parcours_complets()`), rangés
dans l'ordre des niveaux, même message de panique. `ecrire_scripts_des_ajouts` (scripts.rs : scripts 64,
64b, 66) cherche aussi ses niveaux en parallèle avant de les rejouer dans l'ordre. Code de test
seulement, rien dans core/ ni dans le jeu, aucun nouveau test.

### Preuves

Deux extractions propres, `git archive` de 589d6eb (avant) et de carte-6.9 au commit 96a84eb (après) :

- durée de `./docker-cargo.sh test -p game --no-default-features -- --exact
  bot_qa::tests::le_robot_finit_tous_les_niveaux_sans_mourir` (mesurée une fois chaque) : avant
  534,8 s (charge 6,9 au début, 7,8 à la fin) ; après 259,4 s (charge 8,0 puis 5,3). Le temps est
  maintenant celui de la recherche du niveau le plus long, plus le rejeu de la partie ;
- parcours identiques à l'octet : `ecrire_script_navigateur` donne la même empreinte avant et après
  (71df3e47c4844537 : 21 422 images, 0 mort, 14 gouttes, chrono 18 878 pas) et `ecrire_script_ennemis`
  aussi ; les 17 fichiers écrits dans `target/qa/` (script.txt, trace.csv, script-64-1 à 5,
  touches-64-*, trace-64-*) sont identiques (`cmp`) ;
- `./docker-cargo.sh test -p game --no-default-features bot_qa` : 19 verts, 19 ignorés ;
- `./docker-cargo.sh lint` : FMT-OK, LINT-OK ; `./build.sh` : `dist/` produit.

### Ce qui n'est PAS testé

- `ecrire_script_ennemis` n'a pas été plus rapide pendant cette mesure (388 s avant, 394 s après, lancé
  en même temps que d'autres tests, charge jusqu'à 13) : son niveau 5 domine. Les scripts 64b et 66
  (même fonction) n'ont pas été relancés.
- Pas de suite complète (consigne vitesse) ; une seule mesure de chaque côté, sur une machine chargée.

## Sauvegarde : gouttes en poche et records (carte 7.2)

`game/src/sauvegarde.rs` : `Sauvegarde` (porte-monnaie `gouttes_en_poche`, `gouttes_total_gagnees`,
`meilleur_score`, `meilleur_temps`, `parties_finies`, `skins_debloques` avec « pepin » et « pepinette »
d'office, `skin_choisi`), texte `cle=valeur` avec `version=1`. Lecture tolérante : texte absent, vide,
sans `version=1` → défaut ; champ inconnu ignoré ; valeur illisible → défaut du champ ; skin choisi non
débloqué → « pepin ». Méthodes `ajouter_gouttes`, `depenser(prix) -> bool`, `debloquer`,
`noter_partie(score, temps) -> nouveau record ?` (la boutique, carte 7.3, se servira de `depenser`).

- Stockage web : `localStorage["pepin.sauvegarde"]` par un petit plugin miniquad écrit dans
  `web/index.html` (`pepin_sauvegarde_lire` / `pepin_sauvegarde_ecrire`, texte UTF-8 par la mémoire du
  wasm, tout dans un `try` : stockage refusé → rien lu, rien écrit). Natif/tests : en mémoire.
- Branchement (`app.rs`) : niveau réussi (drapeau, boss, bonus) → ses gouttes au porte-monnaie +
  écriture ; niveau recommencé ou perdu → rien (ses gouttes sont remises à zéro). Écran de fin →
  `noter_partie(score, chrono)` + écriture.
- Affichage : écran titre, « GOUTTES : N » en haut à gauche et « RECORD : score » en haut à droite (dès
  une partie finie), échelle 1 au-dessus du titre ; écran de fin, « NOUVEAU RECORD ! » clignote en or au
  milieu de la ligne SCORE.
- Un seul test ajouté (`sauvegarde::tests`, aller-retour texte, textes abîmés, dépense refusée).

### Preuves

- Robot de QA inchangé : `ecrire_script_navigateur` redonne 71df3e47c4844537 (21 422 images, 0 mort,
  14 gouttes, score 2500, chrono 18 878 pas), la même empreinte qu'en 6.9.
- Vrai navigateur (Chromium sans tête, CDP, build de QA servi sur 127.0.0.1) avec
  `docs/capture-sauvegarde.mjs` : première visite, rien d'enregistré ; le script complet du robot
  rejoué, le porte-monnaie monte niveau par niveau (1, 2, 4, 5, 7, 10, 13, 14) et la fin écrit
  `meilleur_score=2500`, `parties_finies=1`, `meilleur_temps=18877` ; page rechargée → la sauvegarde
  est toujours là et le titre affiche « GOUTTES : 14 » et « RECORD : 2500 » ; `localStorage` vidé →
  « GOUTTES : 0 », pas de RECORD ; `localStorage` qui lève une exception → le jeu démarre quand même ;
  0 erreur ni avertissement dans la console sur tout le parcours. Captures
  `docs/qa/72-titre-gouttes-record.png` et `docs/qa/72-fin-nouveau-record.png` (regardées : rien ne se
  chevauche ; `mesure-capture.py` : 0 pixel hors palette, 0 flou).
- Extraction propre (`git archive` de carte-7.2 au commit 6ff6f2e) : `./docker-cargo.sh test -p game
  --no-default-features sauvegarde` 1 vert ; `... bot_qa` 19 verts, 19 ignorés ; `./docker-cargo.sh
  lint` FMT-OK, LINT-OK ; `./build.sh` : `dist/` produit, 0 nom interdit, seule URL celle du SVG.

### Ce qui n'est PAS testé

- Ni vrai téléphone, ni Safari (navigation privée), ni Firefox : le refus de `localStorage` est simulé
  dans Chromium.
- Deux onglets ouverts en même temps : le dernier qui écrit gagne (pas de fusion).
- La boutique (dépenser des gouttes, choisir un skin) : carte 7.3.
- Pas de suite complète (`test --workspace`) : seulement les commandes de la carte ci-dessus.

## Skins : Pépinette, la version fille de Pépin (carte 7.1)

Catalogue `game/src/skins.rs` : `enum Skin { Pepin, Pepinette }`, avec pour chacun un nom affiché
(« PÉPIN », « PÉPINETTE »), un prix en gouttes (0 et 0 : Pépinette est débloquée dès le départ) et son
jeu d'images (`sprites::JeuImages` : repos, course, saut, chute, attente, plané, réapparition, icône des
vies). Ajouter un skin = un fichier dans `sprites/` + une variante (le compilateur signale chaque `match`
oublié). Le skin actif est un paramètre du rendu (`scene::dessiner`, `hud::dessiner`, champ
`Application::skin`, Pépin par défaut) : `dessiner_pepin` et `image_attente` prennent leurs noms dans le
jeu d'images, la logique d'animation est la même pour tous. Rien dans core/ (physique, boîte 10 x 14).
Build de QA seulement : la touche K passe au skin suivant (`entrees::skin_suivant_demande`, faux sans
`debug_niveaux`, donc inerte en production).

Pépinette (`sprites/pepinette.rs`) : les 16 images de Pépin, une à une, mêmes tailles, même silhouette,
mêmes pieds, parapluie au même endroit, plus l'icône des vies (10 x 10, sa tête). Différences : un nœud
corail à gauche sur la tête, des cils (un pixel sombre en haut, à l'extérieur de chaque œil), une petite
robe corail (les rangées d'ombre sous le corps), un parapluie cyan de la pluie (`D`, plis `6`) au lieu du
corail. Joues roses gardées ; étincelles de la réapparition (`mort2`) reprises de Pépin.

Un seul test ajouté (nouvelle mécanique) : `chaque_skin_fournit_toutes_les_images_aux_tailles_de_pepin`
(17 noms par skin, tous présents dans ses sprites, valides, à la taille de l'image de Pépin qu'ils
remplacent, aucun nom partagé entre deux skins, Pépinette à 0 goutte). Les tests d'attente de `scene.rs`
passent le jeu d'images de Pépin (`&pepin::IMAGES`), sans autre changement. `docs/planche-sprites.png`
régénérée (Pépinette après Pépin).

### Preuves

Extraction propre (`git archive carte-7.1`, commit 95428a0) :

- `./docker-cargo.sh test -p game --no-default-features sprites` : 54 verts ; `... skins` : 1 vert ;
- `./docker-cargo.sh test -p game --no-default-features bot_qa` : 19 verts, 19 ignorés ;
- jeu par défaut identique : `ecrire_script_navigateur` donne la même empreinte qu'à la carte 6.9
  (71df3e47c4844537 : 21 422 images, 0 mort, 14 gouttes, chrono 18 878 pas) ;
- `./docker-cargo.sh lint` : FMT-OK, LINT-OK ; `./build.sh` : `dist/` produit, aucun `/home`, aucune
  adresse, seule URL `http://www.w3.org/2000/svg`.

Captures (build de QA `./build.sh --qa` servi sur 127.0.0.1, Chromium 1208 sans tête par CDP,
`docs/capture-visibilite.mjs`, 960 x 540, x3), parcours du robot (`target/qa/script.txt`) avec la touche K
à l'image 5 (`TOUCHES=5:KeyK`) :

- `docs/qa/71-planche-pepin-pepinette.png` : les 17 images, Pépin en haut, Pépinette dessous (écrite par
  `python3 docs/planche-skins.py`, qui lit les grilles des deux fichiers de sprites ; x8) ;
- `71-toits-plane.png` (image 1183) et `71-toits-course.png` (1419) : niveau 1, ciel de nuit ;
- `71-mer-course.png` (16 747) et `71-mer-plane.png` (17 043) : niveau 8, ciel de couchant.

`docs/mesure-capture.py` : 0 pixel hors palette, 0 bloc flou sur les quatre captures ; planche : 0 pixel
hors palette. Regardées : Pépinette se lit comme « Pépin fille » (nœud et robe corail, cils, parapluie
cyan bien distinct du corail de Pépin), l'icône des vies du HUD porte le nœud.

### Ce qui n'est PAS testé

- Personne n'a joué avec Pépinette à la main ; le choix du skin à l'écran (7.3) et sa sauvegarde (7.2)
  ne sont pas faits ici.
- Sur le ciel de couchant (corail), le nœud et la robe corail se détachent surtout par leur contour
  sombre : lisible sur la capture, à rejuger sur un écran délavé.
- Pas de suite complète (consigne vitesse).

## Musiques à débloquer (carte 7.7)

Nouveau module `game/src/musiques.rs` : catalogue `Musique` (identifiant stable, nom en capitales,
prix en gouttes d'or, partition). `orage` (la boucle d'origine, toujours dans `core/src/sons.rs`,
inchangée) : 0 goutte ; trois nouvelles à 25 gouttes, écrites en notes MIDI et jouées par le même
synthétiseur 8 bits (aucun fichier, aucune dépendance) :

- `toits_de_nuit` « TOITS DE NUIT » : nocturne en ré mineur, arpèges lents (11,52 s) ;
- `soleil` « SOLEIL » : entraînante en do majeur, basse oum-pa et charleston de bruit (11,52 s) ;
- `valse_des_nuages` « VALSE DES NUAGES » : valse à trois temps en sol majeur (10,56 s).

`Audio` charge les quatre boucles au démarrage et joue `musique_active` (défaut : Orage) ;
`choisir_musique` change de musique (si elle tourne : arrêt de l'ancienne, départ de la nouvelle).
Même volume, même réglage M / menu, même arrêt en pause. La boutique branchera le choix du joueur
et la sauvegarde. Build de QA seulement (`debug_niveaux`) : F11 = musique suivante (F1 à F10 sont
prises ; dans un vrai navigateur F11 bascule aussi le plein écran, sans gêne).

### Preuves (extraction propre `git archive carte-7.7`)

- `test -p game --no-default-features audio` : 4 verts ; `... musique` : 2 verts (dont le seul
  nouveau test : chaque musique a une partition non vide dont les voix bouclent ensemble, durée
  8-14 s, pic 0,2-1, début et fin silencieux : pas de clic à la jonction) ;
- volume efficace d'une boucle : orage 0,185, toits_de_nuit 0,184, soleil 0,180, valse 0,180 ;
- extraits de 10 s à écouter : `docs/qa/77-musique-*.wav` (régénérés par le test ignoré
  `ecrire_les_extraits_wav_de_qa`) ;
- suite complète `test --workspace --no-default-features` : core 374 verts, game 337 verts ;
- `./docker-cargo.sh lint` : FMT-OK, LINT-OK ; `./build.sh` : `dist/` sans nom interdit,
  game.wasm 501 861 octets (gzip 201 842), build de QA 511 997 ;
- Chromium sans écran (CDP), build de QA servi en local : son activé au titre, puis F11 x4, 4 s
  chacune. Boucles en cours observées (durée du tampon) : 9,6 (orage) + 2 (pluie) → 11,52 + 2 →
  11,52 + 2 → 10,56 + 2 → 9,6 + 2 : toujours une seule musique ; M coupe tout. Console : aucune
  erreur. Même scénario sur `dist/` (production) : F11 ne change rien (orage reste seule).

### Ce qui n'est PAS testé

- Le son à l'oreille (Chromium sans écran : rien n'est entendu ; écouter les WAV ci-dessus) ;
- aucun écran de choix ni sauvegarde (cartes boutique) ; téléphone et Safari non essayés.

## Parapluie fermé tenu par la poignée : Pépin et Pépinette (carte 7.1b)

Demande d'Aga (deux fois) : le parapluie fermé était tenu à l'envers (crosse orange en l'air au-dessus
de la tête, embout de métal au sol : on croyait Pépin le tenir par la pointe). Retourné dans les 11
images où il est fermé (repos x2, course x4, saut, chute, cligne, ciel, tapote), pour Pépin
(`sprites/pepin.rs`) ET Pépinette (`sprites/pepinette.rs`) : embout de métal en haut (rangée 1), toile
roulée au-dessus (corail chez Pépin, cyan chez Pépinette, plis, sangle blanche), fine sous l'embout et
plus large vers le bas, crosse orange en bas (rangées 8 à 12), à hauteur du flanc, le bec recourbé vers
le corps. Même dessin à la même place dans les 11 images (pas de tremblement en course) ; TAPOTE : le
parapluie descend d'un pixel, la crosse vient frapper le sol. Plané (déjà dans le bon sens), mort et
réapparition (pas de parapluie fermé) : inchangés. Corps, pieds, tailles, boîtes, physique : inchangés
(colonnes 0 à 9 identiques ; seul le contour du corps à la colonne 10-11 suit le nouveau parapluie).

Aucun nouveau test. Un test existant décrivait l'ancien sens et a été mis à jour (même nom, mêmes
contrôles, rangées inversées) : `le_parapluie_ferme_a_une_crosse_une_toile_repliee_et_un_embout`
(`game/src/sprites/mod.rs`) : orange seulement dans les rangées 8 à 12, crochet, toile et sangle dans les
rangées 2 à 7, plus large en bas qu'en haut, embout dans les rangées 0 à 1.

### Preuves

Extraction propre (`git archive` du commit 661bc58) :

- `./docker-cargo.sh test -p game --no-default-features sprites` : 54 verts ; `... skins` : 1 vert ;
  `... scene` : 32 verts (attente, tapote) ;
- `./docker-cargo.sh test -p game --no-default-features -- --exact
  bot_qa::tests::le_robot_finit_tous_les_niveaux_sans_mourir` : vert en 491,7 s (machine chargée,
  charge 20 à 32), fin atteinte, 0 mort ;
- `./build.sh` : `dist/` produit (game.wasm 509 466 octets), aucun `/home`, seule URL
  `http://www.w3.org/2000/svg`.

Images : `docs/qa/71-planche-pepin-pepinette.png` régénérée (`python3 docs/planche-skins.py`, x8 ; 0 pixel
hors palette), `docs/planche-sprites.png` réécrite par son test ; `docs/qa/71b-parapluie-avant-apres.png`
(x12) : de haut en bas Pépin avant, Pépin après, Pépinette avant, Pépinette après ; repos, course (2
images), saut ; `docs/mesure-capture.py` : 0 pixel hors palette, 0 bloc flou. Regardée : oui, on lit
« il tient son parapluie par la poignée » : la crosse orange est contre le flanc, à hauteur de la main,
le parapluie pointe vers le haut ; pareil pour Pépinette (toile cyan).

### Ce qui n'est PAS testé

- Pas de capture en jeu (navigateur) ni de lint (`./docker-cargo.sh lint`) refaits ; l'empreinte
  d'images du robot n'a pas été recalculée (elle change, c'est attendu : seules les images changent).
- Pas de suite complète (consigne vitesse).

## Boutique : dépenser les gouttes d'or, choisir son skin (carte 7.5)

Écran titre : nouvelle ligne BOUTIQUE (JOUER, BOUTIQUE, COMMENT JOUER, SON ; le menu remonte à y = 66, Pépin
court à gauche du menu, avec le skin choisi). La boutique (`game/src/boutique.rs`, logique pure ;
`game/src/ecrans/boutique.rs`, dessin et zones cliquables) montre le porte-monnaie (goutte + nombre, en
haut à gauche), une rangée d'ONGLETS, une ligne par objet (aperçu animé du skin qui court, nom, prix ou
GRATUIT, état ÉQUIPÉ / ACHETÉ / À ACHETER), RETOUR et le rappel des touches, sur le ciel du titre.

- Organisation par catégories : `enum Categorie` (un onglet = une variante, sa liste d'`Objet`s :
  identifiant, nom, prix, aperçu ; débloqué / équipé se lisent dans la sauvegarde). Aujourd'hui un seul
  onglet, SKINS (catalogue de la carte 7.1). Ajouter traînées, motifs de parapluie, palettes ou musiques =
  une variante + sa ligne dans `TOUTES` et dans chaque `match` ; l'écran n'a rien à changer. Un onglet
  sans objet n'est pas affiché. Rien de tout cela ne touche au jeu : objets cosmétiques seulement.
- Commandes : haut / bas (ligne), gauche / droite (onglet ; OUI / NON dans la confirmation), Entrée
  (choisir), Échap (retour au titre, curseur sur BOUTIQUE) ; clic ou toucher sur une ligne, un onglet ou
  un bouton (`Navigation` gagne `gauche` et `droite`, lues sur les touches de course).
- Objet débloqué : Entrée l'équipe (`skin_choisi`), la sauvegarde est écrite tout de suite et le rendu
  prend le skin (`Application::skin`, appliqué aussi au démarrage depuis la sauvegarde ; id inconnu :
  Pépin). Objet à acheter : « ACHETER POUR N ? » avec OUI / NON (NON choisi d'abord) ; OUI débite les
  gouttes (`Sauvegarde::depenser`), débloque (`debloquer`) et enregistre. Pas assez : « IL TE MANQUE
  N GOUTTES » + OK, rien ne change.
- `Skin::id` / `Skin::depuis_id` : l'identifiant gardé dans la sauvegarde (« pepin », « pepinette »).
- Build de QA seulement (`debug_niveaux`, et tests) : un objet factice payant « FACTICE QA » (60 gouttes,
  ne s'équipe pas) pour essayer l'achat sans ajouter de vrai skin payant, et la touche G
  (`entrees::gouttes_qa_demandees`) qui met 100 gouttes en poche (enregistrées). Absents de `dist/`
  (vérifié : 0 occurrence de « factice_qa » / « FACTICE QA » ; G inerte sans `debug_niveaux`).

Un seul test ajouté (nouvelle mécanique) : `boutique::tests::equiper_puis_acheter_avec_confirmation`
(Pépinette équipée d'un appui ; factice refusé sans gouttes, « il te manque 60 » ; avec 100 : NON par
défaut, puis OUI, 40 restantes, ACHETÉ, skin inchangé ; RETOUR quitte). Tests existants adaptés au menu à
4 lignes (`les_menus_ont_les_bonnes_lignes`, `la_ligne_son_du_menu_...`, `un_clic_sur_une_ligne_la_valide`,
mise en page du titre dans `ecrans`).

### Preuves

Extraction propre (`git archive`, commit 23cef84 ; la touche G, ajoutée ensuite, a été revérifiée sur
une nouvelle extraction : lint, tests `boutique`, `sauvegarde`, `skins`, `entrees`, build) :

- `./docker-cargo.sh test --workspace --no-default-features` : core 374 verts (1 ignoré), game 339 verts
  (25 ignorés), 0 échec ;
- `./docker-cargo.sh lint` : FMT-OK, LINT-OK ;
- `./build.sh` : `dist/` produit, 0 nom interdit, seule URL `http://www.w3.org/2000/svg`, aucun `/home`.

Captures : build de QA (`./build.sh --qa`) servi sur 127.0.0.1 (port libre, serveur arrêté à la fin),
Chromium 1208 sans tête par CDP, onglet au premier plan, canvas focalisé, horloge virtuelle de
`docs/pilote-qa.js` ; pilote `node docs/capture-boutique.mjs PORT DOSSIER` (sauvegarde préparée dans
localStorage, puis appuis de touches). 960 x 540 (x3) sauf la dernière :

- `75-titre-ligne-boutique.png` : titre, BOUTIQUE choisie, 30 gouttes en poche ;
- `75-boutique.png` : la boutique (onglet SKINS, Pépin ÉQUIPÉ, Pépinette GRATUIT, factice 60 gouttes) ;
- `75-achat-refuse.png` : factice avec 30 gouttes, « IL TE MANQUE 30 GOUTTES » ;
- `75-achat-confirmation-oui.png` : touche G (30 + 100 = 130 gouttes), « ACHETER POUR 60 ? », OUI choisi ;
- `75-achat-fait.png` : 70 gouttes, factice ACHETÉ, Pépinette ÉQUIPÉ ;
- `75-titre-pepinette-recharge.png` : page rechargée, Pépinette court sur le titre ;
- `75-jeu-pepinette.png` : JOUER, Pépinette en jeu (et dans l'icône des vies) ;
- `75-boutique-640x360.png` : la boutique en 640 x 360 (x2, téléphone en paysage), lisible.

localStorage relevé par le pilote : après l'équipement `skin_choisi=pepinette` ; après le refus, rien ne
change (30) ; après la touche G `gouttes_en_poche=130` ; après l'achat `gouttes_en_poche=70`,
`skins_debloques=pepin,pepinette,factice_qa` ; après
rechargement, tout est gardé. 0 erreur de console. `docs/mesure-capture.py` : 0 pixel hors palette,
0 semi-transparent, 0 bloc flou sur les huit captures. Regardées.

### Ce qui n'est PAS testé

- Vrai téléphone (toucher réel sur onglets et boutons, Safari) : seul le clic est couvert, par la logique
  des zones ; la capture 640 x 360 est une fenêtre de bureau.
- Un seul onglet aujourd'hui : le changement d'onglet (gauche / droite, clic) n'est exercé qu'à vide ;
  la liste qui défile (plus de 3 objets, flèches ^ / v) n'apparaît pas encore avec le catalogue actuel.
- Personne n'a joué à la main ; son des menus à l'oreille non vérifié.

## Carte 7.5b : goutte d'or au lieu du mot « GOUTTES »

- Titre : la goutte d'or (sprite `goutte0`) suivie du nombre, taille 1 comme avant ; `LIBELLE_POCHE` supprimé ; « RECORD : score » inchangé. Boutique : prix = goutte + nombre, « GRATUIT » reste, le refus garde « IL TE MANQUE N GOUTTES ».
- `./docker-cargo.sh test -p game --no-default-features ecrans` : 19 ok ; `... contenu` : 9 ok ; `./docker-cargo.sh lint` : FMT-OK, LINT-OK.
- `./build.sh` : `dist/` produit, 0 `/home`, seule URL `http://www.w3.org/2000/svg` ; `./build.sh --qa` + `node docs/capture-boutique.mjs` (Chromium 1208 sans tête, CDP), 0 erreur de console.
- Captures `docs/qa/75b-titre.png` (goutte + 30) et `docs/qa/75b-boutique.png` (GRATUIT x2, goutte + 60) : regardées, `mesure-capture.py` 0 pixel hors palette, 0 flou.
- Pas testé : vrai téléphone ; personne n'a regardé à la main hors captures.

## Sauvegarde v2, défis, médailles et bonus de fin (carte 7.3)

Données et règles seulement (l'écran de carte et la boutique liront la sauvegarde : cartes 7.4, 7.5).

- Sauvegarde `version=2` (`game/src/sauvegarde.rs`) : en plus de la v1, `niveau_max_atteint`, une
  ligne par niveau touché (`niveau.N=gouttes:101 defis:110 medaille:2 temps:2345` : lesquelles des 3
  gouttes trouvées dans un niveau réussi, défis déjà réussis, meilleure médaille, meilleur temps en
  pas), un inventaire générique `inventaire=skin:pepin,skin:pepinette,musique:orage,...` et l'objet
  équipé par catégorie `equipe=musique:orage,skin:pepin`. Une v1 est relue sans perte puis réécrite en
  v2 ; `skins_debloques`/`skin_choisi`/`debloquer`/`depenser` restent (façades de la catégorie `skin`,
  accordées par `normaliser`) ; `possede`, `acquerir`, `equiper`, `equipe` pour la boutique. Lecture
  toujours tolérante (champ inconnu ignoré, morceau illisible → défaut).
- Défis (`game/src/defis.rs`), sans rien changer au jeu : SANS MOURIR (0 mort dans la tentative),
  SANS PLANER (parapluie jamais ouvert) ou LES 3 GOUTTES, CHRONO (`duree_cible_s`). Une tentative
  repart à chaque (re)chargement du niveau. SANS PLANER n'est proposé que là où le robot de
  faisabilité, parapluie interdit, finit le niveau : table `levels::SANS_PLANER_POSSIBLE`, recopiée
  du test ignoré `sans_planer_possible_d_apres_le_robot` (possible : 2, 3, 4, 6, 7, 9 (boss), 11
  (épilogue) ; LES 3 GOUTTES : 1, 5, 8, 10 (bonus)). Le boss, le bonus et l'épilogue ont leurs 3
  défis tels quels (dans le bonus on ne perd pas de vie, mais une chute compte comme une mort).
- Médaille = défis réussis dans une même tentative (1 bronze, 2 argent, 3 or), la meilleure gardée.
- Revenus (constantes `GAIN_*` en tête de `defis.rs`) : défi réussi la 1re fois +3, or la 1re fois +5,
  gouttes ramassées comme en 7.2 ; bonus de fin à chaque partie finie : FIN +5, SANS GAME OVER +5,
  100% GOUTTES +10 (gouttes du parcours, bonus non compris).
- Affichage : sous la carte du niveau suivant, « NIVEAU RÉUSSI : ARGENT, +7 GOUTTES » et les 3 défis
  cochés ou non ; sous l'écran de fin, une bande qui alterne toutes les 3 s entre le bonus de fin
  (gagné en or pâle, raté en gris) et les défis du dernier niveau. Score et physique inchangés.

### Preuves (extraction propre `git archive carte-7.3`)

- Robot de QA : `ecrire_script_navigateur` redonne `71df3e47c4844537` (21 422 images, 0 mort,
  14 gouttes, score 2500), la même empreinte qu'en 7.2 : les écrans de fin ajoutent un dessin, pas
  d'image ni de pas.
- `test -p game --no-default-features sauvegarde defis bot_qa`, `test -p core faisabilite`, lint,
  `./build.sh` : voir le résultat de la carte. Un seul test ajouté (`defis`) et le test de
  `sauvegarde` étendu (aller-retour v2, migration d'une v1).
- Vrai navigateur (Chromium sans tête, CDP, build de QA servi sur 127.0.0.1, `docs/capture-defis.mjs`)
  : sauvegarde vide, niveau 1 joué par le robot (34 s, 0 mort, 1 goutte) → carte du niveau 2 :
  SANS MOURIR et CHRONO 0:50 cochés, LES 3 GOUTTES non, ARGENT, +7 gouttes, `gouttes_en_poche=7`,
  `niveau.1=gouttes:010 defis:101 medaille:2 temps:2036`. Partie entière : 92 gouttes (14 ramassées
  + défis + une médaille d'or au niveau 8 + bonus FIN et SANS GAME OVER), page rechargée → tout est
  conservé ; 0 erreur console. Captures `docs/qa/73-fin-de-niveau-defis.png`,
  `73-fin-de-partie-bonus.png`, `73-fin-de-partie-defis.png` (regardées : rien ne se chevauche ;
  `mesure-capture.py` : 0 pixel hors palette, 0 flou).

### Ce qui n'est PAS testé

- Vrai téléphone (texte à l'échelle 1, comme les autres lignes des écrans) ; Safari, Firefox.
- Pas de boutique branchée (le coordinateur la réconcilie à la fusion avec 7.5) ; pas de suite
  complète (tests ciblés seulement, consigne de la carte).

## Secrets : faux murs, salles cachées, raccourcis, sortie secrète (carte 8.5)

Demande d'AGA : « Des secrets. Sorties cachées, salles derrière un mur, raccourcis. »

### Mécanique

- Lettre de grille `m` : FAUX MUR. Case libre (aucune collision), dessinée comme le solide qu'elle
  prolonge (brique, toit ou nuage). Les faux murs d'un seul tenant forment une zone cachée
  (`Niveau::caches`, `core/src/level/secrets.rs`) ; une poche de cases libres fermée de tous côtés
  par des solides et des faux murs (une salle, sa goutte, sa sortie) est cachée avec eux : les
  objets qui y sont (gouttes) ne se dessinent pas tant que la zone est cachée.
- Pépin touche la zone : elle se révèle pour de bon (`Monde::caches_reveles`) : les faux murs sont
  dessinés tramés en damier (un pixel sur deux, aucune demi-teinte) et le son `Effet::Secret`
  (petite gamme montante) joue une fois.
- INDICE, sur la case d'entrée côté chemin : une brique au joint décalé (`brique_decalee`) dont le
  bout dépasse de 3 px de la façade (`brique_saillie_g` / `_d`) ; dans un nuage, un petit creux
  d'ombre (`nuage_creux`). Les variantes de brique tirées au hasard restent dans leur case : seul
  l'indice sort du mur.
- Lettre `x` : SORTIE SECRÈTE, un second drapeau (même boîte que `X`, une au plus) ; la toucher
  finit le niveau comme une fin normale (`Monde::sortie_secrete` la distingue).
- Un seul nouveau test : `level::tests::faux_murs_zone_cachee_et_sortie_secrete`.

### Contenu (six niveaux, ni le boss, ni le bonus, ni « Au-dessus des nuages », ni la descente)

| Niveau | Secret | Goutte déplacée |
|---|---|---|
| Les toits | salle sous le toit : on entre dans la cheminée gauche du creux et on tombe dans la salle | celle du haut de l'immeuble |
| Gouttières et cheminées | salle derrière la façade gauche du grand immeuble (on y saute depuis le champignon) | celle au-dessus de l'immeuble |
| Le marché couvert | salle dans le premier bloc de boutique (marcher droit dedans) | celle du rebond du 1er auvent |
| Le grand orage | RACCOURCI : tunnel au pied du bloc de l'ascenseur (on évite la montée) | celle des nuages friables, au milieu du tunnel |
| Le clocher | SORTIE CACHÉE : salle dans le mur gauche, 2e corniche du zigzag, drapeau secret (on évite la cage de l'horloge) |, |
| La mer de nuages | RACCOURCI : tunnel au pied de la tour de nuages (on évite la montgolfière et le haut du plané) |, |

Quatre salles (toits, gouttières, marché, clocher), deux raccourcis (orage, mer), une sortie
cachée (clocher), une goutte déplacée sur quatre niveaux. Le chemin normal ne change pas : aucun
faux mur sur une patrouille de flaque, ni sur le passage obligé.

### Robot de QA

- Partie NORMALE : `trouver_parcours` n'admet plus une arrivée après un secret révélé (et la case
  `CaseParcours` retient « secret révélé », pour qu'un raccourci ne remplace pas le chemin
  normal) : il suit le chemin normal, jusqu'au drapeau.
- Partie COMPLÈTE : elle prend toujours les trois gouttes de chaque niveau, donc entre dans les
  salles ; la sortie secrète ne compte jamais comme arrivée du robot.
- `trouver_parcours_secret` (raccourci / sortie secrète) et le test ignoré
  `ecrire_scripts_secrets_85` : scripts et traces des captures (`target/qa/*-85*`).

### Preuves

- Tests ciblés : `test -p core` complet vert (375, dont zones, faisabilité, niveaux touchés et le
  nouveau test) ; `test -p game --no-default-features bot_qa::tests` : 4 verts (les deux parties,
  0 mort ; arc-en-ciel ; clocher et ses lanternes) ; `sprites` : 54 verts (planche des sprites
  régénérée : 4 tuiles nouvelles).
- Captures `docs/qa/85-*` (Chromium sans écran, CDP, build de QA servi en local, x3) : pour chaque
  niveau, l'indice vu du chemin (`-1-indice`), puis l'entrée et la salle révélée / le tunnel / la
  sortie (`-2-`, `-3-`). Rejeu des parcours du robot (partie complète pour les salles, parcours
  secret pour orage, clocher et mer), 0 mort. `docs/mesure-capture.py` : 0 pixel hors palette,
  0 bloc flou sur les 18.
- Avis sur les indices (captures regardées une à une) : le bout de brique qui dépasse se voit
  quand on regarde la façade, sans sauter aux yeux (3 px sur un mur de briques) : trouvable sans
  être évident aux toits, gouttières, marché et orage. Au clocher, le ciel violet a la couleur des
  briques : seuls le liseré clair et le joint du bout de brique se voient (plus discret). Dans la
  mer de nuages, le creux d'ombre est petit mais net sur le nuage uni.
- Une révélation en jeu : `-2-` / `-3-` (faux mur tramé, goutte visible derrière, drapeau secret du
  clocher).
- Extraction propre (`git archive` de 81c6a2a) : `test --workspace --no-default-features` : core
  375 verts, game 339 verts ; `./docker-cargo.sh lint` : LINT-OK (sur 31196bf, même code) ;
  `./build.sh` : `dist/` sans nom interdit, seule URL `http://www.w3.org/2000/svg`, game.wasm
  523 670 octets (gzip 211 121).

### Ce qui n'est PAS testé

- Le son « secret » à l'oreille (Chromium sans écran) ;
- un humain qui cherche les secrets sans les connaître (à faire juger par AGA) ;
- téléphone, Safari.

## Carte 7.8 : icône du haut-parleur à la place de « SON : OUI / NON »

- Menus du titre et de la pause : la ligne du son montre un haut-parleur de 11 x 7 pixels (hauteur de la police, à ECHELLE_MENU, mêmes couleurs/ombre que le texte, pixels nets) : 2 ondes si actif, une croix si coupé. `libelle_son` reste le nom interne (tests) ; comportement inchangé.
- Tests `ecrans` (19) et `contenu` (9) verts sans retouche, `lint` LINT-OK, `./build.sh` (dist/) et `./build.sh --qa` faits.
- Chromium 1208 sans tête, CDP (`docs/capture-son.mjs`), 0 erreur de console : `docs/qa/78-titre-son-coupe.png`, `78-titre-son-actif.png` (après Entrée), `78-pause-son-actif.png` (ligne au repos) ; regardées, `mesure-capture.py` : 0 pixel hors palette, 0 flou.
- Pas testé : vrai téléphone (clic/toucher sur l'icône : même rectangle de ligne qu'avant), Safari, Firefox.

## Carte 7.8b : icône du son en bas à gauche (clic, toucher ou M) au lieu d'une ligne de menu

- Plus de ligne SON (`Choix::Son`, `libelle_son` retirés) : titre JOUER / BOUTIQUE / COMMENT JOUER (y0 74), pause REPRENDRE / RECOMMENCER / QUITTER AU TITRE (panneau raccourci à 124 px). Haut-parleur 11 x 7 à l'échelle 2 en (6, 160), ombre, titre + pause seulement (en jeu, le coin est aux flèches tactiles).
- Bascule : clic/toucher dans une zone 34 x 26 (icône + 6 px de chaque côté, `sur_icone_son`) ou M (aide : « Son, ou l'icône en bas à gauche ») ; son coupé au départ, sans ligne validée.
- Tests existants adaptés (lignes comptées, `l_icone_du_son_bascule_sans_quitter_le_menu`) : `ecrans` 19, `contenu` 9, `app` 53 verts ; `lint` LINT-OK ; `./build.sh` et `--qa` faits.
- CDP (`docs/capture-son.mjs`, Chromium 1208), 0 erreur de console : `docs/qa/78b-titre-son-actif.png` (M), `78b-titre-son-coupe-apres-clic.png`, `78b-pause-son-actif.png` (clic en pause) ; regardées, 0 pixel hors palette, 0 flou.
- Pas testé : vrai téléphone (taille de la zone au doigt), Safari, Firefox.

## Carte des niveaux : des toits jusqu'au soleil (carte 7.4)

- Titre : nouvelle ligne CARTE (sous JOUER ; cinq lignes, titre et sous-titre remontés). La carte
  est VERTICALE : les 9 niveaux normaux montent des toits (en bas, sous la pluie) jusqu'au soleil
  (en haut), le bonus est une branche secrète à gauche du sommet, l'épilogue (« É ») redescend de
  l'autre côté jusqu'aux toits. Tout se déduit de `NIVEAUX` (genres) : aucun numéro en dur.
- Étape : case avec son numéro, cadenas si verrouillée, « ? » pour le bonus caché, bordure à la
  couleur de la médaille, 3 petites gouttes dessous ; Pépin (skin actif) debout sur l'étape choisie.
  Panneau de droite : nom, meilleur temps, gouttes trouvées, médaille, les 3 défis cochés, JOUER.
- Déblocage (`plan::etat_etape`, d'après la sauvegarde v2) : le 1er toujours ; un niveau atteint
  (`niveau_max_atteint`, noté désormais dès qu'une partie « JOUER » arrive à un niveau) ou dont le
  précédent est fini ; épilogue une fois le 9e fini ; bonus révélé avec toutes les gouttes des
  niveaux normaux TROUVÉES au cumul (27), sinon caché (« ? » et le compte 16/27).
- Commandes : Haut/Bas (ou Droite/Gauche) parcourent les étapes, Entrée ou JOUER lance une étape
  jouable, toucher une étape la choisit, molette et glisser du doigt font défiler, Échap ou RETOUR.
- Niveau joué depuis la carte = partie d'UN niveau (5 vies, « plus de vies » = on recommence ce
  niveau, défis/médailles/gouttes comptés comme en 7.3). Au drapeau : bilan (temps, « -1,8 s !
  NOUVEAU MEILLEUR TEMPS » ou « +3,2 s », défis, médaille, gain), REJOUER ou CARTE. Pause : la
  dernière ligne devient CARTE. Les records de partie (score, temps) ne bougent pas. Pendant ce
  niveau, le HUD compte les gouttes du niveau (x/3) et le chrono montre la tentative et le temps
  cible du défi CHRONO (« 0:05/0:55 »), en or tant qu'on est dans les temps.
- La progression n'est lue par les écrans que via `Sauvegarde::progres` / `progres_mut` (un seul
  endroit à changer pour la sauvegarde par clé de niveau, carte 7.9). Aucun crédit ajouté sur
  REJOUER : les gains restent ceux de `defis::noter_niveau` (7.3).
- Build de QA : touche U = progression avancée (1er appui : 7 niveaux finis, bonus caché ; 2e
  appui : tout fini, bonus révélé), enregistrée ; elle n'ajoute qu'à la progression existante.

### Preuves

- Retouche après coup (chrono cible, gouttes x/3, accès `progres`) : tests `app::`, `hud`,
  `sauvegarde`, lint, `./build.sh` et le robot (`--exact le_robot_finit_tous_les_niveaux_sans_mourir`)
  relancés sur l'extraction propre de 8789c40 : 100 tests ciblés verts, robot vert (0 mort),
  LINT-OK, `./build.sh` OK, `dist/` : 0 mention personnelle, seule URL `http://www.w3.org/2000/svg`.
- Extraction propre (`git archive`) : suite complète verte sur 709838a (core 374 ok, game 341 ok,
  partie complète du robot : 25 656 images, empreinte `0d1d8f8435a11586`) ; `lint` → LINT-OK et
  `./build.sh` OK sur dac5197 (seul écart : une conversion inutile retirée pour clippy) ; `dist/` :
  0 mention personnelle, seule URL `http://www.w3.org/2000/svg`.
- Tests ciblés : `app::` (50, dont les menus du titre/bilan adaptés), `plan`, `ecrans`, `contenu`.
  Aucun test ajouté (consigne AGA : pas de nouvelle mécanique de core). Robot de QA : identique (la
  ligne JOUER reste la première), `ecrire_script_navigateur` → `target/qa/script.txt` 21 422 images.
- Vrai navigateur (Chromium sans tête, CDP, build de QA sur 127.0.0.1, `docs/capture-plan.mjs`) :
  sauvegarde vide → seul le niveau 1 ouvert, Entrée sur le 2 sans effet ; niveau 1 lancé depuis la
  carte et fini (niveau 1 du robot rejoué) → bilan « PREMIER TEMPS SUR CE NIVEAU ! », 0:33,9,
  ARGENT, +7 gouttes, `parties_finies=0` (pas de record de partie) ; CARTE → niveau 2 ouvert ; page
  rechargée → conservé ; U puis U → bonus caché puis révélé ; 640 x 360 au toucher : étape touchée
  puis JOUER → carte de présentation du niveau 7, puis le niveau : chrono « 0:05/0:55 » en or,
  gouttes « 0/3 ». 0 erreur console.
- Captures `docs/qa/74-*.png` (15), regardées : carte en bas (toits), milieu, haut (soleil +
  bonus caché, puis débloqué), descente, bilan, titre, téléphone ; rien ne se chevauche ;
  `mesure-capture.py` : 0 pixel hors palette, 0 flou (sauf `74-telephone-chrono-cible` : les
  boutons tactiles translucides, dessinés par-dessus l'image logique, comme avant).

### Ce qui n'est PAS testé

- Vrai téléphone (glisser du doigt seulement émulé par CDP pour le toucher simple, pas le glisser) ;
  Safari, Firefox ; molette d'un vrai pavé tactile (défilement par crans).
- Sur téléphone, le bouton de pause cache la fin du chrono (défaut antérieur, déjà noté à la carte
  6.2) : avec le temps cible, c'est « /0:55 » qui passe dessous (640 x 360).

## Carte 8.7 (8.12) : une musique par zone, et deux musiques nouvelles (Cyclone, Poursuite)

- `musiques::musique_du_niveau` choisit la boucle d'après le THÈME (repli : Orage), jamais le numéro. Ville (toits, gouttières, marché) → Toits de nuit ; orage et clocher → Orage ; cyclone (boss) → Cyclone ; vents, mer, au-dessus des nuages → Valse des nuages ; arc-en-ciel et descente → Soleil. Écrans (`musique_de_l_ecran`) : titre, aide et boutique → Orage, fin → Soleil ; la carte de présentation, la pause et « plus de vies » jouent celle du niveau (sinon une coupure entre deux niveaux d'une même zone).
- Nouvelles partitions : Cyclone (mi mineur, motif haché, 9,6 s) et Poursuite (le motif du Cyclone sans silences puis à l'octave, galop, 11,5 s), au volume des autres (efficace 0,183 / 0,182 contre 0,180 à 0,185). Le niveau final de la carte 8.2 n'existe pas encore ici : la ligne `match` à ajouter est en commentaire (s'il garde le thème Cyclone, il faudra un autre critère que le thème). Catalogue et prix inchangés ; F11 (build de QA seulement) FORCE une musique par-dessus la zone.
- Tests : `test -p game --no-default-features musique` : 3 verts (dont le nouveau `chaque_niveau_a_la_musique_de_sa_zone`) ; `lint` LINT-OK ; `./build.sh` et `--qa` faits. Extraits `docs/qa/87-musique-cyclone.wav`, `87-musique-poursuite.wav` (les `77-*` régénérés sont identiques à l'octet).
- Chromium 1208 sans écran, CDP (`docs/capture-musique.mjs`, build de QA) : un crochet note les boucles audio lancées/arrêtées. Titre Orage ; niveaux 1 → 2 → 3 : même boucle, jamais relancée ; 3 → 4 : Toits de nuit → Orage ; 5 → 6 : Orage → Cyclone ; pause : coupée, reprise : Cyclone. 0 erreur console (`docs/qa/87-musique-journal.json`, captures `87-niveau4-orage.png`, `87-niveau6-cyclone.png` : 0 flou).
- Pas testé : écoute humaine des deux musiques (Chromium sans écran), téléphone, Safari, Firefox.

## Sauvegarde v3 par clé de niveau, chaque récompense créditée une fois (carte 7.9)

- Clé stable `InfosNiveau::cle` (nom du module : `toits`… `arc`, `descente`), unique et vérifiée à la compilation ; sauvegarde v3 `niveau.<cle>=atteint:1 gouttes:… defis:… medaille:… temps:…` (plus aucun indice) ; v1/v2 relues par la table FIGÉE `ORDRE_V2` (11 clés), porte-monnaie gardé, progression v2 = déjà créditée, « FIN » crédité si une partie a été finie. À la fusion de 8.2 : ajouter `cle: "poursuite"` (le compilateur l'exige).
- Règle : goutte (clé + numéro), défi, amélioration de médaille (différence de valeur ; seul l'or vaut 5) et chaque bonus de fin (`bonus_credites`) ne rapportent qu'UNE fois. Bilan et bande : gain réel (+0 possible), « N GOUTTES DÉJÀ COMPTÉES », bonus déjà eu « +0 » en brume.
- Tests (2 ajoutés) : `la_migration_v2_range_chaque_niveau_sous_sa_cle` (niveau.10/11 → `arc`/`descente`, juste avec `poursuite` inséré avant), `rejouer_un_niveau_ne_recredite_pas` ; ciblés `sauvegarde`, `defis`, `boutique`, `carte`, `plan`, `rejouer`, `bilan`, core `levels`/`zones` verts ; `lint` LINT-OK ; `./build.sh` et `--qa` OK, `dist/` sans `/home`.
- Navigateur (build de PRODUCTION, Chromium 1208 CDP, `node docs/capture-migration.mjs PORT docs/qa target/qa/script.txt 20100 21160 215`) : v2 réaliste relue → carte : descente MEILLEUR 0:18,3 OR, arc-en-ciel 0:55,0 ARGENT (`79-carte-*.png`) ; descente rejouée (robot) → « ARGENT, +0 GOUTTE », -0,9 s nouveau meilleur temps, « 1 GOUTTE DÉJÀ COMPTÉE », poche 57 → 57, sauvegarde réécrite en `version=3` par clés ; 0 erreur console ; 6 captures, 0 pixel hors palette.
- Économie (maximum total, une fois) : par niveau 3 gouttes + 3 défis × 3 + or 5 = 17 ; 11 niveaux = 187 + bonus de fin 20 = 207 ; avec la poursuite (8.2) 12 × 17 + 20 = 224. Prix : musiques payantes 5 × 25 = 125 (main), skins payants : aucun aujourd'hui (factice QA 60). 125 + un skin à 60 = 185 ≤ 207 : boutique accessible. Proposition (non appliquée) : au-delà d'un skin payant, skins à 40 ou musiques à 20, pour que tout reste achetable (224 ≥ 5 × 20 + 3 × 40 = 220).
- Pas testé : un joueur v1 n'avait pas de progression : ses gouttes de niveau d'avant la carte 7.3 seront créditées une fois de plus (une seule fois).

## Le mur de pluie : un vrai final (carte 8.2)

Demande d'AGA : un niveau de poursuite, un mur de pluie qui avance et fait défiler l'écran.

- Mécanique (`core/src/poursuite.rs`, lettre de grille `P`, une au plus) : le front du mur avance à
  `VITESSE_MUR` = 64 px/s, 2/3 de la course de Pépin (96 px/s ; le build échoue au-delà de 80 %). Il
  n'est jamais plus de `RETARD_MAX` = 128 px derrière Pépin (rattrapage : 2 s d'avance au plus) et tue
  au contact (`CauseMort::Pluie`). Après une mort, Pépin réapparaît à sa lanterne et le front est
  replacé à `RECUL_REAPPARITION` = 112 px derrière lui : toujours 1,75 s pour repartir, jamais de mort
  en boucle. À 24 tuiles du drapeau (`PERCEE_AVANT_ARRIVEE`), le soleil perce : le mur ne tue plus et
  recule, la caméra redevient libre. Un seul test (toutes ces règles).
- Défilement : tant que le soleil n'a pas percé, la caméra est accrochée au front, à 48 px du bord
  gauche (`Camera::defiler_avec_le_mur`) ; Pépin, entre le front et 128 px devant, est toujours à
  l'écran et voit 9 tuiles devant lui au moins.
- Dessin (`game/src/mur_pluie.rs`) : rideau d'ardoise très sombre au bord dentelé qui ondule, pluie
  battante dedans (traits ardoise claire et cyan sombre), frange de gouttes cyan devant, un éclair
  blanc toutes les 1,6 s ; le soleil qui perce grandit en haut à droite, rayons pointillés qui
  s'allongent, et la pluie du thème s'arrête.
- Niveau 10 « Le mur de pluie » (renommé « Le retour du Grand Nuage » en 8.2b ;
  `core/src/levels/poursuite.rs`, 200 x 11 tuiles, thème Altitude),
  inséré après « Au-dessus des nuages » : pont friable, vent arrière, courant ascendant devant une
  falaise de nuages, deux colonnes de grêle, second pont friable sous un corbeau, long toit final au
  soleil. 3 gouttes (colonnes 19, 98, 160), 3 lanternes (54, 102, 134). Le bonus devient le 11e, la
  descente le 12e ; le bonus demande 30 gouttes (36 dans le jeu, 33 au parcours). Tests qui nommaient
  les niveaux par numéro : un commit séparé.
- Robot QA : il se repose 3 pas au lieu de 20 entre deux plans dans un niveau de poursuite (il ne
  peut pas s'arrêter 1/3 s à chaque atterrissage). Il finit le niveau 10 à 0 mort en temps réel :
  1 902 images (31,7 s), le front jamais à moins de 83 px de lui.

### Preuves

- `test -p core poursuite`, `levels`, `zones`, `faisabilite` : verts (niveau 10 faisable, reprise
  depuis chaque lanterne, décor, manches à air, premier défi à l'écran) ;
- `test -p game --no-default-features le_robot_finit_tous_les_niveaux_sans_mourir` : vert ; partie
  du robot (`ecrire_script_navigateur`) : 23 528 images, 0 mort ;
- extraction propre (`git archive HEAD`) : `test --workspace --no-default-features` core 376 verts,
  game 339 verts (dont la partie complète avec le bonus, 0 mort) ; `./docker-cargo.sh lint` FMT-OK,
  LINT-OK ; `./build.sh` : `dist/` sans nom interdit (0), seule URL `http://www.w3.org/2000/svg`,
  game.wasm 517 083 octets (gzip 206 893) ;
- captures (build de QA `./build.sh --qa` servi sur 127.0.0.1, Chromium 1208 sans tête par CDP,
  `docs/capture-visibilite.mjs`, 960 x 540, x3), parcours du robot : `docs/qa/82-1-depart.png`
  (image 20 130) ; `82-2-talonne.png` et `82-2-reapparition.png` (à l'image 20 300 le robot lâche
  tout : le rideau rattrape Pépin immobile à l'image 20 420 ; 30 images plus tard il est reparti du
  départ, 6 vies, le front ~90 px derrière lui) ; `82-3-eclair.png` (éclair dans le rideau) ;
  `82-4-courant.png` ; `82-5-soleil-perce.png` (le moment fort, 48 images après la percée) ;
  `82-6-arrivee.png` (le drapeau sous le soleil). `docs/mesure-capture.py` : 0 pixel hors palette,
  0 bloc flou sur chacune ; toutes regardées. Deux retouches faites après les avoir regardées : le
  soleil perçait trop tard (hors de l'écran, derrière l'arrivée), et le rideau sortait de l'écran
  quand Pépin courait devant (caméra seulement poussée par le front) : elle est maintenant accrochée
  au front.

### Ce qui n'est PAS testé

- Personne n'a joué le niveau à la main : la pression réelle (vitesse du mur, 2/3 de la course) et
  la difficulté sont à juger par AGA en jouant (pas de mesure de difficulté, consigne) ;
  `difficulte.rs` a seulement reçu une bande Habitué provisoire pour le niveau 10.
- Faisabilité du cœur : le robot de `core/src/faisabilite.rs` ignore le mur (comme les ennemis) ;
  c'est le robot QA du jeu, dans la vraie simulation, qui prouve qu'on finit devant lui.
- Téléphone, Safari, son (aucun son propre au mur : la pluie et l'orage du thème suffisent).

## Le retour du Grand Nuage (carte 8.2b)

- Cohérence (AGA) : le Grand Nuage s'enfuit au niveau 6 et revient au 10, renommé « Le retour du Grand
  Nuage ». Mécanique inchangée ; dessin (`game/src/mur_pluie.rs`) : front d'orage ardoise et violet en volutes,
  pluie et éclair DEDANS, visage du boss (`nuage_colere`, `nuage_essouffle` après la percée) sur le bord qui
  avance ; plus de pluie du thème : ciel clair devant Pépin. Textes : accueil, conseil, Vents d'altitude
  (« s'est enfui »), Au-dessus des nuages, README. Aucun nouveau test.
- Preuves : `test -p core` 376 verts ; `game` `contenu`, `carte`, `scene` verts ; lint FMT-OK, LINT-OK ;
  `./build.sh` 0 nom interdit ; captures CDP regardées (0 hors palette, 0 flou) `docs/qa/82b-1-carte.png` à
  `82b-5-soleil.png` (`82b-4-talonne` : Pépin immobile, le front tout près). Robot complet non relancé.

## Une petite histoire en trois images (carte 8.4)

- Trois images fixes dessinées en code (`game/src/ecrans/histoire.rs`, 320x180, palette du jeu, décor
  des thèmes, tuiles et sprites existants), deux lignes de texte et « APPUIE POUR CONTINUER » dans un
  bandeau en bas, numéro « 1/3 » en haut à droite. Textes dans `contenu::RECIT`.
  1. Avant le niveau 1 : les toits sous l'averse, maisons aux fenêtres éteintes, Pépin sur une cheminée
     qui regarde en haut (il cligne des yeux) ; « La pluie ne s'arrête plus : le soleil dort. / Pépin
     prend son parapluie et monte le réveiller. »
  2. Après le Grand Nuage (avant « Le ciel se déchire ») : la trouée d'or du cyclone, un rai de lumière,
     le Grand Nuage essoufflé qui s'éloigne, une goutte d'or qui descend vers Pépin ; « Le cyclone s'est
     tu. Plus haut, une lueur d'or. / Les gouttes d'or sont des éclats de soleil ! »
  3. Avant la descente (l'épilogue, donc après le sommet et après le bonus s'il est joué) : le soleil
     ouvre un œil au-dessus de la mer de nuages, des gouttes d'or remontent vers lui, Pépin parapluie
     fermé ; « Le soleil ouvre un œil. / Il est temps de redescendre lui montrer la ville. »
- Placement : règle pure `histoire::chapitre_avant(table, indice)` (1er niveau ; niveau qui suit le
  thème `Cyclone` ; niveau `Epilogue`) : le futur niveau de poursuite de la carte 8.2, inséré au
  sommet, passera de lui-même avant l'image 3.
- Montrées une fois par partie lancée par JOUER (ni REJOUER, ni « recommencer », ni « plus de vies », ni
  les raccourcis de niveau du build de QA). L'image ignore les appuis 0,5 s (un saut tenu ne la passe
  pas sans la voir), puis Entrée / Espace / Échap / toucher la passe ; ensuite la carte du niveau. Aucun
  effet sur le score, les gouttes, les vies ni le chrono.
- Titre : nouvelle ligne HISTOIRE (entre BOUTIQUE et COMMENT JOUER) qui montre les trois images à la
  suite puis revient au titre, curseur sur HISTOIRE ; Échap y revient tout de suite.
- Robots de QA (`bot_qa`, `jusqu_a` des tests) : ils appuient sur Entrée dès qu'une image l'accepte
  (`Application::histoire_en_attente`), comme un joueur.

### Preuves

- Tests ciblés : `ecrans::histoire` (5 : bandeau qui tient, ordre, bons niveaux dans la vraie table,
  règle sur de fausses tables, délai) et `app` (4 : JOUER -> image 1 -> carte du niveau 1, focus perdu
  sans pause ; une partie entière montre les 3 images une fois chacune, bonus joué ou non, « recommencer »
  ne les remontre pas ; REJOUER et raccourcis sans histoire ; ligne HISTOIRE du titre + Échap).
- Extraction propre `git archive` de la branche (commit 57d8ada) : `test -p core` 374 ok ;
  `test -p game --no-default-features` 349 ok (26 ignorés, dont la partie complète du robot qui passe
  les images) ; `lint` : LINT-OK ; `./build.sh` : game.wasm 522 965 octets (gzip 210 691), aucun `/home`,
  seule URL `http://www.w3.org/2000/svg`.
- main a avancé pendant la carte (7.3, 8.5, 7.8/7.8b) : `git merge-tree` signale 2 fichiers en conflit,
  `game/src/app.rs` (champs et tests ajoutés des deux côtés, `choix_pour` sans `son_disponible`) et
  `game/src/ecrans/mod.rs` (disposition du titre). Résolution essayée hors dépôt (arbre fusionné +
  conflits résolus) : avec l'icône du son de 7.8b le titre n'a plus que 4 lignes (JOUER, BOUTIQUE,
  HISTOIRE, COMMENT JOUER) : on GARDE alors le titre de main (sol 148, titre 14, sous-titre 56) et le
  menu de y = 66, pas 20 ; les changements de `titre.rs` de cette branche ne servent plus. Sur cet arbre :
  `test -p game` 350 ok, `test -p core` 375 ok, `lint` LINT-OK, `./build.sh` game.wasm 559 337 octets
  (main seul : 555 926, soit + 3 411), build de QA + captures : 0 erreur de console.
- Captures `docs/qa/84-*.png` (Chromium 1208 sans tête, CDP, `node docs/capture-histoire.mjs PORT
  DOSSIER` sur le build de QA) : `84-1-depart`, `84-1-puis-carte-niveau1`, `84-titre-ligne-histoire`,
  `84-2-lueur`, `84-2-lueur-plus-tard` (le nuage s'en va), `84-3-soleil`, `84-echap-titre` ; regardées,
  `mesure-capture.py` : 0 pixel hors palette, 0 flou ; un appui juste après l'ouverture de l'image 1 est
  bien ignoré.

### Ce qui n'est PAS testé

- Vrai téléphone (toucher pour passer l'image) ; personne n'a regardé à la main hors captures ; la
  lisibilité des textes à la taille 640x360.

## Carte 8.1 : démo du robot sur l'écran titre

- Après 10 s sans AUCUNE entrée sur le titre (touche, clic, souris qui bouge, molette, toucher, commande tenue), la démo (`Ecran::Demo`, `game/src/demo/`) rejoue un parcours EMBARQUÉ du robot, en rotation : Les toits, Le grand orage, Vents d'altitude, La mer de nuages (34 à 40 s chacun). Bandeau « DÉMO » clignotant en haut, « APPUIE SUR UNE TOUCHE » en bas (`contenu.rs`). Toute entrée ramène au titre tout de suite, sans autre effet ; drapeau, mort, entrées épuisées ou 45 s : retour au titre derrière les volets. Musique du niveau, aucun bruitage ; un `Monde` à part : rien n'est écrit dans la sauvegarde.
- Parcours : `demo/parcours.rs` GÉNÉRÉ par le test ignoré `ecrire_parcours_de_la_demo` (`trouver_parcours` du robot, compressé en plages `(pas, code)` : 268 plages en tout). À régénérer si l'un de ces niveaux change ; le test `chaque_parcours_de_la_demo_atteint_le_drapeau` (le seul ajouté) le signale. Un parcours périmé ne plante rien : la démo s'arrête et revient au titre.
- Tests : `test -p game --no-default-features --release` : 343 verts, 0 échec (dont `bot_qa` et le nouveau test `demo::`) ; `test -p core --release` vert ; `lint` LINT-OK. Wasm de `./build.sh` : 571 419 octets contre 568 706 avant (+2 713, gzip +1 617).
- Vrai navigateur (Chromium sans tête, CDP, build de QA sur 127.0.0.1, protocole `docs/pilote-qa.js`, horloge virtuelle), sauvegarde préparée à 42 gouttes : après une touche, 9 s → titre, la démo apparaît à l'image 600 exactement (10 s) ; touche A → titre à l'image suivante, curseur sur JOUER ; mouvement de souris et toucher → titre (le toucher n'a pas validé de ligne) ; deux démos laissées jusqu'au bout reviennent seules au titre (< 45 s) ; sauvegarde identique à l'octet du début à la fin ; aucune erreur du pilote, pas d'écran d'erreur. Captures `docs/qa/81-demo.png` (Vents d'altitude, bandeau allumé), `81-retour-titre.png` (porte-monnaie 42), regardées, `mesure-capture.py` : 0 flou.
- Pas testé : vrai téléphone, Safari, Firefox.

## Intégration de l'histoire (carte 8.11)

Après la fusion de 7.9, 8.2/8.2b, 8.4, 8.12 et 7.4 : finir l'intégration de l'histoire et du final.

- Thème `Theme::Poursuite` (dernier de l'enum : les indices des autres thèmes ne bougent pas) pour « Le
  retour du Grand Nuage » : le décor et le ciel d'altitude (`ambiance(Altitude)` repris tel quel), mais
  SANS pluie de thème (`Theme::pluie` faux, 0 goutte d'ambiance) : il ne pleut que dans le front d'orage.
  Luminosité 3 (celle de l'altitude). `musique_du_niveau` : `Theme::Poursuite => Musique::Poursuite` (le
  niveau jouait la valse).
- Verrou HISTOIRE : la ligne n'apparaît au titre que si `parties_finies > 0` (aucune ligne grisée ni
  cadenas). Avant, le titre a 4 lignes, centrées dans la place des 5 (`ecrans::disposition_menu` : y0 + 9
  px, même pas de 18 px) ; clic et survol utilisent la même disposition que le dessin.
- Textes (`contenu.rs`) : histoire 2 « Le Grand Nuage s'enfuit sous la mer de nuages. / Plus haut, une
  lueur d'or : des éclats de soleil ! » ; histoire 3 « Le Grand Nuage s'est vidé de sa pluie. / Le soleil
  ouvre un œil : montrons-lui la ville ! » ; accueils du niveau 1 (« en route vers le soleil »), du Grand
  Nuage (« C'est lui qui cache le soleil »), d'« Au-dessus des nuages » (l'orage gronde encore) et de la
  descente (« Le soleil est réveillé : on redescend lui montrer la ville. ») ; fin « DE RETOUR ! » + « Plus
  de pluie : Pépin replie son parapluie. ». Les `intro` de `core/src/levels/*.rs` suivent. Compteur de
  gouttes « n/33 » et « NIVEAU n/10 » déjà calculés depuis la table (test du HUD : la largeur vient de
  `gouttes_du_parcours`, plus de « 30/30 » en dur).
- Image 2 : le Grand Nuage s'enfuit vers la droite EN DESCENDANT (il passait au-dessus). Numéro « n/3 » :
  dans le bandeau du bas, à droite (il se lisait mal sur le ciel saumon de l'image 3).
- Image 3 : « … : montrons-lui la ville ! » (validé) ; « … : on redescend montrer la ville ! » fait 335 px
  pour 304 utiles et déborderait du bandeau. Commentaires « neuf »/« onze » de arc.rs et descente.rs et
  zone de l'ancienne ligne de son de `docs/qa-clavier-ecran.mjs` (marquée périmée) corrigés au tour 2.
- Site et doc : `web/index.html` (meta, og, texte sr-only : dix niveaux, bonus, épilogue, l'histoire, le
  final, secrets, carte, défis, boutique) ; `README.md` (dix niveaux + bonus + épilogue, le niveau 10,
  l'histoire, secrets, carte/défis/médailles/boutique, l'icône du son à la place de la ligne SON).

### Preuves

- `./docker-cargo.sh test -p game --no-default-features ecrans` 24 ok, `... contenu` 9 ok, `... histoire`
  7 ok (un filtre par commande) ; `test -p core` 377 ok (2 ignorés) ; tests adaptés : pluie par thème
  (12 valeurs), titre de 4 lignes avant une partie finie (`la_ligne_histoire_du_titre...`), clic et survol
  du titre (disposition à 4 lignes), largeur du compteur de gouttes du HUD, phrase de fin.
- Suite complète `test -p game --no-default-features` : 352 ok, 27 ignorés, 1 échec
  `sauvegarde::tests::la_migration_v2_range_chaque_niveau_sous_sa_cle`, DÉJÀ en échec sur main (021ba92,
  extraction `git archive` : même échec) : écrit par 7.9 avant 8.2, il insérait un faux `poursuite` avant
  l'arc-en-ciel alors que la vraie table l'a maintenant (le faux, derrière le vrai jamais fini, était
  verrouillé). Adapté : il vérifie le VRAI niveau `poursuite` (vide, ouvert ; arc-en-ciel et descente
  gardent leur progression) ; `... sauvegarde` 2 ok.
- `./docker-cargo.sh lint` : LINT-OK. `./build.sh` : game.wasm 588 088 octets (gzip 236 883) ; dans
  `dist/`, aucun `/home` ni nom propre, seule URL `http://www.w3.org/2000/svg`.
- Captures `docs/qa/86-*.png` (build de QA servi sur 127.0.0.1, Chromium 1208 sans tête par CDP,
  `node docs/capture-integration.mjs PORT docs/qa SCRIPT 20100 21160 215`, SCRIPT : le `script.txt` du robot
  de la carte 7.9, dont la descente est inchangée) : `86-titre-sans-histoire` (4 lignes),
  `86-titre-avec-histoire` (5 lignes, `parties_finies=1`), `86-histoire-1` à `-3`,
  `86-histoire-puis-titre`, `86-accueil-poursuite` (NIVEAU 10/10), `86-poursuite-en-jeu` (ciel clair,
  pluie seulement dans le front), `86-fin-de-retour` (descente rejouée par le robot : « DE RETOUR ! »,
  « Plus de pluie : Pépin replie son parapluie. », gouttes « 1/33 »). Regardées ; `mesure-capture.py` : 0
  pixel hors palette, 0 flou. Crochet audio : titre Orage (9,60 s), niveau 1 Toits de nuit (11,52 s),
  niveau 9 la valse (10,56 s), niveau 10 une autre boucle (11,52 s, empreinte différente de Toits de
  nuit) : Poursuite. 0 erreur de console.

### Ce qui n'est PAS testé

- Vrai téléphone ; personne n'a lu les textes à la main à 640x360 ; partie complète du robot (ignorée,
  ~10 min) non relancée pour cette retouche.

## Carte 8.13 : retouches de la sauvegarde v3 (niveaux migrés finis, niveau 10 ouvert, défis par clé)

- Migration v2 : un niveau avec une médaille ou un défi réussi est FINI (`ProgresNiveau::fini`) et atteint ; sans temps, la carte écrit « FINI ». Plan : un niveau normal s'ouvre aussi après une partie finie ou si un niveau plus loin est atteint ou fini (`plan::deja_depasse`) ; le bonus garde sa règle. Un test : `un_niveau_v2_medaille_sans_temps_est_fini`.
- « SANS PLANER » rangé par clé : champ `InfosNiveau::sans_planer` à la place de `levels::SANS_PLANER_POSSIBLE` ; robot relancé (`--ignored sans_planer_possible`), aucun écart, poursuite mesurée `false`.
- Laissé par rang, rien changé : `contenu::INTROS`, vérifiée à la main dans l'ordre de `NIVEAUX` (poursuite 10e, « Le Grand Nuage remonte de la mer ») ; `difficulte::BANDES_HABITUE`, outil de mesure `#[ignore]`, à recaler à la prochaine insertion.
- Tests ciblés : core `levels` 29 ok (1 ignoré), `zones` 55 ok ; game `sauvegarde` 3, `plan` 4, `defis` 1 ok. `./docker-cargo.sh lint` LINT-OK ; `./build.sh` (game.wasm 591 252 octets).
- Navigateur (build de production, Chromium 1208 CDP, `node docs/capture-813.mjs PORT docs/qa`) : v2 `parties_finies=1`, `niveau_max_atteint=10`, seul `niveau.11=gouttes:111 defis:111 medaille:3` → `813-carte-descente-finie` (FINI, OR, 3 gouttes, 3 défis cochés), `813-carte-poursuite-jouable` (NIVEAU 10/10, JOUER, pas de cadenas). Regardées ; 0 erreur console.

## Cosmétiques visuels et musiques dans la boutique (carte 7.6)

Demande d'AGA : « autre chose que des skins […] uniquement du cosmétique, pour que le score reste
comparable ». Rien dans `core/` ; le monde (physique, collisions, score) n'est jamais modifié.

- Boutique à 5 onglets : SKINS, TRAÎNÉES, PARAPLUIES, PALETTES, MUSIQUES (rangée centrée). Chaque
  onglet commence par un choix gratuit équipé au départ : AUCUNE, UNI, ORIGINALE, PAR ZONE. Inventaire
  `categorie:id` de la sauvegarde v3 (pas de nouvelle version de format) ; une sauvegarde sans
  `musique:zone` (d'avant la carte, où `musique:orage` était équipée d'office) revient à PAR ZONE.
- PRIX (constantes `PRIX_*` en tête de `game/src/cosmetiques.rs`) : traînées 3 × 15, motifs 3 × 15,
  palettes 3 × 20, musiques 5 × 15 (ORAGE gratuite) : TOTAL 225 gouttes (224 gagnables dans le jeu de
  base, plus les courses contre le robot).
- Traînées ÉTINCELLES (petites croix d'or), BULLES (anneaux cyan qui montent), PLUMES (traits blancs qui
  tombent) : brins à part des particules du jeu, un tous les 3 pas quand Pépin bouge (> 8 px/s), 40 au
  plus, dessinés AVANT les ennemis et les dangers.
- Motifs POIS, RAYURES, ÉTOILES : images de chaque skin recopiées au chargement avec le motif sur la
  seule toile du parapluie (ouvert et fermé) ; couleurs choisies par skin (Pépin : blanc / or pâle ;
  Pépinette : corail / violet).
- Palettes SÉPIA, NUIT FROIDE, ÉCRAN VERT : un shader à l'affichage remplace chacune des 16 couleurs.
  Chaque couleur garde la LUMINANCE de l'originale : tous les contrastes à 1 % près (un seul test
  paramétré, `cosmetiques::tests::chaque_palette_garde…`), donc tous les tests de contraste existants
  (Pépin, gouttes, dangers, plateformes, vent, sur chaque ciel) valent pour chaque palette. ÉCART À LA
  CARTE : ÉCRAN VERT a 16 verts et non 4 tons : avec 4 tons, briques et ciel tombaient à 1:1.
- Musique équipée : remplace celle de la zone en jeu (carte de présentation, jeu, pause, plus de vies,
  bilan) ; titre, menus, démo, histoire et fin gardent la leur (`musiques::musique_jouee`, pure).
- Build de QA seulement : T, Y, O font défiler traînée, motif, palette (« rien » compris), sans
  enregistrer ; inertes en production. Pas F12 : elle ouvre les outils du navigateur.

### Preuves

- Extraction propre (`git archive` de 09ece00 ; le commit suivant n'ajoute que 4 lignes d'assertions
  à `chaque_niveau_a_la_musique_de_sa_zone`, `... musiques` 2 ok) : `./docker-cargo.sh test -p game --no-default-features --
  sprites skins cosmetiques boutique sauvegarde musiques decor scene app:: ecrans entrees rendu
  particules hud` : 242 ok ; `./docker-cargo.sh lint` : LINT-OK ; `./build.sh` : game.wasm 607 750
  octets (gzip 245 931 ; 588 088 notés à la carte 8.11), aucun `/home`, seule URL
  `http://www.w3.org/2000/svg`.
- Robot, une passe (`-- --exact --include-ignored bot_qa::tests::le_robot_finit_tous_les_niveaux_sans_mourir
  bot_qa::tests::ecrire_script_navigateur`, extraction de 09ece00) : vert, 0 mort ; 23 690 images, 13
  gouttes, score 2400, empreinte `e02065b280103725`, IDENTIQUE à celle de main (b4233a3, même commande)
  avec les réglages par défaut.
- Captures `docs/qa/76-*.png` (39 ; Chromium 1208 sans tête, CDP, build de QA, `node
  docs/capture-cosmetiques.mjs PORT docs/qa DOSSIER_GRILLES`) : boutique (5 onglets, achat d'ÉTINCELLES
  100 → 85 puis de SÉPIA → 65, toute la boutique en sépia, retour à ORIGINALE, page rechargée : la traînée
  équipée suit Pépin en jeu) ; chaque traînée en course et en plané ; chaque motif fermé et ouvert sur
  Pépin et sur Pépinette ; chaque palette (et l'originale) aux toits, à l'orage et à la mer de nuages.
  Tampon WebGL lu contre la palette ACTIVE : 0 flou partout, 0 pixel hors palette sur 36 captures ; les 3
  autres (onglet PALETTES) n'ont hors palette que les cases d'aperçu des palettes, voulues (24 cases de
  5 × 8 px = 960 px ; 640 en sépia, ses 8 cases étant dans la palette). 0 erreur de console.
- Toutes regardées (recadrages agrandis pour les parapluies) : lisibles. Pépin, gouttes, plateformes,
  dangers et HUD se lisent dans les trois palettes aux trois niveaux. Retouches faites après les avoir
  regardées : étincelles (un point seul se perdait dans la pluie → petite croix), étoiles du parapluie
  de Pépin (l'or vif se voyait mal sur le corail → or pâle), rayures du parapluie fermé (paraissait
  blanc), onglets (débordaient à 5 : centrés, plus serrés).

### Ce qui n'est PAS testé

- Vrai téléphone, Safari, Firefox (shader WebGL 1 très simple) ; l'écoute d'une musique équipée en
  jeu (le choix est vérifié dans `chaque_niveau_a_la_musique_de_sa_zone`, pas de crochet audio) ; les
  palettes sur les 9 autres niveaux.

## Carte 8.14 : mode FACILE du Grand Nuage (2 vies) proposé après 10 défaites d'affilée

- Cœur : `PV_NUAGE_FACILE = 2` à côté de `PV_NUAGE` ; `GrandNuage::passer_en_facile` (réglé au chargement, sans état global), `motif()` compte les points perdus depuis `pv_max` : en facile, motifs 1 puis 2 seulement. Un test : `en_facile_deux_points_et_les_deux_premiers_motifs`. Rendu : le nuage facile est tramé en damier (plus pâle, 0 pixel hors palette) ; le HUD montre 2 points.
- Jeu (`game/src/facile.rs`, `ecrans/facile.rs`) : `DEFAITES_AVANT_FACILE = 10` ; une mort au boss en normal (pas en démo) ajoute 1 à `niveau.cyclone=... defaites:N` (sauvegarde v3, par clé, champ écrit seulement s'il n'est pas nul) ; une victoire le remet à 0. À 10, après une mort (sauf « plus de vies »), bannière « TROP DUR ? » OUI / NON (NON choisi d'abord, clavier, souris, tactile, 0,5 s d'attente) : OUI recommence le niveau en facile, NON reprend le combat (plus de bannière dans ce chargement). Sur la carte des niveaux, JOUER sur le boss repropose le choix tant qu'il n'est pas battu en normal. Course (8.7) et NG+ (8.8) n'existent pas sur cette branche : à brancher à la fusion.
- RÈGLE des récompenses (7.9, crédit unique) : vaincre en facile = niveau FINI (`facile:1`, la suite s'ouvre), gouttes nouvelles créditées ; ni défi, ni médaille, ni meilleur temps du niveau, ni record de partie (`facile::noter_partie` compte seulement la partie finie). Bilan « NIVEAU RÉUSSI : FACILE » + « Pas de médaille ni de record en facile. », carte « FACILE ». Battre le boss plus tard en normal donne médaille et défis comme d'habitude.
- Preuves : core `nuage` 21, `levels::cyclone` 1, `zones` 55, `faisabilite` 25 ok ; game `sauvegarde` 3, `plan` 4, `defis` 1, `hud` 6, `contenu` 9, `ecrans` 24, `nuage` 18 ok (un filtre par commande) ; `./docker-cargo.sh lint` LINT-OK ; `le_robot_finit_tous_les_niveaux_sans_mourir` ok ; script complet régénéré (`--ignored ecrire_script_navigateur`) : empreinte `e02065b280103725` IDENTIQUE à celle de la base 1dbb50d (mode normal inchangé), 0 mort ; `./build.sh` : game.wasm 594 600 octets, aucun `/home` dans `dist/`. Navigateur (build de QA, Chromium 1208 CDP, `node docs/capture-814.mjs PORT docs/qa SCRIPT 11831 12700 48`, SCRIPT : `script.txt` du robot) : sauvegarde à 9 défaites → mort → `defaites:10` + bannière ; OUI → nuage vaincu en 2 coups → `gouttes:111 defis:000 medaille:0 facile:1` (défaites à 0) ; vents jouable ; rechargé : relu tel quel, la carte repropose le choix. Captures `814-*` regardées, 0 pixel hors palette, 0 erreur console.
- PAS testé : vrai téléphone (tactile seulement par le code partagé avec la boutique), Safari.

## Course contre le robot (carte 8.7)

### Ce qui a été fait

- Mode COURSE : sur la carte des niveaux, un bouton COURSE à droite de JOUER (Gauche/Droite au
  clavier, ou clic ; grisé si l'étape n'est pas jouable ou n'a pas de fantôme) ouvre le choix de la
  force (`Ecran::ChoixForce`, `ecrans/course_force.rs`) : DÉBUTANT, HABITUÉ, EXPERT (les trois
  `PROFILS` de `difficulte.rs`), CARTE ; sous la force choisie, le meilleur résultat contre elle
  (« PAS ENCORE COURU » sinon) et, pour EXPERT, « 1re VICTOIRE : +5 GOUTTES » tant que la prime
  reste à gagner.
- Le fantôme (`game/src/course/mod.rs`) : son propre `Monde` (la même simulation core), avancé d'un
  pas à chaque pas de jeu, nourri par une partie EMBARQUÉE (`course/parties.rs`, généré). Aucun
  contact avec Pépin ni avec le monde du joueur (deux mondes séparés, aucun état global). Dessin
  (`scene::dessiner_fantome`) : les images de Pépin tramées en damier (`Banque::dessiner_trame`, un
  pixel sur deux, aucune demi-teinte), l'étiquette de la force au-dessus ; hors champ, une flèche
  blanc cassé sur ombre d'ardoise au bord de l'écran, de son côté (`course::fleche_hors_champ`). Il
  meurt et réapparaît à sa lanterne comme le joueur simulé (animation de mort non bornée au cadre).
  HUD : sous le chrono, « ROBOT DÉBUTANT », puis « ROBOT ARRIVÉ 0:41,6 ».
- Parties embarquées : le test ignoré `course::tests::ecrire_parties_des_fantomes` (un fil par
  niveau, 785 s) rejoue le joueur simulé de la mesure (`difficulte::partie_du_fantome` : le premier
  essai, graine OFFICIELLE, qui atteint le drapeau) et compresse ses entrées en plages, au format de
  la démo 8.1 (`demo::compresser`). 11 niveaux x 3 forces = 33 parties, 2 988 plages. Le marché
  couvert n'a PAS de fantôme : `Itineraires::tenter` (nouveau, le `calculer` sans panique) n'y
  trouve pas le drapeau (la mesure de difficulté, allégée depuis 6.5, n'y passe plus) ; son bouton
  COURSE est grisé. Le niveau de poursuite a ses trois fantômes.
- Divergence (un niveau changé depuis la génération) : le fantôme s'arrête proprement dès que son
  rejeu s'écarte de la partie prévue (plus d'entrées, drapeau au mauvais pas, plus de morts que
  prévu) : il disparaît, le HUD dit « LE ROBOT A ABANDONNÉ » et la course ne compte pas (aucun
  `unwrap` sur le chemin du jeu). Un niveau renommé perd simplement sa course.
- Fin (`Ecran::BilanCourse`, `ecrans/bilan_course.rs`) : par-dessus le niveau figé, ce que le
  niveau a rapporté (comme un niveau joué seul : médaille, défis, gouttes, règles inchangées), puis
  « GAGNÉ DE 2,5 s » / « PERDU DE 1,0 s » (égalité = perdu), « TOI 0:39,1   ROBOT EXPERT 0:41,6 »,
  le meilleur résultat et la prime ; REJOUER (même force) ou CARTE. Pas de record de partie.
- Sauvegarde : `courses` (par CLÉ de niveau, l'écart signé en pas contre chaque force, négatif =
  gagné), écrit `course.<clé>=0:-978 1:-659 2:-152`, ignoré par les versions d'avant (pas de
  changement de version). Prime : `defis::GAIN_COURSE_EXPERT` = 5, à côté des autres revenus,
  créditée UNE fois par niveau par `bonus_credites` (`course_expert_<clé>`).
- Textes dans `contenu.rs` (forces, COURSE, GAGNÉ/PERDU, ROBOT…), aide de la carte « Haut/Bas :
  niveau  Gauche/Droite : mode », README (un paragraphe).

### Preuves

- Test ciblé (le seul nouveau) : `course::tests::chaque_partie_des_fantomes_rejoue_sans_divergence`
  (chaque partie embarquée, rejouée par `Fantome` comme dans le jeu, touche le drapeau au pas prévu
  avec les morts prévues ; plages identiques à la recompression) : ok.
- Extraction propre (`git archive`) du commit du code : `./docker-cargo.sh test --workspace
  --no-default-features` : core 377 ok (2 ignorés), game 355 ok (29 ignorés), dont
  `bot_qa::tests::le_robot_finit_tous_les_niveaux_sans_mourir` et
  `sauvegarde::tests::la_migration_v2_range_chaque_niveau_sous_sa_cle` ; `./docker-cargo.sh lint` :
  LINT-OK (rejoué sur l'extraction du commit d'après, alias de type pour clippy) ;
  `./build.sh` (production) : dans `dist/`, 0 nom interdit, aucun `/home`, seule URL
  `http://www.w3.org/2000/svg` ; game.wasm 617 223 octets (gzip 246 329), contre 588 088 (gzip 236 883) mesurés à la
  carte 8.11 sur la même base : +29 135 octets, dans le budget de +40 Ko (parties.rs : 36 234 octets
  de source, environ 12 Ko de plages).
- Vrai navigateur (build de QA servi sur 127.0.0.1, Chromium 1208 sans tête par CDP, onglet au
  premier plan, horloge virtuelle `docs/pilote-qa.js`) : `node docs/capture-course.mjs PORT docs/qa 2
  "Gouttières et cheminées" ROBOT 2226 game/src/course/parties.rs 229`, ROBOT : le parcours du robot
  de QA sur le niveau 2 (`bot_qa::trouver_parcours(1)`, 2 346 pas, écrit par un brouillon hors
  dépôt). Sauvegarde vide, touche U (niveaux 1 à 8 ouverts), niveau 2, COURSE, puis pour chaque force
  : Pépin attend au pas 2 226 (posé, avant la dernière ligne droite) le temps de se faire battre,
  puis REJOUER sans attendre. Temps de Pépin = 2 346 pas exactement à chaque fois (même horloge que
  le fantôme). Résultats : PERDU DE 1,0 s contre les trois forces (DÉBUTANT 0:55,4, HABITUÉ 0:50,0,
  EXPERT 0:41,6), puis GAGNÉ DE 16,3 s, 10,9 s, 2,5 s ; « EXPERT BATTU : +5 GOUTTES ! » ;
  localStorage `course.gouttieres=0:-978 1:-659 2:-152`, `bonus_credites=course_expert_gouttieres`,
  poche 0 → 8 ; page rechargée : résultats conservés. 0 erreur de console, 0 erreur du pilote.
- Captures `docs/qa/87c-*.png` (regardées ; `mesure-capture.py` : 0 pixel hors palette, 0 flou) :
  `87c-carte-bouton-course` (bouton COURSE choisi), `87c-choix-force`, `87c-fantome-double` (le
  fantôme DÉBUTANT tramé, étiquette au-dessus, double Pépin qui attend), `87c-fantome-hors-champ`
  (flèche au bord gauche : le fantôme est derrière), `87c-perdu-force-0`, `87c-gagne-force-2`
  (prime), `87c-choix-force-expert-battu` (« MEILLEUR : GAGNÉ DE 2,5 s », plus de prime à gagner).

### Limites, ce qui n'est PAS testé

- Les toits : HABITUÉ et EXPERT font exactement le parcours du robot de QA (2 037 pas, sans
  erreur) ; l'égalité étant perdue, les battre demande de faire mieux que le robot (il attend 40 pas
  de toucher le sol au départ, par exemple). Laissé tel quel : c'est le vrai niveau du joueur simulé.
- L'arc-en-ciel DÉBUTANT : 4 min 40 et 14 morts (la partie la plus longue, 545 plages).
- L'abandon du fantôme n'est pas montré dans le navigateur (il faudrait un niveau modifié) : il
  repose sur `verdict` (`course/mod.rs`) ; recommencer ou « plus de vies » relance aussi le fantôme.
- Pas de vrai téléphone (bouton COURSE au toucher non essayé), pas de son à l'oreille.

## Nouvelle partie + (carte 8.8)

« Après la fin, les mêmes niveaux avec les ennemis déplacés et la pluie plus forte. »

- Déblocage : la ligne NOUVELLE PARTIE+ (`contenu::NOUVELLE_PARTIE_PLUS`, 16 caractères, 190 px pour 192)
  apparaît au titre, sous JOUER, dès qu'une partie est finie (`parties_finies > 0`, comme HISTOIRE). Le titre
  a alors six lignes : `ecrans::TITRE_SIX_LIGNES` (pas de 17 px au lieu de 18, de y = 52 à 154) ; le
  sous-titre s'efface pour leur faire place. La partie ne raconte pas l'histoire (déjà vue) ; REJOUER garde
  le mode de la partie finie ; un niveau joué seul depuis la carte et la démo du titre restent normaux.
- HUD : un « + » en or devant le chrono pendant toute la nouvelle partie +.
- Variantes (`core/src/levels/variantes.rs`) : les GRILLES NE CHANGENT PAS (la carte 8.6 les modifie en même
  temps). Par CLÉ de niveau, une table écrite à la main de déplacements `(case, case)` et d'ajouts `(lettre,
  case)`, appliquée au texte de la grille au chargement, puis lue par le même parseur. Chaque ennemi qui le
  peut change de côté sur son toit (ou de place dans son couloir de vol) ; restent en place les pigeons
  d'un perchoir d'une case, les gardiens d'un secret et le chat du clocher (sa place va avec le souffle du
  ventilateur). Une première variante du clocher (chat au pied de l'arche, corbeau devant la porte, flaque
  sur F4) : la recherche du robot de QA n'y avait pas abouti au bout de 20 min ; refaite (corbeau au bout de
  F4, flaque sur F2). Un ennemi de plus par niveau (flaque, ou corbeau au marché et dans la descente). Aucun ennemi placé par une variante à moins de 6 cases d'une lanterne ou du
  départ à hauteur de Pépin, et aucune flaque nouvelle ne patrouille sur le toit d'une reprise qui n'en avait
  pas. L'arène du Grand Nuage et l'arc-en-ciel (bonus sans ennemi) n'ont pas de table.
- Plus dur, partout : éclairs plus fréquents (`REPOS_ECLAIR_NG_PLUS` = 110 pas de repos au lieu de 150,
  l'annonce ne change pas) ; Grand Nuage « serré » (`GrandNuage::serrer` : à chaque colère le motif suivant,
  le 2e à 3 points de vie puis le 3e ; le côté du courant suit toujours les points de vie) ; léger vent de
  face sur les quatre niveaux de toits (`NIVEAUX_DE_TOITS`, `physics::VENT_DE_FACE` = 6 px/s vers la
  gauche, plané ou non : 10 fois moins que le vent latéral, la course à 96 px/s le remonte sans peine ; une
  zone `VentLateral::de_face`, sans traînées). Le front du niveau de poursuite est inchangé.
- Pluie plus forte (dessin seulement) : 1,5 fois plus de gouttes par couche et 45° au lieu d'une pente de
  2 (`decor::PLUIE_FORTE_NUM / PLUIE_FORTE_DEN`).
- Sauvegarde : records à part (`RecordsNgPlus` : parties finies, meilleur score, meilleur temps, temps de
  chaque niveau par clé ; lignes `ng_plus=parties:… score:… temps:…` et `ng_plus.<clé>=temps:…`, relues
  avec la même tolérance). Gouttes, défis et médailles comptent comme d'habitude pour le porte-monnaie
  (crédités une seule fois) ; le meilleur temps NORMAL d'un niveau n'est pas touché par la nouvelle partie +.

### Preuves

- `./docker-cargo.sh test -p core --release variantes` : 1 ok (617 s, machine chargée) :
  `chaque_variante_ng_plus_est_finissable` (le seul test nouveau, pour la mécanique du cœur) : chaque
  variante s'applique, même géométrie/gouttes/lanternes/départ, un ennemi de plus, distance aux reprises,
  et chaque niveau NG+ finissable (vent de face compris) par le robot de faisabilité précis (arrivée, 3
  gouttes, lanternes) et maladroit, et depuis chaque lanterne.
- `test -p game --no-default-features` ciblés : `sauvegarde` 2 ok (aller-retour avec les records NG+),
  `ecrans` 24 ok (six lignes sous le grand titre, au-dessus des toits), `contenu` 9, `hud` 6, `decor` 37,
  `app::tests` 54 (menu du titre à six lignes), `menu` 10.
- Robot de QA, troisième partie (`le_robot_finit_la_nouvelle_partie_plus_sans_mourir`, `-- --exact`) :
  ok en 1134 s (machine chargée) : choisie au titre (une ligne sous JOUER), les niveaux 1 à 10 puis
  l'épilogue (bonus sauté), 0 mort ; 24 461 images, chrono 21 688 pas, empreinte `ba4a1ce378de679f`.
  Parcours cherchés dans les variantes (`trouver_parcours_mode`, un fil par niveau).
- Passe finale, une seule, sur l'extraction propre (`git archive` de e702995) : `./docker-cargo.sh test
  --workspace --no-default-features` : core 378 ok (2 ignorés), game 355 ok (28 ignorés), 0 échec (les
  deux parties du robot et la nouvelle partie + comprises) ; `./docker-cargo.sh lint` : LINT-OK ;
  `./build.sh` : game.wasm 602 300 octets (gzip 242 461) ; dans `dist/`, 0 nom interdit, aucun `/home`,
  seule URL `http://www.w3.org/2000/svg`.
- Captures `docs/qa/88-*.png` (build de QA sur 127.0.0.1, Chromium 1208 sans tête par CDP,
  `node docs/capture-ng-plus.mjs PORT docs/qa`), regardées ;
  `mesure-capture.py` : 0 pixel hors palette, 0 flou ; 0 erreur de console :
  `88-titre-ng-plus` (six lignes, NOUVELLE PARTIE+ choisie) ; même endroit, mêmes touches, même image
  après la carte, en normal puis en NG+ : `88-toits-*` (Pépin sur la marche des toits ; en NG+, la flaque
  ajoutée en colonne 25 et, vent de face oblige, Pépin un peu moins loin : il y prend la goutte du saut),
  `88-gouttieres-*` (la flaque passée de la colonne 20 à 14, pluie dense et à 45°, « + » dans le HUD),
  `88-marche-*` (le chat du départ passé de la colonne 12 à 17).

### Ce qui n'est PAS testé

- Vrai téléphone (les lignes du titre à six font 17 px, 1 de moins que le minimum « doigt » de 18) ; le
  vent de face n'a pas été joué à la main ; le Grand Nuage serré n'a pas de capture ; partie complète du
  robot (ignorée) non relancée en NG+.

## Lettres cachées P-É-P-I-N et fin secrète (carte 8.6)

Demande d'AGA : « Un quatrième objet par niveau. Par exemple les lettres P, É, P, I, N dispersées dans le
jeu, pour débloquer une fin différente. »

### Mécanique

- Lettre de grille `*` (libre jusque-là) : une LETTRE cachée, boîte 8 x 8 au milieu de la case comme la
  goutte, une au plus par niveau (`ErreurNiveau::LettreMultiple`). Entité `Entite::Lettre`, contact
  `Contact::Lettre` (ni score ni vie). Laquelle des cinq : table `core/src/lettres.rs`, PAR CLÉ de
  niveau (les deux P sont deux lettres différentes).
- Les cinq niveaux, toujours DERRIÈRE UN FAUX MUR de la carte 8.5 (jamais sur le chemin) : P aux toits
  (salle sous le toit, à côté de sa goutte), É au marché couvert (bloc de boutique creux, devant la
  goutte), P dans le grand orage (tunnel du raccourci, après la goutte), I au clocher (salle de la sortie
  cachée, devant le second drapeau : on peut la prendre et ressortir), N dans la mer de nuages (tunnel
  du raccourci). Début (niveaux 1 et 3), milieu (4 et 5), fin (8). Aucune goutte déplacée, aucun
  chemin changé : la partie normale du robot est inchangée.
- Sprite 10 x 10 (`objets::LETTRE_P/E/I/N`) : un jeton VIOLET cerclé d'or, la lettre en or pâle,
  2 px de contour comme la goutte ; il flotte à contretemps d'elle. Prise : la petite gamme des secrets
  (`Effet::Secret`) et une gerbe d'étincelles or et violet.
- SAUVEGARDE : la lettre prise dans un niveau RÉUSSI (drapeau ou sortie secrète) est notée pour toujours
  sous la clé du niveau : `niveau.<cle>=… lettre:1` (v3 inchangée : un mot de plus, absent = 0, relu
  par les anciennes versions sans erreur). Prise puis niveau raté ou recommencé : rien. Collection entre
  les parties : pas besoin de tout faire dans la même.
- HUD : sur la carte de présentation de chaque niveau, en haut à droite, le petit mot « PÉPIN » (lettres
  trouvées en or, les autres en ardoise pâle, celle du niveau présenté en corail tant qu'elle n'est pas
  trouvée). À l'écran de fin, une ligne avant MORTS : « IL MANQUE N LETTRE(S) » (ou « LETTRES ») et le
  même petit mot. Pour qu'elle tienne, les lignes se resserrent d'un pixel à la fois (10 px au moins),
  puis le bloc remonte de 2 px ; la ligne VIES ne s'affiche alors que si elle tient encore.
- FIN SECRÈTE : à la fin d'une partie, si les cinq lettres sont dans la sauvegarde, une 4e image
  d'histoire (`Chapitre::Secret`, hors du récit, sans numéro) : sous le ciel doré, le soleil sourit,
  couronné de rayons, et écrit « PÉPIN » lettre après lettre (un rayon file vers chaque lettre) ; la
  ville a rallumé ses fenêtres, neuf parapluies s'ouvrent un à un au-dessus des toits, Pépin ouvre le
  sien sur sa cheminée. Texte : « Le soleil écrit ton nom en rayons d'or. / La ville ouvre ses
  parapluies : merci, Pépin ! ». Puis l'écran « FIN SECRÈTE » (ciel doré, « La ville entière fête Pépin
  sous le soleil. »). Revue depuis le titre (HISTOIRE), elle suit l'image 3 une fois les cinq réunies.
- Carte des niveaux (7.4, `plan.rs`) : PAS touchée (la carte 8.13 y travaille en parallèle).

### Robot de QA

- Partie normale : inchangée (aucune lettre, pas de fin secrète ; vérifié par
  `le_robot_finit_tous_les_niveaux_sans_mourir`).
- Partie COMPLÈTE : `trouver_parcours_complet` compte la lettre du niveau comme un objet de plus (avec
  les trois gouttes) ; elle prend les cinq lettres, 0 mort, et voit l'image de la fin secrète avant
  l'écran de fin (`la_partie_complete_joue_le_bonus`, assertions ajoutées). Tenue au décalage : la
  lettre prise compte comme les gouttes.
- Test ignoré `bot_qa::scripts::ecrire_scripts_lettres_86l` : scripts, touches et traces des captures
  (`target/qa/*-86l-*`, `*-86ls-5` : parcours secret du clocher, `*-86lf-12` : la descente) ; la trace a
  une colonne `lettre`.

### Preuves

- Un seul test nouveau : `lettres::tests::chaque_lettre_de_pepin_est_cachee_dans_son_niveau` (core : le
  mot, l'ordre, une case `*` dans ces cinq niveaux NORMAUX et eux seuls, chacune dans une zone cachée).
  Le reste : assertions ajoutées aux tests existants (aller-retour de la sauvegarde avec une lettre ;
  bilans de l'écran de fin avec 0, 2 et 5 lettres manquantes dans `rien_ne_se_chevauche…` ; parties du
  robot) et constantes vérifiées à la compilation (le mot PÉPIN tient dans le ciel, entre le bord, les
  rayons du soleil et les toits).
- Extraction propre (`git archive` de 8a9d18b, le dernier commit de code) : `test -p core` 378 ok
  (2 ignorés) ; `test -p game --no-default-features` 354 ok (29 ignorés : 28 + le test d'écriture des
  scripts), robot compris (partie normale, partie complète avec les cinq lettres et la fin secrète,
  0 mort) ; `./build.sh` : game.wasm 596 007 octets (gzip 240 436), `dist/` sans `/home` ni nom
  propre, seule URL `http://www.w3.org/2000/svg`. Avant le regroupement des tests (381f265) : suite
  game 358 ok. Ciblés sur 8a9d18b : `ecrans` 24, `contenu` 9, `histoire` 7, `sauvegarde` 2,
  `sprites` 54, `app::tests` 54. `./docker-cargo.sh lint` : LINT-OK. `faisabilite` : dans la suite
  core (niveaux touchés compris).
- Vrai navigateur, build de QA (`./build.sh --qa`, servi sur 127.0.0.1, Chromium 1208 sans tête par
  CDP, pilote `docs/capture-lettres.mjs PORT docs/qa target/qa`, horloge virtuelle de
  `docs/pilote-qa.js`) :
  - sauvegarde vide, chacun des cinq niveaux rejoué jusqu'à sa lettre puis au drapeau : localStorage
    relevé après chaque niveau, `lettre:1` sous la bonne clé (toits, marche, orage, clocher, mer) ;
  - page RECHARGÉE : la sauvegarde relue garde `lettre:1` (mer) et la carte de présentation de la mer
    montre le N en or ;
  - sauvegarde à 4 lettres, descente jusqu'au drapeau : « DE RETOUR ! » et « IL MANQUE 1 LETTRE »
    (PÉPI en or, N en ardoise) ;
  - sauvegarde à 5 lettres, même descente : l'image de la fin secrète, puis « FIN SECRÈTE » ; page
    rechargée : 5 lignes `lettre:1` ;
  - 0 erreur de console, 0 erreur du pilote.
- Captures `docs/qa/86l-*.png` (16, regardées) : `86l-<niveau>-1-lettre` (la lettre en place, zone
  révélée) et `-2-prise` pour toits, marche, orage, clocher, mer ; `86l-hud-carte-marche-a-trouver`
  (É en corail), `86l-hud-carte-mer-apres-rechargement`, `86l-hud-carte-4-lettres` ;
  `86l-fin-4-lettres`, `86l-fin-secrete-image`, `86l-fin-secrete`. `mesure-capture.py` : 0 pixel hors
  palette, 0 bloc flou sur les 16.
- Au clocher, le rejeu de la partie COMPLÈTE du robot (elle ressort de la salle et grimpe toute la tour)
  ne finit pas le niveau dans le navigateur (à la fin du script, la sauvegarde n'a pas la ligne du
  clocher), alors que la simulation native le finit sans mourir (le test qui écrit le script le
  vérifie) ; cause non cherchée plus loin. Les captures du clocher rejouent donc le parcours SECRET
  (lettre prise, sortie secrète), qui finit dans la salle.

### Ce qui n'est PAS testé

- Un joueur humain qui cherche les lettres sans les connaître ; le son à l'oreille ; vrai téléphone ;
- la partie complète du clocher rejouée en entier dans le navigateur (voir ci-dessus) ;
- la carte des niveaux (7.4) ne montre pas encore les lettres.

## Carte 8.15 : Le grand orage, la goutte du tunnel secret sans mourir (ascenseur plus bas)

- Diagnostic (moteur, sonde jetable : Pépin posé au sol ou sur l'ascenseur, DROITE tenue, départ à chaque pas du cycle de 213 pas) : en bas, l'ascenseur était juste au niveau du tunnel, mais il y fait demi-tour ; en remontant il plaque Pépin contre le plafond de l'entrée (1 case de haut), qui le rejette et il tombe dans le trou de la colonne 112 (mort `Vide` en 1799,160). Depuis l'ascenseur : 1 départ sur 213 entre ; depuis le toit : 0. Pas d'écrasement. Sortie : un saut sur 90 franchit le trou (fenêtre de 6 images), sinon chute (morts en 2211 à 2217). Le parcours secret du robot ne mourait pas (0 mort), mais il passait au pas près.
- Correction (`orage.rs`, grille seule) : l'extrémité basse du `3` passe de la rangée 8 à la rangée 9 (l'ascenseur descend 16 px plus bas, dans le trou existant) ; faux murs de plus en (113, 6) et (135, 5-6) : entrée et sortie du tunnel hautes de 2 cases, invisibles de dehors. Après : depuis l'ascenseur 183 départs sur 267 entrent (6 chutes), depuis le toit 94 ; sortie : fenêtre de 14 images. Chemin normal par l'ascenseur intact.
- Preuves : `test -p core levels::orage` (0 test ne porte ce filtre : 0 ok), `zones` 55 ok, `faisabilite` 25 ok ; `le_robot_finit_tous_les_niveaux_sans_mourir` ok ; démo : `chaque_parcours_de_la_demo_atteint_le_drapeau` cassait (mort au pas 1591 sur l'orage), parcours régénéré (`ecrire_parcours_de_la_demo`), ok. `ecrire_scripts_secrets_85` relancé : parcours secret de l'orage, goutte du tunnel prise, 0 mort (trace `morts` à 0, rejeu moteur 0 mort, drapeau atteint). `./docker-cargo.sh lint` LINT-OK ; `./build.sh` (game.wasm 608 247 octets).
- Captures (build de QA, Chromium 1208 CDP, `SCRIPT=target/qa/script-85s-4.txt` décalé d'une image : l'histoire du début est prête un pas plus tard dans le navigateur) : `docs/qa/815-orage-ascenseur.png` (Pépin sur l'ascenseur au ras du toit, devant l'entrée), `815-orage-goutte.png` (dans le tunnel, 1 goutte). Regardées : 5 vies tout du long, drapeau atteint.
- PAS testé : à la main au clavier ou au tactile ; vrai téléphone, Safari.

## Carte 7.6b : palettes de couleurs retirées (AGA : « c'est moche »)

- Retirés : catalogue et shader des palettes, onglet de la boutique (4 onglets, centrés), touche O de QA, `palette:origine` de départ ; une sauvegarde qui avait acheté une palette est remboursée de 20 gouttes par palette, une fois (`normaliser`, puis réenregistrée), et ses entrées `palette:*` effacées.
- Preuves : tests `boutique`, `cosmetiques`, `sauvegarde`, `sprites` verts, `lint` vert, robot `le_robot_finit_tous_les_niveaux_sans_mourir` vert ; `./build.sh` : game.wasm 630 543 octets contre 634 540 sur main 5b06ad8 (-3 997).
- Chromium sans tête, build de PRODUCTION, CDP : sauvegarde v3 à 10 gouttes avec `palette:sepia` équipée, 30 gouttes au chargement et plus aucune entrée `palette` ; rechargée : toujours 30 ; 0 erreur console, 0 pixel hors palette. Captures regardées `docs/qa/76b-boutique-quatre-onglets.png`, `76b-jeu-couleurs-normales.png`.
- Pas testé : vrai téléphone, Safari ; le pilote de la preuve (sauvegarde avec palette) est resté hors dépôt.

## Carte 8.16 : poursuite accélérée (mur de pluie du 10e niveau)

- Vitesse du mur : `VITESSE_MUR` 64 px/s (2/3 de la course) avant, 76,8 px/s après (80 %, la limite vérifiée à la compilation) ; NG+ inchangé (même constante, aucune variante touchée).
- `RETARD_MAX` 128 px (2 s) avant, 96 px (1,25 s) après ; `RECUL_REAPPARITION` 112 px ramené à 96 px (au-delà, le mur était de toute façon tiré à `RETARD_MAX` au pas suivant). Grille inchangée.
- Temps du robot sur le niveau : 1 902 pas (31,7 s) avant, 1 902 pas après (même parcours, 0 mort ; traces `ecrire_script_navigateur`). Mur rejoué sur la trace : marge mini 41,7 px (2,6 tuiles) au départ, 75 px ou plus ensuite (88 px aux grêles), contre 83 px avant ; toujours plus d'une demi-tuile. Preuves : `test -p core poursuite` (test du mur adapté aux constantes), `faisabilite` (25), `zones` (55), robot `le_robot_finit_tous_les_niveaux_sans_mourir` et `le_robot_finit_la_nouvelle_partie_plus_sans_mourir` verts, lint, `./build.sh`. Non testé : un humain au clavier ou au toucher (1,25 s d'hésitation suffit à se faire prendre), vrai téléphone. Remarque : le niveau 4 a pris 13 pas de plus au robot entre les deux traces sans lien avec cette carte (aucun mur au niveau 4).

## Carte 8.17 : le boss à 95 % de la vitesse de Pépin (mur de pluie du 10e niveau)

- Vitesse du mur : `VITESSE_MUR` 76,8 px/s (80 %, carte 8.16) avant, 91,2 px/s (95 %, `0.95 * VITESSE_COURSE` = 96 px/s) après, soit 5,7 tuiles par seconde. L'assert de compilation passe de `VITESSE_MUR <= 0.8 * VITESSE_COURSE` à `<= 0.95 * VITESSE_COURSE`. NG+ inchangé (même constante, aucune variante touchée). Grille inchangée. Pépin ne le distance plus que de 4,8 px/s en courant : toute hésitation se paie.
- Robot (parcours du niveau rejoué dans `Monde`) : `le_robot_finit_tous_les_niveaux_sans_mourir` et `le_robot_finit_la_nouvelle_partie_plus_sans_mourir` verts (0 mort, drapeau atteint), même parcours (1 878 pas, 31,3 s, normal ; NG+ 1 880 pas). Marge du mur mesurée sur le parcours du robot (front moins le bord gauche de la boîte de Pépin, image par image) :
  - carte 8.16 (76,8 px/s) : marge mini 43,0 px (2,7 tuiles) au pas 45, marge au départ 131,0 px ;
  - carte 8.17 (91,2 px/s) : marge mini 32,3 px (2,0 tuiles) au pas 46, marge au départ 131,0 px ; identique en NG+.
  - Le point le plus serré est au départ (0,8 s, pendant que Pépin accélère depuis 0), puis l'écart se rouvre (le mur reste derrière). 32,3 px, c'est 0,35 s d'avance au front à 91,2 px/s : jouable, plus tendu qu'avant. (La marge 8.16 ici, 43,0 px, est de la même sonde que la 8.17 ; la marge 8.16 mesurée par sa propre sonde était 41,7 px, référence de 2,6 tuiles.)
- Preuves : `test -p core poursuite` (test du mur adapté aux constantes) vert ; suite `core` complète (380) verte ; robots `le_robot_finit_tous_les_niveaux_sans_mourir` + `le_robot_finit_la_nouvelle_partie_plus_sans_mourir` verts ; suite `game` complète (358) verte ; lint LINT-OK ; `./build.sh` : game.wasm 646 683 octets (gzip 259 560), sous l'objectif de 1,2 Mo. Non testé : un humain au clavier ou au toucher (0,35 s d'hésitation suffit à se faire prendre), vrai téléphone, son à l'oreille.
- Correction au passage (échec préexistant, hors périmètre 8.17) : `course::tests::chaque_partie_des_fantomes_rejoue_sans_divergence` (« Le grand orage », force Débutant) échouait déjà sur l'arbre propre 8.16 (reproduit : divergence au pas 1 933, 1 mort, Abandon au lieu d'Arrive 2 651). Cause : la carte 8.15 a modifié le niveau « Le grand orage » (ascenseur une rangée plus bas, entrée et sortie du tunnel hauts de 2 cases) sans régénérer `game/src/course/parties.rs` (les parties des fantômes embarquées, dernier passage en carte 8.7) ; le fantôme du grand orage ne se rejouait plus. Correction : relance de `ecrire_parties_des_fantomes` (test ignoré, commandement documenté dans `course/mod.rs`). Seule « Le grand orage » est re-enregistrée, les 3 forces (pas 2 651, 2 625, 2 925 et 1 mort avant ; 2 463, 2 407, 2 327 et 0 mort après) ; le fantôme de « Le retour du Grand Nuage » (poursuite) est inchangé (le mur à 95 % ne modifie pas le parcours du joueur simulé). Test du fantôme de nouveau vert.

## Carte 9.5 : décors du Monde 2 « Là où va la pluie » (7 thèmes, fonds seulement)

- Thèmes ajoutés EN FIN d'enum (`Caniveau`, `Egouts`, `Cascade`, `Grotte`, `Riviere`, `Port`, `Maree`) : indices du Monde 1 intacts, aucune grille ni physique touchée (`core` : l'enum, `luminosite`, `pluie`, lus par le rendu seulement). Décor : silhouettes `Roche` (joints découpés, sans couleur de plus), `Roseaux`, `Coques` ; plafond à stalactites ; motifs animés en 3 groupes de parallaxe (eau, chutes, brume, tuyaux, bulles, cristaux, lampadaires, grues, mouettes, phare) ; cascade, grotte et marée verticales (`vertige` du clocher) ; grotte sèche. Tuiles : briques mouillées (violet vers cyan sombre) aux égouts et au port, béton clair au port. Musiques : replis existants (les vraies à la carte 9.14).
- Preuves : `test -p core zones` 55 ok ; game `decor` 37 et `sprites` 54 ok (`contraste` : aucun test de ce nom ; Pépin lisible à 4,5:1 sur chaque bande de chaque thème : `l_icone_des_vies_est_pepin_et_se_lit_sur_tous_les_ciels`, dans `sprites`).
- Extraction propre de 2a13ce8 : core 380 ok ; game 357 ok, 1 échec PRÉEXISTANT `course::tests::chaque_partie_des_fantomes_rejoue_sans_divergence` (« Le grand orage » Debutant, pas 1933), identique sur une extraction de d09c84f (fantôme obsolète depuis 8.15, ré-enregistré sur main par f5d2a07, carte 8.17). Robot (tous les niveaux, nouvelle partie +) et démo verts. LINT-OK ; `./build.sh` : game.wasm 653 767 octets contre 646 931 sur d09c84f (+6 836, gzip +2 330) ; `dist/` : 0 nom interdit, seule URL `http://www.w3.org/2000/svg`.
- Captures (build de QA, Chromium 1208 CDP, `node docs/capture-fonds-95.mjs PORT` : la touche V de QA fait défiler les 7 fonds en plein écran) : `docs/qa/95-caniveau|egouts|cascade|grotte|riviere|port|maree.png`, regardées une à une ; `python3 docs/mesure-capture.py <png> 4` : 0 pixel hors palette, 0 bloc flou.
- PAS testé : un vrai niveau avec ces fonds (cartes 9.6 à 9.13), Pépin en jeu devant eux, vrai téléphone, Safari ; sur la planche, les niveaux verticaux sont vus au pied (caméra du fond à 0).

## Carte 9.1 : notion de MONDE (carte des niveaux à deux mondes, déblocage, partie Monde 2)

- Cœur : `InfosNiveau::monde` (1 pour les 12 niveaux) ; numéros, bonus, épilogue et `apres` PAR MONDE (`niveaux_du_monde`, `debut_monde`), une partie ne passe jamais d'un monde à l'autre. Faux Monde 2 du build de QA seulement : `levels::TOITS_MONDE2` (copie des toits, clé `toits_monde2`), ajouté à la fin de `NIVEAUX` par la fonctionnalité `monde2_qa` (activée par `debug_niveaux`) ; les `INDICE_ESSAI*` suivent (`NB_NIVEAUX` vaut 13 en QA, tests des raccourcis d'essai verts avec la fonctionnalité). Point médian ajouté à la police.
- Jeu : porte = `poursuite` fini ; carte des niveaux par monde (Droite après COURSE, Gauche avant JOUER, bouton « MONDE 2 » / « MONDE 1 » à la place du rappel des touches ; une carte plus courte que la vue descend pour la remplir) ; titre : MONDE 2 sous JOUER (7 lignes au pas de 14 px) ; « MONDE 2 · NIVEAU 1/1 » (carte de titre, panneau) ; compteur de gouttes et écran de fin au total du monde, sans ligne des lettres ; records `partie.monde2`, bonus de fin `monde2_*`. NG+ et course : Monde 1 seulement (COURSE grisé sans fantôme). La mesure de difficulté (Monde 1, tableaux de `NB_NIVEAUX`) n'est plus compilée avec `debug_niveaux`.
- Preuves : `test -p core levels` 32, `zones` 55, `font` 22 ; `test -p game --no-default-features` `plan` 7, `app` 60, `ecrans` 24, `sauvegarde` 4 (et `contenu`, `defis`, `hud`) ; avec `--features debug_niveaux` (raccourcis, plan, titre) 18 ; `lint` LINT-OK ; robot `tous_les_niveaux` et `nouvelle_partie_plus` verts (1 335 s) ; `./build.sh` game.wasm 651 624 octets (QA 663 115), `toits_monde2` absent de `dist/`, présent dans `dist-qa/`.
- Navigateur (Chromium 1208, CDP, `docs/capture-91.mjs`, sauvegarde avec `poursuite` fini). QA : titre avec MONDE 2 ; CARTE ouvre le Monde 2, bouton MONDE 1 puis Droite y ramènent ; faux niveau joué depuis la carte (niveau 1 du robot) puis bilan ; titre, MONDE 2, écran de fin : `partie.monde2=parties:1 score:100 temps:2037`, Monde 1 intact (`parties_finies=0`). Production : titre et carte inchangés, Droite reste sur COURSE. 0 erreur console. Captures regardées : `docs/qa/91-qa-titre-ligne-monde-2.png`, `91-qa-carte-monde-2.png`, `91-prod-carte-sans-monde-2.png` (la 1re passe montrait le bas de la vue non redessiné sur une carte d'un seul niveau : corrigé, testé).
- PAS testé : toucher, vrai téléphone, Safari ; un vrai Monde 2 de plusieurs niveaux (son bonus et son épilogue testés seulement sur de fausses listes) ; `test --workspace` complet.

## Carte 9.2 : eau et barque (tuile d'eau, flotter/couler, courants d'eau, rendu, niveau d'essai)

- Mécanique (core) : tuiles `~` (eau calme), `{` et `}` (courant vers la gauche / la droite), NON
  solides, assemblées en étendues (`Niveau::eaux`, surface = haut de l'étendue). Planer dans l'eau :
  Pépin devient une BARQUE et la garde sans tenir la touche ; il rame à 40 % de la course
  (`VITESSE_RAME` 38,4 px/s), le courant pousse la barque à 24 px/s (`VITESSE_COURANT_EAU`, plus
  lent que la rame, vérifié à la compilation : on peut ramer contre) ; le SAUT l'en fait sortir
  (un saut normal, les pieds partant 3 px sous la surface). Pieds sous la surface sans planer (en tombant OU en marchant
  dedans) : noyade, `CauseMort::Noyade`, une vie. Protégé (réapparition) : recueilli en barque.
- Recherche : `Fin::Barque` est un point de décision dans la faisabilité et dans le robot de QA ;
  un départ au ras de l'eau (lanterne posée sur l'eau) part à bord. Règle de grille : une lanterne
  peut être posée sur l'eau (`aucune_lanterne_ne_flotte`).
- Rendu : 9 tuiles d'eau (calme, courant gauche, courant droite x dessus, dessus 1 px plus bas,
  intérieur), deux couleurs seulement (`PLUIE`, `PLUIE_SOMBRE`), la surface ondule d'un pixel
  (alternance toutes les 24 images, à contretemps d'une colonne à l'autre) ; barques 16x16 de
  Pépin (coque corail) et de Pépinette (coque cyan, nœud corail). Planches :
  `docs/qa/92-planche-eau.png`, `docs/qa/92-planche-skins-barque.png` (`docs/planche-skins.py`).
- Niveau d'essai « eau » (`core/src/levels/essai_eau.rs`, touche F11 du build de QA) : bassin creusé
  de 320 px (trop large pour un saut planant depuis le sol), courant vers un quai, une lanterne et
  une goutte posées sur l'eau. Le robot (parcours complet) : 3 gouttes, 0 mort, 23 images en barque.
- Tests ajoutés : `planer_dans_leau_donne_la_barque_sinon_noyade` (core, la mécanique : barque,
  rame, saut, noyade, réapparition protégée) et `le_robot_finit_le_niveau_d_essai_eau` (game) ;
  test ignoré `ecrire_script_essai_eau_92` (script des captures).
- Preuves, sur une extraction propre de 913f1fc : `test --workspace --no-default-features` : core
  381 ok ; game 358 ok, 1 ROUGE, `chaque_partie_des_fantomes_rejoue_sans_divergence` (« Le grand
  orage » Debutant : divergence pas 1933) : échec PRÉEXISTANT, identique à l'octet sur une extraction
  propre de d09c84f (main avant 9.2), dû à 8.15 ; carte de suivi t_18b1afea. `lint` LINT-OK.
  `./build.sh` : game.wasm 655 764 octets (646 931 sur d09c84f), `dist/` : 0 nom interdit, aucun
  `/home`, seule URL `http://www.w3.org/2000/svg`. Monde 1 inchangé : les trois parties du robot
  ont l'empreinte IDENTIQUE à celle de d09c84f : normale (`ecrire_script_navigateur`) 23 703
  images, 0 mort, 13 gouttes, score 2400, `41a73ef4a0866536` ; complète 28 995 images,
  `c070c4ae5c42223d` ; nouvelle partie + 24 472 images, `d95fc98f74c6d8ed`.
- Captures (build de QA `./build.sh --qa` sur 127.0.0.1, Chromium 1208 sans tête par CDP,
  `node docs/capture-92.mjs PORT docs/qa target/qa/script-92-eau.txt`) : `92-eau-depart`,
  `92-eau-plane`, `92-eau-barque-pepin`, `92-eau-saut-vers-le-quai`, `92-eau-quai`, les mêmes pour
  Pépinette (`-pepinette`), `92-eau-noyade-1`, `-2`, `-reapparition`, `92-eau-noyade-pepinette`.
  Toutes regardées ; 0 pixel hors palette, 0 flou, 0 erreur de console.

### Ce qui n'est PAS testé

- Un joueur humain au clavier ou au tactile ; vrai téléphone, Safari ; le son (aucun son ajouté).
- F11 dans un navigateur de bureau fenêtré : le jeu reçoit la touche (pilote CDP), mais le
  navigateur peut AUSSI passer en plein écran (Échap ou F11 pour en sortir).
- Le courant vers la gauche `{` n'est dans aucun niveau : vu sur la planche seulement.
- Aucun niveau du Monde 2 n'utilise encore l'eau (cartes 9.6 et suivantes).

## Carte 9.4 : ennemis d'eau du Monde 2 (grenouille, poisson, crabe, rat, mouette)

- Lettres `j` grenouille (guette, se ramasse 0,5 s avec un « ! », bondit en arc vers Pépin, écrasable), `i` poisson (case d'air au-dessus de l'eau ; remous 0,7 s, saut vertical de 3,5 tuiles, mortel hors de l'eau, jamais écrasable), `k` crabe (patrouille ; sauté dessus il fait REBONDIR sans être blessé, pinces qui claquent ; de côté il tue ; une pince levée, les deux quand Pépin est au-dessus), `r` rat (Pépin sur sa ligne : se dresse 0,5 s avec un « ! », fonce jusqu'au mur, étourdi 0,75 s ; écrasable), `u` mouette (le code du pigeon, autre dessin). Code : `core/src/entities/ennemi/eau.rs`, dessin `game/src/dessin_eau.rs`, 29 sprites `game/src/sprites/ennemis_eau.rs` (planche régénérée).
- Essai `core/src/levels/essai_ennemis_eau.rs`, touche J du build de QA (`NB_NIVEAUX + 12`). Robot QA : 0 mort, 3 gouttes (`le_robot_finit_le_niveau_d_essai_ennemis_eau`, phase des ennemis dans sa case : `ennemis_d_eau_de_la_case`). Robot de faisabilité : ces ennemis sont ignorés comme flaque et pigeon (évitables en attendant ou en sautant), documenté en tête de `faisabilite.rs`.
- Tests : 4 règles centrales dans `eau.rs` (grenouille, poisson, crabe qui fait rebondir sans mourir, rat), la mouette réutilise les tests du pigeon. Extraction propre de a82e66e : core 387 ok, game 365 ok (0 rouge), `./build.sh` : 0 nom interdit, seule URL `http://www.w3.org/2000/svg`, aucun `/home`. `lint` : LINT-OK après correction d'un `redundant_guard` (commit suivant).
- Monde 1 identique : empreintes du robot égales à celles de 844491b (même commande, `--exact`) : normale 23 703 images `41a73ef4a0866536`, complète 28 995 `c070c4ae5c42223d`, nouvelle partie + 24 472 `d95fc98f74c6d8ed`.
- Captures (build de QA, Chromium 1208 CDP, `node docs/capture-94.mjs PORT docs/qa target/qa/script-94-*.txt nom:pas...`, scripts des tests ignorés `ecrire_script_essai_ennemis_eau_94` et `ecrire_scenarios_94`) : `docs/qa/94-*.png`, 17 images (chaque ennemi au repos, en annonce, en attaque ; rebond sur le crabe ; arrivée à 6 vies, 3 gouttes). Toutes regardées ; 0 pixel hors palette, 0 flou, 0 erreur de console.
- Pas testé : un joueur humain, vrai téléphone, Safari ; aucun son ajouté ; aucun niveau du Monde 2 n'utilise encore ces ennemis (cartes 9.6 et suivantes) ; grenouille posée sur un nénuphar (`n`, carte 9.3) non essayée.

## Carte 9.3 : mécaniques d'eau (bulles `Y`, nénuphar `n`, marée `=`, stalactite `V`, chute d'eau `|`)

- Fait : 5 entités (`core/src/entities/{bulles,nenuphar,maree,stalactite,chute}.rs`, un test de règle chacune), grille et `EauMalPosee`, contacts, faisabilité (bulles et chutes = courants, marée basse/pleine dans l'état du robot, aucun état posé sous une stalactite), rendu `game/src/eau.rs` + sprites `objets_eau.rs` (PLUIE/PLUIE_SOMBRE), essai « eau 2 » (F12, `essai_eau2.rs`). `H` étant la cheminée, la marée prend `=`.
- Preuves (extraction propre de 47f6092) : suite complète verte (core 388, game 365), `lint` LINT-OK, `./build.sh` : 0 nom interdit, seule URL `http://www.w3.org/2000/svg`. Monde 1 identique : normale 23 703 images, 0 mort, `41a73ef4a0866536` ; complète `c070c4ae5c42223d` ; nouvelle partie + `d95fc98f74c6d8ed`.
- Robot : `le_robot_finit_le_niveau_d_essai_eau2` (3 gouttes, 0 mort, barque montée par la marée) ; `reprise_depuis_chaque_lanterne` passe par la marée haute.
- Captures `docs/qa/93-*.png` (`node docs/capture-93.mjs PORT docs/qa target/qa/script-93-eau2.txt target/qa/script-93-nenuphar.txt`, scripts du test ignoré `ecrire_script_essai_eau2_93`) : stalactite (tremble, chute), barque, bulles, marée (basse, annonce, barque qui monte, saut), chute d'eau, nénuphar (posé, enfoncé, noyade). Toutes regardées ; 0 hors palette, 0 flou, 0 erreur de console.
- PAS testé : joueur humain, tactile, vrai téléphone, Safari ; F12 dans un navigateur de bureau (ouvre aussi les outils de développement) ; le robot traverse le bassin en barque (le nénuphar n'est prouvé que par le test et les captures) ; aucun niveau du Monde 2 ne les utilise encore (9.6 et suivantes).

## Carte 9.6 : « Le caniveau », 1er niveau du Monde 2, et infrastructure des niveaux du Monde 2

- Fait : le faux Monde 2 (`TOITS_MONDE2`, `monde2_qa`) retiré ; `core/src/levels/caniveau.rs` (128 x 11 : caniveau à sec avec une flaque, filet d'eau `~}` sous le panneau « planer » et sa bordure, gouttière `}` sur la maison sous un avant-toit dont la goutte ne s'atteint qu'en barque, courant de la rue jusqu'à la bouche d'égout ; 3 gouttes, 4 lanternes). `NB_NIVEAUX_MONDE_1`/`NIVEAUX_MONDE_1` pour ce qui ne décrit que le Monde 1 (difficulté, course, robots, `INTROS`) ; `INTROS_MONDE_2` par clé (vérifiées à la compilation) ; 8 places de faisabilité réservées. Recette d'un niveau du Monde 2 : `pepin-notes/ETAPE-9.md`.
- Tests ciblés verts : core `levels::` et `zones::` (avec les 3 du caniveau), `faisabilite::`, `lettres::` ; game hors robots 335 ok SANS et AVEC `debug_niveaux` (les 2 tests `app` rouges depuis 9.1 réparés : dernier niveau DU MONDE 1). `lint` LINT-OK, `fmt` OK. `./build.sh --qa` et `./build.sh` OK (la production de CETTE branche a le caniveau : la branche ne va pas en production).
- Monde 1 identique (`--exact`, comparé à d4e7bee) : normale 23 703 images `41a73ef4a0866536`, complète 28 995 `c070c4ae5c42223d`, nouvelle partie + 24 472 `d95fc98f74c6d8ed`.
- Robot du Monde 2 : `le_robot_finit_le_monde_2_sans_mourir` (titre, MONDE 2, caniveau, écran de fin ; 0 mort, `9c8d16321a6d2fc8`) ; partie complète du test ignoré `ecrire_script_monde_2_96` : 3 gouttes, 0 mort, 29,5 s.
- Captures (build de QA, Chromium 1208 CDP, `node docs/capture-96.mjs PORT DOSSIER target/qa/script-96.txt nom:image...`) : `docs/qa/96-carte-monde-2.png` (MONDE 2 · NIVEAU 1/1 LE CANIVEAU), `96-barque-filet.png` (barque contre la bordure), `96-gouttiere-courant.png` (le courant sous l'avant-toit, la goutte devant), `96-goutte-en-barque.png` (3/3). Regardées ; 0 pixel hors palette, 0 flou, 0 erreur de console.
- PAS testé : joueur humain, tactile, vrai téléphone, Safari ; musique : repli `ToitsDeNuit` (vraies musiques en 9.14) ; la mesure de difficulté ne couvre pas le Monde 2 (AGA juge en jouant).

## Carte 9.12a : la Pieuvre du port (boss du Monde 2 : entité, lettre `8`, essai touche L)

- Fait : entité `core/src/entities/pieuvre.rs` (machine à états en pas, réutilise la `Salve` du Grand Nuage : 9 colonnes de 2 cases, tentacule annoncé 48 pas puis dressé 30 pas (mortel) puis redescente 12 ; motifs 1, 2, 3 colonnes en vague, plus courts après chaque coup ; tête sortie 2 s au milieu, à gauche, à droite ; la toucher autrement qu'en tombant dessus tue ; 3 PV, 2 en facile ; plongée + clé puis niveau fini). Lettre `8` (sortie des plateformes mobiles, aucun niveau ne s'en servait), `Entite::points_du_boss`/`boss_vaincu`/... pour le HUD, le mode facile et le robot. Rendu `game/src/dessin_pieuvre.rs`, 15 sprites `game/src/sprites/pieuvre.rs` (planche régénérée), encre (`CauseMort::Pieuvre`, rangées d'ardoise 0,75 s).
- Essai `core/src/levels/essai_pieuvre.rs`, touche L (`INDICE_ESSAI_PIEUVRE = NB_NIVEAUX + 13`). `NIVEAUX` non touchée.
- Tests : 1 règle centrale (`la_tete_ne_se_touche_qu_en_lui_tombant_dessus`) + robot `le_robot_bat_la_pieuvre_sans_mourir` (0 mort, 3 gouttes, 1 271 images). Faisabilité : tête toujours sortie aux 3 places (`zones` 55 ok, reprise depuis la lanterne comprise). `lint` LINT-OK. `./build.sh --qa` ok.
- Monde 1 identique (`--exact`, release) : normale 23 703 `41a73ef4a0866536`, complète 28 995 `c070c4ae5c42223d`, nouvelle partie + 24 472 `d95fc98f74c6d8ed`.
- Captures `docs/qa/912-*.png` (`node docs/capture-912.mjs PORT docs/qa target/qa/script-912-pieuvre.txt nom:pas...`, script du test ignoré `ecrire_script_essai_pieuvre_912`) : annonce, tentacule dressé, tête sortie, tête touchée, vague, victoire, encre (Pépin immobile). Toutes regardées ; 0 hors palette, 0 flou, 0 erreur de console.
- PAS testé : joueur humain, tactile, vrai téléphone, Safari ; aucun son nouveau (coup = son d'écrasement) ; le vrai niveau du boss (9.12b) ; le mode facile n'est prouvé que par le test.

## Carte 9.8 : « La grande cascade », niveau vertical du Monde 2 (on descend entre les chutes d'eau)

- Fait : `core/src/levels/cascade.rs` (32 x 46, quatre écrans, départ en haut, drapeau en bas) : 4 bassins étagés A à D (courants `}` et `{`), chute C1 à descendre EN PLANANT jusqu'au bassin B (fermé : noyade), puits d'une tuile sous une stalactite à traverser parapluie FERMÉ, goutte dans une niche derrière la chute C2 (franchie en planant) ; un poisson `i` (bassin B) et une mouette `u` (rocher du bas) ; 3 gouttes, 3 lanternes. Une ligne dans `levels.rs` (`numero` 2 dans cette branche, recette point 3) et dans `INTROS_MONDE_2` ; le test de la caméra connaît ce 3e niveau vertical.
- Preuves : core 404 ok (zones dont `monde_2_niveau_2_faisable`, levels, faisabilite) ; game : suite complète 366 ok, le seul rouge (`les_niveaux_d_un_ecran_gardent_la_meme_camera`, liste des niveaux verticaux) corrigé puis vert ; `lint` LINT-OK ; `./build.sh --qa` OK.
- Monde 1 identique (`--exact`) : normale 23 703 images `41a73ef4a0866536`, complète 28 995 `c070c4ae5c42223d`, nouvelle partie + 24 472 `d95fc98f74c6d8ed`.
- Robot du Monde 2 : `le_robot_finit_le_monde_2_sans_mourir` (niveaux [1, 2], 3 823 images, 0 mort, `772a50caec3886a2`) ; partie complète `ecrire_script_monde_2_96` : 6 gouttes sur 6, 0 mort.
- Captures (build de QA, Chromium 1208 CDP, `node docs/capture-96.mjs PORT DOSSIER target/qa/script-96.txt nom:image...`) : `docs/qa/98-chute-en-planant.png` (image 2700), `98-bassins-etages.png` (2640), `98-goutte-derriere-la-chute.png` (3135, 5/6), `98-puits-stalactite.png` (3500), `98-carte-monde-2.png` (NIVEAU 1/2). Regardées ; 0 pixel hors palette, 0 flou, 0 erreur de console.
- PAS testé : joueur humain, tactile, vrai téléphone, Safari ; la mort en planant dans le puits (calcul seul : 7 tuiles à 56 px/s, 2 s, contre 0,75 s avant la chute de la stalactite) ; le poisson et la mouette ne sont qu'évités par le robot.

## Carte 9.1b : le chapitre 2 est la suite du jeu (décision AGA)

- Fait : plus de ligne MONDE 2 au titre (`Choix::Monde2` et `TITRE_SEPT_LIGNES` retirés, 6 lignes au plus) ; JOUER reprend là où on en est (`Application::monde_de_jouer` : Monde 1 tant que `poursuite` n'est pas fini, puis le Monde 2, aussi quand tout est fini) ; carte des niveaux sans bouton de monde, une seule suite (Haut au bout du Monde 1 ou Droite après COURSE : chapitre 2 ; Bas ou Gauche : retour), bandeau « CHAPITRE 2 : LÀ OÙ VA LA PLUIE » (179 px, tient) sur le Monde 2 seulement ; carte de titre du niveau « CHAPITRE 2 · NIVEAU 1/n » (bonus : « CHAPITRE 2 · NIVEAU BONUS »), panneau de la carte « NIVEAU 1/n » (le chapitre ne tient pas dans ses 128 px) ; Monde 1 sans mot chapitre.
- Tests ciblés verts sans et avec `debug_niveaux` : `app` 60, `plan` 7, `ecrans` 24, `contenu` 9 (tests 9.1 adaptés, aucun ajouté). `lint` LINT-OK ; `./build.sh --qa` OK.
- Robots (`--release --exact`) : Monde 1 identique à 9.6 : normale 23 703 `41a73ef4a0866536`, complète 28 995 `c070c4ae5c42223d`, nouvelle partie + 24 472 `d95fc98f74c6d8ed` ; Monde 2 lancé par JOUER, 0 mort, `d7a345b004c0819c` (attendu différent de 9.6 : plus de BAS au titre). `docs/capture-96.mjs` suit le nouveau titre.
- Captures (build de QA, Chromium 1208 CDP, `node docs/capture-91b.mjs PORT docs/qa`) : `91b-titre.png` (JOUER, CARTE, BOUTIQUE, COMMENT JOUER), `91b-carte-chapitre-2.png` (Bas puis Haut : titre du chapitre), `91b-niveau-chapitre-2.png` (CHAPITRE 2 · NIVEAU 1/1). Regardées ; 0 hors palette, 0 flou, 0 erreur console.
- PAS testé : joueur humain, vrai téléphone, Safari ; au tactile, plus aucun moyen de changer de chapitre sur la carte (bouton retiré, AGA ; la carte s'ouvre sur le chapitre où l'on en est).

## Carte 9.9 : « La grotte aux bulles », niveau 4 du Monde 2 (clé `grotte`)

- Fait : `core/src/levels/grotte.rs` (36 x 48, plus de quatre écrans de haut, on MONTE en zigzag) : 4 colonnes de bulles `Y` (la 3e et la 4e se passent EN L'AIR, en planant, sous une stalactite), 7 stalactites `V` (passages, couloir bas, sortie des colonnes), 2 bassins `~` et un nénuphar, un crabe et une grenouille, 3 gouttes (la risquée : long plané depuis le haut de la 4e colonne vers une petite corniche), 3 lanternes, le drapeau en haut. Une ligne dans `levels.rs` et dans `INTROS_MONDE_2` ; `numero: 2` sur cette branche (le coordinateur le range 4e à la fusion). Un seul test de niveau : `les_bulles_font_le_niveau` (sans bulles, ni drapeau ni goutte risquée ; aucun appui entre la 3e et la 4e colonne).
- Preuves : `test -p core zones:: levels:: faisabilite::` vert, puis `levels::grotte` vert (`monde_2_niveau_2_faisable`, reprise depuis chaque lanterne, durées, lanternes sur un danger inchangées) ; `le_robot_finit_le_monde_2_sans_mourir` : niveaux [1, 2], 0 mort, `41507e9ca08c5aa6` ; `lint` LINT-OK ; `./build.sh --qa` OK.
- Monde 1 identique (`--exact`) : normale 23 703 images `41a73ef4a0866536`, complète 28 995 `c070c4ae5c42223d`, nouvelle partie + 24 472 `d95fc98f74c6d8ed`.
- Captures (build de QA, Chromium 1208 CDP, `node docs/capture-96.mjs PORT DOSSIER target/qa/script-99.txt nom:image...` ; script et images du test ignoré `ecrire_script_monde_2_99`, `game/src/bot_qa/captures_99.rs` : 6 gouttes, 0 mort) : `docs/qa/99-stalactite-tombe.png`, `99-ascension-bulles.png`, `99-goutte-risquee.png`. Regardées ; 0 pixel hors palette, 0 flou, 0 erreur de console.
- PAS testé : joueur humain, tactile, vrai téléphone, Safari ; difficulté « un cran au-dessus de la cascade » jugée sur le dessin seulement (la cascade, carte 9.8, est faite en parallèle ; la mesure de difficulté ne couvre pas le Monde 2) ; musique : repli par thème.

## Carte 9.12b : la Pieuvre aussi dure que le Grand Nuage, et son niveau « La Pieuvre du port »

- Recalibrage (`pieuvre.rs`, `MOTIFS`) : part du bassin frappée par salve 44 %, 56 %, 67 % (Grand Nuage 43, 50, 47 % ; 9.12a 11, 22, 11 %), une salve toutes les 60, 40, 30 pas ; motif 3 = un trou de 3 colonnes qui balaie le bassin aller et retour. Annonce 48, `TETE_PAS` 120, accélération : inchangés. Tests `aussi_dure_que_le_grand_nuage`, `chaque_motif_se_traverse` (le raisonnement du nuage, plus strict : aucune impasse au début de chaque frappe).
- Mesure (`difficulte::tests::mesure_des_deux_boss`, ignoré : graine officielle, 48 essais, mêmes profils ; « avant » mesuré sur une copie aux motifs de d6d7e66), morts moyennes :

| Arène | Débutant | Habitué | Expert |
| --- | --- | --- | --- |
| L'œil du cyclone (référence 5.18 retrouvée) | 4,06 | 1,19 | 0,38 |
| Pieuvre, motifs 9.12a | 2,44 | 0,88 | 0,33 |
| Pieuvre, motifs 9.12b | 5,31 | 1,27 | 0,38 |

- Lecture : Habitué 1,27 ≥ 1,19, but atteint. Mais près des deux tiers des morts de l'Habitué à la pieuvre sont des NOYADES (39 sur 61 : sauts ratés entre pontons, que les tentacules imposent) ; tentacules seuls 0,46, contre 1,19 d'éclairs au nuage. Débutant : game over 23/48 (cyclone 16/48) ; le mode facile reste le filet. Tous les profils finissent 48/48.
- Niveau `core/src/levels/pieuvre.rs` (clé `pieuvre`, `Theme::Port`, `Normal` comme le cyclone, arène de l'essai L, test d'identité de géométrie) ; `numero` 2 sur cette branche (`verifier_ordre` refuse 7 sans les niveaux 2 à 6 : au coordinateur de ranger) ; intro « La Pieuvre garde le port. Elle ne te laissera pas passer. », conseil « Elle frappe là où l'eau tremble. ». `durees_plausibles` compte le combat de la pieuvre avec SES durées ; `trouver_parcours_complet` : repli sur le parcours du robot dans une arène de boss sans lettre (dernier essai seulement).
- Preuves : `test -p core` pieuvre, zones, levels, faisabilite ; robots `le_robot_bat_la_pieuvre_sans_mourir` (0 mort, 1 431 images) et `le_robot_finit_le_monde_2_sans_mourir` (niveaux [1, 2], 0 mort) ; Monde 1 identique (`--exact`) : normale 23 703 `41a73ef4a0866536`, complète 28 995 `c070c4ae5c42223d`, nouvelle partie + 24 472 `d95fc98f74c6d8ed` ; `lint` LINT-OK ; `./build.sh --qa` ok. Captures `docs/qa/912b-*.png` (`node docs/capture-912b.mjs PORT docs/qa target/qa/script-912b.txt nom:image...`, script du test ignoré `ecrire_script_pieuvre_912b`) : carte de titre, vague aller (Pépin plane dans le trou), plus loin, retour (Pépin attend sur la caisse) ; regardées, 0 hors palette, 0 flou, 0 erreur de console. PAS testé : joueur humain, tactile, vrai téléphone ; HUD encore « MONDE 2 · NIVEAU 2/2 » (9.1b).

## Carte 9.7 : « Les égouts », niveau 2 du Monde 2

- Fait : `core/src/levels/egouts.rs` (146 x 11, un écran de haut) : couloir d'un rat ; bassin à marée dont la 1re goutte attend au fond, dans une niche (à marée basse) ; collecteur à marée sous une voûte basse (à pied à marée basse, en barque à marée haute) ; canal `}` sous une voûte où la 2e goutte ne se prend qu'en barque ; galerie en cul-de-sac gardée par un rat (3e goutte) ; couloir de deux rats sous une stalactite. 4 rats, 4 lanternes, 2 souches `H`. Intro et conseil dans `INTROS_MONDE_2`.
- Robot de QA (`bot_qa/parcours.rs`, `pres_de_la_maree`) : l'heure de la marée ne compte plus (case, `tient_au_decalage`) qu'à moins de 8 tuiles d'une zone de marée. Avant : x20 cases et x5 simulations dans tout le niveau, la recherche tournait encore après 75 min ; après : 17 à 22 min (machine chargée). Sans marée, rien ne change.
- Tests : core `zones` + `levels` + `faisabilite` 122 ok (dont `monde_2_niveau_2_faisable`) ; game `camera::` `contenu::` `plan::` 31 ok ; game hors robots 342 ok (avec et sans `debug_niveaux`, seul `camera::les_niveaux_d_un_ecran_gardent_la_meme_camera` était rouge sur un premier dessin de 14 rangées, d'où les 11). Essai eau 2 : 3 gouttes, 0 mort.
- Monde 2 : `le_robot_finit_le_monde_2_sans_mourir` (niveaux [1, 2], 0 mort, 4 253 images, `684f684fdfd9d719`) ; partie complète (`ecrire_script_monde_2_96`) : 6 gouttes, 0 mort.
- Monde 1 identique (`--exact`, extraction propre) : normale 23 703 images `41a73ef4a0866536`, complète 28 995 `c070c4ae5c42223d`, nouvelle partie + 24 472 `d95fc98f74c6d8ed`. Extraction propre de 8071ddf : `lint` LINT-OK, `./build.sh --qa` et `./build.sh` OK (0 nom interdit, seule URL `http://www.w3.org/2000/svg`).
- Captures (`node docs/capture-96.mjs PORT DOSSIER target/qa/script-96.txt nom:image...`, images 4317, 3117, 2847) : `docs/qa/97-rats.png` (rat dressé « ! » dans le couloir, rat de la galerie, stalactite), `97-maree.png` (barque sur le collecteur plein), `97-goutte-difficile.png` (au fond du bassin à sec, dans la niche). Regardées ; 0 pixel hors palette, 0 flou, 0 erreur de console.
- PAS testé : joueur humain, tactile, vrai téléphone, Safari ; difficulté (AGA juge en jouant) ; musique : repli du thème.

## Carte 9.2b : BAS retourne le parapluie, la barque se prend avec BAS (plus en planant) ; barque à moitié enfoncée

- Fait : `Entrees::bas` (flèche du bas, S ; bouton tactile BAS sous les flèches, montré seulement si le niveau a de l'eau). BAS tenu = `EtatJoueur::Retourne` (au sol et en l'air, aucun effet sur le mouvement ; SAUT tenu l'emporte : on plane). L'eau donne la barque au parapluie RETOURNÉ ; fermé OU ouvert (en planant), on coule ; réapparition protégée toujours recueillie. Sprites `pepin_retourne` / `pepinette_retourne` (planche régénérée). Barque À MOITIÉ ENFONCÉE par le rendu seul (`ENFONCE_BARQUE_RENDU` = 3 px, eau redessinée devant la coque + rides, dans `scene.rs` ; `ENFONCEMENT_BARQUE` inchangé). Textes : aide (ligne « BAS, S », astuce BAS à la place de « aucune touche en plus »), conseil du caniveau, essai eau.
- Robots : faisabilité et QA tiennent BAS à moins de 16 px au-dessus de l'eau, lâchent le saut à 2 px (`faisabilite::eau_sous_les_pieds`). Scripts navigateur : chiffre en BASE 32, bit 16 = BAS (`bot_qa::code_entrees`, `pilote-qa.js` lit `parseInt(c, 32)` ; sans BAS, mêmes chiffres qu'avant).
- Tests : suite complète core 405 ok, game 368 ok (36 ignorés) ; `player` 28, `faisabilite` 25, `levels` 35, `zones` 63 ; game ciblés (entrees, tactile, skins, aide, contenu, carte, sprites) 100 ok ; robots essai eau (415 images, 3 gouttes ; seuil « images en barque » ramené de > 10 à >= 3 : il en fait 5), eau 2, Monde 2 (0 mort, `8b9b432c96c52c88`). `lint` LINT-OK. `./build.sh --qa` ok.
- Monde 1 identique (`--exact`) : normale 23 703 `41a73ef4a0866536`, complète 28 995 `c070c4ae5c42223d`, nouvelle partie + 24 472 `d95fc98f74c6d8ed`.
- Captures (`node docs/capture-92b.mjs PORT docs/qa GRILLES`, puis `python3 docs/gros-plan-92b.py game/src/sprites/pepin.rs docs/qa/96-barque-filet.png docs/qa/92b-barque-bas.png docs/qa/92b-barque-enfoncee.png 3`) : `92b-retourne-en-l-air` (+ `-pepinette`, `retourne-au-sol`), `92b-barque-bas`, `92b-barque-courant`, `92b-plane-au-dessus-de-l-eau`, `92b-noyade-en-planant` (5 vies -> 4), `92b-bouton-bas-tactile` (+ `-tenu`), `92b-barque-enfoncee` (avant/après x8). Regardées ; 0 hors palette, 0 flou, 0 erreur de console, sauf les deux tactiles : 7 050 px hors palette, les boutons TRANSLUCIDES par-dessus l'image (comme aux cartes précédentes).
- À décider : le panneau « planer » avant le premier filet du caniveau dit encore « planer » (pas de panneau BAS ; grille du caniveau laissée à la 9.6b). PAS testé : joueur humain, vrai téléphone, Safari.

## Carte 9.6b : refonte graphique du caniveau (tuiles de rue par thème, eau lisible, fond de rue, décor de rue)

- Fait (rendu seulement, grille de collision inchangée) : tuiles par thème (`game/src/tuiles_theme.rs` choisit d'après le voisinage et la rangée de la rue, dessins `game/src/sprites/tuiles_monde2.rs` sans teinte) : pavés, bordure de trottoir au bord de l'eau, terre sombre, caniveau à sec, maison de pierre (soubassement, fenêtres, plaque de rue, chéneau), pierres de taille, store de boutique sous l'avant-toit. Eau (tous les niveaux) : ligne claire 1 px, bande cyan, fond ardoise ; stries animées dans le sens du courant (`game/src/dessin_courant.rs`), plus de chevrons. Fond (`game/src/decor_rue.rs`) : immeubles lointains (relief 30), rangée de maisons (fenêtres, vitrines sombres sous store, descentes de gouttière), lampadaires à halo, pluie 56 gouttes. Décor : cheminées et antennes retirées ; lettres `E` bouche d'égout, `I` borne, `U` poubelle, `J` descente de gouttière (2 images).
- Tests ciblés : game `decor sprites scene contraste tuiles` 118 ok ; core `levels zones level::` 125 ok (dont `le_decor_ne_change_rien_a_la_collision`) ; `lint` LINT-OK.
- Robots (release, `--exact`) : Monde 1 normale 23 703 `41a73ef4a0866536`, nouvelle partie + 24 472 `d95fc98f74c6d8ed` (identiques) ; Monde 2 `9c8d16321a6d2fc8` (identique, 0 mort).
- Captures (build de QA, Chromium 1208 CDP 960x540, `node docs/capture-96.mjs PORT DOSSIER target/qa/script-96.txt nom:image...`) : `docs/qa/96b-avant|apres-depart, -eau-calme, -barque-filet, -gouttiere-courant, -rue-courant` (+ `96b-apres-maison`), gros plans `96b-avant|apres-eau-calme-zoom`, `-eau-courant-zoom`. Regardées ; 0 pixel hors palette, 0 flou, 0 erreur de console.
- Monde 1 identique au pixel : `MONDE1=1 node docs/capture-96.mjs ... target/qa/script.txt` (script de `ecrire_script_navigateur`), `96b-avant|apres-monde1-toits` (image 600) et `-gouttieres` (3000) : mêmes pixels (empreintes `50512bab6f379651`, `3f8683c6d1b1565f`). Le pilote n'est pas parfaitement répétable en plein saut (Pépin à 1 px près d'un lancement à l'autre, avant comme après) : comparer plusieurs lancements.
- Recette « tuiles du thème » ajoutée dans `pepin-notes/ETAPE-9.md` (point 7) pour 9.7, 9.8, 9.9.
- PAS testé : joueur humain, téléphone, Safari ; les autres thèmes du Monde 2 gardent toits et briques (leurs cartes) ; « panneau » de rue en plaque de façade (pas de lettre : il se confondrait avec les panneaux d'aide).

## Carte 9.6c : tuiles des égouts, de la cascade, de la grotte et du port ; panneau BAS du caniveau

- Fait (rendu seulement, grilles et physique inchangées sauf le panneau) : 4 jeux dans `tuiles_theme.rs` (`egouts`, `cascade`, `grotte`, `port`), dessins dans `game/src/sprites/tuiles_monde2/` (`tuiles_monde2::JEUX`). Égouts : briques gris-bleu, quai de pierre claire sur chaque passage, voûtes en arches aveugles, fond de bassin sombre. Cascade : roche grise, strates violettes (pas de brun dans la palette), coussin de mousse rousse sur les dessus, margelles de pierre au bord de l'eau, galets. Grotte : roche bleu nuit, sol au rebord éclairé, cristaux pâles (nuage, arête cyan) sur les parois seulement. Port : quai de dalles, anneaux d'amarrage, `#` = ponton en planches sur pieux (ou caisse s'il est posé sur un solide), bitte d'amarrage posée au bout d'un quai/ponton (`objet_pose`, sans collision) ; teinte du port 3 -> 1 (l'eau n'est plus délavée). Rivière, marée : `None`. Aucun cyan sur un dessus praticable.
- Panneau du caniveau (colonne 28) : nouvelle lettre `Z` (`GenrePanneau::Barque`, `panneau_barque` : flèche du BAS plongeant dans le parapluie retourné, sans mot), commentaire du niveau corrigé. Plus aucune majuscule libre (noté dans `ETAPE-9.md`).
- Tests : game `sprites scene decor tuiles` 118 ok ; core `levels zones level:: faisabilite` 151 ok ; `lint` LINT-OK ; `./build.sh --qa` OK ; planche `docs/planche-sprites.png` régénérée.
- Robots (release, `--exact`) : Monde 1 normale 23 703 `41a73ef4a0866536`, nouvelle partie + 24 472 `d95fc98f74c6d8ed` (identiques). Monde 2 : `le_robot_finit_le_monde_2_sans_mourir` ROUGE au niveau 17 (la Pieuvre, « aucun parcours ») comme sur monde-2 `893c288` (vérification du coordinateur, carte 9.12c qui le répare) ; empreinte Monde 2 donc non mesurée. Les panneaux ne servent qu'au dessin (`scene::dessiner_panneaux`, seul lecteur).
- Captures (build de QA, Chromium 1208 CDP 960x540, `node docs/capture-96.mjs PORT DOSSIER - t:Enter:300 ... t:KeyN:300 c:nom`, nouveau mode « pilotage à la main ») : `docs/qa/96c-avant|apres-egouts-depart, -egouts-voute, -cascade-depart, -grotte-depart, -grotte-bassin, -port, -caniveau-panneau`. Regardées ; 0 pixel hors palette, 0 flou, 0 erreur de console.
- PAS testé : joueur humain, téléphone, Safari ; empreinte du Monde 2 (attend la 9.12c) ; la bitte d'amarrage n'apparaît que si la case au-dessus du bout de quai est libre.

## Carte 9.12c : la Pieuvre un peu moins dure que le Grand Nuage (moins de noyades, progression gardée)

- Arène (`levels/pieuvre.rs` = `essai_pieuvre.rs`) : un PLATELAGE continu sur piliers, percé de 4 trous d'une case au bord des têtes (on tombe dedans sur la tête sortie) ; on esquive en courant, plus en sautant de ponton en ponton. Gouttes sous le départ (prises en tombant, avant la colère : sinon la recherche du robot et de la mesure explosait, x8 cases, sans aboutir).
- Motifs (`entities/pieuvre.rs`) : 44 / 67 / 74 % du bassin par salve, toutes les 66 / 48 / 30 pas (9.12b : 44/56/67 %, 60/40/30) ; plus gros ET plus vite à chaque point, motif 3 = la vague (trou de 4, 3 puis 2 colonnes). Les chiffres de la carte (35/40/38 %, 66/46/34) ont été MESURÉS : Habitué 0,17, trop facile sur le platelage ; la visée mesurée prime. Test `aussi_dure_que_le_grand_nuage` remplacé par `chaque_point_perdu_frappe_plus_gros_et_plus_vite`.
- Mesure (`mesure_des_deux_boss`, graine officielle, 48 essais), morts moyennes : cyclone 4,06 / 1,19 / 0,38 ; Pieuvre AVANT (893c288, règle BAS) 0,94 / 0,35 / 0,10 ; APRÈS 2,21 / 0,88 / 0,31 (Débutant / Habitué / Expert). Habitué 74 % du cyclone. Causes : 100 % tentacules, 0 noyade (avant : 0 noyade aussi, la règle BAS de la 9.2b les avait déjà supprimées). Game over Débutant 8/48 (cyclone 16), tous finissent 48/48.
- Robot (`bot_qa/parcours.rs`) : repli `parcours_du_boss_par_points` (dernier essai, arène d'un boss seulement) : un point à la fois depuis un monde exact, puis attente de la plongée. Monde 1 non concerné (sa recherche d'un seul tenant réussit).
- Preuves : `test -p core` pieuvre, levels, zones, faisabilite ; `le_robot_bat_la_pieuvre_sans_mourir` (0 mort, 1 481 images) ; `le_robot_finit_le_monde_2_sans_mourir` (niveaux 1 à 5, 0 mort, 9 gouttes, `95e457bdc30fb688`) ; `lint` LINT-OK ; `./build.sh --qa` ok. Captures `docs/qa/912c-arene.png`, `912c-vague-motif-3.png` (`node docs/capture-912.mjs`, pas 80 et 1200) : regardées, 0 hors palette, 0 flou, 0 erreur de console.
- Monde 1 identique (`--exact`, extraction propre) : normale 23 703 `41a73ef4a0866536`, complète 28 995 `c070c4ae5c42223d`, nouvelle partie + 24 472 `d95fc98f74c6d8ed`. PAS testé : joueur humain, tactile, vrai téléphone.

## Carte 9.2c : le parapluie retourné SOUS Pépin, bouton tactile BAS à droite sous le saut

- Fait (dessin et disposition seulement, aucune physique touchée) : `PEPIN_RETOURNE` / `PEPINETTE_RETOURNE` redessinés en 16x16 : le personnage DEBOUT dans la coque de la barque (même coque que `PEPIN_BARQUE` / `PEPINETTE_BARQUE` : bord clair festonné, toile corail ou cyan, baleines, crosse en quille dessous). `scene::poser_pepin` les pose sous les pieds comme la barque (`ENFONCE_BARQUE_RENDU` plus bas en l'air, crosse sur le sol au sol) : en touchant l'eau, la coque reste à la même place et Pépin s'y assied. Planche `docs/planche-sprites.png` régénérée.
- Tactile (`tactile::Disposition::calculer_avec_bas`) : BAS passe en bas À DROITE, sur la colonne du saut, juste dessous, à 85 % de sa taille ; le saut remonte d'autant ; les flèches ne bougent plus (identiques sans et avec BAS). Zones sensibles du saut et de BAS partagées au milieu de leur espace, rognées sous la pause. Sans eau : disposition inchangée (test existant). Tests adaptés (pas de nouveau test) : `les_boutons_restent_dans_l_ecran_et_sont_assez_gros` (BAS sous le saut, même colonne, >= 80 % du saut, à droite des flèches, flèches inchangées) et `les_boutons_ne_se_chevauchent_pas` (aucun recouvrement des dessins ni des zones), écran 568x320 ajouté.
- Tests : `-p game tactile scene sprites` 95 ok ; extraction propre de 040e0b6 : suite complète verte (core 409, game 368, 39 ignorés), `./build.sh` : 0 nom interdit, seule URL `http://www.w3.org/2000/svg` ; `lint` : un seul écart de `cargo fmt` (un `assert!` du test tactile) corrigé, puis LINT-OK sur l'extraction propre de c13796e. Robots (release, `--exact`, extractions propres de 44a7d62 et c13796e) : empreintes IDENTIQUES avant/après : Monde 1 normale 23 703 `41a73ef4a0866536`, nouvelle partie + 24 472 `d95fc98f74c6d8ed`, Monde 2 (niveaux [1..5], 8 490 images, 0 mort) `95e457bdc30fb688` (rendu et tactile seulement, le robot joue au clavier).
- Captures (`./build.sh --qa`, `CHROME=... node docs/capture-92c.mjs PORT docs/qa GRILLES` ; `TACTILE_SEUL=1` pour la partie tactile seule) : `92c-retourne-en-l-air` (+ `-pepinette`), `92c-retourne-au-sol`, `92c-juste-avant-l-eau` puis `92c-a-l-eau` (2 images plus tard, assis dans la barque), dans le caniveau `92c-caniveau-avant-tactile-*`, `92c-boutons-tactiles-*` et `92c-bouton-bas-tenu-*` en 844x390 et 667x375 (BAS enfoncé, Pépin parapluie sous lui). Regardées ; 0 hors palette, 0 flou, 0 erreur de console, sauf les tactiles (boutons TRANSLUCIDES, comme aux cartes précédentes). Debout (`92c-*-390x844`), le jeu affiche « Tourne ton téléphone » : pas de boutons (comportement d'avant).
- À voir : en 667x375 la pause (inchangée) touche le chrono du HUD (déjà ainsi avant) ; le saut est plus haut qu'avant dans un niveau avec eau. PAS testé : joueur humain, vrai téléphone, Safari.

## Carte 9.20e : « La Pieuvre du port » devient le bout d'une jetée (rendu seulement)
- Fait : `game/src/dessin_port.rs` (niveau de clé `pieuvre` seulement ; `essai_pieuvre` garde l'ancien rendu) et 14 sprites `game/src/sprites/jetee.rs`. Les murs `B` des colonnes 0 et 19 restent solides mais se dessinent en QUAIS : pierre du quai alignée sur le platelage, anneau et cordage jusqu'à l'eau, pierre mouillée au bassin ; au-dessus, la tour en treillis d'une grue (pied rayé, cabine) à gauche, une pile de caisses sanglées à droite. Au fond : la mer et son horizon (y 80, au-dessus du platelage), le phare rayé sur son îlot (feu clignotant, reflets or pâle), un chalutier, une barque et un canot en silhouettes sombres ; les mouettes du thème restent. Le bassin (rangée 9) : la bande cyan de surface est gardée (l'ombre et les remous de l'annonce s'y lisent), l'eau dessous passe à l'ardoise de nuit, reflets du phare.
- Grille INCHANGÉE (aucune lettre ajoutée, `l_arene_de_la_pieuvre_reprend_celle_du_niveau_d_essai` vert) : mesure de difficulté passée, Habitué 0,88 pour 1,19 au cyclone (9.12c) par construction. Robot Monde 2 : même empreinte que la 9.12c, `95e457bdc30fb688` (5 niveaux, 0 mort, 9 gouttes).
- Preuves : `test -p core` levels::pieuvre, zones, faisabilite (88 ok) ; game sprites, scene, decor, dessin_port, tuiles_theme (125 ok, dont 7 nouveaux) ; `le_robot_finit_le_monde_2_sans_mourir` et `le_robot_bat_la_pieuvre_sans_mourir` verts ; `lint` LINT-OK ; `./build.sh --qa` ok.
- Captures (Chromium 1208, `docs/capture-920e.sh`, partie du robot `script-912b` rejouée sur le niveau, pas 120, 1062, 1092, 1440) : `docs/qa/920e-avant|apres-depart, -annonce, -milieu, -sortie.png`. 0 hors palette, 0 flou, 0 erreur de console. Regardées : le port saute aux yeux (grue, caisses, phare rayé, chalutier, horizon), l'annonce reste lisible sur l'eau sombre.
- PAS testé : robot du Monde 1 (code de rendu gardé par la clé du niveau, simulation intouchée), joueur humain, tactile, vrai téléphone.

## Carte 9.20a : « Le caniveau » ressemble à son nom (l'eau reste au caniveau, descente de gouttière, sortie par la bouche d'égout)

- Parcours (`levels/caniveau.rs`, 128 -> 134 x 11) : TOUTE l'eau est au ras de la rue (rangée 8), trois bassins qui grossissent (14, 36 puis 20 x 2 cases). La gouttière sur la maison (eau en rangée 5) devient le courant de la rue sous une MARQUISE de boutique (corniche `#`, store `B`) : la 3e goutte y reste « seulement en barque ». Deux descentes de gouttière `J` se vident dans les deux courants ; la bordure du dernier est la BOUCHE D'ÉGOUT, drapeau planté sur sa grille. Mêmes lettres, 3 gouttes, 4 lanternes, aucune mécanique nouvelle.
- Rendu : `tuiles_theme::bouche_d_egout` (la bordure au bout d'un courant de deux cases de fond : grille, gueule où l'eau s'engouffre, gorge noire ; dessins `can_egout_*`) ; descente de gouttière 48x40 (tuyau, colliers, gargouille jusqu'au caniveau, filet d'écume), posée sans centrage (`scene::dessiner_decors`). Intro : « La pluie file dans le caniveau, jusqu'à la bouche d'égout. Pépin la suit. » (astuce BAS gardée).
- Difficulté (`mesure_d_un_niveau_du_monde_2`, nouveau, graine officielle, 48 essais) : AVANT Débutant 0,08 / Habitué 0,00 / Expert 0,00 ; APRÈS 0,08 / 0,00 / 0,00 (même cause : la flaque, col. 11). Habitué identique.
- Tests : core `levels:: zones:: faisabilite lettres` 131 ok (2 nouveaux : l'eau reste au caniveau et grossit, le drapeau est sur la bouche ; la goutte de l'avant-toit devient celle de la marquise) ; game `sprites scene decor tuiles contenu` 128 ok (dont `la_sortie_du_caniveau_est_la_bouche_d_egout`) ; `lint` LINT-OK ; `./build.sh --qa` OK ; planche régénérée (même taille).
- Robots : `le_robot_finit_le_monde_2_sans_mourir` vert (niveaux 1 à 5, 0 mort, `116cdaec5e48ff99`) ; script complet `ecrire_script_monde_2_96` vert (3 gouttes). Monde 1 identique (release, `--exact`) : normale 23 703 `41a73ef4a0866536`, nouvelle partie + 24 472 `d95fc98f74c6d8ed`.
- Captures (build de QA, Chromium 1208, `SANS_BAS=1 node docs/capture-96.mjs PORT docs/qa target/qa/script-96.txt nom:image...` ; `SANS_BAS` nouveau : BAS + Entrée ouvre la carte, plus JOUER) : `920a-avant|apres-depart, -milieu, -sortie` + `920a-apres-gouttiere`. Regardées : 0 hors palette, 0 flou, 0 erreur de console. Le nom saute aux yeux : on suit l'eau le long du trottoir, la descente de gouttière la verse, le drapeau est sur la grille où elle s'engouffre.
- PAS testé : joueur humain, tactile, vrai téléphone, Safari.

## Carte 9.20b : « Les égouts » ressemblent à leur nom (grille du départ, tuyaux des bassins, sortie par le grand tuyau)

- Fait (rendu seulement : grille, entités et physique inchangées, aucune lettre prise) : `game/src/decor_egouts.rs`, appelé par `scene::dessiner` après les tuiles, thème Égouts seul. Départ : grille de la bouche d'égout au plafond juste au-dessus de Pépin, rai orange du lampadaire en biais jusqu'à lui, pluie qui tombe par la grille. Bassins `=` (même marée 9.3, repères gardés) : gros tuyaux de fonte sortis du plafond, qui gargouillent pendant l'annonce puis crachent un jet par à-coups (4 par montée), synchronisés sur `Maree::pas()` ; rien de commun touché pour le futur thème Marée. 4 autres grilles au plafond avec leur filet de pluie (jamais au-dessus d'un bassin ni d'une lanterne). Sortie : les 9 dernières cases sont l'intérieur d'un grand tuyau de béton (joints des anneaux), bouche ronde en anneau, dehors le ciel de nuit, la falaise d'en face et sa chute d'eau, le rebord de rocher. Tuyaux du fond assombris (`decor.rs`, `Detail::Tuyaux`) : plus d'arête claire qui ressemble à un rebord. En-tête de `levels/egouts.rs` et texte `mecanique` sans « marée ». Intro de `contenu.rs` gardée.
- Tests : `decor_egouts` (5, placement pur : grille du départ, grilles au plafond loin des bassins, tuyaux par bassin, jet par à-coups, grand tuyau autour du drapeau). Extraction propre de `8fb7f29` : `test --workspace` core 409 ok, game 373 ok (robots compris) ; `lint` LINT-OK ; `./build.sh` ok (0 mention interdite, seule URL svg) ; `./build.sh --qa` ok.
- Difficulté : grille non touchée, donc pas de nouvelle mesure (Habitué inchangé par construction). Robots (release, `--exact`) : Monde 2 `95e457bdc30fb688` (identique, 0 mort, 9 gouttes) ; Monde 1 normale 23 703 `41a73ef4a0866536`, nouvelle partie + 24 472 `d95fc98f74c6d8ed` (identiques).
- Captures (build de QA, Chromium 1208 CDP 960x540, script de `ecrire_script_monde_2_96`, `SANS_BAS=1 node docs/capture-96.mjs PORT DOSSIER script-96.txt nom:image` ; nouvelle option : le titre actuel n'a plus besoin de BAS pour reprendre au Monde 2) : `docs/qa/920b-avant|apres-depart` (image 2050), `-milieu` (2860, collecteur plein, deux jets), `-sortie` (4110), `920b-apres-bassin`. Regardées : 0 hors palette, 0 flou, 0 erreur de console. Le nom saute aux yeux : on tombe d'une bouche d'égout sous la pluie, des tuyaux remplissent les bassins, on ressort par un grand tuyau au-dessus de la cascade.
- PAS testé : joueur humain, téléphone, Safari. Le mur du bout (colonne 145) est caché sous le ciel du dehors : invisible, mais le drapeau (colonne 140) finit le niveau avant.

## Carte 9.20c : « La grande cascade » ressemble à son nom (grande chute en fond, départ au grand tuyau, bassin d'arrivée qui part en rivière)

- Fait : `game/src/decor_cascade.rs` (thème Cascade seul, sans collision), appelé par `scene::dessiner` derrière les tuiles (le fond) et après les décors (le tuyau, le bouillon). GRANDE CHUTE sur toute la hauteur (colonnes 11 à 21) : rideau d'ardoise et filets qui dévalent, lèvre de roche en haut, embruns tramés aux paliers, falaise du pied de part et d'autre, brume tramée en bas. Les ruisselles du fond (`decor.rs`, `Detail::Chutes`) passent en ardoise : seul le cyan vif des chutes `|` (ses bras, au premier plan) pousse Pépin. DÉPART : le bout du grand tuyau des égouts (voûte, passerelle sous Pépin, rigole dont l'eau se déverse dans le bassin A, bouche en anneau). Intro « L'eau ressort du grand tuyau au bord de la falaise... » (`contenu.rs` et `INFOS.intro`) ; astuce gardée.
- Grille (`levels/cascade.rs`) : voûte du tuyau en roche (rangées 0 et 1), un oiseau de décor en moins (dans le tuyau) ; le BAS refait : bassin d'arrivée large rangée 42 (colonnes 6 à 31), courant `}` vers la droite (la rivière), drapeau sur la berge de droite (29, 40), 3e goutte (12, 40) au-dessus de l'eau. Mêmes gouttes (3), lanternes (3), ennemis, mécaniques ; aucune lettre prise.
- Difficulté (`mesure_d_un_niveau_du_monde_2` de la 9.20a, CLE=cascade, graine officielle, 48 essais) : AVANT Débutant 0,00 / Habitué 0,00 / Expert 0,00 ; APRÈS 0,00 / 0,00 / 0,00 (48/48 finis, 18,1 s contre 18,7 s pour l'Habitué).
- Preuves (extraction propre de `b4ca722`, release) : core `cascade zones faisabilite` 87 ok ; game `sprites scene decor contenu le_robot_finit` 142 ok : Monde 2 niveaux 1 à 5, 0 mort, 9 gouttes, `9ca52833cb203064` (la cascade change de parcours) ; Monde 1 identique (23 703 `41a73ef4a0866536`, nouvelle partie + 24 472 `d95fc98f74c6d8ed`) ; script `ecrire_script_monde_2_96` 0 mort, 15 gouttes ; `lint` LINT-OK ; `./build.sh` et `./build.sh --qa` ok.
- Captures (`SANS_BAS=1 node docs/capture-96.mjs PORT DOSSIER target/qa/script-96.txt nom:image`, Chromium 1208) : `docs/qa/920c-avant|apres-depart, -milieu, -sortie, -plan` (plan : 4 écrans empilés) et `920c-apres-pied`. Regardées : 0 hors palette, 0 flou, 0 erreur de console. Le nom saute aux yeux : un large rideau d'eau tombe derrière tout le niveau, on sort du tuyau des égouts et l'on finit dans le bassin où il s'écrase, qui file en rivière.
- PAS testé : joueur humain, téléphone, Safari. Hors palette 0, mais la lisibilité du rideau sur petit écran n'est vue qu'en 960x540.

## Carte 9.20d : « La grotte aux bulles » ressemble à son nom (derrière la cascade, sources des bulles, sortie au jour)

- Fait : `game/src/decor_grotte.rs` (appelé par `scene::dessiner` après les tuiles, thème Grotte seul). Entrée : le mur gauche ouvert au pied (colonne 0, rangées 38 à 45) sur le RIDEAU D'EAU de la cascade (filets qui dévalent, jour entre eux, écume, gouttes projetées, rais de lumière en pointillés sur le vide seulement). Sources : vasque `~` qui bouillonne au pied de C1 (rangée 46) et de C3 (rangée 26, bout de la corniche de repli) ; fissure lumineuse d'où sortent les bulles pour C2 (sol de P1) et C4 (sommet d'un nouveau pilier de roche, colonnes 28-29, rangées 21 à 26). Sortie : le mur droit ouvert au-dessus du rocher du drapeau (colonne 35, rangées 1 à 3) sur l'aube pâle, la rivière, les herbes ocre (aussi sur le rocher), rais dorés. Le bord de la carte reste un mur (collision inchangée). Bulles du fond « sorties de nulle part » retirées (`decor.rs`, variante `Detail::Bulles` supprimée). Intro (`contenu.rs` et `INFOS.intro`) : « Derrière la cascade, une grotte. Des sources bouillonnent : les bulles te portent. » En-tête de `levels/grotte.rs` et doc de `Theme::Grotte` corrigés. Aucune lettre prise, aucune mécanique nouvelle ; gouttes 3, lanternes 3.
- Difficulté (`mesure_d_un_niveau_du_monde_2`, de la carte 9.20a, défaut `grotte`, graine officielle, 48 essais) : AVANT 0,15 / 0,02 / 0,00 ; APRÈS 0,10 / 0,02 / 0,00 (Débutant / Habitué / Expert), 48/48 finis, Habitué 26,6 s (avant 28,3). Variante essayée avec une vasque aussi sous C2 : Habitué 0,04 (une mort de plus, pointes colonne 8) ; refusée, C2 a une fissure.
- Tests : `decor_grotte` (2 : une source par colonne, vasque dans sa colonne et bulles depuis son fond ; entrée au départ, sortie au drapeau, bord toujours mur) ; `test -p core` 409 ok (dont `levels::grotte`, `zones`, `faisabilite`) ; `test -p game` 370 ok (dont `sprites`, `scene`, `decor`, robots) ; `lint` LINT-OK ; `./build.sh` et `./build.sh --qa` ok.
- Robots (release, `--exact`) : Monde 2 0 mort, 9 gouttes, 8 401 images, `62766093ffc42d39` (la grotte change, attendu) ; Monde 1 identique : normale 23 703 `41a73ef4a0866536`, complète 28 995 `c070c4ae5c42223d`, nouvelle partie + 24 472 `d95fc98f74c6d8ed`.
- Captures (build de QA, Chromium 1208, `SANS_BAS=1 node docs/capture-96.mjs PORT DOSSIER target/qa/script-99.txt nom:image`, script de `ecrire_script_monde_2_99`) : `docs/qa/920d-avant|apres-depart, -milieu, -sortie, -plan` (plan : 4 vues de haut en bas, ffmpeg). Regardées : 0 hors palette, 0 flou, 0 erreur de console. Le nom saute aux yeux : on entre par le rideau de la cascade, chaque colonne de bulles part d'une source qui bouillonne ou d'une fissure, et l'on ressort au jour au-dessus du drapeau.
- PAS testé : joueur humain, téléphone, Safari. Les rais de lumière sont des pointillés (pas de transparence) : à juger par AGA.

## Carte 8.18 : petit nuage gris au sommet de chaque colonne de grêle

- Avant : la grêle tombait de nulle part. Après : un petit nuage gris d'orage (24 x 11, `grele_nuage0`/`grele_nuage1` dans `sprites/objets.rs`) coiffe chaque colonne `g` (mer, poursuite, descente, essai grêle) : centré, son ventre sur le haut de la colonne, les grêlons en sortent ; il flotte d'un pixel (45 images) et ses mèches s'effilochent (2 images, 40 images), chaque colonne avec sa phase ; une colonne qui part du haut de la carte garde son nuage dans la carte. Colonnes accolées : deux nuages qui se chevauchent, lus comme un seul banc.
- DÉCOR seulement : `grele::nuage` (position pure) et `grele::dessiner_nuage`, appelés dans `scene.rs` après les grêlons ; aucune collision, `core` intact (grêle, niveaux, robots, empreintes inchangés).
- Couleurs : contour 0, ventre ardoise 2, corps 3, ardoise pâle 4, liseré de brume 5 en haut ; ni 6 ni blanc ni rose (pas un nuage où l'on se pose). Le liseré le détache des ciels violets et ardoise : contraste >= 3:1 (corps, liseré ou contour) sur chaque bande de ciel de chaque thème ; sans lui, 2,57 sur l'ardoise claire des gouttières.
- Preuves : aucun nouveau test (retouche de dessin), le nuage ajouté à `les_animations_ont_des_images_de_meme_taille` ; suite complète verte (core 380, game 360), puis `sprites` et `grele` (62) ; lint vert ; `./build.sh` (game.wasm 647 564 octets, gzip 259 648 ; `dist/` : 0 nom interdit, aucun `/home`) et `./build.sh --qa` ; planche `docs/planche-sprites.png` régénérée.
- Captures regardées (build de QA sur 127.0.0.1, Chromium 1208 CDP, `docs/capture-visibilite.mjs`, script du robot régénéré décalé d'une image ; essai grêle : début du script du robot puis `script-54` après F2) : `docs/qa/818-mer-double`, `818-mer-haut`, `818-poursuite`, `818-descente`, `818-essai-grele`, `818-essai-plane` (Pépin plane sous le nuage), `818-gros-plan` (x12). 0 pixel hors palette, 0 bloc flou (`mesure-capture.py`, captures du jeu).
- Non testé : vrai téléphone, Safari ; la première colonne de l'épilogue (colonne 2, rangée 16) n'a pas été capturée.

## Carte 9.12d : la Pieuvre plus lente et lisible pour un humain (sous l'orage), accélération par PV gardée
- Fait (`core/src/entities/pieuvre.rs` seul, grille et rendu jetée 9.20e intacts) : annonce 48 → 72 pas (1,2 s), « touchée » 60 → 90 pas ; écarts entre salves 66/48/30 → 100/78/60 pas ; motifs de 4, 5, 6 salves (5, 6, 8 avant) ; vague du motif 3 : aller, souffle de 140 pas, retour. Progression gardée : 44 %, 67 %, 78 % du bassin par salve, de plus en plus vite (`chaque_point_perdu_frappe_plus_gros_et_plus_vite`).
- Critère humain : nouveau test `un_humain_a_le_temps_de_changer_de_colonne` (depuis toute colonne sûre de la salve d'avant, au moins 36 pas entre le début de l'annonce et le dernier départ possible à 1,6 px/pas vers la colonne sûre la meilleure ; pire marge 37). Avant : 5 pas au motif 1, négatif au motif 3. `chaque_motif_se_traverse` vert (touchée de 90 pas).
- Mesure (graine officielle, 48 essais), morts Débutant / Habitué / Expert (game over Débutant) : `mesure_d_un_niveau_du_monde_2` CLE=pieuvre AVANT 3,40 / 0,73 / 0,29 (12/48), APRÈS 1,60 / 0,54 / 0,42 (2/48) ; `mesure_des_deux_boss` AVANT 2,21 / 0,88 / 0,31 (8/48), APRÈS 1,73 / 0,56 / 0,17 (5/48). En regard : orage 3,5 / 0,8 / 0,4 (13/48, `docs/mesure-difficulte.txt`), cyclone 4,06 / 1,19 / 0,38 (16/48). L'Expert CLE=pieuvre monte de 0,29 à 0,42 (bruit : 0,31 → 0,17 aux deux boss). Durée Habitué 27 → 35 s.
- Preuves : core `pieuvre` 5 ok (dont `levels::pieuvre`), core complet 412 ok (`durees_plausibles` compris) ; robots `le_robot_bat_la_pieuvre_sans_mourir` (0 mort, 3 gouttes, 1 879 images) et `le_robot_finit_le_monde_2_sans_mourir` (5 niveaux, 0 mort, 8 gouttes comme la base, `47a3f914b1b5f052`) ; Monde 1 identique (23 703 `41a73ef4a0866536`, avant et après) ; lint LINT-OK ; `./build.sh --qa` ok. Mode facile (2 PV) inchangé.
- Capture `docs/qa/912d-annonce.png` (`capture-912.mjs`, pas 1336 : motif 3, 7 colonnes annoncées, trou au milieu) : regardée, 0 hors palette, 0 flou, 0 erreur console. PAS testé : joueur humain, téléphone, Safari.

## Carte 9.2d : au sol, le parapluie retourné assied Pépin (on ne bouge avec BAS que sur l'eau)

- Fait (AGA : « quand le parapluie soit sous ses fesses on puisse se déplacer uniquement sur l'eau ») : `player.rs` pas 4, AU SOL avec BAS tenu, gauche et droite sont sans effet ; il freine avec la friction du sol (pas d'arrêt sec), reste assis, peut toujours sauter. En l'air, BAS garde le contrôle aérien (pour viser l'eau) ; en barque, rien ne change. Rendu au sol : le dessin de la barque (assis dans la coque), crosse sur le sol, Pépin et Pépinette. Robots (`faisabilite`, `bot_qa::parcours`) : BAS jamais tenu pieds au sol (`MARGE_BAS` seulement en l'air). Textes : « Comment jouer » (commande et astuce) et l'astuce du caniveau disent barque sur l'eau, immobile au sol.
- Test : `au_sol_le_parapluie_retourne_ne_bouge_plus` (lancé à 96 px/s, BAS + droite : 1er pas freiné de la seule friction, arrêt en 8 pas au plus, glisse au plus 5,1 + 2 px, puis plus un pixel en 60 pas ; saut et contrôle aérien ok ; BAS lâché, il recourt). Le pas 6 du test de la barque ne tient plus BAS qu'en l'air.
- Preuves : core `player` 29 ok, `faisabilite` 25 ok ; game `sprites scene contenu aide` 95 ok ; robots (release) 15 ok : Monde 1 IDENTIQUE (23 703 `41a73ef4a0866536`, nouvelle partie + 24 472 `d95fc98f74c6d8ed`) ; Monde 2 niveaux [1..5], 0 mort, 8 514 images, 8 gouttes (9 avant), `7e08ff40c6818983` (attendu : le robot ne tient plus BAS au sol au bord de l'eau) ; `lint` LINT-OK ; `./build.sh --qa` ok.
- Captures (`CHROME=... node docs/capture-92d.mjs PORT docs/qa GRILLES`, niveau d'essai « eau ») : `92d-assis-au-sol-pepin|pepinette` puis `-60-images-plus-tard` (droite tenue : au même pixel, seule la pluie change), `92d-bas-lache-*-court`, `92d-barque-sur-l-eau`. Regardées ; 0 hors palette, 0 flou, 0 erreur de console.
- PAS testé : joueur humain, tactile, vrai téléphone, Safari. Grilles de niveau non touchées : le coordinateur revérifie les niveaux refaits en parallèle (9.20f, 9.21a, 9.21b, 9.12d) après fusion.

## Carte 9.20f : le caniveau VRAIMENT refait (rue de nuit en pente, mobilier de rue comme plateformes, barque en trois temps)

- Niveau refait (`levels/caniveau.rs`, 134 -> 188 x 11) : rue de nuit qui descend par deux marches ; voitures, deux autobus, abribus, poubelles, bancs à sauter, trois auvents qui relancent ; barque en 3 temps : filet calme et panneau BAS, passage SOUS un autobus garé (une rangée au-dessus de l'eau, toit hors de portée : seulement en barque), long ruisseau `}` de deux cases de fond jusqu'à la bouche d'égout (drapeau sur sa grille). Gouttes : au-dessus d'une flaque, sous la marquise (en barque), sur le toit de l'autobus (depuis l'abribus). 3 flaques, 1 rat hors de la barque, 4 lanternes.
- Core (nouvelle lettre = nouvelle règle, 1 test `le_mobilier_de_rue_est_solide_et_groupe_par_objet`) : mobilier de rue SOLIDE `q` voiture, `a` autobus, `t` toit d'abribus, `d` poubelle, `l` banc (`level::Mobilier`, groupes de cases voisines) ; lettres notées dans `ETAPE-9.md`. Rendu : `game/src/mobilier_rue.rs` (rectangles de la palette, dessus clair praticable, aucun cyan) ; `tuiles_theme::caniveau` ne dépend plus de la rangée du départ (la rue descend : `B` sol, `#` bâti), bouche d'égout cherchée à la hauteur de son eau.
- Difficulté (`mesure_d_un_niveau_du_monde_2`, graine officielle, 48 essais) : AVANT Débutant 0,08 / Habitué 0,00 / Expert 0,00, Habitué 25,4 s ; APRÈS 0,19 / 0,06 / 0,04, Habitué 40,8 s (Débutant 44,6 s). Repère Gouttières (même outil, assert levé sur une copie jetable) : Habitué 0,56, 47,1 s. Habitué > 0 et sous Gouttières.
- Preuves (extraction propre de `0ae7b99`, release ; le commit suivant ne change qu'un test du niveau, repassé) : suite complète verte (core 412, game 383) ; Monde 1 identique : normale 23 703 `41a73ef4a0866536`, complète 28 995 `c070c4ae5c42223d`, nouvelle partie + 24 472 `d95fc98f74c6d8ed` ; Monde 2 0 mort, 7 gouttes, `6ae37549208fc119` ; `ecrire_script_monde_2_96` 0 mort, 15 gouttes. `lint` LINT-OK (après un `needless_range_loop` corrigé) ; `./build.sh` : 0 nom interdit, aucun `/home`, seule URL `http://www.w3.org/2000/svg` ; `./build.sh --qa` OK.
- Captures (Chromium 1208 CDP, `SANS_BAS=1 node docs/capture-96.mjs`) : `920f-avant-{depart,bas,passage,glissade,bouche,ensemble}`, `920f-apres-{depart,bas,passage,toit-autobus,auvents,autobus-2,glissade,marquise,bouche,ensemble}` (vue d'ensemble : bande des écrans en 320x180). 0 hors palette, 0 flou, 0 erreur de console. On voit une vraie rue : trottoir et caniveau sous la pluie, voitures garées, bus à l'arrêt avec son abribus, boutiques, bouche d'égout.
- PAS testé : joueur humain, tactile, vrai téléphone, Safari ; le saut de l'abribus au toit de l'autobus n'a été fait que par les robots.

## Carte 9.2e : sortir de l'eau en barque sans sauter garde le bateau (grande cascade)

- Fait (AGA : « quand on sort de l'eau sans sauter comme pour la grande cascade, ça nous enlève pas le bateau ») : `player.rs`, nouveau champ `bateau_herite`. Quand la barque quitte l'eau SANS saut (le courant l'emporte par-dessus le bord, pas 3bis), Pépin garde le parapluie retourné (`EtatJoueur::Retourne`) pendant toute la chute, même BAS lâché, et redevient barque en touchant l'eau plus bas (pas de noyade). Le bateau hérité s'arrête au sol (retourné seulement si BAS est tenu, règle 9.2d), en planant (saut tenu) ou sur un rebond. Sauter hors de la barque ne donne PAS de bateau hérité (règle 9.2b inchangée : sans BAS, on coule). Faisabilité : une barque partie à bord qui retombe dans l'eau est un nouveau point de décision (`Fin::Barque`). Commentaire de la grande cascade mis à jour (chute C1 : planer OU se laisser emporter en barque).
- Test (un seul, nouvelle mécanique) : `sortir_de_l_eau_sans_sauter_garde_le_bateau` (bassin haut à courant au-dessus d'un bassin bas : aucune touche, chute retournée, barque en bas, jamais noyé ; posé au sol : état normal ; sorti d'un saut sans BAS : noyade).
- Preuves : core `player` 30 ok, `faisabilite` 25 ok. Robots (release, `--exact`, avant 0bffb19 et après, en parallèle) : empreintes IDENTIQUES, Monde 1 23 703 images `41a73ef4a0866536`, Monde 2 niveaux [1..5] 8 912 images, 0 mort, 8 gouttes `789a8352df16f9a6` (le robot plane dans les chutes, il ne passe jamais par le bateau hérité).
- Captures (build de QA, Chromium 1208 CDP 960x540, `node docs/capture-96.mjs PORT DOSSIER - t:Enter:300 t:KeyN:300 t:KeyN:300 m:4:10 m:12:40 m:20:40 m:4:200`, puis droite seule tenue, aucune touche BAS) : `docs/qa/92e-barque-au-bord-de-la-chute`, `92e-chute-avec-le-bateau` (dans la chute C1, parapluie retourné sous lui), `92e-retombe-en-barque` (bassin B, 5 vies). Regardées ; 0 hors palette, 0 flou, 0 erreur de console.
- PAS testé : joueur humain, tactile, vrai téléphone, Safari.

## Carte 9.20g : caniveau, distinguer les obstacles du fond (rendu seul)

- Fait (AGA : « c'est dur de savoir qu'est-ce qu'est un obstacle comparé au fond ») : FOND sombre et terne (`decor_rue.rs` : façades ardoise et ardoise nuit, plus de violet ni de contour, fenêtres et lampadaires violet d'orage au lieu d'orange, toit en pente, plus de corniche, d'appui ni de faîte clairs ; fenêtres lointaines violettes ; poubelle `U`, borne `I` et gouttière `J` sans collision en ardoise sombre ; abribus : montants, vitre et affiche ternes). PREMIER PLAN : un seul liseré clair `LISERE` (nuage) sous le contour de tout dessus praticable (voitures, autobus, abribus, poubelle, banc, pavés, bordures, marquise), poubelle et banc cernés tout autour. Grilles et physique intactes, Monde 1 non touché (thème Caniveau seul).
- Tests (2, `mobilier_rue`) : `les_dessus_de_la_rue_ont_le_lisere_du_mobilier`, `le_decor_sans_collision_de_la_rue_est_terne`. Preuves : `test -p game --no-default-features -- scene sprites decor mobilier tuiles_theme` 128 ok ; `lint` LINT-OK ; `lourd.sh ./build.sh --qa` OK.
- Captures (Chromium 1208 CDP, script `ecrire_script_monde_2_96`, `SANS_BAS=1 node docs/capture-96.mjs`) : `920g-{avant,apres}-{depart,autobus,abribus,autobus-2,glissade,bouche}` ; test de lisibilité `920g-lisibilite-*` (build de QA, touche O : les cases SOLIDES de la grille repassées en or une rangée sur deux, `TOUCHES='{"270":"KeyO"}'`) : l'or tombe pile sur voitures, autobus, toit d'abribus, poubelle, bancs, rue, marquise ; rien sur les façades. Regardées ; 0 hors palette, 0 flou, 0 erreur de console.
- PAS testé : joueur humain, tactile, vrai téléphone, Safari. Roues de l'autobus dans l'eau : dessinées sous l'eau comme à la 9.20f (non solides).

## Carte 9.21a : « Les égouts » plus longs (146 -> 239 colonnes)

- Fait (`core/src/levels/egouts.rs`, en-tête à jour) : trois sections ajoutées, mécaniques et ennemis existants seulement, aucune lettre de grille nouvelle. GRAND COLLECTEUR (colonnes 76 à 120) : salle haute, deux bassins à marée en escalier de 14 cases (trop larges pour un seul plané), palier à lanterne entre les deux. CANAL prolongé (jusqu'à 164) : un poisson `i` saute du canal (147), puis une seconde voûte basse (153 à 162) à passer en barque. GALERIE DES RATS (169 à 192) : deux couloirs bas, une marche plus bas puis plus haut, un rat chacun. Départ, gouttes (3, mêmes sections), sortie au grand tuyau inchangés ; 5 lanternes (+1) ; `duree_cible_s` 55 -> 65. `decor_egouts` : test des tuyaux adapté (4 bassins).
- Mesure (`mesure_d_un_niveau_du_monde_2`, CLE=egouts, graine officielle, 48 essais), morts / durée, AVANT -> APRÈS : Débutant 0,21 / 33,6 s -> 0,31 / 60,6 s ; Habitué 0,08 / 30,8 s -> 0,12 / 56,8 s ; Expert 0,02 / 28,9 s -> 0,00 / 54,5 s. Habitué +0,04 (orage 0,8).
- Outil : avant même cette carte, la mesure des égouts paniquait (« drapeau inatteignable » : 20 000 cases ne suffisent pas près des marées). Elle seule passe à 1 000 000 de cases (`LIMITE_CASES_MONDE_2` ; 167 351 cases avant, 452 489 après, 85 min de calcul) ; `docker-cargo.sh` transmet enfin CLE. Mesure complète et fantômes inchangés.
- Robots (release) : `zones` + `faisabilite` 87 ok (course rapide 41,6 s, maladroit compris) ; `decor_egouts` 5 ok ; `le_robot_finit_le_monde_2_sans_mourir` ok (0 mort, 8 gouttes, `c89d2e6e43c28268`, 44 min) ; Monde 1 identique (23 703 images, `41a73ef4a0866536`). `lint` LINT-OK ; `./build.sh --qa` ok. Première version (galerie juste après le départ) : robot abandonné après 75 min, galerie déplacée après le canal.
- Captures (build de QA, Chromium 1208, `SANS_BAS=1 node docs/capture-96.mjs`, script du robot) : `921a-grand-collecteur` (Pépin au fond du bassin A vide, tuyaux), `921a-canal-poisson-voute` (barque sous la seconde voûte, poisson derrière), `921a-galerie-des-rats` (un rat étourdi, l'autre sur la marche) ; 0 hors palette, 0 flou, 0 erreur de console. `921a-vue-d-ensemble` : PLAN schématique de la grille (une case = un carré de couleur, deux bandes de 120 colonnes), pas une image du jeu. Regardées.
- PAS testé : joueur humain, téléphone, Safari.

## Carte 9.21b : « La grande cascade » plus longue (32 x 94, onze bassins, 50 à 70 s)

- Fait (`levels/cascade.rs`, sans mécanique ni lettre nouvelle) : le haut (tuyau, bassins A, B, C, puits) gardé, puis cinq sections : SALLE DES BRAS (deux bras `|` traversés en planant de corniche en corniche, mouette sur K1, bassin E dessous), GALERIE EN BARQUE (bassin F calme puis courant, poisson), BASSIN DU CRABE (descente de chute, crabe sous une stalactite), MARCHES DE LA CASCADE (trois bassins en escalier reliés par deux chutes, la barque passe par-dessus bord), MARE AUX NÉNUPHARS (cinq nénuphars, grenouille) ; le pied 9.20c gardé (drapeau, rivière). 3 gouttes, 6 lanternes (+3), `duree_cible_s` 60. `decor_cascade.rs` : la falaise du pied et les paliers d'écume suivent la nouvelle hauteur. `zones::durees_plausibles` : course rapide du chapitre 2 permise jusqu'à 60 s (52,0 s ici).
- Mesure (`mesure_d_un_niveau_du_monde_2`, CLE=cascade, graine officielle, 48 essais), morts / durée : Débutant 0,00 / 19,3 s → 0,15 / 58,0 s ; Habitué 0,00 / 18,7 s → 0,08 / 51,6 s ; Expert 0,00 / 18,2 s → 0,00 / 48,7 s. 48/48 finis partout, Habitué +0,08 (sous l'orage 0,8).
- Leçon de la mesure : aucun ennemi d'eau au-dessus d'une eau CALME (l'Habitué, arrêté là, ne retrouvait plus l'heure du poisson : 3 essais sur 48 bloqués 6 min) ; le poisson de la galerie est au-dessus du courant.
- Preuves (extraction propre de `b77fca5`, release) : core `levels::cascade zones faisabilite` 87 ok ; game `sprites scene decor contenu le_robot_finit` 149 ok : Monde 2 niveaux 1 à 5, 0 mort, `35b3dd8277195e78` ; Monde 1 identique (`41a73ef4a0866536`, nouvelle partie + `d95fc98f74c6d8ed`) ; `ecrire_script_monde_2_96` 0 mort, 15 gouttes ; `./build.sh` (0 nom interdit) et `./build.sh --qa` ok ; `lint` LINT-OK sur l'extraction de `7723dd3` (en-tête réindenté).
- Captures (`SANS_BAS=1 node docs/capture-96.mjs`, Chromium 1208, une image toutes les 20) : `docs/qa/921b-bras, -galerie, -crabe, -marches, -mare` et `921b-plan` (8 écrans empilés, réduits de moitié). Regardées : 0 hors palette, 0 flou, 0 erreur de console, Pépin sans mort jusqu'au drapeau. Une 1re série, rejouée par longs sauts d'images, s'était désynchronisée (morts aux marches) : la même partie rejouée par pas de 20 suit la trace du robot.
- PAS testé : joueur humain, téléphone, Safari ; la règle 9.2d (barque gardée en tombant) n'est pas dans cette branche : les parcours passent sans elle.

## Carte 9.21c : « La grotte aux bulles » plus longue (36 x 48 -> 36 x 80)

- Fait (`core/src/levels/grotte.rs`, en-tête à jour) : deux sections ajoutées sous la grotte 9.20d (inchangée, mêmes coordonnées), mécaniques et ennemis existants, aucune lettre nouvelle. LAC SOUS LA CASCADE (rangées 61 à 79) : entrée par le rideau, nénuphar, barque obligatoire sous une voûte pendue (test `le_lac_se_traverse_en_barque`), rive et lanterne, colonne CA (vasque) de 22 rangées. FOSSE DU CRABE (48 à 60) : trois stalactites en série sous un surplomb, fosse de 14 cases où la colonne de la vasque du fond porte au-dessus du crabe, goutte au fond, lanterne, C0 (vasque contre le mur) qui perce le sol de l'ancienne salle. 7 colonnes, 7 sources (`decor_grotte` : test à 7 colonnes, 5 vasques) ; 3 gouttes ; 5 lanternes (+2) ; `duree_cible_s` 55 -> 60 (plafond du test `durees_plausibles`).
- Mesure (`mesure_d_un_niveau_du_monde_2`, CLE=grotte, 48 essais), morts / durée, AVANT -> APRÈS : Débutant 0,10 / 34,4 s -> 0,10 / 79,7 s ; Habitué 0,02 / 26,6 s -> 0,04 / 66,9 s ; Expert 0,00 / 27,0 s -> 0,00 / 51,3 s. Habitué +0,02 (orage 0,8). Course rapide 36,8 s (plafond 45).
- 1re version écartée (4 sections, 86 rangées, avec un pont de nénuphars et deux poissons) : Habitué 102 s, Débutant 137 s ; la barque est lente et poissons et nénuphars font attendre. Leçon notée dans ETAPE-9.md.
- Robots (release) : `levels::grotte` 2, `zones` + `faisabilite` 89 ok ; `decor_grotte` + `camera` 21 ok ; `le_robot_finit_le_monde_2_sans_mourir` ok (0 mort, 7 gouttes, `05c914d51c3d7aa3`) ; Monde 1 identique (23 703 images, `41a73ef4a0866536`). `lint` LINT-OK ; `./build.sh --qa` ok.
- Captures (build de QA, Chromium 1208, `docs/capture-96.mjs -` : JOUER, puis N x3 jusqu'à la grotte, puis le parcours du robot de la grotte seule, test temporaire non gardé) : `921c-lac-voute` (barque sous la voûte), `921c-galerie-stalactite` (une stalactite tombe derrière Pépin), `921c-fosse-bulles` (Pépin porté au-dessus du crabe) ; 0 hors palette, 0 flou, 0 erreur de console. `921c-vue-d-ensemble` : PLAN schématique de la grille (trait jaune : début de l'ajout), pas une image du jeu. Regardées.
- PAS testé : joueur humain, téléphone, Safari.

## Carte 9.20h : caniveau, les gouttières rattachées aux maisons (rendu seul)

- Fait (`decor_rue.rs`, `dessiner_gouttieres` / `dessiner_filets`) : chaque `J` a son pan de maison (48 px, deux étages, mansarde en pente, mêmes tons sombres que le fond 9.20g), un chéneau mince au bord du toit, un coude, le tuyau contre le mur et 3 colliers, le dauphin coudé vers la rue au-dessus de la 1re case d'eau ; le filet d'écume tombe jusqu'à la surface, éclaboussures cyan. Sprites `gouttiere0/1` retirés ; grille et physique intactes.
- Preuves : `test -p game --no-default-features -- scene sprites decor` 128 ok (dont 2 nouveaux : `la_gouttiere_est_accrochee_a_sa_maison`, `la_gouttiere_et_sa_maison_sont_du_fond_terne`) ; `lint` LINT-OK ; `lourd.sh ./build.sh --qa` ok. Captures (Chromium 1208, script `ecrire_script_monde_2_96` régénéré : 0 mort) `920h-{avant,apres,lisibilite}-{depart,voiture,bas}` = les 3 gouttières, du toit au caniveau ; touche O : aucun or sur pan ni tuyau ; 0 hors palette, 0 flou, 0 erreur ; regardées.
- PAS testé : joueur humain, téléphone, Safari. Le pan est dans le niveau (pas en parallaxe) : il glisse devant les maisons du fond.

## Carte 9.22a : « Le caniveau », des ennemis à des endroits intéressants (grille seule)

- Fait (`levels/caniveau.rs`, 3 lettres, rien d'autre) : un RAT au bout de la 1re rue (col. 26, vu depuis le toit de la 2e voiture : on le saute ou on l'écrase), une FLAQUE qui garde le toit de l'abribus (col. 53, sur le chemin de la goutte de l'autobus), un RAT sur le trottoir entre deux voitures (col. 120, rebond d'un toit à l'autre). Passage de la barque sous l'autobus inchangé, sans ennemi ; aucun sur une lanterne ni au départ.
- Mesure (`mesure_d_un_niveau_du_monde_2`, CLE=caniveau, 48 essais), morts / durée, AVANT -> APRÈS : Débutant 0,21 / 44,8 s -> 0,23 / 45,7 s ; Habitué 0,08 / 41,8 s -> 0,12 / 43,0 s ; Expert 0,04 -> 0,04. Habitué sous 0,20 (cible) et sous la Pieuvre (0,54).
- Preuves : core `levels::caniveau` 5 ok, `monde_2_niveau_1_faisable` ok, `lanternes_sur_le_trajet_d_un_danger` ok ; `le_robot_finit_le_monde_2_sans_mourir` ok (0 mort, 8 gouttes, `d20b12aa24498269`) ; `lint` LINT-OK. Captures (build de QA, Chromium 1208, `SANS_BAS=1 node docs/capture-96.mjs`, parcours complet du caniveau, test temporaire non gardé) : `docs/qa/922a-rat-du-trottoir`, `922a-flaque-de-l-abribus`, `922a-rat-entre-les-voitures` (Pépin l'écrase en sautant du toit) ; 0 hors palette, 0 flou, 0 erreur de console ; regardées.

## Carte 9.12e : arène de la Pieuvre, plus de petit ponton au-dessus d'une tête (AGA : « enlever les petites plateformes au dessus des endroit où se positionnait la pieuvre »)
- Fait : `levels/pieuvre.rs` et `essai_pieuvre.rs` (même grille, le test l'exige), repartis de monde-2, ligne 6 `B####.##....##.####B` : seul le ponton flottant 9-10 (au-dessus de la tête du milieu, tuiles 8,5 à 11,5) est retiré ; pontons sur piliers 6-7 et 12-13 et les deux grands gardés (têtes gauche/droite sous les grands). Mesure 1 : Habitué 0,44, donc salves ÉLARGIES (`entities/pieuvre.rs`, `MOTIFS`) : +1 colonne par salve aux motifs 1 et 2, 4 salves sur 6 au motif 3 (à 8 colonnes, l'aller de la vague n'a plus d'issue à temps) ; part frappée 56/78/85 % (44/67/78) ; débuts, annonce, frappe, PV inchangés.
- Preuves : core `pieuvre` 5 ok (colonne libre du test des règles : 0 au lieu de -1), `monde_2_niveau_5_faisable` ok ; mesure CLE=pieuvre, morts Débutant / Habitué / Expert : 1,21 / 0,44 / 0,04 après la grille, puis 3,35 / 0,96 / 0,46 avec les salves (cible Habitué 0,8-1,0, sous le cyclone 1,19) ; robot `le_robot_bat_la_pieuvre_sans_mourir` ok (1 900 images) ; lint LINT-OK ; capture `docs/qa/912e-arene.png` (vrai niveau par la carte, `capture-96.mjs -`, 0 hors palette, 0 flou, 0 erreur, regardée : pas de ponton dessiné au milieu). PAS testé : joueur humain, téléphone, Safari ; Débutant 3,35 (12/48 game over) à surveiller.

## Carte 9.22e : « Le caniveau » vers Habitué 0,50-0,60 (grille seule)

- Fait (`levels/caniveau.rs`, 7 lettres) : un rat au bord du filet à la sortie de l'abribus (col. 56) ; flaque (col. 64) et pigeon (col. 72) sur les toits des deux voitures 63-74, un rat dans le saut entre elles (col. 68) ; un rat avant les bancs (col. 95) ; flaque (col. 116) et pigeon (col. 125) sur les toits des voitures 115-127. Total : 6 flaques, 6 rats, 2 pigeons ; barque sous l'autobus calme, rien sur une lanterne ni au départ. Test existant : limite des rats 3 -> 6.
- Preuves : core `levels::caniveau` 5 ok, `monde_2_niveau_1_faisable` ok, `lanternes_sur_le_trajet_d_un_danger` ok ; `lint` LINT-OK. Mesure CLE=caniveau SAUTÉE (encore en recherche d'itinéraire après 20 min, au-delà du `timeout 300` d'AGA) : Habitué NON mesuré. Aucun robot, aucune capture (décision d'AGA : il teste lui-même).
- PAS testé : partie humaine, téléphone ; la difficulté réelle reste à juger par AGA sur le site de test.

## Carte 9.22b : « Les égouts », des ennemis aux endroits intéressants (grille seule)

- Fait (`core/src/levels/egouts.rs`, en-tête à jour) : 7 ennemis existants de plus, aucune lettre ni mécanique nouvelle. Six FLAQUES `f` : couloir du premier rat (14), quai entre les deux premiers bassins (42 à 52), marche d'élan du bassin A (76 à 79), corniche de sortie du bassin B pendant la marée (115 à 119), galerie des rats avec le rat 175 (178), couloir sous la galerie haute, sous la goutte du plafond (208) ; un 2e POISSON `i` dans le canal (140), à passer en barque. Aucun sur une lanterne ni au départ ; 3 gouttes.
- Leçon : une 1re version avec rats et grenouilles faisait exploser la recherche du robot (un bit par ennemi d'eau ÉCRASÉ dans sa case, `ennemis_d_eau_de_la_case`) : mesure arrêtée après 2 h 30. Les flaques n'y entrent pas. Même sans ajout, la recherche des égouts était déjà la plus lourde (452 489 cases, 85 min, carte 9.21a).
- Preuves : `test -p core` (`levels::egouts`, `monde_2_niveau_2_faisable`, `lanternes_sur_le_trajet_d_un_danger` inchangé, décors, durées) 11 ok ; `lint` LINT-OK. Mesure CLE=egouts SAUTÉE (décision d'AGA : mesure facultative sous `timeout 300` ; elle dépassait 39 min) : Habitué non mesuré (avant : 0,12). Ni robot, ni captures (décision d'AGA : il teste lui-même).
- PAS testé : joueur humain, téléphone, Safari ; difficulté réelle non mesurée.

## Carte 9.22c : « La grande cascade », 5 ennemis existants placés (grille seule)
- Ajouts (`core/src/levels/cascade.rs`, en-tête à jour) : poisson du bassin A (21, 7) à passer en barque entre deux sauts ; mouette au bord de K2 (5, 40) sur l'arrivée en plané ; poisson de la 3e marche (25, 74) avant la bascule ; grenouille sur le nénuphar du milieu (15, 80), à écraser ou à laisser bondir ; mouette au bout de la corniche d'arrivée (5, 87), qu'on écrase pour rebondir vers la goutte. Aucun sur une lanterne ni au départ.
- Mesure `CLE=cascade` (48 essais), AVANT -> APRÈS : Débutant 0,23 / 58,5 s -> 0,21 / 57,0 s ; Habitué 0,04 / 51,3 s -> 0,25 / 52,6 s ; Expert 0,00 -> 0,23 (morts surtout col. 18-26 : poissons et mouette de K2). Cases explorées 38 449 -> 295 352 (calcul 213 s -> 2 228 s). Preuves : `levels::cascade` + `monde_2_niveau_3_faisable` + lanternes/reprise/poches 9 ok ; `le_robot_finit_le_monde_2_sans_mourir` ok (0 mort, 13 847 images, `4411b5588df5c497`) ; `lint` LINT-OK.
- Captures (build de QA, Chromium 1208, `SANS_BAS=1 node docs/capture-96.mjs`, script du robot régénéré par un test temporaire non gardé) : `922c-poisson-bassin-a`, `-mouette-k2`, `-poisson-marche-3`, `-grenouille-nenuphar`, `-mouette-arrivee` ; 0 hors palette, 0 flou, 0 erreur de console ; regardées. PAS testé : joueur humain.

## Carte 9.23 : tri des tests (AGA : « ça allait plus vite au début du jeu, on a ajouté trop de tests »)

- Mesure (`docs/temps-des-tests.txt`, avant et après, chaque test chronométré par libtest) : AVANT, `test --release` core 14 min 59 s + game 41 min 23 s ; 79 tests de plus de 2 s font 99,9 % du temps (robots, faisabilité). APRÈS : `lourd.sh ./docker-cargo.sh lot` 2 min 01 s sur une extraction propre de `06b9a60` (mémoire des parcours pleine), 34 min 14 s à mémoire vide ; `lourd.sh ./docker-cargo.sh carte <clé>` (sans robot : décision d'AGA du 18/11, aucune carte ne lance de robot) : caniveau 17 s (recompilation comprise), égouts 14 s, clocher 19 s, marché 58 s ; tests rapides seuls : core 373 en 2 s, game 368 en 1 s.
- Supprimés : `zones::durees_plausibles` (refaisait la recherche complète de chaque niveau : ses vérifications passent dans `niveau_<n>_faisable`, même recherche) ; la recherche de `grotte::le_lac_se_traverse_en_barque` (doublon de `monde_2_niveau_4_faisable`, qui exige toutes les lanternes ; ses vérifications de grille restent) ; dans `chaque_variante_ng_plus_est_finissable`, les recherches des niveaux que la nouvelle partie + ne change pas (Monde 2, déjà faits par `zones`). `reprise_depuis_chaque_lanterne` (13 min d'un seul tenant) devient un test par niveau. Les 2 tests `app` rouges en build QA : déjà réparés à la carte 9.6.
- En lot (`#[ignore = "lot : ..."]`, 62 core + 19 game, tous ceux de plus de 5 s) : faisabilité et reprise de chaque niveau, « le niveau exige sa mécanique » (marché, clocher, essais), robots de QA (Monde 1, Monde 2, nouvelle partie +, partie complète, arc-en-ciel, Pieuvre, Grand Nuage, horloge, essais), joueur simulé du niveau 1. Garde-fous : faisabilité avec 3 gouttes (lot, et `carte` pour son niveau), robots des deux mondes et Pieuvre (lot), nom interdit (`build.sh`), physique et difficulté (tests rapides, inchangés).
- Plus rapides : recherche de faisabilité arrêtée dès l'arrivée quand seule l'arrivée compte (`Options::arrivee_seule`, même réponse) ; parcours du robot cherché une fois par processus (`OnceLock` : clocher, arc-en-ciel, Grand Nuage, horloge servaient deux tests) et gardé sur disque par empreinte (code de la simulation et du robot + données du niveau), rejoué avant usage ; profil `tests-rapides` (optimisation 3 sans LTO, au lieu de `z` + LTO de `--release`). Aucun changement du jeu : `#[cfg(test)]` et outils seulement.
- Commandes `carte <clé>` et `lot` (`tests-carte-lot.sh`, `lot sans-memoire` pour tout rechercher) : décrites dans `CONSIGNES.md` (section Charge) et `CARTE-DU-CODE.md`. `lint` LINT-OK.
- Limites : à mémoire vide (simulation ou robot modifiés), `lot` refait toutes les recherches (34 min : le robot du Monde 2, à lui seul). `carte <clé> robot` (sur demande) ajoute la recherche du robot de son niveau : caniveau 9 s, égouts 24 min 50 s (vert ; cascade, grotte et Pieuvre non mesurées seules) ; piste pour une carte suivante : accélérer cette recherche aux égouts. PAS mesuré : machine au repos (charge 7 à 65 pendant les mesures).

## Carte 9.22d : « La grotte aux bulles », des ennemis là où ils comptent

- Fait (`core/src/levels/grotte.rs`, grille seule) : +18 ennemis existants, aucun sur une lanterne ni au départ. 4 poissons (2 dans le lac de la barque, 1 dans chaque bassin, bassins élargis d'une case : 8-13 et 14-18), 1 crabe sur la corniche aux trois stalactites, 13 corbeaux (chauves-souris) en travers des sauts, des planés et des colonnes CA, C3, C4. Aucun rat ni grenouille ajouté : chaque ennemi d'eau ÉCRASABLE ajoute un bit à la case du robot et double la recherche d'itinéraires (version à 2 rats + 2 grenouilles : mesure arrêtée après 80 min).
- Mesure `CLE=grotte` (48 essais), morts AVANT -> APRÈS : Débutant 0,58 -> 1,23 ; Habitué 0,15 -> 0,29 ; Expert 0,02 -> 0,27 (12 morts au poisson du bassin du rocher). Étapes : 10 ennemis 0,23, 16 ennemis 0,27. CIBLE 0,65-0,80 NON ATTEINTE : le joueur simulé attend le bon moment pour les corbeaux, et la grotte n'a pas de vide où une erreur tue.
- Preuves : `levels::grotte` 2 ok, `zones::tests::monde_2_niveau_4_faisable` ok, `lint` LINT-OK. PAS lancés (décision d'AGA, il teste lui-même) : robot du Monde 2, captures. PAS testé : joueur humain, téléphone, Safari.

## Carte 9.12f : la Pieuvre saisit les plateformes (AGA : « ajouter une mécanique » plutôt qu'accélérer)
- Par-dessus les motifs inchangés, un tentacule saisit la plateforme de Pépin (4 appuis, une à la fois) : annonce 1,3 s (tentacule enroulé, plateforme qui tremble, elle porte encore), plongée 0,5 s (ne porte plus, la barque sauve), 3 s sous l'eau sous des remous, puis remontée, jamais dans Pépin. Toutes les 9, 7, 5 s selon les points perdus ; règle simple d'équité : quand la tête remonte, la pieuvre lâche la plateforme. Grille intacte : `GrilleSaisie` perce les cases englouties (`core/src/entities/pieuvre.rs`).
- Preuves : `carte pieuvre` VERT (2 tests de saisie : annonce ≥ 72 pas, une seule à la fois, remontée ; pas de retour dans Pépin ; + 4 appuis trouvés), `lint` LINT-OK, `lourd.sh ./build.sh --qa` OK. Captures regardées : `912f-annonce-saisie`, `912f-plateforme-sous-l-eau` (le pilier parti, remous à sa place) ; 0 hors palette, 0 flou.
- Remarque (hors carte) : l'écran « TROP DUR ? » de la Pieuvre dit encore « LE NUAGE N'A QUE 2 VIES ».

## Carte 9.24 : un temps de mort avant la réapparition (AGA : « on voit pas l'animation de noyade ou de dégâts »)
- À la mort, le monde se fige (ennemis, boss, plateformes, mur ; commandes ignorées) et l'animation de la cause se joue (`game/src/mort.rs`, `dessin_mort.rs`) : noyade 1,2 s (Pépin coule en se débattant, bulles cyan, parapluie couché qui flotte), dégât 1,0 s (flash blanc, éjection, chute à travers le décor en tournant d'un quart de tour), vide 0,6 s (la caméra reste au trou), puis un rideau de 0,3 s ; total 0,9 à 1,5 s. Vies, protection de 1 s et « PLUS DE VIES » inchangés (l'écran et la bannière « TROP DUR ? » attendent le rideau fermé).
- Seule la partie joue ce temps (`Monde::temps_de_mort`) : robot de QA, mesure, fantôme de la course et démo gardent la réapparition immédiate d'avant (leurs parcours enregistrés restent valables ; un fantôme qui meurt prend donc un peu d'avance sur le joueur).
- Preuves : `carte caniveau` et `carte toits` VERT (2 tests : pas de réapparition avant la fin, commandes ignorées), `lint` LINT-OK, `lourd.sh ./build.sh --qa` et `./build.sh` OK (0 nom interdit). Captures regardées : `924-noyade`, `924-degat`, `924-chute` (0 hors palette, 0 flou, 0 erreur de console). PAS testé : joueur humain, téléphone, Safari.

## Carte 9.25a : « Le caniveau » à Habitué 0,15 à 0,20 (courbe du chapitre 2)
- Fait (`core/src/levels/caniveau.rs`, grille seule, en-tête à jour) : 4 corbeaux `c` en travers de sauts (col. 16, 78, 121, 140), hors de l'autobus, des lanternes et du départ. Essayé puis retiré : un 5e au-dessus des auvents (col. 90, rangées 3 puis 2) : la relance y jetait Pépin (Habitué 0,38 puis 0,46).
- Mesure `CLE=caniveau` (48 essais), AVANT -> APRÈS : Débutant 0,54 -> 0,81 ; Habitué 0,04 -> 0,21 (48/48 finis, 45,0 s) ; Expert 0,04 -> 0,06 ; 11 644 cases, 379 s. Preuves : `carte caniveau` VERT, `lint` LINT-OK. Ni robot, ni capture ; PAS testé : joueur humain.
## Carte 9.25b : « Les égouts », la difficulté monte (cible Habitué 0,45-0,55)
- Fait (`core/src/levels/egouts.rs`, grille seule) : +2 corbeaux existants au-dessus des deux bassins à marée, (87, 6) et (105, 7), loin des lanternes (66, 95) ; 3 gouttes et lanternes inchangées. Mesure `CLE=egouts` (48 essais), AVANT -> APRÈS : Débutant 0,85 -> 0,98 (game over 1/48) ; Habitué 0,38 -> 0,54 ; Expert 0,17 -> 0,21 ; recherche 107 201 -> 113 073 cases (pas d'explosion ; 1 251 s sous charge 61).
- Preuves : `carte egouts` VERT (core 380 ok, game 368 ok, niveau 1 ok), `lint` LINT-OK. PAS lancés : robot, captures. PAS testé : joueur humain.
## Carte 9.25d : « La grotte aux bulles », la difficulté monte vers la Pieuvre (cible Habitué 0,70-0,80)
- Fait (`core/src/levels/grotte.rs`, grille seule) : un 2e poisson dans chaque bassin, là où l'on saute par-dessus : (8, 45) au bord du bassin de la salle, avant le nénuphar ; (18, 22) au bord du bassin du rocher, avant la grenouille. Aucun sur une lanterne ni au départ, 3 gouttes, 5 lanternes.
- Mesure `CLE=grotte` (48 essais, 16 478 cases), AVANT (19/11) -> APRÈS : Débutant 1,23 -> 0,98 ; Habitué 0,35 -> 0,73 (cible atteinte, 1 mesure ; col. 16-23 ennemi 20, col. 8 pointes 6) ; Expert 0,27 -> 0,27. PAS testé : joueur humain, robot (aucun robot), captures.
## Carte 9.25c : « La grande cascade » à Habitué 0,55-0,65 (courbe du chapitre 2)
- Fait (`cascade.rs`, grille seule) : +4 poissons au-dessus des courants (bassin A, pied de C1, galerie, rivière d'arrivée), +2 stalactites (attente du poisson de la galerie, plané vers la 3e goutte), +1 grenouille sur un nénuphar. Mesure 1 (poissons seuls) : Habitué 0,17 (itinéraire changé).
- Mesure 2 `CLE=cascade` : Débutant 0,44 (42/48 finis) ; Habitué 0,25 -> 0,65 (46/48 finis, max 12, game over 2/48, col. 25-26 ennemi 22) ; Expert 0,27 (max 11, game over 1/48). Morts concentrées en col. 25-26 (poisson du pied de C1 ou de la galerie). `carte cascade` VERT, `lint` LINT-OK ; robot, captures, joueur humain : PAS lancés.
## Carte 9.26a : grotte sans corbeaux dans les bulles, stalactites mortelles même accrochées (demande d'AGA)
- Fait : 9 corbeaux retirés de `grotte.rs` (tous ceux qui volaient dans ou au-dessus d'une colonne de bulles ; restent (18, 34), (10, 43), (11, 51), (8, 74)) ; `Stalactite::blesse` : toucher sa boîte tue (cause Pointe, animation « dégât ») accrochée, tremblante ou en chute, pas brisée ; +1 test `toucher_une_stalactite_accrochee_tue`. Aucune stalactite déplacée : faisabilité VERTE (aucun passage obligé ne la frôle).
- Mesures (1 chacune) : grotte Débutant 1,94, Habitué 0,73 -> 0,69 (col. 25-31 pointes 13), Expert 0,38 ; cascade Débutant 0,46, Habitué 0,65 -> 0,56, Expert 0,23. Aucun Habitué > 1,0 (Débutant grotte 1,94, pour info). `carte grotte`, `carte cascade`, `carte egouts` VERTS, `lint` LINT-OK ; robot, captures, joueur humain : PAS lancés.

## Carte 9.26b : le corbeau devient une chauve-souris au Monde 2 (AGA : « le skin des corbeaux dans le monde 2 soit une chauve souris »)
- Rendu seul (`scene::images_corbeau` : `monde >= 2`) : sprites `chauve_souris0/1` (18x14, 2 images, comme le corbeau ; `sprites/ennemis.rs`) de face, oreilles pointues, yeux rouges `8`, corps violet `F`, ailes membraneuses à pointes qui battent, liseré `4` sur le bord des ailes, contour sombre 1 px ; comportement, boîte et physique inchangés ; corbeau du Monde 1 inchangé. Planche de sprites régénérée.
- Preuves : `carte grotte` et `carte toits` VERT (2 tests neufs : choix par monde ; contour, yeux, liseré contrasté sur ARDOISE_OMBRE/NUIT), `lint` LINT-OK, `lourd.sh ./build.sh --qa` OK. Captures regardées : `926b-chauve-souris-grotte` (départ), `926b-chauve-souris-egouts` (grand collecteur), 0 hors palette, 0 flou, 0 erreur de console. Aucun robot ; PAS testé : joueur humain.

## Carte 9.26c : la cascade cohérente, chaque chute finit dans l'eau (AGA)
- Corrigées : chute du bassin F (col. 28-29) prolongée de 2 rangées dans le bassin G (le rocher de la rangée 60 où elle finissait est ôté) ; le puits (col. 22), qui finissait en l'air au-dessus du palier, retombe en filet de décor DERRIÈRE le palier jusqu'au bassin E (`decor_cascade`, sans collision) ; les paliers d'écume de la grande chute du fond passent des rangées 15 et 45 (en l'air) aux surfaces des bassins B et E (18, 46). Plus d'écume à une chute qui ne tombe pas dans l'eau (`eau.rs`) ; éclaboussures blanches et cyan au pied de chaque chute.
- Grille seule (rangées 60-61) : stalactites, ennemis, 3 gouttes, 6 lanternes inchangés. Test `levels::cascade::tests::chaque_chute_finit_dans_l_eau` (8 chutes, seul le puits débouche sur le vide, au-dessus d'une eau).
- Preuves : `carte cascade` VERT, `lint` LINT-OK, `lourd.sh ./build.sh --qa` OK ; captures `926c-avant|apres-puits`, `926c-avant|apres-chute-f` regardées (0 hors palette, 0 flou, 0 erreur). Mesure CLE=cascade, robot, joueur humain : PAS lancés.

## Carte 9.27 : plus rien de dehors sous terre (AGA : « dans les egouts il y a des cheminées et des décors de dehors »)
- Égouts : les 2 cheminées `H` (col. 5 et 229) retirées de la grille (décor seul, sans collision) ; fond en `Forme::Voutes` (arcade continue linteau + plein cintre + piliers) au lieu des masses `Roche` qui faisaient toits et cheminées ; pluie générale coupée (`Theme::pluie` faux, densité 0 ; rendu seul, aucune règle de jeu), restent les filets des grilles. Grotte : `Forme::Stalagmites` (cônes en pointe) au lieu de `Roche` ; elle était déjà sans pluie. Monde 1 inchangé. Test `decor::tests::sous_terre_rien_de_dehors`.
- Preuves : `carte egouts` et `carte grotte` VERT, `lint` LINT-OK, `lourd.sh ./build.sh --qa` OK ; captures `927-avant|apres-depart-egouts`, `-bassin-egouts`, `-grotte` regardées (0 hors palette, 0 flou, 0 erreur). Robot, joueur humain : PAS lancés.

## Carte 9.10 : « La rivière », Monde 2 niveau 5 (avant la Pieuvre, passée à 6)
- Niveau `core/src/levels/riviere.rs` (244 x 11) : descente en barque de bief en bief (5 courants `}`), 3 rochers `#` à sauter (grenouilles sur 2), 2 petites chutes qui finissent dans l'eau (test `levels::riviere::tests::la_chute_finit_dans_l_eau`), la mare aux 5 nénuphars, 10 poissons, 6 chauves-souris au-dessus de l'eau, 4 grenouilles, 3 gouttes, 3 lanternes. Intro dans `INTROS_MONDE_2`.
- Tuiles `riv_` (`tuiles_monde2/riviere.rs` : herbe violine aux pointes rousses, terre, galets, rochers moussus), branchées dans `JEUX`, `nom_du_theme`, `a_son_jeu` ; fond : `Detail::Foret` (deux rangs de sapins) ajouté au thème Riviere ; planche des sprites régénérée.
- Mesure `CLE=riviere` (48 essais) : Débutant 0,58 ; Habitué 0,81 (48/48, 53,5 s) ; Expert 1,92 (game over 6/48, 77 morts col. 165-166 : poisson du bief 4 sous sa chauve-souris, à surveiller en jeu).
- Preuves : `carte riviere` VERT, `lint` LINT-OK, `lourd.sh ./build.sh --qa` OK ; captures `910-depart`, `910-barque-poisson` regardées (0 hors palette, 0 flou). Robot, joueur humain : PAS lancés.

## Carte 9.11 : « Le port » (Monde 2, numéro 5 dans la branche, la Pieuvre passe à 6)
- `core/src/levels/port.rs` (97 x 11, thème Port) : quai du départ, 3 bateaux en va-et-vient obligés (bassins de 14 et 12 cases), caisse de grue verticale jusqu'au toit de l'entrepôt, 3 mouettes sur des caisses aux arrivées des bateaux, 3 crabes, 3 poissons, 3 gouttes, 3 lanternes, drapeau au bout de la jetée. Rendu : plateformes mobiles du port dessinées en bateau (`port_bateau`) ou caisse pendue à son câble (`port_caisse_grue`), `dessin_port::dessiner_mobile`.
- Preuves : `carte port` VERT, `lint` LINT-OK, `lourd.sh ./build.sh --qa` OK ; captures `911-depart`, `911-bateau`, `911-poisson` regardées (0 hors palette, 0 flou, 0 erreur de console). La grue et l'arrivée ne sont PAS capturées (pilotage à la main trop long).
- Mesure CLE=port : Débutant 1,25 morts, Habitué 0,35 morts (86 s), Expert 0,31. Sous la cible 0,85-1,00 : à juger par AGA en jouant (morts surtout aux arrivées des bateaux, col. 25-28, 75-76, 87-89). Robot de QA : PAS lancé (consigne).

## Carte 9.27b : un vrai collecteur sous les égouts (AGA : « une espèce de route et des voûtes bizarres »)
- Égouts : le fond n'est plus le ciel + couches de silhouettes (les tirets « route » étaient le `Detail::Tuyaux` à H/3, la bande sombre celui à 2H/3, les bosses les `Forme::Voutes`) mais `game/src/fond_egouts.rs` : mur d'ardoise nuit percé d'arches PLEINES tous les 112 px (claveaux, clé, pierres d'angle, galerie au fond), coulures et gouttes, une bouche grillagée sur trois piliers, gros tuyaux à colliers du plafond à l'eau sombre ; 2 plans de parallaxe, 3 ardoises + reflets cyan sombre. Tuiles : les `ego_voute_*` (festons sous chaque plafond) ne sont plus choisies, briques à la place (sprites intacts). Grotte : ciel d'une seule teinte (la trame à mi-hauteur faisait un horizon). Lumière de la grille du départ et Monde 1 inchangés.
- Preuves : `carte egouts` et `carte grotte` VERT, `lint` LINT-OK, `lourd.sh ./build.sh --qa` OK ; captures `927b-apres-depart|bassin|milieu|grotte` regardées (0 hors palette, 0 flou, 0 erreur ; ni route ni dehors). Robot, joueur humain : PAS lancés.

## Carte 9.28 : les deux fonds de la carte plus jolis, titre du chapitre 1, accents du bandeau (AGA)
- Fait : `game/src/ecrans/fond_plan.rs` (neuf). Chapitre 1 : ciel en bandes fondues en damier 2x2, soleil à rayons, oiseaux, nuages en volume (2 plans), éclairs lointains, 2 plans de toits et cheminées en parallaxe, pluie. Chapitre 2 : bandes du bas vers le haut rangées sur les étapes (rue et caniveau, égouts, cascade, grotte, rivière et port en attente, mer, horizon, Pieuvre en silhouette), reflets, gouttes, filets et bulles lents. Bandeau : « CHAPITRE 1 : LÀ D'OÙ VIENT LA PLUIE », accents dessinés à part, titre à y = 5 (tests `les_accents_du_titre_tiennent_dans_le_bandeau`, `les_bandes_du_chapitre_2_suivent_les_niveaux`).
- Preuves : `plan::` et `contenu::` VERTS (15), `lint` LINT-OK, `lourd.sh ./build.sh --qa` OK ; captures `docs/qa/928-carte-chapitre-1-bas|haut`, `928-carte-chapitre-2-bas|haut` regardées (accents visibles, titres centrés, étapes lisibles ; 0 hors palette, 0 flou, 0 erreur de console). Aucun robot ; PAS testé : joueur humain.

## Carte 9.29 : à la mort de Pépin, les ennemis reprennent leur place (AGA : « il faudrait que lors d'une mort les monstres reprennent leur emplacement d'origine »)
- `reprendre_apres_une_mort` appelle `Ennemi::revenir_a_l_origine` (reconstruit avec `new` à l'origine, plus la part `Depart` : sens, premier sommeil du chat, premier repos du poisson) et `Escargot::revenir_a_l_origine`. CHOIX du coordinateur : un ennemi ÉCRASÉ reste écrasé (le chemin gagné reste gagné) ; AGA peut demander l'inverse. Tests `entities::tests::apres_une_mort_les_ennemis_reprennent_leur_depart`, `apres_une_mort_un_ennemi_ecrase_le_reste`.
- Changements voulus : `monde::tests::les_commandes_sont_ignorees_pendant_l_animation` tolère le saut des ennemis au pas du rideau fermé ; `course/parties.rs` régénéré (Gouttières et le Grand Nuage divergeaient ; le marché couvert a maintenant ses 3 fantômes, donc un bouton COURSE). Preuves : `lint` LINT-OK, `carte toits` et `carte egouts` VERTS, `lourd.sh ./build.sh --qa` OK. Robot, mesure, joueur humain : PAS lancés.

## Carte 9.10b : « La rivière » moins linéaire, plus colorée, fond vivant (AGA : « trop linéaire et terne, fond vide et sans âme »)
- Niveau 206 x 34 (était 244 x 11) : la vallée descend en 4 paliers d'eau (rangées 9, 16, 23, 29). 2 CHOIX haut/bas : la gorge (ponts de bois + corniches, 1re goutte et grenouille en haut ; bief 2 et son rocher en barque en bas) et le plateau (la mare, ses nénuphars, un TRONC flottant `1`, la goutte cachée derrière un faux mur `m`) au-dessus de la petite grotte (2 chauves-souris) où passe le bief 5 ; 2 grandes chutes (7 et 6 rangées) qui finissent dans l'eau ; 3 lanternes, 3 gouttes. Caméra verticale : `camera.rs` (liste des niveaux hauts). Intros : `riviere.rs` et `contenu.rs`.
- Rendu : nouveau `game/src/decor_riviere.rs` (ciel couchant ardoise > violet > corail > orange > or pâle, soleil bas, montagnes enneigées, collines, rivière du fond qui scintille, forêt sapins + feuillus 2 tons, moulin à roue qui tourne et fenêtre allumée, grands arbres du premier plan, oiseaux, lucioles ; devant : reflets, écume aux rochers et au pied des chutes, roseaux, tronc corail) branché dans `scene.rs` (3 lignes) ; `decor.rs` NON touché. Tuiles : fleurs dans l'herbe, `riv_pont` (planches), galets plus clairs. Plus de pluie sur la rivière (`Theme::pluie`, rendu seul).
- Preuves : `carte riviere` VERT (core 384, game 376, niveau ok), `lint` LINT-OK, `lourd.sh ./build.sh --qa` OK ; captures `docs/qa/910b-depart`, `-embranchement`, `-grande-chute`, `-arrivee` regardées (0 hors palette, 0 flou, 0 erreur de console ; chute et arrivée prises avec le départ `@` déplacé le temps de la capture, non gardé).
- Mesure CLE=riviere : SAUTÉE (dépassé `timeout 600`) : Habitué NON mesuré (avant : 0,81). Ni robot, ni joueur humain.

## Carte 9.11b : vrai fond de port (demande d'AGA : « le fond est pas assez fini »)
- `game/src/decor_port.rs` (thème Port, branché par 2 lignes dans `decor.rs` et dans `scene.rs`) : ciel de crépuscule en bandes (nuit, violet, lueur corail, étoiles), mer et reflets, 2 cargos (l'un glisse), phare posé sur sa jetée avec faisceau en cône tournant, quai d'en face (entrepôts éclairés, conteneurs, portiques aux câbles tendus et balises), voiliers, pontons sombres avec filets, bouées, lampadaires et reflets ondulants, mouettes de décor ; bruine fine au lieu de la pluie. Pieuvre : même fond, sans le phare lointain (la jetée garde le sien). Quai : `por_taille`, `por_barbacane`, `por_algues` (rangée de l'eau), `por_echelle` (d'un tenant jusqu'à l'eau) ; rendu seul.
- Preuves : `lint` LINT-OK, `carte port` et `carte pieuvre` VERTS (tests `decor_port` 6, `le_mur_du_quai_varie...`), `lourd.sh ./build.sh --qa` OK ; captures `911b-depart`, `-bateaux`, `-grue`, `-pieuvre` regardées (0 hors palette, 0 flou, 0 erreur ; tentacules nettes sur le fond). PAS testé : joueur humain, robot ; « grue » montre le portique du fond, la caisse jouable de la darse n'est pas dans le cadre.

## Carte 9.30 : changer de chapitre au doigt sur la carte des niveaux (AGA : « le changement de chapitre c'est le seul truc qui marche pas »)
- Flèches « ◀ CHAPITRE 1 » / « CHAPITRE 2 ▶ » (74 x 16 px logiques, cadre sombre liseré d'or) dans la rangée libre du panneau au-dessus de JOUER/COURSE, donc hors de la vue : aucune étape cachée, toucher d'une étape inchangé. Affichées seulement si le chapitre voisin est jouable (`EtatPlan::chapitre_voisin`) ; un toucher/clic appelle `aller_monde`, comme Gauche/Droite. `navigation.rs` et `index.html` NON touchés. Tests `plan::tests::toucher_la_fleche_change_de_chapitre`, `pas_de_fleche_vers_un_chapitre_ferme`, `une_seule_fleche_de_chapitre_a_la_fois`.
- Preuves : `lint` LINT-OK, `plan::` (10) et `app::` (56) VERTS en release, `lourd.sh ./build.sh --qa` OK avec `WASM_OPT=0` (apt-get de wasm-opt bloqué sur le réseau). Au doigt (`docs/qa-930-carte-tactile.mjs`, Chromium sans tête, 844x390 DPR 3, `Input.dispatchTouchEvent`) : toucher « CHAPITRE 2 ▶ » passe au chapitre 2, puis « ◀ CHAPITRE 1 » revient au chapitre 1 (3/3 OK, `docs/qa/930-rapport.json`).
- Captures `docs/qa/930-carte-tactile-ch1|ch2` regardées (flèche lisible, sans chevauchement ; 0 hors palette, 0 flou, 0 erreur de console). PAS testé : joueur humain sur téléphone, robot.

## Carte 9.11c : le port finissable et varié (demande d'AGA : mouettes « infinissables », motif répété)
- Grille refaite (110 x 11) : 9 passages, aucun plus de 2 fois (bateau 1, pile de conteneurs, darse et caisse de grue, toit d'entrepôt puis vol plané au-dessus du bassin, quai bas, ponton flottant étroit et son crabe, duc d'albe, bateau 2, jetée et son crabe entre deux caisses) ; 4 lanternes (colonnes 26, 50, 65, 87) ; 3e goutte cachée au ras de l'eau du bassin plané (colonne 60, remontée par la barque et le ponton). Temps et constantes de la mouette et du pigeon NON touchés.
- Mouette 1 (colonne 40, sur une passerelle au-dessus du quai 2, tournée vers la gauche) : on court dessous sans s'arrêter, elle pique derrière Pépin ; attendre la caisse de la grue au bord (colonnes 41-42) est hors de son cône.
- Mouette 2 (colonne 73, passerelle au-dessus du quai bas) : même esquive en courant ; le bord du quai avant le ponton flottant (colonnes 74-77) est derrière elle, hors de son cône. Les deux s'écrasent d'un saut depuis le quai pendant leur lente remontée.
- Tests du module (`levels::port::tests`) : `aucune_mouette_ne_garde_un_endroit_ou_l_on_s_arrete` (départ, lanternes, bouts des plateformes, sol à moins de 4 tuiles d'un bassin), `on_passe_sous_chaque_mouette_en_courant_et_on_peut_l_ecraser` ; `dessin_port` adapté (2 bateaux).
- Preuves : `carte port` VERT (faisabilité, maladroit, reprise depuis chaque lanterne), `lint` LINT-OK, `lourd.sh ./build.sh --qa` OK ; captures `911c-depart`, `911c-mouette` (elle pique derrière Pépin qui l'a dépassée), `911c-conteneurs` regardées (0 hors palette, 0 flou, 0 erreur). PAS testé : joueur humain, robot de QA, mesure de durée.

## Carte 9.11d : le port plus long et plus dur (AGA : « le port : il est devenu court et trop simple »)
- Grille 227 x 11 (avant : 110) : 12 passages, aucun plus de 2 fois (bateau lent ; 2 bateaux à la suite, poisson dans le trou ; cale sèche vide jusqu'en bas, 2 piles de conteneurs et caisse de grue ; vol plané au-dessus d'un bassin, corbeau, 3e goutte cachée au ras de l'eau ; 2 pontons étroits à crabe et un poisson ; 3 bateaux sans repos, corbeau, poisson ; grue au toit de l'entrepôt puis vol plané ; 3 pieux, 2 poissons ; jetée, crabe entre 2 caisses). 7 lanternes (col. 27, 61, 94, 125, 151, 182, 206). Au port, `c` est dessiné en corbeau, plus en chauve-souris (`scene::images_corbeau`). Temps des mouettes NON touché.
- Mouettes, tournées vers la gauche sur une passerelle (rangée 1) au-dessus d'un quai où l'on court : col. 38 (quai 2), col. 102 (quai 4), col. 214 (jetée, avant le crabe). On les esquive en courant dessous sans s'arrêter (elles piquent derrière Pépin) ; on les écrase d'un saut pendant la remontée. Aucun bord de quai, bateau, lanterne ni caisse dans leur plongée (tests du module VERTS).
- Mesure CLE=port, une fois (73 s) : Débutant 1,00 mort (39/48 finis, 136,3 s) ; Habitué 0,69 (39/48 finis, 126,8 s, game over 1/48 ; col. 71-82 chutes de la cale 12, col. 115-120 pontons 11) ; Expert 0,04 (45/48 finis, 83,8 s). SOUS la cible 0,85-1,0, et 126,8 s pour 40-60 s visés : pas retouché, pas remesuré (consigne : une seule mesure).
- Preuves : `carte port` VERT (course rapide 40,3 s), `lint` LINT-OK, `lourd.sh ./build.sh --qa` OK. Captures `911d-depart`, `911d-bateaux` (passage le plus dur), `911d-mouette` (elle pique derrière Pépin) regardées : 0 hors palette, 0 flou, 0 erreur. `bateaux` et `mouette` viennent d'un build jetable, départ déplacé aux col. 126 et 33, hors dépôt. PAS testé : joueur humain, robot.

## Carte 9.12g : la saisie a son eau à elle (AGA : « quand il supprime une colonne il y a la même gerbe d'eau qu'une attaque normale »)
- Constat : `dessiner_saisie` reprenait les sprites d'une attaque (`remous_pieuvre*` sur toute la plateforme, dont les 2 rangées cyan redoublaient la surface, et `ombre_pieuvre`) ; aucune colonne d'attaque n'était annoncée ni dessinée à sa place, l'attaque garde une seule couche (inchangée). Désormais (`game/src/dessin_pieuvre.rs`, rendu seul, poteaux et décor intacts) : annonce = bulles qui remontent et cercle d'écume qui s'élargit ; plongée = creux en cuvette, écume qui tourne, lèvres d'écume aux bords, sans gerbe ; sous l'eau = cercle calme ; remontée = bulles. Sprites `bulle_saisie`, `onde_saisie`, `bord_creux_saisie` (planche régénérée).
- Preuves : `lint` LINT-OK, `carte pieuvre` VERT, `lourd.sh ./build.sh --qa` OK ; captures `912g-attaque`, `912g-saisie-annonce`, `912g-saisie-plongee` regardées (effets nettement différents, pas d'eau en double ; 0 hors palette, 0 flou, 0 erreur). PAS testé : joueur humain, téléphone.

## Carte 9.11e : les poissons de la fin du port sautent plus haut (AGA : « les poissons de la fin devraient sauter plus haut »)
- Hauteur de saut PAR poisson (`Ennemi::saut_poisson`, défaut `HAUT_SAUT_POISSON` = 56 px, gardée à la mort), réglée par niveau sans nouvelle lettre : `InfosNiveau::poissons_hauts` `(colonne, rangée, px)`, appliqué à `charger` et en NG+ (case sans poisson : erreur `PoissonIntrouvable`). Port : poissons (196,8) et (200,8) à 96 px (6 tuiles) ; dessin et contacts suivent la boîte ; faisabilité inchangée (elle ignore les poissons). Remous de 0,7 s et 1,5 s sous l'eau inchangés.
- Tests `levels::port::tests` : `seuls_les_deux_poissons_de_la_fin_sautent_plus_haut` (tous les autres poissons du jeu, NG+ compris, à 56 px) et `les_poissons_de_la_fin_menacent_le_saut_sans_toucher_les_pieux` (sommet au niveau d'un Pépin en plein saut ; jamais Pépin debout sur un pieu ou un quai). Preuves : `carte port`, `carte riviere`, `carte grotte` VERTS, `lint` LINT-OK, `lourd.sh ./build.sh --qa` OK ; capture `911e-poisson-haut` (poisson au sommet, Pépin sur le pieu du milieu, build jetable au départ déplacé) regardée : 0 hors palette, 0 flou, 0 erreur. PAS testé : joueur humain, robot, mesure.

## Carte 9.12h : la Pieuvre tape moins vite (AGA : « la pieuvre tape encore un peu trop vite pour un humain »)
- `core/src/entities/pieuvre.rs` seul : `ANNONCE_PAS` 72 → 84 (1,4 s, `POINTE_PAS` suit) ; `debut` de chaque motif étirés, salve toutes les 120 / 104 / 84 pas (avant 100 / 78 / 60), souffle du motif 3 140 → 196 ; fin de motif et sortie de la tête suivent (`duree_motif`). Bandes, saisie, tête, PV inchangés ; aucun test retouché (seul un commentaire « annoncée 84 »), `chaque_motif_se_traverse`, `un_humain_a_le_temps_de_changer_de_colonne` et la saisie VERTS. Preuves : `lint` LINT-OK, `carte pieuvre` VERT, `lourd.sh ./build.sh --qa` OK.
- Mesure CLE=pieuvre, une fois : Habitué 0,58 mort (avant 1,10 ; 43,1 s), Expert 0,10 (39,6 s), Débutant 3,75 (avant 2,88, plus haut ; 64,1 s, game over 17/48), tous 48/48 finis. Pas retouché, pas remesuré. PAS testé : joueur humain, robot, captures.

## Carte 9.13a : la Pieuvre, phase 2 « fuite et montée » (AGA : « elle manque toujours de quelque chose pour un boss final… Une phase deux ? »)
- Phase 1 inchangée ; au 3e coup, FURIE 2 s (écran qui tremble, bâche `_` déchirée, trombe au-dessus du trou du milieu), puis le quai casse ; MONTÉE dans une tour de 33 rangées au-dessus de l'arène (niveau 20 x 44) : mer qui monte (crue de l'arène, puis 1 tuile/1,3 s, 2 paliers de 2,5 s annoncés sous les corniches larges, x 0,75 en facile), mortelle au contact barque comprise ; un tentacule toutes les 2,2 s depuis la surface (annonce 1,4 s, 4 tuiles, jamais sans issue) ; un saut sur la tête `&` de la vigie : plongée, vaincue. Mort en phase 2 : lanterne du bas de la tour (allumée d'office), mer remise en bas. `entities/pieuvre/fuite.rs` ; `essai_pieuvre` inchangé (une phase).
- Tests `levels::pieuvre::tests` : furie au 3e coup (l'essai plonge toujours), la mer monte et tue, reprise au début de la phase 2, une issue à chaque tentacule (au px près, sur la vraie tour), le dernier coup finit le niveau, arène = essai décalée de 33 rangées. Faisabilité : 2 lanternes, phase 1 35,3 s et phase 2 vérifiées chacune ; `duree_cible_s` 75.
- Preuves : `lint` LINT-OK, `carte pieuvre` VERT, `lourd.sh ./build.sh --qa` OK ; captures `913a-colere`, `913a-montee`, `913a-sommet` regardées (build JETABLE non gardé : furie dès le départ, lanterne de la tour sur la vigie ; 0 hors palette, 0 flou, 0 erreur). Mesure CLE=pieuvre une fois (80 s) : Débutant 3,40 morts / 79,1 s, Habitué 1,50 / 64,3 s (avant 0,58), Expert 0,19 / 55,9 s, 48/48 finis partout.
- PAS testé : joueur humain, robot de QA de partie (ses tests de lot du Monde 2 ne connaissent pas la phase 2 : à vérifier au lot) ; dessin provisoire (bâche en tuiles de toit, mer en eau de la jetée, trombe en remous) : carte 9.13b.

## Carte 9.13b : la montée de la Pieuvre, plus longue et avec interactions (AGA : « la montée est trop simple et trop courte, il faudrait des interactions avec la pieuvre »)
- Tour doublée (niveau 20 x 80, 66 rangées de tour : caisses, vergue, plateforme mobile `1`, friable, sauts plongeants, 6 paliers) ; 2 RENCONTRES (`$`, au tiers et aux deux tiers) : la tête surgit (0,6 s), tient une trappe `-` fermée, la mer s'arrête, 2 tentacules de garde frappent chacun son tour (annonce 1,4 s) ; un saut sur la tête la fait lâcher (secousse), la trappe s'ouvre, la mer recule de 2 tuiles. Gardes aussi au coup final. PV phase 2 : 2 rencontres + vigie. 2 BALAYEURS (`!`, annonce 1 s, 180 px/s au ras de la corniche), 2 ARRACHAGES (`%`, annonce 1,3 s, eau de la saisie 9.12g, ne reviennent pas). Lanterne de mi-hauteur : la mort y ramène, 1re rencontre gardée, mer remise 3 tuiles sous elle. Jets de débris (point 5) : NON faits. `entities/pieuvre/montee.rs`, `game/src/dessin_montee.rs`.
- Tests `levels::pieuvre` : rencontres qui bloquent puis se débloquent (mer arrêtée puis reculée), gardes avec issue, balayeur annoncé et sautable, un chemin après chaque arrachage (robot de faisabilité sur les grilles sans `%`), reprise à mi-hauteur ; 3 lanternes (zones). `lint` LINT-OK, `carte pieuvre` VERT, `lourd.sh ./build.sh --qa` OK.
- Captures `913b-tour`, `-rencontre`, `-balayeur`, `-arrachage` regardées (builds JETABLES dans le scratch : phase 2 d'office, départ posé dans la tour ; 0 hors palette, 0 flou, 0 erreur).
- Mesure CLE=pieuvre lancée une fois : DÉPASSE 600 s (coupée par `timeout`, itinéraires trouvés en 77 s, aucun résultat de profil) : difficulté NON mesurée. PAS testé : joueur humain, robot de QA de partie, mode facile en jeu.

## Carte 9.13c : la Pieuvre GÉANTE grimpe à côté de la tour (AGA : « je n'aime pas la montée, les attaques n'ont pas de sens »)
- Tour resserrée à 13 colonnes ; couloir de 5 colonnes à droite où la géante (tête 64x40, œil qui suit Pépin, plissé et rouge pendant chaque annonce) monte avec la mer, ou grimpe hors de l'eau pour rester 4 tuiles sous Pépin ; son corps tue. Attaques, toutes tirées de son corps (tentacule entier en tronçons) : frappe visée (repère rouge 1,4 s, toutes les 2,5 s, jamais sur la seule place sûre), 2 balayages (`!`, 1 s, droite vers gauche), 2 secousses (`+`, 2 ou 3 caisses annoncées 1 s par leur ombre, jamais voisines), 2 arrachages (`%`, tirés dans la mer), 2 jetés (`$`, cabrée 1,2 s puis sonnée 2 s : un saut sur la tête = mer -3 tuiles, 3 s de répit) et le coup final sur la vigie. Supprimés : tentacules de surface, trappe, gardes. Facile : annonces et rythme x0,75. `entities/pieuvre/montee.rs`, `game/src/dessin_montee.rs`.
- Corrections 9.13b : fond continu sous la tour (le port de l'arène descend, la nuit étoilée continue au-dessus, plus de bande de couchant répétée) ; chrono sur une plaque d'ardoise (le « 0:0A » était le dernier chiffre sur la rayure claire du quai).
- Tests `levels::pieuvre` : géante qui monte et tue, chaque frappe visée laisse une issue (au px près, tous les appuis), balayages et caisses annoncés, chemin après chaque arrachage, jeté sonné frappé et mer qui recule, coup final, reprise à mi-hauteur. `lint` LINT-OK, `carte pieuvre` VERT (zones comprises), `lourd.sh ./build.sh --qa` OK.
- Captures `913c-frappe`, `-secousse`, `-sonnee`, `-haut` regardées (build JETABLE dans le scratch : phase 2 d'office, départ posé dans la tour ; 0 hors palette, 0 flou, 0 erreur). Mesure CLE=pieuvre NON lancée (elle dépassait 600 s à la 9.13b). PAS testé : joueur humain, robot de QA de partie, mode facile en jeu.

## Carte 9.13d : la géante choisit ses attaques, s'énerve, et le coup final demande toutes ses têtes (critique d'AGA, partie « attaques »)
- Plus aucune rangée fixe : son CERVEAU (`entities/pieuvre/montee/cerveau.rs`) liste ce qui est possible là où est Pépin (corniche large : jeté ; corniche contre son couloir : balayage ; debout sous la tour : secousse ; sous une `%` : arrachage ; frappe toujours), une recharge par attaque, et tire au sort (xorshift) avec une graine qui change à chaque reprise. Places de jeté et balayeurs tirés de la grille au début de la montée (4 jetés, 4 balayeurs ici). La frappe vise l'appui où Pépin VA ATTERRIR (sa vitesse, les appuis de la tour), sur lui s'il s'arrête.
- Humeurs : calme, ÉNERVÉE (1er coup ou 50 % de la tour), FURIEUSE (2e coup ou 80 %) : recharges x0,7 puis x0,5, combos balayage ou secousse puis frappe visée (la frappe tombe après la partie mortelle de la 1re). Ça se voit : veines violettes, œil injecté, pupille orange puis rouge, tête qui chauffe et vapeur (sprites `geante_furieuse*`, `veines_geante`, `oeil_injecte*`, `vapeur_geante*`, `game/src/dessin_montee.rs`). Vigie : deux jetés de suite (1,2 s puis 0,8 s), sonnée seulement après le 2e, autant de fois qu'il reste de têtes (3, moins les coups de la montée). Reprise à N'IMPORTE QUELLE lanterne : mer dessous, têtes et humeur de son passage, graine suivante.
- Tests `montee::tests` : deux graines deux suites, visée sur l'appui d'arrivée, humeur après un coup, combo avec issue (graines essayées jusqu'à en voir un de chaque sorte), vigie deux jetés avant sonnée, têtes exigées à la vigie, reprise à trois lanternes ; anciens tests `levels::pieuvre` adaptés. `lint` LINT-OK, `carte pieuvre` VERT, `lourd.sh ./build.sh --qa` OK.
- Captures `913d-visee` (Pépin en l'air, repère sur la corniche d'arrivée), `913d-furieuse` (balayage plus frappe enchaînée, tête orange, vapeur), `913d-vigie` (sonnée après le 2e jeté, 3 têtes) regardées ; build JETABLE dans le scratch (phase 2 d'office, départ dans la tour, attaques filtrées pour la capture) ; 0 hors palette, 0 flou, 0 erreur. PAS testé : joueur humain, mode facile en jeu, robot de partie. Pour 9.13e : `reprendre_a` gère déjà toute lanterne de la tour, rien à changer dans `fuite.rs`.

## Carte 9.13e : la montée, côté décor (AGA : « on ne voit pas la mer », « la tour est monotone », « le fond est vide »)
- Mer toujours sensible (`game/src/mer_montee.rs`) : la caméra garde la surface proche au bas de l'écran (`camera::cadrer_la_mer`, Pépin jamais au-dessus de 30 %) ; crête d'écume et embruns DEVANT la géante (elle plonge dans l'eau, son dessin est inchangé) ; mer hors champ : bande d'écume au bas de l'écran, plus haute et plus agitée quand elle approche ; jauge sur le mur gauche (tête rouge = vigie, crans = lanternes, flèche = Pépin, corail clignotant à moins de 2 tuiles, la mer par le bas).
- Tour en sections (aucune plus de 2 fois) : pile de conteneurs, nuages friables `F` (58, 31, 13), quai de pierre, mât et vergue, cheminée de cargo ; 6 corniches larges (52, 40, 28, 25, 22, vigie : 5 paliers). 2e lanterne de reprise à 69 % (rangée 24, sur la corniche calme 25, juste après le 2e jeté) : test `reprise_a_la_lanterne_de_69_pour_cent` (2 jetés gardés, mer sous ses pieds à 3 tuiles au plus) ; aucun changement de `fuite.rs` nécessaire. Pas de grue à plateforme mobile ni de saut plané dédié (non faits).
- Fond (`decor_port.rs`) : au-dessus du port de l'arène, le PORT LOINTAIN (phare et faisceau qui tourne, grues à feu rouge, cargos, lumières du quai et reflets) s'enfonce lentement vers le bas de l'écran, le crépuscule s'éteint en nuit étoilée, la lune en haut.
- Preuves : `lint` LINT-OK, `carte pieuvre` VERT, `lourd.sh ./build.sh --qa` OK ; captures `913e-mer-proche`, `-mer-loin`, `-tour`, `-hauteur` regardées (build JETABLE dans le scratch : phase 2 d'office, départ posé dans la tour ; 0 hors palette, 0 flou, 0 erreur). PAS testé : joueur humain, robot, mode facile en jeu ; à fusionner avec 9.13d (jetés choisis : `levels/pieuvre.rs` tests, `$` de la grille).

## Carte 9.31 : les textes attendent un appui
- La carte du niveau et « PLUS DE VIES » ne se passent plus seules : appui (Entrée, toucher) accepté après 0,5 s (`histoire::DELAI_PAS`), « continuer » clignote dès lors (`histoire::invite_visible`) ; la carte passée vide `resultat_niveau` et le jeu part au pas suivant (`carte::PASSEE`, premier pas des parcours du robot inchangé). Aucun autre texte ne se passait seul. Robot QA : appuie sur ces écrans (`texte_en_attente`) ; `capture-912/94.mjs` : Entrée ajoutée ; à revoir : `capture-course.mjs` (`delai`) et les scripts tirés d'anciennes traces (à régénérer).
- Preuves : 3 tests (`la_carte_reste_affichee_dix_secondes_sans_appui`, `un_appui_avant_le_delai_est_ignore_apres_il_passe`, `l_invitation_clignote_une_fois_l_appui_accepte`), tests d'`app.rs` adaptés ; `lint` LINT-OK, `carte toits` et `carte pieuvre` VERT (game 398), `ecrire_script_essai_ennemis_eau_94` ok (décalage 55), `lourd.sh ./build.sh` (0 nom QA dans dist) et `--qa` OK ; capture `931-carte-attend` (5,5 s sans appui, 0 hors palette, 0 flou, 0 erreur). PAS testé : téléphone, robots de partie complète (`lot`).

## Carte 10.1a : crate `niveaux`, niveaux en fichiers RON (les `.rs` restent, bascule en 10.1b)
- Crate `niveaux` (serde + ron ; `core` toujours sans dépendance) : `build.rs` embarque par `include_str!` chaque `.ron` de `assets/levels` (monde_1/, monde_2/, essais/ : 33 niveaux, 19 du parcours et 14 essais ; le 34e `.rs` est `variantes.rs`, pas un niveau) ; place donnée par `monde` et `numero`, aucune liste. `charger(texte)`, `charger_table`, `niveaux()` (`OnceLock`, textes en `Box::leak` une fois). Erreurs lisibles (fichier, ligne et colonne de grille et de fichier) qui enveloppent `ErreurNiveau` et `verifier_ordre` de `core`. Seul changement de `core` : `PartialEq` sur `InfosNiveau`.
- Outils : `run -p niveaux --bin exporter` (réécrit les 33 `.ron` et la légende de `assets/levels/LISEZMOI.md`, tirée de `core/src/level.rs` ; relancé : mêmes octets) et `--bin valider [fichier]`. 13 tests : équivalence de CHAQUE niveau (tous les champs, `Niveau` chargé comparé par `{:?}`), table RON = `NIVEAUX`, format stable, légende à jour, exemple du LISEZMOI, une erreur par faute (lettre, dimension, `@`, `X`, aiguilles, poissons, champ, thème, clé en double, dossier, trou de numérotation).
- Preuves : `lint` LINT-OK (clippy niveaux ajouté), `test -p niveaux` 13 ok, `carte toits` et `carte port` VERTS, `./build.sh --qa` OK. wasm (`./build.sh`, TOTAL) : 957544 octets avant ET après (le jeu ne lit pas encore les `.ron`).
- Reste pour 10.1b : le jeu lit `niveaux::niveaux()`, suppression des `.rs` de niveau et de `source_rust.rs` (les tests d'équivalence deviennent sans objet), mesure du wasm avec serde/ron.

## Carte 10.1b (partielle) : le jeu lit ses niveaux dans les fichiers RON
- Fait : `game` (carte, sauvegarde, course, démo, difficulté, robot, essais) lit tout par `niveaux::niveaux()`, `niveaux::par_cle`, `NB_NIVEAUX`/`NB_NIVEAUX_MONDE_1`/`CLES_HORS_MONDE_1` (constantes écrites par `niveaux/build.rs` : `[Intro; NB_NIVEAUX_MONDE_1]` et l'intro de chaque niveau du Monde 2 restent vérifiés à la compilation) ; essais embarqués seulement en QA et tests (fonctionnalité `niveaux/essais`, absents de `dist/`). `./build.sh` lance d'abord `tous_les_fichiers_embarques_sont_valides` ; en QA, un fichier invalide s'affiche à l'écran (`game/src/essai_externe.rs`) et `?essai=1` lance le RON de `localStorage` `pepin.niveau_essai`. `carte <clé>` accepte les clés des `.ron` et lance aussi les tests de `niveaux`.
- Preuves : `lint` LINT-OK ; `carte` toits, orage, cyclone, grotte, port, pieuvre VERTS (core 408, game 399, niveaux 14) ; `./build.sh` (0 nom interdit, wasm 971245 octets, +66944 pour serde/ron) et `--qa` OK ; `capture-101b.mjs` + `comparer-captures.py` : départ de toits, port, pieuvre monde-2 / 10.1b, 0 pixel différent sur 3 (`docs/qa/101b-*`) ; `101b-essai-externe` et `101b-niveau-invalide` regardées (0 hors palette, 0 erreur de console). Niveau bidon (`essais/essai_bidon.ron` seul) : `valider` dit 15 essais, la garde le charge, puis retiré.
- RESTE (étape 3, non faite, trop longue pour la carte) : supprimer `core/src/levels/*.rs`, la table `NIVEAUX` et `source_rust.rs` ; déplacer vers `niveaux/tests/` les ~4000 lignes de tests du cœur qui lisent la table (tests de `zones.rs`, de `levels.rs` sur la vraie table, de `lettres.rs`, `entities/pieuvre/montee/tests.rs` et les `mod tests` de 17 fichiers de niveau ; rendre `pub` ce qu'ils utilisent ; adapter `core-niveau`/`core-module`/`lot` de `tests-carte-lot.sh`) ; reporter les commentaires de conception des `.rs` en `//` dans les `.ron` (script, vérifié sur 3 niveaux) ; `variantes.rs` reste dans `core`.

## Carte 10.3a : éditeur de niveaux web (site de test seulement)
- Fait : `editeur/` (HTML, CSS, JS sans dépendance), copié par `./build.sh --qa` en `dist-qa/editeur.html` + `editeur/`, avec `legende.json` (crate `niveaux`, module `legende` : 78 lettres, catégorie, solide, appui, taille, nombre, couleur ; un test essaie chaque caractère ASCII avec le parseur de `core`) et `niveaux/` (les 33 `.ron` + `index.json`, `--bin editeur`). Grille colorée (zoom, défilement, pinceau, rectangle, gomme, pipette, annuler/refaire, redimensionner, case sous la souris), palettes par catégorie, `@`/`X`/boss uniques, avertissements doux (appui, poche close, pigeon près d'un oiseau, ennemi sur un point de reprise, aiguilles/poissons), formulaire de tous les champs, ouverture du jeu ou d'un fichier local, export canonique (`editeur/ron.js`, commentaires `//` gardés), brouillon dans `localStorage` (`pepin.editeur.brouillon`). `./build.sh` refuse un `dist/` qui contiendrait l'éditeur.
- Preuves : `lint` LINT-OK ; `test -p niveaux` 16 ok ; `node editeur/test-aller-retour.mjs` 33/33 mêmes octets (aussi lancé par `--qa`) ; `./build.sh` (dist = 3 fichiers, 0 nom interdit, wasm 971245 inchangé) et `--qa` OK ; `docs/capture-103a.mjs` sur dist-qa : aller-retour 33/33 dans Chromium, dessin à la souris, `@` unique, Ctrl+Z, brouillon repris, 0 erreur de console ; `docs/qa/103a-grille.png` et `103a-proprietes.png` regardées.
- Le format ne sait PAS régler : gravité, vitesses, saut, planer, force du vent ou des courants, vitesse des plateformes et des ennemis (constantes de `core`) ; seuls `sans_planer`, `duree_cible_s`, `aiguilles` et `poissons_hauts` se règlent par niveau. Pas de vignettes de sprites (cases colorées + lettre).
- PAS fait ou testé : essai immédiat et robot (10.3b, 10.3c), Safari, tactile. `publier-prod.sh` ne copie sur www-test que 3 fichiers : y ajouter `editeur.html editeur legende.json niveaux` de dist-qa.

## Carte 10.3b : TESTER depuis l'éditeur, validation par le jeu, partir d'ici
- Fait : boutons TESTER et Valider dans l'éditeur ; le RON courant est posé dans `localStorage` `pepin.niveau_essai` et le jeu de QA (`index.html?essai=1`) se charge dans un cadre plein écran de l'éditeur (même origine). Le jeu le juge avec SON chargeur (`niveaux::charger_fichier`, aucune règle recopiée en JS) et répond par `postMessage` (import wasm `pepin_niveau_essai_signaler`, `web/index.html`) : `{etat:"pret"}` (le cadre se montre, la carte du niveau affiche SON `intro` et SA `mecanique`), `{etat:"erreur", message, ligne, colonne, ligne_fichier}` (le cadre se ferme, le message s'affiche sous la grille, la case est cerclée de rouge et la vue défile jusqu'à elle ; `ErreurNiveau::case()` dans `core` donne la case de chaque variante qui en a une), `{etat:"fini", raison:"echap"|"fin"}` (Échap lu par le jeu, ou l'écran de fin après le drapeau : retour à l'éditeur, cadre fermé). Clic droit sur une case VIDE : « partir d'ici » (le `@` déplacé dans le texte envoyé seulement, la grille de l'éditeur ne change pas) ; la pipette passe sur Alt + clic (et l'outil Pipette, touche I). Échap dans l'éditeur ferme aussi l'essai. Jeu ouvert seul avec `?essai=1` : la fin de l'essai revient en arrière (`history.back`).
- Preuves : `lint` LINT-OK ; `test -p game --no-default-features --features debug_niveaux` 400 ok (dont `le_rapport_d_erreur_donne_la_case_fautive`, et l'intro du niveau externe) et sans `debug_niveaux` 400 ok ; `test -p niveaux` 16 ok ; `test -p core --lib level` 81 ok ; `./build.sh` (éditeur absent de dist/, wasm 971245 octets INCHANGÉ, aucun import `pepin_niveau_essai*`) et `--qa` OK (aller-retour 33/33) ; `CHROME=…/chromium-1208 node docs/capture-103b.mjs PORT docs/qa` sur dist-qa : ESSAI-OK (13 vérifications : niveau MODIFIÉ reçu tel quel, Échap lu par le jeu, pigeon en plein ciel refusé avec la case (5,1) surlignée et le message du chargeur, Ctrl+Z, partir d'ici en (41,7) avec un seul `@`, fin du niveau depuis la gauche du drapeau → retour à l'éditeur, 0 erreur de console). Captures regardées : `docs/qa/103b-essai-carte`, `103b-essai` (la rangée de toit ajoutée au-dessus du départ est là), `103b-erreur`, `103b-partir-d-ici`, `103b-retour-apres-la-fin`.
- PAS fait ou testé : robot sur le niveau édité (10.3c), Safari, tactile (pas d'Échap au doigt : la fin du niveau ou le bouton retour du navigateur), essai d'un niveau de plus de 64 Kio en vrai (le code relit avec la bonne taille). Une erreur de syntaxe RON (que l'export de l'éditeur ne produit pas) n'a pas de case : seulement `ligne_fichier`.

## Carte 10.3c : éditeur, bouton ROBOT (robot de faisabilité dans le navigateur)
- Fait : crate `robot` (le robot `core::faisabilite` compilé en `robot.wasm`, profil `robot-web` opt 3, build de QA seulement, refusé dans dist/), lancé dans un Web Worker (`editeur/robot-worker.js`) : limite réglable (30 s par défaut) et Arrêter (le Worker est terminé). `Rapport::chemin` (nouveau : les points de décision du trajet le plus rapide, ou jusqu'à l'état le plus à droite) dessiné sur la grille avec le temps de jeu, les sauts, gouttes et lanternes ALLUMABLES (exploration complète : pas seulement celles du chemin) ; en échec, l'endroit bloquant cerclé « bloqué ici » et la raison. « Défi sans planer » + bouton « Recopier dans sans_planer ».
- Mesures (Chromium sans écran, un seul fil en wasm) : toits 3,3 s (343 états), port 7,2 s (505), pieuvre 10,1 s (289) : sous la limite de 30 s, pas trop lent. Robot non optimisé (consigne).
- Preuves : `lint` LINT-OK ; `test -p robot` 4 ok, `-p niveaux` 16 ok, `-p core --lib faisabilite::` 20 ok (dont `le_chemin_va_du_depart_vers_l_arrivee_ou_l_endroit_bloquant`) ; `./build.sh --qa` OK ; `node docs/capture-103c.mjs PORT docs/qa` TOUT OK (8 vérifications, 0 erreur JS). Captures regardées : `docs/qa/103c-chemin.png` (port), `docs/qa/103c-echec.png` (toits avec un mur ajouté colonne 58 : bloqué colonne 57).
- Reste : la raison d'un échec est générique (le robot ne sait pas QUEL saut manque) ; le Worker n'a pas de progression en cours de recherche ; Safari non essayé.

## Carte 10.2a : `scene.rs` devient le dossier `game/src/scene/`
- Constat : `scene.rs` ne contenait QUE du dessin (aucune `Scene`, ni état, boucle, transition, entrée ou caméra : tout cela vit dans `app.rs`, `entrees.rs`, `camera.rs`, `effets.rs`). Un seul `rendu.rs` aurait fait 2 000 lignes : découpé par ce qui est dessiné. Code recopié tel quel (script de plages de lignes), seuls imports, visibilités `pub(super)`/`pub(crate)` et en-têtes changent ; chemins `crate::scene::...` inchangés (réexports dans `mod.rs`, carte des responsabilités en tête).
- Tailles (lignes / Ko) : `mod.rs` 222 / 11 (`dessiner`, ordre des couches, aides, `borner_mort`), `tuiles.rs` 427 / 18, `habillage.rs` 600 / 25, `entites.rs` 624 / 28, `nuage.rs` 309 / 13, `pepin.rs` 422 / 18, `interface.rs` 67 / 3 ; aucun au-delà de 900. Les 33 tests de scène suivent leur code.
- Preuves : `lint` LINT-OK, tests rapides VERTS (`test --workspace --no-default-features` : game 408, core 398), `./build.sh --qa` OK ; captures `docs/qa/102-avant-*` (monde-2, 487c760) et `102-apres-*` (toits, port avec mouette, Pieuvre), `python3 docs/comparer-captures.py docs/qa/102-avant docs/qa/102-apres` : 3/3 IDENTIQUES, 0 pixel différent (refaire : `CHROME=... docs/capture-102.sh PORT docs/qa PREFIXE`).
- Pour 10.2b : plus rien de transitions, entrées, caméra ni effets dans `scene/` ; ces responsabilités sont dans `app.rs` (3 053 lignes), à découper là si AGA le veut.

## Carte 10.2b : `app.rs` devient le dossier `game/src/app/`
- Constat : après 10.2a, transitions, entrées, caméra et effets vivaient dans `app.rs` (3 053 lignes). Découpé par responsabilité, code recopié tel quel (script de plages de lignes) : seuls imports, `pub(super)` et en-têtes changent ; le bloc caméra + effets de `pas_jeu` devient `cadrer_et_animer` (`camera_effets.rs`, à côté de `camera.rs` et `effets.rs` qu'il appelle). Carte des responsabilités en tête de `app/mod.rs` ; `CARTE-DU-CODE.md` (et son générateur) dit quel fichier lire pour un effet, l'interface, une transition, les entrées.
- Tailles (lignes / Ko) : `mod.rs` 455 / 19 (état, chargement, `pas`), `entrees.rs` 351 / 16 (`reagir`), `transitions.rs` 353 / 16 (`ouvrir`, `lancer`, `appliquer_suite`, `Suite`), `progression.rs` 160 / 8, `camera_effets.rs` 46 / 2, `demo_titre.rs` 78 / 3, `rendu.rs` 199 / 9, `tests.rs` 1 570 / 57 (59 tests déplacés tels quels).
- Preuves : `lint` LINT-OK, tests rapides VERTS (game 408, core 398, comme 10.2a), `./build.sh --qa` OK ; `docs/qa/102b-apres-*` contre `102-avant-*` : 3/3 IDENTIQUES, 0 pixel différent ; transition `102b-transition-fermeture` (volets à mi-course après N, même `lancer` que la fin de niveau ; 0 erreur console). PAS testé : robots de partie complète, téléphone, capture de la transition AVANT (non comparée au pixel).

## Carte 10.2c : la chaîne « niveaux en RON » reportée dans `game/src/app/` et `game/src/scene/`
- Patch de monde-2 sur `app.rs`/`scene.rs` (37 blocs) reporté dans les fichiers découpés, imports répartis par fichier ; les 112 autres fichiers de `carte-10-niveaux` (depuis 487c760) copiés tels quels ; `docs/comparer-captures.py` : version 10.2 gardée, elle compare aussi par paires (usage 10.1b).
- Preuves : `lint` LINT-OK ; `carte toits`, `carte pieuvre` VERTS (core 409, game 400, niveaux 16, robot 4, game QA 399) ; `./build.sh` (seule URL : svg) et `--qa` OK ; `docs/qa/102c-apres-*` contre `101b-apres-*` : 3/3 IDENTIQUES, 0 pixel différent, 0 erreur console. PAS testé : robots de partie complète, éditeur web.

## Carte 10.1c : plus aucun niveau en Rust, tests de tous les niveaux dans `niveaux/tests/tous_les_niveaux/`
- Fait : `core/src/levels/*.rs` (sauf `variantes.rs`), `NIVEAUX`, `GOUTTES_TOTAL`, `zones::niveau`, `source_rust.rs` et l'exporteur supprimés ; 142 tests du cœur déplacés (un fichier par sujet + `par_niveau/<clé>.rs`), `core` ouvre 4 réglages de `Pieuvre` derrière `pour_les_tests` ; commentaires reportés dans les 33 `.ron` (`niveaux/outils/reporter_commentaires.py`, relus sur toits, clocher, port), vidés du texte embarqué par `build.rs` (lignes gardées) ; `carte`/`lot` lisent le binaire `tous_les_niveaux` ; README, LISEZMOI, CARTE-DU-CODE (et `carte-du-code.py`, .bak) à jour. 3 tests de `game` lisaient `core::levels` (`decor.rs` 2431, `effets.rs` 271, `dessin_port.rs` 500) : MODIFIÉS d'une ligne chacun vers `niveaux::`, sinon `game` ne compile plus (conflit possible avec 10.2c qui recopie 10.1b dans ces fichiers : garder notre version).
- Preuves : `lint` LINT-OK ; `carte` toits, orage, cyclone, grotte, port, pieuvre (et clocher : 3 tests `par_niveau::clocher`) VERTS ; tests avant 490 (core 474 + niveaux 16) → après 490 (core 332, niveaux 15, tous_les_niveaux 143) : 4 tests d'équivalence Rust/RON retirés, 4 ajoutés (export RON stable, clés valides, texte embarqué = fichier sans commentaires, gouttes de la vraie table) ; `./build.sh` (0 nom interdit, wasm 971237 octets, -8) et `--qa` OK ; aller-retour de l'éditeur 33/33.

## Carte 9.13f : montée de la Pieuvre, la mer qui presse, un final moins répétitif (audit d'AGA du 24/11)
- Fait (commit 8c71180) : 1. LA MER SUIT PÉPIN (`montee.rs`, `avancer_la_mer`) : jamais plus de 8 tuiles (10 en facile) sous le plus haut qu'il a atteint ; au-delà, l'horloge de la mer avance à vitesse TRIPLE (au plus 3 vitesses par pas, plus un pas d'horloge pendant la crue : pas de saut), jamais à moins de 4 tuiles de ses pieds, puis vitesse normale ; jamais de descente sauf le recul de 3 tuiles après un coup (le plus haut de Pépin recule d'autant : le recul n'est pas rattrapé aussitôt). Inchangés : arrêt pendant un jeté, paliers annoncés, mer sous la lanterne à la reprise (le plus haut est remis à la lanterne). 2. LANTERNE sur la caisse de la rangée 10, juste sous la vigie (5 lanternes : tests `nb_lanternes`, faisabilité). 3. TROIS CYCLES DE VIGIE (`cerveau.rs`, `cycle_de_la_vigie`) selon les têtes qui restent : deux jetés ; jeté, frappe visée, jeté vif ; balayage de la vigie (balayeur ajouté sur la vigie, réservé aux cycles), jeté vif. Annonces de 0,8 s au moins (mesurées), une place sûre à chaque pas ; répit de 1,5 s entre deux cycles (`REPIT_VIGIE_PAS`) : 835 pas (13,9 s) au plus vite pour 3 têtes, contre 870 (19 s mesurées par AGA avec les attentes) avant ; 5 jetés au lieu de 6. 4. CORNICHE LARGE rangée 16 (cases 6 à 13, contre le couloir) et caisse ARRACHABLE à gauche (2 à 4) : un palier de plus (6), balayage et secousse au-dessus de 80 % (graines 1 à 5 : balayage 5/5, secousse 5/5) ; aucune section plus de 2 fois ; chemin après chaque arrachage (test vert). 5. ÉCUME : la cause des « crêtes en escalier dans le vide » était la TROMBE, dessinée avec le sprite de remous (une crête d'écume) en diagonale dans l'air ; elle est maintenant faite de BULLES de 2 px (palette), et les remous ne sont dessinés qu'à la surface de la mer (`dessin_pieuvre.rs::dessiner_mer`, seul changement dans `game/src`). Phase 1 inchangée, aucun sprite ajouté ni modifié.
- Tests ajoutés (`niveaux/tests/tous_les_niveaux/pieuvre_montee.rs`) : `la_mer_suit_pepin_sans_jamais_descendre` (normal et facile, deux allures, écart jamais dépassé, aucune descente, aucun saut, paliers annoncés, rattrapage jamais à moins de 4 tuiles), `le_rattrapage_ne_tue_pas`, `reprise_a_la_lanterne_sous_la_vigie` (2 têtes, humeur furieuse de son passage, mer dessous, cycle repris du début), `trois_cycles_de_vigie_differents` (issue, annonces, durée), `le_dernier_tiers_accueille_toutes_les_attaques`. Adaptés : vigie (5 jetés, 1er cycle seul), 5 lanternes, 6 paliers, balayeur de vigie hors du test des balayages.
- Preuves : `lint` LINT-OK ; `carte pieuvre` VERT (core 326, game 400, niveaux 15, tous 89 + faisabilité du niveau) ; `./build.sh` OK (0 nom de QA dans le wasm : `essai_pieuvre`, « Niveau d'essai », `qa_plantage`, `debug_niveaux` : 0) et `--qa` OK. Captures du jeu réel, décor final compris (build JETABLE dans le scratch : phase 2 d'office, départ sur la vigie ou au bas de la tour, attaques non mortelles, coup automatique quand elle est sonnée, mer qui ne s'arrête pas pendant les jetés pour la voir monter) : `913f-rattrapage` (mer qui rattrape, jauge), `913f-vigie-cycle1` (2 jetés), `913f-vigie-cycle2-furieuse-mer-proche` (furieuse, repère de la frappe visée, mer proche, jauge), `913f-vigie-cycle3-balayage`, `913f-lanterne-du-haut`, `913f-ecume-surface` (trombe en bulles, écume seulement sur l'eau) ; regardées, 0 hors palette, 0 flou, 0 erreur du pilote. Pilote CDP : Chromium absent de la machine, captures faites avec le navigateur de l'agent (même `pilote-qa.js`).
- PAS FAIT, la preuve qui manque : partie complète de la phase 2 par un pilote automatique, 5 graines, normal et facile. `carte pieuvre robot` lancé une fois : coupé à 600 s (recherche en cours, aucun parcours, aucun résultat). Ce qui manque au robot : la graine du cerveau n'est pas réglable depuis `Monde` ; le robot ne joue pas en facile ; sa case (`nuage_de_la_case`) ne connaît de la Pieuvre que phase, temps (63 tranches au plus), têtes : ni l'horloge de la mer, ni l'attaque en cours, ni l'humeur, donc la montée (plus de 60 s) se confond ; et sa recherche dépasse 600 s. Mesure `CLE=pieuvre` lancée une fois sous `timeout 600` : NE FINIT PAS (coupée, aucun chiffre).
- PAS testé : joueur humain, téléphone, mode facile en jeu (seulement les tests).

## Carte 9.13g : pilote automatique de la phase 2 de la Pieuvre, et mesure `CLE=pieuvre` (la preuve qui manquait à la 9.13f)
- Diagnostic (qui sait quoi avant cette carte) : `core::faisabilite` ne connaît pas le temps : pour la tour, la géante n'attaque pas et la mer ne monte pas (`Pieuvre::pour_le_robot`), il prouve seulement que la grille se grimpe. Le robot de QA (`game/src/bot_qa/parcours.rs`) cherche un plus court chemin (Dijkstra) entre des CASES de la vraie simulation ; une mécanique à horloge doit mettre son état dans la case, et celle de la Pieuvre (`nuage_de_la_case`) n'y met que phase, 63 tranches de temps et têtes. La montée dure plus de 60 s, et ni l'horloge de la mer, ni l'attaque en cours, ni l'humeur, ni le tirage du cerveau ne sont dans la case : deux états différents se confondent (la recherche écarte un bon chemin) ou se séparent par millions (elle ne finit pas : coupée à 600 s en 9.13f). Le joueur simulé de la mesure (`difficulte.rs`) part des itinéraires de la même recherche, sur le niveau entier : même mur. Aucun ne lit les annonces de la géante. Conclusion : la difficulté n'est pas le niveau, c'est de mêler l'espace (la tour, fixe) et le temps (la géante) dans une seule recherche.
- Fait : `game/src/bot_qa/pilote_pieuvre.rs` (module de test, rien dans le jeu publié ; choix expliqué en tête du fichier : il réutilise les plans et la simulation du robot de QA, et la mesure, elle aussi de test, s'en sert). Il sépare l'ESPACE et le TEMPS. (1) La ROUTE de la tour, calculée une fois (`OnceLock`) : la tour sans géante (`pour_le_robot`), explorée avec les plans du robot de QA depuis la lanterne du bas ; 253 cases posées, toutes reliées à la vigie, coût de chaque case jusqu'à la vigie (Dijkstra à l'envers). (2) À chaque décision, dans la VRAIE partie (`Monde::avancer`), il lit l'état public de la géante (`etat_frappe`, `etat_balayeur`, `etat_secousse`, `etat_arrachage`, `etat_jete`, `surface_mer`) : tête sonnée, il cherche le saut qui la prend ; sinon le saut de sa route qui le rapproche de la vigie sans se poser dans une zone annoncée et encore sûr 0,2 s après ; si aucun (une plateforme arrachée, un nuage tombé), tous les plans du robot essayés sur place ; sinon il attend ; sinon il s'écarte. Comme le joueur simulé de la mesure, il essaie chaque geste sur une copie du monde avant de le jouer : il voit donc une attaque choisie PENDANT son geste (1 s au plus ; un humain la voit à l'annonce, 0,8 s au moins). C'est sa seule avance sur un humain (il ne lit pas la graine). La vigie se joue avec les mêmes règles : les trois cycles sont vus et finis.
- Côté jeu, RIEN à ajouter : la graine se règle par `Pieuvre::fixer_graine` (déjà publique), le mode facile par `Monde::passer_nuage_en_facile` (celui du jeu), et la phase 1 est sautée par les deux appels publics du robot de faisabilité et des tests (`sortir_la_tete` puis `toucher`, autant de fois que la pieuvre a de points), Pépin posé à la lanterne du bas de la tour (`debut_phase_2`). Aucune règle, grille ni timing de la Pieuvre changés (`git diff` : seulement `game/src/bot_qa/` et `game/src/difficulte.rs`).
- LA PREUVE, `le_pilote_finit_la_phase_2` (ignoré, à lancer à la main : `PEPIN_FILS=5 timeout 600 ~/projects/pepin-notes/lourd.sh ./docker-cargo.sh test -p game --no-default-features --profile tests-rapides pilote_pieuvre::tests -- --ignored --nocapture --test-threads 1`, 44 s avec la route et le script) : du bas de la tour à `Vaincue`, 10/10 victoires, 0 mort. « Têtes » : prises pendant la montée (jetés) + sur la vigie ; « cycles » : un par jeté vif de la vigie, avec les têtes qu'il lui restait ; « vigie » compte toutes les étapes des cycles (jetés, frappe visée, balayage de la vigie) ; « combo » : les frappes visées enchaînées.

| mode | graine | victoire | morts | têtes montée+vigie | durée | attaques vues | cycles de vigie (têtes) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| normal | 1 | OUI | 0 | 1+2 | 84,7 s | frappe 24, combo 0, balayage 1, secousse 0, arrachage 1, jeté 2, vigie 4 | 2, 1 |
| normal | 2 | OUI | 0 | 1+2 | 64,1 s | frappe 16, combo 0, balayage 1, secousse 0, arrachage 2, jeté 1, vigie 4 | 2, 1 |
| normal | 3 | OUI | 0 | 0+3 | 63,3 s | frappe 14, combo 0, balayage 1, secousse 0, arrachage 2, jeté 1, vigie 6 | 3, 2, 1 |
| normal | 4 | OUI | 0 | 2+1 | 52,8 s | frappe 13, combo 0, balayage 1, secousse 0, arrachage 1, jeté 2, vigie 1 | 1 |
| normal | 5 | OUI | 0 | 2+1 | 79,6 s | frappe 25, combo 0, balayage 1, secousse 0, arrachage 1, jeté 2, vigie 1 | 1 |
| facile | 1 | OUI | 0 | 1+1 | 60,4 s | frappe 12, combo 1, balayage 2, secousse 0, arrachage 1, jeté 1, vigie 2 | 1 |
| facile | 2 | OUI | 0 | 1+1 | 45,3 s | frappe 8, combo 1, balayage 0, secousse 1, arrachage 1, jeté 1, vigie 2 | 1 |
| facile | 3 | OUI | 0 | 1+1 | 78,5 s | frappe 20, combo 0, balayage 0, secousse 0, arrachage 1, jeté 1, vigie 3 | 1 |
| facile | 4 | OUI | 0 | 1+1 | 48,2 s | frappe 8, combo 1, balayage 0, secousse 1, arrachage 2, jeté 1, vigie 2 | 1 |
| facile | 5 | OUI | 0 | 1+1 | 45,6 s | frappe 7, combo 1, balayage 2, secousse 0, arrachage 1, jeté 1, vigie 2 | 1 |

- Lecture : les trois cycles de vigie sont joués et finis (graine 3 normal : 3, puis 2, puis 1 tête) ; dès qu'une tête est prise pendant la montée, la vigie commence au cycle suivant (normal 1 et 2 : cycles à 2 puis 1 tête ; normal 4 et 5 : deux têtes prises en montée, seul le cycle à 1 tête, balayage puis jeté vif). Le pilote prend presque toujours le premier jeté de la montée : un humain qui l'évite verra plus de cycles. La frappe visée domine (6 à 25 par partie, plus il attend, plus il en voit) ; secousse et combo restent rares sur sa route (il passe vite sous les caisses). Durées de 45 à 85 s pour la phase 2 seule.
- Échecs : AUCUN dans le tableau final, donc pas de capture `913g-echec-*`. La PREMIÈRE version du pilote (il ne vérifiait pas qu'un appui restait sûr après s'y être posé, et ne quittait jamais sa route) gagnait aussi 10/10 mais avec 13 morts : 9 noyé sur la caisse de la rangée 19 (posé là où il ne pouvait plus repartir à temps, la mer l'a rattrapé), 3 balayé sur la vigie, 1 pendant une attente rangée 62. Toutes ont disparu en changeant le PILOTE (essai « sûr ensuite », repli sur tous les plans) sans toucher au niveau : c'était le pilote qui était faible, pas le niveau. Aucun passage sans issue trouvé.
- Capture `docs/qa/913g-pilote-vigie.png` : le jeu réel (build de QA JETABLE dans le scratch, `jetable-913g` : au chargement de « pieuvre », la phase 2 préparée comme `debut_phase_2(Some(1), false)`, puis les entrées du pilote rejouées pas à pas depuis `target/qa/913g-pilote-vigie.txt`, écrit par le test `ecrire_script_pilote_vigie` ; le monde figé à la fin du script). Pépin est sur la vigie, la géante à 2 têtes (HUD) se cabre pour le premier jeté du cycle à 2 têtes, la mer juste sous la lanterne du haut. Même endroit (x 81, pieds 112) et mêmes têtes que dans le test : la partie du navigateur est la même que celle du pilote. Regardée ; lecture exacte du tampon : 0 hors palette, 0 flou. Navigateur de l'agent, `docs/pilote-qa.js`, serveur arrêté et onglet fermé.
- LA MESURE `CLE=pieuvre` (`mesure_d_un_niveau_du_monde_2`) : elle FINIT, en 313 s (`PEPIN_FILS=5`, lancée une fois sous `timeout 600`). Pour la Pieuvre, elle est faite en deux phases (`mesure_de_la_pieuvre`) : la phase 1 dans l'arène du niveau d'essai `essai_pieuvre` (même arène, même pieuvre, sans tour) par le joueur simulé habituel (5 992 cases, 33 s avec la route), la phase 2 par le pilote dont chaque geste passe par `perturber` (les erreurs du profil), avec la graine du jeu (comme un joueur qui la découvre). Graine officielle, 48 essais par profil ; total = somme des deux phases (morts, durée, vies donc game over). Lieux de la phase 2 en RANGÉES (pieds de Pépin).
  - Phase 1 : Débutant 3,75 morts, 48/48 finis, 64,1 s, game over 17/48 ; Habitué 0,58, 43,1 s, game over 1/48 ; Expert 0,10, 39,6 s, 0/48.
  - Phase 2 : Débutant 8,19 morts, 45/48 finis (3 coupés à 6 min de jeu), 160,4 s, max 39, game over 28/48 ; Habitué 1,31, 48/48, 88,3 s, max 6, game over 2/48 ; Expert 0,42, 48/48, 71,9 s, max 2, 0/48. Où : bas de la tour surtout (Débutant : rangées 61 à 66, 210 morts sur 393 ; Habitué : rangée 62, 15 sur 63 ; Expert : rangée 62, 9 sur 20), puis la vigie (rangées 5 à 7).
  - Total : Débutant 11,94 morts, 45/48 finis, 224,5 s, game over 40/48 ; Habitué 1,90, 48/48, 131,4 s, game over 5/48 ; Expert 0,52, 48/48, 111,5 s, 0/48. En regard (9.12b, phase 1 seule à l'époque) : 3,40 / 0,73 / 0,29 avant, 1,60 / 0,54 / 0,42 après ; orage 3,5 / 0,8 / 0,4, cyclone 4,06 / 1,19 / 0,38.
  - Limites : le pilote « avec erreurs » rejoue les erreurs d'exécution de la mesure (élan, durée du saut, plané tardif), pas les erreurs de lecture d'un humain (il voit toujours les annonces, et une seconde d'avance) ; les morts du bas de la tour (une erreur près de la mer qui monte vite au début) ne sont pas analysées une par une dans le temps de la carte. Rien n'est changé au niveau : AGA décide.
- Preuves : `lint` LINT-OK ; `carte pieuvre` VERT (core 326, game 400, niveaux 15, tous 89 + faisabilité du niveau, 20 s) ; preuve du pilote et mesure ci-dessus (une fois chacune ; la preuve a aussi tourné pendant la mise au point du pilote).
- PAS testé : joueur humain, téléphone, mode facile en jeu ; la mesure en mode facile (la mesure n'a pas de mode facile).

## Carte 10.4a : les sprites deviennent des fichiers PNG dans `assets/sprites/`
- Fait : les 335 dessins faits à la main sont des PNG indexés (`assets/sprites/<famille>/<clé>.png`, 16 familles dont les 6 thèmes de `tuiles_monde2/`, 74 Ko en tout), SEULE source : plus aucune grille dans `game/src/sprites/*.rs` (-6 800 lignes ; chaque dessin y garde son commentaire de conception, `// \`clé.png\` : ...`). `game/build.rs` liste le dossier et embarque chaque fichier par `include_bytes!` ; `sprites/fichiers.rs` les décode une fois (crate `png`, celle que macroquad embarque déjà : `Image::from_file_with_format` de macroquad ajoutait 100 Ko de conversions de `image`) en grilles de texte, donc `Banque::charger`, l'éclaircissement par thème, les motifs, le miroir et les tests sont INCHANGÉS. Les 7 alias (`pepinette_mort2` = `pepin_mort2`, `eau_courant_*` = `eau_calme_*`) restent du code (`ALIAS`). Crate `outils` : `cargo run -p outils --bin sprite -- lire|ecrire <fichier>` (encodeur `png` hors du wasm, `game` n'en dépend pas) ; PNG canonique (palette fixe de 17 entrées, blocs IHDR PLTE tRNS IDAT IEND seulement). `assets/sprites/LISEZMOI.md`, README, `lint` (clippy `outils`) et `carte-du-code.py` (notes, .bak) à jour.
- Tests ajoutés : chaque PNG se décode sans pixel hors palette, n'a que les 5 blocs permis, aucun fichier orphelin ni famille vide ni alias doublé d'un fichier, aucun nom demandé par un appel `dessin*(…, "nom")` du code ne manque (plus de 50 noms vus), aller-retour de l'outil aux mêmes octets sur les 335 fichiers ; les tests de tailles et d'animations existants lisent maintenant les PNG. Équivalence : commit 1/2 (`838f77a`), test `chaque_png_est_identique_a_sa_grille_rust` VERT (335 sprites, noms et pixels), puis grilles supprimées au commit 2/2 (le test part avec elles). `docs/planche-sprites.png` refaite par le test à partir des PNG (même taille, ordre alphabétique dans chaque famille).
- Preuves : `lint` LINT-OK ; `test --workspace --no-default-features` VERT (core 326, game 404, niveaux 15 + 84, outils 4, robot 4) et game avec `debug_niveaux,qa_plantage` 403 ; `carte` toits, grotte, port, pieuvre VERTES. Captures (`docs/capture-104a.sh` et `capture-102.sh`, build de QA avant `cd2a559` et après) : 27/27 IDENTIQUES, 0 pixel différent (`comparer-captures.py` par paires) : titre, carte des niveaux, boutique, le départ des 18 premiers niveaux de la table (tous les thèmes des deux mondes, n00 à n17), toits/port/Pieuvre de 10.2, Pieuvre phase 1 à 151, 451 et 1 051 images ; 0 erreur console. Gardées : `docs/qa/104a-apres-*` (titre, boutique, n00 toits, n10 caniveau, n13 grotte, n17 port, pieuvre-b).
- `./build.sh` : wasm 971 277 → 1 059 150 octets (+87 873, +86 Ko, sous la limite de 100 Ko ; gzip 436 066), 0 nom interdit dans les wasm (aucun nom, aucun chemin /home : recherche dans dist/ et dist-qa/) ; `--qa` OK (995 507 → 1 083 060, aller-retour éditeur 33/33). La hausse vient du décodeur PNG (inflate), absent jusqu'ici du wasm.
- Démarrage (`node docs/mesure-demarrage.mjs PORT 3`, dist/ de production, navigateur neuf sans cache, SwiftShader, deux séries alternées) : écran titre dessiné à 599 et 615 ms avant, 627 et 637 ms après (+12 à +38 ms) ; plus longue des 10 premières images 87-91 ms avant, 103-107 ms après (le décodage des 335 PNG, +15 ms une fois). Pas visible.
- Pas testé : la MONTÉE de la Pieuvre en capture (il faudrait gagner la phase 1 : robot de partie complète, interdit ; ses sprites sont couverts par l'équivalence pixel à pixel) ; joueur humain, téléphone, Safari.

## Carte 10.4b : vignettes des vrais sprites dans l'éditeur de niveaux
- Fait : l'éditeur (`editeur/`, build de QA seulement) montre, dans sa palette ET dans la grille, la vignette du vrai sprite de 51 lettres (sur 78), lue dans `dist-qa/sprites/<famille>/<clé>.png` (44 PNG copiés de `assets/sprites` par `--bin editeur`, lancé par `./build.sh --qa`). Le lien lettre → sprite est la table `VIGNETTES` de `niveaux/src/legende.rs` (lettre, CLÉ du sprite, ancrage) ; la famille, donc le chemin, vient de la table `niveaux::SPRITES` écrite par `niveaux/build.rs` d'après les fichiers (aucun chemin recopié), et `legende.json` le donne (champs `sprite` et `ancre`, `null` sinon). Les 27 lettres sans sprite propre (tuiles dessinées par thème avec leurs bords calculés, zones de vent, courant, pluie, marée, mobilier de rue dessiné par code, repères de la Pieuvre) gardent leur case colorée. Ancrage comme dans le jeu : en bas et centré sur la place de la lettre (règle `coin_decor`, voiture et boss compris), en haut pour la stalactite, coin haut-gauche pour les plateformes (friable, mobile, nénuphar), centré pour la goutte, la lettre, les bulles, le nuage de grêle et le cadran ; un sprite plus grand qu'une case est dessiné PAR-DESSUS les voisines, avec un léger contour blanc de la case d'origine ; la lettre reste en petit badge en bas à droite (case « lettres sur les cases »). Bouton « vignettes / cases » (touche V, retenu dans le `localStorage`) ; zoom net : `imageSmoothingEnabled = false` sur la grille, `image-rendering: pixelated` et agrandissement entier dans la palette. `./build.sh` refuse aussi un dossier `sprites` dans `dist/`. LISEZMOI des sprites et README à jour.
- Test ajouté : `chaque_vignette_est_un_vrai_sprite` (chaque clé a son fichier, une vignette par lettre au plus, lettre de la légende, chemin dans `legende.json`, tuiles sans vignette).
- Preuves : `lint` LINT-OK ; `test --workspace --no-default-features` VERT (core 326, game 404, niveaux 16 + 84, outils 4, robot 4) ; `./build.sh` OK (éditeur et `sprites/` absents de dist/, wasm 1 059 150 octets INCHANGÉ : le jeu ne bouge pas, rien à capturer côté joueur) et `--qa` OK (« 78 lettres, 44 vignettes », aller-retour 33/33 mêmes octets). `CHROME=…/chromium-1208 node docs/capture-104b.mjs PORT docs/qa` sur dist-qa : VIGNETTES-OK (51 images chargées, 51 boutons en vignette, 27 cases colorées gardées, bascule cases/vignettes : la case d'une mouette passe de 0 % à 62 % de sa couleur de légende et retour, lissage coupé, export de port et grotte = fichier, 0 erreur de console). Captures regardées, zoom 32 : `docs/qa/104b-vignettes-port.png` (crabes sur les toits, lanterne, poisson, plateformes mobiles 6 et 7 sur l'eau, corbeau) et `104b-vignettes-grotte.png` (stalactites accrochées au plafond, colonnes de bulles, drapeau, goutte) : pixels nets, rien de décalé.
- Pas testé : joueur humain, téléphone, Safari ; Firefox (le rendu pixelisé du canvas vient de `imageSmoothingEnabled`, standard).

## Carte 10.4c : les fonds du Monde 1 deviennent des fichiers PNG (`assets/fonds/`)
- Fait : inventaire et pile de dessin dans `assets/fonds/LISEZMOI.md`. Couches FIXES devenues PNG indexés (palette + transparent, 1 pixel = 1 pixel), seule source du dessin : les ciels en bandes (`<thème>/ciel.png`, UNE colonne de 180 px : la période horizontale d'un ciel est d'un pixel, la répétition sur 320 colonnes reste du code), le ciel du voyage de la descente (`descente/voyage.png`, 1 x 450), le soleil (`soleil/soleil.png`, 45 x 45), l'arc-en-ciel du bonus (`arc_en_ciel/arc.png`, 300 x 150). Les ciels du Monde 2 qui passent par le même rendu (caniveau, cascade, grotte, rivière, marée) sont des fichiers aussi (un seul chemin de dessin). Les couleurs des bandes (`Ambiance::ciel`, `voyage()`) sont LUES dans les fichiers ; supprimés : le tracé des bandes et de leur trame, des bandes du voyage, du disque du soleil, de l'arc (`bande_arc`, `ARC`, `VOYAGE`, `RAYON_ARC`...). `game/build.rs` embarque `assets/fonds` comme `assets/sprites` (`include_bytes!`, aucune liste à la main), `game/src/fonds.rs` les décode une fois (décodeur de 10.4a) et les pose en textures « au plus proche ». Fichiers écrits avec l'outil `sprite` (export par un test temporaire retiré, relecture `sprite lire` identique à la grille exportée pour les 18).
- Pile de dessin UNIQUE : `decor::pile(thème)` (enum `Couche`, PNG ou CALCULÉ, plus le marqueur NIVEAU) est la liste que suivent `dessiner_fond` (jusqu'à NIVEAU) et `dessiner_avant` (après) ; le test `la_pile_documentee_est_celle_du_rendu` la compare, thème par thème (19), au bloc du LISEZMOI.
- Reste du code (calculé, ou d'après la grille, ou dans le doute) : étoiles (scintillent), pluie, éclairs, mur de pluie et soleil de la poursuite, grêle, silhouettes, plafonds de nuages, îlots et fenêtres (semés au hasard par cellule, sans période : un PNG couvrirait chaque niveau sur toute sa largeur), tuiles éclaircies, déchirure, façades. Liste détaillée dans le LISEZMOI.
- Tests ajoutés : chaque couche se décode sans pixel hors palette (lignes de même largeur), n'a que IHDR/PLTE/tRNS/IDAT/IEND, 16 Ko au plus ; aucune couche PNG d'une pile ne manque, aucun fichier orphelin, taille attendue de chaque couche ; la pile du LISEZMOI ; l'arc (huit bandes jointives, distinctes, dans la palette, symétriques, sommet en y = 36) vérifié sur le fichier ; les tests existants du ciel, du voyage et des contrastes lisent maintenant les PNG.
- Poids (plafonds : +120 Ko net, 16 Ko par couche) : 18 couches, 4 635 octets en tout ; wasm de production 1 059 150 → 1 067 866 octets (+8 716, +8,5 Ko NET, la pile et ses textures comprises ; gzip 440 558) ; aucun fichier servi en plus. QA 1 083 060 → 1 091 774.

| Couche | Dimensions | Octets |
| --- | --- | --- |
| `toits/ciel.png` | 1 x 180 | 179 |
| `gouttieres/ciel.png` | 1 x 180 | 179 |
| `marche/ciel.png` | 1 x 180 | 181 |
| `orage/ciel.png` | 1 x 180 | 180 |
| `clocher/ciel.png` | 1 x 180 | 181 |
| `cyclone/ciel.png` | 1 x 180 | 179 |
| `altitude/ciel.png` (vents, poursuite) | 1 x 180 | 183 |
| `mer/ciel.png` | 1 x 180 | 182 |
| `soleil/ciel.png` | 1 x 180 | 182 |
| `arc_en_ciel/ciel.png` | 1 x 180 | 181 |
| `caniveau/ciel.png` | 1 x 180 | 176 |
| `cascade/ciel.png` | 1 x 180 | 183 |
| `grotte/ciel.png` | 1 x 180 | 173 |
| `riviere/ciel.png` | 1 x 180 | 182 |
| `maree/ciel.png` | 1 x 180 | 182 |
| `descente/voyage.png` | 1 x 450 | 196 |
| `soleil/soleil.png` | 45 x 45 | 296 |
| `arc_en_ciel/arc.png` | 300 x 150 | 1 440 |

- Preuves : `lint` LINT-OK ; `test --workspace --no-default-features` VERT (core 326, game 409, niveaux 16 + 84, outils 4, robot 4) et game avec `debug_niveaux,qa_plantage` 408 ; `carte toits`, `carte orage`, `carte cyclone` VERTES. Captures (`docs/capture-104c.sh`, build de QA du début de la carte puis de la fin, `comparer-captures.py` par paires) : 30/30 IDENTIQUES, 0 pixel différent : titre, carte des niveaux, boutique, départ ET milieu (150 images de course : parallaxe) des 11 premiers niveaux du Monde 1, la descente au départ et à quatre hauteurs (voyage remonté). Sur chacune, des couches calculées passent ENTRE les couches PNG et le niveau (étoiles devant le ciel PNG des toits, plafonds de nuages devant celui de l'orage et du cyclone, silhouettes devant le soleil PNG et l'arc PNG, îlots et étoiles devant le voyage PNG), et la pluie devant tout. `docs/capture-104a.sh` refait aussi sur les deux builds : 24 captures dont le départ des 7 niveaux du Monde 2 (ciels du caniveau, de la cascade, de la grotte, de la rivière, de la marée) et la Pieuvre, identiques. Gardées : `docs/qa/104c-apres-*.png` (titre, toits, orage, cyclone, nuages au soleil, arc-en-ciel, descente).
- Note : la capture du TITRE seule est parfois décalée d'une image d'animation au premier lancement de Chromium (vu sur les DEUX builds, avant comme après, et sur `capture-104a.sh` aussi) ; relancée, elle est identique au pixel. Ce n'est pas le rendu.
- `./build.sh` OK, 0 nom interdit (aucun nom ni chemin /home dans dist/ et dist-qa/) ; `--qa` OK (aller-retour éditeur 33/33).
- Démarrage (`node docs/mesure-demarrage.mjs PORT 3`, dist/ de production, navigateur neuf, deux séries alternées) : écran titre 629 et 630 ms avant, 658 et 656 ms après (+26 à +28 ms) ; plus longue des 10 premières images 102-108 ms avant, 125-131 ms après (le décodage des 18 couches et leurs textures, une fois). Même ordre que 10.4a, pas visible.
- PAS fait (temps de la carte dépassé, arrêt propre) : la carte des niveaux (`ecrans/fond_plan.rs` : ciel à bandes de hauteur variable fondues, nuages placés en proportion de la carte ; les dessins `NUAGES` sont des candidats PNG), à faire dans une carte suivante (sa capture est identique : son code n'a pas bougé). Capture de l'ÉCRAN DE FIN non prise : aucune touche de QA n'y mène sans finir la descente (un robot de partie complète est interdit, et le parcours scripté s'arrête au 2e étage) ; son fond est `dessiner_fond(Descente, x, 0)`, la remontée 0 du voyage (les quatre bandes du bas, le ciel des toits) : même couche et même placement que les captures de la descente, sans capture propre. Monde 2 (égouts, port, cascade, rivière, rue) : carte 10.4d.
- Pas testé : joueur humain, téléphone, Safari.

## Carte 10.4d (1re étape) : les fonds des ÉGOUTS et du PORT deviennent des fichiers PNG (`assets/fonds/egouts`, `assets/fonds/port`)
- Fait : les deux fonds du Monde 2 qui étaient une seule couche opaque dans la pile (`FondDesEgouts`, `FondDuPort`) sont découpés en PLANS, un PNG par couche FIXE, jamais d'image aplatie. Égouts (`fond_egouts::Plan`, `PILE`) : mur PNG (une colonne), arches PNG (UNE arche, sa période de 112 px), piliers CALCULÉ (coulures et grilles semées au hasard, gouttes), eau CALCULÉ (reflets qui tremblent), tuyaux CALCULÉ (semés au hasard). Port (`decor_port::Plan`, `PILE`) : crépuscule PNG (colonne), étoiles CALCULÉ, mouettes CALCULÉ, mer PNG (colonne), reflets CALCULÉ, large CALCULÉ (cargo qui glisse, phare, faisceau), port d'en face PNG (UNE période de 640 px : entrepôts, fenêtres, conteneurs, trois grues, quai), balises CALCULÉ, voiliers CALCULÉ (houle), pontons PNG (UNE période de 640 px : platelage, pieux, filets, lampadaires), bouées CALCULÉ (bouées et reflets des halos).
- `decor::pile` reprend ces deux piles telles quelles (`Couche::Egouts(plan)`, `Couche::Port(plan)`) ; `decor_port::dessiner_fond` (utilisé sous la tour de la Pieuvre) suit la MÊME `PILE`. La scène ne fait plus d'exception pour le port : elle passe par `decor::dessiner_fond` comme les autres thèmes (la jetée de la Pieuvre garde `dessiner_fond_sous_la_tour`). Bloc du LISEZMOI mis à jour, vérifié par `la_pile_documentee_est_celle_du_rendu`.
- Code de dessin supprimé : l'arche (`dessiner_arche`, `ouverture`, `anneau`, `demi_corde`), le ciel du port (`CIEL`, `ciel_a`), les entrepôts, conteneurs et grues (`ENTREPOTS`, `CONTENEURS`, `portique`), le fond uni de la mer, les pontons fixes. Les PNG ont été tirés UNE fois de ce code (générateur jetable, non gardé), puis relus par `sprite lire` : aller-retour identique.
- Tests : `les_arches_du_fichier_sont_pleines` (remplace `les_arches_sont_pleines`, vérifié sur le fichier), `le_ciel_va_de_la_nuit_a_la_lueur_de_l_horizon` et `les_cables_de_la_grue_vont_de_la_fleche_au_palonnier` lisent désormais les PNG ; table des tailles de `aucune_couche_png_manquante_ni_orpheline` étendue ; palette, blocs permis et 16 Ko vérifiés pour les 6 nouveaux fichiers.
- Ordre vérifié sans chevauchement : les pièces calculées qui passent ENTRE deux PNG ne recouvrent jamais une pièce fixe dessinée après elles dans l'ancien code (piliers entre deux arches, feux au-dessus des cabines, bouées et reflets devant les pieux, motifs de 640 px sans recouvrement d'une copie à l'autre dans l'écran).
- Poids (plafonds : +120 Ko NET, 16 Ko par couche) : 6 couches, 2 573 octets ; wasm de production 1 067 866 → 1 067 460 octets (-406 octets NET, gzip 440 558 → 441 024) ; QA 1 091 774 → 1 091 368. Aucun fichier servi en plus.

| Couche | Dimensions | Octets |
| --- | --- | --- |
| `egouts/mur.png` | 1 x 180 | 173 |
| `egouts/arches.png` | 112 x 88 | 478 |
| `port/crepuscule.png` | 1 x 56 | 179 |
| `port/mer.png` | 1 x 124 | 172 |
| `port/port_d_en_face.png` | 640 x 60 | 875 |
| `port/pontons.png` | 640 x 92 | 696 |

- Preuves : `lint` LINT-OK ; `test --workspace --no-default-features` VERT (core 326, game 409, niveaux 16 + 84, outils 4, robot 4) et game avec `debug_niveaux,qa_plantage` 408 ; `carte caniveau`, `carte grotte`, `carte port`, `carte pieuvre` VERTES. Captures (`docs/capture-104d.sh`, build de QA du début de la carte puis de la fin, `comparer-captures.py` par paires) : 20/20 IDENTIQUES, 0 pixel différent : départ et milieu des 7 niveaux du Monde 2 (caniveau, égouts, cascade, grotte, rivière, port, Pieuvre), la Pieuvre 300 et 900 images plus tard, et la MONTÉE à deux hauteurs (bas de la tour : le fond de l'arène décalé entre par le bas de l'écran ; haut : nuit et port lointain), chacune deux fois (a, b). Pour la montée, `capture-102.mjs` accepte `ESSAI_RON` (niveau posé dans le `localStorage`, page ouverte avec `?essai=1`) : le niveau de la Pieuvre, départ déplacé dans la tour, sans build jetable. Exemples : `docs/qa/104d-apres-*.png`.
- La planche des fonds (touche V) n'est pas une preuve : deux passages du MÊME build y diffèrent (son pas compte les images réelles du navigateur) ; elle est hors du script.
- `./build.sh` et `./build.sh --qa` OK, 0 nom interdit (aucun nom ni chemin /home dans dist/, dist-qa/ et assets/fonds).
- Démarrage (`node docs/mesure-demarrage.mjs PORT N`, dist/ de production, séries alternées) : écran titre 623, 678, 803, 702 ms avant ; 688, 714, 831, 654 ms après. L'écart entre deux séries du même build (jusqu'à 180 ms) dépasse l'écart avant / après : pas de ralentissement visible.
- RESTE (pas fait dans cette étape) : la rue du caniveau (`decor_rue`, `mobilier_rue`), la grande chute de la cascade (`decor_cascade::dessiner_fond`), la vallée de la rivière (`decor_riviere::dessiner_fond`), la grotte (`decor_grotte`), la Pieuvre (`dessin_port::dessiner_fond`) et la montée (`dessin_montee`), les motifs du Monde 2 (`decor::dessiner_detail`). Attention : la chute de la cascade et la vallée de la rivière sont dessinées par la SCÈNE après `decor::dessiner_fond` (pas dans `decor::pile`) ; il faudra d'abord les faire entrer dans la pile (ou une pile de scène testée de la même façon) avant d'en tirer des PNG. Le premier cargo et le phare lointain du port (plan « large ») restent du code dans le doute (un seul exemplaire, pas une période). Notes pour la rue : les façades ne sont PAS une période (les fenêtres allumées dépendent du numéro de tour, `maison(.., tour * 8 + k)`), elles restent du code ou il faut séparer les maisons nues (PNG) des fenêtres (calculées) ; les lampadaires sont posés en UN exemplaire chacun modulo 320 px (`(lx - dec).rem_euclid(L)`, coupés au bord droit, sans copie à gauche) : une répétition de PNG ne serait pas identique au pixel, il faut un PNG d'un lampadaire posé deux fois par le code. Toujours ouverts depuis 10.4c : la carte des niveaux (`ecrans/fond_plan.rs`) et une capture de l'écran de fin.
- Pas testé : joueur humain, téléphone, Safari.

## Carte 10.4e : les musiques et les bruitages deviennent de vrais fichiers audio (`assets/musiques`, `assets/sons`) : OGG d'abord (bloqué), puis TOUT EN WAV (décision d'Aga)
- Fait (1re étape, commit `ef0283c`) : chaque musique (6) et chaque bruitage (12, plus la pluie) rendus UNE fois par l'ancien synthé en WAV 16 bits (test ignoré `audio::tests::rendre_les_wav_de_reference`, WAV dans `target/sons-reference` seulement), puis encodés par `outils/sons.sh` (ffmpeg pris dans la variable `FFMPEG`, jamais écrit dans le dépôt) : OGG Vorbis MONO, 22 050 Hz (la fréquence du synthé), qualité 4 pour les musiques et 6 pour les bruitages, pages OGG de 1 s et de 10 ms. Outil de contrôle `outils son controler <ogg> [<wav>]` (décodeur `lewton`, déjà dans le Cargo.lock par macroquad) et tests du crate `outils` : mono, 22 050 Hz, AUCUN commentaire Vorbis, aucun mot interdit (`Lavf`, `Lavc`, `encoder`, `/home`...), plafonds de taille, raccord des boucles.
- Fait (2e étape) : `game/build.rs` embarque tous les `.ogg` de `assets/musiques` et `assets/sons` (aucune liste à la main) ; `audio::Audio::charger` les confie à `load_sound_from_bytes` (aucune dépendance nouvelle dans le jeu) ; test `chaque_son_demande_a_son_fichier` (chaque effet, musique et la pluie ont leur fichier, aucun orphelin). SUPPRIMÉS : les partitions (`Partition`, `Voix`, les 5 partitions et leurs timbres de `game/src/musiques.rs`, 1 108 → 258 lignes), les réglages des bruitages, la mélodie d'Orage et la pluie de `core/src/sons.rs` (qui ne garde que `Effet`, `cle`, `volume`), et `pepin_core::synthese` en entier : sans partition, l'outil de rendu n'a plus rien à rendre, il est donc retiré aussi (les partitions restent dans l'historique, commit `ef0283c`). `assets/musiques/LISEZMOI.md` (format, règles, remplacer un son). README mis à jour.
- Réglages trouvés en route : (1) ffmpeg 9 écrit un commentaire `encoder=Lavc libvorbis` MÊME avec `-fflags +bitexact -map_metadata -1` : il faut `-metadata:s:a:0 encoder=` (vérifié : 0 commentaire, 0 `Lavc`/`Lavf` dans les 19 fichiers ; reste le champ fabricant `ffmpeg`, obligatoire dans l'en-tête Vorbis) ; (2) un bruitage court en une seule page OGG était mal coupé par ffmpeg (256 à 500 échantillons de trop ou de moins), et `lewton` se trompait encore avec des pages de 20 à 200 ms : à 10 ms, ffmpeg ET lewton décodent exactement la longueur du WAV pour les 19 sons.
- Qualité : 30 dB de signal/bruit pour les musiques demandaient la qualité 6 (702 Ko au total, plus de 600 Ko) ; la qualité 5 fait 639 Ko. Choix : musiques en qualité 4 (25,6 à 27,4 dB), bruitages en 6. Le signal/bruit mesure l'écart de FORME d'onde : sur des carrés 8 bits (et du bruit, pour la pluie, l'éclair, le vol plané) le Vorbis remplace une partie du signal par un équivalent qui s'entend pareil, d'où les chiffres bas de `pluie` et `plane`. Pas d'écoute humaine faite.

Tableau d'équivalence (référence : le WAV rendu par l'ancien code ; « lewton » = décodé par le crate `outils`, comme ffmpeg ; « Chromium » = `decodeAudioData` dans un OfflineAudioContext à 22 050 Hz, échantillons de TROP à la fin) :

| Fichier | Octets | Durée | Éch. WAV | Écart lewton | Écart Chromium | Signal/bruit |
| --- | --- | --- | --- | --- | --- | --- |
| `musiques/cyclone.ogg` | 62 626 | 9.600 s | 211680 | 0 | +416 | 26.1 dB |
| `musiques/orage.ogg` | 61 290 | 9.600 s | 211680 | 0 | +160 | 27.3 dB |
| `musiques/poursuite.ogg` | 81 740 | 11.520 s | 254015 | 0 | +65 | 26.0 dB |
| `musiques/soleil.ogg` | 68 885 | 11.520 s | 254016 | 0 | +320 | 26.1 dB |
| `musiques/toits_de_nuit.ogg` | 73 873 | 11.520 s | 254016 | 0 | +320 | 27.4 dB |
| `musiques/valse_des_nuages.ogg` | 69 200 | 10.560 s | 232847 | 0 | +497 | 25.6 dB |
| `sons/atterrissage.ogg` | 4 640 | 0.070 s | 1543 | 0 | +121 | 26.2 dB |
| `sons/eclair.ogg` | 10 376 | 0.750 s | 16537 | 0 | +231 | 21.4 dB |
| `sons/ecrase.ogg` | 4 902 | 0.120 s | 2646 | 0 | +42 | 24.1 dB |
| `sons/goutte.ogg` | 6 016 | 0.230 s | 5071 | 0 | +177 | 25.9 dB |
| `sons/lanterne.ogg` | 6 526 | 0.400 s | 8820 | 0 | +268 | 36.3 dB |
| `sons/menu.ogg` | 4 516 | 0.050 s | 1102 | 0 | +178 | 31.4 dB |
| `sons/mort.ogg` | 8 618 | 0.550 s | 12127 | 0 | +33 | 23.9 dB |
| `sons/plane.ogg` | 5 373 | 0.160 s | 3528 | 0 | +56 | 16.1 dB |
| `sons/pluie.ogg` | 19 519 | 2.000 s | 44100 | 0 | +188 | 14.7 dB |
| `sons/rebond.ogg` | 5 774 | 0.220 s | 4851 | 0 | +13 | 20.3 dB |
| `sons/saut.ogg` | 5 275 | 0.140 s | 3087 | 0 | +369 | 23.5 dB |
| `sons/secret.ogg` | 6 257 | 0.360 s | 7937 | 0 | +127 | 31.6 dB |
| `sons/victoire.ogg` | 11 649 | 0.830 s | 18301 | 0 | +259 | 27.7 dB |

- TOTAL de l'audio : 517 055 octets (musiques 417 614, bruitages 99 441), sous les 600 Ko.
- LE PROBLÈME (raison du blocage) : Chromium NE COUPE PAS le remplissage de fin du Vorbis. Le son décodé par le navigateur est arrondi au multiple de 128 échantillons suivant ou plus (colonne « Écart Chromium » : +13 à +497 échantillons, 0,6 à 22,5 ms), et ce surplus est du SILENCE à la fin. Pour un bruitage, rien ne s'entend. Pour une boucle, c'est un blanc au point de boucle à CHAQUE tour : 7 à 22 ms dans les musiques (qui finissent déjà dans le silence : le rythme « boite » un peu à chaque tour), et 8,5 ms de silence toutes les 2 s dans la PLUIE (bruit continu : un trou, probablement audible). ffmpeg et lewton, eux, coupent juste (écart 0). Les tests du dépôt ne le voient pas (ils décodent avec lewton) ; la mesure Chromium est faite à la main. Aucune longueur de boucle actuelle n'est un multiple de 128 : un réglage de l'encodeur ne suffit pas.
- Pistes, à choisir par Aga : (a) accepter ces blancs ; (b) allonger chaque boucle de silence jusqu'au multiple de 128 suivant (au plus 5,8 ms de plus par tour, la boucle ne tombe plus pile sur la mesure) et vérifier que Chromium la décode alors juste, la pluie étant re-rendue sur une période multiple de 128 ; (c) garder la pluie en WAV (88 Ko, 2 s de bruit) et les musiques en OGG avec (a) ou (b) ; (d) revenir à l'ancien synthé pour les boucles seulement.
- Preuves : `lint` LINT-OK ; tests VERTS (`test -p game --no-default-features` 410 ; `-p core` 314, soit 326 moins les 13 tests du synthé et des sons supprimés plus 1 ; `-p niveaux` 16 + 84 ; `-p outils` 8 ; `-p robot` 4) ; `carte toits`, `carte pieuvre` VERTS ; `outils son controler` sur les 19 fichiers avec leur WAV : écart 0, aucune COUTURE au sens du test (saut au point de boucle sous les sauts du son lui-même).
- `./build.sh` OK : wasm de production 1 067 460 → 1 576 916 octets (+509 456 : l'audio embarqué, 517 055, moins 7 599 octets de code de synthèse retirés ; gzip 441 024 → 899 504, l'OGG ne se compresse pas) ; `--qa` OK (1 600 824, aller-retour éditeur 33/33). 0 nom interdit (aucun `lucas`, `/home`, `Lavf`, `Lavc`, `encoder=` dans dist/, dist-qa/, et les OGG).
- Démarrage (`node docs/mesure-demarrage.mjs PORT 5`, servi en local, navigateur neuf ; la sonde compte maintenant les sons décodés par le navigateur, le jeu les attend tous avant l'écran titre) : sons tous décodés à 1 367 ms (médiane) avant, 1 245 ms après : PAS plus lent en local (décoder 19 OGG coûte moins que synthétiser au démarrage). Mais le wasm à télécharger passe de 441 à 900 Ko compressés : sur une connexion lente (10 Mbit/s), environ 0,37 s de plus ; pas mesuré sur un vrai réseau.
- Écoute de contrôle dans Chromium sans écran : 19/19 sons décodés par `decodeAudioData`, 0 échec de décodage, avant comme après (sonde de `mesure-demarrage.mjs`). Pas d'écoute humaine.
- Pas testé : joueur humain, téléphone, SAFARI (qui lit mal certains OGG Vorbis selon la version : le jeu y serait peut-être muet), Firefox (qui, lui, coupe normalement le remplissage du Vorbis).

### Reprise de 10.4e : tout en WAV, servis à côté du jeu

Décision d'Aga sur le blocage, mot pour mot : « tout en wav plutôt histoire que ça marche aussi proprement sur iphone ? ». Les étapes OGG ci-dessus (commits `ef0283c`, `58ebfa5`) restent dans l'historique ; leurs OGG, `outils/sons.sh` (ffmpeg) et le décodeur `lewton` sont retirés.

- Fait : les 19 sons sont des WAV PCM 16 bits MONO à 22 050 Hz, CANONIQUES (en-tête de 44 octets : `RIFF`, `fmt ` de 16 octets, `data`, rien d'autre : aucun bloc `LIST`/`INFO`, aucun octet après les données) : `assets/musiques/<id>.wav` (6) et `assets/sons/<clé>.wav` (12 + `pluie`). Ce sont, octet pour octet, les rendus de l'ancien synthé (test ignoré `rendre_les_wav_de_reference`, rejoué sur une extraction de `ef0283c` : `cmp` identique pour les 19).
- Le jeu ne les EMBARQUE PAS : `game/build.rs` n'écrit plus que leur LISTE (genre, clé) ; `./build.sh` les copie dans `dist/musiques/` et `dist/sons/` (et `dist-qa/`). `audio.rs` : `Audio::new()` ne charge rien ; `Audio::avancer()`, appelé APRÈS chaque `next_frame` (donc à partir de la 2e image : l'écran titre n'attend rien), télécharge les sons par coroutines de macroquad (`load_sound`), 3 à la fois, dans l'ordre `Piste::dans_l_ordre` (bip des menus, musique Orage, pluie, puis bruitages, puis autres musiques). Un son absent est MUET (`Option`/`HashMap::get` partout, plus aucun `[]` qui paniquerait) ; une boucle qui arrive son actif démarre aussitôt. Un fichier introuvable est écrit dans la console (`error!`) et le jeu continue : vérifié en servant un `dist/` SANS `musiques/` (6 erreurs `DownloadFailed` dans la console, la pluie et les bruitages jouent, aucune panique). Hors du web, lecture directe de `assets/`.
- Tests : `audio::tests::chaque_son_demande_a_son_fichier` (chaque effet, musique et la pluie a son `.wav`, une seule fois, aucun orphelin), `le_titre_se_charge_en_premier`, `chaque_son_est_un_wav_mono_16_bits_sans_metadonnees` (en-tête canonique, 22 050 Hz, mono, 16 bits, rien après les données) ; crate `outils` : `lire_wav` (refuse stéréo, autre fréquence, bloc `LIST` ajouté, OGG), `chaque_son_est_un_wav_mono_sans_metadonnees` (et aucun autre fichier que `.wav`/`.md` dans ces dossiers), `chaque_boucle_se_raccorde_sans_couture`, et l'ÉQUIVALENCE `chaque_son_est_le_rendu_du_synthe_d_origine` : longueur exacte et empreinte FNV-1a des échantillons de chacun des 19 fichiers, relevées sur le rendu du synthé (identiques à l'échantillon près). Outil `son controler <wav>...` (taille, durée, raccord ; erreur si le WAV n'est pas canonique).
- `./build.sh` : chaque WAV de `dist/` est vérifié (en-tête RIFF/fmt/data, aucun nom interdit `lucas|tixier|elsay|georges|jarvis|/home/`), au moins 19 fichiers, plafond 3,5 Mo. `~/projects/pepin-notes/publier-prod.sh` (hors dépôt, sauvegarde `.bak-20261124-…`) : contrôle des noms sur wasm + WAV, copie de TOUT `dist/` (`cp -r dist/.`) vers `www`, et compare le sha de `musiques/orage.wav` en ligne. `web/index.html` inchangé (les fichiers sont lus par XHR, même origine : la CSP `connect-src 'self'` du site les permet).

| Fichier | Octets | gzip -9 | Durée | Échantillons |
| --- | --- | --- | --- | --- |
| `musiques/` (6) | 2 836 772 | 2 174 979 | 9,6 à 11,52 s | 211 680 à 254 016 |
| `sons/` (12 + pluie) | 259 872 | 224 452 | 0,05 à 2 s | 1 102 à 44 100 |
| TOTAL | 3 096 644 | 2 399 431 | | |

- Poids : 3 096 644 octets bruts (sous le plafond de 3,5 Mo), 2 399 431 compressés par gzip -9 (les carrés 8 bits et le bruit se compressent mal). ATTENTION, le serveur du site (`elsay-site/nginx.conf`) ne compresse PAS l'audio (`gzip_types` : wasm, js, css, svg seulement) : le navigateur téléchargera 3,1 Mo, en arrière-plan. Y ajouter le type des `.wav` (selon ce que nginx leur donne) serait une modification de la config de production : NON faite, à décider.
- wasm de production : 1 067 460 octets avec le synthé (`ef0283c`) → 1 066 268 (−1 192 ; gzip 441 024 → 440 253) ; l'étape OGG le portait à 1 576 916. `--qa` : 1 090 313, aller-retour éditeur 33/33. 0 nom interdit dans `dist/game.wasm` et les 19 WAV.
- Démarrage (`node docs/mesure-demarrage.mjs PORT 5`, local, navigateur neuf, `CHROME=…/chromium-1208`). La sonde mesure maintenant le PREMIER DESSIN (premier appel de dessin WebGL, y compris `drawElementsInstancedANGLE` de miniquad) : avant, les images d'attente des sons étaient vides et la colonne « titre » ne voyait rien. Médianes : AVANT (`ef0283c`, synthé au démarrage) wasm 312 ms, premier dessin 992 ms, sons prêts 967 ms ; APRÈS wasm 293 ms, premier dessin 595 ms, sons prêts (19/19 téléchargés et décodés) 965 ms. L'écran titre apparaît donc environ 400 ms PLUS TÔT (il n'attend plus la synthèse des sons). Plus longue image : 78 ms avant, 105 ms après (médianes ; d'une série à l'autre, de 77 à 204 ms, avant comme après). En local seulement : sur un vrai réseau, les sons arrivent après le titre, plus ou moins vite selon le débit.
- Écoute de contrôle (Chromium sans écran) : 19/19 sons décodés, 0 échec (`mesure-demarrage.mjs`, longueurs décodées = longueurs des WAV, ×2 au rééchantillonnage 44 100 Hz du navigateur : 88 200 pour la pluie). Pluie enregistrée 10 s DEPUIS LE JEU (`node docs/ecoute-pluie.mjs PORT` : la sonde branche un enregistreur sur la source de la pluie, touche M au titre) :

| Build | Pluie décodée | Plus longue suite de nuls | Plus longue suite quasi muette (≤ 256) | Points de boucle |
| --- | --- | --- | --- | --- |
| synthé (`ef0283c`) | 88 200 | 1 éch. | 7 éch. (0,16 ms) | 3 à 5 quasi muets sur 128 |
| OGG (`58ebfa5`) | 88 576 | 1 éch. | 274 éch. (6,21 ms), au point de boucle | 5 à 9 |
| WAV (cette reprise) | 88 200 | 1 éch. | 7 éch. (0,16 ms) | 5 sur 128 |

  Le blanc de l'OGG n'est pas fait de zéros exacts (rééchantillonné) mais de quasi-silence : d'où la 2e colonne. Le WAV ne montre aucun blanc, comme le synthé. `capture-son.mjs` (icône du son, titre et pause, build `--qa`) : 0 erreur de console, titre dessiné.
- Preuves : `lint` LINT-OK (après `cargo fmt`) ; tests VERTS : `-p game --no-default-features` 412 (59 ignorés), `-p core` 314, `-p niveaux` 16 + 84, `-p outils` 9, `-p robot` 4 ; `./build.sh` et `./build.sh --qa` OK. Pas de `carte toits`/`carte pieuvre` cette fois : aucune logique de jeu ni niveau touchés.
- Pas testé : un VRAI iPhone (Safari), un joueur humain, un vrai réseau lent (le jeu reste muet tant que la musique du titre n'est pas arrivée : 423 Ko).

## Carte 10.4d (suite, 2e étape) : la RUE du caniveau devient des fichiers PNG (`assets/fonds/caniveau`)
- Fait : la rue (`decor_rue`) n'est plus un motif du Monde 2 (`Detail::Facades` et `Detail::Lampadaires` supprimés) mais trois plans de la pile (`decor_rue::Plan`, repris par `decor::pile` en `Couche::Rue(plan)`) : maisons PNG (`caniveau/maisons.png`, UNE période de la rue, 496 px : les huit maisons, vitres ÉTEINTES, croisées, stores, vitrines, portes, descentes de gouttière, et le trottoir d'en face), fenêtres allumées CALCULÉ, lampadaires PNG (`caniveau/lampadaire.png`, UN lampadaire). Pile du caniveau : ciel PNG, silhouettes lointaines, maisons PNG, fenêtres allumées, silhouettes proches, lampadaires PNG, NIVEAU, pluie (même ordre que les anciens motifs 1 et 2 ; bloc du LISEZMOI mis à jour, vérifié par `la_pile_documentee_est_celle_du_rendu`).
- Les deux pièges notés à la 1re étape, résolus ainsi : les fenêtres allumées dépendent du numéro de tour (`allumee(tour * 8 + maison, ..)`) : le PNG n'a QUE des vitres éteintes, et le code repeint en violet, carreau par carreau, les vitres allumées de chaque tour visible (quatre carreaux autour de la croisée du PNG, la lucarne d'un seul) ; les lampadaires restent posés en UN exemplaire chacun modulo 320 px (`(lx - dec).rem_euclid(L)`, coupés au bord droit, sans copie à gauche) : le code pose deux fois le PNG d'un lampadaire, sans répétition.
- Code de dessin supprimé : `Maison` (couleurs, boutique, gouttière), `maison`, `creux`, `vitre`, `dessiner_facades`, `dessiner_lampadaires`, et le trottoir d'en face. Les PNG ont été tirés UNE fois de ce code (générateur jetable : crochet `#[cfg(test)]` dans `rect`, vitres forcées éteintes, test ignoré qui écrit la grille ; supprimé), puis relus par `sprite lire` : aller-retour identique. Restent du code, car posés d'après la grille du niveau : les descentes de gouttière `J` et leur pan de façade (`dessiner_gouttieres`, `dessiner_filets`) et le mobilier du trottoir (`mobilier_rue`).
- Tests : `les_fenetres_allumees_tombent_sur_les_vitres_du_fichier` (taille du fichier = une période ; chaque carreau que le code allume tombe sur une vitre éteinte d'une seule teinte, la croisée entre eux est d'une autre ; 95 fenêtres et 8 lucarnes ; le trottoir d'un bout à l'autre), `les_fenetres_allumees_changent_d_un_tour_a_l_autre` (6 tours, jamais deux pareils : elles ne peuvent pas être dans le PNG), `le_lampadaire_du_fichier_est_pose_sur_son_mat` ; table des tailles de `aucune_couche_png_manquante_ni_orpheline` étendue ; palette, blocs permis et 16 Ko vérifiés par les tests existants pour les 2 nouveaux fichiers.
- Poids (plafonds : +120 Ko NET depuis le début de la carte, 16 Ko par couche) : 2 couches, 1 312 octets ; wasm de production 1 066 268 → 1 067 163 octets (+895 octets : les PNG embarqués pèsent plus que le code retiré ; gzip -9 440 253 → 441 004) ; QA 1 090 313 → 1 091 208. Depuis le début de la carte 10.4d (1 067 866), le wasm a baissé (les sons de 10.4e en sont sortis entre-temps). Aucun fichier servi en plus.

| Couche | Dimensions | Octets |
| --- | --- | --- |
| `caniveau/maisons.png` | 496 x 154 | 1 078 |
| `caniveau/lampadaire.png` | 15 x 51 | 234 |

- Preuves : `lint` LINT-OK ; `test --workspace --no-default-features` VERT (core 314, game 415, niveaux 16 + 84, outils 9, robot 4) et game avec `debug_niveaux,qa_plantage` 414 ; `carte caniveau` VERTE (15 s). Pas de `carte grotte`/`port`/`pieuvre` : leur code et leurs niveaux n'ont pas bougé (leurs captures sont identiques, ci-dessous). Captures (`docs/capture-104d.sh`, build de QA copié AVANT toute modification, puis celui de la fin, `comparer-captures.py` par paires) : 20/20 IDENTIQUES, 0 pixel différent (départ et milieu des 7 niveaux du Monde 2, Pieuvre 300 et 900 images plus tard, montée à deux hauteurs). Le milieu du caniveau ne fait que 150 images de course (Pépin bute sur la poubelle) : 8 captures de plus, Pépin court et saute (8 x 4 sauts), la caméra traverse plusieurs tours de la rue (fenêtres allumées d'autres tours) et descend (parallaxe verticale) : 8/8 IDENTIQUES. Elles sont désormais dans `docs/capture-104d.sh` (`PREFIXE-rue1.png` à `rue8.png`). Exemples : `docs/qa/104d-rue-n12-milieu.png`, `docs/qa/104d-rue-loin.png`.
- `./build.sh` et `./build.sh --qa` OK, 0 nom interdit (aucun nom ni chemin /home dans dist/, dist-qa/ et assets/fonds) ; les deux PNG n'ont que IHDR, PLTE, tRNS, IDAT, IEND.
- Démarrage (`node docs/mesure-demarrage.mjs PORT 4`, dist/ de production, six séries alternées, médianes) : écran titre 236, 213, 230, 260 ms avant ; 270, 241, 230, 221 ms après ; premier dessin 579, 536, 546, 572 avant ; 599, 576, 563, 562 après. Écart dans le bruit d'une série à l'autre du même build : pas de ralentissement visible.
- RESTE (pas fait, arrêt propre après cette étape complète) : la grande chute de la cascade (`decor_cascade::dessiner_fond`) et la vallée de la rivière (`decor_riviere::dessiner_fond`), dessinées par la SCÈNE après `decor::dessiner_fond`, hors de `decor::pile` : les faire d'abord entrer dans la pile (ou une pile de scène testée de la même façon) ; la grotte (`decor_grotte`), la Pieuvre (`dessin_port::dessiner_fond`), la montée (`dessin_montee`), les motifs du Monde 2 restants (`decor::dessiner_detail` : tuyaux, lueur, chutes, brume, cristaux, eaux, forêt, grues, mouettes, phare) ; le plan « large » du port (premier cargo, phare lointain) ; toujours ouverts depuis 10.4c : la carte des niveaux (`ecrans/fond_plan.rs`) et une capture de l'écran de fin.
- Pas testé : joueur humain, téléphone, Safari.

## Carte 10.4f (1re étape) : la grande chute de la CASCADE et la vallée de la RIVIÈRE entrent dans la pile ; le ciel, le soleil et le moulin de la vallée deviennent des fichiers PNG (`assets/fonds/riviere`)
- Fait : la grande chute (`decor_cascade`) et la vallée (`decor_riviere`) n'étaient pas dans `decor::pile` : la SCÈNE les dessinait après `decor::dessiner_fond`. Elles y sont entrées, en plans comme le port et la rue : `decor_cascade::Plan` (falaise du pied, rideau de la chute, brume de la chute, filets des chutes) en `Couche::Chute(plan)`, `decor_riviere::Plan` (15 plans) en `Couche::Vallee(plan)`, après le fond générique du thème. La scène dessine désormais la pile par `decor::dessiner_fond_du_niveau(monde, cx, cy)` : les couches qui suivent la grille reçoivent le monde (sans niveau, titre, histoire, fin, planche de QA : `decor::dessiner_fond`, la grande chute n'y dessine rien, la vallée y prend un niveau d'un écran de haut). Bloc de pile du LISEZMOI mis à jour (cascade et rivière), vérifié par `la_pile_documentee_est_celle_du_rendu`.
- Pile de la rivière : ciel PNG `riviere/ciel.png`, silhouettes lointaines, motifs 1, silhouettes proches, motifs 2 (le fond générique, recouvert en entier, gardé : son ciel nourrit les garde-fous de contraste), puis ciel de la vallée PNG, halo du soleil CALCULÉ, soleil couchant PNG, nuages fins, oiseaux, montagnes, collines, rang clair, rivière du fond (CALCULÉ), moulin PNG, roue du moulin, rang sombre, versant, grands arbres, lucioles (CALCULÉ), NIVEAU. Le soleil d'avant (trois disques) est coupé en trois plans pour que le halo qui respire reste DERRIÈRE le disque fixe ; le moulin d'avant est coupé en deux (le corps fixe, PNG ; la roue qui tourne, devant).
- Ordre : la chute et la vallée passent maintenant AVANT la mer de la Pieuvre, le soleil du mur de pluie, la déchirure et les façades d'auvent (la scène les dessinait après). Aucun niveau de ces deux thèmes n'en a : test `la_chute_et_la_vallee_peuvent_passer_avant_le_reste_du_fond` (tous les niveaux cascade et rivière, essais compris, normaux et NG+ : pas de jetée, pas de mur, ni Nuage ni Auvent).
- Restent du code (listés dans le LISEZMOI) : toute la grande chute (son bas est le bassin d'arrivée du niveau, la falaise court sur toute sa largeur, les filets suivent les chutes `|`, et elle s'anime) ; dans la vallée, ce qui s'anime (halo, nuages, oiseaux, rivière du fond, roue, lucioles) et ce qui est SEMÉ AU HASARD sans période (montagnes et collines en sinus et `hache`, rangs d'arbres, versant, grands arbres qui ondulent), comme les silhouettes du Monde 1.
- Code de dessin supprimé : `bandes_du_ciel` et la trame du ciel (9 appels de `rect` en boucle), les deux disques fixes du soleil, le corps du moulin (murs, toit en gradins, cheminée, fenêtre, porte). Les PNG ont été tirés UNE fois de ce code (générateur jetable : crochet `#[cfg(test)]` dans `rect`, test ignoré qui écrit la grille ; supprimé), écrits par `sprite ecrire` puis relus par `sprite lire` : aller-retour identique.
- Tests : `le_ciel_va_du_sombre_au_clair_sans_or_vif` lit désormais le FICHIER (six bandes, jamais l'or des gouttes), `le_ciel_du_fichier_se_prolonge_par_ses_bouts` (période de deux colonnes, premier et dernier rang pleins), `la_roue_tourne_contre_le_mur_du_moulin` (le mur droit du fichier sous l'axe de la roue), le test d'ordre ci-dessus ; table des tailles de `aucune_couche_png_manquante_ni_orpheline` étendue ; palette, blocs permis et 16 Ko vérifiés par les tests existants pour les 3 nouveaux fichiers.
- Poids (plafonds de la carte : +120 Ko NET depuis son début, 16 Ko par couche) : 3 couches, 677 octets ; wasm de production 1 069 007 → 1 070 242 octets (+1 235 : les PNG et les deux tables de plans pèsent plus que le code retiré ; gzip 442 397 → 443 211) ; QA 1 093 017 → 1 094 251. Aucun fichier servi en plus.

| Couche | Dimensions | Octets |
| --- | --- | --- |
| `riviere/ciel_vallee.png` | 2 x 89 | 196 |
| `riviere/soleil_couchant.png` | 31 x 31 | 246 |
| `riviere/moulin.png` | 36 x 38 | 235 |

- Preuves : `lint` LINT-OK ; `test --workspace --no-default-features` VERT (core 314, game 418, niveaux 16 + 89, outils 9, robot 4) et game avec `debug_niveaux,qa_plantage` 417 ; `carte cascade` VERTE (54 s), `carte riviere` (15 s), `carte grotte`, `carte port`, `carte pieuvre` (10 à 11 s chacune) VERTES. Captures (build de QA copié AVANT toute modification, puis celui de la fin, servis côte à côte, `comparer-captures.py` par paires) : `docs/capture-104d.sh` (départ et milieu des 7 niveaux du Monde 2, Pieuvre 300 et 900 images plus tard, montée à deux hauteurs, rue x 8) et le nouveau `docs/capture-104f.sh` (la cascade x 6 et la rivière x 8, Pépin court et saute, 4 sauts entre deux captures : la caméra avance, monte et descend ; le halo, les nuages, les montagnes, la rivière du fond et la roue passent ENTRE les couches PNG) : 42/42 IDENTIQUES, 0 pixel différent. Exemples : `docs/qa/104f-riviere-moulin.png`, `docs/qa/104f-cascade-chute.png`.
- `./build.sh` et `./build.sh --qa` OK, 0 nom interdit (aucun nom ni chemin /home dans dist/, dist-qa/ et assets/fonds) ; les trois PNG n'ont que IHDR, PLTE, tRNS, IDAT, IEND.
- Démarrage (`node docs/mesure-demarrage.mjs PORT 4`, dist/ de production, huit séries alternées, médianes ; machine très chargée, charge moyenne 22 à 26) : écran titre 738, 668, 699, 681, 695, 692, 657, 752 ms avant ; 752, 739, 747, 728, 686, 737, 708, 733 ms après ; premier dessin 836, 727, 697, 678, 692, 688, 654, 751 avant ; 751, 898, 739, 726, 684, 734, 706, 731 après. Les deux plages se recouvrent (et rien ne change au démarrage : le titre dessine les toits, dont la pile n'a pas bougé) ; la médiane des séries « après » est plus haute d'environ 40 ms, dans le bruit de cette machine : à revérifier sur une machine au repos.
- RESTE après la 1re étape : voir la 2e étape ci-dessous.
- Pas testé : joueur humain, téléphone, Safari.

## Carte 10.4f (2e étape) : les CRISTAUX de la grotte deviennent un fichier PNG (`assets/fonds/grotte/cristaux.png`)
- Fait : les cristaux n'étaient pas une couche à part mais un motif du Monde 2 (`Detail::Cristaux`, groupe 2, le seul motif de la grotte). Le motif est supprimé ; ils sont une couche PNG de la pile, `Couche::Cristaux`, à la même place (devant la couche proche, avant le niveau). Pile de la grotte : ciel PNG, plafond lointain, plafond proche, silhouettes lointaines, silhouettes proches (CALCULÉ), cristaux PNG `grotte/cristaux.png`, NIVEAU (bloc du LISEZMOI mis à jour, vérifié par `la_pile_documentee_est_celle_du_rendu`).
- Le fichier est une PLANCHE : les cinq cristaux (hauteurs différentes) côte à côte, une bande de 8 px chacun ; le code pose chaque bande en UN exemplaire à son abscisse (`CRISTAUX` : 16, 84, 152, 220, 288) modulo la largeur de l'écran, comme les lampadaires de la rue (coupés au bord droit, sans copie à gauche), avec la parallaxe de la couche proche (vertige de la grotte compris). Nouvelle fonction `fonds::dessiner_bande` (une bande de colonnes d'une couche, par le rectangle source de la texture).
- Reste du code à la grotte : les deux plafonds et les silhouettes (semés au hasard) ; l'habillage `decor_grotte` (rideau de la cascade à l'entrée, sources sur les vasques et fissures, jour de la sortie) se pose d'après la grille et reste dessiné par la scène.
- Test : `chaque_bande_des_cristaux_est_un_cristal_pose` (chaque bande a sa base pleine et son liseré, les cristaux ne se chevauchent pas dans le motif) ; table des tailles étendue (40 x 24). Fichier tiré une fois du code supprimé (mêmes rectangles, hors du jeu), écrit par `sprite ecrire`, relu par `sprite lire` : aller-retour identique.
- Poids : 1 couche, 219 octets ; wasm de production 1 070 242 → 1 070 743 (+501) ; depuis le début de la carte : 1 069 007 → 1 070 743, +1 736 octets (gzip 442 397 → 443 528), très loin du plafond de +120 Ko ; QA 1 094 752. Aucun fichier servi en plus.

| Couche | Dimensions | Octets |
| --- | --- | --- |
| `grotte/cristaux.png` | 40 x 24 | 219 |

- Preuves : `lint` LINT-OK ; `test --workspace --no-default-features` VERT (core 314, game 419, niveaux 16 + 89, outils 9, robot 4), game avec `debug_niveaux,qa_plantage` 418 ; `carte grotte`, `carte cascade`, `carte riviere` VERTES après cette étape (`carte port` et `carte pieuvre` VERTES à la 1re, leur code n'a pas bougé depuis). `./build.sh` et `./build.sh --qa` OK, aucun nom ni chemin /home dans dist/, dist-qa/ et assets/fonds ; le PNG n'a que IHDR, PLTE, tRNS, IDAT, IEND.
- Captures : `docs/capture-104f.sh` va maintenant aussi plus loin dans la grotte (6 captures, Pépin court et saute, la caméra monte : les plafonds et les silhouettes, calculés, passent entre le ciel et les cristaux PNG), et `docs/capture-104d.sh` a été repris sur le build de la fin. Le départ et le milieu de la grotte (n15) et les 6 captures de plus : IDENTIQUES au pixel au build d'avant la carte.
- ATTENTION, honnêteté sur les captures : la machine était très chargée pendant cette étape (charge moyenne 22 puis 45). Trois passages de `capture-104f.sh` sur le MÊME build d'avant la carte ont donné, pour 4 images sur 20 (cascade 2, 3, 6 et rivière 7), des captures différentes d'un passage à l'autre (Pépin dans l'eau décalé, toute la scène décalée d'un pixel de caméra) : le pilote n'est pas parfaitement déterministe sous forte charge. Ce n'est pas le code de la carte : le même écart apparaît entre deux passages du build d'AVANT. Bilan sur tous les passages (82 captures avant, 110 après, 48 images distinctes) : 47 images après sur 48 sont identiques au pixel à une capture avant de la même image ; la 48e (cascade 6) a donné trois variantes en tout, et l'après est identique à l'avant dans les deux passages par paires faits machine moins chargée (1er passage : 42/42 ; 3e passage : 19/20, la seule différence, rivière 7, étant l'écart d'un passage d'avant avec lui-même). À refaire sur une machine au repos pour une preuve propre en un seul passage.
- Démarrage : pas de nouvelle série pour cette étape (la machine était à une charge de 45, toute mesure aurait été du bruit) ; le titre dessine les toits, dont la pile n'a pas bougé.
- RESTE (pas fait, arrêt propre après cette 2e étape complète, durée visée dépassée) : la Pieuvre phase 1 (`dessin_port::dessiner_fond`, hors pile), la montée (`dessin_montee`), les motifs du Monde 2 restants (`decor::dessiner_detail` : tuyaux, lueur, chutes, brume, eaux, forêt, grues, mouettes, phare ; la forêt et l'eau de la rivière, sous la vallée, ne se voient plus), le plan « large » du port (premier cargo, phare lointain) ; toujours ouverts depuis 10.4c : la carte des niveaux (`ecrans/fond_plan.rs`, chapitres 1 et 2) et une capture de l'écran de fin.
- Pas testé : joueur humain, téléphone, Safari.

## Carte 10.4f (AGA, clôture) : la chaîne « assets en fichiers » se termine : sons téléchargés seulement au premier son, fin des fonds du Monde 2, compression des WAV proposée, `assets/LISEZMOI.md`

Même worktree que les deux étapes précédentes (`carte-10.4f`, parti de `fbd5f8c`).

### 1. Les sons ne partent qu'à la première activation du son

- AVANT : le jeu téléchargeait les 19 WAV (3,1 Mo) dès la 2e image, que le joueur active le son ou non (et le son est coupé par défaut).
- `game/src/audio.rs` : nouvelle `FileDeSons`, pure et testée, qui décide QUOI télécharger et QUAND ; `Chargement` (les coroutines de macroquad) ne fait plus que lui demander la suivante. Règles :
  - son coupé : la file ne rend RIEN (aucune requête) ; recoupé en cours de chargement, plus aucune nouvelle requête (seuls les téléchargements déjà partis finissent) ;
  - première activation : le bip des menus, LA MUSIQUE DE LA ZONE EN COURS (Orage au titre, celle du niveau en jeu), la pluie, puis les 12 bruitages ; aucune autre musique ;
  - une autre musique ne part que quand sa zone arrive (la musique voulue change, son actif), EN TÊTE de file ; chaque fichier n'est demandé qu'une fois.
  - `Audio::avancer` donne à chaque image l'état du son et la musique voulue à la file ; `Piste::dans_l_ordre` est remplacée par `Piste::au_premier_son(zone)` (et `Piste::toutes`, pour les tests).
- Tests (crate `game`) : `son_coupe_aucun_telechargement` (toutes les zones, 6 000 images chacune, son coupé : zéro), `premiere_activation_la_zone_d_abord_et_rien_d_autre` (en cours de niveau, zone Poursuite : menu, poursuite, pluie dès la 1re image, puis les bruitages, une seule musique), `une_musique_part_quand_sa_zone_arrive` (en tête de file, une fois ; à la fin des zones, les 20 pistes demandées une fois chacune), `son_recoupe_la_file_s_arrete_et_reprend`.
- Dans le navigateur : nouveau `docs/sons-a-la-demande.mjs` (Chromium sans écran, navigateur neuf, cache coupé, vraies touches) sur `dist/` :
  - `coupe` : titre 4 s, puis le premier niveau 6 s, son jamais activé : 0 requête audio (3 requêtes en tout : la page, le JS, le wasm). AVANT (build de `fbd5f8c`, même sonde) : 19 requêtes audio, 3 100 254 octets.
  - `titre` : M au titre : `sons/menu.wav`, `musiques/orage.wav`, `sons/pluie.wav` demandés 22 à 44 ms après l'appui, puis les 11 autres bruitages ; la musique (boucle de 9,6 s) démarre 134 ms puis 156 ms après l'appui (deux passages). 14 fichiers, 685 931 octets.
  - `niveau` : 3 s de jeu son coupé (0 requête), puis M EN COURS DE NIVEAU : `menu`, `musiques/toits_de_nuit.wav` (la musique de la zone), `pluie` d'abord ; la musique démarre 122 ms puis 183 ms après l'appui, sans blocage (le jeu continue, 0 erreur de console). 14 fichiers, 770 603 octets.
  - `zone` : M au titre, puis entrée dans le premier niveau : seul `musiques/toits_de_nuit.wav` est demandé en entrant, et sa boucle (11,52 s) part.
  - SONS-A-LA-DEMANDE-OK. Seuil de la carte (« le son démarre avec plus d'une seconde de retard ») : jamais atteint, 183 ms au pire, en local, machine à une charge de 40 à 68.
  - À savoir : sur un vrai réseau, le retard de la musique sera celui du téléchargement de la boucle (orage : 423 Ko, 386 Ko en gzip) ; les bruitages arrivent après. Rien à faire de plus tant qu'on n'a pas une mesure réseau réelle.

### 2. Compression des WAV par le serveur : PROPOSITION (rien n'est modifié en prod)

- Constat (lecture seule de la configuration nginx de la prod, et de `mime.types` dans son conteneur) : `gzip_types` ne contient pas l'audio, et `.wav` n'a même pas de type (`mime.types` ne le connaît pas : servi en `application/octet-stream`). Les WAV partent donc NON compressés.
- Lignes à ajouter dans le bloc `http` de `nginx.conf`, à côté de `gzip_types` (deux lignes : le type, puis la compression) :

      gzip_types application/wasm application/javascript text/css image/svg+xml audio/wav;
      types { audio/wav wav; }

  (la 1re remplace la ligne `gzip_types` actuelle en y ajoutant `audio/wav` ; la 2e ajoute le type des `.wav` aux types déjà inclus.)
- Essayée sur une COPIE de la configuration, dans un conteneur jetable (même image `nginxinc/nginx-unprivileged:1-alpine`, port de boucle local, `dist/` monté en lecture seule, conteneurs supprimés ensuite) : `.wav` en `audio/wav` + `Content-Encoding: gzip` ; `game.wasm` toujours `application/wasm` gzip, `mq_js_bundle.js` et `index.html` inchangés (le `types {}` s'ajoute bien à `mime.types`).
- Gain mesuré (octets reçus par `curl`, 19 fichiers) : 3 096 644 -> 2 394 858 (-23 %) ; au premier son du titre (14 fichiers) : 683 276 -> 609 534. Le décodage par le navigateur ne change pas (il reçoit les mêmes octets décompressés).
- À appliquer avec l'accord d'Aga (puis redémarrer le conteneur du site) ; rien n'a été touché à la prod.

### 3. Fin des fonds du Monde 2 (« finir la 10.4d »)

Liste reprise de la 2e étape (`t_be104fce`) et rendue finale :

| Fond du Monde 2 | État |
| --- | --- |
| Égouts (mur, arches) | FAIT (10.4d) |
| Port (crépuscule, mer, port d'en face, pontons) | FAIT (10.4d) |
| Port, le large : les deux cargos, le phare | FAIT ICI : `port/cargos.png` (66 x 14, planche des deux cargos) et `port/phare.png` (77 x 31) ; restent du code le glissement du petit cargo, le faisceau qui tourne et la lampe |
| Rue du caniveau (maisons, lampadaire) | FAIT (10.4d suite) |
| Cascade (grande chute) | Dans la pile (10.4f 1re étape), reste du code : elle suit la grille et s'anime |
| Rivière (ciel, soleil couchant, moulin) | FAIT (10.4f 1re étape) |
| Grotte (cristaux) | FAIT (10.4f 2e étape) |
| Motifs du Monde 2 (`decor::Detail`) | FINI ICI : 8 motifs sur 11 n'étaient plus jamais visibles (tuyaux, lueur, eau du port, grues, mouettes, phare : leurs thèmes ont leur propre pile depuis 9.11b et 9.27b, `decor::pile` ne les posait plus ; forêt et eau de la rivière : sous le ciel de la vallée, opaque sur tout l'écran) : supprimés (environ 150 lignes). Restent les 3 motifs ANIMÉS : chutes et brume de la cascade, eau de la marée |
| Pieuvre, phase de la jetée (`dessin_port::dessiner_fond`) | Rien à sortir : mer unie, houle, feu, reflet animés ; bateaux et phare déjà des sprites (`jetee_*`). Seul dessin fixe resté en code : l'îlot du phare, deux rectangles d'une couleur (un fichier de 26 x 5 n'apporterait rien) |
| Montée de la Pieuvre (`dessin_montee`, `mer_montee`, fond sous la tour) | Rien à sortir : sprites de la Pieuvre déjà en fichiers, mer et fond qui suivent la simulation ou la caméra |

- Code : `decor_port::Plan::Large` devient trois plans, `Cargos` (PNG, posé en deux bandes de la planche par `fonds::dessiner_bande`), `Phare` (PNG) et `Faisceau` (code) ; `decor_port::phare()` (les rectangles) est supprimé, le test `le_phare_est_pose_au_bout_de_sa_jetee` lit maintenant le fichier. Sous la tour de la Pieuvre, toujours ni phare ni faisceau ; les cargos y restent, décalés comme avant (`DECALAGE`).
- `assets/fonds/LISEZMOI.md` : deux lignes de plus au tableau des couches, la pile du port et celle de la rivière à jour (vérifié par `la_pile_documentee_est_celle_du_rendu`), et la liste « ce qui reste calculé » close pour le Monde 2. Toujours ouverts HORS Monde 2, depuis 10.4c : la carte des niveaux (`ecrans/fond_plan.rs`) et une capture de l'écran de fin.
- PNG : `sprite lire` redonne la grille d'origine ; blocs IHDR, PLTE, tRNS, IDAT, IEND seulement (208 et 217 octets).

### 4. `assets/LISEZMOI.md`

- Nouveau : ce qui est un fichier (tableau des 5 dossiers : contenu, embarqué dans le wasm ou servi à côté, LISEZMOI de règles) et ce qui reste CALCULÉ, avec pourquoi : la pluie, les éclairs, l'eau du Monde 2, les particules, les tuiles éclaircies, les plans semés au hasard ou animés des fonds, le texte. Il dit aussi que les sons ne partent qu'au premier son.
- `assets/musiques/LISEZMOI.md` : le point 3 (« comment le jeu les charge ») suit la nouvelle règle.
- Nouveau `game/src/assets.rs` (tests seulement) : `chaque_fichier_d_assets_est_lu_par_le_jeu` (chaque fichier de `assets/` est un `.png`, `.ron` ou `.wav` de son dossier, ou un LISEZMOI : un `.ogg` oublié ou un dossier inconnu ferait échouer) et `chaque_dossier_d_assets_est_decrit`. Avec les tests par dossier qui existaient (sprites, fonds, sons, niveaux), c'est le « aucun fichier orphelin ni manquant dans chaque dossier d'assets » de la carte.

### Preuves

- `./docker-cargo.sh lint` : LINT-OK. `test --workspace --no-default-features` VERT : core 314, game 424 (419 + 4 tests de la file des sons - 1 remplacé + 2 tests d'assets), niveaux 16 + 89, outils 9, robot 4. Et game avec `debug_niveaux,qa_plantage` : 423 VERT.
- `./build.sh` et `./build.sh --qa` OK. wasm 1 070 743 -> 1 069 153 octets (-1 590 : les motifs morts).
- Poids par dossier d'`assets/` (fichiers, octets) : sprites 335 / 74 436 ; fonds 32 / 9 841 (+425 : cargos et phare) ; levels 33 / 175 617 ; musiques 6 / 2 836 772 ; sons 13 / 259 872.
- Poids téléchargé, mesuré en servant `dist/` par la configuration nginx de la prod (copie, conteneur jetable) :

| | AVANT (`fbd5f8c`) | APRÈS, nginx actuel | APRÈS, nginx proposé |
| --- | --- | --- | --- |
| Son coupé (page : HTML, JS, wasm, en gzip) | 460 402 + 3 096 644 de sons = 3 557 046 | 460 825 (aucun son) | 460 825 |
| Son activé au titre | idem (tout est déjà là) | + 683 276 (14 fichiers) | + 609 534 |
| Toutes les zones visitées | idem | + 3 096 644 (19 fichiers) | + 2 394 858 |

- Démarrage (`node docs/mesure-demarrage.mjs PORT 5`, navigateur neuf, deux séries alternées avant / après, machine à une charge de 50 à 68 : mesures très bruitées) : premier dessin, médianes : série 1, avant 1 333 ms, après 1 330 ms ; série 2, avant 876 ms, après 655 ms. Avant, les 19 sons étaient décodés à 1,6 s à 2,6 s ; après, aucun son n'est téléchargé tant que le son est coupé. Le titre n'apparaît pas plus tard ; il n'a plus 3 Mo de sons en concurrence sur le réseau.
- Captures (build de QA copié AVANT toute modification de cette carte, `fbd5f8c`, puis celui de la fin, servis côte à côte, `docs/capture-104d.sh` et `docs/capture-104f.sh`, `comparer-captures.py` par paires), deux passages de chaque build, machine à une charge de 40 à 68 : 48 images (départ et milieu des 7 niveaux du Monde 2, Pieuvre 300 et 900 images plus tard, montée à deux hauteurs, rue x 8, cascade x 6, grotte x 6, rivière x 8 ; le port montre le phare et les cargos, désormais en PNG). 1er passage d'avant contre 2e passage d'après : 48/48 IDENTIQUES au pixel. Les autres croisements ont 2 ou 3 images différentes (rivière 1 et 2 ; montée haute b), mais des écarts du même genre apparaissent entre deux passages du MÊME build (avant contre avant : rivière 1 et 2 ; après contre après : rivière 1, 2 et montée haute b) : c'est Pépin (une autre image de son animation) et les lucioles, le pilote n'étant pas parfaitement déterministe sous forte charge (déjà noté à la 2e étape), pas le fond. Aucune image d'après qui ne soit identique à une image d'avant.
- Rien de personnel dans le dépôt : ni nom, ni chemin `/home`, ni IP dans le diff, `assets/`, `dist/` et `dist-qa/` (vérifié par `grep`).

## Carte 10.5 (AGA) : mise en prod du lot « refonte, assets et Pieuvre » : ARRÊTÉE AVANT LA PUBLICATION (lot ROUGE)

- RIEN N'EST PUBLIÉ : ni fusion de `monde-2` dans `main`, ni `publier-prod.sh`, ni push. La prod reste `c4d715b` (wasm en ligne `605ea9a4…`, vérifié identique au dossier `www` du site).
- Raison : critère « lot complet » ROUGE. `./docker-cargo.sh lot` sur une extraction propre du commit gelé : core 314 + 5 de lot, niveaux 16, `tous` 89 + 57 de lot, game 424 rapides : tout VERT ; tests de lot de game : 18 VERTS, 1 ROUGE : `le_robot_finit_le_monde_2_sans_mourir` (5 678 s) : « le robot ne trouve aucun parcours au niveau 19 » (`game/src/bot_qa/mod.rs:348`), c'est-à-dire la PIEUVRE (Monde 2, niveau 7), robot précis comme maladroit. Même échec, au même endroit, dans le `lot` de la carte 11.1 lancé en parallèle sur sa propre branche (`target/lot/` de son dossier, 4 899 s) : l'échec est reproductible. `le_robot_bat_la_pieuvre_sans_mourir` (niveau d'ESSAI de la Pieuvre) est vert ; la faisabilité du vrai niveau (`faisabilite::monde_2_niveau_7_faisable` et `_reprise`, `par_niveau::pieuvre`, `pieuvre_montee`, dans `tous`) aussi : c'est le robot de QA de PARTIE qui ne trouve plus de chemin dans la vraie Pieuvre. Les cartes 9.13a et 9.31 l'avaient noté « PAS testé : robots de partie complète (lot) » ; la phase 2 (9.13a à e) est déjà en prod, 9.13f et 9.13g (la mer qui suit Pépin, la vigie) partiraient avec ce lot. Non vérifié : si `c4d715b` (la prod actuelle) échoue aussi (1 h 30 de calcul sur cette machine).
- Décision demandée à AGA : publier quand même (le reste est vert, voir plus bas), ou d'abord une carte « robot de QA de partie : Pieuvre phase 2 » (puis relancer cette carte sur un nouveau gel).
- Commit gelé : `monde-2` = `dd45fb2` (« Fusion carte 10.4f (Monde 2) » ; `merge-base --is-ancestor carte-10.4f monde-2` vrai au départ ; `monde-2` n'a pas bougé pendant la carte). 33 commits de `monde-2` absents de `main` : 9.13f, 9.13g, 10-niveaux (10.1a à c), 10-scene (10.2a à c), 10.4a à 10.4f (assets en fichiers, sons à la demande), éditeur (10.3a à c, site de test seulement).
- Étiquette `prod-avant-10.5` créée sur `c4d715b` (la prod du 2026-11-23), dans le dépôt local seulement (pas poussée, puisque rien n'est publié).
- Retour arrière prévu (à utiliser seulement après une publication) :
  - site : recopier la sauvegarde écrite par `publier-prod.sh` dans le dossier servi : `cp -a <site>/www.bak-<date>/. <site>/www/` (`<site>` : le dossier du site, celui de `publier-prod.sh`), puis `curl -s "<adresse de la prod>/game.wasm?v=$(date +%s)" | sha256sum` doit redonner `605ea9a4137849a5…` ;
  - git : `git -C ~/projects/pepin revert -m 1 <commit de fusion>` puis `git -C ~/projects/pepin push origin main`.
- Vérifications AVANT, sur une extraction propre (`git archive dd45fb2`, pas de checkout) :
  - `./docker-cargo.sh lint` : FMT-OK, LINT-OK.
  - `./build.sh` : OK ; 19 WAV canoniques, aucun nom interdit ; contrôle refait à part sur tout `dist/` : 22 fichiers, 0 nom ni chemin ni IP, 0 balise de métadonnée, WAV mono 22 050 Hz 16 bits ; éditeur absent ; `?essai=1` absent du wasm de prod (aucun import `pepin_niveau_essai*` ; présent dans celui de QA). `./build.sh --qa` OK.
  - Navigateur (Chromium 1208 sans écran, `dist/` servi en local, horloge virtuelle de `docs/pilote-qa.js`, pilote de recette hors dépôt) : 0 erreur de console, 0 requête en échec ; 0 couleur hors palette et 0 flou sur toutes les captures au clavier, regardées :
    - sauvegarde vide : titre, histoire, carte du niveau qui ATTEND (10 s sans appui : toujours affichée), jeu, pause, M (son), quitter au titre ;
    - Monde 2 ouvert : carte des niveaux, chapitres 1 et 2, caniveau, son ;
    - boutique ;
    - téléphone (844 x 390, DPR 3, tactile émulé) : JOUER au doigt, boutons, droite et saut au doigt, pause au doigt, menu de pause au doigt ; debout (390 x 844) : « Tourne ton téléphone », puis retour en paysage. Les captures EN JEU au téléphone ont des pixels hors palette et du flou : les boutons sont translucides (comme aux cartes 9.2c et 9.30), attendu.
  - Sons à la demande (`node docs/sons-a-la-demande.mjs`, sur `dist/`) : SONS-A-LA-DEMANDE-OK ; 0 requête audio son coupé ; après M, musique lancée en 230 à 445 ms (machine chargée), 14 fichiers ; la musique d'une autre zone arrive en entrant dans son niveau. Sonde sur `AudioBufferSourceNode.start` : au titre 2 sons lancés (musique 9,6 s et pluie 2 s, en boucle), en jeu 7, contexte audio `running` à chaque lancement.
  - Robot du Monde 1 (`ecrire_script_navigateur`) : 21 801 images, 0 mort, 13 gouttes, empreinte `efffa02569f3d66a`. Son rejeu complet dans le navigateur, commencé, a été arrêté (trop lent sous la charge) : non fait.
- Sauvegardes (prod actuelle servie en local À CÔTÉ du nouveau build, même parcours, captures comparées au pixel par `docs/comparer-captures.py`) :
  - sauvegarde v3 riche (212 gouttes, 3 skins dont le doré équipé, traînée étoiles, musique orage, `bonus_credites=fin`, médailles 0 à 3, temps, `defaites`, `facile`, `lettre`, Monde 2 commencé, record de course) : carte des niveaux (2 chapitres, niveau voisin), boutique (puis Droite) : 5/5 captures IDENTIQUES ; texte de la sauvegarde relu à l'identique après la visite, sur les deux builds ;
  - ancienne sauvegarde v2 (`niveau_max_atteint`, niveaux par numéro, celle du test `la_migration_v2_range_chaque_niveau_sous_sa_cle`) : carte, niveau voisin, boutique : 3/3 IDENTIQUES ; texte inchangé ;
  - seul l'écran titre diffère (64 206 et 66 582 pixels) : la pluie et le personnage animés ne tombent pas à la même image d'un build à l'autre ; gouttes, RECORD, lignes du menu et skin identiques à l'œil.
  - Pas une vraie sauvegarde de joueur : des sauvegardes écrites à la main au format de la prod (aucune sauvegarde de joueur n'est accessible ici).
- Poids (son coupé, titre puis 1er niveau, navigateur neuf sans cache, octets reçus d'après CDP, en-têtes compris) :

| | prod actuelle `c4d715b` | nouveau `dd45fb2` |
|---|---|---|
| `game.wasm` brut | 904 301 | 1 069 153 (+164 852 : sprites et fonds en PNG embarqués, niveaux RON, serde et ron) |
| `game.wasm` gzip | 352 501 (reçu en ligne) | 442 515 (tableau de `./build.sh`) |
| html + js gzip | 18 125 (reçu en ligne) | 16 874 (tableau de `./build.sh`) |
| images à part | 0 (dans le wasm) | 0 (PNG dans le wasm) |
| audio son coupé | 0 (sons synthétisés DANS le wasm au démarrage) | 0 |
| audio au premier M | 0 | 686 à 771 Ko (14 fichiers : bruitages, pluie, musique de la zone) |
| total son coupé | 958 122 brut ; 370 626 reçus en ligne | 1 125 061 brut ; environ 459 400 en gzip |
| `musiques/` + `sons/` servis | absents | 3 096 644 (gzip 2 399 431), jamais tous d'un coup |

- Démarrage (`node docs/mesure-demarrage.mjs PORT 4`, deux séries alternées, médianes ; charge de la machine 60 à 150 : mesures très bruitées) : premier dessin 2 697 et 2 870 ms avant (le titre attendait la synthèse des 19 sons), 1 589 et 1 854 ms après ; fin de la 3e image 1 272 et 1 648 avant, 1 599 et 1 931 après ; plus longue des 10 premières images 254 et 283 avant, 560 et 696 après (décodage des PNG, une fois, sous forte charge). L'écran titre apparaît plus tôt.
- PAS fait (la publication n'a pas eu lieu) : fusion, `publier-prod.sh`, empreintes en ligne, recette sur le site en ligne (titre, un niveau par chapitre, la Pieuvre jusqu'à la tour, boutique, son). PAS testé du tout : vrai téléphone, iPhone et Safari, joueur humain, la Pieuvre dans le navigateur (le script du robot du Monde 2, `ecrire_script_monde_2_96`, était encore en calcul à l'arrêt de la carte, après 2 h : il cherche aussi un parcours dans la vraie Pieuvre).

## Carte 10.5 (AGA, reprise après la décision « A ») : lot « refonte, assets et Pieuvre » PUBLIÉ (`main` = `ffe6a4f`)

- Décision d'AGA (2026-11-24, réponse « A ») : on publie malgré le seul test rouge, `le_robot_finit_le_monde_2_sans_mourir` (« le robot ne trouve aucun parcours au niveau 19 », la vraie Pieuvre). C'est le robot de QA de PARTIE qui ne sait pas jouer la phase 2 ; le jeu n'est pas en cause (le pilote de la 9.13g la gagne 10 fois sur 10, et la phase 2 est en prod depuis `c4d715b`). La carte 10.7 apprendra la phase 2 au robot. TEST ROUGE ACCEPTÉ, pas corrigé ici. Les vérifications « avant » de la section précédente restent valables (le lot n'est pas relancé) : `monde-2` n'a pas bougé.
- Gel revérifié juste avant la publication : `monde-2` local et `origin/monde-2` valent toujours `dd45fb2`, dépôt principal propre.
- Pendant : `publier-prod.sh` (code 0) : fusion `--no-ff` de `monde-2` dans `main` = `ffe6a4f` ; arbre de `ffe6a4f` IDENTIQUE à celui de `dd45fb2` (`git diff --stat` vide) ; 0 nom interdit dans le wasm et les sons ; sauvegarde du site `www.bak-20261124-181716` ; empreintes en ligne : `game.wasm` `ad2c65e5ef1bdd37` en local, sur la prod et sur son adresse en `www` (le même wasm que celui vérifié AVANT sur l'extraction propre de `dd45fb2`), `musiques/orage.wav` `834a91ae8da1d2ff` des deux côtés, site de test `c660bc0620021dba` des deux côtés ; 0 secret ; `main` poussé (`c4d715b..ffe6a4f`), `monde-2` déjà à jour. Étiquette `prod-avant-10.5` (sur `c4d715b`) poussée.
- Après, sur le site en ligne :
  - Fichiers servis : les 22 fichiers de `dist/` (html, js, wasm, `musiques/`, `sons/`) téléchargés un par un en ligne (requête sans cache) : 22/22 sha256 identiques. `editeur.html` absent de la prod (404), présent sur le site de test.
  - Navigateur (Chromium 1208 sans écran, pilote de recette de la carte, horloge virtuelle de `docs/pilote-qa.js`, navigateur neuf à chaque parcours) : 0 erreur de console, 0 requête en échec, cadre d'erreur de la page caché, 0 couleur hors palette et 0 flou sur les 18 captures, regardées :
    - écran titre, carte du niveau « Les toits », jeu, M, pause ;
    - chapitre 1 : « Les toits » joué ; chapitre 2 : carte des niveaux (chapitre 2), carte du niveau « Le caniveau », caniveau joué, M ;
    - la Pieuvre (sauvegarde écrite à la main, Monde 2 fini jusqu'au port) : carte des niveaux (niveau 7/7), carte du niveau « La Pieuvre du port », arène de la phase 1 au départ, 5 s plus tard (frappes dans l'eau), puis Pépin vers la droite ;
    - boutique (skins, puis traînées), retour au titre.
  - Son activé (M) : 14 fichiers audio arrivent (bruitages, pluie, musique de la zone : `toits_de_nuit` aux toits comme au caniveau, comme prévu par `musiques.rs` ; `orage` au titre), 0 requête audio son coupé (boutique et Pieuvre : 3 requêtes seulement). Les sons JOUENT : sonde sur les `AudioContext` créés par la page, en ligne comme en local avec le même build, au titre et en jeu, avec un appui du pilote ou un vrai appui clavier (CDP) : contextes `running`, horloge audio qui avance (3,6 s, puis 6,2 s, puis 10,4 s), 2 à 4 sons lancés (musique 9,6 ou 11,5 s et pluie 2 s en boucle). Note : la première sonde (état lu à l'instant du lancement de chaque son) disait `suspended` en ligne et `running` en local ; la sonde sur les contextes montre `running` dans les deux cas, horloge qui avance : écart de mesure, pas un défaut.
  - Poids réellement reçus en ligne (son coupé, titre puis 1er niveau, octets encodés d'après CDP) : wasm 443 946 (gzip), html et js 18 655, total 462 582 (prod d'avant : 370 626) ; au premier M, 779 193 de plus (14 fichiers).
- Critères d'arrêt : aucun. Pas de sauvegarde qui ne se recharge pas (vérifié avant, captures identiques au pixel), pas d'erreur au chargement du site. Pas de retour arrière.
- Retour arrière, si besoin plus tard (commandes exactes) :
  - site : `cp -a <site>/www.bak-20261124-181716/. <site>/www/` (`<site>` : le dossier du site de `publier-prod.sh`) ; le `game.wasm` de cette sauvegarde a l'empreinte `605ea9a4137849a5` (vérifié), `curl -s "<adresse de la prod>/game.wasm?v=$(date +%s)" | sha256sum` doit la redonner. Les dossiers `musiques/` et `sons/` resteraient dans `www`, sans gêne (l'ancien build ne les demande pas).
  - git : `git -C ~/projects/pepin revert -m 1 ffe6a4f` puis `git -C ~/projects/pepin push origin main`.
- PAS testé : la Pieuvre JUSQU'À LA TOUR en ligne (le parcours s'arrête dans l'arène de la phase 1 : pour monter dans la tour il faut battre la phase 1, et aucun robot ne sait encore jouer la vraie Pieuvre en entier dans le navigateur, carte 10.7) ; vrai téléphone, iPhone et Safari, joueur humain ; une vraie sauvegarde de joueur (seulement des sauvegardes écrites au format de la prod) ; écoute réelle des sons (navigateur muet : on vérifie qu'ils se lancent et que l'horloge audio avance).

## Carte 11.1 (AGA) : prototype, se balancer au parapluie (crochet, corde, élan, lâcher)
- Verdict sur les deux conditions d'arrêt : le geste ne demande AUCUN bouton de plus (SAUT tenu près d'un crochet : la crosse s'accroche au lieu de planer ; GAUCHE/DROITE : l'élan ; SAUT lâché : le lancer), et le robot de faisabilité FINIT le niveau d'essai. Pas d'arrêt.
- Le cœur (`core/src/player/balancement.rs`, sous-module de `player`) : nouvel état `EtatJoueur::Suspendu`, `Joueur::actualiser_avec_crochets` (sans crochet et sans balancement, c'est `actualiser` au bit près : un test le vérifie ; le Monde 1 et le Monde 2 ne changent pas). Pendule : angle et vitesse angulaire, Euler semi-implicite (la vitesse, puis l'angle avec la nouvelle vitesse : pas d'énergie fabriquée), léger frottement.
- Règles d'AGA, en constantes nommées (à régler en jouant) : `PORTEE_CROCHET` 2,5 cases (40 px, de la crosse au crochet, crochet plus haut que la crosse) ; `CORDE_MIN` / `CORDE_MAX` 1,5 et 3 cases (la corde = la distance à l'accroche, bornée) ; `ANGLE_MAX` 80° (arrêt net, la pesanteur ramène) ; `BONUS_LACHER` 1,10 ; `DELAI_RACCROCHE_PAS` 18 pas (0,3 s, au même crochet seulement : un autre crochet prend tout de suite) ; décrochage au contact d'un solide ou de l'eau (il garde sa vitesse le long du cercle, sans bonus). Les ennemis blessent comme d'habitude (`resoudre_contacts` inchangé ; un rebond décroche).
- Réglages ajoutés par moi (non fixés par AGA) : `POUSSEE_BALANCEMENT` 250 px/s² le long du cercle, `FROTTEMENT_BALANCEMENT` 2 % de vitesse par seconde, `HAUTEUR_CROSSE` 8 px au-dessus de la tête (le bout de la corde). Au lâcher, la part horizontale au-delà de la course (96 px/s) passe dans l'élan (le même que celui des auvents : les commandes en l'air s'y ajoutent, il s'éteint en ligne droite). Pendant le balancement, ni courant ascendant ni vent ne poussent Pépin (hors du prototype).
- Le crochet : lettre `y` (libre), dans la légende de `core/src/level.rs` (recopiée dans `assets/levels/LISEZMOI.md` par `cargo run -p niveaux --bin lisezmoi`) et dans l'éditeur (`niveaux/src/legende.rs` : lettre, couleur, vignette `crochet`). `Niveau::crochets` (le point d'accroche au milieu de la case), vide partout ailleurs.
- Dessin (`game/src/scene/crochets.rs`) : trois PNG, pas plus : `objets/crochet.png` (anneau de fer 10x8), `objets/crochet_allume.png` (le même, doré : la crosse est à portée, ou Pépin y pend), `pepin/pepin_suspendu.png` (16x23, parapluie fermé tenu à deux mains, crosse orange en haut ; Pépinette prend le même dessin par un ALIAS, `fichiers.rs`). La corde : une ligne de pixels pleins (Bresenham, testée) couleur brume (le contour sombre de la 1re capture se perdait dans le ciel de nuit), du crochet à la crosse. Planche des sprites régénérée.
- Le niveau d'essai `assets/levels/essais/essai_balancement.ron` (touche C du build de QA, `INDICE_ESSAI_BALANCEMENT`), 104 x 14 : un crochet au-dessus du sol (10, 7) ; au-dessus d'un trou (23, 7) ; deux enchaînés (41, 7) et (47, 7) au-dessus d'un trou de 16 cases ; trois en montée (63, 7), (67, 5), (71, 3) jusqu'à une corniche 112 px au-dessus du sol (hors de portée d'un saut) ; un lâcher suivi d'un plané depuis (81, 3), au bord de la corniche, au-dessus d'un trou de 16 cases jusqu'au drapeau. 3 gouttes (une en l'air près du 1er crochet, une sous le 2e crochet enchaîné, une sur la corniche), 4 lanternes.
- Fait à l'éditeur : `node docs/capture-111.mjs PORT DOSSIER` (CHROME=chromium-1208, build de QA servi en local sur un port libre) ouvre `editeur.html`, clique Nouveau, « + à droite » x44, « + en haut » x2, choisit les lettres dans la palette et pose tout à la souris (outil Rectangle pour les trous, le sol et la corniche ; Pinceau pour X, L, o, y), tape le formulaire au clavier (cle, nom, durée, intro, mécanique), clique Valider (« accepté par le chargeur du jeu »), ROBOT, puis « Exporter le .ron » (téléchargé). Le fichier rangé dans le dépôt est cet export, octet pour octet sous l'en-tête (vérifié par `diff` sur une seconde construction complète).
- Fait HORS de l'éditeur : (1) les commentaires de conception en tête du fichier, ajoutés à la main (l'éditeur ne sait pas en écrire ; il les garde à la relecture : `./build.sh --qa` relit et réécrit les 34 fichiers sans écart) ; (2) les positions des crochets, gouttes et lanternes, mises au point AVANT avec un petit script jetable qui écrivait la grille et le robot de faisabilité (2 essais : la 1re goutte, placée trop haut, n'était pas prise), puis saisies à l'éditeur ; (3) la première construction à l'éditeur avait posé des `o` au lieu des `y` (le bouton `y`, en bas de la palette, était hors de la fenêtre et le clic tombait à côté) : le pilote amène désormais chaque bouton à l'écran avant de cliquer.
- Robot de faisabilité (`core/src/faisabilite.rs`) : il se balance avec la VRAIE physique (`actualiser_avec_crochets`). Un plan « planer » qui passe à portée d'un crochet s'y accroche : nouveau point de décision (`Fin::Suspendu`, `Noeud::accroche`, l'état du pendule rangé sous la clé du noeud). Depuis un crochet : 360 plans de lâcher (sens, 0 à 8 demi-balancements d'élan, angle de lâcher de -20° à 60°, vol direction gardée ou non, plané ou non). Sans crochet, rien ne change (mêmes plans, `lacher: None`).
- Robot sur le niveau d'essai : ARRIVÉE, 3 gouttes sur 3, 4 lanternes sur 4, 958 états, course rapide 18,8 s ; 6,4 s de recherche (test natif en release, `par_niveau::essai_balancement::le_robot_de_faisabilite_finit_le_niveau`) et 15 à 17 s dans le navigateur (bouton ROBOT de l'éditeur, wasm : « arrivée atteinte en 18.8 s de jeu, 19 sauts, lanternes allumables 4/4, gouttes 3/3 [robot : 17.3 s, 958 états] », capture `docs/qa/111-editeur-robot.png`). Sans les crochets (`y` remplacés par du vide), l'arrivée est hors d'atteinte (test `sans_crochet_l_arrivee_est_hors_d_atteinte`). Reprise depuis chaque lanterne des niveaux d'essai : verte.
- Tests du cœur (`player::balancement::tests`, 10) : conditions d'accroche (SAUT tenu en l'air, portée, crochet plus haut, au sol c'est un saut), corde bornée, sans appui pas d'énergie gagnée (l'amplitude de chaque demi-balancement ne grandit jamais sur 10 s ; l'énergie mesurée, elle, oscille d'environ 0,4 % autour de sa valeur : c'est le schéma numérique, pas un gain), élan jusqu'à 80° et pas plus, direction et vitesse au lâcher (+10 % exactement, le déplacement du pas du lâcher aussi, plané possible aussitôt), délai de raccroche (18e pas au même crochet, 1er pas à un autre), décrochage sur un mur, même suite d'entrées = même résultat au bit près, sans crochet = `actualiser`.
- Captures (`docs/qa/111-*.png`, cadre d'essai de l'éditeur : clic droit « partir d'ici » près du crochet, puis le clavier en temps réel ; regardées une à une) : `111-1-sol` (balancé au-dessus du sol, crochet allumé, corde), `111-2-trou` (balancé au-dessus du trou), `111-3-enchaines` (accroché au 2e crochet après avoir lâché le 1er, éteint derrière), `111-4-montee` (au crochet du milieu, le 1er en bas, le 3e en haut ; la montée complète à la main au clavier en temps réel n'a pas été réussie par le pilote : le robot, lui, la fait), `111-5-plane` (plané au-dessus du trou après le lâcher du crochet de la corniche). Zone de jeu : 0 pixel hors palette, 0 bloc flou (`docs/mesure-capture.py`, K = 4, sur la zone recadrée).
- Preuves : `lint` FMT-OK, LINT-OK ; `carte toits` VERTE (core 324, game 416, niveaux 16 + 90, `le_niveau_de_la_carte`) ; game avec `debug_niveaux,qa_plantage` 415 VERTS ; tests ignorés du niveau d'essai et reprise des essais VERTS ; `./build.sh --qa` OK (game.wasm de QA 1 099 776 octets). Un test a dû suivre : `chaque_skin_fournit_toutes_les_images_aux_tailles_de_pepin` compte maintenant 19 images + l'icône.
- Suite `lot` du coordinateur (le robot de faisabilité a changé : vérifier qu'il trouve toujours tout ailleurs) : core-lot 5/5, tous-lot 59/59 (faisabilité et reprise de TOUS les niveaux, 680 s), rapides verts ; game-lot (robots de QA, mémoire des parcours invalidée par le changement de code, machine partagée) : 18/19 verts (tous les niveaux du Monde 1, NG+, bonus, clocher, tous les essais, Grand Nuage, Pieuvre), le 19e, `le_robot_finit_le_monde_2_sans_mourir`, n'avait pas fini quand la limite de 50 min a coupé la suite (code 124) : NON VÉRIFIÉ ici. Sans crochet, `actualiser_avec_crochets` est `actualiser` au bit près (test), le Monde 2 n'a aucun crochet.
- « Niveau jouable sur dev en build de QA » : vérifié en LOCAL sur le build de QA (`dist-qa/`, touche C ou l'éditeur). PAS publié sur dev : consignes du chantier (local seulement, le coordinateur publie).
- Hors périmètre, pas fait : niveaux du Monde 3, crochets mobiles, mesure de difficulté ; le robot de QA du jeu (`bot_qa`) ne connaît pas les crochets (il n'a pas été demandé). Pas testé : un joueur humain (le « est-ce agréable ? » reste à juger par AGA), téléphone, Safari.

## Carte 11.2 (AGA) : plus aucun « monde 2 » écrit en dur (un Monde 3 par des données)

But : un troisième monde s'ajoute par des fichiers `.ron`, sans retoucher le code. Les huit endroits de l'audit (numéros de `cd2a559`, retrouvés par grep), et la décision pour chacun :

| Endroit | Avant | Décision |
| --- | --- | --- |
| `defis.rs:269` (`bonus_fin_du_monde`) | `if monde > 1` : les trois constantes `BONUS_*_MONDE2` | CAS DU MONDE 2, remplacé par une règle : `LigneBonus` porte son `monde`, `cle_sauvegarde()` vaut l'identifiant au Monde 1 (`fin`...) et `monde{N}_{id}` ailleurs. Le Monde 2 retrouve EXACTEMENT ses clés (`monde2_fin`...), un Monde 3 a les siennes (`monde3_fin`...). Constantes `BONUS_*_MONDE2` supprimées. |
| `contenu.rs:479` (`etiquette_niveau`) | `if infos.monde > 1` : « CHAPITRE N · » devant | RÈGLE GÉNÉRALE (« il existe un monde précédent » : seul le Monde 1 n'écrit pas son chapitre), laissée et commentée. |
| `app/transitions.rs:238` (records de fin de partie) | `monde > 1` puis le champ `partie_monde2` | Le test `monde > 1` est une règle générale (le Monde 1 garde ses champs d'avant) ; le champ du Monde 2 devient `Sauvegarde::parties_mondes` (`BTreeMap<u8, RecordsPartie>`), `noter_partie_monde(monde, ...)`, `records_partie(monde)`. Ligne `partie.mondeN=...` pour tout N ≥ 2 (lue par `monde_de_cle_partie`, qui refuse `partie.monde1`, `partie.monde`, `partie.monde+3`, `partie.monde2x`). |
| `scene/entites.rs:298` (`images_corbeau`) | `monde >= 2` : chauve-souris | CAS DU MONDE 2, remplacé par une donnée du NIVEAU : son thème (`images_corbeau(theme)`, `match` complet : un thème ajouté devra choisir). Les 7 thèmes du chapitre 2 donnent la chauve-souris, les 12 du Monde 1 le corbeau. Test : pour chaque niveau de la vraie table, la même image qu'avant. Aucun niveau d'essai d'un thème du chapitre 2 n'a de corbeau (vérifié dans leurs grilles) : rien ne change non plus en QA. |
| `ecrans/plan.rs:224` (fond de la carte des niveaux) | `if e.monde >= 2` : `chapitre_2` | CAS DU MONDE 2, remplacé par une donnée des niveaux : `fond_plan::fond_en_descente(tranche)` (un de ses niveaux a un thème avec une bande, `bande_du_theme`). Monde 1 : non, Monde 2 : oui (testé). |
| `plan.rs:226` (`etat_etape`) | `n.monde > 1 && !mondes_suivants_ouverts(...)` | Le `monde > 1` est la règle générale ; la PORTE était un cas du Monde 2 (la constante `CLE_ACCES_MONDES_SUIVANTS = "poursuite"` ouvrait tous les mondes suivants). Remplacée par `porte_du_monde(niveaux, monde)` : le dernier niveau NORMAL du monde précédent, lu dans la table. Pour le Monde 2 c'est `poursuite` (test `la_porte_du_monde_2_est_poursuite`) ; pour un Monde 3, la fin du Monde 2. `monde_ouvert(niveaux, s, monde)` remplace `mondes_suivants_ouverts`. |
| `plan.rs:555` (Bas au bout du chemin) | `self.monde > 1 && aller_monde(monde - 1)` | RÈGLE GÉNÉRALE (« il existe un monde précédent »), laissée et commentée. |
| `plan.rs:573` (Gauche en JOUER) | `self.monde > 1` | RÈGLE GÉNÉRALE, même commentaire. |

Deux endroits VOISINS qui écrivaient encore « 2 » (hors des huit, mais un Monde 3 n'aurait jamais été atteint par JOUER) :
- `app/mod.rs` `monde_de_jouer` : `monde_jouable(..., 2)` devient `plan::dernier_monde_jouable` (on monte tant que le monde suivant est jouable ; Monde 1 au moins). Avec deux mondes : le même résultat (test `jouer_reprend_au_dernier_monde_ouvert`, et `app::tests` inchangés).
- `app/transitions.rs` (écran de fin, ligne CONTINUER) : `bilan.monde == 1 && monde_jouable(2)` devient `bilan.monde >= 1 && monde_jouable(bilan.monde + 1)`. À la fin du Monde 2, `monde_jouable(3)` est faux dans la vraie table : rien ne change.

Faux Monde 3 (test seulement, `game/src/faux_monde_3.rs`, `#[cfg(test)]`) : `essai_chat`, `essai_escargot`, `essai_eau` recopiés après la vraie table avec `monde: 3`, numéros 1 à 3, genre normal (ce que donneraient trois fichiers `monde_3/*.ron`). Tests :
- `la_porte_du_monde_3_est_la_fin_du_monde_2` : fermé tant que le dernier niveau normal du Monde 2 n'est pas fini, puis jouable, 1re étape ouverte, les autres verrouillées puis ouvertes une à une ; JOUER y reprend ;
- `la_carte_des_niveaux_affiche_le_monde_3` : Haut au bout du Monde 2 y mène (géométrie de 3 étapes), flèche de chapitre vers lui, pas de Monde 4, Bas en revient ; « CHAPITRE 3 · NIVEAU 2/3 » ; titre de chapitre absent (le bandeau garde « CARTE DES NIVEAUX », voir limites) ;
- `le_faux_monde_3_a_un_fond_de_carte` (`ecrans/fond_plan.rs`) : fond en descente (il a un niveau du caniveau), bandes jointives du pied au haut ; sans thème du chapitre 2, fond de l'ascension ;
- `les_defis_et_bonus_du_monde_3_existent` : 3 défis par niveau, bonus `monde3_*` crédités une fois, le Monde 2 pas touché ;
- `les_records_du_monde_3_s_enregistrent` : `partie.monde3=...`, progression de ses niveaux, relue à l'identique, Monde 1 intact ;
- `les_entites_du_faux_monde_3_se_dessinent` (`scene/entites.rs`) : ses niveaux se chargent, les images du chat, de l'escargot et du corbeau de son thème sont dans la banque de sprites ; plus rien dans le dessin ne lit le numéro du monde (grep de `infos.monde` dans `game/src` : seuls l'étiquette et les comptes par monde restent) ;
- `une_sauvegarde_actuelle_se_recharge_a_l_identique` : les lignes écrites avant la carte (`bonus_credites=fin,monde2_fin,monde2_toutes_les_gouttes`, `partie.monde2=parties:3 score:700 temps:4000`) se relisent dans les nouvelles structures et se réécrivent pareil ; les bonus du Monde 2 déjà crédités le restent ; aller-retour texte → sauvegarde → texte exact à l'octet.

Format de sauvegarde : INCHANGÉ (pas d'arrêt nécessaire). Mêmes clés, même place des lignes, `version=3` ; les lignes d'un futur Monde 3 (`partie.monde3`, `monde3_*`) sont de simples clés en plus, qu'un jeu plus ancien ignore.

- Preuves : `lint` LINT-OK ; `test --workspace --no-default-features` VERT (core 314, game 424, niveaux 16 + 89, outils 9, robot 4) ; game avec `debug_niveaux,qa_plantage` 423 VERTS. Captures (`docs/capture-112.sh` : départ et milieu, après 150 images de course, des 19 niveaux des Mondes 1 et 2, plus 6 vues de la carte des niveaux tout ouverte du build de QA, chapitre 2 de la Pieuvre à la rue puis chapitre 1 jusqu'aux toits), build de QA de `e6f77de` construit à part AVANT toute modification puis celui de la fin, `comparer-captures.py` par paires : 44/44 IDENTIQUES, 0 pixel différent. Regardées : chauve-souris en vol à la rivière (`n16-milieu`, copiée dans `docs/qa/112-riviere-chauve-souris.png`), fond de l'ascension au chapitre 1 (`carte-e`, `carte-f`), fond de la descente au chapitre 2 (`carte-a` à `carte-c`, `docs/qa/112-carte-chapitre-2.png`).
- Limites : le TITRE d'un chapitre 3 (`contenu::titre_chapitre`) et l'INTRO de ses niveaux (`contenu::INTROS_MONDE_2`, rangées par clé) sont du texte du jeu, pas une donnée des `.ron` : sans eux, le bandeau affiche « CARTE DES NIVEAUX » et l'intro retombe sur celle du dernier niveau du Monde 1 (pas de plantage, mais un texte faux ; une carte de suite pourrait porter l'intro dans le `.ron`, le champ `intro` y existe déjà). Un Monde 3 qui voudrait un AUTRE fond de carte que l'ascension ou la descente, ou de nouveaux thèmes, demande du code (les thèmes sont un `enum`). La porte d'un monde suit désormais la table : si un niveau normal s'ajoute un jour APRÈS `poursuite` au Monde 1, il deviendra la porte du Monde 2 (le test `la_porte_du_monde_2_est_poursuite` le signalera).
- Pas testé : joueur humain, téléphone, Safari.

## Carte 11.3 (AGA) : le texte d'intro et l'astuce dans le fichier du niveau (plus de `INTROS` en Rust)

But : plus aucune donnée d'un niveau écrite en Rust ; un niveau s'ajoute avec son seul fichier. Origine : la carte 9.32 (la marée, faite à l'éditeur) bloquait sur l'astuce, qui n'existait que dans `INTROS_MONDE_2`.

Fait :
- Format : le champ `intro` devient DEUX champs, `intro_texte` (la phrase, en blanc) et `intro_astuce` (le conseil, en corail), juste après `duree_cible_s`, dans les 34 fichiers `.ron` (12 du Monde 1, 7 du Monde 2, 15 essais, dont `essai_balancement` de la 11.1). Textes recopiés par script des listes du jeu (`INTROS`, `INTROS_MONDE_2`, `INTRO_ESSAI_*`), à l'octet. `mecanique` reste (note de conception, jamais affichée).
- Cinq essais n'avaient AUCUNE intro dans le code (eau, eau2, ennemis_eau, pieuvre, balancement : leur carte affichait celle du dernier niveau) : ils ont maintenant la leur, tirée de leur ancien champ `intro` et de leur `mecanique`, raccourcie pour tenir sur la carte.
- Chargeur (`niveaux/src/intro.rs`, nouveau) : un niveau dont `intro_texte` ou `intro_astuce` est vide (ou blanc) est REFUSÉ, de même qu'une intro trop longue pour l'écran : lignes de 46 caractères coupées entre les mots (comme `couper_lignes`), phrase sur 3 lignes au plus, conseil sur 2, 4 lignes ensemble (la ligne « sans perte de vie » du bonus comptée). Ces limites sont celles de la mise en page de `ecrans/carte.rs` (y = 88, 10 px par ligne, fin avant « continuer » à y = 134), qui lit maintenant `niveaux::intro::LIGNE_MAX_INTRO`. Messages : « monde_1/toits.ron : « intro_texte » est vide : chaque niveau a son intro (la phrase dans intro_texte, le conseil dans intro_astuce) », « « intro_astuce » prend 3 lignes sur la carte du niveau, 2 au plus (lignes de 46 caractères) » ; un ancien fichier avec `intro` reçoit en plus « l'intro du niveau s'écrit en deux champs, après duree_cible_s : intro_texte, la phrase, puis intro_astuce, le conseil ».
- Jeu : `contenu::Intro`, `INTROS`, `INTROS_MONDE_2`, les 10 `INTRO_ESSAI_*`, `contenu::intro(indice)`, `essai_externe::intro()`, le contrôle de compilation (`intros_du_monde_2_completes`, `meme_texte`) et la constante générée `CLES_HORS_MONDE_1` (qui ne servait qu'à lui) sont SUPPRIMÉS. `carte::dessiner(infos, ...)` lit `infos.intro_texte` et `infos.intro_astuce` (plus d'indice). Le niveau posé par l'éditeur montre donc SA phrase et SON conseil (avant : `intro` et `mecanique`).
- Éditeur : champs `intro_texte` (3 lignes) et `intro_astuce` (2 lignes) dans le formulaire, avec sous chacun l'aperçu des lignes coupées comme dans le jeu et le compte « 2/3 lignes de 46 car. » (rouge au-delà) ; mêmes refus que le chargeur dans les avertissements (`erreursIntro` de `editeur/ron.js`) ; export canonique des deux champs à leur place ; un fichier avec l'ancien `intro` est refusé avec le nom des deux nouveaux. Le test `l_editeur_a_les_memes_limites_d_intro` relit les constantes de `ron.js` (46, 3, 2, 4) depuis le Rust. `niveaux/outils/reporter_commentaires.py`, `LISEZMOI.md` (tableau des champs, exemple, règles) et `README.md` mis à jour.

Tests :
- AVANT la suppression (commit `b82c22f`) : `l_intro_du_fichier_est_celle_des_listes` : pour les 19 niveaux du parcours et les 10 essais qui avaient une intro, (`intro_texte`, `intro_astuce`) lu dans le fichier = (`texte`, `astuce`) de `contenu::intro(indice)` : VERT (29/29). Le test est parti avec les listes au commit suivant.
- `un_niveau_sans_intro_est_refuse_clairement` (niveaux) : texte vide, conseil en blancs, conseil de 3 lignes, 3 + 2 lignes, champ manquant, ancien champ `intro` ; message exact vérifié ; comptage des lignes aux bornes (46 car., 44 + 1 + 1, 44 + 1 + 2).
- Aller-retour de l'éditeur : `node editeur/test-aller-retour.mjs` : « 34 fichiers relus et réécrits : ALLER-RETOUR OK » (les 34 `.ron` présents, octet pour octet), plus : chaque fichier passe `erreursIntro`, saisie puis export puis relecture, et les quatre refus.
- Dans le navigateur (build de QA, `editeur.html`) : formulaire avec `intro_texte` et `intro_astuce`, le caniveau ouvert montre « 2/2 lignes » et ses deux lignes ; une astuce de 23 mots passe l'aperçu en rouge (« 3/2 lignes ») et ajoute l'avertissement « « intro_astuce » prend 3 lignes sur la carte du niveau, 2 au plus ».

Inventaire : données propres à un niveau encore écrites en Rust (grep des 19 clés et de `cle ==` hors tests)

| Donnée | Où | Décision |
| --- | --- | --- |
| Intro et astuce | `contenu.rs` (`INTROS`, `INTROS_MONDE_2`, `INTRO_ESSAI_*`) | DÉPLACÉE (cette carte). |
| Musique | `musiques.rs` `musique_du_niveau` | Reste, parce que c'est une donnée du THÈME (`match niveau.theme`) : un nouveau niveau d'un thème existant a sa musique sans code ; un nouveau thème demande du code de toute façon (`Theme` est un `enum`). |
| Défis | `defis.rs` `defis_du_niveau` | Reste, parce que c'est une RÈGLE qui lit déjà le fichier (`sans_planer`, `duree_cible_s`) ; la clé de prime `course_expert_{cle}` est calculée. |
| Records, progression, médailles, lettre trouvée | `sauvegarde.rs` (par clé) | Reste, parce que ce sont des données du JOUEUR rangées par clé, pas des données du niveau. |
| `ORDRE_V2` (11 clés) | `sauvegarde.rs:51` | Reste, parce que c'est la table FIGÉE de migration des sauvegardes v2 vers v3 : la changer casserait la relecture des anciennes sauvegardes. |
| Décors et fonds (rue, égouts, cascade, grotte, rivière, port, ciels, `fonds::dessiner("caniveau", ...)`) | `decor.rs` (`match theme`), `decor_*.rs`, `fond_egouts.rs` | Reste, parce que c'est une donnée du THÈME (dossier `assets/fonds/<thème>`), choisie par `infos.theme`. |
| Dessin de la jetée (phare, 3 bateaux amarrés à des x fixes) | `dessin_port.rs` `CLE_JETEE = "pieuvre"`, `BATEAUX` | À DÉPLACER : vrai cas d'UN niveau (le port et la Pieuvre ont le même thème `Port`). Proposition : un champ facultatif du `.ron` (par ex. `habillage: [Jetee]`, les bateaux en cases). |
| Plateformes mobiles dessinées en bateaux et caisse de grue | `dessin_port.rs` `CLE_PORT = "port"`, `sprite_du_port` | À DÉPLACER, même raison (même thème que la Pieuvre) : champ facultatif (par ex. `mobiles: Bateaux`). |
| Lettres cachées P-É-P-I-N (toits, marché, orage, clocher, mer) | `core/src/lettres.rs` `LETTRES` | À DÉPLACER : quelle lettre porte un niveau est une donnée du niveau (la case `*` est déjà dans sa grille). Proposition : champ `lettre: 'P'`, le rang suivant l'ordre du parcours. Sans changer la sauvegarde (la lettre trouvée y est notée par clé de niveau, `lettre:1`). |
| Variantes de nouvelle partie + (ennemis déplacés, ajoutés, 10 niveaux) et vent de face des 4 niveaux de toits | `core/src/levels/variantes.rs` `VARIANTES`, `NIVEAUX_DE_TOITS` | À DÉPLACER : cases de LA grille d'un niveau, écrites à la main en Rust ; proposition : un champ facultatif `ng_plus: (deplaces: [...], ajouts: [...], vent_de_face: true)`. Rien de sauvegardé (la sauvegarde ne note que `ng_plus=...` et les temps par clé). |
| Mesure de difficulté (`RANGS_DE_GRAINE`, `BANDES_HABITUE`, `DUREE_BONUS_EXPERT_S`) | `difficulte.rs` | Reste, parce que c'est l'outil de mesure de la courbe du Monde 1, compilé en TEST seulement (`#[cfg(test)] mod difficulte`) : rien dans le jeu ; un niveau ajouté au Monde 1 doit y être mesuré exprès (le test le demande). |
| Parcours du robot QA (`scripts.rs`, `pilote_pieuvre.rs`, `captures_99.rs`) | `game/src/bot_qa/` | Reste, parce que ce sont des outils de vérification (build de QA), pas des données du jeu. |
| Titres de chapitre | `contenu.rs` `titre_chapitre` | Hors périmètre : donnée du MONDE, pas d'un niveau (déjà signalé en 11.2). |

Format de sauvegarde : aucune donnée de l'inventaire ne demande de le changer (les quatre « à déplacer » sont des données de dessin ou de grille, jamais sauvegardées, ou, pour les lettres, déjà notées par clé de niveau). Pas d'arrêt nécessaire. Les quatre déplacements sont des cartes de suite (format `.ron` élargi, éditeur, tests), pas faites ici.

Preuves :
- `lint` : FMT-OK, clippy core, niveaux, robot (natif et wasm32), outils, game (wasm32, natif, natif QA, wasm32 QA) : LINT-OK.
- `test --workspace --no-default-features` VERT : core 324, game 433, niveaux 18 + 90, outils 9, robot 4 ; game avec `debug_niveaux,qa_plantage` : 432 VERTS.
- Captures (`docs/capture-113.sh`, nouveau : la carte de présentation de chacun des 19 niveaux des Mondes 1 et 2, 60 images après son ouverture, pilote de `capture-102.mjs`) : build de QA de `6c99c23` (avant la carte, tiré par `git archive` et construit à part) puis celui de la fin, `comparer-captures.py` : 19/19 IDENTIQUES, 0 pixel différent, 0 erreur console (Monde 1 : les toits ; Monde 2 : le caniveau, la Pieuvre... vérifiés à l'œil).
- `./build.sh` : OK, « sons : 19 WAV canoniques, aucun nom interdit : OK », « éditeur absent de dist/ : OK », `game.wasm` 1 079 818 octets.
- Limites : la marée (9.32) n'est pas faite ici ; elle peut maintenant porter son intro et son astuce dans son fichier. Les cinq intros d'essai nouvelles sont de mon écriture (niveaux de QA seulement, jamais vus par le joueur).
- Pas testé : joueur humain, téléphone, Safari.

## Carte 9.32 : le niveau de la marée, fait entièrement à l'éditeur (ARRÊT : un test de la carte des niveaux, du code)

Fait :
- `assets/levels/monde_2/maree.ron` (clé `maree`, Monde 2 n° 7, thème `Port`, 166 x 11, `duree_cible_s: 70`) : dessiné, réglé, validé, essayé (TESTER), passé au ROBOT et exporté DANS L'ÉDITEUR du build de QA, piloté par Chromium sans écran (`node docs/capture-932.mjs PORT DOSSIER`, `CHROME=…/chromium-1208`, `dist-qa/` servi sur 127.0.0.1, port libre). Les gestes (rectangles et coups de pinceau à la souris) sont dans `docs/maree-gestes.json` ; le script vérifie que la grille de l'éditeur est celle du plan. Le fichier du dépôt est l'export, octet pour octet sous l'en-tête de commentaires (écrite à la main, voir le journal) ; l'aller-retour `lireNiveau`/`ecrireNiveau` de `editeur/ron.js` le redonne identique.
- La marée en vedette (`=`, thème existant, mécaniques existantes) : bassin sec (à pied à mer basse, goutte dans une niche) ; écluse (deux bassins sur l'eau du port, trop larges pour un plané : en barque, on ATTEND que la mer monte pour remonter au quai) ; chenal des nénuphars (deux poissons) ; passage de basse mer sous une voûte (à pied au fond, ou en barque à mer haute) ; deux pontons à crabe et un poisson. Lanternes 29, 66, 104, 130 (départ 2, drapeau 162 : au plus 38 colonnes sans point de reprise). Trois gouttes.
- La Pieuvre passe 8e : ouverte dans l'éditeur (liste des niveaux du jeu), `numero` 7 -> 8, exportée (seule cette ligne change, vérifié par le script).
- Intro et astuce saisies dans l'éditeur (aperçu : 2/3 et 2/2 lignes). `sans_planer: true` : verdict du ROBOT « sans planer », recopié par le bouton de l'éditeur.
- Test du niveau (`game/src/maree_sauvegarde.rs`, test seulement, une ligne `mod` dans `main.rs`) : une sauvegarde d'un joueur qui avait fini les 7 niveaux d'avant (Pieuvre comprise) et une partie du Monde 2 se relit et se réécrit à l'identique, ses 7 étapes restent « Finie », le Monde 2 jouable, ses records gardés, la porte d'un monde suivant reste la Pieuvre (faux Monde 3 de la 11.2) ; la marée est « Ouverte » et sa progression vide (jamais jouée) ; la carte des niveaux le remet sur la Pieuvre. Et la marée est bien entre le port et la Pieuvre.

Preuves :
- Éditeur (`capture-932.mjs`, CAPTURE-932-OK, 0 erreur de console) : Valider « accepté par le chargeur du jeu » ; ROBOT « arrivée atteinte en 35.1 s de jeu, 32 sauts, lanternes allumables 4/4, gouttes 3/3 » (10 s de calcul, 729 états) ; ROBOT sans planer : arrivée (35.1 s). Chemin du robot : bassin sec à pied au fond, écluse en barque avec une attente de marée (pieds de rangée 8,2 à 5,2 avant le duc d'Albe), nénuphars, voûte en barque à mer haute.
- `lourd.sh ./docker-cargo.sh carte maree robot` : ROUGE pour UN test seulement (voir l'arrêt). core 324, niveaux 18, `tous` 90 rapides + `faisabilite::le_niveau_de_la_carte` (faisabilité, durée, reprise depuis chaque lanterne) VERTS ; game 434 rapides verts, 1 rouge ; robot de QA `le_robot_finit_le_niveau_de_la_carte` VERT (244 s : arrivée sans mourir).
- Mesure, une fois (`CLE=maree timeout 600 lourd.sh ./docker-cargo.sh test -p game --release --no-default-features mesure_d_un_niveau_du_monde_2 -- --ignored --nocapture`, 15 989 cases, 177 s) : Débutant 0,35 morts (39/48 finis, 110,2 s ; col. 148-150 ennemi 4, 70-71 ennemi 3, 144-145 ennemi 3, 44-45 noyade 2, 94 ennemi 2) ; Habitué 0,19 (42/48 finis, 85,3 s, max 1 ; col. 149 ennemi 3, 75 ennemi 2, 144-145 ennemi 2, 14 noyade 1, 72 ennemi 1) ; Expert 0,02 (42/48, 82,6 s). CIBLE NON ATTEINTE : Habitué 0,19, sous le port (0,69) au lieu d'être entre le port et la Pieuvre (1,90), et 85 s au lieu de 60 à 75. Pas retouché, pas remesuré (une seule mesure). Les morts sont aux crabes et poissons des pontons ; la marée elle-même ne tue presque pas (2 noyades sur 48 Habitués) : le joueur simulé attend la bonne marée. Pistes si AGA le veut : bassins secs plus longs (la fenêtre de mer basse serrée), ou un poisson dans l'écluse.
- Captures (`docs/qa/932-*.png`, regardées) : l'éditeur (`932-editeur-dessin`, `932-editeur-robot` avec le chemin du robot), la carte du niveau (titre « LA MARÉE », intro en blanc, astuce en corail, 4 lignes), le départ, le bassin sec, l'écluse (barque, quais de pierre, duc d'Albe), les nénuphars, Pépin à pied sous la voûte à mer basse (goutte sous la voûte), les pontons. Rien de cassé à l'œil ; dans l'essai de l'éditeur la carte dit « NIVEAU 7/7 » (l'essai joue le niveau seul).

ARRÊT (règle de la carte : « si le code doit changer, s'arrêter et dire pourquoi ») :
- Test rouge : `ecrans::fond_plan::tests::les_bandes_du_chapitre_2_suivent_les_niveaux` : « pieuvre (Port) : 70 hors de 92..158 ».
- Cause : le fond du chapitre 2 de la carte des niveaux (`bandes_chapitre_2`, `game/src/ecrans/fond_plan.rs:465`) centre chaque bande de décor sur la MOYENNE des étapes de son thème, et la bande du port s'arrête 22 px (`AU_DESSUS_DU_PORT`) au-dessus de ce centre. Avec trois niveaux dans la bande du port (port, marée, Pieuvre : thèmes `Port` ou `Maree`, tous deux dans `Bande::Port`), le centre descend d'une étape (44 px) et l'étape de la Pieuvre tombe dans la bande de la mer. Aucun thème ne passe : calcul fait pour les 7 thèmes du Monde 2 (copie en Python de `Geometrie::new` et de `bandes_chapitre_2`) : chacun rend le test rouge (un thème d'une autre bande casse l'ordre des bandes). Ce n'est donc pas une donnée du niveau : « ajouter un niveau = ajouter un fichier » ne tient pas quand un chapitre a plus de deux niveaux d'un même thème.
- Correctif proposé (carte de code, petite) : borner chaque bande par les étapes EXTRÊMES de son thème (haut = étape la plus haute - `AU_DESSUS_DU_PORT`, et les limites entre bandes au milieu de deux étapes voisines de thèmes différents) au lieu de la moyenne. Puis relancer `carte maree` (rien d'autre à refaire : le niveau, la mesure, le robot et les captures ne dépendent pas de ce dessin).
- Commité : tout le reste (niveau, Pieuvre 8e, test de sauvegarde, script et gestes, captures, ce rapport).
- RÉSOLU par la carte 9.32b (plus bas) : `bandes_chapitre_2` accepte N niveaux par thème ; le test est vert.

Journal de l'éditeur (tout ce qui a été fait hors de l'éditeur, ou qui a gêné ; la liste des corrections avant le Monde 3) :
1. (1er arrêt, résolu par la carte 11.3) L'astuce d'intro d'un niveau hors Monde 1 était écrite en Rust (`INTROS_MONDE_2`) et l'éditeur ne la voyait pas. Depuis la 11.3, `intro_texte` et `intro_astuce` se saisissent dans l'éditeur, avec l'aperçu des lignes : ça a marché du premier coup.
2. (2e arrêt) La carte des niveaux du chapitre 2 refuse un 3e niveau du même thème (ci-dessus) : du code, invisible depuis l'éditeur, découvert seulement par `carte maree`.
3. L'en-tête de commentaires de conception (le POURQUOI du niveau) a été écrite À LA MAIN : l'éditeur garde les commentaires d'un fichier ouvert mais ne sait pas en écrire. Seul geste hors de l'éditeur dans le `.ron`.
4. L'export part dans le dossier de téléchargement : le copier dans `assets/levels/monde_2/` (et la Pieuvre aussi) se fait hors de l'éditeur.
5. La hauteur : ma 1re version faisait 13 rangées. L'éditeur n'a rien dit ; `carte maree` a échoué (`camera::tests::les_niveaux_d_un_ecran_gardent_la_meme_camera`) : un niveau de plus d'un écran de haut doit être dans une LISTE DE CLÉS écrite en Rust (`game/src/camera.rs`, test). Refait à 11 rangées. Pour le Monde 3 : un niveau haut demandera une ligne de Rust (de test), ou l'éditeur devrait le dire.
6. Deux règles des tests de tout niveau ne sont pas dans les avertissements de l'éditeur : « quelque chose d'intéressant sur l'écran de départ » (`premier_defi_visible_des_le_premier_ecran` : ma 1re goutte était à la colonne 26, hors des 320 px) et « au moins un objet de décor » (`le_decor_ne_change_rien_a_la_collision`) ; découvertes par `carte`, corrigées dans l'éditeur (goutte dans la niche de gauche, deux oiseaux posés `K`).
7. Rien dans l'éditeur pour les règles de rythme : « au plus 45 cases sans lanterne » compté à la main ; la durée vient du ROBOT (35,1 s de course), pas de repère de la cible 60 à 75 s ni de la difficulté (la mesure n'existe qu'en Rust, une fois, à la fin : ici elle a dit « trop facile » trop tard pour retoucher).
8. La marée ne se voit pas dans l'éditeur : les cases `=` sont une couleur, sans le niveau de mer basse et haute ni la hauteur de saut ; les hauteurs (saut 57 px, barque 11 px au-dessus de l'eau, quai à 67 px de l'eau basse) ont été calculées à la main depuis `core/src/physics.rs`. Un calque « mer haute / mer basse » et la portée d'un saut aideraient.
9. TESTER et « partir d'ici » ouvrent d'abord la carte du niveau (Entrée à chaque essai) ; l'essai dit « NIVEAU 7/7 » (il ne connaît pas la table). Gênes mineures.
10. Le test de sauvegarde du niveau vit dans le jeu (crate binaire, pas de `tests/`) : une ligne `mod maree_sauvegarde;` dans `game/src/main.rs`, hors des fichiers du niveau.
11. Hors éditeur, normal : `./build.sh --qa` (le build de QA à servir), un serveur local, `carte`, le robot de QA et la mesure en ligne de commande.

Pas testé : joueur humain, téléphone ; le lot complet (`./docker-cargo.sh lot`) ; la carte des niveaux à l'écran (test rouge seulement).

## Carte 9.32b : la carte des niveaux accepte plusieurs niveaux du même thème (levée du 2e arrêt de la 9.32)

Fait :
- `game/src/ecrans/fond_plan.rs`, `bandes_chapitre_2` : la règle pour N niveaux par thème (écrite dans le commentaire de la fonction).
  1. Le centre d'une bande reste la moyenne de ses étapes. Une bande sans niveau se place entre les étapes les plus PROCHES de ses voisines (avant : entre leurs centres), ou, au bout, à des demi-écarts de l'étape la plus proche.
  2. La limite entre deux bandes voisines reste le milieu de leurs centres, puis elle est poussée pour laisser `MARGE_ETAPE` (11 px, un quart d'écart) entre elle et les étapes des DEUX bandes ; la bande du dessous l'emporte si les deux ne tiennent pas.
  3. Le haut de la dernière bande (le port) reste `AU_DESSUS_DU_PORT` (22 px) au-dessus de son centre, mais jamais sous sa plus haute étape : la Pieuvre se tient sur la ligne de la mer, comme avant la marée (avec 2 niveaux au port, les deux valeurs sont égales : rien ne bouge).
- Le Monde 2 d'aujourd'hui (8 niveaux, port + marée + Pieuvre au port) : étapes du port à 158, 114 et 70 ; rivière/port à 169 (milieu des centres 158, poussé à 158 + 11) ; haut du port à 70 (au lieu de 92 : la Pieuvre, à 70, était dans la mer) ; la mer de 0 à 70.
- Test nouveau `trois_niveaux_d_un_meme_theme_puis_un_autre` (même fichier) : quatre mondes d'essai (3 caniveaux puis égouts ; 3 grottes puis port, la rivière vide entre les deux ; 3 rivières puis port ; rivière, port, marée, port). Chaque étape est dans la bande de son thème, à 11 px au moins de chaque limite (sauf la plus haute du port, sur la ligne de la mer), les bandes se touchent, couvrent la carte, aucune n'est à l'envers. Les tests d'avant (`les_bandes_du_chapitre_2_suivent_les_niveaux`, `le_faux_monde_3_a_un_fond_de_carte`, `aucun_theme_du_monde_1_n_a_de_bande`) n'ont pas changé.
- `docs/capture-932b.sh` (nouveau) : la carte des niveaux du build de QA (tout ouverte), une capture à l'ouverture (la Pieuvre) puis après chaque Bas, 23 en tout, du haut du chapitre 2 aux toits du chapitre 1 (pilote de `capture-102.mjs`).

Vérifié :
- `lourd.sh ./docker-cargo.sh carte maree` : VERT (core 324, game 436 rapides dont `les_bandes_du_chapitre_2_suivent_les_niveaux` et le test nouveau, niveaux 18, `tous` 90 + `faisabilite::le_niveau_de_la_carte`). `carte pieuvre` : VERT, mêmes comptes.
- `lourd.sh ./docker-cargo.sh lint` : FMT-OK, LINT-OK (clippy natif, wasm32 et fonctionnalités de QA).
- Captures (`docs/capture-932b.sh`, Chromium 1208 sans écran, 0 erreur de console sur les quatre passages), quatre builds de QA construits à part (`git archive`) :
  - Avant la marée (`c4cc6a2`, 7 niveaux au chapitre 2) contre le même avec le nouveau `fond_plan.rs` : 23/23 IDENTIQUES au pixel (`comparer-captures.py`). La règle ne change rien aux cartes d'avant (Monde 1 et Monde 2 à 7 niveaux).
  - Avec la marée (`8da5621`) contre le même avec le correctif : 17/23 identiques (les égouts, le caniveau et tout le chapitre 1) ; les 6 différentes sont les vues du haut du chapitre 2 (Pieuvre, marée, port, rivière, grotte, cascade), là où la bande du port grandit et celle de la mer recule. Regardées : avant (`docs/qa/932b-carte-pieuvre-avant.png`), la Pieuvre est sous le soleil levant, en pleine mer, le port écrasé sous elle ; après (`932b-carte-pieuvre-apres.png`), elle est à la même place que la Pieuvre d'avant la marée (sur la ligne de la mer, sous l'horizon), la marée dans l'eau du port au-dessus de la grue. `932b-carte-port-avant/apres.png` : le port sur son quai, la marée au-dessus, plus de soleil coupé en haut de la vue.

Pas testé : le lot complet (`./docker-cargo.sh lot`) ; téléphone ; joueur humain. Rien d'autre à refaire pour la marée (le niveau, la mesure, le robot et les captures de la 9.32 ne dépendent pas de ce dessin).
