const view = document.querySelector('#view');

export function el(tag, attrs = {}, children = []) {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(attrs)) {
    if (value == null || value === false) continue;
    if (key === 'class') node.className = value;
    else if (key === 'text') node.textContent = String(value);
    else if (key.startsWith('on') && typeof value === 'function') node.addEventListener(key.slice(2), value);
    else node.setAttribute(key, value === true ? '' : String(value));
  }
  const values = Array.isArray(children) ? children : [children];
  for (const child of values.flat(Infinity)) {
    if (child == null || child === false) continue;
    node.append(child instanceof Node ? child : document.createTextNode(String(child)));
  }
  return node;
}

function safeMarkdown(html) {
  const content = el('div', { class: 'prose' });
  // `bodyHtml` comes only from the server's strict renderer. That renderer
  // escapes project text and cannot emit links, images, or raw HTML.
  content.innerHTML = html;
  return content;
}

export function replace(content) {
  view.replaceChildren(content);
}

export function showLoading(label) {
  replace(el('section', { class: 'panel skeleton', 'aria-label': label, 'aria-busy': 'true' }));
}

export function showError(error, retry) {
  replace(el('section', { class: 'panel error-state', role: 'alert' }, [
    el('h2', { text: 'This view could not load' }),
    el('p', { text: error.message || 'An unexpected read error occurred.' }),
    el('p', { class: 'muted', text: error.code ? `Error code: ${error.code}` : '' }),
    el('button', { type: 'button', onClick: retry, text: 'Try again' }),
  ]));
}

export function heading(title, description, action) {
  return el('div', { class: 'view-heading' }, [
    el('div', {}, [el('h2', { id: 'view-title', tabindex: '-1', text: title }), el('p', { text: description })]),
    action,
  ]);
}

function badge(label, value, tone) {
  if (!value) return null;
  return el('span', { class: 'badge', 'data-tone': tone, text: `${label}: ${value}` });
}

function statusTone(value) {
  if (['accepted', 'healthy', 'completed'].includes(value)) return 'success';
  if (['failed', 'blocked', 'canceled', 'changes-requested'].includes(value)) return 'danger';
  if (['warnings', 'validation', 'delivered', 'pending', 'high', 'critical'].includes(value)) return 'attention';
  return null;
}

export function badgeRow(item, includeState = true) {
  return el('div', { class: 'badge-row' }, [
    includeState ? badge('State', item.state, statusTone(item.state)) : null,
    badge('Role', item.role || item.type),
    badge('Priority', item.priority, statusTone(item.priority)),
    badge('Accord', item.accordStatus, statusTone(item.accordStatus)),
    badge('Review', item.reviewStatus, statusTone(item.reviewStatus)),
  ]);
}

function itemHref(item, location = item.location) {
  if (item.type === 'decision') return `#decision/${encodeURIComponent(item.id)}`;
  if (location === 'logs') return `#log/${encodeURIComponent(item.id)}`;
  return `#document/${encodeURIComponent(item.id)}`;
}

// Lanes are a projection of the protocol `kind` alone. Tags are topical and
// never classify a Task.
const LANES = [
  { id: 'standard', name: 'Standard', hint: 'ordered by priority, then ID' },
  { id: 'research', name: 'Research', hint: 'kind: research · ordered by priority, then ID' },
  { id: 'papercut', name: 'Papercuts', hint: 'kind: papercut · ordered by priority, then ID' },
];
const PRIORITY_RANK = { critical: 0, high: 1, medium: 2, low: 3 };
const PRIORITY_LABEL = { critical: 'CRIT', high: 'HIGH', medium: 'MED', low: 'LOW' };

// A lane the reader collapsed stays collapsed across the revision-poll re-render.
const collapsedLanes = new Set();

function laneOf(item) {
  if (item.kind === 'research') return 'research';
  if (item.kind === 'papercut') return 'papercut';
  return 'standard';
}

function idParts(id) {
  return String(id).split('-').map((part) => (/^\d+$/.test(part) ? Number(part) : part));
}

function compareIds(a, b) {
  const left = idParts(a);
  const right = idParts(b);
  for (let i = 0; i < Math.max(left.length, right.length); i += 1) {
    if (left[i] === right[i]) continue;
    if (left[i] === undefined) return -1;
    if (right[i] === undefined) return 1;
    if (typeof left[i] === typeof right[i]) return left[i] < right[i] ? -1 : 1;
    return typeof left[i] === 'number' ? -1 : 1;
  }
  return 0;
}

