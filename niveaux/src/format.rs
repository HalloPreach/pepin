//! Le FORMAT d'un fichier de niveau : les structures lues par serde dans un
//! fichier RON, et l'écriture inverse (`ecrire`), utilisée par l'exporteur.
//!
//! Ces structures sont celles du FICHIER, pas celles du jeu : elles possèdent
//! leurs textes (`String`, `Vec`) et refusent tout champ inconnu
//! (`deny_unknown_fields` : une faute de frappe dans un nom de champ est une
//! erreur, pas un champ ignoré en silence). `vers_infos` les convertit en
//! `InfosNiveau` du crate `core`, qui ne dépend ni de serde ni de ron.
//!
//! Notions Rust : `#[derive(Deserialize)]` (le code de lecture est écrit par une
//! macro), énumération « miroir » convertie par un `match` EXHAUSTIF dans les deux
//! sens (ajouter un thème dans `core` sans l'ajouter ici empêche de compiler),
//! `Box::leak` pour obtenir une référence `&'static`.

use pepin_core::level::ReglageAiguille;
use pepin_core::levels::GenreNiveau;
use pepin_core::zones::{InfosNiveau, Theme};
use serde::Deserialize;

/// Un niveau tel qu'il est écrit dans son fichier `.ron`. Mêmes champs que
/// `InfosNiveau` (leur sens y est documenté), dans l'ordre où l'exporteur les écrit.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FichierNiveau {
    pub monde: u8,
    pub numero: u8,
    pub cle: String,
    pub nom: String,
    pub theme: ThemeFichier,
    pub genre: GenreFichier,
    pub sans_planer: bool,
    pub duree_cible_s: u16,
    pub intro: String,
    pub mecanique: String,
    /// Facultatif (vide par défaut) : une ligne par pivot `h` de la grille.
    #[serde(default)]
    pub aiguilles: Vec<AiguilleFichier>,
    /// Facultatif (vide par défaut) : `(colonne, rangée, hauteur_px)` d'un poisson `i`.
    #[serde(default)]
    pub poissons_hauts: Vec<(usize, usize, f32)>,
    /// La grille, en texte brut (légende : `assets/levels/LISEZMOI.md`).
    pub ascii: String,
}

/// Réglage d'une aiguille d'horloge (voir `ReglageAiguille`).
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiguilleFichier {
    pub rayon: f32,
    pub vitesse: f32,
    pub phase: f32,
}

/// Normal, bonus ou épilogue : le miroir de `GenreNiveau`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum GenreFichier {
    Normal,
    Bonus,
    Epilogue,
}

/// Le miroir de `Theme` : une variante NOMMÉE par thème (`theme: Port`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ThemeFichier {
    Toits,
    Gouttieres,
    Marche,
    Orage,
    Clocher,
    Cyclone,
    Altitude,
    Mer,
    Soleil,
    ArcEnCiel,
    Descente,
    Poursuite,
    Caniveau,
    Egouts,
    Cascade,
    Grotte,
    Riviere,
    Port,
    Maree,
}

impl From<GenreFichier> for GenreNiveau {
    fn from(g: GenreFichier) -> Self {
        match g {
            GenreFichier::Normal => GenreNiveau::Normal,
            GenreFichier::Bonus => GenreNiveau::Bonus,
            GenreFichier::Epilogue => GenreNiveau::Epilogue,
        }
    }
}

impl From<GenreNiveau> for GenreFichier {
    fn from(g: GenreNiveau) -> Self {
        match g {
            GenreNiveau::Normal => GenreFichier::Normal,
            GenreNiveau::Bonus => GenreFichier::Bonus,
            GenreNiveau::Epilogue => GenreFichier::Epilogue,
        }
    }
}

impl From<ThemeFichier> for Theme {
    fn from(t: ThemeFichier) -> Self {
        use ThemeFichier as F;
        match t {
            F::Toits => Theme::Toits,
            F::Gouttieres => Theme::Gouttieres,
            F::Marche => Theme::Marche,
            F::Orage => Theme::Orage,
            F::Clocher => Theme::Clocher,
            F::Cyclone => Theme::Cyclone,
            F::Altitude => Theme::Altitude,
            F::Mer => Theme::Mer,
            F::Soleil => Theme::Soleil,
            F::ArcEnCiel => Theme::ArcEnCiel,
            F::Descente => Theme::Descente,
            F::Poursuite => Theme::Poursuite,
            F::Caniveau => Theme::Caniveau,
            F::Egouts => Theme::Egouts,
            F::Cascade => Theme::Cascade,
            F::Grotte => Theme::Grotte,
            F::Riviere => Theme::Riviere,
            F::Port => Theme::Port,
            F::Maree => Theme::Maree,
        }
    }
}

impl From<Theme> for ThemeFichier {
    fn from(t: Theme) -> Self {
        use ThemeFichier as F;
        match t {
            Theme::Toits => F::Toits,
            Theme::Gouttieres => F::Gouttieres,
            Theme::Marche => F::Marche,
            Theme::Orage => F::Orage,
            Theme::Clocher => F::Clocher,
            Theme::Cyclone => F::Cyclone,
            Theme::Altitude => F::Altitude,
            Theme::Mer => F::Mer,
            Theme::Soleil => F::Soleil,
            Theme::ArcEnCiel => F::ArcEnCiel,
            Theme::Descente => F::Descente,
            Theme::Poursuite => F::Poursuite,
            Theme::Caniveau => F::Caniveau,
            Theme::Egouts => F::Egouts,
            Theme::Cascade => F::Cascade,
            Theme::Grotte => F::Grotte,
            Theme::Riviere => F::Riviere,
            Theme::Port => F::Port,
            Theme::Maree => F::Maree,
        }
    }
}

