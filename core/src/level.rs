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
//!      il dort 1,7 s, se réveille (annonce de 0,8 s), reste éveillé 1,2 s, se
//!      rendort ; si Pépin est à portée à la fin de l'annonce (5 tuiles en x,
//!      2 en hauteur, sans mur entre eux), il lui bondit dessus pendant son
//!      éveil (sans quitter son toit), puis revient à sa place ; endormi on
//!      l'écrase en lui sautant dessus, éveillé le toucher tue (même par le
//!      dessus, même en plein bond)
//!   F  plateforme friable      e  éclair (colonne du 1er solide dessus, ou du
//!                                 haut de la carte, au 1er solide dessous)
//!   g  grêle : colonne de grêlons qui tombe EN CONTINU, de la case `g` au 1er
//!      solide dessous (ou au bas de la carte) ; elle tue parapluie fermé, on la
//!      traverse en planant dessous
//!   v  courant ascendant       1-7, 9 extrémités d'une plateforme mobile (2 par chiffre)
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
//!   Z  panneau « barque » (carte 9.6c) : la flèche du BAS vers le parapluie retourné
//!   H T R Q K  décor sans collision, posé sur le solide du dessous : cheminée,
//!              antenne, paratonnerre, manche à air, oiseau posé
//!   E I U J    décor de rue du Monde 2 (carte 9.6b), sans collision lui aussi :
//!              bouche d'égout, borne, poubelle, descente de gouttière
//!   m  faux mur (carte 8.5) : case LIBRE dessinée comme le solide qu'elle
//!      prolonge (voir `ZoneCachee`) ; Pépin y entre et la zone se révèle
//!   x  sortie secrète : un second drapeau, qui finit le niveau comme `X`
//!      (une au plus, en plus du drapeau `X`)
//!   *  lettre cachée (carte 8.6) : une des lettres de « PÉPIN », une au plus,
//!      laquelle : `crate::lettres` (8 x 8 au milieu de la case, comme `o`)
//!   P  mur de pluie (un au plus, carte 8.2) : son front part du bord droit de
//!      la case `P` et avance vers la droite ; il tue au contact (voir
//!      `crate::poursuite`). Hors de ce repère, la case est vide
//!   Ennemis d'eau du Monde 2 (carte 9.4, `entities::ennemi`, sous-module `eau`) :
//!   j  grenouille, posée sur ce qui est dessous (berge, nénuphar ou eau) ;
//!      Pépin à portée, elle se ramasse 0,5 s puis bondit en arc vers lui ;
//!      écrasable
//!   i  poisson : la case d'AIR juste au-dessus de l'eau (l'eau est dessous,
//!      obligatoire) ; remous 0,7 s, puis il saute à la verticale (3,5 tuiles) ;
//!      mortel hors de l'eau, jamais écrasable
//!   k  crabe : patrouille comme la flaque ; lui sauter dessus fait REBONDIR
//!      Pépin sans le blesser (carapace dure) ; de côté, il tue
//!   r  rat : patrouille ; Pépin sur sa ligne (pieds à la même hauteur, moins de
//!      6 tuiles, sans mur entre eux), il se dresse 0,5 s puis fonce jusqu'au
//!      mur ; écrasable
//!   u  mouette, perchée sur le solide du dessous (mât, grue) : un pigeon du
//!      port, même annonce, même piqué
//!   ~  {  }  eau calme, courant d'eau vers la gauche, vers la droite (carte
//!            9.2) : planer dedans donne la barque, y entrer autrement noie
//!   Y  bulles (carte 9.3) : colonne qui soulève comme un courant ascendant,
//!      prolongée vers le bas à travers l'eau qui est juste dessous
//!   n  nénuphar (carte 9.3), dans la case au-dessus de l'eau : une feuille sur
//!      laquelle on se pose ; elle s'enfonce sous Pépin et le met à l'eau en 1 s
//!   =  marée (carte 9.3) : zone où l'eau monte et descend par cycles
//!      (`entities::maree`) ; elle repose sur un solide ou sur de l'eau
//!   V  stalactite (carte 9.3), sous un solide : elle tremble 0,75 s quand
//!      Pépin passe dessous, puis tombe et se brise au sol ; mortelle au
//!      toucher, même accrochée (carte 9.26a)
//!   |  chute d'eau (carte 9.3) : colonne qui pousse vers le BAS (lente
//!      parapluie ouvert, rapide parapluie fermé)
//!   8  la Pieuvre du port, boss du Monde 2 (carte 9.12a, un boss au plus) : le
//!      `8` est dans l'eau de son bassin (case d'eau calme), qui couvre 8 cases à
//!      sa gauche et 9 à sa droite ; 9 colonnes de 2 cases (la colonne 0 : le `8`
//!      et sa voisine) où ses tentacules, annoncés 0,8 s, se dressent jusqu'en
//!      haut de la carte (mortels 0,5 s) ; entre deux motifs, sa tête (3 x 2 cases)
//!      sort sur l'eau au milieu (colonne 0), à gauche (-3) ou à droite (3) : on lui
//!      saute dessus, trois coups. Pas de drapeau `X` : la vaincre finit le niveau
//!   Mobilier de rue du Monde 2 (carte 9.20f) : des cases SOLIDES (des toits
//!   pour la collision) que le jeu dessine comme un seul objet par groupe de
//!   cases voisines de la même lettre (`Mobilier`) :
//!   q  voiture garée (4 x 2 cases : `.qq.` sur `qqqq`, le capot et le coffre
//!      plus bas que le toit)
//!   a  autobus garé (un rectangle ; ses roues sont dessinées sous lui)
//!   t  toit d'abribus (une rangée ; ses montants et sa vitre, dessinés jusqu'au
//!      sol, ne bloquent rien : on passe dessous)
//!   d  poubelle de rue (1 case)
//!   l  banc (une rangée)
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
mod secrets;
#[cfg(test)]
mod tests;

