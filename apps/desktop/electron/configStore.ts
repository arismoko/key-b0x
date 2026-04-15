import TOML from '@iarna/toml';
import { access, mkdir, readFile, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import {
  AppConfig,
  CONFIG_VERSION,
  PIPE_TARGET_LABEL,
  PROFILE_FILE_NAME,
  SetupStatus,
  createDefaultConfig,
  normalizeConfig
} from '../shared/model';

export function configRootForPlatform(
  platform: NodeJS.Platform = process.platform,
  env: NodeJS.ProcessEnv = process.env,
  homeDir: string = os.homedir()
): string {
  if (platform === 'win32') {
    return env.APPDATA ?? path.join(homeDir, 'AppData', 'Roaming');
  }

  return env.XDG_CONFIG_HOME ?? path.join(homeDir, '.config');
}

export function defaultConfigPath(): string {
  return path.join(configRootForPlatform(), 'key-b0x', 'config.toml');
}

export function defaultSlippiUserPath(): string {
  const configRoot = configRootForPlatform();
  if (process.platform === 'win32') {
    return path.join(configRoot, 'Slippi Launcher', 'netplay', 'User');
  }

  return path.join(configRoot, 'SlippiOnline');
}

export async function loadConfig(): Promise<AppConfig> {
  const configPath = defaultConfigPath();

  try {
    const raw = await readFile(configPath, 'utf8');
    const parsed = TOML.parse(raw);
    return normalizeConfig(parsed, defaultSlippiUserPath());
  } catch (error) {
    const err = error as NodeJS.ErrnoException;
    if (err.code !== 'ENOENT') {
      throw error;
    }
  }

  const config = createDefaultConfig(defaultSlippiUserPath());
  await saveConfig(config);
  return config;
}

export async function saveConfig(config: AppConfig): Promise<AppConfig> {
  const configPath = defaultConfigPath();
  const normalized = normalizeConfig(config, defaultSlippiUserPath());
  await mkdir(path.dirname(configPath), { recursive: true });

  const raw = TOML.stringify({
    version: CONFIG_VERSION,
    slippi_user_path: normalized.slippi_user_path,
    port: normalized.port,
    bindings: normalized.bindings
  });

  await writeFile(configPath, raw, 'utf8');
  return normalized;
}

export async function checkSetup(config?: AppConfig): Promise<SetupStatus> {
  const resolvedConfig = config ?? (await loadConfig());
  const profilePath = path.join(
    resolvedConfig.slippi_user_path,
    'Config',
    'Profiles',
    'GCPad',
    PROFILE_FILE_NAME
  );

  const slippiFound = await pathExists(resolvedConfig.slippi_user_path);
  const profileInstalled = await profileLooksInstalled(profilePath);

  return {
    slippiUserPath: resolvedConfig.slippi_user_path,
    slippiFound,
    profileInstalled,
    profilePath,
    pipeTargetLabel: PIPE_TARGET_LABEL,
    error: null
  };
}

async function pathExists(targetPath: string): Promise<boolean> {
  try {
    await access(targetPath);
    return true;
  } catch {
    return false;
  }
}

async function profileLooksInstalled(profilePath: string): Promise<boolean> {
  try {
    const raw = await readFile(profilePath, 'utf8');
    return raw.includes(`Device = ${PIPE_TARGET_LABEL}`);
  } catch {
    return false;
  }
}
