import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mockInvoke, mockSearchManager, mockDialogManager, resetAllMocks } from '../test-utils';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

// Import after mocking
const { noteService } = await import('./noteService.svelte');

describe('noteService', () => {
  beforeEach(() => {
    resetAllMocks();
    noteService.clearError();
  });

  describe('create', () => {
    it('should create a note with .md extension', async () => {
      const noteName = 'test-note';
      const finalName = 'test-note.md';
      const notes = ['test-note.md', 'other-note.md'];

      mockInvoke.mockResolvedValueOnce(undefined);
      mockSearchManager.searchImmediate.mockResolvedValueOnce(notes);

      const onRefresh = vi.fn();
      const onFocus = vi.fn();

      await noteService.create(noteName, mockSearchManager, mockDialogManager, onRefresh, onFocus);

      expect(mockInvoke).toHaveBeenCalledWith('create_new_note', { noteName: finalName });
      expect(mockSearchManager.searchImmediate).toHaveBeenCalledWith('');
      expect(onRefresh).toHaveBeenCalledWith(notes);
      expect(mockDialogManager.closeCreateDialog).toHaveBeenCalled();
      expect(onFocus).toHaveBeenCalled();
      expect(noteService.isLoading).toBe(false);
      expect(noteService.error).toBeNull();
      expect(noteService.lastOperation).toBe('create');
    });

    it('should not add .md extension if already present', async () => {
      const noteName = 'test-note.md';

      mockInvoke.mockResolvedValueOnce(undefined);
      mockSearchManager.searchImmediate.mockResolvedValueOnce([]);

      await noteService.create(noteName, mockSearchManager, mockDialogManager, vi.fn(), vi.fn());

      expect(mockInvoke).toHaveBeenCalledWith('create_new_note', { noteName });
    });

    it('should handle creation errors', async () => {
      const error = new Error('Failed to create');
      mockInvoke.mockRejectedValueOnce(error);

      await noteService.create('test', mockSearchManager, mockDialogManager, vi.fn(), vi.fn());

      expect(noteService.error).toBe('Failed to create note: Error: Failed to create');
      expect(noteService.isLoading).toBe(false);
    });

    it('should not create note with empty name', async () => {
      await noteService.create('', mockSearchManager, mockDialogManager, vi.fn(), vi.fn());
      await noteService.create('   ', mockSearchManager, mockDialogManager, vi.fn(), vi.fn());

      expect(mockInvoke).not.toHaveBeenCalled();
    });
  });

  describe('delete', () => {
    it('should delete a note successfully', async () => {
      const noteName = 'test-note.md';
      const searchInput = 'test';
      const notes = ['other-note.md'];

      mockInvoke.mockResolvedValueOnce(undefined);
      mockSearchManager.searchImmediate.mockResolvedValueOnce(notes);

      const onRefresh = vi.fn();
      const onFocus = vi.fn();

      await noteService.delete(noteName, mockSearchManager, mockDialogManager, onRefresh, searchInput, onFocus);

      expect(mockInvoke).toHaveBeenCalledWith('delete_note', { noteName });
      expect(mockSearchManager.searchImmediate).toHaveBeenCalledWith(searchInput);
      expect(onRefresh).toHaveBeenCalledWith(notes);
      expect(mockDialogManager.closeDeleteDialog).toHaveBeenCalled();
      expect(onFocus).toHaveBeenCalled();
      expect(noteService.lastOperation).toBe('delete');
    });

    it('should handle deletion errors', async () => {
      const error = new Error('Failed to delete');
      mockInvoke.mockRejectedValueOnce(error);

      await noteService.delete('test.md', mockSearchManager, mockDialogManager, vi.fn(), '', vi.fn());

      expect(noteService.error).toBe('Failed to delete note: Error: Failed to delete');
    });

    it('should not delete with empty name', async () => {
      await noteService.delete('', mockSearchManager, mockDialogManager, vi.fn(), '', vi.fn());

      expect(mockInvoke).not.toHaveBeenCalled();
    });
  });

  describe('rename', () => {
    it('should rename a note successfully', async () => {
      const oldName = 'old-note.md';
      const newName = 'new-note';
      const finalNewName = 'new-note.md';
      const searchInput = '';
      const notes = ['new-note.md', 'other-note.md'];

      mockInvoke.mockResolvedValueOnce(undefined);
      mockSearchManager.searchImmediate.mockResolvedValueOnce(notes);

      const onRefresh = vi.fn();
      const onSelectNote = vi.fn();

      await noteService.rename(oldName, newName, mockSearchManager, mockDialogManager, onRefresh, onSelectNote, searchInput);

      expect(mockInvoke).toHaveBeenCalledWith('rename_note', { oldName, newName: finalNewName });
      expect(mockSearchManager.searchImmediate).toHaveBeenCalledWith(searchInput);
      expect(onRefresh).toHaveBeenCalledWith(notes);
      expect(onSelectNote).toHaveBeenCalledWith(finalNewName);
      expect(mockDialogManager.closeRenameDialog).toHaveBeenCalled();
      expect(noteService.lastOperation).toBe('rename');
    });

    it('should handle rename errors', async () => {
      const error = new Error('Failed to rename');
      mockInvoke.mockRejectedValueOnce(error);

      await noteService.rename('old.md', 'new', mockSearchManager, mockDialogManager, vi.fn(), vi.fn(), '');

      expect(noteService.error).toBe('Failed to rename note: Error: Failed to rename');
    });
  });

  describe('content operations', () => {
    it('should get note content', async () => {
      const content = 'Note content';
      mockInvoke.mockResolvedValueOnce(content);

      const result = await noteService.getContent('test.md');

      expect(mockInvoke).toHaveBeenCalledWith('get_note_content', { noteName: 'test.md' });
      expect(result).toBe(content);
    });

    it('should get raw note content', async () => {
      const content = 'Raw note content';
      mockInvoke.mockResolvedValueOnce(content);

      const result = await noteService.getRawContent('test.md');

      expect(mockInvoke).toHaveBeenCalledWith('get_note_raw_content', { noteName: 'test.md' });
      expect(result).toBe(content);
    });

    it('should save note content', async () => {
      const content = 'Updated content';
      mockInvoke.mockResolvedValueOnce(undefined);

      await noteService.save('test.md', content);

      expect(mockInvoke).toHaveBeenCalledWith('save_note', { noteName: 'test.md', content });
    });
  });

  describe('system integration', () => {
    it('should open note in editor', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await noteService.openInEditor('test.md');

      expect(mockInvoke).toHaveBeenCalledWith('open_note_in_editor', { noteName: 'test.md' });
    });

    it('should open note folder', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await noteService.openFolder('test.md');

      expect(mockInvoke).toHaveBeenCalledWith('open_note_folder', { noteName: 'test.md' });
    });
  });

  describe('state management', () => {
    it('should track loading state during operations', async () => {
      let loadingDuringOperation = false;
      mockInvoke.mockImplementation(() => {
        loadingDuringOperation = noteService.isLoading;
        return Promise.resolve();
      });

      await noteService.create('test', mockSearchManager, mockDialogManager, vi.fn(), vi.fn());

      expect(loadingDuringOperation).toBe(true);
      expect(noteService.isLoading).toBe(false);
    });

    it('should clear errors', () => {
      noteService.state.error = 'Some error';

      noteService.clearError();

      expect(noteService.error).toBeNull();
    });
  });
});
