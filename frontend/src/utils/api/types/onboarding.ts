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

/**
 * Lo que el paso actual deja hacer, y dónde mira.
 *
 * `views` y `actions` a `null` significan «todas»; una lista significa «solo estas», y la
 * lista vacía —que también puede escribirse desde el panel— significa «ninguna».
 *
 * El servidor comprueba esto por su cuenta antes de dejar hacer nada. Aquí no se repite la
 * regla: se **explica**, apagando lo que no toca con su motivo. Un botón que se deja pulsar
 * para contestar que no enseña a base de choques.
 */
export type OnboardingRules = {
  views: string[] | null;
  actions: string[] | null;
  focus_view: string | null;
  focus_anchor: string | null;
  /** La frase escrita a mano, para un paso que este cliente no sepa nombrar. */
  hint: string | null;
};

export type Onboarding = {
  /** El primer paso sin hacer. `null` cuando ya están todos. */
  current: string | null;
  steps: OnboardingStep[];
  compiling: boolean;
  practice: PracticeTarget[];

  /** Las reglas del paso actual. `null` cuando no hay paso o el tutorial está apagado. */
  rules: OnboardingRules | null;

  /** Si el tutorial manda. Apagado sigue diciendo el paso y deja de bloquear. */
  enforced: boolean;
};
