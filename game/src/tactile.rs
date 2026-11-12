//! Commandes tactiles : quatre boutons virtuels (gauche, droite, saut, pause)
//! dessinés en pixel art par-dessus le jeu, uniquement sur un écran tactile.
//!
//! `Tactile` est une `SourceEntrees` comme le clavier : le jeu ne voit que des
//! `Commandes`. Plusieurs doigts en même temps sont gérés (courir ET sauter).
//!
//! Détection : l'écran est jugé tactile dès le premier toucher reçu. Avant, les
//! boutons restent cachés (un joueur au clavier ne les verra jamais).
//!
//! Les calculs (disposition, zones touchées) sont purs et testés ; seuls `lire`
//! et `dessiner` appellent macroquad.
//!
//! Notions Rust illustrées ici : struct de données (`Zone`), fonction pure qui
//! transforme des points en `Commandes`, `Rc<Cell<..>>` pour partager un état
//! entre deux propriétaires (programme mono-thread).

use std::cell::Cell;
use std::rc::Rc;

use macroquad::prelude::{
    draw_rectangle, screen_dpi_scale, screen_height, screen_width, touches, Color, TouchPhase,
};
use pepin_core::palette::{ARDOISE_NUIT, BLANC_CASSE, OR, PLUIE};

use crate::entrees::{Commandes, SourceEntrees};
use crate::rendu::couleur_mq;

/// Un rectangle en pixels d'écran (coin haut-gauche + taille).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Zone {
    pub x: f32,
    pub y: f32,
    pub l: f32,
    pub h: f32,
}

impl Zone {
    fn contient(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.l && py >= self.y && py < self.y + self.h
    }

    /// Même rectangle agrandi de `m` pixels de chaque côté.
    fn agrandie(&self, m: f32) -> Zone {
        Zone {
            x: self.x - m,
            y: self.y - m,
            l: self.l + 2.0 * m,
            h: self.h + 2.0 * m,
        }
    }
}

/// Position des quatre boutons : l'aspect visible et la zone de toucher
/// (plus grande que le dessin, pour pardonner un doigt imprécis).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Disposition {
    pub gauche: Zone,
    pub droite: Zone,
    pub saut: Zone,
    pub pause: Zone,
    /// Zones sensibles correspondantes (même ordre).
    pub touche_gauche: Zone,
    pub touche_droite: Zone,
    pub touche_saut: Zone,
    pub touche_pause: Zone,
}

impl Disposition {
    /// Disposition pour un écran de `l` x `h` pixels : flèches en bas à gauche,
    /// gros bouton de saut en bas à droite, petite pause en haut à droite.
    pub fn calculer(l: f32, h: f32) -> Self {
        let petit_cote = l.min(h);
        let taille = (petit_cote * 0.26).clamp(56.0, 180.0);
        let marge = (petit_cote * 0.04).max(8.0);
        let bas = h - marge - taille;
        let gauche = Zone {
            x: marge,
            y: bas,
            l: taille,
            h: taille,
        };
        let droite = Zone {
            x: marge * 2.0 + taille,
            ..gauche
        };
        let grand = taille * 1.3;
        let saut = Zone {
            x: l - marge - grand,
            y: h - marge - grand,
            l: grand,
            h: grand,
        };
        let cote_pause = taille * 0.5;
        let pause = Zone {
            x: l - marge - cote_pause,
            y: marge,
            l: cote_pause,
            h: cote_pause,
        };
        let touche_droite = droite.agrandie(marge / 2.0);
        // Zone de saut nettement plus large que le dessin, mais sans empiéter
        // sur la flèche droite (écrans étroits).
        let mut touche_saut = saut.agrandie(taille * 0.3);
        let bord = touche_droite.x + touche_droite.l;
        if touche_saut.x < bord {
            touche_saut.l -= bord - touche_saut.x;
            touche_saut.x = bord;
        }
        Self {
            // Flèches : les zones sensibles se rejoignent au milieu de l'espace
            // entre les deux boutons, sans se chevaucher.
            touche_gauche: gauche.agrandie(marge / 2.0),
            touche_droite,
            touche_saut,
            touche_pause: pause.agrandie(marge),
            gauche,
            droite,
            saut,
            pause,
        }
    }
}

/// Un doigt posé sur l'écran.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    /// Vrai à l'image où le doigt se pose (sert à la pause : un seul appui).
    pub nouveau: bool,
}

/// Traduit les doigts posés en commandes. Fonction pure.
pub fn commandes_pour(d: &Disposition, points: &[Point]) -> Commandes {
    let dans = |z: &Zone| points.iter().any(|p| z.contient(p.x, p.y));
    Commandes {
        gauche: dans(&d.touche_gauche),
        droite: dans(&d.touche_droite),
        saut: dans(&d.touche_saut),
        pause: points
            .iter()
            .any(|p| p.nouveau && d.touche_pause.contient(p.x, p.y)),
    }
}

/// Ce que le dessin a besoin de savoir : écran tactile détecté ? boutons enfoncés ?
#[derive(Debug, Clone, Copy, Default)]
struct Etat {
    /// Un toucher a-t-il déjà été reçu ? (sinon : boutons cachés)
    actif: bool,
    /// Dernières commandes lues, pour dessiner les boutons enfoncés.
    dernieres: Commandes,
}

