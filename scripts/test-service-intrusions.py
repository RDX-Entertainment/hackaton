"""Only the disposable localhost fixture DB/API. No production writes."""
import json
import os
import subprocess
import urllib.request
import urllib.error
from concurrent.futures import ThreadPoolExecutor

BASE = os.environ.get('HACKATON_TEST_BASE')
assert BASE == 'http://127.0.0.1:18091'
DB = ['podman','exec','hackaton-hardware-test-20260911','psql','-v','ON_ERROR_STOP=1','-U','postgres','-d','hardware_rules_final','-At','-c']
def sql(query):
    return subprocess.check_output(DB+[query],text=True).strip()
def request(path, body=None, token=None, expected=200, method=None):
    req=urllib.request.Request(BASE+'/api'+path,data=None if body is None else json.dumps(body).encode(),method=method or ('GET' if body is None else 'POST'),headers={'Content-Type':'application/json',**({'Authorization':'Bearer '+token} if token else {})})
    try: response=urllib.request.urlopen(req)
    except urllib.error.HTTPError as err: response=err
    raw=response.read().decode()
    assert response.code==expected,(path,response.code,raw)
    try: return json.loads(raw)
    except json.JSONDecodeError: return raw

token=request('/auth/login',{'username':'lmmtxiy3mn','password':'HardwareTestOnly2026'})['token']
assert sql('SELECT login FROM users WHERE id=12')=='lmmtxiy3mn'
server=int(sql('SELECT server_id FROM hardware WHERE user_id=12 AND NOT is_npc AND is_gateway'))
target, ip=map(int,sql('SELECT server_id,public_ip FROM hardware WHERE user_id=13 AND NOT is_npc AND is_gateway').split('|'))
sql("UPDATE certifications SET cert_level=3 WHERE user_id=12; UPDATE users SET tutorial_completed_at=now() WHERE id=12")
sql("UPDATE processes SET status='cancelled',completed_at=now() WHERE creator_id=12 AND completed_at IS NULL")
sql(f"DELETE FROM machine_accesses WHERE attacker_id=12 AND target_ip={ip}; DELETE FROM internet_access WHERE user_id=12 AND ip={ip}; DELETE FROM internet_scan WHERE user_id=12 AND ip={ip}")

def program(cap):
    name='intr65-'+cap
    existing=sql(f"SELECT id FROM software WHERE user_id=12 AND NOT is_npc AND soft_name='{name}'")
    if existing: return int(existing)
    sid=int(sql(f"INSERT INTO software(user_id,server_id,soft_name,soft_type,soft_version,soft_size,soft_ram) VALUES(12,{server},'{name}',99,1,10,10) RETURNING id").splitlines()[0])
    sql(f"INSERT INTO software_profiles(software_id,origin,capability_id,effectiveness,penetration,stealth,manipulation,stability,cpu_need,ram_need,net_need,weight) SELECT {sid},'research',id,95,95,95,95,80,10,10,1,10 FROM software_capabilities WHERE code='{cap}'; INSERT INTO software_running(soft_id,user_id) VALUES({sid},12)")
    return sid

programs={cap:program(cap) for cap in ['web_exploit','ftp_exploit','ssh_exploit','cracker','web_manipulator']}
def intrude(cap, addr=ip, expected=201, sid=None):
    return request('/internet/intrusions',{'ip':addr,'capability':cap,'software_id':sid or programs.get(cap)},token,expected)
def finish(pid, win=True):
    payload=json.loads(sql(f'SELECT process_payload FROM processes WHERE pid={pid}'))
    for phase in payload.get('phase_plan',{}).values(): phase['roll']=1 if win else 100
    value=json.dumps(payload).replace("'","''")
    sql(f"UPDATE processes SET process_payload='{value}'::jsonb,time_end=now()-interval '1 second' WHERE pid={pid}")
    subprocess.run(['distrobox','enter','rust-dev','--','env','DATABASE_URL=postgres://postgres:isolated_hardware_test@127.0.0.1:15439/hardware_rules_final','RUST_LOG=error','/home/raul/Proyectos/hackaton/target/debug/worker','--once'],cwd='/tmp',check=True)
    return request(f'/processes/{pid}',token=token)
