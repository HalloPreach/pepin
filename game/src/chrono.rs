//! Le chronomètre de la partie : on compte des PAS de simulation (1/60 s) et on
//! ne les convertit en texte qu'à l'affichage.
//!
//! Compter des entiers évite les erreurs d'arrondi des nombres à virgule : au
//! bout de 4 minutes, 14 400 pas valent exactement « 4:00 ».

/// Nombre de pas par seconde (doit rester égal à la fréquence du `PasFixe` du cœur).
pub const PAS_PAR_SECONDE: u32 = 60;

/// Temps court pour le HUD : « 2:07 » (minutes, secondes sur deux chiffres).
pub fn temps_court(pas: u32) -> String {
    let secondes = pas / PAS_PAR_SECONDE;
    format!("{}:{:02}", secondes / 60, secondes % 60)
}

/// Temps précis pour l'écran de fin : « 2:07,4 » (virgule française, dixièmes).
pub fn temps_precis(pas: u32) -> String {
    let dixiemes = pas * 10 / PAS_PAR_SECONDE;
    let secondes = dixiemes / 10;
    format!("{}:{:02},{}", secondes / 60, secondes % 60, dixiemes % 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_pas_donne_zero() {
        assert_eq!(temps_court(0), "0:00");
        assert_eq!(temps_precis(0), "0:00,0");
    }

    #[test]
    fn les_secondes_passent_la_minute() {
        assert_eq!(temps_court(59 * 60), "0:59");
        assert_eq!(temps_court(60 * 60), "1:00");
        assert_eq!(temps_court(127 * 60 + 59), "2:07");
        assert_eq!(temps_court(4 * 60 * 60), "4:00");
    }

    #[test]
    fn les_dixiemes_sont_tronques() {
        // 127,4 s = 7 644 pas.
        assert_eq!(temps_precis(127 * 60 + 24), "2:07,4");
        assert_eq!(temps_precis(5), "0:00,0");
        assert_eq!(temps_precis(6), "0:00,1");
    }
}
