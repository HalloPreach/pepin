// Pilote CDP de la QA « clavier AZERTY / raccourcis 1-5 / écran haute densité ».
// Usage : CHROME=/chemin/chrome node docs/qa-clavier-ecran.mjs SCENARIO PORT_HTTP PREFIXE [largeur hauteur dpr]
//
// Scénarios :
//   azerty  : au clavier AZERTY (codes physiques envoyés par le navigateur), bascule le son avec la
//             touche M (code « Semicolon »), navigue au menu avec Z/S, court avec Q/D, saute avec Z.
//   chiffres: en plein jeu (niveau 1), tape la touche 3 (AZERTY : le guillemet) ; en production rien ne
//             doit changer, avec `./build.sh --qa` la carte du niveau 3 apparaît.
//   ecran   : fenêtre largeur x hauteur (pixels CSS) avec le facteur dpr ; capture le titre, mesure le
//             rectangle de jeu (en pixels physiques), clique « JOUER » (souris, ou doigt si dpr > 1 et
//             mobile), puis joue : droite + saut avec les boutons tactiles.
// Écrit PREFIXE-*.png et affiche les mesures sous forme JSON (une ligne par mesure).
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import zlib from 'node:zlib';

const [scenario, httpPort, prefixe, W = '1280', H = '720', DPR = '1'] = process.argv.slice(2);
const largeur = Number(W), hauteur = Number(H), dpr = Number(DPR);
const mobile = process.env.MOBILE === '1';
const CHROME = process.env.CHROME || 'chromium';
const dbg = 9400 + Math.floor(Math.random() * 400);
const profil = fs.mkdtempSync(path.join(process.env.TMPDIR || '/tmp', 'prof-'));
const chrome = spawn(CHROME, [
  `--remote-debugging-port=${dbg}`, `--user-data-dir=${profil}`, '--headless=new', '--no-sandbox',
  '--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist',
  `--window-size=${largeur},${hauteur}`, 'about:blank',
], { stdio: 'ignore' });
const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
let cible;
for (let i = 0; i < 60; i++) {
  try { const l = await (await fetch(`http://127.0.0.1:${dbg}/json`)).json(); cible = l.find((t) => t.type === 'page'); if (cible) break; } catch {}
  await dormir(200);
}
const ws = new WebSocket(cible.webSocketDebuggerUrl);
await new Promise((r) => (ws.onopen = r));
let id = 0; const att = new Map();
ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.id && att.has(d.id)) { att.get(d.id)(d); att.delete(d.id); } };
const cdp = (method, params = {}) => new Promise((r) => { const i = ++id; att.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });

// --- Clavier : on envoie le CODE PHYSIQUE (event.code) ET la lettre imprimée (event.key). ---
const VK = { KeyA: 65, KeyD: 68, KeyS: 83, KeyW: 87, KeyQ: 81, KeyZ: 90, KeyM: 77, Semicolon: 186, Enter: 13, Space: 32,
  ArrowUp: 38, ArrowDown: 40, ArrowLeft: 37, ArrowRight: 39, Digit1: 49, Digit2: 50, Digit3: 51, Digit4: 52, Digit5: 53 };
const touche = (code, key, bas) => cdp('Input.dispatchKeyEvent', { type: bas ? 'keyDown' : 'keyUp', code, key, windowsVirtualKeyCode: VK[code], nativeVirtualKeyCode: VK[code] });
const taper = async (code, key, ms = 120) => { await touche(code, key, true); await dormir(ms); await touche(code, key, false); };
const capture = async (nom) => {
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  const buf = Buffer.from(r.result.data, 'base64');
  fs.writeFileSync(nom, buf);
  return buf;
};

