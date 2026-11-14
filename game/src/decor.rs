//! Décor : ciel qui s'éclaircit d'un niveau à l'autre (orage -> soleil doré),
//! étoiles, silhouettes de toits ou de nuages en parallaxe sur deux couches,
//! soleil, et pluie diagonale. Tout est dessiné par rectangles alignés sur la
//! grille de pixels logiques (320x180) ; les décalages de parallaxe sont ARRONDIS
//! à l'entier : zéro sous-pixel, zéro scintillement.
//!
//! Les fonctions de calcul (`ciel`, `goutte_de_pluie`, `decalage`, `immeuble`) sont
//! pures : elles ne dessinent rien et se testent sans fenêtre.
//!
//! Notions Rust illustrées ici : struct de configuration par thème, `match`
//! exhaustif sur un `enum`, itérateurs sur des intervalles, arithmétique entière
//! qui boucle (`rem_euclid`).

use macroquad::prelude::draw_rectangle;
use pepin_core::palette::*;
use pepin_core::zones::Theme;
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use crate::rendu::couleur_mq;

const L: i32 = LARGEUR_LOGIQUE as i32;
const H: i32 = HAUTEUR_LOGIQUE as i32;

/// Forme des silhouettes d'une couche.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Forme {
    /// Immeubles : corps, toit en gradins, cheminée.
    Immeubles,
    /// Bancs de nuages arrondis (trois étages en dôme).
    Nuages,
    /// Tours ou pics de nuages en quatre étages de plus en plus étroits.
    Pics,
}

/// Plafond de nuages d'orage : des dômes renversés accrochés en haut de l'écran,
/// sur deux couches de parallaxe (lointaine, proche).
#[derive(Debug, Clone, Copy)]
pub struct Plafond {
    pub loin: Couleur,
    pub proche: Couleur,
    /// Épaisseur des nuages : 4 = pleine, 2 = nuages discrets.
    pub ampleur: i32,
}

/// Ce qui distingue l'ambiance d'un niveau.
#[derive(Debug, Clone, Copy)]
pub struct Ambiance {
    /// Quatre bandes de ciel, de haut en bas.
    pub ciel: [Couleur; 4],
    /// Silhouettes lointaines et proches.
    pub lointain: Couleur,
    pub proche: Couleur,
    /// Couleur des traînées de pluie (lointaines, proches).
    pub pluie: (Couleur, Couleur),
    /// Étoiles : `Some(y)` = semées au-dessus de l'ordonnée `y` (pas dans une bande
    /// de ciel où elles seraient invisibles), `None` = pas d'étoiles.
    pub etoiles: Option<i32>,
    /// Gouttes de pluie par couche (l'averse redouble, puis se calme).
    pub gouttes: i32,
    /// Pixels dont la couche PROCHE est descendue : plus aucune silhouette ne dépasse
    /// de la ligne du sol (`LIGNE_SOL`).
    pub descente: i32,
    /// Plafond de nuages d'orage accroché en haut de l'écran, s'il y en a un.
    pub plafond: Option<Plafond>,
    /// Soleil visible ?
    pub soleil: bool,
    /// Forme des silhouettes (immeubles, bancs de nuages ou pics).
    pub forme: Forme,
    /// Pixels ajoutés à la hauteur des silhouettes de la couche LOINTAINE :
    /// un fond qui monte haut au lieu d'un simple liseré en bas d'écran.
    pub relief: i32,
    /// La couche lointaine descend jusqu'au bas de l'écran (pas de bande de ciel
    /// sous les silhouettes dans les trouées de la couche proche).
    pub socle: bool,
    /// Fenêtres allumées (lointaines, proches) sur les immeubles, s'il y en a.
    pub fenetres: Option<(Couleur, Couleur)>,
    /// Petites îles de nuages flottantes (corps, dessous), s'il y en a.
    pub ilots: Option<(Couleur, Couleur)>,
    /// Teinte des TUILES du niveau (toits, briques, nuages) : voir `TeinteTuiles`.
    pub tuiles: TeinteTuiles,
    /// Les dalles friables sont-elles des nuages (sinon des dalles d'ardoise) ?
    pub friable_nuage: bool,
    /// Multiplicateur de la parallaxe VERTICALE des silhouettes (`VERT_*`) : 1.0
    /// partout, sauf au clocher (niveau haut) où la ville doit s'enfoncer
    /// nettement quand on monte. Sans effet sur un niveau d'un écran (la caméra
    /// du fond y reste à 0, hors secousse de mort).
    pub vertige: f32,
}

/// Éclaircissement des tuiles d'un thème : les toits pâlissent d'un niveau à l'autre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeinteTuiles {
    /// Crans gagnés par la rampe d'ardoise (0 = couleurs du dessin, jusqu'au blanc cassé).
    pub eclaircissement: u32,
    /// Les briques violettes virent à l'orange (sous le soleil).
    pub briques_orangees: bool,
}

