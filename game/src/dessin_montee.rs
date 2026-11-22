//! Rendu de la montée de la Pieuvre (carte 9.13c, AGA : « je n'aime pas la montée,
//! les attaques n'ont pas de sens » ; il a choisi « la Pieuvre géante et visible qui
//! grimpe à côté de la tour : chaque attaque part de son corps ») :
//! - la GÉANTE dans son couloir, à droite de la tour : sa tête posée sur la mer, ses
//!   racines dans l'eau, deux tentacules agrippés au flanc de la tour, et son grand
//!   ŒIL dont la pupille suit Pépin ; pendant l'annonce d'une attaque, l'œil se
//!   plisse et rougit ;
//! - chaque attaque est un tentacule dessiné EN ENTIER, du corps jusqu'au bout (un
//!   chapelet de tronçons ronds de plus en plus fins, `dessiner_bras`) : la frappe
//!   visée (levé au-dessus du REPÈRE rouge, puis abattu), le balayage (tiré en
//!   arrière et tendu, puis traîné sur la corniche), la secousse (deux tentacules
//!   qui serrent la tour, l'ombre puis la chute des caisses), l'arrachage (il va
//!   jusqu'à la plateforme, s'enroule, la tire dans la mer) ;
//! - le JETÉ : la tête se cabre, se jette sur la corniche au bout de son cou, reste
//!   sonnée (l'œil en spirale), et, frappée, hurle et retombe ;
//! - le coup final : la tête sonnée sur la vigie plonge, et la clé monte de l'eau.
//!
//! Que des sprites de la Pieuvre (`sprites::pieuvre`), placés au pixel entier :
//! aucune couleur ajoutée, aucun flou.

use pepin_core::aabb::{Aabb, TAILLE_TUILE};
use pepin_core::entities::pieuvre::{
    EtatArrachage, EtatBalai, EtatFrappe, EtatJete, EtatSaisie, EtatSecousse, PhasePieuvre, Pieuvre,
};

use crate::dessin_pieuvre::{dessiner_eau_saisie, la_pieuvre};
use crate::monde::Monde;
use crate::sprites::Banque;

/// Coin haut-gauche (dans la tête) des surcharges de l'œil : la paupière et l'œil
/// sonné (`paupiere_geante`, `oeil_sonne`).
const OEIL: (f32, f32) = (10.0, 10.0);
/// Centre du blanc de l'œil dans la tête.
const CENTRE_OEIL: (f32, f32) = (19.0, 17.0);
/// Jusqu'où la pupille s'écarte du centre de l'œil (px, de côté et en hauteur).
const REGARD: (f32, f32) = (4.0, 2.0);

