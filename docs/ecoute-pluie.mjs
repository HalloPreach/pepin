// Écoute de contrôle de la PLUIE (carte 10.4e) : la boucle de pluie, enregistrée
// depuis le jeu dans Chromium sans écran, a-t-elle un blanc au point de boucle ?
//
// La sonde (installée avant tout script de la page) repère la source audio de la
// pluie au moment où le jeu la lance (la seule boucle de 2 s) et la branche, en plus
// de sa sortie normale, sur un enregistreur (ScriptProcessor) : on enregistre ce que
// le navigateur JOUE, boucle comprise, après son propre décodage du fichier. On active
// le son par la touche M à l'écran titre (la pluie y tombe), on enregistre 10 s, puis
// on mesure la plus longue suite d'échantillons NULS consécutifs, et surtout la plus
// longue suite QUASI MUETTE (|valeur| <= 256, 0,8 % du plein volume) : la pluie est
// un bruit continu, elle n'en a pas ; un blanc de remplissage en est une (celui de
// l'OGG Vorbis de la 1re étape, rééchantillonné par le navigateur, n'est pas fait de
// zéros exacts : 274 échantillons quasi muets, 6,2 ms, au point de boucle). Et, à
// chaque point de boucle (toutes les 2 s), le saut le plus fort d'un échantillon au
// suivant et le nombre d'échantillons quasi muets autour.
// La console du jeu est suivie : toute erreur (son illisible...) est comptée.
// Usage : node docs/ecoute-pluie.mjs PORT_HTTP [SORTIE.wav]  (variable CHROME).
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const [httpPort, sortieWav] = process.argv.slice(2);
if (!httpPort) {
  console.error('Usage : node docs/ecoute-pluie.mjs PORT_HTTP [SORTIE.wav]');
  process.exit(2);
}
const CHROME = process.env.CHROME || 'chromium';
const SECONDES = 10;
const dormir = (ms) => new Promise((r) => setTimeout(r, ms));

const SONDE = `
window.__E = { lances: [], enreg: null };
const lancer = AudioBufferSourceNode.prototype.start;
AudioBufferSourceNode.prototype.start = function (...a) {
  const b = this.buffer;
  window.__E.lances.push(b ? +b.duration.toFixed(3) : -1);
  // La pluie : la seule boucle de 2 s (les musiques font 9 s et plus, les bruitages
  // moins de 1 s ; un remplissage éventuel l'allonge de quelques ms).
  if (b && this.loop && Math.abs(b.duration - 2) < 0.05 && !window.__E.enreg) {
    const ctx = this.context;
    const proc = ctx.createScriptProcessor(4096, 1, 1);
    const muet = ctx.createGain();
    muet.gain.value = 0;
    const e = { taux: ctx.sampleRate, longueur: b.length, morceaux: [], n: 0 };
    const max = ctx.sampleRate * ${SECONDES + 1};
    proc.onaudioprocess = (ev) => {
      if (e.n >= max) return;
      const d = ev.inputBuffer.getChannelData(0);
      e.morceaux.push(Array.from(d, (v) => Math.round(v * 32767)));
      e.n += d.length;
    };
    this.connect(proc);
    proc.connect(muet);
    muet.connect(ctx.destination);
    window.__E.enreg = e;
  }
  return lancer.apply(this, a);
};`;

