'use strict';
// Chú mèo cười (nhân vật gốc, vẽ bằng SVG). Biểu cảm đổi bằng thuộc tính data-mood:
//   happy | cheer | worried | angry | slapped
(function () {
  const INK = '#6b3f0e';
  const FUR = '#ffd25e';

  const markup = `
<svg class="cat" viewBox="0 0 200 210" data-mood="happy" aria-hidden="true">
  <g class="cat-wrap">
    <ellipse cx="100" cy="190" rx="56" ry="26" fill="${FUR}" stroke="${INK}" stroke-width="4"/>
    <path d="M70 182q-6 14 4 20M130 182q6 14-4 20" stroke="${INK}" stroke-width="4" fill="none" stroke-linecap="round"/>
    <g class="cat-head">
      <path d="M33 86 L38 26 L88 56 Z" fill="${FUR}" stroke="${INK}" stroke-width="4" stroke-linejoin="round"/>
      <path d="M46 72 L47 42 L74 58 Z" fill="#ffadb5"/>
      <path d="M167 86 L162 26 L112 56 Z" fill="${FUR}" stroke="${INK}" stroke-width="4" stroke-linejoin="round"/>
      <path d="M154 72 L153 42 L126 58 Z" fill="#ffadb5"/>
      <ellipse cx="100" cy="112" rx="78" ry="68" fill="${FUR}" stroke="${INK}" stroke-width="4"/>
      <path d="M100 48v15M85 51l3 13M115 51l-3 13" stroke="#e0a32e" stroke-width="5" stroke-linecap="round"/>
      <ellipse cx="100" cy="138" rx="36" ry="25" fill="#fff1c9"/>
      <ellipse class="blush" cx="50" cy="132" rx="13" ry="8" fill="#ff7f93" opacity=".55"/>
      <ellipse class="blush" cx="150" cy="132" rx="13" ry="8" fill="#ff7f93" opacity=".55"/>

      <g class="e e-open">
        <circle class="pupil" cx="72" cy="104" r="11" fill="#1d1208"/><circle cx="76" cy="100" r="3.6" fill="#fff"/>
        <circle class="pupil" cx="128" cy="104" r="11" fill="#1d1208"/><circle cx="132" cy="100" r="3.6" fill="#fff"/>
      </g>
      <g class="e e-cheer" fill="none" stroke="#1d1208" stroke-width="6" stroke-linecap="round">
        <path d="M60 108 Q72 90 84 108"/><path d="M116 108 Q128 90 140 108"/>
      </g>
      <g class="e e-worried">
        <circle cx="72" cy="106" r="13" fill="#1d1208"/><circle cx="77" cy="101" r="5" fill="#fff"/><circle cx="68" cy="111" r="2.4" fill="#fff"/>
        <circle cx="128" cy="106" r="13" fill="#1d1208"/><circle cx="133" cy="101" r="5" fill="#fff"/><circle cx="124" cy="111" r="2.4" fill="#fff"/>
      </g>
      <g class="e e-slapped" fill="none" stroke="#1d1208" stroke-width="6" stroke-linecap="round" stroke-linejoin="round">
        <path d="M60 94 L82 104 L60 114"/><path d="M140 94 L118 104 L140 114"/>
      </g>
      <g class="b b-worried" stroke="${INK}" stroke-width="5" stroke-linecap="round"><path d="M54 88 L84 76"/><path d="M146 88 L116 76"/></g>
      <g class="b b-angry" stroke="${INK}" stroke-width="6" stroke-linecap="round"><path d="M52 76 L88 92"/><path d="M148 76 L112 92"/></g>

      <path d="M93 122 h14 l-7 9 z" fill="#ff6f83" stroke="${INK}" stroke-width="2.5" stroke-linejoin="round"/>

      <g class="m m-happy">
        <path d="M64 134 Q100 190 136 134 Q100 146 64 134 Z" fill="#7a1f2b" stroke="${INK}" stroke-width="4" stroke-linejoin="round"/>
        <path d="M84 158 Q100 146 116 158 Q100 172 84 158 Z" fill="#ff7f93"/>
      </g>
      <path class="m m-worried" d="M78 150 q6-9 11 0 t11 0 t11 0 t11 0" fill="none" stroke="${INK}" stroke-width="4.5" stroke-linecap="round"/>
      <path class="m m-angry" d="M76 156 Q100 136 124 156" fill="none" stroke="${INK}" stroke-width="5" stroke-linecap="round"/>
      <g class="m m-slapped">
        <path d="M82 140 q18-10 36 0 q3 20-18 24 q-21-4-18-24z" fill="#7a1f2b" stroke="${INK}" stroke-width="4" stroke-linejoin="round"/>
        <path d="M88 156 q12-8 24 0" fill="#ff7f93" stroke="none"/>
      </g>

      <g stroke="${INK}" stroke-width="2.6" stroke-linecap="round" opacity=".85">
        <path d="M20 126 L52 132M18 142 L50 140M180 126 L148 132M182 142 L150 140"/>
      </g>

      <path class="sweat" d="M162 62 q10 16 0 24 q-10-8 0-24z" fill="#7ad7ff" stroke="#3aa0d6" stroke-width="2"/>
      <g class="mark" fill="#e5334b" opacity=".62">
        <ellipse cx="146" cy="128" rx="17" ry="20"/>
        <ellipse cx="132" cy="102" rx="5" ry="12" transform="rotate(-12 132 102)"/><ellipse cx="143" cy="98" rx="5" ry="14"/>
        <ellipse cx="155" cy="101" rx="5" ry="13" transform="rotate(10 155 101)"/><ellipse cx="165" cy="110" rx="4.5" ry="10" transform="rotate(22 165 110)"/>
      </g>
    </g>
    <g class="stars" fill="#ffe14d" stroke="#e0a32e" stroke-width="2" stroke-linejoin="round">
      <path d="M52 28l4 9 10 1-8 6 3 10-9-6-9 6 3-10-8-6 10-1z"/>
      <path d="M100 10l4 9 10 1-8 6 3 10-9-6-9 6 3-10-8-6 10-1z"/>
      <path d="M148 28l4 9 10 1-8 6 3 10-9-6-9 6 3-10-8-6 10-1z"/>
    </g>
    <g class="sparkles" fill="#fff6b0" stroke="#e0a32e" stroke-width="1.5">
      <path d="M20 50l3 8 8 3-8 3-3 8-3-8-8-3 8-3z"/><path d="M180 38l3 8 8 3-8 3-3 8-3-8-8-3 8-3z"/><path d="M168 170l2 6 6 2-6 2-2 6-2-6-6-2 6-2z"/>
    </g>
  </g>
</svg>`;

  // Chế độ ảnh thật: nếu renderer/cat/ có ảnh (happy.* bắt buộc, các biểu cảm khác tuỳ chọn) thì dùng ảnh,
  // thiếu biểu cảm nào thì dùng ảnh happy; cảm xúc thể hiện bằng khung màu, chuyển động và dấu tay đỏ khi bị tát (không dùng emoji).
  function mountImage(el, mood, imgs) {
    el.innerHTML = `
      <div class="cat catimg" data-mood="happy"><div class="ci-wrap">
        <img class="ci" alt="" draggable="false">
        <svg class="fx fx-mark" viewBox="0 0 60 70"><g fill="#e5334b" opacity=".7">
          <ellipse cx="30" cy="46" rx="17" ry="20"/><ellipse cx="14" cy="22" rx="5" ry="12" transform="rotate(-14 14 22)"/>
          <ellipse cx="26" cy="16" rx="5" ry="14"/><ellipse cx="38" cy="18" rx="5" ry="13" transform="rotate(10 38 18)"/>
          <ellipse cx="49" cy="28" rx="4.5" ry="10" transform="rotate(24 49 28)"/></g></svg>
      </div></div>`;
    const root = el.firstElementChild, img = root.querySelector('img');
    const set = (m) => { root.dataset.mood = m; img.src = imgs[m] || imgs.happy; };
    set(mood || 'happy');
    return { svg: root, set, get mood() { return root.dataset.mood; } };
  }

  function mount(el, mood) {
    const imgs = (window.api && window.api.catImages) || {};
    if (imgs.happy) return mountImage(el, mood, imgs);
    el.innerHTML = markup;
    const svg = el.firstElementChild;
    svg.dataset.mood = mood || 'happy';
    return { svg, set(m) { svg.dataset.mood = m; }, get mood() { return svg.dataset.mood; } };
  }
  window.Cat = { mount, markup };
})();
