//! Un FAUX Monde 3, en test seulement (carte 11.2, AGA : « plus aucun monde 2 écrit
//! en dur »). Trois niveaux d'essai, recopiés après la vraie table avec `monde: 3`
//! et les numéros 1 à 3 : exactement ce que donneraient trois fichiers
//! `assets/levels/monde_3/*.ron`. Le but : vérifier qu'un monde de plus s'ajoute par
//! des DONNÉES, sans retoucher le code. La carte des niveaux l'affiche, ses défis
//! existent, ses records s'enregistrent, ses entités se dessinent (ce dernier point
//! dans les tests de `scene::entites`, qui voient ses fonctions d'images). Et une
//! sauvegarde d'aujourd'hui se recharge à l'identique.
//!
//! Les essais choisis couvrent un thème du ciel (toits), un thème d'orage et un
//! thème du chapitre 2 (le caniveau) : le monde mélange les deux familles, comme
//! pourrait le faire un vrai Monde 3.

use pepin_core::levels::{debut_monde, niveaux_du_monde, GenreNiveau};
use pepin_core::zones::InfosNiveau;

/// Les clés des trois niveaux d'essai du faux Monde 3, dans l'ordre du chemin.
pub const CLES_FAUX_MONDE_3: [&str; 3] = ["essai_chat", "essai_escargot", "essai_eau"];

/// La vraie table (Mondes 1 et 2), puis le faux Monde 3 : trois niveaux normaux.
pub fn table_trois_mondes() -> Vec<InfosNiveau> {
    let mut t = niveaux::niveaux().to_vec();
    for (k, cle) in CLES_FAUX_MONDE_3.iter().enumerate() {
        t.push(InfosNiveau {
            monde: 3,
            numero: k as u8 + 1,
            genre: GenreNiveau::Normal,
            ..*niveaux::par_cle(cle)
        });
    }
    t
}

