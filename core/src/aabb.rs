//! Module `aabb` : boîtes alignées sur les axes, grille de tuiles et collisions.
//!
//! Notions Rust illustrées ici (à reprendre dans le README) :
//! - `struct` + `impl` : regrouper des données et leurs méthodes ;
//! - `#[derive(...)]` : faire générer des traits courants (Copy, Clone, Debug...) ;
//! - `enum` : un type qui vaut « l'une OU l'autre » de plusieurs variantes ;
//! - `trait` : un contrat (`Grille`) que plusieurs types peuvent remplir ;
//! - emprunts : `&T` (lecture seule) et `&mut T` (modification exclusive) ;
//! - `&dyn Trait` : un emprunt vers « n'importe quel type qui implémente le trait ».

/// Taille d'une tuile en pixels logiques.
pub const TAILLE_TUILE: f32 = 16.0;

/// Distance maximale (en pixels) parcourue d'un seul bloc lors d'un déplacement.
/// Au-delà, `deplacer` découpe le mouvement en petits morceaux pour qu'une boîte
/// rapide ne traverse jamais un mur mince (« effet tunnel »).
const PAS_MAX_DEPLACEMENT: f32 = 8.0;

/// Boîte rectangulaire alignée sur les axes (Axis-Aligned Bounding Box).
///
/// `Copy` : la structure est petite (4 flottants), on peut donc la dupliquer
/// implicitement au lieu de la « déplacer » (ownership). Sans `Copy`, écrire
/// `let b = a;` rendrait `a` inutilisable ensuite.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    /// Bord gauche.
    pub x: f32,
    /// Bord haut (l'axe Y pointe vers le BAS, comme à l'écran).
    pub y: f32,
    /// Largeur.
    pub w: f32,
    /// Hauteur.
    pub h: f32,
}

impl Aabb {
    /// Construit une boîte. `const fn` : utilisable dans une constante.
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    /// Bord droit.
    pub fn droite(&self) -> f32 {
        self.x + self.w
    }

    /// Bord bas.
    pub fn bas(&self) -> f32 {
        self.y + self.h
    }

    /// Deux boîtes se chevauchent-elles ? Les bords qui se touchent simplement
    /// ne comptent PAS comme un chevauchement (inégalités strictes) : c'est ce
    /// qui permet de « poser » une boîte sur le sol sans qu'elle y soit coincée.
    pub fn chevauche(&self, autre: &Aabb) -> bool {
        self.x < autre.droite()
            && autre.x < self.droite()
            && self.y < autre.bas()
            && autre.y < self.bas()
    }

    /// Centre de la boîte sous forme de tuple `(x, y)`.
    pub fn centre(&self) -> (f32, f32) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    /// Renvoie une COPIE décalée ; `self` (emprunté en lecture) n'est pas modifié.
    pub fn decale(&self, dx: f32, dy: f32) -> Aabb {
        Aabb::new(self.x + dx, self.y + dy, self.w, self.h)
    }
}

/// Nature d'une tuile. Un `enum` est idéal : une tuile est vide OU solide,
/// jamais autre chose, et le compilateur nous force à traiter chaque cas (`match`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tuile {
    Vide,
    Solide,
}

/// Contrat : « savoir dire si une tuile est solide ».
///
/// Le moteur de collision ne connaît que ce trait, pas la structure concrète du
/// niveau : on pourra donc le tester avec une petite grille de test, puis
/// l'utiliser telle quelle avec les vrais niveaux.
pub trait Grille {
    /// Tuile aux coordonnées de grille `(tx, ty)` (entiers signés : la case peut
    /// être hors de la carte).
    fn tuile(&self, tx: i32, ty: i32) -> Tuile;

    /// Méthode « par défaut » : écrite une fois dans le trait, héritée gratuitement.
    fn est_solide(&self, tx: i32, ty: i32) -> bool {
        self.tuile(tx, ty) == Tuile::Solide
    }
}

/// Grille rectangulaire de tuiles stockée dans un `Vec` (tableau dynamique).
/// La structure POSSÈDE ses données (ownership) : elles sont libérées quand la
/// grille sort de portée.
#[derive(Debug, Clone)]
pub struct GrilleTuiles {
    largeur: usize,
    hauteur: usize,
    tuiles: Vec<Tuile>,
}

impl GrilleTuiles {
    /// Grille entièrement vide.
    pub fn vide(largeur: usize, hauteur: usize) -> Self {
        Self {
            largeur,
            hauteur,
            tuiles: vec![Tuile::Vide; largeur * hauteur],
        }
    }

    /// Construit une grille depuis un dessin ASCII : `#` = solide, tout le reste = vide.
    /// `&[&str]` est une « tranche » (slice) d'emprunts de texte : on lit sans posséder.
    pub fn depuis_texte(lignes: &[&str]) -> Self {
        let largeur = lignes.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let mut grille = Self::vide(largeur, lignes.len());
        for (ty, ligne) in lignes.iter().enumerate() {
            for (tx, c) in ligne.chars().enumerate() {
                if c == '#' {
                    grille.definir(tx, ty, Tuile::Solide);
                }
            }
        }
        grille
    }

