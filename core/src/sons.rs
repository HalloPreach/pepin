//! Catalogue des sons de PÉPIN : chaque effet est « composé » ici à partir des
//! notes de `synthese`, puis encodé en WAV mono 16 bits en mémoire.
//!
//! Aucun fichier audio n'est embarqué : seul le code qui les fabrique pèse dans
//! le binaire. Le jeu (module `audio` du crate `game`) fabrique les WAV au
//! démarrage et les confie à macroquad. Tout ce fichier est pur : il se teste
//! sans carte son.
//!
//! Notions Rust illustrées ici : `enum` + `match` exhaustif pour un catalogue,
//! tableaux constants de notes (la mélodie), écriture d'un format binaire
//! (`extend_from_slice`, `to_le_bytes`).

use crate::synthese::{ajouter, midi_vers_hz, vers_pcm16, Bruit, Note, Onde, TAUX};

/// Les effets sonores ponctuels du jeu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Effet {
    Saut,
    /// Le parapluie s'ouvre.
    Plane,
    Atterrissage,
    /// Goutte d'or ramassée.
    Goutte,
    /// Ennemi écrasé.
    Ecrase,
    /// Champignon trampoline.
    Rebond,
    /// Lanterne allumée (point de reprise).
    Lanterne,
    Mort,
    /// Coup de tonnerre d'un éclair.
    Eclair,
    /// Drapeau d'arrivée atteint.
    Victoire,
    /// Déplacement / validation dans les menus.
    Menu,
}

impl Effet {
    /// Tous les effets, pour tout charger d'un coup.
    pub const TOUS: [Effet; 11] = [
        Effet::Saut,
        Effet::Plane,
        Effet::Atterrissage,
        Effet::Goutte,
        Effet::Ecrase,
        Effet::Rebond,
        Effet::Lanterne,
        Effet::Mort,
        Effet::Eclair,
        Effet::Victoire,
        Effet::Menu,
    ];

    /// Volume de lecture conseillé (de 0.0 à 1.0) : les bruits forts sont baissés.
    pub fn volume(self) -> f32 {
        match self {
            Effet::Eclair => 0.55,
            Effet::Atterrissage => 0.5,
            Effet::Plane => 0.6,
            _ => 0.8,
        }
    }
}

const CARRE: Onde = Onde::Carre { rapport: 0.5 };
const NASILLARD: Onde = Onde::Carre { rapport: 0.25 };

/// Succession de notes (une après l'autre), `pas_s` secondes entre deux départs.
fn arpege(onde: Onde, midis: &[u8], pas_s: f32, duree: f32, volume: f32) -> Vec<f32> {
    let mut bruit = Bruit::new();
    let mut son = Vec::new();
    for (i, &m) in midis.iter().enumerate() {
        let note = Note::fixe(onde, midi_vers_hz(m), duree, volume).declin(1.5);
        ajouter(&mut son, &note.rendre(&mut bruit), i as f32 * pas_s);
    }
    son
}

