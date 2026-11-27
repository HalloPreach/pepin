//! Module `zones` : les métadonnées des niveaux de PÉPIN.
//!
//! Chaque niveau est une « zone » du monde : un thème de décor (le ciel
//! s'éclaircit d'un niveau à l'autre, de l'orage au soleil), un nom, un genre
//! (normal, bonus ou épilogue), un court texte d'introduction et une idée de
//! mécanique. Les niveaux eux-mêmes (ces infos et leur dessin ASCII) sont des
//! fichiers RON (`assets/levels/`), lus et rangés par le crate `niveaux` (carte
//! 10.1c) ; ce module n'en décrit que la FORME (`InfosNiveau`).
//!
//! Notions Rust illustrées ici (à reprendre dans le README) :
//! - `const` et `&'static str` : des données connues à la compilation, qui
//!   vivent pendant tout le programme (donc pas d'ownership à gérer) ;
//! - `enum` sans données pour le thème ; tableau de taille fixe `[T; N]` ;
//! - `pub use` : `GenreNiveau` est défini dans `levels` et ré-exporté ici ;
//! - `Result` : charger un niveau peut échouer, l'appelant décide quoi faire.

use crate::level::{ErreurNiveau, Niveau, ReglageAiguille};
pub use crate::levels::GenreNiveau;

/// Ambiance visuelle d'un niveau. Le rendu choisit ciel, parallaxe et pluie
/// selon cette valeur ; `luminosite` donne l'avancement de l'orage vers le soleil.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    /// Nuit d'orage sur les toits.
    Toits,
    /// Averse, ciel d'ardoise.
    Gouttieres,
    /// Marché couvert : nuit bleue, vitrines et lanternes orange.
    Marche,
    /// Orage au plus fort : éclairs, ciel violet.
    Orage,
    /// Le clocher : nuit violette au-dessus de la ville, que l'on voit s'enfoncer
    /// en montant (niveau vertical).
    Clocher,
    /// L'œil du cyclone : l'arène du Grand Nuage (le boss), violet et or ; une
    /// trouée d'or tout en haut, des murs et des tours de nuages violets.
    Cyclone,
    /// Le ciel se déchire, vent d'altitude.
    Altitude,
    /// La mer de nuages : ciel corail au couchant, une mer de nuages tout en bas,
    /// de petits nuages roses qui flottent ; pas de pluie (la grêle seule tombe).
    Mer,
    /// Au-dessus des nuages : le soleil doré.
    Soleil,
    /// L'arc-en-ciel (le niveau bonus) : toute la palette. Un ciel d'ardoise qui
    /// s'éclaircit vers le bas, un grand arc-en-ciel de huit bandes, et les nuages
    /// clairs du pont ; pas de pluie.
    ArcEnCiel,
    /// La descente (l'épilogue, niveau vertical à l'envers) : le ciel refait tout
    /// le voyage en sens inverse, de l'or du soleil (en haut) à la nuit des toits
    /// (en bas), selon la hauteur ; en bas, la ville du premier niveau. Pas de
    /// pluie : l'orage est fini.
    Descente,
    /// Le retour du Grand Nuage (le niveau final, carte 8.2) : le décor et le ciel
    /// d'altitude, clairs devant le front d'orage, sans pluie de thème (il ne pleut
    /// que DANS le front d'orage). Un thème à lui pour sa musique (`Poursuite`).
    Poursuite,
    // Monde 2, « Là où va la pluie » (carte 9.5) : sept thèmes, ajoutés À LA FIN de
    // l'enum : les indices du Monde 1 ne bougent pas (démo, course, robot,
    // sauvegarde par clé intacts). Le parapluie retourné devient une barque.
    /// Le caniveau : la rue sous l'averse au ras du trottoir, bouches d'égout, lampadaires.
    Caniveau,
    /// Les égouts : tunnels de briques sombres, grilles, tuyaux, lueur cyan.
    Egouts,
    /// La grande cascade (niveau vertical) : falaises, chutes d'eau et brume.
    Cascade,
    /// La grotte derrière la cascade (sans pluie) : nuit bleue, stalactites, sources
    /// d'où montent les bulles, cristaux cyan.
    Grotte,
    /// La rivière : berges et roseaux, ciel grisé qui s'éclaircit, reflets.
    Riviere,
    /// Le port : quais, coques de bateaux, grues, phare au loin et mouettes.
    Port,
    /// La marée (niveau vertical) : mur du port, eau sombre en bas, ciel d'aube.
    Maree,
}

