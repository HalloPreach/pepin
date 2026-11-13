//! Module `level` : format de niveau, grille de tuiles et entités instanciées.
//!
//! Un niveau s'écrit en ASCII dans une constante `&str` (voir `levels/`) et se
//! lit avec `Niveau::depuis_ascii`, qui renvoie un `Result` : soit le niveau,
//! soit une `ErreurNiveau` précise (ligne, colonne...). Aucun fichier externe.
//!
//! Légende (1 caractère = 1 tuile de 16 x 16 px ; la ligne 0 est en HAUT) :
//!
//! ```text
//!   .  vide                    #  toit (solide)      B  brique (solide)
//!   N  nuage (solide)          @  départ de Pépin     X  arrivée (drapeau)
//!   L  lanterne (checkpoint)   o  goutte d'or (3 par niveau)
//!   ^  pointes                 M  champignon-trampoline
//!   A  auvent : toile de boutique inclinée (haute à gauche), traversable ;
//!      tombé dessus par le haut, Pépin est relancé en diagonale, vers le haut et
//!      vers la droite (4 tuiles plus loin sans rien toucher, 3 à 5 en dosant)
//!   f  flaque (patrouille)     c  corbeau (vol)
//!   s  escargot : « un champignon qui marche » ; patrouille comme la flaque
//!      (demi-tour au mur et au bord du vide), lentement (16 px/s) ; impossible
//!      à écraser : lui tomber dessus fait rebondir comme le champignon ; le
//!      toucher de côté est sans danger
//!   p  pigeon, perché sur le solide du dessous ; il regarde du côté où son
//!      perchoir s'arrête (à droite seulement si le perchoir continue à gauche
//!      et s'arrête à droite ; sinon à gauche, vers le départ)
//!   z  chat endormi, couché sur le solide du dessous, la case du dessus libre ;
//!      immobile, il dort 4 s, se réveille (annonce de 0,8 s), reste éveillé
//!      1,2 s, se rendort ; endormi on l'écrase en lui sautant dessus, éveillé
//!      le toucher tue (même par le dessus)
//!   F  plateforme friable      e  éclair (colonne du 1er solide dessus, ou du
//!                                 haut de la carte, au 1er solide dessous)
//!   g  grêle : colonne de grêlons qui tombe EN CONTINU, de la case `g` au 1er
//!      solide dessous (ou au bas de la carte) ; elle tue parapluie fermé, on la
//!      traverse en planant dessous
//!   v  courant ascendant       1-9 extrémités d'une plateforme mobile (2 par chiffre)
//!   w  ventilateur : hélice posée sur le solide du dessous ; son courant
//!      ascendant (du dessus du socle au 1er solide au-dessus, ou au haut de
//!      la carte) souffle 2 s puis s'arrête 2 s ; la reprise est annoncée
//!      comme l'éclair (clignotement qui accélère) ; le socle est solide, on
//!      se tient dessus
//!   b  montgolfière : nacelle (moitié haute de la case, solide : on se tient
//!      dessus), ballon dans les deux cases au-dessus (libres, traversable) ;
//!      Pépin à bord, elle monte jusqu'au 1er solide au-dessus du ballon (ou
//!      8 tuiles au plus) ; sans lui, elle redescend doucement à sa case
//!   <  >  vent latéral (zone rectangulaire) : pousse vers la gauche / vers la
//!           droite ; le plané (parapluie ouvert) est poussé deux fois plus fort
//!   O  le Grand Nuage, boss (un au plus) : `O` est le milieu de la rangée du haut
//!      de son corps de 3 x 2 cases (libres, comme les 3 du dessus) ; il lance des
//!      motifs d'éclairs sur 17 colonnes (8 de chaque côté de `O`, du dessous du
//!      nuage au 1er solide), puis s'essouffle et libère un courant ascendant à
//!      une case de lui (à gauche ou à droite) : on y monte et on lui saute sur la
//!      tête ; trois coups et il est vaincu. Son arène n'a pas de drapeau `X` : le
//!      vaincre finit le niveau
//!   h  pivot d'une aiguille d'horloge : une plateforme 32 x 8 dont le centre
//!      tourne autour du centre de la case, dans le sens horaire, le dessus
//!      toujours horizontal (on s'y tient, elle emporte Pépin) ; rayon, vitesse
//!      et heure de départ dans la table `aiguilles` du fichier de niveau
//!      (`ReglageAiguille`, une ligne par `h` dans l'ordre de lecture) ; son
//!      cercle ne doit toucher aucun solide, ni Pépin debout dessus
//!   D S G C W  panneaux : déplacer, sauter, planer, champignon, vent
//!   H T R Q K  décor sans collision, posé sur le solide du dessous : cheminée,
//!              antenne, paratonnerre, manche à air, oiseau posé
//! ```
//!
//! Le décor (`ObjetDecor`) ne sert qu'au dessin : ni tuile, ni entité, aucune
//! physique. Remplacer ces lettres par `.` donne exactement le même niveau à jouer.
//!
//! Notions Rust illustrées ici (à reprendre dans le README) :
//! - `enum` (`Tuile`, `GenrePanneau`, `GenreDecor`) et `match` exhaustif ;
//! - `Result<T, E>` avec une erreur typée (`ErreurNiveau`) et l'opérateur `?` ;
//! - `trait` : `Niveau` implémente `Grille`, donc le moteur de collision le
//!   comprend sans rien savoir de son format ;
//! - modules découpés en petits fichiers (`level/erreur.rs`, `level/ascii.rs`).

