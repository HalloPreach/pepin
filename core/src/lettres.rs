//! Les lettres cachées (carte 8.6) : P, É, P, I, N, une dans chacun de cinq niveaux.
//!
//! Demande d'AGA : « Un quatrième objet par niveau. Par exemple les lettres P, É, P,
//! I, N dispersées dans le jeu, pour débloquer une fin différente. »
//!
//! Chaque lettre est une case `*` de la grille de son niveau (une au plus par
//! niveau), toujours derrière un faux mur de la carte 8.5 : dans une salle secrète
//! ou au milieu d'un raccourci caché, jamais sur le chemin. LAQUELLE des cinq est
//! écrite ici, par CLÉ de niveau (la sauvegarde les retient par clé : les deux P
//! sont deux lettres différentes). Les cinq réunies (pas forcément dans la même
//! partie) : la fin secrète.
//!
//! Notions Rust illustrées ici : tableau constant de couples, `position` sur un
//! itérateur, `std::array::from_fn`.

/// Les lettres, dans l'ordre du mot ET du parcours : (clé du niveau, lettre).
/// Début (les toits, le marché), milieu (l'orage, le clocher), fin (la mer de nuages).
pub const LETTRES: [(&str, char); NB_LETTRES] = [
    ("toits", 'P'),
    ("marche", 'É'),
    ("orage", 'P'),
    ("clocher", 'I'),
    ("mer", 'N'),
];

/// Nombre de lettres à réunir.
pub const NB_LETTRES: usize = 5;

/// Rang (0 à 4) de la lettre cachée dans le niveau de clé `cle` (`None` : aucune).
pub fn rang_du_niveau(cle: &str) -> Option<usize> {
    LETTRES.iter().position(|&(c, _)| c == cle)
}

/// La lettre cachée dans le niveau de clé `cle` (`None` : aucune).
pub fn lettre_du_niveau(cle: &str) -> Option<char> {
    rang_du_niveau(cle).map(|i| LETTRES[i].1)
}

/// Les lettres trouvées, dans l'ordre du mot : `trouvee(cle)` dit si la lettre du
/// niveau de clé `cle` a été prise (la sauvegarde répond).
pub fn trouvees(trouvee: impl Fn(&str) -> bool) -> [bool; NB_LETTRES] {
    std::array::from_fn(|i| trouvee(LETTRES[i].0))
}

/// Combien en manque-t-il ?
pub fn manquantes(trouvees: &[bool; NB_LETTRES]) -> usize {
    trouvees.iter().filter(|&&t| !t).count()
}
