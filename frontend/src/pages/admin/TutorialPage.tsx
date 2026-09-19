import { tr, useIdioma, idiomaActual } from '../../i18n/text';
import { useCallback, useEffect, useMemo, useState, type FormEvent } from 'react';

import { FOCOS, VISTAS, nombreDeVista } from '../../app/focos';
import { AdminCard } from '../../components/admin/AdminCard';
import { AdminLoadError } from '../../components/admin/AdminLoadError';
import { Loading } from '../../components/Notice';
import { StatusMessage } from '../../components/StatusMessage';
import {
  adminApi,
  type Tutorial,
  type TutorialStep,
  type TutorialStepInput,
  type TutorialVocabulary,
} from '../../utils/api';
import { useStatusMessage } from '../../utils/useStatusMessage';

/** Un paso nuevo empieza sin bloquear nada: se abre eligiendo, no quitando. */
const EN_BLANCO: TutorialStepInput = {
  position: 1,
  code: '',
  fact: 'blueprints',
  views: null,
  actions: null,
  focus_view: null,
  focus_anchor: null,
  hint: null,
  active: true,
};

/**
 * El tutorial, escrito desde aquí.
 *
 * Lo que se escribe no es una lección: es **qué cierra cada paso y qué deja abierto**. Cuatro
 * cosas por paso —el hecho que lo termina, las pantallas, las acciones y el sitio que
 * parpadea— y con eso se monta desde una guía que no estorba hasta un pasillo de una sola
 * puerta.
 *
 * Tres decisiones de esta pantalla:
 *
 *   * **Nada de texto libre donde hay vocabulario.** Las acciones y los hechos los sirve el
 *     servidor, que es quien los va a comprobar; las pantallas y los focos los pone el
 *     cliente, que es quien los tiene. Una errata aquí sería un bloqueo que no se levanta.
 *   * **«Todas» y «solo estas» se preguntan.** Una lista vacía puede querer decir las dos
 *     cosas, y significan lo contrario: todo abierto o todo cerrado.
 *   * **Cada paso se lee en una frase.** Debajo de cada uno está lo que el jugador va a vivir,
 *     escrito en castellano. Es la única forma de ver un bloqueo antes de que lo sufra
 *     alguien.
 */
