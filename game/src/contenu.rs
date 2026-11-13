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
pub const SOUS_TITRE: &str = "Grimpe de l'orage jusqu'au soleil";
/// Rappel des touches sous les menus.
pub const AIDE_MENU: &str = "Haut/Bas : choisir   Entrée : valider";

// --- Libellés de menus -----------------------------------------------------

pub const JOUER: &str = "JOUER";
pub const COMMENT_JOUER: &str = "COMMENT JOUER";
pub const REPRENDRE: &str = "REPRENDRE";
pub const RECOMMENCER: &str = "RECOMMENCER";
pub const QUITTER_AU_TITRE: &str = "QUITTER AU TITRE";
pub const REJOUER: &str = "REJOUER";
pub const RETOUR_TITRE: &str = "TITRE";

/// Libellé de l'option son selon son état (en capitales : à l'échelle du menu, les
/// minuscules flotteraient au-dessus du pied des majuscules).
pub fn libelle_son(actif: bool) -> &'static str {
    if actif {
        "SON : OUI"
    } else {
        "SON : NON"
    }
}

/// Texte affiché pour une ligne de menu ; `son` : état actuel du son.
pub fn libelle(choix: Choix, son: bool) -> &'static str {
    match choix {
        Choix::Jouer => JOUER,
        Choix::Aide => COMMENT_JOUER,
        Choix::Son => libelle_son(son),
        Choix::Reprendre => REPRENDRE,
        Choix::Recommencer => RECOMMENCER,
        Choix::QuitterAuTitre => QUITTER_AU_TITRE,
        Choix::Rejouer => REJOUER,
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
    ("M", "Son : oui ou non"),
];
/// Conseils de jeu, une ligne chacun.
pub const ASTUCES_AIDE: [&str; 4] = [
    "Écrase les flaques en leur sautant dessus.",
    "Les courants d'air te portent vers le haut.",
    "Les lanternes sont des points de reprise.",
    "Trois gouttes d'or par niveau : facultatives !",
];
pub const RETOUR_AIDE: &str = "Entrée ou Échap : retour";

// --- Niveaux ---------------------------------------------------------------

/// Invitation affichée sous la carte d'un niveau.
pub const CONTINUER: &str = "Entrée ou toucher : continuer";

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
        texte: "La pluie tambourine sur les toits. Pépin ouvre son parapluie et part vers le haut.",
        astuce: "Maintiens SAUT en l'air pour planer.",
    },
    Intro {
        texte: "L'averse redouble. Les flaques rôdent entre les cheminées.",
        astuce: "Saute-leur dessus pour les écraser.",
    },
    Intro {
        texte: "L'orage éclate. Chaque éclair est annoncé : garde la tête froide !",
        astuce: "Oui, tenir un parapluie ici est audacieux.",
    },
    Intro {
        texte: "Le ciel se déchire. Les courants montent : laisse-toi porter, puis plane.",
        astuce: "Les brins de vent montrent où ça monte.",
    },
    Intro {
        texte: "Là-haut, le soleil. Prends ton temps : il n'y a plus d'orage.",
        astuce: "Dernière ligne droite : le champignon t'aide.",
    },
];

/// Intro du niveau `indice` (0 pour le premier) ; un indice trop grand donne le dernier.
pub fn intro(indice: usize) -> &'static Intro {
    &INTROS[indice.min(NB_NIVEAUX - 1)]
}

/// Étiquette de la carte de présentation d'un niveau : « NIVEAU 2/5 » pour un niveau
/// normal (`nb_normaux` : nombre de niveaux normaux du jeu ; le bonus et l'épilogue ne
/// comptent pas), « NIVEAU BONUS » ou « ÉPILOGUE » pour les deux autres genres.
pub fn etiquette_niveau(infos: &InfosNiveau, nb_normaux: usize) -> String {
    match infos.genre {
        GenreNiveau::Normal => format!("NIVEAU {}/{nb_normaux}", infos.numero),
        GenreNiveau::Bonus => "NIVEAU BONUS".to_string(),
        GenreNiveau::Epilogue => "ÉPILOGUE".to_string(),
    }
}

// --- Pause -----------------------------------------------------------------

pub const TITRE_PAUSE: &str = "PAUSE";

// --- Écran de fin ----------------------------------------------------------

pub const TITRE_FIN: &str = "LE SOLEIL !";
pub const LIBELLE_TEMPS: &str = "TEMPS";
pub const LIBELLE_GOUTTES: &str = "GOUTTES D'OR";
pub const LIBELLE_MORTS: &str = "MORTS";
pub const LIBELLE_SCORE: &str = "SCORE";

/// Petit mot de fin selon le nombre de gouttes d'or ramassées.
pub fn message_fin(gouttes: u32, total: u32) -> &'static str {
    if gouttes >= total {
        "Collection complète : pas une goutte oubliée !"
    } else if gouttes * 2 >= total {
        "Pas mal ! Il reste des gouttes d'or ici et là."
    } else {
        "Il reste plein de gouttes d'or à dénicher."
    }
}

#[cfg(test)]
mod tests {
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
            libelle_son(true),
            libelle_son(false),
            libelle(Choix::Son, true),
            TITRE_AIDE,
            RETOUR_AIDE,
            CONTINUER,
            TITRE_PAUSE,
            TITRE_FIN,
            LIBELLE_TEMPS,
            LIBELLE_GOUTTES,
            LIBELLE_MORTS,
            LIBELLE_SCORE,
            message_fin(15, 15),
            message_fin(8, 15),
            message_fin(0, 15),
        ];
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

    #[test]
    fn le_message_de_fin_depend_des_gouttes() {
        assert!(message_fin(15, 15).contains("complète"));
        assert!(message_fin(9, 15).contains("Pas mal"));
        assert!(message_fin(2, 15).contains("plein"));
    }

    #[test]
    fn etiquette_de_niveau() {
        use pepin_core::levels::{nb_normaux, NIVEAUX};
        // Les cinq niveaux actuels : comme avant la carte 5.1.
        assert_eq!(
            etiquette_niveau(&NIVEAUX[1], nb_normaux(NIVEAUX)),
            "NIVEAU 2/5"
        );
        // Un jeu de 11 niveaux (9 normaux, bonus, épilogue) : le bonus et l'épilogue
        // ne sont pas comptés dans « /9 ».
        let mut infos = NIVEAUX[0];
        infos.numero = 9;
        assert_eq!(etiquette_niveau(&infos, 9), "NIVEAU 9/9");
        infos.genre = GenreNiveau::Bonus;
        assert_eq!(etiquette_niveau(&infos, 9), "NIVEAU BONUS");
        infos.genre = GenreNiveau::Epilogue;
        assert_eq!(etiquette_niveau(&infos, 9), "ÉPILOGUE");
        // Chaque caractère existe dans la police.
        for e in ["NIVEAU BONUS", "ÉPILOGUE", "NIVEAU 10/10"] {
            for c in e.chars() {
                assert!(pepin_core::font::glyphe(c).is_some(), "{c:?}");
            }
        }
    }
}
