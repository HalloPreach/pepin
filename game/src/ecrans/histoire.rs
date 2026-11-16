//! L'histoire en trois images fixes (carte 8.4) : pourquoi Pépin monte, et ce qu'il
//! trouve là-haut.
//!
//! - `Depart`, avant le premier niveau : les toits sous l'averse, fenêtres éteintes,
//!   Pépin sur une cheminée qui regarde en haut ;
//! - `Lueur`, après le Grand Nuage : il s'enfuit (il reviendra au sommet), une lueur
//!   d'or perce, une goutte d'or (un éclat de soleil) y descend ;
//! - `Soleil`, avant la descente : le soleil ouvre un œil au-dessus de la mer de
//!   nuages, Pépin à côté, parapluie fermé.
//!
//! Chaque image est DESSINÉE EN CODE avec les briques du jeu (décor des thèmes,
//! tuiles, sprites, palette de 16 couleurs) et s'anime à peine (pluie, clignement,
//! goutte qui descend, œil qui s'ouvre). Deux lignes de texte et « APPUIE POUR
//! CONTINUER » dans un bandeau en bas. Où elles se placent dans la partie :
//! `chapitre_avant`, une règle pure sur la table des niveaux.

use pepin_core::font::HAUTEUR_GLYPHE;
use pepin_core::levels::GenreNiveau;
use pepin_core::palette::{
    ARDOISE, ARDOISE_CLAIRE, ARDOISE_NUIT, ARDOISE_OMBRE, BLANC_CASSE, BRUME, NUAGE, OR,
    ORANGE_SOLEIL, OR_PALE,
};
use pepin_core::zones::{InfosNiveau, Theme};
use pepin_core::LARGEUR_LOGIQUE;

use super::{image, panneau, rect};
use crate::contenu;
use crate::decor;
use crate::skins::Skin;
use crate::sprites::objets::GOUTTE;
use crate::sprites::Banque;
use crate::texte::{self, Style};

/// Les trois images, dans l'ordre du récit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chapitre {
    /// Avant le premier niveau : la pluie sans fin, le soleil qui dort.
    Depart,
    /// Après le Grand Nuage (le boss) : une lueur d'or.
    Lueur,
    /// Avant la descente (l'épilogue) : le soleil se réveille.
    Soleil,
}

impl Chapitre {
    pub const TOUS: [Chapitre; 3] = [Chapitre::Depart, Chapitre::Lueur, Chapitre::Soleil];

    /// Place dans le récit, à partir de 0.
    pub fn indice(self) -> usize {
        self as usize
    }

    /// L'image suivante (`None` après la dernière).
    pub fn suivant(self) -> Option<Chapitre> {
        Self::TOUS.get(self.indice() + 1).copied()
    }

    /// Ses deux lignes de texte (`contenu::RECIT`).
    pub fn lignes(self) -> [&'static str; 2] {
        contenu::RECIT[self.indice()]
    }
}

/// L'image à montrer juste avant le niveau `indice` de `niveaux` (règle pure, testée
/// sur de fausses tables) :
/// - le premier niveau : `Depart` ;
/// - le niveau qui suit celui du Grand Nuage (thème `Cyclone`) : `Lueur` ;
/// - l'épilogue (la descente) : `Soleil`. Il vient après le sommet (le retour du Grand
///   Nuage, carte 8.2) et après le bonus s'il est joué.
pub fn chapitre_avant(niveaux: &[InfosNiveau], indice: usize) -> Option<Chapitre> {
    let niveau = niveaux.get(indice)?;
    if indice == 0 {
        Some(Chapitre::Depart)
    } else if niveau.genre == GenreNiveau::Epilogue {
        Some(Chapitre::Soleil)
    } else if niveaux[indice - 1].theme == Theme::Cyclone {
        Some(Chapitre::Lueur)
    } else {
        None
    }
}

/// Pas pendant lesquels l'image ignore les appuis : un saut (Espace, qui valide aussi
/// les menus) tenu à la fin d'un niveau ne la fait pas disparaître sans être vue.
pub const DELAI_PAS: u32 = 30;

/// L'image accepte-t-elle qu'on la passe ? `anim` : pas depuis son ouverture.
pub fn prete(anim: u32) -> bool {
    anim >= DELAI_PAS
}

