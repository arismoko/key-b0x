import { EventEmitter } from 'node:events';
import { existsSync, unwatchFile, watchFile } from 'node:fs';
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { ChildProcessWithoutNullStreams, spawn } from 'node:child_process';
import path from 'node:path';
import { InstallProfileResult, RuntimeState } from '../shared/model';
import { configRootForPlatform } from './configStore';

type TransportLogState = 'connected' | 'newly_connected' | 'waiting_for_reader';

export class RuntimeService extends EventEmitter {
  private child: ChildProcessWithoutNullStreams | null = null;
  private state: RuntimeState = { status: 'idle', startedAt: null, lastError: null };
  private stopRequested = false;
  private readonly debugLogPath = path.join(
    configRootForPlatform(),
    'key-b0x',
    'runtime-debug.log'
  );

  getState(): RuntimeState {
    return { ...this.state };
  }

  hasLiveChild(): boolean {
    return this.child !== null;
  }

  async start(): Promise<RuntimeState> {
    if (this.child) {
      return this.getState();
    }

    const runtimePath = resolveRuntimeBinary();
    await mkdir(path.dirname(this.debugLogPath), { recursive: true });
    await rm(this.debugLogPath, { force: true });
    await writeFile(this.debugLogPath, '', 'utf8');

    this.stopRequested = false;
    this.setState({ status: 'starting', startedAt: Date.now(), lastError: null });

    const child = spawn(runtimePath, ['run'], {
      env: {
        ...process.env,
        KEY_B0X_DEBUG_LOG: this.debugLogPath
      },
      stdio: ['pipe', 'pipe', 'pipe']
    });

    this.child = child;
    this.watchDebugLog();

    child.stderr.on('data', (chunk: Buffer) => {
      const message = chunk.toString().trim();
      if (message.length > 0) {
        this.setState({
          ...this.state,
          lastError: message
        });
      }
    });

    child.on('error', (error) => {
      this.cleanupWatchers();
      this.child = null;
      this.setState({
        status: 'error',
        startedAt: null,
        lastError: error.message
      });
    });

    child.on('exit', (code, signal) => {
      this.cleanupWatchers();
      this.child = null;

      if (this.stopRequested) {
        this.stopRequested = false;
        this.setState({
          status: 'idle',
          startedAt: null,
          lastError: null
        });
        return;
      }

      const lastError =
        this.state.lastError ??
        `Runtime exited unexpectedly${code !== null ? ` with code ${code}` : ''}${
          signal ? ` (${signal})` : ''
        }`;

      this.setState({
        status: 'error',
        startedAt: null,
        lastError
      });
    });

    return this.getState();
  }

  async stop(): Promise<RuntimeState> {
    if (!this.child) {
      this.setState({
        status: 'idle',
        startedAt: null,
        lastError: null
      });
      return this.getState();
    }

    this.stopRequested = true;
    this.setState({
      ...this.state,
      status: 'stopping'
    });

    this.child.stdin.end();

    await new Promise<void>((resolve) => {
      if (!this.child) {
        resolve();
        return;
      }

      const timeout = setTimeout(() => {
        this.child?.kill();
      }, 1500);

      this.child.once('exit', () => {
        clearTimeout(timeout);
        resolve();
      });
    });

    return this.getState();
  }

  async installProfile(slippiUserPath: string): Promise<InstallProfileResult> {
    const { stdout } = await runRuntimeCommand([
      'install-profile',
      '--slippi-user-path',
      slippiUserPath
    ]);

    let profilePath = '';
    let pipesPath: string | null = null;

    for (const line of stdout.split(/\r?\n/)) {
      if (line.startsWith('Installed ')) {
        profilePath = line.slice('Installed '.length).trim();
      }
      if (line.startsWith('Created ')) {
        pipesPath = line.slice('Created '.length).trim();
      }
    }

    return {
      profilePath,
      pipesPath
    };
  }
  private async updateTransportState(): Promise<void> {
    const raw = await readFile(this.debugLogPath, 'utf8').catch(() => '');
    const transportState = parseLatestTransportState(raw);
    if (!transportState || this.state.status === 'stopping') {
      return;
    }

    const nextStatus =
      transportState === 'waiting_for_reader' ? 'waiting_for_slippi' : 'running';

    if (this.state.status !== nextStatus) {
      this.setState({
        ...this.state,
        status: nextStatus
      });
    }
  }

  private watchDebugLog(): void {
    watchFile(this.debugLogPath, { interval: 250 }, () => {
      void this.updateTransportState();
    });
    void this.updateTransportState();
  }

  private cleanupWatchers(): void {
    unwatchFile(this.debugLogPath);
  }

  private setState(nextState: RuntimeState): void {
    this.state = nextState;
    this.emit('state', this.getState());
  }
}

function resolveRuntimeBinary(): string {
  const envPath = process.env.KEY_B0X_RUNTIME_PATH;
  if (envPath && existsSync(envPath)) {
    return envPath;
  }

  const executableName = process.platform === 'win32' ? 'key-b0x-runtime.exe' : 'key-b0x-runtime';
  const repoRoot = path.resolve(__dirname, '../../..');
  const candidates = [
    path.join(repoRoot, 'target', 'debug', executableName),
    path.join(repoRoot, 'target', 'release', executableName),
    path.join(process.resourcesPath, 'bin', executableName)
  ];

  for (const candidate of candidates) {
    if (existsSync(candidate)) {
      return candidate;
    }
  }

  throw new Error(
    `Unable to locate ${executableName}. Set KEY_B0X_RUNTIME_PATH or build the runtime first.`
  );
}

async function runRuntimeCommand(args: string[]): Promise<{ stdout: string; stderr: string }> {
  const runtimePath = resolveRuntimeBinary();

  return new Promise((resolve, reject) => {
    const child = spawn(runtimePath, args, {
      stdio: ['ignore', 'pipe', 'pipe']
    });

    let stdout = '';
    let stderr = '';

    child.stdout.on('data', (chunk: Buffer) => {
      stdout += chunk.toString();
    });

    child.stderr.on('data', (chunk: Buffer) => {
      stderr += chunk.toString();
    });

    child.on('error', reject);
    child.on('exit', (code) => {
      if (code === 0) {
        resolve({ stdout, stderr });
        return;
      }

      reject(
        new Error(stderr.trim() || `Runtime command failed with exit code ${code ?? 'unknown'}`)
      );
    });
  });
}

export function parseLatestTransportState(rawLog: string): TransportLogState | null {
  const lines = rawLog.split(/\r?\n/).reverse();

  for (const line of lines) {
    if (line.startsWith('emit=')) {
      return line.slice('emit='.length).trim() as TransportLogState;
    }
    if (line.startsWith('startup_emit=')) {
      return line.slice('startup_emit='.length).trim() as TransportLogState;
    }
  }

  return null;
}
