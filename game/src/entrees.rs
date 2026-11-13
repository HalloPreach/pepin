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

/// Les raccourcis de niveau (touches 1 à 9, niveau suivant / précédent) n'existent
/// que dans un build de mise au point : fonctionnalité Cargo `debug_niveaux`, absente
/// de la production. En production, un appui accidentel ne doit jamais perdre la
/// partie ni permettre de finir avec un temps truqué (sur AZERTY, les touches 1 à 9
/// sont & é " ' ( - è _ ç).
pub const RACCOURCIS_NIVEAUX: bool = cfg!(feature = "debug_niveaux");

/// Changement de niveau demandé par un raccourci du build de QA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaccourciNiveau {
    /// Aller au niveau d'indice donné (touches 1 à 9 : indices 0 à 8).
    Aller(usize),
    /// Niveau suivant dans la table (bonus compris, sans condition de gouttes).
    Suivant,
    /// Niveau précédent dans la table.
    Precedent,
    /// Niveau d'essai vertical (touche 0), hors de la table : carte 5.2.
    Essai,
    /// Niveau d'essai « vent » (touche F1), hors de la table : carte 5.3.
    Essai2,
    /// Niveau d'essai « grêle » (touche F2), hors de la table : carte 5.4.
    Essai3,
    /// Niveau d'essai « auvent » (touche F3), hors de la table : carte 5.5.
    Essai4,
    /// Niveau d'essai « ventilateur » (touche F4), hors de la table : carte 5.6.
    Essai5,
    /// Niveau d'essai « montgolfière » (touche F5), hors de la table : carte 5.7.
    Essai6,
    /// Niveau d'essai « escargot » (touche F6), hors de la table : carte 5.8.
    Essai7,
    /// Niveau d'essai « chat » (touche F7), hors de la table : carte 5.9.
    Essai8,
    /// Niveau d'essai « horloge » (touche F8), hors de la table : carte 5.10.
    Essai9,
    /// Niveau d'essai « nuage » (touche F10), hors de la table : carte 5.11.
    Essai10,
}

/// Touches de saut direct, dans l'ordre des niveaux 1 à 9. Une touche au-delà du
/// nombre de niveaux ne fait rien (`cible_raccourci`).
const TOUCHES_NIVEAUX: [KeyCode; 9] = [
    KeyCode::Key1,
    KeyCode::Key2,
    KeyCode::Key3,
    KeyCode::Key4,
    KeyCode::Key5,
    KeyCode::Key6,
    KeyCode::Key7,
    KeyCode::Key8,
    KeyCode::Key9,
];
/// Niveau suivant : Page suivante, ou N (même place en QWERTY et en AZERTY), pour
/// les claviers sans touches de page. Utilisable quel que soit le nombre de niveaux.
const TOUCHES_SUIVANT: [KeyCode; 2] = [KeyCode::PageDown, KeyCode::N];
/// Niveau précédent : Page précédente, ou B (à gauche de N).
const TOUCHES_PRECEDENT: [KeyCode; 2] = [KeyCode::PageUp, KeyCode::B];
/// Niveau d'essai vertical : 0 (« à » en AZERTY), juste après les touches 1 à 9.
const TOUCHES_ESSAI: [KeyCode; 1] = [KeyCode::Key0];
/// Niveau d'essai « vent » : F1, loin des touches de jeu (aucun appui
/// accidentel ; même emplacement en QWERTY et en AZERTY).
const TOUCHES_ESSAI2: [KeyCode; 1] = [KeyCode::F1];
/// Niveau d'essai « grêle » : F2, à côté de F1 (même raison).
const TOUCHES_ESSAI3: [KeyCode; 1] = [KeyCode::F2];
/// Niveau d'essai « auvent » : F3, à côté de F2 (même raison).
const TOUCHES_ESSAI4: [KeyCode; 1] = [KeyCode::F3];
/// Niveau d'essai « ventilateur » : F4, à côté de F3 (même raison).
const TOUCHES_ESSAI5: [KeyCode; 1] = [KeyCode::F4];
/// Niveau d'essai « montgolfière » : F5, à côté de F4 (même raison).
const TOUCHES_ESSAI6: [KeyCode; 1] = [KeyCode::F5];
/// Niveau d'essai « escargot » : F6, à côté de F5 (même raison).
const TOUCHES_ESSAI7: [KeyCode; 1] = [KeyCode::F6];
/// Niveau d'essai « chat » : F7, à côté de F6 (même raison).
const TOUCHES_ESSAI8: [KeyCode; 1] = [KeyCode::F7];
/// Niveau d'essai « horloge » : F8, à côté de F7 (même raison ; F9 reste au plantage).
const TOUCHES_ESSAI9: [KeyCode; 1] = [KeyCode::F8];
/// Niveau d'essai « nuage » : F10 (F9 est déjà le plantage volontaire).
const TOUCHES_ESSAI10: [KeyCode; 1] = [KeyCode::F10];

