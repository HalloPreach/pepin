//! Sprites des mécaniques d'eau du Monde 2 (carte 9.3) : bulles, chutes d'eau,
//! nénuphar, stalactite, graduation et flèches de la marée. Même règle que les
//! autres fichiers de sprites : 1 caractère = 1 pixel, `.` transparent, chiffres
//! et lettres = indices de la palette (D cyan de la pluie, E cyan sombre, 7 blanc
//! cassé, B or, 0 ombre de contour, 2 à 5 ardoises).

// `bulle0.png` : Bulle 5x5 (lettre `Y`) : un anneau de cyan clair, un reflet blanc cassé.

// `bulle1.png` : Petite bulle 3x3.

// `chute0.png` : Chute d'eau (lettre `|`), une case 16x16 : des filets de cyan clair sur le cyan
// sombre, un reflet blanc cassé. Quatre images : le motif descend de 4 px à chaque
// image (il boucle sur ses 16 rangées), l'eau coule.

// `ecume0.png` : Écume au pied d'une chute d'eau (16x4), deux images qui alternent.

// `nenuphar.png` : Nénuphar (lettre `n`), 16x8 : une fleur corail posée sur une feuille vue de
// profil (cyan clair et cyan sombre : la palette n'a pas de vert), cernées de
// sombre. La feuille (rangées 4 à 7) est la boîte solide ; seules ses rangées
// au-dessus de l'eau se voient (la surface est redessinée par-dessus).

// `stalactite.png` : Stalactite (lettre `V`), 10x14 : une pointe de pierre (ardoises) accrochée au
// plafond, cernée de sombre.

// `stalactite_eclats.png` : Éclats d'une stalactite brisée au sol (16x4).

// `graduation.png` : Graduation de la marée (lettre `=`), 4x16 par case : une règle au mur, un
// trait long par case, un moyen par demi-case, un court par quart.

// `maree_monte.png` : Flèche de la marée (7x7, or) : l'eau va monter (clignote pendant l'annonce).

// `maree_descend.png` : La même flèche vers le bas : l'eau va descendre.

/// Bulles : la grosse, puis la petite.
pub const BULLES: [&str; 2] = ["bulle0", "bulle1"];
/// Chute d'eau : 4 images (le motif descend).
pub const CHUTE: [&str; 4] = ["chute0", "chute1", "chute2", "chute3"];
/// Écume au pied d'une chute d'eau : 2 images.
pub const ECUME: [&str; 2] = ["ecume0", "ecume1"];

/// Les sprites de ce fichier : (nom, dessin), lus dans `assets/sprites/objets_eau/`
/// (carte 10.4a : les dessins sont des fichiers PNG, voir `super::fichiers`).
pub fn sprites() -> &'static [(&'static str, crate::sprites::Image)] {
    crate::sprites::fichiers::famille("objets_eau")
}
