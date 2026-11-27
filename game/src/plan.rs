//! Carte des niveaux (carte 7.4) : choisir un niveau pour le REJOUER et s'améliorer
//! (meilleur temps, gouttes, défis, médaille). Une carte VERTICALE qui monte des
//! toits (en bas) jusqu'au soleil (en haut) : les niveaux normaux sont les étapes
//! du chemin qui monte, le bonus est une branche secrète à gauche près du soleil,
//! l'épilogue redescend de l'autre côté (à droite).
//!
//! Ce module est PUR (aucun dessin) : il dit où est chaque étape (`Geometrie`),
//! si elle est jouable (`etat_etape`, d'après la progression de la sauvegarde,
//! rangée par CLÉ de niveau depuis la carte 7.9) et
//! ce que fait un appui (`EtatPlan::reagir`). L'écran est dans `ecrans/plan.rs`.
//!
//! Tout se déduit de la table `NIVEAUX` (genre de chaque niveau) : aucun numéro ni
//! nombre de niveaux en dur. Un niveau normal ajouté à la table devient une étape
//! de plus sur le chemin, la carte s'allonge toute seule.
//!
//! Notions Rust illustrées ici : `enum` d'états, itérateurs (`filter`, `position`,
//! `rev`), `f32::clamp`, trait pour les zones cliquables (`Zones`) que l'écran
//! fournit (la logique ne connaît pas la mise en page exacte).

use pepin_core::levels::{debut_monde, gouttes_pour_le_bonus, niveaux_du_monde, GenreNiveau};
use pepin_core::zones::InfosNiveau;

use crate::navigation::Navigation;
use crate::sauvegarde::{ProgresNiveau, Sauvegarde};

/// Ce qu'une étape montre et permet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtatEtape {
    /// Pas encore atteinte : grisée, avec un cadenas ; pas jouable.
    Verrouillee,
    /// Le bonus tant qu'il n'est pas débloqué : « ? », nom caché ; pas jouable.
    Cachee,
    /// Jouable, jamais finie.
    Ouverte,
    /// Finie au moins une fois (meilleur temps, médaille ou défi réussi :
    /// `ProgresNiveau::fini`).
    Finie,
}

impl EtatEtape {
    pub fn jouable(self) -> bool {
        matches!(self, EtatEtape::Ouverte | EtatEtape::Finie)
    }
}

/// Progression de l'étape `i` de `niveaux`, lue par la CLÉ du niveau (vide hors de
/// la table).
pub fn progres(niveaux: &[InfosNiveau], s: &Sauvegarde, i: usize) -> ProgresNiveau {
    niveaux.get(i).map(|n| s.progres(n.cle)).unwrap_or_default()
}

/// Le niveau `i` a-t-il été fini au moins une fois (`ProgresNiveau::fini`) ?
fn fini(niveaux: &[InfosNiveau], s: &Sauvegarde, i: usize) -> bool {
    progres(niveaux, s, i).fini()
}

