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
  focusManager: {
    selectedIndex: number;
    setSelectedIndex: (index: number) => void;
  };
  editorManager: {
    exitEditMode: () => void;
  };
}

interface SearchActions {
  updateFilteredNotes(notes: string[]): void;
  resetSearchState(): void;
  clearHighlights(): void;
  clearSearch(): void;
}

export function createSearchActions(deps: SearchActionDeps): SearchActions {
  const { searchManager, contentManager, focusManager, editorManager } = deps;

  function updateFilteredNotes(notes: string[]): void {
    searchManager.setFilteredNotes(notes);

    // Handle selection normalization when filtered notes change
    const currentIndex = focusManager.selectedIndex;

    if (notes.length > 0 && (currentIndex === -1 || currentIndex >= notes.length)) {
      editorManager.exitEditMode();
      focusManager.setSelectedIndex(0);
    }
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
