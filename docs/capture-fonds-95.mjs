// Pilote CDP (carte 9.5) : charge le build de QA, entre en jeu, puis la touche V
// défile les 7 nouveaux fonds du Monde 2 à l'écran plein (`decor::planche_fonds_qa`)
// ; on capture chacun pour `docs/qa/95-<theme>.png`.
// Usage : node capture-fonds-95.mjs PORT_HTTP  (à servir depuis dist-qa/, `./build.sh --qa`)
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const httpPort = process.argv[2];
const NOMS = ['caniveau', 'egouts', 'cascade', 'grotte', 'riviere', 'port', 'maree'];
const CHROME = process.env.CHROME || 'chromium';
const dbg = 9400 + Math.floor(Math.random() * 400);
const profil = fs.mkdtempSync(path.join(process.env.TMPDIR || '/tmp', 'prof-'));
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

const CODES = { Enter: [13, 'Enter'], KeyV: [86, 'v'] };
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

await taper('Enter'); await dormir(3000); // titre -> carte des niveaux
await taper('Enter'); await dormir(3000); // carte -> niveau 1 en jeu
for (const nom of NOMS) {
  // Un appui V = le fond suivant (`planche_fonds_qa`), plein écran. On laisse
  // l'animation s'installer (pluie, motifs) avant la capture.
  await taper('KeyV');
  await dormir(1500);
  await capture(`docs/qa/95-${nom}.png`);
}
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
