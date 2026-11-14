//! Écran de fin : le bilan de la partie et le choix de rejouer.
//!
//! Deux décors (carte 5.17) : quand la partie se termine par l'épilogue (« La
//! descente », le retour sur le toit du premier niveau), « DE RETOUR ! » sur la nuit
//! tranquille de la ville, et « Pépin replie son parapluie. » ; sinon (un jeu sans
//! épilogue), « LE SOLEIL ! » sur le ciel doré, comme avant. Le compteur de gouttes
//! est celui du PARCOURS (tous les niveaux sauf le bonus) ; le bonus, s'il a été
//! joué, a sa propre ligne.
//!
//! La mise en page est calculée par une fonction pure (`mise_en_page`), testée sans
//! fenêtre ; `dessiner` ne fait que la suivre.

use pepin_core::levels::{self, GenreNiveau, NIVEAUX};
use pepin_core::palette::{
    ARDOISE_CLAIRE, ARDOISE_NUIT, ARDOISE_OMBRE, BLANC_CASSE, BRUME, CORAIL_CLAIR, OR, OR_PALE,
    PLUIE,
};
use pepin_core::zones::Theme;

use super::{dessiner_menu, panneau, Disposition};
use crate::chrono;
use crate::contenu::{self, BonusFin};
use crate::decor;
use crate::menu::Menu;
use crate::particules::Particules;
use crate::texte::{self, couper_lignes, Style};

/// Largeur maximale (en caractères) du petit mot de fin : assez large pour que
/// « Il reste plein de gouttes d'or à dénicher. » tienne sur une seule ligne.
const LIGNE_MAX_MOT: usize = 42;

/// Ordonnée de la phrase sous le titre (épilogue) : sous l'ombre du titre (échelle
/// 3, de 10 à 34), au-dessus de la première ligne du bilan (46).
const Y_PHRASE: f32 = 35.0;

/// Les chiffres de la partie terminée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bilan {
    /// Durée en pas de simulation (1/60 s).
    pub pas: u32,
    /// Gouttes d'or du PARCOURS (tous les niveaux sauf le bonus).
    pub gouttes: u32,
    pub morts: u32,
    pub score: u32,
    /// Gouttes ramassées dans le bonus, s'il a été joué (`None` : pas joué).
    pub gouttes_bonus: Option<u32>,
    /// La partie s'est-elle terminée par l'épilogue (le retour sur les toits) ?
    pub epilogue: bool,
}