/// Ambiance d'un thème : l'orage noir s'éclaircit jusqu'au soleil doré.
///
/// C'est LA table d'ambiance des niveaux : chaque niveau désigne son thème
/// (`InfosNiveau::theme`) et tout ce qui en dépend au rendu (ciel, silhouettes, pluie,
/// teinte des tuiles, dessin des dalles friables) se lit ici. Un nouveau niveau à
/// l'ambiance inédite = un nouveau `Theme` (dans `pepin_core::zones`, avec
/// `Theme::TOUS`) et une entrée de plus dans ce `match` : le compilateur refuse
/// d'oublier l'une ou l'autre.
pub fn ambiance(theme: Theme) -> Ambiance {
    match theme {
        Theme::Toits => Ambiance {
            ciel: [ARDOISE_OMBRE, ARDOISE_NUIT, ARDOISE, ARDOISE],
            lointain: ARDOISE_NUIT,
            proche: ARDOISE_OMBRE,
            pluie: (PLUIE_SOMBRE, PLUIE),
            etoiles: Some(H / 2),
            gouttes: 24,
            descente: DESCENTE_IMMEUBLES,
            plafond: None,
            soleil: false,
            forme: Forme::Immeubles,
            relief: 0,
            socle: true,
            fenetres: None,
            ilots: None,
            tuiles: TeinteTuiles {
                eclaircissement: 0,
                briques_orangees: false,
            },
            friable_nuage: false,
            vertige: 1.0,
        },
        // L'averse redouble : deux fois plus de gouttes, et un plafond de nuages
        // discrets en haut du ciel.
        Theme::Gouttieres => Ambiance {
            ciel: [ARDOISE_NUIT, ARDOISE, ARDOISE_CLAIRE, ARDOISE_CLAIRE],
            lointain: ARDOISE,
            proche: ARDOISE_NUIT,
            pluie: (PLUIE_SOMBRE, PLUIE),
            etoiles: None,
            gouttes: 48,
            descente: DESCENTE_IMMEUBLES,
            plafond: Some(Plafond {
                loin: ARDOISE,
                proche: ARDOISE_CLAIRE,
                ampleur: 2,
            }),
            soleil: false,
            forme: Forme::Immeubles,
            relief: 0,
            socle: true,
            fenetres: None,
            ilots: None,
            tuiles: TeinteTuiles {
                eclaircissement: 1,
                briques_orangees: false,
            },
            friable_nuage: false,
            vertige: 1.0,
        },
        // Marché couvert : nuit bleue qui s'éclaircit vers l'horizon (la lueur du
        // marché), étoiles en haut, immeubles aux fenêtres orange (les lanternes et
        // les vitrines des marchands), une pluie entre l'averse et l'orage.
        Theme::Marche => Ambiance {
            ciel: [ARDOISE_OMBRE, ARDOISE_NUIT, ARDOISE, ARDOISE_CLAIRE],
            lointain: ARDOISE_NUIT,
            proche: ARDOISE_OMBRE,
            pluie: (PLUIE_SOMBRE, PLUIE),
            etoiles: Some(H / 4),
            gouttes: 36,
            descente: DESCENTE_IMMEUBLES,
            plafond: None,
            soleil: false,
            forme: Forme::Immeubles,
            // Pas de relief : les immeubles lointains (bleu nuit) restent plus bas que
            // la bande de ciel bleu nuit (y de 45 à 90), où ils disparaîtraient.
            relief: 0,
            socle: true,
            // Comme à l'orage : l'or est réservé aux gouttes et aux éclairs.
            fenetres: Some((ORANGE_SOLEIL, CORAIL_CLAIR)),
            ilots: None,
            tuiles: TeinteTuiles {
                eclaircissement: 1,
                briques_orangees: false,
            },
            friable_nuage: false,
            vertige: 1.0,
        },
        // Orage : le ciel est violet à l'horizon ; les immeubles, plus hauts
        // qu'aux niveaux 1 et 2, s'y découpent en noir et bleu nuit. Un plafond
        // de gros nuages d'orage pèse sur le haut de l'écran.
        Theme::Orage => Ambiance {
            ciel: [ARDOISE_OMBRE, ARDOISE, VIOLET_ORAGE, VIOLET_ORAGE],
            lointain: ARDOISE_NUIT,
            proche: ARDOISE_OMBRE,
            pluie: (PLUIE_SOMBRE, PLUIE),
            etoiles: None,
            gouttes: 60,
            descente: DESCENTE_IMMEUBLES,
            plafond: Some(Plafond {
                loin: ARDOISE_NUIT,
                proche: ARDOISE,
                ampleur: 4,
            }),
            soleil: false,
            forme: Forme::Immeubles,
            relief: 22,
            socle: true,
            // L'or est réservé aux gouttes à ramasser et aux éclairs : fenêtres
            // orange (loin) et corail clair (près).
            fenetres: Some((ORANGE_SOLEIL, CORAIL_CLAIR)),
            ilots: None,
            tuiles: TeinteTuiles {
                eclaircissement: 0,
                briques_orangees: false,
            },
            friable_nuage: false,
            vertige: 1.0,
        },
        // Clocher (niveau vertical) : nuit violette après l'orage, une lueur pâle
        // à l'horizon (la lune derrière les nuages qui s'en vont), étoiles en haut,
        // immeubles aux fenêtres orange. En montant, la ville s'enfonce deux fois
        // et demie plus vite qu'au niveau d'essai vertical (`vertige`) et découvre
        // la lueur : au sommet, on SENT la hauteur.
        Theme::Clocher => Ambiance {
            ciel: [ARDOISE_NUIT, VIOLET_ORAGE, VIOLET_ORAGE, ARDOISE_PALE],
            lointain: ARDOISE,
            proche: ARDOISE_OMBRE,
            pluie: (PLUIE_SOMBRE, PLUIE),
            // Étoiles dans la bande du haut seulement (bleu nuit).
            etoiles: Some(H / 4),
            // L'orage s'éloigne : une petite pluie.
            gouttes: 16,
            descente: DESCENTE_IMMEUBLES,
            plafond: None,
            soleil: false,
            forme: Forme::Immeubles,
            relief: 22,
            socle: true,
            // Comme à l'orage : l'or est réservé aux gouttes et aux éclairs.
            fenetres: Some((ORANGE_SOLEIL, CORAIL_CLAIR)),
            ilots: None,
            // Briques violettes, toits d'ardoise sombre : le violet du clocher.
            tuiles: TeinteTuiles {
                eclaircissement: 0,
                briques_orangees: false,
            },
            friable_nuage: false,
            vertige: 2.5,
        },
        // L'œil du cyclone (le boss) : violet et or. Tout en haut, la trouée d'or
        // (le ciel au-dessus de l'œil), bordée des nuages d'orage qui tournent
        // autour (plafond) ; plus bas, le mur de l'œil s'assombrit jusqu'à la nuit,
        // et des tours de nuages violettes montent du fond, les proches presque
        // noires. Un peu de pluie que le cyclone fait tourner.
        Theme::Cyclone => Ambiance {
            ciel: [ORANGE_SOLEIL, ARDOISE, ARDOISE_NUIT, ARDOISE_NUIT],
            lointain: VIOLET_ORAGE,
            proche: ARDOISE_OMBRE,
            pluie: (PLUIE_SOMBRE, PLUIE),
            // Pas d'étoiles : la bande du haut est la trouée d'or.
            etoiles: None,
            gouttes: 32,
            descente: DESCENTE_PICS,
            plafond: Some(Plafond {
                loin: ARDOISE_NUIT,
                proche: VIOLET_ORAGE,
                ampleur: 2,
            }),
            soleil: false,
            forme: Forme::Pics,
            relief: 26,
            socle: true,
            fenetres: None,
            ilots: None,
            // L'arène est faite de nuages (`N`) : la rampe d'ardoise ne sert pas.
            tuiles: TeinteTuiles {
                eclaircissement: 0,
                briques_orangees: false,
            },
            friable_nuage: true,
            vertige: 1.0,
        },
        // Altitude : tours de nuages violettes sur le ciel corail, tours proches
        // plus sombres, petites îles flottantes dans le tiers médian.
        Theme::Altitude => Ambiance {
            ciel: [VIOLET_ORAGE, CORAIL, CORAIL_CLAIR, ORANGE_SOLEIL],
            lointain: VIOLET_ORAGE,
            proche: ARDOISE_NUIT,
            // Cyan et blanc : visibles devant les quatre bandes du ciel.
            pluie: (PLUIE, BLANC_CASSE),
            // Étoiles dans la bande violette du haut seulement (invisibles ailleurs).
            etoiles: Some(H / 4),
            gouttes: 18,
            descente: DESCENTE_PICS,
            plafond: None,
            soleil: false,
            forme: Forme::Pics,
            relief: 14,
            socle: true,
            fenetres: None,
            ilots: Some((VIOLET_ORAGE, ARDOISE_NUIT)),
            tuiles: TeinteTuiles {
                eclaircissement: 2,
                briques_orangees: false,
            },
            friable_nuage: true,
            vertige: 1.0,
        },
        // Soleil : pas de pluie. Bancs de nuages rosés par le soleil, bas (leurs
        // sommets restent sous la ligne du sol), d'une couleur absente des
        // plateformes et des bandes de ciel de derrière.
        Theme::Soleil => Ambiance {
            ciel: [CORAIL_CLAIR, ORANGE_SOLEIL, OR, OR_PALE],
            lointain: CORAIL_CLAIR,
            proche: BRUME,
            pluie: (OR_PALE, BLANC_CASSE),
            etoiles: None,
            gouttes: 0,
            descente: 0,
            plafond: None,
            soleil: true,
            forme: Forme::Nuages,
            relief: 0,
            socle: false,
            fenetres: None,
            ilots: None,
            tuiles: TeinteTuiles {
                eclaircissement: 3,
                briques_orangees: true,
            },
            friable_nuage: true,
            vertige: 1.0,
        },
    }
}

