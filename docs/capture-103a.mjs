// Pilote CDP de la carte 10.3a (éditeur de niveaux, build de QA) : avec Chromium
// sans écran, ouvre `editeur.html` servi par un serveur de fichiers statiques
// (dist-qa/), puis :
//  1. ouvre CHAQUE niveau de `niveaux/index.json` dans l'éditeur et vérifie que le
//     texte exporté sans modification est le fichier d'origine (aller-retour dans le
//     navigateur), et liste les avertissements affichés ;
//  2. dessine à la souris (vraies actions : clic sur la palette, glisser sur la
//     grille), vérifie que la grille a changé, que le brouillon est dans le
//     localStorage, puis annule (Ctrl+Z) et retrouve le texte d'origine ;
//  3. capture `103a-grille` (toits) et `103a-proprietes` (port).
// Usage : node docs/capture-103a.mjs PORT_HTTP DOSSIER_SORTIE   (CHROME=chemin de chromium)
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const [httpPort, sortie] = process.argv.slice(2);
const CHROME = process.env.CHROME || 'chromium';
const dbg = 9400 + Math.floor(Math.random() * 400);
const profil = fs.mkdtempSync(path.join(process.env.TMPDIR || '/tmp', 'prof-'));
const chrome = spawn(CHROME, [
  `--remote-debugging-port=${dbg}`, `--user-data-dir=${profil}`, '--headless=new', '--no-sandbox',
  '--window-size=1400,820', '--force-device-scale-factor=1', 'about:blank',
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
  if (d.method === 'Runtime.exceptionThrown') erreurs.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text);
  if (d.method === 'Runtime.consoleAPICalled' && (d.params.type === 'error' || d.params.type === 'warning')) {
    erreurs.push(d.params.type + ' : ' + d.params.args.map((a) => a.value ?? a.description).join(' '));
  }
  if (d.method === 'Log.entryAdded' && d.params.entry.level === 'error') erreurs.push(d.params.entry.text);
};
const cdp = (method, params = {}) => new Promise((r) => { const i = ++id; att.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
const evalue = async (expr) => (await cdp('Runtime.evaluate', { expression: expr, returnByValue: true, awaitPromise: true })).result?.result?.value;
const capture = async (nom) => {
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  fs.writeFileSync(path.join(sortie, `${nom}.png`), Buffer.from(r.result.data, 'base64'));
  console.log('capture', nom);
};
const souris = (type, x, y, extra = {}) => cdp('Input.dispatchMouseEvent', { type, x, y, button: 'left', clickCount: 1, ...extra });
const cliquer = async (x, y) => { await souris('mousePressed', x, y); await souris('mouseReleased', x, y); };
const centre = (sel) => evalue(`(() => { const r = document.querySelector(${JSON.stringify(sel)}).getBoundingClientRect(); return [r.x + r.width / 2, r.y + r.height / 2]; })()`);
let echecs = 0;
const verifier = (nom, ok, detail = '') => { console.log(`${ok ? 'OK   ' : 'ECHEC'} ${nom} ${detail}`); if (!ok) echecs++; };

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
// « Quitter la page ? » (modifications non exportées) : on accepte, comme un humain qui recharge.
ws.addEventListener('message', (m) => {
  const d = JSON.parse(m.data);
  if (d.method === 'Page.javascriptDialogOpening') cdp('Page.handleJavaScriptDialog', { accept: true });
});
await cdp('Emulation.setDeviceMetricsOverride', { width: 1400, height: 820, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });
const base = `http://127.0.0.1:${httpPort}`;
await cdp('Page.navigate', { url: `${base}/editeur.html` });
await dormir(1000);
await evalue('localStorage.clear(); 1');

// 1. Chaque niveau : ouvert puis exporté sans modification = mêmes octets.
await cdp('Page.navigate', { url: `${base}/editeur.html` });
await dormir(1500);
const index = await evalue('fetch("niveaux/index.json").then((r) => r.json())');
let identiques = 0;
const avertis = {};
for (const chemin of index) {
  const r = await evalue(`(async () => {
    const texte = await (await fetch('niveaux/${chemin}')).text();
    editeurPepin.ouvrirTexte(texte, '${chemin}');
    return { pareil: editeurPepin.texteExporte() === texte,
             avert: [...document.querySelectorAll('#avertissements li:not(.ok)')].map((li) => li.textContent) };
  })()`);
  if (r.pareil) identiques++; else verifier(`aller-retour ${chemin}`, false);
  if (r.avert.length) avertis[chemin] = r.avert;
}
verifier(`aller-retour dans le navigateur`, identiques === index.length, `${identiques}/${index.length}`);
console.log('avertissements sur les niveaux du dépôt :', JSON.stringify(avertis, null, 1));

// 2. Dessiner à la souris sur toits, brouillon, puis annuler.
await cdp('Page.navigate', { url: `${base}/editeur.html?niveau=monde_1/toits.ron` });
await dormir(1500);
const original = await evalue('fetch("niveaux/monde_1/toits.ron").then((r) => r.text())');
const [bx, by] = await centre('.palette button[data-lettre="f"]');
await cliquer(bx, by);
const toile = await evalue('(() => { const r = document.querySelector("#toile").getBoundingClientRect(); return [r.x, r.y]; })()');
const z = await evalue('editeurPepin.etat.zoom');
// Glisser sur la rangée 3 (vide), des colonnes 20 à 24 : 5 flaques.
await souris('mousePressed', toile[0] + 20.5 * z, toile[1] + 3.5 * z);
for (let c = 21; c <= 24; c++) await souris('mouseMoved', toile[0] + (c + 0.5) * z, toile[1] + 3.5 * z, { buttons: 1 });
await souris('mouseReleased', toile[0] + 24.5 * z, toile[1] + 3.5 * z);
const rangee = await evalue('editeurPepin.etat.grille[3].slice(18, 27).join("")');
verifier('pinceau à la souris', rangee === '..fffff..', `rangée 3 : ${rangee}`);
const brouillon = await evalue('localStorage.getItem("pepin.editeur.brouillon") || ""');
verifier('brouillon sauvegardé', brouillon.includes('fffff') && brouillon !== original);
// Départ unique : poser un 2e @ efface l'ancien.
const [ax, ay] = await centre('.palette button[data-lettre="@"]');
await cliquer(ax, ay);
await cliquer(toile[0] + 30.5 * z, toile[1] + 3.5 * z);
const departs = await evalue('editeurPepin.etat.grille.flat().filter((c) => c === "@").length');
verifier('un seul départ', departs === 1, `${departs} @`);
for (let i = 0; i < 2; i++) {
  await cdp('Input.dispatchKeyEvent', { type: 'keyDown', key: 'z', code: 'KeyZ', modifiers: 2, windowsVirtualKeyCode: 90 });
  await cdp('Input.dispatchKeyEvent', { type: 'keyUp', key: 'z', code: 'KeyZ', modifiers: 2, windowsVirtualKeyCode: 90 });
}
verifier('annuler (Ctrl+Z deux fois) redonne le fichier', (await evalue('editeurPepin.texteExporte()')) === original);
// Rechargement de la page sans ?niveau : le brouillon est repris.
await cdp('Page.navigate', { url: `${base}/editeur.html` });
await dormir(1500);
verifier('brouillon repris au rechargement', (await evalue('editeurPepin.etat.niveau.cle')) === 'toits');

// 3. Captures.
await evalue('localStorage.clear(); 1');
await cdp('Page.navigate', { url: `${base}/editeur.html?niveau=monde_1/toits.ron` });
await dormir(1500);
await cliquer(...(await centre('.palette button[data-lettre="L"]')));
await souris('mouseMoved', toile[0] + 47.5 * z, toile[1] + 7.5 * z);
await capture('103a-grille');
await cdp('Page.navigate', { url: `${base}/editeur.html?niveau=monde_2/port.ron` });
await dormir(1500);
await evalue('document.querySelector("#defilement").scrollLeft = 2900; document.querySelector("#proprietes").scrollTop = 120; 1');
await capture('103a-proprietes');

// Chromium refuse d'afficher « Quitter la page ? » sans geste de l'utilisateur et le
// note dans la console : ce message vient du navigateur, pas de l'éditeur.
const vraies = erreurs.filter((e) => !e.includes("'beforeunload' confirmation panel"));
verifier('aucune erreur de console', vraies.length === 0, vraies.join(' | '));
console.log(echecs === 0 ? 'EDITEUR-OK' : `${echecs} échec(s)`);
ws.close();
chrome.kill();
await dormir(500);
try { fs.rmSync(profil, { recursive: true, force: true }); } catch { /* Chromium écrit encore : dossier temporaire laissé */ }
process.exit(echecs === 0 ? 0 : 1);
