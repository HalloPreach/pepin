//! Écran de la carte des niveaux (carte 7.4) : à gauche, la carte verticale qui monte
//! des toits jusqu'au soleil (elle défile) ; à droite, le panneau de l'étape choisie
//! (nom, état, meilleur temps, gouttes trouvées, médaille, défis cochés, JOUER).
//! La logique (déblocage, appuis) est dans `crate::plan` ; ici on ne fait que
//! dessiner et dire où sont les zones cliquables (`MiseEnPage`).
//!
//! Mise en page 320 x 180 (pixels logiques) : bandeau du titre (y 0 à 14), vue de la
//! carte (y 14 à 166, x 0 à 176), panneau (x 180 à 316), bandeau du bas (RETOUR et
//! rappel des touches). La carte est dessinée en entier puis recouverte par les
//! bandeaux et le panneau : rien ne déborde (pas de découpage à faire).

use pepin_core::font::largeur_texte;
use pepin_core::levels::{nb_normaux, GenreNiveau, NIVEAUX};
use pepin_core::palette::{
    Couleur, ARDOISE, ARDOISE_CLAIRE, ARDOISE_NUIT, ARDOISE_OMBRE, ARDOISE_PALE, BLANC_CASSE,
    BRUME, CORAIL, CORAIL_CLAIR, NUAGE, OR, ORANGE_SOLEIL, OR_PALE, PLUIE, VIOLET_ORAGE,
};
use pepin_core::zones::Theme;
use pepin_core::LARGEUR_LOGIQUE;

use super::carte::titre_niveau;
use super::resultats::{case, grille, icone_medaille, medaille, ECART_ICONE, L_ICONE};
use super::{image, panneau, rect};
use crate::chrono;
use crate::contenu;
use crate::defis::defis_du_niveau;
use crate::plan::{
    self, etat_etape, gouttes_trouvees_normales, EtatEtape, EtatPlan, Geometrie, Zones, H_TOITS,
    H_VUE, Y_SOLEIL,
};
use crate::sauvegarde::Sauvegarde;
use crate::skins::Skin;
use crate::sprites::Banque;
use crate::texte::{self, couper_lignes, Style};
use pepin_core::levels::gouttes_pour_le_bonus;

/// Haut de la vue de la carte à l'écran.
const Y_VUE: f32 = 14.0;
/// Largeur de la carte à l'écran.
const L_CARTE: f32 = 176.0;
/// Panneau de l'étape choisie, et son bouton JOUER.
const PANNEAU: (f32, f32, f32, f32) = (180.0, 16.0, 136.0, 148.0);
const BOUTON_JOUER: (f32, f32, f32, f32) = (188.0, 140.0, 120.0, 20.0);
/// Bouton RETOUR, dans le bandeau du bas.
const BOUTON_RETOUR: (f32, f32, f32, f32) = (4.0, 166.0, 62.0, 13.0);
/// Demi-côté d'une étape, et demi-côté de sa zone de toucher (plus large).
const DEMI: f32 = 7.0;
const DEMI_TOUCHER: f32 = 11.0;
/// Largeur du texte du panneau, en caractères (128 px / 6).
const LIGNE_PANNEAU: usize = 21;

const _: () = {
    assert!(Y_VUE + H_VUE <= BOUTON_RETOUR.1);
    assert!(BOUTON_JOUER.1 + BOUTON_JOUER.3 <= PANNEAU.1 + PANNEAU.3 - 2.0);
    assert!(L_CARTE < PANNEAU.0);
};

/// Ciel de la carte, de haut en bas : le soleil doré au sommet, l'orage au pied.
const CIEL: [Couleur; 9] = [
    OR_PALE,
    OR,
    ORANGE_SOLEIL,
    CORAIL_CLAIR,
    CORAIL,
    VIOLET_ORAGE,
    ARDOISE,
    ARDOISE_NUIT,
    ARDOISE_OMBRE,
];

/// Cadenas d'une étape verrouillée : `#` contour, `o` corps.
const CADENAS: [&str; 7] = [
    "..###..", ".#...#.", ".#...#.", "#######", "#ooooo#", "#oo#oo#", "#######",
];

fn dedans((rx, ry, rl, rh): (f32, f32, f32, f32), x: f32, y: f32) -> bool {
    x >= rx && x < rx + rl && y >= ry && y < ry + rh
}

/// Zones cliquables de la carte.
pub struct MiseEnPage;

