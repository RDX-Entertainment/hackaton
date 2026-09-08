import assert from 'node:assert/strict';
import { randomBytes } from 'node:crypto';
import { execFileSync } from 'node:child_process';

const base = process.env.HACKATON_TEST_BASE;
assert(base, 'Define HACKATON_TEST_BASE apuntando a una instancia de pruebas vacía.');
const suffix = randomBytes(4).toString('hex');
const password = randomBytes(18).toString('base64');

async function request(path, token, body, expected = 200) {
  const response = await fetch(`${base}/api${path}`, {
    method: body === undefined ? 'GET' : 'POST',
    headers: { 'Content-Type': 'application/json', ...(token ? { Authorization: `Bearer ${token}` } : {}) },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const text = await response.text();
  assert.equal(response.status, expected, `${path}: ${text}`);
  return text ? JSON.parse(text) : null;
}

async function finish(id, token) {
  for (let attempt = 0; attempt < 40; attempt++) {
    const process = await request(`/processes/${id}`, token);
    if (process.completed) return process;
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }
  throw new Error(`La tarea ${id} no terminó`);
}

const first = await request('/auth/register', null, { username: `sw${suffix}`, password }, 201);
const second = await request('/auth/register', null, { username: `swb${suffix}`, password }, 201);
const token = first.token;
assert(token && second.token);
const input = { name: 'á'.repeat(25), capability: 'ssh_exploit', module_ids: [] };
const modules = await request('/software-system/research/modules', token);
const compatible = modules.filter((module) => !module.capability || module.capability === input.capability);
if (compatible.length) {
  const modular = { ...input, name: 'Con módulo', module_ids: [compatible[0].id] };
  const projected = await request('/software-system/blueprints/preview', token, modular);
  const saved = await request('/software-system/blueprints', token, modular, 201);
  assert.deepEqual(saved.projected, projected);
}

const before = await request('/software-system/blueprints', token);
await request('/software-system/blueprints', token, { ...input, module_ids: [1, 1] }, 400);
await request('/software-system/blueprints', token, { ...input, module_ids: [99999999] }, 400);
assert.equal((await request('/software-system/blueprints', token)).length, before.length);
console.log('OK: módulos inválidos no dejan diseños parciales');

const preview = await request('/software-system/blueprints/preview', token, input);
const blueprint = await request('/software-system/blueprints', token, input, 201);
assert.deepEqual(blueprint.projected, preview);
await request('/software-system/compile', second.token, { blueprint_id: blueprint.id }, 404);
console.log('OK: vista previa, diseño Unicode y propiedad del diseño');

const attempts = await Promise.all(Array.from({ length: 3 }, async () => {
  const response = await fetch(`${base}/api/software-system/compile`, {
    method: 'POST', headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token}` },
    body: JSON.stringify({ blueprint_id: blueprint.id }),
  });
  return { status: response.status, body: await response.json() };
}));
assert.equal(attempts.filter((attempt) => attempt.status === 201).length, 1);
assert.equal(attempts.filter((attempt) => attempt.status === 400).length, 2);
const queued = attempts.find((attempt) => attempt.status === 201).body;
const active = await request('/processes', token);
assert.equal(active.filter((row) => row.action === 91 && !row.completed).length, 1);
assert.equal((await finish(queued.process_id, token)).status, 'succeeded');
console.log('OK: peticiones concurrentes crean una sola compilación y el worker la materializa');

let programs = await request('/software-system/programs', token);
const compiled = programs.find((program) => program.name === input.name);
assert(compiled);
assert.deepEqual(compiled.stats, preview);
assert(compiled.server_id > 0 && compiled.resources.cpu > 0);
const files = await request('/software', token);
assert.equal(files.find((file) => file.id === compiled.software_id).server_id, compiled.server_id);
await request('/software-system/operations', token, { capability: input.capability, target_user_id: second.user.id, software_id: compiled.software_id }, 400);
console.log('OK: atributos congelados, disco concreto y rechazo de programa sin instalar');

const installed = await request('/processes', token, { action: 13, software_id: compiled.software_id }, 201);
assert.equal((await finish(installed.process_id ?? installed.id, token)).status, 'succeeded');
programs = await request('/software-system/programs', token);
assert.equal(programs.find((program) => program.software_id === compiled.software_id).running, true);
await request('/software-system/operations', second.token, { capability: input.capability, target_user_id: first.user.id, software_id: compiled.software_id }, 400);
await request('/software-system/operations', token, { capability: input.capability, target_user_id: first.user.id, software_id: compiled.software_id }, 400);
const operation = await request('/software-system/operations', token, { capability: input.capability, target_user_id: second.user.id, software_id: compiled.software_id }, 201);
assert.deepEqual(operation.attacker, preview);
const outcome = await finish(operation.process_id, token);
assert(['succeeded', 'failed'].includes(outcome.status));
assert(outcome.result_code && outcome.result_message);
console.log('OK: instalación, aislamiento entre jugadores, selección explícita y resultado de operación');

if (process.env.HACKATON_TEST_SSH) {
  const sql = (query) => execFileSync('ssh', ['-o', 'BatchMode=yes', process.env.HACKATON_TEST_SSH,
    'docker exec -i hackaton-software-test-postgres-1 psql -U software_test -d software_test -v ON_ERROR_STOP=1 -At'],
  { input: query, encoding: 'utf8' }).trim();
  const defense = await request('/software-system/blueprints', second.token, { name: 'Defensa', capability: 'firewall_ssh', module_ids: [] }, 201);
  const build = await request('/software-system/compile', second.token, { blueprint_id: defense.id }, 201);
  await finish(build.process_id, second.token);
  const defender = (await request('/software-system/programs', second.token)).find((program) => program.name === 'Defensa');
  const load = await request('/processes', second.token, { action: 13, software_id: defender.software_id }, 201);
  await finish(load.process_id ?? load.id, second.token);
  const attack = () => request('/software-system/operations', token, { capability: input.capability, target_user_id: second.user.id, software_id: compiled.software_id }, 201);
  const protectedTarget = await attack();
  assert.equal(protectedTarget.defender.resistance, defender.stats.resistance);
  await finish(protectedTarget.process_id, token);
  const otherServer = Number(sql(`INSERT INTO hardware(user_id,name,cpu,hdd,ram,net,is_gateway) VALUES(${second.user.id},'Otra maquina',1000,1000,1024,1,false) RETURNING server_id;`).split('\n')[0]);
  assert(otherServer > 0);
  sql(`UPDATE software SET server_id=${otherServer} WHERE id=${defender.software_id};`);
  const unprotectedTarget = await attack();
  assert.equal(unprotectedTarget.defender.resistance, 0);
  await finish(unprotectedTarget.process_id, token);
  sql(`UPDATE software SET server_id=${defender.server_id} WHERE id=${defender.software_id};`);
  console.log('OK: la defensa de otra máquina no protege el objetivo');
}
console.log('Todas las pruebas integradas de software pasaron.');
