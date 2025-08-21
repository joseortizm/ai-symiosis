import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mockInvoke, resetAllMocks } from '../../test-utils'
import { createSearchManager } from '../../../lib/core/searchManager.svelte'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}))

// Create a fresh instance for each test
let searchManager: ReturnType<typeof createSearchManager>
let mockNoteService: { search: ReturnType<typeof vi.fn> }

describe('searchManager', () => {
  beforeEach(() => {
    resetAllMocks()
    vi.clearAllMocks()
    mockNoteService = {
      search: vi.fn(),
    }
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    searchManager = createSearchManager({ noteService: mockNoteService as any })
  })

  describe('existing functionality', () => {
    it('should update search state with debouncing', async () => {
      const notes = ['note1.md', 'note2.md']
      mockNoteService.search.mockResolvedValueOnce(notes)

      // Use fake timers BEFORE setting searchInput
      vi.useFakeTimers()

      searchManager.searchInput = 'test query'

      expect(searchManager.isLoading).toBe(false)

      // Fast-forward past debounce delay
      vi.advanceTimersByTime(200)

      // Wait for promise resolution
      await vi.runAllTimersAsync()

      expect(mockNoteService.search).toHaveBeenCalledWith('test query')
      expect(searchManager.query).toBe('test query')
      expect(searchManager.filteredNotes).toEqual(notes)

      vi.useRealTimers()
    })

    it('should handle immediate search', async () => {
      const notes = ['immediate.md']
      mockNoteService.search.mockResolvedValueOnce(notes)

      const result = await searchManager.searchImmediate('immediate')

      expect(mockNoteService.search).toHaveBeenCalledWith('immediate')
      expect(result).toEqual(notes)
      expect(searchManager.filteredNotes).toEqual(notes)
    })

    it('should abort ongoing operations', () => {
      // Test the public interface behavior rather than internal implementation
      searchManager.searchInput = 'test'

      // Verify abort works without errors and resets loading state
      expect(() => searchManager.abort()).not.toThrow()
      expect(searchManager.isLoading).toBe(false)
    })
  })

  describe('search clearing functionality', () => {
    it('should clear search input and query', () => {
      searchManager.searchInput = 'some query'

      searchManager.clearSearch()
      expect(searchManager.searchInput).toBe('')
      expect(searchManager.query).toBe('')
    })

    it('should provide searchInput getter', () => {
      searchManager.searchInput = 'test input'

      expect(searchManager.searchInput).toBe('test input')
    })

    it('should handle search input coordination with highlight clearing', () => {
      const onHighlightsClear = vi.fn()

      searchManager.updateSearchInputWithEffects('new query', onHighlightsClear)

      expect(searchManager.searchInput).toBe('new query')
      expect(onHighlightsClear).toHaveBeenCalledWith(false)
    })

    it('should actually trigger search when using updateSearchInputWithEffects', async () => {
      const notes = ['search-result.md', 'another-note.md']
      mockNoteService.search.mockResolvedValue(notes)
      const onHighlightsClear = vi.fn()

      // Use fake timers BEFORE calling updateSearchInputWithEffects
      vi.useFakeTimers()

      // CRITICAL: This test verifies search execution actually happens (catches state pre-setting bugs)
      searchManager.updateSearchInputWithEffects(
        'test search',
        onHighlightsClear
      )

      // Fast-forward past debounce delay
      vi.advanceTimersByTime(200)

      // Wait for promise resolution
      await vi.runAllTimersAsync()

      // The critical assertion: verify search was actually performed
      expect(mockNoteService.search).toHaveBeenCalledWith('test search')
      expect(searchManager.filteredNotes).toEqual(notes)
      expect(searchManager.query).toBe('test search')

      vi.useRealTimers()
    })

    it('should provide refreshSearch method', async () => {
      const notes = ['refresh.md', 'test.md']
      mockNoteService.search.mockResolvedValueOnce(notes)

      const result = await searchManager.refreshSearch('refresh query')

      expect(mockNoteService.search).toHaveBeenCalledWith('refresh query')
      expect(result).toEqual(notes)
      expect(searchManager.filteredNotes).toEqual(notes)
    })
  })
})
