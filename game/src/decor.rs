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
    /// Falaise ou mur de blocs (Monde 2, carte 9.5) : une masse au sommet
    /// arrondi, scindée en blocs par des joints sombres.
    Roche,
    /// Roseaux (Monde 2, carte 9.5) : de fines tiges de hauteurs inégales,
    /// chacune coiffée d'une touffe.
    Roseaux,
    /// Coques de bateaux amarrés (Monde 2, carte 9.5) : une coque au fond
    /// creusé et un mât nu.
    Coques,
    /// Voûtes de tunnel (égouts, carte 9.27) : une arcade continue de piliers
    /// et d'arcs en gradins, les bouches des galeries voisines. Rien de dehors
    /// (ni toit, ni cheminée).
    Voutes,
    /// Stalagmites (grotte, carte 9.27) : des pointes de roche qui montent du
    /// sol en gradins, sans sommet plat (rien qui se lise comme un toit).
    Stalagmites,
}

/// Plafond de nuages d'orage : des dômes renversés accrochés en haut de l'écran,
/// sur deux couches de parallaxe (lointaine, proche).
#[derive(Debug, Clone, Copy)]
pub struct Plafond {
    pub loin: Couleur,
    pub proche: Couleur,
    /// Épaisseur des nuages : 4 = pleine, 2 = nuages discrets.
    pub ampleur: i32,
    /// Stalactites pointues (Monde 2, carte 9.5, la grotte) au lieu de dômes :
    /// des pointes vers le bas, de profondeurs inégales.
    pub stalactites: bool,
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
    /// Couleurs des traînées du vent latéral (carte 5.15), choisies pour se voir
    /// sur le ciel du thème.
    pub vent: CouleursVent,
    /// Un grand arc-en-ciel derrière les silhouettes (carte 5.16, le bonus) :
    /// `dessiner_arc`.
    pub arc: bool,
    /// Le ciel du voyage de retour (carte 5.17, l'épilogue) : au lieu des quatre
    /// bandes fixes, les bandes de `VOYAGE` défilent selon la hauteur de la caméra
    /// (`dessiner_voyage`) ; `ciel` et les silhouettes décrivent alors le BAS de la
    /// descente (la ville du premier niveau, la nuit).
    pub voyage: bool,
    /// Motifs de décor du Monde 2 (carte 9.5) : jusqu'à quatre, rangés dans
    /// l'ordre du dessin (`dessiner_detail` ; `None` = motif absent).
    pub details: [Option<Detail>; 4],
}

/// Couleurs d'une traînée de vent latéral (`particules::souffle_lateral`) : une
/// queue (`traine`) et une tête plus claire à l'avant (`tete`). Sur chaque bande
/// du ciel, l'une des deux au moins tranche nettement (test
/// `le_vent_lateral_se_voit_sur_chaque_bande_de_ciel`) : le vent se lit sur une
/// image fixe, et la tête claire dit son sens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CouleursVent {
    pub traine: Couleur,
    pub tete: Couleur,
}

/// Vent clair sur les ciels sombres (nuit, orage, clocher, cyclone).
const VENT_NUIT: CouleursVent = CouleursVent {
    traine: ARDOISE_PALE,
    tete: BLANC_CASSE,
};
/// Vent sombre sur les ciels clairs (corail, orange, or) : queue violette, tête
/// blanche.
const VENT_JOUR: CouleursVent = CouleursVent {
    traine: VIOLET_ORAGE,
    tete: BLANC_CASSE,
};

/// Éclaircissement des tuiles d'un thème : les toits pâlissent d'un niveau à l'autre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeinteTuiles {
    /// Crans gagnés par la rampe d'ardoise (0 = couleurs du dessin, jusqu'au blanc cassé).
    pub eclaircissement: u32,
    /// Les briques violettes virent à l'orange (sous le soleil).
    pub briques_orangees: bool,
    /// Les briques violettes virent au cyan sombre (Monde 2, carte 9.5) : les
    /// murs de briques mouillés des égouts et du port.
    pub briques_mouillees: bool,
}

