//! Le décor des niveaux (autrefois dans `core/src/zones.rs`) : chaque objet repose
//! sur un solide, ne change rien à la collision, ne cache aucun danger ; manches à air.

use crate::commun::*;

use pepin_core::level::{GenreDecor, ObjetDecor};

/// Case `(tx, ty)` d'un objet de décor.
pub(crate) fn case_decor(d: &ObjetDecor) -> (i32, i32) {
    let t = pepin_core::aabb::TAILLE_TUILE;
    ((d.x / t) as i32, (d.y / t) as i32)
}

/// Place qu'un objet de décor peut occuper à l'écran : sa case et celle du
/// dessus (les plus hauts sprites font 29 px), élargies de 4 px de chaque côté
/// (les sprites des objets et des ennemis débordent un peu de leur boîte).
fn place_decor(d: &ObjetDecor) -> pepin_core::aabb::Aabb {
    let t = pepin_core::aabb::TAILLE_TUILE;
    pepin_core::aabb::Aabb::new(d.x - 4.0, d.y - t, t + 8.0, 2.0 * t)
}

/// Chaque objet de décor est posé au milieu d'un toit : sa case est vide, la
/// case du dessous est solide, et ses deux voisines du dessous aussi (jamais au
/// bord d'un toit, là où l'on prend son élan ou où l'on se rattrape).
#[test]
fn chaque_objet_de_decor_repose_sur_un_solide() {
    for infos in niveaux() {
        let niv = infos.charger().unwrap();
        for d in &niv.decors {
            let (tx, ty) = case_decor(d);
            let n = infos.numero;
            assert!(!niv.tuile_decor(tx, ty).est_solide());
            for dx in -1..=1 {
                assert!(
                    niv.tuile_decor(tx + dx, ty + 1).est_solide() && ty + 1 < niv.hauteur() as i32,
                    "niveau {n} : {:?} en colonne {tx}, rangée {ty} : la case ({}, {}) \
                     dessous n'est pas un solide",
                    d.genre,
                    tx + dx,
                    ty + 1
                );
            }
        }
    }
}

/// Le décor ne sert qu'au dessin : en remplaçant ses lettres par `.`, le niveau
/// est EXACTEMENT le même (tuiles, entités, courants, panneaux, départ,
/// arrivée). Aucun risque pour la faisabilité, le robot ni la mesure de difficulté.
#[test]
fn le_decor_ne_change_rien_a_la_collision() {
    for infos in niveaux() {
        let avec = infos.charger().unwrap();
        let sans_texte: String = infos
            .ascii
            .chars()
            .map(|c| match GenreDecor::depuis_lettre(c) {
                Some(_) => '.',
                None => c,
            })
            .collect();
        let mut sans = Niveau::depuis_ascii_avec_aiguilles(&sans_texte, infos.aiguilles).unwrap();
        sans.regler_sauts_poissons(infos.poissons_hauts).unwrap();
        let n = infos.numero;
        // Chaque niveau a son décor, sauf l'arène du Grand Nuage (pas de
        // drapeau) : ses 17 colonnes d'éclairs couvrent tout le sol, et un objet
        // de décor n'est jamais devant un danger (carte 4.2). Son décor, c'est
        // le fond (trouée d'or, tours de nuages) et le nuage lui-même. Les
        // égouts non plus (carte 9.27 : leurs souches se lisaient comme des
        // cheminées de toit ; tous les objets de décor sont de dehors) : leur
        // décor, c'est leur habillage (grilles, tuyaux, voûtes).
        if infos.theme == Theme::Egouts {
            assert!(
                avec.decors.is_empty(),
                "niveau {n} : décor de dehors sous terre"
            );
        } else if avec.arrivee.is_some() {
            assert!(!avec.decors.is_empty(), "niveau {n} : aucun décor");
        }
        assert!(sans.decors.is_empty());
        assert_eq!(
            (avec.largeur(), avec.hauteur()),
            (sans.largeur(), sans.hauteur())
        );
        for ty in -1..=avec.hauteur() as i32 {
            for tx in -1..=avec.largeur() as i32 {
                assert_eq!(
                    avec.tuile_decor(tx, ty),
                    sans.tuile_decor(tx, ty),
                    "niveau {n} : tuile ({tx}, {ty})"
                );
            }
        }
        // Les entités n'implémentent pas `PartialEq` : leur description `Debug`
        // complète (positions, états, minuteries) suffit à les comparer.
        assert_eq!(
            format!("{:?}", avec.entites),
            format!("{:?}", sans.entites),
            "niveau {n} : entités"
        );
        assert_eq!(avec.courants, sans.courants, "niveau {n} : courants");
        assert_eq!(avec.panneaux, sans.panneaux, "niveau {n} : panneaux");
        assert_eq!(avec.depart, sans.depart);
        assert_eq!(avec.arrivee, sans.arrivee);
    }
}

