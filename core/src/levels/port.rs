//! Monde 2, niveau 6 : « Le port » (carte 9.11, refait aux cartes 9.11c et
//! 9.11d). Juste
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
//! Carte 9.11d (AGA : « le port : il est devenu court et trop simple ») : deux
//! fois plus long, plus dur (enchaînements de bateaux sans repos, poissons dans
//! les trous, crabes sur des pontons étroits, conteneurs et grue au-dessus du
//! vide, corbeaux au-dessus de l'eau) ; 3 mouettes, toujours sur des passerelles.
//!
//! Lecture du niveau (227 x 11 tuiles, un écran de haut), de gauche à droite :
//! 1. le QUAI DU DÉPART (colonnes 0 à 10) : une caisse et sa goutte ;
//! 2. le BASSIN 1 (11 à 24) : le BATEAU 1, lent, et un poisson avant le quai 2 ;
//! 3. le QUAI 2 (25 à 44) : la lanterne 1, une caisse, la MOUETTE 1 sur sa
//!    passerelle (colonnes 38 à 40, rangée 1) au-dessus du quai où l'on court ;
//! 4. le BASSIN 2 (45 à 59) : DEUX bateaux à la suite, de courses différentes
//!    (ils se rapprochent puis s'éloignent), un poisson qui saute dans le trou ;
//! 5. le QUAI 3 (60 à 66, lanterne 2) puis la CALE SÈCHE (67 à 82), vide jusqu'en
//!    bas : deux piles de conteneurs, la caisse d'une grue (colonne 77, la 2e
//!    goutte au-dessus de son point haut), la 3e pile ;
//! 6. le VOL PLANÉ au-dessus du bassin 3 (83 à 92), un corbeau dans le vol ; la
//!    3e goutte, cachée, attend au ras de l'eau (barque, puis ponton) ;
//! 7. le QUAI 4 (93 à 110) : la lanterne 3, la MOUETTE 2 sur sa passerelle
//!    (colonnes 102 à 104) ;
//! 8. le BASSIN 4 (111 à 124) : deux PONTONS étroits, chacun son crabe, un
//!    poisson entre eux, puis le DUC D'ALBE (125-126, lanterne 4) ;
//! 9. le BASSIN 5 (127 à 149) : TROIS bateaux sans repos, de courses
//!    différentes, un corbeau au-dessus, un poisson entre les deux derniers ;
//! 10. le QUAI 5 (150 à 159, lanterne 5), la DARSE de la grue (sa caisse,
//!     colonnes 163-164, monte au TOIT DE L'ENTREPÔT, 165 à 172) d'où l'on PLANE
//!     au-dessus du bassin 6 et de son poisson jusqu'au quai 6 (lanterne 6) ;
//! 11. le BASSIN 7 (191 à 204) : trois PIEUX d'amarrage, deux poissons entre eux
//!     qui sautent PLUS HAUT (6 tuiles, `poissons_hauts`, carte 9.11e) ;
//! 12. le BOUT DE LA JETÉE (205 à 226) : la lanterne 7, la MOUETTE 3 sur sa
//!     passerelle (colonnes 214 à 216), un crabe entre deux caisses, le drapeau.
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
    mecanique: "Bateaux en va-et-vient et à la suite, conteneurs et grue au-dessus du \
                vide, caisse de grue qui monte au toit de l'entrepôt, vols planés, \
                pontons étroits à crabes, pieux, mouettes perchées sur les passerelles.",
    duree_cible_s: 60,
    ascii: ASCII,
    aiguilles: &[],
    // Les 2 poissons de la fin (bassin 7, entre les pieux) sautent 6 tuiles au
    // lieu de 3,5 : ils menacent Pépin qui saute de pieu en pieu (carte 9.11e).
    // Debout sur un pieu, on ne les touche pas : on attend qu'ils retombent.
    poissons_hauts: &[
        (196, 8, HAUT_POISSONS_DE_LA_FIN),
        (200, 8, HAUT_POISSONS_DE_LA_FIN),
    ],
};

/// Hauteur (px) du saut des 2 poissons de la fin du port (carte 9.11e).
pub const HAUT_POISSONS_DE_LA_FIN: f32 = 96.0;

