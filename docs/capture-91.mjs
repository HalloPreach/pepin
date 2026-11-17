// Pilote CDP de la carte 9.1 (notion de MONDE) : une sauvegarde où « Le retour du Grand
// Nuage » (`poursuite`) est fini ouvre le Monde 2.
// - Build de QA (`./build.sh --qa`, faux Monde 2 = copie des toits) : titre avec la ligne
//   MONDE 2 ; carte des niveaux sur le Monde 2, bouton « MONDE 1 », Droite pour y revenir ;
//   le faux niveau joué depuis la carte et fini (niveau 1 du robot rejoué) → bilan ;
//   titre → MONDE 2 : carte de titre « MONDE 2 · NIVEAU 1/1 », partie finie → écran de fin,
//   records `partie.monde2`.
// - Build de production (`./build.sh`) : même sauvegarde, aucune trace du Monde 2 (ni ligne
//   au titre, ni bouton, ni Droite qui mène ailleurs).
// Relève localStorage et compte les erreurs de la console.
// Usage : node capture-91.mjs PORT_HTTP DOSSIER_SORTIE qa|prod SCRIPT IMAGE_FIN_N1
//   IMAGE_FIN_N1 : nombre d'images du script (depuis l'Entrée) jusqu'à la première image
//   hors de l'écran de jeu après le niveau 1.
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ici = path.dirname(fileURLToPath(import.meta.url));
const [httpPort, sortie, mode, fichierScript, imageFin] = process.argv.slice(2);
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
  await evalue('window.__P.file.push({ n: 60, m: 0 }); 1');
  await dormir(1500);
  return evalue('typeof wasm_exports !== "undefined" && !!wasm_exports && getComputedStyle(document.getElementById("erreur")).display === "none"');
};
const vide = async () => {
  for (let i = 0; i < 400 && await evalue('window.__P.file.length') > 0; i++) await dormir(50);
  await dormir(200);
};
// Appuie sur une touche (une image), puis laisse passer `n` images sans rien.
const touche = async (code, n = 20) => {
  await evalue(`window.__P.file.push({ n: ${n}, m: 0, taper: [${JSON.stringify(code)}] }); 1`);
  await vide();
};
// Clic de souris au point logique (x, y) de l'image 320x180, puis `n` images.
const cliquer = async (x, y, n = 20) => {
  const r = await evalue('(() => { const c = document.querySelector("canvas").getBoundingClientRect(); return [c.left, c.top, c.width, c.height]; })()');
  const k = Math.max(1, Math.min(Math.floor(r[2] / 320), Math.floor(r[3] / 180)));
  const x0 = r[0] + Math.floor((r[2] - 320 * k) / 2), y0 = r[1] + Math.floor((r[3] - 180 * k) / 2);
  const p = { x: x0 + (x + 0.5) * k, y: y0 + (y + 0.5) * k, button: 'left', clickCount: 1 };
  await cdp('Input.dispatchMouseEvent', { type: 'mouseMoved', ...p });
  await cdp('Input.dispatchMouseEvent', { type: 'mousePressed', ...p });
  await evalue('window.__P.file.push({ n: 2, m: 0 }); 1');
  await vide();
  await cdp('Input.dispatchMouseEvent', { type: 'mouseReleased', ...p });
  await evalue(`window.__P.file.push({ n: ${n}, m: 0 }); 1`);
  await vide();
};
// Rejoue le script du robot depuis son début (sa 1re image appuie sur Entrée) jusqu'à
// l'image `jusqua`, puis laisse passer `apres` images.
const script = fs.readFileSync(fichierScript, 'utf8').trim();
const rejouer = async (jusqua, apres = 60, depuis = 0) => {
  await evalue(`window.__P.script = ${JSON.stringify(script)}; window.__P.pos = ${depuis}; window.__P.cible = ${jusqua}; window.__P.parRaf = 16; 1`);
  for (let i = 0; i < 6000; i++) {
    if (await evalue('window.__P.pos >= window.__P.cible')) break;
    await dormir(100);
  }
  await evalue(`window.__P.file.push({ n: ${apres}, m: 0 }); 1`);
  await vide();
};
const ecrire = (texte) => evalue(`localStorage.setItem("pepin.sauvegarde", ${JSON.stringify(texte)}); 1`);

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
await cdp('Page.bringToFront');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });

// Sauvegarde v3 : « Le retour du Grand Nuage » fini (la porte du Monde 2), aucune partie finie.
await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
await dormir(1500);
await evalue('localStorage.clear(); 1');
await ecrire('version=3\nniveau.poursuite=atteint:1 gouttes:111 defis:100 medaille:1 temps:2000\n');
console.log(`1. ${mode} démarré :`, await ouvrir(), '| sauvegarde :', await sauvegarde());
await capture(`91-${mode}-titre`);

if (mode === 'qa') {
  // 2. Titre : JOUER, MONDE 2, CARTE... → CARTE : la carte s'ouvre sur le Monde 2 (le plus
  // loin des niveaux jouables), bouton « MONDE 1 » en bas.
  await touche('ArrowDown', 6);
  await touche('ArrowDown', 6);
  await touche('Enter', 60);
  await capture('91-qa-carte-monde-2');
  // Le bouton « MONDE 1 » : retour au Monde 1, bouton « MONDE 2 ».
  await cliquer(70 + 29, 166 + 6, 30);
  await capture('91-qa-carte-monde-1-bouton');
  // Droite, Droite : JOUER → COURSE → Monde 2 (sur JOUER).
  await touche('ArrowRight', 6);
  await touche('ArrowRight', 30);
  await capture('91-qa-carte-monde-2-par-droite');
  // 3. JOUER sur le faux niveau (Entrée = 1re image du script), joué jusqu'au drapeau.
  await rejouer(120, 0);
  await capture('91-qa-niveau-seul-carte-titre');
  await rejouer(Number(imageFin), 60, 120);
  console.log('3. erreurs du pilote :', JSON.stringify(await evalue('window.__P.erreurs')));
  await capture('91-qa-bilan-monde-2');
  console.log('3. sauvegarde :', await sauvegarde());
  // 4. CARTE (2e ligne du bilan) puis Échap : le titre, curseur sur CARTE ; Haut : MONDE 2.
  await touche('ArrowDown', 10);
  await touche('Enter', 80);
  await touche('Escape', 60);
  await touche('ArrowUp', 10);
  await rejouer(120, 0);
  await capture('91-qa-partie-monde-2-carte-titre');
  await rejouer(Number(imageFin), 150, 120);
  await capture('91-qa-fin-monde-2');
  console.log('4. sauvegarde :', await sauvegarde());
} else {
  // Production : CARTE est la 2e ligne ; la carte reste au Monde 1, Droite n'y change rien.
  await touche('ArrowDown', 6);
  await touche('Enter', 60);
  await capture('91-prod-carte');
  await touche('ArrowRight', 6);
  await touche('ArrowRight', 6);
  await touche('ArrowRight', 30);
  await capture('91-prod-carte-apres-droite');
  console.log('2. sauvegarde :', await sauvegarde());
}

console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
