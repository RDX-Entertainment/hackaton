export type SoftwareType = {
  id: number;
  name: string;
};

export type ProcessAction = {
  id: number;
  name: string;
};

export type ProcessCommand = 'pause' | 'resume' | 'delete';
export type MissionCommand = 'accept' | 'abort';
export type HackKind = 'bf' | 'scan';
export type TransferDirection = 'download' | 'upload';

