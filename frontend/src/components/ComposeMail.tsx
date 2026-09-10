import { useEffect, useState, type FormEvent } from 'react';

import { mailApi } from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';
import { StatusMessage } from './StatusMessage';

const LARGO_ASUNTO = 40;

type ComposeMailProps = {
  /** Se llama al enviar, para que quien lo monta recargue su lista. */
  onSent: () => void;
  /** Cerrar el redactor. Sin esto no se puede salir sin enviar. */
  onCerrar?: () => void;
  /** Destinatario y asunto ya puestos, para responder. */
  para?: string;
  asunto?: string;
};

/** El redactor. Vive dentro del panel de lectura, en el sitio del mensaje. */
export function ComposeMail({ onSent, onCerrar, para = '', asunto = '' }: ComposeMailProps) {
  const status = useStatusMessage();

  const [destinatario, setDestinatario] = useState(para);
  const [titulo, setTitulo] = useState(asunto);
  const [texto, setTexto] = useState('');

  // Responder a otro mensaje reutiliza el mismo formulario: si no se repusieran,
  // seguiria puesto el destinatario del anterior.
  useEffect(() => {
    setDestinatario(para);
    setTitulo(asunto);
  }, [para, asunto]);

  const enviar = async (event: FormEvent) => {
    event.preventDefault();

    try {
      await mailApi.sendMail(destinatario, titulo, texto);
      setTexto('');
      status.report('Enviado.');
      onSent();
      onCerrar?.();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <form className="correo__redactar" onSubmit={enviar}>
      <label className="correo__campo">
        <span className="micro">para</span>
        <input
          value={destinatario}
          placeholder="nombre del jugador"
          onChange={(event) => setDestinatario(event.target.value)}
          required
        />
      </label>

      <label className="correo__campo">
        <span className="micro">asunto</span>
        <input
          value={titulo}
          maxLength={LARGO_ASUNTO}
          onChange={(event) => setTitulo(event.target.value)}
          required
        />
      </label>

      <textarea
        className="correo__cuerpo-editable"
        value={texto}
        placeholder="Escribe aquí."
        onChange={(event) => setTexto(event.target.value)}
        required
      />

      <div className="correo__acciones">
        <StatusMessage text={status.message} />
        {onCerrar && (
          <button type="button" onClick={onCerrar}>
            Descartar
          </button>
        )}
        <button>Enviar</button>
      </div>
    </form>
  );
}
