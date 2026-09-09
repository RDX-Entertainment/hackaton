import { Navigate, Route, Routes } from 'react-router-dom';

import { LoginPage } from '../pages/LoginPage';
import { RegisterPage } from '../pages/RegisterPage';
import { AdminShell } from './AdminShell';
import { GameShell } from './GameShell';
import { useAppSelector } from './hooks';

/**
 * La única decisión de este nivel: hay sesión o no la hay.
 *
 * Y las tres capas de tubo —barrido, grano y viñeta—, que se montan aquí
 * porque valen para el juego y para la pantalla de entrada por igual.
 */
export function App() {
  const token = useAppSelector((state) => state.auth.token);

  return (
    <>
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route path="/register" element={<RegisterPage />} />
        <Route
          path="/admin/*"
          element={token ? <AdminShell /> : <Navigate to="/login" replace />}
        />
        <Route path="/*" element={token ? <GameShell /> : <Navigate to="/login" replace />} />
      </Routes>

      <div className="capa capa--vineta" aria-hidden="true" />
      <div className="capa capa--barrido" aria-hidden="true" />
      <div className="capa capa--grano" aria-hidden="true" />
    </>
  );
}
