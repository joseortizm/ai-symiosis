/**
 * Core Layer - Search Manager
 * Search input, debouncing, and filtered note results.
 * Handles hybrid search queries to Rust backend and search state management.
 */

import type { createNoteService } from '../services/noteService.svelte'

interface SearchState {
  searchInput: string
  query: string
  isLoading: boolean
  searchTimeout: ReturnType<typeof setTimeout> | undefined
  requestController: AbortController | null
  filteredNotes: string[]
}

interface SearchManagerDeps {
  noteService: ReturnType<typeof createNoteService>
}

interface SearchManager {
  readonly isLoading: boolean
  readonly filteredNotes: string[]
  searchInput: string
  readonly query: string
  setSearchInput(value: string): void
  setFilteredNotes(notes: string[]): void
  updateSearchInputWithEffects(
    newInput: string,
    onHighlightsClear: (cleared: boolean) => void
  ): void
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
    isLoading: false,
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
      state.isLoading = true
      const notes = await deps.noteService.search(query)

      if (currentController.signal.aborted) {
        return
      }

      state.filteredNotes = notes
      onSearchCompleteCallback?.(notes)
    } catch (e) {
      if (!currentController.signal.aborted) {
        console.error('❌ Failed to load notes:', e)
        state.filteredNotes = []
        onSearchCompleteCallback?.([])
      }
    } finally {
      if (!currentController.signal.aborted) {
        state.isLoading = false
      }
    }
  }

  // Core search operations
  function setSearchInput(value: string): void {
    if (value !== state.searchInput) {
      clearTimeout(state.searchTimeout)
      state.requestController?.abort()

      state.searchInput = value

      state.searchTimeout = setTimeout(async () => {
        state.query = state.searchInput
        await performSearch(state.searchInput)
      }, 100)
    }
  }

  function setFilteredNotes(notes: string[]): void {
    state.filteredNotes = notes
  }

  function updateSearchInputWithEffects(
    newInput: string,
    onHighlightsClear: (cleared: boolean) => void
  ): void {
    if (newInput.trim()) {
      onHighlightsClear(false)
    }

    setSearchInput(newInput)
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
    updateSearchInputWithEffects,

    // State getters/setters
    get isLoading(): boolean {
      return state.isLoading
    },
    get filteredNotes(): string[] {
      return state.filteredNotes
    },
    get searchInput(): string {
      return state.searchInput
    },
    set searchInput(value: string) {
      updateSearchInputWithEffects(value, () => {})
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

      state.isLoading = false
    },
  }
}
