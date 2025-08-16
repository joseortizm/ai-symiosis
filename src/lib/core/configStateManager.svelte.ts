/**
 * Core Layer - Config State Manager
 * Reactive configuration state management with real-time updates from backend.
 * Listens to Tauri config-changed events and provides reactive access to config values.
 */

import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { configService } from '../services/configService.svelte'
import type { ThemeConfig } from '../services/configService.svelte'

interface ConfigState {
  notesDirectory: string
  maxSearchResults: number
  globalShortcut: string
  editorMode: string
  markdownTheme: string
  theme: ThemeConfig
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
  theme: ThemeConfig
}

export interface ConfigStateManager {
  readonly notesDirectory: string
  readonly maxSearchResults: number
  readonly globalShortcut: string
  readonly editorMode: string
  readonly markdownTheme: string
  readonly theme: ThemeConfig
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
    theme: {
      name: 'gruvbox-dark',
      font_family: 'Inter, sans-serif',
      font_size: 14,
      editor_font_family: 'JetBrains Mono, Consolas, monospace',
      editor_font_size: 14,
    },
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
    state.theme = config.theme
  }

  async function initialize(): Promise<void> {
    if (state.isInitialized) {
      return
    }

    state.isLoading = true
    state.error = null

    try {
      // Get initial config values
      const [editorMode, markdownTheme, themeConfig] = await Promise.all([
        configService.getEditorMode(),
        configService.getMarkdownTheme(),
        configService.getThemeConfig(),
      ])

      state.editorMode = editorMode
      state.markdownTheme = markdownTheme
      state.theme = themeConfig

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
      const [editorMode, markdownTheme, themeConfig] = await Promise.all([
        configService.getEditorMode(),
        configService.getMarkdownTheme(),
        configService.getThemeConfig(),
      ])

      state.editorMode = editorMode
      state.markdownTheme = markdownTheme
      state.theme = themeConfig
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

    get theme() {
      return state.theme
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
