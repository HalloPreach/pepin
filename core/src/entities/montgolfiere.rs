//! Montgolfière : un ballon et sa nacelle. La nacelle est une plateforme
//! (solide, 16 x 8 px : on se tient dessus) qui MONTE quand Pépin s'y tient,
//! jusqu'au premier solide au-dessus du ballon ou à sa hauteur maximale
//! (`MONTEE_MAX`), puis REDESCEND doucement à son altitude de départ quand il la
//! quitte.
//!
//! Qui décide de quoi, à chaque pas (ordre de `entities`) :
//! 1. `actualiser` : la nacelle monte (un passager à bord), reste en haut, ou
//!    redescend (personne à bord) ;
//! 2. `contacts::porter_par_plateformes` : Pépin posé dessus suit la nacelle,
//!    comme sur une plateforme mobile — il est transporté sans glisser ;
//! 3. `contacts::resoudre_contacts` appelle `embarquer` : « Pépin est-il posé
//!    sur la nacelle à la fin de ce pas ? ». La réponse sert au pas SUIVANT.
//!
//! Le ballon (l'enveloppe et ses cordes, `HAUT_BALLON` px au-dessus de la
//! nacelle) n'est PAS solide : c'est un dessin, on le traverse. Mais il ne
//! traverse rien lui-même : la montée s'arrête dès que le ballon, ou Pépin
//! debout dans la nacelle, toucherait un solide.
//!
//! Notions Rust : `Option<Aabb>` (« la boîte du passager, ou personne »), un
//! `enum` d'état pour le dessin, une fonction pure (`sommet`) testée seule,
//! `f32::max` / `f32::min` pour borner un mouvement sans `if`.

use super::Actualisable;
use crate::aabb::{touche_un_obstacle, Aabb, TAILLE_TUILE};
use crate::physics::{Environnement, Vec2, DT};
use crate::player::Joueur;

/// Vitesse de montée (px/s) avec Pépin à bord : 0,8 px par pas, 3 tuiles par
/// seconde. Un peu plus vite qu'une plateforme mobile (36 px/s).
pub const VITESSE_MONTEE: f32 = 48.0;
/// Vitesse de descente (px/s) sans personne à bord : deux fois plus lente que
/// la montée, la nacelle redescend « doucement ».
pub const VITESSE_DESCENTE: f32 = 24.0;
/// Hauteur maximale de la montée (px) : 8 tuiles au-dessus de l'altitude de
/// départ, s'il n'y a pas de solide avant.
pub const MONTEE_MAX: f32 = 8.0 * TAILLE_TUILE;
/// Hauteur du ballon (enveloppe + cordes) au-dessus du DESSUS de la nacelle :
/// deux tuiles. Pépin (14 px de haut) se tient sous l'enveloppe, entre les cordes.
pub const HAUT_BALLON: f32 = 2.0 * TAILLE_TUILE;
/// Largeur de la nacelle (px) : une tuile.
pub const LARGEUR_NACELLE: f32 = TAILLE_TUILE;
/// Épaisseur de la nacelle (px) : la moitié haute de sa case.
pub const HAUTEUR_NACELLE: f32 = 8.0;

/// Ce que fait la montgolfière en ce moment (pour le dessin : le brûleur ne
/// s'allume que pendant la montée).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtatMontgolfiere {
    /// À son altitude de départ, personne à bord.
    Repos,
    /// Pépin est à bord : elle monte (brûleur allumé).
    Montee,
    /// Pépin est à bord mais elle ne monte plus : hauteur maximale atteinte, ou
    /// le ballon (ou Pépin) toucherait un solide au-dessus.
    EnHaut,
    /// Personne à bord : elle redescend doucement vers son altitude de départ.
    Descente,
}

#[derive(Debug, Clone)]
pub struct Montgolfiere {
    /// Boîte de la NACELLE (16 x 8 px) : solide, on se tient sur son dessus.
    pub boite: Aabb,
    /// Altitude de départ (haut de la nacelle), où elle revient se poser.
    y_depart: f32,
    /// Altitude la plus haute (haut de la nacelle) : premier solide au-dessus
    /// du ballon, ou `MONTEE_MAX`.
    y_haut: f32,
    /// Boîte de Pépin s'il était posé sur la nacelle à la fin du dernier pas.
    passager: Option<Aabb>,
    /// Pépin est juste SOUS la nacelle : elle ne redescend pas sur lui.
    pepin_dessous: bool,
    etat: EtatMontgolfiere,
    /// Déplacement effectué pendant le DERNIER pas (sert à porter le joueur).
    pub deplacement: Vec2,
}

