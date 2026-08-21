-- Port of NPC::generateNPC (legacy classes/NPC.class.php) and of the census that
-- cron2/restoreNPC.py and cron2/removeExpiredNPC.py kept alive in the legacy game.
--
-- The legacy built every NPC from PHP with a random IP and password, so the world differed
-- on each install and only existed once somebody ran the cron. Here the census is
-- deterministic: every machine on the project shares one world, and the missions the worker
-- replenishes from these NPCs are reproducible.
--
-- npc_type = 70 + level, as in cron/generateMissions.php (71, 72 and 73). Twelve NPCs per
-- level give 132 hirer/victim pairs, well above the 50/30/25 missions the worker asks for.
--
-- Ids are explicit (1001..1036) so npc_info and software_original can reference them
-- without reading the sequence back; setval then moves it clear of that range.
--
-- The legacy also gave each NPC a hardware row and a log row. That is not possible here:
-- both tables reference users(id) and an NPC is not a user. How an NPC owns hardware in
-- this port is still undecided.

INSERT INTO npc (id, npc_type, npc_ip, npc_pass)
SELECT
  1000 + (lvl - 1) * 12 + i,
  (70 + lvl)::smallint,
  ((1000 + (lvl - 1) * 12 + i) * 2654435761::bigint) % 4294967291::bigint,
  substr(md5('npc-seed:' || (1000 + (lvl - 1) * 12 + i)::text), 1, 8)
FROM generate_series(1, 3) AS lvl, generate_series(1, 12) AS i
ON CONFLICT DO NOTHING;

INSERT INTO npc_info (npc_id, name, web)
SELECT
  1000 + (lvl - 1) * 12 + i,
  (ARRAY['Initech', 'Umbrella Data', 'Cyberdyne', 'Tyrell', 'Aperture', 'Massive Dynamic',
         'Soylent', 'Weyland', 'Vehement Capital', 'Globex', 'Hooli', 'Vandelay'])[i]
    || ' ' || (ARRAY['Corp', 'Labs', 'Systems'])[lvl],
  ''
FROM generate_series(1, 3) AS lvl, generate_series(1, 12) AS i
ON CONFLICT DO NOTHING;

-- Original software for each NPC. Types come from PC::getSoftwareName: 1 cracker,
-- 4 firewall. The version scales with the NPC level, which is what separates a level 1
-- target from a level 3 one.
INSERT INTO software_original (npc_id, soft_name, soft_version, soft_size, soft_ram, soft_type)
SELECT
  1000 + (lvl - 1) * 12 + i,
  s.soft_name,
  10 * lvl,
  s.soft_size,
  s.soft_ram,
  s.soft_type
FROM generate_series(1, 3) AS lvl,
     generate_series(1, 12) AS i,
     (VALUES ('Cracker', 200, 100, 1::smallint),
             ('Firewall', 300, 150, 4::smallint)) AS s(soft_name, soft_size, soft_ram, soft_type);

SELECT setval(pg_get_serial_sequence('npc', 'id'), 1100, true);
