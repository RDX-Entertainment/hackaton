-- Track what is installed and what a hidden file was hidden with.
--
-- soft_hidden survived into 0001 but softHiddenWith did not, and on its own the flag is
-- half a mechanic: SEEK works by comparing the seeker's version against the hider that
-- buried the file, so without the second column hiding cannot be beaten or lost.
ALTER TABLE software ADD COLUMN soft_hidden_with INTEGER NOT NULL DEFAULT 0;

-- software_running is the legacy table that separated owning a program from having it
-- loaded. getBestSoftware asked for installed software specifically, so until this exists
-- every check that says "the best cracker you have running" is really just "the best one
-- you own", and RAM buys nothing.
CREATE TABLE software_running (
  soft_id BIGINT PRIMARY KEY,
  user_id BIGINT NOT NULL,
  is_npc BOOLEAN NOT NULL DEFAULT FALSE
);
CREATE INDEX software_running_owner_idx ON software_running(user_id, is_npc);

-- The NPC census comes with its defences loaded, which is what makes them defences.
INSERT INTO software_running (soft_id, user_id, is_npc)
SELECT id, user_id, TRUE FROM software WHERE is_npc;
