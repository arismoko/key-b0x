import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import type {
  AppConfig,
  InstallProfileResult,
  KeyboardTestState,
  RuntimeState,
  SetupStatus
} from '../shared/model';

async function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw new Error(messageFromInvokeError(error));
  }
}

function messageFromInvokeError(error: unknown): string {
  if (typeof error === 'string') {
    return error;
  }

  if (
    typeof error === 'object' &&
    error !== null &&
    'message' in error &&
    typeof error.message === 'string'
  ) {
    return error.message;
  }

  return 'Something went wrong.';
}

export const api = {
  getConfig: (): Promise<AppConfig> => invokeCommand('load_config'),
  saveConfig: (config: AppConfig): Promise<AppConfig> => invokeCommand('save_config', { config }),
  checkSetup: (config?: AppConfig): Promise<SetupStatus> => invokeCommand('check_setup', { config }),
  installProfile: (slippiUserPath?: string): Promise<InstallProfileResult> =>
    invokeCommand('install_profile', { slippiUserPath }),
  getRuntimeState: (): Promise<RuntimeState> => invokeCommand('get_runtime_state'),
  startRuntime: (): Promise<RuntimeState> => invokeCommand('start_runtime'),
  stopRuntime: (): Promise<RuntimeState> => invokeCommand('stop_runtime'),
  getKeyboardTestState: (): Promise<KeyboardTestState> => invokeCommand('get_keyboard_test_state'),
  startKeyboardTest: (): Promise<KeyboardTestState> => invokeCommand('start_keyboard_test'),
  stopKeyboardTest: (): Promise<KeyboardTestState> => invokeCommand('stop_keyboard_test'),
  onRuntimeState: (listener: (state: RuntimeState) => void) => {
    let disposed = false;
    const unlistenPromise = listen<RuntimeState>('runtime://state', (event) => {
      listener(event.payload);
    });

    return () => {
      disposed = true;
      void unlistenPromise.then((unlisten) => {
        if (disposed) {
          unlisten();
        }
      });
    };
  },
  onKeyboardTestState: (listener: (state: KeyboardTestState) => void) => {
    let disposed = false;
    const unlistenPromise = listen<KeyboardTestState>('keyboard-test://state', (event) => {
      listener(event.payload);
    });

    return () => {
      disposed = true;
      void unlistenPromise.then((unlisten) => {
        if (disposed) {
          unlisten();
        }
      });
    };
  },
  async pickSlippiUserPath(currentPath?: string): Promise<string | null> {
    const selected = await open({
      title: 'Select Slippi user folder',
      defaultPath: currentPath,
      directory: true,
      multiple: false
    });

    if (Array.isArray(selected)) {
      return selected[0] ?? null;
    }

    return selected ?? null;
  }
};
