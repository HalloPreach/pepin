// Pilote CDP de la carte 9.2c (le parapluie retourné SOUS Pépin ; le bouton tactile BAS à
// droite, sous le saut) sur le build de QA, avec l'horloge virtuelle de `pilote-qa.js` (bit 16
// = BAS). Chaque partie : sauvegarde d'une partie déjà finie, JOUER, niveau 1, puis F11 : le
// niveau d'essai « eau » (Pépin au départ, le bassin à sa droite).
// 1. Pépin saute puis tient BAS : le parapluie retourné sous lui, en l'air puis au sol ;
//    Pépinette (touche K) en l'air.
// 2. Pépin saute vers l'eau en tenant BAS : juste avant l'eau (debout dans la coque), puis
//    dans la barque (la même coque, au même endroit).
// 3. Les boutons tactiles dans le caniveau (Monde 2, niveau 1), en 844x390 (téléphone couché),
//    667x375 et 390x844 (debout : « Tourne ton téléphone », pas de boutons) : un doigt au
//    milieu (rien n'y répond) active le tactile, puis le
//    doigt tenu sur BAS (son centre, calculé comme `tactile::Disposition::calculer_avec_bas`) :
//    le bouton enfoncé, Pépin parapluie retourné.
// Écrit aussi la grille logique (320x180) de chaque capture dans DOSSIER_GRILLES ; compte les
// erreurs de la console et mesure chaque capture (couleurs hors palette, flou).
// Usage : node capture-92c.mjs PORT_HTTP DOSSIER_SORTIE DOSSIER_GRILLES
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
const ecran = (width, height) => cdp('Emulation.setDeviceMetricsOverride', { width, height, deviceScaleFactor: 1, mobile: false });
const clic = async (x, y) => {
  for (const type of ['mousePressed', 'mouseReleased']) {
    await cdp('Input.dispatchMouseEvent', { type, x, y, button: 'left', clickCount: 1 });
    await attendre(3);
  }
};
// Le caniveau (Monde 2, niveau 1) : sauvegarde où le Monde 2 est ouvert (comme `capture-96.mjs`),
// carte des niveaux, chapitre 2, niveau 1, clic sur JOUER (écran 960x540), puis l'écran voulu.
const partieCaniveau = async (l, h) => {
  await ecran(960, 540);
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(1500);
  await evalue('localStorage.clear(); localStorage.setItem("pepin.sauvegarde", "version=3\\nniveau.poursuite=atteint:1 medaille:1\\n"); 1');
  console.log('démarré :', await ouvrir());
  await touche('ArrowDown', 10);
  await touche('Enter', 120);
  for (let k = 0; k < 4; k++) await touche('ArrowRight', 40);
  for (let k = 0; k < 4; k++) await touche('ArrowDown', 20);
  await attendre(30);
  await clic(646, 450);
  await attendre(240);
  await touche('Enter', 120);
  await ecran(l, h);
  await attendre(30);
};
// Centre du bouton BAS, comme `tactile::Disposition::calculer_avec_bas(l, h, true)`.
const centreBas = (l, h) => {
  const p = Math.min(l, h);
  const taille = Math.min(Math.max(p * 0.26, 56), 180);
  const marge = Math.max(p * 0.04, 8);
  const grand = taille * 1.3;
  const cote = Math.max(grand * 0.85, 48);
  return [l - marge - grand / 2, h - marge - cote / 2];
};

const BAS = 16, SAUT = 8, DROITE = 4;
await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
await ecran(960, 540);
await cdp('Emulation.setTouchEmulationEnabled', { enabled: true, maxTouchPoints: 5 });
fs.mkdirSync(sortie, { recursive: true });
fs.mkdirSync(grilles, { recursive: true });

// 1. Le parapluie retourné SOUS Pépin : un saut, puis BAS tenu (le saut lâché) en redescendant.
// (TACTILE_SEUL=1 : seulement la partie 3.)
if (process.env.TACTILE_SEUL !== '1') {
await partie();
await attendre(14, SAUT);
await attendre(10, BAS);
await capture('92c-retourne-en-l-air');
await attendre(30, BAS);
await capture('92c-retourne-au-sol');
await attendre(10);
await touche('KeyK', 20);
await attendre(14, SAUT);
await attendre(10, BAS);
await capture('92c-retourne-en-l-air-pepinette');

// 2. Vers l'eau : un saut en courant, puis BAS tenu ; juste avant l'eau (debout dans la coque),
//    puis 2 images plus tard, assis dans la barque (horloge virtuelle : mêmes images à chaque fois).
await partie();
await attendre(12, DROITE);
await attendre(12, DROITE | SAUT);
await attendre(18 + 10, DROITE | BAS);
await capture('92c-juste-avant-l-eau');
await attendre(2, DROITE | BAS);
await capture('92c-a-l-eau');
}

// 3. Les boutons tactiles dans le caniveau, BAS à droite sous le saut : téléphone couché (844x390),
//    petit téléphone couché (667x375), debout (390x844 : le jeu demande de tourner le téléphone).
for (const [l, h] of [[844, 390], [667, 375], [390, 844]]) {
  await partieCaniveau(l, h);
  await capture(`92c-caniveau-avant-tactile-${l}x${h}`);
  await doigt('touchStart', l / 2, h / 4);
  await attendre(2);
  await doigt('touchEnd');
  await attendre(10);
  await capture(`92c-boutons-tactiles-${l}x${h}`);
  const [x, y] = centreBas(l, h);
  await doigt('touchStart', x, y);
  await attendre(20);
  await capture(`92c-bouton-bas-tenu-${l}x${h}`);
  await doigt('touchEnd');
  await attendre(5);
}

console.log('erreurs du pilote :', JSON.stringify(await evalue('window.__P.erreurs')));
console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
