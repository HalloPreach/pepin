//! Tous les textes du jeu, rangés ici et nulle part ailleurs (sauf ceux d'un
//! niveau, écrits dans son fichier `.ron` : nom, intro et conseil).
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

use pepin_core::levels::{nb_normaux, niveaux_du_monde, GenreNiveau};
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
/// Carte 8.8 : 16 caractères, la plus longue ligne de menu (190 px sur 192).
pub const NOUVELLE_PARTIE_PLUS: &str = "NOUVELLE PARTIE+";
/// Le repère de la nouvelle partie + dans le HUD.
pub const REPERE_NG_PLUS: &str = "+";
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
pub const CONTINUER_CHAPITRE: &str = "CONTINUER";
pub const RETOUR_TITRE: &str = "TITRE";

/// Texte affiché pour une ligne de menu. Le son n'a plus de ligne (carte 7.8b) : une
/// icône de haut-parleur en bas à gauche du titre et de la pause (`ecrans/mod.rs`).
pub fn libelle(choix: Choix) -> &'static str {
    match choix {
        Choix::Jouer => JOUER,
        Choix::NouvellePartiePlus => NOUVELLE_PARTIE_PLUS,
        Choix::Boutique => BOUTIQUE,
        Choix::Carte => CARTE,
        Choix::Histoire => HISTOIRE,
        Choix::Aide => COMMENT_JOUER,
        Choix::Reprendre => REPRENDRE,
        Choix::Recommencer => RECOMMENCER,
        Choix::QuitterAuTitre => QUITTER_AU_TITRE,
        Choix::Rejouer | Choix::RejouerNiveau | Choix::RejouerCourse => REJOUER,
        Choix::Continuer => CONTINUER_CHAPITRE,
        Choix::RetourTitre => RETOUR_TITRE,
        Choix::Force(f) => f.libelle(),
    }
}

// --- Écran « comment jouer » -----------------------------------------------

