/**
 * Core Layer - Theme Manager
 * Markdown theme loading and management with automatic config change detection.
 * Handles DOM manipulation for theme CSS files and provides reactive theme state.
 */

import type { ConfigStateManager } from './configStateManager.svelte'

interface ThemeState {
  currentTheme: string
  isInitialized: boolean
  isLoading: boolean
  error: string | null
}

export interface ThemeManager {
  readonly currentTheme: string
  readonly isInitialized: boolean
  readonly isLoading: boolean
  readonly error: string | null
  initialize(configStateManager: ConfigStateManager): Promise<void>
  loadTheme(theme: string): Promise<void>
  cleanup(): void
  getThemeInitializer(): (element: HTMLElement) => { destroy(): void }
}

export function createThemeManager(): ThemeManager {
  const state = $state<ThemeState>({
    currentTheme: 'dark_dimmed',
    isInitialized: false,
    isLoading: false,
    error: null,
  })

  let configStateManager: ConfigStateManager | null = null
  let themeWatcher: (() => void) | null = null

  async function loadTheme(theme: string): Promise<void> {
    state.isLoading = true
    state.error = null

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
        link.onload = () => {
          resolve()
        }
        link.onerror = () => {
          resolve() // Don't fail on theme load errors
        }
      })

      state.currentTheme = theme
    } catch (e) {
      const error = `Failed to load theme: ${e}`
      state.error = error
      console.error('Failed to load theme:', e)
    } finally {
      state.isLoading = false
    }
  }

  async function initializeTheme(): Promise<void> {
    if (!configStateManager) {
      return
    }

    try {
      const theme = configStateManager.markdownTheme

      if (theme !== state.currentTheme || !state.isInitialized) {
        await loadTheme(theme)
        state.isInitialized = true
      }
    } catch (e) {
      console.error('Failed to initialize theme:', e)
      // Fallback to default theme
      if (state.currentTheme !== 'dark_dimmed' || !state.isInitialized) {
        await loadTheme('dark_dimmed')
        state.isInitialized = true
      }
    }
  }

  async function initialize(configManager: ConfigStateManager): Promise<void> {
    configStateManager = configManager

    // Initial theme load
    await initializeTheme()

    // Watch for config changes using $effect
    if (themeWatcher) {
      themeWatcher()
    }

    // Create reactive watcher for theme changes
    themeWatcher = $effect.root(() => {
      $effect(() => {
        if (configStateManager && configStateManager.isInitialized) {
          const theme = configStateManager.markdownTheme
          if (theme !== state.currentTheme && state.isInitialized) {
            loadTheme(theme)
          }
        }
      })

      return () => {
        // Cleanup when effect is destroyed
      }
    })
  }

  function cleanup(): void {
    if (themeWatcher) {
      themeWatcher()
      themeWatcher = null
    }

    // Remove theme link
    const existingLink = document.head.querySelector(
      'link[data-markdown-theme]'
    )
    if (existingLink) {
      existingLink.remove()
    }

    state.isInitialized = false
    configStateManager = null
  }

  function getThemeInitializer(): (element: HTMLElement) => {
    destroy(): void
  } {
    return function themeInitializer(_element: HTMLElement) {
      // Initialize theme when element is mounted
      if (configStateManager) {
        initializeTheme()
      }

      return {
        destroy() {
          // Cleanup if needed when element is destroyed
        },
      }
    }
  }

  return {
    // Reactive getters
    get currentTheme() {
      return state.currentTheme
    },

    get isInitialized() {
      return state.isInitialized
    },

    get isLoading() {
      return state.isLoading
    },

    get error() {
      return state.error
    },

    // Actions
    initialize,
    loadTheme,
    cleanup,
    getThemeInitializer,
  }
}