/// Fabrique les échantillons (entre -1 et 1) d'un effet.
pub fn echantillons(effet: Effet) -> Vec<f32> {
    let mut bruit = Bruit::new();
    let mut note = |n: Note| n.rendre(&mut bruit);
    match effet {
        // Petit « boing » montant.
        Effet::Saut => note(Note::new(CARRE, 260.0, 620.0, 0.14, 0.30)),
        // Souffle doux : le parapluie se tend.
        Effet::Plane => note(Note::new(Onde::Bruit, 2400.0, 5200.0, 0.16, 0.22).declin(0.7)),
        Effet::Atterrissage => note(Note::new(Onde::Bruit, 500.0, 200.0, 0.07, 0.30).declin(2.0)),
        // Deux notes aiguës, façon pièce.
        Effet::Goutte => arpege(CARRE, &[88, 95], 0.07, 0.16, 0.28),
        Effet::Ecrase => note(Note::new(NASILLARD, 320.0, 70.0, 0.12, 0.34).declin(1.3)),
        // Ressort : glissando montant plus long, onde nasillarde.
        Effet::Rebond => note(Note::new(NASILLARD, 180.0, 900.0, 0.22, 0.30).declin(0.8)),
        // Petit carillon à trois notes.
        Effet::Lanterne => arpege(Onde::Triangle, &[72, 76, 79], 0.09, 0.22, 0.45),
        // Chute en spirale vers le grave.
        Effet::Mort => {
            let mut son = note(Note::new(CARRE, 520.0, 60.0, 0.55, 0.30).declin(0.8));
            let fond = note(Note::new(Onde::Bruit, 300.0, 90.0, 0.4, 0.18).declin(1.5));
            ajouter(&mut son, &fond, 0.0);
            son
        }
        // Craquement aigu puis grondement grave.
        Effet::Eclair => {
            let mut son = note(Note::new(Onde::Bruit, 8000.0, 1500.0, 0.12, 0.55).declin(1.0));
            let gronde = note(Note::new(Onde::Bruit, 180.0, 60.0, 0.7, 0.5).declin(1.2));
            ajouter(&mut son, &gronde, 0.05);
            son
        }
        // Fanfare ascendante : do mi sol do aigu.
        Effet::Victoire => {
            let mut son = arpege(CARRE, &[72, 76, 79], 0.11, 0.2, 0.26);
            let fin = note(Note::fixe(CARRE, midi_vers_hz(84), 0.5, 0.28).declin(1.2));
            ajouter(&mut son, &fin, 0.33);
            let basse = note(Note::fixe(Onde::Triangle, midi_vers_hz(60), 0.8, 0.4));
            ajouter(&mut son, &basse, 0.0);
            son
        }
        Effet::Menu => note(Note::fixe(CARRE, 740.0, 0.05, 0.22)),
    }
}

/// Pluie douce : un souffle de bruit adouci (filtre passe-bas), à jouer en
/// boucle. Le bruit étant continu, la jonction de la boucle ne s'entend pas.
pub fn pluie() -> Vec<f32> {
    let n = 2 * TAUX as usize;
    let mut bruit = Bruit::new();
    let mut lisse = 0.0_f32;
    (0..n)
        .map(|_| {
            // Filtre à un pôle : chaque valeur glisse vers le nouveau bruit.
            lisse += 0.18 * (bruit.suivant() - lisse);
            lisse * 0.6
        })
        .collect()
}

/// Mélodie en boucle : (note MIDI, durée en croches). `0` = silence.
/// Quatre mesures de 8 croches, ambiance calme en la mineur puis do majeur.
const MELODIE: [(u8, u8); 19] = [
    // Mesure 1
    (69, 2),
    (72, 1),
    (76, 1),
    (74, 2),
    (72, 2),
    // Mesure 2
    (71, 2),
    (67, 2),
    (64, 2),
    (0, 2),
    // Mesure 3
    (69, 2),
    (72, 1),
    (76, 1),
    (79, 2),
    (76, 2),
    // Mesure 4
    (74, 2),
    (72, 1),
    (74, 1),
    (72, 2),
    (0, 2),
];

/// Basse : une note par mesure de 8 croches (la, sol, fa, mi pour le retour).
const BASSE: [u8; 4] = [45, 43, 41, 40];

/// Durée d'une croche de la boucle musicale, en secondes (≈ 100 battements/min).
const CROCHE: f32 = 0.3;

/// Boucle musicale chiptune de ~ 10 s : mélodie carrée nasillarde + basse triangle.
pub fn musique() -> Vec<f32> {
    let mut bruit = Bruit::new();
    let mut son = Vec::new();
    let mut t = 0.0_f32;
    for &(midi, croches) in &MELODIE {
        let duree = croches as f32 * CROCHE;
        if midi != 0 {
            let note = Note::fixe(NASILLARD, midi_vers_hz(midi), duree * 0.9, 0.16).declin(0.6);
            ajouter(&mut son, &note.rendre(&mut bruit), t);
        }
        t += duree;
    }
    for (mesure, &midi) in BASSE.iter().enumerate() {
        let debut = mesure as f32 * 8.0 * CROCHE;
        for moitie in 0..2 {
            let note = Note::fixe(Onde::Triangle, midi_vers_hz(midi), 4.0 * CROCHE * 0.95, 0.4)
                .declin(0.5);
            ajouter(
                &mut son,
                &note.rendre(&mut bruit),
                debut + moitie as f32 * 4.0 * CROCHE,
            );
        }
    }
    // La boucle dure exactement 4 mesures : on coupe les queues qui débordent et
    // on complète par du silence, pour que la boucle retombe juste.
    son.resize((4.0 * 8.0 * CROCHE * TAUX as f32) as usize, 0.0);
    son
}