mod ascii;
mod erreur;
#[cfg(test)]
mod tests;

pub use erreur::ErreurNiveau;

/// Vitesse (px/s) des plateformes mobiles posées par le parseur.
pub const VITESSE_PLATEFORMES: f32 = ascii::VITESSE_MOBILE;
/// Taille (px) de la plateforme d'une aiguille d'horloge (`h`), largeur puis hauteur.
pub const TAILLE_AIGUILLE: (f32, f32) = ascii::TAILLE_AIGUILLE;

use crate::aabb::{self, Aabb, Grille, TAILLE_TUILE};
use crate::entities::Entite;
use crate::physics::{CourantAscendant, VentLateral};

/// Réglage d'une aiguille d'horloge (pivot `h`). Le fichier de niveau en donne
/// une petite table (`InfosNiveau::aiguilles`) : la 1re ligne règle le 1er `h`
/// rencontré en lisant la grille (de haut en bas, puis de gauche à droite), la
/// 2e le 2e, etc. — comme un chiffre désigne une plateforme mobile.
///
/// ```text
/// const AIGUILLES: &[ReglageAiguille] = &[
///     ReglageAiguille { rayon: 3.0, vitesse: 30.0, phase: 0.0 }, // 1er `h`
///     ReglageAiguille { rayon: 2.0, vitesse: 20.0, phase: 6.0 }, // 2e `h`
/// ];
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReglageAiguille {
    /// Rayon en TUILES, du centre du pivot au centre de la plateforme (au moins 1).
    pub rayon: f32,
    /// Vitesse le long du cercle, en px/s (plus de 0, au plus 120 : 2 px par pas).
    pub vitesse: f32,
    /// Heure de départ sur le cadran, de 0 à 12 : 0 = midi (au-dessus du pivot),
    /// 3 = 3 h (à droite), 6 = 6 h (en dessous), 9 = 9 h (à gauche).
    pub phase: f32,
}

/// Nombre de gouttes d'or dans chaque niveau.
pub const GOUTTES_PAR_NIVEAU: usize = 3;

/// Nature d'une tuile du décor. Tout sauf `Vide` est solide ; la variante ne
/// sert qu'à choisir le dessin (toit en tuiles, brique de cheminée, nuage).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tuile {
    Vide,
    Toit,
    Brique,
    Nuage,
}

impl Tuile {
    /// La tuile bloque-t-elle le joueur ?
    pub fn est_solide(self) -> bool {
        !matches!(self, Tuile::Vide)
    }
}

/// Sujet d'un panneau dessiné (tutoriel sans texte).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenrePanneau {
    Deplacer,
    Sauter,
    Planer,
    Champignon,
    Vent,
}

/// Panneau décoratif ; `(x, y)` est le coin haut-gauche de sa tuile.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Panneau {
    pub x: f32,
    pub y: f32,
    pub genre: GenrePanneau,
}

/// Genre d'un objet de décor : ne change que le dessin (aucune collision).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenreDecor {
    /// `H` : souche de cheminée (les toits, les gouttières, le marché couvert).
    Cheminee,
    /// `T` : antenne de télévision (les toits, les gouttières, le marché couvert).
    Antenne,
    /// `R` : paratonnerre (le grand orage, le clocher).
    Paratonnerre,
    /// `Q` : manche à air, au bord d'un courant ascendant qu'elle signale (vents
    /// d'altitude).
    MancheAAir,
    /// `K` : oiseau posé (au-dessus des nuages).
    Oiseau,
}

impl GenreDecor {
    /// Tous les genres, dans l'ordre de la légende.
    pub const TOUS: [GenreDecor; 5] = [
        GenreDecor::Cheminee,
        GenreDecor::Antenne,
        GenreDecor::Paratonnerre,
        GenreDecor::MancheAAir,
        GenreDecor::Oiseau,
    ];

