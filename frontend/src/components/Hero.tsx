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
  return (
    <div className="hero" aria-hidden={hidden}>
      <div className="hero__inner">
        <p className="hero__eyebrow">v0.1.0 · port en curso</p>

        <h1 className="hero__title">
          <span className="hero__title-top">Hackaton</span>
          <span className="hero__title-bottom">V-Rustup</span>
        </h1>

        <p className="hero__slogan">
          El acceso no se pide.
          <br />
          <b>Se toma.</b>
        </p>

        <p className="hero__note">
          Intrusión, de principio a fin. Cada IP, cada log y cada fichero que
          robes vive en tu base de datos.
        </p>

        <ul className="hero__facts">
          <li>
            <b>25</b> acciones de proceso
          </li>
          <li>
            <b>17</b> migraciones
          </li>
          <li>
            <b>19</b> comandos en la consola
          </li>
        </ul>
      </div>
    </div>
  );
}