/// Ordonnée du sol de la couche lointaine et de la couche proche (avant parallaxe verticale).
const BASE_LOINTAINE: i32 = H - 18;
const BASE_PROCHE: i32 = H + 4;
/// Ordonnée de la ligne du sol (rangée 8 du niveau, soit 48 px au-dessus du bas) :
/// aucune silhouette de la couche proche ne doit la dépasser (elle se lirait
/// comme un objet posé sur le toit où l'on marche).
#[cfg(test)]
pub const LIGNE_SOL: i32 = H - 48;
/// Descente de la couche proche : immeubles (jusqu'à 71 px) et tours (jusqu'à 64 px),
/// avec une marge pour la parallaxe verticale (jusqu'à une dizaine de pixels).
const DESCENTE_IMMEUBLES: i32 = 32;
const DESCENTE_PICS: i32 = 24;
/// Plafond de nuages : parallaxe, graines, cellules, et épaisseur maximale.
const FACTEUR_PLAFOND_LOIN: f32 = 0.10;
const FACTEUR_PLAFOND_PROCHE: f32 = 0.20;
const GRAINE_PLAFOND_LOIN: u32 = 31;
const GRAINE_PLAFOND_PROCHE: u32 = 37;
const CELLULE_PLAFOND: i32 = 56;
/// Facteurs de parallaxe horizontale des couches (lointaine, îlots, proche).
const FACTEUR_LOINTAINE: f32 = 0.15;
const FACTEUR_ILOTS: f32 = 0.25;
const FACTEUR_PROCHE: f32 = 0.35;
/// Facteurs de parallaxe VERTICALE des mêmes couches : bien plus petits qu'en largeur
/// (un niveau haut de quatre écrans fait monter la caméra de 540 px : la couche proche
/// descend alors de 54 px, la lointaine de 27). Le ciel, les étoiles, le soleil et le
/// plafond de nuages restent fixés à l'écran : ils sont « à l'infini ».
const VERT_LOINTAINE: f32 = 0.05;
const VERT_ILOTS: f32 = 0.07;
const VERT_PROCHE: f32 = 0.10;
/// Graines de hasard des deux couches de silhouettes.
const GRAINE_LOINTAINE: u32 = 11;
const GRAINE_PROCHE: u32 = 23;
/// Largeur d'une cellule d'îlots et fourchette d'ordonnées des îlots (tiers médian).
const CELLULE_ILOT: i32 = 72;
const ILOT_Y_MIN: i32 = 58;
const ILOT_Y_ALEA: u32 = 20;
const ILOT_HAUTEUR: i32 = 8;

#[cfg(test)]
/// Couleur de la bande de ciel située à l'ordonnée `y` (fonction pure : le test
/// vérifie par là qu'aucune silhouette n'a la couleur du ciel derrière elle).
pub(crate) fn ciel_a(a: &Ambiance, y: i32) -> Couleur {
    let bande = (H / 4) as usize;
    a.ciel[(y.max(0) as usize / bande).min(3)]
}

/// Décalage de parallaxe entier d'une couche : plus `facteur` est petit, plus la
/// couche semble lointaine (elle bouge moins vite que la caméra).
pub fn decalage(camera: i32, facteur: f32) -> i32 {
    (camera as f32 * facteur).round() as i32
}

/// Ordonnée de la caméra telle que le FOND la voit : comptée depuis la position la plus
/// basse de la caméra dans le niveau (bas du niveau sur le bas de l'écran). Niveau haut
/// (carte 5.2) : 0 tout en bas, -540 en haut d'un niveau de quatre écrans. Niveau d'un
/// écran (pas plus haut que l'écran) : `cam_y` tel quel, exactement comme avant la
/// carte 5.2 (secousse de mort comprise). Fonction pure.
pub fn camera_fond(cam_y: i32, hauteur_niveau: i32) -> i32 {
    cam_y - (hauteur_niveau - H).max(0)
}

/// Descente verticale (px, vers le bas de l'écran) d'une couche de silhouettes de
/// `facteur` quand la caméra du fond est en `cam_y_fond` : nulle en bas d'un niveau,
/// d'autant plus grande que la caméra monte (les silhouettes restent « en bas », sous
/// Pépin qui grimpe), et toujours plus petite que le défilement des tuiles. Fonction pure.
pub fn enfoncement(cam_y_fond: i32, facteur: f32) -> i32 {
    -decalage(cam_y_fond, facteur)
}

/// Nombre pseudo-aléatoire stable dérivé d'un entier (même entrée, même sortie).
fn hache(n: i32, graine: u32) -> u32 {
    let mut h = (n as u32).wrapping_mul(0x9E37_79B1) ^ graine.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h
}

