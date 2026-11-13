//! Module `zones` : les métadonnées des niveaux de PÉPIN.
//!
//! Chaque niveau est une « zone » du monde : un thème de décor (le ciel
//! s'éclaircit d'un niveau à l'autre, de l'orage au soleil), un nom, un genre
//! (normal, bonus ou épilogue), un court texte d'introduction et une idée de
//! mécanique. Le dessin ASCII lui-même vit dans `levels/` (un petit fichier par
//! niveau) ; la table ordonnée des niveaux, `NIVEAUX`, aussi (`levels.rs`).
//!
//! Notions Rust illustrées ici (à reprendre dans le README) :
//! - `const` et `&'static str` : des données connues à la compilation, qui
//!   vivent pendant tout le programme (donc pas d'ownership à gérer) ;
//! - `enum` sans données pour le thème ; tableau de taille fixe `[T; N]` ;
//! - `pub use` : `NIVEAUX` est défini dans `levels` et ré-exporté ici ;
//! - `Result` : charger un niveau peut échouer, l'appelant décide quoi faire.

use crate::level::{ErreurNiveau, Niveau, ReglageAiguille};
pub use crate::levels::{GenreNiveau, NB_NIVEAUX, NIVEAUX};

/// Ambiance visuelle d'un niveau. Le rendu choisit ciel, parallaxe et pluie
/// selon cette valeur ; `luminosite` donne l'avancement de l'orage vers le soleil.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    /// Nuit d'orage sur les toits.
    Toits,
    /// Averse, ciel d'ardoise.
    Gouttieres,
    /// Marché couvert : nuit bleue, vitrines et lanternes orange.
    Marche,
    /// Orage au plus fort : éclairs, ciel violet.
    Orage,
    /// Le clocher : nuit violette au-dessus de la ville, que l'on voit s'enfoncer
    /// en montant (niveau vertical).
    Clocher,
    /// Le ciel se déchire, vent d'altitude.
    Altitude,
    /// Au-dessus des nuages : le soleil doré.
    Soleil,
}

impl Theme {
    /// Tous les thèmes, dans l'ordre de leur déclaration : le rendu y range un jeu
    /// de tuiles par thème (`Theme::indice`). Un nouveau thème s'ajoute ici ET dans
    /// l'`enum` (le test `tous_les_themes_dans_l_ordre` vérifie les deux).
    pub const TOUS: [Theme; 7] = [
        Theme::Toits,
        Theme::Gouttieres,
        Theme::Marche,
        Theme::Orage,
        Theme::Clocher,
        Theme::Altitude,
        Theme::Soleil,
    ];

    /// Place du thème dans `Theme::TOUS` (son rang de déclaration).
    pub fn indice(self) -> usize {
        self as usize
    }

    /// Clarté du ciel de 0 (orage noir) à 4 (soleil) ; l'orage du niveau 4
    /// reste volontairement sombre, c'est le sommet de la tension.
    pub fn luminosite(self) -> u8 {
        match self {
            Theme::Toits => 0,
            Theme::Gouttieres => 1,
            Theme::Marche => 1,
            Theme::Orage => 0,
            Theme::Clocher => 1,
            Theme::Altitude => 3,
            Theme::Soleil => 4,
        }
    }

    /// Pleut-il dans ce thème ?
    pub fn pluie(self) -> bool {
        !matches!(self, Theme::Soleil)
    }
}

/// Description d'un niveau : tout sauf le niveau chargé.
#[derive(Debug, Clone, Copy)]
pub struct InfosNiveau {
    /// Place dans la table `NIVEAUX`, à partir de 1 (vérifié par `verifier_ordre`).
    pub numero: u8,
    /// Titre affiché.
    pub nom: &'static str,
    pub theme: Theme,
    /// Normal, bonus ou épilogue (voir `levels`).
    pub genre: GenreNiveau,
    /// Texte d'introduction (placeholder), une ou deux phrases courtes.
    pub intro: &'static str,
    /// Mécanique introduite (documentation et écran d'aide).
    pub mecanique: &'static str,
    /// Durée visée en secondes (de jeu normal, sans mort).
    pub duree_cible_s: u16,
    /// Dessin ASCII du niveau (voir la légende dans `level`).
    pub ascii: &'static str,
    /// Réglage des aiguilles d'horloge du dessin : une ligne par pivot `h`, dans
    /// l'ordre de lecture (voir `level::ReglageAiguille`). Vide sans aiguille.
    pub aiguilles: &'static [ReglageAiguille],
}

impl InfosNiveau {
    /// Lit le dessin ASCII (et le réglage de ses aiguilles). Ne peut échouer que
    /// si le niveau embarqué est invalide, ce que les tests de ce module interdisent.
    pub fn charger(&self) -> Result<Niveau, ErreurNiveau> {
        Niveau::depuis_ascii_avec_aiguilles(self.ascii, self.aiguilles)
    }
}

/// Nombre total de gouttes d'or du jeu (tous les niveaux : bonus et épilogue compris).
pub const GOUTTES_TOTAL: usize = NB_NIVEAUX * crate::level::GOUTTES_PAR_NIVEAU;