    pub fn largeur(&self) -> usize {
        self.largeur
    }

    pub fn hauteur(&self) -> usize {
        self.hauteur
    }

    /// Modifie une tuile (`&mut self` : emprunt exclusif en écriture).
    /// Ne fait rien si la case est hors de la carte.
    pub fn definir(&mut self, tx: usize, ty: usize, tuile: Tuile) {
        if tx < self.largeur && ty < self.hauteur {
            self.tuiles[ty * self.largeur + tx] = tuile;
        }
    }
}

impl Grille for GrilleTuiles {
    /// Règle hors-carte : les côtés gauche/droite sont des MURS (on ne sort pas
    /// du monde), le haut est ouvert (on peut sauter au-dessus), le bas est ouvert
    /// aussi (c'est un vide : le jeu décidera de la mort du joueur).
    fn tuile(&self, tx: i32, ty: i32) -> Tuile {
        if tx < 0 || tx >= self.largeur as i32 {
            return Tuile::Solide;
        }
        if ty < 0 || ty >= self.hauteur as i32 {
            return Tuile::Vide;
        }
        self.tuiles[ty as usize * self.largeur + tx as usize]
    }
}

/// Faces touchées pendant un déplacement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Contacts {
    pub sol: bool,
    pub plafond: bool,
    pub mur_gauche: bool,
    pub mur_droite: bool,
}

impl Contacts {
    /// Un mur (gauche ou droite) a-t-il été touché ?
    pub fn mur(&self) -> bool {
        self.mur_gauche || self.mur_droite
    }

    fn fusionner(&mut self, autre: Contacts) {
        self.sol |= autre.sol;
        self.plafond |= autre.plafond;
        self.mur_gauche |= autre.mur_gauche;
        self.mur_droite |= autre.mur_droite;
    }
}

/// Appelle `action` pour chaque obstacle (tuile solide ou boîte solide
/// supplémentaire) qui chevauche `boite`.
///
/// `impl FnMut(&Aabb)` : on reçoit une *fonction anonyme* (closure) qui a le droit
/// de modifier ce qu'elle a capturé. Le paramètre `boite` est copié, ce qui
/// permet à `action` de modifier la vraie boîte entre deux obstacles.
fn pour_chaque_obstacle(
    boite: &Aabb,
    grille: &dyn Grille,
    solides: &[Aabb],
    mut action: impl FnMut(&Aabb),
) {
    let tx0 = (boite.x / TAILLE_TUILE).floor() as i32;
    let tx1 = (boite.droite() / TAILLE_TUILE).ceil() as i32 - 1;
    let ty0 = (boite.y / TAILLE_TUILE).floor() as i32;
    let ty1 = (boite.bas() / TAILLE_TUILE).ceil() as i32 - 1;
    for ty in ty0..=ty1 {
        for tx in tx0..=tx1 {
            if grille.est_solide(tx, ty) {
                let tuile = Aabb::new(
                    tx as f32 * TAILLE_TUILE,
                    ty as f32 * TAILLE_TUILE,
                    TAILLE_TUILE,
                    TAILLE_TUILE,
                );
                action(&tuile);
            }
        }
    }
    for s in solides {
        if boite.chevauche(s) {
            action(s);
        }
    }
}

/// `boite` chevauche-t-elle un obstacle (tuile solide de `grille`, ou une des
/// boîtes `solides`) ? Sert aux entités qui vérifient qu'elles ont la place de
/// bouger AVANT de bouger (la montgolfière qui monte, par exemple).
pub fn touche_un_obstacle(boite: &Aabb, grille: &dyn Grille, solides: &[Aabb]) -> bool {
    let mut touche = false;
    pour_chaque_obstacle(boite, grille, solides, |o| touche |= boite.chevauche(o));
    touche
}

/// Déplace `boite` de `(dx, dy)` en résolvant les collisions AXE PAR AXE :
/// d'abord X (on se repousse hors des murs), puis Y (sol / plafond).
/// C'est ce qui permet de glisser le long d'un mur sans rester collé.
///
/// `boite: &mut Aabb` est un emprunt EXCLUSIF en écriture : pendant l'appel,
/// personne d'autre ne peut lire ni modifier cette boîte.
pub fn deplacer(
    boite: &mut Aabb,
    dx: f32,
    dy: f32,
    grille: &dyn Grille,
    solides: &[Aabb],
) -> Contacts {
    let mut contacts = Contacts::default();
    let plus_grand = dx.abs().max(dy.abs());
    let morceaux = (plus_grand / PAS_MAX_DEPLACEMENT).ceil().max(1.0) as u32;
    let (sdx, sdy) = (dx / morceaux as f32, dy / morceaux as f32);
    for _ in 0..morceaux {
        contacts.fusionner(deplacer_un_morceau(boite, sdx, sdy, grille, solides));
    }
    contacts
}

