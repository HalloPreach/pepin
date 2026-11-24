// Pilote CDP de la carte 10.3b (éditeur : TESTER, validation par le jeu, partir
// d'ici), sur le build de QA servi par un serveur de fichiers statiques (dist-qa/) :
//  1. ouvre « toits » dans l'éditeur, MODIFIE le niveau (une rangée de toit `#`
//     au-dessus du départ), clique TESTER : le jeu (index.html?essai=1, dans le
//     cadre de l'éditeur) valide le niveau avec son chargeur Rust, dit « pret »,
//     se montre en plein écran ; captures `103b-essai-carte` (la carte du niveau,
//     avec SON intro) et `103b-essai`. Échap (lu par le jeu) ramène à
//     l'éditeur ;
//  2. pose une lettre FAUSSE (une 4e goutte ne suffit pas : un pigeon `p` en
//     plein ciel, sans perchoir) et clique Valider : le jeu refuse, l'éditeur
//     montre le message et SURLIGNE la case ; capture `103b-erreur` ;
//  3. clic droit sur une case vide loin du départ : « partir d'ici », Pépin
//     posé là ; capture `103b-partir-d-ici` ;
//  4. partir juste à gauche du drapeau X et courir : la FIN du niveau ramène à
//     l'éditeur ; capture `103b-retour-apres-la-fin`.
// Usage : node docs/capture-103b.mjs PORT_HTTP DOSSIER_SORTIE   (CHROME=chemin de chromium)
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const [httpPort, sortie] = process.argv.slice(2);
const CHROME = process.env.CHROME || 'chromium';
const dbg = 9400 + Math.floor(Math.random() * 400);
const profil = fs.mkdtempSync(path.join(process.env.TMPDIR || '/tmp', 'prof-'));
const chrome = spawn(CHROME, [
  `--remote-debugging-port=${dbg}`, `--user-data-dir=${profil}`, '--headless=new', '--no-sandbox',
  '--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist',
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
  if (d.method === 'Page.javascriptDialogOpening') cdp('Page.handleJavaScriptDialog', { accept: true });
};
const cdp = (method, params = {}) => new Promise((r) => { const i = ++id; att.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
const evalue = async (expr) => (await cdp('Runtime.evaluate', { expression: expr, returnByValue: true, awaitPromise: true })).result?.result?.value;
const capture = async (nom) => {
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  fs.writeFileSync(path.join(sortie, `${nom}.png`), Buffer.from(r.result.data, 'base64'));
  console.log('capture', nom);
};
const souris = (type, x, y, extra = {}) => cdp('Input.dispatchMouseEvent', { type, x, y, button: 'left', clickCount: 1, ...extra });
const cliquer = async (x, y, bouton = 'left') => {
  await souris('mousePressed', x, y, { button: bouton });
  await souris('mouseReleased', x, y, { button: bouton });
};
const centre = (sel) => evalue(`(() => { const r = document.querySelector(${JSON.stringify(sel)}).getBoundingClientRect(); return [r.x + r.width / 2, r.y + r.height / 2]; })()`);
const touche = async (key, code, vk) => {
  await cdp('Input.dispatchKeyEvent', { type: 'keyDown', key, code, windowsVirtualKeyCode: vk });
  await dormir(80);
  await cdp('Input.dispatchKeyEvent', { type: 'keyUp', key, code, windowsVirtualKeyCode: vk });
};
/// Attend que `expr` soit vrai (au plus `ms`).
const attendre = async (expr, ms = 20000) => {
  for (let t = 0; t < ms; t += 200) { if (await evalue(expr)) return true; await dormir(200); }
  return false;
};
let echecs = 0;
const verifier = (nom, ok, detail = '') => { console.log(`${ok ? 'OK   ' : 'ECHEC'} ${nom} ${detail}`); if (!ok) echecs++; };

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Emulation.setDeviceMetricsOverride', { width: 1400, height: 820, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });
const base = `http://127.0.0.1:${httpPort}`;
await cdp('Page.navigate', { url: `${base}/editeur.html` });
await dormir(1000);
await evalue('localStorage.clear(); 1');
await cdp('Page.navigate', { url: `${base}/editeur.html?niveau=monde_1/toits.ron` });
await dormir(1500);

// 1. Modifier, TESTER, jouer, Échap.
const depart = await evalue(`(() => { const g = editeurPepin.etat.grille;
  for (let y = 0; y < g.length; y++) { const x = g[y].indexOf('@'); if (x >= 0) return [x, y]; } })()`);
await evalue(`(() => { const e = editeurPepin; e.memoriser();
  for (let x = ${depart[0]} - 1; x <= ${depart[0]} + 6; x++) e.poser(x, ${depart[1]} - 3, '#');
  e.toutRafraichir(); return 1; })()`);
const modifie = await evalue('editeurPepin.texteExporte()');
await cliquer(...(await centre('#tester')));
const pret = await attendre('!document.querySelector("#essai").hidden && !document.querySelector("#essai").classList.contains("cache")');
verifier('TESTER : le jeu accepte et se montre', pret, await evalue('document.querySelector("#verdict").textContent'));
verifier('le jeu a reçu le niveau modifié', (await evalue('localStorage.getItem("pepin.niveau_essai")')) === modifie);
await dormir(2500); // la transition d'entrée du niveau
await capture('103b-essai-carte'); // la carte du niveau : SON intro, SA mécanique
await touche('Enter', 'Enter', 13);
await dormir(1500);
await capture('103b-essai');
await touche('Escape', 'Escape', 27);
const retour = await attendre('document.querySelector("#essai").hidden', 5000);
verifier('Échap (lu par le jeu) ramène à l\'éditeur', retour
  && (await evalue('document.querySelector("#etat").textContent')) === 'essai fini par le jeu (Échap)',
  await evalue('document.querySelector("#etat").textContent'));

// 2. Une faute (pigeon sans perchoir, en plein ciel) : refusée, case surlignée.
const faute = [depart[0] + 3, 1];
await evalue(`(() => { const e = editeurPepin; e.memoriser(); e.poser(${faute[0]}, ${faute[1]}, 'p'); e.toutRafraichir(); return 1; })()`);
await cliquer(...(await centre('#valider')));
const refuse = await attendre('editeurPepin.etat.faute !== null');
const vue = await evalue('JSON.stringify(editeurPepin.etat.faute)');
verifier('Valider : le jeu refuse et la case est surlignée', refuse && vue === JSON.stringify({ x: faute[0], y: faute[1] }), `faute ${vue}`);
verifier('le message vient du chargeur Rust', (await evalue('document.querySelector("#verdict").textContent')).includes('pigeon'),
  await evalue('document.querySelector("#verdict").textContent'));
await capture('103b-erreur');
// Annuler la faute (Ctrl+Z) : le texte modifié revient.
await cdp('Input.dispatchKeyEvent', { type: 'keyDown', key: 'z', code: 'KeyZ', modifiers: 2, windowsVirtualKeyCode: 90 });
await cdp('Input.dispatchKeyEvent', { type: 'keyUp', key: 'z', code: 'KeyZ', modifiers: 2, windowsVirtualKeyCode: 90 });
verifier('Ctrl+Z retire la faute', (await evalue('editeurPepin.texteExporte()')) === modifie);

// 3. Partir d'ici : clic droit sur une case vide, plus loin dans le niveau.
const ici = await evalue(`(() => { const g = editeurPepin.etat.grille;
  // la première case vide posée sur un toit, à partir de la colonne 40
  for (let x = 40; x < g[0].length; x++) for (let y = 1; y < g.length - 1; y++)
    if (g[y][x] === '.' && g[y + 1][x] === '#' && g[y - 1][x] === '.') return [x, y]; })()`);
const z = await evalue('editeurPepin.etat.zoom');
await evalue(`(() => { const d = document.querySelector('#defilement'); d.scrollLeft = ${ici[0]} * ${z} - 300; return 1; })()`);
const toile = await evalue('(() => { const r = document.querySelector("#toile").getBoundingClientRect(); return [r.x, r.y]; })()');
await cliquer(toile[0] + (ici[0] + 0.5) * z, toile[1] + (ici[1] + 0.5) * z, 'right');
const pret2 = await attendre('!document.querySelector("#essai").hidden && !document.querySelector("#essai").classList.contains("cache")');
const envoye = await evalue('localStorage.getItem("pepin.niveau_essai")');
const lignes = envoye.split('ascii: r#"')[1].split('"#')[0].split('\n').filter((l) => l.length);
verifier('partir d\'ici : Pépin posé sur la case choisie', pret2 && lignes[ici[1]][ici[0]] === '@'
  && lignes.join('').split('@').length === 2, `case (${ici})`);
verifier('la grille de l\'éditeur garde son départ', (await evalue('editeurPepin.texteExporte()')) === modifie);
await dormir(2500);
await touche('Enter', 'Enter', 13);
await dormir(1500);
await capture('103b-partir-d-ici');
await touche('Escape', 'Escape', 27);
verifier('Échap ramène à l\'éditeur (2)', await attendre('document.querySelector("#essai").hidden', 5000));

// 4. La fin du niveau ramène aussi à l'éditeur : partir juste à gauche du drapeau
//    X, passer la carte du niveau (Entrée), courir à droite.
const drapeau = await evalue(`(() => { const g = editeurPepin.etat.grille;
  for (let y = 0; y < g.length; y++) { const x = g[y].indexOf('X'); if (x >= 0) return [x, y]; } })()`);
let avant = null;
for (let d = 1; d <= 6 && drapeau; d++) {
  const c = await evalue(`editeurPepin.etat.grille[${drapeau[1]}][${drapeau[0] - d}]`);
  if (c === '.') { avant = [drapeau[0] - d, drapeau[1]]; break; }
}
verifier('une case vide à gauche du drapeau', !!avant, `drapeau (${drapeau}) case (${avant})`);
if (avant) {
  await evalue(`editeurPepin.partirDIci({ x: ${avant[0]}, y: ${avant[1]} }); 1`);
  await attendre('!document.querySelector("#essai").hidden && !document.querySelector("#essai").classList.contains("cache")');
  await dormir(1500);
  await touche('Enter', 'Enter', 13);
  await dormir(1500);
  await cdp('Input.dispatchKeyEvent', { type: 'keyDown', key: 'ArrowRight', code: 'ArrowRight', windowsVirtualKeyCode: 39 });
  const fin = await attendre('document.querySelector("#essai").hidden', 20000);
  await cdp('Input.dispatchKeyEvent', { type: 'keyUp', key: 'ArrowRight', code: 'ArrowRight', windowsVirtualKeyCode: 39 });
  verifier('la fin du niveau ramène à l\'éditeur', fin
    && (await evalue('document.querySelector("#etat").textContent')) === 'essai fini : niveau terminé',
    await evalue('document.querySelector("#etat").textContent'));
  await capture('103b-retour-apres-la-fin');
}

verifier('aucune erreur de console', erreurs.length === 0, erreurs.join(' | '));
console.log(echecs === 0 ? 'ESSAI-OK' : `${echecs} échec(s)`);
ws.close();
chrome.kill();
await dormir(500);
try { fs.rmSync(profil, { recursive: true, force: true }); } catch { /* Chromium écrit encore */ }
process.exit(echecs === 0 ? 0 : 1);
