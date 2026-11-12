//! Contacts joueur <-> entités : ramassage, écrasement, rebond, dégâts, portage.
//!
//! La fonction `resoudre_contacts` ne décide PAS de ce qui arrive au score ou à
//! la vie du joueur : elle se contente de RAPPORTER ce qui s'est passé sous la
//! forme d'une liste de `Contact`. C'est `EtatJeu::traiter` qui en tire les
//! conséquences. Cette séparation rend chaque morceau testable seul.
//!
//! Notions Rust : emprunt exclusif `&mut` sur le joueur ET sur la tranche
//! d'entités (deux emprunts distincts, donc permis en même temps) ; `match` sur
//! `&mut Entite` pour modifier la variante concernée ; `enum` à données.

use super::{solides_des_entites, Entite};
use crate::aabb::Aabb;
use crate::physics::Environnement;
use crate::player::Joueur;

/// Vitesse (px/s) donnée au joueur quand il écrase un ennemi.
pub const VITESSE_REBOND_ECRASEMENT: f32 = 240.0;
/// Profondeur maximale (px) d'enfoncement du bas du joueur dans le haut d'un
/// ennemi pour que le contact compte comme un « écrasement par le dessus ».
/// Au-delà, c'est un contact de côté : le joueur est blessé.
/// (À vitesse de chute maximale, le joueur avance de 6 px par pas : 7 px suffisent.)
pub const MARGE_ECRASEMENT: f32 = 7.0;

/// Cause d'une mort.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CauseMort {
    Ennemi,
    Pointe,
    Eclair,
    Vide,
}

/// Ce qui s'est passé pendant ce pas de simulation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Contact {
    /// Une goutte d'or vient d'être ramassée (numéro dans son niveau).
    Goutte(u8),
    /// Une lanterne vient de s'allumer : voici où réapparaître `(x, y)`.
    Lanterne(f32, f32),
    /// Un ennemi vient d'être écrasé.
    Ecrase,
    /// Le joueur a rebondi sur un champignon.
    Rebond,
    /// Le joueur est mort.
    Mort(CauseMort),
}

/// Le joueur tombe-t-il sous le monde ? `bas_monde` : ordonnée (px) du bas de la carte.
/// On laisse une tuile de marge pour que la chute soit visible avant la mort.
pub fn verifier_vide(joueur: &Joueur, bas_monde: f32) -> Option<Contact> {
    if joueur.boite.y > bas_monde + crate::aabb::TAILLE_TUILE {
        Some(Contact::Mort(CauseMort::Vide))
    } else {
        None
    }
}

/// Fait suivre au joueur les plateformes mobiles sur lesquelles il se tient.
///
/// À appeler APRÈS le déplacement des plateformes, AVANT `Joueur::actualiser`.
/// `env.solides` doit contenir les plateformes à leur NOUVELLE position
/// (construire `solides_des_entites` après la mise à jour des entités).
pub fn porter_par_plateformes(joueur: &mut Joueur, entites: &[Entite], env: &Environnement<'_>) {
    if !joueur.au_sol {
        return;
    }
    for entite in entites {
        if let Entite::Mobile(p) = entite {
            // Où était la plateforme avant ce pas ?
            let avant = p.boite.decale(-p.deplacement.x, -p.deplacement.y);
            let pose_dessus =
                joueur.boite.decale(0.0, 1.0).chevauche(&avant) && !joueur.boite.chevauche(&avant);
            if pose_dessus {
                joueur.porter(p.deplacement.x, p.deplacement.y, env);
                return; // une seule plateforme porte le joueur
            }
        }
    }
}

/// Le joueur est-il posé sur le dessus de `boite` (à 1 px près, sans la chevaucher) ?
fn pose_sur(joueur: &Joueur, boite: &Aabb) -> bool {
    joueur.boite.decale(0.0, 1.0).chevauche(boite) && !joueur.boite.chevauche(boite)
}

