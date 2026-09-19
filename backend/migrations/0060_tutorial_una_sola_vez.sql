-- Completion is an achievement, not the current contents of a disk.
ALTER TABLE users ADD COLUMN tutorial_completed_at timestamptz;

CREATE FUNCTION complete_tutorial_if_ready(player bigint) RETURNS boolean LANGUAGE plpgsql AS $$
BEGIN
  IF EXISTS (SELECT 1 FROM users WHERE id=player AND tutorial_completed_at IS NOT NULL) THEN RETURN true; END IF;
  IF NOT EXISTS (SELECT 1 FROM users WHERE id=player) OR NOT EXISTS (SELECT 1 FROM tutorial_steps WHERE active) THEN RETURN false; END IF;
  IF EXISTS (
    SELECT 1 FROM tutorial_steps s WHERE s.active AND NOT CASE s.fact
      WHEN 'modules' THEN EXISTS (SELECT 1 FROM user_research_modules WHERE user_id=player)
      WHEN 'blueprints' THEN EXISTS (SELECT 1 FROM software_blueprints WHERE user_id=player)
      WHEN 'programs' THEN EXISTS (SELECT 1 FROM software WHERE user_id=player AND NOT is_npc AND NOT is_folder)
      WHEN 'running' THEN EXISTS (SELECT 1 FROM software_running WHERE user_id=player AND NOT is_npc)
      WHEN 'intrusions' THEN EXISTS (SELECT 1 FROM machine_accesses WHERE attacker_id=player)
        OR EXISTS (SELECT 1 FROM internet_access WHERE user_id=player)
      ELSE true END
  ) THEN RETURN false; END IF;
  UPDATE users SET tutorial_completed_at=now() WHERE id=player AND tutorial_completed_at IS NULL;
  RETURN true;
END $$;

-- Recover graduates of the original five-step tutorial whose software was already
-- removed before this fix: intrusion was its final milestone. Do not apply this
-- shortcut to a custom/reordered tutorial.
UPDATE users u SET tutorial_completed_at=now()
WHERE (SELECT array_agg(code || ':' || fact ORDER BY position) FROM tutorial_steps WHERE active)
  = ARRAY['modules:modules','blueprint:blueprints','compile:programs','install:running','intrusion:intrusions']
AND (EXISTS (SELECT 1 FROM machine_accesses WHERE attacker_id=u.id)
  OR EXISTS (SELECT 1 FROM internet_access WHERE user_id=u.id));
DO $$ DECLARE player bigint; BEGIN
  FOR player IN SELECT id FROM users WHERE tutorial_completed_at IS NULL LOOP
    PERFORM complete_tutorial_if_ready(player);
  END LOOP;
END $$;

-- Persist at the actual achievement, not on the next browser poll. Check before
-- removing evidence too; deleting an already-earned tutorial asset cannot reopen it.
CREATE FUNCTION remember_tutorial_completion() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE item jsonb; player bigint;
BEGIN
  IF TG_OP='DELETE' THEN item:=to_jsonb(OLD); ELSE item:=to_jsonb(NEW); END IF;
  IF coalesce((item->>'is_npc')::boolean,false) THEN
    IF TG_OP='DELETE' THEN RETURN OLD; ELSE RETURN NEW; END IF;
  END IF;
  player:=coalesce((item->>'attacker_id')::bigint,(item->>'user_id')::bigint);
  IF player IS NOT NULL THEN PERFORM complete_tutorial_if_ready(player); END IF;
  IF TG_OP='DELETE' THEN RETURN OLD; ELSE RETURN NEW; END IF;
END $$;
DO $$ DECLARE relation text; BEGIN
  FOREACH relation IN ARRAY ARRAY['user_research_modules','software_blueprints','software','software_running','machine_accesses','internet_access'] LOOP
    EXECUTE format('CREATE TRIGGER tutorial_achievement AFTER INSERT OR UPDATE ON %I FOR EACH ROW EXECUTE FUNCTION remember_tutorial_completion()',relation);
    EXECUTE format('CREATE TRIGGER tutorial_before_removal BEFORE DELETE ON %I FOR EACH ROW EXECUTE FUNCTION remember_tutorial_completion()',relation);
  END LOOP;
END $$;
