import { useEffect, useRef, useState, type CSSProperties } from 'react';
import { createPortal } from 'react-dom';
import { Link, useLocation } from 'react-router-dom';

import { useTutorial } from '../../app/tutorial';
import { t } from '../../i18n';
import { formatIp } from '../../utils/ip';
import { RUTA_DEL_PASO } from './circuito';
import { useGuideTarget } from './useGuideTarget';
import '../../styles/mission-guide.css';

export function MissionGuide() {
  const { onboarding } = useTutorial();
  const location = useLocation();
  const [folded, setFolded] = useState(false);
  const [finished, setFinished] = useState(false);
  const previous = useRef<string | null>(null);
  const revealed = useRef('');
  const card = useRef<HTMLElement>(null);
  const [height, setHeight] = useState(320);
  const step = onboarding?.current ?? null;
  const route = `/${onboarding?.rules?.focus_view ?? RUTA_DEL_PASO[step ?? '']?.slice(1) ?? ''}`;
  const here = location.pathname.replace(/\/$/, '') === route.replace(/\/$/, '');
  const waiting = step === 'compile' && Boolean(onboarding?.compiling);
  const anchor = here && !folded && !waiting ? onboarding?.rules?.focus_anchor ?? null : null;
  const target = useGuideTarget(anchor, location.pathname, step && !here && !folded && !waiting ? route : null);

  useEffect(() => {
    if (!onboarding) return;
    if (previous.current && !step) setFinished(true);
    if (step !== previous.current) setFolded(false);
    previous.current = step;
  }, [step, onboarding]);

  useEffect(() => {
    if (!card.current) return;
    const observer = new ResizeObserver(([entry]) => setHeight(entry.target.getBoundingClientRect().height));
    observer.observe(card.current);
    return () => observer.disconnect();
  }, [Boolean(step), finished, folded]);

  useEffect(() => {
    if (!step && !finished) return;
    document.documentElement.style.setProperty('--mission-space', `${height + 32}px`);
    document.body.classList.toggle('mission-active', !folded);
    return () => {
      document.documentElement.style.removeProperty('--mission-space');
      document.body.classList.remove('mission-active');
    };
  }, [step, finished, folded, height]);

  useEffect(() => {
    const key = `${step}:${location.pathname}:${anchor}:${target?.hint}:${height}:${window.innerWidth}`;
    if (!target || revealed.current === key) return;
    revealed.current = key;
    const active = document.activeElement;
    if (active?.matches('input, select, textarea')) return;
    target.element.scrollIntoView({ block: 'center', behavior: 'instant' });
  }, [step, location.pathname, anchor, target, height]);

  if (!onboarding || (!step && !finished)) return null;

  const total = onboarding.steps.length;
  const done = onboarding.steps.filter((one) => one.done).length;
  const index = Math.max(0, onboarding.steps.findIndex((one) => one.code === step));
  const title = step ? t(`onboarding.${step}.title`) ?? step : t('mission.complete');
  const hint = finished && !step ? t('mission.complete.hint')
    : waiting ? t('onboarding.compile.waiting')
    : target?.hint ? t(`mission.hint.${target.hint}`)
    : onboarding.rules?.hint || t(`mission.${step}`) || t(`onboarding.${step}.hint`);

  const width = Math.min(352, window.innerWidth - 24);
  const rect = target?.rect;
  const onscreen = rect && rect.bottom > 110 && rect.top < window.innerHeight - 40;
  let placement: CSSProperties = { right: 12, bottom: 12, width };
  let docked = true;
  if (onscreen && window.innerWidth > 900 && window.innerHeight > height + 140) {
    const right = rect.right + 24;
    const left = rect.left - width - 24;
    if (right + width < window.innerWidth - 12 || left > 90) {
      placement = {
        width,
        left: right + width < window.innerWidth - 12 ? right : left,
        top: Math.max(84, Math.min(rect.top, window.innerHeight - height - 12)),
      };
      docked = false;
    }
  }

  const reveal = () => {
    target?.element.scrollIntoView({ block: 'center', behavior: 'instant' });
    target?.element.focus({ preventScroll: true });
  };

  return createPortal(
    <>
      <aside ref={card} className={`mission-guide${waiting ? ' is-waiting' : ''}${folded ? ' is-folded' : ''}`}
        style={placement} aria-label={t('mission.label') ?? 'Misión de iniciación'} data-docked={docked}>
        <header className="mission-guide__header">
          <span className="mission-guide__signal" aria-hidden="true">◈</span>
          <span>{t('mission.channel')}</span>
          {step && <button type="button" onClick={() => setFolded(!folded)}
            aria-expanded={!folded} aria-label={t(folded ? 'mission.expand' : 'mission.fold') ?? ''}>
            {folded ? '+' : '−'}
          </button>}
        </header>
        <div className="mission-guide__content" aria-live="polite" aria-atomic="true">
          <div className="mission-guide__eyebrow">
            <span>{t(step ? 'mission.operation' : 'mission.confirmed')}</span>
            <b>{step ? `${String(index + 1).padStart(2, '0')} / ${String(total).padStart(2, '0')}` : '✓'}</b>
          </div>
          <h2>{title}</h2>
          {!folded && <p>{hint}</p>}
          <div className="mission-guide__progress" role="progressbar" aria-label={t('mission.progress') ?? ''}
            aria-valuemin={0} aria-valuemax={total} aria-valuenow={done}>
            {onboarding.steps.map((one) => <i key={one.code}
              className={one.done ? 'is-done' : one.code === step ? 'is-current' : ''} />)}
          </div>
        </div>
        {!folded && (
          <footer className="mission-guide__footer">
            {!step ? <button type="button" onClick={() => setFinished(false)}>{t('mission.finish')}</button>
              : !here ? <Link to={route}>{t(`onboarding.${step}.action`) ?? t('tutorial.go')} →</Link>
              : waiting ? <Link to="/processes">{t('mission.queue')} →</Link>
              : target ? <button type="button" onClick={reveal}>{t('mission.locate')} ↗</button>
              : <span className="mission-guide__missing">{t('mission.find')}</span>}
            {step === 'intrusion' && onboarding.practice.length > 0 && (!here || !target) && (
              <div className="mission-guide__targets">
                <span>{t('onboarding.practice.title')}</span>
                {onboarding.practice.map((one) => <Link key={one.ip}
                  to={`/internet?practice=${one.ip}`} title={one.name}>{formatIp(one.ip)} ↗</Link>)}
              </div>
            )}
            {step && <small>{t(waiting || target?.hint === 'waiting' ? 'mission.pending' : 'mission.automatic')}</small>}
          </footer>
        )}
      </aside>
    </>, document.body,
  );
}
