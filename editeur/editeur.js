// Éditeur de niveaux de PÉPIN (carte 10.3a), site de TEST seulement.
//
// La légende (lettres, catégories, couleurs, thèmes, genres) est lue dans
// `legende.json`, écrit par le crate Rust `niveaux` (`--bin editeur`) : aucune
// lettre n'est recopiée ici. Lecture et écriture des fichiers : `ron.js`
// (format canonique de l'exporteur Rust : un fichier ouvert puis exporté sans
// modification redonne les mêmes octets).
//
// L'éditeur ne VALIDE pas définitivement un niveau : il avertit des erreurs
// courantes ; le juge reste le chargeur du jeu : boutons Valider et TESTER (carte
// 10.3b, plus bas : le jeu de QA vérifie et joue le niveau), ou
// `./docker-cargo.sh run -p niveaux --bin valider -- fichier.ron`.
import { lireNiveau, ecrireNiveau, rangees, versAscii } from './ron.js';

const $ = (s) => document.querySelector(s);
const etat = {
  legende: null,       // contenu de legende.json
  parLettre: new Map(),
  niveau: null,        // l'objet rendu par lireNiveau (ascii remplacé par `grille`)
  grille: [],          // tableau de rangées, chacune un tableau de caractères
  lettre: '#',
  outil: 'pinceau',
  zoom: 16,
  passe: [],           // pile « annuler » (instantanés JSON)
  futur: [],           // pile « refaire »
  modifie: false,
  faute: null,         // case fautive dite par le jeu : {x, y} (x null : rangée entière)
  robot: null,         // dernier résultat du robot (carte 10.3c), voir robot-worker.js
};

// --- Chargement --------------------------------------------------------------

async function demarrer() {
  etat.legende = await (await fetch('legende.json')).json();
  for (const l of etat.legende.lettres) etat.parLettre.set(l.lettre, l);
  construirePalettes();
  remplirListes();
  try {
    const index = await (await fetch('niveaux/index.json')).json();
    for (const chemin of index) {
      const o = document.createElement('option');
      o.value = chemin;
      o.textContent = chemin;
      $('#liste-niveaux').append(o);
    }
  } catch { /* pas d'index : on peut toujours ouvrir un fichier local */ }
  const demande = new URLSearchParams(location.search).get('niveau');
  if (demande) await ouvrirDuJeu(demande);
  else if (!reprendreBrouillon()) nouveau();
  brancher();
}

async function ouvrirDuJeu(chemin) {
  const reponse = await fetch(`niveaux/${chemin}`);
  if (!reponse.ok) { signaler(`impossible d'ouvrir ${chemin}`); return; }
  ouvrirTexte(await reponse.text(), chemin);
  $('#liste-niveaux').value = chemin;
}

function ouvrirTexte(texte, nomFichier) {
  let niveau;
  try {
    niveau = lireNiveau(texte);
  } catch (e) {
    signaler(`${nomFichier} : ${e.message}`, true);
    return;
  }
  etat.niveau = niveau;
  etat.grille = rangees(niveau.ascii).map((r) => [...r]);
  etat.passe = [];
  etat.futur = [];
  etat.modifie = false;
  signaler(`ouvert : ${nomFichier}`);
  toutRafraichir();
}

/// Un niveau vide, jouable tel quel une fois les 3 gouttes posées : un sol, un départ, une arrivée.
function nouveau() {
  const largeur = 60, hauteur = 12;
  const grille = Array.from({ length: hauteur }, (_, y) =>
    Array.from({ length: largeur }, () => (y >= hauteur - 2 ? (y === hauteur - 2 ? '#' : 'B') : '.')));
  grille[hauteur - 3][2] = '@';
  grille[hauteur - 3][largeur - 4] = 'X';
  etat.niveau = {
    entete: [], commentaires: {}, commentairesFin: [],
    monde: 1, numero: 0, cle: 'mon_essai', nom: 'Mon niveau',
    theme: etat.legende.themes[0], genre: 'Normal', sans_planer: false, duree_cible_s: 60,
    intro: 'Une phrase pour la carte du niveau.', mecanique: 'Ce que le niveau apprend.',
    aiguilles: [], poissons_hauts: [], ascii: '',
  };
  etat.grille = grille;
  etat.passe = [];
  etat.futur = [];
  etat.modifie = false;
  signaler('nouveau niveau (rangez-le dans essais/ pour l\'essayer)');
  toutRafraichir();
}

// --- Annuler / refaire ---------------------------------------------------------

const instantane = () => JSON.stringify({ niveau: etat.niveau, grille: etat.grille });
function memoriser() {
  etat.passe.push(instantane());
  if (etat.passe.length > 200) etat.passe.shift();
  etat.futur = [];
  etat.modifie = true;
  etat.faute = null; // la grille change : l'ancienne faute n'est plus sûre
  etat.robot = null;  // ... et l'ancien chemin du robot non plus
}
function restaurer(depuis, vers) {
  if (!depuis.length) return;
  vers.push(instantane());
  const s = JSON.parse(depuis.pop());
  etat.niveau = s.niveau;
  etat.grille = s.grille;
  etat.modifie = true;
  toutRafraichir();
}

// --- Dessin de la grille ----------------------------------------------------------

const toile = $('#toile');
const ctx = toile.getContext('2d');
const largeur = () => (etat.grille[0] || []).length;
const hauteur = () => etat.grille.length;

