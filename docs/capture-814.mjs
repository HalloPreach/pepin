// Pilote CDP de la carte 8.14 (mode FACILE du Grand Nuage), sur le build de QA, avec l'horloge
// virtuelle de docs/pilote-qa.js :
// 1. une sauvegarde v3 avec 9 défaites au cyclone (`niveau.cyclone=... defaites:9`) : CARTE, JOUER
//    (pas encore de bannière : 9 < 10), Pépin attend sans bouger sous les éclairs et meurt ;
// 2. la 10e défaite ouvre la bannière « TROP DUR ? » (capture), la sauvegarde dit `defaites:10` ;
// 3. OUI : le niveau recommence en facile (nuage tramé, 2 points de vie au HUD), rejoué par le
//    script du robot de QA (ses deux premiers coups sont les mêmes qu'en normal) ;
// 4. le ciel se déchire après 2 coups : bilan « FACILE », sans médaille ; CARTE : l'étape cyclone
//    dit FACILE, le niveau suivant (vents) est jouable ;
// 5. rechargement de la page : la sauvegarde (facile:1, défaites remises à 0) est relue ; JOUER
//    sur le cyclone propose encore le mode facile (pas battu en normal).
// Compte les erreurs de la console et lit chaque écran clé en pixels logiques (`__P.lire`).
// Usage : node capture-814.mjs PORT_HTTP DOSSIER_SORTIE SCRIPT IMAGE_DEBUT IMAGE_FIN DELAI
//   SCRIPT, IMAGE_DEBUT, IMAGE_FIN : le script du robot et les images de jeu du cyclone à rejouer
//   (jusqu'après le 2e coup) ; DELAI : images entre OUI et la première image de jeu (volets).
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
const attendre = async (n) => { await evalue(`window.__P.file.push({ n: ${n}, m: 0 }); 1`); await vide(); };
const ligne = (texte, cle) => (texte || '').split('\n').find((l) => l.startsWith(cle)) || '(absente)';
const sauvegarde = async () => evalue('localStorage.getItem("pepin.sauvegarde")');
const ouvrir = async () => {
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(4000);
  await cdp('Page.bringToFront');
  await evalue('window.focus(); document.querySelector("canvas").focus(); 1');
  await evalue(fs.readFileSync(path.join(ici, 'pilote-qa.js'), 'utf8'));
  await attendre(60);
  return evalue('typeof wasm_exports !== "undefined" && !!wasm_exports && getComputedStyle(document.getElementById("erreur")).display === "none"');
};

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });

// La sauvegarde v3 : le clocher fini, le cyclone atteint avec 9 défaites d'affilée.
const v3 = [
  'version=3', 'gouttes_en_poche=0', 'gouttes_total_gagnees=0', 'meilleur_score=0', 'parties_finies=0',
  'inventaire=skin:pepin,skin:pepinette,musique:orage', 'equipe=musique:orage,skin:pepin', 'bonus_credites=',
  'niveau.clocher=atteint:1 gouttes:000 defis:000 medaille:0 temps:3000',
  'niveau.cyclone=atteint:1 gouttes:000 defis:000 medaille:0 defaites:9',
  '',
].join('\n');
await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
await dormir(1500);
await evalue(`localStorage.clear(); localStorage.setItem("pepin.sauvegarde", ${JSON.stringify(v3)}); 1`);
console.log('démarré :', await ouvrir());

// 1. Titre → CARTE (ouverte sur le cyclone, où l'on en est) → JOUER : pas de bannière (9 défaites).
await touche('ArrowDown', 10);
await touche('Enter', 40);
await touche('Enter', 48 + 130);
console.log('1. avant la mort :', ligne(await sauvegarde(), 'niveau.cyclone='));
// Pépin ne bouge pas : les salves du motif 1 couvrent les 17 colonnes, il meurt.
await attendre(700);
// 2. La 10e défaite : la bannière, le monde figé dessous.
console.log('2. après la mort :', ligne(await sauvegarde(), 'niveau.cyclone='));
await capture('814-banniere');
await touche('ArrowLeft', 10);
await capture('814-banniere-oui');
// 3. OUI : le niveau recommence en facile, rejoué par le robot.
await evalue('window.__P.file.push({ n: 1, m: 0, taper: ["Enter"] }); 1');
await evalue(`window.__P.file.push({ n: ${Number(delai) - 1}, m: 0 }); 1`);
await vide();
const script = fs.readFileSync(fichierScript, 'utf8').trim();
const debut = Number(imageDebut), fin = Number(imageFin);
// Une capture du combat facile, au milieu du rejeu (après le prélude et le 1er motif).
await evalue(`window.__P.script = ${JSON.stringify(script)}; window.__P.pos = ${debut}; window.__P.cible = ${debut + 420}; window.__P.parRaf = 16; 1`);
for (let i = 0; i < 600 && !(await evalue('window.__P.pos >= window.__P.cible')); i++) await dormir(100);
await capture('814-combat-facile');
await evalue(`window.__P.cible = ${fin}; 1`);
for (let i = 0; i < 3000 && !(await evalue('window.__P.pos >= window.__P.cible')); i++) await dormir(100);
console.log('3. fin du rejeu, erreurs du pilote :', JSON.stringify(await evalue('window.__P.erreurs')));
// 4. Le ciel se déchire (2 s) : le bilan du niveau joué seul.
await attendre(240);
await capture('814-bilan-facile');
console.log('4. après la victoire :', ligne(await sauvegarde(), 'niveau.cyclone='));
await touche('ArrowDown', 10);
await touche('Enter', 48 + 20);
await capture('814-carte-cyclone-facile');
await touche('ArrowUp', 20);
await capture('814-carte-vents-ouvert');

// 5. Rechargement : la sauvegarde est relue ; JOUER sur le cyclone propose encore le mode facile.
console.log('5. relancé :', await ouvrir());
console.log('5. relue :', ligne(await sauvegarde(), 'niveau.cyclone='));
await touche('ArrowDown', 10);
await touche('Enter', 40);
await touche('ArrowDown', 20);
await touche('Enter', 20);
await capture('814-banniere-carte');
await touche('Escape', 20);
console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
