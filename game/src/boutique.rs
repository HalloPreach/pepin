//! Boutique (carte 7.5) : on y dépense les gouttes d'or du porte-monnaie (sauvegarde,
//! carte 7.2) pour des objets UNIQUEMENT cosmétiques (demande d'AGA : le score reste
//! comparable d'une partie à l'autre). Rien ici ne touche à la physique ni aux règles.
//!
//! Organisation à ONGLETS : un onglet = une `Categorie`, avec sa liste d'`Objet`s
//! (identifiant, nom, prix, aperçu ; débloqué ? équipé ? se lisent dans la sauvegarde).
//! Quatre onglets : SKINS (carte 7.1), puis TRAÎNÉES, PARAPLUIES (motifs) et MUSIQUES
//! (carte 7.6, catalogues et prix dans `cosmetiques` ; l'onglet du choix de
//! palette a été retiré à la carte 7.6b). Chaque catégorie a
//! son préfixe dans l'inventaire de la sauvegarde (« trainee:bulles »…) ; ajouter une
//! catégorie = une variante de `Categorie`, sa ligne dans `TOUTES` et dans chaque
//! `match`. Un onglet sans objet n'est pas affiché (`onglets`).
//!
//! Ce module est PUR (aucun dessin) : l'écran est dans `ecrans/boutique.rs`, et
//! `app.rs` ne fait qu'appeler `EtatBoutique::reagir` puis appliquer l'`Action`.
//!
//! Notions Rust illustrées ici : `enum` porteur de données (`Apercu`, `Dialogue`),
//! `#[cfg(feature = ...)]` sur une variante et sur un élément de liste (objet factice
//! présent seulement dans le build de QA), `Option::take`.

use crate::cosmetiques::{self as cosm, Motif, Trainee};
use crate::menu::Menu;
use crate::musiques::Musique;
use crate::navigation::Navigation;
use crate::sauvegarde::Sauvegarde;
use crate::skins::Skin;

/// Une catégorie d'objets : un onglet de la boutique.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Categorie {
    Skins,
    Trainees,
    Motifs,
    Musiques,
}

impl Categorie {
    /// Toutes les catégories, dans l'ordre des onglets.
    pub const TOUTES: [Categorie; 4] = [
        Categorie::Skins,
        Categorie::Trainees,
        Categorie::Motifs,
        Categorie::Musiques,
    ];

    /// Nom de l'onglet.
    pub fn nom(self) -> &'static str {
        match self {
            Categorie::Skins => "SKINS",
            Categorie::Trainees => "TRAÎNÉES",
            Categorie::Motifs => "PARAPLUIES",
            Categorie::Musiques => "MUSIQUES",
        }
    }

    /// Préfixe des objets de la catégorie dans la sauvegarde (« categorie:id »).
    pub fn prefixe(self) -> &'static str {
        match self {
            Categorie::Skins => "skin",
            Categorie::Trainees => "trainee",
            Categorie::Motifs => "motif",
            Categorie::Musiques => "musique",
        }
    }

    /// Les objets de la catégorie, dans l'ordre d'affichage.
    pub fn objets(self) -> Vec<Objet> {
        match self {
            Categorie::Skins => {
                #[allow(unused_mut)] // pas mutable sans le build de QA
                let mut v: Vec<Objet> = Skin::TOUS
                    .into_iter()
                    .map(|s| Objet {
                        categorie: self,
                        id: s.id(),
                        nom: s.nom(),
                        prix: s.prix(),
                        apercu: Apercu::Skin(s),
                    })
                    .collect();
                // Build de QA (et tests) seulement : un objet payant factice, pour
                // essayer l'achat sans ajouter un vrai skin payant au jeu.
                #[cfg(any(test, feature = "debug_niveaux"))]
                v.push(Objet {
                    categorie: self,
                    id: "factice_qa",
                    nom: "FACTICE QA",
                    prix: 60,
                    apercu: Apercu::Factice,
                });
                v
            }
            // Le choix « rien » d'abord (gratuit), puis le catalogue.
            Categorie::Trainees => {
                let mut v =
                    vec![self.objet(cosm::SANS_TRAINEE, "AUCUNE", 0, Apercu::Trainee(None))];
                v.extend(Trainee::TOUTES.map(|t| {
                    self.objet(
                        t.id(),
                        t.nom(),
                        cosm::PRIX_TRAINEE,
                        Apercu::Trainee(Some(t)),
                    )
                }));
                v
            }
            Categorie::Motifs => {
                let mut v = vec![self.objet(cosm::SANS_MOTIF, "UNI", 0, Apercu::Motif(None))];
                v.extend(Motif::TOUS.map(|m| {
                    self.objet(m.id(), m.nom(), cosm::PRIX_MOTIF, Apercu::Motif(Some(m)))
                }));
                v
            }
            Categorie::Musiques => {
                let mut v =
                    vec![self.objet(cosm::MUSIQUE_PAR_ZONE, "PAR ZONE", 0, Apercu::Musique(None))];
                v.extend(
                    Musique::TOUTES
                        .map(|m| self.objet(m.id(), m.nom(), m.prix(), Apercu::Musique(Some(m)))),
                );
                v
            }
        }
    }

    fn objet(self, id: &'static str, nom: &'static str, prix: u32, apercu: Apercu) -> Objet {
        Objet {
            categorie: self,
            id,
            nom,
            prix,
            apercu,
        }
    }
}