function dessiner() {
  const z = etat.zoom;
  toile.width = largeur() * z;
  toile.height = hauteur() * z;
  const vide = etat.parLettre.get('.');
  ctx.fillStyle = vide ? vide.couleur : '#20263a';
  ctx.fillRect(0, 0, toile.width, toile.height);
  const lettres = $('#lettres').checked && z >= 10;
  ctx.font = `bold ${Math.floor(z * 0.75)}px monospace`;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  for (let y = 0; y < hauteur(); y++) {
    for (let x = 0; x < largeur(); x++) {
      const c = etat.grille[y][x];
      if (c === '.') continue;
      const l = etat.parLettre.get(c);
      ctx.fillStyle = l ? l.couleur : '#ff00ff';
      if (l && !l.solide) {
        // Les cases libres (objets, ennemis) : un carré inscrit, le fond reste visible.
        ctx.fillRect(x * z + 1, y * z + 1, z - 2, z - 2);
      } else {
        ctx.fillRect(x * z, y * z, z, z);
      }
      if (lettres || !l) {
        ctx.fillStyle = l && clair(l.couleur) ? '#000' : '#fff';
        ctx.fillText(c, x * z + z / 2, y * z + z / 2 + 1);
      }
    }
  }
  // Quadrillage léger, plus marqué toutes les 10 cases.
  if (z >= 8) {
    for (let x = 0; x <= largeur(); x++) {
      ctx.fillStyle = x % 10 === 0 ? 'rgba(255,255,255,.25)' : 'rgba(255,255,255,.06)';
      ctx.fillRect(x * z, 0, 1, toile.height);
    }
    for (let y = 0; y <= hauteur(); y++) {
      ctx.fillStyle = y % 10 === 0 ? 'rgba(255,255,255,.25)' : 'rgba(255,255,255,.06)';
      ctx.fillRect(0, y * z, toile.width, 1);
    }
  }
  // Carte 10.3b : la case fautive trouvée par le chargeur Rust, cerclée de rouge.
  if (etat.faute && etat.faute.y < hauteur()) {
    const { x, y } = etat.faute;
    ctx.fillStyle = 'rgba(255,40,40,.35)';
    ctx.strokeStyle = '#ff3030';
    ctx.lineWidth = 3;
    if (x === null) {
      ctx.fillRect(0, y * z, toile.width, z);
      ctx.strokeRect(1, y * z, toile.width - 2, z);
    } else {
      ctx.fillRect(x * z, y * z, z, z);
      ctx.strokeRect(x * z - 2, y * z - 2, z + 4, z + 4);
    }
  }
  dessinerRobot(z);
  if (rectangle) {
    const [x0, y0, x1, y1] = bornes(rectangle);
    ctx.strokeStyle = '#f0c020';
    ctx.lineWidth = 2;
    ctx.strokeRect(x0 * z, y0 * z, (x1 - x0 + 1) * z, (y1 - y0 + 1) * z);
  }
}

function clair(couleur) {
  const n = parseInt(couleur.slice(1), 16);
  return ((n >> 16) * 299 + ((n >> 8) & 255) * 587 + (n & 255) * 114) / 1000 > 140;
}

// --- Outils ------------------------------------------------------------------

let trace = false;     // bouton de la souris enfoncé (pinceau / gomme)
let rectangle = null;  // {x0, y0, x1, y1} pendant un tracé de rectangle

function caseSous(e) {
  const r = toile.getBoundingClientRect();
  const x = Math.floor((e.clientX - r.left) / etat.zoom);
  const y = Math.floor((e.clientY - r.top) / etat.zoom);
  return x >= 0 && y >= 0 && x < largeur() && y < hauteur() ? { x, y } : null;
}
const bornes = (r) => [Math.min(r.x0, r.x1), Math.min(r.y0, r.y1), Math.max(r.x0, r.x1), Math.max(r.y0, r.y1)];

/// Pose `c` en (x, y). Une lettre limitée à « au plus N » (départ, arrivée,
/// boss...) : au-delà de N, la plus ancienne est effacée (remplacée par du vide).
function poser(x, y, c) {
  if (etat.grille[y][x] === c) return;
  const l = etat.parLettre.get(c);
  if (l && l.au_plus) {
    const cases = [];
    etat.grille.forEach((r, yy) => r.forEach((cc, xx) => { if (cc === c) cases.push([xx, yy]); }));
    while (cases.length >= l.au_plus) {
      const [xx, yy] = cases.shift();
      etat.grille[yy][xx] = '.';
    }
  }
  etat.grille[y][x] = c;
}

function appliquer(p) {
  if (etat.outil === 'pipette') { choisir(etat.grille[p.y][p.x]); choisirOutil('pinceau'); return; }
  poser(p.x, p.y, etat.outil === 'gomme' ? '.' : etat.lettre);
  dessiner();
}

