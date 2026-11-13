# PÉPIN — un jeu de plateformes en Rust, pour apprendre Rust

PÉPIN (« pépin » : parapluie, en argot) est un petit jeu de plateformes en
pixel art. Pépin est une créature ronde armée d'un grand parapluie corail. Elle
grimpe de toit en toit, de l'orage jusqu'au soleil, en cinq niveaux : le ciel
passe peu à peu du bleu nuit à l'or.

Le projet a deux buts : un jeu qui tourne **dans un navigateur** (WebAssembly),
et un code **lisible par quelqu'un qui apprend Rust**. Presque chaque fichier
commence par un commentaire (`//!`) qui dit ce qu'il fait et quelles notions de
Rust il illustre. Ce README fait le tour du propriétaire ; la section
« Les notions de Rust, avec le code réel » renvoie vers les fichiers exacts.

Tout est dans le code : **aucun fichier image, aucun fichier son, aucune police
externe**. Les sprites sont des grilles de texte, les sons sont calculés au
démarrage, la police est un tableau de bits.

## Sommaire

1. Jouer
2. Architecture : `core` (pur) et `game` (graphique)
3. La boucle à pas fixe, 60 Hz
4. Pourquoi macroquad
5. Contraintes de la machine de développement : pas de `cc`, Docker
6. Construire, tester, servir
7. Les notions de Rust, avec le code réel
8. Palette, police 5x7, pixel art procédural
9. Son et tactile
10. Poids du wasm
11. Pistes d'amélioration

## 1. Jouer

Commandes (voir l'en-tête de `game/src/main.rs`) :

| Action               | Touches                                              |
|----------------------|------------------------------------------------------|
| Courir               | flèches gauche/droite, ou Q / A et D (AZERTY et QWERTY) |
| Sauter               | Espace, flèche haut, W ou Z (AZERTY et QWERTY)       |
| Planer               | maintenir la touche de saut en l'air (parapluie ouvert) |
| Pause                | Échap ou P ; s'ouvre aussi seule si la fenêtre perd le focus en pleine partie (onglet masqué, autre application) |
| Son (coupé au départ)| M (sur AZERTY comme sur QWERTY), ou la ligne « SON » des menus |
| Menus                | flèches + Entrée, clic de souris ou toucher          |
| Aller à un niveau    | touches 1 à 9 (niveaux existants), niveau suivant : Page suivante ou N, précédent : Page précédente ou B, niveau d'essai vertical (quatre écrans de haut) : 0, niveaux d'essai du vent latéral, de la grêle, des auvents, du ventilateur, des montgolfières, des escargots, du chat endormi et des aiguilles d'horloge : F1, F2, F3, F4, F5, F6, F7 et F8 ; UNIQUEMENT dans le build de mise au point `./build.sh --qa` (dossier `dist-qa/`) ; absentes de la production |

Les touches sont lues par leur POSITION physique (le « code » du navigateur), pas
par la lettre imprimée : sur un clavier AZERTY, A et Q (QWERTY) sont à la place de Q
et A, Z et W sont échangés, et M est à la place du point-virgule. `game/src/entrees.rs`
accepte donc les deux positions pour chaque lettre et en garde toujours une
indépendante de la disposition (flèches, Espace, Entrée, Échap). Un effet de bord :
sur QWERTY, la touche point-virgule coupe aussi le son.

Sur un écran tactile, quatre boutons (gauche, droite, saut, pause) apparaissent
au premier toucher.

Les cinq niveaux (noms et mécaniques tirés de `core/src/levels/*.rs`) :

1. Les toits : course, saut, planer.
2. Gouttières et cheminées : flaques à écraser, champignons-trampolines.
3. Le grand orage : éclairs annoncés, plateformes mobiles et friables.
4. Vents d'altitude : courants ascendants, longs planés, nuage friable, corbeaux,
   pigeon.
5. Au-dessus des nuages : final calme, nuages friables, pigeon.

Chaque niveau contient trois gouttes d'or et des lanternes (points de
réapparition). Il n'y a pas de « game over » : on réapparaît à la dernière
lanterne allumée et on compte seulement les morts (`core/src/state.rs`).
Pendant 1 s après chaque réapparition, Pépin clignote et aucun danger ne peut
le tuer, sauf le vide (`PROTECTION_REAPPARITION_PAS`, `core/src/player.rs`).