pub use erreur::ErreurNiveau;

/// Vitesse (px/s) des plateformes mobiles posées par le parseur.
pub const VITESSE_PLATEFORMES: f32 = ascii::VITESSE_MOBILE;
/// Taille (px) de la plateforme d'une aiguille d'horloge (`h`), largeur puis hauteur.
pub const TAILLE_AIGUILLE: (f32, f32) = ascii::TAILLE_AIGUILLE;

use crate::aabb::{self, Aabb, Grille, TAILLE_TUILE};
use crate::entities::{Entite, TypeEnnemi};
use crate::physics::{CourantAscendant, Eau, VentLateral};

/// Réglage d'une aiguille d'horloge (pivot `h`). Le fichier de niveau en donne
/// une petite table (`InfosNiveau::aiguilles`) : la 1re ligne règle le 1er `h`
/// rencontré en lisant la grille (de haut en bas, puis de gauche à droite), la
/// 2e le 2e, etc., comme un chiffre désigne une plateforme mobile.
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

/// Nature d'une tuile du décor. Tout sauf `Vide` et `Eau` est solide ; la
/// variante ne sert qu'à choisir le dessin (toit en tuiles, brique de
/// cheminée, nuage, eau). `Eau` (carte 9.2) n'est PAS solide : le moteur de
/// collision ne la voit pas, elle n'existe que pour le dessin et la
/// mécanique de la barque (les étendues d'eau sont assemblées en zones dans
/// `Niveau::eaux`, voir `level::ascii`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tuile {
    Vide,
    Toit,
    Brique,
    Nuage,
    /// Eau (lettres `~`, `{`, `}` de la grille) : non solide, surface = le haut
    /// de l'étendue (voir `Niveau::eaux`).
    Eau,
}

impl Tuile {
    /// La tuile bloque-t-elle le joueur ? L'eau ne bloque rien.
    pub fn est_solide(self) -> bool {
        !matches!(self, Tuile::Vide | Tuile::Eau)
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
    /// La barque (carte 9.6c) : BAS retourne le parapluie sur l'eau.
    Barque,
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
    /// `E` : bouche d'égout à grille (la rue du Monde 2, carte 9.6b).
    BoucheEgout,
    /// `I` : borne de trottoir (sa silhouette est celle de la lettre).
    Borne,
    /// `U` : poubelle de rue (sa silhouette est celle de la lettre).
    Poubelle,
    /// `J` : descente de gouttière qui déverse de l'eau dans la rue (le tuyau
    /// et son coude en J).
    Gouttiere,
}

impl GenreDecor {
    /// Tous les genres, dans l'ordre de la légende.
    pub const TOUS: [GenreDecor; 9] = [
        GenreDecor::Cheminee,
        GenreDecor::Antenne,
        GenreDecor::Paratonnerre,
        GenreDecor::MancheAAir,
        GenreDecor::Oiseau,
        GenreDecor::BoucheEgout,
        GenreDecor::Borne,
        GenreDecor::Poubelle,
        GenreDecor::Gouttiere,
    ];