/// Ce que montre l'écran de fin et où : calculé sans rien dessiner.
#[derive(Debug, Clone, PartialEq)]
pub struct MiseEnPage {
    /// Le décor du fond.
    pub fond: Theme,
    pub titre: &'static str,
    /// Phrase sous le titre (l'épilogue seulement).
    pub phrase: Option<&'static str>,
    /// Lignes du bilan : libellé, valeur.
    pub lignes: Vec<(&'static str, String)>,
    /// Ordonnée de la première ligne du bilan, et écart d'une ligne à la suivante.
    pub y_lignes: f32,
    pub pas_lignes: f32,
    /// Le petit mot de fin, coupé en lignes, et l'ordonnée de sa première ligne.
    pub mot: Vec<String>,
    pub y_mot: f32,
}

/// Le bonus du jeu, vu de l'écran de fin.
fn bonus_fin(bilan: &Bilan) -> BonusFin {
    let total = levels::gouttes_du_bonus(NIVEAUX);
    match bilan.gouttes_bonus {
        Some(gouttes) => BonusFin::Joue { gouttes, total },
        None if NIVEAUX.iter().any(|n| n.genre == GenreNiveau::Bonus) => BonusFin::Manque,
        None => BonusFin::Absent,
    }
}

/// Mise en page de l'écran de fin pour `bilan`. Sans épilogue ni bonus, c'est
/// exactement celle d'avant la carte 5.17 (lignes en 42, tous les 13 px ; mot en
/// 98). L'épilogue ajoute une phrase sous le titre, le bonus une ligne : les lignes
/// se resserrent (tous les 11 px) pour laisser le menu à sa place.
pub fn mise_en_page(bilan: &Bilan) -> MiseEnPage {
    let total = levels::gouttes_du_parcours(NIVEAUX);
    let mut lignes = vec![
        (contenu::LIBELLE_TEMPS, chrono::temps_precis(bilan.pas)),
        (
            contenu::LIBELLE_GOUTTES,
            format!("{}/{}", bilan.gouttes, total),
        ),
    ];
    if let Some(b) = bilan.gouttes_bonus {
        let total_bonus = levels::gouttes_du_bonus(NIVEAUX);
        lignes.push((contenu::LIBELLE_BONUS, format!("{b}/{total_bonus}")));
    }
    lignes.push((contenu::LIBELLE_MORTS, bilan.morts.to_string()));
    lignes.push((contenu::LIBELLE_SCORE, bilan.score.to_string()));
    let serre = bilan.epilogue || bilan.gouttes_bonus.is_some();
    let (y_lignes, pas_lignes, ecart_mot) = if serre {
        (46.0, 11.0, 3.0)
    } else {
        (42.0, 13.0, 4.0)
    };
    let mot = couper_lignes(
        contenu::message_fin(bilan.gouttes, total, bonus_fin(bilan)),
        LIGNE_MAX_MOT,
    );
    MiseEnPage {
        fond: if bilan.epilogue {
            Theme::Descente
        } else {
            Theme::Soleil
        },
        titre: if bilan.epilogue {
            contenu::TITRE_FIN_RETOUR
        } else {
            contenu::TITRE_FIN
        },
        phrase: bilan.epilogue.then_some(contenu::PHRASE_RETOUR),
        y_mot: y_lignes + lignes.len() as f32 * pas_lignes + ecart_mot,
        lignes,
        y_lignes,
        pas_lignes,
        mot,
    }
}

/// Ajoute de temps en temps une gerbe d'étincelles dans les marges (sans hasard système).
pub fn confettis(particules: &mut Particules, anim: u32) {
    if !anim.is_multiple_of(9) {
        return;
    }
    let h = anim.wrapping_mul(2_654_435_761) >> 8;
    let x = if h & 1 == 0 { 4 + h % 30 } else { 286 + h % 30 };
    let y = 20 + (h >> 4) % 130;
    let couleur = [OR, CORAIL_CLAIR, PLUIE, OR_PALE][(h >> 12) as usize % 4];
    particules.etincelles(x as f32, y as f32, 5, couleur);
}

pub fn dessiner(
    bilan: &Bilan,
    anim: u32,
    particules: &Particules,
    libelles: &[&str],
    menu: &Menu,
    d: &Disposition,
) {
    let m = mise_en_page(bilan);
    // Le fond tel qu'en bas d'un niveau (caméra du fond à 0) : pour l'épilogue, la
    // nuit de la ville du premier niveau, là où Pépin vient d'arriver.
    decor::dessiner_fond(m.fond, (anim / 2) as i32, 0, anim);
    panneau(20.0, 4.0, 280.0, 166.0, ARDOISE_NUIT, ARDOISE_CLAIRE);
    let ombre = |c| Style::simple(c, 1).avec_ombre(ARDOISE_OMBRE);
    texte::dessiner_centre(
        m.titre,
        10.0,
        Style::simple(OR_PALE, 3).avec_ombre(ARDOISE_OMBRE),
    );
    if let Some(phrase) = m.phrase {
        texte::dessiner_centre(phrase, Y_PHRASE, ombre(BLANC_CASSE));
    }
    for (i, (nom, valeur)) in m.lignes.iter().enumerate() {
        let y = m.y_lignes + i as f32 * m.pas_lignes;
        texte::dessiner(nom, 56.0, y, ombre(BRUME));
        texte::dessiner_a_droite(valeur, 264.0, y, ombre(BLANC_CASSE));
    }
    for (i, ligne) in m.mot.iter().enumerate() {
        texte::dessiner_centre(ligne, m.y_mot + i as f32 * 10.0, ombre(CORAIL_CLAIR));
    }
    dessiner_menu(libelles, menu, d, anim);
    particules.dessiner(0, 0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecrans::{disposition, Ecran};
    use pepin_core::font::HAUTEUR_GLYPHE;

    /// Tous les bilans qui comptent : avec ou sans épilogue, bonus joué ou non, peu
    /// ou toutes les gouttes.
    fn bilans() -> Vec<Bilan> {
        let mut v = Vec::new();
        for epilogue in [false, true] {
            for gouttes_bonus in [None, Some(0), Some(3)] {
                for gouttes in [0, 15, 26, 30] {
                    v.push(Bilan {
                        pas: 3600,
                        gouttes,
                        morts: 2,
                        score: 900,
                        gouttes_bonus,
                        epilogue,
                    });
                }
            }
        }
        v
    }

    #[test]
    fn le_mot_de_fin_tient_sur_deux_lignes_sans_coupure_maladroite() {
        for mot in contenu::tests::tous_les_mots_de_fin() {
            let lignes = couper_lignes(mot, LIGNE_MAX_MOT);
            for l in &lignes {
                assert!(!l.starts_with([':', '!', '?', ';', '.']), "{l}");
            }
            assert!(lignes.len() <= 2, "{mot} : {lignes:?}");
        }
        let plein = couper_lignes(contenu::message_fin(0, 15, BonusFin::Absent), LIGNE_MAX_MOT);
        assert_eq!(plein, ["Il reste plein de gouttes d'or à dénicher."]);
        // Le bonus manqué ou trouvé (carte 5.17) : chaque phrase sur sa ligne.
        let coupe = |gouttes, bonus| couper_lignes(contenu::message_fin(gouttes, 30, bonus), 42);
        assert_eq!(
            coupe(3, BonusFin::Manque),
            [
                "Il reste plein de gouttes d'or à dénicher.",
                "Toutes ouvrent un niveau secret."
            ]
        );
        assert_eq!(
            coupe(20, BonusFin::Manque),
            [
                "Pas mal ! Il reste quelques gouttes d'or.",
                "Toutes ouvrent un niveau secret."
            ]
        );
        assert_eq!(
            coupe(
                30,
                BonusFin::Joue {
                    gouttes: 1,
                    total: 3
                }
            ),
            [
                "Bravo, tu as trouvé le niveau secret !",
                "Quelques gouttes d'or manquent encore."
            ]
        );
    }

    /// Sans épilogue ni bonus, l'écran de fin est celui d'avant la carte 5.17 : le
    /// soleil, « LE SOLEIL ! », quatre lignes en 42, 55, 68, 81, le mot en 98.
    #[test]
    fn sans_epilogue_l_ecran_de_fin_reste_le_soleil() {
        let m = mise_en_page(&Bilan {
            pas: 3600,
            gouttes: 9,
            morts: 2,
            score: 900,
            gouttes_bonus: None,
            epilogue: false,
        });
        assert_eq!(
            (m.fond, m.titre, m.phrase),
            (Theme::Soleil, "LE SOLEIL !", None)
        );
        let noms: Vec<&str> = m.lignes.iter().map(|l| l.0).collect();
        assert_eq!(noms, ["TEMPS", "GOUTTES D'OR", "MORTS", "SCORE"]);
        assert_eq!((m.y_lignes, m.pas_lignes, m.y_mot), (42.0, 13.0, 98.0));
    }

    /// Après l'épilogue : « DE RETOUR ! », la phrase du retour, et le fond de nuit
    /// de la descente (le ciel du premier niveau, en bas du voyage), sans pluie.
    #[test]
    fn apres_l_epilogue_pepin_est_de_retour_la_nuit() {
        let m = mise_en_page(&Bilan {
            gouttes: 30,
            epilogue: true,
            ..Bilan::default()
        });
        assert_eq!(m.fond, Theme::Descente);
        assert_eq!(m.titre, "DE RETOUR !");
        assert_eq!(m.phrase, Some("Pépin replie son parapluie."));
        assert_eq!(m.lignes[1], ("GOUTTES D'OR", "30/30".to_string()));
        assert!(!Theme::Descente.pluie());
    }

    /// Le compteur : les gouttes du parcours sur 30 (les neuf niveaux normaux et
    /// l'épilogue) ; le bonus, joué, sur sa propre ligne (x/3) ; pas joué, pas de
    /// ligne, et le mot annonce le niveau secret.
    #[test]
    fn les_gouttes_du_parcours_et_du_bonus_sont_comptees_a_part() {
        let joue = mise_en_page(&Bilan {
            gouttes: 30,
            gouttes_bonus: Some(2),
            epilogue: true,
            ..Bilan::default()
        });
        let noms: Vec<&str> = joue.lignes.iter().map(|l| l.0).collect();
        assert_eq!(
            noms,
            ["TEMPS", "GOUTTES D'OR", "NIVEAU SECRET", "MORTS", "SCORE"]
        );
        assert_eq!(joue.lignes[1].1, "30/30");
        assert_eq!(joue.lignes[2].1, "2/3");
        let manque = mise_en_page(&Bilan {
            gouttes: 20,
            epilogue: true,
            ..Bilan::default()
        });
        assert_eq!(manque.lignes.len(), 4);
        assert_eq!(manque.lignes[1].1, "20/30");
        let mot = manque.mot.join(" ");
        assert!(mot.contains("niveau secret"), "{mot}");
    }

    /// Dans tous les cas, rien ne se chevauche : la phrase sous l'ombre du titre, les
    /// lignes sous la phrase, le mot sous les lignes, le menu sous le mot.
    #[test]
    fn rien_ne_se_chevauche_sur_l_ecran_de_fin() {
        let ligne = HAUTEUR_GLYPHE as f32 + 1.0; // hauteur d'une ligne, ombre comprise
        let haut_du_menu = disposition(Ecran::Fin).unwrap().y0;
        let bas_titre = 10.0 + 3.0 * (HAUTEUR_GLYPHE as f32 + 1.0);
        for b in bilans() {
            let m = mise_en_page(&b);
            let mut bas = bas_titre;
            if m.phrase.is_some() {
                assert!(Y_PHRASE >= bas_titre, "{b:?}");
                bas = Y_PHRASE + ligne;
            }
            assert!(m.y_lignes >= bas, "{b:?}");
            assert!(m.pas_lignes >= ligne + 2.0, "{b:?}");
            let fin_lignes = m.y_lignes + (m.lignes.len() - 1) as f32 * m.pas_lignes + ligne;
            assert!(m.y_mot >= fin_lignes, "{b:?}");
            let fin_mot = m.y_mot + (m.mot.len() - 1) as f32 * 10.0 + ligne;
            assert!(fin_mot <= haut_du_menu, "{b:?} : mot jusqu'à {fin_mot}");
        }
    }
}
