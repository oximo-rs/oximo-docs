(function() {
  'use strict';

  var wrappers = [];
  document.querySelectorAll('.article__content table').forEach(function(table) {
    var wrapper = document.createElement('div');
    wrapper.className = 'table-scroll';

    table.parentNode.insertBefore(wrapper, table);
    wrapper.appendChild(table);
    wrappers.push(wrapper);

    wrapper.addEventListener('scroll', function() {
      wrapper.classList.toggle('table-scroll--scrolled', wrapper.scrollLeft > 0);
    }, {passive: true});
  });

  function updateOverflow() {
    wrappers.forEach(function(wrapper) {
      var isOverflowing = wrapper.scrollWidth > wrapper.clientWidth + 1;
      wrapper.classList.toggle('table-scroll--overflowing', isOverflowing);
      if (isOverflowing) {
        wrapper.setAttribute('tabindex', '0');
        wrapper.setAttribute('role', 'region');
        wrapper.setAttribute('aria-label', 'Scrollable table. Scroll horizontally to view all columns.');
      } else {
        wrapper.removeAttribute('tabindex');
        wrapper.removeAttribute('role');
        wrapper.removeAttribute('aria-label');
        wrapper.classList.remove('table-scroll--scrolled');
      }
    });
  }

  updateOverflow();
  window.addEventListener('resize', updateOverflow, {passive: true});
})();
