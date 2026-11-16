//! Module `poursuite` : le MUR DE PLUIE du niveau de poursuite (carte 8.2).
//!
//! Un rideau de pluie avance depuis la gauche à vitesse constante et tue Pépin au
//! contact : il faut courir devant lui jusqu'au soleil. Dans la grille, la lettre `P`
//! (une seule, sur n'importe quelle rangée) place son FRONT au départ : le bord droit
//! de la case `P` (voir la légende de `crate::level`).
//!
//! Les règles, toutes ici (le jeu ne fait que les appeler à chaque pas) :
//! - le front avance de `VITESSE_MUR` px/s (au plus 80 % de la course de Pépin) ;
//! - il ne reste jamais plus de `RETARD_MAX` px derrière Pépin : si Pépin le distance,
//!   le mur est tiré derrière lui (rattrapage). Ainsi il est toujours à l'écran, la
//!   caméra défile avec lui (`game/src/camera.rs`) et la pression ne retombe jamais ;
//! - il tue dès que son front dépasse le bord gauche de Pépin (`touche`) ;
//! - après une mort, Pépin réapparaît à sa lanterne et le front est replacé
//!   `RECUL_REAPPARITION` px derrière lui (`replacer`) : toujours le même délai pour
//!   repartir, donc jamais de mort en boucle ;
//! - quand Pépin arrive à `PERCEE_AVANT_ARRIVEE` px du drapeau (la dernière ligne
//!   droite), le soleil perce : le mur s'arrête et recule (`RECUL_SOLEIL` px/s), il
//!   ne tue plus ; la caméra redevient libre.
//!
//! Notions Rust illustrées ici : une petite `struct` dont les règles sont gardées par
//! ses méthodes (le champ `front` est en lecture seule hors du module), `Option` pour
//! « ce niveau n'a pas de mur », constantes calculées à la compilation.

use crate::aabb::Aabb;
use crate::level::Niveau;
use crate::physics::{DT, VITESSE_COURSE};

/// Vitesse du front (px/s) : 80 % de la course de Pépin (96 px/s), soit 4,8 tuiles
/// par seconde (carte 8.16 : 64 avant, « accélère la poursuite »). Pépin le distance
/// en courant, pas en s'arrêtant.
pub const VITESSE_MUR: f32 = 76.8;
/// La vitesse du mur ne dépasse pas 80 % de celle de la course (vérifié à la
/// compilation : changer `VITESSE_MUR` au-delà fait échouer le build).
const _: () = assert!(VITESSE_MUR <= 0.8 * VITESSE_COURSE);
/// Distance maximale (px) entre le front et Pépin (son bord gauche) : 6 tuiles,
/// 1,25 s d'avance au plus (carte 8.16 : 128 px avant, la pression se sent). Pépin
/// courant au centre de l'écran, le front reste visible au bord gauche (la caméra
/// regarde 28 px devant lui).
pub const RETARD_MAX: f32 = 96.0;
/// Distance (px) du front derrière Pépin quand il réapparaît : 6 tuiles (1,25 s).
/// Pas plus que `RETARD_MAX` : au pas suivant, le mur serait tiré à `RETARD_MAX`.
pub const RECUL_REAPPARITION: f32 = 96.0;
const _: () = assert!(RECUL_REAPPARITION <= RETARD_MAX);
/// Le soleil perce quand le bord gauche de Pépin arrive à cette distance (px) du
/// drapeau : 24 tuiles, environ 4 s de course au soleil avant l'arrivée.
pub const PERCEE_AVANT_ARRIVEE: f32 = 384.0;
/// Vitesse (px/s) à laquelle le mur recule une fois le soleil percé.
pub const RECUL_SOLEIL: f32 = 48.0;

/// Le mur de pluie d'un niveau de poursuite.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MurDePluie {
    /// Abscisse (px) du front : tout ce qui est à sa gauche est sous le mur.
    front: f32,
    /// Abscisse de Pépin (bord gauche) où le soleil perce.
    percee: f32,
    /// Pas écoulés depuis que le soleil a percé (`None` avant) : le mur recule et
    /// ne tue plus personne ; le jeu s'en sert pour faire grandir le soleil.
    soleil: Option<u32>,
}

impl MurDePluie {
    /// Le mur d'un niveau : `None` si sa grille n'a pas de `P`. Le soleil perce
    /// `PERCEE_AVANT_ARRIVEE` px avant le drapeau (jamais, sans drapeau).
    pub fn du_niveau(niveau: &Niveau) -> Option<MurDePluie> {
        let front = niveau.mur_de_pluie?;
        let percee = niveau
            .arrivee
            .map_or(f32::INFINITY, |a| a.x - PERCEE_AVANT_ARRIVEE);
        Some(MurDePluie {
            front,
            percee,
            soleil: None,
        })
    }