impl Theme {
    /// Tous les thèmes, dans l'ordre de leur déclaration : le rendu y range un jeu
    /// de tuiles par thème (`Theme::indice`). Un nouveau thème s'ajoute ici ET dans
    /// l'`enum` (le test `tous_les_themes_dans_l_ordre` vérifie les deux).
    pub const TOUS: [Theme; 19] = [
        Theme::Toits,
        Theme::Gouttieres,
        Theme::Marche,
        Theme::Orage,
        Theme::Clocher,
        Theme::Cyclone,
        Theme::Altitude,
        Theme::Mer,
        Theme::Soleil,
        Theme::ArcEnCiel,
        Theme::Descente,
        Theme::Poursuite,
        // Monde 2 (carte 9.5) : après le Monde 1, jamais inséré au milieu.
        Theme::Caniveau,
        Theme::Egouts,
        Theme::Cascade,
        Theme::Grotte,
        Theme::Riviere,
        Theme::Port,
        Theme::Maree,
    ];

    /// Place du thème dans `Theme::TOUS` (son rang de déclaration).
    pub fn indice(self) -> usize {
        self as usize
    }

    /// Clarté du ciel de 0 (orage noir) à 4 (soleil) ; l'orage du niveau 4
    /// reste volontairement sombre, c'est le sommet de la tension.
    pub fn luminosite(self) -> u8 {
        match self {
            Theme::Toits => 0,
            Theme::Gouttieres => 1,
            Theme::Marche => 1,
            Theme::Orage => 0,
            Theme::Clocher => 1,
            Theme::Cyclone => 2,
            Theme::Altitude => 3,
            Theme::Mer => 3,
            Theme::Soleil => 4,
            // L'arc-en-ciel : le ciel d'ardoise après l'averse, éclairé par l'arc.
            Theme::ArcEnCiel => 2,
            // La descente va du soleil (4) à la nuit (0) : la moyenne.
            Theme::Descente => 2,
            // Le retour du Grand Nuage : le ciel d'altitude.
            Theme::Poursuite => 3,
            // Monde 2, « Là où va la pluie » (carte 9.5) : le monde de l'eau, après
            // le soleil du Monde 1. L'avancement repart de la nuit humide (caniveau,
            // égouts) vers l'aube du port et de la marée.
            Theme::Caniveau => 1,
            Theme::Egouts => 0,
            Theme::Cascade => 1,
            Theme::Grotte => 1,
            Theme::Riviere => 2,
            Theme::Port => 3,
            Theme::Maree => 3,
        }
    }

    /// Pleut-il dans ce thème ? Ni au soleil, ni au-dessus de la mer de nuages
    /// (seule la grêle y tombe, en colonnes), ni sous l'arc-en-ciel (la pluie
    /// vient de s'arrêter), ni pendant la descente (l'orage est fini : on rentre), ni
    /// devant le front d'orage du Grand Nuage (il ne pleut que dedans), ni dans la
    /// grotte du Monde 2 (carte 9.5 : la pluie est restée dehors), ni dans les
    /// égouts (carte 9.27 : sous terre ; seuls les filets d'eau des grilles y
    /// tombent, dessinés par leur habillage), ni sur la rivière (carte 9.10b :
    /// l'orage est passé, le soleil se couche). Rendu seulement : aucune règle de
    /// jeu n'en dépend.
    pub fn pluie(self) -> bool {
        !matches!(
            self,
            Theme::Soleil
                | Theme::Mer
                | Theme::ArcEnCiel
                | Theme::Descente
                | Theme::Poursuite
                | Theme::Egouts
                | Theme::Grotte
                | Theme::Riviere
        )
    }
}

