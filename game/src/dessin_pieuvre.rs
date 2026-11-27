//! Rendu de la Pieuvre du port, boss du Monde 2 (carte 9.12a) : ses tentacules
//! (annonce, dressés, qui redescendent), sa tête (qui remonte, sortie, touchée,
//! qui replonge en lâchant une clé) et l'encre quand elle a eu Pépin.
//!
//! La règle du jeu est visible : AUCUN tentacule ne se dresse sans son annonce. La
//! 1re moitié de l'annonce (0,4 s) : l'ombre de la pieuvre sous la surface et des
//! remous sur toute la largeur de la colonne ; la 2e moitié : le bout du tentacule
//! pointe hors de l'eau. Tout est en sprites de la palette (`sprites::pieuvre`).
//!
//! La saisie (carte 9.12f) : un tentacule monte de l'eau et s'enroule sur la
//! plateforme, qui tremble (l'annonce) ; elle s'enfonce, découpée à la surface ;
//! sous l'eau, un cercle d'écume marque sa place ; puis elle remonte. Son eau
//! est la sienne (carte 9.12g) : bulles, cercle de remous, tourbillon, jamais les
//! remous ni l'ombre d'un tentacule d'attaque.

use pepin_core::aabb::TAILLE_TUILE;
use pepin_core::entities::pieuvre::{
    EtatSaisie, EtatTentacule, PhasePieuvre, Pieuvre, LARGEUR_COLONNE, MARGE_TENTACULE, NB_COLONNES,
};
use pepin_core::entities::Entite;
use pepin_core::palette::{ARDOISE_OMBRE, BLANC_CASSE, NUAGE, PLUIE, PLUIE_SOMBRE};

use crate::monde::{Monde, ENCRE_PAS};
use crate::rendu::couleur_mq;
use crate::sprites::Banque;
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

/// Hauteur (px) d'un tronçon de tentacule (`tentacule0`).
const TRONCON: f32 = 8.0;
/// Hauteur (px) du bout recourbé (`tentacule_bout`).
const BOUT: f32 = 16.0;

