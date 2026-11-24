# Les sons de PÉPIN

Depuis la carte 10.4e, chaque musique et chaque bruitage est un VRAI fichier audio,
seule source du son :

- `assets/musiques/<id>.ogg` : une boucle par musique du catalogue
  (`game/src/musiques.rs`, `Musique::id`) : `orage`, `toits_de_nuit`, `soleil`,
  `valse_des_nuages`, `cyclone`, `poursuite` ;
- `assets/sons/<clé>.ogg` : un bruitage par effet (`core/src/sons.rs`, `Effet::cle`),
  plus la boucle de pluie `pluie.ogg`.

`game/build.rs` embarque tous les `.ogg` de ces deux dossiers dans le jeu
(`include_bytes!`, aucune liste à la main) ; `game/src/audio.rs` les confie à
macroquad (`load_sound_from_bytes`), et c'est le navigateur qui les décode. Un test
(`audio::tests::chaque_son_demande_a_son_fichier`) vérifie que chaque son demandé a
son fichier et qu'aucun fichier n'est orphelin.

## Format

- OGG Vorbis, MONO, 22 050 Hz (la fréquence du synthé 8 bits qui les a rendus).
- Qualité Vorbis 4 pour les musiques (environ 50 kbit/s : 61 à 82 Ko par boucle de
  10 à 12 s), 6 pour les bruitages (courts : la qualité coûte peu).
- Pages OGG de 1 s pour les musiques, de 10 ms pour les bruitages : avec de plus
  grandes pages, ffmpeg ou `lewton` décodaient un son court avec quelques centaines
  d'échantillons de trop ou de moins.
- Tout est fait par `outils/sons.sh` (ffmpeg avec libvorbis, chemin donné par la
  variable `FFMPEG`, jamais écrit dans le dépôt).

## Règles

1. AUCUNE métadonnée : pas de commentaire Vorbis (ni titre, ni artiste, ni
   `encoder=Lavc...`), aucune marque `Lavf`/`Lavc`, aucun chemin. Seul reste le
   champ « fabricant » de l'en-tête Vorbis (`ffmpeg`), obligatoire dans le format.
   Vérifié par `outils` (`son::tests::chaque_son_est_un_ogg_mono_sans_metadonnees`).
2. Boucles SANS COUTURE (musiques et pluie) : le son décodé a exactement la
   longueur voulue, sans blanc ajouté, et le saut entre le dernier échantillon et le
   premier n'est pas plus fort que les sauts du son lui-même (aucun clic). Une
   musique part et finit presque dans le silence. Vérifié par
   `son::tests::chaque_boucle_se_raccorde_sans_couture`.
3. Taille : 100 000 octets au plus par musique, 32 000 par bruitage (même test que
   la règle 1). Total actuel : 517 055 octets.

## Remplacer une musique (ou un bruitage)

1. Préparer un WAV 16 bits mono à 22 050 Hz, `SOURCE/musiques/<id>.wav` (ou
   `SOURCE/sons/<clé>.wav`). Pour une boucle : la fin doit retomber sur le début
   (même phrase musicale, dernière note éteinte), et la durée être celle d'un tour
   exact.
2. L'encoder : `FFMPEG=chemin/de/ffmpeg outils/sons.sh SOURCE` (tous les WAV du
   dossier sont encodés, avec les bons réglages et sans métadonnées).
3. Le contrôler, avec son WAV comme référence :
   `./docker-cargo.sh run -p outils --bin son -- controler assets/musiques/<id>.ogg SOURCE/musiques/<id>.wav`
   affiche taille, durée, raccord de la boucle, écart de longueur (0 attendu) et
   rapport signal/bruit ; il sort en erreur si le fichier n'est pas mono à
   22 050 Hz, porte un commentaire, ou n'a pas la longueur du WAV.
4. `./docker-cargo.sh test -p outils` (règles 1 à 3), puis écouter dans le jeu.

Une NOUVELLE musique demande aussi son entrée dans le catalogue
(`Musique::TOUTES`, `id`, `nom`, `prix`) ; un nouveau bruitage, son `Effet` (avec sa
`cle`) et l'endroit du jeu qui le joue.

## Non testé

Safari lit mal certains OGG Vorbis (selon la version de macOS ou d'iOS) : le jeu
n'y a pas été écouté.