/// Examine tous les contacts de ce pas. Modifie le joueur (rebonds) et les
/// entités (ramassées, écrasées, allumées) et renvoie la liste des événements.
///
/// Ordre de la liste : ordre des entités. Une mort est toujours placée EN DERNIER
/// et il n'y en a qu'une, même si plusieurs dangers touchent en même temps.
pub fn resoudre_contacts(joueur: &mut Joueur, entites: &mut [Entite]) -> Vec<Contact> {
    let mut contacts = Vec::new();
    let mut mort: Option<CauseMort> = None;
    let mut lanterne_allumee: Option<usize> = None;

    for (i, entite) in entites.iter_mut().enumerate() {
        match entite {
            Entite::Goutte(g) => {
                if !g.est_ramassee() && joueur.boite.chevauche(&g.boite) && g.ramasser() {
                    contacts.push(Contact::Goutte(g.index));
                }
            }
            Entite::Lanterne(l) => {
                if joueur.boite.chevauche(&l.boite) && l.activer() {
                    let (x, y) = l.point_de_reapparition();
                    contacts.push(Contact::Lanterne(x, y));
                    lanterne_allumee = Some(i);
                }
            }
            Entite::Ennemi(e) => {
                if e.est_dangereux() && joueur.boite.chevauche(&e.boite) {
                    let par_le_dessus = joueur.vitesse.y > 0.0
                        && joueur.boite.bas() - e.boite.y <= MARGE_ECRASEMENT;
                    if par_le_dessus {
                        e.ecraser();
                        joueur.rebondir(VITESSE_REBOND_ECRASEMENT);
                        contacts.push(Contact::Ecrase);
                    } else {
                        mort.get_or_insert(CauseMort::Ennemi);
                    }
                }
            }
            Entite::Pointe(p) => {
                if joueur.boite.chevauche(&p.zone_danger()) {
                    mort.get_or_insert(CauseMort::Pointe);
                }
            }
            Entite::Eclair(e) => {
                if e.frappe() && joueur.boite.chevauche(&e.zone) {
                    mort.get_or_insert(CauseMort::Eclair);
                }
            }
            Entite::Champignon(c) => {
                if joueur.vitesse.y >= 0.0 && joueur.boite.chevauche(&c.boite) {
                    joueur.rebondir(super::champignon::VITESSE_CHAMPIGNON);
                    c.rebondir();
                    contacts.push(Contact::Rebond);
                }
            }
            Entite::Friable(f) => {
                if pose_sur(joueur, &f.boite) {
                    f.toucher();
                }
                f.reapparaitre_si_libre(&joueur.boite);
            }
            Entite::Mobile(_) => {}
        }
    }

    // Une seule lanterne allumée à la fois : on éteint les autres.
    if let Some(allumee) = lanterne_allumee {
        for (i, entite) in entites.iter_mut().enumerate() {
            if let Entite::Lanterne(l) = entite {
                if i != allumee {
                    l.eteindre();
                }
            }
        }
    }

    if let Some(cause) = mort {
        contacts.push(Contact::Mort(cause));
    }
    contacts
}

