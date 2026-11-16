// Pilote CDP de la carte 8.4 (histoire en trois images) : appuis de touches avec l'horloge
// virtuelle de docs/pilote-qa.js. JOUER -> image 1 (un appui trop tôt est ignoré) -> carte du
// niveau 1 ; puis, page rechargée, titre -> HISTOIRE -> images 1, 2, 3 -> retour au titre ;
// enfin Échap au milieu de l'histoire. Compte les erreurs de la console.
// Usage : node capture-histoire.mjs PORT_HTTP DOSSIER_SORTIE
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ici = path.dirname(fileURLToPath(import.meta.url));
const [httpPort, sortie] = process.argv.slice(2);
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
const ouvrir = async () => {
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(4000);
  await cdp('Page.bringToFront');
  await evalue('window.focus(); document.querySelector("canvas").focus(); 1');
  await evalue(fs.readFileSync(path.join(ici, 'pilote-qa.js'), 'utf8'));
  // 60 images sans rien : les appuis des toutes premières images se perdaient.
  await evalue('window.__P.file.push({ n: 60, m: 0 }); 1');
  await dormir(1500);
  return evalue('typeof wasm_exports !== "undefined" && !!wasm_exports && getComputedStyle(document.getElementById("erreur")).display === "none"');
};
// Appuie sur une touche (une image), puis laisse passer `n` images sans rien.
const touche = async (code, n = 20) => {
  if (process.env.DEBUG) console.log('  touche', code, 'image', await evalue('window.__P.n'), 'erreurs', await evalue('JSON.stringify(window.__P.erreurs)'));
  await evalue(`window.__P.file.push({ n: ${n}, m: 0, taper: [${JSON.stringify(code)}] }); 1`);
  for (let i = 0; i < 200 && await evalue('window.__P.file.length') > 0; i++) await dormir(50);
  await dormir(200);
};

// Laisse passer `n` images sans rien appuyer.
const attendre = async (n) => {
  await evalue(`window.__P.file.push({ n: ${n}, m: 0 }); 1`);
  for (let i = 0; i < 200 && await evalue('window.__P.file.length') > 0; i++) await dormir(50);
  await dormir(200);
};

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
await cdp('Page.bringToFront');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });

// 1. JOUER : la première image ; un appui tout de suite est ignoré (elle reste), puis la
//    carte du niveau 1 après un appui.
console.log('1. démarré :', await ouvrir());
await capture('84-titre');
await touche('Enter', 30);
await touche('Enter', 100);
await capture('84-1-depart');
await touche('Enter', 40);
await capture('84-1-puis-carte-niveau1');

// 2. Page rechargée : titre -> HISTOIRE (3e ligne) -> les trois images -> titre.
console.log('2. démarré :', await ouvrir());
await touche('ArrowDown', 5);
await touche('ArrowDown', 30);
await capture('84-titre-ligne-histoire');
await touche('Enter', 100);
await capture('84-1-depart-depuis-titre');
await touche('Enter', 130);
await capture('84-2-lueur');
await attendre(240);
await capture('84-2-lueur-plus-tard');
await touche('Enter', 120);
await capture('84-3-soleil');
await touche('Enter', 40);
await capture('84-3-puis-titre');

// 3. Échap au milieu de l'histoire : retour au titre.
await touche('Enter', 100);
await touche('Enter', 60);
await touche('Escape', 30);
await capture('84-echap-titre');

console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
