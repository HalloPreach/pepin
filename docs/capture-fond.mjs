// Pilote CDP : charge le jeu, saute au niveau N (touche), avance et capture à plusieurs positions.
// Usage : node cap.mjs PORT_HTTP NIVEAU PREFIXE  (écrit PREFIXE-debut|milieu|fin.png)
import { spawn } from 'node:child_process';
import fs from 'node:fs';

const [httpPort, niveau, prefixe] = [process.argv[2], process.argv[3], process.argv[4]];
const CHROME = '/home/lucas/.hermes/tools/chromium-1208/chrome-linux64/chrome';
const dbg = 9400 + Math.floor(Math.random() * 400);
const profil = fs.mkdtempSync('/home/lucas/.hermes/cache/scratch/fonds/prof-');
const chrome = spawn(CHROME, [
  `--remote-debugging-port=${dbg}`, `--user-data-dir=${profil}`, '--headless=new', '--no-sandbox',
  '--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist',
  '--window-size=1280,720', '--force-device-scale-factor=1', 'about:blank',
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

await cdp('Page.enable');
await cdp('Page.bringToFront');
await cdp('Emulation.setDeviceMetricsOverride', { width: 1280, height: 720, deviceScaleFactor: 1, mobile: false });
await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
await dormir(4000);
await cdp('Page.bringToFront');

const CODES = { ArrowRight: [39, 'ArrowRight'], ArrowLeft: [37, 'ArrowLeft'], Space: [32, ' '], Enter: [13, 'Enter'], Digit1: [49, '1'], Digit2: [50, '2'], Digit3: [51, '3'], Digit4: [52, '4'], Digit5: [53, '5'] };
const touche = async (code, bas) => {
  const [vk, key] = CODES[code];
  await cdp('Input.dispatchKeyEvent', { type: bas ? 'keyDown' : 'keyUp', code, key, windowsVirtualKeyCode: vk, nativeVirtualKeyCode: vk });
};
const taper = async (code, ms = 120) => { await touche(code, true); await dormir(ms); await touche(code, false); };
const capture = async (nom) => {
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  fs.writeFileSync(nom, Buffer.from(r.result.data, 'base64'));
  console.log('capture', nom);
};

await taper('Enter'); await dormir(3000);          // titre -> JOUER -> carte du niveau 1
await taper('Enter'); await dormir(3000);          // carte -> niveau 1 en jeu
await taper(`Digit${niveau}`); await dormir(3000); // raccourci : carte du niveau demandé
await taper('Enter'); await dormir(3000);          // carte -> niveau en jeu
await capture(`${prefixe}-debut.png`);

// Avancer en tenant -> et en sautant (planer = maintenir Espace) pour déplacer la caméra.
const avancer = async (ms) => {
  await touche('ArrowRight', true);
  const fin = Date.now() + ms;
  while (Date.now() < fin) { await taper('Space', 250); await dormir(450); }
  await touche('ArrowRight', false);
};
await avancer(Number(process.env.T1 || 9000)); await capture(`${prefixe}-milieu.png`);
await avancer(Number(process.env.T2 || 12000)); await capture(`${prefixe}-fin.png`);
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
