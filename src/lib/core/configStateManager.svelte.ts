/**
 * Core Layer - Config State Manager
 * Reactive configuration state management with real-time updates from backend.
 * Listens to Tauri config-changed events and provides reactive access to config values.
 */

import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'

interface ConfigState {
  notesDirectory: string
  maxSearchResults: number
  globalShortcut: string
  editorMode: string
  markdownTheme: string
  isLoading: boolean
  error: string | null
  isInitialized: boolean
}

interface ConfigChanged {
  notes_directory: string
  max_search_results: number
  global_shortcut: string
  editor_mode: string
  markdown_theme: string
}

export interface ConfigStateManager {
  readonly notesDirectory: string
  readonly maxSearchResults: number
  readonly globalShortcut: string
  readonly editorMode: string
  readonly markdownTheme: string
  readonly isLoading: boolean
  readonly error: string | null
  readonly isInitialized: boolean
  initialize(): Promise<void>
  cleanup(): void
  forceRefresh(): Promise<void>
}

export function createConfigStateManager(): ConfigStateManager {
  const state = $state<ConfigState>({
    notesDirectory: '',
    maxSearchResults: 100,
    globalShortcut: 'Ctrl+Shift+N',
    editorMode: 'basic',
    markdownTheme: 'dark_dimmed',
    isLoading: false,
    error: null,
    isInitialized: false,
  })

  let unlistenConfigChanged: (() => void) | null = null

  function updateConfigState(config: ConfigChanged): void {
    state.notesDirectory = config.notes_directory
    state.maxSearchResults = config.max_search_results
    state.globalShortcut = config.global_shortcut
    state.editorMode = config.editor_mode
    state.markdownTheme = config.markdown_theme
  }

  async function initialize(): Promise<void> {
    if (state.isInitialized) {
      return
    }

    state.isLoading = true
    state.error = null

    try {
      // Get initial config values
      const [editorMode, markdownTheme] = await Promise.all([
        invoke<string>('get_editor_mode'),
        invoke<string>('get_markdown_theme'),
      ])

      state.editorMode = editorMode
      state.markdownTheme = markdownTheme

      // Listen for config changes
      unlistenConfigChanged = await listen<ConfigChanged>(
        'config-changed',
        (event) => {
          updateConfigState(event.payload)
        }
      )

      state.isInitialized = true
    } catch (e) {
      const error = `Failed to initialize config state: ${e}`
      state.error = error
      console.error('Failed to initialize config state:', e)
    } finally {
      state.isLoading = false
    }
  }

  async function forceRefresh(): Promise<void> {
    state.isLoading = true
    state.error = null

    try {
      // Force refresh from backend
      await invoke<void>('refresh_cache')

      // Get fresh config values
      const [editorMode, markdownTheme] = await Promise.all([
        invoke<string>('get_editor_mode'),
        invoke<string>('get_markdown_theme'),
      ])

      state.editorMode = editorMode
      state.markdownTheme = markdownTheme
    } catch (e) {
      const error = `Failed to refresh config: ${e}`
      state.error = error
      console.error('Failed to refresh config:', e)
    } finally {
      state.isLoading = false
    }
  }

  function cleanup(): void {
    if (unlistenConfigChanged) {
      unlistenConfigChanged()
      unlistenConfigChanged = null
    }
    state.isInitialized = false
  }

  return {
    // Reactive getters following existing manager patterns
    get notesDirectory() {
      return state.notesDirectory
    },

    get maxSearchResults() {
      return state.maxSearchResults
    },

    get globalShortcut() {
      return state.globalShortcut
    },

    get editorMode() {
      return state.editorMode
    },

    get markdownTheme() {
      return state.markdownTheme
    },

    get isLoading() {
      return state.isLoading
    },

    get error() {
      return state.error
    },

    get isInitialized() {
      return state.isInitialized
    },

    // Actions
    initialize,
    cleanup,
    forceRefresh,
  }
}
