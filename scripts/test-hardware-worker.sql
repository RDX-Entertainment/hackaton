-- Run only after test-hardware-bank.mjs, against the same isolated database.
-- Then run worker --once: the process must remain incomplete while its origin is off.
DO $$
DECLARE machine bigint; owner bigint; component bigint;
BEGIN
  SELECT h.server_id,h.user_id INTO machine,owner FROM hardware h
    JOIN users u ON u.id=h.user_id WHERE h.name='Test multipro' AND NOT h.is_npc
    AND u.login LIKE 'ha%' ORDER BY h.server_id DESC LIMIT 1;
  IF machine IS NULL THEN RAISE EXCEPTION 'Missing test fixture'; END IF;
  SELECT c.id INTO component FROM hardware_components c JOIN hardware_parts p ON p.id=c.part_id
    WHERE c.installed_in=machine AND p.kind='cpu' ORDER BY c.id LIMIT 1;
  INSERT INTO processes(creator_id,victim_id,action,soft_id,info,info_str,time_end,time_ideal,origin_server,status,process_payload)
    VALUES(owner,0,27,0,'hardware-wait','hardware-wait',now()-interval '1 second',1,machine,'running',
      jsonb_build_object('component',component,'machine',machine));
  UPDATE hardware_components SET installed_in=NULL WHERE id=component;
  IF (SELECT operational FROM hardware WHERE server_id=machine) THEN RAISE EXCEPTION 'Unbalanced machine stayed on'; END IF;
END $$;
