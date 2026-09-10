export type Wallet = {
  address: string;
  amount: number;
  created_at?: string;
};

export type Bank = {
  id: number;
  slug: string;
  name: string;
};

export type BankAccount = {
  id: number;
  account: number;
  bank_id: number;
  bank_name: string;
  cash: number;
  pin_set: boolean;
};

export type ListEntry = {
  id: number;
  ip: number;
  login: string;
  hacked_at: string;
  virus_id: number;
};

export type Certification = {
  level: number;
  name: string;
  description: string;
  price: number;
  completed: boolean;
  available: boolean;
  learning: boolean;

  /**
   * El paso del tutorial que la aprueba, cuando no se estudia sino que se hace.
   *
   * Los dos primeros niveles **son** el tutorial: el 1 al compilar el primer programa y el 2
   * al entrar en la primera máquina. Con esto la pantalla dice qué hay que hacer en vez de
   * ofrecer una matrícula que el servidor va a rechazar.
   */
  earned_by: string | null;
};

export type Certifications = {
  level: number;
  items: Certification[];
};

export type Settings = {
  login: string;
  email: string;
  premium: boolean;
  last_login: string;
};

