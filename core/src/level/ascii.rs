//! Parseur ASCII -> `Niveau`.
//!
//! Étapes : 1) découper les lignes et vérifier le rectangle ; 2) lire chaque
//! caractère (tuile, entité, repère, décor) ; 3) assembler les éléments qui dépendent
//! de la grille complète (éclairs, grêle, courants, plateformes mobiles, appui du décor) ;
//! 4) valider (un départ, une arrivée (ou un Grand Nuage), trois gouttes) ;
//! 5) poser les aiguilles d'horloge, dont le cercle se vérifie sur le niveau assemblé.

use std::collections::BTreeMap;

use super::{
    ErreurNiveau, GenreDecor, GenrePanneau, Niveau, ObjetDecor, Panneau, ReglageAiguille, Tuile,
    GOUTTES_PAR_NIVEAU,
};
use crate::aabb::{touche_un_obstacle, Aabb, TAILLE_TUILE};
use crate::entities::ennemi::SOUS_L_EAU_PAS;
use crate::entities::montgolfiere::HAUT_BALLON;
use crate::entities::nuage::{NB_COLONNES, PORTEE_COLONNES, TAILLE_NUAGE};
use crate::entities::pieuvre::{self, MARGE_TENTACULE, PLACES_TETE, TAILLE_TETE};
use crate::entities::{
    Actualisable, Auvent, Bulles, Champignon, Chute, Eclair, Ennemi, Entite, Escargot, GoutteDor,
    GrandNuage, Grele, Lanterne, Lettre, Maree, Montgolfiere, Nenuphar, Pieuvre, PlateformeFriable,
    PlateformeMobile, Pointe, Stalactite, TypeEnnemi, Ventilateur,
};
use crate::physics::{CourantAscendant, Eau, Environnement, VentLateral, JOUEUR_H};

/// Vitesse (px/s) des plateformes mobiles.
pub(super) const VITESSE_MOBILE: f32 = 36.0;
/// Largeur (px) d'une plateforme mobile : deux tuiles.
const LARGEUR_MOBILE: f32 = 32.0;
/// Taille (px) de la plateforme d'une aiguille d'horloge : celle d'une plateforme mobile.
pub(super) const TAILLE_AIGUILLE: (f32, f32) = (LARGEUR_MOBILE, 8.0);
/// Vitesse maximale (px/s) d'une aiguille : 2 px par pas, Pépin suit sans peine.
const VITESSE_MAX_AIGUILLE: f32 = 120.0;
/// Repos (pas) entre deux coups d'éclair.
const REPOS_ECLAIR: u16 = 150;

/// Position d'une case : colonne `tx`, ligne `ty` (0 = ligne du haut).
type Case = (usize, usize);

/// Pixels du coin haut-gauche d'une case.
fn px(case: Case) -> (f32, f32) {
    (case.0 as f32 * TAILLE_TUILE, case.1 as f32 * TAILLE_TUILE)
}

