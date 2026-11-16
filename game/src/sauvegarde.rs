//! Sauvegarde (carte 7.2) : ce qui reste d'une partie à l'autre.
//!
//! - le porte-monnaie : les gouttes d'or gagnées (niveaux RÉUSSIS seulement), la
//!   monnaie des skins (la boutique vient avec la carte 7.3) ;
//! - les records : meilleur score et meilleur temps d'une partie finie (affichés,
//!   jamais dépensés) ;
//! - les skins débloqués (« pepin » et « pepinette » d'office) et le skin choisi.
//!
//! Format texte, une ligne `cle=valeur` par champ, avec `version=1` en tête. La
//! lecture est TOLÉRANTE : texte absent, vide ou sans `version=1` → valeurs par
//! défaut ; champ inconnu → ignoré ; valeur illisible → valeur par défaut de ce
//! champ. Jamais de plantage.
//!
//! Stockage : dans le navigateur, `localStorage` (clé `pepin.sauvegarde`), par le
//! petit plugin JavaScript de `web/index.html` (fonctions `pepin_sauvegarde_lire` et
//! `pepin_sauvegarde_ecrire`). Si `localStorage` est refusé (navigation privée…), le
//! plugin ne lit rien et n'écrit rien : le jeu tourne sans sauvegarde. En natif
//! (tests), rien n'est écrit sur le disque : la sauvegarde vit en mémoire.
//!
//! Notions Rust illustrées ici : `str::split_once`, `parse::<u32>().ok()`,
//! `saturating_add`, `#[cfg(target_arch = "wasm32")]` pour un code propre au web, et
//! `extern "C"` pour appeler une fonction JavaScript depuis le WebAssembly.

