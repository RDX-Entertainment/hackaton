export type User = {
  id: number;
  login: string;
  email: string;
  game_ip: number;
  premium: boolean;
};

export type Session = {
  token: string | null;
  user: User | null;
};

export type AuthenticatedSession = {
  token: string;
  user: User;
};

