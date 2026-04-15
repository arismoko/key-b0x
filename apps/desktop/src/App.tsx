import { useEffect, useMemo, useRef, useState } from 'react';
import { api } from './api';
import {
  BINDING_GROUPS,
  DEFAULT_BINDINGS,
  DEFAULT_MELEE_CONFIG,
  type AppConfig,
  cloneMeleeConfig,
  type BindingId,
  type BindingMap,
  type DownDiagonalBehavior,
  type HorizontalSocdOverride,
  type MeleeConfig,
  type NormalizedKey,
  type RuntimeState,
  type SocdMode,
  type SetupStatus,
  findDuplicateBindings,
  formatBindingLabel,
  formatKeyLabel,
  isNormalizedKey
} from '../shared/model';

type AppView = 'onboarding' | 'dashboard';
type OnboardingStep = 'path' | 'profile';
type ToastTone = 'info' | 'success' | 'warning' | 'error';

type Toast = {
  id: number;
  message: string;
  tone: ToastTone;
};

type MeleeSettingsDraft = {
  socd_mode: SocdMode;
  down_diagonal: DownDiagonalBehavior;
  horizontal_socd_override: HorizontalSocdOverride;
  airdodge_kind: MeleeConfig['airdodge']['kind'];
  airdodge_x: string;
  airdodge_y: string;
};

type SelectOption<T extends string> = {
  value: T;
  label: string;
  detail?: string;
  tag?: string;
};

const IDLE_RUNTIME: RuntimeState = {
  status: 'idle',
  startedAt: null,
  lastError: null
};
const TOAST_LIFETIME_MS = 3200;
const SOCD_MODE_OPTIONS: SelectOption<SocdMode>[] = [
  {
    value: 'second_input_priority_no_reactivation',
    label: '2IP No Reactivation',
    detail: 'Latest input wins without reactivation',
    tag: 'Default'
  },
  {
    value: 'second_input_priority',
    label: '2IP',
    detail: 'Latest input wins'
  },
  {
    value: 'neutral',
    label: 'Neutral',
    detail: 'Opposites cancel out'
  },
  {
    value: 'dir1_priority',
    label: 'Dir1 Priority',
    detail: 'First direction wins'
  },
  {
    value: 'dir2_priority',
    label: 'Dir2 Priority',
    detail: 'Second direction wins'
  }
];
const DOWN_DIAGONAL_OPTIONS: SelectOption<DownDiagonalBehavior>[] = [
  {
    value: 'auto_jab_cancel',
    label: 'Auto Jab Cancel',
    detail: 'Standard down-diagonal behavior',
    tag: 'Default'
  },
  {
    value: 'crouch_walk_os',
    label: 'Crouch Walk OS',
    detail: 'Favor crouch walk option select'
  }
];
const HORIZONTAL_SOCD_OVERRIDE_OPTIONS: SelectOption<HorizontalSocdOverride>[] = [
  {
    value: 'max_jump_trajectory',
    label: 'Max Jump Trajectory',
    detail: 'Preserve the jump arc',
    tag: 'Default'
  },
  {
    value: 'disabled',
    label: 'Disabled',
    detail: 'Use raw horizontal SOCD'
  }
];
const AIRDODGE_KIND_OPTIONS: SelectOption<MeleeSettingsDraft['airdodge_kind']>[] = [
  {
    value: 'default',
    label: 'Default',
    detail: 'Standard runtime diagonal',
    tag: 'Default'
  },
  {
    value: 'custom_mod_x_diagonal',
    label: 'Custom Mod-X Shield Diagonal',
    detail: 'Tune the custom X/Y diagonal'
  }
];

