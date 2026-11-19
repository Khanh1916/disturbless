// Evaluate JavaScript inside one of the app's windows through WebView2's debug port, optionally saving a screenshot.
//   node scripts/dev/cdp.js <port> <exact-page-url> <expression | file.js> [screenshot.png]
// Start the app with run-sandbox.ps1 first. Match pages by their EXACT url, e.g. http://tauri.localhost/ is the main
// window, while http://tauri.localhost/overlay.html?mode=mini (the clock) also contains that text.// usage: node cdp.js <port> <urlSubstr> <expr-file-or-expr> [screenshot.png]
const fs = require('fs');
(async () => {
  const [port, match, exprArg, shot] = process.argv.slice(2);
  const pages = await (await fetch(`http://127.0.0.1:${port}/json`)).json();
  const page = pages.find((p) => p.type === 'page' && p.url === match) || pages.find((p) => p.type === 'page' && p.url.includes(match));
  if (!page) { console.log('NO PAGE', pages.map((p) => p.url)); process.exit(2); }
  const ws = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((r) => ws.addEventListener('open', r));
  let id = 0; const waiting = new Map();
  ws.addEventListener('message', (m) => { const d = JSON.parse(m.data); if (waiting.has(d.id)) { waiting.get(d.id)(d); waiting.delete(d.id); } });
  const send = (method, params) => new Promise((res) => { const i = ++id; waiting.set(i, res); ws.send(JSON.stringify({ id: i, method, params })); });
  const expr = fs.existsSync(exprArg) ? fs.readFileSync(exprArg, 'utf8') : exprArg;
  const r = await send('Runtime.evaluate', { expression: expr, returnByValue: true, awaitPromise: true });
  console.log(JSON.stringify(r.result.result.value ?? r.result, null, 1));
  if (shot) { const s = await send('Page.captureScreenshot', { format: 'png' }); fs.writeFileSync(shot, Buffer.from(s.result.data, 'base64')); console.log('shot', shot); }
  ws.close();
})();

