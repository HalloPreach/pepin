//! Sauvegarde (cartes 7.2, 7.3 et 7.9) : ce qui reste d'une partie à l'autre.
//!
//! - le porte-monnaie : les gouttes d'or gagnées (niveaux RÉUSSIS, défis, médailles
//!   et bonus de fin, voir `defis`), la monnaie de la boutique ;
//! - les records : meilleur score et meilleur temps d'une partie finie (affichés,
//!   jamais dépensés) ;
//! - la PROGRESSION, rangée par CLÉ de niveau (carte 7.9 : `InfosNiveau::cle`,
//!   `toits`, `arc`…, jamais par la place dans la table) : pour chaque niveau, atteint
//!   ou non, les gouttes trouvées (lesquelles des 3), le meilleur temps, les défis
//!   déjà réussis et la meilleure médaille (`ProgresNiveau`). C'est aussi le registre
//!   de ce qui a DÉJÀ ÉTÉ CRÉDITÉ (chaque récompense ne l'est qu'une fois, voir
//!   `defis`) ; les bonus de fin déjà crédités sont dans `bonus_credites` ;
//! - l'INVENTAIRE des cosmétiques (carte 7.3) : des identifiants « categorie:id »
//!   (`skin:pepinette`, `musique:orage`, `trainee:etoiles`…) et l'objet ÉQUIPÉ par
//!   catégorie. Les champs de la v1 (`skins_debloques`, `skin_choisi`) restent : ce
//!   sont des façades de la catégorie `skin`, tenues à jour dans les deux sens
//!   (`normaliser`).
//!
//! Format texte, une ligne `cle=valeur` par champ, avec `version=3` en tête ; une
//! ligne par niveau touché : `niveau.toits=atteint:1 gouttes:101 defis:110 medaille:2
//! temps:2345` (plus, pour le boss, ` defaites:9 facile:1` s'ils ne sont pas nuls,
//! carte 8.14). Une v1 ou une v2 est relue sans perte, puis réécrite en v3 :
//! MIGRATION v2 → v3, la v2 rangeait les niveaux par NUMÉRO (`niveau.10=…`,
//! `niveau_max_atteint=9`) ; ces numéros se lisent dans `ORDRE_V2`, la table FIGÉE de
//! l'ordre publié en v2 (surtout pas la table vivante `NIVEAUX`, qui peut gagner un
//! niveau au milieu). La lecture est TOLÉRANTE : texte absent, vide ou de version
//! inconnue → valeurs par défaut ; champ inconnu → ignoré ; valeur illisible → valeur
//! par défaut de ce champ ; niveau de clé inconnue (version future) → gardé tel quel
//! et réécrit. Jamais de plantage.
//!
//! Stockage : dans le navigateur, `localStorage` (clé `pepin.sauvegarde`), par le
//! petit plugin JavaScript de `web/index.html` (fonctions `pepin_sauvegarde_lire` et
//! `pepin_sauvegarde_ecrire`). Si `localStorage` est refusé (navigation privée…), le
//! plugin ne lit rien et n'écrit rien : le jeu tourne sans sauvegarde. En natif
//! (tests), rien n'est écrit sur le disque : la sauvegarde vit en mémoire.
//!
//! Notions Rust illustrées ici : `str::split_once`, `parse::<u32>().ok()`,
//! `saturating_add`, `BTreeMap` / `BTreeSet` (ordre stable à l'écriture), API
//! `entry`, `#[cfg(target_arch = "wasm32")]` pour un code propre au web, et
//! `extern "C"` pour appeler une fonction JavaScript depuis le WebAssembly.

use std::collections::{BTreeMap, BTreeSet};

use pepin_core::levels::{indice_de_cle, NIVEAUX};

