import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { Provider } from 'react-redux';
import { BrowserRouter } from 'react-router-dom';

import { App } from './app/App';
import { store } from './app/store';

import './styles/tokens.css';
import './styles/base.css';
import './styles/tables.css';
import './styles/shell.css';
import './styles/terminal.css';
import './styles/hero.css';
import './styles/login.css';
import './styles/profile.css';
import './styles/hardware.css';
import './styles/internet.css';
import './styles/adminshell.css';
import './styles/admin.css';
import './styles/kits.css';
import './styles/forms.css';

const container = document.getElementById('root');

if (!container) {
  throw new Error('index.html is missing the #root element');
}

createRoot(container).render(
  <StrictMode>
    <Provider store={store}>
      <BrowserRouter>
        <App />
      </BrowserRouter>
    </Provider>
  </StrictMode>,
);
