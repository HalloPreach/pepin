//! Les écrans du jeu (titre, aide, histoire, carte de niveau, pause, « plus de vies »,
//! fin) et leurs outils de dessin communs.
//!
//! Chaque écran vit dans son petit fichier et ne fait que DESSINER ce qu'on lui
//! donne : l'état (quelle ligne est choisie, combien de gouttes...) appartient à
//! `app.rs`. La palette n'a pas de transparence : pour assombrir un fond on le
//! tramé en damier, un pixel sur deux (`trame`), comme sur une vieille console.
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
pub mod bilan_course;
pub mod bilan_niveau;
pub mod boutique;
pub mod carte;
pub mod course_force;
pub mod facile;
pub mod fin;
pub mod histoire;
pub mod pause;
pub mod plan;
pub mod plus_de_vies;
pub mod resultats;
pub mod titre;

/// L'écran affiché.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ecran {
    Titre,
    Aide,
    /// La boutique (carte 7.5) : sa mise en page est dans `boutique.rs`.
    Boutique,
    /// Une image de l'histoire (carte 8.4) : avant certains niveaux, ou revue depuis
    /// le titre.
    Histoire(histoire::Chapitre),
    /// Carte de présentation d'un niveau ; le nombre est le nombre de pas restants.
    Carte(u16),
    Jeu,
    Pause,
    /// Pépin a perdu sa dernière vie (carte 6.2) ; le nombre est le nombre de pas
    /// restants avant que le niveau ne recommence tout seul.
    PlusDeVies(u16),
    Fin,
    /// La carte des niveaux (carte 7.4) : sa logique est dans `crate::plan`.
    Plan,
    /// Bilan d'un niveau joué seul depuis la carte (carte 7.4) : temps, défis,
    /// médaille, puis REJOUER ou CARTE.
    BilanNiveau,
    /// La démo du titre (carte 8.1) : le robot joue seul, voir `crate::demo`.
    Demo,
    /// Course contre le robot (carte 8.7) : le choix de la force du fantôme pour
    /// l'étape choisie sur la carte des niveaux (`course_force.rs`).
    ChoixForce,
    /// Bilan d'une course (carte 8.7) : GAGNÉ / PERDU et l'écart, puis REJOUER ou
    /// CARTE.
    BilanCourse,
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
        // Jusqu'à cinq lignes (cartes 7.4, 7.8b et 8.4) : JOUER, CARTE, BOUTIQUE,
        // HISTOIRE, COMMENT JOUER ; le son est une icône en bas à gauche. Resserrées
        // (18 px, le minimum pour un doigt), elles finissent en y = 154, sur le dessus
        // des toits (Pépin court à gauche).
        Ecran::Titre => Some(Disposition {
            y0: 64.0,
            pas: 18.0,
        }),
        Ecran::Pause => Some(Disposition {
            y0: 68.0,
            pas: 20.0,
        }),
        Ecran::Fin => Some(Disposition {
            y0: 124.0,
            pas: 20.0,
        }),
        // REJOUER et CARTE, dans le panneau du bilan (`bilan_niveau.rs`) ; même place
        // dans celui d'une course (carte 8.7).
        Ecran::BilanNiveau | Ecran::BilanCourse => Some(Disposition {
            y0: 124.0,
            pas: 20.0,
        }),
        // Les trois forces et CARTE (carte 8.7), sous le titre et le nom du niveau.
        Ecran::ChoixForce => Some(Disposition {
            y0: 50.0,
            pas: 20.0,
        }),
        Ecran::Aide
        | Ecran::Boutique
        | Ecran::Histoire(_)
        | Ecran::Carte(_)
        | Ecran::Jeu
        | Ecran::PlusDeVies(_)
        | Ecran::Plan
        | Ecran::Demo => None,
    }
}

/// Lignes du menu du titre avant une partie finie (sans HISTOIRE ni NOUVELLE PARTIE+,
/// 4) et après (6) : la disposition de base (`disposition`) est calculée pour 5.
pub const LIGNES_TITRE_MAX: usize = 5;
/// Titre à six lignes (carte 8.8, une fois le jeu fini) : le sous-titre s'efface,
/// et les lignes, au pas de 17 px, partent juste sous le grand titre (y = 52) pour
/// finir sur le dessus des toits (y = 154).
pub const TITRE_SIX_LIGNES: Disposition = Disposition {
    y0: 52.0,
    pas: 17.0,
};