export function TutorialPage() {
  useIdioma();
  const status = useStatusMessage();

  const [tutorial, setTutorial] = useState<Tutorial>();
  const [vocabulario, setVocabulario] = useState<TutorialVocabulary>();
  const [editando, setEditando] = useState<TutorialStep | 'nuevo' | null>(null);
  const [ocupado, setOcupado] = useState(false);
  const [loadError, setLoadError] = useState<unknown>(null);

  const cargar = useCallback(async () => {
    setLoadError(null);
    try {
      const [datos, vocab] = await Promise.all([
        adminApi.getTutorial(),
        adminApi.getTutorialVocabulary(),
      ]);

      setTutorial(datos);
      setVocabulario(vocab);
    } catch (error) { setLoadError(error); }
  }, []);

  useEffect(() => {
    cargar().catch((error) => status.reportError(error));
  }, [cargar, status]);

  const hacer = async (trabajo: () => Promise<string>) => {
    setOcupado(true);

    try {
      status.report(await trabajo());
      await cargar();
    } catch (error) {
      status.reportError(error);
    } finally {
      setOcupado(false);
    }
  };

  const cambiarInterruptor = (valor: boolean) =>
    void hacer(async () => {
      await adminApi.setTutorialEnforced(valor);
      return valor
        ? tr("El tutorial vuelve a mandar: bloquea lo que no toca.")
        : tr("El tutorial queda como guía: dice el paso y no bloquea nada.");
    });

  const mover = (paso: TutorialStep, salto: number) =>
    void hacer(async () => {
      await adminApi.editTutorialStep(paso.id, {
        ...paso,
        position: Math.max(1, paso.position + salto),
      });

      return tr("«{0}» ahora va {1}.", paso.code, salto < 0 ? 'antes' : tr("después"));
    });

  const borrar = (paso: TutorialStep) =>
    void hacer(async () => {
      await adminApi.removeTutorialStep(paso.id);
      setEditando(null);
      return tr("Paso «{0}» borrado.", paso.code);
    });

  const guardar = (paso: TutorialStep | 'nuevo', valores: TutorialStepInput) =>
    void hacer(async () => {
      if (paso === 'nuevo') {
        await adminApi.createTutorialStep(valores);
      } else {
        await adminApi.editTutorialStep(paso.id, valores);
      }

      setEditando(null);
      return tr("Paso «{0}» guardado.", valores.code);
    });

  if (loadError) return <AdminLoadError title="Tutorial" error={loadError} onRetry={() => void cargar()} />;
  if (!tutorial || !vocabulario) {
    return <Loading />;
  }

  return (
    <>
      <h2>Tutorial</h2>

      <p className="lead">
        {tr("El circuito que recorre quien acaba de registrarse. Mientras aprende, el paso se deduce del mundo. Al terminar, la finalización queda guardada y nunca se reinicia.")} </p>

      <StatusMessage text={status.message} as="block" />

      <AdminCard
        prefix="TUTOR"
        brand="IAL"
        accent="lime"
        note={tr("{0} paso{1}", tutorial.steps.length, tutorial.steps.length === 1 ? '' : 's')}
      >
        <div className={tutorial.enforced ? 'tutmando' : 'tutmando is-off'}>
          <label className="tutmando__sw">
            <input
              type="checkbox"
              checked={tutorial.enforced}
              disabled={ocupado}
              onChange={(evento) => cambiarInterruptor(evento.target.checked)}
            />
            <b>{tutorial.enforced ? tr("El tutorial manda") : tr("El tutorial es solo una guía")}</b>
          </label>

          <p>
            {tutorial.enforced
              ? tr("Bloquea lo que el paso no permite, en el servidor y en la pantalla.")
              : tr("Sigue diciendo qué toca y señalando dónde, pero no cierra nada.")}
          </p>
        </div>

        <ol className="tutlista">
          {tutorial.steps.map((paso, indice) => (
            <li className={paso.active ? 'tutpaso' : 'tutpaso is-off'} key={paso.id}>
              <header className="tutpaso__cab">
                <span className="tutpaso__num">{String(paso.position).padStart(2, '0')}</span>
                <b className="tutpaso__code">{paso.code}</b>
                {!paso.active && <span className="tutpaso__apagado">{tr("apagado")}</span>}
              </header>

              <p className="tutpaso__frase">{enCristiano(paso, vocabulario)}</p>

              <div className="tutpaso__acciones">
                <button type="button" disabled={ocupado} onClick={() => setEditando(paso)}>
                  {tr("editar")} </button>
                <button
                  type="button"
                  disabled={ocupado || indice === 0}
                  onClick={() => mover(paso, -1)}
                >
                  ▲
                </button>
                <button
                  type="button"
                  disabled={ocupado || indice === tutorial.steps.length - 1}
                  onClick={() => mover(paso, 1)}
                >
                  ▼
                </button>
                <button type="button" disabled={ocupado} onClick={() => borrar(paso)}>
                  {tr("borrar")} </button>
              </div>
            </li>
          ))}
        </ol>

        {tutorial.steps.length === 0 && (
          <p className="tutlista__vacia">
            {tr("Sin pasos, el juego no tiene tutorial: nadie ve un siguiente paso y nada se bloquea.")} </p>
        )}

        <div className="tutlista__pie">
          <button type="button" disabled={ocupado} onClick={() => setEditando('nuevo')}>
            {tr("+ Paso nuevo")} </button>
        </div>
      </AdminCard>

      {editando && (
        <EditorDePaso
          paso={editando}
          vocabulario={vocabulario}
          siguientePosicion={tutorial.steps.length + 1}
          ocupado={ocupado}
          onGuardar={(valores) => guardar(editando, valores)}
          onCancelar={() => setEditando(null)}
        />
      )}
    </>
  );
}

/**
 * El paso, contado como lo va a vivir el jugador.
 *
 * Es lo que convierte cuatro campos en algo que se puede juzgar de un vistazo. Sin esta
 * frase, «views: [lab, settings]» hay que traducirlo mentalmente cada vez, y ahí es donde se
 * cuela el paso que encierra a alguien.
 */
