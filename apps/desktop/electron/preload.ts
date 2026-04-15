import { contextBridge, ipcRenderer } from 'electron';
import {
  AppConfig,
  InstallProfileResult,
  RuntimeState,
  SetupStatus
} from '../shared/model';

const api = {
  getConfig: (): Promise<AppConfig> => ipcRenderer.invoke('config:get'),
  saveConfig: (config: AppConfig): Promise<AppConfig> => ipcRenderer.invoke('config:save', config),
  checkSetup: (): Promise<SetupStatus> => ipcRenderer.invoke('setup:check'),
  installProfile: (): Promise<InstallProfileResult> => ipcRenderer.invoke('profile:install'),
  pickSlippiUserPath: (currentPath?: string): Promise<string | null> =>
    ipcRenderer.invoke('path:pick-slippi-user', currentPath),
  getRuntimeState: (): Promise<RuntimeState> => ipcRenderer.invoke('runtime:get-state'),
  startRuntime: (): Promise<RuntimeState> => ipcRenderer.invoke('runtime:start'),
  stopRuntime: (): Promise<RuntimeState> => ipcRenderer.invoke('runtime:stop'),
  onRuntimeState: (listener: (state: RuntimeState) => void) => {
    const wrapped = (_event: unknown, state: RuntimeState) => {
      listener(state);
    };

    ipcRenderer.on('runtime:state', wrapped);

    return () => {
      ipcRenderer.removeListener('runtime:state', wrapped);
    };
  }
};

contextBridge.exposeInMainWorld('keyB0x', api);
