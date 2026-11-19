//! Module `entities` : tout ce qui vit dans un niveau en dehors du joueur et des tuiles.
//!
//! Le module est découpé en petits fichiers (un par famille d'entités) :
//! - `ennemi` : les flaques patrouilleuses (écrasables) et les corbeaux volants ;
//! - `pointe`, `eclair`, `grele` : les dangers fixes, temporisés et continus ;
//! - `friable`, `mobile` : les plateformes qui s'effritent ou qui voyagent ;
//! - `montgolfiere` : la nacelle qui monte avec Pépin et redescend sans lui ;
//! - `champignon` : le trampoline ; `auvent` : la toile qui relance en diagonale ;
//! - `escargot` : le tremplin qui marche (un champignon qui patrouille) ;
//! - `nuage` : le Grand Nuage, boss de l'œil du cyclone (motifs d'éclairs,
//!   essoufflement et courant ascendant, trois points de vie) ;
//! - `marche` : la marche au sol partagée par la flaque et l'escargot ;
//! - `collectibles` : les gouttes d'or, les lettres cachées et les lanternes (checkpoints) ;
//! - les mécaniques d'eau du Monde 2 (carte 9.3) : `bulles` (colonne qui
//!   soulève), `nenuphar` (feuille qui s'enfonce), `maree` (niveau d'eau
//!   cyclique), `stalactite` (pointe qui tombe du plafond), `chute` (chute
//!   d'eau qui pousse vers le bas) ;
//! - `contacts` : ce qui se passe quand le joueur touche une entité.
//!
//! Ordre conseillé pour UN pas de simulation (60 Hz), à respecter par le jeu :
//! 1. `Entite::actualiser` sur chaque entité (les plateformes bougent, les éclairs avancent) ;
//! 2. `contacts::porter_par_plateformes` (le joueur suit sa plateforme mobile ou
//!    la nacelle de sa montgolfière) ;
//! 3. `Joueur::actualiser` (le joueur bouge) ;
//! 4. `contacts::resoudre_contacts` (ramassage, écrasement, dégâts... et chaque
//!    montgolfière note si Pépin est à bord) ;
//! 5. `EtatJeu::traiter` (score, checkpoint, mort et réapparition).
//!
//! Notions Rust illustrées ici (à reprendre dans le README) :
//! - `trait` : le contrat `Actualisable` (« je sais avancer d'un pas de temps ») ;
//! - `enum` à données : `Entite` est UNE des familles, chacune portant ses
//!   propres champs ; `match` oblige à traiter toutes les variantes ;
//! - ownership : l'enum POSSÈDE l'entité qu'il contient (pas de pointeur partagé) ;
//! - emprunts : `&self` pour lire, `&mut self` pour modifier, `&[T]` / `&mut [T]`
//!   pour parcourir une tranche sans en prendre possession.

pub mod auvent;
pub mod bulles;
pub mod champignon;
pub mod chute;
pub mod collectibles;
pub mod contacts;
pub mod eclair;
pub mod ennemi;
pub mod escargot;
pub mod friable;
pub mod grele;
mod marche;
pub mod maree;
pub mod mobile;
pub mod montgolfiere;
pub mod nenuphar;
pub mod nuage;
pub mod pieuvre;
pub mod pointe;
pub mod stalactite;
pub mod ventilateur;

use crate::aabb::Aabb;
use crate::physics::{CourantAscendant, Eau, Environnement};

pub use auvent::Auvent;
pub use bulles::Bulles;
pub use champignon::Champignon;
pub use chute::Chute;
pub use collectibles::{GoutteDor, Lanterne, Lettre};
pub use eclair::Eclair;
pub use ennemi::{Ennemi, TypeEnnemi};
pub use escargot::Escargot;
pub use friable::PlateformeFriable;
pub use grele::Grele;
pub use maree::Maree;
pub use mobile::PlateformeMobile;
pub use montgolfiere::Montgolfiere;
pub use nenuphar::Nenuphar;
pub use nuage::GrandNuage;
pub use pieuvre::Pieuvre;
pub use pointe::Pointe;
pub use stalactite::Stalactite;
pub use ventilateur::Ventilateur;

