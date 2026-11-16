//! Cosmétiques visuels (carte 7.6) : TRAÎNÉES derrière Pépin et MOTIFS de parapluie,
//! vendus dans la boutique (carte 7.5) à côté des skins et des musiques (le choix de
//! la palette de couleurs, jugé moche par AGA, a été retiré à la carte 7.6b). Demande
//! d'AGA : « uniquement du cosmétique, pour que le score reste
//! comparable ». Rien ici ne touche au monde : ni physique, ni collision, ni score.
//! Tout est du RENDU, avec ses propres données (les brins de traînée ne sont pas les
//! particules du jeu).
//!
//! - Traînée : quelques brins légers, semés derrière Pépin QUAND IL BOUGE, dessinés
//!   sous les ennemis et les dangers (`scene::dessiner`) : ils ne cachent jamais ce
//!   qui compte.
//! - Motif : appliqué une fois pour toutes, au chargement, sur une copie des images
//!   du skin (`appliquer_motif`) : les pixels de la TOILE du parapluie (ses deux
//!   couleurs, dans la zone du parapluie, ouvert ou fermé) prennent le motif. Marche
//!   pour chaque skin, par-dessus la couleur de SON parapluie.
//!
//! Chaque catégorie a un choix « rien » (AUCUNE, UNI), gratuit et équipé
//! au départ : `None` dans le rendu, le jeu tel qu'il était.
//!
//! Notions Rust illustrées ici : fonctions génériques avec contrainte de trait
//! (`suivant<T: Copy + PartialEq>`), `Option<T>` comme « rien de choisi », tableaux
//! constants, générateur pseudo-aléatoire sans état (`alea`).

use macroquad::prelude::draw_rectangle;
use pepin_core::palette::{Couleur, BLANC_CASSE, NUAGE, OR, OR_PALE, PLUIE, PLUIE_SOMBRE};

use crate::monde::Monde;
use crate::musiques::Musique;
use crate::rendu::couleur_mq;
use crate::sauvegarde::Sauvegarde;
use crate::skins::Skin;

// --- Prix, en gouttes d'or (regroupés ici pour être retouchés facilement) ----------
//
// Total de la boutique : 3 traînées x 15 + 3 motifs x 15 + 5 musiques x 15
// = 165 gouttes, pour 224 gagnables dans le jeu de base (plus les
// courses contre le robot).

/// Prix d'une traînée.
pub const PRIX_TRAINEE: u32 = 15;
/// Prix d'un motif de parapluie.
pub const PRIX_MOTIF: u32 = 15;
/// Prix d'une musique (sauf ORAGE, la musique d'origine, gratuite).
pub const PRIX_MUSIQUE: u32 = 15;

// --- Identifiants du choix « rien », gratuits et équipés au départ -----------------
// (les mêmes que dans `sauvegarde::INVENTAIRE_DE_DEPART` et `EQUIPES_DE_DEPART`).

/// Pas de traînée.
pub const SANS_TRAINEE: &str = "aucune";
/// Parapluie uni (pas de motif).
pub const SANS_MOTIF: &str = "uni";
/// La musique de la zone (rien d'imposé).
pub const MUSIQUE_PAR_ZONE: &str = "zone";

/// Une traînée derrière Pépin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trainee {
    /// Petits points dorés qui retombent.
    Etincelles,
    /// Bulles cyan qui montent.
    Bulles,
    /// Plumes blanches qui tombent en se balançant.
    Plumes,
}

impl Trainee {
    pub const TOUTES: [Trainee; 3] = [Trainee::Etincelles, Trainee::Bulles, Trainee::Plumes];

    pub fn id(self) -> &'static str {
        match self {
            Trainee::Etincelles => "etincelles",
            Trainee::Bulles => "bulles",
            Trainee::Plumes => "plumes",
        }
    }

    pub fn nom(self) -> &'static str {
        match self {
            Trainee::Etincelles => "ÉTINCELLES",
            Trainee::Bulles => "BULLES",
            Trainee::Plumes => "PLUMES",
        }
    }
}

/// Un motif sur la toile du parapluie.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Motif {
    Pois,
    Rayures,
    Etoiles,
}

impl Motif {
    pub const TOUS: [Motif; 3] = [Motif::Pois, Motif::Rayures, Motif::Etoiles];