pub(super) fn analyser(
    texte: &str,
    reglages_aiguilles: &[ReglageAiguille],
) -> Result<Niveau, ErreurNiveau> {
    let mut lignes: Vec<&str> = texte.lines().map(str::trim_end).collect();
    while lignes.first().is_some_and(|l| l.is_empty()) {
        lignes.remove(0);
    }
    while lignes.last().is_some_and(|l| l.is_empty()) {
        lignes.pop();
    }
    if lignes.is_empty() {
        return Err(ErreurNiveau::Vide);
    }
    let largeur = lignes[0].chars().count();
    for (i, l) in lignes.iter().enumerate() {
        let n = l.chars().count();
        if n != largeur {
            return Err(ErreurNiveau::LargeurInegale {
                ligne: i + 1,
                attendu: largeur,
                trouve: n,
            });
        }
    }
    let hauteur = lignes.len();
    let mut tuiles = vec![Tuile::Vide; largeur * hauteur];
    let mut entites: Vec<Entite> = Vec::new();
    let mut panneaux = Vec::new();
    // Décor : (case, genre), vérifié une fois la grille entière lue (il faut la
    // case du dessous).
    let mut decors: Vec<(Case, GenreDecor)> = Vec::new();
    let mut departs: Vec<Case> = Vec::new();
    let mut arrivees: Vec<Case> = Vec::new();
    let mut eclairs: Vec<Case> = Vec::new();
    // Grêle : il faut la grille entière pour trouver le solide sous la colonne.
    let mut greles: Vec<Case> = Vec::new();
    let mut vent = vec![false; largeur * hauteur];
    // Vents latéraux : `<` pousse vers la gauche (-1), `>` vers la droite (+1).
    let mut vents: Vec<(Case, i8)> = Vec::new();
    // Eau (carte 9.2) : `~` eau calme (courant 0), `{` courant vers la gauche
    // (-1), `}` vers la droite (+1). L'eau n'est PAS une tuile solide : elle ne
    // sert qu'au dessin et à la mécanique de la barque (zones dans `Niveau::eaux`).
    let mut eaux_cases: Vec<(Case, i8)> = Vec::new();
    let mut mobiles: BTreeMap<char, Vec<Case>> = BTreeMap::new();
    // Pigeons : il faut la grille entière pour trouver leur perchoir.
    let mut pigeons: Vec<Case> = Vec::new();
    // Ennemis d'eau à placer d'après la grille (carte 9.4) : mouettes (perchées
    // comme les pigeons), poissons (sur l'eau du dessous).
    let mut mouettes: Vec<Case> = Vec::new();
    let mut poissons: Vec<Case> = Vec::new();
    // Ventilateurs : il faut la grille entière (appui en dessous, plafond au-dessus).
    let mut ventilateurs: Vec<Case> = Vec::new();
    // Montgolfières : il faut la grille entière (place du ballon, plafond au-dessus).
    let mut montgolfieres: Vec<Case> = Vec::new();
    // Chats endormis : il faut la grille entière (appui dessous, place au-dessus).
    let mut chats: Vec<Case> = Vec::new();
    // Pivots des aiguilles d'horloge : il faut la grille entière (leur cercle ne
    // doit toucher aucun solide).
    let mut pivots: Vec<Case> = Vec::new();
    // Grand Nuage (au plus un) : il faut la grille entière (sa place, ses colonnes
    // d'éclairs jusqu'au sol, ses courants).
    let mut nuages: Vec<Case> = Vec::new();
    // Pieuvre du port (carte 9.12a, au plus un boss en tout) : son bassin entier.
    let mut pieuvres: Vec<Case> = Vec::new();
    let mut gouttes = 0usize;
    let mut lettres = 0usize;
    // Secrets (carte 8.5) : faux murs `m` et sortie secrète `x`.
    let mut faux_murs: Vec<Case> = Vec::new();
    let mut sorties_secretes: Vec<Case> = Vec::new();
    // Mur de pluie (au plus un, carte 8.2) : cases `P`.
    let mut murs: Vec<Case> = Vec::new();
    // Mécaniques d'eau (carte 9.3) : il faut la grille entière (l'eau sous un
    // nénuphar, le plafond d'une stalactite, l'appui d'une marée...). Bulles
    // `Y`, chutes d'eau `|` et marées `=` forment des rectangles, comme le vent.
    let mut bulles = vec![false; largeur * hauteur];
    let mut chutes = vec![false; largeur * hauteur];
    let mut marees = vec![false; largeur * hauteur];
    let mut nenuphars: Vec<Case> = Vec::new();
    let mut stalactites: Vec<Case> = Vec::new();

    for (ty, ligne) in lignes.iter().enumerate() {
        for (tx, c) in ligne.chars().enumerate() {
            let case = (tx, ty);
            let (x, y) = px(case);
            let mut panneau = |genre| panneaux.push(Panneau { x, y, genre });
            match c {
                '.' => {}
                '#' => tuiles[ty * largeur + tx] = Tuile::Toit,
                'B' => tuiles[ty * largeur + tx] = Tuile::Brique,
                'N' => tuiles[ty * largeur + tx] = Tuile::Nuage,
                '@' => departs.push(case),
                'X' => arrivees.push(case),
                'm' => faux_murs.push(case),
                'x' => sorties_secretes.push(case),
                'L' => entites.push(Entite::Lanterne(Lanterne::new(x + 4.0, y))),
                'o' => {
                    entites.push(Entite::Goutte(GoutteDor::new(
                        gouttes as u8,
                        x + 4.0,
                        y + 4.0,
                    )));
                    gouttes += 1;
                }
                // Lettre cachée (carte 8.6) : 8 x 8 au milieu de la case, comme la goutte.
                '*' => {
                    lettres += 1;
                    entites.push(Entite::Lettre(Lettre::new(x + 4.0, y + 4.0)));
                }
                '^' => entites.push(Entite::Pointe(Pointe::new(x, y + 8.0))),
                'M' => entites.push(Entite::Champignon(Champignon::new(x, y + 8.0))),
                // Escargot : boîte 16 x 10 au bas de la case (il marche sur le toit
                // du dessous, comme la flaque ; pas d'appui exigé : en l'air, il tombe).
                's' => entites.push(Entite::Escargot(Escargot::new(x, y + 6.0))),
                'A' => entites.push(Entite::Auvent(Auvent::dans_la_case(x, y))),
                'f' => entites.push(Entite::Ennemi(Ennemi::new(
                    TypeEnnemi::Flaque,
                    x + 2.0,
                    y + 8.0,
                ))),
                'c' => entites.push(Entite::Ennemi(Ennemi::new(
                    TypeEnnemi::Corbeau,
                    x + 2.0,
                    y + 3.0,
                ))),
                'p' => pigeons.push(case),
                // Ennemis d'eau (carte 9.4) : grenouille et crabe tombent sur ce
                // qui est dessous (la grenouille se pose aussi sur l'eau), le rat
                // de même ; la mouette et le poisson se placent après la lecture.
                'j' => entites.push(Entite::Ennemi(Ennemi::grenouille(x, y))),
                'k' => entites.push(Entite::Ennemi(Ennemi::crabe(x, y))),
                'r' => entites.push(Entite::Ennemi(Ennemi::rat(x, y))),
                'u' => mouettes.push(case),
                'i' => poissons.push(case),
                'F' => entites.push(Entite::Friable(PlateformeFriable::new(Aabb::new(
                    x, y, 16.0, 8.0,
                )))),
                'e' => eclairs.push(case),
                'g' => greles.push(case),
                'v' => vent[ty * largeur + tx] = true,
                '<' => vents.push((case, -1)),
                '>' => vents.push((case, 1)),
                'w' => ventilateurs.push(case),
                'b' => montgolfieres.push(case),
                'z' => chats.push(case),
                'h' => pivots.push(case),
                'O' => nuages.push(case),
                'P' => murs.push(case),
                'Y' => bulles[ty * largeur + tx] = true,
                '|' => chutes[ty * largeur + tx] = true,
                '=' => marees[ty * largeur + tx] = true,
                'n' => nenuphars.push(case),
                'V' => stalactites.push(case),
                // Eau (carte 9.2) : `~` calme, `{` courant vers la gauche, `}`
                // vers la droite. La tuile n'est PAS solide : le moteur de
                // collision ne la voit pas, seule la surface compte (le haut de
                // l'étendue, assemblée dans `Niveau::eaux`).
                '~' => {
                    tuiles[ty * largeur + tx] = Tuile::Eau;
                    eaux_cases.push((case, 0));
                }
                '{' => {
                    tuiles[ty * largeur + tx] = Tuile::Eau;
                    eaux_cases.push((case, -1));
                }
                '}' => {
                    tuiles[ty * largeur + tx] = Tuile::Eau;
                    eaux_cases.push((case, 1));
                }
                // La Pieuvre (carte 9.12a) : sa case est de l'eau calme, elle vit
                // dessous. Avant `'1'..='9'` : le chiffre 8 n'est plus une
                // plateforme mobile (aucun niveau ne s'en servait).
                '8' => {
                    tuiles[ty * largeur + tx] = Tuile::Eau;
                    eaux_cases.push((case, 0));
                    pieuvres.push(case);
                }
                'D' => panneau(GenrePanneau::Deplacer),
                'S' => panneau(GenrePanneau::Sauter),
                'G' => panneau(GenrePanneau::Planer),
                'C' => panneau(GenrePanneau::Champignon),
                'W' => panneau(GenrePanneau::Vent),
                '1'..='9' => mobiles.entry(c).or_default().push(case),
                // Le reste : une lettre de décor, ou un caractère hors légende.
                _ => match GenreDecor::depuis_lettre(c) {
                    Some(genre) => decors.push((case, genre)),
                    None => {
                        return Err(ErreurNiveau::CaractereInconnu {
                            ligne: ty + 1,
                            colonne: tx + 1,
                            caractere: c,
                        })
                    }
                },
            }
        }
    }

    // --- Validation des repères ------------------------------------------
    let depart_case = match departs.as_slice() {
        [] => return Err(ErreurNiveau::DepartManquant),
        [d] => *d,
        autres => {
            return Err(ErreurNiveau::DepartMultiple {
                nombre: autres.len(),
            })
        }
    };
    // L'arène du Grand Nuage n'a pas de drapeau : le vaincre finit le niveau.
    // Tout autre niveau a exactement une arrivée.
    if nuages.len() > 1 {
        return Err(ErreurNiveau::NuageMultiple {
            nombre: nuages.len(),
        });
    }
    if !pieuvres.is_empty() && nuages.len() + pieuvres.len() > 1 {
        return Err(ErreurNiveau::BossMultiple {
            nombre: nuages.len() + pieuvres.len(),
        });
    }
    let arrivee_case = match (
        arrivees.as_slice(),
        nuages.is_empty() && pieuvres.is_empty(),
    ) {
        ([], true) => return Err(ErreurNiveau::ArriveeManquante),
        ([], false) => None,
        ([a], true) => Some(*a),
        ([_], false) if nuages.is_empty() => return Err(ErreurNiveau::ArriveeEtPieuvre),
        ([_], false) => return Err(ErreurNiveau::ArriveeEtNuage),
        (autres, _) => {
            return Err(ErreurNiveau::ArriveeMultiple {
                nombre: autres.len(),
            })
        }
    };
    if lettres > 1 {
        return Err(ErreurNiveau::LettreMultiple { nombre: lettres });
    }
    if sorties_secretes.len() > 1 {
        return Err(ErreurNiveau::SortieSecreteMultiple {
            nombre: sorties_secretes.len(),
        });
    }
    if gouttes != GOUTTES_PAR_NIVEAU {
        return Err(ErreurNiveau::NombreDeGouttes { trouve: gouttes });
    }

    // --- Éléments qui dépendent de la grille complète ----------------------
    // Éclair : colonne depuis le premier solide AU-DESSUS du `e` (le haut de la carte s'il
    // n'y en a pas) jusqu'au premier solide en dessous. Dans un niveau d'un écran, le `e` est
    // posé sur la rangée du haut : la colonne part du haut de la carte, comme toujours. Dans
    // une grille haute, elle ne traverse pas les corniches au-dessus d'elle.
    for (k, &(tx, ty)) in eclairs.iter().enumerate() {
        let solide = |r: usize| tuiles[r * largeur + tx].est_solide();
        let premier_solide = (ty..hauteur).find(|&r| solide(r)).unwrap_or(hauteur);
        let plafond = (0..ty).rev().find(|&r| solide(r)).map_or(0, |r| r + 1);
        let haut = plafond as f32 * TAILLE_TUILE;
        let bas = (premier_solide as f32 * TAILLE_TUILE).max(haut + TAILLE_TUILE);
        let zone = Aabb::new(tx as f32 * TAILLE_TUILE + 2.0, haut, 12.0, bas - haut);
        let decalage = 30 + 47 * (k as u16 % 4);
        entites.push(Entite::Eclair(Eclair::new(zone, REPOS_ECLAIR, decalage)));
    }
    // Grêle : colonne de la case `g` elle-même (pas du haut de la carte, à la
    // différence de l'éclair) jusqu'au premier solide en dessous, ou jusqu'au bas
    // de la carte s'il n'y en a pas (rideau de grêle au-dessus d'un vide). Même
    // largeur que l'éclair : 12 px centrés dans la case (2 px de pardon par côté).
    for &(tx, ty) in &greles {
        let premier_solide = (ty..hauteur)
            .find(|&r| tuiles[r * largeur + tx].est_solide())
            .unwrap_or(hauteur);
        let haut = ty as f32 * TAILLE_TUILE;
        let bas = premier_solide as f32 * TAILLE_TUILE;
        let zone = Aabb::new(tx as f32 * TAILLE_TUILE + 2.0, haut, 12.0, bas - haut);
        entites.push(Entite::Grele(Grele::new(zone)));
    }
    // Mécaniques d'eau (carte 9.3) : voir `poser_mecaniques_d_eau`.
    poser_mecaniques_d_eau(
        &tuiles,
        largeur,
        hauteur,
        (&bulles, &chutes, &marees),
        (&nenuphars, &stalactites),
        &mut entites,
    )?;
    // Plateformes mobiles : un chiffre = deux extrémités (ordre de lecture).
    for (marque, cases) in &mobiles {
        let [a, b] = cases.as_slice() else {
            return Err(ErreurNiveau::PlateformeMobileIncomplete {
                marque: *marque,
                trouve: cases.len(),
            });
        };
        let (pa, pb) = (px(*a), px(*b));
        let boite = Aabb::new(pa.0, pa.1, LARGEUR_MOBILE, 8.0);
        entites.push(Entite::Mobile(PlateformeMobile::new(
            boite,
            pa,
            pb,
            VITESSE_MOBILE,
        )));
    }

    // Pigeons : perchés sur la tuile du dessous (obligatoire), boîte 12 x 10
    // centrée dans la case, pattes sur le perchoir. Ils regardent du côté où le
    // perchoir s'arrête : à droite seulement si le perchoir continue à gauche et
    // s'arrête à droite (sinon à gauche, vers le départ de Pépin).
    let solide = |tx: usize, ty: usize| tuiles[ty * largeur + tx].est_solide();
    for (tx, ty) in pigeons {
        if ty + 1 >= hauteur || !solide(tx, ty + 1) {
            return Err(ErreurNiveau::PigeonSansPerchoir {
                ligne: ty + 1,
                colonne: tx + 1,
            });
        }
        let gauche = tx > 0 && solide(tx - 1, ty + 1);
        let droite = tx + 1 < largeur && solide(tx + 1, ty + 1);
        let (x, y) = px((tx, ty));
        entites.push(Entite::Ennemi(Ennemi::pigeon(
            x + 2.0,
            y + 6.0,
            gauche && !droite,
        )));
    }

    // Mouettes (carte 9.4) : perchées et tournées comme les pigeons ; poissons :
    // la case `i` est la case d'air juste au-dessus de la surface, l'eau est
    // dessous (obligatoire). Deux poissons voisins ne sautent pas ensemble.
    for (tx, ty) in mouettes {
        if ty + 1 >= hauteur || !solide(tx, ty + 1) {
            return Err(ErreurNiveau::EnnemiDEauMalPlace {
                ligne: ty + 1,
                colonne: tx + 1,
                caractere: 'u',
            });
        }
        let gauche = tx > 0 && solide(tx - 1, ty + 1);
        let droite = tx + 1 < largeur && solide(tx + 1, ty + 1);
        let (x, y) = px((tx, ty));
        entites.push(Entite::Ennemi(Ennemi::mouette(
            x + 2.0,
            y + 6.0,
            gauche && !droite,
        )));
    }
    for (k, &(tx, ty)) in poissons.iter().enumerate() {
        if ty + 1 >= hauteur || tuiles[(ty + 1) * largeur + tx] != Tuile::Eau {
            return Err(ErreurNiveau::EnnemiDEauMalPlace {
                ligne: ty + 1,
                colonne: tx + 1,
                caractere: 'i',
            });
        }
        let (x, y) = px((tx, ty + 1));
        let decalage = (40 * k as u16) % SOUS_L_EAU_PAS;
        entites.push(Entite::Ennemi(Ennemi::poisson(x, y, decalage)));
    }

    // Ventilateurs : hélice posée sur le solide du dessous (le socle repose
    // dessus, donc il faut un appui). Le courant part du dessus du socle et
    // monte jusqu'au 1er solide au-dessus (ou au haut de la carte s'il n'y en
    // a pas). Le socle est solide (16 x 8 px, bas de la case) : on se tient
    // dessus ; le courant fait 16 px de large (la largeur du socle), à la
    // différence des colonnes de 12 px de l'éclair et de la grêle.
    for (k, &(tx, ty)) in ventilateurs.iter().enumerate() {
        if ty + 1 >= hauteur || !solide(tx, ty + 1) {
            return Err(ErreurNiveau::VentilateurSansAppui {
                ligne: ty + 1,
                colonne: tx + 1,
            });
        }
        // 1er solide au-dessus de la case (le haut de la carte s'il n'y en a pas).
        let plafond = (0..ty).rev().find(|&r| solide(tx, r)).map_or(0, |r| r + 1);
        let haut = plafond as f32 * TAILLE_TUILE;
        let dessus_socle = ty as f32 * TAILLE_TUILE + 8.0;
        let zone = Aabb::new(tx as f32 * TAILLE_TUILE, haut, 16.0, dessus_socle - haut);
        let boite = Aabb::new(tx as f32 * TAILLE_TUILE, dessus_socle, 16.0, 8.0);
        // Désynchronisation des ventilateurs (comme pour les éclairs).
        let decalage = 12 + 18 * (k as u16 % 4);
        entites.push(Entite::Ventilateur(Ventilateur::nouveau(
            boite, zone, decalage,
        )));
    }

    // Montgolfières : la nacelle occupe la moitié HAUTE de la case `b` (son dessus
    // est au ras d'un toit de la même rangée : on y monte en marchant). Le ballon
    // occupe les deux cases au-dessus : elles doivent être libres. La nacelle
    // monte au plus jusqu'au 1er solide au-dessus de la colonne (le haut du
    // ballon s'y arrête ; le haut de la carte s'il n'y en a pas), et jamais plus
    // de `MONTEE_MAX` (8 tuiles).
    let rangees_ballon = (HAUT_BALLON / TAILLE_TUILE) as usize;
    for &(tx, ty) in &montgolfieres {
        if ty < rangees_ballon || (ty - rangees_ballon..ty).any(|r| solide(tx, r)) {
            return Err(ErreurNiveau::MontgolfiereSansPlace {
                ligne: ty + 1,
                colonne: tx + 1,
            });
        }
        let plafond = (0..ty).rev().find(|&r| solide(tx, r)).map_or(0, |r| r + 1);
        let (x, y) = px((tx, ty));
        let y_haut = Montgolfiere::sommet(y, plafond as f32 * TAILLE_TUILE);
        entites.push(Entite::Montgolfiere(Montgolfiere::nouvelle(x, y, y_haut)));
    }

    // Chats endormis : couchés sur le solide du dessous (obligatoire), boîte 14 x 8
    // au bas de la case, centrée. La case du dessus doit être libre (ni solide, ni
    // bord haut de la carte) : on doit toujours pouvoir lui sauter dessus pendant
    // son sommeil. Il regarde à gauche (vers le départ), sauf si un solide le
    // borde à gauche et que la droite est libre. Les premiers sommeils sont
    // raccourcis de 0, 30, 60 ou 90 pas (le sommeil dure 100 pas depuis la carte
    // 6.3) : deux chats ne se réveillent pas ensemble (comme les éclairs et les
    // ventilateurs).
    for (k, &(tx, ty)) in chats.iter().enumerate() {
        let appui = ty + 1 < hauteur && solide(tx, ty + 1);
        let place = ty > 0 && !solide(tx, ty - 1);
        if !appui || !place {
            return Err(ErreurNiveau::ChatSansPlace {
                ligne: ty + 1,
                colonne: tx + 1,
            });
        }
        let mur_gauche = tx > 0 && solide(tx - 1, ty);
        let libre_droite = tx + 1 < largeur && !solide(tx + 1, ty);
        let (x, y) = px((tx, ty));
        let decalage = 30 * (k as u16 % 4);
        let mut chat = Ennemi::chat(x + 1.0, y + 8.0, decalage);
        chat.vers_la_droite = mur_gauche && libre_droite;
        entites.push(Entite::Ennemi(chat));
    }

    // Grand Nuage : voir `poser_nuage`.
    for &case in &nuages {
        let nuage = poser_nuage(&tuiles, largeur, hauteur, case)?;
        entites.push(Entite::Nuage(Box::new(nuage)));
    }
    // Pieuvre du port : voir `poser_pieuvre`.
    for &case in &pieuvres {
        let p = poser_pieuvre(&tuiles, largeur, case)?;
        entites.push(Entite::Pieuvre(Box::new(p)));
    }

    // Vents latéraux : les cases `<` et `>` forment des rectangles de même
    // sens (deux zones voisines de sens opposés restent distinctes).
    let grille_vent = |sens: i8| {
        let mut grille = vec![false; largeur * hauteur];
        for ((tx, ty), s) in &vents {
            if *s == sens {
                grille[ty * largeur + tx] = true;
            }
        }
        grille
    };
    let mut vents_laterales: Vec<VentLateral> = Vec::new();
    for sens in [-1i8, 1] {
        let grille = grille_vent(sens);
        for (g, d, h, b) in fusionner_zones(&grille, largeur, hauteur) {
            vents_laterales.push(VentLateral::new(
                Aabb::new(
                    g as f32 * TAILLE_TUILE,
                    h as f32 * TAILLE_TUILE,
                    (d - g + 1) as f32 * TAILLE_TUILE,
                    (b - h + 1) as f32 * TAILLE_TUILE,
                ),
                sens,
            ));
        }
    }

    let courants = fusionner_courants(&vent, largeur, hauteur);

    // Eau (carte 9.2) : les cases d'un même courant forment des rectangles
    // (comme les vents latéraux, par sens). La surface de l'eau, c'est le HAUT
    // du rectangle : `physics::eau_touchee` cherche le rectangle que Pépin
    // chevauche en abscisse ET en hauteur, du plus haut au plus bas (c'est pour
    // cela que l'ordre du vecteur est par surface croissante).
    let grille_eau = |courant: i8| {
        let mut grille = vec![false; largeur * hauteur];
        for ((tx, ty), c) in &eaux_cases {
            if *c == courant {
                grille[*ty * largeur + *tx] = true;
            }
        }
        grille
    };
    let mut eaux: Vec<Eau> = Vec::new();
    for sens in [0i8, -1, 1] {
        for (g, d, h, b) in fusionner_zones(&grille_eau(sens), largeur, hauteur) {
            eaux.push(Eau {
                zone: Aabb::new(
                    g as f32 * TAILLE_TUILE,
                    h as f32 * TAILLE_TUILE,
                    (d - g + 1) as f32 * TAILLE_TUILE,
                    (b - h + 1) as f32 * TAILLE_TUILE,
                ),
                courant: sens,
            });
        }
    }
    // Du plus haut au plus bas : si une boîte touche deux étendues empilées,
    // `eau_touchee` retient la plus haute.
    eaux.sort_by(|a, b| a.surface().total_cmp(&b.surface()));
    // Décor : posé sur la tuile du dessous, qui doit être solide (la dernière
    // ligne n'a pas de dessous : le décor y serait au-dessus du vide).
    let mut objets_decor = Vec::with_capacity(decors.len());
    for ((tx, ty), genre) in decors {
        let appui = ty + 1 < hauteur && tuiles[(ty + 1) * largeur + tx].est_solide();
        if !appui {
            return Err(ErreurNiveau::DecorSansAppui {
                ligne: ty + 1,
                colonne: tx + 1,
                caractere: genre.lettre(),
            });
        }
        let (x, y) = px((tx, ty));
        objets_decor.push(ObjetDecor { x, y, genre });
    }
    let caches = super::secrets::zones_cachees(&tuiles, largeur, hauteur, &faux_murs);
    let (dx, dy) = px(depart_case);
    let mut niveau = Niveau {
        largeur,
        hauteur,
        tuiles,
        // Pépin est posé au centre de la case, les pieds au bas de la case.
        depart: (
            dx + (TAILLE_TUILE - 10.0) / 2.0,
            dy + TAILLE_TUILE - JOUEUR_H,
        ),
        // Drapeau : la case `X` et celle du dessus (aucun dans l'arène du Grand Nuage).
        arrivee: arrivee_case.map(|case| {
            let (ax, ay) = px(case);
            Aabb::new(ax, ay - TAILLE_TUILE, TAILLE_TUILE, 2.0 * TAILLE_TUILE)
        }),
        entites,
        courants,
        vents: vents_laterales,
        eaux,
        panneaux,
        decors: objets_decor,
        caches,
        // Même boîte que le drapeau `X`.
        sortie_secrete: sorties_secretes.first().map(|&case| {
            let (sx, sy) = px(case);
            Aabb::new(sx, sy - TAILLE_TUILE, TAILLE_TUILE, 2.0 * TAILLE_TUILE)
        }),
        mur_de_pluie: match murs.as_slice() {
            [] => None,
            [m] => Some(px(*m).0 + TAILLE_TUILE),
            autres => {
                return Err(ErreurNiveau::MurMultiple {
                    nombre: autres.len(),
                })
            }
        },
    };
    // Aiguilles d'horloge : en DERNIER, une fois le niveau assemblé, pour vérifier
    // leur cercle contre ses tuiles (`touche_un_obstacle` sur le niveau lui-même).
    ajouter_aiguilles(&mut niveau, &pivots, reglages_aiguilles)?;
    Ok(niveau)
}