/// Version du format texte écrit (les versions 1, 2 et 3 sont lues).
const VERSION: u32 = 3;
/// Migration v2 → v3 : les clés des niveaux dans l'ordre PUBLIÉ en v2 (`niveau.1` =
/// `toits`… `niveau.11` = `descente`). Table FIGÉE : ne jamais la modifier, même
/// quand un niveau est ajouté ou déplacé dans `NIVEAUX`.
pub const ORDRE_V2: [&str; 11] = [
    "toits",
    "gouttieres",
    "marche",
    "orage",
    "clocher",
    "cyclone",
    "vents",
    "mer",
    "nuages",
    "arc",
    "descente",
];
/// Skins disponibles dès le départ.
pub const SKINS_DE_DEPART: [&str; 2] = ["pepin", "pepinette"];
/// Inventaire de départ (cartes 7.3 et 7.6) : les deux skins, la musique d'origine et
/// les choix « rien » de chaque catégorie (musique de la zone, pas de traînée,
/// parapluie uni, palette d'origine : voir `cosmetiques`).
pub const INVENTAIRE_DE_DEPART: [&str; 7] = [
    "skin:pepin",
    "skin:pepinette",
    "musique:orage",
    "musique:zone",
    "trainee:aucune",
    "motif:uni",
    "palette:origine",
];
/// Objets équipés au départ, par catégorie : le jeu tel qu'il était.
pub const EQUIPES_DE_DEPART: [(&str, &str); 5] = [
    ("skin", "pepin"),
    ("musique", "zone"),
    ("trainee", "aucune"),
    ("motif", "uni"),
    ("palette", "origine"),
];

/// Progression d'un niveau (cartes 7.3 et 7.9). Ce qui est noté ici a AUSSI été
/// crédité au porte-monnaie, une fois pour toutes (voir `defis::noter_niveau`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProgresNiveau {
    /// Pépin est arrivé à ce niveau (en partie ou depuis la carte des niveaux) : il
    /// est ouvert sur la carte des niveaux. Remplace le `niveau_max_atteint` de la v2.
    pub atteint: bool,
    /// Gouttes d'or trouvées (et créditées) dans un niveau RÉUSSI : `gouttes[i]`
    /// pour la goutte i.
    pub gouttes: [bool; 3],
    /// Meilleur temps d'une tentative réussie, en pas de 1/60 s (`None` : jamais fini).
    pub meilleur_temps: Option<u32>,
    /// Défis réussis (et crédités) au moins une fois (dans l'ordre de
    /// `defis::defis_du_niveau`).
    pub defis: [bool; 3],
    /// Meilleure médaille (créditée) : défis réussis dans UNE même tentative (0 à 3 ;
    /// 3 = or).
    pub medaille: u8,
    /// Carte 8.14 (boss) : morts d'affilée dans ce niveau, en mode normal, depuis la
    /// dernière victoire (voir `facile`).
    pub defaites: u32,
    /// Carte 8.14 : le boss a été vaincu en MODE FACILE (le niveau compte comme fini,
    /// sans médaille ni meilleur temps).
    pub facile: bool,
}

impl ProgresNiveau {
    /// Le niveau a-t-il été fini au moins une fois ? Oui s'il a un meilleur temps,
    /// mais aussi (carte 8.13) s'il a une médaille ou un défi réussi : ils ne se
    /// gagnent qu'en finissant le niveau, et la v2 ne notait pas toujours le temps
    /// (un niveau migré en OR sans temps restait « PAS ENCORE FINI »).
    pub fn fini(&self) -> bool {
        self.battu_en_normal() || self.facile
    }

    /// Fini en mode NORMAL (carte 8.14) : meilleur temps, médaille ou défi. Une
    /// victoire en facile (`facile`) ne les donne pas.
    pub fn battu_en_normal(&self) -> bool {
        self.meilleur_temps.is_some() || self.medaille > 0 || self.defis.contains(&true)
    }
}

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
    /// Façade v1 de la catégorie `skin` de l'inventaire.
    pub skins_debloques: Vec<String>,
    /// Façade v1 du skin équipé.
    pub skin_choisi: String,
    /// Progression niveau par niveau, par CLÉ de niveau (`InfosNiveau::cle`). Les
    /// clés inconnues de cette version (relues d'une version future) sont gardées.
    pub niveaux: BTreeMap<String, ProgresNiveau>,
    /// Bonus de fin de partie déjà crédités (identifiants de `defis::bonus_fin`).
    pub bonus_credites: BTreeSet<String>,
    /// Cosmétiques possédés, « categorie:id » (contient `INVENTAIRE_DE_DEPART`).
    pub inventaire: Vec<String>,
    /// Objet équipé par catégorie (catégorie → id).
    pub equipes: BTreeMap<String, String>,
    /// Course contre le robot (carte 8.7) : par CLÉ de niveau, le meilleur résultat
    /// contre chaque force (rang de `course::Force`), en écart SIGNÉ de pas : négatif =
    /// gagné de tant, positif = perdu de tant (`None` : jamais couru et fini).
    pub courses: BTreeMap<String, [Option<i32>; 3]>,
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
            niveaux: BTreeMap::new(),
            bonus_credites: BTreeSet::new(),
            inventaire: INVENTAIRE_DE_DEPART.iter().map(|s| s.to_string()).collect(),
            equipes: EQUIPES_DE_DEPART
                .iter()
                .map(|&(c, i)| (c.to_string(), i.to_string()))
                .collect(),
            courses: BTreeMap::new(),
        }
    }
}