/// Mise en page d'un menu de `nb` lignes. Le titre n'a HISTOIRE qu'après une partie
/// finie (carte 8.11) : ses 4 lignes d'avant sont centrées dans la place des 5, au
/// même pas de 18 px ; avec la NOUVELLE PARTIE + (6 lignes) : `TITRE_SIX_LIGNES`.
/// Les autres écrans : `disposition`.
pub fn disposition_menu(ecran: Ecran, nb: usize) -> Option<Disposition> {
    let mut d = disposition(ecran)?;
    if ecran == Ecran::Titre {
        if nb > LIGNES_TITRE_MAX {
            return Some(TITRE_SIX_LIGNES);
        }
        d.y0 += LIGNES_TITRE_MAX.saturating_sub(nb) as f32 * d.pas / 2.0;
    }
    Some(d)
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

/// Assombrit tout l'écran : un pixel sur deux, en damier (quinconce), est recouvert
/// d'ardoise sombre. Des lignes entières donnaient des rayures très dures sur les
/// niveaux clairs ; le damier garde le texte lisible et se fond dans le décor.
pub fn trame() {
    for y in 0..HAUTEUR_LOGIQUE {
        for x in (y % 2..LARGEUR_LOGIQUE).step_by(2) {
            rect(x as f32, y as f32, 1.0, 1.0, ARDOISE_OMBRE);
        }
    }
}

/// Image `(pas / duree)` d'une animation, en bouclant.
pub fn image<'a>(anim: &'a [&'a str], pas: u32, duree: u32) -> &'a str {
    anim[(pas / duree) as usize % anim.len()]
}

/// Icône du haut-parleur, son actif : le haut-parleur et deux ondes (7 pixels de haut,
/// comme la police).
const ICONE_SON_ACTIF: [&str; 7] = [
    "....X....X.",
    "...XX.X...X",
    "XXXXX..X..X",
    "XXXXX..X..X",
    "XXXXX..X..X",
    "...XX.X...X",
    "....X....X.",
];

/// Icône du haut-parleur, son coupé : une croix à la place des ondes (même largeur,
/// l'icône ne saute pas quand on bascule).
const ICONE_SON_COUPE: [&str; 7] = [
    "....X......",
    "...XX.X...X",
    "XXXXX..X.X.",
    "XXXXX...X..",
    "XXXXX..X.X.",
    "...XX.X...X",
    "....X......",
];

/// Icône du son en bas à gauche du titre et de la pause (carte 7.8b, à la place de
/// la ligne de menu de la carte 7.8) : échelle, marge au bord de l'écran.
const ECHELLE_ICONE_SON: u32 = 2;
const MARGE_ICONE_SON: f32 = 6.0;
/// Débord de la zone de clic autour de l'icône, de chaque côté : assez grande pour
/// un doigt (34 x 26 pixels logiques, ~1 cm sur un téléphone) sans toucher le menu
/// (qui commence en x = `MENU_X`).
const DEBORD_CLIC_ICONE: f32 = 6.0;

/// Rectangle `(x, y, largeur, hauteur)` de l'icône du son (pixels logiques).
fn rect_icone_son() -> (f32, f32, f32, f32) {
    let e = ECHELLE_ICONE_SON as f32;
    let l = ICONE_SON_ACTIF[0].len() as f32 * e;
    let h = ICONE_SON_ACTIF.len() as f32 * e;
    (
        MARGE_ICONE_SON,
        HAUTEUR_LOGIQUE as f32 - MARGE_ICONE_SON - h,
        l,
        h,
    )
}

/// Le point `(x, y)` (clic, toucher) tombe-t-il sur l'icône du son ? La zone déborde
/// de l'icône de `DEBORD_CLIC_ICONE` de chaque côté (bornée par l'écran).
pub fn sur_icone_son(x: f32, y: f32) -> bool {
    let (ix, iy, l, h) = rect_icone_son();
    let d = DEBORD_CLIC_ICONE;
    x >= ix - d && x < ix + l + d && y >= iy - d && y < iy + h + d
}

/// Dessine l'icône du son en bas à gauche : haut-parleur et ondes si `actif`, barré
/// d'une croix sinon. Couleurs du texte des menus (ombre portée), pixels entiers.
pub fn dessiner_icone_son(actif: bool) {
    let motif: &[&str] = if actif {
        &ICONE_SON_ACTIF
    } else {
        &ICONE_SON_COUPE
    };
    let (x, y, _, _) = rect_icone_son();
    let e = ECHELLE_ICONE_SON as f32;
    let passe = |dx: f32, couleur: Couleur| {
        for (ly, ligne) in motif.iter().enumerate() {
            for (lx, c) in ligne.chars().enumerate() {
                if c == 'X' {
                    rect(
                        x + dx + lx as f32 * e,
                        y + dx + ly as f32 * e,
                        e,
                        e,
                        couleur,
                    );
                }
            }
        }
    };
    passe(e, ARDOISE_OMBRE);
    passe(0.0, BRUME);
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
    use crate::course::Force;
    use crate::menu::Choix;
    use pepin_core::font::largeur_texte;

    #[test]
    fn tous_les_libelles_tiennent_dans_la_ligne_de_menu() {
        let tous = [
            Choix::Jouer,
            Choix::NouvellePartiePlus,
            Choix::Boutique,
            Choix::Histoire,
            Choix::Aide,
            Choix::Reprendre,
            Choix::Recommencer,
            Choix::QuitterAuTitre,
            Choix::Rejouer,
            Choix::RetourTitre,
            Choix::Carte,
            Choix::RejouerNiveau,
            Choix::RejouerCourse,
            Choix::Force(Force::Debutant),
            Choix::Force(Force::Habitue),
            Choix::Force(Force::Expert),
        ];
        for c in tous {
            let t = contenu::libelle(c);
            // Marge pour les chevrons de chaque côté (8 + 12 px).
            assert!(
                (largeur_texte(t, ECHELLE_MENU) as f32) <= MENU_L - 2.0 * 24.0,
                "« {t} » déborde de la ligne de menu"
            );
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
        assert_eq!(d.ligne_sous(3, 5.0, d.y0 + 5.0), None, "à gauche du menu");
        assert_eq!(d.ligne_sous(3, 160.0, d.y0 - 1.0), None, "au-dessus");
        // Une ligne qui n'existe pas (menu de 2 lignes) ne répond pas.
        assert_eq!(d.ligne_sous(2, 160.0, d.y0 + 2.5 * d.pas), None);
    }

    #[test]
    fn les_menus_tiennent_dans_l_ecran_et_les_lignes_sont_assez_grandes() {
        // 3 lignes au plus (titre, pause : le son est une icône depuis la carte 7.8b).
        for e in [Ecran::Titre, Ecran::Pause, Ecran::Fin] {
            let d = disposition(e).unwrap();
            assert!(d.pas >= 18.0, "lignes assez hautes pour un doigt");
            let (_, y, _, h) = d.rect_ligne(2);
            assert!(y + h <= HAUTEUR_LOGIQUE as f32 + 20.0);
        }
        let d = disposition(Ecran::Pause).unwrap();
        let (_, y, _, h) = d.rect_ligne(2);
        assert!(y + h <= HAUTEUR_LOGIQUE as f32 - 40.0, "pause : 3 lignes");
        let d = disposition(Ecran::Fin).unwrap();
        let (_, y, _, h) = d.rect_ligne(1);
        assert!(y + h <= HAUTEUR_LOGIQUE as f32, "fin : 2 lignes");
        let d = disposition(Ecran::Titre).unwrap();
        let (_, y, _, h) = d.rect_ligne(4);
        assert!(y + h <= titre::SOL, "titre : 5 lignes au-dessus du sol");
        assert!(d.y0 >= titre::FIN_SOUS_TITRE, "sous le sous-titre");
        // Six lignes (carte 8.8) : sous le grand titre, au-dessus du sol.
        let d = disposition_menu(Ecran::Titre, 6).unwrap();
        let (_, y, _, h) = d.rect_ligne(5);
        assert!(
            y + h <= titre::SOL && d.y0 >= titre::FIN_TITRE,
            "titre : 6 lignes"
        );
    }

    #[test]
    fn les_ecrans_sans_menu_n_ont_pas_de_disposition() {
        assert!(disposition(Ecran::Jeu).is_none());
        assert!(disposition(Ecran::Aide).is_none());
        assert!(disposition(Ecran::Carte(10)).is_none());
        assert!(disposition(Ecran::PlusDeVies(10)).is_none());
        assert!(disposition(Ecran::Histoire(histoire::Chapitre::Depart)).is_none());
    }

    #[test]
    fn l_animation_boucle() {
        let anim = ["a", "b"];
        assert_eq!(image(&anim, 0, 10), "a");
        assert_eq!(image(&anim, 10, 10), "b");
        assert_eq!(image(&anim, 20, 10), "a");
    }
}
