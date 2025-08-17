/**
 * Core Layer - Config State Manager
 * Reactive configuration state management with real-time updates from backend.
 * Listens to Tauri config-changed events and provides reactive access to config values.
 * Also handles theme management for UI and markdown rendering.
 */

import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { configService } from '../services/configService.svelte'
import type {
  GeneralConfig,
  InterfaceConfig,
  EditorConfig,
  ShortcutsConfig,
  PreferencesConfig,
} from '../services/configService.svelte'

interface ConfigState {
  notesDirectory: string
  globalShortcut: string
  general: GeneralConfig
  interface: InterfaceConfig
  editor: EditorConfig
  shortcuts: ShortcutsConfig
  preferences: PreferencesConfig
  isLoading: boolean
  error: string | null
  isInitialized: boolean
  isThemeInitialized: boolean
}

interface ConfigChanged {
  notes_directory: string
  global_shortcut: string
  general: GeneralConfig
  interface: InterfaceConfig
  editor: EditorConfig
  shortcuts: ShortcutsConfig
  preferences: PreferencesConfig
}

export interface ConfigStateManager {
  readonly notesDirectory: string
  readonly globalShortcut: string
  readonly general: GeneralConfig
  readonly interface: InterfaceConfig
  readonly editor: EditorConfig
  readonly shortcuts: ShortcutsConfig
  readonly preferences: PreferencesConfig
  readonly isLoading: boolean
  readonly error: string | null
  readonly isInitialized: boolean
  readonly isThemeInitialized: boolean
  readonly currentUITheme: string
  readonly currentMarkdownTheme: string
  initialize(): Promise<void>
  cleanup(): void
  forceRefresh(): Promise<void>
  loadTheme(theme: string): Promise<void>
  loadMarkdownTheme(theme: string): Promise<void>
}

