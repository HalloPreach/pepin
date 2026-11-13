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

use super::{solides_des_entites, Entite, PlateformeMobile};
use crate::aabb::{deplacer, touche_un_obstacle, Aabb};
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
    /// Grêle reçue parapluie fermé.
    Grele,
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
    /// Le joueur a rebondi sur un champignon (ou sur un escargot).
    Rebond,
    /// Un auvent vient de relancer le joueur en diagonale.
    Auvent,
    /// Pépin vient de sauter sur la tête du Grand Nuage essoufflé : un point de
    /// vie en moins pour lui (`GrandNuage::toucher`).
    Nuage,
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

/// Fait suivre au joueur les plateformes mobiles (aiguilles d'horloge comprises,
/// voir `porter_sur_aiguille`) et les nacelles de montgolfière sur lesquelles il
/// se tient.
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
            let avant = p.boite_avant();
            let pose_dessus =
                joueur.boite.decale(0.0, 1.0).chevauche(&avant) && !joueur.boite.chevauche(&avant);
            if pose_dessus {
                if p.cercle().is_some() {
                    porter_sur_aiguille(joueur, p, env);
                } else {
                    joueur.porter(p.deplacement.x, p.deplacement.y, env);
                }
                return; // une seule plateforme porte le joueur
            }
        }
        if let Entite::Montgolfiere(m) = entite {
            if m.deplacement.y == 0.0 {
                continue; // immobile : rien à porter
            }
            let avant = m.boite.decale(0.0, -m.deplacement.y);
            if pose_sur(joueur, &avant) {
                // La nacelle ne bouge qu'en hauteur. Au lieu d'AJOUTER son
                // déplacement (un arrondi de flottant peut alors laisser les pieds
                // un cheveu DANS la nacelle, et la collision repousserait Pépin
                // dessous), on pose ses pieds PILE sur le dessus de la nacelle :
                // `y - h + h` redonne exactement `y`. La nacelle a déjà vérifié
                // (`Montgolfiere::actualiser`) que cette place est libre ; on
                // revérifie quand même contre les solides de ce pas.
                let cible = Aabb::new(
                    joueur.boite.x,
                    m.boite.y - joueur.boite.h,
                    joueur.boite.w,
                    joueur.boite.h,
                );
                if !touche_un_obstacle(&cible, env.grille, env.solides) {
                    joueur.boite = cible;
                }
                return;
            }
        }
    }
}

/// Le joueur est-il posé sur le dessus de `boite` (à 1 px près, sans la chevaucher) ?
fn pose_sur(joueur: &Joueur, boite: &Aabb) -> bool {
    joueur.boite.decale(0.0, 1.0).chevauche(boite) && !joueur.boite.chevauche(boite)
}

/// Porte Pépin, debout sur une aiguille d'horloge, d'un pas.
///
/// L'aiguille bouge en x ET en y à chaque pas (elle suit un cercle). Déplacer
/// Pépin des deux d'un coup, comme pour une plateforme en aller-retour, ne marche
/// pas : la collision traite l'axe X d'abord, et quand l'aiguille MONTE, ses pieds
/// sont déjà un peu dans l'aiguille (qui a bougé avant lui) : il serait repoussé
/// sur le côté, et tomberait. Donc, en deux temps :
/// 1. en x, il suit l'aiguille en ignorant CETTE aiguille (les murs l'arrêtent) ;
/// 2. en y, ses pieds sont posés PILE sur le dessus de l'aiguille, comme dans la
///    nacelle de la montgolfière, si la place est libre et s'il est encore
///    au-dessus d'elle (un mur a pu le retenir pendant qu'elle filait).
fn porter_sur_aiguille(joueur: &mut Joueur, p: &PlateformeMobile, env: &Environnement<'_>) {
    let autres: Vec<Aabb> = env
        .solides
        .iter()
        .copied()
        .filter(|s| *s != p.boite)
        .collect();
    deplacer(&mut joueur.boite, p.deplacement.x, 0.0, env.grille, &autres);
    let b = joueur.boite;
    let cible = Aabb::new(b.x, p.boite.y - b.h, b.w, b.h);
    let au_dessus = cible.x < p.boite.droite() && p.boite.x < cible.droite();
    if au_dessus && !touche_un_obstacle(&cible, env.grille, &autres) {
        joueur.boite = cible;
    }
}