## 2. Architecture : `core` (pur) et `game` (graphique)

Le dépôt est un *workspace* Cargo (`Cargo.toml` à la racine) avec deux crates :

    core/   logique pure : aucune dépendance, aucun appel graphique
    game/   le jeu : fenêtre, rendu, écrans, audio, entrées (macroquad)

Le découpage est volontaire. Tout ce qui est « règle du jeu » vit dans `core` ;
`game` ne fait que **lire** l'état et le **dessiner**. Conséquence très
pratique : `core` se teste sans fenêtre ni carte graphique, en quelques
secondes, avec un simple `cargo test`.

Détail à connaître : le paquet s'appelle `core`, mais sa bibliothèque s'appelle
`pepin_core` (voir `core/Cargo.toml`, section `[lib]`), pour ne pas entrer en
collision avec le crate `core` de la bibliothèque standard de Rust. Pour
cargo, on écrit donc `-p core` (nom du paquet) ; dans le code Rust, on écrit
`use pepin_core::...`.

Contenu de `core/src` :

- `aabb.rs` : boîtes de collision (`Aabb`), grille de tuiles, déplacement avec
  collisions axe par axe (`deplacer`).
- `physics.rs` : constantes (gravité, vitesse de saut...) et `Environnement`.
- `player.rs` : Pépin (`Joueur::actualiser` : course, saut variable, coyote
  time, tampon de saut, planer, courants ascendants, vent latéral, élan d'auvent).
- `entities/` : un fichier par famille d'entités (flaques et corbeaux dans
  `ennemi.rs`, pigeons dans `ennemi/pigeon.rs`, le chat endormi (il dort, se
  réveille après une annonce, se rendort) dans `ennemi/chat.rs`, `pointe.rs`, `eclair.rs`,
  plateformes `friable.rs` et `mobile.rs` (allers-retours, ou aiguilles d'horloge qui
  tournent autour d'un pivot, le dessus toujours horizontal), `champignon.rs`, l'auvent qui relance
  en diagonale dans `auvent.rs`, le ventilateur qui souffle un courant vers le
  haut (2 s de souffle, annonce qui clignote, 2 s d'arrêt) dans `ventilateur.rs`,
  la montgolfière dont la nacelle monte avec Pépin et redescend sans lui dans
  `montgolfiere.rs`, l'escargot, « un champignon qui marche », dans `escargot.rs`
  (sa marche au sol, la même que la flaque, dans `marche.rs`), gouttes et
  lanternes dans `collectibles.rs`) et `contacts.rs` pour ce qui se passe quand
  Pépin touche quelque chose.
- `level.rs`, `level/ascii.rs`, `level/erreur.rs` : le format de niveau ASCII
  et son analyseur.
- `levels.rs` et `levels/` : la table ORDONNÉE des niveaux (`NIVEAUX`, un fichier
  par niveau) et les règles qui les enchaînent : genre de chaque niveau (normal,
  bonus, épilogue), niveau suivant (`apres`), bonus débloqué par toutes les gouttes
  des niveaux normaux. Fonctions pures, testées sur de fausses listes.
- `zones.rs` : les métadonnées d'un niveau (nom, genre, texte d'introduction, thème
  de décor).
- `state.rs` : score, checkpoint et phase de la partie.
- `timestep.rs` : l'accumulateur de la boucle à pas fixe.
- `faisabilite.rs` : un « robot joueur » qui prouve par simulation qu'un niveau
  peut être terminé.
- `palette.rs`, `font.rs` (+ `font/`) : palette de 16 couleurs, police 5x7.
- `synthese.rs`, `sons.rs` : synthèse sonore 8 bits (calcul pur de tableaux de
  nombres).

Contenu de `game/src` (les fichiers de dessin, d'entrée et de son) :

- `main.rs` : la boucle principale macroquad.
- `app.rs` + `ecrans/` : la machine à écrans (titre, aide, carte de niveau,
  jeu, pause, fin).
- `monde.rs` : un niveau en cours de jeu ; il enchaîne les étapes d'un pas de
  simulation.
- `scene.rs`, `decor.rs`, `camera.rs`, `particules.rs`, `secousse.rs`,
  `transition.rs` : le rendu du monde.
- `sprites/` : le pixel art ; `texte.rs` : le dessin de texte ; `rendu.rs` : la
  cible 320x180 et l'agrandissement entier.
- `entrees.rs`, `tactile.rs`, `navigation.rs`, `menu.rs` : entrées et menus ;
  `focus.rs` : perte de focus de la fenêtre (pause automatique).
- `audio.rs` : joue les sons fabriqués par `core`.

Dans `game`, la même idée se répète à plus petite échelle : la logique (qui
mène où, quelle ligne de menu est choisie...) est écrite en fonctions pures
testées, et seule une mince couche appelle macroquad.

## 3. La boucle à pas fixe, 60 Hz

Le rendu suit la fréquence de l'écran (60 Hz, 144 Hz...), mais la simulation
avance **toujours** par pas de 1/60 s (`DT` dans `core/src/physics.rs`).
Le principe est celui de « Fix Your Timestep » : `core/src/timestep.rs`
contient un accumulateur, `PasFixe`. À chaque image, on lui donne le temps réel
écoulé (`avancer`) ; il répond combien de pas de simulation exécuter. Le reste
de temps (`alpha`) sert au rendu à interpoler entre le pas précédent et le pas
courant, pour un mouvement fluide à toute fréquence.

Dans `game/src/main.rs`, la boucle ressemble à ceci :

    for _ in 0..pas_fixe.avancer(get_frame_time() as f64) {
        if app.pas(&entrees) { ... }          // la simulation : 1/60 s
    }
    cible.commencer();
    app.dessiner(&banque, pas_fixe.alpha());   // le rendu, interpolé
    cible.afficher();

Deux garde-fous sont dans `timestep.rs` : au plus `MAX_PAS_PAR_IMAGE` (5) pas
par image, et au plus `DT_REEL_MAX` (0,25 s) de temps pris en compte. Si
l'onglet reste 10 secondes en arrière-plan, le jeu ne simule pas 10 secondes
d'un coup. Avantage d'un pas fixe : la physique est **déterministe**, donc les
tests donnent toujours le même résultat.

Un pas de simulation suit l'ordre documenté en tête de `core/src/entities.rs`
et appliqué par `Monde::avancer` (`game/src/monde.rs`) : les entités avancent,
les plateformes mobiles portent le joueur, le joueur bouge, on résout les
contacts, puis on met à jour l'état (score, mort, checkpoint).

## 4. Pourquoi macroquad

Pour sortir un jeu 2D dans un navigateur depuis Rust, il y a deux grandes
familles de solutions.

- **wasm-bindgen + web-sys + un canvas** : on écrit soi-même le pont entre Rust
  et les API du navigateur (canvas 2D ou WebGL, clavier, souris, boucle
  `requestAnimationFrame`, audio...), et on a besoin d'un outil de génération
  supplémentaire (`wasm-bindgen`, souvent `wasm-pack` ou `trunk`) pour fabriquer
  les fichiers JavaScript de liaison.
- **macroquad** : une petite bibliothèque de jeu qui fournit déjà fenêtre,
  entrées, dessin, textures, audio, et qui compile **le même code** pour le
  bureau et pour `wasm32-unknown-unknown`. Côté web, elle s'appuie sur un fichier
  JavaScript fourni (`mq_js_bundle.js`) et charge le `.wasm` avec un `load(...)`.
  Il n'y a pas d'étape `wasm-bindgen`.

Le choix est motivé par le but pédagogique : avec macroquad, le code Rust reste
le sujet du projet (boucle de jeu, structures de données), au lieu de se noyer
dans du code de liaison avec le navigateur. Le compromis est qu'on dépend d'un
moteur (version 0.4.16 d'après `Cargo.lock`) et qu'on reste sur des
fonctionnalités simples. `game/Cargo.toml` désactive les fonctionnalités par
défaut de macroquad (`default-features = false`) et ne garde que l'audio, derrière
une option `son`.

À noter : `game/build.rs` ajoute `--allow-undefined` à l'édition de liens
sur la cible wasm32. Les fonctions audio (`audio_init`...) sont fournies par
JavaScript au chargement, et Rust ne les connaît pas à la compilation.

## 5. Contraintes de la machine de développement : pas de `cc`, Docker

La machine de développement n'a ni `cc` ni `gcc` (vérifié : `which cc gcc` ne
renvoie rien) et on n'y a pas les droits d'installer un compilateur C. Or
`cargo build` et `cargo test` ont besoin d'un éditeur de liens. La solution est
d'utiliser Docker comme boîte à outils : un conteneur jetable `rust:1-slim`
compile et teste, avec le dossier du projet monté dedans.

Le script `docker-cargo.sh` encapsule tout cela :

- il lance le conteneur avec **votre identifiant d'utilisateur** (donc `target/`
  et `Cargo.lock` ne deviennent pas la propriété de root) ;
- il garde un dossier `CARGO_HOME` persistant (par défaut
  `~/.cache/pepin-cargo-home`, modifiable avec la variable
  `PEPIN_CARGO_HOME`) pour ne télécharger les dépendances qu'une fois ;
- il copie rustup dans ce dossier, y ajoute la cible `wasm32-unknown-unknown`
  puis `clippy` et `rustfmt` quand c'est nécessaire.

Dernier détail : sur Linux, le son réel (`quad-snd`) demande `libasound`, que
l'image Docker n'a pas. Les tests natifs de `game` se lancent donc avec
`--no-default-features` (sans le son réel) ; la version web garde le son.

## 6. Construire, tester, servir

Prérequis : Docker, et un navigateur récent pour jouer. Rien d'autre (ni
Rust ni compilateur C sur la machine hôte).

