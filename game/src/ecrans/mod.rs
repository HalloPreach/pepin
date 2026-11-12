//! Les écrans du jeu (titre, aide, carte de niveau, pause, fin) et leurs
//! outils de dessin communs.
//!
//! Chaque écran vit dans son petit fichier et ne fait que DESSINER ce qu'on lui
//! donne : l'état (quelle ligne est choisie, combien de gouttes...) appartient à
//! `app.rs`. La palette n'a pas de transparence : pour assombrir un fond on le
//! raye de lignes pleines une sur deux (`trame`), comme sur une vieille console.
//!
//! Notions Rust illustrées ici : sous-modules, `enum` d'écrans, `const fn`,
//! fonctions pures de mise en page (testées sans fenêtre).

use macroquad::prelude::draw_rectangle;
use pepin_core::palette::{Couleur, ARDOISE_OMBRE, BRUME, OR, OR_PALE};
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use crate::menu::Menu;
use crate::rendu::couleur_mq;
use crate::texte::{self, Style};

pub mod aide;
pub mod carte;
pub mod fin;
pub mod pause;
pub mod titre;

/// L'écran affiché.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ecran {
    Titre,
    Aide,
    /// Carte de présentation d'un niveau ; le nombre est le nombre de pas restants.
    Carte(u16),
    Jeu,
    Pause,
    Fin,
}

/// Position horizontale et largeur des lignes de menu (pixels logiques).
const MENU_X: f32 = 40.0;
const MENU_L: f32 = 240.0;
/// Échelle du texte des menus.
const ECHELLE_MENU: u32 = 2;

/// Où sont posées les lignes d'un menu : première ligne en `y0`, une ligne tous les `pas`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Disposition {
    pub y0: f32,
    pub pas: f32,
}

impl Disposition {
    /// Rectangle `(x, y, largeur, hauteur)` de la ligne `i` (c'est aussi sa zone cliquable).
    pub fn rect_ligne(&self, i: usize) -> (f32, f32, f32, f32) {
        (MENU_X, self.y0 + i as f32 * self.pas, MENU_L, self.pas)
    }

    /// Quelle ligne (parmi `nb`) contient le point `(x, y)` ? `None` si aucune.
    pub fn ligne_sous(&self, nb: usize, x: f32, y: f32) -> Option<usize> {
        (0..nb).find(|&i| {
            let (rx, ry, rl, rh) = self.rect_ligne(i);
            x >= rx && x < rx + rl && y >= ry && y < ry + rh
        })
    }
}

/// Mise en page du menu de l'écran donné (`None` : cet écran n'a pas de menu).
pub fn disposition(ecran: Ecran) -> Option<Disposition> {
    match ecran {
        Ecran::Titre => Some(Disposition {
            y0: 74.0,
            pas: 20.0,
        }),
        Ecran::Pause => Some(Disposition {
            y0: 68.0,
            pas: 20.0,
        }),
        Ecran::Fin => Some(Disposition {
            y0: 124.0,
            pas: 20.0,
        }),
        Ecran::Aide | Ecran::Carte(_) | Ecran::Jeu => None,
    }
}

/// Rectangle plein.
pub fn rect(x: f32, y: f32, w: f32, h: f32, c: Couleur) {
    draw_rectangle(x, y, w, h, couleur_mq(c));
}

/// Boîte avec un liseré : `bordure` sur 1 pixel, `fond` à l'intérieur.
pub fn panneau(x: f32, y: f32, w: f32, h: f32, fond: Couleur, bordure: Couleur) {
    rect(x, y, w, h, bordure);
    rect(x + 1.0, y + 1.0, w - 2.0, h - 2.0, fond);
}

/// Assombrit tout l'écran : une ligne sur deux est recouverte d'ardoise sombre.
pub fn trame() {
    for y in (0..HAUTEUR_LOGIQUE).step_by(2) {
        rect(0.0, y as f32, LARGEUR_LOGIQUE as f32, 1.0, ARDOISE_OMBRE);
    }
}

