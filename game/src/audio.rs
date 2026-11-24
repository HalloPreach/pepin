//! Audio : joue les sons du jeu via l'API audio de macroquad.
//!
//! Les sons sont de VRAIS fichiers audio (carte 10.4e) : `assets/musiques/<id>.wav`
//! (une boucle par musique du catalogue `musiques`) et `assets/sons/<clé>.wav` (un
//! bruitage par `Effet`, plus la boucle `pluie`). WAV PCM 16 bits mono : lu partout,
//! Safari sur iPhone compris, et sans remplissage, donc des boucles exactes. Format
//! et règles : `assets/musiques/LISEZMOI.md`.
//!
//! Ils ne sont PAS dans le wasm (3 Mo) : `./build.sh` les copie à côté du jeu
//! (`dist/musiques`, `dist/sons`), `build.rs` en écrit la LISTE (`FICHIERS`), et
//! `Chargement` les télécharge EN ARRIÈRE-PLAN, après la première image (l'écran
//! titre n'attend rien), trois à la fois, ceux du titre d'abord. Tant qu'un son n'est
//! pas arrivé, il est muet ; une boucle qui arrive pendant que les boucles tournent
//! démarre aussitôt.
//!
//! Le son est DÉSACTIVÉ par défaut (et un navigateur n'autorise de toute façon
//! l'audio qu'après une première interaction : clic, touche ou toucher). Tant
//! que `regler(false)` est en vigueur, rien ne joue et les boucles sont arrêtées.
//!
//! La partie « que doit-on entendre ? » (`effets_du_pas`, `Detecteur`, `EtatSon`,
//! l'ordre de chargement) est pure et testée ; seuls `Audio` et `Chargement` parlent
//! à macroquad.
//!
//! Notions Rust illustrées ici : les coroutines de macroquad (une tâche `async`
//! avancée à chaque image, dont on relève le résultat quand elle a fini), `enum` à
//! données (`Piste`) comme clé de `HashMap`, détection de « front montant » par
//! comparaison avec la valeur de l'image précédente.

use std::collections::{HashMap, VecDeque};

use macroquad::audio::{
    load_sound, play_sound, play_sound_once, set_sound_volume, stop_sound, PlaySoundParams, Sound,
};
use macroquad::experimental::coroutines::{start_coroutine, Coroutine};
use pepin_core::entities::contacts::Contact;
use pepin_core::entities::Entite;
use pepin_core::player::Evenements;
use pepin_core::sons::Effet;
use pepin_core::LARGEUR_LOGIQUE;

use crate::monde::Monde;
use crate::musiques::Musique;

// `FICHIERS` : (genre, clé) de chaque `.wav` de `assets/musiques` et `assets/sons`,
// triés par chemin (écrit par `build.rs`).
include!(concat!(env!("OUT_DIR"), "/sons_disponibles.rs"));

/// Où le jeu va chercher les sons : sur le web, à côté de la page (`dist/musiques`,
/// `dist/sons`) ; hors du web (`cargo run` depuis la racine du dépôt), dans `assets`.
#[cfg(target_arch = "wasm32")]
const DOSSIER: &str = "";
#[cfg(not(target_arch = "wasm32"))]
const DOSSIER: &str = "assets/";

/// Volume de la boucle musicale et de la pluie douce (discrets).
const VOLUME_MUSIQUE: f32 = 0.30;
const VOLUME_PLUIE: f32 = 0.22;

/// Nombre de fichiers téléchargés EN MÊME TEMPS.
const EN_PARALLELE: usize = 3;

/// Quels effets déclencher après un pas de simulation ? (fonction pure)
pub fn effets_du_pas(evenements: &Evenements, contacts: &[Contact]) -> Vec<Effet> {
    let mut v = Vec::new();
    if evenements.saut {
        v.push(Effet::Saut);
    }
    if evenements.debut_plane {
        v.push(Effet::Plane);
    }
    if evenements.atterrissage {
        v.push(Effet::Atterrissage);
    }
    for c in contacts {
        match c {
            Contact::Goutte(_) => v.push(Effet::Goutte),
            // La lettre cachée (carte 8.6) : la petite fanfare des secrets.
            Contact::Lettre => v.push(Effet::Secret),
            Contact::Lanterne(..) => v.push(Effet::Lanterne),
            // Un coup sur la tête du Grand Nuage sonne comme un ennemi écrasé.
            Contact::Ecrase | Contact::Nuage | Contact::Pieuvre => v.push(Effet::Ecrase),
            // L'auvent fait le même « boing » que le champignon.
            Contact::Rebond | Contact::Auvent => v.push(Effet::Rebond),
            // (Le tonnerre d'un éclair est déjà déclenché par `Detecteur`.)
            Contact::Mort(_) => v.push(Effet::Mort),
        }
    }
    v
}

