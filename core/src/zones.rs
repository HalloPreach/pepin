//! Module `zones` : les cinq niveaux de PÉPIN et leurs métadonnées.
//!
//! Chaque niveau est une « zone » du monde : un thème de décor (le ciel
//! s'éclaircit d'un niveau à l'autre, de l'orage au soleil), un nom, un court
//! texte d'introduction et une idée de mécanique. Le dessin ASCII lui-même
//! vit dans `levels/` (un petit fichier par niveau).
//!
//! Notions Rust illustrées ici (à reprendre dans le README) :
//! - `const` et `&'static str` : des données connues à la compilation, qui
//!   vivent pendant tout le programme (donc pas d'ownership à gérer) ;
//! - `enum` sans données pour le thème ; tableau de taille fixe `[T; 5]` ;
//! - `Result` : charger un niveau peut échouer, l'appelant décide quoi faire.

use crate::level::{ErreurNiveau, Niveau};
use crate::levels::{gouttieres, nuages, orage, toits, vents};

/// Ambiance visuelle d'un niveau. Le rendu choisit ciel, parallaxe et pluie
/// selon cette valeur ; `luminosite` donne l'avancement de l'orage vers le soleil.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    /// Nuit d'orage sur les toits.
    Toits,
    /// Averse, ciel d'ardoise.
    Gouttieres,
    /// Orage au plus fort : éclairs, ciel violet.
    Orage,
    /// Le ciel se déchire, vent d'altitude.
    Altitude,
    /// Au-dessus des nuages : le soleil doré.
    Soleil,
}

impl Theme {
    /// Clarté du ciel de 0 (orage noir) à 4 (soleil) ; l'orage du niveau 3
    /// reste volontairement sombre, c'est le sommet de la tension.
    pub fn luminosite(self) -> u8 {
        match self {
            Theme::Toits => 0,
            Theme::Gouttieres => 1,
            Theme::Orage => 0,
            Theme::Altitude => 3,
            Theme::Soleil => 4,
        }
    }

    /// Pleut-il dans ce thème ?
    pub fn pluie(self) -> bool {
        !matches!(self, Theme::Soleil)
    }
}

/// Description d'un niveau : tout sauf le niveau chargé.
#[derive(Debug, Clone, Copy)]
pub struct InfosNiveau {
    /// Numéro de 1 à 5.
    pub numero: u8,
    /// Titre affiché.
    pub nom: &'static str,
    pub theme: Theme,
    /// Texte d'introduction (placeholder), une ou deux phrases courtes.
    pub intro: &'static str,
    /// Mécanique introduite (documentation et écran d'aide).
    pub mecanique: &'static str,
    /// Durée visée en secondes (de jeu normal, sans mort).
    pub duree_cible_s: u16,
    /// Dessin ASCII du niveau (voir la légende dans `level`).
    pub ascii: &'static str,
}

impl InfosNiveau {
    /// Lit le dessin ASCII. Ne peut échouer que si le niveau embarqué est
    /// invalide, ce que les tests de ce module interdisent.
    pub fn charger(&self) -> Result<Niveau, ErreurNiveau> {
        Niveau::depuis_ascii(self.ascii)
    }
}

/// Nombre de niveaux du jeu.
pub const NB_NIVEAUX: usize = 5;

/// Les cinq niveaux, dans l'ordre du parcours.
pub const NIVEAUX: [InfosNiveau; NB_NIVEAUX] = [
    toits::INFOS,
    gouttieres::INFOS,
    orage::INFOS,
    vents::INFOS,
    nuages::INFOS,
];

/// Nombre total de gouttes d'or du jeu.
pub const GOUTTES_TOTAL: usize = NB_NIVEAUX * crate::level::GOUTTES_PAR_NIVEAU;

