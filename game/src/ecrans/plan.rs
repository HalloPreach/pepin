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

use pepin_core::font::{largeur_texte, ESPACEMENT, LARGEUR_GLYPHE};
use pepin_core::levels::{niveaux_du_monde, GenreNiveau};
use pepin_core::palette::{
    Couleur, ARDOISE, ARDOISE_CLAIRE, ARDOISE_NUIT, ARDOISE_OMBRE, ARDOISE_PALE, BLANC_CASSE,
    BRUME, OR, OR_PALE,
};
use pepin_core::LARGEUR_LOGIQUE;

use super::carte::titre_niveau;
use super::resultats::{case, grille, icone_medaille, medaille, ECART_ICONE, L_ICONE};
use super::{fond_plan, image, panneau, rect};
use crate::chrono;
use crate::contenu;
use crate::defis::defis_du_niveau;
use crate::plan::{
    self, etat_etape, gouttes_trouvees_normales, EtatEtape, EtatPlan, Geometrie, Zones, H_TOITS,
    H_VUE,
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
/// Panneau de l'étape choisie, et ses boutons JOUER et COURSE (carte 8.7) côte à côte.
const PANNEAU: (f32, f32, f32, f32) = (180.0, 16.0, 136.0, 148.0);
const BOUTON_JOUER: (f32, f32, f32, f32) = (184.0, 140.0, 63.0, 20.0);
const BOUTON_COURSE: (f32, f32, f32, f32) = (249.0, 140.0, 63.0, 20.0);
/// Bouton RETOUR, dans le bandeau du bas.
const BOUTON_RETOUR: (f32, f32, f32, f32) = (4.0, 166.0, 62.0, 13.0);
/// Flèches tactiles de chapitre (carte 9.30), dans le panneau, sur la rangée libre
/// au-dessus de JOUER/COURSE (le texte du panneau finit au plus à y = 119) :
/// « ◀ CHAPITRE 1 » à gauche, « CHAPITRE 2 ▶ » à droite. Hors de la vue de la carte,
/// elles ne cachent aucune étape. Avec deux chapitres, une seule est montrée à la
/// fois (test `une_seule_fleche_de_chapitre_a_la_fois`) : elles peuvent donc être
/// plus larges que la moitié du panneau.
const L_FLECHE: f32 = 74.0;
const H_FLECHE: f32 = 16.0;
const Y_FLECHE: f32 = 122.0;
const FLECHE_PRECEDENT: (f32, f32, f32, f32) = (BOUTON_JOUER.0, Y_FLECHE, L_FLECHE, H_FLECHE);
const FLECHE_SUIVANT: (f32, f32, f32, f32) = (
    BOUTON_COURSE.0 + BOUTON_COURSE.2 - L_FLECHE,
    Y_FLECHE,
    L_FLECHE,
    H_FLECHE,
);
/// Demi-côté d'une étape, et demi-côté de sa zone de toucher (plus large).
const DEMI: f32 = 7.0;
const DEMI_TOUCHER: f32 = 11.0;
/// Largeur du texte du panneau, en caractères (128 px / 6).
const LIGNE_PANNEAU: usize = 21;

const _: () = {
    assert!(Y_VUE + H_VUE <= BOUTON_RETOUR.1);
    assert!(BOUTON_JOUER.1 + BOUTON_JOUER.3 <= PANNEAU.1 + PANNEAU.3 - 2.0);
    assert!(BOUTON_JOUER.0 + BOUTON_JOUER.2 < BOUTON_COURSE.0);
    assert!(BOUTON_COURSE.0 + BOUTON_COURSE.2 <= PANNEAU.0 + PANNEAU.2 - 4.0);
    assert!(L_CARTE < PANNEAU.0);
    assert!(Y_FLECHE + H_FLECHE < BOUTON_JOUER.1);
    assert!(L_FLECHE >= 24.0 && H_FLECHE >= 16.0);
};

/// Ligne du haut du titre du bandeau (carte 9.28 : 1 px plus bas qu'avant, pour
/// laisser deux rangées aux accents au-dessus des lettres).
const Y_TITRE: f32 = 5.0;
// Le titre (7 rangées) et son ombre tiennent au-dessus de la vue de la carte.
const _: () = assert!(Y_TITRE + 7.0 + 1.0 <= Y_VUE);

/// Accents du titre, en pixels relatifs au coin de la lettre (rangées au-dessus).
const GRAVE: &[(f32, f32)] = &[(1.0, -3.0), (2.0, -2.0)];
const AIGU: &[(f32, f32)] = &[(3.0, -3.0), (2.0, -2.0)];
const CIRCONFLEXE: &[(f32, f32)] = &[(1.0, -2.0), (2.0, -3.0), (3.0, -2.0)];
const TREMA: &[(f32, f32)] = &[(1.0, -2.0), (3.0, -2.0)];

/// Une capitale accentuée du titre : sa lettre sans accent, et son accent (carte
/// 9.28). La police tasse ces lettres sur 6 rangées et y colle l'accent : au
/// bandeau, « LÀ OÙ » se lisait « LA OU ». Le titre les dessine en lettre entière,
/// l'accent à part, séparé d'une rangée.
fn accent_de(c: char) -> Option<(char, &'static [(f32, f32)])> {
    Some(match c {
        'À' => ('A', GRAVE),
        'Â' => ('A', CIRCONFLEXE),
        'Ä' => ('A', TREMA),
        'É' => ('E', AIGU),
        'È' => ('E', GRAVE),
        'Ê' => ('E', CIRCONFLEXE),
        'Ë' => ('E', TREMA),
        'Î' => ('I', CIRCONFLEXE),
        'Ï' => ('I', TREMA),
        'Ô' => ('O', CIRCONFLEXE),
        'Ö' => ('O', TREMA),
        'Ù' => ('U', GRAVE),
        'Û' => ('U', CIRCONFLEXE),
        'Ü' => ('U', TREMA),
        _ => return None,
    })
}

/// Le titre du bandeau du haut, centré sur l'écran, ombré, ses accents bien visibles.
fn dessiner_titre(titre: &str, y: f32, c: Couleur) {
    let x0 = texte::x_centre(titre, LARGEUR_LOGIQUE as f32 / 2.0, 1).round();
    let pas = (LARGEUR_GLYPHE + ESPACEMENT) as f32;
    let style = Style::simple(c, 1).avec_ombre(ARDOISE_OMBRE);
    for (i, car) in titre.chars().enumerate() {
        let x = x0 + i as f32 * pas;
        let (lettre, accent) = accent_de(car).unwrap_or((car, &[]));
        texte::dessiner(&lettre.to_string(), x, y, style);
        for &(dx, dy) in accent {
            rect(x + dx + 1.0, y + dy + 1.0, 1.0, 1.0, ARDOISE_OMBRE);
        }
        for &(dx, dy) in accent {
            rect(x + dx, y + dy, 1.0, 1.0, c);
        }
    }
}

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

    fn sur_course(&self, x: f32, y: f32) -> bool {
        dedans(BOUTON_COURSE, x, y)
    }

    fn sur_retour(&self, x: f32, y: f32) -> bool {
        dedans(BOUTON_RETOUR, x, y)
    }

    fn sur_chapitre_precedent(&self, x: f32, y: f32) -> bool {
        dedans(FLECHE_PRECEDENT, x, y)
    }

    fn sur_chapitre_suivant(&self, x: f32, y: f32) -> bool {
        dedans(FLECHE_SUIVANT, x, y)
    }
}

