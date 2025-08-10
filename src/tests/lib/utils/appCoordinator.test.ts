import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mockInvoke, resetAllMocks } from '../../test-utils';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

vi.mock('svelte', () => ({
  tick: vi.fn(() => Promise.resolve()),
}));

const { appCoordinator } = await import('../../../lib/utils/appCoordinator.svelte');
const { searchManager } = await import('../../../lib/utils/searchManager.svelte');

describe('appCoordinator', () => {
  beforeEach(() => {
    resetAllMocks();
    // Reset the appCoordinator state between tests
    appCoordinator.resetState();
  });

  describe('state management', () => {
    it('should provide reactive getters for central state', () => {
      expect(appCoordinator.query).toBe('');
      expect(appCoordinator.isLoading).toBe(false);
      expect(appCoordinator.areHighlightsCleared).toBe(false);
      expect(appCoordinator.filteredNotes).toEqual([]);
      expect(appCoordinator.selectedNote).toBe(null);
      expect(appCoordinator.selectedIndex).toBe(-1);
    });

    it('should update selectedIndex state', () => {
      appCoordinator.setSelectedIndex(3);
      expect(appCoordinator.selectedIndex).toBe(3);
    });

    it('should handle selectNote correctly', () => {
      appCoordinator.selectNote('note1.md', 2);
      expect(appCoordinator.selectedIndex).toBe(2);
    });

    it('should not update selectedIndex if it is the same', () => {
      appCoordinator.setSelectedIndex(5);
      appCoordinator.selectNote('note.md', 5);
      expect(appCoordinator.selectedIndex).toBe(5);
    });

    it('should auto-select first note when notes are loaded', () => {
      // Reset state to ensure clean start
      appCoordinator.resetState();
      expect(appCoordinator.selectedNote).toBe(null);
      expect(appCoordinator.selectedIndex).toBe(-1);

      // Simulate notes being loaded via searchManager
      searchManager.updateState({
        filteredNotes: ['note1.md', 'note2.md', 'note3.md']
      });

      // The derived selectedNote should return the first note
      expect(appCoordinator.selectedNote).toBe('note1.md');
      expect(typeof appCoordinator.selectedNote).toBe('string');

      // selectedIndex might not auto-update since effects aren't running in test
      // But the derived selectedNote should still work correctly
    });

    it('should handle selectedNote properly when no notes available', () => {
      appCoordinator.resetState();

      // Ensure no notes
      searchManager.updateState({ filteredNotes: [] });

      // selectedNote should be null (not a function)
      expect(appCoordinator.selectedNote).toBe(null);
      expect(typeof appCoordinator.selectedNote).not.toBe('function');
      expect(appCoordinator.selectedIndex).toBe(-1);
    });

    it('should reset selection when notes become empty', () => {
      // Start with notes
      searchManager.updateState({
        filteredNotes: ['note1.md', 'note2.md']
      });
      appCoordinator.setSelectedIndex(1);
      expect(appCoordinator.selectedNote).toBe('note2.md');

      // Clear notes
      searchManager.updateState({ filteredNotes: [] });

      // Should reset selection (selectedNote should be null with empty notes)
      expect(appCoordinator.selectedNote).toBe(null);
      // selectedIndex won't auto-reset without effects running, but that's ok for this test
    });
  });

  describe('keyboard handler integration', () => {
    it('should provide keyboardState aggregation', () => {
      const keyboardState = appCoordinator.keyboardState;

      expect(keyboardState).toHaveProperty('isSearchInputFocused');
      expect(keyboardState).toHaveProperty('isEditMode');
      expect(keyboardState).toHaveProperty('isNoteContentFocused');
      expect(keyboardState).toHaveProperty('selectedIndex');
      expect(keyboardState).toHaveProperty('filteredNotes');
      expect(keyboardState).toHaveProperty('selectedNote');
      expect(keyboardState).toHaveProperty('noteContentElement');
      expect(keyboardState).toHaveProperty('areHighlightsCleared');
      expect(keyboardState).toHaveProperty('isEditorDirty');
    });

    it('should provide keyboardActions', () => {
      const keyboardActions = appCoordinator.keyboardActions;

      expect(keyboardActions).toHaveProperty('setSelectedIndex');
      expect(keyboardActions).toHaveProperty('enterEditMode');
      expect(keyboardActions).toHaveProperty('exitEditMode');
      expect(keyboardActions).toHaveProperty('saveNote');
      expect(keyboardActions).toHaveProperty('showDeleteDialog');
      expect(keyboardActions).toHaveProperty('showCreateDialog');
      expect(keyboardActions).toHaveProperty('showRenameDialog');
      expect(keyboardActions).toHaveProperty('clearHighlights');
      expect(keyboardActions).toHaveProperty('clearSearch');

      expect(typeof keyboardActions.setSelectedIndex).toBe('function');
      expect(typeof keyboardActions.saveNote).toBe('function');
    });
  });

  describe('context provider', () => {
    it('should provide comprehensive context object', () => {
      const context = appCoordinator.context;

      expect(context).toHaveProperty('state');
      expect(context).toHaveProperty('editorManager');
      expect(context).toHaveProperty('focusManager');
      expect(context).toHaveProperty('contentManager');

      // Business logic functions
      expect(context).toHaveProperty('selectNote');
      expect(context).toHaveProperty('deleteNote');
      expect(context).toHaveProperty('createNote');
      expect(context).toHaveProperty('renameNote');
      expect(context).toHaveProperty('saveNote');
      expect(context).toHaveProperty('enterEditMode');
      expect(context).toHaveProperty('exitEditMode');

      // Dialog functions
      expect(context).toHaveProperty('openCreateDialog');
      expect(context).toHaveProperty('closeCreateDialog');
      expect(context).toHaveProperty('openRenameDialog');
      expect(context).toHaveProperty('closeRenameDialog');

      // Utility functions
      expect(context).toHaveProperty('clearHighlights');
      expect(context).toHaveProperty('clearSearch');
      expect(context).toHaveProperty('invoke');
    });

    it('should provide current state in context', () => {
      appCoordinator.setSelectedIndex(1);

      const context = appCoordinator.context;

      expect(context.state.selectedIndex).toBe(1);
    });
  });

  describe('placeholder business logic methods', () => {
    it('should have deleteNote method that is callable', async () => {
      expect(typeof appCoordinator.deleteNote).toBe('function');
      await expect(appCoordinator.deleteNote()).resolves.toBeUndefined();
    });

    it('should have createNote method that is callable', async () => {
      expect(typeof appCoordinator.createNote).toBe('function');
      await expect(appCoordinator.createNote()).resolves.toBeUndefined();
    });

    it('should have renameNote method that is callable', async () => {
      expect(typeof appCoordinator.renameNote).toBe('function');
      await expect(appCoordinator.renameNote()).resolves.toBeUndefined();
    });

    it('should have saveNote method that is callable', async () => {
      expect(typeof appCoordinator.saveNote).toBe('function');
      await expect(appCoordinator.saveNote()).resolves.toBeUndefined();
    });

    it('should have enterEditMode method that is callable', async () => {
      expect(typeof appCoordinator.enterEditMode).toBe('function');
      await expect(appCoordinator.enterEditMode()).resolves.toBeUndefined();
    });

    it('should have exitEditMode method that is callable', () => {
      expect(typeof appCoordinator.exitEditMode).toBe('function');
      expect(() => appCoordinator.exitEditMode()).not.toThrow();
    });
  });

  describe('initialization', () => {
    it('should provide initialize method that returns cleanup function', async () => {
      expect(typeof appCoordinator.initialize).toBe('function');
      const cleanup = await appCoordinator.initialize();
      expect(typeof cleanup).toBe('function');
    });

    it('should populate filteredNotes on initialization when config exists', async () => {
      const mockNotes = ['note1.md', 'note2.md', 'note3.md'];

      // Mock config exists
      mockInvoke.mockImplementation((command) => {
        if (command === 'config_exists') {
          return Promise.resolve(true);
        }
        if (command === 'search_notes') {
          return Promise.resolve(mockNotes);
        }
        return Promise.resolve();
      });

      // Before initialization, filteredNotes should be empty
      expect(appCoordinator.filteredNotes).toEqual([]);

      // Initialize the manager
      const cleanup = await appCoordinator.initialize();

      // After initialization, filteredNotes should be populated
      // This should come from searchManager.filteredNotes via reactive effects
      expect(appCoordinator.filteredNotes).toEqual(mockNotes);
      expect(mockInvoke).toHaveBeenCalledWith('config_exists');
      expect(mockInvoke).toHaveBeenCalledWith('search_notes', { query: '' });

      cleanup();
    });


    it('should provide reactive context that updates when state changes', async () => {
      // Get initial context
      let context = appCoordinator.context;
      expect(context.state.filteredNotes).toEqual([]);

      // Simulate state change (like what happens during initialization)
      appCoordinator.updateFilteredNotes(['test1.md', 'test2.md']);

      // Get context again - this should reflect the updated state
      context = appCoordinator.context;
      expect(context.state.filteredNotes).toEqual(['test1.md', 'test2.md']);
    });

    it('should not populate filteredNotes when config does not exist', async () => {
      // Mock config does not exist
      mockInvoke.mockImplementation((command) => {
        if (command === 'config_exists') {
          return Promise.resolve(false);
        }
        return Promise.resolve();
      });

      // Before initialization, filteredNotes should be empty
      expect(appCoordinator.filteredNotes).toEqual([]);

      // Initialize the manager
      const cleanup = await appCoordinator.initialize();

      // After initialization, filteredNotes should still be empty since no config exists
      expect(appCoordinator.filteredNotes).toEqual([]);
      expect(mockInvoke).toHaveBeenCalledWith('config_exists');
      expect(mockInvoke).not.toHaveBeenCalledWith('search_notes', { query: '' });

      cleanup();
    });
  });
});
