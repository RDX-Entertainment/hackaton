/**
 * One import point for the whole API. Namespaces rather than a flat re-export,
 * because several domains legitimately own a verb of the same name: signing in
 * to the game is `authApi.login`, signing in to someone else's machine is
 * `internetApi.login`.
 */
export * as accountApi from './account';
export * as adminApi from './admin';
export * as authApi from './auth';
export * as bitcoinApi from './bitcoin';
export * as certificationsApi from './certifications';
export * as clansApi from './clans';
export * as dashboardApi from './dashboard';
export * as financesApi from './finances';
export * as forumApi from './forum';
export * as hardwareApi from './hardware';
export * as internetApi from './internet';
export * as listsApi from './lists';
export * as mailApi from './mail';
export * as missionsApi from './missions';
export * as processesApi from './processes';
export * as researchApi from './research';
export * as softwareApi from './software';
export * as wikiApi from './wiki';

export * from './types';
