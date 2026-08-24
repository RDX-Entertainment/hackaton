import { createSlice, type PayloadAction } from '@reduxjs/toolkit';

import type { AuthenticatedSession, Session } from '../utils/api';
import { readSession } from '../utils/session';

const initialState: Session = readSession();

const authSlice = createSlice({
  name: 'auth',
  initialState,
  reducers: {
    authenticated: (_state, action: PayloadAction<AuthenticatedSession>) => action.payload,
    signedOut: () => ({ token: null, user: null }),
  },
});

export const { authenticated, signedOut } = authSlice.actions;

export const authReducer = authSlice.reducer;