/// Le faux Monde 3 seul (la tranche de `table_trois_mondes`).
pub fn faux_monde_3(t: &[InfosNiveau]) -> &[InfosNiveau] {
    niveaux_du_monde(t, 3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contenu;
    use crate::defis::{self, Defi};
    use crate::navigation::Navigation;
    use crate::plan::{
        dernier_monde_jouable, etat_etape, monde_jouable, porte_du_monde, EtatEtape, EtatPlan,
        Geometrie, Zones,
    };
    use crate::sauvegarde::{ProgresNiveau, Sauvegarde};

    /// Des zones où rien ne répond : seul le clavier bouge la carte.
    struct SansZone;
    impl Zones for SansZone {
        fn etape_sous(&self, _: &Geometrie, _: f32, _: f32, _: f32) -> Option<usize> {
            None
        }
        fn sur_jouer(&self, _: f32, _: f32) -> bool {
            false
        }
        fn sur_course(&self, _: f32, _: f32) -> bool {
            false
        }
        fn sur_retour(&self, _: f32, _: f32) -> bool {
            false
        }
        fn sur_chapitre_precedent(&self, _: f32, _: f32) -> bool {
            false
        }
        fn sur_chapitre_suivant(&self, _: f32, _: f32) -> bool {
            false
        }
    }

    /// Une sauvegarde où le niveau de clé `cle` est fini (médaille de bronze).
    fn fini(s: &mut Sauvegarde, cle: &str) {
        s.niveaux.insert(
            cle.to_string(),
            ProgresNiveau {
                atteint: true,
                medaille: 1,
                ..ProgresNiveau::default()
            },
        );
    }

    /// La porte du Monde 3 se lit dans la table : le dernier niveau normal du Monde 2.
    /// Fermée sans lui, ouverte avec : le Monde 3 est alors jouable, sa première étape
    /// ouverte, les deux suivantes verrouillées ; JOUER y reprend.
    #[test]
    fn la_porte_du_monde_3_est_la_fin_du_monde_2() {
        let t = table_trois_mondes();
        let porte = porte_du_monde(&t, 3).expect("le Monde 2 a des niveaux normaux");
        assert_eq!(porte.monde, 2);
        assert_eq!(porte.genre, GenreNiveau::Normal);
        let derniere_normale = niveaux_du_monde(&t, 2)
            .iter()
            .rfind(|n| n.genre == GenreNiveau::Normal)
            .unwrap();
        assert_eq!(porte.cle, derniere_normale.cle);

        let mut s = Sauvegarde::default();
        fini(&mut s, "poursuite");
        assert!(monde_jouable(&t, &s, 2));
        assert!(!monde_jouable(&t, &s, 3), "Monde 2 pas fini");
        assert_eq!(dernier_monde_jouable(&t, &s), 2);
        let d3 = debut_monde(&t, 3).unwrap();
        assert_eq!(etat_etape(&t, &s, d3), EtatEtape::Verrouillee);

        fini(&mut s, porte.cle);
        assert!(monde_jouable(&t, &s, 3));
        assert_eq!(dernier_monde_jouable(&t, &s), 3);
        assert_eq!(etat_etape(&t, &s, d3), EtatEtape::Ouverte);
        assert_eq!(etat_etape(&t, &s, d3 + 1), EtatEtape::Verrouillee);
        assert_eq!(etat_etape(&t, &s, d3 + 2), EtatEtape::Verrouillee);
        // Le premier niveau du Monde 3 fini ouvre le deuxième.
        fini(&mut s, CLES_FAUX_MONDE_3[0]);
        assert_eq!(etat_etape(&t, &s, d3), EtatEtape::Finie);
        assert_eq!(etat_etape(&t, &s, d3 + 1), EtatEtape::Ouverte);
    }

    /// La carte des niveaux l'affiche : on y monte depuis le bout du Monde 2 (Haut),
    /// sa géométrie a ses trois étapes, la flèche de chapitre y mène, et Bas ramène
    /// au Monde 2. Son étiquette porte le chapitre 3, son fond suit ses thèmes.
    #[test]
    fn la_carte_des_niveaux_affiche_le_monde_3() {
        let t = table_trois_mondes();
        let mut s = Sauvegarde::default();
        fini(&mut s, "poursuite");
        fini(&mut s, porte_du_monde(&t, 3).unwrap().cle);
        let d3 = debut_monde(&t, 3).unwrap();
        let fin_monde_2 = d3 - 1;

        let mut e = EtatPlan::new(&t, fin_monde_2);
        assert_eq!(e.monde, 2);
        assert_eq!(e.chapitre_voisin(true, &t, &s), Some(3));
        let haut = Navigation {
            haut: true,
            ..Navigation::default()
        };
        // Haut sur la dernière étape du Monde 2 : la première du Monde 3.
        e.reagir(&haut, &t, &s, &SansZone);
        assert_eq!((e.monde, e.debut_monde, e.selection), (3, d3, d3));
        assert_eq!(e.geometrie, Geometrie::new(faux_monde_3(&t)));
        assert_eq!(e.geometrie.etapes.len(), 3);
        assert_eq!(e.chapitre_voisin(true, &t, &s), None, "pas de Monde 4");
        assert_eq!(e.chapitre_voisin(false, &t, &s), Some(2));
        let bas = Navigation {
            bas: true,
            ..Navigation::default()
        };
        e.reagir(&bas, &t, &s, &SansZone);
        assert_eq!((e.monde, e.selection), (2, fin_monde_2));

        // Étiquettes : « CHAPITRE 3 · NIVEAU 2/3 » sur la carte de titre, sans le
        // chapitre dans le panneau de la carte.
        let n2 = &faux_monde_3(&t)[1];
        assert_eq!(contenu::etiquette_niveau(n2, 3), "CHAPITRE 3 · NIVEAU 2/3");
        assert_eq!(contenu::etiquette_dans_le_chapitre(n2, 3), "NIVEAU 2/3");
        // Son fond de carte (`fond_plan::fond_en_descente`) est vérifié dans les tests
        // de `ecrans::fond_plan`. Son titre de chapitre n'est pas encore écrit : le
        // bandeau garde le titre générique (`ecrans::plan::dessiner`,
        // `unwrap_or(TITRE_PLAN)`).
        assert_eq!(contenu::titre_chapitre(3), None);
    }

    /// Ses défis existent (les trois de chaque niveau, chrono compris) ; ses bonus de
    /// fin ont leurs clés à lui et se créditent une fois.
    #[test]
    fn les_defis_et_bonus_du_monde_3_existent() {
        let t = table_trois_mondes();
        for n in faux_monde_3(&t) {
            let d = defis::defis_du_niveau(n);
            assert_eq!(d[0], Defi::SansMourir);
            assert_eq!(d[2], Defi::Chrono(u32::from(n.duree_cible_s)));
            assert!(n.duree_cible_s > 0, "{}", n.cle);
        }
        let mut s = Sauvegarde::default();
        let mut b = defis::bonus_fin_du_monde(3, true, true);
        assert_eq!(
            b.map(|l| l.cle_sauvegarde()),
            [
                "monde3_fin",
                "monde3_sans_game_over",
                "monde3_toutes_les_gouttes"
            ]
        );
        assert!(defis::crediter_bonus_fin(&mut s, &mut b) > 0);
        assert_eq!(
            defis::crediter_bonus_fin(&mut s, &mut defis::bonus_fin_du_monde(3, true, true)),
            0
        );
        // Le Monde 2 n'en est pas touché : ses bonus restent à gagner.
        assert!(
            defis::crediter_bonus_fin(&mut s, &mut defis::bonus_fin_du_monde(2, true, true)) > 0
        );
    }

    /// Ses records s'enregistrent (`partie.monde3`), à part du Monde 1 et du Monde 2,
    /// et se relisent à l'identique avec la progression de ses niveaux.
    #[test]
    fn les_records_du_monde_3_s_enregistrent() {
        let t = table_trois_mondes();
        let mut s = Sauvegarde::default();
        for n in faux_monde_3(&t) {
            fini(&mut s, n.cle);
        }
        assert!(s.noter_partie_monde(3, 1234, 5678));
        let texte = s.vers_texte();
        assert!(texte.contains("partie.monde3=parties:1 score:1234 temps:5678\n"));
        assert!(!texte.contains("partie.monde2"));
        assert!(texte.contains("niveau.essai_eau=atteint:1"));
        let lu = Sauvegarde::depuis_texte(&texte);
        assert_eq!(lu, s);
        assert_eq!(lu.records_partie(3).meilleur_score, 1234);
        assert_eq!(
            (lu.parties_finies, lu.meilleur_score),
            (0, 0),
            "Monde 1 intact"
        );
    }

    /// Une sauvegarde d'AUJOURD'HUI (Monde 1 et Monde 2 joués, records et bonus du
    /// Monde 2 compris, écrite avant la carte 11.2) se recharge à l'identique : relue
    /// puis réécrite, le texte ne change pas d'un octet. Le format ne change pas.
    #[test]
    fn une_sauvegarde_actuelle_se_recharge_a_l_identique() {
        // Les lignes du Monde 2 telles que les écrivait le jeu avant la carte 11.2
        // (champ `partie_monde2`, constantes `BONUS_*_MONDE2`), relues dans les
        // nouvelles structures.
        let avant = "version=3\n\
                     bonus_credites=fin,monde2_fin,monde2_toutes_les_gouttes\n\
                     partie.monde2=parties:3 score:700 temps:4000\n";
        let lu = Sauvegarde::depuis_texte(avant);
        assert_eq!(
            lu.records_partie(2),
            crate::sauvegarde::RecordsPartie {
                parties_finies: 3,
                meilleur_score: 700,
                meilleur_temps: Some(4000)
            }
        );
        assert_eq!(lu.parties_mondes.len(), 1, "aucun autre monde inventé");
        let ecrit = lu.vers_texte();
        assert!(ecrit.contains("bonus_credites=fin,monde2_fin,monde2_toutes_les_gouttes\n"));
        assert!(ecrit.ends_with("partie.monde2=parties:3 score:700 temps:4000\n"));
        // Les bonus du Monde 2 déjà crédités restent crédités (mêmes clés qu'avant).
        let mut s2 = lu.clone();
        let mut b = defis::bonus_fin_du_monde(2, false, true);
        defis::crediter_bonus_fin(&mut s2, &mut b);
        assert!(b[0].deja && b[2].deja);

        // Une sauvegarde complète, construite comme le jeu le fait : l'aller-retour
        // texte → sauvegarde → texte est exact, à l'octet.
        let mut s = Sauvegarde::depuis_texte(
            "version=3\ngouttes_en_poche=120\ngouttes_total_gagnees=340\n\
             meilleur_score=8100\nparties_finies=2\nmeilleur_temps=40000\n",
        );
        fini(&mut s, "toits");
        fini(&mut s, "poursuite");
        fini(&mut s, "caniveau");
        s.crediter_bonus("fin");
        s.crediter_bonus("monde2_fin");
        s.noter_partie_monde(2, 900, 5000);
        let ecrit = s.vers_texte();
        let relu = Sauvegarde::depuis_texte(&ecrit);
        assert_eq!(relu, s);
        assert_eq!(relu.vers_texte(), ecrit, "un octet a changé");
    }
}