/// Contrat : « savoir avancer d'un pas de simulation fixe (1/60 s) ».
///
/// Un `trait` décrit un comportement que plusieurs types peuvent offrir. Le jeu
/// peut ainsi boucler sur toutes les entités sans savoir de quel type elles sont.
///
/// - `&mut self` : on a besoin d'un emprunt EXCLUSIF en écriture (l'entité change) ;
/// - `env: &Environnement` : un simple PRÊT en lecture du niveau (grille, solides).
pub trait Actualisable {
    /// Avance l'entité d'un pas de temps (`physics::DT`).
    fn actualiser(&mut self, env: &Environnement<'_>);
}

/// Une entité du niveau. Chaque variante contient (possède) une structure différente.
///
/// Avantage de l'enum face à une collection d'objets de types variés : tout est
/// rangé dans un seul `Vec<Entite>`, contigu en mémoire, sans allocation par entité.
#[derive(Debug, Clone)]
pub enum Entite {
    Ennemi(Ennemi),
    Pointe(Pointe),
    Eclair(Eclair),
    Grele(Grele),
    Friable(PlateformeFriable),
    Mobile(PlateformeMobile),
    Montgolfiere(Montgolfiere),
    Champignon(Champignon),
    Escargot(Escargot),
    Auvent(Auvent),
    Ventilateur(Ventilateur),
    /// Le Grand Nuage est bien plus gros que les autres entités (ses 17 colonnes
    /// d'éclairs) : il vit dans une `Box`, sur le tas. Sans elle, CHAQUE case du
    /// `Vec<Entite>` aurait sa taille (un `enum` est aussi grand que sa plus grande
    /// variante) ; avec, la variante n'est plus qu'un pointeur.
    Nuage(Box<GrandNuage>),
    Goutte(GoutteDor),
    Lanterne(Lanterne),
    /// Lettre cachée (carte 8.6).
    Lettre(Lettre),
    /// Colonne de bulles qui soulève (carte 9.3, `Y`).
    Bulles(Bulles),
    /// Nénuphar qui s'enfonce sous Pépin (carte 9.3, `n`).
    Nenuphar(Nenuphar),
    /// Niveau d'eau cyclique (carte 9.3, `=`).
    Maree(Maree),
    /// Stalactite qui tombe (carte 9.3, `V`).
    Stalactite(Stalactite),
    /// Chute d'eau qui pousse vers le bas (carte 9.3, `|`).
    Chute(Chute),
    /// La Pieuvre du port, boss du Monde 2 (carte 9.12a, `8`) : dans une `Box`,
    /// comme le Grand Nuage (ses 9 colonnes et ses 3 places de tête).
    Pieuvre(Box<Pieuvre>),
}

/// `Entite` implémente le trait en DÉLÉGUANT à la variante réelle.
impl Actualisable for Entite {
    fn actualiser(&mut self, env: &Environnement<'_>) {
        // `match self` avec `self: &mut Entite` : chaque `e` est un `&mut` vers
        // l'entité interne (le compilateur gère les emprunts pour nous).
        match self {
            Entite::Ennemi(e) => e.actualiser(env),
            Entite::Pointe(e) => e.actualiser(env),
            Entite::Eclair(e) => e.actualiser(env),
            Entite::Grele(e) => e.actualiser(env),
            Entite::Friable(e) => e.actualiser(env),
            Entite::Mobile(e) => e.actualiser(env),
            Entite::Montgolfiere(e) => e.actualiser(env),
            Entite::Champignon(e) => e.actualiser(env),
            Entite::Escargot(e) => e.actualiser(env),
            Entite::Auvent(e) => e.actualiser(env),
            Entite::Ventilateur(e) => e.actualiser(env),
            Entite::Nuage(e) => e.actualiser(env),
            Entite::Pieuvre(e) => e.actualiser(env),
            Entite::Goutte(e) => e.actualiser(env),
            Entite::Lanterne(e) => e.actualiser(env),
            Entite::Lettre(e) => e.actualiser(env),
            Entite::Bulles(e) => e.actualiser(env),
            Entite::Nenuphar(e) => e.actualiser(env),
            Entite::Maree(e) => e.actualiser(env),
            Entite::Stalactite(e) => e.actualiser(env),
            Entite::Chute(e) => e.actualiser(env),
        }
    }
}

impl Entite {
    /// Boîte principale de l'entité (pour l'affichage et les tests rapides).
    pub fn boite(&self) -> Aabb {
        match self {
            Entite::Ennemi(e) => e.boite,
            Entite::Pointe(e) => e.boite,
            Entite::Eclair(e) => e.zone,
            Entite::Grele(e) => e.zone,
            Entite::Friable(e) => e.boite,
            Entite::Mobile(e) => e.boite,
            Entite::Montgolfiere(e) => e.boite,
            Entite::Champignon(e) => e.boite,
            Entite::Escargot(e) => e.boite,
            Entite::Auvent(e) => e.boite,
            Entite::Ventilateur(e) => e.boite,
            Entite::Nuage(e) => e.boite,
            // La place de la tête en cours (elle ne bouge qu'en changeant de phase).
            Entite::Pieuvre(e) => e.tete().unwrap_or(e.places[0]),
            Entite::Goutte(e) => e.boite,
            Entite::Lanterne(e) => e.boite,
            Entite::Lettre(e) => e.boite,
            Entite::Bulles(e) => e.zone,
            Entite::Nenuphar(e) => e.boite,
            Entite::Maree(e) => e.zone,
            Entite::Stalactite(e) => e.boite,
            Entite::Chute(e) => e.zone,
        }
    }

