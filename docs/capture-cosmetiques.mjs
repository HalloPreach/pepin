// Pilote CDP de la carte 7.6 (cosmétiques visuels : traînées, motifs de parapluie, palettes,
// musiques dans la boutique). Build de QA servi en local, horloge virtuelle de docs/pilote-qa.js.
//  A. Boutique : sauvegarde préparée (100 gouttes, POIS acheté et équipé), les cinq onglets,
//     achat + équipement d'ÉTINCELLES puis de SÉPIA (la boutique passe en sépia), retour à
//     ORIGINALE, onglet MUSIQUES ; page rechargée : la traînée équipée court derrière Pépin.
//  B. Jeu, touches du build de QA (T traînée, Y motif, O palette, K skin) : chaque traînée en
//     course et en plané ; chaque motif fermé et ouvert, sur Pépin et sur Pépinette ; chaque
//     palette sur trois niveaux contrastés (toits 1, orage 4, mer 8).
// Après chaque capture : pixels hors de la palette ACTIVE et blocs flous, lus dans le tampon WebGL.
// Usage : CHROME=... node capture-cosmetiques.mjs PORT_HTTP DOSSIER_SORTIE DOSSIER_GRILLES
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ici = path.dirname(fileURLToPath(import.meta.url));
const [httpPort, sortie, grilles] = process.argv.slice(2);
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

// Les palettes (game/src/cosmetiques.rs), dans l'ordre de la touche O.
const PALETTES = {
  originale: ['10141F', '1C2336', '2D3A54', '46587A', '7F93B5', 'A9B8D0', 'C3D1E6', 'F3F1EA',
    'FF5A4F', 'FF9A7A', 'FFB347', 'FFD84A', 'FFF1A8', '62D0E8', '2F8F9D', '5B4A8A'],
  sepia: ['19130F', '2D2118', '4A3624', '6D5334', 'AF8C60', 'C9B496', 'DDCEB7', 'F7F0E7',
    'D57A6C', 'E1A794', 'E7BA8B', 'F0D9A5', 'F8F0D4', 'C5C2AF', '928161', '704A52'],
  nuit_froide: ['0E1422', '1B223D', '2A395E', '425785', '7B93BC', 'A8B8D2', 'C4D1E4', 'EBF2F7',
    'BF77D6', 'CEA6E0', 'BCBFEB', 'B8E2F0', 'E5F2F9', '69D1D9', '308EA3', '634495'],
  ecran_vert: ['0C1708', '152810', '29401C', '42602A', '749E48', 'A0C272', 'C3D897', 'EEF4D9',
    '48A648', '81C571', 'ACCE6A', 'D5E389', 'EFF3CF', '92CFA7', '41935C', '355E31'],
};
// Lecture exacte de l'image affichée (comme P.lire de pilote-qa.js), contre une palette donnée :
// grille logique (indice de la palette, 'x' hors palette), blocs KxK non uniformes.
const LIRE = (pal) => `(() => {
  wasm_exports.frame();
  const canvas = document.getElementById('glcanvas');
  const w = canvas.width, h = canvas.height, buf = new Uint8Array(w * h * 4);
  gl.readPixels(0, 0, w, h, gl.RGBA, gl.UNSIGNED_BYTE, buf);
  const K = Math.max(1, Math.min(Math.floor(w / 320), Math.floor(h / 180)));
  const x0 = Math.floor((w - 320 * K) / 2), y0 = Math.floor((h - 180 * K) / 2);
  const px = (x, y) => { const o = ((h - 1 - y) * w + x) * 4; return (buf[o] << 16) | (buf[o + 1] << 8) | buf[o + 2]; };
  const index = new Map(${JSON.stringify(pal)}.map((c, i) => [parseInt(c, 16), i.toString(16).toUpperCase()]));
  let grille = '', flou = 0; const hors = {};
  for (let ly = 0; ly < 180; ly++) {
    for (let lx = 0; lx < 320; lx++) {
      const ref = px(x0 + lx * K, y0 + ly * K);
      let uni = true;
      for (let dy = 0; dy < K && uni; dy++) for (let dx = 0; dx < K; dx++) {
        if (px(x0 + lx * K + dx, y0 + ly * K + dy) !== ref) { uni = false; break; }
      }
      if (!uni) flou++;
      const c = index.get(ref);
      if (c === undefined) { grille += 'x'; const k = ref.toString(16).padStart(6, '0'); hors[k] = (hors[k] || 0) + 1; }
      else grille += c;
    }
    grille += '\\n';
  }
  return JSON.stringify({ K, flou, hors, grille });
})()`;
const bilan = [];
const capture = async (nom, palette = 'originale') => {
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  fs.writeFileSync(path.join(sortie, `${nom}.png`), Buffer.from(r.result.data, 'base64'));
  const l = JSON.parse(await evalue(LIRE(PALETTES[palette])));
  fs.writeFileSync(path.join(grilles, `${nom}.txt`), l.grille);
  const nbHors = Object.values(l.hors).reduce((a, b) => a + b, 0);
  bilan.push({ nom, palette, hors: nbHors, flou: l.flou });
  console.log('capture', nom.padEnd(44), palette.padEnd(11), 'hors palette', nbHors, 'flou', l.flou,
    nbHors ? JSON.stringify(l.hors).slice(0, 120) : '');
};
const sauvegarde = async () => await evalue('localStorage.getItem("pepin.sauvegarde")');
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
// File d'images : `n` images avec les touches `m` tenues (1 Entrée, 2 gauche, 4 droite, 8 saut),
// `taper` : touches appuyées à la première image seulement.
const images = async (n, m = 0, taper = null) => {
  const e = taper ? `{ n: ${n}, m: ${m}, taper: [${JSON.stringify(taper)}] }` : `{ n: ${n}, m: ${m} }`;
  await evalue(`window.__P.file.push(${e}); 1`);
  for (let i = 0; i < 400 && await evalue('window.__P.file.length') > 0; i++) await dormir(50);
  await dormir(150);
};
const touche = (code, n = 20) => images(n, 0, code);
const ecrire = (texte) => evalue(`localStorage.setItem("pepin.sauvegarde", ${JSON.stringify(texte)}); 1`);

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
await cdp('Page.bringToFront');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });
fs.mkdirSync(grilles, { recursive: true });
const seule = process.env.PARTIE || 'AB';