    pub fn id(self) -> &'static str {
        match self {
            Motif::Pois => "pois",
            Motif::Rayures => "rayures",
            Motif::Etoiles => "etoiles",
        }
    }

    pub fn nom(self) -> &'static str {
        match self {
            Motif::Pois => "POIS",
            Motif::Rayures => "RAYURES",
            Motif::Etoiles => "ÉTOILES",
        }
    }
}

/// L'élément suivant d'un catalogue, « rien » compris : rien → le premier → … → le
/// dernier → rien. Sert aux touches de mise au point du build de QA.
pub fn suivant<T: Copy + PartialEq>(tous: &[T], actuel: Option<T>) -> Option<T> {
    match actuel.and_then(|a| tous.iter().position(|&t| t == a)) {
        None => tous.first().copied(),
        Some(i) => tous.get(i + 1).copied(),
    }
}

/// Les cosmétiques en service (rendu et musique). `None` : rien d'imposé.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Choix {
    pub trainee: Option<Trainee>,
    pub motif: Option<Motif>,
    /// Musique imposée en jeu à la place de celle de la zone.
    pub musique: Option<Musique>,
}

impl Choix {
    /// Ce qui est équipé dans la sauvegarde (identifiant inconnu : rien).
    pub fn depuis(s: &Sauvegarde) -> Self {
        let equipe = |categorie: &str| s.equipe(categorie).unwrap_or("");
        Self {
            trainee: Trainee::TOUTES
                .into_iter()
                .find(|t| t.id() == equipe("trainee")),
            motif: Motif::TOUS.into_iter().find(|m| m.id() == equipe("motif")),
            musique: Musique::depuis_id(equipe("musique")),
        }
    }
}

// --- Motifs de parapluie ------------------------------------------------------------

/// La toile du parapluie d'un skin : ses deux couleurs (indices de palette, comme
/// dans les dessins) et les deux couleurs de motif qui s'y voient (claire pour les
/// pois et les rayures, vive pour les étoiles). Un `match` sans joker : un skin
/// ajouté au catalogue oblige à choisir les siennes.
fn toile(skin: Skin) -> ([char; 2], [char; 2]) {
    match skin {
        // Parapluie corail : pois et rayures blancs, étoiles d'or pâle (l'or vif se
        // voyait mal sur le corail).
        Skin::Pepin => (['8', '9'], ['7', 'C']),
        // Parapluie cyan : pois et rayures corail (comme son nœud), étoiles violettes.
        Skin::Pepinette => (['D', '6'], ['8', 'F']),
    }
}

/// Les baleines du parapluie ouvert (colonnes, dans l'image de 20 px) : les rayures
/// colorent un panneau sur deux, entre deux baleines.
const BALEINES: [usize; 3] = [6, 10, 13];

/// Le pixel `(x, y)` de la toile prend-il la couleur du motif ? Renvoie `Some(0)`
/// (couleur claire), `Some(1)` (couleur vive) ou `None` (la toile reste). `y` part
/// du haut de la toile ; `x` part du bord gauche de l'image (ouvert) ou de la
/// colonne du parapluie fermé (0 à 2).
fn motif_en(motif: Motif, x: usize, y: usize, ouvert: bool) -> Option<usize> {
    let oui = match (motif, ouvert) {
        // Pois en quinconce ; fermé : un pois sur deux rangées, au milieu.
        (Motif::Pois, true) => y % 2 == 1 && (x + 2 * (y / 2 % 2)) % 4 == 1,
        (Motif::Pois, false) => y.is_multiple_of(2) && x == 1,
        // Un panneau sur deux ; fermé : une rangée sur deux (sucre d'orge).
        (Motif::Rayures, true) => BALEINES.iter().filter(|&&b| x > b).count() % 2 == 1,
        // Rangées impaires : la rangée blanche du dessin d'origine (la 4e) tombe alors
        // dans le rythme ; avec les paires, le parapluie fermé de Pépin paraissait blanc.
        (Motif::Rayures, false) => y % 2 == 1,
        // Quelques points épars ; fermé : deux étincelles.
        (Motif::Etoiles, true) => (x * 3 + y * 7).is_multiple_of(11),
        (Motif::Etoiles, false) => (x + y) % 4 == 1,
    };
    let couleur = if motif == Motif::Etoiles { 1 } else { 0 };
    oui.then_some(couleur)
}

