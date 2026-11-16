//! Module `poursuite` : le MUR DE PLUIE du niveau de poursuite (carte 8.2).
//!
//! Un rideau de pluie avance depuis la gauche à vitesse constante et tue Pépin au
//! contact : il faut courir devant lui jusqu'au soleil. Dans la grille, la lettre `P`
//! (une seule, sur n'importe quelle rangée) place son FRONT au départ : le bord droit
//! de la case `P` (voir la légende de `crate::level`).
//!
//! Les règles, toutes ici (le jeu ne fait que les appeler à chaque pas) :
//! - le front avance de `VITESSE_MUR` px/s (moins de 80 % de la course de Pépin) ;
//! - il ne reste jamais plus de `RETARD_MAX` px derrière Pépin : si Pépin le distance,
//!   le mur est tiré derrière lui (rattrapage). Ainsi il est toujours à l'écran, la
//!   caméra défile avec lui (`game/src/camera.rs`) et la pression ne retombe jamais ;
//! - il tue dès que son front dépasse le bord gauche de Pépin (`touche`) ;
//! - après une mort, Pépin réapparaît à sa lanterne et le front est replacé
//!   `RECUL_REAPPARITION` px derrière lui (`replacer`) : toujours le même délai pour
//!   repartir, donc jamais de mort en boucle ;
//! - près de l'arrivée (`ARRET_AVANT_SOLEIL` px avant le drapeau), le soleil perce :
//!   le mur s'arrête et recule (`RECUL_SOLEIL` px/s), la caméra est libre.
//!
//! Notions Rust illustrées ici : une petite `struct` dont les règles sont gardées par
//! ses méthodes (le champ `front` est en lecture seule hors du module), `Option` pour
//! « ce niveau n'a pas de mur », constantes calculées à la compilation.

use crate::aabb::Aabb;
use crate::level::Niveau;
use crate::physics::{DT, VITESSE_COURSE};

/// Vitesse du front (px/s) : 2/3 de la course de Pépin (96 px/s), soit 4 tuiles par
/// seconde. Pépin le distance en courant, pas en s'arrêtant.
pub const VITESSE_MUR: f32 = 64.0;
/// La vitesse du mur ne dépasse pas 80 % de celle de la course (vérifié à la
/// compilation : changer `VITESSE_MUR` au-delà fait échouer le build).
const _: () = assert!(VITESSE_MUR <= 0.8 * VITESSE_COURSE);
/// Distance maximale (px) entre le front et Pépin (son bord gauche) : 8 tuiles, 2 s
/// d'avance au plus. Pépin courant au centre de l'écran, le front reste visible au
/// bord gauche (la caméra regarde 28 px devant lui).
pub const RETARD_MAX: f32 = 128.0;
/// Distance (px) du front derrière Pépin quand il réapparaît : 7 tuiles (1,75 s).
pub const RECUL_REAPPARITION: f32 = 112.0;
/// Le mur s'arrête quand son front arrive à cette distance (px) du drapeau : 14 tuiles.
pub const ARRET_AVANT_SOLEIL: f32 = 224.0;
/// Vitesse (px/s) à laquelle le mur recule une fois le soleil percé.
pub const RECUL_SOLEIL: f32 = 48.0;

/// Le mur de pluie d'un niveau de poursuite.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MurDePluie {
    /// Abscisse (px) du front : tout ce qui est à sa gauche est sous le mur.
    front: f32,
    /// Abscisse du front où le soleil perce : le mur s'arrête là.
    arret: f32,
    /// Pas écoulés depuis que le soleil a percé (`None` avant) : le mur recule et
    /// ne tue plus personne ; le jeu s'en sert pour faire grandir le soleil.
    soleil: Option<u32>,
}

impl MurDePluie {
    /// Le mur d'un niveau : `None` si sa grille n'a pas de `P`. Il s'arrête
    /// `ARRET_AVANT_SOLEIL` px avant le drapeau (jamais, sans drapeau).
    pub fn du_niveau(niveau: &Niveau) -> Option<MurDePluie> {
        let front = niveau.mur_de_pluie?;
        let arret = niveau
            .arrivee
            .map_or(f32::INFINITY, |a| a.x - ARRET_AVANT_SOLEIL);
        Some(MurDePluie {
            front,
            arret,
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

    /// Un pas (1/60 s) : le front avance, rattrape Pépin s'il est trop loin devant,
    /// s'arrête au soleil puis recule.
    pub fn avancer(&mut self, joueur: &Aabb) {
        if let Some(t) = &mut self.soleil {
            *t += 1;
            self.front -= RECUL_SOLEIL * DT;
            return;
        }
        self.front = (self.front + VITESSE_MUR * DT).max(joueur.x - RETARD_MAX);
        if self.front >= self.arret {
            self.front = self.arret;
            self.soleil = Some(0);
        }
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
        let mut lignes = vec!["...................................................."; 11];
        lignes[8] = "...........o.o.o....................................";
        lignes[9] = "P..@..............................................X.";
        lignes[10] = "####################################################";
        let niveau = Niveau::depuis_ascii(&lignes.join("\n")).unwrap();
        let mut mur = MurDePluie::du_niveau(&niveau).expect("un `P` : un mur");
        assert_eq!(mur.front(), 16.0, "bord droit de la case `P`");
        let sans = lignes.join("\n").replace('P', ".");
        assert!(MurDePluie::du_niveau(&Niveau::depuis_ascii(&sans).unwrap()).is_none());

        // 1 s : 64 px, Pépin immobile plus loin n'est pas touché.
        let mut pepin = Aabb::new(140.0, 146.0, 10.0, 14.0);
        for _ in 0..60 {
            mur.avancer(&pepin);
        }
        assert!((mur.front() - 80.0).abs() < 0.01, "{}", mur.front());
        assert!(!mur.touche(&pepin));
        // Pépin file loin devant : le mur est tiré à RETARD_MAX derrière lui.
        pepin.x = 400.0;
        mur.avancer(&pepin);
        assert_eq!(mur.front(), 400.0 - RETARD_MAX);
        // Pépin s'arrête : le mur le touche en 2 s, pas avant.
        let mut pas = 0;
        while !mur.touche(&pepin) {
            mur.avancer(&pepin);
            pas += 1;
        }
        assert!((120..=121).contains(&pas), "{pas} pas");
        // Mort : Pépin réapparaît à sa lanterne, le mur recule derrière lui.
        pepin.x = 300.0;
        mur.replacer(&pepin);
        assert_eq!(mur.front(), 300.0 - RECUL_REAPPARITION);
        assert!(!mur.touche(&pepin));
        // Au soleil : le mur s'arrête 14 tuiles avant le drapeau, puis recule et ne
        // tue plus.
        let arret = niveau.arrivee.unwrap().x - ARRET_AVANT_SOLEIL;
        pepin.x = arret + RETARD_MAX + 50.0;
        mur.avancer(&pepin);
        assert!(mur.vaincu());
        assert_eq!(mur.front(), arret);
        pepin.x = 0.0;
        mur.avancer(&pepin);
        assert!(mur.front() < arret && !mur.touche(&pepin));
    }
}