/// 227 x 11 tuiles, thème Port. `1`, `3`, `4`, `6`, `7`, `9` : les bateaux
/// (rangée 8, posés sur l'eau) ; `5`, `2` : les caisses des grues (colonnes 77
/// et 163) ; `u` : les mouettes, sur leurs passerelles ; `c` : les corbeaux.
const ASCII: &str = r"
......................................u.......................................o.......................u....................................................................K..........................................u............
......................................###.............................................................###..........................................................2.BBBBBBBB.........................................###..........
.............................................................................5...##..................................................................................BBBBBBBB......................................................
.....o..............................................................##...........##....c.............................................c...............................BBBBBBBB......................................................
.@...#.....................L...#.............................L......##..##.......##...........L..............................L.........................L.............BBBBBBBB.........L.......................L............#.k.#.X.
BBBBBBBBBBB..............BBBBBBBBBBBBBBBBBBBB...............BBBBBBB.##..##.......##..........BBBBBBBBBBBBBBBBBB..............BB.......................BBBBBBBBBB.....BBBBBBBB........BBBBBBBBBB..............BBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBB..............BBBBBBBBBBBBBBBBBBBB...............BBBBBBB.##..##.......##..........BBBBBBBBBBBBBBBBBB....k.....k...BB.......................BBBBBBBBBB.....BBBBBBBB........BBBBBBBBBB...#...#...#..BBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBB#.............BBBBBBBBBBBBBBBBBBBB#..............BBBBBBB.##..##.......##......o..#BBBBBBBBBBBBBBBBBB#..###...###..BB#......................BBBBBBBBBB.....BBBBBBBB.......#BBBBBBBBBB#..#...#...#..BBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBB.1........1.i.BBBBBBBBBBBBBBBBBBBB.3..3.i4....4..BBBBBBB.##..##...5...##..........BBBBBBBBBBBBBBBBBB.......i......BB.6..6..7....7.i9...9...BBBBBBBBBB...2.BBBBBBBB....i...BBBBBBBBBB...#.i.#.i.#..BBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBB~~~~~~~~~~~~~~BBBBBBBBBBBBBBBBBBBB~~~~~~~~~~~~~~~BBBBBBB.##..##.......##~~~~~~~~~~BBBBBBBBBBBBBBBBBB~~~~~~~~~~~~~~BB~~~~~~~~~~~~~~~~~~~~~~~BBBBBBBBBB~~~~~BBBBBBBB~~~~~~~~BBBBBBBBBB~~~~~~~~~~~~~~BBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB.##..##.......##BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::{Aabb, TAILLE_TUILE};
    use crate::entities::ennemi::{decalage_vol, PhasePigeon, PLONGEE_PAS};
    use crate::entities::{Actualisable, Ennemi, Entite, TypeEnnemi};
    use crate::level::{Niveau, Tuile};
    use crate::physics::{Environnement, JOUEUR_H, JOUEUR_L, VITESSE_COURSE};

    /// Six bateaux horizontaux et deux caisses de grue verticales, trois gouttes,
    /// sept lanternes, un drapeau au bout de la jetée (carte 9.11d).
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
        assert_eq!((horizontaux, verticaux), (6, 2));
        let gouttes = n
            .entites
            .iter()
            .filter(|e| matches!(e, Entite::Goutte(_)))
            .count();
        assert_eq!(gouttes, 3);
        assert_eq!(n.nb_lanternes(), 7);
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

    /// Carte 9.11c (3 depuis la carte 9.11d) : les mouettes, chacune tournée
    /// vers Pépin qui arrive (vers la
    /// gauche), écrasable, et dont la plongée ne passe sur AUCUN endroit où l'on
    /// s'arrête : le départ, une lanterne, un bateau ou la caisse de la grue à
    /// l'un de leurs bouts, et toute case de sol à moins de 4 tuiles d'un bassin
    /// (là où l'on attend un bateau, où l'on saute, où l'on atterrit).
    #[test]
    fn aucune_mouette_ne_garde_un_endroit_ou_l_on_s_arrete() {
        let n = INFOS.charger().unwrap();
        let ms = mouettes(&n);
        assert_eq!(ms.len(), 3);
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

    /// Carte 9.11e : seuls les 2 poissons de la fin du port sautent plus haut ;
    /// tous les autres poissons du jeu (NG+ compris) gardent `HAUT_SAUT_POISSON`,
    /// et la hauteur réglée survit à une mort de Pépin.
    #[test]
    fn seuls_les_deux_poissons_de_la_fin_sautent_plus_haut() {
        use crate::entities::ennemi::HAUT_SAUT_POISSON;
        use crate::levels::{variantes::charger_ng_plus, NIVEAUX};
        let mut hauts = 0;
        let mut autres = 0;
        for infos in NIVEAUX {
            for n in [infos.charger().unwrap(), charger_ng_plus(infos).unwrap()] {
                for e in &n.entites {
                    let Entite::Ennemi(p) = e else { continue };
                    if p.type_ennemi != TypeEnnemi::Poisson {
                        continue;
                    }
                    let fin =
                        infos.cle == "port" && [(196, 8), (200, 8)].contains(&p.case_du_poisson());
                    let attendu = if fin {
                        hauts += 1;
                        HAUT_POISSONS_DE_LA_FIN
                    } else {
                        autres += 1;
                        HAUT_SAUT_POISSON
                    };
                    assert_eq!(
                        p.saut_poisson(),
                        attendu,
                        "{} {:?}",
                        infos.cle,
                        p.case_du_poisson()
                    );
                    let mut mort = p.clone();
                    mort.revenir_a_l_origine();
                    assert_eq!(mort.saut_poisson(), attendu);
                }
            }
        }
        assert_eq!(hauts, 4, "2 poissons, chargement normal et NG+");
        assert!(autres > 0);
        assert_eq!(HAUT_SAUT_POISSON, 56.0);
    }

    /// Carte 9.11e : au sommet, le poisson de la fin monte à la hauteur de Pépin
    /// en plein saut de pieu en pieu, mais il ne touche JAMAIS Pépin debout sur un
    /// pieu (ni sur un quai) : on attend qu'il retombe. Et il reste caché sous
    /// l'eau plus longtemps qu'un saut complet ne dure : on a le temps de passer.
    #[test]
    fn les_poissons_de_la_fin_menacent_le_saut_sans_toucher_les_pieux() {
        use crate::aabb::Grille;
        use crate::entities::ennemi::{PERIODE_POISSON_PAS, SOUS_L_EAU_PAS};
        use crate::physics::{GRAVITE, HZ, VITESSE_SAUT};
        let n = INFOS.charger().unwrap();
        let env = Environnement::simple(&n);
        let t = TAILLE_TUILE;
        // Les appuis du bassin 7 et de ses bords : le dessus de chaque case
        // solide sous une case d'air (pas d'eau), des colonnes 189 à 206.
        let mut appuis = Vec::new();
        for tx in 189..=206 {
            for ty in 1..n.hauteur() as i32 {
                if n.est_solide(tx, ty) && n.tuile_decor(tx, ty - 1) == Tuile::Vide {
                    appuis.push((tx, ty));
                    break;
                }
            }
        }
        let pieu = appuis.iter().find(|&&(tx, _)| tx == 194).unwrap().1 as f32 * t;
        for e in &n.entites {
            let Entite::Ennemi(p0) = e else { continue };
            if p0.type_ennemi != TypeEnnemi::Poisson || p0.case_du_poisson().0 < 190 {
                continue;
            }
            let mut p = p0.clone();
            let mut sommet = f32::MAX;
            for _ in 0..PERIODE_POISSON_PAS {
                p.actualiser(&env);
                sommet = sommet.min(p.boite.y);
                if !p.est_dangereux() {
                    continue;
                }
                for &(tx, ty) in &appuis {
                    let sol = ty as f32 * t;
                    let mut x = tx as f32 * t - JOUEUR_L + 1.0;
                    while x < (tx + 1) as f32 * t {
                        let pepin = Aabb::new(x, sol - JOUEUR_H, JOUEUR_L, JOUEUR_H);
                        assert!(
                            !p.boite.chevauche(&pepin),
                            "poisson {:?} : touche Pépin debout en x = {x}",
                            p0.case_du_poisson()
                        );
                        x += 1.0;
                    }
                }
            }
            // Pépin en plein saut au-dessus du poisson a les pieds 40 à 57 px
            // au-dessus du pieu : le poisson monte au moins jusque-là.
            assert!(
                sommet <= pieu - 40.0,
                "poisson {:?} : sommet {sommet}, pieu {pieu}",
                p0.case_du_poisson()
            );
        }
        let duree_saut = 2.0 * VITESSE_SAUT / GRAVITE;
        assert!(f32::from(SOUS_L_EAU_PAS) / HZ as f32 > 1.5 * duree_saut);
    }
}
