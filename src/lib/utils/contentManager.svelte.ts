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

export const contentManager = {
  // Reactive getters
  get noteContent(): string {
    return state.noteContent;
  },

  get highlightedContent(): string {
    return contentHighlighter.highlighted;
  },

  // Content actions
  setNoteContent(content: string): void {
    state.noteContent = content;
    contentHighlighter.updateState({ content });
  },

  clearHighlights(): void {
    searchManager.areHighlightsCleared = true;
  },

  scrollToFirstMatch(): void {
    if (focusManager.noteContentElement && !searchManager.areHighlightsCleared) {
      setTimeout(() => {
        const firstMatch = focusManager.noteContentElement!.querySelector('.highlight');
        if (firstMatch) {
          firstMatch.scrollIntoView({ behavior: 'smooth', block: 'center' });
        }
      }, 100);
    }
  },

  // Content access
  async getNoteContent(noteName: string): Promise<string> {
    return await noteService.getContent(noteName);
  },

  // Content refresh workflows
  async refreshContent(noteName: string): Promise<string> {
    const content = await noteService.getContent(noteName);
    this.setNoteContent(content);
    return content;
  },

  async refreshAfterSave(noteName: string, searchInput: string): Promise<RefreshAfterSaveResult> {
    // Refresh cache
    await invoke<void>("refresh_cache");

    // Refresh search
    const searchResults = await searchManager.searchImmediate(searchInput);

    // Refresh content
    const content = await this.refreshContent(noteName);

    return {
      searchResults,
      content
    };
  },

  // Integration helper for updating contentHighlighter state
  updateHighlighterState(newState: {
    content?: string;
    query?: string;
    areHighlightsCleared?: boolean;
  }): void {
    contentHighlighter.updateState(newState);
  }
};
