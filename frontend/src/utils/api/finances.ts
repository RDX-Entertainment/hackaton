import { get, post } from './client';
import type { UnknownRow } from './types';

export function listAccounts(): Promise<UnknownRow[]> {
  return get<UnknownRow[]>('/finances/accounts');
}

export function openAccount(bankId: number, password: string): Promise<{ account: number }> {
  return post('/finances/accounts', { bank_id: bankId, password });
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
