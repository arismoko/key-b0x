// @vitest-environment jsdom

import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type {
  AppConfig,
  KeyboardTestState,
  RuntimeState,
  SetupStatus,
  UpdateState
} from '../shared/model';
import App from './App';

const defaultConfig: AppConfig = {
  version: 2,
  slippi_user_path: '/tmp/SlippiOnline',
  onboarding_completed: false,
  port: 1,
  bindings: {
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
  },
  melee: {
    socd: {
      main_x: 'second_input_priority_no_reactivation',
      main_y: 'second_input_priority_no_reactivation',
      c_x: 'second_input_priority_no_reactivation',
      c_y: 'second_input_priority_no_reactivation'
    },
    down_diagonal: 'auto_jab_cancel',
    horizontal_socd_override: 'max_jump_trajectory',
    airdodge: {
      kind: 'default'
    }
  }
};

const incompleteSetup: SetupStatus = {
  slippiUserPath: '/tmp/SlippiOnline',
  slippiFound: false,
  profileInstalled: false,
  profilePath: '/tmp/SlippiOnline/Config/Profiles/GCPad/key-b0x.ini',
  pipeTargetLabel: 'Pipe/0/slippibot1',
  error: null
};

const completeSetup: SetupStatus = {
  ...incompleteSetup,
  slippiFound: true,
  profileInstalled: true
};

const idleRuntime: RuntimeState = {
  status: 'idle',
  startedAt: null,
  lastError: null
};

const runningRuntime: RuntimeState = {
  status: 'running',
  startedAt: 123,
  lastError: null
};

const idleKeyboardTest: KeyboardTestState = {
  status: 'idle',
  pressedKeys: [],
  lastError: null
};

const idleUpdateState: UpdateState = {
  status: 'idle',
  currentVersion: '0.1.0',
  latestVersion: null,
  notes: null,
  publishedAt: null,
  target: null,
  downloadedBytes: null,
  contentLength: null,
  lastError: null
};

const runningKeyboardTest: KeyboardTestState = {
  status: 'running',
  pressedKeys: ['KeyA', 'KeyS', 'Space'],
  lastError: null
};

let runtimeListener: ((state: RuntimeState) => void) | null = null;
let keyboardTestListener: ((state: KeyboardTestState) => void) | null = null;
let updateStateListener: ((state: UpdateState) => void) | null = null;

const mockApi = vi.hoisted(() => ({
  getConfig: vi.fn(),
  saveConfig: vi.fn(),
  checkSetup: vi.fn(),
  installProfile: vi.fn(),
  getRuntimeState: vi.fn(),
  startRuntime: vi.fn(),
  stopRuntime: vi.fn(),
  getKeyboardTestState: vi.fn(),
  startKeyboardTest: vi.fn(),
  stopKeyboardTest: vi.fn(),
  getAppVersion: vi.fn(),
  checkForUpdate: vi.fn(),
  downloadUpdate: vi.fn(),
  installUpdate: vi.fn(),
  pickSlippiUserPath: vi.fn(),
  onRuntimeState: vi.fn((listener: (state: RuntimeState) => void) => {
    runtimeListener = listener;
    return () => {
      runtimeListener = null;
    };
  }),
  onKeyboardTestState: vi.fn((listener: (state: KeyboardTestState) => void) => {
    keyboardTestListener = listener;
    return () => {
      keyboardTestListener = null;
    };
  }),
  onUpdateState: vi.fn((listener: (state: UpdateState) => void) => {
    updateStateListener = listener;
    return () => {
      updateStateListener = null;
    };
  })
}));

vi.mock('./api', () => ({
  api: mockApi
}));

