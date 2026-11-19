//! COURSE CONTRE LE ROBOT (carte 8.7) : on choisit un niveau et une force (DÉBUTANT,
//! HABITUÉ, EXPERT : les trois profils de joueur simulé de la mesure de difficulté),
//! et un FANTÔME de Pépin court le même niveau en même temps que le joueur.
//!
//! Le fantôme vit dans SON PROPRE `Monde` (une copie du niveau, toute neuve) : il n'a
//! aucune collision avec Pépin et aucun effet sur le monde du joueur (il ne ramasse
//! rien, n'écrase rien, ne fait aucun bruit). Seul le rendu le montre, tramé en damier
//! (`scene`), avec l'étiquette de sa force au-dessus et une flèche au bord de l'écran
//! quand il est hors champ.
//!
//! Le joueur simulé (`difficulte.rs`) prend bien trop longtemps à calculer pendant le
//! jeu : comme pour la démo du titre (`crate::demo`), on EMBARQUE ses parties
//! (`parties.rs`, fichier GÉNÉRÉ par le test ignoré `ecrire_parties_des_fantomes`) :
//! pour chaque niveau et chaque force, les entrées de chaque pas, compressées en
//! plages, d'une partie jouée avec la graine officielle de la mesure. Le fantôme peut
//! donc hésiter et mourir comme un vrai débutant ; mort, il réapparaît à sa lanterne
//! (son `Monde` s'en charge, comme pour Pépin).
//!
//! Si un niveau change sans que les parties soient régénérées, le rejeu DIVERGE : le
//! fantôme meurt plus souvent que prévu, finit trop tôt, ou n'a plus d'entrées avant
//! le drapeau. Il s'arrête alors proprement : « LE ROBOT A ABANDONNÉ », jamais de
//! plantage (le test `chaque_partie_des_fantomes_rejoue_sans_divergence` le signale
//! avant).
//!
//! Notions Rust illustrées ici : réutilisation d'un itérateur (`Rejeu` de la démo),
//! `enum` d'état avec donnée (`EtatFantome::Arrive(u32)`), `Option<&'static T>`.

mod parties;

use pepin_core::physics::HZ;
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use crate::chrono;
use crate::contenu;
use crate::defis;
use crate::demo::{indice_du_niveau, Rejeu};
use crate::monde::Monde;
use crate::sauvegarde::Sauvegarde;

pub use parties::PARTIES;

/// Force du fantôme : un des trois profils du joueur simulé (`difficulte::PROFILS`,
/// dans le même ordre).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Force {
    Debutant,
    Habitue,
    Expert,
}

impl Force {
    /// Les trois forces, de la plus faible à la plus forte.
    pub const TOUTES: [Force; 3] = [Force::Debutant, Force::Habitue, Force::Expert];

    /// Rang de la force (0 à 2) : dans `PROFILS`, dans la sauvegarde et dans `PARTIES`.
    pub fn rang(self) -> usize {
        match self {
            Force::Debutant => 0,
            Force::Habitue => 1,
            Force::Expert => 2,
        }
    }

    /// Libellé affiché (menu, étiquette du fantôme), en capitales.
    pub fn libelle(self) -> &'static str {
        contenu::FORCES[self.rang()]
    }
}

/// La partie embarquée d'un fantôme, désignée par le NOM de son niveau (comme la démo :
/// le rang d'un niveau dans `NIVEAUX` peut changer).
pub struct PartieFantome {
    pub nom: &'static str,
    /// Rang de la force (`Force::rang`).
    pub force: u8,
    /// Durée prévue : le fantôme touche le drapeau à ce pas (le dernier des plages).
    pub pas: u32,
    /// Morts prévues pendant la partie (au-delà : le rejeu a divergé).
    pub morts: u16,
    /// Plages `(nombre de pas, code des entrées)`, codées comme la démo
    /// (`demo::entrees`).
    pub plages: &'static [(u16, u8)],
}

/// La partie embarquée du niveau `indice` pour cette force (`None` : niveau sans
/// partie, par exemple un niveau ajouté depuis la génération).
pub fn partie(indice: usize, force: Force) -> Option<&'static PartieFantome> {
    PARTIES
        .iter()
        .find(|p| usize::from(p.force) == force.rang() && indice_du_niveau(p.nom) == Some(indice))
}

/// Le niveau `indice` a-t-il au moins une partie de fantôme (bouton COURSE actif) ?
pub fn disponible(indice: usize) -> bool {
    Force::TOUTES.iter().any(|&f| partie(indice, f).is_some())
}

