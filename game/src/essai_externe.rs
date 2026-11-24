//! Niveau d'essai VENU DE L'EXTÉRIEUR et erreurs de niveau à l'écran (carte 10.1b),
//! build de QA seulement (`debug_niveaux`) : la porte d'entrée de l'éditeur de
//! niveaux (carte 10.3b).
//!
//! - Ouvrir la page avec `?essai=1` lance directement le niveau écrit en RON dans
//!   le `localStorage` du navigateur, sous la clé `pepin.niveau_essai` (en natif :
//!   le fichier dont le chemin est dans la variable `PEPIN_NIVEAU_ESSAI`).
//! - Un fichier invalide (ce niveau-là, ou un fichier embarqué) ne fait pas
//!   planter le jeu : l'écran affiche l'erreur lisible du crate `niveaux`
//!   (fichier, ligne, colonne) et ce qu'il faut corriger.
//!
//! Le jeu publié n'a rien de tout cela : ses fichiers embarqués sont vérifiés
//! avant le build (`./build.sh`), et il ne lit aucun niveau de l'extérieur.
//!
//! - Carte 10.3b : le jeu PARLE à l'éditeur qui l'a ouvert (`signaler`) : le
//!   verdict du chargeur (`pret`, ou `erreur` avec la case fautive) et la fin de
//!   l'essai (Échap, ou la fin du niveau), pour revenir à l'éditeur. Aucune
//!   règle n'est recopiée en JavaScript : le juge est `niveaux::charger_fichier`.
//!
//! Notions Rust : `OnceLock` (une valeur posée une fois, lue ensuite partout),
//! `extern "C"` vers une fonction JavaScript, `#[cfg(target_arch = ...)]` pour
//! écrire deux versions d'une même fonction (navigateur et natif).

// Les tests natifs n'ont que le niveau externe (pas d'écran ni de lancement).
#![cfg_attr(not(feature = "debug_niveaux"), allow(dead_code))]

use macroquad::prelude::{draw_rectangle, is_key_pressed, next_frame, KeyCode};
use pepin_core::palette::{ARDOISE_NUIT, BLANC_CASSE, CORAIL, OR};
use pepin_core::zones::InfosNiveau;
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};
use std::sync::OnceLock;

use crate::contenu::Intro;
use crate::ecrans::Ecran;
use crate::rendu::{couleur_mq, Cible};
use crate::texte::{self, Style};

/// Nom donné au niveau externe dans les messages d'erreur.
pub const SOURCE: &str = "localStorage pepin.niveau_essai";

/// Le niveau externe, une fois lu et validé (`None` : pas demandé).
static NIVEAU: OnceLock<InfosNiveau> = OnceLock::new();

/// Ce que le jeu fait au lancement.
#[derive(Debug)]
pub enum Lancement {
    /// Comme d'habitude : l'écran titre.
    Normal,
    /// `?essai=1` et un niveau valide : on le lance tout de suite.
    Essai,
    /// Un fichier invalide : on affiche `message` au lieu de jouer, et on envoie
    /// `rapport` (JSON, voir `rapport_erreur`) à l'éditeur.
    Erreur { message: String, rapport: String },
}

