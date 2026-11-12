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
| 3 | Pas de nom de Lucas, pas de lien portfolio | RÉUSSI | `lucas`, `tixier`, `elsay`, `portfolio`, `github` : 0 occurrence dans `game.wasm` et `index.html` ; un test (`aucune_mention_hors_sujet`) interdit `http`, `www.`, `lucas`, `tixier`, `@` dans tous les textes |
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
