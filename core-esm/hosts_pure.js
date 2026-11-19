// Phần thuần của hosts.js: xử lý văn bản file hosts, không đọc/ghi file, không xin quyền.

export const BEGIN = '# >>> DISTURBLESS BEGIN (không sửa đoạn này, tự gỡ khi phiên kết thúc)';
export const END = '# <<< DISTURBLESS END';

const DOMAIN_RE = /^(?=.{1,253}$)([a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z]{2,}$/;

export function cleanDomain(raw) {
  let d = String(raw || '').trim().toLowerCase();
  d = d.replace(/^[a-z]+:\/\//, '').split(/[/?#:]/)[0].replace(/^www\./, '');
  return DOMAIN_RE.test(d) ? d : null;
}

/** Danh sách tên miền (kèm www./m. cho tên miền gốc), loại trùng và loại giá trị sai. */
export function expandDomains(list) {
  const out = new Set();
  for (const raw of list || []) {
    const d = cleanDomain(raw);
    if (!d) continue;
    out.add(d);
    if (d.split('.').length <= 2) { out.add('www.' + d); out.add('m.' + d); }
  }
  return [...out];
}

export function stripBlock(text) {
  const re = new RegExp(`\\r?\\n?${escapeRe(BEGIN)}[\\s\\S]*?${escapeRe(END)}\\r?\\n?`, 'g');
  return text.replace(re, '\r\n').replace(/(\r?\n){3,}/g, '\r\n\r\n').trimEnd() + '\r\n';
}
const escapeRe = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

export function withBlock(text, domains) {
  const base = stripBlock(text);
  if (!domains.length) return base;
  const lines = [BEGIN];
  for (const d of domains) { lines.push(`0.0.0.0 ${d}`); lines.push(`:: ${d}`); }
  lines.push(END);
  return base + '\r\n' + lines.join('\r\n') + '\r\n';
}

export const hasBlock = (text) => text.includes(BEGIN);