toile.addEventListener('mousedown', (e) => {
  const p = caseSous(e);
  if (!p) return;
  e.preventDefault();
  // Clic droit (carte 10.3b) : « partir d'ici », essai avec Pépin posé sur cette
  // case. Alt + clic : la pipette (aussi l'outil Pipette, touche I).
  if (e.button === 2) { partirDIci(p); return; }
  if (e.altKey) { choisir(etat.grille[p.y][p.x]); return; }
  if (etat.outil === 'rectangle') {
    memoriser();
    rectangle = { x0: p.x, y0: p.y, x1: p.x, y1: p.y };
    dessiner();
    return;
  }
  if (etat.outil === 'pipette') { appliquer(p); return; }
  memoriser();
  trace = true;
  appliquer(p);
});
toile.addEventListener('contextmenu', (e) => e.preventDefault());
toile.addEventListener('mousemove', (e) => {
  const p = caseSous(e);
  montrerCase(p);
  if (!p) return;
  if (rectangle) { rectangle.x1 = p.x; rectangle.y1 = p.y; dessiner(); return; }
  if (trace && etat.outil !== 'pipette') appliquer(p);
});
window.addEventListener('mouseup', () => {
  if (rectangle) {
    const [x0, y0, x1, y1] = bornes(rectangle);
    for (let y = y0; y <= y1; y++) for (let x = x0; x <= x1; x++) poser(x, y, etat.lettre);
    rectangle = null;
    toutRafraichir();
  } else if (trace) {
    trace = false;
    toutRafraichir();
  }
});

function montrerCase(p) {
  if (!p) { $('#sous-la-souris').innerHTML = '&nbsp;'; return; }
  const c = etat.grille[p.y][p.x];
  const l = etat.parLettre.get(c);
  // Colonne et rangée comptées depuis 0 (comme `poissons_hauts`) et, entre
  // parenthèses, la ligne et la colonne de la grille comptées depuis 1 (comme
  // les messages d'erreur du jeu).
  $('#sous-la-souris').textContent =
    `colonne ${p.x}, rangée ${p.y} (erreur du jeu : ligne ${p.y + 1}, colonne ${p.x + 1}) : « ${c} » ${l ? l.nom : 'lettre inconnue'}`;
}

function choisir(c) {
  montrerChoisie(c);
  if (etat.outil === 'gomme' || etat.outil === 'pipette') choisirOutil('pinceau');
}

function montrerChoisie(c) {
  etat.lettre = c;
  document.querySelectorAll('.palette button').forEach((b) => b.classList.toggle('actif', b.dataset.lettre === c));
  const l = etat.parLettre.get(c);
  $('#choisie').textContent = l
    ? `« ${c} » ${l.nom}${l.largeur > 1 || l.hauteur > 1 ? ` (occupe ${l.largeur} x ${l.hauteur} tuiles)` : ''}${l.au_plus ? `, au plus ${l.au_plus}` : ''}${l.exactement ? `, exactement ${l.exactement}` : ''}${l.appui !== 'libre' ? `, appui : ${l.appui.replace(/_/g, ' ')}` : ''}`
    : `« ${c} » (inconnue)`;
}

function choisirOutil(o) {
  etat.outil = o;
  document.querySelectorAll('#boutons-outils button').forEach((b) => b.classList.toggle('actif', b.dataset.outil === o));
}

// --- Taille de la grille ----------------------------------------------------------

function redimensionner(sens) {
  const cote = sens.slice(0, -1), plus = sens.endsWith('+');
  if (!plus && ((cote === 'gauche' || cote === 'droite') ? largeur() : hauteur()) <= 1) return;
  memoriser();
  const g = etat.grille;
  const rangeeVide = () => Array.from({ length: largeur() }, () => '.');
  if (cote === 'gauche') g.forEach((r) => (plus ? r.unshift('.') : r.shift()));
  if (cote === 'droite') g.forEach((r) => (plus ? r.push('.') : r.pop()));
  if (cote === 'haut') { if (plus) g.unshift(rangeeVide()); else g.shift(); }
  if (cote === 'bas') { if (plus) g.push(rangeeVide()); else g.pop(); }
  // Les poissons sont repérés par leur case : ils suivent la grille.
  const dx = cote === 'gauche' ? (plus ? 1 : -1) : 0;
  const dy = cote === 'haut' ? (plus ? 1 : -1) : 0;
  for (const p of etat.niveau.poissons_hauts) { p.colonne += dx; p.rangee += dy; }
  toutRafraichir();
}

// --- Avertissements (le juge final reste le chargeur du jeu) ------------------------

