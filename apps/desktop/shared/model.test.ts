import { describe, expect, it } from 'vitest';
import {
  DEFAULT_MELEE_CONFIG,
  DEFAULT_BINDINGS,
  cloneMeleeConfig,
  findDuplicateBindings,
  formatKeyLabel
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

  it('clones melee configs without sharing nested objects', () => {
    const copy = cloneMeleeConfig({
      ...DEFAULT_MELEE_CONFIG,
      airdodge: {
        kind: 'custom_mod_x_diagonal',
        x: 0.625,
        y: 0.75
      }
    });

    expect(copy).toEqual({
      socd: {
        main_x: 'second_input_priority_no_reactivation',
        main_y: 'second_input_priority_no_reactivation',
        c_x: 'second_input_priority_no_reactivation',
        c_y: 'second_input_priority_no_reactivation'
      },
      down_diagonal: 'auto_jab_cancel',
      horizontal_socd_override: 'max_jump_trajectory',
      airdodge: {
        kind: 'custom_mod_x_diagonal',
        x: 0.625,
        y: 0.75
      }
    });
    expect(copy.socd).not.toBe(DEFAULT_MELEE_CONFIG.socd);
  });
});
