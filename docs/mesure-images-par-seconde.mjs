// Mesure des images par seconde (carte 5.18) : rejoue le script du robot de QA
// (target/qa/script.txt, ou SCRIPT=...) avec l'horloge virtuelle de docs/pilote-qa.js,
// puis, à chaque image demandée, joue la suite du script en TEMPS RÉEL : une image du
// jeu par rappel de requestAnimationFrame du navigateur (comme la boucle normale du
// jeu), pendant DUREE secondes (défaut 10). Affiche, pour chaque point de mesure :
// - les images jouées par seconde réelle (le navigateur appelle requestAnimationFrame
//   au rythme de l'écran, 60 Hz en général : 60 i/s veut dire qu'aucune image n'est
//   sautée) ;
// - le coût d'une image du jeu (simulation + dessin, `wasm_exports.frame()`), mesuré
//   avec performance.now() : moyenne, 95e centile et pire image, à comparer aux
//   16,7 ms d'une image à 60 Hz.
// Usage : node mesure-images-par-seconde.mjs PORT_HTTP nom=IMAGE [nom=IMAGE ...]
// (images par ordre croissant : le rejeu ne revient pas en arrière).
// Variables d'environnement : SCRIPT, DUREE (secondes), FENETRE (défaut 960x540), CHROME.
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ici = path.dirname(fileURLToPath(import.meta.url));
const [httpPort, ...demandes] = process.argv.slice(2);
const CHROME = process.env.CHROME || 'chromium';
const DUREE = Number(process.env.DUREE || 10);
const [largeur, hauteur] = (process.env.FENETRE || '960x540').split('x').map(Number);
const dbg = 9400 + Math.floor(Math.random() * 400);
const profil = fs.mkdtempSync(path.join(process.env.TMPDIR || '/tmp', 'prof-'));
const chrome = spawn(CHROME, [
  `--remote-debugging-port=${dbg}`, `--user-data-dir=${profil}`, '--headless=new', '--no-sandbox',
  '--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist',
  `--window-size=${largeur},${hauteur}`, '--force-device-scale-factor=1', 'about:blank',
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
await cdp('Emulation.setDeviceMetricsOverride', { width: largeur, height: hauteur, deviceScaleFactor: 1, mobile: false });
await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
await dormir(4000);
await cdp('Page.bringToFront');
await evalue('window.focus(); document.querySelector("canvas").focus(); 1');
await evalue(fs.readFileSync(path.join(ici, 'pilote-qa.js'), 'utf8'));
const script = fs.readFileSync(process.env.SCRIPT || path.join(ici, '..', 'target/qa/script.txt'), 'utf8').trim();
await evalue(`window.__P.script = ${JSON.stringify(script)}; 1`);

// Chronométrage : on enveloppe wasm_exports.frame() (une image du jeu) une fois pour toutes.
// Les exports d'une instance WebAssembly sont gelés (on ne peut pas remplacer `frame`
// dessus) ; `wasm_exports` est une variable globale du chargeur de macroquad : on la
// remplace par une copie dont seule `frame` change (les autres exports, la mémoire...
// restent les mêmes objets).
await evalue(`(() => {
  const P = window.__P; const f = wasm_exports.frame;
  P.durees = null;
  window.wasm_exports = Object.assign({}, wasm_exports, {
    frame: () => { const t0 = performance.now(); f(); if (P.durees) P.durees.push(performance.now() - t0); },
  });
  return 1; })()`);

const jusqua = async (image) => {
  await evalue(`(() => { const P = window.__P; P.parRaf = 8; P.cible = ${image}; P.gel = 0; return 1; })()`);
  for (let i = 0; i < 6000; i++) {
    const ok = await evalue('(() => { const P = window.__P; return P.pos >= Math.min(P.cible, P.script.length) && P.file.length === 0 && P.gel >= 3; })()');
    if (ok) break;
    await dormir(50);
  }
};

console.log(`point          image   i/s réelles  coût moyen  95e centile  pire image  (ms, ${DUREE} s)`);
for (const d of demandes) {
  const [nom, image] = d.split('=');
  await jusqua(Number(image));
  // Temps réel : une image du jeu par rappel de requestAnimationFrame.
  const debut = await evalue(`(() => { const P = window.__P; P.durees = []; P.parRaf = 1; P.gel = 0;
    P.cible = P.script.length; P.t0 = performance.now(); P.n0 = P.n; return P.n; })()`);
  await dormir(DUREE * 1000);
  const r = await evalue(`(() => { const P = window.__P; const n = P.n - P.n0; const s = (performance.now() - P.t0) / 1000;
    const d = P.durees.slice().sort((a, b) => a - b); P.durees = null; P.cible = P.pos; P.parRaf = 8;
    const moy = d.reduce((a, b) => a + b, 0) / Math.max(1, d.length);
    return JSON.stringify({ n, s, moy, p95: d[Math.floor(d.length * 0.95)] || 0, max: d[d.length - 1] || 0, erreurs: P.erreurs.length }); })()`);
  const m = JSON.parse(r);
  console.log(`${nom.padEnd(14)} ${String(debut).padStart(6)}   ${(m.n / m.s).toFixed(1).padStart(10)}  ${m.moy.toFixed(2).padStart(10)}  ${m.p95.toFixed(2).padStart(11)}  ${m.max.toFixed(2).padStart(10)}  (${m.n} images, erreurs JS : ${m.erreurs})`);
}
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
