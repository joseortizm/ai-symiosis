/**
 * Service Layer - Config Service
 * Application configuration settings and the settings pane state.
 * Handles configuration loading, saving, and reactive settings panel visibility.
 */

import { invoke } from '@tauri-apps/api/core'

// Configuration type definitions
export interface GeneralConfig {
  // Future extensible core settings
  [key: string]: unknown
}

export interface InterfaceConfig {
  ui_theme: string
  font_family: string
  font_size: number
  editor_font_family: string
  editor_font_size: number
  markdown_render_theme: string
  md_render_code_theme: string
  always_on_top: boolean
}

export interface EditorConfig {
  mode: string
  theme: string
  word_wrap: boolean
  tab_size: number
  expand_tabs: boolean
  show_line_numbers: boolean
}

export interface ShortcutsConfig {
  create_note: string
  rename_note: string
  delete_note: string
  save_and_exit: string
  open_external: string
  open_folder: string
  refresh_cache: string
  scroll_up: string
  scroll_down: string
  vim_up: string
  vim_down: string
  navigate_previous: string
  navigate_next: string
  navigate_code_previous: string
  navigate_code_next: string
  copy_current_section: string
  open_settings: string
}

export interface PreferencesConfig {
  max_search_results: number
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
  clearError(): void
  openPane(): Promise<void>
  closePane(): void
  getGeneralConfig(): Promise<GeneralConfig>
  getInterfaceConfig(): Promise<InterfaceConfig>
  getEditorConfig(): Promise<EditorConfig>
  getShortcutsConfig(): Promise<ShortcutsConfig>
  getPreferencesConfig(): Promise<PreferencesConfig>
  getAvailableThemes(): Promise<{
    ui_themes: string[]
    markdown_themes: string[]
  }>
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

  async function getGeneralConfig(): Promise<GeneralConfig> {
    try {
      const result = await invoke<GeneralConfig>('get_general_config')
      return result
    } catch (e) {
      console.error('Failed to get general config:', e)
      return {}
    }
  }

  async function getInterfaceConfig(): Promise<InterfaceConfig> {
    try {
      const result = await invoke<InterfaceConfig>('get_interface_config')
      return result
    } catch (e) {
      console.error('Failed to get interface config:', e)
      return {
        ui_theme: 'gruvbox-dark',
        font_family: 'Inter, sans-serif',
        font_size: 14,
        editor_font_family: 'JetBrains Mono, Consolas, monospace',
        editor_font_size: 14,
        markdown_render_theme: 'dark_dimmed',
        md_render_code_theme: 'gruvbox-dark-medium',
        always_on_top: false,
      }
    }
  }

  async function getEditorConfig(): Promise<EditorConfig> {
    try {
      const result = await invoke<EditorConfig>('get_editor_config')
      return result
    } catch (e) {
      console.error('Failed to get editor config:', e)
      return {
        mode: 'basic',
        theme: 'gruvbox-dark',
        word_wrap: true,
        tab_size: 2,
        expand_tabs: true,
        show_line_numbers: true,
      }
    }
  }

  async function getShortcutsConfig(): Promise<ShortcutsConfig> {
    try {
      const result = await invoke<ShortcutsConfig>('get_shortcuts_config')
      return result
    } catch (e) {
      console.error('Failed to get shortcuts config:', e)
      return {
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
        navigate_code_previous: 'Ctrl+h',
        navigate_code_next: 'Ctrl+l',
        copy_current_section: 'Ctrl+y',
        open_settings: 'Meta+,',
      }
    }
  }

  async function getPreferencesConfig(): Promise<PreferencesConfig> {
    try {
      const result = await invoke<PreferencesConfig>('get_preferences_config')
      return result
    } catch (e) {
      console.error('Failed to get preferences config:', e)
      return {
        max_search_results: 100,
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
    clearError,
    openPane,
    closePane,
    getGeneralConfig,
    getInterfaceConfig,
    getEditorConfig,
    getShortcutsConfig,
    getPreferencesConfig,

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

    async getAvailableThemes(): Promise<{
      ui_themes: string[]
      markdown_themes: string[]
    }> {
      try {
        const result = await invoke<{
          ui_themes: string[]
          markdown_themes: string[]
        }>('scan_available_themes')
        return result
      } catch (error) {
        console.error('Failed to scan available themes:', error)
        return {
          ui_themes: ['gruvbox-dark', 'one-dark'],
          markdown_themes: [
            'light',
            'dark',
            'dark_dimmed',
            'auto',
            'modern_dark',
            'article',
            'gruvbox',
            'dark_high_contrast',
          ],
        }
      }
    },

    get lastSaved(): number {
      return state.lastSaved
    },
  }
}

export const configService = createConfigService()
