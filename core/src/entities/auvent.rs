//! Auvent de boutique (lettre `A`) : une toile inclinée qui relance Pépin en
//! DIAGONALE, vers le haut et vers l'avant (vers la droite).
//!
//! Ce n'est pas un solide : comme le champignon, l'auvent agit au CONTACT, quand
//! Pépin tombe dessus par le haut (pieds à moins de `MARGE_AUVENT` du dessus de
//! la toile). Il le pose alors sur la toile et le relance (`Joueur::lancer_par_auvent`) :
//! - vers le haut à `VITESSE_AUVENT` (300 px/s : 50 px de haut, 40 pas de vol) ;
//! - vers la droite avec l'élan `ELAN_AUVENT` (192 px/s), qui s'éteint pendant le vol.
//!
//! Le vol ne dépend que de l'endroit où Pépin touche la toile. Sans rien
//! toucher, il retombe 4 tuiles plus loin, à la même hauteur : pile sur un
//! auvent espacé de 4 tuiles. Garder la gauche raccourcit le vol (3 tuiles),
//! garder la droite l'allonge (5 tuiles et plus). Six auvents à la suite
//! s'enchaînent ainsi sans toucher le sol (tests ci-dessous, sur un mini-niveau).
//!
//! Par-dessous ou par le côté, l'auvent ne fait rien : on passe à travers la
//! toile (c'est un décor de boutique, pas un mur).

use super::Actualisable;
use crate::aabb::Aabb;
use crate::physics::{Environnement, JOUEUR_H};
use crate::player::Joueur;

/// Profondeur (px) sous le dessus de la toile où les pieds de Pépin doivent
/// encore être pour que l'auvent le relance. À vitesse de chute maximale,
/// Pépin descend de 6 px par pas : 7 px suffisent pour ne jamais rater la
/// toile. Au-delà, il est arrivé par le côté ou par-dessous : rien ne se passe.
pub const MARGE_AUVENT: f32 = 7.0;
/// Pas pendant lesquels la toile reste « enfoncée » à l'écran après un rebond.
pub const ENFONCEMENT_PAS: u8 = 8;
/// Distance (px) entre le haut de la case `A` et le dessus de la zone de
/// contact : la toile dessinée descend de 2 px (à gauche) à 9 px (à droite) sous
/// le haut de la case ; le dessus de la zone est au milieu de cette pente.
pub const HAUT_TOILE: f32 = 4.0;

#[derive(Debug, Clone)]
pub struct Auvent {
    /// Zone de contact : 16 x 8 px, le dessus est la toile.
    pub boite: Aabb,
    /// Pas d'animation restants (0 = toile tendue).
    enfoncement: u8,
}

impl Auvent {
    /// Auvent dont la zone de contact (16 x 8 px) a son coin haut-gauche en `(x, y)`.
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            boite: Aabb::new(x, y, 16.0, 8.0),
            enfoncement: 0,
        }
    }

    /// Auvent de la case `A` dont le coin haut-gauche est en `(x, y)` : la zone
    /// commence `HAUT_TOILE` px sous le haut de la case (`level/ascii.rs`).
    pub fn dans_la_case(x: f32, y: f32) -> Self {
        Self::new(x, y + HAUT_TOILE)
    }

    /// Pépin tombe-t-il sur la toile par le haut en ce moment ?
    /// Il descend (ou est immobile), touche la zone, et ses pieds sont à moins
    /// de `MARGE_AUVENT` sous le dessus de la toile.
    pub fn recoit(&self, joueur: &Joueur) -> bool {
        joueur.vitesse.y >= 0.0
            && joueur.boite.chevauche(&self.boite)
            && joueur.boite.bas() - self.boite.y <= MARGE_AUVENT
    }

    /// Relance Pépin : il est posé sur la toile (pieds sur le dessus de la zone),
    /// puis lancé en diagonale (`Joueur::lancer_par_auvent`). La toile s'enfonce.
    pub fn lancer(&mut self, joueur: &mut Joueur) {
        joueur.boite.y = self.boite.y - JOUEUR_H;
        joueur.lancer_par_auvent();
        self.enfoncement = ENFONCEMENT_PAS;
    }

    /// La toile est-elle enfoncée en ce moment (pour le dessin) ?
    pub fn est_enfonce(&self) -> bool {
        self.enfoncement > 0
    }
}

