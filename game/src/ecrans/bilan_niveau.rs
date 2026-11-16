//! Bilan d'un niveau joué seul depuis la carte des niveaux (carte 7.4) : par-dessus le
//! niveau figé, le temps réalisé comparé au meilleur temps d'avant (« -1,8 s !
//! NOUVEAU MEILLEUR TEMPS » ou « +3,2 s »), les défis réussis, la médaille et le gain,
//! puis le choix REJOUER (le même niveau, tout de suite) ou CARTE.

use pepin_core::font::largeur_texte;
use pepin_core::palette::{
    ARDOISE_CLAIRE, ARDOISE_NUIT, ARDOISE_OMBRE, BLANC_CASSE, CORAIL_CLAIR, OR, OR_PALE,
};
use pepin_core::LARGEUR_LOGIQUE;

use super::carte::titre_niveau;
use super::resultats::{entete, icone_medaille, ligne_defis, ECART_ICONE, L_ICONE};
use super::{dessiner_menu, panneau, trame, Disposition};
use crate::chrono;
use crate::contenu;
use crate::defis::ResultatNiveau;
use crate::menu::Menu;
use crate::texte::{self, Style};

/// Le temps de la tentative réussie et le meilleur temps du niveau AVANT elle
/// (`None` : le niveau n'avait jamais été fini).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Temps {
    pub pas: u32,
    pub ancien: Option<u32>,
}

impl Temps {
    /// La ligne sous le temps et sa couleur : écart au meilleur temps d'avant.
    pub fn ligne_ecart(&self) -> (String, bool) {
        match self.ancien {
            None => (contenu::PREMIER_TEMPS.to_string(), true),
            Some(a) if self.pas < a => (
                format!(
                    "{} ! {}",
                    chrono::ecart(self.pas, a),
                    contenu::NOUVEAU_MEILLEUR_TEMPS
                ),
                true,
            ),
            Some(a) => (
                format!(
                    "{}   {} {}",
                    chrono::ecart(self.pas, a),
                    contenu::LIBELLE_MEILLEUR,
                    chrono::temps_precis(a)
                ),
                false,
            ),
        }
    }
}

/// Dessine le bilan (le niveau figé est déjà dessiné dessous).
pub fn dessiner(
    temps: &Temps,
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
    // « NIVEAU RÉUSSI : ARGENT, +9 GOUTTES », médaille devant.
    let titre = entete(r);
    let x = ((l - largeur_texte(&titre, 1) as f32 - L_ICONE - ECART_ICONE) / 2.0).round();
    icone_medaille(x, 26.0, r.medaille);
    texte::dessiner(&titre, x + L_ICONE + ECART_ICONE, 26.0, ombre(OR_PALE, 1));
    let nom = titre_niveau(r.nom);
    texte::dessiner_centre(&nom, 40.0, ombre(BLANC_CASSE, 2));
    let t = format!(
        "{} {}",
        contenu::LIBELLE_TEMPS,
        chrono::temps_precis(temps.pas)
    );
    texte::dessiner_centre(&t, 62.0, ombre(BLANC_CASSE, 2));
    // L'écart clignote doucement quand c'est un nouveau meilleur temps.
    let (ecart, mieux) = temps.ligne_ecart();
    let couleur = match (mieux, (anim / 20).is_multiple_of(2)) {
        (true, true) => OR,
        (true, false) => OR_PALE,
        (false, _) => CORAIL_CLAIR,
    };
    texte::dessiner_centre(&ecart, 82.0, ombre(couleur, 1));
    ligne_defis(r, 100.0, false);
    dessiner_menu(libelles, menu, d, anim);
}
