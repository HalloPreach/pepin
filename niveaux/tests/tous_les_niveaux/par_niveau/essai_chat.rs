//! Tests propres au niveau « essai_chat » (autrefois le `mod tests` de
//! `core/src/levels/essai_chat.rs` ; la grille et ses commentaires de conception
//! sont dans son fichier `.ron`).

use crate::commun::*;
use pepin_core::entities::{Entite, TypeEnnemi};

/// Le niveau de ce module, lu dans son fichier `.ron`.
fn infos() -> &'static InfosNiveau {
    par_cle("essai_chat")
}

/// Un seul chat, au fond de la fente (colonne 26, posé sur le sol), tourné
/// vers le passage (à droite).
#[test]
fn un_chat_au_fond_de_la_fente() {
    let n = infos().charger().unwrap();
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
