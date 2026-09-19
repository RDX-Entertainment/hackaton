"""Module inventory/abilities integration. Explicit isolated API and database only."""
import concurrent.futures
import json
import os
import subprocess
import urllib.error
import urllib.request

BASE = os.environ.get('HACKATON_TEST_BASE')
assert BASE == 'http://127.0.0.1:18091'
DB = ['podman','exec','hackaton-hardware-test-20260911','psql','-v','ON_ERROR_STOP=1','-U','postgres','-d','hardware_rules_final','-At','-c']
def sql(q): return subprocess.check_output(DB+[q],text=True).strip()
def req(path,body=None,token=None,expected=200,method=None):
    request=urllib.request.Request(BASE+'/api'+path,data=None if body is None else json.dumps(body).encode(),method=method or ('POST' if body is not None else 'GET'),headers={'Content-Type':'application/json',**({'Authorization':'Bearer '+token} if token else {})})
    try: response=urllib.request.urlopen(request)
    except urllib.error.HTTPError as e: response=e
    raw=response.read().decode()
    if expected is not None: assert response.code==expected,(path,response.code,raw)
    return (response.code,json.loads(raw) if raw else None) if expected is None else (json.loads(raw) if raw else None)
def login(name): return req('/auth/login',{'username':name,'password':'HardwareTestOnly2026'})['token']
user,other,admin=login('lmmtxiy3mn'),login('lnmtxiy3mn'),login('hardwaretest')
assert sql('SELECT login FROM users WHERE id=12')=='lmmtxiy3mn'
sql("UPDATE users SET tutorial_completed_at=now() WHERE id=12; DELETE FROM software_running WHERE soft_id IN (SELECT id FROM software WHERE user_id=12 AND soft_name IN ('mw64-cracker','mw64-virus')); DELETE FROM software WHERE user_id=12 AND soft_name IN ('mw64-cracker','mw64-virus'); UPDATE processes SET completed_at=now(),status='cancelled' WHERE creator_id=12 AND completed_at IS NULL")
server=int(sql('SELECT server_id FROM hardware WHERE user_id=12 AND NOT is_npc AND is_gateway'))
pid=int(sql(f"INSERT INTO software(user_id,server_id,soft_name,soft_type,soft_version,soft_size,soft_ram) VALUES(12,{server},'mw64-cracker',1,1,10,10) RETURNING id").splitlines()[0])
sql(f"INSERT INTO software_profiles(software_id,origin,capability_id,effectiveness,penetration,stealth,stability,cpu_need,ram_need,net_need,weight) SELECT {pid},'research',id,50,45,45,55,10,10,1,10 FROM software_capabilities WHERE code='cracker'")
def program(): return next(p for p in req('/software-system/programs',token=user) if p['software_id']==pid)
def inv(): return req('/software-system/workshop',token=user)
def stock(module,tier): return int(sql(f'SELECT COALESCE((SELECT quantity FROM module_stock WHERE user_id=12 AND module_id={module} AND tier={tier}),0)'))
def equip(module,tier=0,slot=1,expected=204,token=user): return req('/software-system/workshop/equip',{'software_id':pid,'slot':slot,'module_id':module,'tier':tier},token,expected)
def remove(slot=1,expected=204): return req('/software-system/workshop/unequip',{'software_id':pid,'slot':slot},user,expected)
sensor=next(m for m in inv()['catalog'] if m['code']=='heuristic_sensor')['id']
purge=next(m for m in inv()['catalog'] if m['code']=='purge_engine')['id']
req('/admin/software-modules',token=user,expected=404)
req('/software-system/workshop',expected=401)
req('/software-lab/programs/'+str(pid),{'stats':program()['stats']},user,400,'PUT')
assert program()['module_slots']==1
print('PASS permissions and manual tuning restricted; module stock now comes from contracts')

sql(f'DELETE FROM module_stock WHERE user_id=12 AND module_id={sensor}; INSERT INTO module_stock VALUES(12,{sensor},0,3)')
def fusion(): return req('/software-system/workshop/fuse',{'module_id':sensor,'tier':0},user,None)[0]
with concurrent.futures.ThreadPoolExecutor(max_workers=2) as executor: outcomes=list(executor.map(lambda _:fusion(),range(2)))
assert sorted(outcomes)==[200,400] and stock(sensor,0)==0 and stock(sensor,1)==1
req('/software-system/workshop/fuse',{'module_id':sensor,'tier':4},user,400)
req('/software-system/workshop/fuse',{'module_id':sensor,'tier':-1},user,400)
print('PASS concurrent fusion cannot double-spend; invalid/max tiers rejected')

equip(sensor,1,2,400) # locked
equip(sensor,1,1,404,other) # ownership
equip(sensor,1)
assert stock(sensor,1)==0 and program()['stats']['detection']==41
assert 'antivirus_scan' in program()['functions']
assert program()['capability']=='cracker'
equip(purge,0,1,400) # occupied
req('/software-system/workshop/fuse',{'module_id':sensor,'tier':1},user,400)
remove()
assert stock(sensor,1)==1 and program()['stats']['detection']==0
assert 'antivirus_scan' not in program()['functions']
assert sql(f'SELECT soft_size,soft_ram FROM software WHERE id={pid}')=='10|10'
print('PASS installation consumes, removal refunds/reverts exactly, capabilities are real grants')