impl Actualisable for Auvent {
    fn actualiser(&mut self, _env: &Environnement<'_>) {
        self.enfoncement = self.enfoncement.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::{GrilleTuiles, TAILLE_TUILE};
    use crate::entities::contacts::{resoudre_contacts, Contact};
    use crate::entities::Entite;
    use crate::physics::{
        Environnement, BAISSE_ELAN_AUVENT, ELAN_AUVENT, JOUEUR_L, VITESSE_AUVENT, VOL_AUVENT_PAS,
    };
    use crate::player::{Direction, Entrees};

    // -----------------------------------------------------------------------
    // Le mini-niveau de test : six auvents en rang, au-dessus du vide
    // -----------------------------------------------------------------------

    /// Ligne (tuiles) des auvents du mini-niveau.
    const LIGNE: usize = 6;
    /// Colonne du premier auvent.
    const COLONNE0: usize = 2;

    /// Mini-niveau de test : une grille VIDE de 60 x 12 tuiles (aucun sol : toucher
    /// le bas, c'est tomber) et six auvents sur la ligne 6, le premier en colonne 2,
    /// les suivants tous les `pas` tuiles. Chaque auvent est posé dans sa case comme
    /// par le lecteur de niveaux (`Auvent::dans_la_case`).
    fn six_auvents(pas: usize) -> (GrilleTuiles, Vec<Entite>) {
        let grille = GrilleTuiles::vide(60, 12);
        let entites = (0..6)
            .map(|k| {
                let tx = COLONNE0 + k * pas;
                Entite::Auvent(Auvent::dans_la_case(
                    tx as f32 * TAILLE_TUILE,
                    LIGNE as f32 * TAILLE_TUILE,
                ))
            })
            .collect();
        (grille, entites)
    }

    /// Ordonnée (px) du dessus des toiles du mini-niveau.
    fn toile() -> f32 {
        LIGNE as f32 * TAILLE_TUILE + HAUT_TOILE
    }

    /// Joue un pas de simulation complet (entités, joueur, contacts) ; renvoie
    /// l'indice de l'auvent qui vient de relancer Pépin, s'il y en a un.
    fn pas_de_jeu(
        joueur: &mut Joueur,
        entites: &mut [Entite],
        env: &Environnement<'_>,
        entrees: &Entrees,
    ) -> Option<usize> {
        for e in entites.iter_mut() {
            e.actualiser(env);
        }
        joueur.actualiser(entrees, env);
        let contacts = resoudre_contacts(joueur, entites);
        if !contacts.contains(&Contact::Auvent) {
            return None;
        }
        // Le relanceur : l'auvent sur la toile duquel Pépin vient d'être posé.
        let b = joueur.boite;
        entites.iter().position(|e| {
            matches!(e, Entite::Auvent(a)
                if a.boite.y == b.bas() && b.x < a.boite.droite() && a.boite.x < b.droite())
        })
    }

    /// Pépin lâché au-dessus du premier auvent, `decalage` px à droite de son bord
    /// gauche, vitesse nulle.
    fn pepin_au_dessus_du_premier(decalage: f32) -> Joueur {
        let x = COLONNE0 as f32 * TAILLE_TUILE + decalage;
        let y = toile() - JOUEUR_H - 20.0;
        Joueur::new(x, y)
    }

    // -----------------------------------------------------------------------
    // Trajectoire d'un rebond
    // -----------------------------------------------------------------------

    /// Un seul rebond, sans toucher à rien : Pépin part vers le haut et vers la
    /// droite, monte d'environ 50 px, et retombe à la hauteur de la toile 4 tuiles
    /// plus loin (64 px, à quelques pixels près), au bout de `VOL_AUVENT_PAS` pas.
    #[test]
    fn un_rebond_part_en_diagonale_et_retombe_quatre_tuiles_plus_loin() {
        let grille = GrilleTuiles::vide(30, 12);
        let env = Environnement::simple(&grille);
        let mut auvent = Auvent::new(64.0, 96.0);
        let mut j = Joueur::new(67.0, 96.0 - JOUEUR_H + 1.0);
        j.vitesse.y = 60.0; // il tombe sur la toile (pieds 1 px dedans)
        assert!(auvent.recoit(&j));
        auvent.lancer(&mut j);
        assert_eq!(j.boite.bas(), 96.0, "posé sur la toile");
        assert_eq!(j.vitesse.y, -VITESSE_AUVENT);
        assert_eq!(j.vitesse.x, 0.0, "la vitesse propre repart de 0");
        assert_eq!(j.elan_auvent(), ELAN_AUVENT);
        assert!(auvent.est_enfonce());

        let (x0, y0) = (j.boite.x, j.boite.y);
        let mut sommet = y0;
        let mut pas = 0;
        let aucune = Entrees::default();
        // Premier pas : en diagonale, vers le haut ET vers la droite.
        j.actualiser(&aucune, &env);
        pas += 1;
        assert!(j.boite.y < y0 && j.boite.x > x0, "diagonale haut-droite");
        while j.vitesse.y < 0.0 || j.boite.y < y0 {
            j.actualiser(&aucune, &env);
            pas += 1;
            sommet = sommet.min(j.boite.y);
            assert!(pas < 200);
        }
        let hauteur = y0 - sommet;
        assert!((45.0..=52.0).contains(&hauteur), "hauteur = {hauteur}");
        assert!((pas - VOL_AUVENT_PAS as i32).abs() <= 1, "{pas} pas de vol");
        let avance = j.boite.x - x0;
        assert!((60.0..=70.0).contains(&avance), "avance = {avance}");
        assert!(
            j.elan_auvent() <= BAISSE_ELAN_AUVENT + 1e-3,
            "l'élan est (presque) éteint à la fin du vol : {}",
            j.elan_auvent()
        );
    }

    /// L'élan s'éteint en ligne droite (`BAISSE_ELAN_AUVENT` par pas), et les
    /// commandes s'y ajoutent : garder la droite va bien plus loin, garder la
    /// gauche fait retomber Pépin presque sur place.
    #[test]
    fn les_commandes_allongent_ou_raccourcissent_le_vol() {
        let grille = GrilleTuiles::vide(40, 12);
        let env = Environnement::simple(&grille);
        let avance = |direction: Direction| {
            let mut j = Joueur::new(100.0, 100.0);
            j.lancer_par_auvent();
            let (x0, y0) = (j.boite.x, j.boite.y);
            let e = Entrees {
                direction,
                saut: false,
            };
            let mut elans = vec![j.elan_auvent()];
            loop {
                j.actualiser(&e, &env);
                elans.push(j.elan_auvent());
                if j.vitesse.y > 0.0 && j.boite.y >= y0 {
                    break;
                }
            }
            for w in elans.windows(2) {
                let baisse = w[0] - w[1];
                assert!(
                    (baisse - BAISSE_ELAN_AUVENT).abs() < 1e-3 || w[1] == 0.0,
                    "baisse {baisse}"
                );
            }
            j.boite.x - x0
        };
        let (rien, droite, gauche) = (
            avance(Direction::Immobile),
            avance(Direction::Droite),
            avance(Direction::Gauche),
        );
        assert!((60.0..=70.0).contains(&rien), "sans rien : {rien}");
        assert!(droite > 100.0, "droite : {droite}");
        assert!((0.0..30.0).contains(&gauche), "gauche : {gauche}");
    }

    /// Par-dessous (en montant) ou par le côté (pieds bien plus bas que la toile),
    /// l'auvent ne relance pas. Il ne bloque rien non plus : ce n'est pas un solide.
    #[test]
    fn l_auvent_ne_relance_que_par_le_dessus() {
        let auvent = Auvent::new(64.0, 96.0);
        // En montant à travers la toile.
        let mut j = Joueur::new(67.0, 92.0);
        j.vitesse.y = -100.0;
        assert!(!auvent.recoit(&j));
        // Par le côté : il descend, mais ses pieds sont 10 px sous la toile.
        let mut j = Joueur::new(56.0, 96.0 + 10.0 - JOUEUR_H);
        j.vitesse.y = 30.0;
        assert!(j.boite.chevauche(&auvent.boite));
        assert!(!auvent.recoit(&j));
        // Par le dessus, pieds 1 px dans la toile : oui.
        let mut j = Joueur::new(67.0, 96.0 + 1.0 - JOUEUR_H);
        j.vitesse.y = 30.0;
        assert!(auvent.recoit(&j));
        // Pas un solide.
        assert_eq!(Entite::Auvent(auvent).solide(), None);
    }

    /// L'élan tombe à 0 contre un mur, et quand Pépin se pose (ici sur un toit
    /// plus haut que la toile, avant la fin du vol : l'élan n'était pas fini).
    #[test]
    fn l_elan_s_arrete_contre_un_mur_ou_au_sol() {
        // Mur en colonne 8, du haut en bas.
        let lignes = vec!["........#..."; 12];
        let grille = GrilleTuiles::depuis_texte(&lignes);
        let env = Environnement::simple(&grille);
        let mut j = Joueur::new(8.0 * TAILLE_TUILE - JOUEUR_L - 2.0, 100.0);
        j.lancer_par_auvent();
        j.actualiser(&Entrees::default(), &env);
        assert_eq!(j.elan_auvent(), 0.0, "le mur arrête l'élan");

        // Toit des colonnes 4 à 7 sur la ligne 8 (dessus : y = 128) ; Pépin est
        // lancé de plus bas (pieds en y = 164), à gauche du toit.
        let mut lignes = vec!["............"; 12];
        lignes[8] = "....####....";
        let grille = GrilleTuiles::depuis_texte(&lignes);
        let env = Environnement::simple(&grille);
        let mut j = Joueur::new(16.0, 164.0 - JOUEUR_H);
        j.lancer_par_auvent();
        let mut pas = 0;
        while !j.au_sol {
            j.actualiser(&Entrees::default(), &env);
            pas += 1;
            assert!(pas < 200);
        }
        assert_eq!(j.boite.bas(), 128.0, "posé sur le toit");
        assert!(pas < VOL_AUVENT_PAS as usize, "posé en {pas} pas");
        assert_eq!(j.elan_auvent(), 0.0, "posé : plus d'élan");
    }

    // -----------------------------------------------------------------------
    // Enchaînement : six auvents sans toucher le sol
    // -----------------------------------------------------------------------

    /// Un joueur simple qui vise : pendant chaque vol, il garde la droite si le
    /// point de chute (prévu en supposant qu'il lâche tout) est avant le centre de
    /// l'auvent suivant, la gauche s'il est après. Renvoie le nombre d'auvents qui
    /// l'ont relancé, et s'il est tombé.
    fn enchainer(pas: usize, decalage: f32, viser: bool) -> (usize, Vec<f32>) {
        let (grille, mut entites) = six_auvents(pas);
        let env = Environnement::simple(&grille);
        let mut j = pepin_au_dessus_du_premier(decalage);
        let mut relances: Vec<usize> = Vec::new();
        let mut decalages: Vec<f32> = Vec::new();
        let bas = grille_bas(&grille);
        for _ in 0..2000 {
            let suivant = relances.last().map_or(0, |&i| i + 1);
            let direction = if !viser || relances.is_empty() || suivant >= 6 {
                Direction::Immobile
            } else {
                let cible = (COLONNE0 + suivant * pas) as f32 * TAILLE_TUILE + 8.0;
                let chute = point_de_chute(&j);
                if chute < cible - 2.0 {
                    Direction::Droite
                } else if chute > cible + 2.0 {
                    Direction::Gauche
                } else {
                    Direction::Immobile
                }
            };
            let e = Entrees {
                direction,
                saut: false,
            };
            if let Some(i) = pas_de_jeu(&mut j, &mut entites, &env, &e) {
                decalages.push(j.boite.x - (COLONNE0 + i * pas) as f32 * TAILLE_TUILE);
                relances.push(i);
                if relances.len() == 6 {
                    break;
                }
            }
            if j.boite.y > bas {
                break; // tombé dans le vide
            }
        }
        // Les auvents doivent s'être succédé dans l'ordre 0, 1, 2... sans doublon.
        let dans_l_ordre = relances.iter().enumerate().all(|(k, &i)| k == i);
        assert!(dans_l_ordre, "pas {pas} : ordre des rebonds {relances:?}");
        (relances.len(), decalages)
    }

    /// Bas de la grille (px).
    fn grille_bas(g: &GrilleTuiles) -> f32 {
        g.hauteur() as f32 * TAILLE_TUILE
    }

    /// Centre de Pépin quand il repassera à la hauteur de la toile, s'il lâche
    /// tout dès maintenant (simulation sur une copie, sans entité).
    fn point_de_chute(j: &Joueur) -> f32 {
        let grille = GrilleTuiles::vide(60, 12);
        let env = Environnement::simple(&grille);
        let mut c = j.clone();
        let toile = toile();
        for _ in 0..200 {
            c.actualiser(&Entrees::default(), &env);
            if c.vitesse.y > 0.0 && c.boite.bas() >= toile {
                break;
            }
        }
        c.boite.x + JOUEUR_L / 2.0
    }

    /// Le cœur de la carte : six auvents espacés de 3, 4 ou 5 tuiles (et même 6)
    /// s'enchaînent SANS TOUCHER LE SOL, d'où que Pépin tombe sur le premier.
    #[test]
    fn six_auvents_espaces_de_3_a_5_tuiles_s_enchainent() {
        for pas in [3, 4, 5, 6] {
            for decalage in [-6.0, 0.0, 3.0, 8.0, 12.0] {
                let (n, decalages) = enchainer(pas, decalage, true);
                assert_eq!(
                    n, 6,
                    "pas de {pas} tuiles, départ {decalage:+} px : {n} rebonds ({decalages:?})"
                );
            }
        }
    }

    /// Espacés de 4 tuiles, il n'y a même rien à faire : chaque vol retombe pile
    /// sur l'auvent suivant, et toujours au même endroit de la toile (le vol ne
    /// dépend que du point de contact, qui se répète).
    #[test]
    fn espaces_de_4_tuiles_ils_s_enchainent_sans_rien_toucher() {
        let (n, decalages) = enchainer(4, 3.0, false);
        assert_eq!(n, 6, "{decalages:?}");
        let ecart = decalages[1..]
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0f32, f32::max);
        assert!(ecart < 3.0, "le point de contact dérive : {decalages:?}");
    }

    /// Sans viser, espacés de 3 ou de 5 tuiles, Pépin finit par rater un auvent :
    /// l'enchaînement demande bien de doser la direction (ce n'est pas automatique).
    #[test]
    fn espaces_de_3_ou_5_tuiles_il_faut_doser_la_direction() {
        for pas in [3, 5] {
            let (n, _) = enchainer(pas, 3.0, false);
            assert!(n < 6, "pas de {pas} : {n} rebonds sans rien toucher");
        }
    }
}
