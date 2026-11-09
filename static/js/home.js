(function() {
  'use strict';

  function fallbackCopy(text) {
    var textarea = document.createElement('textarea');
    textarea.value = text;
    textarea.setAttribute('readonly', '');
    textarea.style.cssText = 'position:fixed;left:-9999px;top:0;opacity:0';
    document.body.appendChild(textarea);
    textarea.focus();
    textarea.select();
    textarea.setSelectionRange(0, textarea.value.length);

    var copied = false;
    try {
      copied = document.execCommand('copy');
    } catch (error) {
      copied = false;
    }
    document.body.removeChild(textarea);
    return copied;
  }

  function copyText(text) {
    if (navigator.clipboard && window.isSecureContext) {
      return navigator.clipboard.writeText(text)
        .then(function() { return true; })
        .catch(function() { return fallbackCopy(text); });
    }
    return Promise.resolve(fallbackCopy(text));
  }

  document.querySelectorAll('[data-copy]').forEach(function(button) {
    button.addEventListener('click', function() {
      var text = button.getAttribute('data-copy');
      var label = button.querySelector('.docs-home__install-copy-label');
      var previousLabel = label ? label.textContent : null;

      copyText(text).then(function(copied) {
        if (!copied) {
          if (label) label.textContent = 'Copy failed';
          window.setTimeout(function() {
            if (label && previousLabel !== null) label.textContent = previousLabel;
          }, 1600);
          return;
        }

        button.classList.add('is-copied');
        if (label) label.textContent = 'Copied';
        window.setTimeout(function() {
          button.classList.remove('is-copied');
          if (label && previousLabel !== null) label.textContent = previousLabel;
        }, 1600);
      });
    });
  });

  var panels = document.querySelectorAll('.home-grid');
  if (!panels.length) return;

  var cellSize = 30;
  var maxCells = 2400;
  var mediaQuery = window.matchMedia('(min-width: 1340px)');

  function buildGrid() {
    Array.prototype.forEach.call(panels, function(panel) {
      if (!mediaQuery.matches) return;

      panel.textContent = '';
      var width = panel.clientWidth;
      var height = panel.clientHeight;
      if (width < 40 || height < 40) return;

      var columns = Math.max(1, Math.round(width / cellSize));
      var rows = Math.max(1, Math.round(height / cellSize));
      var count = Math.min(columns * rows, maxCells);
      panel.style.gridTemplateColumns = 'repeat(' + columns + ', 1fr)';
      panel.style.gridTemplateRows = 'repeat(' + rows + ', 1fr)';

      var fragment = document.createDocumentFragment();
      for (var i = 0; i < count; i++) {
        var cell = document.createElement('span');
        cell.className = 'home-grid__cell';
        fragment.appendChild(cell);
      }
      panel.appendChild(fragment);
    });
  }

  var resizeTimer;
  function scheduleGridBuild() {
    window.clearTimeout(resizeTimer);
    resizeTimer = window.setTimeout(buildGrid, 150);
  }

  buildGrid();
  window.addEventListener('resize', scheduleGridBuild);
  if (mediaQuery.addEventListener) mediaQuery.addEventListener('change', buildGrid);
})();