/// Encode des échantillons en fichier WAV (mono, 16 bits, `TAUX` Hz) en mémoire.
pub fn vers_wav(echantillons: &[f32]) -> Vec<u8> {
    let pcm = vers_pcm16(echantillons);
    let octets_donnees = (pcm.len() * 2) as u32;
    let mut wav = Vec::with_capacity(44 + pcm.len() * 2);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + octets_donnees).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // taille du bloc fmt
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // mono
    wav.extend_from_slice(&TAUX.to_le_bytes());
    wav.extend_from_slice(&(TAUX * 2).to_le_bytes()); // octets par seconde
    wav.extend_from_slice(&2u16.to_le_bytes()); // octets par échantillon
    wav.extend_from_slice(&16u16.to_le_bytes()); // bits par échantillon
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&octets_donnees.to_le_bytes());
    for s in pcm {
        wav.extend_from_slice(&s.to_le_bytes());
    }
    wav
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pic(s: &[f32]) -> f32 {
        s.iter().fold(0.0_f32, |m, v| m.max(v.abs()))
    }

    #[test]
    fn chaque_effet_est_audible_court_et_sans_ecretage() {
        for e in Effet::TOUS {
            let s = echantillons(e);
            let duree = s.len() as f32 / TAUX as f32;
            assert!((0.04..1.0).contains(&duree), "{e:?} dure {duree} s");
            let p = pic(&s);
            assert!(p > 0.1, "{e:?} trop faible ({p})");
            assert!(p < 1.0, "{e:?} écrête ({p})");
            assert!(s.iter().all(|v| v.is_finite()));
        }
    }

    #[test]
    fn les_effets_sont_tous_differents() {
        let sons: Vec<Vec<f32>> = Effet::TOUS.iter().map(|&e| echantillons(e)).collect();
        for i in 0..sons.len() {
            for j in (i + 1)..sons.len() {
                assert_ne!(
                    sons[i],
                    sons[j],
                    "{:?} = {:?}",
                    Effet::TOUS[i],
                    Effet::TOUS[j]
                );
            }
        }
    }

    #[test]
    fn la_musique_dure_quatre_mesures_et_ne_depasse_pas() {
        let m = musique();
        let attendu = (32.0 * CROCHE * TAUX as f32) as usize;
        assert_eq!(m.len(), attendu);
        assert!(pic(&m) < 1.0 && pic(&m) > 0.2, "pic {}", pic(&m));
    }

    #[test]
    fn la_melodie_remplit_exactement_les_quatre_mesures() {
        let total: u32 = MELODIE.iter().map(|&(_, c)| c as u32).sum();
        assert_eq!(total, 32);
        assert_eq!(BASSE.len() * 8, 32);
    }

    #[test]
    fn la_pluie_est_douce_et_boucle_sans_saut_brutal() {
        let p = pluie();
        assert_eq!(p.len(), 2 * TAUX as usize);
        assert!(pic(&p) < 0.5 && pic(&p) > 0.05, "pic {}", pic(&p));
        // Le filtre empêche les sauts brusques d'un échantillon à l'autre.
        let saut_max = p
            .windows(2)
            .fold(0.0_f32, |m, w| m.max((w[1] - w[0]).abs()));
        assert!(saut_max < 0.4, "saut {saut_max}");
    }

    #[test]
    fn l_en_tete_wav_est_correct() {
        let wav = vers_wav(&[0.0, 0.5, -0.5]);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..16], b"WAVEfmt ");
        assert_eq!(
            u32::from_le_bytes(wav[4..8].try_into().unwrap()),
            wav.len() as u32 - 8
        );
        assert_eq!(
            u16::from_le_bytes(wav[20..22].try_into().unwrap()),
            1,
            "PCM"
        );
        assert_eq!(
            u16::from_le_bytes(wav[22..24].try_into().unwrap()),
            1,
            "mono"
        );
        assert_eq!(u32::from_le_bytes(wav[24..28].try_into().unwrap()), TAUX);
        assert_eq!(&wav[36..40], b"data");
        assert_eq!(u32::from_le_bytes(wav[40..44].try_into().unwrap()), 6);
        assert_eq!(wav.len(), 44 + 6);
        assert_eq!(i16::from_le_bytes(wav[46..48].try_into().unwrap()), 16383);
    }
}