/// Haut du bandeau de texte (il va jusqu'en bas de l'écran).
const Y_BANDEAU: f32 = 136.0;
/// Les deux lignes du récit, puis l'invitation.
const Y_LIGNE_1: f32 = 142.0;
const Y_LIGNE_2: f32 = 153.0;
const Y_CONTINUER: f32 = 168.0;
/// Le sol de chaque image (dessus des toits ou des nuages), au-dessus du bandeau.
const SOL: f32 = 112.0;

/// Hauteur d'une ligne de texte à l'échelle 1, ombre comprise.
const LIGNE: f32 = HAUTEUR_GLYPHE as f32 + 1.0;

// La mise en page tient, vérifiée À LA COMPILATION : le sol (et ce qui y est posé,
// tuiles de 16 px) au-dessus du bandeau, les trois lignes du bandeau sans se toucher
// (2 px au moins), la dernière dans l'écran (180 px).
const _: () = {
    assert!(SOL + 16.0 <= Y_BANDEAU + 8.0);
    assert!(Y_LIGNE_1 >= Y_BANDEAU + 4.0);
    assert!(Y_LIGNE_2 >= Y_LIGNE_1 + LIGNE + 2.0);
    assert!(Y_CONTINUER >= Y_LIGNE_2 + LIGNE + 4.0);
    assert!(Y_CONTINUER + LIGNE <= 180.0 - 3.0);
};

/// Dessine l'image `chapitre`. `anim` : pas écoulés depuis son ouverture (petites
/// animations, invitation qui clignote quand l'image peut être passée).
pub fn dessiner(banque: &Banque, skin: Skin, chapitre: Chapitre, anim: u32) {
    match chapitre {
        Chapitre::Depart => dessiner_depart(banque, skin, anim),
        Chapitre::Lueur => dessiner_lueur(banque, skin, anim),
        Chapitre::Soleil => dessiner_soleil(banque, skin, anim),
    }
    dessiner_bandeau(chapitre, anim);
}

/// Le bandeau du bas : les deux lignes du récit, l'invitation, et le numéro de l'image.
fn dessiner_bandeau(chapitre: Chapitre, anim: u32) {
    let l = LARGEUR_LOGIQUE as f32;
    // Liseré en haut seulement : le panneau déborde de l'écran à gauche, à droite et en bas.
    panneau(
        -1.0,
        Y_BANDEAU,
        l + 2.0,
        50.0,
        ARDOISE_OMBRE,
        ARDOISE_CLAIRE,
    );
    let ombre = |c| Style::simple(c, 1).avec_ombre(ARDOISE_NUIT);
    let [a, b] = chapitre.lignes();
    texte::dessiner_centre(a, Y_LIGNE_1, ombre(BLANC_CASSE));
    texte::dessiner_centre(b, Y_LIGNE_2, ombre(OR_PALE));
    if prete(anim) && ((anim - DELAI_PAS) / 30).is_multiple_of(2) {
        texte::dessiner_centre(contenu::APPUIE_POUR_CONTINUER, Y_CONTINUER, ombre(BRUME));
    }
    let numero = format!("{}/{}", chapitre.indice() + 1, Chapitre::TOUS.len());
    // Dans le bandeau, en bas à droite, aux couleurs de l'invitation (carte 8.11) : en
    // haut à droite, il se lisait mal sur le ciel saumon de l'image 3.
    texte::dessiner_a_droite(&numero, l - 4.0, Y_CONTINUER, ombre(BRUME));
}

/// Pépin (ou le skin choisi) qui regarde en l'air, et cligne des yeux de temps en temps.
fn pepin_regarde_en_haut(skin: Skin, anim: u32) -> &'static str {
    let [cligne, ciel, _] = skin.images().attente;
    if anim % 150 < 8 {
        cligne
    } else {
        ciel
    }
}

/// Une rangée de tuiles de toit sur toute la largeur, dessus en `SOL`.
fn toits(banque: &Banque) {
    for k in 0..=20 {
        let x = k as f32 * 16.0;
        banque.dessiner_tuile(Theme::Toits, "toit_haut", x, SOL);
        banque.dessiner_tuile(Theme::Toits, "toit_plein", x, SOL + 16.0);
    }
}

