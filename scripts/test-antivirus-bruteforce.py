"""Integration test against the explicitly isolated localhost API/PG container only.
Run after starting that API with migration 63. Does not contact production.
"""
import json
import os
import subprocess
import urllib.error
import urllib.request

BASE = os.environ.get('HACKATON_TEST_BASE')
assert BASE == 'http://127.0.0.1:18091', 'Use the isolated test API on port 18091'
DB = ['podman', 'exec', 'hackaton-hardware-test-20260911', 'psql', '-v', 'ON_ERROR_STOP=1', '-U', 'postgres', '-d', 'hardware_rules_final', '-At', '-c']

def sql(query):
    return subprocess.check_output(DB + [query], text=True).strip()

def request(path, body=None, token=None, expected=200, method=None):
    req = urllib.request.Request(BASE + '/api' + path, data=None if body is None else json.dumps(body).encode(),
        method=method or ('GET' if body is None else 'POST'),
        headers={'Content-Type': 'application/json', **({'Authorization': 'Bearer ' + token} if token else {})})
    try:
        response = urllib.request.urlopen(req)
    except urllib.error.HTTPError as error:
        response = error
    raw = response.read().decode()
    assert response.code == expected, (path, response.code, raw)
    if not raw: return None
    try: return json.loads(raw)
    except json.JSONDecodeError: return raw

token = request('/auth/login', {'username': 'lmmtxiy3mn', 'password': 'HardwareTestOnly2026'})['token']
other = request('/auth/login', {'username': 'lnmtxiy3mn', 'password': 'HardwareTestOnly2026'})['token']
assert sql("SELECT login FROM users WHERE id=12") == 'lmmtxiy3mn'
server = int(sql('SELECT server_id FROM hardware WHERE user_id=12 AND NOT is_npc AND is_gateway'))
foreign = int(sql('SELECT server_id FROM hardware WHERE user_id=13 AND NOT is_npc AND is_gateway'))
sql('UPDATE users SET tutorial_completed_at=now() WHERE id IN (12,13)')

capabilities = request('/software-system/capabilities', token=token)
assert len(capabilities) >= 13
assert [c['code'] for c in capabilities if c['code'].startswith('antivirus')] == ['antivirus']
programs = request('/software-system/programs', token=token)
detector = next(p for p in programs if p['name'] == 'av63detector')
cleaner = next(p for p in programs if p['name'] == 'av63cleaner')
assert detector['capability'] == cleaner['capability'] == 'antivirus'
assert (detector['stats']['detection'], detector['stats']['removal']) == (90,10)
assert (cleaner['stats']['detection'], cleaner['stats']['removal']) == (10,90)
assert all(b['capability'] == 'antivirus' for b in request('/software-system/blueprints', token=token) if b['name'] in ['antivirus_detect','antivirus_remove'])
for alias in ['antivirus_detect', 'antivirus_remove']:
    request('/software-system/blueprints/preview', {'name':'compat', 'capability':alias, 'module_ids':[]}, token)
print('PASS migration: one antivirus, both profiles/designs retained, old capability aliases accepted')

# Repeatable fixtures, never user data outside this isolated test account and exact names.
fixture_names = "'av63-original','av63-infection','av63-legacy-virus','av63-hidden-notes','av63-foreign-virus','av63-other-node','av63-cracker','av63-compiled'"
sql(f"DELETE FROM software_running WHERE soft_id IN (SELECT id FROM software WHERE user_id IN (12,13) AND NOT is_npc AND soft_name IN ({fixture_names})); DELETE FROM software WHERE user_id IN (12,13) AND NOT is_npc AND soft_name IN ({fixture_names}); DELETE FROM software_blueprints WHERE user_id=12 AND name='av63-compiled'")
sql("UPDATE processes SET completed_at=now(),status='cancelled' WHERE creator_id=12 AND action IN (11,96,97) AND completed_at IS NULL")

def make_software(name, kind, owner=12, machine=server, hidden=False, source=0, capability=None, running=False):
    soft = int(sql(f"INSERT INTO software(user_id,server_id,soft_name,soft_type,soft_version,soft_size,soft_ram,soft_hidden,original_from) VALUES({owner},{machine},'{name}',{kind},1,10,10,{str(hidden).lower()},{source}) RETURNING id").splitlines()[0])
    if capability:
        sql(f"INSERT INTO software_profiles(software_id,origin,capability_id,stealth,persistence,cpu_need,ram_need,net_need,weight) SELECT {soft},'research',id,50,50,10,10,1,10 FROM software_capabilities WHERE code='{capability}'")
    if running:
        sql(f'INSERT INTO software_running(soft_id,user_id) VALUES({soft},{owner})')
    return soft

