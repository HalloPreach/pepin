# Les niveaux de PÉPIN en fichiers RON

Un fichier par niveau, lu par le crate `niveaux` (chargement et validation),
puis converti en `InfosNiveau` du crate `core`.

- `monde_1/<cle>.ron`, `monde_2/<cle>.ron`... : les niveaux du parcours ;
- `essais/<cle>.ron` : les niveaux d'essai (hors parcours, `numero: 0`).

AJOUTER UN NIVEAU = AJOUTER UN FICHIER : sa place vient de `monde` et `numero`
(numéros 1, 2, 3... sans trou dans chaque monde), aucune liste à modifier. Le
fichier est embarqué dans le jeu à la compilation (rien à télécharger).

Depuis la carte 10.1b, le JEU lit ses niveaux dans ces fichiers ; depuis la carte
10.1c, ce sont les SEULS niveaux (les anciens `core/src/levels/*.rs` sont
supprimés, l'historique git les garde). Un fichier invalide arrête `./build.sh` (le jeu publié
n'affiche donc jamais d'erreur de niveau) ; le build de QA (`./build.sh --qa`)
affiche l'erreur à l'écran. Pour essayer un niveau sans recompiler (build de QA) :
poser son texte dans le `localStorage` du navigateur, clé `pepin.niveau_essai`,
puis ouvrir la page avec `?essai=1`.

Éditeur de niveaux (carte 10.3a, site de test seulement) : `editeur.html` à côté
du jeu de QA (source : `editeur/`). Il ouvre un `.ron` du jeu ou un fichier local,
dessine la grille avec les lettres de `legende.json` (écrit par le crate `niveaux`,
module `legende`) et exporte au format canonique (`editeur/ron.js`, le même que
`niveaux::ecrire`).

Essai depuis l'éditeur (carte 10.3b) : TESTER pose le niveau dans le
`localStorage` et ouvre le jeu de QA (`?essai=1`) en plein écran dans l'éditeur ;
Échap ou la fin du niveau ramène à l'éditeur. Clic droit sur une case vide :
« partir d'ici » (Pépin posé là, la grille de l'éditeur ne change pas ; Alt + clic
garde la pipette). Valider fait juger le niveau par le chargeur Rust sans jouer.
Un niveau refusé : le message du chargeur s'affiche et la case fautive est cerclée
de rouge (le jeu la renvoie par `postMessage`, `game/src/essai_externe.rs`).

## Commentaires de conception

Le POURQUOI d'un niveau (recette, colonnes d'un passage, piège voulu, réglage
mesuré par le robot) s'écrit en commentaires `//` : en tête du fichier (avant la
parenthèse ouvrante) pour le niveau entier, ou juste au-dessus d'un champ (la
grille : au-dessus de `ascii:`). Ce sont ceux des anciens fichiers Rust, reportés
une fois par `niveaux/outils/reporter_commentaires.py` (carte 10.1c). Le
chargeur les ignore ; l'éditeur les garde à leur place à l'export.

## Les tests des niveaux

`niveaux/tests/tous_les_niveaux/` : un fichier par sujet (table, faisabilité,
grilles, décor, pigeons, essais...) et un module par niveau qui a des tests à lui
(`par_niveau/<clé>.rs`). `./docker-cargo.sh carte <clé>` lance les tests rapides,
la faisabilité et la reprise de CE niveau, et son module `par_niveau::<clé>::`.

## Les champs

| champ | sens |
|---|---|
| `monde` | 1, 2... ; égal au dossier `monde_<N>` |
| `numero` | place dans le monde (1 pour le premier) ; 0 pour un essai |
| `cle` | identifiant stable (minuscules, chiffres, `_`) = nom du fichier ; ne jamais le changer |
| `nom` | titre affiché |
| `theme` | `Toits`, `Gouttieres`, `Marche`, `Orage`, `Clocher`, `Cyclone`, `Altitude`, `Mer`, `Soleil`, `ArcEnCiel`, `Descente`, `Poursuite`, `Caniveau`, `Egouts`, `Cascade`, `Grotte`, `Riviere`, `Port`, `Maree` |
| `genre` | `Normal`, `Bonus` (au plus un par monde, après les normaux) ou `Epilogue` (le dernier du monde) |
| `sans_planer` | le défi « SANS PLANER » est-il possible (mesuré par le robot) |
| `duree_cible_s` | durée visée en secondes |
| `intro_texte` | la phrase de la carte du niveau (en blanc sous le nom) ; obligatoire, 3 lignes au plus |
| `intro_astuce` | le conseil de la carte du niveau (en corail dessous) ; obligatoire, 2 lignes au plus |
| `mecanique` | note de conception : ce que le niveau apprend (pas affichée en jeu) |
| `aiguilles` | facultatif : `(rayon: 3.0, vitesse: 30.0, phase: 0.0)` par pivot `h`, dans l'ordre de lecture |
| `poissons_hauts` | facultatif : `(colonne, rangée, hauteur_px)` d'un poisson `i`, comptées depuis 0 |
| `ascii` | la grille, en chaîne brute `r#"..."#` (une lettre = une tuile de 16 px) |

