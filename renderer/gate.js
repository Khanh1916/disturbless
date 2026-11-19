'use strict';
// Cổng dừng vài giây: khi mở app hay tab bị chặn, đứng lại hít một hơi và nói lý do. Cổng không cho giữ app lại;
// lối ra duy nhất là đóng nó (hoặc dùng nút khẩn cấp trên đồng hồ).
(function () {
  const $ = (id) => document.getElementById(id);
  const esc = (s) => String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
  const REASONS = ['bored', 'habit', 'notify', 'curious', 'need'];
  const cat = Cat.mount($('gcat'), 'worried');
  let timer = null, info = null;

  function render(p, settings) {
    I18N.set(I18N.detect(settings));
    const sc = Math.min([1, 1.15, 1.3].includes(Number(settings.fontScale)) ? Number(settings.fontScale) : 1, 1.15);
    document.documentElement.style.zoom = String(sc);
    info = p;
    const sec = Math.max(1, Math.round(p.graceMs / 1000));
    $('gtitle').textContent = t('gate.title');
    $('gwhat').innerHTML = t(p.kind === 'tab' ? 'gate.tab' : 'gate.app', { name: esc(p.name) });
    $('gwhy').textContent = t('gate.why');
    $('greasons').innerHTML = REASONS.map((r) => `<button class="chip-btn" data-r="${r}">${t('gate.r.' + r)}</button>`).join('');
    $('gneed').textContent = t('gate.need.hint');
    $('gneed').hidden = true;
    $('gclose').innerHTML = `${Icon('check')} ${esc(t('gate.close', { name: p.name }))}`;
    $('gate').hidden = false;
    let left = sec;
    const tick = () => {
      $('gsec').textContent = String(left);
      $('gbreathe').textContent = t('gate.breathe', { s: left });
      $('garc').style.strokeDashoffset = String(163.4 * (1 - left / sec));
      if (left <= 0) { clearInterval(timer); window.api.gateDone(); return; }
      left -= 1;
    };
    clearInterval(timer);
    tick();
    timer = setInterval(tick, 1000);
  }

  $('greasons').addEventListener('click', (e) => {
    const b = e.target.closest('[data-r]');
    if (!b || !info) return;
    $('greasons').querySelectorAll('.chip-btn').forEach((x) => x.classList.toggle('sel', x === b));
    $('gneed').hidden = b.dataset.r !== 'need';
    window.api.gateReason({ name: info.name, reason: b.dataset.r });
  });
  $('gclose').addEventListener('click', () => { clearInterval(timer); window.api.gateClose({ kind: info.kind, pid: info.pid, title: info.title }); });

  window.api.onGate((p) => { window.api.getState().then((d) => render(p, d.settings)); });
})();
