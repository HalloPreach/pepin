//! Niveau 5 : « Le clocher ».
//!
//! « Une seule direction : en haut. » Après l'Orage, le niveau VERTICAL du jeu :
//! 34 tuiles de large, 45 de haut (quatre écrans), à gravir autour de la tour du
//! clocher (colonnes 14 à 18), sous un ciel violet. La caméra suit Pépin en
//! hauteur (carte 5.2). En bas, tomber ne tue pas (le bas de la carte est plein) :
//! on retombe sur un étage plus bas, et on a perdu de la hauteur ; le danger, ce
//! sont les pigeons. En haut, dans la cage de l'horloge, une poutre hérissée de
//! pointes anti-pigeons barre le vide sous les aiguilles : là, tomber tue.
//!
//! Le parcours, de bas en haut :
//! 1. le pied de la tour (à gauche) : un escargot (lettre `s`) patrouille le
//!    sol ; on lui saute dessus pour rebondir sur le socle (80 px plus haut, plus
//!    qu'un saut). Sur le socle, un second escargot : son rebond seul porte à la
//!    goutte suspendue au-dessus ;
//! 2. une marche, l'arche du bas à travers la tour, puis la galerie de droite :
//!    un pigeon (lettre `p`) guette sur une corniche, tourné vers l'arche ; on
//!    passe dessous sans s'arrêter, il plonge derrière Pépin ;
//! 3. le ventilateur (lettre `w`) au bout de la galerie : son souffle (2 s, puis
//!    2 s d'arrêt annoncées comme un éclair) porte Pépin huit rangées plus haut,
//!    jusqu'à la première corniche du zigzag ;
//! 4. le zigzag : quatre corniches à trois rangées l'une de l'autre, alternées à
//!    gauche (contre la tour) et à droite (contre le mur). Au bout de la
//!    première, la 2e goutte, sous un second pigeon tourné vers Pépin qui
//!    arrive : on y court sans s'arrêter (il plonge derrière lui) ; s'arrêter
//!    sur sa diagonale, pour préparer le saut vers la corniche suivante, est
//!    le vrai danger ;
//! 5. la porte de la cage (l'arche du haut), puis le MOMENT FORT : la cage de
//!    l'horloge. Deux aiguilles (lettre `h`) tournent dans le sens horaire : on
//!    saute sur la grande quand elle passe au plus bas, elle emporte Pépin
//!    jusqu'au sommet du pilier (la 3e goutte) ; de là, la petite aiguille,
//!    puis la tour du drapeau, avec un paratonnerre. Entre le pilier et la
//!    tour, sous la petite aiguille, la poutre aux pointes : une chute y tue,
//!    on reprend à la lanterne du pilier.
//!
//! La cage reprend la géométrie du niveau d'essai « horloge » (mêmes rayons,
//! vitesses et heures de départ, mêmes écarts entre pivots et tours) : le
//! robot de QA y passe à coup sûr, aiguilles décalées d'une ou deux images.

use crate::level::ReglageAiguille;
use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 5,
    nom: "Le clocher",
    theme: Theme::Clocher,
    genre: GenreNiveau::Normal,
    intro: "Une seule direction : en haut.",
    mecanique:
        "Niveau vertical : escargots-tremplins, ventilateur, pigeons, aiguilles de l'horloge.",
    duree_cible_s: 55,
    ascii: ASCII,
    aiguilles: AIGUILLES,
};

/// Réglage des deux aiguilles, dans l'ordre de lecture de la grille (de haut en
/// bas) — les mêmes que le niveau d'essai « horloge » :
/// - 1re `h` (ligne 6, colonne 24) : la petite aiguille, 3 tuiles de rayon,
///   36 px/s (un tour en 8,4 s), partie de 9 h ;
/// - 2e `h` (ligne 11, colonne 8) : la grande aiguille, 4 tuiles de rayon,
///   40 px/s (un tour en 10 s), partie de 4 h.
const AIGUILLES: &[ReglageAiguille] = &[
    ReglageAiguille {
        rayon: 3.0,
        vitesse: 36.0,
        phase: 9.0,
    },
    ReglageAiguille {
        rayon: 4.0,
        vitesse: 40.0,
        phase: 4.0,
    },
];