/// Un îlot de nuages de `n` tuiles (bords compris), dessus en `SOL`, à partir de `x`.
fn ilot(banque: &Banque, theme: Theme, x: f32, n: usize) {
    for k in 0..n {
        let nom = match k {
            0 => "nuage_bord_g",
            _ if k + 1 == n => "nuage_bord_d",
            _ => "nuage_haut",
        };
        let xk = x + k as f32 * 16.0;
        banque.dessiner_tuile(theme, nom, xk, SOL);
        banque.dessiner_tuile(theme, "nuage_plein", xk, SOL + 16.0);
    }
}

/// Une maison de la ville derrière les toits : façade d'ardoise sombre au contour d'un
/// pixel, toit en pente (marches de 2 px), deux rangées de fenêtres ÉTEINTES.
/// `(x, haut)` : coin haut-gauche de la façade ; elle descend jusqu'au `SOL`.
fn maison(x: f32, haut: f32, largeur: f32) {
    let h = SOL - haut;
    rect(x, haut, largeur, h, ARDOISE_OMBRE);
    rect(x + 1.0, haut + 1.0, largeur - 2.0, h - 1.0, ARDOISE_NUIT);
    // Le toit : de plus en plus étroit, 2 px de moins de chaque côté par rangée.
    let mut k = 0.0;
    while largeur - 4.0 * k > 4.0 {
        rect(
            x + 2.0 * k,
            haut - 2.0 * (k + 1.0),
            largeur - 4.0 * k,
            2.0,
            ARDOISE_OMBRE,
        );
        k += 1.0;
    }
    // Fenêtres : 6 x 8, cadre d'ardoise, vitre noire (personne n'allume plus).
    let n = ((largeur - 6.0) / 12.0).floor().max(1.0) as i32;
    let marge = (largeur - n as f32 * 12.0 + 6.0) / 2.0;
    for rangee in 0..2 {
        let y = haut + 6.0 + rangee as f32 * 13.0;
        if y + 8.0 > SOL - 2.0 {
            break;
        }
        for i in 0..n {
            let xf = x + marge + i as f32 * 12.0;
            rect(xf, y, 6.0, 8.0, ARDOISE);
            rect(xf + 1.0, y + 1.0, 4.0, 6.0, ARDOISE_OMBRE);
        }
    }
}

/// Image 1 : nuit d'averse sur les toits, fenêtres éteintes. Pépin est debout sur une
/// cheminée et regarde le ciel.
fn dessiner_depart(banque: &Banque, skin: Skin, anim: u32) {
    decor::dessiner_fond(Theme::Toits, 0, 0, anim);
    for (x, haut, largeur) in [
        (6.0, 70.0, 54.0),
        (78.0, 82.0, 42.0),
        (188.0, 66.0, 42.0),
        (252.0, 78.0, 62.0),
    ] {
        maison(x, haut, largeur);
    }
    toits(banque);
    banque.dessiner("antenne", 52.0, SOL - 24.0, false);
    banque.dessiner("cheminee", 236.0, SOL - 20.0, false);
    // La cheminée de Pépin, au milieu (12 x 20) ; Pépin (16 x 16) debout dessus.
    let x_cheminee = (LARGEUR_LOGIQUE as f32 - 12.0) / 2.0;
    banque.dessiner("cheminee", x_cheminee, SOL - 20.0, false);
    banque.dessiner(
        pepin_regarde_en_haut(skin, anim),
        x_cheminee - 2.0,
        SOL - 36.0,
        false,
    );
    // La pluie tombe devant tout (elle ne s'arrête plus).
    decor::dessiner_avant(Theme::Toits, anim);
}