def permissions(addr=ip): return request(f'/internet/{addr}',token=token)['permissions']

assert any(c['code']=='web_exploit' for c in request('/software-system/capabilities',token=token))
assert 'SCAN_REQUIRED' in str(intrude('web_exploit',expected=400))
sql(f'INSERT INTO internet_scan(user_id,ip) VALUES(12,{ip}) ON CONFLICT DO NOTHING')
intrude('virus_infect',expected=400)
intrude('web_exploit',sid=programs['ftp_exploit'],expected=400)
request('/processes',{'action':16,'victim_id':13},token,400)
assert finish(intrude('web_exploit')['process_id'])['status']=='succeeded'
assert set(permissions())=={'read_web','write_web'}
request(f'/internet/{ip}/software',token=token,expected=400)
request(f'/internet/{ip}/log',token=token,expected=400)
request(f'/internet/{ip}/software',{'software_id':programs['cracker'],'direction':'upload'},token,400)
request(f'/internet/{ip}/software/{programs["cracker"]}',token=token,method='DELETE',expected=400)
request('/processes',{'action':2,'victim_id':13,'software_id':programs['cracker']},token,400)
print('PASS web-only access cannot read/write storage, logs, or bypass via legacy actions')

before=request(f'/software-system/accesses/{ip}/web',token=token)
request(f'/software-system/accesses/{ip}/web',{'content':'<script>alert(1)</script>'},token,400,'PUT')
request(f'/software-system/accesses/{ip}/web',{'content':'<img src="https://example.com/x">'},token,400,'PUT')
content='<style>body{background:#102020;color:#aaffdd}</style><h1>intr65 published</h1>'
active_manipulators=sql("SELECT r.soft_id FROM software_running r JOIN software s ON s.id=r.soft_id WHERE s.user_id=12 AND NOT s.is_npc AND program_has_function(s.id,'web_manipulator')").splitlines()
try:
    for sid in active_manipulators: sql(f'DELETE FROM software_running WHERE soft_id={int(sid)}')
    write=request(f'/software-system/accesses/{ip}/web',{'content':content},token,201,'PUT')
    assert write['content']==content and write['seconds']==0
    receipt=request(f'/processes/{write["process_id"]}',token=token)
    assert receipt['completed'] and receipt['status']=='succeeded' and receipt['result_code']=='WEB_PUBLISHED'
finally:
    for sid in active_manipulators: sql(f'INSERT INTO software_running(soft_id,user_id) VALUES({int(sid)},12) ON CONFLICT DO NOTHING')
assert request(f'/internet/{ip}',token=token)['web']==content
after=request(f'/software-system/accesses/{ip}/web',token=token)
assert after['content']==content and after['revision']>before['revision']
assert sql(f'SELECT count(*) FROM node_web_revisions WHERE process_id={write["process_id"]}')=='1'
assert sql(f'SELECT content FROM node_web_revisions WHERE process_id={write["process_id"]}')==before['content']
observer=request('/auth/login',{'username':'lnmtxiy3mn','password':'HardwareTestOnly2026'})['token']
assert request(f'/internet/{ip}',token=observer)['web']==content
# Concurrent saves must preserve both revisions, and a worker pass must not replay them.
def save_parallel(html):
    return request(f'/software-system/accesses/{ip}/web',{'content':html},token,201,'PUT')
with ThreadPoolExecutor(max_workers=2) as executor:
    writes=list(executor.map(save_parallel,['<p>hola</p>','<p>segunda publicación</p>']))
assert sorted(w['revision'] for w in writes)==[after['revision']+1,after['revision']+2]
latest=max(writes,key=lambda w:w['revision'])
assert request(f'/internet/{ip}',token=observer)['web']==latest['content']
assert finish(latest['process_id'])['status']=='succeeded'
assert request(f'/software-system/accesses/{ip}/web',token=token)['revision']==latest['revision']
save_parallel(content)
print('PASS immediate publication without manipulator, public observer, atomic revisions/concurrent saves, no worker replay')

