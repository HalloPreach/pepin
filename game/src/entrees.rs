//! Entrées : une abstraction `Commandes` indépendante du périphérique.
//!
//! Le jeu ne sait pas si l'on joue au clavier, à la manette ou au doigt : il lit
//! des `Commandes` (gauche, droite, saut, pause). Chaque périphérique est une
//! `SourceEntrees` ; `Sources` en fusionne plusieurs. Pour ajouter le tactile
//! plus tard, il suffira d'écrire une nouvelle structure qui implémente le trait.
//!
//! Notions Rust illustrées ici : `trait` (comportement commun à plusieurs types),
//! objet-trait `Box<dyn SourceEntrees>` (liste de sources de types différents),
//! `Copy` pour une petite structure passée par valeur.

use macroquad::prelude::{is_key_down, is_key_pressed, KeyCode};
use pepin_core::player::{Direction, Entrees};

/// Ce que le joueur demande pendant une image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Commandes {
    pub gauche: bool,
    pub droite: bool,
    /// Saut MAINTENU (en l'air, maintenir ouvre le parapluie).
    pub saut: bool,
    /// Demande de pause : vrai seulement à l'image de l'appui (front montant).
    pub pause: bool,
}

impl Commandes {
    /// Direction horizontale. Gauche ET droite en même temps s'annulent.
    pub fn direction(&self) -> Direction {
        match (self.gauche, self.droite) {
            (true, false) => Direction::Gauche,
            (false, true) => Direction::Droite,
            _ => Direction::Immobile,
        }
    }

    /// Les entrées que comprend la simulation du cœur.
    pub fn vers_entrees(&self) -> Entrees {
        Entrees {
            direction: self.direction(),
            saut: self.saut,
        }
    }

    /// Réunit deux jeux de commandes : une commande est active si l'un des deux
    /// périphériques la demande (clavier ET tactile peuvent coexister).
    pub fn fusionner(self, autre: Commandes) -> Commandes {
        Commandes {
            gauche: self.gauche || autre.gauche,
            droite: self.droite || autre.droite,
            saut: self.saut || autre.saut,
            pause: self.pause || autre.pause,
        }
    }
}

/// Un périphérique d'entrée (clavier, tactile, manette...).
pub trait SourceEntrees {
    /// Lit l'état du périphérique, une fois par image affichée.
    fn lire(&mut self) -> Commandes;
}

// --- Table des touches ----------------------------------------------------
//
// Le moteur lit la position PHYSIQUE des touches (le « code » du navigateur), pas
// la lettre imprimée dessus. Une même position porte donc deux lettres selon la
// disposition du clavier :
//
//     position QWERTY   lettre AZERTY      position QWERTY   lettre AZERTY
//     Q                 A                  W                 Z
//     A                 Q                  Z                 W
//     M                 ,  (virgule)       Semicolon (;)     M
//
// Pour chaque action on accepte donc les DEUX positions qui portent la lettre
// attendue, ainsi qu'au moins une touche qui ne dépend pas de la disposition
// (flèches, Espace, Entrée, Échap). Les listes d'une même situation (en jeu, dans
// un menu) ne partagent aucune touche : une touche ne déclenche jamais deux actions.

/// Courir à gauche : flèche, A et Q (WASD / ZQSD).
pub const TOUCHES_GAUCHE: [KeyCode; 3] = [KeyCode::Left, KeyCode::A, KeyCode::Q];
/// Courir à droite : flèche et D (même position sur les deux dispositions).
pub const TOUCHES_DROITE: [KeyCode; 2] = [KeyCode::Right, KeyCode::D];
/// Sauter / planer : Espace, flèche haut, W et Z (les deux positions du Z/W).
pub const TOUCHES_SAUT: [KeyCode; 4] = [KeyCode::Space, KeyCode::Up, KeyCode::W, KeyCode::Z];
/// Pause en jeu : Échap et P (même position sur les deux dispositions).
pub const TOUCHES_PAUSE: [KeyCode; 2] = [KeyCode::Escape, KeyCode::P];
/// Son : M (position QWERTY) et point-virgule (position de M sur AZERTY).
pub const TOUCHES_SON: [KeyCode; 2] = [KeyCode::M, KeyCode::Semicolon];

