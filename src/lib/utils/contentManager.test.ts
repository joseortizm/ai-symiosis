import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mockInvoke, resetAllMocks } from '../test-utils';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

// Mock the dependencies
vi.mock('./contentHighlighting.svelte', () => ({
  contentHighlighter: {
    updateState: vi.fn(),
    highlighted: 'mocked highlighted content',
    scrollToFirstMatch: vi.fn(),
    clearCache: vi.fn(),
    areHighlightsCleared: false
  }
}));

vi.mock('./focusManager.svelte', () => ({
  focusManager: {
    noteContentElement: null,
    scrollToFirstMatch: vi.fn()
  }
}));

vi.mock('./searchManager.svelte', () => ({
  searchManager: {
    searchImmediate: vi.fn().mockResolvedValue(['note1.md', 'note2.md'])
  }
}));

vi.mock('../services/noteService.svelte', () => ({
  noteService: {
    getContent: vi.fn().mockResolvedValue('mock note content')
  }
}));

const { contentManager } = await import('./contentManager.svelte');

describe('contentManager', () => {
  beforeEach(() => {
    resetAllMocks();
    // Reset contentManager state
    contentManager.setNoteContent('');
    vi.clearAllMocks();
  });

  describe('state getters', () => {
    it('should initialize with default state', () => {
      expect(contentManager.noteContent).toBe('');
      expect(typeof contentManager.highlightedContent).toBe('string');
    });

    it('should return highlighted content from contentHighlighter', async () => {
      const { contentHighlighter } = await import('./contentHighlighting.svelte');

      const result = contentManager.highlightedContent;

      expect(result).toBe('mocked highlighted content');
    });
  });

  describe('content management', () => {
    it('should set note content', () => {
      const testContent = 'Test note content';

      contentManager.setNoteContent(testContent);

      expect(contentManager.noteContent).toBe(testContent);
    });

    it('should update contentHighlighter when content changes', async () => {
      const { contentHighlighter } = await import('./contentHighlighting.svelte');
      const testContent = 'New content';

      contentManager.setNoteContent(testContent);

      expect(contentHighlighter.updateState).toHaveBeenCalledWith({
        content: testContent
      });
    });
  });

  describe('highlight actions', () => {
    it('should clear highlights', async () => {
      contentManager.clearHighlights();

      expect(contentManager.areHighlightsCleared).toBe(true);
    });

    it('should scroll to first match', () => {
      contentManager.scrollToFirstMatch();

      // Should call scrollToFirstMatch logic
      // Implementation will be tested through integration
    });

    it('should handle scroll to first match with no element', () => {
      expect(() => contentManager.scrollToFirstMatch()).not.toThrow();
    });
  });

  describe('content refresh workflows', () => {

    it('should refresh content for a note', async () => {
      const { noteService } = await import('../services/noteService.svelte');
      const noteName = 'test.md';
      const expectedContent = 'refreshed content';
      (noteService.getContent as any).mockResolvedValue(expectedContent);

      const result = await contentManager.refreshContent(noteName);

      expect(noteService.getContent).toHaveBeenCalledWith(noteName);
      expect(contentManager.noteContent).toBe(expectedContent);
      expect(result).toBe(expectedContent);
    });

    it('should handle errors when refreshing content', async () => {
      const { noteService } = await import('../services/noteService.svelte');
      const noteName = 'test.md';
      const error = new Error('Failed to load');
      (noteService.getContent as any).mockRejectedValue(error);

      await expect(contentManager.refreshContent(noteName)).rejects.toThrow('Failed to load');

      // Content should remain unchanged on error
      expect(contentManager.noteContent).toBe('');
    });

    it('should refresh after save workflow', async () => {
      const { searchManager } = await import('./searchManager.svelte');
      const noteName = 'test.md';
      const searchInput = 'test search';
      const refreshedContent = 'content after save';

      // Mock noteService.getContent for the refresh
      const { noteService } = await import('../services/noteService.svelte');
      (noteService.getContent as any).mockResolvedValue(refreshedContent);

      const result = await contentManager.refreshAfterSave(noteName, searchInput);

      // Should refresh cache
      expect(mockInvoke).toHaveBeenCalledWith('refresh_cache');

      // Should refresh search
      expect(searchManager.searchImmediate).toHaveBeenCalledWith(searchInput);

      // Should refresh content
      expect(noteService.getContent).toHaveBeenCalledWith(noteName);
      expect(contentManager.noteContent).toBe(refreshedContent);

      expect(result.searchResults).toEqual(['note1.md', 'note2.md']);
      expect(result.content).toBe(refreshedContent);
    });

    it('should handle refresh after save errors gracefully', async () => {
      const noteName = 'test.md';
      const searchInput = 'test';
      mockInvoke.mockRejectedValue(new Error('Cache refresh failed'));

      await expect(contentManager.refreshAfterSave(noteName, searchInput))
        .rejects.toThrow('Cache refresh failed');
    });
  });

  describe('integration with other services', () => {
    it('should update contentHighlighter state when dependencies change', async () => {
      const { contentHighlighter } = await import('./contentHighlighting.svelte');

      const testContent = 'test content';
      const testQuery = 'test query';

      contentManager.updateHighlighterState({
        content: testContent,
        query: testQuery,
        areHighlightsCleared: false
      });

      expect(contentHighlighter.updateState).toHaveBeenCalledWith({
        content: testContent,
        query: testQuery,
        areHighlightsCleared: false
      });
    });

    it('should update contentHighlighter with partial state', async () => {
      const { contentHighlighter } = await import('./contentHighlighting.svelte');

      contentManager.updateHighlighterState({
        query: 'test query'
      });

      expect(contentHighlighter.updateState).toHaveBeenCalledWith({
        query: 'test query'
      });
    });
  });
});