pub const TITRE_AIDE: &str = "COMMENT JOUER";
/// Couples (touches, effet), affichés sur deux colonnes.
/// Carte 9.2b : une sixième ligne, BAS (le parapluie retourné, la barque).
/// Carte 9.2d : au sol, BAS assied Pépin, il ne bouge plus.
pub const COMMANDES_AIDE: [(&str, &str); 6] = [
    ("FLÈCHES, Q, D", "Courir"),
    ("ESPACE, HAUT, W", "Sauter, plus haut si maintenu"),
    ("SAUT EN L'AIR", "Maintenu : le parapluie plane !"),
    ("BAS, S", "Barque sur l'eau, figé au sol"),
    ("ÉCHAP, P", "Pause"),
    ("M", "Son, ou l'icône en bas à gauche"),
];
/// Conseils de jeu, une ligne chacun. Carte 5.18 : les nouveautés des
/// niveaux (vent, grêle, auvents, ventilateurs, montgolfières, escargots, chats,
/// aiguilles, Grand Nuage) se jouent avec les MÊMES touches : l'aide le dit, et
/// rappelle la seule règle à savoir d'avance (la grêle, parapluie ouvert). Carte
/// 6.2 : une septième ligne, la règle des vies. Carte 9.2b : « aucune touche en
/// plus » n'est plus vrai (BAS fait la barque) ; la ligne dit à quoi sert BAS.
/// 48 caractères au plus : une ligne du panneau (test `les_astuces_tiennent`).
pub const ASTUCES_AIDE: [&str; 7] = [
    "Écrase les flaques en leur sautant dessus.",
    "Les courants d'air te portent vers le haut.",
    "Sous la grêle, garde le parapluie ouvert !",
    "BAS : barque sur l'eau (au sol, on ne bouge pas)",
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
/// `l_histoire_tient_dans_son_bandeau` de `ecrans::histoire`). La 4e (carte 8.6) est
/// la fin secrète : elle ne se voit qu'avec les cinq lettres P, É, P, I, N.
pub const RECIT: [[&str; 2]; 4] = [
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
    [
        "Le soleil écrit ton nom en rayons d'or.",
        "La ville ouvre ses parapluies : merci, Pépin !",
    ],
];

/// Invitation sous une image de l'histoire.
pub const APPUIE_POUR_CONTINUER: &str = "APPUIE POUR CONTINUER";

// Les textes de la carte de présentation d'un niveau (la phrase et le conseil)
// sont dans le FICHIER du niveau depuis la carte 11.3 (`intro_texte`,
// `intro_astuce` : voir `assets/levels/LISEZMOI.md`), plus ici : un niveau s'ajoute
// avec son seul fichier.

/// Étiquette de la carte de présentation d'un niveau : « NIVEAU 2/5 » pour un niveau
/// normal (`nb_normaux` : nombre de niveaux normaux du jeu ; le bonus et l'épilogue ne
/// comptent pas), « NIVEAU BONUS » ou « ÉPILOGUE » pour les deux autres genres. Le
/// numéro 0 est celui d'un niveau hors de la table (le niveau d'essai du build de QA).
///
/// Hors du Monde 1, le chapitre vient devant : « CHAPITRE 2 · NIVEAU 3/8 » (carte
/// 9.1b : le Monde 2 est la suite du jeu, son 2e chapitre ; `nb_normaux` est alors le
/// nombre de niveaux normaux DU MONDE). Le Monde 1 garde exactement ses libellés
/// d'avant, sans le mot chapitre.
pub fn etiquette_niveau(infos: &InfosNiveau, nb_normaux: usize) -> String {
    let base = etiquette_dans_le_chapitre(infos, nb_normaux);
    // Règle GÉNÉRALE (carte 11.2), pas un cas du Monde 2 : seul le Monde 1 n'a pas de
    // chapitre écrit ; tout monde suivant (2, 3...) affiche le sien, numéroté par
    // `libelle_chapitre`.
    if infos.monde > 1 {
        format!("{} · {base}", libelle_chapitre(infos.monde))
    } else {
        base
    }
}

/// `etiquette_niveau` SANS le chapitre devant (« NIVEAU 1/8 », « NIVEAU BONUS ») :
/// pour le panneau de la carte des niveaux (carte 9.1b), trop étroit pour
/// « CHAPITRE 2 · NIVEAU BONUS », et dont le bandeau du haut nomme déjà le chapitre.
pub fn etiquette_dans_le_chapitre(infos: &InfosNiveau, nb_normaux: usize) -> String {
    match infos.genre {
        GenreNiveau::Normal if infos.numero == 0 => "NIVEAU D'ESSAI".to_string(),
        GenreNiveau::Normal => format!("NIVEAU {}/{nb_normaux}", infos.numero),
        GenreNiveau::Bonus => "NIVEAU BONUS".to_string(),
        GenreNiveau::Epilogue => "ÉPILOGUE".to_string(),
    }
}

/// `etiquette_niveau` avec le nombre de niveaux normaux DU MONDE du niveau, compté
/// dans la vraie table (carte 9.1) : ce qu'affiche la carte de titre du niveau
/// (« CHAPITRE 2 · NIVEAU 1/8 » au Monde 2).
pub fn etiquette(infos: &InfosNiveau) -> String {
    etiquette_niveau(
        infos,
        nb_normaux(niveaux_du_monde(niveaux::niveaux(), infos.monde)),
    )
}

/// `etiquette_dans_le_chapitre` comptée dans la vraie table : le panneau de la carte
/// des niveaux (carte 9.1b).
pub fn etiquette_courte(infos: &InfosNiveau) -> String {
    etiquette_dans_le_chapitre(
        infos,
        nb_normaux(niveaux_du_monde(niveaux::niveaux(), infos.monde)),
    )
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
/// Fin secrète (carte 8.6) : les cinq lettres P, É, P, I, N réunies.
pub const TITRE_FIN_SECRETE: &str = "FIN SECRÈTE";
pub const PHRASE_SECRETE: &str = "La ville entière fête Pépin sous le soleil.";
/// Ligne des lettres cachées de l'écran de fin (carte 8.6) : son nom dans le bilan,
/// et ce qui s'affiche selon les lettres qui manquent (`libelle_lettres`).
pub const LIBELLE_LETTRES: &str = "LETTRES";

/// « IL MANQUE 2 LETTRES », « IL MANQUE 1 LETTRE », « LETTRES » (toutes trouvées).
pub fn libelle_lettres(manquantes: usize) -> String {
    match manquantes {
        0 => LIBELLE_LETTRES.to_string(),
        1 => "IL MANQUE 1 LETTRE".to_string(),
        n => format!("IL MANQUE {n} LETTRES"),
    }
}
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

// --- Mode facile du Grand Nuage (carte 8.14) ------------------------------------

pub const TROP_DUR: &str = "TROP DUR ?";
pub const MODE_FACILE_PROPOSE: &str = "MODE FACILE : LE NUAGE N'A QUE 2 VIES";
pub const FACILE_SANS_MEDAILLE: &str = "Pas de médaille ni de record en facile.";
/// Mention du bilan et de la carte des niveaux : boss vaincu en facile.
pub const FACILE: &str = "FACILE";
/// Rappel des touches en bas de la boutique.
pub const AIDE_BOUTIQUE: &str = "Entrée : choisir   Échap : retour";

// --- Carte des niveaux (carte 7.4) ----------------------------------------------

pub const TITRE_PLAN: &str = "CARTE DES NIVEAUX";
/// Le libellé d'un monde vu par le joueur (carte 9.1b) : un CHAPITRE de la même
/// histoire (« CHAPITRE 2 »), pas un monde à part. Le code garde le mot `monde`.
pub fn libelle_chapitre(monde: u8) -> String {
    format!("CHAPITRE {monde}")
}
/// Le titre du chapitre 1 (carte 9.28), pendant de « Là où va la pluie » : Pépin
/// y monte des toits jusqu'aux nuages et au soleil.
pub const TITRE_CHAPITRE_1: &str = "CHAPITRE 1 : LÀ D'OÙ VIENT LA PLUIE";
/// Le titre du chapitre 2 (« Là où va la pluie », étape 9).
pub const TITRE_CHAPITRE_2: &str = "CHAPITRE 2 : LÀ OÙ VA LA PLUIE";
/// Titre de la carte des niveaux d'un chapitre (carte 9.1b), dans son bandeau du
/// haut : chaque chapitre porte son nom (carte 9.28 : le Monde 1 aussi, au lieu de
/// « CARTE DES NIVEAUX »). Il tient dans le bandeau (test `etiquette_de_niveau`).
pub fn titre_chapitre(monde: u8) -> Option<&'static str> {
    match monde {
        1 => Some(TITRE_CHAPITRE_1),
        2 => Some(TITRE_CHAPITRE_2),
        _ => None,
    }
}
/// Rappel des touches en bas de la carte (Gauche/Droite : bouton JOUER ou COURSE,
/// carte 8.7).
pub const AIDE_PLAN: &str = "Haut/Bas : niveau  Gauche/Droite : mode";
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
/// Étape finie sans meilleur temps connu (niveau migré de la v2 avec une médaille ou
/// un défi, carte 8.13) : à la place du meilleur temps.
pub const FINI_SANS_TEMPS: &str = "FINI";
pub const LIBELLE_MEILLEUR: &str = "MEILLEUR";
/// Bilan d'un niveau joué depuis la carte.
pub const NOUVEAU_MEILLEUR_TEMPS: &str = "NOUVEAU MEILLEUR TEMPS";
pub const PREMIER_TEMPS: &str = "PREMIER TEMPS SUR CE NIVEAU !";

// --- Course contre le robot (carte 8.7) ---------------------------------------

/// Les trois forces du fantôme, de la plus faible à la plus forte (`course::Force`).
pub const FORCES: [&str; 3] = ["DÉBUTANT", "HABITUÉ", "EXPERT"];
/// Bouton de la carte des niveaux et titre de l'écran du choix de la force.
pub const COURSE: &str = "COURSE";
pub const TITRE_COURSE: &str = "COURSE CONTRE LE ROBOT";
/// Choix de la force : rien de couru encore contre elle.
pub const PAS_ENCORE_COURU: &str = "PAS ENCORE COURU";
pub const MEILLEUR_RESULTAT: &str = "MEILLEUR :";
/// La prime pas encore touchée, sous le choix de la force (EXPERT).
pub fn prime_a_gagner(n: u32) -> String {
    format!("1re VICTOIRE : +{n} GOUTTES")
}
pub const GAGNE: &str = "GAGNÉ";
pub const PERDU: &str = "PERDU";
/// Bilan d'une course : le temps du joueur.
pub const TOI: &str = "TOI";
/// Sous le chrono pendant la course.
pub const ROBOT: &str = "ROBOT";
pub const ROBOT_ARRIVE: &str = "ROBOT ARRIVÉ";
pub const ROBOT_ABANDON: &str = "LE ROBOT A ABANDONNÉ";
/// Bilan d'une course.
pub const NOUVEAU_MEILLEUR_RESULTAT: &str = "MEILLEUR RÉSULTAT !";
/// La prime touchée au bilan.
pub fn prime_gagnee(n: u32) -> String {
    format!("EXPERT BATTU : +{n} GOUTTES !")
}

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
            NOUVELLE_PARTIE_PLUS,
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
            TITRE_FIN_SECRETE,
            PHRASE_SECRETE,
            LIBELLE_LETTRES,
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
            TITRE_CHAPITRE_1,
            TITRE_CHAPITRE_2,
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
            TROP_DUR,
            MODE_FACILE_PROPOSE,
            FACILE_SANS_MEDAILLE,
            FACILE,
            AIDE_BOUTIQUE,
            SANS_PERTE_DE_VIE,
            TITRE_PLUS_DE_VIES,
            GOUTTES_A_REPRENDRE,
            HISTOIRE,
            APPUIE_POUR_CONTINUER,
            COURSE,
            TITRE_COURSE,
            PAS_ENCORE_COURU,
            MEILLEUR_RESULTAT,
            GAGNE,
            PERDU,
            TOI,
            ROBOT,
            ROBOT_ARRIVE,
            ROBOT_ABANDON,
            NOUVEAU_MEILLEUR_RESULTAT,
        ];
        t.extend(FORCES);
        t.push(prime_a_gagner(5).leak());
        t.push(prime_gagnee(5).leak());
        t.extend(RECIT.iter().flatten());
        let vies = recommencer_avec_vies(pepin_core::vies::VIES_DEPART);
        t.push(vies.leak());
        t.extend(tous_les_mots_de_fin());
        for (touches, effet) in COMMANDES_AIDE {
            t.push(touches);
            t.push(effet);
        }
        t.extend(ASTUCES_AIDE);
        // Les intros de TOUS les niveaux (parcours et essais), lues dans leurs fichiers.
        for n in niveaux::niveaux().iter().chain(niveaux::essais()) {
            t.push(n.intro_texte);
            t.push(n.intro_astuce);
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
        use pepin_core::levels::{nb_normaux, niveaux_du_monde};
        // Les dix niveaux normaux (le marché couvert inséré en 3e place, carte 5.12 ;
        // le clocher en 5e, carte 5.13 ; l'œil du cyclone en 6e, carte 5.14 ; la mer
        // de nuages en 8e, carte 5.15 ; le retour du Grand Nuage en 10e, 8.2), puis le
        // bonus (carte 5.16) et l'épilogue (carte 5.17).
        // Compté PAR MONDE (carte 9.1) : les normaux du Monde 1.
        assert_eq!(
            etiquette_niveau(
                &niveaux::niveaux()[1],
                nb_normaux(niveaux_du_monde(niveaux::niveaux(), 1))
            ),
            "NIVEAU 2/10"
        );
        assert_eq!(
            etiquette_niveau(&niveaux::niveaux()[10], 10),
            "NIVEAU BONUS"
        );
        assert_eq!(etiquette_niveau(&niveaux::niveaux()[11], 10), "ÉPILOGUE");
        // Un jeu de 11 niveaux (9 normaux, bonus, épilogue) : le bonus et l'épilogue
        // ne sont pas comptés dans « /9 ».
        let mut infos = niveaux::niveaux()[0];
        infos.numero = 9;
        assert_eq!(etiquette_niveau(&infos, 9), "NIVEAU 9/9");
        infos.genre = GenreNiveau::Bonus;
        assert_eq!(etiquette_niveau(&infos, 9), "NIVEAU BONUS");
        infos.genre = GenreNiveau::Epilogue;
        assert_eq!(etiquette_niveau(&infos, 9), "ÉPILOGUE");
        // Le niveau d'essai vertical (numéro 0, hors de la table).
        let essai = *niveaux::par_cle("essai_vertical");
        assert_eq!(essai.numero, 0);
        assert_eq!(etiquette_niveau(&essai, 5), "NIVEAU D'ESSAI");
        // Monde 2 (cartes 9.1 et 9.1b) : le CHAPITRE devant, compté PAR MONDE ; le
        // Monde 1 inchangé (`etiquette` compte les normaux du monde dans la vraie table).
        let m2 = *niveaux::par_cle("caniveau");
        assert_eq!(etiquette_niveau(&m2, 8), "CHAPITRE 2 · NIVEAU 1/8");
        let mut bonus_m2 = m2;
        bonus_m2.genre = GenreNiveau::Bonus;
        assert_eq!(etiquette_niveau(&bonus_m2, 8), "CHAPITRE 2 · NIVEAU BONUS");
        assert_eq!(etiquette(&niveaux::niveaux()[1]), "NIVEAU 2/10");
        assert_eq!(etiquette(&niveaux::niveaux()[11]), "ÉPILOGUE");
        // Le panneau de la carte des niveaux (128 px de texte) : sans le chapitre ; la
        // carte de titre du niveau (296 px de large) a la place pour le chapitre.
        assert_eq!(etiquette_dans_le_chapitre(&bonus_m2, 8), "NIVEAU BONUS");
        assert!(pepin_core::font::largeur_texte("CHAPITRE 2 · NIVEAU BONUS", 1) <= 280);
        // Le titre du chapitre 2 tient dans le bandeau de la carte des niveaux : centré
        // sur l'écran (320 px), à droite du compteur de gouttes (x < 48), soit 224 px.
        for titre in [TITRE_CHAPITRE_1, TITRE_CHAPITRE_2] {
            assert!(pepin_core::font::largeur_texte(titre, 1) <= 224, "{titre}");
        }
        assert_eq!(titre_chapitre(1), Some(TITRE_CHAPITRE_1));
        assert_eq!(titre_chapitre(2), Some(TITRE_CHAPITRE_2));
        // Chaque caractère existe dans la police.
        for e in ["NIVEAU BONUS", "ÉPILOGUE", "NIVEAU 10/10", "NIVEAU D'ESSAI"]
            .into_iter()
            .chain([
                "CHAPITRE 2 · NIVEAU 3/8",
                TITRE_CHAPITRE_1,
                TITRE_CHAPITRE_2,
            ])
        {
            for c in e.chars() {
                assert!(pepin_core::font::glyphe(c).is_some(), "{c:?}");
            }
        }
    }
}
