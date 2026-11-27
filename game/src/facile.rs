//! Mode FACILE du Grand Nuage (carte 8.14) : un boss à `PV_NUAGE_FACILE` points de
//! vie (2 au lieu de 3), PROPOSÉ (jamais imposé) à qui est mort
//! `DEFAITES_AVANT_FACILE` fois d'affilée PENDANT SA VISITE du niveau (carte 9.34).
//!
//! Les règles :
//! - une DÉFAITE est une mort de Pépin dans un niveau à boss (Grand Nuage ou Pieuvre),
//!   en mode normal, hors démo du titre et hors course contre le robot ;
//! - le compte qui décide est celui de la VISITE (`EtatFacile::defaites_visite`) : il
//!   repart de zéro à chaque entrée dans le niveau (carte des niveaux, CONTINUER,
//!   suite de la partie), mais « recommencer » et « plus de vies » restent dans la
//!   même visite (sinon, avec 5 vies, on n'atteindrait jamais 10 défaites) ;
//! - à partir de `DEFAITES_AVANT_FACILE` défaites de la visite, après une mort au
//!   boss, une bannière propose « TROP DUR ? MODE FACILE » : OUI recommence le niveau
//!   en facile, NON reprend le combat là où il en était (en normal) et la bannière ne
//!   revient plus de la visite ;
//! - un boss déjà battu en NORMAL ne fait jamais proposer le facile (ni en jeu, ni
//!   sur la carte des niveaux) ;
//! - sur l'étape du boss de la carte des niveaux (après JOUER), le choix n'est
//!   proposé d'emblée qu'au joueur qui a DÉJÀ vaincu ce boss en facile, tant qu'il ne
//!   l'a pas battu en normal (`proposer`) : un raccourci, pas un jugement sur des
//!   défaites anciennes ;
//! - le compte ENREGISTRÉ (`ProgresNiveau::defaites`, dans la sauvegarde) est gardé
//!   tel quel pour ne pas changer le format : il compte toujours les défaites depuis
//!   la dernière victoire, mais ce n'est plus qu'une statistique, il ne déclenche
//!   plus rien (avant la carte 9.34, une vieille session de 10 défaites faisait
//!   proposer le facile dès la carte des niveaux ou la 1re mort du retour) ;
//! - RÉCOMPENSES (règle de la carte 7.9, chaque récompense une seule fois) : vaincre
//!   en facile compte comme niveau FINI (`ProgresNiveau::facile` : la suite s'ouvre),
//!   les gouttes ramassées sont créditées comme d'habitude, mais ni défi, ni
//!   médaille, ni meilleur temps du niveau, ni record de partie complète
//!   (`noter_partie`) ; battre le boss plus tard en normal donne la médaille comme
//!   d'habitude.
//!
//! Ce module est PUR (aucun dessin) : la bannière est dans `ecrans/facile.rs`.
//!
//! Notions Rust illustrées ici : `enum` avec donnée (`Origine::Plan(usize)`),
//! `Option` pour « une bannière, ou rien », fermeture passée en paramètre
//! (`impl Fn`) pour les zones cliquables fournies par l'écran.

use pepin_core::zones::InfosNiveau;

use crate::defis::{defis_du_niveau, ResultatNiveau, Tentative};
use crate::navigation::Navigation;
use crate::sauvegarde::{ProgresNiveau, Sauvegarde};

/// Défaites d'affilée au boss avant que le mode facile soit proposé.
pub const DEFAITES_AVANT_FACILE: u32 = 10;
/// Après une mort, la bannière ignore les appuis pendant une demi-seconde : un saut
/// tapé dans l'élan ne répond pas à la place du joueur.
pub const ATTENTE_PAS: u16 = 30;

/// Le mode facile est-il proposé sur ce niveau (carte des niveaux) ? Seulement si le
/// boss a déjà été vaincu en facile et pas encore en normal (carte 9.34). Les
/// défaites enregistrées (`p.defaites`) ne comptent plus : seules celles de la
/// visite en cours ouvrent la bannière, en jeu (`proposer_apres_une_mort`).
pub fn proposer(p: &ProgresNiveau) -> bool {
    !p.battu_en_normal() && p.facile
}

/// Ce qui décide de la bannière après une mort au boss.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApresUneMort {
    /// Défaites au boss dans la visite en cours (`EtatFacile::defaites_visite`).
    pub defaites_visite: u32,
    /// NON a déjà été répondu dans cette visite.
    pub refusee: bool,
    /// C'était la dernière vie : l'écran « plus de vies » passe d'abord.
    pub plus_de_vies: bool,
    /// Le boss a déjà été battu en mode normal (`ProgresNiveau::battu_en_normal`).
    pub battu_en_normal: bool,
}

/// Après une mort au boss : la bannière s'ouvre-t-elle ? Oui à partir de
/// `DEFAITES_AVANT_FACILE` défaites dans la visite en cours, sauf après un NON dans
/// cette visite, sur la dernière vie, ou si le boss a déjà été battu en normal (il
/// n'y a plus rien à débloquer : jamais de proposition, carte 9.34).
pub fn proposer_apres_une_mort(m: ApresUneMort) -> bool {
    m.defaites_visite >= DEFAITES_AVANT_FACILE
        && !m.refusee
        && !m.plus_de_vies
        && !m.battu_en_normal
}

