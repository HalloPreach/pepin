//! Tous les textes du jeu, rangés ici et nulle part ailleurs.
//!
//! Ce module ne contient que des DONNÉES (plus quelques toutes petites
//! fonctions de choix de phrase) : l'interface (`ui/`) les affiche, elle ne
//! les connaît pas. Pour corriger une faute ou changer un ton, c'est donc le
//! seul fichier à ouvrir.
//!
//! Notions Rust illustrées ici : `const` de type `&str` (texte qui vit pendant
//! tout le programme), tableaux de taille fixe `[T; N]` dont la taille est
//! vérifiée par le compilateur, `struct` de données, tests qui parcourent les
//! données.

use pepin_core::levels::{GenreNiveau, NB_NIVEAUX};
use pepin_core::zones::InfosNiveau;

use crate::menu::Choix;

// --- Écran titre -----------------------------------------------------------

/// Le nom du jeu (« pépin » = parapluie, en argot).
pub const TITRE: &str = "PÉPIN";
/// Le voyage (carte 5.18) : on monte de l'orage au soleil, puis l'épilogue
/// redescend jusqu'aux toits du départ.
pub const SOUS_TITRE: &str = "De l'orage jusqu'au soleil, et retour";
/// Rappel des touches sous les menus.
pub const AIDE_MENU: &str = "Haut/Bas : choisir   Entrée : valider";

// --- Libellés de menus -----------------------------------------------------

pub const JOUER: &str = "JOUER";
pub const BOUTIQUE: &str = "BOUTIQUE";
/// La carte des niveaux (carte 7.4).
pub const CARTE: &str = "CARTE";
/// Revoir les trois images de l'histoire (carte 8.4).
pub const HISTOIRE: &str = "HISTOIRE";
pub const COMMENT_JOUER: &str = "COMMENT JOUER";
pub const REPRENDRE: &str = "REPRENDRE";
pub const RECOMMENCER: &str = "RECOMMENCER";
pub const QUITTER_AU_TITRE: &str = "QUITTER AU TITRE";
pub const REJOUER: &str = "REJOUER";
pub const RETOUR_TITRE: &str = "TITRE";

/// Texte affiché pour une ligne de menu. Le son n'a plus de ligne (carte 7.8b) : une
/// icône de haut-parleur en bas à gauche du titre et de la pause (`ecrans/mod.rs`).
pub fn libelle(choix: Choix) -> &'static str {
    match choix {
        Choix::Jouer => JOUER,
        Choix::Boutique => BOUTIQUE,
        Choix::Carte => CARTE,
        Choix::Histoire => HISTOIRE,
        Choix::Aide => COMMENT_JOUER,
        Choix::Reprendre => REPRENDRE,
        Choix::Recommencer => RECOMMENCER,
        Choix::QuitterAuTitre => QUITTER_AU_TITRE,
        Choix::Rejouer | Choix::RejouerNiveau => REJOUER,
        Choix::RetourTitre => RETOUR_TITRE,
    }
}

// --- Écran « comment jouer » -----------------------------------------------

pub const TITRE_AIDE: &str = "COMMENT JOUER";
/// Couples (touches, effet), affichés sur deux colonnes.
pub const COMMANDES_AIDE: [(&str, &str); 5] = [
    ("FLÈCHES, Q, D", "Courir"),
    ("ESPACE, HAUT, W", "Sauter, plus haut si maintenu"),
    ("SAUT EN L'AIR", "Maintenu : le parapluie plane !"),
    ("ÉCHAP, P", "Pause"),
    ("M", "Son, ou l'icône en bas à gauche"),
];
/// Conseils de jeu, une ligne chacun. Carte 5.18 : les nouveautés des
/// niveaux (vent, grêle, auvents, ventilateurs, montgolfières, escargots, chats,
/// aiguilles, Grand Nuage) se jouent avec les MÊMES touches : l'aide le dit, et
/// rappelle la seule règle à savoir d'avance (la grêle, parapluie ouvert). Carte
/// 6.2 : une septième ligne, la règle des vies.
/// 48 caractères au plus : une ligne du panneau (test `les_astuces_tiennent`).
pub const ASTUCES_AIDE: [&str; 7] = [
    "Écrase les flaques en leur sautant dessus.",
    "Les courants d'air te portent vers le haut.",
    "Sous la grêle, garde le parapluie ouvert !",
    "Vent, auvent, horloge : aucune touche en plus !",
    "Les lanternes sont des points de reprise.",
    "Trois gouttes d'or par niveau : facultatives !",
    "Trois gouttes d'un niveau : une vie de plus !",
];
pub const RETOUR_AIDE: &str = "Entrée ou Échap : retour";