function enCristiano(paso: TutorialStep, vocabulario: TutorialVocabulary): string {
  const hecho =
    vocabulario.facts.find((entrada) => entrada.code === paso.fact)?.label ?? paso.fact;

  const pantallas = paso.views
    ? paso.views.length
      ? tr("solo abre {0}", paso.views.map(nombreDeVista).join(', '))
      : tr("no abre ninguna pantalla")
    : tr("puede abrir cualquier pantalla");

  const acciones = paso.actions
    ? paso.actions.length
      ? tr("solo puede: {0}", paso.actions
          .map(
            (codigo) =>
              vocabulario.actions.find((entrada) => entrada.code === codigo)?.label ?? codigo,
          )
          .join('; '))
      : tr("no puede hacer nada")
    : tr("puede hacer cualquier cosa");

  const foco = paso.focus_anchor
    ? tr(" Le parpadea {0}.", FOCOS.find((uno) => uno.code === paso.focus_anchor)?.label ?? paso.focus_anchor)
    : '';

  return tr("Mientras no {0}: {1}, y {2}.{3}", hecho.toLowerCase(), pantallas, acciones, foco);
}

type EditorProps = {
  paso: TutorialStep | 'nuevo';
  vocabulario: TutorialVocabulary;
  siguientePosicion: number;
  ocupado: boolean;
  onGuardar: (valores: TutorialStepInput) => void;
  onCancelar: () => void;
};

