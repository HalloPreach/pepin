// Pilote CDP de la carte 7.4 (carte des niveaux) : sauvegarde vide (seul le 1er niveau
// ouvert), défilement (bas : toits, milieu, haut : soleil et descente), touche U du build de
// QA (progression avancée : bonus caché, puis révélé), un niveau lancé depuis la carte et
// FINI (rejeu du niveau 1 du robot de QA, target/qa/script.txt), bilan, retour à la carte mise
// à jour, page rechargée (conservé ?), version téléphone (640 x 360) au toucher.
// Relève localStorage et compte les erreurs de la console.
// Usage : node capture-plan.mjs PORT_HTTP DOSSIER_SORTIE SCRIPT IMAGE_FIN_N1
//   IMAGE_FIN_N1 : nombre d'images du script (depuis l'Entrée du titre) jusqu'au drapeau
//   du niveau 1 compris (première image hors de l'écran de jeu après le niveau 1).
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ici = path.dirname(fileURLToPath(import.meta.url));
const [httpPort, sortie, fichierScript, imageFin] = process.argv.slice(2);
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
const sauvegarde = async () => JSON.stringify(await evalue('localStorage.getItem("pepin.sauvegarde")'));
const ouvrir = async () => {
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(4000);
  await cdp('Page.bringToFront');
  await evalue('window.focus(); document.querySelector("canvas").focus(); 1');
  await evalue(fs.readFileSync(path.join(ici, 'pilote-qa.js'), 'utf8'));
  await evalue('window.__P.file.push({ n: 60, m: 0 }); 1');
  await dormir(1500);
  return evalue('typeof wasm_exports !== "undefined" && !!wasm_exports && getComputedStyle(document.getElementById("erreur")).display === "none"');
};
const vide = async () => {
  for (let i = 0; i < 400 && await evalue('window.__P.file.length') > 0; i++) await dormir(50);
  await dormir(200);
};
// Appuie sur une touche (une image), puis laisse passer `n` images sans rien.
const touche = async (code, n = 20) => {
  await evalue(`window.__P.file.push({ n: ${n}, m: 0, taper: [${JSON.stringify(code)}] }); 1`);
  await vide();
};
// Touche l'écran au point logique (x, y) de l'image 320x180 (toucher CDP), puis `n` images.
const toucher = async (x, y, n = 20) => {
  const r = await evalue('(() => { const c = document.querySelector("canvas").getBoundingClientRect(); return [c.left, c.top, c.width, c.height]; })()');
  const k = Math.max(1, Math.min(Math.floor(r[2] / 320), Math.floor(r[3] / 180)));
  const x0 = r[0] + Math.floor((r[2] - 320 * k) / 2), y0 = r[1] + Math.floor((r[3] - 180 * k) / 2);
  const point = { x: x0 + (x + 0.5) * k, y: y0 + (y + 0.5) * k };
  await cdp('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [point] });
  await evalue('window.__P.file.push({ n: 2, m: 0 }); 1');
  await vide();
  await cdp('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
  await evalue(`window.__P.file.push({ n: ${n}, m: 0 }); 1`);
  await vide();
};
// Molette : `crans` crans (positif : vers le bas de la page, la carte descend vers les toits).
const molette = async (crans) => {
  for (let i = 0; i < Math.abs(crans); i++) {
    await cdp('Input.dispatchMouseEvent', { type: 'mouseWheel', x: 200, y: 200, deltaX: 0, deltaY: Math.sign(crans) * 100 });
    await evalue('window.__P.file.push({ n: 2, m: 0 }); 1');
    await vide();
  }
  await evalue('window.__P.file.push({ n: 10, m: 0 }); 1');
  await vide();
};
const ecrire = (texte) => evalue(`localStorage.setItem("pepin.sauvegarde", ${JSON.stringify(texte)}); 1`);

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
await cdp('Page.bringToFront');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });

// 1. Sauvegarde vide : titre (ligne CARTE), puis la carte : seul le 1er niveau est ouvert.
await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
await dormir(1500);
await evalue('localStorage.clear(); 1');
console.log('1. démarré :', await ouvrir(), '| sauvegarde :', await sauvegarde());
await touche('ArrowDown', 30);
await capture('74-titre-ligne-carte');
await touche('Enter', 40);
await capture('74-carte-vide-bas-toits');
await touche('ArrowUp', 30);
await touche('Enter', 30); // niveau verrouillé : Entrée ne fait rien
await capture('74-carte-vide-niveau2-verrouille');
await molette(-10);
await capture('74-carte-vide-haut-bonus-cache');

// 2. Un niveau lancé depuis la carte et fini : le niveau 1 du robot de QA rejoué.
await molette(10);
await touche('ArrowDown', 10);
await evalue(fs.readFileSync(path.join(ici, 'pilote-qa.js'), 'utf8'));
const script = fs.readFileSync(fichierScript, 'utf8').trim();
await evalue(`window.__P.script = ${JSON.stringify(script)}; window.__P.pos = 0; window.__P.cible = ${Number(imageFin)}; window.__P.parRaf = 16; 1`);
for (let i = 0; i < 6000; i++) {
  if (await evalue('window.__P.pos >= window.__P.cible')) break;
  await dormir(100);
}
console.log('2. fin du rejeu, erreurs du pilote :', JSON.stringify(await evalue('window.__P.erreurs')));
await evalue('window.__P.file.push({ n: 60, m: 0 }); 1');
await vide();
await capture('74-bilan-niveau-1');
console.log('2. sauvegarde :', await sauvegarde());
// CARTE (2e ligne du bilan) : retour à la carte, mise à jour (niveau 2 ouvert).
await touche('ArrowDown', 10);
await touche('Enter', 80);
await capture('74-carte-apres-niveau-1');

// 3. Page rechargée : conservé.
console.log('3. démarré (rechargé) :', await ouvrir(), '| sauvegarde :', await sauvegarde());
await touche('ArrowDown', 10);
await touche('Enter', 40);
await capture('74-carte-rechargee');

// 4. Touche U du build de QA : sept niveaux finis, bonus encore caché ; milieu, puis le haut
// (soleil) sur l'étape du bonus (indice 9 de la table).
await touche('KeyU', 10);
console.log('4. après U :', await sauvegarde());
for (let i = 0; i < 4; i++) await touche('ArrowUp', 4);
await touche('ArrowUp', 30);
await capture('74-carte-avancee-milieu');
for (let i = 0; i < 2; i++) await touche('ArrowUp', 4);
await touche('ArrowUp', 30);
await capture('74-carte-avancee-haut-bonus-cache');
// Second U : tout fini, toutes les gouttes : le bonus se révèle.
await touche('KeyU', 30);
await capture('74-carte-haut-bonus-debloque');
await touche('ArrowUp', 30);
await capture('74-carte-descente');
console.log('4. après le second U :', await sauvegarde());

// 5. Téléphone en paysage (640 x 360, x2) : la carte au toucher.
await cdp('Emulation.setDeviceMetricsOverride', { width: 640, height: 360, deviceScaleFactor: 1, mobile: true });
await cdp('Emulation.setTouchEmulationEnabled', { enabled: true, maxTouchPoints: 1 });
console.log('5. démarré (640x360) :', await ouvrir());
await toucher(160, 58 + 18 + 9, 40); // ligne CARTE du titre
await capture('74-telephone-carte');
await toucher(124, 90, 20); // l'étape du niveau 7 (choisie)
await capture('74-telephone-etape-touchee');
await toucher(248, 150, 60); // JOUER
await capture('74-telephone-jouer');

console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
