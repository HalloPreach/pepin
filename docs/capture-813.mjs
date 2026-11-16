// Pilote CDP de la carte 8.13 (retouches de la sauvegarde v3) : une sauvegarde v2 d'un joueur qui
// a fini le jeu (`parties_finies=1`, `niveau_max_atteint=10`, la descente en OR avec ses 3 gouttes
// et ses 3 défis, mais SANS meilleur temps : la v2 ne le notait pas toujours) est posée dans
// localStorage ; le jeu (build de PRODUCTION) la relit. La carte des niveaux doit montrer la
// descente « FINI » avec OR, et le niveau 10 (la poursuite, inséré après coup) jouable.
// Usage : node capture-813.mjs PORT_HTTP DOSSIER_SORTIE
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

// La sauvegarde v2 de la carte : seule la descente (niveau.11 en v2) est notée, en OR, sans temps.
const v2 = [
  'version=2', 'gouttes_en_poche=57', 'gouttes_total_gagnees=140', 'meilleur_score=8100',
  'parties_finies=1', 'niveau_max_atteint=10', 'inventaire=skin:pepin,skin:pepinette,musique:orage',
  'equipe=musique:orage,skin:pepin', 'meilleur_temps=40000',
  'niveau.11=gouttes:111 defis:111 medaille:3',
  '',
].join('\n');
await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
await dormir(1500);
await evalue(`localStorage.clear(); localStorage.setItem("pepin.sauvegarde", ${JSON.stringify(v2)}); 1`);
await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
await dormir(4000);
await cdp('Page.bringToFront');
await evalue('window.focus(); document.querySelector("canvas").focus(); 1');
await evalue(fs.readFileSync(path.join(ici, 'pilote-qa.js'), 'utf8'));
await evalue('window.__P.file.push({ n: 60, m: 0 }); 1');
await dormir(1500);
console.log('démarré :', await evalue('typeof wasm_exports !== "undefined" && !!wasm_exports && getComputedStyle(document.getElementById("erreur")).display === "none"'));

// Titre → CARTE : la carte s'ouvre « où l'on en est », sur la descente.
await touche('ArrowDown', 10);
await touche('Enter', 40);
await capture('813-carte-descente-finie');
// Deux pas vers le bas du chemin : l'arc-en-ciel (bonus, caché), puis le niveau 10.
await touche('ArrowDown', 30);
await touche('ArrowDown', 30);
await capture('813-carte-poursuite-jouable');
const apres = await evalue('localStorage.getItem("pepin.sauvegarde")');
console.log('sauvegarde réécrite :', ligne(apres, 'version='), '|', ligne(apres, 'niveau.descente='),
  '|', ligne(apres, 'niveau.poursuite='));
console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