function avertissements() {
  const liste = [];
  const g = etat.grille, n = etat.niveau;
  const compte = new Map();
  const cases = new Map();
  g.forEach((r, y) => r.forEach((c, x) => {
    compte.set(c, (compte.get(c) || 0) + 1);
    if (!cases.has(c)) cases.set(c, []);
    cases.get(c).push([x, y]);
  }));
  const nb = (c) => compte.get(c) || 0;
  if (nb('@') !== 1) liste.push([`il faut exactement un départ @ (il y en a ${nb('@')})`, true]);
  // Arrivée : le parseur de `core` (level/ascii.rs). Un boss (Grand Nuage O ou
  // Pieuvre 8) finit le niveau : pas de drapeau X avec lui ; sinon exactement un X.
  const boss = nb('O') + nb('8');
  if (boss > 1) liste.push([`un seul boss par niveau (O ou 8) : il y en a ${boss}`, true]);
  if (boss === 0 && nb('X') === 0) liste.push(['pas d\'arrivée X (seule une arène de boss O ou 8 s\'en passe)', true]);
  if (boss > 0 && nb('X') > 0) liste.push(['arène de boss : pas de drapeau X, vaincre le boss finit le niveau', true]);
  if (nb('8') === 0) {
    for (const c of ['_', '&', '$', '!', '+']) {
      if (nb(c) > 0) liste.push([`« ${c} » ${etat.parLettre.get(c).nom} sans la Pieuvre 8`, true]);
    }
  }
  for (const l of etat.legende.lettres) {
    if (l.au_plus && nb(l.lettre) > l.au_plus) liste.push([`« ${l.lettre} » ${l.nom} : au plus ${l.au_plus} (il y en a ${nb(l.lettre)})`, true]);
    if (l.exactement && nb(l.lettre) !== l.exactement && n.genre === 'Normal') liste.push([`« ${l.lettre} » ${l.nom} : il en faut ${l.exactement} (il y en a ${nb(l.lettre)})`, false]);
  }
  for (const [c, lc] of cases) {
    const l = etat.parLettre.get(c);
    if (!l) { liste.push([`lettre inconnue « ${c} » en colonne ${lc[0][0]}, rangée ${lc[0][1]}`, true]); continue; }
    if (l.appui !== 'libre') {
      const solide = (x, y) => y >= 0 && y < g.length && Boolean(etat.parLettre.get(g[y][x])?.solide);
      const eau = (x, y) => y >= 0 && y < g.length && etat.parLettre.get(g[y][x])?.categorie === 'eau';
      const tenue = {
        solide_dessous: (x, y) => solide(x, y + 1),
        solide_ou_eau_dessous: (x, y) => solide(x, y + 1) || eau(x, y + 1),
        solide_dessus: (x, y) => solide(x, y - 1),
        eau_dessous: (x, y) => eau(x, y + 1),
      }[l.appui];
      const besoin = {
        solide_dessous: 'pas de solide dessous', solide_ou_eau_dessous: 'ni solide ni eau dessous',
        solide_dessus: 'pas de solide au-dessus', eau_dessous: 'pas d\'eau dessous',
      }[l.appui];
      for (const [x, y] of lc) {
        if (tenue && !tenue(x, y)) {
          liste.push([`« ${c} » ${l.nom} en colonne ${x}, rangée ${y} : ${besoin}`, false]);
          break;
        }
      }
    }
  }
  // Les règles de `core/src/zones.rs` qui se voient sans le robot (avertissements doux).
  const solideZones = (x, y) => {
    if (x < 0 || x >= largeur()) return true;
    if (y < 0 || y >= g.length) return false;
    return Boolean(etat.parLettre.get(g[y][x])?.solide) && etat.parLettre.get(g[y][x]).categorie === 'tuile'
      && !['F', '%', '_', 'm'].includes(g[y][x]);
  };
  // Poche de vide close : coiffée d'un solide et bordée de solide jusqu'en bas (`aucune_poche_de_vide_close`).
  poches: for (let y = 1; y < g.length; y++) {
    for (let x = 0; x < largeur(); x++) {
      if (solideZones(x, y) || !solideZones(x, y - 1)) continue;
      let fin = y;
      while (fin < g.length && !solideZones(x, fin)) fin++;
      let close = true;
      for (let r = y; r < fin; r++) if (!solideZones(x - 1, r) || !solideZones(x + 1, r)) close = false;
      if (close) { liste.push([`poche de vide close en colonne ${x}, rangées ${y} à ${fin - 1} (inaccessible)`, false]); break poches; }
    }
  }
  // Pigeon à moins de 4 tuiles d'un oiseau de décor (`..._loin_des_oiseaux_de_decor`).
  for (const [px, py] of cases.get('p') || []) {
    for (const [ox, oy] of cases.get('K') || []) {
      if (Math.max(Math.abs(px - ox), Math.abs(py - oy)) < 4) {
        liste.push([`pigeon (${px}, ${py}) à moins de 4 tuiles de l'oiseau de décor (${ox}, ${oy})`, false]);
      }
    }
  }
  // Ennemi ou piège sur un passage obligé : sur une case voisine du départ ou d'une
  // lanterne, là où Pépin apparaît (`dangers_sur_les_lanternes`, en plus simple).
  const reprises = [...(cases.get('@') || []), ...(cases.get('L') || [])];
  for (const [c, lc] of cases) {
    const l = etat.parLettre.get(c);
    if (!l || (l.categorie !== 'ennemi' && c !== '^')) continue;
    for (const [x, y] of lc) {
      const proche = reprises.find(([rx, ry]) => Math.abs(rx - x) <= 1 && Math.abs(ry - y) <= 1);
      if (proche) liste.push([`« ${c} » ${l.nom} (${x}, ${y}) tout près d'un point de reprise (${proche[0]}, ${proche[1]})`, false]);
    }
  }
  if (nb('h') !== n.aiguilles.length) liste.push([`${nb('h')} pivot(s) h pour ${n.aiguilles.length} aiguille(s) : il en faut autant`, true]);
  for (const p of n.poissons_hauts) {
    if (!g[p.rangee] || g[p.rangee][p.colonne] !== 'i') liste.push([`poisson_haut (${p.colonne}, ${p.rangee}) : pas de poisson i sur cette case`, true]);
  }
  if (!/^[a-z0-9_]+$/.test(n.cle)) liste.push(['cle : lettres minuscules, chiffres et _ seulement (c\'est aussi le nom du fichier)', true]);
  const ul = $('#avertissements');
  ul.innerHTML = '';
  if (!liste.length) {
    const li = document.createElement('li');
    li.className = 'ok';
    li.textContent = 'Aucun avertissement. Le chargeur du jeu reste le juge : Valider ou TESTER.';
    ul.append(li);
  }
  for (const [texte, grave] of liste) {
    const li = document.createElement('li');
    li.textContent = texte;
    if (grave) li.className = 'erreur';
    ul.append(li);
  }
}