/// Une flèche tactile de chapitre (carte 9.30) : cadre sombre liseré d'or (lisible
/// sur les deux fonds), « CHAPITRE n » et un triangle plein vers le chapitre.
fn dessiner_fleche((x, y, w, h): (f32, f32, f32, f32), monde: u8, suivant: bool, anim: u32) {
    panneau(x, y, w, h, ARDOISE_NUIT, OR);
    let libelle = contenu::libelle_chapitre(monde);
    let lt = largeur_texte(&libelle, 1) as f32;
    let ty = y + (h - 7.0) / 2.0;
    // Le triangle (4 x 7) bat d'un pixel vers le chapitre, pour inviter au toucher.
    let pousse = if (anim / 20).is_multiple_of(2) {
        0.0
    } else {
        1.0
    };
    let (tx, lx) = if suivant {
        (x + w - 8.0 + pousse, x + 3.0)
    } else {
        (x + 4.0 - pousse, x + 11.0)
    };
    debug_assert!(lt <= w - 15.0);
    for k in 0..4 {
        let col = if suivant {
            tx + k as f32
        } else {
            tx + 3.0 - k as f32
        };
        let demi = 3.0 - k as f32;
        rect(col, ty + 3.0 - demi, 1.0, 2.0 * demi + 1.0, OR_PALE);
    }
    texte::dessiner(
        &libelle,
        lx,
        ty,
        Style::simple(OR_PALE, 1).avec_ombre(ARDOISE_OMBRE),
    );
}

