//! Module `player` : état du joueur (Pépin) et sa mise à jour à pas fixe.
//!
//! Mécaniques : course (accélération / friction), saut à hauteur variable,
//! coyote time, jump buffer, PLANER (parapluie ouvert), courants ascendants,
//! vent latéral et élan des auvents.
//!
//! Notions Rust illustrées ici (à reprendre dans le README) :
//! - `enum` sans données (`Direction`, `EtatJoueur`) et `match` exhaustif ;
//! - `struct` d'entrées : on passe à la logique des DONNÉES, pas le clavier ;
//!   le cœur reste pur et testable (les tests « appuient » sur des touches virtuelles) ;
//! - `&mut self` : la méthode modifie le joueur ; `&Entrees` / `&Environnement` :
//!   elle ne fait que LIRE le reste du monde (emprunts partagés) ;
//! - `Default` : valeur de départ « tout à faux / à zéro » ;
//! - tests unitaires `#[cfg(test)]`.

use crate::aabb::{deplacer, Aabb};
use crate::physics::*;

/// Protection après une réapparition : 1 s (60 pas), celui de la réapparition compris.
///
/// Pendant ce temps, aucun DANGER (ennemi, pointe, éclair, et tout danger futur
/// traité par `contacts::resoudre_contacts`) ne peut tuer Pépin ; la chute dans le
/// vide, elle, reste mortelle (sinon il pourrait rester coincé en dessous du monde).
/// Sans elle, un ennemi qui passe sur la lanterne tuerait Pépin à chaque pas :
/// il réapparaîtrait dedans et mourrait de nouveau, jusqu'à ce que l'ennemi s'éloigne.
///
/// Durée FIXE (elle ne s'arrête pas quand le joueur bouge) : plus simple et
/// prévisible. Elle ne se recharge qu'à une réapparition : on ne peut donc pas
/// s'en servir pour traverser un danger, sauf juste à côté de la lanterne.
/// 1 s suffit pour qu'une flaque (24 px/s) passe entièrement sur Pépin immobile :
/// 22 px de chevauchement (12 px de flaque + 10 px de Pépin), soit 0,92 s.
pub const PROTECTION_REAPPARITION_PAS: u16 = HZ as u16;

/// Direction horizontale demandée par le joueur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    Gauche,
    #[default]
    Immobile,
    Droite,
}

impl Direction {
    /// Signe numérique : -1, 0 ou +1. Un `match` doit couvrir TOUTES les variantes,
    /// sinon le programme ne compile pas : impossible d'en oublier une.
    pub fn signe(self) -> f32 {
        match self {
            Direction::Gauche => -1.0,
            Direction::Immobile => 0.0,
            Direction::Droite => 1.0,
        }
    }
}

/// Entrées d'UN pas de simulation (clavier, manette ou tactile, peu importe).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Entrees {
    pub direction: Direction,
    /// Touche de saut ENFONCÉE (maintenue). Le front montant (appui) est détecté
    /// par le joueur lui-même ; maintenue en l'air, elle ouvre le parapluie.
    pub saut: bool,
    /// Touche BAS ENFONCÉE (carte 9.2b) : tant qu'elle est tenue, Pépin tient
    /// son parapluie RETOURNÉ (la toile en bol, la crosse en bas). Il entre
    /// ainsi dans l'eau en barque. SAUT tenu l'emporte : on plane.
    pub bas: bool,
}

/// Phase de mouvement du joueur (pratique pour choisir l'animation).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtatJoueur {
    Sol,
    Montee,
    Chute,
    Plane,
    /// Parapluie RETOURNÉ (carte 9.2b) : BAS tenu, au sol ou en l'air, sans
    /// planer. Prêt à devenir une barque en touchant l'eau.
    Retourne,
    /// Barque (carte 9.2) : pépin flottant sur l'eau, parapluie retourné
    /// sous lui.
    Barque,
}

/// Ce qui s'est passé pendant un pas : le jeu s'en sert pour les particules et le son.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Evenements {
    pub saut: bool,
    pub atterrissage: bool,
    pub debut_plane: bool,
}

/// État complet du joueur.
#[derive(Debug, Clone)]
pub struct Joueur {
    /// Boîte de collision (position = coin haut-gauche).
    pub boite: Aabb,
    /// Vitesse en px/s.
    pub vitesse: Vec2,
    /// Le joueur est-il posé sur un solide ?
    pub au_sol: bool,
    /// Regarde vers la droite ? (pour l'affichage)
    pub regarde_droite: bool,
    /// Parapluie ouvert ?
    pub plane: bool,
    /// Parapluie retourné (BAS tenu sans planer, carte 9.2b) ? Recalculé à
    /// chaque pas depuis les entrées, comme `plane`.
    retourne: bool,
    saut_precedent: bool,
    /// Pas restants pendant lesquels un saut est encore permis après avoir quitté le sol.
    coyote: u8,
    /// Pas restants pendant lesquels un appui sur saut est mémorisé.
    tampon: u8,
    /// Vrai pendant la montée d'un vrai saut (permet la coupure du saut).
    monte_par_saut: bool,
    /// Pas restants de protection après une réapparition (0 = aucune protection).
    /// Voir `PROTECTION_REAPPARITION_PAS`.
    protection: u16,
    /// Élan horizontal donné par un auvent (px/s, vers la droite ; 0 = aucun).
    /// Rangé à part de `vitesse.x` : les commandes du joueur s'y AJOUTENT au lieu
    /// de le freiner (garder la droite va plus loin, garder la gauche moins loin).
    /// Il s'éteint en ligne droite (`BAISSE_ELAN_AUVENT` par pas) et tombe à 0 dès
    /// que Pépin touche un mur ou se pose. Voir `lancer_par_auvent`.
    elan_auvent: f32,
    /// Barque (carte 9.2) : Pépin flotte sur l'eau, parapluie retourné sous
    /// lui. Il en sort d'un SAUT (comme du sol) ; il en entre en PLANANT.
    en_barque: bool,
    /// Noyade (carte 9.2) : Pépin est entré dans l'eau SANS planer, pieds sous
    /// la surface, et non protégé. Signalée une fois à `resoudre_contacts`
    /// (`CauseMort::Noyade`), remise à zéro à chaque `placer` (respawn).
    noyade: bool,
}

