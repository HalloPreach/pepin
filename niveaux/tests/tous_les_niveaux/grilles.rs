//! Contrôles des grilles (autrefois dans `core/src/zones.rs`) : lanternes posées,
//! poches de vide, dangers près des points de reprise, grêle.

use crate::commun::*;

/// Ni le départ ni aucune lanterne sous la grêle : Pépin y apparaît (ou y
/// réapparaît) au sol, parapluie fermé, et la grêle le tuerait dès la fin de sa
/// protection (`PROTECTION_REAPPARITION_PAS`).
#[test]
fn aucun_point_de_reprise_sous_la_grele() {
    use pepin_core::aabb::Aabb;
    use pepin_core::entities::Entite;
    use pepin_core::physics::{JOUEUR_H, JOUEUR_L};
    for infos in niveaux_et_essai() {
        let niv = infos.charger().unwrap();
        let mut reprises = vec![niv.depart];
        for e in &niv.entites {
            if let Entite::Lanterne(l) = e {
                reprises.push(l.point_de_reapparition());
            }
        }
        for e in &niv.entites {
            let Entite::Grele(g) = e else { continue };
            for &(x, y) in &reprises {
                assert!(
                    !Aabb::new(x, y, JOUEUR_L, JOUEUR_H).chevauche(&g.zone),
                    "{} : point de reprise ({x}, {y}) sous la grêle {:?}",
                    infos.nom,
                    g.zone
                );
            }
        }
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

/// Tuile pleine du dessin (`#` toit, `B` brique, `N` nuage, et le mobilier de
/// rue `q a t d l` de la carte 9.20f, rien d'autre).
/// Hors de la grille à gauche et à droite : mur ; en haut : ciel ouvert.
fn plein(g: &[Vec<char>], x: i32, y: i32) -> bool {
    if x < 0 || x >= g[0].len() as i32 {
        return true;
    }
    if y < 0 || y >= g.len() as i32 {
        return false;
    }
    matches!(
        g[y as usize][x as usize],
        '#' | 'B' | 'N' | 'q' | 'a' | 't' | 'd' | 'l'
    )
}

/// Une lanterne (point de reprise) ne flotte jamais : la case juste en
/// dessous est un solide (`#`, `B` ou `N`), ou de l'eau (carte 9.2 : posée
/// sur l'eau comme une goutte, Pépin y réapparaît protégé et l'eau le
/// recueille en barque, voir `Joueur::actualiser`).
#[test]
fn aucune_lanterne_ne_flotte() {
    for infos in niveaux_et_essai() {
        infos.charger().unwrap();
        let g = grille(infos);
        for (y, ligne) in g.iter().enumerate() {
            for (x, &c) in ligne.iter().enumerate() {
                if c == 'L' {
                    let sur_l_eau = y + 1 < g.len() && matches!(g[y + 1][x], '~' | '{' | '}');
                    assert!(
                        (plein(&g, x as i32, y as i32 + 1) || sur_l_eau) && y + 1 < g.len(),
                        "niveau {} : la lanterne de la colonne {x}, rangée {y} flotte \
                         (la case en dessous n'est ni un solide ni de l'eau)",
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
    for infos in niveaux_et_essai() {
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
///
/// Le chat ne bouge pas sans Pépin : son bond (carte 6.3) se calcule à part,
/// lanterne par lanterne (`Ennemi::bond_atteint` : Pépin réapparu immobile à la
/// lanterne, le chat bondirait-il sur lui à son réveil ?), noté « bond de chat ».
fn dangers_sur_les_lanternes(pas: u32) -> Vec<(u8, i32, &'static str)> {
    use pepin_core::aabb::Aabb;
    use pepin_core::entities::{solides_des_entites, Actualisable, Entite, TypeEnnemi};
    use pepin_core::physics::{Environnement, JOUEUR_H, JOUEUR_L};
    let mut trouves = Vec::new();
    for infos in niveaux() {
        let niv = infos.charger().unwrap();
        let pepins: Vec<(i32, Aabb)> = niv
            .entites
            .iter()
            .filter_map(|e| match e {
                Entite::Lanterne(l) => {
                    let (x, y) = l.point_de_reapparition();
                    let colonne = (l.boite.x / pepin_core::aabb::TAILLE_TUILE) as i32;
                    Some((colonne, Aabb::new(x, y, JOUEUR_L, JOUEUR_H)))
                }
                _ => None,
            })
            .collect();
        let solides = solides_des_entites(&niv.entites);
        for e in &niv.entites {
            if let Entite::Ennemi(c) = e {
                for &(colonne, pepin) in &pepins {
                    if c.type_ennemi == TypeEnnemi::Chat && c.bond_atteint(&pepin, &niv, &solides) {
                        trouves.push((infos.numero, colonne, "bond de chat"));
                    }
                }
            }
        }
        let mut entites = niv.entites.clone();
        for _ in 0..pas {
            let solides = solides_des_entites(&entites);
            let env = Environnement {
                grille: &niv,
                solides: &solides,
                courants: &niv.courants,
                vents: &niv.vents,
                eaux: &[],
            };
            for e in entites.iter_mut() {
                e.actualiser(&env);
            }
            for e in &entites {
                let (zone, nom) = match e {
                    Entite::Ennemi(en) if en.est_dangereux() => match en.type_ennemi {
                        TypeEnnemi::Flaque => (en.boite, "flaque"),
                        TypeEnnemi::Corbeau => (en.boite, "corbeau"),
                        TypeEnnemi::Pigeon => (en.boite, "pigeon"),
                        TypeEnnemi::Chat => (en.boite, "chat"),
                        TypeEnnemi::Grenouille => (en.boite, "grenouille"),
                        TypeEnnemi::Poisson => (en.boite, "poisson"),
                        TypeEnnemi::Crabe => (en.boite, "crabe"),
                        TypeEnnemi::Rat => (en.boite, "rat"),
                        TypeEnnemi::Mouette => (en.boite, "mouette"),
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
/// niveau 4 (le grand orage, niveau 3 avant l'arrivée du marché couvert),
/// colonne 145 (flaque de la colonne 154). Depuis la carte 6.3, le marché
/// couvert en ajoute deux, à portée du BOND d'un chat (ses flaques gardent des
/// toits sans lanterne) : colonne 50 (chat de la colonne 54) et colonne 110
/// (chat de la colonne 114). Pas un piège non plus : à la réapparition, la
/// protection (1 s) couvre tout un bond en cours ; le bond suivant vient au
/// plus tôt 4 s plus tard, annoncé 0,8 s par le « ! » ; au plus une mort par
/// réveil, pour un joueur qui reste planté à la lanterne. Si ce test échoue
/// après une retouche de niveau : vérifier la nouvelle liste, la recopier ici
/// et dans `QA.md` (section « Protection après réapparition »).
#[test]
fn lanternes_sur_le_trajet_d_un_danger() {
    // 2 min : plus qu'un aller-retour complet de chaque ennemi (la plus longue
    // patrouille de flaque, un toit de 61 tuiles au niveau 4, dure 80 s à 24 px/s).
    let trouves = dangers_sur_les_lanternes(120 * 60);
    println!("lanternes sur le trajet d'un danger : {trouves:?}");
    assert_eq!(
        trouves,
        vec![
            (2, 66, "flaque"),
            (2, 104, "flaque"),
            (3, 50, "bond de chat"),
            (3, 110, "bond de chat"),
            (4, 145, "flaque")
        ]
    );
}

// --- Objets de décor (carte 4.2) -----------------------------------------
