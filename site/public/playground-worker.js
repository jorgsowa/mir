import init, { Playground } from './wasm/mir_wasm.js';

let playground = null;

try {
  await init();
  playground = new Playground();
  postMessage({ type: 'ready' });
} catch (e) {
  postMessage({ type: 'load-error', message: String(e) });
}

onmessage = ({ data }) => {
  if (!playground) return;
  const { id, code, php } = data;
  const t0 = performance.now();
  try {
    const issues = JSON.parse(playground.analyze(code, php));
    postMessage({ type: 'result', id, php, issues, ms: performance.now() - t0 });
  } catch (e) {
    // A Rust panic leaves the instance unusable; start a fresh one for the next request.
    try { playground = new Playground(); } catch {}
    postMessage({ type: 'analysis-error', id, message: String(e) });
  }
};