/// Copie d'une image du skin avec le motif sur la toile du parapluie. Le parapluie
/// est au même endroit dans toutes les images de tous les skins (carte 7.1) : FERMÉ,
/// dans les colonnes 12 à 14 des images de 16 px ; OUVERT, dans les 6 premières
/// rangées des images de 20 px (au-dessus du bord festonné). Les autres images
/// (icône des vies) ne changent pas. Fonction pure : testable, appelée au chargement.
pub fn appliquer_motif(image: &[&str], skin: Skin, motif: Motif) -> Vec<String> {
    let (couleurs, motifs) = toile(skin);
    let largeur = image.first().map_or(0, |l| l.chars().count());
    let dans_zone = |x: usize, y: usize| match largeur {
        16 => (12..=14).contains(&x),
        20 => y < 6,
        _ => false,
    };
    let est_toile = |x: usize, y: usize, c: char| dans_zone(x, y) && couleurs.contains(&c);
    // Haut de la toile : le motif suit le parapluie quand l'image monte d'un pixel.
    let haut = image
        .iter()
        .position(|l| l.chars().enumerate().any(|(x, c)| est_toile(x, 0, c)))
        .unwrap_or(0);
    image
        .iter()
        .enumerate()
        .map(|(y, ligne)| {
            ligne
                .chars()
                .enumerate()
                .map(|(x, c)| {
                    if !est_toile(x, y, c) || y < haut {
                        return c;
                    }
                    let (mx, ouvert) = if largeur == 20 {
                        (x, true)
                    } else {
                        (x - 12, false)
                    };
                    motif_en(motif, mx, y - haut, ouvert).map_or(c, |i| motifs[i])
                })
                .collect()
        })
        .collect()
}

// --- Traînées -----------------------------------------------------------------------

/// Brins vivants au plus (une traînée reste légère).
const MAX_BRINS: usize = 40;
/// Un brin semé tous les `CADENCE` pas quand Pépin bouge.
const CADENCE: u32 = 3;
/// Vitesse (px/s) au-dessus de laquelle Pépin « bouge » (celle de l'image de course).
const BOUGE: f32 = 8.0;

/// Nombre pseudo-aléatoire entre 0 et 1, tiré de `n` seul (pas d'état, pas de hasard
/// système : deux parties identiques ont les mêmes traînées).
fn alea(n: u32) -> f32 {
    let h = n.wrapping_mul(2_654_435_761).rotate_left(13) ^ 0x5bd1_e995;
    (h.wrapping_mul(0x27d4_eb2d) >> 16) as f32 / 65_535.0
}

#[derive(Clone, Copy, Debug)]
struct Brin {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    /// Pas vécus.
    age: u8,
    duree: u8,
    genre: Trainee,
    /// Tirage propre au brin (couleur, balancement).
    graine: u32,
}

/// Les brins de la traînée, en coordonnées du niveau.
#[derive(Clone, Debug, Default)]
pub struct Trainees {
    brins: Vec<Brin>,
    pas: u32,
}

impl Trainees {
    pub fn new() -> Self {
        Self::default()
    }

    /// Un pas : les brins avancent et vieillissent ; si Pépin bouge (et n'est pas en
    /// train de réapparaître), un nouveau brin naît derrière lui. Ne lit le monde
    /// qu'en lecture : rien du jeu ne change.
    pub fn actualiser(&mut self, choix: Option<Trainee>, monde: &Monde) {
        self.pas = self.pas.wrapping_add(1);
        for b in &mut self.brins {
            let t = f32::from(b.age);
            match b.genre {
                Trainee::Etincelles => b.vy += 0.03,
                // La bulle monte en ondulant, la plume descend en se balançant.
                Trainee::Bulles => b.vx = 0.25 * (t * 0.3 + (b.graine % 7) as f32).sin(),
                Trainee::Plumes => b.vx = 0.4 * (t * 0.15 + (b.graine % 7) as f32).sin(),
            }
            b.x += b.vx;
            b.y += b.vy;
            b.age += 1;
        }
        self.brins.retain(|b| b.age < b.duree);
        let j = &monde.joueur;
        let bouge = j.vitesse.x.abs() > BOUGE || j.vitesse.y.abs() > BOUGE;
        let Some(genre) = choix else {
            return;
        };
        if !bouge || monde.reapparition > 0 || !self.pas.is_multiple_of(CADENCE) {
            return;
        }
        if self.brins.len() >= MAX_BRINS {
            self.brins.remove(0);
        }
        let n = self.pas;
        let dos = if j.regarde_droite { -1.0 } else { 1.0 };
        let b = &j.boite;
        let x = b.x + b.w / 2.0 + dos * (b.w / 2.0 + 1.0) + (alea(n) - 0.5) * 3.0;
        let y = b.y + b.h * 0.55 + (alea(n + 7) - 0.5) * 6.0;
        let (vx, vy, duree) = match genre {
            Trainee::Etincelles => (
                dos * 0.3 + (alea(n + 3) - 0.5) * 0.4,
                -0.4 * alea(n + 5),
                18,
            ),
            Trainee::Bulles => (0.0, -0.35 - 0.2 * alea(n + 5), 30),
            Trainee::Plumes => (0.0, 0.2 + 0.1 * alea(n + 5), 36),
        };
        self.brins.push(Brin {
            x,
            y,
            vx,
            vy,
            age: 0,
            duree,
            genre,
            graine: n,
        });
    }