/// Raccourci demandé d'après les touches appuyées cette image (saut direct d'abord).
/// Toujours `None` quand `actif` est faux : la logique est isolée ici pour être
/// testée sans fenêtre, avec ou sans la fonctionnalité.
pub fn niveau_demande(actif: bool, appuyee: impl Fn(KeyCode) -> bool) -> Option<RaccourciNiveau> {
    if !actif {
        return None;
    }
    if let Some(i) = TOUCHES_NIVEAUX.iter().position(|&k| appuyee(k)) {
        Some(RaccourciNiveau::Aller(i))
    } else if TOUCHES_SUIVANT.iter().any(|&k| appuyee(k)) {
        Some(RaccourciNiveau::Suivant)
    } else if TOUCHES_PRECEDENT.iter().any(|&k| appuyee(k)) {
        Some(RaccourciNiveau::Precedent)
    } else if TOUCHES_ESSAI.iter().any(|&k| appuyee(k)) {
        Some(RaccourciNiveau::Essai)
    } else if TOUCHES_ESSAI2.iter().any(|&k| appuyee(k)) {
        Some(RaccourciNiveau::Essai2)
    } else if TOUCHES_ESSAI3.iter().any(|&k| appuyee(k)) {
        Some(RaccourciNiveau::Essai3)
    } else if TOUCHES_ESSAI4.iter().any(|&k| appuyee(k)) {
        Some(RaccourciNiveau::Essai4)
    } else if TOUCHES_ESSAI5.iter().any(|&k| appuyee(k)) {
        Some(RaccourciNiveau::Essai5)
    } else if TOUCHES_ESSAI6.iter().any(|&k| appuyee(k)) {
        Some(RaccourciNiveau::Essai6)
    } else if TOUCHES_ESSAI7.iter().any(|&k| appuyee(k)) {
        Some(RaccourciNiveau::Essai7)
    } else if TOUCHES_ESSAI8.iter().any(|&k| appuyee(k)) {
        Some(RaccourciNiveau::Essai8)
    } else if TOUCHES_ESSAI9.iter().any(|&k| appuyee(k)) {
        Some(RaccourciNiveau::Essai9)
    } else if TOUCHES_ESSAI10.iter().any(|&k| appuyee(k)) {
        Some(RaccourciNiveau::Essai10)
    } else {
        None
    }
}

/// Indice du niveau à charger pour le raccourci `r`, depuis le niveau `courant`, dans
/// un jeu de `nb` niveaux ; `None` : rien à faire (touche 7 avec 5 niveaux, « suivant »
/// au dernier niveau, « précédent » au premier). Le niveau d'essai a l'indice `nb`,
/// juste après la table (« suivant » et « précédent » ne le rencontrent jamais). Pure.
pub fn cible_raccourci(r: RaccourciNiveau, courant: usize, nb: usize) -> Option<usize> {
    let cible = match r {
        RaccourciNiveau::Aller(i) => Some(i),
        RaccourciNiveau::Suivant => courant.checked_add(1),
        RaccourciNiveau::Precedent => courant.checked_sub(1),
        RaccourciNiveau::Essai => return Some(nb),
        RaccourciNiveau::Essai2 => return Some(nb + 1),
        RaccourciNiveau::Essai3 => return Some(nb + 2),
        RaccourciNiveau::Essai4 => return Some(nb + 3),
        RaccourciNiveau::Essai5 => return Some(nb + 4),
        RaccourciNiveau::Essai6 => return Some(nb + 5),
        RaccourciNiveau::Essai7 => return Some(nb + 6),
        RaccourciNiveau::Essai8 => return Some(nb + 7),
        RaccourciNiveau::Essai9 => return Some(nb + 8),
        RaccourciNiveau::Essai10 => return Some(nb + 9),
    };
    cible.filter(|&i| i < nb)
}