impl Zones for MiseEnPage {
    fn etape_sous(&self, g: &Geometrie, defil: f32, x: f32, y: f32) -> Option<usize> {
        if x >= L_CARTE || !(Y_VUE..Y_VUE + H_VUE).contains(&y) {
            return None;
        }
        g.etapes.iter().position(|&(ex, ey)| {
            let ey = Y_VUE + ey - defil;
            (x - ex).abs() <= DEMI_TOUCHER && (y - ey).abs() <= DEMI_TOUCHER
        })
    }

    fn sur_jouer(&self, x: f32, y: f32) -> bool {
        dedans(BOUTON_JOUER, x, y)
    }

    fn sur_retour(&self, x: f32, y: f32) -> bool {
        dedans(BOUTON_RETOUR, x, y)
    }
}

/// Dessine la carte des niveaux. `anim` : pas écoulés depuis son ouverture.
pub fn dessiner(banque: &Banque, skin: Skin, anim: u32, s: &Sauvegarde, e: &EtatPlan) {
    let g = &e.geometrie;
    // Ordonnée écran d'une ordonnée de la carte.
    let ecran = |y: f32| (Y_VUE + y - e.defil).round();
    dessiner_ciel(g, &ecran);
    dessiner_soleil(ecran(Y_SOLEIL));
    dessiner_nuages(g, &ecran);
    dessiner_pluie(g, &ecran, anim);
    let y_toits = ecran(g.hauteur - H_TOITS);
    for k in 0..=(L_CARTE as i32 / 16) {
        let x = k as f32 * 16.0;
        banque.dessiner_tuile(Theme::Toits, "toit_haut", x, y_toits);
        banque.dessiner_tuile(Theme::Toits, "toit_plein", x, y_toits + 16.0);
    }
    dessiner_chemins(g, s, &ecran);
    for (i, &(x, y)) in g.etapes.iter().enumerate() {
        dessiner_etape(i, x, ecran(y), s, i == e.selection, anim);
    }
    // Pépin (skin actif) debout sur l'étape choisie.
    if let Some(&(x, y)) = g.etapes.get(e.selection) {
        let pepin = image(&skin.images().repos, anim, 30);
        banque.dessiner(pepin, x - 8.0, ecran(y) - DEMI - 16.0, false);
    }

    // Bandeaux et panneau par-dessus : la carte ne déborde nulle part.
    let l = LARGEUR_LOGIQUE as f32;
    rect(0.0, 0.0, l, Y_VUE, ARDOISE_OMBRE);
    rect(0.0, Y_VUE + H_VUE, l, 180.0 - Y_VUE - H_VUE, ARDOISE_OMBRE);
    rect(L_CARTE, Y_VUE, l - L_CARTE, H_VUE, ARDOISE_OMBRE);
    let ombre = |c| Style::simple(c, 1).avec_ombre(ARDOISE_OMBRE);
    texte::dessiner_centre(contenu::TITRE_PLAN, 4.0, ombre(OR_PALE));
    banque.dessiner("goutte0", 4.0, 2.0, false);
    texte::dessiner(
        &s.gouttes_en_poche.to_string(),
        16.0,
        4.0,
        ombre(BLANC_CASSE),
    );
    dessiner_panneau(banque, s, e.selection, anim);
    let (x, y, w, h) = BOUTON_RETOUR;
    panneau(x, y, w, h, ARDOISE_NUIT, BRUME);
    texte::dessiner_centre_sur(contenu::RETOUR, x + w / 2.0, y + 3.0, ombre(BRUME));
    texte::dessiner(
        contenu::AIDE_PLAN,
        x + w + 8.0,
        y + 3.0,
        Style::simple(BRUME, 1),
    );
}

/// Le ciel en bandes, du soleil (haut) à l'orage (bas), avec un damier entre deux
/// bandes (pas de dégradé : la palette n'en a pas).
fn dessiner_ciel(g: &Geometrie, ecran: &impl Fn(f32) -> f32) {
    let h_bande = g.hauteur / CIEL.len() as f32;
    for (i, &c) in CIEL.iter().enumerate() {
        let y0 = ecran(i as f32 * h_bande);
        let y1 = ecran((i + 1) as f32 * h_bande);
        if y1 < Y_VUE || y0 > Y_VUE + H_VUE {
            continue;
        }
        rect(0.0, y0, L_CARTE, y1 - y0 + 1.0, c);
        // Damier de la bande suivante sur les 4 dernières rangées.
        if let Some(&suivante) = CIEL.get(i + 1) {
            for r in 0..4 {
                let y = y1 - 4.0 + r as f32;
                let depart = (r % 2) as usize;
                for x in (depart..L_CARTE as usize).step_by(if r < 2 { 4 } else { 2 }) {
                    rect(x as f32, y, 1.0, 1.0, suivante);
                }
            }
        }
    }
}