/// Menus : monter, comme pour le saut (flèche, W, Z).
pub const TOUCHES_HAUT: [KeyCode; 3] = [KeyCode::Up, KeyCode::W, KeyCode::Z];
/// Menus : descendre (flèche et S, même position sur les deux dispositions).
pub const TOUCHES_BAS: [KeyCode; 2] = [KeyCode::Down, KeyCode::S];
/// Menus : valider.
pub const TOUCHES_VALIDER: [KeyCode; 3] = [KeyCode::Enter, KeyCode::KpEnter, KeyCode::Space];
/// Menus : retour.
pub const TOUCHES_RETOUR: [KeyCode; 2] = [KeyCode::Escape, KeyCode::Backspace];

/// Le clavier : flèches, Q/A/D (WASD et ZQSD), Espace, Haut, W, Z, Échap, P.
pub struct Clavier;

/// Une touche de la liste est-elle enfoncée ?
fn une_enfoncee(touches: &[KeyCode]) -> bool {
    touches.iter().any(|&t| is_key_down(t))
}

impl SourceEntrees for Clavier {
    fn lire(&mut self) -> Commandes {
        // `is_key_pressed` rattrape un appui très bref, relâché avant l'image suivante.
        let saut = une_enfoncee(&TOUCHES_SAUT) || TOUCHES_SAUT.iter().any(|&t| is_key_pressed(t));
        Commandes {
            gauche: une_enfoncee(&TOUCHES_GAUCHE),
            droite: une_enfoncee(&TOUCHES_DROITE),
            saut,
            pause: TOUCHES_PAUSE.iter().any(|&t| is_key_pressed(t)),
        }
    }
}

/// Toutes les sources actives, fusionnées.
#[derive(Default)]
pub struct Sources {
    liste: Vec<Box<dyn SourceEntrees>>,
}

impl Sources {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ajoute un périphérique (la boîte prend la possession de la source).
    pub fn ajouter(&mut self, source: Box<dyn SourceEntrees>) {
        self.liste.push(source);
    }

    /// Lit chaque source et fusionne leurs commandes.
    pub fn lire(&mut self) -> Commandes {
        self.liste
            .iter_mut()
            .fold(Commandes::default(), |acc, s| acc.fusionner(s.lire()))
    }
}

/// Les raccourcis 1 à 5 (saut à un niveau) n'existent que dans un build de mise au
/// point : fonctionnalité Cargo `debug_niveaux`, absente de la production.
/// En production, un appui accidentel ne doit jamais perdre la partie ni permettre
/// de finir avec un temps truqué (sur AZERTY, ces touches sont & é " ' ( ).
pub const RACCOURCIS_NIVEAUX: bool = cfg!(feature = "debug_niveaux");

/// Touches de saut de niveau, dans l'ordre des niveaux 1 à 5.
const TOUCHES_NIVEAUX: [KeyCode; 5] = [
    KeyCode::Key1,
    KeyCode::Key2,
    KeyCode::Key3,
    KeyCode::Key4,
    KeyCode::Key5,
];

/// Niveau demandé (indice 0 à 4) d'après les touches appuyées cette image.
/// Toujours `None` quand `actif` est faux : la logique est isolée ici pour être
/// testée sans fenêtre, avec ou sans la fonctionnalité.
pub fn niveau_demande(actif: bool, appuyee: impl Fn(KeyCode) -> bool) -> Option<usize> {
    if !actif {
        return None;
    }
    TOUCHES_NIVEAUX.iter().position(|&k| appuyee(k))
}

/// Raccourci de mise au point : touches 1 à 5 pour sauter à un niveau
/// (uniquement avec la fonctionnalité `debug_niveaux`).
pub fn raccourci_niveau() -> Option<usize> {
    niveau_demande(RACCOURCIS_NIVEAUX, is_key_pressed)
}

/// Plantage volontaire (touche F9) : n'existe que dans le build de QA, fonctionnalité
/// Cargo `qa_plantage`. Il sert à vérifier dans un vrai navigateur qu'un plantage en
/// cours de partie (panique Rust, devenue « RuntimeError: unreachable » en wasm)
/// affiche bien le message d'erreur de la page web au lieu d'une image figée.
pub const PLANTAGE_QA: bool = cfg!(feature = "qa_plantage");

/// Touche du plantage volontaire : F9, loin des touches de jeu (aucun appui accidentel).
const TOUCHE_PLANTAGE: KeyCode = KeyCode::F9;

/// Le plantage volontaire est-il demandé ? Toujours faux quand `actif` est faux.
pub fn plantage_demande(actif: bool, appuyee: impl Fn(KeyCode) -> bool) -> bool {
    actif && appuyee(TOUCHE_PLANTAGE)
}

