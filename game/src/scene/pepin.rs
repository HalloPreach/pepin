//! Pépin à l'écran : son image selon son état (attente, clignotement de protection),
//! la barque où il s'assied et l'eau devant elle, et le fantôme de la course contre
//! le robot. Sa physique est dans `pepin_core::player` : ici, seulement le dessin.

use pepin_core::aabb::TAILLE_TUILE;
use pepin_core::entities::eaux_des_entites;
use pepin_core::level::Tuile;
use pepin_core::palette::*;
use pepin_core::physics::{ENFONCEMENT_BARQUE, HZ};
use pepin_core::player::EtatJoueur;

use super::tuiles::nom_tuile;
use super::{image, rect};
use crate::course::{self, Bord};
use crate::monde::Monde;
use crate::skins::Skin;
use crate::sprites::{Banque, JeuImages};
use crate::texte::{self, Style};

/// Pépin est-il caché à cette image ? Pendant la protection de réapparition
/// (`protection` pas restants, voir `PROTECTION_REAPPARITION_PAS`), il clignote :
/// 3 images dessiné, 3 images caché (10 fois par seconde à 60 Hz). Un sprite
/// absent une image sur deux plutôt qu'un sprite à moitié transparent : la palette
/// n'a que des couleurs pleines. Fonction pure.
pub(crate) fn pepin_cache(protection: u16, pas: u32) -> bool {
    protection > 0 && (pas / 3).is_multiple_of(2)
}

/// Pépin s'occupe après 2 s (120 pas) immobile au sol.
const ATTENTE_PAS: u32 = 2 * HZ;
/// Durée d'un cycle de l'attente (5 s), rejoué tant que Pépin ne bouge pas.
const CYCLE_ATTENTE: u32 = 5 * HZ;
/// Début (dans le cycle) des coups de parapluie sur le sol, et durée d'un « tac ».
const DEBUT_TAPOTE: u32 = 180;
const DUREE_TAC: u32 = 8;
/// Nombre de coups de parapluie par cycle.
const NB_TACS: u32 = 4;

/// Image de l'attente quand Pépin est immobile au sol depuis `immobile` pas (voir
/// `monde::compter_immobile`), ou `None` : il respire simplement (images du repos).
/// Les noms viennent du jeu d'images du skin actif (`images`).
/// Avant 2 s, toujours `None`. Ensuite, un cycle de 5 s qui recommence :
/// - 0 à 8 : il cligne des yeux ;
/// - 60 à 120 : il lève les yeux vers le ciel (1 s), puis cligne de nouveau (120 à 128) ;
/// - 180 à 244 : il tapote le sol de son parapluie, 4 fois (« tac » de 8 pas, puis 8 pas
///   de repos, `pepin_idle0` : le corps ne bouge pas entre deux coups).
///
/// Comme la minuterie repart de zéro au premier mouvement, l'attente s'arrête net dès
/// qu'on appuie sur une touche. Fonction pure.
pub(crate) fn image_attente(images: &JeuImages, immobile: u32) -> Option<&'static str> {
    // `checked_sub` : `None` avant 2 s, et `?` sort aussitôt de la fonction avec ce `None`.
    let t = immobile.checked_sub(ATTENTE_PAS)? % CYCLE_ATTENTE;
    let [cligne, ciel, tapote] = images.attente;
    match t {
        0..8 | 120..128 => Some(cligne),
        60..120 => Some(ciel),
        _ if (DEBUT_TAPOTE..DEBUT_TAPOTE + 2 * DUREE_TAC * NB_TACS).contains(&t) => {
            let coup = (t - DEBUT_TAPOTE) / DUREE_TAC;
            Some(if coup.is_multiple_of(2) {
                tapote
            } else {
                images.repos[0]
            })
        }
        _ => None,
    }
}

