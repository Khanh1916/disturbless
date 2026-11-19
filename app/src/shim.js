// Gives the web UI the same `window.api` the Electron preload gave it, on top of Tauri's IPC.
// Runs before the page's own scripts. The tokens below are filled in per window by main.rs.
(() => {
  const QUERY = '__QUERY__';
  if (QUERY && !location.search) history.replaceState(null, '', location.pathname + QUERY);

  const invoke = (channel, ...args) => window.__TAURI_INTERNALS__.invoke('ipc', { channel, args });
  const on = (channel) => (cb) => {
    let off = null, dead = false;
    window.__TAURI__.event.listen(channel, (e) => cb(e.payload)).then((f) => { if (dead) f(); else off = f; });
    return () => { dead = true; if (off) off(); };
  };

  Object.defineProperty(window, 'api', {
    value: Object.freeze({
      catImages: __CAT_IMAGES__,
      getState: () => invoke('state:get'),
      saveProfile: (p) => invoke('profile:save', p),
      deleteProfile: (id) => invoke('profile:delete', id),
      saveSettings: (s) => invoke('settings:save', s),
      listWindows: () => invoke('apps:list'),
      previewStart: (req) => invoke('session:preview', req),
      startSession: (req) => invoke('session:start', req),
      exitRequest: () => invoke('exit:request'),
      exitCancel: () => invoke('exit:cancel'),
      exitPhrase: () => invoke('exit:phrase'),
      exitConfirm: (text) => invoke('exit:confirm', text),
      cleanHosts: () => invoke('hosts:clean'),
      openMain: () => invoke('window:openMain'),
      testScold: (level) => invoke('scold:test', level),
      emergencyPause: () => invoke('emergency:pause'),
      emergencyResume: () => invoke('emergency:resume'),
      emergencyStop: (reason) => invoke('emergency:stop', reason),
      gateClose: (info) => invoke('gate:close', info),
      gateReason: (info) => invoke('gate:reason', info),
      gateDone: () => invoke('gate:done'),
      onGate: on('gate'),
      onScold: on('scold'),
      onCheer: on('cheer'),
      onSession: on('session'),
      onToast: on('toast'),
      onData: on('data'),
    }),
  });

  // Lock-screen and mini windows: swallow the keys that would reload, close or leave the page.
  if (__LOCK__) {
    addEventListener('keydown', (e) => {
      const k = String(e.key).toLowerCase();
      if ((e.ctrlKey && (k === 'w' || k === 'r')) || k === 'f11' || k === 'f5' || (e.altKey && k === 'f4')) e.preventDefault();
    }, true);
    addEventListener('contextmenu', (e) => e.preventDefault());
  }
})();
