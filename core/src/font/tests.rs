//! Tests de la police bitmap : couverture, accents, largeur, rendu.

use super::*;

/// Phrase de recette du jeu : doit s'afficher entièrement, sans repli.
const PHRASE_TEST: &str = "Où êtes-vous allé, Pépin ? Œuvre, façade, été.";

/// Rend un texte en art ASCII (utile pour vérifier les formes à l'œil).
fn rendu_ascii(texte: &str) -> Vec<String> {
    let pixels = pixels_texte(texte);
    (0..HAUTEUR_GLYPHE)
        .map(|y| {
            (0..largeur_texte(texte, 1))
                .map(|x| if pixels.contains(&(x, y)) { '#' } else { '.' })
                .collect()
        })
        .collect()
}

#[test]
fn ascii_complet_present() {
    for c in (0x20u8..0x7F).map(char::from) {
        assert!(glyphe(c).is_some(), "ASCII manquant : {c:?}");
    }
}

#[test]
fn tous_les_caracteres_francais_sont_presents() {
    let francais = "éèêëàâäçùûüôöîïÿœÉÈÊËÀÂÄÇÙÛÜÔÖÎÏŒ«»’…–—°€";
    for c in francais.chars() {
        assert!(glyphe(c).is_some(), "caractère français manquant : {c:?}");
    }
}

#[test]
fn espaces_insecables_presentes_et_vides() {
    for c in [' ', '\u{00A0}', '\u{202F}'] {
        assert_eq!(glyphe(c), Some(&[0u8; 7]));
    }
}

#[test]
fn la_phrase_de_recette_n_utilise_aucun_repli() {
    for c in PHRASE_TEST.chars() {
        assert!(glyphe(c).is_some(), "repli inattendu pour {c:?}");
    }
}

#[test]
fn caractere_inconnu_donne_none_puis_le_repli() {
    assert!(glyphe('☃').is_none());
    assert_eq!(glyphe_ou_inconnu('☃'), &GLYPHE_INCONNU);
    assert_eq!(glyphe_ou_inconnu('A'), glyphe('A').unwrap());
}

#[test]
fn pas_de_doublon_dans_la_table() {
    let mut vus = std::collections::HashSet::new();
    for bloc in blocs() {
        for (c, _) in bloc {
            assert!(vus.insert(*c), "caractère défini deux fois : {c:?}");
        }
    }
}

#[test]
fn les_glyphes_tiennent_sur_cinq_bits() {
    for bloc in blocs() {
        for (c, dessin) in bloc {
            assert!(dessin.iter().all(|r| *r < 32), "rangée > 5 bits pour {c:?}");
        }
    }
}

#[test]
fn les_glyphes_visibles_ne_sont_pas_vides() {
    for bloc in blocs() {
        for (c, dessin) in bloc {
            let invisible = c.is_whitespace();
            assert_eq!(dessin.iter().all(|r| *r == 0), invisible, "glyphe {c:?}");
        }
    }
}

#[test]
fn les_lettres_accentuees_sont_distinctes_entre_elles_et_de_la_base() {
    let groupes = [
        "eéèêë",
        "aàâä",
        "iîï",
        "oôö",
        "uùûü",
        "cç",
        "EÉÈÊË",
        "AÀÂÄ",
        "CÇ",
        "OÔÖŒ",
    ];
    for groupe in groupes {
        let dessins: Vec<_> = groupe.chars().map(|c| *glyphe(c).unwrap()).collect();
        for i in 0..dessins.len() {
            for j in i + 1..dessins.len() {
                assert_ne!(dessins[i], dessins[j], "{groupe} : doublon visuel");
            }
        }
    }
}

#[test]
fn les_accents_majuscules_sont_sur_la_premiere_rangee() {
    // Une majuscule accentuée a un accent en rangée 0 et rien d'autre au-dessus.
    for c in "ÉÈÊËÀÂÄÎÏÔÖÙÛÜ".chars() {
        let d = glyphe(c).unwrap();
        assert_ne!(d[0], 0, "accent manquant pour {c:?}");
        assert!(
            d[0].count_ones() <= 3,
            "rangée d'accent trop chargée pour {c:?}"
        );
    }
}

#[test]
fn pixel_allume_lit_le_bit_de_gauche_en_premier() {
    let d = glyphe('L').unwrap(); // colonne de gauche pleine
    assert!(pixel_allume(d, 0, 0));
    assert!(!pixel_allume(d, 4, 0));
    assert!(pixel_allume(d, 4, 6)); // base horizontale
    assert!(!pixel_allume(d, 5, 0)); // hors glyphe
    assert!(!pixel_allume(d, 0, 7));
}