Tests :

    ./docker-cargo.sh test -p core
    ./docker-cargo.sh test -p game --no-default-features

Mesure à la dernière vérification de ce README : **143 tests réussis** dans
`core` (plus 1 test ignoré volontairement : il régénère l'image
`docs/police-test.png`) et **112 tests réussis** dans `game` (plus 1 test
ignoré : il écrit le script du robot de QA, voir `QA.md`).

Hygiène du code (format et avertissements) :

    ./docker-cargo.sh lint

Cette commande enchaîne `cargo fmt --all --check`, `cargo clippy` sur `core`
(avec les tests), `cargo clippy` sur `game` en cible wasm32 puis sur `game`
natif avec ses tests, tous avec `-D warnings`.

Compiler ET préparer le dossier à publier (une seule commande) :

    ./build.sh

`build.sh` compile en release, récupère le chargeur JavaScript de macroquad
(`mq_js_bundle.js`, sans son plugin réseau inutile), réduit le wasm avec
`wasm-opt -Oz`, copie la page `web/index.html` et produit le dossier `dist/`,
complet et statique. `docs/build.md` détaille chaque étape. `dist/` n'est pas
versionné (`.gitignore`) : c'est un produit régénérable.

Build de mise au point (touches 1 à 9 pour sauter à un niveau, Page suivante / N et
Page précédente / B pour le niveau suivant ou précédent, 0 pour le niveau d'essai
vertical de la caméra, F1, F2, F3, F4, F5, F6, F7 et F8 pour les niveaux d'essai du vent latéral, de
la grêle, des auvents, du ventilateur, des montgolfières, des escargots, du chat endormi et des
aiguilles d'horloge, F9 pour provoquer un vrai plantage et vérifier le message de la
page ; sert aux captures et à la QA, jamais à publier) :

    ./build.sh --qa        # produit dist-qa/ (ignoré par git)

