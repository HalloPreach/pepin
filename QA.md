# QA finale de PÉPIN

Date : 12/11/2026. Tout ce qui suit a été mesuré ou observé pendant cette passe ;
ce qui n'a pas pu être vérifié est dit explicitement (section « Limites »).

Méthode : le dernier commit (`1acb71f`) a été extrait avec `git archive` dans un
dossier vierge, puis `./docker-cargo.sh lint`, les tests et `./build.sh` y ont été
lancés. Les empreintes SHA-256 de `dist/` de cette extraction et de `dist/` du
dépôt sont identiques (build reproductible). Le jeu a ensuite été joué dans un
vrai navigateur (Chromium piloté par le protocole CDP) servi par `python3 -m http.server`.

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

- `game/src/bot_qa.rs` (compilé uniquement pour les tests) : le robot. `cargo test -p game --no-default-features -- --ignored ecrire_script_navigateur` écrit `target/qa/script.txt` (un chiffre hexadécimal par image : 1 Entrée, 2 gauche, 4 droite, 8 saut) et `trace.csv`. Un petit pilote JavaScript, collé dans la console, rejoue ce script avec une horloge virtuelle.
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

## Corrections des niveaux et textes (audit) — carte 4/4

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
