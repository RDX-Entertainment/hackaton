import { useState, type ChangeEvent } from 'react';

import { authenticated } from '../../app/authSlice';
import { useAppDispatch, useAppSelector } from '../../app/hooks';
import {
  IDIOMAS,
  NOMBRE_DEL_IDIOMA,
  esIdioma,
  t,
  usarIdioma,
  useIdioma,
} from '../../i18n';
import { authApi } from '../../utils/api';
import { messageFrom } from '../../utils/errors';

/**
 * El selector que acompaña al jugador en las dos barras.
 *
 * La primera lengua ya la resolvió el catálogo a partir del navegador. Desde el momento
 * en que el jugador elige una, se guarda en el servidor y viaja dentro de su sesión: esa
 * elección pasa a mandar sobre el navegador, también cuando entra desde otro dispositivo.
 */
export function LanguageSelect() {
  const dispatch = useAppDispatch();
  const session = useAppSelector((state) => state.auth);
  const idioma = useIdioma();
  const [guardando, setGuardando] = useState(false);
  const [error, setError] = useState<string>();

  const cambiar = async (event: ChangeEvent<HTMLSelectElement>) => {
    const siguiente = event.target.value;
    if (!esIdioma(siguiente) || siguiente === idioma) {
      return;
    }

    setError(undefined);

    // También funciona sin sesión por si la pieza se reutiliza en la entrada. En las barras
    // actuales siempre hay token y la elección queda además guardada en la cuenta.
    if (!session.token) {
      usarIdioma(siguiente);
      return;
    }

    setGuardando(true);
    try {
      const user = await authApi.setLanguage(siguiente);
      dispatch(authenticated({ token: session.token, user }));
    } catch (cause) {
      setError(messageFrom(cause));
    } finally {
      setGuardando(false);
    }
  };

  const etiqueta = t('language.label') ?? 'Idioma';

  return (
    <label className="selector-idioma" title={error}>
      <span className="selector-idioma__label">{etiqueta}</span>
      <select
        value={idioma}
        onChange={(event) => void cambiar(event)}
        disabled={guardando}
        aria-label={etiqueta}
        aria-invalid={error ? true : undefined}
      >
        {IDIOMAS.map((codigo) => (
          <option key={codigo} value={codigo}>
            {NOMBRE_DEL_IDIOMA[codigo]}
          </option>
        ))}
      </select>
    </label>
  );
}