/// Le soleil au sommet de la carte, en rangées de pixels : un halo orange (le ciel
/// du sommet est déjà OR_PALE), un disque d'or, un cœur blanc.
fn dessiner_soleil(cy: f32) {
    let cx = 120.0;
    for (r, c) in [(19i32, ORANGE_SOLEIL), (16, OR), (10, BLANC_CASSE)] {
        for dy in -r..=r {
            let w = ((r * r - dy * dy) as f32).sqrt() as i32;
            rect(cx - w as f32, cy + dy as f32, (2 * w + 1) as f32, 1.0, c);
        }
    }
}

/// Une petite rangée de nuages vers le haut de la carte (la mer de nuages).
fn dessiner_nuages(g: &Geometrie, ecran: &impl Fn(f32) -> f32) {
    for (fx, fy, r) in [
        (20.0, 0.28, 9),
        (70.0, 0.31, 12),
        (130.0, 0.27, 10),
        (160.0, 0.33, 8),
    ] {
        let cy = ecran(g.hauteur * fy);
        let r: i32 = r;
        for dy in -r / 2..=r / 2 {
            let w = ((r * r - 4 * dy * dy) as f32).sqrt() as i32;
            let c = if dy > 0 { ARDOISE_PALE } else { NUAGE };
            rect(fx - w as f32, cy + dy as f32, (2 * w + 1) as f32, 1.0, c);
        }
    }
}

/// La pluie de l'orage, en bas de la carte : de courts traits qui tombent.
fn dessiner_pluie(g: &Geometrie, ecran: &impl Fn(f32) -> f32, anim: u32) {
    let haut = g.hauteur * 0.6;
    let bas = g.hauteur - H_TOITS;
    for i in 0..40u32 {
        let x = (i * 37 % L_CARTE as u32) as f32;
        let span = (bas - haut) as u32;
        let y = haut + ((i * 53 + anim * 3) % span) as f32;
        let (y, x) = (ecran(y), x + ((anim + i * 7) % 4) as f32);
        rect(x, y, 1.0, 3.0, PLUIE);
    }
}

/// Pointillés entre deux points.
fn pointilles((x0, y0): (f32, f32), (x1, y1): (f32, f32), c: Couleur) {
    let n = ((x1 - x0).hypot(y1 - y0) / 5.0).max(1.0) as i32;
    for k in 1..n {
        let t = k as f32 / n as f32;
        rect(
            (x0 + (x1 - x0) * t).round() - 1.0,
            (y0 + (y1 - y0) * t).round() - 1.0,
            2.0,
            2.0,
            c,
        );
    }
}

/// Le chemin qui monte (niveaux normaux), la branche secrète du bonus (une fois
/// révélé) et la descente de l'épilogue jusqu'aux toits.
fn dessiner_chemins(g: &Geometrie, s: &Sauvegarde, ecran: &impl Fn(f32) -> f32) {
    let pos = |i: usize| (g.etapes[i].0, ecran(g.etapes[i].1));
    let couleur = |i: usize| match etat_etape(NIVEAUX, s, i) {
        EtatEtape::Ouverte | EtatEtape::Finie => OR_PALE,
        _ => ARDOISE_PALE,
    };
    let normaux: Vec<usize> = (0..NIVEAUX.len())
        .filter(|&i| NIVEAUX[i].genre == GenreNiveau::Normal)
        .collect();
    // Des toits jusqu'au premier niveau.
    if let Some(&premier) = normaux.first() {
        let (x, y) = pos(premier);
        pointilles((x, ecran(g.hauteur - H_TOITS)), (x, y), OR_PALE);
    }
    for w in normaux.windows(2) {
        pointilles(pos(w[0]), pos(w[1]), couleur(w[1]));
    }
    let Some(&sommet) = normaux.last() else {
        return;
    };
    for (i, n) in NIVEAUX.iter().enumerate() {
        match n.genre {
            GenreNiveau::Bonus if etat_etape(NIVEAUX, s, i) != EtatEtape::Cachee => {
                pointilles(pos(sommet), pos(i), couleur(i));
            }
            // La descente part du sommet vers la droite, puis redescend.
            GenreNiveau::Epilogue => {
                let (x, y) = pos(i);
                let coude = (x, pos(sommet).1);
                pointilles(pos(sommet), coude, couleur(i));
                pointilles(coude, (x, y), couleur(i));
                pointilles((x, y), (x, ecran(g.hauteur - H_TOITS)), couleur(i));
            }
            _ => {}
        }
    }
}