/// Niveau `indice` (0 pour le premier) ou `None` hors de la table.
pub fn niveau(indice: usize) -> Option<&'static InfosNiveau> {
    NIVEAUX.get(indice)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faisabilite::{analyser, Options};
    use crate::levels::{
        essai_auvent, essai_chat, essai_escargot, essai_grele, essai_horloge, essai_montgolfiere,
        essai_nuage, essai_vent, essai_ventilateur, essai_vertical,
    };

    #[test]
    fn les_niveaux_sont_numerotes_dans_l_ordre() {
        for (i, n) in NIVEAUX.iter().enumerate() {
            assert_eq!(n.numero as usize, i + 1);
            assert!(!n.nom.is_empty() && !n.intro.is_empty() && !n.mecanique.is_empty());
        }
        assert_eq!(niveau(0).unwrap().nom, "Les toits");
        assert!(niveau(NB_NIVEAUX).is_none());
    }

    #[test]
    fn tous_les_themes_dans_l_ordre() {
        for (i, t) in Theme::TOUS.into_iter().enumerate() {
            assert_eq!(t.indice(), i, "{t:?}");
        }
        // Chaque thème utilisé par un niveau est dans la liste (le `match` de
        // `luminosite` oblige déjà à traiter chaque variante).
        for n in NIVEAUX {
            assert!(Theme::TOUS.contains(&n.theme), "{}", n.nom);
        }
    }

    #[test]
    fn tous_les_niveaux_se_chargent() {
        for n in NIVEAUX.iter().chain([
            &essai_vertical::INFOS,
            &essai_vent::INFOS,
            &essai_grele::INFOS,
            &essai_auvent::INFOS,
            &essai_ventilateur::INFOS,
            &essai_montgolfiere::INFOS,
            &essai_escargot::INFOS,
            &essai_chat::INFOS,
            &essai_horloge::INFOS,
            &essai_nuage::INFOS,
        ]) {
            let niv = n
                .charger()
                .unwrap_or_else(|e| panic!("niveau {} : {e}", n.numero));
            // Au moins un écran de haut (11 rangées, 176 px) : un niveau plus haut que
            // l'écran (carte 5.2) fait défiler la caméra en hauteur.
            assert!(
                niv.hauteur() >= 11,
                "niveau {} : 11 lignes au moins",
                n.numero
            );
        }
        // Le niveau d'essai vertical fait quatre écrans de haut et n'est pas dans la table.
        let essai = essai_vertical::INFOS.charger().unwrap();
        assert_eq!(essai.hauteur(), 45);
        assert!(NIVEAUX.iter().all(|n| n.nom != essai_vertical::INFOS.nom));
    }

    /// Les niveaux du jeu, puis les niveaux d'essai (hors de la table) : les
    /// contrôles de grille s'appliquent aussi à eux.
    fn niveaux_et_essai() -> impl Iterator<Item = &'static InfosNiveau> {
        NIVEAUX.iter().chain([
            &essai_vertical::INFOS,
            &essai_vent::INFOS,
            &essai_grele::INFOS,
            &essai_auvent::INFOS,
            &essai_ventilateur::INFOS,
            &essai_montgolfiere::INFOS,
            &essai_escargot::INFOS,
            &essai_chat::INFOS,
            &essai_nuage::INFOS,
        ])
    }

    /// Le niveau d'essai vertical (45 rangées) passe les mêmes contrôles qu'un vrai
    /// niveau : arrivée, 3 gouttes, lanternes, robot maladroit compris.
    #[test]
    fn niveau_d_essai_vertical_faisable() {
        verifier_infos(&essai_vertical::INFOS);
    }

    /// Le niveau d'essai « vent » passe les mêmes contrôles : la traversée du vide
    /// n'est faisable que parce que le robot simule les vents latéraux.
    #[test]
    fn niveau_d_essai_vent_faisable() {
        verifier_infos(&essai_vent::INFOS);
    }

    /// Sans vents latéraux, le vide du niveau d'essai est trop large : le niveau
    /// doit être bloqué (c'est ce qui prouve que le vent fait la traversée).
    #[test]
    fn niveau_d_essai_vent_exige_les_vents() {
        let niv = essai_vent::INFOS.charger().unwrap();
        assert!(!niv.vents.is_empty());
        let r = analyser(
            &niv,
            niv.depart,
            Options {
                vents: false,
                ..Options::default()
            },
        );
        assert!(!r.arrivee, "sans vent le vide doit rester infranchissable");
    }

    /// Le niveau d'essai « grêle » passe les mêmes contrôles (arrivée, 3 gouttes,
    /// lanternes, robot maladroit) : le robot traverse les trois rideaux.
    #[test]
    fn niveau_d_essai_grele_faisable() {
        verifier_infos(&essai_grele::INFOS);
    }

    /// Les quatre colonnes du niveau d'essai « grêle » sont bien là où le dessin le
    /// dit : du ciel au toit (colonne 9), du ciel au bas de la carte au-dessus du
    /// vide (colonnes 21 et 22), et arrêtée par la corniche (colonne 28, ligne 5).
    /// Et sans parapluie, le niveau est bloqué.
    #[test]
    fn niveau_d_essai_grele_a_ses_trois_rideaux() {
        use crate::entities::Entite;
        let niv = essai_grele::INFOS.charger().unwrap();
        let t = crate::aabb::TAILLE_TUILE;
        let mut colonnes: Vec<(i32, f32, f32)> = niv
            .entites
            .iter()
            .filter_map(|e| match e {
                Entite::Grele(g) => Some(((g.zone.x / t) as i32, g.zone.y / t, g.zone.bas() / t)),
                _ => None,
            })
            .collect();
        colonnes.sort_by_key(|c| c.0);
        assert_eq!(
            colonnes,
            vec![
                (9, 1.0, 8.0),
                (21, 2.0, 11.0),
                (22, 2.0, 11.0),
                (28, 1.0, 5.0)
            ]
        );
        let r = analyser(
            &niv,
            niv.depart,
            Options {
                planer: false,
                ..Options::default()
            },
        );
        assert!(!r.arrivee, "sans parapluie, la grêle et le vide bloquent");
    }

    /// Le niveau d'essai « auvent » passe les mêmes contrôles (arrivée, 3 gouttes,
    /// lanternes, robot maladroit) : le robot enchaîne les six auvents.
    #[test]
    fn niveau_d_essai_auvent_faisable() {
        verifier_infos(&essai_auvent::INFOS);
    }

    /// Six auvents au-dessus du vide, aux colonnes 9, 13, 17, 22, 25 et 29 (écarts
    /// de 4, 4, 5, 3 et 4 tuiles) ; sans eux, le vide est infranchissable : c'est
    /// bien la chaîne d'auvents qui fait la traversée.
    #[test]
    fn niveau_d_essai_auvent_exige_les_auvents() {
        use crate::entities::Entite;
        let niv = essai_auvent::INFOS.charger().unwrap();
        let t = crate::aabb::TAILLE_TUILE;
        let colonnes: Vec<i32> = niv
            .entites
            .iter()
            .filter_map(|e| match e {
                Entite::Auvent(a) => Some((a.boite.x / t) as i32),
                _ => None,
            })
            .collect();
        assert_eq!(colonnes, vec![9, 13, 17, 22, 25, 29]);
        let mut sans = niv.clone();
        sans.entites.retain(|e| !matches!(e, Entite::Auvent(_)));
        let r = analyser(&sans, sans.depart, Options::default());
        assert!(
            !r.arrivee,
            "sans auvents, le vide doit rester infranchissable"
        );
    }

    /// Le niveau d'essai « ventilateur » passe les mêmes contrôles (arrivée, 3
    /// gouttes, lanternes, robot maladroit) : le robot monte par le souffle.
    #[test]
    fn niveau_d_essai_ventilateur_faisable() {
        verifier_infos(&essai_ventilateur::INFOS);
    }

    /// Sans les courants ascendants (le souffle du ventilateur), le balcon du
    /// drapeau est hors de portée : c'est bien le ventilateur seul qui fait la
    /// montée (le saut ne porte pas à 18 tuiles).
    #[test]
    fn niveau_d_essai_ventilateur_exige_les_courants() {
        let niv = essai_ventilateur::INFOS.charger().unwrap();
        let r = analyser(
            &niv,
            niv.depart,
            Options {
                courants: false,
                ..Options::default()
            },
        );
        assert!(
            !r.arrivee,
            "sans le souffle du ventilateur, le balcon doit rester hors de portée"
        );
    }

    /// Le niveau d'essai « montgolfière » passe les mêmes contrôles (arrivée, 3
    /// gouttes, lanternes, robot maladroit) : le robot gravit les deux étages en
    /// montgolfière.
    #[test]
    fn niveau_d_essai_montgolfiere_faisable() {
        verifier_infos(&essai_montgolfiere::INFOS);
    }

    /// Sans les montgolfières (les `b` remplacés par du vide), aucun étage n'est à
    /// portée : ni l'étage B depuis le quai (112 px), ni l'étage du drapeau depuis
    /// l'étage B (64 px, plus qu'un saut). Avec la PREMIÈRE seulement, l'étage B est
    /// atteint mais pas le drapeau : chacune des deux sert.
    #[test]
    fn niveau_d_essai_montgolfiere_exige_les_deux_montgolfieres() {
        let ascii = essai_montgolfiere::INFOS.ascii;
        let sans = Niveau::depuis_ascii(&ascii.replace('b', ".")).unwrap();
        let r = analyser(&sans, sans.depart, Options::default());
        assert!(
            !r.arrivee,
            "sans montgolfière, le drapeau doit rester hors de portée"
        );
        // La seconde (ligne 9) retirée, la première (ligne 16) gardée.
        let lignes: Vec<String> = ascii
            .lines()
            .enumerate()
            .map(|(i, l)| {
                if i == 10 {
                    l.replace('b', ".")
                } else {
                    l.to_string()
                }
            })
            .collect();
        let une = Niveau::depuis_ascii(&lignes.join("\n")).unwrap();
        assert_eq!(
            une.entites
                .iter()
                .filter(|e| matches!(e, crate::entities::Entite::Montgolfiere(_)))
                .count(),
            1
        );
        let r = analyser(&une, une.depart, Options::default());
        assert!(
            !r.arrivee,
            "sans la seconde, le drapeau doit rester hors de portée"
        );
        assert!(
            r.gouttes[1],
            "mais la première porte bien à l'étage B : {r:?}"
        );
    }

    /// Le niveau d'essai « escargot » passe les mêmes contrôles (arrivée, 3 gouttes,
    /// lanternes, robot maladroit) : le robot gravit les deux falaises en
    /// rebondissant sur les escargots.
    #[test]
    fn niveau_d_essai_escargot_faisable() {
        verifier_infos(&essai_escargot::INFOS);
    }

    /// Sans les escargots (les `s` remplacés par du vide), la falaise 1 est hors de
    /// portée (80 px, plus qu'un saut). Avec le PREMIER seulement, la falaise 1 est
    /// atteinte mais pas la falaise 2 (trop loin, même en planant depuis la
    /// falaise 1) : chacun des deux sert.
    #[test]
    fn niveau_d_essai_escargot_exige_les_deux_escargots() {
        let ascii = essai_escargot::INFOS.ascii;
        let sans = Niveau::depuis_ascii(&ascii.replace('s', ".")).unwrap();
        let r = analyser(&sans, sans.depart, Options::default());
        // Gouttes dans l'ordre de lecture : 0 en l'air (ligne 1), 1 sur la falaise 2
        // (ligne 2), 2 sur la falaise 1 (ligne 3).
        assert!(
            !r.arrivee && r.gouttes == [false; 3],
            "sans escargot : {r:?}"
        );
        // Le second (colonne 28) retiré, le premier gardé.
        let lignes: Vec<String> = ascii
            .lines()
            .map(|l| {
                let mut c: Vec<char> = l.chars().collect();
                if c.len() > 28 && c[28] == 's' {
                    c[28] = '.';
                }
                c.into_iter().collect()
            })
            .collect();
        let un = Niveau::depuis_ascii(&lignes.join("\n")).unwrap();
        assert_eq!(
            un.entites
                .iter()
                .filter(|e| matches!(e, crate::entities::Entite::Escargot(_)))
                .count(),
            1
        );
        let r = analyser(&un, un.depart, Options::default());
        assert!(
            !r.arrivee,
            "sans le second, le drapeau doit rester hors de portée"
        );
        assert!(
            r.gouttes[0] && r.gouttes[2] && !r.gouttes[1],
            "mais le premier porte bien sur la falaise 1 (et à la goutte en l'air) : {r:?}"
        );
    }

    /// Le niveau d'essai « chat » passe les mêmes contrôles (arrivée, 3 gouttes,
    /// lanternes, robot maladroit) : le robot tombe du toit bas sur le chat
    /// endormi, l'écrase, et passe dessous le grand mur.
    #[test]
    fn niveau_d_essai_chat_faisable() {
        verifier_infos(&essai_chat::INFOS);
    }

    /// La case du chat est le SEUL passage : bouchée par un toit (`#` à la place
    /// du `z`), ni la goutte cachée ni le drapeau ne sont atteints ; seule la
    /// goutte du haut de la fente l'est. (Le chat vivant, lui, ne se traverse pas : voir
    /// `faisabilite::tests::marcher_dans_un_chat_vivant_tue_le_robot`.)
    #[test]
    fn niveau_d_essai_chat_le_chat_garde_le_seul_passage() {
        let bouche = Niveau::depuis_ascii(&essai_chat::INFOS.ascii.replace('z', "#")).unwrap();
        let r = analyser(&bouche, bouche.depart, Options::default());
        // Gouttes dans l'ordre de lecture : 0 en haut de la fente (ligne 6, encore
        // atteinte : on se pose sur le toit qui bouche la case), 1 cachée derrière
        // le chat, 2 avant le drapeau (ligne 8).
        assert!(
            !r.arrivee && r.gouttes == [true, false, false],
            "passage bouché : {r:?}"
        );
    }

    /// Le niveau d'essai « horloge » passe les mêmes contrôles (arrivée, 3 gouttes,
    /// lanternes, robot maladroit) : le robot se laisse porter par les deux
    /// aiguilles d'une tour à l'autre.
    #[test]
    fn niveau_d_essai_horloge_faisable() {
        verifier_infos(&essai_horloge::INFOS);
    }

    /// Les deux aiguilles servent. Sans aiguille (les `h` vidés), ni la tour B ni
    /// le drapeau ne sont atteints : seule la goutte du sol l'est. Sans la petite
    /// (la 1re, en haut), la tour B l'est (goutte 1) mais pas la tour C.
    #[test]
    fn niveau_d_essai_horloge_exige_les_deux_aiguilles() {
        let ascii = essai_horloge::INFOS.ascii;
        let sans = Niveau::depuis_ascii(&ascii.replace('h', ".")).unwrap();
        let r = analyser(&sans, sans.depart, Options::default());
        // Gouttes dans l'ordre de lecture : 0 sur la tour C, 1 sur la tour B, 2 au sol.
        assert!(
            !r.arrivee && r.gouttes == [false, false, true],
            "sans aiguille : {r:?}"
        );
        let grande = Niveau::depuis_ascii_avec_aiguilles(
            &ascii.replacen('h', ".", 1),
            &essai_horloge::INFOS.aiguilles[1..],
        )
        .unwrap();
        let r = analyser(&grande, grande.depart, Options::default());
        assert!(
            !r.arrivee && r.gouttes == [false, true, true],
            "sans la petite aiguille : {r:?}"
        );
    }

    /// L'arène du Grand Nuage (essai) : pas de drapeau, l'« arrivée » du robot est
    /// la tête du nuage (essoufflé, ses deux courants actifs : voir l'en-tête de
    /// `faisabilite`). Il l'atteint, précis comme maladroit, prend les 3 gouttes ;
    /// une seule lanterne (le début de l'arène : on y reprend après une mort), d'où
    /// il l'atteint aussi.
    #[test]
    fn niveau_d_essai_nuage_faisable() {
        let niv = essai_nuage::INFOS.charger().unwrap();
        let r = analyser(&niv, niv.depart, Options::default());
        assert!(r.arrivee && r.gouttes == [true; 3], "{r:?}");
        assert!(r.lanternes.iter().all(|&l| l) && r.lanternes.len() == 1);
        let maladroit = Options {
            precis: false,
            ..Options::default()
        };
        assert!(analyser(&niv, niv.depart, maladroit).arrivee);
    }

    /// Chacun des deux courants suffit pour atteindre sa tête (ils alternent d'un
    /// motif à l'autre). Sans courant, la tête est hors de portée (80 px au-dessus
    /// du sol), et seule la goutte du sol est prise.
    #[test]
    fn niveau_d_essai_nuage_chaque_courant_mene_a_la_tete() {
        use crate::entities::Entite;
        let niv = essai_nuage::INFOS.charger().unwrap();
        // Un courant « éteint » : réduit à une zone vide, sous le sol.
        let eteint = crate::aabb::Aabb::new(0.0, 400.0, 0.0, 0.0);
        let avec = |gauche: bool, droite: bool| {
            let mut n = niv.clone();
            for e in &mut n.entites {
                if let Entite::Nuage(g) = e {
                    if !gauche {
                        g.courants[0] = eteint;
                    }
                    if !droite {
                        g.courants[1] = eteint;
                    }
                }
            }
            analyser(&n, n.depart, Options::default())
        };
        // Gouttes dans l'ordre de lecture : 0 en haut à gauche, 1 en haut à droite,
        // 2 au sol.
        let gauche = avec(true, false);
        assert!(
            gauche.arrivee && gauche.gouttes == [true, false, true],
            "{gauche:?}"
        );
        let droite = avec(false, true);
        assert!(
            droite.arrivee && droite.gouttes == [false, true, true],
            "{droite:?}"
        );
        let aucun = avec(false, false);
        assert!(
            !aucun.arrivee && aucun.gouttes == [false, false, true],
            "{aucun:?}"
        );
    }

    /// Ni le départ ni aucune lanterne sous la grêle : Pépin y apparaît (ou y
    /// réapparaît) au sol, parapluie fermé, et la grêle le tuerait dès la fin de sa
    /// protection (`PROTECTION_REAPPARITION_PAS`).
    #[test]
    fn aucun_point_de_reprise_sous_la_grele() {
        use crate::aabb::Aabb;
        use crate::entities::Entite;
        use crate::physics::{JOUEUR_H, JOUEUR_L};
        for infos in niveaux_et_essai() {
            let niv = infos.charger().unwrap();
            let mut reprises = vec![niv.depart];
            for e in &niv.entites {
                if let Entite::Lanterne(l) = e {
                    reprises.push(l.point_de_reapparition());
                }
            }
            for e in &niv.entites {
                let Entite::Grele(g) = e else { continue };
                for &(x, y) in &reprises {
                    assert!(
                        !Aabb::new(x, y, JOUEUR_L, JOUEUR_H).chevauche(&g.zone),
                        "{} : point de reprise ({x}, {y}) sous la grêle {:?}",
                        infos.nom,
                        g.zone
                    );
                }
            }
        }
    }

    /// Un test de faisabilité PAR niveau (ils tournent en parallèle). La macro écrit
    /// les fonctions de test et la liste des indices couverts : un niveau ajouté sans
    /// sa ligne ici fait échouer `chaque_niveau_a_son_test_de_faisabilite`.
    macro_rules! tests_de_faisabilite {
        ($($nom:ident => $indice:expr),* $(,)?) => {
            $(
                #[test]
                fn $nom() {
                    verifier($indice);
                }
            )*
            const INDICES_VERIFIES: &[usize] = &[$($indice),*];
        };
    }

    tests_de_faisabilite! {
        niveau_1_faisable => 0,
        niveau_2_faisable => 1,
        niveau_3_faisable => 2,
        niveau_4_faisable => 3,
        niveau_5_faisable => 4,
        niveau_6_faisable => 5,
        niveau_7_faisable => 6,
    }

    #[test]
    fn chaque_niveau_a_son_test_de_faisabilite() {
        let mut v = INDICES_VERIFIES.to_vec();
        v.sort_unstable();
        assert_eq!(v, (0..NB_NIVEAUX).collect::<Vec<_>>());
    }

    #[test]
    fn niveau_1_exige_de_planer() {
        let niv = crate::levels::toits::INFOS.charger().unwrap();
        let r = analyser(
            &niv,
            niv.depart,
            Options {
                planer: false,
                ..Options::default()
            },
        );
        println!("sans planer : {:?}", r);
        assert!(!r.arrivee, "le grand vide doit exiger le parapluie");
    }

    #[test]
    fn niveau_6_exige_les_courants_ascendants() {
        let niv = crate::levels::vents::INFOS.charger().unwrap();
        assert!(!niv.courants.is_empty());
        let r = analyser(
            &niv,
            niv.depart,
            Options {
                courants: false,
                ..Options::default()
            },
        );
        assert!(!r.arrivee, "sans vent le niveau 6 doit être bloqué");
    }

    /// Le marché couvert (niveau 3) : son moment fort, six auvents au-dessus d'un
    /// vide de 23 tuiles (colonnes 85 à 105, tous espacés de 4 tuiles), ne se
    /// franchit pas sans eux.
    #[test]
    fn niveau_3_le_marche_exige_ses_six_auvents() {
        use crate::entities::Entite;
        use crate::levels::marche;
        let niv = marche::INFOS.charger().unwrap();
        let t = crate::aabb::TAILLE_TUILE;
        let colonnes: Vec<i32> = niv
            .entites
            .iter()
            .filter_map(|e| match e {
                Entite::Auvent(a) => Some((a.boite.x / t) as i32),
                _ => None,
            })
            .collect();
        assert_eq!(
            colonnes,
            vec![20, 40, 44, 85, 89, 93, 97, 101, 105, 135, 139]
        );
        // Sans les six auvents du moment fort (les autres restent), on bute sur le vide.
        let mut sans = niv.clone();
        sans.entites.retain(
            |e| !matches!(e, Entite::Auvent(a) if (85.0 * t..=105.0 * t).contains(&a.boite.x)),
        );
        let r = analyser(&sans, sans.depart, Options::default());
        assert!(
            !r.arrivee,
            "sans les six auvents, le vide du moment fort doit rester infranchissable"
        );
        assert!(r.x_max < 108.0 * t, "x max {}", r.x_max);
    }

    /// Le marché couvert : la goutte cachée (colonne 67, au fond du passage) est
    /// gardée par le chat de la fente (colonne 64). Fente bouchée (un toit à la
    /// place du chat), la goutte n'est plus atteinte ; l'arrivée, si (le chemin
    /// principal passe par-dessus le mur).
    #[test]
    fn niveau_3_le_chat_de_la_fente_garde_la_goutte_cachee() {
        let grille: Vec<&str> = crate::levels::marche::INFOS
            .ascii
            .lines()
            .filter(|l| !l.is_empty())
            .collect();
        let fente: String = grille
            .iter()
            .enumerate()
            .map(|(r, l)| {
                if r == 8 {
                    assert_eq!(&l[64..68], "z..o", "la fente et la goutte cachée");
                    format!("{}#{}", &l[..64], &l[65..])
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let bouche = Niveau::depuis_ascii(&fente).unwrap();
        let r = analyser(&bouche, bouche.depart, Options::default());
        // Gouttes dans l'ordre de lecture : 0 et 1 en rangée 5 (premier auvent,
        // moment fort), 2 cachée (rangée 8).
        assert!(
            r.arrivee && r.gouttes == [true, true, false],
            "fente bouchée : {r:?}"
        );
    }

    /// Le marché couvert : on RESSORT de la cachette. Pépin posé sur la goutte
    /// cachée (colonne 67, rangée 8), le chat de la fente déjà écrasé (on n'entre
    /// pas autrement) : le drapeau reste atteignable (on remonte la fente d'un
    /// saut, sur le bloc de boutique). Aucun piège sans retour au fond du passage.
    #[test]
    fn niveau_3_on_ressort_de_la_cachette_du_chat() {
        let lignes: Vec<String> = crate::levels::marche::INFOS
            .ascii
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| {
                let mut l = l.to_string();
                if l.as_bytes().get(64) == Some(&b'z') {
                    l.replace_range(64..65, "."); // le chat de la fente, écrasé
                }
                l
            })
            .collect();
        let ecrase = Niveau::depuis_ascii(&lignes.join("\n")).unwrap();
        let t = crate::aabb::TAILLE_TUILE;
        let cachette = (67.0 * t + 3.0, 9.0 * t - crate::physics::JOUEUR_H);
        let r = analyser(&ecrase, cachette, Options::default());
        assert!(r.arrivee, "coincé dans la cachette : {r:?}");
    }

    /// Vérifie un niveau : arrivée, 3 gouttes, toutes les lanternes atteignables,
    /// y compris par le robot « maladroit » (pas de précision au pixel près).
    fn verifier(i: usize) {
        verifier_infos(&NIVEAUX[i]);
    }

    fn verifier_infos(infos: &InfosNiveau) {
        let niv = infos.charger().unwrap();
        let r = analyser(&niv, niv.depart, Options::default());
        let n = infos.numero;
        println!(
            "niveau {n} : {:.1} s en course rapide, {} états",
            r.pas_arrivee.unwrap_or(0) as f32 / 60.0,
            r.etats
        );
        assert!(
            r.arrivee,
            "niveau {n} : arrivée inatteignable (x max {} en {:?})",
            r.x_max, r.dernier_etat
        );
        assert_eq!(r.gouttes, [true; 3], "niveau {n} : gouttes {:?}", r.gouttes);
        assert!(
            r.lanternes.iter().all(|&l| l),
            "niveau {n} : lanternes {:?}",
            r.lanternes
        );
        assert_eq!(r.lanternes.len(), niv.nb_lanternes());
        assert!(
            niv.nb_lanternes() >= 3,
            "niveau {n} : au moins 3 checkpoints"
        );

        let maladroit = analyser(
            &niv,
            niv.depart,
            Options {
                precis: false,
                ..Options::default()
            },
        );
        assert!(
            maladroit.arrivee,
            "niveau {n} : trop exigeant (robot maladroit bloqué à x = {})",
            maladroit.x_max
        );
    }

    /// Chaque lanterne est aussi un point de reprise : le niveau doit se
    /// terminer DEPUIS elle (sinon un respawn pourrait coincer le joueur).
    #[test]
    fn reprise_depuis_chaque_lanterne() {
        for infos in niveaux_et_essai() {
            let niv = infos.charger().unwrap();
            for e in &niv.entites {
                if let crate::entities::Entite::Lanterne(l) = e {
                    let r = analyser(&niv, l.point_de_reapparition(), Options::default());
                    assert!(
                        r.arrivee,
                        "niveau {} : bloqué depuis la lanterne en {:?}",
                        infos.numero,
                        l.point_de_reapparition()
                    );
                }
            }
        }
    }

    /// Un niveau dure 40 à 60 s pour un joueur normal. Le robot « course
    /// rapide » est ~2 fois plus rapide qu'un humain découvrant le niveau.
    #[test]
    fn durees_plausibles() {
        for infos in NIVEAUX {
            let niv = infos.charger().unwrap();
            let r = analyser(&niv, niv.depart, Options::default());
            let s = r.pas_arrivee.unwrap() as f32 / 60.0;
            println!(
                "niveau {} : course rapide {s:.1} s (cible {} s)",
                infos.numero, infos.duree_cible_s
            );
            assert!(
                (12.0..=45.0).contains(&s),
                "niveau {} : {s:.1} s",
                infos.numero
            );
            assert!((40..=60).contains(&infos.duree_cible_s));
        }
    }

    #[test]
    fn premier_defi_visible_des_le_premier_ecran() {
        // Fun immédiat : à l'écran de départ (320 px de large) on voit déjà une
        // goutte, un ennemi, un éclair, un courant ou un trampoline.
        for infos in NIVEAUX {
            let niv = infos.charger().unwrap();
            let ecran = crate::LARGEUR_LOGIQUE as f32;
            let proche = niv.courants.iter().any(|c| c.zone.x < ecran)
                || niv.entites.iter().any(|e| match e {
                    crate::entities::Entite::Goutte(g) => g.boite.x < ecran,
                    crate::entities::Entite::Ennemi(en) => en.boite.x < ecran,
                    crate::entities::Entite::Eclair(ec) => ec.zone.x < ecran,
                    crate::entities::Entite::Champignon(c) => c.boite.x < ecran,
                    _ => false,
                });
            assert!(
                proche,
                "niveau {} : rien d'intéressant à l'écran de départ",
                infos.numero
            );
        }
    }

    /// Grille ASCII d'un niveau : les lignes du dessin, rangée 0 en haut.
    fn grille(infos: &InfosNiveau) -> Vec<Vec<char>> {
        infos
            .ascii
            .lines()
            .map(str::trim_end)
            .filter(|l| !l.is_empty())
            .map(|l| l.chars().collect())
            .collect()
    }

    /// Tuile pleine du dessin (`#` toit, `B` brique, `N` nuage, rien d'autre).
    /// Hors de la grille à gauche et à droite : mur ; en haut : ciel ouvert.
    fn plein(g: &[Vec<char>], x: i32, y: i32) -> bool {
        if x < 0 || x >= g[0].len() as i32 {
            return true;
        }
        if y < 0 || y >= g.len() as i32 {
            return false;
        }
        matches!(g[y as usize][x as usize], '#' | 'B' | 'N')
    }

    /// Une lanterne (point de reprise) ne flotte jamais : la case juste en
    /// dessous est un solide (`#`, `B` ou `N`).
    #[test]
    fn aucune_lanterne_ne_flotte() {
        for infos in niveaux_et_essai() {
            infos.charger().unwrap();
            let g = grille(infos);
            for (y, ligne) in g.iter().enumerate() {
                for (x, &c) in ligne.iter().enumerate() {
                    if c == 'L' {
                        assert!(
                            plein(&g, x as i32, y as i32 + 1) && y + 1 < g.len(),
                            "niveau {} : la lanterne de la colonne {x}, rangée {y} flotte \
                             (la case en dessous n'est pas un solide)",
                            infos.numero
                        );
                    }
                }
            }
        }
    }

    /// Aucune « poche close » : une case vide coiffée d'un solide dont la
    /// colonne de vide (jusqu'en bas) est bordée d'un solide des deux côtés à
    /// chaque rangée est inaccessible et se voit comme un trou sombre.
    #[test]
    fn aucune_poche_de_vide_close() {
        for infos in niveaux_et_essai() {
            infos.charger().unwrap();
            let g = grille(infos);
            for y in 1..g.len() as i32 {
                for x in 0..g[0].len() as i32 {
                    if plein(&g, x, y) || !plein(&g, x, y - 1) {
                        continue;
                    }
                    let mut fin = y;
                    while fin < g.len() as i32 && !plein(&g, x, fin) {
                        fin += 1;
                    }
                    let close = (y..fin).all(|r| plein(&g, x - 1, r) && plein(&g, x + 1, r));
                    assert!(
                        !close,
                        "niveau {} : poche de vide close en colonne {x}, rangées {y} à {} \
                         (coiffée et bordée de solide, inaccessible)",
                        infos.numero,
                        fin - 1
                    );
                }
            }
        }
    }

    /// Dangers qui passent sur le point de réapparition d'une lanterne, niveau
    /// par niveau : `(numéro du niveau, colonne de la lanterne, danger)`.
    ///
    /// Les entités du niveau avancent seules (sans joueur) pendant `pas` pas ; on
    /// note chaque lanterne dont la boîte de Pépin réapparu est touchée par un
    /// ennemi actif (flaque, corbeau), une pointe ou un éclair qui frappe.
    fn dangers_sur_les_lanternes(pas: u32) -> Vec<(u8, i32, &'static str)> {
        use crate::aabb::Aabb;
        use crate::entities::{solides_des_entites, Actualisable, Entite, TypeEnnemi};
        use crate::physics::{Environnement, JOUEUR_H, JOUEUR_L};
        let mut trouves = Vec::new();
        for infos in NIVEAUX {
            let niv = infos.charger().unwrap();
            let pepins: Vec<(i32, Aabb)> = niv
                .entites
                .iter()
                .filter_map(|e| match e {
                    Entite::Lanterne(l) => {
                        let (x, y) = l.point_de_reapparition();
                        let colonne = (l.boite.x / crate::aabb::TAILLE_TUILE) as i32;
                        Some((colonne, Aabb::new(x, y, JOUEUR_L, JOUEUR_H)))
                    }
                    _ => None,
                })
                .collect();
            let mut entites = niv.entites.clone();
            for _ in 0..pas {
                let solides = solides_des_entites(&entites);
                let env = Environnement {
                    grille: &niv,
                    solides: &solides,
                    courants: &niv.courants,
                    vents: &niv.vents,
                };
                for e in entites.iter_mut() {
                    e.actualiser(&env);
                }
                for e in &entites {
                    let (zone, nom) = match e {
                        Entite::Ennemi(en) if en.est_dangereux() => match en.type_ennemi {
                            TypeEnnemi::Flaque => (en.boite, "flaque"),
                            TypeEnnemi::Corbeau => (en.boite, "corbeau"),
                            TypeEnnemi::Pigeon => (en.boite, "pigeon"),
                            TypeEnnemi::Chat => (en.boite, "chat"),
                        },
                        Entite::Pointe(p) => (p.zone_danger(), "pointe"),
                        Entite::Eclair(ec) if ec.frappe() => (ec.zone, "éclair"),
                        _ => continue,
                    };
                    for &(colonne, pepin) in &pepins {
                        let trouve = (infos.numero, colonne, nom);
                        if pepin.chevauche(&zone) && !trouves.contains(&trouve) {
                            trouves.push(trouve);
                        }
                    }
                }
            }
        }
        trouves
    }

    /// Inventaire des lanternes sur le trajet d'un danger (carte 3.1b). Depuis la
    /// protection de réapparition (`PROTECTION_REAPPARITION_PAS`), ce n'est plus un
    /// piège (au plus une mort par passage), mais c'est à savoir en dessinant un
    /// niveau. Trois aujourd'hui, toutes sur la patrouille d'une flaque qui longe
    /// son toit d'un mur (ou d'un bord) à l'autre : niveau 2, colonnes 66 (flaque
    /// de la colonne 70) et 104 (flaque de la colonne 108, celle de la mesure 3.1) ;
    /// niveau 4 (le grand orage, niveau 3 avant l'arrivée du marché couvert),
    /// colonne 145 (flaque de la colonne 154). Le marché couvert n'en ajoute aucune
    /// (ses flaques gardent des toits sans lanterne, ses chats dorment loin des
    /// lanternes). Si ce test échoue après une
    /// retouche de niveau : vérifier la nouvelle liste, la recopier ici et dans
    /// `QA.md` (section « Protection après réapparition »).
    #[test]
    fn lanternes_sur_le_trajet_d_un_danger() {
        // 2 min : plus qu'un aller-retour complet de chaque ennemi (la plus longue
        // patrouille de flaque, un toit de 61 tuiles au niveau 4, dure 80 s à 24 px/s).
        let trouves = dangers_sur_les_lanternes(120 * 60);
        println!("lanternes sur le trajet d'un danger : {trouves:?}");
        assert_eq!(
            trouves,
            vec![(2, 66, "flaque"), (2, 104, "flaque"), (4, 145, "flaque")]
        );
    }

    // --- Objets de décor (carte 4.2) -----------------------------------------

    use crate::level::{GenreDecor, ObjetDecor};

    /// Case `(tx, ty)` d'un objet de décor.
    fn case_decor(d: &ObjetDecor) -> (i32, i32) {
        let t = crate::aabb::TAILLE_TUILE;
        ((d.x / t) as i32, (d.y / t) as i32)
    }

    /// Place qu'un objet de décor peut occuper à l'écran : sa case et celle du
    /// dessus (les plus hauts sprites font 29 px), élargies de 4 px de chaque côté
    /// (les sprites des objets et des ennemis débordent un peu de leur boîte).
    fn place_decor(d: &ObjetDecor) -> crate::aabb::Aabb {
        let t = crate::aabb::TAILLE_TUILE;
        crate::aabb::Aabb::new(d.x - 4.0, d.y - t, t + 8.0, 2.0 * t)
    }

    /// Chaque objet de décor est posé au milieu d'un toit : sa case est vide, la
    /// case du dessous est solide, et ses deux voisines du dessous aussi (jamais au
    /// bord d'un toit, là où l'on prend son élan ou où l'on se rattrape).
    #[test]
    fn chaque_objet_de_decor_repose_sur_un_solide() {
        for infos in NIVEAUX {
            let niv = infos.charger().unwrap();
            for d in &niv.decors {
                let (tx, ty) = case_decor(d);
                let n = infos.numero;
                assert!(!niv.tuile_decor(tx, ty).est_solide());
                for dx in -1..=1 {
                    assert!(
                        niv.tuile_decor(tx + dx, ty + 1).est_solide()
                            && ty + 1 < niv.hauteur() as i32,
                        "niveau {n} : {:?} en colonne {tx}, rangée {ty} : la case ({}, {}) \
                         dessous n'est pas un solide",
                        d.genre,
                        tx + dx,
                        ty + 1
                    );
                }
            }
        }
    }

    /// Le décor ne sert qu'au dessin : en remplaçant ses lettres par `.`, le niveau
    /// est EXACTEMENT le même (tuiles, entités, courants, panneaux, départ,
    /// arrivée). Aucun risque pour la faisabilité, le robot ni la mesure de difficulté.
    #[test]
    fn le_decor_ne_change_rien_a_la_collision() {
        for infos in NIVEAUX {
            let avec = infos.charger().unwrap();
            let sans_texte: String = infos
                .ascii
                .chars()
                .map(|c| match GenreDecor::depuis_lettre(c) {
                    Some(_) => '.',
                    None => c,
                })
                .collect();
            let sans = Niveau::depuis_ascii_avec_aiguilles(&sans_texte, infos.aiguilles).unwrap();
            let n = infos.numero;
            assert!(!avec.decors.is_empty(), "niveau {n} : aucun décor");
            assert!(sans.decors.is_empty());
            assert_eq!(
                (avec.largeur(), avec.hauteur()),
                (sans.largeur(), sans.hauteur())
            );
            for ty in -1..=avec.hauteur() as i32 {
                for tx in -1..=avec.largeur() as i32 {
                    assert_eq!(
                        avec.tuile_decor(tx, ty),
                        sans.tuile_decor(tx, ty),
                        "niveau {n} : tuile ({tx}, {ty})"
                    );
                }
            }
            // Les entités n'implémentent pas `PartialEq` : leur description `Debug`
            // complète (positions, états, minuteries) suffit à les comparer.
            assert_eq!(
                format!("{:?}", avec.entites),
                format!("{:?}", sans.entites),
                "niveau {n} : entités"
            );
            assert_eq!(avec.courants, sans.courants, "niveau {n} : courants");
            assert_eq!(avec.panneaux, sans.panneaux, "niveau {n} : panneaux");
            assert_eq!(avec.depart, sans.depart);
            assert_eq!(avec.arrivee, sans.arrivee);
        }
    }

    /// Le décor raconte le niveau : cheminées et antennes sur les toits (1, 2 et le
    /// marché couvert, sans manche à air : on ne doit pas la confondre avec un
    /// auvent), paratonnerres sous l'orage, manches à air au vent, oiseaux au soleil.
    #[test]
    fn chaque_niveau_a_son_decor() {
        use crate::levels::{clocher, gouttieres, marche, nuages, orage, toits, vents};
        use GenreDecor::*;
        // Rangé par niveau (et non par place dans la table) : insérer un niveau ne
        // décale rien ; un niveau absent de cette liste n'a pas de décor imposé.
        let attendus: &[(InfosNiveau, &[GenreDecor])] = &[
            (toits::INFOS, &[Cheminee, Antenne]),
            (gouttieres::INFOS, &[Cheminee, Antenne]),
            (marche::INFOS, &[Cheminee, Antenne]),
            (orage::INFOS, &[Paratonnerre]),
            (clocher::INFOS, &[Paratonnerre]),
            (vents::INFOS, &[MancheAAir]),
            (nuages::INFOS, &[Oiseau]),
        ];
        for &(infos, genres) in attendus {
            assert!(
                NIVEAUX.iter().any(|n| n.nom == infos.nom),
                "{} hors de la table",
                infos.nom
            );
            let niv = infos.charger().unwrap();
            let n = infos.numero;
            assert!(niv.decors.len() >= 3, "niveau {n} : au moins 3 objets");
            for d in &niv.decors {
                assert!(genres.contains(&d.genre), "niveau {n} : {:?}", d.genre);
            }
            for g in genres {
                assert!(
                    niv.decors.iter().any(|d| d.genre == *g),
                    "niveau {n} : pas de {g:?}"
                );
            }
        }
    }

    /// Chaque manche à air signale un courant ascendant : elle se tient à 4 tuiles
    /// au plus de son bord, à une hauteur que le courant traverse.
    #[test]
    fn chaque_manche_a_air_est_au_bord_d_un_courant() {
        let t = crate::aabb::TAILLE_TUILE;
        for infos in NIVEAUX {
            let niv = infos.charger().unwrap();
            for d in niv
                .decors
                .iter()
                .filter(|d| d.genre == GenreDecor::MancheAAir)
            {
                let signale = niv.courants.iter().any(|c| {
                    let z = c.zone;
                    let ecart = (z.x - (d.x + t)).max(d.x - (z.x + z.w));
                    (0.0..=4.0 * t).contains(&ecart) && z.y <= d.y && d.y < z.y + z.h
                });
                assert!(
                    signale,
                    "niveau {} : manche à air en {:?} loin de tout courant",
                    infos.numero,
                    case_decor(d)
                );
            }
        }
    }

    /// Jamais devant un danger ni sur un objet à voir : pendant 2 minutes de jeu
    /// (plus qu'un aller-retour de chaque ennemi), aucune entité (ennemi, pointe,
    /// colonne d'éclair, plateforme, lanterne, goutte, champignon) ne passe sur la
    /// place d'un objet de décor ; aucun panneau, ni le départ, ni l'arrivée non plus.
    #[test]
    fn aucun_decor_devant_un_danger_ni_un_objet() {
        use crate::aabb::Aabb;
        use crate::entities::{solides_des_entites, Actualisable};
        use crate::physics::{Environnement, JOUEUR_H, JOUEUR_L};
        let t = crate::aabb::TAILLE_TUILE;
        for infos in NIVEAUX {
            let niv = infos.charger().unwrap();
            let n = infos.numero;
            let places: Vec<(Aabb, &ObjetDecor)> =
                niv.decors.iter().map(|d| (place_decor(d), d)).collect();
            let mut fixes: Vec<(Aabb, String)> = niv
                .panneaux
                .iter()
                .map(|p| (Aabb::new(p.x, p.y, t, t), format!("{:?}", p.genre)))
                .collect();
            fixes.extend(niv.arrivee.map(|a| (a, "arrivée".into())));
            let (dx, dy) = niv.depart;
            fixes.push((Aabb::new(dx, dy, JOUEUR_L, JOUEUR_H), "départ".into()));
            for (place, d) in &places {
                for (b, nom) in &fixes {
                    assert!(
                        !place.chevauche(b),
                        "niveau {n} : {:?} en {:?} sur {nom}",
                        d.genre,
                        case_decor(d)
                    );
                }
            }
            let mut entites = niv.entites.clone();
            for pas in 0..120 * 60 {
                for e in &entites {
                    let b = e.boite();
                    for (place, d) in &places {
                        assert!(
                            !place.chevauche(&b),
                            "niveau {n}, pas {pas} : {:?} en {:?} sous {e:?}",
                            d.genre,
                            case_decor(d)
                        );
                    }
                }
                let solides = solides_des_entites(&entites);
                let env = Environnement {
                    grille: &niv,
                    solides: &solides,
                    courants: &niv.courants,
                    vents: &niv.vents,
                };
                for e in entites.iter_mut() {
                    e.actualiser(&env);
                }
            }
        }
    }

    // --- Pigeons (carte 4.4) -------------------------------------------------

    use crate::entities::ennemi::{decalage_vol, PhasePigeon, PLONGEE_PAS};
    use crate::entities::{Ennemi, Entite as E, TypeEnnemi as T};

    /// Écart minimal (en tuiles, horizontal OU vertical) entre un pigeon et un
    /// oiseau de décor : on ne doit pas confondre l'oiseau inoffensif avec celui
    /// qui plonge (exigence du coordinateur, carte 4.4).
    const ECART_PIGEON_OISEAU: i32 = 4;

    /// Les pigeons d'un niveau.
    fn pigeons(niv: &Niveau) -> Vec<&Ennemi> {
        niv.entites
            .iter()
            .filter_map(|e| match e {
                E::Ennemi(p) if p.type_ennemi == T::Pigeon => Some(p),
                _ => None,
            })
            .collect()
    }

    /// Case `(tx, ty)` d'un pigeon perché.
    fn case_pigeon(p: &Ennemi) -> (i32, i32) {
        let t = crate::aabb::TAILLE_TUILE;
        let (x, y) = p.perchoir();
        ((x / t) as i32, (y / t) as i32)
    }

    /// Un à trois pigeons dans le Clocher (sur les corniches), Vents d'altitude et
    /// Au-dessus des nuages (le deuxième ennemi de ces niveaux), aucun ailleurs ;
    /// jamais à moins de 4 tuiles d'un oiseau de décor.
    #[test]
    fn les_pigeons_sont_dans_le_clocher_vents_et_nuages_loin_des_oiseaux_de_decor() {
        use crate::levels::{clocher, nuages, vents};
        for infos in NIVEAUX {
            let niv = infos.charger().unwrap();
            let n = infos.numero;
            let ps = pigeons(&niv);
            if [clocher::INFOS.nom, vents::INFOS.nom, nuages::INFOS.nom].contains(&infos.nom) {
                assert!(
                    (1..=3).contains(&ps.len()),
                    "niveau {n} : {} pigeons",
                    ps.len()
                );
            } else {
                assert!(ps.is_empty(), "niveau {n} : un pigeon");
            }
            for p in &ps {
                let (px, py) = case_pigeon(p);
                for d in niv.decors.iter().filter(|d| d.genre == GenreDecor::Oiseau) {
                    let (ox, oy) = case_decor(d);
                    let ecart = (px - ox).abs().max((py - oy).abs());
                    assert!(
                        ecart >= ECART_PIGEON_OISEAU,
                        "niveau {n} : pigeon en ({px}, {py}) à {ecart} tuile(s) de l'oiseau \
                         en ({ox}, {oy})"
                    );
                }
            }
        }
    }

    /// La plongée de chaque pigeon (toutes les positions de sa diagonale, dans le sens
    /// où il regarde) reste dans le ciel : elle ne traverse aucune tuile solide, sauf le
    /// coin de son propre perchoir au départ, et ne passe ni sur une lanterne (là où
    /// Pépin réapparaît) ni sur le départ.
    #[test]
    fn la_plongee_des_pigeons_reste_en_l_air_et_loin_des_lanternes() {
        use crate::aabb::{Aabb, TAILLE_TUILE as T16};
        use crate::physics::{JOUEUR_H, JOUEUR_L};
        for infos in NIVEAUX {
            let niv = infos.charger().unwrap();
            let n = infos.numero;
            let mut reperes: Vec<(Aabb, &str)> = niv
                .entites
                .iter()
                .filter_map(|e| match e {
                    E::Lanterne(l) => {
                        let (x, y) = l.point_de_reapparition();
                        Some((Aabb::new(x, y, JOUEUR_L, JOUEUR_H), "lanterne"))
                    }
                    _ => None,
                })
                .collect();
            let (dx, dy) = niv.depart;
            reperes.push((Aabb::new(dx, dy, JOUEUR_L, JOUEUR_H), "départ"));
            for p in pigeons(&niv) {
                let (cx, cy) = case_pigeon(p);
                let perchoir = (cx, cy + 1);
                let sens = if p.vers_la_droite { 1.0 } else { -1.0 };
                for k in 1..=PLONGEE_PAS {
                    let (ex, ey) = decalage_vol(PhasePigeon::Plongee(k), sens);
                    let b = p.boite.decale(ex, ey);
                    let (tx0, ty0) = ((b.x / T16).floor() as i32, (b.y / T16).floor() as i32);
                    for ty in ty0..=((b.bas() / T16).floor() as i32) {
                        for tx in tx0..=((b.droite() / T16).floor() as i32) {
                            let tuile = Aabb::new(tx as f32 * T16, ty as f32 * T16, T16, T16);
                            let solide = niv.tuile_decor(tx, ty).est_solide();
                            assert!(
                                !(solide && tuile.chevauche(&b)) || (tx, ty) == perchoir,
                                "niveau {n} : le pigeon de ({cx}, {cy}) plonge dans la tuile \
                                 ({tx}, {ty}) au pas {k}"
                            );
                        }
                    }
                    for (r, nom) in &reperes {
                        assert!(
                            !r.chevauche(&b),
                            "niveau {n} : le pigeon de ({cx}, {cy}) plonge sur {nom} au pas {k}"
                        );
                    }
                }
            }
        }
    }
}
