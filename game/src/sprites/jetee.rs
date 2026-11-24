//! La jetée de « La Pieuvre du port » (carte 9.20e) : la grue et la pile de
//! caisses qui bordent l'arène, le quai mouillé et son cordage, l'eau profonde du
//! bassin, et au fond du port le phare rayé et les bateaux amarrés. Ces dessins
//! ne servent qu'à ce niveau (`crate::dessin_port`) ; aucun n'a de collision.

// `jetee_grue.png` : Grue du quai GAUCHE, un tronçon de sa tour en treillis (corail) : deux montants,
// une traverse, une croix. Entre les barres, le ciel et la mer se voient.

// `jetee_grue_pied.png` : Pied de la grue : le bas du treillis sur sa plaque rayée ocre et sombre, posée
// sur la pierre du quai.

// `jetee_grue_cabine.png` : Cabine du grutier, sur la tour : vitre sombre et ses reflets.

// `jetee_caisse_ocre.png` : Caisse ocre à croix, de la pile du quai DROIT ; un cordage la sangle.

// `jetee_caisse_violette.png` : Caisse violette à lattes (avec sa marque claire), de la même pile, sanglée.

// `jetee_quai_mouille.png` : Pierre du quai au niveau du bassin : l'écume bat le haut, la pierre noyée
// est plus sombre que celle du quai.

// `jetee_cordage.png` : Cordage d'amarrage (16 x 28) : il pend de l'anneau du quai jusque dans l'eau.

// `jetee_eau_haut.png` : Bassin de la Pieuvre, surface : la ligne claire et la bande cyan sombre des
// tuiles d'eau (l'ombre de la pieuvre s'y lit toujours), puis l'eau PROFONDE,
// d'ardoise de nuit.

// `jetee_eau_vague.png` : La même surface, 1 px plus bas (l'autre temps de la vague).

// `jetee_eau_plein.png` : Bassin, sous la surface : l'eau profonde.

// `jetee_phare.png` : Le phare rayé blanc et corail (12 x 48), au fond du port ; la lampe est
// dessinée à part (elle clignote).

// `jetee_chalutier.png` : Chalutier amarré (40 x 26), silhouette sombre : coque, timonerie et ses
// hublots, mât, portique et filins du chalut. Le bas est la ligne de flottaison.

// `jetee_barque.png` : Barque à voile ferlée (20 x 14), silhouette sombre.

// `jetee_canot.png` : Canot (16 x 5), silhouette sombre.

// `port_bateau.png` : « Le port » (carte 9.11) : un bateau en va-et-vient (32 x 15), dessiné sur une
// plateforme mobile horizontale. Les 8 premières rangées couvrent sa collision
// (32 x 8) : le pont de bois, clair sur le dessus (on s'y pose), puis la coque
// corail et sa bande blanche ; le bas, sombre, est la ligne de flottaison.

// `port_caisse_grue.png` : « Le port » (carte 9.11) : la caisse de la grue (32 x 16), dessinée sur une
// plateforme mobile verticale. Les 8 rangées du haut sont les élingues qui
// montent au crochet (le câble part de là vers le haut, `dessin_port`) ; les 8
// du bas couvrent la collision : la caisse de bois cerclée, claire sur le dessus.

/// Les sprites de ce fichier : (nom, dessin), lus dans `assets/sprites/jetee/`
/// (carte 10.4a : les dessins sont des fichiers PNG, voir `super::fichiers`).
pub fn sprites() -> &'static [(&'static str, crate::sprites::Image)] {
    crate::sprites::fichiers::famille("jetee")
}
