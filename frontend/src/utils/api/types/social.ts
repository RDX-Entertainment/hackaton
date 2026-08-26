export type Profile = {
  id: number;
  login: string;
  game_ip: number;
  biography: string;
  web: string;
  rank: number;
  hacks: number;
  money_hardware: number;
  exp: number;
  date_joined: string | null;
};

export type ClanMember = {
  id: number;
  login: string;
  auth_level: number;
};

export type Clan = {
  id: number;
  name: string;
  nick: string;
  description: string;
  slots: number;
  money: number;
  power: number;
  auth_level: number;
  members: ClanMember[];
};

export type ClanSummary = {
  id: number;
  name: string;
  nick: string;
  members: number;
  slots: number;
  description: string;
};

export type ClanRequest = {
  id: number;
  login: string;
  message: string;
  created_at: string;
};


export type RankedPlayer = {
  rank: number;
  user_id: number;
  login: string;
};
