import { describe, expect, it } from 'vitest';
import {
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
});