    /// Lettre de la grille ASCII.
    pub fn lettre(self) -> char {
        match self {
            GenreDecor::Cheminee => 'H',
            GenreDecor::Antenne => 'T',
            GenreDecor::Paratonnerre => 'R',
            GenreDecor::MancheAAir => 'Q',
            GenreDecor::Oiseau => 'K',
        }
    }

    /// Genre désigné par une lettre de la grille, `None` si ce n'est pas du décor.
    /// (`find` sur `TOUS` : une seule table à tenir, `lettre`.)
    pub fn depuis_lettre(c: char) -> Option<GenreDecor> {
        GenreDecor::TOUS.into_iter().find(|g| g.lettre() == c)
    }
}

/// Objet de décor ; `(x, y)` est le coin haut-gauche de sa case. La case du
/// dessous est toujours solide (le parseur le vérifie) : l'objet y est posé.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjetDecor {
    pub x: f32,
    pub y: f32,
    pub genre: GenreDecor,
}

/// Un niveau entièrement chargé : tuiles, entités, courants, repères.
#[derive(Debug, Clone)]
pub struct Niveau {
    pub(crate) largeur: usize,
    pub(crate) hauteur: usize,
    pub(crate) tuiles: Vec<Tuile>,
    /// Coin haut-gauche du joueur au départ.
    pub depart: (f32, f32),
    /// Zone d'arrivée (le drapeau) : la toucher termine le niveau. `None` dans
    /// l'arène du Grand Nuage (`O`) : là, c'est le vaincre qui termine le niveau.
    pub arrivee: Option<Aabb>,
    /// Entités à l'état initial (à cloner pour recommencer).
    pub entites: Vec<Entite>,
    /// Colonnes de vent ascendant.
    pub courants: Vec<CourantAscendant>,
    /// Zones de vent latéral (`<` vers la gauche, `>` vers la droite).
    pub vents: Vec<VentLateral>,
    /// Panneaux du tutoriel.
    pub panneaux: Vec<Panneau>,
    /// Objets de décor (dessin seulement).
    pub decors: Vec<ObjetDecor>,
}

impl Niveau {
    /// Lit un niveau ASCII (voir la légende en tête de module). Un niveau sans
    /// aiguille d'horloge : une lettre `h` y est une erreur (son réglage manque).
    pub fn depuis_ascii(texte: &str) -> Result<Niveau, ErreurNiveau> {
        ascii::analyser(texte, &[])
    }

    /// Lit un niveau ASCII qui a des aiguilles d'horloge : `aiguilles` donne le
    /// réglage de chaque pivot `h`, dans l'ordre de lecture (voir `ReglageAiguille`).
    pub fn depuis_ascii_avec_aiguilles(
        texte: &str,
        aiguilles: &[ReglageAiguille],
    ) -> Result<Niveau, ErreurNiveau> {
        ascii::analyser(texte, aiguilles)
    }

    /// Largeur en tuiles.
    pub fn largeur(&self) -> usize {
        self.largeur
    }

    /// Hauteur en tuiles.
    pub fn hauteur(&self) -> usize {
        self.hauteur
    }

    /// Largeur en pixels.
    pub fn largeur_px(&self) -> f32 {
        self.largeur as f32 * TAILLE_TUILE
    }

    /// Hauteur en pixels ; sert aussi de « bas du monde » pour détecter le vide.
    pub fn hauteur_px(&self) -> f32 {
        self.hauteur as f32 * TAILLE_TUILE
    }

    /// Tuile de décor en `(tx, ty)`. Mêmes règles hors carte que `GrilleTuiles` :
    /// murs à gauche et à droite, ciel ouvert en haut, vide en bas.
    pub fn tuile_decor(&self, tx: i32, ty: i32) -> Tuile {
        if tx < 0 || tx >= self.largeur as i32 {
            return Tuile::Brique;
        }
        if ty < 0 || ty >= self.hauteur as i32 {
            return Tuile::Vide;
        }
        self.tuiles[ty as usize * self.largeur + tx as usize]
    }

    /// Nombre de lanternes (checkpoints).
    pub fn nb_lanternes(&self) -> usize {
        self.entites
            .iter()
            .filter(|e| matches!(e, Entite::Lanterne(_)))
            .count()
    }
}

/// Le moteur de collision ne connaît que ce contrat : un niveau en est un.
impl Grille for Niveau {
    fn tuile(&self, tx: i32, ty: i32) -> aabb::Tuile {
        if self.tuile_decor(tx, ty).est_solide() {
            aabb::Tuile::Solide
        } else {
            aabb::Tuile::Vide
        }
    }
}