/// Niveau `indice` (0 à 4) ou `None` hors bornes.
pub fn niveau(indice: usize) -> Option<&'static InfosNiveau> {
    NIVEAUX.get(indice)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faisabilite::{analyser, Options};

    #[test]
    fn cinq_niveaux_dans_l_ordre() {
        assert_eq!(NIVEAUX.len(), 5);
        for (i, n) in NIVEAUX.iter().enumerate() {
            assert_eq!(n.numero as usize, i + 1);
            assert!(!n.nom.is_empty() && !n.intro.is_empty() && !n.mecanique.is_empty());
        }
        assert_eq!(niveau(0).unwrap().nom, "Les toits");
        assert!(niveau(5).is_none());
    }

    #[test]
    fn tous_les_niveaux_se_chargent() {
        for n in &NIVEAUX {
            let niv = n
                .charger()
                .unwrap_or_else(|e| panic!("niveau {} : {e}", n.numero));
            assert_eq!(niv.hauteur(), 11, "niveau {} : 11 lignes", n.numero);
        }
    }

    #[test]
    fn niveau_1_faisable() {
        verifier(0);
    }
    #[test]
    fn niveau_2_faisable() {
        verifier(1);
    }
    #[test]
    fn niveau_3_faisable() {
        verifier(2);
    }
    #[test]
    fn niveau_4_faisable() {
        verifier(3);
    }
    #[test]
    fn niveau_5_faisable() {
        verifier(4);
    }

    #[test]
    fn niveau_1_exige_de_planer() {
        let niv = NIVEAUX[0].charger().unwrap();
        let r = analyser(
            &niv,
            niv.depart,
            Options {
                planer: false,
                ..Options::default()
            },
        );
        println!("sans planer : {:?}", r);
        assert!(!r.arrivee, "le grand vide doit exiger le parapluie");
    }

    #[test]
    fn niveau_4_exige_les_courants_ascendants() {
        let niv = NIVEAUX[3].charger().unwrap();
        assert!(!niv.courants.is_empty());
        let r = analyser(
            &niv,
            niv.depart,
            Options {
                courants: false,
                ..Options::default()
            },
        );
        assert!(!r.arrivee, "sans vent le niveau 4 doit être bloqué");
    }

    /// Vérifie un niveau : arrivée, 3 gouttes, toutes les lanternes atteignables,
    /// y compris par le robot « maladroit » (pas de précision au pixel près).
    fn verifier(i: usize) {
        let infos = &NIVEAUX[i];
        let niv = infos.charger().unwrap();
        let r = analyser(&niv, niv.depart, Options::default());
        let n = infos.numero;
        println!(
            "niveau {n} : {:.1} s en course rapide, {} états",
            r.pas_arrivee.unwrap_or(0) as f32 / 60.0,
            r.etats
        );
        assert!(
            r.arrivee,
            "niveau {n} : arrivée inatteignable (x max {} en {:?})",
            r.x_max, r.dernier_etat
        );
        assert_eq!(r.gouttes, [true; 3], "niveau {n} : gouttes {:?}", r.gouttes);
        assert!(
            r.lanternes.iter().all(|&l| l),
            "niveau {n} : lanternes {:?}",
            r.lanternes
        );
        assert_eq!(r.lanternes.len(), niv.nb_lanternes());
        assert!(
            niv.nb_lanternes() >= 3,
            "niveau {n} : au moins 3 checkpoints"
        );

        let maladroit = analyser(
            &niv,
            niv.depart,
            Options {
                precis: false,
                ..Options::default()
            },
        );
        assert!(
            maladroit.arrivee,
            "niveau {n} : trop exigeant (robot maladroit bloqué à x = {})",
            maladroit.x_max
        );
    }

    /// Chaque lanterne est aussi un point de reprise : le niveau doit se
    /// terminer DEPUIS elle (sinon un respawn pourrait coincer le joueur).
    #[test]
    fn reprise_depuis_chaque_lanterne() {
        for infos in &NIVEAUX {
            let niv = infos.charger().unwrap();
            for e in &niv.entites {
                if let crate::entities::Entite::Lanterne(l) = e {
                    let r = analyser(&niv, l.point_de_reapparition(), Options::default());
                    assert!(
                        r.arrivee,
                        "niveau {} : bloqué depuis la lanterne en {:?}",
                        infos.numero,
                        l.point_de_reapparition()
                    );
                }
            }
        }
    }

    /// Un niveau dure 40 à 60 s pour un joueur normal. Le robot « course
    /// rapide » est ~2 fois plus rapide qu'un humain découvrant le niveau.
    #[test]
    fn durees_plausibles() {
        for infos in &NIVEAUX {
            let niv = infos.charger().unwrap();
            let r = analyser(&niv, niv.depart, Options::default());
            let s = r.pas_arrivee.unwrap() as f32 / 60.0;
            println!(
                "niveau {} : course rapide {s:.1} s (cible {} s)",
                infos.numero, infos.duree_cible_s
            );
            assert!(
                (12.0..=45.0).contains(&s),
                "niveau {} : {s:.1} s",
                infos.numero
            );
            assert!((40..=60).contains(&infos.duree_cible_s));
        }
    }

    #[test]
    fn premier_defi_visible_des_le_premier_ecran() {
        // Fun immédiat : à l'écran de départ (320 px de large) on voit déjà une
        // goutte, un ennemi, un éclair, un courant ou un trampoline.
        for infos in &NIVEAUX {
            let niv = infos.charger().unwrap();
            let ecran = crate::LARGEUR_LOGIQUE as f32;
            let proche = niv.courants.iter().any(|c| c.zone.x < ecran)
                || niv.entites.iter().any(|e| match e {
                    crate::entities::Entite::Goutte(g) => g.boite.x < ecran,
                    crate::entities::Entite::Ennemi(en) => en.boite.x < ecran,
                    crate::entities::Entite::Eclair(ec) => ec.zone.x < ecran,
                    crate::entities::Entite::Champignon(c) => c.boite.x < ecran,
                    _ => false,
                });
            assert!(
                proche,
                "niveau {} : rien d'intéressant à l'écran de départ",
                infos.numero
            );
        }
    }

    /// Grille ASCII d'un niveau : les lignes du dessin, rangée 0 en haut.
    fn grille(infos: &InfosNiveau) -> Vec<Vec<char>> {
        infos
            .ascii
            .lines()
            .map(str::trim_end)
            .filter(|l| !l.is_empty())
            .map(|l| l.chars().collect())
            .collect()
    }

    /// Tuile pleine du dessin (`#` toit, `B` brique, `N` nuage, rien d'autre).
    /// Hors de la grille à gauche et à droite : mur ; en haut : ciel ouvert.
    fn plein(g: &[Vec<char>], x: i32, y: i32) -> bool {
        if x < 0 || x >= g[0].len() as i32 {
            return true;
        }
        if y < 0 || y >= g.len() as i32 {
            return false;
        }
        matches!(g[y as usize][x as usize], '#' | 'B' | 'N')
    }

    /// Une lanterne (point de reprise) ne flotte jamais : la case juste en
    /// dessous est un solide (`#`, `B` ou `N`).
    #[test]
    fn aucune_lanterne_ne_flotte() {
        for infos in &NIVEAUX {
            infos.charger().unwrap();
            let g = grille(infos);
            for (y, ligne) in g.iter().enumerate() {
                for (x, &c) in ligne.iter().enumerate() {
                    if c == 'L' {
                        assert!(
                            plein(&g, x as i32, y as i32 + 1) && y + 1 < g.len(),
                            "niveau {} : la lanterne de la colonne {x}, rangée {y} flotte \
                             (la case en dessous n'est pas un solide)",
                            infos.numero
                        );
                    }
                }
            }
        }
    }

    /// Aucune « poche close » : une case vide coiffée d'un solide dont la
    /// colonne de vide (jusqu'en bas) est bordée d'un solide des deux côtés à
    /// chaque rangée est inaccessible et se voit comme un trou sombre.
    #[test]
    fn aucune_poche_de_vide_close() {
        for infos in &NIVEAUX {
            infos.charger().unwrap();
            let g = grille(infos);
            for y in 1..g.len() as i32 {
                for x in 0..g[0].len() as i32 {
                    if plein(&g, x, y) || !plein(&g, x, y - 1) {
                        continue;
                    }
                    let mut fin = y;
                    while fin < g.len() as i32 && !plein(&g, x, fin) {
                        fin += 1;
                    }
                    let close = (y..fin).all(|r| plein(&g, x - 1, r) && plein(&g, x + 1, r));
                    assert!(
                        !close,
                        "niveau {} : poche de vide close en colonne {x}, rangées {y} à {} \
                         (coiffée et bordée de solide, inaccessible)",
                        infos.numero,
                        fin - 1
                    );
                }
            }
        }
    }

    /// Dangers qui passent sur le point de réapparition d'une lanterne, niveau
    /// par niveau : `(numéro du niveau, colonne de la lanterne, danger)`.
    ///
    /// Les entités du niveau avancent seules (sans joueur) pendant `pas` pas ; on
    /// note chaque lanterne dont la boîte de Pépin réapparu est touchée par un
    /// ennemi actif (flaque, corbeau), une pointe ou un éclair qui frappe.
    fn dangers_sur_les_lanternes(pas: u32) -> Vec<(u8, i32, &'static str)> {
        use crate::aabb::Aabb;
        use crate::entities::{solides_des_entites, Actualisable, Entite, TypeEnnemi};
        use crate::physics::{Environnement, JOUEUR_H, JOUEUR_L};
        let mut trouves = Vec::new();
        for infos in &NIVEAUX {
            let niv = infos.charger().unwrap();
            let pepins: Vec<(i32, Aabb)> = niv
                .entites
                .iter()
                .filter_map(|e| match e {
                    Entite::Lanterne(l) => {
                        let (x, y) = l.point_de_reapparition();
                        let colonne = (l.boite.x / crate::aabb::TAILLE_TUILE) as i32;
                        Some((colonne, Aabb::new(x, y, JOUEUR_L, JOUEUR_H)))
                    }
                    _ => None,
                })
                .collect();
            let mut entites = niv.entites.clone();
            for _ in 0..pas {
                let solides = solides_des_entites(&entites);
                let env = Environnement {
                    grille: &niv,
                    solides: &solides,
                    courants: &niv.courants,
                };
                for e in entites.iter_mut() {
                    e.actualiser(&env);
                }
                for e in &entites {
                    let (zone, nom) = match e {
                        Entite::Ennemi(en) if en.est_dangereux() => match en.type_ennemi {
                            TypeEnnemi::Flaque => (en.boite, "flaque"),
                            TypeEnnemi::Corbeau => (en.boite, "corbeau"),
                        },
                        Entite::Pointe(p) => (p.zone_danger(), "pointe"),
                        Entite::Eclair(ec) if ec.frappe() => (ec.zone, "éclair"),
                        _ => continue,
                    };
                    for &(colonne, pepin) in &pepins {
                        let trouve = (infos.numero, colonne, nom);
                        if pepin.chevauche(&zone) && !trouves.contains(&trouve) {
                            trouves.push(trouve);
                        }
                    }
                }
            }
        }
        trouves
    }

    /// Inventaire des lanternes sur le trajet d'un danger (carte 3.1b). Depuis la
    /// protection de réapparition (`PROTECTION_REAPPARITION_PAS`), ce n'est plus un
    /// piège (au plus une mort par passage), mais c'est à savoir en dessinant un
    /// niveau. Trois aujourd'hui, toutes sur la patrouille d'une flaque qui longe
    /// son toit d'un mur (ou d'un bord) à l'autre : niveau 2, colonnes 66 (flaque
    /// de la colonne 70) et 104 (flaque de la colonne 108, celle de la mesure 3.1) ;
    /// niveau 3, colonne 145 (flaque de la colonne 154). Si ce test échoue après une
    /// retouche de niveau : vérifier la nouvelle liste, la recopier ici et dans
    /// `QA.md` (section « Protection après réapparition »).
    #[test]
    fn lanternes_sur_le_trajet_d_un_danger() {
        // 2 min : plus qu'un aller-retour complet de chaque ennemi (la plus longue
        // patrouille de flaque, un toit de 61 tuiles au niveau 3, dure 80 s à 24 px/s).
        let trouves = dangers_sur_les_lanternes(120 * 60);
        println!("lanternes sur le trajet d'un danger : {trouves:?}");
        assert_eq!(
            trouves,
            vec![(2, 66, "flaque"), (2, 104, "flaque"), (3, 145, "flaque")]
        );
    }
}
