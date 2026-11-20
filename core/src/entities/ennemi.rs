//! Les ennemis : « flaques » qui patrouillent au sol, corbeaux qui volent en ligne
//! droite et font demi-tour au bout de leur portée (ou contre un mur), pigeons
//! perchés qui plongent sur Pépin quand il passe dessous (sous-module `pigeon`),
//! chats endormis qui se réveillent brièvement, en cycle, et bondissent sur Pépin
//! s'il est à portée à la fin de l'annonce (sous-module `chat`). Le Monde 2
//! (carte 9.4) ajoute les ennemis d'eau : grenouille, poisson, crabe, rat
//! (sous-module `eau`), et la mouette, un pigeon du port.
//!
//! Notions Rust : un `enum` pour le TYPE (`TypeEnnemi`) et un autre pour l'ÉTAT
//! (`EtatEnnemi`) ; l'état `Ecrase(u8)` PORTE une donnée (les pas d'animation restants) ;
//! `Option<f32>` pour une portée qui peut ne pas exister (`None` : sans limite).

mod chat;
mod eau;
mod pigeon;

pub use chat::{
    decalage_bond, decalage_chat, pas_du_cycle, phase_chat_suivante, PhaseChat, BOND_PAS, DORT_PAS,
    EVEILLE_PAS, PERIODE_CHAT_PAS, PORTEE_BOND_X, PORTEE_BOND_Y, POSE_BOND_PAS, REVEIL_PAS,
    VITESSE_BOND_Y,
};
pub use eau::{
    hauteur_saut_poisson, PhaseEau, ALERTE_CRABE, ANNONCE_GRENOUILLE_PAS, ANNONCE_RAT_PAS,
    CLAQUE_CRABE_PAS, HAUT_SAUT_POISSON, PERIODE_POISSON_PAS, PORTEE_GRENOUILLE, PORTEE_RAT,
    REMOUS_PAS, SAUT_POISSON_PAS, SOUS_L_EAU_PAS,
};
pub use pigeon::{
    decalage_vol, phase_suivante, PhasePigeon, ANNONCE_PIGEON_PAS, MARGE_CONE, PLONGEE_PAS,
    PROFONDEUR_PLONGEE, REMONTEE_PAS, REPOS_PIGEON_PAS,
};

use super::Actualisable;
use crate::aabb::{deplacer, Aabb, TAILLE_TUILE};
use crate::physics::{Environnement, DT, HZ};

/// Nombre de pas pendant lesquels une flaque écrasée reste affichée (aplatie).
pub const ANIMATION_ECRASE_PAS: u8 = 20;

/// Portée de patrouille du corbeau (px) : distance maximale entre sa position et
/// son point d'apparition, de chaque côté. Il fait demi-tour en l'atteignant, ou
/// plus tôt s'il rencontre un mur : le premier des deux atteint.
///
/// 5 tuiles de chaque côté = une patrouille de 10 tuiles (160 px, la moitié de
/// l'écran logique de 320 px), soit 4 s pour un aller à 40 px/s : on le voit
/// revenir, on peut donc l'attendre. Sans portée, dans un niveau ouvert (niveau 4,
/// sans murs en altitude), il traverserait toute la carte.
pub const PORTEE_CORBEAU: f32 = 5.0 * TAILLE_TUILE;

/// Espèce d'ennemi. `Copy` : un simple petit « tag », on peut le dupliquer sans souci.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeEnnemi {
    /// Flaque : glisse au sol, fait demi-tour aux murs et au bord du vide.
    Flaque,
    /// Corbeau : vole droit (ignore la gravité et le vide), fait demi-tour au bout
    /// de sa portée (`PORTEE_CORBEAU`) ou contre un mur.
    Corbeau,
    /// Pigeon : perché, il plonge en diagonale quand Pépin passe dessous, puis
    /// remonte se poser (voir le sous-module `pigeon`).
    Pigeon,
    /// Chat endormi : immobile ; il dort, se réveille brièvement (annoncé), se
    /// rendort. Éveillé, on ne peut pas l'écraser (voir le sous-module `chat`).
    Chat,
    /// Grenouille (Monde 2) : guette, se ramasse (annonce) et bondit en arc vers
    /// Pépin ; écrasable (sous-module `eau`).
    Grenouille,
    /// Poisson (Monde 2) : sous l'eau, saute à la verticale sur sa colonne après
    /// des remous ; mortel hors de l'eau, jamais écrasable.
    Poisson,
    /// Crabe (Monde 2) : patrouille ; carapace dure, on rebondit dessus sans le
    /// blesser ; de côté, il tue.
    Crabe,
    /// Rat (Monde 2) : patrouille son tunnel ; Pépin sur sa ligne, il se dresse
    /// (annonce) puis fonce jusqu'au mur ; écrasable.
    Rat,
    /// Mouette (Monde 2, le port) : un pigeon perché sur un mât ou une grue, même
    /// annonce, même piqué ; seul son dessin change.
    Mouette,
}

