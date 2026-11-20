//! Monde 2, niveau 6 : « Le port » (carte 9.11, refait à la carte 9.11c). Juste
//! avant le repaire de la Pieuvre : la pluie arrive au port, et Pépin le traverse
//! de quai en quai, porté par les bateaux et par la caisse d'une grue, jusqu'au
//! BOUT DE LA JETÉE, là où commence « La Pieuvre du port ». Plan d'AGA : « Le
//! port : bateaux en plateformes mobiles, grues, mouettes ».
//!
//! Carte 9.11c (AGA : « les mouettes du port rendent le niveau infinissable pour
//! un humain, et il y a trop de fois le même pattern d'obstacles ») : aucune
//! mouette ne garde plus un endroit où l'on s'arrête (bord de quai avant un
//! bateau, pieu d'atterrissage) ; chacune est perchée sur une PASSERELLE au-dessus
//! d'un quai où l'on court, tournée vers Pépin qui arrive : on passe dessous en
//! courant, ou on la fait plonger et on passe pendant qu'elle remonte (ou on
//! l'écrase alors d'un saut). Leur temps n'a pas changé (celui du pigeon).
//! Et chaque passage est différent : aucun ne revient plus de deux fois.
//!
//! Lecture du niveau (110 x 11 tuiles, un écran de haut), de gauche à droite :
//! 1. le QUAI DU DÉPART (colonnes 0 à 10) : une caisse et sa goutte ;
//! 2. le BASSIN 1 (colonnes 11 à 24, trop large pour un saut plané) : le BATEAU 1
//!    va et vient ; un poisson guette dans l'eau libre avant le quai 2 ;
//! 3. le QUAI 2 (colonnes 25 à 42) : la lanterne 1, une PILE DE CONTENEURS
//!    qu'on gravit et redescend (la 2e goutte au sommet), puis la MOUETTE 1 sur
//!    sa passerelle (colonnes 40 à 42, rangée 1) au-dessus du quai ;
//! 4. la DARSE de la GRUE (colonnes 43 à 47) : sa caisse (colonnes 46-47) monte
//!    de l'eau jusqu'au toit de l'entrepôt, trop haut pour un saut ;
//! 5. le TOIT DE L'ENTREPÔT (colonnes 48 à 55, rangée 1 : la lanterne 2, une
//!    mouette au repos en décor) d'où l'on PLANE au-dessus du bassin suivant
//!    (colonnes 56 à 63) jusqu'au quai bas ; la 3e goutte, un peu cachée, attend
//!    au ras de l'eau : il faut s'y laisser tomber, puis remonter en barque ;
//! 6. le QUAI BAS (colonnes 64 à 77) : la lanterne 3, deux caisses empilées, la
//!    MOUETTE 2 sur sa passerelle (colonnes 73 à 75) ;
//! 7. le PONTON FLOTTANT étroit (colonnes 81 à 83) et son crabe, puis le DUC
//!    D'ALBE (le pilier d'amarrage, colonnes 87-88, la lanterne 4) ;
//! 8. le BASSIN 2 (colonnes 89 à 99) : le BATEAU 2, un poisson avant la jetée ;
//! 9. le BOUT DE LA JETÉE (colonnes 100 à 109) : un crabe entre deux caisses, et
//!    le drapeau.
//!
//! Tombé à l'eau en planant : la barque. Contre chaque quai de départ d'un bateau
//! et au bord du quai bas, un ponton (`#` rangée 7) fait l'échelle : barque,
//! ponton, quai ; dans la darse, la caisse de la grue, quand elle descend.
//!
//! Le rendu (`game/src/scene.rs`) dessine les plateformes mobiles du port en
//! bateaux (va-et-vient horizontal) ou en caisse pendue à son câble (vertical).

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    monde: 2,
    // Recette de l'étape 9 : 5 dans SA branche (la Pieuvre passe à 6) ; place
    // finale 6 (après la rivière), rangée par le coordinateur à la fusion.
    numero: 6,
    cle: "port",
    sans_planer: false,
    nom: "Le port",
    theme: Theme::Port,
    genre: GenreNiveau::Normal,
    intro: "La pluie arrive au port. Au bout de la jetée, quelque chose remue dans l'eau.",
    mecanique: "Bateaux en va-et-vient, pile de conteneurs, caisse de grue qui monte au \
                toit de l'entrepôt, vol plané au-dessus du bassin, ponton flottant, \
                mouettes perchées sur les passerelles.",
    duree_cible_s: 60,
    ascii: ASCII,
    aiguilles: &[],
};