/// Les mécaniques d'eau de la carte 9.3, ajoutées aux entités :
/// - colonnes de bulles `Y` (rectangles), PROLONGÉES vers le bas à travers
///   l'eau qui est juste dessous : les bulles partent du fond du bassin ;
/// - chutes d'eau `|` (rectangles, tels quels) ;
/// - marées `=` (rectangles) : la rangée sous chacune est solide ou de l'eau
///   (le bas de la carte compte comme un appui manquant) ;
/// - nénuphars `n` : la case du dessous est de l'eau ;
/// - stalactites `V` : la case du dessus est solide ; elles tombent jusqu'au
///   premier solide ou à la première eau en dessous (le bas de la carte sinon).
fn poser_mecaniques_d_eau(
    tuiles: &[Tuile],
    largeur: usize,
    hauteur: usize,
    (bulles, chutes, marees): (&[bool], &[bool], &[bool]),
    (nenuphars, stalactites): (&[Case], &[Case]),
    entites: &mut Vec<Entite>,
) -> Result<(), ErreurNiveau> {
    let t = TAILLE_TUILE;
    let tuile = |x: usize, y: usize| tuiles[y * largeur + x];
    let rect = |g: usize, d: usize, h: usize, b: usize| {
        Aabb::new(
            g as f32 * t,
            h as f32 * t,
            (d - g + 1) as f32 * t,
            (b - h + 1) as f32 * t,
        )
    };
    let mal_pose = |(tx, ty): Case, caractere| ErreurNiveau::EauMalPosee {
        ligne: ty + 1,
        colonne: tx + 1,
        caractere,
    };
    for (g, d, h, mut b) in fusionner_zones(bulles, largeur, hauteur) {
        while b + 1 < hauteur && (g..=d).all(|x| tuile(x, b + 1) == Tuile::Eau) {
            b += 1;
        }
        entites.push(Entite::Bulles(Bulles::new(rect(g, d, h, b))));
    }
    for (g, d, h, b) in fusionner_zones(chutes, largeur, hauteur) {
        entites.push(Entite::Chute(Chute::new(rect(g, d, h, b))));
    }
    for (g, d, h, b) in fusionner_zones(marees, largeur, hauteur) {
        let appui = |x: usize| b + 1 < hauteur && tuile(x, b + 1) != Tuile::Vide;
        if let Some(x) = (g..=d).find(|&x| !appui(x)) {
            return Err(mal_pose((x, b), '='));
        }
        entites.push(Entite::Maree(Maree::new(rect(g, d, h, b))));
    }
    for &(tx, ty) in nenuphars {
        if ty + 1 >= hauteur || tuile(tx, ty + 1) != Tuile::Eau {
            return Err(mal_pose((tx, ty), 'n'));
        }
        let (x, y) = px((tx, ty));
        entites.push(Entite::Nenuphar(Nenuphar::dans_la_case(x, y)));
    }
    for &(tx, ty) in stalactites {
        if ty == 0 || !tuile(tx, ty - 1).est_solide() {
            return Err(mal_pose((tx, ty), 'V'));
        }
        let sol = (ty + 1..hauteur)
            .find(|&y| tuile(tx, y) != Tuile::Vide)
            .unwrap_or(hauteur);
        let (x, y) = px((tx, ty));
        entites.push(Entite::Stalactite(Stalactite::new(x, y, sol as f32 * t)));
    }
    Ok(())
}

