import { post } from './client';
import type { BankAccount } from './types';

/**
 * Las operaciones del banco. La credencial es la cuenta y su PIN, no la sesión: quien las
 * trae manda, que es lo que hace posible tanto cobrar como que te roben.
 */

export type BankAccess = {
  account: number;
  bank_id: number;
  bank_name: string;
  holder: string;
  cash: number;
};

export type BankMovement = {
  account: number;
  other_account: number | null;
  amount: number;
  balance: number;
  concept: string;
};

export type Charge = {
  id: number;
  to_account: number;
  from_account: number | null;
  amount: number;
  concept: string;
  status: string;
  paid_by: number | null;
  open: boolean;
};

type Credentials = { account: number; pin: string };

export function login(input: Credentials): Promise<BankAccess> {
  return post<BankAccess>('/bank/login', input);
}

export function statement(input: Credentials): Promise<BankMovement[]> {
  return post<BankMovement[]>('/bank/statement', input);
}

export function transfer(input: Credentials & {
  to_account: number;
  amount: number;
  concept?: string;
}): Promise<BankMovement> {
  return post<BankMovement>('/bank/transfer', input);
}

export function charges(input: Credentials): Promise<Charge[]> {
  return post<Charge[]>('/bank/charges', input);
}

export function createCharge(input: Credentials & {
  from_account?: number | null;
  amount: number;
  concept: string;
}): Promise<Charge> {
  return post<Charge>('/bank/charges/new', input);
}

export function payCharge(id: number, input: Credentials): Promise<BankMovement> {
  return post<BankMovement>(`/bank/charges/${id}/pay`, input);
}

export function cancelCharge(id: number, input: Credentials): Promise<void> {
  return post<void>(`/bank/charges/${id}/cancel`, input);
}

export type { BankAccount };