/// Donne à un texte la durée de vie `'static` en le « laissant fuir ».
///
/// Pourquoi ce choix : `InfosNiveau` (crate `core`) garde ses textes en
/// `&'static str`, parce que les niveaux du jeu vivent TOUTE la partie. Un niveau
/// lu dans un fichier possède ses textes (`String`) ; `Box::leak` rend la mémoire
/// au programme pour toujours et nous donne en échange une référence `'static`.
/// C'est sans danger et sans coût ici : la table est chargée UNE seule fois au
/// démarrage (`crate::niveaux`, gardée par un `OnceLock`), quelques dizaines de
/// kilo-octets, rendus au système à la fermeture du jeu. L'autre voie (changer
/// `InfosNiveau` en `String` partout, ou lui ajouter une durée de vie `'a`)
/// toucherait tout le jeu pour aucun gain.
fn laisser_fuir(texte: String) -> &'static str {
    Box::leak(texte.into_boxed_str())
}

impl FichierNiveau {
    /// La description `core` de ce niveau (les textes deviennent `'static`, voir
    /// `laisser_fuir`). Ne vérifie RIEN : c'est le rôle de `crate::charger`.
    pub fn vers_infos(self) -> InfosNiveau {
        let aiguilles: Vec<ReglageAiguille> = self
            .aiguilles
            .iter()
            .map(|a| ReglageAiguille {
                rayon: a.rayon,
                vitesse: a.vitesse,
                phase: a.phase,
            })
            .collect();
        InfosNiveau {
            monde: self.monde,
            numero: self.numero,
            cle: laisser_fuir(self.cle),
            sans_planer: self.sans_planer,
            nom: laisser_fuir(self.nom),
            theme: self.theme.into(),
            genre: self.genre.into(),
            intro: laisser_fuir(self.intro),
            mecanique: laisser_fuir(self.mecanique),
            duree_cible_s: self.duree_cible_s,
            ascii: laisser_fuir(self.ascii),
            aiguilles: Box::leak(aiguilles.into_boxed_slice()),
            poissons_hauts: Box::leak(self.poissons_hauts.into_boxed_slice()),
        }
    }
}

/// Écrit `infos` au format RON, toujours de la MÊME façon (mêmes octets à
/// chaque fois) : l'exporteur peut être relancé sans rien changer, et le test
/// `ecrire_redonne_chaque_fichier` le vérifie sur chaque fichier du dépôt.
///
/// La grille est écrite en chaîne BRUTE (`r#"..."#`) : elle reste lisible et se
/// modifie à la main, case par case.
pub fn ecrire(infos: &InfosNiveau) -> String {
    let mut s = String::new();
    s.push_str("// Niveau de PÉPIN. Format : voir assets/levels/LISEZMOI.md.\n(\n");
    s.push_str(&format!("    monde: {},\n", infos.monde));
    s.push_str(&format!("    numero: {},\n", infos.numero));
    s.push_str(&format!("    cle: {},\n", chaine(infos.cle)));
    s.push_str(&format!("    nom: {},\n", chaine(infos.nom)));
    s.push_str(&format!(
        "    theme: {:?},\n",
        ThemeFichier::from(infos.theme)
    ));
    s.push_str(&format!(
        "    genre: {:?},\n",
        GenreFichier::from(infos.genre)
    ));
    s.push_str(&format!("    sans_planer: {},\n", infos.sans_planer));
    s.push_str(&format!("    duree_cible_s: {},\n", infos.duree_cible_s));
    s.push_str(&format!("    intro: {},\n", chaine(infos.intro)));
    s.push_str(&format!("    mecanique: {},\n", chaine(infos.mecanique)));
    if infos.aiguilles.is_empty() {
        s.push_str("    aiguilles: [],\n");
    } else {
        s.push_str("    aiguilles: [\n");
        for a in infos.aiguilles {
            // `{:?}` d'un `f32` : le plus court texte qui redonne EXACTEMENT ce
            // nombre à la lecture (et toujours avec un point : `3.0`).
            s.push_str(&format!(
                "        (rayon: {:?}, vitesse: {:?}, phase: {:?}),\n",
                a.rayon, a.vitesse, a.phase
            ));
        }
        s.push_str("    ],\n");
    }
    if infos.poissons_hauts.is_empty() {
        s.push_str("    poissons_hauts: [],\n");
    } else {
        s.push_str("    poissons_hauts: [\n");
        for (colonne, rangee, hauteur) in infos.poissons_hauts {
            s.push_str(&format!("        ({colonne}, {rangee}, {hauteur:?}),\n"));
        }
        s.push_str("    ],\n");
    }
    // Autant de `#` qu'il faut pour que la grille ne puisse pas fermer la chaîne.
    let mut dieses = String::from("#");
    while infos.ascii.contains(&format!("\"{dieses}")) {
        dieses.push('#');
    }
    s.push_str(&format!(
        "    ascii: r{dieses}\"{}\"{dieses},\n",
        infos.ascii
    ));
    s.push_str(")\n");
    s
}

/// Une chaîne RON entre guillemets : `\` et `"` échappés, ainsi que les fins de
/// ligne ; les lettres accentuées restent telles quelles (le fichier est en UTF-8).
fn chaine(texte: &str) -> String {
    let mut s = String::from("\"");
    for c in texte.chars() {
        match c {
            '\\' => s.push_str("\\\\"),
            '"' => s.push_str("\\\""),
            '\n' => s.push_str("\\n"),
            '\t' => s.push_str("\\t"),
            _ => s.push(c),
        }
    }
    s.push('"');
    s
}