// --- Formulaire des propriétés ---------------------------------------------------

const CHAMPS_FORM = ['monde', 'numero', 'cle', 'nom', 'theme', 'genre', 'sans_planer', 'duree_cible_s', 'intro', 'mecanique'];

function remplirListes() {
  const f = $('#formulaire');
  for (const [nom, valeurs] of [['theme', etat.legende.themes], ['genre', etat.legende.genres]]) {
    for (const v of valeurs) {
      const o = document.createElement('option');
      o.value = v;
      o.textContent = v;
      f.elements[nom].append(o);
    }
  }
}

function remplirFormulaire() {
  const f = $('#formulaire'), n = etat.niveau;
  for (const nom of CHAMPS_FORM) {
    const champ = f.elements[nom];
    if (champ.type === 'checkbox') champ.checked = n[nom];
    else champ.value = n[nom];
  }
  tableau('#aiguilles', n.aiguilles, ['rayon', 'vitesse', 'phase']);
  tableau('#poissons', n.poissons_hauts, ['colonne', 'rangee', 'hauteur']);
  const com = [...(n.entete || []), ...Object.entries(n.commentaires || {}).flatMap(([c, l]) => l.map((x) => `${x}   (avant ${c})`))];
  $('#commentaires').textContent = com.length ? com.join('\n') : '(aucun ; ceux du fichier sont gardés à l\'export)';
}

function tableau(sel, lignes, colonnes) {
  const tbody = $(`${sel} tbody`);
  tbody.innerHTML = '';
  lignes.forEach((ligne, i) => {
    const tr = document.createElement('tr');
    for (const c of colonnes) {
      const td = document.createElement('td');
      const input = document.createElement('input');
      input.type = 'number';
      input.step = 'any';
      input.value = ligne[c];
      input.addEventListener('change', () => { memoriser(); ligne[c] = Number(input.value); avertissements(); majEtat(); });
      td.append(input);
      tr.append(td);
    }
    const td = document.createElement('td');
    const x = document.createElement('button');
    x.textContent = '×';
    x.title = 'retirer';
    x.addEventListener('click', () => { memoriser(); lignes.splice(i, 1); toutRafraichir(); });
    td.append(x);
    tr.append(td);
    tbody.append(tr);
  });
}

// --- Export -------------------------------------------------------------------

function texteExporte() {
  return ecrireNiveau({ ...etat.niveau, ascii: versAscii(etat.grille.map((r) => r.join(''))) });
}

function exporter() {
  const texte = texteExporte();
  const a = document.createElement('a');
  a.href = URL.createObjectURL(new Blob([texte], { type: 'text/plain;charset=utf-8' }));
  a.download = `${etat.niveau.cle}.ron`;
  document.body.append(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(a.href), 1000);
  etat.modifie = false;
  signaler(`exporté : ${etat.niveau.cle}.ron (à ranger dans assets/levels/essais/ ou monde_N/)`);
}

// --- Essai et validation par le jeu (carte 10.3b) ------------------------------
//
// L'éditeur ne juge pas lui-même : il pose le RON dans le localStorage (clé
// `pepin.niveau_essai`, la même origine que le jeu de QA) et charge le jeu
// (`index.html?essai=1`) dans un cadre. Le jeu passe le texte au chargeur Rust
// (`niveaux::charger_fichier`, celui des fichiers embarqués) puis répond par
// postMessage (game/src/essai_externe.rs, web/index.html) :
//   {etat: 'erreur', message, ligne, colonne, ligne_fichier}  niveau refusé ;
//   {etat: 'pret'}   niveau accepté, la partie commence ;
//   {etat: 'fini', raison: 'echap' | 'fin'}   retour à l'éditeur.

const CLE_ESSAI = 'pepin.niveau_essai';
let essai = null; // {cadre, jouer, minuterie} pendant un essai ou une validation

/// Valide (jouer = false) ou joue (jouer = true) le niveau `texte`.
function lancerEssai(texte, jouer) {
  fermerEssai();
  try {
    localStorage.setItem(CLE_ESSAI, texte);
  } catch {
    montrerVerdict('localStorage refusé par le navigateur : essai impossible', false);
    return;
  }
  const zone = $('#essai');
  const cadre = document.createElement('iframe');
  cadre.title = 'PÉPIN : essai du niveau';
  cadre.src = `index.html?essai=1&t=${Date.now()}`;
  zone.append(cadre);
  zone.hidden = false;
  zone.classList.add('cache'); // invisible tant que le jeu n'a pas dit « pret »
  const minuterie = setTimeout(() => {
    fermerEssai();
    montrerVerdict('le jeu n\'a pas répondu en 30 s (est-ce bien le build de QA, ./build.sh --qa ?)', false);
  }, 30000);
  essai = { cadre, jouer, minuterie };
  signaler(jouer ? 'essai : le jeu vérifie le niveau…' : 'validation par le jeu…');
}

function fermerEssai() {
  if (!essai) return;
  clearTimeout(essai.minuterie);
  essai.cadre.remove();
  essai = null;
  $('#essai').hidden = true;
  $('#essai').classList.remove('cache');
  toile.focus?.();
}

