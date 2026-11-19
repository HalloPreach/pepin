// Pilote CDP de la carte 9.3 (mécaniques d'eau) sur le build de QA, avec l'horloge virtuelle de
// `pilote-qa.js` : JOUER (niveau 1), F12 (niveau d'essai « eau 2 »), puis le parcours du robot de
// QA (`parcours_essai_eau2`) rejoué image par image : la stalactite qui tremble puis tombe, la
// barque du bassin, les bulles, la marée basse, l'annonce et la montée (la barque monte avec
// l'eau), la chute d'eau, l'arrivée. Puis, dans une partie neuve, Pépin posé sur un nénuphar : la
// feuille s'enfonce et le met à l'eau. Compte les erreurs de la console et mesure chaque capture
// (couleurs hors palette, flou).
// Usage : node capture-93.mjs PORT_HTTP DOSSIER_SORTIE SCRIPT_ROBOT SCRIPT_NENUPHAR
//   les deux scripts (un chiffre hexadécimal par pas : 2 gauche, 4 droite, 8 saut) sont écrits
//   par le test ignoré `ecrire_script_essai_eau2_93` (game, `--ignored`), qui donne aussi le
//   nombre d'images entre F12 et le premier pas du niveau (228) : le pilote les laisse passer
//   avant de rejouer, pour que la marée et la stalactite soient à la même heure que dans le test.
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ici = path.dirname(fileURLToPath(import.meta.url));
const [httpPort, sortie, fichierScript, fichierFeuille] = process.argv.slice(2);
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
const ouvrir = async () => {
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(4000);
  await cdp('Page.bringToFront');
  await evalue('window.focus(); document.querySelector("canvas").focus(); 1');
  await evalue(fs.readFileSync(path.join(ici, 'pilote-qa.js'), 'utf8'));
  await attendre(60);
  return evalue('typeof wasm_exports !== "undefined" && !!wasm_exports && getComputedStyle(document.getElementById("erreur")).display === "none"');
};
const lire = (f) => fs.readFileSync(f, 'utf8').trim();
const robot = lire(fichierScript);
const feuille = lire(fichierFeuille);
// Rejoue `script` depuis le pas `de` jusqu'au pas `a` (exclu).
const rejouer = async (script, de, a) => {
  await evalue(`window.__P.script = ${JSON.stringify(script)}; window.__P.pos = ${de}; window.__P.cible = ${a}; window.__P.parRaf = 8; 1`);
  for (let i = 0; i < 3000 && !(await evalue('window.__P.pos >= window.__P.cible')); i++) await dormir(50);
  await dormir(150);
};
// Une partie neuve : la page rechargée avec la même sauvegarde (une partie déjà finie une fois),
// JOUER, niveau 1, puis F12 : le niveau d'essai « eau 2 ». Les 228 images de F12 au jeu (volets
// et carte de présentation) passent : le niveau n'a pas encore fait un seul pas.
const partie = async () => {
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(1500);
  await evalue('localStorage.clear(); localStorage.setItem("pepin.sauvegarde", "version=3\\nparties_finies=1\\nmeilleur_score=1234\\n"); 1');
  console.log('démarré :', await ouvrir());
  await touche('Enter', 100);
  await touche('Enter', 300);
  await touche('F12', 228);
};

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });

// 1. Le parcours du robot (les pas viennent de `ecrire_script_essai_eau2_93`).
await partie();
await capture('93-eau2-depart');
const etapes = [
  [125, '93-stalactite-tremble'],
  [150, '93-stalactite-chute'],
  [290, '93-barque-bassin'],
  [340, '93-bulles'],
  [620, '93-maree-basse'],
  [750, '93-maree-annonce'],
  [840, '93-maree-barque'],
  [864, '93-maree-saut'],
  [1003, '93-chute-d-eau'],
  [robot.length, '93-arrivee'],
];
let pos = 0;
for (const [a, nom] of etapes) {
  await rejouer(robot, pos, a);
  pos = a;
  await capture(nom);
}
console.log('1. erreurs du pilote :', JSON.stringify(await evalue('window.__P.erreurs')));

// 2. Le nénuphar : posé au pas 210 (posé depuis 20 images), la feuille s'enfonce, Pépin coule.
await partie();
pos = 0;
for (const [a, nom] of [[205, '93-nenuphar-pose'], [240, '93-nenuphar-enfonce'], [262, '93-nenuphar-noyade']]) {
  await rejouer(feuille, pos, a);
  pos = a;
  await capture(nom);
}

console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