Essayer le jeu dans un navigateur :

    cd dist && python3 -m http.server 8080

puis ouvrir http://127.0.0.1:8080/ . Il faut passer par un petit serveur : en général, un
navigateur ne charge pas un `.wasm` depuis un fichier ouvert directement
(`file://`).
Dans le jeu, appuyer sur une touche ou cliquer avant d'attendre du son (les
navigateurs bloquent l'audio avant la première interaction).
Si le jeu plante en cours de partie (erreur WebAssembly, affichage WebGL perdu), la
page coupe le son et affiche un message avec un bouton « Recharger » ; les erreurs des
scripts que certains navigateurs injectent dans la page (portefeuille de Brave...)
sont ignorées.

Pour compiler seulement le wasm, sans `dist/` :

    ./docker-cargo.sh build -p game --target wasm32-unknown-unknown --release

(résultat : `target/wasm32-unknown-unknown/release/game.wasm`).

## 7. Les notions de Rust, avec le code réel

Chaque exemple ci-dessous est tiré du dépôt : le chemin du fichier est donné,
vous pouvez l'ouvrir et lire le contexte.

### Ownership (la possession)

En Rust, chaque valeur a **un seul propriétaire** ; quand il disparaît, la
valeur est libérée. Pas de ramasse-miettes.