/// Un immeuble (ou un banc de nuages) de la couche : `(x, largeur, hauteur, cheminee)`
/// dans le repère de la couche, pour la cellule d'indice `cellule` (48 px de large).
pub fn immeuble(cellule: i32, graine: u32) -> (i32, i32, i32, i32) {
    let h = hache(cellule, graine);
    let largeur = 28 + (h % 17) as i32;
    let hauteur = 22 + ((h >> 5) % 40) as i32;
    let cheminee = 6 + ((h >> 11) % (largeur as u32 - 12)) as i32;
    (
        cellule * 48 + ((h >> 17) % 6) as i32,
        largeur,
        hauteur,
        cheminee,
    )
}

/// Hauteur totale (en pixels) de la silhouette de la cellule `cellule`, pour la
/// forme de l'ambiance et `relief` pixels de plus. Pure : sert au dessin ET aux tests.
pub fn hauteur_silhouette(a: &Ambiance, cellule: i32, graine: u32, relief: i32) -> i32 {
    let (_, _, h, _) = immeuble(cellule, graine);
    match a.forme {
        // Corps + cheminée qui dépasse de 10 px.
        Forme::Immeubles => h + relief + 10,
        Forme::Nuages => 8 + h / 3 + relief,
        // Quatre étages égaux (arrondis vers le haut).
        Forme::Pics => 4 * ((h + relief + 3) / 4),
    }
}

/// Fenêtre allumée `k` (0 à 2) de l'immeuble de la cellule `cellule` : décalage
/// `(dx, dy)` depuis le coin haut-gauche du corps, ou `None` (environ une fenêtre
/// sur trois seulement existe). Toujours à l'intérieur du corps du bâtiment.
pub fn fenetre(
    cellule: i32,
    graine: u32,
    k: i32,
    largeur: i32,
    hauteur: i32,
) -> Option<(i32, i32)> {
    let h = hache(cellule * 3 + k, graine ^ 0xF3);
    if !h.is_multiple_of(3) {
        return None;
    }
    let dx = 3 + ((h >> 4) % (largeur as u32 - 7)) as i32;
    let dy = 5 + ((h >> 12) % (hauteur as u32 - 10)) as i32;
    Some((dx, dy))
}

/// Îlot de nuages de la cellule `cellule` (72 px de large) : `(x, y, largeur)` dans le
/// repère de la couche, ou `None` (une cellule sur trois est vide).
pub fn ilot(cellule: i32) -> Option<(i32, i32, i32)> {
    let h = hache(cellule, 41);
    if h.is_multiple_of(3) {
        return None;
    }
    Some((
        cellule * CELLULE_ILOT + ((h >> 3) % 24) as i32,
        ILOT_Y_MIN + ((h >> 8) % ILOT_Y_ALEA) as i32,
        18 + ((h >> 14) % 15) as i32,
    ))
}

/// Nuage du plafond de la cellule `cellule` (56 px de large) : `(x, largeur, profondeur)`
/// dans le repère de la couche. `ampleur` (2 à 4) règle l'épaisseur : la profondeur va de
/// 6 à 6 + 6 * ampleur px, toujours dans la bande de ciel du haut. Pure.
pub fn nuage_plafond(cellule: i32, graine: u32, ampleur: i32) -> (i32, i32, i32) {
    let h = hache(cellule, graine);
    let largeur = 40 + (h % 25) as i32;
    let profondeur = 6 + (((h >> 5) % 18) as i32 * ampleur) / 3;
    (
        cellule * CELLULE_PLAFOND + ((h >> 17) % 8) as i32,
        largeur,
        profondeur,
    )
}

/// Épaisseur maximale d'un nuage du plafond (mesurée sur 200 cellules).
#[cfg(test)]
pub(crate) fn profondeur_plafond_max(ampleur: i32) -> i32 {
    [GRAINE_PLAFOND_LOIN, GRAINE_PLAFOND_PROCHE]
        .into_iter()
        .flat_map(|g| (-100..100).map(move |c| nuage_plafond(c, g, ampleur).2))
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
/// Une zone d'écran occupée par une couche de silhouettes : couleur et ordonnées
/// `[haut, bas)` occupées (avant parallaxe verticale), pour le contrôle de contraste.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Zone {
    pub nom: &'static str,
    pub couleur: Couleur,
    pub haut: i32,
    pub bas: i32,
}

#[cfg(test)]
/// Les couches de silhouettes d'une ambiance (lointaine, proche, îlots) avec la zone
/// d'écran où elles se dessinent. Hauteur maximale mesurée sur 200 cellules.
pub(crate) fn zones(a: &Ambiance) -> Vec<Zone> {
    let max_h = |graine: u32, relief: i32| {
        (-100..100)
            .map(|c| hauteur_silhouette(a, c, graine, relief))
            .max()
            .unwrap_or(0)
    };
    let mut v = vec![
        Zone {
            nom: "lointaine",
            couleur: a.lointain,
            haut: BASE_LOINTAINE - max_h(GRAINE_LOINTAINE, a.relief),
            bas: if a.socle { H } else { BASE_LOINTAINE },
        },
        Zone {
            nom: "proche",
            couleur: a.proche,
            haut: BASE_PROCHE + a.descente - max_h(GRAINE_PROCHE, 0),
            bas: H,
        },
    ];
    if let Some(p) = a.plafond {
        let bas = profondeur_plafond_max(p.ampleur);
        for (nom, couleur) in [("plafond-loin", p.loin), ("plafond-proche", p.proche)] {
            v.push(Zone {
                nom,
                couleur,
                haut: 0,
                bas,
            });
        }
    }
    if let Some((corps, dessous)) = a.ilots {
        for (nom, couleur) in [("ilots", corps), ("ilots-dessous", dessous)] {
            v.push(Zone {
                nom,
                couleur,
                haut: ILOT_Y_MIN,
                bas: ILOT_Y_MIN + ILOT_Y_ALEA as i32 + ILOT_HAUTEUR,
            });
        }
    }
    v
}

/// Position `(x, y)` écran de la goutte de pluie `i` au pas `pas`, pour une couche
/// de vitesse `vitesse` (px par pas vers le bas). Chute en diagonale (1 px de
/// dérive vers la gauche pour 2 de chute) et boucle sur l'écran.
pub fn goutte_de_pluie(i: i32, pas: u32, vitesse: i32, graine: u32) -> (i32, i32) {
    let h = hache(i, graine);
    let x0 = (h % L as u32) as i32;
    let y0 = ((h >> 9) % H as u32) as i32;
    let t = pas as i32 * vitesse;
    ((x0 - t / 2).rem_euclid(L), (y0 + t).rem_euclid(H))
}

/// Nombre maximal de gouttes par couche (garde-fou des tests).
#[cfg(test)]
const GOUTTES_MAX: i32 = 64;