window.addEventListener('message', (e) => {
  if (e.origin !== location.origin || !e.data || e.data.source !== 'pepin' || !essai) return;
  const m = e.data;
  if (m.etat === 'erreur') {
    fermerEssai();
    // Ligne et colonne de la GRILLE, depuis 1 : la case (colonne - 1, ligne - 1).
    etat.faute = Number.isInteger(m.ligne)
      ? { x: Number.isInteger(m.colonne) ? m.colonne - 1 : null, y: m.ligne - 1 }
      : null;
    const ou = m.ligne_fichier ? ` (ligne ${m.ligne_fichier} du .ron exporté)` : '';
    montrerVerdict(`REFUSÉ par le jeu : ${m.message}${ou}`, false);
    dessiner();
    faireVoirFaute();
  } else if (m.etat === 'pret') {
    clearTimeout(essai.minuterie);
    etat.faute = null;
    dessiner();
    montrerVerdict('accepté par le chargeur du jeu (niveaux::charger_fichier)', true);
    if (essai.jouer) {
      $('#essai').classList.remove('cache');
      // Le clavier va au canevas du jeu (même origine : on peut l'atteindre).
      essai.cadre.focus();
      essai.cadre.contentWindow.focus();
      essai.cadre.contentDocument?.getElementById('glcanvas')?.focus();
      signaler('essai en cours : Échap ou la fin du niveau ramène ici');
    } else {
      fermerEssai();
      signaler('niveau valide');
    }
  } else if (m.etat === 'fini') {
    fermerEssai();
    signaler(m.raison === 'fin' ? 'essai fini : niveau terminé' : 'essai fini par le jeu (Échap)');
  }
});

function montrerVerdict(texte, ok) {
  const v = $('#verdict');
  v.textContent = texte;
  v.className = ok ? 'ok' : '';
  v.hidden = false;
}

/// Fait défiler la grille jusqu'à la case fautive.
function faireVoirFaute() {
  if (!etat.faute) return;
  const d = $('#defilement');
  const z = etat.zoom;
  const x = (etat.faute.x ?? 0) * z;
  d.scrollLeft = Math.max(0, x - d.clientWidth / 2);
  d.scrollTop = Math.max(0, etat.faute.y * z - d.clientHeight / 2);
}

/// « Partir d'ici » : le même niveau, le départ @ déplacé sur la case `p`
/// (vide seulement, pour ne rien effacer : une goutte, un ennemi...). La grille
/// de l'éditeur ne change pas, seul le texte envoyé au jeu.
function partirDIci(p) {
  if (etat.grille[p.y][p.x] !== '.') {
    signaler(`partir d'ici : choisis une case vide (celle-ci contient « ${etat.grille[p.y][p.x]} »)`, true);
    return;
  }
  const grille = etat.grille.map((r) => r.map((c) => (c === '@' ? '.' : c)));
  grille[p.y][p.x] = '@';
  const texte = ecrireNiveau({ ...etat.niveau, ascii: versAscii(grille.map((r) => r.join(''))) });
  lancerEssai(texte, true);
  signaler(`essai depuis la colonne ${p.x}, rangée ${p.y}…`);
}

// --- Robot de faisabilité (carte 10.3c) ---------------------------------------
//
// Le robot de `core` (`faisabilite::analyser`) compilé en wasm (`robot.wasm`,
// crate `robot`), lancé dans un Web Worker (`robot-worker.js`) : la page reste
// libre, et Arrêter ou la limite de temps terminent simplement le Worker.

let robot = null; // {worker, minuterie, sansPlaner, debut} pendant une recherche

function lancerRobot() {
  arreterRobot();
  const sansPlaner = $('#robot-sans-planer').checked;
  const limite = Math.max(1, Number($('#robot-limite').value) || 30);
  const worker = new Worker('editeur/robot-worker.js');
  const minuterie = setTimeout(() => {
    arreterRobot();
    montrerRobot(`ROBOT arrêté : limite de ${limite} s atteinte sans réponse (relance avec une limite plus longue)`, false);
  }, limite * 1000);
  robot = { worker, minuterie, sansPlaner, debut: performance.now() };
  worker.onmessage = (e) => finRobot(e.data);
  worker.onerror = (e) => { arreterRobot(); montrerRobot(`ROBOT : erreur ${e.message}`, false); };
  worker.postMessage({ texte: texteExporte(), planer: !sansPlaner });
  $('#robot-arreter').hidden = false;
  $('#robot').disabled = true;
  etat.robot = null;
  dessiner();
  montrerRobot(`ROBOT : recherche${sansPlaner ? ' sans planer' : ''}… (au plus ${limite} s)`, true);
}

function arreterRobot() {
  if (!robot) return;
  clearTimeout(robot.minuterie);
  robot.worker.terminate();
  robot = null;
  $('#robot-arreter').hidden = true;
  $('#robot').disabled = false;
}

