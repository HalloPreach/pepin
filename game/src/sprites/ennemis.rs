//! Sprites des ennemis : flaques qui patrouillent (16x10), corbeaux (18x14),
//! pigeons (18x14), chats endormis (16x14, avec leurs « z » et leur « ! »), et
//! l'escargot (16x12), qui n'est pas un ennemi mais patrouille comme eux ; le
//! Grand Nuage, le boss (48x32 : colère, essoufflé, touché), et ses points de vie
//! dans le HUD (10x6).

// `corbeau0.png` : Corbeau en vol (18x14, de profil, tourné vers la droite) : aile HAUTE. Carte 6.1 :
// l'ancien corbeau (16x12, deux ailes en V autour d'un corps rond) se lisait comme
// une chauve-souris. Celui-ci ne se lit que d'un côté : gros bec gris clair (BRUME,
// NUAGE) devant, œil orange juste derrière le bec, tête ronde au crâne éclairé,
// queue en éventail derrière, et UNE aile qui bat depuis le dos, ses rémiges
// écartées en « doigts » au bout. Plumage bleu nuit (`1`) ; reflets ARDOISE_CLAIRE
// (`3`) sur le dos et ARDOISE_PALE (`4`) sur le bord d'attaque de l'aile et le
// crâne : ce sont eux qui le détachent du ciel d'ardoise du grand orage (le bleu
// nuit seul s'y perdrait). Contour sombre de 1 px tout autour.

// `corbeau1.png` : Corbeau en vol : aile BASSE (elle descend sous le ventre, pointe vers l'arrière).
// Le corps, la tête et le bec ne bougent pas : seule l'aile bat.

// `chauve_souris0.png` : Chauve-souris (carte 9.26b) : le corbeau du Monde 2 (grotte, égouts…), même
// taille (18x14), même boîte, même battement en 2 images ; seul le dessin change.
// De face : petites oreilles pointues, deux yeux rouges (`8`, CORAIL), corps
// VIOLET_ORAGE (`F`) au ventre sombre (`1`), ailes membraneuses à pointes, leurs
// doigts en bleu nuit (`1`) et un liseré ARDOISE_PALE (`4`) sur le bord d'attaque :
// c'est lui qui la détache des fonds sombres de la grotte et des égouts. Contour
// sombre de 1 px tout autour. Symétrique : le miroir du rendu ne change rien.
// Image 0 : ailes HAUTES, pointes en l'air.

// `chauve_souris1.png` : Chauve-souris, image 1 : ailes BASSES, pointes vers le bas. La tête et le corps
// (colonnes 6 à 11) ne bougent pas : seules les ailes battent.

// `pigeon0.png` : Pigeon perché (18x14, tourné vers la droite). Carte 6.6 : l'ancien pigeon, une
// masse d'ardoise sombre, ne se lisait pas (ni tête, ni queue) ; celui-ci est un
// pigeon des villes. Corps rond et dodu gris-bleu MOYEN (`4`, ARDOISE_PALE), aile
// repliée plus CLAIRE (`5`, BRUME) barrée de deux traits sombres (`1`), ventre et
// dessous de la queue ombrés (`3`), queue courte au bout sombre ; petite tête
// ronde ardoise (`3`) au front éclairé, œil orange (`A`), bec court sombre à la
// cire claire ; cou irisé : sarcelle (`E`), cyan (`D`) et une touche violette
// (`F`) ; pattes corail clair (`9`). Contour sombre de 1 px tout autour. Rien du
// corbeau (bleu nuit presque noir, gros bec, queue en éventail) ni de l'oiseau
// de décor (petit et blanc, `objets_decor::OISEAU0`), qui est inoffensif.

// `pigeon1.png` : Pigeon perché, 2e image : il hoche la tête (tête plus basse et en avant) ; le
// corps, l'aile et les pattes ne bougent pas.

// `pigeon_annonce.png` : Annonce (0,3 s avant la plongée) : il se ramasse, corps plus bas et tendu vers
// l'avant, tête rentrée, ailes entrouvertes (les rémiges se dressent au-dessus du
// dos), œil rouge corail (`8`) sous un sourcil sombre ; la scène le fait aussi
// trembler d'un pixel.

// `pigeon_plongee0.png` : Plongée, 1re image : il s'élance, ailes grandes ouvertes en V, le corps déjà
// couché sur la diagonale (tête en bas à droite, queue en haut à gauche).

// `pigeon_plongee1.png` : Plongée, 2e image (le reste du piqué) : il fond sur Pépin le long de sa diagonale
// à 45°, tête et bec devant (en bas à droite, œil rouge), ailes relevées et
// repliées en flèche vers l'arrière, rémiges sombres en « doigts » au bout,
// queue derrière (en haut à gauche).

// `pigeon_remontee0.png` : Remontée vers le perchoir, 1re image : ailes levées au-dessus du dos. La scène
// le retourne pour qu'il regarde son perchoir (il revient en arrière).

// `pigeon_remontee1.png` : Remontée, 2e image : ailes rabattues sous le ventre (battement d'ailes).

// `pigeon_ecrase.png` : Pigeon écrasé : aplati, plumes ébouriffées qui dépassent du dos.

// `escargot0.png` : Escargot (16x12, tourné vers la gauche) : « un champignon qui marche ». Sa
// coquille reprend le chapeau du champignon (corail, reflet corail clair, pois
// blancs) : on devine tout de suite qu'il fait rebondir. Corps bleu-gris pâle
// (nuage, ventre brume), œil sombre et joue rose ; deux cornes sombres à
// boule. 1re image de la marche : cornes dressées.