// --- Niveaux ---------------------------------------------------------------

/// Invitation affichée sous la carte d'un niveau.
pub const CONTINUER: &str = "Entrée ou toucher : continuer";

/// Ligne ajoutée à la carte d'un niveau qui ne coûte aucune vie (le bonus, carte
/// 6.2 : `pepin_core::vies::coute_des_vies`).
pub const SANS_PERTE_DE_VIE: &str = "Ici, tomber ne coûte aucune vie.";

// --- Vies (carte 6.2) ------------------------------------------------------

/// Titre de l'écran affiché quand Pépin perd sa dernière vie.
pub const TITRE_PLUS_DE_VIES: &str = "PLUS DE VIES";

/// Première phrase de l'écran « PLUS DE VIES » : ce qui va se passer, avec le
/// nombre de vies du départ (`VIES_DEPART`, réglable).
pub fn recommencer_avec_vies(vies: u8) -> String {
    format!("Le niveau recommence, avec {vies} vies.")
}

/// Seconde phrase : les gouttes ramassées dans ce niveau sont perdues.
pub const GOUTTES_A_REPRENDRE: &str = "Ses gouttes d'or sont à reprendre.";

// --- Histoire (carte 8.4) --------------------------------------------------

/// Les deux lignes de chacune des trois images de l'histoire, dans l'ordre :
/// avant le premier niveau, après le Grand Nuage, avant la descente. La ville est
/// sous une pluie sans fin, les gouttes d'or sont des éclats de soleil tombés, et
/// Pépin monte réveiller le soleil. Le Grand Nuage n'est pas vaincu au niveau 6 : il
/// s'enfuit, et revient au sommet (« Le retour du Grand Nuage », carte 8.11). Chaque
/// ligne tient dans le bandeau du bas, 8 px de marge de chaque côté (test
/// `l_histoire_tient_dans_son_bandeau` de `ecrans::histoire`).
pub const RECIT: [[&str; 2]; 3] = [
    [
        "La pluie ne s'arrête plus : le soleil dort.",
        "Pépin prend son parapluie et monte le réveiller.",
    ],
    [
        "Le Grand Nuage s'enfuit sous la mer de nuages.",
        "Plus haut, une lueur d'or : des éclats de soleil !",
    ],
    [
        "Le Grand Nuage s'est vidé de sa pluie.",
        "Le soleil ouvre un œil : montrons-lui la ville !",
    ],
];

/// Invitation sous une image de l'histoire.
pub const APPUIE_POUR_CONTINUER: &str = "APPUIE POUR CONTINUER";

/// Texte d'accueil d'un niveau : une phrase d'ambiance et un conseil.
#[derive(Debug, Clone, Copy)]
pub struct Intro {
    pub texte: &'static str,
    pub astuce: &'static str,
}

