// Thống kê từ lịch sử phiên. Ngày tính theo giờ địa phương.

const pad = (n) => String(n).padStart(2, '0');
export const dayKey = (ts) => {
  const d = new Date(ts);
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
};
const addDays = (ts, n) => {
  const d = new Date(ts);
  d.setDate(d.getDate() + n);
  return d.getTime();
};

/**
 * history: [{ startedAt, endedAt, focusedMin, plannedMin, completed, violations, restarts }]
 * Một ngày được tính vào streak nếu có ít nhất 1 phiên hoàn thành.
 */
export function summarize(history, now = Date.now()) {
  const minPerDay = {};
  const doneDays = new Set();
  let total = 0, completed = 0, abandoned = 0, violations = 0;

  let emergencies = 0;
  const quitDays = new Set(); // ngày có phiên bỏ dở (không phải khẩn cấp)

  for (const h of history) {
    // Phiên khẩn cấp không tính giờ, không tính vào tỉ lệ hoàn thành và không làm mất chuỗi.
    if (h.emergency) { emergencies++; continue; }
    const k = dayKey(h.endedAt || h.startedAt);
    minPerDay[k] = (minPerDay[k] || 0) + (h.focusedMin || 0);
    total += h.focusedMin || 0;
    violations += h.violations || 0;
    if (h.completed) { completed++; doneDays.add(k); } else { abandoned++; quitDays.add(k); }
  }

  // streak hiện tại: tính lùi từ hôm nay (hoặc hôm qua nếu hôm nay chưa có phiên).
  // Bỏ dở một phiên trong hôm nay mà chưa hoàn thành phiên nào thì mất chuỗi ngay.
  const todayKey = dayKey(now);
  let cursor = now;
  if (!doneDays.has(todayKey)) cursor = addDays(cursor, -1);
  let streak = 0;
  if (!(quitDays.has(todayKey) && !doneDays.has(todayKey))) {
    while (doneDays.has(dayKey(cursor))) { streak++; cursor = addDays(cursor, -1); }
  }

  // streak dài nhất
  const sorted = [...doneDays].sort();
  let best = 0, run = 0, prev = null;
  for (const k of sorted) {
    const [y, m, d] = k.split('-').map(Number);
    const ts = new Date(y, m - 1, d, 12).getTime();
    run = prev !== null && dayKey(addDays(prev, 1)) === k ? run + 1 : 1;
    best = Math.max(best, run);
    prev = ts;
  }

  const days = [];
  for (let i = 6; i >= 0; i--) {
    const ts = addDays(now, -i);
    const k = dayKey(ts);
    days.push({ key: k, dow: new Date(ts).getDay(), min: minPerDay[k] || 0 });
  }

  const sessions = completed + abandoned;
  return {
    todayMin: minPerDay[dayKey(now)] || 0,
    weekMin: days.reduce((a, d) => a + d.min, 0),
    totalMin: total,
    sessions,
    completed,
    abandoned,
    completionRate: sessions ? Math.round((completed / sessions) * 100) : 0,
    emergencies,
    violations,
    streak,
    bestStreak: best,
    days,
  };
}

