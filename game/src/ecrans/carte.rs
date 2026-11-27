//! Carte de présentation d'un niveau : numéro, nom, phrase d'ambiance et conseil.
//! Elle s'affiche par-dessus le niveau figé, juste après la transition, et attend
//! un appui du joueur (carte 9.31), accepté après le court délai de l'histoire.

use pepin_core::font::largeur_texte;
use pepin_core::lettres::{rang_du_niveau, NB_LETTRES};
use pepin_core::palette::{
    ARDOISE_CLAIRE, ARDOISE_NUIT, ARDOISE_OMBRE, BLANC_CASSE, BRUME, CORAIL_CLAIR, OR_PALE,
};
use pepin_core::vies::coute_des_vies;
use pepin_core::zones::InfosNiveau;
use pepin_core::LARGEUR_LOGIQUE;

use super::{panneau, trame};
use crate::contenu;
use crate::texte::{self, couper_lignes, Style};

/// Valeur portée par `Ecran::Carte` à l'ouverture. Depuis la carte 9.31, la carte ne
/// se passe plus jamais seule : ce nombre ne déclenche plus rien.
pub const DUREE_PAS: u16 = 180;
/// Valeur de `Ecran::Carte` quand le joueur vient de la passer : le jeu commence au
/// pas suivant, sans avancer pendant ce pas-là (comme avant la carte 9.31, quand la
/// carte finissait seule), pour que les parcours du robot QA gardent leur premier pas.
pub const PASSEE: u16 = 0;
/// Largeur maximale d'une ligne de texte, en caractères (320 px / 6 px = 53 ; marge
/// comprise). Elle vient du chargeur des niveaux, qui refuse une intro trop longue
/// pour cette carte (carte 11.3) : une seule valeur pour le jeu, le chargeur et
/// l'éditeur.
const LIGNE_MAX: usize = niveaux::intro::LIGNE_MAX_INTRO;

/// Largeur utile du cadre de la carte, en pixels logiques.
const LARGEUR_CADRE: u32 = LARGEUR_LOGIQUE - 24;

/// Nom du niveau tel qu'il est dessiné : en capitales. La police pose les minuscules
/// 2 rangées plus haut que le pied des majuscules : à l'échelle 3, un titre mixte
/// aurait l'air de flotter. Les capitales accentuées existent dans la police.
pub fn titre_niveau(nom: &str) -> String {
    nom.to_uppercase()
}

/// Plus grande échelle (3, puis 2) à laquelle le nom du niveau tient dans le cadre.
fn echelle_nom(nom: &str) -> u32 {
    if largeur_texte(nom, 3) < LARGEUR_CADRE - 8 {
        3
    } else {
        2
    }
}

/// `infos` : le niveau (sa phrase et son conseil sont dans son fichier, carte 11.3) ;
/// `anim` : pas écoulés
/// (clignotement) ; `lettres` : les lettres cachées déjà trouvées (carte 8.6), en
/// petit en haut à droite, celle de ce niveau en corail tant qu'elle n'est pas trouvée.
pub fn dessiner(infos: &InfosNiveau, anim: u32, lettres: &[bool; NB_LETTRES]) {
    let nom = titre_niveau(infos.nom);
    let nom = nom.as_str();
    trame();
    let l = LARGEUR_LOGIQUE as f32;
    panneau(12.0, 34.0, l - 24.0, 112.0, ARDOISE_NUIT, ARDOISE_CLAIRE);
    let ombre = |c| Style::simple(c, 1).avec_ombre(ARDOISE_OMBRE);

    // « NIVEAU 3/10 » ; hors du Monde 1 (carte 9.1b) : « CHAPITRE 2 · NIVEAU 1/8 ».
    let etiquette = contenu::etiquette(infos);
    texte::dessiner_centre(&etiquette, 44.0, ombre(BRUME));
    super::fin::dessiner_lettres(lettres, rang_du_niveau(infos.cle), l - 20.0, 44.0);
    texte::dessiner_centre(
        nom,
        56.0 + if echelle_nom(nom) == 3 { 0.0 } else { 4.0 },
        Style::simple(OR_PALE, echelle_nom(nom)).avec_ombre(ARDOISE_OMBRE),
    );
    let mut y = 88.0;
    for ligne in couper_lignes(infos.intro_texte, LIGNE_MAX) {
        texte::dessiner_centre(&ligne, y, ombre(BLANC_CASSE));
        y += 10.0;
    }
    y += 4.0;
    for ligne in couper_lignes(infos.intro_astuce, LIGNE_MAX) {
        texte::dessiner_centre(&ligne, y, ombre(CORAIL_CLAIR));
        y += 10.0;
    }
    // Le bonus ne coûte aucune vie (carte 6.2) : la carte le dit.
    if let Some(ligne) = ligne_vies(infos) {
        texte::dessiner_centre(ligne, y, ombre(OR_PALE));
    }
    // Rien tant qu'un appui serait ignoré, puis un clignotement doux (carte 9.31).
    if super::histoire::invite_visible(anim) {
        texte::dessiner_centre(contenu::CONTINUER, 134.0, ombre(BRUME));
    }
}