    /// Boîte SOLIDE de l'entité si elle en a une en ce moment (sinon `None`).
    /// `Option<Aabb>` : « une boîte, ou rien », pas de valeur magique.
    pub fn solide(&self) -> Option<Aabb> {
        match self {
            Entite::Friable(p) => p.solide(),
            Entite::Mobile(p) => Some(p.boite),
            // La nacelle seulement : le ballon au-dessus n'est qu'un dessin.
            Entite::Montgolfiere(m) => Some(m.boite),
            Entite::Ventilateur(v) => Some(v.boite),
            // La feuille du nénuphar : on s'y pose (elle s'enfonce ensuite).
            Entite::Nenuphar(n) => Some(n.boite),
            _ => None,
        }
    }
}

/// Rassemble les boîtes solides de toutes les entités, à passer à
/// `Environnement::solides`. On EMPRUNTE la tranche (`&[Entite]`) : l'appelant
/// garde ses entités, et la fonction renvoie un nouveau `Vec` qu'il possédera.
pub fn solides_des_entites(entites: &[Entite]) -> Vec<Aabb> {
    entites.iter().filter_map(Entite::solide).collect()
}

/// Rassemble les courants ascendants TEMPORAIRES des entités, actifs en ce
/// moment, à ajouter aux courants du niveau (`niveau.courants`, statiques) :
/// celui de chaque ventilateur qui SOUFFLE, et celui du Grand Nuage ESSOUFFLÉ.
/// Hors de ces phases, ils n'existent pas. Les colonnes de bulles (vers le haut)
/// et les chutes d'eau (vers le bas, accélération négative) de la carte 9.3, elles,
/// soufflent toujours.
pub fn courants_des_entites(entites: &[Entite]) -> Vec<CourantAscendant> {
    entites
        .iter()
        .filter_map(|e| match e {
            Entite::Ventilateur(v) => v.courant_actif(),
            Entite::Nuage(n) => n.courant_actif(),
            Entite::Bulles(b) => Some(b.courant()),
            Entite::Chute(c) => Some(c.courant()),
            _ => None,
        })
        .collect()
}

/// Les étendues d'eau du moment (carte 9.3) : celles du niveau (`statiques`),
/// plus l'eau de chaque marée qui n'est pas à basse mer, rangées du plus haut
/// au plus bas (l'ordre qu'attend `physics::eau_touchee`). Le jeu ne l'appelle
/// que dans un niveau qui a une marée : ailleurs, l'eau du niveau sert telle quelle.
pub fn eaux_des_entites(statiques: &[Eau], entites: &[Entite]) -> Vec<Eau> {
    let mut eaux: Vec<Eau> = entites
        .iter()
        .filter_map(|e| match e {
            Entite::Maree(m) => m.eau(),
            _ => None,
        })
        .chain(statiques.iter().copied())
        .collect();
    eaux.sort_by(|a, b| a.surface().total_cmp(&b.surface()));
    eaux
}

/// Le BOSS du niveau, vu sans savoir lequel (carte 9.12a) : le Grand Nuage (Monde 1)
/// ou la Pieuvre du port (Monde 2). Le jeu (points de vie du HUD, fin du niveau, mode
/// facile) et le robot de QA (son horloge dans la case) passent par ces méthodes :
/// un `match` à deux bras ici, plutôt qu'un dans chaque fichier qui s'en sert.
impl Entite {
    /// Points de vie du boss et son maximum, `(pv, pv_max)` ; `None` si ce n'est
    /// pas un boss.
    pub fn points_du_boss(&self) -> Option<(u8, u8)> {
        match self {
            Entite::Nuage(g) => Some((g.pv(), g.pv_max())),
            Entite::Pieuvre(p) => Some((p.pv(), p.pv_max())),
            _ => None,
        }
    }

