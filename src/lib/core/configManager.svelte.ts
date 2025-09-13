import { listen } from '@tauri-apps/api/event'
import { configService } from '../services/configService.svelte'
import type {
  GeneralConfig,
  InterfaceConfig,
  EditorConfig,
  ShortcutsConfig,
  PreferencesConfig,
} from '../types/config'

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

export interface ConfigManager {
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
  readonly currentCodeTheme: string
  initialize(): Promise<void>
  cleanup(): void
  forceRefresh(): Promise<void>
  loadTheme(theme: string): Promise<void>
  loadMarkdownTheme(theme: string): Promise<void>
  loadHighlightJSTheme(theme: string): Promise<void>
}

export function createConfigManager(): ConfigManager {
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
      md_render_code_theme: 'gruvbox-dark-medium',
      always_on_top: false,
    },
    editor: {
      mode: 'basic',
      theme: 'gruvbox-dark',
      word_wrap: true,
      tab_size: 2,
      expand_tabs: true,
      show_line_numbers: true,
    },
    shortcuts: {
      create_note: 'Ctrl+Enter',
      rename_note: 'Ctrl+m',
      delete_note: 'Ctrl+x',
      edit_note: 'Enter',
      save_and_exit: 'Ctrl+s',
      open_external: 'Ctrl+o',
      open_folder: 'Ctrl+f',
      refresh_cache: 'Ctrl+r',
      scroll_up: 'Ctrl+u',
      scroll_down: 'Ctrl+d',
      up: 'Ctrl+k',
      down: 'Ctrl+j',
      navigate_previous: 'Ctrl+p',
      navigate_next: 'Ctrl+n',
      navigate_code_previous: 'Ctrl+Alt+h',
      navigate_code_next: 'Ctrl+Alt+l',
      navigate_link_previous: 'Ctrl+h',
      navigate_link_next: 'Ctrl+l',
      copy_current_section: 'Ctrl+y',
      open_settings: 'Meta+,',
      version_explorer: 'Ctrl+/',
      recently_deleted: 'Ctrl+.',
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

  let validUIThemes: string[] = []

  async function fetchAvailableThemes(): Promise<void> {
    try {
      const themes = await configService.getAvailableThemes()
      validUIThemes = themes.ui_themes
    } catch (error) {
      console.warn('Failed to fetch available themes, using defaults:', error)
      validUIThemes = ['gruvbox-dark', 'one-dark']
    }
  }

  function applyInterfaceConfig(interfaceConfig: InterfaceConfig): void {
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

  function updateConfigState(config: ConfigChanged): void {
    const previousUITheme = state.interface.ui_theme
    const previousMarkdownTheme = state.interface.markdown_render_theme
    const previousCodeTheme = state.interface.md_render_code_theme

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
        configService.loadTheme(config.interface.ui_theme, validUIThemes)
      }
      if (config.interface.markdown_render_theme !== previousMarkdownTheme) {
        configService.loadMarkdownTheme(config.interface.markdown_render_theme)
      }
      if (config.interface.md_render_code_theme !== previousCodeTheme) {
        configService.loadHighlightJSTheme(
          config.interface.md_render_code_theme
        )
      }
    }
  }

  async function loadAllConfigs(): Promise<{
    generalConfig: GeneralConfig
    interfaceConfig: InterfaceConfig
    editorConfig: EditorConfig
    shortcutsConfig: ShortcutsConfig
    preferencesConfig: PreferencesConfig
  }> {
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

    return {
      generalConfig,
      interfaceConfig,
      editorConfig,
      shortcutsConfig,
      preferencesConfig,
    }
  }

  function updateStateWithConfigs(configs: {
    generalConfig: GeneralConfig
    interfaceConfig: InterfaceConfig
    editorConfig: EditorConfig
    shortcutsConfig: ShortcutsConfig
    preferencesConfig: PreferencesConfig
  }): void {
    state.general = configs.generalConfig
    state.interface = configs.interfaceConfig
    state.editor = configs.editorConfig
    state.shortcuts = configs.shortcutsConfig
    state.preferences = configs.preferencesConfig
  }

  async function setupThemes(interfaceConfig: InterfaceConfig): Promise<void> {
    applyInterfaceConfig(interfaceConfig)
    await configService.loadTheme(interfaceConfig.ui_theme, validUIThemes)
    await configService.loadMarkdownTheme(interfaceConfig.markdown_render_theme)
    await configService.loadHighlightJSTheme(
      interfaceConfig.md_render_code_theme
    )
    state.isThemeInitialized = true
  }

  async function setupConfigListener(): Promise<void> {
    unlistenConfigChanged = await listen<ConfigChanged>(
      'config-updated',
      (event) => {
        updateConfigState(event.payload)
      }
    )
  }

  async function initialize(): Promise<void> {
    if (state.isInitialized) {
      return
    }

    state.isLoading = true
    state.error = null

    try {
      const configs = await loadAllConfigs()
      await fetchAvailableThemes()

      updateStateWithConfigs(configs)
      await setupThemes(configs.interfaceConfig)
      await setupConfigListener()

      state.isInitialized = true
    } catch (e) {
      const error = `Failed to initialize config state: ${e}`
      state.error = error
      console.error('Failed to initialize config state:', e)
    } finally {
      state.isLoading = false
    }
  }

  async function refreshConfigsAndThemes(): Promise<void> {
    await configService.refreshCache()

    const configs = await loadAllConfigs()
    updateStateWithConfigs(configs)

    if (state.isThemeInitialized) {
      await setupThemes(configs.interfaceConfig)
    }
  }

  async function forceRefresh(): Promise<void> {
    state.isLoading = true
    state.error = null

    try {
      await refreshConfigsAndThemes()
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

    const highlightThemeLink = document.head.querySelector(
      'link[data-highlight-theme]'
    )
    if (highlightThemeLink) {
      highlightThemeLink.remove()
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

    get currentCodeTheme() {
      return state.interface.md_render_code_theme
    },

    initialize,
    cleanup,
    forceRefresh,
    loadTheme: (theme: string) => configService.loadTheme(theme, validUIThemes),
    loadMarkdownTheme: configService.loadMarkdownTheme,
    loadHighlightJSTheme: configService.loadHighlightJSTheme,
  }
}
