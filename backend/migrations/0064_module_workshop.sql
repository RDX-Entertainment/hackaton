-- Physical expansion modules; old compiled profiles remain their immutable base.
ALTER TABLE research_modules ADD COLUMN workshop_config jsonb NOT NULL DEFAULT '{}';
CREATE TABLE module_stock (
 user_id bigint REFERENCES users(id) ON DELETE CASCADE,
 module_id bigint REFERENCES research_modules(id) ON DELETE RESTRICT,
 tier integer NOT NULL CHECK(tier BETWEEN 0 AND 4),
 quantity integer NOT NULL CHECK(quantity >= 0),
 PRIMARY KEY(user_id,module_id,tier)
);
INSERT INTO module_stock SELECT user_id,module_id,0,1 FROM user_research_modules;
CREATE TABLE module_collection (
 user_id bigint PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
 collected_at timestamptz NOT NULL
);
CREATE TABLE program_modules (
 software_id bigint REFERENCES software(id) ON DELETE CASCADE,
 slot integer CHECK(slot BETWEEN 1 AND 6),
 module_id bigint NOT NULL REFERENCES research_modules(id) ON DELETE RESTRICT,
 tier integer NOT NULL CHECK(tier BETWEEN 0 AND 4),
 owner_id bigint NOT NULL REFERENCES users(id) ON DELETE CASCADE,
 snapshot jsonb NOT NULL,
 PRIMARY KEY(software_id,slot)
);
UPDATE research_modules m SET workshop_config=jsonb_build_object(
 'allowed_capabilities',COALESCE((SELECT jsonb_build_array(c.code) FROM software_capabilities c WHERE c.id=m.capability_id),'[]'::jsonb),
 'allowed_slots','[1,2,3,4,5,6]'::jsonb,'min_level',1,'max_copies',1,
 'excludes','[]'::jsonb,'grants','[]'::jsonb,'active',true,'collectable',true,
 'deltas',jsonb_build_object('effectiveness',m.effectiveness_delta,'resistance',m.resistance_delta,'stealth',m.stealth_delta,'detection',m.detection_delta,'removal',m.removal_delta,'penetration',m.penetration_delta,'hardening',m.hardening_delta,'manipulation',m.manipulation_delta,'integrity',m.integrity_delta,'anti_forensics',m.anti_forensics_delta,'forensics',m.forensics_delta,'persistence',m.persistence_delta,'stability',m.stability_delta,'cpu',m.cpu_delta,'ram',m.ram_delta,'net',m.net_delta,'weight',m.size_delta));
INSERT INTO research_modules(code,name,detection_delta,cpu_delta,ram_delta,size_delta,workshop_config)
VALUES('heuristic_sensor','Sensor heurístico',30,40,24,8,
'{"allowed_capabilities":[],"allowed_slots":[1,3,5,6],"min_level":1,"max_copies":1,"excludes":[],"grants":["antivirus_scan"],"active":true,"collectable":true,"deltas":{"detection":30,"cpu":40,"ram":24,"weight":8}}');
INSERT INTO research_modules(code,name,removal_delta,cpu_delta,ram_delta,size_delta,workshop_config)
VALUES('purge_engine','Motor de purga',30,40,24,8,
'{"allowed_capabilities":[],"allowed_slots":[1,3,5,6],"min_level":1,"max_copies":1,"excludes":[],"grants":["antivirus_clean"],"active":true,"collectable":true,"deltas":{"removal":30,"cpu":40,"ram":24,"weight":8}}');
CREATE FUNCTION module_slots(level integer) RETURNS integer LANGUAGE sql IMMUTABLE AS $$
 SELECT count(*)::integer FROM unnest(ARRAY[1,3,6,10,15,21]) n WHERE n <= greatest(level,1)