fn rect(x: i32, y: i32, w: i32, h: i32, c: Couleur) {
    draw_rectangle(x as f32, y as f32, w as f32, h as f32, couleur_mq(c));
}

/// Le ciel : quatre bandes séparées par une petite zone « tramée » (lignes alternées).
fn dessiner_ciel(a: &Ambiance) {
    let bande = H / 4;
    for (i, &c) in a.ciel.iter().enumerate() {
        let y = i as i32 * bande;
        rect(0, y, L, bande + 1, c);
        // Trame : sur les 6 dernières lignes de la bande, une ligne sur deux
        // reprend la couleur de la bande suivante.
        if let Some(&suivante) = a.ciel.get(i + 1) {
            for k in 0..3 {
                rect(0, y + bande - 6 + k * 2 + 1, L, 1, suivante);
            }
        }
    }
}

fn dessiner_etoiles(pas: u32, cam_x: i32, ymax: i32) {
    let dec = decalage(cam_x, 0.03);
    for i in 0..40 {
        let h = hache(i, 77);
        let x = ((h % (L as u32 + 64)) as i32 - dec).rem_euclid(L);
        let y = ((h >> 10) % ymax as u32) as i32;
        // Scintillement : certaines étoiles s'éteignent un instant.
        let eteinte = (pas / 20 + h % 7).is_multiple_of(9);
        let couleur = if h.is_multiple_of(5) {
            OR_PALE
        } else {
            ARDOISE_PALE
        };
        if !eteinte {
            rect(x, y, 1, 1, couleur);
        }
    }
}

fn dessiner_soleil(cam_x: i32) {
    let (cx, cy, r) = (236 - decalage(cam_x, 0.02) % 40, 44, 22);
    for dy in -r..=r {
        let w = ((r * r - dy * dy) as f32).sqrt() as i32;
        rect(cx - w, cy + dy, 2 * w + 1, 1, OR_PALE);
    }
    let r2 = 14;
    for dy in -r2..=r2 {
        let w = ((r2 * r2 - dy * dy) as f32).sqrt() as i32;
        rect(cx - w, cy + dy, 2 * w + 1, 1, BLANC_CASSE);
    }
}

/// Une couche de silhouettes. `base` : ordonnée du sol de la couche. `relief` :
/// pixels ajoutés à la hauteur. `fenetres` : couleur des fenêtres allumées (immeubles).
#[allow(clippy::too_many_arguments)]
fn dessiner_couche(
    a: &Ambiance,
    cam_x: i32,
    base: i32,
    facteur: f32,
    couleur: Couleur,
    graine: u32,
    relief: i32,
    fenetres: Option<Couleur>,
) {
    let dec = decalage(cam_x, facteur);
    let premiere = dec.div_euclid(48) - 1;
    let derniere = (dec + L).div_euclid(48) + 1;
    for cellule in premiere..=derniere {
        let (x, l, h, chem) = immeuble(cellule, graine);
        let x = x - dec;
        let h = h + relief;
        let y = base - h;
        match a.forme {
            Forme::Nuages => {
                // Banc de nuages : trois étages de plus en plus étroits (un dôme).
                let hn = hauteur_silhouette(a, cellule, graine, relief);
                rect(x, base - hn / 2, l, hn / 2, couleur);
                rect(x + l / 8, base - hn * 3 / 4, l * 3 / 4, hn / 4, couleur);
                rect(x + l / 3, base - hn, l / 3, hn / 4, couleur);
            }
            Forme::Pics => {
                // Tour de nuages : quatre étages égaux, de plus en plus étroits ; la
                // position du sommet (`chem`) change d'une cellule à l'autre.
                let etage = hauteur_silhouette(a, cellule, graine, relief) / 4;
                for s in 0..4 {
                    let w = l - s * l / 5;
                    let decal = (l - w) * (chem - 6) / (l - 12);
                    rect(x + decal, base - (s + 1) * etage, w, etage, couleur);
                }
            }
            Forme::Immeubles => {
                // Immeuble : corps, toit en gradins, cheminée.
                rect(x, y, l, h, couleur);
                rect(x + 3, y - 2, l - 6, 2, couleur);
                rect(x + 7, y - 4, l - 14, 2, couleur);
                rect(x + chem, y - 10, 5, 8, couleur);
                if let Some(c) = fenetres {
                    for k in 0..3 {
                        if let Some((dx, dy)) = fenetre(cellule, graine, k, l, h) {
                            rect(x + dx, y + dy, 1, 2, c);
                        }
                    }
                }
            }
        }
    }
    // Socle : la couche lointaine descend jusqu'au bas de l'écran, sans trouée.
    if a.socle && base < H {
        rect(0, base, L, H - base, couleur);
    }
}

/// Plafond de nuages d'orage : dômes renversés accrochés en haut de l'écran
/// (trois étages de plus en plus étroits vers le bas).
fn dessiner_plafond(cam_x: i32, facteur: f32, graine: u32, p: &Plafond, couleur: Couleur) {
    let dec = decalage(cam_x, facteur);
    let premiere = dec.div_euclid(CELLULE_PLAFOND) - 1;
    let derniere = (dec + L).div_euclid(CELLULE_PLAFOND) + 1;
    for cellule in premiere..=derniere {
        let (x, l, d) = nuage_plafond(cellule, graine, p.ampleur);
        let x = x - dec;
        rect(x, 0, l, d / 2, couleur);
        rect(x + l / 8, d / 2, l * 3 / 4, d / 4, couleur);
        rect(x + l / 3, d / 2 + d / 4, l / 3, d - d / 2 - d / 4, couleur);
    }
}

/// Îlots de nuages flottants (couche intermédiaire), dessinés en blocs entiers.
/// Le dessus est arrondi (1 px sur un quart de la largeur, puis 1 px sur la moitié)
/// pour ne pas ressembler à une plateforme plate.
fn dessiner_ilots(corps: Couleur, dessous: Couleur, cam_x: i32, vert: i32) {
    let dec = decalage(cam_x, FACTEUR_ILOTS);
    let premiere = dec.div_euclid(CELLULE_ILOT) - 1;
    let derniere = (dec + L).div_euclid(CELLULE_ILOT) + 1;
    for cellule in premiere..=derniere {
        if let Some((x, y, l)) = ilot(cellule) {
            let (x, y) = (x - dec, y + vert);
            rect(x + 3 * l / 8, y, l / 4, 1, corps);
            rect(x + l / 4, y + 1, l / 2, 1, corps);
            rect(x, y + 2, l, 4, corps);
            rect(x + 3, y + 6, l - 6, ILOT_HAUTEUR - 6, dessous);
        }
    }
}

