import assert from 'node:assert/strict';
const base = process.env.HACKATON_TEST_BASE;
assert(base && /^http:\/\/127\.0\.0\.1:\d+$/.test(base), 'Use an isolated local API');
let admin, player;
async function call(path, body, method = body === undefined ? 'GET' : 'POST', token = path.startsWith('/admin/') ? admin?.token : player?.token) {
  const response = await fetch(`${base}/api${path}`, { method, headers: { 'Content-Type': 'application/json', ...(token ? { Authorization: `Bearer ${token}` } : {}) }, body: body === undefined ? undefined : JSON.stringify(body) });
  const raw = await response.text();
  assert(response.ok, `${method} ${path}: ${response.status} ${raw}`);
  return raw ? JSON.parse(raw) : null;
}
admin = await call('/auth/login', { username: 'hardwaretest', password: 'HardwareTestOnly2026' });
const original = await call('/admin/tutorial');
const edit = (step) => call(`/admin/tutorial/steps/${step.id}`, step, 'PUT');
try {
  for (const step of original.steps) await edit({ ...step, active: step.fact === 'programs' });
  await call('/admin/tutorial/config', { enforced: true }, 'PUT');
  player = await call('/auth/register', { username: `once${Date.now().toString(36)}`, password: 'HardwareTestOnly2026' });
  let state = await call('/onboarding');
  assert.equal(state.completed, false);
  assert.equal(state.current, 'compile');
  const tryLoad = async () => {
    const response = await fetch(`${base}/api/processes`, { method: 'POST', headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${player.token}` }, body: JSON.stringify({ action: 13 }) });
    return { status: response.status, body: await response.json() };
  };
  assert.equal((await tryLoad()).body.code, 'TUTORIAL_LOCKED');
  const program = await call(`/admin/users/${player.user.id}/software`, { name: 'Tutorial once', soft_type: 1, version: 1, size: 1, ram: 1, running: true });
  // No onboarding poll between earning and deleting: completion is recorded by the event.
  await call(`/admin/users/${player.user.id}/software/${program.id}/running`, { running: false }, 'PUT');
  await call(`/admin/users/${player.user.id}/software/${program.id}`, undefined, 'DELETE');
  for (const step of original.steps) await edit(step);
  state = await call('/onboarding');
  assert.equal(state.completed, true);
  assert.equal(state.current, null);
  assert.equal(state.rules, null);
  assert(state.steps.every(step => step.done));
  assert.notEqual((await tryLoad()).body.code, 'TUTORIAL_LOCKED', 'server guard never re-locks a graduate');
  const again = await call('/auth/login', { username: player.user.login, password: 'HardwareTestOnly2026' });
  assert.equal((await call('/onboarding', undefined, 'GET', again.token)).completed, true, 'graduation survives a new session and definition changes');
  console.log('PASS tutorial: completion survives unloading/deleting its only program before a poll, restored tutorial steps and a new login');
} finally {
  for (const step of original.steps) await edit(step);
  await call('/admin/tutorial/config', { enforced: original.enforced }, 'PUT');
}
