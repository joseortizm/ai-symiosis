/**
 * Core Layer - Search Manager
 * Search input, debouncing, and filtered note results.
 * Handles hybrid search queries to Rust backend and search state management.
 */

import type { createNoteService } from '../services/noteService.svelte'

interface SearchState {
  searchInput: string
  query: string
  searchTimeout: ReturnType<typeof setTimeout> | undefined
  requestController: AbortController | null
  filteredNotes: string[]
}

interface SearchManagerDeps {
  noteService: ReturnType<typeof createNoteService>
  progressManager: {
    readonly isLoading: boolean
    start(message: string): void
    complete(): void
    setError(errorMessage: string): void
  }
}

interface SearchManager {
  readonly isLoading: boolean
  readonly filteredNotes: string[]
  searchInput: string
  readonly query: string
  setSearchInput(value: string): void
  setFilteredNotes(notes: string[]): void
  clearSearch(): void
  searchImmediate(query: string): Promise<string[]>
  refreshSearch(searchInput: string): Promise<string[]>
  setSearchCompleteCallback(callback: (notes: string[]) => void): void
  abort(): void
}

// Manager factory function
export function createSearchManager(deps: SearchManagerDeps): SearchManager {
  const state = $state<SearchState>({
    searchInput: '',
    query: '',
    searchTimeout: undefined,
    requestController: null,
    filteredNotes: [],
  })

  // Callback storage
  let onSearchCompleteCallback: ((notes: string[]) => void) | null = null

  // Private helper functions
  async function performSearch(query: string): Promise<void> {
    if (state.requestController) {
      state.requestController.abort()
    }

    state.requestController = new AbortController()
    const currentController = state.requestController

    try {
      deps.progressManager.start('Searching notes...')
      const notes = await deps.noteService.search(query)

      if (currentController.signal.aborted) {
        return
      }

      state.filteredNotes = notes
      onSearchCompleteCallback?.(notes)
      deps.progressManager.complete()
    } catch (e) {
      if (!currentController.signal.aborted) {
        console.error('❌ Failed to load notes:', e)
        deps.progressManager.setError('Failed to search notes')
        state.filteredNotes = []
        onSearchCompleteCallback?.([])
      }
    }
  }

  // Core search operations
  function setSearchInput(value: string): void {
    if (value !== state.searchInput) {
      clearTimeout(state.searchTimeout)
      state.requestController?.abort()

      state.searchInput = value

      if (value.length < 3) {
        state.query = ''
        // Show recent notes by searching with empty string
        state.searchTimeout = setTimeout(async () => {
          await performSearch('')
        }, 100)
        return
      }

      state.searchTimeout = setTimeout(async () => {
        state.query = state.searchInput
        await performSearch(state.searchInput)
      }, 100)
    }
  }

  function setFilteredNotes(notes: string[]): void {
    state.filteredNotes = notes
  }

  function clearSearch(): void {
    setSearchInput('')
  }

  // Public API
  return {
    // Core operations
    setSearchInput,
    setFilteredNotes,
    clearSearch,

    // State getters/setters
    get isLoading(): boolean {
      return deps.progressManager.isLoading
    },
    get filteredNotes(): string[] {
      return state.filteredNotes
    },
    get searchInput(): string {
      return state.searchInput
    },
    set searchInput(value: string) {
      setSearchInput(value)
    },
    get query(): string {
      return state.query
    },

    // Callback management
    setSearchCompleteCallback(callback: (notes: string[]) => void): void {
      onSearchCompleteCallback = callback
    },

    // Advanced search operations
    async searchImmediate(query: string): Promise<string[]> {
      await performSearch(query)
      return state.filteredNotes
    },
    async refreshSearch(searchInput: string): Promise<string[]> {
      const results = await this.searchImmediate(searchInput)
      return results
    },

    // Cleanup
    abort(): void {
      if (state.searchTimeout !== undefined) {
        clearTimeout(state.searchTimeout)
        state.searchTimeout = undefined
      }

      if (state.requestController) {
        state.requestController.abort()
        state.requestController = null
      }

      deps.progressManager.complete()
    },
  }
}