Tout champ inconnu est une erreur. La grille doit contenir un `@`, une sortie
`X` (sauf l'arène d'un boss) et exactement 3 gouttes `o`.

L'intro (carte 11.3) est écrite dans le fichier, et nulle part ailleurs : un niveau
s'ajoute avec son seul fichier. La carte du niveau coupe chaque texte entre les mots
en lignes de 46 caractères ; `intro_texte` tient sur 3 lignes, `intro_astuce` sur 2,
et les deux ensemble sur 4 (3 pour un bonus, qui ajoute la ligne « sans perte de
vie »). Un champ vide ou trop long est refusé au chargement, avec le nom du champ et
son nombre de lignes ; l'éditeur montre les lignes et les mêmes refus pendant la
saisie (`niveaux/src/intro.rs`, `editeur/ron.js`).

## Exemple minimal

Un niveau d'essai, `essais/mon_essai.ron` (le test `l_exemple_du_lisezmoi_est_valide`
le charge) :

```ron
(
    monde: 1,
    numero: 0,
    cle: "mon_essai",
    nom: "Mon niveau",
    theme: Toits,
    genre: Normal,
    sans_planer: false,
    duree_cible_s: 30,
    intro_texte: "Une phrase.",
    intro_astuce: "Un conseil.",
    mecanique: "Ce que le niveau apprend.",
    ascii: r#"
....................
....................
...........o........
..@.....o......o..X.
####################
"#,
)
```

## Valider un fichier

```text
./docker-cargo.sh run -p niveaux --bin valider -- assets/levels/monde_1/toits.ron
./docker-cargo.sh run -p niveaux --bin valider      (tout le dossier, ordre compris)
```

L'erreur donne le fichier, la ligne et la colonne (de la grille et du fichier).

## Légende de la grille

