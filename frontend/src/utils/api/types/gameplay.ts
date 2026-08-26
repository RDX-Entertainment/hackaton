export type SoftwareType = {
  id: number;
  name: string;
};

export type ProcessAction = {
  id: number;
  name: string;
};

/**
 * Lo que responde encolar una tarea. `seconds` es lo que va a tardar, y es lo único que
 * distingue «se ha encolado» de «se ha hecho»: hasta que no vence, no ha pasado nada.
 */
export type ProcessStarted = {
  id: number;
  action: number;
  seconds: number;
};

export type ProcessCommand = 'pause' | 'resume' | 'delete';
export type MissionCommand = 'accept' | 'abort';
export type HackKind = 'bf' | 'scan';
export type TransferDirection = 'download' | 'upload';

