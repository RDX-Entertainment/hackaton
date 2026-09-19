-- Only run against the isolated test DB. All fixture changes roll back.
BEGIN;
DO $$
DECLARE player bigint; program bigint; machine bigint; npc_owner bigint; npc_program bigint;
BEGIN
  SELECT id INTO STRICT player FROM users WHERE login LIKE 'once%' ORDER BY id DESC LIMIT 1;
  SELECT server_id INTO STRICT machine FROM hardware WHERE user_id=player AND NOT is_npc AND is_gateway;
  DELETE FROM internet_access WHERE user_id=player;
  DELETE FROM machine_accesses WHERE attacker_id=player;
  DELETE FROM software_running WHERE user_id=player AND NOT is_npc;
  DELETE FROM software WHERE user_id=player AND NOT is_npc AND NOT is_folder;
  DELETE FROM software_blueprints WHERE user_id=player;
  DELETE FROM user_research_modules WHERE user_id=player;
  UPDATE users SET tutorial_completed_at=NULL WHERE id=player;
  IF complete_tutorial_if_ready(player) THEN RAISE EXCEPTION 'new player completed too early'; END IF;
  INSERT INTO user_research_modules(user_id,module_id) SELECT player,id FROM research_modules ORDER BY id LIMIT 1;
  INSERT INTO software_blueprints(user_id,name,capability_id) SELECT player,'Once regression',id FROM software_capabilities ORDER BY id LIMIT 1;
  INSERT INTO software(user_id,server_id,soft_name,soft_type,soft_version,soft_size,soft_ram)
    VALUES(player,machine,'Once regression',1,1,1,1) RETURNING id INTO program;
  INSERT INTO software_running(soft_id,user_id) VALUES(program,player);
  IF (SELECT tutorial_completed_at IS NOT NULL FROM users WHERE id=player) THEN RAISE EXCEPTION 'intrusion is still missing'; END IF;
  INSERT INTO internet_access(user_id,ip) VALUES(player,123456789);
  IF NOT (SELECT tutorial_completed_at IS NOT NULL FROM users WHERE id=player) THEN RAISE EXCEPTION 'final achievement did not persist immediately'; END IF;
  DELETE FROM software_running WHERE soft_id=program;
  DELETE FROM software WHERE id=program;
  DELETE FROM internet_access WHERE user_id=player;
  IF NOT complete_tutorial_if_ready(player) THEN RAISE EXCEPTION 'deleted evidence reopened tutorial'; END IF;
  UPDATE tutorial_steps SET active=false;
  IF NOT complete_tutorial_if_ready(player) THEN RAISE EXCEPTION 'editing definition reopened tutorial'; END IF;

  -- The new BEFORE DELETE trigger must not suppress normal NPC deletions.
  SELECT user_id,server_id INTO npc_owner,machine FROM hardware WHERE is_npc ORDER BY server_id LIMIT 1;
  INSERT INTO software(user_id,server_id,soft_name,soft_type,soft_version,soft_size,soft_ram,is_npc)
    VALUES(npc_owner,machine,'NPC deletion regression',1,1,1,1,true) RETURNING id INTO npc_program;
  DELETE FROM software WHERE id=npc_program;
  IF EXISTS (SELECT 1 FROM software WHERE id=npc_program) THEN RAISE EXCEPTION 'NPC deletion was suppressed'; END IF;
  RAISE NOTICE 'PASS complete original circuit, persistence after deleting all evidence, edited definitions, NPC deletion';
END $$;
ROLLBACK;
