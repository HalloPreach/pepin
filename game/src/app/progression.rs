//! La progression dans un niveau, pas après pas (carte 10.2b) : le monde avance,
//! le fantôme de la course aussi, les défis sont notés, et le niveau se termine
//! (gouttes créditées, médailles, bilan, niveau suivant) ou recommence (« plus de
//! vies »). Le mode facile du Grand Nuage compte ici les défaites au boss.
//!
//! La caméra et les effets du pas sont branchés dans `camera_effets.rs`, les
//! changements d'écran qui suivent dans `transitions.rs`.

use pepin_core::levels::{self, Apres, GenreNiveau};
use pepin_core::player::Entrees;

use crate::course;
use crate::defis;
use crate::ecrans::bilan_course::TempsCourse;
use crate::ecrans::bilan_niveau::Temps;
use crate::ecrans::{plus_de_vies, Ecran};
use crate::facile::{self as mode_facile, Origine, Proposition};
use crate::{hud, mort};

use super::transitions::Suite;
use super::Application;

impl Application {
    pub(super) fn pas_jeu(&mut self, entrees: &Entrees) {
        let (largeur, hauteur) = (
            self.monde.niveau.largeur_px(),
            self.monde.niveau.hauteur_px(),
        );
        let vies_avant = self.monde.etat.vies.restantes();
        self.monde.avancer(entrees);
        self.trainees
            .actualiser(self.cosmetiques.trainee, &self.monde);
        // Course (carte 8.7) : le fantôme fait le même pas, dans son propre monde.
        if let Some(f) = &mut self.fantome {
            f.avancer();
        }
        self.eclat_vie = self.eclat_vie.saturating_sub(1);
        if self.monde.etat.vies.restantes() > vies_avant {
            self.eclat_vie = hud::DUREE_ECLAT_VIE;
        }
        // Caméra et effets du pas (secousse, particules) : `camera_effets.rs`.
        self.cadrer_et_animer(largeur, hauteur);
        self.pas_partie += 1;
        // Défis (carte 7.3) : on regarde seulement, rien ne change dans le jeu.
        self.tentative.pas += 1;
        self.tentative.a_plane |= self.monde.joueur.plane;
        self.tentative.morts = self.monde.etat.morts - self.morts_debut_tentative;
        self.compter_defaite_au_boss();
        // La dernière vie perdue : l'écran « plus de vies », puis le niveau recommence
        // (avant tout, même si ce pas-là finissait aussi le niveau). Après l'animation
        // de la mort (carte 9.24), le rideau fermé.
        if self.monde.etat.vies.plus_de_vies() && self.monde.reapparition <= mort::RIDEAU_FERME {
            self.game_over_partie = true;
            self.ouvrir(Ecran::PlusDeVies(plus_de_vies::DUREE_PAS));
            return;
        }
        if self.monde.termine {
            // Niveau réussi (bonus compris) : ses gouttes NOUVELLES vont au
            // porte-monnaie, tout de suite enregistré (une goutte déjà comptée ne
            // rapporte plus, carte 7.9). Celles d'un niveau raté ne comptent pas : elles
            // sont remises à zéro quand le niveau recommence. Les défis (carte 7.3) sont
            // notés au passage, et leurs premières réussites créditées.
            self.tentative.gouttes = self.monde.gouttes_niveau;
            // La lettre cachée (carte 8.6) d'un niveau réussi : gardée pour toujours.
            if self.monde.lettre_prise {
                self.sauvegarde.noter_lettre(self.monde.infos.cle);
            }
            let ancien = self.sauvegarde.progres(self.monde.infos.cle).meilleur_temps;
            // Boss vaincu en mode facile (carte 8.14) : fini, gouttes créditées, mais
            // ni défi ni médaille ni meilleur temps.
            let r = if self.facile.actif && self.monde.pv_nuage().is_some() {
                self.facile.dans_la_partie = true;
                mode_facile::noter_victoire_facile(
                    &mut self.sauvegarde,
                    self.monde.infos,
                    &self.tentative,
                )
            } else {
                defis::noter_niveau(&mut self.sauvegarde, self.monde.infos, &self.tentative)
            };
            self.resultat_niveau = Some(r);
            // Nouvelle partie + (carte 8.8) : gouttes et défis comptent comme d'habitude,
            // mais le temps du niveau va dans ses records à elle.
            if self.ng_plus {
                let cle = self.monde.infos.cle;
                if let Some(p) = self.sauvegarde.progres_mut(cle) {
                    p.meilleur_temps = ancien;
                    self.sauvegarde.ng_plus.noter_temps(cle, self.tentative.pas);
                }
            }
            self.sauvegarde.enregistrer();
            // Un niveau joué seul (carte 7.4) : son bilan, par-dessus le niveau figé.
            if self.niveau_seul {
                // Une course (carte 8.7) : son bilan à elle (gagné ou perdu, l'écart).
                if let Some(f) = &self.fantome {
                    let robot = f.temps();
                    self.resultat_course = Some(course::noter_course(
                        &mut self.sauvegarde,
                        self.monde.infos.cle,
                        f.force,
                        self.tentative.pas,
                        robot,
                    ));
                    self.sauvegarde.enregistrer();
                    self.temps_course = TempsCourse {
                        joueur: self.tentative.pas,
                        robot,
                    };
                    self.ouvrir(Ecran::BilanCourse);
                    return;
                }
                self.temps_niveau = Temps {
                    pas: self.tentative.pas,
                    ancien,
                };
                self.ouvrir(Ecran::BilanNiveau);
                return;
            }
            // Le bonus fini : ses gouttes sont mises à part (compteur, écran de fin).
            if self.monde.infos.genre == GenreNiveau::Bonus {
                self.gouttes_bonus = Some(self.gouttes_du_niveau());
            }
            // Niveau suivant (le score est gardé), en sautant le bonus s'il manque des
            // gouttes ; après l'épilogue ou le dernier niveau : l'écran de fin.
            self.lancer(
                match levels::apres(
                    niveaux::niveaux(),
                    self.monde.indice,
                    self.monde.etat.gouttes,
                ) {
                    Apres::Niveau(indice) => Suite::Niveau {
                        indice,
                        etat: Some(self.monde.etat.clone()),
                    },
                    Apres::Fin => Suite::Fin,
                },
            );
        }
    }

