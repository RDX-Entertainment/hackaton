import assert from 'node:assert/strict';
const base = process.env.HACKATON_TEST_BASE;
assert(base && /^http:\/\/127\.0\.0\.1:\d+$/.test(base), 'Use an isolated local test API');
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
token = (await call('/auth/login', { username: 'lmmtxiy3mn', password: 'HardwareTestOnly2026' })).token;
const modules = (await call('/software-system/research/modules')).filter(module => module.code.startsWith('tactical_'));
assert.equal(modules.length, 6);
const changes = {
  tactical_penetration: { penetration: 12, effectiveness: 8 },
  tactical_resistance: { resistance: 12, hardening: 8 },
  tactical_evasion: { stealth: 12, anti_forensics: 8 },
  tactical_detection: { detection: 12, forensics: 8 },
  tactical_persistence: { persistence: 12, manipulation: 8 },
  tactical_recovery: { removal: 12, integrity: 8 },
};
const capabilities = await call('/software-system/capabilities');
const run = Date.now().toString(36);
for (const capability of capabilities) {
  const plain = await call('/software-system/blueprints/preview', { name: 'Plain', capability: capability.code, module_ids: [] });
  for (const module of modules) {
    assert.equal(module.capability, null);
    const projected = await call('/software-system/blueprints/preview', { name: 'Combat', capability: capability.code, module_ids: [module.id] });
    for (const key of Object.keys(plain)) {
      const delta = changes[module.code][key] ?? ({ cpu: 8, ram: 6, weight: 4 }[key] ?? 0);
      assert.equal(projected[key], ['cpu', 'ram', 'net', 'weight'].includes(key) ? plain[key] + delta : Math.min(100, plain[key] + delta), `${capability.code}/${module.code}/${key}`);
    }
  }
  const ids = modules.slice(0, 3).map(module => module.id);
  const combined = await call('/software-system/blueprints/preview', { name: 'Combined', capability: capability.code, module_ids: ids });
  const saved = await call('/software-system/blueprints', { name: `${run}-${capabilities.indexOf(capability)}`, capability: capability.code, module_ids: ids });
  assert.deepEqual(saved.projected, combined);
  const restored = (await call('/software-system/blueprints')).find(blueprint => blueprint.id === saved.id);
  assert.deepEqual(restored.projected, combined);
}
console.log(`PASS 6 tactical modules × ${capabilities.length} capacities: all 12 combat attributes, resource costs and saved/reloaded combined stats`);
