-- Una sola capacidad, sin perder diseños, módulos ni atributos de copias existentes.
UPDATE software_capabilities SET code='antivirus', name='Antivirus', action_kind='antivirus',
  relevant_attributes='["detection","removal"]',
  secondary_attributes='["resistance","hardening","stability"]',
  available_actions='["detect","remove","defend"]'
WHERE code='antivirus_remove';

UPDATE software_profiles SET capability_id=(SELECT id FROM software_capabilities WHERE code='antivirus')
WHERE capability_id=(SELECT id FROM software_capabilities WHERE code='antivirus_detect');
UPDATE software_blueprints SET capability_id=(SELECT id FROM software_capabilities WHERE code='antivirus')
WHERE capability_id=(SELECT id FROM software_capabilities WHERE code='antivirus_detect');
UPDATE research_modules SET capability_id=(SELECT id FROM software_capabilities WHERE code='antivirus')
WHERE capability_id=(SELECT id FROM software_capabilities WHERE code='antivirus_detect');
INSERT INTO capability_counters(attacker_capability_id,defender_capability_id)
SELECT attacker_capability_id,(SELECT id FROM software_capabilities WHERE code='antivirus')
FROM capability_counters WHERE defender_capability_id=(SELECT id FROM software_capabilities WHERE code='antivirus_detect')
ON CONFLICT DO NOTHING;
DELETE FROM software_capabilities WHERE code='antivirus_detect';

UPDATE processes SET process_payload=jsonb_set(process_payload,'{capability}','"antivirus"')
WHERE completed_at IS NULL AND process_payload->>'capability' IN ('antivirus_detect','antivirus_remove');

UPDATE software_capabilities SET name='Fuerza bruta' WHERE code='cracker';

-- Compatibilidad para programas del catálogo/kit, incluidos los que se instalen después.
-- La versión queda como metadato: no decide las estadísticas ni el resultado del combate.
CREATE FUNCTION ensure_legacy_combat_profiles(owner_id bigint, npc_owner boolean) RETURNS void
LANGUAGE sql AS $$
  INSERT INTO software_profiles(software_id,origin,capability_id,effectiveness,penetration,
    stealth,resistance,hardening,detection,removal,stability,cpu_need,ram_need,net_need,weight)
  SELECT s.id,'legacy',c.id,50,CASE WHEN s.soft_type=1 THEN 45 ELSE 0 END,
    CASE WHEN s.soft_type=1 THEN 35 ELSE 0 END,
    CASE WHEN s.soft_type IN (2,7) THEN 55 ELSE 0 END,
    CASE WHEN s.soft_type IN (2,7) THEN 50 ELSE 0 END,
    CASE WHEN s.soft_type=7 THEN 50 WHEN s.soft_type=2 THEN 30 ELSE 0 END,
    CASE WHEN s.soft_type=7 THEN 35 ELSE 0 END,55,
    GREATEST(1,s.soft_ram),GREATEST(1,s.soft_ram),1,GREATEST(1,s.soft_size)
  FROM software s JOIN software_capabilities c ON c.code=CASE s.soft_type
    WHEN 1 THEN 'cracker' WHEN 2 THEN 'anti_bruteforce' WHEN 7 THEN 'antivirus' END
  WHERE s.user_id=owner_id AND s.is_npc=npc_owner
  ON CONFLICT(software_id) DO UPDATE SET capability_id=EXCLUDED.capability_id,
    resistance=EXCLUDED.resistance,hardening=EXCLUDED.hardening,detection=EXCLUDED.detection
  WHERE software_profiles.capability_id IS NULL;
$$;
SELECT ensure_legacy_combat_profiles(user_id,is_npc) FROM
  (SELECT DISTINCT user_id,is_npc FROM software WHERE soft_type IN (1,2,7)) owners;