/// Le Grand Nuage de la case `O` (`tx`, `ty`) :
/// - son corps (48 x 32) couvre la case `O` et ses voisines de gauche et de droite,
///   sur deux rangées (`O` est le milieu de sa rangée du haut) ; ces six cases et
///   les trois du dessus (on lui saute sur la tête) doivent être libres, dans la
///   carte ;
/// - ses 17 colonnes d'éclairs, de 8 colonnes à gauche de `O` à 8 à droite (toutes
///   dans la carte), partent du dessous du nuage et descendent jusqu'au 1er solide
///   (ou au bas de la carte), 12 px de large au milieu de leur case, comme un
///   éclair `e` ;
/// - ses deux courants (2 cases de large) sont à une case de lui, à gauche et à
///   droite ; ils partent d'une demi-case au-dessus de sa tête (Pépin y monte un peu
///   plus haut qu'elle, puis lui retombe dessus) et descendent jusqu'au 1er solide
///   sous le nuage, dans la plus profonde de leurs deux colonnes.
fn poser_nuage(
    tuiles: &[Tuile],
    largeur: usize,
    hauteur: usize,
    (tx, ty): Case,
) -> Result<GrandNuage, ErreurNiveau> {
    let erreur = ErreurNiveau::NuageSansPlace {
        ligne: ty + 1,
        colonne: tx + 1,
    };
    let portee = PORTEE_COLONNES as usize;
    if ty == 0 || ty + 2 >= hauteur || tx < portee || tx + portee >= largeur {
        return Err(erreur);
    }
    let solide = |x: usize, y: usize| tuiles[y * largeur + x].est_solide();
    if (tx - 1..=tx + 1).any(|x| (ty - 1..=ty + 1).any(|y| solide(x, y))) {
        return Err(erreur);
    }
    let t = TAILLE_TUILE;
    // 1re rangée solide à partir de `depuis` dans la colonne `x` (le bas de la carte
    // s'il n'y en a pas).
    let premier_solide = |x: usize, depuis: usize| {
        (depuis..hauteur).find(|&y| solide(x, y)).unwrap_or(hauteur) as f32 * t
    };
    let (x, y) = px((tx - 1, ty));
    let boite = Aabb::new(x, y, TAILLE_NUAGE.0, TAILLE_NUAGE.1);
    let dessous = boite.bas();
    let colonnes: [Aabb; NB_COLONNES] = std::array::from_fn(|i| {
        let cx = tx + i - portee;
        let bas = premier_solide(cx, ty + 2).max(dessous + t);
        Aabb::new(cx as f32 * t + 2.0, dessous, 12.0, bas - dessous)
    });
    let courant = |gauche: usize| {
        let haut = boite.y - t / 2.0;
        let bas = premier_solide(gauche, ty).max(premier_solide(gauche + 1, ty));
        Aabb::new(gauche as f32 * t, haut, 2.0 * t, bas - haut)
    };
    Ok(GrandNuage::new(
        boite,
        colonnes,
        [courant(tx - 4), courant(tx + 3)],
    ))
}