/// Détecte les changements d'une image à l'autre (front montant) : un éclair qui
/// vient de frapper dans la zone visible, le drapeau qui vient d'être atteint.
#[derive(Debug, Default)]
pub struct Detecteur {
    eclair_visible: bool,
    termine: bool,
}

impl Detecteur {
    /// `eclair_visible` : un éclair frappe-t-il en ce moment dans l'écran ?
    /// `termine` : le niveau est-il fini ? Renvoie les effets « de transition ».
    pub fn comparer(&mut self, eclair_visible: bool, termine: bool) -> Vec<Effet> {
        let mut v = Vec::new();
        if eclair_visible && !self.eclair_visible {
            v.push(Effet::Eclair);
        }
        if termine && !self.termine {
            v.push(Effet::Victoire);
        }
        self.eclair_visible = eclair_visible;
        self.termine = termine;
        v
    }
}

/// Un éclair frappe-t-il dans la zone visible (bord gauche `cam_x`) ? Ceux du Grand
/// Nuage comptent aussi : même tonnerre.
fn eclair_en_vue(monde: &Monde, cam_x: f32) -> bool {
    let en_vue =
        |z: &pepin_core::aabb::Aabb| z.droite() >= cam_x && z.x <= cam_x + LARGEUR_LOGIQUE as f32;
    monde.entites.iter().any(|e| match e {
        Entite::Eclair(ec) => ec.frappe() && en_vue(&ec.zone),
        Entite::Nuage(g) => g.colonnes.iter().enumerate().any(|(i, z)| {
            g.etat_colonne(i) == pepin_core::entities::nuage::EtatColonne::Frappe && en_vue(z)
        }),
        _ => false,
    })
}

/// Ce qu'il faut faire aux sons quand le réglage change (voir `EtatSon::regler`).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Consignes {
    pub demarrer_boucles: bool,
    pub arreter_boucles: bool,
    pub arreter_effets: bool,
}

/// État du son, sans macroquad (donc testable) : le son est-il activé, et la
/// musique et la pluie (les « boucles ») tournent-elles ?
#[derive(Debug, Default)]
pub struct EtatSon {
    actif: bool,
    boucles: bool,
}

impl EtatSon {
    /// `actif` : le joueur veut du son. `boucles_voulues` : faux pendant la pause.
    /// Les boucles ne tournent que si les deux sont vrais ; renvoie les changements.
    pub fn regler(&mut self, actif: bool, boucles_voulues: bool) -> Consignes {
        let boucles = actif && boucles_voulues;
        let consignes = Consignes {
            demarrer_boucles: boucles && !self.boucles,
            arreter_boucles: !boucles && self.boucles,
            arreter_effets: !actif && self.actif,
        };
        self.actif = actif;
        self.boucles = boucles;
        consignes
    }
}

/// Un son du jeu, tel qu'on l'attend : un effet, une musique ou la pluie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Piste {
    Effet(Effet),
    Musique(Musique),
    Pluie,
}

impl Piste {
    /// (genre, clé) de son fichier : `assets/<genre>/<clé>.wav`.
    pub fn fichier(self) -> (&'static str, &'static str) {
        match self {
            Piste::Effet(e) => ("sons", e.cle()),
            Piste::Musique(m) => ("musiques", m.id()),
            Piste::Pluie => ("sons", "pluie"),
        }
    }

    /// Toutes les pistes, dans l'ORDRE DE CHARGEMENT : ce qu'on entend au titre
    /// d'abord (le bip des menus, la musique Orage, la pluie), puis les bruitages
    /// (petits), puis les autres musiques (les plus lourdes).
    pub fn dans_l_ordre() -> Vec<Piste> {
        let mut v = vec![
            Piste::Effet(Effet::Menu),
            Piste::Musique(Musique::Orage),
            Piste::Pluie,
        ];
        for e in Effet::TOUS {
            if e != Effet::Menu {
                v.push(Piste::Effet(e));
            }
        }
        for m in Musique::TOUTES {
            if m != Musique::Orage {
                v.push(Piste::Musique(m));
            }
        }
        v
    }
}

/// Le téléchargement en arrière-plan des sons : une file d'attente, et au plus
/// `EN_PARALLELE` coroutines de macroquad en cours (chacune télécharge un fichier
/// puis attend que le navigateur l'ait décodé).
struct Chargement {
    attente: VecDeque<Piste>,
    en_cours: Vec<(Piste, Coroutine<Result<Sound, String>>)>,
}

