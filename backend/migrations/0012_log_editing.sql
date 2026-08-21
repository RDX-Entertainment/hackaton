-- Give a pending log edit somewhere to keep its text.
--
-- Process::tmpLog wrote the replacement text into log_edit and handed the row id to the
-- E_LOG process, which applied it when the process finished. 0001 carried log_edit over
-- without that logText column, so an edit had nowhere to hold what it was about to write.
--
-- The legacy threaded the log_edit id through the process row; a column pointing the other
-- way says the same thing and survives a process being deleted.
ALTER TABLE log_edit ADD COLUMN log_text TEXT NOT NULL DEFAULT '';
ALTER TABLE log_edit ADD COLUMN process_id BIGINT;
CREATE INDEX log_edit_process_idx ON log_edit(process_id);
