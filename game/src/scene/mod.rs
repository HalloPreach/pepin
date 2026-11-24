//! Scène : dessine un `Monde` dans la cible 320x180, couche par couche.
//!
//! Ordre (arrière -> avant) : fond (ciel, silhouettes) -> déchirure du ciel (Grand
//! Nuage vaincu) -> façades des boutiques derrière les auvents -> tuiles -> décor ->
//! panneaux -> cadrans des aiguilles d'horloge ->
//! plateformes -> objets -> ennemis (Grand Nuage et ses éclairs compris) -> Pépin ->
//! particules -> pluie.
//! L'interface (compteurs, chrono) est dessinée par `hud`, les menus par `ecrans`.
//! Chaque sprite est posé à une position ENTIÈRE (positions interpolées puis
//! arrondies) : aucun sous-pixel.
//!
//! CARTE DES RESPONSABILITÉS (carte 10.2a). La scène ne fait QUE dessiner : elle lit
//! le `Monde` sans jamais le modifier. L'état du jeu, la boucle, les transitions et
//! les entrées vivent dans `app.rs`, `entrees.rs` et `camera.rs`, pas ici.
//! - `mod.rs` (ce fichier) : `dessiner`, l'ordre des couches d'une image ; les petites
//!   aides partagées (`rect`, `image`, `entier`) ; `borner_mort` (cadre de l'animation
//!   de mort, utilisé par `crate::effets`).
//! - `tuiles` : quel dessin poser sur chaque case de la grille (variantes de brique,
//!   ombre sous les toits, cases cachées, tuiles du thème), puis leur pose.
//! - `habillage` : décor fixe du niveau (façades des boutiques, panneaux, oiseaux et
//!   manches à air, cadrans des aiguilles d'horloge, drapeau d'arrivée).
//! - `entites` : ennemis, plateformes, objets et auvents, avec l'image de chaque état.
//! - `nuage` : le Grand Nuage, ses éclairs annoncés et la déchirure du ciel.
//! - `pepin` : Pépin (attente, clignotement), sa barque et le fantôme de la course.
//! - `interface` : les repères d'interface dessinés dans la scène (flèche de Pépin
//!   au-dessus de l'écran). Compteurs et chrono : `crate::hud`.
//!
//! Pour changer un dessin, ouvrir le sous-module de ce qu'on dessine : aucun ne
//! connaît la progression du joueur (niveaux finis, sauvegarde).

mod entites;
mod habillage;
mod interface;
mod nuage;
mod pepin;
mod tuiles;

// Ce que le reste du jeu utilise de la scène : les chemins `crate::scene::...` ne
// changent pas, même si le code vit maintenant dans un sous-module.
pub(crate) use entites::image_pigeon;
pub(crate) use nuage::{annonce_allumee, image_nuage};
pub(crate) use pepin::dessiner_fantome;
pub(crate) use tuiles::{nom_tuile, noms_de_la_case, PAS_VAGUE};
// Utilisés seulement par des tests ailleurs dans le jeu (robot de QA, difficulté).
#[cfg(test)]
pub(crate) use interface::repere_haut;
#[cfg(test)]
pub(crate) use pepin::pepin_cache;

// Les couches que `dessiner` empile, chacune dans son sous-module.
use entites::dessiner_entites;
use habillage::{
    dessiner_arrivee, dessiner_cadrans, dessiner_decors, dessiner_facades, dessiner_panneaux,
};
use interface::dessiner_repere_pepin;
use nuage::dessiner_dechirure;
use pepin::dessiner_pepin;
use tuiles::dessiner_tuiles;

use macroquad::prelude::draw_rectangle;
use pepin_core::aabb::Aabb;
use pepin_core::palette::*;
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use crate::cosmetiques::Trainees;
use crate::decor;
use crate::monde::Monde;
use crate::mur_pluie;
use crate::particules::Particules;
use crate::rendu::couleur_mq;
use crate::skins::Skin;
use crate::sprites::Banque;

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

/// Marge (px) gardée entre le bas de l'écran et l'animation de mort d'une chute.
const MARGE_MORT_BAS: f32 = 16.0;
/// Largeur (px) réservée à l'animation de mort pour qu'elle reste entière à l'écran.
const LARGEUR_MORT: f32 = 16.0;

/// Place l'animation de mort (et les éclats) dans le cadre. `pos` : où Pépin est mort, dans le
/// niveau (coin haut-gauche) ; `camera` : coin haut-gauche de l'écran dans le niveau. Une chute
/// dans un trou finit sous l'écran, et la caméra repart vers le point de reprise : sans cette
/// limite, l'animation se jouerait hors champ. L'ordonnée est bornée à 16 px au-dessus du bord
/// bas, l'abscisse à la largeur de l'écran. Fonction pure.
pub(crate) fn borner_mort(pos: (f32, f32), camera: (f32, f32)) -> (f32, f32) {
    let x_max = camera.0 + LARGEUR_LOGIQUE as f32 - LARGEUR_MORT;
    let y_max = camera.1 + HAUTEUR_LOGIQUE as f32 - MARGE_MORT_BAS;
    (pos.0.clamp(camera.0, x_max), pos.1.min(y_max))
}

