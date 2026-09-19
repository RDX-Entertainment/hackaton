-- Rename existing tiers without replacing any owned parts.
ALTER TABLE hardware_parts DROP CONSTRAINT hardware_parts_tier_check;
UPDATE hardware_parts SET tier = CASE tier WHEN 'gaming' THEN 'gamer'
  WHEN 'industrial' THEN 'workstation' ELSE tier END;
ALTER TABLE hardware_parts ADD CONSTRAINT hardware_parts_tier_check
  CHECK (tier IN ('domestica', 'gamer', 'workstation', 'server'));

-- Every CPU socket owns an equally sized bank of DIMMs. Round old boards up,
-- never down: no installed component is removed during migration.
UPDATE hardware_parts SET cpu_slots = greatest(cpu_slots, 1),
  ram_slots = (greatest(1, ceil(ram_slots::numeric / greatest(cpu_slots, 1))) * greatest(cpu_slots, 1))::smallint
WHERE kind = 'mobo';
ALTER TABLE hardware_parts ADD CONSTRAINT hardware_ram_groups CHECK
  (kind <> 'mobo' OR (cpu_slots > 0 AND ram_slots >= cpu_slots AND ram_slots % cpu_slots = 0));

ALTER TABLE hardware
  ADD COLUMN node_kind text NOT NULL DEFAULT 'computer' CHECK (node_kind IN ('computer', 'switch')),
  ADD COLUMN switch_ports smallint NOT NULL DEFAULT 0 CHECK (switch_ports BETWEEN 0 AND 48),
  ADD COLUMN component_managed boolean NOT NULL DEFAULT true,
  ADD COLUMN operational boolean NOT NULL DEFAULT false,
  ADD COLUMN disabled_reasons text[] NOT NULL DEFAULT '{NO_BOARD,NO_CPU,NO_RAM,NO_DISK}';
ALTER TABLE hardware ADD CONSTRAINT hardware_switch_shape CHECK
  (node_kind <> 'switch' OR (switch_ports > 0 AND NOT is_gateway AND public_ip IS NULL AND cluster_id IS NULL));

-- Legacy NPCs and practice templates store capacities directly. Do not erase their
-- virtual hardware. Once a real part is installed they permanently use assembly rules.
UPDATE hardware h SET component_managed = NOT h.is_npc OR EXISTS(
  SELECT 1 FROM hardware_components c WHERE c.installed_in=h.server_id);
CREATE FUNCTION legacy_hardware_readiness() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  IF TG_OP='INSERT' AND NEW.is_npc THEN NEW.component_managed := false; END IF;
  IF NOT NEW.component_managed THEN
    NEW.disabled_reasons := '{}';
    IF NEW.cpu<=0 THEN NEW.disabled_reasons := array_append(NEW.disabled_reasons,'NO_CPU'); END IF;
    IF NEW.ram<=0 THEN NEW.disabled_reasons := array_append(NEW.disabled_reasons,'NO_RAM'); END IF;
    IF NEW.hdd<=0 THEN NEW.disabled_reasons := array_append(NEW.disabled_reasons,'NO_DISK'); END IF;
    NEW.operational := cardinality(NEW.disabled_reasons)=0;
  END IF;
  RETURN NEW;
END $$;
CREATE TRIGGER hardware_legacy_readiness BEFORE INSERT OR UPDATE ON hardware
  FOR EACH ROW EXECUTE FUNCTION legacy_hardware_readiness();

