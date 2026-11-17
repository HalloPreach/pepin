// Pilote CDP de la carte 8.6 (lettres cachées P, É, P, I, N et fin secrète), sur le build de
// QA servi en local, avec l'horloge virtuelle de docs/pilote-qa.js :
// 1. sauvegarde vide : pour chacun des cinq niveaux à lettre, un parcours du robot qui la prend
//    (target/qa/script-86l-N.txt, ou 86ls au clocher ; ses touches de raccourci, sa trace)
//    rejoué jusqu'à la
//    lettre : une capture la lettre en place (zone révélée, pas encore prise), une juste
//    après. À la fin du niveau, la sauvegarde (localStorage) est relevée : `lettre:1` sous
//    la clé du niveau. Puis la page est RECHARGÉE : la sauvegarde est relue (la lettre reste).
// 2. la carte de présentation du niveau suivant après rechargement : le petit mot PÉPIN,
//    lettre(s) trouvée(s) en or, celle du niveau soulignée (HUD).
// 3. sauvegarde à 4 lettres : la descente (script-86lf) jusqu'au drapeau : fin habituelle,
//    « IL MANQUE 1 LETTRE » ;
// 4. sauvegarde à 5 lettres : la même descente : l'image de la fin secrète, puis l'écran
//    « FIN SECRÈTE » ; rechargée, la sauvegarde garde ses 5 lettres.
// Compte les erreurs de la console.
// Usage : node capture-lettres.mjs PORT_HTTP DOSSIER_SORTIE DOSSIER_QA
//   DOSSIER_QA : où sont les scripts du robot (target/qa, test ignoré
//   `bot_qa::scripts::ecrire_scripts_lettres_86l`).
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ici = path.dirname(fileURLToPath(import.meta.url));
const [httpPort, sortie, qa] = process.argv.slice(2);
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
const vide = async () => {
  for (let i = 0; i < 400 && await evalue('window.__P.file.length') > 0; i++) await dormir(50);
  await dormir(200);
};
const sauvegarde = async () => await evalue('localStorage.getItem("pepin.sauvegarde")');
// `blanc` : 60 images sans rien avant de rendre la main (menus pilotés à la main). Pour
// rejouer un script du robot, aucune : son image 0 suit le chargement, comme dans
// `capture-visibilite.mjs`.
const ouvrir = async (blanc = true) => {
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(4000);
  await cdp('Page.bringToFront');
  await evalue('window.focus(); document.querySelector("canvas").focus(); 1');
  await evalue(fs.readFileSync(path.join(ici, 'pilote-qa.js'), 'utf8'));
  if (blanc) {
    await evalue('window.__P.file.push({ n: 60, m: 0 }); 1');
    await vide();
  }
  return evalue('typeof wasm_exports !== "undefined" && !!wasm_exports && getComputedStyle(document.getElementById("erreur")).display === "none"');
};
// Pose une sauvegarde (texte v3, ou null : vide), puis (re)charge la page, prête pour un
// script du robot.
const poser = async (texte) => {
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(1500);
  await evalue(texte === null ? 'localStorage.clear(); 1'
    : `localStorage.clear(); localStorage.setItem("pepin.sauvegarde", ${JSON.stringify(texte)}); 1`);
  return ouvrir();
};
// Le script du robot `pref-n` (touches de raccourci comprises) et sa trace.
const charger = (pref, n) => {
  const script = fs.readFileSync(path.join(qa, `script-${pref}-${n}.txt`), 'utf8').trim();
  const touches = {};
  for (const l of fs.readFileSync(path.join(qa, `touches-${pref}-${n}.txt`), 'utf8').split('\n')) {
    const [image, code] = l.trim().split(':');
    if (code) touches[Number(image)] = code;
  }
  const lignes = fs.readFileSync(path.join(qa, `trace-${pref}-${n}.csv`), 'utf8').trim().split('\n');
  const tete = lignes[0].split(',');
  const trace = lignes.slice(1).map((l) => {
    const v = l.split(',');
    return Object.fromEntries(tete.map((c, i) => [c, v[i]]));
  });
  return { script, touches, trace };
};
// Rejoue le script chargé jusqu'à l'image `image` (exclue), puis attend que l'image soit
// redessinée (horloge figée, comme `capture-visibilite.mjs`).
const jusqua = async (image) => {
  await evalue(`(() => { const P = window.__P; P.parRaf = 8; P.cible = ${image}; P.gel = 0; return 1; })()`);
  for (let i = 0; i < 12000; i++) {
    const ok = await evalue('(() => { const P = window.__P; return P.pos >= Math.min(P.cible, P.script.length) && P.file.length === 0 && P.gel >= 3; })()');
    if (ok) break;
    await dormir(50);
  }
};
// Démarre le script chargé : JOUER et l'image 1 de l'histoire sont passées À LA MAIN (le
// robot appuie sur l'image dès la première image où elle l'accepte ; dans le navigateur,
// l'image l'accepte parfois une image plus tard et l'appui serait perdu), puis le script
// reprend à la première image de jeu, `delai` images après l'appui qui passe l'histoire
// (`delai` lu dans la trace, comme pour `capture-integration.mjs`).
const demarrer = async ({ script, touches, trace }) => {
  const appuis = [...script].map((c, i) => [i, parseInt(c, 32) & 1]).filter(([, v]) => v).map(([i]) => i);
  const histoire = appuis[1];
  const debut = trace.findIndex((r) => r.ecran === 'jeu');
  await evalue('window.__P.file.push({ n: 1, m: 0, taper: ["Enter"] }); 1');
  await evalue('window.__P.file.push({ n: 99, m: 0 }); 1');
  await evalue('window.__P.file.push({ n: 1, m: 0, taper: ["Enter"] }); 1');
  await evalue(`window.__P.file.push({ n: ${debut - histoire - 1}, m: 0 }); 1`);
  await vide();
  await evalue(`(() => { const P = window.__P; P.script = ${JSON.stringify(script)}; P.pos = ${debut}; P.cible = ${debut}; P.touches = ${JSON.stringify(touches)}; return 1; })()`);
};

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
await cdp('Page.bringToFront');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });

