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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::Entite;
    use crate::levels::{indice_de_cle, GenreNiveau, NIVEAUX};

    /// Le seul test de la mécanique (carte 8.6) : les lettres écrivent « PÉPIN » dans
    /// l'ordre du parcours, chacune dans un niveau NORMAL différent, du début (deux
    /// premiers niveaux) à la fin (8e niveau ou plus) ; une case `*` dans chacun de ces
    /// niveaux et dans eux seuls, toujours derrière un faux mur (dans une zone cachée) :
    /// jamais sur le chemin. Et le compte de celles qui manquent.
    #[test]
    fn chaque_lettre_de_pepin_est_cachee_dans_son_niveau() {
        let mot: String = LETTRES.iter().map(|&(_, l)| l).collect();
        assert_eq!(mot, "PÉPIN");
        let indices: Vec<usize> = LETTRES
            .iter()
            .map(|&(cle, _)| indice_de_cle(NIVEAUX, cle).expect("clé de la table"))
            .collect();
        assert!(indices.windows(2).all(|p| p[0] < p[1]), "{indices:?}");
        assert!(indices[0] <= 1 && indices[4] >= 7, "{indices:?}");
        for infos in NIVEAUX {
            let n = infos.charger().unwrap();
            let lettres: Vec<_> = n
                .entites
                .iter()
                .filter_map(|e| match e {
                    Entite::Lettre(l) => Some(l.boite),
                    _ => None,
                })
                .collect();
            let attendu = usize::from(rang_du_niveau(infos.cle).is_some());
            assert_eq!(lettres.len(), attendu, "{}", infos.cle);
            if attendu == 1 {
                assert_eq!(infos.genre, GenreNiveau::Normal, "{}", infos.cle);
            }
            for b in lettres {
                assert!(
                    n.caches.iter().any(|z| z.touche(&b)),
                    "{} : lettre hors d'une zone cachée",
                    infos.cle
                );
            }
        }
        let t = trouvees(|cle| cle == "toits" || cle == "mer");
        assert_eq!(t, [true, false, false, false, true]);
        assert_eq!(manquantes(&t), 3);
        assert_eq!(lettre_du_niveau("orage"), Some('P'));
        assert_eq!(lettre_du_niveau("cyclone"), None);
    }
}
