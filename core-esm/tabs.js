// Nhận diện tab trình duyệt bị cấm qua tiêu đề cửa sổ.
// Tiêu đề cửa sổ trình duyệt = tên tab đang xem + tên trình duyệt, ví dụ:
//   "Mèo nhảy múa - YouTube - Google Chrome"
//   "(3) Facebook - Personal - Microsoft Edge"
//   "youtube.com - Microsoft Edge"   (trang lỗi khi website đã bị chặn bằng hosts)
// Chỉ khớp khi tên trang web đứng CUỐI tiêu đề (hoặc tiêu đề chính là tên miền), nhờ vậy
// bài viết "Steam engine - Wikipedia" không bị nhầm với Steam.

export const BROWSERS = new Set(['chrome', 'msedge', 'firefox', 'brave', 'opera', 'vivaldi', 'iexplore', 'chromium']);
const BROWSER_TAIL = /\s[-–—]\s(Google Chrome|Microsoft.{0,2}Edge|Mozilla Firefox|Brave|Opera|Vivaldi|Chromium)\s*$/i;

const esc = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

/** Bỏ phần tên trình duyệt và "and N more pages" ở cuối tiêu đề. */
export function pageTitle(title) {
  return String(title || '').replace(/\s+and\s+\d+\s+more\s+pages?/i, '').replace(BROWSER_TAIL, '').trim();
}

/** Tên thương hiệu suy ra từ tên miền: "tinhte.vn" → "tinhte". Quá ngắn (vd "x") thì bỏ. */
export function brandOf(domain) {
  const parts = String(domain || '').toLowerCase().split('.').filter(Boolean);
  if (parts.length < 2) return null;
  const label = parts[parts.length - 2];
  return label.length >= 3 ? label : null;
}

/**
 * @param {string} title   tiêu đề cửa sổ trình duyệt
 * @param {string[]} brands tên thương hiệu cần chặn (YouTube, Facebook…)
 * @param {string[]} domains tên miền cần chặn (khớp khi tab là trang lỗi chỉ hiện tên miền)
 * @returns tên khớp đầu tiên, hoặc null
 */
export function matchBanned(title, brands, domains) {
  const t = pageTitle(title);
  if (!t) return null;
  for (const d of domains || []) {
    if (new RegExp(`^(https?://)?(www\\.|m\\.)?${esc(d)}(/.*)?$`, 'i').test(t)) return d;
  }
  for (const b of brands || []) {
    const re = new RegExp(`^(\\(\\d+\\)\\s*)?(.*?\\s*[-|–—/·:]\\s*)?${esc(b)}(\\s*[-–—]\\s[^-–—]{1,30})?$`, 'i');
    if (re.test(t)) return b;
  }
  return null;
}

