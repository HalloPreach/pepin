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

## Mesure de difficulté

Outil ajouté par la carte 3.1 (`game/src/difficulte.rs`, compilé pour les tests seulement). Il répond à
« combien de fois meurt-on dans chaque niveau ? » par un chiffre reproductible, à comparer avant/après
chaque carte qui touche un niveau, un ennemi ou une mécanique.

### Commande

    ./docker-cargo.sh test -p game --no-default-features mesure_de_difficulte -- --ignored --nocapture

Elle affiche les tableaux et réécrit `docs/mesure-difficulte.txt` (fichier versionné : `git diff` montre
l'effet d'une carte). Durée : 30 à 90 s sur la machine de développement (un fil d'exécution par niveau
puis par niveau et profil ; la durée dépend de la charge). Le test échoue si un profil finit moins de la
moitié de ses essais sur un niveau, si l'ordre Débutant > Habitué > Expert n'est plus respecté sur le
total, ou si la forme de la courbe sort de ses bandes (voir ci-dessous).

Depuis la carte 3.3, le fichier a une 3e partie, « Lieux de mort » : pour chaque niveau et chaque profil,
les endroits où l'on meurt (colonne de tuile du centre de Pépin, cause : chute, ennemi, pointes, éclair),
colonnes voisines de même cause regroupées, du plus meurtrier au moins meurtrier. C'est là qu'il faut
agir pour adoucir ou corser un niveau.

Deux autres tests :
- `courbe_de_difficulte_dans_les_bandes` (lancé avec tous les tests, environ 30 s) : non-régression de la
  forme de la courbe sur la mesure officielle. Débutant et Habitué : le niveau 3 est le plus meurtrier des
  cinq, le 4 l'est plus que le 5. Habitué : chaque niveau dans sa bande (`BANDES_HABITUE` : N1 [0 ; 0,3[,
  N2 [0,3 ; 0,8[, N3 [0,5 ; 1,5[, N4 [0,3 ; 0,65[, N5 [0 ; 0,2[) et le 4 au moins 0,15 sous le 3. En cas
  d'échec, le message liste tous les écarts et le tableau.
- `bruit_de_la_mesure` (ignoré, environ 2 min 30) : la même mesure avec 4 graines (l'officielle et ses 3
  voisines) et les lieux de mort cumulés sur toutes. Sert à savoir si un écart est réel ou dû au hasard :
  une différence plus petite que l'écart entre graines ne prouve rien. (Une variable `GRAINES=a,b,c`
  choisit d'autres graines, mais `docker-cargo.sh` ne la transmet pas au conteneur : seulement avec un
  `cargo` local.)

      ./docker-cargo.sh test -p game --no-default-features bruit_de_la_mesure -- --ignored --nocapture

Pour le tableau avant/après d'une carte : lancer la commande sur une extraction propre du commit de départ
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
n'importe où ailleurs sur le toit).

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