impl Joueur {
    /// Crée un joueur dont le coin haut-gauche est en `(x, y)`.
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            boite: Aabb::new(x, y, JOUEUR_L, JOUEUR_H),
            vitesse: Vec2::default(),
            au_sol: false,
            regarde_droite: true,
            plane: false,
            retourne: false,
            saut_precedent: false,
            coyote: 0,
            tampon: 0,
            monte_par_saut: false,
            protection: 0,
            elan_auvent: 0.0,
            en_barque: false,
            noyade: false,
        }
    }

    /// Replace le joueur (respawn) en remettant vitesse et minuteries à zéro.
    /// La protection aussi : c'est à l'appelant de la donner (`proteger`), seulement
    /// pour une vraie réapparition (`EtatJeu::traiter`).
    pub fn placer(&mut self, x: f32, y: f32) {
        *self = Joueur::new(x, y);
    }

    /// Donne la protection de réapparition (durée `PROTECTION_REAPPARITION_PAS`).
    /// On REMPLACE le reste éventuel, on n'additionne pas : impossible de cumuler.
    pub fn proteger(&mut self) {
        self.protection = PROTECTION_REAPPARITION_PAS;
    }

    /// Pépin est-il protégé des dangers (juste après une réapparition) ?
    pub fn est_protege(&self) -> bool {
        self.protection > 0
    }

    /// Pas de protection restants (pour l'affichage et les tests).
    pub fn protection_restante(&self) -> u16 {
        self.protection
    }

    /// Phase de mouvement courante, déduite des champs.
    pub fn etat(&self) -> EtatJoueur {
        if self.en_barque {
            EtatJoueur::Barque
        } else if self.retourne {
            // Au sol comme en l'air (carte 9.2b) : le parapluie retourné se
            // voit. (`retourne` est toujours faux quand on plane.)
            EtatJoueur::Retourne
        } else if self.au_sol {
            EtatJoueur::Sol
        } else if self.plane {
            EtatJoueur::Plane
        } else if self.vitesse.y < 0.0 {
            EtatJoueur::Montee
        } else {
            EtatJoueur::Chute
        }
    }

    /// Pépin est-il entré dans l'eau sans planer (carte 9.2) ? Vrai jusqu'au
    /// prochain `placer` : `resoudre_contacts` en tire la mort par noyade.
    pub fn est_noye(&self) -> bool {
        self.noyade
    }

    /// Barque (carte 9.2) ? Flottant à la surface, parapluie retourné sous lui.
    pub fn en_barque(&self) -> bool {
        self.en_barque
    }

    /// Met Pépin À BORD de la barque (carte 9.2) : flottant à la surface,
    /// parapluie retourné sous lui. Les robots le reconstituent depuis un
    /// état (leur noeud ne garde que la position) ; le jeu lui l'obtient
    /// tout seul en PLANANT dans l'eau (`actualiser`).
    pub fn entrer_en_barque(&mut self) {
        self.en_barque = true;
        self.plane = false;
        self.au_sol = false;
        self.vitesse.y = 0.0;
    }

    /// Rebond forcé (trampoline, écrasement d'ennemi) : propulse vers le haut
    /// à `vitesse` px/s. Ce n'est pas un « saut » : il ne peut pas être coupé.
    pub fn rebondir(&mut self, vitesse: f32) {
        self.vitesse.y = -vitesse;
        self.au_sol = false;
        self.plane = false;
        self.monte_par_saut = false;
        self.coyote = 0;
    }

    /// Lancer d'auvent : vers le haut à `VITESSE_AUVENT` et vers la droite avec
    /// l'élan `ELAN_AUVENT`. La vitesse horizontale propre repart de 0 : le vol
    /// ne dépend donc QUE de l'endroit où Pépin touche la toile (pas de la façon
    /// dont il est arrivé), ce qui le rend prévisible pour le joueur comme pour
    /// les robots. Comme `rebondir`, ce n'est pas un saut : il ne se coupe pas.
    pub fn lancer_par_auvent(&mut self) {
        self.rebondir(VITESSE_AUVENT);
        self.vitesse.x = 0.0;
        self.elan_auvent = ELAN_AUVENT;
        self.tampon = 0;
        self.regarde_droite = true;
    }

    /// Élan d'auvent restant (px/s ; 0 = aucun). Pour les tests et l'affichage.
    pub fn elan_auvent(&self) -> f32 {
        self.elan_auvent
    }

    /// Déplace le joueur d'une plateforme mobile (sans entrée, sans gravité) en
    /// respectant les collisions. À appeler avec le déplacement de la plateforme.
    pub fn porter(&mut self, dx: f32, dy: f32, env: &Environnement<'_>) {
        deplacer(&mut self.boite, dx, dy, env.grille, env.solides);
    }

    /// Un pas de simulation de 1/60 s.
    pub fn actualiser(&mut self, entrees: &Entrees, env: &Environnement<'_>) -> Evenements {
        let mut ev = Evenements::default();
        let etait_au_sol = self.au_sol;
        let etait_plane = self.plane;

        // --- 0. Protection de réapparition : un pas de moins ----------------
        // `saturating_sub` : 0 - 1 reste 0 (un `u16` ne peut pas être négatif).
        self.protection = self.protection.saturating_sub(1);

        // --- 1. Minuteries : coyote time et jump buffer ---------------------
        let appui = entrees.saut && !self.saut_precedent; // front montant
        self.saut_precedent = entrees.saut;
        if appui {
            self.tampon = TAMPON_SAUT_PAS;
        }
        self.coyote = if self.au_sol {
            COYOTE_PAS
        } else {
            self.coyote.saturating_sub(1)
        };

        // --- 2. Saut --------------------------------------------------------
        // Depuis la barque (carte 9.2), le saut est un SAUT NORMAL qui sort
        // Pépin de l'eau, comme depuis le sol.
        if self.tampon > 0 && (self.au_sol || self.coyote > 0 || self.en_barque) {
            self.vitesse.y = -VITESSE_SAUT;
            self.au_sol = false;
            self.en_barque = false;
            self.monte_par_saut = true;
            self.tampon = 0;
            self.coyote = 0;
            ev.saut = true;
        } else {
            self.tampon = self.tampon.saturating_sub(1);
        }

        // Saut à hauteur variable : relâcher pendant la montée la raccourcit.
        if self.monte_par_saut && !entrees.saut && self.vitesse.y < -VITESSE_COUPE_SAUT {
            self.vitesse.y = -VITESSE_COUPE_SAUT;
        }
        if self.vitesse.y >= 0.0 {
            self.monte_par_saut = false;
        }

        // --- 3. Planer : saut maintenu, en l'air, en train de descendre -----
        self.plane = !self.au_sol && entrees.saut && self.vitesse.y >= 0.0;
        if self.plane && !etait_plane {
            ev.debut_plane = true;
        }
        // --- 3. bis Parapluie retourné (carte 9.2b) : BAS tenu -------------
        // Au sol comme en l'air, BAS tenu retourne le parapluie (la toile en
        // bol). Cela ne change RIEN au mouvement : en l'air, Pépin tombe
        // normalement. Seule l'eau le remarque (étape 8 : il y devient une
        // barque). SAUT tenu l'emporte : planer et retourner en même temps,
        // c'est planer.
        self.retourne = entrees.bas && !self.plane;

        // --- 3bis. Barque (carte 9.2) : le parapluie retourné flotte --------
        // Ramer est LENT (40 % de la course, `VITESSE_RAME`) ; le courant
        // d'eau (`VITESSE_COURANT_EAU`) pousse la barque en DÉPLACEMENT, comme
        // le vent latéral (la vitesse propre de Pépin n'y mêle pas). La barque
        // reste posée sur la surface de l'étendue d'eau qui passe sous son
        // CENTRE, et quitte l'eau quand ce centre en sort (le reste du pas
        // reprend alors : chute, atterrissage sur le quai...).
        if self.en_barque {
            // Le parapluie est retourné sous la barque, pas ouvert pour planer.
            // À bord, plus besoin de tenir BAS : la barque reste.
            self.plane = false;
            self.retourne = false;
            let cible = match entrees.direction {
                Direction::Droite => VITESSE_RAME,
                Direction::Gauche => -VITESSE_RAME,
                Direction::Immobile => 0.0,
            };
            if cible != 0.0 {
                self.regarde_droite = cible > 0.0;
            }
            let pas_rame = if cible != 0.0 {
                ACCEL_RAME
            } else {
                FRICTION_RAME
            };
            self.vitesse.x = approcher(self.vitesse.x, cible, pas_rame * DT);
            // Le courant d'eau ne pousse QUE la barque : un Pépin qui plane au-
            // dessus de l'eau n'est pas touché (à la différence du vent).
            let poussee_eau: f32 = env
                .eaux
                .iter()
                .filter(|e| e.courant != 0 && e.zone.chevauche(&self.boite))
                .map(|e| e.courant as f32 * VITESSE_COURANT_EAU)
                .sum();
            let contacts = deplacer(
                &mut self.boite,
                self.vitesse.x * DT + poussee_eau * DT,
                0.0,
                env.grille,
                env.solides,
            );
            if contacts.mur() {
                // La barque s'arrête au bord (le quai) ; la rame ne la pousse
                // pas plus loin.
                self.vitesse.x = 0.0;
            }
            // La surface de l'étendue d'eau où flotte VRAIMENT la barque : sous
            // son centre ET à sa hauteur (`eau_touchee`). Un bassin suspendu
            // au-dessus ne la happe pas (correctif d'AGA).
            match eau_touchee(&self.boite, env.eaux).map(Eau::surface) {
                Some(surface) => {
                    // La barque reste flottante : les pieds de Pépin restent
                    // `ENFONCEMENT_BARQUE` sous la surface.
                    self.boite.y = surface - JOUEUR_H + ENFONCEMENT_BARQUE;
                    self.vitesse.y = 0.0;
                }
                None => {
                    // Le centre est sorti de l'étendue : la barque quitte
                    // l'eau et retombe comme n'importe quel joueur.
                    self.en_barque = false;
                }
            }
            return ev;
        }

        // --- 4. Mouvement horizontal ---------------------------------------
        let signe = entrees.direction.signe();
        let (vmax, accel, friction) = if self.au_sol {
            (VITESSE_COURSE, ACCEL_SOL, FRICTION_SOL)
        } else if self.plane {
            (VITESSE_PLANER_X, ACCEL_PLANER, FRICTION_PLANER)
        } else {
            (VITESSE_COURSE, ACCEL_AIR, FRICTION_AIR)
        };
        self.vitesse.x = if signe != 0.0 {
            self.regarde_droite = signe > 0.0;
            approcher(self.vitesse.x, signe * vmax, accel * DT)
        } else {
            approcher(self.vitesse.x, 0.0, friction * DT)
        };

        // --- 5. Gravité (ou parapluie) -------------------------------------
        if self.plane {
            if self.vitesse.y > VITESSE_PLANER_CHUTE {
                // Chute rapide : le parapluie freine progressivement.
                self.vitesse.y = (self.vitesse.y - FREIN_PLANER * DT).max(VITESSE_PLANER_CHUTE);
            } else {
                self.vitesse.y = (self.vitesse.y + GRAVITE * DT).min(VITESSE_PLANER_CHUTE);
            }
        } else {
            self.vitesse.y = (self.vitesse.y + GRAVITE * DT).min(VITESSE_MAX_CHUTE);
        }

        // --- 6. Courants ascendants ----------------------------------------
        // `iter().find` parcourt la tranche en EMPRUNTANT chaque courant.
        if let Some(courant) = env.courants.iter().find(|c| c.zone.chevauche(&self.boite)) {
            if self.vitesse.y > -VITESSE_MAX_COURANT {
                self.vitesse.y =
                    (self.vitesse.y - courant.acceleration * DT).max(-VITESSE_MAX_COURANT);
            }
        }

        // --- 6bis. Vent latéral -------------------------------------------
        // Le vent pousse en DÉPLACEMENT, pas dans `vitesse.x` (la vitesse du
        // joueur reste propre : l'animation de course et la friction ne la
        // voient pas). Parapluie fermé : `VENT_LAT` (60 px/s, 1 px par pas) ;
        // en planant (parapluie ouvert) : deux fois plus fort. On somme les
        // sens des zones qui touchent la boîte : deux vents opposés qui se
        // chevauchent s'annulent, plusieurs vents pareils ne se cumulent pas
        // au-delà d'un vent.
        let sens_vent: i32 = env
            .vents
            .iter()
            .filter(|v| !v.leger && v.zone.chevauche(&self.boite))
            .map(|v| i32::from(v.sens))
            .sum();
        let sens_vent = if sens_vent.abs() > 1 {
            sens_vent.signum()
        } else {
            sens_vent
        };
        let poussee_vent = sens_vent as f32 * if self.plane { VENT_LAT_PLANE } else { VENT_LAT };

        // --- 7. Déplacement + collisions, axe par axe ----------------------
        // L'élan d'auvent s'ajoute au déplacement, comme le vent : il ne passe
        // pas par `vitesse.x`, donc ni la friction ni la vitesse maximale de
        // course ne le rognent. (Ajouté seulement s'il existe : sans auvent, le
        // calcul reste exactement celui d'avant, au bit près.)
        let mut dx = self.vitesse.x * DT + poussee_vent * DT;
        if self.elan_auvent != 0.0 {
            dx += self.elan_auvent * DT;
        }
        // Le léger vent de face de la nouvelle partie + (carte 8.8) : même force
        // plané ou non. Ajouté à part, et seulement s'il existe : en partie normale,
        // le calcul reste celui d'avant, au bit près.
        let poussee_legere: f32 = env
            .vents
            .iter()
            .filter(|v| v.leger && v.zone.chevauche(&self.boite))
            .map(|v| f32::from(v.sens) * VENT_DE_FACE)
            .sum();
        if poussee_legere != 0.0 {
            dx += poussee_legere * DT;
        }
        let contacts = deplacer(
            &mut self.boite,
            dx,
            self.vitesse.y * DT,
            env.grille,
            env.solides,
        );
        // Il s'éteint en ligne droite : 0 au bout de `VOL_AUVENT_PAS` pas.
        self.elan_auvent = approcher(self.elan_auvent, 0.0, BAISSE_ELAN_AUVENT);
        if contacts.mur() {
            self.vitesse.x = 0.0;
            self.elan_auvent = 0.0;
        }
        if contacts.plafond && self.vitesse.y < 0.0 {
            self.vitesse.y = 0.0;
            self.monte_par_saut = false;
        }
        if contacts.sol {
            self.vitesse.y = 0.0;
            self.au_sol = true;
            self.plane = false;
            self.monte_par_saut = false;
            self.elan_auvent = 0.0;
            if !etait_au_sol {
                ev.atterrissage = true;
            }
        } else {
            self.au_sol = false;
        }

        // --- 8. Eau (cartes 9.2 et 9.2b) : parapluie retourné = barque ------
        // L'étendue est un VRAI obstacle : sous les pieds, dans sa zone, les
        // pieds SOUS la surface (que l'on marche au sol, que l'on saute ou
        // que l'on plane), c'est la noyade ; parapluie RETOURNÉ (BAS tenu),
        // c'est la barque. (Carte 9.2b, demande d'AGA : la flèche du bas met
        // le parapluie en bateau, plus le saut. Planer dans l'eau noie
        // désormais, comme tomber parapluie fermé.) Au ras de la surface
        // (marcher au bord du bassin, les pieds AU NIVEAU de l'eau, `bas() ==
        // surface`), rien ne se passe : le `>` strict fait la différence. La
        // barque, elle, est flottante (les pieds restent 3 px sous la
        // surface) : c'est l'état normal de l'eau, pas une noyade.
        // Seule compte l'étendue que Pépin chevauche VRAIMENT (`eau_touchee` :
        // pieds sous la surface, tête au-dessus du fond) : marcher au sol sous
        // un bassin suspendu ne noie pas (correctif d'AGA).
        if !self.en_barque {
            if let Some(surface) = eau_touchee(&self.boite, env.eaux).map(Eau::surface) {
                // Toujours vrai ici (`eau_touchee` l'exige) ; gardé pour la lecture.
                if self.boite.bas() > surface {
                    if self.retourne || self.est_protege() {
                        // Parapluie RETOURNÉ : Pépin devient une barque, la
                        // toile en bol flotte à la surface. Il la garde sans
                        // rien tenir (voir le pas 3bis). PROTÉGÉ (il vient de
                        // réapparaître), il ne peut pas couler : l'eau le
                        // recueille aussi en barque. C'est ce qui garde
                        // jouable une lanterne posée sur l'eau : sans cela, la
                        // réapparition au-dessus de l'eau finirait noyée dès
                        // la fin de la protection.
                        self.en_barque = true;
                        self.plane = false;
                        self.retourne = false;
                        self.vitesse.y = 0.0;
                        self.boite.y = surface - JOUEUR_H + ENFONCEMENT_BARQUE;
                    } else {
                        // Parapluie fermé OU OUVERT (on plane) : il coule, une
                        // vie perdue. Marcher AU-DESSOUS de la
                        // surface compte : l'eau est un obstacle, pas une fosse
                        // contournable.
                        self.noyade = true;
                    }
                }
            }
        }
        ev
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Planer est plus lent que la course. Vérifié à la compilation.
    const _: () = assert!(VITESSE_PLANER_X < VITESSE_COURSE);
    use crate::aabb::{GrilleTuiles, TAILLE_TUILE};

    /// Salle 20 × 12 tuiles : murs latéraux, sol sur la ligne 11 (haut du sol : y = 176),
    /// plafond sur la ligne 0 (bas du plafond : y = 16).
    fn salle() -> GrilleTuiles {
        let mut lignes = vec!["#..................#"; 12];
        lignes[0] = "####################";
        lignes[11] = "####################";
        GrilleTuiles::depuis_texte(&lignes)
    }

    const SOL_Y: f32 = 176.0;
    const AUCUNE: Entrees = Entrees {
        direction: Direction::Immobile,
        saut: false,
        bas: false,
    };
    const SAUT: Entrees = Entrees {
        direction: Direction::Immobile,
        saut: true,
        bas: false,
    };
    /// BAS tenu seul (carte 9.2b) : le parapluie retourné.
    const BAS: Entrees = Entrees {
        direction: Direction::Immobile,
        saut: false,
        bas: true,
    };

    fn droite() -> Entrees {
        Entrees {
            direction: Direction::Droite,
            saut: false,
            bas: false,
        }
    }

    /// Joueur posé sur le sol de la salle, à x donné, après quelques pas pour se stabiliser.
    fn au_sol(grille: &GrilleTuiles, x: f32) -> Joueur {
        let env = Environnement::simple(grille);
        let mut j = Joueur::new(x, SOL_Y - JOUEUR_H - 4.0);
        for _ in 0..10 {
            j.actualiser(&AUCUNE, &env);
        }
        assert!(j.au_sol);
        j
    }

    /// Hauteur maximale atteinte (en px au-dessus du point de départ) pendant `n` pas.
    fn hauteur_atteinte(j: &mut Joueur, env: &Environnement<'_>, entrees: &[Entrees]) -> f32 {
        let depart = j.boite.y;
        let mut haut = depart;
        for e in entrees {
            j.actualiser(e, env);
            haut = haut.min(j.boite.y);
        }
        depart - haut
    }

    #[test]
    fn gravite_acceleration_puis_chute() {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut j = Joueur::new(100.0, 40.0);
        j.actualiser(&AUCUNE, &env);
        assert!((j.vitesse.y - GRAVITE * DT).abs() < 1e-3);
        let mut y_prec = j.boite.y;
        for _ in 0..10 {
            j.actualiser(&AUCUNE, &env);
            assert!(j.boite.y > y_prec, "le joueur descend");
            y_prec = j.boite.y;
        }
        assert_eq!(j.etat(), EtatJoueur::Chute);
        // Chute plus longue : la vitesse ne dépasse jamais le plafond.
        let mut j = Joueur::new(100.0, -2000.0);
        for _ in 0..200 {
            j.actualiser(&AUCUNE, &env);
            assert!(j.vitesse.y <= VITESSE_MAX_CHUTE);
        }
        assert_eq!(j.vitesse.y, VITESSE_MAX_CHUTE);
    }

    #[test]
    fn atterrissage_pile_sur_le_sol() {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut j = Joueur::new(100.0, 60.0);
        let mut atterri = false;
        for _ in 0..120 {
            let ev = j.actualiser(&AUCUNE, &env);
            atterri |= ev.atterrissage;
        }
        assert!(atterri, "l'événement d'atterrissage a bien été émis");
        assert!(j.au_sol);
        assert_eq!(j.etat(), EtatJoueur::Sol);
        assert_eq!(j.boite.bas(), SOL_Y);
        assert_eq!(j.vitesse.y, 0.0);
    }

    #[test]
    fn course_acceleration_puis_friction() {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut j = au_sol(&g, 100.0);
        j.actualiser(&droite(), &env);
        assert!(
            (j.vitesse.x - ACCEL_SOL * DT).abs() < 1e-3,
            "accélère progressivement"
        );
        for _ in 0..30 {
            j.actualiser(&droite(), &env);
        }
        assert_eq!(
            j.vitesse.x, VITESSE_COURSE,
            "plafonnée à la vitesse de course"
        );
        assert!(j.regarde_droite);
        let mut dernier = j.vitesse.x;
        for _ in 0..30 {
            j.actualiser(&AUCUNE, &env);
            assert!(j.vitesse.x <= dernier);
            dernier = j.vitesse.x;
        }
        assert_eq!(j.vitesse.x, 0.0, "la friction arrête le joueur");
    }

    #[test]
    fn collision_mur() {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut j = au_sol(&g, 100.0);
        for _ in 0..400 {
            j.actualiser(&droite(), &env);
        }
        assert_eq!(j.boite.droite(), 19.0 * TAILLE_TUILE);
        assert_eq!(j.vitesse.x, 0.0);
        assert!(j.au_sol);
    }

    #[test]
    fn collision_plafond() {
        // Plafond bas : sa face inférieure est à y = 144, soit 18 px au-dessus de la tête.
        let mut lignes = vec!["#..................#"; 12];
        lignes[8] = "####################";
        lignes[11] = "####################";
        let g = GrilleTuiles::depuis_texte(&lignes);
        let env = Environnement::simple(&g);
        let mut j = au_sol(&g, 100.0);
        let mut touche = false;
        for _ in 0..30 {
            j.actualiser(&SAUT, &env);
            if j.boite.y == 144.0 {
                touche = true;
                assert_eq!(j.vitesse.y, 0.0, "la montée est coupée net");
                break;
            }
        }
        assert!(touche, "la tête a touché le plafond");
        j.actualiser(&SAUT, &env);
        assert!(j.vitesse.y > 0.0, "puis le joueur retombe");
    }

    #[test]
    fn saut_complet_atteint_la_hauteur_prevue() {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut j = au_sol(&g, 100.0);
        let h = hauteur_atteinte(&mut j, &env, &[SAUT; 40]);
        let theorique = hauteur_saut_theorique();
        assert!(
            (h - theorique).abs() < 6.0,
            "h = {h}, théorique = {theorique}"
        );
    }

    #[test]
    fn saut_variable_relacher_raccourcit() {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut long = au_sol(&g, 100.0);
        let mut court = au_sol(&g, 100.0);
        let mut touches_court = vec![SAUT];
        touches_court.extend([AUCUNE; 39]);
        let h_long = hauteur_atteinte(&mut long, &env, &[SAUT; 40]);
        let h_court = hauteur_atteinte(&mut court, &env, &touches_court);
        assert!(h_court > 5.0, "même un appui bref décolle ({h_court})");
        assert!(h_court < 20.0, "saut court = {h_court}");
        assert!(
            h_long - h_court > 30.0,
            "long = {h_long}, court = {h_court}"
        );
    }

    /// Fait marcher le joueur vers la droite jusqu'au bord d'une corniche (x = 112).
    fn joueur_au_bord(g: &GrilleTuiles) -> Joueur {
        let env = Environnement::simple(g);
        let mut j = au_sol(g, 60.0);
        for _ in 0..200 {
            j.actualiser(&droite(), &env);
            if !j.au_sol {
                return j;
            }
        }
        panic!("le joueur n'a jamais quitté la corniche");
    }

    fn corniche() -> GrilleTuiles {
        let mut lignes = vec!["#..................#"; 12];
        lignes[0] = "####################";
        lignes[11] = "#######.............";
        GrilleTuiles::depuis_texte(&lignes)
    }

    #[test]
    fn coyote_time_saut_juste_apres_le_bord() {
        let g = corniche();
        let env = Environnement::simple(&g);
        let mut j = joueur_au_bord(&g);
        for _ in 0..3 {
            j.actualiser(&AUCUNE, &env);
        }
        assert!(!j.au_sol);
        let ev = j.actualiser(&SAUT, &env);
        assert!(ev.saut, "saut accepté dans la fenêtre de coyote time");
        assert!(j.vitesse.y < 0.0);
    }

    #[test]
    fn coyote_time_expire() {
        let g = corniche();
        let env = Environnement::simple(&g);
        let mut j = joueur_au_bord(&g);
        for _ in 0..(COYOTE_PAS as usize + 4) {
            j.actualiser(&AUCUNE, &env);
        }
        let ev = j.actualiser(&SAUT, &env);
        assert!(!ev.saut, "trop tard : plus de saut en l'air");
        assert!(j.vitesse.y >= 0.0);
    }

    /// Lâche un joueur de haut ; appuie sur SAUT (un seul pas) quand il est à moins de
    /// `distance` px du sol ; renvoie `true` si un saut a eu lieu à l'atterrissage.
    fn appui_avant_atterrissage(distance: f32) -> bool {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut j = Joueur::new(100.0, 40.0);
        let mut deja_appuye = false;
        let mut a_saute = false;
        for _ in 0..150 {
            let doit_appuyer = !deja_appuye && j.boite.bas() >= SOL_Y - distance;
            deja_appuye |= doit_appuyer;
            let e = if doit_appuyer { SAUT } else { AUCUNE };
            a_saute |= j.actualiser(&e, &env).saut;
        }
        a_saute
    }

    #[test]
    fn jump_buffer_appui_juste_avant_le_sol() {
        assert!(
            appui_avant_atterrissage(12.0),
            "appui ~3 pas avant : mémorisé"
        );
    }

    #[test]
    fn jump_buffer_expire() {
        assert!(!appui_avant_atterrissage(90.0), "appui trop tôt : oublié");
    }

    #[test]
    fn planer_plafonne_la_chute_et_relacher_la_retablit() {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut j = Joueur::new(100.0, 24.0);
        // Chute libre puis parapluie ouvert.
        for _ in 0..8 {
            j.actualiser(&AUCUNE, &env);
        }
        assert!(j.vitesse.y > VITESSE_PLANER_CHUTE);
        for _ in 0..20 {
            j.actualiser(&SAUT, &env);
            assert!(j.vitesse.y <= VITESSE_MAX_CHUTE);
        }
        assert_eq!(j.etat(), EtatJoueur::Plane);
        assert_eq!(j.vitesse.y, VITESSE_PLANER_CHUTE, "chute plafonnée");
        for _ in 0..10 {
            j.actualiser(&SAUT, &env);
            assert_eq!(j.vitesse.y, VITESSE_PLANER_CHUTE, "reste plafonnée");
        }
        // Parapluie fermé : la gravité reprend.
        for _ in 0..10 {
            j.actualiser(&AUCUNE, &env);
        }
        assert_eq!(j.etat(), EtatJoueur::Chute);
        assert!(
            j.vitesse.y > VITESSE_PLANER_CHUTE + 50.0,
            "vy = {}",
            j.vitesse.y
        );
    }

    #[test]
    fn planer_ne_demarre_pas_pendant_la_montee() {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut j = au_sol(&g, 100.0);
        let ev = j.actualiser(&SAUT, &env);
        assert!(ev.saut && !j.plane);
        assert_eq!(j.etat(), EtatJoueur::Montee);
        // Bouton maintenu : après le sommet, le parapluie s'ouvre tout seul.
        let mut a_plane = false;
        for _ in 0..60 {
            a_plane |= j.actualiser(&SAUT, &env).debut_plane;
        }
        assert!(a_plane);
    }

    #[test]
    fn planer_controle_horizontal_leger() {
        let g = salle();
        let env = Environnement::simple(&g);
        let planer_droite = Entrees {
            direction: Direction::Droite,
            saut: true,
            bas: false,
        };
        let mut j = Joueur::new(60.0, 24.0);
        for _ in 0..50 {
            j.actualiser(&planer_droite, &env);
        }
        assert_eq!(j.etat(), EtatJoueur::Plane);
        assert_eq!(j.vitesse.x, VITESSE_PLANER_X, "plus lent que la course");
        // Sans parapluie, la même chute permet de dépasser cette vitesse.
        let mut k = Joueur::new(60.0, 24.0);
        for _ in 0..25 {
            k.actualiser(&droite(), &env);
        }
        assert!(k.vitesse.x > VITESSE_PLANER_X);
    }

    #[test]
    fn courant_ascendant_fait_monter() {
        let g = salle();
        let courants = [CourantAscendant::new(Aabb::new(48.0, 64.0, 32.0, 112.0))];
        let env = Environnement {
            grille: &g,
            solides: &[],
            courants: &courants,
            vents: &[],
            eaux: &[],
        };
        let mut j = au_sol(&g, 56.0);
        let depart = j.boite.y;
        let mut vy_min = 0.0_f32;
        for _ in 0..90 {
            j.actualiser(&AUCUNE, &env);
            vy_min = vy_min.min(j.vitesse.y);
        }
        assert_eq!(vy_min, -VITESSE_MAX_COURANT, "montée plafonnée");
        assert!(j.boite.y < depart - 60.0, "le vent a soulevé le joueur");
        // Hors du courant : aucun effet, le joueur reste au sol.
        let mut hors = au_sol(&g, 200.0);
        for _ in 0..30 {
            hors.actualiser(&AUCUNE, &env);
        }
        assert!(hors.au_sol);
    }

    #[test]
    fn planer_dans_un_courant_ne_tombe_pas() {
        let g = salle();
        let courants = [CourantAscendant::new(Aabb::new(48.0, 32.0, 32.0, 144.0))];
        let env = Environnement {
            grille: &g,
            solides: &[],
            courants: &courants,
            vents: &[],
            eaux: &[],
        };
        let mut j = Joueur::new(56.0, 120.0);
        for _ in 0..90 {
            j.actualiser(&SAUT, &env);
        }
        assert!(j.boite.y < 100.0, "y = {}", j.boite.y);
    }

    // --- Vent latéral (carte 5.3) -----------------------------------------

    /// Zone de vent d'essai : une zone unique, sens `sens`, qui couvre le sol
    /// (x de 48 à 280).
    fn zone_vent(sens: i8) -> VentLateral {
        VentLateral::new(Aabb::new(48.0, 64.0, 232.0, 112.0), sens)
    }

    #[test]
    fn vent_lateral_pousse_au_sol() {
        let g = salle();
        let vents = [zone_vent(1)];
        let env = Environnement {
            grille: &g,
            solides: &[],
            courants: &[],
            vents: &vents,
            eaux: &[],
        };
        let mut j = au_sol(&g, 56.0);
        for _ in 0..60 {
            j.actualiser(&AUCUNE, &env);
        }
        // 60 px/s = exactement 1 px par pas (aucune entrée, au sol).
        assert_eq!(j.boite.x, 56.0 + 60.0, "poussé par le vent à droite");
        // La poussée est un DÉPLACEMENT : la vitesse du joueur est restée à zéro.
        assert_eq!(j.vitesse.x, 0.0, "le vent ne rentre pas dans vitesse.x");
        assert!(j.au_sol);
    }

    #[test]
    fn vent_lateral_gauche_pousse_vers_la_gauche() {
        let g = salle();
        let vents = [zone_vent(-1)];
        let env = Environnement {
            grille: &g,
            solides: &[],
            courants: &[],
            vents: &vents,
            eaux: &[],
        };
        let mut j = au_sol(&g, 100.0);
        for _ in 0..60 {
            j.actualiser(&AUCUNE, &env);
        }
        assert_eq!(j.boite.x, 100.0 - 60.0, "poussé par le vent à gauche");
    }

    #[test]
    fn vent_lateral_double_en_planant() {
        let g = salle();
        let vents = [zone_vent(1)];
        let env = Environnement {
            grille: &g,
            solides: &[],
            courants: &[],
            vents: &vents,
            eaux: &[],
        };
        // En l'air, SAUT maintenu : le parapluie plane dès le premier pas
        // (vitesse verticale nulle au départ, donc `plane` est vrai).
        let mut j = Joueur::new(56.0, 100.0);
        for _ in 0..30 {
            j.actualiser(&SAUT, &env);
        }
        assert!(j.plane, "Pépin plane");
        // 120 px/s = 2 px par pas, le double du vent fermé.
        assert_eq!(j.boite.x, 56.0 + 60.0, "plané porté par le vent");
    }

    #[test]
    fn vent_lateral_en_chute_fermee_est_plus_faible() {
        let g = salle();
        let vents = [zone_vent(1)];
        let env = Environnement {
            grille: &g,
            solides: &[],
            courants: &[],
            vents: &vents,
            eaux: &[],
        };
        let mut ferme = Joueur::new(56.0, 100.0);
        let mut ouvert = Joueur::new(56.0, 100.0);
        for _ in 0..30 {
            ferme.actualiser(&AUCUNE, &env);
            ouvert.actualiser(&SAUT, &env);
        }
        assert!(!ferme.plane, "parapluie fermé en chute libre");
        assert!(ouvert.plane);
        assert!(
            ferme.boite.x < ouvert.boite.x,
            "le plané (parapluie ouvert) est poussé plus loin"
        );
        assert_eq!(ferme.boite.x, 56.0 + 30.0, "fermé : 60 px/s");
        assert_eq!(ouvert.boite.x, 56.0 + 60.0, "ouvert : 120 px/s");
    }

    #[test]
    fn deux_vents_opposes_qui_chevauchent_s_annulent() {
        let g = salle();
        let droit = zone_vent(1);
        let gauche = zone_vent(-1);
        let env = Environnement {
            grille: &g,
            solides: &[],
            courants: &[],
            vents: &[droit, gauche],
            eaux: &[],
        };
        let mut j = au_sol(&g, 56.0);
        for _ in 0..30 {
            j.actualiser(&AUCUNE, &env);
        }
        assert_eq!(j.boite.x, 56.0, "deux sens opposés : aucune poussée");
    }

    #[test]
    fn coursier_resiste_au_vent_de_face() {
        let g = salle();
        let vent = [zone_vent(-1)];
        let avec = Environnement {
            grille: &g,
            solides: &[],
            courants: &[],
            vents: &vent,
            eaux: &[],
        };
        let sans = Environnement::simple(&g);
        // Vent de face (vers la gauche) ; on court à DROITE dans les deux cas.
        let mut j_avant = au_sol(&g, 60.0);
        let mut j_avec = au_sol(&g, 60.0);
        for _ in 0..120 {
            j_avant.actualiser(&droite(), &sans);
            j_avec.actualiser(&droite(), &avec);
        }
        // Le vent de face retarde, mais ne bloque pas : on progresse toujours.
        assert!(
            j_avec.boite.x > 60.0 + 20.0,
            "progrès malgré le vent de face"
        );
        assert!(j_avec.boite.x < j_avant.boite.x, "le vent de face ralentit");
        // Fermer le parapluie, c'est résister : au sol le vent ne fait que 60,
        // pas 120 (la course à 96 gagne dans les deux cas).
        assert!(j_avec.au_sol);
    }

    #[test]
    fn rebond_et_placer() {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut j = au_sol(&g, 100.0);
        j.rebondir(400.0);
        assert_eq!(j.vitesse.y, -400.0);
        assert!(!j.au_sol);
        j.actualiser(&AUCUNE, &env);
        assert!(j.vitesse.y < 0.0);
        j.placer(30.0, 30.0);
        assert_eq!(
            (j.boite.x, j.boite.y, j.vitesse),
            (30.0, 30.0, Vec2::default())
        );
    }

    /// La protection de réapparition dure exactement `PROTECTION_REAPPARITION_PAS`
    /// pas (celui de la réapparition compris, donc `- 1` pas de simulation ensuite),
    /// puis expire ; un nouveau `placer` l'efface, `proteger` la remet au maximum
    /// sans jamais la cumuler.
    #[test]
    fn la_protection_de_reapparition_expire() {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut j = au_sol(&g, 100.0);
        assert!(!j.est_protege(), "pas de protection au départ");
        j.proteger();
        assert_eq!(j.protection_restante(), PROTECTION_REAPPARITION_PAS);
        for k in 1..PROTECTION_REAPPARITION_PAS {
            j.actualiser(&droite(), &env);
            assert!(j.est_protege(), "encore protégé au pas {k}");
        }
        j.actualiser(&droite(), &env);
        assert!(!j.est_protege(), "protection expirée");
        j.actualiser(&droite(), &env);
        assert_eq!(j.protection_restante(), 0, "reste à 0");

        j.proteger();
        j.actualiser(&AUCUNE, &env);
        j.proteger();
        assert_eq!(
            j.protection_restante(),
            PROTECTION_REAPPARITION_PAS,
            "pas de cumul"
        );
        j.placer(30.0, 30.0);
        assert!(!j.est_protege(), "placer seul ne protège pas");
        assert_eq!(PROTECTION_REAPPARITION_PAS, 60, "1 s à 60 Hz");
    }

    #[test]
    fn porter_deplace_avec_collisions() {
        let g = salle();
        let env = Environnement::simple(&g);
        let mut j = au_sol(&g, 100.0);
        j.porter(10.0, 0.0, &env);
        assert_eq!(j.boite.x, 110.0);
        j.porter(500.0, 0.0, &env);
        assert_eq!(j.boite.droite(), 19.0 * TAILLE_TUILE);
    }

    #[test]
    fn simulation_deterministe() {
        // Deux exécutions identiques donnent EXACTEMENT les mêmes nombres.
        let g = salle();
        let env = Environnement::simple(&g);
        let scenario = |n: usize| {
            let mut j = Joueur::new(40.0, 100.0);
            for i in 0..n {
                let e = Entrees {
                    direction: if i % 50 < 30 {
                        Direction::Droite
                    } else {
                        Direction::Gauche
                    },
                    saut: i % 40 < 15,
                    bas: false,
                };
                j.actualiser(&e, &env);
            }
            (j.boite, j.vitesse)
        };
        assert_eq!(scenario(500), scenario(500));
    }

    // --- Eau et barque (carte 9.2) : LE test de la mécanique centrale --------

    /// Salle avec un bassin calme : la surface de l'eau est à 32 px au-dessus
    /// du sol (y = 144), du haut de la colonne 6 à la colonne 9 (x 96 à 144).
    fn bassin() -> (GrilleTuiles, Eau) {
        let g = salle();
        let eau = Eau {
            zone: Aabb::new(96.0, 144.0, 48.0, 32.0),
            courant: 0,
        };
        (g, eau)
    }

    /// Le mécanisme (carte 9.2b, demande d'AGA) : le parapluie RETOURNÉ (BAS
    /// tenu) dans l'eau donne la barque (on flotte, on rame, on saute hors de
    /// l'eau) ; entrer SANS parapluie retourné noie Pépin, en PLANANT aussi.
    #[test]
    fn retourner_le_parapluie_dans_l_eau_donne_la_barque_sinon_noyade() {
        let (g, eau) = bassin();
        let env = Environnement {
            grille: &g,
            solides: &[],
            courants: &[],
            vents: &[],
            eaux: std::slice::from_ref(&eau),
        };

        // 1. BAS tenu : en l'air, le parapluie se retourne, sans planer (Pépin
        // tombe normalement) ; en touchant l'eau, il devient une barque qui
        // flotte à la surface.
        let mut j = Joueur::new(110.0, 120.0); // juste au-dessus de la surface (144)
        j.actualiser(&BAS, &env);
        assert_eq!(j.etat(), EtatJoueur::Retourne, "BAS en l'air : retourné");
        assert!(!j.plane, "parapluie retourné : pas de plané");
        for _ in 0..60 {
            j.actualiser(&BAS, &env);
            if j.en_barque() {
                break;
            }
        }
        assert!(j.en_barque(), "parapluie retourné dans l'eau : la barque");
        assert_eq!(j.etat(), EtatJoueur::Barque);
        assert_eq!(
            j.boite.bas(),
            eau.surface() + ENFONCEMENT_BARQUE,
            "les pieds flottent juste sous la surface"
        );

        // À bord, plus besoin de tenir BAS : la barque reste. La rame est
        // LENTE (40 % de la course) et le courant n'existe pas ici. 20 pas : la
        // vitesse de rame est atteinte (~10 pas à 240 px/s²) et la barque
        // (~10 px parcourus) reste dans ce bassin étroit de 48 px.
        for _ in 0..20 {
            j.actualiser(&droite(), &env);
        }
        assert!(j.boite.x > 110.0, "ramer pousse la barque");
        assert!(j.vitesse.x <= VITESSE_RAME + 1e-3, "la rame est lente");
        assert!(j.en_barque(), "la barque reste sans tenir BAS");

        // Le saut sort Pépin de l'eau (comme un saut au sol).
        j.actualiser(&SAUT, &env);
        assert!(!j.en_barque(), "saut depuis la barque : Pépin quitte l'eau");
        assert!(j.vitesse.y < 0.0, "le saut repousse vers le haut");

        // 2. Parapluie FERMÉ : même chute, Pépin COULE (la noyade est marquée).
        let mut j = Joueur::new(110.0, 120.0);
        for _ in 0..60 {
            j.actualiser(&AUCUNE, &env);
        }
        assert!(j.est_noye(), "parapluie fermé dans l'eau : noyade");

        // 3. En PLANANT (SAUT tenu, né en l'air : pas de sol, pas de coyote,
        // l'appui ne fait pas sauter) : il coule aussi. C'est la nouvelle règle
        // de la carte 9.2b (avant, planer donnait la barque).
        let mut j = Joueur::new(110.0, 120.0);
        for _ in 0..60 {
            j.actualiser(&SAUT, &env);
            assert!(!j.en_barque(), "planer ne donne plus la barque");
        }
        assert!(j.est_noye(), "planer dans l'eau : noyade");

        // 4. SAUT et BAS ensemble : le plané l'emporte, donc la noyade aussi.
        let mut j = Joueur::new(110.0, 120.0);
        let les_deux = Entrees { saut: true, ..BAS };
        j.actualiser(&les_deux, &env);
        assert_eq!(j.etat(), EtatJoueur::Plane, "saut et bas : on plane");
        for _ in 0..60 {
            j.actualiser(&les_deux, &env);
        }
        assert!(j.est_noye(), "planant (bas en plus) : noyade");

        // 5. Même chute parapluie fermé, mais PROTÉGÉ (il vient de réapparaître,
        // par exemple à une lanterne posée sur l'eau) : il ne coule pas, l'eau
        // le recueille en barque (exception gardée).
        let mut j = Joueur::new(110.0, 120.0);
        j.proteger();
        for _ in 0..60 {
            j.actualiser(&AUCUNE, &env);
        }
        assert!(!j.est_noye(), "protégé, Pépin ne coule pas");
        assert!(
            j.en_barque(),
            "réapparu au-dessus de l'eau : la barque le recueille"
        );

        // 6. Loin de l'eau, BAS ne change RIEN au mouvement (Monde 1
        // inchangé) : même course, même saut, parapluie retourné visible.
        let g = salle();
        let env = Environnement::simple(&g);
        let mut avec = au_sol(&g, 40.0);
        let mut sans = avec.clone();
        for pas in 0..120 {
            let e = Entrees {
                direction: Direction::Droite,
                saut: (10..25).contains(&pas),
                bas: false,
            };
            sans.actualiser(&e, &env);
            avec.actualiser(&Entrees { bas: true, ..e }, &env);
            assert_eq!(avec.boite, sans.boite, "pas {pas} : même trajectoire");
        }
        assert_eq!(
            avec.etat(),
            EtatJoueur::Retourne,
            "au sol aussi, il se voit"
        );
        avec.actualiser(&AUCUNE, &env);
        assert_ne!(avec.etat(), EtatJoueur::Retourne, "BAS lâché : refermé");
    }

    /// Correctif d'AGA : l'eau compte À LA HAUTEUR de Pépin, pas seulement dans
    /// sa colonne. Deux étendues empilées (égouts, cascade) : un bassin
    /// suspendu (y 48 à 80, x 120 à 160) au-dessus d'un bassin bas (y 144 à 176).
    #[test]
    fn l_eau_ne_compte_qu_a_la_hauteur_de_pepin() {
        let g = salle();
        let suspendu = Eau {
            zone: Aabb::new(120.0, 48.0, 40.0, 32.0),
            courant: 0,
        };
        let bas = Eau {
            zone: Aabb::new(40.0, 144.0, 200.0, 32.0),
            courant: 0,
        };
        // Rangées comme le parseur : du plus haut au plus bas.
        let eaux = [suspendu, bas];

        // 1. Marcher au sol SOUS le bassin suspendu (rien d'autre) : pas de
        // noyade. Avant le correctif, Pépin mourait en passant dessous.
        let env = Environnement {
            grille: &g,
            solides: &[],
            courants: &[],
            vents: &[],
            eaux: std::slice::from_ref(&suspendu),
        };
        let mut j = Joueur::new(40.0, SOL_Y - JOUEUR_H);
        for _ in 0..240 {
            j.actualiser(&droite(), &env);
            assert!(
                !j.est_noye(),
                "au sol, cinq rangées sous l'eau : pas de noyade"
            );
        }
        assert!(j.boite.x > 170.0, "Pépin est bien passé sous le bassin");

        // 2. Une barque sur le bassin bas passe sous le bassin suspendu : elle
        // reste sur SA surface. Avant le correctif, elle était téléportée en haut.
        let env = Environnement {
            grille: &g,
            solides: &[],
            courants: &[],
            vents: &[],
            eaux: &eaux,
        };
        let mut j = Joueur::new(60.0, bas.surface() - JOUEUR_H + ENFONCEMENT_BARQUE);
        j.entrer_en_barque();
        for _ in 0..220 {
            j.actualiser(&droite(), &env);
            assert!(j.en_barque(), "la barque reste dans l'eau");
            assert_eq!(
                j.boite.bas(),
                bas.surface() + ENFONCEMENT_BARQUE,
                "la barque reste sur la surface du bassin bas"
            );
        }
        assert!(
            j.boite.x > 160.0,
            "la barque est bien passée sous le bassin"
        );
    }
}
