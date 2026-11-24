//! Captures de la carte 9.9 (« La grotte aux bulles ») : le script de la partie
//! complète du Monde 2 et les images à photographier dans la grotte. Test ignoré,
//! lancé à la main :
//! `./docker-cargo.sh test -p game --no-default-features ecrire_script_monde_2_99 -- --ignored --nocapture`
//!
//! Il écrit `target/qa/script-99.txt` (même format que `script-96.txt` : le pilote
//! `docs/capture-96.mjs` le rejoue depuis le titre) et affiche, pour chaque moment
//! de la grotte, l'image du script où le photographier.
//!
//! Comment on trouve l'image d'un moment : on rejoue le parcours de la grotte seul
//! (`Monde::new`), on note le pas du niveau où le moment arrive, puis on cherche
//! dans la partie l'image où Pépin a fait les mêmes 30 derniers pas, aux mêmes
//! positions (la partie et le niveau seul jouent la même physique).

use pepin_core::entities::stalactite::EtatStalactite;
use pepin_core::entities::Entite;
use pepin_core::levels::debut_monde;

use super::{jouer_monde_2, tous_les_parcours_complets_du_monde, Partie};
use crate::monde::Monde;

/// Nombre de positions comparées pour retrouver un pas du niveau dans la partie.
const FENETRE: usize = 30;

/// L'image de la partie où Pépin vient de jouer le pas `pas` du niveau numéro
/// `numero`, d'après les positions du niveau rejoué seul (`positions[k]` : la
/// position après le pas `k`).
fn image_du_pas(partie: &Partie, numero: u8, positions: &[(f32, f32)], pas: usize) -> usize {
    let n = FENETRE.min(pas);
    (n..partie.trames.len())
        .find(|&i| {
            (0..=n).all(|j| {
                let t = &partie.trames[i - j];
                t.niveau == numero && (t.x, t.y) == positions[pas - j]
            })
        })
        .unwrap_or_else(|| panic!("pas {pas} introuvable dans la partie"))
}

#[test]
#[ignore]
fn ecrire_script_monde_2_99() {
    let parcours = tous_les_parcours_complets_du_monde(2);
    let partie = jouer_monde_2(&parcours);
    assert!(partie.fin_atteinte);
    assert_eq!(partie.morts, 0);
    std::fs::create_dir_all("../target/qa").unwrap();
    let script: String = partie.trames.iter().map(|t| t.code()).collect();
    std::fs::write("../target/qa/script-99.txt", &script).unwrap();
    println!(
        "script-99.txt : {} images ; gouttes {} ; morts {}",
        script.len(),
        partie.gouttes,
        partie.morts
    );

    // La grotte rejouée seule : les pas de ses trois moments.
    let numero = niveaux::par_cle("grotte").numero;
    let indice = debut_monde(niveaux::niveaux(), 2).unwrap() + usize::from(numero) - 1;
    assert_eq!(niveaux::niveaux()[indice].cle, "grotte");
    let mut m = Monde::new(indice);
    let mut positions = Vec::new();
    let mut moments: Vec<(&str, usize)> = Vec::new();
    for (pas, e) in parcours[usize::from(numero) - 1].iter().enumerate() {
        let avant = m.gouttes_niveau;
        m.avancer(e);
        let j = &m.joueur;
        positions.push((j.boite.x, j.boite.y));
        let mut noter = |nom: &'static str, vrai: bool| {
            if vrai && !moments.iter().any(|&(n, _)| n == nom) {
                moments.push((nom, pas));
            }
        };
        // Une ascension : en pleine montée dans la 3e colonne (colonnes 22-23),
        // à mi-hauteur, la 4e colonne à côté.
        noter(
            "99-ascension-bulles",
            (352.0..374.0).contains(&j.boite.x)
                && (200.0..300.0).contains(&j.boite.y)
                && j.vitesse.y < -100.0,
        );
        // Une stalactite qui tombe, à moins de 6 tuiles de Pépin (à l'écran),
        // déjà détachée du plafond (elle tombe depuis au moins 1/6 s).
        let chute_proche = m.entites.iter().any(|e| match e {
            Entite::Stalactite(s) => {
                matches!(s.etat, EtatStalactite::Chute(v) if v > 150.0)
                    && (s.boite.x - j.boite.x).abs() < 96.0
                    && (s.boite.y - j.boite.y).abs() < 96.0
            }
            _ => false,
        });
        noter("99-stalactite-tombe", chute_proche);
        // La goutte risquée : la 1re dans l'ordre de lecture (la corniche du
        // haut), juste prise.
        noter("99-goutte-risquee", !avant[0] && m.gouttes_niveau[0]);
    }
    assert_eq!(moments.len(), 3, "moments trouvés : {moments:?}");
    for (nom, pas) in moments {
        // Quelques images après le moment : la chute bien visible, la goutte
        // comptée dans l'affichage.
        let pas = (pas + 4).min(positions.len() - 1);
        // Le pilote photographie APRÈS avoir joué les images `0..cible` : l'image
        // d'indice `i` est donc à demander comme `nom:i+1`.
        let image = image_du_pas(&partie, numero, &positions, pas) + 1;
        println!("{nom}:{image} (pas {pas} de la grotte)");
    }
}
