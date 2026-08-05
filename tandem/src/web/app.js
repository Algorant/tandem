import { api } from './api.js';
import {
  renderAttention,
  renderBoard,
  renderDecisions,
  renderDetail,
  renderHealth,
  renderLogs,
  renderRules,
  showError,
  showLoading,
  replace,
} from './ui.js';

const title = document.querySelector('#project-title');
const revision = document.querySelector('#revision');
const status = document.querySelector('#app-status');
const warnings = document.querySelector('#warning-banner');
const attentionCount = document.querySelector('#attention-count');
const refresh = document.querySelector('#refresh');
const skipLink = document.querySelector('.skip-link');
const main = document.querySelector('#content');

const state = {
  project: null,
  warnings: [],
  revision: null,
  boardFilters: { query: '', state: '', priority: '' },
  logQuery: '',
  request: 0,
  pollInFlight: false,
  pollTimer: null,
};

const REVISION_POLL_MS = 3000;

function route() {
  const raw = location.hash.slice(1) || 'board';
  const [name, ...parts] = raw.split('/');
  let id = null;
  try { id = parts.length ? decodeURIComponent(parts.join('/')) : null; } catch { id = null; }
  return { name, id };
}

function setActiveNav(name) {
  const main = ['board', 'validation', 'logs', 'rules', 'decisions', 'health'].includes(name) ? name : null;
  for (const link of document.querySelectorAll('[data-route]')) {
    if (link.dataset.route === main) link.setAttribute('aria-current', 'page');
    else link.removeAttribute('aria-current');
  }
}

function applyEnvelope(envelope) {
  state.revision = envelope.revision;
  state.warnings = envelope.warnings || [];
  revision.textContent = `Revision ${envelope.revision}`;
  if (state.warnings.length) {
    const details = document.createElement('details');
    const summary = document.createElement('summary');
    summary.textContent = `${state.warnings.length} project warning${state.warnings.length === 1 ? '' : 's'}`;
    const list = document.createElement('ul');
    for (const message of state.warnings) {
      const item = document.createElement('li');
      item.textContent = message;
      list.append(item);
    }
    details.append(summary, list);
    warnings.replaceChildren(details);
    warnings.hidden = false;
  } else {
    warnings.replaceChildren();
    warnings.hidden = true;
  }
}

async function loadProject() {
  const [projectEnvelope, attentionEnvelope] = await Promise.all([api.project(), api.attention()]);
  applyEnvelope(projectEnvelope);
  state.project = projectEnvelope.data;
  title.textContent = state.project.title;
  document.title = `${state.project.title} · Tandem`;
  attentionCount.textContent = String(attentionEnvelope.data.items.length);
  return projectEnvelope;
}

function captureTransientState() {
  const boardQuery = document.querySelector('#board-query');
  const logSearch = document.querySelector('#log-search');
  if (boardQuery) state.boardFilters.query = boardQuery.value;
  if (logSearch) state.logQuery = logSearch.value;
}

function captureViewport() {
  const active = document.activeElement;
  return {
    x: window.scrollX,
    y: window.scrollY,
    focusId: active instanceof HTMLElement ? active.id : '',
    selectionStart: active instanceof HTMLInputElement ? active.selectionStart : null,
    selectionEnd: active instanceof HTMLInputElement ? active.selectionEnd : null,
  };
}

function restoreViewport(saved) {
  window.scrollTo(saved.x, saved.y);
  if (!saved.focusId) return;
  const target = document.getElementById(saved.focusId);
  if (!(target instanceof HTMLElement)) return;
  target.focus({ preventScroll: true });
  if (target instanceof HTMLInputElement && saved.selectionStart != null) {
    target.setSelectionRange(saved.selectionStart, saved.selectionEnd);
  }
}

async function renderRoute({ preserveViewport = false, changed = false, focusHeading = false } = {}) {
  captureTransientState();
  const savedViewport = preserveViewport ? captureViewport() : null;
  const current = route();
  const request = ++state.request;
  setActiveNav(current.name);
  showLoading(`Loading ${current.name} view`);
  status.textContent = `Loading ${current.name}…`;
  refresh.disabled = true;
  try {
    if (!state.project) await loadProject();
    let envelope;
    let content;
    switch (current.name) {
      case 'board':
        envelope = await api.board({ state: state.boardFilters.state, priority: state.boardFilters.priority });
        content = renderBoard(envelope.data, state.boardFilters, (filters) => { state.boardFilters = filters; renderRoute(); });
        break;
      case 'validation':
        envelope = await api.attention();
        attentionCount.textContent = String(envelope.data.items.length);
        content = renderAttention(envelope.data.items);
        break;
      case 'logs':
        envelope = await api.logs(state.logQuery);
        content = renderLogs(envelope.data, state.logQuery, (query) => { state.logQuery = query; renderRoute(); });
        break;
      case 'rules':
        envelope = await api.rules();
        content = renderRules(envelope.data.categories);
        break;
      case 'decisions':
        envelope = await api.decisions();
        content = renderDecisions(envelope.data.items);
        break;
      case 'health':
        envelope = await api.project();
        state.project = envelope.data;
        content = renderHealth(envelope.data, envelope.warnings, envelope.revision);
        break;
      case 'document':
        if (!current.id) throw new Error('No document ID was provided.');
        envelope = await api.document(current.id);
        content = renderDetail(envelope.data, 'document');
        break;
      case 'log':
        if (!current.id) throw new Error('No log ID was provided.');
        envelope = await api.log(current.id);
        content = renderDetail(envelope.data, 'log');
        break;
      case 'decision':
        if (!current.id) throw new Error('No decision ID was provided.');
        envelope = await api.decision(current.id);
        content = renderDetail(envelope.data, 'decision');
        break;
      default:
        location.replace('#board');
        return;
    }
    if (request !== state.request) return;
    applyEnvelope(envelope);
    replace(content);
    const viewName = current.name === 'validation' ? 'Validation' : current.name[0].toUpperCase() + current.name.slice(1);
    status.textContent = changed ? `${viewName} updated to the latest revision.` : `${viewName} loaded. Read-only.`;
    requestAnimationFrame(() => {
      if (savedViewport) restoreViewport(savedViewport);
      else if (focusHeading) document.querySelector('#view-title')?.focus({ preventScroll: true });
    });
  } catch (error) {
    if (request !== state.request) return;
    status.textContent = 'The requested view could not load.';
    showError(error, renderRoute);
  } finally {
    if (request === state.request) refresh.disabled = false;
  }
}

async function pollRevision() {
  if (document.hidden || state.pollInFlight || !state.revision) return;
  state.pollInFlight = true;
  try {
    const envelope = await api.project();
    if (envelope.revision === state.revision) return;
    state.project = envelope.data;
    applyEnvelope(envelope);
    await renderRoute({ preserveViewport: true, changed: true });
  } catch {
    // Keep the current canonical view. The next visible poll retries quietly.
  } finally {
    state.pollInFlight = false;
  }
}

function updatePolling() {
  if (state.pollTimer) {
    clearInterval(state.pollTimer);
    state.pollTimer = null;
  }
  if (document.hidden) return;
  state.pollTimer = setInterval(pollRevision, REVISION_POLL_MS);
}

window.addEventListener('hashchange', () => renderRoute({ focusHeading: true }));
document.addEventListener('visibilitychange', () => {
  updatePolling();
  if (!document.hidden) pollRevision();
});
skipLink.addEventListener('click', (event) => {
  event.preventDefault();
  main.focus();
});
refresh.addEventListener('click', async () => {
  captureTransientState();
  state.project = null;
  await renderRoute();
});
updatePolling();
renderRoute();