/// Image 2 : le ciel du cyclone, sans pluie. Le Grand Nuage, essoufflé, s'enfuit vers
/// la droite en descendant (sous la mer de nuages : il reviendra au sommet) ; un rai
/// de lumière d'or perce par la trouée et une goutte d'or (un éclat de soleil) y
/// descend vers Pépin.
fn dessiner_lueur(banque: &Banque, skin: Skin, anim: u32) {
    decor::dessiner_fond(Theme::Cyclone, 0, 0, anim);
    let centre = LARGEUR_LOGIQUE as i32 / 2;
    // Le rai : un trapèze tramé (un pixel sur deux) qui s'élargit vers le bas,
    // bordé d'or plus franc au centre.
    for y in 0..SOL as i32 {
        let demi = 4 + y / 10;
        for x in (centre - demi)..=(centre + demi) {
            if (x + y) % 2 == 0 {
                rect(x as f32, y as f32, 1.0, 1.0, OR_PALE);
            }
        }
        rect((centre - 1) as f32, y as f32, 2.0, 1.0, OR);
    }
    // Le Grand Nuage s'enfuit en descendant (il quitte l'écran en une dizaine de
    // secondes).
    let x_nuage = 196.0 + (anim / 5).min(160) as f32;
    let y_nuage = 18.0 + (anim / 20).min(24) as f32;
    banque.dessiner(
        image(&["nuage_essouffle0", "nuage_essouffle1"], anim, 20),
        x_nuage,
        y_nuage,
        false,
    );
    ilot(banque, Theme::Cyclone, centre as f32 - 48.0, 6);
    banque.dessiner(
        pepin_regarde_en_haut(skin, anim),
        centre as f32 - 8.0,
        SOL - 16.0,
        false,
    );
    // La goutte d'or descend dans le rai, de l'or du haut jusqu'au-dessus de Pépin,
    // puis recommence (10 x 10, elle tourne sur elle-même).
    let course = (SOL - 16.0 - 12.0) as u32;
    let y_goutte = ((anim * 2 / 5) % (course + 20)) as f32 - 10.0;
    if y_goutte < course as f32 {
        banque.dessiner(
            image(&GOUTTE, anim, 8),
            centre as f32 - 5.0,
            y_goutte,
            false,
        );
    }
}

/// Centre du soleil du thème `Soleil` dessiné par `decor` (caméra à 0).
const SOLEIL: (f32, f32) = (236.0, 44.0);

/// Image 3 : au-dessus de la mer de nuages, le soleil (celui du décor) ouvre un œil ;
/// des gouttes d'or remontent vers lui. Pépin, parapluie fermé, le regarde.
fn dessiner_soleil(banque: &Banque, skin: Skin, anim: u32) {
    decor::dessiner_fond(Theme::Soleil, 0, 0, anim);
    // Visage du soleil : deux yeux fermés (traits), le gauche s'ouvre après une
    // seconde, puis cligne de temps en temps ; un petit sourire.
    let (sx, sy) = SOLEIL;
    let oeil_ouvert = anim >= 60 && (anim - 60) % 180 >= 6;
    for (dx, ouvert) in [(-8.0, oeil_ouvert), (5.0, false)] {
        if ouvert {
            rect(sx + dx, sy - 4.0, 3.0, 4.0, ARDOISE_NUIT);
            rect(sx + dx + 1.0, sy - 4.0, 1.0, 1.0, BLANC_CASSE);
        } else {
            rect(sx + dx, sy - 1.0, 3.0, 1.0, ORANGE_SOLEIL);
        }
    }
    rect(sx - 3.0, sy + 4.0, 6.0, 1.0, ORANGE_SOLEIL);
    rect(sx - 4.0, sy + 3.0, 1.0, 1.0, ORANGE_SOLEIL);
    rect(sx + 3.0, sy + 3.0, 1.0, 1.0, ORANGE_SOLEIL);
    // Trois éclats d'or remontent de Pépin au soleil, l'un après l'autre.
    let depart = (112.0, SOL - 24.0);
    for k in 0..3u32 {
        let t = ((anim + k * 50) % 150) as f32 / 150.0;
        let x = depart.0 + (sx - 5.0 - depart.0) * t;
        let y = depart.1 + (sy + 16.0 - depart.1) * t;
        banque.dessiner(image(&GOUTTE, anim + k * 3, 8), x, y, false);
    }
    mer_de_nuages(anim);
    ilot(banque, Theme::Soleil, 64.0, 6);
    banque.dessiner(pepin_regarde_en_haut(skin, anim), 104.0, SOL - 16.0, false);
}

