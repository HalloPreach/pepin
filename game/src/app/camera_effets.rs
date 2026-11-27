//! Le branchement de la caméra et des effets dans un pas de jeu (carte 10.2b).
//!
//! La caméra elle-même (suivre, recentrer, figer, défiler avec le mur) est dans
//! `crate::camera` ; les effets (particules, secousse) dans `crate::effets`. Ce
//! fichier dit seulement QUAND on les appelle, à chaque pas du niveau en cours.
//! Pour changer un effet visuel : `crate::effets` ou `crate::particules`, sans rien
//! lire de la progression (niveaux finis, sauvegarde).
//!
//! Notion Rust : un bloc `impl Application` peut vivre dans un sous-module ; il voit
//! les champs privés de `Application` car ils sont visibles dans `app` et ses
//! sous-modules.

use crate::effets::effets;
use crate::mort;

use super::Application;

impl Application {
    /// Après le pas du monde : la caméra suit (ou attend) Pépin, puis les effets
    /// visuels du pas (particules, secousse) sont lancés depuis `crate::effets`.
    /// `largeur`, `hauteur` : taille du niveau en pixels, mesurée avant le pas.
    pub(super) fn cadrer_et_animer(&mut self, largeur: f32, hauteur: f32) {
        // Temps de mort (carte 9.24) : la caméra reste sur l'endroit de la mort (elle
        // ne suit pas une chute dans le vide), puis saute à la lanterne quand le rideau
        // est fermé.
        // Carte 9.13e : la mer de la montée, que la caméra garde dans le cadre.
        self.camera.mer = crate::mer_montee::surface(&self.monde);
        // Carte 9.36c : la fuite de la Pieuvre au sommet mène la caméra (vers la mer).
        if self.fuite_au_sommet.is_some() {
            self.mener_la_camera();
        } else if self.monde.reapparition > mort::RIDEAU_FERME {
            self.camera.figer();
        } else if self.monde.reapparition == mort::RIDEAU_FERME {
            self.camera.recentrer(&self.monde.joueur, largeur, hauteur);
        } else {
            self.camera.suivre(&self.monde.joueur, largeur, hauteur);
        }
        if let Some(mur) = self.monde.mur.filter(|m| !m.vaincu()) {
            self.camera.defiler_avec_le_mur(mur.front(), largeur);
        }
        effets(
            &self.monde,
            &mut self.particules,
            &mut self.secousse,
            self.camera.x,
            self.camera.y,
        );
    }
}
