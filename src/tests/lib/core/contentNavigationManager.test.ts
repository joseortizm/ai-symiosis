/**
 * Tests for Content Navigation Manager
 * Verifies navigation between search highlights and markdown headers
 */

import { describe, it, expect, vi, beforeEach } from 'vitest'
import { createContentNavigationManager } from '../../../lib/core/contentNavigationManager.svelte'

describe('ContentNavigationManager', () => {
  let mockNoteContentElement: HTMLElement
  let mockDeps: {
    focusManager: {
      noteContentElement: HTMLElement | null
    }
    searchManager: {
      query: string
      areHighlightsCleared: boolean
    }
  }
  let navigationManager: ReturnType<typeof createContentNavigationManager>

  beforeEach(() => {
    // Create a mock DOM element
    mockNoteContentElement = document.createElement('div')
    mockNoteContentElement.innerHTML = `
      <h1>First Header</h1>
      <p>Some content with <mark class="highlight">search</mark> term</p>
      <h2>Second Header</h2>
      <p>More content with another <mark class="highlight">search</mark> result</p>
      <h3>Third Header</h3>
      <p>Final content</p>
    `

    // Mock getBoundingClientRect for elements
    Element.prototype.getBoundingClientRect = vi.fn(() => ({
      top: 100,
      left: 0,
      bottom: 120,
      right: 100,
      width: 100,
      height: 20,
      x: 0,
      y: 100,
      toJSON: vi.fn(),
    }))

    // Mock scrollIntoView
    Element.prototype.scrollIntoView = vi.fn()

    // Mock scroll methods for container element
    mockNoteContentElement.scrollTo = vi.fn()
    mockNoteContentElement.scrollTop = 0
    Object.defineProperty(mockNoteContentElement, 'clientHeight', {
      value: 500,
      configurable: true,
    })
    mockNoteContentElement.getBoundingClientRect = vi.fn(() => ({
      top: 0,
      left: 0,
      bottom: 500,
      right: 800,
      width: 800,
      height: 500,
      x: 0,
      y: 0,
      toJSON: vi.fn(),
    }))

    mockDeps = {
      focusManager: {
        noteContentElement: mockNoteContentElement,
      },
      searchManager: {
        query: '',
        areHighlightsCleared: false,
      },
    }

    navigationManager = createContentNavigationManager(mockDeps)
  })

  describe('search highlight navigation', () => {
    beforeEach(() => {
      mockDeps.searchManager.query = 'search'
    })

    it('should navigate to first highlight on navigateNext', () => {
      navigationManager.navigateNext()

      const highlights =
        mockNoteContentElement.querySelectorAll('mark.highlight')
      expect(highlights[0].scrollIntoView).toHaveBeenCalledWith({
        behavior: 'smooth',
        block: 'center',
        inline: 'nearest',
      })
      expect(highlights[0].classList.contains('highlight-current')).toBe(true)
    })

    it('should navigate to second highlight on consecutive navigateNext calls', () => {
      navigationManager.navigateNext()
      navigationManager.navigateNext()

      const highlights =
        mockNoteContentElement.querySelectorAll('mark.highlight')
      expect(highlights[1].scrollIntoView).toHaveBeenCalledWith({
        behavior: 'smooth',
        block: 'center',
        inline: 'nearest',
      })
    })

    it('should stay at last highlight when navigating beyond bounds', () => {
      navigationManager.navigateNext()
      navigationManager.navigateNext()
      navigationManager.navigateNext()

      const highlights =
        mockNoteContentElement.querySelectorAll('mark.highlight')
      expect(highlights[1].scrollIntoView).toHaveBeenCalledTimes(3)
    })

    it('should navigate backwards with navigatePrevious', () => {
      navigationManager.navigateNext()
      navigationManager.navigateNext()
      navigationManager.navigatePrevious()

      const highlights =
        mockNoteContentElement.querySelectorAll('mark.highlight')
      expect(highlights[0].scrollIntoView).toHaveBeenCalledTimes(3)
    })
  })

  describe('header navigation', () => {
    beforeEach(() => {
      mockDeps.searchManager.query = '' // No search query = header mode
    })

    it('should navigate to first header on navigateNext', () => {
      navigationManager.navigateNext()

      const headers = mockNoteContentElement.querySelectorAll(
        'h1, h2, h3, h4, h5, h6'
      )
      expect(headers[0].scrollIntoView).toHaveBeenCalledWith({
        behavior: 'smooth',
        block: 'center',
        inline: 'nearest',
      })
      expect(headers[0].classList.contains('header-current')).toBe(true)
    })

    it('should navigate through all headers sequentially', () => {
      navigationManager.navigateNext()
      navigationManager.navigateNext()
      navigationManager.navigateNext()

      const headers = mockNoteContentElement.querySelectorAll(
        'h1, h2, h3, h4, h5, h6'
      )
      expect(headers[0].scrollIntoView).toHaveBeenCalledTimes(3)
      expect(headers[1].scrollIntoView).toHaveBeenCalledTimes(3)
      expect(headers[2].scrollIntoView).toHaveBeenCalledTimes(3)
    })

    it('should navigate backwards through headers', () => {
      navigationManager.navigateNext()
      navigationManager.navigateNext()
      navigationManager.navigatePrevious()

      const headers = mockNoteContentElement.querySelectorAll(
        'h1, h2, h3, h4, h5, h6'
      )
      expect(headers[0].scrollIntoView).toHaveBeenCalledTimes(3)
    })
  })

  describe('highlight cleared behavior', () => {
    it('should switch to header navigation when highlights are cleared even with query present', () => {
      // Set up: query present, highlights cleared
      mockDeps.searchManager.query = 'search'
      mockDeps.searchManager.areHighlightsCleared = true

      navigationManager.navigateNext()

      // Should navigate to first header, not highlights
      const headers = mockNoteContentElement.querySelectorAll(
        'h1, h2, h3, h4, h5, h6'
      )
      expect(headers[0].scrollIntoView).toHaveBeenCalledWith({
        behavior: 'smooth',
        block: 'center',
        inline: 'nearest',
      })
      expect(headers[0].classList.contains('header-current')).toBe(true)
    })

    it('should use highlight navigation when query present and highlights not cleared', () => {
      // Set up: query present, highlights not cleared
      mockDeps.searchManager.query = 'search'
      mockDeps.searchManager.areHighlightsCleared = false

      navigationManager.navigateNext()

      // Should navigate to first highlight, not headers
      const highlights =
        mockNoteContentElement.querySelectorAll('mark.highlight')
      expect(highlights[0].scrollIntoView).toHaveBeenCalledWith({
        behavior: 'smooth',
        block: 'center',
        inline: 'nearest',
      })
      expect(highlights[0].classList.contains('highlight-current')).toBe(true)
    })
  })

  describe('edge cases', () => {
    it('should handle missing noteContentElement gracefully', () => {
      mockDeps.focusManager.noteContentElement = null

      expect(() => {
        navigationManager.navigateNext()
        navigationManager.navigatePrevious()
      }).not.toThrow()
    })

    it('should handle empty content gracefully', () => {
      mockNoteContentElement.innerHTML = ''

      expect(() => {
        navigationManager.navigateNext()
        navigationManager.navigatePrevious()
      }).not.toThrow()
    })

    it('should reset navigation state', () => {
      navigationManager.navigateNext()
      navigationManager.resetNavigation()
      navigationManager.navigateNext()

      const headers = mockNoteContentElement.querySelectorAll(
        'h1, h2, h3, h4, h5, h6'
      )
      expect(headers[0].scrollIntoView).toHaveBeenCalledTimes(2)
    })

    it('should clear current styles with clearCurrentStyles', () => {
      navigationManager.navigateNext()
      const headers = mockNoteContentElement.querySelectorAll(
        'h1, h2, h3, h4, h5, h6'
      )
      expect(headers[0].classList.contains('header-current')).toBe(true)

      navigationManager.clearCurrentStyles()
      expect(headers[0].classList.contains('header-current')).toBe(false)
    })
  })
})
