//! Tests du crate `niveaux` : chaque fichier se charge, l'ÉQUIVALENCE avec les
//! niveaux Rust actuels (tous les champs, et le `Niveau` chargé), la stabilité du
//! format, la légende du LISEZMOI, et une erreur lisible pour chaque faute.

use super::*;
use crate::source_rust::{avec_legende, legende, sources, ESSAIS};
use pepin_core::level::ErreurNiveau;
use pepin_core::levels::NIVEAUX;

/// Le texte embarqué du fichier `chemin` (panique s'il manque).
fn fichier(chemin: &str) -> &'static str {
    tous()
        .into_iter()
        .find(|&(c, _)| c == chemin)
        .unwrap_or_else(|| panic!("fichier manquant : {chemin}"))
        .1
}

/// Tous les fichiers embarqués (parcours et essais), en paires (chemin, texte).
fn tous() -> Vec<(&'static str, &'static str)> {
    fichiers_embarques()
}

/// LA GARDE DU JEU PUBLIÉ (carte 10.1b) : chaque fichier embarqué se charge et la
/// table est complète et rangée. `./build.sh` lance ce test avant de construire le
/// jeu : un fichier invalide empêche donc de publier (le jeu publié n'a jamais à
/// afficher d'erreur de niveau ; le build de QA, lui, l'affiche à l'écran).
#[test]
fn tous_les_fichiers_embarques_sont_valides() {
    if let Err(e) = table_verifiee() {
        panic!("fichier de niveau invalide : {e}");
    }
    assert_eq!(niveaux().len(), NB_NIVEAUX);
    assert_eq!(niveaux_monde_1().len(), NB_NIVEAUX_MONDE_1);
    assert!(niveaux_monde_1().iter().all(|n| n.monde == 1));
    assert_eq!(essais().len(), NB_ESSAIS);
    let mut cles: Vec<&str> = niveaux()[NB_NIVEAUX_MONDE_1..]
        .iter()
        .map(|n| n.cle)
        .collect();
    cles.sort_unstable();
    assert_eq!(cles, CLES_HORS_MONDE_1);
}

#[test]
fn un_fichier_par_niveau_rust_et_rien_de_plus() {
    let mut attendus: Vec<String> = sources().into_iter().map(|(c, _)| c).collect();
    let mut trouves: Vec<String> = tous().iter().map(|f| f.0.to_string()).collect();
    attendus.sort();
    trouves.sort();
    assert_eq!(trouves, attendus);
    assert_eq!(trouves.len(), NIVEAUX.len() + ESSAIS.len());
}

/// LA PREUVE D'ÉQUIVALENCE : pour chaque niveau, le niveau lu dans son fichier RON
/// a les mêmes champs que le niveau Rust (`InfosNiveau` compare tout : textes,
/// thème, genre, grille, aiguilles, poissons), et le `Niveau` chargé est le même.
/// `Niveau` n'a pas de `PartialEq` (ses entités non plus) : on compare leur
/// affichage de débogage `{:?}`, qui écrit TOUS les champs, tuiles et entités
/// comprises (et les `f32` au bit près : le plus court texte qui les redonne).
#[test]
fn chaque_niveau_ron_est_identique_au_niveau_rust() {
    for (chemin, source) in sources() {
        let charge = charger_fichier(&chemin, fichier(&chemin)).unwrap();
        assert_eq!(&charge, source, "{chemin} : champs différents");
        let niveau_ron = format!("{:?}", charge.charger().unwrap());
        let niveau_rust = format!("{:?}", source.charger().unwrap());
        assert!(
            niveau_ron == niveau_rust,
            "{chemin} : Niveau chargé différent"
        );
    }
}

/// La table rangée par (`monde`, `numero`) est EXACTEMENT `core::levels::NIVEAUX`,
/// dans le même ordre ; les essais sont les mêmes.
#[test]
fn la_table_ron_est_la_table_rust() {
    assert_eq!(niveaux(), NIVEAUX);
    assert_eq!(essais(), ESSAIS);
}

