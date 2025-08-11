/**
 * Core Layer - Content Manager
 * Note content loading, caching, and display with search highlighting.
 * Coordinates between content fetching, highlighting service, and UI updates.
 */

import { invoke } from '@tauri-apps/api/core';
import { getHighlightedContent } from './contentHighlighting.svelte';

interface ContentManagerDeps {
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

  const highlightedContent = $derived(
    getHighlightedContent(
      state.noteContent,
      deps.searchManager.query,
      deps.searchManager.areHighlightsCleared
    )
  );

  function setNoteContent(content: string): void {
    state.noteContent = content;
  }

  function clearHighlights(): void {
    deps.searchManager.clearHighlights();
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
  }

  function setHighlightsClearedState(cleared: boolean): void {
    deps.searchManager.areHighlightsCleared = cleared;
  }

  deps.searchManager.setHighlightsClearCallback((cleared: boolean) => {
    setHighlightsClearedState(cleared);
  });

  return {
    get noteContent(): string {
      return state.noteContent;
    },

    get highlightedContent(): string {
      return highlightedContent;
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

