'use strict';
// Cảnh mèo mắng / tát. Cấp 1: nhắc nhở · cấp 2: mắng + tát vừa · cấp 3: tát mạnh + rung màn hình.
(function () {
  const $ = (id) => document.getElementById(id);
  const stage = $('stage'), bubble = $('bubble'), hand = $('hand'), boom = $('boom'), flash = $('flash'), box = $('catbox');
  const cat = Cat.mount($('cat'), 'worried');
  const wait = (ms) => new Promise((r) => setTimeout(r, ms));

  function slap(strong, second) {
    hand.className = '';
    void hand.offsetWidth; // khởi động lại animation
    hand.className = second ? 'go2' : 'go';
    return wait(second ? 230 : 260).then(() => {
      cat.set('slapped');
      stage.className = strong ? 'shake3' : 'shake';
      void stage.offsetWidth;
      stage.classList.add(strong ? 'shake3' : 'shake');
      flash.className = strong ? 'f3' : 'f2';
      boom.textContent = strong ? (second ? t('scold.slap3') : t('scold.slap2')) : t('scold.slap');
      boom.className = '';
      void boom.offsetWidth;
      boom.className = 'hit';
    });
  }

  window.api.onScold(async (p) => {
    I18N.set(p.lang);
    const lv = Math.max(1, Math.min(3, p.level));
    document.body.className = '';
    stage.className = 'lv' + lv;
    stage.hidden = false;
    bubble.textContent = p.text;
    cat.set(lv === 1 ? 'worried' : 'angry');
    requestAnimationFrame(() => document.body.classList.add('on'));
    if (p.voiceOn) TTS.speak(p.text, p.persona.voice, p.lang);

    if (lv === 1) {
      await wait(900);
      box.classList.add('headshake');
    } else if (lv === 2) {
      await wait(1400);
      await slap(false, false);
    } else {
      await wait(1200);
      await slap(true, false);
      await wait(1000);
      cat.set('angry');
      await wait(300);
      await slap(true, true);
    }
  });
})();
