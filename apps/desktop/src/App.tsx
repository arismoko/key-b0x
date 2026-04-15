import { useEffect, useMemo, useState } from 'react';
import {
  BINDING_GROUPS,
  DEFAULT_BINDINGS,
  type AppConfig,
  type BindingId,
  type BindingMap,
  type NormalizedKey,
  type RuntimeState,
  type SetupStatus,
  findDuplicateBindings,
  formatBindingLabel,
  formatKeyLabel,
  isNormalizedKey
} from '../shared/model';

type AppView = 'onboarding' | 'dashboard';

const IDLE_RUNTIME: RuntimeState = {
  status: 'idle',
  startedAt: null,
  lastError: null
};

function App() {
  const [appView, setAppView] = useState<AppView>('onboarding');
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [bindingDraft, setBindingDraft] = useState<BindingMap | null>(null);
  const [setup, setSetup] = useState<SetupStatus | null>(null);
  const [runtime, setRuntime] = useState<RuntimeState>(IDLE_RUNTIME);
  const [slippiPathDraft, setSlippiPathDraft] = useState('');
  const [captureTarget, setCaptureTarget] = useState<BindingId | null>(null);
  const [loading, setLoading] = useState(true);
  const [screenError, setScreenError] = useState<string | null>(null);
  const [setupFeedback, setSetupFeedback] = useState<string | null>(null);
  const [bindingFeedback, setBindingFeedback] = useState<string | null>(null);
  const [busyAction, setBusyAction] = useState<string | null>(null);

  useEffect(() => {
    let mounted = true;

    async function bootstrap() {
      try {
        const [loadedConfig, loadedRuntimeState] = await Promise.all([
          window.keyB0x.getConfig(),
          window.keyB0x.getRuntimeState()
        ]);

        if (!mounted) {
          return;
        }

        const setupStatus = await window.keyB0x.checkSetup();
        if (!mounted) {
          return;
        }

        const initialRoute = getInitialRoute(setupStatus);

        setConfig(cloneConfig(loadedConfig));
        setBindingDraft({ ...loadedConfig.bindings });
        setSlippiPathDraft(loadedConfig.slippi_user_path);
        setRuntime(loadedRuntimeState);
        setSetup(setupStatus);
        setAppView(initialRoute);

        if (
          initialRoute === 'dashboard' &&
          (loadedRuntimeState.status === 'idle' || loadedRuntimeState.status === 'error') &&
          setupStatus.slippiFound &&
          setupStatus.profileInstalled &&
          findDuplicateBindings(loadedConfig.bindings).length === 0
        ) {
          setBusyAction('auto-start-runtime');

          try {
            await window.keyB0x.startRuntime();
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

    const unsubscribe = window.keyB0x.onRuntimeState((nextRuntimeState) => {
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
    if (!captureTarget) {
      return;
    }

    const target = captureTarget;

    function handleKeyDown(event: KeyboardEvent) {
      event.preventDefault();
      event.stopPropagation();

      if (event.code === 'Escape') {
        setCaptureTarget(null);
        setBindingFeedback('Rebind cancelled.');
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
      setBindingFeedback(`Assigned ${formatBindingLabel(target)} to ${formatKeyLabel(event.code)}.`);
    }

    window.addEventListener('keydown', handleKeyDown, true);
    return () => {
      window.removeEventListener('keydown', handleKeyDown, true);
    };
  }, [captureTarget]);

  const runtimeLocked = isRuntimeLocked(runtime.status);
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
    const nextSetup = await window.keyB0x.checkSetup();
    setSetup(nextSetup);
    return nextSetup;
  }

  async function startRuntimeForDashboard() {
    setBusyAction('auto-start-runtime');
    setScreenError(null);

    try {
      await window.keyB0x.startRuntime();
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
    setSetupFeedback(null);

    try {
      const savedConfig = await window.keyB0x.saveConfig({
        ...config,
        slippi_user_path: nextPath
      });

      setConfig(cloneConfig(savedConfig));
      setBindingDraft({ ...savedConfig.bindings });
      setSlippiPathDraft(savedConfig.slippi_user_path);
      await window.keyB0x.installProfile();
      const nextSetup = await refreshSetup();
      setSetupFeedback(pathChanged ? 'Updated.' : 'Installed.');
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
    setBindingFeedback(null);

    try {
      const shouldRestartRuntime = !settingsOpen && isRuntimeLive(runtime.status);
      const shouldStartRuntime =
        !settingsOpen &&
        !shouldRestartRuntime &&
        setupComplete &&
        !setupDirty &&
        duplicateGroups.length === 0 &&
        (runtime.status === 'idle' || runtime.status === 'error');

      const savedConfig = await window.keyB0x.saveConfig({
        ...config,
        bindings: bindingDraft
      });

      setConfig(cloneConfig(savedConfig));
      setBindingDraft({ ...savedConfig.bindings });

      if (shouldRestartRuntime) {
        await window.keyB0x.stopRuntime();
        await window.keyB0x.startRuntime();
      } else if (shouldStartRuntime) {
        await window.keyB0x.startRuntime();
      }

      setBindingFeedback('Bindings saved.');
    } catch (error) {
      setScreenError(messageFromError(error));
    } finally {
      setBusyAction(null);
    }
  }

  function handleRestoreDefaults() {
    setBindingDraft({ ...DEFAULT_BINDINGS });
    setBindingFeedback('Restored default bindings. Save to keep them.');
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

  async function goToNextStep() {
    const setupReady = await handleApplySetup();

    if (!setupReady) {
      return;
    }

    setCaptureTarget(null);
    setSettingsOpen(false);
    setAppView('dashboard');

    if (setupReady && bindingsReady && (runtime.status === 'idle' || runtime.status === 'error')) {
      void startRuntimeForDashboard();
    }
  }

  if (loading || !config || !bindingDraft || !setup) {
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
              <h2>Detected Slippi Path</h2>
            </div>

            <SetupSection
              runtimeLocked={runtimeLocked}
              busyAction={busyAction}
              slippiPathDraft={slippiPathDraft}
              setSlippiPathDraft={setSlippiPathDraft}
              setupFeedback={null}
              label="Path"
            />

            <footer className="wizard-footer">
              <div className="wizard-actions">
                <button
                  type="button"
                  className="button button-primary"
                  disabled={
                    runtimeLocked ||
                    busyAction === 'apply-setup' ||
                    slippiPathDraft.trim().length === 0
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
              bindingFeedback={bindingFeedback}
              bindingsDirty={bindingsDirty}
              onRestoreDefaults={handleRestoreDefaults}
              onSaveBindings={handleSaveBindings}
              onSelectBinding={(binding) => {
                setCaptureTarget(binding);
                setBindingFeedback(null);
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
              <h2 id="settings-title" className="settings-modal-title">
                Settings
              </h2>
              <button
                type="button"
                className="settings-close"
                aria-label="Close settings"
                onClick={() => {
                  void closeSettings();
                }}
              >
                x
              </button>
            </div>

            <SetupSection
              runtimeLocked={runtimeLocked}
              busyAction={busyAction}
              slippiPathDraft={slippiPathDraft}
              setSlippiPathDraft={setSlippiPathDraft}
              setupFeedback={setupFeedback}
              label="Slippi Path"
              onApplySetup={setupDirty || !setupComplete ? handleApplySetup : undefined}
            />
          </section>
        </div>
      ) : null}
    </div>
  );
}

function SetupSection({
  runtimeLocked,
  busyAction,
  slippiPathDraft,
  setSlippiPathDraft,
  setupFeedback,
  label,
  onApplySetup
}: {
  runtimeLocked: boolean;
  busyAction: string | null;
  slippiPathDraft: string;
  setSlippiPathDraft: (value: string) => void;
  setupFeedback: string | null;
  label: string;
  onApplySetup?: () => Promise<boolean>;
}) {
  return (
    <div className="editor-shell">
      <div className="field-row">
        <label htmlFor="slippi-path" className="field-label">
          {label}
        </label>
        <input
          id="slippi-path"
          className="text-input"
          value={slippiPathDraft}
          disabled={runtimeLocked || busyAction === 'apply-setup'}
          onChange={(event) => setSlippiPathDraft(event.target.value)}
        />
      </div>

      {onApplySetup ? (
        <div className="editor-toolbar">
          <div className="toolbar-actions">
            <button
              type="button"
              className="button button-primary"
              disabled={runtimeLocked || busyAction === 'apply-setup'}
              onClick={() => {
                void onApplySetup();
              }}
            >
              Save
            </button>
          </div>
        </div>
      ) : null}

      {setupFeedback ? <div className="banner banner-info">{setupFeedback}</div> : null}
    </div>
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
  bindingFeedback,
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
  bindingFeedback: string | null;
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
      ) : bindingFeedback ? (
        <div className="banner banner-info">{bindingFeedback}</div>
      ) : (
        <div className="binding-note">{bindingsDirty ? 'Unsaved binding changes.' : '24 bindings ready.'}</div>
      )}

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

function getInitialRoute(
  setup: SetupStatus
): AppView {
  const setupComplete = setup.slippiFound && setup.profileInstalled;

  return setupComplete ? 'dashboard' : 'onboarding';
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

function cloneConfig(config: AppConfig): AppConfig {
  return {
    ...config,
    bindings: { ...config.bindings }
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

export default App;