function compareItems(a, b) {
  const rank = (item) => PRIORITY_RANK[item.priority] ?? 4;
  return rank(a) - rank(b) || compareIds(a.id, b.id);
}

// Orders one lane: priority then ID, with a Task nested beneath its parent when
// that parent sits in the same lane. Anything else is a lane root and carries
// its parent as context.
function laneRows(laneItems) {
  const ids = new Set(laneItems.map((item) => item.id));
  const children = new Map();
  for (const item of [...laneItems].sort(compareItems)) {
    const key = item.parentId && ids.has(item.parentId) ? item.parentId : null;
    if (!children.has(key)) children.set(key, []);
    children.get(key).push(item);
  }
  const rows = [];
  const append = (parent, depth, seen) => {
    for (const item of children.get(parent) || []) {
      if (seen.has(item.id)) continue;
      seen.add(item.id);
      rows.push({ item, depth });
      append(item.id, Math.min(depth + 1, 2), seen);
    }
  };
  const seen = new Set();
  append(null, 0, seen);
  for (const item of laneItems) if (!seen.has(item.id)) rows.push({ item, depth: 0 });
  return rows;
}

function laneRow({ item, depth }, ctx) {
  const parent = item.parentId && !ctx.inLane(item) ? ctx.byId.get(item.parentId) : null;
  const childCount = ctx.childCounts.get(item.id) || 0;
  const context = item.parentId && !ctx.inLane(item)
    ? `under ${item.parentId}${parent ? ` · ${parent.title}` : ''}`
    : null;
  return el('li', { class: 'lane-row', 'data-depth': depth, 'data-kind': item.kind || 'task' }, [
    el('span', { class: 'row-id', text: item.id }),
    el('span', { class: 'chip', text: item.state || 'no state' }),
    item.priority ? el('span', { class: 'chip', 'data-priority': item.priority, text: PRIORITY_LABEL[item.priority] || item.priority }) : null,
    item.kind === 'epic' ? el('span', { class: 'chip chip-epic', text: 'EPIC' }) : null,
    item.accordStatus && item.accordStatus !== 'ready' ? el('span', { class: 'chip', 'data-tone': statusTone(item.accordStatus), text: item.accordStatus }) : null,
    el('span', { class: 'row-main' }, [
      el('a', { class: 'row-title', href: itemHref(item), text: item.title }),
      context ? el('span', { class: 'row-context', text: context }) : null,
    ]),
    childCount ? el('span', { class: 'row-children', text: `+${childCount} ${childCount === 1 ? 'child' : 'children'}` }) : null,
  ]);
}

function laneSection(lane, laneItems, ctx) {
  const count = el('span', { class: 'lane-count', text: laneItems.length });
  const name = el('span', { class: 'lane-name', text: lane.name });
  const attrs = { class: 'lane', 'data-lane': lane.id, 'aria-label': `${lane.name} lane, ${laneItems.length} ${laneItems.length === 1 ? 'task' : 'tasks'}` };
  if (!laneItems.length) {
    // An empty lane collapses to its header.
    return el('section', { ...attrs, 'data-empty': 'true' }, el('div', { class: 'lane-head' }, [
      el('span', { class: 'lane-title' }, [name, count]),
      el('span', { class: 'lane-hint', text: 'empty' }),
    ]));
  }
  const details = el('details', { open: !collapsedLanes.has(lane.id) }, [
    el('summary', { class: 'lane-head' }, [
      el('span', { class: 'lane-title' }, [name, count]),
      el('span', { class: 'lane-hint', text: lane.hint }),
    ]),
    el('ul', { class: 'lane-rows' }, laneRows(laneItems).map((row) => laneRow(row, ctx))),
  ]);
  details.addEventListener('toggle', () => {
    if (details.open) collapsedLanes.delete(lane.id);
    else collapsedLanes.add(lane.id);
  });
  return el('section', attrs, details);
}

