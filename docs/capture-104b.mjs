// Pilote CDP de la carte 10.4b (vignettes des vrais sprites dans l'éditeur de
// niveaux, build de QA) : avec Chromium sans écran, ouvre `editeur.html` servi par
// un serveur de fichiers statiques (dist-qa/), puis vérifie :
//  1. chaque lettre qui a un `sprite` dans `legende.json` a son image CHARGÉE
//     (sprites/<famille>/<clé>.png copié par `./build.sh --qa`) et un bouton de
//     palette qui la montre ; une lettre sans sprite garde sa case colorée ;
//  2. le bouton « vignettes / cases » bascule l'affichage (palette et grille) et
//     revient ;
//  3. le texte exporté sans modification reste le fichier d'origine ;
//  4. captures `104b-vignettes-port` et `104b-vignettes-grotte` (zoom 32 px).
// Usage : CHROME=chemin/de/chromium node docs/capture-104b.mjs PORT_HTTP DOSSIER_SORTIE
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const [httpPort, sortie] = process.argv.slice(2);
const CHROME = process.env.CHROME || 'chromium';
const dbg = 9400 + Math.floor(Math.random() * 400);
const profil = fs.mkdtempSync(path.join(process.env.TMPDIR || '/tmp', 'prof-'));
const chrome = spawn(CHROME, [
  `--remote-debugging-port=${dbg}`, `--user-data-dir=${profil}`, '--headless=new', '--no-sandbox',
  '--window-size=1400,820', '--force-device-scale-factor=1', 'about:blank',
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
  if (d.method === 'Runtime.exceptionThrown') erreurs.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text);
  if (d.method === 'Runtime.consoleAPICalled' && (d.params.type === 'error' || d.params.type === 'warning')) {
    erreurs.push(d.params.type + ' : ' + d.params.args.map((a) => a.value ?? a.description).join(' '));
  }
  if (d.method === 'Log.entryAdded' && d.params.entry.level === 'error') erreurs.push(d.params.entry.text);
};
const cdp = (method, params = {}) => new Promise((r) => { const i = ++id; att.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
const evalue = async (expr) => (await cdp('Runtime.evaluate', { expression: expr, returnByValue: true, awaitPromise: true })).result?.result?.value;
const capture = async (nom) => {
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  fs.writeFileSync(path.join(sortie, `${nom}.png`), Buffer.from(r.result.data, 'base64'));
  console.log('capture', nom);
};
const souris = (type, x, y) => cdp('Input.dispatchMouseEvent', { type, x, y, button: 'left', clickCount: 1 });
const cliquer = async (x, y) => { await souris('mousePressed', x, y); await souris('mouseReleased', x, y); };
const centre = (sel) => evalue(`(() => { const r = document.querySelector(${JSON.stringify(sel)}).getBoundingClientRect(); return [r.x + r.width / 2, r.y + r.height / 2]; })()`);
let echecs = 0;
const verifier = (nom, ok, detail = '') => { console.log(`${ok ? 'OK   ' : 'ECHEC'} ${nom} ${detail}`); if (!ok) echecs++; };

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
ws.addEventListener('message', (m) => {
  const d = JSON.parse(m.data);
  if (d.method === 'Page.javascriptDialogOpening') cdp('Page.handleJavaScriptDialog', { accept: true });
});
await cdp('Emulation.setDeviceMetricsOverride', { width: 1400, height: 820, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });
const base = `http://127.0.0.1:${httpPort}`;
await cdp('Page.navigate', { url: `${base}/editeur.html` });
await dormir(1000);
await evalue('localStorage.clear(); 1');

// 1. Images chargées, palette en vignettes.
await cdp('Page.navigate', { url: `${base}/editeur.html?niveau=monde_2/port.ron` });
await dormir(2000);
const etatImages = await evalue(`(() => {
  const e = editeurPepin.etat;
  const avec = e.legende.lettres.filter((l) => l.sprite);
  const sans = e.legende.lettres.filter((l) => !l.sprite);
  const pasChargees = avec.filter((l) => { const i = e.images.get(l.lettre); return !i || !i.complete || !i.naturalWidth; }).map((l) => l.lettre);
  const boutonsVignette = avec.filter((l) => document.querySelector('.palette button[data-lettre="' + CSS.escape(l.lettre) + '"]').classList.contains('vignette')).length;
  const sansCase = sans.filter((l) => document.querySelector('.palette button[data-lettre="' + CSS.escape(l.lettre) + '"]').classList.contains('vignette')).map((l) => l.lettre);
  return { avec: avec.length, sans: sans.length, pasChargees, boutonsVignette, sansCase, mode: e.vignettes };
})()`);
verifier('mode vignettes par défaut', etatImages.mode === true);
verifier(`${etatImages.avec} vignettes chargées`, etatImages.avec > 40 && etatImages.pasChargees.length === 0, JSON.stringify(etatImages.pasChargees));
verifier('chaque lettre à sprite a sa vignette dans la palette', etatImages.boutonsVignette === etatImages.avec, `${etatImages.boutonsVignette}/${etatImages.avec}`);
verifier(`${etatImages.sans} lettres sans sprite gardent leur case colorée`, etatImages.sansCase.length === 0, JSON.stringify(etatImages.sansCase));
const original = await evalue('fetch("niveaux/monde_2/port.ron").then((r) => r.text())');
verifier('port : texte exporté = fichier', (await evalue('editeurPepin.texteExporte()')) === original);

// Part de la case d'une mouette (`u`) peinte de la couleur de sa case : presque
// rien en vignette (le sprite), presque tout en mode « cases ».
const pixelMouette = `(() => {
  const e = editeurPepin.etat; const z = e.zoom; const c = e.parLettre.get('u').couleur;
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(c.slice(i, i + 2), 16));
  for (let y = 0; y < e.grille.length; y++) for (let x = 0; x < e.grille[y].length; x++) if (e.grille[y][x] === 'u') {
    const d = document.querySelector('#toile').getContext('2d').getImageData(x * z, y * z, z, z).data;
    let n = 0; for (let i = 0; i < d.length; i += 4) if (d[i] === r && d[i + 1] === g && d[i + 2] === b) n++;
    return Math.round((100 * n) / (z * z));
  }
  return -1;
})()`;
const enVignette = await evalue(pixelMouette);

// 2. Bascule : bouton, puis retour.
await cliquer(...(await centre('#affichage')));
await dormir(200);
const enCases = await evalue(pixelMouette);
const apresBascule = await evalue(`[editeurPepin.etat.vignettes, document.querySelector('#affichage').textContent, document.querySelectorAll('.palette button.vignette').length]`);
verifier('bouton → cases (grille et palette)', apresBascule[0] === false && apresBascule[1] === 'cases' && apresBascule[2] === 0, JSON.stringify(apresBascule));
verifier('la case de la mouette change de dessin', enVignette < 10 && enCases > 50, `couleur de la case : ${enVignette} % en vignette, ${enCases} % en case`);
await cliquer(...(await centre('#affichage')));
await dormir(200);
verifier('bouton → vignettes', (await evalue('editeurPepin.etat.vignettes')) === true);
verifier('lissage coupé (pixels nets)', (await evalue('document.querySelector("#toile").getContext("2d").imageSmoothingEnabled')) === false
  && (await evalue('getComputedStyle(document.querySelector("#toile")).imageRendering')) === 'pixelated');

// 3 et 4. Captures, zoom 32 (2 pixels d'écran par pixel du jeu).
const zoom32 = async () => { for (let i = 0; i < 8; i++) await cliquer(...(await centre('#zoom-plus'))); await dormir(200); };
await zoom32();
verifier('zoom 32', (await evalue('editeurPepin.etat.zoom')) === 32);
// Le port : crabes, corbeau, plateformes mobiles 6 et 7, poissons (colonnes 112 à 138).
await evalue(`(() => { const d = document.querySelector('#defilement'); d.scrollLeft = 112 * 32; d.scrollTop = 0; return 1; })()`);
await dormir(200);
await capture('104b-vignettes-port');
await cdp('Page.navigate', { url: `${base}/editeur.html?niveau=monde_2/grotte.ron` });
await dormir(2000);
const originalGrotte = await evalue('fetch("niveaux/monde_2/grotte.ron").then((r) => r.text())');
verifier('grotte : texte exporté = fichier', (await evalue('editeurPepin.texteExporte()')) === originalGrotte);
await zoom32();
// La grotte : autour de la première stalactite (`V`).
const v = await evalue(`(() => { const e = editeurPepin.etat;
  for (let y = 0; y < e.grille.length; y++) for (let x = 0; x < e.grille[y].length; x++) if (e.grille[y][x] === 'V') {
    document.querySelector('#defilement').scrollLeft = Math.max(0, x * 32 - 250);
    document.querySelector('#defilement').scrollTop = Math.max(0, y * 32 - 120); return [x, y]; }
  return null; })()`);
verifier('grotte : une stalactite trouvée', Array.isArray(v), JSON.stringify(v));
await dormir(200);
await capture('104b-vignettes-grotte');

const vraies = erreurs.filter((e) => !e.includes("'beforeunload' confirmation panel"));
verifier('aucune erreur de console', vraies.length === 0, vraies.join(' | '));
console.log(echecs === 0 ? 'VIGNETTES-OK' : `${echecs} échec(s)`);
ws.close();
chrome.kill();
await dormir(500);
try { fs.rmSync(profil, { recursive: true, force: true }); } catch { /* dossier temporaire laissé */ }
process.exit(echecs === 0 ? 0 : 1);