// La boutique utilisera `depenser`, `acquerir`, `equiper`…
#[cfg_attr(not(test), allow(dead_code))]
impl Sauvegarde {
    /// Gouttes gagnées (niveau réussi, défi, médaille, bonus) : dans le porte-monnaie.
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

    /// Ajoute un skin aux skins débloqués (sans doublon) ; façade de
    /// `acquerir("skin:…")`.
    pub fn debloquer(&mut self, skin: &str) {
        if !self.skins_debloques.iter().any(|s| s == skin) {
            self.skins_debloques.push(skin.to_string());
        }
        let objet = format!("skin:{skin}");
        if !self.inventaire.contains(&objet) {
            self.inventaire.push(objet);
        }
    }

    /// Possède-t-on l'objet « categorie:id » ?
    pub fn possede(&self, objet: &str) -> bool {
        match objet.split_once(':') {
            Some(("skin", id)) => self.skins_debloques.iter().any(|s| s == id),
            _ => self.inventaire.iter().any(|o| o == objet),
        }
    }

    /// Ajoute l'objet « categorie:id » à l'inventaire (sans doublon). Un texte qui
    /// n'est pas un identifiant d'objet est ignoré.
    pub fn acquerir(&mut self, objet: &str) {
        let Some((categorie, id)) = decouper_objet(objet) else {
            return;
        };
        if categorie == "skin" {
            self.debloquer(id);
        } else if !self.inventaire.iter().any(|o| o == objet) {
            self.inventaire.push(objet.to_string());
        }
    }

    /// Équipe l'objet « categorie:id » s'il est possédé (`false` sinon).
    pub fn equiper(&mut self, objet: &str) -> bool {
        let Some((categorie, id)) = decouper_objet(objet) else {
            return false;
        };
        if !self.possede(objet) {
            return false;
        }
        if categorie == "skin" {
            self.skin_choisi = id.to_string();
        }
        self.equipes.insert(categorie.to_string(), id.to_string());
        true
    }

    /// Objet équipé dans une catégorie (`None` : rien d'équipé).
    pub fn equipe(&self, categorie: &str) -> Option<&str> {
        if categorie == "skin" {
            return Some(&self.skin_choisi);
        }
        self.equipes.get(categorie).map(String::as_str)
    }

    /// Progression du niveau de clé `cle` (vide s'il n'a jamais été touché). Carte
    /// 7.9 : toute la game lit la progression par ici, PAR CLÉ (carte des niveaux,
    /// défis, bilan).
    pub fn progres(&self, cle: &str) -> ProgresNiveau {
        self.niveaux.get(cle).copied().unwrap_or_default()
    }

    /// Progression modifiable du niveau de clé `cle`, créée vide au besoin. `None`
    /// si la clé n'est pas dans la table `NIVEAUX` (niveaux d'essai du build de QA) :
    /// rien n'est noté pour eux.
    pub fn progres_mut(&mut self, cle: &str) -> Option<&mut ProgresNiveau> {
        indice_de_cle(NIVEAUX, cle)?;
        Some(self.niveaux.entry(cle.to_string()).or_default())
    }

    /// Pépin est arrivé au niveau de clé `cle` (hors de la table : ignoré).
    pub fn noter_niveau_atteint(&mut self, cle: &str) {
        if let Some(p) = self.progres_mut(cle) {
            p.atteint = true;
        }
    }