<!-- legende : recopiee de core/src/level.rs (cargo run -p niveaux --bin lisezmoi) -->
```text
  .  vide                    #  toit (solide)      B  brique (solide)
  N  nuage (solide)          @  départ de Pépin     X  arrivée (drapeau)
  L  lanterne (checkpoint)   o  goutte d'or (3 par niveau)
  ^  pointes                 M  champignon-trampoline
  A  auvent : toile de boutique inclinée (haute à gauche), traversable ;
     tombé dessus par le haut, Pépin est relancé en diagonale, vers le haut et
     vers la droite (4 tuiles plus loin sans rien toucher, 3 à 5 en dosant)
  f  flaque (patrouille)     c  corbeau (vol)
  s  escargot : « un champignon qui marche » ; patrouille comme la flaque
     (demi-tour au mur et au bord du vide), lentement (16 px/s) ; impossible
     à écraser : lui tomber dessus fait rebondir comme le champignon ; le
     toucher de côté est sans danger
  p  pigeon, perché sur le solide du dessous ; il regarde du côté où son
     perchoir s'arrête (à droite seulement si le perchoir continue à gauche
     et s'arrête à droite ; sinon à gauche, vers le départ)
  z  chat endormi, couché sur le solide du dessous, la case du dessus libre ;
     il dort 1,7 s, se réveille (annonce de 0,8 s), reste éveillé 1,2 s, se
     rendort ; si Pépin est à portée à la fin de l'annonce (5 tuiles en x,
     2 en hauteur, sans mur entre eux), il lui bondit dessus pendant son
     éveil (sans quitter son toit), puis revient à sa place ; endormi on
     l'écrase en lui sautant dessus, éveillé le toucher tue (même par le
     dessus, même en plein bond)
  F  plateforme friable      e  éclair (colonne du 1er solide dessus, ou du
                                haut de la carte, au 1er solide dessous)
  g  grêle : colonne de grêlons qui tombe EN CONTINU, de la case `g` au 1er
     solide dessous (ou au bas de la carte) ; elle tue parapluie fermé, on la
     traverse en planant dessous
  v  courant ascendant       1-7, 9 extrémités d'une plateforme mobile (2 par chiffre)
  w  ventilateur : hélice posée sur le solide du dessous ; son courant
     ascendant (du dessus du socle au 1er solide au-dessus, ou au haut de
     la carte) souffle 2 s puis s'arrête 2 s ; la reprise est annoncée
     comme l'éclair (clignotement qui accélère) ; le socle est solide, on
     se tient dessus
  b  montgolfière : nacelle (moitié haute de la case, solide : on se tient
     dessus), ballon dans les deux cases au-dessus (libres, traversable) ;
     Pépin à bord, elle monte jusqu'au 1er solide au-dessus du ballon (ou
     8 tuiles au plus) ; sans lui, elle redescend doucement à sa case
  <  >  vent latéral (zone rectangulaire) : pousse vers la gauche / vers la
          droite ; le plané (parapluie ouvert) est poussé deux fois plus fort
  O  le Grand Nuage, boss (un au plus) : `O` est le milieu de la rangée du haut
     de son corps de 3 x 2 cases (libres, comme les 3 du dessus) ; il lance des
     motifs d'éclairs sur 17 colonnes (8 de chaque côté de `O`, du dessous du
     nuage au 1er solide), puis s'essouffle et libère un courant ascendant à
     une case de lui (à gauche ou à droite) : on y monte et on lui saute sur la
     tête ; trois coups et il est vaincu. Son arène n'a pas de drapeau `X` : le
     vaincre finit le niveau
  h  pivot d'une aiguille d'horloge : une plateforme 32 x 8 dont le centre
     tourne autour du centre de la case, dans le sens horaire, le dessus
     toujours horizontal (on s'y tient, elle emporte Pépin) ; rayon, vitesse
     et heure de départ dans la table `aiguilles` du fichier de niveau
     (`ReglageAiguille`, une ligne par `h` dans l'ordre de lecture) ; son
     cercle ne doit toucher aucun solide, ni Pépin debout dessus
  D S G C W  panneaux : déplacer, sauter, planer, champignon, vent
  Z  panneau « barque » (carte 9.6c) : la flèche du BAS vers le parapluie retourné
  H T R Q K  décor sans collision, posé sur le solide du dessous : cheminée,
             antenne, paratonnerre, manche à air, oiseau posé
  E I U J    décor de rue du Monde 2 (carte 9.6b), sans collision lui aussi :
             bouche d'égout, borne, poubelle, descente de gouttière
  m  faux mur (carte 8.5) : case LIBRE dessinée comme le solide qu'elle
     prolonge (voir `ZoneCachee`) ; Pépin y entre et la zone se révèle
  x  sortie secrète : un second drapeau, qui finit le niveau comme `X`
     (une au plus, en plus du drapeau `X`)
  *  lettre cachée (carte 8.6) : une des lettres de « PÉPIN », une au plus,
     laquelle : `crate::lettres` (8 x 8 au milieu de la case, comme `o`)
  P  mur de pluie (un au plus, carte 8.2) : son front part du bord droit de
     la case `P` et avance vers la droite ; il tue au contact (voir
     `crate::poursuite`). Hors de ce repère, la case est vide
  Ennemis d'eau du Monde 2 (carte 9.4, `entities::ennemi`, sous-module `eau`) :
  j  grenouille, posée sur ce qui est dessous (berge, nénuphar ou eau) ;
     Pépin à portée, elle se ramasse 0,5 s puis bondit en arc vers lui ;
     écrasable
  i  poisson : la case d'AIR juste au-dessus de l'eau (l'eau est dessous,
     obligatoire) ; remous 0,7 s, puis il saute à la verticale (3,5 tuiles) ;
     mortel hors de l'eau, jamais écrasable
  k  crabe : patrouille comme la flaque ; lui sauter dessus fait REBONDIR
     Pépin sans le blesser (carapace dure) ; de côté, il tue
  r  rat : patrouille ; Pépin sur sa ligne (pieds à la même hauteur, moins de
     6 tuiles, sans mur entre eux), il se dresse 0,5 s puis fonce jusqu'au
     mur ; écrasable
  u  mouette, perchée sur le solide du dessous (mât, grue) : un pigeon du
     port, même annonce, même piqué
  ~  {  }  eau calme, courant d'eau vers la gauche, vers la droite (carte
           9.2) : planer dedans donne la barque, y entrer autrement noie
  Y  bulles (carte 9.3) : colonne qui soulève comme un courant ascendant,
     prolongée vers le bas à travers l'eau qui est juste dessous
  n  nénuphar (carte 9.3), dans la case au-dessus de l'eau : une feuille sur
     laquelle on se pose ; elle s'enfonce sous Pépin et le met à l'eau en 1 s
  =  marée (carte 9.3) : zone où l'eau monte et descend par cycles
     (`entities::maree`) ; elle repose sur un solide ou sur de l'eau
  V  stalactite (carte 9.3), sous un solide : elle tremble 0,75 s quand
     Pépin passe dessous, puis tombe et se brise au sol ; mortelle au
     toucher, même accrochée (carte 9.26a)
  |  chute d'eau (carte 9.3) : colonne qui pousse vers le BAS (lente
     parapluie ouvert, rapide parapluie fermé)
  8  la Pieuvre du port, boss du Monde 2 (carte 9.12a, un boss au plus) : le
     `8` est dans l'eau de son bassin (case d'eau calme), qui couvre 8 cases à
     sa gauche et 9 à sa droite ; 9 colonnes de 2 cases (la colonne 0 : le `8`
     et sa voisine) où ses tentacules, annoncés 0,8 s, se dressent jusqu'en
     haut de la carte (mortels 0,5 s) ; entre deux motifs, sa tête (3 x 2 cases)
     sort sur l'eau au milieu (colonne 0), à gauche (-3) ou à droite (3) : on lui
     saute dessus, trois coups. Pas de drapeau `X` : la vaincre finit le niveau
  _  la bâche de l'arène de la Pieuvre (carte 9.13a, une rangée) : solide en
     phase 1, déchirée au 3e coup ; la tour de la phase 2 est au-dessus
  &  la vigie de la Pieuvre en haut de la tour (carte 9.13a) : coin bas-gauche
     de sa tête de géante (4 x 3 cases, carte 9.13c), où elle se jette pour
     le coup final ; un saut dessus finit le niveau. Va avec `_`
  La montée de la Pieuvre (cartes 9.13b et 9.13c, avec `&`) :
  $  un JETÉ : coin bas-gauche de la tête de la géante (4 x 3 cases libres,
     posée sur la corniche) quand elle se jette au bout d'une corniche large
  !  un BALAYAGE : au-dessus du bout droit d'une corniche, contre le couloir
     de la géante ; son tentacule balaie la corniche vers la gauche
  +  une SECOUSSE : quand les pieds de Pépin passent au-dessus de cette
     rangée, elle secoue la tour et des caisses tombent (case libre)
  %  une plateforme ARRACHABLE (solide, comme un toit) : elle la tire dans
     la mer quand Pépin arrive dessous
  Mobilier de rue du Monde 2 (carte 9.20f) : des cases SOLIDES (des toits
  pour la collision) que le jeu dessine comme un seul objet par groupe de
  cases voisines de la même lettre (`Mobilier`) :
  q  voiture garée (4 x 2 cases : `.qq.` sur `qqqq`, le capot et le coffre
     plus bas que le toit)
  a  autobus garé (un rectangle ; ses roues sont dessinées sous lui)
  t  toit d'abribus (une rangée ; ses montants et sa vitre, dessinés jusqu'au
     sol, ne bloquent rien : on passe dessous)
  d  poubelle de rue (1 case)
  l  banc (une rangée)
  Monde 3, prototype (carte 11.1) :
  y  crochet : en l'air, SAUT tenu à 2,5 cases au plus de la crosse, et
     plus haut qu'elle, le parapluie s'y accroche au lieu de planer ; Pépin
     se balance (GAUCHE et DROITE donnent de l'élan), SAUT lâché le lance
     (voir `crate::player::Crochet`). Case libre, sans collision
```
<!-- fin de la legende -->
