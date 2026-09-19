INSERT INTO software_capabilities(domain_id,code,name,role,action_kind,relevant_attributes,secondary_attributes,available_actions,max_access_scope,granted_permissions)
SELECT id,'port_scan','Escáner de puertos','utility','scan','["effectiveness","stealth"]','["stability"]','["scan_ports"]','service','[]'
FROM software_domains WHERE code='network';
INSERT INTO capability_counters(attacker_capability_id,defender_capability_id)
SELECT a.id,d.id FROM software_capabilities a CROSS JOIN software_capabilities d
WHERE a.code='port_scan' AND d.code IN ('intrusion_ids','firewall_ssh','firewall_ftp');

-- Compatibilidad con los PortScan antiguos, sin reemplazar perfiles personalizados.
CREATE FUNCTION ensure_port_scan_profiles(owner_id bigint, npc_owner boolean) RETURNS void
LANGUAGE sql AS $$
 INSERT INTO software_profiles(software_id,origin,capability_id,effectiveness,stealth,stability,cpu_need,ram_need,net_need,weight)
 SELECT s.id,'legacy',c.id,50,45,55,40,GREATEST(s.soft_ram,1),1,GREATEST(s.soft_size,1)
 FROM software s CROSS JOIN software_capabilities c
 WHERE s.user_id=owner_id AND s.is_npc=npc_owner AND s.soft_type=3 AND c.code='port_scan'
 AND NOT EXISTS(SELECT 1 FROM software_profiles p WHERE p.software_id=s.id AND p.capability_id IS NOT NULL)
 ON CONFLICT(software_id) DO UPDATE SET capability_id=EXCLUDED.capability_id WHERE software_profiles.capability_id IS NULL
$$;
SELECT ensure_port_scan_profiles(user_id,is_npc) FROM (SELECT DISTINCT user_id,is_npc FROM software WHERE soft_type=3) owners;