CREATE FUNCTION refresh_hardware(machine bigint) RETURNS void LANGUAGE plpgsql AS $$
DECLARE s record; reasons text[] := '{}'; kind text; managed boolean;
BEGIN
  SELECT node_kind,component_managed INTO kind,managed FROM hardware WHERE server_id = machine FOR UPDATE;
  IF NOT FOUND THEN RETURN; END IF;
  IF NOT managed THEN
    -- Re-run the legacy readiness trigger without changing its capacities.
    UPDATE hardware SET operational=operational WHERE server_id=machine;
    RETURN;
  END IF;
  IF kind = 'switch' THEN
    UPDATE hardware SET operational = true, disabled_reasons = '{}' WHERE server_id = machine;
    RETURN;
  END IF;
  SELECT count(*) FILTER (WHERE p.kind='mobo') AS boards,
    count(*) FILTER (WHERE p.kind='cpu') AS cpus,
    count(*) FILTER (WHERE p.kind='ram') AS dimms,
    count(*) FILTER (WHERE p.kind='hdd') AS disks,
    max(p.cpu_slots) FILTER (WHERE p.kind='mobo') AS sockets,
    max(p.ram_slots) FILTER (WHERE p.kind='mobo') AS ram_slots,
    coalesce(sum(p.power) FILTER (WHERE p.kind='cpu'),0) AS cpu,
    coalesce(sum(p.power) FILTER (WHERE p.kind='ram'),0) AS ram,
    coalesce(sum(p.power) FILTER (WHERE p.kind='hdd'),0) AS hdd,
    coalesce(sum(p.power) FILTER (WHERE p.kind='net'),0) AS net
  INTO s FROM hardware_components c JOIN hardware_parts p ON p.id=c.part_id
  WHERE c.installed_in=machine;
  IF s.boards=0 THEN reasons := array_append(reasons,'NO_BOARD'); END IF;
  IF s.cpus=0 THEN reasons := array_append(reasons,'NO_CPU'); END IF;
  IF s.dimms=0 THEN reasons := array_append(reasons,'NO_RAM'); END IF;
  IF s.disks=0 THEN reasons := array_append(reasons,'NO_DISK'); END IF;
  IF s.cpus>0 AND (s.dimms % s.cpus <> 0 OR s.dimms < s.cpus) THEN
    reasons := array_append(reasons,'RAM_UNBALANCED');
  END IF;
  IF s.boards>0 AND s.cpus>0 AND s.dimms > s.cpus * (s.ram_slots / s.sockets) THEN
    reasons := array_append(reasons,'RAM_NEEDS_CPU');
  END IF;
  -- Capacity remains physical: being off must not erase files or installed RAM.
  UPDATE hardware SET cpu=s.cpu, ram=s.ram, hdd=s.hdd, net=s.net,
    operational=cardinality(reasons)=0, disabled_reasons=reasons WHERE server_id=machine;
END $$;

CREATE FUNCTION refresh_changed_hardware() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  IF TG_OP <> 'INSERT' AND OLD.installed_in IS NOT NULL THEN PERFORM refresh_hardware(OLD.installed_in); END IF;
  IF TG_OP <> 'DELETE' AND NEW.installed_in IS NOT NULL THEN
    UPDATE hardware SET component_managed=true WHERE server_id=NEW.installed_in;
    PERFORM refresh_hardware(NEW.installed_in);
  END IF;
  RETURN NULL;
END $$;
CREATE TRIGGER hardware_component_readiness AFTER INSERT OR UPDATE OR DELETE ON hardware_components
  FOR EACH ROW EXECUTE FUNCTION refresh_changed_hardware();
CREATE FUNCTION refresh_changed_part() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE machine bigint;
BEGIN
  FOR machine IN SELECT DISTINCT installed_in FROM hardware_components WHERE part_id=NEW.id AND installed_in IS NOT NULL
  LOOP PERFORM refresh_hardware(machine); END LOOP;
  RETURN NULL;
END $$;
CREATE TRIGGER hardware_part_readiness AFTER UPDATE ON hardware_parts
  FOR EACH ROW EXECUTE FUNCTION refresh_changed_part();
DO $$ DECLARE machine bigint; BEGIN
  FOR machine IN SELECT server_id FROM hardware LOOP PERFORM refresh_hardware(machine); END LOOP;
END $$;

-- The database also enforces the rule for every process entry point.
CREATE FUNCTION require_operational_origin() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE ready boolean;
BEGIN
  IF NEW.origin_server IS NULL THEN
    SELECT server_id INTO NEW.origin_server FROM hardware
      WHERE user_id=NEW.creator_id AND NOT is_npc AND is_gateway;
  END IF;
  SELECT operational AND node_kind='computer' INTO ready FROM hardware WHERE server_id=NEW.origin_server FOR SHARE;
  IF ready IS DISTINCT FROM true THEN RAISE EXCEPTION USING ERRCODE='P0059', MESSAGE='HARDWARE_DISABLED'; END IF;
  RETURN NEW;
END $$;
CREATE TRIGGER process_hardware_guard BEFORE INSERT ON processes FOR EACH ROW EXECUTE FUNCTION require_operational_origin();

ALTER TABLE users ADD COLUMN main_bank_account bigint REFERENCES bank_accounts(id) ON DELETE SET NULL;
UPDATE users u SET main_bank_account=(SELECT a.id FROM bank_accounts a WHERE a.bank_user=u.id ORDER BY a.date_created,a.id LIMIT 1);
CREATE FUNCTION first_bank_account_is_main() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  IF NEW.bank_user IS NOT NULL THEN
    UPDATE users SET main_bank_account=NEW.id WHERE id=NEW.bank_user AND main_bank_account IS NULL;
  END IF;
  RETURN NULL;
END $$;
CREATE TRIGGER first_bank_account AFTER INSERT ON bank_accounts FOR EACH ROW EXECUTE FUNCTION first_bank_account_is_main();