    /// Le boss est-il vaincu (le niveau est fini) ?
    pub fn boss_vaincu(&self) -> bool {
        match self {
            Entite::Nuage(g) => g.est_vaincu(),
            Entite::Pieuvre(p) => p.est_vaincue(),
            _ => false,
        }
    }

    /// Mode facile du boss (carte 8.14) : deux points de vie au lieu de trois.
    pub fn passer_boss_en_facile(&mut self) {
        match self {
            Entite::Nuage(g) => g.passer_en_facile(),
            Entite::Pieuvre(p) => p.passer_en_facile(),
            _ => {}
        }
    }

    /// L'horloge du boss : `(rang de sa phase, pas écoulés dans la phase)`.
    pub fn horloge_du_boss(&self) -> Option<(u8, u16)> {
        match self {
            Entite::Nuage(g) => Some((g.phase() as u8, g.ecoule())),
            Entite::Pieuvre(p) => Some((p.phase() as u8, p.ecoule())),
            _ => None,
        }
    }

    /// Décale le boss de `pas` pas dans le temps (robot de QA) ; rien sinon.
    pub fn decaler_boss(&mut self, pas: i32) {
        match self {
            Entite::Nuage(g) => g.decaler(pas),
            Entite::Pieuvre(p) => p.decaler(pas),
            _ => {}
        }
    }
}

/// Pépin vient de mourir : chaque Grand Nuage reprend au début de son combat
/// (prélude), mais GARDE ses points de vie perdus (`GrandNuage::reprendre`) ; et
/// chaque plateforme friable se reforme tout de suite (`PlateformeFriable::reformer`,
/// carte 5.16), sauf une qui chevaucherait `occupant` (la boîte de Pépin, déjà
/// replacé à son point de reprise). À appeler par le jeu quand un `Contact::Mort`
/// est traité.
///
/// Pourquoi les friables (carte 5.16) : dans le bonus, sans lanterne, une chute
/// ramène toujours au départ ; sans cette règle, après une chute aux premiers
/// vides, le pont du départ était encore effondré (une dalle ne revient que 3 s
/// après sa chute) et l'on retombait aussitôt.
///
/// Les ennemis et les escargots (carte 9.29). AGA l'a demandé : « il faudrait
/// que lors d'une mort les monstres reprennent leur emplacement d'origine ».
/// Chaque ennemi VIVANT revient donc à sa place, à son sens et à son état de
/// départ (`Ennemi::revenir_a_l_origine`) : la scène que Pépin retrouve est
/// celle qu'il a déjà vue, il peut la rejouer à l'identique. Un ennemi ÉCRASÉ
/// reste écrasé : le chemin déjà gagné reste gagné.
pub fn reprendre_apres_une_mort(entites: &mut [Entite], occupant: &Aabb) {
    for e in entites {
        match e {
            Entite::Nuage(n) => n.reprendre(),
            Entite::Pieuvre(p) => p.reprendre(),
            Entite::Friable(f) => f.reformer(occupant),
            // Carte 9.3 : le nénuphar remonte, la stalactite est raccrochée.
            Entite::Nenuphar(n) => n.remettre(),
            Entite::Stalactite(s) => s.raccrocher(),
            // Carte 9.29 : les monstres reprennent leur place d'origine.
            Entite::Ennemi(en) => en.revenir_a_l_origine(),
            Entite::Escargot(es) => es.revenir_a_l_origine(),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::GrilleTuiles;

    #[test]
    fn solides_ne_garde_que_les_entites_solides() {
        let mut entites = vec![
            Entite::Goutte(GoutteDor::new(0, 0.0, 0.0)),
            Entite::Mobile(PlateformeMobile::new(
                Aabb::new(10.0, 10.0, 32.0, 8.0),
                (10.0, 10.0),
                (50.0, 10.0),
                20.0,
            )),
            Entite::Friable(PlateformeFriable::new(Aabb::new(60.0, 10.0, 16.0, 8.0))),
        ];
        assert_eq!(solides_des_entites(&entites).len(), 2);

        // Une plateforme friable absente n'est plus solide.
        if let Entite::Friable(f) = &mut entites[2] {
            f.toucher();
        }
        let grille = GrilleTuiles::vide(10, 10);
        let env = Environnement::simple(&grille);
        for _ in 0..friable::EFFRITEMENT_PAS {
            entites[2].actualiser(&env);
        }
        assert_eq!(solides_des_entites(&entites).len(), 1);
    }

    #[test]
    fn actualiser_delegue_a_la_variante() {
        let grille = GrilleTuiles::vide(10, 10);
        let env = Environnement::simple(&grille);
        let mut e = Entite::Eclair(Eclair::new(Aabb::new(0.0, 0.0, 16.0, 64.0), 100, 1));
        e.actualiser(&env);
        match &e {
            Entite::Eclair(x) => assert!(x.en_annonce(), "1 pas : la phase d'annonce commence"),
            _ => unreachable!(),
        }
        assert_eq!(e.boite().h, 64.0);
    }

    /// Carte 9.29 : après une mort, un corbeau déplacé et un pigeon en plein
    /// piqué reviennent exactement à leur place et à leur état de départ.
    #[test]
    fn apres_une_mort_les_ennemis_reprennent_leur_depart() {
        use crate::entities::ennemi::PhasePigeon;
        use crate::physics::{JOUEUR_H, JOUEUR_L};
        let grille = GrilleTuiles::vide(40, 20);
        let env = Environnement::simple(&grille);
        let corbeau = Ennemi::new(TypeEnnemi::Corbeau, 300.0, 50.0);
        let pigeon = Ennemi::pigeon(100.0, 54.0, true);
        let mut entites = vec![
            Entite::Ennemi(corbeau.clone()),
            Entite::Ennemi(pigeon.clone()),
        ];
        // Pépin passe sous le pigeon, devant lui : annonce, puis piqué.
        let proie = Aabb::new(120.0 - JOUEUR_L / 2.0, 104.0, JOUEUR_L, JOUEUR_H);
        if let Entite::Ennemi(p) = &mut entites[1] {
            p.guetter(&proie);
        }
        for _ in 0..u32::from(ennemi::ANNONCE_PIGEON_PAS) + 8 {
            for e in entites.iter_mut() {
                e.actualiser(&env);
            }
        }
        let (Entite::Ennemi(c), Entite::Ennemi(p)) = (&entites[0], &entites[1]) else {
            unreachable!()
        };
        assert_ne!(c.boite.x, corbeau.boite.x, "le corbeau a bougé");
        assert!(
            matches!(p.phase(), PhasePigeon::Plongee(_)),
            "{:?}",
            p.phase()
        );
        reprendre_apres_une_mort(&mut entites, &Aabb::new(0.0, 0.0, 12.0, 14.0));
        for (e, depart) in entites.iter().zip([&corbeau, &pigeon]) {
            let Entite::Ennemi(en) = e else {
                unreachable!()
            };
            // `Debug` montre TOUS les champs, privés compris : rien n'est oublié.
            assert_eq!(format!("{en:?}"), format!("{depart:?}"));
        }
    }

    /// Carte 9.29 : un ennemi écrasé le reste après une mort de Pépin.
    #[test]
    fn apres_une_mort_un_ennemi_ecrase_le_reste() {
        use crate::entities::ennemi::EtatEnnemi;
        let mut f = Ennemi::new(TypeEnnemi::Flaque, 64.0, 40.0);
        f.ecraser();
        let mut entites = vec![Entite::Ennemi(f)];
        reprendre_apres_une_mort(&mut entites, &Aabb::new(0.0, 0.0, 12.0, 14.0));
        let Entite::Ennemi(f) = &entites[0] else {
            unreachable!()
        };
        assert!(matches!(f.etat, EtatEnnemi::Ecrase(_)), "{:?}", f.etat);
    }
}
