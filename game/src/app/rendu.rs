//! Le dessin de l'écran courant (carte 10.2b) : le monde (`crate::scene`), puis
//! l'interface en jeu (`crate::hud`), l'écran à menu par-dessus (`crate::ecrans`),
//! la bannière du mode facile et enfin les volets de la transition.
//!
//! Seul endroit d'`app` qui appelle macroquad : tout le reste se teste sans fenêtre.

use pepin_core::levels::{GenreNiveau, NIVEAUX};
use pepin_core::physics::HZ;

use crate::course;
use crate::demo;
use crate::ecrans::bilan_course;
use crate::ecrans::bilan_niveau;
use crate::ecrans::fin;
use crate::ecrans::histoire;
use crate::ecrans::{
    self, aide, boutique as ecran_boutique, carte, course_force, pause, plan as ecran_plan,
    plus_de_vies, resultats, titre, Disposition, Ecran,
};
use crate::sprites::Banque;
use crate::{contenu, hud, scene};

use super::Application;

impl Application {
    /// Dessine l'écran courant dans la cible logique. `alpha` : interpolation entre deux pas.
    pub fn dessiner(&self, banque: &Banque, alpha: f32) {
        // Monde figé (pause, transition, carte) : on montre le dernier pas tel quel,
        // sinon l'interpolation ferait trembler les sprites sur place.
        let anime = matches!(self.ecran, Ecran::Jeu | Ecran::Demo) && !self.transition.est_active();
        let alpha = if anime { alpha } else { 1.0 };
        let libelles: Vec<&str> = self.choix.iter().map(|&c| contenu::libelle(c)).collect();
        let dispo = ecrans::disposition_menu(self.ecran, self.choix.len())
            .unwrap_or(Disposition { y0: 0.0, pas: 20.0 });
        match self.ecran {
            Ecran::Titre => {
                titre::dessiner(banque, self.skin, self.anim, &libelles, &self.menu, &dispo);
                titre::dessiner_poche(banque, &self.sauvegarde);
                if self.son_disponible {
                    ecrans::dessiner_icone_son(self.son);
                }
            }
            Ecran::Aide => aide::dessiner(),
            Ecran::Histoire(chapitre) => histoire::dessiner(banque, self.skin, chapitre, self.anim),
            Ecran::Boutique => {
                ecran_boutique::dessiner(banque, self.anim, &self.sauvegarde, &self.boutique)
            }
            Ecran::Fin => fin::dessiner(
                &self.bilan,
                self.anim,
                &self.particules,
                &libelles,
                &self.menu,
                &dispo,
                self.nouveau_record,
            ),
            Ecran::Plan => {
                ecran_plan::dessiner(banque, self.skin, self.anim, &self.sauvegarde, &self.plan)
            }
            // Course (carte 8.7) : le choix de la force, par-dessus la carte des niveaux.
            Ecran::ChoixForce => {
                ecran_plan::dessiner(banque, self.skin, self.anim, &self.sauvegarde, &self.plan);
                course_force::dessiner(
                    &NIVEAUX[self.plan.selection],
                    &self.sauvegarde,
                    &self.choix,
                    &libelles,
                    &self.menu,
                    &dispo,
                    self.anim,
                );
            }
            Ecran::Jeu
            | Ecran::Carte(_)
            | Ecran::Pause
            | Ecran::PlusDeVies(_)
            | Ecran::BilanNiveau
            | Ecran::BilanCourse
            | Ecran::Demo => {
                // Pépin et la caméra sont arrondis ensemble (écart arrondi une seule fois) : pas de
                // tremblement d'un pixel en courant (ni en tombant, dans un niveau haut).
                let pepin = self.monde.joueur_interpole(alpha);
                let (cx, cy) = self.camera.position_rendue(alpha, pepin);
                let (sx, sy) = self.secousse.decalage();
                scene::dessiner(
                    &self.monde,
                    banque,
                    &self.particules,
                    &self.trainees,
                    self.skin,
                    alpha,
                    cx + sx,
                    cy + sy,
                );
                // Course (carte 8.7) : le fantôme par-dessus, tant qu'il court.
                if let Some(f) = self.fantome.as_ref().filter(|f| f.visible()) {
                    scene::dessiner_fantome(
                        &f.monde,
                        banque,
                        alpha,
                        (cx + sx, cy + sy),
                        f.force.libelle(),
                    );
                }
                // Sous la trame (carte, pause) le HUD serait illisible : on ne le montre qu'en jeu.
                if self.ecran == Ecran::Jeu {
                    // Niveau joué seul (carte 7.4) : les gouttes de CE niveau, sur 3.
                    let compteur = hud::compteur(
                        self.monde.infos.monde,
                        self.niveau_seul || self.monde.infos.genre == GenreNiveau::Bonus,
                        self.monde.etat.gouttes,
                        self.gouttes_bonus,
                        self.gouttes_du_niveau(),
                    );
                    let vies = hud::vies(self.monde.etat.vies.restantes(), self.eclat_vie);
                    // Niveau joué seul (carte 7.4) : le chrono de la tentative et le temps
                    // cible du défi CHRONO.
                    let (pas, cible) = if self.niveau_seul {
                        let c = u32::from(self.monde.infos.duree_cible_s) * HZ;
                        (self.tentative.pas, Some(c))
                    } else {
                        (self.pas_partie, None)
                    };
                    hud::dessiner(
                        banque,
                        self.skin,
                        compteur,
                        &vies,
                        pas,
                        cible,
                        self.monde.points_nuage(),
                        self.monde.ng_plus,
                        self.monde.boss_pieuvre(),
                    );
                    // Course (carte 8.7) : l'état du fantôme sous le chrono.
                    if let Some(f) = &self.fantome {
                        hud::dessiner_ligne_course(&course::ligne_hud(f));
                    }
                }
                match self.ecran {
                    Ecran::Carte(_) => carte::dessiner(
                        self.monde.indice,
                        self.monde.infos,
                        self.anim,
                        &self.sauvegarde.lettres(),
                    ),
                    Ecran::Pause => {
                        pause::dessiner(self.anim, &libelles, &self.menu, &dispo);
                        // En jeu, pas d'icône : le coin est aux commandes tactiles et au HUD.
                        if self.son_disponible {
                            ecrans::dessiner_icone_son(self.son);
                        }
                    }
                    Ecran::PlusDeVies(_) => plus_de_vies::dessiner(banque, self.anim),
                    Ecran::Demo => demo::dessiner_bandeau(self.anim),
                    Ecran::BilanCourse => {
                        if let (Some(c), Some(r)) = (&self.resultat_course, &self.resultat_niveau) {
                            bilan_course::dessiner(
                                &self.temps_course,
                                c,
                                r,
                                self.anim,
                                &libelles,
                                &self.menu,
                                &dispo,
                            );
                        }
                    }
                    Ecran::BilanNiveau => {
                        if let Some(r) = &self.resultat_niveau {
                            bilan_niveau::dessiner(
                                &self.temps_niveau,
                                r,
                                self.anim,
                                &libelles,
                                &self.menu,
                                &dispo,
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
        // Défis et bonus (carte 7.3), sous la carte du niveau suivant ou l'écran de fin.
        match (self.ecran, &self.resultat_niveau) {
            (Ecran::Carte(_), Some(r)) => resultats::dessiner_bande_niveau(r, self.anim),
            (Ecran::Fin, r) => {
                resultats::dessiner_bande_fin(r.as_ref(), &self.bonus_fin, self.anim)
            }
            _ => {}
        }
        // Bannière du mode facile (carte 8.14), par-dessus le jeu ou la carte des niveaux.
        if let Some(p) = &self.facile.proposition {
            ecrans::facile::dessiner(p);
        }
        self.transition.dessiner();
    }
}
