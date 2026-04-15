export const CONFIG_VERSION = 2 as const;
export const SUPPORTED_PORT = 1 as const;
export const PIPE_TARGET_LABEL = 'Pipe/0/slippibot1';
export const PROFILE_FILE_NAME = 'key-b0x.ini';

export const NORMALIZED_KEYS = [
  'Digit0',
  'Digit1',
  'Digit2',
  'Digit3',
  'Digit4',
  'Digit5',
  'Digit6',
  'Digit7',
  'Digit8',
  'Digit9',
  'KeyA',
  'KeyB',
  'KeyC',
  'KeyD',
  'KeyE',
  'KeyF',
  'KeyG',
  'KeyH',
  'KeyI',
  'KeyJ',
  'KeyK',
  'KeyL',
  'KeyM',
  'KeyN',
  'KeyO',
  'KeyP',
  'KeyQ',
  'KeyR',
  'KeyS',
  'KeyT',
  'KeyU',
  'KeyV',
  'KeyW',
  'KeyX',
  'KeyY',
  'KeyZ',
  'Minus',
  'Equal',
  'BracketLeft',
  'BracketRight',
  'Backslash',
  'Semicolon',
  'Quote',
  'Backquote',
  'Comma',
  'Period',
  'Slash',
  'Space',
  'Tab',
  'Enter',
  'Backspace',
  'Escape',
  'CapsLock',
  'ShiftLeft',
  'ShiftRight',
  'ControlLeft',
  'ControlRight',
  'AltLeft',
  'AltRight',
  'MetaLeft',
  'MetaRight',
  'ArrowUp',
  'ArrowDown',
  'ArrowLeft',
  'ArrowRight'
] as const;

export type NormalizedKey = (typeof NORMALIZED_KEYS)[number];

export const BINDING_IDS = [
  'analog_up',
  'analog_down',
  'analog_left',
  'analog_right',
  'mod_x',
  'mod_y',
  'a',
  'b',
  'l',
  'r',
  'x',
  'y',
  'z',
  'c_up',
  'c_down',
  'c_left',
  'c_right',
  'light_shield',
  'mid_shield',
  'start',
  'd_up',
  'd_down',
  'd_left',
  'd_right'
] as const;

export type BindingId = (typeof BINDING_IDS)[number];

export type BindingMap = Record<BindingId, NormalizedKey>;

export interface AppConfig {
  version: typeof CONFIG_VERSION;
  slippi_user_path: string;
  port: typeof SUPPORTED_PORT;
  bindings: BindingMap;
}

export interface SetupStatus {
  slippiUserPath: string;
  slippiFound: boolean;
  profileInstalled: boolean;
  profilePath: string;
  pipeTargetLabel: string;
  error?: string | null;
}

export type RuntimeStatus =
  | 'idle'
  | 'starting'
  | 'running'
  | 'waiting_for_slippi'
  | 'stopping'
  | 'error';

export interface RuntimeState {
  status: RuntimeStatus;
  startedAt?: number | null;
  lastError?: string | null;
}

export interface InstallProfileResult {
  profilePath: string;
  pipesPath?: string | null;
}

export interface BindingGroup {
  id: string;
  title: string;
  description: string;
  bindings: BindingId[];
}

export const BINDING_LABELS: Record<BindingId, string> = {
  analog_up: 'Analog Up',
  analog_down: 'Analog Down',
  analog_left: 'Analog Left',
  analog_right: 'Analog Right',
  mod_x: 'ModX',
  mod_y: 'ModY',
  a: 'A',
  b: 'B',
  l: 'L',
  r: 'R',
  x: 'X',
  y: 'Y',
  z: 'Z',
  c_up: 'C-stick Up',
  c_down: 'C-stick Down',
  c_left: 'C-stick Left',
  c_right: 'C-stick Right',
  light_shield: 'Light Shield',
  mid_shield: 'Mid Shield',
  start: 'Start',
  d_up: 'D-pad Up',
  d_down: 'D-pad Down',
  d_left: 'D-pad Left',
  d_right: 'D-pad Right'
};

export const DEFAULT_BINDINGS: BindingMap = {
  analog_up: 'BracketRight',
  analog_down: 'Digit3',
  analog_left: 'Digit2',
  analog_right: 'Digit4',
  mod_x: 'KeyV',
  mod_y: 'KeyB',
  a: 'KeyM',
  b: 'KeyO',
  l: 'KeyQ',
  r: 'Digit9',
  x: 'KeyP',
  y: 'Digit0',
  z: 'BracketLeft',
  c_up: 'KeyK',
  c_down: 'Space',
  c_left: 'KeyN',
  c_right: 'Comma',
  light_shield: 'Minus',
  mid_shield: 'Equal',
  start: 'Digit7',
  d_up: 'ArrowUp',
  d_down: 'ArrowDown',
  d_left: 'ArrowLeft',
  d_right: 'ArrowRight'
};

