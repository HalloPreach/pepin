// Pilote CDP de la carte 7.9 (sauvegarde v3 par clé, chaque récompense créditée une fois) :
// une sauvegarde v2 RÉALISTE (niveaux rangés par numéro : niveau.10 = l'arc-en-ciel,
// niveau.11 = la descente) est posée dans localStorage, le jeu (build de PRODUCTION) la relit :
// la carte des niveaux doit montrer les bons temps et médailles (arc-en-ciel, descente), la
// sauvegarde est réécrite en v3 (par clés). Puis la descente est REJOUÉE depuis la carte (la
// descente du robot de QA, target/qa/script.txt) : le gain doit être +0 et le porte-monnaie ne
// doit pas bouger. Relève localStorage et compte les erreurs de la console.
// Usage : node capture-migration.mjs PORT_HTTP DOSSIER_SORTIE SCRIPT IMAGE_DEBUT IMAGE_FIN DELAI
//   IMAGE_DEBUT : première image de jeu de la descente dans le script (après sa carte de
//   présentation) ; IMAGE_FIN : première image de l'écran de fin ; DELAI : images entre
//   l'Entrée de JOUER (carte des niveaux) et la première image de jeu (volets + présentation).
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
const sauvegarde = async () => await evalue('localStorage.getItem("pepin.sauvegarde")');
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
const touche = async (code, n = 20) => {
  await evalue(`window.__P.file.push({ n: ${n}, m: 0, taper: [${JSON.stringify(code)}] }); 1`);
  await vide();
};
const ligne = (texte, cle) => (texte || '').split('\n').find((l) => l.startsWith(cle)) || '(absente)';

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });

// 1. Une sauvegarde v2 réaliste (écrite par la carte 7.4) : 9 niveaux normaux finis avec
// toutes leurs gouttes, l'arc-en-ciel en ARGENT (55,0 s), la descente en OR (18,3 s), toutes
// ses gouttes et tous ses défis déjà eus. 57 gouttes en poche.
const v2 = [
  'version=2', 'gouttes_en_poche=57', 'gouttes_total_gagnees=140', 'meilleur_score=8100',
  'parties_finies=2', 'niveau_max_atteint=10', 'inventaire=skin:pepin,skin:pepinette,musique:orage',
  'equipe=musique:orage,skin:pepin', 'meilleur_temps=40000',
  ...[2400, 2900, 3100, 3300, 3500, 2700, 3200, 3400, 3600].map((t, i) =>
    `niveau.${i + 1}=gouttes:111 defis:${i % 2 ? '110' : '111'} medaille:${i % 2 ? 2 : 3} temps:${t}`),
  'niveau.10=gouttes:111 defis:011 medaille:2 temps:3300',
  'niveau.11=gouttes:111 defis:111 medaille:3 temps:1100',
  '',
].join('\n');
await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
await dormir(1500);
await evalue(`localStorage.clear(); localStorage.setItem("pepin.sauvegarde", ${JSON.stringify(v2)}); 1`);
console.log('1. démarré :', await ouvrir());
await capture('79-titre-v2-relue');
// Le jeu n'écrit qu'à la première occasion : on ouvre la carte des niveaux (titre, CARTE).
await touche('ArrowDown', 10);
await touche('Enter', 40);
await capture('79-carte-descente');
await touche('ArrowDown', 30); // vers le bas du chemin : l'arc-en-ciel (bonus), à gauche
await capture('79-carte-arc-en-ciel');
await touche('ArrowUp', 30);

// 2. La descente REJOUÉE depuis la carte (Entrée : JOUER), puis la descente du robot de QA
// rejouée image par image jusqu'au drapeau.
await touche('Enter', 1);
await capture('79-descente-carte-de-presentation');
// La carte de présentation passe d'elle-même, comme dans la partie du robot : `DELAI` images
// après l'Entrée de JOUER, le niveau commence et le rejeu prend la suite.
await evalue(`window.__P.file.push({ n: ${Number(delai) - 1}, m: 0 }); 1`);
await vide();
const script = fs.readFileSync(fichierScript, 'utf8').trim();
await evalue(`window.__P.script = ${JSON.stringify(script)}; window.__P.pos = ${Number(imageDebut)}; window.__P.cible = ${Number(imageDebut) + 150}; window.__P.parRaf = 16; 1`);
for (let i = 0; i < 600; i++) {
  if (await evalue('window.__P.pos >= window.__P.cible')) break;
  await dormir(100);
}
await capture('79-descente-en-cours');
await evalue(`window.__P.cible = ${Number(imageFin)}; 1`);
for (let i = 0; i < 6000; i++) {
  if (await evalue('window.__P.pos >= window.__P.cible')) break;
  await dormir(100);
}
console.log('2. fin du rejeu, erreurs du pilote :', JSON.stringify(await evalue('window.__P.erreurs')));
await evalue('window.__P.file.push({ n: 60, m: 0 }); 1');
await vide();
await capture('79-bilan-descente-rejouee');
const apres = await sauvegarde();
console.log('2. sauvegarde après la descente rejouée :\n' + apres);
console.log('2. en poche :', ligne(apres, 'gouttes_en_poche='), '| descente :', ligne(apres, 'niveau.descente='),
  '| arc :', ligne(apres, 'niveau.arc='));
console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