/// Les onglets affichés : les catégories qui ont au moins un objet.
pub fn onglets() -> Vec<Categorie> {
    Categorie::TOUTES
        .into_iter()
        .filter(|c| !c.objets().is_empty())
        .collect()
}

/// Ce que montre l'aperçu d'un objet (et ce qu'on équipe).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Apercu {
    /// Un skin : aperçu animé (course).
    Skin(Skin),
    /// Une traînée derrière Pépin qui court (`None` : aucune).
    Trainee(Option<Trainee>),
    /// Le parapluie ouvert avec ce motif (`None` : uni).
    Motif(Option<Motif>),
    /// Une note de musique (`None` : la musique de chaque zone).
    Musique(Option<Musique>),
    /// L'objet factice du build de QA : une goutte d'or, rien à équiper.
    #[cfg(any(test, feature = "debug_niveaux"))]
    Factice,
}

/// Un objet de la boutique.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Objet {
    pub categorie: Categorie,
    /// Identifiant enregistré dans la sauvegarde.
    pub id: &'static str,
    pub nom: &'static str,
    /// Prix en gouttes d'or (0 : gratuit, débloqué d'office).
    pub prix: u32,
    pub apercu: Apercu,
}

/// État d'un objet pour le joueur.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Etat {
    AAcheter,
    Achete,
    Equipe,
}

impl Objet {
    /// Identifiant dans l'inventaire de la sauvegarde : « categorie:id ».
    pub fn cle(&self) -> String {
        format!("{}:{}", self.categorie.prefixe(), self.id)
    }

    /// L'objet est-il à lui (acheté ou gratuit) ?
    pub fn debloque(&self, s: &Sauvegarde) -> bool {
        self.prix == 0 || s.possede(&self.cle())
    }

    /// L'objet est-il celui en service ?
    pub fn equipe(&self, s: &Sauvegarde) -> bool {
        s.equipe(self.categorie.prefixe()) == Some(self.id)
    }

    pub fn etat(&self, s: &Sauvegarde) -> Etat {
        if self.equipe(s) {
            Etat::Equipe
        } else if self.debloque(s) {
            Etat::Achete
        } else {
            Etat::AAcheter
        }
    }
}

/// Une petite fenêtre par-dessus la liste.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dialogue {
    /// « ACHETER POUR N ? » ; `oui` : le bouton OUI est choisi (NON au départ).
    Confirmer { prix: u32, oui: bool },
    /// « IL TE MANQUE N GOUTTES ».
    Manque { n: u32 },
}

/// Ce que l'application doit faire après une réaction de la boutique.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Rien,
    /// Retour à l'écran titre.
    Quitter,
    /// La sauvegarde a changé (achat ou équipement) : l'enregistrer et appliquer le skin
    /// et les cosmétiques.
    Enregistrer,
}

/// Mise en page des zones cliquables (pixels logiques), fournie par l'écran : la
/// logique reste testable sans fenêtre.
pub trait Zones {
    /// Ligne (objet visible ou RETOUR, indice dans le menu) sous le point.
    fn ligne_sous(&self, etat: &EtatBoutique, x: f32, y: f32) -> Option<usize>;
    /// Onglet sous le point.
    fn onglet_sous(&self, nb_onglets: usize, x: f32, y: f32) -> Option<usize>;
    /// Bouton du dialogue sous le point : `Some(true)` OUI (ou OK), `Some(false)` NON.
    fn bouton_sous(&self, dialogue: Dialogue, x: f32, y: f32) -> Option<bool>;
}

