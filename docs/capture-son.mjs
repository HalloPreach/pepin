// Pilote CDP des cartes 7.8 / 7.8b (icône du son, en bas à gauche depuis la 7.8b) : au
// titre (le son part coupé, règle des navigateurs), touche M (son actif), puis clic sur
// l'icône (son coupé) ; puis jeu, pause et clic sur l'icône (son actif). Horloge
// virtuelle de docs/pilote-qa.js ; compte les erreurs de la console.
// Usage : node capture-son.mjs PORT_HTTP DOSSIER_SORTIE
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
// Appuie sur une touche (une image), puis laisse passer `n` images sans rien.
const touche = async (code, n = 20) => {
  await evalue(`window.__P.file.push({ n: ${n}, m: 0, taper: [${JSON.stringify(code)}] }); 1`);
  for (let i = 0; i < 200 && await evalue('window.__P.file.length') > 0; i++) await dormir(50);
  await dormir(200);
};

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
await cdp('Page.bringToFront');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });

await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
await dormir(4000);
await evalue('window.focus(); document.querySelector("canvas").focus(); 1');
await evalue(fs.readFileSync(path.join(ici, 'pilote-qa.js'), 'utf8'));
await evalue('window.__P.file.push({ n: 60, m: 0 }); 1');
await dormir(1500);

// Clic (ou toucher) au point logique (x, y) : la fenêtre fait 960 x 540, 3 pixels par
// pixel logique. Puis `n` images sans rien.
const cliquer = async (x, y, n = 20) => {
  const p = { x: x * 3, y: y * 3, button: 'left', clickCount: 1 };
  await cdp('Input.dispatchMouseEvent', { type: 'mouseMoved', ...p, button: 'none' });
  await cdp('Input.dispatchMouseEvent', { type: 'mousePressed', ...p });
  await cdp('Input.dispatchMouseEvent', { type: 'mouseReleased', ...p });
  await evalue(`window.__P.file.push({ n: ${n}, m: 0 }); 1`);
  for (let i = 0; i < 200 && await evalue('window.__P.file.length') > 0; i++) await dormir(50);
  await dormir(200);
};
// Centre de l'icône du son (22 x 14 pixels logiques, coin haut à gauche en (6, 160)).
const ICONE = [17, 167];

// 1. Titre : touche M, le son est actif (ondes).
await touche('KeyM', 30);
await capture('78b-titre-son-actif');
// 2. Clic sur l'icône : le son est coupé (croix), on reste au titre.
await cliquer(...ICONE, 30);
await capture('78b-titre-son-coupe-apres-clic');
// 3. Jouer, pause (3 lignes, le curseur sur REPRENDRE), clic sur l'icône : son actif.
await touche('Enter', 300);
await touche('Escape', 30);
await cliquer(...ICONE, 30);
await capture('78b-pause-son-actif');

console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