impl TypeEnnemi {
    /// Vitesse de déplacement horizontale (px/s).
    pub fn vitesse(self) -> f32 {
        match self {
            TypeEnnemi::Flaque => 24.0,
            TypeEnnemi::Corbeau => 40.0,
            // Vitesse de plongée, dans chaque axe (120 px/s).
            TypeEnnemi::Pigeon | TypeEnnemi::Mouette => {
                PROFONDEUR_PLONGEE / PLONGEE_PAS as f32 * HZ as f32
            }
            // Le chat ne bouge jamais. La grenouille et le poisson ont leur propre
            // trajectoire (bond, saut vertical).
            TypeEnnemi::Chat | TypeEnnemi::Grenouille | TypeEnnemi::Poisson => 0.0,
            TypeEnnemi::Crabe => eau::VITESSE_CRABE,
            TypeEnnemi::Rat => eau::VITESSE_RAT,
        }
    }

    /// Dimensions `(largeur, hauteur)` de la boîte de collision.
    pub fn taille(self) -> (f32, f32) {
        match self {
            TypeEnnemi::Flaque => (12.0, 8.0),
            TypeEnnemi::Corbeau => (12.0, 10.0),
            TypeEnnemi::Pigeon => (12.0, 10.0),
            // Roulé en boule : plus large que haut (le sprite fait 16 x 14).
            TypeEnnemi::Chat => (14.0, 8.0),
            TypeEnnemi::Grenouille | TypeEnnemi::Mouette => (12.0, 10.0),
            // Le poisson saute la tête en haut : plus haut que large.
            TypeEnnemi::Poisson => (10.0, 12.0),
            TypeEnnemi::Crabe => (14.0, 9.0),
            TypeEnnemi::Rat => (14.0, 8.0),
        }
    }

    /// Portée de patrouille par défaut (px de chaque côté du point d'apparition).
    ///
    /// `Option<f32>` : `None` veut dire « pas de limite ». La flaque n'en a pas
    /// besoin : le bord du vide et les murs la retiennent déjà sur son toit. Le
    /// pigeon non plus : il ne patrouille pas, il revient toujours à son perchoir.
    /// Le chat ne bouge pas.
    pub fn portee(self) -> Option<f32> {
        match self {
            TypeEnnemi::Corbeau => Some(PORTEE_CORBEAU),
            _ => None,
        }
    }
}

/// Cycle de vie d'un ennemi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtatEnnemi {
    /// Vivant et dangereux.
    Actif,
    /// Écrasé : inoffensif, encore visible `n` pas.
    Ecrase(u8),
    /// Terminé : à ne plus afficher.
    Disparu,
}

#[derive(Debug, Clone)]
pub struct Ennemi {
    pub type_ennemi: TypeEnnemi,
    pub boite: Aabb,
    pub etat: EtatEnnemi,
    /// Regarde (et avance) vers la droite ?
    pub vers_la_droite: bool,
    vitesse_y: f32,
    au_sol: bool,
    /// Abscisse (px) du coin haut-gauche à l'apparition : le centre de la patrouille.
    origine_x: f32,
    /// Ordonnée (px) du coin haut-gauche à l'apparition (le perchoir du pigeon).
    origine_y: f32,
    /// Distance maximale (px) à `origine_x`, de chaque côté (`None` : sans limite).
    portee: Option<f32>,
    /// Phase de vol du pigeon (toujours `Perche` pour les autres ennemis).
    phase: PhasePigeon,
    /// Cycle de sommeil du chat (toujours `Dort(DORT_PAS)` pour les autres).
    sommeil: PhaseChat,
    /// Chat : la boîte de Pépin vue au dernier pas (`guetter`) ; `None` avant
    /// le premier pas, et toujours pour les autres ennemis.
    proie: Option<Aabb>,
    /// Chat : pendant l'annonce, voit-il Pépin à portée de bond ? (Posture
    /// « prêt à bondir ».) Toujours faux pour les autres.
    pret: bool,
    /// Chat : longueur (px) du bond en cours pendant l'éveil, `None` s'il se
    /// réveille sur place. Toujours `None` pour les autres.
    bond: Option<f32>,
    /// Ennemis d'eau (carte 9.4) : leur phase (toujours `Guet` pour les autres).
    eau: PhaseEau,
    /// Grenouille : vitesse horizontale (px/s) du bond en cours, 0 sinon.
    elan: f32,
    /// Poisson : hauteur (px) de son saut, `HAUT_SAUT_POISSON` sauf réglage du
    /// niveau (carte 9.11e). Inutilisée pour les autres.
    saut_poisson: f32,
    /// Ce qui, au départ, n'est pas celui de `new` (sens du regard, premier
    /// sommeil du chat, premier repos et hauteur de saut du poisson) :
    /// `revenir_a_l_origine` le remet (cartes 9.29 et 9.11e).
    depart: Depart,
}