export function renderBoard(data, filters, onFilter) {
  const states = data.states || [];
  const items = data.items.filter((item) => {
    const query = filters.query.trim().toLowerCase();
    return item.type === 'task' && (!query || `${item.id} ${item.title} ${(item.tags || []).join(' ')}`.toLowerCase().includes(query));
  });
  const stateOptions = [el('option', { value: '', text: 'All configured states' })];
  for (const state of states) stateOptions.push(el('option', { value: state, text: state }));
  const form = el('form', { class: 'toolbar', 'aria-label': 'Board filters' }, [
    el('div', { class: 'field grow' }, [el('label', { for: 'board-query', text: 'Filter by ID, title, or tag' }), el('input', { id: 'board-query', type: 'search', value: filters.query, placeholder: 'Filter this board' })]),
    el('div', { class: 'field' }, [el('label', { for: 'board-state', text: 'Workflow state' }), el('select', { id: 'board-state' }, stateOptions)]),
    el('div', { class: 'field' }, [el('label', { for: 'board-priority', text: 'Priority' }), el('select', { id: 'board-priority' }, [
      el('option', { value: '', text: 'All priorities' }), ...['critical', 'high', 'medium', 'low'].map((value) => el('option', { value, text: value })),
    ])]),
    el('button', { type: 'submit', text: 'Apply filters' }),
  ]);
  form.querySelector('#board-state').value = filters.state;
  form.querySelector('#board-priority').value = filters.priority;
  form.addEventListener('submit', (event) => {
    event.preventDefault();
    onFilter({
      query: form.querySelector('#board-query').value,
      state: form.querySelector('#board-state').value,
      priority: form.querySelector('#board-priority').value,
    });
  });
  // Parent context and child counts read the whole payload, not the text
  // filter, so a filtered-out parent is still named.
  const byId = new Map(data.items.map((item) => [item.id, item]));
  const childCounts = new Map();
  for (const item of data.items) if (item.parentId) childCounts.set(item.parentId, (childCounts.get(item.parentId) || 0) + 1);
  const laneIds = new Map(items.map((item) => [item.id, laneOf(item)]));
  const ctx = { byId, childCounts, inLane: (item) => laneIds.get(item.parentId) === laneOf(item) };
  const lanes = LANES.map((lane) => laneSection(lane, items.filter((item) => laneOf(item) === lane.id), ctx));
  return el('div', {}, [
    heading('Board', 'Tasks grouped into Standard, Research, and Papercuts lanes by their kind; the state, priority, and text filters narrow every lane.'),
    form,
    el('div', { class: 'lanes' }, lanes),
  ]);
}

export function renderAttention(items) {
  const list = items.length
    ? el('div', { class: 'panel' }, el('ul', { class: 'item-list' }, items.map((item) => el('li', {}, el('a', { class: 'item-link', href: itemHref(item) }, [
      el('div', { class: 'item-title' }, [el('span', { text: item.title }), el('span', { class: 'muted', text: item.id })]),
      badgeRow(item),
    ])))))
    : el('p', { class: 'empty', text: 'Nothing currently needs validation or review attention.' });
  return el('div', {}, [heading('Validation', 'Tasks surfaced by the canonical attention query: validation, delivered accords, and pending or requested review changes.'), list]);
}

function listValue(values, link = false) {
  if (!values?.length) return 'None';
  if (!link) return values.join(', ');
  return el('span', {}, values.flatMap((value, index) => [index ? ', ' : null, el('a', { href: `#document/${encodeURIComponent(value)}`, text: value })]));
}

// Reference rows arrive from the web detail projection as `referenceLinks`,
// where the server (protocol classification) marks absolute HTTP(S) URLs as
// external. External values keep their exact stored text as the href; every
// other value stays a percent-encoded internal document fragment.
function referenceValueList(links) {
  if (!links?.length) return 'None';
  return el('span', {}, links.flatMap((link, index) => [
    index ? ', ' : null,
    link.external
      ? el('a', { href: link.value, target: '_blank', rel: 'noopener noreferrer', text: link.value })
      : el('a', { href: `#document/${encodeURIComponent(link.value)}`, text: link.value }),
  ]));
}

function detailLink(item, label) {
  return el('a', { href: itemHref(item), text: `${label ? `${label}: ` : ''}${item.id} · ${item.title}` });
}

