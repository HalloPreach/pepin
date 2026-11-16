// Pilote CDP de la carte 7.3 (défis, médailles, bonus de fin) : part d'une sauvegarde vide,
// rejoue le script du robot de QA (target/qa/script.txt, voir QA.md) avec l'horloge virtuelle de
// docs/pilote-qa.js, s'arrête sur la carte du niveau 2 (défis du niveau 1 : capture), va jusqu'à
// l'écran de fin (bonus de fin : capture, puis la page des défis du dernier niveau), relève
// localStorage (clé « pepin.sauvegarde ») et recharge la page (sauvegarde conservée ?).
// Compte les erreurs de la console.
// Usage : node capture-defis.mjs PORT_HTTP DOSSIER_SORTIE SCRIPT IMAGE_CARTE_2
//   IMAGE_CARTE_2 : numéro d'image (dans le script) où la carte du niveau 2 est affichée.
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ici = path.dirname(fileURLToPath(import.meta.url));
const [httpPort, sortie, fichierScript, imageCarte2] = process.argv.slice(2);
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
  console.log('capture', nom);
};
const sauvegarde = () => evalue('localStorage.getItem("pepin.sauvegarde")');
const ouvrir = async () => {
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(4000);
  await cdp('Page.bringToFront');
  await evalue('window.focus(); document.querySelector("canvas").focus(); 1');
  return evalue('typeof wasm_exports !== "undefined" && !!wasm_exports && getComputedStyle(document.getElementById("erreur")).display === "none"');
};
// Attend que le pilote ait joué tout ce qu'on lui a demandé (horloge gelée ensuite).
const attendre = async () => {
  for (let i = 0; i < 6000; i++) {
    if (await evalue('(() => { const P = window.__P; return P.pos >= P.cible && !P.file.length && P.gel >= 3; })()')) return;
    await dormir(100);
  }
};

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
await cdp('Page.bringToFront');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });

// 1. Sauvegarde vide.
await ouvrir();
await evalue('localStorage.clear(); 1');
console.log('1. démarré :', await ouvrir(), '| sauvegarde :', JSON.stringify(await sauvegarde()));

// 2. Le niveau 1 joué par le robot, jusqu'à la carte du niveau 2 : défis du niveau 1.
await evalue(fs.readFileSync(path.join(ici, 'pilote-qa.js'), 'utf8'));
const script = fs.readFileSync(fichierScript, 'utf8').trim();
await evalue(`window.__P.script = ${JSON.stringify(script)}; window.__P.parRaf = 16; window.__P.cible = ${Number(imageCarte2)}; window.__P.gel = 0; 1`);
await attendre();
console.log('2. image', await evalue('window.__P.n'), '| sauvegarde :', JSON.stringify(await sauvegarde()));
await capture('73-fin-de-niveau-defis');

// 3. La suite de la partie, jusqu'à l'écran de fin : bonus de fin de partie.
await evalue('window.__P.cible = window.__P.script.length; window.__P.gel = 0; 1');
await attendre();
console.log('3. fin du rejeu, erreurs du pilote :', JSON.stringify(await evalue('window.__P.erreurs')));
console.log('3. sauvegarde :', JSON.stringify(await sauvegarde()));
await capture('73-fin-de-partie-bonus');
// 3 s plus tard, la bande montre les défis du dernier niveau.
await evalue('window.__P.file.push({ m: 0, n: 200 }); window.__P.gel = 0; 1');
await attendre();
await capture('73-fin-de-partie-defis');

// 4. Page rechargée : tout est conservé.
console.log('4. démarré :', await ouvrir(), '| sauvegarde :', JSON.stringify(await sauvegarde()));

console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
