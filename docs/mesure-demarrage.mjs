// Temps de démarrage du jeu (carte 10.4a) : du début du chargement de la page à
// l'écran titre. Le chargement des sprites (décodage des PNG, textures) se fait dans
// les toutes premières images du jeu ; on mesure donc, avec performance.now() (0 =
// début de la navigation) :
// - `wasm` : la fin de l'instanciation du module (le jeu est téléchargé et compilé) ;
// - `titre` : la fin de la 3e image du jeu (l'écran titre est dessiné, les premières
//   images, celles du chargement, sont passées) ;
// - `max` : la plus longue des 10 premières images (celle qui charge les sprites) ;
// - `sons` (carte 10.4e) : la fin du décodage du DERNIER son par le navigateur
//   (`decodeAudioData`). Depuis la reprise de la carte 10.4e, les sons sont des WAV
//   servis à côté du jeu et téléchargés APRÈS la première image : le titre ne les
//   attend plus, `sons` dit quand le dernier est prêt. On compte les sons décodés et
//   les ÉCHECS de décodage (un son que le navigateur ne sait pas lire).
// - `dessin` (carte 10.4e) : la fin de la PREMIÈRE image qui dessine quelque chose
//   (un appel de dessin WebGL) : l'écran titre apparaît. Avant la reprise de
//   10.4e, les images d'attente des sons étaient vides : `titre` ne suffisait plus.
// Chaque mesure se fait dans un navigateur NEUF (profil vide, pas de cache).
// Usage : node docs/mesure-demarrage.mjs PORT_HTTP [ESSAIS=5]  (variable CHROME).
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const [httpPort, essaisTexte] = process.argv.slice(2);
if (!httpPort) {
  console.error('Usage : node docs/mesure-demarrage.mjs PORT_HTTP [ESSAIS]');
  process.exit(2);
}
const ESSAIS = Number(essaisTexte || 5);
const CHROME = process.env.CHROME || 'chromium';
const dormir = (ms) => new Promise((r) => setTimeout(r, ms));

// Installé avant tout script de la page : on chronomètre chaque rappel de
// requestAnimationFrame (une image du jeu) et l'instanciation du wasm.
const SONDE = `
window.__D = { images: [], wasm: null, dessin: null, dessine: false, sons: { debut: 0, fin: 0, echecs: 0, dernier: null, longueurs: [] } };
const decoderOrig = BaseAudioContext.prototype.decodeAudioData;
BaseAudioContext.prototype.decodeAudioData = function (octets, reussite, echec) {
  const s = window.__D.sons;
  s.debut++;
  return decoderOrig.call(this, octets,
    (b) => { s.fin++; s.dernier = performance.now(); s.longueurs.push(b.length); if (reussite) reussite(b); },
    (e) => { s.echecs++; if (echec) echec(e); });
};
// Tout appel de dessin WebGL lève le drapeau. miniquad (WebGL 1) dessine par
// « instances », via l'extension ANGLE_instanced_arrays : on enveloppe aussi les
// fonctions de l'extension, au moment où le jeu la demande.
const marquer = (objet, nom) => {
  const orig = objet[nom];
  if (typeof orig === 'function') objet[nom] = function (...a) { window.__D.dessine = true; return orig.apply(this, a); };
};
for (const C of [window.WebGLRenderingContext, window.WebGL2RenderingContext]) {
  if (!C) continue;
  for (const nom of ['drawElements', 'drawArrays', 'drawElementsInstanced', 'drawArraysInstanced']) marquer(C.prototype, nom);
  const extension = C.prototype.getExtension;
  C.prototype.getExtension = function (nom) {
    const e = extension.call(this, nom);
    if (e && nom === 'ANGLE_instanced_arrays' && !e.__marquee) {
      e.__marquee = true;
      marquer(e, 'drawElementsInstancedANGLE');
      marquer(e, 'drawArraysInstancedANGLE');
    }
    return e;
  };
}
const rafOrig = window.requestAnimationFrame.bind(window);
window.requestAnimationFrame = (f) => rafOrig((t) => {
  const debut = performance.now();
  window.__D.dessine = false;
  f(t);
  if (window.__D.dessine && window.__D.dessin === null) window.__D.dessin = performance.now();
  if (window.__D.images.length < 10) window.__D.images.push([debut, performance.now()]);
});
for (const nom of ['instantiate', 'instantiateStreaming']) {
  const orig = WebAssembly[nom];
  if (orig) WebAssembly[nom] = function (...a) {
    return orig.apply(this, a).then((r) => { window.__D.wasm ??= performance.now(); return r; });
  };
}`;

