#!/usr/bin/env python3
"""End-to-end contract reward check. Only accepts an explicitly local isolated API."""
import json
import os
import subprocess
import time
import urllib.error
import urllib.request

BASE = os.environ.get("HACKATON_TEST_BASE", "")
CONTAINER = os.environ.get("HACKATON_TEST_DB", "hackaton-gameplay-test-20260919")
assert BASE.startswith("http://127.0.0.1:"), "Use an isolated API bound to 127.0.0.1"

def api(path, body=None, token=None, expected=200):
    data = None if body is None else json.dumps(body).encode()
    request = urllib.request.Request(
        BASE + "/api" + path,
        data=data,
        headers={"content-type": "application/json", **({"authorization": "Bearer " + token} if token else {})},
    )
    try:
        with urllib.request.urlopen(request) as response:
            raw = response.read()
            assert response.status == expected, (path, response.status, raw)
            return json.loads(raw) if raw else None
    except urllib.error.HTTPError as error:
        raw = error.read()
        assert error.code == expected, (path, error.code, raw)
        return json.loads(raw) if raw else None

def sql(statement):
    output = subprocess.check_output([
        "docker", "exec", CONTAINER, "psql", "-U", "postgres", "-d", "hackaton_gameplay",
        "-qAt", "-v", "ON_ERROR_STOP=1", "-c", statement,
    ], text=True).strip()
    return output.splitlines()[0] if output else ""

suffix = str(int(time.time() * 1000))[-9:]
session = api("/auth/register", {"username": "m" + suffix, "password": "MissionLoop2026"}, expected=201)
token = session["token"]
user = session["user"]["id"]
target_id, target_ip, target_server = sql(
    "SELECT n.id,n.npc_ip,h.server_id FROM npc n JOIN hardware h ON h.user_id=n.id AND h.is_npc "
    "WHERE n.npc_type=71 ORDER BY n.id LIMIT 1"
).split("|")
before = api("/finances/main", token=token)["cash"]
sql(f"UPDATE users SET tutorial_completed_at=now() WHERE id={user}")
mission = int(sql(
    f"INSERT INTO missions(type,status,hirer,victim,prize,user_id,level) "
    f"VALUES(1,1,{target_ip}+1,{target_ip},375,NULL,1) RETURNING id"
))
accepted = api(f"/missions/{mission}", {"action": "accept"}, token=token)
assert accepted == {"id": mission, "action": "accept"}, accepted
sql(
    "INSERT INTO processes(creator_id,victim_id,victim_server,action,info,info_str,time_end,time_ideal,origin_server) "
    f"SELECT {user},{target_id},{target_server},11,'npc','npc',now()-interval '1 second',1,server_id "
    f"FROM hardware WHERE user_id={user} AND is_gateway AND NOT is_npc"
)

deadline = time.time() + 12
completed = None
while time.time() < deadline:
    completed = next((row for row in api("/missions", token=token) if row["id"] == mission), None)
    if completed and completed["status"] == 3:
        break
    time.sleep(.5)
assert completed and completed["status"] == 3, completed
assert completed["objective"] == "gain_access"
assert completed["target_name"]
assert completed["reward_module_name"]
assert completed["reward_module_tier"] == 0
assert api("/finances/main", token=token)["cash"] == before + 375
workshop = api("/software-system/workshop", token=token)
assert any(item["quantity"] == 1 for item in workshop["stock"])
assert sql(f"SELECT mission_count||'|'||money_earned FROM users_stats WHERE uid={user}") == "1|375"
print("PASS mission loop: accept, access, payment, stats and module loot")