/// Dessine Pépin avec les images du skin actif : même logique d'animation pour tous les
/// skins, seuls les noms des images changent (`JeuImages`).
pub(super) fn dessiner_pepin(
    monde: &Monde,
    banque: &Banque,
    skin: Skin,
    alpha: f32,
    cx: i32,
    cy: i32,
) {
    let poser = |nom: &str, x: f32, y: f32, miroir: bool| banque.dessiner(nom, x, y, miroir);
    // Le temps de mort (carte 9.24) : l'animation de la cause, à l'endroit de la mort.
    crate::dessin_mort::dessiner(monde, banque, skin.images(), (cx, cy));
    poser_pepin(monde, skin.images(), alpha, (cx, cy), &poser);
}

/// Le dessin de Pépin, quelles que soient ses images (`images`) et la façon de les
/// poser (`poser` : pleines, ou tramées pour le fantôme de la course, carte 8.7).
fn poser_pepin(
    monde: &Monde,
    images: &JeuImages,
    alpha: f32,
    (cx, cy): (i32, i32),
    poser: &dyn Fn(&str, f32, f32, bool),
) {
    let pas = monde.pas;
    let j = &monde.joueur;
    let miroir = !j.regarde_droite;

    // Temps de mort (carte 9.24) : déjà replacé à la lanterne, Pépin n'y paraît
    // qu'à l'ouverture du rideau (l'animation est dans `dessin_mort`).
    if monde.temps_de_mort && crate::dessin_mort::pepin_cache(monde.reapparition) {
        return;
    }
    // Tout juste réapparu, Pépin est protégé des dangers (1 s) : il clignote.
    if pepin_cache(j.protection_restante(), pas) {
        return;
    }

    let (px, py) = monde.joueur_interpole(alpha);
    let (bx, by) = (px.round() as i32 - cx, py.round() as i32 - cy);

    let (bas, milieu) = (by + j.boite.h as i32, bx + j.boite.w as i32 / 2);
    match j.etat() {
        EtatJoueur::Plane => {
            let nom = image(&images.planer, pas, 8);
            poser(nom, (milieu - 10) as f32, (bas - 20) as f32, miroir);
        }
        // Parapluie retourné (carte 9.2b, BAS tenu), SOUS Pépin depuis la 9.2c :
        // en l'air, Pépin debout dans la coque de la barque (16x16), posé comme
        // la barque (`ENFONCE_BARQUE_RENDU` plus bas) : la coque est à la même
        // place sous les pieds qu'en barque, Pépin s'y assied en touchant
        // l'eau. Au sol (carte 9.2d), il ne court plus : ASSIS dans la coque
        // (le dessin de la barque), la crosse posée sur le sol (pas dedans).
        EtatJoueur::Retourne => {
            let (nom, enfonce) = if j.au_sol {
                (images.barque, 0)
            } else {
                (images.retourne, ENFONCE_BARQUE_RENDU)
            };
            poser(
                nom,
                (milieu - 8) as f32,
                (bas - 16 + enfonce) as f32,
                miroir,
            );
        }
        // Barque (carte 9.2) : le parapluie retourné avec Pépin assis dedans.
        // À MOITIÉ ENFONCÉE (carte 9.2b, AGA : « il flotte trop haut ») : un
        // décalage de RENDU seulement (`ENFONCE_BARQUE_RENDU`, la physique ne
        // bouge pas), puis une bande d'eau DEVANT la coque.
        EtatJoueur::Barque => {
            poser(
                images.barque,
                (milieu - 8) as f32,
                (bas - 16 + ENFONCE_BARQUE_RENDU) as f32,
                miroir,
            );
            dessiner_eau_devant_barque(monde, (milieu, bas), (cx, cy));
        }
        etat => {
            let nom = match etat {
                EtatJoueur::Montee => images.saut,
                EtatJoueur::Chute => images.chute,
                _ if j.vitesse.x.abs() > 8.0 => image(&images.course, pas, 5),
                _ => image_attente(images, monde.immobile)
                    .unwrap_or_else(|| image(&images.repos, pas, 30)),
            };
            poser(nom, (milieu - 8) as f32, (bas - 16) as f32, miroir);
        }
    }
}