/// Ce qu'on écrit dans la case d'une étape : son numéro (niveaux normaux), « B » pour
/// le bonus, « É » pour l'épilogue, « ? » pour le bonus caché.
fn marque(i: usize, etat: EtatEtape) -> String {
    let n = &NIVEAUX[i];
    match (etat, n.genre) {
        (EtatEtape::Cachee, _) => "?".into(),
        (_, GenreNiveau::Normal) => n.numero.to_string(),
        (_, GenreNiveau::Bonus) => "B".into(),
        (_, GenreNiveau::Epilogue) => "É".into(),
    }
}

/// Une étape : une case (bordure de la couleur de la médaille quand elle est finie),
/// cadenas ou marque, et ses 3 gouttes en petit dessous.
fn dessiner_etape(i: usize, x: f32, y: f32, s: &Sauvegarde, choisie: bool, anim: u32) {
    let etat = etat_etape(NIVEAUX, s, i);
    let progres = plan::progres(NIVEAUX, s, i);
    if choisie && (anim / 20).is_multiple_of(2) {
        panneau(x - DEMI - 2.0, y - DEMI - 2.0, 19.0, 19.0, OR, OR);
    }
    let (fond, bord, texte_c) = match etat {
        EtatEtape::Verrouillee => (ARDOISE_NUIT, ARDOISE, ARDOISE_PALE),
        EtatEtape::Cachee => (ARDOISE_NUIT, ARDOISE_CLAIRE, BRUME),
        EtatEtape::Ouverte => (ARDOISE_OMBRE, BRUME, BLANC_CASSE),
        EtatEtape::Finie if progres.medaille > 0 => {
            (ARDOISE_OMBRE, medaille(progres.medaille).0, OR_PALE)
        }
        EtatEtape::Finie => (ARDOISE_OMBRE, OR_PALE, OR_PALE),
    };
    panneau(
        x - DEMI,
        y - DEMI,
        2.0 * DEMI + 1.0,
        2.0 * DEMI + 1.0,
        fond,
        bord,
    );
    if etat == EtatEtape::Verrouillee {
        grille(&CADENAS, x - 3.0, y - 3.0, |c| match c {
            '#' => Some(ARDOISE_PALE),
            'o' => Some(ARDOISE),
            _ => None,
        });
    } else {
        let m = marque(i, etat);
        let lx = (x - largeur_texte(&m, 1) as f32 / 2.0 + 0.5).floor();
        texte::dessiner(&m, lx, y - 3.0, Style::simple(texte_c, 1));
    }
    if etat.jouable() {
        for (k, &trouvee) in progres.gouttes.iter().enumerate() {
            let c = if trouvee { OR } else { ARDOISE_OMBRE };
            rect(x - 5.0 + k as f32 * 4.0, y + DEMI + 2.0, 2.0, 3.0, c);
        }
    }
}

/// Une goutte d'or du panneau : le sprite si elle a été trouvée, sa silhouette sinon.
fn goutte(banque: &Banque, x: f32, y: f32, trouvee: bool) {
    if trouvee {
        banque.dessiner("goutte0", x, y, false);
    } else {
        grille(crate::sprites::objets::GOUTTE0, x, y, |c| match c {
            '.' => None,
            '0' => Some(ARDOISE_NUIT),
            _ => Some(ARDOISE),
        });
    }
}

