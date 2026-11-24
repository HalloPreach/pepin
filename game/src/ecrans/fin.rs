//! Écran de fin : le bilan de la partie et le choix de rejouer.
//!
//! Deux décors (carte 5.17) : quand la partie se termine par l'épilogue (« La
//! descente », le retour sur le toit du premier niveau), « DE RETOUR ! » sur la nuit
//! tranquille de la ville, et « Pépin replie son parapluie. » ; sinon (un jeu sans
//! épilogue), « LE SOLEIL ! » sur le ciel doré, comme avant. Le compteur de gouttes
//! est celui du PARCOURS (tous les niveaux sauf le bonus) ; le bonus, s'il a été
//! joué, a sa propre ligne.
//!
//! Carte 6.2 : une ligne VIES (vies restantes), seulement quand elle tient sans
//! serrer le mot de fin contre le menu (`ligne_vies_tient`).
//!
//! La mise en page est calculée par une fonction pure (`mise_en_page`), testée sans
//! fenêtre ; `dessiner` ne fait que la suivre.

use pepin_core::lettres::{self, NB_LETTRES};
use pepin_core::levels::{self, GenreNiveau};
use pepin_core::palette::{
    ARDOISE_CLAIRE, ARDOISE_NUIT, ARDOISE_OMBRE, ARDOISE_PALE, BLANC_CASSE, BRUME, CORAIL_CLAIR,
    OR, OR_PALE, PLUIE,
};
use pepin_core::zones::{InfosNiveau, Theme};

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
    /// Vies restantes à la fin de la partie (carte 6.2).
    pub vies: u8,
    pub score: u32,
    /// Gouttes ramassées dans le bonus, s'il a été joué (`None` : pas joué).
    pub gouttes_bonus: Option<u32>,
    /// La partie s'est-elle terminée par l'épilogue (le retour sur les toits) ?
    pub epilogue: bool,
    /// Les lettres cachées trouvées (carte 8.6, toutes parties confondues), dans
    /// l'ordre du mot ; `None` : pas de ligne (un bilan sans lettres, tests).
    pub lettres: Option<[bool; NB_LETTRES]>,
    /// Le monde de la partie (carte 9.1) : les totaux de gouttes sont ceux de CE monde.
    /// 0 (la valeur par défaut des tests) compte comme le Monde 1.
    pub monde: u8,
}

/// Les niveaux du monde de la partie (carte 9.1) : avec le seul Monde 1, toute la table.
fn niveaux_de_la_partie(bilan: &Bilan) -> &'static [InfosNiveau] {
    levels::niveaux_du_monde(niveaux::niveaux(), bilan.monde.max(1))
}

impl Bilan {
    /// Les cinq lettres sont réunies : la fin secrète (carte 8.6).
    pub fn fin_secrete(&self) -> bool {
        self.lettres.is_some_and(|l| lettres::manquantes(&l) == 0)
    }
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
    /// La ligne des lettres (carte 8.6), si le bilan en a : son libellé
    /// (« LETTRES » ou « IL MANQUE N LETTRES ») et les lettres trouvées. Elle est
    /// dans `lignes` sous le nom `contenu::LIBELLE_LETTRES` (sa place), valeur vide.
    pub lettres: Option<(String, [bool; NB_LETTRES])>,
}

/// Le bonus du jeu, vu de l'écran de fin.
fn bonus_fin(bilan: &Bilan) -> BonusFin {
    let niveaux = niveaux_de_la_partie(bilan);
    let total = levels::gouttes_du_bonus(niveaux);
    match bilan.gouttes_bonus {
        Some(gouttes) => BonusFin::Joue { gouttes, total },
        None if niveaux.iter().any(|n| n.genre == GenreNiveau::Bonus) => BonusFin::Manque,
        None => BonusFin::Absent,
    }
}

/// Bas du mot de fin si l'écran a `n` lignes de bilan et un mot de `lignes_mot`
/// lignes (ombre comprise), pour la mise en page `(y_lignes, pas_lignes, ecart_mot)`.
fn bas_du_mot((y, pas, ecart): (f32, f32, f32), n: usize, lignes_mot: usize) -> f32 {
    let ligne = pepin_core::font::HAUTEUR_GLYPHE as f32 + 1.0;
    y + n as f32 * pas + ecart + lignes_mot.saturating_sub(1) as f32 * 10.0 + ligne
}

/// La ligne VIES tient-elle ? Oui si, avec elle, le mot de fin finit au moins
/// `MARGE_MENU` px au-dessus du menu (sinon elle n'est pas affichée : le bilan garde
/// ses lignes d'avant la carte 6.2). Pure.
fn ligne_vies_tient(disposition: (f32, f32, f32), n_sans: usize, lignes_mot: usize) -> bool {
    bas_du_mot(disposition, n_sans + 1, lignes_mot) + MARGE_MENU <= Y_MENU
}