const dbg = 9400 + Math.floor(Math.random() * 400);
const profil = fs.mkdtempSync(path.join(process.env.TMPDIR || '/tmp', 'prof-pluie-'));
const chrome = spawn(CHROME, [
  `--remote-debugging-port=${dbg}`, `--user-data-dir=${profil}`, '--headless=new', '--no-sandbox',
  '--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist',
  '--autoplay-policy=no-user-gesture-required',
  '--window-size=960,540', '--force-device-scale-factor=1', 'about:blank',
], { stdio: 'ignore' });
let code = 0;
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
  ws.onmessage = (m) => {
    const d = JSON.parse(m.data);
    if (d.id && att.has(d.id)) { att.get(d.id)(d); att.delete(d.id); }
    if (d.method === 'Runtime.exceptionThrown') erreurs.push(d.params.exceptionDetails.text);
    if (d.method === 'Runtime.consoleAPICalled' && (d.params.type === 'error' || d.params.type === 'warning')) {
      erreurs.push(d.params.type + ' : ' + d.params.args.map((a) => a.value ?? a.description).join(' '));
    }
  };
  const cdp = (method, params = {}) => new Promise((r) => { const i = ++id; att.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
  const evalue = async (expr) => (await cdp('Runtime.evaluate', { expression: expr, returnByValue: true })).result?.result?.value;
  await cdp('Page.enable');
  await cdp('Runtime.enable');
  await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
  await cdp('Page.addScriptToEvaluateOnNewDocument', { source: SONDE });
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  // Le temps que le jeu démarre et télécharge ses sons (la pluie est parmi les premiers).
  await dormir(4000);
  await evalue('document.querySelector("canvas").focus(); 1');
  // Touche M : le son s'active, la musique et la pluie partent.
  for (const type of ['keyDown', 'keyUp']) {
    await cdp('Input.dispatchKeyEvent', { type, key: 'm', code: 'KeyM', windowsVirtualKeyCode: 77, text: type === 'keyDown' ? 'm' : undefined });
  }
  let e;
  for (let i = 0; i < 300; i++) {
    await dormir(100);
    e = await evalue(`window.__E.enreg ? window.__E.enreg.n : -1`);
    if (e >= 0 && e >= (await evalue('window.__E.enreg.taux')) * (SECONDES + 1)) break;
  }
  const brut = JSON.parse(await evalue(`JSON.stringify({ lances: window.__E.lances, taux: window.__E.enreg && window.__E.enreg.taux, longueur: window.__E.enreg && window.__E.enreg.longueur, ech: window.__E.enreg ? window.__E.enreg.morceaux.flat() : [] })`));
  ws.close();
  if (!brut.taux) throw new Error(`pluie jamais lancée (sons lancés : ${brut.lances.join(' ')})`);
  // On part du premier échantillon non nul (l'enregistreur tourne un peu avant la
  // source) et on garde 10 s.
  const debut = brut.ech.findIndex((v) => v !== 0);
  const s = brut.ech.slice(debut, debut + brut.taux * SECONDES);
  const QUASI = 256;
  const suite = (muet) => {
    let plus = 0, cour = 0, ou = -1;
    s.forEach((v, i) => {
      cour = muet(v) ? cour + 1 : 0;
      if (cour > plus) { plus = cour; ou = i - cour + 1; }
    });
    return [plus, ou];
  };
  const [plusLongue, ou] = suite((v) => v === 0);
  const [plusLongueQuasi, ouQuasi] = suite((v) => Math.abs(v) <= QUASI);
  let sautInterieur = 0;
  for (let i = 1; i < s.length; i++) sautInterieur = Math.max(sautInterieur, Math.abs(s[i] - s[i - 1]));
  // Les points de boucle : tous les `longueur` échantillons depuis le début.
  const raccords = [];
  for (let k = brut.longueur; k < s.length; k += brut.longueur) {
    const autour = s.slice(k - 64, k + 64);
    let quasi = 0, max = 0;
    for (let i = 0; i < autour.length; i++) {
      if (Math.abs(autour[i]) <= QUASI) quasi++;
      if (i) max = Math.max(max, Math.abs(autour[i] - autour[i - 1]));
    }
    raccords.push(`${(k / brut.taux).toFixed(3)} s : saut max ${max}, ${quasi} quasi muets sur 128`);
  }
  console.log(`sons lancés (durée décodée, s) : ${brut.lances.join(' ')}`);
  console.log(`pluie décodée : ${brut.longueur} échantillons à ${brut.taux} Hz (${(brut.longueur / brut.taux).toFixed(4)} s)`);
  console.log(`enregistré : ${s.length} échantillons (${(s.length / brut.taux).toFixed(2)} s)`);
  const ms = (n) => (n / brut.taux * 1000).toFixed(2);
  console.log(`plus longue suite de nuls : ${plusLongue} échantillons (${ms(plusLongue)} ms) à ${(ou / brut.taux).toFixed(3)} s`);
  console.log(`plus longue suite quasi muette (<= ${QUASI}) : ${plusLongueQuasi} échantillons (${ms(plusLongueQuasi)} ms) à ${(ouQuasi / brut.taux).toFixed(3)} s`);
  console.log(`plus fort saut d'un échantillon au suivant (tout l'enregistrement) : ${sautInterieur}`);
  console.log(`points de boucle :\n  ${raccords.join('\n  ')}`);
  console.log(`erreurs de la console : ${erreurs.length}${erreurs.length ? '\n  ' + erreurs.join('\n  ') : ''}`);
  if (sortieWav) {
    const donnees = Buffer.alloc(s.length * 2);
    s.forEach((v, i) => donnees.writeInt16LE(Math.max(-32768, Math.min(32767, v)), i * 2));
    const t = Buffer.alloc(44);
    t.write('RIFF', 0); t.writeUInt32LE(36 + donnees.length, 4); t.write('WAVEfmt ', 8);
    t.writeUInt32LE(16, 16); t.writeUInt16LE(1, 20); t.writeUInt16LE(1, 22);
    t.writeUInt32LE(brut.taux, 24); t.writeUInt32LE(brut.taux * 2, 28); t.writeUInt16LE(2, 32); t.writeUInt16LE(16, 34);
    t.write('data', 36); t.writeUInt32LE(donnees.length, 40);
    fs.writeFileSync(sortieWav, Buffer.concat([t, donnees]));
    console.log(`enregistrement : ${sortieWav}`);
  }
} catch (err) {
  console.error(err.message);
  code = 1;
} finally {
  try { chrome.kill('SIGKILL'); } catch {}
  await dormir(300);
  fs.rmSync(profil, { recursive: true, force: true });
}
process.exit(code);