/// Une intro par niveau, dans l'ordre. Le type `[Intro; NB_NIVEAUX]` fait
/// échouer la compilation si on ajoute un niveau sans écrire son texte.
pub const INTROS: [Intro; NB_NIVEAUX] = [
    Intro {
        texte: "La pluie tambourine sur les toits. Pépin ouvre son parapluie : en route vers le soleil !",
        astuce: "Maintiens SAUT en l'air pour planer.",
    },
    Intro {
        texte: "L'averse redouble. Les flaques rôdent entre les cheminées.",
        astuce: "Saute-leur dessus pour les écraser.",
    },
    Intro {
        texte: "Sous les toiles, tout rebondit.",
        astuce:
            "Tombe sur un auvent : il te relance. Écrase le chat endormi : au « ! », il bondit.",
    },
    Intro {
        texte: "L'orage éclate. Chaque éclair est annoncé : garde la tête froide !",
        astuce: "Oui, tenir un parapluie ici est audacieux.",
    },
    Intro {
        texte: "Une seule direction : en haut.",
        astuce: "L'escargot est un tremplin. Sur une aiguille, laisse-toi porter.",
    },
    Intro {
        texte: "C'est lui qui cache le soleil : le Grand Nuage ne veut pas te laisser passer.",
        astuce: "Quand il s'essouffle, monte dans son vent et saute-lui sur la tête.",
    },
    Intro {
        texte: "Le Grand Nuage s'est enfui. Les courants montent : laisse-toi porter, puis plane.",
        astuce: "Les brins de vent montrent où ça monte.",
    },
    Intro {
        texte: "Laisse le vent choisir, puis reprends la main.",
        astuce: "Parapluie ouvert, le vent te porte ; fermé, tu lui résistes.",
    },
    Intro {
        texte: "Là-haut, le soleil. Prends ton temps : l'orage gronde encore en bas.",
        astuce: "Le champignon t'aide à monter.",
    },
    // Le sommet (cartes 8.2 et 8.2b) : le Grand Nuage, enfui au niveau 6, revient.
    Intro {
        texte: "Le Grand Nuage remonte de la mer : il veut garder le soleil endormi.",
        astuce: "Ne t'arrête pas : il avance !",
    },
    // Le bonus (carte 5.16) : seulement avec toutes les gouttes.
    Intro {
        texte: "Il ne tient que le temps de le traverser.",
        astuce: "Ne t'arrête pas : tout s'effrite derrière toi.",
    },
    // L'épilogue (carte 5.17) : la descente, du soleil jusqu'aux toits.
    Intro {
        texte: "Le soleil est réveillé : on redescend lui montrer la ville.",
        astuce: "Parapluie ouvert, la grêle ne te touche pas.",
    },
];

/// Intro du niveau `indice` (0 pour le premier) ; un indice trop grand donne le dernier,
/// sauf, dans les tests et le build de QA, ceux des niveaux d'essai (vertical, vent,
/// grêle, auvent, ventilateur, montgolfière).
pub fn intro(indice: usize) -> &'static Intro {
    #[cfg(any(test, feature = "debug_niveaux"))]
    if indice == crate::monde::INDICE_ESSAI {
        return &INTRO_ESSAI;
    }
    #[cfg(any(test, feature = "debug_niveaux"))]
    if indice == crate::monde::INDICE_ESSAI_VENT {
        return &INTRO_ESSAI_VENT;
    }
    #[cfg(any(test, feature = "debug_niveaux"))]
    if indice == crate::monde::INDICE_ESSAI_GRELE {
        return &INTRO_ESSAI_GRELE;
    }
    #[cfg(any(test, feature = "debug_niveaux"))]
    if indice == crate::monde::INDICE_ESSAI_AUVENT {
        return &INTRO_ESSAI_AUVENT;
    }
    #[cfg(any(test, feature = "debug_niveaux"))]
    if indice == crate::monde::INDICE_ESSAI_VENTILATEUR {
        return &INTRO_ESSAI_VENTILATEUR;
    }
    #[cfg(any(test, feature = "debug_niveaux"))]
    if indice == crate::monde::INDICE_ESSAI_MONTGOLFIERE {
        return &INTRO_ESSAI_MONTGOLFIERE;
    }
    #[cfg(any(test, feature = "debug_niveaux"))]
    if indice == crate::monde::INDICE_ESSAI_ESCARGOT {
        return &INTRO_ESSAI_ESCARGOT;
    }
    #[cfg(any(test, feature = "debug_niveaux"))]
    if indice == crate::monde::INDICE_ESSAI_CHAT {
        return &INTRO_ESSAI_CHAT;
    }
    #[cfg(any(test, feature = "debug_niveaux"))]
    if indice == crate::monde::INDICE_ESSAI_HORLOGE {
        return &INTRO_ESSAI_HORLOGE;
    }
    #[cfg(any(test, feature = "debug_niveaux"))]
    if indice == crate::monde::INDICE_ESSAI_NUAGE {
        return &INTRO_ESSAI_NUAGE;
    }
    &INTROS[indice.min(NB_NIVEAUX - 1)]
}