for p in [detector, cleaner]:
    sql(f"INSERT INTO software_running(soft_id,user_id) VALUES({p['software_id']},12) ON CONFLICT DO NOTHING")
own_source = make_software('av63-original',99,capability='virus_infect',running=True)
virus = make_software('av63-infection',99,hidden=True,source=own_source,capability='virus_infect',running=True)
legacy = make_software('av63-legacy-virus',8,hidden=True,running=True)
benign = make_software('av63-hidden-notes',30,hidden=True)
outsider = make_software('av63-foreign-virus',8,owner=13,machine=foreign,hidden=True)
second = int(sql("INSERT INTO hardware(user_id,name,cpu,ram,hdd,net) VALUES(12,'AV test node',800,256,1000,10) RETURNING server_id").splitlines()[0])
other_node = make_software('av63-other-node',8,machine=second,hidden=True)

def antivirus(program, mode, expected=201, auth=token):
    return request('/software-system/antivirus', {'software_id':program['software_id'],'mode':mode}, auth, expected)

def payload(pid):
    return json.loads(sql(f'SELECT process_payload FROM processes WHERE pid={pid}'))

def finish(pid, roll=50):
    data = payload(pid)
    if 'targets' in data:
        for target in data['targets']:
            target['roll'] = roll
    else:
        for phase in data['phase_plan'].values():
            phase['roll'] = roll
    # Control randomness/time only in this isolated test, not in the application.
    sql(f"UPDATE processes SET process_payload='{json.dumps(data)}'::jsonb,time_end=now()-interval '1 second' WHERE pid={pid}")
    subprocess.run(['distrobox','enter','rust-dev','--','env',
        'DATABASE_URL=postgres://postgres:isolated_hardware_test@127.0.0.1:15439/hardware_rules_final',
        'RUST_LOG=error','/home/raul/Proyectos/hackaton/target/debug/worker','--once'], cwd='/tmp', check=True)
    return request(f'/processes/{pid}',token=token)

antivirus(detector,'scan',400,other)
request('/software-system/antivirus', {'software_id':detector['software_id'],'mode':'delete_everything'},token,400)
for action in [7,11,96,97]:
    request('/processes',{'action':action,'software_id':detector['software_id']},token,400)
early = antivirus(cleaner,'clean')['process_id']
assert payload(early)['targets'] == []
assert finish(early)['status'] == 'succeeded'
assert sql(f'SELECT soft_hidden FROM software WHERE id={virus}') == 't'

weak_scan = antivirus(cleaner,'scan')['process_id']
assert next(t for t in payload(weak_scan)['targets'] if t['software_id']==virus)['probability'] == 18
assert finish(weak_scan)['status'] == 'succeeded'
assert sql(f'SELECT soft_hidden FROM software WHERE id={virus}') == 't'
scan = antivirus(detector,'scan')['process_id']
antivirus(detector,'scan',400)
targets = payload(scan)['targets']
assert {t['software_id'] for t in targets} == {virus,legacy}
assert next(t for t in targets if t['software_id']==virus)['probability'] == 82
assert not any(p['software_id']==virus for p in request('/software-system/programs',token=token))
assert finish(scan)['status'] == 'succeeded'
assert sql(f'SELECT soft_hidden FROM software WHERE id={virus}') == 'f'
assert all(sql(f'SELECT soft_hidden FROM software WHERE id={i}') == 't' for i in [benign,outsider,other_node])
low_clean = antivirus(detector,'clean')['process_id']
assert next(t for t in payload(low_clean)['targets'] if t['software_id']==virus)['probability'] == 18
assert finish(low_clean)['status'] == 'failed'
assert sql(f'SELECT count(*) FROM software WHERE id={virus}') == '1'
high_clean = antivirus(cleaner,'clean')['process_id']
sql(f"UPDATE software_profiles SET removal=0 WHERE software_id={cleaner['software_id']}")
assert finish(high_clean)['status'] == 'succeeded', 'queued stats remain frozen'
assert sql(f'SELECT count(*) FROM software WHERE id IN ({virus},{legacy})') == '0'
assert sql(f'SELECT count(*) FROM software_running WHERE soft_id IN ({virus},{legacy})') == '0'
assert sql(f'SELECT count(*) FROM software WHERE id IN ({benign},{outsider},{other_node},{own_source})') == '4'
sql(f"UPDATE software_profiles SET removal=90 WHERE software_id={cleaner['software_id']}")
print('PASS antivirus: separate detection/cleaning rolls, hidden/visible gating, frozen stats, local ownership, safe cleanup')

