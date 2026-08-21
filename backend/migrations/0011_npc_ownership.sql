-- Let an NPC own hardware, software and a log.
--
-- 0001 carried over the legacy is_npc column on log, hardware and software, but also gave
-- each of them a foreign key to users(id). The two contradict each other: an NPC id is not
-- a user id, so is_npc could never be true and the column sat there unusable. That is why
-- 0008 had to leave the NPC census without the hardware and log rows NPC::generateNPC
-- always created, and why the NPC software went to software_original alone.
--
-- The legacy resolved this the way it is resolved here: one owner column, with isNPC
-- naming which id space it points into. The cost is real and worth stating: dropping the
-- constraint also drops ON DELETE CASCADE, so deleting a player no longer clears their
-- hardware and software. Callers must delete those rows explicitly, which is already the
-- case for log and processes.

ALTER TABLE log DROP CONSTRAINT log_user_id_fkey;
ALTER TABLE hardware DROP CONSTRAINT hardware_user_id_fkey;
ALTER TABLE software DROP CONSTRAINT software_user_id_fkey;

CREATE INDEX hardware_owner_idx ON hardware(user_id, is_npc);
CREATE INDEX software_owner_idx ON software(user_id, is_npc);
CREATE INDEX log_owner_idx ON log(user_id, is_npc);

-- The rest of NPC::generateNPC, which 0008 could not apply: every NPC got a machine with
-- cpu 1500 and net 10, and an empty log ready to be written to.
INSERT INTO hardware (user_id, name, cpu, net, is_npc)
SELECT id, 'npc', 1500, 10, TRUE FROM npc;

INSERT INTO log (user_id, text, is_npc)
SELECT id, '', TRUE FROM npc;

-- cron/restoreSoftware.php installed each NPC's original software onto the NPC itself.
-- With the constraint gone that copy is finally possible, so getBestSoftware can read an
-- NPC's defences from software like it does for a player.
INSERT INTO software (user_id, soft_name, soft_version, soft_size, soft_ram, soft_type, is_npc)
SELECT npc_id, soft_name, soft_version, soft_size, soft_ram, soft_type, TRUE
FROM software_original;