/// Intro du niveau d'essai vertical (build de QA seulement, voir `monde::INDICE_ESSAI`).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INTRO_ESSAI: Intro = Intro {
    texte: "Niveau d'essai : quatre écrans de haut. La caméra suit Pépin en hauteur.",
    astuce: "Grimpe, saute sur le champignon, redescends.",
};

/// Intro du niveau d'essai « vent » (build de QA seulement, voir
/// `monde::INDICE_ESSAI_VENT`).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INTRO_ESSAI_VENT: Intro = Intro {
    texte: "Le vent latéral pousse Pépin : 60 px par seconde, deux fois plus fort en planant.",
    astuce: "Planer avec le vent, sauter contre.",
};

/// Intro du niveau d'essai « grêle » (build de QA seulement, voir
/// `monde::INDICE_ESSAI_GRELE`).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INTRO_ESSAI_GRELE: Intro = Intro {
    texte: "La grêle tombe sans arrêt : parapluie fermé, elle est mortelle.",
    astuce: "Saute tôt et plane dessous.",
};

/// Intro du niveau d'essai « auvent » (build de QA seulement, voir
/// `monde::INDICE_ESSAI_AUVENT`).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INTRO_ESSAI_AUVENT: Intro = Intro {
    texte: "Un auvent relance Pépin en diagonale : quatre tuiles plus loin.",
    astuce: "Droite : plus loin. Gauche : plus près.",
};

/// Intro du niveau d'essai « ventilateur » (build de QA seulement, voir
/// `monde::INDICE_ESSAI_VENTILATEUR`).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INTRO_ESSAI_VENTILATEUR: Intro = Intro {
    texte: "Un ventilateur souffle vers le haut : il porte Pépin bien plus haut qu'un saut.",
    astuce: "Attends le souffle (la lampe clignote), reste dans la colonne.",
};

/// Intro du niveau d'essai « montgolfière » (build de QA seulement, voir
/// `monde::INDICE_ESSAI_MONTGOLFIERE`).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INTRO_ESSAI_MONTGOLFIERE: Intro = Intro {
    texte: "Une montgolfière monte quand Pépin est dans sa nacelle, et redescend sans lui.",
    astuce: "Attends en haut qu'elle s'arrête, puis avance vers l'étage.",
};

/// Intro du niveau d'essai « escargot » (build de QA seulement, voir
/// `monde::INDICE_ESSAI_ESCARGOT`).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INTRO_ESSAI_ESCARGOT: Intro = Intro {
    texte: "L'escargot est un champignon qui marche : impossible à écraser, il fait rebondir.",
    astuce: "Attends-le au pied de la falaise, puis saute-lui dessus.",
};

/// Intro du niveau d'essai « chat » (build de QA seulement, voir
/// `monde::INDICE_ESSAI_CHAT`).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INTRO_ESSAI_CHAT: Intro = Intro {
    texte: "Le chat dort, puis se réveille. Endormi, on l'écrase ; réveillé, il bondit sur toi.",
    astuce: "Attends les « z », puis tombe-lui dessus.",
};

/// Intro du niveau d'essai « horloge » (build de QA seulement, voir
/// `monde::INDICE_ESSAI_HORLOGE`).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INTRO_ESSAI_HORLOGE: Intro = Intro {
    texte: "Les aiguilles de l'horloge tournent et emportent ce qui est posé dessus.",
    astuce: "Saute dessus quand elle passe en bas, saute en haut.",
};

/// Intro du niveau d'essai « nuage » : l'arène du Grand Nuage (build de QA seulement,
/// voir `monde::INDICE_ESSAI_NUAGE`).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INTRO_ESSAI_NUAGE: Intro = Intro {
    texte: "Le Grand Nuage ne veut pas te laisser passer.",
    astuce: "Quand il s'essouffle, monte dans son vent et saute-lui sur la tête.",
};

