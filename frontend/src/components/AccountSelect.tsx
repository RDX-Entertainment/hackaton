import { tr, useIdioma } from '../i18n/text';
import type { UnknownRow } from '../utils/api';
import { formatCell } from '../utils/format';

type AccountSelectProps = {
  accounts: UnknownRow[];
  value: string;
  onChange: (account: string) => void;
  label?: string;
};

/**
 * Picks the bank account that pays for something. The accounts endpoint is
 * untyped, so the fields are read defensively.
 */
export function AccountSelect({
  accounts,
  value,
  onChange,
  label = tr("Paying account"),
}: AccountSelectProps) {
  useIdioma();
  return (
    <label>
      {label}
      <select value={value} onChange={(event) => onChange(event.target.value)}>
        <option value="">{tr("Select an account")}</option>
        {accounts.map((account) => {
          const number = formatCell(account.account);
          return (
            <option key={number} value={number}>
              {number} · ${formatCell(account.cash ?? 0)}
            </option>
          );
        })}
      </select>
    </label>
  );
}
