//! Les lettres de « PÉPIN » cachées dans les niveaux (autrefois dans
//! `core/src/lettres.rs`).

use crate::commun::*;
use pepin_core::entities::Entite;
use pepin_core::lettres::*;
use pepin_core::levels::{indice_de_cle, GenreNiveau};

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
        .map(|&(cle, _)| indice_de_cle(niveaux(), cle).expect("clé de la table"))
        .collect();
    assert!(indices.windows(2).all(|p| p[0] < p[1]), "{indices:?}");
    assert!(indices[0] <= 1 && indices[4] >= 7, "{indices:?}");
    for infos in niveaux() {
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
