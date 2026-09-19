import { tr } from '../../i18n/text';
import { useCallback, useEffect, useRef, useState } from 'react';

import { bankApi, financesApi, type BankAccess } from '../../utils/api';
import { messageFrom } from '../../utils/errors';
import { formatMoney } from '../../utils/format';
import type { SiteFill, SiteMessage } from './siteRuntime';

type Bank = { id: number; name: string };

/**
 * La banca de una web autorizada.
 *
 * Aquí no se pinta nada: el banco ya trae su login, sus botones y sus huecos en su propio
 * HTML. Esto solo atiende lo que la página pide, habla con el servidor y le devuelve los
 * datos para que los ponga donde quiera.
 *
 * Solo funciona en la web de un banco, y no porque nadie lo prohíba: `bank` llega en `null`
 * salvo que el servidor diga que esa máquina es de una organización con `banking`. En
 * cualquier otra página no hay a quién llamar.
 *
 * La sesión manual vive en memoria. La principal se recupera desde el servidor, sin
 * almacenar ni devolver el PIN, y solo al visitar el banco al que pertenece.
 */
export function useBankSession(bank: Bank | null, ip: number) {
  const [holder, setHolder] = useState<string | null>(null);
  const session = useRef<{ access: BankAccess; pin: string; main?: boolean } | null>(null);
  const replyRef = useRef<((fill: SiteFill) => void) | null>(null);

  // Lo último que se le mandó a la página. Cambiar de sección reemplaza el documento y se
  // lleva por delante lo rellenado, así que hay que poder volver a ponerlo.
  const painted = useRef<SiteFill | null>(null);

  // La banca pertenece a la pestaÃ±a de la mÃ¡quina, no a la organizaciÃ³n. Dos IPs del
  // mismo banco pueden servir la misma web, pero saltar entre ellas equivale a abrir una
  // pestaÃ±a nueva y nunca debe conservar credenciales ni saldo ya pintado.
  useEffect(() => {
    let cancelled = false;
    session.current = null;
    painted.current = null;
    setHolder(null);
    if (bank) void financesApi.mainAccount().then((access) => {
      if (cancelled || access.bank_id !== bank.id || session.current) return;
      session.current = { access, pin: '', main: true };
      setHolder(access.holder);
      const fill: SiteFill = { session: true, values: {
        bank: bank.name, account: String(access.account), holder: access.holder,
        balance: formatMoney(access.cash), status: tr("Cuenta principal activa."),
      } };
      painted.current = fill;
      replyRef.current?.(fill);
    }).catch(() => {});
    return () => { cancelled = true; replyRef.current = null; };
  }, [ip, bank?.id]);

  const handle = useCallback(
    async (message: SiteMessage, reply: (fill: SiteFill) => void) => {
      if (!bank) {
        return;
      }

      const { fields, params } = message;

      // Cada respuesta se guarda además de mandarse, para poder repetirla al cambiar de
      // sección. El estado no se acumula: se guarda entero el último que valía.
      const send = (fill: SiteFill) => {
        // Quién está dentro lo sabe esto, no la página: así una barra puede cambiar sola.
        fill = { ...fill, session: session.current !== null };

        painted.current = {
          ...fill,
          values: { ...painted.current?.values, ...fill.values },
          lists: { ...painted.current?.lists, ...fill.lists },
          clear: false,
        };

        reply(fill);
      };

      const say = (status: string, extra?: Partial<SiteFill>) =>
        send({ values: { status, bank: bank.name }, ...extra });

      const shown = (access: BankAccess) => ({
        bank: bank.name,
        account: String(access.account),
        holder: access.holder,
        balance: formatMoney(access.cash),
      });

      try {
        if (message.kind === 'bank_logout') {
          if (session.current?.main) { say(tr("Tu cuenta principal permanece activa para ti.")); return; }
          session.current = null;
          setHolder(null);
          send({
            values: { status: tr("Sesión cerrada."), account: '', holder: '', balance: '—' },
            clear: true,
          });
          return;
        }

        if (message.kind === 'bank_open') {
          const opened = await financesApi.openAccount(bank.id, fields.pin ?? '');
          send({
            values: {
              status: tr("Cuenta {0} abierta. Entra con ella y ese PIN.", opened.account),
              account: String(opened.account),
            },
            clear: true,
          });
          return;
        }

        if (message.kind === 'bank_login') {
          const access = await bankApi.login({
            account: Number(fields.account),
            pin: fields.pin ?? '',
          });

          session.current = { access, pin: fields.pin ?? '' };
          setHolder(access.holder);
          send({
            values: { ...shown(access), status: tr("Hola, {0}.", access.holder) },
            clear: true,
          });
          return;
        }

        // Lo que queda necesita estar dentro.
        const open = session.current;

        if (!open) {
          say(tr("Entra en tu cuenta primero."));
          return;
        }

        if (message.kind === 'bank_transfer') {
          const amount = Number(fields.amount);
          const movement = open.main ? await financesApi.transferMain(open.access.account, Number(fields.to_account), amount, fields.concept ?? '') : await bankApi.transfer({
            account: open.access.account,
            pin: open.pin,
            to_account: Number(fields.to_account),
            amount,
            concept: fields.concept ?? '',
          });

          const access = { ...open.access, cash: movement.balance };
          session.current = { ...open, access };
          send({
            values: {
              ...shown(access),
              status: tr("Enviados {0} a {1}.", formatMoney(amount), fields.to_account),
            },
            clear: true,
          });
          return;
        }

        if (message.kind === 'bank_statement') {
          const rows = open.main ? await financesApi.mainStatement() : await bankApi.statement({
            account: open.access.account,
            pin: open.pin,
          });

          send({
            values: { ...shown(open.access), status: tr("{0} movimientos.", rows.length) },
            lists: {
              movements: rows.length
                ? rows.map((one) => ({
                amount: `${one.amount > 0 ? '+' : ''}${formatMoney(one.amount)}`,
                concept: one.concept || tr("sin concepto"),
                balance: formatMoney(one.balance),
                other_account: one.other_account ? String(one.other_account) : '—',
                  }))
                : [{
                    amount: '-',
                    concept: tr("Sin movimientos todavia."),
                    balance: formatMoney(open.access.cash),
                    other_account: '-',
                  }],
            },
          });
        }
      } catch (error) {
        say(messageFrom(error));
      }

      void params;
    },
    [bank],
  );

  /** Vuelve a pintar lo que la sesión ya sabía, en la página que acaba de cargar. */
  const restore = useCallback((reply: (fill: SiteFill) => void) => {
    replyRef.current = reply;
    reply(painted.current ?? {
      session: session.current !== null,
    });
  }, []);

  return { holder, handle, restore };
}