// --- Décodage PNG minimal (RGBA 8 bits) pour mesurer sans dépendance. ---
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
// Rectangle occupé par le jeu : tout ce qui n'est pas la couleur de la bande (coin haut gauche).
function rectangleDeJeu(png) {
  const { l, h, bpp, px } = png; const ref = [px[0], px[1], px[2]];
  const diff = (x, y) => { const i = (y * l + x) * bpp; return Math.abs(px[i] - ref[0]) + Math.abs(px[i + 1] - ref[1]) + Math.abs(px[i + 2] - ref[2]) > 24; };
  let x0 = l, x1 = -1, y0 = h, y1 = -1;
  for (let y = 0; y < h; y += 1) for (let x = 0; x < l; x += 1) if (diff(x, y)) { if (x < x0) x0 = x; if (x > x1) x1 = x; if (y < y0) y0 = y; if (y > y1) y1 = y; }
  return x1 < 0 ? null : { x: x0, y: y0, l: x1 - x0 + 1, h: y1 - y0 + 1, canvas_l: l, canvas_h: h };
}
const mesure = (nom, v) => console.log(JSON.stringify({ mesure: nom, ...v }));
// Part de pixels qui diffèrent entre deux captures (0 = identiques).
// `zone` : [x0, y0, x1, y1] en fractions de l'image, pour ignorer l'animation du fond.
function ecart(a, b, zone = [0, 0, 1, 1]) {
  const A = decoderPng(a), B = decoderPng(b); if (A.l !== B.l || A.h !== B.h) return 1;
  const [x0, y0, x1, y1] = [zone[0] * A.l, zone[1] * A.h, zone[2] * A.l, zone[3] * A.h].map(Math.floor);
  let n = 0;
  for (let y = y0; y < y1; y++) for (let x = x0; x < x1; x++) {
    const i = (y * A.l + x) * A.bpp;
    if (Math.abs(A.px[i] - B.px[i]) + Math.abs(A.px[i + 1] - B.px[i + 1]) + Math.abs(A.px[i + 2] - B.px[i + 2]) > 30) n++;
  }
  return n / ((x1 - x0) * (y1 - y0));
}
// Ligne « SON : ... » du menu du titre (logique y 114-134 sur 180, x 60-260 sur 320).
const ZONE_SON = [0.33, 0.63, 0.7, 0.75];
// Empreinte du texte de la ligne « SON » : colonnes (x) occupées par les pixels de la couleur du texte.
// Le fond animé ne pèse pas : on ne compte que les pixels très clairs du texte (lettres des menus).
function empreinteTexte(buf, zone = ZONE_SON) {
  const A = decoderPng(buf);
  const [x0, y0, x1, y1] = [zone[0] * A.l, zone[1] * A.h, zone[2] * A.l, zone[3] * A.h].map(Math.floor);
  let n = 0;
  for (let y = y0; y < y1; y++) for (let x = x0; x < x1; x++) {
    const i = (y * A.l + x) * A.bpp;
    if (A.px[i] > 150 && A.px[i + 1] > 165 && A.px[i + 2] > 200) n++;
  }
  return n;
}

await cdp('Page.enable');
await cdp('Page.bringToFront');
await cdp('Emulation.setDeviceMetricsOverride', { width: largeur, height: hauteur, deviceScaleFactor: dpr, mobile });
if (mobile) await cdp('Emulation.setTouchEmulationEnabled', { enabled: true, maxTouchPoints: 5 });
await cdp('Page.navigate', { url: `http://127.0.0.1:${httpPort}/index.html` });
await dormir(4500);
await cdp('Page.bringToFront');
const focus = async () => { await cdp('Runtime.evaluate', { expression: 'document.getElementById("glcanvas").focus()' }); };
await focus();

