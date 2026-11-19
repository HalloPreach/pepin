//! La DÉMO de l'écran titre (carte 8.1) : après dix secondes sans aucune entrée, le
//! robot joue tout seul un niveau, comme sur les bornes d'arcade. Toute entrée ramène
//! au titre ; la fin du niveau, ou 45 secondes, aussi.
//!
//! La recherche du robot de QA (`bot_qa`) prend plusieurs secondes : bien trop lent
//! pour le jeu. On EMBARQUE donc des parcours pré-enregistrés (`parcours.rs`, fichier
//! GÉNÉRÉ par le test ignoré `ecrire_parcours_de_la_demo`) : pour chaque niveau, la
//! suite des entrées du robot, un pas après l'autre, compressée en plages (« 37 pas à
//! droite, puis 12 pas à droite en sautant... »). La simulation étant déterministe, les
//! rejouer sur le même niveau refait exactement le parcours du robot.
//!
//! Si un niveau change plus tard sans que les parcours soient régénérés, le rejeu peut
//! ne plus finir (Pépin meurt, ou les entrées s'épuisent avant le drapeau) : la démo
//! s'arrête alors proprement et revient au titre. Le test
//! `chaque_parcours_de_la_demo_atteint_le_drapeau` le signale avant.
//!
//! La démo n'écrit RIEN dans la sauvegarde : elle fait avancer un `Monde` à part
//! (`app.rs`, `pas_demo`), sans défis, sans gouttes au porte-monnaie, sans record.
//!
//! Notions Rust illustrées ici : codage par plages (RLE), tableau `&'static` de
//! tuples généré par un programme, itérateur fait main (`Rejeu`).

mod parcours;

use pepin_core::levels::NIVEAUX;
use pepin_core::palette::{ARDOISE_OMBRE, BLANC_CASSE, OR};
use pepin_core::physics::HZ;
use pepin_core::player::{Direction, Entrees};
use pepin_core::LARGEUR_LOGIQUE;

use crate::contenu;
use crate::ecrans::rect;
use crate::texte::{self, Style};

pub use parcours::PARCOURS;

/// Pas sans aucune entrée sur le titre avant que la démo ne démarre (10 s).
pub const ATTENTE_PAS: u32 = 10 * HZ;
/// Durée maximale d'une démo (45 s) : un long niveau est coupé avant son drapeau.
pub const DUREE_MAX_PAS: u32 = 45 * HZ;

/// Le parcours embarqué d'un niveau, désigné par son NOM (le rang d'un niveau dans
/// `NIVEAUX` peut changer quand on en ajoute un).
pub struct ParcoursDemo {
    pub nom: &'static str,
    /// Plages `(nombre de pas, code des entrées)` ; code : 2 = gauche, 4 = droite,
    /// 8 = saut (le même que les scripts du navigateur, sans le 1 d'« Entrée »).
    pub plages: &'static [(u16, u8)],
}

/// Code d'un pas d'entrées (voir `ParcoursDemo::plages`). Sert à l'enregistrement.
#[cfg(test)]
pub fn code(e: &Entrees) -> u8 {
    let direction = match e.direction {
        Direction::Gauche => 2,
        Direction::Droite => 4,
        Direction::Immobile => 0,
    };
    direction | if e.saut { 8 } else { 0 }
}

/// Les entrées d'un code (l'inverse de `code`).
pub fn entrees(code: u8) -> Entrees {
    Entrees {
        direction: match code & 6 {
            2 => Direction::Gauche,
            4 => Direction::Droite,
            _ => Direction::Immobile,
        },
        saut: code & 8 != 0,
        bas: false,
    }
}

/// Rang dans `NIVEAUX` du niveau qui porte ce nom (`None` : il n'existe plus).
pub fn indice_du_niveau(nom: &str) -> Option<usize> {
    NIVEAUX.iter().position(|n| n.nom == nom)
}

/// Relit un parcours embarqué, un pas après l'autre.
#[derive(Debug, Clone)]
pub struct Rejeu {
    plages: &'static [(u16, u8)],
    /// Plage en cours, et pas déjà joués dans cette plage.
    plage: usize,
    fait: u16,
}

impl Rejeu {
    pub fn new(plages: &'static [(u16, u8)]) -> Self {
        Self {
            plages,
            plage: 0,
            fait: 0,
        }
    }
}

impl Iterator for Rejeu {
    type Item = Entrees;

    /// Les entrées du pas suivant ; `None` quand le parcours est fini.
    fn next(&mut self) -> Option<Entrees> {
        while let Some(&(nombre, code)) = self.plages.get(self.plage) {
            if self.fait < nombre {
                self.fait += 1;
                return Some(entrees(code));
            }
            self.plage += 1;
            self.fait = 0;
        }
        None
    }
}

/// La démo en cours : quel niveau, où en est le rejeu, depuis combien de pas.
#[derive(Debug, Clone)]
pub struct Demo {
    pub rejeu: Rejeu,
    pub pas: u32,
}

