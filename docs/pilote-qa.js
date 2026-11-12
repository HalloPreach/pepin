// Pilote d'audit (lecture seule) : horloge virtuelle + rejeu image par image.
// Injecté dans la page du jeu. Ne modifie rien sur disque.
// - Date.now est remplacé par une horloge virtuelle : chaque image « active »
//   avance d'exactement 1/60 s, donc le jeu simule exactement 1 pas par image.
// - La boucle du jeu (requestAnimationFrame) est remplacée par la nôtre : on
//   appelle wasm_exports.frame() nous-mêmes, plusieurs fois par image réelle
//   (avance rapide) ou avec l'horloge figée (gel : le jeu redessine sans avancer).
(() => {
  if (window.__P) return 'deja';
  const canvas = document.getElementById('glcanvas');
  const rafReel = window.requestAnimationFrame.bind(window);
  const P = window.__P = {
    t: 1e6, n: 0, script: '', pos: 0, cible: 0, masque: 0,
    file: [], parRaf: 4, erreurs: [], gel: 0,
  };
  Date.now = () => P.t;
  window.requestAnimationFrame = () => 0; // arrête la boucle propre du jeu
  const CODES = { 1: 'Enter', 2: 'ArrowLeft', 4: 'ArrowRight', 8: 'Space' };
  const ev = (code) => ({
    code, key: code, ctrlKey: false, shiftKey: false, altKey: false, metaKey: false,
    repeat: false, preventDefault() {}, stopPropagation() {},
  });
  P.bas = (code) => canvas.onkeydown(ev(code));
  P.haut = (code) => canvas.onkeyup(ev(code));
  P.appliquer = (m) => {
    for (const bit of [1, 2, 4, 8]) {
      const veut = (m & bit) !== 0, a = (P.masque & bit) !== 0;
      if (veut && !a) P.bas(CODES[bit]);
      if (!veut && a) P.haut(CODES[bit]);
    }
    P.masque = m;
  };
  const image = (avance) => {
    if (avance) { P.n++; P.t = 1e6 + P.n * 16.66666667; }
    wasm_exports.frame();
  };
  const boucle = () => {
    try {
      let fait = 0;
      for (let k = 0; k < P.parRaf; k++) {
        if (P.pos < P.cible && P.pos < P.script.length) {
          P.appliquer(parseInt(P.script[P.pos], 16)); P.pos++; image(true); fait++;
        } else if (P.file.length) {
          const s = P.file[0];
          const debut = s.taper && !s.fait;
          if (debut) for (const c of s.taper) P.bas(c);
          P.appliquer(s.m || 0);
          image(true); fait++;
          if (debut) { for (const c of s.taper) P.haut(c); s.fait = true; }
          if (--s.n <= 0) P.file.shift();
        } else break;
      }
      if (!fait) { image(false); P.gel++; }
    } catch (e) { P.erreurs.push(String(e)); }
    rafReel(boucle);
  };
  rafReel(boucle);
  // Lecture EXACTE de l'image affichée : on redessine (horloge figée) puis on lit
  // le tampon WebGL dans la même tâche JS. Renvoie la grille logique 320x180
  // (1 caractère = 1 pixel logique : indice hexadécimal de palette, 'x' = hors
  // palette), le nombre de blocs KxK non uniformes (flou / sous-pixels) et les
  // couleurs hors palette.
  const PAL = ['10141F', '1C2336', '2D3A54', '46587A', '7F93B5', 'A9B8D0', 'C3D1E6', 'F3F1EA',
    'FF5A4F', 'FF9A7A', 'FFB347', 'FFD84A', 'FFF1A8', '62D0E8', '2F8F9D', '5B4A8A'];
  const index = new Map(PAL.map((c, i) => [parseInt(c, 16), i.toString(16).toUpperCase()]));
  P.lire = () => {
    wasm_exports.frame();
    const lie = gl.getParameter(gl.FRAMEBUFFER_BINDING);
    const w = canvas.width, h = canvas.height;
    const buf = new Uint8Array(w * h * 4);
    gl.readPixels(0, 0, w, h, gl.RGBA, gl.UNSIGNED_BYTE, buf);
    const K = Math.max(1, Math.min(Math.floor(w / 320), Math.floor(h / 180)));
    const x0 = Math.floor((w - 320 * K) / 2), y0 = Math.floor((h - 180 * K) / 2);
    const px = (x, y) => { const o = ((h - 1 - y) * w + x) * 4; return (buf[o] << 16) | (buf[o + 1] << 8) | buf[o + 2]; };
    let grille = '', flou = 0, alpha = 0; const hors = {};
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
    }
    // Bandes hors zone de jeu (si la fenêtre n'est pas un multiple exact) : couleur du coin.
    return JSON.stringify({ w, h, K, x0, y0, flou, hors, lie: lie === null, coin: px(0, 0).toString(16), grille });
  };
  return 'installe';
})()
