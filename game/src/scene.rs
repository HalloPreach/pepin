//! Scène : dessine un `Monde` dans la cible 320x180, couche par couche.
//!
//! Ordre (arrière -> avant) : fond (ciel, silhouettes) -> tuiles -> décor -> panneaux ->
//! cadrans des aiguilles d'horloge -> plateformes -> objets -> ennemis -> Pépin ->
//! particules -> pluie.
//! L'interface (compteurs, chrono) est dessinée par `hud`, les menus par `ecrans`.
//! Chaque sprite est posé à une position ENTIÈRE (positions interpolées puis
//! arrondies) : aucun sous-pixel.

use macroquad::prelude::draw_rectangle;
use pepin_core::aabb::{Aabb, Grille, TAILLE_TUILE};
use pepin_core::entities::auvent::HAUT_TOILE;
use pepin_core::entities::ennemi::{EtatEnnemi, PhaseChat, PhasePigeon};
use pepin_core::entities::friable::EtatFriable;
use pepin_core::entities::montgolfiere::{EtatMontgolfiere, HAUT_BALLON};
use pepin_core::entities::{Entite, TypeEnnemi};
use pepin_core::level::{GenreDecor, GenrePanneau, Tuile};
use pepin_core::palette::*;
use pepin_core::physics::{CourantAscendant, HZ};
use pepin_core::player::EtatJoueur;
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use crate::decor;
use crate::grele;
use crate::monde::{Monde, REAPPARITION_PAS};
use crate::particules::Particules;
use crate::rendu::couleur_mq;
use crate::sprites::{ennemis, hauteur, largeur, objets, objets_decor, pepin, Banque};

fn rect(x: i32, y: i32, w: i32, h: i32, c: Couleur) {
    draw_rectangle(x as f32, y as f32, w as f32, h as f32, couleur_mq(c));
}

/// Image d'une animation : `pas` / `duree` modulo le nombre d'images.
fn image<'a>(anim: &'a [&'a str], pas: u32, duree: u32) -> &'a str {
    anim[(pas / duree) as usize % anim.len()]
}

/// Arrondit une boîte interpolée au pixel : `(x, y)` entiers.
fn entier(b: &Aabb) -> (i32, i32) {
    (b.x.round() as i32, b.y.round() as i32)
}

/// Noms des variantes de brique pleine (`sprites/tuiles.rs`), dans l'ordre du tirage.
const VARIANTES_BRIQUE: [&str; 3] = ["brique_fissure", "brique_manquante", "brique_coulure"];

/// En moyenne, une brique pleine sur `UNE_BRIQUE_SUR` tire une variante (un peu moins au final :
/// on en retire quelques-unes pour ne jamais coller deux variantes identiques).
const UNE_BRIQUE_SUR: u32 = 6;

/// Hachage entier FIXE d'une case : mélange les bits de `tx` et `ty` (multiplications par de
/// grands nombres impairs, puis « xor-décalages », à la manière de murmur3). Le même `(tx, ty)`
/// donne toujours le même nombre : aucun aléatoire, aucun état, donc le mur est identique d'une
/// partie à l'autre et d'une image à l'autre. `wrapping_mul` : on veut le débordement modulo 2³²
/// (en Rust, `*` panique en cas de débordement quand les vérifications sont actives, comme en
/// mode debug et dans les tests).
fn hachage_case(tx: i32, ty: i32) -> u32 {
    let mut h = (tx as u32).wrapping_mul(0x9E37_79B1) ^ (ty as u32).wrapping_mul(0x85EB_CA77);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB_352D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846C_A68B);
    h ^= h >> 16;
    h
}

/// Tirage brut d'une case : `Some(indice de variante)` environ une fois sur `UNE_BRIQUE_SUR`
/// (le reste de `h` divisé par 6 vaut 0 une fois sur 6 ; le quotient choisit le dessin).
fn tirage_brique(tx: i32, ty: i32) -> Option<usize> {
    let h = hachage_case(tx, ty);
    h.is_multiple_of(UNE_BRIQUE_SUR)
        .then_some(((h / UNE_BRIQUE_SUR) % VARIANTES_BRIQUE.len() as u32) as usize)
}

/// Variante de brique pleine à dessiner en `(tx, ty)`, ou `None` pour la brique ordinaire.
/// Fonction PURE de la position. Une case renonce à sa variante si la case de GAUCHE ou celle du
/// DESSUS a tiré la même : comme chaque case regarde ces deux voisines-là, deux cases voisines
/// (côte à côte ou l'une sur l'autre) ne montrent jamais la même variante.
fn variante_brique(tx: i32, ty: i32) -> Option<&'static str> {
    let v = tirage_brique(tx, ty)?;
    if tirage_brique(tx - 1, ty) == Some(v) || tirage_brique(tx, ty - 1) == Some(v) {
        return None;
    }
    Some(VARIANTES_BRIQUE[v])
}

/// La bande d'ombre du rebord se pose sur une brique dont la case du dessus est un toit : c'est
/// la première tuile pleine sous le toit (dans les niveaux, un toit a une seule tuile
/// d'épaisseur). Sous une cheminée posée sur le toit, la règle reste la même : la bande court
/// sans interruption tout le long du rebord.
fn ombre_sous_toit(t: Tuile, dessus: Tuile) -> bool {
    t == Tuile::Brique && dessus == Tuile::Toit
}

/// Nom de la tuile à dessiner : version « haut » si rien de solide au-dessus ; un nuage dont la
/// case voisine est vide (à gauche et/ou à droite) prend un bord arrondi. Dessin seulement :
/// la collision reste celle de la case.
fn nom_tuile(monde: &Monde, tx: i32, ty: i32, t: Tuile) -> &'static str {
    let haut = monde.niveau.tuile_decor(tx, ty - 1) == Tuile::Vide;
    if t == Tuile::Nuage && haut {
        let (gauche, droite) = (
            monde.niveau.tuile_decor(tx - 1, ty) == Tuile::Vide,
            monde.niveau.tuile_decor(tx + 1, ty) == Tuile::Vide,
        );
        match (gauche, droite) {
            (true, true) => return "nuage_ilot",
            (true, false) => return "nuage_bord_g",
            (false, true) => return "nuage_bord_d",
            (false, false) => {}
        }
    }
    match (t, haut) {
        (Tuile::Toit, true) => "toit_haut",
        (Tuile::Toit, false) => "toit_plein",
        (Tuile::Brique, true) => "brique_haut",
        (Tuile::Brique, false) => variante_brique(tx, ty).unwrap_or("brique_plein"),
        (Tuile::Nuage, true) => "nuage_haut",
        (Tuile::Nuage, false) => "nuage_plein",
        (Tuile::Vide, _) => "",
    }
}

fn dessiner_tuiles(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    let taille = TAILLE_TUILE as i32;
    let (t0, t1) = (
        cx.div_euclid(taille),
        (cx + LARGEUR_LOGIQUE as i32).div_euclid(taille),
    );
    let (u0, u1) = (
        cy.div_euclid(taille),
        (cy + HAUTEUR_LOGIQUE as i32).div_euclid(taille),
    );
    for ty in u0..=u1 {
        for tx in t0..=t1 {
            // Hors carte : `tuile_decor` renvoie des murs à gauche/droite ; on ne les dessine pas.
            if tx < 0 || tx >= monde.niveau.largeur() as i32 {
                continue;
            }
            let t = monde.niveau.tuile_decor(tx, ty);
            if t != Tuile::Vide {
                let nom = nom_tuile(monde, tx, ty, t);
                let (x, y) = ((tx * taille - cx) as f32, (ty * taille - cy) as f32);
                banque.dessiner_tuile(monde.infos.theme, nom, x, y);
                // Bande d'ombre du rebord, par-dessus le haut de la brique.
                if ombre_sous_toit(t, monde.niveau.tuile_decor(tx, ty - 1)) {
                    banque.dessiner_tuile(monde.infos.theme, "ombre_toit", x, y);
                }
            }
        }
    }
}