/// Ligne de plus sur la carte d'un niveau qui ne coûte aucune vie (le bonus), `None`
/// pour les autres.
pub fn ligne_vies(infos: &InfosNiveau) -> Option<&'static str> {
    (!coute_des_vies(infos.genre)).then_some(contenu::SANS_PERTE_DE_VIE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pepin_core::zones::InfosNiveau;

    /// Tous les niveaux : le parcours, puis les niveaux d'essai (build de QA).
    fn tous() -> impl Iterator<Item = &'static InfosNiveau> {
        niveaux::niveaux().iter().chain(niveaux::essais())
    }

    #[test]
    fn toutes_les_cartes_tiennent_dans_leur_cadre() {
        // Les niveaux de la table, puis les niveaux d'essai (build de QA).
        // Le chargeur refuse déjà une intro trop longue (`niveaux::intro`) : ce test
        // vérifie que ses règles correspondent bien à CE dessin, en pixels.
        for n in tous() {
            let a = couper_lignes(n.intro_texte, LIGNE_MAX);
            let b = couper_lignes(n.intro_astuce, LIGNE_MAX);
            assert!(a.len() <= 3, "{} : intro sur {} lignes", n.nom, a.len());
            assert!(b.len() <= 2, "{} : conseil sur {} lignes", n.nom, b.len());
            // Le texte commence en y = 88 ; la dernière ligne (celle des vies du
            // bonus comprise, carte 6.2) doit finir avant « continuer » (134).
            let c = ligne_vies(n).map_or(0, |l| {
                assert!(largeur_texte(l, 1) < LARGEUR_CADRE - 8, "{l}");
                1
            });
            let fin = 88.0 + a.len() as f32 * 10.0 + 4.0 + (b.len() + c) as f32 * 10.0;
            assert!(fin <= 134.0, "{} : texte jusqu'à y = {fin}", n.nom);
            for ligne in a.iter().chain(&b) {
                assert!(largeur_texte(ligne, 1) < LARGEUR_CADRE - 8, "{}", ligne);
            }
            // Le nom, en capitales et à l'échelle choisie, doit tenir dans le cadre.
            let nom = titre_niveau(n.nom);
            assert!(
                largeur_texte(&nom, echelle_nom(&nom)) < LARGEUR_CADRE - 8,
                "{}",
                nom
            );
        }
    }

    #[test]
    fn les_noms_de_niveau_sont_en_capitales_avec_un_glyphe_chacun() {
        for n in tous() {
            let nom = titre_niveau(n.nom);
            assert!(!nom.chars().any(|c| c.is_lowercase()), "{nom}");
            for c in nom.chars() {
                assert!(
                    pepin_core::font::glyphe(c).is_some(),
                    "pas de glyphe pour {c:?} dans « {nom} »"
                );
            }
        }
        assert_eq!(titre_niveau("Le grand orage"), "LE GRAND ORAGE");
        assert_eq!(
            titre_niveau("Gouttières et cheminées"),
            "GOUTTIÈRES ET CHEMINÉES"
        );
    }

    #[test]
    fn aucune_ligne_de_carte_ne_commence_par_une_ponctuation_seule() {
        for n in tous() {
            for texte in [n.intro_texte, n.intro_astuce] {
                for ligne in couper_lignes(texte, LIGNE_MAX) {
                    assert!(
                        !ligne.starts_with([':', '!', '?', ';', '.']),
                        "{} : ligne « {ligne} » commençant par une ponctuation",
                        n.nom
                    );
                    assert!(ligne.chars().count() <= LIGNE_MAX, "{ligne}");
                }
            }
        }
    }

    #[test]
    fn les_coupures_signalees_par_l_audit_sont_corrigees() {
        // L'orage (niveau 4 depuis l'arrivée du marché couvert).
        assert_eq!(
            couper_lignes(niveaux::par_cle("orage").intro_texte, LIGNE_MAX),
            [
                "L'orage éclate. Chaque éclair est annoncé :",
                "garde la tête froide !"
            ]
        );
        // Conseil d'« Au-dessus des nuages » (niveau 9 depuis l'arrivée de la mer de
        // nuages ; ce n'est plus la dernière ligne droite depuis la carte 8.2) : une
        // seule ligne.
        assert_eq!(
            couper_lignes(niveaux::par_cle("nuages").intro_astuce, LIGNE_MAX),
            ["Le champignon t'aide à monter."]
        );
        assert!(!niveaux::par_cle("orage")
            .intro_texte
            .to_lowercase()
            .contains("flash"));
        // Le bonus (carte 5.16) : la phrase d'AGA et le conseil tiennent chacun sur
        // une ligne (une première version laissait « toi. » seul sur la seconde).
        // (11e niveau depuis la carte 8.2.)
        let arc = niveaux::par_cle("arc");
        for texte in [arc.intro_texte, arc.intro_astuce] {
            assert_eq!(couper_lignes(texte, LIGNE_MAX), [texte]);
        }
    }

    /// Seul le bonus dit qu'on n'y perd aucune vie (carte 6.2).
    #[test]
    fn seule_la_carte_du_bonus_parle_des_vies() {
        use pepin_core::levels::GenreNiveau;
        for n in niveaux::niveaux() {
            assert_eq!(
                ligne_vies(n).is_some(),
                n.genre == GenreNiveau::Bonus,
                "{}",
                n.nom
            );
        }
        assert!(niveaux::niveaux().iter().any(|n| ligne_vies(n).is_some()));
    }
}