#[test]
fn largeur_du_texte() {
    assert_eq!(largeur_texte("", 3), 0);
    assert_eq!(largeur_texte("A", 1), 5);
    assert_eq!(largeur_texte("AB", 1), 11); // 5 + 1 + 5
    assert_eq!(largeur_texte("ABC", 2), 34); // (15 + 2) * 2
                                             // Les caractères accentués comptent pour un seul glyphe (pas pour leurs octets UTF-8).
    assert_eq!(largeur_texte("éœ", 1), 11);
    assert_eq!(
        largeur_texte(PHRASE_TEST, 1),
        PHRASE_TEST.chars().count() as u32 * 6 - 1
    );
    assert_eq!(hauteur_texte(3), 21);
}

#[test]
fn pixels_texte_reste_dans_la_boite() {
    let l = largeur_texte(PHRASE_TEST, 1);
    let pixels = pixels_texte(PHRASE_TEST);
    assert!(!pixels.is_empty());
    assert!(pixels.iter().all(|&(x, y)| x < l && y < HAUTEUR_GLYPHE));
}

#[test]
fn rendu_ascii_d_un_a_accent_aigu() {
    // Hauteur d'x de 4 rangées (1-4) : accent en rangée 0, corps du « e » en 1-4.
    let attendu = [
        "...#.", ".###.", "#####", "#....", ".###.", ".....", ".....",
    ];
    assert_eq!(rendu_ascii("é"), attendu);
}

#[test]
fn affichage_visuel_de_la_phrase_de_recette() {
    // `cargo test -p core -- --nocapture affichage_visuel` pour voir le rendu.
    for ligne in rendu_ascii(PHRASE_TEST) {
        println!("{ligne}");
    }
    assert_eq!(rendu_ascii(PHRASE_TEST).len(), 7);
}

/// Les cinq lettres à jambage.
const A_JAMBAGE: [char; 5] = ['g', 'p', 'q', 'y', 'j'];

#[test]
fn les_lettres_a_jambage_sont_distinctes_entre_elles_et_de_o_a_e() {
    for (i, a) in A_JAMBAGE.iter().enumerate() {
        for b in &A_JAMBAGE[i + 1..] {
            assert_ne!(glyphe(*a), glyphe(*b), "{a} et {b} sont identiques");
        }
        for voyelle in ['o', 'a', 'e'] {
            assert_ne!(
                glyphe(*a),
                glyphe(voyelle),
                "{a} et {voyelle} sont identiques"
            );
        }
    }
}

#[test]
fn les_lettres_a_jambage_restent_distinctes_a_l_echelle_1_et_3() {
    // Le rendu à l'échelle n est un simple agrandissement des pixels : deux
    // dessins différents le restent. On le vérifie quand même sur les pixels.
    for echelle in [1u32, 3] {
        let rendu = |c: char| {
            let mut v: Vec<(u32, u32)> = pixels_texte(&c.to_string())
                .into_iter()
                .flat_map(|(x, y)| {
                    (0..echelle).flat_map(move |dx| {
                        (0..echelle).map(move |dy| (x * echelle + dx, y * echelle + dy))
                    })
                })
                .collect();
            v.sort_unstable();
            v
        };
        let lettres = ['g', 'p', 'q', 'y', 'j', 'o', 'a', 'e'];
        for (i, a) in lettres.iter().enumerate() {
            for b in &lettres[i + 1..] {
                assert_ne!(rendu(*a), rendu(*b), "échelle {echelle} : {a} = {b}");
            }
        }
    }
}

#[test]
fn les_lettres_a_jambage_ont_un_pixel_sur_la_derniere_rangee() {
    for c in A_JAMBAGE {
        let d = glyphe(c).unwrap();
        assert_ne!(d[6], 0, "{c} n'a aucun pixel sur la dernière rangée");
        // Le jambage fait 2 rangées : la rangée 5 est allumée aussi.
        assert_ne!(d[5], 0, "{c} : jambage trop court");
    }
}

#[test]
fn les_minuscules_sans_jambage_ne_descendent_pas_sous_la_ligne_de_base() {
    for c in 'a'..='z' {
        if A_JAMBAGE.contains(&c) {
            continue;
        }
        let d = glyphe(c).unwrap();
        assert_eq!((d[5], d[6]), (0, 0), "{c} déborde sous la ligne de base");
    }
}

#[test]
fn les_minuscules_ont_la_meme_hauteur_d_x() {
    // Les lettres sans hampe ni jambage occupent les rangées 1-4, sans toucher la 0.
    for c in "acemnorsuvwxz".chars() {
        let d = glyphe(c).unwrap();
        assert_eq!(d[0], 0, "{c} touche la rangée 0");
        assert_ne!(d[1], 0, "{c} ne commence pas à la rangée 1");
        assert_ne!(d[4], 0, "{c} n'atteint pas la ligne de base (rangée 4)");
    }
}