impl Lancement {
    /// Une erreur sans case connue (fichier embarqué, niveau absent).
    fn erreur_simple(message: String) -> Self {
        let rapport = format!(r#"{{"etat":"erreur","message":{}}}"#, json_texte(&message));
        Lancement::Erreur { message, rapport }
    }
}

/// Vérifie les fichiers embarqués, puis lit le niveau externe s'il est demandé.
pub fn au_lancement() -> Lancement {
    if let Err(e) = niveaux::table_verifiee() {
        return Lancement::erreur_simple(e.to_string());
    }
    match texte_demande() {
        None => Lancement::Normal,
        Some(None) => Lancement::erreur_simple(format!(
            "?essai=1 : aucun niveau dans le navigateur ({SOURCE} est vide)"
        )),
        Some(Some(texte)) => match lire(&texte) {
            Ok(()) => Lancement::Essai,
            Err(e) => Lancement::Erreur {
                message: e.to_string(),
                rapport: rapport_erreur(&e),
            },
        },
    }
}

/// Lit et valide un niveau RON (même chargeur que les fichiers embarqués) et le
/// garde pour `niveau()`. `Err` : l'erreur TYPÉE (son texte est le message
/// lisible ; `rapport_erreur` y retrouve la case fautive).
pub fn lire(texte: &str) -> Result<(), niveaux::ErreurChargement> {
    let infos = niveaux::charger_fichier(SOURCE, texte)?;
    // Déjà posé (deuxième appel) : on garde le premier, comme `OnceLock` l'impose.
    let _ = NIVEAU.set(infos);
    Ok(())
}

/// Le rapport d'erreur envoyé à l'éditeur, en JSON :
/// `{"etat":"erreur","message":...,"ligne":L,"colonne":C,"ligne_fichier":F}`.
/// `ligne` et `colonne` sont celles de la GRILLE (depuis 1, comme le message),
/// `ligne_fichier` celle du fichier pour une faute de syntaxe ; un champ
/// inconnu vaut `null`. L'éditeur surligne la case `(colonne - 1, ligne - 1)`.
pub fn rapport_erreur(e: &niveaux::ErreurChargement) -> String {
    use niveaux::GenreErreur;
    let (case, ligne_fichier) = match &e.genre {
        GenreErreur::Grille { erreur, .. } => (erreur.case(), None),
        GenreErreur::Syntaxe { ligne, .. } => (None, Some(*ligne)),
        _ => (None, None),
    };
    // `Option<usize>` vers un nombre JSON, ou `null`.
    let nombre = |n: Option<usize>| n.map_or_else(|| "null".to_string(), |n| n.to_string());
    format!(
        r#"{{"etat":"erreur","message":{},"ligne":{},"colonne":{},"ligne_fichier":{}}}"#,
        json_texte(&e.to_string()),
        nombre(case.map(|(l, _)| l)),
        nombre(case.and_then(|(_, c)| c)),
        nombre(ligne_fichier),
    )
}

/// Une chaîne JSON entre guillemets : `"` et `\` échappés, caractères de
/// contrôle en `\uXXXX`. (Le reste de l'UTF-8 passe tel quel, JSON l'accepte.)
fn json_texte(s: &str) -> String {
    let mut sortie = String::with_capacity(s.len() + 2);
    sortie.push('"');
    for c in s.chars() {
        match c {
            '"' => sortie.push_str("\\\""),
            '\\' => sortie.push_str("\\\\"),
            c if (c as u32) < 0x20 => sortie.push_str(&format!("\\u{:04x}", c as u32)),
            c => sortie.push(c),
        }
    }
    sortie.push('"');
    sortie
}

/// Suit un essai lancé par l'éditeur (carte 10.3b) : Échap, ou l'écran de fin
/// (le niveau est fini), le termine et ramène à l'éditeur. Une seule fois.
pub struct SuiviEssai {
    en_cours: bool,
}

impl SuiviEssai {
    pub fn new(lancement: &Lancement) -> Self {
        let en_cours = matches!(lancement, Lancement::Essai);
        if en_cours {
            signaler(r#"{"etat":"pret"}"#);
        }
        SuiviEssai { en_cours }
    }

    /// À appeler à chaque image, après la simulation.
    pub fn apres_image(&mut self, ecran: Ecran) {
        if !self.en_cours {
            return;
        }
        let raison = if is_key_pressed(KeyCode::Escape) {
            "echap"
        } else if ecran == Ecran::Fin {
            "fin"
        } else {
            return;
        };
        self.en_cours = false;
        signaler(&format!(r#"{{"etat":"fini","raison":"{raison}"}}"#));
    }
}

/// Envoie un message JSON à l'éditeur (navigateur) ; en natif, l'écrit sur la
/// sortie d'erreur.
pub fn signaler(json: &str) {
    #[cfg(target_arch = "wasm32")]
    navigateur::signaler(json);
    #[cfg(not(target_arch = "wasm32"))]
    eprintln!("essai : {json}");
}

/// Le niveau externe validé, s'il y en a un (`monde::INDICE_ESSAI_EXTERNE`).
pub fn niveau() -> Option<&'static InfosNiveau> {
    NIVEAU.get()
}

/// L'intro de la carte du niveau externe (carte 10.3b) : ses champs `intro` et
/// `mecanique`, tels qu'écrits dans l'éditeur (sinon : celle du dernier niveau).
pub fn intro() -> Option<&'static Intro> {
    static INTRO: OnceLock<Intro> = OnceLock::new();
    let infos = niveau()?;
    Some(INTRO.get_or_init(|| Intro {
        texte: infos.intro,
        astuce: infos.mecanique,
    }))
}

/// Boucle sans fin de l'écran d'erreur : fond uni, titre, message coupé en lignes.
pub async fn afficher_erreur(cible: &Cible, message: &str) -> ! {
    let lignes = texte::couper_lignes(message, 50);
    loop {
        cible.commencer();
        draw_rectangle(
            0.0,
            0.0,
            LARGEUR_LOGIQUE as f32,
            HAUTEUR_LOGIQUE as f32,
            couleur_mq(ARDOISE_NUIT),
        );
        texte::dessiner_centre("NIVEAU INVALIDE", 14.0, Style::simple(CORAIL, 2));
        for (i, ligne) in lignes.iter().take(14).enumerate() {
            texte::dessiner(
                ligne,
                10.0,
                40.0 + 9.0 * i as f32,
                Style::simple(BLANC_CASSE, 1),
            );
        }
        texte::dessiner_centre(
            "corrige le fichier puis recharge la page",
            168.0,
            Style::simple(OR, 1),
        );
        cible.afficher();
        next_frame().await;
    }
}

/// `None` : pas demandé ; `Some(None)` : demandé mais rien à lire ;
/// `Some(Some(texte))` : le texte RON à charger.
#[cfg(target_arch = "wasm32")]
fn texte_demande() -> Option<Option<String>> {
    navigateur::texte_demande()
}

/// Natif : la variable d'environnement `PEPIN_NIVEAU_ESSAI` donne le chemin du
/// fichier (pratique pour essayer un niveau sans navigateur).
#[cfg(not(target_arch = "wasm32"))]
fn texte_demande() -> Option<Option<String>> {
    let chemin = std::env::var("PEPIN_NIVEAU_ESSAI").ok()?;
    Some(std::fs::read_to_string(chemin).ok())
}

/// Navigateur : le plugin JavaScript de `web/index.html` (avec la sauvegarde).
#[cfg(target_arch = "wasm32")]
mod navigateur {
    extern "C" {
        /// 1 si l'adresse de la page contient `?essai=1`, 0 sinon.
        fn pepin_niveau_essai_demande() -> u32;
        /// Copie le niveau (UTF-8) en `ptr` s'il tient dans `max` octets ; renvoie
        /// sa longueur en octets (0 : rien, ou `localStorage` refusé).
        fn pepin_niveau_essai_lire(ptr: *mut u8, max: u32) -> u32;
        /// Passe le message JSON (UTF-8, `len` octets en `ptr`) à l'éditeur.
        fn pepin_niveau_essai_signaler(ptr: *const u8, len: u32);
    }

