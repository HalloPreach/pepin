// Pilote CDP de la carte 8.7 (course contre le robot) : part d'une sauvegarde vide, ouvre la
// carte des niveaux (titre, CARTE ; touche U du build de QA : niveaux 1 à 8 ouverts), choisit
// le bouton COURSE du niveau NIVEAU, puis court contre
// chaque force (DÉBUTANT, HABITUÉ, EXPERT) deux fois : d'abord en laissant gagner le fantôme
// (Pépin joue le parcours du robot de QA mais attend, posé, au pas COUPE, avant la dernière
// ligne droite), puis en le battant (REJOUER, le parcours du robot joué sans attendre). Captures : la carte (bouton COURSE), le choix de la force, le
// fantôme tramé et son étiquette, la flèche du fantôme hors champ, chaque bilan ; relève
// localStorage (clé « pepin.sauvegarde ») et compte les erreurs de la console.
// Usage : node capture-course.mjs PORT_HTTP DOSSIER_SORTIE NIVEAU NOM SCRIPT_ROBOT COUPE PARTIES_RS DELAI
//   NIVEAU : rang du niveau dans la table (1 à 8) ; NOM : son nom (celui de parties.rs) ;
//   SCRIPT_ROBOT : le parcours du robot de QA sur ce niveau, un chiffre hexadécimal par pas
//   (2 gauche, 4 droite, 8 saut), depuis le début du niveau (`bot_qa::trouver_parcours`) ;
//   COUPE : pas du parcours où Pépin peut attendre sans danger (posé, loin des ennemis :
//   attendre au départ décalerait les ennemis, Pépin mourrait) ;
//   PARTIES_RS : game/src/course/parties.rs (temps des fantômes du niveau) ;
//   DELAI : images entre l'Entrée de la force et la première image de jeu (volets +
//   présentation) : 229 à la carte 8.7, le même que pour JOUER (215 à la carte 7.9). Juste,
//   le temps de Pépin est exactement la longueur du parcours du robot (preuve de l'horloge).
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ici = path.dirname(fileURLToPath(import.meta.url));
const [httpPort, sortie, niveau, nomNiveau, fichierRobot, coupe, fichierParties, delai] = process.argv.slice(2);
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
const sauvegarde = () => evalue('localStorage.getItem("pepin.sauvegarde")');
const ouvrir = async () => {
  await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
  await dormir(4000);
  await cdp('Page.bringToFront');
  await evalue('window.focus(); document.querySelector("canvas").focus(); 1');
  await evalue(fs.readFileSync(path.join(ici, 'pilote-qa.js'), 'utf8'));
  await evalue('window.__P.parRaf = 16; window.__P.file.push({ n: 60, m: 0 }); 1');
  await dormir(1500);
  return evalue('typeof wasm_exports !== "undefined" && !!wasm_exports && getComputedStyle(document.getElementById("erreur")).display === "none"');
};
// Attend que le pilote ait joué tout ce qu'on lui a demandé (horloge gelée ensuite).
const vide = async () => {
  for (let i = 0; i < 6000; i++) {
    if (await evalue('(() => { const P = window.__P; return P.pos >= P.cible && !P.file.length; })()')) break;
    await dormir(50);
  }
  await dormir(200);
};
const touche = async (code, n = 20) => {
  await evalue(`window.__P.file.push({ n: ${n}, m: 0, taper: [${JSON.stringify(code)}] }); 1`);
  await vide();
};
// Joue `n` images de plus du script en cours.
const jouer = async (n) => {
  await evalue(`window.__P.cible = Math.min(window.__P.script.length, window.__P.pos + ${n}); 1`);
  await vide();
};
const ligne = (texte, cle) => (texte || '').split('\n').filter((l) => l.startsWith(cle)).join(' | ') || '(absente)';

// Les temps (pas) des fantômes du niveau, par force.
const sourceParties = fs.readFileSync(fichierParties, 'utf8');
const temps = [...sourceParties.matchAll(/nom: "([^"]+)",\s*force: (\d),\s*pas: (\d+),\s*morts: (\d+)/g)]
  .map(([, nom, force, pas, morts]) => ({ nom, force: Number(force), pas: Number(pas), morts: Number(morts) }));
const fantomes = temps.filter((t) => t.nom === nomNiveau);
const robot = fs.readFileSync(fichierRobot, 'utf8').trim();
console.log(`niveau ${niveau} :`, nomNiveau, '| fantômes :', JSON.stringify(fantomes), '| robot de QA :', robot.length, 'pas');

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Log.enable');
await cdp('Emulation.setDeviceMetricsOverride', { width: 960, height: 540, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });

