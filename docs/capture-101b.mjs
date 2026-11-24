// Pilote CDP de la carte 10.1b (le jeu lit ses niveaux dans les fichiers RON) : sur un
// build de QA, avec l'horloge virtuelle de `pilote-qa.js` (une image = un pas, donc la
// même suite de touches donne les mêmes images), capture le départ de trois niveaux :
// toits (touche 1), port et pieuvre (touche N répétée depuis les toits, chaque fois
// suivie d'Entrée pour passer la carte du niveau). Lancé sur le build d'AVANT et sur celui d'APRÈS, puis
// `docs/comparer-captures.py` compte les pixels différents (attendu : 0).
// Usage : node capture-101b.mjs PORT_HTTP DOSSIER_SORTIE [?essai=1 FICHIER.ron NOM]
//   avec `?essai=1` : pose FICHIER.ron dans localStorage (`pepin.niveau_essai`), ouvre
//   la page avec `?essai=1` et capture l'écran sous le nom NOM (le niveau lancé, ou
//   l'écran d'erreur si le fichier est invalide).
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ici = path.dirname(fileURLToPath(import.meta.url));
const [httpPort, sortie, requete = '', fichierEssai = '', nomEssai = 'essai'] = process.argv.slice(2);
const CHROME = process.env.CHROME || 'chromium';
const dbg = 9400 + Math.floor(Math.random() * 400);
const profil = fs.mkdtempSync(path.join(process.env.TMPDIR || '/tmp', 'prof-'));
const chrome = spawn(CHROME, [
  `--remote-debugging-port=${dbg}`, `--user-data-dir=${profil}`, '--headless=new', '--no-sandbox',
  '--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist',
  '--window-size=960,540', '--force-device-scale-factor=1', 'about:blank',
], { stdio: 'ignore' });
const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
let cible;
for (let i = 0; i < 50; i++) {
  try { const l = await (await fetch(`http://127.0.0.1:${dbg}/json`)).json(); cible = l.find((t) => t.type === 'page'); if (cible) break; } catch {}
  await dormir(200);
}
const ws = new WebSocket(cible.webSocketDebuggerUrl);
await new Promise((r) => (ws.onopen = r));
let id = 0; const att = new Map();
const erreurs = [];
ws.onmessage = (m) => {
  const d = JSON.parse(m.data);
  if (d.id && att.has(d.id)) { att.get(d.id)(d); att.delete(d.id); }
  if (d.method === 'Runtime.exceptionThrown') erreurs.push(d.params.exceptionDetails.text);
  if (d.method === 'Runtime.consoleAPICalled' && (d.params.type === 'error' || d.params.type === 'warning')) {
    erreurs.push(d.params.type + ' : ' + d.params.args.map((a) => a.value ?? a.description).join(' '));
  }
  if (d.method === 'Log.entryAdded' && d.params.entry.level === 'error') erreurs.push(d.params.entry.text);
};
const cdp = (method, params = {}) => new Promise((r) => { const i = ++id; att.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
const evalue = async (expr) => (await cdp('Runtime.evaluate', { expression: expr, returnByValue: true })).result?.result?.value;
const capture = async (nom) => {
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  fs.writeFileSync(path.join(sortie, `${nom}.png`), Buffer.from(r.result.data, 'base64'));
  const l = JSON.parse(await evalue('window.__P.lire()'));
  const hors = Object.values(l.hors).reduce((a, b) => a + b, 0);
  console.log('capture', nom, '| hors palette :', hors, '| flou :', l.flou);
};
const vide = async () => {
  for (let i = 0; i < 2000 && await evalue('window.__P.file.length') > 0; i++) await dormir(50);
  await dormir(200);
};
const touche = async (code, n = 20) => {
  await evalue(`window.__P.file.push({ n: ${n}, m: 0, taper: [${JSON.stringify(code)}] }); 1`);
  await vide();
};
const attendre = async (n) => { await evalue(`window.__P.file.push({ n: ${n}, m: 0 }); 1`); await vide(); };

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });

// Une partie neuve (une partie déjà finie : JOUER lance le niveau 1).
await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
await dormir(1500);
await evalue('localStorage.clear(); localStorage.setItem("pepin.sauvegarde", "version=3\\nparties_finies=1\\nmeilleur_score=1234\\n"); 1');
if (fichierEssai) {
  const texte = fs.readFileSync(fichierEssai, 'utf8');
  await evalue(`localStorage.setItem("pepin.niveau_essai", ${JSON.stringify(texte)}); 1`);
}
await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html${requete}` });
await dormir(4000);
await cdp('Page.bringToFront');
await evalue('window.focus(); document.querySelector("canvas").focus(); 1');
await evalue(fs.readFileSync(path.join(ici, 'pilote-qa.js'), 'utf8'));
await attendre(60);
if (requete) {
  // `?essai=1` : le niveau posé dans le navigateur, sans JOUER (carte 10.1b).
  await attendre(120);
  await touche('Enter', 30);
  await attendre(30);
  await capture(nomEssai);
} else {
  await touche('Enter', 100); // JOUER
  await touche('Enter', 300); // la carte du niveau où reprend la partie
  // 1 : le premier niveau (raccourci du build de QA), puis Entrée pour passer sa carte
  // (carte 9.31 : elle attend un appui ; une touche de niveau n'agit qu'en jeu).
  await touche('Digit1', 60);
  await touche('Enter', 30);
  await attendre(30);
  await capture('toits');
  // N : niveau suivant, puis Entrée (sa carte). Port : le 18e niveau, Pieuvre le 19e.
  const suivant = async () => { await touche('KeyN', 60); await touche('Enter', 30); };
  for (let i = 0; i < 17; i++) await suivant();
  await attendre(30);
  await capture('port');
  await suivant();
  await attendre(30);
  await capture('pieuvre');
}
console.log('erreurs du pilote :', JSON.stringify(await evalue('window.__P.erreurs')));
console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
