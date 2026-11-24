//! Entités du niveau : ennemis, plateformes, objets, auvents... chacun avec l'image
//! qui correspond à son état (fonctions pures `image_*`, testables sans écran).
//! Ne dessine ni Pépin (`pepin`) ni le Grand Nuage (`nuage`).

use pepin_core::aabb::{Grille, TAILLE_TUILE};
use pepin_core::entities::auvent::HAUT_TOILE;
use pepin_core::entities::ennemi::{EtatEnnemi, PhaseChat, PhasePigeon};
use pepin_core::entities::friable::EtatFriable;
use pepin_core::entities::montgolfiere::{EtatMontgolfiere, HAUT_BALLON};
use pepin_core::entities::{Entite, TypeEnnemi};
use pepin_core::palette::*;
use pepin_core::player::EtatJoueur;

use super::habillage::SOMMET_DU_MAT;
use super::nuage::{annonce_allumee, dessiner_eclair, dessiner_nuage};
use super::tuiles::boite_cachee;
use super::{entier, image, rect};
use crate::decor;
use crate::grele;
use crate::monde::Monde;
use crate::sprites::{ennemis, largeur, objets, Banque};

pub(super) fn dessiner_entites(monde: &Monde, banque: &Banque, alpha: f32, cx: i32, cy: i32) {
    let pas = monde.pas;
    // Parapluie ouvert de Pépin (la grêle se brise dessus), à sa position dessinée.
    let parapluie = (monde.joueur.etat() == EtatJoueur::Plane).then(|| {
        let (px, py) = monde.joueur_interpole(alpha);
        grele::parapluie(px.round(), py.round())
    });
    for (i, entite) in monde.entites.iter().enumerate() {
        let b = monde.boite_interpolee(i, alpha);
        // Derrière un faux mur pas encore révélé (carte 8.5) : rien ne se voit.
        if boite_cachee(monde, &b) {
            continue;
        }
        let (x, y) = entier(&b);
        let (sx, sy) = ((x - cx) as f32, (y - cy) as f32);
        match entite {
            Entite::Goutte(g) => {
                if !g.est_ramassee() {
                    // Petit flottement d'un pixel, en 4 temps. Le dessin (10x10) déborde d'un
                    // pixel de chaque côté de la boîte de ramassage (8x8).
                    let flotte = [0.0, -1.0, -1.0, 0.0][(pas / 15) as usize % 4];
                    banque.dessiner(
                        image(&objets::GOUTTE, pas, 8),
                        sx - 1.0,
                        sy + flotte - 1.0,
                        false,
                    );
                }
            }
            Entite::Lanterne(l) => {
                let nom = if l.est_active() {
                    objets::LANTERNE[1]
                } else {
                    objets::LANTERNE[0]
                };
                banque.dessiner(nom, sx, sy, false);
            }
            // La lettre cachée (carte 8.6), comme la goutte : dessin 10x10 autour de
            // la boîte 8x8, flottement d'un pixel (à contretemps de la goutte).
            Entite::Lettre(l) => {
                if !l.est_prise() {
                    let lettre = pepin_core::lettres::lettre_du_niveau(monde.infos.cle);
                    let flotte = [0.0, 0.0, -1.0, -1.0][(pas / 15) as usize % 4];
                    banque.dessiner(
                        objets::lettre(lettre.unwrap_or('P')),
                        sx - 1.0,
                        sy + flotte - 1.0,
                        false,
                    );
                }
            }
            Entite::Champignon(c) => {
                // Sprite 16x12 : le chapeau commence à la hauteur de la boîte.
                let nom = if c.est_comprime() {
                    objets::CHAMPIGNON[1]
                } else {
                    objets::CHAMPIGNON[0]
                };
                banque.dessiner(nom, sx, sy - 3.0, false);
            }
            Entite::Escargot(e) => {
                // Sprite 16x12 (tourné vers la gauche) posé sur le bas de la boîte
                // 16x10. Sa coquille de champignon dit déjà « je fais rebondir ».
                banque.dessiner(
                    image_escargot(e.est_comprime(), pas),
                    sx,
                    sy - 2.0,
                    e.vers_la_droite,
                );
            }
            Entite::Pointe(p) => {
                // La boîte fait 16 px de large : deux pointes de 8 px.
                let n = (p.boite.w / 8.0) as i32;
                for k in 0..n {
                    banque.dessiner("pointe", sx + (k * 8) as f32, sy, false);
                }
            }
            Entite::Friable(f) => {
                let tremble = match f.etat {
                    EtatFriable::Effritement(_) => [0.0, 1.0, 0.0, -1.0][(pas / 2) as usize % 4],
                    EtatFriable::Absente(_) => continue,
                    EtatFriable::Stable => 0.0,
                };
                // Dalle d'ardoise sur les toits, nuage en altitude (table d'ambiance).
                let anim = if decor::ambiance(monde.infos.theme).friable_nuage {
                    &objets::FRIABLE
                } else {
                    &objets::FRIABLE_ARDOISE
                };
                let nom = if matches!(f.etat, EtatFriable::Effritement(_)) {
                    anim[1]
                } else {
                    anim[0]
                };
                for k in 0..(f.boite.w / 16.0).ceil() as i32 {
                    banque.dessiner(nom, sx + (k * 16) as f32 + tremble, sy, false);
                }
            }
            // Une aiguille d'horloge (cadran et tige dessinés avant, voir
            // `dessiner_cadrans`) ou une plateforme en aller-retour.
            Entite::Mobile(m) => {
                // Au port (carte 9.11) : bateaux et caisse de grue.
                if crate::dessin_port::dessiner_mobile(monde, banque, m.extremites(), sx, sy)
                    || crate::decor_riviere::dessiner_tronc(monde, sx, sy)
                {
                    continue;
                }
                let nom = if m.cercle().is_some() {
                    "aiguille"
                } else {
                    "mobile"
                };
                banque.dessiner(nom, sx, sy, false);
            }
            Entite::Montgolfiere(m) => {
                // Sprite 20x40 : la nacelle (16x8, la boîte `b`) occupe ses 8 dernières
                // rangées, le ballon les 32 du dessus et déborde de 2 px de chaque
                // côté. Brûleur allumé (la flamme vacille, une image sur 4 pas)
                // seulement pendant la MONTÉE : c'est l'indice « Pépin la fait monter ».
                let nom = if m.etat() == EtatMontgolfiere::Montee {
                    objets::MONTGOLFIERE[1 + (pas / 4) as usize % 2]
                } else {
                    objets::MONTGOLFIERE[0]
                };
                banque.dessiner(nom, sx - 2.0, sy - HAUT_BALLON, false);
            }
            Entite::Auvent(a) => {
                // Sprite 16x15 posé `SOMMET_DU_MAT` (1 px) sous le haut de la case :
                // le dessus de la zone de contact (`HAUT_TOILE` sous le haut de la
                // case) tombe au milieu de la pente de la toile.
                let nom = if a.est_enfonce() {
                    objets::AUVENT[1]
                } else {
                    objets::AUVENT[0]
                };
                banque.dessiner(nom, sx, sy - HAUT_TOILE + SOMMET_DU_MAT as f32, false);
            }
            Entite::Ventilateur(v) => {
                // Sprite 16x16 qui couvre toute la case : la casette (socle 16x8)
                // tombe sur le BAS de la boîte `b`, le corps du ventilateur monte au-
                // dessus. On choisit l'image selon la phase : à l'arrêt, à l'annonce
                // (la lampe orange clignote de plus en plus vite, comme l'annonce
                // d'un éclair) ou en souffle (le courant d'air cyan remonte).
                let nom = if v.souffle() {
                    objets::VENTILATEUR[2]
                } else if v.en_annonce() {
                    // Clignotement de la lampe qui ACCÉLÈRE pendant l'annonce
                    // (comme l'éclair) : de ~2,5 Hz au début à ~15 Hz à la fin.
                    if annonce_allumee(v.progression_annonce()) {
                        objets::VENTILATEUR[1]
                    } else {
                        objets::VENTILATEUR[0]
                    }
                } else {
                    objets::VENTILATEUR[0]
                };
                banque.dessiner(nom, sx, sy - 8.0, false);
                // Repère au sol : pendant l'annonce, une barre orange reste allumée
                // au PIED de la casette (là où elle touche le sol) : on voit la
                // COLONNE d'où le souffle va reprendre, comme la barre dorée au
                // pied de l'éclair.
                if v.en_annonce() {
                    rect(sx as i32, sy as i32 + 6, 16, 2, OR);
                }
            }
            // Le Grand Nuage et ses éclairs (son courant : des brins, `effets` ; la
            // déchirure du ciel : `dessiner_dechirure`, derrière les tuiles).
            Entite::Nuage(g) => dessiner_nuage(g, banque, pas, sx, sy, cx, cy),
            // La Pieuvre du port (carte 9.12a) : `dessin_pieuvre.rs`.
            Entite::Pieuvre(p) => crate::dessin_pieuvre::dessiner(p, banque, pas, cx, cy),
            // Les mécaniques d'eau du Monde 2 (carte 9.3) : `eau.rs`.
            Entite::Bulles(_)
            | Entite::Chute(_)
            | Entite::Nenuphar(_)
            | Entite::Stalactite(_)
            | Entite::Maree(_) => crate::eau::dessiner(monde, banque, entite, (sx, sy), cx, cy),
            Entite::Grele(g) => {
                // Un solide sous la colonne ? (Sinon elle tombe dans le vide.)
                let t = TAILLE_TUILE;
                let sol = monde
                    .niveau
                    .est_solide((g.zone.x / t) as i32, (g.zone.bas() / t) as i32);
                let morceaux = grele::disposer(g.zone, pas, sol, parapluie);
                grele::dessiner(&morceaux, banque, cx, cy);
                // Le petit nuage gris d'où elle tombe (carte 8.18), par-dessus.
                grele::dessiner_nuage(g.zone, pas, banque, cx, cy);
            }
            Entite::Eclair(e) => dessiner_eclair(
                &e.zone,
                e.en_annonce(),
                e.progression_annonce(),
                e.frappe(),
                cx,
                cy,
            ),
            Entite::Ennemi(e) => {
                if !e.est_visible() {
                    continue;
                }
                let ecrase = matches!(e.etat, EtatEnnemi::Ecrase(_));
                match e.type_ennemi {
                    TypeEnnemi::Flaque => {
                        let nom = if ecrase {
                            "flaque_ecrasee"
                        } else {
                            image(&ennemis::FLAQUE, pas, 12)
                        };
                        // Sprite 16x10 posé sur le bas de la boîte 12x8, centré.
                        banque.dessiner(nom, sx - 2.0, sy - 2.0, e.vers_la_droite);
                    }
                    TypeEnnemi::Corbeau => {
                        // Le corbeau du dessin regarde à droite. Sprite 18x14 centré
                        // sur la boîte 12x10 (3 px de chaque côté, 2 px en haut et en
                        // bas) : le miroir garde le corps sur la boîte. Au Monde 2
                        // (carte 9.26b), même ennemi dessiné en chauve-souris.
                        banque.dessiner(
                            image(images_corbeau(monde.infos.monde), pas, 10),
                            sx - 3.0,
                            sy - 2.0,
                            !e.vers_la_droite,
                        );
                    }
                    TypeEnnemi::Pigeon => {
                        // Le pigeon du dessin regarde à droite. Sprite 18x14 (carte 6.6)
                        // centré sur la boîte 12x10 (3 px de chaque côté : le miroir le
                        // garde centré), sa dernière rangée (le contour sous les pattes)
                        // sur la dernière rangée de la boîte : 4 px dépassent en haut.
                        let (nom, tremble, retourne) = image_pigeon(e.phase(), ecrase, pas);
                        banque.dessiner(
                            nom,
                            sx - 3.0 + tremble,
                            sy - 4.0,
                            e.vers_la_droite == retourne,
                        );
                    }
                    // Les ennemis d'eau du Monde 2 (carte 9.4) : `dessin_eau`.
                    TypeEnnemi::Grenouille
                    | TypeEnnemi::Poisson
                    | TypeEnnemi::Crabe
                    | TypeEnnemi::Rat
                    | TypeEnnemi::Mouette => crate::dessin_eau::dessiner(e, banque, pas, sx, sy),
                    TypeEnnemi::Chat => {
                        // Le chat du dessin regarde à gauche. Sprite 16x14 posé sur le
                        // bas de la boîte 14x8, centré (1 px de chaque côté). Il se
                        // tourne vers Pépin quand il est prêt à bondir, et dans le sens
                        // du saut pendant un bond (carte 6.3).
                        let (x, y) = (sx - 1.0, sy - 6.0);
                        let miroir = e.regarde_a_droite();
                        let posture = PostureChat {
                            pret: e.est_pret_a_bondir(),
                            en_vol: e.en_plein_bond(),
                        };
                        let nom = image_chat(e.phase_chat(), ecrase, posture, pas);
                        banque.dessiner(nom, x, y, miroir);
                        // L'indice au-dessus de sa tête : les « z » du sommeil, puis le
                        // « ! » de l'annonce, qui clignote de plus en plus vite.
                        if !ecrase {
                            for (nom, dx, dy) in bulles_chat(e.phase_chat(), pas) {
                                // La tête est à gauche du dessin : miroir, elle passe à droite.
                                let l = largeur(sprite_chat(nom)) as f32;
                                let bx = if miroir { x + 16.0 - dx - l } else { x + dx };
                                banque.dessiner(nom, bx, y + dy, false);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Images du corbeau selon le monde du niveau (carte 9.26b) : un corbeau au
/// Monde 1, une chauve-souris à partir du Monde 2 (grotte, égouts…). Même ennemi,
/// même boîte : seul le dessin change. Fonction pure.
pub(crate) fn images_corbeau(monde: u8) -> &'static [&'static str; 2] {
    if monde >= 2 {
        &ennemis::CHAUVE_SOURIS
    } else {
        &ennemis::CORBEAU
    }
}

/// Image de l'escargot : il marche (2 images de 0,25 s), ou sa coquille est
/// aplatie juste après un rebond. Fonction pure.
pub(crate) fn image_escargot(comprime: bool, pas: u32) -> &'static str {
    if comprime {
        "escargot_rebond"
    } else {
        image(&ennemis::ESCARGOT, pas, 15)
    }
}

/// Posture du chat (carte 6.3), en plus de sa phase : prêt à bondir pendant
/// l'annonce (`Ennemi::est_pret_a_bondir`), en l'air pendant un bond
/// (`Ennemi::en_plein_bond`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PostureChat {
    pub(crate) pret: bool,
    pub(crate) en_vol: bool,
}

/// Image du chat selon sa phase et sa posture. Endormi, il respire (2 images de
/// 0,5 s) ; pendant l'annonce il lève la tête, un œil ouvert, ou, si Pépin est
/// à portée, il se ramasse, prêt à bondir (tête basse, arrière-train levé) ;
/// éveillé, il est assis, hérissé, ou en plein bond, pattes tendues. Fonction pure.
pub(crate) fn image_chat(
    phase: PhaseChat,
    ecrase: bool,
    posture: PostureChat,
    pas: u32,
) -> &'static str {
    if ecrase {
        return "chat_ecrase";
    }
    match phase {
        PhaseChat::Dort(_) => image(&ennemis::CHAT_DORT, pas, 30),
        PhaseChat::Reveil(_) if posture.pret => "chat_pret",
        PhaseChat::Reveil(_) => "chat_reveil",
        PhaseChat::Eveille(_) if posture.en_vol => "chat_bond",
        PhaseChat::Eveille(_) => "chat_eveille",
    }
}

/// Durée (pas) de la montée d'un « z » au-dessus du chat endormi.
const MONTEE_Z_PAS: u32 = 60;

/// Les indices dessinés au-dessus de la tête du chat : `(sprite, dx, dy)`, en px
/// depuis le coin haut-gauche du sprite du chat (tourné vers la gauche).
/// - Il dort : deux « z » montent l'un après l'autre en s'écartant de sa tête
///   (10 px de montée en 1 s, décalés d'une demi-seconde).
/// - Il se réveille : le « ! » doré clignote de plus en plus vite (comme
///   l'annonce d'un éclair), tout le temps de l'annonce.
/// - Éveillé : rien (il est assis, hérissé : son image suffit).
///
/// Fonction pure.
pub(crate) fn bulles_chat(phase: PhaseChat, pas: u32) -> Vec<(&'static str, f32, f32)> {
    match phase {
        PhaseChat::Dort(_) => (0..2)
            .map(|k| {
                let t = (pas + k * MONTEE_Z_PAS / 2) % MONTEE_Z_PAS;
                // Entiers : le « z » reste sur la grille des pixels.
                let montee = t * 10 / MONTEE_Z_PAS;
                ("chat_z", 1.0 - (montee / 2) as f32, -2.0 - montee as f32)
            })
            .collect(),
        PhaseChat::Reveil(n) => {
            let progression = 1.0 - n as f32 / pepin_core::entities::ennemi::REVEIL_PAS as f32;
            if annonce_allumee(progression) {
                vec![("chat_alerte", 2.0, -6.0)]
            } else {
                Vec::new()
            }
        }
        PhaseChat::Eveille(_) => Vec::new(),
    }
}

/// Dessin d'un indice du chat (pour connaître sa largeur).
fn sprite_chat(nom: &str) -> crate::sprites::Image {
    if nom == "chat_alerte" {
        crate::sprites::dessin("chat_alerte")
    } else {
        crate::sprites::dessin("chat_z")
    }
}

/// Pas d'élan au début de la plongée (carte 6.6) : pendant les 8 premiers pas
/// (0,13 s, le quart de la plongée), le pigeon s'élance ailes ouvertes ; ensuite
/// il les relève en flèche pour le reste du piqué.
const ELAN_PIGEON_PAS: u16 = 8;

/// Durée (pas) d'une image du battement d'ailes de la remontée : 8 pas, soit
/// quatre battements complets pendant les 64 pas de la remontée.
const BATTEMENT_PIGEON_PAS: u16 = 8;

/// Image du pigeon selon sa phase : `(nom du sprite, tremblement en px, retourné)`.
/// Perché ou au repos, il hoche la tête ; pendant l'annonce il tremble d'un pixel
/// (comme une plateforme friable qui va céder) ; en plongée, il s'élance ailes
/// ouvertes puis pique ailes en flèche ; pendant la remontée il bat des ailes et
/// il est « retourné » : il revient vers son perchoir, donc il regarde derrière
/// lui. L'image de la plongée et de la remontée suit le compteur de la phase (et
/// non l'horloge `pas`) : chaque piqué commence par l'élan. Fonction pure.
pub(crate) fn image_pigeon(
    phase: PhasePigeon,
    ecrase: bool,
    pas: u32,
) -> (&'static str, f32, bool) {
    if ecrase {
        return ("pigeon_ecrase", 0.0, false);
    }
    match phase {
        PhasePigeon::Perche | PhasePigeon::Repos(_) => {
            (image(&ennemis::PIGEON, pas, 24), 0.0, false)
        }
        PhasePigeon::Annonce(_) => (
            "pigeon_annonce",
            [0.0, 1.0, 0.0, -1.0][(pas / 2) as usize % 4],
            false,
        ),
        PhasePigeon::Plongee(n) => {
            let i = usize::from(n > ELAN_PIGEON_PAS);
            (ennemis::PIGEON_PLONGEE[i], 0.0, false)
        }
        PhasePigeon::Remontee(n) => {
            let i = usize::from(n.saturating_sub(1) / BATTEMENT_PIGEON_PAS % 2);
            (ennemis::PIGEON_REMONTEE[i], 0.0, true)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Corbeau / chauve-souris (carte 9.26b) ---

    /// Le corbeau reste un corbeau au Monde 1 ; à partir du Monde 2 il est
    /// dessiné en chauve-souris, avec le même battement (2 images, même taille).
    #[test]
    fn le_corbeau_devient_une_chauve_souris_au_monde_2() {
        assert_eq!(images_corbeau(1), &ennemis::CORBEAU);
        assert_eq!(images_corbeau(2), &ennemis::CHAUVE_SOURIS);
        assert_eq!(images_corbeau(3), &ennemis::CHAUVE_SOURIS);
    }

    // --- Escargot (carte 5.8) ---

    #[test]
    fn l_escargot_marche_sur_deux_images_et_s_aplatit_au_rebond() {
        let marche: Vec<&str> = (0..60).map(|p| image_escargot(false, p)).collect();
        assert!(marche.contains(&"escargot0") && marche.contains(&"escargot1"));
        assert!(marche
            .iter()
            .all(|n| n.starts_with("escargot") && *n != "escargot_rebond"));
        assert!((0..60).all(|p| image_escargot(true, p) == "escargot_rebond"));
        let noms: Vec<&str> = crate::sprites::tous().iter().map(|(n, _)| *n).collect();
        for n in ["escargot0", "escargot1", "escargot_rebond"] {
            assert!(noms.contains(&n), "{n}");
        }
    }

    // --- Chat endormi (carte 5.9) ---

    #[test]
    fn le_chat_a_une_image_par_phase() {
        use pepin_core::entities::ennemi::{DORT_PAS, EVEILLE_PAS, REVEIL_PAS};
        // Endormi, il respire : les deux images alternent.
        let rien = PostureChat::default();
        let dort: Vec<_> = (0..120)
            .map(|p| image_chat(PhaseChat::Dort(DORT_PAS), false, rien, p))
            .collect();
        assert!(dort.contains(&"chat_dort0") && dort.contains(&"chat_dort1"));
        assert!(dort.iter().all(|n| n.starts_with("chat_dort")));
        assert_eq!(
            image_chat(PhaseChat::Reveil(REVEIL_PAS), false, rien, 3),
            "chat_reveil"
        );
        assert_eq!(
            image_chat(PhaseChat::Eveille(EVEILLE_PAS), false, rien, 3),
            "chat_eveille"
        );
        // Carte 6.3 : prêt à bondir pendant l'annonce, pattes tendues en l'air.
        let pret = PostureChat {
            pret: true,
            en_vol: false,
        };
        let en_vol = PostureChat {
            pret: false,
            en_vol: true,
        };
        assert_eq!(
            image_chat(PhaseChat::Reveil(REVEIL_PAS), false, pret, 3),
            "chat_pret"
        );
        assert_eq!(
            image_chat(PhaseChat::Eveille(40), false, en_vol, 3),
            "chat_bond"
        );
        // Posé au bout du bond : debout, hérissé, comme l'éveil sur place.
        assert_eq!(
            image_chat(PhaseChat::Eveille(40), false, rien, 3),
            "chat_eveille"
        );
        for phase in [
            PhaseChat::Dort(5),
            PhaseChat::Reveil(5),
            PhaseChat::Eveille(5),
        ] {
            for posture in [rien, pret, en_vol] {
                assert_eq!(image_chat(phase, true, posture, 9), "chat_ecrase");
            }
        }
        // Toutes ces images existent dans la banque de sprites.
        let noms: Vec<&str> = crate::sprites::tous().iter().map(|(n, _)| *n).collect();
        for n in [
            "chat_dort0",
            "chat_dort1",
            "chat_reveil",
            "chat_eveille",
            "chat_ecrase",
            "chat_pret",
            "chat_bond",
        ] {
            assert!(noms.contains(&n), "{n}");
        }
    }

    #[test]
    fn les_z_montent_pendant_le_sommeil_et_le_point_d_exclamation_annonce_le_reveil() {
        use pepin_core::entities::ennemi::{EVEILLE_PAS, REVEIL_PAS};
        // Endormi : toujours deux « z », qui montent puis repartent d'en bas.
        let hauteurs: Vec<f32> = (0..MONTEE_Z_PAS)
            .map(|p| {
                let b = bulles_chat(PhaseChat::Dort(100), p);
                assert_eq!(b.len(), 2);
                assert!(b.iter().all(|&(n, _, dy)| n == "chat_z" && dy < 0.0));
                b[0].2
            })
            .collect();
        assert_eq!(hauteurs[0], -2.0);
        assert!(hauteurs.windows(2).all(|w| w[1] <= w[0]), "il monte");
        assert!(hauteurs.last().unwrap() <= &-11.0);
        // Annonce du réveil : le « ! » clignote, de plus en plus vite.
        let allume: Vec<bool> = (1..=REVEIL_PAS)
            .rev()
            .map(|n| !bulles_chat(PhaseChat::Reveil(n), 0).is_empty())
            .collect();
        let changements = |t: &[bool]| t.windows(2).filter(|w| w[0] != w[1]).count();
        let (debut, fin) = allume.split_at(allume.len() / 2);
        assert!(allume[0], "allumé dès le début de l'annonce");
        assert!(changements(fin) > changements(debut), "{allume:?}");
        // Éveillé : aucun indice.
        assert!(bulles_chat(PhaseChat::Eveille(EVEILLE_PAS), 0).is_empty());
    }

    // --- Pigeon (carte 4.4, redessiné carte 6.6) ---

    #[test]
    fn le_pigeon_a_une_image_par_phase() {
        use pepin_core::entities::ennemi::{
            ANNONCE_PIGEON_PAS, PLONGEE_PAS, REMONTEE_PAS, REPOS_PIGEON_PAS,
        };
        // Perché ou au repos : il hoche la tête (2 images), sans trembler.
        let perche: Vec<_> = (0..48)
            .map(|p| image_pigeon(PhasePigeon::Perche, false, p))
            .collect();
        assert!(perche.iter().all(|&(_, t, r)| t == 0.0 && !r));
        assert!(perche.iter().any(|&(n, _, _)| n == "pigeon0"));
        assert!(perche.iter().any(|&(n, _, _)| n == "pigeon1"));
        assert_eq!(
            image_pigeon(PhasePigeon::Repos(REPOS_PIGEON_PAS), false, 0),
            image_pigeon(PhasePigeon::Perche, false, 0)
        );
        // Annonce : il tremble d'un pixel, d'un côté puis de l'autre.
        let tremble: Vec<f32> = (0..8)
            .map(|p| {
                let (nom, t, r) = image_pigeon(PhasePigeon::Annonce(ANNONCE_PIGEON_PAS), false, p);
                assert_eq!((nom, r), ("pigeon_annonce", false));
                t
            })
            .collect();
        assert!(tremble.contains(&1.0) && tremble.contains(&-1.0) && tremble.contains(&0.0));
        // Plongée (carte 6.6) : l'élan ailes ouvertes pendant les 8 premiers pas,
        // puis le piqué ailes en flèche jusqu'au bout ; l'horloge `pas` n'y change rien.
        for n in 1..=PLONGEE_PAS {
            let attendu = if n <= ELAN_PIGEON_PAS {
                "pigeon_plongee0"
            } else {
                "pigeon_plongee1"
            };
            for p in [0, 5, 13] {
                assert_eq!(
                    image_pigeon(PhasePigeon::Plongee(n), false, p),
                    (attendu, 0.0, false),
                    "plongée, pas {n}"
                );
            }
        }
        // Remontée : retourné, il regarde son perchoir, et il bat des ailes (les deux
        // images alternent tous les 8 pas, en commençant ailes levées).
        let remontee: Vec<&str> = (1..=REMONTEE_PAS)
            .map(|n| {
                let (nom, t, r) = image_pigeon(PhasePigeon::Remontee(n), false, 5);
                assert_eq!((t, r), (0.0, true));
                nom
            })
            .collect();
        assert_eq!(remontee[0], "pigeon_remontee0");
        assert_eq!(remontee[8], "pigeon_remontee1");
        assert_eq!(remontee[16], "pigeon_remontee0");
        let changements = remontee.windows(2).filter(|w| w[0] != w[1]).count();
        assert_eq!(changements, 7, "quatre battements pendant la remontée");
        // Écrasé : quelle que soit la phase où on l'a eu.
        for phase in [
            PhasePigeon::Perche,
            PhasePigeon::Plongee(10),
            PhasePigeon::Remontee(1),
        ] {
            assert_eq!(image_pigeon(phase, true, 7), ("pigeon_ecrase", 0.0, false));
        }
    }
}
