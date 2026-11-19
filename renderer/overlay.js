'use strict';
// Dùng cho 2 kiểu cửa sổ: "overlay" (khoá toàn màn hình) và "mini" (đồng hồ nhỏ + mèo ở góc màn hình).
(function () {
  const q = new URLSearchParams(location.search);
  const mode = q.get('mode') === 'mini' ? 'mini' : 'overlay';
  const primary = q.get('primary') !== '0';
  document.body.classList.add(mode);
  if (mode === 'overlay' && !primary) document.body.classList.add('secondary');

  const $ = (id) => document.getElementById(id);
  const esc = (s) => String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
  const cat = Cat.mount($('cat'), 'happy');
  let bubbleTimer = null, cheerTimer = null, last = null;
  let exitFlow = null, emgFlow = null, ready = false;

  function baseMood() {
    return last && last.exit && last.exit.requested ? 'worried' : 'happy';
  }
  function say(text, ms) {
    const b = $('bubble');
    b.textContent = text;
    b.classList.add('show');
    clearTimeout(bubbleTimer);
    bubbleTimer = setTimeout(() => b.classList.remove('show'), ms || 7000);
  }

  // Ngôn ngữ và cỡ chữ nằm trong cài đặt: đọc trước rồi mới dựng phần có chữ.
  function setup(settings) {
    I18N.set(I18N.detect(settings));
    // cửa sổ nhỏ có kích thước cố định nên không phóng to
    const sc = [1, 1.15, 1.3].includes(Number(settings.fontScale)) ? Number(settings.fontScale) : 1;
    if (mode === 'overlay') document.documentElement.style.zoom = String(sc);
    if (ready) return;
    ready = true;
    if (mode === 'overlay' && primary) {
      $('exit').hidden = false;
      exitFlow = ExitFlow.mount($('exit'));
      $('emg').hidden = false;
      emgFlow = EmergencyFlow.mount($('emg'));
    }
    if (mode === 'mini') {
      $('open').hidden = false;
      $('open').textContent = t('ov.open');
      $('open').addEventListener('click', () => window.api.openMain());
    }
  }

  function render(s) {
    if (!s || !s.active || !ready) return;
    last = s;
    $('time').textContent = ExitFlow.fmt(s.paused ? s.pauseLeftMs : s.phaseRemainingMs);
    $('phase').textContent = s.paused ? t('ov.paused')
      : s.phase === 'work' ? (s.intent ? t('ov.doing', { what: s.intent }) + (s.rounds > 1 ? ` · ${s.round}/${s.rounds}` : '') : t('ov.work', { r: s.round, n: s.rounds }))
        : t('ov.break');
    $('bar').style.width = `${s.phaseTotalMs ? (1 - s.phaseRemainingMs / s.phaseTotalMs) * 100 : 0}%`;
    $('meta').innerHTML = t('ov.left', { t: ExitFlow.fmt(s.totalRemainingMs) }) + (s.violations ? t('ov.viol', { n: s.violations }) : '');
    if (!cheerTimer) cat.set(baseMood());
    if (exitFlow) exitFlow.update(s);
    if (emgFlow) emgFlow.update(s);
  }

  window.api.onSession(render);
  window.api.onData((d) => { setup(d.settings); });
  window.api.getState().then((d) => { setup(d.settings); render(d.session); });

  window.api.onCheer((p) => {
    say(p.text);
    cat.set('cheer');
    clearTimeout(cheerTimer);
    cheerTimer = setTimeout(() => { cheerTimer = null; cat.set(baseMood()); }, 4500);
    if (p.voiceOn) TTS.speak(p.text, p.persona.voice, p.lang);
  });
})();