/// La Pieuvre du port de la case `8` (`tx`, `ty`), carte 9.12a :
/// - la rangée `ty` est la surface de son bassin : de 8 cases à gauche du `8` à 9
///   à droite, toutes d'eau (18 cases, dans la carte) ;
/// - ses 9 colonnes de tentacules font deux cases de large (la colonne 0 : le `8`
///   et sa voisine de droite) ; chaque tentacule (24 px, `MARGE_TENTACULE` de
///   chaque côté) va de la surface jusqu'en haut de la carte, « de toute la
///   hauteur » : il traverse les pontons et les caisses ;
/// - ses 3 places de tête (`PLACES_TETE` : milieu, gauche, droite), 48 x 32,
///   centrées sur leur colonne et posées sur l'eau, ne touchent aucun solide.
fn poser_pieuvre(
    tuiles: &[Tuile],
    largeur: usize,
    (tx, ty): Case,
) -> Result<Pieuvre, ErreurNiveau> {
    let erreur = ErreurNiveau::PieuvreSansPlace {
        ligne: ty + 1,
        colonne: tx + 1,
    };
    let portee = 2 * pieuvre::PORTEE_COLONNES as usize;
    if ty < 2 || tx < portee || tx + portee + 1 >= largeur {
        return Err(erreur);
    }
    if (tx - portee..=tx + portee + 1).any(|x| tuiles[ty * largeur + x] != Tuile::Eau) {
        return Err(erreur);
    }
    let t = TAILLE_TUILE;
    let surface = ty as f32 * t;
    // Abscisse (px) du bord gauche de la colonne `k` (de -4 à 4).
    let gauche = |k: i8| (tx as f32 + 2.0 * f32::from(k)) * t;
    let colonnes: [Aabb; pieuvre::NB_COLONNES] = std::array::from_fn(|i| {
        let k = i as i8 - pieuvre::PORTEE_COLONNES;
        Aabb::new(
            gauche(k) + MARGE_TENTACULE,
            0.0,
            pieuvre::LARGEUR_COLONNE - 2.0 * MARGE_TENTACULE,
            surface,
        )
    });
    let places = PLACES_TETE.map(|k| {
        let milieu = gauche(k) + pieuvre::LARGEUR_COLONNE / 2.0;
        Aabb::new(
            milieu - TAILLE_TETE.0 / 2.0,
            surface - TAILLE_TETE.1,
            TAILLE_TETE.0,
            TAILLE_TETE.1,
        )
    });
    for tete in &places {
        let (x0, x1) = ((tete.x / t) as usize, ((tete.droite() - 1.0) / t) as usize);
        let solide = (ty - 2..ty).any(|y| (x0..=x1).any(|x| tuiles[y * largeur + x].est_solide()));
        if solide {
            return Err(erreur);
        }
    }
    Ok(Pieuvre::new(colonnes, places))
}

