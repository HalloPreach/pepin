// ron.js : lire et écrire un fichier de niveau de PÉPIN (format RON, voir
// assets/levels/LISEZMOI.md), sans dépendance. Utilisé par l'éditeur (navigateur)
// et par le test aller-retour (node, editeur/test-aller-retour.mjs).
//
// `lireNiveau(texte)` rend un objet niveau ; `ecrireNiveau(niveau)` rend le texte
// au format CANONIQUE de l'exporteur Rust (niveaux/src/format.rs, fonction
// `ecrire`) : même ordre des champs, mêmes nombres, grille en chaîne brute.
// Ouvrir puis écrire un fichier du dépôt redonne donc les MÊMES octets.
//
// Les commentaires `//` sont gardés : ceux d'avant la parenthèse ouvrante (l'en-tête)
// et ceux posés juste avant un champ, réécrits à la même place.
// Ce module ne VALIDE pas la grille : c'est le rôle du chargeur Rust du jeu.

/// L'en-tête écrit par l'exporteur Rust (gardé tel quel s'il est déjà là).
export const EN_TETE = '// Niveau de PÉPIN. Format : voir assets/levels/LISEZMOI.md.';

/// L'ordre canonique des champs (celui de `FichierNiveau`).
export const CHAMPS = [
  'monde', 'numero', 'cle', 'nom', 'theme', 'genre', 'sans_planer', 'duree_cible_s',
  'intro', 'mecanique', 'aiguilles', 'poissons_hauts', 'ascii',
];
const FACULTATIFS = ['aiguilles', 'poissons_hauts'];

/// Une erreur de lecture, avec la ligne et la colonne du fichier (comptées depuis 1).
export class ErreurRon extends Error {
  constructor(message, ligne, colonne) {
    super(`ligne ${ligne}, colonne ${colonne} : ${message}`);
    this.ligne = ligne;
    this.colonne = colonne;
  }
}

// --- Lecture : découpage en jetons ------------------------------------------

function decouper(texte) {
  const jetons = [];
  let i = 0, ligne = 1, debutLigne = 0;
  let commentaires = []; // commentaires `//` lus depuis le dernier jeton
  const pos = () => ({ ligne, colonne: i - debutLigne + 1 });
  const erreur = (m) => { const p = pos(); return new ErreurRon(m, p.ligne, p.colonne); };
  while (i < texte.length) {
    const c = texte[i];
    if (c === '\n') { ligne++; i++; debutLigne = i; continue; }
    if (c === ' ' || c === '\t' || c === '\r') { i++; continue; }
    if (texte.startsWith('//', i)) {
      const fin = texte.indexOf('\n', i);
      const bout = fin < 0 ? texte.length : fin;
      commentaires.push(texte.slice(i, bout).replace(/\s+$/, ''));
      i = bout;
      continue;
    }
    if (texte.startsWith('/*', i)) {
      const fin = texte.indexOf('*/', i + 2);
      if (fin < 0) throw erreur('commentaire /* non fermé');
      for (let k = i; k < fin; k++) if (texte[k] === '\n') { ligne++; debutLigne = k + 1; }
      i = fin + 2;
      continue;
    }
    const p = pos();
    const jeton = { ...p, commentaires };
    commentaires = [];
    // Chaîne brute r#"..."# (autant de # à la fermeture qu'à l'ouverture).
    if (c === 'r' && (texte[i + 1] === '#' || texte[i + 1] === '"')) {
      let j = i + 1, dieses = '';
      while (texte[j] === '#') { dieses += '#'; j++; }
      if (texte[j] !== '"') throw erreur('chaîne brute mal ouverte');
      const fermeture = '"' + dieses;
      const fin = texte.indexOf(fermeture, j + 1);
      if (fin < 0) throw erreur('chaîne brute non fermée');
      const valeur = texte.slice(j + 1, fin);
      for (let k = i; k < fin; k++) if (texte[k] === '\n') { ligne++; debutLigne = k + 1; }
      i = fin + fermeture.length;
      jetons.push({ ...jeton, genre: 'chaine', valeur });
      continue;
    }
    if (c === '"') {
      let j = i + 1, valeur = '';
      while (j < texte.length && texte[j] !== '"') {
        if (texte[j] === '\\') {
          const e = texte[j + 1];
          const echappes = { n: '\n', t: '\t', r: '\r', '\\': '\\', '"': '"', '0': '\0' };
          if (e in echappes) { valeur += echappes[e]; j += 2; continue; }
          if (e === 'u' && texte[j + 2] === '{') {
            const fin = texte.indexOf('}', j);
            valeur += String.fromCodePoint(parseInt(texte.slice(j + 3, fin), 16));
            j = fin + 1;
            continue;
          }
          throw erreur(`échappement inconnu \\${e}`);
        }
        if (texte[j] === '\n') { ligne++; debutLigne = j + 1; }
        valeur += texte[j++];
      }
      if (j >= texte.length) throw erreur('chaîne non fermée');
      i = j + 1;
      jetons.push({ ...jeton, genre: 'chaine', valeur });
      continue;
    }
    if ('(),:[]'.includes(c)) { jetons.push({ ...jeton, genre: c }); i++; continue; }
    const m = /^(-?[0-9][0-9_]*(\.[0-9]*)?([eE][-+]?[0-9]+)?|[A-Za-z_][A-Za-z0-9_]*)/.exec(texte.slice(i, i + 64));
    if (!m) throw erreur(`caractère inattendu « ${c} »`);
    const mot = m[0];
    i += mot.length;
    if (/^[-0-9]/.test(mot)) jetons.push({ ...jeton, genre: 'nombre', valeur: Number(mot.replace(/_/g, '')), texte: mot });
    else jetons.push({ ...jeton, genre: 'mot', valeur: mot });
  }
  jetons.push({ ...pos(), commentaires, genre: 'fin' });
  return jetons;
}

