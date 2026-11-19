// Phần thuần của blocker.js: chọn cửa sổ vi phạm. Không chạy PowerShell, không đóng tiến trình.

// Tiến trình hệ thống KHÔNG BAO GIỜ bị đóng, kể cả khi người dùng tự thêm vào danh sách chặn.
// (Không có chúng thì Windows không dùng được, hoặc đóng giữa chừng có thể hỏng cài đặt/cập nhật.)
const SYSTEM_SAFE = new Set([
  // giao diện Windows
  'explorer', 'applicationframehost', 'searchhost', 'searchapp', 'searchui',
  'startmenuexperiencehost', 'shellexperiencehost', 'textinputhost', 'ctfmon',
  'lockapp', 'logonui', 'dwm', 'widgets', 'systemsettings', 'securityhealthsystray',
  'sihost', 'runtimebroker', 'screenclippinghost', 'snippingtool', 'taskhostw', 'fontdrvhost',
  // lõi hệ thống
  'csrss', 'winlogon', 'wininit', 'smss', 'services', 'lsass', 'svchost', 'system', 'registry', 'userinit',
  // cài đặt, cập nhật, quyền quản trị (đóng giữa chừng có thể làm hỏng)
  'msiexec', 'wusa', 'trustedinstaller', 'tiworker', 'mousocoreworker', 'musnotification', 'musnotificationux',
  'usoclient', 'wuauclt', 'consent', 'credentialuibroker', 'dllhost',
]);

export const normName = (n) => String(n || '').toLowerCase().replace(/\.exe$/, '').trim();

/** Chương trình nằm trong thư mục Windows (C:\Windows\...) là chương trình của hệ điều hành. */
export function isSystemPath(p, systemRoot) {
  const root = String(systemRoot || '').toLowerCase().replace(/[\\/]+$/, '');
  return !!root && String(p || '').toLowerCase().replace(/\//g, '\\').startsWith(root + '\\');
}

/**
 * Chọn cửa sổ vi phạm. Ba lớp bảo vệ luôn áp dụng: không đụng chính app này, không đụng
 * SYSTEM_SAFE, và (ở chế độ "chỉ cho phép") không đụng chương trình của Windows hay chương trình
 * không xác minh được đường dẫn (thường là quyền cao, vốn cũng không đóng được).
 * @param {Array<{pid,name,title,path}>} windows
 * @param {{appMode:'allow'|'block'|'off', allowed:string[], blocked:string[], selfPids:number[], systemRoot?:string}} cfg
 * @returns danh sách cửa sổ vi phạm
 */
export function evaluate(windows, cfg) {
  if (!cfg || cfg.appMode === 'off') return [];
  const allowed = new Set((cfg.allowed || []).map(normName));
  const blocked = new Set((cfg.blocked || []).map(normName));
  const self = new Set(cfg.selfPids || []);
  return windows.filter((w) => {
    const n = normName(w.name);
    if (self.has(w.pid) || SYSTEM_SAFE.has(n)) return false;
    if (cfg.appMode === 'block') return blocked.has(n); // chỉ đóng đúng thứ được liệt kê
    // allow-list: mọi thứ không có trong danh sách đều vi phạm, trừ chương trình hệ thống
    if (allowed.has(n)) return false;
    if (!w.path) return false;                          // không xác minh được → bỏ qua cho an toàn
    if (isSystemPath(w.path, cfg.systemRoot)) return false;
    return true;
  });
}

