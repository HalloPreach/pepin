//! Tests propres au niveau « port » (autrefois le `mod tests` de
//! `core/src/levels/port.rs` ; la grille et ses commentaires de conception
//! sont dans son fichier `.ron`).

use crate::commun::*;
use pepin_core::aabb::{Aabb, TAILLE_TUILE};
use pepin_core::entities::ennemi::{decalage_vol, PhasePigeon, PLONGEE_PAS};
use pepin_core::entities::{Actualisable, Ennemi, Entite, TypeEnnemi};
use pepin_core::level::{Niveau, Tuile};
use pepin_core::physics::{Environnement, JOUEUR_H, JOUEUR_L, VITESSE_COURSE};

/// Le niveau de ce module, lu dans son fichier `.ron`.
fn infos() -> &'static InfosNiveau {
    par_cle("port")
}

/// Hauteur (px) du saut des 2 poissons de la fin du port (carte 9.11e) : la
/// valeur écrite dans `poissons_hauts` de `monde_2/port.ron`.
const HAUT_POISSONS_DE_LA_FIN: f32 = 96.0;

/// Six bateaux horizontaux et deux caisses de grue verticales, trois gouttes,
/// sept lanternes, un drapeau au bout de la jetée (carte 9.11d).
#[test]
fn le_port_a_ses_bateaux_sa_grue_et_ses_gouttes() {
    let n = infos().charger().unwrap();
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
    let n = infos().charger().unwrap();
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
    let n = infos().charger().unwrap();
    let env = Environnement::simple(&n);
    let t = TAILLE_TUILE;
    for m0 in mouettes(&n) {
        let colonne = (m0.boite.centre().0 / t) as i32;
        let sol = (1..n.hauteur() as i32)
            .find(|&ty| ty as f32 * t > m0.boite.bas() && n.tuile_decor(colonne, ty).est_solide())
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
    use pepin_core::entities::ennemi::HAUT_SAUT_POISSON;
    use pepin_core::levels::variantes::charger_ng_plus;
    let mut hauts = 0;
    let mut autres = 0;
    for infos in niveaux() {
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
    use pepin_core::aabb::Grille;
    use pepin_core::entities::ennemi::{PERIODE_POISSON_PAS, SOUS_L_EAU_PAS};
    use pepin_core::physics::{GRAVITE, HZ, VITESSE_SAUT};
    let n = infos().charger().unwrap();
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