/// Examine tous les contacts de ce pas. Modifie le joueur (rebonds) et les
/// entités (ramassées, écrasées, allumées) et renvoie la liste des événements.
///
/// Ordre de la liste : ordre des entités. Une mort est toujours placée EN DERNIER
/// et il n'y en a qu'une, même si plusieurs dangers touchent en même temps.
///
/// Protection de réapparition (`Joueur::est_protege`) : aucun danger ne tue
/// (ennemi touché de côté, pointe, éclair, grêle, éclair du Grand Nuage) ; tout le
/// reste continue (gouttes, lanternes, champignons, escargots, auvents, écrasement d'un
/// ennemi par le dessus, saut sur la tête du Grand Nuage). Tout NOUVEAU
/// danger ajouté ici doit passer par la même règle. La chute dans le vide n'est
/// pas traitée ici (`verifier_vide`) : elle reste mortelle.
///
/// C'est aussi ici que les pigeons regardent Pépin (`Ennemi::guetter`) : un
/// pigeon perché qui le voit passer dessous commence son annonce, et plongera
/// aux pas suivants (`Entite::actualiser`). Tous les jeux de simulation qui
/// appellent cette fonction (le jeu, le robot de QA, la mesure de difficulté)
/// ont donc les mêmes pigeons, sans rien ajouter.
pub fn resoudre_contacts(joueur: &mut Joueur, entites: &mut [Entite]) -> Vec<Contact> {
    let mut contacts = Vec::new();
    let mut mort: Option<CauseMort> = None;
    let mut lanterne_allumee: Option<usize> = None;
    // Pépin vient-il de réapparaître ? Alors les dangers ci-dessous l'ignorent.
    let protege = joueur.est_protege();

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
                e.guetter(&joueur.boite);
                if e.est_dangereux() && joueur.boite.chevauche(&e.boite) {
                    let par_le_dessus = joueur.vitesse.y > 0.0
                        && joueur.boite.bas() - e.boite.y <= MARGE_ECRASEMENT;
                    // Un chat réveillé ne s'écrase pas : même par le dessus, le
                    // toucher tue (`est_ecrasable` est vrai pour tous les autres).
                    if par_le_dessus && e.est_ecrasable() {
                        e.ecraser();
                        joueur.rebondir(VITESSE_REBOND_ECRASEMENT);
                        contacts.push(Contact::Ecrase);
                    } else if !protege {
                        mort.get_or_insert(CauseMort::Ennemi);
                    }
                }
            }
            Entite::Pointe(p) => {
                if !protege && joueur.boite.chevauche(&p.zone_danger()) {
                    mort.get_or_insert(CauseMort::Pointe);
                }
            }
            Entite::Eclair(e) => {
                if !protege && e.frappe() && joueur.boite.chevauche(&e.zone) {
                    mort.get_or_insert(CauseMort::Eclair);
                }
            }
            Entite::Grele(g) => {
                // Parapluie fermé dans la colonne : mortel. En planant, rien.
                if !protege && g.blesse(joueur) {
                    mort.get_or_insert(CauseMort::Grele);
                }
            }
            Entite::Champignon(c) => {
                if joueur.vitesse.y >= 0.0 && joueur.boite.chevauche(&c.boite) {
                    joueur.rebondir(super::champignon::VITESSE_CHAMPIGNON);
                    c.rebondir();
                    contacts.push(Contact::Rebond);
                }
            }
            Entite::Escargot(s) => {
                // Par le dessus seulement : rebond du champignon (`Contact::Rebond`).
                // De côté, rien : l'escargot est inoffensif et ne s'écrase pas.
                if s.recoit(joueur) {
                    s.lancer(joueur);
                    contacts.push(Contact::Rebond);
                }
            }
            Entite::Auvent(a) => {
                // Par le dessus seulement ; une fois relancé, Pépin monte : les
                // autres auvents de la liste ne le reprennent pas dans le même pas.
                if a.recoit(joueur) {
                    a.lancer(joueur);
                    contacts.push(Contact::Auvent);
                }
            }
            Entite::Friable(f) => {
                if pose_sur(joueur, &f.boite) {
                    f.toucher();
                }
                f.reapparaitre_si_libre(&joueur.boite);
            }
            Entite::Mobile(_) => {}
            // La nacelle est un SOLIDE (géré par `solides_des_entites`) ; elle
            // regarde si Pépin est à bord APRÈS cette boucle (un rebond plus loin
            // dans la liste peut encore le faire décoller).
            Entite::Montgolfiere(_) => {}
            // La casette du ventilateur est un SOLIDE (on s'y pose), géré par
            // `solides_des_entites` : aucun contact ni danger ici.
            Entite::Ventilateur(_) => {}
            Entite::Nuage(n) => {
                // Sa tête, pendant l'essoufflement : un point de moins, et Pépin
                // rebondit comme sur un ennemi écrasé. Son corps ne blesse pas :
                // seuls ses éclairs (colonnes qui frappent) sont dangereux.
                if n.recoit(joueur) {
                    n.toucher();
                    joueur.rebondir(VITESSE_REBOND_ECRASEMENT);
                    contacts.push(Contact::Nuage);
                } else if !protege && n.foudroie(&joueur.boite) {
                    mort.get_or_insert(CauseMort::Eclair);
                }
            }
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

    // Montgolfières : Pépin est-il posé sur la nacelle à la fin de ce pas ? (La
    // réponse décide si elle monte ou redescend au pas suivant.)
    for entite in entites.iter_mut() {
        if let Entite::Montgolfiere(m) = entite {
            m.embarquer(joueur);
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
    fn le_pigeon_s_ecrase_par_le_dessus_et_tue_autrement() {
        // Perché (boîte 12x10 en (100, 100)) : on lui tombe dessus, il est écrasé.
        let mut j = joueur(100.0, 100.0 - 14.0 + 3.0);
        j.vitesse.y = 200.0;
        let mut e = vec![Entite::Ennemi(Ennemi::pigeon(100.0, 100.0, false))];
        assert_eq!(resoudre_contacts(&mut j, &mut e), vec![Contact::Ecrase]);
        assert_eq!(j.vitesse.y, -VITESSE_REBOND_ECRASEMENT);
        assert!(matches!(&e[0], Entite::Ennemi(p) if !p.est_dangereux()));

        // Touché de côté, ou en montant dans lui : mort.
        let mut j = joueur(95.0, 96.0);
        let mut e = vec![Entite::Ennemi(Ennemi::pigeon(100.0, 100.0, false))];
        assert_eq!(
            resoudre_contacts(&mut j, &mut e),
            vec![Contact::Mort(CauseMort::Ennemi)]
        );
        let mut j = joueur(100.0, 104.0);
        j.vitesse.y = -100.0;
        let mut e = vec![Entite::Ennemi(Ennemi::pigeon(100.0, 100.0, false))];
        assert_eq!(
            resoudre_contacts(&mut j, &mut e),
            vec![Contact::Mort(CauseMort::Ennemi)]
        );
    }

    // --- Chat endormi (carte 5.9) ---

    /// Chat (boîte 14 x 8 en (100, 100)) avancé de `pas` pas dans son cycle.
    fn chat_apres(pas: u16) -> Entite {
        let g = GrilleTuiles::vide(16, 16);
        let env = Environnement::simple(&g);
        let mut c = Entite::Ennemi(Ennemi::chat(100.0, 100.0, 0));
        for _ in 0..pas {
            c.actualiser(&env);
        }
        c
    }

    /// Pépin qui tombe sur le chat, le bas enfoncé de 3 px dans son dos.
    fn pepin_tombe_sur_le_chat() -> Joueur {
        let mut j = joueur(102.0, 100.0 - 14.0 + 3.0);
        j.vitesse.y = 200.0;
        j
    }

    #[test]
    fn le_chat_endormi_s_ecrase_par_le_dessus_et_tue_de_cote() {
        use crate::entities::ennemi::{DORT_PAS, REVEIL_PAS};
        // Endormi (début et fin du sommeil) et pendant l'annonce du réveil :
        // écrasé, Pépin rebondit comme sur les autres ennemis.
        for pas in [0, DORT_PAS - 1, DORT_PAS, DORT_PAS + REVEIL_PAS - 1] {
            let mut e = vec![chat_apres(pas)];
            let mut j = pepin_tombe_sur_le_chat();
            assert_eq!(
                resoudre_contacts(&mut j, &mut e),
                vec![Contact::Ecrase],
                "pas {pas}"
            );
            assert_eq!(j.vitesse.y, -VITESSE_REBOND_ECRASEMENT);
            assert!(matches!(&e[0], Entite::Ennemi(c) if !c.est_dangereux()));
        }
        // Heurté de côté pendant son sommeil : mort (comme une flaque).
        let mut e = vec![chat_apres(0)];
        let mut j = joueur(92.0, 94.0);
        assert_eq!(
            resoudre_contacts(&mut j, &mut e),
            vec![Contact::Mort(CauseMort::Ennemi)]
        );
    }

    #[test]
    fn le_chat_eveille_tue_meme_par_le_dessus_et_ne_s_ecrase_pas() {
        use crate::entities::ennemi::{DORT_PAS, EVEILLE_PAS, REVEIL_PAS};
        let reveil = DORT_PAS + REVEIL_PAS;
        for pas in [reveil, reveil + EVEILLE_PAS - 1] {
            let mut e = vec![chat_apres(pas)];
            let mut j = pepin_tombe_sur_le_chat();
            assert_eq!(
                resoudre_contacts(&mut j, &mut e),
                vec![Contact::Mort(CauseMort::Ennemi)],
                "pas {pas}"
            );
            assert_eq!(j.vitesse.y, 200.0, "aucun rebond");
            assert!(matches!(&e[0], Entite::Ennemi(c) if c.est_dangereux()));
        }
        // Un pas plus tard, il se rendort : de nouveau écrasable.
        let mut e = vec![chat_apres(reveil + EVEILLE_PAS)];
        let mut j = pepin_tombe_sur_le_chat();
        assert_eq!(resoudre_contacts(&mut j, &mut e), vec![Contact::Ecrase]);
    }

    #[test]
    fn protege_le_chat_eveille_ne_tue_pas() {
        use crate::entities::ennemi::{DORT_PAS, REVEIL_PAS};
        let mut e = vec![chat_apres(DORT_PAS + REVEIL_PAS)];
        let mut j = pepin_tombe_sur_le_chat();
        j.proteger();
        assert!(resoudre_contacts(&mut j, &mut e).is_empty());
        // Endormi, l'écrasement continue malgré la protection (comme les autres).
        let mut e = vec![chat_apres(0)];
        let mut j = pepin_tombe_sur_le_chat();
        j.proteger();
        assert_eq!(resoudre_contacts(&mut j, &mut e), vec![Contact::Ecrase]);
    }

    #[test]
    fn le_pigeon_qui_plonge_sur_pepin_le_tue_et_s_ecrase_en_vol_par_le_dessus() {
        use crate::entities::ennemi::{PhasePigeon, ANNONCE_PIGEON_PAS};
        let g = GrilleTuiles::vide(30, 20);
        let env = Environnement::simple(&g);
        // Pigeon tourné vers la gauche en (200, 40) ; Pépin immobile sous lui,
        // sur sa diagonale (40 px plus bas, 40 px devant).
        let mut e = vec![Entite::Ennemi(Ennemi::pigeon(200.0, 40.0, false))];
        let mut j = joueur(160.0, 90.0);
        // `resoudre_contacts` le fait guetter : l'annonce commence.
        assert!(resoudre_contacts(&mut j, &mut e).is_empty());
        assert!(matches!(&e[0], Entite::Ennemi(p)
            if p.phase() == PhasePigeon::Annonce(ANNONCE_PIGEON_PAS)));
        let mut mort_au_pas = None;
        for pas in 0..120 {
            e[0].actualiser(&env);
            if resoudre_contacts(&mut j, &mut e) == vec![Contact::Mort(CauseMort::Ennemi)] {
                mort_au_pas = Some(pas);
                break;
            }
        }
        let pas = mort_au_pas.expect("la plongée atteint Pépin");
        assert!(
            pas >= ANNONCE_PIGEON_PAS as u32,
            "pas avant la fin de l'annonce ({pas})"
        );

        // En pleine plongée, Pépin qui lui retombe dessus l'écrase quand même.
        let mut e = vec![Entite::Ennemi(Ennemi::pigeon(200.0, 40.0, false))];
        let mut loin = joueur(170.0, 100.0);
        resoudre_contacts(&mut loin, &mut e);
        for _ in 0..(ANNONCE_PIGEON_PAS as u32 + 10) {
            e[0].actualiser(&env);
        }
        let b = e[0].boite();
        let mut j = joueur(b.x, b.y - 14.0 + 3.0);
        j.vitesse.y = 200.0;
        assert_eq!(resoudre_contacts(&mut j, &mut e), vec![Contact::Ecrase]);
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
                vents: &[],
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
                vents: &[],
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

    /// Un éclair en pleine frappe sur la colonne x = 96..120.
    fn eclair_qui_frappe() -> Entite {
        use crate::entities::Eclair;
        let g = GrilleTuiles::vide(20, 20);
        let env = Environnement::simple(&g);
        let mut e = Entite::Eclair(Eclair::new(Aabb::new(96.0, 0.0, 24.0, 200.0), 1000, 1));
        while !matches!(&e, Entite::Eclair(x) if x.frappe()) {
            e.actualiser(&env);
        }
        e
    }

    /// Juste après une réapparition, Pépin est protégé : flaque touchée de côté,
    /// pointes et éclair ne le tuent pas ; sans protection, les mêmes contacts tuent.
    /// Le reste continue : goutte ramassée, flaque écrasée par le dessus.
    #[test]
    fn protege_aucun_danger_ne_tue_mais_le_reste_continue() {
        let dangers = || {
            vec![
                Entite::Ennemi(Ennemi::new(TypeEnnemi::Flaque, 100.0, 100.0)),
                Entite::Pointe(Pointe::new(100.0, 104.0)),
                eclair_qui_frappe(),
                Entite::Goutte(GoutteDor::new(0, 102.0, 102.0)),
            ]
        };
        // Sans protection : mort (une seule, en dernier).
        let mut j = joueur(95.0, 96.0);
        let mut e = dangers();
        assert_eq!(
            resoudre_contacts(&mut j, &mut e),
            vec![Contact::Goutte(0), Contact::Mort(CauseMort::Ennemi)]
        );
        // Protégé, même position, mêmes dangers : seulement la goutte.
        let mut j = joueur(95.0, 96.0);
        j.proteger();
        let mut e = dangers();
        assert_eq!(resoudre_contacts(&mut j, &mut e), vec![Contact::Goutte(0)]);
        assert!(
            matches!(&e[0], Entite::Ennemi(f) if f.est_dangereux()),
            "la flaque touchée de côté n'est pas écrasée"
        );
        // Protégé, il peut toujours écraser une flaque en tombant dessus.
        let mut j = joueur(100.0, 100.0 - 14.0 + 3.0);
        j.vitesse.y = 200.0;
        j.proteger();
        let mut e = vec![Entite::Ennemi(Ennemi::new(
            TypeEnnemi::Flaque,
            100.0,
            100.0,
        ))];
        assert_eq!(resoudre_contacts(&mut j, &mut e), vec![Contact::Ecrase]);
    }

    /// Grêle : parapluie fermé dans la colonne, mort (cause `Grele`) ; en planant
    /// au même endroit, rien ; protégé après une réapparition, rien non plus (la
    /// règle commune à tous les dangers).
    #[test]
    fn la_grele_tue_parapluie_ferme_et_epargne_le_plane() {
        use crate::entities::Grele;
        let colonne = || vec![Entite::Grele(Grele::new(Aabb::new(98.0, 0.0, 12.0, 160.0)))];
        let mut j = joueur(100.0, 80.0);
        let mut e = colonne();
        assert_eq!(
            resoudre_contacts(&mut j, &mut e),
            vec![Contact::Mort(CauseMort::Grele)]
        );
        let mut j = joueur(100.0, 80.0);
        j.plane = true;
        assert!(resoudre_contacts(&mut j, &mut colonne()).is_empty());
        let mut j = joueur(100.0, 80.0);
        j.proteger();
        assert!(resoudre_contacts(&mut j, &mut colonne()).is_empty());
        // Hors de la colonne, parapluie fermé : rien.
        let mut j = joueur(120.0, 80.0);
        assert!(resoudre_contacts(&mut j, &mut colonne()).is_empty());
    }

    /// La chute dans le vide reste mortelle pendant la protection (sinon Pépin
    /// tomberait sans fin sous le monde).
    #[test]
    fn le_vide_tue_meme_protege() {
        let mut j = joueur(10.0, 160.0 + 17.0);
        j.proteger();
        assert!(j.est_protege());
        assert_eq!(
            verifier_vide(&j, 160.0),
            Some(Contact::Mort(CauseMort::Vide))
        );
    }

    // --- Aiguilles d'horloge (carte 5.10) : mini-niveau de test ---

    /// Mini-niveau : un pivot `h` au centre de la case (7, 5), soit (120, 88) ;
    /// aiguille de 2 tuiles de rayon à 30 px/s. Sol en ligne 11 (dessus à y = 176) :
    /// à 6 h, le bas de la plateforme est à y = 124 : un saut depuis le sol (57 px)
    /// y cogne la tête de Pépin.
    const MINI_HORLOGE: &str = "
...............
...............
...............
...............
...............
.......h.......
...............
...............
...............
...............
.@.o.o.o.....X.
###############
";

    /// Le mini-niveau, son aiguille partie de l'heure `phase`, et ses entités.
    fn mini_horloge(phase: f32) -> (crate::level::Niveau, Vec<Entite>) {
        let reglage = crate::level::ReglageAiguille {
            rayon: 2.0,
            vitesse: 30.0,
            phase,
        };
        let n = crate::level::Niveau::depuis_ascii_avec_aiguilles(MINI_HORLOGE, &[reglage])
            .expect("mini-niveau valide");
        let e = n.entites.clone();
        (n, e)
    }

    /// L'aiguille du mini-niveau (seule plateforme mobile).
    fn aiguille(e: &[Entite]) -> PlateformeMobile {
        e.iter()
            .find_map(|x| match x {
                Entite::Mobile(m) => Some(m.clone()),
                _ => None,
            })
            .expect("une aiguille")
    }

    /// Un pas de simulation complet, dans l'ordre du jeu (voir l'en-tête de
    /// `entities`) : les entités bougent, le joueur est porté, il bouge, contacts.
    fn pas_du_jeu(
        n: &crate::level::Niveau,
        e: &mut [Entite],
        j: &mut Joueur,
        entrees: &crate::player::Entrees,
    ) {
        let env = Environnement::simple(n);
        for x in e.iter_mut() {
            x.actualiser(&env);
        }
        let s = solides(e);
        let env = Environnement {
            grille: n,
            solides: &s,
            courants: &[],
            vents: &[],
        };
        porter_par_plateformes(j, e, &env);
        j.actualiser(entrees, &env);
        resoudre_contacts(j, e);
    }

    /// Debout sur l'aiguille (partie de midi), sans toucher à rien, Pépin fait
    /// deux tours complets : à chaque pas il a les pieds PILE sur le dessus de la
    /// plateforme, au même endroit de la plateforme (11 px de son bord gauche).
    /// Il monte et descend de toute la hauteur du cercle (64 px).
    #[test]
    fn l_aiguille_porte_pepin_pendant_deux_tours() {
        let (n, mut e) = mini_horloge(0.0);
        let p = aiguille(&e);
        let mut j = joueur(p.boite.x + 11.0, p.boite.y - 14.0);
        j.au_sol = true;
        let pas = (2.0 * p.cercle().unwrap().pas_par_tour()) as u32;
        let (mut haut, mut bas) = (f32::MAX, f32::MIN);
        for k in 0..pas {
            pas_du_jeu(&n, &mut e, &mut j, &crate::player::Entrees::default());
            let p = aiguille(&e);
            assert!(j.au_sol, "pas {k} : Pépin n'est plus sur l'aiguille");
            assert!(
                (j.boite.bas() - p.boite.y).abs() < 1e-3,
                "pas {k} : pieds à {} pour un dessus à {}",
                j.boite.bas(),
                p.boite.y
            );
            assert!(
                (j.boite.x - p.boite.x - 11.0).abs() < 0.01,
                "pas {k} : Pépin glisse sur l'aiguille ({} / {})",
                j.boite.x,
                p.boite.x
            );
            haut = haut.min(j.boite.bas());
            bas = bas.max(j.boite.bas());
        }
        assert!((bas - haut - 64.0).abs() < 0.1, "{haut} -> {bas}");
    }

    /// Pas de traversée par le dessus : lâché 40 px au-dessus de l'aiguille, à
    /// pleine vitesse de chute, à chacune des 12 heures du cadran (elle monte, elle
    /// descend, elle file de côté), Pépin ne la chevauche jamais et finit posé dessus.
    #[test]
    fn pepin_tombe_sur_l_aiguille_sans_la_traverser() {
        for heure in 0..12 {
            let (n, mut e) = mini_horloge(heure as f32);
            let p = aiguille(&e);
            let mut j = joueur(p.boite.x + 11.0, p.boite.y - 14.0 - 40.0);
            j.vitesse.y = crate::physics::VITESSE_MAX_CHUTE;
            for k in 0..40 {
                pas_du_jeu(&n, &mut e, &mut j, &crate::player::Entrees::default());
                let p = aiguille(&e);
                assert!(
                    !j.boite.chevauche(&p.boite),
                    "{heure} h, pas {k} : Pépin est dans l'aiguille"
                );
            }
            let p = aiguille(&e);
            assert!(j.au_sol, "{heure} h : pas posé");
            assert!(
                (j.boite.bas() - p.boite.y).abs() < 1e-3,
                "{heure} h : posé à {} au lieu de {}",
                j.boite.bas(),
                p.boite.y
            );
        }
    }

    /// Pas de traversée par le dessous : Pépin saute sous l'aiguille (partie de
    /// 5 h 30, elle passe au plus bas pendant le saut) ; il s'y cogne la tête sans
    /// jamais la chevaucher, puis retombe au sol.
    #[test]
    fn pepin_se_cogne_sous_l_aiguille_sans_la_traverser() {
        let (n, mut e) = mini_horloge(5.5);
        let mut j = joueur(115.0, 176.0 - 14.0);
        let saut = crate::player::Entrees {
            direction: crate::player::Direction::Immobile,
            saut: true,
        };
        for _ in 0..5 {
            pas_du_jeu(&n, &mut e, &mut j, &crate::player::Entrees::default());
        }
        assert!(j.au_sol);
        let mut cogne = false;
        for k in 0..120 {
            let entrees = if k < 30 {
                saut
            } else {
                crate::player::Entrees::default()
            };
            pas_du_jeu(&n, &mut e, &mut j, &entrees);
            let p = aiguille(&e);
            assert!(
                !j.boite.chevauche(&p.boite),
                "pas {k} : Pépin est dans l'aiguille"
            );
            cogne |= (j.boite.y - p.boite.bas()).abs() < 1e-3;
        }
        assert!(cogne, "Pépin aurait dû se cogner sous l'aiguille");
        assert!(j.au_sol && j.boite.bas() == 176.0, "retombé au sol");
    }

    // --- Grand Nuage (carte 5.11), sur une petite arène de test ---

    /// Une arène d'un écran de large : le `O` au milieu (colonne 9), corps en
    /// (128, 16), tête à y = 16 ; courants en x = 80 et x = 192 ; sol à y = 96.
    const MINI_ARENE: &str = r"
...................
.........O.........
...................
...................
...................
.@..o.o.o.........X
###################
";

    /// L'arène (sans le drapeau : l'arène du nuage n'en a pas) et ses entités.
    fn mini_arene() -> (crate::level::Niveau, Vec<Entite>) {
        let n = crate::level::Niveau::depuis_ascii(&MINI_ARENE.replace('X', "."))
            .expect("arène valide");
        let e = n.entites.clone();
        (n, e)
    }

    fn le_nuage(e: &[Entite]) -> &crate::entities::GrandNuage {
        e.iter()
            .find_map(|x| match x {
                Entite::Nuage(g) => Some(g),
                _ => None,
            })
            .expect("un nuage")
    }

    fn le_nuage_mut(e: &mut [Entite]) -> &mut crate::entities::GrandNuage {
        e.iter_mut()
            .find_map(|x| match x {
                Entite::Nuage(g) => Some(g),
                _ => None,
            })
            .expect("un nuage")
    }

    /// Un pas complet dans l'ordre du jeu, AVEC les courants des entités (celui du
    /// nuage essoufflé). Renvoie les contacts du pas.
    fn pas_de_l_arene(
        n: &crate::level::Niveau,
        e: &mut [Entite],
        j: &mut Joueur,
        entrees: &crate::player::Entrees,
    ) -> Vec<Contact> {
        let env = Environnement::simple(n);
        for x in e.iter_mut() {
            x.actualiser(&env);
        }
        let s = solides(e);
        let courants = crate::entities::courants_des_entites(e);
        let env = Environnement {
            grille: n,
            solides: &s,
            courants: &courants,
            vents: &[],
        };
        porter_par_plateformes(j, e, &env);
        j.actualiser(entrees, &env);
        resoudre_contacts(j, e)
    }

    /// Essoufflé, le nuage laisse échapper son courant à gauche : Pépin, posé au
    /// sol dedans, monte plus haut que la tête du nuage ; il file vers la droite et
    /// lui retombe dessus : UN coup (2 points restants), un rebond, aucune mort.
    #[test]
    fn pepin_monte_dans_le_courant_et_tape_sur_la_tete_du_nuage() {
        use crate::entities::nuage::PhaseNuage;
        use crate::player::{Direction, Entrees};
        let (n, mut e) = mini_arene();
        le_nuage_mut(&mut e).essouffler();
        let tete = le_nuage(&e).boite.y;
        // Au sol dans le courant de gauche (x de 80 à 112), loin des gouttes.
        let mut j = joueur(82.0, 96.0 - 14.0);
        let mut plus_haut = f32::MAX;
        for _ in 0..60 {
            let c = pas_de_l_arene(&n, &mut e, &mut j, &Entrees::default());
            assert!(c.is_empty(), "{c:?}");
            plus_haut = plus_haut.min(j.boite.bas());
        }
        assert!(
            plus_haut < tete,
            "pieds montés à {plus_haut}, tête à {tete}"
        );
        let droite = Entrees {
            direction: Direction::Droite,
            saut: false,
        };
        let mut coups = 0;
        for _ in 0..90 {
            let c = pas_de_l_arene(&n, &mut e, &mut j, &droite);
            assert!(!c.iter().any(|x| matches!(x, Contact::Mort(_))), "{c:?}");
            if c.contains(&Contact::Nuage) {
                coups += 1;
                assert!(j.vitesse.y < 0.0, "Pépin rebondit sur sa tête");
            }
        }
        assert_eq!(coups, 1, "un seul coup par essoufflement");
        let g = le_nuage(&e);
        assert_eq!((g.pv(), g.phase()), (2, PhaseNuage::Touche));
        assert!(g.courant_actif().is_none(), "touché : plus de courant");
    }

    /// Hors de l'essoufflement, son courant n'existe pas : Pépin reste au sol.
    #[test]
    fn pas_de_courant_hors_de_l_essoufflement() {
        let (n, mut e) = mini_arene();
        let mut j = joueur(82.0, 96.0 - 14.0);
        for _ in 0..60 {
            pas_de_l_arene(&n, &mut e, &mut j, &crate::player::Entrees::default());
        }
        assert!(j.au_sol && j.boite.bas() == 96.0);
    }

    /// Ses éclairs tuent pendant la frappe seulement (cause : éclair), et jamais
    /// Pépin protégé ; l'annonce (48 pas) est sans danger. Son corps ne blesse pas.
    #[test]
    fn les_eclairs_du_nuage_tuent_pendant_la_frappe() {
        use crate::entities::nuage::{EtatColonne, PRELUDE_PAS};
        use crate::physics::Environnement as Env;
        let (n, mut e) = mini_arene();
        // Pépin debout dans la 1re colonne (la plus à gauche : x de 18 à 30).
        let au_sol = joueur(19.0, 96.0 - 14.0);
        let env = Env::simple(&n);
        for _ in 1..PRELUDE_PAS {
            for x in e.iter_mut() {
                x.actualiser(&env);
            }
            let mut j = au_sol.clone();
            assert!(resoudre_contacts(&mut j, &mut e).is_empty());
            assert_eq!(le_nuage(&e).etat_colonne(0), EtatColonne::Calme);
        }
        // Le motif 1 commence : la colonne 0 est annoncée 48 pas, sans danger.
        let mut annonce = 0;
        loop {
            for x in e.iter_mut() {
                x.actualiser(&env);
            }
            let mut j = au_sol.clone();
            let c = resoudre_contacts(&mut j, &mut e);
            match le_nuage(&e).etat_colonne(0) {
                EtatColonne::Annonce(_) => {
                    annonce += 1;
                    assert!(c.is_empty(), "{c:?}");
                }
                EtatColonne::Frappe => {
                    assert_eq!(c, vec![Contact::Mort(CauseMort::Eclair)]);
                    let mut protege = au_sol.clone();
                    protege.proteger();
                    assert!(resoudre_contacts(&mut protege, &mut e).is_empty());
                    // Au milieu de l'arène (hors de la 1re salve), rien.
                    let mut milieu = joueur(150.0, 96.0 - 14.0);
                    assert!(resoudre_contacts(&mut milieu, &mut e).is_empty());
                    break;
                }
                EtatColonne::Calme => panic!("colonne 0 calme pendant la 1re salve"),
            }
        }
        assert_eq!(annonce, 48);
        // Dans le corps du nuage (de côté), rien : seuls ses éclairs blessent.
        let mut dedans = joueur(130.0, 20.0);
        assert!(resoudre_contacts(&mut dedans, &mut e).is_empty());
    }

    /// Une mort ramène le nuage au prélude, sans lui rendre ses points ; pendant la
    /// déchirure du ciel, une mort ne change rien.
    #[test]
    fn apres_une_mort_le_nuage_reprend_avec_ses_points_perdus() {
        use crate::entities::nuage::PhaseNuage;
        let (_, mut e) = mini_arene();
        le_nuage_mut(&mut e).essouffler();
        le_nuage_mut(&mut e).toucher();
        crate::entities::reprendre_apres_une_mort(&mut e);
        let g = le_nuage(&e);
        assert_eq!((g.phase(), g.pv(), g.ecoule()), (PhaseNuage::Prelude, 2, 0));
        le_nuage_mut(&mut e).essouffler();
        le_nuage_mut(&mut e).toucher();
        le_nuage_mut(&mut e).essouffler();
        le_nuage_mut(&mut e).toucher();
        crate::entities::reprendre_apres_une_mort(&mut e);
        assert_eq!(le_nuage(&e).phase(), PhaseNuage::Dechirure);
    }
}
