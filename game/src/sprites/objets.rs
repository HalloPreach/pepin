//! Sprites des objets : gouttes d'or, lanternes, champignons, pointes, plateformes,
//! panneaux du tutoriel et drapeau d'arrivée.

// `lettre_p.png` : Lettre cachée (carte 8.6) 10x10 : un jeton VIOLET cerclé d'or (la goutte est un
// dessin d'or sans violet), la lettre en or pâle ; 2 px de contour (ombre puis or) comme
// la goutte, pour rester lisible sur tous les ciels. La boîte de ramassage (8x8) est au
// milieu. Les deux P du mot ont le même dessin.

// `goutte0.png` : Goutte d'or 10x10 : la goutte de 8x8 (boîte de ramassage) cernée de 2 px de contour sombre
// (le contour d'origine + un anneau), pour rester lisible devant tous les ciels, même dorés.

// `lanterne0.png` : Lanterne éteinte : vitre claire (ARDOISE_PALE/BRUME) pour se lire devant les immeubles du fond.

// `friable_ardoise0.png` : Plateforme friable des niveaux 1 à 3 : dalle d'ardoise claire à angles écornés, avec une fine fêlure.

// `friable_ardoise1.png` : Dalle d'ardoise qui s'effrite : la fêlure s'ouvre de part en part.

// `mobile.png` : Plateforme mobile 32x8 : le dessin couvre toute la boîte de collision (8 px de haut).

// `aiguille.png` : Aiguille d'horloge 32x8 (carte 5.10) : une plaque de laiton qui tourne autour
// de son cadran. Même taille que la plateforme mobile, le dessin couvre toute la
// boîte de collision ; dessus or pâle (le bord où l'on se tient), ombre orange
// dessous, trois rivets (celui du milieu tient la tige des maillons).

// `cadran.png` : Cadran d'horloge 32x32, centré sur le pivot `h` d'une aiguille : contour sombre,
// bord doré (ombré orange en bas à droite), filet sombre, fond blanc cassé,
// douze graduations (12, 3, 6 et 9 plus grosses) et le moyeu doré du pivot.
// Dessin seulement : aucune collision.

// `crochet.png` : Crochet du balancement (carte 11.1, prototype) : un anneau de fer 10x8, gris
// ardoise (reflet pâle en haut à gauche), le trou transparent ; centré sur le point d'accroche
// (le milieu de la case `y`), la corde passe dedans.
// `crochet_allume.png` : Le même anneau, doré : la crosse de Pépin est à sa portée.
// `maillon.png` : Maillon 4x4 de la tige qui relie le moyeu du cadran à l'aiguille : posés en
// ligne du pivot jusqu'au milieu de la plaque, ils tournent avec elle comme une
// aiguille de montre (chaque maillon est posé à une position entière, jamais
// tourné ni agrandi).

// `panneau_barque.png` : Panneau « barque » (carte 9.6c, lettre `Z`) : la flèche du BAS plonge dans le
// parapluie retourné (la coque corail, son bord clair). Sans mot.

// `grelon0.png` : Grêlon 4x4 : une bille de glace de 2x2 px (reflet blanc cassé, nuage, ombre de
// brume) cernée d'un contour sombre. Sur les ciels sombres la glace claire se
// détache, sur les ciels dorés c'est le contour. Quatre images : le reflet fait le
// tour de la bille (elle roule en tombant).

// `grele_eclat.png` : Éclat d'un grêlon qui se brise sur ce qui l'arrête (toit, parapluie ouvert) :
// deux miettes de glace d'un pixel qui s'écartent, chacune cernée de sombre.

// `grele_nuage0.png` : Petit nuage gris d'où tombe la grêle (carte 8.18), 24x11 : du DÉCOR, posé en haut
// de chaque colonne de grêle. Gris d'orage (ardoise claire), bosses éclairées par le
// haut (liseré de brume puis ardoise pâle : c'est lui qui le détache des ciels violets
// et ardoise), ventre plus sombre (ardoise) qui pend en deux bosses, d'où sortent les
// grêlons ; une mèche effilochée de chaque côté. Rien à voir avec les nuages où l'on
// se pose (blancs ou roses, en tuiles) : plus petit, plus sombre, il flotte. Deux
// images : les mèches et les bosses du ventre bougent un peu (le nuage s'effiloche).

// `auvent0.png` : Auvent de boutique (lettre `A`, 16x15), vu de profil : un mât de métal sombre
// (« 030 »), la toile en pente haute à gauche (rayée corail / blanc cassé /
// corail), le lambrequin au bout (rayures verticales, bas festonné) et une jambe
// de force. La pente dit d'où part le rebond : vers le haut et vers la droite.

// `auvent1.png` : Auvent enfoncé : la toile plie au milieu sous le poids de Pépin et le
// lambrequin descend d'un pixel (image du rebond).

