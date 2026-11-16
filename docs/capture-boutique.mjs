// Pilote CDP de la carte 7.5 (boutique) : sauvegarde préparée dans localStorage (clé
// « pepin.sauvegarde »), puis appuis de touches avec l'horloge virtuelle de docs/pilote-qa.js :
// titre -> BOUTIQUE, équiper Pépinette, achat refusé (pas assez de gouttes), touche G (+100
// gouttes), achat confirmé (objet factice du build de QA), retour au titre, page rechargée (skin gardé ?), jeu.
// Relève localStorage après chaque étape et compte les erreurs de la console.
// Usage : node capture-boutique.mjs PORT_HTTP DOSSIER_SORTIE
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
const sauvegarde = async () => JSON.stringify(await evalue('localStorage.getItem("pepin.sauvegarde")'));
const ouvrir = async () => {
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(4000);
  await cdp('Page.bringToFront');
  await evalue('window.focus(); document.querySelector("canvas").focus(); 1');
  await evalue(fs.readFileSync(path.join(ici, 'pilote-qa.js'), 'utf8'));
  // 60 images sans rien : les appuis des toutes premières images se perdaient.
  await evalue('window.__P.file.push({ n: 60, m: 0 }); 1');
  await dormir(1500);
  return evalue('typeof wasm_exports !== "undefined" && !!wasm_exports && getComputedStyle(document.getElementById("erreur")).display === "none"');
};
// Appuie sur une touche (une image), puis laisse passer `n` images sans rien.
const touche = async (code, n = 20) => {
  if (process.env.DEBUG) console.log('  touche', code, 'image', await evalue('window.__P.n'), 'erreurs', await evalue('JSON.stringify(window.__P.erreurs)'));
  await evalue(`window.__P.file.push({ n: ${n}, m: 0, taper: [${JSON.stringify(code)}] }); 1`);
  for (let i = 0; i < 200 && await evalue('window.__P.file.length') > 0; i++) await dormir(50);
  await dormir(200);
};
const ecrire = (texte) => evalue(`localStorage.setItem("pepin.sauvegarde", ${JSON.stringify(texte)}); 1`);

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
await cdp('Page.bringToFront');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });

// 1. 30 gouttes en poche : titre, puis la boutique (2e ligne).
await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
await dormir(1500);
await ecrire('version=1\ngouttes_en_poche=30\ngouttes_total_gagnees=30\n');
console.log('1. démarré :', await ouvrir(), '| sauvegarde :', await sauvegarde());
await touche('ArrowDown', 30);
await capture('75-titre-ligne-boutique');
await touche('Enter', 40);
await capture('75-boutique');

// 2. Pépinette (gratuite) : équipée d'un appui.
await touche('ArrowDown');
await touche('Enter', 40);
console.log('2. Pépinette équipée :', await sauvegarde());
await capture('75-boutique-pepinette-equipee');

// 3. Le factice (60 gouttes) avec 30 : refusé.
await touche('ArrowDown');
await touche('Enter', 30);
await capture('75-achat-refuse');
console.log('3. après refus :', await sauvegarde());
await touche('Enter');

// 4. Touche G du build de QA (+100 gouttes) : confirmation (NON choisi d'abord), OUI, achat.
console.log('4. démarré :', await ouvrir());
await touche('KeyG');
console.log('4. après la touche G :', await sauvegarde());
await touche('ArrowDown');
await touche('Enter', 30);
for (let i = 0; i < 2; i++) await touche('ArrowDown', 5);
await touche('Enter', 30);
await capture('75-achat-confirmation');
await touche('ArrowRight', 30);
await capture('75-achat-confirmation-oui');
await touche('Enter', 40);
console.log('4. après achat :', await sauvegarde());
await capture('75-achat-fait');

// 5. Retour au titre, puis page rechargée : Pépinette court toujours sur le titre.
await touche('Escape', 60);
await capture('75-titre-pepinette');
console.log('5. démarré (rechargé) :', await ouvrir(), '| sauvegarde :', await sauvegarde());
await capture('75-titre-pepinette-recharge');

// 6. Jouer : Pépinette en jeu (et dans l'icône des vies).
await touche('Enter', 400);
await capture('75-jeu-pepinette');

// 7. Petit écran de téléphone en paysage (640 x 360, x2) : la boutique reste lisible.
await cdp('Emulation.setDeviceMetricsOverride', { width: 640, height: 360, deviceScaleFactor: 1, mobile: false });
console.log('7. démarré (640x360) :', await ouvrir());
await touche('ArrowDown');
await touche('Enter', 40);
await capture('75-boutique-640x360');

console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
