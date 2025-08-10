import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mockSearchManager, resetAllMocks } from '../test-utils';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const mockFocusManager = {
  focusSearch: vi.fn()
};

vi.mock('./focusManager.svelte', () => ({
  focusManager: mockFocusManager
}));

const { dialogManager } = await import('./dialogManager.svelte');

describe('dialogManager', () => {
  beforeEach(() => {
    resetAllMocks();
    mockFocusManager.focusSearch.mockClear();
    // Reset dialog states
    if (dialogManager.showCreateDialog) dialogManager.closeCreateDialog();
    if (dialogManager.showRenameDialog) dialogManager.closeRenameDialog();
    if (dialogManager.showDeleteDialog) dialogManager.closeDeleteDialog();
    if (dialogManager.showUnsavedChangesDialog) dialogManager.closeUnsavedChangesDialog();
  });

  describe('create dialog', () => {
    it('should open create dialog with empty name by default', () => {
      dialogManager.updateState({ query: '', highlightedContent: '' });

      dialogManager.openCreateDialog();

      expect(dialogManager.showCreateDialog).toBe(true);
      expect(dialogManager.newNoteName).toBe('');
    });

    it('should pre-fill create dialog with query when no content and query exists', () => {
      dialogManager.updateState({
        query: 'test note',
        highlightedContent: ''
      });

      dialogManager.openCreateDialog();

      expect(dialogManager.showCreateDialog).toBe(true);
      expect(dialogManager.newNoteName).toBe('test note');
    });

    it('should not pre-fill when content exists', () => {
      dialogManager.updateState({
        query: 'test note',
        highlightedContent: 'some content'
      });

      dialogManager.openCreateDialog();

      expect(dialogManager.showCreateDialog).toBe(true);
      expect(dialogManager.newNoteName).toBe('');
    });

    it('should close create dialog and clear name', () => {
      dialogManager.openCreateDialog();
      dialogManager.setNewNoteName('test');

      dialogManager.closeCreateDialog();

      expect(dialogManager.showCreateDialog).toBe(false);
      expect(dialogManager.newNoteName).toBe('');
    });
  });

  describe('rename dialog', () => {
    it('should open rename dialog with note name without extension', () => {
      dialogManager.updateState({ selectedNote: 'test-note.md' });

      dialogManager.openRenameDialog();

      expect(dialogManager.showRenameDialog).toBe(true);
      expect(dialogManager.newNoteNameForRename).toBe('test-note');
    });

    it('should open rename dialog with full name if no .md extension', () => {
      dialogManager.updateState({ selectedNote: 'test-note.txt' });

      dialogManager.openRenameDialog();

      expect(dialogManager.showRenameDialog).toBe(true);
      expect(dialogManager.newNoteNameForRename).toBe('test-note.txt');
    });

    it('should not open rename dialog if no selected note', () => {
      dialogManager.updateState({ selectedNote: null });

      dialogManager.openRenameDialog();

      expect(dialogManager.showRenameDialog).toBe(false);
    });

    it('should close rename dialog and clear name', () => {
      dialogManager.updateState({ selectedNote: 'test.md' });
      dialogManager.openRenameDialog();

      dialogManager.closeRenameDialog();

      expect(dialogManager.showRenameDialog).toBe(false);
      expect(dialogManager.newNoteNameForRename).toBe('');
    });
  });

  describe('delete dialog', () => {
    it('should open delete dialog and reset count', () => {
      dialogManager.openDeleteDialog();

      expect(dialogManager.showDeleteDialog).toBe(true);
      expect(dialogManager.deleteKeyPressCount).toBe(0);
    });

    it('should handle delete key press count', () => {
      dialogManager.openDeleteDialog();
      const onConfirmDelete = vi.fn();

      dialogManager.handleDeleteKeyPress(onConfirmDelete);

      expect(dialogManager.deleteKeyPressCount).toBe(1);
      expect(onConfirmDelete).not.toHaveBeenCalled();
    });

    it('should confirm delete on second key press', () => {
      dialogManager.openDeleteDialog();
      const onConfirmDelete = vi.fn();

      dialogManager.handleDeleteKeyPress(onConfirmDelete);
      dialogManager.handleDeleteKeyPress(onConfirmDelete);

      expect(dialogManager.deleteKeyPressCount).toBe(2);
      expect(onConfirmDelete).toHaveBeenCalledOnce();
    });

    it('should reset count after timeout', async () => {
      dialogManager.openDeleteDialog();
      const onConfirmDelete = vi.fn();

      dialogManager.handleDeleteKeyPress(onConfirmDelete);
      expect(dialogManager.deleteKeyPressCount).toBe(1);

      // Wait for timeout + a bit more
      await new Promise(resolve => setTimeout(resolve, 2100));

      expect(dialogManager.deleteKeyPressCount).toBe(0);
    });

    it('should close delete dialog and reset state', () => {
      dialogManager.openDeleteDialog();
      dialogManager.handleDeleteKeyPress(vi.fn());

      dialogManager.closeDeleteDialog();

      expect(dialogManager.showDeleteDialog).toBe(false);
      expect(dialogManager.deleteKeyPressCount).toBe(0);
    });
  });

  describe('unsaved changes dialog', () => {
    it('should open unsaved changes dialog', () => {
      dialogManager.openUnsavedChangesDialog();

      expect(dialogManager.showUnsavedChangesDialog).toBe(true);
    });

    it('should close unsaved changes dialog', () => {
      dialogManager.openUnsavedChangesDialog();

      dialogManager.closeUnsavedChangesDialog();

      expect(dialogManager.showUnsavedChangesDialog).toBe(false);
    });
  });

  describe('state management', () => {
    it('should update context state', () => {
      const newState = {
        selectedNote: 'test.md',
        query: 'test query',
        highlightedContent: 'highlighted content'
      };

      dialogManager.updateState(newState);

      expect(dialogManager.newNoteNameForRename).toBe('');
      dialogManager.openRenameDialog();
      expect(dialogManager.newNoteNameForRename).toBe('test');
    });

    it('should set note names', () => {
      dialogManager.setNewNoteName('create test');
      dialogManager.setNewNoteNameForRename('rename test');

      expect(dialogManager.newNoteName).toBe('create test');
      expect(dialogManager.newNoteNameForRename).toBe('rename test');
    });
  });

  describe('unsaved changes workflow (functions that should exist in dialogManager)', () => {
    it('should have handleSaveAndExit method', async () => {
      const mockSaveAndExitNote = vi.fn().mockResolvedValue(undefined);

      dialogManager.openUnsavedChangesDialog();

      await dialogManager.handleSaveAndExit(mockSaveAndExitNote);

      expect(dialogManager.showUnsavedChangesDialog).toBe(false);
      expect(mockSaveAndExitNote).toHaveBeenCalled();
    });

    it('should have handleDiscardAndExit method', () => {
      const mockExitEditMode = vi.fn();

      dialogManager.openUnsavedChangesDialog();

      dialogManager.handleDiscardAndExit(mockExitEditMode);

      expect(dialogManager.showUnsavedChangesDialog).toBe(false);
      expect(mockExitEditMode).toHaveBeenCalled();
    });

    it('should have showExitEditDialog method', () => {
      dialogManager.showExitEditDialog();

      expect(dialogManager.showUnsavedChangesDialog).toBe(true);
    });
  });
});
