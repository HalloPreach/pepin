//! Le robot de faisabilité de `core` (`faisabilite::analyser`), pour l'éditeur de
//! niveaux (carte 10.3c). Le MÊME code Rust que les tests, compilé en un petit
//! module WebAssembly que l'éditeur lance dans un Web Worker (la page ne fige pas,
//! et le bouton d'arrêt ou la limite de temps n'ont qu'à arrêter le Worker).
//!
//! Le module n'a besoin d'aucun outil JavaScript (pas de wasm-bindgen) : il
//! échange des octets avec la page, par trois fonctions exportées en « ABI C » :
//!
//! 1. `robot_tampon(n)` réserve `n` octets dans la mémoire du module et rend leur
//!    adresse : la page y écrit le texte RON du niveau ;
//! 2. `robot_lancer(n, planer)` lit les `n` octets du niveau, lance le robot (parapluie
//!    permis si `planer` vaut 1) et rend la longueur du résultat (du JSON) ;
//! 3. `robot_resultat()` rend l'adresse de ce JSON, que la page lit.
//!
//! Notions Rust illustrées ici : `crate-type = ["cdylib"]` (une bibliothèque pour
//! un autre langage), `#[no_mangle] pub extern "C" fn`, pointeurs bruts et bloc
//! `unsafe` (le seul endroit où on fait confiance à la page), `thread_local!` +
//! `RefCell` pour garder une valeur entre deux appels sans `static mut`.

use std::cell::RefCell;
use std::fmt::Write as _;

use pepin_core::aabb::TAILLE_TUILE;
use pepin_core::faisabilite::{analyser, Options};

/// Pas de physique par seconde (le jeu tourne à 60 pas par seconde).
const PAS_PAR_SECONDE: f32 = 60.0;

/// Lance le robot sur le niveau `texte` (RON, format de `assets/levels`) et rend
/// le résultat en JSON (champs décrits dans `editeur/robot-worker.js`). Le robot
/// explore TOUT le niveau (`Options::default`, pas d'arrêt à l'arrivée) : la
/// durée est alors celle du trajet le plus rapide, et les lanternes celles qu'un
/// trajet quelconque allume.
pub fn analyser_texte(texte: &str, planer: bool) -> String {
    let infos = match niveaux::charger_fichier("niveau de l'éditeur", texte) {
        Ok(infos) => infos,
        Err(e) => return erreur(&e.to_string()),
    };
    let niveau = match infos.charger() {
        Ok(niveau) => niveau,
        Err(e) => return erreur(&e.to_string()),
    };
    let options = Options {
        planer,
        ..Options::default()
    };
    let rapport = analyser(&niveau, niveau.depart, options);

    let allumees = rapport.lanternes.iter().filter(|&&l| l).count();
    let gouttes = rapport.gouttes.iter().filter(|&&g| g).count();
    let raison = if rapport.arrivee {
        String::new()
    } else if niveau.arrivee.is_none() {
        "le niveau n'a pas d'arrivée X".to_string()
    } else if rapport.etats <= 1 {
        "Pépin ne peut pas quitter le départ : chaque saut essayé tombe ou meurt".to_string()
    } else {
        let (x, y) = rapport.dernier_etat;
        let mut r = format!(
            "aucun saut connu du robot ne mène plus loin que la colonne {}, rangée {}",
            case(x + pepin_core::physics::JOUEUR_L / 2.0),
            case(y + pepin_core::physics::JOUEUR_H - 1.0),
        );
        if !planer {
            r.push_str(" (parapluie interdit)");
        }
        r
    };

    let mut json = String::from("{\"etat\":\"ok\"");
    let _ = write!(json, ",\"arrivee\":{}", rapport.arrivee);
    match rapport.pas_arrivee {
        Some(pas) => {
            let _ = write!(
                json,
                ",\"pas\":{pas},\"secondes\":{:.1}",
                pas as f32 / PAS_PAR_SECONDE
            );
        }
        None => json.push_str(",\"pas\":null,\"secondes\":null"),
    }
    let _ = write!(
        json,
        ",\"lanternes\":{allumees},\"lanternes_total\":{},\"gouttes\":{gouttes},\"etats\":{}",
        rapport.lanternes.len(),
        rapport.etats
    );
    let _ = write!(
        json,
        ",\"dernier\":[{:.1},{:.1}],\"tuile\":{TAILLE_TUILE},\"joueur\":[{},{}]",
        rapport.dernier_etat.0,
        rapport.dernier_etat.1,
        pepin_core::physics::JOUEUR_L,
        pepin_core::physics::JOUEUR_H
    );
    json.push_str(",\"chemin\":[");
    for (i, (x, y)) in rapport.chemin.iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        let _ = write!(json, "[{x:.1},{y:.1}]");
    }
    json.push(']');
    let _ = write!(json, ",\"raison\":{}}}", texte_json(&raison));
    json
}

/// La case (colonne ou rangée) qui contient la coordonnée `px`.
fn case(px: f32) -> i32 {
    (px / TAILLE_TUILE).floor() as i32
}

/// Un résultat d'erreur : le niveau ne se charge pas (le bouton Valider en dit
/// plus, avec la case fautive).
fn erreur(message: &str) -> String {
    format!(
        "{{\"etat\":\"erreur\",\"message\":{}}}",
        texte_json(message)
    )
}

