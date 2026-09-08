/**
 * El sistema modular de software. El backend lo llama `software_system` y lo sirve bajo
 * `/software-system/`; aquí es el arsenal, que es lo que el jugador ve: qué sabe construir,
 * con qué piezas, qué ha compilado y qué puede lanzar con ello.
 */

/**
 * Los diez números de un programa. Los seis primeros van de 0 a 100 y describen lo que hace;
 * los tres siguientes son lo que cuesta tenerlo corriendo, en las unidades del catálogo de
 * piezas —las mismas de `hardware.cpu/ram/net`—, y `weight` es lo que ocupa en disco.
 */
export type ProgramStats = {
    effectiveness: number;
  resistance: number;
  stealth: number;
  detection: number;
  removal: number;
  penetration: number;
  hardening: number;
  manipulation: number;
  integrity: number;
  anti_forensics: number;
  forensics: number;
  persistence: number;
  stability: number;
  cpu: number;
  ram: number;
  net: number;
  weight: number;
};

export type CapabilityRole = 'attacker' | 'defender' | 'utility';

/** Qué sabe hacer un programa. El dominio agrupa; el rol dice de qué lado juega. */
export type Capability = {
  code: string;
  name: string;
  domain: string;
  role: CapabilityRole;
  /** `intrude`, `infect`, `block`, `detect`, `remove`: lo que pasa si el ataque gana. */
  action_kind: string;
  relevant_attributes: string[];
  secondary_attributes: string[];
  available_actions: string[];
  max_access_scope: string;
  granted_permissions: string[];
};

/** Una pieza de research. Con `capability` a null sería una que encaja en cualquier cosa. */
export type ResearchModule = {
  id: number;
  code: string;
  name: string;
  capability: string | null;
  deltas: ProgramStats;
};

/**
 * Un diseño: una capacidad y hasta tres módulos. `projected` es lo que daría al compilarlo, y
 * lo calcula el servidor: la regla de cómo suman los módulos vive en un solo sitio.
 */
export type Blueprint = {
  id: number;
  name: string;
  revision: number;
  capability: string;
  modules: number[];
  projected: ProgramStats;
};

export type CompilationQueued = {
  process_id: number;
  blueprint_id: number;
  name: string;
  capability: string;
  seconds: number;
  stats: ProgramStats;
};
export type Skill = { action: string; attribute: string; threshold: number; unlocked: boolean };

/**
 * Una copia ya compilada. `running` es lo que separa tenerla de poder usarla: una operación
 * exige el programa cargado en memoria, y cargarlo se hace desde Software.
 */
export type Program = {
  software_id: number;
  server_id: number;
  resources: { cpu: number; ram: number; net: number } | null;
  name: string;
  version: number;
  capability: string | null;
  capability_name: string | null;
  role: string | null;
  action_kind: string | null;
  origin: string;
  running: boolean;
  stats: ProgramStats;
  skills: Skill[];
};

export type OperationStarted = {
  process_id: number;
  seconds: number;
  capability: string;
  work_units: number;
  attacker: ProgramStats;
  defender: ProgramStats;
  estimated_chance: 'muy baja' | 'baja' | 'equilibrada' | 'alta' | 'muy alta';
};

export type MachineAccess = { ip: number; identity: string; privilege: string; scope: string; permissions: string[]; status: string; persistence_installed: boolean };
export type WebContent = { path: string; content: string; revision: number };
export type TacticalActionStarted = { process_id: number; seconds: number; estimated_chance: 'muy baja' | 'baja' | 'equilibrada' | 'alta' | 'muy alta' };
export type EvidenceSummary = { id: number; process_id: number; action: string; state: string; visible: boolean; can_tamper: boolean; apparent_origin_ip: number | null; confidence: number };
