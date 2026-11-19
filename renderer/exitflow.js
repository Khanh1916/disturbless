'use strict';
// Luồng thoát sớm gồm 3 bước: bấm "muốn thoát" → chờ cooldown → gõ lại đoạn văn (không dán được).
// Mọi kiểm tra thật nằm ở tiến trình chính; đây chỉ là giao diện.
(function () {
  const fmt = (ms) => {
    const s = Math.ceil(ms / 1000);
    return `${String(Math.floor(s / 60)).padStart(2, '0')}:${String(s % 60).padStart(2, '0')}`;
  };
  const norm = (s) => String(s || '').normalize('NFC').replace(/\s+/g, ' ').trim();

  function mount(el) {
    el.innerHTML = `
      <div class="exit">
        <div data-s="idle">
          <p class="hint">${t('exit.hint')}</p>
          <button class="btn ghost" data-a="request">${t('exit.btn')}</button>
        </div>
        <div data-s="cooldown" hidden>
          <p class="hint">${t('exit.wait', { t: '<span class="count">--:--</span>' })}</p>
          <button class="btn primary" data-a="cancel">${t('exit.back')}</button>
        </div>
        <div data-s="phrase" hidden>
          <p class="hint">${t('exit.type')}</p>
          <div class="quote"></div>
          <textarea spellcheck="false" autocomplete="off" placeholder="${t('exit.ph')}"></textarea>
          <p class="hint err" style="color:var(--danger)" hidden></p>
          <div class="row">
            <button class="btn primary" data-a="cancel">${t('exit.back2')}</button>
            <button class="btn danger" data-a="confirm" disabled>${t('exit.confirm')}</button>
          </div>
        </div>
      </div>`;
    const $ = (sel) => el.querySelector(sel);
    const stages = { idle: $('[data-s=idle]'), cooldown: $('[data-s=cooldown]'), phrase: $('[data-s=phrase]') };
    const ta = $('textarea'), quote = $('.quote'), err = $('.err'), confirmBtn = $('[data-a=confirm]');
    let phrase = null, fetching = false, stage = 'idle';

    const show = (name) => {
      stage = name;
      for (const [k, node] of Object.entries(stages)) node.hidden = k !== name;
    };
    const check = () => {
      const match = phrase !== null && norm(ta.value) === norm(phrase);
      confirmBtn.disabled = !match;
      ta.classList.toggle('ok', match);
    };
    ['paste', 'drop', 'contextmenu', 'cut', 'copy'].forEach((ev) => ta.addEventListener(ev, (e) => e.preventDefault()));
    ta.addEventListener('keydown', (e) => {
      if ((e.ctrlKey || e.metaKey) && ['v', 'V', 'Insert'].includes(e.key)) e.preventDefault();
      if (e.shiftKey && e.key === 'Insert') e.preventDefault();
    });
    ta.addEventListener('input', () => { err.hidden = true; check(); });

    el.addEventListener('click', async (e) => {
      const a = e.target.closest('[data-a]');
      if (!a) return;
      if (a.dataset.a === 'request') { await window.api.exitRequest(); }
      if (a.dataset.a === 'cancel') { await window.api.exitCancel(); phrase = null; ta.value = ''; show('idle'); check(); }
      if (a.dataset.a === 'confirm') {
        const r = await window.api.exitConfirm(ta.value);
        if (!r.ok) { err.textContent = t('exit.mismatch'); err.hidden = false; }
      }
    });

    return {
      update(s) {
        const ex = s && s.exit;
        if (!ex || !ex.requested) { phrase = null; fetching = false; if (stage !== 'idle') { ta.value = ''; show('idle'); check(); } return; }
        if (!ex.ready) { show('cooldown'); $('.count').textContent = fmt(ex.cooldownLeftMs); return; }
        if (phrase === null && !fetching) {
          fetching = true;
          window.api.exitPhrase().then((p) => { phrase = p; fetching = false; if (p) { quote.textContent = p; show('phrase'); ta.focus(); } });
        } else if (phrase !== null && stage !== 'phrase') show('phrase');
      },
    };
  }
  window.ExitFlow = { mount, fmt };
})();
