import { Menu, app, BrowserWindow, ipcMain } from 'electron';
import path from 'node:path';
import { AppConfig } from '../shared/model';
import { checkSetup, loadConfig, saveConfig } from './configStore';
import { RuntimeService } from './runtimeService';

let mainWindow: BrowserWindow | null = null;
const runtimeService = new RuntimeService();
let quittingAfterRuntimeStop = false;

function createWindow(): void {
  mainWindow = new BrowserWindow({
    width: 1180,
    height: 860,
    minWidth: 980,
    minHeight: 760,
    backgroundColor: '#f3efe7',
    title: 'key-b0x',
    autoHideMenuBar: true,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false
    }
  });

  const devServerUrl = process.env.VITE_DEV_SERVER_URL;
  if (devServerUrl) {
    void mainWindow.loadURL(devServerUrl);
  } else {
    void mainWindow.loadFile(path.join(__dirname, '../dist/index.html'));
  }

  mainWindow.on('closed', () => {
    mainWindow = null;
  });
}

function registerIpc(): void {
  ipcMain.handle('config:get', async () => loadConfig());

  ipcMain.handle('config:save', async (_event, config: AppConfig) => {
    const saved = await saveConfig(config);
    return saved;
  });

  ipcMain.handle('setup:check', async () => checkSetup());

  ipcMain.handle('profile:install', async () => {
    const config = await loadConfig();
    return runtimeService.installProfile(config.slippi_user_path);
  });

  ipcMain.handle('runtime:get-state', async () => runtimeService.getState());
  ipcMain.handle('runtime:start', async () => runtimeService.start());
  ipcMain.handle('runtime:stop', async () => runtimeService.stop());
}

runtimeService.on('state', (state) => {
  mainWindow?.webContents.send('runtime:state', state);
});

app.whenReady().then(() => {
  app.setName('key-b0x');
  Menu.setApplicationMenu(null);
  registerIpc();
  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

app.on('before-quit', (event) => {
  if (quittingAfterRuntimeStop || !runtimeService.hasLiveChild()) {
    return;
  }

  event.preventDefault();
  quittingAfterRuntimeStop = true;
  void runtimeService.stop().finally(() => {
    app.quit();
  });
});

app.on('window-all-closed', () => {
  app.quit();
});
