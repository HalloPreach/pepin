// Web Worker du ROBOT de faisabilité (carte 10.3c), site de TEST seulement.
//
// Charge `robot.wasm` (crate Rust `robot` : le robot `core::faisabilite`, le même
// code que les tests) et le lance sur le niveau reçu. Il tourne ici, hors de la
// page : la page ne fige pas, et l'éditeur arrête le robot (bouton Arrêter ou
// limite de temps) en terminant simplement ce Worker.
//
// Message reçu : { texte: <RON du niveau>, planer: <bool> }.
// Message rendu : le JSON de `robot::analyser_texte`, lu, plus `duree_ms` :
//   { etat: 'ok', arrivee, pas, secondes, lanternes, lanternes_total, gouttes,
//     etats, dernier: [x, y], tuile, joueur: [l, h], chemin: [[x, y]...], raison }
//   (positions en px : coin haut gauche de Pépin), ou { etat: 'erreur', message }.
let module = null;

async function charger() {
  if (!module) {
    // `instantiate` (pas `instantiateStreaming`) : marche même si le serveur ne
    // dit pas `application/wasm`.
    const reponse = await fetch('../robot.wasm');
    if (!reponse.ok) throw new Error(`robot.wasm introuvable (${reponse.status}) : build de QA ?`);
    const { instance } = await WebAssembly.instantiate(await reponse.arrayBuffer(), {});
    module = instance.exports;
  }
  return module;
}

self.onmessage = async (e) => {
  try {
    const m = await charger();
    const octets = new TextEncoder().encode(e.data.texte);
    const adresse = m.robot_tampon(octets.length);
    new Uint8Array(m.memory.buffer, adresse, octets.length).set(octets);
    const debut = performance.now();
    const n = m.robot_lancer(octets.length, e.data.planer ? 1 : 0);
    const duree_ms = Math.round(performance.now() - debut);
    // La mémoire a pu grandir pendant le robot : on relit `memory.buffer`.
    const json = new TextDecoder().decode(new Uint8Array(m.memory.buffer, m.robot_resultat(), n));
    self.postMessage({ ...JSON.parse(json), duree_ms });
  } catch (err) {
    self.postMessage({ etat: 'erreur', message: `robot : ${err}` });
  }
};