impl Chargement {
    fn new() -> Self {
        // Un son sans fichier (impossible : test `chaque_son_demande_a_son_fichier`)
        // resterait muet.
        Self {
            attente: Piste::dans_l_ordre()
                .into_iter()
                .filter(|p| FICHIERS.contains(&p.fichier()))
                .collect(),
            en_cours: Vec::new(),
        }
    }

    /// À chaque image : relève les sons arrivés et lance les téléchargements
    /// suivants. Un fichier absent ou illisible est signalé une fois et reste muet :
    /// le jeu ne s'arrête jamais pour un son.
    fn avancer(&mut self) -> Vec<(Piste, Sound)> {
        let mut arrives = Vec::new();
        self.en_cours.retain(|(piste, tache)| {
            if !tache.is_done() {
                return true;
            }
            match tache.retrieve() {
                Some(Ok(son)) => arrives.push((*piste, son)),
                Some(Err(e)) => {
                    macroquad::logging::error!("son {:?} : {}", piste, e);
                }
                None => {
                    macroquad::logging::error!("son {:?} : chargement arrêté", piste);
                }
            }
            false
        });
        while self.en_cours.len() < EN_PARALLELE {
            let Some(piste) = self.attente.pop_front() else {
                break;
            };
            let (genre, cle) = piste.fichier();
            let chemin = format!("{DOSSIER}{genre}/{cle}.wav");
            let tache = start_coroutine(async move {
                load_sound(&chemin)
                    .await
                    .map_err(|e| format!("{chemin} : {e:?}"))
            });
            self.en_cours.push((piste, tache));
        }
        arrives
    }

    /// Tout est-il arrivé (ou abandonné) ?
    fn fini(&self) -> bool {
        self.attente.is_empty() && self.en_cours.is_empty()
    }
}

/// Les sons du jeu : ceux déjà arrivés, prêts à jouer, et le chargement des autres.
pub struct Audio {
    effets: HashMap<Effet, Sound>,
    /// Une boucle par musique du catalogue (`musiques.rs`), quand elle est arrivée.
    musiques: HashMap<Musique, Sound>,
    /// La musique jouée (défaut : Orage, celle du titre). La boucle principale y met
    /// à chaque image celle de la zone (`musiques::musique_de_l_ecran`) ou un forçage.
    musique_active: Musique,
    pluie: Option<Sound>,
    etat: EtatSon,
    detecteur: Detecteur,
    /// `None` avant la première image (rien ne part avant l'écran titre).
    chargement: Option<Chargement>,
}

impl Audio {
    /// Aucun son encore : `avancer`, appelé à chaque image, les fait venir.
    pub fn new() -> Self {
        Self {
            effets: HashMap::new(),
            musiques: HashMap::new(),
            musique_active: Musique::Orage,
            pluie: None,
            etat: EtatSon::default(),
            detecteur: Detecteur::default(),
            chargement: None,
        }
    }

    /// À appeler une fois par image, APRÈS la première : la première fois, lance le
    /// chargement des sons ; ensuite, range ceux qui arrivent. Une boucle (musique
    /// active ou pluie) qui arrive pendant que les boucles tournent démarre aussitôt.
    pub fn avancer(&mut self) {
        let chargement = self.chargement.get_or_insert_with(Chargement::new);
        if chargement.fini() {
            return;
        }
        for (piste, son) in chargement.avancer() {
            match piste {
                Piste::Effet(e) => {
                    self.effets.insert(e, son);
                }
                Piste::Musique(m) => {
                    if self.etat.boucles && m == self.musique_active {
                        jouer_boucle(&son, VOLUME_MUSIQUE);
                    }
                    self.musiques.insert(m, son);
                }
                Piste::Pluie => {
                    if self.etat.boucles {
                        jouer_boucle(&son, VOLUME_PLUIE);
                    }
                    self.pluie = Some(son);
                }
            }
        }
    }

    /// Le son est-il activé ?
    pub fn est_actif(&self) -> bool {
        self.etat.actif
    }

    /// La musique choisie.
    pub fn musique_active(&self) -> Musique {
        self.musique_active
    }

    /// Change de musique. Si les boucles tournent, l'ancienne s'arrête et la
    /// nouvelle part de son début (dès son arrivée si elle n'est pas encore
    /// chargée) ; sinon elle attendra le prochain démarrage.
    pub fn choisir_musique(&mut self, m: Musique) {
        if m == self.musique_active {
            return;
        }
        if self.etat.boucles {
            if let Some(s) = self.musiques.get(&self.musique_active) {
                stop_sound(s);
            }
            if let Some(s) = self.musiques.get(&m) {
                jouer_boucle(s, VOLUME_MUSIQUE);
            }
        }
        self.musique_active = m;
    }

