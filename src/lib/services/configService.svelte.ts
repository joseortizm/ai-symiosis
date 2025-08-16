/**
 * Service Layer - Config Service
 * Application configuration settings and the settings pane state.
 * Handles configuration loading, saving, and reactive settings panel visibility.
 */

import { invoke } from '@tauri-apps/api/core'

// Configuration type definitions
export interface ThemeConfig {
  name: string
  font_family: string
  font_size: number
  editor_font_family: string
  editor_font_size: number
}

export interface SearchInputShortcuts {
  create_note: string
  rename_note: string
  open_external: string
  open_folder: string
  refresh_cache: string
  delete_note: string
  scroll_up: string
  scroll_down: string
  vim_up: string
  vim_down: string
  navigate_previous: string
  navigate_next: string
  open_settings: string
}

export interface EditModeShortcuts {
  save_and_exit: string
  open_settings: string
}

export interface ShortcutConfig {
  search_input: SearchInputShortcuts
  edit_mode: EditModeShortcuts
}

export interface EditorConfig {
  mode: string
  theme: string
  word_wrap: boolean
  tab_size: number
  line_height: number
  show_line_numbers: boolean
  markdown_theme: string
  shortcuts: {
    save: string
    fold: string
    unfold: string
    fold_all: string
    unfold_all: string
  }
}

export interface WindowConfig {
  default_width: number
  default_height: number
  center_on_startup: boolean
  remember_size: boolean
  remember_position: boolean
  always_on_top: boolean
}

interface ConfigServiceState {
  content: string
  isVisible: boolean
  isLoading: boolean
  error: string | null
  lastSaved: number // Timestamp to trigger reactive updates
}

export interface ConfigService {
  content: string
  readonly isVisible: boolean
  readonly isLoading: boolean
  readonly error: string | null
  readonly lastSaved: number
  open(): Promise<void>
  close(): void
  save(): Promise<{ success: boolean; error?: string }>
  updateContent(content: string): void
  exists(): Promise<boolean>
  refreshCache(): Promise<void>
  getMarkdownTheme(): Promise<string>
  getEditorMode(): Promise<string>
  clearError(): void
  openPane(): Promise<void>
  closePane(): void
  getThemeConfig(): Promise<ThemeConfig>
  getShortcutConfig(): Promise<ShortcutConfig>
  getEditorConfig(): Promise<EditorConfig>
  getWindowConfig(): Promise<WindowConfig>
}