/// Une mort au boss (mode normal) : une défaite de plus dans le compte ENREGISTRÉ du
/// niveau de clé `cle` (une statistique depuis la carte 9.34 : elle ne décide plus
/// de rien). Renvoie ce compte (0 hors de la table : niveaux d'essai). N'enregistre
/// pas.
pub fn noter_defaite(s: &mut Sauvegarde, cle: &str) -> u32 {
    s.progres_mut(cle).map_or(0, |p| {
        p.defaites = p.defaites.saturating_add(1);
        p.defaites
    })
}

/// Le boss vaincu en MODE FACILE : le niveau est fini (`facile`), ses défaites
/// repartent de zéro, ses gouttes NOUVELLES sont créditées (règle 7.9) ; ni défi, ni
/// médaille, ni meilleur temps. N'enregistre pas.
pub fn noter_victoire_facile(
    s: &mut Sauvegarde,
    infos: &InfosNiveau,
    t: &Tentative,
) -> ResultatNiveau {
    let (mut gain, mut deja_comptees) = (0, 0);
    if let Some(p) = s.progres_mut(infos.cle) {
        p.atteint = true;
        p.facile = true;
        p.defaites = 0;
        for (deja, &trouvee) in p.gouttes.iter_mut().zip(&t.gouttes) {
            if trouvee && *deja {
                deja_comptees += 1;
            } else if trouvee {
                *deja = true;
                gain += 1;
            }
        }
    }
    s.ajouter_gouttes(gain);
    ResultatNiveau {
        nom: infos.nom,
        defis: defis_du_niveau(infos),
        reussis: [false; 3],
        medaille: 0,
        gain,
        deja_comptees,
        facile: true,
    }
}

/// Une partie finie. Avec un boss vaincu en facile (`facile`), elle compte comme
/// finie mais ne bat aucun record (`false` : pas de « nouveau record »).
pub fn noter_partie(s: &mut Sauvegarde, score: u32, temps: u32, facile: bool) -> bool {
    if facile {
        s.parties_finies = s.parties_finies.saturating_add(1);
        false
    } else {
        s.noter_partie(score, temps)
    }
}

/// D'où vient la bannière.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origine {
    /// Après une mort au boss, en jeu (le monde est figé dessous).
    Mort,
    /// Après JOUER sur l'étape `indice` de la carte des niveaux.
    Plan(usize),
}

/// Ce que l'application doit faire après un appui sur la bannière.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reponse {
    Rien,
    Oui,
    Non,
    /// Échap sur la carte des niveaux : on n'y joue pas.
    Annuler,
}

/// La bannière « TROP DUR ? » ouverte : son origine et le bouton choisi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Proposition {
    pub origine: Origine,
    /// OUI choisi (sinon NON). NON au départ : un appui distrait ne passe jamais en
    /// facile.
    pub oui: bool,
    /// Pas restants pendant lesquels les appuis sont ignorés (`ATTENTE_PAS` après
    /// une mort, 0 sur la carte des niveaux).
    pub attente: u16,
}

impl Proposition {
    pub fn new(origine: Origine) -> Self {
        let attente = match origine {
            Origine::Mort => ATTENTE_PAS,
            Origine::Plan(_) => 0,
        };
        Self {
            origine,
            oui: false,
            attente,
        }
    }

    /// Un pas de jeu figé sous la bannière : l'attente s'écoule.
    pub fn attendre(&mut self) {
        self.attente = self.attente.saturating_sub(1);
    }

    /// Un appui : gauche / haut choisissent OUI, droite / bas NON, Entrée valide ;
    /// un clic ou un toucher sur un bouton (`bouton_sous` : `Some(true)` pour OUI)
    /// répond aussitôt, le survol le choisit ; Échap répond NON (après une mort) ou
    /// annule (carte des niveaux).
    pub fn reagir(
        &mut self,
        nav: &Navigation,
        bouton_sous: impl Fn(f32, f32) -> Option<bool>,
    ) -> Reponse {
        if self.attente > 0 {
            return Reponse::Rien;
        }
        if nav.retour {
            return match self.origine {
                Origine::Mort => Reponse::Non,
                Origine::Plan(_) => Reponse::Annuler,
            };
        }
        if nav.gauche || nav.haut {
            self.oui = true;
        }
        if nav.droite || nav.bas {
            self.oui = false;
        }
        if let Some(oui) = nav.survol.and_then(|(x, y)| bouton_sous(x, y)) {
            self.oui = oui;
        }
        if let Some(oui) = nav.clic.and_then(|(x, y)| bouton_sous(x, y)) {
            self.oui = oui;
            return if oui { Reponse::Oui } else { Reponse::Non };
        }
        match (nav.valider, self.oui) {
            (true, true) => Reponse::Oui,
            (true, false) => Reponse::Non,
            (false, _) => Reponse::Rien,
        }
    }
}

/// L'état du mode facile dans l'application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EtatFacile {
    /// Le niveau en cours a son boss en facile (gardé par « recommencer » et « plus
    /// de vies »).
    pub actif: bool,
    /// Le prochain niveau chargé (carte des niveaux, OUI) sera en facile.
    pub au_prochain: bool,
    /// Un boss a été joué en facile dans la partie en cours : pas de record.
    pub dans_la_partie: bool,
    /// NON a été répondu : plus de bannière dans cette visite du niveau.
    pub refusee: bool,
    /// Défaites au boss pendant la visite en cours du niveau (carte 9.34) : remis à
    /// zéro à chaque entrée dans le niveau (`Application::recharger`), gardé par
    /// « recommencer » et « plus de vies ». C'est lui qui ouvre la bannière.
    pub defaites_visite: u32,
    /// La bannière ouverte, s'il y en a une.
    pub proposition: Option<Proposition>,
}