impl Montgolfiere {
    /// Nacelle dont le coin haut-gauche est en `(x, y_depart)` ; elle montera au
    /// plus jusqu'à `y_haut` (ramené à `y_depart` s'il est plus bas).
    pub fn nouvelle(x: f32, y_depart: f32, y_haut: f32) -> Self {
        Self {
            boite: Aabb::new(x, y_depart, LARGEUR_NACELLE, HAUTEUR_NACELLE),
            y_depart,
            y_haut: y_haut.min(y_depart),
            passager: None,
            pepin_dessous: false,
            etat: EtatMontgolfiere::Repos,
            deplacement: Vec2::default(),
        }
    }

    /// Altitude la plus haute (haut de la nacelle) d'une montgolfière partie de
    /// `y_depart`, sous un plafond dont le BAS est en `plafond` (0 : le haut de la
    /// carte) : le haut du ballon s'arrête au plafond, et la montée ne dépasse
    /// jamais `MONTEE_MAX`. Fonction pure (utilisée par le parseur de niveaux).
    pub fn sommet(y_depart: f32, plafond: f32) -> f32 {
        (y_depart - MONTEE_MAX)
            .max(plafond + HAUT_BALLON)
            .min(y_depart)
    }

    /// Le ballon (enveloppe et cordes) : la zone au-dessus de la nacelle, de
    /// `HAUT_BALLON` px de haut. Pas solide ; elle sert à arrêter la montée.
    pub fn ballon(&self) -> Aabb {
        Aabb::new(
            self.boite.x,
            self.boite.y - HAUT_BALLON,
            self.boite.w,
            HAUT_BALLON,
        )
    }

    /// `(y_depart, y_haut)` : altitudes (haut de la nacelle) de départ et la plus
    /// haute, pour l'analyse de faisabilité des niveaux.
    pub fn altitudes(&self) -> (f32, f32) {
        (self.y_depart, self.y_haut)
    }

    /// Ce que fait la montgolfière en ce moment (pour le dessin).
    pub fn etat(&self) -> EtatMontgolfiere {
        self.etat
    }

    /// Pépin a-t-il été vu à bord au dernier pas ?
    pub fn a_un_passager(&self) -> bool {
        self.passager.is_some()
    }

    /// Regarde où est Pépin à la fin du pas (appelé par `resoudre_contacts`) :
    /// posé sur la nacelle (au sol, son bas à 1 px du dessus de la nacelle, sans
    /// la chevaucher) ou juste en dessous. La réponse sert au pas suivant.
    pub fn embarquer(&mut self, joueur: &Joueur) {
        let b = joueur.boite;
        let pose = joueur.au_sol
            && b.decale(0.0, 1.0).chevauche(&self.boite)
            && !b.chevauche(&self.boite);
        self.passager = pose.then_some(b);
        // Juste sous la nacelle : la bande que la nacelle balaierait en
        // descendant d'un pas (plus 1 px de marge).
        let sous = Aabb::new(
            self.boite.x,
            self.boite.bas(),
            self.boite.w,
            VITESSE_DESCENTE * DT + 1.0,
        );
        self.pepin_dessous = !pose && b.chevauche(&sous);
    }
}

