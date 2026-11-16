//! Bilan d'une course contre le robot (carte 8.7) : par-dessus le niveau figé, ce que
//! le niveau a rapporté (comme un niveau joué seul : médaille, gouttes, défis), puis
//! « GAGNÉ DE 2,3 s » ou « PERDU DE 0,4 s » (ou « LE ROBOT A ABANDONNÉ » si son rejeu
//! a divergé), les deux temps, le meilleur résultat contre cette force et la prime de
//! la première victoire contre EXPERT ; enfin REJOUER (la même course) ou CARTE.

use pepin_core::font::largeur_texte;
use pepin_core::palette::{
    ARDOISE_CLAIRE, ARDOISE_NUIT, ARDOISE_OMBRE, BLANC_CASSE, BRUME, CORAIL_CLAIR, OR, OR_PALE,
};
use pepin_core::LARGEUR_LOGIQUE;

use super::resultats::{entete, icone_medaille, ligne_defis, ECART_ICONE, L_ICONE};
use super::{dessiner_menu, panneau, trame, Disposition};
use crate::chrono;
use crate::contenu;
use crate::course::{texte_resultat, Issue, ResultatCourse};
use crate::defis::ResultatNiveau;
use crate::menu::Menu;
use crate::texte::{self, Style};

/// Les temps de la course : celui du joueur et celui du fantôme (`None` : abandon).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TempsCourse {
    pub joueur: u32,
    pub robot: Option<u32>,
}

/// Le grand titre du bilan et sa couleur de base (`true` : victoire, il clignote en or).
pub fn titre(c: &ResultatCourse) -> (String, bool) {
    match c.issue.ecart_signe() {
        Some(e) => (texte_resultat(e), e < 0),
        None => (contenu::ROBOT_ABANDON.to_string(), false),
    }
}

/// La ligne du meilleur résultat : « MEILLEUR RÉSULTAT ! » si la course le bat (ou est
/// la première finie), sinon le meilleur d'avant. Rien si le robot a abandonné.
pub fn ligne_meilleur(c: &ResultatCourse) -> Option<(String, bool)> {
    let e = c.issue.ecart_signe()?;
    Some(match c.ancien {
        Some(a) if a <= e => (
            format!("{} {}", contenu::MEILLEUR_RESULTAT, texte_resultat(a)),
            false,
        ),
        _ => (contenu::NOUVEAU_MEILLEUR_RESULTAT.to_string(), true),
    })
}

/// Dessine le bilan (le niveau figé est déjà dessiné dessous).
#[allow(clippy::too_many_arguments)]
pub fn dessiner(
    temps: &TempsCourse,
    c: &ResultatCourse,
    r: &ResultatNiveau,
    anim: u32,
    libelles: &[&str],
    menu: &Menu,
    d: &Disposition,
) {
    trame();
    let l = LARGEUR_LOGIQUE as f32;
    panneau(12.0, 20.0, l - 24.0, 150.0, ARDOISE_NUIT, ARDOISE_CLAIRE);
    let ombre = |c, e| Style::simple(c, e).avec_ombre(ARDOISE_OMBRE);
    // Ce que le niveau a rapporté, médaille devant (comme le bilan d'un niveau seul).
    let ligne = entete(r);
    let x = ((l - largeur_texte(&ligne, 1) as f32 - L_ICONE - ECART_ICONE) / 2.0).round();
    icone_medaille(x, 26.0, r.medaille);
    texte::dessiner(&ligne, x + L_ICONE + ECART_ICONE, 26.0, ombre(OR_PALE, 1));
    // GAGNÉ / PERDU et l'écart.
    let clignote = (anim / 20).is_multiple_of(2);
    let (t, gagne) = titre(c);
    let couleur = match (gagne, clignote, c.issue) {
        (true, true, _) => OR,
        (true, false, _) => OR_PALE,
        (false, _, Issue::Abandon) => BRUME,
        (false, _, _) => CORAIL_CLAIR,
    };
    texte::dessiner_centre(&t, 42.0, ombre(couleur, 2));
    // Les deux temps.
    let robot = temps
        .robot
        .map_or_else(|| "-".to_string(), chrono::temps_precis);
    let ligne = format!(
        "{} {}   {} {} {}",
        contenu::TOI,
        chrono::temps_precis(temps.joueur),
        contenu::ROBOT,
        c.force.libelle(),
        robot
    );
    texte::dessiner_centre(&ligne, 64.0, ombre(BLANC_CASSE, 1));
    if let Some((m, mieux)) = ligne_meilleur(c) {
        texte::dessiner_centre(&m, 78.0, ombre(if mieux { OR_PALE } else { BRUME }, 1));
    }
    if c.prime > 0 {
        let p = contenu::prime_gagnee(c.prime);
        texte::dessiner_centre(&p, 92.0, ombre(if clignote { OR } else { OR_PALE }, 1));
    }
    ligne_defis(r, 108.0, false);
    dessiner_menu(libelles, menu, d, anim);
}