/// Site de test (AGA, 2026-11-19 : « pourquoi tout est pas débloqué sur
/// dev.elsay.dev, c'est l'environnement de test ») : le build de QA ouvre TOUTE la
/// carte des niveaux, Monde 2 compris, SANS rien écrire de faux dans la sauvegarde
/// (records, gouttes et médailles restent les vrais). Le drapeau est levé par `main`
/// au démarrage du navigateur seulement : les tests ne le lèvent jamais, ils gardent
/// les vraies règles. En production (elsay.dev), la fonctionnalité `debug_niveaux` est
/// absente : `tout_ouvert_qa` vaut toujours faux et la carte se débloque au fur et à
/// mesure.
///
/// `AtomicBool` : un booléen global qu'on peut lire et écrire sans `unsafe` (une
/// `static mut` l'exigerait). `Relaxed` suffit : un seul fil d'exécution le lit.
#[cfg(feature = "debug_niveaux")]
static TOUT_OUVERT_QA: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Lève le drapeau « tout ouvert » du site de test (appelé par `main`, build de QA).
#[cfg(feature = "debug_niveaux")]
pub fn ouvrir_tout_qa() {
    TOUT_OUVERT_QA.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// Toute la carte est-elle ouverte (site de test) ? Toujours faux en production.
fn tout_ouvert_qa() -> bool {
    #[cfg(feature = "debug_niveaux")]
    {
        TOUT_OUVERT_QA.load(std::sync::atomic::Ordering::Relaxed)
    }
    #[cfg(not(feature = "debug_niveaux"))]
    {
        false
    }
}

/// Calcule `f` avec les VRAIES règles de déblocage, même sur le site de test. Sert à
/// JOUER (« qu'il reprenne où j'en suis ») : sans cela, la carte tout ouverte du site
/// de test le ferait toujours partir au dernier niveau du Monde 2.
pub fn selon_la_vraie_progression<T>(f: impl FnOnce() -> T) -> T {
    #[cfg(feature = "debug_niveaux")]
    {
        use std::sync::atomic::Ordering::Relaxed;
        let avant = TOUT_OUVERT_QA.swap(false, Relaxed);
        let r = f();
        TOUT_OUVERT_QA.store(avant, Relaxed);
        r
    }
    #[cfg(not(feature = "debug_niveaux"))]
    {
        f()
    }
}

/// La PORTE du monde `monde` (cartes 9.1 et 11.2) : le dernier niveau NORMAL du
/// monde précédent (ni son bonus ni son épilogue), à finir au moins une fois. Pour
/// le Monde 2, c'est « Le retour du Grand Nuage » (`poursuite`), la règle de la
/// carte 9.1 (test `la_porte_du_monde_2_est_poursuite`). Une donnée lue dans la
/// table, aucun numéro de monde écrit : la porte d'un Monde 3 serait le dernier
/// niveau normal du Monde 2. `None` pour le Monde 1 (il n'a pas de porte) ou si le
/// monde précédent n'a aucun niveau normal. Pure.
pub fn porte_du_monde(niveaux: &[InfosNiveau], monde: u8) -> Option<&InfosNiveau> {
    let precedent = monde.checked_sub(1).filter(|&m| m >= 1)?;
    niveaux_du_monde(niveaux, precedent)
        .iter()
        .rev()
        .find(|n| n.genre == GenreNiveau::Normal)
}

/// Le monde `monde` est-il ouvert ? Le Monde 1 toujours ; un monde suivant quand sa
/// porte (`porte_du_monde`) est finie. Tant qu'elle ne l'est pas, la carte des
/// niveaux ne montre pas ce monde et une partie ne peut pas s'y lancer. Pure (la
/// sauvegarde est en lecture).
pub fn monde_ouvert(niveaux: &[InfosNiveau], s: &Sauvegarde, monde: u8) -> bool {
    if monde <= 1 || tout_ouvert_qa() {
        return true;
    }
    porte_du_monde(niveaux, monde).is_some_and(|n| s.progres(n.cle).fini())
}

/// Le monde `monde` peut-il se jouer (carte 9.1) ? Le Monde 1 toujours ; un monde
/// suivant seulement s'il a au moins un niveau dans `niveaux` ET que sa porte est
/// ouverte (`monde_ouvert`). C'est la règle de la carte des niveaux (le monde y
/// apparaît) et de JOUER au titre (carte 9.1b : il reprend au dernier monde
/// ouvert). Pure.
pub fn monde_jouable(niveaux: &[InfosNiveau], s: &Sauvegarde, monde: u8) -> bool {
    monde == 1 || (!niveaux_du_monde(niveaux, monde).is_empty() && monde_ouvert(niveaux, s, monde))
}

/// Le dernier monde jouable (cartes 9.1b et 11.2) : celui où JOUER reprend. Les
/// mondes s'ouvrent l'un après l'autre (`porte_du_monde`), on monte donc tant que le
/// suivant est jouable. Le Monde 1 au moins. Pure.
pub fn dernier_monde_jouable(niveaux: &[InfosNiveau], s: &Sauvegarde) -> u8 {
    (2..=u8::MAX)
        .take_while(|&m| monde_jouable(niveaux, s, m))
        .last()
        .unwrap_or(1)
}

/// Le joueur est-il déjà allé AU-DELÀ de l'étape `i` de la tranche `niveaux`
/// (carte 8.13) ? Oui s'il a fini une partie, ou si une étape de la tranche plus
/// loin (normal ou épilogue, pas le bonus) est déjà atteinte ou finie. Un niveau
/// inséré dans la table après coup (`poursuite`, carte 8.2) ne reste ainsi pas
/// cadenassé pour qui l'a dépassé. La recherche reste dans la tranche (le MONDE) :
/// un niveau du Monde 2 ne compte pas comme « dépassé » pour le Monde 1, et
/// réciproquement.
///
/// AGA (« enlève le déblocage de toute la carte et fais en sorte qu'on débloque la
/// carte au fur et à mesure ») : une partie finie n'ouvre d'un coup que le MONDE 1.
/// Avant, elle ouvrait aussi toute la tranche du Monde 2 (le compteur
/// `parties_finies` ne connaît pas les mondes) ; le Monde 2 s'ouvre maintenant niveau
/// après niveau, comme le Monde 1 la première fois.
fn deja_depasse(niveaux: &[InfosNiveau], s: &Sauvegarde, i: usize) -> bool {
    let monde_1 = niveaux.first().is_some_and(|n| n.monde == 1);
    (monde_1 && s.parties_finies > 0)
        || niveaux.iter().enumerate().skip(i + 1).any(|(j, n)| {
            n.genre != GenreNiveau::Bonus && (s.progres(n.cle).atteint || fini(niveaux, s, j))
        })
}

/// Gouttes TROUVÉES (au cumul de la sauvegarde) dans les niveaux normaux DE LA
/// TRANCHE passée : la règle du bonus s'applique PAR MONDE (carte 9.1), on passe donc
/// la tranche du monde (`niveaux_du_monde`).
pub fn gouttes_trouvees_normales(niveaux: &[InfosNiveau], s: &Sauvegarde) -> u32 {
    niveaux
        .iter()
        .filter(|n| n.genre == GenreNiveau::Normal)
        .map(|n| s.progres(n.cle).gouttes.iter().filter(|&&g| g).count() as u32)
        .sum()
}

/// État de l'étape `local` de la tranche `niveaux` (UN monde, indices locaux) :
/// - la première étape est toujours ouverte ;
/// - une étape normale ou l'épilogue est ouverte si elle a été atteinte
///   (`ProgresNiveau::atteint`) ou si l'étape qui la précède sur le chemin (bonus
///   sauté) a été finie : l'épilogue s'ouvre donc une fois la dernière étape normale
///   du monde finie ;
/// - une étape normale est aussi ouverte si le joueur l'a déjà dépassée
///   (`deja_depasse`, restreint au monde) ;
/// - le bonus se révèle avec TOUTES les gouttes des étapes normales DU MONDE
///   trouvées (au cumul de la sauvegarde, la règle de `levels::bonus_debloque`), ou
///   s'il a déjà été fini ; sinon il reste caché.
fn etat_etape_monde(niveaux: &[InfosNiveau], s: &Sauvegarde, i: usize) -> EtatEtape {
    let Some(n) = niveaux.get(i) else {
        return EtatEtape::Verrouillee;
    };
    // Site de test : toutes les étapes ouvertes, bonus compris.
    if tout_ouvert_qa() {
        return if fini(niveaux, s, i) {
            EtatEtape::Finie
        } else {
            EtatEtape::Ouverte
        };
    }
    let ouverte = match n.genre {
        GenreNiveau::Bonus => {
            let toutes = gouttes_trouvees_normales(niveaux, s) >= gouttes_pour_le_bonus(niveaux);
            if !(toutes || fini(niveaux, s, i)) {
                return EtatEtape::Cachee;
            }
            true
        }
        GenreNiveau::Normal | GenreNiveau::Epilogue => {
            let precedent = (0..i)
                .rev()
                .find(|&j| niveaux[j].genre != GenreNiveau::Bonus);
            i == 0
                || s.progres(n.cle).atteint
                || precedent.is_some_and(|j| fini(niveaux, s, j))
                || (n.genre == GenreNiveau::Normal && deja_depasse(niveaux, s, i))
        }
    };
    match (ouverte, fini(niveaux, s, i)) {
        (_, true) => EtatEtape::Finie,
        (true, false) => EtatEtape::Ouverte,
        (false, false) => EtatEtape::Verrouillee,
    }
}

/// État de l'étape `i` de la table `niveaux` (indice TABLE) d'après la sauvegarde :
/// la règle du MONDE de l'étape (les indices sont convertis en indices locaux de la
/// tranche du monde), et, pour tout monde autre que le Monde 1, la PORTE
/// (`monde_ouvert`) : tant que la porte du monde n'est pas finie (pour le Monde 2,
/// « poursuite »), toute la tranche du monde est verrouillée (cartes 9.1 et 11.2).
pub fn etat_etape(niveaux: &[InfosNiveau], s: &Sauvegarde, i: usize) -> EtatEtape {
    let Some(n) = niveaux.get(i) else {
        return EtatEtape::Verrouillee;
    };
    // Règle générale « il existe un monde précédent » (carte 11.2) : `monde_ouvert`
    // laisse toujours passer le Monde 1 et lit la porte de tout monde suivant.
    if !monde_ouvert(niveaux, s, n.monde) {
        return EtatEtape::Verrouillee;
    }
    let Some(debut) = debut_monde(niveaux, n.monde) else {
        return EtatEtape::Verrouillee;
    };
    let tranche = niveaux_du_monde(niveaux, n.monde);
    etat_etape_monde(tranche, s, i - debut)
}

/// Progression de mise au point (touche U du build de QA, carte 7.4) : premier appui,
/// les sept premiers niveaux finis (médailles et gouttes variées, bonus encore
/// caché) ; appui suivant, tous les niveaux finis avec toutes leurs gouttes (bonus
/// révélé). Seulement la progression : le porte-monnaie et les records ne bougent pas
/// (et ce qui est ainsi « trouvé » compte comme déjà crédité, carte 7.9).
pub fn progression_qa(niveaux: &[InfosNiveau], s: &mut Sauvegarde) {
    let tout = (0..7).all(|i| fini(niveaux, s, i));
    let atteint = if tout { niveaux.len() - 1 } else { 7 };
    // On ne fait qu'AJOUTER à la progression déjà là (rien de vrai n'est effacé).
    for (i, n) in niveaux.iter().enumerate() {
        let Some(p) = s.progres_mut(n.cle) else {
            continue;
        };
        p.atteint |= i <= atteint;
        let (gouttes, temps, medaille) = if tout {
            ([true; 3], 45 * 60 + i as u32 * 97, 1 + (i % 3) as u8)
        } else if i < 7 {
            (
                [i % 3 != 2, true, i % 2 == 0],
                30 * 60 + i as u32 * 431,
                (i % 4) as u8,
            )
        } else {
            continue;
        };
        for (k, (g, d)) in p.gouttes.iter_mut().zip(&mut p.defis).enumerate() {
            *g |= gouttes[k];
            *d |= medaille as usize > k;
        }
        p.meilleur_temps = Some(p.meilleur_temps.map_or(temps, |t| t.min(temps)));
        p.medaille = p.medaille.max(medaille);
    }
}

// --- Géométrie de la carte (pixels du « monde » de la carte) ---------------------

/// Écart vertical entre deux niveaux normaux.
pub const ECART: f32 = 44.0;
/// Ordonnée du soleil (son centre) en haut de la carte.
pub const Y_SOLEIL: f32 = 26.0;
/// Le niveau normal le plus haut, sous le soleil.
const Y_SOMMET: f32 = 70.0;
/// Hauteur des toits sous la première étape.
const SOUS_PREMIERE: f32 = 26.0;
/// Hauteur des toits dessinés en bas de la carte.
pub const H_TOITS: f32 = 32.0;
/// Colonnes du chemin qui serpente (centre des étapes), puis la branche secrète du
/// bonus (à gauche) et la descente (à droite).
const COLONNES: [f32; 4] = [60.0, 92.0, 124.0, 92.0];
pub const X_BONUS: f32 = 22.0;
pub const X_DESCENTE: f32 = 154.0;

/// Où sont les étapes de la carte.
#[derive(Debug, Clone, PartialEq)]
pub struct Geometrie {
    /// Centre de chaque étape, dans l'ordre de la table.
    pub etapes: Vec<(f32, f32)>,
    /// Hauteur totale de la carte.
    pub hauteur: f32,
}

impl Geometrie {
    pub fn new(niveaux: &[InfosNiveau]) -> Self {
        let nb = niveaux
            .iter()
            .filter(|n| n.genre == GenreNiveau::Normal)
            .count()
            .max(1);
        let y_premier = Y_SOMMET + (nb - 1) as f32 * ECART;
        let hauteur = y_premier + SOUS_PREMIERE + H_TOITS;
        let (mut k, mut b, mut e) = (0usize, 0usize, 0usize);
        let etapes = niveaux
            .iter()
            .map(|n| match n.genre {
                GenreNiveau::Normal => {
                    let p = (COLONNES[k % COLONNES.len()], y_premier - k as f32 * ECART);
                    k += 1;
                    p
                }
                // La branche secrète part du sommet vers la gauche, un peu plus haut.
                GenreNiveau::Bonus => {
                    b += 1;
                    (X_BONUS, Y_SOMMET - 22.0 + (b - 1) as f32 * ECART)
                }
                // La descente : de l'autre côté du soleil, deux étages plus bas.
                GenreNiveau::Epilogue => {
                    e += 1;
                    (X_DESCENTE, Y_SOMMET + (e + 1) as f32 * ECART)
                }
            })
            .collect::<Vec<_>>();
        // Une carte plus basse que la vue (un monde de peu de niveaux, comme le faux
        // Monde 2 de QA, carte 9.1) laisserait le bas de la vue sans dessin : tout
        // descend pour la remplir (le soleil reste en haut, les toits en bas). Le Monde 1,
        // bien plus haut que la vue, ne bouge pas.
        let decalage = (H_VUE - hauteur).max(0.0);
        let etapes = etapes.into_iter().map(|(x, y)| (x, y + decalage)).collect();
        Self {
            etapes,
            hauteur: hauteur + decalage,
        }
    }
}

// --- Réactions aux appuis ---------------------------------------------------------

/// Hauteur visible de la carte à l'écran (entre le bandeau du haut et celui du bas).
pub const H_VUE: f32 = 152.0;
/// Défilement d'un cran de molette.
pub const PAS_MOLETTE: f32 = 24.0;

/// Les zones cliquables, fournies par l'écran (coordonnées logiques de l'écran).
pub trait Zones {
    /// L'étape sous le point (avec le défilement `defil`).
    fn etape_sous(&self, g: &Geometrie, defil: f32, x: f32, y: f32) -> Option<usize>;
    /// Le bouton JOUER contient-il le point ?
    fn sur_jouer(&self, x: f32, y: f32) -> bool;
    /// Le bouton COURSE (carte 8.7) contient-il le point ?
    fn sur_course(&self, x: f32, y: f32) -> bool;
    /// Le bouton RETOUR contient-il le point ?
    fn sur_retour(&self, x: f32, y: f32) -> bool;
    /// La flèche du chapitre précédent (« ◀ CHAPITRE 1 », carte 9.30) contient-elle
    /// le point ? (La flèche n'est là que si `chapitre_voisin` la montre.)
    fn sur_chapitre_precedent(&self, x: f32, y: f32) -> bool;
    /// La flèche du chapitre suivant (« CHAPITRE 2 ▶ ») contient-elle le point ?
    fn sur_chapitre_suivant(&self, x: f32, y: f32) -> bool;
}

/// Ce que l'application doit faire après un appui.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionPlan {
    Rien,
    /// Retour au titre.
    Quitter,
    /// Jouer le niveau d'indice donné (partie d'un seul niveau).
    Jouer(usize),
    /// Courir le niveau d'indice donné contre le robot (carte 8.7) : choisir la force.
    Course(usize),
}