/// Motif de décor du Monde 2 (carte 9.5) : un élément du fond qui n'est PAS une
/// couche de silhouettes : l'eau au pied de l'écran, les chutes, les bulles, les
/// grues... (dessinés par `dessiner_detail`, devant ou derrière la couche lointaine
/// selon le groupe).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Detail {
    /// Les lampadaires de la rue (le caniveau, carte 9.6b) : mât de fonte, lanterne
    /// orange et son halo (`decor_rue`).
    Lampadaires,
    /// Les maisons de la rue du caniveau (carte 9.6b) : façades, fenêtres
    /// allumées, vitrines sous leur store, descentes de gouttière (`decor_rue`).
    Facades,
    /// Les égouts : deux tuyaux qui traversent le tunnel et leurs colliers.
    Tuyaux,
    /// La lueur cyan des égouts : une bande d'eau sombre au sol, ses reflets.
    Lueur,
    /// Les chutes d'eau de la cascade : trois ruisselles qui tombent devant
    /// les falaises, leur mousse au pied.
    Chutes,
    /// La brume de la cascade : deux nappes basses qui dérivent.
    Brume,
    /// Les cristaux cyan de la grotte : des pointes au pied de l'écran.
    Cristaux,
    /// L'eau de la rivière : une bande de courant sombre, ses reflets.
    EauRiviere,
    /// La forêt de la rivière (carte 9.10) : deux rangs de sapins sombres à
    /// mi-hauteur, au-dessus des berges hautes du niveau.
    Foret,
    /// L'eau du port : une bande plus haute, une écume claire au bord.
    EauPort,
    /// L'eau de la marée : une bande sombre plus épaisse, l'écume qui monte.
    EauMaree,
    /// Les grues du port : deux mâts sombres, leur flèche et leur treuil.
    Grues,
    /// Les mouettes du port : trois petits oiseaux qui traversent le ciel.
    Mouettes,
    /// Le phare au loin, à l'entrée du port : une tour blanche cerclée de corail,
    /// sa lanterne qui clignote et son faisceau.
    Phare,
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
                briques_mouillees: false,
            },
            friable_nuage: false,
            vertige: 1.0,
            arc: false,
            voyage: false,
            details: [None; 4],
            vent: VENT_NUIT,
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
                stalactites: false,
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
                briques_mouillees: false,
            },
            friable_nuage: false,
            vertige: 1.0,
            arc: false,
            voyage: false,
            details: [None; 4],
            vent: VENT_NUIT,
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
                briques_mouillees: false,
            },
            friable_nuage: false,
            vertige: 1.0,
            arc: false,
            voyage: false,
            details: [None; 4],
            vent: VENT_NUIT,
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
                stalactites: false,
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
                briques_mouillees: false,
            },
            friable_nuage: false,
            vertige: 1.0,
            arc: false,
            voyage: false,
            details: [None; 4],
            vent: VENT_NUIT,
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
                briques_mouillees: false,
            },
            friable_nuage: false,
            vertige: 2.5,
            arc: false,
            voyage: false,
            details: [None; 4],
            vent: VENT_NUIT,
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
                stalactites: false,
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
                briques_mouillees: false,
            },
            friable_nuage: true,
            vertige: 1.0,
            arc: false,
            voyage: false,
            details: [None; 4],
            vent: VENT_NUIT,
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
                briques_mouillees: false,
            },
            friable_nuage: true,
            vertige: 1.0,
            arc: false,
            voyage: false,
            details: [None; 4],
            // Le ciel du haut est violet : la queue des traînées est bleu nuit.
            vent: CouleursVent {
                traine: ARDOISE_NUIT,
                tete: BLANC_CASSE,
            },
        },
        // Le retour du Grand Nuage (carte 8.11) : le même décor d'altitude, clair,
        // sans pluie de thème (`Theme::pluie`) : il ne pleut que dans le front d'orage.
        Theme::Poursuite => Ambiance {
            gouttes: 0,
            ..ambiance(Theme::Altitude)
        },
        // La mer de nuages (carte 5.15) : un couchant corail qui se dore vers
        // l'horizon ; tout en bas, la mer de nuages, rose (la lumière du couchant)
        // et sans trouée jusqu'au bas de l'écran, avec ses bancs en dôme ; devant,
        // des bancs violets dans l'ombre ; au milieu du ciel, des îlots corail au
        // ventre violet. Pas de pluie : seule la grêle tombe, en colonnes. Les
        // nuages du niveau (clairs, bleutés) ne se confondent avec aucune de ces
        // couleurs, et les gouttes d'or (lignes 1 à 3) se détachent sur le corail.
        // Carte 5.18 : les bancs roses du fond restent SOUS le plus bas des quais du
        // niveau (relief -16 : sommets à y = 150 au plus haut, le quai le plus bas
        // est à y = 144 ; en dessous, la mer de nuages, pleine jusqu'en bas). Avec
        // 10 px de relief, leur sommet plat montait jusqu'au dessus des plateformes,
        // dans les trouées : on aurait cru pouvoir s'y poser (captures
        // 515-plane-porte et 518-mer-plane-avant) ; avec -8, ils arrivaient encore à
        // la hauteur du quai bas de l'arrivée du moment fort (518-mer-rive).
        Theme::Mer => Ambiance {
            ciel: [CORAIL, CORAIL_CLAIR, ORANGE_SOLEIL, OR],
            lointain: CORAIL_CLAIR,
            proche: VIOLET_ORAGE,
            // Sans pluie (`Theme::pluie`) : couleurs jamais dessinées, mais
            // visibles sur chaque bande si un jour il pleuvait.
            pluie: (PLUIE, BLANC_CASSE),
            etoiles: None,
            gouttes: 0,
            descente: 0,
            plafond: None,
            soleil: false,
            forme: Forme::Nuages,
            relief: -16,
            socle: true,
            fenetres: None,
            ilots: Some((CORAIL, VIOLET_ORAGE)),
            tuiles: TeinteTuiles {
                eclaircissement: 2,
                briques_orangees: false,
                briques_mouillees: false,
            },
            friable_nuage: true,
            vertige: 1.0,
            arc: false,
            voyage: false,
            details: [None; 4],
            vent: VENT_JOUR,
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
                briques_mouillees: false,
            },
            friable_nuage: true,
            vertige: 1.0,
            arc: false,
            voyage: false,
            details: [None; 4],
            vent: VENT_JOUR,
        },
        // L'arc-en-ciel (carte 5.16, le bonus) : TOUTE LA PALETTE. L'averse vient
        // de s'arrêter : un ciel d'ardoise, sombre en haut, qui s'éclaircit vers
        // l'horizon ; devant lui, un grand arc-en-ciel de huit bandes (`ARC`), du
        // corail au violet, qui glisse lentement de droite à gauche (on le
        // traverse) ; en bas, des bancs de nuages pâles au loin et sombres devant.
        // Les nuages du pont, clairs et cernés de sombre, tranchent sur tout cela.
        // Pas de pluie, ni étoiles, ni soleil, ni îlots (un îlot clair se lirait
        // comme une dalle du pont).
        Theme::ArcEnCiel => Ambiance {
            ciel: [ARDOISE_NUIT, ARDOISE, ARDOISE_CLAIRE, ARDOISE_PALE],
            lointain: BRUME,
            proche: ARDOISE_CLAIRE,
            // Sans pluie (`Theme::pluie`) : couleurs jamais dessinées, mais
            // visibles sur chaque bande si un jour il pleuvait.
            pluie: (PLUIE, BLANC_CASSE),
            etoiles: None,
            gouttes: 0,
            descente: 0,
            plafond: None,
            soleil: false,
            forme: Forme::Nuages,
            relief: 0,
            socle: true,
            fenetres: None,
            ilots: None,
            tuiles: TeinteTuiles {
                eclaircissement: 2,
                briques_orangees: false,
                briques_mouillees: false,
            },
            friable_nuage: true,
            vertige: 1.0,
            arc: true,
            voyage: false,
            details: [None; 4],
            vent: VENT_NUIT,
        },
        // La descente (carte 5.17, l'épilogue) : le ciel refait le voyage en sens
        // inverse, de l'or du soleil (en haut du niveau) à la nuit des toits (en bas),
        // voir `VOYAGE` et `dessiner_voyage`. Les champs ci-dessous décrivent le BAS
        // de la descente, là où l'on arrive : le ciel et la ville du premier niveau
        // (mêmes bandes, mêmes immeubles), avec quelques fenêtres allumées : on rentre
        // à la maison. Plus de pluie : l'orage est fini.
        Theme::Descente => Ambiance {
            ciel: [ARDOISE_OMBRE, ARDOISE_NUIT, ARDOISE, ARDOISE],
            lointain: ARDOISE_NUIT,
            proche: ARDOISE_OMBRE,
            // Sans pluie (`Theme::pluie`) : couleurs jamais dessinées, celles des toits.
            pluie: (PLUIE_SOMBRE, PLUIE),
            etoiles: Some(H / 2),
            gouttes: 0,
            descente: DESCENTE_IMMEUBLES,
            plafond: None,
            // Le soleil du haut de la descente est dessiné par `dessiner_voyage`.
            soleil: false,
            forme: Forme::Immeubles,
            relief: 0,
            socle: true,
            // Comme au clocher : l'or est réservé aux gouttes et aux éclairs.
            fenetres: Some((ORANGE_SOLEIL, CORAIL_CLAIR)),
            // Les îlots du milieu de la descente sont dessinés par `dessiner_voyage`
            // (devant la bande corail), pas ici (ils seraient devant la nuit).
            ilots: None,
            tuiles: TeinteTuiles {
                eclaircissement: 0,
                briques_orangees: false,
                briques_mouillees: false,
            },
            friable_nuage: false,
            vertige: 1.0,
            arc: false,
            voyage: true,
            details: [None; 4],
            vent: VENT_NUIT,
        },
        // MONDE 2, « Là où va la pluie » (carte 9.5) : le parapluie retourné devient
        // une barque. Le ciel reste dans la nuit bleue du Monde 1 (la pluie n'a pas
        // fini) et s'éclaircit doucement jusqu'à l'aube du port et de la marée.
        // La palette tient déjà : les deux cyans (`PLUIE`, `PLUIE_SOMBRE`) font
        // l'eau, les silhouettes et les cristaux.
        //
        // Le caniveau (refait à la carte 9.6b) : la rue sous une forte averse, vue
        // du trottoir. Au loin, les immeubles de la ville (ardoise d'ombre, quelques
        // fenêtres orange) sur un ciel de nuit ; devant eux, une rangée de maisons
        // (`Detail::Facades` : façades d'ardoise sombres et ternes depuis la carte
        // 9.20g, fenêtres, vitrines
        // sous leur store, descentes de gouttière) ; devant, les lampadaires et leur
        // halo. La couche proche reste sous la rue (cachée par les pavés).
        Theme::Caniveau => Ambiance {
            ciel: [ARDOISE_NUIT, ARDOISE_NUIT, ARDOISE, ARDOISE],
            lointain: ARDOISE_OMBRE,
            proche: ARDOISE_OMBRE,
            pluie: (PLUIE_SOMBRE, PLUIE),
            etoiles: None,
            gouttes: 56,
            descente: DESCENTE_IMMEUBLES,
            plafond: None,
            soleil: false,
            forme: Forme::Immeubles,
            relief: 30,
            socle: true,
            // Carte 9.20g : le fond terne, les fenêtres lointaines faiblement
            // allumées (violet d'orage) ; l'orange vif reste au premier plan.
            fenetres: Some((VIOLET_ORAGE, VIOLET_ORAGE)),
            ilots: None,
            // Briques mouillées : les murs violets du dessin virent au cyan sombre.
            tuiles: TeinteTuiles {
                eclaircissement: 1,
                briques_orangees: false,
                briques_mouillees: true,
            },
            friable_nuage: false,
            vertige: 1.0,
            arc: false,
            voyage: false,
            details: [Some(Detail::Facades), Some(Detail::Lampadaires), None, None],
            vent: VENT_NUIT,
        },
        // Les égouts : tunnel de briques sombres, des voûtes au fond (les bouches
        // des galeries voisines, carte 9.27 : plus de masses de `Roche`, qui se
        // lisaient comme des toits et des cheminées), un plafond bas (l'arche du
        // tunnel), deux tuyaux et leur grille au fond, la lueur cyan de l'eau
        // stagnante au sol. Sous terre, il ne pleut pas (carte 9.27 :
        // `Theme::pluie` est faux, densité 0) : seuls tombent les filets d'eau des
        // grilles du plafond (`decor_egouts`). Le ciel du tunnel s'éclaircit vers
        // le bas (la lueur de l'eau), mais jamais à la couleur d'une paroi : la
        // lointaine (ardoise nuit) et la proche (brique violette) tranchent sur
        // les quatre bandes.
        Theme::Egouts => Ambiance {
            ciel: [ARDOISE_OMBRE, ARDOISE_NUIT, ARDOISE_OMBRE, ARDOISE],
            lointain: ARDOISE_NUIT,
            proche: VIOLET_ORAGE,
            pluie: (PLUIE_SOMBRE, PLUIE),
            etoiles: None,
            gouttes: 0,
            descente: DESCENTE_IMMEUBLES,
            plafond: Some(Plafond {
                loin: ARDOISE_NUIT,
                proche: VIOLET_ORAGE,
                ampleur: 2,
                stalactites: false,
            }),
            soleil: false,
            forme: Forme::Voutes,
            relief: 0,
            socle: true,
            fenetres: None,
            ilots: None,
            tuiles: TeinteTuiles {
                eclaircissement: 0,
                briques_orangees: false,
                briques_mouillees: true,
            },
            friable_nuage: false,
            vertige: 1.0,
            arc: false,
            voyage: false,
            details: [Some(Detail::Tuyaux), Some(Detail::Lueur), None, None],
            vent: VENT_NUIT,
        },
        // La grande cascade : niveau vertical (`vertige`) entre les chutes d'eau ;
        // falaises de fond et ruisselles devant, deux nappes de brume, ciel qui
        // s'éclaircit vers l'horizon (la lumière qui remonte de la brèche).
        Theme::Cascade => Ambiance {
            ciel: [ARDOISE_NUIT, VIOLET_ORAGE, BRUME, NUAGE],
            lointain: ARDOISE,
            proche: ARDOISE_NUIT,
            pluie: (PLUIE, BLANC_CASSE),
            etoiles: None,
            gouttes: 24,
            descente: DESCENTE_IMMEUBLES,
            plafond: None,
            soleil: false,
            // Falaises hautes : le relief remonte la couche lointaine.
            forme: Forme::Roche,
            relief: 26,
            socle: true,
            fenetres: None,
            ilots: None,
            tuiles: TeinteTuiles {
                eclaircissement: 2,
                briques_orangees: false,
                briques_mouillees: false,
            },
            friable_nuage: false,
            // Niveau vertical : comme le clocher, la falaise s'enfonce en montant.
            vertige: 2.5,
            arc: false,
            voyage: false,
            details: [Some(Detail::Chutes), Some(Detail::Brume), None, None],
            vent: VENT_NUIT,
        },
        // La grotte aux bulles : nuit bleue profonde, stalactites au plafond (les
        // seules, carte 9.5) et des cristaux cyan au pied. Niveau vertical
        // (`vertige`) : l'ascension se fait en bulles. Carte 9.20d : plus de
        // bulles au fond, sorties de nulle part ; les vraies jaillissent de leurs
        // sources, au premier plan (`decor_grotte`).
        Theme::Grotte => Ambiance {
            ciel: [ARDOISE_OMBRE, ARDOISE_OMBRE, ARDOISE_NUIT, ARDOISE_NUIT],
            lointain: ARDOISE,
            proche: ARDOISE_OMBRE,
            // La grotte est sèche (la pluie est restée dehors) : `Theme::pluie` est
            // faux et la densité vaut 0. Couleurs jamais dessinées, mais visibles
            // sur chaque bande si un jour il pleuvait.
            pluie: (PLUIE_SOMBRE, PLUIE),
            etoiles: None,
            gouttes: 0,
            descente: DESCENTE_IMMEUBLES,
            plafond: Some(Plafond {
                loin: ARDOISE_NUIT,
                proche: ARDOISE,
                ampleur: 2,
                stalactites: true,
            }),
            soleil: false,
            // Carte 9.27 : des stalagmites (pointes) au lieu de masses de `Roche`
            // au sommet plat, qui se lisaient comme des toits.
            forme: Forme::Stalagmites,
            relief: 10,
            socle: true,
            fenetres: None,
            ilots: None,
            tuiles: TeinteTuiles {
                eclaircissement: 0,
                briques_orangees: false,
                briques_mouillees: false,
            },
            friable_nuage: false,
            vertige: 2.5,
            arc: false,
            voyage: false,
            details: [Some(Detail::Cristaux), None, None, None],
            vent: VENT_NUIT,
        },
        // La rivière : berges de fond et roseaux devant, ciel grisé qui
        // s'éclaircit vers l'horizon ; l'eau du courant au pied, ses reflets.
        // Carte 9.10 : une forêt de sapins à mi-hauteur, au-dessus des berges.
        Theme::Riviere => Ambiance {
            ciel: [ARDOISE, ARDOISE_CLAIRE, BRUME, NUAGE],
            lointain: ARDOISE_CLAIRE,
            proche: ARDOISE_OMBRE,
            pluie: (PLUIE_SOMBRE, PLUIE),
            etoiles: None,
            gouttes: 24,
            descente: DESCENTE_IMMEUBLES,
            plafond: None,
            soleil: false,
            // Devant : des roseaux (tiges fines), pas des silhouettes hautes.
            forme: Forme::Roseaux,
            relief: 0,
            socle: true,
            fenetres: None,
            ilots: None,
            tuiles: TeinteTuiles {
                eclaircissement: 2,
                briques_orangees: false,
                briques_mouillees: false,
            },
            friable_nuage: false,
            vertige: 1.0,
            arc: false,
            voyage: false,
            details: [Some(Detail::Foret), Some(Detail::EauRiviere), None, None],
            vent: VENT_NUIT,
        },
        // Le port : quais et coques de bateaux, grues au premier plan, mouettes au
        // loin ; l'eau sombre du port au pied, une écume claire au bord. Le ciel
        // s'éclaircit encore : la nuit tire sur la brume.
        Theme::Port => Ambiance {
            ciel: [ARDOISE_NUIT, ARDOISE_NUIT, ARDOISE, ARDOISE_CLAIRE],
            lointain: ARDOISE_PALE,
            proche: ARDOISE_OMBRE,
            pluie: (PLUIE_SOMBRE, PLUIE),
            etoiles: None,
            gouttes: 16,
            descente: DESCENTE_IMMEUBLES,
            plafond: None,
            soleil: false,
            // Les quais : des coques de bateaux amarrés au premier plan.
            forme: Forme::Coques,
            relief: 0,
            socle: true,
            fenetres: None,
            ilots: None,
            // Les quais ont leur jeu de tuiles (carte 9.6c) : la teinte ne touche
            // plus que l'eau, gardée sombre et lisible comme au caniveau.
            tuiles: TeinteTuiles {
                eclaircissement: 1,
                briques_orangees: false,
                briques_mouillees: true,
            },
            friable_nuage: false,
            vertige: 1.0,
            arc: false,
            voyage: false,
            details: [
                Some(Detail::EauPort),
                Some(Detail::Grues),
                Some(Detail::Mouettes),
                Some(Detail::Phare),
            ],
            vent: VENT_NUIT,
        },
        // La marée (bonus) : le mur de blocs du port, l'eau sombre qui monte au
        // bas de l'écran, un ciel d'aube qui s'empourpre vers l'horizon. Niveau
        // vertical (`vertige`) : la course vers le haut, la poursuite de la carte
        // 8.2 à la verticale.
        Theme::Maree => Ambiance {
            ciel: [ARDOISE_OMBRE, ARDOISE_NUIT, VIOLET_ORAGE, CORAIL_CLAIR],
            lointain: ARDOISE,
            proche: ARDOISE_NUIT,
            pluie: (PLUIE_SOMBRE, PLUIE),
            etoiles: None,
            gouttes: 40,
            descente: DESCENTE_IMMEUBLES,
            plafond: None,
            soleil: false,
            forme: Forme::Roche,
            relief: 18,
            socle: true,
            fenetres: None,
            ilots: None,
            tuiles: TeinteTuiles {
                eclaircissement: 2,
                briques_orangees: false,
                briques_mouillees: false,
            },
            friable_nuage: false,
            vertige: 2.5,
            arc: false,
            voyage: false,
            details: [Some(Detail::EauMaree), None, None, None],
            vent: VENT_NUIT,
        },
    }
}