/// Haut du menu de l'écran de fin (`ecrans::disposition(Ecran::Fin)`), et marge
/// gardée au-dessus de lui par la ligne VIES.
const Y_MENU: f32 = 124.0;
const MARGE_MENU: f32 = 2.0;
/// Écart minimal entre deux lignes du bilan : 7 px de glyphe, 1 d'ombre, 2 de vide.
const PAS_LIGNES_MIN: f32 = 10.0;

/// Mise en page de l'écran de fin pour `bilan`. Sans épilogue ni bonus, c'est
/// exactement celle d'avant la carte 5.17 (lignes en 42, tous les 13 px ; mot en
/// 98). L'épilogue ajoute une phrase sous le titre, le bonus une ligne : les lignes
/// se resserrent (tous les 11 px) pour laisser le menu à sa place. La ligne VIES
/// (carte 6.2) s'ajoute après MORTS quand elle tient (`ligne_vies_tient`). La ligne
/// des lettres (carte 8.6) passe avant elle : si le bilan ne tient plus, les lignes
/// se resserrent d'un pixel à la fois (jusqu'à 10 px), puis le bloc remonte de 2 px
/// avec 2 px seulement avant le mot. Les cinq lettres réunies : « FIN SECRÈTE ».
pub fn mise_en_page(bilan: &Bilan) -> MiseEnPage {
    let total = levels::gouttes_du_parcours(niveaux_de_la_partie(bilan));
    let mut lignes = vec![
        (contenu::LIBELLE_TEMPS, chrono::temps_precis(bilan.pas)),
        (
            contenu::LIBELLE_GOUTTES,
            format!("{}/{}", bilan.gouttes, total),
        ),
    ];
    if let Some(b) = bilan.gouttes_bonus {
        let total_bonus = levels::gouttes_du_bonus(niveaux_de_la_partie(bilan));
        lignes.push((contenu::LIBELLE_BONUS, format!("{b}/{total_bonus}")));
    }
    // Les lettres cachées : leur place dans les lignes, dessinées à part (`lettres`).
    let lettres = bilan
        .lettres
        .map(|l| (contenu::libelle_lettres(lettres::manquantes(&l)), l));
    if lettres.is_some() {
        lignes.push((contenu::LIBELLE_LETTRES, String::new()));
    }
    lignes.push((contenu::LIBELLE_MORTS, bilan.morts.to_string()));
    lignes.push((contenu::LIBELLE_SCORE, bilan.score.to_string()));
    let serre = bilan.epilogue || bilan.gouttes_bonus.is_some();
    let (mut y_lignes, mut pas_lignes, mut ecart_mot) = if serre {
        (46.0, 11.0, 3.0)
    } else {
        (42.0, 13.0, 4.0)
    };
    let mot = couper_lignes(
        contenu::message_fin(bilan.gouttes, total, bonus_fin(bilan)),
        LIGNE_MAX_MOT,
    );
    // Le bilan et le mot tiennent-ils au-dessus du menu ? (Toujours sans la ligne
    // des lettres : rien ne bouge alors.)
    let bilan_tient = |y: f32, pas: f32, ecart: f32| {
        bas_du_mot((y, pas, ecart), lignes.len(), mot.len()) <= Y_MENU
    };
    while pas_lignes > PAS_LIGNES_MIN && !bilan_tient(y_lignes, pas_lignes, ecart_mot) {
        pas_lignes -= 1.0;
    }
    if !bilan_tient(y_lignes, pas_lignes, ecart_mot) {
        y_lignes -= 2.0;
        ecart_mot = 2.0;
    }
    // La ligne VIES : à l'écart habituel si elle tient, sinon avec des lignes
    // resserrées d'un pixel (jamais moins de 10 px : 2 px entre deux lignes), sinon
    // pas du tout.
    let tient = |pas: f32| {
        pas >= PAS_LIGNES_MIN
            && ligne_vies_tient((y_lignes, pas, ecart_mot), lignes.len(), mot.len())
    };
    let pas_avec_vies = [pas_lignes, pas_lignes - 1.0]
        .into_iter()
        .find(|&p| tient(p));
    if let Some(pas) = pas_avec_vies {
        pas_lignes = pas;
        // Juste avant SCORE, la dernière ligne.
        let i = lignes.len() - 1;
        lignes.insert(i, (contenu::LIBELLE_VIES, bilan.vies.to_string()));
    }
    let secrete = bilan.fin_secrete();
    MiseEnPage {
        fond: if bilan.epilogue && !secrete {
            Theme::Descente
        } else {
            Theme::Soleil
        },
        titre: if secrete {
            contenu::TITRE_FIN_SECRETE
        } else if bilan.epilogue {
            contenu::TITRE_FIN_RETOUR
        } else {
            contenu::TITRE_FIN
        },
        // La phrase sous le titre : après l'épilogue seulement (les lignes sont plus bas).
        phrase: bilan.epilogue.then_some(if secrete {
            contenu::PHRASE_SECRETE
        } else {
            contenu::PHRASE_RETOUR
        }),
        y_mot: y_lignes + lignes.len() as f32 * pas_lignes + ecart_mot,
        lignes,
        y_lignes,
        pas_lignes,
        mot,
        lettres,
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
    nouveau_record: bool,
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
        // La ligne des lettres (carte 8.6) : « IL MANQUE N LETTRES », puis le mot
        // PÉPIN, lettres trouvées en or, les autres grisées.
        if *nom == contenu::LIBELLE_LETTRES {
            if let Some((libelle, trouvees)) = &m.lettres {
                texte::dessiner(libelle, 56.0, y, ombre(BRUME));
                dessiner_lettres(trouvees, None, 264.0, y);
            }
            continue;
        }
        texte::dessiner(nom, 56.0, y, ombre(BRUME));
        texte::dessiner_a_droite(valeur, 264.0, y, ombre(BLANC_CASSE));
        // Record battu (carte 7.2) : au milieu de la ligne SCORE, qui clignote.
        if nouveau_record && *nom == contenu::LIBELLE_SCORE && (anim / 30) % 4 != 3 {
            texte::dessiner_centre(contenu::NOUVEAU_RECORD, y, ombre(OR));
        }
    }
    for (i, ligne) in m.mot.iter().enumerate() {
        texte::dessiner_centre(ligne, m.y_mot + i as f32 * 10.0, ombre(CORAIL_CLAIR));
    }
    dessiner_menu(libelles, menu, d, anim);
    particules.dessiner(0, 0);
}

