import { invoke } from "@tauri-apps/api/core";

interface SearchState {
  searchInput: string;
  isLoading: boolean;
  searchTimeout: number | undefined;
  requestController: AbortController | null;
  filteredNotes: string[];
}

const state = $state<SearchState>({
  searchInput: '',
  isLoading: false,
  searchTimeout: undefined,
  requestController: null,
  filteredNotes: []
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

export const searchManager = {
  updateState(newState: Partial<SearchState>): void {
    // Handle search input change with debouncing
    if (newState.searchInput !== undefined && newState.searchInput !== state.searchInput) {
      clearTimeout(state.searchTimeout);
      state.requestController?.abort();
      
      Object.assign(state, newState);
      
      state.searchTimeout = setTimeout(async () => {
        await performSearch(state.searchInput);
      }, 100);
    } else {
      Object.assign(state, newState);
    }
  },

  get isLoading(): boolean {
    return state.isLoading;
  },

  get filteredNotes(): string[] {
    return state.filteredNotes;
  },

  async searchImmediate(query: string): Promise<string[]> {
    await performSearch(query);
    return state.filteredNotes;
  },

  abort(): void {
    clearTimeout(state.searchTimeout);
    state.requestController?.abort();
    state.requestController = null;
    state.isLoading = false;
  }
};