/// Le ciel du voyage de retour (carte 5.17, « La descente ») : dix bandes de H/4 =
/// 45 px, du haut du niveau vers le bas. On part de l'or du soleil (le 9e niveau),
/// on passe par le corail du couchant (la mer de nuages, les vents) et le violet de
/// l'orage (le clocher, le cyclone), et l'on arrive à la nuit des toits : les quatre
/// dernières bandes sont EXACTEMENT le ciel du premier niveau. Chaque couleur est
/// déjà une bande du ciel d'un autre niveau (test
/// `chaque_couleur_du_voyage_est_deja_le_ciel_d_un_autre_niveau`) : tout ce qui se
/// lit sur les autres ciels (sprites, grêle) se lit aussi ici.
pub const VOYAGE: [Couleur; 10] = [
    OR_PALE,
    OR,
    ORANGE_SOLEIL,
    CORAIL_CLAIR,
    CORAIL,
    VIOLET_ORAGE,
    ARDOISE_OMBRE,
    ARDOISE_NUIT,
    ARDOISE,
    ARDOISE,
];
/// Hauteur (px) du ciel du voyage, et remontée maximale de son « fenêtrage » : en
/// bas du niveau, l'écran montre les 4 dernières bandes ; en haut, les 4 premières.
const HAUTEUR_VOYAGE: i32 = VOYAGE.len() as i32 * (H / 4);
pub const REMONTEE_VOYAGE_MAX: i32 = HAUTEUR_VOYAGE - H;
/// Parallaxe verticale du voyage : une descente de quatre écrans (540 px de caméra)
/// fait défiler les 270 px du voyage. Le ciel, le soleil, les îlots, les étoiles et
/// la ville bougent ENSEMBLE : chaque élément reste devant la même bande de ciel.
const FACTEUR_VOYAGE: f32 = 0.5;
/// Soleil du haut de la descente (repère du voyage : 0 = haut de la 1re bande),
/// dans la bande d'or (45 à 89), à l'abscisse du soleil du 9e niveau.
const SOLEIL_VOYAGE_Y: i32 = 70;
/// Îlots du milieu de la descente (repère du voyage) : leur haut le plus haut, devant
/// la bande corail (180 à 224 : 185 + 20 + 8 = 213 au plus bas).
const ILOTS_VOYAGE_Y: i32 = 185;
/// Couleurs des îlots du voyage (corps, dessous) : celles des îlots des vents
/// d'altitude, qui se lisent déjà sur le corail.
const ILOTS_VOYAGE: (Couleur, Couleur) = (VIOLET_ORAGE, ARDOISE_NUIT);

/// Remontée du fond du voyage (px, de 0 en bas du niveau à `REMONTEE_VOYAGE_MAX` en
/// haut d'un niveau de quatre écrans) pour une caméra du fond `cam_y_fond`
/// (`camera_fond` : 0 tout en bas, négative en montant). Entière, bornée. Pure.
pub fn remontee_voyage(cam_y_fond: i32) -> i32 {
    decalage(-cam_y_fond, FACTEUR_VOYAGE).clamp(0, REMONTEE_VOYAGE_MAX)
}

/// Bande de `VOYAGE` derrière la ligne d'écran `y` quand le fond est remonté de
/// `remontee` px (sans la trame des 6 dernières lignes de chaque bande) : la règle
/// que suit `dessiner_voyage`, pour les tests. Pure.
#[cfg(test)]
pub fn bande_voyage(y: i32, remontee: i32) -> usize {
    let s = y + REMONTEE_VOYAGE_MAX - remontee;
    (s.max(0) / (H / 4)).min(VOYAGE.len() as i32 - 1) as usize
}

/// Ordonnée écran d'un point du repère du voyage (0 = haut de la 1re bande).
fn y_voyage(y: i32, remontee: i32) -> i32 {
    y - REMONTEE_VOYAGE_MAX + remontee
}

/// Le fond de la descente : le ciel du voyage (dix bandes séparées par la même
/// trame que `dessiner_ciel`), le soleil en haut, des îlots devant le corail, les
/// étoiles dans la nuit, et en bas la ville du premier niveau. Tout défile de la
/// même remontée : en bas du niveau (et sur l'écran de fin), l'écran est celui du
/// premier niveau, fenêtres allumées en plus.
fn dessiner_voyage(a: &Ambiance, cam_x: i32, cam_y: i32, pas: u32) {
    let r = remontee_voyage(cam_y);
    let bande = H / 4;
    for (i, &c) in VOYAGE.iter().enumerate() {
        let y = y_voyage(i as i32 * bande, r);
        if y >= H || y + bande < 0 {
            continue;
        }
        rect(0, y, L, bande + 1, c);
        if let Some(&suivante) = VOYAGE.get(i + 1) {
            for k in 0..3 {
                rect(0, y + bande - 6 + k * 2 + 1, L, 1, suivante);
            }
        }
    }
    let soleil = y_voyage(SOLEIL_VOYAGE_Y, r);
    if soleil > -30 {
        dessiner_soleil_en(cam_x, soleil);
    }
    let ilots = y_voyage(ILOTS_VOYAGE_Y, r);
    if ilots > -30 && ilots < H {
        dessiner_ilots(ILOTS_VOYAGE.0, ILOTS_VOYAGE.1, cam_x, ilots - ILOT_Y_MIN);
    }
    // Étoiles et ville : celles du premier niveau, descendues de la remontée.
    if let Some(ymax) = a.etoiles {
        dessiner_etoiles_en(pas, cam_x, ymax, r);
    }
    let (fen_loin, fen_proche) = match a.fenetres {
        Some((l, p)) => (Some(l), Some(p)),
        None => (None, None),
    };
    if BASE_LOINTAINE + r - 80 < H {
        dessiner_couche(
            a,
            cam_x,
            BASE_LOINTAINE + r,
            FACTEUR_LOINTAINE,
            a.lointain,
            GRAINE_LOINTAINE,
            a.relief,
            fen_loin,
        );
        dessiner_couche(
            a,
            cam_x,
            BASE_PROCHE + a.descente + r,
            FACTEUR_PROCHE,
            a.proche,
            GRAINE_PROCHE,
            0,
            fen_proche,
        );
    }
}

