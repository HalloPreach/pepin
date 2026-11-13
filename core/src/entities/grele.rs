//! Grêle : une colonne de grêlons qui tombe EN CONTINU (ni pause ni annonce,
//! contrairement à l'éclair). Elle tue Pépin parapluie FERMÉ ; parapluie OUVERT
//! (il plane), les grêlons rebondissent sur la toile et il traverse la colonne
//! sans danger. Aucune touche nouvelle : on se sert du parapluie qu'on connaît.
//!
//! « Parapluie fermé », c'est tout ce qui n'est pas planer : marcher ou attendre
//! au sol, monter pendant un saut, tomber sans tenir SAUT (voir
//! `Joueur::actualiser`, étape 3 : on ne plane qu'en l'air, SAUT maintenu, en
//! descendant). On traverse donc une colonne de grêle en planant DESSOUS.
//!
//! Notions Rust : une structure qui ne change jamais (la colonne est fixe ; le
//! mouvement des grêlons n'est qu'un dessin, calculé au rendu à partir du numéro
//! du pas) ; une règle de jeu écrite comme une fonction pure (`blesse`) qui ne
//! fait qu'EMPRUNTER le joueur (`&Joueur`), donc testable seule.

use super::Actualisable;
use crate::aabb::Aabb;
use crate::physics::Environnement;
use crate::player::Joueur;

#[derive(Debug, Clone)]
pub struct Grele {
    /// Colonne où tombent les grêlons : de la case `g` jusqu'au premier solide
    /// en dessous (le bas de la carte s'il n'y en a pas).
    pub zone: Aabb,
}

impl Grele {
    pub fn new(zone: Aabb) -> Self {
        Self { zone }
    }

    /// Les grêlons blessent-ils ce joueur en ce moment ? Oui s'il touche la
    /// colonne parapluie fermé ; non s'il plane (le parapluie ouvert le couvre).
    pub fn blesse(&self, joueur: &Joueur) -> bool {
        !joueur.plane && joueur.boite.chevauche(&self.zone)
    }
}