/// Description d'un niveau : tout sauf le niveau chargé. `PartialEq` compare tous
/// les champs (le chargeur du crate `niveaux` s'en sert dans ses tests).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InfosNiveau {
    /// Monde du niveau (carte 9.1) : `1` pour le Monde 1 ; les mondes suivants
    /// (le Monde 2, « Là où va la pluie ») s'ajoutent à la FIN de la table, chacun
    /// en un seul bloc contigu, dans l'ordre des numéros (vérifié par
    /// `verifier_ordre`). La numérotation est PAR MONDE : le premier niveau d'un
    /// monde est le « NIVEAU 1 ». Les indices du Monde 1 ne bougent jamais : la
    /// démo, la course et la sauvegarde par clé le supposent.
    pub monde: u8,
    /// Place dans le MONDE, à partir de 1 (vérifié par `verifier_ordre`).
    pub numero: u8,
    /// CLÉ STABLE du niveau (carte 7.9) : un identifiant court (`toits`, `arc`…), le
    /// nom de son module. La sauvegarde range la progression par cette clé, jamais par
    /// la place dans la table : insérer un niveau ne fait glisser aucune médaille.
    /// Ne JAMAIS la changer une fois publiée (unique : vérifié par les tests).
    pub cle: &'static str,
    /// Défi « SANS PLANER » (carte 7.3, rangé par clé depuis la carte 8.13) : le
    /// niveau se finit-il sans JAMAIS ouvrir le parapluie ? Sinon, le défi devient
    /// « LES 3 GOUTTES ». Résultat du robot de faisabilité (parapluie interdit,
    /// courants et vents gardés), recopié à la main : le test ignoré
    /// `sans_planer_possible_d_apres_le_robot` (`niveaux/tests/tous_les_niveaux/`)
    /// le relance (à refaire si une grille change) :
    /// `./docker-cargo.sh test -p niveaux --release -- --ignored sans_planer_possible`.
    /// Niveaux d'essai : `false` (non mesurés).
    pub sans_planer: bool,
    /// Titre affiché.
    pub nom: &'static str,
    pub theme: Theme,
    /// Normal, bonus ou épilogue (voir `levels`).
    pub genre: GenreNiveau,
    /// Carte de présentation du niveau (carte 11.3) : la phrase d'ambiance, affichée
    /// en blanc sous le nom. Jamais vide (refusé au chargement) ; elle doit tenir
    /// sur l'écran d'intro (limites : `assets/levels/LISEZMOI.md`).
    pub intro_texte: &'static str,
    /// Carte de présentation : le conseil, affiché en corail sous la phrase. Jamais
    /// vide non plus.
    pub intro_astuce: &'static str,
    /// Niveau à boss (carte 9.34) : ce que le mode facile change POUR CE BOSS, la
    /// ligne de la bannière « TROP DUR ? » (par exemple « MODE FACILE : LE NUAGE N'A
    /// QUE 2 VIES »). Obligatoire pour un boss, vide pour les autres niveaux (vérifié
    /// au chargement, limites : `assets/levels/LISEZMOI.md`).
    pub facile_texte: &'static str,
    /// Mécanique introduite (note de conception : ce que le niveau apprend ; n'est
    /// pas affichée en jeu).
    pub mecanique: &'static str,
    /// Durée visée en secondes (de jeu normal, sans mort).
    pub duree_cible_s: u16,
    /// Dessin ASCII du niveau (voir la légende dans `level`).
    pub ascii: &'static str,
    /// Réglage des aiguilles d'horloge du dessin : une ligne par pivot `h`, dans
    /// l'ordre de lecture (voir `level::ReglageAiguille`). Vide sans aiguille.
    pub aiguilles: &'static [ReglageAiguille],
    /// Poissons qui sautent plus (ou moins) haut que `HAUT_SAUT_POISSON` (carte
    /// 9.11e) : `(colonne, rangée, hauteur_px)` de leur lettre `i`, comptées
    /// depuis 0 (voir `Niveau::regler_sauts_poissons`). Vide le plus souvent.
    pub poissons_hauts: &'static [(usize, usize, f32)],
}

impl InfosNiveau {
    /// Lit le dessin ASCII (et le réglage de ses aiguilles). Ne peut échouer que
    /// si le niveau est invalide, ce que le chargeur du crate `niveaux` refuse déjà.
    pub fn charger(&self) -> Result<Niveau, ErreurNiveau> {
        let mut niveau = Niveau::depuis_ascii_avec_aiguilles(self.ascii, self.aiguilles)?;
        niveau.regler_sauts_poissons(self.poissons_hauts)?;
        Ok(niveau)
    }
}