/// Version du format texte.
const VERSION: u32 = 1;
/// Skins disponibles dès le départ.
pub const SKINS_DE_DEPART: [&str; 2] = ["pepin", "pepinette"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sauvegarde {
    /// Porte-monnaie : gouttes d'or à dépenser.
    pub gouttes_en_poche: u32,
    /// Toutes les gouttes jamais gagnées (dépensées ou non).
    pub gouttes_total_gagnees: u32,
    /// Meilleur score d'une partie finie (0 : aucune partie finie).
    pub meilleur_score: u32,
    /// Meilleur temps d'une partie finie, en pas de 1/60 s (`None` : aucune).
    pub meilleur_temps: Option<u32>,
    pub parties_finies: u32,
    /// Identifiants des skins débloqués (contient toujours `SKINS_DE_DEPART`).
    pub skins_debloques: Vec<String>,
    pub skin_choisi: String,
}

impl Default for Sauvegarde {
    fn default() -> Self {
        Self {
            gouttes_en_poche: 0,
            gouttes_total_gagnees: 0,
            meilleur_score: 0,
            meilleur_temps: None,
            parties_finies: 0,
            skins_debloques: SKINS_DE_DEPART.iter().map(|s| s.to_string()).collect(),
            skin_choisi: SKINS_DE_DEPART[0].to_string(),
        }
    }
}

// La boutique (carte 7.3) utilisera `depenser`, `debloquer` et `skin_choisi`.
#[cfg_attr(not(test), allow(dead_code))]
impl Sauvegarde {
    /// Gouttes d'un niveau réussi : dans le porte-monnaie.
    pub fn ajouter_gouttes(&mut self, n: u32) {
        self.gouttes_en_poche = self.gouttes_en_poche.saturating_add(n);
        self.gouttes_total_gagnees = self.gouttes_total_gagnees.saturating_add(n);
    }

    /// Paie `prix` gouttes. Refusé (`false`, rien ne change) s'il n'y en a pas assez.
    pub fn depenser(&mut self, prix: u32) -> bool {
        match self.gouttes_en_poche.checked_sub(prix) {
            Some(reste) => {
                self.gouttes_en_poche = reste;
                true
            }
            None => false,
        }
    }

    /// Ajoute un skin aux skins débloqués (sans doublon).
    pub fn debloquer(&mut self, skin: &str) {
        if !self.skins_debloques.iter().any(|s| s == skin) {
            self.skins_debloques.push(skin.to_string());
        }
    }

    /// Une partie finie : met à jour les records. Renvoie `true` si le score est un
    /// NOUVEAU RECORD (la première partie finie en est toujours un).
    pub fn noter_partie(&mut self, score: u32, temps: u32) -> bool {
        let record = self.parties_finies == 0 || score > self.meilleur_score;
        self.parties_finies = self.parties_finies.saturating_add(1);
        self.meilleur_score = self.meilleur_score.max(score);
        self.meilleur_temps = Some(self.meilleur_temps.map_or(temps, |t| t.min(temps)));
        record
    }

    /// Le texte enregistré.
    pub fn vers_texte(&self) -> String {
        let mut t = format!(
            "version={VERSION}\ngouttes_en_poche={}\ngouttes_total_gagnees={}\n\
             meilleur_score={}\nparties_finies={}\nskins_debloques={}\nskin_choisi={}\n",
            self.gouttes_en_poche,
            self.gouttes_total_gagnees,
            self.meilleur_score,
            self.parties_finies,
            self.skins_debloques.join(","),
            self.skin_choisi,
        );
        if let Some(temps) = self.meilleur_temps {
            t.push_str(&format!("meilleur_temps={temps}\n"));
        }
        t
    }

    /// Relit un texte de `vers_texte`, avec tolérance (voir l'en-tête du module).
    pub fn depuis_texte(texte: &str) -> Self {
        let mut s = Self::default();
        let paires = texte
            .lines()
            .filter_map(|l| l.split_once('='))
            .map(|(c, v)| (c.trim(), v.trim()));
        if !paires.clone().any(|p| p == ("version", "1")) {
            return s;
        }
        let nombre = |v: &str, defaut: u32| v.parse::<u32>().unwrap_or(defaut);
        for (cle, valeur) in paires {
            match cle {
                "gouttes_en_poche" => s.gouttes_en_poche = nombre(valeur, 0),
                "gouttes_total_gagnees" => s.gouttes_total_gagnees = nombre(valeur, 0),
                "meilleur_score" => s.meilleur_score = nombre(valeur, 0),
                "meilleur_temps" => s.meilleur_temps = valeur.parse().ok(),
                "parties_finies" => s.parties_finies = nombre(valeur, 0),
                "skins_debloques" => {
                    for skin in valeur.split(',').map(str::trim) {
                        if est_identifiant(skin) {
                            s.debloquer(skin);
                        }
                    }
                }
                "skin_choisi" => s.skin_choisi = valeur.to_string(),
                _ => {} // champ inconnu (version future) : ignoré
            }
        }
        // Un skin choisi qui n'est pas débloqué (texte abîmé) : retour au skin de base.
        if !s.skins_debloques.contains(&s.skin_choisi) {
            s.skin_choisi = SKINS_DE_DEPART[0].to_string();
        }
        s
    }

    /// Lit la sauvegarde du stockage (défaut s'il n'y a rien ou si elle est abîmée).
    pub fn charger() -> Self {
        stockage::lire().map_or_else(Self::default, |t| Self::depuis_texte(&t))
    }

    /// Écrit la sauvegarde dans le stockage (sans effet s'il est indisponible).
    pub fn enregistrer(&self) {
        stockage::ecrire(&self.vers_texte());
    }
}

/// Un identifiant de skin : lettres minuscules, chiffres, `_` ou `-`.
fn est_identifiant(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

/// Navigateur : `localStorage`, par le plugin JavaScript de `web/index.html`.
#[cfg(target_arch = "wasm32")]
mod stockage {
    extern "C" {
        /// Copie la sauvegarde (UTF-8) en `ptr` si elle tient dans `max` octets ;
        /// renvoie sa longueur en octets (0 : rien, ou `localStorage` refusé).
        fn pepin_sauvegarde_lire(ptr: *mut u8, max: u32) -> u32;
        /// Enregistre les `len` octets UTF-8 en `ptr` (rien si `localStorage` est refusé).
        fn pepin_sauvegarde_ecrire(ptr: *const u8, len: u32);
    }

    /// Version du plugin : le chargeur de miniquad la compare à celle du plugin JS
    /// (sinon il écrit dans la console que le plugin n'est pas utilisé).
    #[no_mangle]
    pub extern "C" fn pepin_sauvegarde_crate_version() -> u32 {
        1
    }

    pub fn lire() -> Option<String> {
        let mut tampon = vec![0u8; 1024];
        // SAFETY : le plugin écrit au plus `max` octets dans le tampon fourni.
        let mut n = unsafe { pepin_sauvegarde_lire(tampon.as_mut_ptr(), tampon.len() as u32) };
        if n as usize > tampon.len() {
            // Plus long que prévu : un second essai avec la bonne taille.
            tampon.resize(n as usize, 0);
            // SAFETY : idem.
            n = unsafe { pepin_sauvegarde_lire(tampon.as_mut_ptr(), tampon.len() as u32) };
        }
        let n = n as usize;
        if n == 0 || n > tampon.len() {
            return None;
        }
        tampon.truncate(n);
        String::from_utf8(tampon).ok()
    }

    pub fn ecrire(texte: &str) {
        // SAFETY : le plugin lit exactement `len` octets à partir de `ptr`.
        unsafe { pepin_sauvegarde_ecrire(texte.as_ptr(), texte.len() as u32) }
    }
}

/// Natif (tests) : pas de fichier, la sauvegarde vit en mémoire dans l'application.
#[cfg(not(target_arch = "wasm32"))]
mod stockage {
    pub fn lire() -> Option<String> {
        None
    }

    pub fn ecrire(_texte: &str) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Aller-retour texte ; texte absent, vide ou abîmé → défaut ; dépense refusée
    /// sans assez de gouttes.
    #[test]
    fn la_sauvegarde_fait_l_aller_retour_et_tolere_un_texte_abime() {
        let mut s = Sauvegarde::default();
        s.ajouter_gouttes(7);
        assert!(s.depenser(5));
        assert!(!s.depenser(3), "il n'en reste que 2");
        assert_eq!((s.gouttes_en_poche, s.gouttes_total_gagnees), (2, 7));
        s.debloquer("pepin_dore");
        s.skin_choisi = "pepinette".into();
        assert!(s.noter_partie(900, 5000), "première partie finie");
        assert!(!s.noter_partie(800, 4000));
        assert_eq!((s.meilleur_score, s.meilleur_temps), (900, Some(4000)));
        assert_eq!(Sauvegarde::depuis_texte(&s.vers_texte()), s);

        for abime in [
            "",
            "n'importe quoi",
            "gouttes_en_poche=50",
            "version=99\nparties_finies=3",
        ] {
            assert_eq!(
                Sauvegarde::depuis_texte(abime),
                Sauvegarde::default(),
                "{abime:?}"
            );
        }
        let t = "version=1\ngouttes_en_poche=abc\ninconnu=1\nskin_choisi=fantome\nparties_finies=4";
        let lu = Sauvegarde::depuis_texte(t);
        assert_eq!((lu.gouttes_en_poche, lu.parties_finies), (0, 4));
        assert_eq!(lu.skin_choisi, "pepin");
        assert_eq!(lu.skins_debloques, ["pepin", "pepinette"]);
    }
}