/// Image d'une animation à deux images de `duree` pas chacune.
fn image(anim: [&'static str; 2], pas: u32, duree: u32) -> &'static str {
    anim[(pas / duree) as usize % 2]
}

/// Hauteur (px) du tentacule hors de l'eau, de 0 à `pleine` : rien au calme ni
/// pendant l'annonce (le bout qui pointe a son sprite), tout dressé, puis il
/// redescend en 12 pas. Pure.
fn hauteur_tentacule(etat: EtatTentacule, pleine: f32) -> f32 {
    match etat {
        EtatTentacule::Dresse => pleine,
        EtatTentacule::Redescente(p) => (pleine * (1.0 - p)).round(),
        EtatTentacule::Calme | EtatTentacule::Annonce(_) => 0.0,
    }
}

/// Dessine la pieuvre `p` (caméra `cx`, `cy`), au pas `pas` du monde.
pub fn dessiner(p: &Pieuvre, banque: &Banque, pas: u32, cx: i32, cy: i32) {
    let (cx, cy) = (cx as f32, cy as f32);
    for i in 0..NB_COLONNES {
        let corps = p.colonnes[i];
        let surface = corps.bas() - cy;
        let gauche = corps.x - MARGE_TENTACULE - cx;
        match p.etat_tentacule(i) {
            EtatTentacule::Calme => {}
            EtatTentacule::Annonce(a) => {
                banque.dessiner("ombre_pieuvre", gauche, surface + 1.0, false);
                banque.dessiner(
                    image(["remous_pieuvre0", "remous_pieuvre1"], pas, 6),
                    gauche,
                    surface - 5.0,
                    false,
                );
                if a >= 0.5 {
                    let nom = image(["tentacule_pointe0", "tentacule_pointe1"], pas, 8);
                    banque.dessiner(nom, corps.x - cx, surface - 11.0, false);
                }
            }
            etat => {
                let h = hauteur_tentacule(etat, corps.h);
                dessiner_tentacule(banque, corps.x - cx, surface, h, pas);
                banque.dessiner(
                    image(["remous_pieuvre0", "remous_pieuvre1"], pas, 6),
                    gauche,
                    surface - 5.0,
                    false,
                );
            }
        }
    }
    dessiner_tete(p, banque, pas, cx, cy);
}

/// La bâche de l'arène (carte 9.13a), tant qu'elle tient : une rangée de toiles
/// (les tuiles de toit du thème, en attendant son dessin : carte 9.13b).
pub fn dessiner_bache(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    let Some(bache) = la_pieuvre(monde).and_then(|p| p.bache_solide()) else {
        return;
    };
    let mut x = bache.x;
    while x < bache.droite() {
        let (sx, sy) = (x - cx as f32, bache.y - cy as f32);
        banque.dessiner_tuile(monde.infos.theme, "toit_haut", sx, sy);
        x += TAILLE_TUILE;
    }
}

/// La mer qui monte (carte 9.13a), devant la tour : les tuiles d'eau du thème
/// jusqu'au bas de l'écran, la surface qui ondule (carte 9.36a : plus de
/// paliers, donc plus de graduation aux murs). La trombe de la furie : des
/// bulles qui montent.
pub fn dessiner_mer(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    let Some(p) = la_pieuvre(monde) else {
        return;
    };
    let t = TAILLE_TUILE as i32;
    let mer = p.surface_mer().map(|m| m.round() as i32);
    if let Some(z) = p.trombe().map(|c| c.zone) {
        // Carte 9.13f (AGA : « des crêtes d'écume en escalier dans le vide ») :
        // dans l'air, la trombe est faite de BULLES (petits carrés) ; l'écume
        // (les remous) ne se dessine qu'à la surface de la mer, où elle bout.
        let h = z.h as i32;
        let surface = mer.unwrap_or(i32::MAX);
        for k in 0..(h / 10).max(1) {
            let monte = (monde.pas as i32 * 3 + k * 10).rem_euclid(h);
            let x = z.x as i32 + (k * 13).rem_euclid(z.w as i32 - 6);
            let y = z.bas() as i32 - monte;
            if y + 2 < surface {
                let c = if k % 2 == 0 { BLANC_CASSE } else { NUAGE };
                let (bx, by) = ((x + 2 - cx) as f32, (y - cy) as f32);
                macroquad::prelude::draw_rectangle(bx, by, 2.0, 2.0, couleur_mq(c));
                macroquad::prelude::draw_rectangle(bx, by + 2.0, 2.0, 1.0, couleur_mq(PLUIE));
            }
        }
        if surface < z.bas() as i32 && surface > z.y as i32 {
            for (k, dx) in [0, (z.w as i32 - 16) / 2, z.w as i32 - 16]
                .into_iter()
                .enumerate()
            {
                let nom = image(
                    ["remous_pieuvre0", "remous_pieuvre1"],
                    monde.pas + k as u32 * 3,
                    6,
                );
                let x = z.x as i32 + dx;
                banque.dessiner(nom, (x - cx) as f32, (surface - 5 - cy) as f32, false);
            }
        }
    }
    let Some(mer) = p.surface_mer() else {
        return;
    };
    let surface = mer.round() as i32;
    let bas = cy + HAUTEUR_LOGIQUE as i32;
    let largeur = monde.niveau.largeur_px() as i32;
    for x in (t..largeur - t).step_by(t as usize) {
        let mut y = surface + t;
        while y < bas {
            banque.dessiner("jetee_eau_plein", (x - cx) as f32, (y - cy) as f32, false);
            y += t;
        }
        let vague = (monde.pas / 10) as i32 + x / t;
        let nom = if vague.rem_euclid(2) == 1 {
            "jetee_eau_vague"
        } else {
            "jetee_eau_haut"
        };
        banque.dessiner(nom, (x - cx) as f32, (surface - cy) as f32, false);
    }
}

/// Un tentacule de `h` px dressé au-dessus de `surface` (à l'écran) : des tronçons
/// empilés depuis l'eau, puis le bout recourbé en haut (dessiné en dernier, il couvre
/// le tronçon qui dépasse).
pub(crate) fn dessiner_tentacule(banque: &Banque, x: f32, surface: f32, h: f32, pas: u32) {
    if h <= 0.0 {
        return;
    }
    if h < BOUT {
        // Il finit de replonger : seul le haut du bout dépasse encore.
        banque.dessiner_haut("tentacule_bout", x, surface - h, h);
        return;
    }
    let haut = surface - h;
    let n = ((h - BOUT) / TRONCON).ceil() as u32;
    for k in 1..=n {
        // Les ventouses glissent d'un tronçon à l'autre : le tentacule ondule.
        let nom = image(["tentacule0", "tentacule1"], pas / 4 + k, 1);
        banque.dessiner(nom, x, surface - TRONCON * k as f32, false);
    }
    banque.dessiner("tentacule_bout", x, haut, false);
}

/// La tête : remous à sa place quand elle remonte, puis la tête posée sur l'eau ;
/// touchée, elle clignote ; vaincue, elle replonge et la clé monte de l'eau.
fn dessiner_tete(p: &Pieuvre, banque: &Banque, pas: u32, cx: f32, cy: f32) {
    let Some(tete) = p.tete() else {
        return;
    };
    let (x, y) = (tete.x - cx, tete.y - cy);
    let surface = tete.bas() - cy;
    let remous = image(["remous_pieuvre0", "remous_pieuvre1"], pas, 6);
    match p.phase() {
        PhasePieuvre::Remous => {
            banque.dessiner("ombre_pieuvre", x + 8.0, surface + 1.0, false);
            for dx in [0.0, LARGEUR_COLONNE / 2.0] {
                banque.dessiner(remous, x + dx, surface - 5.0, false);
            }
        }
        // Phase 2 (carte 9.36a) : de la furie (elle se retire au large) à sa
        // fuite, elle est dessinée par `dessin_montee` (`Pieuvre::au_loin`).
        PhasePieuvre::Furie => {}
        PhasePieuvre::Montee | PhasePieuvre::Plongee | PhasePieuvre::Vaincue if p.en_fuite() => {}
        PhasePieuvre::Tete => {
            let nom = image(["pieuvre_colere0", "pieuvre_colere1"], pas, 20);
            if p.est_facile() {
                banque.dessiner_pale(nom, x, y);
            } else {
                banque.dessiner(nom, x, y, false);
            }
        }
        // Touchée : 4 images visibles, 4 cachées, comme le Grand Nuage.
        PhasePieuvre::Touchee if (p.ecoule() / 4).is_multiple_of(2) => {
            banque.dessiner("pieuvre_touchee", x, y, false);
        }
        PhasePieuvre::Plongee => {
            let k = p.progression_plongee();
            let descente = (tete.h * k).round();
            banque.dessiner_haut("pieuvre_vaincue", x, y + descente, tete.h - descente);
            for dx in [0.0, LARGEUR_COLONNE / 2.0] {
                banque.dessiner(remous, x + dx, surface - 5.0, false);
            }
            // La clé sort de l'eau et monte de 40 px au-dessus d'elle.
            let montee = (40.0 * (k * 2.0).min(1.0)).round();
            banque.dessiner("cle_pieuvre", x + 17.0, surface - 8.0 - montee, false);
        }
        _ => {}
    }
}

/// L'encre de la Pieuvre (carte 9.12a) : l'écran s'assombrit brièvement quand elle a
/// eu Pépin. Sans transparence (palette fixe) : des rangées d'ardoise sombre, une
/// sur quatre au début et à la fin, une sur deux au milieu.
pub fn dessiner_encre(monde: &Monde) {
    if monde.encre == 0 {
        return;
    }
    let ecoule = ENCRE_PAS - monde.encre;
    let pas_rangee = if ecoule < ENCRE_PAS / 3 || monde.encre < ENCRE_PAS / 3 {
        4
    } else {
        2
    };
    let c = couleur_mq(ARDOISE_OMBRE);
    for y in (0..HAUTEUR_LOGIQUE).step_by(pas_rangee) {
        macroquad::prelude::draw_rectangle(0.0, y as f32, LARGEUR_LOGIQUE as f32, 1.0, c);
    }
}

/// La pieuvre du niveau, s'il y en a une.
pub(crate) fn la_pieuvre(monde: &Monde) -> Option<&Pieuvre> {
    monde.entites.iter().find_map(|e| match e {
        Entite::Pieuvre(p) => Some(p.as_ref()),
        _ => None,
    })
}

/// La case `(tx, ty)` appartient-elle à la plateforme saisie en ce moment ? Le
/// jeu de tuiles ne la dessine pas : `dessiner_saisie` s'en charge.
pub fn case_saisie(monde: &Monde, tx: i32, ty: i32) -> bool {
    la_pieuvre(monde)
        .and_then(|p| {
            p.saisie()
                .map(|s| p.plateformes[s.cible].contains(&(tx, ty)))
                // Le quai cassé de la phase 2 (carte 9.13a) : tombé à l'eau.
                .or(Some(p.case_engloutie(tx, ty)))
        })
        .unwrap_or(false)
}

/// Enfoncement (px) de la plateforme saisie sous sa place, de 0 à `fond` (elle
/// est alors tout entière sous la surface), et son tremblement (px). Pure.
fn enfoncement(etat: EtatSaisie, fond: f32, pas: u32) -> (f32, f32) {
    match etat {
        // Le tremblement grandit avec l'annonce : 1 px, puis 2 px à la fin.
        EtatSaisie::Annonce(a) => {
            let ampleur = if a < 0.5 { 1.0 } else { 2.0 };
            (0.0, [0.0, ampleur, 0.0, -ampleur][(pas / 2 % 4) as usize])
        }
        EtatSaisie::Plongee(p) => ((fond * p).round(), 0.0),
        EtatSaisie::SousLEau => (fond, 0.0),
        EtatSaisie::Remontee(p) => ((fond * (1.0 - p)).round(), 0.0),
    }
}

/// La plateforme que la Pieuvre emporte (carte 9.12f), après l'eau du bassin :
/// ses tuiles (qui tremblent, puis coulent, découpées à la surface), le
/// tentacule qui monte de l'eau et s'enroule sur son dessus, les remous.
pub fn dessiner_saisie(monde: &Monde, banque: &Banque, cx: i32, cy: i32) {
    let Some(p) = la_pieuvre(monde) else {
        return;
    };
    let Some(saisie) = p.saisie() else {
        return;
    };
    let t = TAILLE_TUILE;
    let (cx, cy) = (cx as f32, cy as f32);
    let cases = &p.plateformes[saisie.cible];
    // La surface du bassin : le bas des colonnes de tentacules.
    let surface = p.colonnes[0].bas();
    let haut = cases.iter().map(|c| c.1).min().unwrap_or(0) as f32 * t;
    let (g, d) = (
        cases.iter().map(|c| c.0).min().unwrap_or(0) as f32 * t,
        (cases.iter().map(|c| c.0).max().unwrap_or(0) + 1) as f32 * t,
    );
    let etat = saisie.etat();
    let (dy, dx) = enfoncement(etat, surface - haut, monde.pas);
    // L'eau d'abord : la plateforme, dessinée par-dessus, cache le fond du cercle.
    let bassin = (
        p.colonnes[0].x - MARGE_TENTACULE,
        p.colonnes[NB_COLONNES - 1].droite() + MARGE_TENTACULE,
    );
    dessiner_eau_saisie(banque, etat, (g, d), bassin, (-cx, surface - cy), monde.pas);
    // Les tuiles, et l'objet posé dessus : seulement ce qui dépasse de l'eau.
    for &(tx, ty) in cases {
        let (nom, objet) = crate::scene::noms_de_la_case(monde, tx, ty);
        let (x, y) = (tx as f32 * t + dx, ty as f32 * t + dy);
        let reste = surface - y;
        banque.dessiner_tuile_haut(monde.infos.theme, nom, x - cx, y - cy, reste);
        if let Some(objet) = objet {
            banque.dessiner_tuile_haut(monde.infos.theme, objet, x - cx, y - t - cy, reste + t);
        }
    }
    // Le tentacule : il sort au milieu de la plateforme, monte pendant la 1re
    // moitié de l'annonce, s'enroule sur le dessus ensuite et l'entraîne.
    let milieu = ((g + d) / 2.0 - 12.0).round();
    let tirage = match etat {
        EtatSaisie::Annonce(a) => (a * 2.0).min(1.0),
        EtatSaisie::Plongee(_) => 1.0,
        _ => 0.0,
    };
    if tirage > 0.0 {
        let plein = surface - haut + 4.0;
        let h = (plein * tirage).round() - dy;
        dessiner_tentacule(banque, milieu - cx, surface - cy, h, monde.pas);
        if tirage >= 1.0 {
            let nom = image(["tentacule_enroule0", "tentacule_enroule1"], monde.pas, 8);
            let mut x = g;
            while x < d {
                let y = haut - 3.0 + dy;
                banque.dessiner_haut(nom, x + dx - cx, y - cy, surface - y);
                x += t;
            }
        }
    }
}

/// L'effet d'eau d'une saisie (carte 9.12g, AGA : « la même gerbe d'eau qu'une
/// attaque normale ») : le sien, posé à la surface, jamais les remous ni l'ombre
/// d'un tentacule d'attaque (dont les 2 rangées cyan redoublaient l'eau du
/// bassin). `(g, d)` : bords de la plateforme, `bassin` : bords de l'eau (monde) ;
/// `(sx, sy)` : décalage de la caméra en x et surface à l'écran.
pub(crate) fn dessiner_eau_saisie(
    banque: &Banque,
    etat: EtatSaisie,
    (g, d): (f32, f32),
    bassin: (f32, f32),
    (sx, sy): (f32, f32),
    pas: u32,
) {
    let (w, milieu) = (d - g, (g + d) / 2.0);
    match etat {
        // Des bulles remontent, un cercle de remous s'élargit autour de son pied
        // (de 4 à 12 px au-delà de ses bords : jamais jusqu'à l'appui voisin).
        EtatSaisie::Annonce(a) => {
            let r = w / 2.0 + 4.0 + 4.0 * (a * 3.0).floor().min(2.0);
            dessiner_onde(banque, (milieu - r, milieu + r), bassin, sx, sy);
            dessiner_bulles(banque, &[g + 3.0, milieu - 2.0, d - 7.0], sx, sy, pas);
        }
        // Un tourbillon la creuse et l'avale, l'écume aux bords ; pas de gerbe.
        EtatSaisie::Plongee(_) => {
            dessiner_creux((g, d), sx, sy, pas);
            if g - 6.0 >= bassin.0 {
                banque.dessiner("bord_creux_saisie", g - 6.0 + sx, sy - 3.0, false);
            }
            if d + 6.0 <= bassin.1 {
                banque.dessiner("bord_creux_saisie", d + sx, sy - 3.0, true);
            }
        }
        // Sa place sous l'eau : un cercle calme, des bulles.
        EtatSaisie::SousLEau => {
            let r = w / 2.0 + 4.0;
            dessiner_onde(banque, (milieu - r, milieu + r), bassin, sx, sy);
            dessiner_bulles(banque, &[g + 5.0, d - 9.0], sx, sy, pas);
        }
        // Elle remonte : quelques bulles.
        EtatSaisie::Remontee(_) => {
            dessiner_bulles(banque, &[milieu - 2.0, d - 5.0], sx, sy, pas);
        }
    }
}

/// Le creux du tourbillon qui avale la plateforme, de `g` à `d` (monde) : la ligne
/// claire de la surface plonge en cuvette (3 px au plus au milieu) au-dessus d'un
/// creux d'ardoise d'ombre, un filet cyan sombre dessous, et des traits d'écume
/// qui glissent le long de la ligne (l'eau tourne). Une seule couche, sous la
/// surface : la plateforme découpée passe par-dessus.
fn dessiner_creux((g, d): (f32, f32), sx: f32, sy: f32, pas: u32) {
    let (creux, ligne, filet, ecume) = (
        couleur_mq(ARDOISE_OMBRE),
        couleur_mq(PLUIE),
        couleur_mq(PLUIE_SOMBRE),
        couleur_mq(BLANC_CASSE),
    );
    let n = (d - g).max(1.0) as u32;
    for i in 0..n {
        let x = g + i as f32 + sx;
        let t = (i as f32 + 0.5) / n as f32;
        let fond = (3.0 * (std::f32::consts::PI * t).sin()).round();
        if fond > 0.0 {
            macroquad::prelude::draw_rectangle(x, sy, 1.0, fond, creux);
        }
        let mousse = (i + pas / 3) % 8 < 2;
        let c = if mousse { ecume } else { ligne };
        macroquad::prelude::draw_rectangle(x, sy + fond, 1.0, 1.0, c);
        macroquad::prelude::draw_rectangle(x, sy + fond + 1.0, 1.0, 1.0, filet);
    }
}

/// Le cercle de remous de la saisie, vu de côté : un anneau d'écume à plat sur la
/// surface, de `g` à `d` (monde), rogné aux bords du `bassin` (un bout rogné n'a
/// pas sa courbe). Le trait du fond passe derrière la plateforme (dessinée
/// ensuite), celui de devant dans l'eau. Que de l'écume : aucune bande d'eau.
fn dessiner_onde(banque: &Banque, (g, d): (f32, f32), bassin: (f32, f32), sx: f32, sy: f32) {
    let (g, d) = (g.round(), d.round());
    let (mut a, mut b) = (g.max(bassin.0), d.min(bassin.1));
    if g >= bassin.0 {
        banque.dessiner("onde_saisie", g + sx, sy - 2.0, false);
        a = g + 4.0;
    }
    if d <= bassin.1 {
        banque.dessiner("onde_saisie", d - 4.0 + sx, sy - 2.0, true);
        b = d - 4.0;
    }
    if b <= a {
        return;
    }
    let (ecume, contour) = (couleur_mq(BLANC_CASSE), couleur_mq(ARDOISE_OMBRE));
    for (dy, c) in [(-2.0, contour), (-1.0, ecume), (1.0, ecume), (2.0, contour)] {
        macroquad::prelude::draw_rectangle(a + sx, sy + dy, b - a, 1.0, c);
    }
}

/// Des bulles qui remontent de 12 px sous la surface `sy` (à l'écran) jusqu'à
/// elle et y crèvent, une par abscisse de `xs` (monde, décalées de `sx`), chacune
/// à son rythme. Pure animation : rien ne dépend du hasard.
fn dessiner_bulles(banque: &Banque, xs: &[f32], sx: f32, sy: f32, pas: u32) {
    const MONTEE: u32 = 24;
    for (k, &x) in xs.iter().enumerate() {
        let phase = (pas + 9 * k as u32) % MONTEE;
        let y = sy + 12.0 - (phase * 11 / MONTEE) as f32;
        // Elles oscillent d'un pixel en montant.
        let ecart = if (phase / 6).is_multiple_of(2) {
            0.0
        } else {
            1.0
        };
        banque.dessiner("bulle_saisie", x + ecart + sx, y, false);
    }
}