- `game/src/monde.rs` : la structure `Monde` **possède** le niveau, les entités,
  le joueur et l'état de la partie. Quand le `Monde` est détruit (changement de
  niveau), tout ce qu'il contient l'est aussi.
- `game/src/entrees.rs` : `Sources::ajouter(&mut self, source: Box<dyn SourceEntrees>)`.
  Le commentaire dit « la boîte prend la possession de la source » : après
  l'appel, l'appelant ne peut plus utiliser la source qu'il a donnée.
- `Monde::new` fait `let entites = niveau.entites.clone();` : le niveau garde
  les entités dans leur état initial, le monde en reçoit une **copie**
  (`clone`) qu'il va modifier.
- `Copy` : les petits types comme `Aabb` (`core/src/aabb.rs`) sont copiés
  implicitement. Le commentaire du fichier explique pourquoi : sans `Copy`,
  écrire `let b = a;` rendrait `a` inutilisable ensuite.
- `core/src/player.rs` : `Joueur::placer` fait `*self = Joueur::new(x, y)`, qui
  remplace tout le joueur d'un coup (vitesse et minuteries remises à zéro).

### Emprunts (borrow) et références

Au lieu de donner une valeur, on peut la **prêter** : `&T` pour lire (autant de
lecteurs qu'on veut), `&mut T` pour modifier (un seul à la fois, et plus aucun
lecteur pendant ce temps). Le compilateur le vérifie, donc pas de course aux
données ni de pointeur invalide.

- `core/src/player.rs` :
  `pub fn actualiser(&mut self, entrees: &Entrees, env: &Environnement<'_>) -> Evenements`.
  Le joueur est modifié (`&mut self`) ; les entrées et le décor sont seulement
  lus.
- `core/src/aabb.rs` : `deplacer(boite: &mut Aabb, dx, dy, grille: &dyn Grille, solides: &[Aabb])`.
  `&mut Aabb` : emprunt exclusif en écriture ; `&[Aabb]` : une *tranche*, une
  vue sur un tableau qu'on ne possède pas.
- **Durées de vie** (`'a`) : `core/src/physics.rs` définit
  `pub struct Environnement<'a>` dont les champs sont des références
  (`grille: &'a dyn Grille`, `solides: &'a [Aabb]`...). Le `'a` dit au
  compilateur : « cette structure ne peut pas vivre plus longtemps que les
  données qu'elle emprunte ».
- `game/src/monde.rs` (`Monde::avancer`) : on construit un `Environnement` qui
  emprunte `&self.niveau` pendant qu'on modifie un autre champ (`self.entites`).
  Rust accepte, car ce sont des champs différents.

### Enums et pattern matching

Un `enum` est un type qui vaut « l'une OU l'autre » de ses variantes, et chaque
variante peut porter des données. `match` oblige à traiter **tous** les cas :
si on ajoute une variante, le compilateur montre tous les endroits à compléter.

- `core/src/entities.rs` : `enum Entite { Ennemi(Ennemi), Pointe(Pointe), Eclair(Eclair), Friable(..), Mobile(..), Champignon(..), Goutte(..), Lanterne(..) }`.
  Toutes les entités d'un niveau tiennent dans un seul `Vec<Entite>`.
- `core/src/entities/ennemi.rs` : `enum EtatEnnemi { Actif, Ecrase(u8), Disparu }`.
  La variante `Ecrase(u8)` **porte** le nombre de pas d'animation restants.
- `core/src/entities/contacts.rs` : `resoudre_contacts` fait un `match entite`
  sur toutes les variantes de `Entite` pour décider ce que provoque chaque
  contact (ramasser une goutte, écraser une flaque, mourir sur une pointe...).
- `core/src/state.rs` : une machine à états. `EtatJeu::appliquer` fait un
  `match (self.phase, ordre)` sur un **couple**. Chaque transition permise est
  listée (par exemple `(Phase::Titre, Ordre::Demarrer) => Phase::EnJeu`) ; toute
  autre combinaison renvoie une erreur au lieu de corrompre l'état.
- `core/src/player.rs` : `Direction::signe` renvoie -1, 0 ou +1 par un `match`
  sans cas oublié possible.

