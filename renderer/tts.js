'use strict';
// Đọc lời mèo bằng giọng có sẵn trong Windows theo ngôn ngữ đang dùng (tiếng Việt hoặc tiếng Anh), nếu máy có cài.
(function () {
  let voices = [];
  const ok = 'speechSynthesis' in window;
  const load = () => { voices = ok ? speechSynthesis.getVoices() : []; };
  if (ok) { load(); speechSynthesis.addEventListener('voiceschanged', load); }

  const voiceFor = (lang) => voices.find((v) => new RegExp('^' + (lang === 'en' ? 'en' : 'vi'), 'i').test(v.lang)) || null;

  function speak(text, opts, lang) {
    const v = voiceFor(lang || (window.I18N && I18N.lang) || 'vi');
    if (!ok || !v) return false;
    const { pitch = 1, rate = 1 } = opts || {};
    speechSynthesis.cancel();
    const u = new SpeechSynthesisUtterance(text);
    u.voice = v; u.lang = v.lang; u.pitch = pitch; u.rate = rate;
    speechSynthesis.speak(u);
    return true;
  }
  window.TTS = { speak, hasVoice: (lang) => !!voiceFor(lang), stop: () => ok && speechSynthesis.cancel() };
})();
