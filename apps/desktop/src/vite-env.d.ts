/// <reference types="vite/client" />

import type {
  AppConfig,
  InstallProfileResult,
  RuntimeState,
  SetupStatus
} from '../shared/model';

declare global {
  interface Window {
    keyB0x: {
      getConfig: () => Promise<AppConfig>;
      saveConfig: (config: AppConfig) => Promise<AppConfig>;
      checkSetup: () => Promise<SetupStatus>;
      installProfile: () => Promise<InstallProfileResult>;
      getRuntimeState: () => Promise<RuntimeState>;
      startRuntime: () => Promise<RuntimeState>;
      stopRuntime: () => Promise<RuntimeState>;
      onRuntimeState: (listener: (state: RuntimeState) => void) => () => void;
    };
  }
}

export {};