fn dessiner_panneaux(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    for p in &monde.niveau.panneaux {
        let nom = match p.genre {
            GenrePanneau::Deplacer => "panneau_deplacer",
            GenrePanneau::Sauter => "panneau_sauter",
            GenrePanneau::Planer => "panneau_planer",
            GenrePanneau::Champignon => "panneau_champignon",
            GenrePanneau::Vent => "panneau_vent",
        };
        banque.dessiner(nom, p.x.round() - cx as f32, p.y.round() - cy as f32, false);
    }
}

/// Durée (pas) d'un cycle de l'oiseau posé, et durée de la tête tournée en fin de cycle.
const CYCLE_OISEAU: u32 = 300;
const TETE_TOURNEE: u32 = 40;

/// Image de l'oiseau posé n° `rang` au pas `pas` : il regarde devant, puis tourne la tête
/// 2/3 s toutes les 5 s. Chaque oiseau a son décalage (97 pas de plus que le précédent) :
/// ils ne tournent jamais la tête tous ensemble. Fonction pure, testable.
fn image_oiseau(pas: u32, rang: usize) -> &'static str {
    let phase = (pas + 97 * rang as u32) % CYCLE_OISEAU;
    objets_decor::OISEAU[usize::from(phase >= CYCLE_OISEAU - TETE_TOURNEE)]
}

/// La manche à air s'étire vers le courant ascendant le plus proche (l'air y est aspiré) :
/// vers la droite si son centre est à droite de la manche (ou s'il n'y a aucun courant).
fn manche_vers_la_droite(x: f32, courants: &[CourantAscendant]) -> bool {
    let centre_manche = x + TAILLE_TUILE / 2.0;
    courants
        .iter()
        .map(|c| c.zone.x + c.zone.w / 2.0 - centre_manche)
        .min_by(|a, b| a.abs().total_cmp(&b.abs()))
        .is_none_or(|ecart| ecart >= 0.0)
}

/// Coin haut-gauche (au pixel) d'un sprite de décor de `largeur` x `hauteur` posé sur sa
/// case `(x, y)` : centré dans la largeur de la case, le bas sur le bas de la case (donc
/// sur la tuile solide du dessous).
fn coin_decor(x: f32, y: f32, largeur: usize, hauteur: usize) -> (i32, i32) {
    let t = TAILLE_TUILE as i32;
    (
        x.round() as i32 + (t - largeur as i32) / 2,
        y.round() as i32 + t - hauteur as i32,
    )
}

/// Objets de décor : derrière les panneaux, les objets, les ennemis et Pépin (il passe
/// devant une cheminée), sans aucune collision.
fn dessiner_decors(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    let mut oiseaux = 0;
    for d in &monde.niveau.decors {
        let (nom, miroir) = match d.genre {
            GenreDecor::Cheminee => ("cheminee", false),
            GenreDecor::Antenne => ("antenne", false),
            GenreDecor::Paratonnerre => ("paratonnerre", false),
            GenreDecor::MancheAAir => (
                "manche_a_air",
                !manche_vers_la_droite(d.x, &monde.niveau.courants),
            ),
            GenreDecor::Oiseau => {
                oiseaux += 1;
                (image_oiseau(monde.pas, oiseaux - 1), false)
            }
        };
        let Some(dessin) = objets_decor::SPRITES
            .iter()
            .find(|(n, _)| *n == nom)
            .map(|&(_, dessin)| dessin)
        else {
            continue;
        };
        let (x, y) = coin_decor(d.x, d.y, largeur(dessin), hauteur(dessin));
        banque.dessiner(nom, (x - cx) as f32, (y - cy) as f32, miroir);
    }
}

/// Annonce de l'éclair : le pointillé est-il allumé à cet instant ? `progression` va de 0.0
/// (début de l'annonce) à 1.0 (frappe imminente) : le clignotement ACCÉLÈRE, de 2,5 Hz au début
/// à 15 Hz à la fin (7 clignotements en 0,8 s). Fonction pure, testable.
fn annonce_allumee(progression: f32) -> bool {
    let p = progression.clamp(0.0, 1.0);
    let phase = 2.0 * p + 5.0 * p * p; // nombre de cycles écoulés
    phase.fract() < 0.5
}

/// Colonne de l'éclair à l'écran : `(y, hauteur)`. `zone_y`, `hauteur` : la zone dans le niveau ;
/// `cy` : ordonnée de la caméra. Quand la zone part du haut du niveau (y = 0) et que le niveau
/// est plus bas que l'écran, quelques lignes de ciel resteraient au-dessus : la colonne est alors
/// prolongée jusqu'au haut de l'écran. Dessin seulement : la zone dangereuse ne change pas.
fn colonne_eclair(zone_y: i32, hauteur: i32, cy: i32) -> (i32, i32) {
    let y0 = zone_y - cy;
    if zone_y <= 0 && y0 > 0 {
        (0, hauteur + y0)
    } else {
        (y0, hauteur)
    }
}

/// Éclair : une colonne annoncée par un pointillé clignotant sur ses deux bords et une barre
/// dorée à son pied, puis un zigzag éblouissant. `progression` : avancement de l'annonce (0 à 1).
fn dessiner_eclair(zone: &Aabb, annonce: bool, progression: f32, frappe: bool, cx: i32, cy: i32) {
    let x0 = zone.x.round() as i32 - cx;
    let (y0, h) = colonne_eclair(zone.y.round() as i32, zone.h.round() as i32, cy);
    let w = zone.w.round() as i32;
    let milieu = x0 + w / 2;
    if annonce {
        // Le pied de la colonne s'allume tout de suite et reste allumé : on voit OÙ ça va tomber.
        rect(x0, y0 + h - 2, w, 2, OR);
        if annonce_allumee(progression) {
            for y in (y0..y0 + h - 2).step_by(4) {
                rect(x0, y, 1, 2, OR);
                rect(x0 + w - 1, y, 1, 2, OR);
            }
        }
    }
    if frappe {
        rect(x0, y0, w, h, OR_PALE);
        const ZIGZAG: [i32; 6] = [0, 2, -1, 2, -2, 1];
        for (k, y) in (y0..y0 + h).step_by(4).enumerate() {
            let dx = ZIGZAG[k % ZIGZAG.len()];
            rect(milieu - 1 + dx, y, 3, 4, BLANC_CASSE);
        }
    }
}

/// Écart (px) entre deux maillons de la tige d'une aiguille d'horloge.
const PAS_MAILLON: f32 = 6.0;