/// 110 x 11 tuiles, thème Port. `1`, `3` : les bateaux (rangée 8, posés sur
/// l'eau) ; `2` : la caisse de la grue (colonne 46, rangées 8 à 1) ; `u` : les
/// mouettes, sur leurs passerelles.
const ASCII: &str = r"
........................................u.........L...K..................u....................................
................................o.......###...2.BBBBBBBB.................###..................................
...............................##...............BBBBBBBB......................................................
.....o........................####..............BBBBBBBB...........#..........................................
.@...#....................L..######.............BBBBBBBB.........L.#...................L..............#.k.#.X.
BBBBBBBBBBB..............BBBBBBBBBBBBBBBBBB.....BBBBBBBB........BBBBBBBBBBBBBB.........BB...........BBBBBBBBBB
BBBBBBBBBBB..............BBBBBBBBBBBBBBBBBB.....BBBBBBBB....o...BBBBBBBBBBBBBB....k....BB...........BBBBBBBBBB
BBBBBBBBBBB#.............BBBBBBBBBBBBBBBBBB.....BBBBBBBB.......#BBBBBBBBBBBBBB...###...BB#..........BBBBBBBBBB
BBBBBBBBBBB.1........1.i.BBBBBBBBBBBBBBBBBB...2.BBBBBBBB........BBBBBBBBBBBBBB.........BB.3.....3.i.BBBBBBBBBB
BBBBBBBBBBB~~~~~~~~~~~~~~BBBBBBBBBBBBBBBBBB~~~~~BBBBBBBB~~~~~~~~BBBBBBBBBBBBBB~~~~~~~~~BB~~~~~~~~~~~BBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::{Aabb, TAILLE_TUILE};
    use crate::entities::ennemi::{decalage_vol, PhasePigeon, PLONGEE_PAS};
    use crate::entities::{Actualisable, Ennemi, Entite, TypeEnnemi};
    use crate::level::{Niveau, Tuile};
    use crate::physics::{Environnement, JOUEUR_H, JOUEUR_L, VITESSE_COURSE};

    /// Deux bateaux horizontaux et une caisse de grue verticale, trois gouttes,
    /// quatre lanternes, un drapeau au bout de la jetée.
    #[test]
    fn le_port_a_ses_bateaux_sa_grue_et_ses_gouttes() {
        let n = INFOS.charger().unwrap();
        let mobiles: Vec<_> = n
            .entites
            .iter()
            .filter_map(|e| match e {
                Entite::Mobile(m) => m.extremites(),
                _ => None,
            })
            .collect();
        let horizontaux = mobiles.iter().filter(|(a, b)| a.1 == b.1).count();
        let verticaux = mobiles.iter().filter(|(a, b)| a.0 == b.0).count();
        assert_eq!((horizontaux, verticaux), (2, 1));
        let gouttes = n
            .entites
            .iter()
            .filter(|e| matches!(e, Entite::Goutte(_)))
            .count();
        assert_eq!(gouttes, 3);
        assert_eq!(n.nb_lanternes(), 4);
        assert!(n.arrivee.is_some());
    }

    fn mouettes(n: &Niveau) -> Vec<Ennemi> {
        n.entites
            .iter()
            .filter_map(|e| match e {
                Entite::Ennemi(m) if m.type_ennemi == TypeEnnemi::Mouette => Some(m.clone()),
                _ => None,
            })
            .collect()
    }

    /// Toutes les boîtes de la plongée d'une mouette.
    fn plongee(m: &Ennemi) -> Vec<Aabb> {
        let sens = if m.vers_la_droite { 1.0 } else { -1.0 };
        (1..=PLONGEE_PAS)
            .map(|k| {
                let (dx, dy) = decalage_vol(PhasePigeon::Plongee(k), sens);
                m.boite.decale(dx, dy)
            })
            .collect()
    }

    /// Carte 9.11c : 2 mouettes, chacune tournée vers Pépin qui arrive (vers la
    /// gauche), écrasable, et dont la plongée ne passe sur AUCUN endroit où l'on
    /// s'arrête : le départ, une lanterne, un bateau ou la caisse de la grue à
    /// l'un de leurs bouts, et toute case de sol à moins de 4 tuiles d'un bassin
    /// (là où l'on attend un bateau, où l'on saute, où l'on atterrit).
    #[test]
    fn aucune_mouette_ne_garde_un_endroit_ou_l_on_s_arrete() {
        let n = INFOS.charger().unwrap();
        let ms = mouettes(&n);
        assert_eq!(ms.len(), 2);
        let mut arrets: Vec<(Aabb, String)> = vec![(
            Aabb::new(n.depart.0, n.depart.1, JOUEUR_L, JOUEUR_H),
            "départ".into(),
        )];
        for e in &n.entites {
            match e {
                Entite::Lanterne(l) => {
                    let (x, y) = l.point_de_reapparition();
                    arrets.push((Aabb::new(x, y, JOUEUR_L, JOUEUR_H), "lanterne".into()));
                }
                Entite::Mobile(m) => {
                    let (a, b) = m.extremites().unwrap();
                    for (x, y) in [a, b] {
                        let sur = Aabb::new(x, y - JOUEUR_H, m.boite.w, JOUEUR_H);
                        arrets.push((sur, "plateforme mobile".into()));
                    }
                }
                _ => {}
            }
        }
        let (larg, haut) = (n.largeur() as i32, n.hauteur() as i32);
        let bassin = |tx: i32| (0..haut).any(|ty| n.tuile_decor(tx, ty) == Tuile::Eau);
        for tx in 0..larg {
            if !(tx - 4..=tx + 4).any(|c| (0..larg).contains(&c) && bassin(c)) {
                continue;
            }
            for ty in 1..haut {
                if n.tuile_decor(tx, ty).est_solide() && !n.tuile_decor(tx, ty - 1).est_solide() {
                    let sol = ty as f32 * TAILLE_TUILE;
                    let case = Aabb::new(tx as f32 * TAILLE_TUILE, sol - JOUEUR_H, 16.0, JOUEUR_H);
                    arrets.push((case, format!("sol au bord d'un bassin ({tx}, {ty})")));
                }
            }
        }
        for m in &ms {
            assert!(
                !m.vers_la_droite,
                "mouette {:?} : tournée vers Pépin",
                m.perchoir()
            );
            assert!(m.est_ecrasable());
            for b in plongee(m) {
                // Sa propre passerelle (d'où elle part) ne compte pas.
                for (a, nom) in arrets.iter().filter(|(a, _)| !a.chevauche(&m.boite)) {
                    assert!(
                        !b.chevauche(a),
                        "mouette {:?} : plonge sur {nom}",
                        m.perchoir()
                    );
                }
            }
        }
    }

    /// On esquive chaque mouette EN COURANT : Pépin qui court sous elle sans
    /// s'arrêter (de 6 tuiles avant à 4 tuiles après, sur le sol de sa colonne)
    /// déclenche sa plongée mais n'est jamais touché. Et on peut toujours
    /// l'écraser par le dessus, d'un saut depuis le quai, quand elle remonte.
    #[test]
    fn on_passe_sous_chaque_mouette_en_courant_et_on_peut_l_ecraser() {
        let n = INFOS.charger().unwrap();
        let env = Environnement::simple(&n);
        let t = TAILLE_TUILE;
        for m0 in mouettes(&n) {
            let colonne = (m0.boite.centre().0 / t) as i32;
            let sol = (1..n.hauteur() as i32)
                .find(|&ty| {
                    ty as f32 * t > m0.boite.bas() && n.tuile_decor(colonne, ty).est_solide()
                })
                .expect("un quai sous la mouette") as f32
                * t;
            let mut m = m0.clone();
            let mut x = m.boite.x - 6.0 * t;
            let fin = m.boite.x + 4.0 * t;
            let mut plonge = false;
            while x < fin {
                let p = Aabb::new(x, sol - JOUEUR_H, JOUEUR_L, JOUEUR_H);
                m.guetter(&p);
                m.actualiser(&env);
                plonge |= matches!(m.phase(), PhasePigeon::Plongee(_));
                assert!(
                    !(m.est_dangereux() && m.boite.chevauche(&p)),
                    "mouette {:?} : touche Pépin qui court en x = {x}",
                    m0.perchoir()
                );
                x += VITESSE_COURSE / 60.0;
            }
            assert!(plonge, "mouette {:?} : jamais déclenchée", m0.perchoir());
            // Au bas de sa plongée, elle frôle le quai sans y entrer, et son dos
            // est 10 px sous le sommet d'un saut (57 px) : en remontant (lentement)
            // elle s'écrase d'un saut depuis le quai.
            let bas = plongee(&m0).last().copied().unwrap();
            assert!(
                bas.bas() <= sol,
                "mouette {:?} : plonge dans le quai",
                m0.perchoir()
            );
            assert!(
                bas.y >= sol - 57.0 + 10.0,
                "mouette {:?} : hors de portée d'un saut",
                m0.perchoir()
            );
        }
    }
}