/// Le décor raconte le niveau : cheminées et antennes sur les toits (1, 2 et le
/// marché couvert, sans manche à air : on ne doit pas la confondre avec un
/// auvent), paratonnerres sous l'orage, manches à air au vent, oiseaux au soleil.
#[test]
fn chaque_niveau_a_son_decor() {
    use GenreDecor::*;
    // Rangé par niveau (et non par place dans la table) : insérer un niveau ne
    // décale rien ; un niveau absent de cette liste n'a pas de décor imposé.
    let attendus: &[(InfosNiveau, &[GenreDecor])] = &[
        (*par_cle("toits"), &[Cheminee, Antenne]),
        (*par_cle("gouttieres"), &[Cheminee, Antenne]),
        (*par_cle("marche"), &[Cheminee, Antenne]),
        (*par_cle("orage"), &[Paratonnerre]),
        (*par_cle("clocher"), &[Paratonnerre]),
        (*par_cle("vents"), &[MancheAAir]),
        (*par_cle("mer"), &[MancheAAir]),
        (*par_cle("nuages"), &[Oiseau]),
        // Le bonus : des oiseaux sur les deux seuls nuages solides (départ, arrivée).
        (*par_cle("arc"), &[Oiseau]),
        // L'épilogue : des oiseaux sur le nuage du départ, et le décor du toit du
        // premier niveau à l'arrivée (cheminée, antenne).
        (*par_cle("descente"), &[Oiseau, Cheminee, Antenne]),
    ];
    for &(infos, genres) in attendus {
        assert!(
            niveaux().iter().any(|n| n.nom == infos.nom),
            "{} hors de la table",
            infos.nom
        );
        let niv = infos.charger().unwrap();
        let n = infos.numero;
        assert!(niv.decors.len() >= 3, "niveau {n} : au moins 3 objets");
        for d in &niv.decors {
            assert!(genres.contains(&d.genre), "niveau {n} : {:?}", d.genre);
        }
        for g in genres {
            assert!(
                niv.decors.iter().any(|d| d.genre == *g),
                "niveau {n} : pas de {g:?}"
            );
        }
    }
}

/// Écart horizontal (px) entre la case d'une manche à air et la zone `z` (0 :
/// collée à son bord gauche ou droit ; négatif : elle est au-dessus ou dedans),
/// et la manche est-elle à une hauteur que la zone traverse ?
fn ecart_manche(d: &ObjetDecor, z: pepin_core::aabb::Aabb) -> Option<f32> {
    let t = pepin_core::aabb::TAILLE_TUILE;
    let ecart = (z.x - (d.x + t)).max(d.x - (z.x + z.w));
    (z.y <= d.y && d.y < z.y + z.h).then_some(ecart)
}

