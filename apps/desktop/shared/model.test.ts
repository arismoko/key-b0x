import { describe, expect, it } from 'vitest';
import {
  DEFAULT_MELEE_CONFIG,
  DEFAULT_BINDINGS,
  createDefaultConfig,
  findDuplicateBindings,
  formatKeyLabel,
  normalizeConfig
} from './model';

describe('model helpers', () => {
  it('formats key labels for common bindings', () => {
    expect(formatKeyLabel('BracketRight')).toBe(']');
    expect(formatKeyLabel('Digit3')).toBe('3');
    expect(formatKeyLabel('KeyV')).toBe('V');
    expect(formatKeyLabel('ArrowUp')).toBe('Up');
  });

  it('finds duplicate bindings', () => {
    const duplicates = findDuplicateBindings({
      ...DEFAULT_BINDINGS,
      a: 'KeyM',
      b: 'KeyM'
    });

    expect(duplicates).toHaveLength(1);
    expect(duplicates[0]).toEqual({
      key: 'KeyM',
      bindings: ['a', 'b']
    });
  });

  it('normalizes sparse configs onto v2 defaults', () => {
    const config = normalizeConfig(
      {
        version: 2,
        bindings: {
          analog_up: 'KeyW'
        }
      },
      '/tmp/SlippiOnline'
    );

    expect(config).toEqual({
      ...createDefaultConfig('/tmp/SlippiOnline'),
      bindings: {
        ...DEFAULT_BINDINGS,
        analog_up: 'KeyW'
      }
    });
  });

  it('creates default configs with the default melee subtree', () => {
    const config = createDefaultConfig('/tmp/SlippiOnline');

    expect(config.melee).toEqual(DEFAULT_MELEE_CONFIG);
  });

  it('normalizes melee settings from config files', () => {
    const config = normalizeConfig(
      {
        version: 2,
        melee: {
          socd: {
            main_x: 'dir1_priority',
            main_y: 'dir1_priority',
            c_x: 'dir1_priority',
            c_y: 'dir1_priority'
          },
          down_diagonal: 'crouch_walk_os',
          horizontal_socd_override: 'disabled',
          airdodge: {
            kind: 'custom_mod_x_diagonal',
            x: 0.625,
            y: 0.75
          }
        }
      },
      '/tmp/SlippiOnline'
    );

    expect(config.melee).toEqual({
      socd: {
        main_x: 'dir1_priority',
        main_y: 'dir1_priority',
        c_x: 'dir1_priority',
        c_y: 'dir1_priority'
      },
      down_diagonal: 'crouch_walk_os',
      horizontal_socd_override: 'disabled',
      airdodge: {
        kind: 'custom_mod_x_diagonal',
        x: 0.625,
        y: 0.75
      }
    });
  });
});