    /// Active ou coupe le son, et dit si la musique et la pluie doivent tourner
    /// (`boucles` : faux pendant la pause). Appelé à chaque image ; n'agit que sur
    /// un changement. Une boucle arrêtée puis relancée repart de son début. Un son
    /// pas encore arrivé est simplement sauté (`avancer` le lancera à son arrivée).
    pub fn regler(&mut self, actif: bool, boucles: bool) {
        let c = self.etat.regler(actif, boucles);
        let musique = self.musiques.get(&self.musique_active);
        if c.demarrer_boucles {
            if let Some(s) = musique {
                jouer_boucle(s, VOLUME_MUSIQUE);
            }
            if let Some(s) = &self.pluie {
                jouer_boucle(s, VOLUME_PLUIE);
            }
        }
        if c.arreter_boucles {
            for s in [musique, self.pluie.as_ref()].into_iter().flatten() {
                stop_sound(s);
            }
        }
        if c.arreter_effets {
            for s in self.effets.values() {
                stop_sound(s);
            }
        }
    }

    /// Joue un effet (ignoré si le son est coupé).
    pub fn jouer(&self, effet: Effet) {
        if !self.etat.actif {
            return;
        }
        if let Some(s) = self.effets.get(&effet) {
            set_sound_volume(s, effet.volume());
            play_sound_once(s);
        }
    }

    /// À appeler après chaque pas de simulation du monde.
    /// `cam_x` : bord gauche de la zone visible (pour n'entendre que les éclairs vus).
    pub fn apres_pas(&mut self, monde: &Monde, cam_x: f32) {
        let mut a_jouer = effets_du_pas(&monde.evenements, &monde.contacts);
        if monde.secret_revele {
            a_jouer.push(Effet::Secret);
        }
        let vue = eclair_en_vue(monde, cam_x);
        a_jouer.extend(self.detecteur.comparer(vue, monde.termine));
        for e in a_jouer {
            self.jouer(e);
        }
    }
}