/// Raccourci : boîtes solides des entités, prêtes pour `Environnement::solides`.
pub fn solides(entites: &[Entite]) -> Vec<Aabb> {
    solides_des_entites(entites)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::GrilleTuiles;
    use crate::entities::champignon::VITESSE_CHAMPIGNON;
    use crate::entities::{
        Actualisable, Champignon, Ennemi, GoutteDor, Lanterne, PlateformeFriable, PlateformeMobile,
        Pointe, TypeEnnemi,
    };

    /// Joueur dont la boîte occupe `(x, y)`.
    fn joueur(x: f32, y: f32) -> Joueur {
        Joueur::new(x, y)
    }

    #[test]
    fn ramassage_d_une_goutte_une_seule_fois() {
        let mut j = joueur(100.0, 100.0);
        let mut e = vec![Entite::Goutte(GoutteDor::new(2, 102.0, 104.0))];
        assert_eq!(resoudre_contacts(&mut j, &mut e), vec![Contact::Goutte(2)]);
        assert!(resoudre_contacts(&mut j, &mut e).is_empty(), "déjà prise");
    }

    #[test]
    fn pas_de_ramassage_a_distance() {
        let mut j = joueur(0.0, 0.0);
        let mut e = vec![Entite::Goutte(GoutteDor::new(0, 100.0, 100.0))];
        assert!(resoudre_contacts(&mut j, &mut e).is_empty());
    }

    #[test]
    fn ecrasement_par_le_dessus_rebondit() {
        // La flaque (12x8) est en y = 100 ; le joueur (14 de haut) tombe dessus.
        let mut j = joueur(100.0, 100.0 - 14.0 + 3.0); // bas = 103 : enfoncé de 3 px
        j.vitesse.y = 200.0;
        let mut e = vec![Entite::Ennemi(Ennemi::new(
            TypeEnnemi::Flaque,
            100.0,
            100.0,
        ))];
        assert_eq!(resoudre_contacts(&mut j, &mut e), vec![Contact::Ecrase]);
        assert_eq!(j.vitesse.y, -VITESSE_REBOND_ECRASEMENT);
        match &e[0] {
            Entite::Ennemi(f) => assert!(!f.est_dangereux()),
            _ => unreachable!(),
        }
        // Ennemi écrasé : plus de contact, plus de danger.
        j.vitesse.y = 0.0;
        assert!(resoudre_contacts(&mut j, &mut e).is_empty());
    }

    #[test]
    fn contact_de_cote_tue() {
        let mut j = joueur(95.0, 96.0); // bas = 110 : au milieu de la flaque (100..108)... côté
        j.vitesse.y = 0.0;
        let mut e = vec![Entite::Ennemi(Ennemi::new(
            TypeEnnemi::Flaque,
            100.0,
            100.0,
        ))];
        assert_eq!(
            resoudre_contacts(&mut j, &mut e),
            vec![Contact::Mort(CauseMort::Ennemi)]
        );
    }

    #[test]
    fn monter_dans_un_ennemi_tue_meme_en_le_survolant() {
        // Vitesse montante : jamais un écrasement.
        let mut j = joueur(100.0, 90.0);
        j.vitesse.y = -100.0;
        let mut e = vec![Entite::Ennemi(Ennemi::new(
            TypeEnnemi::Flaque,
            100.0,
            100.0,
        ))];
        assert_eq!(
            resoudre_contacts(&mut j, &mut e),
            vec![Contact::Mort(CauseMort::Ennemi)]
        );
    }

    #[test]
    fn les_pointes_tuent_mais_pas_au_ras_du_bord() {
        let mut e = vec![Entite::Pointe(Pointe::new(100.0, 120.0))];
        // Posé dessus : bas du joueur = 124, dedans.
        let mut j = joueur(103.0, 110.0);
        assert_eq!(
            resoudre_contacts(&mut j, &mut e),
            vec![Contact::Mort(CauseMort::Pointe)]
        );
        // Juste à côté de la boîte (marge de pardon de 2 px) : sauf.
        let mut j = joueur(100.0 - 10.0 + 1.0, 110.0); // droite = 101 < 102
        assert!(resoudre_contacts(&mut j, &mut e).is_empty());
    }

    #[test]
    fn l_eclair_ne_tue_que_pendant_la_frappe() {
        use crate::entities::eclair::{ANNONCE_PAS, FRAPPE_PAS};
        use crate::entities::Eclair;
        let g = GrilleTuiles::vide(20, 20);
        let env = Environnement::simple(&g);
        let mut j = joueur(100.0, 100.0);
        let mut e = vec![Entite::Eclair(Eclair::new(
            Aabb::new(96.0, 0.0, 24.0, 200.0),
            1000,
            1,
        ))];
        // Pendant toute l'annonce : aucun danger (1 pas de repos + l'annonce + 1 de marge).
        for _ in 0..(ANNONCE_PAS + 2) {
            e[0].actualiser(&env);
            if let Entite::Eclair(x) = &e[0] {
                if x.frappe() {
                    break;
                }
            }
            assert!(resoudre_contacts(&mut j, &mut e).is_empty());
        }
        // Frappe : mort.
        assert!(matches!(&e[0], Entite::Eclair(x) if x.frappe()));
        assert_eq!(
            resoudre_contacts(&mut j, &mut e),
            vec![Contact::Mort(CauseMort::Eclair)]
        );
        // Après la frappe : plus de danger.
        for _ in 0..FRAPPE_PAS {
            e[0].actualiser(&env);
        }
        assert!(resoudre_contacts(&mut j, &mut e).is_empty());
    }

    #[test]
    fn le_champignon_rebondit_plus_haut_qu_un_saut() {
        let mut j = joueur(100.0, 100.0 - 14.0 + 2.0);
        j.vitesse.y = 150.0;
        let mut e = vec![Entite::Champignon(Champignon::new(98.0, 100.0))];
        assert_eq!(resoudre_contacts(&mut j, &mut e), vec![Contact::Rebond]);
        assert_eq!(j.vitesse.y, -VITESSE_CHAMPIGNON);
        match &e[0] {
            Entite::Champignon(c) => assert!(c.est_comprime()),
            _ => unreachable!(),
        }
    }

    #[test]
    fn une_lanterne_donne_un_checkpoint_et_eteint_les_autres() {
        let mut e = vec![
            Entite::Lanterne(Lanterne::new(50.0, 100.0)),
            Entite::Lanterne(Lanterne::new(200.0, 100.0)),
        ];
        let mut j = joueur(48.0, 100.0);
        let c = resoudre_contacts(&mut j, &mut e);
        assert!(matches!(c[..], [Contact::Lanterne(_, _)]));
        // Le joueur va à la seconde lanterne.
        let mut j = joueur(198.0, 100.0);
        resoudre_contacts(&mut j, &mut e);
        let actives: Vec<bool> = e
            .iter()
            .map(|x| matches!(x, Entite::Lanterne(l) if l.est_active()))
            .collect();
        assert_eq!(actives, vec![false, true]);
        // Retoucher la même lanterne n'est pas un nouvel événement.
        assert!(resoudre_contacts(&mut j, &mut e).is_empty());
    }

    #[test]
    fn la_plateforme_friable_s_effrite_quand_on_se_pose_dessus() {
        let f = PlateformeFriable::new(Aabb::new(96.0, 100.0, 32.0, 8.0));
        let mut e = vec![Entite::Friable(f)];
        // Joueur posé dessus : bas = 100.
        let mut j = joueur(100.0, 100.0 - 14.0);
        j.au_sol = true;
        resoudre_contacts(&mut j, &mut e);
        assert!(
            matches!(&e[0], Entite::Friable(p) if p.etat != crate::entities::friable::EtatFriable::Stable)
        );
    }

    #[test]
    fn la_plateforme_ne_revient_pas_dans_le_joueur() {
        use crate::entities::friable::{EtatFriable, ABSENCE_PAS, EFFRITEMENT_PAS};
        let g = GrilleTuiles::vide(20, 20);
        let env = Environnement::simple(&g);
        let mut f = PlateformeFriable::new(Aabb::new(96.0, 100.0, 32.0, 8.0));
        f.toucher();
        let mut e = vec![Entite::Friable(f)];
        for _ in 0..(EFFRITEMENT_PAS + ABSENCE_PAS) {
            e[0].actualiser(&env);
        }
        let mut j = joueur(100.0, 98.0); // dans la zone de la plateforme
        resoudre_contacts(&mut j, &mut e);
        assert!(matches!(&e[0], Entite::Friable(p) if p.etat == EtatFriable::Absente(0)));
        let mut j = joueur(300.0, 0.0);
        resoudre_contacts(&mut j, &mut e);
        assert!(matches!(&e[0], Entite::Friable(p) if p.etat == EtatFriable::Stable));
    }

    #[test]
    fn la_plateforme_mobile_porte_le_joueur() {
        let g = GrilleTuiles::vide(40, 20);
        let mut e = vec![Entite::Mobile(PlateformeMobile::new(
            Aabb::new(0.0, 0.0, 32.0, 8.0),
            (32.0, 100.0),
            (160.0, 100.0),
            60.0,
        ))];
        // Joueur posé sur la plateforme : bas = 100.
        let mut j = joueur(40.0, 100.0 - 14.0);
        j.au_sol = true;
        let x0 = j.boite.x;
        for _ in 0..30 {
            let env = Environnement::simple(&g);
            e[0].actualiser(&env);
            let s = solides(&e);
            let env = Environnement {
                grille: &g,
                solides: &s,
                courants: &[],
            };
            porter_par_plateformes(&mut j, &e, &env);
        }
        // 30 pas à 1 px/pas : le joueur a suivi de 30 px.
        assert!((j.boite.x - x0 - 30.0).abs() < 1e-3, "x = {}", j.boite.x);
        assert!((j.boite.bas() - 100.0).abs() < 1e-3);
    }

    #[test]
    fn une_plateforme_mobile_qui_monte_porte_aussi_le_joueur() {
        let g = GrilleTuiles::vide(40, 20);
        let mut e = vec![Entite::Mobile(PlateformeMobile::new(
            Aabb::new(0.0, 0.0, 32.0, 8.0),
            (32.0, 200.0),
            (32.0, 100.0),
            60.0,
        ))];
        let mut j = joueur(40.0, 200.0 - 14.0);
        j.au_sol = true;
        for _ in 0..30 {
            let env = Environnement::simple(&g);
            e[0].actualiser(&env);
            let s = solides(&e);
            let env = Environnement {
                grille: &g,
                solides: &s,
                courants: &[],
            };
            porter_par_plateformes(&mut j, &e, &env);
        }
        assert!(
            (j.boite.bas() - 170.0).abs() < 1e-2,
            "bas = {}",
            j.boite.bas()
        );
    }

    #[test]
    fn pas_de_portage_en_l_air() {
        let g = GrilleTuiles::vide(40, 20);
        let mut e = vec![Entite::Mobile(PlateformeMobile::new(
            Aabb::new(0.0, 0.0, 32.0, 8.0),
            (32.0, 100.0),
            (160.0, 100.0),
            60.0,
        ))];
        let mut j = joueur(40.0, 100.0 - 14.0);
        j.au_sol = false;
        let env = Environnement::simple(&g);
        e[0].actualiser(&env);
        let x0 = j.boite.x;
        porter_par_plateformes(&mut j, &e, &env);
        assert_eq!(j.boite.x, x0);
    }

    #[test]
    fn la_chute_dans_le_vide_tue() {
        let mut j = joueur(10.0, 10.0);
        assert_eq!(verifier_vide(&j, 160.0), None);
        j.boite.y = 160.0 + 17.0;
        assert_eq!(
            verifier_vide(&j, 160.0),
            Some(Contact::Mort(CauseMort::Vide))
        );
    }

    #[test]
    fn plusieurs_dangers_donnent_une_seule_mort_en_dernier() {
        let mut j = joueur(100.0, 100.0);
        let mut e = vec![
            Entite::Pointe(Pointe::new(100.0, 105.0)),
            Entite::Goutte(GoutteDor::new(1, 102.0, 102.0)),
            Entite::Pointe(Pointe::new(100.0, 104.0)),
        ];
        let c = resoudre_contacts(&mut j, &mut e);
        assert_eq!(
            c,
            vec![Contact::Goutte(1), Contact::Mort(CauseMort::Pointe)]
        );
    }
}
