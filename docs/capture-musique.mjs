// Pilote CDP de la carte 8.7 (une musique par zone) : build de QA, son activé (M), puis
// raccourcis de niveau 1 → 2 (même zone : la boucle ne doit PAS repartir), 3 → 4
// (ville → orage) et 5 → 6 (orage → cyclone). Un crochet posé AVANT le chargement de la
// page note chaque boucle (AudioBufferSourceNode avec `loop`) lancée ou arrêtée, avec
// une empreinte de son tampon (durée + somme des 4 410 premiers échantillons) : on voit
// quelle musique tourne sans l'entendre. Horloge virtuelle de docs/pilote-qa.js ;
// compte les erreurs de la console.
// Usage : node capture-musique.mjs PORT_HTTP DOSSIER_SORTIE
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
  '--autoplay-policy=no-user-gesture-required',
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
// Appuie sur une touche (une image), puis laisse passer `n` images sans rien.
const touche = async (code, n = 20) => {
  await evalue(`window.__P.file.push({ n: ${n}, m: 0, taper: [${JSON.stringify(code)}] }); 1`);
  for (let i = 0; i < 400 && await evalue('window.__P.file.length') > 0; i++) await dormir(50);
  await dormir(300);
};

// Le crochet audio : posé avant tout script de la page.
const CROCHET = `(() => {
  const M = window.__M = { actives: new Set(), journal: [] };
  const empreinte = (b) => {
    if (!b) return 'vide';
    const d = b.getChannelData(0); let s = 0;
    for (let i = 0; i < Math.min(4410, d.length); i++) s += Math.abs(d[i]);
    return b.duration.toFixed(2) + 's/' + s.toFixed(1);
  };
  const P = AudioBufferSourceNode.prototype;
  const start = P.start, deco = P.disconnect;
  P.start = function (...a) {
    if (this.loop) { this.__e = empreinte(this.buffer); M.actives.add(this); M.journal.push('+' + this.__e); }
    return start.apply(this, a);
  };
  P.disconnect = function (...a) {
    if (M.actives.delete(this)) M.journal.push('-' + this.__e);
    return deco.apply(this, a);
  };
  M.lire = () => ({ actives: [...M.actives].map((s) => s.__e).sort(), journal: M.journal.splice(0) });
})();`;

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
await cdp('Page.addScriptToEvaluateOnNewDocument', { source: CROCHET });
await cdp('Page.bringToFront');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });

await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
await dormir(4000);
await evalue('window.focus(); document.querySelector("canvas").focus(); 1');
await evalue(fs.readFileSync(path.join(ici, 'pilote-qa.js'), 'utf8'));
await evalue('window.__P.file.push({ n: 60, m: 0 }); 1');
await dormir(1500);

const etapes = [];
const noter = async (quoi) => {
  const r = await evalue('JSON.stringify(window.__M.lire())');
  const e = { quoi, ...JSON.parse(r) };
  etapes.push(e);
  console.log(quoi.padEnd(34), 'actives', JSON.stringify(e.actives), 'journal', JSON.stringify(e.journal));
};
// Raccourci de niveau (en jeu), puis Entrée sur la carte de présentation.
const niveau = async (n) => {
  await touche(`Digit${n}`, 120);
  await noter(`carte du niveau ${n}`);
  await touche('Enter', 120);
  await noter(`niveau ${n} en jeu`);
};

await touche('KeyM', 30);
await noter('titre, son actif (M)');
await touche('Enter', 200); // JOUER : carte du niveau 1
await touche('Enter', 120);
await noter('niveau 1 en jeu');
await niveau(2);
await niveau(3);
await niveau(4);
await capture('87-niveau4-orage');
await niveau(5);
await niveau(6);
await capture('87-niveau6-cyclone');
await touche('Escape', 30);
await noter('pause au niveau 6');
await touche('Escape', 30);
await noter('reprise au niveau 6');

// Vérifications. La pluie (2,00 s) tourne avec toute musique ; on la retire. Durées
// attendues : Toits de nuit 32 croches x 0,36 s = 11,52 s ; Cyclone 64 x 0,15 = 9,60 s
// (Orage dure aussi 9,60 s, mais son empreinte est celle du titre).
const musique = (quoi) => etapes.find((e) => e.quoi === quoi).actives.filter((x) => !x.startsWith('2.00s'));
const relances = (quoi) => etapes.find((e) => e.quoi === quoi).journal.filter((x) => x.startsWith('+') && !x.includes('2.00s'));
const [orage] = musique('titre, son actif (M)');
const [toits] = musique('niveau 1 en jeu');
const [cyclone] = musique('niveau 6 en jeu');
const verifs = [
  ['titre : Orage (une seule musique)', musique('titre, son actif (M)').length === 1],
  ['niveau 1 : Toits de nuit (11,52 s)', toits?.startsWith('11.52s') && toits !== orage],
  ['niveaux 2 et 3 : même boucle, pas relancée', ['carte du niveau 2', 'niveau 2 en jeu', 'carte du niveau 3', 'niveau 3 en jeu']
    .every((q) => musique(q).join() === toits && relances(q).length === 0)],
  ['niveaux 4 et 5 : Orage', ['niveau 4 en jeu', 'niveau 5 en jeu'].every((q) => musique(q).join() === orage)],
  ['niveau 6 : Cyclone (9,60 s, autre que Orage)', cyclone?.startsWith('9.60s') && cyclone !== orage],
  ['pause : musique coupée', musique('pause au niveau 6').length === 0],
  ['reprise : Cyclone relancé', musique('reprise au niveau 6').join() === cyclone],
];
for (const [quoi, ok] of verifs) console.log(ok ? 'OK    ' : 'ÉCHEC ', quoi);
fs.writeFileSync(path.join(sortie, '87-musique-journal.json'), JSON.stringify({ etapes, verifs, erreurs }, null, 1));
console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