/// Image `(pas / duree)` d'une animation, en bouclant.
pub fn image<'a>(anim: &'a [&'a str], pas: u32, duree: u32) -> &'a str {
    anim[(pas / duree) as usize % anim.len()]
}

/// Dessine un menu : la ligne choisie est encadrée d'or, avec des chevrons qui
/// bougent. `anim` : compteur de pas, pour le mouvement.
pub fn dessiner_menu(libelles: &[&str], menu: &Menu, d: &Disposition, anim: u32) {
    for (i, libelle) in libelles.iter().enumerate() {
        let (x, y, w, h) = d.rect_ligne(i);
        let choisi = i == menu.selection();
        let style = if choisi {
            rect(x, y, w, h - 2.0, OR);
            rect(x + 1.0, y + 1.0, w - 2.0, h - 4.0, ARDOISE_OMBRE);
            Style::simple(OR_PALE, ECHELLE_MENU)
        } else {
            Style::simple(BRUME, ECHELLE_MENU).avec_ombre(ARDOISE_OMBRE)
        };
        texte::dessiner_centre(libelle, y + 2.0, style);
        if choisi {
            // Les chevrons font un petit va-et-vient d'un pixel d'écran.
            let dx = if (anim / 20).is_multiple_of(2) {
                0.0
            } else {
                2.0
            };
            let chevron = Style::simple(OR, ECHELLE_MENU);
            texte::dessiner(">", x + 8.0 + dx, y + 2.0, chevron);
            texte::dessiner_a_droite("<", x + w - 8.0 - dx, y + 2.0, chevron);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contenu;
    use crate::menu::Choix;
    use pepin_core::font::largeur_texte;

    #[test]
    fn tous_les_libelles_tiennent_dans_la_ligne_de_menu() {
        let tous = [
            Choix::Jouer,
            Choix::Aide,
            Choix::Son,
            Choix::Reprendre,
            Choix::Recommencer,
            Choix::QuitterAuTitre,
            Choix::Rejouer,
            Choix::RetourTitre,
        ];
        for c in tous {
            for son in [true, false] {
                let t = contenu::libelle(c, son);
                // Marge pour les chevrons de chaque côté (8 + 12 px).
                assert!(
                    (largeur_texte(t, ECHELLE_MENU) as f32) <= MENU_L - 2.0 * 24.0,
                    "« {t} » déborde de la ligne de menu"
                );
            }
        }
    }

    #[test]
    fn un_point_dans_une_ligne_la_designe() {
        let d = disposition(Ecran::Titre).unwrap();
        let (x, y, _, h) = d.rect_ligne(1);
        assert_eq!(d.ligne_sous(3, x + 5.0, y + 1.0), Some(1));
        assert_eq!(d.ligne_sous(3, x + 5.0, y + h - 0.5), Some(1));
        assert_eq!(d.ligne_sous(3, x + 5.0, y + h + 0.5), Some(2));
    }

    #[test]
    fn hors_des_lignes_rien_n_est_designe() {
        let d = disposition(Ecran::Pause).unwrap();
        assert_eq!(d.ligne_sous(4, 5.0, d.y0 + 5.0), None, "à gauche du menu");
        assert_eq!(d.ligne_sous(4, 160.0, d.y0 - 1.0), None, "au-dessus");
        // Une ligne qui n'existe pas (menu de 2 lignes) ne répond pas.
        assert_eq!(d.ligne_sous(2, 160.0, d.y0 + 2.5 * d.pas), None);
    }

    #[test]
    fn les_menus_tiennent_dans_l_ecran_et_les_lignes_sont_assez_grandes() {
        // 4 lignes au plus (pause avec son).
        for e in [Ecran::Titre, Ecran::Pause, Ecran::Fin] {
            let d = disposition(e).unwrap();
            assert!(d.pas >= 18.0, "lignes assez hautes pour un doigt");
            let (_, y, _, h) = d.rect_ligne(3);
            assert!(y + h <= HAUTEUR_LOGIQUE as f32 + 40.0);
        }
        let d = disposition(Ecran::Pause).unwrap();
        let (_, y, _, h) = d.rect_ligne(3);
        assert!(y + h <= HAUTEUR_LOGIQUE as f32 - 20.0, "pause : 4 lignes");
        let d = disposition(Ecran::Fin).unwrap();
        let (_, y, _, h) = d.rect_ligne(1);
        assert!(y + h <= HAUTEUR_LOGIQUE as f32, "fin : 2 lignes");
        let d = disposition(Ecran::Titre).unwrap();
        let (_, y, _, h) = d.rect_ligne(2);
        assert!(y + h <= 140.0, "titre : 3 lignes au-dessus du sol");
    }

    #[test]
    fn les_ecrans_sans_menu_n_ont_pas_de_disposition() {
        assert!(disposition(Ecran::Jeu).is_none());
        assert!(disposition(Ecran::Aide).is_none());
        assert!(disposition(Ecran::Carte(10)).is_none());
    }

    #[test]
    fn l_animation_boucle() {
        let anim = ["a", "b"];
        assert_eq!(image(&anim, 0, 10), "a");
        assert_eq!(image(&anim, 10, 10), "b");
        assert_eq!(image(&anim, 20, 10), "a");
    }
}