/// Dessine tout le monde dans la cible logique. `(cx, cy)` : caméra entière ; `skin` :
/// la tenue de Pépin (`Skin::Pepin` par défaut) ; `trainees` : la traînée cosmétique
/// (carte 7.6), sous les ennemis et les dangers.
#[allow(clippy::too_many_arguments)]
pub(crate) fn dessiner(
    monde: &Monde,
    banque: &Banque,
    particules: &Particules,
    trainees: &Trainees,
    skin: Skin,
    alpha: f32,
    cx: i32,
    cy: i32,
) {
    // Le fond du port (carte 9.11b) : sans son phare lointain sur la jetée de la
    // Pieuvre, qui a le sien, rayé, plus près.
    if crate::dessin_port::est_la_jetee(monde) {
        // Carte 9.13c : sous la tour de la Pieuvre, le fond de l'arène descend
        // quand Pépin grimpe, et la nuit continue au-dessus (un fond continu).
        let cy_fond = decor::camera_fond(cy, monde.niveau.hauteur_px() as i32);
        crate::decor_port::dessiner_fond_sous_la_tour(cx, monde.pas, -cy_fond);
    } else {
        // La pile du thème (carte 10.4f : avec la grande chute de la cascade et
        // la vallée de la rivière, qui y sont entrées).
        decor::dessiner_fond_du_niveau(monde, cx, cy);
    }
    // La Pieuvre du port (carte 9.20e) : la mer, le phare et les bateaux du port.
    crate::dessin_port::dessiner_fond(monde, banque, cx, cy);
    mur_pluie::dessiner_soleil(monde);
    dessiner_dechirure(monde, cx);
    dessiner_facades(monde, banque, cx, cy);
    // L'eau des marées (carte 9.3), derrière les tuiles qui la bordent.
    crate::eau::dessiner_marees(monde, banque, cx, cy);
    // Les roues de l'autobus du caniveau (carte 9.20f), dans l'eau qui les couvre.
    crate::mobilier_rue::dessiner_derriere(monde, cx, cy);
    // Les descentes de gouttière du caniveau et leur pan de maison (carte 9.20h).
    crate::decor_rue::dessiner_gouttieres(monde, cx, cy);
    dessiner_tuiles(monde, banque, cx, cy);
    // La jetée de la Pieuvre (carte 9.20e) : bassin profond, quais, grue, caisses.
    crate::dessin_port::dessiner_jetee(monde, banque, cx, cy);
    // La plateforme que la Pieuvre emporte sous l'eau (carte 9.12f).
    crate::dessin_pieuvre::dessiner_saisie(monde, banque, cx, cy);
    // La bâche de l'arène, et la mer qui monte de la phase 2 (carte 9.13a).
    crate::dessin_pieuvre::dessiner_bache(monde, banque, cx, cy);
    crate::dessin_pieuvre::dessiner_mer(monde, banque, cx, cy);
    // La montée (carte 9.13c) : la géante et ses attaques, qui partent de son corps.
    crate::dessin_montee::dessiner(monde, banque, cx, cy);
    // Carte 9.13e : la mer toujours sensible, DEVANT la géante (elle y plonge) :
    // sa crête et ses embruns, ou la bande d'écume au bas de l'écran.
    crate::mer_montee::dessiner_devant(monde, cx, cy);
    // Les stries du courant (carte 9.6b), sur l'eau qui coule.
    crate::dessin_courant::dessiner(monde, cx, cy);
    // L'habillage des égouts (carte 9.20b) : grilles, tuyaux des bassins, sortie.
    crate::decor_egouts::dessiner(monde, cx, cy);
    // L'habillage de la grotte (carte 9.20d) : rideau de la cascade, sources, jour.
    crate::decor_grotte::dessiner(monde, cx, cy);
    // La rivière (carte 9.10b) : reflets, écume des rochers et des chutes, roseaux.
    crate::decor_riviere::dessiner(monde, cx, cy);
    // Le mobilier de rue du caniveau (carte 9.20f) : voitures, autobus, abribus...
    crate::mobilier_rue::dessiner(monde, cx, cy);
    dessiner_decors(monde, banque, cx, cy);
    // Le filet d'eau de chaque gouttière, sur l'eau du caniveau (carte 9.20h).
    crate::decor_rue::dessiner_filets(monde, cx, cy);
    // Le grand tuyau du départ et le bassin d'arrivée de la cascade (carte 9.20c).
    crate::decor_cascade::dessiner(monde, cx, cy);
    dessiner_panneaux(monde, banque, cx, cy);
    dessiner_arrivee(monde, banque, cx, cy);
    dessiner_cadrans(monde, banque, alpha, cx, cy);
    trainees.dessiner(cx, cy);
    dessiner_entites(monde, banque, alpha, cx, cy);
    dessiner_pepin(monde, banque, skin, alpha, cx, cy);
    particules.dessiner(cx, cy);
    // Niveau de poursuite (cartes 8.2 et 8.2b) : le front d'orage du Grand Nuage
    // devant tout. Il pleut DANS le nuage seulement : pas de pluie du thème devant
    // Pépin, le ciel d'altitude reste clair.
    mur_pluie::dessiner_mur(monde, banque, cx);
    if monde.mur.is_none() {
        decor::dessiner_avant(monde.infos.theme, monde.pas, monde.ng_plus);
    }
    dessiner_repere_pepin(monde, alpha, cx, cy);
    // La jauge de la montée de la Pieuvre (carte 9.13e), sur le mur gauche.
    crate::mer_montee::dessiner_jauge(monde);
    // L'encre de la Pieuvre (carte 9.12a), devant tout.
    crate::dessin_pieuvre::dessiner_encre(monde);
    // Le rideau du temps de mort (carte 9.24), devant tout.
    if monde.temps_de_mort {
        crate::dessin_mort::dessiner_rideau(monde.reapparition);
    }
    // Build de QA seulement (carte 9.20g) : la touche O repasse les cases SOLIDES
    // de la grille en couleur vive (test de lisibilité des captures).
    #[cfg(feature = "debug_niveaux")]
    if crate::entrees::solides_qa() {
        crate::mobilier_rue::dessiner_solides_qa(monde, cx, cy);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
