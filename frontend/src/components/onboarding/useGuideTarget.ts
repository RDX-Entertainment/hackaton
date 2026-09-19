import { useEffect, useState } from 'react';

type Target = { element: HTMLElement; rect: DOMRect; hint: string | null };

function visible(element: HTMLElement) {
  return element.getClientRects().length > 0 && getComputedStyle(element).visibility !== 'hidden';
}

function locate(anchor: string): { element: HTMLElement; hint: string | null } | null {
  const guide = (code: string) => [...document.querySelectorAll<HTMLElement>('[data-guia]')]
    .find((element) => element.dataset.guia === code && visible(element));
  const found = (element: HTMLElement | undefined, hint: string | null = null) =>
    element ? { element, hint } : null;

  if (anchor === 'lab.disenar') {
    const name = guide('lab.nombre') as HTMLInputElement | undefined;
    if (name && !name.value.trim()) return found(name, 'name');
    const capability = guide('lab.capacidad') as HTMLSelectElement | undefined;
    if (capability && capability.value !== 'cracker') return found(capability, 'capability');
  }

  if (anchor === 'red.entrar' || anchor === 'red.hackear') {
    const attack = guide('red.cracker');
    if (attack) return found(attack, attack.matches(':disabled') ? 'waiting' : 'attack');
    const force = guide('red.force');
    if (force && force.getAttribute('aria-current') !== 'true') return found(force, 'force');
  }

  const controls = [...document.querySelectorAll<HTMLElement>('[data-foco]')]
    .filter((element) => element.dataset.foco === anchor && visible(element));
  const enabled = controls.find((element) => !element.matches(':disabled'));
  if (enabled) return found(enabled);

  if (anchor === 'lab.disenar' || anchor === 'lab.compilar') {
    const tab = guide('lab.crear');
    if (!guide('lab.nombre') && tab) return found(tab, 'create');
  }
  if (anchor === 'lab.investigar') {
    const tab = guide('lab.investigar');
    if (tab && !tab.classList.contains('is-current')) return found(tab, 'research');
  }
  if (anchor === 'discos.cargar') {
    const file = guide('discos.fichero');
    if (file) return found(file, 'file');
    const volume = guide('discos.volumen');
    if (volume) return found(volume, 'volume');
  }
  return found(controls[0], controls.length ? 'prepare' : null);
}

/** Las anclas pueden aparecer después de una petición, al abrir un disco o cambiar de pestaña. */
export function useGuideTarget(anchor: string | null, route: string, destination: string | null = null) {
  const [target, setTarget] = useState<Target | null>(null);

  useEffect(() => {
    setTarget(null);
    if (!anchor && !destination) return;
    let frame = 0;
    let glowing: HTMLElement | null = null;
    const update = () => {
      const links = destination
        ? [...document.querySelectorAll<HTMLAnchorElement>('.tira a'),
          ...document.querySelectorAll<HTMLAnchorElement>('.carril a')].filter(visible)
        : [];
      const navigation = links.find((link) => link.pathname === destination)
        ?? links.find((link) => link.dataset.guiaVistas?.split(' ').includes(destination!));
      const next = destination ? (navigation ? { element: navigation, hint: null } : null) : locate(anchor!);
      const actionable = next && !next.element.matches(':disabled, [aria-disabled="true"]') ? next.element : null;
      if (glowing !== actionable) glowing?.classList.remove('tutorial-glow');
      glowing = actionable;
      if (glowing && !glowing.classList.contains('tutorial-glow')) glowing.classList.add('tutorial-glow');
      const rect = next?.element.getBoundingClientRect();
      setTarget((previous) => {
        if (!next || !rect) return null;
        if (previous?.element === next.element && previous.hint === next.hint &&
            previous.rect.x === rect.x && previous.rect.y === rect.y &&
            previous.rect.width === rect.width && previous.rect.height === rect.height) return previous;
        return { ...next, rect };
      });
    };
    const schedule = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(update);
    };
    const observer = new MutationObserver(schedule);
    const content = document.querySelector('.shell');
    if (content) observer.observe(content, { subtree: true, childList: true, attributes: true });
    const timer = window.setInterval(schedule, 500);
    window.addEventListener('resize', schedule);
    window.addEventListener('scroll', schedule, true);
    document.addEventListener('input', schedule);
    schedule();
    return () => {
      observer.disconnect();
      glowing?.classList.remove('tutorial-glow');
      clearInterval(timer);
      cancelAnimationFrame(frame);
      window.removeEventListener('resize', schedule);
      window.removeEventListener('scroll', schedule, true);
      document.removeEventListener('input', schedule);
    };
  }, [anchor, route, destination]);

  return target;
}
