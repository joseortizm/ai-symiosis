import { invoke } from '@tauri-apps/api/core';

interface ContentManagerDeps {
  contentHighlighter: {
    setContent: (content: string) => void;
    updateHighlighterState: (state: { query?: string; areHighlightsCleared?: boolean }) => void;
    highlighted: string;
    areHighlightsCleared: boolean;
  };
  noteService: {
    getContent: (noteName: string) => Promise<string>;
  };
  searchManager: {
    areHighlightsCleared: boolean;
    clearHighlights: () => void;
    setHighlightsClearCallback: (callback: (cleared: boolean) => void) => void;
    query: string;
  };
  getNoteContentElement: () => HTMLElement | null;
  refreshSearch: (query: string) => Promise<string[]>;
  invoke: typeof invoke;
}

interface ContentState {
  noteContent: string;
}

interface RefreshAfterSaveResult {
  searchResults: string[];
  content: string;
}

export function createContentManager(deps: ContentManagerDeps) {
  const state = $state<ContentState>({
    noteContent: ''
  });

  function setNoteContent(content: string): void {
    state.noteContent = content;
    deps.contentHighlighter.setContent(content);
  }

  function clearHighlights(): void {
    deps.searchManager.clearHighlights();
    deps.contentHighlighter.areHighlightsCleared = true;
  }

  function scrollToFirstMatch(): void {
    const noteContentElement = deps.getNoteContentElement();
    if (noteContentElement && !deps.searchManager.areHighlightsCleared) {
      setTimeout(() => {
        const firstMatch = noteContentElement.querySelector('.highlight');
        if (firstMatch) {
          firstMatch.scrollIntoView({ behavior: 'smooth', block: 'center' });
        }
      }, 100);
    }
  }

  async function refreshContent(noteName: string): Promise<string> {
    const content = await deps.noteService.getContent(noteName);
    setNoteContent(content);
    return content;
  }

  async function refreshAfterSave(noteName: string, searchInput: string): Promise<RefreshAfterSaveResult> {
    await deps.invoke("refresh_cache");
    const searchResults = await deps.refreshSearch(searchInput);
    const content = await refreshContent(noteName);

    return {
      searchResults,
      content
    };
  }

  function updateHighlighterState(newState: {
    query?: string;
    areHighlightsCleared?: boolean;
  }): void {
    // Use searchManager query if none provided
    const actualQuery = newState.query !== undefined ? newState.query : deps.searchManager.query;
    deps.contentHighlighter.updateHighlighterState({
      query: actualQuery,
      areHighlightsCleared: newState.areHighlightsCleared
    });
  }

  function setHighlightsClearedState(cleared: boolean): void {
    deps.searchManager.areHighlightsCleared = cleared;
    deps.contentHighlighter.areHighlightsCleared = cleared;
  }

  deps.searchManager.setHighlightsClearCallback((cleared: boolean) => {
    setHighlightsClearedState(cleared);
  });

  return {
    get noteContent(): string {
      return state.noteContent;
    },

    get highlightedContent(): string {
      return deps.contentHighlighter.highlighted;
    },

    get areHighlightsCleared(): boolean {
      return deps.searchManager.areHighlightsCleared;
    },

    set areHighlightsCleared(value: boolean) {
      setHighlightsClearedState(value);
    },

    setNoteContent,
    clearHighlights,
    scrollToFirstMatch,
    refreshContent,
    refreshAfterSave,
    updateHighlighterState,
    setHighlightsClearedState
  };
}