export function renderDetail(detail, kind = 'document') {
  const resolution = detail.resolution;
  const metadata = [
    ['ID', detail.id], ['Type / role', [detail.type, detail.role].filter(Boolean).join(' / ')], ['Location', detail.location],
    ['State', detail.state || 'Not applicable'], ['Priority', detail.priority || 'Not set'], ['Assignee', detail.assignee || 'Not set'],
    ['Due date', detail.dueDate || 'Not set'], ['Created', detail.createdAt || 'Unknown'], ['Updated', detail.updatedAt || 'Unknown'],
    ['Tags', listValue(detail.tags)], ['Blockers', listValue(detail.blockers, true)], ['References', referenceValueList(detail.referenceLinks)],
    ['Related files', listValue(detail.relatedFiles)],
  ];
  const relationships = [
    detail.parent ? el('li', {}, detailLink(detail.parent, `Parent · ${detail.parentRelationship || 'relationship'}`)) : null,
    ...(detail.children || []).map((child) => el('li', {}, detailLink(child, `Child · ${child.parentRelationship || 'relationship'}`))),
  ].filter(Boolean);
  const sections = [];
  const detailSection = (title, rows) => el('section', { class: 'panel metadata' }, [
    el('h3', { text: title }),
    el('dl', { class: 'detail-grid' }, rows.flatMap(([term, value]) => [el('dt', { text: term }), el('dd', {}, value ?? 'Not recorded')])),
  ]);
  if (detail.accord) sections.push(detailSection('Accord', [
    ['Status', badge('Status', detail.accord.status, statusTone(detail.accord.status))],
    ['Claimed', detail.accord.claimedAt], ['Delivered', detail.accord.deliveredAt],
    ['Summary', detail.accord.summary], ['Deliverables', listValue(detail.accord.deliverables)],
    ['Validation', listValue(detail.accord.validations)], ['Constraints', listValue(detail.accord.constraints)],
    ['Evidence', listValue(detail.accord.evidence)], ['Files changed', listValue(detail.accord.filesChanged)],
    ['Reviewer', detail.accord.reviewer], ['Note', detail.accord.note], ['Reason', detail.accord.reason],
  ]));
  if (detail.validation) sections.push(detailSection('Validation', [
    ['State', badge('State', detail.validation.state, statusTone(detail.validation.state))],
    ['Criterion', detail.validation.criterion], ['Note', detail.validation.note],
    ['Reviewer', detail.validation.reviewer], ['Requested', detail.validation.requestedAt],
  ]));
  if (detail.decision) sections.push(detailSection('Decision record', [
    ['Status', badge('Status', detail.decision.status, statusTone(detail.decision.status))],
    ['Date', detail.decision.date], ['Deciders', listValue(detail.decision.deciders)],
    ['Context', detail.decision.context], ['Consequences', listValue(detail.decision.consequences)],
    ['Alternatives', listValue(detail.decision.alternatives)], ['Supersedes', listValue(detail.decision.supersedes)],
    ['Superseded by', listValue(detail.decision.supersededBy)],
  ]));
  if (resolution) sections.push(el('section', { class: 'panel metadata' }, [
    el('h3', { text: 'Resolution' }),
    el('dl', { class: 'detail-grid' }, [
      el('dt', { text: 'Outcome' }), el('dd', {}, badge('Outcome', resolution.outcome, statusTone(resolution.outcome))),
      el('dt', { text: 'Note' }), el('dd', { text: resolution.note || 'Not recorded' }),
      el('dt', { text: 'Reviewer' }), el('dd', { text: resolution.reviewer || 'Not recorded' }),
      el('dt', { text: 'Files changed' }), el('dd', { text: listValue(resolution.filesChanged) }),
    ]),
  ]));
  const back = kind === 'log' ? '#logs' : kind === 'decision' ? '#decisions' : '#board';
  return el('div', { class: 'detail-page' }, [
    el('div', { class: 'breadcrumbs' }, el('a', { href: back, text: `← Back to ${kind === 'document' ? 'Board' : `${kind}s`}` })),
    el('article', { class: 'panel' }, [
      el('header', { class: 'detail-header' }, [
        el('span', { class: 'card-kicker', text: `${detail.id} · ${detail.role || detail.type}` }),
        el('h2', { id: 'view-title', tabindex: '-1', text: detail.title }),
        badgeRow(detail),
      ]),
      el('div', { class: 'split' }, [
        el('div', { class: 'detail-body' }, [
          el('h3', { text: 'Document body' }),
          detail.bodyHtml ? safeMarkdown(detail.bodyHtml) : el('p', { class: 'empty', text: 'This document has no body.' }),
        ]),
        el('aside', { class: 'metadata', 'aria-label': 'Document metadata' }, [
          el('h3', { text: 'Metadata' }),
          el('dl', { class: 'detail-grid' }, metadata.flatMap(([term, value]) => [el('dt', { text: term }), el('dd', {}, value)])),
          el('h3', { text: 'Relationships' }),
          relationships.length ? el('ul', { class: 'relationship-list' }, relationships) : el('p', { class: 'muted', text: 'No direct parent or children.' }),
        ]),
      ]),
    ]),
    ...sections,
  ]);
}