/// Étiquette de la carte de présentation d'un niveau : « NIVEAU 2/5 » pour un niveau
/// normal (`nb_normaux` : nombre de niveaux normaux du jeu ; le bonus et l'épilogue ne
/// comptent pas), « NIVEAU BONUS » ou « ÉPILOGUE » pour les deux autres genres. Le
/// numéro 0 est celui d'un niveau hors de la table (le niveau d'essai du build de QA).
pub fn etiquette_niveau(infos: &InfosNiveau, nb_normaux: usize) -> String {
    match infos.genre {
        GenreNiveau::Normal if infos.numero == 0 => "NIVEAU D'ESSAI".to_string(),
        GenreNiveau::Normal => format!("NIVEAU {}/{nb_normaux}", infos.numero),
        GenreNiveau::Bonus => "NIVEAU BONUS".to_string(),
        GenreNiveau::Epilogue => "ÉPILOGUE".to_string(),
    }
}

// --- Pause -----------------------------------------------------------------

pub const TITRE_PAUSE: &str = "PAUSE";

// --- Écran de fin ----------------------------------------------------------

pub const TITRE_FIN: &str = "LE SOLEIL !";
/// Titre et phrase de l'écran de fin quand la partie se termine par l'épilogue
/// (carte 5.17) : Pépin est revenu, la nuit, sur le toit du premier niveau. La phrase
/// conclut l'histoire (carte 8.11) : la pluie s'est arrêtée, le soleil est réveillé.
pub const TITRE_FIN_RETOUR: &str = "DE RETOUR !";
pub const PHRASE_RETOUR: &str = "Plus de pluie : Pépin replie son parapluie.";
pub const LIBELLE_TEMPS: &str = "TEMPS";
pub const LIBELLE_GOUTTES: &str = "GOUTTES D'OR";
/// Gouttes du niveau bonus, comptées à part (ligne affichée s'il a été joué).
pub const LIBELLE_BONUS: &str = "NIVEAU SECRET";
pub const LIBELLE_MORTS: &str = "MORTS";
/// Vies restantes (ligne affichée seulement quand elle tient, carte 6.2).
pub const LIBELLE_VIES: &str = "VIES";
pub const LIBELLE_SCORE: &str = "SCORE";
/// Sur la ligne SCORE de l'écran de fin, quand la partie bat le record (carte 7.2).
pub const NOUVEAU_RECORD: &str = "NOUVEAU RECORD !";
/// Écran titre (carte 7.2) : le meilleur score (le porte-monnaie n'a plus de mot depuis la
/// carte 7.5b, seulement la goutte d'or).
pub const LIBELLE_RECORD: &str = "RECORD";

// --- Boutique (carte 7.5) ----------------------------------------------------

/// Dernière ligne de la boutique.
pub const RETOUR: &str = "RETOUR";
pub const GRATUIT: &str = "GRATUIT";
pub const ETAT_EQUIPE: &str = "ÉQUIPÉ";
pub const ETAT_ACHETE: &str = "ACHETÉ";
pub const ETAT_A_ACHETER: &str = "À ACHETER";
pub const OUI: &str = "OUI";
pub const NON: &str = "NON";
pub const OK: &str = "OK";
pub const IL_TE_MANQUE: &str = "IL TE MANQUE";
/// Rappel des touches en bas de la boutique.
pub const AIDE_BOUTIQUE: &str = "Entrée : choisir   Échap : retour";

// --- Carte des niveaux (carte 7.4) ----------------------------------------------

pub const TITRE_PLAN: &str = "CARTE DES NIVEAUX";
/// Rappel des touches en bas de la carte.
pub const AIDE_PLAN: &str = "Haut/Bas : choisir  Entrée : jouer";
pub const NIVEAU_SECRET: &str = "NIVEAU SECRET";

// --- Démo du titre (carte 8.1) ----------------------------------------------