$$;
CREATE FUNCTION program_functions(program bigint) RETURNS text[] LANGUAGE sql STABLE AS $$
 SELECT COALESCE(array_agg(DISTINCT code) FILTER(WHERE code IS NOT NULL),ARRAY[]::text[]) FROM (
 SELECT c.code FROM software_profiles p JOIN software_capabilities c ON c.id=p.capability_id WHERE p.software_id=program
 UNION ALL SELECT jsonb_array_elements_text(m.snapshot->'grants') FROM program_modules m WHERE m.software_id=program
 ) f
$$;
CREATE FUNCTION program_has_function(program bigint,code text) RETURNS boolean LANGUAGE sql STABLE AS $$
 SELECT code=ANY(program_functions(program))
 OR (code IN ('antivirus_scan','antivirus_clean') AND 'antivirus'=ANY(program_functions(program)))
 OR (code='antivirus' AND program_functions(program) && ARRAY['antivirus_scan','antivirus_clean'])
$$;
CREATE VIEW effective_software_profiles AS
 SELECT p.software_id,p.blueprint_id,p.origin,p.capability_id,
 least(100,greatest(0,p.effectiveness+COALESCE((d.boost->>'effectiveness')::integer,0))) AS effectiveness,
 least(100,greatest(0,p.resistance+COALESCE((d.boost->>'resistance')::integer,0))) AS resistance,
 least(100,greatest(0,p.stealth+COALESCE((d.boost->>'stealth')::integer,0))) AS stealth,
 least(100,greatest(0,p.detection+COALESCE((d.boost->>'detection')::integer,0))) AS detection,
 least(100,greatest(0,p.removal+COALESCE((d.boost->>'removal')::integer,0))) AS removal,
 least(100,greatest(0,p.penetration+COALESCE((d.boost->>'penetration')::integer,0))) AS penetration,
 least(100,greatest(0,p.hardening+COALESCE((d.boost->>'hardening')::integer,0))) AS hardening,
 least(100,greatest(0,p.manipulation+COALESCE((d.boost->>'manipulation')::integer,0))) AS manipulation,
 least(100,greatest(0,p.integrity+COALESCE((d.boost->>'integrity')::integer,0))) AS integrity,
 least(100,greatest(0,p.anti_forensics+COALESCE((d.boost->>'anti_forensics')::integer,0))) AS anti_forensics,
 least(100,greatest(0,p.forensics+COALESCE((d.boost->>'forensics')::integer,0))) AS forensics,
 least(100,greatest(0,p.persistence+COALESCE((d.boost->>'persistence')::integer,0))) AS persistence,
 least(100,greatest(0,p.stability+COALESCE((d.boost->>'stability')::integer,0))) AS stability,
 greatest(1,p.cpu_need+COALESCE((d.boost->>'cpu')::integer,0)) AS cpu_need,
 greatest(1,p.ram_need+COALESCE((d.boost->>'ram')::integer,0)) AS ram_need,
 greatest(1,p.net_need+COALESCE((d.boost->>'net')::integer,0)) AS net_need,
 greatest(1,p.weight+COALESCE((d.boost->>'weight')::integer,0)) AS weight
 FROM software_profiles p LEFT JOIN LATERAL (
 SELECT jsonb_object_agg(key,total) AS boost FROM (
 SELECT key,sum(value::integer)::integer AS total FROM program_modules m
 CROSS JOIN LATERAL jsonb_each_text(m.snapshot->'deltas')
 WHERE m.software_id=p.software_id GROUP BY key
 ) sums
 ) d ON true;
-- Deleting a file returns its physical modules; copies never acquire module ownership.
CREATE FUNCTION refund_program_module() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF EXISTS(SELECT 1 FROM users WHERE id=OLD.owner_id) THEN
 INSERT INTO module_stock VALUES(OLD.owner_id,OLD.module_id,OLD.tier,1)
 ON CONFLICT(user_id,module_id,tier) DO UPDATE SET quantity=module_stock.quantity+1;
 END IF;
 RETURN OLD;
END $$;
CREATE TRIGGER refund_program_module AFTER DELETE ON program_modules
 FOR EACH ROW EXECUTE FUNCTION refund_program_module();
