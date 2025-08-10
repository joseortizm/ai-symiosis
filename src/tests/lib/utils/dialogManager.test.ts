import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mockSearchManager, resetAllMocks } from '../../test-utils';

describe('dialogManager (factory-based - TDD)', () => {
  let dialogManager: any;
  let mockFocusCallback: any;

  beforeEach(async () => {
    resetAllMocks();

    mockFocusCallback = vi.fn();

    try {
      const { createDialogManager } = await import('../../../lib/utils/dialogManager.svelte');
      dialogManager = createDialogManager({ focusSearch: mockFocusCallback });
    } catch (e) {
      dialogManager = null;
    }
  });

  it('should create dialogManager with focus callback', async () => {
    const { createDialogManager } = await import('../../../lib/utils/dialogManager.svelte');
    const manager = createDialogManager({ focusSearch: mockFocusCallback });

    expect(manager).toBeDefined();
    expect(typeof manager.openCreateDialog).toBe('function');
    expect(typeof manager.closeCreateDialog).toBe('function');
  });

  it('should call focus callback when closing create dialog', () => {
    if (!dialogManager) return;

    dialogManager.openCreateDialog();
    dialogManager.closeCreateDialog();

    expect(mockFocusCallback).toHaveBeenCalled();
  });

  it('should call focus callback when closing rename dialog', () => {
    if (!dialogManager) return;

    dialogManager.openRenameDialog();
    dialogManager.closeRenameDialog();

    expect(mockFocusCallback).toHaveBeenCalled();
  });

  it('should call focus callback when closing delete dialog', () => {
    if (!dialogManager) return;

    dialogManager.openDeleteDialog();
    dialogManager.closeDeleteDialog();

    expect(mockFocusCallback).toHaveBeenCalled();
  });

  it('should call focus callback when closing unsaved changes dialog', () => {
    if (!dialogManager) return;

    dialogManager.openUnsavedChangesDialog();
    dialogManager.closeUnsavedChangesDialog();

    expect(mockFocusCallback).toHaveBeenCalled();
  });
});

