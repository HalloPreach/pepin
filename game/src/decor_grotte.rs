//! Habillage de LA GROTTE AUX BULLES (carte 9.20d) : le niveau ressemble à son nom
//! et se raccorde à la cascade (avant) et à la rivière (après). Rendu seulement :
//! aucune collision ; la grille (vasques, pilier, ouvertures) dit où dessiner.
//!
//! 1. L'ENTRÉE : la grotte est DERRIÈRE la cascade. Le mur de gauche est ouvert au
//!    pied (là où Pépin part) sur le RIDEAU D'EAU de la chute, qu'il vient de
//!    traverser : des filets d'eau qui dévalent, le jour qui passe entre eux,
//!    l'écume au pied, des gouttes projetées et des rais de lumière pâle qui
//!    entrent dans la grotte (pixels clairsemés, sans transparence).
//! 2. Les SOURCES : chaque colonne de bulles (`Y`) jaillit d'une source. Posée sur
//!    une petite vasque d'eau (`~` de la grille, au pied de la colonne), la source
//!    BOUILLONNE : des bouillons qui gonflent et crèvent, l'écume de la surface, un
//!    rebord de pierre. Posée sur la roche (la 2e colonne, dans le sol ; la 4e, au
//!    sommet d'un pilier), c'est une FISSURE d'où l'eau luit et sortent les bulles.
//! 3. La SORTIE : le mur de droite est ouvert au-dessus du rocher du drapeau sur le
//!    JOUR qui se lève (aube pâle), la rivière et sa rive d'herbes ocre ; des herbes
//!    poussent sur le rocher près de l'ouverture, des rais dorés entrent.
//!
//! Les fonctions de placement (`sources`, `entree`, `sortie`) sont pures et
//! testées. La scène appelle `dessiner` juste après les tuiles ; rien hors du
//! thème Grotte.

use macroquad::prelude::draw_rectangle;
use pepin_core::aabb::{Aabb, TAILLE_TUILE};
use pepin_core::entities::Entite;
use pepin_core::level::{Niveau, Tuile};
use pepin_core::palette::{
    Couleur, ARDOISE_NUIT, ARDOISE_OMBRE, ARDOISE_PALE, BLANC_CASSE, BRUME, NUAGE, OR,
    ORANGE_SOLEIL, OR_PALE, PLUIE, PLUIE_SOMBRE,
};
use pepin_core::physics::JOUEUR_H;
use pepin_core::zones::Theme;

use crate::monde::Monde;
use crate::rendu::couleur_mq;

const T: i32 = TAILLE_TUILE as i32;

/// Écart (px) entre deux bouillons d'une vasque.
const ECART_BOUILLONS: i32 = 8;
/// Durée (pas) d'une image d'un bouillon (il gonfle en trois images, crève, se tait).
const IMAGE_BOUILLON_PAS: u32 = 6;
/// Nombre de rais de lumière du rideau d'eau, et des rais du jour à la sortie.
const RAIS_RIDEAU: i32 = 4;
const RAIS_JOUR: i32 = 3;

fn rect(x: i32, y: i32, w: i32, h: i32, c: Couleur) {
    if w > 0 && h > 0 {
        draw_rectangle(x as f32, y as f32, w as f32, h as f32, couleur_mq(c));
    }
}

/// Le point `(x, y)` (px) est-il sur une case vide du niveau ?
fn vide(n: &Niveau, x: i32, y: i32) -> bool {
    n.tuile_decor(x.div_euclid(T), y.div_euclid(T)) == Tuile::Vide
}

/// D'où jaillit une colonne de bulles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GenreSource {
    /// Une vasque d'eau au pied de la colonne : elle bouillonne.
    Vasque,
    /// La roche sous la colonne : une fissure d'où sortent les bulles.
    Fissure,
}

/// La source d'une colonne de bulles : son bord gauche et sa largeur (ceux de la
/// colonne, px), l'ordonnée de la surface de la vasque ou du dessus de la roche.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Source {
    pub x: i32,
    pub largeur: i32,
    pub y: i32,
    pub genre: GenreSource,
}

/// La source de la colonne de bulles `z` : la première case d'eau de la colonne
/// (`level::ascii` la prolonge jusqu'au fond de l'eau qui est dessous), sinon la
/// roche juste sous elle. `None` si la colonne finit dans le vide.
fn source(n: &Niveau, z: &Aabb) -> Option<Source> {
    let (x, largeur) = (z.x.round() as i32, z.w.round() as i32);
    let tx = x.div_euclid(T);
    let (haut, bas) = (
        (z.y / TAILLE_TUILE).round() as i32,
        (z.bas() / TAILLE_TUILE).round() as i32,
    );
    let source = |ty: i32, genre| Source {
        x,
        largeur,
        y: ty * T,
        genre,
    };
    if let Some(ty) = (haut..bas).find(|&ty| n.tuile_decor(tx, ty) == Tuile::Eau) {
        return Some(source(ty, GenreSource::Vasque));
    }
    n.tuile_decor(tx, bas)
        .est_solide()
        .then(|| source(bas, GenreSource::Fissure))
}

