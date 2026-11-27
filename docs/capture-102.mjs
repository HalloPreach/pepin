// Captures de référence de la carte 10.2 : trois scènes du jeu prises sur un build de QA
// (dist-qa/), ENTIÈREMENT déterministes, pour vérifier qu'une carte de rendu (refonte de
// game/src/scene.rs...) ne change pas un seul pixel. On les prend AVANT et APRÈS la carte,
// puis on les compare avec `docs/comparer-captures.py`.
//
// Ce qui rend les captures reproductibles au pixel près :
// - l'horloge est FIGÉE dès le premier octet de la page (Date.now renvoie toujours la même
//   valeur, avant même que le jeu ne démarre) : tant que le pilote n'est pas installé, le jeu
//   redessine sans avancer, quel que soit le nombre d'images réelles que le navigateur affiche ;
// - ensuite `pilote-qa.js` fait avancer l'horloge d'exactement 1/60 s par image, avec les
//   touches appuyées à une image précise : la partie est la même à chaque passage ;
// - un profil de navigateur neuf, une sauvegarde vide (localStorage vidé), une fenêtre de
//   960 x 540 (agrandissement 3 de l'écran logique 320 x 180) et le rendu logiciel
//   (SwiftShader), le même sur toutes les machines.
//
// Usage : node docs/capture-102.mjs PORT_HTTP DOSSIER_SORTIE PREFIXE [commande...]
// (le dist-qa doit être servi sur 127.0.0.1:PORT_HTTP ; variable CHROME : chemin de Chromium).
// Produit PREFIXE-toits.png, PREFIXE-port.png et PREFIXE-pieuvre.png dans DOSSIER_SORTIE.
// Avec des commandes en plus, on remplace le parcours par le sien (mise au point d'un
// parcours) : `t:Code:n` tape la touche (code DOM) puis attend n images ; `m:masque:n` tient
// le masque n images (1 Entrée, 2 gauche, 4 droite, 8 saut, 16 bas) ; `w:n` attend n images ;
// `c:nom` capture PREFIXE-nom.png.
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ici = path.dirname(fileURLToPath(import.meta.url));
const [httpPort, sortie, prefixe, ...commandes] = process.argv.slice(2);
if (!httpPort || !sortie || !prefixe) {
  console.error('Usage : node docs/capture-102.mjs PORT_HTTP DOSSIER_SORTIE PREFIXE [commande...]');
  process.exit(2);
}

// Le parcours de référence. Chaque nombre est un nombre d'IMAGES (1/60 s de jeu).
// 1. Titre : Entrée sur JOUER, puis Entrée deux fois pour passer les écrans d'avant la
//    partie ; la carte de titre du 1er niveau attend un appui Entrée accepté après 0,5 s
//    (carte 9.31), d'où les 60 images d'attente avant chaque Entrée.
// 2. Les toits : la carte de titre passée, 60 images au départ (Pépin immobile).
// 3. Le port (avant-dernier niveau de la table, indice 17) : touche 9 (indice 8), puis
//    9 fois « Entrée (passer la carte de titre), N (niveau suivant) ». Sur le port, Pépin
//    court (droite 100 images), saute en courant (192 images : il retombe sur le 1er
//    bateau), se laisse porter 405 images, puis court et saute jusqu'au toit du 2e quai
//    (8 + 108 images) et attend 30 images : la 1re mouette est perchée sur sa passerelle,
//    en haut à droite de l'écran.
// 4. La Pieuvre (dernier niveau, indice 18) : N depuis le port, Entrée, puis 151 images
//    (Entrée comprise) : Pépin est sur le quai de gauche, les cinq bouts de tentacules
//    sortent de l'eau (annonce de la première colonne de frappe).
const TITRE = ['t:Enter:120', 't:Enter:60', 't:Enter:120'];
const VERS_PORT = ['t:Digit9:60'];
for (let i = 0; i < 9; i++) VERS_PORT.push('t:Enter:60', 't:KeyN:60');
const PORT = ['m:4:100', 'm:12:192', 'w:405', 'm:4:8', 'm:12:108', 'w:30'];
const PARCOURS = [
  ...TITRE, 'w:60', 'c:toits',
  ...VERS_PORT, 't:Enter:30', ...PORT, 'c:port',
  't:KeyN:60', 't:Enter:151', 'c:pieuvre',
];
const etapes = commandes.length ? commandes : PARCOURS;

