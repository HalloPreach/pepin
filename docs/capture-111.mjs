// Pilote CDP de la carte 11.1 (prototype : se balancer au parapluie), sur le build
// de QA servi par un serveur de fichiers statiques (dist-qa/) :
//  1. CONSTRUIT le niveau d'essai « essai_balancement » dans l'éditeur, comme une
//     personne : bouton Nouveau, boutons « + à droite » / « + en haut », palette,
//     outils Pinceau et Rectangle à la souris sur la grille, formulaire au clavier,
//     bouton Valider (le chargeur Rust du jeu), bouton ROBOT (le robot de
//     faisabilité dans le navigateur), puis « Exporter le .ron » (téléchargé dans
//     DOSSIER_SORTIE/essai_balancement.ron) ;
//  2. joue les cinq situations (clic droit « partir d'ici » près d'un crochet, puis
//     le clavier dans le jeu) et capture chacune : 111-1-sol, 111-2-trou,
//     111-3-enchaines, 111-4-montee, 111-5-plane.
// Usage : node docs/capture-111.mjs PORT_HTTP DOSSIER_SORTIE   (CHROME=chemin de chromium)
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const [httpPort, sortie] = process.argv.slice(2);
const CHROME = process.env.CHROME || 'chromium';
const dbg = 9400 + Math.floor(Math.random() * 400);
const profil = fs.mkdtempSync(path.join(process.env.TMPDIR || '/tmp', 'prof-'));
const chrome = spawn(CHROME, [
  `--remote-debugging-port=${dbg}`, `--user-data-dir=${profil}`, '--headless=new', '--no-sandbox',
  '--window-size=2000,900', '--force-device-scale-factor=1', 'about:blank',
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
  if (d.method === 'Page.javascriptDialogOpening') cdp('Page.handleJavaScriptDialog', { accept: true });
};
const cdp = (method, params = {}) => new Promise((r) => { const i = ++id; att.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
const evalue = async (expr) => (await cdp('Runtime.evaluate', { expression: expr, returnByValue: true, awaitPromise: true })).result?.result?.value;
const capture = async (nom) => {
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  fs.writeFileSync(path.join(sortie, `${nom}.png`), Buffer.from(r.result.data, 'base64'));
  console.log('capture', nom);
};
// Le bouton est d'abord amené à l'écran (la palette est plus haute que la fenêtre).
const centre = (sel) => evalue(`(() => { const e = document.querySelector(${JSON.stringify(sel)}); e.scrollIntoView({ block: 'center' }); const r = e.getBoundingClientRect(); return [r.x + r.width / 2, r.y + r.height / 2]; })()`);
const souris = (type, x, y, button = 'left') => cdp('Input.dispatchMouseEvent', { type, x, y, button, clickCount: 1 });
const cliquer = async (sel) => { const [x, y] = await centre(sel); await souris('mousePressed', x, y); await souris('mouseReleased', x, y); };
const attendre = async (expr, ms = 30000) => {
  for (let t = 0; t < ms; t += 250) { if (await evalue(expr)) return true; await dormir(250); }
  return false;
};
let echecs = 0;
const verifier = (nom, ok, detail = '') => { console.log(`${ok ? 'OK   ' : 'ECHEC'} ${nom} ${detail}`); if (!ok) echecs++; };

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Emulation.setDeviceMetricsOverride', { width: 2000, height: 900, deviceScaleFactor: 1, mobile: false });
await cdp('Browser.setDownloadBehavior', { behavior: 'allow', downloadPath: path.resolve(sortie) });
fs.mkdirSync(sortie, { recursive: true });
const base = `http://127.0.0.1:${httpPort}`;
await cdp('Page.navigate', { url: `${base}/editeur.html` });
await dormir(800);
await evalue('localStorage.clear(); 1');
await cdp('Page.navigate', { url: `${base}/editeur.html` });
await attendre('window.editeurPepin && editeurPepin.etat.grille.length > 0');

/// Centre de la case (x, y) de la grille, à l'écran.
const caseEcran = async (x, y) => {
  const [l, t, z] = await evalue('(() => { const r = document.querySelector("#toile").getBoundingClientRect(); return [r.left, r.top, editeurPepin.etat.zoom]; })()');
  return [l + x * z + z / 2, t + y * z + z / 2];
};
// OUVRIR=essais/essai_balancement.ron : ouvrir le niveau déjà exporté et ne
// rejouer que les captures du jeu (partie 2).
const OUVRIR = process.env.OUVRIR;
if (OUVRIR) {
  await cdp('Page.navigate', { url: `${base}/editeur.html?niveau=${OUVRIR}` });
  await attendre('window.editeurPepin && editeurPepin.etat.grille.length === 14');
  for (let i = 0; i < 2; i++) await cliquer('#zoom-moins');
}
// --- 1. Construire le niveau dans l'éditeur ---------------------------------------
if (!OUVRIR) {
await cliquer('#nouveau');
await dormir(200);
// Zoom 12 : la grille de 104 cases (1248 px) tient dans la fenêtre.
for (let i = 0; i < 2; i++) await cliquer('#zoom-moins');
for (let i = 0; i < 44; i++) await cliquer('[data-taille="droite+"]');
for (let i = 0; i < 2; i++) await cliquer('[data-taille="haut+"]');
const taille = await evalue('[editeurPepin.etat.grille[0].length, editeurPepin.etat.grille.length]');
verifier('grille 104 x 14', taille[0] === 104 && taille[1] === 14, JSON.stringify(taille));

const choisirLettre = (c) => cliquer(`.palette button[data-lettre=${JSON.stringify(c)}]`);
const outil = (o) => cliquer(`#boutons-outils button[data-outil="${o}"]`);
/// Outil Rectangle, lettre `c`, de la case (x0, y0) à (x1, y1) : appui, glisser, relâcher.
async function rectangle(c, x0, y0, x1, y1) {
  await choisirLettre(c);
  await outil('rectangle');
  const [ax, ay] = await caseEcran(x0, y0);
  const [bx, by] = await caseEcran(x1, y1);
  await souris('mousePressed', ax, ay);
  await souris('mouseMoved', bx, by);
  await souris('mouseReleased', bx, by);
}
/// Pinceau, lettre `c`, un clic sur chaque case.
async function pinceau(c, cases) {
  await choisirLettre(c);
  await outil('pinceau');
  for (const [x, y] of cases) {
    const [ex, ey] = await caseEcran(x, y);
    await souris('mousePressed', ex, ey);
    await souris('mouseReleased', ex, ey);
  }
}
// Les trous (vide) et le sol qui manque, la corniche, le sol du drapeau.
await rectangle('.', 18, 12, 28, 13);
await rectangle('.', 37, 12, 52, 13);
await pinceau('#', [[60, 12]]);
await pinceau('B', [[60, 13]]);
await rectangle('#', 74, 5, 79, 13);
await rectangle('#', 96, 12, 103, 12);
await rectangle('B', 96, 13, 103, 13);
// Repères, lanternes, gouttes, crochets.
await pinceau('X', [[100, 11]]);
await pinceau('L', [[3, 11], [30, 11], [54, 11], [75, 4]]);
await pinceau('o', [[12, 8], [47, 10], [77, 4]]);
await pinceau('y', [[10, 7], [23, 7], [41, 7], [47, 7], [63, 7], [67, 5], [71, 3], [81, 3]]);

// Le formulaire, au clavier (sélection du champ, texte tapé, Tab : l'événement change).
async function taper(nom, texte) {
  await evalue(`(() => { const c = document.querySelector('#formulaire [name=${nom}]'); c.focus(); c.select(); return 1; })()`);
  await cdp('Input.insertText', { text: texte });
  await cdp('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Tab', code: 'Tab', windowsVirtualKeyCode: 9 });
  await cdp('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Tab', code: 'Tab', windowsVirtualKeyCode: 9 });
}
await taper('cle', 'essai_balancement');
await taper('nom', 'Essai balancement');
await taper('duree_cible_s', '60');
await taper('intro', "Saute, tiens SAUT près d'un crochet : ton parapluie s'y accroche.");
await taper('mecanique', "Se balancer au crochet, prendre de l'élan, lâcher et planer.");
const champs = await evalue('JSON.stringify({cle: editeurPepin.etat.niveau.cle, theme: editeurPepin.etat.niveau.theme, duree: editeurPepin.etat.niveau.duree_cible_s})');
verifier('formulaire', champs.includes('essai_balancement') && champs.includes('Toits'), champs);

// Valider : le chargeur du jeu.
await evalue('document.querySelector("#verdict").textContent = ""; 1');
await cliquer('#valider');
await attendre('!document.querySelector("#verdict").hidden && document.querySelector("#verdict").textContent.length > 0');
const verdict = await evalue('document.querySelector("#verdict").textContent');
verifier('Valider', verdict.includes('accepté'), verdict);
// ROBOT : le robot de faisabilité, dans le navigateur.
await cliquer('#robot');
await attendre('!document.querySelector("#robot").disabled', 600000);
const robot = await evalue('document.querySelector("#robot-texte").textContent');
verifier('ROBOT', /arriv/i.test(robot) && !/échec|ECHEC/i.test(robot), robot);
await capture('111-editeur-robot');
// Exporter.
await cliquer('#exporter');
const fichier = path.join(sortie, 'essai_balancement.ron');
for (let i = 0; i < 40 && !fs.existsSync(fichier); i++) await dormir(250);
verifier('export', fs.existsSync(fichier), fichier);
}

// --- 2. Les cinq situations, dans le jeu ------------------------------------------
const touche = (type, key, code, vk) => cdp('Input.dispatchKeyEvent', { type, key, code, windowsVirtualKeyCode: vk });
const T = {
  saut: [' ', 'Space', 32], droite: ['ArrowRight', 'ArrowRight', 39], gauche: ['ArrowLeft', 'ArrowLeft', 37],
};
const appuyer = (t) => touche('keyDown', ...T[t]);
const lacher = (t) => touche('keyUp', ...T[t]);
/// Une suite d'actions : [touche, 'bas' | 'haut'] ou un nombre (ms d'attente).
async function jouer(suite) {
  for (const a of suite) {
    if (typeof a === 'number') await dormir(a);
    else if (a[1] === 'bas') await appuyer(a[0]);
    else await lacher(a[0]);
  }
}
/// Clic droit « partir d'ici » en (x, y), attend que le jeu tourne.
async function partir(x, y) {
  const [ex, ey] = await caseEcran(x, y);
  await souris('mousePressed', ex, ey, 'right');
  await souris('mouseReleased', ex, ey, 'right');
  const ok = await attendre('document.querySelector("#essai") && !document.querySelector("#essai").hidden && !document.querySelector("#essai").classList.contains("cache")', 15000);
  if (!ok) console.log('verdict :', await evalue('document.querySelector("#verdict").textContent'));
  await dormir(400);
  // La carte du niveau (titre, intro) : Entrée pour commencer.
  await touche('keyDown', 'Enter', 'Enter', 13);
  await touche('keyUp', 'Enter', 'Enter', 13);
  await dormir(700);
  return ok;
}
const fermer = async () => { await touche('keyDown', 'Escape', 'Escape', 27); await touche('keyUp', 'Escape', 'Escape', 27); await dormir(600); await evalue('document.querySelector("#essai").hidden || window.editeurPepin && 1'); };

const SITUATIONS = (process.env.SITUATIONS && JSON.parse(fs.readFileSync(process.env.SITUATIONS, 'utf8'))) || [
  // nom, case de départ (clic droit « partir d'ici »), suite d'actions jusqu'à la
  // capture (touches tenues ou lâchées, attentes en ms ; le jeu tourne en temps réel,
  // le résultat varie donc un peu d'une fois à l'autre : regarder chaque capture).
  ['111-1-sol', [8, 9], [['saut', 'bas'], ['droite', 'bas'], 700, ['droite', 'haut'], ['gauche', 'bas'], 500, ['gauche', 'haut'], ['droite', 'bas'], 350]],
  ['111-2-trou', [21, 9], [['saut', 'bas'], ['droite', 'bas'], 700, ['droite', 'haut'], ['gauche', 'bas'], 450]],
  // Accroché au 1er crochet, élan, lâcher vers la droite, SAUT de nouveau : le 2e.
  ['111-3-enchaines', [39, 9], [['saut', 'bas'], ['droite', 'bas'], 680, ['droite', 'haut'], ['gauche', 'bas'], 700, ['gauche', 'haut'], ['droite', 'bas'], 330, ['saut', 'haut'], 90, ['saut', 'bas'], 700]],
  // Au crochet du milieu de la montée : le 1er en bas à gauche, le 3e en haut à droite.
  ['111-4-montee', [66, 7], [['saut', 'bas'], ['droite', 'bas'], 600, ['droite', 'haut'], ['gauche', 'bas'], 400]],
  // Au crochet du bord de la corniche : élan, lâcher, SAUT de nouveau : le plané.
  ['111-5-plane', [80, 5], [['saut', 'bas'], ['droite', 'bas'], 680, ['droite', 'haut'], ['gauche', 'bas'], 700, ['gauche', 'haut'], ['droite', 'bas'], 250, ['saut', 'haut'], 60, ['saut', 'bas'], 900]],
];
for (const [nom, [x, y], suite] of SITUATIONS) {
  const ok = await partir(x, y);
  verifier(`partir d'ici ${nom}`, ok);
  await jouer(suite);
  await capture(nom);
  for (const t of Object.keys(T)) await lacher(t);
  await fermer();
}

verifier('aucune erreur JavaScript', erreurs.length === 0, erreurs.join(' | '));
console.log(echecs ? `${echecs} échec(s)` : 'tout est vert');
chrome.kill();
process.exit(echecs ? 1 : 0);
