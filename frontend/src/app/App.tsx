import { Navigate, Route, Routes } from 'react-router-dom';

import { LoginPage } from '../pages/LoginPage';
import { RegisterPage } from '../pages/RegisterPage';
import { AdminShell } from './AdminShell';
import { GameShell } from './GameShell';
import { useAppSelector } from './hooks';

/** The only decision at this level: is there a session, or is there not. */
export function App() {
  const token = useAppSelector((state) => state.auth.token);

  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route path="/register" element={<RegisterPage />} />
      <Route
        path="/admin/*"
        element={token ? <AdminShell /> : <Navigate to="/login" replace />}
      />
      <Route path="/*" element={token ? <GameShell /> : <Navigate to="/login" replace />} />
    </Routes>
  );
}