// 1. Chaque lettre, une partie à chaque fois sur une sauvegarde VIDE (numéros des niveaux :
// ceux des scripts du robot, `script-86l-N` ; au clocher, le parcours SECRET `script-86ls-5`,
// qui prend la lettre dans la salle de la sortie secrète et finit là : le rejeu de la partie
// complète du clocher ne finit pas le niveau dans le navigateur, voir QA.md).
const NIVEAUX = [[1, 'toits', '86l'], [3, 'marche', '86l'], [4, 'orage', '86l'], [5, 'clocher', '86ls'], [8, 'mer', '86l']];
// Début (image) de la DERNIÈRE carte de présentation d'une trace.
const derniereCarte = (trace) => trace.reduce(
  (acc, r, i) => (r.ecran === 'carte' && (i === 0 || trace[i - 1].ecran !== 'carte') ? i : acc), -1);
// Variable d'environnement SEULEMENT=cle : ce niveau seulement (étape 1), avec une capture
// tous les DEBUG images de jeu après la lettre (`86l-cle-dbg-IMAGE`), pour un rejeu à
// regarder de près ; les étapes 2 à 4 sont sautées.
const seulement = process.env.SEULEMENT;
for (const [n, cle, pref] of NIVEAUX.filter(([, c]) => !seulement || c === seulement)) {
  const s = charger(pref, n);
  // Ligne i de la trace = image i du script ; la lettre prise, la zone révélée.
  const prise = s.trace.findIndex((r) => r.lettre === '1');
  const revele = s.trace.findIndex((r) => Number(r.secrets) > 0);
  const avant = Math.max(revele + 1, prise - 20);
  console.log(`1. ${cle} (niveau ${n}) : zone révélée image ${revele}, lettre prise image ${prise}, capture ${avant}`);
  console.log('   démarré :', await poser(null));
  await demarrer(s);
  // Au marché : sa carte de présentation (lettre du niveau pas encore trouvée : en corail).
  if (cle === 'marche' && !process.env.SANS_CAPTURE) {
    await jusqua(derniereCarte(s.trace) + 60);
    await capture('86l-hud-carte-marche-a-trouver');
  }
  if (!process.env.SANS_CAPTURE) {
    await jusqua(avant);
    await capture(`86l-${cle}-1-lettre`);
    await jusqua(prise + 8);
    await capture(`86l-${cle}-2-prise`);
  }
  if (seulement && process.env.DEBUG) {
    for (let i = prise + 8 + Number(process.env.DEBUG); i < s.script.length; i += Number(process.env.DEBUG)) {
      await jusqua(i);
      await capture(`86l-${cle}-dbg-${i}`);
    }
  }
  await jusqua(s.script.length);
  await evalue('window.__P.file.push({ n: 120, m: 0 }); 1');
  await vide();
  const t = await sauvegarde();
  console.log(`   sauvegarde après le niveau : ${(t || '').split('\n').filter((l) => l.startsWith(`niveau.${cle}=`)).join(' | ')}`);
}
if (seulement) {
  ws.close(); chrome.kill('SIGKILL');
  await dormir(300);
  fs.rmSync(profil, { recursive: true, force: true });
  process.exit(0);
}

