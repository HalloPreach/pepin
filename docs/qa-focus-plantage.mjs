// Pilote CDP de la QA « pause automatique à la perte de focus » et « plantage en cours de partie ».
// Usage : CHROME=/chemin/chrome node docs/qa-focus-plantage.mjs SCENARIO PORT_HTTP PREFIXE
//
// Scénarios :
//   focus-jeu   : son activé au titre, niveau 1 lancé ; onglet « masqué » 6 s puis revenu ; mesure
//                 l'écran (pause ?), les boucles sonores (musique + pluie) et l'état de l'AudioContext ;
//                 puis Échap (reprise) ; puis fenêtre qui perd le focus en restant visible (blur).
//   focus-titre : mêmes pertes de focus sur l'écran titre : aucune pause ne doit apparaître.
//   brave       : en pleine partie, erreurs typiques des scripts injectés par un navigateur (portefeuille
//                 de Brave : window.ethereum.selectedAddress, « Script error. », promesse rejetée) :
//                 le jeu doit continuer, sans message.
//   plantage    : build de QA (`./build.sh --qa`) : en pleine partie, son activé, F9 provoque une vraie
//                 panique Rust ; le message plein écran et le bouton « Recharger » doivent apparaître,
//                 le son être coupé ; puis clic sur « Recharger » (la page revient au titre).
//   webgl       : en pleine partie, perte du contexte WebGL (extension WEBGL_lose_context).
//   wasm-bloque : game.wasm bloqué (Network.setBlockedURLs) : le message de chargement doit rester.
//
// Simulation de l'onglet masqué (Chromium sans écran ne masque jamais vraiment un onglet) : un script
// injecté avant la page force `document.visibilityState` à « hidden », envoie `visibilitychange`, et
// RETIENT les rappels de requestAnimationFrame comme le fait un vrai navigateur pour un onglet masqué
// (le jeu ne dessine plus). Au retour, il les relâche. La perte de focus « fenêtre visible » force
// `document.hasFocus()` à faux et envoie `blur`. Les boucles sonores sont comptées en observant
// AudioBufferSourceNode.start (sources en boucle) et AudioNode.disconnect (arrêt par macroquad).
// Écrit PREFIXE-*.png et affiche les mesures en JSON (une ligne par mesure).
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import zlib from 'node:zlib';

