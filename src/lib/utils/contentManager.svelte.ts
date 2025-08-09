import { invoke } from '@tauri-apps/api/core';
import { contentHighlighter } from './contentHighlighting.svelte';
import { focusManager } from './focusManager.svelte';
import { searchManager } from './searchManager.svelte';
import { noteService } from '../services/noteService.svelte';

interface ContentState {
  noteContent: string;
}

const state = $state<ContentState>({
  noteContent: ''
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
  searchManager.areHighlightsCleared = true;
}

function scrollToFirstMatch(): void {
  if (focusManager.noteContentElement && !searchManager.areHighlightsCleared) {
    setTimeout(() => {
      const firstMatch = focusManager.noteContentElement!.querySelector('.highlight');
      if (firstMatch) {
        firstMatch.scrollIntoView({ behavior: 'smooth', block: 'center' });
      }
    }, 100);
  }
}

async function getNoteContent(noteName: string): Promise<string> {
  return await noteService.getContent(noteName);
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
  const searchResults = await searchManager.searchImmediate(searchInput);

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
}

export const contentManager = {
  // Reactive getters
  get noteContent(): string {
    return state.noteContent;
  },

  get highlightedContent(): string {
    return contentHighlighter.highlighted;
  },

  // Content actions
  setNoteContent,
  clearHighlights,
  scrollToFirstMatch,

  // Content access
  getNoteContent,

  // Content refresh workflows
  refreshContent,
  refreshAfterSave,

  // Integration helper for updating contentHighlighter state
  updateHighlighterState
};