async function essai() {
  const dbg = 9400 + Math.floor(Math.random() * 400);
  const profil = fs.mkdtempSync(path.join(process.env.TMPDIR || '/tmp', 'prof-demarrage-'));
  const chrome = spawn(CHROME, [
    `--remote-debugging-port=${dbg}`, `--user-data-dir=${profil}`, '--headless=new', '--no-sandbox',
    '--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist',
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
    let id = 0; const att = new Map();
    ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.id && att.has(d.id)) { att.get(d.id)(d); att.delete(d.id); } };
    const cdp = (method, params = {}) => new Promise((r) => { const i = ++id; att.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
    const evalue = async (expr) => (await cdp('Runtime.evaluate', { expression: expr, returnByValue: true })).result?.result?.value;
    await cdp('Page.enable');
    await cdp('Network.enable');
    await cdp('Network.setCacheDisabled', { cacheDisabled: true });
    await cdp('Page.addScriptToEvaluateOnNewDocument', { source: SONDE });
    await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
    let d;
    // Le jeu demande ses sons par petits groupes : on attend dix images ET une seconde
    // sans nouvelle demande, tous les sons demandés étant décodés.
    let demandes = -1, calme = 0;
    for (let i = 0; i < 300; i++) {
      await dormir(100);
      d = await evalue('JSON.stringify(window.__D)');
      const e = d && JSON.parse(d);
      if (!e) continue;
      calme = e.sons.debut === demandes ? calme + 1 : 0;
      demandes = e.sons.debut;
      if (e.images.length >= 10 && e.dessin !== null && calme >= 10 && e.sons.fin + e.sons.echecs === e.sons.debut) break;
    }
    ws.close();
    const { images, wasm, dessin, sons } = JSON.parse(d);
    if (images.length < 3) throw new Error('moins de 3 images');
    return {
      wasm: Math.round(wasm),
      titre: Math.round(images[2][1]),
      dessin: Math.round(dessin ?? -1),
      max: Math.round(Math.max(...images.map(([a, b]) => b - a))),
      sons: Math.round(sons.dernier ?? -1),
      nsons: `${sons.fin}/${sons.debut}`,
      echecs: sons.echecs,
      longueurs: sons.longueurs,
    };
  } finally {
    try { chrome.kill('SIGKILL'); } catch {}
    await dormir(300);
    fs.rmSync(profil, { recursive: true, force: true });
  }
}

const mesures = [];
for (let i = 0; i < ESSAIS; i++) {
  const m = await essai();
  mesures.push(m);
  console.log(`essai ${i + 1} : wasm ${m.wasm} ms, titre ${m.titre} ms, premier dessin ${m.dessin} ms, plus longue image ${m.max} ms, sons décodés ${m.nsons} à ${m.sons} ms, échecs ${m.echecs}`);
}
// Les longueurs (en échantillons) des sons décodés par le navigateur, dans l'ordre
// de chargement : à comparer à celles des WAV de référence (carte 10.4e).
console.log(`longueurs décodées (essai 1) : ${mesures[0].longueurs.join(' ')}`);
const mediane = (cle) => mesures.map((m) => m[cle]).sort((a, b) => a - b)[Math.floor(mesures.length / 2)];
console.log(`MÉDIANES (${ESSAIS} essais) : wasm ${mediane('wasm')} ms, titre ${mediane('titre')} ms, premier dessin ${mediane('dessin')} ms, plus longue image ${mediane('max')} ms, sons ${mediane('sons')} ms`);
