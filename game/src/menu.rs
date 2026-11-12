//! Un menu vertical : seulement l'indice sélectionné, sans aucun dessin.
//!
//! Le texte des lignes vit ailleurs (`contenu.rs`) et le dessin dans `ui/` ;
//! ici on ne gère que « quelle ligne est surlignée ? ». Parce que le module est
//! pur, ses règles (retour à l'autre bout, indice borné) se testent sans fenêtre.
//!
//! Notions Rust illustrées ici : `struct` avec champs privés et accesseurs,
//! arithmétique modulaire sans dépassement (`saturating`, `%`).

/// Ce qu'une ligne de menu déclenche quand on la valide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choix {
    Jouer,
    Aide,
    Son,
    Reprendre,
    Recommencer,
    QuitterAuTitre,
    Rejouer,
    RetourTitre,
}

/// Le curseur d'un menu vertical.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Menu {
    nb: usize,
    selection: usize,
}

impl Menu {
    /// Un menu de `nb` lignes (au moins une), première ligne sélectionnée.
    pub fn new(nb: usize) -> Self {
        Self {
            nb: nb.max(1),
            selection: 0,
        }
    }

    /// Indice de la ligne sélectionnée (0 = première).
    pub fn selection(&self) -> usize {
        self.selection
    }

    /// Remonte d'une ligne ; depuis la première on retombe sur la dernière.
    pub fn monter(&mut self) {
        self.selection = (self.selection + self.nb - 1) % self.nb;
    }

    /// Descend d'une ligne ; depuis la dernière on revient à la première.
    pub fn descendre(&mut self) {
        self.selection = (self.selection + 1) % self.nb;
    }

    /// Sélectionne directement la ligne `i` (clic ou tap). Renvoie `false` si elle n'existe pas.
    pub fn choisir(&mut self, i: usize) -> bool {
        if i < self.nb {
            self.selection = i;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_part_de_la_premiere_ligne() {
        assert_eq!(Menu::new(3).selection(), 0);
    }

    #[test]
    fn la_navigation_boucle_dans_les_deux_sens() {
        let mut m = Menu::new(3);
        m.monter();
        assert_eq!(m.selection(), 2, "depuis le haut, on passe en bas");
        m.descendre();
        assert_eq!(m.selection(), 0, "depuis le bas, on repasse en haut");
        m.descendre();
        m.descendre();
        assert_eq!(m.selection(), 2);
    }

    #[test]
    fn choisir_refuse_une_ligne_inexistante() {
        let mut m = Menu::new(3);
        assert!(m.choisir(2));
        assert_eq!(m.selection(), 2);
        assert!(!m.choisir(3));
        assert_eq!(m.selection(), 2, "inchangé après un refus");
    }

    #[test]
    fn un_menu_vide_est_ramene_a_une_ligne() {
        let mut m = Menu::new(0);
        m.descendre();
        m.monter();
        assert_eq!(m.selection(), 0);
    }
}
