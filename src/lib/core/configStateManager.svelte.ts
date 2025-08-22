/**
 * Core Layer - Config State Manager
 * Reactive configuration state management with real-time updates from backend.
 * Listens to Tauri config-changed events and provides reactive access to config values.
 * Also handles theme management for UI and markdown rendering.
 */

import { listen } from '@tauri-apps/api/event'
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

  // Valid UI themes that have corresponding CSS files
  const VALID_UI_THEMES = ['gruvbox-dark', 'one-dark']

  function applyInterfaceConfig(interfaceConfig: InterfaceConfig): void {
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
    try {
      // Remove existing UI theme
      const existingLink = document.head.querySelector('link[data-ui-theme]')
      if (existingLink) {
        existingLink.remove()
      }

      // Validate theme
      if (!VALID_UI_THEMES.includes(theme)) {
        console.warn(
          `Unknown UI theme: ${theme}. Using gruvbox-dark as default.`
        )
        theme = 'gruvbox-dark'
      }

      // Create new UI theme link
      const link = document.createElement('link')
      link.rel = 'stylesheet'
      link.href = `/css/ui-themes/ui-${theme}.css`
      link.setAttribute('data-ui-theme', theme)

      document.head.appendChild(link)

      // Wait for theme to load
      await new Promise<void>((resolve) => {
        link.onload = () => resolve()
        link.onerror = () => resolve() // Don't fail on theme load errors
      })
    } catch (e) {
      console.error('Failed to load UI theme:', e)
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
      link.href = `/css/md_render_themes/${theme}.css`
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

    // Apply interface config changes automatically when config updates
    if (state.isThemeInitialized) {
      // Always apply interface config (fonts, etc.) when config changes
      applyInterfaceConfig(config.interface)

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
      await loadTheme(interfaceConfig.ui_theme)
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
      await configService.refreshCache()

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
        await loadTheme(interfaceConfig.ui_theme)
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

    // Remove theme links
    const markdownThemeLink = document.head.querySelector(
      'link[data-markdown-theme]'
    )
    if (markdownThemeLink) {
      markdownThemeLink.remove()
    }

    const uiThemeLink = document.head.querySelector('link[data-ui-theme]')
    if (uiThemeLink) {
      uiThemeLink.remove()
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