/// Où en est le fantôme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtatFantome {
    /// Il court.
    EnCourse,
    /// Il a touché le drapeau, au pas donné.
    Arrive(u32),
    /// Le rejeu a divergé (ou aucune partie n'existe) : il s'est arrêté.
    Abandon,
}

/// Le fantôme d'une course : son monde à lui et le rejeu de sa partie.
pub struct Fantome {
    pub force: Force,
    pub monde: Monde,
    pub etat: EtatFantome,
    partie: Option<&'static PartieFantome>,
    rejeu: Rejeu,
}

impl Fantome {
    /// Le fantôme de force `force` au départ du niveau `indice`.
    pub fn new(indice: usize, force: Force) -> Self {
        let partie = partie(indice, force);
        Self {
            force,
            monde: Monde::new(indice),
            etat: if partie.is_some() {
                EtatFantome::EnCourse
            } else {
                EtatFantome::Abandon
            },
            partie,
            rejeu: Rejeu::new(partie.map_or(&[], |p| p.plages)),
        }
    }

    /// Temps prévu du fantôme (pas), tant qu'il n'a pas abandonné : celui de sa partie
    /// embarquée (le même que son arrivée réelle, puisqu'il ne diverge pas).
    pub fn temps(&self) -> Option<u32> {
        match self.etat {
            EtatFantome::Arrive(pas) => Some(pas),
            EtatFantome::EnCourse => self.partie.map(|p| p.pas),
            EtatFantome::Abandon => None,
        }
    }

    /// Un pas du fantôme (rien s'il est arrivé ou a abandonné).
    pub fn avancer(&mut self) {
        if self.etat != EtatFantome::EnCourse {
            return;
        }
        let Some(p) = self.partie else {
            self.etat = EtatFantome::Abandon;
            return;
        };
        let Some(entrees) = self.rejeu.next() else {
            // Plus d'entrées avant le drapeau : divergence.
            self.etat = EtatFantome::Abandon;
            return;
        };
        self.monde.avancer(&entrees);
        self.etat = verdict(&self.monde, p);
    }

    /// Le fantôme est-il à dessiner ? Seulement pendant qu'il court (arrivé, il est
    /// rentré ; abandonné, il a disparu).
    pub fn visible(&self) -> bool {
        self.etat == EtatFantome::EnCourse
    }
}

/// L'état du fantôme après un pas de son monde `m`, comparé à sa partie `p` : drapeau
/// au bon pas (arrivé), trop tôt ou trop de morts (divergence : abandon), sinon il
/// court toujours.
fn verdict(m: &Monde, p: &PartieFantome) -> EtatFantome {
    if m.termine {
        if m.pas == p.pas {
            EtatFantome::Arrive(m.pas)
        } else {
            EtatFantome::Abandon
        }
    } else if m.etat.morts > u32::from(p.morts) || m.pas >= p.pas {
        EtatFantome::Abandon
    } else {
        EtatFantome::EnCourse
    }
}

/// Issue d'une course, au drapeau du joueur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Issue {
    /// Le joueur a fini avant le fantôme. `ecart` : pas d'avance (> 0).
    Gagne { ecart: u32 },
    /// Le fantôme a fini avant (ou en même temps). `ecart` : pas de retard (>= 0).
    Perdu { ecart: u32 },
    /// Le fantôme a abandonné : la course ne compte pas.
    Abandon,
}

impl Issue {
    /// Issue d'une course finie en `temps` pas contre un fantôme qui finit (ou finira)
    /// en `fantome` pas (`None` : abandon).
    pub fn de(temps: u32, fantome: Option<u32>) -> Self {
        match fantome {
            None => Issue::Abandon,
            Some(f) if temps < f => Issue::Gagne { ecart: f - temps },
            Some(f) => Issue::Perdu { ecart: temps - f },
        }
    }

    /// L'écart signé (pas) gardé dans la sauvegarde : négatif = victoire.
    pub fn ecart_signe(self) -> Option<i32> {
        match self {
            Issue::Gagne { ecart } => Some(-(ecart as i32)),
            Issue::Perdu { ecart } => Some(ecart as i32),
            Issue::Abandon => None,
        }
    }
}

/// Ce que montre le bilan d'une course.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResultatCourse {
    pub force: Force,
    pub issue: Issue,
    /// Meilleur résultat d'avant contre cette force (écart signé, voir la sauvegarde).
    pub ancien: Option<i32>,
    /// Prime créditée (première victoire contre EXPERT sur ce niveau), 0 sinon.
    pub prime: u32,
}

