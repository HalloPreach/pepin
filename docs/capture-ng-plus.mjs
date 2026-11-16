// Pilote CDP de la carte 8.8 (nouvelle partie +) sur le build de QA, avec l'horloge virtuelle de
// docs/pilote-qa.js :
// 1. une partie déjà finie (`parties_finies=1`) : le titre à six lignes (NOUVELLE PARTIE+) ;
// 2. JOUER : image 1 de l'histoire, niveau 1, puis « suivant » (N) jusqu'aux niveaux 2 et 3 ; à
//    chacun, la même attente après la carte de présentation, puis une capture (au niveau 1, Pépin
//    court d'abord jusque sur la marche des toits, mêmes touches dans les deux modes) ;
// 3. retour au titre (pause, QUITTER AU TITRE), NOUVELLE PARTIE+ : les mêmes niveaux, les mêmes
//    attentes, les mêmes captures (ennemis déplacés, pluie forte, « + » dans le HUD) ;
// 4. les records : la sauvegarde relue après la nouvelle partie + (rien de fini : aucun record).
// Compte les erreurs de la console.
// Usage : node capture-ng-plus.mjs PORT_HTTP DOSSIER_SORTIE
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ici = path.dirname(fileURLToPath(import.meta.url));
const [httpPort, sortie] = process.argv.slice(2);
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

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
await cdp('Page.bringToFront');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });

// 1. Une partie déjà finie : six lignes, NOUVELLE PARTIE+ sous JOUER.
await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
await dormir(1500);
await evalue(`localStorage.setItem("pepin.sauvegarde", "version=3\\nparties_finies=1\\nmeilleur_score=1234\\n"); 1`);
console.log('1. démarré :', await ouvrir());
await touche('ArrowDown', 30);
await capture('88-titre-ng-plus');
await touche('ArrowUp', 10);

// Niveaux photographiés : (numéro, nom, touches tenues) ; la même attente partout après la
// carte. Touches : { n images, m masque } (2 gauche, 4 droite, 8 saut).
const NIVEAUX = [
  // Les toits : courir jusqu'à la marche (colonne 16), sauter dessus, s'y arrêter : la
  // flaque ajoutée (colonne 25) est dans le champ.
  [1, 'toits', [{ n: 120, m: 4 }, { n: 15, m: 12 }, { n: 30, m: 4 }, { n: 20, m: 0 }]],
  [2, 'gouttieres', []],
  [3, 'marche', []],
];
const ATTENTE = 240; // volets (48) + carte de présentation (180) + un peu de jeu
async function parcourir(mode) {
  let niveau = 1;
  for (const [n, nom, tenues] of NIVEAUX) {
    while (niveau < n) { await touche('KeyN', ATTENTE); niveau++; }
    for (const t of tenues) await evalue(`window.__P.file.push(${JSON.stringify(t)}); 1`);
    await vide();
    await capture(`88-${nom}-${mode}`);
  }
}

// 2. JOUER : l'image 1 de l'histoire, puis le niveau 1 et les niveaux photographiés.
await touche('Enter', 100);
await touche('Enter', ATTENTE);
await parcourir('normal');

// 3. Retour au titre (pause : QUITTER AU TITRE, 3e ligne), puis NOUVELLE PARTIE+ (2e ligne).
await touche('Escape', 10);
await touche('ArrowDown', 5);
await touche('ArrowDown', 5);
await touche('Enter', 80);
await touche('ArrowDown', 10);
await touche('Enter', ATTENTE);
await parcourir('ng-plus');

console.log('sauvegarde :', JSON.stringify(await evalue('localStorage.getItem("pepin.sauvegarde")')));
console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