const [scenario, httpPort, prefixe] = process.argv.slice(2);
const CHROME = process.env.CHROME || 'chromium';
const dbg = 9400 + Math.floor(Math.random() * 400);
const profil = fs.mkdtempSync(path.join(process.env.TMPDIR || '/tmp', 'prof-'));
const chrome = spawn(CHROME, [
  `--remote-debugging-port=${dbg}`, `--user-data-dir=${profil}`, '--headless=new', '--no-sandbox',
  '--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist',
  '--autoplay-policy=no-user-gesture-required', '--window-size=1280,720', 'about:blank',
], { stdio: 'ignore' });
const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
let cible;
for (let i = 0; i < 60; i++) {
  try { const l = await (await fetch(`http://127.0.0.1:${dbg}/json`)).json(); cible = l.find((t) => t.type === 'page'); if (cible) break; } catch {}
  await dormir(200);
}
const ws = new WebSocket(cible.webSocketDebuggerUrl);
await new Promise((r) => (ws.onopen = r));
let id = 0; const att = new Map(); const consoleJeu = [];
ws.onmessage = (m) => {
  const d = JSON.parse(m.data);
  if (d.id && att.has(d.id)) { att.get(d.id)(d); att.delete(d.id); }
  if (d.method === 'Runtime.consoleAPICalled') consoleJeu.push(d.params.args.map((a) => a.value ?? a.description ?? '').join(' '));
  if (d.method === 'Runtime.exceptionThrown') consoleJeu.push('EXCEPTION ' + (d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text));
};
const cdp = (method, params = {}) => new Promise((r) => { const i = ++id; att.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
const evaluer = async (expr) => (await cdp('Runtime.evaluate', { expression: expr, returnByValue: true })).result?.result?.value;

const VK = { KeyM: 77, Enter: 13, Escape: 27, ArrowRight: 39, ArrowLeft: 37, Space: 32, F9: 120 };
const touche = (code, key, bas) => cdp('Input.dispatchKeyEvent', { type: bas ? 'keyDown' : 'keyUp', code, key, windowsVirtualKeyCode: VK[code], nativeVirtualKeyCode: VK[code] });
const taper = async (code, key, ms = 120) => { await touche(code, key, true); await dormir(ms); await touche(code, key, false); };
const capture = async (nom) => {
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  const buf = Buffer.from(r.result.data, 'base64');
  fs.writeFileSync(nom, buf);
  return buf;
};

// --- Décodage PNG minimal (RGB/RGBA 8 bits) pour comparer deux captures sans dépendance. ---
function decoderPng(buf) {
  let pos = 8; let l = 0, h = 0, couleur = 0; const idat = [];
  while (pos < buf.length) {
    const n = buf.readUInt32BE(pos); const type = buf.toString('ascii', pos + 4, pos + 8);
    const data = buf.subarray(pos + 8, pos + 8 + n);
    if (type === 'IHDR') { l = data.readUInt32BE(0); h = data.readUInt32BE(4); couleur = data[9]; }
    if (type === 'IDAT') idat.push(data);
    pos += 12 + n;
  }
  const bpp = couleur === 6 ? 4 : 3;
  const brut = zlib.inflateSync(Buffer.concat(idat));
  const px = Buffer.alloc(l * h * bpp); const stride = l * bpp;
  for (let y = 0; y < h; y++) {
    const f = brut[y * (stride + 1)]; const ligne = brut.subarray(y * (stride + 1) + 1, (y + 1) * (stride + 1));
    for (let x = 0; x < stride; x++) {
      const a = x >= bpp ? px[y * stride + x - bpp] : 0;
      const b = y > 0 ? px[(y - 1) * stride + x] : 0;
      const c = x >= bpp && y > 0 ? px[(y - 1) * stride + x - bpp] : 0;
      let v = ligne[x];
      if (f === 1) v += a; else if (f === 2) v += b; else if (f === 3) v += (a + b) >> 1;
      else if (f === 4) { const p = a + b - c, pa = Math.abs(p - a), pb = Math.abs(p - b), pc = Math.abs(p - c); v += pa <= pb && pa <= pc ? a : pb <= pc ? b : c; }
      px[y * stride + x] = v & 255;
    }
  }
  return { l, h, bpp, px };
}
// Part de pixels qui diffèrent entre deux captures (0 = identiques) ; `zone` en fractions de l'image.
function ecart(a, b, zone = [0, 0, 1, 1]) {
  const A = decoderPng(a), B = decoderPng(b); if (A.l !== B.l || A.h !== B.h) return 1;
  const [x0, y0, x1, y1] = [zone[0] * A.l, zone[1] * A.h, zone[2] * A.l, zone[3] * A.h].map(Math.floor);
  let n = 0;
  for (let y = y0; y < y1; y++) for (let x = x0; x < x1; x++) {
    const i = (y * A.l + x) * A.bpp;
    if (Math.abs(A.px[i] - B.px[i]) + Math.abs(A.px[i + 1] - B.px[i + 1]) + Math.abs(A.px[i + 2] - B.px[i + 2]) > 30) n++;
  }
  return Number((n / ((x1 - x0) * (y1 - y0))).toFixed(4));
}
const mesure = (nom, v) => console.log(JSON.stringify({ mesure: nom, ...v }));

// --- Observateur injecté AVANT la page (donc avant index.html et mq_js_bundle.js). ---
const OBSERVATEUR = `(() => {
  const C = window.AudioContext || window.webkitAudioContext;
  window.__qaContextes = [];
  const Obs = function (o) { const c = o === undefined ? new C() : new C(o); window.__qaContextes.push(c); return c; };
  Obs.prototype = C.prototype; window.AudioContext = Obs; window.webkitAudioContext = Obs;
  window.__qaBoucles = 0;
  const start = AudioBufferSourceNode.prototype.start;
  AudioBufferSourceNode.prototype.start = function (...a) { if (this.loop) { this.__qaBoucle = true; window.__qaBoucles++; } return start.apply(this, a); };
  const deco = AudioNode.prototype.disconnect;
  AudioNode.prototype.disconnect = function (...a) { if (this.__qaBoucle && !this.__qaFini) { this.__qaFini = true; window.__qaBoucles--; } return deco.apply(this, a); };
  let masque = false, flou = false, enAttente = [];
  const rafOrig = window.requestAnimationFrame.bind(window);
  const focusOrig = Document.prototype.hasFocus;
  Object.defineProperty(document, 'visibilityState', { get: () => (masque ? 'hidden' : 'visible'), configurable: true });
  Object.defineProperty(document, 'hidden', { get: () => masque, configurable: true });
  document.hasFocus = function () { return !masque && !flou && focusOrig.call(document); };
  window.requestAnimationFrame = (cb) => rafOrig((t) => { if (masque) enAttente.push(cb); else cb(t); });
  window.__qaMasquer = () => { masque = true; document.dispatchEvent(new Event('visibilitychange')); };
  window.__qaMontrer = () => { masque = false; document.dispatchEvent(new Event('visibilitychange'));
    const l = enAttente; enAttente = []; l.forEach((cb) => rafOrig(cb)); };
  window.__qaFlou = () => { flou = true; window.dispatchEvent(new Event('blur')); };
  window.__qaRetourFocus = () => { flou = false; window.dispatchEvent(new Event('focus')); };
  window.__qaAudio = () => ({ boucles: window.__qaBoucles, contextes: window.__qaContextes.map((c) => c.state) });
})();`;

await cdp('Page.enable');
await cdp('Runtime.enable');
if (scenario === 'wasm-bloque') {
  await cdp('Network.enable');
  await cdp('Network.setBlockedURLs', { urls: ['*game.wasm'] });
}
await cdp('Page.addScriptToEvaluateOnNewDocument', { source: OBSERVATEUR });
await cdp('Emulation.setFocusEmulationEnabled', { enabled: true });
await cdp('Page.bringToFront');
await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
await dormir(4500);
await cdp('Page.bringToFront');
await evaluer('document.getElementById("glcanvas").focus()');
const audio = () => evaluer('window.__qaAudio()');
// État du message d'erreur de la page.
const etatErreur = () => evaluer(`(() => { const e = document.getElementById('erreur');
  return { affiche: getComputedStyle(e).display !== 'none',
    titre: (document.getElementById('erreur-titre') || e.querySelector('h1')).textContent.trim(),
    texte: document.getElementById('erreur-texte').textContent.trim().replace(/\\s+/g, ' '),
    bouton: (document.getElementById('recharger') || {}).offsetWidth > 0,
    focus: document.activeElement && document.activeElement.id,
    chargement: getComputedStyle(document.getElementById('chargement')).display !== 'none' }; })()`);
// Son activé au titre, JOUER, carte passée : partie en cours au niveau 1, Pépin a un peu couru.
async function lancerPartie() {
  await taper('KeyM', 'm'); await dormir(500);
  await taper('Enter', 'Enter'); await dormir(3000);
  await taper('Enter', 'Enter'); await dormir(2500);
  await touche('ArrowRight', 'ArrowRight', true); await dormir(500); await touche('ArrowRight', 'ArrowRight', false);
  await dormir(500);
}
// Pépin court 0,6 s à droite : l'image doit changer si le jeu tourne encore.
async function courir() {
  const a = await capture('/dev/null');
  await touche('ArrowRight', 'ArrowRight', true); await dormir(600); await touche('ArrowRight', 'ArrowRight', false);
  await dormir(300);
  return ecart(a, await capture('/dev/null'));
}

if (scenario === 'wasm-bloque') {
  mesure('wasm_bloque_4s', await etatErreur());
  await dormir(23000); // 4,5 + 23 s : le message est prévu à 25 s
  await capture(`${prefixe}-1-apres-27s.png`);
  mesure('wasm_bloque_27s', await etatErreur());
} else if (scenario === 'focus-jeu') {
  await taper('KeyM', 'm'); await dormir(500);                 // son activé au titre
  mesure('titre_son_active', await audio());
  await taper('Enter', 'Enter'); await dormir(3000);           // JOUER -> carte du niveau 1
  await taper('Enter', 'Enter'); await dormir(2500);           // carte -> jeu
  await touche('ArrowRight', 'ArrowRight', true); await dormir(700); await touche('ArrowRight', 'ArrowRight', false);
  await dormir(800);
  const avant = await capture(`${prefixe}-1-jeu-avant-masquage.png`);
  mesure('jeu_avant_masquage', await audio());
  // (a) Onglet masqué 6 s (requestAnimationFrame retenu, comme un vrai onglet masqué).
  await evaluer('window.__qaMasquer()'); await dormir(300);
  mesure('onglet_masque', await audio());
  await dormir(5700);
  await evaluer('window.__qaMontrer()'); await dormir(800);
  const retour = await capture(`${prefixe}-2-retour-sur-l-onglet.png`);
  mesure('retour_sur_l_onglet', { ...(await audio()), ecart_avec_avant: ecart(avant, retour) });
  await dormir(3000);
  const retour3s = await capture(`${prefixe}-3-pause-3s-plus-tard.png`);
  mesure('pause_3s_plus_tard', { ...(await audio()), ecart_avec_retour: ecart(retour, retour3s) });
  // (c) Reprise par le joueur (Échap) : retour au jeu, à la même place, boucles relancées.
  await taper('Escape', 'Escape'); await dormir(150);
  const reprise = await capture(`${prefixe}-4-reprise-echap.png`);
  await dormir(400);
  mesure('reprise_echap', { ...(await audio()), ecart_avec_avant_masquage: ecart(avant, reprise) });
  // Fenêtre visible qui perd le focus (autre application au premier plan).
  await evaluer('window.__qaFlou()'); await dormir(600);
  const flou = await capture(`${prefixe}-5-fenetre-sans-focus.png`);
  mesure('fenetre_sans_focus', { ...(await audio()), ecart_avec_reprise: ecart(reprise, flou) });
  await evaluer('window.__qaRetourFocus()'); await dormir(800);
  const focusRevenu = await capture(`${prefixe}-6-focus-revenu.png`);
  mesure('focus_revenu', { ...(await audio()), ecart_avec_fenetre_sans_focus: ecart(flou, focusRevenu) });
  await taper('Escape', 'Escape'); await dormir(600);
  await capture(`${prefixe}-7-reprise-finale.png`);
  mesure('reprise_finale', await audio());
} else if (scenario === 'focus-titre') {
  await taper('KeyM', 'm'); await dormir(500);
  const titre = await capture(`${prefixe}-1-titre.png`);
  mesure('titre', await audio());
  await evaluer('window.__qaMasquer()'); await dormir(300);
  mesure('titre_onglet_masque', await audio());
  await dormir(2700);
  await evaluer('window.__qaMontrer()'); await dormir(800);
  const retour = await capture(`${prefixe}-2-titre-retour.png`);
  mesure('titre_retour', { ...(await audio()), ecart_avec_titre: ecart(titre, retour) });
  await evaluer('window.__qaFlou()'); await dormir(600);
  const flou = await capture(`${prefixe}-3-titre-sans-focus.png`);
  mesure('titre_fenetre_sans_focus', { ...(await audio()), ecart_avec_titre: ecart(titre, flou) });
  await evaluer('window.__qaRetourFocus()'); await dormir(800);
  mesure('titre_focus_revenu', await audio());
  // Le menu répond toujours : Entrée lance bien la partie (pas de pause cachée).
  await taper('Enter', 'Enter'); await dormir(3000);
  const carte = await capture(`${prefixe}-4-entree-carte.png`);
  mesure('entree_apres_retour', { ecart_avec_titre: ecart(titre, carte) });
} else if (scenario === 'brave') {
  await lancerPartie();
  await capture(`${prefixe}-1-jeu.png`);
  // 1. Script injecté (en ligne, donc « fichier » = adresse de la page) qui lit window.ethereum.
  await evaluer(`(() => { const s = document.createElement('script');
    s.textContent = 'window.ethereum.selectedAddress'; document.head.appendChild(s); })()`);
  await dormir(300);
  mesure('brave_ethereum', { ...(await etatErreur()), jeu_continue: await courir() });
  // 2. Erreur d'un script d'une autre origine, masquée par le navigateur : « Script error. ».
  await evaluer(`window.dispatchEvent(new ErrorEvent('error', { message: 'Script error.', filename: '', lineno: 0 }))`);
  await dormir(300);
  mesure('brave_script_error', { ...(await etatErreur()), jeu_continue: await courir() });
  // 3. Promesse rejetée par un script injecté.
  await evaluer(`(() => { const s = document.createElement('script');
    s.textContent = 'Promise.reject(new TypeError("Cannot read properties of undefined (reading \\'selectedAddress\\')"))';
    document.head.appendChild(s); })()`);
  await dormir(300);
  mesure('brave_promesse', { ...(await etatErreur()), jeu_continue: await courir() });
  // 4. Erreur quelconque d'un script injecté plus tard (minuterie), sans rapport avec le jeu.
  await evaluer(`(() => { const s = document.createElement('script');
    s.textContent = 'setTimeout(function () { web3.currentProvider.enable(); }, 10)'; document.head.appendChild(s); })()`);
  await dormir(400);
  mesure('brave_minuterie', { ...(await etatErreur()), jeu_continue: await courir() });
  await capture(`${prefixe}-2-apres-erreurs-injectees.png`);
  mesure('brave_audio', await audio());
} else if (scenario === 'plantage') {
  await lancerPartie();
  await capture(`${prefixe}-1-jeu.png`);
  mesure('avant_plantage', { ...(await etatErreur()), ...(await audio()) });
  await taper('F9', 'F9'); await dormir(800);
  await capture(`${prefixe}-2-plantage-f9.png`);
  mesure('apres_f9', { ...(await etatErreur()), ...(await audio()) });
  await dormir(2000);
  mesure('apres_f9_2s', { ...(await etatErreur()), ...(await audio()) });
  // Bouton « Recharger » (clic de souris à son centre) : la page revient à l'écran titre.
  const r = await evaluer(`(() => { const e = document.getElementById('recharger');
    if (!e || !e.offsetWidth) return null; const b = e.getBoundingClientRect();
    return [b.x + b.width / 2, b.y + b.height / 2]; })()`);
  if (r) {
    await cdp('Input.dispatchMouseEvent', { type: 'mousePressed', x: r[0], y: r[1], button: 'left', clickCount: 1 });
    await cdp('Input.dispatchMouseEvent', { type: 'mouseReleased', x: r[0], y: r[1], button: 'left', clickCount: 1 });
    await dormir(4500);
    await capture(`${prefixe}-3-apres-recharger.png`);
    mesure('apres_recharger', { ...(await etatErreur()), jeu_charge: await evaluer('typeof wasm_exports !== "undefined" && !!wasm_exports') });
  } else {
    mesure('apres_recharger', { bouton: 'absent' });
  }
} else if (scenario === 'webgl') {
  await lancerPartie();
  await capture(`${prefixe}-1-jeu.png`);
  const perdu = await evaluer(`(() => { const c = document.getElementById('glcanvas');
    const gl = c.getContext('webgl') || c.getContext('webgl2'); const ext = gl && gl.getExtension('WEBGL_lose_context');
    if (!ext) return 'extension absente'; ext.loseContext(); return 'contexte perdu'; })()`);
  await dormir(800);
  await capture(`${prefixe}-2-contexte-perdu.png`);
  mesure('webgl', { action: perdu, ...(await etatErreur()), ...(await audio()) });
}
const erreurs = consoleJeu.filter((l) => /EXCEPTION|error|panicked/i.test(l));
mesure('console', { lignes: consoleJeu.length, erreurs });
ws.close(); chrome.kill();
await dormir(500);
try { fs.rmSync(profil, { recursive: true, force: true }); } catch {}
process.exit(0);
