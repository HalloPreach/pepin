// Pilote CDP pour les corrections de visibilité : rejoue le script du robot de QA
// (target/qa/script.txt) avec l'horloge virtuelle de docs/pilote-qa.js et capture l'écran.
// Usage : node capture-visibilite.mjs PORT_HTTP DOSSIER_SORTIE demande [demande ...]
// Demandes (par ordre d'image croissant : le rejeu ne revient pas en arrière) :
//   nom=IMAGE                 une capture à l'image IMAGE du parcours du robot
//   suite:prefixe=IMAGE:K     K captures sur K images consécutives à partir de IMAGE
//   chute:prefixe=IMAGE:K:PAS à IMAGE, le robot lâche son script : Pépin court tout droit (donc
//                             tombe dans un trou) ; K captures, une toutes les PAS images
//   attente:prefixe=IMAGE:K:PAS à IMAGE, le robot lâche son script et ne touche plus à rien :
//                             Pépin reste immobile ; K captures, une toutes les PAS images
//   touche:nom=IMAGE:CODE:N   à IMAGE, le robot lâche son script ; touche CODE appuyée une
//                             image, N images sans rien, puis capture (nom « - » : aucune)
// Variable d'environnement GRILLE=1 : écrit aussi la grille logique 320x180 de chaque capture
// (un caractère = un pixel logique, indice hexadécimal de palette) dans NOM.txt.
// Variable d'environnement SCRIPT : autre script à rejouer (défaut : target/qa/script.txt), par
// exemple target/qa/script-31b.txt (réapparition protégée à la lanterne 104, voir QA.md).
// Variable d'environnement TOUCHES : touches à appuyer pendant le rejeu, sans lâcher le script,
// « IMAGE:CODE[,IMAGE:CODE...] » ou un fichier qui en contient (une par ligne) ; par exemple
// TOUCHES=target/qa/touches-52.txt (touche 0 du build de QA : niveau d'essai vertical).
// Variable d'environnement FENETRE : taille de la fenêtre (défaut 960x540, image agrandie x3) ;
// FENETRE=1280x720 donne une image agrandie x4.
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ici = path.dirname(fileURLToPath(import.meta.url));
const [httpPort, sortie, ...demandes] = process.argv.slice(2);
const CHROME = process.env.CHROME || 'chromium';
const [largeur, hauteur] = (process.env.FENETRE || '960x540').split('x').map(Number);
const dbg = 9400 + Math.floor(Math.random() * 400);
const profil = fs.mkdtempSync(path.join(process.env.TMPDIR || '/tmp', 'prof-'));
const chrome = spawn(CHROME, [
  `--remote-debugging-port=${dbg}`, `--user-data-dir=${profil}`, '--headless=new', '--no-sandbox',
  '--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist',
  `--window-size=${largeur},${hauteur}`, '--force-device-scale-factor=1', 'about:blank',
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
ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.id && att.has(d.id)) { att.get(d.id)(d); att.delete(d.id); } };
const cdp = (method, params = {}) => new Promise((r) => { const i = ++id; att.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
const evalue = async (expr) => (await cdp('Runtime.evaluate', { expression: expr, returnByValue: true })).result?.result?.value;

await cdp('Page.enable');
await cdp('Page.bringToFront');
await cdp('Emulation.setDeviceMetricsOverride', { width: largeur, height: hauteur, deviceScaleFactor: 1, mobile: false });
await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
await dormir(4000);
await cdp('Page.bringToFront');
await evalue('window.focus(); document.querySelector("canvas").focus(); 1');
await evalue(fs.readFileSync(path.join(ici, 'pilote-qa.js'), 'utf8'));
const script = fs.readFileSync(process.env.SCRIPT || path.join(ici, '..', 'target/qa/script.txt'), 'utf8').trim();
await evalue(`window.__P.script = ${JSON.stringify(script)}; 1`);
if (process.env.TOUCHES) {
  const texte = fs.existsSync(process.env.TOUCHES)
    ? fs.readFileSync(process.env.TOUCHES, 'utf8')
    : process.env.TOUCHES;
  const touches = {};
  for (const t of texte.split(/[\s,]+/).filter(Boolean)) {
    const [image, code] = t.split(':');
    touches[Number(image)] = code;
  }
  await evalue(`window.__P.touches = ${JSON.stringify(touches)}; 1`);
}
fs.mkdirSync(sortie, { recursive: true });

const jusqua = async (image) => {
  await evalue(`(() => { const P = window.__P; P.parRaf = 8; P.cible = ${image}; P.gel = 0; return 1; })()`);
  for (let i = 0; i < 6000; i++) {
    const ok = await evalue('(() => { const P = window.__P; return P.pos >= Math.min(P.cible, P.script.length) && P.file.length === 0 && P.gel >= 3; })()');
    if (ok) break;
    await dormir(50);
  }
};
const capturer = async (nom) => {
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  fs.writeFileSync(path.join(sortie, `${nom}.png`), Buffer.from(r.result.data, 'base64'));
  if (process.env.GRILLE) {
    const g = JSON.parse(await evalue('window.__P.lire()'));
    fs.writeFileSync(path.join(sortie, `${nom}.txt`), g.grille.match(/.{320}/g).join('\n') + '\n');
  }
  console.log('capture', nom, 'image', await evalue('window.__P.n'));
};

for (const d of demandes) {
  if (d.startsWith('suite:')) {
    const [nom, reste] = d.slice(6).split('=');
    const [image, k] = reste.split(':').map(Number);
    for (let i = 0; i < k; i++) {
      await jusqua(image + i + 1);
      await capturer(`${nom}-${String(i).padStart(2, '0')}`);
    }
  } else if (d.startsWith('chute:') || d.startsWith('attente:')) {
    // chute : touche « droite » (masque 4) ; attente : aucune touche (masque 0).
    const masque = d.startsWith('chute:') ? 4 : 0;
    const [nom, reste] = d.slice(d.indexOf(':') + 1).split('=');
    const [image, k, pas] = reste.split(':').map(Number);
    await jusqua(image);
    // Le script du robot s'arrête ici ; on pousse nos propres images à la place.
    await evalue('(() => { const P = window.__P; P.script = P.script.slice(0, P.pos); return 1; })()');
    for (let i = 0; i < k; i++) {
      await evalue(`(() => { const P = window.__P; P.parRaf = 1; P.gel = 0; P.file.push({ m: ${masque}, n: ${pas} }); return 1; })()`);
      for (let j = 0; j < 400; j++) {
        if (await evalue('(() => { const P = window.__P; return P.file.length === 0 && P.gel >= 3; })()')) break;
        await dormir(50);
      }
      await capturer(`${nom}-${String(i).padStart(2, '0')}`);
    }
  } else if (d.startsWith('touche:')) {
    // touche:nom=IMAGE:CODE:N : à IMAGE, le robot lâche son script ; la touche CODE (code
    // DOM : PageDown, KeyN, Digit5, Enter...) est appuyée pendant une image, puis N images
    // passent sans rien toucher, et on capture (nom « - » : pas de capture). Sert aux
    // raccourcis du build de QA ; les demandes suivantes doivent aussi être des « touche: ».
    const [nom, reste] = d.slice(7).split('=');
    const [image, code, n] = reste.split(':');
    await jusqua(Number(image));
    await evalue('(() => { const P = window.__P; P.script = P.script.slice(0, P.pos); return 1; })()');
    await evalue(`(() => { const P = window.__P; P.parRaf = 8; P.gel = 0;
      P.file.push({ taper: [${JSON.stringify(code)}], m: 0, n: 1 }, { m: 0, n: ${Number(n)} }); return 1; })()`);
    for (let j = 0; j < 400; j++) {
      if (await evalue('(() => { const P = window.__P; return P.file.length === 0 && P.gel >= 3; })()')) break;
      await dormir(50);
    }
    if (nom !== '-') await capturer(nom);
  } else {
    const [nom, image] = d.split('=');
    await jusqua(Number(image) + 1);
    await capturer(nom);
  }
}
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
