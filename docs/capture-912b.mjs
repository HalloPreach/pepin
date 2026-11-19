// Pilote CDP de la carte 9.12b (« La Pieuvre du port », niveau du boss du Monde 2) sur le build
// de QA, avec l'horloge virtuelle de `pilote-qa.js`. La sauvegarde du navigateur ouvre le Monde 2
// (« Le retour du Grand Nuage » fini, `SAUVEGARDE_MONDE_2_OUVERT` du robot de QA), puis la partie
// du Monde 2 du robot (`ecrire_script_pieuvre_912b`, `--ignored`, qui écrit le script et affiche
// les images à viser : la carte de titre de la pieuvre, les frappes du motif 3) est rejouée image
// par image : BAS au titre à l'image 0 (le script ne code pas cette touche), Entrée à l'image 1
// (ligne MONDE 2), puis les niveaux du Monde 2 jusqu'à la pieuvre.
// Chaque capture rejoue le script depuis une page NEUVE, d'un seul tenant (comme `capture-912.mjs`) :
// un arrêt en cours de route décale parfois le rejeu d'une image, et sous les tentacules serrés
// de la carte 9.12b le robot, réglé au pas près, pourrait alors être touché.
// Compte les erreurs de la console et mesure chaque capture (couleurs hors palette, flou).
// Usage : node capture-912b.mjs PORT_HTTP DOSSIER_SORTIE SCRIPT nom:image [nom:image...]
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
const attendre = async (n, m = 0) => { await evalue(`window.__P.file.push({ n: ${n}, m: ${m} }); 1`); await vide(); };
// La page rechargée avec une sauvegarde où le Monde 2 est ouvert.
const SAUVEGARDE = 'version=3\\nniveau.poursuite=atteint:1 medaille:1\\n';
const ouvrir = async () => {
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(1500);
  await evalue(`localStorage.clear(); localStorage.setItem("pepin.sauvegarde", "${SAUVEGARDE}"); 1`);
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(4000);
  await cdp('Page.bringToFront');
  await evalue('window.focus(); document.querySelector("canvas").focus(); 1');
  await evalue(fs.readFileSync(path.join(ici, 'pilote-qa.js'), 'utf8'));
  await attendre(60);
  return evalue('typeof wasm_exports !== "undefined" && !!wasm_exports && getComputedStyle(document.getElementById("erreur")).display === "none"');
};
// Rejoue `script` depuis l'image 0 jusqu'à l'image `a` (exclue), d'un seul tenant.
const rejouer = async (script, a) => {
  await evalue(`window.__P.touches = { 0: "ArrowDown" }; window.__P.script = ${JSON.stringify(script)}; window.__P.pos = 0; window.__P.cible = ${a}; window.__P.parRaf = 8; 1`);
  for (let i = 0; i < 12000 && !(await evalue('window.__P.pos >= window.__P.cible')); i++) await dormir(50);
  await dormir(150);
};

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });

const script = fs.readFileSync(fichierScript, 'utf8').trim();
for (const d of demandes) {
  const [nom, n] = d.split(':');
  console.log('démarré :', await ouvrir());
  await rejouer(script, Number(n));
  await capture(nom);
}
console.log('erreurs du pilote :', JSON.stringify(await evalue('window.__P.erreurs')));
console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