/// Les sources des colonnes de bulles du niveau, dans l'ordre des entités.
pub(crate) fn sources(n: &Niveau, entites: &[Entite]) -> Vec<Source> {
    entites
        .iter()
        .filter_map(|e| match e {
            Entite::Bulles(b) => source(n, &b.zone),
            _ => None,
        })
        .collect()
}

/// Une ouverture dans un mur du bord : sa colonne (bord gauche, px) et ses
/// rangées (haut et bas exclu, px).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Ouverture {
    pub x: i32,
    pub haut: i32,
    pub bas: i32,
}

/// Les ouvertures de la colonne de bord `tx` : ses suites de cases vides, sous le
/// plafond (rangée 0) et au-dessus du sol (dernière rangée).
fn ouvertures(n: &Niveau, tx: i32) -> Vec<Ouverture> {
    let mut liste = Vec::new();
    let mut debut = None;
    for ty in 1..n.hauteur() as i32 {
        let ouvert = ty < n.hauteur() as i32 - 1 && n.tuile_decor(tx, ty) == Tuile::Vide;
        match (ouvert, debut) {
            (true, None) => debut = Some(ty),
            (false, Some(d)) => {
                liste.push(Ouverture {
                    x: tx * T,
                    haut: d * T,
                    bas: ty * T,
                });
                debut = None;
            }
            _ => {}
        }
    }
    liste
}

/// Les ouvertures des deux murs du bord (colonnes 0 et dernière).
fn ouvertures_du_bord(n: &Niveau) -> Vec<Ouverture> {
    let mut liste = ouvertures(n, 0);
    liste.extend(ouvertures(n, n.largeur() as i32 - 1));
    liste
}

/// L'ENTRÉE : l'ouverture d'un mur du bord à la hauteur des pieds de Pépin au
/// départ (le rideau de la cascade).
pub(crate) fn entree(n: &Niveau) -> Option<Ouverture> {
    let pieds = (n.depart.1 + JOUEUR_H - 1.0).floor() as i32;
    ouvertures_du_bord(n)
        .into_iter()
        .find(|o| (o.haut..o.bas).contains(&pieds))
}

/// La SORTIE : l'ouverture d'un mur du bord qui touche les rangées du drapeau, ou
/// les deux rangées au-dessus (le jour).
pub(crate) fn sortie(n: &Niveau) -> Option<Ouverture> {
    let a = n.arrivee?;
    let (haut, bas) = (a.y.round() as i32 - 2 * T, a.bas().round() as i32);
    ouvertures_du_bord(n)
        .into_iter()
        .find(|o| o.haut < bas && haut < o.bas)
}

/// Dessine l'habillage de la grotte (thème Grotte seulement), juste après les
/// tuiles. `(cx, cy)` : la caméra.
pub fn dessiner(monde: &Monde, cx: i32, cy: i32) {
    if monde.infos.theme != Theme::Grotte {
        return;
    }
    let (n, pas) = (&monde.niveau, monde.pas);
    if let Some(o) = entree(n) {
        dessiner_rideau(n, &o, pas, (cx, cy));
    }
    if let Some(o) = sortie(n) {
        dessiner_jour(n, &o, pas, (cx, cy));
    }
    for (k, s) in sources(n, &monde.entites).iter().enumerate() {
        let pas = pas + k as u32 * 13;
        match s.genre {
            GenreSource::Vasque => dessiner_vasque(s, pas, (cx, cy)),
            GenreSource::Fissure => dessiner_fissure(s, pas, (cx, cy)),
        }
    }
}

/// Un rai de lumière en pointillés : `nb` points depuis `(x, y)`, espacés de `pas_xy`,
/// sur le vide du niveau seulement (jamais sur une roche ni un sol : un rai ne se
/// lit pas comme un appui). Un point sur trois s'éteint à tour de rôle : la lumière
/// tremble à travers l'eau.
fn dessiner_rai(
    n: &Niveau,
    (x, y): (i32, i32),
    (dx, dy): (i32, i32),
    nb: i32,
    c: Couleur,
    pas: u32,
    (cx, cy): (i32, i32),
) {
    let tour = (pas / 8) as i32;
    for i in 0..nb {
        if (i + tour).rem_euclid(3) == 0 {
            continue;
        }
        let (px, py) = (x + i * dx, y + i * dy);
        if vide(n, px, py) {
            rect(px - cx, py - cy, 1, 1, c);
        }
    }
}

