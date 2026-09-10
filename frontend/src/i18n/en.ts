/** English. Anything missing here falls back to Spanish, not to a bare code. */
export const en: Record<string, string> = {
  'error.UNAUTHORIZED': 'Your session has expired. Sign in again.',
  'error.NOT_FOUND': 'Nothing there.',
  'error.INTERNAL': 'Something broke on our side. Not your fault.',
  'error.BAD_REQUEST': 'That request cannot be served.',

  'error.NO_CERTIFICATION': 'You are missing the certification. It is free at the Academy and takes a minute.',
  'error.NO_SOFT': 'You have no program that can do this. Design one in the Lab and compile it.',
  'error.BAD_CRACKER': 'Your program is weaker than the defence in front of it. Raise its version or pick a softer target.',
  'error.SERVER_DOWN': 'That machine is down. Try again in a while.',
  'error.NOT_LOGGED_IN': 'You have not got into that machine yet. The ways in are under «force».',
  'error.NO_ACCESS': 'You have no access to that machine.',
  'error.TARGET_GONE': 'That machine no longer exists.',
  'error.PAYLOAD_GONE': 'The program carrying the operation is gone from your disk.',
  'error.EXPLOIT_BLOCKED': 'The firewall turned your exploits away.',

  'error.DISK_FULL': 'It does not fit on any disk of yours. Free some space or buy another.',
  'error.NO_DISK_SPACE': 'No room on that disk.',
  'error.COMPILATION_ACTIVE': 'You already have a build running. Finish that one first.',
  'error.CANNOT_TRANSFER': 'That file cannot be moved.',
  'error.SOFTWARE_RUNNING': 'It is loaded in memory. Unload it first.',
  'error.REQUIRED_SOFTWARE_NOT_RUNNING': 'You need that program loaded in memory, not just stored on disk.',
  'error.INSUFFICIENT_RESOURCES': 'Your machine does not have the resources for this.',
  'error.SKILL_LOCKED': 'Your program cannot do that yet. It needs more of one particular attribute.',
  'error.INSUFFICIENT_PERMISSION': 'The access you hold does not reach that far.',
  'error.SAME_MACHINE': 'Source and destination are the same machine.',
  'error.NO_MACHINE': 'That player has no machine to put this on.',

  'error.INSUFFICIENT_FUNDS': 'Not enough money.',
  'error.BAD_PASSWORD': 'That is not the password.',
  'error.PIN_ALREADY_SET': 'That account already has a PIN.',

  'error.TOO_MANY_MACHINES': 'No slots left. Buy one before building another machine.',
  'error.LAST_MACHINE': 'It is your last machine. With none you could not play.',
  'error.MACHINE_NOT_EMPTY': 'Empty the machine before decommissioning it.',
  'error.NO_FREE_SLOT': 'No free slot of that kind left.',
  'error.NO_SLOTS': 'The board takes no parts of that kind.',
  'error.CHIPSET_MISMATCH': 'That part does not fit the board chipset.',
  'error.DDR_MISMATCH': 'That memory is not the type the board takes.',
  'error.OVER_CAPACITY': 'The board does not reach that far.',
  'error.PART_IN_USE': 'That part is installed. Remove it first.',
  'error.RAM_IN_USE': 'Loaded programs need that memory.',
  'error.DISK_IN_USE': 'There are files stored on that disk.',
  'error.NOT_CABLED': 'Those two machines are not linked.',
  'error.NO_ROUTE': 'There is no route there.',

  'error.WORKER_ERROR': 'The task could not be finished.',
  'error.UNKNOWN_LANGUAGE': 'That language is not one the game speaks.',

  // The circuit: the five steps.
  'onboarding.modules.step': 'Research',
  'onboarding.modules.title': 'Get modules to build with',
  'onboarding.modules.hint': 'A program is a capability plus the modules you fit to it. With no modules you only get bare programs.',
  'onboarding.modules.action': 'Go to Research',

  'onboarding.blueprint.step': 'Design',
  'onboarding.blueprint.title': 'Design your first program',
  'onboarding.blueprint.hint': 'Pick what it should do — break into a machine, leave something behind — and fit up to three modules. You see what comes out before spending a second compiling it.',
  'onboarding.blueprint.action': 'Design in the Lab',

  'onboarding.compile.step': 'Compile',
  'onboarding.compile.title': 'Compile the design',
  'onboarding.compile.hint': 'Compiling takes time, and there is no file until it finishes. You can only have one build running.',
  'onboarding.compile.action': 'Compile in the Lab',
  'onboarding.compile.waiting': 'Building. When it finishes, the file lands on the disk of whichever machine has room.',

  'onboarding.install.step': 'Load',
  'onboarding.install.title': 'Load it into memory',
  'onboarding.install.hint': 'What sits on disk does not fire. A program only works on the machine it is loaded on, and holds that machine RAM while it is.',
  'onboarding.install.action': 'Load from Software',

  'onboarding.intrusion.step': 'Break in',
  'onboarding.intrusion.title': 'Break into your first machine',
  'onboarding.intrusion.hint': 'You have what it takes. These two addresses are yours and exist for this: nobody else sees them, and they do not mind.',
  'onboarding.intrusion.action': 'Open Internet',

  'onboarding.done.title': 'The circuit is closed',
  'onboarding.done.hint': 'You design, compile, load and break in. From here the game is yours: missions, banks and whatever you come up with.',

  'onboarding.practice.title': 'Your firing range',
  'onboarding.practice.hint': 'Two practice machines only you can see.',
  'onboarding.rail.title': 'The circuit of a program',

  //
  // El tutorial: lo que dice cuando algo esta cerrado, y por que.
  //
  'error.TUTORIAL_LOCKED':
    'Not that, not yet. Finish the tutorial step you are on and this opens on its own.',
  'error.CERTIFICATION_BY_DOING':
    'This certification is not studied: it is earned by playing. Do what the tutorial asks and it is yours.',
  'tutorial.locked':
    'Locked until you finish: {paso}',
  'tutorial.go':
    'Go to the step',
  'tutorial.earned':
    'Earned by doing',
};
