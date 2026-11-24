//! Tests du CHARGEUR : chaque fichier se charge, la stabilité du format, la
//! légende du LISEZMOI, et une erreur lisible pour chaque faute. (Les tests du
//! CONTENU des niveaux, eux, sont dans `tests/tous_les_niveaux/`.)
//!
//! Carte 10.1c : les tests d'équivalence avec les anciens niveaux Rust de `core`
//! sont partis avec eux (l'historique git les garde).

use super::*;
use crate::lisezmoi::{avec_legende, legende};
use pepin_core::level::ErreurNiveau;
use pepin_core::levels::cles_valides;

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
}

/// Les clés sont des identifiants (minuscules, chiffres, `_`), toutes
/// différentes, parcours et essais ensemble (autrefois vérifié à la compilation
/// sur la table Rust de `core`).
#[test]
fn les_cles_sont_des_identifiants_uniques() {
    let tous: Vec<InfosNiveau> = niveaux().iter().chain(essais()).copied().collect();
    assert!(cles_valides(&tous));
    let mut doublon = tous.clone();
    doublon.push(tous[0]);
    assert!(!cles_valides(&doublon));
}

/// Les lignes d'un fichier sans ses commentaires `//` (que `ecrire` n'écrit pas,
/// sauf l'en-tête) ni ses lignes vides (celles où `build.rs` a vidé un
/// commentaire du texte embarqué).
fn sans_commentaires(texte: &str) -> Vec<&str> {
    texte
        .lines()
        .filter(|l| !l.trim_start().starts_with("//") && !l.is_empty())
        .collect()
}

/// Le texte EMBARQUÉ de chaque fichier est celui du dépôt, chaque ligne de
/// commentaire `//` vidée par `build.rs` (le jeu publié n'emporte pas les
/// commentaires de conception) : mêmes lignes, au même numéro, donc une erreur
/// donne la ligne du fichier.
#[test]
fn le_texte_embarque_est_le_fichier_sans_ses_commentaires() {
    let racine = concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/levels");
    let mut vides = 0;
    for (chemin, embarque) in tous() {
        let disque = std::fs::read_to_string(format!("{racine}/{chemin}")).unwrap();
        let (d, e): (Vec<&str>, Vec<&str>) = (disque.lines().collect(), embarque.lines().collect());
        assert_eq!(d.len(), e.len(), "{chemin} : nombre de lignes");
        for (k, (ld, le)) in d.iter().zip(&e).enumerate() {
            if ld.trim_start().starts_with("//") {
                assert_eq!(*le, "", "{chemin} : ligne {}", k + 1);
                vides += 1;
            } else {
                assert_eq!(ld, le, "{chemin} : ligne {}", k + 1);
            }
        }
    }
    assert!(
        vides > 1000,
        "les commentaires de conception sont dans les fichiers"
    );
}

/// Format STABLE : chaque fichier du dépôt est au format canonique. Relu puis
/// réécrit par `ecrire` (l'écriture que reproduit l'éditeur, `editeur/ron.js`), il
/// redonne les mêmes lignes, commentaires de conception mis à part.
#[test]
fn l_export_du_ron_est_stable() {
    for (chemin, texte) in tous() {
        let relu = charger_fichier(chemin, texte).unwrap();
        let ecrit = ecrire(&relu);
        assert_eq!(
            sans_commentaires(&ecrit),
            sans_commentaires(texte),
            "{chemin}"
        );
        assert_eq!(charger(&ecrit).unwrap(), relu, "{chemin}");
    }
}