/// Coins haut-gauche (entiers) des maillons 4x4 de la tige d'une aiguille, du
/// pivot `pivot` jusqu'au centre `bout` de sa plaque : un tous les `PAS_MAILLON`
/// px, le premier à 6 px du pivot (le moyeu du cadran est au centre), le dernier
/// au moins 4 px avant le bout (caché sous la plaque sinon). La tige tourne avec
/// la plaque comme l'aiguille d'une montre : seules les POSITIONS changent, aucun
/// sprite n'est tourné. Fonction pure.
fn maillons(pivot: (f32, f32), bout: (f32, f32)) -> Vec<(i32, i32)> {
    let (dx, dy) = (bout.0 - pivot.0, bout.1 - pivot.1);
    let longueur = (dx * dx + dy * dy).sqrt();
    if longueur < 1.0 {
        return Vec::new();
    }
    let n = ((longueur - 4.0) / PAS_MAILLON).floor().max(0.0) as usize;
    (1..=n)
        .map(|k| {
            let t = k as f32 * PAS_MAILLON / longueur;
            let (x, y) = (pivot.0 + dx * t, pivot.1 + dy * t);
            ((x - 2.0).round() as i32, (y - 2.0).round() as i32)
        })
        .collect()
}

/// Les cadrans des aiguilles d'horloge (32x32, centrés sur leur pivot) et la tige
/// de maillons qui va du moyeu au milieu de la plaque. Dessinés AVANT les entités :
/// la plaque, Pépin, les gouttes passent devant.
fn dessiner_cadrans(monde: &Monde, banque: &Banque, alpha: f32, cx: i32, cy: i32) {
    for (i, entite) in monde.entites.iter().enumerate() {
        let Entite::Mobile(m) = entite else {
            continue;
        };
        let Some(cercle) = m.cercle() else {
            continue;
        };
        let (px, py) = (cercle.pivot.0.round() as i32, cercle.pivot.1.round() as i32);
        banque.dessiner(
            "cadran",
            (px - 16 - cx) as f32,
            (py - 16 - cy) as f32,
            false,
        );
        // Bout de la tige : le milieu de la plaque, à sa position DESSINÉE (arrondie).
        let (x, y) = entier(&monde.boite_interpolee(i, alpha));
        let bout = (x as f32 + m.boite.w / 2.0, y as f32 + m.boite.h / 2.0);
        for (mx, my) in maillons(cercle.pivot, bout) {
            banque.dessiner("maillon", (mx - cx) as f32, (my - cy) as f32, false);
        }
    }
}

