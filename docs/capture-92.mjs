// Pilote CDP de la carte 9.2 (eau et barque) sur le build de QA, avec l'horloge virtuelle de
// `pilote-qa.js` : JOUER (niveau 1), F11 (niveau d'essai « eau »), puis le parcours du robot de
// QA (`parcours_essai_eau`) rejoué image par image, avec Pépin, puis avec Pépinette (touche K
// du build de QA). Enfin, Pépin marche droit dans l'eau sans planer : il coule (une vie de
// moins), puis Pépinette aussi. Captures : le plané au-dessus du bassin, la barque sur le courant,
// le saut de la barque vers le quai, la noyade (Pépin et Pépinette), l'arrivée. Compte les erreurs de la console et
// mesure chaque capture (couleurs hors palette, flou).
// Usage : node capture-92.mjs PORT_HTTP DOSSIER_SORTIE SCRIPT_ROBOT
//   SCRIPT_ROBOT : le parcours du robot, un chiffre hexadécimal par pas (2 gauche, 4 droite,
//   8 saut), écrit par le test ignoré `ecrire_script_essai_eau_92` (game, `--ignored`).
// Le niveau n'a AUCUNE entité qui bouge (gouttes et lanternes seulement) et le parcours commence
// par des pas immobiles : le rejouer quelques images après le début du jeu donne exactement la
// trajectoire du test (Pépin attend au départ, le monde ne change pas).
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ici = path.dirname(fileURLToPath(import.meta.url));
const [httpPort, sortie, fichierScript] = process.argv.slice(2);
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
const script = fs.readFileSync(fichierScript, 'utf8').trim();
// Rejoue le script du robot depuis le pas `de` jusqu'au pas `a` (exclu).
const rejouer = async (de, a) => {
  await evalue(`window.__P.script = ${JSON.stringify(script)}; window.__P.pos = ${de}; window.__P.cible = ${a}; window.__P.parRaf = 8; 1`);
  for (let i = 0; i < 3000 && !(await evalue('window.__P.pos >= window.__P.cible')); i++) await dormir(50);
  await dormir(150);
};
// Niveau d'essai « eau » (F11), sa carte de présentation passée (volets + carte : ~230 images,
// on en laisse 300 ; Pépin attend, immobile, au départ).
const essai_eau = async () => { await touche('F11', 300); };

// Une partie neuve : la page rechargée avec la même sauvegarde (une partie déjà finie une fois :
// JOUER lance le niveau 1 sans l'histoire du début ; skin Pépin), JOUER, image 1, niveau 1, puis
// F11 : le niveau d'essai « eau ».
const partie = async () => {
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(1500);
  await evalue('localStorage.clear(); localStorage.setItem("pepin.sauvegarde", "version=3\\nparties_finies=1\\nmeilleur_score=1234\\n"); 1');
  console.log('démarré :', await ouvrir());
  await touche('Enter', 100);
  await touche('Enter', 300);
  await essai_eau();
};

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });

// 1. Le niveau d'essai « eau », Pépin au départ.
await partie();
await capture('92-eau-depart');
// 2. Le parcours du robot, avec Pépin : plané au-dessus du bassin, ricochet, barque, quai.
await rejouer(0, 150);
await capture('92-eau-plane');
await rejouer(150, 322);
await capture('92-eau-barque-pepin');
await rejouer(322, 345);
await capture('92-eau-saut-vers-le-quai');
await rejouer(345, script.length);
await attendre(10);
await capture('92-eau-quai');
console.log('2. erreurs du pilote :', JSON.stringify(await evalue('window.__P.erreurs')));

// 3. Pépinette (touche K), même parcours, dans une partie neuve : le plané, la barque sur le
// courant, le saut vers le quai.
await partie();
await touche('KeyK', 20);
await rejouer(0, 150);
await capture('92-eau-plane-pepinette');
await rejouer(150, 322);
await capture('92-eau-barque-pepinette');
await rejouer(322, 345);
await capture('92-eau-saut-vers-le-quai-pepinette');

// 4. Pépin (partie neuve) marche droit dans l'eau, parapluie fermé : il coule.
await partie();
await attendre(60, 4);
await capture('92-eau-noyade-1');
await attendre(20);
await capture('92-eau-noyade-2');
await attendre(150);
await capture('92-eau-noyade-reapparition');

// 5. Pépinette coule aussi.
await partie();
await touche('KeyK', 20);
await attendre(60, 4);
await capture('92-eau-noyade-pepinette');

console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
