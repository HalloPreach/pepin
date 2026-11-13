//! Niveau 3 : « Le marché couvert ».
//!
//! « Sous les toiles, tout rebondit. » Entre les Gouttières et l'Orage, la nuit
//! bleue du marché : des auvents de boutique (lettre `A`) relancent Pépin de toit
//! en toit, des chats (lettre `z`) dorment sur les toits. La nouveauté du niveau,
//! c'est l'auvent : on se laisse tomber dessus, il relance en diagonale, vers le
//! haut et vers l'avant (4 tuiles plus loin sans rien toucher).
//!
//! Le parcours, de gauche à droite :
//! 1. le toit de départ, un premier chat endormi (on l'écrase ou on le saute) ;
//! 2. un premier auvent au bord du toit : en marchant droit devant, Pépin tombe
//!    dessus et rebondit (la première goutte est sur la trajectoire du rebond) ;
//! 3. un toit gardé par une flaque, puis deux auvents au-dessus de la rue du
//!    marché (deux rangées plus bas : on y tombe sans mourir, on remonte d'un saut) ;
//! 4. la goutte CACHÉE : un bloc de boutique, une fente où dort un chat, et un
//!    mur dont le pied est creusé d'un passage en cul-de-sac. Le chemin principal
//!    passe par-dessus (bloc, puis mur) ; la goutte est au fond du passage,
//!    derrière le chat : il faut se laisser tomber dans la fente pendant qu'il
//!    dort, l'écraser, puis ressortir par la fente ;
//! 5. le moment fort : SIX auvents à la suite au-dessus d'un vide de 23 tuiles,
//!    sans toucher le sol, tous espacés de 4 tuiles : sans rien toucher, chaque
//!    rebond retombe pile sur le suivant (le rythme se lit, il suffit de ne pas
//!    le casser) ; une goutte attend en l'air, sur le rebond du troisième ;
//! 6. un chat au pied d'un bloc (on le saute en montant sur le bloc), une flaque,
//!    deux derniers auvents au-dessus de la rue, et le drapeau.
//!
//! Chaque auvent est accroché à une boutique : au bord d'un toit, le toit est sa
//! façade ; au-dessus du vide, le rendu dessine une façade de boutique derrière
//! son mât (`game/src/scene.rs`, dessin seulement, aucune collision). Aucune
//! manche à air ici : on ne doit pas la confondre avec un auvent.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 3,
    nom: "Le marché couvert",
    theme: Theme::Marche,
    genre: GenreNiveau::Normal,
    intro: "Sous les toiles, tout rebondit.",
    mecanique: "Auvents qui relancent en diagonale, chats endormis à écraser pendant leur sommeil.",
    duree_cible_s: 50,
    ascii: ASCII,
    aiguilles: &[],
};

/// Dessin du niveau (153 x 11 tuiles). Légende : voir `crate::level`.
/// - Toits (dessus en rangée 8) : départ 0-19, B 24-39, C 48-58, D 71-84,
///   E 108-115, G 122-134, arrivée 143-152 ; blocs de boutique (dessus en
///   rangée 6) 59-63 et 116-121 ; mur (dessus en rangée 5) 65-70.
/// - Auvents (rangée 8, toile 4 px sous le dessus des toits) : 20 ; 40, 44 ;
///   85, 89, 93, 97, 101, 105 (le moment fort, au-dessus du vide) ; 135, 139.
/// - Rue du marché (rangée 10, sous les paires d'auvents) : colonnes 40-47 et
///   135-142 ; partout ailleurs, sous les auvents, le vide.
/// - Chats : 12 et 54 (sur les toits), 64 (au fond de la fente), 114 (au pied
///   du bloc 116). Flaques : 34 (toit B), 128 (toit G).
/// - Gouttes : 22 (rangée 5, sur le rebond du premier auvent), 67 (cachée au
///   fond du passage, derrière le chat de la fente), 95 (rangée 5, sur le rebond
///   du troisième auvent du moment fort).
/// - Lanternes : 3, 50, 73 (avant le moment fort), 110 (après).
const ASCII: &str = r"
.........................................................................................................................................................
.........................................................................................................................................................
.........................................................................................................................................................
.........................................................................................................................................................
.........................................................................................................................................................
......................o..........................................######........................o......................T..................................
...........................................................#####.BBBBBB.............................................######...............................
.@.L..H.....z.....................f...............L...z....BBBBB.BBBBBB..L...H...T............................L...z.BBBBBB......f....................X...
####################A...################A...A...###########BBBBBz..o.BB##############A...A...A...A...A...A..########BBBBBB#############A...A...##########
BBBBBBBBBBBBBBBBBBBB....BBBBBBBBBBBBBBBB........BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB.......................BBBBBBBBBBBBBBBBBBBBBBBBBBB........BBBBBBBBBB
BBBBBBBBBBBBBBBBBBBB....BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB.......................BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";