impl Actualisable for Montgolfiere {
    fn actualiser(&mut self, env: &Environnement<'_>) {
        let y = self.boite.y;
        let mut nouveau = y;
        if let Some(passager) = self.passager {
            // Monter d'un pas, sans dépasser `y_haut` (arrivée pile dessus).
            let vise = (y - VITESSE_MONTEE * DT).max(self.y_haut);
            let dy = vise - y;
            // Place libre au-dessus ? Ni le ballon, ni Pépin debout dans la
            // nacelle ne doivent entrer dans un solide (une corniche à côté de la
            // colonne du ballon peut cogner la tête de Pépin, par exemple).
            let place = dy < 0.0
                && !touche_un_obstacle(&self.ballon().decale(0.0, dy), env.grille, env.solides)
                && !touche_un_obstacle(&passager.decale(0.0, dy), env.grille, env.solides);
            if place {
                nouveau = vise;
                self.etat = EtatMontgolfiere::Montee;
            } else {
                self.etat = EtatMontgolfiere::EnHaut;
            }
        } else if y < self.y_depart {
            // Personne à bord : redescendre doucement, sans dépasser l'altitude de
            // départ, et jamais sur Pépin.
            if !self.pepin_dessous {
                nouveau = (y + VITESSE_DESCENTE * DT).min(self.y_depart);
            }
            self.etat = EtatMontgolfiere::Descente;
        } else {
            self.etat = EtatMontgolfiere::Repos;
        }
        self.deplacement = Vec2::new(0.0, nouveau - y);
        self.boite.y = nouveau;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::contacts::{porter_par_plateformes, resoudre_contacts};
    use crate::entities::{solides_des_entites, Entite};
    use crate::level::Niveau;
    use crate::physics::JOUEUR_H;
    use crate::player::{Direction, Entrees};

    /// Mini-niveau de test, 8 x 16 tuiles, sans plafond : la nacelle (`b`) est en
    /// (2, 12), soit x = 32, haut à y = 192 ; le sol est sur la ligne 15.
    const SANS_PLAFOND: &str = "
........
........
........
........
........
........
........
........
........
........
........
........
..b.....
........
@..ooo.X
########
";

    /// Le même, avec un plafond (ligne 5) au-dessus de la colonne du ballon : bas
    /// du plafond à y = 96, donc le haut du ballon s'y arrête quand la nacelle
    /// est en y = 96 + 32 = 128.
    const AVEC_PLAFOND: &str = "
........
........
........
........
........
..#.....
........
........
........
........
........
........
..b.....
........
@..ooo.X
########
";

    /// Une corniche en (3, 8) À CÔTÉ de la colonne du ballon (colonne 2) : le
    /// ballon passe, mais Pépin debout sur le bord droit de la nacelle s'y
    /// cognerait la tête (bas de la corniche à y = 144).
    const CORNICHE: &str = "
........
........
........
........
........
........
........
........
...#....
........
........
........
..b.....
........
@..ooo.X
########
";

    /// Un banc d'essai : le mini-niveau, ses entités et Pépin, avancés pas à pas
    /// dans l'ordre du jeu (`Monde::avancer`) : entités, portage, joueur, contacts.
    struct Banc {
        niveau: Niveau,
        entites: Vec<Entite>,
        joueur: Joueur,
    }

    impl Banc {
        /// Pépin est posé sur la nacelle, décalé de `dx` px depuis son bord gauche.
        fn new(ascii: &str, dx: f32) -> Self {
            let niveau = Niveau::depuis_ascii(ascii).expect("mini-niveau valide");
            let entites = niveau.entites.clone();
            let mut banc = Self {
                niveau,
                entites,
                joueur: Joueur::new(0.0, 0.0),
            };
            let n = banc.nacelle().boite;
            banc.joueur = Joueur::new(n.x + dx, n.y - JOUEUR_H);
            banc
        }

        fn nacelle(&self) -> &Montgolfiere {
            self.entites
                .iter()
                .find_map(|e| match e {
                    Entite::Montgolfiere(m) => Some(m),
                    _ => None,
                })
                .expect("une montgolfière dans le mini-niveau")
        }

        fn pas(&mut self, entrees: &Entrees) {
            let solides = solides_des_entites(&self.entites);
            {
                let env = Environnement {
                    grille: &self.niveau,
                    solides: &solides,
                    courants: &[],
                    vents: &[],
                };
                for e in &mut self.entites {
                    e.actualiser(&env);
                }
            }
            let solides = solides_des_entites(&self.entites);
            let env = Environnement {
                grille: &self.niveau,
                solides: &solides,
                courants: &[],
                vents: &[],
            };
            porter_par_plateformes(&mut self.joueur, &self.entites, &env);
            self.joueur.actualiser(entrees, &env);
            resoudre_contacts(&mut self.joueur, &mut self.entites);
        }

        fn attendre(&mut self, n: u32) {
            for _ in 0..n {
                self.pas(&Entrees::default());
            }
        }
    }

    #[test]
    fn le_sommet_est_le_plafond_ou_la_hauteur_max() {
        // Sans plafond (haut de la carte loin) : 8 tuiles au-dessus du départ.
        assert_eq!(Montgolfiere::sommet(192.0, 0.0), 192.0 - MONTEE_MAX);
        // Plafond bas en y = 96 : le haut du ballon s'y arrête (nacelle à 128).
        assert_eq!(Montgolfiere::sommet(192.0, 96.0), 128.0);
        // Plafond juste au-dessus du ballon : elle ne monte pas du tout.
        assert_eq!(Montgolfiere::sommet(192.0, 160.0), 192.0);
        // Près du haut de la carte : le ballon ne sort pas de la carte.
        assert_eq!(Montgolfiere::sommet(64.0, 0.0), HAUT_BALLON);
    }

    #[test]
    fn le_parseur_place_la_nacelle_et_son_sommet() {
        let banc = Banc::new(SANS_PLAFOND, 3.0);
        let n = banc.nacelle();
        assert_eq!(n.boite, Aabb::new(32.0, 192.0, 16.0, 8.0));
        assert_eq!(n.altitudes(), (192.0, 192.0 - MONTEE_MAX));
        assert_eq!(n.etat(), EtatMontgolfiere::Repos);
        let banc = Banc::new(AVEC_PLAFOND, 3.0);
        assert_eq!(banc.nacelle().altitudes(), (192.0, 128.0));
    }

    /// Pépin posé sur la nacelle : elle monte à 0,8 px par pas, et Pépin est
    /// TRANSPORTÉ sans glisser — à chaque pas il est au sol, ses pieds pile sur le
    /// dessus de la nacelle, et il ne bouge pas d'un pixel en x.
    #[test]
    fn la_montee_porte_pepin_sans_glisser() {
        let mut banc = Banc::new(SANS_PLAFOND, 3.0);
        let x0 = banc.joueur.boite.x;
        // Pas 1 : Pépin se pose (la nacelle ne le voit qu'à la fin de ce pas).
        banc.pas(&Entrees::default());
        assert_eq!(banc.nacelle().boite.y, 192.0, "pas encore de passager");
        assert!(banc.nacelle().a_un_passager());
        let mut y_avant = banc.nacelle().boite.y;
        for k in 0..100 {
            banc.pas(&Entrees::default());
            let n = banc.nacelle();
            let dy = n.boite.y - y_avant;
            assert!(
                (dy + VITESSE_MONTEE * DT).abs() < 1e-3,
                "pas {k} : la nacelle monte de 0,8 px (dy = {dy})"
            );
            assert_eq!(n.etat(), EtatMontgolfiere::Montee);
            assert!(banc.joueur.au_sol, "pas {k} : Pépin reste posé");
            assert!(
                (banc.joueur.boite.bas() - n.boite.y).abs() < 1e-3,
                "pas {k} : pieds sur la nacelle ({} contre {})",
                banc.joueur.boite.bas(),
                n.boite.y
            );
            assert_eq!(banc.joueur.boite.x, x0, "pas {k} : aucun glissement");
            y_avant = n.boite.y;
        }
        // 100 pas à 0,8 px : 80 px plus haut.
        assert!((banc.nacelle().boite.y - (192.0 - 80.0)).abs() < 1e-2);
    }

    /// Sans plafond, la montée s'arrête PILE à 8 tuiles au-dessus du départ, et
    /// la nacelle y reste tant que Pépin est à bord.
    #[test]
    fn la_montee_s_arrete_a_la_hauteur_max() {
        let mut banc = Banc::new(SANS_PLAFOND, 3.0);
        // 1 pas pour se poser + 160 pas de montée (128 px à 0,8 px), et quelques
        // pas de plus : les arrondis de flottants peuvent en demander un de plus,
        // le dernier pas s'arrête PILE sur le sommet (`max`).
        banc.attendre(1 + 165);
        assert_eq!(banc.nacelle().boite.y, 192.0 - MONTEE_MAX);
        banc.attendre(120);
        let n = banc.nacelle();
        assert_eq!(n.boite.y, 192.0 - MONTEE_MAX, "elle reste en haut");
        assert_eq!(n.etat(), EtatMontgolfiere::EnHaut);
        assert_eq!(n.deplacement, Vec2::default());
        assert!(banc.joueur.au_sol);
        assert!((banc.joueur.boite.bas() - n.boite.y).abs() < 1e-3);
    }

    /// Avec un plafond au-dessus de la colonne, le haut du ballon s'arrête
    /// contre lui : la nacelle s'immobilise en y = 128, sans que ni le ballon ni
    /// Pépin n'entrent dans le solide.
    #[test]
    fn la_montee_s_arrete_sous_le_premier_solide() {
        let mut banc = Banc::new(AVEC_PLAFOND, 3.0);
        banc.attendre(300);
        let n = banc.nacelle();
        assert_eq!(n.boite.y, 128.0);
        assert_eq!(n.etat(), EtatMontgolfiere::EnHaut);
        assert_eq!(n.ballon().y, 96.0, "le haut du ballon touche le plafond");
        assert!(!touche_un_obstacle(&n.ballon(), &banc.niveau, &[]));
        assert!(banc.joueur.boite.y > 96.0, "Pépin sous le plafond");
        assert!(banc.joueur.au_sol);
    }

    /// Pépin debout sur le bord droit de la nacelle déborde sur la colonne
    /// voisine : la montée s'arrête avant que sa tête n'entre dans la corniche
    /// (le ballon, lui, passerait). Il recule vers le centre : elle repart.
    #[test]
    fn la_montee_s_arrete_avant_que_pepin_ne_se_cogne() {
        // Pépin en x = 32 + 12 = 44 : sa boîte (44..54) déborde sur la colonne 3.
        let mut banc = Banc::new(CORNICHE, 12.0);
        banc.attendre(300);
        let n = banc.nacelle().boite.y;
        assert_eq!(banc.nacelle().etat(), EtatMontgolfiere::EnHaut);
        // La tête de Pépin (haut de sa boîte) est sous la corniche (bas en y = 144),
        // à moins d'un pas de montée.
        let tete = banc.joueur.boite.y;
        assert!(
            (144.0..144.0 + VITESSE_MONTEE * DT + 1e-3).contains(&tete),
            "tête en y = {tete}, nacelle en y = {n}"
        );
        // Pépin recule vers la gauche (6 pas d'appui, puis il glisse : 7,6 px en
        // tout), il est tout entier au-dessus de la nacelle : la montée reprend.
        let gauche = Entrees {
            direction: Direction::Gauche,
            saut: false,
        };
        for _ in 0..6 {
            banc.pas(&gauche);
        }
        banc.attendre(30);
        assert!(banc.nacelle().boite.y < n - 10.0, "elle remonte");
        assert!(banc.joueur.boite.droite() <= 48.0, "Pépin a quitté le bord");
        assert!(banc.joueur.boite.x >= 32.0, "toujours dans la nacelle");
        assert!(banc.joueur.au_sol);
    }

    /// Pépin quitte la nacelle (ici, il est replacé au sol, loin d'elle) : elle
    /// redescend DOUCEMENT (0,4 px par pas, deux fois moins vite que la montée)
    /// jusqu'à son altitude de départ exactement, puis se tient au repos.
    #[test]
    fn la_nacelle_redescend_doucement_quand_pepin_la_quitte() {
        let mut banc = Banc::new(SANS_PLAFOND, 3.0);
        banc.attendre(1 + 80); // 80 pas de montée : 64 px plus haut
        let haut = banc.nacelle().boite.y;
        assert!((haut - 128.0).abs() < 1e-2, "nacelle en y = {haut}");
        // Pépin s'en va : posé au sol, à droite (pieds sur la ligne 15).
        banc.joueur.placer(100.0, 240.0 - JOUEUR_H);
        // Au pas suivant, la nacelle le croit encore à bord (elle monte une
        // dernière fois), puis constate qu'il est parti.
        banc.attendre(1);
        let mut y_avant = banc.nacelle().boite.y;
        for k in 0..100 {
            banc.pas(&Entrees::default());
            let n = banc.nacelle();
            let dy = n.boite.y - y_avant;
            assert!(
                (dy - VITESSE_DESCENTE * DT).abs() < 1e-3,
                "pas {k} : descente de 0,4 px (dy = {dy})"
            );
            assert_eq!(n.etat(), EtatMontgolfiere::Descente);
            y_avant = n.boite.y;
        }
        // Le reste du chemin (192 - 167,2 = 24,8 px à 0,4 px : 62 pas), puis le repos.
        banc.attendre(80);
        let n = banc.nacelle();
        assert_eq!(n.boite.y, 192.0, "pile à l'altitude de départ");
        assert_eq!(n.etat(), EtatMontgolfiere::Repos);
        assert_eq!(n.deplacement, Vec2::default());
    }

    /// Pépin saute de la nacelle et retombe dessus : pendant le saut elle
    /// redescend un peu, puis repart vers le haut dès qu'il est de nouveau à bord.
    #[test]
    fn pepin_peut_sauter_et_reprendre_la_montee() {
        let mut banc = Banc::new(SANS_PLAFOND, 3.0);
        banc.attendre(1 + 40);
        let avant_saut = banc.nacelle().boite.y;
        let saut = Entrees {
            direction: Direction::Immobile,
            saut: true,
        };
        banc.pas(&saut);
        banc.pas(&saut);
        assert!(!banc.joueur.au_sol, "Pépin a sauté");
        let mut a_descendu = false;
        for _ in 0..120 {
            banc.pas(&Entrees::default());
            a_descendu |= banc.nacelle().etat() == EtatMontgolfiere::Descente;
        }
        assert!(a_descendu, "sans passager, elle a redescendu");
        assert!(banc.joueur.au_sol, "Pépin est retombé dans la nacelle");
        assert_eq!(banc.nacelle().etat(), EtatMontgolfiere::Montee);
        assert!(banc.nacelle().boite.y < avant_saut, "et elle a repris la montée");
    }

    /// Une nacelle qui redescend ne s'écrase jamais sur Pépin juste en dessous :
    /// elle attend qu'il s'en aille.
    #[test]
    fn la_nacelle_ne_redescend_pas_sur_pepin() {
        let grille = crate::aabb::GrilleTuiles::vide(8, 16);
        let env = Environnement::simple(&grille);
        let mut m = Montgolfiere::nouvelle(32.0, 192.0, 64.0);
        m.boite.y = 100.0; // en l'air, au-dessus de son altitude de départ
        // Pépin juste sous la nacelle (en plein saut, par exemple).
        let mut j = Joueur::new(35.0, 108.2);
        j.au_sol = false;
        m.embarquer(&j);
        m.actualiser(&env);
        assert_eq!(m.boite.y, 100.0, "elle attend");
        assert_eq!(m.etat(), EtatMontgolfiere::Descente);
        // Pépin s'écarte : elle repart vers le bas.
        j.boite.x = 80.0;
        m.embarquer(&j);
        m.actualiser(&env);
        assert!((m.boite.y - (100.0 + VITESSE_DESCENTE * DT)).abs() < 1e-4);
    }

    /// Personne à bord et à l'altitude de départ : la nacelle ne bouge pas, et un
    /// Pépin qui la frôle en l'air (sans s'y poser) ne la fait pas monter.
    #[test]
    fn sans_passager_elle_reste_au_repos() {
        let grille = crate::aabb::GrilleTuiles::vide(8, 16);
        let env = Environnement::simple(&grille);
        let mut m = Montgolfiere::nouvelle(32.0, 192.0, 64.0);
        let mut j = Joueur::new(35.0, 192.0 - JOUEUR_H);
        j.au_sol = false; // pieds au ras de la nacelle, mais en l'air
        for _ in 0..60 {
            m.embarquer(&j);
            m.actualiser(&env);
        }
        assert_eq!(m.boite.y, 192.0);
        assert_eq!(m.etat(), EtatMontgolfiere::Repos);
        assert!(!m.a_un_passager());
    }
}