/// Format STABLE : réécrire chaque niveau redonne les octets du fichier du dépôt
/// (relancer l'exporteur ne change rien).
#[test]
fn ecrire_redonne_chaque_fichier() {
    for (chemin, source) in sources() {
        assert!(
            ecrire(source) == fichier(&chemin),
            "{chemin} : relancer l'exporteur"
        );
        let relu = charger(fichier(&chemin)).unwrap();
        assert_eq!(ecrire(&relu), fichier(&chemin), "{chemin}");
    }
}

/// La légende du LISEZMOI est celle de `core/src/level.rs` (recopiée par
/// l'exporteur, jamais à la main).
#[test]
fn la_legende_du_lisezmoi_est_a_jour() {
    let racine = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    let source = std::fs::read_to_string(format!("{racine}/core/src/level.rs")).unwrap();
    let lisezmoi = std::fs::read_to_string(format!("{racine}/assets/levels/LISEZMOI.md")).unwrap();
    let a_jour = avec_legende(&lisezmoi, &legende(&source)).expect("marques de légende");
    assert!(a_jour == lisezmoi, "relancer l'exporteur");
    assert!(lisezmoi.contains("@  départ de Pépin"));
}

/// L'exemple minimal du LISEZMOI se charge (le bloc ```` ```ron ````).
#[test]
fn l_exemple_du_lisezmoi_est_valide() {
    let racine = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    let lisezmoi = std::fs::read_to_string(format!("{racine}/assets/levels/LISEZMOI.md")).unwrap();
    let debut = lisezmoi.find("```ron\n").unwrap() + "```ron\n".len();
    let fin = debut + lisezmoi[debut..].find("```").unwrap();
    let texte = &lisezmoi[debut..fin];
    let table = charger_table(&[
        ("monde_1/toits.ron", fichier("monde_1/toits.ron")),
        ("essais/mon_essai.ron", texte),
    ])
    .unwrap();
    assert_eq!(table.essais[0].nom, "Mon niveau");
}

/// Le texte RON du premier niveau, dont la grille passe par `modifier`.
fn toits_avec_grille(modifier: impl Fn(&str) -> String) -> String {
    let mut infos = NIVEAUX[0];
    infos.ascii = Box::leak(modifier(infos.ascii).into_boxed_str());
    ecrire(&infos)
}

/// L'erreur de grille (panique si l'erreur est d'un autre genre).
fn erreur_de_grille(texte: &str) -> ErreurNiveau {
    match charger_fichier("monde_1/toits.ron", texte)
        .unwrap_err()
        .genre
    {
        GenreErreur::Grille { erreur, .. } => erreur,
        autre => panic!("erreur de grille attendue, pas {autre:?}"),
    }
}

#[test]
fn lettre_inconnue_avec_sa_ligne_et_sa_colonne() {
    let texte = toits_avec_grille(|g| g.replacen("@..D", "@.?D", 1));
    let e = charger_fichier("monde_1/toits.ron", &texte).unwrap_err();
    assert_eq!(
        e.genre,
        GenreErreur::Grille {
            erreur: ErreurNiveau::CaractereInconnu {
                ligne: 8,
                colonne: 5,
                caractere: '?'
            },
            premiere_ligne: 16,
        }
    );
    // La ligne 8 de la grille est la ligne 23 du fichier (16 + 8 - 1).
    assert_eq!(texte.lines().nth(22).unwrap().chars().nth(4), Some('?'));
    let message = e.to_string();
    assert!(message.contains("monde_1/toits.ron"), "{message}");
    assert!(message.contains("ligne 8, colonne 5"), "{message}");
    assert!(message.contains("« ? »"), "{message}");
}

