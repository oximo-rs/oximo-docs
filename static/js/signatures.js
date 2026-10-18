// =============================================================================
// API Signature Popovers
// -----------------------------------------------------------------------------
// Two kinds of targets are recognized:
//   1. links whose href points at docs.rs, and
//   2. identifiers inside Rust code (blocks and inline spans), resolved against
//      the receiver/type context of the surrounding code.
// =============================================================================

(function () {
  'use strict';

  var POPOVER_ID = 'oximo-signature-popover';
  var TYPE_KINDS = { struct: 1, enum: 1, trait: 1, union: 1, type: 1, fn: 1, macro: 1, constant: 1, static: 1, primitive: 1 };
  var RETURN_RE = /[A-Za-z_][A-Za-z0-9_]*/g;
  var IDENT_RE = /[A-Za-z_][A-Za-z0-9_]*/g;
  var CORRIDOR_PAD = 6;

  var INDEX_URL = null;
  var CHANNEL = null;
  var items = null;    // Channel item map: key -> { signature, summary, source }
  var symbols = null;  // Derived lookup tables
  var urlByKey = null; // key -> docs.rs URL
  var loading = null;

  var currentAnchor = null;
  var corridor = null;
  var hideTimer = null;
  var hoverCapable = window.matchMedia && window.matchMedia('(hover: hover)').matches;

  // --- Index loading --------------------------------------------------------

  function readMeta(name) {
    var el = document.querySelector('meta[name="' + name + '"]');
    return el ? el.getAttribute('content') : null;
  }

  function indexUrlFromScript() {
    var el = document.currentScript;
    if (!el) {
      var scripts = document.querySelectorAll('script[src]');
      for (var i = 0; i < scripts.length; i++) {
        var src = scripts[i].getAttribute('src') || '';
        if (/\/js\/signatures\.js(\?|#|$)/.test(src)) { el = scripts[i]; break; }
      }
    }
    if (!el || !el.src) return null;
    try {
      return new URL('../docs-signatures.json', el.src).href;
    } catch (error) {
      return null;
    }
  }

  function resolveChannel(data) {
    var channels = data && data.channels;
    if (!channels) return null;
    var key = CHANNEL === 'dev' ? 'main' : CHANNEL;
    return channels[key] || channels.main || channels[Object.keys(channels)[0]] || null;
  }

  function ensureLoaded() {
    if (items) return Promise.resolve(symbols);
    if (loading) return loading;
    if (!INDEX_URL) return Promise.resolve(null);

    loading = fetch(INDEX_URL, { credentials: 'omit' })
      .then(function (response) {
        if (!response.ok) throw new Error('HTTP ' + response.status);
        return response.json();
      })
      .then(function (data) {
        var channel = resolveChannel(data);
        items = (channel && channel.items) || {};
        symbols = buildSymbols(items);
        return symbols;
      })
      .catch(function (error) {
        console.warn('[signatures] index unavailable:', error);
        items = {};
        symbols = { types: new Map(), qualified: new Map(), byName: new Map() };
        urlByKey = new Map();
        return symbols;
      });

    return loading;
  }

  // --- Symbol tables --------------------------------------------------------

  function parseKey(key) {
    var hash = key.indexOf('#');
    var pathPart = hash === -1 ? key : key.slice(0, hash);
    var frag = hash === -1 ? '' : key.slice(hash + 1);
    var slash = pathPart.indexOf('/');
    var crate = pathPart.slice(0, slash);
    var rest = pathPart.slice(slash + 1);
    var file = rest.split('/').pop();

    var fileKind = null;
    var typeName = null;
    var fileMatch = /^(struct|enum|trait|union|type|fn|macro|constant|static|primitive)\.([A-Za-z0-9_]+)\.html$/.exec(file);
    if (fileMatch) {
      fileKind = fileMatch[1];
      typeName = fileMatch[2];
    }

    var memberKind = null;
    var memberName = null;
    var memberMatch = /^([a-z]+)\.(.+)$/.exec(frag);
    if (memberMatch) {
      memberKind = memberMatch[1];
      memberName = memberMatch[2];
    }

    return { crate: crate, rest: rest, frag: frag, fileKind: fileKind, typeName: typeName, memberKind: memberKind, memberName: memberName };
  }

  function buildSymbols(map) {
    var types = new Map();
    var qualified = new Map();
    var byName = new Map();
    var seenByName = new Set();
    urlByKey = new Map();

    Object.keys(map).forEach(function (key) {
      var entry = map[key];
      var parsed = parseKey(key);
      var url = 'https://docs.rs/' + parsed.crate + '/' + (entry.source || 'latest') + '/' + parsed.rest;
      if (parsed.frag) url += '#' + parsed.frag;
      urlByKey.set(key, url);

      if (parsed.memberName && parsed.memberKind && parsed.typeName) {
        var sym = { key: key, name: parsed.memberName, kind: parsed.memberKind, container: parsed.typeName, signature: entry.signature || '' };
        qualified.set(parsed.typeName + '\u0000' + parsed.memberName, sym);
        var dedup = parsed.typeName + '\u0000' + parsed.memberName;
        if (!seenByName.has(dedup)) {
          seenByName.add(dedup);
          if (!byName.has(parsed.memberName)) byName.set(parsed.memberName, []);
          byName.get(parsed.memberName).push(sym);
        }
        return;
      }

      if (parsed.typeName && TYPE_KINDS[parsed.fileKind] && !types.has(parsed.typeName)) {
        types.set(parsed.typeName, { key: key, name: parsed.typeName, kind: parsed.fileKind });
      }
    });

    return { types: types, qualified: qualified, byName: byName };
  }

  function returnContainer(sym) {
    if (!sym) return null;
    var arrow = sym.signature.indexOf('->');
    if (arrow === -1) return null;
    var rest = sym.signature.slice(arrow + 2).split(/\bwhere\b/)[0].replace(/[{;][\s\S]*$/, '');
    var identifiers = rest.match(RETURN_RE) || [];
    for (var i = 0; i < identifiers.length; i++) {
      if (symbols.types.has(identifiers[i])) return identifiers[i];
    }
    return null;
  }

  // Best-effort variable -> container inference from `let` bindings.
  function inferVarTypes(code) {
    var vars = new Map();
    var setVar = function (name, type) {
      if (type && symbols.types.has(type)) vars.set(name, type);
    };

    for (var pass = 0; pass < 3; pass++) {
      var match;
      var annotated = /\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([A-Za-z_][A-Za-z0-9_]*)/g;
      while ((match = annotated.exec(code))) setVar(match[1], match[2]);

      var called = /\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*([A-Za-z_][A-Za-z0-9_]*)\s*(::|\.)\s*([A-Za-z_][A-Za-z0-9_]*)\s*[(<{]/g;
      while ((match = called.exec(code))) {
        var receiver = match[2];
        var method = match[4];
        var container = symbols.types.has(receiver) ? receiver : vars.get(receiver);
        if (!container) continue;
        if (method === 'new') { setVar(match[1], container); continue; }
        var sym = symbols.qualified.get(container + '\u0000' + method);
        if (sym) setVar(match[1], returnContainer(sym));
      }
    }
    return vars;
  }

  // --- Token resolution -----------------------------------------------------

  function containerForReceiver(receiver, vars) {
    if (!receiver) return null;
    if (symbols.types.has(receiver)) return receiver;
    return vars.get(receiver) || null;
  }

  function resolveMatch(match, vars, blockContainers) {
    var name = match.name;
    var container = containerForReceiver(match.receiver, vars);

    if (!match.dot && !match.path) {
      var type = symbols.types.get(name);
      return type ? type.key : null;
    }

    if (container) {
      var qualified = symbols.qualified.get(container + '\u0000' + name);
      return qualified ? qualified.key : null;
    }

    var candidates = (symbols.byName.get(name) || []).filter(function (sym) {
      if (match.call) return sym.kind === 'method' || sym.kind === 'tymethod';
      if (match.path) return sym.kind === 'variant' || sym.kind === 'associatedconstant' || sym.kind === 'associatedtype';
      return sym.kind === 'structfield' || sym.kind === 'associatedconstant' || sym.kind === 'associatedtype';
    });
    if (candidates.length === 0) return null;

    var containers = [];
    candidates.forEach(function (sym) {
      if (containers.indexOf(sym.container) === -1) containers.push(sym.container);
    });
    if (containers.length === 1) return candidates[0].key;

    var inBlock = containers.filter(function (name) { return blockContainers.has(name); });
    if (inBlock.length === 1) {
      return candidates.filter(function (sym) { return sym.container === inBlock[0]; })[0].key;
    }
    return null;
  }

  function collectMatches(text) {
    var matches = [];
    var regex = new RegExp(IDENT_RE.source, 'g');
    var match;
    while ((match = regex.exec(text))) {
      var start = match.index;
      var end = start + match[0].length;
      var dot = text[start - 1] === '.' && text[start - 2] !== '.';
      var path = text.slice(start - 2, start) === '::';
      var receiver = null;
      var separator = path ? 2 : dot ? 1 : 0;
      if (separator) {
        var receiverEnd = start - separator;
        var receiverStart = receiverEnd;
        while (receiverStart > 0 && /[A-Za-z0-9_]/.test(text[receiverStart - 1])) receiverStart--;
        if (receiverStart < receiverEnd) receiver = text.slice(receiverStart, receiverEnd);
      }
      matches.push({ name: match[0], start: start, end: end, dot: dot, path: path, call: text[end] === '(', receiver: receiver });
    }
    return matches;
  }

  function wrapMatches(codeEl, matches, vars) {
    var blockContainers = new Set();
    matches.forEach(function (match) {
      var container = containerForReceiver(match.receiver, vars);
      if (container) { blockContainers.add(container); return; }
      if (match.call && !match.dot && !match.path) return;
      var candidates = symbols.byName.get(match.name);
      if (candidates && candidates.length > 0) {
        var containers = [];
        candidates.forEach(function (sym) { if (sym.container && containers.indexOf(sym.container) === -1) containers.push(sym.container); });
        if (containers.length === 1) blockContainers.add(containers[0]);
      }
    });

    var segments = [];
    (function () {
      var walker = document.createTreeWalker(codeEl, NodeFilter.SHOW_TEXT, null);
      var node, offset = 0;
      while ((node = walker.nextNode())) {
        segments.push({ node: node, start: offset, end: offset + node.nodeValue.length });
        offset += node.nodeValue.length;
      }
    })();

    var perNode = new Map();
    matches.forEach(function (match) {
      var key = resolveMatch(match, vars, blockContainers);
      if (!key) return;
      var segment = null;
      for (var i = 0; i < segments.length; i++) {
        if (match.start >= segments[i].start && match.end <= segments[i].end) { segment = segments[i]; break; }
      }
      if (!segment) return;
      if (!perNode.has(segment.node)) perNode.set(segment.node, []);
      perNode.get(segment.node).push({ start: match.start - segment.start, end: match.end - segment.start, key: key });
    });

    perNode.forEach(function (list, node) {
      list.sort(function (a, b) { return b.start - a.start; });
      list.forEach(function (item) {
        var target = node.splitText(item.start);
        var after = target.splitText(item.end - item.start);
        var span = document.createElement('span');
        span.className = 'sig-ref';
        span.setAttribute('data-key', item.key);
        span.textContent = target.nodeValue;
        target.parentNode.replaceChild(span, target);
        void after;
      });
    });
  }

  function linkCode(root) {
    if (!symbols) return;
    var pageText = '';
    var blocks = [];

    root.querySelectorAll('pre > code').forEach(function (code) {
      var lang = (code.getAttribute('data-lang') || code.className || '').toLowerCase();
      if (lang.indexOf('rust') !== 0 && lang.indexOf('language-rust') === -1) return;
      blocks.push(code);
      pageText += code.textContent + '\n';
    });

    root.querySelectorAll('code').forEach(function (code) {
      if (code.parentElement && code.parentElement.tagName === 'PRE') return;
      if (code.closest('a')) return;
      if (code.closest('h1, h2, h3, h4, h5, h6')) return;
      if (code.children.length > 0) return;
      blocks.push(code);
    });

    if (blocks.length === 0) return;
    var vars = inferVarTypes(pageText);

    blocks.forEach(function (code) {
      if (code.dataset.oximoSig === '1') return;
      code.dataset.oximoSig = '1';
      var matches = collectMatches(code.textContent);
      if (matches.length) wrapMatches(code, matches, vars);
    });
  }

  // --- Key from a docs.rs anchor -------------------------------------------

  function keyForAnchor(anchor) {
    var url;
    try {
      url = new URL(anchor.href, window.location.href);
    } catch (error) {
      return null;
    }
    if (url.hostname !== 'docs.rs') return null;
    var segments = url.pathname.split('/').filter(Boolean);
    if (segments.length < 3) return null;
    var rest = segments.slice(2);
    if (rest.length === 0) return null;
    var key = segments[0] + '/' + rest.join('/');
    if (url.hash) key += url.hash;
    return key;
  }

  // --- Popover element ------------------------------------------------------

  var popover, codeEl, summaryEl, linkEl;

  function buildPopover() {
    popover = document.createElement('div');
    popover.id = POPOVER_ID;
    popover.className = 'signature-popover';
    popover.setAttribute('role', 'tooltip');
    popover.setAttribute('aria-hidden', 'true');

    codeEl = document.createElement('div');
    codeEl.className = 'signature-popover__code';

    summaryEl = document.createElement('p');
    summaryEl.className = 'signature-popover__summary';

    linkEl = document.createElement('a');
    linkEl.className = 'signature-popover__link';
    linkEl.target = '_blank';
    linkEl.rel = 'noopener noreferrer';
    linkEl.tabIndex = -1;
    linkEl.textContent = 'Open on docs.rs ↗';

    popover.appendChild(codeEl);
    popover.appendChild(summaryEl);
    popover.appendChild(linkEl);
    document.body.appendChild(popover);

    popover.addEventListener('mouseenter', cancelHide);
    popover.addEventListener('mouseleave', scheduleHide);
  }

  function updateCorridor() {
    if (!currentAnchor || !popover) {
      corridor = null;
      return;
    }
    var a = currentAnchor.getBoundingClientRect();
    var p = popover.getBoundingClientRect();
    if (!p.width && !p.height) {
      corridor = null;
      return;
    }
    corridor = {
      aLeft: a.left, aRight: a.right, aTop: a.top, aBottom: a.bottom,
      pLeft: p.left, pRight: p.right, pTop: p.top, pBottom: p.bottom,
      below: p.top >= a.bottom
    };
  }

  function inCorridor(x, y) {
    if (!corridor) return false;
    var aEdge = corridor.below ? corridor.aBottom : corridor.aTop;
    var pEdge = corridor.below ? corridor.pTop : corridor.pBottom;
    if (y < Math.min(aEdge, pEdge) - CORRIDOR_PAD || y > Math.max(aEdge, pEdge) + CORRIDOR_PAD) {
      return false;
    }

    var span = pEdge - aEdge;
    var t = span === 0 ? 0.5 : (y - aEdge) / span;
    if (t < 0) t = 0;
    else if (t > 1) t = 1;

    var x1 = corridor.aLeft + (corridor.pLeft - corridor.aLeft) * t;
    var x2 = corridor.aRight + (corridor.pRight - corridor.aRight) * t;
    return x >= Math.min(x1, x2) - CORRIDOR_PAD && x <= Math.max(x1, x2) + CORRIDOR_PAD;
  }

  function renderSummary(text) {
    summaryEl.textContent = '';
    if (!text) return;

    var ticks = (text.match(/`/g) || []).length;
    if (ticks % 2 === 1) text += '`';
    var parts = text.split('`');

    for (var i = 0; i < parts.length; i++) {
      if (!parts[i]) continue;
      if (i % 2 === 0) {
        summaryEl.appendChild(document.createTextNode(parts[i]));
        continue;
      }
      var code = document.createElement('code');
      code.textContent = parts[i];
      var type = symbols && symbols.types ? symbols.types.get(parts[i]) : null;
      var url = type ? urlByKey.get(type.key) : null;
      if (!url) {
        summaryEl.appendChild(code);
        continue;
      }
      var link = document.createElement('a');
      link.href = url;
      link.target = '_blank';
      link.rel = 'noopener noreferrer';
      link.appendChild(code);
      summaryEl.appendChild(link);
    }
  }

  function positionPopover(target) {
    var gap = 8;
    var rect = target.getBoundingClientRect();
    var box = popover.getBoundingClientRect();
    var scrollX = window.pageXOffset || document.documentElement.scrollLeft;
    var scrollY = window.pageYOffset || document.documentElement.scrollTop;

    var top = rect.bottom + gap + scrollY;
    if (rect.bottom + gap + box.height > window.innerHeight && rect.top - gap - box.height > 0) {
      top = rect.top - gap - box.height + scrollY;
    }

    var left = rect.left + scrollX;
    var maxLeft = scrollX + window.innerWidth - box.width - gap;
    left = Math.max(scrollX + gap, Math.min(left, maxLeft));

    popover.style.transform = 'translate(' + Math.round(left) + 'px, ' + Math.round(top) + 'px)';
  }

  function show(target, entry, url) {
    clearTimeout(hideTimer);
    hideTimer = null;

    if (currentAnchor && currentAnchor !== target) {
      currentAnchor.removeAttribute('aria-describedby');
    }
    currentAnchor = target;

    codeEl.textContent = entry.signature;
    if (entry.summary) {
      renderSummary(entry.summary);
      summaryEl.hidden = false;
    } else {
      renderSummary('');
      summaryEl.hidden = true;
    }
    linkEl.href = url;

    popover.setAttribute('aria-hidden', 'false');
    linkEl.tabIndex = 0;
    if (target.tagName === 'A') target.setAttribute('aria-describedby', POPOVER_ID);

    positionPopover(target);
    popover.classList.add('is-visible');
    updateCorridor();
  }

  function hide() {
    clearTimeout(hideTimer);
    hideTimer = null;
    corridor = null;
    if (currentAnchor) {
      currentAnchor.removeAttribute('aria-describedby');
      currentAnchor = null;
    }
    if (popover) {
      popover.classList.remove('is-visible');
      popover.setAttribute('aria-hidden', 'true');
      linkEl.tabIndex = -1;
    }
  }

  function scheduleHide() {
    clearTimeout(hideTimer);
    hideTimer = setTimeout(hide, 120);
  }

  function cancelHide() {
    clearTimeout(hideTimer);
    hideTimer = null;
  }

  // --- Event handling -------------------------------------------------------

  function targetFromEvent(event) {
    var target = event.target;
    if (!target || !target.closest) return null;
    var anchor = target.closest('a[href^="https://docs.rs/"]');
    if (anchor) return { element: anchor, key: keyForAnchor(anchor), url: anchor.href };
    var span = target.closest('.sig-ref[data-key]');
    if (span) return { element: span, key: span.getAttribute('data-key'), url: urlByKey.get(span.getAttribute('data-key')) };
    return null;
  }

  function handleOver(event) {
    if (event.type === 'mouseover' && !hoverCapable) return;
    var found = targetFromEvent(event);
    if (!found || !found.key) return;
    if (popover && popover.contains(found.element)) return;
    if (event.type === 'mouseover' && found.element !== currentAnchor && inCorridor(event.clientX, event.clientY)) {
      cancelHide();
      return;
    }
    if (event.type === 'focusin' && found.element.tagName === 'A') {
      try {
        if (!found.element.matches(':focus-visible')) return;
      } catch (error) { /* unsupported */ }
    }
    ensureLoaded().then(function () {
      if (!found.element.isConnected) return;
      var entry = items[found.key];
      if (!entry) return;
      show(found.element, entry, found.url);
    });
  }

  function handleOut(event) {
    if (event.type === 'mouseout' && inCorridor(event.clientX, event.clientY)) return;
    var found = targetFromEvent(event);
    if (!found) return;
    var related = event.relatedTarget;
    if (related && (found.element.contains(related) || (popover && popover.contains(related)))) return;
    if (event.type === 'focusout' && popover && popover.contains(related)) return;
    scheduleHide();
  }

  function init() {
    INDEX_URL = indexUrlFromScript() || readMeta('oximo-signatures');
    CHANNEL = readMeta('oximo-docs-version');
    if (!INDEX_URL) return;

    var content = document.querySelector('.article__content');
    var hasLinks = document.querySelector('a[href^="https://docs.rs/"]');
    if (!content && !hasLinks) return;

    buildPopover();

    document.addEventListener('mouseover', handleOver);
    document.addEventListener('mouseout', handleOut);
    document.addEventListener('focusin', handleOver);
    document.addEventListener('focusout', handleOut);
    document.addEventListener('mousemove', function (event) {
      if (corridor && inCorridor(event.clientX, event.clientY)) cancelHide();
    });

    document.addEventListener('keydown', function (event) {
      if (event.key === 'Escape') hide();
    });
    window.addEventListener('scroll', hide, { capture: true, passive: true });
    window.addEventListener('resize', hide, { passive: true });

    if (content) {
      ensureLoaded().then(function (data) {
        if (data) linkCode(content);
      });
    }

    if ('requestIdleCallback' in window) {
      window.requestIdleCallback(ensureLoaded, { timeout: 3000 });
    } else {
      window.setTimeout(ensureLoaded, 2000);
    }
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
