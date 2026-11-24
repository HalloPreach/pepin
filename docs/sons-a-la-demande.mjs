// Sons téléchargés À LA DEMANDE (carte 10.4f) : le jeu ne demande AUCUN fichier
// audio tant que le son est coupé ; activé, il demande d'abord le bip des menus, la
// musique de la zone en cours et la pluie, puis les bruitages, et une autre musique
// seulement quand sa zone arrive. Et la musique doit démarrer MOINS D'UNE SECONDE
// après l'activation.
//
// Trois scénarios, chacun dans un navigateur NEUF (profil vide, cache coupé) :
// - `coupe` : écran titre 4 s, puis Entrée x3 (le premier niveau, les toits), 6 s de
//   jeu ; le son n'est jamais activé. On compte les requêtes audio (`musiques/`,
//   `sons/`) : il en faut ZÉRO.
// - `titre` : touche M à l'écran titre (musique Orage).
// - `niveau` : Entrée x3 (le premier niveau), 3 s de jeu son coupé, puis touche M.
// - `zone` : touche M au titre, puis Entrée x3 : en entrant dans le premier niveau,
//   la musique de sa zone (`toits_de_nuit`) doit partir, et elle seule.
// Pour `titre` et `niveau` : les fichiers demandés, dans l'ordre, avec leur instant
// (ms après l'appui sur M), et le RETARD de la musique : de l'appui sur M (vu par la
// page) au lancement de sa boucle (`AudioBufferSourceNode.start` d'une boucle de plus
// de 5 s : seules les musiques durent autant). Puis 6 s d'attente : aucune autre
// musique ne doit partir.
// Pour chaque scénario : le poids reçu (corps des réponses, `encodedDataLength`),
// total et audio seul. Le serveur de test (`python3 -m http.server`) ne compresse
// pas : en prod, wasm et JS arrivent en gzip (voir le tableau de `./build.sh`).
// Usage : node docs/sons-a-la-demande.mjs PORT_HTTP [coupe|titre|niveau|zone...]
// (variable CHROME ; dist/ servi sur 127.0.0.1:PORT_HTTP).
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const [httpPort, ...choix] = process.argv.slice(2);
if (!httpPort) {
  console.error('Usage : node docs/sons-a-la-demande.mjs PORT_HTTP [coupe|titre|niveau|zone...]');
  process.exit(2);
}
const SCENARIOS = choix.length ? choix : ['coupe', 'titre', 'niveau', 'zone'];
const CHROME = process.env.CHROME || 'chromium';
const dormir = (ms) => new Promise((r) => setTimeout(r, ms));

// Avant tout script de la page : l'instant de l'appui sur M, et chaque boucle lancée.
const SONDE = `
window.__S = { m: null, boucles: [] };
window.addEventListener('keydown', (e) => { if (e.code === 'KeyM' && window.__S.m === null) window.__S.m = performance.now(); }, true);
const lancer = AudioBufferSourceNode.prototype.start;
AudioBufferSourceNode.prototype.start = function (...a) {
  if (this.loop && this.buffer) window.__S.boucles.push([performance.now(), +this.buffer.duration.toFixed(2)]);
  return lancer.apply(this, a);
};`;

const touches = {
  Enter: { key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13, text: '\r' },
  KeyM: { key: 'm', code: 'KeyM', windowsVirtualKeyCode: 77, text: 'm' },
};

