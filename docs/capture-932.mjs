// Pilote CDP de la carte 9.32 (« le niveau de la marée, fait entièrement à
// l'éditeur »), sur le build de QA servi par un serveur de fichiers statiques
// (dist-qa/). Il fait dans `editeur.html` ce qu'une personne y ferait :
//  1. CONSTRUIT « La marée » : bouton Nouveau, boutons de taille (« + à droite »,
//     « + en haut »), palette, outils Rectangle et Pinceau à la souris sur la grille
//     (les gestes sont lus dans docs/maree-gestes.json), formulaire au clavier
//     (listes : la valeur choisie, puis l'événement change) ;
//  2. Valider (le chargeur Rust du jeu), ROBOT (parapluie permis), ROBOT sans
//     planer puis « Recopier dans sans_planer », TESTER (le jeu de QA joue le
//     niveau dans l'éditeur, captures) ;
//  3. « Exporter le .ron » (téléchargé dans DOSSIER_SORTIE/maree.ron) ;
//  4. ouvre monde_2/pieuvre.ron dans la liste des niveaux du jeu, passe son numéro
//     de 7 à 8 (la marée prend la 7e place) et l'exporte (DOSSIER_SORTIE/pieuvre.ron).
// Captures : 932-editeur-*.png (l'éditeur), 932-jeu-*.png (le jeu dans l'éditeur).
// Usage : node docs/capture-932.mjs PORT_HTTP DOSSIER_SORTIE   (CHROME=chemin de chromium)
// Variable SANS_CONSTRUIRE=1 : ouvre DOSSIER_SORTIE/maree.ron (fichier local) au lieu
// de le construire, pour ne refaire que les captures du jeu.
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const [httpPort, sortie] = process.argv.slice(2);
const ici = path.dirname(fileURLToPath(import.meta.url));
const CHROME = process.env.CHROME || 'chromium';
const dbg = 9400 + Math.floor(Math.random() * 400);
const profil = fs.mkdtempSync(path.join(process.env.TMPDIR || '/tmp', 'prof-'));
const LARGEUR_FENETRE = 2400;
const chrome = spawn(CHROME, [
  `--remote-debugging-port=${dbg}`, `--user-data-dir=${profil}`, '--headless=new', '--no-sandbox',
  `--window-size=${LARGEUR_FENETRE},1000`, '--force-device-scale-factor=1', 'about:blank',
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
  if (d.method === 'Runtime.exceptionThrown') erreurs.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text);
  if (d.method === 'Page.javascriptDialogOpening') cdp('Page.handleJavaScriptDialog', { accept: true });
};
const cdp = (method, params = {}) => new Promise((r) => { const i = ++id; att.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
const evalue = async (expr) => (await cdp('Runtime.evaluate', { expression: expr, returnByValue: true, awaitPromise: true })).result?.result?.value;
const capture = async (nom) => {
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  fs.writeFileSync(path.join(sortie, `${nom}.png`), Buffer.from(r.result.data, 'base64'));
  console.log('capture', nom);
};
const centre = (sel) => evalue(`(() => { const e = document.querySelector(${JSON.stringify(sel)}); e.scrollIntoView({ block: 'center' }); const r = e.getBoundingClientRect(); return [r.x + r.width / 2, r.y + r.height / 2]; })()`);
const souris = (type, x, y, button = 'left') => cdp('Input.dispatchMouseEvent', { type, x, y, button, clickCount: 1 });
const cliquer = async (sel) => { const [x, y] = await centre(sel); await souris('mousePressed', x, y); await souris('mouseReleased', x, y); };
const attendre = async (expr, ms = 30000) => {
  for (let t = 0; t < ms; t += 250) { if (await evalue(expr)) return true; await dormir(250); }
  return false;
};
let echecs = 0;
const verifier = (nom, ok, detail = '') => { console.log(`${ok ? 'OK   ' : 'ECHEC'} ${nom} ${detail}`); if (!ok) echecs++; };
const touche = (type, key, code, vk) => cdp('Input.dispatchKeyEvent', { type, key, code, windowsVirtualKeyCode: vk });
const appuie = async (key, code, vk) => { await touche('keyDown', key, code, vk); await touche('keyUp', key, code, vk); };

await cdp('Page.enable');
await cdp('Runtime.enable');
await cdp('Emulation.setDeviceMetricsOverride', { width: LARGEUR_FENETRE, height: 1000, deviceScaleFactor: 1, mobile: false });
fs.mkdirSync(sortie, { recursive: true });
await cdp('Browser.setDownloadBehavior', { behavior: 'allow', downloadPath: path.resolve(sortie) });
const base = `http://127.0.0.1:${httpPort}`;
await cdp('Page.navigate', { url: `${base}/editeur.html` });
await dormir(800);
await evalue('localStorage.clear(); 1');
await cdp('Page.navigate', { url: `${base}/editeur.html` });
await attendre('window.editeurPepin && editeurPepin.etat.grille.length > 0');

/// Centre de la case (x, y) de la grille, à l'écran (la toile est d'abord amenée
/// à l'écran, comme on fait défiler la page à la main).
const caseEcran = async (x, y) => {
  const [l, t, z] = await evalue(`(() => { const toile = document.querySelector("#toile"); let r = toile.getBoundingClientRect(); const z = editeurPepin.etat.zoom; const yy = r.top + ${y} * z; if (yy < 0 || yy > innerHeight - 20) { window.scrollBy(0, yy - innerHeight / 2); r = toile.getBoundingClientRect(); } return [r.left, r.top, z]; })()`);
  return [l + x * z + z / 2, t + y * z + z / 2];
};
const choisirLettre = (c) => cliquer(`.palette button[data-lettre=${JSON.stringify(c)}]`);
const outil = (o) => cliquer(`#boutons-outils button[data-outil="${o}"]`);
/// Outil Rectangle, lettre `c`, de la case (x0, y0) à (x1, y1) : appui, glisser, relâcher.
async function rectangle(c, x0, y0, x1, y1) {
  await choisirLettre(c);
  await outil('rectangle');
  const [ax, ay] = await caseEcran(x0, y0);
  const [bx, by] = await caseEcran(x1, y1);
  await souris('mousePressed', ax, ay);
  await souris('mouseMoved', bx, by);
  await souris('mouseReleased', bx, by);
}
/// Pinceau, lettre `c`, un clic sur chaque case.
async function pinceau(c, cases) {
  await choisirLettre(c);
  await outil('pinceau');
  for (const [x, y] of cases) {
    const [ex, ey] = await caseEcran(x, y);
    await souris('mousePressed', ex, ey);
    await souris('mouseReleased', ex, ey);
  }
}
/// Un champ du formulaire, au clavier : sélection, texte tapé, Tab (l'événement change).
async function taper(nom, texte) {
  await evalue(`(() => { const c = document.querySelector('#formulaire [name=${nom}]'); c.scrollIntoView({ block: 'center' }); c.focus(); c.select && c.select(); return 1; })()`);
  await cdp('Input.insertText', { text: texte });
  await appuie('Tab', 'Tab', 9);
}
/// Une liste du formulaire : la valeur choisie, puis l'événement change (ce que fait
/// le navigateur quand on choisit dans la liste).
const choisirDansLaListe = (nom, valeur) => evalue(`(() => { const c = document.querySelector('#formulaire [name=${nom}]'); c.value = ${JSON.stringify(valeur)}; c.dispatchEvent(new Event('change', { bubbles: true })); return c.value; })()`);
const texteDe = (sel) => evalue(`document.querySelector(${JSON.stringify(sel)}).textContent`);

/// ROBOT (sansPlaner : la case « sans planer » cochée), limite 600 s ; rend le texte du résultat.
async function robot(sansPlaner) {
  await evalue(`(() => { const c = document.querySelector('#robot-sans-planer'); if (c.checked !== ${sansPlaner}) c.click(); const l = document.querySelector('#robot-limite'); l.value = '600'; l.dispatchEvent(new Event('change', { bubbles: true })); return 1; })()`);
  await cliquer('#robot');
  await attendre('!document.querySelector("#robot").disabled', 620000);
  return texteDe('#robot-texte');
}

if (!process.env.SANS_CONSTRUIRE) {
  // --- 1. Construire le niveau ---------------------------------------------------------
  const plan = JSON.parse(fs.readFileSync(path.join(ici, 'maree-gestes.json'), 'utf8'));
  await cliquer('#nouveau');
  await dormir(200);
  // Zoom 8 : la grille de 166 cases (1 328 px) tient dans la fenêtre.
  for (let i = 0; i < 4; i++) await cliquer('#zoom-moins');
  const [l0, h0] = await evalue('[editeurPepin.etat.grille[0].length, editeurPepin.etat.grille.length]');
  for (let i = l0; i < plan.largeur; i++) await cliquer('[data-taille="droite+"]');
  for (let i = h0; i < plan.hauteur; i++) await cliquer('[data-taille="haut+"]');
  for (let i = h0; i > plan.hauteur; i--) await cliquer('[data-taille="haut-"]');
  const taille = await evalue('[editeurPepin.etat.grille[0].length, editeurPepin.etat.grille.length]');
  verifier(`grille ${plan.largeur} x ${plan.hauteur}`, taille[0] === plan.largeur && taille[1] === plan.hauteur, JSON.stringify(taille));
  const droite = await evalue('document.querySelector("#toile").getBoundingClientRect().right');
  verifier('la grille entière est à l\'écran', droite <= LARGEUR_FENETRE, `bord droit ${Math.round(droite)} px`);
  for (const g of plan.gestes) {
    if (g[0] === 'rect') await rectangle(g[1], g[2], g[3], g[4], g[5]);
    else await pinceau(g[1], g[2]);
  }
  // La grille dessinée est-elle celle du plan ? (relue dans l'état de l'éditeur)
  const attendue = Array.from({ length: plan.hauteur }, () => Array(plan.largeur).fill('.'));
  for (const g of plan.gestes) {
    if (g[0] === 'rect') {
      for (let y = Math.min(g[3], g[5]); y <= Math.max(g[3], g[5]); y++) for (let x = Math.min(g[2], g[4]); x <= Math.max(g[2], g[4]); x++) attendue[y][x] = g[1];
    } else for (const [x, y] of g[2]) attendue[y][x] = g[1];
  }
  const grille = await evalue('editeurPepin.etat.grille.map((r) => r.join("")).join("\\n")');
  verifier('grille dessinée = plan', grille === attendue.map((r) => r.join('')).join('\n'));

  // Le formulaire.
  await taper('monde', '2');
  await taper('numero', '7');
  await taper('cle', 'maree');
  await taper('nom', 'La marée');
  verifier('thème Port', (await choisirDansLaListe('theme', 'Port')) === 'Port');
  verifier('genre Normal', (await choisirDansLaListe('genre', 'Normal')) === 'Normal');
  await taper('duree_cible_s', '70');
  await taper('intro_texte', "Dans le port, la mer monte et descend. Au fond des bassins, l'eau n'attend personne.");
  await taper('intro_astuce', 'Mer basse : on marche au fond. Mer haute : la barque monte avec elle.');
  await taper('mecanique', "La marée en vedette : traverser un bassin à pied quand la mer est basse ; attendre en barque que la mer monte pour remonter au quai ; passer sous une voûte au fond à mer basse, ou en barque à mer haute ; puis les nénuphars, les poissons et les crabes des pontons.");
  const champs = await evalue('JSON.stringify((({monde, numero, cle, nom, theme, genre, duree_cible_s}) => ({monde, numero, cle, nom, theme, genre, duree_cible_s}))(editeurPepin.etat.niveau))');
  verifier('formulaire', champs.includes('"cle":"maree"') && champs.includes('"theme":"Port"') && champs.includes('"numero":7'), champs);
  const apercu = await texteDe('#apercu-intro_texte') + ' | ' + await texteDe('#apercu-intro_astuce');
  console.log('aperçu de l\'intro :', apercu.replace(/\n/g, ' / '));
  const avert = await evalue('[...document.querySelectorAll("#avertissements li")].map((l) => (l.className === "erreur" ? "[ERREUR] " : "") + l.textContent)');
  console.log('avertissements :', JSON.stringify(avert, null, 1));
  await capture('932-editeur-dessin');

  // --- 2. Valider, ROBOT, TESTER ---------------------------------------------------------
  await evalue('document.querySelector("#verdict").textContent = ""; 1');
  await cliquer('#valider');
  await attendre('!document.querySelector("#verdict").hidden && document.querySelector("#verdict").textContent.length > 0');
  const verdict = await texteDe('#verdict');
  verifier('Valider', verdict.includes('accepté'), verdict);
  const r1 = await robot(false);
  verifier('ROBOT (parapluie permis)', /arrivée atteinte/.test(r1) && /gouttes 3\/3/.test(r1), r1);
  await capture('932-editeur-robot');
  const r2 = await robot(true);
  console.log('ROBOT sans planer :', r2);
  if (await evalue('!document.querySelector("#robot-remplir").hidden')) await cliquer('#robot-remplir');
  console.log('sans_planer =', await evalue('editeurPepin.etat.niveau.sans_planer'));

  // --- 3. Exporter ----------------------------------------------------------------------------
  const fichier = path.join(sortie, 'maree.ron');
  if (fs.existsSync(fichier)) fs.unlinkSync(fichier);
  await cliquer('#exporter');
  for (let i = 0; i < 40 && !fs.existsSync(fichier); i++) await dormir(250);
  verifier('export maree.ron', fs.existsSync(fichier), fichier);
} else {
  const texte = fs.readFileSync(path.join(sortie, 'maree.ron'), 'utf8');
  await evalue(`editeurPepin.ouvrirTexte(${JSON.stringify(texte)}, 'maree.ron'); 1`);
  for (let i = 0; i < 4; i++) await cliquer('#zoom-moins');
}

// --- TESTER, et des parties lancées d'une case (clic droit « partir d'ici ») ---------------
const T = { saut: [' ', 'Space', 32], droite: ['ArrowRight', 'ArrowRight', 39], gauche: ['ArrowLeft', 'ArrowLeft', 37], bas: ['ArrowDown', 'ArrowDown', 40] };
async function jouer(suite) {
  for (const a of suite) {
    if (typeof a === 'number') await dormir(a);
    else await touche(a[1] === 'bas' ? 'keyDown' : 'keyUp', ...T[a[0]]);
  }
}
const enJeu = 'document.querySelector("#essai") && !document.querySelector("#essai").hidden && !document.querySelector("#essai").classList.contains("cache")';
const fermer = async () => { await appuie('Escape', 'Escape', 27); await dormir(800); };
// TESTER : la carte du niveau (titre et intro), puis le départ.
await cliquer('#tester');
verifier('TESTER : le jeu se lance', await attendre(enJeu, 20000));
await dormir(1500);
await capture('932-jeu-carte');
await appuie('Enter', 'Enter', 13);
await dormir(1500);
await capture('932-jeu-depart');
await fermer();
// Des moments du niveau, depuis une case : [nom, [x, y], actions].
const MOMENTS = [
  ['932-jeu-bassin-sec', [8, 3], [['droite', 'bas'], 650, ['droite', 'haut'], 900]],
  ['932-jeu-ecluse', [33, 3], [['droite', 'bas'], 500, ['saut', 'bas'], 450, ['saut', 'haut'], 80, ['saut', 'bas'], 1200]],
  ['932-jeu-nenuphars', [75, 3], [['droite', 'bas'], 400, ['saut', 'bas'], 300, ['saut', 'haut'], 400]],
  ['932-jeu-voute', [110, 3], [['droite', 'bas'], 900, ['droite', 'haut'], 700]],
  ['932-jeu-pontons', [134, 3], [['droite', 'bas'], 300, ['saut', 'bas'], 250, ['saut', 'haut'], 250]],
];
for (const [nom, [x, y], suite] of MOMENTS) {
  const [ex, ey] = await caseEcran(x, y);
  await souris('mousePressed', ex, ey, 'right');
  await souris('mouseReleased', ex, ey, 'right');
  if (!await attendre(enJeu, 20000)) { verifier(nom, false, await texteDe('#verdict')); continue; }
  await dormir(1200);
  await appuie('Enter', 'Enter', 13);
  await dormir(1200);
  await jouer(suite);
  await capture(nom);
  for (const t of Object.keys(T)) await touche('keyUp', ...T[t]);
  await fermer();
}

// --- 4. La Pieuvre passe 8e : ouverte depuis la liste des niveaux du jeu ---------------------
await evalue('editeurPepin.etat.modifie = false; 1');
await evalue(`(() => { const l = document.querySelector('#liste-niveaux'); l.value = 'monde_2/pieuvre.ron'; l.dispatchEvent(new Event('change', { bubbles: true })); return 1; })()`);
await attendre('editeurPepin.etat.niveau && editeurPepin.etat.niveau.cle === "pieuvre"');
const pieuvreAvant = await evalue('editeurPepin.texteExporte()');
await taper('numero', '8');
verifier('Pieuvre : numéro 8', (await evalue('editeurPepin.etat.niveau.numero')) === 8);
const fichierPieuvre = path.join(sortie, 'pieuvre.ron');
if (fs.existsSync(fichierPieuvre)) fs.unlinkSync(fichierPieuvre);
await cliquer('#exporter');
for (let i = 0; i < 40 && !fs.existsSync(fichierPieuvre); i++) await dormir(250);
verifier('export pieuvre.ron', fs.existsSync(fichierPieuvre));
if (fs.existsSync(fichierPieuvre)) {
  const apres = fs.readFileSync(fichierPieuvre, 'utf8');
  const diff = apres.split('\n').filter((l, i) => l !== pieuvreAvant.split('\n')[i]);
  verifier('Pieuvre : seule la ligne du numéro change', diff.length === 1 && /numero: 8/.test(diff[0]), JSON.stringify(diff));
}

verifier('aucune erreur de console', erreurs.length === 0, erreurs.join(' | '));
console.log(echecs ? `CAPTURE-932-ECHECS ${echecs}` : 'CAPTURE-932-OK');
ws.close();
chrome.kill();
process.exit(echecs ? 1 : 0);
