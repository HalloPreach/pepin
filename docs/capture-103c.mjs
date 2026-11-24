// Pilote CDP de la carte 10.3c (éditeur : bouton ROBOT, le robot de faisabilité
// de `core` compilé en wasm et lancé dans un Web Worker), sur le build de QA servi
// par un serveur de fichiers statiques (dist-qa/) :
//  1. ouvre « port », clique ROBOT : le chemin trouvé est dessiné sur la grille ;
//     capture `103c-chemin` ;
//  2. mesure la durée du robot (dans le navigateur) sur toits, port et pieuvre ;
//  3. « toits » volontairement CASSÉ (un mur de toit `#` du haut en bas) : le robot
//     échoue, l'endroit le plus loin atteint est cerclé ; capture `103c-echec` ;
//  4. le défi sans planer (parapluie interdit) corrige le champ `sans_planer`
//     (mis faux exprès) par le bouton « Recopier dans sans_planer » ;
//  5. le bouton Arrêter termine le Worker (la page reste libre pendant la recherche).
// Usage : node docs/capture-103c.mjs PORT_HTTP DOSSIER_SORTIE   (CHROME=chemin de chromium)
// LIMITE_S=secondes : limite du robot pour les mesures (600 par défaut).
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const [httpPort, sortie] = process.argv.slice(2);
const CHROME = process.env.CHROME || 'chromium';
const LIMITE_S = Number(process.env.LIMITE_S || 600);
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
  if (d.method === 'Page.javascriptDialogOpening') cdp('Page.handleJavaScriptDialog', { accept: true });
};
const cdp = (method, params = {}) => new Promise((r) => { const i = ++id; att.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
const evalue = async (expr) => (await cdp('Runtime.evaluate', { expression: expr, returnByValue: true, awaitPromise: true })).result?.result?.value;
const capture = async (nom) => {
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  fs.writeFileSync(path.join(sortie, `${nom}.png`), Buffer.from(r.result.data, 'base64'));
  console.log('capture', nom);
};
const centre = (sel) => evalue(`(() => { const r = document.querySelector(${JSON.stringify(sel)}).getBoundingClientRect(); return [r.x + r.width / 2, r.y + r.height / 2]; })()`);
const cliquer = async (x, y) => {
  await cdp('Input.dispatchMouseEvent', { type: 'mousePressed', x, y, button: 'left', clickCount: 1 });
  await cdp('Input.dispatchMouseEvent', { type: 'mouseReleased', x, y, button: 'left', clickCount: 1 });
};
const attendre = async (expr, ms = 20000) => {
  for (let t = 0; t < ms; t += 250) { if (await evalue(expr)) return true; await dormir(250); }
  return false;
};
let echecs = 0;
const verifier = (nom, ok, detail = '') => { console.log(`${ok ? 'OK   ' : 'ECHEC'} ${nom} ${detail}`); if (!ok) echecs++; };
const texteRobot = () => evalue('document.querySelector("#robot-texte").textContent');
const fini = '!document.querySelector("#robot").disabled';

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Emulation.setDeviceMetricsOverride', { width: 1400, height: 820, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });
const base = `http://127.0.0.1:${httpPort}`;
await cdp('Page.navigate', { url: `${base}/editeur.html` });
await dormir(1000);
await evalue('localStorage.clear(); 1');

/// Ouvre `chemin`, lance le robot (limite `limite` s) et rend le résultat.
async function robotSur(chemin, { limite = LIMITE_S, zoom = null, sansPlaner = false, casser = null } = {}) {
  await cdp('Page.navigate', { url: `${base}/editeur.html?niveau=${chemin}` });
  await attendre('window.editeurPepin && editeurPepin.etat.niveau && editeurPepin.etat.grille.length > 0');
  await dormir(300);
  if (zoom) await evalue(`(() => { editeurPepin.etat.zoom = ${zoom}; editeurPepin.toutRafraichir(); return 1; })()`);
  if (casser) await evalue(casser);
  await evalue(`(() => { document.querySelector('#robot-limite').value = ${limite};
    document.querySelector('#robot-sans-planer').checked = ${sansPlaner}; return 1; })()`);
  const debut = Date.now();
  await cliquer(...(await centre('#robot')));
  const ok = await attendre(fini, (limite + 5) * 1000);
  const mur = ((Date.now() - debut) / 1000).toFixed(1);
  const r = await evalue('JSON.stringify(editeurPepin.etat.robot)');
  return { ok, mur, r: r ? JSON.parse(r) : null, texte: await texteRobot() };
}

// 1. Port : le chemin trouvé.
const port = await robotSur('monde_2/port.ron', { zoom: 6 });
verifier('port : le robot finit et trouve l\'arrivée', port.ok && port.r?.arrivee === true, port.texte);
verifier('port : le chemin est dessiné (plusieurs points)', (port.r?.chemin.length ?? 0) > 5, `${port.r?.chemin.length} points`);
await capture('103c-chemin');

// 2. Mesures (durée du robot dans le Worker, et durée vue par la page).
const mesures = { port };
for (const [nom, chemin] of [['toits', 'monde_1/toits.ron'], ['pieuvre', 'monde_2/pieuvre.ron']]) {
  mesures[nom] = await robotSur(chemin);
}
for (const [nom, m] of Object.entries(mesures)) {
  console.log(`MESURE ${nom} : ${m.r ? `robot ${(m.r.duree_ms / 1000).toFixed(1)} s, ${m.r.etats} états, arrivée ${m.r.arrivee}` : 'pas de résultat'} (page : ${m.mur} s) | ${m.texte}`);
}

// 3. Toits cassé : un mur de toit du haut en bas, au tiers du niveau.
const casser = `(() => { const e = editeurPepin, g = e.etat.grille; e.memoriser();
  const x = Math.floor(g[0].length / 3); for (let y = 0; y < g.length; y++) e.poser(x, y, '#');
  e.toutRafraichir(); return 1; })()`;
const casse = await robotSur('monde_1/toits.ron', { limite: 120, zoom: 10, casser });
const mur = await evalue('Math.floor(editeurPepin.etat.grille[0].length / 3)');
const bloque = casse.r ? Math.floor((casse.r.dernier[0] + casse.r.joueur[0] / 2) / casse.r.tuile) : null;
verifier('toits cassé : le robot échoue et dit pourquoi', casse.ok && casse.r?.arrivee === false && casse.r.raison.length > 0, casse.texte);
verifier('toits cassé : bloqué juste avant le mur', bloque !== null && bloque < mur && bloque >= mur - 3, `colonne ${bloque}, mur ${mur}`);
await capture('103c-echec');

// 4. Défi sans planer sur toits : le résultat remplit `sans_planer`.
// Le champ est d'abord mis FAUX exprès (vrai) : le bouton doit le corriger.
const fausser = `(() => { const e = editeurPepin; e.memoriser(); e.etat.niveau.sans_planer = true; e.toutRafraichir(); return 1; })()`;
const defi = await robotSur('monde_1/toits.ron', { limite: 120, sansPlaner: true, casser: fausser });
const avant = await evalue('editeurPepin.etat.niveau.sans_planer');
const bouton = await evalue('!document.querySelector("#robot-remplir").hidden');
if (bouton) await cliquer(...(await centre('#robot-remplir')));
const apres = await evalue('editeurPepin.etat.niveau.sans_planer');
verifier('défi sans planer : verdict recopié dans sans_planer', defi.r?.sansPlaner === true && bouton && avant !== apres && apres === defi.r.arrivee
  && (await evalue('editeurPepin.texteExporte()')).includes(`sans_planer: ${apres},`),
  `robot ${defi.r?.arrivee}, avant ${avant}, bouton ${bouton}, après ${apres}`);

// 5. Arrêter : la page répond pendant la recherche, puis le Worker est terminé.
await cdp('Page.navigate', { url: `${base}/editeur.html?niveau=monde_2/pieuvre.ron` });
await attendre('window.editeurPepin && editeurPepin.etat.grille.length > 0');
await cliquer(...(await centre('#robot')));
await dormir(1500);
const vivante = await evalue('1 + 1');
await cliquer(...(await centre('#robot-arreter')));
verifier('Arrêter : la page répond pendant la recherche, puis le robot s\'arrête', vivante === 2 && (await evalue(fini))
  && (await texteRobot()) === 'ROBOT arrêté', await texteRobot());

verifier('aucune erreur JavaScript', erreurs.length === 0, erreurs.join(' | '));
ws.close();
chrome.kill();
console.log(echecs ? `${echecs} ÉCHEC(S)` : 'TOUT OK');
process.exit(echecs ? 1 : 0);