/// Pose une aiguille d'horloge sur chaque pivot `h` (dans l'ordre de lecture),
/// avec la ligne correspondante de la table `reglages`. Erreurs : pas autant de
/// réglages que de pivots ; réglage hors des bornes ; cercle qui touche un solide.
fn ajouter_aiguilles(
    niveau: &mut Niveau,
    pivots: &[Case],
    reglages: &[ReglageAiguille],
) -> Result<(), ErreurNiveau> {
    if pivots.len() != reglages.len() {
        return Err(ErreurNiveau::AiguillesSansReglage {
            pivots: pivots.len(),
            reglages: reglages.len(),
        });
    }
    for (&(tx, ty), r) in pivots.iter().zip(reglages) {
        let ou = (ty + 1, tx + 1);
        let valide = r.rayon.is_finite()
            && r.rayon >= 1.0
            && r.vitesse > 0.0
            && r.vitesse <= VITESSE_MAX_AIGUILLE
            && r.phase.is_finite();
        if !valide {
            return Err(ErreurNiveau::AiguilleInvalide {
                ligne: ou.0,
                colonne: ou.1,
            });
        }
        let (x, y) = px((tx, ty));
        let centre = (x + TAILLE_TUILE / 2.0, y + TAILLE_TUILE / 2.0);
        let aiguille = PlateformeMobile::aiguille(
            TAILLE_AIGUILLE,
            centre,
            r.rayon * TAILLE_TUILE,
            r.vitesse,
            r.phase / 12.0,
        );
        // Un tour complet, pas à pas, exactement comme dans le jeu (sur une copie :
        // l'aiguille du niveau reste à son heure de départ) : ni la plateforme ni
        // un Pépin debout dessus (14 px au-dessus d'elle) ne doivent toucher une
        // tuile solide. Ainsi, porté par l'aiguille, Pépin ne se cogne jamais au
        // décor (il ne serait plus emporté, et l'aiguille lui passerait au travers).
        let mut essai = aiguille.clone();
        let tour = essai.cercle().map_or(0.0, |c| c.pas_par_tour()).ceil() as u32;
        let env = Environnement::simple(niveau);
        for _ in 0..=tour {
            let b = essai.boite;
            let avec_pepin = Aabb::new(b.x, b.y - JOUEUR_H, b.w, b.h + JOUEUR_H);
            if touche_un_obstacle(&avec_pepin, niveau, &[]) {
                return Err(ErreurNiveau::AiguilleBloquee {
                    ligne: ou.0,
                    colonne: ou.1,
                });
            }
            essai.actualiser(&env);
        }
        niveau.entites.push(Entite::Mobile(aiguille));
    }
    Ok(())
}