    /// Lettre de la grille ASCII.
    pub fn lettre(self) -> char {
        match self {
            GenreDecor::Cheminee => 'H',
            GenreDecor::Antenne => 'T',
            GenreDecor::Paratonnerre => 'R',
            GenreDecor::MancheAAir => 'Q',
            GenreDecor::Oiseau => 'K',
            GenreDecor::BoucheEgout => 'E',
            GenreDecor::Borne => 'I',
            GenreDecor::Poubelle => 'U',
            GenreDecor::Gouttiere => 'J',
        }
    }

    /// Genre désigné par une lettre de la grille, `None` si ce n'est pas du décor.
    /// (`find` sur `TOUS` : une seule table à tenir, `lettre`.)
    pub fn depuis_lettre(c: char) -> Option<GenreDecor> {
        GenreDecor::TOUS.into_iter().find(|g| g.lettre() == c)
    }
}

/// Mobilier de rue (carte 9.20f) : des cases solides dessinées comme un objet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenreMobilier {
    /// `q` : voiture garée.
    Voiture,
    /// `a` : autobus garé.
    Autobus,
    /// `t` : toit d'abribus.
    Abribus,
    /// `d` : poubelle de rue.
    Poubelle,
    /// `l` : banc.
    Banc,
}

impl GenreMobilier {
    /// Tous les genres, dans l'ordre de la légende.
    pub const TOUS: [GenreMobilier; 5] = [
        GenreMobilier::Voiture,
        GenreMobilier::Autobus,
        GenreMobilier::Abribus,
        GenreMobilier::Poubelle,
        GenreMobilier::Banc,
    ];

    /// Lettre de la grille ASCII.
    pub fn lettre(self) -> char {
        match self {
            GenreMobilier::Voiture => 'q',
            GenreMobilier::Autobus => 'a',
            GenreMobilier::Abribus => 't',
            GenreMobilier::Poubelle => 'd',
            GenreMobilier::Banc => 'l',
        }
    }

    /// Genre désigné par une lettre de la grille, `None` si ce n'est pas du mobilier.
    pub fn depuis_lettre(c: char) -> Option<GenreMobilier> {
        GenreMobilier::TOUS.into_iter().find(|g| g.lettre() == c)
    }
}

/// Un objet du mobilier de rue : un groupe de cases voisines de la même lettre.
/// `(tx, ty)` : la case en haut à gauche du rectangle qui l'entoure, `(l, h)` :
/// sa taille en cases. Seules les cases de la lettre sont solides (la voiture a
/// deux coins libres en haut).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mobilier {
    pub genre: GenreMobilier,
    pub tx: i32,
    pub ty: i32,
    pub l: i32,
    pub h: i32,
}

impl Mobilier {
    /// La case `(tx, ty)` est-elle dans le rectangle de l'objet ?
    pub fn couvre(&self, tx: i32, ty: i32) -> bool {
        (self.tx..self.tx + self.l).contains(&tx) && (self.ty..self.ty + self.h).contains(&ty)
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
    /// Étendues d'eau (`~`, `{`, `}` de la grille, carte 9.2) : leur surface
    /// (le haut de la zone) est la seule règle de l'eau pour la mécanique de
    /// la barque. Vide dans tous les niveaux du Monde 1.
    pub eaux: Vec<Eau>,
    /// Panneaux du tutoriel.
    pub panneaux: Vec<Panneau>,
    /// Objets de décor (dessin seulement).
    pub decors: Vec<ObjetDecor>,
    /// Mobilier de rue (carte 9.20f) : ses cases sont solides (des toits), le
    /// jeu dessine chaque objet d'un seul tenant. Vide hors de la rue.
    pub mobilier: Vec<Mobilier>,
    /// Zones cachées derrière des faux murs `m` (carte 8.5).
    pub caches: Vec<ZoneCachee>,
    /// Sortie secrète `x` (carte 8.5) : la toucher finit le niveau comme le drapeau.
    pub sortie_secrete: Option<Aabb>,
    /// Front de départ du mur de pluie (case `P`, carte 8.2), en px : `None` sans mur.
    pub mur_de_pluie: Option<f32>,
}

/// Zone cachée (carte 8.5) : des faux murs `m` d'un seul tenant, et les cases
/// libres qu'ils enferment (la goutte d'une salle secrète). Aucune collision :
/// seul le dessin la cache, jusqu'à ce que Pépin la touche.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneCachee {
    /// Cases `(colonne, rangée)` et la tuile que chacune imite : le solide qui
    /// prolonge sa rangée de cases cachées (à droite, sinon à gauche), sinon
    /// celui du dessous, sinon la brique.
    pub cases: Vec<(i32, i32, Tuile)>,
    /// Case de l'indice (une brique décalée) : une case d'entrée, voisine d'une
    /// case libre du chemin ; `None` si aucune case ne s'y prête.
    pub indice: Option<(i32, i32)>,
}

