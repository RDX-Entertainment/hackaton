import { Route, Routes } from 'react-router-dom';

import { AdminNav } from '../components/admin/AdminNav';
import { CatalogPage } from '../pages/admin/CatalogPage';
import { KitsPage } from '../pages/admin/KitsPage';
import { OrganizationsPage } from '../pages/admin/OrganizationsPage';
import { SiteActionsPage } from '../pages/admin/SiteActionsPage';
import { SitesPage } from '../pages/admin/SitesPage';
import { UsersPage } from '../pages/admin/UsersPage';

export function AdminShell() {
  return (
    <div className="shell shell--admin">
      <AdminNav />

      <main className="game">
        <div className="game__inner">
          <Routes>
            <Route index element={<CatalogPage />} />
            <Route path="kits" element={<KitsPage />} />
            <Route path="organizaciones" element={<OrganizationsPage />} />
            <Route path="sitios" element={<SitesPage />} />
            <Route path="acciones" element={<SiteActionsPage />} />
            <Route path="usuarios" element={<UsersPage />} />
          </Routes>
        </div>
      </main>
    </div>
  );
}
