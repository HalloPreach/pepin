// Test aller-retour de l'éditeur (carte 10.3a) : chaque fichier de niveau du
// dépôt, ouvert puis exporté SANS modification par le code de l'éditeur
// (editeur/ron.js, la grille passant par ses rangées comme dans l'éditeur),
// redonne EXACTEMENT les mêmes octets. Vérifie aussi un commentaire ajouté.
//
//   node editeur/test-aller-retour.mjs        (code de sortie 0 si tout est vert)
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { lireNiveau, ecrireNiveau, rangees, versAscii, flottant } from './ron.js';

const racine = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'assets', 'levels');
const fichiers = [];
(function lister(dossier) {
  for (const e of fs.readdirSync(dossier, { withFileTypes: true })) {
    const p = path.join(dossier, e.name);
    if (e.isDirectory()) lister(p);
    else if (e.name.endsWith('.ron')) fichiers.push(p);
  }
})(racine);
fichiers.sort();

let echecs = 0;
const verifier = (nom, ok, detail = '') => {
  if (!ok) { echecs++; console.log(`ECHEC ${nom} ${detail}`); }
};

for (const f of fichiers) {
  const texte = fs.readFileSync(f, 'utf8');
  const niveau = lireNiveau(texte);
  niveau.ascii = versAscii(rangees(niveau.ascii)); // comme l'éditeur : grille en rangées
  const sortie = ecrireNiveau(niveau);
  verifier(path.relative(racine, f), sortie === texte,
    sortie === texte ? '' : `(premier écart à l'octet ${[...texte].findIndex((c, i) => c !== sortie[i])})`);
}
for (const obligatoire of ['monde_1/toits.ron', 'monde_2/port.ron']) {
  verifier(`présence de ${obligatoire}`, fichiers.some((f) => f.endsWith(obligatoire)));
}

// Les commentaires `//` posés avant un champ sont gardés, à leur place.
const toits = fs.readFileSync(path.join(racine, 'monde_1/toits.ron'), 'utf8');
const commente = toits.replace('    nom:', '    // Le tout premier niveau.\n    nom:');
verifier('commentaire avant un champ', ecrireNiveau(lireNiveau(commente)) === commente);

// Nombres f32 écrits comme Rust (`{:?}`).
for (const [x, attendu] of [[3, '3.0'], [0.1, '0.1'], [-0.5, '-0.5'], [-0, '-0.0'], [57.6, '57.6']]) {
  verifier(`flottant(${x})`, flottant(x) === attendu, `donne ${flottant(x)}`);
}

console.log(`${fichiers.length} fichiers relus et réécrits : ${echecs === 0 ? 'ALLER-RETOUR OK' : echecs + ' échec(s)'}`);
process.exit(echecs === 0 ? 0 : 1);