/// Note une course finie dans la sauvegarde (sans l'enregistrer) : meilleur résultat
/// par niveau et par force, et la prime de la première victoire contre EXPERT.
pub fn noter_course(
    s: &mut Sauvegarde,
    cle: &str,
    force: Force,
    temps: u32,
    fantome: Option<u32>,
) -> ResultatCourse {
    let issue = Issue::de(temps, fantome);
    let ancien = s.meilleure_course(cle, force.rang());
    if let Some(e) = issue.ecart_signe() {
        s.noter_course(cle, force.rang(), e);
    }
    let prime = if force == Force::Expert && matches!(issue, Issue::Gagne { .. }) {
        defis::crediter_course_expert(s, cle)
    } else {
        0
    };
    ResultatCourse {
        force,
        issue,
        ancien,
        prime,
    }
}

/// « GAGNÉ DE 2,3 s », « PERDU DE 0,4 s » (pour un écart signé de la sauvegarde).
pub fn texte_resultat(ecart: i32) -> String {
    let d = ecart.unsigned_abs() * 10 / HZ;
    let mot = if ecart < 0 {
        contenu::GAGNE
    } else {
        contenu::PERDU
    };
    format!("{mot} DE {},{} s", d / 10, d % 10)
}

/// Ligne d'état du fantôme sous le chrono : « ROBOT EXPERT », « ROBOT ARRIVÉ
/// 0:41,6 », ou « LE ROBOT A ABANDONNÉ ».
pub fn ligne_hud(f: &Fantome) -> String {
    match f.etat {
        EtatFantome::EnCourse => format!("{} {}", contenu::ROBOT, f.force.libelle()),
        EtatFantome::Arrive(pas) => {
            format!("{} {}", contenu::ROBOT_ARRIVE, chrono::temps_precis(pas))
        }
        EtatFantome::Abandon => contenu::ROBOT_ABANDON.to_string(),
    }
}

/// Bord de l'écran où pointe la flèche d'un fantôme hors champ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bord {
    Gauche,
    Droite,
    Haut,
    Bas,
}

/// Marge (px) entre la flèche et le bord de l'écran.
const MARGE_FLECHE: f32 = 2.0;
/// Bandeau du HUD (px) : la flèche du haut se pose dessous.
const HAUT_FLECHE: f32 = 26.0;

/// Où dessiner la flèche d'un fantôme hors champ : `(x, y)` sont les coordonnées
/// ÉCRAN de sa boîte (`w` x `h`). `None` tant qu'il est (même en partie) à l'écran.
/// La flèche se pose sur le bord le plus proche de lui, bornée dans l'écran (à gauche
/// ou à droite d'abord : c'est le cas courant, il court devant ou derrière). Pure.
pub fn fleche_hors_champ(x: f32, y: f32, w: f32, h: f32) -> Option<(f32, f32, Bord)> {
    let (l, hl) = (LARGEUR_LOGIQUE as f32, HAUTEUR_LOGIQUE as f32);
    let (cx, cy) = (x + w / 2.0, y + h / 2.0);
    let borne_y = cy.clamp(HAUT_FLECHE, hl - 8.0 - MARGE_FLECHE);
    let borne_x = cx.clamp(8.0 + MARGE_FLECHE, l - 8.0 - MARGE_FLECHE);
    if x + w <= 0.0 {
        Some((MARGE_FLECHE, borne_y, Bord::Gauche))
    } else if x >= l {
        Some((l - MARGE_FLECHE, borne_y, Bord::Droite))
    } else if y + h <= 0.0 {
        Some((borne_x, HAUT_FLECHE, Bord::Haut))
    } else if y >= hl {
        Some((borne_x, hl - MARGE_FLECHE, Bord::Bas))
    } else {
        None
    }
}

