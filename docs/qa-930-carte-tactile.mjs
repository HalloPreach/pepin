// Test AU DOIGT de la carte des niveaux (carte 9.30), Chromium sans tête, émulation tactile CDP,
// paysage 844x390, DPR 3, sur le build de QA servi en local. Sauvegarde : « poursuite » finie
// (le chapitre 2 est ouvert). Un seul geste : changer de chapitre en touchant les flèches du
// panneau (chapitre 1 -> 2, puis 2 -> 1), jugé sur l'image affichée (`__P.lire()`, 320x180).
// Usage : CHROME=/chemin/chrome node docs/qa-930-carte-tactile.mjs PORT_HTTP DOSSIER PREFIXE [captures]
// `captures` : écrit PREFIXE-carte-tactile-ch1.png et -ch2.png (la carte de chaque chapitre).
// Écrit PREFIXE-rapport.json et affiche une ligne OK / NON par geste.
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ici = path.dirname(fileURLToPath(import.meta.url));
const [httpPort, sortie, prefixe, captures] = process.argv.slice(2);
const CHROME = process.env.CHROME || 'chromium';
const L = 844, H = 390, DPR = 3;
const dbg = 9400 + Math.floor(Math.random() * 400);
const profil = fs.mkdtempSync(path.join(process.env.TMPDIR || '/tmp', 'prof-'));
const chrome = spawn(CHROME, [
  `--remote-debugging-port=${dbg}`, `--user-data-dir=${profil}`, '--headless=new', '--no-sandbox',
  '--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist',
  `--window-size=${L},${H}`, 'about:blank',
], { stdio: 'ignore' });
const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
let cible;
for (let i = 0; i < 300; i++) {
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
  if (d.method === 'Runtime.consoleAPICalled' && d.params.type === 'error') {
    erreurs.push(d.params.args.map((a) => a.value ?? a.description).join(' '));
  }
};
const cdp = (method, params = {}) => new Promise((r) => { const i = ++id; att.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
const evalue = async (expr) => (await cdp('Runtime.evaluate', { expression: expr, returnByValue: true })).result?.result?.value;
const vide = async () => {
  for (let i = 0; i < 2000 && await evalue('window.__P.file.length') > 0; i++) await dormir(30);
  await dormir(100);
};
const attendre = async (n) => { await evalue(`window.__P.file.push({ n: ${n}, m: 0 }); 1`); await vide(); };
const touche = async (code, n = 20) => {
  await evalue(`window.__P.file.push({ n: ${n}, m: 0, taper: [${JSON.stringify(code)}] }); 1`);
  await vide();
};
// Pixels CSS du point logique (x, y) de l'image 320x180.
let geo;
const vers = (x, y) => ({ x: geo.x0 + (x + 0.5) * geo.k, y: geo.y0 + (y + 0.5) * geo.k, id: 1, radiusX: 4, radiusY: 4 });
const doigt = (type, p) => cdp('Input.dispatchTouchEvent', { type, touchPoints: type === 'touchEnd' ? [] : [p] });
const toucher = async (x, y, n = 20) => {
  await doigt('touchStart', vers(x, y)); await attendre(3);
  await doigt('touchEnd'); await attendre(n);
};
// Lecture de l'image affichée (grille logique 320x180, un caractère par pixel).
const lire = async () => JSON.parse(await evalue('window.__P.lire()')).grille;
const zone = (g, x0, y0, x1, y1) => { let s = ''; for (let y = y0; y < y1; y++) s += g.slice(y * 320 + x0, y * 320 + x1); return s; };
// La carte est-elle affichée ? Le bandeau du bas (RETOUR + aide), fixe sur la carte.
let bandeauCarte;
const surCarte = (g) => zone(g, 0, 166, 176, 180) === bandeauCarte;
const rapport = [];
const noter = (geste, ok, detail) => { rapport.push({ geste, ok, detail }); console.log(ok ? 'OK ' : 'NON', '|', geste, '|', detail); };

const SAUVEGARDE = 'version=3\\nniveau.poursuite=atteint:1 medaille:1\\n';
await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Emulation.setDeviceMetricsOverride', { width: L, height: H, deviceScaleFactor: DPR, mobile: true });
await cdp('Emulation.setTouchEmulationEnabled', { enabled: true, maxTouchPoints: 5 });
fs.mkdirSync(sortie, { recursive: true });
// Page rechargée, sauvegarde « poursuite finie », puis la carte (ligne CARTE du titre, au
// clavier : le test porte sur la carte elle-même). Elle s'ouvre au chapitre 2 (où l'on en est) ;
// Gauche deux fois mène au chapitre 1 (de JOUER à la COURSE du chapitre 1, puis à JOUER).
let titre;
const ouvrirCarte = async (chapitre) => {
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(1500);
  await evalue(`localStorage.clear(); localStorage.setItem("pepin.sauvegarde", "${SAUVEGARDE}"); 1`);
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(4000);
  await cdp('Page.bringToFront');
  await evalue('window.focus(); document.querySelector("canvas").focus(); 1');
  await evalue(fs.readFileSync(path.join(ici, 'pilote-qa.js'), 'utf8'));
  await attendre(60);
  geo = await evalue(`(() => { const c = document.querySelector("canvas").getBoundingClientRect();
    const k = Math.max(1, Math.min(Math.floor(c.width * ${DPR} / 320), Math.floor(c.height * ${DPR} / 180))) / ${DPR};
    return { x0: c.left + Math.floor((c.width - 320 * k) / 2 * ${DPR}) / ${DPR}, y0: c.top + Math.floor((c.height - 180 * k) / 2 * ${DPR}) / ${DPR}, k }; })()`);
  titre = await lire();
  await touche('ArrowDown', 10);
  await touche('Enter', 120);
  if (chapitre === 1) { await touche('ArrowLeft', 10); await touche('ArrowLeft', 30); }
  await attendre(30);
  const g = await lire();
  bandeauCarte ??= zone(g, 0, 166, 176, 180);
  return g;
};
const capturer = async (nom) => {
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  fs.writeFileSync(path.join(sortie, `${prefixe}-${nom}.png`), Buffer.from(r.result.data, 'base64'));
  const l = JSON.parse(await evalue('window.__P.lire()'));
  console.log('capture', nom, '| hors palette :', Object.values(l.hors).reduce((a, b) => a + b, 0), '| flou :', l.flou);
};

// Le seul geste de la carte 9.30 : au chapitre 1, toucher « CHAPITRE 2 ▶ » (à droite de la
// rangée des flèches du panneau, y 122 à 138) passe au chapitre 2 ; là, toucher « ◀ CHAPITRE 1 »
// (à gauche) revient au chapitre 1. Jugé sur le bandeau du haut (le titre du chapitre).
const bandeauHaut = (g) => zone(g, 0, 0, 320, 14);
const g = await ouvrirCarte(1);
const titre1 = bandeauHaut(g);
noter('ch1 carte ouverte', surCarte(g) && g !== titre, '');
if (captures) await capturer('carte-tactile-ch1');
await toucher(290, 130, 40);
const g2 = await lire();
const ok2 = surCarte(g2) && bandeauHaut(g2) !== titre1;
noter('ch1 toucher « CHAPITRE 2 ▶ » (290,130)', ok2, ok2 ? 'passe au chapitre 2 (le titre du bandeau change)' : 'RIEN');
if (captures) await capturer('carte-tactile-ch2');
await toucher(200, 130, 40);
const g3 = await lire();
const ok3 = surCarte(g3) && bandeauHaut(g3) === titre1;
noter('ch2 toucher « ◀ CHAPITRE 1 » (200,130)', ok3, ok3 ? "revient au chapitre 1 (même titre qu'au départ)" : 'RIEN');
console.log('erreurs du pilote :', JSON.stringify(await evalue('window.__P.erreurs')), '| erreurs console :', erreurs.length, JSON.stringify(erreurs));
fs.writeFileSync(path.join(sortie, `${prefixe}-rapport.json`), JSON.stringify(rapport, null, 1));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