// 1. Sauvegarde vide ; titre, CARTE : la carte des niveaux ; U (build de QA) ouvre les niveaux 1
// à 8 ; l'étape NIVEAU (Bas jusqu'au niveau 1, puis Haut), puis le bouton COURSE (Droite).
await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
await dormir(1500);
await evalue('localStorage.clear(); 1');
console.log('1. démarré :', await ouvrir());
await touche('ArrowDown', 10);
await touche('Enter', 40);
await touche('KeyU', 20);
for (let k = 0; k < 12; k++) await touche('ArrowDown', 4);
for (let k = 1; k < Number(niveau); k++) await touche('ArrowUp', 4);
await touche('ArrowRight', 20);
await capture('87c-carte-bouton-course');

// Une course contre la force de rang `force` (0 à 2) depuis le choix de la force (`depuis` :
// « choix ») ou le bilan d'avant (« rejouer » : Entrée sur REJOUER). `attente` : images
// immobiles au pas COUPE du parcours du robot (le fantôme gagne si elle dépasse l'avance du
// robot). `captures.pendant` : des captures [nom, image du niveau], dans l'ordre (pendant
// l'attente, le fantôme est d'abord hors champ derrière, puis il double Pépin).
const courir = async (force, depuis, attente, nom, captures = {}) => {
  if (depuis === 'choix') {
    for (let k = 0; k < force; k++) await touche('ArrowDown', 6);
    if (captures.choix) await capture(captures.choix);
  }
  await touche('Enter', 1);
  await evalue(`window.__P.file.push({ n: ${Number(delai) - 1}, m: 0 }); 1`);
  await vide();
  // Le niveau commence : le parcours du robot, avec `attente` images immobiles au pas COUPE.
  const c = Number(coupe);
  const script = robot.slice(0, c) + '0'.repeat(attente) + robot.slice(c);
  await evalue(`window.__P.script = ${JSON.stringify(script)}; window.__P.pos = 0; window.__P.cible = 0; 1`);
  for (const [nomCapture, image] of captures.pendant || []) {
    await jouer(image - await evalue('window.__P.pos'));
    await capture(nomCapture);
  }
  await jouer(script.length);
  // Le bilan s'ouvre au drapeau : on laisse passer une seconde.
  await evalue('window.__P.file.push({ n: 60, m: 0 }); 1');
  await vide();
  await capture(nom);
  console.log(`   ${nom} : pilote à l'image ${await evalue('window.__P.n')}, erreurs du pilote ${JSON.stringify(await evalue('window.__P.erreurs'))}`);
  console.log('   ' + ligne(await sauvegarde(), 'course.'));
};

// 2. Chaque force : le fantôme gagne (Pépin attend l'écart + 1 s), puis Pépin le bat (REJOUER).
for (const f of fantomes) {
  const avance = f.pas - robot.length; // avance du robot de QA sur ce fantôme (pas)
  const attente = Math.max(0, avance) + 60;
  console.log(`2. force ${f.force} : fantôme ${f.pas} pas (${f.morts} morts), robot de QA ${robot.length}, attente ${attente}`);
  if (f.force > 0) {
    // Retour au choix de la force : CARTE (2e ligne du bilan, la carte s'ouvre sur ce
    // niveau), puis COURSE.
    await touche('ArrowDown', 6);
    await touche('Enter', 1);
    await evalue('window.__P.file.push({ n: 120, m: 0 }); 1');
    await vide();
    await touche('ArrowRight', 10);
    await touche('Enter', 20);
  } else {
    await touche('Enter', 20); // bouton COURSE de la carte
  }
  await courir(f.force, 'choix', attente, `87c-perdu-force-${f.force}`, f.force === 0
    ? {
      choix: '87c-choix-force',
      pendant: [
        ['87c-fantome-hors-champ', Number(coupe) + Math.floor(attente / 2)],
        ['87c-fantome-double', Number(coupe) + attente - 38],
      ],
    }
    : { choix: `87c-choix-force-${f.force}` });
  await courir(f.force, 'rejouer', 0, `87c-gagne-force-${f.force}`);
}

// 3. Le choix de la force montre les meilleurs résultats (EXPERT battu : plus de prime à gagner).
await touche('ArrowDown', 6);
await touche('Enter', 1);
await evalue('window.__P.file.push({ n: 120, m: 0 }); 1');
await vide();
await touche('ArrowRight', 10);
await touche('Enter', 20);
await touche('ArrowDown', 6);
await touche('ArrowDown', 6);
await capture('87c-choix-force-expert-battu');
const s = await sauvegarde();
console.log('3. sauvegarde :', ligne(s, 'course.'), '|', ligne(s, 'gouttes_en_poche'), '|', ligne(s, 'bonus_credites'));

// 4. Page rechargée : les meilleurs résultats sont conservés.
console.log('4. démarré :', await ouvrir(), '|', ligne(await sauvegarde(), 'course.'));

console.log('erreurs console :', erreurs.length, JSON.stringify(erreurs));
ws.close(); chrome.kill('SIGKILL');
await dormir(300);
fs.rmSync(profil, { recursive: true, force: true });
process.exit(0);