// `escargot1.png` : Escargot, 2e image de la marche : les boules des cornes se penchent vers
// l'arrière d'un pixel (seules les deux rangées du haut changent).

// `escargot_rebond.png` : Rebond : la coquille s'aplatit sous le poids de Pépin (comme le chapeau du
// champignon), les cornes rentrées, l'œil fermé ; il reste mignon.

// `chat_dort0.png` : Chat endormi (16x14, tourné vers la gauche) : roulé en boule, tête posée sur
// ses pattes blanches, yeux fermés, la queue rayée enroulée sous lui. Pelage
// orange (A), rayures et queue corail (8), oreilles roses (9), museau et
// pattes blancs (7). Les rangées du haut restent vides : l'image éveillée,
// assise, est plus haute. 1re image : il inspire (le dos se soulève).

// `chat_dort1.png` : Chat endormi, 2e image : il expire (le dos s'abaisse d'un pixel).

// `chat_reveil.png` : Réveil (l'annonce) : il lève la tête, oreilles dressées, un œil ouvert (or).

// `chat_eveille.png` : Éveillé : assis, dos rond, poil hérissé, queue dressée, grands yeux d'or :
// le toucher tue (même par le dessus).

// `chat_pret.png` : Prêt à bondir (pendant l'annonce, quand Pépin est à portée, carte 6.3) :
// tête basse posée sur les pattes avant tendues, yeux d'or grand ouverts,
// arrière-train levé, queue dressée au bout recourbé. Distinct de l'annonce
// simple (`CHAT_REVEIL`, couché, un œil ouvert).

// `chat_bond.png` : En plein bond (carte 6.3) : corps allongé, pattes avant tendues vers
// l'avant, pattes arrière tendues vers l'arrière, queue en arrière.

// `chat_ecrase.png` : Écrasé : aplati comme une crêpe, oreilles rentrées dans la tête. Il reste
// mignon : yeux plissés en « ^ ^ », petit nez rose, museau blanc.

// `chat_z.png` : « z » du sommeil (6x6) : blanc cerné de sombre, lisible sur tous les ciels.
// La scène en fait monter trois au-dessus du chat endormi, l'un après l'autre.

// `chat_alerte.png` : « ! » de l'annonce du réveil (4x8) : or cerné de sombre ; la scène le fait
// clignoter de plus en plus vite, comme l'annonce d'un éclair.

// `nuage_colere0.png` : Grand Nuage en colère (48x32) : sourcils froncés, yeux d'or, dents serrées.
// Ventre violet d'orage (F), corps ardoise pâle (4), reflet nuage clair (6) sous le
// contour du haut. Il gronde pendant le prélude et lance ses éclairs.

// `nuage_colere1.png` : En colère, 2e image : les sourcils descendent d'un pixel (il fronce).

// `nuage_essouffle0.png` : Essoufflé : paupières lourdes, sourcils tombants, bouche ouverte qui halète
// (langue corail), goutte de sueur. C'est le moment de lui sauter sur la tête.

// `nuage_essouffle1.png` : Essoufflé, 2e image : la bouche se referme un peu (il reprend son souffle).

// `nuage_touche.png` : Touché : yeux plissés « > < », bouche en vague. La scène le fait clignoter.

// `pv_nuage.png` : Point de vie du Grand Nuage dans le HUD (10x6) : un petit nuage plein.

// `pv_nuage_perdu.png` : Point de vie perdu (10x6) : le même petit nuage, éteint (ardoise sombre).

/// Images de la patrouille d'une flaque (3 images).
pub const FLAQUE: [&str; 3] = ["flaque0", "flaque1", "flaque2"];
/// Images d'un corbeau (battement d'ailes).
pub const CORBEAU: [&str; 2] = ["corbeau0", "corbeau1"];
/// Le corbeau du Monde 2 dessiné en chauve-souris (battement d'ailes, carte 9.26b).
pub const CHAUVE_SOURIS: [&str; 2] = ["chauve_souris0", "chauve_souris1"];
/// Pigeon perché : il hoche la tête (2 images).
pub const PIGEON: [&str; 2] = ["pigeon0", "pigeon1"];
/// Pigeon en piqué : il s'élance ailes ouvertes, puis les relève en flèche (2 images).
pub const PIGEON_PLONGEE: [&str; 2] = ["pigeon_plongee0", "pigeon_plongee1"];
/// Pigeon qui remonte à son perchoir : il bat des ailes (2 images).
pub const PIGEON_REMONTEE: [&str; 2] = ["pigeon_remontee0", "pigeon_remontee1"];
/// Escargot qui marche (2 images).
pub const ESCARGOT: [&str; 2] = ["escargot0", "escargot1"];
/// Chat endormi : il respire (2 images).
pub const CHAT_DORT: [&str; 2] = ["chat_dort0", "chat_dort1"];
/// Grand Nuage en colère : il fronce les sourcils (2 images).
pub const NUAGE_COLERE: [&str; 2] = ["nuage_colere0", "nuage_colere1"];
/// Grand Nuage essoufflé : il halète (2 images).
pub const NUAGE_ESSOUFFLE: [&str; 2] = ["nuage_essouffle0", "nuage_essouffle1"];

/// Les sprites de ce fichier : (nom, dessin), lus dans `assets/sprites/ennemis/`
/// (carte 10.4a : les dessins sont des fichiers PNG, voir `super::fichiers`).
pub fn sprites() -> &'static [(&'static str, crate::sprites::Image)] {
    crate::sprites::fichiers::famille("ennemis")
}
