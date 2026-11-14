//! Effets visuels déclenchés par ce qui s'est passé pendant un pas de simulation :
//! éclaboussures, poussière, étincelles, vent visible, secousse de mort.

use pepin_core::aabb::TAILLE_TUILE;
use pepin_core::entities::contacts::Contact;
use pepin_core::entities::Entite;
use pepin_core::palette::{BLANC_CASSE, CORAIL_CLAIR, OR, OR_PALE, PLUIE, PLUIE_SOMBRE};
use pepin_core::LARGEUR_LOGIQUE;

use crate::decor::ambiance;
use crate::monde::Monde;
use crate::particules::Particules;
use crate::scene::borner_mort;
use crate::secousse::Secousse;

/// Tuiles de zone de vent latéral par traînée qui naît (tous les 3 pas).
const TUILES_PAR_TRAINEE: f32 = 16.0;

/// Traînées qui naissent dans une zone de vent latéral de `largeur` x `hauteur` px
/// (tous les 3 pas) : une pour `TUILES_PAR_TRAINEE` tuiles, au moins une. Pure.
fn trainees_par_zone(largeur: f32, hauteur: f32) -> usize {
    let tuiles = largeur * hauteur / (TAILLE_TUILE * TAILLE_TUILE);
    ((tuiles / TUILES_PAR_TRAINEE).round() as usize).max(1)
}