/// L'étape `i` se court-elle contre le robot ? Jouable, et au moins une partie de
/// fantôme embarquée pour elle (carte 8.7).
pub fn course_possible(niveaux: &[InfosNiveau], s: &Sauvegarde, i: usize) -> bool {
    etat_etape(niveaux, s, i).jouable() && crate::course::disponible(i)
}

/// L'écran de la carte : étape choisie et défilement. La carte affiche UN MONDE à
/// la fois (carte 9.1) : `monde` est celui affiché, `geometrie` est celle de sa
/// tranche, et `selection` reste un indice DANS LA TABLE (pour charger le niveau).
#[derive(Debug, Clone, PartialEq)]
pub struct EtatPlan {
    pub geometrie: Geometrie,
    /// Le bouton choisi au clavier sous le panneau : COURSE (`true`) ou JOUER.
    pub course: bool,
    /// Indice (dans la table) de l'étape choisie.
    pub selection: usize,
    /// Monde affiché (1 = Monde 1, 2 = Monde 2...).
    pub monde: u8,
    /// Début (dans la table) de la tranche du monde affiché.
    pub debut_monde: usize,
    /// Ordonnée de la carte en haut de la vue.
    pub defil: f32,
}

impl EtatPlan {
    /// Ouvre la carte sur `selection` (bornée), dans le MONDE de cette étape, la vue
    /// centrée dessus.
    pub fn new(niveaux: &[InfosNiveau], selection: usize) -> Self {
        let selection = selection.min(niveaux.len().saturating_sub(1));
        let monde = niveaux.get(selection).map(|n| n.monde).unwrap_or(1);
        let debut_monde = debut_monde(niveaux, monde).unwrap_or(0);
        let mut e = Self {
            geometrie: Geometrie::new(niveaux_du_monde(niveaux, monde)),
            course: false,
            selection,
            monde,
            debut_monde,
            defil: 0.0,
        };
        e.centrer();
        e
    }

