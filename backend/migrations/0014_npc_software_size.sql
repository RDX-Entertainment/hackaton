-- Make NPC software small enough to actually steal.
--
-- 0008 invented 200 and 300 for the seeded cracker and firewall. A fresh player starts
-- with hdd 100, the legacy default, so nothing an NPC owned could ever be downloaded: the
-- disk check in completeProcess refused every copy. The sizes were guessed before download
-- existed, and downloading is exactly what makes an NPC worth breaking into.
--
-- 30 and 50 keep a level 1 target's files within a starting disk while leaving the upgrade
-- path meaningful, which is what the legacy shop is for.
UPDATE software_original SET soft_size = 30 WHERE soft_type = 1;
UPDATE software_original SET soft_size = 50 WHERE soft_type = 4;
UPDATE software SET soft_size = 30 WHERE is_npc AND soft_type = 1;
UPDATE software SET soft_size = 50 WHERE is_npc AND soft_type = 4;