/// Choisit la prochaine démo, en tournant dans `PARCOURS` à partir de `rang` : le
/// rang du parcours retenu et l'indice de son niveau. Un parcours dont le niveau
/// n'existe plus est sauté ; `None` si aucun ne convient.
pub fn prochaine(rang: usize) -> Option<(usize, usize)> {
    (0..PARCOURS.len())
        .map(|k| (rang + k) % PARCOURS.len())
        .find_map(|r| indice_du_niveau(PARCOURS[r].nom).map(|i| (r, i)))
}

/// Le bandeau de la démo, par-dessus le niveau : « DÉMO » qui clignote en haut,
/// « APPUIE SUR UNE TOUCHE » en bas. `anim` : pas écoulés (clignotement).
pub fn dessiner_bandeau(anim: u32) {
    let l = LARGEUR_LOGIQUE as f32;
    rect(0.0, 0.0, l, 20.0, ARDOISE_OMBRE);
    if (anim / 30).is_multiple_of(2) {
        texte::dessiner_centre(contenu::DEMO, 3.0, Style::simple(OR, 2));
    }
    rect(0.0, 164.0, l, 16.0, ARDOISE_OMBRE);
    texte::dessiner_centre(
        contenu::APPUIE_SUR_UNE_TOUCHE,
        168.0,
        Style::simple(BLANC_CASSE, 1),
    );
}

/// Compresse une suite d'entrées en plages `(nombre, code)` (au plus `u16::MAX` pas
/// par plage).
#[cfg(test)]
pub fn compresser(suite: &[Entrees]) -> Vec<(u16, u8)> {
    let mut plages: Vec<(u16, u8)> = Vec::new();
    for e in suite {
        let c = code(e);
        match plages.last_mut() {
            Some((n, dernier)) if *dernier == c && *n < u16::MAX => *n += 1,
            _ => plages.push((1, c)),
        }
    }
    plages
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monde::Monde;

    /// Les niveaux de la démo, par leur nom : variés (toits, orage, vents, mer).
    const NIVEAUX_DEMO: [&str; 4] = [
        "Les toits",
        "Le grand orage",
        "Vents d'altitude",
        "La mer de nuages",
    ];

    /// Chaque parcours embarqué, rejoué dans la vraie simulation, atteint le drapeau
    /// sans mourir (sinon : régénérer `parcours.rs`, voir le test d'en bas). Les
    /// plages se décompressent en la même suite d'entrées qu'avant compression.
    #[test]
    fn chaque_parcours_de_la_demo_atteint_le_drapeau() {
        assert!(!PARCOURS.is_empty());
        for p in PARCOURS {
            let i = indice_du_niveau(p.nom)
                .unwrap_or_else(|| panic!("niveau « {} » introuvable", p.nom));
            let suite: Vec<Entrees> = Rejeu::new(p.plages).collect();
            assert_eq!(compresser(&suite), p.plages, "« {} » : plages", p.nom);
            let mut m = Monde::new(i);
            for e in &suite {
                m.avancer(e);
                assert_eq!(m.etat.morts, 0, "« {} » : mort au pas {}", p.nom, m.pas);
            }
            assert!(m.termine, "« {} » : drapeau non atteint", p.nom);
        }
    }

    /// Régénère `src/demo/parcours.rs` à partir des parcours du robot de QA (à
    /// relancer si l'un des niveaux de la démo change) :
    /// `./docker-cargo.sh test -p game --no-default-features -- --ignored ecrire_parcours_de_la_demo`
    #[test]
    #[ignore]
    fn ecrire_parcours_de_la_demo() {
        use crate::bot_qa::trouver_parcours;
        let mut source = String::from(
            "//! FICHIER GÉNÉRÉ par le test ignoré `ecrire_parcours_de_la_demo`\n\
             //! (`demo/mod.rs`) : ne pas le modifier à la main.\n\
             //!\n\
             //! Parcours du robot de QA, compressés en plages `(nombre de pas, code)`.\n\n\
             use super::ParcoursDemo;\n\n\
             #[rustfmt::skip]\n\
             pub const PARCOURS: &[ParcoursDemo] = &[\n",
        );
        for nom in NIVEAUX_DEMO {
            let i = indice_du_niveau(nom).expect("niveau de la démo");
            let suite = trouver_parcours(i, false)
                .or_else(|| trouver_parcours(i, true))
                .expect("parcours du robot");
            let plages = compresser(&suite);
            println!(
                "{nom} : {} pas ({:.1} s), {} plages",
                suite.len(),
                suite.len() as f32 / HZ as f32,
                plages.len()
            );
            source.push_str(&format!(
                "    ParcoursDemo {{\n        nom: {nom:?},\n        plages: &["
            ));
            for (k, (n, c)) in plages.iter().enumerate() {
                if k % 10 == 0 {
                    source.push_str("\n            ");
                } else {
                    source.push(' ');
                }
                source.push_str(&format!("({n}, {c}),"));
            }
            source.push_str("\n        ],\n    },\n");
        }
        source.push_str("];\n");
        std::fs::write("src/demo/parcours.rs", source).unwrap();
    }
}
