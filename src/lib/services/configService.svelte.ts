/**
 * Service Layer - Config Service
 * Application configuration settings and the settings pane state.
 * Handles configuration loading, saving, and reactive settings panel visibility.
 */

import { invoke } from '@tauri-apps/api/core'

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
  clearError(): void
  openPane(): Promise<void>
  closePane(): void
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

  return {
    open,
    close,
    save,
    updateContent,
    exists,
    refreshCache,
    getMarkdownTheme,
    clearError,
    openPane,
    closePane,

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
