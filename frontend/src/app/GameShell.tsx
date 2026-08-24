import { Route, Routes } from 'react-router-dom';

import { TopBar } from '../components/shell/TopBar';
import { ForumPage } from '../pages/ForumPage';
import { ForumThreadPage } from '../pages/ForumThreadPage';
import { HomePage } from '../pages/HomePage';
import { WikiIndexPage } from '../pages/WikiIndexPage';
import { WikiPage } from '../pages/WikiPage';
import { LegacyRoute } from './LegacyRoute';
import { LEGACY_ITEMS } from './navigation';

/**
 * The frame around every authenticated screen. All the chrome lives in TopBar;
 * what is left here is the routing table, which is all this file should ever
 * have been.
 */
export function GameShell() {
  return (
    <div className="shell">
      <TopBar />

      <main className="game">
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
  );
}
