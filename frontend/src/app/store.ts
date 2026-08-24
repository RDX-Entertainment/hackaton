import { configureStore } from '@reduxjs/toolkit';

import { writeSession } from '../utils/session';
import { authReducer } from './authSlice';

export const store = configureStore({
  reducer: { auth: authReducer },
});

// Mirroring the slice into localStorage is what survives a page reload.
store.subscribe(() => {
  writeSession(store.getState().auth);
});

export type RootState = ReturnType<typeof store.getState>;

export type AppDispatch = typeof store.dispatch;