fn deplacer_un_morceau(
    boite: &mut Aabb,
    dx: f32,
    dy: f32,
    grille: &dyn Grille,
    solides: &[Aabb],
) -> Contacts {
    let mut contacts = Contacts::default();

    // --- Axe X ---
    if dx != 0.0 {
        boite.x += dx;
        let copie = *boite;
        pour_chaque_obstacle(&copie, grille, solides, |o| {
            if boite.chevauche(o) {
                if dx > 0.0 {
                    boite.x = o.x - boite.w;
                    contacts.mur_droite = true;
                } else {
                    boite.x = o.droite();
                    contacts.mur_gauche = true;
                }
            }
        });
    }

    // --- Axe Y ---
    if dy != 0.0 {
        boite.y += dy;
        let copie = *boite;
        pour_chaque_obstacle(&copie, grille, solides, |o| {
            if boite.chevauche(o) {
                if dy > 0.0 {
                    boite.y = o.y - boite.h;
                    contacts.sol = true;
                } else {
                    boite.y = o.bas();
                    contacts.plafond = true;
                }
            }
        });
    }
    contacts
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Salle de test : murs aux deux bords, sol sur la dernière ligne, plafond en haut.
    fn salle() -> GrilleTuiles {
        GrilleTuiles::depuis_texte(&[
            "##########",
            "#........#",
            "#........#",
            "#........#",
            "##########",
        ])
    }

    #[test]
    fn bords_qui_se_touchent_ne_chevauchent_pas() {
        let a = Aabb::new(0.0, 0.0, 10.0, 10.0);
        let b = Aabb::new(10.0, 0.0, 10.0, 10.0);
        assert!(!a.chevauche(&b));
        assert!(a.chevauche(&b.decale(-1.0, 0.0)));
    }

    #[test]
    fn grille_depuis_texte_et_hors_carte() {
        let g = salle();
        assert_eq!((g.largeur(), g.hauteur()), (10, 5));
        assert!(g.est_solide(0, 2));
        assert!(!g.est_solide(1, 1));
        assert!(g.est_solide(-3, 1), "hors carte à gauche = mur");
        assert!(g.est_solide(40, 1), "hors carte à droite = mur");
        assert!(!g.est_solide(3, -2), "au-dessus de la carte = ouvert");
        assert!(!g.est_solide(3, 99), "sous la carte = vide");
    }

    #[test]
    fn atterrissage_sur_le_sol() {
        let g = salle();
        let mut b = Aabb::new(48.0, 40.0, 10.0, 14.0);
        let c = deplacer(&mut b, 0.0, 30.0, &g, &[]);
        assert!(c.sol && !c.plafond && !c.mur());
        assert_eq!(b.bas(), 4.0 * TAILLE_TUILE, "posée exactement sur la tuile");
    }

    #[test]
    fn collision_plafond() {
        let g = salle();
        let mut b = Aabb::new(48.0, 20.0, 10.0, 14.0);
        let c = deplacer(&mut b, 0.0, -30.0, &g, &[]);
        assert!(c.plafond && !c.sol);
        assert_eq!(b.y, TAILLE_TUILE, "collée sous la tuile du plafond");
    }

    #[test]
    fn collision_murs_gauche_et_droit() {
        let g = salle();
        let mut b = Aabb::new(48.0, 40.0, 10.0, 14.0);
        let c = deplacer(&mut b, 200.0, 0.0, &g, &[]);
        assert!(c.mur_droite && !c.mur_gauche);
        assert_eq!(b.droite(), 9.0 * TAILLE_TUILE);
        let c = deplacer(&mut b, -500.0, 0.0, &g, &[]);
        assert!(c.mur_gauche);
        assert_eq!(b.x, TAILLE_TUILE);
    }

    #[test]
    fn glisse_le_long_du_mur() {
        let g = salle();
        let mut b = Aabb::new(16.0, 20.0, 10.0, 14.0);
        // On pousse contre le mur gauche tout en descendant : Y reste libre.
        let c = deplacer(&mut b, -3.0, 5.0, &g, &[]);
        assert!(c.mur_gauche);
        assert_eq!(b.x, 16.0);
        assert_eq!(b.y, 25.0);
    }

    #[test]
    fn pas_d_effet_tunnel_sur_un_mur_mince() {
        let g = GrilleTuiles::depuis_texte(&["..........", "....#.....", ".........."]);
        let mut b = Aabb::new(0.0, 16.0, 10.0, 10.0);
        let c = deplacer(&mut b, 120.0, 0.0, &g, &[]);
        assert!(c.mur_droite);
        assert_eq!(b.droite(), 4.0 * TAILLE_TUILE);
    }

    #[test]
    fn solides_supplementaires_comme_plateforme() {
        let g = GrilleTuiles::vide(10, 5);
        let plateforme = [Aabb::new(32.0, 48.0, 48.0, 8.0)];
        let mut b = Aabb::new(40.0, 20.0, 10.0, 14.0);
        let c = deplacer(&mut b, 0.0, 40.0, &g, &plateforme);
        assert!(c.sol);
        assert_eq!(b.bas(), 48.0);
    }
}
