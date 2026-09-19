import assert from 'node:assert/strict';

const base = process.env.HACKATON_TEST_BASE;
assert(base && /^http:\/\/127\.0\.0\.1:\d+$/.test(base), 'Use an explicitly isolated local API');
const password = 'HardwareTestOnly2026';
let token;
async function call(path, body) {
  const response = await fetch(`${base}/api${path}`, {
    method: body === undefined ? 'GET' : 'POST',
    headers: { 'Content-Type': 'application/json', ...(token ? { Authorization: `Bearer ${token}` } : {}) },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const result = await response.json();
  assert(response.ok, `${path}: ${response.status} ${JSON.stringify(result)}`);
  return result;
}
// The isolated fixture already has the initial universal research unlocks.
token = (await call('/auth/login', { username: 'lmmtxiy3mn', password })).token;
const modules = await call('/software-system/research/modules');
const basicCodes = ['runtime_stability', 'memory_optimizer', 'execution_optimizer'];
const universal = modules.filter(module => basicCodes.includes(module.code));
assert.equal(universal.length, 3);
const capabilities = await call('/software-system/capabilities');
assert(capabilities.length >= 14);
for (const capability of capabilities) {
  const ids = universal.map(module => module.id);
  const plain = await call('/software-system/blueprints/preview', { name: 'Plain', capability: capability.code, module_ids: [] });
  const boosted = await call('/software-system/blueprints/preview', { name: 'Universal', capability: capability.code, module_ids: ids });
  assert.equal(boosted.stability, Math.min(100, plain.stability + 6));
  assert.equal(boosted.cpu, plain.cpu - 6);
  assert.equal(boosted.ram, plain.ram - 2);
  assert.equal(boosted.weight, plain.weight + 6);
  const saved = await call('/software-system/blueprints', { name: `Test ${capability.code}`.slice(0, 25), capability: capability.code, module_ids: ids });
  assert.deepEqual(saved.modules, ids);
  assert.deepEqual(saved.projected, boosted);
}
// A new account also receives the universals, without an extra collection step.
token = (await call('/auth/register', { username: `mc${Date.now().toString(36)}`, password })).token;
assert.equal((await call('/software-system/research/modules')).filter(module => basicCodes.includes(module.code)).length, 3);
console.log(`PASS ${capabilities.length} capabilities: real preview and saved blueprint with universal modules for existing/new accounts`);