function finRobot(r) {
  const sansPlaner = robot.sansPlaner;
  arreterRobot();
  if (r.etat !== 'ok') {
    montrerRobot(`ROBOT : le niveau ne se charge pas : ${r.message} (Valider montre la case fautive)`, false);
    return;
  }
  etat.robot = { ...r, sansPlaner };
  const mode = sansPlaner ? 'sans planer' : 'parapluie permis';
  const lanternes = r.lanternes_total ? `, lanternes allumables ${r.lanternes}/${r.lanternes_total}` : '';
  const duree = `robot : ${(r.duree_ms / 1000).toFixed(1)} s, ${r.etats} états`;
  if (r.arrivee) {
    montrerRobot(`ROBOT (${mode}) : arrivée atteinte en ${r.secondes} s de jeu (trajet le plus rapide trouvé), `
      + `${r.chemin.length - 1} sauts${lanternes}, gouttes ${r.gouttes}/3 [${duree}]`, true);
  } else {
    montrerRobot(`ROBOT (${mode}) : ÉCHEC, ${r.raison}${lanternes} [${duree}]`, false);
  }
  // Le défi sans planer : son verdict peut remplir le champ `sans_planer`.
  const b = $('#robot-remplir');
  b.hidden = !sansPlaner || etat.niveau.sans_planer === r.arrivee;
  b.textContent = `Recopier dans sans_planer (${r.arrivee})`;
  dessiner();
  faireVoirRobot();
}

function montrerRobot(texte, ok) {
  const v = $('#robot-resultat');
  $('#robot-texte').textContent = texte;
  v.className = ok ? '' : 'echec';
  v.hidden = false;
  $('#robot-remplir').hidden = true; // `finRobot` le remontre après un défi
}

function remplirSansPlaner() {
  if (!etat.robot || !etat.robot.sansPlaner) return;
  const garde = etat.robot;
  memoriser();
  etat.robot = garde; // la grille ne change pas : le chemin reste juste
  etat.niveau.sans_planer = garde.arrivee;
  toutRafraichir();
  $('#robot-remplir').hidden = true;
  signaler(`sans_planer = ${garde.arrivee} (verdict du robot)`);
}

/// Le chemin du robot (trait passant par le centre de Pépin à chaque point de
/// décision) et, en cas d'échec, l'endroit le plus loin atteint (cercle).
function dessinerRobot(z) {
  const r = etat.robot;
  if (!r) return;
  const k = z / r.tuile;
  const centre = ([x, y]) => [(x + r.joueur[0] / 2) * k, (y + r.joueur[1] / 2) * k];
  ctx.lineWidth = Math.max(2, z / 6);
  ctx.strokeStyle = r.arrivee ? '#40e0ff' : '#ffb040';
  ctx.beginPath();
  r.chemin.forEach((p, i) => { const [x, y] = centre(p); if (i) ctx.lineTo(x, y); else ctx.moveTo(x, y); });
  // Le dernier point est l'élan du saut qui touche l'arrivée : on finit le trait
  // sur le drapeau X de la grille.
  const ligneX = etat.grille.findIndex((l) => l.includes('X'));
  if (r.arrivee && ligneX >= 0) ctx.lineTo((etat.grille[ligneX].indexOf('X') + 0.5) * z, (ligneX + 0.5) * z);
  ctx.stroke();
  ctx.fillStyle = ctx.strokeStyle;
  for (const p of r.chemin) { const [x, y] = centre(p); ctx.fillRect(x - 2, y - 2, 5, 5); }
  if (!r.arrivee) {
    const [x, y] = centre(r.dernier);
    ctx.strokeStyle = '#ff3030';
    ctx.lineWidth = 3;
    ctx.beginPath();
    ctx.arc(x, y, Math.max(12, z * 1.2), 0, Math.PI * 2);
    ctx.stroke();
    ctx.font = `bold ${Math.max(11, Math.floor(z * 0.8))}px sans-serif`;
    ctx.textAlign = 'left';
    ctx.fillStyle = '#ff5050';
    ctx.fillText('bloqué ici', x + Math.max(14, z * 1.4), y);
  }
}

/// Fait défiler la grille jusqu'au bout du chemin (l'arrivée ou l'endroit bloquant).
function faireVoirRobot() {
  const r = etat.robot;
  if (!r || !r.chemin.length) return;
  const d = $('#defilement');
  const k = etat.zoom / r.tuile;
  const [x, y] = r.arrivee ? r.chemin[r.chemin.length - 1] : r.dernier;
  d.scrollLeft = Math.max(0, x * k - d.clientWidth / 2);
  d.scrollTop = Math.max(0, y * k - d.clientHeight / 2);
}

// --- Branchements --------------------------------------------------------------

function construirePalettes() {
  const noms = { tuile: 'Tuiles', repere: 'Repères', collectible: 'Collectibles', ennemi: 'Ennemis',
    objet: 'Objets', eau: 'Eau', boss: 'Boss', decor: 'Panneaux et décor' };
  const zone = $('#palettes');
  for (const [cat, titre] of Object.entries(noms)) {
    const lettres = etat.legende.lettres.filter((l) => l.categorie === cat);
    if (!lettres.length) continue;
    const h = document.createElement('h2');
    h.textContent = titre;
    const div = document.createElement('div');
    div.className = 'palette';
    for (const l of lettres) {
      const b = document.createElement('button');
      b.dataset.lettre = l.lettre;
      b.textContent = l.lettre;
      b.title = `${l.lettre} : ${l.nom}`;
      b.style.background = l.couleur;
      b.style.color = clair(l.couleur) ? '#000' : '#fff';
      b.addEventListener('click', () => choisir(l.lettre));
      div.append(b);
    }
    zone.append(h, div);
  }
}