export function createConfigStateManager(): ConfigStateManager {
  const state = $state<ConfigState>({
    notesDirectory: '',
    globalShortcut: 'Ctrl+Shift+N',
    general: {},
    interface: {
      ui_theme: 'gruvbox-dark',
      font_family: 'Inter, sans-serif',
      font_size: 14,
      editor_font_family: 'JetBrains Mono, Consolas, monospace',
      editor_font_size: 14,
      markdown_render_theme: 'dark_dimmed',
      default_width: 1200,
      default_height: 800,
      center_on_startup: true,
      remember_size: true,
      remember_position: true,
      always_on_top: false,
    },
    editor: {
      mode: 'basic',
      theme: 'gruvbox-dark',
      word_wrap: true,
      tab_size: 2,
      show_line_numbers: true,
    },
    shortcuts: {
      create_note: 'Ctrl+Enter',
      rename_note: 'Ctrl+m',
      delete_note: 'Ctrl+x',
      save_and_exit: 'Ctrl+s',
      open_external: 'Ctrl+o',
      open_folder: 'Ctrl+f',
      refresh_cache: 'Ctrl+r',
      scroll_up: 'Ctrl+u',
      scroll_down: 'Ctrl+d',
      vim_up: 'Ctrl+k',
      vim_down: 'Ctrl+j',
      navigate_previous: 'Ctrl+p',
      navigate_next: 'Ctrl+n',
      open_settings: 'Meta+,',
    },
    preferences: {
      max_search_results: 100,
    },
    isLoading: false,
    error: null,
    isInitialized: false,
    isThemeInitialized: false,
  })

  let unlistenConfigChanged: (() => void) | null = null

  // Theme definitions for CSS custom properties
  const THEME_DEFINITIONS = {
    'gruvbox-dark': {
      '--theme-bg-primary': '#282828',
      '--theme-bg-secondary': '#3c3836',
      '--theme-bg-tertiary': '#504945',
      '--theme-text-primary': '#ebdbb2',
      '--theme-text-secondary': '#d5c4a1',
      '--theme-text-muted': '#928374',
      '--theme-accent': '#83a598',
      '--theme-accent-hover': '#8ec07c',
      '--theme-highlight': '#fe8019',
      '--theme-highlight-bg': 'rgba(254, 128, 25, 0.2)',
      '--theme-warning': '#fb4934',
      '--theme-success': '#b8bb26',
      '--theme-border': '#504945',
      '--theme-border-focus': '#83a598',
      '--theme-shadow-focus': 'rgba(131, 165, 152, 0.2)',
    },
    'gruvbox-light': {
      '--theme-bg-primary': '#fbf1c7',
      '--theme-bg-secondary': '#f2e5bc',
      '--theme-bg-tertiary': '#ebdbb2',
      '--theme-text-primary': '#3c3836',
      '--theme-text-secondary': '#504945',
      '--theme-text-muted': '#928374',
      '--theme-accent': '#076678',
      '--theme-accent-hover': '#427b58',
      '--theme-highlight': '#af3a03',
      '--theme-highlight-bg': 'rgba(175, 58, 3, 0.2)',
      '--theme-warning': '#cc241d',
      '--theme-success': '#79740e',
      '--theme-border': '#bdae93',
      '--theme-border-focus': '#076678',
      '--theme-shadow-focus': 'rgba(7, 102, 120, 0.2)',
    },
    'one-dark': {
      '--theme-bg-primary': '#282c34',
      '--theme-bg-secondary': '#21252b',
      '--theme-bg-tertiary': '#181a1f',
      '--theme-text-primary': '#abb2bf',
      '--theme-text-secondary': '#9ca0a9',
      '--theme-text-muted': '#5c6370',
      '--theme-accent': '#61afef',
      '--theme-accent-hover': '#56b6c2',
      '--theme-highlight': '#e06c75',
      '--theme-highlight-bg': 'rgba(224, 108, 117, 0.2)',
      '--theme-warning': '#e06c75',
      '--theme-success': '#98c379',
      '--theme-border': '#4b5263',
      '--theme-border-focus': '#61afef',
      '--theme-shadow-focus': 'rgba(97, 175, 239, 0.2)',
    },
    'github-light': {
      '--theme-bg-primary': '#ffffff',
      '--theme-bg-secondary': '#f6f8fa',
      '--theme-bg-tertiary': '#e1e4e8',
      '--theme-text-primary': '#24292e',
      '--theme-text-secondary': '#586069',
      '--theme-text-muted': '#6a737d',
      '--theme-accent': '#0366d6',
      '--theme-accent-hover': '#0366d6',
      '--theme-highlight': '#ffcc02',
      '--theme-highlight-bg': 'rgba(255, 204, 2, 0.2)',
      '--theme-warning': '#d73a49',
      '--theme-success': '#28a745',
      '--theme-border': '#e1e4e8',
      '--theme-border-focus': '#0366d6',
      '--theme-shadow-focus': 'rgba(3, 102, 214, 0.2)',
    },
  }

  function applyInterfaceConfig(interfaceConfig: InterfaceConfig): void {
    // Apply theme colors
    if (
      interfaceConfig.ui_theme &&
      THEME_DEFINITIONS[
        interfaceConfig.ui_theme as keyof typeof THEME_DEFINITIONS
      ]
    ) {
      const themeColors =
        THEME_DEFINITIONS[
          interfaceConfig.ui_theme as keyof typeof THEME_DEFINITIONS
        ]
      const root = document.documentElement

      Object.entries(themeColors).forEach(([property, value]) => {
        root.style.setProperty(property, value)
      })
    }

    // Apply font configuration
    const root = document.documentElement
    root.style.setProperty('--theme-font-family', interfaceConfig.font_family)
    root.style.setProperty(
      '--theme-font-size',
      `${interfaceConfig.font_size}px`
    )
    root.style.setProperty(
      '--theme-editor-font-family',
      interfaceConfig.editor_font_family
    )
    root.style.setProperty(
      '--theme-editor-font-size',
      `${interfaceConfig.editor_font_size}px`
    )
  }

  async function loadTheme(theme: string): Promise<void> {
    if (THEME_DEFINITIONS[theme as keyof typeof THEME_DEFINITIONS]) {
      const themeColors =
        THEME_DEFINITIONS[theme as keyof typeof THEME_DEFINITIONS]
      const root = document.documentElement

      Object.entries(themeColors).forEach(([property, value]) => {
        root.style.setProperty(property, value)
      })
    } else {
      console.warn(`Unknown theme: ${theme}. Using default.`)
    }
  }

  async function loadMarkdownTheme(theme: string): Promise<void> {
    try {
      // Remove existing theme
      const existingLink = document.head.querySelector(
        'link[data-markdown-theme]'
      )
      if (existingLink) {
        existingLink.remove()
      }

      // Create new theme link
      const link = document.createElement('link')
      link.rel = 'stylesheet'
      link.href = `/css/${theme}.css`
      link.setAttribute('data-markdown-theme', theme)

      document.head.appendChild(link)

      // Wait for theme to load
      await new Promise<void>((resolve) => {
        link.onload = () => resolve()
        link.onerror = () => resolve() // Don't fail on theme load errors
      })
    } catch (e) {
      console.error('Failed to load markdown theme:', e)
    }
  }

  function updateConfigState(config: ConfigChanged): void {
    const previousUITheme = state.interface.ui_theme
    const previousMarkdownTheme = state.interface.markdown_render_theme

    state.notesDirectory = config.notes_directory
    state.globalShortcut = config.global_shortcut
    state.general = config.general
    state.interface = config.interface
    state.editor = config.editor
    state.shortcuts = config.shortcuts
    state.preferences = config.preferences

    // Apply theme changes automatically when config updates
    if (state.isThemeInitialized) {
      if (config.interface.ui_theme !== previousUITheme) {
        loadTheme(config.interface.ui_theme)
      }
      if (config.interface.markdown_render_theme !== previousMarkdownTheme) {
        loadMarkdownTheme(config.interface.markdown_render_theme)
      }
    }
  }

  async function initialize(): Promise<void> {
    if (state.isInitialized) {
      return
    }

    state.isLoading = true
    state.error = null

    try {
      // Get initial config values
      const [
        generalConfig,
        interfaceConfig,
        editorConfig,
        shortcutsConfig,
        preferencesConfig,
      ] = await Promise.all([
        configService.getGeneralConfig(),
        configService.getInterfaceConfig(),
        configService.getEditorConfig(),
        configService.getShortcutsConfig(),
        configService.getPreferencesConfig(),
      ])

      state.general = generalConfig
      state.interface = interfaceConfig
      state.editor = editorConfig
      state.shortcuts = shortcutsConfig
      state.preferences = preferencesConfig

      // Initialize themes
      applyInterfaceConfig(interfaceConfig)
      await loadMarkdownTheme(interfaceConfig.markdown_render_theme)
      state.isThemeInitialized = true

      // Listen for config changes
      unlistenConfigChanged = await listen<ConfigChanged>(
        'config-updated',
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
      const [
        generalConfig,
        interfaceConfig,
        editorConfig,
        shortcutsConfig,
        preferencesConfig,
      ] = await Promise.all([
        configService.getGeneralConfig(),
        configService.getInterfaceConfig(),
        configService.getEditorConfig(),
        configService.getShortcutsConfig(),
        configService.getPreferencesConfig(),
      ])

      state.general = generalConfig
      state.interface = interfaceConfig
      state.editor = editorConfig
      state.shortcuts = shortcutsConfig
      state.preferences = preferencesConfig

      // Reapply themes
      if (state.isThemeInitialized) {
        applyInterfaceConfig(interfaceConfig)
        await loadMarkdownTheme(interfaceConfig.markdown_render_theme)
      }
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

    // Remove theme link
    const existingLink = document.head.querySelector(
      'link[data-markdown-theme]'
    )
    if (existingLink) {
      existingLink.remove()
    }

    state.isInitialized = false
    state.isThemeInitialized = false
  }

  return {
    // Reactive getters following existing manager patterns
    get notesDirectory() {
      return state.notesDirectory
    },

    get globalShortcut() {
      return state.globalShortcut
    },

    get general() {
      return state.general
    },

    get interface() {
      return state.interface
    },

    get editor() {
      return state.editor
    },

    get shortcuts() {
      return state.shortcuts
    },

    get preferences() {
      return state.preferences
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

    get isThemeInitialized() {
      return state.isThemeInitialized
    },

    get currentUITheme() {
      return state.interface.ui_theme
    },

    get currentMarkdownTheme() {
      return state.interface.markdown_render_theme
    },

    // Actions
    initialize,
    cleanup,
    forceRefresh,
    loadTheme,
    loadMarkdownTheme,
  }
}
