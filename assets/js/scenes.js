/* Theme-matrix card art: one scene per family, painted in the card's own palette.
 *
 *   synthwave       outrun sun over a perspective grid
 *   cyberdyne       targeting reticle, readouts, scan band
 *   cyberpunk       rain-soaked neon skyline
 *   neosynth        oscilloscope, spectrum bars, laser fan
 *   dystopian       ruins in smog, hazard stripes
 *   default         code editor (the classic editor themes)
 *   cybercore-tech  circuit traces into the hex core
 *   mashup          base family's scene × accent family's scene, spliced diagonally
 *
 * Each scene is seeded from the theme name, so cards in one family share a
 * motif but never look identical. All ids are prefixed per card.
 */
window.CC_SCENE = (() => {
  const W = 160, H = 80;
  const seedOf = str => { let h = 2166136261; for (const c of str) h = Math.imul(h ^ c.charCodeAt(0), 16777619); return h >>> 0; };
  const rng = seed => () => { seed |= 0; seed = seed + 0x6D2B79F5 | 0; let t = Math.imul(seed ^ seed >>> 15, 1 | seed); t = t + Math.imul(t ^ t >>> 7, 61 | t) ^ t; return ((t ^ t >>> 14) >>> 0) / 4294967296; };
  const c = (p, k) => `#${p[k]}`;
  const glow = s => `<filter id="gl${s}" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="1.4" result="b"/><feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge></filter>`;
  const stars = (p, r, n = 7, maxY = 40) => Array.from({ length: n }, () => `<circle cx="${(r() * W).toFixed(1)}" cy="${(r() * maxY).toFixed(1)}" r="${(.3 + r() * .5).toFixed(2)}"/>`).join('');

  const S = {
    synthwave(p, s, r) {
      const h = 46, sx = 62 + r() * 36, rows = [50, 53, 57, 62, 69, 79], cols = [-80, -50, -25, 0, 25, 50, 80];
      return `<defs><linearGradient id="sk${s}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="${c(p, 'bg')}"/><stop offset="1" stop-color="${c(p, 'purple')}" stop-opacity=".6"/></linearGradient>
        <linearGradient id="su${s}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="${c(p, 'orange')}"/><stop offset="1" stop-color="${c(p, 'hot_pink')}"/></linearGradient>
        <clipPath id="cl${s}"><rect width="${W}" height="${h}"/></clipPath>${glow(s)}</defs>
        <rect width="${W}" height="${H}" fill="${c(p, 'bg')}"/><rect width="${W}" height="${h}" fill="url(#sk${s})"/>
        <g fill="${c(p, 'white')}" opacity=".75">${stars(p, r, 8, 30)}</g>
        <g clip-path="url(#cl${s})" filter="url(#gl${s})"><circle cx="${sx}" cy="${h}" r="23" fill="url(#su${s})"/>
        ${[30, 35, 39, 42.5].map((y, i) => `<rect x="${sx - 30}" y="${y}" width="60" height="${1 + i * .6}" fill="${c(p, 'bg')}" opacity=".9"/>`).join('')}</g>
        <path d="M0 ${h}L18 ${33 + r() * 6} 30 40 46 ${26 + r() * 6} 60 ${h}M100 ${h}l14-${10 + r() * 6} 10 6 16-12 20 20" fill="${c(p, 'panel')}" stroke="${c(p, 'cyan')}" stroke-width=".8"/>
        <rect y="${h}" width="${W}" height="${H - h}" fill="${c(p, 'bg')}"/>
        <g stroke="${c(p, 'hot_pink')}" stroke-width=".7">${rows.map(y => `<path d="M0 ${y}H${W}"/>`).join('')}${cols.map(x => `<path d="M${80 + x * .2} ${h}L${80 + x * 2.2} ${H}"/>`).join('')}</g>
        <path d="M0 ${h}H${W}" stroke="${c(p, 'cyan')}" stroke-width="1.2" filter="url(#gl${s})"/>`;
    },

    cyberdyne(p, s, r) {
      const cx = 92 + r() * 30, cy = 38 + r() * 8;
      const bars = Array.from({ length: 7 }, (_, i) => `<rect x="8" y="${12 + i * 5}" width="${8 + r() * 26}" height="2" fill="${c(p, i % 3 ? 'acid_green' : 'cyan')}" opacity="${.5 + r() * .5}"/>`).join('');
      const graph = Array.from({ length: 9 }, (_, i) => { const bh = 4 + r() * 16; return `<rect x="${8 + i * 4}" y="${72 - bh}" width="2.6" height="${bh}" fill="${c(p, i > 6 ? 'red' : 'orange')}"/>`; }).join('');
      return `<defs>${glow(s)}<radialGradient id="ey${s}"><stop offset="0" stop-color="${c(p, 'red')}" stop-opacity=".9"/><stop offset="1" stop-color="${c(p, 'red')}" stop-opacity="0"/></radialGradient>
        <linearGradient id="sc${s}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="${c(p, 'red')}" stop-opacity="0"/><stop offset=".5" stop-color="${c(p, 'red')}" stop-opacity=".28"/><stop offset="1" stop-color="${c(p, 'red')}" stop-opacity="0"/></linearGradient>
        <pattern id="gr${s}" width="8" height="8" patternUnits="userSpaceOnUse"><path d="M8 0H0V8" fill="none" stroke="${c(p, 'line')}" stroke-width=".5"/></pattern></defs>
        <rect width="${W}" height="${H}" fill="${c(p, 'bg')}"/><rect width="${W}" height="${H}" fill="url(#gr${s})" opacity=".8"/>
        <rect y="${20 + r() * 30}" width="${W}" height="12" fill="url(#sc${s})"/>
        <circle cx="${cx}" cy="${cy}" r="20" fill="url(#ey${s})" opacity=".35"/>
        <g fill="none" stroke="${c(p, 'red')}" filter="url(#gl${s})">
          <circle cx="${cx}" cy="${cy}" r="21" stroke-width=".9"/>
          <circle cx="${cx}" cy="${cy}" r="14" stroke-width=".7" stroke-dasharray="3 2"/>
          <path d="M${cx - 30} ${cy}h12M${cx + 18} ${cy}h12M${cx} ${cy - 30}v12M${cx} ${cy + 18}v12" stroke-width=".8"/>
          <path d="M${cx - 26} ${cy - 20}v-6h6M${cx + 20} ${cy - 26}h6v6M${cx + 26} ${cy + 20}v6h-6M${cx - 20} ${cy + 26}h-6v-6" stroke="${c(p, 'white')}" stroke-width="1"/>
        </g>
        <circle cx="${cx}" cy="${cy}" r="4" fill="${c(p, 'red')}" filter="url(#gl${s})"/><circle cx="${cx}" cy="${cy}" r="1.4" fill="${c(p, 'white')}"/>
        ${bars}${graph}
        <path d="M8 8h30" stroke="${c(p, 'red')}" stroke-width="1.6"/>
        <path d="M${W - 30} 8h22M${W - 20} 12h12" stroke="${c(p, 'muted')}" stroke-width="1.2"/>
        <path d="M0 ${H - 1}H${W}" stroke="${c(p, 'acid_green')}" stroke-width="1" opacity=".7"/>`;
    },

    cyberpunk(p, s, r) {
      const far = [], near = [], win = [], signs = [];
      for (let x = -4; x < W;) { const w = 8 + r() * 12, h = 22 + r() * 30; far.push(`<rect x="${x.toFixed(1)}" y="${(H - h).toFixed(1)}" width="${w.toFixed(1)}" height="${h.toFixed(1)}"/>`); x += w + 1; }
      for (let x = -2; x < W;) {
        const w = 12 + r() * 16, h = 16 + r() * 40, y = H - h;
        near.push(`<rect x="${x.toFixed(1)}" y="${y.toFixed(1)}" width="${w.toFixed(1)}" height="${h.toFixed(1)}"/>`);
        if (r() > .7) near.push(`<path d="M${(x + w / 2).toFixed(1)} ${y.toFixed(1)}v-${(4 + r() * 8).toFixed(1)}"/>`);
        for (let wy = y + 3; wy < H - 3; wy += 4) for (let wx = x + 2; wx < x + w - 2; wx += 3.2) if (r() > .62) win.push(`<rect x="${wx.toFixed(1)}" y="${wy.toFixed(1)}" width="1.6" height="1.6" fill="${c(p, ['cyan', 'orange', 'hot_pink', 'white'][(r() * 4) | 0])}" opacity="${(.35 + r() * .6).toFixed(2)}"/>`);
        if (r() > .55) signs.push(`<rect x="${(x + 2).toFixed(1)}" y="${(y + 4 + r() * 8).toFixed(1)}" width="2.2" height="${(8 + r() * 12).toFixed(1)}" fill="${c(p, r() > .5 ? 'hot_pink' : 'cyan')}"/>`);
        x += w + 2;
      }
      const rain = Array.from({ length: 34 }, () => { const x = r() * (W + 20), y = r() * H; return `<path d="M${x.toFixed(1)} ${y.toFixed(1)}l-3 7"/>`; }).join('');
      return `<defs>${glow(s)}<linearGradient id="sk${s}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="${c(p, 'bg')}"/><stop offset=".65" stop-color="${c(p, 'purple')}" stop-opacity=".55"/><stop offset="1" stop-color="${c(p, 'hot_pink')}" stop-opacity=".45"/></linearGradient></defs>
        <rect width="${W}" height="${H}" fill="${c(p, 'bg')}"/><rect width="${W}" height="${H}" fill="url(#sk${s})"/>
        <circle cx="${30 + r() * 100}" cy="16" r="7" fill="${c(p, 'cyan')}" opacity=".18"/>
        <g fill="${c(p, 'panel')}" opacity=".85">${far.join('')}</g>
        <g fill="${c(p, 'bg')}" stroke="${c(p, 'line')}" stroke-width=".5">${near.join('')}</g>
        <g>${win.join('')}</g><g filter="url(#gl${s})">${signs.join('')}</g>
        <g stroke="${c(p, 'white')}" stroke-width=".35" opacity=".28">${rain}</g>
        <rect y="${H - 3}" width="${W}" height="3" fill="${c(p, 'hot_pink')}" opacity=".35"/>`;
    },

    neosynth(p, s, r) {
      const ph = r() * 6, f1 = 2 + r() * 2;
      const wave = (amp, freq, phase) => { let d = ''; for (let x = 0; x <= W; x += 2) { const y = 34 + Math.sin(x / W * Math.PI * freq + phase) * amp * Math.sin(x / W * Math.PI) + Math.sin(x * .35 + phase) * 2; d += `${x ? 'L' : 'M'}${x} ${y.toFixed(1)}`; } return d; };
      const bars = Array.from({ length: 26 }, (_, i) => { const h = 3 + Math.abs(Math.sin(i * .5 + ph)) * 14 + r() * 6; return `<rect x="${3 + i * 6}" y="${(H - 2 - h).toFixed(1)}" width="4" height="${h.toFixed(1)}" fill="url(#sp${s})"/>`; }).join('');
      const lasers = Array.from({ length: 7 }, (_, i) => `<path d="M80 -4L${(i - 3) * 40 + 80} ${H}"/>`).join('');
      return `<defs>${glow(s)}<radialGradient id="pl${s}" cx=".5" cy=".45" r=".6"><stop offset="0" stop-color="${c(p, 'purple')}" stop-opacity=".7"/><stop offset="1" stop-color="${c(p, 'bg')}" stop-opacity="0"/></radialGradient>
        <linearGradient id="sp${s}" x1="0" y1="1" x2="0" y2="0"><stop offset="0" stop-color="${c(p, 'cyan')}"/><stop offset="1" stop-color="${c(p, 'acid_green')}"/></linearGradient></defs>
        <rect width="${W}" height="${H}" fill="${c(p, 'bg')}"/><rect width="${W}" height="${H}" fill="url(#pl${s})"/>
        <g stroke="${c(p, 'cyan')}" stroke-width=".4" opacity=".35">${lasers}</g>
        <path d="M0 34H${W}" stroke="${c(p, 'line')}" stroke-width=".6"/>
        <g fill="none" stroke-linejoin="round" filter="url(#gl${s})">
          <path d="${wave(13, f1, ph)}" stroke="${c(p, 'hot_pink')}" stroke-width="1.5"/>
          <path d="${wave(8, f1 * 1.6, ph + 1.4)}" stroke="${c(p, 'cyan')}" stroke-width="1"/>
        </g>
        <g opacity=".95">${bars}</g>
        <circle cx="${20 + r() * 120}" cy="12" r="2" fill="${c(p, 'acid_green')}" filter="url(#gl${s})"/>`;
    },

    dystopian(p, s, r) {
      const ruins = []; let x = -2;
      while (x < W) {
        const w = 10 + r() * 18, h = 14 + r() * 30, y = 64 - h;
        const notch = `L${(x + w * .7).toFixed(1)} ${(y + 4 + r() * 6).toFixed(1)}L${(x + w * .45).toFixed(1)} ${(y + r() * 3).toFixed(1)}`;
        ruins.push(`<path d="M${x.toFixed(1)} 64V${y.toFixed(1)}L${(x + w * .3).toFixed(1)} ${y.toFixed(1)}${notch}L${(x + w).toFixed(1)} ${(y + 6 + r() * 8).toFixed(1)}V64Z"/>`);
        x += w + r() * 4;
      }
      const smoke = Array.from({ length: 5 }, () => `<ellipse cx="${(r() * W).toFixed(1)}" cy="${(10 + r() * 25).toFixed(1)}" rx="${(18 + r() * 20).toFixed(1)}" ry="${(4 + r() * 4).toFixed(1)}"/>`).join('');
      return `<defs><linearGradient id="hz${s}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="${c(p, 'bg')}"/><stop offset=".8" stop-color="${c(p, 'orange')}" stop-opacity=".35"/><stop offset="1" stop-color="${c(p, 'acid_green')}" stop-opacity=".25"/></linearGradient>
        <pattern id="st${s}" width="8" height="8" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><rect width="4" height="8" fill="${c(p, 'acid_green')}"/></pattern>
        <filter id="bl${s}"><feGaussianBlur stdDeviation="2.4"/></filter></defs>
        <rect width="${W}" height="${H}" fill="${c(p, 'bg')}"/><rect width="${W}" height="64" fill="url(#hz${s})"/>
        <circle cx="${40 + r() * 80}" cy="${34 + r() * 8}" r="13" fill="${c(p, 'orange')}" opacity=".55" filter="url(#bl${s})"/>
        <g fill="${c(p, 'muted')}" opacity=".22" filter="url(#bl${s})">${smoke}</g>
        <g fill="${c(p, 'panel')}" stroke="${c(p, 'line')}" stroke-width=".6">${ruins.join('')}</g>
        <circle cx="${20 + r() * 120}" cy="${30 + r() * 10}" r="1.3" fill="${c(p, 'red')}"/>
        <rect y="64" width="${W}" height="${H - 64}" fill="${c(p, 'bg')}"/>
        <rect y="68" width="${W}" height="7" fill="url(#st${s})" opacity=".85"/>
        <path d="M0 64H${W}" stroke="${c(p, 'orange')}" stroke-width=".8" opacity=".7"/>`;
    },

    default(p, s, r) {
      const tok = ['hot_pink', 'cyan', 'acid_green', 'orange', 'purple', 'white', 'white'];
      let lines = '', y = 16, indent = 0;
      for (let i = 0; i < 10; i++, y += 6) {
        lines += `<rect x="4" y="${y}" width="6" height="2" fill="${c(p, 'muted')}" opacity=".5"/>`;
        let x = 16 + indent * 6;
        const n = 2 + ((r() * 4) | 0);
        for (let k = 0; k < n && x < 118; k++) { const w = 5 + r() * 18; lines += `<rect x="${x.toFixed(1)}" y="${y}" width="${w.toFixed(1)}" height="2.4" rx="1" fill="${c(p, tok[(r() * tok.length) | 0])}" opacity=".9"/>`; x += w + 3; }
        indent = Math.max(0, Math.min(3, indent + ((r() * 3) | 0) - 1));
      }
      const mini = Array.from({ length: 22 }, (_, i) => `<rect x="134" y="${15 + i * 2.8}" width="${(4 + r() * 16).toFixed(1)}" height="1.2" fill="${c(p, tok[i % tok.length])}" opacity=".45"/>`).join('');
      return `<rect width="${W}" height="${H}" fill="${c(p, 'bg')}"/>
        <rect width="${W}" height="10" fill="${c(p, 'panel')}"/><path d="M0 10H${W}" stroke="${c(p, 'line')}"/>
        <circle cx="7" cy="5" r="1.8" fill="${c(p, 'red')}"/><circle cx="13" cy="5" r="1.8" fill="${c(p, 'orange')}"/><circle cx="19" cy="5" r="1.8" fill="${c(p, 'acid_green')}"/>
        <rect x="28" y="2" width="30" height="8" fill="${c(p, 'bg')}"/><rect x="31" y="5" width="20" height="2" fill="${c(p, 'white')}" opacity=".7"/>
        <rect x="0" y="10" width="13" height="${H - 10}" fill="${c(p, 'panel')}" opacity=".6"/>
        <rect x="13" y="${16 + ((r() * 9) | 0) * 6 - 2}" width="117" height="6" fill="${c(p, 'line')}" opacity=".45"/>
        ${lines}${mini}
        <path d="M130 10V${H}" stroke="${c(p, 'line')}"/>
        <rect x="${40 + r() * 60}" y="${16 + ((r() * 9) | 0) * 6 - 1}" width="1.4" height="4.4" fill="${c(p, 'acid_green')}"/>
        <rect y="${H - 6}" width="${W}" height="6" fill="${c(p, 'purple')}" opacity=".8"/>`;
    },

    'cybercore-tech'(p, s) {
      const cx = 80, cy = 40, R = 15, cols = ['cyan', 'acid_green', 'orange', 'red', 'hot_pink', 'purple'];
      const hex = (rad) => Array.from({ length: 6 }, (_, i) => { const a = Math.PI / 3 * i - Math.PI / 2; return `${(cx + rad * Math.cos(a)).toFixed(1)},${(cy + rad * Math.sin(a)).toFixed(1)}`; }).join(' ');
      const arcs = cols.map((k, i) => { const a0 = Math.PI / 3 * i - Math.PI / 2 + .12, a1 = a0 + Math.PI / 3 - .24, rr = R + 6; return `<path d="M${(cx + rr * Math.cos(a0)).toFixed(1)} ${(cy + rr * Math.sin(a0)).toFixed(1)}A${rr} ${rr} 0 0 1 ${(cx + rr * Math.cos(a1)).toFixed(1)} ${(cy + rr * Math.sin(a1)).toFixed(1)}" stroke="${c(p, k)}"/>`; }).join('');
      const traces = [['M58 40H34V18H10', 'cyan'], ['M58 34H44V8H22', 'purple'], ['M58 46H40V66H8', 'hot_pink'], ['M102 40H128V60H152', 'acid_green'], ['M102 34H118V12H150', 'orange'], ['M102 46H112V72H140', 'red']];
      return `<defs>${glow(s)}<pattern id="gr${s}" width="10" height="10" patternUnits="userSpaceOnUse"><circle cx="5" cy="5" r=".5" fill="${c(p, 'line')}"/></pattern></defs>
        <rect width="${W}" height="${H}" fill="${c(p, 'bg')}"/><rect width="${W}" height="${H}" fill="url(#gr${s})"/>
        <g fill="none" stroke-width="1" filter="url(#gl${s})">${traces.map(([d, k]) => `<path d="${d}" stroke="${c(p, k)}"/>`).join('')}</g>
        ${traces.map(([d, k]) => { const pts = d.match(/[\d.]+/g); return `<rect x="${pts[pts.length - 2] - 1.6}" y="${pts[pts.length - 1] - 1.6}" width="3.2" height="3.2" fill="${c(p, k)}"/>`; }).join('')}
        <polygon points="${hex(R + 12)}" fill="none" stroke="${c(p, 'hot_pink')}" stroke-width="1.2" filter="url(#gl${s})"/>
        <g fill="none" stroke-width="2.2" stroke-linecap="round">${arcs}</g>
        <polygon points="${hex(R - 3)}" fill="${c(p, 'panel')}" stroke="${c(p, 'white')}" stroke-width=".8"/>
        <path d="M${cx + 4.6} ${cy - 4.6}a6.5 6.5 0 1 0 0 9.2" fill="none" stroke="${c(p, 'white')}" stroke-width="2.4"/>
        <rect x="${cx - 1.5}" y="${cy - 1.5}" width="3" height="3" fill="${c(p, 'acid_green')}"/>`;
    }
  };

  function inner(family, p, s, r) { return (S[family] || S.synthwave)(p, s, r); }

  /* scene(id, palette, family, blend?) → <svg> string.
     blend = { baseFamily, accentFamily } for mashups. */
  return function scene(id, p, family, blend) {
    const s = id.replace(/[^a-z0-9]/gi, '');
    const r = rng(seedOf(id));
    let body;
    if (blend) {
      const a = inner(blend.baseFamily, p, `${s}a`, r), b = inner(blend.accentFamily, p, `${s}b`, r);
      body = `<defs><clipPath id="L${s}"><path d="M0 0H96L64 80H0Z"/></clipPath><clipPath id="R${s}"><path d="M96 0H${W}V${H}H64Z"/></clipPath>${glow(s)}</defs>
        <g clip-path="url(#L${s})">${a}</g><g clip-path="url(#R${s})">${b}</g>
        <path d="M96 0L64 80" stroke="${c(p, 'white')}" stroke-width="1.6" filter="url(#gl${s})"/>
        <g transform="translate(80 40)"><rect x="-7" y="-7" width="14" height="14" transform="rotate(45)" fill="${c(p, 'bg')}" stroke="${c(p, 'hot_pink')}" stroke-width="1.2"/><path d="M-2.6-2.6l5.2 5.2M2.6-2.6l-5.2 5.2" stroke="${c(p, 'white')}" stroke-width="1.3"/></g>`;
    } else body = inner(family, p, s, r);
    return `<svg class="scene" viewBox="0 0 ${W} ${H}" aria-hidden="true">${body}</svg>`;
  };
})();