/// La part de l'état de départ d'un ennemi que `Ennemi::new` ne fixe pas seul :
/// les constructeurs (`pigeon`, `chat`, `poisson`) et le chargeur de niveau la
/// règlent. Tout le reste se reconstruit avec `new` (type, origine, portée).
#[derive(Debug, Clone, Copy)]
struct Depart {
    vers_la_droite: bool,
    sommeil: PhaseChat,
    eau: PhaseEau,
    saut_poisson: f32,
}

impl Ennemi {
    /// Crée un ennemi dont le coin haut-gauche est en `(x, y)`, qui part vers la gauche.
    /// Sa patrouille est centrée sur ce point, avec la portée par défaut de son type.
    pub fn new(type_ennemi: TypeEnnemi, x: f32, y: f32) -> Self {
        let (w, h) = type_ennemi.taille();
        Self {
            type_ennemi,
            boite: Aabb::new(x, y, w, h),
            etat: EtatEnnemi::Actif,
            vers_la_droite: false,
            vitesse_y: 0.0,
            au_sol: false,
            origine_x: x,
            origine_y: y,
            portee: type_ennemi.portee(),
            phase: PhasePigeon::Perche,
            sommeil: PhaseChat::Dort(DORT_PAS),
            proie: None,
            pret: false,
            bond: None,
            eau: PhaseEau::Guet,
            elan: 0.0,
            saut_poisson: HAUT_SAUT_POISSON,
            depart: Depart {
                vers_la_droite: false,
                sommeil: PhaseChat::Dort(DORT_PAS),
                eau: PhaseEau::Guet,
                saut_poisson: HAUT_SAUT_POISSON,
            },
        }
    }

    /// Fixe le sens du regard au départ : l'ennemi regarde de ce côté dès
    /// maintenant, et y regardera de nouveau après une mort de Pépin.
    pub fn regarder_au_depart(&mut self, vers_la_droite: bool) {
        self.vers_la_droite = vers_la_droite;
        self.depart.vers_la_droite = vers_la_droite;
    }

    /// Pépin vient de mourir (carte 9.29) : un ennemi VIVANT reprend sa place,
    /// son sens et son état de départ (chat endormi, pigeon perché, corbeau en
    /// début de patrouille), compteurs remis à zéro comme dans `new`. Un ennemi
    /// écrasé (ou disparu) le reste : le chemin déjà gagné reste gagné.
    ///
    /// On RECONSTRUIT avec `new` à l'origine (même type, donc même taille et
    /// même portée), puis on remet la part du départ que `new` ne connaît pas
    /// (`Depart`) : aucun compteur ne peut être oublié.
    pub fn revenir_a_l_origine(&mut self) {
        if self.etat != EtatEnnemi::Actif {
            return;
        }
        let depart = self.depart;
        *self = Ennemi::new(self.type_ennemi, self.origine_x, self.origine_y);
        self.depart = depart;
        self.vers_la_droite = depart.vers_la_droite;
        self.sommeil = depart.sommeil;
        self.eau = depart.eau;
        self.saut_poisson = depart.saut_poisson;
    }

    /// Peut-il blesser le joueur (ou être écrasé) ?
    pub fn est_dangereux(&self) -> bool {
        // Le poisson caché sous l'eau est inoffensif (`hors_de_l_eau` est vrai
        // pour tous les autres ennemis).
        self.etat == EtatEnnemi::Actif && self.hors_de_l_eau()
    }