// --- Lecture : la structure --------------------------------------------------

/// Lit un niveau. Rend `{ entete: [lignes //], commentaires: {champ: [lignes //]},
/// monde, numero, cle, ..., ascii }` ; lève `ErreurRon` si le texte est mal écrit.
export function lireNiveau(texte) {
  const jetons = decouper(texte);
  let k = 0;
  const voir = () => jetons[k];
  const erreur = (m, j = voir()) => new ErreurRon(m, j.ligne, j.colonne);
  const attendre = (genre) => {
    const j = jetons[k];
    if (j.genre !== genre) throw erreur(`« ${genre} » attendu`, j);
    k++;
    return j;
  };
  // Une valeur : nombre, chaîne, mot (true, false, variante), liste ou tuple.
  const valeur = () => {
    const j = jetons[k];
    if (j.genre === 'nombre' || j.genre === 'chaine' || j.genre === 'mot') { k++; return j; }
    if (j.genre === '[' || j.genre === '(') {
      const fermant = j.genre === '[' ? ']' : ')';
      k++;
      const elements = [];
      const champs = {};
      while (voir().genre !== fermant) {
        // `(rayon: 3.0, ...)` : un tuple aux champs nommés.
        if (voir().genre === 'mot' && jetons[k + 1].genre === ':') {
          const nom = jetons[k].valeur;
          k += 2;
          champs[nom] = valeur();
        } else {
          elements.push(valeur());
        }
        if (voir().genre === ',') k++;
        else if (voir().genre !== fermant) throw erreur(`« , » ou « ${fermant} » attendu`);
      }
      k++;
      return { genre: j.genre, elements, champs, ligne: j.ligne, colonne: j.colonne };
    }
    throw erreur('valeur attendue', j);
  };

  const niveau = { entete: voir().commentaires, commentaires: {} };
  // Commentaires d'en-tête : ceux d'avant la parenthèse ouvrante.
  attendre('(');
  const vus = new Set();
  while (voir().genre !== ')') {
    const nomJeton = attendre('mot');
    const nom = nomJeton.valeur;
    if (!CHAMPS.includes(nom)) throw erreur(`champ inconnu « ${nom} »`, nomJeton);
    if (vus.has(nom)) throw erreur(`champ « ${nom} » en double`, nomJeton);
    vus.add(nom);
    if (nomJeton.commentaires.length) niveau.commentaires[nom] = nomJeton.commentaires;
    attendre(':');
    niveau[nom] = convertir(nom, valeur(), erreur);
    if (voir().genre === ',') k++;
    else if (voir().genre !== ')') throw erreur('« , » ou « ) » attendu');
  }
  const fermant = attendre(')');
  niveau.commentairesFin = [...voir().commentaires];
  if (fermant && voir().genre !== 'fin') throw erreur('texte après la parenthèse fermante');
  for (const nom of CHAMPS) {
    if (!vus.has(nom)) {
      if (FACULTATIFS.includes(nom)) niveau[nom] = [];
      else throw erreur(`champ « ${nom} » manquant`);
    }
  }
  return niveau;
}

