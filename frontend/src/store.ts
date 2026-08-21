import { configureStore, createSlice, type PayloadAction } from '@reduxjs/toolkit';

export type User = { id: number; login: string; email: string; game_ip: number; premium: boolean };
type AuthState = { token: string | null; user: User | null };
const saved = localStorage.getItem('legacy.session');
const initialState: AuthState = saved ? JSON.parse(saved) : { token: null, user: null };
const auth = createSlice({ name: 'auth', initialState, reducers: {
  authenticated: (_, action: PayloadAction<AuthState>) => action.payload,
  signedOut: () => ({ token: null, user: null }),
}});
export const { authenticated, signedOut } = auth.actions;
export const store = configureStore({ reducer: { auth: auth.reducer } });
store.subscribe(() => localStorage.setItem('legacy.session', JSON.stringify(store.getState().auth)));
export type RootState = ReturnType<typeof store.getState>;
