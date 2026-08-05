const status = document.querySelector('#status');
const title = document.querySelector('#project-title');
const health = document.querySelector('#health');

fetch('/api/v1/project', { headers: { Accept: 'application/json' }, cache: 'no-store' })
  .then((response) => response.json().then((body) => ({ response, body })))
  .then(({ response, body }) => {
    if (!response.ok || !body.ok) throw new Error(body.error?.message || 'Project read failed');
    title.textContent = body.data.title;
    status.textContent = `Connected at revision ${body.revision}.`;
    const entries = [
      ['Protocol', body.data.protocolVersion],
      ['Board documents', body.data.health.boardDocuments],
      ['Logs', body.data.health.logs],
      ['Warnings', body.warnings.length],
    ];
    health.replaceChildren(...entries.flatMap(([term, value]) => {
      const dt = document.createElement('dt');
      const dd = document.createElement('dd');
      dt.textContent = term;
      dd.textContent = String(value);
      return [dt, dd];
    }));
  })
  .catch((error) => {
    status.textContent = `Could not load the project: ${error.message}`;
  });
