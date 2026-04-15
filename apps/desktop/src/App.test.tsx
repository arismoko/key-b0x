// @vitest-environment jsdom

import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { AppConfig, RuntimeState, SetupStatus } from '../shared/model';
import App from './App';

const defaultConfig: AppConfig = {
  version: 2,
  slippi_user_path: '/tmp/SlippiOnline',
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

let runtimeListener: ((state: RuntimeState) => void) | null = null;

const mockApi = vi.hoisted(() => ({
  getConfig: vi.fn(),
  saveConfig: vi.fn(),
  checkSetup: vi.fn(),
  installProfile: vi.fn(),
  getRuntimeState: vi.fn(),
  startRuntime: vi.fn(),
  stopRuntime: vi.fn(),
  pickSlippiUserPath: vi.fn(),
  onRuntimeState: vi.fn((listener: (state: RuntimeState) => void) => {
    runtimeListener = listener;
    return () => {
      runtimeListener = null;
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
    mockApi.pickSlippiUserPath.mockResolvedValue(null);
    mockApi.onRuntimeState.mockClear();
    mockApi.getConfig.mockClear();
    mockApi.saveConfig.mockClear();
    mockApi.checkSetup.mockClear();
    mockApi.installProfile.mockClear();
    mockApi.getRuntimeState.mockClear();
    mockApi.startRuntime.mockClear();
    mockApi.stopRuntime.mockClear();
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

  it('saves setup through the same onboarding flow', async () => {
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
    await waitFor(() => {
      expect(mockApi.startRuntime).toHaveBeenCalledTimes(1);
    });
  });

  it('auto-starts the runtime on the dashboard when setup is complete', async () => {
    mockApi.checkSetup.mockResolvedValue(completeSetup);

    render(<App />);

    await screen.findByRole('button', { name: 'Open settings' });
    await waitFor(() => {
      expect(mockApi.startRuntime).toHaveBeenCalledTimes(1);
    });
  });

  it('updates the visible runtime status from runtime events', async () => {
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
  });
});