/// Pas d'une lettre à l'autre dans le petit mot « PÉPIN » (glyphe de 5 px + 2).
const PAS_LETTRE: f32 = 7.0;
/// Largeur du petit mot « PÉPIN » dessiné par `dessiner_lettres`.
pub const LARGEUR_LETTRES: f32 = PAS_LETTRE * NB_LETTRES as f32 - 2.0;

/// Le petit mot « PÉPIN » (carte 8.6), son bord droit en `droite` : les lettres
/// trouvées en or, les autres en ardoise pâle. `ici` : la lettre du niveau présenté
/// (rang) ; pas encore trouvée, elle est en corail (« elle est cachée ici »). Sert à
/// l'écran de fin et à la carte de présentation d'un niveau.
pub fn dessiner_lettres(trouvees: &[bool; NB_LETTRES], ici: Option<usize>, droite: f32, y: f32) {
    let x0 = droite - LARGEUR_LETTRES;
    for (i, &(_, lettre)) in lettres::LETTRES.iter().enumerate() {
        let x = x0 + i as f32 * PAS_LETTRE;
        let couleur = if trouvees[i] {
            OR
        } else if ici == Some(i) {
            CORAIL_CLAIR
        } else {
            ARDOISE_PALE
        };
        let style = Style::simple(couleur, 1).avec_ombre(ARDOISE_OMBRE);
        texte::dessiner(&lettre.to_string(), x, y, style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecrans::{disposition, Ecran};
    use pepin_core::font::HAUTEUR_GLYPHE;

    /// Tous les bilans qui comptent : avec ou sans épilogue, bonus joué ou non, peu
    /// ou toutes les gouttes, sans ligne des lettres, avec 2 manquantes ou toutes.
    fn bilans() -> Vec<Bilan> {
        let mut v = Vec::new();
        for epilogue in [false, true] {
            for gouttes_bonus in [None, Some(0), Some(3)] {
                for gouttes in [0, 15, 29, 33] {
                    for lettres in [
                        None,
                        Some([true, false, true, true, false]),
                        Some([true; 5]),
                    ] {
                        v.push(Bilan {
                            monde: 1,
                            pas: 3600,
                            gouttes,
                            morts: 2,
                            vies: 9,
                            score: 900,
                            gouttes_bonus,
                            epilogue,
                            lettres,
                        });
                    }
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
        let tout = BonusFin::Joue {
            gouttes: 3,
            total: 3,
        };
        let complet = couper_lignes(contenu::message_fin(30, 30, tout), LIGNE_MAX_MOT);
        assert_eq!(complet, [contenu::COLLECTION_COMPLETE], "sur une ligne");
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
    /// soleil, « LE SOLEIL ! », quatre lignes en 42, 55, 68, 81, le mot en 98 (sur
    /// deux lignes : la ligne VIES ne tient pas, elle n'est pas affichée).
    #[test]
    fn sans_epilogue_l_ecran_de_fin_reste_le_soleil() {
        let m = mise_en_page(&Bilan {
            monde: 1,
            pas: 3600,
            gouttes: 9,
            morts: 2,
            vies: 5,
            score: 900,
            gouttes_bonus: None,
            epilogue: false,
            lettres: None,
        });
        assert_eq!(m.mot.len(), 2);
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
            monde: 1,
            gouttes: 33,
            epilogue: true,
            ..Bilan::default()
        });
        assert_eq!(m.fond, Theme::Descente);
        assert_eq!(m.titre, "DE RETOUR !");
        assert_eq!(m.phrase, Some(contenu::PHRASE_RETOUR));
        assert_eq!(m.lignes[1], ("GOUTTES D'OR", "33/33".to_string()));
        assert!(!Theme::Descente.pluie());
    }

    /// Le compteur : les gouttes du parcours sur 33 (les dix niveaux normaux et
    /// l'épilogue, carte 8.2) ; le bonus, joué, sur sa propre ligne (x/3) ; pas joué, pas de
    /// ligne, et le mot annonce le niveau secret.
    #[test]
    fn les_gouttes_du_parcours_et_du_bonus_sont_comptees_a_part() {
        let joue = mise_en_page(&Bilan {
            monde: 1,
            gouttes: 33,
            gouttes_bonus: Some(2),
            epilogue: true,
            ..Bilan::default()
        });
        let noms: Vec<&str> = joue.lignes.iter().map(|l| l.0).collect();
        assert_eq!(
            noms,
            ["TEMPS", "GOUTTES D'OR", "NIVEAU SECRET", "MORTS", "SCORE"]
        );
        assert_eq!(joue.lignes[1].1, "33/33");
        assert_eq!(joue.lignes[2].1, "2/3");
        let manque = mise_en_page(&Bilan {
            monde: 1,
            gouttes: 20,
            epilogue: true,
            ..Bilan::default()
        });
        assert_eq!(manque.lignes.len(), 5, "VIES comprise");
        assert_eq!(manque.lignes[1].1, "20/33");
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
        assert_eq!(
            Y_MENU, haut_du_menu,
            "la ligne VIES compte avec le vrai menu"
        );
    }

    /// La ligne VIES (carte 6.2) : après MORTS, avant SCORE, dès qu'elle tient avec
    /// 2 px de marge au-dessus du menu (lignes resserrées d'un pixel s'il le faut).
    /// Partie normale (épilogue, bonus manqué ou complet) : elle y est ; bonus joué
    /// mais incomplet (mot sur deux lignes, cinq lignes de bilan) : elle n'y est pas.
    #[test]
    fn les_vies_restantes_s_affichent_quand_elles_tiennent() {
        let noms =
            |b: &Bilan| -> Vec<&str> { mise_en_page(b).lignes.iter().map(|l| l.0).collect() };
        let base = Bilan {
            monde: 1,
            pas: 3600,
            gouttes: 20,
            morts: 7,
            vies: 3,
            score: 900,
            gouttes_bonus: None,
            epilogue: true,
            lettres: None,
        };
        assert_eq!(
            noms(&base),
            ["TEMPS", "GOUTTES D'OR", "MORTS", "VIES", "SCORE"],
            "bonus manqué"
        );
        assert_eq!(mise_en_page(&base).lignes[3].1, "3");
        let complet = Bilan {
            monde: 1,
            gouttes: 33,
            gouttes_bonus: Some(3),
            ..base
        };
        assert_eq!(
            noms(&complet),
            [
                "TEMPS",
                "GOUTTES D'OR",
                "NIVEAU SECRET",
                "MORTS",
                "VIES",
                "SCORE"
            ],
            "tout ramassé : le mot tient sur une ligne"
        );
        assert_eq!(
            mise_en_page(&complet).pas_lignes,
            10.0,
            "resserrées d'un pixel"
        );
        assert_eq!(mise_en_page(&base).pas_lignes, 11.0, "à l'écart habituel");
        let incomplet = Bilan {
            monde: 1,
            gouttes: 33,
            gouttes_bonus: Some(1),
            ..base
        };
        assert!(
            !noms(&incomplet).contains(&"VIES"),
            "ne tient pas : pas de ligne"
        );
        // Quand elle est là, le mot finit au moins 2 px au-dessus du menu.
        for b in bilans() {
            let m = mise_en_page(&b);
            if m.lignes.iter().any(|l| l.0 == "VIES") {
                let fin_mot = m.y_mot + (m.mot.len() - 1) as f32 * 10.0 + 8.0;
                assert!(fin_mot + MARGE_MENU <= Y_MENU, "{b:?}");
            }
        }
    }
}
