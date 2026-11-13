//! Le chat endormi (lettre `z`) : immobile sur un toit, il dort, se réveille
//! brièvement, puis se rendort, en boucle. Le réveil est ANNONCÉ, comme un éclair :
//!
//! ```text
//!   phase :  Dort (4 s) ──> Réveil (0,8 s, l'annonce) ──> Éveillé (1,2 s) ──┐
//!              ^                                                             │
//!              └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! - Endormi (`Dort`) ou en train de se réveiller (`Reveil`) : c'est un ennemi
//!   comme les autres. On l'écrase en lui sautant dessus ; le heurter de côté tue
//!   (il se réveille en sursaut et griffe).
//! - Éveillé (`Eveille`) : on ne peut PAS l'écraser. Le toucher, même par le
//!   dessus, tue Pépin.
//!
//! Il ne bouge jamais : il garde un passage (dans le jeu, l'accès à une goutte
//! cachée). Il faut attendre qu'il dorme, puis lui sauter dessus.
//!
//! Comme le pigeon, ce fichier est un SOUS-MODULE de `ennemi` : il lit et modifie
//! les champs privés de `Ennemi` sans les rendre publics.
//!
//! Notions Rust : un `enum` à données (`PhaseChat`) dont chaque variante porte
//! son compteur ; une fonction pure (`phase_chat_suivante`) qui contient toute
//! la machine à états ; des durées en ENTIERS (pas de 1/60 s), exactement
//! reproductibles.

use super::{Ennemi, EtatEnnemi, TypeEnnemi};

/// Durée du sommeil : 4 s (« plusieurs secondes »), le temps de lui sauter dessus.
pub const DORT_PAS: u16 = 240;
/// Durée de l'annonce du réveil : 0,8 s, comme l'annonce d'un éclair.
pub const REVEIL_PAS: u16 = 48;
/// Durée de l'éveil : 1,2 s (« il se réveille brièvement »).
pub const EVEILLE_PAS: u16 = 72;
/// Période complète du cycle : 6 s (360 pas).
pub const PERIODE_CHAT_PAS: u16 = DORT_PAS + REVEIL_PAS + EVEILLE_PAS;

/// Où en est le chat dans son cycle. Chaque variante porte le nombre de pas
/// qui lui restent (celui-ci compris) : `Dort(1)` est le dernier pas de sommeil.
/// Les autres ennemis gardent toujours `Dort(DORT_PAS)`, qui ne leur sert à rien.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseChat {
    /// Il dort (« zzz ») : écrasable.
    Dort(u16),
    /// Il se réveille (l'annonce) : encore écrasable.
    Reveil(u16),
    /// Il est réveillé : intouchable, le toucher tue.
    Eveille(u16),
}

/// Phase suivante (un pas de 1/60 s). Fonction pure : toute la machine à états
/// tient dans ce `match`.
pub fn phase_chat_suivante(phase: PhaseChat) -> PhaseChat {
    match phase {
        PhaseChat::Dort(n) if n > 1 => PhaseChat::Dort(n - 1),
        PhaseChat::Dort(_) => PhaseChat::Reveil(REVEIL_PAS),
        PhaseChat::Reveil(n) if n > 1 => PhaseChat::Reveil(n - 1),
        PhaseChat::Reveil(_) => PhaseChat::Eveille(EVEILLE_PAS),
        PhaseChat::Eveille(n) if n > 1 => PhaseChat::Eveille(n - 1),
        PhaseChat::Eveille(_) => PhaseChat::Dort(DORT_PAS),
    }
}

/// Pas écoulés depuis le début du sommeil en cours : où en est le chat dans son
/// cycle, de 0 (il vient de s'endormir) à `PERIODE_CHAT_PAS - 1` (il va se
/// rendormir). Fonction pure : le robot de QA s'en sert pour savoir combien de
/// temps attendre (voir `game/src/bot_qa.rs`).
pub fn pas_du_cycle(phase: PhaseChat) -> u16 {
    match phase {
        PhaseChat::Dort(n) => DORT_PAS - n,
        PhaseChat::Reveil(n) => DORT_PAS + REVEIL_PAS - n,
        PhaseChat::Eveille(n) => PERIODE_CHAT_PAS - n,
    }
}

impl Ennemi {
    /// Pose un chat endormi dont le coin haut-gauche est en `(x, y)`. Son premier
    /// sommeil est raccourci de `decalage_pas` (borné : il dort au moins 1 pas) :
    /// deux chats d'un même niveau ne se réveillent pas en même temps.
    pub fn chat(x: f32, y: f32, decalage_pas: u16) -> Self {
        let mut c = Ennemi::new(TypeEnnemi::Chat, x, y);
        c.sommeil = PhaseChat::Dort(DORT_PAS - decalage_pas.min(DORT_PAS - 1));
        c
    }

