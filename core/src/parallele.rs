//! Répartir un calcul sur les cœurs de la machine (allègement des outils).
//!
//! Les trois recherches du projet (robot de faisabilité, robot de QA, itinéraires
//! de la mesure de difficulté) passent presque tout leur temps au même endroit :
//! depuis un état, simuler CHAQUE plan possible. Ces simulations ne dépendent pas
//! les unes des autres : on peut les faire en même temps, sur plusieurs cœurs.
//!
//! [`appliquer`] fait exactement cela, sans rien changer au résultat :
//! - les résultats reviennent DANS L'ORDRE des éléments, quel que soit le fil qui
//!   les a calculés : la recherche les dépouille ensuite un par un comme avant,
//!   elle trouve donc les mêmes chemins, à l'image près (mêmes empreintes) ;
//! - le nombre de fils est partagé par TOUT le processus (`AIDES_EN_COURS`) :
//!   quand beaucoup de tests tournent en même temps (`lot`), chacun reste presque
//!   seul sur son cœur ; quand un test tourne seul (`carte <clé>`), il prend tous
//!   les cœurs. La machine n'est jamais surchargée par nos propres fils.
//!
//! Réglage : la variable d'environnement `PEPIN_FILS` (1 = tout en série, comme
//! avant). Par défaut : le nombre de cœurs de la machine.
//!
//! Notions Rust illustrées ici : `std::thread::scope` (des fils qui empruntent des
//! données locales, tous finis à la sortie du bloc), `AtomicUsize` (un compteur
//! partagé sans verrou), `OnceLock` (une valeur calculée une seule fois), les
//! bornes `Sync` / `Send` (ce qui peut être partagé entre fils, ou leur être
//! donné), et `Drop` (rendre ce qu'on a pris, même si un fil panique).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

/// Fils d'aide en cours dans tout le processus (voir l'en-tête).
static AIDES_EN_COURS: AtomicUsize = AtomicUsize::new(0);

/// En dessous de ce nombre d'éléments, tout se fait en série : lancer des fils
/// coûterait plus que ce qu'ils feraient gagner.
const SEUIL: usize = 4;

/// Nombre de fils que le processus peut occuper en tout : `PEPIN_FILS` s'il est
/// donné (au moins 1), sinon le nombre de cœurs.
pub fn fils_maximum() -> usize {
    static MAX: OnceLock<usize> = OnceLock::new();
    *MAX.get_or_init(|| {
        std::env::var("PEPIN_FILS")
            .ok()
            .and_then(|v| v.trim().parse::<usize>().ok())
            .filter(|&n| n >= 1)
            .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, usize::from))
    })
}

/// Un fil d'aide réservé. Le rendre, c'est le lâcher (`Drop`) : chaque fil d'aide
/// garde le sien et le rend DÈS qu'il n'a plus rien à faire (même s'il panique),
/// sans attendre la fin de l'appel. Un calcul long resté seul (un essai de la
/// mesure qui cherche un détour, par exemple) peut alors reprendre ces fils pour
/// ses propres recherches.
struct Jeton;

impl Jeton {
    /// Réserve au plus `voulu` fils d'aide parmi ceux encore libres (peut-être aucun).
    fn prendre(voulu: usize) -> Vec<Jeton> {
        let plafond = fils_maximum().saturating_sub(1);
        let mut pris = 0;
        // `try_update` relit et réessaie tant qu'un autre fil a changé le compteur.
        let _ = AIDES_EN_COURS.try_update(Ordering::AcqRel, Ordering::Acquire, |en_cours| {
            pris = voulu.min(plafond.saturating_sub(en_cours));
            Some(en_cours + pris)
        });
        (0..pris).map(|_| Jeton).collect()
    }
}

impl Drop for Jeton {
    fn drop(&mut self) {
        AIDES_EN_COURS.fetch_sub(1, Ordering::AcqRel);
    }
}