    /// Dessine les brins (caméra entière `(cx, cy)`), aux pixels entiers.
    pub fn dessiner(&self, cx: i32, cy: i32) {
        for b in &self.brins {
            let (x, y) = (b.x.round() as i32 - cx, b.y.round() as i32 - cy);
            dessiner_brin(b.genre, x, y, b.age, b.graine);
        }
    }
}

fn point(x: i32, y: i32, l: i32, couleur: Couleur) {
    draw_rectangle(x as f32, y as f32, l as f32, 1.0, couleur_mq(couleur));
}

/// Un brin à l'écran, `(x, y)` en pixels logiques.
fn dessiner_brin(genre: Trainee, x: i32, y: i32, age: u8, graine: u32) {
    match genre {
        // Une petite croix d'or au cœur pâle, puis un simple point d'or qui retombe
        // (un point seul se perdait dans la pluie : captures 76-trainee-etincelles).
        Trainee::Etincelles => {
            if age < 8 + (graine % 3) as u8 {
                point(x - 1, y, 3, OR);
                point(x, y - 1, 1, OR);
                point(x, y + 1, 1, OR);
                point(x, y, 1, OR_PALE);
            } else {
                point(x, y, 1, OR);
            }
        }
        // Une petite bulle : un anneau de 4 pixels, plus sombre en vieillissant.
        Trainee::Bulles => {
            let c = if age < 20 { PLUIE } else { PLUIE_SOMBRE };
            point(x + 1, y, 1, c);
            point(x, y + 1, 1, c);
            point(x + 2, y + 1, 1, c);
            point(x + 1, y + 2, 1, c);
        }
        // Une plume : un trait blanc de 3 pixels, la pointe qui bascule.
        Trainee::Plumes => {
            point(x, y, 3, BLANC_CASSE);
            let penche = (u32::from(age) / 8 + graine).is_multiple_of(2);
            point(x + 3, if penche { y - 1 } else { y + 1 }, 1, NUAGE);
        }
    }
}

/// Aperçu de la boutique : quelques brins qui filent vers la gauche, derrière un
/// Pépin qui court en `(x + 8, y)` (pixels logiques). `anim` : compteur d'images.
pub fn dessiner_apercu_trainee(genre: Trainee, x: f32, y: f32, anim: u32) {
    for k in 0..4u32 {
        let age = (anim + k * 9) % 36;
        let bx = x as i32 + 10 - (age / 4) as i32;
        let dy = match genre {
            Trainee::Etincelles => (age / 6) as i32,
            Trainee::Bulles => -((age / 5) as i32),
            Trainee::Plumes => (age / 7) as i32,
        };
        let by = y as i32 + 9 + (k % 2) as i32 * 2 + dy;
        dessiner_brin(genre, bx, by, (age % 30) as u8, k);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un motif ne touche que la toile du parapluie, ouvert et fermé, pour chaque skin.
    #[test]
    fn les_motifs_restent_sur_la_toile() {
        for skin in Skin::TOUS {
            let images = skin.images();
            let (couleurs, _) = toile(skin);
            for motif in Motif::TOUS {
                for nom in [images.repos[0], images.planer[0]] {
                    let image = skin.sprites().iter().find(|(n, _)| *n == nom).unwrap().1;
                    let avec = appliquer_motif(image, skin, motif);
                    let mut changes = 0;
                    for (a, b) in image.iter().zip(&avec) {
                        for (ca, cb) in a.chars().zip(b.chars()) {
                            if ca != cb {
                                assert!(couleurs.contains(&ca), "{skin:?} {nom} : {ca}");
                                changes += 1;
                            }
                        }
                    }
                    assert!(changes >= 1, "{skin:?} {motif:?} {nom} : motif invisible");
                }
            }
        }
    }
}
