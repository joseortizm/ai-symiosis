import { invoke } from '@tauri-apps/api/core';
import { contentHighlighter } from './contentHighlighting.svelte';
import { focusManager } from './focusManager.svelte';
import { searchManager } from './searchManager.svelte';
import { noteService } from '../services/noteService.svelte';

interface ContentState {
  noteContent: string;
  areHighlightsCleared: boolean;
}

const state = $state<ContentState>({
  noteContent: '',
  areHighlightsCleared: false
});

interface RefreshAfterSaveResult {
  searchResults: string[];
  content: string;
}

function setNoteContent(content: string): void {
  state.noteContent = content;
  contentHighlighter.updateState({ content });
}

function clearHighlights(): void {
  state.areHighlightsCleared = true;
  contentHighlighter.areHighlightsCleared = true;
}

function scrollToFirstMatch(): void {
  if (focusManager.noteContentElement && !state.areHighlightsCleared) {
    setTimeout(() => {
      const firstMatch = focusManager.noteContentElement!.querySelector('.highlight');
      if (firstMatch) {
        firstMatch.scrollIntoView({ behavior: 'smooth', block: 'center' });
      }
    }, 100);
  }
}


async function refreshContent(noteName: string): Promise<string> {
  const content = await noteService.getContent(noteName);
  setNoteContent(content);
  return content;
}

async function refreshAfterSave(noteName: string, searchInput: string): Promise<RefreshAfterSaveResult> {
  // Refresh cache
  await invoke<void>("refresh_cache");

  // Refresh search
  const searchResults = await searchManager.refreshSearch(searchInput);

  // Refresh content
  const content = await refreshContent(noteName);

  return {
    searchResults,
    content
  };
}

function updateHighlighterState(newState: {
  content?: string;
  query?: string;
  areHighlightsCleared?: boolean;
}): void {
  contentHighlighter.updateState(newState);
  if (newState.areHighlightsCleared !== undefined) {
    state.areHighlightsCleared = newState.areHighlightsCleared;
  }
}

function setHighlightsClearedState(cleared: boolean): void {
  state.areHighlightsCleared = cleared;
  contentHighlighter.areHighlightsCleared = cleared;
}

searchManager.setHighlightsClearCallback((cleared: boolean) => {
  setHighlightsClearedState(cleared);
});

export const contentManager = {
  // Reactive getters
  get noteContent(): string {
    return state.noteContent;
  },

  get highlightedContent(): string {
    return contentHighlighter.highlighted;
  },

  get areHighlightsCleared(): boolean {
    return state.areHighlightsCleared;
  },

  set areHighlightsCleared(value: boolean) {
    setHighlightsClearedState(value);
  },

  // Content actions
  setNoteContent,
  clearHighlights,
  scrollToFirstMatch,

  // Content refresh workflows
  refreshContent,
  refreshAfterSave,

  // Integration helper for updating contentHighlighter state
  updateHighlighterState,
  setHighlightsClearedState
};
