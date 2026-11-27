// Test aller-retour de l'éditeur (carte 10.3a) : chaque fichier de niveau du
// dépôt, ouvert puis exporté SANS modification par le code de l'éditeur
// (editeur/ron.js, la grille passant par ses rangées comme dans l'éditeur),
// redonne EXACTEMENT les mêmes octets. Vérifie aussi un commentaire ajouté.
//
//   node editeur/test-aller-retour.mjs        (code de sortie 0 si tout est vert)
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { lireNiveau, ecrireNiveau, rangees, versAscii, flottant, erreursIntro, erreursFacile, lignesIntro } from './ron.js';

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

// L'intro (carte 11.3) : chaque fichier du dépôt a ses deux champs, acceptés par les
// règles de l'éditeur (les mêmes que le chargeur Rust, niveaux/src/intro.rs).
for (const f of fichiers) {
  const n = lireNiveau(fs.readFileSync(f, 'utf8'));
  verifier(`intro de ${path.relative(racine, f)}`, erreursIntro(n).length === 0, erreursIntro(n).join(' ; '));
}
// Saisie dans l'éditeur, puis export : les deux champs sont écrits à leur place.
const saisi = lireNiveau(toits);
saisi.intro_texte = 'Une phrase « à moi ».';
saisi.intro_astuce = 'Un conseil.';
const exporte = ecrireNiveau(saisi);
verifier('intro saisie exportée', exporte.includes('    intro_texte: "Une phrase « à moi ».",\n    intro_astuce: "Un conseil.",\n    mecanique:'));
verifier('intro saisie relue', lireNiveau(exporte).intro_texte === saisi.intro_texte);
// Les refus : vide, trop long, trop long à deux, l'ancien champ `intro`.
verifier('intro vide refusée', erreursIntro({ ...saisi, intro_astuce: '  ' }).length === 1);
verifier('phrase de 4 lignes refusée', erreursIntro({ ...saisi, intro_texte: 'mot '.repeat(40) }).length === 1);
verifier('3 + 2 lignes refusées', erreursIntro({ ...saisi, intro_texte: 'mot '.repeat(30), intro_astuce: 'mot '.repeat(15) }).length === 1);
verifier('bonus : 3 + 1 lignes refusées', erreursIntro({ ...saisi, genre: 'Bonus', intro_texte: 'mot '.repeat(30) }).length === 1);
verifier('coupure comme le jeu', JSON.stringify(lignesIntro('un deux')) === '["un deux"]' && lignesIntro('x'.repeat(50)).length === 1);
let ancien = '';
try { lireNiveau(toits.replace(/    intro_texte: .*\n    intro_astuce: .*\n/, '    intro: "Vieux.",\n')); } catch (e) { ancien = e.message; }
verifier('ancien champ intro refusé clairement', ancien.includes('intro_texte') && ancien.includes('intro_astuce'), ancien);

// La phrase du mode facile (carte 9.34) : chaque boss du dépôt a la sienne, les
// autres niveaux n'en ont pas ; saisie puis exportée à sa place ; les refus.
let boss = 0;
for (const f of fichiers) {
  const n = lireNiveau(fs.readFileSync(f, 'utf8'));
  if (n.facile_texte) boss++;
  verifier(`facile_texte de ${path.relative(racine, f)}`, erreursFacile(n).length === 0, erreursFacile(n).join(' ; '));
}
verifier('quatre boss (cyclone, pieuvre, deux essais)', boss === 4, `${boss}`);
const cyclone = lireNiveau(fs.readFileSync(path.join(racine, 'monde_1/cyclone.ron'), 'utf8'));
verifier('facile_texte relu', cyclone.facile_texte === "MODE FACILE : LE NUAGE N'A QUE 2 VIES", cyclone.facile_texte);
cyclone.facile_texte = 'MODE FACILE : un autre texte';
verifier('facile_texte saisi exporté', ecrireNiveau(cyclone).includes('    facile_texte: "MODE FACILE : un autre texte",\n    mecanique:'));
verifier('boss sans facile_texte refusé', erreursFacile({ ...cyclone, facile_texte: '' }).length === 1);
verifier('facile_texte de 3 lignes refusé', erreursFacile({ ...cyclone, facile_texte: 'mot '.repeat(30) }).length === 1);
verifier('facile_texte sans boss refusé', erreursFacile({ ...saisi, facile_texte: 'MODE FACILE' }).length === 1);
verifier('sans boss : pas de facile_texte exporté', !ecrireNiveau(saisi).includes('facile_texte'));

// Nombres f32 écrits comme Rust (`{:?}`).
for (const [x, attendu] of [[3, '3.0'], [0.1, '0.1'], [-0.5, '-0.5'], [-0, '-0.0'], [57.6, '57.6']]) {
  verifier(`flottant(${x})`, flottant(x) === attendu, `donne ${flottant(x)}`);
}

console.log(`${fichiers.length} fichiers relus et réécrits : ${echecs === 0 ? 'ALLER-RETOUR OK' : echecs + ' échec(s)'}`);
process.exit(echecs === 0 ? 0 : 1);
