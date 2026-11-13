//! Niveau d'essai « chat » : un chat endormi garde le seul passage vers la goutte
//! cachée et le drapeau (carte 5.9 du plan de capture). Hors de la table
//! `NIVEAUX`, il n'existe QUE dans les tests et dans le build de QA
//! (fonctionnalité `debug_niveaux`, touche F7) ; en production il n'est pas
//! compilé dans le binaire, comme les autres niveaux d'essai.
//!
//! Le chat dort au fond d'une fente étroite, entre un bloc de toit et un grand
//! mur. On ne peut pas le contourner : le mur monte jusqu'au haut de la carte, et
//! le seul passage vers la suite (sous le mur) part du fond de la fente, là où il
//! dort. Il faut attendre qu'il dorme (les « z »), puis se laisser tomber du bloc
//! dans la fente, sur lui : il est écrasé et le passage s'ouvre. Pendant
//! l'annonce du réveil (le « ! ») on peut encore l'écraser ; réveillé, il tue même
//! par le dessus.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 0,
    nom: "Essai chat",
    theme: Theme::Toits,
    genre: GenreNiveau::Normal,
    intro: "Le chat endormi garde la goutte cachée : écrase-le pendant son sommeil.",
    mecanique: "Attendre que le chat dorme, puis lui tomber dessus ; réveillé, le toucher tue.",
    duree_cible_s: 30,
    ascii: ASCII,
};

/// 42 x 11 tuiles, thème Toits (la nuit bleue du futur marché couvert).
/// - Sol : lignes 9-10 (dessus à y = 144). Départ en colonne 1.
/// - Bloc de toit (colonnes 20 à 25, lignes 7-8, dessus à y = 112 : un saut y
///   monte), plein jusqu'au sol : pas de passage dessous.
/// - Grand mur (colonnes 27 à 32, lignes 0 à 7) : il monte jusqu'au haut de la
///   carte, on ne passe pas par-dessus. Dessous, le passage (ligne 8) n'a qu'une
///   tuile de haut.
/// - Fente (colonne 26) entre le bloc et le mur : le chat dort au fond (ligne 8),
///   tourné vers le passage (un solide à sa gauche, du vide à sa droite).
/// - La course jusqu'au bloc est assez longue pour que Pépin y arrive pendant
///   l'éveil du chat (entre 4,8 s et 6 s) : il faut l'attendre sur le bloc.
/// - Gouttes : en haut de la fente, juste au-dessus du chat (colonne 26, ligne 6 :
///   on la prend en se laissant tomber du bloc), cachée dans le passage derrière
///   le chat (colonne 29), avant le drapeau (colonne 38), sur le chemin du robot
///   de QA. Lanternes : départ (2), bloc (21), sortie du passage (34).
const ASCII: &str = r"
...........................######.........
...........................######.........
...........................######.........
...........................######.........
...........................######.........
...........................######.........
.....................L....o######.........
....................######.######.........
.@L.................######z..o....L...o..X
##########################################
##########################################
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{Entite, TypeEnnemi};

    /// Un seul chat, au fond de la fente (colonne 26, posé sur le sol), tourné
    /// vers le passage (à droite).
    #[test]
    fn un_chat_au_fond_de_la_fente() {
        let n = INFOS.charger().unwrap();
        let chats: Vec<(f32, f32, bool)> = n
            .entites
            .iter()
            .filter_map(|e| match e {
                Entite::Ennemi(c) if c.type_ennemi == TypeEnnemi::Chat => {
                    Some((c.boite.x, c.boite.bas(), c.vers_la_droite))
                }
                _ => None,
            })
            .collect();
        assert_eq!(chats, vec![(417.0, 144.0, true)]);
        assert_eq!((n.largeur(), n.hauteur()), (42, 11));
    }
}