export function createConfigService(): ConfigService {
  const state = $state<ConfigServiceState>({
    content: '',
    isVisible: false,
    isLoading: false,
    error: null,
    lastSaved: 0,
  })

  async function open(): Promise<void> {
    state.isLoading = true
    state.error = null

    try {
      const content = await invoke<string>('get_config_content')
      state.content = content
      state.isVisible = true
    } catch (e) {
      state.error = `Failed to load config: ${e}`
      console.error('Failed to load config:', e)
    } finally {
      state.isLoading = false
    }
  }

  function close(): void {
    state.isVisible = false
    state.content = ''
    state.error = null
  }

  async function save(): Promise<{ success: boolean; error?: string }> {
    state.isLoading = true
    state.error = null

    try {
      await invoke<void>('save_config_content', { content: state.content })
      await invoke<void>('refresh_cache')

      // Update timestamp to trigger reactive config reloads
      state.lastSaved = Date.now()

      close()

      return { success: true }
    } catch (e) {
      const error = `Failed to save config: ${e}`
      state.error = error
      console.error('Failed to save config:', e)
      return { success: false, error }
    } finally {
      state.isLoading = false
    }
  }

  function updateContent(content: string): void {
    state.content = content
  }

  async function exists(): Promise<boolean> {
    try {
      return await invoke<boolean>('config_exists')
    } catch (e) {
      console.error('Failed to check config existence:', e)
      return false
    }
  }

  async function refreshCache(): Promise<void> {
    try {
      await invoke<void>('refresh_cache')
    } catch (e) {
      console.error('Failed to refresh cache:', e)
      throw e
    }
  }

  async function getMarkdownTheme(): Promise<string> {
    try {
      return await invoke<string>('get_markdown_theme')
    } catch (e) {
      console.error('Failed to get markdown theme:', e)
      return 'dark_dimmed'
    }
  }

  async function getEditorMode(): Promise<string> {
    try {
      return await invoke<string>('get_editor_mode')
    } catch (e) {
      console.error('Failed to get editor mode:', e)
      return 'basic'
    }
  }

  function clearError(): void {
    state.error = null
  }

  // Pane management methods for direct use in +page.svelte
  async function openPane(): Promise<void> {
    await open()
  }

  function closePane(): void {
    close()
  }

  async function getThemeConfig(): Promise<ThemeConfig> {
    try {
      return await invoke<ThemeConfig>('get_theme_config')
    } catch (e) {
      console.error('Failed to get theme config:', e)
      // Return default theme config
      return {
        name: 'gruvbox-dark',
        font_family: 'Inter, sans-serif',
        font_size: 14,
        editor_font_family: 'JetBrains Mono, Consolas, monospace',
        editor_font_size: 14,
      }
    }
  }

  async function getShortcutConfig(): Promise<ShortcutConfig> {
    try {
      return await invoke<ShortcutConfig>('get_shortcut_config')
    } catch (e) {
      console.error('Failed to get shortcut config:', e)
      // Return default shortcut config
      return {
        search_input: {
          create_note: 'Ctrl+Enter',
          rename_note: 'Ctrl+m',
          open_external: 'Ctrl+o',
          open_folder: 'Ctrl+f',
          refresh_cache: 'Ctrl+r',
          delete_note: 'Ctrl+x',
          scroll_up: 'Ctrl+u',
          scroll_down: 'Ctrl+d',
          vim_up: 'Ctrl+k',
          vim_down: 'Ctrl+j',
          navigate_previous: 'Ctrl+p',
          navigate_next: 'Ctrl+n',
          open_settings: 'Meta+,',
        },
        edit_mode: {
          save_and_exit: 'Ctrl+s',
          open_settings: 'Meta+,',
        },
      }
    }
  }

  async function getEditorConfig(): Promise<EditorConfig> {
    try {
      return await invoke<EditorConfig>('get_editor_config')
    } catch (e) {
      console.error('Failed to get editor config:', e)
      // Return default editor config
      return {
        mode: 'basic',
        theme: 'gruvbox-dark',
        word_wrap: true,
        tab_size: 2,
        line_height: 1.5,
        show_line_numbers: true,
        markdown_theme: 'dark_dimmed',
        shortcuts: {
          save: 'Ctrl+s',
          fold: 'Ctrl+Shift+[',
          unfold: 'Ctrl+Shift+]',
          fold_all: 'Ctrl+Alt+[',
          unfold_all: 'Ctrl+Alt+]',
        },
      }
    }
  }

  async function getWindowConfig(): Promise<WindowConfig> {
    try {
      return await invoke<WindowConfig>('get_window_config')
    } catch (e) {
      console.error('Failed to get window config:', e)
      // Return default window config
      return {
        default_width: 1200,
        default_height: 800,
        center_on_startup: true,
        remember_size: true,
        remember_position: true,
        always_on_top: false,
      }
    }
  }

  return {
    open,
    close,
    save,
    updateContent,
    exists,
    refreshCache,
    getMarkdownTheme,
    getEditorMode,
    clearError,
    openPane,
    closePane,
    getThemeConfig,
    getShortcutConfig,
    getEditorConfig,
    getWindowConfig,

    // Reactive getters and setters (to support bind:value)
    get content(): string {
      return state.content
    },

    set content(value: string) {
      state.content = value
    },

    get isVisible(): boolean {
      return state.isVisible
    },

    get isLoading(): boolean {
      return state.isLoading
    },

    get error(): string | null {
      return state.error
    },

    get lastSaved(): number {
      return state.lastSaved
    },
  }
}

export const configService = createConfigService()