/// Raccourci de mise au point : touches 1 à 9, niveau suivant ou précédent
/// (uniquement avec la fonctionnalité `debug_niveaux`).
pub fn raccourci_niveau() -> Option<RaccourciNiveau> {
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

    /// Toutes les touches de raccourci de niveau.
    fn touches_de_niveau() -> impl Iterator<Item = KeyCode> {
        TOUCHES_NIVEAUX
            .into_iter()
            .chain(TOUCHES_SUIVANT)
            .chain(TOUCHES_PRECEDENT)
            .chain(TOUCHES_ESSAI)
            .chain(TOUCHES_ESSAI2)
            .chain(TOUCHES_ESSAI3)
            .chain(TOUCHES_ESSAI4)
            .chain(TOUCHES_ESSAI5)
            .chain(TOUCHES_ESSAI6)
            .chain(TOUCHES_ESSAI7)
            .chain(TOUCHES_ESSAI8)
            .chain(TOUCHES_ESSAI9)
            .chain(TOUCHES_ESSAI10)
    }

    #[test]
    fn sans_la_fonctionnalite_aucune_touche_ne_saute_de_niveau() {
        // Même avec TOUTES les touches enfoncées : rien.
        assert_eq!(niveau_demande(false, |_| true), None);
        for k in touches_de_niveau() {
            assert_eq!(niveau_demande(false, |t| t == k), None, "{k:?}");
        }
    }

    #[test]
    fn avec_la_fonctionnalite_chaque_touche_donne_son_niveau() {
        for (i, k) in TOUCHES_NIVEAUX.into_iter().enumerate() {
            assert_eq!(
                niveau_demande(true, |t| t == k),
                Some(RaccourciNiveau::Aller(i))
            );
        }
        for k in TOUCHES_SUIVANT {
            assert_eq!(
                niveau_demande(true, |t| t == k),
                Some(RaccourciNiveau::Suivant)
            );
        }
        for k in TOUCHES_PRECEDENT {
            assert_eq!(
                niveau_demande(true, |t| t == k),
                Some(RaccourciNiveau::Precedent)
            );
        }
        assert_eq!(
            niveau_demande(true, |t| t == KeyCode::Key0),
            Some(RaccourciNiveau::Essai)
        );
        assert_eq!(
            niveau_demande(true, |t| t == KeyCode::F1),
            Some(RaccourciNiveau::Essai2)
        );
        assert_eq!(
            niveau_demande(true, |t| t == KeyCode::F2),
            Some(RaccourciNiveau::Essai3)
        );
        assert_eq!(
            niveau_demande(true, |t| t == KeyCode::F3),
            Some(RaccourciNiveau::Essai4)
        );
        assert_eq!(
            niveau_demande(true, |t| t == KeyCode::F4),
            Some(RaccourciNiveau::Essai5)
        );
        assert_eq!(
            niveau_demande(true, |t| t == KeyCode::F5),
            Some(RaccourciNiveau::Essai6)
        );
        assert_eq!(
            niveau_demande(true, |t| t == KeyCode::F6),
            Some(RaccourciNiveau::Essai7)
        );
        assert_eq!(
            niveau_demande(true, |t| t == KeyCode::F7),
            Some(RaccourciNiveau::Essai8)
        );
        assert_eq!(
            niveau_demande(true, |t| t == KeyCode::F8),
            Some(RaccourciNiveau::Essai9)
        );
        assert_eq!(
            niveau_demande(true, |t| t == KeyCode::F10),
            Some(RaccourciNiveau::Essai10)
        );
        assert_eq!(niveau_demande(true, |_| false), None);
    }

    /// Les raccourcis ne prennent aucune touche de jeu ni de menu.
    #[test]
    fn les_raccourcis_de_niveau_ne_touchent_a_aucune_commande() {
        let commandes: Vec<KeyCode> = TOUCHES_GAUCHE
            .iter()
            .chain(&TOUCHES_DROITE)
            .chain(&TOUCHES_SAUT)
            .chain(&TOUCHES_PAUSE)
            .chain(&TOUCHES_SON)
            .chain(&TOUCHES_HAUT)
            .chain(&TOUCHES_BAS)
            .chain(&TOUCHES_VALIDER)
            .chain(&TOUCHES_RETOUR)
            .copied()
            .collect();
        for k in touches_de_niveau() {
            assert!(!commandes.contains(&k), "{k:?}");
        }
    }

    /// Au-delà de 9 niveaux, « suivant » et « précédent » atteignent tous les niveaux ;
    /// les touches au-delà du nombre de niveaux ne font rien.
    #[test]
    fn cible_des_raccourcis() {
        use RaccourciNiveau::{Aller, Precedent, Suivant};
        // 11 niveaux : du premier au dernier avec « suivant », et retour.
        let mut i = 0;
        for attendu in 1..11 {
            i = cible_raccourci(Suivant, i, 11).unwrap();
            assert_eq!(i, attendu);
        }
        assert_eq!(
            cible_raccourci(Suivant, 10, 11),
            None,
            "rien après le dernier"
        );
        assert_eq!(cible_raccourci(Precedent, 10, 11), Some(9));
        assert_eq!(
            cible_raccourci(Precedent, 0, 11),
            None,
            "rien avant le premier"
        );
        assert_eq!(cible_raccourci(Aller(8), 0, 11), Some(8));
        // 5 niveaux : les touches 1 à 5 comme avant, 6 à 9 sans effet.
        for k in 0..5 {
            assert_eq!(cible_raccourci(Aller(k), 2, 5), Some(k));
        }
        for k in 5..9 {
            assert_eq!(cible_raccourci(Aller(k), 2, 5), None);
        }
        assert_eq!(cible_raccourci(Suivant, 4, 5), None);
        // Niveau d'essai : juste après la table, depuis n'importe quel niveau ; de là,
        // « précédent » ramène au dernier niveau et « suivant » ne fait rien.
        for nb in [5, 11] {
            for courant in [0, nb - 1, nb] {
                assert_eq!(
                    cible_raccourci(RaccourciNiveau::Essai, courant, nb),
                    Some(nb)
                );
            }
            assert_eq!(cible_raccourci(Precedent, nb, nb), Some(nb - 1));
            assert_eq!(cible_raccourci(Suivant, nb, nb), None);
            // Le niveau d'essai « vent » : juste après le premier (carte 5.3).
            for courant in [0, nb - 1, nb] {
                assert_eq!(
                    cible_raccourci(RaccourciNiveau::Essai2, courant, nb),
                    Some(nb + 1)
                );
            }
            // Le niveau d'essai « grêle » : juste après celui du vent (carte 5.4).
            for courant in [0, nb - 1, nb, nb + 1] {
                assert_eq!(
                    cible_raccourci(RaccourciNiveau::Essai3, courant, nb),
                    Some(nb + 2)
                );
            }
            // Le niveau d'essai « auvent » : juste après celui de la grêle (carte 5.5).
            for courant in [0, nb - 1, nb, nb + 2] {
                assert_eq!(
                    cible_raccourci(RaccourciNiveau::Essai4, courant, nb),
                    Some(nb + 3)
                );
            }
            // Le niveau d'essai « ventilateur » : juste après celui de l'auvent
            // (carte 5.6).
            for courant in [0, nb - 1, nb, nb + 2, nb + 3] {
                assert_eq!(
                    cible_raccourci(RaccourciNiveau::Essai5, courant, nb),
                    Some(nb + 4)
                );
            }
            // Le niveau d'essai « montgolfière » : juste après celui du ventilateur
            // (carte 5.7).
            for courant in [0, nb - 1, nb, nb + 3, nb + 4] {
                assert_eq!(
                    cible_raccourci(RaccourciNiveau::Essai6, courant, nb),
                    Some(nb + 5)
                );
            }
            // Le niveau d'essai « escargot » : juste après celui des montgolfières
            // (carte 5.8).
            for courant in [0, nb - 1, nb, nb + 4, nb + 5] {
                assert_eq!(
                    cible_raccourci(RaccourciNiveau::Essai7, courant, nb),
                    Some(nb + 6)
                );
            }
            // Le niveau d'essai « chat » : juste après celui des escargots (carte 5.9).
            for courant in [0, nb - 1, nb, nb + 5, nb + 6] {
                assert_eq!(
                    cible_raccourci(RaccourciNiveau::Essai8, courant, nb),
                    Some(nb + 7)
                );
            }
            // Le niveau d'essai « horloge » : juste après celui du chat (carte 5.10).
            for courant in [0, nb - 1, nb, nb + 6, nb + 7] {
                assert_eq!(
                    cible_raccourci(RaccourciNiveau::Essai9, courant, nb),
                    Some(nb + 8)
                );
            }
            // Le niveau d'essai « nuage » : juste après celui de l'horloge (carte 5.11).
            for courant in [0, nb - 1, nb, nb + 7, nb + 8] {
                assert_eq!(
                    cible_raccourci(RaccourciNiveau::Essai10, courant, nb),
                    Some(nb + 9)
                );
            }
        }
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
            Some(RaccourciNiveau::Aller(2))
        );
        assert_eq!(
            niveau_demande(RACCOURCIS_NIVEAUX, |k| k == KeyCode::PageDown),
            Some(RaccourciNiveau::Suivant)
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
            .copied()
            .chain(touches_de_niveau());
        for k in autres {
            assert!(!plantage_demande(true, |t| t == k), "{k:?}");
        }
    }

    #[cfg(not(feature = "qa_plantage"))]
    #[test]
    fn le_build_de_production_ne_plante_pas_sur_f9() {
        assert!(!plantage_demande(PLANTAGE_QA, |_| true));
    }
}
