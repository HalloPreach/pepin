//! Module `vies` : les vies de Pépin (carte 6.2).
//!
//! Les règles, chacune réglable en une ligne :
//! - une partie commence avec `VIES_DEPART` vies ; une mort en retire une (Pépin
//!   réapparaît quand même à la dernière lanterne, comme avant) ;
//! - la troisième goutte d'or d'un niveau (toutes ses gouttes) donne une vie de
//!   plus, UNE fois par niveau, jusqu'à `VIES_MAX` ;
//! - à zéro, c'est « PLUS DE VIES » : le jeu fait recommencer le niveau en cours
//!   avec `VIES_DEPART` vies (`repartir`). Le niveau bonus ne coûte aucune vie
//!   (`coute_des_vies`) : c'est une récompense, pas un piège.
//!
//! Notions Rust illustrées ici : `struct` aux champs PRIVÉS dont les méthodes
//! gardent les règles vraies (on ne peut pas écrire 12 vies ni -1 vie de
//! l'extérieur : il n'y a pas de setter), `const fn` (utilisable dans une
//! constante), type `Copy` (une poignée d'octets, copiée plutôt que prêtée).

use crate::levels::GenreNiveau;

/// Vies au début d'une partie, et après un « PLUS DE VIES ».
pub const VIES_DEPART: u8 = 5;

/// Plafond des vies : un seul chiffre dans le HUD.
pub const VIES_MAX: u8 = 9;

/// Un niveau de ce genre coûte-t-il des vies ? Tous, sauf le niveau bonus (un sprint
/// au chrono sans lanterne, qu'on n'ouvre qu'avec toutes les gouttes : une
/// récompense). L'épilogue en coûte, comme un niveau normal.
pub const fn coute_des_vies(genre: GenreNiveau) -> bool {
    !matches!(genre, GenreNiveau::Bonus)
}

/// Les vies de la partie en cours.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vies {
    /// Vies restantes, de 0 à `VIES_MAX`. 0 : « PLUS DE VIES ».
    restantes: u8,
    /// La vie des trois gouttes du niveau en cours a-t-elle déjà été donnée ? Elle
    /// reste donnée après un « PLUS DE VIES » : le niveau recommence, les gouttes
    /// reviennent, mais la vie ne se regagne pas (une fois par niveau).
    donnee_ici: bool,
    /// Le niveau en cours coûte-t-il des vies (`coute_des_vies`) ?
    comptees: bool,
}

impl Default for Vies {
    fn default() -> Self {
        Self::new()
    }
}

impl Vies {
    /// Début de partie : `VIES_DEPART` vies, dans un niveau qui en coûte.
    pub const fn new() -> Self {
        Self {
            restantes: VIES_DEPART,
            donnee_ici: false,
            comptees: true,
        }
    }

    pub fn restantes(&self) -> u8 {
        self.restantes
    }

    /// Plus aucune vie : le jeu doit faire recommencer le niveau.
    pub fn plus_de_vies(&self) -> bool {
        self.restantes == 0
    }

    /// Le niveau en cours coûte-t-il des vies ?
    pub fn comptees(&self) -> bool {
        self.comptees
    }

    /// La vie des trois gouttes de ce niveau a-t-elle déjà été donnée ?
    pub fn donnee_ici(&self) -> bool {
        self.donnee_ici
    }

    /// Une mort. Retire une vie, sauf dans un niveau qui n'en coûte pas, ou s'il n'y
    /// en a déjà plus. Renvoie `true` si c'était la DERNIÈRE (« PLUS DE VIES »).
    pub fn perdre(&mut self) -> bool {
        if !self.comptees || self.restantes == 0 {
            return false;
        }
        self.restantes -= 1;
        self.restantes == 0
    }

    /// Toutes les gouttes du niveau viennent d'être ramassées : une vie de plus, si
    /// elle n'a pas déjà été donnée dans ce niveau, sous le plafond, et pas après la
    /// dernière vie perdue (« PLUS DE VIES » est sans appel, jusqu'à `repartir`).
    /// Renvoie `true` si une vie a vraiment été gagnée (pour le petit signe du HUD).
    pub fn trois_gouttes(&mut self) -> bool {
        if self.donnee_ici || self.plus_de_vies() {
            return false;
        }
        // Donnée même au plafond : la ramasser de nouveau après un « PLUS DE VIES »
        // ne doit rien changer non plus.
        self.donnee_ici = true;
        if self.restantes >= VIES_MAX {
            return false;
        }
        self.restantes += 1;
        true
    }

    /// Entrée dans un nouveau niveau de genre `genre` : sa vie des trois gouttes est à
    /// prendre. Les vies restantes ne changent pas.
    pub fn nouveau_niveau(&mut self, genre: GenreNiveau) {
        self.donnee_ici = false;
        self.comptees = coute_des_vies(genre);
    }

