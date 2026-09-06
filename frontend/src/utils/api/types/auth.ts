export type User = {
  id: number;
  login: string;
  email: string;
  game_ip: number;
  premium: boolean;
  /** En que lengua lee. Vive en el servidor, asi que le sigue de un navegador a otro. */
  lang: string;
};

export type Session = {
  token: string | null;
  user: User | null;
};

export type AuthenticatedSession = {
  token: string;
  user: User;
};

