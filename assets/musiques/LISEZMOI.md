# Les sons de PÉPIN

Depuis la carte 10.4e, chaque musique et chaque bruitage est un VRAI fichier audio,
seule source du son :

- `assets/musiques/<id>.wav` : une boucle par musique du catalogue
  (`game/src/musiques.rs`, `Musique::id`) : `orage`, `toits_de_nuit`, `soleil`,
  `valse_des_nuages`, `cyclone`, `poursuite` ;
- `assets/sons/<clé>.wav` : un bruitage par effet (`core/src/sons.rs`, `Effet::cle`),
  plus la boucle de pluie `pluie.wav`.

Ils ont été rendus une dernière fois par l'ancien synthétiseur 8 bits du jeu (commit
`ef0283c`), puis le synthétiseur a été retiré : ces fichiers sont, à l'échantillon
près, le son d'avant.

## Comment le jeu les charge

Les fichiers ne sont PAS dans `game.wasm` (3 Mo : le jeu serait trois fois plus
lourd à télécharger avant l'écran titre).

1. `game/build.rs` écrit la LISTE des `.wav` de ces deux dossiers (aucune liste à la
   main) ; un test (`audio::tests::chaque_son_demande_a_son_fichier`) vérifie que
   chaque son demandé a son fichier et qu'aucun fichier n'est orphelin.
2. `./build.sh` les copie à côté du jeu : `dist/musiques/`, `dist/sons/` (et vérifie
   que ce sont des WAV canoniques sans nom interdit).
3. Le jeu ne télécharge RIEN tant que le son est coupé (il l'est par défaut : un
   joueur qui ne l'active jamais ne télécharge aucun son). À la PREMIÈRE activation
   (touche M, icône du son ou menu), il télécharge EN ARRIÈRE-PLAN, trois à la fois :
   le bip des menus, la musique de la zone en cours, la pluie, puis les bruitages.
   Une autre musique ne part que quand sa zone arrive (en tête de file). Son recoupé :
   plus aucune nouvelle requête. Règle et tests : `audio::FileDeSons` (carte 10.4f).
   Tant qu'un son n'est pas arrivé, il est muet ; une boucle qui arrive pendant que
   le son est actif démarre aussitôt. Un fichier absent est signalé dans la console
   et reste muet : le jeu continue.

Hors du web (`cargo run` depuis la racine du dépôt), le jeu lit `assets/` directement.

## Format

- WAV PCM 16 bits, MONO, 22 050 Hz (la fréquence de l'ancien synthé).
- CANONIQUE : un en-tête de 44 octets, les blocs `fmt ` et `data` et RIEN d'autre :
  aucun bloc `LIST`/`INFO`, aucune métadonnée (ni titre, ni logiciel, ni nom), aucun
  octet après les données.
- Pourquoi du WAV et pas de l'OGG Vorbis (essayé d'abord, 6 fois plus léger) :
  Chromium ne coupe pas le remplissage de fin du Vorbis (un blanc de 7 à 22 ms à
  chaque tour de boucle, mesuré), et Safari sur iPhone lit mal l'OGG. Le WAV est lu
  par tous les navigateurs, à l'échantillon près : les boucles sont exactes. Il se
  compresse un peu à l'envoi (3,1 Mo, 2,4 Mo en gzip -9).

## Règles (vérifiées par les tests)

1. Format ci-dessus : `audio::tests::chaque_son_est_un_wav_mono_16_bits_sans_metadonnees`
   (crate `game`) et `son::tests::chaque_son_est_un_wav_mono_sans_metadonnees` (crate
   `outils`, qui refuse aussi tout autre fichier, `.ogg` ou `.mp3`, dans ces dossiers).
2. Boucles SANS COUTURE (musiques et pluie) : le saut entre le dernier échantillon et
   le premier n'est pas plus fort que les sauts du son lui-même (aucun clic), et une
   musique part et finit presque dans le silence
   (`son::tests::chaque_boucle_se_raccorde_sans_couture`).
3. Taille : 600 000 octets au plus par musique, 100 000 par bruitage, 3,5 Mo en tout
   (`./build.sh` refuse au-delà). Total actuel : 3 096 644 octets.
4. Équivalence avec l'ancien synthé : `son::tests::chaque_son_est_le_rendu_du_synthe_d_origine`
   garde la longueur et l'empreinte des échantillons de chaque fichier.

## Remplacer une musique (ou un bruitage)

1. Préparer un WAV 16 bits mono à 22 050 Hz SANS métadonnées. Avec ffmpeg :
   `ffmpeg -i source.wav -ac 1 -ar 22050 -c:a pcm_s16le -map_metadata -1 -fflags +bitexact -flags:a +bitexact assets/musiques/<id>.wav`.
   Pour une boucle : la fin doit retomber sur le début (même phrase musicale,
   dernière note éteinte), et la durée être celle d'un tour exact.
2. Le contrôler :
   `./docker-cargo.sh run -p outils --bin son -- controler assets/musiques/<id>.wav`
   affiche taille, durée et raccord de la boucle ; il sort en erreur si le fichier
   n'est pas un WAV canonique mono 16 bits à 22 050 Hz.
3. Mettre à jour sa ligne dans `son::tests::chaque_son_est_le_rendu_du_synthe_d_origine`
   (le test donne la nouvelle longueur ; l'empreinte se lit dans son message), puis
   `./docker-cargo.sh test -p outils` et écouter dans le jeu.

Une NOUVELLE musique demande aussi son entrée dans le catalogue
(`Musique::TOUTES`, `id`, `nom`, `prix`) ; un nouveau bruitage, son `Effet` (avec sa
`cle`) et l'endroit du jeu qui le joue.

## Non testé

Un vrai iPhone (Safari) : le WAV y est lu par tous les navigateurs connus, mais le
jeu n'y a pas été écouté.
