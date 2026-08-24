import { get, post } from './client';
import type { Wallet } from './types';

export function listWallets(): Promise<Wallet[]> {
  return get<Wallet[]>('/bitcoin/wallets');
}

export function createWallet(): Promise<Wallet> {
  return post<Wallet>('/bitcoin/wallets');
}

/** Amounts travel as strings: bitcoin has more decimals than a float keeps. */
export function transferBitcoin(from: string, to: string, amount: string): Promise<void> {
  return post<void>('/bitcoin/transfers', { from, to, amount });
}
