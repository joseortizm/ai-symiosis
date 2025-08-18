/**
 * Core Layer - Content Manager
 * Note content loading, caching, and display with search highlighting.
 * Coordinates between content fetching, highlighting service, and UI updates.
 */

import { invoke } from '@tauri-apps/api/core'
import { getHighlightedContent } from './contentHighlighting.svelte'

export interface ContentManagerDeps {
  noteService: {
    getContent: (noteName: string) => Promise<string>
  }
  getQuery: () => string
  getAreHighlightsCleared: () => boolean
  clearHighlights: () => void
  setHighlightsClearCallback: (callback: (cleared: boolean) => void) => void
  setHighlightsClearedState: (cleared: boolean) => void
  getNoteContentElement: () => HTMLElement | null
  refreshSearch: (query: string) => Promise<string[]>
  invoke: typeof invoke
}

interface ContentState {
  noteContent: string
}

interface RefreshAfterSaveResult {
  searchResults: string[]
  content: string
}

export interface ContentManager {
  readonly noteContent: string
  readonly highlightedContent: string
  areHighlightsCleared: boolean
  setNoteContent(content: string): void
  clearHighlights(): void
  scrollToFirstMatch(): void
  refreshContent(noteName: string): Promise<string>
  refreshAfterSave(
    noteName: string,
    searchInput: string
  ): Promise<RefreshAfterSaveResult>
  setHighlightsClearedState(cleared: boolean): void
}

export function createContentManager(deps: ContentManagerDeps): ContentManager {
  const state = $state<ContentState>({
    noteContent: '',
  })

  const highlightedContent = $derived(
    getHighlightedContent(
      state.noteContent,
      deps.getQuery(),
      deps.getAreHighlightsCleared()
    )
  )

  function setNoteContent(content: string): void {
    state.noteContent = content
  }

  function clearHighlights(): void {
    deps.clearHighlights()
  }

  function scrollToFirstMatch(): void {
    const noteContentElement = deps.getNoteContentElement()
    if (noteContentElement && !deps.getAreHighlightsCleared()) {
      setTimeout(() => {
        const firstMatch = noteContentElement.querySelector('.highlight')
        if (firstMatch) {
          firstMatch.scrollIntoView({ behavior: 'smooth', block: 'center' })
        }
      }, 100)
    }
  }

  async function refreshContent(noteName: string): Promise<string> {
    const content = await deps.noteService.getContent(noteName)
    setNoteContent(content)
    return content
  }

  async function refreshAfterSave(
    noteName: string,
    searchInput: string
  ): Promise<RefreshAfterSaveResult> {
    const searchResults = await deps.refreshSearch(searchInput)
    const content = await refreshContent(noteName)

    return {
      searchResults,
      content,
    }
  }

  function setHighlightsClearedState(cleared: boolean): void {
    deps.setHighlightsClearedState(cleared)
  }

  deps.setHighlightsClearCallback((cleared: boolean) => {
    setHighlightsClearedState(cleared)
  })

  return {
    get noteContent(): string {
      return state.noteContent
    },

    get highlightedContent(): string {
      return highlightedContent
    },

    get areHighlightsCleared(): boolean {
      return deps.getAreHighlightsCleared()
    },

    set areHighlightsCleared(value: boolean) {
      setHighlightsClearedState(value)
    },

    setNoteContent,
    clearHighlights,
    scrollToFirstMatch,
    refreshContent,
    refreshAfterSave,
    setHighlightsClearedState,
  }
}