/// Le rideau d'eau de la cascade dans l'ouverture de l'entrée, et la lumière qui
/// le traverse.
fn dessiner_rideau(n: &Niveau, o: &Ouverture, pas: u32, (cx, cy): (i32, i32)) {
    let (x0, y0, h) = (o.x - cx, o.haut - cy, o.bas - o.haut);
    // Le jour derrière la chute : clair, plus pâle vers le bas (la brume).
    rect(x0, y0, T, h, NUAGE);
    rect(x0, y0 + h * 2 / 3, T, h - h * 2 / 3, BRUME);
    // L'eau : deux filets sur trois pleins ; entre eux, la lumière passe. Sur chaque
    // filet, des éclats qui dévalent (4 px par pas : la chute est rapide).
    for c in 0..T {
        let x = x0 + c;
        let plein = c % 3 != 1;
        if plein {
            rect(x, y0, 1, h, if c % 3 == 0 { PLUIE } else { PLUIE_SOMBRE });
        }
        let periode = 9 + (c * 5) % 7;
        let mut y = (pas as i32 * 4 + c * 13).rem_euclid(periode) - periode;
        while y < h {
            let (haut, bas) = (y.max(0), (y + 3).min(h));
            rect(
                x,
                y0 + haut,
                1,
                bas - haut,
                if plein { BLANC_CASSE } else { PLUIE },
            );
            y += periode;
        }
    }
    // L'eau passe le linteau : un bourrelet d'écume en haut.
    rect(x0, y0, T, 2, BLANC_CASSE);
    rect(x0, y0 + 2, T, 1, PLUIE);
    // Au pied, l'écume qui rebondit, et des gouttes projetées dans la grotte.
    let pied = o.bas - cy;
    for k in 0..5 {
        let saut = ((pas / 5) as i32 + k * 3).rem_euclid(6) / 2;
        rect(x0 + k * 3 + k % 2, pied - 2 - saut, 2, 1, BLANC_CASSE);
    }
    for k in 0..3 {
        let t = (pas as i32 + k * 11).rem_euclid(24);
        if t < 16 {
            let y = pied - 2 - t * (16 - t) / 8;
            rect(x0 + T + t + k * 2, y, 1, 1, PLUIE);
        }
    }
    // Les rais de lumière qui traversent le rideau et entrent dans la grotte.
    for r in 0..RAIS_RIDEAU {
        dessiner_rai(
            n,
            (o.x + T + 1, o.haut + 10 + r * 30),
            (2, 1),
            22,
            BRUME,
            pas + r as u32 * 5,
            (cx, cy),
        );
    }
}

/// Le jour qui se lève dans l'ouverture de la sortie : le ciel pâle, le soleil,
/// la rivière et sa rive d'herbes ocre ; des herbes sur le rocher près de
/// l'ouverture, des rais dorés qui entrent.
fn dessiner_jour(n: &Niveau, o: &Ouverture, pas: u32, (cx, cy): (i32, i32)) {
    let (x0, y0, h) = (o.x - cx, o.haut - cy, o.bas - o.haut);
    // Le ciel de l'aube, de haut en bas : brume, nuage, or pâle à l'horizon.
    let horizon = h - 12;
    rect(x0, y0, T, horizon, NUAGE);
    rect(x0, y0, T, horizon / 3, BRUME);
    rect(
        x0,
        y0 + horizon * 2 / 3,
        T,
        horizon - horizon * 2 / 3,
        OR_PALE,
    );
    // Le soleil qui se lève derrière la rive d'en face.
    rect(x0 + 8, y0 + horizon - 3, 5, 3, OR);
    rect(x0 + 9, y0 + horizon - 4, 3, 1, OR);
    // La rive d'en face, puis la rivière et ses reflets qui dérivent.
    rect(x0, y0 + horizon, T, 2, ARDOISE_PALE);
    rect(x0, y0 + horizon + 2, T, 6, PLUIE);
    let d = (pas / 20) as i32;
    rect(
        x0 + (2 + d).rem_euclid(12),
        y0 + horizon + 3,
        4,
        1,
        BLANC_CASSE,
    );
    rect(
        x0 + (9 + d).rem_euclid(13),
        y0 + horizon + 6,
        3,
        1,
        BLANC_CASSE,
    );
    // La rive de ce côté : des herbes ocre devant l'eau, qui se balancent.
    let pied = o.bas - cy;
    for i in 0..8 {
        let x = x0 + i * 2 + (i % 2);
        let haut = 3 + (i * 5) % 4;
        let penche = ((pas / 24) as i32 + i).rem_euclid(2);
        rect(x, pied - haut + 1, 1, haut - 1, ORANGE_SOLEIL);
        rect(x + penche, pied - haut, 1, 1, OR);
    }
    // Des herbes qui pendent du haut de l'ouverture.
    for i in 0..5 {
        rect(x0 + 1 + i * 3, y0, 1, 2 + (i * 3) % 3, ORANGE_SOLEIL);
    }
    // Sur le rocher, des herbes près de l'ouverture, de plus en plus rares vers
    // l'ombre ; seulement là où le dessus est un vrai sol (vide dessus, roche dessous).
    for k in 0..8 {
        let x = o.x - 2 - k * 5 - (k * 3) % 2;
        if vide(n, x, o.bas - 1) && !vide(n, x, o.bas) {
            let haut = 4 - k / 3;
            rect(x - cx, pied - haut + 1, 1, haut - 1, ORANGE_SOLEIL);
            rect(
                x - cx,
                pied - haut,
                1,
                1,
                if k < 4 { OR } else { ORANGE_SOLEIL },
            );
        }
    }
    // Les rais du jour, en biais vers le bas de la grotte.
    for r in 0..RAIS_JOUR {
        dessiner_rai(
            n,
            (o.x - 1, o.haut + 6 + r * 12),
            (-2, 1),
            30,
            OR_PALE,
            pas + r as u32 * 7,
            (cx, cy),
        );
    }
}

