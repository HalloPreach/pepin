// Pilote CDP de la carte 9.2b (BAS retourne le parapluie ; la barque se prend avec BAS, plus en
// planant) sur le build de QA, avec l'horloge virtuelle de `pilote-qa.js` (le bit 16 de ses
// masques est BAS, la flèche du bas). Chaque partie : sauvegarde d'une partie déjà finie, JOUER,
// niveau 1, puis F11 : le niveau d'essai « eau » (Pépin au départ, le bassin à sa droite).
// 1. Pépin saute puis tient BAS : le parapluie retourné en l'air ; puis Pépinette (touche K).
// 2. Pépin marche dans l'eau en tenant BAS : la barque (puis le courant l'emporte).
// 3. Pépin saute dans l'eau en PLANANT (saut tenu, sans BAS) : il coule.
// 4. Un doigt (toucher CDP) : les boutons tactiles, BAS compris (niveau avec de l'eau) ; le doigt
//    tenu sur BAS : le parapluie retourné au sol, le bouton enfoncé.
// Écrit aussi la grille logique (320x180, un caractère de palette par pixel) de chaque capture
// dans DOSSIER_GRILLES. Le gros plan avant/après de la barque (`92b-barque-enfoncee.png`) se fait
// ensuite avec `docs/gros-plan-92b.py` (avant : `96-barque-filet.png`, après : `92b-barque-bas.png`).
// Compte les erreurs de la console et mesure chaque capture (couleurs hors palette, flou).
// Usage : node capture-92b.mjs PORT_HTTP DOSSIER_SORTIE DOSSIER_GRILLES
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ici = path.dirname(fileURLToPath(import.meta.url));
const [httpPort, sortie, grilles] = process.argv.slice(2);
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
  fs.writeFileSync(path.join(grilles, `${nom}.txt`), l.grille.match(/.{320}/g).join('\n') + '\n');
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
// `n` images avec le masque `m` tenu (1 Entrée, 2 gauche, 4 droite, 8 saut, 16 bas).
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
// Une partie neuve dans le niveau d'essai « eau » (comme `capture-92.mjs`).
const partie = async () => {
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(1500);
  await evalue('localStorage.clear(); localStorage.setItem("pepin.sauvegarde", "version=3\\nparties_finies=1\\nmeilleur_score=1234\\n"); 1');
  console.log('démarré :', await ouvrir());
  await touche('Enter', 100);
  await touche('Enter', 300);
  await touche('F11', 300);
};
const doigt = (type, x, y) => cdp('Input.dispatchTouchEvent', {
  type, touchPoints: type === 'touchEnd' ? [] : [{ x, y, id: 1 }],
});

const BAS = 16, SAUT = 8, DROITE = 4;
await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
await cdp('Emulation.setTouchEmulationEnabled', { enabled: true, maxTouchPoints: 5 });
fs.mkdirSync(sortie, { recursive: true });
fs.mkdirSync(grilles, { recursive: true });

// 1. Le parapluie retourné en l'air : un saut, puis BAS tenu (le saut lâché) en redescendant.
await partie();
await attendre(14, SAUT);
await attendre(10, BAS);
await capture('92b-retourne-en-l-air');
await attendre(30, BAS);
await capture('92b-retourne-au-sol');
await attendre(10);
await touche('KeyK', 20);
await attendre(14, SAUT);
await attendre(10, BAS);
await capture('92b-retourne-en-l-air-pepinette');

// 2. La barque avec BAS : marcher droit dans l'eau, BAS tenu ; à bord, plus rien de tenu.
await partie();
await attendre(70, DROITE | BAS);
await attendre(10);
await capture('92b-barque-bas');
await attendre(60);
await capture('92b-barque-courant');

// 3. La noyade en planant : un saut vers l'eau, saut TENU (le parapluie s'ouvre), sans BAS.
await partie();
await attendre(60, DROITE | SAUT);
await capture('92b-plane-au-dessus-de-l-eau');
await attendre(60, DROITE | SAUT);
await capture('92b-noyade-en-planant');

// 4. Les boutons tactiles : un doigt au milieu de l'écran (rien n'y répond) active le tactile ;
// puis le doigt tenu sur BAS (centré sous les flèches : 960x540, voir `tactile::Disposition`).
await partie();
await doigt('touchStart', 480, 120);
await attendre(2);
await doigt('touchEnd');
await attendre(10);
await capture('92b-bouton-bas-tactile');
await doigt('touchStart', 173, 476);
await attendre(20);
await capture('92b-bouton-bas-tactile-tenu');
await doigt('touchEnd');
await attendre(5);

console.log('erreurs du pilote :', JSON.stringify(await evalue('window.__P.erreurs')));
console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
