// Pilote CDP de la carte 7.2 (sauvegarde) : rejoue le script du robot de QA
// (target/qa/script.txt, voir QA.md) avec l'horloge virtuelle de docs/pilote-qa.js jusqu'à
// l'écran de fin, en relevant localStorage (clé « pepin.sauvegarde ») à chaque niveau fini ;
// puis recharge la page (porte-monnaie conservé ?), vide localStorage (retour à 0 ?) et
// relance avec un localStorage qui refuse tout (le jeu démarre quand même ?).
// Compte les erreurs de la console. Usage : node capture-sauvegarde.mjs PORT_HTTP DOSSIER_SORTIE
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
const sauvegarde = () => evalue('localStorage.getItem("pepin.sauvegarde")');
const ouvrir = async () => {
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(4000);
  await cdp('Page.bringToFront');
  await evalue('window.focus(); document.querySelector("canvas").focus(); 1');
  return evalue('typeof wasm_exports !== "undefined" && !!wasm_exports && getComputedStyle(document.getElementById("erreur")).display === "none"');
};

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
await cdp('Page.bringToFront');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });

// 1. Première visite : rien d'enregistré.
console.log('1. démarré :', await ouvrir(), '| sauvegarde :', JSON.stringify(await sauvegarde()));

// 2. Une partie complète rejouée ; on note chaque changement de la sauvegarde.
await evalue(fs.readFileSync(path.join(ici, 'pilote-qa.js'), 'utf8'));
const script = fs.readFileSync(path.join(ici, '..', 'target/qa/script.txt'), 'utf8').trim();
await evalue(`window.__P.script = ${JSON.stringify(script)}; window.__P.parRaf = 16; window.__P.cible = window.__P.script.length; window.__P.gel = 0; 1`);
let avant = null;
for (let i = 0; i < 6000; i++) {
  const s = await sauvegarde();
  if (s !== avant) {
    const poche = /gouttes_en_poche=(\d+)/.exec(s || '');
    console.log(`2. image ${await evalue('window.__P.n')} : gouttes_en_poche=${poche ? poche[1] : '-'}`);
    avant = s;
  }
  if (await evalue('(() => { const P = window.__P; return P.pos >= P.script.length && P.gel >= 3; })()')) break;
  await dormir(100);
}
console.log('2. fin du rejeu, erreurs du pilote :', JSON.stringify(await evalue('window.__P.erreurs')));
console.log('2. sauvegarde :', JSON.stringify(await sauvegarde()));
await capture('72-fin-nouveau-record');

// 3. Page rechargée : le porte-monnaie et le record sont toujours là.
console.log('3. démarré :', await ouvrir(), '| sauvegarde :', JSON.stringify(await sauvegarde()));
await capture('72-titre-gouttes-record');

// 4. localStorage vidé : retour à 0, sans erreur.
await evalue('localStorage.clear(); 1');
console.log('4. démarré :', await ouvrir(), '| sauvegarde :', JSON.stringify(await sauvegarde()));

// 5. localStorage refusé (comme certains modes privés) : le jeu démarre quand même.
await cdp('Page.addScriptToEvaluateOnNewDocument', {
  source: 'Object.defineProperty(window, "localStorage", { get() { throw new DOMException("refusé", "SecurityError"); } });',
});
console.log('5. démarré (localStorage refusé) :', await ouvrir());

console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