/// Les huit bandes de l'arc-en-ciel, de l'extérieur (rouge : le corail) vers
/// l'intérieur (violet). La palette n'a pas de vert : le cyan de la pluie et son
/// cyan sombre en tiennent lieu, entre le jaune (or) et le violet.
pub const ARC: [Couleur; 8] = [
    CORAIL,
    CORAIL_CLAIR,
    ORANGE_SOLEIL,
    OR,
    OR_PALE,
    PLUIE,
    PLUIE_SOMBRE,
    VIOLET_ORAGE,
];
/// Épaisseur (px) d'une bande de l'arc.
const BANDE_ARC: i32 = 3;
/// Rayon EXTÉRIEUR de l'arc (px) et ordonnée de son centre, sous le bas de
/// l'écran : le sommet est en y = 36, dans la bande de ciel la plus sombre, et
/// les deux pieds plongent derrière les bancs de nuages du bas.
const RAYON_ARC: i32 = 150;
const CENTRE_ARC_Y: i32 = 186;
/// Parallaxe de l'arc, et abscisse de son centre quand la caméra est à 0 : il
/// glisse d'un écran vers la gauche sur toute la longueur du bonus (240 tuiles,
/// 3 520 px de caméra : 352 px). Au départ, son pied gauche se lève devant Pépin ;
/// au milieu du pont, on voit l'arc entier ; à l'arrivée, son pied droit, derrière.
const FACTEUR_ARC: f32 = 0.10;
const CENTRE_ARC_X0: i32 = 336;

/// Abscisse écran du centre de l'arc pour la caméra `cam_x`. Pure.
pub fn centre_arc(cam_x: i32) -> i32 {
    CENTRE_ARC_X0 - decalage(cam_x, FACTEUR_ARC)
}

/// Ordonnées `[haut, bas)` de la bande `k` de l'arc (0 : l'extérieure) dans la
/// colonne de pixels `x`, pour un arc centré en `cx` ; `None` si la bande ne
/// passe pas par cette colonne. Fonction PURE (testée sans fenêtre).
///
/// Chaque bande est la couronne entre deux cercles ; pour une colonne, on calcule
/// où le cercle extérieur et le cercle intérieur la coupent (au centre du pixel)
/// et on ARRONDIT à l'entier : chaque pixel a une seule couleur, sans dégradé ni
/// lissage, et la bande `k + 1` commence exactement là où finit la bande `k`.
/// Sous le centre (bas de l'écran), une colonne qui ne coupe pas le cercle
/// intérieur descend jusqu'au centre de l'arc.
pub fn bande_arc(x: i32, cx: i32, k: usize) -> Option<(i32, i32)> {
    let dx = (x as f32 + 0.5 - cx as f32).abs();
    // Ordonnée où le cercle de rayon `r` coupe la colonne (moitié haute), ou le
    // centre si la colonne est en dehors du cercle.
    let coupe = |r: i32| -> i32 {
        let r = r as f32;
        if dx < r {
            (CENTRE_ARC_Y as f32 - (r * r - dx * dx).sqrt()).round() as i32
        } else {
            CENTRE_ARC_Y
        }
    };
    let exterieur = RAYON_ARC - BANDE_ARC * k as i32;
    let (haut, bas) = (coupe(exterieur), coupe(exterieur - BANDE_ARC));
    (haut < bas).then_some((haut, bas))
}