### Traits

Un `trait` est un contrat : « tout type qui sait faire ceci ». Plusieurs types
le remplissent, et le code qui l'utilise n'a pas besoin de savoir lequel.

- `core/src/aabb.rs` : le trait `Grille` (« savoir dire si une tuile est
  solide »). Il a une **méthode par défaut**, `est_solide`, écrite une seule
  fois dans le trait. `Niveau` (`core/src/level.rs`) l'implémente
  (`impl Grille for Niveau`) : le moteur de collision s'utilise donc avec le vrai
  niveau, mais aussi avec une petite grille de test.
- `core/src/entities.rs` : le trait `Actualisable` (« avancer d'un pas ») ; 
  `Entite` l'implémente en déléguant à sa variante interne.
- `game/src/entrees.rs` : le trait `SourceEntrees` (clavier, tactile...).
  `Sources` range des `Box<dyn SourceEntrees>`, des **objets-traits** : une liste
  de types différents, appelés de la même façon. C'est ainsi que clavier et
  tactile coexistent sans que le jeu les distingue.
- `core/src/level/erreur.rs` : `impl fmt::Display for ErreurNiveau` (message
  lisible) et `impl std::error::Error for ErreurNiveau {}`.
- Traits « dérivés » : `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` fait
  écrire le code par le compilateur (voir `Phase` dans `core/src/state.rs`).

### Modules et visibilité

- `core/src/lib.rs` déclare les modules publics (`pub mod aabb;`...), et les
  constantes `LARGEUR_LOGIQUE` (320) et `HAUTEUR_LOGIQUE` (180).
- Un module peut être un fichier (`palette.rs`) ou un fichier plus un dossier
  du même nom : `entities.rs` + `entities/` (les sous-modules sont déclarés
  par `pub mod champignon;`...), de même `level.rs` + `level/`.
- Visibilité fine : `pub(crate)` (visible dans tout le crate) pour les champs
  de `Niveau` dans `core/src/level.rs`, et `pub(super)` (visible dans le module
  parent seulement) pour `analyser` dans `core/src/level/ascii.rs`. Les champs
  privés protègent les invariants : on ne peut modifier `PasFixe`
  (`core/src/timestep.rs`) qu'à travers ses méthodes.
- `use pepin_core::...` dans `game` : le crate `game` utilise le crate `core`
  comme une bibliothèque (déclaré dans `game/Cargo.toml`).

### Result et Option

Rust n'a pas de `null` ni d'exceptions. Une valeur qui « peut manquer » est un
`Option<T>` (`Some` ou `None`) ; une opération qui « peut échouer » renvoie un
`Result<T, E>` (`Ok` ou `Err`).

- `core/src/zones.rs` : `pub fn niveau(indice: usize) -> Option<&'static InfosNiveau>`
  renvoie `None` hors bornes (`NIVEAUX.get(indice)`).
- `core/src/entities.rs` : `Entite::solide(&self) -> Option<Aabb>` : « une boîte
  solide, ou rien », sans valeur magique. Puis
  `entites.iter().filter_map(Entite::solide).collect()` ne garde que les `Some`.
- `core/src/level.rs` : `Niveau::depuis_ascii(texte) -> Result<Niveau, ErreurNiveau>`.
  Les erreurs sont **typées** (`core/src/level/erreur.rs`) : `LargeurInegale { ligne,
  attendu, trouve }`, `DepartManquant`, `CaractereInconnu { .. }`... chacune avec
  son contexte.
- `game/src/monde.rs` : `Monde::new` charge un niveau embarqué avec
  `.expect("niveau embarqué valide")`. Ici un échec serait une erreur de
  programmation (les tests de `core/src/zones.rs` garantissent que tous les
  niveaux se chargent), d'où un `expect` plutôt qu'une gestion d'erreur.
- `core/src/state.rs` : `appliquer` renvoie
  `Result<Phase, TransitionInvalide>` ; l'état reste inchangé en cas d'erreur.
- `game/src/sprites/mod.rs` : la fonction de test `verifier` utilise
  `pixel(c).map_err(|c| format!(...))?` : on transforme l'erreur, puis l'opérateur
  `?` la propage à l'appelant.

### Itérateurs et fermetures

- `game/src/entrees.rs` : `Sources::lire` combine toutes les sources avec
  `.iter_mut().fold(Commandes::default(), |acc, s| acc.fusionner(s.lire()))`.
- `core/src/player.rs` : `env.courants.iter().find(|c| c.zone.chevauche(&self.boite))`
  cherche le premier courant ascendant qui touche le joueur.

### Programmation asynchrone

`game/src/audio.rs` : `pub async fn charger() -> Self`. Dans un navigateur, le
chargement des sons doit pouvoir attendre ; `main` (macroquad) est `async` et
appelle `Audio::charger().await`.

### Tests unitaires

Les tests vivent à côté du code, dans un module `#[cfg(test)] mod tests`
(compilé seulement pour `cargo test`). Chaque fonction marquée `#[test]` est un
test ; elle échoue si une assertion (`assert!`, `assert_eq!`) est fausse.

- Exemple simple : `core/src/aabb.rs`, test `bords_qui_se_touchent_ne_chevauchent_pas`.
- Les tests de `core/src/timestep.rs` vérifient que 60 images à 60 Hz donnent
  exactement 60 pas (`un_pas_par_image_a_60_hz`), et le comportement quand le
  rendu est lent.
- Vérifications **à la compilation** : `const _: () = assert!(COURANT_ACCEL > GRAVITE);`
  dans `core/src/physics.rs`. Si quelqu'un change les constantes de façon
  incohérente, le projet ne compile plus.
- Tests de données : `game/src/contenu.rs` parcourt tous les textes du jeu et
  vérifie qu'ils ne contiennent ni adresse web ni arobase.
- Un test plus ambitieux : `core/src/faisabilite.rs` simule un joueur avec la
  **vraie** physique et explore le niveau pour prouver qu'on peut l'achever
  (`niveau_1_faisable`... dans `core/src/zones.rs`). C'est pour cela que
  `Cargo.toml` compile les tests avec `opt-level = 2` (`[profile.test]`).

## 8. Palette, police 5x7, pixel art procédural

**Palette** (`core/src/palette.rs`). Le jeu n'utilise que **16 couleurs**,
nommées dans le code : ardoise (5 teintes), brume, nuage, blanc cassé, corail
(Pépin), corail clair, orange soleil, or, or pâle, cyan de pluie (2 teintes) et
violet d'orage. La liste est dans le tableau `TOUTES`, et un test
(`au_plus_seize_couleurs_distinctes`) interdit d'en ajouter plus de 16 ou de
créer un doublon. Pour assombrir un fond sans introduire de nouvelle couleur, les écrans le
recouvrent d'ardoise sombre une ligne sur deux (voir `game/src/ecrans/mod.rs`).

**Résolution** (`game/src/rendu.rs`). Tout le jeu est dessiné dans une image de
320x180 pixels (la « cible »). Cette image est ensuite agrandie d'un facteur
**entier** (x1, x2, x3...), centrée dans la fenêtre, avec le filtre
« nearest » : jamais de flou ni de demi-pixel. Les fonctions pures
`facteur_entier` et `placement` sont testées.

**Police 5x7** (`core/src/font.rs` et `core/src/font/`). Un glyphe fait 5
colonnes sur 7 rangées, avec 1 pixel d'espacement. Les glyphes sont des
dessins en texte (`"#...#"`, `".###."`), rangés par famille : majuscules,
minuscules, chiffres, ponctuation, majuscules accentuées et minuscules
accentuées (é è ê à ù ç œ ï ô...). Les minuscules ont leur hauteur d'x sur les
rangées 1 à 4, et les lettres à jambage (g j p q y) descendent de deux rangées.
`glyphe_ou_inconnu` renvoie un cadre vide pour un caractère absent, et
`game/src/texte.rs` dessine chaque pixel allumé comme un petit carré.

**Pixel art procédural** (`game/src/sprites/`). Un sprite est une grille de
texte : une ligne par rangée de pixels, un caractère par pixel. `.` est
transparent ; `0`-`9` et `A`-`F` désignent une des 16 couleurs de `TOUTES`
(`0` : contour sombre, `8` : corail, `B` : or...). Pépin fait 16x16 pixels,
parapluie fermé, et 20x20 parapluie ouvert. Au démarrage,
`Banque::charger` transforme chaque grille en texture (`Texture2D`) en filtre
« nearest ». Le décodage (`pixel`, `verifier`) est pur et testé : un test
rejette tout caractère interdit ou toute ligne de longueur inégale. Les tuiles
changent de teinte selon le thème du niveau (`remplacer` dans
`sprites/mod.rs`), ce qui donne l'éclaircie progressive du ciel. Le décor
(`game/src/decor.rs`) dessine ciel, étoiles, silhouettes en parallaxe, soleil et
pluie à l'aide de rectangles alignés sur la grille de pixels, avec des
décalages arrondis à l'entier. Les planches `docs/planche-sprites.png` et
`docs/police-test.png` montrent les sprites et la police ; `docs/capture-*.png`
sont des captures du jeu dans un navigateur, et `docs/mesure-capture.py`
compte les couleurs distinctes d'une capture.

## 9. Son et tactile

- **Son** (`core/src/synthese.rs`, `core/src/sons.rs`, `game/src/audio.rs`) :
  ondes carrée, triangle et bruit calculées échantillon par échantillon à
  22 050 Hz, comme sur une vieille console. On obtient onze effets, une
  musique en boucle et le bruit de la pluie, tous fabriqués en mémoire au
  démarrage. Le son est **coupé par défaut** (un navigateur n'autorise l'audio
  qu'après une première interaction) ; la touche M ou le menu l'active.
- **Tactile** (`game/src/tactile.rs`) : quatre boutons en pixel art, plusieurs
  doigts en même temps (courir et sauter). L'écran est jugé tactile au premier
  toucher reçu. Le tactile est une `SourceEntrees`, comme le clavier.

## 10. Poids du wasm

Le profil release du `Cargo.toml` racine vise la taille minimale :
`opt-level = "z"`, `lto = true`, `strip = true`, `codegen-units = 1`,
`panic = "abort"`, puis `build.sh` passe `wasm-opt -Oz`.

Mesure du dossier `dist/` produit par `./build.sh` (relevée sur les fichiers,
`gzip -9` pour la colonne de droite) :

    fichier              octets     gzip
    index.html             7 397    3 140
    mq_js_bundle.js       36 148    9 839
    game.wasm            422 556  172 990

Le wasm pèse donc environ 0,42 Mo, soit 173 Ko transférés par un serveur qui
compresse. Sans `wasm-opt`, la compilation seule donne un wasm de 473 956 octets.
Le budget de `build.sh` est : avertissement au-delà de 1,2 Mo, échec au-delà de
2,5 Mo. Ces chiffres dépendent du code et de la version du compilateur ; ils
sont à re-mesurer après chaque changement important.

## 11. Pistes d'amélioration

Ce sont des idées pour continuer à apprendre, pas des fonctionnalités
existantes :

- Sauvegarder le meilleur temps et les gouttes ramassées (stockage du
  navigateur).
- Ajouter un niveau : il suffit d'un nouveau fichier dans `core/src/levels/`,
  d'une entrée à sa place dans la table `NIVEAUX` de `core/src/levels.rs` (avec son
  genre), d'une ligne dans la macro des tests de faisabilité de `core/src/zones.rs`,
  de son texte dans `game/src/contenu.rs` et de sa bande de difficulté dans
  `game/src/difficulte.rs` (les tests et le compilateur signalent chaque oubli). Une
  ambiance inédite = un `Theme` de plus et une entrée dans `decor::ambiance`. Un niveau
  peut être plus haut que l'écran (plus de 11 rangées, par exemple 45 pour quatre
  écrans) : la caméra le suit alors en hauteur (`game/src/camera.rs`) ; le niveau
  d'essai `core/src/levels/essai_vertical.rs` (touche 0 du build de QA) en est l'exemple.
- Ajouter de nouveaux ennemis ou plateformes en suivant le modèle de
  `core/src/entities/` (une variante de plus dans `Entite`, et le compilateur
  indique tous les `match` à compléter).
- Ajouter une manette comme nouvelle `SourceEntrees`.
- Écrire un éditeur de niveaux qui lit et écrit le format ASCII.
- Tester le jeu dans plusieurs navigateurs et sur plusieurs tailles d'écran.

## 12. Licence

Ce projet est publié sous licence **MIT** (voir le fichier `LICENSE`) : vous pouvez
l'utiliser, le copier, le modifier et le redistribuer librement, en conservant la
mention de copyright.