export const BINDING_GROUPS: BindingGroup[] = [
  {
    id: 'movement',
    title: 'Movement',
    description: 'Main stick directions and analog modifiers.',
    bindings: ['analog_left', 'analog_down', 'analog_right', 'analog_up', 'mod_x', 'mod_y']
  },
  {
    id: 'face',
    title: 'Face Buttons',
    description: 'Primary action buttons and start.',
    bindings: ['a', 'b', 'x', 'y', 'z', 'start']
  },
  {
    id: 'cstick',
    title: 'C-Stick',
    description: 'Dedicated smash and Firefox angles.',
    bindings: ['c_up', 'c_down', 'c_left', 'c_right']
  },
  {
    id: 'shield',
    title: 'Shield',
    description: 'Digital shoulders plus analog shield strengths.',
    bindings: ['l', 'r', 'light_shield', 'mid_shield']
  },
  {
    id: 'dpad',
    title: 'D-Pad',
    description: 'Menu navigation and mapped D-pad inputs.',
    bindings: ['d_up', 'd_down', 'd_left', 'd_right']
  }
];

const KEY_LABELS: Record<NormalizedKey, string> = {
  Digit0: '0',
  Digit1: '1',
  Digit2: '2',
  Digit3: '3',
  Digit4: '4',
  Digit5: '5',
  Digit6: '6',
  Digit7: '7',
  Digit8: '8',
  Digit9: '9',
  KeyA: 'A',
  KeyB: 'B',
  KeyC: 'C',
  KeyD: 'D',
  KeyE: 'E',
  KeyF: 'F',
  KeyG: 'G',
  KeyH: 'H',
  KeyI: 'I',
  KeyJ: 'J',
  KeyK: 'K',
  KeyL: 'L',
  KeyM: 'M',
  KeyN: 'N',
  KeyO: 'O',
  KeyP: 'P',
  KeyQ: 'Q',
  KeyR: 'R',
  KeyS: 'S',
  KeyT: 'T',
  KeyU: 'U',
  KeyV: 'V',
  KeyW: 'W',
  KeyX: 'X',
  KeyY: 'Y',
  KeyZ: 'Z',
  Minus: '-',
  Equal: '=',
  BracketLeft: '[',
  BracketRight: ']',
  Backslash: '\\',
  Semicolon: ';',
  Quote: "'",
  Backquote: '`',
  Comma: ',',
  Period: '.',
  Slash: '/',
  Space: 'Space',
  Tab: 'Tab',
  Enter: 'Enter',
  Backspace: 'Backspace',
  Escape: 'Esc',
  CapsLock: 'Caps Lock',
  ShiftLeft: 'Left Shift',
  ShiftRight: 'Right Shift',
  ControlLeft: 'Left Ctrl',
  ControlRight: 'Right Ctrl',
  AltLeft: 'Left Alt',
  AltRight: 'Right Alt',
  MetaLeft: 'Left Meta',
  MetaRight: 'Right Meta',
  ArrowUp: 'Up',
  ArrowDown: 'Down',
  ArrowLeft: 'Left',
  ArrowRight: 'Right'
};

export function isNormalizedKey(value: string): value is NormalizedKey {
  return (NORMALIZED_KEYS as readonly string[]).includes(value);
}

export function createDefaultConfig(defaultSlippiUserPath: string): AppConfig {
  return {
    version: CONFIG_VERSION,
    slippi_user_path: defaultSlippiUserPath,
    port: SUPPORTED_PORT,
    bindings: { ...DEFAULT_BINDINGS }
  };
}

export function normalizeConfig(input: unknown, defaultSlippiUserPath: string): AppConfig {
  if (!isRecord(input)) {
    return createDefaultConfig(defaultSlippiUserPath);
  }

  const version = input.version;
  if (typeof version === 'number' && version !== CONFIG_VERSION) {
    throw new Error(
      `unsupported key-b0x config version ${version}; delete it and regenerate a v${CONFIG_VERSION} config`
    );
  }

  const bindings = { ...DEFAULT_BINDINGS };
  if (isRecord(input.bindings)) {
    for (const binding of BINDING_IDS) {
      const candidate = input.bindings[binding];
      if (typeof candidate === 'string' && isNormalizedKey(candidate)) {
        bindings[binding] = candidate;
      }
    }
  }

  return {
    version: CONFIG_VERSION,
    slippi_user_path:
      typeof input.slippi_user_path === 'string' && input.slippi_user_path.trim().length > 0
        ? input.slippi_user_path
        : defaultSlippiUserPath,
    port: SUPPORTED_PORT,
    bindings
  };
}

export function formatBindingLabel(binding: BindingId): string {
  return BINDING_LABELS[binding];
}

export function formatKeyLabel(key: NormalizedKey): string {
  return KEY_LABELS[key];
}

export function findDuplicateBindings(bindings: BindingMap): Array<{
  key: NormalizedKey;
  bindings: BindingId[];
}> {
  const grouped = new Map<NormalizedKey, BindingId[]>();

  for (const binding of BINDING_IDS) {
    const key = bindings[binding];
    const bucket = grouped.get(key) ?? [];
    bucket.push(binding);
    grouped.set(key, bucket);
  }

  return Array.from(grouped.entries())
    .filter(([, assignedBindings]) => assignedBindings.length > 1)
    .map(([key, duplicateBindings]) => ({ key, bindings: duplicateBindings }));
}

function isRecord(value: unknown): value is Record<string, any> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
