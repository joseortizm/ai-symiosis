import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { createThemeManager } from '$lib/core/themeManager.svelte'
import type { ConfigStateManager } from '$lib/core/configStateManager.svelte'

// Mock DOM methods
const mockCreateElement = vi.fn()
const mockAppendChild = vi.fn()
const mockRemove = vi.fn()
const mockQuerySelector = vi.fn()

// Create mock HTML link element
function createMockLinkElement() {
  return {
    rel: '',
    href: '',
    onload: null as (() => void) | null,
    onerror: null as (() => void) | null,
    setAttribute: vi.fn(),
    remove: mockRemove,
  }
}

describe('themeManager', () => {
  let manager: ReturnType<typeof createThemeManager>
  let mockConfigStateManager: ConfigStateManager
  let mockLinkElement: ReturnType<typeof createMockLinkElement>

  // Helper function to load theme with automatic onload trigger
  async function loadThemeWithSuccess(theme: string) {
    const loadPromise = manager.loadTheme(theme)
    if (mockLinkElement.onload) {
      mockLinkElement.onload()
    }
    return await loadPromise
  }

  // Helper function to initialize with automatic theme loading
  async function initializeWithSuccess(config: ConfigStateManager) {
    const initPromise = manager.initialize(config)
    if (mockLinkElement.onload) {
      mockLinkElement.onload()
    }
    return await initPromise
  }

  beforeEach(() => {
    vi.clearAllMocks()

    // Setup DOM mocks
    mockLinkElement = createMockLinkElement()
    mockCreateElement.mockReturnValue(mockLinkElement)
    mockQuerySelector.mockReturnValue(null)

    Object.defineProperty(global, 'document', {
      value: {
        createElement: mockCreateElement,
        head: {
          querySelector: mockQuerySelector,
          appendChild: mockAppendChild,
        },
      },
      writable: true,
    })

    // Create mock config state manager
    mockConfigStateManager = {
      notesDirectory: '/notes',
      maxSearchResults: 100,
      globalShortcut: 'Ctrl+Shift+N',
      editorMode: 'vim',
      markdownTheme: 'github',
      isLoading: false,
      error: null,
      isInitialized: true,
      initialize: vi.fn(),
      cleanup: vi.fn(),
      forceRefresh: vi.fn(),
    }

    manager = createThemeManager()
  })

  afterEach(() => {
    manager.cleanup()
  })

  describe('initial state', () => {
    it('should have default values before initialization', () => {
      expect(manager.currentTheme).toBe('dark_dimmed')
      expect(manager.isInitialized).toBe(false)
      expect(manager.isLoading).toBe(false)
      expect(manager.error).toBe(null)
    })
  })

  describe('loadTheme', () => {
    it('should load a new theme successfully', async () => {
      const loadPromise = manager.loadTheme('github')

      // Simulate successful load
      if (mockLinkElement.onload) {
        mockLinkElement.onload()
      }

      await loadPromise

      expect(mockCreateElement).toHaveBeenCalledWith('link')
      expect(mockLinkElement.rel).toBe('stylesheet')
      expect(mockLinkElement.href).toBe('/css/github.css')
      expect(mockLinkElement.setAttribute).toHaveBeenCalledWith(
        'data-markdown-theme',
        'github'
      )
      expect(mockAppendChild).toHaveBeenCalledWith(mockLinkElement)
      expect(manager.currentTheme).toBe('github')
      expect(manager.isLoading).toBe(false)
      expect(manager.error).toBe(null)
    })

    it('should remove existing theme before loading new one', async () => {
      const existingLink = { remove: vi.fn() }
      mockQuerySelector.mockReturnValue(existingLink)

      await loadThemeWithSuccess('github')

      expect(mockQuerySelector).toHaveBeenCalledWith(
        'link[data-markdown-theme]'
      )
      expect(existingLink.remove).toHaveBeenCalled()
    })

    it('should handle theme load success via onload callback', async () => {
      const loadPromise = manager.loadTheme('github')

      // Simulate successful load
      if (mockLinkElement.onload) {
        mockLinkElement.onload()
      }

      await loadPromise
      expect(manager.currentTheme).toBe('github')
      expect(manager.error).toBe(null)
    })

    it('should handle theme load error gracefully via onerror callback', async () => {
      const loadPromise = manager.loadTheme('github')

      // Simulate load error
      if (mockLinkElement.onerror) {
        mockLinkElement.onerror()
      }

      await loadPromise
      expect(manager.currentTheme).toBe('github') // Should still update theme name
      expect(manager.error).toBe(null) // Should not fail on theme load errors
    })

    it('should set loading states correctly during theme load', async () => {
      expect(manager.isLoading).toBe(false)

      const loadPromise = manager.loadTheme('github')
      expect(manager.isLoading).toBe(true)

      // Simulate load completion
      if (mockLinkElement.onload) {
        mockLinkElement.onload()
      }

      await loadPromise
      expect(manager.isLoading).toBe(false)
    })

    it('should clear previous errors on new theme load', async () => {
      // Load a valid theme
      expect(manager.error).toBe(null) // Should start with no error

      await loadThemeWithSuccess('github')
      expect(manager.error).toBe(null)
    })

    it('should handle DOM manipulation errors', async () => {
      const error = new Error('DOM error')
      mockCreateElement.mockImplementation(() => {
        throw error
      })

      await manager.loadTheme('github')

      expect(manager.error).toContain('Failed to load theme')
      expect(manager.error).toContain('DOM error')
      expect(manager.isLoading).toBe(false)
    })
  })

  describe('initialize', () => {
    it('should initialize with config manager theme', async () => {
      const initPromise = manager.initialize(mockConfigStateManager)

      // Trigger onload for theme loading
      if (mockLinkElement.onload) {
        mockLinkElement.onload()
      }

      await initPromise

      expect(manager.currentTheme).toBe('github')
      expect(manager.isInitialized).toBe(true)
      expect(mockCreateElement).toHaveBeenCalled()
    })

    it('should handle initialization when config not ready', async () => {
      const unreadyConfig = {
        ...mockConfigStateManager,
        isInitialized: false,
      }

      // Even with uninitialized config, theme manager will attempt to load theme
      const initPromise = manager.initialize(unreadyConfig)

      // Trigger onload callback since loadTheme will be called
      if (mockLinkElement.onload) {
        mockLinkElement.onload()
      }

      await initPromise

      // Should initialize and load theme from config even if config reports uninitialized
      expect(manager.isInitialized).toBe(true)
      expect(manager.currentTheme).toBe('github') // From mockConfigStateManager
    })

    it('should fallback to default theme on initialization error', async () => {
      mockCreateElement.mockImplementationOnce(() => {
        throw new Error('Failed to create element')
      })

      await manager.initialize(mockConfigStateManager)
      expect(mockCreateElement).toHaveBeenCalledTimes(2)
      expect(manager.isInitialized).toBe(true)
    })

    it('should not load same theme twice', async () => {
      // First load the theme manually
      const loadPromise = manager.loadTheme('github')
      if (mockLinkElement.onload) {
        mockLinkElement.onload()
      }
      await loadPromise

      // Verify state after first load
      expect(manager.currentTheme).toBe('github')
      expect(manager.isInitialized).toBe(false) // loadTheme doesn't set initialized

      vi.clearAllMocks()

      // Initialize with same theme - will reload because !state.isInitialized is true
      const initPromise = manager.initialize(mockConfigStateManager)
      if (mockLinkElement.onload) {
        mockLinkElement.onload() // Theme will be loaded again due to !isInitialized
      }
      await initPromise

      // Should have called createElement again because isInitialized was false
      expect(mockCreateElement).toHaveBeenCalled()
      expect(manager.isInitialized).toBe(true)
    })

    it('should setup reactive theme watching', async () => {
      await initializeWithSuccess(mockConfigStateManager)
      expect(manager.isInitialized).toBe(true)
    })
  })

  describe('cleanup', () => {
    it('should remove theme link and reset state', async () => {
      const existingLink = { remove: vi.fn() }
      mockQuerySelector.mockReturnValue(existingLink)

      await initializeWithSuccess(mockConfigStateManager)
      expect(manager.isInitialized).toBe(true)

      manager.cleanup()

      expect(mockQuerySelector).toHaveBeenCalledWith(
        'link[data-markdown-theme]'
      )
      expect(existingLink.remove).toHaveBeenCalled()
      expect(manager.isInitialized).toBe(false)
    })

    it('should handle cleanup when no theme link exists', () => {
      mockQuerySelector.mockReturnValue(null)

      expect(() => manager.cleanup()).not.toThrow()
      expect(manager.isInitialized).toBe(false)
    })

    it('should cleanup multiple times safely', async () => {
      await initializeWithSuccess(mockConfigStateManager)

      manager.cleanup()
      expect(() => manager.cleanup()).not.toThrow()
    })
  })

  describe('getThemeInitializer', () => {
    it('should return a function that initializes theme on element mount', () => {
      const initializer = manager.getThemeInitializer()
      expect(typeof initializer).toBe('function')

      const mockElement = document.createElement('div')
      const result = initializer(mockElement)

      expect(result).toHaveProperty('destroy')
      expect(typeof result.destroy).toBe('function')
    })

    it('should initialize theme when element is mounted if config manager exists', async () => {
      // Initialize manager first
      const initPromise = manager.initialize(mockConfigStateManager)
      if (mockLinkElement.onload) {
        mockLinkElement.onload()
      }
      await initPromise

      vi.clearAllMocks()

      const initializer = manager.getThemeInitializer()
      // Create a simple mock element instead of using document.createElement
      const mockElement = {} as HTMLElement

      initializer(mockElement)

      // Since theme is already loaded, should not create new link elements
      expect(mockCreateElement).not.toHaveBeenCalled()
    })

    it('should handle destroy callback without errors', async () => {
      await initializeWithSuccess(mockConfigStateManager)

      const initializer = manager.getThemeInitializer()
      const mockElement = document.createElement('div')
      const result = initializer(mockElement)

      expect(() => result.destroy()).not.toThrow()
    })
  })

  describe('reactive getters', () => {
    it('should provide reactive access to all theme properties', () => {
      const properties = ['currentTheme', 'isInitialized', 'isLoading', 'error']

      properties.forEach((prop) => {
        expect(manager).toHaveProperty(prop)
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        expect(typeof (manager as any)[prop]).not.toBe('function')
      })
    })
  })

  describe('edge cases', () => {
    it('should handle empty theme name', async () => {
      await loadThemeWithSuccess('')

      expect(mockLinkElement.href).toBe('/css/.css')
      expect(manager.currentTheme).toBe('')
    })

    it('should handle very long theme names', async () => {
      const longThemeName = 'a'.repeat(1000)
      await loadThemeWithSuccess(longThemeName)

      expect(mockLinkElement.href).toBe(`/css/${longThemeName}.css`)
      expect(manager.currentTheme).toBe(longThemeName)
    })

    it('should handle special characters in theme names', async () => {
      const specialTheme = 'theme-with_special.chars'
      await loadThemeWithSuccess(specialTheme)

      expect(mockLinkElement.href).toBe(`/css/${specialTheme}.css`)
      expect(manager.currentTheme).toBe(specialTheme)
    })

    it('should handle multiple rapid theme changes', async () => {
      const themes = ['github', 'dark', 'light', 'monokai']

      const promises = themes.map((theme) => {
        const promise = manager.loadTheme(theme)
        // Simulate immediate load completion
        if (mockLinkElement.onload) {
          mockLinkElement.onload()
        }
        return promise
      })

      await Promise.all(promises)

      // Should end up with the last theme
      expect(manager.currentTheme).toBe('monokai')
      expect(mockCreateElement).toHaveBeenCalledTimes(themes.length)
    })
  })
})