fn dessiner_entites(monde: &Monde, banque: &Banque, alpha: f32, cx: i32, cy: i32) {
    let pas = monde.pas;
    // Parapluie ouvert de Pépin (la grêle se brise dessus), à sa position dessinée.
    let parapluie = (monde.joueur.etat() == EtatJoueur::Plane).then(|| {
        let (px, py) = monde.joueur_interpole(alpha);
        grele::parapluie(px.round(), py.round())
    });
    for (i, entite) in monde.entites.iter().enumerate() {
        let b = monde.boite_interpolee(i, alpha);
        let (x, y) = entier(&b);
        let (sx, sy) = ((x - cx) as f32, (y - cy) as f32);
        match entite {
            Entite::Goutte(g) => {
                if !g.est_ramassee() {
                    // Petit flottement d'un pixel, en 4 temps. Le dessin (10x10) déborde d'un
                    // pixel de chaque côté de la boîte de ramassage (8x8).
                    let flotte = [0.0, -1.0, -1.0, 0.0][(pas / 15) as usize % 4];
                    banque.dessiner(
                        image(&objets::GOUTTE, pas, 8),
                        sx - 1.0,
                        sy + flotte - 1.0,
                        false,
                    );
                }
            }
            Entite::Lanterne(l) => {
                let nom = if l.est_active() {
                    objets::LANTERNE[1]
                } else {
                    objets::LANTERNE[0]
                };
                banque.dessiner(nom, sx, sy, false);
            }
            Entite::Champignon(c) => {
                // Sprite 16x12 : le chapeau commence à la hauteur de la boîte.
                let nom = if c.est_comprime() {
                    objets::CHAMPIGNON[1]
                } else {
                    objets::CHAMPIGNON[0]
                };
                banque.dessiner(nom, sx, sy - 3.0, false);
            }
            Entite::Escargot(e) => {
                // Sprite 16x12 (tourné vers la gauche) posé sur le bas de la boîte
                // 16x10. Sa coquille de champignon dit déjà « je fais rebondir ».
                banque.dessiner(
                    image_escargot(e.est_comprime(), pas),
                    sx,
                    sy - 2.0,
                    e.vers_la_droite,
                );
            }
            Entite::Pointe(p) => {
                // La boîte fait 16 px de large : deux pointes de 8 px.
                let n = (p.boite.w / 8.0) as i32;
                for k in 0..n {
                    banque.dessiner("pointe", sx + (k * 8) as f32, sy, false);
                }
            }
            Entite::Friable(f) => {
                let tremble = match f.etat {
                    EtatFriable::Effritement(_) => [0.0, 1.0, 0.0, -1.0][(pas / 2) as usize % 4],
                    EtatFriable::Absente(_) => continue,
                    EtatFriable::Stable => 0.0,
                };
                // Dalle d'ardoise sur les toits, nuage en altitude (table d'ambiance).
                let anim = if decor::ambiance(monde.infos.theme).friable_nuage {
                    &objets::FRIABLE
                } else {
                    &objets::FRIABLE_ARDOISE
                };
                let nom = if matches!(f.etat, EtatFriable::Effritement(_)) {
                    anim[1]
                } else {
                    anim[0]
                };
                for k in 0..(f.boite.w / 16.0).ceil() as i32 {
                    banque.dessiner(nom, sx + (k * 16) as f32 + tremble, sy, false);
                }
            }
            // Une aiguille d'horloge (cadran et tige dessinés avant, voir
            // `dessiner_cadrans`) ou une plateforme en aller-retour.
            Entite::Mobile(m) => {
                let nom = if m.cercle().is_some() {
                    "aiguille"
                } else {
                    "mobile"
                };
                banque.dessiner(nom, sx, sy, false);
            }
            Entite::Montgolfiere(m) => {
                // Sprite 20x40 : la nacelle (16x8, la boîte `b`) occupe ses 8 dernières
                // rangées, le ballon les 32 du dessus et déborde de 2 px de chaque
                // côté. Brûleur allumé (la flamme vacille, une image sur 4 pas)
                // seulement pendant la MONTÉE : c'est l'indice « Pépin la fait monter ».
                let nom = if m.etat() == EtatMontgolfiere::Montee {
                    objets::MONTGOLFIERE[1 + (pas / 4) as usize % 2]
                } else {
                    objets::MONTGOLFIERE[0]
                };
                banque.dessiner(nom, sx - 2.0, sy - HAUT_BALLON, false);
            }
            Entite::Auvent(a) => {
                // Sprite 16x15 posé 1 px sous le haut de la case : le dessus de la
                // zone de contact (`HAUT_TOILE` sous le haut de la case) tombe au
                // milieu de la pente de la toile.
                let nom = if a.est_enfonce() {
                    objets::AUVENT[1]
                } else {
                    objets::AUVENT[0]
                };
                banque.dessiner(nom, sx, sy - HAUT_TOILE + 1.0, false);
            }
            Entite::Ventilateur(v) => {
                // Sprite 16x16 qui couvre toute la case : la casette (socle 16x8)
                // tombe sur le BAS de la boîte `b`, le corps du ventilateur monte au-
                // dessus. On choisit l'image selon la phase : à l'arrêt, à l'annonce
                // (la lampe orange clignote de plus en plus vite, comme l'annonce
                // d'un éclair) ou en souffle (le courant d'air cyan remonte).
                let nom = if v.souffle() {
                    objets::VENTILATEUR[2]
                } else if v.en_annonce() {
                    // Clignotement de la lampe qui ACCÉLÈRE pendant l'annonce
                    // (comme l'éclair) : de ~2,5 Hz au début à ~15 Hz à la fin.
                    if annonce_allumee(v.progression_annonce()) {
                        objets::VENTILATEUR[1]
                    } else {
                        objets::VENTILATEUR[0]
                    }
                } else {
                    objets::VENTILATEUR[0]
                };
                banque.dessiner(nom, sx, sy - 8.0, false);
                // Repère au sol : pendant l'annonce, une barre orange reste allumée
                // au PIED de la casette (là où elle touche le sol) : on voit la
                // COLONNE d'où le souffle va reprendre, comme la barre dorée au
                // pied de l'éclair.
                if v.en_annonce() {
                    rect(sx as i32, sy as i32 + 6, 16, 2, OR);
                }
            }
            Entite::Grele(g) => {
                // Un solide sous la colonne ? (Sinon elle tombe dans le vide.)
                let t = TAILLE_TUILE;
                let sol = monde
                    .niveau
                    .est_solide((g.zone.x / t) as i32, (g.zone.bas() / t) as i32);
                let morceaux = grele::disposer(g.zone, pas, sol, parapluie);
                grele::dessiner(&morceaux, banque, cx, cy);
            }
            Entite::Eclair(e) => dessiner_eclair(
                &e.zone,
                e.en_annonce(),
                e.progression_annonce(),
                e.frappe(),
                cx,
                cy,
            ),
            Entite::Ennemi(e) => {
                if !e.est_visible() {
                    continue;
                }
                let ecrase = matches!(e.etat, EtatEnnemi::Ecrase(_));
                match e.type_ennemi {
                    TypeEnnemi::Flaque => {
                        let nom = if ecrase {
                            "flaque_ecrasee"
                        } else {
                            image(&ennemis::FLAQUE, pas, 12)
                        };
                        // Sprite 16x10 posé sur le bas de la boîte 12x8, centré.
                        banque.dessiner(nom, sx - 2.0, sy - 2.0, e.vers_la_droite);
                    }
                    TypeEnnemi::Corbeau => {
                        // Le corbeau du dessin regarde à droite.
                        banque.dessiner(
                            image(&ennemis::CORBEAU, pas, 10),
                            sx - 2.0,
                            sy - 1.0,
                            !e.vers_la_droite,
                        );
                    }
                    TypeEnnemi::Pigeon => {
                        // Le pigeon du dessin regarde à droite. Sprite 16x12 centré sur la
                        // boîte 12x10, sa dernière rangée (les pattes) sur le bas de la boîte.
                        let (nom, tremble, retourne) = image_pigeon(e.phase(), ecrase, pas);
                        banque.dessiner(
                            nom,
                            sx - 2.0 + tremble,
                            sy - 2.0,
                            e.vers_la_droite == retourne,
                        );
                    }
                    TypeEnnemi::Chat => {
                        // Le chat du dessin regarde à gauche. Sprite 16x14 posé sur le
                        // bas de la boîte 14x8, centré (1 px de chaque côté).
                        let (x, y) = (sx - 1.0, sy - 6.0);
                        let miroir = e.vers_la_droite;
                        banque.dessiner(image_chat(e.phase_chat(), ecrase, pas), x, y, miroir);
                        // L'indice au-dessus de sa tête : les « z » du sommeil, puis le
                        // « ! » de l'annonce, qui clignote de plus en plus vite.
                        if !ecrase {
                            for (nom, dx, dy) in bulles_chat(e.phase_chat(), pas) {
                                // La tête est à gauche du dessin : miroir, elle passe à droite.
                                let l = largeur(sprite_chat(nom)) as f32;
                                let bx = if miroir { x + 16.0 - dx - l } else { x + dx };
                                banque.dessiner(nom, bx, y + dy, false);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Image de l'escargot : il marche (2 images de 0,25 s), ou sa coquille est
/// aplatie juste après un rebond. Fonction pure.
pub(crate) fn image_escargot(comprime: bool, pas: u32) -> &'static str {
    if comprime {
        "escargot_rebond"
    } else {
        image(&ennemis::ESCARGOT, pas, 15)
    }
}

/// Image du chat selon sa phase. Endormi, il respire (2 images de 0,5 s) ; pendant
/// l'annonce il lève la tête, un œil ouvert ; éveillé, il est assis, hérissé.
/// Fonction pure.
pub(crate) fn image_chat(phase: PhaseChat, ecrase: bool, pas: u32) -> &'static str {
    if ecrase {
        return "chat_ecrase";
    }
    match phase {
        PhaseChat::Dort(_) => image(&ennemis::CHAT_DORT, pas, 30),
        PhaseChat::Reveil(_) => "chat_reveil",
        PhaseChat::Eveille(_) => "chat_eveille",
    }
}

/// Durée (pas) de la montée d'un « z » au-dessus du chat endormi.
const MONTEE_Z_PAS: u32 = 60;

/// Les indices dessinés au-dessus de la tête du chat : `(sprite, dx, dy)`, en px
/// depuis le coin haut-gauche du sprite du chat (tourné vers la gauche).
/// - Il dort : deux « z » montent l'un après l'autre en s'écartant de sa tête
///   (10 px de montée en 1 s, décalés d'une demi-seconde).
/// - Il se réveille : le « ! » doré clignote de plus en plus vite (comme
///   l'annonce d'un éclair), tout le temps de l'annonce.
/// - Éveillé : rien (il est assis, hérissé : son image suffit).
///
/// Fonction pure.
pub(crate) fn bulles_chat(phase: PhaseChat, pas: u32) -> Vec<(&'static str, f32, f32)> {
    match phase {
        PhaseChat::Dort(_) => (0..2)
            .map(|k| {
                let t = (pas + k * MONTEE_Z_PAS / 2) % MONTEE_Z_PAS;
                // Entiers : le « z » reste sur la grille des pixels.
                let montee = t * 10 / MONTEE_Z_PAS;
                ("chat_z", 1.0 - (montee / 2) as f32, -2.0 - montee as f32)
            })
            .collect(),
        PhaseChat::Reveil(n) => {
            let progression = 1.0 - n as f32 / pepin_core::entities::ennemi::REVEIL_PAS as f32;
            if annonce_allumee(progression) {
                vec![("chat_alerte", 2.0, -6.0)]
            } else {
                Vec::new()
            }
        }
        PhaseChat::Eveille(_) => Vec::new(),
    }
}

/// Dessin d'un indice du chat (pour connaître sa largeur).
fn sprite_chat(nom: &str) -> crate::sprites::Image {
    if nom == "chat_alerte" {
        ennemis::CHAT_ALERTE
    } else {
        ennemis::CHAT_Z
    }
}

/// Image du pigeon selon sa phase : `(nom du sprite, tremblement en px, retourné)`.
/// Perché ou au repos, il hoche la tête ; pendant l'annonce il tremble d'un pixel
/// (comme une plateforme friable qui va céder) ; pendant la remontée il est
/// « retourné » : il revient vers son perchoir, donc il regarde derrière lui.
/// Fonction pure.
pub(crate) fn image_pigeon(
    phase: PhasePigeon,
    ecrase: bool,
    pas: u32,
) -> (&'static str, f32, bool) {
    if ecrase {
        return ("pigeon_ecrase", 0.0, false);
    }
    match phase {
        PhasePigeon::Perche | PhasePigeon::Repos(_) => {
            (image(&ennemis::PIGEON, pas, 24), 0.0, false)
        }
        PhasePigeon::Annonce(_) => (
            "pigeon_annonce",
            [0.0, 1.0, 0.0, -1.0][(pas / 2) as usize % 4],
            false,
        ),
        PhasePigeon::Plongee(_) => ("pigeon_plongee", 0.0, false),
        PhasePigeon::Remontee(_) => ("pigeon_remontee", 0.0, true),
    }
}

fn dessiner_arrivee(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    let a = monde.niveau.arrivee;
    let x = (a.x + a.w / 2.0 - 8.0).round() as i32 - cx;
    let y = a.bas().round() as i32 - 32 - cy;
    banque.dessiner(
        image(&objets::DRAPEAU, monde.pas, 12),
        x as f32,
        y as f32,
        false,
    );
}

/// Marge (px) gardée entre le bas de l'écran et l'animation de mort d'une chute.
const MARGE_MORT_BAS: f32 = 16.0;
/// Largeur (px) réservée à l'animation de mort pour qu'elle reste entière à l'écran.
const LARGEUR_MORT: f32 = 16.0;

/// Place l'animation de mort (et les éclats) dans le cadre. `pos` : où Pépin est mort, dans le
/// niveau (coin haut-gauche) ; `camera` : coin haut-gauche de l'écran dans le niveau. Une chute
/// dans un trou finit sous l'écran, et la caméra repart vers le point de reprise : sans cette
/// limite, l'animation se jouerait hors champ. L'ordonnée est bornée à 16 px au-dessus du bord
/// bas, l'abscisse à la largeur de l'écran. Fonction pure.
pub fn borner_mort(pos: (f32, f32), camera: (f32, f32)) -> (f32, f32) {
    let x_max = camera.0 + LARGEUR_LOGIQUE as f32 - LARGEUR_MORT;
    let y_max = camera.1 + HAUTEUR_LOGIQUE as f32 - MARGE_MORT_BAS;
    (pos.0.clamp(camera.0, x_max), pos.1.min(y_max))
}

/// Taille (largeur, hauteur) du repère « Pépin est au-dessus de l'écran ».
const REPERE: (i32, i32) = (5, 3);

/// Où poser le repère en haut de l'écran ? `boite_y`, `boite_h` : ordonnée écran du haut de la
/// boîte de Pépin et sa hauteur ; `milieu_x` : abscisse écran de son milieu. Renvoie l'abscisse
/// écran (bord gauche) du repère, bornée pour qu'il reste entier dans l'écran, ou `None` tant
/// que Pépin est visible (au moins une ligne de sa boîte dans le cadre). Fonction pure.
pub(crate) fn repere_haut(boite_y: i32, boite_h: i32, milieu_x: i32) -> Option<i32> {
    if boite_y + boite_h > 0 {
        return None;
    }
    let max = LARGEUR_LOGIQUE as i32 - REPERE.0;
    Some((milieu_x - REPERE.0 / 2).clamp(0, max))
}

/// Petite flèche corail collée en haut de l'écran, à l'abscisse de Pépin, quand il saute
/// au-dessus du cadre : au-dessus d'un niveau d'un écran (caméra verticale figée), ou
/// au-dessus du HAUT d'un niveau haut (la caméra s'arrête à sa borne haute).
fn dessiner_repere(x: i32) {
    const FLECHE: [(i32, i32); 3] = [(2, 1), (1, 3), (0, 5)]; // (décalage, largeur) par rangée
    for (k, &(dx, w)) in FLECHE.iter().enumerate() {
        rect(x + dx, k as i32, w, 1, CORAIL);
    }
}

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
/// Avant 2 s, toujours `None`. Ensuite, un cycle de 5 s qui recommence :
/// - 0 à 8 : il cligne des yeux ;
/// - 60 à 120 : il lève les yeux vers le ciel (1 s), puis cligne de nouveau (120 à 128) ;
/// - 180 à 244 : il tapote le sol de son parapluie, 4 fois (« tac » de 8 pas, puis 8 pas
///   de repos, `pepin_idle0` : le corps ne bouge pas entre deux coups).
///
/// Comme la minuterie repart de zéro au premier mouvement, l'attente s'arrête net dès
/// qu'on appuie sur une touche. Fonction pure.
pub(crate) fn image_attente(immobile: u32) -> Option<&'static str> {
    // `checked_sub` : `None` avant 2 s, et `?` sort aussitôt de la fonction avec ce `None`.
    let t = immobile.checked_sub(ATTENTE_PAS)? % CYCLE_ATTENTE;
    let [cligne, ciel, tapote] = pepin::ATTENTE;
    match t {
        0..8 | 120..128 => Some(cligne),
        60..120 => Some(ciel),
        _ if (DEBUT_TAPOTE..DEBUT_TAPOTE + 2 * DUREE_TAC * NB_TACS).contains(&t) => {
            let coup = (t - DEBUT_TAPOTE) / DUREE_TAC;
            Some(if coup.is_multiple_of(2) {
                tapote
            } else {
                pepin::REPOS[0]
            })
        }
        _ => None,
    }
}

fn dessiner_pepin(monde: &Monde, banque: &Banque, alpha: f32, cx: i32, cy: i32) {
    let pas = monde.pas;
    let j = &monde.joueur;
    let miroir = !j.regarde_droite;

    // Mort : trois images jouées à l'endroit de la mort, puis réapparition clignotante.
    if monde.reapparition > 0 {
        let ecoule = REAPPARITION_PAS - monde.reapparition;
        let k = (ecoule as usize * 3 / REAPPARITION_PAS as usize).min(2);
        // Chute dans le vide ou réapparition lointaine : l'animation reste dans le cadre.
        let (mort_x, mort_y) = borner_mort(monde.pos_mort, (cx as f32, cy as f32));
        let (mx, my) = (
            mort_x.round() as i32 - cx - 3,
            mort_y.round() as i32 - cy - 2,
        );
        banque.dessiner(pepin::REAPPARITION[k], mx as f32, my as f32, false);
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
            let nom = image(&pepin::PLANER, pas, 8);
            banque.dessiner(nom, (milieu - 10) as f32, (bas - 20) as f32, miroir);
        }
        etat => {
            let nom = match etat {
                EtatJoueur::Montee => "pepin_saut",
                EtatJoueur::Chute => "pepin_chute",
                _ if j.vitesse.x.abs() > 8.0 => image(&pepin::COURSE, pas, 5),
                _ => image_attente(monde.immobile).unwrap_or_else(|| image(&pepin::REPOS, pas, 30)),
            };
            banque.dessiner(nom, (milieu - 8) as f32, (bas - 16) as f32, miroir);
        }
    }
}

/// Flèche de repère par-dessus tout le monde (pluie comprise) quand Pépin est au-dessus du cadre.
fn dessiner_repere_pepin(monde: &Monde, alpha: f32, cx: i32, cy: i32) {
    let j = &monde.joueur;
    let (px, py) = monde.joueur_interpole(alpha);
    let (bx, by) = (px.round() as i32 - cx, py.round() as i32 - cy);
    if let Some(x) = repere_haut(by, j.boite.h as i32, bx + j.boite.w as i32 / 2) {
        dessiner_repere(x);
    }
}

/// Dessine tout le monde dans la cible logique. `(cx, cy)` : caméra entière.
pub fn dessiner(
    monde: &Monde,
    banque: &Banque,
    particules: &Particules,
    alpha: f32,
    cx: i32,
    cy: i32,
) {
    // Le fond compte la caméra depuis le bas du niveau (parallaxe verticale d'un niveau
    // haut) ; un niveau d'un écran lui passe la caméra telle quelle.
    let cy_fond = decor::camera_fond(cy, monde.niveau.hauteur_px() as i32);
    decor::dessiner_fond(monde.infos.theme, cx, cy_fond, monde.pas);
    dessiner_tuiles(monde, banque, cx, cy);
    dessiner_decors(monde, banque, cx, cy);
    dessiner_panneaux(monde, banque, cx, cy);
    dessiner_arrivee(monde, banque, cx, cy);
    dessiner_cadrans(monde, banque, alpha, cx, cy);
    dessiner_entites(monde, banque, alpha, cx, cy);
    dessiner_pepin(monde, banque, alpha, cx, cy);
    particules.dessiner(cx, cy);
    decor::dessiner_avant(monde.infos.theme, monde.pas);
    dessiner_repere_pepin(monde, alpha, cx, cy);
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Aiguilles d'horloge (carte 5.10) ---

    /// La tige d'une aiguille de 48 px de rayon : 7 maillons tous les 6 px, du
    /// moyeu jusque sous la plaque, alignés sur le rayon, à midi comme à 3 h.
    #[test]
    fn les_maillons_vont_du_pivot_a_la_plaque() {
        let midi = maillons((120.0, 88.0), (120.0, 40.0));
        assert_eq!(midi.len(), 7);
        assert_eq!(midi[0], (118, 80));
        assert_eq!(midi[6], (118, 44));
        assert!(midi
            .windows(2)
            .all(|w| w[0].0 == w[1].0 && w[0].1 - w[1].1 == 6));
        let trois_heures = maillons((120.0, 88.0), (168.0, 88.0));
        let attendu: Vec<(i32, i32)> = (1..=7).map(|k| (118 + 6 * k, 86)).collect();
        assert_eq!(trois_heures, attendu);
        // En diagonale : chaque maillon reste à moins d'un pixel du rayon.
        let diag = maillons((0.0, 0.0), (30.0, 40.0));
        assert_eq!(diag.len(), 7);
        for (x, y) in diag {
            let (cx, cy) = (x as f32 + 2.0, y as f32 + 2.0);
            let ecart = (cx * 40.0 - cy * 30.0).abs() / 50.0;
            assert!(ecart <= 0.75, "({x}, {y}) : {ecart}");
        }
        assert!(maillons((5.0, 5.0), (5.0, 5.0)).is_empty());
    }

    // --- Variantes de brique et ombre sous les toits ---

    #[test]
    fn le_choix_de_la_variante_de_brique_est_fixe() {
        // Même case, même réponse, à chaque appel (fonction pure, sans aléatoire).
        for ty in -3..15 {
            for tx in -3..250 {
                assert_eq!(variante_brique(tx, ty), variante_brique(tx, ty));
            }
        }
        // Valeurs de référence (relevées une fois avec ce hachage) : si elles changent,
        // tous les murs du jeu changent d'aspect ; il faut alors le vouloir.
        assert_eq!(hachage_case(0, 0), 0);
        assert_eq!(hachage_case(1, 0), 1_834_104_592);
        assert_eq!(hachage_case(-1, -1), 857_305_552);
        let rangee_9: Vec<(i32, &str)> = (0..40)
            .filter_map(|tx| variante_brique(tx, 9).map(|v| (tx, v)))
            .collect();
        assert_eq!(
            rangee_9,
            [
                (0, "brique_manquante"),
                (4, "brique_manquante"),
                (11, "brique_coulure"),
                (14, "brique_coulure"),
                (16, "brique_fissure"),
                (32, "brique_coulure"),
                (36, "brique_fissure"),
                (37, "brique_manquante"),
                (39, "brique_manquante"),
            ]
        );
    }

    #[test]
    fn environ_une_brique_sur_six_est_une_variante() {
        let mut compte = [0usize; 3];
        let mut total = 0;
        for ty in 0..40 {
            for tx in 0..400 {
                total += 1;
                if let Some(v) = variante_brique(tx, ty) {
                    compte[VARIANTES_BRIQUE.iter().position(|&n| n == v).unwrap()] += 1;
                }
            }
        }
        let part = compte.iter().sum::<usize>() as f32 / total as f32;
        // Demande : une tuile sur 5 à 8.
        assert!((1.0 / 8.0..=1.0 / 5.0).contains(&part), "part : {part}");
        // Les trois dessins sortent à peu près autant les uns que les autres.
        for (i, &n) in compte.iter().enumerate() {
            let p = n as f32 / compte.iter().sum::<usize>() as f32;
            assert!((0.25..=0.42).contains(&p), "{} : {p}", VARIANTES_BRIQUE[i]);
        }
    }

    #[test]
    fn deux_variantes_identiques_ne_se_touchent_jamais() {
        for ty in -5..40 {
            for tx in -50..400 {
                if let Some(v) = variante_brique(tx, ty) {
                    assert_ne!(variante_brique(tx + 1, ty), Some(v), "({tx},{ty}) à droite");
                    assert_ne!(variante_brique(tx, ty + 1), Some(v), "({tx},{ty}) dessous");
                }
            }
        }
    }

    /// Dans les vrais niveaux : part des briques pleines dessinées en variante, et nombre de
    /// briques qui reçoivent la bande d'ombre d'un toit.
    #[test]
    fn variantes_et_ombres_dans_les_vrais_niveaux() {
        use pepin_core::zones::{Theme, NIVEAUX};
        let (mut pleines, mut variantes) = (0, 0);
        for infos in NIVEAUX {
            let n = infos.charger().unwrap();
            let mut ombres = 0;
            for ty in 0..n.hauteur() as i32 {
                for tx in 0..n.largeur() as i32 {
                    let (t, dessus) = (n.tuile_decor(tx, ty), n.tuile_decor(tx, ty - 1));
                    if t == Tuile::Brique && dessus.est_solide() {
                        pleines += 1;
                        variantes += usize::from(variante_brique(tx, ty).is_some());
                    }
                    if ombre_sous_toit(t, dessus) {
                        ombres += 1;
                    }
                }
            }
            // Les trois niveaux de toits ont un long rebord ; les nuages n'en ont pas.
            match infos.theme {
                Theme::Toits | Theme::Gouttieres | Theme::Orage => {
                    assert!(ombres > 100, "{} : {ombres} ombres", infos.nom)
                }
                Theme::Soleil => assert_eq!(ombres, 0, "{}", infos.nom),
                Theme::Altitude => {}
            }
        }
        let part = variantes as f32 / pleines as f32;
        assert!(
            (1.0 / 8.0..=1.0 / 5.0).contains(&part),
            "{variantes} / {pleines}"
        );
    }

    #[test]
    fn l_ombre_ne_tombe_que_d_un_toit_sur_une_brique() {
        assert!(ombre_sous_toit(Tuile::Brique, Tuile::Toit));
        assert!(!ombre_sous_toit(Tuile::Brique, Tuile::Brique));
        assert!(!ombre_sous_toit(Tuile::Brique, Tuile::Vide));
        assert!(!ombre_sous_toit(Tuile::Toit, Tuile::Toit));
        assert!(!ombre_sous_toit(Tuile::Nuage, Tuile::Toit));
        assert!(!ombre_sous_toit(Tuile::Vide, Tuile::Toit));
    }

    #[test]
    fn le_repere_n_apparait_que_pepin_hors_du_cadre() {
        // Boîte de 14 px de haut : visible tant que sa dernière ligne est dans l'écran.
        assert_eq!(repere_haut(100, 14, 160), None);
        assert_eq!(repere_haut(-13, 14, 160), None, "une ligne encore visible");
        assert!(repere_haut(-14, 14, 160).is_some(), "entièrement au-dessus");
        assert!(repere_haut(-60, 14, 160).is_some());
    }

    #[test]
    fn le_repere_est_centre_sur_pepin_et_borne_a_l_ecran() {
        assert_eq!(repere_haut(-20, 14, 160), Some(158));
        assert_eq!(repere_haut(-20, 14, 0), Some(0));
        assert_eq!(repere_haut(-20, 14, -50), Some(0));
        assert_eq!(repere_haut(-20, 14, 319), Some(315));
        assert_eq!(repere_haut(-20, 14, 900), Some(315));
    }

    #[test]
    fn l_annonce_clignote_de_plus_en_plus_vite() {
        let transitions = |a: f32, b: f32| {
            let n = 480;
            let mut t = 0;
            let mut avant = annonce_allumee(a);
            for k in 1..=n {
                let p = a + (b - a) * k as f32 / n as f32;
                let maintenant = annonce_allumee(p);
                if maintenant != avant {
                    t += 1;
                }
                avant = maintenant;
            }
            t
        };
        let debut = transitions(0.0, 0.5);
        let fin = transitions(0.5, 1.0);
        assert!(fin > debut, "début : {debut}, fin : {fin}");
        assert!(annonce_allumee(0.0), "allumée dès la première image");
        // 7 clignotements en tout : 14 changements d'état.
        assert_eq!(transitions(0.0, 1.0), 14);
    }

    #[test]
    fn le_clignotement_le_plus_rapide_reste_lisible_a_60_images_par_seconde() {
        // 48 images d'annonce : jamais plus de 2 changements d'état à la suite sans plateau.
        let etats: Vec<bool> = (0..48).map(|k| annonce_allumee(k as f32 / 48.0)).collect();
        let mut plus_court = usize::MAX;
        let mut serie = 1;
        for w in etats.windows(2) {
            if w[0] == w[1] {
                serie += 1;
            } else {
                plus_court = plus_court.min(serie);
                serie = 1;
            }
        }
        assert!(plus_court >= 2, "plus petite série : {plus_court} image(s)");
    }

    #[test]
    fn l_eclair_part_du_haut_de_l_ecran() {
        // Niveau de 176 px, caméra à -4 : la zone (y = 0) commence 4 px sous le haut.
        assert_eq!(colonne_eclair(0, 80, -4), (0, 84));
        // Caméra à 0 : rien à prolonger.
        assert_eq!(colonne_eclair(0, 80, 0), (0, 80));
        // Zone qui ne part pas du haut du niveau : inchangée.
        assert_eq!(colonne_eclair(32, 80, -4), (36, 80));
        // Niveau défilé : la zone remonte au-dessus de l'écran, rien à prolonger.
        assert_eq!(colonne_eclair(0, 80, 20), (-20, 80));
    }

    #[test]
    fn la_mort_dans_le_vide_reste_dans_le_cadre() {
        let cam = (200.0, -4.0);
        // Chute sous le niveau : ordonnée bornée à 16 px au-dessus du bas de l'écran.
        let (x, y) = borner_mort((300.0, 400.0), cam);
        assert_eq!((x, y), (300.0, -4.0 + 180.0 - 16.0));
        // Déjà dans le cadre : inchangé.
        assert_eq!(borner_mort((250.0, 100.0), cam), (250.0, 100.0));
        // Mort hors champ (réapparition lointaine) : abscisse ramenée dans l'écran.
        assert_eq!(borner_mort((20.0, 100.0), cam).0, 200.0);
        assert_eq!(borner_mort((900.0, 100.0), cam).0, 200.0 + 320.0 - 16.0);
    }

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

    // --- Objets de décor (carte 4.2) ---

    /// Chaque oiseau tourne la tête 40 pas sur 300 (2/3 s toutes les 5 s), jamais tous
    /// ensemble : deux oiseaux voisins ne changent pas d'image au même pas.
    #[test]
    fn l_oiseau_tourne_la_tete_de_temps_en_temps() {
        for rang in 0..6 {
            let tete: Vec<bool> = (0..CYCLE_OISEAU)
                .map(|pas| image_oiseau(pas, rang) == "oiseau1")
                .collect();
            assert_eq!(tete.iter().filter(|&&t| t).count(), TETE_TOURNEE as usize);
            // Une seule série de tête tournée par cycle (pas de clignotement).
            let changements = (0..CYCLE_OISEAU as usize)
                .filter(|&i| tete[i] != tete[(i + 1) % CYCLE_OISEAU as usize])
                .count();
            assert_eq!(changements, 2, "oiseau {rang}");
        }
        for pas in 0..3 * CYCLE_OISEAU {
            let ensemble = (0..6).all(|rang| image_oiseau(pas, rang) == "oiseau1");
            assert!(!ensemble, "pas {pas} : tous les oiseaux tournent la tête");
        }
    }

    /// Au niveau 4, chaque manche à air est à gauche de son courant : elle s'étire vers la
    /// droite. Retournée si le courant le plus proche est à gauche.
    #[test]
    fn la_manche_a_air_s_etire_vers_son_courant() {
        let niv = pepin_core::levels::vents::INFOS.charger().unwrap();
        let manches: Vec<f32> = niv
            .decors
            .iter()
            .filter(|d| d.genre == GenreDecor::MancheAAir)
            .map(|d| d.x)
            .collect();
        assert_eq!(manches.len(), 4);
        assert!(manches
            .iter()
            .all(|&x| manche_vers_la_droite(x, &niv.courants)));
        let a_gauche = CourantAscendant::new(Aabb::new(0.0, 0.0, 48.0, 64.0));
        let a_droite = CourantAscendant::new(Aabb::new(300.0, 0.0, 48.0, 64.0));
        assert!(!manche_vers_la_droite(80.0, &[a_gauche]));
        assert!(manche_vers_la_droite(80.0, &[a_droite]));
        // Le plus proche l'emporte.
        assert!(!manche_vers_la_droite(80.0, &[a_droite, a_gauche]));
        assert!(manche_vers_la_droite(250.0, &[a_gauche, a_droite]));
        assert!(
            manche_vers_la_droite(80.0, &[]),
            "sans courant : vers la droite"
        );
    }

    /// Un sprite de décor est centré dans sa case et posé sur la tuile du dessous ; chaque
    /// genre de décor a son dessin.
    #[test]
    fn le_decor_est_pose_sur_sa_case() {
        assert_eq!(coin_decor(48.0, 112.0, 12, 20), (50, 108));
        assert_eq!(coin_decor(48.0, 112.0, 16, 22), (48, 106));
        assert_eq!(coin_decor(48.0, 112.0, 7, 24), (52, 104));
        let noms = ["cheminee", "antenne", "paratonnerre", "manche_a_air"];
        for nom in noms.iter().chain(objets_decor::OISEAU.iter()) {
            assert!(
                objets_decor::SPRITES.iter().any(|(n, _)| n == nom),
                "{nom} : pas de dessin"
            );
        }
        assert_eq!(GenreDecor::TOUS.len(), noms.len() + 1);
    }

    // --- Attente de Pépin (carte 4.3) ---

    #[test]
    fn pas_d_attente_avant_2_s_immobile() {
        for immobile in 0..ATTENTE_PAS {
            assert_eq!(image_attente(immobile), None, "pas {immobile}");
        }
        // Pile à 2 s, il cligne des yeux.
        assert_eq!(image_attente(ATTENTE_PAS), Some("pepin_cligne"));
    }

    /// Un cycle de 5 s montre les trois images d'attente, chacune dans sa plage, et laisse
    /// Pépin respirer (images du repos) plus de la moitié du temps.
    #[test]
    fn un_cycle_d_attente_montre_les_trois_images() {
        let cycle: Vec<Option<&str>> = (0..CYCLE_ATTENTE)
            .map(|t| image_attente(ATTENTE_PAS + t))
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
        let mut precedente = image_attente(debut - 1);
        for t in debut..debut + 80 {
            let i = image_attente(t);
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
            let i = image_attente(ATTENTE_PAS + t);
            assert_eq!(image_attente(ATTENTE_PAS + t + CYCLE_ATTENTE), i, "t = {t}");
            assert_eq!(
                image_attente(ATTENTE_PAS + t + 1000 * CYCLE_ATTENTE),
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
        assert_eq!(image_attente(immobile), Some("pepin_ciel"));
        let droite = Entrees {
            direction: Direction::Droite,
            saut: false,
        };
        immobile = compter_immobile(immobile, &m.joueur, &droite);
        assert_eq!(image_attente(immobile), None);
    }

    // --- Escargot (carte 5.8) ---

    #[test]
    fn l_escargot_marche_sur_deux_images_et_s_aplatit_au_rebond() {
        let marche: Vec<&str> = (0..60).map(|p| image_escargot(false, p)).collect();
        assert!(marche.contains(&"escargot0") && marche.contains(&"escargot1"));
        assert!(marche
            .iter()
            .all(|n| n.starts_with("escargot") && *n != "escargot_rebond"));
        assert!((0..60).all(|p| image_escargot(true, p) == "escargot_rebond"));
        let noms: Vec<&str> = crate::sprites::tous().iter().map(|(n, _)| *n).collect();
        for n in ["escargot0", "escargot1", "escargot_rebond"] {
            assert!(noms.contains(&n), "{n}");
        }
    }

    // --- Chat endormi (carte 5.9) ---

    #[test]
    fn le_chat_a_une_image_par_phase() {
        use pepin_core::entities::ennemi::{DORT_PAS, EVEILLE_PAS, REVEIL_PAS};
        // Endormi, il respire : les deux images alternent.
        let dort: Vec<_> = (0..120)
            .map(|p| image_chat(PhaseChat::Dort(DORT_PAS), false, p))
            .collect();
        assert!(dort.contains(&"chat_dort0") && dort.contains(&"chat_dort1"));
        assert!(dort.iter().all(|n| n.starts_with("chat_dort")));
        assert_eq!(
            image_chat(PhaseChat::Reveil(REVEIL_PAS), false, 3),
            "chat_reveil"
        );
        assert_eq!(
            image_chat(PhaseChat::Eveille(EVEILLE_PAS), false, 3),
            "chat_eveille"
        );
        for phase in [
            PhaseChat::Dort(5),
            PhaseChat::Reveil(5),
            PhaseChat::Eveille(5),
        ] {
            assert_eq!(image_chat(phase, true, 9), "chat_ecrase");
        }
        // Toutes ces images existent dans la banque de sprites.
        let noms: Vec<&str> = crate::sprites::tous().iter().map(|(n, _)| *n).collect();
        for n in [
            "chat_dort0",
            "chat_dort1",
            "chat_reveil",
            "chat_eveille",
            "chat_ecrase",
        ] {
            assert!(noms.contains(&n), "{n}");
        }
    }

    #[test]
    fn les_z_montent_pendant_le_sommeil_et_le_point_d_exclamation_annonce_le_reveil() {
        use pepin_core::entities::ennemi::{EVEILLE_PAS, REVEIL_PAS};
        // Endormi : toujours deux « z », qui montent puis repartent d'en bas.
        let hauteurs: Vec<f32> = (0..MONTEE_Z_PAS)
            .map(|p| {
                let b = bulles_chat(PhaseChat::Dort(100), p);
                assert_eq!(b.len(), 2);
                assert!(b.iter().all(|&(n, _, dy)| n == "chat_z" && dy < 0.0));
                b[0].2
            })
            .collect();
        assert_eq!(hauteurs[0], -2.0);
        assert!(hauteurs.windows(2).all(|w| w[1] <= w[0]), "il monte");
        assert!(hauteurs.last().unwrap() <= &-11.0);
        // Annonce du réveil : le « ! » clignote, de plus en plus vite.
        let allume: Vec<bool> = (1..=REVEIL_PAS)
            .rev()
            .map(|n| !bulles_chat(PhaseChat::Reveil(n), 0).is_empty())
            .collect();
        let changements = |t: &[bool]| t.windows(2).filter(|w| w[0] != w[1]).count();
        let (debut, fin) = allume.split_at(allume.len() / 2);
        assert!(allume[0], "allumé dès le début de l'annonce");
        assert!(changements(fin) > changements(debut), "{allume:?}");
        // Éveillé : aucun indice.
        assert!(bulles_chat(PhaseChat::Eveille(EVEILLE_PAS), 0).is_empty());
    }

    // --- Pigeon (carte 4.4) ---

    #[test]
    fn le_pigeon_a_une_image_par_phase() {
        use pepin_core::entities::ennemi::{ANNONCE_PIGEON_PAS, REPOS_PIGEON_PAS};
        // Perché ou au repos : il hoche la tête (2 images), sans trembler.
        let perche: Vec<_> = (0..48)
            .map(|p| image_pigeon(PhasePigeon::Perche, false, p))
            .collect();
        assert!(perche.iter().all(|&(_, t, r)| t == 0.0 && !r));
        assert!(perche.iter().any(|&(n, _, _)| n == "pigeon0"));
        assert!(perche.iter().any(|&(n, _, _)| n == "pigeon1"));
        assert_eq!(
            image_pigeon(PhasePigeon::Repos(REPOS_PIGEON_PAS), false, 0),
            image_pigeon(PhasePigeon::Perche, false, 0)
        );
        // Annonce : il tremble d'un pixel, d'un côté puis de l'autre.
        let tremble: Vec<f32> = (0..8)
            .map(|p| {
                let (nom, t, r) = image_pigeon(PhasePigeon::Annonce(ANNONCE_PIGEON_PAS), false, p);
                assert_eq!((nom, r), ("pigeon_annonce", false));
                t
            })
            .collect();
        assert!(tremble.contains(&1.0) && tremble.contains(&-1.0) && tremble.contains(&0.0));
        // Plongée, puis remontée : retourné, il regarde son perchoir.
        assert_eq!(
            image_pigeon(PhasePigeon::Plongee(3), false, 5),
            ("pigeon_plongee", 0.0, false)
        );
        assert_eq!(
            image_pigeon(PhasePigeon::Remontee(3), false, 5),
            ("pigeon_remontee", 0.0, true)
        );
        // Écrasé : quelle que soit la phase où on l'a eu.
        for phase in [
            PhasePigeon::Perche,
            PhasePigeon::Plongee(10),
            PhasePigeon::Remontee(1),
        ] {
            assert_eq!(image_pigeon(phase, true, 7), ("pigeon_ecrase", 0.0, false));
        }
    }
}