/// Dessine la carte des niveaux. `anim` : pas écoulés depuis son ouverture.
pub fn dessiner(banque: &Banque, skin: Skin, anim: u32, s: &Sauvegarde, e: &EtatPlan) {
    let g = &e.geometrie;
    // Ordonnée écran d'une ordonnée de la carte.
    let ecran = |y: f32| (Y_VUE + y - e.defil).round();
    // Le fond du chapitre (carte 9.28) : l'ascension au chapitre 1, la descente de
    // l'eau au chapitre 2.
    let vue = fond_plan::Vue {
        y_vue: Y_VUE,
        l: L_CARTE,
        defil: e.defil,
    };
    if e.monde >= 2 {
        fond_plan::chapitre_2(
            banque,
            &vue,
            g,
            niveaux_du_monde(niveaux::niveaux(), e.monde),
            anim,
        );
    } else {
        fond_plan::chapitre_1(banque, &vue, g, anim);
    }
    dessiner_chemins(e, s, &ecran);
    for (k, &(x, y)) in g.etapes.iter().enumerate() {
        let i = e.debut_monde + k; // Indice DANS LA TABLE (l'étape est un indice local du monde).
        dessiner_etape(i, x, ecran(y), s, i == e.selection, anim);
    }
    // Pépin (skin actif) debout sur l'étape choisie (indice local de la tranche).
    if let Some(&(x, y)) = g.etapes.get(e.selection.saturating_sub(e.debut_monde)) {
        let pepin = image(&skin.images().repos, anim, 30);
        banque.dessiner(pepin, x - 8.0, ecran(y) - DEMI - 16.0, false);
    }

    // Bandeaux et panneau par-dessus : la carte ne déborde nulle part.
    let l = LARGEUR_LOGIQUE as f32;
    rect(0.0, 0.0, l, Y_VUE, ARDOISE_OMBRE);
    rect(0.0, Y_VUE + H_VUE, l, 180.0 - Y_VUE - H_VUE, ARDOISE_OMBRE);
    rect(L_CARTE, Y_VUE, l - L_CARTE, H_VUE, ARDOISE_OMBRE);
    let ombre = |c| Style::simple(c, 1).avec_ombre(ARDOISE_OMBRE);
    // Le bandeau du haut (carte 9.1b) : le titre du chapitre (carte 9.28 : le
    // chapitre 1 aussi), centré ; ses accents dessinés à part, bien visibles.
    let titre = contenu::titre_chapitre(e.monde).unwrap_or(contenu::TITRE_PLAN);
    dessiner_titre(titre, Y_TITRE, OR_PALE);
    banque.dessiner("goutte0", 4.0, 2.0, false);
    texte::dessiner(
        &s.gouttes_en_poche.to_string(),
        16.0,
        4.0,
        ombre(BLANC_CASSE),
    );
    dessiner_panneau(banque, s, e, anim);
    // Les flèches de chapitre (carte 9.30), seulement vers un chapitre jouable.
    for (suivant, zone) in [(false, FLECHE_PRECEDENT), (true, FLECHE_SUIVANT)] {
        if let Some(m) = e.chapitre_voisin(suivant, niveaux::niveaux(), s) {
            dessiner_fleche(zone, m, suivant, anim);
        }
    }
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
/// révélé) et la descente de l'épilogue jusqu'aux toits. Dessine le MONDE affiché :
/// les indices de `g.etapes` sont locaux à sa tranche, ceux de `etat_etape` sont des
/// indices de table (`debut_monde + k`).
fn dessiner_chemins(e: &EtatPlan, s: &Sauvegarde, ecran: &impl Fn(f32) -> f32) {
    let g = &e.geometrie;
    let debut = e.debut_monde;
    let tranche = niveaux_du_monde(niveaux::niveaux(), e.monde);
    let pos = |k: usize| (g.etapes[k].0, ecran(g.etapes[k].1));
    let couleur = |i: usize| match etat_etape(niveaux::niveaux(), s, i) {
        EtatEtape::Ouverte | EtatEtape::Finie => OR_PALE,
        _ => ARDOISE_PALE,
    };
    // Indices LOCAUX de la tranche du monde (l'étape `k` est `debut + k` en table).
    let normaux: Vec<usize> = tranche
        .iter()
        .enumerate()
        .filter(|(_, n)| n.genre == GenreNiveau::Normal)
        .map(|(k, _)| k)
        .collect();
    // Des toits jusqu'au premier niveau.
    if let Some(&premier) = normaux.first() {
        let (x, y) = pos(premier);
        pointilles((x, ecran(g.hauteur - H_TOITS)), (x, y), OR_PALE);
    }
    for w in normaux.windows(2) {
        pointilles(pos(w[0]), pos(w[1]), couleur(debut + w[1]));
    }
    let Some(&sommet) = normaux.last() else {
        return;
    };
    for (k, n) in tranche.iter().enumerate() {
        let i = debut + k;
        match n.genre {
            GenreNiveau::Bonus if etat_etape(niveaux::niveaux(), s, i) != EtatEtape::Cachee => {
                pointilles(pos(sommet), pos(k), couleur(i));
            }
            // La descente part du sommet vers la droite, puis redescend.
            GenreNiveau::Epilogue => {
                let (x, y) = pos(k);
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
    let n = &niveaux::niveaux()[i];
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
    let etat = etat_etape(niveaux::niveaux(), s, i);
    let progres = plan::progres(niveaux::niveaux(), s, i);
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
fn dessiner_panneau(banque: &Banque, s: &Sauvegarde, e: &EtatPlan, anim: u32) {
    let i = e.selection;
    let (px, py, pw, ph) = PANNEAU;
    panneau(px, py, pw, ph, ARDOISE_NUIT, ARDOISE_CLAIRE);
    let centre = px + pw / 2.0;
    let ombre = |c| Style::simple(c, 1).avec_ombre(ARDOISE_OMBRE);
    let Some(infos) = niveaux::niveaux().get(i) else {
        return;
    };
    let etat = etat_etape(niveaux::niveaux(), s, i);
    let progres = plan::progres(niveaux::niveaux(), s, i);
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
        // Où on en est : gouttes des niveaux normaux DU MONDE trouvées, sur le total du monde.
        let tranche = niveaux_du_monde(niveaux::niveaux(), e.monde);
        let compte = format!(
            "{}/{}",
            gouttes_trouvees_normales(tranche, s),
            gouttes_pour_le_bonus(tranche)
        );
        let lc = largeur_texte(&compte, 1) as f32 + 12.0;
        let gx = (centre - lc / 2.0).round();
        banque.dessiner("goutte0", gx, y + 3.0, false);
        texte::dessiner(&compte, gx + 12.0, y + 5.0, ombre(OR_PALE));
        dessiner_boutons(false, false, e.course, anim);
        return;
    }
    texte::dessiner_centre_sur(
        // Le numéro est PAR MONDE (carte 9.1b) : « NIVEAU 1/8 », sans le chapitre
        // (trop large pour le panneau ; le bandeau du haut le nomme déjà).
        &contenu::etiquette_courte(infos),
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
        dessiner_boutons(false, false, e.course, anim);
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
    let (couleur, mut nom) = medaille(progres.medaille);
    // Boss vaincu seulement en mode facile (carte 8.14) : « FACILE », sans médaille.
    if progres.facile && !progres.battu_en_normal() {
        nom = contenu::FACILE;
    }
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
    dessiner_boutons(
        true,
        plan::course_possible(niveaux::niveaux(), s, i),
        e.course,
        anim,
    );
}

/// Les boutons JOUER et COURSE du panneau (carte 8.7) : grisés si l'étape n'est pas
/// jouable (COURSE aussi sans partie de fantôme) ; celui que choisit le clavier
/// (gauche / droite, `course`) a le liseré d'or et les chevrons.
fn dessiner_boutons(jouer: bool, course: bool, choix_course: bool, anim: u32) {
    bouton(
        BOUTON_JOUER,
        contenu::JOUER,
        jouer,
        jouer && !choix_course,
        anim,
    );
    bouton(
        BOUTON_COURSE,
        contenu::COURSE,
        course,
        course && choix_course,
        anim,
    );
}

/// Un bouton du panneau : `actif` (sinon grisé), `choisi` (liseré d'or, chevrons).
fn bouton((x, y, w, h): (f32, f32, f32, f32), libelle: &str, actif: bool, choisi: bool, anim: u32) {
    let (fond, bord, c) = match (actif, choisi) {
        (true, true) => (ARDOISE_OMBRE, OR, OR_PALE),
        (true, false) => (ARDOISE_OMBRE, ARDOISE_CLAIRE, BRUME),
        (false, _) => (ARDOISE_NUIT, ARDOISE, ARDOISE_CLAIRE),
    };
    panneau(x, y, w, h, fond, bord);
    let ty = y + (h - 7.0) / 2.0;
    texte::dessiner_centre_sur(
        libelle,
        x + w / 2.0,
        ty,
        Style::simple(c, 1).avec_ombre(ARDOISE_OMBRE),
    );
    if choisi && (anim / 20).is_multiple_of(2) {
        let chevron = Style::simple(OR, 1);
        texte::dessiner(">", x + 4.0, ty, chevron);
        texte::dessiner_a_droite("<", x + w - 4.0, ty, chevron);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les titres des deux chapitres : chaque capitale accentuée a son accent dessiné
    /// à part, dans le bandeau (rangée 2 au moins), et le titre centré ne touche pas le
    /// compteur de gouttes (x < 48).
    #[test]
    fn les_accents_du_titre_tiennent_dans_le_bandeau() {
        for monde in [1, 2] {
            let titre = contenu::titre_chapitre(monde).unwrap();
            for c in titre.chars().filter(|c| !c.is_ascii()) {
                let (lettre, accent) = accent_de(c).unwrap_or_else(|| panic!("{c:?}"));
                assert!(lettre.is_ascii_uppercase());
                for &(_, dy) in accent {
                    assert!(Y_TITRE + dy >= 1.0, "{c:?} : l'accent sort du bandeau");
                    assert!(dy <= -2.0, "{c:?} : une rangée vide sous l'accent");
                }
            }
            let x0 = texte::x_centre(titre, LARGEUR_LOGIQUE as f32 / 2.0, 1);
            assert!(x0 >= 48.0, "{titre} : x {x0}");
        }
    }
}
