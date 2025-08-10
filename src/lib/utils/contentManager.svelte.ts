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
  getNoteContentElement: () => HTMLElement | null;
  refreshSearch: (query: string) => Promise<string[]>;
  setHighlightsClearCallback: (callback: (cleared: boolean) => void) => void;
  invoke: typeof invoke;
}

interface ContentState {
  noteContent: string;
  areHighlightsCleared: boolean;
}

interface RefreshAfterSaveResult {
  searchResults: string[];
  content: string;
}

export function createContentManager(deps: ContentManagerDeps) {
  const state = $state<ContentState>({
    noteContent: '',
    areHighlightsCleared: false
  });

  function setNoteContent(content: string): void {
    state.noteContent = content;
    deps.contentHighlighter.setContent(content);
  }

  function clearHighlights(): void {
    state.areHighlightsCleared = true;
    deps.contentHighlighter.areHighlightsCleared = true;
  }

  function scrollToFirstMatch(): void {
    const noteContentElement = deps.getNoteContentElement();
    if (noteContentElement && !state.areHighlightsCleared) {
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
    deps.contentHighlighter.updateHighlighterState(newState);
    if (newState.areHighlightsCleared !== undefined) {
      state.areHighlightsCleared = newState.areHighlightsCleared;
    }
  }

  function setHighlightsClearedState(cleared: boolean): void {
    state.areHighlightsCleared = cleared;
    deps.contentHighlighter.areHighlightsCleared = cleared;
  }

  deps.setHighlightsClearCallback((cleared: boolean) => {
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
      return state.areHighlightsCleared;
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

