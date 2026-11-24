//! Carte 9.32 (« La marée », 7e niveau du Monde 2, ajoutée entre le port et la
//! Pieuvre par son seul fichier `.ron`) : la sauvegarde d'un joueur qui avait déjà
//! battu la Pieuvre. Test seulement.
//!
//! La sauvegarde range la progression PAR CLÉ de niveau (carte 7.9) : un niveau
//! inséré dans la table ne décale rien. Le joueur garde son chapitre fini (ses sept
//! niveaux, sa partie du Monde 2, la porte du monde suivant), et la marée lui
//! apparaît comme une étape OUVERTE et jamais jouée (`deja_depasse` : il est allé
//! au-delà, elle n'est donc pas cadenassée).

#[cfg(test)]
mod tests {
    use crate::faux_monde_3::table_trois_mondes;
    use crate::plan::{etat_etape, monde_jouable, porte_du_monde, EtatEtape, EtatPlan};
    use crate::sauvegarde::{ProgresNiveau, Sauvegarde};
    use pepin_core::levels::debut_monde;

    /// Les niveaux du Monde 2 d'avant la carte 9.32, dans l'ordre du chemin.
    const MONDE_2_AVANT: [&str; 7] = [
        "caniveau", "egouts", "cascade", "grotte", "riviere", "port", "pieuvre",
    ];

    /// Le texte de la sauvegarde d'un joueur qui a fini le Monde 1 jusqu'à sa porte
    /// et les sept niveaux du Monde 2 d'avant la marée (gouttes, médailles, temps),
    /// et une partie complète du Monde 2.
    fn texte_du_joueur() -> String {
        let mut s = Sauvegarde::default();
        for (k, cle) in ["poursuite"].iter().chain(MONDE_2_AVANT.iter()).enumerate() {
            s.niveaux.insert(
                cle.to_string(),
                ProgresNiveau {
                    atteint: true,
                    gouttes: [true, k % 2 == 0, true],
                    defis: [true, false, false],
                    medaille: 1,
                    meilleur_temps: Some(3_000 + 97 * k as u32),
                    ..ProgresNiveau::default()
                },
            );
        }
        s.noter_partie_monde(2, 900, 5_000);
        s.vers_texte()
    }

    #[test]
    fn la_maree_est_entre_le_port_et_la_pieuvre() {
        let t = niveaux::niveaux();
        let i = t.iter().position(|n| n.cle == "maree").expect("maree");
        assert_eq!((t[i].monde, t[i].numero), (2, 7));
        assert_eq!(t[i - 1].cle, "port");
        assert_eq!(t[i + 1].cle, "pieuvre");
        assert_eq!(t[i + 1].numero, 8);
    }

    #[test]
    fn qui_a_battu_la_pieuvre_garde_son_chapitre_et_trouve_la_maree_non_jouee() {
        let texte = texte_du_joueur();
        assert!(
            !texte.contains("niveau.maree"),
            "la sauvegarde d'avant ne connaît pas la marée"
        );
        let s = Sauvegarde::depuis_texte(&texte);
        // Relue puis réécrite : rien ne change (aucune ligne ajoutée pour la marée).
        assert_eq!(s.vers_texte(), texte);

        let t = niveaux::niveaux();
        let i = |cle: &str| t.iter().position(|n| n.cle == cle).unwrap();
        // Le chapitre reste fini : ses sept niveaux, sa partie, le Monde 2 ouvert.
        for cle in MONDE_2_AVANT {
            assert_eq!(etat_etape(t, &s, i(cle)), EtatEtape::Finie, "{cle}");
        }
        assert!(monde_jouable(t, &s, 2));
        assert_eq!(s.records_partie(2).parties_finies, 1);
        // La marée : ouverte (il est allé plus loin), jamais jouée.
        assert_eq!(etat_etape(t, &s, i("maree")), EtatEtape::Ouverte);
        assert_eq!(s.progres("maree"), ProgresNiveau::default());
        // La carte des niveaux le remet au bout du chemin : la Pieuvre.
        assert_eq!(EtatPlan::ou_on_en_est_du_monde(t, &s, 2), i("pieuvre"));

        // La porte d'un monde suivant (le dernier niveau normal du Monde 2) reste la
        // Pieuvre, qu'il a battue : la marée ne referme rien derrière elle.
        let t3 = table_trois_mondes();
        assert_eq!(porte_du_monde(&t3, 3).unwrap().cle, "pieuvre");
        assert!(monde_jouable(&t3, &s, 3));
        assert_eq!(
            etat_etape(&t3, &s, debut_monde(&t3, 3).unwrap()),
            EtatEtape::Ouverte
        );
    }
}