async function scenario(nom) {
  const dbg = 9400 + Math.floor(Math.random() * 400);
  const profil = fs.mkdtempSync(path.join(process.env.TMPDIR || '/tmp', 'prof-sons-'));
  const chrome = spawn(CHROME, [
    `--remote-debugging-port=${dbg}`, `--user-data-dir=${profil}`, '--headless=new', '--no-sandbox',
    '--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist',
    '--autoplay-policy=no-user-gesture-required',
    '--window-size=960,540', '--force-device-scale-factor=1', '--mute-audio', 'about:blank',
  ], { stdio: 'ignore' });
  try {
    let cible;
    for (let i = 0; i < 75 && !cible; i++) {
      try { cible = (await (await fetch(`http://127.0.0.1:${dbg}/json`)).json()).find((t) => t.type === 'page'); } catch {}
      if (!cible) await dormir(200);
    }
    if (!cible) throw new Error('Chromium ne répond pas (variable CHROME ?)');
    const ws = new WebSocket(cible.webSocketDebuggerUrl);
    await new Promise((r) => (ws.onopen = r));
    let id = 0; const att = new Map(); const erreurs = [];
    // Requêtes : url, instant d'envoi (horloge de la page, ms), octets reçus.
    const requetes = new Map();
    let origine = null; // Network.timestamp (s) <-> performance.now() (ms)
    ws.onmessage = (m) => {
      const d = JSON.parse(m.data);
      if (d.id && att.has(d.id)) { att.get(d.id)(d); att.delete(d.id); }
      if (d.method === 'Network.requestWillBeSent') {
        requetes.set(d.params.requestId, { url: d.params.request.url, t: d.params.timestamp, octets: 0 });
      }
      if (d.method === 'Network.loadingFinished' && requetes.has(d.params.requestId)) {
        requetes.get(d.params.requestId).octets = d.params.encodedDataLength;
      }
      if (d.method === 'Runtime.exceptionThrown') erreurs.push(d.params.exceptionDetails.text);
      if (d.method === 'Runtime.consoleAPICalled' && d.params.type === 'error') {
        erreurs.push(d.params.args.map((a) => a.value ?? a.description).join(' '));
      }
    };
    const cdp = (method, params = {}) => new Promise((r) => { const i = ++id; att.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
    const evalue = async (expr) => (await cdp('Runtime.evaluate', { expression: expr, returnByValue: true })).result?.result?.value;
    const taper = async (t) => {
      for (const type of ['keyDown', 'keyUp']) {
        await cdp('Input.dispatchKeyEvent', { type, ...touches[t], text: type === 'keyDown' ? touches[t].text : undefined });
      }
    };
    await cdp('Page.enable');
    await cdp('Runtime.enable');
    await cdp('Network.enable');
    await cdp('Network.setCacheDisabled', { cacheDisabled: true });
    await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
    await cdp('Page.addScriptToEvaluateOnNewDocument', { source: `try { localStorage.clear(); } catch (e) {}\n${SONDE}` });
    await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
    let pret = false;
    for (let i = 0; i < 300 && !pret; i++) {
      await dormir(100);
      pret = await evalue('typeof wasm_exports !== "undefined" && !!wasm_exports');
    }
    if (!pret) throw new Error('le jeu ne démarre pas');
    await dormir(1500);
    await cdp('Page.bringToFront');
    await evalue('window.focus(); document.querySelector("canvas").focus(); 1');
    // Correspondance des horloges : instant réseau d'une requête lancée par la page.
    const avant = await evalue('fetch("index.html?horloge").then(() => 0), performance.now()');
    await dormir(300);
    const reqHorloge = [...requetes.values()].find((r) => r.url.endsWith('?horloge'));
    origine = reqHorloge.t * 1000 - avant;
    const page = (r) => r.t * 1000 - origine;
    const audio = (r) => /\/(musiques|sons)\//.test(r.url);

    let lignes = [];
    if (nom === 'titre' || nom === 'zone') {
      await dormir(1000);
    } else {
      if (nom === 'coupe') await dormir(4000);
      await taper('Enter'); await dormir(2000);
      await taper('Enter'); await dormir(1000);
      await taper('Enter'); await dormir(3000);
    }
    const audioAvant = [...requetes.values()].filter(audio).length;
    lignes.push(`requêtes audio avant toute activation : ${audioAvant}`);
    let retard = null;
    let zone = null;
    if (nom === 'coupe') {
      await dormir(3000);
    } else {
      await taper('KeyM');
      // Attend la musique (au plus 5 s), puis 6 s de plus.
      for (let i = 0; i < 50; i++) {
        await dormir(100);
        const s = JSON.parse(await evalue('JSON.stringify(window.__S)'));
        if (s.boucles.some(([, d]) => d > 5)) break;
      }
      await dormir(6000);
      const s = JSON.parse(await evalue('JSON.stringify(window.__S)'));
      const musique = s.boucles.find(([, d]) => d > 5);
      retard = musique ? Math.round(musique[0] - s.m) : null;
      const apres = [...requetes.values()].filter(audio).sort((a, b) => a.t - b.t);
      lignes.push(`fichiers demandés après M (ms après l'appui) : ${apres.map((r) => `${r.url.replace(/.*\/(musiques|sons)\//, '$1/')} +${Math.round(page(r) - s.m)}`).join(', ')}`);
      lignes.push(`boucles lancées (durée s, ms après M) : ${s.boucles.map(([t, d]) => `${d} s +${Math.round(t - s.m)}`).join(', ')}`);
      lignes.push(`RETARD DE LA MUSIQUE : ${retard === null ? 'JAMAIS LANCÉE' : `${retard} ms`}`);
      if (nom === 'zone') {
        // Son actif au titre, puis le premier niveau (zone des toits) : SA musique
        // part alors, et elle seule.
        const avantNiveau = requetes.size;
        await taper('Enter'); await dormir(2000);
        await taper('Enter'); await dormir(1000);
        await taper('Enter'); await dormir(4000);
        const nouvelles = [...requetes.values()].slice(avantNiveau).filter(audio);
        const s2 = JSON.parse(await evalue('JSON.stringify(window.__S)'));
        zone = nouvelles.map((r) => r.url.replace(/.*\/(musiques|sons)\//, '$1/'));
        lignes.push(`fichiers demandés en entrant dans le niveau : ${zone.join(', ') || 'aucun'}`);
        lignes.push(`boucles lancées depuis M (durée s) : ${s2.boucles.map(([, d]) => d).join(', ')}`);
      }
    }
    const toutes = [...requetes.values()].filter((r) => !r.url.endsWith('?horloge'));
    const somme = (l) => l.reduce((a, r) => a + r.octets, 0);
    const lAudio = toutes.filter(audio);
    lignes.push(`reçu : ${somme(toutes)} octets en ${toutes.length} requêtes, dont audio ${somme(lAudio)} octets en ${lAudio.length} fichiers`);
    lignes.push(`erreurs console : ${erreurs.length} ${erreurs.length ? JSON.stringify(erreurs) : ''}`);
    ws.close();
    return { lignes, retard, audioAvant, nAudio: lAudio.length, zone };
  } finally {
    try { chrome.kill('SIGKILL'); } catch {}
    await dormir(300);
    fs.rmSync(profil, { recursive: true, force: true });
  }
}

let code = 0;
for (const nom of SCENARIOS) {
  const r = await scenario(nom);
  console.log(`== ${nom}`);
  for (const l of r.lignes) console.log(`   ${l}`);
  if (nom === 'coupe' && r.nAudio !== 0) code = 1;
  if (nom !== 'coupe' && (r.retard === null || r.retard > 1000)) code = 1;
  if (nom === 'zone' && JSON.stringify(r.zone) !== JSON.stringify(['musiques/toits_de_nuit.wav'])) code = 1;
}
console.log(code === 0 ? 'SONS-A-LA-DEMANDE-OK' : 'SONS-A-LA-DEMANDE-ECHEC');
process.exit(code);