    /// Après « PLUS DE VIES » : `VIES_DEPART` vies, pour recommencer le niveau. La vie
    /// des trois gouttes, si elle a été donnée, le reste.
    pub fn repartir(&mut self) {
        self.restantes = VIES_DEPART;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Perd `n` vies (sans regarder le résultat).
    fn perdre(v: &mut Vies, n: u8) {
        for _ in 0..n {
            v.perdre();
        }
    }

    #[test]
    fn une_partie_commence_avec_cinq_vies() {
        let v = Vies::new();
        assert_eq!(v.restantes(), VIES_DEPART);
        assert_eq!(VIES_DEPART, 5);
        assert!(!v.plus_de_vies());
        assert!(v.comptees());
        assert_eq!(Vies::default(), v);
    }

    #[test]
    fn chaque_mort_retire_une_vie_et_la_cinquieme_est_la_derniere() {
        let mut v = Vies::new();
        for attendu in (1..VIES_DEPART).rev() {
            assert!(!v.perdre(), "il en reste {attendu}");
            assert_eq!(v.restantes(), attendu);
        }
        assert!(v.perdre(), "la cinquième mort est la dernière");
        assert_eq!(v.restantes(), 0);
        assert!(v.plus_de_vies());
        // Plus rien à perdre : ni négatif, ni « dernière » une seconde fois.
        assert!(!v.perdre());
        assert_eq!(v.restantes(), 0);
    }

    #[test]
    fn la_troisieme_goutte_donne_une_vie_une_seule_fois_par_niveau() {
        let mut v = Vies::new();
        perdre(&mut v, 2);
        assert!(v.trois_gouttes());
        assert_eq!(v.restantes(), 4);
        assert!(v.donnee_ici());
        assert!(!v.trois_gouttes(), "pas deux fois dans le même niveau");
        assert_eq!(v.restantes(), 4);
        // Niveau suivant : de nouveau à prendre.
        v.nouveau_niveau(GenreNiveau::Normal);
        assert!(!v.donnee_ici());
        assert!(v.trois_gouttes());
        assert_eq!(v.restantes(), 5);
    }

    #[test]
    fn les_vies_plafonnent_a_neuf() {
        let mut v = Vies::new();
        let mut gagnees = 0;
        for _ in 0..10 {
            v.nouveau_niveau(GenreNiveau::Normal);
            if v.trois_gouttes() {
                gagnees += 1;
            }
        }
        assert_eq!(VIES_MAX, 9);
        assert_eq!(v.restantes(), VIES_MAX);
        assert_eq!(gagnees, VIES_MAX - VIES_DEPART, "rien au-delà du plafond");
        // Au plafond, la vie du niveau est « donnée » quand même (sans effet).
        assert!(v.donnee_ici());
    }

    #[test]
    fn apres_plus_de_vies_on_repart_avec_cinq_et_la_vie_du_niveau_reste_donnee() {
        let mut v = Vies::new();
        assert!(v.trois_gouttes());
        perdre(&mut v, VIES_DEPART + 1);
        assert!(v.plus_de_vies());
        // À zéro, les gouttes ne ressuscitent pas (c'est déjà perdu).
        let mut sans_vie = Vies::new();
        perdre(&mut sans_vie, VIES_DEPART);
        assert!(!sans_vie.trois_gouttes());
        assert!(sans_vie.plus_de_vies());
        // Le niveau recommence : cinq vies, et les trois gouttes reprises ne
        // redonnent rien.
        v.repartir();
        assert_eq!(v.restantes(), VIES_DEPART);
        assert!(!v.trois_gouttes());
        assert_eq!(v.restantes(), VIES_DEPART);
        // Celle qui n'avait pas été donnée avant le « PLUS DE VIES » l'est après.
        sans_vie.repartir();
        assert!(sans_vie.trois_gouttes());
        assert_eq!(sans_vie.restantes(), VIES_DEPART + 1);
    }

    #[test]
    fn le_niveau_bonus_ne_coute_aucune_vie() {
        assert!(coute_des_vies(GenreNiveau::Normal));
        assert!(coute_des_vies(GenreNiveau::Epilogue));
        assert!(!coute_des_vies(GenreNiveau::Bonus));
        let mut v = Vies::new();
        v.nouveau_niveau(GenreNiveau::Bonus);
        assert!(!v.comptees());
        for _ in 0..50 {
            assert!(!v.perdre());
        }
        assert_eq!(
            v.restantes(),
            VIES_DEPART,
            "cinquante chutes, aucune vie perdue"
        );
        // Ses trois gouttes donnent quand même une vie, gardée pour l'épilogue.
        assert!(v.trois_gouttes());
        v.nouveau_niveau(GenreNiveau::Epilogue);
        assert!(v.comptees());
        assert_eq!(v.restantes(), VIES_DEPART + 1);
        assert!(!v.perdre());
        assert_eq!(v.restantes(), VIES_DEPART);
    }
}