/// Applique `f` à chaque élément et rend les résultats dans l'ordre des éléments.
///
/// Le calcul est réparti sur les fils libres du processus ; sans fil libre, ou
/// pour une poignée d'éléments, il se fait en série. `f` ne doit dépendre que de
/// son élément (et de données partagées en lecture) : c'est ce que garantit la
/// borne `Sync`. `f` peut elle-même appeler `appliquer` : elle prendra les fils
/// encore libres, ou travaillera en série.
pub fn appliquer<T, R, F>(elements: &[T], f: F) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> R + Sync,
{
    // Dans le navigateur (wasm32), il n'y a pas de fils : toujours en série.
    if cfg!(target_arch = "wasm32") || elements.len() < SEUIL {
        return elements.iter().map(f).collect();
    }
    let jetons = Jeton::prendre(elements.len() - 1);
    if jetons.is_empty() {
        return elements.iter().map(f).collect();
    }
    // Chaque fil prend le prochain élément libre (`suivant`) : les plans longs et
    // les plans courts se répartissent tout seuls.
    let suivant = AtomicUsize::new(0);
    let travailler = || {
        let mut faits: Vec<(usize, R)> = Vec::new();
        loop {
            let i = suivant.fetch_add(1, Ordering::Relaxed);
            let Some(element) = elements.get(i) else {
                return faits;
            };
            faits.push((i, f(element)));
        }
    };
    let travailler = &travailler;
    let mut tout: Vec<(usize, R)> = std::thread::scope(|portee| {
        let aides: Vec<_> = jetons
            .into_iter()
            .map(|jeton| {
                portee.spawn(move || {
                    // Lâché à la fin de ce bloc : le fil d'aide rend sa place.
                    let _jeton = jeton;
                    travailler()
                })
            })
            .collect();
        let mut tout = travailler();
        for aide in aides {
            match aide.join() {
                Ok(faits) => tout.extend(faits),
                // La panique d'un fil d'aide est celle de l'appelant.
                Err(panique) => std::panic::resume_unwind(panique),
            }
        }
        tout
    });
    tout.sort_unstable_by_key(|&(i, _)| i);
    tout.into_iter().map(|(_, r)| r).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_resultats_gardent_l_ordre_des_elements() {
        let elements: Vec<u32> = (0..1000).collect();
        let carres = appliquer(&elements, |&n| n * n);
        let attendu: Vec<u32> = elements.iter().map(|&n| n * n).collect();
        assert_eq!(carres, attendu);
    }

    #[test]
    fn peu_d_elements_ou_aucun() {
        assert!(appliquer(&[] as &[u8], |&n| n).is_empty());
        assert_eq!(appliquer(&[7u8], |&n| n + 1), [8]);
        assert_eq!(appliquer(&[1u8, 2, 3], |&n| n * 2), [2, 4, 6]);
    }

    #[test]
    fn les_fils_d_aide_sont_rendus() {
        // Après un appel, tout ce qui a été réservé ICI est rendu : on peut de
        // nouveau réserver autant qu'avant (d'autres tests tournent peut-être en
        // même temps, d'où la comparaison à une réservation, pas à zéro).
        let elements: Vec<u64> = (0..64).collect();
        let somme: u64 = appliquer(&elements, |&n| n).iter().sum();
        assert_eq!(somme, 63 * 64 / 2);
        assert!(Jeton::prendre(0).is_empty());
    }

    #[test]
    fn un_appel_imbrique_donne_le_meme_resultat() {
        // `f` appelle elle-même `appliquer` (comme un essai de la mesure qui
        // cherche un détour) : avec ou sans fil libre, le résultat est le même.
        let lignes: Vec<u64> = (0..16).collect();
        let colonnes: Vec<u64> = (0..16).collect();
        let sommes = appliquer(&lignes, |&l| {
            appliquer(&colonnes, |&c| l * 16 + c).iter().sum::<u64>()
        });
        let attendu: Vec<u64> = lignes
            .iter()
            .map(|&l| colonnes.iter().map(|&c| l * 16 + c).sum())
            .collect();
        assert_eq!(sommes, attendu);
    }

    #[test]
    fn une_panique_remonte_a_l_appelant() {
        let elements: Vec<u32> = (0..64).collect();
        let resultat = std::panic::catch_unwind(|| {
            appliquer(&elements, |&n| {
                assert!(n != 40, "plan 40");
                n
            })
        });
        assert!(resultat.is_err());
        // Et les fils réservés sont rendus quand même (`Drop`).
        assert_eq!(appliquer(&elements, |&n| n).len(), 64);
    }
}