    /// Le niveau où l'on en est : le plus loin des niveaux jouables du chemin (le
    /// bonus n'y compte pas), DANS LE MONDE `monde`.
    pub fn ou_on_en_est_du_monde(niveaux: &[InfosNiveau], s: &Sauvegarde, monde: u8) -> usize {
        let debut = debut_monde(niveaux, monde).unwrap_or(0);
        let tranche = niveaux_du_monde(niveaux, monde);
        (0..tranche.len())
            .rev()
            .find(|&loc| {
                let i = debut + loc;
                niveaux[i].genre != GenreNiveau::Bonus && etat_etape(niveaux, s, i).jouable()
            })
            .map_or(debut, |loc| debut + loc)
    }

    /// Le niveau où l'on en est (tout monde confondu) : le plus loin des niveaux
    /// jouables du chemin (le bonus n'y compte pas).
    pub fn ou_on_en_est(niveaux: &[InfosNiveau], s: &Sauvegarde) -> usize {
        (0..niveaux.len())
            .rev()
            .find(|&i| {
                niveaux[i].genre != GenreNiveau::Bonus && etat_etape(niveaux, s, i).jouable()
            })
            .unwrap_or(0)
    }

    /// Change de MONDE affiché (touches Gauche/Droite, ou Haut/Bas au bout du chemin ;
    /// cartes 9.1 et 9.1b : la carte est UNE SEULE SUITE de chapitres, sans bouton) :
    /// la géométrie se reconstruit sur la tranche du monde, l'étape choisie est la
    /// plus avancée jouable de ce monde, et la vue est recentrée. `course` : le bouton
    /// (JOUER ou COURSE) dans ce monde. Un monde qui n'est pas jouable ne s'ouvre pas
    /// (renvoie `false`).
    pub fn aller_monde(
        &mut self,
        monde: u8,
        course: bool,
        niveaux: &[InfosNiveau],
        s: &Sauvegarde,
    ) -> bool {
        if !monde_jouable(niveaux, s, monde) {
            return false;
        }
        self.monde = monde;
        self.debut_monde = debut_monde(niveaux, monde).unwrap_or(0);
        self.geometrie = Geometrie::new(niveaux_du_monde(niveaux, monde));
        self.course = course;
        self.selection = Self::ou_on_en_est_du_monde(niveaux, s, monde);
        self.centrer();
        true
    }