const CHROME = process.env.CHROME || 'chromium';
// Port de débogage du navigateur : tiré au hasard pour pouvoir lancer deux captures à la
// fois (sans effet sur les images).
const dbg = 9400 + Math.floor(Math.random() * 400);
const profil = fs.mkdtempSync(path.join(process.env.TMPDIR || '/tmp', 'prof-102-'));
const chrome = spawn(CHROME, [
  `--remote-debugging-port=${dbg}`, `--user-data-dir=${profil}`, '--headless=new', '--no-sandbox',
  '--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist',
  '--window-size=960,540', '--force-device-scale-factor=1', '--mute-audio', 'about:blank',
], { stdio: 'ignore' });
const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
const finir = async (code) => {
  try { chrome.kill('SIGKILL'); } catch {}
  await dormir(300);
  fs.rmSync(profil, { recursive: true, force: true });
  process.exit(code);
};
let cible;
for (let i = 0; i < 75 && !cible; i++) {
  try { cible = (await (await fetch(`http://127.0.0.1:${dbg}/json`)).json()).find((t) => t.type === 'page'); } catch {}
  if (!cible) await dormir(200);
}
if (!cible) { console.error('Chromium ne répond pas (variable CHROME ?)'); await finir(1); }
const ws = new WebSocket(cible.webSocketDebuggerUrl);
await new Promise((r) => (ws.onopen = r));
let id = 0; const att = new Map();
const erreurs = [];
ws.onmessage = (m) => {
  const d = JSON.parse(m.data);
  if (d.id && att.has(d.id)) { att.get(d.id)(d); att.delete(d.id); }
  if (d.method === 'Runtime.exceptionThrown') erreurs.push(d.params.exceptionDetails.text);
  if (d.method === 'Runtime.consoleAPICalled' && d.params.type === 'error') {
    erreurs.push(d.params.args.map((a) => a.value ?? a.description).join(' '));
  }
};
const cdp = (method, params = {}) => new Promise((r) => { const i = ++id; att.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
const evalue = async (expr) => (await cdp('Runtime.evaluate', { expression: expr, returnByValue: true })).result?.result?.value;

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
// Avant tout script de la page : horloge figée (la valeur de départ du pilote) et
// sauvegarde vide. Le build de QA ouvre de lui-même tous les niveaux.
// Carte 10.4d : avec la variable ESSAI_RON (chemin d'un niveau RON), ce niveau est posé
// dans le `localStorage` (clé `pepin.niveau_essai`) et la page s'ouvre avec `?essai=1` :
// le build de QA le lance directement (voir `game/src/essai_externe.rs`).
const essai = process.env.ESSAI_RON ? fs.readFileSync(process.env.ESSAI_RON, 'utf8') : null;
// Carte 9.34 : avec la variable SAUVEGARDE (chemin d'un texte de sauvegarde v3), elle
// est posée dans le `localStorage` (clé `pepin.sauvegarde`) au lieu d'une sauvegarde vide.
const sauvegarde = process.env.SAUVEGARDE ? fs.readFileSync(process.env.SAUVEGARDE, 'utf8') : null;
const poser = (essai ? `localStorage.setItem('pepin.niveau_essai', ${JSON.stringify(essai)});` : '')
  + (sauvegarde ? `localStorage.setItem('pepin.sauvegarde', ${JSON.stringify(sauvegarde)});` : '');
await cdp('Page.addScriptToEvaluateOnNewDocument', {
  source: `Date.now = () => 1e6; try { localStorage.clear(); ${poser} } catch (e) {}`,
});
const page = essai ? 'index.html?essai=1' : 'index.html';
await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/${page}` });
// Attente RÉELLE du chargement (sans effet sur la partie : l'horloge est figée).
let pret = false;
for (let i = 0; i < 300 && !pret; i++) {
  await dormir(100);
  pret = await evalue('typeof wasm_exports !== "undefined" && !!wasm_exports && getComputedStyle(document.getElementById("erreur")).display === "none"');
}
if (!pret) { console.error('le jeu ne démarre pas sur le port', httpPort); await finir(1); }
await dormir(500);
await cdp('Page.bringToFront');
await evalue('window.focus(); document.querySelector("canvas").focus(); 1');
await evalue(fs.readFileSync(path.join(ici, 'pilote-qa.js'), 'utf8'));
await evalue('window.__P.parRaf = 8; 1');

// Attend que la file du pilote soit vide, puis quelques images réelles « gelées ».
const vide = async () => {
  for (let i = 0; i < 4000 && await evalue('window.__P.file.length') > 0; i++) await dormir(25);
  await dormir(200);
};
const pousser = async (s) => { await evalue(`window.__P.file.push(${JSON.stringify(s)}); 1`); await vide(); };
fs.mkdirSync(sortie, { recursive: true });
for (const e of etapes) {
  const [k, a, b] = e.split(':');
  if (k === 't') await pousser({ n: Number(b || 20), m: 0, taper: [a] });
  else if (k === 'm') await pousser({ n: Number(b), m: Number(a) });
  else if (k === 'w') await pousser({ n: Number(a), m: 0 });
  else if (k === 'c') {
    const r = await cdp('Page.captureScreenshot', { format: 'png' });
    const fichier = path.join(sortie, `${prefixe}-${a}.png`);
    fs.writeFileSync(fichier, Buffer.from(r.result.data, 'base64'));
    console.log('capture', fichier, '| image de jeu', await evalue('window.__P.n'));
  } else { console.error('commande inconnue :', e); await finir(2); }
}
// Carte 9.34 : la sauvegarde posée est-elle encore la même, octet pour octet ?
if (sauvegarde !== null) {
  const relue = await evalue("localStorage.getItem('pepin.sauvegarde')");
  console.log('sauvegarde relue à l\'identique :', relue === sauvegarde ? 'OUI' : `NON\n${relue}`);
}
const errPilote = await evalue('JSON.stringify(window.__P.erreurs)');
console.log('erreurs du pilote :', errPilote, '| erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close();
await finir(errPilote === '[]' && erreurs.length === 0 ? 0 : 1);