/// Les entrées d'une partie embarquée, un pas après l'autre (test).
#[cfg(test)]
fn entrees_de(p: &PartieFantome) -> Vec<pepin_core::player::Entrees> {
    Rejeu::new(p.plages).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demo::compresser;
    use pepin_core::physics::HZ;
    use pepin_core::player::Entrees;
    // La course contre le robot est celle du MONDE 1 seulement (carte 9.6).
    use pepin_core::levels::NIVEAUX_MONDE_1 as NIVEAUX;

    /// Chaque partie embarquée, rejouée dans la vraie simulation comme le fait le jeu,
    /// touche le drapeau au pas prévu, avec les morts prévues et sans jamais être
    /// déclarée abandonnée. Sinon : régénérer `parties.rs` (test d'en bas). Un niveau
    /// sans partie (le joueur simulé n'y trouve pas le drapeau) n'a pas de COURSE.
    #[test]
    fn chaque_partie_des_fantomes_rejoue_sans_divergence() {
        assert!(!PARTIES.is_empty());
        for p in PARTIES {
            let indice = indice_du_niveau(p.nom)
                .unwrap_or_else(|| panic!("niveau « {} » introuvable", p.nom));
            let force = Force::TOUTES[usize::from(p.force)];
            let pareilles = PARTIES
                .iter()
                .filter(|q| q.nom == p.nom && q.force == p.force);
            assert_eq!(
                pareilles.count(),
                1,
                "« {} » {force:?} : deux parties",
                p.nom
            );
            assert_eq!(
                compresser(&entrees_de(p)),
                p.plages,
                "« {} » : plages",
                p.nom
            );
            let mut f = Fantome::new(indice, force);
            while f.etat == EtatFantome::EnCourse {
                f.avancer();
            }
            assert_eq!(
                f.etat,
                EtatFantome::Arrive(p.pas),
                "« {} » {force:?} : divergence (pas {}, morts {})",
                p.nom,
                f.monde.pas,
                f.monde.etat.morts
            );
            assert_eq!(
                f.monde.etat.morts,
                u32::from(p.morts),
                "« {} » morts",
                p.nom
            );
        }
    }

    /// Une partie trouvée par le générateur : force, numéro de l'essai, morts, entrées.
    type PartieTrouvee = (Force, u32, u32, Vec<Entrees>);

    /// Régénère `src/course/parties.rs` avec le joueur simulé de la mesure de difficulté
    /// (graine officielle), à relancer si un niveau change (un fil d'exécution par
    /// niveau ; plusieurs minutes) :
    /// `./docker-cargo.sh test -p game --no-default-features -- --ignored --exact course::tests::ecrire_parties_des_fantomes`
    #[test]
    #[ignore]
    fn ecrire_parties_des_fantomes() {
        use crate::difficulte::{partie_du_fantome, Itineraires};
        let mut source = String::from(
            "//! FICHIER GÉNÉRÉ par le test ignoré `ecrire_parties_des_fantomes`\n\
             //! (`course/mod.rs`) : ne pas le modifier à la main.\n\
             //!\n\
             //! Parties des fantômes (joueur simulé de la mesure, graine officielle), par\n\
             //! niveau et par force, compressées en plages `(nombre de pas, code)`.\n\n\
             use super::PartieFantome;\n\n\
             #[rustfmt::skip]\n\
             pub const PARTIES: &[PartieFantome] = &[\n",
        );
        // Un fil par niveau (`thread::scope`) ; les résultats gardent l'ordre de la table.
        let parties: Vec<Vec<PartieTrouvee>> = std::thread::scope(|portee| {
            let fils: Vec<_> = (0..NIVEAUX.len())
                .map(|indice| {
                    portee.spawn(move || {
                        let Some(itin) = Itineraires::tenter(indice) else {
                            return Vec::new();
                        };
                        Force::TOUTES
                            .iter()
                            .filter_map(|&force| {
                                partie_du_fantome(&itin, force.rang()).map(
                                    |(essai, bilan, journal)| (force, essai, bilan.morts, journal),
                                )
                            })
                            .collect()
                    })
                })
                .collect();
            fils.into_iter().map(|f| f.join().unwrap()).collect()
        });
        let mut total = 0;
        for (n, liste) in NIVEAUX.iter().zip(&parties) {
            if liste.is_empty() {
                println!(
                    "{} : AUCUNE partie (drapeau hors d'atteinte du joueur simulé)",
                    n.nom
                );
            }
            for (force, essai, morts, journal) in liste {
                let plages = compresser(journal);
                total += plages.len();
                println!(
                    "{} / {force:?} : essai {essai}, {} pas ({:.1} s), {} morts, {} plages",
                    n.nom,
                    journal.len(),
                    journal.len() as f32 / HZ as f32,
                    morts,
                    plages.len()
                );
                source.push_str(&format!(
                    "    PartieFantome {{\n        nom: {:?},\n        force: {},\n        \
                     pas: {},\n        morts: {},\n        plages: &[",
                    n.nom,
                    force.rang(),
                    journal.len(),
                    morts
                ));
                for (k, (nb, c)) in plages.iter().enumerate() {
                    if k % 10 == 0 {
                        source.push_str("\n            ");
                    } else {
                        source.push(' ');
                    }
                    source.push_str(&format!("({nb}, {c}),"));
                }
                source.push_str("\n        ],\n    },\n");
            }
        }
        source.push_str("];\n");
        println!("{total} plages en tout");
        std::fs::write("src/course/parties.rs", source).unwrap();
    }
}
