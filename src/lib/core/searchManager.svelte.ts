/**
 * Core Layer - Search Manager
 * Search input, debouncing, and filtered note results.
 * Handles hybrid search queries to Rust backend and search state management.
 */

import { invoke } from '@tauri-apps/api/core'

interface SearchState {
  searchInput: string
  query: string
  isLoading: boolean
  searchTimeout: ReturnType<typeof setTimeout> | undefined
  requestController: AbortController | null
  filteredNotes: string[]
  areHighlightsCleared: boolean
}

interface SearchManager {
  readonly isLoading: boolean
  readonly filteredNotes: string[]
  searchInput: string
  readonly query: string
  areHighlightsCleared: boolean
  setSearchInput(value: string): void
  setFilteredNotes(notes: string[]): void
  updateSearchInputWithEffects(
    newInput: string,
    onHighlightsClear: (cleared: boolean) => void
  ): void
  clearSearch(): void
  clearHighlights(): void
  searchImmediate(query: string): Promise<string[]>
  refreshSearch(searchInput: string): Promise<string[]>
  setHighlightsClearCallback(callback: (cleared: boolean) => void): void
  abort(): void
}

export function createSearchManager(): SearchManager {
  const state = $state<SearchState>({
    searchInput: '',
    query: '',
    isLoading: false,
    searchTimeout: undefined,
    requestController: null,
    filteredNotes: [],
    areHighlightsCleared: false,
  })

  let onHighlightsClearCallback: ((cleared: boolean) => void) | null = null

  async function performSearch(query: string): Promise<void> {
    if (state.requestController) {
      state.requestController.abort()
    }

    state.requestController = new AbortController()
    const currentController = state.requestController

    try {
      state.isLoading = true
      const notes = await invoke<string[]>('search_notes', { query })

      if (currentController.signal.aborted) {
        return
      }

      state.filteredNotes = notes
    } catch (e) {
      if (!currentController.signal.aborted) {
        console.error('❌ Failed to load notes:', e)
        state.filteredNotes = []
      }
    } finally {
      if (!currentController.signal.aborted) {
        state.isLoading = false
      }
    }
  }

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
      state.areHighlightsCleared = false
    }

    setSearchInput(newInput)
  }

  function clearSearch(): void {
    state.searchInput = ''
    state.query = ''
  }

  function clearHighlights(): void {
    state.areHighlightsCleared = true
  }

  return {
    setSearchInput,
    setFilteredNotes,
    clearSearch,
    updateSearchInputWithEffects,

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
      updateSearchInputWithEffects(
        value,
        onHighlightsClearCallback || (() => {})
      )
    },

    setHighlightsClearCallback(callback: (cleared: boolean) => void): void {
      onHighlightsClearCallback = callback
    },

    get query(): string {
      return state.query
    },

    get areHighlightsCleared(): boolean {
      return state.areHighlightsCleared
    },

    set areHighlightsCleared(value: boolean) {
      state.areHighlightsCleared = value
    },

    clearHighlights,

    async searchImmediate(query: string): Promise<string[]> {
      await performSearch(query)
      return state.filteredNotes
    },

    async refreshSearch(searchInput: string): Promise<string[]> {
      const results = await this.searchImmediate(searchInput)
      return results
    },

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