#[test]
fn depart_et_sortie_manquants_ou_en_double() {
    let sans_depart = toits_avec_grille(|g| g.replace('@', "."));
    assert_eq!(erreur_de_grille(&sans_depart), ErreurNiveau::DepartManquant);
    let deux_departs = toits_avec_grille(|g| g.replacen("..@..D", "@.@..D", 1));
    assert_eq!(
        erreur_de_grille(&deux_departs),
        ErreurNiveau::DepartMultiple { nombre: 2 }
    );
    let sans_sortie = toits_avec_grille(|g| g.replace('X', "."));
    assert_eq!(
        erreur_de_grille(&sans_sortie),
        ErreurNiveau::ArriveeManquante
    );
}

#[test]
fn dimension_inegale() {
    let texte = toits_avec_grille(|g| g.replacen("..@", ".@", 1));
    assert!(matches!(
        erreur_de_grille(&texte),
        ErreurNiveau::LargeurInegale { ligne: 8, .. }
    ));
}

#[test]
fn aiguille_ou_poisson_sans_case() {
    // Une aiguille réglée, mais aucun pivot `h` dans la grille des toits.
    let texte = ecrire(&NIVEAUX[0]).replace(
        "aiguilles: [],",
        "aiguilles: [(rayon: 2.0, vitesse: 20.0, phase: 0.0)],",
    );
    assert_eq!(
        erreur_de_grille(&texte),
        ErreurNiveau::AiguillesSansReglage {
            pivots: 0,
            reglages: 1
        }
    );
    let texte =
        ecrire(&NIVEAUX[0]).replace("poissons_hauts: [],", "poissons_hauts: [(3, 4, 50.0)],");
    assert_eq!(
        erreur_de_grille(&texte),
        ErreurNiveau::PoissonIntrouvable {
            ligne: 5,
            colonne: 4
        }
    );
}

#[test]
fn champ_inconnu_ou_theme_inconnu_avec_la_ligne_du_fichier() {
    let texte =
        ecrire(&NIVEAUX[0]).replace("    sans_planer:", "    vitesse: 3,\n    sans_planer:");
    let e = charger(&texte).unwrap_err();
    match &e.genre {
        GenreErreur::Syntaxe { ligne, message, .. } => {
            assert_eq!(*ligne, 9);
            assert!(message.contains("vitesse"), "{message}");
        }
        autre => panic!("{autre:?}"),
    }
    assert!(e.to_string().starts_with("(texte) : ligne 9"), "{e}");
    let texte = ecrire(&NIVEAUX[0]).replace("theme: Toits", "theme: Plage");
    assert!(matches!(
        charger(&texte).unwrap_err().genre,
        GenreErreur::Syntaxe { ligne: 7, .. }
    ));
}

#[test]
fn erreurs_de_table_cle_dossier_et_numerotation() {
    let toits = fichier("monde_1/toits.ron");
    let genre = |fichiers: &[(&str, &str)]| charger_table(fichiers).unwrap_err().genre;
    assert_eq!(
        genre(&[("monde_1/toits.ron", toits), ("essais/toits.ron", toits)]),
        GenreErreur::CleEnDouble {
            cle: "toits".into(),
            autre_fichier: "monde_1/toits.ron".into()
        }
    );
    assert_eq!(
        genre(&[("monde_2/toits.ron", toits)]),
        GenreErreur::MondeHorsDeSonDossier {
            dossier: 2,
            monde: 1
        }
    );
    assert_eq!(
        genre(&[("monde_1/autre.ron", toits)]),
        GenreErreur::CleDifferenteDuFichier {
            cle: "toits".into()
        }
    );
    assert_eq!(
        genre(&[("divers/toits.ron", toits)]),
        GenreErreur::DossierInconnu
    );
    assert_eq!(
        genre(&[("essais/toits.ron", toits)]),
        GenreErreur::EssaiNumerote { numero: 1 }
    );
    // Un trou dans la numérotation : le Monde 1 sans son niveau 2.
    let sans_le_2: Vec<_> = tous()
        .into_iter()
        .filter(|(c, _)| *c != "monde_1/gouttieres.ron")
        .collect();
    let e = charger_table(&sans_le_2).unwrap_err();
    assert_eq!(e.fichier, "monde_1/marche.ron");
    assert!(matches!(
        e.genre,
        GenreErreur::Ordre(ErreurOrdre::NumeroFaux { numero: 3, .. })
    ));
    assert!(e.to_string().contains("sans trou"), "{e}");
}

