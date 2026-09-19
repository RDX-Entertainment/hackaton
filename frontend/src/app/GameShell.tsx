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
import { Cerrada } from '../components/onboarding/Cerrada';
import { TutorialProvider, useTutorial } from './tutorial';
import { MissionGuide } from '../components/onboarding/MissionGuide';
import { t } from '../i18n';

/**
 * El armazón de toda pantalla con sesión: el HUD arriba, el carril de destinos
 * a la izquierda, la tira del destino sobre la página, y la tabla de rutas,
 * que es lo único que este fichero debió tener nunca.
 *
 * El tutorial envuelve el armazón entero: lo que deja hacer cada paso lo preguntan el carril,
 * la tira, la guía y los botones comparten la respuesta. Se refresca al actuar y cada cuatro
 * segundos mientras el worker resuelve lo pendiente.
 */
export function GameShell() {
  return (
    <TutorialProvider>
      <Armazon />
    </TutorialProvider>
  );
}

/**
 * Una pantalla cerrada por el tutorial no se sirve, ni siquiera escribiendo su dirección.
 *
 * Y no por seguridad —lo que de verdad importa lo veta el servidor— sino por coherencia: un
 * carril que dice que Banco está cerrado y una barra de direcciones que lo abre igualmente
 * son dos juegos distintos, y el jugador acaba sin saber cuál de los dos le está contando la
 * verdad.
 */
function Vigilada({ path, label }: { path: string; label: string }) {
  const tutorial = useTutorial();

  if (!tutorial.permiteVista(path)) {
    return <Cerrada titulo={label} motivo={tutorial.motivo} />;
  }

  return <LegacyRoute path={path} label={label} />;
}

function Armazon() {
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
                  element={
                    <Vigilada
                      path={item.path}
                      label={t(`nav.screen.${item.path}`) ?? item.label}
                    />
                  }
                />
              ))}
            </Routes>
          </div>
        </main>
      </div>
      <MissionGuide />
    </div>
  );
}