    /// À afficher ? (actif ou en train d'être aplati)
    pub fn est_visible(&self) -> bool {
        self.etat != EtatEnnemi::Disparu
    }

    /// Écrase l'ennemi. Sans effet s'il l'est déjà : on ne peut pas l'écraser deux fois.
    pub fn ecraser(&mut self) {
        if self.etat == EtatEnnemi::Actif {
            self.etat = EtatEnnemi::Ecrase(ANIMATION_ECRASE_PAS);
        }
    }

    /// Demi-tour.
    pub fn retourner(&mut self) {
        self.vers_la_droite = !self.vers_la_droite;
    }

    fn sens(&self) -> f32 {
        if self.vers_la_droite {
            1.0
        } else {
            -1.0
        }
    }

    /// Patrouille de la flaque : gravité, marche, demi-tour au mur ou avant le vide
    /// (la même marche que l'escargot : `marche::patrouiller`).
    fn patrouiller(&mut self, env: &Environnement<'_>) {
        super::marche::patrouiller(
            &mut self.boite,
            &mut self.vers_la_droite,
            &mut self.vitesse_y,
            &mut self.au_sol,
            self.type_ennemi.vitesse(),
            env,
        );
    }

    /// Distance (px) qui reste avant la limite de portée, dans le sens de la marche.
    /// `None` si l'ennemi n'a pas de portée.
    fn reste_avant_limite(&self) -> Option<f32> {
        // `map` transforme l'intérieur d'une `Option` : `None` reste `None`.
        self.portee.map(|portee| {
            if self.vers_la_droite {
                self.origine_x + portee - self.boite.x
            } else {
                self.boite.x - (self.origine_x - portee)
            }
        })
    }

    /// Vol du corbeau : ligne droite horizontale, demi-tour au mur OU à la limite
    /// de sa portée, le premier des deux atteint.
    fn voler(&mut self, env: &Environnement<'_>) {
        let mut dx = self.sens() * self.type_ennemi.vitesse() * DT;
        // Si le pas dépasse la limite, on le raccourcit pour s'arrêter PILE dessus :
        // la patrouille reste exactement symétrique, sans dérive au fil des allers.
        let mut limite_atteinte = false;
        if let Some(reste) = self.reste_avant_limite() {
            if dx.abs() >= reste {
                dx = self.sens() * reste.max(0.0);
                limite_atteinte = true;
            }
        }
        let contacts = deplacer(&mut self.boite, dx, 0.0, env.grille, env.solides);
        if contacts.mur() || limite_atteinte {
            self.retourner();
        }
    }
}

