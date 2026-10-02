/** URL shortcuts are available only on the local development server. */
export function localQaRun(search: string, development: boolean, hostname: string) {
  if (!development || !['localhost', '127.0.0.1', '0.0.0.0', '[::1]', '::1'].includes(hostname.toLowerCase())) return null;
  const params = new URLSearchParams(search);
  if (params.get('qa') !== '1') return null;
  const value = params.get('lvl');
  const level = value !== null && /^\d+$/.test(value) ? Number(value) : 1;
  return { level: Number.isInteger(level) && level >= 1 && level <= 25 ? level : 1 };
}