/// Une vasque qui bouillonne : un rebord de pierre de chaque côté (dans la roche,
/// au ras de la surface : le dessus reste plat), l'écume qui court sur l'eau, des
/// bouillons qui gonflent et crèvent en éclaboussant.
fn dessiner_vasque(s: &Source, pas: u32, (cx, cy): (i32, i32)) {
    let (x0, y) = (s.x - cx, s.y - cy);
    rect(x0 - 3, y, 3, 2, ARDOISE_PALE);
    rect(x0 - 2, y + 2, 2, 1, ARDOISE_PALE);
    rect(x0 + s.largeur, y, 3, 2, ARDOISE_PALE);
    rect(x0 + s.largeur, y + 2, 2, 1, ARDOISE_PALE);
    let d = (pas / 4) as i32;
    for i in (0..s.largeur).step_by(5) {
        rect(x0 + (i + d).rem_euclid(s.largeur), y + 1, 2, 1, BLANC_CASSE);
    }
    for (k, i) in (ECART_BOUILLONS / 2..s.largeur)
        .step_by(ECART_BOUILLONS as usize)
        .enumerate()
    {
        let x = x0 + i;
        match ((pas / IMAGE_BOUILLON_PAS) as i32 + k as i32 * 3).rem_euclid(7) {
            0 => rect(x - 1, y - 1, 3, 1, BLANC_CASSE),
            1 => {
                rect(x - 2, y - 2, 5, 2, PLUIE);
                rect(x - 1, y - 2, 1, 1, BLANC_CASSE);
            }
            2 => {
                rect(x - 1, y - 4, 3, 1, PLUIE);
                rect(x - 2, y - 3, 5, 3, PLUIE);
                rect(x - 1, y - 3, 1, 1, BLANC_CASSE);
            }
            3 => {
                // Le bouillon crève : trois éclaboussures.
                rect(x - 3, y - 4, 1, 1, BLANC_CASSE);
                rect(x + 3, y - 4, 1, 1, BLANC_CASSE);
                rect(x, y - 6, 1, 1, BLANC_CASSE);
                rect(x - 2, y - 1, 5, 1, BLANC_CASSE);
            }
            _ => {}
        }
    }
}

