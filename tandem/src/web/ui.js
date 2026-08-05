const view = document.querySelector('#view');

export function el(tag, attrs = {}, children = []) {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(attrs)) {
    if (value == null || value === false) continue;
    if (key === 'class') node.className = value;
    else if (key === 'text') node.textContent = String(value);
    else if (key === 'html') node.innerHTML = value;
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

function depthFor(item, byId) {
  let depth = 0;
  let parent = item.parentId;
  const seen = new Set([item.id]);
  while (parent && byId.has(parent) && !seen.has(parent) && depth < 2) {
    seen.add(parent);
    depth += 1;
    parent = byId.get(parent).parentId;
  }
  return depth;
}

function orderedItems(items) {
  const byId = new Map(items.map((item) => [item.id, item]));
  const children = new Map();
  for (const item of items) {
    const key = item.parentId && byId.has(item.parentId) ? item.parentId : null;
    if (!children.has(key)) children.set(key, []);
    children.get(key).push(item);
  }
  const result = [];
  const append = (parent) => {
    for (const item of children.get(parent) || []) {
      result.push(item);
      append(item.id);
    }
  };
  append(null);
  for (const item of items) if (!result.includes(item)) result.push(item);
  return { items: result, byId };
}

function taskCard(item, byId) {
  const relationship = item.parentRelationship ? `${item.parentRelationship} · ` : '';
  return el('article', { class: 'task-card', 'data-role': item.role, 'data-depth': depthFor(item, byId) }, [
    el('div', { class: 'card-kicker', text: `${relationship}${item.id}` }),
    el('h4', {}, el('a', { href: itemHref(item), text: item.title })),
    item.assignee ? el('div', { class: 'card-meta', text: `Assignee: ${item.assignee}` }) : null,
    badgeRow(item, false),
  ]);
}

export function renderBoard(data, filters, onFilter) {
  const states = data.states || [];
  const items = data.items.filter((item) => {
    const query = filters.query.trim().toLowerCase();
    return !query || `${item.id} ${item.title} ${(item.tags || []).join(' ')}`.toLowerCase().includes(query);
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
  const columns = states
    .filter((state) => !filters.state || state === filters.state)
    .map((state) => {
      const stateItems = items.filter((item) => item.state === state);
      const ordered = orderedItems(stateItems);
      return el('section', { class: 'board-column', 'aria-labelledby': `state-${state}` }, [
        el('h3', { class: 'column-heading', id: `state-${state}` }, [el('span', { text: state }), el('span', { class: 'count', text: stateItems.length })]),
        stateItems.length
          ? el('div', { class: 'card-list' }, ordered.items.map((item) => taskCard(item, ordered.byId)))
          : el('p', { class: 'empty', text: 'No documents in this state.' }),
      ]);
    });
  const unconfigured = items.filter((item) => !states.includes(item.state));
  if (unconfigured.length && !filters.state) {
    const ordered = orderedItems(unconfigured);
    columns.push(el('section', { class: 'board-column', 'aria-labelledby': 'state-other' }, [
      el('h3', { class: 'column-heading', id: 'state-other' }, [el('span', { text: 'Other / no state' }), el('span', { class: 'count', text: unconfigured.length })]),
      el('div', { class: 'card-list' }, ordered.items.map((item) => taskCard(item, ordered.byId))),
    ]));
  }
  return el('div', {}, [
    heading('Board', 'Configured workflow states with canonical roles and relationships from the read API.'),
    form,
    columns.length ? el('div', { class: 'board', style: `--state-count:${Math.max(1, columns.length)}` }, columns) : el('p', { class: 'empty', text: 'No configured state matches this filter.' }),
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

function detailLink(item, label) {
  return el('a', { href: itemHref(item), text: `${label ? `${label}: ` : ''}${item.id} · ${item.title}` });
}

export function renderDetail(detail, kind = 'document') {
  const completion = detail.completion;
  const metadata = [
    ['ID', detail.id], ['Type / role', [detail.type, detail.role].filter(Boolean).join(' / ')], ['Location', detail.location],
    ['State', detail.state || 'Not applicable'], ['Priority', detail.priority || 'Not set'], ['Assignee', detail.assignee || 'Not set'],
    ['Due date', detail.dueDate || 'Not set'], ['Created', detail.createdAt || 'Unknown'], ['Updated', detail.updatedAt || 'Unknown'],
    ['Tags', listValue(detail.tags)], ['Blockers', listValue(detail.blockers, true)], ['References', listValue(detail.references, true)],
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
    ['Assignee', detail.accord.assignee], ['Claimed', detail.accord.claimedAt], ['Delivered', detail.accord.deliveredAt],
    ['Summary', detail.accord.summary], ['Deliverables', listValue(detail.accord.deliverables)],
    ['Validation', listValue(detail.accord.validations)], ['Constraints', listValue(detail.accord.constraints)],
    ['Evidence', listValue(detail.accord.evidence)], ['Files changed', listValue(detail.accord.filesChanged)],
    ['Reviewer', detail.accord.reviewer], ['Note', detail.accord.note], ['Reason', detail.accord.reason],
  ]));
  if (detail.review) sections.push(detailSection('Review', [
    ['Status', badge('Status', detail.review.status, statusTone(detail.review.status))],
    ['Reviewer', detail.review.reviewer], ['Requested', detail.review.requestedAt],
    ['Decided', detail.review.decidedAt], ['Note', detail.review.note],
  ]));
  if (detail.decision) sections.push(detailSection('Decision record', [
    ['Status', badge('Status', detail.decision.status, statusTone(detail.decision.status))],
    ['Date', detail.decision.date], ['Deciders', listValue(detail.decision.deciders)],
    ['Context', detail.decision.context], ['Consequences', listValue(detail.decision.consequences)],
    ['Alternatives', listValue(detail.decision.alternatives)], ['Supersedes', listValue(detail.decision.supersedes)],
    ['Superseded by', listValue(detail.decision.supersededBy)],
  ]));
  if (completion) sections.push(el('section', { class: 'panel metadata' }, [
    el('h3', { text: 'Completion' }),
    el('dl', { class: 'detail-grid' }, [
      el('dt', { text: 'Outcome' }), el('dd', {}, badge('Outcome', completion.outcome, statusTone(completion.outcome))),
      el('dt', { text: 'Summary' }), el('dd', { text: completion.summary || 'Not recorded' }),
      el('dt', { text: 'Validation' }), el('dd', { text: completion.validation || 'Not recorded' }),
      el('dt', { text: 'Reviewer' }), el('dd', { text: completion.reviewer || 'Not recorded' }),
      el('dt', { text: 'Files changed' }), el('dd', { text: listValue(completion.filesChanged) }),
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
          detail.bodyHtml ? el('div', { class: 'prose', html: detail.bodyHtml }) : el('p', { class: 'empty', text: 'This document has no body.' }),
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
    el('p', { class: 'item-summary', text: `${item.id} · ${item.completedAt || 'Completion date unknown'}${item.summary ? ` · ${item.summary}` : ''}` }),
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
    el('div', { class: 'split', style: 'margin-top:1rem' }, [
      el('section', { class: 'panel panel-pad' }, [el('h3', { text: 'Project' }), el('dl', { class: 'health-grid' }, [
        el('dt', { text: 'Health status' }), el('dd', {}, badge('Status', health.status, statusTone(health.status))),
        el('dt', { text: 'Protocol version' }), el('dd', { text: project.protocolVersion }),
        el('dt', { text: 'Configured states' }), el('dd', { text: project.states.join(', ') || 'None' }),
        el('dt', { text: 'Snapshot revision' }), el('dd', { text: revision }),
        el('dt', { text: 'Access mode' }), el('dd', { text: project.readOnly ? 'Read-only' : 'Unknown' }),
      ])]),
      el('section', { class: 'panel panel-pad' }, [el('h3', { text: `Warnings (${warnings.length})` }), warnings.length ? el('ul', {}, warnings.map((warning) => el('li', { text: warning }))) : el('p', { text: 'No snapshot warnings.' })]),
    ]),
    el('section', { class: 'panel panel-pad', style: 'margin-top:1rem' }, [el('h3', { text: 'Tasks by workflow state' }), el('dl', { class: 'health-grid' }, Object.entries(health.byState).flatMap(([state, count]) => [el('dt', { text: state }), el('dd', { text: count })]))]),
  ]);
}
