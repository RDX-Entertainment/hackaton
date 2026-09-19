"""Compile and use a game port scanner, only in the isolated local fixture DB."""
import json, os, subprocess, urllib.request, urllib.error
BASE=os.environ.get('HACKATON_TEST_BASE')
assert BASE=='http://127.0.0.1:18091'
DB=['podman','exec','hackaton-hardware-test-20260911','psql','-v','ON_ERROR_STOP=1','-U','postgres','-d','hardware_rules_final','-At','-c']
def sql(q):return subprocess.check_output(DB+[q],text=True).strip()
def request(path,body=None,expected=200,method=None,auth=None):
 r=urllib.request.Request(BASE+'/api'+path,data=None if body is None else json.dumps(body).encode(),method=method or ('POST' if body is not None else 'GET'),headers={'Content-Type':'application/json',**({'Authorization':'Bearer '+auth} if auth else {})})
 try: result=urllib.request.urlopen(r)
 except urllib.error.HTTPError as e:result=e
 raw=result.read().decode();assert result.code==expected,(path,result.code,raw)
 return json.loads(raw) if raw else None
token=request('/auth/login',{'username':'lmmtxiy3mn','password':'HardwareTestOnly2026'})['token']
assert sql('SELECT login FROM users WHERE id=12')=='lmmtxiy3mn'
sql('UPDATE certifications SET cert_level=3 WHERE user_id=12')
server=int(sql('SELECT server_id FROM hardware WHERE user_id=12 AND NOT is_npc AND is_gateway'))
target,ip=map(int,sql('SELECT server_id,public_ip FROM hardware WHERE user_id=13 AND NOT is_npc AND is_gateway').split('|'))
def api(path,body=None,expected=200,method=None):return request(path,body,expected,method,token)
def finish(pid,win=True):
 payload=json.loads(sql(f'SELECT process_payload FROM processes WHERE pid={pid}'))
 for phase in payload.get('phase_plan',{}).values():phase['roll']=1 if win else 100
 frozen=json.dumps(payload).replace("'","''")
 sql(f"UPDATE processes SET process_payload='{frozen}'::jsonb,time_end=now()-interval '1 second' WHERE pid={pid}")
 subprocess.run(['distrobox','enter','rust-dev','--','env','DATABASE_URL=postgres://postgres:isolated_hardware_test@127.0.0.1:15439/hardware_rules_final','RUST_LOG=error','/home/raul/Proyectos/hackaton/target/debug/worker','--once'],cwd='/tmp',check=True)
 return api(f'/processes/{pid}')

cap=next(c for c in api('/software-system/capabilities') if c['code']=='port_scan')
assert cap['role']=='utility' and cap['granted_permissions']==[]
blueprint=next((b for b in api('/software-system/blueprints') if b['name']=='port66scanner'),None)
if not blueprint:blueprint=api('/software-system/blueprints',{'name':'port66scanner','capability':'port_scan','module_ids':[]},201)
pid=api('/software-system/compile',{'blueprint_id':blueprint['id']},201)['process_id']
assert finish(pid)['status']=='succeeded'
program=next(p for p in api('/software-system/programs') if p['name']=='port66scanner')
sid=program['software_id']
assert sql(f'SELECT soft_type FROM software WHERE id={sid}')=='3'
assert 'port_scan' in program['functions']
api('/internet/hack',{'ip':ip,'kind':'scan','software_id':sid},400)
install=api('/processes',{'action':13,'software_id':sid},201)['id']
assert finish(install)['status']=='succeeded'
assert next(p for p in api('/software-system/programs') if p['software_id']==sid)['running']
api('/processes',{'action':15,'victim_id':13},400)
print('PASS scanner can be designed, compiled as type 3 and activated; legacy scan bypass blocked')

sql(f'DELETE FROM internet_scan WHERE user_id=12 AND ip={ip}')
before=api(f'/internet/{ip}')['permissions']
scan=api('/internet/hack',{'ip':ip,'kind':'scan','software_id':sid},201)
assert scan['attacker_stats']['effectiveness']==program['stats']['effectiveness']
assert scan['attacker_version'] is None
api('/internet/hack',{'ip':ip,'kind':'scan','software_id':sid},400)
assert not api(f'/internet/{ip}')['scanned']
assert finish(scan['process_id'],False)['status']=='failed'
assert not api(f'/internet/{ip}')['scanned']
assert before==api(f'/internet/{ip}')['permissions']
scan=api('/internet/hack',{'ip':ip,'kind':'scan','software_id':sid},201)
assert finish(scan['process_id'])['status']=='succeeded'
assert api(f'/internet/{ip}')['scanned']
assert before==api(f'/internet/{ip}')['permissions']
assert int(sql(f'SELECT victim_server FROM processes WHERE pid={scan["process_id"]}'))==target
print('PASS scan uses frozen attributes, exact node, success/failure, duplicate protection; never grants access')

npc_ip=int(sql("SELECT h.public_ip FROM hardware h JOIN npc_purpose e ON e.npc_id=h.user_id WHERE h.is_npc AND e.owner_id=12 AND e.kind='practice' LIMIT 1"))
scan=api('/internet/hack',{'ip':npc_ip,'kind':'scan','software_id':sid},201)
assert finish(scan['process_id'])['status']=='succeeded'
assert api(f'/internet/{npc_ip}')['scanned']
assert sql(f'SELECT target_is_npc FROM game_evidence WHERE process_id={scan["process_id"]}')=='t'
print('PASS scanner works on own practice NPC with correctly scoped evidence')

legacy=int(sql(f"INSERT INTO software(user_id,server_id,soft_name,soft_type,soft_version,soft_size,soft_ram) VALUES(12,{server},'port66legacy',3,1,10,10) RETURNING id").splitlines()[0])
assert 'port_scan' in next(p for p in api('/software-system/programs') if p['software_id']==legacy)['functions']
print('PASS legacy PortScan copies gain a compatible profile without free software copies')
