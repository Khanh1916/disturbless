'use strict';
(function () {
  const $ = (s, r = document) => r.querySelector(s);
  const $$ = (s, r = document) => [...r.querySelectorAll(s)];
  const esc = (s) => String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
  const lines = (s) => [...new Set(String(s || '').split(/[\n,;]+/).map((x) => x.trim()).filter(Boolean))];
  const ic = (name, cls) => Icon(name, cls);
  const fmtMin = (m) => (m >= 60 ? `${Math.floor(m / 60)} ${t('unit.h')}${m % 60 ? ` ${m % 60} ${t('unit.min')}` : ''}` : `${m} ${t('unit.min')}`);
  const DOWS = { vi: ['CN', 'T2', 'T3', 'T4', 'T5', 'T6', 'T7'], en: ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'] };
  const FONT_SCALES = [1, 1.15, 1.3];

  // ───────── dữ liệu hiển thị (chữ lấy từ i18n.js theo khoá) ─────────
  const GOALS = { study: { ic: 'graduation-cap' }, work: { ic: 'briefcase' }, read: { ic: 'book-open' } };
  const goalName = (id) => t('goal.' + id);
  const GROUPS = [
    { id: 'social', ic: 'smartphone', apps: [],
      titles: ['Facebook', 'Instagram', 'TikTok', 'Twitter', 'Threads', 'Reddit', 'Pinterest'],
      sites: ['facebook.com', 'instagram.com', 'tiktok.com', 'twitter.com', 'x.com', 'threads.net', 'reddit.com', 'pinterest.com'] },
    { id: 'video', ic: 'play-circle', apps: ['vlc', 'spotify'],
      titles: ['YouTube', 'Netflix', 'Twitch', 'Disney+', 'Vimeo', 'FPT Play'],
      sites: ['youtube.com', 'netflix.com', 'twitch.tv', 'disneyplus.com', 'vimeo.com', 'fptplay.vn'] },
    { id: 'game', ic: 'gamepad',
      titles: ['Steam', 'Roblox', 'Epic Games', 'Garena', 'Poki'],
      apps: ['steam', 'epicgameslauncher', 'leagueclient', 'riotclientservices', 'valorant', 'robloxplayerbeta', 'minecraft'],
      sites: ['store.steampowered.com', 'steamcommunity.com', 'epicgames.com', 'roblox.com', 'garena.vn', 'poki.com'] },
    { id: 'chat', ic: 'chat', apps: ['discord', 'telegram', 'zalo', 'whatsapp', 'messenger', 'skype'],
      titles: ['Discord', 'Messenger', 'WhatsApp', 'Telegram', 'Zalo'],
      sites: ['discord.com', 'messenger.com', 'web.whatsapp.com', 'web.telegram.org', 'chat.zalo.me'] },
    { id: 'shop', ic: 'bag', apps: [],
      titles: ['Shopee', 'Lazada', 'Tiki', 'Amazon'],
      sites: ['shopee.vn', 'lazada.vn', 'tiki.vn', 'amazon.com'] },
  ];
  const groupName = (g) => t('group.' + g.id);
  const LEVELS = { easy: { ic: 'level-1' }, normal: { ic: 'level-2' }, hard: { ic: 'level-3' } };
  const levelName = (id) => t('level.' + id);
  const QUICK = [
    { min: 25, ic: 'timer', brk: 'none' },
    { min: 50, ic: 'book-open', brk: 'none' },
    { min: 90, ic: 'zap', brk: 'short' },
  ];
  const INTRO_MOODS = ['cheer', 'worried', 'happy'];

  // ───────── trạng thái ─────────
  const content = $('#content');
  let D = null;
  let view = 'home';
  let W = null;            // trạng thái trình hướng dẫn
  let running = null;
  let introIdx = 0;
  let appsCache = null;    // null | 'loading' | [{name,label,icon}]

  // ───────── tiện ích giao diện ─────────
  function applyLook() {
    const s = (D && D.settings) || {};
    I18N.set(I18N.detect(s));
    const sc = FONT_SCALES.includes(Number(s.fontScale)) ? Number(s.fontScale) : 1;
    document.documentElement.style.zoom = String(sc);
  }
  function toast(text, kind) {
    const el = document.createElement('div');
    el.className = 'toast ' + (kind || '');
    el.textContent = text;
    $('#toasts').appendChild(el);
    setTimeout(() => el.remove(), 6500);
  }
  function modal(html) {
    const m = $('#modal');
    m.innerHTML = `<div class="modal" role="dialog" aria-modal="true">${html}</div>`;
    m.hidden = false;
    return { el: m, close() { m.hidden = true; m.innerHTML = ''; } };
  }
  const saveSettings = async (patch) => {
    D.settings = { ...D.settings, ...patch };
    const r = await window.api.saveSettings(D.settings);
    if (!r.ok) toast(r.error, 'warn');
  };
  const appIcon = (a) => (a.icon ? `<img src="${a.icon}" alt="">` : `<span class="ph">${ic('window')}</span>`);

  // ───────── kế hoạch từ trình hướng dẫn ─────────
  function newWizard(seed) {
    return Object.assign({
      step: 0, goal: 'study', minutes: 50, breakType: 'none', blockMode: 'block',
      groups: ['social', 'video', 'game', 'chat'], extraApps: [], allowedApps: [], extraSites: '', closeTabs: true,
      strictness: (D.settings && D.settings.strictness) || 'normal',
    }, seed || {});
  }
  const stepsOf = () => (W.goal === 'read' ? ['goal', 'time', 'level', 'review'] : ['goal', 'time', 'block', 'level', 'review']);

  function buildPlan() {
    const full = W.goal === 'read';
    const chunk = W.breakType === 'short' ? 25 : W.breakType === 'long' ? 50 : 0;
    const brk = W.breakType === 'short' ? 5 : W.breakType === 'long' ? 10 : 0;
    let workMin = W.minutes, breakMin = 0, rounds = 1;
    if (chunk && W.minutes >= chunk * 1.5) { rounds = splitRounds(W.minutes, chunk); workMin = Math.round(W.minutes / rounds); breakMin = brk; }
    const gs = GROUPS.filter((g) => W.groups.includes(g.id));
    const allow = !full && W.blockMode === 'allow';
    return {
      name: goalName(W.goal), goal: W.goal, mode: full ? 'fullscreen' : 'apps', workMin, breakMin, rounds,
      appMode: full ? 'off' : allow ? 'allow' : 'block',
      allowedApps: allow ? [...W.allowedApps] : [],
      blockedApps: full || allow ? [] : [...gs.flatMap((g) => g.apps), ...W.extraApps],
      blockedSites: full ? [] : [...gs.flatMap((g) => g.sites), ...lines(W.extraSites)],
      blockedTitles: full ? [] : gs.flatMap((g) => g.titles),
      closeTabs: !full && W.closeTabs,
      strictness: W.strictness,
    };
  }
  // Chia tổng thời gian học thành các vòng đều nhau, gần với độ dài vòng mong muốn nhất (ưu tiên chia hết).
  function splitRounds(total, chunk) {
    let best = null;
    for (let r = 1; r <= Math.ceil(total / (chunk * 0.6)); r++) {
      if (total % r) continue;
      const w = total / r;
      if (w < chunk * 0.5 || w > chunk * 1.6) continue;
      const score = Math.abs(w - chunk);
      if (!best || score < best.score) best = { r, score };
    }
    return best ? best.r : Math.max(1, Math.round(total / chunk));
  }
  const studyMinutes = (p) => Math.round(p.workMin * p.rounds);                     // thời gian học thuần
  const planMinutes = (p) => Math.round(p.workMin * p.rounds + p.breakMin * (p.rounds - 1)); // gồm cả giờ nghỉ
  const planBrief = (p) => `${GOALS[p.goal] ? goalName(p.goal) : p.name} · ${fmtMin(studyMinutes(p))}`;

  function wizardFromPlan(p) {
    const gs = GROUPS.filter((g) => g.sites.every((s) => p.blockedSites.includes(s)));
    const inGroups = (list, key) => gs.some((g) => g[key].includes(list));
    return newWizard({
      step: 0, goal: p.goal || 'study', minutes: studyMinutes(p),
      breakType: p.breakMin === 5 ? 'short' : p.breakMin === 10 ? 'long' : 'none',
      blockMode: p.appMode === 'allow' ? 'allow' : 'block', groups: gs.map((g) => g.id),
      extraApps: p.blockedApps.filter((a) => !inGroups(a, 'apps')), allowedApps: [...p.allowedApps],
      extraSites: p.blockedSites.filter((s) => !inGroups(s, 'sites')).join('\n'), strictness: p.strictness || 'normal',
      closeTabs: p.closeTabs !== false,
    });
  }

  function summaryItems(p) {
    const items = [];
    items.push({ ic: GOALS[p.goal].ic, t: `${goalName(p.goal)} ${fmtMin(studyMinutes(p))}`, s: p.rounds > 1
      ? t('sum.rounds', { rounds: p.rounds, each: fmtMin(p.workMin), brk: fmtMin(p.breakMin), total: fmtMin(planMinutes(p)) })
      : t('sum.once') });
    if (p.mode === 'fullscreen') items.push({ ic: 'monitor', t: t('sum.full'), s: t('sum.full.s') });
    else if (p.appMode === 'allow') items.push({ ic: 'check-circle', t: t('sum.allow', { n: p.allowedApps.length }), s: t('sum.allow.s') });
    else {
      const names = GROUPS.filter((g) => W.groups.includes(g.id)).map(groupName);
      items.push({ ic: 'ban', t: names.length ? t('sum.block', { names: names.join(', ') }) : t('sum.block.none'), s: `${t('sum.sites', { n: p.blockedSites.length })}${p.blockedApps.length ? t('sum.apps', { n: p.blockedApps.length }) : ''}` });
    }
    if (p.mode === 'apps') {
      items.push({ ic: 'window', t: p.closeTabs ? t('sum.tabs.on') : t('sum.tabs.off'), s: p.closeTabs ? t('sum.tabs.on.s') : t('sum.tabs.off.s') });
    }
    items.push({ ic: LEVELS[p.strictness].ic, t: t('sum.level', { name: levelName(p.strictness) }), s: t('sum.level.s', { brief: t('level.' + p.strictness + '.brief') }) });
    return items;
  }

  // ───────── điều hướng ─────────
  const TAB_ICONS = { home: 'play', stats: 'flame', settings: 'sliders' };
  function renderTabs() {
    const tabs = $('#tabs');
    if (view === 'running' || view === 'intro') { tabs.innerHTML = ''; return; }
    const cur = view === 'wizard' ? 'home' : view;
    tabs.innerHTML = ['home', 'stats', 'settings']
      .map((v) => `<button class="tab ${cur === v ? 'on' : ''}" data-v="${v}">${ic(TAB_ICONS[v])}${t('nav.' + v)}</button>`).join('');
  }
  $('#tabs').addEventListener('click', (e) => {
    const b = e.target.closest('[data-v]');
    if (!b) return;
    view = b.dataset.v; if (view === 'home') W = null;
    render();
  });
  Cat.mount($('#brandCat'), 'happy');

  function render() {
    applyLook();
    const s = D.session;
    if (s.active) {
      if (view !== 'running') { view = 'running'; running = null; }
      if (!running) mountRunning();
      updateRunning(s); renderTabs(); return;
    }
    if (view === 'running') { view = 'home'; running = null; }
    if (!D.settings.introSeen && view !== 'intro') { view = 'intro'; introIdx = 0; }
    ({ intro: renderIntro, home: renderHome, wizard: renderWizard, stats: renderStats, settings: renderSettings })[view]();
    renderTabs();
  }

  // ───────── giới thiệu lần đầu ─────────
  function renderIntro() {
    const n = introIdx + 1, last = introIdx === INTRO_MOODS.length - 1;
    content.innerHTML = `
      <div class="intro">
        <div class="pic" id="icat"></div>
        <div class="card">
          <h2>${esc(t(`intro.${n}.title`))}</h2><p>${esc(t(`intro.${n}.text`))}</p>
          <div class="dots">${INTRO_MOODS.map((_, i) => `<i class="${i < introIdx ? 'done' : i === introIdx ? 'on' : ''}"></i>`).join('')}</div>
          <div class="actions">
            ${last ? '' : `<button class="btn ghost" data-a="skip">${t('intro.skip')}</button>`}
            <button class="btn primary big" data-a="next">${last ? t('intro.start') : `${t('intro.next')} ${ic('arrow-right')}`}</button>
          </div>
        </div>
      </div>`;
    Cat.mount($('#icat'), INTRO_MOODS[introIdx]);
    const done = async () => { await saveSettings({ introSeen: true }); view = 'home'; render(); };
    $('[data-a=next]').addEventListener('click', () => { if (last) done(); else { introIdx++; renderIntro(); } });
    const sk = $('[data-a=skip]');
    if (sk) sk.addEventListener('click', done);
  }

  // ───────── trang chủ ─────────
  function renderHome() {
    const st = D.stats, last = D.lastPlan, pers = D.personas[0];
    content.innerHTML = `
      ${D.staleHosts ? `<div class="banner"><span>${t('home.stale')}</span><button class="btn" id="cleanHosts">${t('home.stale.btn')}</button></div>` : ''}
      <section class="home">
        <div class="home-cat" id="hcat"></div>
        <div class="home-text">
          <div class="speech">${esc(pers.start[0])}</div>
          <h1>${t('home.h1')}</h1>
          <p class="lead">${t('home.lead')}</p>
          <div class="quick">
            ${QUICK.map((q, i) => `<button class="qcard" data-q="${i}">${ic(q.ic)}<b>${fmtMin(q.min)}</b><small>${t('quick.' + q.min)}</small></button>`).join('')}
          </div>
          <div class="home-more">
            <button class="btn" id="goWiz">${ic('sliders')} ${t('home.custom')}</button>
            ${last ? `<button class="btn" id="goLast">${ic('history')} ${t('home.last')} · ${esc(planBrief(last))}</button>` : ''}
          </div>
          <div class="chips">
            <span class="chip">${ic('flame')} ${t('home.streak', { n: st.streak })}</span>
            <span class="chip">${ic('timer')} ${t('home.today', { v: st.todayMin ? fmtMin(st.todayMin) : t('home.today.none') })}</span>
          </div>
        </div>
      </section>`;
    Cat.mount($('#hcat'), 'cheer');
    $('#goWiz').addEventListener('click', () => { W = newWizard(); view = 'wizard'; render(); });
    $$('[data-q]').forEach((b) => b.addEventListener('click', () => quickStart(QUICK[Number(b.dataset.q)])));
    const gl = $('#goLast');
    if (gl) gl.addEventListener('click', () => { W = wizardFromPlan(last); W.step = stepsOf().length - 1; view = 'wizard'; render(); });
    const ch = $('#cleanHosts');
    if (ch) ch.addEventListener('click', async () => { ch.disabled = true; await window.api.cleanHosts(); });
  }

  // Bắt đầu nhanh: lấy cách chặn và mức nghiêm khắc của lần trước (trừ chế độ khoá màn hình), chỉ đổi thời gian.
  function quickStart(q) {
    const last = D.lastPlan;
    const base = last && last.goal !== 'read' ? wizardFromPlan(last) : newWizard();
    W = Object.assign(base, { goal: base.goal === 'read' ? 'study' : base.goal, minutes: q.min, breakType: q.brk, step: 0 });
    confirmStart(null);
  }

  // ───────── trình hướng dẫn ─────────
  async function loadApps() {
    if (appsCache === 'loading' || Array.isArray(appsCache)) return;
    appsCache = 'loading';
    renderWizard();
    appsCache = await window.api.listWindows();
    if (view === 'wizard') renderWizard();
  }
  const choice = (k, v, icon, name, sub, sel) =>
    `<button class="choice ${sel ? 'sel' : ''}" data-k="${k}" data-v="${esc(v)}"><span class="em">${ic(icon)}</span><b>${esc(name)}</b>${sub ? `<small>${esc(sub)}</small>` : ''}<span class="tick">${ic('check')}</span></button>`;

  function stepGoal() {
    return { mood: 'happy', html: `
      <h2 class="q">${t('wiz.goal.q')}</h2><p class="qs">${t('wiz.goal.s')}</p>
      <div class="choices">${Object.entries(GOALS).map(([id, g]) => choice('goal', id, g.ic, goalName(id), t(`goal.${id}.sub`), W.goal === id)).join('')}</div>` };
  }
  function stepTime() {
    const presets = [[25, 'timer', fmtMin(25), t('quick.25')], [50, 'book-open', fmtMin(50), t('quick.50')], [90, 'zap', fmtMin(90), t('wiz.time.90.sub')], [120, 'hourglass', t('wiz.time.2h'), t('wiz.time.2h.sub')]];
    const isPreset = presets.some((p) => p[0] === W.minutes);
    const p = buildPlan();
    const note = p.rounds > 1 ? t('wiz.rest.note.rounds', { min: fmtMin(W.minutes), rounds: p.rounds, each: fmtMin(p.workMin), total: fmtMin(planMinutes(p)) }) : (W.breakType !== 'none' ? t('wiz.rest.note.short') : '');
    return { mood: 'cheer', ok: W.minutes >= 1 && W.minutes <= 480, html: `
      <h2 class="q">${t('wiz.time.q')}</h2><p class="qs">${t('wiz.time.s')}</p>
      <div class="choices">${presets.map(([m, i, n, s]) => choice('minutes', m, i, n, s, W.minutes === m)).join('')}</div>
      <label class="custom">${t('wiz.time.other')} <input class="num" id="customMin" type="number" min="1" max="480" value="${isPreset ? '' : W.minutes}" placeholder="45"> ${t('wiz.time.min')}</label>
      <span class="sub-q">${t('wiz.rest.q')}</span>
      <div class="choices">
        ${choice('breakType', 'none', 'zap', t('wiz.rest.none'), t('wiz.rest.none.sub'), W.breakType === 'none')}
        ${choice('breakType', 'short', 'coffee', t('wiz.rest.short'), t('wiz.rest.short.sub'), W.breakType === 'short')}
        ${choice('breakType', 'long', 'moon', t('wiz.rest.long'), t('wiz.rest.long.sub'), W.breakType === 'long')}
      </div>
      <p class="qs" style="margin:14px 0 0">${esc(note || t('wiz.rest.note.default'))}</p>` };
  }
  function appsGrid(selected, attr) {
    if (appsCache === 'loading') return `<p class="muted">${t('apps.loading')}</p>`;
    if (!Array.isArray(appsCache)) return '';
    if (!appsCache.length) return `<p class="muted">${t('apps.none')}</p>`;
    return `<div class="apps">${appsCache.map((a) => `<button class="app-chip ${selected.includes(a.name) ? 'sel' : ''}" data-${attr}="${esc(a.name)}" title="${esc(a.title)}">${appIcon(a)}<span class="nm">${esc(a.label)}</span><span class="tick">${ic('check')}</span></button>`).join('')}</div>`;
  }
  function stepBlock() {
    const allow = W.blockMode === 'allow';
    if (allow && appsCache === null) setTimeout(loadApps, 0);
    return { mood: 'angry', ok: !allow || W.allowedApps.length > 0, html: `
      <h2 class="q">${t('wiz.block.q')}</h2><p class="qs">${t('wiz.block.s')}</p>
      <div class="seg">
        <button class="${!allow ? 'on' : ''}" data-k="blockMode" data-v="block">${ic('ban')} ${t('wiz.block.mode.block')}</button>
        <button class="${allow ? 'on' : ''}" data-k="blockMode" data-v="allow">${ic('check-circle')} ${t('wiz.block.mode.allow')}</button>
      </div>
      <p class="seg-note">${allow ? t('wiz.block.note.allow') : t('wiz.block.note.block')}</p>
      ${allow ? `<span class="sub-q" style="margin-top:6px">${t('wiz.block.allow.q')}</span>
        <p class="qs" style="margin-bottom:6px">${t('wiz.block.allow.s')}</p>
        <div class="actions"><button class="btn" data-a="pickallow">${ic('refresh')} ${t('wiz.block.rescan')}</button></div>${appsGrid(W.allowedApps, 'allow')}` : ''}
      <span class="sub-q">${allow ? t('wiz.block.groups.allow') : t('wiz.block.groups')}</span>
      <div class="grid2">${GROUPS.map((g) => `
        <label class="optrow ${W.groups.includes(g.id) ? 'sel' : ''}" data-g="${g.id}"><span class="em">${ic(g.ic)}</span>
          <span class="txt"><b>${esc(groupName(g))}</b><small>${esc(t('group.' + g.id + '.eg'))}</small></span>
          <span class="sw"><input type="checkbox" ${W.groups.includes(g.id) ? 'checked' : ''}><span></span></span></label>`).join('')}</div>
      <div class="stack" style="margin-top:12px">
        <label class="optrow ${W.closeTabs ? 'sel' : ''}" data-ct="1"><span class="em">${ic('window')}</span>
          <span class="txt"><b>${t('wiz.block.tabs')}</b><small>${t('wiz.block.tabs.s')}</small></span>
          <span class="sw"><input type="checkbox" ${W.closeTabs ? 'checked' : ''}><span></span></span></label>
      </div>
      <details><summary>${ic('chevron-down')} ${t('wiz.block.more')}</summary><div class="inner">
        ${allow ? '' : `<div class="actions"><button class="btn" data-a="pickblock">${ic('plus')} ${t('wiz.block.addapp')}</button></div>${appsGrid(W.extraApps, 'extra')}`}
        <textarea class="f" id="extraSites" style="margin-top:12px" placeholder="${esc(t('wiz.block.sites'))}">${esc(W.extraSites)}</textarea>
      </div></details>` };
  }
  function stepLevel() {
    return { mood: 'worried', html: `
      <h2 class="q">${t('wiz.level.q')}</h2><p class="qs">${t('wiz.level.s')}</p>
      <div class="choices">${Object.entries(LEVELS).map(([id, l]) => choice('strictness', id, l.ic, levelName(id), t(`level.${id}.desc`), W.strictness === id)).join('')}</div>` };
  }
  function stepReview() {
    const p = buildPlan();
    return { mood: 'cheer', html: `
      <h2 class="q">${t('wiz.review.q')}</h2><p class="qs">${t('wiz.review.s')}</p>
      <ul class="sum">${summaryItems(p).map((i) => `<li><span class="em">${ic(i.ic)}</span><span><b>${esc(i.t)}</b><small>${esc(i.s)}</small></span></li>`).join('')}</ul>` };
  }

  function renderWizard() {
    const steps = stepsOf();
    if (W.step >= steps.length) W.step = steps.length - 1;
    const key = steps[W.step];
    const body = { goal: stepGoal, time: stepTime, block: stepBlock, level: stepLevel, review: stepReview }[key]();
    content.innerHTML = `
      <div class="wiz">
        <aside class="wiz-cat"><div class="speech" id="wsay"></div><div class="pic" id="wcat"></div></aside>
        <section class="wiz-card">
          <div class="dots">${steps.map((_, i) => `<i class="${i < W.step ? 'done' : i === W.step ? 'on' : ''}"></i>`).join('')}<span>${t('wiz.step', { n: W.step + 1, total: steps.length })}</span></div>
          ${body.html}
          <div class="wiz-nav">
            <button class="btn ghost" data-nav="back">${ic('arrow-left')} ${W.step === 0 ? t('wiz.home') : t('wiz.back')}</button>
            ${key === 'review'
              ? `<button class="btn primary big" data-nav="start">${ic('play')} ${t('wiz.start')}</button>`
              : `<button class="btn primary" data-nav="next" ${body.ok === false ? 'disabled' : ''}>${t('wiz.next')} ${ic('arrow-right')}</button>`}
          </div>
        </section>
      </div>`;
    $('#wsay').textContent = t('say.' + key);
    Cat.mount($('#wcat'), body.mood || 'happy');
    bindWizard(key);
  }

  function toggle(list, v) { const i = list.indexOf(v); if (i >= 0) list.splice(i, 1); else list.push(v); }

  function bindWizard(key) {
    $$('[data-k]').forEach((b) => b.addEventListener('click', () => {
      const k = b.dataset.k;
      W[k] = k === 'minutes' ? Number(b.dataset.v) : b.dataset.v;
      renderWizard();
    }));
    const cm = $('#customMin');
    if (cm) cm.addEventListener('input', () => {
      W.minutes = Number(cm.value) || 0;
      $$('[data-k=minutes]').forEach((b) => b.classList.toggle('sel', Number(b.dataset.v) === W.minutes));
      $('[data-nav=next]').disabled = !(W.minutes >= 1 && W.minutes <= 480);
    });
    $$('[data-g] input').forEach((inp) => inp.addEventListener('change', () => {
      const row = inp.closest('[data-g]');
      toggle(W.groups, row.dataset.g);
      row.classList.toggle('sel', inp.checked);
    }));
    $$('[data-ct] input').forEach((inp) => inp.addEventListener('change', () => {
      W.closeTabs = inp.checked;
      inp.closest('[data-ct]').classList.toggle('sel', inp.checked);
    }));
    const es = $('#extraSites');
    if (es) es.addEventListener('input', () => { W.extraSites = es.value; });
    $$('[data-allow]').forEach((b) => b.addEventListener('click', () => {
      toggle(W.allowedApps, b.dataset.allow); b.classList.toggle('sel');
      $('[data-nav=next]').disabled = W.blockMode === 'allow' && !W.allowedApps.length;
    }));
    $$('[data-extra]').forEach((b) => b.addEventListener('click', () => { toggle(W.extraApps, b.dataset.extra); b.classList.toggle('sel'); }));
    $$('[data-a=pickblock],[data-a=pickallow]').forEach((b) => b.addEventListener('click', () => { appsCache = null; loadApps(); }));

    $('[data-nav=back]').addEventListener('click', () => { if (W.step === 0) { W = null; view = 'home'; } else W.step--; render(); });
    const nx = $('[data-nav=next]');
    if (nx) nx.addEventListener('click', () => { W.step++; render(); });
    const st = $('[data-nav=start]');
    if (st) st.addEventListener('click', () => confirmStart(st));
  }

  // Hộp xác nhận cuối: cho biết app nào sẽ bị đóng, hỏi mục tiêu của phiên, rồi mới bắt đầu.
  async function confirmStart(btn) {
    const plan = buildPlan();
    if (btn) { btn.disabled = true; btn.textContent = t('confirm.checking'); }
    const prev = await window.api.previewStart({ plan });
    if (btn) { btn.disabled = false; btn.innerHTML = `${ic('play')} ${t('wiz.start')}`; }
    if (!prev.ok) return toast(prev.error, 'warn');
    const m = modal(`
      <h2>${ic('target')} ${t('confirm.title')}</h2>
      <p>${t('confirm.body', { brief: esc(planBrief(plan)) })}</p>
      <label class="lbl" for="mIntent">${t('confirm.intent')} <span class="muted">${t('confirm.optional')}</span></label>
      <input class="field" id="mIntent" maxlength="80" placeholder="${esc(t('confirm.intent.ph'))}" autocomplete="off">
      <p class="muted" style="margin:6px 0 0;font-size:14.5px">${t('confirm.intent.hint')}</p>
      ${prev.toClose.length ? `<p style="margin-top:16px"><b>${t('confirm.close')}</b></p>
        <ul>${prev.toClose.map((a) => `<li>${appIcon(a)} ${esc(a.label)}</li>`).join('')}</ul>
        <p class="muted">${t('confirm.save')}</p>` : ''}
      ${prev.sites ? `<p class="muted">${t('confirm.uac')}</p>` : ''}
      <div class="actions"><button class="btn ghost" id="mNo">${t('confirm.no')}</button><button class="btn primary big" id="mYes">${ic('play')} ${t('confirm.yes')}</button></div>`);
    const input = $('#mIntent', m.el);
    input.focus();
    $('#mNo', m.el).addEventListener('click', m.close);
    const go = async () => {
      const intent = input.value.trim();
      m.close();
      const r = await window.api.startSession({ plan: intent ? { ...plan, intent } : plan });
      if (!r.ok) toast(r.error, 'warn');
    };
    $('#mYes', m.el).addEventListener('click', go);
    input.addEventListener('keydown', (e) => { if (e.key === 'Enter') go(); });
  }

  // ───────── đang chạy ─────────
  function mountRunning() {
    content.innerHTML = `
      <div class="run">
        <div><div class="pic" id="rcat"></div><p class="who" id="rwho"></p></div>
        <div>
          <div class="ring">
            <svg width="260" height="260" viewBox="0 0 270 270">
              <circle cx="135" cy="135" r="120" fill="none" stroke="var(--line)" stroke-width="14"/>
              <circle id="rarc" cx="135" cy="135" r="120" fill="none" stroke="var(--accent)" stroke-width="14" stroke-linecap="round" stroke-dasharray="754" stroke-dashoffset="754"/>
            </svg>
            <div class="tm"><div><b id="rtime">--:--</b><span id="rphase"></span></div></div>
          </div>
          <div class="intent" id="rintent"></div>
          <div class="rounds" id="rrounds"></div>
          <div class="facts" id="rfacts"></div>
          <div id="rexit"></div>
          <div id="remg"></div>
        </div>
      </div>`;
    const pers = D.personas[0];
    running = { cat: Cat.mount($('#rcat'), 'happy'), exit: ExitFlow.mount($('#rexit')), emg: EmergencyFlow.mount($('#remg')) };
    $('#rwho').innerHTML = `${ic('smile')} ${esc(t('run.who', { name: pers.name }))}`;
  }
  function updateRunning(s) {
    if (!running || !s.active) return;
    $('#rtime').textContent = ExitFlow.fmt(s.phaseRemainingMs);
    const g = GOALS[s.goal];
    $('#rphase').innerHTML = s.paused ? `${ic('pause')} ${t('run.paused')}` : s.phase === 'work' ? `${ic(g ? g.ic : 'target')} ${esc(g ? goalName(s.goal) : t('run.focus'))}` : `${ic('coffee')} ${t('run.break')}`;
    $('#rintent').innerHTML = s.intent ? `${ic('target')} <span>${esc(s.intent)}</span>` : '';
    $('#rarc').style.strokeDashoffset = String(754 * (1 - (s.totalMs ? s.elapsedMs / s.totalMs : 0)));
    $('#rrounds').innerHTML = s.rounds > 1 ? Array.from({ length: s.rounds }, (_, i) =>
      `<i class="${i + 1 < s.round || (i + 1 === s.round && s.phase === 'break') ? 'done' : i + 1 === s.round ? 'cur' : ''}"></i>`).join('') : '';
    const facts = [`<span class="chip">${ic('hourglass')} ${t('run.left', { t: ExitFlow.fmt(s.totalRemainingMs) })}</span>`];
    if (s.hosts && s.hosts.state === 'active') facts.push(`<span class="chip ok">${ic('shield')} ${esc(s.hosts.message)}</span>`);
    else if (s.hosts && s.hosts.state === 'pending') facts.push(`<span class="chip warn">${esc(s.hosts.message)}</span>`);
    else if (s.hosts && s.hosts.state === 'failed') facts.push(`<span class="chip bad">${esc(s.hosts.message)}</span>`);
    if (s.closeTabs) facts.push(`<span class="chip">${ic('window')} ${t('run.tabs')}</span>`);
    if (s.violations) facts.push(`<span class="chip warn">${ic('alert')} ${t('run.viol', { n: s.violations })}</span>`);
    if (s.restarts) facts.push(`<span class="chip bad">${ic('alert')} ${t('run.restarts', { n: s.restarts })}</span>`);
    $('#rfacts').innerHTML = facts.join('');
    running.cat.set(s.exit && s.exit.requested ? 'worried' : s.phase === 'break' || s.paused ? 'cheer' : 'happy');
    running.exit.update(s);
    running.emg.update(s);
  }

  // ───────── thống kê ─────────
  // Gom các lần bị kéo đi (app/tab bị chặn) trong 100 phiên gần nhất: app hay mở nhất và lý do hay gặp nhất.
  function pullStats() {
    const apps = new Map(), why = new Map();
    for (const h of D.history) for (const d of h.distractions || []) {
      apps.set(d.name, (apps.get(d.name) || 0) + 1);
      if (d.reason) why.set(d.reason, (why.get(d.reason) || 0) + 1);
    }
    const top = (m) => [...m.entries()].sort((a, b) => b[1] - a[1]).slice(0, 3);
    return { apps: top(apps), why: top(why) };
  }
  function renderStats() {
    const s = D.stats;
    const max = Math.max(60, ...s.days.map((d) => d.min));
    const dow = DOWS[I18N.lang];
    const pull = pullStats();
    const kpi = (icon, big, label) => `<div class="card kpi"><span class="em">${ic(icon)}</span><b>${big}</b><span class="l">${label}</span></div>`;
    content.innerHTML = `
      <h1 style="font-size:34px">${t('stats.title')}</h1>
      <p class="lead">${t('stats.lead')}</p>
      <div class="kpis">
        ${kpi('flame', t('days', { n: s.streak }), t('stats.streak', { n: s.bestStreak }))}
        ${kpi('timer', s.todayMin ? fmtMin(s.todayMin) : '0', t('stats.today'))}
        ${kpi('calendar', fmtMin(s.weekMin), t('stats.week'))}
        ${kpi('award', `${s.completionRate}%`, t('stats.rate', { a: s.completed, b: s.sessions }))}
      </div>
      ${s.emergencies ? `<p class="muted" style="margin:-6px 0 18px">${ic('life-buoy')} ${t('stats.emg', { n: s.emergencies })}</p>` : ''}
      <div class="card" style="margin-bottom:22px"><h2>${t('stats.chart')}</h2>
        <div class="bars">${s.days.map((d) => `<div title="${d.min} ${t('unit.min')}"><span>${d.min ? fmtMin(d.min).replace(` ${t('unit.h')}`, 'h').replace(` ${t('unit.min')}`, 'm') : ''}</span><i style="height:${Math.max(3, (d.min / max) * 100)}%"></i><span>${dow[d.dow]}</span></div>`).join('')}</div></div>
      <div class="card" style="margin-bottom:22px"><h2>${ic('target')} ${t('stats.pull')}</h2>
        ${pull.apps.length ? `<div class="pull">
          <div><small>${t('stats.pull.apps')}</small>${pull.apps.map(([n, c]) => `<span class="chip">${esc(n)} · ${t('stats.pull.times', { n: c })}</span>`).join('')}</div>
          ${pull.why.length ? `<div><small>${t('stats.pull.why')}</small>${pull.why.map(([r, c]) => `<span class="chip">${esc(t('gate.r.' + r))} · ${t('stats.pull.times', { n: c })}</span>`).join('')}</div>` : ''}
        </div>` : `<p class="muted">${t('stats.pull.empty')}</p>`}
      </div>
      <div class="card"><h2>${t('stats.recent')}</h2>
        ${D.history.length ? `<div class="log">${D.history.slice(0, 20).map((h) => `
          <div class="item"><span class="em ${h.emergency ? 'warn' : h.completed ? 'ok' : 'bad'}">${ic(h.emergency ? 'life-buoy' : h.completed ? 'check-circle' : 'x-circle')}</span>
            <span class="t"><b>${esc(h.intent || h.profileName)}</b><small>${esc(h.intent ? h.profileName + ' · ' : '')}${new Date(h.endedAt).toLocaleString(I18N.locale(), { day: '2-digit', month: '2-digit', hour: '2-digit', minute: '2-digit' })} · ${h.emergency ? t('stats.reason', { r: esc(EmergencyFlow.label(h.reason)) }) : t('stats.focused', { a: fmtMin(h.focusedMin), b: fmtMin(h.plannedMin) })}${h.violations && !h.emergency ? ` · ${t('stats.viol', { n: h.violations })}` : ''}${h.pauses ? ` · ${t('stats.paused')}` : ''}</small></span>
            <span class="chip ${h.emergency ? 'warn' : h.completed ? 'ok' : 'bad'}">${h.emergency ? t('stats.tag.emg') : h.completed ? t('stats.tag.done') : t('stats.tag.quit')}</span></div>`).join('')}</div>`
          : `<p class="muted">${t('stats.empty')}</p>`}
      </div>`;
  }

  // ───────── cài đặt ─────────
  function renderSettings() {
    const st = D.settings, hasVoice = TTS.hasVoice(I18N.lang);
    const sel = (id, opts, cur) => `<select id="${id}">${opts.map(([v, l]) => `<option value="${v}" ${String(cur) === String(v) ? 'selected' : ''}>${l}</option>`).join('')}</select>`;
    content.innerHTML = `
      <h1 style="font-size:34px">${t('set.title')}</h1>
      <p class="lead">${t('set.lead')}</p>
      <div class="card sect"><h2>${ic('smile')} ${t('set.cat')}</h2>
        <div class="setrow"><div>${t('set.voice')}<p>${hasVoice ? t('set.voice.yes') : t('set.voice.no')}</p></div>
          <label class="sw"><input type="checkbox" id="sVoice" ${st.voice ? 'checked' : ''}><span></span></label></div>
        <div class="setrow"><div>${t('set.cheer')}<p>${t('set.cheer.s')}</p></div>
          ${sel('sCheer', [0, 60, 120, 240, 600].map((v) => [v, t('set.cheer.' + v)]), st.cheerEverySec)}</div>
        <div class="setrow"><div>${t('set.scold')}<p>${t('set.scold.s')}</p></div>
          <div class="actions"><button class="btn" data-t="1">${t('set.scold.1')}</button><button class="btn" data-t="2">${t('set.scold.2')}</button><button class="btn danger" data-t="3">${t('set.scold.3')}</button></div></div>
      </div>
      <div class="card sect"><h2>${ic('lock')} ${t('set.level')}</h2>
        <div class="choices" id="lvls">${Object.entries(LEVELS).map(([id, l]) => choice('lv', id, l.ic, levelName(id), t(`level.${id}.desc`), st.strictness === id)).join('')}</div></div>
      <div class="card sect"><h2>${ic('laptop')} ${t('set.general')}</h2>
        <div class="setrow"><div>${t('set.lang')}<p>${t('set.lang.s')}</p></div>
          ${sel('sLang', [['vi', 'Tiếng Việt'], ['en', 'English']], I18N.lang)}</div>
        <div class="setrow"><div>${t('set.font')}<p>${t('set.font.s')}</p></div>
          ${sel('sFont', FONT_SCALES.map((v) => [v, t('set.font.' + v)]), Number(st.fontScale) || 1)}</div>
        <div class="setrow"><div>${t('set.auto')}<p>${t('set.auto.s')}</p></div>
          <label class="sw"><input type="checkbox" id="sAuto" ${st.autostart ? 'checked' : ''}><span></span></label></div>
        <div class="setrow"><div>${t('set.intro')}<p>${t('set.intro.s')}</p></div>
          <button class="btn" id="sIntro">${t('set.intro.btn')}</button></div></div>`;
    $('#sVoice').addEventListener('change', (e) => saveSettings({ voice: e.target.checked }));
    $('#sCheer').addEventListener('change', (e) => saveSettings({ cheerEverySec: Number(e.target.value) }));
    $('#sAuto').addEventListener('change', (e) => saveSettings({ autostart: e.target.checked }));
    $('#sFont').addEventListener('change', async (e) => { await saveSettings({ fontScale: Number(e.target.value) }); render(); });
    $('#sLang').addEventListener('change', async (e) => { await saveSettings({ lang: e.target.value }); render(); });
    $$('[data-t]').forEach((b) => b.addEventListener('click', () => window.api.testScold(Number(b.dataset.t))));
    $$('#lvls [data-k=lv]').forEach((b) => b.addEventListener('click', () => {
      $$('#lvls .choice').forEach((x) => x.classList.toggle('sel', x === b));
      saveSettings({ strictness: b.dataset.v });
    }));
    $('#sIntro').addEventListener('click', async () => { await saveSettings({ introSeen: false }); view = 'intro'; introIdx = 0; render(); });
  }

  // ───────── kết nối với tiến trình chính ─────────
  window.api.onToast((m) => toast(m.text, m.kind));
  window.api.onData((d) => {
    const was = D && D.session.active;
    D = d;
    if (d.session.active || was || view === 'home' || view === 'stats') render();
  });
  window.api.onSession((s) => {
    if (!D) return;
    D.session = s;
    if (s.active) { if (!running) render(); else updateRunning(s); }
  });
  window.api.getState().then(async (d) => {
    D = d;
    // lần đầu chưa có lựa chọn ngôn ngữ: ghi lại ngôn ngữ đã nhận diện để phía Rust nói cùng thứ tiếng
    if (!d.settings.lang && !d.session.active) {
      D.settings.lang = I18N.detect(d.settings);
      await window.api.saveSettings(D.settings);
    }
    render();
  });
})();