    /// Phase du cycle du chat (toujours `Dort(DORT_PAS)` pour un autre ennemi).
    pub fn phase_chat(&self) -> PhaseChat {
        self.sommeil
    }

    /// Le chat est-il réveillé (intouchable) ? Toujours faux pour un autre ennemi.
    pub fn est_eveille(&self) -> bool {
        self.type_ennemi == TypeEnnemi::Chat && matches!(self.sommeil, PhaseChat::Eveille(_))
    }

    /// Avancement de l'annonce du réveil entre 0.0 (début) et 1.0 (réveil
    /// imminent), pour faire clignoter plus vite à l'écran (comme l'éclair).
    /// 0.0 hors de l'annonce.
    pub fn progression_reveil(&self) -> f32 {
        match self.sommeil {
            PhaseChat::Reveil(n) => 1.0 - f32::from(n) / f32::from(REVEIL_PAS),
            _ => 0.0,
        }
    }

    /// Peut-on l'écraser en lui tombant dessus ? Un ennemi actif, sauf un chat
    /// réveillé.
    pub fn est_ecrasable(&self) -> bool {
        self.etat == EtatEnnemi::Actif && !self.est_eveille()
    }

    /// Un pas du chat : il ne bouge pas, seul son cycle avance.
    pub(super) fn dormir(&mut self) {
        self.sommeil = phase_chat_suivante(self.sommeil);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::GrilleTuiles;
    use crate::entities::ennemi::ANIMATION_ECRASE_PAS;
    use crate::entities::Actualisable;
    use crate::physics::Environnement;

    fn avancer(c: &mut Ennemi, pas: u32) {
        let g = GrilleTuiles::vide(8, 8);
        let env = Environnement::simple(&g);
        for _ in 0..pas {
            c.actualiser(&env);
        }
    }

    #[test]
    fn le_cycle_dure_ce_qu_annoncent_les_constantes() {
        assert_eq!(
            (DORT_PAS, REVEIL_PAS, EVEILLE_PAS, PERIODE_CHAT_PAS),
            (240, 48, 72, 360)
        );
        let mut c = Ennemi::chat(32.0, 40.0, 0);
        assert_eq!(c.phase_chat(), PhaseChat::Dort(DORT_PAS));
        // Compte des pas passés dans chaque phase sur trois périodes.
        let (mut dort, mut reveil, mut eveille) = (0u32, 0u32, 0u32);
        for _ in 0..3 * u32::from(PERIODE_CHAT_PAS) {
            match c.phase_chat() {
                PhaseChat::Dort(_) => dort += 1,
                PhaseChat::Reveil(_) => reveil += 1,
                PhaseChat::Eveille(_) => eveille += 1,
            }
            avancer(&mut c, 1);
        }
        assert_eq!((dort, reveil, eveille), (3 * 240, 3 * 48, 3 * 72));
        // Retour exact au point de départ après un nombre entier de périodes.
        assert_eq!(c.phase_chat(), PhaseChat::Dort(DORT_PAS));
    }

    #[test]
    fn l_ordre_des_phases_est_dort_reveil_eveille() {
        let mut c = Ennemi::chat(0.0, 0.0, 0);
        avancer(&mut c, u32::from(DORT_PAS) - 1);
        assert_eq!(c.phase_chat(), PhaseChat::Dort(1));
        avancer(&mut c, 1);
        assert_eq!(c.phase_chat(), PhaseChat::Reveil(REVEIL_PAS));
        avancer(&mut c, u32::from(REVEIL_PAS));
        assert_eq!(c.phase_chat(), PhaseChat::Eveille(EVEILLE_PAS));
        avancer(&mut c, u32::from(EVEILLE_PAS));
        assert_eq!(c.phase_chat(), PhaseChat::Dort(DORT_PAS));
    }

    /// `pas_du_cycle` compte les pas depuis l'endormissement : 0, 1, 2… jusqu'à
    /// `PERIODE_CHAT_PAS - 1`, puis retour à 0 ; un chat décalé part plus loin.
    #[test]
    fn le_pas_du_cycle_avance_d_un_a_chaque_pas() {
        let mut c = Ennemi::chat(0.0, 0.0, 0);
        for attendu in (0..PERIODE_CHAT_PAS).chain(0..3) {
            assert_eq!(
                pas_du_cycle(c.phase_chat()),
                attendu,
                "{:?}",
                c.phase_chat()
            );
            avancer(&mut c, 1);
        }
        assert_eq!(pas_du_cycle(PhaseChat::Reveil(REVEIL_PAS)), DORT_PAS);
        assert_eq!(
            pas_du_cycle(PhaseChat::Eveille(EVEILLE_PAS)),
            DORT_PAS + REVEIL_PAS
        );
        assert_eq!(pas_du_cycle(Ennemi::chat(0.0, 0.0, 40).phase_chat()), 40);
    }

    #[test]
    fn ecrasable_endormi_et_pendant_l_annonce_jamais_eveille() {
        let mut c = Ennemi::chat(0.0, 0.0, 0);
        for _ in 0..PERIODE_CHAT_PAS {
            let eveille = matches!(c.phase_chat(), PhaseChat::Eveille(_));
            assert_eq!(c.est_eveille(), eveille);
            assert_eq!(c.est_ecrasable(), !eveille, "{:?}", c.phase_chat());
            // Éveillé ou non, il reste dangereux au toucher (de côté).
            assert!(c.est_dangereux());
            avancer(&mut c, 1);
        }
    }

    #[test]
    fn le_chat_ne_bouge_jamais() {
        let mut c = Ennemi::chat(48.0, 40.0, 0);
        for _ in 0..2 * PERIODE_CHAT_PAS {
            avancer(&mut c, 1);
            assert_eq!((c.boite.x, c.boite.y), (48.0, 40.0));
        }
    }

    #[test]
    fn le_decalage_raccourcit_le_premier_sommeil() {
        let c = Ennemi::chat(0.0, 0.0, 100);
        assert_eq!(c.phase_chat(), PhaseChat::Dort(DORT_PAS - 100));
        // Borné : un décalage trop grand laisse quand même 1 pas de sommeil.
        let c = Ennemi::chat(0.0, 0.0, 1000);
        assert_eq!(c.phase_chat(), PhaseChat::Dort(1));
    }

    #[test]
    fn la_progression_du_reveil_va_de_0_a_1() {
        let mut c = Ennemi::chat(0.0, 0.0, 0);
        assert_eq!(c.progression_reveil(), 0.0);
        avancer(&mut c, u32::from(DORT_PAS));
        assert!(c.progression_reveil() < 0.05);
        avancer(&mut c, u32::from(REVEIL_PAS) - 1);
        assert!(c.progression_reveil() > 0.95);
        avancer(&mut c, 1);
        assert_eq!(c.progression_reveil(), 0.0, "éveillé : plus d'annonce");
    }

    #[test]
    fn un_chat_ecrase_ne_se_reveille_plus_et_disparait() {
        let mut c = Ennemi::chat(0.0, 0.0, 0);
        c.ecraser();
        let phase = c.phase_chat();
        assert!(!c.est_ecrasable() && !c.est_dangereux());
        avancer(&mut c, u32::from(ANIMATION_ECRASE_PAS) - 1);
        assert_eq!(c.phase_chat(), phase, "écrasé, son cycle s'arrête");
        assert!(c.est_visible());
        avancer(&mut c, 1);
        assert!(!c.est_visible());
    }

    // --- Mini-niveau de test : le chat qui garde une goutte cachée ---

    /// Le chat (colonne 5) dort devant une niche basse (colonnes 6 à 12, sous un
    /// plafond de nuages) où attend une goutte (colonne 10). Le dessus du chat
    /// est libre : on peut lui sauter dessus. Par la niche, impossible de passer
    /// au-dessus de lui (la tête cognerait le plafond) : il faut l'écraser.
    const NICHE: &str = "
o.o...........
..............
..............
......NNNNNNN.
.@...z....o..X
##############
";

    /// Le mini-niveau, ses entités et Pépin, avancés pas à pas dans l'ordre du
    /// jeu (`Monde::avancer`) : entités, joueur, contacts.
    struct Banc {
        niveau: crate::level::Niveau,
        entites: Vec<crate::entities::Entite>,
        joueur: crate::player::Joueur,
    }

    impl Banc {
        /// Pépin en `(x, y)` (coin haut-gauche) ; le chat a déjà vécu `pas_chat` pas.
        fn new(x: f32, y: f32, pas_chat: u32) -> Self {
            let niveau = crate::level::Niveau::depuis_ascii(NICHE).expect("mini-niveau valide");
            let entites = niveau.entites.clone();
            let mut banc = Self {
                niveau,
                entites,
                joueur: crate::player::Joueur::new(x, y),
            };
            let g = GrilleTuiles::vide(4, 4);
            let env = Environnement::simple(&g);
            for _ in 0..pas_chat {
                for e in &mut banc.entites {
                    e.actualiser(&env);
                }
            }
            banc
        }

        fn chat(&self) -> &Ennemi {
            self.entites
                .iter()
                .find_map(|e| match e {
                    crate::entities::Entite::Ennemi(c) => Some(c),
                    _ => None,
                })
                .expect("un chat dans le mini-niveau")
        }

        /// Un pas ; renvoie les contacts.
        fn pas(
            &mut self,
            entrees: &crate::player::Entrees,
        ) -> Vec<crate::entities::contacts::Contact> {
            let solides = crate::entities::solides_des_entites(&self.entites);
            let env = Environnement {
                grille: &self.niveau,
                solides: &solides,
                courants: &[],
                vents: &[],
            };
            for e in &mut self.entites {
                e.actualiser(&env);
            }
            self.joueur.actualiser(entrees, &env);
            crate::entities::contacts::resoudre_contacts(&mut self.joueur, &mut self.entites)
        }

        /// `n` pas avec `entrees` ; renvoie tous les contacts, dans l'ordre.
        fn jouer(
            &mut self,
            entrees: &crate::player::Entrees,
            n: u32,
        ) -> Vec<crate::entities::contacts::Contact> {
            (0..n).flat_map(|_| self.pas(entrees)).collect()
        }
    }

    fn droite() -> crate::player::Entrees {
        crate::player::Entrees {
            direction: crate::player::Direction::Droite,
            ..Default::default()
        }
    }

    fn mort(contacts: &[crate::entities::contacts::Contact]) -> bool {
        use crate::entities::contacts::{CauseMort, Contact};
        contacts.contains(&Contact::Mort(CauseMort::Ennemi))
    }

    #[test]
    fn la_niche_est_gardee_par_le_chat() {
        let b = Banc::new(24.0, 50.0, 0);
        let c = b.chat();
        assert_eq!((c.boite.x, c.boite.y), (81.0, 72.0));
        assert_eq!(c.phase_chat(), PhaseChat::Dort(DORT_PAS));
    }

    #[test]
    fn foncer_dans_le_chat_endormi_tue() {
        // Pépin part du départ et court vers la niche : il heurte le chat de côté.
        let mut b = Banc::new(24.0, 66.0, 0);
        let contacts = b.jouer(&droite(), 60);
        assert!(mort(&contacts), "{contacts:?}");
        assert!(b.chat().est_dangereux(), "le chat n'est pas écrasé");
    }

    #[test]
    fn tomber_sur_le_chat_endormi_l_ecrase_et_ouvre_la_niche() {
        use crate::entities::contacts::Contact;
        // Pépin lâché au-dessus du dos du chat endormi : il tombe dessus.
        let mut b = Banc::new(83.0, 30.0, 0);
        let contacts = b.jouer(&Default::default(), 40);
        assert!(contacts.contains(&Contact::Ecrase), "{contacts:?}");
        assert!(!mort(&contacts));
        assert!(!b.chat().est_dangereux());
        // Le chat disparu, Pépin entre dans la niche et prend la goutte cachée.
        b.jouer(&Default::default(), u32::from(ANIMATION_ECRASE_PAS));
        let contacts = b.jouer(&droite(), 60);
        assert!(contacts.contains(&Contact::Goutte(2)), "{contacts:?}");
        assert!(!mort(&contacts));
    }

    #[test]
    fn tomber_sur_le_chat_eveille_tue() {
        use crate::entities::contacts::Contact;
        // Même chute, mais le chat est réveillé (dernier pas de l'annonce passé).
        let reveil = u32::from(DORT_PAS + REVEIL_PAS);
        let mut b = Banc::new(83.0, 30.0, reveil - 10);
        let contacts = b.jouer(&Default::default(), 40);
        assert!(mort(&contacts), "{contacts:?}");
        assert!(!contacts.contains(&Contact::Ecrase));
        assert!(b.chat().est_dangereux(), "un chat éveillé ne s'écrase pas");
    }

    #[test]
    fn les_autres_ennemis_ne_sont_jamais_eveilles() {
        for t in [TypeEnnemi::Flaque, TypeEnnemi::Corbeau, TypeEnnemi::Pigeon] {
            let mut e = Ennemi::new(t, 32.0, 32.0);
            for _ in 0..PERIODE_CHAT_PAS {
                assert!(!e.est_eveille() && e.est_ecrasable(), "{t:?}");
                avancer(&mut e, 1);
            }
        }
    }
}
