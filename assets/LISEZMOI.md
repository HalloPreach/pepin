# Les assets de PÉPIN : ce qui est un fichier, ce qui reste du code

Depuis la chaîne de cartes 10.4 (10.4a à 10.4f), tout ce qui est DESSINÉ UNE FOIS POUR
TOUTES ou ENREGISTRÉ est un fichier de ce dossier, seule source de son contenu : on le
modifie en modifiant le fichier, jamais le code. Ce qui reste du code, c'est ce qui se
CALCULE (animé, semé au hasard, qui suit la grille du niveau, le temps ou la caméra
pixel par pixel) : en faire un fichier voudrait dire l'enregistrer image par image.

## Les dossiers

| Dossier | Contenu | Comment le jeu l'a | Règles |
| --- | --- | --- | --- |
| `sprites/` | Les sprites : Pépin, ennemis, objets, tuiles (PNG indexé). | Embarqués dans `game.wasm` (`game/build.rs`, `include_bytes!`). | `sprites/LISEZMOI.md` |
| `fonds/` | Les couches FIXES des fonds : ciels, soleil, arc-en-ciel, égouts, port, rue, vallée, cristaux (PNG indexé). | Embarqués dans `game.wasm`. | `fonds/LISEZMOI.md` (avec la pile de chaque thème) |
| `levels/` | Les niveaux (RON), un fichier par niveau. | Embarqués dans `game.wasm` (`niveaux/build.rs`, `include_str!`). | `levels/LISEZMOI.md` |
| `musiques/` | Une boucle par musique du catalogue (WAV). | Servis À CÔTÉ du jeu (`dist/musiques/`), téléchargés à la demande. | `musiques/LISEZMOI.md` |
| `sons/` | Un bruitage par effet, plus la boucle de pluie (WAV). | Servis à côté du jeu (`dist/sons/`), téléchargés à la demande. | `musiques/LISEZMOI.md` |

Ajouter un sprite, une couche, un niveau ou un son = ajouter un fichier : les listes
sont écrites par les `build.rs`, jamais à la main.

Les images et les niveaux sont petits (moins de 300 Ko à eux tous) : ils sont dans le
wasm, prêts dès la première image. Les sons pèsent 3 Mo : ils n'y sont pas. Le jeu ne
les télécharge QUE si le joueur active le son (coupé par défaut) : à la première
activation, le bip des menus, la musique de la zone en cours, la pluie et les
bruitages ; une autre musique seulement quand sa zone arrive (`audio::FileDeSons`,
carte 10.4f). Un joueur qui ne touche pas au son ne télécharge aucun son.

## Ce qui reste CALCULÉ par le code, et pourquoi

- La PLUIE (`decor::dessiner_pluie`, `decor::goutte_de_pluie`) : des milliers de
  gouttes en diagonale, leur place calculée à chaque pas d'après le numéro du pas, une
  densité par niveau (bruine, averse, forte pluie sous le Grand Nuage). Une image fixe
  ne tomberait pas ; une animation enregistrée pèserait plus que le code et ne se
  répéterait jamais aussi longtemps sans se voir.
- Les ÉCLAIRS (`scene/nuage.rs`, le Grand Nuage ; `mur_pluie.rs`, le front d'orage) :
  annoncés par un pointillé qui clignote puis frappés à une colonne précise, ils SUIVENT
  la règle du jeu (une colonne d'éclair tue) et la hauteur de la caméra. Ils se
  dessinent là où la simulation les place, au pas près.
- L'EAU du Monde 2 (`eau.rs`, `mer_montee.rs`, `dessin_eau.rs`) : la marée qui monte
  et sa graduation, les chutes et leurs filets, les bulles, les remous du poisson, la
  mer de la montée. Les morceaux sont des sprites (`sprites/objets_eau`,
  `sprites/tuiles_monde2`), mais leur hauteur, leur place et leur mouvement viennent de
  la simulation à chaque pas.
- Les PARTICULES et effets (`particules.rs`, `effets.rs`) : éclaboussures, poussière,
  étincelles, vent visible ; elles naissent d'un événement (atterrissage, saut, goutte
  ramassée) et vivent quelques images, d'après un petit générateur déterministe.
- Les TUILES ÉCLAIRCIES (`decor::TeinteTuiles`, `sprites::remplacer`) : le dessin d'une
  tuile est un fichier, mais d'un niveau à l'autre les toits pâlissent (l'ardoise gagne
  des crans vers le blanc cassé), les briques virent à l'orange au soleil ou au cyan
  mouillé au Monde 2. Une seule tuile, recolorée par le code, au lieu d'une copie par
  niveau ; c'est l'histoire du jeu (l'orage qui s'éclaircit) qui choisit la teinte.
- Dans les FONDS : les plans semés au hasard (silhouettes, montagnes, collines, rangs
  d'arbres, plafonds de la grotte), ce qui s'anime (feux, bouées, halos, oiseaux,
  nuages, roue du moulin, lucioles, la grande chute) et ce qui se pose d'après la
  grille du niveau (habillage de la grotte, rideau de la cascade). La liste exacte, thème
  par thème et couche par couche, est dans `fonds/LISEZMOI.md`.
- Le TEXTE (`texte.rs`) : la police 5 x 7 est une petite table de bits du crate `core`
  (`pepin_core::font`), testée avec lui ; les phrases, les chiffres du chrono et des
  scores sont composés par le code.

## Vérifié par les tests

- Aucun fichier manquant ni orphelin, dossier par dossier : sprites
  (`sprites::fichiers::tests::aucun_sprite_demande_par_le_code_ne_manque` et
  `aucun_fichier_orphelin`), fonds
  (`decor::tests::aucune_couche_png_manquante_ni_orpheline`), musiques et sons
  (`audio::tests::chaque_son_demande_a_son_fichier`), niveaux (crate `niveaux` : un
  fichier = un niveau, numérotés sans trou, `les_niveaux_sont_numerotes_dans_l_ordre`).
- Aucun fichier que le jeu ne lirait pas (un `.ogg` oublié, un `.jpg`, un dossier
  inconnu) : `assets::tests::chaque_fichier_d_assets_est_lu_par_le_jeu` ; et chaque
  dossier est décrit dans ce LISEZMOI (`chaque_dossier_d_assets_est_decrit`).
- Rien de personnel dans les fichiers : PNG sans bloc de texte
  (`chaque_png_n_a_que_les_blocs_permis`, sprites et fonds), WAV sans métadonnée
  (`chaque_son_est_un_wav_mono_16_bits_sans_metadonnees`) ; `./build.sh` refuse tout
  nom interdit.
