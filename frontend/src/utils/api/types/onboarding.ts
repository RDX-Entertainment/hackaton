/**
 * El paso en el que está el jugador, deducido del mundo por el servidor.
 *
 * Vienen códigos y no frases: el servidor no sabe en qué lengua lee quien pregunta. La
 * frase, el verbo y la pantalla a la que lleva cada paso los pone este lado.
 */
export type OnboardingStep = {
  code: string;
  done: boolean;
};

export type PracticeTarget = {
  ip: number;
  name: string;
};

export type Onboarding = {
  /** El primer paso sin hacer. `null` cuando ya están todos. */
  current: string | null;
  steps: OnboardingStep[];
  compiling: boolean;
  practice: PracticeTarget[];
};
