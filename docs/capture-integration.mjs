// Pilote CDP de la carte 8.11 (intégration de l'histoire) sur le build de QA, avec l'horloge
// virtuelle de docs/pilote-qa.js et un crochet qui note les boucles audio lancées :
// 1. sauvegarde vide : le titre SANS la ligne HISTOIRE (4 lignes) ;
// 2. une partie déjà finie (`parties_finies=1`) : le titre AVEC HISTOIRE, puis les trois
//    images ;
// 3. JOUER, son actif : image 1, niveau 1, puis « suivant » (N) jusqu'au niveau 10 : sa
//    carte d'accueil, le niveau en jeu et sa musique (Poursuite, pas la valse) ;
// 4. « suivant » jusqu'à la descente, rejouée par le script du robot de QA jusqu'au
//    drapeau : l'écran de fin « DE RETOUR ! » qui conclut l'histoire.
// Compte les erreurs de la console.
// Usage : node capture-integration.mjs PORT_HTTP DOSSIER_SORTIE SCRIPT IMAGE_DEBUT IMAGE_FIN DELAI
//   SCRIPT, IMAGE_DEBUT, IMAGE_FIN : le script du robot et les images de jeu de la descente
//   (comme `capture-migration.mjs`) ; DELAI : images entre la touche N et la première image
//   de jeu de la descente (volets + carte de présentation).
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ici = path.dirname(fileURLToPath(import.meta.url));
const [httpPort, sortie, fichierScript, imageDebut, imageFin, delai] = process.argv.slice(2);
const CHROME = process.env.CHROME || 'chromium';
const dbg = 9400 + Math.floor(Math.random() * 400);
const profil = fs.mkdtempSync(path.join(process.env.TMPDIR || '/tmp', 'prof-'));
const chrome = spawn(CHROME, [
  `--remote-debugging-port=${dbg}`, `--user-data-dir=${profil}`, '--headless=new', '--no-sandbox',
  '--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist',
  '--autoplay-policy=no-user-gesture-required',
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
const vide = async () => {
  for (let i = 0; i < 400 && await evalue('window.__P.file.length') > 0; i++) await dormir(50);
  await dormir(200);
};
const ouvrir = async () => {
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(4000);
  await cdp('Page.bringToFront');
  await evalue('window.focus(); document.querySelector("canvas").focus(); 1');
  await evalue(fs.readFileSync(path.join(ici, 'pilote-qa.js'), 'utf8'));
  await evalue('window.__P.file.push({ n: 60, m: 0 }); 1');
  await vide();
  return evalue('typeof wasm_exports !== "undefined" && !!wasm_exports && getComputedStyle(document.getElementById("erreur")).display === "none"');
};
// Appuie sur une touche (une image), puis laisse passer `n` images sans rien.
const touche = async (code, n = 20) => {
  await evalue(`window.__P.file.push({ n: ${n}, m: 0, taper: [${JSON.stringify(code)}] }); 1`);
  await vide();
};

// Le crochet audio (celui de capture-musique.mjs) : posé avant tout script de la page.
const CROCHET = `(() => {
  const M = window.__M = { actives: new Set(), journal: [] };
  const empreinte = (b) => {
    if (!b) return 'vide';
    const d = b.getChannelData(0); let s = 0;
    for (let i = 0; i < Math.min(4410, d.length); i++) s += Math.abs(d[i]);
    return b.duration.toFixed(2) + 's/' + s.toFixed(1);
  };
  const P = AudioBufferSourceNode.prototype;
  const start = P.start, deco = P.disconnect;
  P.start = function (...a) {
    if (this.loop) { this.__e = empreinte(this.buffer); M.actives.add(this); M.journal.push('+' + this.__e); }
    return start.apply(this, a);
  };
  P.disconnect = function (...a) {
    if (M.actives.delete(this)) M.journal.push('-' + this.__e);
    return deco.apply(this, a);
  };
  M.lire = () => ({ actives: [...M.actives].map((s) => s.__e).sort(), journal: M.journal.splice(0) });
})();`;

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
await cdp('Page.addScriptToEvaluateOnNewDocument', { source: CROCHET });
await cdp('Page.bringToFront');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });
const musiques = async () => JSON.parse(await evalue('JSON.stringify(window.__M.lire())'))
  .actives.filter((x) => !x.startsWith('2.00s')); // la pluie (2,00 s) tourne avec toute musique

