import { invoke } from "@tauri-apps/api/core";

interface SearchState {
  searchInput: string;
  query: string;
  isLoading: boolean;
  searchTimeout: NodeJS.Timeout | undefined;
  requestController: AbortController | null;
  filteredNotes: string[];
}

export function createSearchManager() {
  const state = $state<SearchState>({
    searchInput: '',
    query: '',
    isLoading: false,
    searchTimeout: undefined,
    requestController: null,
    filteredNotes: []
  });

  let onHighlightsClearCallback: ((cleared: boolean) => void) | null = null;

  async function performSearch(query: string): Promise<void> {
    if (state.requestController) {
      state.requestController.abort();
    }

    state.requestController = new AbortController();
    const currentController = state.requestController;

    try {
      state.isLoading = true;
      const notes = await invoke<string[]>("search_notes", { query });

      if (currentController.signal.aborted) {
        return;
      }

      state.filteredNotes = notes;
    } catch (e) {
      if (!currentController.signal.aborted) {
        console.error('❌ Failed to load notes:', e);
        state.filteredNotes = [];
      }
    } finally {
      if (!currentController.signal.aborted) {
        state.isLoading = false;
      }
    }
  }

  function setSearchInput(value: string): void {
    if (value !== state.searchInput) {
      clearTimeout(state.searchTimeout);
      state.requestController?.abort();

      state.searchInput = value;

      state.searchTimeout = setTimeout(async () => {
        state.query = state.searchInput;
        await performSearch(state.searchInput);
      }, 100);
    }
  }

  function setFilteredNotes(notes: string[]): void {
    state.filteredNotes = notes;
  }

  function updateSearchInputWithEffects(
    newInput: string,
    onHighlightsClear: (cleared: boolean) => void
  ): void {
    if (newInput.trim()) {
      onHighlightsClear(false);
    }

    setSearchInput(newInput);
  }

  function clearSearch(): void {
    state.searchInput = '';
    state.query = '';
  }

  return {
    setSearchInput,
    setFilteredNotes,
    clearSearch,
    updateSearchInputWithEffects,

    get isLoading(): boolean {
      return state.isLoading;
    },

    get filteredNotes(): string[] {
      return state.filteredNotes;
    },

    get searchInput(): string {
      return state.searchInput;
    },

    set searchInput(value: string) {
      updateSearchInputWithEffects(value, onHighlightsClearCallback || (() => {}));
    },

    setHighlightsClearCallback(callback: (cleared: boolean) => void): void {
      onHighlightsClearCallback = callback;
    },

    get query(): string {
      return state.query;
    },

    async searchImmediate(query: string): Promise<string[]> {
      await performSearch(query);
      return state.filteredNotes;
    },

    async refreshSearch(searchInput: string): Promise<string[]> {
      const results = await this.searchImmediate(searchInput);
      return results;
    },

    abort(): void {
      if (state.searchTimeout !== undefined) {
        clearTimeout(state.searchTimeout);
        state.searchTimeout = undefined;
      }

      if (state.requestController) {
        state.requestController.abort();
        state.requestController = null;
      }

      state.isLoading = false;
    }
  };
}