impl ZoneCachee {
    /// La boîte `boite` touche-t-elle une case de la zone ?
    pub fn touche(&self, boite: &Aabb) -> bool {
        self.cases.iter().any(|&(x, y, _)| {
            let t = TAILLE_TUILE;
            boite.chevauche(&Aabb::new(x as f32 * t, y as f32 * t, t, t))
        })
    }
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

    /// Règle la hauteur de saut (px) de poissons précis (carte 9.11e) : chaque
    /// ligne `(colonne, rangée, hauteur)` vise le poisson dont la lettre `i` est
    /// dans cette case (comptée depuis 0). Les autres gardent `HAUT_SAUT_POISSON`.
    /// Une case sans poisson est une erreur (une faute de frappe ne passe pas).
    pub fn regler_sauts_poissons(
        &mut self,
        reglages: &[(usize, usize, f32)],
    ) -> Result<(), ErreurNiveau> {
        for &(colonne, rangee, hauteur) in reglages {
            let poisson = self.entites.iter_mut().find_map(|e| match e {
                Entite::Ennemi(p)
                    if p.type_ennemi == TypeEnnemi::Poisson
                        && p.case_du_poisson() == (colonne, rangee) =>
                {
                    Some(p)
                }
                _ => None,
            });
            match poisson {
                Some(p) => p.regler_saut_poisson(hauteur),
                None => {
                    return Err(ErreurNiveau::PoissonIntrouvable {
                        ligne: rangee + 1,
                        colonne: colonne + 1,
                    })
                }
            }
        }
        Ok(())
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

    /// Zone cachée qui contient la case `(tx, ty)` (son indice dans `caches`) et la
    /// tuile que cette case imite.
    pub fn cache_en(&self, tx: i32, ty: i32) -> Option<(usize, Tuile)> {
        self.caches.iter().enumerate().find_map(|(k, z)| {
            z.cases
                .iter()
                .find(|&&(x, y, _)| (x, y) == (tx, ty))
                .map(|&(_, _, t)| (k, t))
        })
    }

    /// Nombre de lanternes (checkpoints).
    pub fn nb_lanternes(&self) -> usize {
        self.entites
            .iter()
            .filter(|e| matches!(e, Entite::Lanterne(_)))
            .count()
    }

    /// L'étendue d'eau où la `boite` est vraiment (carte 9.2), voir
    /// `physics::eau_touchee` : en abscisse ET en hauteur.
    pub fn eau_touchee(&self, boite: &Aabb) -> Option<&crate::physics::Eau> {
        crate::physics::eau_touchee(boite, &self.eaux)
    }

    /// Le courant de l'étendue d'eau qui couvre la case `(tx, ty)` (carte 9.2)
    /// : 0 calme, -1 vers la gauche, +1 vers la droite. Sert AUSSIT au décor
    /// (`nom_tuile` en `scene.rs`) ; hors étendue d'eau, renvoie 0.
    pub fn courant_eau(&self, tx: i32, ty: i32) -> i8 {
        let case = Aabb::new(
            tx as f32 * TAILLE_TUILE,
            ty as f32 * TAILLE_TUILE,
            TAILLE_TUILE,
            TAILLE_TUILE,
        );
        self.eaux
            .iter()
            .find(|e| e.zone.chevauche(&case))
            .map_or(0, |e| e.courant)
    }

    /// La case `(tx, ty)` est-elle une case SOLIDE du mobilier de rue (carte
    /// 9.20f) ? Le jeu ne la dessine pas comme une tuile : l'objet s'en charge.
    pub fn est_mobilier(&self, tx: i32, ty: i32) -> bool {
        self.tuile_decor(tx, ty).est_solide() && self.mobilier.iter().any(|m| m.couvre(tx, ty))
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