fn dessiner_pluie(a: &Ambiance, pas: u32) {
    for (couche, (vitesse, couleur, longueur)) in [(2, a.pluie.0, 1), (3, a.pluie.1, 2)]
        .into_iter()
        .enumerate()
    {
        for i in 0..a.gouttes {
            let (x, y) = goutte_de_pluie(i, pas, vitesse, 100 + couche as u32);
            // Un trait diagonal de 1 à 2 pixels, calé sur la grille.
            for k in 0..longueur + 1 {
                rect(x + k, y - k * 2, 1, 1, couleur);
            }
        }
    }
}

/// Dessine tout ce qui est DERRIÈRE le niveau (ciel, étoiles, soleil, silhouettes).
/// `cam_y` : ordonnée de la caméra vue par le fond (`camera_fond`).
pub fn dessiner_fond(theme: Theme, cam_x: i32, cam_y: i32, pas: u32) {
    let a = ambiance(theme);
    dessiner_ciel(&a);
    if let Some(ymax) = a.etoiles {
        dessiner_etoiles(pas, cam_x, ymax);
    }
    if a.soleil {
        dessiner_soleil(cam_x);
    }
    if let Some(p) = &a.plafond {
        dessiner_plafond(cam_x, FACTEUR_PLAFOND_LOIN, GRAINE_PLAFOND_LOIN, p, p.loin);
        dessiner_plafond(
            cam_x,
            FACTEUR_PLAFOND_PROCHE,
            GRAINE_PLAFOND_PROCHE,
            p,
            p.proche,
        );
    }
    // Les silhouettes se décalent aussi un peu à la verticale (parallaxe verticale),
    // davantage au clocher (`vertige`).
    let vert = |f: f32| enfoncement(cam_y, f * a.vertige);
    if let Some((corps, dessous)) = a.ilots {
        dessiner_ilots(corps, dessous, cam_x, vert(VERT_ILOTS));
    }
    let (fen_loin, fen_proche) = match a.fenetres {
        Some((l, p)) => (Some(l), Some(p)),
        None => (None, None),
    };
    dessiner_couche(
        &a,
        cam_x,
        BASE_LOINTAINE + vert(VERT_LOINTAINE),
        FACTEUR_LOINTAINE,
        a.lointain,
        GRAINE_LOINTAINE,
        a.relief,
        fen_loin,
    );
    dessiner_couche(
        &a,
        cam_x,
        BASE_PROCHE + a.descente + vert(VERT_PROCHE),
        FACTEUR_PROCHE,
        a.proche,
        GRAINE_PROCHE,
        0,
        fen_proche,
    );
}