/// Ajouter un niveau = ajouter un fichier : un nouveau fichier `monde_3/` prend
/// sa place d'après ses champs, sans aucune liste à modifier.
#[test]
fn ajouter_un_niveau_c_est_ajouter_un_fichier() {
    let mut nouveau = NIVEAUX[0];
    nouveau.monde = 3;
    nouveau.cle = "nouveau";
    let texte = ecrire(&nouveau);
    let mut fichiers = vec![("monde_3/nouveau.ron", texte.as_str())];
    fichiers.extend(tous());
    let table = charger_table(&fichiers).unwrap();
    assert_eq!(table.niveaux.len(), NIVEAUX.len() + 1);
    assert_eq!(table.niveaux.last().unwrap().cle, "nouveau");
}

/// Carte 10.3a : la légende de l'éditeur (`crate::legende`) contient EXACTEMENT
/// les lettres que le parseur de `core` accepte. Chaque caractère ASCII visible
/// est posé dans une grille valide : le parseur ne doit répondre « caractère
/// inconnu » que pour ceux qui n'ont pas de ligne dans la légende.
#[test]
fn la_legende_couvre_exactement_les_lettres_du_parseur() {
    use crate::legende::LETTRES;
    let base = toits_avec_grille(|g| g.to_string());
    let rangee_depart = NIVEAUX[0]
        .ascii
        .lines()
        .position(|r| r.contains('@'))
        .unwrap();
    for code in 0x21u8..=0x7e {
        let c = code as char;
        // Le caractère remplace la case de la colonne 0 de la rangée du départ
        // (un `.` ; `"` fermerait la grille brute, on le pose quand même : le
        // RON devient invalide, ce n'est donc pas une lettre acceptée).
        let texte = base.replacen(
            NIVEAUX[0].ascii.lines().nth(rangee_depart).unwrap(),
            &format!(
                "{c}{}",
                &NIVEAUX[0].ascii.lines().nth(rangee_depart).unwrap()[1..]
            ),
            1,
        );
        let inconnu = matches!(
            charger(&texte),
            Err(ErreurChargement {
                genre: GenreErreur::Grille {
                    erreur: ErreurNiveau::CaractereInconnu { .. },
                    ..
                },
                ..
            }) | Err(ErreurChargement {
                genre: GenreErreur::Syntaxe { .. },
                ..
            })
        );
        let dans_la_legende = LETTRES.iter().any(|l| l.lettre == c);
        assert_eq!(
            !inconnu, dans_la_legende,
            "« {c} » : accepté par le parseur = {}, dans la légende = {dans_la_legende}",
            !inconnu
        );
    }
    // Pas de lettre en double, et le vide `.` y est.
    let mut vues: Vec<char> = LETTRES.iter().map(|l| l.lettre).collect();
    vues.sort();
    vues.dedup();
    assert_eq!(
        vues.len(),
        LETTRES.len(),
        "lettre en double dans la légende"
    );
    assert!(vues.contains(&'.'));
}

/// `legende.json` cite chaque lettre, chaque thème et chaque genre.
#[test]
fn legende_json_contient_lettres_themes_et_genres() {
    let texte = crate::legende::legende_json();
    for l in crate::legende::LETTRES {
        let lettre = if l.lettre == '\\' || l.lettre == '"' {
            format!("\\{}", l.lettre)
        } else {
            l.lettre.to_string()
        };
        assert!(
            texte.contains(&format!("\"lettre\": \"{lettre}\"")),
            "{lettre}"
        );
    }
    for theme in ["\"Toits\"", "\"Port\"", "\"Maree\""] {
        assert!(texte.contains(theme), "{theme}");
    }
    assert!(texte.contains("\"genres\": [\"Normal\", \"Bonus\", \"Epilogue\"]"));
}
