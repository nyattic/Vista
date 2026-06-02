// Turn a raw backend/IPC error into a short, user-facing message. Backend errors
// arrive as the serialized AppError Display string (e.g. "http 404 for …",
// "network error: …"); this maps the known shapes to friendly text and avoids
// surfacing internal paths or stack detail for anything unrecognized.
export function friendlyError(e: unknown): string {
  const raw = typeof e === 'string' ? e : e instanceof Error ? e.message : String(e);
  const msg = raw.toLowerCase();

  if (msg.startsWith('network error')) {
    return 'Network error — check your connection and try again.';
  }
  if (msg.includes('http 404') || msg.startsWith('not found')) {
    return 'Not found.';
  }
  if (msg.includes('http 429')) {
    return 'Too many requests — wait a moment and try again.';
  }
  const httpMatch = raw.match(/http (\d{3})/i);
  if (httpMatch) {
    return `Server error (${httpMatch[1]}) — please try again later.`;
  }
  if (
    msg.startsWith('decode error') ||
    msg.includes('invalid json') ||
    msg.includes('html response')
  ) {
    return "Couldn't read the response from the server.";
  }
  if (msg.startsWith('database error')) {
    return 'A local storage error occurred.';
  }
  if (msg.startsWith('io error')) {
    return 'A file system error occurred.';
  }
  if (msg.includes('too large')) {
    return 'The response was too large to load.';
  }

  // Short validation-style messages from the backend are already safe to show.
  if (raw && raw.length <= 80 && !raw.includes('/') && !raw.includes('\\')) {
    return raw;
  }
  return 'Something went wrong. Please try again.';
}