/// Une chaîne JSON entre guillemets, caractères spéciaux échappés.
fn texte_json(s: &str) -> String {
    let mut r = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => r.push_str("\\\""),
            '\\' => r.push_str("\\\\"),
            '\n' => r.push_str("\\n"),
            c if (c as u32) < 0x20 => {
                let _ = write!(r, "\\u{:04x}", c as u32);
            }
            c => r.push(c),
        }
    }
    r.push('"');
    r
}

// --- Les trois fonctions exportées vers la page ---------------------------------

thread_local! {
    /// Le tampon d'entrée (le texte RON écrit par la page).
    static ENTREE: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    /// Le dernier résultat (JSON), lu par la page après `robot_lancer`.
    static RESULTAT: RefCell<String> = const { RefCell::new(String::new()) };
}

/// Réserve `n` octets pour le texte du niveau ; rend leur adresse.
#[no_mangle]
pub extern "C" fn robot_tampon(n: usize) -> *mut u8 {
    ENTREE.with_borrow_mut(|tampon| {
        *tampon = vec![0; n];
        tampon.as_mut_ptr()
    })
}

/// Lance le robot sur les `n` octets écrits dans le tampon ; rend la longueur du
/// JSON produit. `planer` : 1 pour permettre le parapluie, 0 pour l'interdire.
#[no_mangle]
pub extern "C" fn robot_lancer(n: usize, planer: u32) -> usize {
    let texte = ENTREE.with_borrow(|tampon| {
        let n = n.min(tampon.len());
        String::from_utf8_lossy(&tampon[..n]).into_owned()
    });
    let json = analyser_texte(&texte, planer != 0);
    let longueur = json.len();
    RESULTAT.set(json);
    longueur
}

/// Adresse du dernier résultat (`robot_lancer` en a rendu la longueur).
#[no_mangle]
pub extern "C" fn robot_resultat() -> *const u8 {
    RESULTAT.with_borrow(|r| r.as_ptr())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un petit niveau RON : sol plat, un trou de `trou` tuiles, l'arrivée au bout.
    fn niveau(trou: usize) -> String {
        let largeur = 12 + trou + 8;
        let mut lignes = vec![".".repeat(largeur); 10];
        lignes[1].replace_range(2..5, "ooo");
        lignes[7].replace_range(2..3, "@");
        lignes[7].replace_range(largeur - 3..largeur - 2, "X");
        for ligne in &mut lignes[8..10] {
            *ligne = (0..largeur)
                .map(|x| {
                    if (12..12 + trou).contains(&x) {
                        '.'
                    } else {
                        '#'
                    }
                })
                .collect();
        }
        format!(
            "(\n    monde: 1,\n    numero: 0,\n    cle: \"essai_robot\",\n    nom: \"Essai robot\",\n    \
             theme: Altitude,\n    genre: Normal,\n    sans_planer: false,\n    duree_cible_s: 30,\n    \
             intro: \"\",\n    mecanique: \"\",\n    aiguilles: [],\n    poissons_hauts: [],\n    \
             ascii: r#\"\n{}\n\"#,\n)\n",
            lignes.join("\n")
        )
    }

    #[test]
    fn un_niveau_simple_est_termine_avec_son_chemin() {
        let json = analyser_texte(&niveau(2), true);
        assert!(json.contains("\"arrivee\":true"), "{json}");
        assert!(json.contains("\"raison\":\"\""), "{json}");
        // Le chemin part du départ et compte plus d'un point (au moins un saut).
        let chemin = json.split("\"chemin\":[").nth(1).unwrap();
        assert!(chemin.starts_with('['), "{json}");
        assert!(chemin.split("],[").count() >= 2, "{json}");
    }

    #[test]
    fn un_trou_trop_large_dit_ou_le_robot_bloque() {
        // 12 tuiles : plus que n'importe quel saut, même en planant.
        let json = analyser_texte(&niveau(14), true);
        assert!(json.contains("\"arrivee\":false"), "{json}");
        assert!(json.contains("\"pas\":null"), "{json}");
        // Bloqué au bord du trou : colonne 11 (dernière tuile de sol avant le trou).
        assert!(json.contains("plus loin que la colonne 11"), "{json}");
    }

    #[test]
    fn un_texte_invalide_rend_une_erreur() {
        let json = analyser_texte("(monde: ", true);
        assert!(json.starts_with("{\"etat\":\"erreur\""), "{json}");
    }

    #[test]
    fn les_fonctions_exportees_font_l_aller_retour() {
        let texte = niveau(2);
        let adresse = robot_tampon(texte.len());
        // SAFETY : `robot_tampon` vient de réserver exactement `texte.len()` octets.
        unsafe { std::ptr::copy_nonoverlapping(texte.as_ptr(), adresse, texte.len()) };
        let n = robot_lancer(texte.len(), 1);
        // SAFETY : `robot_resultat` pointe sur les `n` octets du résultat.
        let lu = unsafe { std::slice::from_raw_parts(robot_resultat(), n) };
        assert_eq!(
            std::str::from_utf8(lu).unwrap(),
            analyser_texte(&texte, true)
        );
    }
}
