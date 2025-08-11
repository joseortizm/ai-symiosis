/**
 * App Layer - Search Actions
 * Search-related operations including clearing search state
 * and managing filtered note results.
 */

interface SearchActionDeps {
  searchManager: {
    clearSearch: () => void;
    setFilteredNotes: (notes: string[]) => void;
  };
  contentManager: {
    clearHighlights: () => void;
  };
}

interface SearchActions {
  updateFilteredNotes(notes: string[]): void;
  resetSearchState(): void;
  clearHighlights(): void;
  clearSearch(): void;
}

export function createSearchActions(deps: SearchActionDeps): SearchActions {
  const { searchManager, contentManager } = deps;

  function updateFilteredNotes(notes: string[]): void {
    searchManager.setFilteredNotes(notes);
  }

  function resetSearchState(): void {
    searchManager.clearSearch();
    searchManager.setFilteredNotes([]);
    contentManager.clearHighlights();
  }

  return {
    updateFilteredNotes,
    resetSearchState,
    clearHighlights: contentManager.clearHighlights,
    clearSearch: searchManager.clearSearch
  };
}
