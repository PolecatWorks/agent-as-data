import { initFederation } from '@angular-architects/native-federation';

initFederation()
  .then(() => import('./bootstrap'))
  .catch((err) => {
    console.error('Fatal bootstrap error:', err);
    const errorEl = document.createElement('pre');
    errorEl.style.color = '#b91c1c';
    errorEl.style.padding = '16px';
    errorEl.textContent = `Application failed to start: ${err instanceof Error ? err.message : String(err)}`;
    document.body.replaceChildren(errorEl);
  });
