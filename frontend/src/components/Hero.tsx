import { tr, useIdioma } from '../i18n/text';
/**
 * The left half of the signed-out page. Only shown in normal mode: switching to
 * the console collapses it and lets the window slide to the middle.
 *
 * `aria-hidden` follows the collapse so a screen reader is not read a title
 * that is visually gone.
 */
type HeroProps = {
  hidden: boolean;
};

export function Hero({ hidden }: HeroProps) {
  useIdioma();
  return (
    <div className="hero" aria-hidden={hidden}>
      <div className="hero__inner">
        <p className="hero__eyebrow">{tr("v0.1.0 · port en curso")}</p>

        <h1 className="hero__title">
          <span className="hero__title-top">Hackaton</span>
          <span className="hero__title-bottom">V-Rustup</span>
        </h1>

        <p className="hero__slogan">
          {tr("El acceso no se pide.")} <br />
          <b>{tr("Se toma.")}</b>
        </p>

        <p className="hero__note">
          {tr("Intrusión, de principio a fin. Cada IP, cada log y cada fichero que robes vive en tu base de datos.")} </p>

        <ul className="hero__facts">
          <li>
            <b>25</b> {tr("acciones de proceso")} </li>
          <li>
            <b>17</b> {tr("migraciones")} </li>
          <li>
            <b>19</b> {tr("comandos en la consola")} </li>
        </ul>
      </div>
    </div>
  );
}
