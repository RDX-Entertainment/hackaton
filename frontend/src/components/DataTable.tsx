import type { UnknownRow } from '../utils/api';
import { formatCell } from '../utils/format';

type DataTableProps = {
  rows: UnknownRow[];
  /** Shown instead of an empty table. */
  emptyMessage?: string;
};

/**
 * Renders whatever an endpoint returns, taking the first row's keys as the
 * columns. It is the fallback for screens that have no bespoke layout yet.
 */
export function DataTable({ rows, emptyMessage = 'No records.' }: DataTableProps) {
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
              <th key={column}>{column}</th>
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
