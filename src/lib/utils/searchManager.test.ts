import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mockInvoke, resetAllMocks } from '../test-utils';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

const { searchManager } = await import('./searchManager.svelte');

describe('searchManager', () => {
  beforeEach(() => {
    resetAllMocks();
    searchManager.abort();
  });

  describe('existing functionality', () => {
    it('should update search state with debouncing', async () => {
      const notes = ['note1.md', 'note2.md'];
      mockInvoke.mockResolvedValueOnce(notes);
      const onQueryCommit = vi.fn();

      searchManager.updateState({
        searchInput: 'test query',
        onQueryCommit
      });

      expect(searchManager.isLoading).toBe(false);
      
      await new Promise(resolve => setTimeout(resolve, 150)); // Wait for debounce

      expect(mockInvoke).toHaveBeenCalledWith('search_notes', { query: 'test query' });
      expect(onQueryCommit).toHaveBeenCalledWith('test query');
      expect(searchManager.filteredNotes).toEqual(notes);
    });

    it('should handle immediate search', async () => {
      const notes = ['immediate.md'];
      mockInvoke.mockResolvedValueOnce(notes);

      const result = await searchManager.searchImmediate('immediate');

      expect(mockInvoke).toHaveBeenCalledWith('search_notes', { query: 'immediate' });
      expect(result).toEqual(notes);
      expect(searchManager.filteredNotes).toEqual(notes);
    });

    it('should abort ongoing operations', () => {
      // Test the public interface behavior rather than internal implementation
      searchManager.updateState({ searchInput: 'test' });
      
      // Verify abort works without errors and resets loading state
      expect(() => searchManager.abort()).not.toThrow();
      expect(searchManager.isLoading).toBe(false);
    });
  });

  describe('search clearing functionality (to be added)', () => {
    it('should have clearSearch method that resets search input', () => {
      searchManager.updateState({ searchInput: 'some query' });

      // This should exist but doesn't yet - RED test
      searchManager.clearSearch();

      // Should reset to empty and trigger effects
      expect(searchManager.searchInput).toBe('');
      expect(searchManager.areHighlightsCleared).toBe(false);
    });

    it('should have searchInput getter', () => {
      searchManager.updateState({ searchInput: 'test input' });

      // This should exist but doesn't yet - RED test
      expect(searchManager.searchInput).toBe('test input');
    });

    it('should have areHighlightsCleared getter and setter', () => {
      // This should exist but doesn't yet - RED test  
      searchManager.areHighlightsCleared = true;
      expect(searchManager.areHighlightsCleared).toBe(true);
    });

    it('should handle search input coordination with highlight clearing', () => {
      const onQueryCommit = vi.fn();
      const onHighlightsClear = vi.fn();

      // This functionality should be moved from main component - RED test
      searchManager.updateSearchInputWithEffects('new query', onQueryCommit, onHighlightsClear);

      expect(searchManager.searchInput).toBe('new query');
      expect(searchManager.areHighlightsCleared).toBe(false);
      expect(onHighlightsClear).toHaveBeenCalledWith(false);
    });
  });
});