/// Regroupe les cases d'un vent en rectangles : d'abord des tronçons
/// verticaux par colonne, puis fusion des colonnes voisines de même hauteur.
fn fusionner_courants(vent: &[bool], largeur: usize, hauteur: usize) -> Vec<CourantAscendant> {
    fusionner_zones(vent, largeur, hauteur)
        .into_iter()
        .map(|(g, d, h, b)| {
            CourantAscendant::new(Aabb::new(
                g as f32 * TAILLE_TUILE,
                h as f32 * TAILLE_TUILE,
                (d - g + 1) as f32 * TAILLE_TUILE,
                (b - h + 1) as f32 * TAILLE_TUILE,
            ))
        })
        .collect()
}

/// Regroupe les cases marquées `true` d'une grille en rectangles : d'abord
/// des tronçons verticaux par colonne, puis fusion des colonnes voisines de
/// même hauteur. Bornes incluses : (colonne gauche, colonne droite, ligne
/// haute, ligne basse).
fn fusionner_zones(
    zone: &[bool],
    largeur: usize,
    hauteur: usize,
) -> Vec<(usize, usize, usize, usize)> {
    let mut rects: Vec<(usize, usize, usize, usize)> = Vec::new();
    for tx in 0..largeur {
        let mut ty = 0;
        while ty < hauteur {
            if !zone[ty * largeur + tx] {
                ty += 1;
                continue;
            }
            let haut = ty;
            while ty < hauteur && zone[ty * largeur + tx] {
                ty += 1;
            }
            let bas = ty - 1;
            match rects
                .iter_mut()
                .find(|r| r.1 + 1 == tx && r.2 == haut && r.3 == bas)
            {
                Some(r) => r.1 = tx,
                None => rects.push((tx, tx, haut, bas)),
            }
        }
    }
    rects
}