/// La colonne ne bouge pas : corps vide, comme les pointes.
impl Actualisable for Grele {
    fn actualiser(&mut self, _env: &Environnement<'_>) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::{JOUEUR_H, JOUEUR_L};

    fn colonne() -> Grele {
        // Colonne de 12 px de large, de y = 0 à y = 128.
        Grele::new(Aabb::new(162.0, 0.0, 12.0, 128.0))
    }

    #[test]
    fn parapluie_ferme_dans_la_colonne_blesse() {
        let g = colonne();
        let mut j = Joueur::new(160.0, 60.0);
        j.plane = false;
        assert!(g.blesse(&j));
    }

    #[test]
    fn parapluie_ouvert_dans_la_colonne_protege() {
        let g = colonne();
        let mut j = Joueur::new(160.0, 60.0);
        j.plane = true;
        assert!(!g.blesse(&j));
    }

    /// Limites de la colonne : juste à côté (à gauche, à droite) ou juste dessous,
    /// rien ; un pixel de chevauchement suffit à blesser.
    #[test]
    fn limites_de_la_colonne() {
        let g = colonne();
        let z = g.zone;
        let ferme = |x: f32, y: f32| {
            let mut j = Joueur::new(x, y);
            j.plane = false;
            j
        };
        // Bords gauche et droit : collé sans chevaucher, puis 1 px dedans.
        assert!(!g.blesse(&ferme(z.x - JOUEUR_L, 60.0)));
        assert!(g.blesse(&ferme(z.x - JOUEUR_L + 1.0, 60.0)));
        assert!(!g.blesse(&ferme(z.droite(), 60.0)));
        assert!(g.blesse(&ferme(z.droite() - 1.0, 60.0)));
        // Dessous (sous le solide qui arrête la colonne) : rien.
        assert!(!g.blesse(&ferme(z.x, z.bas())));
        assert!(g.blesse(&ferme(z.x, z.bas() - JOUEUR_H + 1.0)));
    }

    // --- Mini-niveau de test : la vraie physique, pas à pas -----------------

    use crate::entities::contacts::{resoudre_contacts, CauseMort, Contact};
    use crate::entities::{solides_des_entites, Entite};
    use crate::level::Niveau;
    use crate::player::{Direction, Entrees};

    /// Un toit plat, une colonne de grêle (colonne 8, de la ligne 2 au sol) entre
    /// le départ et le drapeau.
    const MINI: &str = "
.o.o.o..............
....................
........g...........
....................
....................
.@................X.
####################
";

    /// Ce que le mini-niveau a donné.
    struct Issue {
        mort: Option<CauseMort>,
        /// Pépin était-il en l'air au moment de sa mort ?
        mort_en_l_air: bool,
        arrivee: bool,
        /// Pépin a-t-il été vu DANS la colonne en planant ?
        plane_dedans: bool,
    }

    /// Joue `pilote` (entrées selon le joueur) jusqu'au drapeau, à une mort ou à
    /// 600 pas.
    fn jouer(pilote: impl Fn(&Joueur) -> Entrees) -> Issue {
        let niv = Niveau::depuis_ascii(MINI).unwrap();
        let mut entites = niv.entites.clone();
        let zone = entites
            .iter()
            .find_map(|e| match e {
                Entite::Grele(g) => Some(g.zone),
                _ => None,
            })
            .unwrap();
        let mut j = Joueur::new(niv.depart.0, niv.depart.1);
        let mut issue = Issue {
            mort: None,
            mort_en_l_air: false,
            arrivee: false,
            plane_dedans: false,
        };
        for _ in 0..600 {
            let solides = solides_des_entites(&entites);
            let env = Environnement {
                grille: &niv,
                solides: &solides,
                courants: &niv.courants,
                vents: &niv.vents,
            };
            let entrees = pilote(&j);
            j.actualiser(&entrees, &env);
            issue.plane_dedans |= j.plane && j.boite.chevauche(&zone);
            for c in resoudre_contacts(&mut j, &mut entites) {
                if let Contact::Mort(cause) = c {
                    issue.mort = Some(cause);
                    issue.mort_en_l_air = !j.au_sol;
                    return issue;
                }
            }
            if j.boite.chevauche(&niv.arrivee) {
                issue.arrivee = true;
                return issue;
            }
        }
        issue
    }

    /// Parapluie fermé, Pépin marche droit dans la colonne : la grêle le tue.
    #[test]
    fn mini_niveau_marcher_dans_la_grele_tue() {
        let issue = jouer(|_| Entrees {
            direction: Direction::Droite,
            saut: false,
        });
        assert_eq!(issue.mort, Some(CauseMort::Grele));
        assert!(!issue.mort_en_l_air, "tué au sol");
        assert!(!issue.arrivee);
    }

    /// Pépin prend son élan, saute bien avant la colonne et garde SAUT enfoncé :
    /// au sommet du saut le parapluie s'ouvre, il traverse la colonne en planant
    /// et atteint le drapeau sans mourir.
    #[test]
    fn mini_niveau_planer_sous_la_grele_protege() {
        let issue = jouer(|j| Entrees {
            direction: Direction::Droite,
            saut: j.boite.x >= 60.0 && j.boite.x < 200.0,
        });
        assert_eq!(issue.mort, None);
        assert!(issue.arrivee, "le drapeau est atteint");
        assert!(issue.plane_dedans, "Pépin a traversé la colonne en planant");
    }

    /// Le même saut, mais pris trop tard (juste avant la colonne) : Pépin entre
    /// dans la grêle en MONTANT, et on ne plane jamais en montant (le parapluie ne
    /// s'ouvre qu'en descendant). La grêle le tue en l'air, SAUT pourtant enfoncé.
    #[test]
    fn mini_niveau_sauter_dans_la_grele_tue() {
        let issue = jouer(|j| Entrees {
            direction: Direction::Droite,
            saut: j.boite.x >= 104.0,
        });
        assert_eq!(issue.mort, Some(CauseMort::Grele));
        assert!(issue.mort_en_l_air, "tué pendant la montée");
        assert!(!issue.plane_dedans);
    }
}