// 2. Rechargement : la sauvegarde (celle du dernier niveau joué, la mer) est relue ; la carte
// de présentation de la mer montre N en or, souligné.
console.log("2. rechargé :", await ouvrir());
console.log('   sauvegarde relue :', JSON.stringify((await sauvegarde() || '').split('\n').filter((l) => l.includes('lettre'))));
const mer = charger('86l', 8);
await demarrer(mer);
await jusqua(derniereCarte(mer.trace) + 60);
await capture('86l-hud-carte-mer-apres-rechargement');

// 3 et 4. Fin de partie avec 4 lettres, puis avec les 5.
const ligne = (c) => `niveau.${c}=atteint:1 gouttes:000 defis:000 medaille:0 lettre:1`;
const v3 = (cles) => ['version=3', 'parties_finies=1', ...cles.map(ligne), ''].join('\n');
const fin = charger('86lf', 12);
const finJeu = fin.script.length;
for (const [nb, cles] of [[4, ['toits', 'marche', 'orage', 'clocher']], [5, ['toits', 'marche', 'orage', 'clocher', 'mer']]]) {
  console.log(`${nb === 4 ? 3 : 4}. ${nb} lettres, démarré :`, await poser(v3(cles)));
  // La carte de présentation de la descente : le mot PÉPIN avec ses lettres.
  await demarrer(fin);
  const carteDescente = derniereCarte(fin.trace);
  if (nb === 4 && carteDescente > 0) {
    await jusqua(carteDescente + 60);
    await capture('86l-hud-carte-4-lettres');
  }
  await jusqua(finJeu);
  if (nb === 4) {
    await evalue('window.__P.file.push({ n: 200, m: 0 }); 1');
    await vide();
    await capture('86l-fin-4-lettres');
  } else {
    await evalue('window.__P.file.push({ n: 260, m: 0 }); 1');
    await vide();
    await capture('86l-fin-secrete-image');
    await evalue('window.__P.file.push({ n: 1, m: 0, taper: ["Enter"] }); 1');
    await evalue('window.__P.file.push({ n: 160, m: 0 }); 1');
    await vide();
    await capture('86l-fin-secrete');
    console.log('   rechargé :', await ouvrir());
    const relue = (await sauvegarde() || '').split('\n').filter((l) => l.includes('lettre:1'));
    console.log(`   après rechargement : ${relue.length} lignes avec lettre:1`);
  }
}
console.log('erreurs du pilote :', JSON.stringify(await evalue('window.__P && window.__P.erreurs')));
console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
