/**
 * Core Layer - Theme Manager
 * Full UI theming system with CSS custom properties and markdown theme loading.
 * Handles dynamic theme switching, font configuration, and provides reactive theme state.
 */

import type { ConfigStateManager } from './configStateManager.svelte'
import type { ThemeConfig } from '../services/configService.svelte'

interface ThemeState {
  currentTheme: string
  currentMarkdownTheme: string
  isInitialized: boolean
  isLoading: boolean
  error: string | null
}

export interface ThemeManager {
  readonly currentTheme: string
  readonly currentMarkdownTheme: string
  readonly isInitialized: boolean
  readonly isLoading: boolean
  readonly error: string | null
  initialize(configStateManager: ConfigStateManager): Promise<void>
  loadTheme(theme: string): Promise<void>
  loadMarkdownTheme(theme: string): Promise<void>
  applyThemeConfig(themeConfig: ThemeConfig): void
  cleanup(): void
  getThemeInitializer(): (element?: HTMLElement) => { destroy(): void }
}

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

export function createThemeManager(): ThemeManager {
  const state = $state<ThemeState>({
    currentTheme: 'gruvbox-dark',
    currentMarkdownTheme: 'dark_dimmed',
    isInitialized: false,
    isLoading: false,
    error: null,
  })

  let configStateManager: ConfigStateManager | null = null
  let themeWatcher: (() => void) | null = null

  function applyThemeConfig(themeConfig: ThemeConfig): void {
    // Apply theme colors
    if (
      themeConfig.name &&
      THEME_DEFINITIONS[themeConfig.name as keyof typeof THEME_DEFINITIONS]
    ) {
      const themeColors =
        THEME_DEFINITIONS[themeConfig.name as keyof typeof THEME_DEFINITIONS]
      const root = document.documentElement

      Object.entries(themeColors).forEach(([property, value]) => {
        root.style.setProperty(property, value)
      })

      state.currentTheme = themeConfig.name
    }

    // Apply font configuration
    const root = document.documentElement
    root.style.setProperty('--theme-font-family', themeConfig.font_family)
    root.style.setProperty('--theme-font-size', `${themeConfig.font_size}px`)
    root.style.setProperty(
      '--theme-editor-font-family',
      themeConfig.editor_font_family
    )
    root.style.setProperty(
      '--theme-editor-font-size',
      `${themeConfig.editor_font_size}px`
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

      state.currentTheme = theme
    } else {
      console.warn(`Unknown theme: ${theme}. Using default.`)
    }
  }

  async function loadMarkdownTheme(theme: string): Promise<void> {
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

      state.currentMarkdownTheme = theme
    } catch (e) {
      const error = `Failed to load markdown theme: ${e}`
      state.error = error
      console.error('Failed to load markdown theme:', e)
    } finally {
      state.isLoading = false
    }
  }

  async function initializeTheme(): Promise<void> {
    if (!configStateManager) {
      return
    }

    try {
      // Apply full theme configuration from config state
      const themeConfig = configStateManager.theme
      const markdownTheme = configStateManager.markdownTheme

      // Apply UI theme and fonts
      applyThemeConfig(themeConfig)

      // Load markdown theme CSS
      if (
        markdownTheme !== state.currentMarkdownTheme ||
        !state.isInitialized
      ) {
        await loadMarkdownTheme(markdownTheme)
      }

      state.isInitialized = true
    } catch (e) {
      console.error('Failed to initialize theme:', e)
      // Fallback to default theme
      if (!state.isInitialized) {
        await loadMarkdownTheme('dark_dimmed')
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
          const themeConfig = configStateManager.theme
          const markdownTheme = configStateManager.markdownTheme

          // Check if UI theme changed
          if (themeConfig.name !== state.currentTheme && state.isInitialized) {
            applyThemeConfig(themeConfig)
          }

          // Check if markdown theme changed
          if (
            markdownTheme !== state.currentMarkdownTheme &&
            state.isInitialized
          ) {
            loadMarkdownTheme(markdownTheme)
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

  function getThemeInitializer() {
    return function themeInitializer(_element?: HTMLElement) {
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

    get currentMarkdownTheme() {
      return state.currentMarkdownTheme
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
    loadMarkdownTheme,
    applyThemeConfig,
    cleanup,
    getThemeInitializer,
  }
}