export function renderLogs(data, query, onSearch) {
  const form = el('form', { class: 'toolbar', role: 'search' }, [
    el('div', { class: 'field grow' }, [el('label', { for: 'log-search', text: 'Search completed work' }), el('input', { id: 'log-search', type: 'search', value: query, placeholder: 'ID, title, summary, or body' })]),
    el('button', { type: 'submit', text: 'Search logs' }),
  ]);
  form.addEventListener('submit', (event) => { event.preventDefault(); onSearch(form.querySelector('input').value); });
  const list = data.items.length ? el('div', { class: 'panel' }, el('ul', { class: 'item-list' }, data.items.map((item) => el('li', {}, el('a', { class: 'item-link', href: `#log/${encodeURIComponent(item.id)}` }, [
    el('div', { class: 'item-title' }, [el('span', { text: item.title }), badge('Outcome', item.outcome, statusTone(item.outcome))]),
    el('p', { class: 'item-summary', text: `${item.id} · ${item.archivedAt || 'Archive date unknown'}${item.summary ? ` · ${item.summary}` : ''}` }),
  ]))))) : el('p', { class: 'empty', text: query ? 'No completed work matches this search.' : 'No completed work is available.' });
  return el('div', {}, [heading('Logs', `${data.total} completed or canceled record${data.total === 1 ? '' : 's'} found.`), form, list]);
}

export function renderRules(categories) {
  const preferred = ['always', 'never', 'prefer', 'context'];
  const names = [...preferred.filter((name) => name in categories), ...Object.keys(categories).filter((name) => !preferred.includes(name))];
  return el('div', {}, [heading('Rules', 'Workspace guidance grouped by its canonical category.'), el('div', { class: 'rule-groups' }, names.map((name) => el('section', { class: 'panel rule-group' }, [
    el('h3', {}, [el('span', { text: name }), el('span', { class: 'count', text: categories[name].length })]),
    categories[name].length ? el('div', {}, categories[name].map((rule) => el('article', { class: 'rule' }, [
      el('p', { text: rule.rule }), el('small', { text: `Rule ${rule.id}${rule.source ? ` · Source: ${rule.source}` : ''}` }),
    ]))) : el('p', { class: 'empty', text: `No ${name} rules.` }),
  ])))]);
}

export function renderDecisions(items) {
  const list = items.length ? el('div', { class: 'panel' }, el('ul', { class: 'item-list' }, items.map((item) => el('li', {}, el('a', { class: 'item-link', href: `#decision/${encodeURIComponent(item.id)}` }, [
    el('div', { class: 'item-title' }, [el('span', { text: item.title }), badge('Status', item.status, statusTone(item.status))]),
    el('p', { class: 'item-summary', text: `${item.id}${item.date ? ` · ${item.date}` : ''}${item.summary ? ` · ${item.summary}` : ''}` }),
  ]))))) : el('p', { class: 'empty', text: 'No active decisions are available.' });
  return el('div', {}, [heading('Decisions', 'ADR-compatible project decisions and their recorded status.'), list]);
}

export function renderHealth(project, warnings, revision) {
  const health = project.health;
  const cards = [
    ['Board documents', health.boardDocuments], ['Active tasks', health.tasks], ['Decisions', health.decisions], ['Completed logs', health.logs],
  ];
  return el('div', {}, [
    heading('Project health', 'Read API metadata, workflow totals, and snapshot warnings.'),
    el('div', { class: 'health-cards' }, cards.map(([label, value]) => el('section', { class: 'panel health-card' }, [el('strong', { text: value }), el('span', { text: label })]))),
    el('div', { class: 'split panel-gap' }, [
      el('section', { class: 'panel panel-pad' }, [el('h3', { text: 'Project' }), el('dl', { class: 'health-grid' }, [
        el('dt', { text: 'Health status' }), el('dd', {}, badge('Status', health.status, statusTone(health.status))),
        el('dt', { text: 'Protocol version' }), el('dd', { text: project.protocolVersion }),
        el('dt', { text: 'Configured states' }), el('dd', { text: project.states.join(', ') || 'None' }),
        el('dt', { text: 'Snapshot revision' }), el('dd', { text: revision }),
        el('dt', { text: 'Access mode' }), el('dd', { text: project.readOnly ? 'Read-only' : 'Unknown' }),
      ])]),
      el('section', { class: 'panel panel-pad' }, [el('h3', { text: `Warnings (${warnings.length})` }), warnings.length ? el('ul', {}, warnings.map((warning) => el('li', { text: warning }))) : el('p', { text: 'No snapshot warnings.' })]),
    ]),
    el('section', { class: 'panel panel-pad panel-gap' }, [el('h3', { text: 'Tasks by workflow state' }), el('dl', { class: 'health-grid' }, Object.entries(health.byState).flatMap(([state, count]) => [el('dt', { text: state }), el('dd', { text: count })]))]),
  ]);
}