/// Course contre le robot (carte 8.7) : le fantôme, Pépin de son monde à lui (`monde`)
/// tramé en damier avec les images de base, et l'étiquette de sa force au-dessus ;
/// hors champ, une flèche au bord de l'écran, de son côté. `(cx, cy)` : la caméra du
/// joueur.
pub(crate) fn dessiner_fantome(
    monde: &Monde,
    banque: &Banque,
    alpha: f32,
    (cx, cy): (i32, i32),
    etiquette: &str,
) {
    let poser = |nom: &str, x: f32, y: f32, miroir: bool| banque.dessiner_trame(nom, x, y, miroir);
    poser_pepin(monde, Skin::Pepin.images(), alpha, (cx, cy), &poser);
    let j = &monde.joueur;
    let (px, py) = monde.joueur_interpole(alpha);
    let (bx, by) = (
        (px.round() as i32 - cx) as f32,
        (py.round() as i32 - cy) as f32,
    );
    match course::fleche_hors_champ(bx, by, j.boite.w, j.boite.h) {
        None => {
            // Au-dessus de la tête (le sprite dépasse de la boîte par le haut).
            let y = by + j.boite.h - 16.0 - ETIQUETTE_FANTOME;
            let style = Style::simple(BLANC_CASSE, 1).avec_ombre(ARDOISE_OMBRE);
            texte::dessiner_centre_sur(etiquette, bx + j.boite.w / 2.0, y, style);
        }
        Some((x, y, bord)) => dessiner_fleche_fantome(x.round() as i32, y.round() as i32, bord),
    }
}

/// Écart (px) entre le haut du sprite du fantôme et le haut de son étiquette.
const ETIQUETTE_FANTOME: f32 = 9.0;

/// La flèche d'un fantôme hors champ : un triangle de 4 rangées, pointe `(x, y)` vers
/// le `bord`, blanc cassé sur une ombre d'ardoise (lisible sur tous les fonds).
fn dessiner_fleche_fantome(x: i32, y: i32, bord: Bord) {
    for (dx, dy, c) in [(1, 1, ARDOISE_OMBRE), (0, 0, BLANC_CASSE)] {
        for k in 0..4 {
            let l = 2 * k + 1;
            match bord {
                Bord::Gauche => rect(x + k + dx, y - k + dy, 1, l, c),
                Bord::Droite => rect(x - 1 - k + dx, y - k + dy, 1, l, c),
                Bord::Haut => rect(x - k + dx, y + k + dy, l, 1, c),
                Bord::Bas => rect(x - k + dx, y - 1 - k + dy, l, 1, c),
            }
        }
    }
}

/// Décalage de RENDU (px) de la barque vers le bas (carte 9.2b, AGA : « il flotte
/// trop haut par rapport à l'eau ») : la coque (rangées 8 à 12 de l'image) passe à
/// moitié sous la surface, et Pépin s'assied plus bas. La physique ne bouge pas
/// (`ENFONCEMENT_BARQUE` reste 3 px) : sauts, robots et niveaux sont inchangés.
const ENFONCE_BARQUE_RENDU: i32 = 3;

/// Pas (1/60 s) d'un cran des rides autour de la barque (carte 9.2b).
const PAS_RIDE: u32 = 10;