/// L'état de l'écran boutique : onglet, ligne choisie (les objets puis RETOUR),
/// dialogue ouvert.
#[derive(Clone, Debug, PartialEq)]
pub struct EtatBoutique {
    pub onglets: Vec<Categorie>,
    pub onglet: usize,
    pub objets: Vec<Objet>,
    /// Une ligne par objet, plus RETOUR en dernier.
    pub menu: Menu,
    pub dialogue: Option<Dialogue>,
}

impl EtatBoutique {
    pub fn new() -> Self {
        let mut e = Self {
            onglets: onglets(),
            onglet: 0,
            objets: Vec::new(),
            menu: Menu::new(1),
            dialogue: None,
        };
        e.changer_onglet(0);
        e
    }

    fn changer_onglet(&mut self, i: usize) {
        self.onglet = i;
        self.objets = self.onglets.get(i).map_or_else(Vec::new, |c| c.objets());
        self.menu = Menu::new(self.objets.len() + 1);
        self.dialogue = None;
    }

    /// La ligne choisie est-elle RETOUR ?
    pub fn sur_retour(&self) -> bool {
        self.menu.selection() == self.objets.len()
    }

    /// Réagit aux appuis d'une image. `s` : la sauvegarde (achat, équipement).
    pub fn reagir(&mut self, nav: &Navigation, s: &mut Sauvegarde, z: &impl Zones) -> Action {
        if let Some(d) = self.dialogue {
            return self.reagir_dialogue(d, nav, s, z);
        }
        if nav.retour {
            return Action::Quitter;
        }
        let nb = self.onglets.len();
        if nb > 1 && (nav.gauche || nav.droite) {
            let i = if nav.droite {
                (self.onglet + 1) % nb
            } else {
                (self.onglet + nb - 1) % nb
            };
            self.changer_onglet(i);
        }
        if nav.haut {
            self.menu.monter();
        }
        if nav.bas {
            self.menu.descendre();
        }
        if let Some((x, y)) = nav.survol {
            if let Some(i) = z.ligne_sous(self, x, y) {
                self.menu.choisir(i);
            }
        }
        let mut valider = nav.valider;
        if let Some((x, y)) = nav.clic {
            if let Some(i) = z.onglet_sous(nb, x, y) {
                self.changer_onglet(i);
            } else if let Some(i) = z.ligne_sous(self, x, y) {
                self.menu.choisir(i);
                valider = true;
            }
        }
        if !valider {
            return Action::Rien;
        }
        let Some(&objet) = self.objets.get(self.menu.selection()) else {
            return Action::Quitter; // RETOUR
        };
        if objet.debloque(s) {
            return if equiper(&objet, s) {
                Action::Enregistrer
            } else {
                Action::Rien
            };
        }
        self.dialogue = Some(if s.gouttes_en_poche >= objet.prix {
            Dialogue::Confirmer {
                prix: objet.prix,
                oui: false,
            }
        } else {
            Dialogue::Manque {
                n: objet.prix - s.gouttes_en_poche,
            }
        });
        Action::Rien
    }

    fn reagir_dialogue(
        &mut self,
        d: Dialogue,
        nav: &Navigation,
        s: &mut Sauvegarde,
        z: &impl Zones,
    ) -> Action {
        let clic = nav.clic.and_then(|(x, y)| z.bouton_sous(d, x, y));
        match d {
            Dialogue::Manque { .. } => {
                if nav.valider || nav.retour || nav.clic.is_some() {
                    self.dialogue = None;
                }
                Action::Rien
            }
            Dialogue::Confirmer { prix, oui } => {
                if nav.retour || clic == Some(false) || (nav.valider && !oui) {
                    self.dialogue = None;
                    return Action::Rien;
                }
                if clic == Some(true) || (nav.valider && oui) {
                    self.dialogue = None;
                    let objet = self.objets[self.menu.selection()];
                    return if acheter(&objet, s) {
                        Action::Enregistrer
                    } else {
                        // Impossible en pratique (le prix a été vérifié) : on le dit.
                        self.dialogue = Some(Dialogue::Manque {
                            n: prix.saturating_sub(s.gouttes_en_poche),
                        });
                        Action::Rien
                    };
                }
                if nav.gauche || nav.droite || nav.haut || nav.bas {
                    self.dialogue = Some(Dialogue::Confirmer { prix, oui: !oui });
                }
                if let Some((x, y)) = nav.survol {
                    if let Some(b) = z.bouton_sous(d, x, y) {
                        self.dialogue = Some(Dialogue::Confirmer { prix, oui: b });
                    }
                }
                Action::Rien
            }
        }
    }
}