const doigt = async (x, y, ms = 150) => {
  await cdp('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [{ x, y }] });
  await dormir(ms);
  await cdp('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
};

if (scenario === 'azerty') {
  const titre = await capture(`${prefixe}-1-titre.png`);
  // M d'un clavier AZERTY = position physique « Semicolon ».
  await taper('Semicolon', 'm'); await dormir(600);
  const apresM = await capture(`${prefixe}-2-titre-apres-M.png`);
  mesure('azerty_M_change_le_titre', { pixels_texte_son_avant: empreinteTexte(titre), pixels_texte_son_apres: empreinteTexte(apresM) });
  await taper('Semicolon', 'm'); await dormir(600);
  const apresM2 = await capture(`${prefixe}-3-titre-apres-M-bis.png`);
  mesure('azerty_M_bis_revient', { pixels_texte_son: empreinteTexte(apresM2) });
  // QWERTY : la touche M reste valable.
  await taper('KeyM', 'm'); await dormir(600);
  const apresQ = await capture(`${prefixe}-4-titre-apres-M-qwerty.png`);
  mesure('qwerty_M_change_le_titre', { pixels_texte_son: empreinteTexte(apresQ) });
  await taper('KeyM', 'm'); await dormir(600);
  // Menu : S descend, Z (code KeyW) remonte.
  await taper('KeyS', 's'); await dormir(300);
  const bas = await capture(`${prefixe}-5-menu-S.png`);
  await taper('KeyW', 'z'); await dormir(300);
  const haut = await capture(`${prefixe}-6-menu-Z.png`);
  mesure('menu_S_puis_Z', { ecart_bas_vs_titre: ecart(titre, bas), ecart_haut_vs_titre: ecart(titre, haut) });
  await taper('Enter', 'Enter'); await dormir(3000);
  await taper('Enter', 'Enter'); await dormir(3000);
  const depart = await capture(`${prefixe}-7-jeu-depart.png`);
  // Courir à droite avec D (même position partout), puis revenir avec Q (AZERTY : code KeyA).
  await touche('KeyD', 'd', true); await dormir(1800); await touche('KeyD', 'd', false); await dormir(400);
  const droite = await capture(`${prefixe}-8-jeu-apres-D.png`);
  await touche('KeyA', 'q', true); await dormir(1200); await touche('KeyA', 'q', false); await dormir(400);
  const gauche = await capture(`${prefixe}-9-jeu-apres-Q.png`);
  mesure('jeu_D_deplace', { ecart: ecart(depart, droite) });
  mesure('jeu_Q_deplace_a_gauche', { ecart_avec_droite: ecart(droite, gauche), ecart_avec_depart: ecart(depart, gauche) });
  // Sauter avec Z (AZERTY : code KeyW) : capture en l'air.
  await touche('KeyW', 'z', true); await dormir(250);
  const air = await capture(`${prefixe}-10-jeu-saut-Z.png`);
  await touche('KeyW', 'z', false);
  mesure('jeu_Z_saute', { ecart_avec_sol: ecart(gauche, air) });
} else if (scenario === 'chiffres') {
  await taper('Enter', 'Enter'); await dormir(3000); // titre -> carte du niveau 1
  await taper('Enter', 'Enter'); await dormir(3500); // carte -> jeu
  const avant = await capture(`${prefixe}-1-jeu-niveau1.png`);
  // Le sol (bande basse) est fixe tant que la caméra ne bouge pas : sert de témoin de niveau.
  const SOL = [0, 0.78, 1, 1];
  // Touches 1 à 5 (AZERTY : & é " ' ( sans majuscule) : le code physique est Digit1 à Digit5.
  const lettres = ['&', 'é', '"', "'", '('];
  for (let n = 1; n <= 5; n++) {
    await taper(`Digit${n}`, lettres[n - 1]); await dormir(2500);
    const c = await capture(`${prefixe}-${n + 1}-apres-touche-${n}.png`);
    mesure(`touche_${n}`, { ecart_sol_avec_niveau1: ecart(avant, c, SOL), ecart_total: ecart(avant, c) });
  }
} else if (scenario === 'ecran') {
  const titre = await capture(`${prefixe}-1-titre.png`);
  const rect = rectangleDeJeu(decoderPng(titre));
  mesure('rectangle_de_jeu_physique', { fenetre_css: [largeur, hauteur], dpr, ...rect,
    facteur: rect ? rect.l / 320 : null, ratio_ecran_utilise: rect ? (rect.l * rect.h) / (rect.canvas_l * rect.canvas_h) : null });
  // « JOUER » : première ligne du menu (logique x 40-280, y 74-94) -> centre en pixels CSS.
  const k = rect.l / 320; const ox = rect.x, oy = rect.y;
  const ligne = Number(process.env.LIGNE || 0); // 0 JOUER, 1 COMMENT JOUER, ...
  const cy = 74 + 20 * ligne + 10;              // menu du titre : y0 = 74, pas = 20 (ecrans/mod.rs)
  const pxCss = (ox + 160 * k) / dpr, pyCss = (oy + cy * k) / dpr;
  if (mobile) { await doigt(pxCss, pyCss); await dormir(500); await doigt(pxCss, pyCss); }
  else {
    await cdp('Input.dispatchMouseEvent', { type: 'mouseMoved', x: pxCss, y: pyCss });
    await dormir(300);
    await capture(`${prefixe}-2-survol-jouer.png`);
    await cdp('Input.dispatchMouseEvent', { type: 'mousePressed', x: pxCss, y: pyCss, button: 'left', clickCount: 1 });
    await cdp('Input.dispatchMouseEvent', { type: 'mouseReleased', x: pxCss, y: pyCss, button: 'left', clickCount: 1 });
  }
  await dormir(3000);
  const carte = await capture(`${prefixe}-3-apres-clic-jouer.png`);
  mesure('clic_jouer_a_change_ecran', { ecart_avec_titre: ecart(titre, carte), ligne, clic_css: [Math.round(pxCss), Math.round(pyCss)] });
  if (ligne > 0) { ws.close(); chrome.kill(); await dormir(500); try { fs.rmSync(profil, { recursive: true, force: true }); } catch {} process.exit(0); }
  // Carte -> jeu (tactile : un toucher ; souris : Entrée).
  if (mobile) await doigt(largeur / 2, hauteur / 2); else await taper('Enter', 'Enter');
  await dormir(3500);
  const jeu = await capture(`${prefixe}-4-jeu.png`);
  if (mobile) {
    // Boutons tactiles : maintenir « droite » (zone droite = 2e bouton en bas à gauche) puis sauter.
    // Disposition (tactile.rs) : taille = clamp(0.26 * petit côté, 56, 180) en pixels CSS.
    const pc = Math.min(largeur, hauteur); const taille = Math.min(180, Math.max(56, pc * 0.26)); const marge = Math.max(8, pc * 0.04);
    const bas = hauteur - marge - taille / 2;
    const dxd = marge * 2 + taille + taille / 2, dxs = largeur - marge - taille * 1.3 / 2;
    await cdp('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [{ x: dxd, y: bas, id: 1 }] });
    await dormir(1800);
    const marche = await capture(`${prefixe}-5-tactile-droite.png`);
    await cdp('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
    await dormir(300);
    await cdp('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [{ x: dxs, y: hauteur - marge - taille * 1.3 / 2, id: 2 }] });
    await dormir(300);
    const saut = await capture(`${prefixe}-6-tactile-saut.png`);
    await cdp('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
    // Gauche : revenir vers le début (le décor défile dans l'autre sens).
    await dormir(300);
    await cdp('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [{ x: marge * 0.5 + taille / 2 + marge, y: bas, id: 3 }] });
    await dormir(900);
    const gauche = await capture(`${prefixe}-7-tactile-gauche.png`);
    await cdp('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
    mesure('tactile', { droite_deplace: ecart(jeu, marche), saut: ecart(marche, saut), gauche_deplace: ecart(saut, gauche) });
    // Pause : petit bouton en haut à droite (côté = taille / 2, marge en haut et à droite).
    await dormir(300);
    const avantPause = await capture(`${prefixe}-8-avant-pause.png`);
    await doigt(largeur - marge - taille / 4, marge + taille / 4); await dormir(700);
    const pause = await capture(`${prefixe}-9-pause-tactile.png`);
    mesure('tactile_pause', { ecart: ecart(avantPause, pause), bouton_css: [Math.round(largeur - marge - taille / 4), Math.round(marge + taille / 4)] });
    // Menu pause : un doigt sur « REPRENDRE » (première ligne) doit revenir au jeu.
    // (menu de pause : y0 = 68, pas = 20 ; même cadre de jeu que sur le titre)
    await doigt((rect.x + 160 * k) / dpr, (rect.y + (68 + 10) * k) / dpr); await dormir(700);
    const reprise = await capture(`${prefixe}-10-apres-reprendre-tactile.png`);
    mesure('tactile_menu_pause_reprendre', { ecart_avec_pause: ecart(pause, reprise) });
  }
}
ws.close(); chrome.kill();
await dormir(500);
try { fs.rmSync(profil, { recursive: true, force: true }); } catch {}
process.exit(0);
