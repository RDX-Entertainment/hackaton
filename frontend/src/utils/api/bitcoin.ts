import { get, post } from './client';
import type { Wallet } from './types';

export function listWallets(): Promise<Wallet[]> {
  return get<Wallet[]>('/bitcoin/wallets');
}

export function createWallet(): Promise<Wallet> {
  return post<Wallet>('/bitcoin/wallets');
}

/** El importe viaja en satoshis, ya convertido: la API no sabe de comas. */
export function transferBitcoin(from: string, to: string, amount: number): Promise<void> {
  return post<void>('/bitcoin/transfers', { from, to, amount });
}