/// Effets visuels déclenchés par ce qui s'est passé pendant un pas.
/// `(cam_x, cam_y)` : coin haut-gauche de la zone visible, pour n'animer que le vent qu'on voit
/// et garder les éclats de mort dans le cadre.
pub fn effets(
    monde: &Monde,
    particules: &mut Particules,
    secousse: &mut Secousse,
    cam_x: f32,
    cam_y: f32,
) {
    let (x, y) = (monde.joueur.boite.centre().0, monde.joueur.boite.bas());
    let sens = if monde.joueur.regarde_droite {
        1.0
    } else {
        -1.0
    };
    if monde.evenements.atterrissage {
        particules.eclabousser(x, y, 7, PLUIE, 1.6);
        // Poussière des deux côtés des pieds.
        particules.poussiere(x - 3.0, y - 1.0, 3, BLANC_CASSE, -1.0);
        particules.poussiere(x + 3.0, y - 1.0, 3, BLANC_CASSE, 1.0);
    }
    if monde.evenements.saut {
        particules.eclabousser(x, y, 4, PLUIE_SOMBRE, 1.2);
        particules.poussiere(x, y - 1.0, 3, BLANC_CASSE, -sens);
    }
    // En courant : un filet d'eau et un peu de poussière derrière les pieds.
    if monde.joueur.au_sol && monde.joueur.vitesse.x.abs() > 8.0 {
        if monde.pas.is_multiple_of(6) {
            particules.eclabousser(x, y, 1, PLUIE_SOMBRE, 0.8);
        }
        if monde.pas.is_multiple_of(5) {
            particules.poussiere(x - sens * 4.0, y - 1.0, 1, BLANC_CASSE, -sens);
        }
    }
    // Le vent des courants ascendants devient visible (brins qui montent).
    if monde.pas.is_multiple_of(3) {
        for (k, c) in monde.niveau.courants.iter().enumerate() {
            let z = c.zone;
            if z.x + z.w < cam_x || z.x > cam_x + LARGEUR_LOGIQUE as f32 {
                continue;
            }
            let decal = (monde.pas as usize * 7 + k * 13) as f32 % z.w.max(1.0);
            // Le brin naît à une hauteur tirée sur toute la zone (et reste dedans) : la colonne
            // se voit de bas en haut, pas seulement sous les pieds de Pépin.
            particules.souffle(z.x + decal, z.y, z.bas(), BLANC_CASSE);
        }
        // Le courant d'un ventilateur se voit comme un courant du niveau, mais
        // SEULEMENT tant que l'hélice souffle : aucun brin ne naît pendant l'arrêt
        // ni pendant l'annonce, et la vie de chaque brin est bornée au souffle qui
        // reste (aucun ne survit à l'arrêt). L'indice `k` continue celui des
        // courants du niveau : les brins des colonnes voisines ne se calquent pas.
        // Le courant du Grand Nuage essoufflé se voit de la même façon, borné à
        // l'essoufflement qui reste.
        let nb_courants = monde.niveau.courants.len();
        let ventilateurs = monde.entites.iter().filter_map(|e| match e {
            Entite::Ventilateur(v) => v.courant_actif().map(|c| (c.zone, v.souffle_restant())),
            Entite::Nuage(g) => g
                .courant_actif()
                .map(|c| (c.zone, g.essoufflement_restant())),
            _ => None,
        });
        for (k, (z, restant)) in ventilateurs.enumerate() {
            if z.x + z.w < cam_x || z.x > cam_x + LARGEUR_LOGIQUE as f32 {
                continue;
            }
            let decal = (monde.pas as usize * 7 + (nb_courants + k) * 13) as f32 % z.w.max(1.0);
            // Un brin de vie `v`, né à ce pas, est dessiné après ce pas et les
            // `v - 2` suivants (chaque pas lui retire une vie, il disparaît à 0) ;
            // l'hélice souffle encore ce pas et les `restant - 1` suivants. Avec
            // `v <= restant + 1`, le dernier brin s'efface pile à la première
            // image où l'hélice est arrêtée.
            let vie_max = (restant + 1).min(u8::MAX as u16) as u8;
            particules.souffle_borne(z.x + decal, z.y, z.bas(), BLANC_CASSE, vie_max);
        }
        // Le vent latéral devient visible aussi (traînées horizontales qui
        // traversent la zone, dans son sens, tête claire à l'avant), aux couleurs de
        // l'ambiance du niveau et en nombre proportionnel à la surface de la zone
        // (carte 5.15 : une seule traînée tous les 3 pas se perdait dans une grande
        // zone).
        let couleurs = ambiance(monde.infos.theme).vent;
        for v in &monde.niveau.vents {
            let z = v.zone;
            if z.x + z.w < cam_x || z.x > cam_x + LARGEUR_LOGIQUE as f32 {
                continue;
            }
            for _ in 0..trainees_par_zone(z.w, z.h) {
                particules.souffle_lateral(
                    z.x,
                    z.x + z.w,
                    z.y,
                    z.bas(),
                    v.sens,
                    couleurs.traine,
                    couleurs.tete,
                );
            }
        }
    }
    for c in &monde.contacts {
        match c {
            Contact::Goutte(_) => {
                particules.eclabousser(x, y - 6.0, 10, OR, 2.0);
                particules.etincelles(x, y - 8.0, 8, OR_PALE);
            }
            Contact::Ecrase => particules.eclabousser(x, y, 10, PLUIE, 2.0),
            // Coup sur la tête du Grand Nuage : gerbe de pluie, étincelles d'or et
            // l'écran qui tremble.
            Contact::Nuage => {
                particules.eclabousser(x, y, 12, PLUIE, 2.4);
                particules.etincelles(x, y, 10, OR_PALE);
                secousse.declencher(2, 10);
            }
            Contact::Rebond | Contact::Auvent => particules.eclabousser(x, y, 6, CORAIL_CLAIR, 2.0),
            Contact::Lanterne(..) => particules.etincelles(x, y - 8.0, 10, OR),
            Contact::Mort(_) => {
                // Une chute finit sous l'écran : les éclats naissent dans le cadre.
                let (mx, my) = borner_mort(monde.pos_mort, (cam_x, cam_y));
                particules.eclabousser(mx + 5.0, my + 7.0, 14, CORAIL_CLAIR, 2.4);
                secousse.declencher(2, 14);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pepin_core::entities::ventilateur::Ventilateur;
    use pepin_core::player::Entrees;

    use crate::monde::INDICE_ESSAI_VENTILATEUR;

    fn le_ventilateur(monde: &Monde) -> &Ventilateur {
        monde
            .entites
            .iter()
            .find_map(|e| match e {
                Entite::Ventilateur(v) => Some(v),
                _ => None,
            })
            .expect("le niveau d'essai a un ventilateur")
    }

    /// Carte 5.6b : le courant du ventilateur se voit (brins qui montent dans sa
    /// colonne) pendant le souffle, et RIEN ne reste à l'écran pendant l'arrêt et
    /// l'annonce. On rejoue l'ordre de `Application::pas` : le monde avance, les
    /// effets naissent, les particules vieillissent, puis l'image est dessinée.
    #[test]
    fn le_courant_du_ventilateur_se_voit_pendant_le_souffle_seulement() {
        let mut monde = Monde::new(INDICE_ESSAI_VENTILATEUR);
        assert!(
            monde.niveau.courants.is_empty(),
            "aucun courant `v` dans l'essai : les brins viennent du ventilateur"
        );
        let mut particules = Particules::new();
        let mut secousse = Secousse::new(false);
        // Pépin reste au départ (à gauche de la case) : seul le ventilateur souffle.
        let rien = Entrees::default();
        let mut souffles = 0;
        let mut images_de_souffle = 0;
        let mut images_avec_brins = 0;
        let (mut en_haut, mut en_bas) = (false, false);
        let mut etait_en_souffle = false;
        // Trois périodes complètes (4 s chacune) et un peu plus.
        for _ in 0..3 * 240 + 30 {
            monde.avancer(&rien);
            effets(&monde, &mut particules, &mut secousse, 0.0, 0.0);
            particules.actualiser();
            let v = le_ventilateur(&monde);
            let z = v.zone;
            let brins = particules.brins_de_souffle();
            if !v.souffle() {
                assert!(
                    brins.is_empty(),
                    "pas {} ({:?}) : {} brin(s) de vent survivent hors du souffle",
                    monde.pas,
                    v.phase(),
                    brins.len()
                );
                etait_en_souffle = false;
                continue;
            }
            if !etait_en_souffle {
                souffles += 1;
            }
            etait_en_souffle = true;
            images_de_souffle += 1;
            if !brins.is_empty() {
                images_avec_brins += 1;
            }
            for &(x, y) in &brins {
                // Le brin dérive un peu de côté (0,15 px par pas au plus, soit
                // 6 px sur sa vie) ; il ne dépasse ni le haut ni le bas de la zone.
                assert!(
                    x >= z.x - 8.0 && x <= z.x + z.w + 8.0,
                    "brin hors de la colonne : x = {x}"
                );
                assert!(
                    y >= z.y - 1.0 && y <= z.bas(),
                    "brin hors de la zone : y = {y}"
                );
                en_haut |= y < z.y + z.h / 4.0;
                en_bas |= y > z.bas() - z.h / 4.0;
            }
        }
        assert!(souffles >= 3, "trois souffles vus, pas {souffles}");
        // Les brins apparaissent dès les premiers pas du souffle (un tous les 3 pas).
        assert!(
            images_avec_brins * 10 >= images_de_souffle * 9,
            "brins visibles sur {images_avec_brins} des {images_de_souffle} images de souffle"
        );
        assert!(en_haut && en_bas, "la colonne se voit de bas en haut");
    }

    /// Carte 5.15 : dans la mer de nuages, chaque zone de vent latéral à l'écran se
    /// remplit de traînées, aux couleurs de l'ambiance du niveau, qui avancent dans
    /// le sens de SA zone (au moment fort, trois couches : `>`, `<`, `>`), et
    /// qu'aucune ne déborde de sa zone.
    #[test]
    fn le_vent_lateral_de_la_mer_de_nuages_se_voit_dans_chaque_couche() {
        let indice = pepin_core::levels::NIVEAUX
            .iter()
            .position(|n| n.nom == pepin_core::levels::mer::INFOS.nom)
            .expect("la mer de nuages est dans la table");
        let mut monde = Monde::new(indice);
        let couleurs = ambiance(monde.infos.theme).vent;
        let mut particules = Particules::new();
        let mut secousse = Secousse::new(false);
        let rien = Entrees::default();
        // La caméra sur le moment fort (la tour, colonne 106 : x = 1696).
        let cam_x = 1696.0;
        for _ in 0..90 {
            monde.avancer(&rien);
            effets(&monde, &mut particules, &mut secousse, cam_x, 0.0);
            particules.actualiser();
        }
        let trainees = particules.trainees();
        let vents = &monde.niveau.vents;
        let mut par_zone = vec![0; vents.len()];
        for &(x, y, vx, traine, tete) in &trainees {
            assert_eq!((traine, tete), (couleurs.traine, couleurs.tete));
            let k = vents
                .iter()
                .position(|v| {
                    let z = v.zone;
                    x >= z.x - 0.01
                        && x + f32::from(crate::particules::LONGUEUR_TRAINEE) <= z.x + z.w + 0.01
                        && y >= z.y
                        && y <= z.bas()
                })
                .unwrap_or_else(|| panic!("traînée hors de toute zone : ({x}, {y})"));
            assert_eq!(vx.signum(), f32::from(vents[k].sens), "({x}, {y})");
            par_zone[k] += 1;
        }
        // Les trois couches du moment fort (à l'écran) ont chacune leurs traînées.
        let couches: Vec<usize> = (0..vents.len())
            .filter(|&k| {
                let z = vents[k].zone;
                z.x < cam_x + LARGEUR_LOGIQUE as f32 && z.x + z.w > cam_x
            })
            .collect();
        assert_eq!(couches.len(), 3, "{vents:?}");
        for k in couches {
            assert!(
                par_zone[k] >= 8,
                "zone {:?} : {} traînée(s)",
                vents[k].zone,
                par_zone[k]
            );
        }
    }

    /// Carte 5.15 : le vent latéral se voit. Dans le niveau d'essai « vent »
    /// (ciel du soleil, où les anciens tirets blancs de 3 px se perdaient), la zone
    /// se remplit de traînées aux couleurs de l'ambiance (queue violette, tête
    /// blanche), qui avancent dans le sens du vent sans déborder de la zone.
    #[test]
    fn le_vent_lateral_se_voit_dans_le_niveau_d_essai() {
        let mut monde = Monde::new(crate::monde::INDICE_ESSAI_VENT);
        let couleurs = ambiance(monde.infos.theme).vent;
        assert_eq!(couleurs.traine, pepin_core::palette::VIOLET_ORAGE);
        let mut particules = Particules::new();
        let mut secousse = Secousse::new(false);
        let rien = Entrees::default();
        for _ in 0..90 {
            monde.avancer(&rien);
            effets(&monde, &mut particules, &mut secousse, 0.0, 0.0);
            particules.actualiser();
        }
        let z = monde.niveau.vents[0].zone;
        let trainees = particules.trainees();
        // Une zone de 12 x 3 tuiles : 2 traînées tous les 3 pas, qui vivent au
        // moins 18 pas (bornées par le bord) : une dizaine à l'écran.
        assert!(trainees.len() >= 10, "{} traînées", trainees.len());
        for &(x, y, vx, traine, tete) in &trainees {
            assert_eq!((traine, tete), (couleurs.traine, couleurs.tete));
            assert!(vx > 0.0, "vent vers la droite");
            assert!(
                x >= z.x - 0.01
                    && x + f32::from(crate::particules::LONGUEUR_TRAINEE) <= z.x + z.w + 0.01
                    && (z.y..=z.bas()).contains(&y),
                "traînée hors de la zone : ({x}, {y})"
            );
        }
    }

    #[test]
    fn plus_la_zone_de_vent_est_grande_plus_il_y_a_de_trainees() {
        let t = TAILLE_TUILE;
        assert_eq!(trainees_par_zone(t, t), 1);
        assert_eq!(trainees_par_zone(13.0 * t, 4.0 * t), 3);
        assert_eq!(trainees_par_zone(20.0 * t, 2.0 * t), 3);
        assert_eq!(trainees_par_zone(17.0 * t, 3.0 * t), 3);
    }
}