/// L'arc-en-ciel : pour chaque bande, des colonnes de 1 px de large, regroupées
/// quand plusieurs colonnes voisines ont exactement la même hauteur (le sommet
/// de l'arc) : moins de rectangles, même image.
fn dessiner_arc(cam_x: i32) {
    let cx = centre_arc(cam_x);
    for (k, &couleur) in ARC.iter().enumerate() {
        let mut x = 0;
        while x < L {
            let Some((haut, bas)) = bande_arc(x, cx, k) else {
                x += 1;
                continue;
            };
            let mut l = 1;
            while x + l < L && bande_arc(x + l, cx, k) == Some((haut, bas)) {
                l += 1;
            }
            let (haut, bas) = (haut.max(0), bas.min(H));
            if haut < bas {
                rect(x, haut, l, bas - haut, couleur);
            }
            x += l;
        }
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

/// Hauteur des voûtes du fond des égouts (`Forme::Voutes`, carte 9.27), sans relief.
const HAUTEUR_VOUTES: i32 = 44;
/// Épaisseur du linteau, au-dessus de l'arc d'une voûte.
const VOUTE_LINTEAU: i32 = 6;
/// Rayon de l'arc d'une voûte : demi-largeur de l'ouverture entre les piliers
/// (24 - 18 : des piliers de 6 px de chaque côté de la cellule, 12 px à deux).
const VOUTE_RAYON: i32 = 18;

/// Demi-largeur de l'ouverture d'une voûte, `dy` px sous son sommet (0 : rangée
/// pleine, le linteau). Un plein cintre arrondi au pair, puis les piliers droits.
pub(crate) fn ouverture_voute(dy: i32) -> i32 {
    let centre = VOUTE_LINTEAU + VOUTE_RAYON;
    if dy < VOUTE_LINTEAU {
        0
    } else if dy >= centre {
        VOUTE_RAYON
    } else {
        let k = centre - dy;
        let w = ((VOUTE_RAYON * VOUTE_RAYON - k * k) as f32).sqrt() as i32;
        w & !1
    }
}

/// Largeur (paire, de 20 à 2 px) d'une stalagmite de `hauteur` px, `dy` px au-dessus
/// de son pied (`Forme::Stalagmites`, carte 9.27) : un cône qui finit en pointe.
pub(crate) fn largeur_stalagmite(dy: i32, hauteur: i32) -> i32 {
    (2 + 18 * (hauteur - dy).max(0) / hauteur.max(1)) & !1
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
        // Falaise ou mur de blocs : une masse de 24 à 58 px (Monde 2, carte 9.5).
        Forme::Roche => 24 + (h % 35) + relief,
        // Roseaux : des tiges fines de 24 à 47 px (Monde 2, carte 9.5).
        Forme::Roseaux => 24 + (h % 24) + relief,
        // Coques de bateaux : la coque (12 px) et le mât, de 30 à 40 px au total.
        Forme::Coques => 30 + (h % 11) + relief,
        // Voûtes : une arcade de hauteur constante (44 px), comme un mur bâti.
        Forme::Voutes => HAUTEUR_VOUTES + relief,
        // Stalagmites : la plus haute des deux pointes, de 24 à 58 px.
        Forme::Stalagmites => 24 + (h % 35) + relief,
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
/// dérive vers la gauche pour 2 de chute) et boucle sur l'écran. Tests seulement :
/// le dessin passe par `goutte_penchee` (carte 8.8, pluie normale ou forte).
#[cfg(test)]
pub fn goutte_de_pluie(i: i32, pas: u32, vitesse: i32, graine: u32) -> (i32, i32) {
    goutte_penchee(i, pas, vitesse, graine, 2)
}

/// Comme `goutte_de_pluie`, avec `pente` px de chute pour 1 px de dérive vers la
/// gauche (2 : la pluie de tous les jours ; 1 : la pluie forte, à 45°).
fn goutte_penchee(i: i32, pas: u32, vitesse: i32, graine: u32, pente: i32) -> (i32, i32) {
    let h = hache(i, graine);
    let x0 = (h % L as u32) as i32;
    let y0 = ((h >> 9) % H as u32) as i32;
    let t = pas as i32 * vitesse;
    ((x0 - t / pente).rem_euclid(L), (y0 + t).rem_euclid(H))
}

/// Pluie FORTE de la nouvelle partie + (carte 8.8) : 1,5 fois plus de gouttes par
/// couche (`PLUIE_FORTE_NUM / PLUIE_FORTE_DEN`), et plus penchée (45°, contre une
/// pente de 2 d'habitude). Dessin seulement : elle ne touche personne.
pub const PLUIE_FORTE_NUM: i32 = 3;
pub const PLUIE_FORTE_DEN: i32 = 2;

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
    dessiner_etoiles_en(pas, cam_x, ymax, 0);
}

/// Les étoiles, descendues de `dy` px (le voyage de la descente) : semées sur
/// `[dy, dy + ymax)`.
fn dessiner_etoiles_en(pas: u32, cam_x: i32, ymax: i32, dy: i32) {
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
        if !eteinte && y + dy < H {
            rect(x, y + dy, 1, 1, couleur);
        }
    }
}

fn dessiner_soleil(cam_x: i32) {
    dessiner_soleil_en(cam_x, 44);
}

/// Le soleil, centré à l'ordonnée écran `cy`.
fn dessiner_soleil_en(cam_x: i32, cy: i32) {
    let (cx, r) = (236 - decalage(cam_x, 0.02) % 40, 22);
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
            Forme::Roche => {
                // Falaise ou mur de blocs (Monde 2) : une masse en trois assises
                // (le sommet plus étroit), coupée par des joints : le ciel passe
                // à travers les coutures, sans couleur hors palette.
                let s = h / 3;
                let j1 = x + l / 3 + (hache(cellule * 7, graine) % 4) as i32;
                let j2 = x + 2 * l / 3 + (hache(cellule * 7 + 1, graine) % 4) as i32;
                rect(x, base - s, j1 - x, s, couleur);
                rect(j1 + 2, base - s, j2 - j1 - 2, s, couleur);
                rect(j2 + 2, base - s, x + l - j2 - 2, s, couleur);
                let jm = x + 2 + (l - 4) / 2 + (hache(cellule * 7 + 2, graine) % 4) as i32;
                rect(x + 2, base - 2 * s, jm - x - 2, s, couleur);
                rect(jm + 2, base - 2 * s, x + l - 2 - jm - 2, s, couleur);
                rect(x + 5, base - h, l - 10, s + 1, couleur);
                rect(x + 8, base - h - 2, l - 16, 2, couleur);
            }
            Forme::Roseaux => {
                // Roseaux (Monde 2) : trois tiges fines de hauteurs inégales,
                // chacune coiffée d'une touffe (la graine les décale).
                for k in 0..3 {
                    let hx = x + 3 + k * 4 + (hache(cellule * 13 + k, graine) % 6) as i32;
                    let th = 10 + ((hache(cellule * 17 + k, graine) % 3) as i32) * 5;
                    rect(hx, y + h - th, 1, th, couleur);
                    rect(hx - 1, y + h - th - 3, 3, 3, couleur);
                }
            }
            Forme::Coques => {
                // Coque de bateau amarré (Monde 2) : la coque au fond creusé,
                // le mât nu qui porte la voile (la voile attendra les mécaniques).
                let coque = 12;
                rect(x, y + h - coque, l, coque, couleur);
                rect(x + 2, y + h - 2, l - 4, 2, couleur);
                rect(x + 4, y + h, l - 8, 2, couleur);
                let mx = x + l / 2 + (hache(cellule, graine) % 5) as i32;
                rect(mx, y - 2, 1, h - 2, couleur);
            }
            Forme::Voutes => {
                // Une bouche de galerie par cellule de 48 px (sans décalage : les
                // piliers voisins se touchent, l'arcade est continue) : un linteau,
                // un arc en plein cintre, deux piliers ; l'ouverture laisse voir le
                // fond du tunnel.
                let x0 = cellule * 48 - dec;
                let haut = base - hauteur_silhouette(a, cellule, graine, relief);
                for dy in 0..base - haut {
                    let o = ouverture_voute(dy);
                    if o == 0 {
                        rect(x0, haut + dy, 48, 1, couleur);
                    } else {
                        rect(x0, haut + dy, 24 - o, 1, couleur);
                        rect(x0 + 24 + o, haut + dy, 24 - o, 1, couleur);
                    }
                }
            }
            Forme::Stalagmites => {
                // Deux pointes de roche par cellule, en cônes : 20 px à la base,
                // 2 px à la pointe, une marche toutes les 2 rangées ; la seconde
                // aux deux tiers de la première.
                let hp = hauteur_silhouette(a, cellule, graine, relief);
                for k in 0..2 {
                    let hk = if k == 0 { hp } else { hp * 2 / 3 };
                    let cx = x + 10 + k * l / 2 + (hache(cellule * 5 + k, graine) % 8) as i32;
                    for dy in (0..hk).step_by(2) {
                        let hh = 2.min(hk - dy);
                        rect(
                            cx - largeur_stalagmite(dy, hk) / 2,
                            base - dy - hh,
                            largeur_stalagmite(dy, hk),
                            hh,
                            couleur,
                        );
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
/// (trois étages de plus en plus étroits vers le bas), ou, dans la grotte
/// (Monde 2, carte 9.5), des stalactites : des pointes vers le bas, de
/// profondeurs inégales.
fn dessiner_plafond(cam_x: i32, facteur: f32, graine: u32, p: &Plafond, couleur: Couleur) {
    let dec = decalage(cam_x, facteur);
    let premiere = dec.div_euclid(CELLULE_PLAFOND) - 1;
    let derniere = (dec + L).div_euclid(CELLULE_PLAFOND) + 1;
    for cellule in premiere..=derniere {
        let (x, l, d) = nuage_plafond(cellule, graine, p.ampleur);
        let x = x - dec;
        if p.stalactites {
            // Deux stalactites par cellule : une pointe à la base large, amincie
            // en trois gradins vers le bas.
            for k in 0..2 {
                let h = hache(cellule * 5 + k, graine);
                let sx = x + k * l / 2 + (h % 10) as i32;
                let sd = d + ((h >> 3) % 6) as i32;
                rect(sx, 0, 4, sd - 3, couleur);
                rect(sx + 1, sd - 3, 2, 2, couleur);
                rect(sx + 2, sd - 1, 1, 1, couleur);
            }
        } else {
            rect(x, 0, l, d / 2, couleur);
            rect(x + l / 8, d / 2, l * 3 / 4, d / 4, couleur);
            rect(x + l / 3, d / 2 + d / 4, l / 3, d - d / 2 - d / 4, couleur);
        }
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

fn dessiner_pluie(a: &Ambiance, pas: u32, forte: bool) {
    let (gouttes, pente) = if forte {
        (a.gouttes * PLUIE_FORTE_NUM / PLUIE_FORTE_DEN, 1)
    } else {
        (a.gouttes, 2)
    };
    for (couche, (vitesse, couleur, longueur)) in [(2, a.pluie.0, 1), (3, a.pluie.1, 2)]
        .into_iter()
        .enumerate()
    {
        for i in 0..gouttes {
            let (x, y) = goutte_penchee(i, pas, vitesse, 100 + couche as u32, pente);
            // Un trait diagonal de 1 à 2 pixels, calé sur la grille (sa pente suit
            // celle de la chute).
            for k in 0..longueur + 1 {
                rect(x + k, y - k * pente, 1, 1, couleur);
            }
        }
    }
}

/// Motif de décor du Monde 2 (carte 9.5) : le dessin d'un motif (`Detail`) pour
/// une ambiance, en trois groupes de parallaxe : les motifs du fond (mouettes,
/// phare) restent fixés à l'écran (« à l'infini »), les motifs du milieu
/// (chutes, brume, tuyaux, lueur) suivent la couche des îlots, les motifs du
/// premier plan (eau, lampadaires, grues, cristaux) suivent la couche proche.
/// Pur : ne dépend que de `cam_x`, `cam_y` et `pas`.
fn dessiner_detail(d: Detail, a: &Ambiance, cam_x: i32, cam_y: i32, pas: u32) {
    let (facteur, vert) = match d {
        Detail::Mouettes => (0.0, 0),
        Detail::Chutes
        | Detail::Brume
        | Detail::Tuyaux
        | Detail::Lueur
        | Detail::Facades
        | Detail::Foret => (FACTEUR_ILOTS, enfoncement(cam_y, VERT_ILOTS * a.vertige)),
        _ => (FACTEUR_PROCHE, enfoncement(cam_y, VERT_PROCHE * a.vertige)),
    };
    let dec = decalage(cam_x, facteur);
    // Le « pied » du décor : le bas de l'écran, qui s'enfonce avec la parallaxe
    // verticale du groupe (nulle en bas d'un niveau d'un écran).
    let pied = H + vert;
    match d {
        // La rue du caniveau (carte 9.6b) : posée sur la ligne du sol (48 px
        // au-dessus du bas de l'écran, juste sous le dessus des pavés).
        Detail::Lampadaires => crate::decor_rue::dessiner_lampadaires(dec, pied - 48),
        Detail::Facades => crate::decor_rue::dessiner_facades(dec, pied - 48),
        Detail::Tuyaux => {
            // Deux tuyaux qui traversent le tunnel (segments en parallaxe),
            // leurs colliers, et une grille d'évacuation au fond. Carte 9.20b :
            // tuyaux sombres (ardoise, plus d'arête claire), pour qu'aucun ne se
            // lise comme une plate-forme.
            for (y, gy) in [(H / 3, 21), (2 * H / 3, 29)] {
                for seg in 0..3 {
                    let x0 = (seg * 120 + gy - dec).rem_euclid(L);
                    rect(x0, y, 80, 3, ARDOISE);
                    rect(x0, y + 3, 80, 1, ARDOISE_OMBRE);
                    rect(x0 + 20, y - 1, 4, 5, ARDOISE_NUIT);
                    rect(x0 + 60, y - 1, 4, 5, ARDOISE_NUIT);
                }
            }
            let gx = (200i32 - dec).rem_euclid(L);
            rect(gx, H / 3 + 10, 14, 22, ARDOISE_NUIT);
            for b in 0..4 {
                rect(gx + 2 + b * 3, H / 3 + 12, 1, 18, ARDOISE_CLAIRE);
            }
        }
        Detail::Lueur => {
            // L'eau stagnante du fond des égouts : une bande sombre et sa
            // lueur cyan, qui dérive lentement.
            rect(0, H - 10, L, 10, PLUIE_SOMBRE);
            for k in 0..4 {
                let x = (k * 83 + pas as i32 / 6).rem_euclid(L);
                rect(x, H - 8 + (k % 2) * 3, 6, 1, PLUIE);
            }
        }
        Detail::Chutes => {
            // Trois ruisselles qui tombent devant les falaises : le corps sombre,
            // le courant clair, des éclats qui dévalent, la mousse au pied. Carte
            // 9.20c : en ardoise, comme la grande chute du fond (`decor_cascade`) ;
            // le cyan vif est aux chutes jouables `|`, seules à pousser Pépin.
            for (i, cx) in [45i32, 150, 262].into_iter().enumerate() {
                let x = (cx - dec).rem_euclid(L);
                rect(x, 0, 5, H, ARDOISE);
                rect(x + 1, 0, 2, H, ARDOISE_CLAIRE);
                for k in 0..2 {
                    let y = (pas as i32 * 3 / 2 + k * 60 + i as i32 * 30).rem_euclid(H);
                    rect(x + 1 + k, y, 1, 8, ARDOISE_PALE);
                }
                rect(x - 2, H - 4, 9, 2, ARDOISE_PALE);
            }
        }
        Detail::Brume => {
            // Deux nappes de brume basse qui dérivent devant les falaises. En
            // lentille (bouts effilés, cœur en traits clairs décalés) : un
            // rectangle plein se lirait comme une plateforme (captures 95-*).
            for (y0, gy) in [(H / 3, 51), (2 * H / 3, 61)] {
                for seg in 0..3 {
                    let x0 = (seg * 120 + gy - dec + pas as i32 / 8).rem_euclid(L);
                    rect(x0 + 12, y0, 36, 2, BRUME);
                    rect(x0, y0 + 2, 60, 4, BRUME);
                    rect(x0 + 14, y0 + 6, 32, 2, BRUME);
                    rect(x0 + 8, y0 + 3, 18, 1, NUAGE);
                    rect(x0 + 30, y0 + 4, 22, 1, NUAGE);
                }
            }
        }
        Detail::Cristaux => {
            // Des cristaux cyan au pied de la grotte : des pointes en trois
            // gradins, un liseré clair sur l'arête.
            for k in 0..5 {
                let x = (k * 68 + 16 - dec).rem_euclid(L);
                let h = 12 + (k * 7) % 10;
                rect(x, pied - 4, 8, 4, PLUIE_SOMBRE);
                rect(x + 1, pied - 4 - h / 2, 6, h / 2, PLUIE_SOMBRE);
                rect(x + 3, pied - 4 - h, 2, h / 2, PLUIE_SOMBRE);
                rect(x + 3, pied - 4 - h, 1, h, PLUIE);
            }
        }
        Detail::Foret => {
            // Deux rangs de sapins : le rang du fond (ardoise claire), puis un
            // rang plus sombre, plus bas ; chaque sapin est un empilement de
            // gradins. Le pied de chaque rang descend jusqu'au bas de l'écran
            // (les berges du niveau le cachent).
            for (rang, (couleur, sommet, pas_x)) in [(ARDOISE_CLAIRE, 44, 13), (ARDOISE, 58, 17)]
                .into_iter()
                .enumerate()
            {
                let dec_rang = dec * (rang as i32 + 1) / 2;
                rect(0, sommet + 22, L, pied - sommet - 22, couleur);
                for k in 0..(L / pas_x + 2) {
                    let h = hache(k + rang as i32 * 97, 0x5A9) as i32;
                    let x = (k * pas_x - dec_rang).rem_euclid(L + pas_x) - pas_x / 2;
                    let haut = sommet + h % 9;
                    for g in 0..6 {
                        let demi = 1 + g;
                        rect(x - demi, haut + g * 4, 2 * demi + 1, 4, couleur);
                    }
                }
            }
        }
        Detail::EauRiviere => {
            // L'eau du courant au pied de l'écran : une bande sombre, sa surface
            // claire, des reflets en pointillés qui dérivent.
            rect(0, pied - 16, L, 16, PLUIE_SOMBRE);
            rect(0, pied - 16, L, 2, PLUIE);
            for k in 0..4 {
                let x = (k * 79 + pas as i32 / 5).rem_euclid(L);
                rect(x, pied - 13 + (k % 3) * 3, 1, 6, BLANC_CASSE);
            }
        }
        Detail::EauPort => {
            // L'eau du port : une bande plus haute, l'écume claire qui bouge au
            // bord.
            rect(0, pied - 24, L, 24, PLUIE_SOMBRE);
            rect(0, pied - 24, L, 2, PLUIE);
            for k in 0..6 {
                let x = (k * 57 + 30 + pas as i32 / 7).rem_euclid(L);
                rect(x, pied - 21, 5, 1, BLANC_CASSE);
            }
        }
        Detail::EauMaree => {
            // La marée : une bande d'eau sombre plus épaisse encore, l'écume qui
            // grimpe le long du mur (la course du bonus, carte 9.13).
            rect(0, pied - 36, L, 36, PLUIE_SOMBRE);
            rect(0, pied - 36, L, 2, PLUIE);
            for k in 0..7 {
                let x = (k * 49 + 12 + pas as i32 / 9).rem_euclid(L);
                rect(x, pied - 33 + (k % 2), 4, 1, BLANC_CASSE);
            }
        }
        Detail::Grues => {
            // Deux grues de port au premier plan : un mât, une flèche, un treuil
            // (des silhouettes claires sur la nuit : elles ne se lisent pas
            // comme des plateformes, trop fines et trop hautes).
            for (i, gx) in [58i32, 232].into_iter().enumerate() {
                let x = (gx - dec).rem_euclid(L);
                let haut = 34 + (i as i32) * 6;
                rect(x, pied - haut, 2, haut, ARDOISE_OMBRE);
                rect(x - 6, pied - haut, 18, 2, ARDOISE_OMBRE);
                rect(x + 8, pied - haut + 2, 1, 8 + (i as i32) * 4, ARDOISE_OMBRE);
            }
        }
        Detail::Mouettes => {
            // Trois mouettes qui traversent le ciel du port : un V de trois
            // pixels, qui dérivent lentement.
            for k in 0..3u32 {
                let x = (k as i32 * 97 + 20 - pas as i32 / 5).rem_euclid(L);
                let y = 30 + (k as i32) * 22 + ((pas / 20 + k) % 4) as i32 - 2;
                rect(x, y, 1, 1, BLANC_CASSE);
                rect(x - 2, y - 1, 1, 1, NUAGE);
                rect(x + 2, y - 1, 1, 1, NUAGE);
            }
        }
        Detail::Phare => {
            // Le phare au loin, fixé à l'écran comme les mouettes (« à
            // l'infini ») : une tour blanche à trois anneaux corail, sa galerie et
            // sa lanterne sombres. Le pied passe derrière les coques et l'eau
            // (dessinées après). La lampe s'allume une seconde sur deux (60 pas)
            // et lance deux traits de faisceau or pâle.
            let (x, haut) = (L - 64, 98);
            rect(x, haut + 8, 6, H - 20 - haut - 8, BLANC_CASSE);
            for k in 0..3 {
                rect(x, haut + 14 + k * 12, 6, 3, CORAIL);
            }
            rect(x - 1, haut + 6, 8, 2, ARDOISE_OMBRE);
            rect(x + 1, haut, 4, 6, ARDOISE_OMBRE);
            if (pas / 60).is_multiple_of(2) {
                rect(x + 2, haut + 2, 2, 3, OR);
                rect(x - 26, haut + 3, 24, 1, OR_PALE);
                rect(x + 8, haut + 3, 24, 1, OR_PALE);
            }
        }
    }
}

/// Dessine les motifs du Monde 2 rangés dans `a.details`, groupe par groupe :
/// `groupe` 0 = derrière les couches de silhouettes (le plus lointain : mouettes,
/// phare), `groupe` 1 = entre les deux couches (chutes, brume, tuyaux, lueur),
/// `groupe` 2 = devant la couche proche (eau, lampadaires, grues, cristaux).
fn dessiner_details(a: &Ambiance, groupe: u8, cam_x: i32, cam_y: i32, pas: u32) {
    for d in a.details.iter().copied().flatten() {
        let g = match d {
            Detail::Mouettes | Detail::Phare => 0,
            Detail::Chutes
            | Detail::Brume
            | Detail::Tuyaux
            | Detail::Lueur
            | Detail::Facades
            | Detail::Foret => 1,
            _ => 2,
        };
        if g == groupe {
            dessiner_detail(d, a, cam_x, cam_y, pas);
        }
    }
}

/// Dessine tout ce qui est DERRIÈRE le niveau (ciel, étoiles, soleil, silhouettes).
/// `cam_y` : ordonnée de la caméra vue par le fond (`camera_fond`).
pub fn dessiner_fond(theme: Theme, cam_x: i32, cam_y: i32, pas: u32) {
    let a = ambiance(theme);
    if a.voyage {
        dessiner_voyage(&a, cam_x, cam_y, pas);
        return;
    }
    dessiner_ciel(&a);
    if let Some(ymax) = a.etoiles {
        dessiner_etoiles(pas, cam_x, ymax);
    }
    if a.soleil {
        dessiner_soleil(cam_x);
    }
    if a.arc {
        dessiner_arc(cam_x);
    }
    // Motifs du Monde 2 (carte 9.5) : le groupe 0 (mouettes, phare) est le plus
    // lointain, derrière le plafond et les deux couches de silhouettes.
    dessiner_details(&a, 0, cam_x, cam_y, pas);
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
    // Motifs du Monde 2 (carte 9.5) : le groupe 1 (chutes, brume, tuyaux, lueur)
    // se glisse entre les deux couches de silhouettes.
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
    dessiner_details(&a, 1, cam_x, cam_y, pas);
    // Le groupe 2 (eau, lampadaires, grues, cristaux) est devant la couche proche.
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
    dessiner_details(&a, 2, cam_x, cam_y, pas);
}

/// La planche du Monde 2 (build de QA seulement, carte 9.5) : le plein écran
/// dessine les 7 nouveaux fonds, un par un (la pluie et les motifs animés, le nom
/// en haut de l'écran) pour les captures `docs/qa/95-*.png`. Un appui de la
/// touche V fait défiler vers le fond suivant (voir `main`).
#[cfg(feature = "debug_niveaux")]
pub fn planche_fonds_qa(k: usize, pas: u32) {
    use crate::texte::{self, Style};
    const NOMS: [&str; 7] = [
        "Le caniveau",
        "Les égouts",
        "La cascade",
        "La grotte",
        "La rivière",
        "Le port",
        "La marée",
    ];
    const THEMES: [Theme; 7] = [
        Theme::Caniveau,
        Theme::Egouts,
        Theme::Cascade,
        Theme::Grotte,
        Theme::Riviere,
        Theme::Port,
        Theme::Maree,
    ];
    let t = THEMES[k.min(6)];
    // Tous les fonds sont vus au PIED du niveau (caméra du fond à 0), même les
    // verticaux : au sommet, l'eau, les cristaux et la couche proche s'enfoncent
    // sous l'écran (le modèle du clocher, testé par
    // `au_sommet_du_clocher_la_ville_s_est_enfoncee`) et la planche les cacherait.
    let cam_y = 0;
    // Un léger défilement horizontal pour que l'on voie la parallaxe vivre.
    dessiner_fond(t, pas as i32 / 16, cam_y, pas);
    dessiner_avant(t, pas, false);
    texte::dessiner(
        NOMS[k.min(6)],
        8.0,
        8.0,
        Style::simple(BLANC_CASSE, 1).avec_ombre(ARDOISE_OMBRE),
    );
}

/// Dessine la pluie PAR-DESSUS le niveau (devant tout). `forte` : la pluie de la
/// nouvelle partie + (carte 8.8), plus dense et plus penchée.
pub fn dessiner_avant(theme: Theme, pas: u32, forte: bool) {
    if theme.pluie() {
        dessiner_pluie(&ambiance(theme), pas, forte);
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
            Theme::Toits | Theme::Orage | Theme::Clocher | Theme::Descente => {
                (0, false, false, false)
            }
            Theme::Gouttieres | Theme::Marche => (1, false, false, false),
            Theme::Cyclone => (0, false, false, true),
            Theme::Altitude | Theme::Mer | Theme::ArcEnCiel | Theme::Poursuite => {
                (2, false, false, true)
            }
            Theme::Soleil => (3, true, false, true),
            // Monde 2 (carte 9.5) : briques mouillées (cyan sombre) aux égouts
            // et au port ; les autres thèmes gardent leurs couleurs.
            Theme::Caniveau => (1, false, true, false),
            Theme::Egouts => (0, false, true, false),
            Theme::Cascade => (2, false, false, false),
            Theme::Grotte => (0, false, false, false),
            Theme::Riviere => (2, false, false, false),
            Theme::Port => (1, false, true, false),
            Theme::Maree => (2, false, false, false),
        };
        for t in Theme::TOUS {
            let a = ambiance(t);
            let lu = (
                a.tuiles.eclaircissement,
                a.tuiles.briques_orangees,
                a.tuiles.briques_mouillees,
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
            couleurs.extend([a.vent.traine, a.vent.tete]);
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

    /// Carte 5.16 : l'arc-en-ciel a huit bandes de 3 px JOINTIVES (la bande `k + 1`
    /// commence au pixel où finit la bande `k`, sans trou ni recouvrement), des
    /// couleurs toutes différentes, prises dans la palette, et son sommet est en
    /// y = 36, dans le ciel, au-dessus du pont (rangée 3 au plus haut : y = 48).
    #[test]
    fn l_arc_en_ciel_a_huit_bandes_jointives() {
        let distinctes: std::collections::HashSet<Couleur> = ARC.into_iter().collect();
        assert_eq!(distinctes.len(), 8);
        assert!(ARC.iter().all(|c| TOUTES.contains(c)));
        let cx = 160;
        assert_eq!(bande_arc(cx, cx, 0), Some((36, 39)));
        for x in -200..520 {
            for k in 0..ARC.len() - 1 {
                if let (Some((_, bas)), Some((haut, _))) =
                    (bande_arc(x, cx, k), bande_arc(x, cx, k + 1))
                {
                    assert_eq!(bas, haut, "x = {x}, bandes {k} et {}", k + 1);
                }
            }
            // Symétrique autour du centre (la colonne x et sa jumelle de l'autre côté).
            for k in 0..ARC.len() {
                assert_eq!(bande_arc(x, cx, k), bande_arc(2 * cx - 1 - x, cx, k));
            }
        }
        // Hors du grand cercle, aucune bande.
        assert!((0..ARC.len()).all(|k| bande_arc(cx + RAYON_ARC + 1, cx, k).is_none()));
    }

    /// L'arc glisse d'un écran de droite à gauche sur toute la longueur du bonus
    /// (caméra de 0 à 3 520 px) : au départ, son centre est à droite de l'écran ;
    /// à l'arrivée, à gauche.
    #[test]
    fn l_arc_glisse_d_un_ecran_sur_le_bonus() {
        assert_eq!(centre_arc(0), 336);
        assert_eq!(centre_arc(3520), -16);
        assert!(ambiance(Theme::ArcEnCiel).arc);
        assert!(Theme::TOUS
            .into_iter()
            .filter(|&t| t != Theme::ArcEnCiel)
            .all(|t| !ambiance(t).arc));
    }

    /// « Toute la palette » (demande d'AGA pour l'arc-en-ciel) : le ciel, les bancs
    /// de nuages, l'arc et les dalles du pont (nuages friables : contour, corps,
    /// reflets) montrent ensemble les 16 couleurs.
    #[test]
    fn l_arc_en_ciel_montre_toute_la_palette() {
        let a = ambiance(Theme::ArcEnCiel);
        let mut vues: Vec<Couleur> = a.ciel.to_vec();
        vues.extend([a.lointain, a.proche]);
        vues.extend(ARC);
        for ligne in crate::sprites::objets::FRIABLE0 {
            for c in ligne.chars() {
                if let Ok(Some(couleur)) = crate::sprites::pixel(c) {
                    vues.push(couleur);
                }
            }
        }
        for c in TOUTES {
            assert!(vues.contains(&c), "{c:?} absente de l'arc-en-ciel");
        }
    }

    #[test]
    fn le_ciel_s_eclaircit_de_l_orage_au_soleil() {
        let clarte = |t: Theme| {
            let c = ambiance(t).ciel[3];
            c.r as u32 + c.g as u32 + c.b as u32
        };
        assert!(clarte(Theme::Toits) < clarte(Theme::Gouttieres));
        assert!(clarte(Theme::Gouttieres) < clarte(Theme::Altitude));
        // La mer de nuages (niveau 8) entre les vents d'altitude et le soleil.
        assert!(clarte(Theme::Altitude) < clarte(Theme::Mer));
        assert!(clarte(Theme::Mer) < clarte(Theme::Soleil));
    }

    /// Luminance perçue d'une couleur (0 à 255), pour juger un contraste.
    fn luminance(c: Couleur) -> f32 {
        0.299 * f32::from(c.r) + 0.587 * f32::from(c.g) + 0.114 * f32::from(c.b)
    }

    /// Carte 5.15 : le vent latéral se voit. Sur CHAQUE bande du ciel de chaque
    /// thème, la queue ou la tête des traînées tranche nettement (au moins 50 de
    /// luminance d'écart) ; la tête est plus claire que la queue (c'est elle qui
    /// dit le sens). Dans la mer de nuages, où le vent fait le niveau, la QUEUE
    /// seule tranche déjà sur toutes les bandes et sur la mer de nuages du fond.
    #[test]
    fn le_vent_lateral_se_voit_sur_chaque_bande_de_ciel() {
        const ECART_MIN: f32 = 50.0;
        let ecart = |a: Couleur, b: Couleur| (luminance(a) - luminance(b)).abs();
        for t in Theme::TOUS {
            let a = ambiance(t);
            let v = a.vent;
            assert!(luminance(v.tete) > luminance(v.traine), "{t:?}");
            for bande in a.ciel {
                assert!(
                    ecart(v.traine, bande).max(ecart(v.tete, bande)) >= ECART_MIN,
                    "{t:?} : vent invisible sur {bande:?}"
                );
            }
        }
        let a = ambiance(Theme::Mer);
        for fond in a.ciel.into_iter().chain([a.lointain]) {
            assert!(
                ecart(a.vent.traine, fond) >= ECART_MIN,
                "mer de nuages : queue du vent invisible sur {fond:?}"
            );
        }
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
    /// enfoncée (la couche proche de plus de 100 px, la lointaine de plus de 50)
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
        // Les niveaux verticaux (le clocher, carte 5.9, et, dans le Monde 2,
        // carte 9.5, la cascade où l'on descend, la grotte aux ascensions en
        // bulles et la marée, course vers le haut) suivent le même modèle : leur fond
        // s'enfonce davantage en montant. Les autres gardent `vertige` à 1.
        for t in Theme::TOUS.into_iter().filter(|&t| {
            !matches!(
                t,
                Theme::Clocher | Theme::Cascade | Theme::Grotte | Theme::Maree
            )
        }) {
            assert!(ambiance(t).vertige == 1.0, "{t:?}");
        }
        for t in [Theme::Cascade, Theme::Grotte, Theme::Maree] {
            assert_eq!(
                ambiance(t).vertige,
                ambiance(Theme::Clocher).vertige,
                "{t:?}"
            );
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

    /// Carte 5.18 (QA des onze niveaux) : dans TOUS les thèmes à bancs de nuages
    /// (au-dessus des nuages, la mer de nuages, l'arc-en-ciel), la couche lointaine
    /// reste sous la ligne du sol : un sommet plat à la hauteur des plateformes, dans
    /// une trouée mortelle, se lirait comme un nuage où se poser. (Les îlots de la
    /// mer flottent au milieu du ciel, petits et d'une autre couleur que les nuages
    /// du niveau : ils ne sont pas concernés.)
    #[test]
    fn les_bancs_de_nuages_du_fond_restent_sous_la_ligne_du_sol() {
        let mut themes = Vec::new();
        for t in Theme::TOUS {
            let a = ambiance(t);
            if a.forme != Forme::Nuages {
                continue;
            }
            themes.push(t);
            let loin = zones(&a)
                .into_iter()
                .find(|z| z.nom == "lointaine")
                .unwrap();
            assert!(
                loin.haut >= LIGNE_SOL,
                "{t:?} : bancs du fond à y={}",
                loin.haut
            );
        }
        assert!(themes.contains(&Theme::Mer) && themes.contains(&Theme::Soleil));
        // Et, niveau par niveau (niveaux d'un écran de haut, où le fond ne bouge pas
        // en hauteur) : les bancs restent au moins 4 px sous le PLUS BAS des dessus
        // où l'on se pose (tuiles et plateformes : friables, nacelles...). La mer a
        // des quais à y = 144, plus bas que la ligne du sol (relief négatif).
        for infos in pepin_core::levels::NIVEAUX {
            let a = ambiance(infos.theme);
            let n = infos.charger().unwrap();
            if a.forme != Forme::Nuages || n.hauteur() > 11 {
                continue;
            }
            let t = pepin_core::aabb::TAILLE_TUILE;
            let tuiles = (0..n.largeur() as i32).filter_map(|tx| {
                (0..n.hauteur() as i32)
                    .find(|&ty| n.tuile_decor(tx, ty).est_solide())
                    .map(|ty| ty as f32 * t)
            });
            let entites = pepin_core::entities::solides_des_entites(&n.entites)
                .into_iter()
                .map(|b| b.y);
            let plus_bas = tuiles.chain(entites).fold(0.0f32, f32::max) as i32;
            let loin = zones(&a)[0];
            assert!(
                loin.haut >= plus_bas + 4,
                "{} : bancs du fond à y={}, quai le plus bas à y={plus_bas}",
                infos.nom,
                loin.haut
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

    /// Carte 9.27 (AGA : « dans les égouts il y a des cheminées et des décors de
    /// dehors ») : sous terre, ni pluie ni silhouette de dehors (immeubles,
    /// masses au sommet plat). Les voûtes forment une arcade continue (piliers de
    /// 6 px, ouverture en plein cintre) ; les stalagmites finissent en pointe.
    #[test]
    fn sous_terre_rien_de_dehors() {
        for t in [Theme::Egouts, Theme::Grotte] {
            let a = ambiance(t);
            assert!(!t.pluie() && a.gouttes == 0, "{t:?} : il pleut sous terre");
            assert!(
                !matches!(a.forme, Forme::Immeubles | Forme::Roche | Forme::Nuages),
                "{t:?} : silhouette de dehors {:?}",
                a.forme
            );
        }
        assert_eq!(ambiance(Theme::Egouts).forme, Forme::Voutes);
        assert_eq!(ambiance(Theme::Grotte).forme, Forme::Stalagmites);
        // La voûte : le linteau plein, l'arc qui s'ouvre sans jamais se refermer,
        // puis les piliers droits jusqu'en bas.
        let o: Vec<i32> = (0..HAUTEUR_VOUTES).map(ouverture_voute).collect();
        assert!(o[..VOUTE_LINTEAU as usize].iter().all(|&w| w == 0));
        assert!(o.windows(2).all(|p| p[0] <= p[1]), "{o:?}");
        assert_eq!(*o.last().unwrap(), VOUTE_RAYON);
        assert!(o.iter().all(|&w| w % 2 == 0 && w <= 24 - 6));
        // La stalagmite : 20 px au pied, 2 px à la pointe, jamais plus large en
        // montant (pas de sommet plat qui ferait un toit).
        for h in [16, 24, 40, 68] {
            let w: Vec<i32> = (0..h).map(|dy| largeur_stalagmite(dy, h)).collect();
            assert_eq!(w[0], 20, "{h}");
            assert_eq!(*w.last().unwrap(), 2, "{h}");
            assert!(w.windows(2).all(|p| p[0] >= p[1]), "{w:?}");
        }
    }

    /// I-20 : la pluie diffère d'un niveau à l'autre, reste sous le garde-fou,
    /// et ses deux couleurs ne se confondent avec aucune bande de ciel.
    #[test]
    fn la_pluie_a_une_densite_par_niveau_et_se_voit() {
        let n: Vec<i32> = Theme::TOUS.iter().map(|t| ambiance(*t).gouttes).collect();
        assert_eq!(
            n,
            [24, 48, 36, 60, 16, 32, 18, 0, 0, 0, 0, 0, 56, 0, 24, 0, 24, 16, 40]
        );
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
    /// de celle-ci, et il n'existe qu'aux gouttières, à l'orage, dans l'œil du
    /// cyclone (les nuages qui tournent autour de la trouée d'or) et, dans le
    /// Monde 2 (carte 9.5), aux égouts (l'arche du tunnel) et à la grotte
    /// (des stalactites à la place des dômes).
    #[test]
    fn le_plafond_de_nuages_tient_dans_la_bande_du_haut() {
        for t in Theme::TOUS {
            let a = ambiance(t);
            let attendu = matches!(
                t,
                Theme::Gouttieres | Theme::Orage | Theme::Cyclone | Theme::Egouts | Theme::Grotte
            );
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

    // --- Le voyage de la descente (carte 5.17) ------------------------------------

    /// Seule la descente a le ciel du voyage.
    #[test]
    fn seule_la_descente_refait_le_voyage() {
        for t in Theme::TOUS {
            assert_eq!(ambiance(t).voyage, t == Theme::Descente, "{t:?}");
        }
    }

    /// Le voyage va de l'or à la nuit : de plus en plus sombre, du haut jusqu'à la
    /// bande la plus noire (l'ombre), puis les bandes du ciel du premier niveau.
    #[test]
    fn le_voyage_va_de_l_or_a_la_nuit() {
        assert_eq!(VOYAGE[0], OR_PALE);
        for k in 0..6 {
            assert!(
                luminance(VOYAGE[k]) > luminance(VOYAGE[k + 1]),
                "bande {k} : {:?} puis {:?}",
                VOYAGE[k],
                VOYAGE[k + 1]
            );
        }
        // Le ciel de la descente (`ciel`, le bas du voyage) EST celui du premier niveau.
        assert_eq!(VOYAGE[6..], ambiance(Theme::Toits).ciel);
        assert_eq!(VOYAGE[6..], ambiance(Theme::Descente).ciel);
        assert_eq!(REMONTEE_VOYAGE_MAX, 6 * H / 4);
    }

    /// En bas du niveau (caméra du fond à 0, comme sur l'écran de fin), l'écran montre
    /// exactement le ciel du premier niveau ; en haut d'un niveau de quatre écrans
    /// (45 rangées), les quatre premières bandes, l'or. Entre les deux, la remontée
    /// est entière et suit la caméra (deux fois plus lentement).
    #[test]
    fn la_remontee_du_voyage_suit_la_hauteur_de_la_camera() {
        let toits = ambiance(Theme::Toits);
        let haut_niveau = 45 * 16;
        let fond = |cam_y: i32| camera_fond(cam_y, haut_niveau);
        // En bas : la caméra du fond vaut 0, la remontée aussi.
        assert_eq!(remontee_voyage(fond(haut_niveau - H)), 0);
        for y in 0..H {
            assert_eq!(VOYAGE[bande_voyage(y, 0)], ciel_a(&toits, y), "y = {y}");
        }
        // En haut : la remontée est maximale, l'écran montre les bandes 0 à 3.
        assert_eq!(remontee_voyage(fond(0)), REMONTEE_VOYAGE_MAX);
        for y in 0..H {
            assert_eq!(bande_voyage(y, REMONTEE_VOYAGE_MAX), (y / (H / 4)) as usize);
        }
        let mut avant = REMONTEE_VOYAGE_MAX;
        for cam_y in 0..=(haut_niveau - H) {
            let r = remontee_voyage(fond(cam_y));
            assert!(
                r <= avant && avant - r <= 1,
                "caméra {cam_y} : {avant} puis {r}"
            );
            assert_eq!(r, (((haut_niveau - H - cam_y) as f32) * 0.5).round() as i32);
            avant = r;
        }
        // Hors du niveau (secousse de mort) : bornée.
        assert_eq!(remontee_voyage(5), 0);
        assert_eq!(remontee_voyage(-10_000), REMONTEE_VOYAGE_MAX);
    }

    /// Chaque couleur du voyage est déjà une bande du ciel d'un autre niveau : les
    /// tests de contraste des sprites, de la grêle, des gouttes (faits sur les ciels
    /// de tous les thèmes) couvrent donc aussi la descente.
    #[test]
    fn chaque_couleur_du_voyage_est_deja_le_ciel_d_un_autre_niveau() {
        for c in VOYAGE {
            assert!(
                Theme::TOUS
                    .iter()
                    .filter(|&&t| t != Theme::Descente)
                    .any(|&t| ambiance(t).ciel.contains(&c)),
                "{c:?}"
            );
        }
    }

    /// Ce qui défile avec le voyage reste devant les bandes où il se voit : le
    /// soleil dans l'or (son disque pâle tranche sur l'or et l'orange), les îlots
    /// devant le corail (violets, comme aux vents d'altitude), les étoiles et la
    /// ville dans la nuit (comme au premier niveau).
    #[test]
    fn soleil_ilots_etoiles_et_ville_restent_devant_leur_bande() {
        let bande = |y: i32| VOYAGE[(y / (H / 4)) as usize];
        for y in SOLEIL_VOYAGE_Y - 22..=SOLEIL_VOYAGE_Y + 22 {
            assert!([OR, ORANGE_SOLEIL].contains(&bande(y)), "soleil, y = {y}");
        }
        for y in ILOTS_VOYAGE_Y..ILOTS_VOYAGE_Y + ILOT_Y_ALEA as i32 + ILOT_HAUTEUR {
            assert_eq!(bande(y), CORAIL, "îlots, y = {y}");
            assert!(bande(y) != ILOTS_VOYAGE.0 && bande(y) != ILOTS_VOYAGE.1);
        }
        let d = ambiance(Theme::Descente);
        let ymax = d.etoiles.unwrap();
        for y in 0..ymax {
            let fond = bande(REMONTEE_VOYAGE_MAX + y);
            assert!(
                [ARDOISE_OMBRE, ARDOISE_NUIT].contains(&fond),
                "étoile, y = {y}"
            );
        }
        // La ville : les mêmes zones que celles du premier niveau, devant le même ciel.
        let toits = ambiance(Theme::Toits);
        for (z, zt) in zones(&d).iter().zip(zones(&toits).iter()) {
            assert_eq!((z.haut, z.bas, z.couleur), (zt.haut, zt.bas, zt.couleur));
        }
    }
}