if (seule.includes('A')) {
  // --- A. Boutique -------------------------------------------------------------------------
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(1500);
  await ecrire('version=3\ngouttes_en_poche=100\ngouttes_total_gagnees=100\nparties_finies=1\n'
    + 'inventaire=skin:pepin,skin:pepinette,musique:orage,musique:zone,trainee:aucune,motif:uni,palette:origine,motif:pois\n'
    + 'equipe=skin:pepin,musique:zone,trainee:aucune,motif:pois,palette:origine\n');
  console.log('A. démarré :', await ouvrir(), '|', JSON.stringify(await sauvegarde()));
  // BOUTIQUE : 3e ligne du titre (JOUER, CARTE, BOUTIQUE).
  await touche('ArrowDown', 10);
  await touche('ArrowDown', 30);
  await touche('Enter', 40);
  await capture('76-boutique-skins');
  await touche('ArrowRight', 30);
  await capture('76-boutique-trainees');
  // ÉTINCELLES (15) : confirmation, OUI, achat, puis Entrée l'équipe.
  await touche('ArrowDown');
  await touche('Enter', 30);
  await touche('ArrowRight', 20);
  await touche('Enter', 30);
  await touche('Enter', 30);
  await capture('76-boutique-etincelles-achetee-equipee');
  console.log('   après ÉTINCELLES :', JSON.stringify(await sauvegarde()));
  await touche('ArrowRight', 30);
  await capture('76-boutique-parapluies');
  await touche('ArrowRight', 30);
  await capture('76-boutique-palettes');
  // SÉPIA (20) : achat, équipement ; toute la boutique passe en sépia.
  await touche('ArrowDown');
  await touche('Enter', 30);
  await touche('ArrowRight', 20);
  await touche('Enter', 30);
  await touche('Enter', 30);
  await capture('76-boutique-sepia-equipee', 'sepia');
  console.log('   après SÉPIA :', JSON.stringify(await sauvegarde()));
  await touche('ArrowUp');
  await touche('Enter', 30);
  await capture('76-boutique-retour-originale');
  await touche('ArrowRight', 30);
  for (let i = 0; i < 4; i++) await touche('ArrowDown', 8);
  await capture('76-boutique-musiques');
  await touche('Escape', 40);
  console.log('A. rechargé :', await ouvrir(), '|', JSON.stringify(await sauvegarde()));
  // JOUER : l'image de l'histoire, la carte du niveau 1, puis le jeu ; Pépin court : la
  // traînée équipée le suit.
  await touche('Enter', 200);
  await touche('Enter', 120);
  await touche('Enter', 120);
  await images(40, 4);
  await capture('76-jeu-etincelles-de-la-boutique');
}

if (seule.includes('B')) {
  // --- B. Jeu, touches du build de QA -------------------------------------------------------
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(1500);
  await evalue('localStorage.removeItem("pepin.sauvegarde"); 1');
  console.log('B. démarré :', await ouvrir());
  await touche('Enter', 200);
  await touche('Enter', 120);
  // Retour au début du niveau 1 (raccourci), pour partir d'un état connu.
  const niveau = async (n) => { await touche(`Digit${n}`, 150); await touche('Enter', 120); };
  await niveau(1);
  // Traînées : course vers la droite, puis saut tenu vers la gauche (parapluie ouvert).
  for (const nom of ['etincelles', 'bulles', 'plumes']) {
    await touche('KeyT', 2);
    await images(40, 4);
    await capture(`76-trainee-${nom}-course`);
    await images(34, 8 | 2);
    await capture(`76-trainee-${nom}-plane`);
    await images(70, 0);
  }
  await touche('KeyT', 2); // plus de traînée
  // Motifs : fermé (Pépin immobile), ouvert (saut tenu, en descente), sur les deux skins.
  for (const skin of ['pepin', 'pepinette']) {
    for (const nom of ['pois', 'rayures', 'etoiles']) {
      await touche('KeyY', 30);
      await capture(`76-motif-${nom}-${skin}-ferme`);
      await images(34, 8);
      await capture(`76-motif-${nom}-${skin}-ouvert`);
      await images(70, 0);
    }
    await touche('KeyY', 2); // parapluie uni
    if (skin === 'pepin') await touche('KeyK', 2);
  }
  await touche('KeyK', 2); // retour à Pépin
  // Palettes, sur trois niveaux contrastés : Pépin un peu avancé dans le niveau.
  for (const [n, nomNiveau] of [[1, 'toits'], [4, 'orage'], [8, 'mer']]) {
    await niveau(n);
    await images(30, 4);
    for (const pal of ['sepia', 'nuit_froide', 'ecran_vert']) {
      await touche('KeyO', 4);
      await capture(`76-palette-${pal}-${nomNiveau}`, pal);
    }
    await touche('KeyO', 4); // retour à l'originale
    await capture(`76-palette-originale-${nomNiveau}`);
  }
}

fs.writeFileSync(path.join(grilles, 'bilan.json'), JSON.stringify({ bilan, erreurs }, null, 1));
console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