    /// Le bonus de fin `id` est crédité : `true` la PREMIÈRE fois seulement (il est
    /// alors noté), `false` s'il l'avait déjà été.
    pub fn crediter_bonus(&mut self, id: &str) -> bool {
        self.bonus_credites.insert(id.to_string())
    }

    /// Meilleur résultat de course (écart signé, voir `courses`) sur le niveau de clé
    /// `cle` contre la force de rang `force`.
    pub fn meilleure_course(&self, cle: &str, force: usize) -> Option<i32> {
        self.courses
            .get(cle)
            .and_then(|r| r.get(force).copied().flatten())
    }

    /// Une course finie (carte 8.7) : garde le meilleur écart (le plus petit). Renvoie
    /// `true` si c'est un nouveau meilleur résultat. Hors de la table : ignoré.
    pub fn noter_course(&mut self, cle: &str, force: usize, ecart: i32) -> bool {
        if indice_de_cle(NIVEAUX, cle).is_none() || force >= 3 {
            return false;
        }
        let r = self.courses.entry(cle.to_string()).or_default();
        let mieux = r[force].is_none_or(|e| ecart < e);
        if mieux {
            r[force] = Some(ecart);
        }
        mieux
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

    /// Accorde les façades v1 et l'inventaire, dans les deux sens : skins débloqués
    /// ↔ `skin:…`, `skin_choisi` ↔ objet équipé de la catégorie `skin` ; objets de
    /// départ toujours présents ; un objet équipé qu'on ne possède pas est retiré
    /// (le skin, lui, revient à Pépin).
    pub fn normaliser(&mut self) {
        for depart in INVENTAIRE_DE_DEPART {
            self.acquerir(depart);
        }
        for skin in self.skins_debloques.clone() {
            self.debloquer(&skin);
        }
        for objet in self.inventaire.clone() {
            if let Some(("skin", id)) = objet.split_once(':') {
                self.debloquer(id);
            }
        }
        if !self.skins_debloques.contains(&self.skin_choisi) {
            self.skin_choisi = SKINS_DE_DEPART[0].to_string();
        }
        let possedes: Vec<(String, String)> = self
            .equipes
            .iter()
            .filter(|(c, i)| self.possede(&format!("{c}:{i}")))
            .map(|(c, i)| (c.clone(), i.clone()))
            .collect();
        self.equipes = possedes.into_iter().collect();
        for (categorie, id) in EQUIPES_DE_DEPART {
            self.equipes
                .entry(categorie.to_string())
                .or_insert_with(|| id.to_string());
        }
        self.equipes
            .insert("skin".to_string(), self.skin_choisi.clone());
    }

    /// Le texte enregistré (format v3).
    pub fn vers_texte(&self) -> String {
        // Les façades ont pu être changées directement : on écrit leur accord.
        let mut s = self.clone();
        s.normaliser();
        let mut t = format!(
            "version={VERSION}\ngouttes_en_poche={}\ngouttes_total_gagnees={}\n\
             meilleur_score={}\nparties_finies={}\n\
             inventaire={}\nequipe={}\nbonus_credites={}\n",
            s.gouttes_en_poche,
            s.gouttes_total_gagnees,
            s.meilleur_score,
            s.parties_finies,
            s.inventaire.join(","),
            s.equipes
                .iter()
                .map(|(c, i)| format!("{c}:{i}"))
                .collect::<Vec<_>>()
                .join(","),
            s.bonus_credites
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(","),
        );
        if let Some(temps) = s.meilleur_temps {
            t.push_str(&format!("meilleur_temps={temps}\n"));
        }
        // Une ligne par niveau déjà touché, par CLÉ : `niveau.toits=atteint:1
        // gouttes:101 defis:110 medaille:2 temps:2345` (bits dans l'ordre 0, 1, 2).
        for (cle, p) in &s.niveaux {
            if *p == ProgresNiveau::default() {
                continue;
            }
            t.push_str(&format!(
                "niveau.{cle}=atteint:{} gouttes:{} defis:{} medaille:{}",
                u8::from(p.atteint),
                bits(p.gouttes),
                bits(p.defis),
                p.medaille
            ));
            if let Some(temps) = p.meilleur_temps {
                t.push_str(&format!(" temps:{temps}"));
            }
            // Carte 8.14 : seulement s'ils servent (les autres lignes ne changent pas).
            if p.defaites > 0 {
                t.push_str(&format!(" defaites:{}", p.defaites));
            }
            if p.facile {
                t.push_str(" facile:1");
            }
            t.push('\n');
        }
        // Courses (carte 8.7) : `course.toits=0:-120 2:35` (rang de la force : écart).
        for (cle, r) in &s.courses {
            let morceaux: Vec<String> = r
                .iter()
                .enumerate()
                .filter_map(|(f, e)| e.map(|e| format!("{f}:{e}")))
                .collect();
            if !morceaux.is_empty() {
                t.push_str(&format!("course.{cle}={}\n", morceaux.join(" ")));
            }
        }
        t
    }

    /// Relit un texte de `vers_texte` (v3), de la v2 ou de la v1, avec tolérance
    /// (voir l'en-tête du module).
    pub fn depuis_texte(texte: &str) -> Self {
        let mut s = Self::default();
        let paires = texte
            .lines()
            .filter_map(|l| l.split_once('='))
            .map(|(c, v)| (c.trim(), v.trim()));
        let Some(version) = paires
            .clone()
            .filter(|&(c, _)| c == "version")
            .find_map(|(_, v)| v.parse::<u32>().ok().filter(|v| (1..=VERSION).contains(v)))
        else {
            return s;
        };
        // v1/v2 : niveaux rangés par NUMÉRO, lus dans la table figée `ORDRE_V2`.
        let ancienne = version < 3;
        let mut max_atteint_v2: Option<usize> = None;
        // Avant la carte 7.6, aucune musique ne se choisissait : la musique équipée
        // (ORAGE, celle du départ d'alors) n'était pas un choix du joueur. Une
        // sauvegarde de cette époque (sans « musique:zone ») revient à PAR ZONE.
        let mut choix_de_musique = false;
        let nombre = |v: &str, defaut: u32| v.parse::<u32>().unwrap_or(defaut);
        for (cle, valeur) in paires {
            match cle {
                "gouttes_en_poche" => s.gouttes_en_poche = nombre(valeur, 0),
                "gouttes_total_gagnees" => s.gouttes_total_gagnees = nombre(valeur, 0),
                "meilleur_score" => s.meilleur_score = nombre(valeur, 0),
                "meilleur_temps" => s.meilleur_temps = valeur.parse().ok(),
                "parties_finies" => s.parties_finies = nombre(valeur, 0),
                // v1 : migrés vers l'inventaire par `normaliser`.
                "skins_debloques" => {
                    for skin in valeur.split(',').map(str::trim) {
                        if est_identifiant(skin) {
                            s.debloquer(skin);
                        }
                    }
                }
                "skin_choisi" => s.skin_choisi = valeur.to_string(),
                // v2 seulement (indice dans `ORDRE_V2`) : appliqué après la boucle.
                "niveau_max_atteint" if ancienne => {
                    max_atteint_v2 = valeur.parse::<usize>().ok();
                }
                // v3.
                "bonus_credites" => {
                    for id in valeur.split(',').map(str::trim) {
                        if est_identifiant(id) {
                            s.bonus_credites.insert(id.to_string());
                        }
                    }
                }
                "inventaire" => {
                    for objet in valeur.split(',').map(str::trim) {
                        choix_de_musique |= objet == "musique:zone";
                        s.acquerir(objet);
                    }
                }
                "equipe" => {
                    for objet in valeur.split(',').map(str::trim) {
                        if let Some((categorie, id)) = decouper_objet(objet) {
                            s.equipes.insert(categorie.to_string(), id.to_string());
                            if categorie == "skin" {
                                s.skin_choisi = id.to_string();
                            }
                        }
                    }
                }
                _ => {
                    // Courses (carte 8.7) : un champ de plus en v3, ignoré par les
                    // versions d'avant (pas de changement de version).
                    if let Some(c) = cle.strip_prefix("course.").filter(|c| est_identifiant(c)) {
                        let r = lire_course(valeur);
                        if r.iter().any(Option::is_some) {
                            s.courses.insert(c.to_string(), r);
                        }
                        continue;
                    }
                    let cle_niveau = cle.strip_prefix("niveau.").and_then(|n| {
                        if ancienne {
                            // v2 : `niveau.10` → 10e clé de l'ordre publié en v2.
                            let numero = n.parse::<usize>().ok()?;
                            ORDRE_V2.get(numero.checked_sub(1)?).copied()
                        } else {
                            Some(n).filter(|n| est_identifiant(n))
                        }
                    });
                    if let Some(c) = cle_niveau {
                        s.niveaux.insert(c.to_string(), lire_niveau(valeur));
                    }
                    // Sinon, champ inconnu (version future) : ignoré.
                }
            }
        }
        if ancienne {
            s.migrer_v2(max_atteint_v2);
        }
        if !choix_de_musique {
            s.equipes.remove("musique");
        }
        s.normaliser();
        s
    }

    /// Fin de la migration v1/v2 → v3. Le « niveau le plus loin » (un indice de
    /// `ORDRE_V2`) devient l'état `atteint` de chaque niveau jusqu'à lui. Les gouttes,
    /// défis et médailles déjà notés restent tels quels : ils comptent comme DÉJÀ
    /// CRÉDITÉS (c'était le cas, le porte-monnaie est gardé). Un niveau v2 avec une
    /// médaille ou un défi réussi a été FINI (`ProgresNiveau::fini`, carte 8.13) : il
    /// est donc aussi atteint. Le bonus « FIN » a été
    /// crédité à chaque partie finie de la v2 : il l'est déjà si une partie a été
    /// finie. Les deux autres bonus (non mémorisés en v2) restent à gagner une fois.
    fn migrer_v2(&mut self, max_atteint: Option<usize>) {
        if let Some(max) = max_atteint {
            for cle in ORDRE_V2.iter().take(max.saturating_add(1)) {
                self.niveaux.entry(cle.to_string()).or_default().atteint = true;
            }
        }
        for p in self.niveaux.values_mut() {
            p.atteint |= p.fini();
        }
        if self.parties_finies > 0 {
            self.bonus_credites
                .insert(crate::defis::BONUS_FIN.to_string());
        }
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

/// `[vrai, faux, vrai]` → `"101"`.
fn bits(b: [bool; 3]) -> String {
    b.iter().map(|&x| if x { '1' } else { '0' }).collect()
}

/// `"101"` → `[vrai, faux, vrai]` ; autre chose → `None`.
fn lire_bits(t: &str) -> Option<[bool; 3]> {
    let c: Vec<char> = t.chars().collect();
    if c.len() != 3 || c.iter().any(|&x| x != '0' && x != '1') {
        return None;
    }
    Some([c[0] == '1', c[1] == '1', c[2] == '1'])
}

/// Relit `atteint:1 gouttes:101 defis:110 medaille:2 temps:2345` (morceau illisible :
/// défaut).
fn lire_niveau(valeur: &str) -> ProgresNiveau {
    let mut p = ProgresNiveau::default();
    for (cle, v) in valeur.split_whitespace().filter_map(|m| m.split_once(':')) {
        match cle {
            "atteint" => p.atteint = v == "1",
            "gouttes" => p.gouttes = lire_bits(v).unwrap_or_default(),
            "defis" => p.defis = lire_bits(v).unwrap_or_default(),
            "medaille" => p.medaille = v.parse::<u8>().map_or(0, |m| m.min(3)),
            "temps" => p.meilleur_temps = v.parse().ok(),
            "defaites" => p.defaites = v.parse().unwrap_or(0),
            "facile" => p.facile = v == "1",
            _ => {}
        }
    }
    p
}

/// Relit `0:-120 2:35` (morceau illisible : ignoré).
fn lire_course(valeur: &str) -> [Option<i32>; 3] {
    let mut r = [None; 3];
    for (f, e) in valeur.split_whitespace().filter_map(|m| m.split_once(':')) {
        if let (Ok(f), Ok(e)) = (f.parse::<usize>(), e.parse::<i32>()) {
            if let Some(case) = r.get_mut(f) {
                *case = Some(e);
            }
        }
    }
    r
}

/// Un identifiant (de skin, de catégorie…) : lettres minuscules, chiffres, `_` ou `-`.
fn est_identifiant(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

/// « categorie:id » → `(categorie, id)` si les deux sont des identifiants.
fn decouper_objet(objet: &str) -> Option<(&str, &str)> {
    objet
        .split_once(':')
        .filter(|(c, i)| est_identifiant(c) && est_identifiant(i))
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
    /// sans assez de gouttes ; une v1 relue sans perte (carte 7.3).
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
        // v3 : progression par clé, bonus crédités, inventaire générique, objet équipé.
        s.noter_niveau_atteint("clocher");
        s.noter_niveau_atteint("essai_vent");
        *s.progres_mut("marche").unwrap() = ProgresNiveau {
            atteint: false,
            gouttes: [true, false, true],
            meilleur_temps: Some(2345),
            defis: [true, true, false],
            medaille: 2,
            ..ProgresNiveau::default()
        };
        assert!(s.crediter_bonus("sans_game_over"));
        assert!(!s.crediter_bonus("sans_game_over"), "une seule fois");
        s.acquerir("trainee:etoiles");
        assert!(s.equiper("trainee:etoiles"));
        assert!(!s.equiper("motif:pois"), "pas acheté");
        s.normaliser();
        assert_eq!(Sauvegarde::depuis_texte(&s.vers_texte()), s);
        assert!(s.progres("clocher").atteint);
        assert_eq!(s.niveaux.len(), 2, "hors de la table : rien de noté");
        assert_eq!(s.equipe("skin"), Some("pepinette"));
        assert_eq!(s.equipe("musique"), Some("zone"));
        assert!(s.possede("skin:pepin_dore") && s.possede("musique:orage"));

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

        // Une v1 (carte 7.2) : rien ne se perd, les skins passent dans l'inventaire.
        let v1 = "version=1\ngouttes_en_poche=14\ngouttes_total_gagnees=20\nmeilleur_score=2500\n\
                  parties_finies=1\nskins_debloques=pepin,pepinette,pepin_dore\n\
                  skin_choisi=pepin_dore\nmeilleur_temps=18877\n";
        let lu = Sauvegarde::depuis_texte(v1);
        assert_eq!((lu.gouttes_en_poche, lu.gouttes_total_gagnees), (14, 20));
        assert_eq!((lu.meilleur_score, lu.meilleur_temps), (2500, Some(18877)));
        assert!(lu.possede("skin:pepin_dore"));
        assert_eq!(lu.equipe("skin"), Some("pepin_dore"));
        assert!(lu.vers_texte().starts_with("version=3\n"));
        assert_eq!(Sauvegarde::depuis_texte(&lu.vers_texte()), lu);
        // Ligne de niveau abîmée : défaut pour le morceau illisible seulement.
        let lu = Sauvegarde::depuis_texte("version=2\nniveau.1=gouttes:1x1 defis:111 medaille:9");
        assert_eq!(lu.progres("toits").gouttes, [false; 3]);
        assert_eq!(
            (lu.progres("toits").defis, lu.progres("toits").medaille),
            ([true; 3], 3)
        );
    }

    /// Migration v2 → v3 (carte 7.9) : une sauvegarde v2 réaliste (niveaux rangés
    /// par numéro) retrouve chaque niveau sous sa CLÉ (`niveau.10` → `arc`,
    /// `niveau.11` → `descente`, lus dans la table figée `ORDRE_V2`), garde son
    /// porte-monnaie, et reste juste quand un niveau s'insère AVANT eux dans la table.
    #[test]
    fn la_migration_v2_range_chaque_niveau_sous_sa_cle() {
        use crate::plan::{etat_etape, progres, EtatEtape};
        let v2 = "version=2\ngouttes_en_poche=57\ngouttes_total_gagnees=140\n\
                  meilleur_score=8100\nparties_finies=2\nmeilleur_temps=40000\n\
                  niveau_max_atteint=10\ninventaire=skin:pepin,skin:pepinette,musique:orage\n\
                  equipe=skin:pepinette,musique:orage\n\
                  niveau.1=gouttes:111 defis:111 medaille:3 temps:2400\n\
                  niveau.9=gouttes:101 defis:100 medaille:1 temps:5000\n\
                  niveau.10=gouttes:111 defis:011 medaille:2 temps:3300\n\
                  niveau.11=gouttes:110 defis:101 medaille:2 temps:1100\n";
        let lu = Sauvegarde::depuis_texte(v2);
        assert_eq!((lu.gouttes_en_poche, lu.gouttes_total_gagnees), (57, 140));
        let (arc, descente) = (lu.progres("arc"), lu.progres("descente"));
        assert_eq!((arc.meilleur_temps, arc.medaille), (Some(3300), 2));
        assert_eq!((arc.gouttes, arc.defis), ([true; 3], [false, true, true]));
        assert_eq!(
            (descente.meilleur_temps, descente.medaille),
            (Some(1100), 2)
        );
        assert_eq!(descente.gouttes, [true, true, false]);
        assert_eq!(lu.progres("nuages").meilleur_temps, Some(5000));
        assert_eq!(lu.progres("toits").medaille, 3);
        // `niveau_max_atteint=10` : les 11 niveaux de la v2 atteints ; « FIN » déjà
        // crédité (une partie finie en v2).
        assert!(ORDRE_V2.iter().all(|c| lu.progres(c).atteint));
        assert!(lu.bonus_credites.contains(crate::defis::BONUS_FIN));
        // Réécrite en v3, par clés, relue à l'identique.
        let v3 = lu.vers_texte();
        assert!(v3.starts_with("version=3\n") && !v3.contains("niveau_max_atteint"));
        assert!(v3.contains("niveau.arc=atteint:1 gouttes:111 defis:011 medaille:2 temps:3300"));
        assert_eq!(Sauvegarde::depuis_texte(&v3), lu);
        // Le niveau inséré AVANT le bonus par la carte 8.2 (`poursuite`, dans la vraie
        // table depuis la fusion) : il est vide et ouvert, l'arc-en-ciel et la descente
        // gardent leur progression.
        let i = indice_de_cle(NIVEAUX, "poursuite").unwrap();
        assert_eq!(indice_de_cle(NIVEAUX, "arc"), Some(i + 1));
        assert_eq!(progres(NIVEAUX, &lu, i), ProgresNiveau::default());
        assert_eq!(etat_etape(NIVEAUX, &lu, i), EtatEtape::Ouverte);
        assert_eq!(progres(NIVEAUX, &lu, i + 1), arc);
        assert_eq!(progres(NIVEAUX, &lu, i + 2), descente);
        assert_eq!(etat_etape(NIVEAUX, &lu, i + 2), EtatEtape::Finie);
    }

    /// Carte 8.13 : un niveau v2 avec une médaille ou un défi, mais SANS temps, est
    /// FINI (et donc atteint) ; le niveau inséré (`poursuite`) est jouable pour qui a
    /// fini une partie, ou sans partie finie s'il a déjà atteint plus loin.
    #[test]
    fn un_niveau_v2_medaille_sans_temps_est_fini() {
        use crate::plan::{etat_etape, EtatEtape};
        let v2 = "version=2\nparties_finies=1\nniveau_max_atteint=10\n\
                  niveau.11=gouttes:111 defis:111 medaille:3\nniveau.8=defis:010\n";
        let lu = Sauvegarde::depuis_texte(v2);
        let descente = indice_de_cle(NIVEAUX, "descente").unwrap();
        let poursuite = indice_de_cle(NIVEAUX, "poursuite").unwrap();
        assert!(lu.progres("descente").fini() && lu.progres("mer").fini());
        assert_eq!(etat_etape(NIVEAUX, &lu, descente), EtatEtape::Finie);
        assert_eq!(etat_etape(NIVEAUX, &lu, poursuite), EtatEtape::Ouverte);
        // Sans partie finie ni `niveau_max_atteint` : le défi de la mer suffit à la
        // dire atteinte, et la poursuite s'ouvre parce que la descente l'est.
        let sans = Sauvegarde::depuis_texte("version=2\nniveau.8=defis:010\nniveau.11=medaille:1");
        assert!(sans.progres("mer").atteint && sans.progres("descente").atteint);
        assert_eq!(etat_etape(NIVEAUX, &sans, poursuite), EtatEtape::Ouverte);
        assert_eq!(etat_etape(NIVEAUX, &sans, 1), EtatEtape::Ouverte);
        assert_eq!(
            etat_etape(NIVEAUX, &Sauvegarde::default(), 1),
            EtatEtape::Verrouillee
        );
    }
}
