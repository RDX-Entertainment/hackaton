-- El editor administrativo creaba solo el fichero heredado. El arsenal modular exige
-- además un perfil que traduzca soft_type a la capacidad moderna; sin él, por ejemplo,
-- un SSH no acepta los módulos restringidos a ssh_exploit.
INSERT INTO software_profiles(
  software_id, origin, capability_id, effectiveness, penetration, stealth,
  stability, cpu_need, ram_need, net_need, weight
)
SELECT s.id, 'legacy', c.id,
  LEAST(100, GREATEST(20, 30+s.soft_version)),
  CASE WHEN s.soft_type IN (1,13,14,99) THEN 35 ELSE 0 END,
  CASE WHEN s.soft_type IN (13,14,99) THEN 35 ELSE 0 END,
  55, GREATEST(1,s.soft_ram), GREATEST(1,s.soft_ram), 1, GREATEST(1,s.soft_size)
FROM software s
JOIN software_capabilities c ON c.code=CASE s.soft_type
  WHEN 1 THEN 'cracker'
  WHEN 2 THEN 'anti_bruteforce'
  WHEN 3 THEN 'port_scan'
  WHEN 4 THEN 'firewall_ssh'
  WHEN 7 THEN 'antivirus'
  WHEN 13 THEN 'ftp_exploit'
  WHEN 14 THEN 'ssh_exploit'
  WHEN 99 THEN 'virus_infect'
END
WHERE NOT s.is_npc
ON CONFLICT(software_id) DO UPDATE SET capability_id=EXCLUDED.capability_id
WHERE software_profiles.capability_id IS NULL;
