'use strict';
// Checks that renderer/i18n.js has the same keys (and {placeholders}) in Vietnamese and English.
// Run: node scripts/dev/check-i18n.js
const fs = require('fs');
const path = require('path');
const vm = require('vm');
const src = fs.readFileSync(path.join(__dirname, '..', '..', 'renderer', 'i18n.js'), 'utf8');
const sandbox = { window: {}, document: { documentElement: {}, title: '' }, navigator: { language: 'en' } };
vm.runInNewContext(src, sandbox);
const { vi, en } = sandbox.window.I18N.DICT;
const holders = (v) => (typeof v === 'string' ? [...v.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort().join(',') : '(fn)');
const problems = [];
for (const k of Object.keys(vi)) if (!(k in en)) problems.push(`missing in en: ${k}`);
for (const k of Object.keys(en)) if (!(k in vi)) problems.push(`missing in vi: ${k}`);
for (const k of Object.keys(vi)) {
  if (k in en && typeof vi[k] === 'string' && typeof en[k] === 'string' && holders(vi[k]) !== holders(en[k])) problems.push(`placeholders differ: ${k}`);
}
// every t('key') used by the renderer must exist
const used = new Set();
for (const f of fs.readdirSync(path.join(__dirname, '..', '..', 'renderer')).filter((n) => n.endsWith('.js') && n !== 'i18n.js')) {
  const text = fs.readFileSync(path.join(__dirname, '..', '..', 'renderer', f), 'utf8');
  for (const m of text.matchAll(/\bt\('([\w.]+)'/g)) used.add(m[1]);
}
// a key ending in a dot is a prefix built at run time, e.g. t('goal.' + id)
for (const k of used) {
  const ok = k.endsWith('.') ? Object.keys(vi).some((x) => x.startsWith(k)) : k in vi;
  if (!ok) problems.push(`used but undefined: ${k}`);
}
if (problems.length) { console.error(problems.join('\n')); process.exit(1); }
console.log(`i18n ok: ${Object.keys(vi).length} keys in vi and en, ${used.size} literal keys used`);