    // --- Mode facile du Grand Nuage (carte 8.14) ------------------------------------

    /// Une mort au boss en mode normal : une défaite de plus dans la visite (et dans
    /// la statistique enregistrée) ; à partir de `DEFAITES_AVANT_FACILE` défaites de
    /// la VISITE (carte 9.34), la bannière « TROP DUR ? » s'ouvre (pas pendant « plus
    /// de vies », ni si NON a déjà été répondu dans la visite, ni pour un boss déjà
    /// battu en normal). La démo du titre ne
    /// passe pas par ici (`pas_demo`), et une course ne compte pas.
    fn compter_defaite_au_boss(&mut self) {
        // Une fois l'animation de la mort jouée, le rideau fermé (carte 9.24) : la
        // bannière ne la cache pas.
        let mort = self.monde.reapparition == mort::RIDEAU_FERME;
        // Une course contre le robot (carte 8.7) ne compte pas : on y rejoue pour le chrono.
        if !mort || self.facile.actif || self.fantome.is_some() || self.monde.pv_nuage().is_none() {
            return;
        }
        mode_facile::noter_defaite(&mut self.sauvegarde, self.monde.infos.cle);
        self.sauvegarde.enregistrer();
        self.facile.defaites_visite = self.facile.defaites_visite.saturating_add(1);
        if mode_facile::proposer_apres_une_mort(mode_facile::ApresUneMort {
            defaites_visite: self.facile.defaites_visite,
            refusee: self.facile.refusee,
            plus_de_vies: self.monde.etat.vies.plus_de_vies(),
            battu_en_normal: self
                .sauvegarde
                .progres(self.monde.infos.cle)
                .battu_en_normal(),
        }) {
            self.facile.proposition = Some(Proposition::new(Origine::Mort));
        }
    }

    /// La phrase de la bannière (carte 9.34) : le `facile_texte` du niveau du boss,
    /// celui qu'on joue (après une mort) ou celui choisi sur la carte des niveaux.
    pub(super) fn facile_texte(&self, origine: Origine) -> &'static str {
        match origine {
            Origine::Mort => self.monde.infos.facile_texte,
            Origine::Plan(indice) => niveaux::niveaux()
                .get(indice)
                .map_or("", |n| n.facile_texte),
        }
    }
}