/// La mer de nuages, tout en bas, plus sombre que l'îlot de Pépin : une bande continue
/// (brume) coiffée de dômes qui se chevauchent, liserés de nuage clair, et qui
/// dérivent lentement vers la gauche.
fn mer_de_nuages(anim: u32) {
    let derive = ((anim / 6) % 20) as f32;
    let haut = SOL + 8.0;
    rect(0.0, haut, LARGEUR_LOGIQUE as f32, Y_BANDEAU - haut, BRUME);
    for k in 0..18 {
        let x = k as f32 * 20.0 - derive;
        let y = haut - if k % 2 == 0 { 6.0 } else { 3.0 };
        for (dx, dy, l) in [(0.0, 2.0, 24.0), (4.0, 0.0, 16.0)] {
            rect(x + dx, y + dy, l, haut - y - dy, BRUME);
            rect(x + dx + 1.0, y + dy, l - 2.0, 1.0, NUAGE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pepin_core::font::largeur_texte;

    /// Marge de texte de chaque côté de l'écran.
    const MARGE: u32 = 8;
    use pepin_core::levels::{cyclone, descente, NIVEAUX};

    /// Les lignes du récit et l'invitation tiennent dans le bandeau (8 px de marge
    /// de chaque côté) ; la hauteur est vérifiée à la compilation (`const _`).
    #[test]
    fn l_histoire_tient_dans_son_bandeau() {
        let utile = LARGEUR_LOGIQUE - 2 * MARGE;
        for c in Chapitre::TOUS {
            for l in c.lignes() {
                assert!(largeur_texte(l, 1) <= utile, "« {l} » déborde");
            }
        }
        assert!(largeur_texte(contenu::APPUIE_POUR_CONTINUER, 1) <= utile);
    }

    #[test]
    fn les_chapitres_se_suivent_dans_l_ordre() {
        assert_eq!(Chapitre::Depart.suivant(), Some(Chapitre::Lueur));
        assert_eq!(Chapitre::Lueur.suivant(), Some(Chapitre::Soleil));
        assert_eq!(Chapitre::Soleil.suivant(), None);
        for (i, c) in Chapitre::TOUS.iter().enumerate() {
            assert_eq!(c.indice(), i);
        }
    }

    /// Dans le vrai jeu : l'image 1 avant le premier niveau, la 2 avant le niveau qui
    /// suit le Grand Nuage, la 3 avant la descente ; aucune ailleurs.
    #[test]
    fn les_trois_images_tombent_aux_bons_niveaux() {
        let apres_le_boss = NIVEAUX
            .iter()
            .position(|n| n.nom == cyclone::INFOS.nom)
            .unwrap()
            + 1;
        let descente = NIVEAUX
            .iter()
            .position(|n| n.nom == descente::INFOS.nom)
            .unwrap();
        for i in 0..NIVEAUX.len() {
            let attendu = match i {
                0 => Some(Chapitre::Depart),
                _ if i == apres_le_boss => Some(Chapitre::Lueur),
                _ if i == descente => Some(Chapitre::Soleil),
                _ => None,
            };
            assert_eq!(chapitre_avant(NIVEAUX, i), attendu, "niveau {}", i + 1);
        }
        assert_eq!(chapitre_avant(NIVEAUX, NIVEAUX.len()), None, "hors table");
    }

    /// Sur de fausses tables : sans boss, pas d'image 2 ; sans épilogue, pas d'image 3 ;
    /// un niveau inséré au sommet (avant le bonus) ne déplace pas l'image 3.
    #[test]
    fn la_regle_suit_la_table_et_non_des_numeros() {
        let normal = |theme| InfosNiveau {
            genre: GenreNiveau::Normal,
            theme,
            ..NIVEAUX[0]
        };
        let genre = |genre| InfosNiveau {
            genre,
            ..NIVEAUX[0]
        };
        let sans_boss = [normal(Theme::Toits), normal(Theme::Mer)];
        assert_eq!(chapitre_avant(&sans_boss, 0), Some(Chapitre::Depart));
        assert_eq!(chapitre_avant(&sans_boss, 1), None);
        let complet = [
            normal(Theme::Toits),
            normal(Theme::Cyclone),
            normal(Theme::Altitude),
            normal(Theme::Soleil),
            normal(Theme::Soleil), // un niveau de plus au sommet
            genre(GenreNiveau::Bonus),
            genre(GenreNiveau::Epilogue),
        ];
        let lu: Vec<_> = (0..complet.len())
            .map(|i| chapitre_avant(&complet, i))
            .collect();
        assert_eq!(
            lu,
            [
                Some(Chapitre::Depart),
                None,
                Some(Chapitre::Lueur),
                None,
                None,
                None,
                Some(Chapitre::Soleil)
            ]
        );
    }

    #[test]
    fn l_image_ignore_les_appuis_un_instant() {
        assert!(!prete(0));
        assert!(!prete(DELAI_PAS - 1));
        assert!(prete(DELAI_PAS));
    }
}
