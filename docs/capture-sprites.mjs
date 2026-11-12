// Pilote CDP : rejoue le script du robot de QA (target/qa/script.txt, voir QA.md) avec une horloge
// virtuelle (docs/pilote-qa.js) jusqu'à des images précises, et capture l'écran à chacune.
// Usage : node capture-sprites.mjs PORT_HTTP DOSSIER_SORTIE nom=image [nom=image ...]
// Les images doivent être données par ordre croissant (le rejeu ne revient pas en arrière).
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ici = path.dirname(fileURLToPath(import.meta.url));
const [httpPort, sortie, ...demandes] = process.argv.slice(2);
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
ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.id && att.has(d.id)) { att.get(d.id)(d); att.delete(d.id); } };
const cdp = (method, params = {}) => new Promise((r) => { const i = ++id; att.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
const evalue = async (expr) => (await cdp('Runtime.evaluate', { expression: expr, returnByValue: true })).result?.result?.value;

await cdp('Page.enable');
await cdp('Page.bringToFront');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
await dormir(4000);
await cdp('Page.bringToFront');
await evalue('window.focus(); document.querySelector("canvas").focus(); 1');
await evalue(fs.readFileSync(path.join(ici, 'pilote-qa.js'), 'utf8'));
const script = fs.readFileSync(path.join(ici, '..', 'target/qa/script.txt'), 'utf8').trim();
await evalue(`window.__P.script = ${JSON.stringify(script)}; 1`);
fs.mkdirSync(sortie, { recursive: true });

for (const d of demandes) {
  const [nom, image] = d.split('=');
  await evalue(`(() => { const P = window.__P; P.parRaf = 8; P.cible = ${Number(image) + 1}; P.gel = 0; return 1; })()`);
  for (let i = 0; i < 3000; i++) {
    const ok = await evalue('(() => { const P = window.__P; return P.pos >= Math.min(P.cible, P.script.length) && P.gel >= 3; })()');
    if (ok) break;
    await dormir(100);
  }
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  fs.writeFileSync(path.join(sortie, `${nom}.png`), Buffer.from(r.result.data, 'base64'));
  console.log('capture', nom, 'image', await evalue('window.__P.n'));
}
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
