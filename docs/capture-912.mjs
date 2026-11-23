// Pilote CDP de la carte 9.12a (la Pieuvre du port) sur le build de QA, avec l'horloge virtuelle
// de `pilote-qa.js` : JOUER (niveau 1), L (arène d'essai de la Pieuvre), puis le parcours du
// robot de QA (`parcours_essai_pieuvre`) rejoué image par image ; une capture à chaque pas
// demandé (annonce d'une colonne, tentacule dressé, tête sortie, tête touchée, victoire).
// Compte les erreurs de la console et mesure chaque capture (couleurs hors palette, flou).
// Usage : node capture-912.mjs PORT_HTTP DOSSIER_SORTIE SCRIPT_ROBOT nom:pas [nom:pas ...]
//   SCRIPT_ROBOT : le parcours du robot, un chiffre hexadécimal par pas (2 gauche, 4 droite,
//   8 saut), écrit par le test ignoré `ecrire_script_essai_pieuvre_912` (game, `--ignored`),
//   qui affiche aussi la chronologie de la pieuvre (pour choisir les pas).
//   Chaque capture rejoue le script depuis une partie neuve jusqu'à son pas.
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ici = path.dirname(fileURLToPath(import.meta.url));
const [httpPort, sortie, fichierScript, ...demandes] = process.argv.slice(2);
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
const attendre = async (n, m = 0) => { await evalue(`window.__P.file.push({ n: ${n}, m: ${m} }); 1`); await vide(); };
const ouvrir = async () => {
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(4000);
  await cdp('Page.bringToFront');
  await evalue('window.focus(); document.querySelector("canvas").focus(); 1');
  await evalue(fs.readFileSync(path.join(ici, 'pilote-qa.js'), 'utf8'));
  await attendre(60);
  return evalue('typeof wasm_exports !== "undefined" && !!wasm_exports && getComputedStyle(document.getElementById("erreur")).display === "none"');
};
const script = fs.readFileSync(fichierScript, 'utf8').trim();
// Décalage (images) entre l'appui sur L et la première image de jeu du niveau d'essai, écrit
// par le même test ignoré (`target/qa/decalage-912.txt`) : L est appuyée au pas 0 du script
// prolongé, puis `decalage` images immobiles (la carte du niveau), puis le parcours. Le monde
// du navigateur est ainsi celui du robot, au pas près (la pieuvre vit sans Pépin). Le pilote
// appuie sur L AVANT une image, mais le jeu ne lit la touche qu'à l'image suivante : une
// image de plus (comme pour la carte 9.4).
const decalage = Number(process.env.DECALAGE ?? Number(fs.readFileSync(path.join(path.dirname(fichierScript), 'decalage-912.txt'), 'utf8')) + 1);
// Carte 9.31 : la carte du niveau ne passe plus seule ; Entrée (1) à la dernière image
// immobile, comme le robot (le jeu commence à l'image suivante).
const prolonge = '0'.repeat(decalage - 1) + '1' + script;
const rejouer = async (de, a) => {
  await evalue(`window.__P.script = ${JSON.stringify(prolonge)}; window.__P.touches = { 0: 'KeyL' }; window.__P.pos = ${de + (de > 0 ? decalage : 0)}; window.__P.cible = ${a + decalage}; window.__P.parRaf = 8; 1`);
  for (let i = 0; i < 3000 && !(await evalue('window.__P.pos >= window.__P.cible')); i++) await dormir(50);
  await dormir(150);
};
// Une partie neuve (sauvegarde d'une partie déjà finie : JOUER lance le niveau 1), JOUER,
// niveau 1 ; L est appuyée par `rejouer` (pas 0 du script prolongé).
const partie = async () => {
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(1500);
  await evalue('localStorage.clear(); localStorage.setItem("pepin.sauvegarde", "version=3\\nparties_finies=1\\nmeilleur_score=1234\\n"); 1');
  console.log('démarré :', await ouvrir());
  await touche('Enter', 100);
  await touche('Enter', 300);
};

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });

// Chaque capture rejoue le script depuis une partie NEUVE, d'un seul tenant : un arrêt en
// cours de route (capture, images figées) décale parfois le rejeu d'une image, et le robot
// réglé au pas près pourrait alors toucher un tentacule.
for (const d of demandes) {
  const [nom, pasTexte] = d.split(':');
  const pas = Number(pasTexte);
  await partie();
  await rejouer(0, pas);
  await capture(nom);
}
console.log('erreurs du pilote :', JSON.stringify(await evalue('window.__P.erreurs')));
console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