/// Lance un son en boucle.
fn jouer_boucle(son: &Sound, volume: f32) {
    play_sound(
        son,
        PlaySoundParams {
            looped: true,
            volume,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use pepin_core::entities::contacts::CauseMort;

    /// Chaque son que le jeu demande a son fichier (chaque effet, chaque musique, la
    /// pluie), une seule fois dans l'ordre de chargement, et aucun fichier n'est
    /// orphelin : tout `.wav` de `assets/musiques` et `assets/sons` est demandé.
    /// Ajouter un son = ajouter un fichier ET sa clé.
    #[test]
    fn chaque_son_demande_a_son_fichier() {
        let pistes = Piste::dans_l_ordre();
        assert_eq!(pistes.len(), Effet::TOUS.len() + Musique::TOUTES.len() + 1);
        let demandes: Vec<(&str, &str)> = pistes.iter().map(|p| p.fichier()).collect();
        for (i, d) in demandes.iter().enumerate() {
            assert!(!demandes[i + 1..].contains(d), "demandé deux fois : {d:?}");
            assert!(
                FICHIERS.contains(d),
                "fichier manquant : assets/{}/{}.wav",
                d.0,
                d.1
            );
        }
        for f in FICHIERS {
            assert!(
                demandes.contains(f),
                "fichier orphelin : assets/{}/{}.wav",
                f.0,
                f.1
            );
        }
        assert_eq!(FICHIERS.len(), demandes.len());
    }

    /// Ce qu'on entend au titre arrive d'abord : le bip des menus, la musique du
    /// titre, la pluie.
    #[test]
    fn le_titre_se_charge_en_premier() {
        assert_eq!(
            Piste::dans_l_ordre()[..3],
            [
                Piste::Effet(Effet::Menu),
                Piste::Musique(Musique::Orage),
                Piste::Pluie
            ]
        );
    }

    /// Chaque fichier est un WAV PCM 16 bits mono à 22 050 Hz, en-tête simple de 44
    /// octets (RIFF, fmt, data et RIEN d'autre : aucune métadonnée), sans octet de
    /// trop ; une musique dure au moins 5 s (9,6 à 11,5 s aujourd'hui), un bruitage
    /// au plus 2 s.
    #[test]
    fn chaque_son_est_un_wav_mono_16_bits_sans_metadonnees() {
        let racine = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets");
        let mut total = 0;
        for (genre, cle) in FICHIERS {
            let nom = format!("{genre}/{cle}.wav");
            let o = std::fs::read(racine.join(&nom)).unwrap();
            let u16_a = |i: usize| u16::from_le_bytes([o[i], o[i + 1]]);
            let u32_a = |i: usize| u32::from_le_bytes([o[i], o[i + 1], o[i + 2], o[i + 3]]);
            assert_eq!(&o[..4], b"RIFF", "{nom}");
            assert_eq!(&o[8..16], b"WAVEfmt ", "{nom}");
            assert_eq!(u32_a(16), 16, "{nom} : bloc fmt simple");
            assert_eq!(
                (u16_a(20), u16_a(22), u32_a(24), u16_a(34)),
                (1, 1, 22_050, 16),
                "{nom} : PCM, mono, 22 050 Hz, 16 bits"
            );
            assert_eq!(&o[36..40], b"data", "{nom} : aucun autre bloc");
            let donnees = u32_a(40) as usize;
            assert_eq!(
                o.len(),
                44 + donnees,
                "{nom} : aucun octet après les données"
            );
            assert_eq!(u32_a(4) as usize, o.len() - 8, "{nom}");
            let secondes = donnees as f32 / 2.0 / 22_050.0;
            if *genre == "musiques" {
                assert!(secondes >= 5.0, "{nom} : {secondes} s");
            } else {
                assert!(secondes <= 2.0, "{nom} : {secondes} s");
            }
            total += o.len();
        }
        assert!(total < 3_200_000, "{total} octets de sons");
    }

    #[test]
    fn rien_ne_se_passe_rien_ne_sonne() {
        assert!(effets_du_pas(&Evenements::default(), &[]).is_empty());
    }

    #[test]
    fn les_evenements_et_contacts_donnent_les_bons_effets() {
        let ev = Evenements {
            saut: true,
            atterrissage: false,
            debut_plane: true,
        };
        let contacts = [
            Contact::Goutte(1),
            Contact::Rebond,
            Contact::Ecrase,
            Contact::Lanterne(10.0, 20.0),
            Contact::Mort(CauseMort::Vide),
        ];
        assert_eq!(
            effets_du_pas(&ev, &contacts),
            vec![
                Effet::Saut,
                Effet::Plane,
                Effet::Goutte,
                Effet::Rebond,
                Effet::Ecrase,
                Effet::Lanterne,
                Effet::Mort,
            ]
        );
    }

    #[test]
    fn le_detecteur_ne_reagit_qu_au_front_montant() {
        let mut d = Detecteur::default();
        assert!(d.comparer(false, false).is_empty());
        assert_eq!(d.comparer(true, false), vec![Effet::Eclair]);
        assert!(d.comparer(true, false).is_empty(), "éclair déjà en cours");
        assert!(d.comparer(false, false).is_empty());
        assert_eq!(d.comparer(true, true), vec![Effet::Eclair, Effet::Victoire]);
        assert!(d.comparer(true, true).is_empty(), "victoire déjà annoncée");
    }

    #[test]
    fn la_pause_coupe_les_boucles_et_la_sortie_les_relance() {
        let mut s = EtatSon::default();
        // Son coupé au départ : rien ne tourne, pause ou pas.
        assert_eq!(s.regler(false, true), Consignes::default());
        assert_eq!(s.regler(false, false), Consignes::default());
        // Le joueur active le son en pause : les boucles attendent la sortie de pause.
        assert_eq!(s.regler(true, false), Consignes::default());
        // Sortie de pause : musique et pluie démarrent.
        let c = s.regler(true, true);
        assert!(c.demarrer_boucles && !c.arreter_boucles && !c.arreter_effets);
        // Image suivante, rien ne change : aucune consigne (pas de redémarrage en boucle).
        assert_eq!(s.regler(true, true), Consignes::default());
        // Pause (manuelle ou perte de focus) : les boucles s'arrêtent, le son reste actif.
        let c = s.regler(true, false);
        assert!(c.arreter_boucles && !c.arreter_effets && !c.demarrer_boucles);
        assert!(s.actif);
        assert_eq!(s.regler(true, false), Consignes::default());
        // Reprise : elles repartent.
        assert!(s.regler(true, true).demarrer_boucles);
        // Couper le son : tout s'arrête.
        let c = s.regler(false, true);
        assert!(c.arreter_boucles && c.arreter_effets);
        // Couper le son pendant la pause : les boucles étaient déjà arrêtées.
        s.regler(true, true);
        s.regler(true, false);
        let c = s.regler(false, false);
        assert!(!c.arreter_boucles && c.arreter_effets);
    }
}