sql(f"UPDATE machine_accesses SET expires_at=now()-interval '1 second' WHERE attacker_id=12 AND target_ip={ip}")
request(f'/software-system/accesses/{ip}/web',{'content':'<p>expired</p>'},token,400,'PUT')
assert request(f'/internet/{ip}',token=observer)['web']==content
sql(f'UPDATE machine_accesses SET expires_at=NULL WHERE attacker_id=12 AND target_ip={ip}')
print('PASS expired access cannot publish or change public content')

assert finish(intrude('ftp_exploit')['process_id'])['status']=='succeeded'
assert set(permissions())=={'read_web','write_web','read_files','write_files'}
request(f'/internet/{ip}/software',token=token)
request(f'/internet/{ip}/log',token=token,expected=400)
assert finish(intrude('ssh_exploit')['process_id'])['status']=='succeeded'
assert 'modify_logs' in permissions() and 'write_web' in permissions()
assert finish(intrude('cracker')['process_id'])['status']=='succeeded'
assert finish(intrude('web_exploit')['process_id'])['status']=='succeeded'
assert sql(f'SELECT scope FROM machine_accesses WHERE attacker_id=12 AND target_ip={ip}')=='root'
assert 'modify_logs' in permissions()
print('PASS independent permissions accumulate; service exploit never downgrades root')

# Revoke and acquire only FTP: old root rights must not resurrect.
sql(f"UPDATE machine_accesses SET status='revoked' WHERE attacker_id=12 AND target_ip={ip}")
assert finish(intrude('ftp_exploit')['process_id'])['status']=='succeeded'
assert set(permissions())=={'read_files','write_files'}
request(f'/software-system/accesses/{ip}/web',token=token,expected=400)
request(f'/software-system/accesses/{ip}/web',{'content':'<h1>No</h1>'},token,400,'PUT')

# An arbitrary file id on another node cannot be used through this node's access.
other=int(sql("INSERT INTO hardware(user_id,name,cpu,ram,hdd,net) VALUES(13,'intr65 other',800,256,1000,10) RETURNING server_id").splitlines()[0])
file=int(sql(f"INSERT INTO software(user_id,server_id,soft_name,soft_type,soft_version,soft_size,soft_ram) VALUES(13,{other},'intr65 private',30,1,1,1) RETURNING id").splitlines()[0])
assert file not in [f['id'] for f in request(f'/internet/{ip}/software',token=token)]
request(f'/internet/{ip}/software',{'software_id':file,'direction':'download'},token,404)
request(f'/internet/{ip}/software/{file}',token=token,expected=404,method='DELETE')
print('PASS revoked permissions stay revoked; storage operations are node-scoped')

# Own practice NPC shares no web state with the player-id namespace.
npc_row=sql("SELECT h.server_id,h.public_ip,h.user_id FROM hardware h JOIN npc_purpose e ON e.npc_id=h.user_id WHERE h.is_npc AND e.owner_id=12 AND e.kind='practice' LIMIT 1")
assert npc_row, 'Fixture owner needs a practice NPC'
npc_server,npc_ip,npc_id=map(int,npc_row.split('|'))
sql(f'INSERT INTO internet_scan(user_id,ip) VALUES(12,{npc_ip}) ON CONFLICT DO NOTHING')
assert finish(intrude('web_exploit',npc_ip)['process_id'])['status']=='succeeded'
npc_content='<h1>intr65 NPC only</h1>'
write=request(f'/software-system/accesses/{npc_ip}/web',{'content':npc_content},token,201,'PUT')
assert request(f'/processes/{write["process_id"]}',token=token)['status']=='succeeded'
assert request(f'/internet/{npc_ip}',token=token)['web']==npc_content
assert request(f'/internet/{ip}',token=token)['web']==content
assert sql(f'SELECT target_is_npc FROM game_evidence WHERE process_id={write["process_id"]}')=='t'
print('PASS NPC HTML and evidence use correct domain and node; player site unchanged')

# Leave convenient full access for read-only browser checks.
assert finish(intrude('cracker')['process_id'])['status']=='succeeded'
print(f'PASS all service integration checks; browser target={ip}, npc={npc_ip}')