/// La source d'entrée tactile. Elle se clone (`Rc`) : un exemplaire part dans
/// `Sources` (qui lit les touchers), l'autre reste à la boucle principale (qui
/// dessine les boutons). Les deux partagent le même `Etat`.
#[derive(Clone, Default)]
pub struct Tactile {
    etat: Rc<Cell<Etat>>,
}

impl Tactile {
    pub fn new() -> Self {
        Self::default()
    }

    /// Un toucher a-t-il déjà été reçu ? (l'appareil est alors jugé tactile)
    pub fn est_actif(&self) -> bool {
        self.etat.get().actif
    }

    /// Dessine les boutons par-dessus l'écran (à appeler après le rendu du jeu,
    /// en caméra par défaut). Sans effet tant qu'aucun toucher n'a eu lieu.
    pub fn dessiner(&self) {
        let Etat {
            actif,
            dernieres: c,
        } = self.etat.get();
        if !actif {
            return;
        }
        let d = Disposition::calculer(screen_width(), screen_height());
        dessiner_bouton(&d.gauche, &FLECHE_GAUCHE, c.gauche, PLUIE);
        dessiner_bouton(&d.droite, &FLECHE_DROITE, c.droite, PLUIE);
        dessiner_bouton(&d.saut, &FLECHE_HAUT, c.saut, OR);
        dessiner_bouton(&d.pause, &PAUSE, false, BLANC_CASSE);
    }
}

impl SourceEntrees for Tactile {
    fn lire(&mut self) -> Commandes {
        let dpi = screen_dpi_scale();
        // Les positions des touchers sont en pixels PHYSIQUES : on revient aux
        // pixels logiques de l'écran, comme `screen_width()`.
        let points: Vec<Point> = touches()
            .iter()
            .filter(|t| !matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled))
            .map(|t| Point {
                x: t.position.x / dpi,
                y: t.position.y / dpi,
                nouveau: t.phase == TouchPhase::Started,
            })
            .collect();
        let actif = self.etat.get().actif || !points.is_empty();
        let d = Disposition::calculer(screen_width(), screen_height());
        let commandes = commandes_pour(&d, &points);
        self.etat.set(Etat {
            actif,
            dernieres: commandes,
        });
        commandes
    }
}

// --- Dessin pixel art --------------------------------------------------------