/// La légende du LISEZMOI est celle de `core/src/level.rs` (recopiée par
/// `--bin lisezmoi`, jamais à la main).
#[test]
fn la_legende_du_lisezmoi_est_a_jour() {
    let racine = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    let source = std::fs::read_to_string(format!("{racine}/core/src/level.rs")).unwrap();
    let lisezmoi = std::fs::read_to_string(format!("{racine}/assets/levels/LISEZMOI.md")).unwrap();
    let a_jour = avec_legende(&lisezmoi, &legende(&source)).expect("marques de légende");
    assert!(
        a_jour == lisezmoi,
        "relancer ./docker-cargo.sh run -p niveaux --bin lisezmoi"
    );
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
    let mut infos = niveaux()[0];
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
            premiere_ligne: 17,
        }
    );
    // La ligne 8 de la grille est la ligne 24 du fichier (17 + 8 - 1).
    assert_eq!(texte.lines().nth(23).unwrap().chars().nth(4), Some('?'));
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
    let texte = ecrire(&niveaux()[0]).replace(
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
        ecrire(&niveaux()[0]).replace("poissons_hauts: [],", "poissons_hauts: [(3, 4, 50.0)],");
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
        ecrire(&niveaux()[0]).replace("    sans_planer:", "    vitesse: 3,\n    sans_planer:");
    let e = charger(&texte).unwrap_err();
    match &e.genre {
        GenreErreur::Syntaxe { ligne, message, .. } => {
            assert_eq!(*ligne, 9);
            assert!(message.contains("vitesse"), "{message}");
        }
        autre => panic!("{autre:?}"),
    }
    assert!(e.to_string().starts_with("(texte) : ligne 9"), "{e}");
    let texte = ecrire(&niveaux()[0]).replace("theme: Toits", "theme: Plage");
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

/// Carte 11.3 : un niveau sans intro (ou à l'intro trop longue pour la carte du
/// niveau) est refusé, avec un message qui nomme le champ ; l'ancien champ `intro`
/// est refusé avec le nom des deux nouveaux.
#[test]
fn un_niveau_sans_intro_est_refuse_clairement() {
    use crate::intro::ErreurIntro;
    let toits = fichier("monde_1/toits.ron");
    let ligne_de = |champ: &str| {
        toits
            .lines()
            .find(|l| l.starts_with(&format!("    {champ}: ")))
            .unwrap()
            .to_string()
    };
    let avec = |champ: &str, valeur: &str| {
        toits.replace(&ligne_de(champ), &format!("    {champ}: \"{valeur}\","))
    };
    let refus = |texte: &str| charger_fichier("monde_1/toits.ron", texte).unwrap_err();

    let e = refus(&avec("intro_texte", ""));
    assert_eq!(
        e.genre,
        GenreErreur::Intro(ErreurIntro::Vide {
            champ: "intro_texte"
        })
    );
    assert_eq!(
        e.to_string(),
        "monde_1/toits.ron : « intro_texte » est vide : chaque niveau a son intro \
         (la phrase dans intro_texte, le conseil dans intro_astuce)"
    );
    // Des espaces seuls ne font pas une intro.
    assert_eq!(
        refus(&avec("intro_astuce", "   ")).genre,
        GenreErreur::Intro(ErreurIntro::Vide {
            champ: "intro_astuce"
        })
    );
    // Trois lignes de conseil : trop pour la carte.
    let long = "mot ".repeat(30);
    let e = refus(&avec("intro_astuce", long.trim()));
    assert_eq!(
        e.genre,
        GenreErreur::Intro(ErreurIntro::TropLongue {
            champ: "intro_astuce",
            lignes: 3,
            max: 2
        })
    );
    assert!(e.to_string().contains("3 lignes"), "{e}");
    // Trois lignes de phrase et deux de conseil : chacune tient, pas les deux.
    let e = refus(
        &avec("intro_texte", long.trim()).replace(&ligne_de("intro_astuce"), &{
            let deux = "mot ".repeat(15);
            format!("    intro_astuce: \"{}\",", deux.trim())
        }),
    );
    assert_eq!(
        e.genre,
        GenreErreur::Intro(ErreurIntro::TropLongueEnTout { lignes: 5 })
    );
    // Un champ manquant, ou l'ancien champ `intro` (fichier d'avant la carte 11.3).
    let sans = toits.replace(&format!("{}\n", ligne_de("intro_astuce")), "");
    let e = refus(&sans).to_string();
    assert!(
        e.contains("intro_astuce") && e.contains("le conseil"),
        "{e}"
    );
    let ancien = toits.replace(
        &format!("{}\n{}", ligne_de("intro_texte"), ligne_de("intro_astuce")),
        "    intro: \"Vieux.\",",
    );
    let e = refus(&ancien).to_string();
    assert!(
        e.contains("`intro`") && e.contains("intro_texte, la phrase"),
        "{e}"
    );
    // Le comptage des lignes est celui de la carte (coupure entre les mots, 46).
    assert_eq!(crate::intro::nb_lignes(""), 0);
    assert_eq!(crate::intro::nb_lignes(&"a".repeat(46)), 1);
    assert_eq!(crate::intro::nb_lignes(&format!("{} b", "a".repeat(44))), 1);
    assert_eq!(
        crate::intro::nb_lignes(&format!("{} bc", "a".repeat(44))),
        2
    );
}

/// L'éditeur applique les MÊMES limites que le chargeur (`editeur/ron.js`, qui ne
/// peut pas importer le Rust) : ce test relit ses constantes.
#[test]
fn l_editeur_a_les_memes_limites_d_intro() {
    use crate::intro::*;
    let ron_js = include_str!("../../editeur/ron.js");
    for attendu in [
        format!("export const LIGNE_MAX_INTRO = {LIGNE_MAX_INTRO};"),
        format!(
            "export const LIGNES_MAX = {{ intro_texte: {LIGNES_MAX_TEXTE}, intro_astuce: {LIGNES_MAX_ASTUCE} }};"
        ),
        format!("export const LIGNES_MAX_INTRO = {LIGNES_MAX_INTRO};"),
    ] {
        assert!(ron_js.contains(&attendu), "editeur/ron.js : « {attendu} » manque");
    }
}

/// Ajouter un niveau = ajouter un fichier : un nouveau fichier `monde_3/` prend
/// sa place d'après ses champs, sans aucune liste à modifier.
#[test]
fn ajouter_un_niveau_c_est_ajouter_un_fichier() {
    let mut nouveau = niveaux()[0];
    nouveau.monde = 3;
    nouveau.cle = "nouveau";
    let texte = ecrire(&nouveau);
    let mut fichiers = vec![("monde_3/nouveau.ron", texte.as_str())];
    fichiers.extend(tous());
    let table = charger_table(&fichiers).unwrap();
    assert_eq!(table.niveaux.len(), niveaux().len() + 1);
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
    let rangee_depart = niveaux()[0]
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
            niveaux()[0].ascii.lines().nth(rangee_depart).unwrap(),
            &format!(
                "{c}{}",
                &niveaux()[0].ascii.lines().nth(rangee_depart).unwrap()[1..]
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

/// Carte 10.4b : chaque vignette de la légende est un VRAI sprite (un fichier
/// `assets/sprites/<famille>/<clé>.png`, trouvé dans la table écrite par
/// `build.rs`), chaque lettre a au plus une vignette et elle appartient à la
/// légende ; `legende.json` donne le chemin et l'ancrage de chacune.
#[test]
fn chaque_vignette_est_un_vrai_sprite() {
    use crate::legende::{vignette, LETTRES, VIGNETTES};
    let racine = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/sprites");
    let json = crate::legende::legende_json();
    let mut vues = Vec::new();
    for &(c, cle, ancre) in VIGNETTES {
        assert!(!vues.contains(&c), "« {c} » a deux vignettes");
        vues.push(c);
        assert!(
            LETTRES.iter().any(|l| l.lettre == c),
            "« {c} » n'est pas une lettre"
        );
        let (chemin, a) =
            vignette(c).unwrap_or_else(|| panic!("« {c} » : pas de fichier {cle}.png"));
        assert_eq!(a, ancre);
        assert!(chemin.ends_with(&format!("/{cle}")), "{chemin}");
        assert!(
            racine.join(format!("{chemin}.png")).is_file(),
            "{chemin}.png absent"
        );
        assert!(
            json.contains(&format!("\"sprite\": \"{chemin}\"")),
            "{chemin} dans le JSON"
        );
    }
    // Les tuiles du thème gardent leur case colorée.
    assert!(vignette('#').is_none() && vignette('~').is_none());
    assert!(json.contains("\"sprite\": null, \"ancre\": null"));
    // La table des sprites est bien celle des fichiers (335 au moment de la 10.4a).
    assert!(crate::SPRITES.len() >= 300, "{}", crate::SPRITES.len());
}
