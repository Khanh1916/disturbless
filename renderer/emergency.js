'use strict';
// Nút khẩn cấp: tạm dừng 10 phút (1 lần/phiên) hoặc dừng hẳn phiên (chọn lý do + giữ nút 3 giây).
// Không giới hạn số lần dùng, nhưng mọi lần đều được ghi vào lịch sử kèm lý do.
(function () {
  const REASONS = [
    ['health', 'heart-pulse'],
    ['call', 'phone'],
    ['need', 'unlock'],
    ['mistake', 'undo'],
    ['bug', 'bug'],
    ['other', 'alert'],
  ];
  const HOLD_MS = 3000;
  const label = (id) => t(REASONS.some((r) => r[0] === id) ? 'emg.r.' + id : 'emg.r.default');

  function mount(el) {
    el.innerHTML = `
      <div class="emg">
        <div data-s="idle"><button class="btn ghost" data-e="open">${Icon('life-buoy')} ${t('emg.open')}</button></div>
        <div data-s="panel" class="emg-panel" hidden>
          <p class="hint">${t('emg.ask')}</p>
          <div class="emg-reasons">${REASONS.map(([id, em]) => `<button class="chip-btn" data-r="${id}">${Icon(em)} ${t('emg.r.' + id)}</button>`).join('')}</div>
          <div class="emg-actions">
            <button class="btn col" data-e="pause"><b>${Icon('pause')} ${t('emg.pause')}</b><small data-n="pause">${t('emg.pause.s')}</small></button>
            <button class="btn danger hold col" data-e="stop" disabled><i class="fill"></i><b>${Icon('stop')} ${t('emg.stop')}</b><small>${t('emg.stop.s')}</small></button>
          </div>
          <p class="hint err" style="color:var(--danger)" hidden></p>
          <button class="btn ghost" data-e="close">${t('emg.close')}</button>
        </div>
        <div data-s="paused" class="emg-paused" hidden>
          <p><b>${Icon('pause')} ${t('emg.paused')}</b><span class="rest">${t('emg.paused.left', { t: '<span class="count">--:--</span>' })}</span></p>
          <button class="btn primary" data-e="resume">${Icon('play')} ${t('emg.resume')}</button>
        </div>
      </div>`;
    const $ = (s) => el.querySelector(s);
    const stages = { idle: $('[data-s=idle]'), panel: $('[data-s=panel]'), paused: $('[data-s=paused]') };
    const stopBtn = $('[data-e=stop]'), pauseBtn = $('[data-e=pause]'), err = $('.err');
    let open = false, reason = null, holdTimer = null, last = null;

    const show = (name) => { for (const [k, n] of Object.entries(stages)) n.hidden = k !== name; };
    const fail = (msg) => { err.textContent = msg; err.hidden = !msg; };
    const cancelHold = () => { clearTimeout(holdTimer); holdTimer = null; stopBtn.classList.remove('holding'); };
    const startHold = () => {
      if (stopBtn.disabled || holdTimer) return;
      stopBtn.classList.add('holding');
      holdTimer = setTimeout(async () => {
        holdTimer = null; stopBtn.classList.remove('holding');
        const r = await window.api.emergencyStop(reason);
        if (!r.ok) fail(r.error || t('emg.stop.fail'));
      }, HOLD_MS);
    };

    el.addEventListener('click', async (e) => {
      const r = e.target.closest('[data-r]');
      if (r) {
        reason = r.dataset.r; fail('');
        el.querySelectorAll('[data-r]').forEach((b) => b.classList.toggle('sel', b === r));
        stopBtn.disabled = false;
        return;
      }
      const a = e.target.closest('[data-e]');
      if (!a) return;
      const act = a.dataset.e;
      if (act === 'open') { open = true; update(last); }
      if (act === 'close') { open = false; update(last); }
      if (act === 'pause') { const res = await window.api.emergencyPause(); fail(res.ok ? '' : res.error); }
      if (act === 'resume') { await window.api.emergencyResume(); }
    });
    stopBtn.addEventListener('pointerdown', startHold);
    ['pointerup', 'pointerleave', 'pointercancel', 'blur'].forEach((ev) => stopBtn.addEventListener(ev, cancelHold));
    stopBtn.addEventListener('keydown', (e) => { if ((e.key === ' ' || e.key === 'Enter') && !e.repeat) { e.preventDefault(); startHold(); } });
    stopBtn.addEventListener('keyup', (e) => { if (e.key === ' ' || e.key === 'Enter') cancelHold(); });

    function update(s) {
      last = s;
      if (!s || !s.active) return;
      if (s.paused) { show('paused'); $('.count').textContent = ExitFlow.fmt(s.pauseLeftMs); return; }
      document.body.classList.toggle('emg-open', open); // lớp khoá thu nhỏ phần trên cho vừa màn hình
      pauseBtn.disabled = !!s.pauseUsed;
      $('[data-n=pause]').textContent = s.pauseUsed ? t('emg.pause.used') : t('emg.pause.s');
      show(open ? 'panel' : 'idle');
    }
    return { update };
  }

  window.EmergencyFlow = { mount, REASONS, label };
})();