function EditorDePaso({
  paso,
  vocabulario,
  siguientePosicion,
  ocupado,
  onGuardar,
  onCancelar,
}: EditorProps) {
  useIdioma();
  const inicial = useMemo<TutorialStepInput>(
    () => (paso === 'nuevo' ? { ...EN_BLANCO, position: siguientePosicion } : { ...paso }),
    [paso, siguientePosicion, idiomaActual()],
  );

  const [valores, setValores] = useState<TutorialStepInput>(inicial);

  useEffect(() => setValores(inicial), [inicial]);

  const cambiar = (parche: Partial<TutorialStepInput>) =>
    setValores((actuales) => ({ ...actuales, ...parche }));

  const alternar = (campo: 'views' | 'actions', codigo: string) => {
    const lista = valores[campo] ?? [];

    cambiar({
      [campo]: lista.includes(codigo)
        ? lista.filter((uno) => uno !== codigo)
        : [...lista, codigo],
    } as Partial<TutorialStepInput>);
  };

  const enviar = (evento: FormEvent) => {
    evento.preventDefault();
    onGuardar(valores);
  };

  const porDestino = useMemo(() => {
    const grupos = new Map<string, typeof VISTAS>();

    for (const vista of VISTAS) {
      grupos.set(vista.destino, [...(grupos.get(vista.destino) ?? []), vista]);
    }

    return [...grupos.entries()];
  }, [idiomaActual()]);

  return (
    <AdminCard
      prefix="PASO"
      brand={paso === 'nuevo' ? 'NUEVO' : valores.code.toUpperCase()}
      note={paso === 'nuevo' ? tr("sin guardar") : `#${valores.position}`}
    >
      <form className="tutform" onSubmit={enviar}>
        <div className="tutform__fila">
          <label>
            {tr("Código del paso")} <input
              value={valores.code}
              maxLength={32}
              required
              onChange={(evento) => cambiar({ code: evento.target.value })}
            />
            <small>
              {tr("Lo que viaja al jugador. Si coincide con uno del catálogo de idiomas —")} <code>modules</code>, <code>blueprint</code>, <code>compile</code>,{' '}
              <code>install</code>, <code>intrusion</code>{tr("— habla en su lengua; si no, dice la frase de aquí abajo.")} </small>
          </label>

          <label>
            {tr("Lo cierra")} <select
              value={valores.fact}
              onChange={(evento) => cambiar({ fact: evento.target.value })}
            >
              {vocabulario.facts.map((hecho) => (
                <option key={hecho.code} value={hecho.code}>
                  {hecho.label}
                </option>
              ))}
            </select>
            <small>
              {tr("El hecho que da el paso por terminado. Se pregunta al mundo cada vez: no hay nada guardado que pueda desincronizarse.")} </small>
          </label>
        </div>

        <fieldset className="tutform__grupo">
          <legend>{tr("Pantallas")}</legend>

          <div className="tutform__modo">
            <label>
              <input
                type="radio"
                checked={valores.views === null}
                onChange={() => cambiar({ views: null })}
              />
              {tr("Todas")} </label>

            <label>
              <input
                type="radio"
                checked={valores.views !== null}
                onChange={() => cambiar({ views: [] })}
              />
              {tr("Solo estas")} </label>
          </div>

          {valores.views !== null && (
            <div className="tutform__vistas">
              {porDestino.map(([destino, vistas]) => (
                <div className="tutform__destino" key={destino}>
                  <b>{destino}</b>

                  {vistas.map((vista) => (
                    <label key={vista.path}>
                      <input
                        type="checkbox"
                        checked={valores.views?.includes(vista.path) ?? false}
                        onChange={() => alternar('views', vista.path)}
                      />
                      {vista.label}
                    </label>
                  ))}
                </div>
              ))}
            </div>
          )}
        </fieldset>

        <fieldset className="tutform__grupo">
          <legend>{tr("Acciones")}</legend>

          <div className="tutform__modo">
            <label>
              <input
                type="radio"
                checked={valores.actions === null}
                onChange={() => cambiar({ actions: null })}
              />
              {tr("Todas")} </label>

            <label>
              <input
                type="radio"
                checked={valores.actions !== null}
                onChange={() => cambiar({ actions: [] })}
              />
              {tr("Solo estas")} </label>
          </div>

          {valores.actions !== null && (
            <div className="tutform__acciones">
              {vocabulario.actions.map((accion) => (
                <label key={accion.code}>
                  <input
                    type="checkbox"
                    checked={valores.actions?.includes(accion.code) ?? false}
                    onChange={() => alternar('actions', accion.code)}
                  />
                  {accion.label}
                </label>
              ))}
            </div>
          )}
        </fieldset>

        <div className="tutform__fila">
          <label>
            {tr("Dónde mira")} <select
              value={valores.focus_anchor ?? ''}
              onChange={(evento) => {
                const foco = FOCOS.find((uno) => uno.code === evento.target.value);

                // La pantalla del foco no se pregunta aparte: el sitio ya sabe en cuál está,
                // y dos campos para un dato son dos formas de que no cuadren.
                cambiar({
                  focus_anchor: foco?.code ?? null,
                  focus_view: foco?.view ?? null,
                });
              }}
            >
              <option value="">{tr("En ningún sitio")}</option>
              {FOCOS.map((foco) => (
                <option key={foco.code} value={foco.code}>
                  {foco.label}
                </option>
              ))}
            </select>
            <small>
              {tr("Ese botón parpadea, y es lo único que parpadea en el juego. Si señalara tres cosas no señalaría ninguna.")} </small>
          </label>

          <label className="tutform__activo">
            <input
              type="checkbox"
              checked={valores.active}
              onChange={(evento) => cambiar({ active: evento.target.checked })}
            />
            {tr("Activo")} <small>
              {tr("Un paso apagado sigue escrito y deja de gobernar: es como se prepara uno nuevo sin soltarlo encima de quien está jugando.")} </small>
          </label>
        </div>

        <label className="tutform__frase">
          {tr("Frase del paso")} <textarea
            value={valores.hint ?? ''}
            rows={2}
            placeholder={tr("Solo hace falta si el código no está en los catálogos de idioma.")}
            onChange={(evento) => cambiar({ hint: evento.target.value || null })}
          />
        </label>

        <div className="tutform__pie">
          <button type="button" disabled={ocupado} onClick={onCancelar}>
            {tr("cancelar")} </button>
          <button className="is-primary" disabled={ocupado || !valores.code.trim()}>
            {tr("guardar el paso")} </button>
        </div>
      </form>
    </AdminCard>
  );
}