function brancher() {
  $('#liste-niveaux').addEventListener('change', (e) => { if (e.target.value && confirmerPerte()) ouvrirDuJeu(e.target.value); });
  $('#fichier-local').addEventListener('change', async (e) => {
    const f = e.target.files[0];
    if (f && confirmerPerte()) ouvrirTexte(await f.text(), f.name);
    e.target.value = '';
  });
  $('#nouveau').addEventListener('click', () => { if (confirmerPerte()) nouveau(); });
  $('#exporter').addEventListener('click', exporter);
  $('#valider').addEventListener('click', () => lancerEssai(texteExporte(), false));
  $('#tester').addEventListener('click', () => lancerEssai(texteExporte(), true));
  $('#robot').addEventListener('click', lancerRobot);
  $('#robot-arreter').addEventListener('click', () => { arreterRobot(); montrerRobot('ROBOT arrêté', false); });
  $('#robot-remplir').addEventListener('click', remplirSansPlaner);
  $('#annuler').addEventListener('click', () => restaurer(etat.passe, etat.futur));
  $('#refaire').addEventListener('click', () => restaurer(etat.futur, etat.passe));
  document.querySelectorAll('#boutons-outils button').forEach((b) => b.addEventListener('click', () => choisirOutil(b.dataset.outil)));
  document.querySelectorAll('[data-taille]').forEach((b) => b.addEventListener('click', () => redimensionner(b.dataset.taille)));
  $('#zoom-plus').addEventListener('click', () => zoomer(2));
  $('#zoom-moins').addEventListener('click', () => zoomer(-2));
  $('#lettres').addEventListener('change', dessiner);
  $('#formulaire').addEventListener('change', (e) => {
    const champ = e.target;
    if (!CHAMPS_FORM.includes(champ.name)) return;
    memoriser();
    etat.niveau[champ.name] = champ.type === 'checkbox' ? champ.checked : champ.type === 'number' ? Number(champ.value) : champ.value;
    avertissements();
    majEtat();
  });
  $('#ajouter-aiguille').addEventListener('click', () => { memoriser(); etat.niveau.aiguilles.push({ rayon: 3, vitesse: 30, phase: 0 }); toutRafraichir(); });
  $('#ajouter-poisson').addEventListener('click', () => { memoriser(); etat.niveau.poissons_hauts.push({ colonne: 0, rangee: 0, hauteur: 96 }); toutRafraichir(); });
  window.addEventListener('keydown', (e) => {
    if (essai && e.key === 'Escape') { fermerEssai(); signaler('essai interrompu (Échap dans l\'éditeur)'); return; }
    if (e.target.closest('input, textarea, select')) return;
    const k = e.key.toLowerCase();
    if ((e.ctrlKey || e.metaKey) && k === 'z') { e.preventDefault(); restaurer(etat.passe, etat.futur); return; }
    if ((e.ctrlKey || e.metaKey) && (k === 'y' || (e.shiftKey && k === 'z'))) { e.preventDefault(); restaurer(etat.futur, etat.passe); return; }
    if (e.ctrlKey || e.metaKey || e.altKey) return;
    const outils = { p: 'pinceau', r: 'rectangle', e: 'gomme', i: 'pipette' };
    if (outils[k] && e.shiftKey === false && k === e.key) choisirOutil(outils[k]);
  });
  window.addEventListener('beforeunload', (e) => { if (etat.modifie) e.preventDefault(); });
}

const confirmerPerte = () => !etat.modifie || confirm('Abandonner les modifications non exportées ?');

function zoomer(d) {
  etat.zoom = Math.max(4, Math.min(40, etat.zoom + d));
  $('#zoom').textContent = etat.zoom;
  dessiner();
}

function majEtat() {
  $('#taille').textContent = `${largeur()} colonnes x ${hauteur()} rangées`;
  $('#etat').textContent = `${etat.niveau.cle}.ron${etat.modifie ? ' (modifié)' : ''}`;
}

function signaler(texte, grave = false) {
  const s = $('#etat');
  s.textContent = texte;
  s.style.color = grave ? '#ff7070' : '';
  if (grave) console.warn(texte);
}

function toutRafraichir() {
  dessiner();
  remplirFormulaire();
  avertissements();
  majEtat();
  montrerChoisie(etat.lettre);
  sauverBrouillon();
}

// --- Brouillon : sauvegardé à chaque changement dans le localStorage ------------

const CLE_BROUILLON = 'pepin.editeur.brouillon';
function sauverBrouillon() {
  if (!etat.modifie) return;
  try {
    localStorage.setItem(CLE_BROUILLON, texteExporte());
  } catch { /* stockage plein ou interdit : pas de brouillon */ }
}
/// Rouvre le brouillon s'il y en a un. Rend `true` s'il a été rouvert.
function reprendreBrouillon() {
  const texte = localStorage.getItem(CLE_BROUILLON);
  if (!texte) return false;
  ouvrirTexte(texte, 'brouillon');
  if (!etat.niveau) return false;
  etat.modifie = true; // pas encore exporté
  signaler('brouillon repris (sauvegardé automatiquement dans ce navigateur)');
  majEtat();
  return true;
}

// Pour les captures et les tests pilotés (CDP) : l'état et le texte exporté.
window.editeurPepin = { etat, texteExporte, poser, memoriser, toutRafraichir, ouvrirTexte, choisir, choisirOutil, lancerEssai, partirDIci, lancerRobot, arreterRobot };

demarrer();