/// Dessine la pluie PAR-DESSUS le niveau (devant tout).
pub fn dessiner_avant(theme: Theme, pas: u32) {
    if theme.pluie() {
        dessiner_pluie(&ambiance(theme), pas);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Teinte des tuiles et dalles friables : mêmes valeurs qu'avant la carte 5.1 (elles
    /// étaient écrites dans `sprites::remplacer` et `scene.rs`) ; seule leur place change.
    #[test]
    fn teinte_des_tuiles_et_friables_par_theme() {
        let attendu = |t: Theme| match t {
            Theme::Toits | Theme::Orage | Theme::Clocher => (0, false, false),
            Theme::Gouttieres | Theme::Marche => (1, false, false),
            Theme::Cyclone => (0, false, true),
            Theme::Altitude => (2, false, true),
            Theme::Soleil => (3, true, true),
        };
        for t in Theme::TOUS {
            let a = ambiance(t);
            let lu = (
                a.tuiles.eclaircissement,
                a.tuiles.briques_orangees,
                a.friable_nuage,
            );
            assert_eq!(lu, attendu(t), "{t:?}");
        }
    }

    #[test]
    fn toutes_les_couleurs_du_decor_sont_dans_la_palette() {
        for t in Theme::TOUS {
            let a = ambiance(t);
            let mut couleurs = a.ciel.to_vec();
            couleurs.extend([a.lointain, a.proche, a.pluie.0, a.pluie.1]);
            for (c1, c2) in a.fenetres.into_iter().chain(a.ilots) {
                couleurs.extend([c1, c2]);
            }
            if let Some(p) = a.plafond {
                couleurs.extend([p.loin, p.proche]);
            }
            for c in couleurs {
                assert!(TOUTES.contains(&c), "{t:?} : couleur hors palette {c:?}");
            }
        }
    }

    #[test]
    fn aucune_silhouette_n_a_la_couleur_du_ciel_derriere_elle() {
        for t in Theme::TOUS {
            let a = ambiance(t);
            for z in zones(&a) {
                assert!(z.haut < z.bas, "{t:?} {} : zone vide", z.nom);
                for y in z.haut..z.bas {
                    assert_ne!(
                        z.couleur,
                        ciel_a(&a, y),
                        "{t:?} : la couche {} a la couleur du ciel à y={y}",
                        z.nom
                    );
                }
            }
        }
    }

    #[test]
    fn la_couche_lointaine_monte_a_40_px_au_moins_dans_chaque_theme() {
        for t in Theme::TOUS {
            let a = ambiance(t);
            // Le niveau 5 est l'exception voulue : ses bancs de nuages restent BAS
            // (sous la ligne du sol, test ci-dessous) pour ne pas imiter les plateformes.
            if a.forme == Forme::Nuages {
                continue;
            }
            let max = (-100..100)
                .map(|c| hauteur_silhouette(&a, c, GRAINE_LOINTAINE, a.relief))
                .max()
                .unwrap();
            assert!(max >= 40, "{t:?} : fond « aplat », silhouette de {max} px");
        }
        // Les deux niveaux signalés ont de vraies silhouettes hautes.
        for t in [Theme::Orage, Theme::Altitude] {
            let a = ambiance(t);
            let z = zones(&a)[0];
            assert!(z.bas - z.haut >= 60, "{t:?} : couche lointaine trop basse");
        }
    }

    #[test]
    fn le_niveau_3_a_des_immeubles_avec_quelques_fenetres_allumees() {
        let a = ambiance(Theme::Orage);
        assert_eq!(a.forme, Forme::Immeubles);
        assert!(a.fenetres.is_some());
        let (mut total, mut cellules) = (0, 0);
        for c in -100..100 {
            let (_, l, h, _) = immeuble(c, GRAINE_LOINTAINE);
            let h = h + a.relief;
            cellules += 1;
            for k in 0..3 {
                if let Some((dx, dy)) = fenetre(c, GRAINE_LOINTAINE, k, l, h) {
                    total += 1;
                    // Une fenêtre de 1x2 reste dans le corps du bâtiment.
                    assert!((0..l).contains(&dx) && dy >= 0 && dy + 2 <= h, "{dx},{dy}");
                }
            }
        }
        // Peu nombreuses : entre 0,3 et 1,5 par immeuble en moyenne.
        assert!(
            total > cellules * 3 / 10 && total < cellules * 3 / 2,
            "{total}"
        );
    }

    #[test]
    fn les_ilots_du_niveau_4_restent_dans_le_tiers_median() {
        assert!(ambiance(Theme::Altitude).ilots.is_some());
        let mut vus = 0;
        for c in -100..100 {
            if let Some((_, y, l)) = ilot(c) {
                vus += 1;
                assert!(
                    y >= ILOT_Y_MIN
                        && y + ILOT_HAUTEUR <= ILOT_Y_MIN + ILOT_Y_ALEA as i32 + ILOT_HAUTEUR
                );
                assert!((18..33).contains(&l));
                assert!(y >= H / 4 && y + ILOT_HAUTEUR <= 3 * H / 4);
            }
        }
        assert!(vus > 100, "trop peu d'îlots : {vus}");
    }

    #[test]
    fn le_ciel_s_eclaircit_de_l_orage_au_soleil() {
        let clarte = |t: Theme| {
            let c = ambiance(t).ciel[3];
            c.r as u32 + c.g as u32 + c.b as u32
        };
        assert!(clarte(Theme::Toits) < clarte(Theme::Gouttieres));
        assert!(clarte(Theme::Gouttieres) < clarte(Theme::Altitude));
        assert!(clarte(Theme::Altitude) < clarte(Theme::Soleil));
    }

    #[test]
    fn la_parallaxe_est_entiere_et_proportionnelle() {
        assert_eq!(decalage(0, 0.35), 0);
        assert_eq!(decalage(100, 0.5), 50);
        assert_eq!(decalage(1000, 0.15), 150);
        // La couche lointaine bouge moins que la proche.
        assert!(decalage(777, 0.15) < decalage(777, 0.35));
    }

    /// Niveau d'un écran (176 px) : la caméra du fond est la caméra elle-même, donc les
    /// silhouettes sont exactement là où elles étaient avant la carte 5.2, y compris
    /// pendant la secousse de mort (caméra à -5 ou -3 au lieu de -4).
    #[test]
    fn un_niveau_d_un_ecran_garde_le_meme_fond() {
        for cam_y in [-6, -5, -4, -3, -2] {
            assert_eq!(camera_fond(cam_y, 176), cam_y);
            assert_eq!(camera_fond(cam_y, 180), cam_y);
            // L'ancienne formule était `-decalage(cam_y, f)` avec 0,07, 0,05 et 0,10.
            for (f, ancien) in [
                (VERT_ILOTS, 0.07),
                (VERT_LOINTAINE, 0.05),
                (VERT_PROCHE, 0.10),
            ] {
                assert_eq!(
                    enfoncement(camera_fond(cam_y, 176), f),
                    -decalage(cam_y, ancien)
                );
            }
        }
    }

    /// Niveau haut : en bas, le fond est celui d'un niveau d'un écran ; plus la caméra
    /// monte, plus les silhouettes descendent, la proche plus vite que les îlots, les îlots
    /// plus vite que la lointaine, et toutes moins vite que les tuiles.
    #[test]
    fn en_hauteur_les_silhouettes_descendent_quand_on_monte() {
        let hauteur = 45 * 16; // quatre écrans
        let bas = hauteur - H;
        assert_eq!(camera_fond(bas, hauteur), 0);
        assert_eq!(camera_fond(0, hauteur), -bas);
        for f in [VERT_LOINTAINE, VERT_ILOTS, VERT_PROCHE] {
            assert_eq!(enfoncement(camera_fond(bas, hauteur), f), 0);
        }
        let mut avant = (0, 0, 0);
        for cam_y in (0..=bas).rev() {
            let c = camera_fond(cam_y, hauteur);
            let montee = bas - cam_y;
            let e = (
                enfoncement(c, VERT_LOINTAINE),
                enfoncement(c, VERT_ILOTS),
                enfoncement(c, VERT_PROCHE),
            );
            assert!(
                e.0 >= avant.0 && e.1 >= avant.1 && e.2 >= avant.2,
                "{cam_y}"
            );
            assert!(e.0 <= e.1 && e.1 <= e.2 && e.2 <= montee, "{cam_y} : {e:?}");
            avant = e;
        }
        assert_eq!(avant, (27, 38, 54), "en haut d'un niveau de quatre écrans");
    }

    /// Le clocher (45 rangées, quatre écrans) : au sommet, la ville s'est nettement
    /// enfoncée — la couche proche de plus de 100 px, la lointaine de plus de 50 —
    /// donc bien plus de ciel à l'écran : on sent la hauteur (note du coordinateur).
    /// Les autres thèmes gardent la parallaxe verticale de la carte 5.2 (`vertige`
    /// à 1) : leur rendu ne change pas.
    #[test]
    fn au_sommet_du_clocher_la_ville_s_est_enfoncee() {
        let a = ambiance(Theme::Clocher);
        let hauteur = 45 * 16;
        let sommet = camera_fond(0, hauteur);
        let proche = enfoncement(sommet, VERT_PROCHE * a.vertige);
        let loin = enfoncement(sommet, VERT_LOINTAINE * a.vertige);
        assert_eq!((proche, loin), (135, 68));
        assert!(proche < hauteur - H, "moins vite que les tuiles");
        for t in Theme::TOUS.into_iter().filter(|&t| t != Theme::Clocher) {
            assert!(ambiance(t).vertige == 1.0, "{t:?}");
        }
    }

    /// En descendant (niveau haut), une couche passe devant des bandes de ciel plus
    /// basses : sa couleur doit rester différente de TOUT le ciel qu'elle peut couvrir,
    /// pour une caméra jusqu'à 64 rangées de haut (plus que les quatre écrans prévus).
    #[test]
    fn en_hauteur_aucune_silhouette_n_a_la_couleur_du_ciel() {
        let montee_max = 64 * 16 - H;
        for t in Theme::TOUS {
            let a = ambiance(t);
            for z in zones(&a) {
                let facteur = a.vertige
                    * match z.nom {
                        "lointaine" => VERT_LOINTAINE,
                        "proche" => VERT_PROCHE,
                        "ilots" | "ilots-dessous" => VERT_ILOTS,
                        // Le plafond est fixé en haut de l'écran : il ne descend pas.
                        _ => continue,
                    };
                let s = enfoncement(-montee_max, facteur);
                for y in z.haut..(z.bas + s).min(H) {
                    assert_ne!(
                        z.couleur,
                        ciel_a(&a, y),
                        "{t:?} : la couche {} a la couleur du ciel à y={y}",
                        z.nom
                    );
                }
            }
        }
    }

    #[test]
    fn la_pluie_reste_a_l_ecran_et_tombe_en_diagonale() {
        for pas in [0, 1, 59, 600, 123_456] {
            for i in 0..GOUTTES_MAX {
                let (x, y) = goutte_de_pluie(i, pas, 3, 101);
                assert!((0..L).contains(&x) && (0..H).contains(&y), "({x},{y})");
            }
        }
        // Après 2 pas à vitesse 2 : 4 px de chute, 2 px de dérive à gauche.
        let (x0, y0) = goutte_de_pluie(5, 0, 2, 100);
        let (x1, y1) = goutte_de_pluie(5, 2, 2, 100);
        assert_eq!((y1 - y0).rem_euclid(H), 4);
        assert_eq!((x0 - x1).rem_euclid(L), 2);
    }

    #[test]
    fn les_immeubles_sont_stables_et_bornes() {
        for c in -50..50 {
            assert_eq!(immeuble(c, 11), immeuble(c, 11));
            let (_, l, h, chem) = immeuble(c, 23);
            assert!((28..=44).contains(&l) && (22..62).contains(&h));
            assert!(chem >= 6 && chem + 5 <= l, "cheminée {chem} hors de {l}");
        }
    }

    /// I-04 : la couche proche (immeubles noirs, tours sombres) ne dépasse jamais de
    /// la ligne du sol, même avec la parallaxe verticale maximale (-10 px).
    #[test]
    fn la_couche_proche_ne_depasse_pas_de_la_ligne_du_sol() {
        for t in Theme::TOUS {
            let a = ambiance(t);
            let proche = zones(&a).into_iter().find(|z| z.nom == "proche").unwrap();
            if a.forme == Forme::Nuages {
                continue;
            }
            assert!(
                proche.haut - 10 >= LIGNE_SOL,
                "{t:?} : couche proche à y={} au-dessus du sol (y={LIGNE_SOL})",
                proche.haut - 10
            );
        }
    }

    /// I-03 : au niveau 5, les bancs de nuages du fond restent sous la ligne du sol
    /// (ils ne bouchent pas un trou à la hauteur des plateformes) et n'ont ni la
    /// couleur des plateformes ni celle du ciel.
    #[test]
    fn les_nuages_du_niveau_5_restent_sous_la_ligne_du_sol() {
        let a = ambiance(Theme::Soleil);
        for z in zones(&a).into_iter().filter(|z| z.nom != "proche") {
            assert!(z.haut >= LIGNE_SOL, "{} monte à y={}", z.nom, z.haut);
        }
        assert_ne!(a.lointain, NUAGE);
        assert_ne!(a.proche, BLANC_CASSE);
    }

    /// I-06 : plus de bande de ciel clair sous les immeubles de fond (niveaux 1 à 3).
    #[test]
    fn les_immeubles_de_fond_descendent_jusqu_en_bas() {
        for t in [Theme::Toits, Theme::Gouttieres, Theme::Orage] {
            assert!(ambiance(t).socle, "{t:?}");
        }
    }

    /// I-17 : l'or ne sert qu'aux gouttes à ramasser et aux éclairs.
    #[test]
    fn aucun_decor_n_utilise_l_or() {
        for t in Theme::TOUS {
            let a = ambiance(t);
            let (c1, c2) = a.fenetres.unwrap_or((ORANGE_SOLEIL, ORANGE_SOLEIL));
            assert!(c1 != OR && c2 != OR, "{t:?}");
        }
    }

    /// I-19 : les étoiles ne se dessinent que dans une bande de ciel où elles se voient.
    #[test]
    fn les_etoiles_se_voient_sur_leur_bande_de_ciel() {
        for t in Theme::TOUS {
            let a = ambiance(t);
            if let Some(ymax) = a.etoiles {
                for y in 0..ymax {
                    let fond = ciel_a(&a, y);
                    assert!(fond != BLANC_CASSE && fond != OR_PALE, "{t:?} y={y}");
                    assert!(
                        fond != CORAIL && fond != CORAIL_CLAIR && fond != ORANGE_SOLEIL,
                        "{t:?} : étoile invisible sur {fond:?} (y={y})"
                    );
                }
            }
        }
    }

    /// I-20 : la pluie diffère d'un niveau à l'autre, reste sous le garde-fou,
    /// et ses deux couleurs ne se confondent avec aucune bande de ciel.
    #[test]
    fn la_pluie_a_une_densite_par_niveau_et_se_voit() {
        let n: Vec<i32> = Theme::TOUS.iter().map(|t| ambiance(*t).gouttes).collect();
        assert_eq!(n, [24, 48, 36, 60, 16, 32, 18, 0]);
        for t in Theme::TOUS {
            let a = ambiance(t);
            assert!(a.gouttes <= GOUTTES_MAX, "{t:?}");
            if a.gouttes == 0 {
                continue;
            }
            for (couleur, nom) in [(a.pluie.0, "lointaine"), (a.pluie.1, "proche")] {
                for bande in a.ciel {
                    assert!(
                        couleur != bande,
                        "{t:?} : pluie {nom} invisible sur {bande:?}"
                    );
                }
            }
        }
    }

    /// I-21 : le plafond de nuages tient dans la bande du haut, sa couleur diffère
    /// de celle-ci, et il n'existe qu'aux gouttières, à l'orage et dans l'œil du
    /// cyclone (les nuages qui tournent autour de la trouée d'or).
    #[test]
    fn le_plafond_de_nuages_tient_dans_la_bande_du_haut() {
        for t in Theme::TOUS {
            let a = ambiance(t);
            let attendu = matches!(t, Theme::Gouttieres | Theme::Orage | Theme::Cyclone);
            assert_eq!(a.plafond.is_some(), attendu, "{t:?}");
            if let Some(p) = a.plafond {
                for c in [p.loin, p.proche] {
                    assert!(c != a.ciel[0], "{t:?} : plafond invisible sur le ciel");
                }
                assert!(profondeur_plafond_max(p.ampleur) <= H / 4, "{t:?}");
            }
        }
        for c in -50..50 {
            assert_eq!(nuage_plafond(c, 31, 4), nuage_plafond(c, 31, 4));
            let (_, l, d) = nuage_plafond(c, 31, 4);
            assert!((40..=64).contains(&l) && (6..=34).contains(&d), "{l} {d}");
        }
    }
}