impl Actualisable for Ennemi {
    fn actualiser(&mut self, env: &Environnement<'_>) {
        // `EtatEnnemi` est `Copy` : on le copie dans le `match`, ce qui libère
        // `self` pour pouvoir l'emprunter en `&mut` dans les branches (sinon le
        // borrow checker refuserait : `self.etat` serait emprunté pendant l'appel).
        match self.etat {
            EtatEnnemi::Actif => match self.type_ennemi {
                TypeEnnemi::Flaque => self.patrouiller(env),
                TypeEnnemi::Corbeau => self.voler(env),
                // Le pigeon vole sans regarder les tuiles : `env` ne lui sert pas.
                TypeEnnemi::Pigeon | TypeEnnemi::Mouette => self.voler_pigeon(),
                // Le chat ne tombe pas (pas de gravité : il dort sur son toit) ;
                // il ne bouge que pour bondir, sur une trajectoire calculée.
                TypeEnnemi::Chat => self.dormir(env),
                TypeEnnemi::Grenouille
                | TypeEnnemi::Poisson
                | TypeEnnemi::Crabe
                | TypeEnnemi::Rat => self.actualiser_eau(env),
            },
            EtatEnnemi::Ecrase(restant) => {
                self.etat = if restant <= 1 {
                    EtatEnnemi::Disparu
                } else {
                    EtatEnnemi::Ecrase(restant - 1)
                };
            }
            EtatEnnemi::Disparu => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::GrilleTuiles;

    fn couloir() -> GrilleTuiles {
        GrilleTuiles::depuis_texte(&["##########", "#........#", "#........#", "##########"])
    }

    #[test]
    fn la_flaque_fait_demi_tour_aux_murs() {
        let g = couloir();
        let env = Environnement::simple(&g);
        // Sol = rangée 3 (y = 48) ; flaque posée dessus (bas = 48).
        let mut f = Ennemi::new(TypeEnnemi::Flaque, 64.0, 40.0);
        let (mut min_x, mut max_x) = (f32::MAX, f32::MIN);
        for _ in 0..1500 {
            f.actualiser(&env);
            min_x = min_x.min(f.boite.x);
            max_x = max_x.max(f.boite.droite());
        }
        assert!(
            min_x >= TAILLE_TUILE - 1e-3,
            "ne traverse pas le mur gauche"
        );
        assert!(
            max_x <= 9.0 * TAILLE_TUILE + 1e-3,
            "ne traverse pas le mur droit"
        );
        assert!(min_x < 17.0, "atteint bien le mur gauche : min = {min_x}");
        assert!(max_x > 143.0, "atteint bien le mur droit : max = {max_x}");
    }

    #[test]
    fn la_flaque_fait_demi_tour_avant_le_vide() {
        // Une île de 6 tuiles (x de 64 à 160) au-dessus du vide.
        let g = GrilleTuiles::depuis_texte(&[
            "..............",
            "..............",
            "....######....",
            "..............",
        ]);
        let env = Environnement::simple(&g);
        let mut f = Ennemi::new(TypeEnnemi::Flaque, 80.0, 24.0);
        let (mut min_x, mut max_x) = (f32::MAX, f32::MIN);
        for _ in 0..1500 {
            f.actualiser(&env);
            min_x = min_x.min(f.boite.x);
            max_x = max_x.max(f.boite.droite());
            assert!((f.boite.bas() - 32.0).abs() < 1e-3, "reste sur l'île");
        }
        assert!(min_x >= 64.0 - 1e-3 && max_x <= 160.0 + 1e-3);
        assert!(min_x < 66.0 && max_x > 158.0, "va jusqu'aux deux bords");
    }

    #[test]
    fn la_flaque_tombe_puis_se_pose() {
        let g = couloir();
        let env = Environnement::simple(&g);
        let mut f = Ennemi::new(TypeEnnemi::Flaque, 64.0, 16.0);
        for _ in 0..60 {
            f.actualiser(&env);
        }
        assert!((f.boite.bas() - 3.0 * TAILLE_TUILE).abs() < 1e-3);
    }

    #[test]
    fn le_corbeau_vole_sans_tomber_et_rebondit_sur_les_murs() {
        let g = couloir();
        let env = Environnement::simple(&g);
        let mut c = Ennemi::new(TypeEnnemi::Corbeau, 64.0, 24.0);
        let mut a_change_de_sens = false;
        for _ in 0..600 {
            let avant = c.vers_la_droite;
            c.actualiser(&env);
            a_change_de_sens |= avant != c.vers_la_droite;
            assert_eq!(c.boite.y, 24.0, "reste à la même altitude");
        }
        assert!(a_change_de_sens);
    }

    /// Fait voler (ou marcher) `e` pendant `pas` pas ; renvoie `(x min, x max,
    /// nombre de demi-tours)` de son coin haut-gauche.
    fn patrouille(e: &mut Ennemi, env: &Environnement<'_>, pas: usize) -> (f32, f32, usize) {
        let (mut min_x, mut max_x, mut demi_tours) = (f32::MAX, f32::MIN, 0);
        for _ in 0..pas {
            let avant = e.vers_la_droite;
            e.actualiser(env);
            min_x = min_x.min(e.boite.x);
            max_x = max_x.max(e.boite.x);
            if avant != e.vers_la_droite {
                demi_tours += 1;
            }
        }
        (min_x, max_x, demi_tours)
    }

    #[test]
    fn en_monde_ouvert_le_corbeau_oscille_autour_de_son_point() {
        // 40 tuiles de large, aucun mur à moins de 5 tuiles du corbeau.
        let g = GrilleTuiles::vide(40, 10);
        let env = Environnement::simple(&g);
        let origine = 300.0;
        let mut c = Ennemi::new(TypeEnnemi::Corbeau, origine, 50.0);
        let (min_x, max_x, demi_tours) = patrouille(&mut c, &env, 1500);
        // Exactement la portée de chaque côté, jamais au-delà.
        assert!(
            (min_x - (origine - PORTEE_CORBEAU)).abs() < 1e-3,
            "min = {min_x}"
        );
        assert!(
            (max_x - (origine + PORTEE_CORBEAU)).abs() < 1e-3,
            "max = {max_x}"
        );
        // 1500 pas à 40 px/s = 1000 px parcourus : plusieurs allers-retours.
        assert!(demi_tours >= 5, "demi-tours : {demi_tours}");
        assert_eq!(c.boite.y, 50.0, "reste à la même altitude");

        // Il repasse régulièrement par son point d'apparition : la patrouille
        // est centrée dessus, elle ne dérive pas au fil des allers-retours.
        let mut c = Ennemi::new(TypeEnnemi::Corbeau, origine, 50.0);
        let mut passages = 0;
        let mut cote = c.boite.x < origine;
        for _ in 0..1500 {
            c.actualiser(&env);
            let nouveau_cote = c.boite.x < origine;
            if nouveau_cote != cote {
                passages += 1;
                cote = nouveau_cote;
            }
        }
        // 1000 px parcourus, un passage tous les 160 px environ.
        assert!(passages >= 5, "passages par l'origine : {passages}");
    }

    #[test]
    fn le_mur_fait_faire_demi_tour_au_corbeau_avant_sa_portee() {
        // Mur à gauche (colonne 0, x < 16), rien à droite sur 19 tuiles.
        let g = GrilleTuiles::depuis_texte(&[
            "#...................",
            "#...................",
            "#...................",
            "#...................",
        ]);
        let env = Environnement::simple(&g);
        // Le mur est à 32 px (2 tuiles) : il arrive AVANT la portée de 80 px.
        let mut c = Ennemi::new(TypeEnnemi::Corbeau, 48.0, 24.0);
        let (min_x, max_x, demi_tours) = patrouille(&mut c, &env, 1500);
        assert!(
            (min_x - TAILLE_TUILE).abs() < 1e-3,
            "s'arrête au mur : {min_x}"
        );
        // À droite, pas de mur : c'est la portée qui joue.
        assert!(
            (max_x - (48.0 + PORTEE_CORBEAU)).abs() < 1e-3,
            "max = {max_x}"
        );
        assert!(demi_tours >= 5, "demi-tours : {demi_tours}");
    }

    #[test]
    fn la_flaque_n_a_pas_de_portee() {
        assert_eq!(TypeEnnemi::Flaque.portee(), None);
        assert_eq!(TypeEnnemi::Corbeau.portee(), Some(PORTEE_CORBEAU));
        // Toit de 28 tuiles entre deux murs : la flaque va d'un mur à l'autre,
        // bien plus loin que la portée d'un corbeau.
        let g = GrilleTuiles::depuis_texte(&[
            "##############################",
            "#............................#",
            "#............................#",
            "##############################",
        ]);
        let env = Environnement::simple(&g);
        let mut f = Ennemi::new(TypeEnnemi::Flaque, 240.0, 40.0);
        let (min_x, max_x, _) = patrouille(&mut f, &env, 2400);
        assert!(
            min_x < TAILLE_TUILE + 1.0,
            "atteint le mur gauche : {min_x}"
        );
        assert!(
            max_x + 12.0 > 29.0 * TAILLE_TUILE - 1.0,
            "atteint le mur droit : {max_x}"
        );
        assert!(240.0 - min_x > PORTEE_CORBEAU && max_x - 240.0 > PORTEE_CORBEAU);
    }

    #[test]
    fn ecraser_aplatit_puis_fait_disparaitre() {
        let g = couloir();
        let env = Environnement::simple(&g);
        let mut f = Ennemi::new(TypeEnnemi::Flaque, 64.0, 40.0);
        assert!(f.est_dangereux());
        f.ecraser();
        assert!(!f.est_dangereux() && f.est_visible());
        assert_eq!(f.etat, EtatEnnemi::Ecrase(ANIMATION_ECRASE_PAS));

        // Écraser deux fois ne relance pas l'animation.
        f.actualiser(&env);
        f.ecraser();
        assert_eq!(f.etat, EtatEnnemi::Ecrase(ANIMATION_ECRASE_PAS - 1));

        let x = f.boite.x;
        for _ in 0..ANIMATION_ECRASE_PAS {
            f.actualiser(&env);
        }
        assert_eq!(f.etat, EtatEnnemi::Disparu);
        assert!(!f.est_visible());
        assert_eq!(f.boite.x, x, "une flaque écrasée ne bouge plus");
    }
}