/// Lit la touche du plantage volontaire (uniquement avec la fonctionnalité `qa_plantage`).
pub fn plantage_qa() -> bool {
    plantage_demande(PLANTAGE_QA, is_key_pressed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gauche() -> Commandes {
        Commandes {
            gauche: true,
            ..Commandes::default()
        }
    }

    /// Fausse source : renvoie toujours les mêmes commandes.
    struct Fixe(Commandes);
    impl SourceEntrees for Fixe {
        fn lire(&mut self) -> Commandes {
            self.0
        }
    }

    #[test]
    fn gauche_et_droite_ensemble_s_annulent() {
        let c = Commandes {
            gauche: true,
            droite: true,
            ..Commandes::default()
        };
        assert_eq!(c.direction(), Direction::Immobile);
        assert_eq!(gauche().direction(), Direction::Gauche);
    }

    #[test]
    fn la_conversion_garde_direction_et_saut() {
        let c = Commandes {
            droite: true,
            saut: true,
            ..Commandes::default()
        };
        let e = c.vers_entrees();
        assert_eq!(e.direction, Direction::Droite);
        assert!(e.saut);
    }

    #[test]
    fn la_fusion_est_un_ou_logique() {
        let saut = Commandes {
            saut: true,
            pause: true,
            ..Commandes::default()
        };
        let f = gauche().fusionner(saut);
        assert!(f.gauche && f.saut && f.pause && !f.droite);
    }

    #[test]
    fn plusieurs_sources_se_fusionnent() {
        let mut s = Sources::new();
        assert_eq!(s.lire(), Commandes::default(), "aucune source = rien");
        s.ajouter(Box::new(Fixe(gauche())));
        s.ajouter(Box::new(Fixe(Commandes {
            saut: true,
            ..Commandes::default()
        })));
        let c = s.lire();
        assert!(c.gauche && c.saut && !c.droite);
    }

    /// Toutes les listes d'une même situation : aucune touche en double.
    fn sans_doublon(listes: &[(&str, &[KeyCode])]) {
        for (i, (nom_a, a)) in listes.iter().enumerate() {
            for (nom_b, b) in &listes[i + 1..] {
                for k in *a {
                    assert!(
                        !b.contains(k),
                        "{k:?} sert à la fois à « {nom_a} » et « {nom_b} »"
                    );
                }
            }
        }
    }

    #[test]
    fn en_jeu_aucune_touche_ne_declenche_deux_actions() {
        sans_doublon(&[
            ("gauche", &TOUCHES_GAUCHE),
            ("droite", &TOUCHES_DROITE),
            ("saut", &TOUCHES_SAUT),
            ("pause", &TOUCHES_PAUSE),
            ("son", &TOUCHES_SON),
        ]);
    }

    #[test]
    fn dans_les_menus_aucune_touche_ne_declenche_deux_actions() {
        // « Échap » est à la fois retour (menus) et pause (en jeu) : ce sont deux
        // situations distinctes, et dans le menu pause les deux veulent dire REPRENDRE.
        sans_doublon(&[
            ("haut", &TOUCHES_HAUT),
            ("bas", &TOUCHES_BAS),
            ("valider", &TOUCHES_VALIDER),
            ("retour", &TOUCHES_RETOUR),
            ("son", &TOUCHES_SON),
        ]);
    }

    #[test]
    fn chaque_action_a_une_touche_independante_de_la_disposition() {
        // Une touche qui ne change pas de place entre QWERTY et AZERTY.
        let fixes = [
            KeyCode::Left,
            KeyCode::Right,
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::Space,
            KeyCode::Enter,
            KeyCode::Escape,
        ];
        let actions: [(&str, &[KeyCode]); 7] = [
            ("gauche", &TOUCHES_GAUCHE),
            ("droite", &TOUCHES_DROITE),
            ("saut", &TOUCHES_SAUT),
            ("pause", &TOUCHES_PAUSE),
            ("haut", &TOUCHES_HAUT),
            ("bas", &TOUCHES_BAS),
            ("valider", &TOUCHES_VALIDER),
        ];
        for (nom, touches) in actions {
            assert!(
                touches.iter().any(|k| fixes.contains(k)),
                "« {nom} » n'a que des lettres : injouable sur une disposition"
            );
        }
        // Retour : Échap. Le son n'a que des lettres : voir le test AZERTY.
        assert!(TOUCHES_RETOUR.contains(&KeyCode::Escape));
    }

    #[test]
    fn azerty_les_lettres_imprimees_tombent_sur_une_touche_acceptee() {
        // Position physique (nom QWERTY) de la lettre imprimée sur un clavier AZERTY.
        let azerty = |lettre: char| match lettre {
            'A' => KeyCode::Q,
            'Q' => KeyCode::A,
            'Z' => KeyCode::W,
            'W' => KeyCode::Z,
            'M' => KeyCode::Semicolon,
            'D' => KeyCode::D,
            'S' => KeyCode::S,
            'P' => KeyCode::P,
            autre => panic!("lettre non prévue : {autre}"),
        };
        // Gauche : A et Q (ZQSD ou WASD) ; droite : D ; saut : Z et W.
        for l in ['A', 'Q'] {
            assert!(TOUCHES_GAUCHE.contains(&azerty(l)), "gauche : {l}");
        }
        assert!(TOUCHES_DROITE.contains(&azerty('D')));
        for l in ['Z', 'W'] {
            assert!(TOUCHES_SAUT.contains(&azerty(l)), "saut : {l}");
            assert!(TOUCHES_HAUT.contains(&azerty(l)), "menu haut : {l}");
        }
        assert!(TOUCHES_BAS.contains(&azerty('S')));
        assert!(TOUCHES_PAUSE.contains(&azerty('P')));
        assert!(TOUCHES_SON.contains(&azerty('M')), "M sur AZERTY");
    }

    #[test]
    fn qwerty_la_touche_m_reste_acceptee() {
        assert!(TOUCHES_SON.contains(&KeyCode::M));
        assert!(TOUCHES_SON.contains(&KeyCode::Semicolon));
    }

    #[test]
    fn sans_la_fonctionnalite_aucune_touche_ne_saute_de_niveau() {
        // Même avec TOUTES les touches enfoncées : rien.
        assert_eq!(niveau_demande(false, |_| true), None);
        for k in TOUCHES_NIVEAUX {
            assert_eq!(niveau_demande(false, |t| t == k), None, "{k:?}");
        }
    }

    #[test]
    fn avec_la_fonctionnalite_chaque_touche_donne_son_niveau() {
        for (i, k) in TOUCHES_NIVEAUX.into_iter().enumerate() {
            assert_eq!(niveau_demande(true, |t| t == k), Some(i));
        }
        assert_eq!(niveau_demande(true, |_| false), None);
    }

    #[cfg(not(feature = "debug_niveaux"))]
    #[test]
    fn le_build_de_production_n_a_pas_de_raccourci() {
        // Clavier où TOUT est enfoncé, avec le réglage réel du build : aucun saut.
        assert_eq!(niveau_demande(RACCOURCIS_NIVEAUX, |_| true), None);
    }

    #[cfg(feature = "debug_niveaux")]
    #[test]
    fn le_build_de_mise_au_point_a_les_raccourcis() {
        // Clavier où seule la touche 3 est enfoncée : niveau 3 (indice 2).
        assert_eq!(
            niveau_demande(RACCOURCIS_NIVEAUX, |k| k == KeyCode::Key3),
            Some(2)
        );
    }

    #[test]
    fn le_plantage_volontaire_ne_part_que_de_f9_et_avec_la_fonctionnalite() {
        assert!(!plantage_demande(false, |_| true), "désactivé : jamais");
        assert!(plantage_demande(true, |k| k == KeyCode::F9));
        // Aucune touche de jeu ni de menu ne le déclenche.
        let autres = TOUCHES_GAUCHE
            .iter()
            .chain(&TOUCHES_DROITE)
            .chain(&TOUCHES_SAUT)
            .chain(&TOUCHES_PAUSE)
            .chain(&TOUCHES_SON)
            .chain(&TOUCHES_HAUT)
            .chain(&TOUCHES_BAS)
            .chain(&TOUCHES_VALIDER)
            .chain(&TOUCHES_RETOUR)
            .chain(&TOUCHES_NIVEAUX);
        for &k in autres {
            assert!(!plantage_demande(true, |t| t == k), "{k:?}");
        }
    }

    #[cfg(not(feature = "qa_plantage"))]
    #[test]
    fn le_build_de_production_ne_plante_pas_sur_f9() {
        assert!(!plantage_demande(PLANTAGE_QA, |_| true));
    }
}
