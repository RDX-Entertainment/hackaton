-- Remember where a downloaded file came from.
--
-- The legacy INSERT in case 1 of completeProcess carried originalFrom, the id of the file
-- the copy was taken from, and licensedTo. 0001 left both out, so a stolen file was
-- indistinguishable from one the player wrote. Research and the licence checks in the
-- legacy both lean on that distinction, so the columns come back now that downloads exist.
ALTER TABLE software ADD COLUMN original_from BIGINT NOT NULL DEFAULT 0;
ALTER TABLE software ADD COLUMN licensed_to BIGINT NOT NULL DEFAULT 0;
CREATE INDEX software_original_from_idx ON software(original_from) WHERE original_from <> 0;
