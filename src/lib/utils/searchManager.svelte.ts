import { invoke } from "@tauri-apps/api/core";

interface SearchState {
  searchInput: string;
  isLoading: boolean;
  searchTimeout: number | undefined;
  requestController: AbortController | null;
  filteredNotes: string[];
  onQueryCommit?: (query: string) => void;
  areHighlightsCleared: boolean;
}

const state = $state<SearchState>({
  searchInput: '',
  isLoading: false,
  searchTimeout: undefined,
  requestController: null,
  filteredNotes: [],
  onQueryCommit: undefined,
  areHighlightsCleared: false
});

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

function updateState(newState: Partial<SearchState>): void {
  if (newState.searchInput !== undefined && newState.searchInput !== state.searchInput) {
    clearTimeout(state.searchTimeout);
    state.requestController?.abort();

    Object.assign(state, newState);

    state.searchTimeout = setTimeout(async () => {
      if (state.onQueryCommit) {
        state.onQueryCommit(state.searchInput);
      }
      await performSearch(state.searchInput);
    }, 100);
  } else {
    Object.assign(state, newState);
  }
}

function updateSearchInputWithEffects(
  newInput: string,
  onQueryCommit: (query: string) => void,
  onHighlightsClear: (cleared: boolean) => void
): void {
  if (newInput.trim()) {
    state.areHighlightsCleared = false;
    onHighlightsClear(false);
  }

  updateState({
    searchInput: newInput,
    onQueryCommit
  });
}

function clearSearch(): void {
  state.searchInput = '';
  state.areHighlightsCleared = false;
}

export const searchManager = {
  updateState,
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

  get areHighlightsCleared(): boolean {
    return state.areHighlightsCleared;
  },

  set areHighlightsCleared(value: boolean) {
    state.areHighlightsCleared = value;
  },

  async searchImmediate(query: string): Promise<string[]> {
    await performSearch(query);
    return state.filteredNotes;
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
