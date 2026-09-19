import { tr, useIdioma } from '../i18n/text';
import type { UnknownRow } from '../utils/api';
import { formatCell } from '../utils/format';

type DataTableProps = {
  rows: UnknownRow[];
  /** Shown instead of an empty table. */
  emptyMessage?: string;
};

const columnLabels: Record<string, string> = {
  id: 'ID', login: 'Usuario', name: 'Nombre', title: 'Título',
  rank: 'Puesto', exp: 'Experiencia', hacks: 'Entradas',
  money: 'Dinero', amount: 'Cantidad', balance: 'Saldo',
  date: 'Fecha', created_at: 'Fecha', date_joined: 'Desde',
  status: 'Estado', type: 'Tipo', description: 'Descripción',
  content: 'Contenido', ip: 'IP', game_ip: 'IP', user_id: 'Usuario',
  email: 'Correo electrónico', author: 'Autor', level: 'Nivel',
};

/**
 * Renders whatever an endpoint returns, taking the first row's keys as the
 * columns. It is the fallback for screens that have no bespoke layout yet.
 */
export function DataTable({ rows, emptyMessage = tr("No records.") }: DataTableProps) {
  useIdioma();
  if (!rows.length) {
    return <p className="muted">{emptyMessage}</p>;
  }

  const columns = Object.keys(rows[0]);

  return (
    <div className="table-wrap">
      <table>
        <thead>
          <tr>
            {columns.map((column) => (
              <th key={column}>{tr(columnLabels[column] ?? column)}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row, index) => (
            <tr key={String(row.id ?? index)}>
              {columns.map((column) => (
                <td key={column}>{formatCell(row[column])}</td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