    pub fn signaler(json: &str) {
        // SAFETY : le plugin lit exactement `len` octets à partir de `ptr`.
        unsafe { pepin_niveau_essai_signaler(json.as_ptr(), json.len() as u32) }
    }

    pub fn texte_demande() -> Option<Option<String>> {
        // SAFETY : fonction JavaScript sans argument.
        if unsafe { pepin_niveau_essai_demande() } == 0 {
            return None;
        }
        let mut tampon = vec![0u8; 64 * 1024];
        // SAFETY : le plugin écrit au plus `max` octets dans le tampon fourni.
        let mut n = unsafe { pepin_niveau_essai_lire(tampon.as_mut_ptr(), tampon.len() as u32) };
        if n as usize > tampon.len() {
            // Plus long que prévu : un second essai avec la bonne taille.
            tampon.resize(n as usize, 0);
            // SAFETY : idem.
            n = unsafe { pepin_niveau_essai_lire(tampon.as_mut_ptr(), tampon.len() as u32) };
        }
        let n = n as usize;
        if n == 0 || n > tampon.len() {
            return Some(None);
        }
        tampon.truncate(n);
        Some(String::from_utf8(tampon).ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un niveau externe valide est gardé et devient le niveau de
    /// `monde::INDICE_ESSAI_EXTERNE` ; une faute donne un message avec sa ligne.
    #[test]
    fn un_niveau_externe_se_charge_et_une_faute_s_affiche() {
        let bon = niveaux::ecrire(niveaux::par_cle("essai_vent"));
        let faux = bon.replace("theme: Soleil", "theme: Lune");
        assert_ne!(faux, bon);
        let message = lire(&faux).unwrap_err().to_string();
        assert!(
            message.contains(SOURCE) && message.contains("ligne"),
            "{message}"
        );
        lire(&bon).unwrap();
        let infos = crate::monde::infos_niveau(crate::monde::INDICE_ESSAI_EXTERNE).unwrap();
        assert_eq!(infos.cle, "essai_vent");
        // Sa carte de niveau montre SON intro (carte 10.3b).
        let intro = crate::contenu::intro(crate::monde::INDICE_ESSAI_EXTERNE);
        assert_eq!((intro.texte, intro.astuce), (infos.intro, infos.mecanique));
    }

    /// Le rapport JSON de l'éditeur (carte 10.3b) donne la case fautive de la
    /// grille, et échappe guillemets et retours à la ligne.
    #[test]
    fn le_rapport_d_erreur_donne_la_case_fautive() {
        let bon = niveaux::ecrire(niveaux::par_cle("essai_vent"));
        // Une lettre inconnue « ; » à la place du départ : ligne et colonne connues.
        let faux = bon.replacen('@', ";", 1);
        let e = lire(&faux).unwrap_err();
        let rapport = rapport_erreur(&e);
        let (ligne, colonne) = match &e.genre {
            niveaux::GenreErreur::Grille { erreur, .. } => match erreur.case() {
                Some((l, Some(c))) => (l, c),
                autre => panic!("case attendue : {autre:?}"),
            },
            autre => panic!("erreur de grille attendue : {autre:?}"),
        };
        assert!(
            rapport.starts_with(r#"{"etat":"erreur","message":""#)
                && rapport.contains(&format!(r#""ligne":{ligne},"colonne":{colonne},"#))
                && rapport.ends_with(r#""ligne_fichier":null}"#),
            "{rapport}"
        );
        assert_eq!(json_texte("a\"b\\c\n"), r#""a\"b\\c\u000a""#);
    }
}