/// Achète `objet` : débit des gouttes puis déblocage. `false` (rien ne change) s'il
/// n'y a pas assez de gouttes. L'appelant enregistre la sauvegarde.
pub fn acheter(objet: &Objet, s: &mut Sauvegarde) -> bool {
    if !s.depenser(objet.prix) {
        return false;
    }
    s.acquerir(&objet.cle());
    true
}

/// Équipe `objet` (déjà débloqué). `false` si rien n'a changé (déjà équipé, ou
/// objet factice qui ne s'équipe pas).
pub fn equiper(objet: &Objet, s: &mut Sauvegarde) -> bool {
    #[cfg(any(test, feature = "debug_niveaux"))]
    if objet.apercu == Apercu::Factice {
        return false;
    }
    if objet.equipe(s) {
        return false;
    }
    // Un objet gratuit est à tout le monde, même absent d'une vieille sauvegarde.
    if objet.prix == 0 {
        s.acquerir(&objet.cle());
    }
    s.equiper(&objet.cle())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SansZones;
    impl Zones for SansZones {
        fn ligne_sous(&self, _: &EtatBoutique, _: f32, _: f32) -> Option<usize> {
            None
        }
        fn onglet_sous(&self, _: usize, _: f32, _: f32) -> Option<usize> {
            None
        }
        fn bouton_sous(&self, _: Dialogue, _: f32, _: f32) -> Option<bool> {
            None
        }
    }

    /// Le seul test de la carte : équiper Pépinette, achat refusé sans assez de
    /// gouttes, puis confirmé (débit, déblocage) avec assez.
    #[test]
    fn equiper_puis_acheter_avec_confirmation() {
        let nav = |f: fn(&mut Navigation)| {
            let mut n = Navigation::default();
            f(&mut n);
            n
        };
        let (bas, valider) = (nav(|n| n.bas = true), nav(|n| n.valider = true));
        let droite = nav(|n| n.droite = true);
        let mut s = Sauvegarde::default();
        let mut e = EtatBoutique::new();
        assert_eq!(e.onglets.len(), 4, "skins, traînées, parapluies, musiques");
        assert_eq!(e.objets.len(), 3, "Pépin, Pépinette, factice (tests)");
        // Pépinette : gratuite, équipée d'un appui.
        e.reagir(&bas, &mut s, &SansZones);
        assert_eq!(e.reagir(&valider, &mut s, &SansZones), Action::Enregistrer);
        assert_eq!(s.skin_choisi, "pepinette");
        // Le factice (60) sans gouttes : refusé.
        e.reagir(&bas, &mut s, &SansZones);
        e.reagir(&valider, &mut s, &SansZones);
        assert_eq!(e.dialogue, Some(Dialogue::Manque { n: 60 }));
        e.reagir(&valider, &mut s, &SansZones);
        // Avec 100 gouttes : confirmation, NON par défaut, puis OUI.
        s.ajouter_gouttes(100);
        e.reagir(&valider, &mut s, &SansZones);
        e.reagir(&valider, &mut s, &SansZones);
        assert_eq!((e.dialogue, s.gouttes_en_poche), (None, 100), "NON");
        e.reagir(&valider, &mut s, &SansZones);
        e.reagir(&droite, &mut s, &SansZones);
        assert_eq!(e.reagir(&valider, &mut s, &SansZones), Action::Enregistrer);
        assert_eq!(s.gouttes_en_poche, 40);
        assert_eq!(e.objets[2].etat(&s), Etat::Achete);
        assert_eq!(s.skin_choisi, "pepinette", "le factice ne s'équipe pas");
        e.reagir(&bas, &mut s, &SansZones);
        assert!(e.sur_retour());
        assert_eq!(e.reagir(&valider, &mut s, &SansZones), Action::Quitter);
    }
}
