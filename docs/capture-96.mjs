// Pilote CDP de la carte 9.6 (« Le caniveau », 1er niveau du Monde 2) sur le build de QA, avec
// l'horloge virtuelle de `pilote-qa.js`. La sauvegarde du navigateur ouvre le Monde 2 (« Le retour
// du Grand Nuage » fini, `SAUVEGARDE_MONDE_2_OUVERT` du robot de QA) :
// 1. la carte des niveaux : CARTE au titre (2 lignes sous JOUER), puis Droite jusqu'au Monde 2 ;
// 2. la partie du Monde 2 du robot de QA (`ecrire_script_monde_2_96`, `--ignored`) rejouée image
//    par image : BAS au titre à l'image 0 (le script ne code pas cette touche), Entrée à l'image 1
//    (ligne MONDE 2), puis le caniveau ; une capture aux images données en argument.
// Compte les erreurs de la console et mesure chaque capture (couleurs hors palette, flou).
// Usage : node capture-96.mjs PORT_HTTP DOSSIER_SORTIE SCRIPT nom:image [nom:image...]
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ici = path.dirname(fileURLToPath(import.meta.url));
const [httpPort, sortie, fichierScript, ...demandes] = process.argv.slice(2);
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
const attendre = async (n, m = 0) => { await evalue(`window.__P.file.push({ n: ${n}, m: ${m} }); 1`); await vide(); };
// La page rechargée avec une sauvegarde où le Monde 2 est ouvert.
const SAUVEGARDE = 'version=3\\nniveau.poursuite=atteint:1 medaille:1\\n';
const ouvrir = async () => {
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(1500);
  await evalue(`localStorage.clear(); localStorage.setItem("pepin.sauvegarde", "${SAUVEGARDE}"); 1`);
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(4000);
  await cdp('Page.bringToFront');
  await evalue('window.focus(); document.querySelector("canvas").focus(); 1');
  await evalue(fs.readFileSync(path.join(ici, 'pilote-qa.js'), 'utf8'));
  await attendre(60);
  return evalue('typeof wasm_exports !== "undefined" && !!wasm_exports && getComputedStyle(document.getElementById("erreur")).display === "none"');
};
// Rejoue `script` depuis l'image `de` jusqu'à l'image `a` (exclue).
const rejouer = async (script, de, a) => {
  await evalue(`window.__P.script = ${JSON.stringify(script)}; window.__P.pos = ${de}; window.__P.cible = ${a}; window.__P.parRaf = 8; 1`);
  for (let i = 0; i < 6000 && !(await evalue('window.__P.pos >= window.__P.cible')); i++) await dormir(50);
  await dormir(150);
};

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });

// 1. La carte des niveaux du Monde 2.
console.log('démarré :', await ouvrir());
await capture('96-titre');
await touche('ArrowDown', 10);
await touche('ArrowDown', 10);
await touche('Enter', 120);
for (let k = 0; k < 4; k++) await touche('ArrowRight', 40);
await attendre(60);
await capture('96-carte-monde-2');

// 2. La partie du Monde 2 du robot.
const script = fs.readFileSync(fichierScript, 'utf8').trim();
console.log('démarré :', await ouvrir());
await evalue('window.__P.touches = { 0: "ArrowDown" }; 1');
let pos = 0;
const etapes = demandes.map((d) => { const [nom, n] = d.split(':'); return [Number(n), nom]; }).sort((a, b) => a[0] - b[0]);
for (const [a, nom] of etapes) {
  await rejouer(script, pos, a);
  pos = a;
  await capture(nom);
}
console.log('erreurs du pilote :', JSON.stringify(await evalue('window.__P.erreurs')));
console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
