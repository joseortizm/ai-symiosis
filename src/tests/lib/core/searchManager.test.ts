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

  describe('minimum search length', () => {
    it('should show recent notes on initial load', async () => {
      const recentNotes = ['recent1.md', 'recent2.md']
      mockNoteService.search.mockResolvedValueOnce(recentNotes)

      // Trigger initial load by calling refreshSearch with empty string
      await searchManager.refreshSearch('')

      expect(mockNoteService.search).toHaveBeenCalledWith('')
      expect(searchManager.filteredNotes).toEqual(recentNotes)
    })

    it('should show recent notes for queries shorter than 3 characters', async () => {
      const recentNotes = ['recent1.md', 'recent2.md', 'recent3.md']
      mockNoteService.search.mockResolvedValueOnce(recentNotes)
      vi.useFakeTimers()

      // First set a non-empty value, then set to empty to trigger the change
      searchManager.searchInput = 'test'
      searchManager.searchInput = ''
      vi.advanceTimersByTime(200)
      await vi.runAllTimersAsync()
      expect(mockNoteService.search).toHaveBeenCalledWith('')
      expect(searchManager.filteredNotes).toEqual(recentNotes)

      // Reset for next test
      mockNoteService.search.mockClear()
      mockNoteService.search.mockResolvedValueOnce(recentNotes)

      // Test 1 character - should show recent notes
      searchManager.searchInput = 'a'
      vi.advanceTimersByTime(200)
      await vi.runAllTimersAsync()
      expect(mockNoteService.search).toHaveBeenCalledWith('')
      expect(searchManager.filteredNotes).toEqual(recentNotes)

      // Reset for next test
      mockNoteService.search.mockClear()
      mockNoteService.search.mockResolvedValueOnce(recentNotes)

      // Test 2 characters - should show recent notes
      searchManager.searchInput = 'ab'
      vi.advanceTimersByTime(200)
      await vi.runAllTimersAsync()
      expect(mockNoteService.search).toHaveBeenCalledWith('')
      expect(searchManager.filteredNotes).toEqual(recentNotes)

      vi.useRealTimers()
    })

    it('should trigger search for queries with 3 or more characters', async () => {
      const notes = ['note1.md', 'note2.md']
      mockNoteService.search.mockResolvedValueOnce(notes)
      vi.useFakeTimers()

      // Test exactly 3 characters
      searchManager.searchInput = 'abc'
      vi.advanceTimersByTime(200)
      await vi.runAllTimersAsync()

      expect(mockNoteService.search).toHaveBeenCalledWith('abc')
      expect(searchManager.filteredNotes).toEqual(notes)

      vi.useRealTimers()
    })

    it('should show recent notes when search input drops below 3 characters', async () => {
      const searchNotes = ['search1.md']
      const recentNotes = ['recent1.md', 'recent2.md']

      mockNoteService.search.mockResolvedValueOnce(searchNotes)
      vi.useFakeTimers()

      // First set a valid search
      searchManager.searchInput = 'test'
      vi.advanceTimersByTime(200)
      await vi.runAllTimersAsync()
      expect(searchManager.filteredNotes).toEqual(searchNotes)

      // Reset mock and prepare recent notes response
      mockNoteService.search.mockClear()
      mockNoteService.search.mockResolvedValueOnce(recentNotes)

      // Then reduce to less than 3 characters - should show recent notes
      searchManager.searchInput = 'te'
      vi.advanceTimersByTime(200)
      await vi.runAllTimersAsync()
      expect(mockNoteService.search).toHaveBeenCalledWith('')
      expect(searchManager.filteredNotes).toEqual(recentNotes)

      vi.useRealTimers()
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