/// Une fissure au sommet de la roche, d'où sortent les bulles : la fente sombre
/// et ses branches SOUS l'arête du dessus (le sol reste entier : seule une bouche
/// de 2 px l'entaille), l'eau qui y luit, de petites bulles qui s'en échappent.
fn dessiner_fissure(s: &Source, pas: u32, (cx, cy): (i32, i32)) {
    let (x0, y) = (s.x - cx, s.y - cy);
    let l = s.largeur;
    rect(x0 + l / 2 - 1, y, 2, 2, ARDOISE_OMBRE);
    rect(x0 + l / 4, y + 2, l / 2, 2, ARDOISE_OMBRE);
    rect(x0 + l / 4 - 3, y + 4, 4, 1, ARDOISE_OMBRE);
    rect(x0 + l * 3 / 4 - 1, y + 4, 4, 1, ARDOISE_OMBRE);
    rect(x0 + l / 2 - 2, y + 4, 2, 4, ARDOISE_OMBRE);
    rect(x0 + l / 2 + 3, y + 5, 1, 4, ARDOISE_NUIT);
    // L'eau qui luit au fond de la fente (elle bat comme une source).
    let lueur = if (pas / 10).is_multiple_of(3) {
        PLUIE
    } else {
        PLUIE_SOMBRE
    };
    rect(x0 + l / 4 + 2, y + 2, l / 2 - 4, 1, lueur);
    rect(x0 + l / 2 - 1, y + 1, 2, 1, lueur);
    // De petites bulles qui sortent de la fente et montent.
    for k in 0..4 {
        let t = (pas as i32 + k * 7).rem_euclid(28);
        let bx = x0 + l / 4 + 2 + k * (l / 8) + ((t / 6) % 2);
        let by = y - 1 - t / 2;
        rect(bx, by, 2, 1, PLUIE);
        rect(bx, by, 1, 1, BLANC_CASSE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pepin_core::aabb::Grille;
    use pepin_core::levels::grotte;

    fn niveau() -> Niveau {
        Niveau::depuis_ascii(grotte::INFOS.ascii).expect("grotte")
    }

    /// Chaque colonne de bulles a sa source : deux vasques d'eau (au pied de C1 et
    /// de C3) et deux fissures dans la roche (sous C2 et C4). Une vasque est DANS sa
    /// colonne (on y est soulevé, jamais noyé en la traversant) et les bulles
    /// partent de son fond ; une fissure est au sommet d'une roche.
    #[test]
    fn chaque_colonne_de_bulles_jaillit_d_une_source() {
        let n = niveau();
        let colonnes: Vec<Aabb> = n
            .entites
            .iter()
            .filter_map(|e| match e {
                Entite::Bulles(b) => Some(b.zone),
                _ => None,
            })
            .collect();
        let s = sources(&n, &n.entites);
        assert_eq!(s.len(), colonnes.len(), "une source par colonne : {s:?}");
        assert_eq!(colonnes.len(), 4);
        let vasques = s.iter().filter(|s| s.genre == GenreSource::Vasque).count();
        assert_eq!(vasques, 2, "{s:?}");
        for (s, z) in s.iter().zip(&colonnes) {
            let (tx, ty) = (s.x / T, s.y / T);
            assert_eq!((s.x, s.largeur), (z.x as i32, z.w as i32));
            match s.genre {
                GenreSource::Vasque => {
                    for x in tx..tx + s.largeur / T {
                        assert_eq!(n.tuile_decor(x, ty), Tuile::Eau, "vasque ({x}, {ty})");
                        assert_eq!(n.tuile_decor(x, ty - 1), Tuile::Vide, "dessus ({x})");
                    }
                    let fond = z.bas() as i32 / T;
                    assert!(
                        n.tuile_decor(tx, fond).est_solide(),
                        "les bulles partent du fond"
                    );
                }
                GenreSource::Fissure => {
                    assert!(n.tuile_decor(tx, ty).est_solide(), "fissure ({tx}, {ty})");
                    assert_eq!(s.y, z.bas() as i32, "la colonne sort de la roche");
                }
            }
        }
    }

    /// L'entrée est au pied, à gauche, à la hauteur du départ (le rideau de la
    /// cascade) ; la sortie, en haut à droite, au-dessus du rocher du drapeau.
    /// Aucune autre ouverture dans les murs, et le bord de la carte reste un mur :
    /// on ne sort pas par les ouvertures.
    #[test]
    fn entree_par_le_rideau_sortie_au_jour() {
        let n = niveau();
        let e = entree(&n).expect("entrée");
        assert_eq!(e.x, 0);
        assert!(e.bas - e.haut >= 4 * T, "un vrai rideau : {e:?}");
        let s = sortie(&n).expect("sortie");
        assert_eq!(s.x, (n.largeur() as i32 - 1) * T);
        let a = n.arrivee.expect("drapeau");
        assert_eq!(s.bas, a.bas() as i32, "le jour au ras du rocher du drapeau");
        assert_eq!(ouvertures_du_bord(&n), vec![e, s]);
        for o in [e, s] {
            let tx = o.x / T + if o.x == 0 { -1 } else { 1 };
            for ty in o.haut / T..o.bas / T {
                assert!(n.est_solide(tx, ty), "hors carte en ({tx}, {ty}) : un mur");
            }
        }
    }
}