/// Le panneau de l'étape choisie.
fn dessiner_panneau(banque: &Banque, s: &Sauvegarde, i: usize, anim: u32) {
    let (px, py, pw, ph) = PANNEAU;
    panneau(px, py, pw, ph, ARDOISE_NUIT, ARDOISE_CLAIRE);
    let centre = px + pw / 2.0;
    let ombre = |c| Style::simple(c, 1).avec_ombre(ARDOISE_OMBRE);
    let Some(infos) = NIVEAUX.get(i) else {
        return;
    };
    let etat = etat_etape(NIVEAUX, s, i);
    let progres = plan::progres(NIVEAUX, s, i);
    let x = px + 6.0;
    let mut y = py + 4.0;
    if etat == EtatEtape::Cachee {
        texte::dessiner_centre_sur(contenu::NIVEAU_SECRET, centre, y, ombre(BRUME));
        texte::dessiner_centre_sur("?", centre, y + 14.0, Style::simple(OR_PALE, 3));
        y += 42.0;
        for ligne in couper_lignes(contenu::INDICE_SECRET, LIGNE_PANNEAU) {
            texte::dessiner_centre_sur(&ligne, centre, y, ombre(BLANC_CASSE));
            y += 10.0;
        }
        // Où on en est : gouttes des niveaux normaux trouvées, sur le total.
        let compte = format!(
            "{}/{}",
            gouttes_trouvees_normales(NIVEAUX, s),
            gouttes_pour_le_bonus(NIVEAUX)
        );
        let lc = largeur_texte(&compte, 1) as f32 + 12.0;
        let gx = (centre - lc / 2.0).round();
        banque.dessiner("goutte0", gx, y + 3.0, false);
        texte::dessiner(&compte, gx + 12.0, y + 5.0, ombre(OR_PALE));
        dessiner_bouton_jouer(false, anim);
        return;
    }
    texte::dessiner_centre_sur(
        &contenu::etiquette_niveau(infos, nb_normaux(NIVEAUX)),
        centre,
        y,
        ombre(BRUME),
    );
    y += 11.0;
    for ligne in couper_lignes(&titre_niveau(infos.nom), LIGNE_PANNEAU)
        .iter()
        .take(2)
    {
        texte::dessiner_centre_sur(ligne, centre, y, ombre(OR_PALE));
        y += 10.0;
    }
    y = py + 38.0;
    if etat == EtatEtape::Verrouillee {
        grille(&CADENAS, centre - 3.0, y, |c| match c {
            '#' => Some(ARDOISE_PALE),
            'o' => Some(ARDOISE),
            _ => None,
        });
        y += 12.0;
        texte::dessiner_centre_sur(contenu::VERROUILLE, centre, y, ombre(BRUME));
        y += 12.0;
        for ligne in couper_lignes(contenu::INDICE_VERROUILLE, LIGNE_PANNEAU) {
            texte::dessiner_centre_sur(&ligne, centre, y, ombre(ARDOISE_PALE));
            y += 10.0;
        }
        dessiner_bouton_jouer(false, anim);
        return;
    }
    // Meilleur temps (« FINI » sans temps : niveau migré de la v2, carte 8.13).
    match progres.meilleur_temps {
        Some(t) => {
            let ligne = format!("{} {}", contenu::LIBELLE_MEILLEUR, chrono::temps_precis(t));
            texte::dessiner(&ligne, x, y, ombre(BLANC_CASSE));
        }
        None if progres.fini() => {
            texte::dessiner(contenu::FINI_SANS_TEMPS, x, y, ombre(BLANC_CASSE))
        }
        None => texte::dessiner(contenu::PAS_ENCORE_FINI, x, y, ombre(ARDOISE_PALE)),
    }
    y += 11.0;
    // Gouttes trouvées (au cumul), puis la médaille sur sa propre ligne (« AUCUNE
    // MÉDAILLE » ne tiendrait pas à côté des gouttes).
    for (k, &trouvee) in progres.gouttes.iter().enumerate() {
        goutte(banque, x + k as f32 * 13.0, y, trouvee);
    }
    y += 13.0;
    let (couleur, nom) = medaille(progres.medaille);
    icone_medaille(x, y, progres.medaille);
    texte::dessiner(nom, x + L_ICONE + ECART_ICONE, y, ombre(couleur));
    y += 13.0;
    // Les 3 défis, cochés s'ils ont déjà été réussis (le CHRONO montre le temps cible).
    for (defi, &ok) in defis_du_niveau(infos).iter().zip(&progres.defis) {
        case(x, y, ok);
        let c = if ok { BLANC_CASSE } else { ARDOISE_PALE };
        texte::dessiner(&defi.libelle(), x + L_ICONE + ECART_ICONE, y, ombre(c));
        y += 10.0;
    }
    dessiner_bouton_jouer(true, anim);
}

/// Le bouton JOUER du panneau (grisé si l'étape n'est pas jouable).
fn dessiner_bouton_jouer(actif: bool, anim: u32) {
    let (x, y, w, h) = BOUTON_JOUER;
    let (fond, bord, c) = if actif {
        (ARDOISE_OMBRE, OR, OR_PALE)
    } else {
        (ARDOISE_NUIT, ARDOISE, ARDOISE_CLAIRE)
    };
    panneau(x, y, w, h, fond, bord);
    texte::dessiner_centre_sur(
        contenu::JOUER,
        x + w / 2.0,
        y + 3.0,
        Style::simple(c, 2).avec_ombre(ARDOISE_OMBRE),
    );
    if actif && (anim / 20).is_multiple_of(2) {
        let chevron = Style::simple(OR, 2);
        texte::dessiner(">", x + 6.0, y + 3.0, chevron);
        texte::dessiner_a_droite("<", x + w - 6.0, y + 3.0, chevron);
    }
}