/// Bandeau clignotant en haut pendant que le robot joue seul.
pub const DEMO: &str = "DÉMO";
/// Bandeau du bas : toute entrée ramène au titre.
pub const APPUIE_SUR_UNE_TOUCHE: &str = "APPUIE SUR UNE TOUCHE";
pub const INDICE_SECRET: &str = "Trouve toutes les gouttes d'or des niveaux pour l'ouvrir.";
pub const VERROUILLE: &str = "VERROUILLÉ";
pub const INDICE_VERROUILLE: &str = "Atteins-le en jouant pour l'ouvrir.";
/// Étape jamais finie : à la place du meilleur temps.
pub const PAS_ENCORE_FINI: &str = "PAS ENCORE FINI";
pub const LIBELLE_MEILLEUR: &str = "MEILLEUR";
/// Bilan d'un niveau joué depuis la carte.
pub const NOUVEAU_MEILLEUR_TEMPS: &str = "NOUVEAU MEILLEUR TEMPS";
pub const PREMIER_TEMPS: &str = "PREMIER TEMPS SUR CE NIVEAU !";

/// « 1 GOUTTE », « 60 GOUTTES ».
pub fn gouttes(n: u32) -> String {
    if n == 1 {
        "1 GOUTTE".to_string()
    } else {
        format!("{n} GOUTTES")
    }
}

/// Gouttes d'or ramassées mais déjà comptées (carte 7.9) : « 1 GOUTTE DÉJÀ COMPTÉE »,
/// « 3 GOUTTES DÉJÀ COMPTÉES ».
pub fn deja_comptees(n: u32) -> String {
    if n == 1 {
        "1 GOUTTE DÉJÀ COMPTÉE".to_string()
    } else {
        format!("{n} GOUTTES DÉJÀ COMPTÉES")
    }
}

/// Question de la confirmation d'achat.
pub fn acheter_pour(prix: u32) -> String {
    format!("ACHETER POUR {prix} ?")
}

/// Mot de fin quand tout a été ramassé. Sur UNE ligne de l'écran de fin (42
/// caractères au plus) : « Collection complète : pas une goutte oubliée ! » laissait
/// « oubliée ! » seul sur une seconde ligne (vu sur la capture 517-complet-fin).
pub const COLLECTION_COMPLETE: &str = "Collection complète : rien n'est oublié !";

/// Le niveau bonus, vu de l'écran de fin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BonusFin {
    /// Le jeu n'a pas de bonus.
    Absent,
    /// Il y en a un, mais il n'a pas été débloqué (il manquait des gouttes).
    Manque,
    /// Il a été joué : gouttes ramassées dans le bonus, et son total.
    Joue { gouttes: u32, total: u32 },
}