function App() {
  const [appView, setAppView] = useState<AppView>('onboarding');
  const [onboardingStep, setOnboardingStep] = useState<OnboardingStep>('path');
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [bindingDraft, setBindingDraft] = useState<BindingMap | null>(null);
  const [meleeDraft, setMeleeDraft] = useState<MeleeSettingsDraft | null>(null);
  const [setup, setSetup] = useState<SetupStatus | null>(null);
  const [runtime, setRuntime] = useState<RuntimeState>(IDLE_RUNTIME);
  const [slippiPathDraft, setSlippiPathDraft] = useState('');
  const [captureTarget, setCaptureTarget] = useState<BindingId | null>(null);
  const [loading, setLoading] = useState(true);
  const [screenError, setScreenError] = useState<string | null>(null);
  const [toasts, setToasts] = useState<Toast[]>([]);
  const [busyAction, setBusyAction] = useState<string | null>(null);
  const nextToastId = useRef(0);
  const toastTimeouts = useRef(new Map<number, number>());

  useEffect(() => {
    let mounted = true;

    async function bootstrap() {
      try {
        const [loadedConfig, loadedRuntimeState] = await Promise.all([
          api.getConfig(),
          api.getRuntimeState()
        ]);

        if (!mounted) {
          return;
        }

        const setupStatus = await api.checkSetup();
        if (!mounted) {
          return;
        }

        const initialRoute = getInitialRoute(loadedConfig, setupStatus);
        const initialOnboardingStep = getInitialOnboardingStep(loadedConfig, setupStatus);

        setConfig(cloneConfig(loadedConfig));
        setBindingDraft({ ...loadedConfig.bindings });
        setMeleeDraft(meleeDraftFromConfig(loadedConfig.melee));
        setSlippiPathDraft(loadedConfig.slippi_user_path);
        setRuntime(loadedRuntimeState);
        setSetup(setupStatus);
        setAppView(initialRoute);
        setOnboardingStep(initialOnboardingStep);

        if (
          initialRoute === 'dashboard' &&
          (loadedRuntimeState.status === 'idle' || loadedRuntimeState.status === 'error') &&
          setupStatus.slippiFound &&
          setupStatus.profileInstalled &&
          findDuplicateBindings(loadedConfig.bindings).length === 0
        ) {
          setBusyAction('auto-start-runtime');

          try {
            await api.startRuntime();
          } catch (error) {
            if (mounted) {
              setScreenError(messageFromError(error));
            }
          } finally {
            if (mounted) {
              setBusyAction(null);
            }
          }
        }
      } catch (error) {
        if (mounted) {
          setScreenError(messageFromError(error));
        }
      } finally {
        if (mounted) {
          setLoading(false);
        }
      }
    }

    void bootstrap();

    const unsubscribe = api.onRuntimeState((nextRuntimeState) => {
      setRuntime(nextRuntimeState);
      if (nextRuntimeState.status === 'error') {
        setAppView('dashboard');
      }
    });

    return () => {
      mounted = false;
      unsubscribe();
    };
  }, []);

  useEffect(() => {
    return () => {
      for (const timeoutId of toastTimeouts.current.values()) {
        window.clearTimeout(timeoutId);
      }

      toastTimeouts.current.clear();
    };
  }, []);

  function dismissToast(id: number) {
    const timeoutId = toastTimeouts.current.get(id);

    if (timeoutId !== undefined) {
      window.clearTimeout(timeoutId);
      toastTimeouts.current.delete(id);
    }

    setToasts((currentToasts) => currentToasts.filter((toast) => toast.id !== id));
  }

  function pushToast(message: string, tone: ToastTone = 'info') {
    nextToastId.current += 1;
    const id = nextToastId.current;

    setToasts((currentToasts) => [...currentToasts, { id, message, tone }]);

    const timeoutId = window.setTimeout(() => {
      toastTimeouts.current.delete(id);
      setToasts((currentToasts) => currentToasts.filter((toast) => toast.id !== id));
    }, TOAST_LIFETIME_MS);

    toastTimeouts.current.set(id, timeoutId);
  }

  useEffect(() => {
    if (!captureTarget) {
      return;
    }

    const target = captureTarget;

    function handleKeyDown(event: KeyboardEvent) {
      event.preventDefault();
      event.stopPropagation();

      if (event.code === 'Escape') {
        setCaptureTarget(null);
        pushToast('Rebind cancelled.');
        return;
      }

      if (!isNormalizedKey(event.code)) {
        return;
      }

      setBindingDraft((currentDraft) => {
        if (!currentDraft) {
          return currentDraft;
        }

        return {
          ...currentDraft,
          [target]: event.code
        };
      });
      setCaptureTarget(null);
      pushToast(`Assigned ${formatBindingLabel(target)} to ${formatKeyLabel(event.code)}.`);
    }

    window.addEventListener('keydown', handleKeyDown, true);
    return () => {
      window.removeEventListener('keydown', handleKeyDown, true);
    };
  }, [captureTarget]);

  const runtimeLocked = isRuntimeLocked(runtime.status);
  const meleeLocked =
    busyAction === 'save-melee' ||
    busyAction === 'apply-setup' ||
    runtime.status === 'starting' ||
    runtime.status === 'stopping';
  const bindingsLocked =
    busyAction === 'save-bindings' ||
    busyAction === 'apply-setup' ||
    runtime.status === 'starting' ||
    runtime.status === 'stopping';
  const setupDirty = Boolean(config) && slippiPathDraft.trim() !== config?.slippi_user_path;
  const duplicateGroups = useMemo(
    () => (bindingDraft ? findDuplicateBindings(bindingDraft) : []),
    [bindingDraft]
  );
  const duplicateBindings = useMemo(
    () => new Set(duplicateGroups.flatMap((group) => group.bindings)),
    [duplicateGroups]
  );
  const bindingsDirty = Boolean(
    config && bindingDraft && !bindingMapsEqual(config.bindings, bindingDraft)
  );
  const meleeDirty = Boolean(config && meleeDraft && !meleeDraftMatchesConfig(meleeDraft, config.melee));
  const meleeValidationMessage = useMemo(
    () => (meleeDraft ? validateMeleeDraft(meleeDraft) : null),
    [meleeDraft]
  );
  const setupComplete = Boolean(setup?.slippiFound && setup?.profileInstalled);
  const bindingsReady = duplicateGroups.length === 0 && !bindingsDirty;
  const configurationReady = setupComplete && !setupDirty && bindingsReady;
  const dashboardNotice = getDashboardNotice({
    setupComplete,
    duplicateGroups
  });

  useEffect(() => {
    if (!settingsOpen || appView !== 'dashboard') {
      return;
    }

    function handleDialogKeyDown(event: KeyboardEvent) {
      if (event.code !== 'Escape') {
        return;
      }

      event.preventDefault();
      event.stopPropagation();
      void closeSettings();
    }

    window.addEventListener('keydown', handleDialogKeyDown, true);
    return () => {
      window.removeEventListener('keydown', handleDialogKeyDown, true);
    };
  }, [settingsOpen, appView, configurationReady, runtime.status]);

  async function refreshSetup() {
    const nextSetup = await api.checkSetup();
    setSetup(nextSetup);
    return nextSetup;
  }

  async function startRuntimeForDashboard() {
    setBusyAction('auto-start-runtime');
    setScreenError(null);

    try {
      await api.startRuntime();
      setAppView('dashboard');
    } catch (error) {
      setScreenError(messageFromError(error));
    } finally {
      setBusyAction(null);
    }
  }

  async function handleApplySetup(): Promise<boolean> {
    if (!config) {
      return false;
    }

    const nextPath = slippiPathDraft.trim();
    if (nextPath.length === 0) {
      setScreenError('Slippi user path cannot be empty.');
      return false;
    }

    const pathChanged = nextPath !== config.slippi_user_path;

    setBusyAction('apply-setup');
    setScreenError(null);

    try {
      const shouldRestartRuntime = isRuntimeLive(runtime.status);
      const savedConfig = await api.saveConfig({
        ...config,
        slippi_user_path: nextPath
      });

      setConfig((currentConfig) =>
        currentConfig
          ? {
              ...currentConfig,
              slippi_user_path: savedConfig.slippi_user_path
            }
          : cloneConfig(savedConfig)
      );
      setSlippiPathDraft(savedConfig.slippi_user_path);
      await api.installProfile();
      const nextSetup = await refreshSetup();

      if (shouldRestartRuntime) {
        await api.stopRuntime();
        await api.startRuntime();
      }

      pushToast(pathChanged ? 'Updated.' : 'Installed.', 'success');
      return nextSetup.slippiFound && nextSetup.profileInstalled;
    } catch (error) {
      setScreenError(messageFromError(error));
      return false;
    } finally {
      setBusyAction(null);
    }
  }

  async function handleSaveBindings() {
    if (!config || !bindingDraft) {
      return;
    }

    if (duplicateGroups.length > 0) {
      setScreenError(duplicateMessage(duplicateGroups[0]));
      return;
    }

    setBusyAction('save-bindings');
    setScreenError(null);

    try {
      const shouldRestartRuntime = !settingsOpen && isRuntimeLive(runtime.status);
      const shouldStartRuntime =
        !settingsOpen &&
        !shouldRestartRuntime &&
        setupComplete &&
        !setupDirty &&
        duplicateGroups.length === 0 &&
        (runtime.status === 'idle' || runtime.status === 'error');

      const savedConfig = await api.saveConfig({
        ...config,
        bindings: bindingDraft
      });

      setConfig(cloneConfig(savedConfig));
      setBindingDraft({ ...savedConfig.bindings });

      if (shouldRestartRuntime) {
        await api.stopRuntime();
        await api.startRuntime();
      } else if (shouldStartRuntime) {
        await api.startRuntime();
      }

      pushToast('Bindings saved.', 'success');
    } catch (error) {
      setScreenError(messageFromError(error));
    } finally {
      setBusyAction(null);
    }
  }

  function handleRestoreDefaults() {
    setBindingDraft({ ...DEFAULT_BINDINGS });
    pushToast('Restored default bindings. Save to keep them.');
  }

  async function handleSaveMelee() {
    if (!config || !meleeDraft) {
      return;
    }

    const validationMessage = validateMeleeDraft(meleeDraft);
    if (validationMessage) {
      return;
    }

    const meleeConfig = meleeConfigFromDraft(meleeDraft);
    setBusyAction('save-melee');
    setScreenError(null);

    try {
      const shouldRestartRuntime = isRuntimeLive(runtime.status);
      const savedConfig = await api.saveConfig({
        ...config,
        melee: meleeConfig
      });

      setConfig((currentConfig) =>
        currentConfig
          ? {
              ...currentConfig,
              melee: cloneMeleeConfig(savedConfig.melee)
            }
          : cloneConfig(savedConfig)
      );
      setMeleeDraft(meleeDraftFromConfig(savedConfig.melee));

      if (shouldRestartRuntime) {
        await api.stopRuntime();
        await api.startRuntime();
      }

      pushToast('Melee settings saved.', 'success');
    } catch (error) {
      setScreenError(messageFromError(error));
    } finally {
      setBusyAction(null);
    }
  }

  function handleRestoreDefaultMelee() {
    setMeleeDraft(meleeDraftFromConfig(DEFAULT_MELEE_CONFIG));
    pushToast('Restored default melee settings. Save to keep them.');
  }

  async function handleBrowseSlippiPath() {
    try {
      const pickedPath = await api.pickSlippiUserPath(slippiPathDraft);

      if (pickedPath) {
        setSlippiPathDraft(pickedPath);
      }
    } catch (error) {
      setScreenError(messageFromError(error));
    }
  }

  function openSettings() {
    setSettingsOpen(true);
    setCaptureTarget(null);
  }

  async function closeSettings() {
    setSettingsOpen(false);
    setCaptureTarget(null);

    if (configurationReady && (runtime.status === 'idle' || runtime.status === 'error')) {
      await startRuntimeForDashboard();
    }
  }

  async function toggleSettings() {
    if (settingsOpen) {
      await closeSettings();
      return;
    }

    openSettings();
  }

  async function enterDashboardFromOnboarding() {
    setCaptureTarget(null);
    setSettingsOpen(false);
    setOnboardingStep('path');
    setAppView('dashboard');

    if (bindingsReady && (runtime.status === 'idle' || runtime.status === 'error')) {
      await startRuntimeForDashboard();
    }
  }

  async function completeOnboarding() {
    if (!config) {
      return;
    }

    setBusyAction('complete-onboarding');
    setScreenError(null);

    try {
      const savedConfig = await api.saveConfig({
        ...config,
        onboarding_completed: true
      });

      setConfig(cloneConfig(savedConfig));
      await enterDashboardFromOnboarding();
    } catch (error) {
      setScreenError(messageFromError(error));
    } finally {
      setBusyAction(null);
    }
  }

  async function goToNextStep() {
    if (onboardingStep === 'path') {
      const setupReady = await handleApplySetup();

      if (!setupReady) {
        return;
      }

      if (config?.onboarding_completed) {
        await enterDashboardFromOnboarding();
        return;
      }

      setOnboardingStep('profile');
      return;
    }

    await completeOnboarding();
  }

  function goToPreviousStep() {
    setOnboardingStep('path');
  }

  if (loading || !config || !bindingDraft || !meleeDraft || !setup) {
    return (
      <div className="app-shell">
        <div className="app-backdrop" />
        <div className="loading-shell">
          <div className="loading-mark">
            <div className="app-badge">key-b0x</div>
            <h1>Loading</h1>
            <p>Opening the desktop app.</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="app-shell">
      <div className="app-backdrop" />
      <main className="app-frame">
        <header className={`topbar${appView === 'onboarding' ? ' topbar-compact' : ''}`}>
          <div className="topbar-copy">
            <div className="app-badge">key-b0x</div>
          </div>

          {appView === 'dashboard' ? (
            <div className="topbar-actions">
              <button
                type="button"
                className={`icon-button${settingsOpen ? ' icon-button-active' : ''}`}
                aria-label={settingsOpen ? 'Close settings' : 'Open settings'}
                onClick={() => {
                  void toggleSettings();
                }}
              >
                <span className="icon-button-sliders" aria-hidden="true">
                  <span />
                  <span />
                  <span />
                </span>
              </button>
            </div>
          ) : null}
        </header>

        {appView === 'onboarding' ? (
          <section className="wizard-shell">
            {screenError ? <div className="banner banner-error">{screenError}</div> : null}

            <div className="wizard-intro">
              <div className="wizard-intro-copy">
                <div className="section-eyebrow">
                  {onboardingStep === 'path' ? 'Step 1 of 2' : 'Step 2 of 2'}
                </div>
                <h2>{onboardingStep === 'path' ? 'Detected Slippi Path' : 'Load Controller Profile'}</h2>
              </div>
            </div>

            {onboardingStep === 'path' ? (
              <SetupSection
                busyAction={busyAction}
                slippiPathDraft={slippiPathDraft}
                setSlippiPathDraft={setSlippiPathDraft}
                label="Path"
                onBrowsePath={handleBrowseSlippiPath}
              />
            ) : (
              <div className="settings-card settings-card-setup">
                <div className="settings-card-copy">
                  <h3 className="settings-card-title">Dolphin / Ishiiruka Setup</h3>
                  <p className="settings-card-description">
                    Finish the controller setup in Dolphin or Ishiiruka before starting the runtime.
                  </p>
                </div>
                <ol className="wizard-step-list">
                  <li>Press the Controller button in Dolphin or Ishiiruka.</li>
                  <li>Set Port 1 to Standard Controller.</li>
                  <li>Select the key-b0x profile and press Load.</li>
                </ol>
                <p className="wizard-step-note">When that is done, press Next.</p>
              </div>
            )}

            <footer className="wizard-footer">
              <div className="wizard-actions">
                {onboardingStep === 'profile' ? (
                  <button
                    type="button"
                    className="button button-secondary"
                    disabled={busyAction === 'complete-onboarding'}
                    onClick={goToPreviousStep}
                  >
                    Back
                  </button>
                ) : null}
                <button
                  type="button"
                  className="button button-primary"
                  disabled={
                    runtimeLocked ||
                    busyAction === 'apply-setup' ||
                    busyAction === 'complete-onboarding' ||
                    (onboardingStep === 'path' && slippiPathDraft.trim().length === 0)
                  }
                  onClick={goToNextStep}
                >
                  Next
                </button>
              </div>
            </footer>
          </section>
        ) : (
          <section className="dashboard-shell">
            {screenError ? <div className="banner banner-error">{screenError}</div> : null}
            {dashboardNotice ? <div className={`banner banner-${dashboardNotice.tone}`}>{dashboardNotice.message}</div> : null}

            <BindingsSection
              status={runtime.status}
              runtimeLocked={bindingsLocked}
              busyAction={busyAction}
              bindingDraft={bindingDraft}
              duplicateGroups={duplicateGroups}
              duplicateBindings={duplicateBindings}
              captureTarget={captureTarget}
              bindingsDirty={bindingsDirty}
              onRestoreDefaults={handleRestoreDefaults}
              onSaveBindings={handleSaveBindings}
              onSelectBinding={(binding) => {
                setCaptureTarget(binding);
              }}
            />
          </section>
        )}
      </main>

      {appView === 'dashboard' && settingsOpen ? (
        <div
          className="settings-shell"
          role="presentation"
          onClick={() => {
            void closeSettings();
          }}
        >
          <section
            className="settings-modal"
            role="dialog"
            aria-modal="true"
            aria-labelledby="settings-title"
            onClick={(event) => event.stopPropagation()}
          >
            <div className="settings-modal-header">
              <div className="settings-modal-copy">
                <div className="section-eyebrow">Runtime Config</div>
                <h2 id="settings-title" className="settings-modal-title">
                  Settings
                </h2>
                <p className="settings-modal-blurb">
                  Tweak runtime behavior and setup without touching unsaved binding edits.
                </p>
              </div>
              <button
                type="button"
                className="settings-close"
                aria-label="Close settings"
                onClick={() => {
                  void closeSettings();
                }}
              >
                Close
              </button>
            </div>

            <div className="settings-section">
              <div className="section-eyebrow">Setup</div>
              <SetupSection
                busyAction={busyAction}
                slippiPathDraft={slippiPathDraft}
                setSlippiPathDraft={setSlippiPathDraft}
                label="Slippi Path"
                onBrowsePath={handleBrowseSlippiPath}
                onApplySetup={setupDirty || !setupComplete ? handleApplySetup : undefined}
              />
            </div>

            <div className="settings-section">
              <div className="section-eyebrow">Controls</div>
              <MeleeSettingsSection
                meleeDraft={meleeDraft}
                meleeDirty={meleeDirty}
                validationMessage={meleeValidationMessage}
                disabled={meleeLocked}
                onChange={setMeleeDraft}
                onRestoreDefaults={handleRestoreDefaultMelee}
                onSave={handleSaveMelee}
              />
            </div>
          </section>
        </div>
      ) : null}

      <ToastViewport toasts={toasts} onDismiss={dismissToast} />
    </div>
  );
}

function SetupSection({
  busyAction,
  slippiPathDraft,
  setSlippiPathDraft,
  label,
  onBrowsePath,
  onApplySetup
}: {
  busyAction: string | null;
  slippiPathDraft: string;
  setSlippiPathDraft: (value: string) => void;
  label: string;
  onBrowsePath: () => Promise<void>;
  onApplySetup?: () => Promise<boolean>;
}) {
  return (
    <div className="settings-card settings-card-setup">
      <div className="settings-card-copy">
        <h3 className="settings-card-title">{label}</h3>
        <p className="settings-card-description">
          Choose the Slippi user directory used for profile installation and pipe transport.
        </p>
      </div>

      <label htmlFor="slippi-path" className="settings-input-stack">
        <span className="settings-input-label">Slippi User Path</span>
        <div className="settings-path-row">
          <input
            id="slippi-path"
            className="text-input settings-text-input"
            value={slippiPathDraft}
            disabled={busyAction === 'apply-setup'}
            onChange={(event) => setSlippiPathDraft(event.target.value)}
          />
          <button
            type="button"
            className="button button-secondary settings-browse-button"
            disabled={busyAction === 'apply-setup'}
            onClick={() => {
              void onBrowsePath();
            }}
          >
            Browse
          </button>
        </div>
      </label>

      {onApplySetup ? (
        <div className="settings-actions">
          <div className="settings-inline-note">
            Saving updates the installed controller profile and restarts the runtime if needed.
          </div>
          <div className="toolbar-actions">
            <button
              type="button"
              className="button button-primary"
              disabled={busyAction === 'apply-setup'}
              onClick={() => {
                void onApplySetup();
              }}
            >
              Save Setup
            </button>
          </div>
        </div>
      ) : null}
    </div>
  );
}

function MeleeSettingsSection({
  meleeDraft,
  meleeDirty,
  validationMessage,
  disabled,
  onChange,
  onRestoreDefaults,
  onSave
}: {
  meleeDraft: MeleeSettingsDraft;
  meleeDirty: boolean;
  validationMessage: string | null;
  disabled: boolean;
  onChange: (draft: MeleeSettingsDraft) => void;
  onRestoreDefaults: () => void;
  onSave: () => Promise<void>;
}) {
  return (
    <div className="settings-stack">
      <SettingChoiceGroup
        label="SOCD Mode"
        description="Applies the same SOCD rule to main stick and C-stick axes for now."
        value={meleeDraft.socd_mode}
        options={SOCD_MODE_OPTIONS}
        disabled={disabled}
        columns="wide"
        onChange={(value) => onChange({ ...meleeDraft, socd_mode: value })}
      />

      <SettingChoiceGroup
        label="Down Diagonal"
        description="Choose between auto jab cancel behavior and crouch walk OS."
        value={meleeDraft.down_diagonal}
        options={DOWN_DIAGONAL_OPTIONS}
        disabled={disabled}
        columns="compact"
        onChange={(value) => onChange({ ...meleeDraft, down_diagonal: value })}
      />

      <SettingChoiceGroup
        label="Horizontal SOCD Override"
        description="Keep the jump-friendly horizontal override or disable it outright."
        value={meleeDraft.horizontal_socd_override}
        options={HORIZONTAL_SOCD_OVERRIDE_OPTIONS}
        disabled={disabled}
        columns="compact"
        onChange={(value) =>
          onChange({
            ...meleeDraft,
            horizontal_socd_override: value
          })
        }
      />

      <SettingChoiceGroup
        label="Airdodge"
        description="Use the default runtime diagonal or tune a custom Mod-X shield diagonal."
        value={meleeDraft.airdodge_kind}
        options={AIRDODGE_KIND_OPTIONS}
        disabled={disabled}
        columns="compact"
        onChange={(value) => onChange({ ...meleeDraft, airdodge_kind: value })}
      />

      {meleeDraft.airdodge_kind === 'custom_mod_x_diagonal' ? (
        <AirdodgeAxisCard
          x={meleeDraft.airdodge_x}
          y={meleeDraft.airdodge_y}
          disabled={disabled}
          onChangeX={(value) => onChange({ ...meleeDraft, airdodge_x: value })}
          onChangeY={(value) => onChange({ ...meleeDraft, airdodge_y: value })}
        />
      ) : null}

      {validationMessage ? <div className="banner banner-warning">{validationMessage}</div> : null}
      {meleeDirty ? <div className="settings-inline-note">Unsaved melee settings.</div> : null}

      <div className="settings-actions settings-actions-elevated">
        <div className="settings-inline-note">Saving restarts the runtime.</div>
        <div className="toolbar-actions">
          <button
            type="button"
            className="button button-secondary"
            disabled={disabled}
            onClick={onRestoreDefaults}
          >
            Restore Defaults
          </button>
          <button
            type="button"
            className="button button-primary"
            disabled={disabled || !meleeDirty || Boolean(validationMessage)}
            onClick={() => {
              void onSave();
            }}
          >
            Save Melee Settings
          </button>
        </div>
      </div>
    </div>
  );
}

function SettingChoiceGroup<T extends string>({
  label,
  description,
  value,
  options,
  disabled,
  columns,
  onChange
}: {
  label: string;
  description: string;
  value: T;
  options: SelectOption<T>[];
  disabled: boolean;
  columns: 'compact' | 'wide';
  onChange: (value: T) => void;
}) {
  return (
    <section className="settings-card">
      <div className="settings-card-copy">
        <h3 className="settings-card-title">{label}</h3>
        <p className="settings-card-description">{description}</p>
      </div>

      <div
        className={`settings-choice-grid settings-choice-grid-${columns}`}
        role="group"
        aria-label={label}
      >
        {options.map((option) => (
          <button
            key={option.value}
            type="button"
            className={`settings-choice${option.value === value ? ' settings-choice-active' : ''}`}
            aria-pressed={option.value === value}
            disabled={disabled}
            onClick={() => onChange(option.value)}
          >
            <span className="settings-choice-head">
              <span className="settings-choice-label">{option.label}</span>
              {option.tag ? <span className="settings-choice-tag">{option.tag}</span> : null}
            </span>
            {option.detail ? <span className="settings-choice-detail">{option.detail}</span> : null}
          </button>
        ))}
      </div>
    </section>
  );
}

function AirdodgeAxisCard({
  x,
  y,
  disabled,
  onChangeX,
  onChangeY
}: {
  x: string;
  y: string;
  disabled: boolean;
  onChangeX: (value: string) => void;
  onChangeY: (value: string) => void;
}) {
  return (
    <section className="settings-card settings-card-subtle">
      <div className="settings-card-copy">
        <h3 className="settings-card-title">Custom Airdodge Diagonal</h3>
        <p className="settings-card-description">
          Set precise X and Y values between 0 and 1. Step size: 0.0125.
        </p>
      </div>

      <div className="settings-axis-grid">
        <label className="settings-axis-field">
          <span className="settings-axis-label">Custom Airdodge X</span>
          <input
            type="number"
            inputMode="decimal"
            min="0"
            max="1"
            step="0.0125"
            className="text-input settings-text-input"
            value={x}
            disabled={disabled}
            onChange={(event) => onChangeX(event.target.value)}
          />
        </label>

        <label className="settings-axis-field">
          <span className="settings-axis-label">Custom Airdodge Y</span>
          <input
            type="number"
            inputMode="decimal"
            min="0"
            max="1"
            step="0.0125"
            className="text-input settings-text-input"
            value={y}
            disabled={disabled}
            onChange={(event) => onChangeY(event.target.value)}
          />
        </label>
      </div>
    </section>
  );
}

function BindingsSection({
  status,
  runtimeLocked,
  busyAction,
  bindingDraft,
  duplicateGroups,
  duplicateBindings,
  captureTarget,
  bindingsDirty,
  onRestoreDefaults,
  onSaveBindings,
  onSelectBinding
}: {
  status: RuntimeState['status'];
  runtimeLocked: boolean;
  busyAction: string | null;
  bindingDraft: BindingMap;
  duplicateGroups: Array<{ key: NormalizedKey; bindings: BindingId[] }>;
  duplicateBindings: Set<BindingId>;
  captureTarget: BindingId | null;
  bindingsDirty: boolean;
  onRestoreDefaults: () => void;
  onSaveBindings: () => Promise<void>;
  onSelectBinding: (binding: BindingId) => void;
}) {
  return (
    <div className="editor-shell">
      <div className="editor-toolbar">
        <div className="toolbar-status">
          <StatusFlag status={status} />
        </div>
        <div className="toolbar-actions">
          <button
            type="button"
            className="button button-secondary"
            disabled={runtimeLocked}
            onClick={onRestoreDefaults}
          >
            Restore Defaults
          </button>
          <button
            type="button"
            className="button button-primary"
            disabled={
              runtimeLocked ||
              busyAction === 'save-bindings' ||
              duplicateGroups.length > 0 ||
              !bindingsDirty
            }
            onClick={() => {
              void onSaveBindings();
            }}
          >
            Save Bindings
          </button>
        </div>
      </div>

      {duplicateGroups.length > 0 ? (
        <div className="banner banner-warning">{duplicateMessage(duplicateGroups[0])}</div>
      ) : captureTarget ? (
        <div className="banner banner-info">Press a key. Esc cancels.</div>
      ) : bindingsDirty ? (
        <div className="binding-note">Unsaved binding changes.</div>
      ) : null}

      <div className="binding-groups">
        {BINDING_GROUPS.map((group) => (
          <section key={group.id} className="binding-group">
            <div className="binding-group-header">
              <h3>{group.title}</h3>
              <p>{group.description}</p>
            </div>
            <div className="binding-table">
              {group.bindings.map((binding) => {
                const key = bindingDraft[binding];
                const isCapturing = captureTarget === binding;
                const hasConflict = duplicateBindings.has(binding);

                return (
                  <button
                    key={binding}
                    type="button"
                    className={`binding-row${isCapturing ? ' binding-row-capturing' : ''}${
                      hasConflict ? ' binding-row-conflict' : ''
                    }`}
                    disabled={runtimeLocked}
                    onClick={() => onSelectBinding(binding)}
                  >
                    <span className="binding-row-label">{formatBindingLabel(binding)}</span>
                    <span className="binding-row-key">
                      {isCapturing ? 'Press key' : formatKeyLabel(key)}
                    </span>
                    <span className="binding-row-state">
                      {hasConflict ? 'Conflict' : 'Edit'}
                    </span>
                  </button>
                );
              })}
            </div>
          </section>
        ))}
      </div>
    </div>
  );
}

function ToastViewport({
  toasts,
  onDismiss
}: {
  toasts: Toast[];
  onDismiss: (id: number) => void;
}) {
  if (toasts.length === 0) {
    return null;
  }

  return (
    <div className="toast-viewport" aria-live="polite" aria-atomic="true">
      {toasts.map((toast) => (
        <div key={toast.id} className={`toast toast-${toast.tone}`}>
          <div className="toast-message">{toast.message}</div>
          <button
            type="button"
            className="toast-dismiss"
            aria-label="Dismiss notification"
            onClick={() => onDismiss(toast.id)}
          >
            x
          </button>
        </div>
      ))}
    </div>
  );
}

function StatusFlag({
  status,
  large = false
}: {
  status: RuntimeState['status'];
  large?: boolean;
}) {
  return (
    <div className={`status-flag status-flag-${status}${large ? ' status-flag-large' : ''}`}>
      {statusLabel(status)}
    </div>
  );
}

function statusLabel(status: RuntimeState['status']): string {
  switch (status) {
    case 'idle':
      return 'Idle';
    case 'starting':
      return 'Starting';
    case 'running':
      return 'Running';
    case 'waiting_for_slippi':
      return 'Waiting for Slippi';
    case 'stopping':
      return 'Stopping';
    case 'error':
      return 'Error';
  }
}

function duplicateMessage(conflict: { key: NormalizedKey; bindings: BindingId[] }): string {
  const bindingNames = conflict.bindings.map((binding) => formatBindingLabel(binding)).join(' and ');
  return `${formatKeyLabel(conflict.key)} is assigned to ${bindingNames}.`;
}

function getInitialRoute(config: AppConfig, setup: SetupStatus): AppView {
  const setupComplete = setup.slippiFound && setup.profileInstalled;

  return setupComplete && config.onboarding_completed ? 'dashboard' : 'onboarding';
}

function getInitialOnboardingStep(config: AppConfig, setup: SetupStatus): OnboardingStep {
  const setupComplete = setup.slippiFound && setup.profileInstalled;

  if (!setupComplete) {
    return 'path';
  }

  return config.onboarding_completed ? 'path' : 'profile';
}

function getDashboardNotice({
  setupComplete,
  duplicateGroups
}: {
  setupComplete: boolean;
  duplicateGroups: Array<{ key: NormalizedKey; bindings: BindingId[] }>;
}): { tone: 'warning' | 'info'; message: string } | null {
  if (!setupComplete) {
    return {
      tone: 'warning',
      message: 'Setup is incomplete. Open Settings to finish it.'
    };
  }

  if (duplicateGroups.length > 0) {
    return {
      tone: 'warning',
      message: duplicateMessage(duplicateGroups[0])
    };
  }

  return null;
}

function isRuntimeLocked(status: RuntimeState['status']): boolean {
  return (
    status === 'starting' ||
    status === 'running' ||
    status === 'waiting_for_slippi' ||
    status === 'stopping'
  );
}

function isRuntimeLive(status: RuntimeState['status']): boolean {
  return status === 'starting' || status === 'running' || status === 'waiting_for_slippi';
}

function meleeDraftFromConfig(melee: MeleeConfig): MeleeSettingsDraft {
  return {
    socd_mode: melee.socd.main_x,
    down_diagonal: melee.down_diagonal,
    horizontal_socd_override: melee.horizontal_socd_override,
    airdodge_kind: melee.airdodge.kind,
    airdodge_x:
      melee.airdodge.kind === 'custom_mod_x_diagonal' ? formatDraftNumber(melee.airdodge.x) : '',
    airdodge_y:
      melee.airdodge.kind === 'custom_mod_x_diagonal' ? formatDraftNumber(melee.airdodge.y) : ''
  };
}

function meleeConfigFromDraft(draft: MeleeSettingsDraft): MeleeConfig {
  if (draft.airdodge_kind === 'custom_mod_x_diagonal') {
    return {
      socd: createUniformSocdConfig(draft.socd_mode),
      down_diagonal: draft.down_diagonal,
      horizontal_socd_override: draft.horizontal_socd_override,
      airdodge: {
        kind: 'custom_mod_x_diagonal',
        x: Number(draft.airdodge_x.trim()),
        y: Number(draft.airdodge_y.trim())
      }
    };
  }

  return {
    socd: createUniformSocdConfig(draft.socd_mode),
    down_diagonal: draft.down_diagonal,
    horizontal_socd_override: draft.horizontal_socd_override,
    airdodge: {
      kind: 'default'
    }
  };
}

function validateMeleeDraft(draft: MeleeSettingsDraft): string | null {
  if (draft.airdodge_kind !== 'custom_mod_x_diagonal') {
    return null;
  }

  const xText = draft.airdodge_x.trim();
  const yText = draft.airdodge_y.trim();

  if (xText.length === 0 || yText.length === 0) {
    return 'Custom airdodge X and Y are required.';
  }

  const x = Number(xText);
  const y = Number(yText);

  if (!Number.isFinite(x) || !Number.isFinite(y)) {
    return 'Custom airdodge X and Y must be numeric.';
  }

  if (x <= 0 || x > 1 || y <= 0 || y > 1) {
    return 'Custom airdodge X and Y must be within (0, 1].';
  }

  return null;
}

function meleeDraftMatchesConfig(draft: MeleeSettingsDraft, melee: MeleeConfig): boolean {
  if (
    draft.socd_mode !== melee.socd.main_x ||
    draft.socd_mode !== melee.socd.main_y ||
    draft.socd_mode !== melee.socd.c_x ||
    draft.socd_mode !== melee.socd.c_y ||
    draft.down_diagonal !== melee.down_diagonal ||
    draft.horizontal_socd_override !== melee.horizontal_socd_override ||
    draft.airdodge_kind !== melee.airdodge.kind
  ) {
    return false;
  }

  if (draft.airdodge_kind !== 'custom_mod_x_diagonal') {
    return true;
  }

  const validationMessage = validateMeleeDraft(draft);
  if (validationMessage) {
    return false;
  }

  return (
    melee.airdodge.kind === 'custom_mod_x_diagonal' &&
    Number(draft.airdodge_x.trim()) === melee.airdodge.x &&
    Number(draft.airdodge_y.trim()) === melee.airdodge.y
  );
}

function cloneConfig(config: AppConfig): AppConfig {
  return {
    ...config,
    bindings: { ...config.bindings },
    melee: cloneMeleeConfig(config.melee)
  };
}

function bindingMapsEqual(left: BindingMap, right: BindingMap): boolean {
  return Object.keys(left).every(
    (binding) => left[binding as BindingId] === right[binding as BindingId]
  );
}

function messageFromError(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  return 'Something went wrong.';
}

function createUniformSocdConfig(mode: SocdMode): MeleeConfig['socd'] {
  return {
    main_x: mode,
    main_y: mode,
    c_x: mode,
    c_y: mode
  };
}

function formatDraftNumber(value: number): string {
  return `${value}`;
}

export default App;