describe('App', () => {
  afterEach(() => {
    cleanup();
  });

  beforeEach(() => {
    runtimeListener = null;
    keyboardTestListener = null;
    updateStateListener = null;
    mockApi.getConfig.mockResolvedValue(structuredClone(defaultConfig));
    mockApi.saveConfig.mockImplementation(async (config) => structuredClone(config));
    mockApi.checkSetup.mockResolvedValue(incompleteSetup);
    mockApi.installProfile.mockResolvedValue({
      profilePath: completeSetup.profilePath,
      pipesPath: '/tmp/SlippiOnline/Pipes'
    });
    mockApi.getRuntimeState.mockResolvedValue(idleRuntime);
    mockApi.startRuntime.mockResolvedValue(runningRuntime);
    mockApi.stopRuntime.mockResolvedValue(idleRuntime);
    mockApi.getKeyboardTestState.mockResolvedValue(idleKeyboardTest);
    mockApi.startKeyboardTest.mockResolvedValue(runningKeyboardTest);
    mockApi.stopKeyboardTest.mockResolvedValue(idleKeyboardTest);
    mockApi.getAppVersion.mockResolvedValue('0.1.0');
    mockApi.checkForUpdate.mockResolvedValue(null);
    mockApi.downloadUpdate.mockResolvedValue(undefined);
    mockApi.installUpdate.mockResolvedValue(undefined);
    mockApi.pickSlippiUserPath.mockResolvedValue(null);
    mockApi.onRuntimeState.mockClear();
    mockApi.onKeyboardTestState.mockClear();
    mockApi.onUpdateState.mockClear();
    mockApi.getConfig.mockClear();
    mockApi.saveConfig.mockClear();
    mockApi.checkSetup.mockClear();
    mockApi.installProfile.mockClear();
    mockApi.getRuntimeState.mockClear();
    mockApi.startRuntime.mockClear();
    mockApi.stopRuntime.mockClear();
    mockApi.getKeyboardTestState.mockClear();
    mockApi.startKeyboardTest.mockClear();
    mockApi.stopKeyboardTest.mockClear();
    mockApi.getAppVersion.mockClear();
    mockApi.checkForUpdate.mockClear();
    mockApi.downloadUpdate.mockClear();
    mockApi.installUpdate.mockClear();
    mockApi.pickSlippiUserPath.mockClear();
  });

  it('renders the onboarding bootstrap state', async () => {
    render(<App />);

    expect(await screen.findByText('Detected Slippi Path')).toBeTruthy();
    expect((screen.getByLabelText('Slippi User Path') as HTMLInputElement).value).toBe(
      '/tmp/SlippiOnline'
    );
    expect(mockApi.startRuntime).not.toHaveBeenCalled();
  });

  it('advances to the controller instructions after saving setup', async () => {
    mockApi.checkSetup
      .mockResolvedValueOnce(incompleteSetup)
      .mockResolvedValueOnce(completeSetup);

    render(<App />);

    const input = await screen.findByLabelText('Slippi User Path');
    fireEvent.change(input, { target: { value: '/tmp/UpdatedSlippi' } });
    fireEvent.click(screen.getByRole('button', { name: 'Next' }));

    await waitFor(() => {
      expect(mockApi.saveConfig).toHaveBeenCalledWith({
        ...defaultConfig,
        slippi_user_path: '/tmp/UpdatedSlippi'
      });
    });
    expect(mockApi.installProfile).toHaveBeenCalledTimes(1);
    expect(await screen.findByText('Load Controller Profile')).toBeTruthy();
    expect(mockApi.startRuntime).not.toHaveBeenCalled();
  });

  it('auto-starts the runtime on the dashboard when setup is complete', async () => {
    mockApi.getConfig.mockResolvedValue({
      ...structuredClone(defaultConfig),
      onboarding_completed: true
    });
    mockApi.checkSetup.mockResolvedValue(completeSetup);

    render(<App />);

    await screen.findByRole('button', { name: 'Open settings' });
    await waitFor(() => {
      expect(mockApi.startRuntime).toHaveBeenCalledTimes(1);
    });
  });

  it('updates the visible runtime status from runtime events', async () => {
    mockApi.getConfig.mockResolvedValue({
      ...structuredClone(defaultConfig),
      onboarding_completed: true
    });
    mockApi.checkSetup.mockResolvedValue(completeSetup);
    mockApi.getRuntimeState.mockResolvedValue(runningRuntime);

    render(<App />);

    await screen.findByText('Running');

    await act(async () => {
      runtimeListener?.({
        status: 'waiting_for_slippi',
        startedAt: 123,
        lastError: null
      });
    });

    expect(screen.getByText('Waiting for Slippi')).toBeTruthy();
    expect(screen.getByText('Try restarting Slippi/Dolphin.')).toBeTruthy();
  });

  it('shows a dashboard update banner when an update is available', async () => {
    mockApi.getConfig.mockResolvedValue({
      ...structuredClone(defaultConfig),
      onboarding_completed: true
    });
    mockApi.checkSetup.mockResolvedValue(completeSetup);

    render(<App />);

    await screen.findByRole('button', { name: 'Open settings' });

    await act(async () => {
      updateStateListener?.({
        ...idleUpdateState,
        status: 'available',
        latestVersion: '0.2.0',
        notes: 'Bug fixes.',
        target: 'linux-x86_64'
      });
    });

    expect(screen.getByText('Update 0.2.0 is available.')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Download Update' })).toBeTruthy();
  });

  it('shows download progress in the settings updates section', async () => {
    mockApi.getConfig.mockResolvedValue({
      ...structuredClone(defaultConfig),
      onboarding_completed: true
    });
    mockApi.checkSetup.mockResolvedValue(completeSetup);

    render(<App />);

    fireEvent.click(await screen.findByRole('button', { name: 'Open settings' }));
    await screen.findByRole('heading', { name: 'Settings', level: 2 });

    await act(async () => {
      updateStateListener?.({
        ...idleUpdateState,
        status: 'downloading',
        latestVersion: '0.2.0',
        target: 'linux-x86_64',
        downloadedBytes: 512,
        contentLength: 1024
      });
    });

    expect(screen.getByText('Downloading')).toBeTruthy();
    expect(screen.getByText('50% (512 B / 1.0 KB)')).toBeTruthy();
  });

  it('shows the ready-to-install banner once an update is downloaded', async () => {
    mockApi.getConfig.mockResolvedValue({
      ...structuredClone(defaultConfig),
      onboarding_completed: true
    });
    mockApi.checkSetup.mockResolvedValue(completeSetup);

    render(<App />);

    await screen.findByRole('button', { name: 'Open settings' });

    await act(async () => {
      updateStateListener?.({
        ...idleUpdateState,
        status: 'downloaded',
        latestVersion: '0.2.0',
        target: 'linux-x86_64',
        downloadedBytes: 1024,
        contentLength: 1024
      });
    });

    expect(screen.getByText('Update 0.2.0 is ready to apply.')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Restart to Update' })).toBeTruthy();
  });

  it('shows updater errors in the settings updates section', async () => {
    mockApi.getConfig.mockResolvedValue({
      ...structuredClone(defaultConfig),
      onboarding_completed: true
    });
    mockApi.checkSetup.mockResolvedValue(completeSetup);

    render(<App />);

    fireEvent.click(await screen.findByRole('button', { name: 'Open settings' }));
    await screen.findByRole('heading', { name: 'Settings', level: 2 });

    await act(async () => {
      updateStateListener?.({
        ...idleUpdateState,
        status: 'error',
        latestVersion: '0.2.0',
        target: 'linux-x86_64',
        lastError: 'Move key-b0x.AppImage to a writable location such as ~/Applications/key-b0x.AppImage and try again.'
      });
    });

    expect(
      screen.getByText(
        'Move key-b0x.AppImage to a writable location such as ~/Applications/key-b0x.AppImage and try again.'
      )
    ).toBeTruthy();
    const settingsDialog = screen.getByRole('dialog', { name: 'Settings' });
    expect(within(settingsDialog).getByRole('button', { name: 'Download Update' })).toBeTruthy();
  });

  it('advances from the profile instructions to the keyboard test step', async () => {
    mockApi.checkSetup.mockResolvedValue(completeSetup);

    render(<App />);

    expect(await screen.findByText('Load Controller Profile')).toBeTruthy();

    fireEvent.click(screen.getByRole('button', { name: 'Next' }));

    expect(await screen.findByRole('heading', { name: 'Keyboard Test', level: 2 })).toBeTruthy();
    expect(mockApi.saveConfig).not.toHaveBeenCalled();
  });

  it('marks onboarding complete after the keyboard test step', async () => {
    mockApi.checkSetup.mockResolvedValue(completeSetup);

    render(<App />);

    expect(await screen.findByText('Load Controller Profile')).toBeTruthy();

    fireEvent.click(screen.getByRole('button', { name: 'Next' }));
    expect(await screen.findByRole('heading', { name: 'Keyboard Test', level: 2 })).toBeTruthy();

    fireEvent.click(screen.getByRole('button', { name: 'Next' }));

    await waitFor(() => {
      expect(mockApi.saveConfig).toHaveBeenCalledWith({
        ...defaultConfig,
        onboarding_completed: true
      });
    });
    await waitFor(() => {
      expect(mockApi.startRuntime).toHaveBeenCalledTimes(1);
    });
    expect(await screen.findByRole('button', { name: 'Open settings' })).toBeTruthy();
  });

  it('lets the user go back to the path step from the profile instructions', async () => {
    mockApi.checkSetup.mockResolvedValue(completeSetup);

    render(<App />);

    expect(await screen.findByText('Load Controller Profile')).toBeTruthy();

    fireEvent.click(screen.getByRole('button', { name: 'Back' }));

    expect(await screen.findByText('Detected Slippi Path')).toBeTruthy();
    expect((screen.getByLabelText('Slippi User Path') as HTMLInputElement).value).toBe(
      '/tmp/SlippiOnline'
    );
  });

  it('opens the keyboard test modal from onboarding and shows detected keys', async () => {
    mockApi.checkSetup.mockResolvedValue(completeSetup);

    render(<App />);

    expect(await screen.findByText('Load Controller Profile')).toBeTruthy();

    fireEvent.click(screen.getByRole('button', { name: 'Next' }));
    expect(await screen.findByRole('heading', { name: 'Keyboard Test', level: 2 })).toBeTruthy();

    fireEvent.click(screen.getByRole('button', { name: 'Open Keyboard Test' }));

    await waitFor(() => {
      expect(mockApi.startKeyboardTest).toHaveBeenCalledTimes(1);
    });
    expect(await screen.findByRole('dialog', { name: 'Keyboard Test' })).toBeTruthy();

    await act(async () => {
      keyboardTestListener?.({
        status: 'running',
        pressedKeys: ['KeyA', 'KeyS', 'Space'],
        lastError: null
      });
    });

    expect(screen.getByText('A')).toBeTruthy();
    expect(screen.getByText('S')).toBeTruthy();
    expect(screen.getByText('Space')).toBeTruthy();
  });
});