    /// Le chapitre voisin (précédent, ou `suivant`) s'il est jouable : sa flèche
    /// tactile est alors dessinée et répond au toucher (carte 9.30).
    pub fn chapitre_voisin(
        &self,
        suivant: bool,
        niveaux: &[InfosNiveau],
        s: &Sauvegarde,
    ) -> Option<u8> {
        let monde = if suivant {
            self.monde.checked_add(1)?
        } else {
            self.monde.checked_sub(1).filter(|&m| m >= 1)?
        };
        monde_jouable(niveaux, s, monde).then_some(monde)
    }

    fn defil_max(&self) -> f32 {
        (self.geometrie.hauteur - H_VUE).max(0.0)
    }

    fn borner(&mut self) {
        self.defil = self.defil.clamp(0.0, self.defil_max()).round();
    }

    /// Centre la vue sur l'étape choisie (indice local dans la tranche du monde).
    pub fn centrer(&mut self) {
        let local = self.selection.saturating_sub(self.debut_monde);
        if let Some(&(_, y)) = self.geometrie.etapes.get(local) {
            self.defil = y - H_VUE / 2.0;
        }
        self.borner();
    }

    /// Un appui (une image) : haut / bas parcourent les étapes dans l'ordre du chemin,
    /// et la carte est UNE SEULE SUITE de chapitres (carte 9.1b, sans bouton de
    /// monde) : Haut sur la dernière étape d'un chapitre passe à la première du
    /// suivant, Bas sur la première revient à la dernière du précédent ; Gauche/Droite
    /// se déplacent entre JOUER/COURSE puis d'un chapitre à l'autre (Droite avance :
    /// de JOUER à COURSE, puis vers le chapitre suivant ; Gauche revient) ; Entrée,
    /// JOUER ou COURSE lance l'étape choisie si elle est jouable ; toucher une étape
    /// la choisit ; molette et glisser font défiler ; Échap ou RETOUR revient au
    /// titre. Sans Monde 2 jouable, tout se passe exactement comme avant.
    pub fn reagir(
        &mut self,
        nav: &Navigation,
        niveaux: &[InfosNiveau],
        s: &Sauvegarde,
        z: &impl Zones,
    ) -> ActionPlan {
        if nav.retour {
            return ActionPlan::Quitter;
        }
        if niveaux.is_empty() {
            return ActionPlan::Rien;
        }
        let nb = self.geometrie.etapes.len();
        let local = self.selection.saturating_sub(self.debut_monde);
        let jouer = |sel: usize| {
            if etat_etape(niveaux, s, sel).jouable() {
                ActionPlan::Jouer(sel)
            } else {
                ActionPlan::Rien
            }
        };
        let courir = |sel: usize| {
            if course_possible(niveaux, s, sel) {
                ActionPlan::Course(sel)
            } else {
                ActionPlan::Rien
            }
        };
        // Haut/Bas : parcourent les étapes du MONDE affiché ; au bout du chemin, on
        // passe au chapitre voisin (s'il est jouable), sur son étape la plus proche.
        if nav.haut {
            if local + 1 < nb {
                self.selection += 1;
                self.centrer();
            } else if self.aller_monde(self.monde + 1, self.course, niveaux, s) {
                self.selection = self.debut_monde;
                self.centrer();
            }
        }
        // Bas, au bout du chemin : `monde > 1` est la règle générale « il existe un
        // monde précédent » (carte 11.2) ; le Monde 1 est le premier.
        if nav.bas {
            if local > 0 {
                self.selection -= 1;
                self.centrer();
            } else if self.monde > 1 && self.aller_monde(self.monde - 1, self.course, niveaux, s) {
                self.selection = self.debut_monde + self.geometrie.etapes.len() - 1;
                self.centrer();
            }
        }
        // Droite : JOUER → COURSE, puis (en COURSE) vers le monde suivant.
        if nav.droite {
            if !self.course {
                self.course = true;
            } else {
                self.aller_monde(self.monde + 1, false, niveaux, s);
            }
        }
        // Gauche : COURSE → JOUER, puis (en JOUER, hors du Monde 1) vers le monde
        // précédent (en COURSE). `monde > 1` : la même règle générale « il existe un
        // monde précédent » que pour Bas (carte 11.2).
        if nav.gauche {
            if self.course {
                self.course = false;
            } else if self.monde > 1 {
                self.aller_monde(self.monde - 1, true, niveaux, s);
            }
        }
        if nav.molette != 0.0 {
            self.defil -= nav.molette * PAS_MOLETTE;
            self.borner();
        }
        if nav.glisse != 0.0 {
            self.defil -= nav.glisse;
            self.borner();
        }
        if let Some((x, y)) = nav.clic {
            if z.sur_retour(x, y) {
                return ActionPlan::Quitter;
            }
            if z.sur_jouer(x, y) {
                self.course = false;
                return jouer(self.selection);
            }
            if z.sur_course(x, y) {
                self.course = true;
                return courir(self.selection);
            }
            // Les flèches de chapitre (carte 9.30) : le même changement de monde que
            // Gauche/Droite au clavier. Sans chapitre voisin jouable, pas de flèche :
            // le toucher retombe sur la carte.
            for (suivant, sur) in [
                (false, z.sur_chapitre_precedent(x, y)),
                (true, z.sur_chapitre_suivant(x, y)),
            ] {
                if let Some(m) = self.chapitre_voisin(suivant, niveaux, s).filter(|_| sur) {
                    self.aller_monde(m, self.course, niveaux, s);
                    return ActionPlan::Rien;
                }
            }
            if let Some(i) = z.etape_sous(&self.geometrie, self.defil, x, y) {
                // Toucher choisit seulement (un glisser commence aussi par un appui) :
                // on lance avec JOUER ou Entrée. L'étape est un indice LOCAL de la
                // tranche du monde : on la ramène à un indice de table.
                self.selection = self.debut_monde + i;
            }
        }
        if nav.valider {
            return if self.course {
                courir(self.selection)
            } else {
                jouer(self.selection)
            };
        }
        ActionPlan::Rien
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::navigation::Navigation;
    use crate::sauvegarde::Sauvegarde;
    use pepin_core::zones::InfosNiveau;

    /// Table de test : le Monde 1 de la vraie table, puis UN niveau du Monde 2 (le
    /// caniveau, son 1er niveau). Une table à part : le vrai Monde 2 grandit avec
    /// chaque carte (9.6 à 9.13), ces tests ne doivent pas en dépendre.
    fn table_deux_mondes() -> Vec<InfosNiveau> {
        let mut t: Vec<InfosNiveau> = niveaux::niveaux_monde_1().to_vec();
        t.push(*niveaux::par_cle("caniveau"));
        t
    }

    /// Sauvegarde où « Le retour du Grand Nuage » est fini : la porte du Monde 2.
    fn poursuite_finie() -> Sauvegarde {
        Sauvegarde::depuis_texte("version=3\nniveau.poursuite=atteint:1 medaille:1")
    }

    /// Des zones où rien ne répond (la carte n'a plus de bouton de monde, 9.1b).
    struct SansZone;
    impl Zones for SansZone {
        fn etape_sous(&self, _: &Geometrie, _: f32, _: f32, _: f32) -> Option<usize> {
            None
        }
        fn sur_jouer(&self, _: f32, _: f32) -> bool {
            false
        }
        fn sur_course(&self, _: f32, _: f32) -> bool {
            false
        }
        fn sur_retour(&self, _: f32, _: f32) -> bool {
            false
        }
        fn sur_chapitre_precedent(&self, _: f32, _: f32) -> bool {
            false
        }
        fn sur_chapitre_suivant(&self, _: f32, _: f32) -> bool {
            false
        }
    }

    /// Des zones où tout point est sur les deux flèches de chapitre (carte 9.30).
    struct Fleches;
    impl Zones for Fleches {
        fn etape_sous(&self, _: &Geometrie, _: f32, _: f32, _: f32) -> Option<usize> {
            None
        }
        fn sur_jouer(&self, _: f32, _: f32) -> bool {
            false
        }
        fn sur_course(&self, _: f32, _: f32) -> bool {
            false
        }
        fn sur_retour(&self, _: f32, _: f32) -> bool {
            false
        }
        fn sur_chapitre_precedent(&self, _: f32, _: f32) -> bool {
            true
        }
        fn sur_chapitre_suivant(&self, _: f32, _: f32) -> bool {
            true
        }
    }

    /// Toucher la flèche de chapitre (carte 9.30) : chapitre 2 ouvert, du 1 on va au
    /// 2, puis du 2 on revient au 1, comme Droite/Gauche au clavier (`aller_monde`).
    #[test]
    fn toucher_la_fleche_change_de_chapitre() {
        let t = table_deux_mondes();
        let s = poursuite_finie();
        let mut e = EtatPlan::new(&t, 0);
        assert_eq!(e.chapitre_voisin(false, &t, &s), None, "pas de chapitre 0");
        assert_eq!(e.chapitre_voisin(true, &t, &s), Some(2));
        let clic = Navigation {
            clic: Some((10.0, 10.0)),
            ..Navigation::default()
        };
        assert_eq!(e.reagir(&clic, &t, &s, &Fleches), ActionPlan::Rien);
        let mut attendu = EtatPlan::new(&t, 0);
        attendu.aller_monde(2, false, &t, &s);
        assert_eq!(e, attendu);
        assert_eq!(e.chapitre_voisin(true, &t, &s), None, "pas de chapitre 3");
        e.reagir(&clic, &t, &s, &Fleches);
        assert_eq!(e.monde, 1);
    }

    /// Chapitre 2 fermé : pas de flèche, et toucher sa place ne change rien.
    #[test]
    fn pas_de_fleche_vers_un_chapitre_ferme() {
        let t = table_deux_mondes();
        let vide = Sauvegarde::default();
        let mut e = EtatPlan::new(&t, 0);
        assert_eq!(e.chapitre_voisin(true, &t, &vide), None);
        let avant = e.clone();
        let clic = Navigation {
            clic: Some((10.0, 10.0)),
            ..Navigation::default()
        };
        assert_eq!(e.reagir(&clic, &t, &vide, &Fleches), ActionPlan::Rien);
        assert_eq!(e, avant);
    }

    /// Les deux flèches de chapitre se chevauchent dans le panneau (carte 9.30) :
    /// avec la vraie table, jamais les deux à la fois (pas de chapitre 3).
    #[test]
    fn une_seule_fleche_de_chapitre_a_la_fois() {
        let s = poursuite_finie();
        for monde in [1, 2] {
            let mut e = EtatPlan::new(niveaux::niveaux(), 0);
            assert!(e.aller_monde(monde, false, niveaux::niveaux(), &s));
            let fleches = [false, true]
                .iter()
                .filter(|&&suivant| e.chapitre_voisin(suivant, niveaux::niveaux(), &s).is_some())
                .count();
            assert_eq!(fleches, 1, "chapitre {monde}");
        }
    }

    /// Une carte d'un seul niveau (le Monde 2 réduit au caniveau) remplit quand même la
    /// vue ; la carte du Monde 1, plus haute, garde sa géométrie d'avant.
    #[test]
    fn une_carte_courte_remplit_la_vue() {
        let g = Geometrie::new(&[*niveaux::par_cle("caniveau")]);
        assert_eq!(g.hauteur, H_VUE);
        assert_eq!(g.etapes[0].1, g.hauteur - SOUS_PREMIERE - H_TOITS);
        let m1 = Geometrie::new(niveaux::niveaux_monde_1());
        assert_eq!(m1.hauteur, Y_SOMMET + 9.0 * ECART + SOUS_PREMIERE + H_TOITS);
        assert_eq!(m1.etapes[0].1, Y_SOMMET + 9.0 * ECART);
    }

    /// La porte du Monde 2 : fermée tant que « poursuite » n'est pas fini ; le
    /// Monde 2 n'a alors ni étape ouverte, ni Droite ni Haut pour y aller.
    #[test]
    fn la_porte_du_monde_2_est_fermee_sans_poursuite_finie() {
        let t = table_deux_mondes();
        let idx = t.len() - 1;
        let vide = Sauvegarde::default();
        assert!(!monde_ouvert(&t, &vide, 2));
        assert!(!monde_jouable(&t, &vide, 2));
        assert_eq!(etat_etape(&t, &vide, idx), EtatEtape::Verrouillee);
        let mut e = EtatPlan::new(&t, idx - 1);
        let droite = Navigation {
            droite: true,
            haut: true,
            ..Navigation::default()
        };
        e.reagir(&droite, &t, &vide, &SansZone);
        e.reagir(&droite, &t, &vide, &SansZone);
        assert_eq!((e.monde, e.course), (1, true), "Droite s'arrête sur COURSE");
        assert_eq!(e.selection, idx - 1, "Haut s'arrête au bout du Monde 1");
        // La vraie table a un Monde 2 depuis la carte 9.6 (le caniveau) : porte
        // ouverte, il est jouable, avec ou sans le build de QA.
        assert!(monde_jouable(niveaux::niveaux(), &poursuite_finie(), 2));
    }

    /// Le Monde 1 est le seul monde ouvert pour qui commence : JOUER y reprend.
    /// « poursuite » fini, JOUER reprend au Monde 2 (carte 9.1b).
    #[test]
    fn jouer_reprend_au_dernier_monde_ouvert() {
        let t = table_deux_mondes();
        assert_eq!(dernier_monde_jouable(&t, &Sauvegarde::default()), 1);
        assert_eq!(dernier_monde_jouable(&t, &poursuite_finie()), 2);
    }

    /// Carte 11.2 : la porte du Monde 2, lue dans la table (le dernier niveau normal
    /// du Monde 1), est bien « Le retour du Grand Nuage », la règle de la carte 9.1.
    /// Le Monde 1 n'a pas de porte.
    #[test]
    fn la_porte_du_monde_2_est_poursuite() {
        assert_eq!(
            porte_du_monde(niveaux::niveaux(), 2).map(|n| n.cle),
            Some("poursuite")
        );
        assert!(porte_du_monde(niveaux::niveaux(), 1).is_none());
        assert!(porte_du_monde(niveaux::niveaux(), 0).is_none());
    }

    /// « poursuite » fini : le Monde 2 s'ouvre ; la carte est une seule suite (carte
    /// 9.1b) : on y va par Droite (après COURSE) ou Haut au bout du Monde 1, on en
    /// revient par Gauche ou Bas.
    #[test]
    fn le_monde_2_s_ouvre_quand_poursuite_est_finie() {
        let t = table_deux_mondes();
        let idx = t.len() - 1;
        let s = poursuite_finie();
        assert!(monde_ouvert(&t, &s, 2) && monde_jouable(&t, &s, 2));
        // La première étape du Monde 2 est ouverte, le Monde 1 n'a pas bougé.
        assert_eq!(etat_etape(&t, &s, idx), EtatEtape::Ouverte);
        assert_eq!(etat_etape(&t, &s, 0), EtatEtape::Ouverte);
        // Haut sur la dernière étape du Monde 1 : la première du Monde 2, avec la
        // géométrie de sa tranche.
        let mut e = EtatPlan::new(&t, idx - 1);
        let haut = Navigation {
            haut: true,
            ..Navigation::default()
        };
        assert_eq!(e.reagir(&haut, &t, &s, &SansZone), ActionPlan::Rien);
        assert_eq!((e.monde, e.debut_monde, e.selection), (2, idx, idx));
        assert_eq!(e.geometrie.etapes.len(), 1);
        // JOUER lance l'étape du Monde 2 par son indice DANS LA TABLE.
        let valider = Navigation {
            valider: true,
            ..Navigation::default()
        };
        assert_eq!(
            e.reagir(&valider, &t, &s, &SansZone),
            ActionPlan::Jouer(idx)
        );
        // Bas sur la première étape du Monde 2 : retour à la dernière du Monde 1.
        let bas = Navigation {
            bas: true,
            ..Navigation::default()
        };
        e.reagir(&bas, &t, &s, &SansZone);
        assert_eq!((e.monde, e.selection), (1, idx - 1));
        // Droite (JOUER → COURSE → chapitre suivant) puis Gauche (au Monde 2 : retour
        // au Monde 1, sur COURSE).
        let droite = Navigation {
            droite: true,
            ..Navigation::default()
        };
        e.reagir(&droite, &t, &s, &SansZone);
        e.reagir(&droite, &t, &s, &SansZone);
        assert_eq!((e.monde, e.course), (2, false));
        let gauche = Navigation {
            gauche: true,
            ..Navigation::default()
        };
        e.reagir(&gauche, &t, &s, &SansZone);
        assert_eq!((e.monde, e.course), (1, true));
        // La carte rouverte sur une étape du Monde 2 montre le Monde 2.
        assert_eq!(EtatPlan::new(&t, idx).monde, 2);
    }
}