// 1. Sauvegarde vide : pas de ligne HISTOIRE.
await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
await dormir(1500);
await evalue('localStorage.clear(); 1');
console.log('1. démarré :', await ouvrir());
await capture('86-titre-sans-histoire');

// 2. Une partie déjà finie : la ligne HISTOIRE (4e), puis les trois images.
await evalue(`localStorage.setItem("pepin.sauvegarde", "version=3\\nparties_finies=1\\nmeilleur_score=1234\\n"); 1`);
console.log('2. démarré :', await ouvrir());
for (let i = 0; i < 3; i++) await touche('ArrowDown', 5);
await touche('ArrowDown', 30);
await capture('86-titre-avec-histoire');
await touche('ArrowUp', 30); // retour sur HISTOIRE (la 4e ligne)
await touche('Enter', 100);
await capture('86-histoire-1');
await touche('Enter', 130);
await attendre_et_capturer('86-histoire-2', 240);
await touche('Enter', 120);
await capture('86-histoire-3');
await touche('Enter', 40);
await capture('86-histoire-puis-titre');

async function attendre_et_capturer(nom, n) {
  await evalue(`window.__P.file.push({ n: ${n}, m: 0 }); 1`);
  await vide();
  await capture(nom);
}

// 3. JOUER (son actif) : image 1, niveau 1, puis N jusqu'au niveau 10.
await touche('KeyM', 30);
console.log('3. titre, musiques :', JSON.stringify(await musiques()));
for (let i = 0; i < 3; i++) await touche('ArrowUp', 5); // JOUER
await touche('Enter', 100);
// Image 1 passée : volets (48 images) et carte du niveau 1 (180), puis le jeu ; un N
// pendant la carte serait ignoré.
await touche('Enter', 300);
console.log('3. niveau 1, musiques :', JSON.stringify(await musiques()));
for (let n = 2; n <= 9; n++) await touche('KeyN', 230);
const valse = await musiques();
console.log('3. niveau 9, musiques :', JSON.stringify(valse));
await touche('KeyN', 120);
await capture('86-accueil-poursuite');
await touche('Enter', 200);
await capture('86-poursuite-en-jeu');
const poursuite = await musiques();
console.log('3. niveau 10, musiques :', JSON.stringify(poursuite));
console.log('3. musique du niveau 10 différente de la valse du niveau 9 :',
  poursuite.length === 1 && valse.length === 1 && poursuite[0] !== valse[0]);

// 4. N, N : le bonus puis la descente, rejouée par le robot jusqu'au drapeau.
await touche('KeyN', 230);
await evalue(`window.__P.file.push({ n: 1, m: 0, taper: ["KeyN"] }); 1`);
await evalue(`window.__P.file.push({ n: ${Number(delai) - 1}, m: 0 }); 1`);
await vide();
const script = fs.readFileSync(fichierScript, 'utf8').trim();
await evalue(`window.__P.script = ${JSON.stringify(script)}; window.__P.pos = ${Number(imageDebut)}; window.__P.cible = ${Number(imageFin)}; window.__P.parRaf = 16; 1`);
for (let i = 0; i < 6000; i++) {
  if (await evalue('window.__P.pos >= window.__P.cible')) break;
  await dormir(100);
}
console.log('4. fin du rejeu, erreurs du pilote :', JSON.stringify(await evalue('window.__P.erreurs')));
await evalue('window.__P.file.push({ n: 200, m: 0 }); 1');
await vide();
await capture('86-fin-de-retour');

console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