cracker = make_software('av63-cracker',1,running=True)
npc, npc_server, ip = map(int, sql('SELECT h.user_id,h.server_id,h.public_ip FROM hardware h JOIN npc_purpose p ON p.npc_id=h.user_id WHERE h.is_npc AND p.owner_id=12 AND p.kind=\'practice\' ORDER BY h.server_id LIMIT 1').split('|'))
sql(f'DELETE FROM machine_accesses WHERE attacker_id=12 AND target_ip={ip}; DELETE FROM internet_access WHERE user_id=12 AND ip={ip}; DELETE FROM compromised_lists WHERE user_id=12 AND ip={ip}')
hasher = int(sql(f"INSERT INTO software(user_id,is_npc,server_id,soft_name,soft_type,soft_version,soft_size,soft_ram) VALUES({npc},true,{npc_server},'av63-hasher',2,999,10,10) RETURNING id").splitlines()[0])
sql(f'INSERT INTO software_running(soft_id,user_id,is_npc) VALUES({hasher},{npc},true)')
first = request('/internet/hack', {'ip':ip,'kind':'bf'},token,201)
assert first['attacker_stats'] and first['attacker_version'] is None and first['defender_version'] is None
pid = first['process_id']
frozen = payload(pid)
assert frozen['target_is_npc'] is True
assert sql(f'SELECT origin_server,victim_server FROM processes WHERE pid={pid}') == f'{server}|{npc_server}'
request('/internet/hack', {'ip':ip,'kind':'bf'},token,400)
assert finish(pid,100)['status'] == 'failed'
assert sql(f'SELECT count(*) FROM internet_access WHERE user_id=12 AND ip={ip}') == '0'
sql(f'UPDATE software SET soft_version=99999 WHERE id={cracker}')
second_attack = request('/internet/hack',{'ip':ip,'kind':'bf'},token,201)
assert {k:v['probability'] for k,v in payload(second_attack['process_id'])['phase_plan'].items()} == {k:v['probability'] for k,v in frozen['phase_plan'].items()}, 'versions do not change combat'
assert finish(second_attack['process_id'],100)['status'] == 'failed'
sql(f'UPDATE software_profiles SET effectiveness=100,penetration=100,stealth=100 WHERE software_id={cracker}')
strong = request('/internet/hack',{'ip':ip,'kind':'bf'},token,201)
plan = payload(strong['process_id'])['phase_plan']
assert plan['execution']['probability'] > frozen['phase_plan']['execution']['probability']
assert plan['penetration']['probability'] > frozen['phase_plan']['penetration']['probability']
assert plan['detection']['probability'] < frozen['phase_plan']['detection']['probability']
assert finish(strong['process_id'],1)['status'] == 'succeeded'
assert sql(f'SELECT target_is_npc,target_user_id FROM machine_accesses WHERE attacker_id=12 AND target_ip={ip}') == f't|{npc}'
assert sql(f'SELECT target_is_npc FROM game_evidence WHERE process_id={strong["process_id"]}') == 't'
print('PASS brute force: stats not versions, failed access blocked, duplicate rejected, NPC/node access and evidence correct')

collision = int(sql(f"INSERT INTO game_evidence(process_id,target_user_id,target_is_npc,action_code,real_identity,visible_to_target) VALUES({strong['process_id']},13,true,'npc-id-collision','user:12',true) RETURNING id").splitlines()[0])
assert not any(e['id']==collision for e in request('/software-system/evidence',token=other))
assert any(e['id']==collision for e in request('/software-system/evidence',token=token))
print('PASS evidence: NPC/player numeric-ID collisions do not leak attack records')

blueprint = request('/software-system/blueprints',{'name':'av63-compiled','capability':'antivirus','module_ids':[]},token,201)
compile_job = request('/software-system/compile',{'blueprint_id':blueprint['id']},token,201)
sql(f"UPDATE processes SET time_end=now()-interval '1 second' WHERE pid={compile_job['process_id']}")
subprocess.run(['distrobox','enter','rust-dev','--','env','DATABASE_URL=postgres://postgres:isolated_hardware_test@127.0.0.1:15439/hardware_rules_final','RUST_LOG=error','/home/raul/Proyectos/hackaton/target/debug/worker','--once'],cwd='/tmp',check=True)
assert request(f'/processes/{compile_job["process_id"]}',token=token)['status'] == 'succeeded'
assert sql("SELECT s.soft_type,c.code FROM software s JOIN software_profiles p ON p.software_id=s.id JOIN software_capabilities c ON c.id=p.capability_id WHERE s.user_id=12 AND s.soft_name='av63-compiled'") == '7|antivirus'
print('PASS unified antivirus compiles into an actual type-7 program')
