import { TerminalScreen } from '../components/terminal/TerminalScreen';

/**
 * The same window as /login, opened on the account-creation path in whichever
 * mode the visitor last chose.
 */
export function RegisterPage() {
  return <TerminalScreen initialCommand="useradd" startRegistering />;
}