/// Convertit la valeur lue du champ `nom` en valeur JavaScript.
function convertir(nom, v, erreur) {
  const nombre = (x) => {
    if (x.genre !== 'nombre') throw erreur(`nombre attendu pour « ${nom} »`, x);
    return x.valeur;
  };
  switch (nom) {
    case 'monde': case 'numero': case 'duree_cible_s': return nombre(v);
    case 'cle': case 'nom': case 'intro': case 'mecanique': case 'ascii':
      if (v.genre !== 'chaine') throw erreur(`chaîne attendue pour « ${nom} »`, v);
      return v.valeur;
    case 'theme': case 'genre':
      if (v.genre !== 'mot') throw erreur(`nom attendu pour « ${nom} »`, v);
      return v.valeur;
    case 'sans_planer':
      if (v.genre !== 'mot' || !['true', 'false'].includes(v.valeur)) throw erreur('true ou false attendu', v);
      return v.valeur === 'true';
    case 'aiguilles':
      if (v.genre !== '[') throw erreur('liste [...] attendue pour « aiguilles »', v);
      return v.elements.map((a) => {
        if (a.genre !== '(') throw erreur('(rayon: ..., vitesse: ..., phase: ...) attendu', a);
        const r = {};
        for (const c of ['rayon', 'vitesse', 'phase']) {
          if (!(c in a.champs)) throw erreur(`« ${c} » manquant`, a);
          r[c] = nombre(a.champs[c]);
        }
        return r;
      });
    case 'poissons_hauts':
      if (v.genre !== '[') throw erreur('liste [...] attendue pour « poissons_hauts »', v);
      return v.elements.map((p) => {
        if (p.genre !== '(' || p.elements.length !== 3) throw erreur('(colonne, rangée, hauteur_px) attendu', p);
        const [colonne, rangee, hauteur] = p.elements.map(nombre);
        return { colonne, rangee, hauteur };
      });
  }
  throw erreur(`champ inconnu « ${nom} »`);
}

// --- Écriture canonique -----------------------------------------------------

/// Un `f32` écrit comme `{:?}` en Rust : le plus court texte qui redonne ce
/// nombre en f32, toujours avec un point (`3.0`).
export function flottant(x) {
  const f = Math.fround(x);
  if (Object.is(f, -0)) return '-0.0';
  let texte = String(f);
  for (let chiffres = 1; chiffres <= 9; chiffres++) {
    const essai = String(Number(f.toPrecision(chiffres)));
    if (Math.fround(Number(essai)) === f) { texte = essai; break; }
  }
  return /[.e]/.test(texte) ? texte : texte + '.0';
}

/// Une chaîne RON entre guillemets, échappée comme `chaine` en Rust.
export function chaine(texte) {
  let s = '"';
  for (const c of texte) {
    if (c === '\\') s += '\\\\';
    else if (c === '"') s += '\\"';
    else if (c === '\n') s += '\\n';
    else if (c === '\t') s += '\\t';
    else s += c;
  }
  return s + '"';
}

/// Écrit le niveau au format canonique (celui de l'exporteur Rust).
export function ecrireNiveau(n) {
  const commentaires = n.commentaires || {};
  const entete = n.entete && n.entete.length ? n.entete : [EN_TETE];
  let s = entete.join('\n') + '\n(\n';
  const champ = (nom, texte) => {
    for (const c of commentaires[nom] || []) s += `    ${c}\n`;
    s += `    ${nom}: ${texte},\n`;
  };
  champ('monde', String(n.monde));
  champ('numero', String(n.numero));
  champ('cle', chaine(n.cle));
  champ('nom', chaine(n.nom));
  champ('theme', n.theme);
  champ('genre', n.genre);
  champ('sans_planer', String(Boolean(n.sans_planer)));
  champ('duree_cible_s', String(n.duree_cible_s));
  champ('intro', chaine(n.intro));
  champ('mecanique', chaine(n.mecanique));
  champ('aiguilles', n.aiguilles.length === 0 ? '[]' : '[\n' + n.aiguilles
    .map((a) => `        (rayon: ${flottant(a.rayon)}, vitesse: ${flottant(a.vitesse)}, phase: ${flottant(a.phase)}),\n`)
    .join('') + '    ]');
  champ('poissons_hauts', n.poissons_hauts.length === 0 ? '[]' : '[\n' + n.poissons_hauts
    .map((p) => `        (${p.colonne}, ${p.rangee}, ${flottant(p.hauteur)}),\n`)
    .join('') + '    ]');
  // Autant de # qu'il faut pour que la grille ne puisse pas fermer la chaîne.
  let dieses = '#';
  while (n.ascii.includes('"' + dieses)) dieses += '#';
  champ('ascii', `r${dieses}"${n.ascii}"${dieses}`);
  s += ')\n';
  for (const c of n.commentairesFin || []) s += c + '\n';
  return s;
}

// --- La grille --------------------------------------------------------------

/// Les rangées de la grille (le saut de ligne du début et celui de la fin, que
/// l'exporteur écrit toujours, sont retirés).
export function rangees(ascii) {
  const r = ascii.split('\n');
  if (r.length && r[0] === '') r.shift();
  if (r.length && r[r.length - 1] === '') r.pop();
  return r;
}

/// L'inverse de `rangees` : le texte `ascii` tel que l'exporteur l'écrit.
export function versAscii(lignes) {
  return '\n' + lignes.join('\n') + '\n';
}
