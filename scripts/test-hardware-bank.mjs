import assert from 'node:assert/strict';

// Explicitly opt in to an isolated local API. Never aim this at a shared game.
const base = process.env.HACKATON_TEST_BASE;
assert(base && /^http:\/\/127\.0\.0\.1:\d+$/.test(base), 'Set HACKATON_TEST_BASE to an isolated local test API');
const suffix = Date.now().toString(36);
let token, adminToken;
async function request(path, body, expected = 200, method = body === undefined ? 'GET' : 'POST', auth = path.startsWith('/admin/') ? adminToken : token) {
  const response = await fetch(`${base}/api${path}`, {
    method, headers: { 'Content-Type': 'application/json', ...(auth ? { Authorization: `Bearer ${auth}` } : {}) },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const raw = await response.text();
  assert.equal(response.status, expected, `${method} ${path}: ${raw}`);
  return raw ? JSON.parse(raw) : null;
}
const admin = await request('/auth/login', { username: 'hardwaretest', password: 'HardwareTestOnly2026' });
adminToken = admin.token;
const owner = await request('/auth/register', { username: `ha${suffix}`, password: 'HardwareTestOnly2026' }, 201, 'POST', null);
token = owner.token;
const onboarding = await request('/onboarding');
assert(onboarding.practice.length > 0);
for (const target of onboarding.practice) {
  assert.equal((await request(`/internet/${target.ip}`)).down, false, 'legacy practice NPC keeps its virtual hardware');
}
console.log('PASS legacy compatibility: new-player practice NPCs remain operational');
await request('/admin/tutorial/config', { enforced: false }, 204, 'PUT');
let accounts = await request('/finances/accounts');
const original = accounts.find(a => a.is_main);
assert(original && accounts.filter(a => a.is_main).length === 1);
assert.equal((await request('/finances/main')).account, original.account);
await request(`/admin/accounts/${original.id}/cash`, { amount: 1000000 });
const banks = await request('/finances/banks');
const bank = banks.find(b => b.id !== original.bank_id) ?? banks[0];
const opened = await request('/finances/accounts', { bank_id: bank.id, password: 'test42' }, 201);
accounts = await request('/finances/accounts');
const alternate = accounts.find(a => a.account === opened.account);
assert(!alternate.is_main, 'opening another account preserves the first main');
await request('/finances/main/transfer', { from_account: original.account, to_account: alternate.account, amount: 50 });
assert((await request('/finances/main/statement')).some(t => t.amount === -50));
await request(`/finances/accounts/${alternate.id}/main`, {}, 204);
assert.equal((await request('/finances/main')).account, alternate.account);
await request('/finances/main/transfer', { from_account: original.account, to_account: alternate.account, amount: 1 }, 401);
await request(`/finances/accounts/${original.id}/main`, {}, 204);
const outsider = await request('/auth/register', { username: `hw${suffix}`, password: 'HardwareTestOnly2026' }, 201, 'POST', null);
await request(`/finances/accounts/${original.id}/main`, {}, 404, 'POST', outsider.token);
await request('/bank/login', { account: original.account, pin: '' }, 401, 'POST', outsider.token);
console.log('PASS bank: first main, always-active owner access, transfer/statement, switching, ownership and PIN isolation');

const input = { kind: 'mobo', name: `Board${suffix}`, power: 0, price: 10, generation: 4, tier: 'domestica',
  cpu_slots: 2, ram_slots: 8, hdd_slots: 2, net_slots: 2, max_cpu: 10000, max_ram: 10000, max_hdd: 10000, max_net: 1000 };
await request('/admin/parts', { ...input, cpu_slots: 4, ram_slots: 2 }, 400);
const board = await request('/admin/parts', input, 200);
const cpu = await request('/admin/parts', { kind: 'cpu', name: `CPU${suffix}`, power: 500, price: 10, tier: 'domestica' }, 200);
const ram = await request('/admin/parts', { kind: 'ram', name: `RAM${suffix}`, power: 256, price: 10, tier: 'domestica', generation: 4 }, 200);
const disk = await request('/admin/parts', { kind: 'hdd', name: `SSD${suffix}`, power: 1000, price: 10, tier: 'domestica' }, 200);
const nic = await request('/admin/parts', { kind: 'net', name: `NIC${suffix}`, power: 10, price: 10, tier: 'domestica' }, 200);
const incompatible = await request('/admin/parts', { kind: 'ram', name: `DDR${suffix}`, power: 256, price: 10, tier: 'gamer', generation: 5 }, 200);
await request('/admin/parts', { ...input, tier: 'industrial' }, 400);
const bought = await request('/hardware/machines', { account: original.account, name: 'Test multipro' });
const machine = bought.machine;
const detail = async () => (await request('/hardware/machines')).find(d => d.machine.id === machine);
const install = async (part) => request('/hardware/shop', { part: part.id, account: original.account, machine });
assert.equal((await detail()).machine.operational, false);
await install(board); await install(cpu); await install(ram); await install(disk);
assert.equal((await detail()).machine.operational, true);
const balance = (await request('/finances/main')).cash;
await request('/hardware/shop', { part: incompatible.id, account: original.account, machine }, 400);
assert.equal((await request('/finances/main')).cash, balance, 'incompatible purchase rolls back its charge');
await install(cpu);
assert((await detail()).machine.disabled_reasons.includes('RAM_UNBALANCED'));
await install(ram);
assert.equal((await detail()).machine.operational, true);
await install(ram);
assert.equal((await detail()).machine.operational, false);
const fourth = await install(ram);
assert.equal((await detail()).machine.operational, true);
await request(`/hardware/components/${fourth.component}/install`, undefined, 200, 'DELETE');
assert.equal((await detail()).machine.operational, false);
await request(`/hardware/components/${fourth.component}/install`, { machine });
assert.equal((await detail()).machine.operational, true);
await request(`/admin/parts/${board.id}`, { ...input, cpu_slots: 4, ram_slots: 8 }, 204, 'PUT');
await install(cpu);
assert.equal((await detail()).machine.operational, false);
await install(cpu);
assert.equal((await detail()).machine.operational, true);
assert.equal((await detail()).slots.cpu_total, 4);
await install(nic);
const running = await detail();
const diskComponent = running.components.find(c => c.kind === 'hdd');
await request(`/hardware/components/${diskComponent.id}/install`, undefined, 200, 'DELETE');
assert((await detail()).machine.disabled_reasons.includes('NO_DISK'));
assert.equal((await request(`/internet/${running.machine.public_ip}`)).down, true);
await request(`/hardware/components/${diskComponent.id}/install`, { machine });
console.log('PASS hardware: 2/4 CPU RAM balance, missing disk disables machine, reactivation, direct install, compatibility and tier validation');

const gateway = (await request('/hardware/machines')).find(d => d.machine.public_ip === owner.user.game_ip);
const gatewayCpu = gateway.components.find(c => c.kind === 'cpu');
await request(`/hardware/components/${gatewayCpu.id}/install`, undefined, 200, 'DELETE');
const blocked = await request('/processes', { action: 25 }, 400);
assert.equal(blocked.code, 'HARDWARE_DISABLED', 'disabled hardware cannot execute tasks through the API');
await request(`/hardware/components/${gatewayCpu.id}/install`, { machine: gateway.machine.id });
console.log('PASS execution: server-side process guard rejects disabled hardware and restoration is possible');

assert(gateway.machine.is_gateway);
const gatewayNic = gateway.components.find(c => c.kind === 'net');
const nextNic = (await detail()).components.find(c => c.kind === 'net');
const setGateway = (id, expected = 204, auth = token) => request(`/hardware/machines/${id}/gateway`, {}, expected, 'PUT', auth);
await setGateway(machine, 404, outsider.token);
await setGateway(999999999, 404);
await request(`/hardware/components/${nextNic.id}/install`, undefined, 200, 'DELETE');
assert.equal((await setGateway(machine, 400)).code, 'GATEWAY_NEEDS_NIC');
await request(`/hardware/components/${nextNic.id}/install`, { machine });
await request(`/hardware/components/${diskComponent.id}/install`, undefined, 200, 'DELETE');
assert.equal((await setGateway(machine, 400)).code, 'GATEWAY_MUST_BE_ACTIVE');
await request(`/hardware/components/${diskComponent.id}/install`, { machine });
assert.equal((await request(`/hardware/components/${gatewayNic.id}/install`, undefined, 400, 'DELETE')).code, 'GATEWAY_NEEDS_NIC');
assert.equal((await request(`/hardware/components/${gatewayNic.id}/install`, { machine }, 400)).code, 'GATEWAY_NEEDS_NIC', 'moving the last NIC also protects the principal');
await setGateway(machine);
let principals = (await request('/hardware/machines')).filter(d => d.machine.is_gateway);
assert.deepEqual(principals.map(d => d.machine.id), [machine]);
assert.equal((await request('/me')).game_ip, principals[0].machine.public_ip);
await setGateway(machine); // idempotent
await request(`/hardware/components/${gatewayNic.id}/install`, undefined, 200, 'DELETE');
assert.equal((await request('/hardware/machines')).find(d => d.machine.id === gateway.machine.id).machine.public_ip, null);
assert.equal((await request(`/hardware/components/${nextNic.id}/install`, undefined, 400, 'DELETE')).code, 'GATEWAY_NEEDS_NIC');
await request(`/hardware/components/${gatewayNic.id}/install`, { machine: gateway.machine.id });
await Promise.all([setGateway(gateway.machine.id), setGateway(machine)]);
principals = (await request('/hardware/machines')).filter(d => d.machine.is_gateway);
assert.equal(principals.length, 1);
assert.equal((await request('/me')).game_ip, principals[0].machine.public_ip);
await setGateway(gateway.machine.id);
const race = await Promise.all([
  fetch(`${base}/api/hardware/machines/${machine}/gateway`, { method: 'PUT', headers: { Authorization: `Bearer ${token}` } }),
  fetch(`${base}/api/hardware/components/${nextNic.id}/install`, { method: 'DELETE', headers: { Authorization: `Bearer ${token}` } }),
]);
assert.deepEqual(race.map(r => r.status).sort(), race[0].status === 204 ? [204, 400] : [200, 400], 'promotion/removal cannot leave a principal without its NIC');
if (race[1].status === 200) await request(`/hardware/components/${nextNic.id}/install`, { machine });
await setGateway(gateway.machine.id);
console.log('PASS gateway: ownership, active/NIC validation, atomic selection, account IP sync, NIC removal/move guard and concurrent changes');

const before = await request('/hardware/machines/quote');
const root = await request('/hardware/switches', { account: original.account, name: 'Switch prueba' });
assert.equal((await setGateway(root.machine, 400)).code, 'GATEWAY_COMPUTER_ONLY');
assert.equal((await request('/hardware/machines/quote')).machines, before.machines);
assert(!(await request('/hardware')).some(h => h.id === root.machine), 'switch is not a compute target');
await request('/hardware/shop', { part: cpu.id, account: original.account, machine: root.machine }, 400);
await request('/hardware/links', { from: root.machine, to: machine });
await request('/hardware/links', { from: root.machine, to: machine }, 400, 'POST', outsider.token);
const leaves = [];
for (let n = 0; n < 8; n++) leaves.push(await request('/hardware/switches', { account: original.account, name: `Leaf ${n}` }));
for (let n = 0; n < 6; n++) await request('/hardware/links', { from: root.machine, to: leaves[n].machine });
const concurrent = await Promise.all(leaves.slice(6).map(leaf => fetch(`${base}/api/hardware/links`, {
  method: 'POST', headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token}` },
  body: JSON.stringify({ from: root.machine, to: leaf.machine }),
})));
assert.deepEqual(concurrent.map(r => r.status).sort(), [200, 400], 'concurrent links respect eight physical ports');
const rootDetail = (await request('/hardware/machines')).find(d => d.machine.id === root.machine);
assert.equal(rootDetail.machine.node_kind, 'switch');
assert.equal(rootDetail.machine.switch_ports, 8);
const slotCount = (await request('/hardware/machines/quote')).machines;
await request(`/hardware/machines/${root.machine}`, undefined, 204, 'DELETE');
assert.equal((await request('/hardware/machines/quote')).machines, slotCount);
assert(!(await request('/hardware/links')).some(l => l.from_server === root.machine || l.to_server === root.machine));
console.log('PASS switches: distinct LAN entity, eight ports, concurrent limits, ownership, no CPU slots consumed, removable links');
console.log(`UI fixture: ${owner.user.login}`);