sql(f'UPDATE software SET soft_version=6 WHERE id={pid}')
assert program()['module_slots']==3
equip(sensor,1,2,400) # sensor only 1/3/5/6
equip(sensor,1)
equip(sensor,1,3,400) # max copies even with room
remove()
sql(f'INSERT INTO module_stock VALUES(12,{sensor},4,1) ON CONFLICT(user_id,module_id,tier) DO UPDATE SET quantity=1')
equip(sensor,4)
assert program()['stats']['detection']==96
assert req('/software-system/blueprints/preview',{'name':'base','capability':'antivirus','module_ids':[]},user)['detection']==50
assert program()['stats']['cpu']==138 and program()['stats']['ram']==87
sql(f'INSERT INTO software_running(soft_id,user_id) VALUES({pid},12)')
remove(expected=400)
equip(purge,0,3,400) # running
virus=int(sql(f"INSERT INTO software(user_id,server_id,soft_name,soft_type,soft_version,soft_size,soft_ram,soft_hidden) VALUES(12,{server},'mw64-virus',8,1,1,1,true) RETURNING id").splitlines()[0])
req('/software-system/antivirus',{'software_id':pid,'mode':'clean'},user,400)
operation=req('/software-system/antivirus',{'software_id':pid,'mode':'scan'},user,201)['process_id']
payload=json.loads(sql(f'SELECT process_payload FROM processes WHERE pid={operation}'))
assert next(t for t in payload['targets'] if t['software_id']==virus)['probability']==95
for target in payload['targets']: target['roll']=1
sql(f"UPDATE processes SET process_payload='{json.dumps(payload)}',time_end=now()-interval '1 second' WHERE pid={operation}")
subprocess.run(['distrobox','enter','rust-dev','--','env','DATABASE_URL=postgres://postgres:isolated_hardware_test@127.0.0.1:15439/hardware_rules_final','RUST_LOG=error','/home/raul/Proyectos/hackaton/target/debug/worker','--once'],cwd='/tmp',check=True)
assert sql(f'SELECT soft_hidden FROM software WHERE id={virus}')=='f'
assert req('/processes/'+str(operation),token=user)['status']=='succeeded'
print('PASS Zero-day cracker detection 96 exceeds base antivirus 50; real scan reveals virus, cleaning denied')
sql(f'DELETE FROM software_running WHERE soft_id={pid}')
remove()

template={'code':'mw64_test_module','name':'Módulo prueba 64','config':{'allowed_capabilities':['cracker'],'allowed_slots':[3],'min_level':6,'max_copies':1,'excludes':[],'grants':['antivirus_clean'],'active':True,'collectable':True,'deltas':{'removal':30,'ram':2}}}
catalog=req('/admin/software-modules',token=admin)
existing=next((m for m in catalog if m['code']==template['code']),None)
if existing: template['id']=existing['id']
module=req('/admin/software-modules',template,admin)['id']; template['id']=module
sql(f'INSERT INTO module_stock VALUES(12,{module},0,3) ON CONFLICT(user_id,module_id,tier) DO UPDATE SET quantity=3')
equip(module,0,1,400)
equip(module,0,3)
assert program()['stats']['removal']==30
template['config']['deltas']['removal']=60
req('/admin/software-modules',template,admin)
assert program()['stats']['removal']==30, 'installed configuration is frozen'
remove(3)
equip(module,0,3)
assert program()['stats']['removal']==60
remove(3)
template['config']['active']=False
req('/admin/software-modules',template,admin)
equip(module,0,3,400)
template['config']['active']=True
template['config']['excludes']=['heuristic_sensor']
req('/admin/software-modules',template,admin)
equip(sensor,4)
equip(module,0,3,400)
remove()
equip(module,0,3)
equip(sensor,4,1,400) # symmetric exclusion
remove(3)
template['config']['grants']=['ssh_exploit']
template['config']['excludes']=[]
req('/admin/software-modules',template,admin)
equip(module,0,3)
sql(f'INSERT INTO software_running(soft_id,user_id) VALUES({pid},12)')
attack=req('/software-system/operations',{'capability':'ssh_exploit','target_user_id':13,'software_id':pid},user,201)
attack_payload=json.loads(sql(f"SELECT process_payload FROM processes WHERE pid={attack['process_id']}"))
assert attack_payload['capability']=='ssh_exploit' and attack_payload['action_kind']=='intrude'
assert program()['capability']=='cracker' and 'ssh_exploit' in program()['functions']
sql(f"DELETE FROM software_running WHERE soft_id={pid}; UPDATE processes SET completed_at=now(),status='cancelled' WHERE pid={attack['process_id']}")
remove(3)
print('PASS granted SSH capability launches a real operation from a cracker without changing its primary category')
template['config']['grants']=['nonexistent_function']
req('/admin/software-modules',template,admin,400)
print('PASS admin creation/edit, restrictions, frozen configuration, deactivation and symmetric exclusions')

# A physical module survives deletion of its owning file; never duplicates.
equip(sensor,4)
before=stock(sensor,4)
sql(f'DELETE FROM software WHERE id={pid}')
assert stock(sensor,4)==before+1
assert sql(f'SELECT count(*) FROM program_modules WHERE software_id={pid}')=='0'
sql(f'DELETE FROM software WHERE id={virus}')
print('PASS deleting a file refunds equipped modules exactly once')
print('ALL MODULE WORKSHOP TESTS PASSED')
