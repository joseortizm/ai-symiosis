/**
 * Core Layer - Content Navigation Manager
 * Navigation between search highlights and markdown headers.
 * Handles Ctrl+H/L navigation with position tracking and smooth scrolling.
 */

import { SvelteSet } from 'svelte/reactivity'

interface NavigationState {
  currentIndex: number
  isNavigatingHighlights: boolean
  currentElement: Element | null
  collapsedSections: SvelteSet<Element>
  currentSection: Element | null
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
    collapsedSections: new SvelteSet<Element>(),
    currentSection: null,
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

  function getHeaderLevel(header: Element): number {
    return parseInt(header.tagName.charAt(1))
  }

  function getContentBetweenHeaders(startHeader: Element): Element[] {
    const content: Element[] = []
    const startLevel = getHeaderLevel(startHeader)
    let current = startHeader.nextElementSibling

    while (current) {
      if (current.matches('h1, h2, h3, h4, h5, h6')) {
        const currentLevel = getHeaderLevel(current)
        // Stop if we hit a header at the same level or higher (parent level)
        if (currentLevel <= startLevel) {
          break
        }
        // Include sub-headers as content (they belong to this section)
        content.push(current)
      } else {
        // Include all non-header content
        content.push(current)
      }
      current = current.nextElementSibling
    }

    return content
  }

  function updateAccordionState(currentHeader: Element): void {
    // Skip accordion logic for highlight navigation
    if (state.isNavigatingHighlights) return

    const contentElement = deps.focusManager.noteContentElement
    if (!contentElement) return

    // Get all headers for hierarchical folding
    const allHeaders = Array.from(
      contentElement.querySelectorAll('h1, h2, h3, h4, h5, h6')
    )

    // Clear all existing header styling
    allHeaders.forEach((header) => {
      header.classList.remove('header-expanded', 'header-collapsed')
    })

    // Clear all content styling
    const allElements = Array.from(contentElement.querySelectorAll('*'))
    allElements.forEach((el) => {
      el.classList.remove('content-collapsed')
    })

    state.collapsedSections.clear()

    // For each header, determine if it should be expanded or collapsed
    allHeaders.forEach((header) => {
      const content = getContentBetweenHeaders(header)

      // Check if this header or any of its descendants is the current header
      const isCurrentPath =
        header === currentHeader ||
        content.some(
          (el) =>
            el === currentHeader ||
            (el.matches('h1, h2, h3, h4, h5, h6') &&
              isHeaderInPath(el, currentHeader))
        )

      if (isCurrentPath) {
        // This header is in the path to current - expand it
        header.classList.add('header-expanded')
      } else {
        // This header is not in the current path - collapse it
        header.classList.add('header-collapsed')
        // Hide its content (but keep sub-headers visible for structure)
        content.forEach((el) => {
          if (!el.matches('h1, h2, h3, h4, h5, h6')) {
            el.classList.add('content-collapsed')
          }
        })
        state.collapsedSections.add(header)
      }
    })
  }

  function isHeaderInPath(header: Element, target: Element): boolean {
    const content = getContentBetweenHeaders(header)
    return (
      content.includes(target) ||
      content.some(
        (el) =>
          el.matches('h1, h2, h3, h4, h5, h6') && isHeaderInPath(el, target)
      )
    )
  }

  function setCurrentElementStyle(element: Element): void {
    clearCurrentElementStyle()
    state.currentElement = element
    if (state.isNavigatingHighlights) {
      element.classList.add('highlight-current')
    } else {
      element.classList.add('header-current')
      updateAccordionState(element)
    }
  }

  function scrollToElement(element: Element): void {
    setCurrentElementStyle(element)

    const contentElement = deps.focusManager.noteContentElement
    if (!contentElement) {
      element.scrollIntoView({
        behavior: 'smooth',
        block: 'start',
        inline: 'nearest',
      })
      return
    }

    const elementRect = element.getBoundingClientRect()
    const containerRect = contentElement.getBoundingClientRect()
    const containerHeight = contentElement.clientHeight
    const targetPosition = containerHeight * 0.25 // 3/4 up = 1/4 down from top

    const scrollTop =
      contentElement.scrollTop +
      (elementRect.top - containerRect.top) -
      targetPosition

    contentElement.scrollTo({
      top: scrollTop,
      behavior: 'smooth',
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

    // Clear accordion state
    const contentElement = deps.focusManager.noteContentElement
    if (contentElement) {
      // Remove all accordion styling
      const allElements = Array.from(contentElement.querySelectorAll('*'))
      allElements.forEach((el) => {
        el.classList.remove(
          'header-expanded',
          'header-collapsed',
          'content-collapsed'
        )
      })
    }

    state.collapsedSections.clear()
    state.currentSection = null
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
