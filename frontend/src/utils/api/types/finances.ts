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

