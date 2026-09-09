import { Route, Routes } from 'react-router-dom';

import { Carril, Tira } from '../components/shell/Carril';
import { TopBar } from '../components/shell/TopBar';
import { ForumPage } from '../pages/ForumPage';
import { ForumThreadPage } from '../pages/ForumThreadPage';
import { HomePage } from '../pages/HomePage';
import { WikiIndexPage } from '../pages/WikiIndexPage';
import { WikiPage } from '../pages/WikiPage';
import { LegacyRoute } from './LegacyRoute';
import { LEGACY_ITEMS } from './navigation';

/**
 * El armazón de toda pantalla con sesión: el HUD arriba, el carril de destinos
 * a la izquierda, la tira del destino sobre la página, y la tabla de rutas,
 * que es lo único que este fichero debió tener nunca.
 */
export function GameShell() {
  return (
    <div className="shell">
      <TopBar />

      <div className="shell__cuerpo">
        <Carril />

        <main className="game">
          <Tira />

          <div className="game__inner">
            <Routes>
              <Route index element={<HomePage />} />

              <Route path="wiki" element={<WikiIndexPage />} />
              <Route path="wiki/:slug" element={<WikiPage />} />
              <Route path="forum" element={<ForumPage />} />
              <Route path="forum/:id" element={<ForumThreadPage />} />

              {LEGACY_ITEMS.map((item) => (
                <Route
                  key={item.path}
                  path={item.path}
                  element={<LegacyRoute path={item.path} label={item.label} />}
                />
              ))}
            </Routes>
          </div>
        </main>
      </div>
    </div>
  );
}