/// Image d'une animation à deux images de `duree` pas chacune.
fn image(anim: [&'static str; 2], pas: u32, duree: u32) -> &'static str {
    anim[(pas / duree) as usize % 2]
}

/// Un point (px) du monde.
type Point = (f32, f32);

/// Le point de la courbe de `a` à `b` tirée vers `c` (courbe de Bézier du second
/// degré), à `k` (de 0 à 1). Pure.
fn sur_la_courbe(a: Point, c: Point, b: Point, k: f32) -> Point {
    let u = 1.0 - k;
    (
        u * u * a.0 + 2.0 * u * k * c.0 + k * k * b.0,
        u * u * a.1 + 2.0 * u * k * c.1 + k * k * b.1,
    )
}

/// Le milieu d'une boîte.
fn milieu(b: &Aabb) -> Point {
    (b.x + b.w / 2.0, b.y + b.h / 2.0)
}

/// Un tentacule de la géante, EN ENTIER : un chapelet de tronçons ronds de son
/// pied `a` (sur le corps) jusqu'à son bout `b`, le long d'une courbe tirée vers
/// `c` ; gros près du corps, de plus en plus fins vers le bout. On pose du bout
/// vers le corps : les gros tronçons couvrent les fins. Il ondule un peu (`pas`).
fn dessiner_bras(banque: &Banque, a: Point, c: Point, b: Point, pas: u32, (cx, cy): Point) {
    let longueur = (b.0 - a.0).hypot(b.1 - a.1) + (c.0 - a.0).hypot(c.1 - a.1) * 0.5;
    let n = ((longueur / 4.0).ceil() as u32).max(4);
    for i in (0..=n).rev() {
        let k = i as f32 / n as f32;
        let (x, y) = sur_la_courbe(a, c, b, k);
        // L'ondulation : nulle au pied et au bout, un pixel au milieu.
        let onde = ((pas as f32 * 0.12 + k * 7.0).sin() * 1.4 * (k * (1.0 - k) * 4.0)).round();
        let (nom, r) = if k < 0.35 {
            ("bras_gros", 5.0)
        } else if k < 0.7 {
            ("bras_moyen", 4.0)
        } else if k < 0.93 {
            ("bras_fin", 3.0)
        } else {
            ("bras_pointe", 2.0)
        };
        let (sx, sy) = ((x - r - cx).round(), (y - r + onde - cy).round());
        banque.dessiner(nom, sx, sy, false);
    }
}

/// Les cases arrachables qui ne se dessinent plus comme des tuiles (le jeu de
/// tuiles les saute : `dessiner` les dessine, tremblantes ou emportées).
pub fn case_arrachee(p: &Pieuvre, tx: i32, ty: i32) -> bool {
    (0..p.tour.arrachables.len())
        .any(|k| p.etat_arrachage(k).is_some() && p.tour.arrachables[k].contains(&(tx, ty)))
}

/// Tout ce que la montée ajoute au dessin de la pieuvre du niveau (caméra `cx`,
/// `cy`), devant les tuiles et derrière Pépin.
pub fn dessiner(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    let Some(p) = la_pieuvre(monde).filter(|p| p.en_fuite()) else {
        return;
    };
    let camera = (cx as f32, cy as f32);
    if matches!(p.phase(), PhasePieuvre::Plongee | PhasePieuvre::Vaincue) {
        dessiner_plongee(p, banque, camera);
        return;
    }
    let Some(tete) = p.geante() else {
        return;
    };
    let pas = monde.pas;
    dessiner_arrachages(monde, p, banque, &tete, camera);
    dessiner_secousse(p, banque, &tete, pas, camera);
    dessiner_geante(monde, p, banque, &tete, camera);
    dessiner_balayages(p, banque, &tete, pas, camera);
    dessiner_frappe(p, banque, &tete, pas, camera);
}

/// La géante : le cou (quand la tête s'est jetée sur une corniche), les deux
/// tentacules agrippés à la tour, la tête, l'œil qui suit Pépin, la bouche qui
/// hurle.
fn dessiner_geante(monde: &Monde, p: &Pieuvre, banque: &Banque, tete: &Aabb, camera: Point) {
    let (cx, cy) = camera;
    let pas = monde.pas;
    let Some(mer) = p.surface_mer() else {
        return;
    };
    let (g, d) = p.tour.couloir.unwrap_or((tete.x, tete.droite()));
    let jete = p.etat_jete();
    let lachee = p.tete_lachee();
    // Le cou : de son corps, dans la mer au milieu du couloir, jusque sous la
    // tête jetée (un chapelet de gros tronçons).
    let pied = ((g + d) / 2.0, mer + 4.0);
    let sous_la_tete = (tete.x + tete.w * 0.7, tete.bas() - 6.0);
    if jete.is_some() || lachee.is_some() {
        let coude = (d - 8.0, (pied.1 + sous_la_tete.1) / 2.0);
        let n = (((pied.1 - sous_la_tete.1).abs() + (pied.0 - sous_la_tete.0).abs()) / 4.0)
            .ceil()
            .max(2.0) as u32;
        for i in 0..=n {
            let (x, y) = sur_la_courbe(pied, coude, sous_la_tete, i as f32 / n as f32);
            if y < mer {
                banque.dessiner(
                    "bras_gros",
                    (x - 5.0 - cx).round(),
                    (y - 5.0 - cy).round(),
                    false,
                );
            }
        }
    }
    // Les deux tentacules agrippés au flanc de la tour (au repos, elle grimpe) :
    // de sa racine gauche jusqu'au bord de la tour, plus haut.
    if jete.is_none() && lachee.is_none() {
        let tremble = if p.etat_secousse().is_some() {
            [0.0, 1.0, 0.0, -1.0][(pas / 2 % 4) as usize]
        } else {
            0.0
        };
        for (h, depart) in [(28.0, 30.0), (60.0, 22.0)] {
            let a = (tete.x + 8.0, tete.y + depart);
            let b = (g + 1.0 + tremble, tete.y - h + 30.0);
            let c = (g - 10.0, (a.1 + b.1) / 2.0 + 6.0);
            dessiner_bras(banque, a, c, b, pas, camera);
        }
    }
    // La tête (coupée à la surface quand elle remonte de l'eau).
    let nom = image(["geante0", "geante1"], pas, 16);
    let (x, y) = ((tete.x - cx).round(), (tete.y - cy).round());
    let visible = (mer - tete.y).clamp(0.0, tete.h);
    if p.est_facile() && visible >= tete.h {
        banque.dessiner_pale(nom, x, y);
    } else {
        banque.dessiner_haut(nom, x, y, visible);
    }
    if visible < CENTRE_OEIL.1 + 6.0 {
        return;
    }
    let (ox, oy) = (x + OEIL.0, y + OEIL.1);
    let sonnee = matches!(jete, Some((_, EtatJete::Sonnee(_))));
    if sonnee || lachee.is_some() {
        banque.dessiner("oeil_sonne", ox, oy, false);
    } else {
        // La pupille regarde Pépin.
        let j = milieu(&monde.joueur.boite);
        let (ex, ey) = (tete.x + CENTRE_OEIL.0, tete.y + CENTRE_OEIL.1);
        let (dx, dy) = (j.0 - ex, j.1 - ey);
        let n = dx.hypot(dy).max(1.0);
        let (px, py) = ((dx / n * REGARD.0).round(), (dy / n * REGARD.1).round());
        if p.annonce_en_cours() {
            // L'annonce : la pupille rougit, la paupière plisse l'œil.
            banque.dessiner(
                "pupille_rouge",
                x + CENTRE_OEIL.0 - 4.0 + px,
                y + CENTRE_OEIL.1 - 4.0 + py,
                false,
            );
            banque.dessiner("paupiere_geante", ox, oy, false);
        } else {
            banque.dessiner(
                "pupille_geante",
                x + CENTRE_OEIL.0 - 3.0 + px,
                y + CENTRE_OEIL.1 - 3.0 + py,
                false,
            );
        }
    }
    // Elle hurle : en se jetant, et quand on la frappe.
    if lachee.is_some() || matches!(jete, Some((_, EtatJete::Chute(_)))) {
        banque.dessiner("bouche_geante", x + 24.0, y + 22.0, false);
    }
}

/// La frappe visée : le REPÈRE rouge sur la place visée, et le tentacule levé
/// au-dessus d'elle (l'annonce), abattu dessus (la frappe), puis retiré.
fn dessiner_frappe(p: &Pieuvre, banque: &Banque, tete: &Aabb, pas: u32, camera: Point) {
    let Some((cible, etat)) = p.etat_frappe() else {
        return;
    };
    let (cx, cy) = camera;
    let epaule = (tete.x + 14.0, tete.y + 6.0);
    let (vx, sol) = (cible.x + cible.w / 2.0, cible.bas());
    let leve = (vx, cible.y - 18.0);
    let abattu = (vx, sol - 6.0);
    let bout = match etat {
        EtatFrappe::Annonce(a) => (leve.0, leve.1 - 10.0 * a),
        EtatFrappe::Frappe => abattu,
        EtatFrappe::Retrait(r) => (
            abattu.0 + (epaule.0 - abattu.0) * r * 0.5,
            abattu.1 + (leve.1 - abattu.1) * r,
        ),
    };
    if matches!(etat, EtatFrappe::Annonce(_)) {
        let nom = image(["repere_frappe0", "repere_frappe1"], pas, 6);
        banque.dessiner(
            nom,
            (vx - 8.0 - cx).round(),
            (sol - 16.0 - cy).round(),
            false,
        );
    }
    // La courbe passe au-dessus de la tour, entre l'épaule et la place visée.
    let haut = epaule.1.min(bout.1) - 36.0;
    let coude = ((epaule.0 + bout.0) / 2.0 + 10.0, haut);
    dessiner_bras(banque, epaule, coude, bout, pas, camera);
}

/// Les balayages : le tentacule part du bas du corps ; à l'annonce, son bout est
/// tiré en arrière contre le couloir et le tentacule se tend ; puis il est traîné
/// sur la corniche, de droite à gauche, son bout au ras du sol.
fn dessiner_balayages(p: &Pieuvre, banque: &Banque, tete: &Aabb, pas: u32, camera: Point) {
    for (k, b) in p.tour.balayeurs.iter().enumerate() {
        let Some(etat) = p.etat_balayeur(k) else {
            continue;
        };
        let sol = b.y - 5.0;
        let bout = match etat {
            EtatBalai::Annonce(a) => (b.depart - 8.0 + 10.0 * a, sol),
            EtatBalai::Balaie(bout) => (bout.x + 4.0, sol),
            EtatBalai::Retrait(r) => {
                let fin = b.bout(b.course_pas()).x + 4.0;
                (fin + (b.depart - fin) * r, sol)
            }
        };
        let pied = (tete.x + 10.0, tete.y + 30.0);
        // Tendu à l'annonce (un coude bas, qui recule), couché derrière son bout
        // quand il balaie.
        let coude = match etat {
            EtatBalai::Annonce(a) => (b.depart + 14.0 * a, sol + 6.0 * a),
            _ => (bout.0 + 28.0, sol - 2.0),
        };
        dessiner_bras(banque, pied, coude, bout, pas, camera);
    }
}

/// La secousse : deux tentacules serrent la tour (dessinés par `dessiner_geante`,
/// qui tremblent), l'ombre des caisses sur leur sol pendant l'annonce, puis les
/// caisses qui tombent et se brisent.
fn dessiner_secousse(p: &Pieuvre, banque: &Banque, tete: &Aabb, pas: u32, camera: Point) {
    let Some((caisses, etat)) = p.etat_secousse() else {
        return;
    };
    let (cx, cy) = camera;
    // Deux tentacules de plus, qui serrent la tour plus haut, là où les caisses
    // vont partir.
    let g = p.tour.couloir.map_or(tete.x, |c| c.0);
    let tremble = [0.0, 1.0, 0.0, -1.0][(pas / 2 % 4) as usize];
    for (h, depart) in [(90.0, 12.0), (124.0, 4.0)] {
        let a = (tete.x + 12.0, tete.y + depart);
        let b = (g - 2.0 + tremble, tete.y - h + 30.0);
        let c = (g + 8.0, (a.1 + b.1) / 2.0);
        dessiner_bras(banque, a, c, b, pas, camera);
    }
    for c in caisses {
        let (x, sol) = ((c.x - cx).round(), (c.sol - cy).round());
        match etat {
            EtatSecousse::Annonce(_) => banque.dessiner("ombre_caisse", x, sol - 3.0, false),
            EtatSecousse::Chute(s) => match c.boite(s) {
                Some(b) => {
                    banque.dessiner("ombre_caisse", x, sol - 3.0, false);
                    banque.dessiner("caisse_chute", x, (b.y - cy).round(), false);
                }
                None if s <= c.chute_pas() + 10 => {
                    banque.dessiner("eclats_caisse", x, sol - 6.0, false)
                }
                None => {}
            },
        }
    }
}

/// Les arrachages : le tentacule va du corps jusqu'à la plateforme (1re moitié de
/// l'annonce), s'y enroule (elle tremble), puis la tire vers le couloir et dans
/// la mer, découpée à la surface (l'eau de la saisie, carte 9.12g).
fn dessiner_arrachages(monde: &Monde, p: &Pieuvre, banque: &Banque, tete: &Aabb, camera: Point) {
    let (cx, cy) = camera;
    let pas = monde.pas;
    let mer = p.surface_mer();
    let enroule = image(["tentacule_enroule0", "tentacule_enroule1"], pas, 8);
    let couloir = p.tour.couloir.map_or(0.0, |c| c.0);
    for (k, cases) in p.tour.arrachables.iter().enumerate() {
        let Some(etat) = p.etat_arrachage(k) else {
            continue;
        };
        let t = TAILLE_TUILE;
        let haut = cases.iter().map(|c| c.1).min().unwrap_or(0) as f32 * t;
        let g = cases.iter().map(|c| c.0).min().unwrap_or(0) as f32 * t;
        let d = (cases.iter().map(|c| c.0).max().unwrap_or(0) + 1) as f32 * t;
        let (dx, dy, saisie, va) = match etat {
            EtatArrachage::Annonce(a) => {
                let ampleur = if a < 0.5 { 0.0 } else { 1.0 };
                let dx = [0.0, ampleur, 0.0, -ampleur][(pas / 2 % 4) as usize];
                (dx, 0.0, EtatSaisie::Annonce(a), (a * 2.0).min(1.0))
            }
            // Emportée : tirée vers le couloir et dans la mer en 0,5 s.
            EtatArrachage::Emportee(e) => {
                let fond = mer.map_or(5.0 * t, |m| m - haut);
                let vers = ((couloir - d) * e * e).max(0.0).round();
                (vers, (fond * e * e).round(), EtatSaisie::Plongee(e), 1.0)
            }
            EtatArrachage::Partie => continue,
        };
        if let Some(mer) = mer {
            dessiner_eau_saisie(
                banque,
                saisie,
                (g + dx, d + dx),
                (t, couloir),
                (-cx, mer - cy),
                pas,
            );
        }
        let reste = mer.map_or(t, |m| m - haut - dy).max(0.0);
        for &(tx, ty) in cases {
            let (nom, _) = crate::scene::noms_de_la_case(monde, tx, ty);
            let (x, y) = (tx as f32 * t + dx, ty as f32 * t + dy);
            banque.dessiner_tuile_haut(monde.infos.theme, nom, x - cx, y - cy, reste.min(t));
        }
        // Le tentacule, du corps jusqu'au bout droit de la plateforme…
        let pied = (tete.x + 8.0, tete.y + 18.0);
        let prise = (d + dx - 4.0, haut + dy + 2.0);
        let bout = (
            pied.0 + (prise.0 - pied.0) * va,
            pied.1 + (prise.1 - pied.1) * va,
        );
        let coude = ((pied.0 + bout.0) / 2.0, pied.1.min(bout.1) - 16.0);
        if reste > 0.0 {
            dessiner_bras(banque, pied, coude, bout, pas, camera);
        }
        // … et enroulé sur le dessus (à la 2e moitié de l'annonce).
        if !matches!(etat, EtatArrachage::Annonce(a) if a < 0.5) {
            let mut x = g;
            while x < d {
                banque.dessiner_haut(enroule, x + dx - cx, haut - 3.0 + dy - cy, reste.min(6.0));
                x += t;
            }
        }
    }
}

/// Le coup final : la tête sonnée de la géante lâche la vigie et retombe dans le
/// couloir, vers la mer ; la clé sort de la vigie et monte.
fn dessiner_plongee(p: &Pieuvre, banque: &Banque, (cx, cy): Point) {
    let Some(tete) = p.tete_finale else {
        return;
    };
    let k = p.progression_plongee();
    let couloir = p
        .tour
        .couloir
        .map_or(tete.x, |c| (c.0 + c.1 - tete.w) / 2.0);
    let chute = k * k;
    let (x, y) = (
        (tete.x + (couloir - tete.x) * chute - cx).round(),
        (tete.y + 200.0 * chute - cy).round(),
    );
    banque.dessiner("geante0", x, y, false);
    banque.dessiner("oeil_sonne", x + OEIL.0, y + OEIL.1, false);
    banque.dessiner("bouche_geante", x + 24.0, y + 22.0, false);
    // La clé monte de 40 px au-dessus de la vigie.
    let montee = (40.0 * (k * 2.0).min(1.0)).round();
    let (kx, ky) = (
        (tete.x + 25.0 - cx).round(),
        (tete.bas() - 8.0 - montee - cy).round(),
    );
    banque.dessiner("cle_pieuvre", kx, ky, false);
}