    /// Abscisse du front (px).
    pub fn front(&self) -> f32 {
        self.front
    }

    /// Le soleil a-t-il percé (mur arrêté, qui recule) ?
    pub fn vaincu(&self) -> bool {
        self.soleil.is_some()
    }

    /// Pas écoulés depuis que le soleil a percé (`None` avant).
    pub fn pas_soleil(&self) -> Option<u32> {
        self.soleil
    }

    /// Un pas (1/60 s) : le front avance et rattrape Pépin s'il est trop loin devant ;
    /// une fois le soleil percé (Pépin dans la dernière ligne droite), il recule.
    pub fn avancer(&mut self, joueur: &Aabb) {
        if let Some(t) = &mut self.soleil {
            *t += 1;
            self.front -= RECUL_SOLEIL * DT;
            return;
        }
        if joueur.x >= self.percee {
            self.soleil = Some(0);
            return;
        }
        self.front = (self.front + VITESSE_MUR * DT).max(joueur.x - RETARD_MAX);
    }

    /// Le mur touche-t-il Pépin (son front a passé le bord gauche de sa boîte) ?
    pub fn touche(&self, joueur: &Aabb) -> bool {
        self.soleil.is_none() && self.front > joueur.x
    }

    /// Après une mort, Pépin replacé à sa lanterne : le front recule à
    /// `RECUL_REAPPARITION` px derrière lui (ou reste où il est s'il était plus loin).
    pub fn replacer(&mut self, joueur: &Aabb) {
        self.front = self.front.min(joueur.x - RECUL_REAPPARITION);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Toutes les règles du mur sur un petit niveau : il avance à sa vitesse, rattrape
    /// Pépin qui le distance, le touche quand il s'arrête, recule après une mort,
    /// s'arrête puis recule devant le soleil.
    #[test]
    fn le_mur_avance_rattrape_tue_recule_et_s_arrete_au_soleil() {
        let mut lignes = ["...................................................."; 11];
        lignes[8] = "...........o.o.o....................................";
        lignes[9] = "P..@..............................................X.";
        lignes[10] = "####################################################";
        let niveau = Niveau::depuis_ascii(&lignes.join("\n")).unwrap();
        let mut mur = MurDePluie::du_niveau(&niveau).expect("un `P` : un mur");
        assert_eq!(mur.front(), 16.0, "bord droit de la case `P`");
        let sans = lignes.join("\n").replace('P', ".");
        assert!(MurDePluie::du_niveau(&Niveau::depuis_ascii(&sans).unwrap()).is_none());

        // 1 s : `VITESSE_MUR` px, Pépin immobile plus loin (à moins de `RETARD_MAX`
        // du départ : le mur n'est pas tiré) n'est pas touché.
        let mut pepin = Aabb::new(110.0, 146.0, 10.0, 14.0);
        for _ in 0..60 {
            mur.avancer(&pepin);
        }
        assert!(
            (mur.front() - (16.0 + VITESSE_MUR)).abs() < 0.01,
            "{}",
            mur.front()
        );
        assert!(!mur.touche(&pepin));
        // Pépin file loin devant : le mur est tiré à RETARD_MAX derrière lui.
        pepin.x = 400.0;
        mur.avancer(&pepin);
        assert_eq!(mur.front(), 400.0 - RETARD_MAX);
        // Pépin s'arrête : le mur le touche en `RETARD_MAX / VITESSE_MUR` s (1,25 s),
        // pas avant.
        let mut pas = 0;
        while !mur.touche(&pepin) {
            mur.avancer(&pepin);
            pas += 1;
        }
        let attendu = (RETARD_MAX / (VITESSE_MUR * DT)).round() as u32;
        assert!((attendu..=attendu + 1).contains(&pas), "{pas} pas");
        // Mort : Pépin réapparaît à sa lanterne, le mur recule derrière lui.
        pepin.x = 300.0;
        mur.replacer(&pepin);
        assert_eq!(mur.front(), 300.0 - RECUL_REAPPARITION);
        assert!(!mur.touche(&pepin));
        // Pépin à 24 tuiles du drapeau : le soleil perce, le mur s'arrête (sans
        // rattraper Pépin), puis recule et ne tue plus.
        let avant = mur.front();
        pepin.x = niveau.arrivee.unwrap().x - PERCEE_AVANT_ARRIVEE;
        mur.avancer(&pepin);
        assert!(mur.vaincu());
        assert_eq!(mur.front(), avant);
        pepin.x = 0.0;
        mur.avancer(&pepin);
        assert!(mur.front() < avant && !mur.touche(&pepin));
        assert_eq!(mur.pas_soleil(), Some(1));
    }
}