/// Petit mot de fin selon les gouttes d'or ramassées : `gouttes` sur `total` pour
/// le parcours (tous les niveaux sauf le bonus), et le bonus à part. Quand le bonus
/// n'a pas été débloqué, le mot dit comment l'ouvrir : un niveau secret attend
/// celui qui trouve toutes les gouttes (celles des niveaux normaux).
pub fn message_fin(gouttes: u32, total: u32, bonus: BonusFin) -> &'static str {
    match bonus {
        BonusFin::Joue {
            gouttes: b,
            total: tb,
        } if gouttes >= total && b >= tb => COLLECTION_COMPLETE,
        BonusFin::Joue { .. } => {
            "Bravo, tu as trouvé le niveau secret ! Quelques gouttes d'or manquent encore."
        }
        BonusFin::Absent if gouttes >= total => COLLECTION_COMPLETE,
        BonusFin::Absent if gouttes * 2 >= total => {
            "Pas mal ! Il reste des gouttes d'or ici et là."
        }
        BonusFin::Absent => "Il reste plein de gouttes d'or à dénicher.",
        BonusFin::Manque if gouttes * 2 >= total => {
            "Pas mal ! Il reste quelques gouttes d'or. Toutes ouvrent un niveau secret."
        }
        BonusFin::Manque => {
            "Il reste plein de gouttes d'or à dénicher. Toutes ouvrent un niveau secret."
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use pepin_core::font;

    /// Tous les textes fixes, pour les contrôles d'ensemble.
    fn tous_les_textes() -> Vec<&'static str> {
        let mut t = vec![
            TITRE,
            SOUS_TITRE,
            AIDE_MENU,
            JOUER,
            COMMENT_JOUER,
            REPRENDRE,
            RECOMMENCER,
            QUITTER_AU_TITRE,
            REJOUER,
            RETOUR_TITRE,
            TITRE_AIDE,
            RETOUR_AIDE,
            CONTINUER,
            TITRE_PAUSE,
            TITRE_FIN,
            TITRE_FIN_RETOUR,
            PHRASE_RETOUR,
            LIBELLE_TEMPS,
            LIBELLE_GOUTTES,
            LIBELLE_BONUS,
            LIBELLE_MORTS,
            LIBELLE_VIES,
            LIBELLE_SCORE,
            NOUVEAU_RECORD,
            LIBELLE_RECORD,
            BOUTIQUE,
            CARTE,
            TITRE_PLAN,
            AIDE_PLAN,
            NIVEAU_SECRET,
            INDICE_SECRET,
            VERROUILLE,
            INDICE_VERROUILLE,
            PAS_ENCORE_FINI,
            LIBELLE_MEILLEUR,
            NOUVEAU_MEILLEUR_TEMPS,
            PREMIER_TEMPS,
            RETOUR,
            GRATUIT,
            ETAT_EQUIPE,
            ETAT_ACHETE,
            ETAT_A_ACHETER,
            OUI,
            NON,
            OK,
            IL_TE_MANQUE,
            AIDE_BOUTIQUE,
            SANS_PERTE_DE_VIE,
            TITRE_PLUS_DE_VIES,
            GOUTTES_A_REPRENDRE,
            HISTOIRE,
            APPUIE_POUR_CONTINUER,
        ];
        t.extend(RECIT.iter().flatten());
        let vies = recommencer_avec_vies(pepin_core::vies::VIES_DEPART);
        t.push(vies.leak());
        t.extend(tous_les_mots_de_fin());
        for (touches, effet) in COMMANDES_AIDE {
            t.push(touches);
            t.push(effet);
        }
        t.extend(ASTUCES_AIDE);
        for i in INTROS {
            t.push(i.texte);
            t.push(i.astuce);
        }
        t
    }

    #[test]
    fn chaque_caractere_a_un_glyphe_dans_la_police() {
        for texte in tous_les_textes() {
            for c in texte.chars() {
                assert!(
                    font::glyphe(c).is_some(),
                    "pas de glyphe pour {c:?} dans « {texte} »"
                );
            }
        }
    }

    #[test]
    fn les_accents_francais_utilises_sont_bien_presents() {
        // Garde-fou : si quelqu'un « simplifie » les accents, le test le voit.
        let tout = tous_les_textes().join(" ");
        for c in ['É', 'è', 'é', 'ê', 'à'] {
            assert!(tout.contains(c), "l'accent {c} a disparu des textes");
        }
    }

    /// Les astuces de l'aide tiennent chacune sur une ligne du panneau (312 px de
    /// large, marges comprises : 48 caractères de 6 px au plus).
    #[test]
    fn les_astuces_tiennent() {
        for a in ASTUCES_AIDE {
            assert!(
                a.chars().count() <= 48,
                "« {a} » : trop long pour une ligne"
            );
        }
    }

    #[test]
    fn aucun_texte_n_est_vide() {
        for t in tous_les_textes() {
            assert!(!t.trim().is_empty());
        }
    }

    #[test]
    fn aucune_mention_hors_sujet() {
        let tout = tous_les_textes().join(" ").to_lowercase();
        for interdit in ["http", "www.", "@"] {
            assert!(
                !tout.contains(interdit),
                "« {interdit} » ne doit pas figurer"
            );
        }
    }

    #[test]
    fn l_intro_hors_bornes_donne_le_dernier_niveau() {
        assert_eq!(intro(99).texte, INTROS[NB_NIVEAUX - 1].texte);
        assert_eq!(intro(0).texte, INTROS[0].texte);
    }

    /// Tous les mots de fin possibles (chaque cas de `message_fin`).
    pub(crate) fn tous_les_mots_de_fin() -> Vec<&'static str> {
        let joue = |gouttes| BonusFin::Joue { gouttes, total: 3 };
        vec![
            message_fin(30, 30, joue(3)),
            message_fin(30, 30, joue(2)),
            message_fin(28, 30, joue(3)),
            message_fin(15, 15, BonusFin::Absent),
            message_fin(8, 15, BonusFin::Absent),
            message_fin(0, 15, BonusFin::Absent),
            message_fin(26, 30, BonusFin::Manque),
            message_fin(3, 30, BonusFin::Manque),
        ]
    }

    #[test]
    fn le_message_de_fin_depend_des_gouttes() {
        // Sans bonus dans le jeu : les trois mots d'avant la carte 5.17.
        assert!(message_fin(15, 15, BonusFin::Absent).contains("complète"));
        assert!(message_fin(9, 15, BonusFin::Absent).contains("Pas mal"));
        assert!(message_fin(2, 15, BonusFin::Absent).contains("plein"));
        // Bonus manqué : le mot annonce le niveau secret.
        for gouttes in [0, 14, 15, 26] {
            let mot = message_fin(gouttes, 30, BonusFin::Manque);
            assert!(mot.contains("niveau secret"), "{gouttes} : {mot}");
        }
        assert!(message_fin(20, 30, BonusFin::Manque).contains("Pas mal"));
        assert!(message_fin(3, 30, BonusFin::Manque).contains("plein"));
        // Bonus joué : complète seulement avec TOUT (parcours et bonus).
        let joue = |gouttes| BonusFin::Joue { gouttes, total: 3 };
        assert!(message_fin(30, 30, joue(3)).contains("complète"));
        assert!(message_fin(30, 30, joue(1)).contains("trouvé le niveau secret"));
        assert!(message_fin(27, 30, joue(3)).contains("trouvé le niveau secret"));
    }

    #[test]
    fn etiquette_de_niveau() {
        use pepin_core::levels::{nb_normaux, NIVEAUX};
        // Les dix niveaux normaux (le marché couvert inséré en 3e place, carte 5.12 ;
        // le clocher en 5e, carte 5.13 ; l'œil du cyclone en 6e, carte 5.14 ; la mer
        // de nuages en 8e, carte 5.15 ; le retour du Grand Nuage en 10e, 8.2), puis le
        // bonus (carte 5.16) et l'épilogue (carte 5.17).
        assert_eq!(
            etiquette_niveau(&NIVEAUX[1], nb_normaux(NIVEAUX)),
            "NIVEAU 2/10"
        );
        assert_eq!(etiquette_niveau(&NIVEAUX[10], 10), "NIVEAU BONUS");
        assert_eq!(etiquette_niveau(&NIVEAUX[11], 10), "ÉPILOGUE");
        // Un jeu de 11 niveaux (9 normaux, bonus, épilogue) : le bonus et l'épilogue
        // ne sont pas comptés dans « /9 ».
        let mut infos = NIVEAUX[0];
        infos.numero = 9;
        assert_eq!(etiquette_niveau(&infos, 9), "NIVEAU 9/9");
        infos.genre = GenreNiveau::Bonus;
        assert_eq!(etiquette_niveau(&infos, 9), "NIVEAU BONUS");
        infos.genre = GenreNiveau::Epilogue;
        assert_eq!(etiquette_niveau(&infos, 9), "ÉPILOGUE");
        // Le niveau d'essai vertical (numéro 0, hors de la table) et son intro.
        let essai = pepin_core::levels::essai_vertical::INFOS;
        assert_eq!(essai.numero, 0);
        assert_eq!(etiquette_niveau(&essai, 5), "NIVEAU D'ESSAI");
        assert_eq!(intro(crate::monde::INDICE_ESSAI).texte, INTRO_ESSAI.texte);
        // Chaque caractère existe dans la police.
        for e in ["NIVEAU BONUS", "ÉPILOGUE", "NIVEAU 10/10", "NIVEAU D'ESSAI"]
            .into_iter()
            .chain([INTRO_ESSAI.texte, INTRO_ESSAI.astuce])
            .chain([INTRO_ESSAI_CHAT.texte, INTRO_ESSAI_CHAT.astuce])
            .chain([INTRO_ESSAI_HORLOGE.texte, INTRO_ESSAI_HORLOGE.astuce])
            .chain([INTRO_ESSAI_NUAGE.texte, INTRO_ESSAI_NUAGE.astuce])
        {
            for c in e.chars() {
                assert!(pepin_core::font::glyphe(c).is_some(), "{c:?}");
            }
        }
    }
}
