import { get, post } from './client';
import type { Bank, BankAccount } from './types';
import type { BankAccess, BankMovement } from './bank';

export const setMainAccount = (id: number) => post<void>(`/finances/accounts/${id}/main`, {});
export const mainAccount = () => get<BankAccess>('/finances/main');
export const mainStatement = () => get<BankMovement[]>('/finances/main/statement');
export const transferMain = (from_account: number, to_account: number, amount: number, concept = '') =>
  post<BankMovement>('/finances/main/transfer', { from_account, to_account, amount, concept });

export function listBanks(): Promise<Bank[]> {
  return get<Bank[]>('/finances/banks');
}

export function listAccounts(): Promise<BankAccount[]> {
  return get<BankAccount[]>('/finances/accounts');
}

export function openAccount(bankId: number, password: string): Promise<{ account: number }> {
  return post('/finances/accounts', { bank_id: bankId, password });
}

/** El PIN que se elige la primera vez. No sirve para cambiarlo después. */
export function setAccountPin(account: number, password: string): Promise<void> {
  return post<void>(`/finances/accounts/${account}/pin`, { password });
}

export function transferFunds(
  fromAccount: number,
  password: string,
  toAccount: number,
  amount: number,
): Promise<{ amount: number }> {
  return post('/finances/transfers', {
    from_account: fromAccount,
    password,
    to_account: toAccount,
    amount,
  });
}
