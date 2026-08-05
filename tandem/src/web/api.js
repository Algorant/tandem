const API_ROOT = '/api/v1';

export class ApiError extends Error {
  constructor(message, code = 'request_failed', status = 0) {
    super(message);
    this.name = 'ApiError';
    this.code = code;
    this.status = status;
  }
}

async function request(path) {
  const response = await fetch(`${API_ROOT}${path}`, {
    headers: { Accept: 'application/json' },
    cache: 'no-store',
  });
  let envelope;
  try {
    envelope = await response.json();
  } catch {
    throw new ApiError('The server returned an unreadable response.', 'invalid_response', response.status);
  }
  if (!response.ok || !envelope.ok) {
    throw new ApiError(
      envelope.error?.message || `Request failed with status ${response.status}.`,
      envelope.error?.code,
      response.status,
    );
  }
  return envelope;
}

function queryString(values) {
  const query = new URLSearchParams();
  for (const [key, value] of Object.entries(values)) {
    if (value) query.set(key, value);
  }
  const encoded = query.toString();
  return encoded ? `?${encoded}` : '';
}

export const api = {
  project: () => request('/project'),
  board: (filters = {}) => request(`/board${queryString(filters)}`),
  attention: () => request('/attention'),
  document: (id) => request(`/documents/${encodeURIComponent(id)}`),
  logs: (query = '') => request(`/logs${queryString({ query, limit: '200' })}`),
  log: (id) => request(`/logs/${encodeURIComponent(id)}`),
  rules: () => request('/rules'),
  decisions: () => request('/decisions'),
  decision: (id) => request(`/decisions/${encodeURIComponent(id)}`),
};