/// Chaque manche à air signale un courant ascendant (à 4 tuiles au plus de son
/// bord, à une hauteur que le courant traverse) ou, depuis la carte 5.15, une
/// zone de vent latéral (collée à son entrée : une tuile au plus de son bord).
#[test]
fn chaque_manche_a_air_est_au_bord_d_un_courant_ou_d_un_vent() {
    let t = pepin_core::aabb::TAILLE_TUILE;
    for infos in niveaux() {
        let niv = infos.charger().unwrap();
        for d in niv
            .decors
            .iter()
            .filter(|d| d.genre == GenreDecor::MancheAAir)
        {
            let courant = niv
                .courants
                .iter()
                .any(|c| ecart_manche(d, c.zone).is_some_and(|e| (0.0..=4.0 * t).contains(&e)));
            let vent = niv
                .vents
                .iter()
                .any(|v| ecart_manche(d, v.zone).is_some_and(|e| (0.0..=t).contains(&e)));
            assert!(
                courant || vent,
                "niveau {} : manche à air en {:?} loin de tout courant et de tout vent",
                infos.numero,
                case_decor(d)
            );
        }
    }
}

/// Carte 5.15 : chaque zone de vent latéral du jeu a sa manche à air à son
/// entrée (collée à son bord, à une hauteur où le vent souffle) : on voit où le
/// vent souffle, et dans quel sens (`scene::manche_vers_la_droite`), avant d'y
/// entrer.
#[test]
fn chaque_zone_de_vent_lateral_a_sa_manche_a_air() {
    let t = pepin_core::aabb::TAILLE_TUILE;
    for infos in niveaux() {
        let niv = infos.charger().unwrap();
        for v in &niv.vents {
            assert!(
                niv.decors.iter().any(|d| d.genre == GenreDecor::MancheAAir
                    && ecart_manche(d, v.zone).is_some_and(|e| (0.0..=t).contains(&e))),
                "niveau {} : vent {:?} sans manche à air à son entrée",
                infos.numero,
                v.zone
            );
        }
    }
}

/// Jamais devant un danger ni sur un objet à voir : pendant 2 minutes de jeu
/// (plus qu'un aller-retour de chaque ennemi), aucune entité (ennemi, pointe,
/// colonne d'éclair, plateforme, lanterne, goutte, champignon) ne passe sur la
/// place d'un objet de décor ; aucun panneau, ni le départ, ni l'arrivée non plus.
#[test]
fn aucun_decor_devant_un_danger_ni_un_objet() {
    use pepin_core::aabb::Aabb;
    use pepin_core::entities::{solides_des_entites, Actualisable};
    use pepin_core::physics::{Environnement, JOUEUR_H, JOUEUR_L};
    let t = pepin_core::aabb::TAILLE_TUILE;
    for infos in niveaux() {
        let niv = infos.charger().unwrap();
        let n = infos.numero;
        let places: Vec<(Aabb, &ObjetDecor)> =
            niv.decors.iter().map(|d| (place_decor(d), d)).collect();
        let mut fixes: Vec<(Aabb, String)> = niv
            .panneaux
            .iter()
            .map(|p| (Aabb::new(p.x, p.y, t, t), format!("{:?}", p.genre)))
            .collect();
        fixes.extend(niv.arrivee.map(|a| (a, "arrivée".into())));
        let (dx, dy) = niv.depart;
        fixes.push((Aabb::new(dx, dy, JOUEUR_L, JOUEUR_H), "départ".into()));
        for (place, d) in &places {
            for (b, nom) in &fixes {
                assert!(
                    !place.chevauche(b),
                    "niveau {n} : {:?} en {:?} sur {nom}",
                    d.genre,
                    case_decor(d)
                );
            }
        }
        let mut entites = niv.entites.clone();
        for pas in 0..120 * 60 {
            for e in &entites {
                let b = e.boite();
                for (place, d) in &places {
                    assert!(
                        !place.chevauche(&b),
                        "niveau {n}, pas {pas} : {:?} en {:?} sous {e:?}",
                        d.genre,
                        case_decor(d)
                    );
                }
            }
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
        }
    }
}

// --- Pigeons (carte 4.4) -------------------------------------------------