/// L'eau DEVANT la barque (carte 9.2b) : de la surface jusqu'au bas de l'image de la
/// barque, sur sa largeur, on redessine l'eau par-dessus la coque, pixel par pixel,
/// avec les couleurs des tuiles d'eau voisines (vague comprise) : la moitié basse
/// de la coque plonge dans l'eau. De part et d'autre, deux petites rides claires
/// (2 px) s'éloignent sur la surface. `(milieu, bas)` : milieu et pieds de Pépin, en
/// pixels d'écran ; `(cx, cy)` : la caméra. Dans le code de Pépin, pas dans
/// `eau.rs` : le rendu de l'eau elle-même reste à la carte 9.6b.
fn dessiner_eau_devant_barque(monde: &Monde, (milieu, bas): (i32, i32), (cx, cy): (i32, i32)) {
    use crate::sprites::{pixel, tuiles};
    let taille = TAILLE_TUILE as i32;
    // La surface : les pieds de la barque sont `ENFONCEMENT_BARQUE` (3 px) dessous.
    let surface = bas - ENFONCEMENT_BARQUE as i32;
    // Bas de l'image de la barque (16 px de haut, posée à `bas - 16 + décalage`).
    let fond = bas + ENFONCE_BARQUE_RENDU;
    // Les étendues du moment (marées comprises, carte 9.3).
    let eaux = eaux_des_entites(&monde.niveau.eaux, &monde.entites);
    let dans_l_eau = |wx: i32, wy: i32| {
        let (x, y) = (wx as f32 + 0.5, wy as f32 + 0.5);
        eaux.iter()
            .any(|e| e.zone.x < x && x < e.zone.droite() && e.zone.y < y && y < e.zone.bas())
    };
    // Couleur de l'eau au pixel du monde `(wx, wy)` : celle de la tuile d'eau qui
    // est là (son dessin, vague comprise), sinon (marée) clair en surface, sombre
    // dessous. `None` : pas d'eau (le quai, ou le creux d'une vague).
    let couleur = |wx: i32, wy: i32| -> Option<Couleur> {
        let (tx, ty) = (wx.div_euclid(taille), wy.div_euclid(taille));
        let t = monde.niveau.tuile_decor(tx, ty);
        if t == Tuile::Eau {
            let nom = nom_tuile(monde, tx, ty, t);
            let (_, dessin) = tuiles::SPRITES.iter().find(|(n, _)| *n == nom)?;
            let ligne = dessin.get(wy.rem_euclid(taille) as usize)?;
            let c = ligne.chars().nth(wx.rem_euclid(taille) as usize)?;
            pixel(c).ok().flatten()
        } else if dans_l_eau(wx, wy) {
            Some(if wy == surface + cy {
                PLUIE
            } else {
                PLUIE_SOMBRE
            })
        } else {
            None
        }
    };
    for x in milieu - 8..milieu + 8 {
        for y in surface..fond {
            if let Some(c) = couleur(x + cx, y + cy) {
                rect(x, y, 1, 1, c);
            }
        }
    }
    // Les rides : deux traits clairs de 2 px, juste au-dessus de la surface, qui
    // s'éloignent de la coque en 3 crans puis repartent de son bord.
    let k = (monde.pas / PAS_RIDE % 3) as i32;
    for x in [milieu - 10 - 2 * k, milieu + 8 + 2 * k] {
        // Seulement au-dessus de l'eau (pas sur le quai).
        if dans_l_eau(x + cx, surface + cy) && dans_l_eau(x + 1 + cx, surface + cy) {
            rect(x, surface - 1, 2, 1, PLUIE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sprites::pepin;

    /// Pendant toute la protection, Pépin clignote (3 images visible, 3 cachées,
    /// jamais à moitié transparent) ; sans protection, il est toujours dessiné.
    #[test]
    fn pepin_clignote_pendant_toute_la_protection() {
        use pepin_core::player::PROTECTION_REAPPARITION_PAS;
        assert!(
            (0..600).all(|pas| !pepin_cache(0, pas)),
            "pas de protection"
        );
        // 1 s de protection à partir d'une image quelconque : la moitié cachée.
        let depart = 1_234;
        let caches: Vec<bool> = (0..PROTECTION_REAPPARITION_PAS)
            .map(|k| pepin_cache(PROTECTION_REAPPARITION_PAS - k, depart + k as u32))
            .collect();
        assert_eq!(caches.iter().filter(|&&c| c).count(), 30);
        // Séries de 3 images exactement (sauf aux deux bouts) : lisible à 60 Hz.
        let series: Vec<usize> = caches.chunk_by(|a, b| a == b).map(<[bool]>::len).collect();
        assert!(
            series[1..series.len() - 1].iter().all(|&n| n == 3),
            "{series:?}"
        );
    }

    // --- Attente de Pépin (carte 4.3) ---

    #[test]
    fn pas_d_attente_avant_2_s_immobile() {
        for immobile in 0..ATTENTE_PAS {
            assert_eq!(
                image_attente(&pepin::IMAGES, immobile),
                None,
                "pas {immobile}"
            );
        }
        // Pile à 2 s, il cligne des yeux.
        assert_eq!(
            image_attente(&pepin::IMAGES, ATTENTE_PAS),
            Some("pepin_cligne")
        );
    }

    /// Un cycle de 5 s montre les trois images d'attente, chacune dans sa plage, et laisse
    /// Pépin respirer (images du repos) plus de la moitié du temps.
    #[test]
    fn un_cycle_d_attente_montre_les_trois_images() {
        let cycle: Vec<Option<&str>> = (0..CYCLE_ATTENTE)
            .map(|t| image_attente(&pepin::IMAGES, ATTENTE_PAS + t))
            .collect();
        let duree = |nom: &str| cycle.iter().filter(|i| **i == Some(nom)).count();
        assert_eq!(duree("pepin_cligne"), 16, "deux clignements de 8 pas");
        assert_eq!(duree("pepin_ciel"), 60, "1 s le nez en l'air");
        assert_eq!(duree("pepin_tapote"), 32, "4 coups de 8 pas");
        let respire = cycle.iter().filter(|i| i.is_none()).count();
        assert!(respire * 2 > cycle.len(), "{respire} pas de respiration");
        // Rien d'autre que l'attente et le repos.
        for nom in cycle.iter().flatten() {
            assert!(
                pepin::ATTENTE.contains(nom) || pepin::REPOS.contains(nom),
                "{nom}"
            );
        }
    }

    /// Les coups de parapluie alternent avec l'image de repos de même silhouette : « tac »
    /// (embout au sol) puis parapluie relevé, quatre fois de suite.
    #[test]
    fn pepin_tapote_quatre_fois() {
        let debut = ATTENTE_PAS + DEBUT_TAPOTE;
        let mut changements = Vec::new();
        let mut precedente = image_attente(&pepin::IMAGES, debut - 1);
        for t in debut..debut + 80 {
            let i = image_attente(&pepin::IMAGES, t);
            if i != precedente {
                changements.push((t - debut, i));
                precedente = i;
            }
        }
        let tac = Some("pepin_tapote");
        let leve = Some("pepin_idle0");
        assert_eq!(
            changements,
            vec![
                (0, tac),
                (8, leve),
                (16, tac),
                (24, leve),
                (32, tac),
                (40, leve),
                (48, tac),
                (56, leve),
                (64, None)
            ]
        );
    }

    /// Le cycle se répète tant que Pépin ne bouge pas (même après des heures).
    #[test]
    fn l_attente_se_repete_tant_que_pepin_ne_bouge_pas() {
        for t in 0..CYCLE_ATTENTE {
            let i = image_attente(&pepin::IMAGES, ATTENTE_PAS + t);
            assert_eq!(
                image_attente(&pepin::IMAGES, ATTENTE_PAS + t + CYCLE_ATTENTE),
                i,
                "t = {t}"
            );
            assert_eq!(
                image_attente(&pepin::IMAGES, ATTENTE_PAS + t + 1000 * CYCLE_ATTENTE),
                i,
                "t = {t}"
            );
        }
    }

    /// Bout à bout avec la vraie minuterie : en plein « nez en l'air », un seul pas avec une
    /// touche et Pépin redevient normal (la minuterie retombe à zéro, plus d'image d'attente).
    #[test]
    fn l_attente_cesse_au_premier_mouvement() {
        use crate::monde::compter_immobile;
        use pepin_core::player::{Direction, Entrees};
        let mut m = Monde::new(0);
        for _ in 0..30 {
            m.avancer(&Entrees::default());
        }
        let mut immobile = 0;
        for _ in 0..ATTENTE_PAS + 90 {
            immobile = compter_immobile(immobile, &m.joueur, &Entrees::default());
        }
        assert_eq!(image_attente(&pepin::IMAGES, immobile), Some("pepin_ciel"));
        let droite = Entrees {
            direction: Direction::Droite,
            saut: false,
            bas: false,
        };
        immobile = compter_immobile(immobile, &m.joueur, &droite);
        assert_eq!(image_attente(&pepin::IMAGES, immobile), None);
    }
}