type Bitmap = [&'static str; 7];

const FLECHE_GAUCHE: Bitmap = [
    "...#...", "..##...", ".###...", "#######", ".###...", "..##...", "...#...",
];
const FLECHE_DROITE: Bitmap = [
    "...#...", "...##..", "...###.", "#######", "...###.", "...##..", "...#...",
];
const FLECHE_HAUT: Bitmap = [
    "...#...", "..###..", ".#####.", "#######", "..###..", "..###..", "..###..",
];
const PAUSE: Bitmap = [
    ".##.##.", ".##.##.", ".##.##.", ".##.##.", ".##.##.", ".##.##.", ".##.##.",
];

/// Couleur de palette avec une transparence (les boutons laissent voir le jeu).
fn avec_alpha(c: pepin_core::palette::Couleur, a: f32) -> Color {
    let mut k = couleur_mq(c);
    k.a = a;
    k
}

/// Un bouton : fond arrondi « à l'ancienne » (coins coupés d'un bloc), contour
/// clair, symbole en blocs. Enfoncé : fond plus lumineux.
fn dessiner_bouton(
    z: &Zone,
    symbole: &Bitmap,
    enfonce: bool,
    accent: pepin_core::palette::Couleur,
) {
    // Un « pixel » du dessin fait `u` pixels d'écran (toujours entier : net).
    let u = (z.l / 24.0).floor().max(1.0);
    let fond = if enfonce {
        avec_alpha(accent, 0.55)
    } else {
        avec_alpha(ARDOISE_NUIT, 0.55)
    };
    let contour = avec_alpha(accent, 0.9);
    let (x, y, l, h) = (z.x.round(), z.y.round(), z.l.round(), z.h.round());
    // Contour : quatre barres + un bloc dans chaque coin (aspect arrondi).
    // Fond : trois bandes. Aucune pièce ne recouvre une autre (pas de double
    // transparence).
    let contour_pieces = [
        (u, 0.0, l - 2.0 * u, u),
        (u, h - u, l - 2.0 * u, u),
        (0.0, u, u, h - 2.0 * u),
        (l - u, u, u, h - 2.0 * u),
        (u, u, u, u),
        (l - 2.0 * u, u, u, u),
        (u, h - 2.0 * u, u, u),
        (l - 2.0 * u, h - 2.0 * u, u, u),
    ];
    let fond_pieces = [
        (2.0 * u, u, l - 4.0 * u, u),
        (u, 2.0 * u, l - 2.0 * u, h - 4.0 * u),
        (2.0 * u, h - 2.0 * u, l - 4.0 * u, u),
    ];
    for (dx, dy, w, hh) in contour_pieces {
        draw_rectangle(x + dx, y + dy, w, hh, contour);
    }
    for (dx, dy, w, hh) in fond_pieces {
        draw_rectangle(x + dx, y + dy, w, hh, fond);
    }
    // Symbole centré : 7x7 blocs de `s` pixels d'écran.
    let s = ((l.min(h) * 0.5) / 7.0).floor().max(1.0);
    let ox = x + ((l - 7.0 * s) / 2.0).floor();
    let oy = y + ((h - 7.0 * s) / 2.0).floor();
    let encre = avec_alpha(BLANC_CASSE, 0.95);
    for (j, ligne) in symbole.iter().enumerate() {
        for (i, c) in ligne.chars().enumerate() {
            if c == '#' {
                draw_rectangle(ox + i as f32 * s, oy + j as f32 * s, s, s, encre);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn centre(z: &Zone) -> (f32, f32) {
        (z.x + z.l / 2.0, z.y + z.h / 2.0)
    }

    fn doigt(z: &Zone, nouveau: bool) -> Point {
        let (x, y) = centre(z);
        Point { x, y, nouveau }
    }

    /// Tailles d'écran de téléphones (paysage, portrait) et de tablette.
    const ECRANS: [(f32, f32); 5] = [
        (844.0, 390.0),
        (390.0, 844.0),
        (667.0, 375.0),
        (1024.0, 768.0),
        (320.0, 480.0),
    ];

    #[test]
    fn les_boutons_restent_dans_l_ecran_et_sont_assez_gros() {
        for (l, h) in ECRANS {
            let d = Disposition::calculer(l, h);
            for z in [d.gauche, d.droite, d.saut, d.pause] {
                assert!(z.x >= 0.0 && z.y >= 0.0, "{l}x{h} : {z:?}");
                assert!(z.x + z.l <= l && z.y + z.h <= h, "{l}x{h} : {z:?}");
            }
            // Au moins ~ 9 mm : 48 px est le minimum usuel pour un doigt.
            for z in [d.gauche, d.droite, d.saut] {
                assert!(z.l >= 48.0 && z.h >= 48.0, "{l}x{h} : {z:?}");
            }
        }
    }

    #[test]
    fn les_boutons_ne_se_chevauchent_pas() {
        for (l, h) in ECRANS {
            let d = Disposition::calculer(l, h);
            let (g, dr, s, p) = (
                d.touche_gauche,
                d.touche_droite,
                d.touche_saut,
                d.touche_pause,
            );
            // Gauche et droite se touchent au plus (jamais de recouvrement).
            assert!(g.x + g.l <= dr.x + 1e-3, "{l}x{h}");
            // Le centre de chaque bouton ne tombe que dans sa propre zone.
            for (z, nom) in [
                (d.gauche, "g"),
                (d.droite, "d"),
                (d.saut, "s"),
                (d.pause, "p"),
            ] {
                let (cx, cy) = centre(&z);
                let n = [g, dr, s, p]
                    .iter()
                    .filter(|zz| zz.contient(cx, cy))
                    .count();
                assert_eq!(n, 1, "{l}x{h} bouton {nom}");
            }
        }
    }

    #[test]
    fn un_doigt_sur_chaque_bouton_donne_la_bonne_commande() {
        let d = Disposition::calculer(844.0, 390.0);
        let c = commandes_pour(&d, &[doigt(&d.gauche, false)]);
        assert!(c.gauche && !c.droite && !c.saut && !c.pause);
        let c = commandes_pour(&d, &[doigt(&d.droite, false)]);
        assert!(!c.gauche && c.droite && !c.saut);
        let c = commandes_pour(&d, &[doigt(&d.saut, false)]);
        assert!(c.saut && !c.gauche && !c.droite);
    }

    #[test]
    fn plusieurs_doigts_se_cumulent() {
        let d = Disposition::calculer(844.0, 390.0);
        let c = commandes_pour(&d, &[doigt(&d.droite, false), doigt(&d.saut, false)]);
        assert!(c.droite && c.saut && !c.gauche);
    }

    #[test]
    fn la_pause_ne_part_qu_a_l_appui_pas_en_maintenant() {
        let d = Disposition::calculer(844.0, 390.0);
        assert!(commandes_pour(&d, &[doigt(&d.pause, true)]).pause);
        assert!(!commandes_pour(&d, &[doigt(&d.pause, false)]).pause);
    }

    #[test]
    fn un_doigt_au_milieu_de_l_ecran_ne_fait_rien() {
        let d = Disposition::calculer(844.0, 390.0);
        let c = commandes_pour(
            &d,
            &[Point {
                x: 422.0,
                y: 100.0,
                nouveau: true,
            }],
        );
        assert_eq!(c, Commandes::default());
    }

    #[test]
    fn la_zone_sensible_depasse_le_dessin_du_saut() {
        let d = Disposition::calculer(844.0, 390.0);
        // Un peu à gauche du dessin du saut : touche quand même.
        let p = Point {
            x: d.saut.x - 5.0,
            y: d.saut.y + d.saut.h / 2.0,
            nouveau: false,
        };
        assert!(commandes_pour(&d, &[p]).saut);
    }
}
