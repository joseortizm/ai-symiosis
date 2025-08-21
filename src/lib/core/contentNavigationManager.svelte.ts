/**
 * Core Layer - Content Navigation Manager
 * Navigation between search highlights and markdown headers.
 * Handles Ctrl+H/L navigation with position tracking and smooth scrolling.
 */

interface NavigationState {
  currentIndex: number
  isNavigatingHighlights: boolean
  currentElement: Element | null
}

interface NavigationDeps {
  focusManager: {
    readonly noteContentElement: HTMLElement | null
  }
  searchManager: {
    readonly query: string
    readonly areHighlightsCleared: boolean
  }
}

export interface ContentNavigationManager {
  navigateNext(): void
  navigatePrevious(): void
  resetNavigation(): void
  clearCurrentStyles(): void
}

export function createContentNavigationManager(
  deps: NavigationDeps
): ContentNavigationManager {
  const state = $state<NavigationState>({
    currentIndex: -1,
    isNavigatingHighlights: false,
    currentElement: null,
  })

  function getNavigationElements(): Element[] {
    const contentElement = deps.focusManager.noteContentElement
    if (!contentElement) return []

    const query = deps.searchManager.query
    const areHighlightsCleared = deps.searchManager.areHighlightsCleared
    const hasQuery = query.trim() !== ''

    // Use highlight navigation only if there's a query AND highlights haven't been cleared
    const shouldNavigateHighlights = hasQuery && !areHighlightsCleared

    if (state.isNavigatingHighlights !== shouldNavigateHighlights) {
      clearCurrentElementStyle()
      state.currentIndex = -1
    }

    if (shouldNavigateHighlights) {
      state.isNavigatingHighlights = true
      return Array.from(contentElement.querySelectorAll('mark.highlight'))
    } else {
      state.isNavigatingHighlights = false
      return Array.from(
        contentElement.querySelectorAll('h1, h2, h3, h4, h5, h6')
      )
    }
  }

  function clearCurrentElementStyle(): void {
    if (state.currentElement) {
      if (state.isNavigatingHighlights) {
        state.currentElement.classList.remove('highlight-current')
      } else {
        state.currentElement.classList.remove('header-current')
      }
      state.currentElement = null
    }
  }

  function setCurrentElementStyle(element: Element): void {
    clearCurrentElementStyle()
    state.currentElement = element
    if (state.isNavigatingHighlights) {
      element.classList.add('highlight-current')
    } else {
      element.classList.add('header-current')
    }
  }

  function scrollToElement(element: Element): void {
    setCurrentElementStyle(element)
    element.scrollIntoView({
      behavior: 'smooth',
      block: 'center',
      inline: 'nearest',
    })
  }

  function navigateNext(): void {
    const elements = getNavigationElements()
    if (elements.length === 0) return

    // If no current position, start with first element
    if (state.currentIndex === -1) {
      state.currentIndex = 0
      scrollToElement(elements[state.currentIndex])
      return
    }

    // Move to next element
    state.currentIndex = Math.min(state.currentIndex + 1, elements.length - 1)
    scrollToElement(elements[state.currentIndex])
  }

  function navigatePrevious(): void {
    const elements = getNavigationElements()
    if (elements.length === 0) return

    // If no current position, start with first element
    if (state.currentIndex === -1) {
      state.currentIndex = 0
      scrollToElement(elements[state.currentIndex])
      return
    }

    // Move to previous element
    state.currentIndex = Math.max(state.currentIndex - 1, 0)
    scrollToElement(elements[state.currentIndex])
  }

  function resetNavigation(): void {
    clearCurrentElementStyle()
    state.currentIndex = -1
  }

  function clearCurrentStyles(): void {
    clearCurrentElementStyle()
  }

  return {
    navigateNext,
    navigatePrevious,
    resetNavigation,
    clearCurrentStyles,
  }
}