// `facade_vitrine.png` : Façade de boutique (16x15) : la vitrine allumée derrière le mât d'un auvent
// (carte 5.12). Dessin SEUL, aucune collision : posée dans la case à gauche de
// l'auvent, à partir du sommet du mât (1 px sous le haut de la case, voir
// `SOMMET_DU_MAT` dans `scene.rs`), elle accroche la toile à une boutique.
//
// Carte 5.12b : elle n'a plus de « dessus ». Sa première version finissait par
// une corniche claire et plate 4 px AU-DESSUS des toits voisins : vue de loin,
// une tour d'une tuile où l'on croyait pouvoir se poser, au-dessus du vide.
// Désormais :
// - un toit en APPENTIS adossé au mât : le faîte (en haut à droite) est au
// sommet du mât, la pente descend vers la gauche d'un pixel tous les deux
// pixels, comme la toile descend vers la droite. Les deux font un seul toit
// dont le haut est l'auvent : le seul endroit où l'on se pose. Le faîte est
// 1 px SOUS le dessus des toits voisins. Dans ce jeu, tout ce qui porte Pépin
// est PLAT avec une arête claire ; une pente sombre n'est jamais un sol ;
// - les teintes du fond (ardoise ombre et ardoise nuit, comme les immeubles
// de l'arrière-plan) : rien de clair au sommet, la seule lumière est la
// vitrine orange (deux carreaux, des étalages en silhouette), sous l'avant-toit.
//
// Ni or (gouttes, éclairs), ni corail (parapluie, toile de l'auvent), ni cyan
// (flaques, pluie).

// `facade_mur.png` : Mur de la façade de boutique (16x16), répété sous la vitrine jusqu'au premier
// solide (ou au bas du niveau) : bleu nuit, fenêtres éteintes. Plus sombre que
// les tuiles, sans briques : on voit que c'est un fond, pas un mur où se poser.

// `ventilateur_arret.png` : Ventilateur : un ventilateur à pied sur sa casette (socle 16x8, en bas de la
// case). Trois états : à l'arrêt (pale dorée, rien au-dessus), à l'annonce
// (la lampe orange clignote au-dessus de la cage, comme l'annonce d'un éclair)
// et en souffle (le courant d'air cyan remonte, les pales tournent). Le
// courant ne porte qu'en phase de souffle : la boîte SOLIDE est la casette.

// `ventilateur_aveugle.png` : Ventilateur à l'annonce : la lampe orange clignote (c'est le signal, comme
// pour un éclair : « le souffle va reprendre »).

// `ventilateur_souffle.png` : Ventilateur en souffle : le courant d'air (cyan) remonte au-dessus de la
// cage, les pales sont en mouvement (image du courant actif).

// `montgolfiere_repos.png` : Montgolfière (20 x 40) : l'enveloppe à fuseaux cyan et blancs (le fuseau de
// droite dans l'ombre), les cordes, puis la NACELLE (16 x 8, rangées 32 à 39 : sa
// boîte solide ; le dessus est la rangée 32). Brûleur éteint : au repos, en
// redescente, ou arrêtée en haut. On la dessine 2 px à gauche de la nacelle et
// 32 px au-dessus (le ballon déborde de 2 px de chaque côté).

// `montgolfiere_feu0.png` : Montgolfière qui MONTE : la flamme du brûleur éclaire la jupe de l'enveloppe
// (image 1 sur 2 ; la flamme vacille d'une image à l'autre).

// `montgolfiere_feu1.png` : Montgolfière qui monte : image 2 sur 2 de la flamme.

/// Goutte d'or : 4 images de brillance.
pub const GOUTTE: [&str; 4] = ["goutte0", "goutte1", "goutte2", "goutte3"];
/// Nom du sprite de la lettre cachée `lettre` (carte 8.6) : P, É, I ou N.
pub fn lettre(lettre: char) -> &'static str {
    match lettre {
        'P' => "lettre_p",
        'É' => "lettre_e",
        'I' => "lettre_i",
        _ => "lettre_n",
    }
}
/// Lanterne : éteinte puis allumée.
pub const LANTERNE: [&str; 2] = ["lanterne0", "lanterne1"];
/// Champignon-trampoline : détendu puis comprimé.
pub const CHAMPIGNON: [&str; 2] = ["champignon0", "champignon1"];
/// Auvent : toile tendue puis enfoncée (rebond).
pub const AUVENT: [&str; 2] = ["auvent0", "auvent1"];
/// Façade de boutique derrière un auvent : la vitrine, puis le mur en dessous.
pub const FACADE: [&str; 2] = ["facade_vitrine", "facade_mur"];
/// Ventilateur : arrêt, annonce (lampe clignotante), souffle (courant actif).
pub const VENTILATEUR: [&str; 3] = [
    "ventilateur_arret",
    "ventilateur_aveugle",
    "ventilateur_souffle",
];
/// Montgolfière : brûleur éteint, puis les deux images de la flamme (montée).
pub const MONTGOLFIERE: [&str; 3] = [
    "montgolfiere_repos",
    "montgolfiere_feu0",
    "montgolfiere_feu1",
];
/// Plateforme friable (nuages, niveaux 4 et 5) : intacte puis fissurée.
pub const FRIABLE: [&str; 2] = ["friable0", "friable1"];
/// Plateforme friable (dalle d'ardoise, niveaux 1 à 3) : intacte puis fendue.
pub const FRIABLE_ARDOISE: [&str; 2] = ["friable_ardoise0", "friable_ardoise1"];
/// Drapeau d'arrivée (2 images).
pub const DRAPEAU: [&str; 2] = ["drapeau0", "drapeau1"];
/// Grêlon qui roule en tombant (4 images, le reflet fait le tour).
pub const GRELON: [&str; 4] = ["grelon0", "grelon1", "grelon2", "grelon3"];
/// Petit nuage de grêle (carte 8.18) : 2 images, ses mèches s'effilochent.
pub const GRELE_NUAGE: [&str; 2] = ["grele_nuage0", "grele_nuage1"];

/// Les sprites de ce fichier : (nom, dessin), lus dans `assets/sprites/objets/`
/// (carte 10.4a : les dessins sont des fichiers PNG, voir `super::fichiers`).
pub fn sprites() -> &'static [(&'static str, crate::sprites::Image)] {
    crate::sprites::fichiers::famille("objets")
}