/// Dessin du niveau (34 x 45 tuiles). Légende : voir `crate::level`.
/// - Tour du clocher : colonnes 14 à 18, du sommet du pilier (dessus en ligne 7)
///   au bas ; arche du bas lignes 35-36 (sol en 37), porte de la cage lignes
///   15-16 (sol en 17). Mur extérieur : colonnes 32-33.
/// - Pied de la tour (à gauche) : sol en ligne 43, départ colonne 1 ; escargot 1
///   (colonne 5) entre le bord gauche et le socle (colonnes 8 à 13, dessus en
///   ligne 38 : 80 px au-dessus du sol) ; escargot 2 sur le socle ; goutte 1 en
///   l'air (colonne 10, ligne 31), au sommet du rebond sur l'escargot 2 ;
///   lanterne sur le socle (12).
/// - Galerie de droite (dessus en ligne 37, colonnes 19 à 31) : lanterne (20),
///   pigeon 1 sur une corniche isolée (colonne 26, 4 rangées au-dessus de la
///   galerie), ventilateur (colonne 30) ; son courant monte jusque sous la
///   corniche F2 (ligne 26).
/// - Zigzag : F1 ligne 29 (colonnes 19 à 28 ; goutte 2 en colonne 20, gardée par
///   le pigeon 2 perché en (19, 24)), F2 ligne 26 (25 à 31), F3 ligne 23
///   (19 à 26), F4 ligne 20 (22 à 31), porte ligne 17 (19 à 23).
/// - Cage : sol en ligne 17 (colonnes 0 à 13, lanterne en 12) ; grande aiguille
///   (pivot (8, 11)) ; pilier (dessus en ligne 7 : lanterne 15, goutte 3 en 17) ;
///   petite aiguille (pivot (24, 6)) ; tour du drapeau (colonnes 29 à 33,
///   dessus en ligne 3, drapeau en colonne 30) ; entre les deux, la poutre
///   (ligne 14, colonnes 19 à 28) et ses pointes (ligne 13), 3 rangées sous le
///   plus bas de la petite aiguille.
/// - Décor : trois paratonnerres (cage, galerie, tour du drapeau).
const ASCII: &str = r"
..................................
..................................
..............................X.R.
.............................#####
.............................BBBBB
.............................BBBBB
...............L.o......h....BBBBB
..............#####..........BBBBB
..............BBBBB..........BBBBB
..............BBBBB..........BBBBB
..............BBBBB..........BBBBB
........h.....BBBBB..........BBBBB
..............BBBBB..........BBBBB
..............BBBBB^^^^^^^^^^BBBBB
..............BBBBBBBBBBBBBBBBBBBB
.............................BBBBB
.R..........L................BBBBB
########################........BB
BBBBBBBBBBBBBBBBBBB.............BB
BBBBBBBBBBBBBBBBBBB.............BB
BBBBBBBBBBBBBBBBBBB...##########BB
BBBBBBBBBBBBBBBBBBB.............BB
BBBBBBBBBBBBBBBBBBB.............BB
BBBBBBBBBBBBBBBBBBB########.....BB
BBBBBBBBBBBBBBBBBBBp............BB
BBBBBBBBBBBBBBBBBBBB............BB
BBBBBBBBBBBBBBBBBBB......#######BB
BBBBBBBBBBBBBBBBBBB.............BB
BBBBBBBBBBBBBBBBBBB.o...........BB
..............BBBBB##########...BB
..............BBBBB.............BB
..........o...BBBBB.............BB
..............BBBBB.......p.....BB
..............BBBBB.......B.....BB
..............BBBBB.............BB
................................BB
....................L...R.....w.BB
..........s.L.##################BB
........######BBBBBBBBBBBBBBBBBBBB
........BBBBBBBBBBBBBBBBBBBBBBBBBB
........BBBBBBBBBBBBBBBBBBBBBBBBBB
........BBBBBBBBBBBBBBBBBBBBBBBBBB
.@...s..BBBBBBBBBBBBBBBBBBBBBBBBBB
##############BBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::Entite;
    use crate::faisabilite::{analyser, Options};
    use crate::level::Niveau;

    /// Le clocher avec les lettres `lettre` remplacées par du vide, ligne par ligne
    /// (`garder(ligne, colonne)` : `true` pour garder celle-ci). Les aiguilles
    /// gardées reçoivent leur réglage, dans l'ordre de lecture.
    fn sans(lettre: char, garder: impl Fn(usize, usize) -> bool) -> Niveau {
        let mut reglages = Vec::new();
        let mut k = 0;
        let lignes: Vec<String> = ASCII
            .lines()
            .filter(|l| !l.is_empty())
            .enumerate()
            .map(|(r, l)| {
                l.chars()
                    .enumerate()
                    .map(|(c, ch)| {
                        if ch == 'h' {
                            k += 1;
                        }
                        if ch == lettre && !garder(r, c) {
                            '.'
                        } else {
                            if ch == 'h' {
                                reglages.push(AIGUILLES[k - 1]);
                            }
                            ch
                        }
                    })
                    .collect()
            })
            .collect();
        Niveau::depuis_ascii_avec_aiguilles(&lignes.join("\n"), &reglages).unwrap()
    }

    /// Gouttes dans l'ordre de lecture : 0 sur le pilier de la cage (ligne 6),
    /// 1 sur la corniche F1 (ligne 28), 2 en l'air au-dessus du socle (ligne 31).
    ///
    /// Les escargots servent : sans eux, Pépin reste au pied de la tour (le socle
    /// est 80 px plus haut, plus qu'un saut) ; sans celui du socle, le niveau se
    /// finit mais la goutte en l'air n'est pas prise (seul son rebond y porte).
    #[test]
    fn le_clocher_exige_ses_escargots() {
        let niv = sans('s', |_, _| false);
        let r = analyser(&niv, niv.depart, Options::default());
        assert!(
            !r.arrivee && r.gouttes == [false; 3],
            "sans escargot : {r:?}"
        );
        let niv = sans('s', |ligne, _| ligne != 37);
        let r = analyser(&niv, niv.depart, Options::default());
        assert!(
            r.arrivee && r.gouttes == [true, true, false],
            "sans l'escargot du socle : {r:?}"
        );
    }

    /// Sans le souffle du ventilateur, la galerie est un cul-de-sac : la corniche
    /// F1 est 128 px plus haut (seule la goutte en l'air, avant, est prise).
    #[test]
    fn le_clocher_exige_le_ventilateur() {
        let n = INFOS.charger().unwrap();
        let sans_courant = Options {
            courants: false,
            ..Options::default()
        };
        let r = analyser(&n, n.depart, sans_courant);
        assert!(
            !r.arrivee && r.gouttes == [false, false, true],
            "sans courant : {r:?}"
        );
    }

    /// Le moment fort, la cage de l'horloge, exige ses deux aiguilles : sans
    /// aiguille, le pilier (et sa goutte) est hors de portée ; sans la petite
    /// (la 1re en lisant), le pilier l'est, la tour du drapeau non.
    #[test]
    fn le_clocher_exige_ses_deux_aiguilles() {
        let niv = sans('h', |_, _| false);
        let r = analyser(&niv, niv.depart, Options::default());
        assert!(
            !r.arrivee && r.gouttes == [false, true, true],
            "sans aiguille : {r:?}"
        );
        let niv = sans('h', |ligne, _| ligne != 6);
        let r = analyser(&niv, niv.depart, Options::default());
        assert!(
            !r.arrivee && r.gouttes == [true; 3],
            "sans la petite aiguille : {r:?}"
        );
    }

    /// Aucune aiguille ne passe là où Pépin se tient debout sur une tuile : il
    /// serait « ramassé » par la plaque (le parseur ne le refuse pas, voir la note
    /// de la carte 5.10). Sur un tour complet, image par image, la plaque ne touche
    /// jamais les 14 px au-dessus d'un dessus de tuile.
    #[test]
    fn aucune_aiguille_ne_ramasse_pepin_debout() {
        use crate::aabb::{Aabb, TAILLE_TUILE as T};
        use crate::physics::JOUEUR_H;
        let n = INFOS.charger().unwrap();
        let solide = |tx: usize, ty: usize| n.tuile_decor(tx as i32, ty as i32).est_solide();
        let mut aiguilles = 0;
        for e in &n.entites {
            let Entite::Mobile(m) = e else { continue };
            let Some(tour) = m.cercle().map(|c| c.pas_par_tour().ceil() as u32) else {
                continue;
            };
            aiguilles += 1;
            let mut m = m.clone();
            for pas in 0..=tour {
                m.tourner(1);
                for ty in 1..n.hauteur() {
                    for tx in 0..n.largeur() {
                        if !solide(tx, ty) || solide(tx, ty - 1) {
                            continue;
                        }
                        let debout =
                            Aabb::new(tx as f32 * T, ty as f32 * T - JOUEUR_H, T, JOUEUR_H);
                        assert!(
                            !debout.chevauche(&m.boite),
                            "au pas {pas}, l'aiguille {:?} passe sur la tuile ({tx}, {ty})",
                            m.boite
                        );
                    }
                }
            }
        }
        assert_eq!(aiguilles, 2);
    }

    /// Les pièces du clocher sont là où le commentaire le dit : deux escargots, un
    /// ventilateur, deux aiguilles (pivots, rayons, heures) et les dix pointes de la
    /// poutre, sous la petite aiguille.
    #[test]
    fn les_pieces_du_clocher_a_leur_place() {
        let n = INFOS.charger().unwrap();
        assert_eq!((n.largeur(), n.hauteur()), (34, 45));
        let t = crate::aabb::TAILLE_TUILE;
        let case = |x: f32, y: f32| ((x / t) as i32, (y / t) as i32);
        let mut escargots = Vec::new();
        let mut ventilateurs = Vec::new();
        let mut aiguilles = Vec::new();
        let mut pointes = Vec::new();
        for e in &n.entites {
            match e {
                Entite::Pointe(p) => pointes.push(case(p.boite.x, p.boite.y)),
                Entite::Escargot(s) => escargots.push(case(s.boite.x, s.boite.y)),
                Entite::Ventilateur(v) => ventilateurs.push(case(v.boite.x, v.boite.y)),
                Entite::Mobile(m) => {
                    if let Some(c) = m.cercle() {
                        aiguilles.push((c.pivot, c.rayon, c.tour()));
                    }
                }
                _ => {}
            }
        }
        assert_eq!(escargots, vec![(10, 37), (5, 42)]);
        assert_eq!(ventilateurs, vec![(30, 36)]);
        assert_eq!(pointes, (19..=28).map(|c| (c, 13)).collect::<Vec<_>>());
        assert_eq!(
            aiguilles,
            vec![
                ((392.0, 104.0), 48.0, 0.75),
                ((136.0, 184.0), 64.0, 4.0 / 12.0)
            ]
        );
    }
}
