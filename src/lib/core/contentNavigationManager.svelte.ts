/**
 * Core Layer - Content Navigation Manager
 * Navigation between search highlights and markdown headers.
 * Handles Ctrl+H/L navigation with position tracking and smooth scrolling.
 */

import { SvelteSet } from 'svelte/reactivity'

interface NavigationState {
  currentIndex: number
  navigationMode: 'inactive' | 'highlights' | 'headers' | 'links'
  highlightVisibility: 'visible' | 'hidden'
  currentElement: Element | null
  collapsedSections: SvelteSet<Element>
  currentSection: Element | null
  // Separate state for code block navigation
  codeBlockIndex: number
  codeBlockElement: Element | null
  // Separate state for link navigation
  linkIndex: number
  linkElement: Element | null
}

interface NavigationDeps {
  focusManager: {
    readonly noteContentElement: HTMLElement | null
  }
  searchManager: {
    readonly query: string
    readonly searchInput: string
    clearSearch(): void
  }
}

export type EscapeAction =
  | 'navigation_cleared'
  | 'highlights_cleared'
  | 'search_cleared'
  | 'focus_search'

export interface ContentNavigationManager {
  navigateNext(): void
  navigatePrevious(): void
  navigateCodeNext(): void
  navigateCodePrevious(): void
  navigateLinkNext(): void
  navigateLinkPrevious(): void
  openCurrentLink(): void
  resetNavigation(): void
  clearCurrentStyles(): void
  clearHighlights(): void
  showHighlights(): void
  startHighlightNavigation(): void
  handleEscape(): EscapeAction
  copyCurrentSection(): Promise<boolean>
  getCurrentHeaderText(): string
  navigateToHeader(headerText: string): boolean
  readonly isActivelyNavigating: boolean
  readonly hideHighlights: boolean
  readonly isNavigatingLinks: boolean
}

export function createContentNavigationManager(
  deps: NavigationDeps
): ContentNavigationManager {
  const state = $state<NavigationState>({
    currentIndex: -1,
    navigationMode: 'inactive',
    highlightVisibility: 'visible',
    currentElement: null,
    collapsedSections: new SvelteSet<Element>(),
    currentSection: null,
    codeBlockIndex: -1,
    codeBlockElement: null,
    linkIndex: -1,
    linkElement: null,
  })

  function determineNavigationMode(): 'highlights' | 'headers' {
    const query = deps.searchManager.query
    const hasQuery = query.trim() !== ''

    // Use highlights if: query exists AND highlights are visible AND highlights exist in DOM
    if (hasQuery && state.highlightVisibility === 'visible') {
      const contentElement = deps.focusManager.noteContentElement
      if (contentElement) {
        const highlightElements =
          contentElement.querySelectorAll('mark.highlight')
        if (highlightElements.length > 0) {
          return 'highlights'
        }
      }
    }

    // Otherwise use headers
    return 'headers'
  }

  function getHighlightElements(): Element[] {
    const contentElement = deps.focusManager.noteContentElement
    if (!contentElement) return []

    return Array.from(contentElement.querySelectorAll('mark.highlight'))
  }

  function getHeaderElements(): Element[] {
    const contentElement = deps.focusManager.noteContentElement
    if (!contentElement) return []

    return Array.from(contentElement.querySelectorAll('h1, h2, h3, h4, h5, h6'))
  }

  function getCodeBlockElements(): Element[] {
    const contentElement = deps.focusManager.noteContentElement
    if (!contentElement) return []

    return Array.from(contentElement.querySelectorAll('pre > code'))
  }

  function getLinkElements(): Element[] {
    const contentElement = deps.focusManager.noteContentElement
    if (!contentElement) return []

    return Array.from(contentElement.querySelectorAll('a[href]'))
  }

  function getCurrentNavigationElements(): Element[] {
    const targetMode = determineNavigationMode()

    // If mode changed, reset navigation state
    if (
      state.navigationMode !== 'inactive' &&
      state.navigationMode !== targetMode
    ) {
      clearCurrentElementStyle()
      state.currentIndex = -1
    }

    return targetMode === 'highlights'
      ? getHighlightElements()
      : getHeaderElements()
  }

  function clearCurrentElementStyle(): void {
    if (state.currentElement) {
      if (state.navigationMode === 'highlights') {
        state.currentElement.classList.remove('highlight-current')
      } else {
        state.currentElement.classList.remove('header-current')
      }
      state.currentElement = null
    }

    if (state.codeBlockElement) {
      state.codeBlockElement.classList.remove('codeblock-current')
      state.codeBlockElement = null
    }

    if (state.linkElement) {
      state.linkElement.classList.remove('link-current')
      state.linkElement = null
    }

    // Reset navigation mode when clearing all styles
    if (state.navigationMode === 'links') {
      state.navigationMode = 'inactive'
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
    if (state.navigationMode === 'highlights') return

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

  function setCurrentElementStyle(
    element: Element,
    mode: 'highlights' | 'headers'
  ): void {
    clearCurrentElementStyle()
    state.currentElement = element
    state.navigationMode = mode

    if (mode === 'highlights') {
      element.classList.add('highlight-current')
    } else {
      element.classList.add('header-current')
      updateAccordionState(element)
    }
  }

  function scrollToElement(element: Element): void {
    const mode = determineNavigationMode()
    setCurrentElementStyle(element, mode)

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

  function scrollToCodeBlock(element: Element): void {
    if (state.codeBlockElement) {
      state.codeBlockElement.classList.remove('codeblock-current')
    }

    state.codeBlockElement = element
    element.classList.add('codeblock-current')

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
    const targetPosition = containerHeight * 0.25

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
    const elements = getCurrentNavigationElements()
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
    const elements = getCurrentNavigationElements()
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

  function navigateCodeNext(): void {
    const elements = getCodeBlockElements()
    if (elements.length === 0) return

    if (state.codeBlockIndex === -1) {
      state.codeBlockIndex = 0
    } else {
      state.codeBlockIndex = Math.min(
        state.codeBlockIndex + 1,
        elements.length - 1
      )
    }

    scrollToCodeBlock(elements[state.codeBlockIndex])
  }

  function navigateCodePrevious(): void {
    const elements = getCodeBlockElements()
    if (elements.length === 0) return

    if (state.codeBlockIndex === -1) {
      state.codeBlockIndex = 0
    } else {
      state.codeBlockIndex = Math.max(state.codeBlockIndex - 1, 0)
    }

    scrollToCodeBlock(elements[state.codeBlockIndex])
  }

  function scrollToLink(element: Element): void {
    if (state.linkElement) {
      state.linkElement.classList.remove('link-current')
    }

    state.linkElement = element
    element.classList.add('link-current')

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
    const targetPosition = containerHeight * 0.25

    const scrollTop =
      contentElement.scrollTop +
      (elementRect.top - containerRect.top) -
      targetPosition

    contentElement.scrollTo({
      top: scrollTop,
      behavior: 'smooth',
    })
  }

  function navigateLinkNext(): void {
    const elements = getLinkElements()
    if (elements.length === 0) return

    // Clear any existing navigation state before switching to link mode
    if (state.navigationMode !== 'links') {
      resetNavigation()
    }

    state.navigationMode = 'links'
    if (state.linkIndex === -1) {
      state.linkIndex = 0
    } else {
      state.linkIndex = Math.min(state.linkIndex + 1, elements.length - 1)
    }

    scrollToLink(elements[state.linkIndex])
  }

  function navigateLinkPrevious(): void {
    const elements = getLinkElements()
    if (elements.length === 0) return

    // Clear any existing navigation state before switching to link mode
    if (state.navigationMode !== 'links') {
      resetNavigation()
    }

    state.navigationMode = 'links'
    if (state.linkIndex === -1) {
      state.linkIndex = 0
    } else {
      state.linkIndex = Math.max(state.linkIndex - 1, 0)
    }

    scrollToLink(elements[state.linkIndex])
  }

  function isUrl(href: string): boolean {
    return (
      href.startsWith('http://') ||
      href.startsWith('https://') ||
      href.startsWith('mailto:') ||
      href.startsWith('tel:') ||
      href.startsWith('ftp://') ||
      href.startsWith('ftps://')
    )
  }

  function isSection(href: string): boolean {
    return href.startsWith('#')
  }

  function isFilePath(href: string): boolean {
    // Check for absolute paths (starting with / or C:\ etc)
    if (href.startsWith('/') || /^[A-Za-z]:\\/.test(href)) {
      return true
    }

    // Check for relative paths (starting with ./ or ../)
    if (href.startsWith('./') || href.startsWith('../')) {
      return true
    }

    // Check for file extensions (common file types)
    const fileExtensions =
      /\.(txt|md|pdf|doc|docx|xls|xlsx|ppt|pptx|jpg|jpeg|png|gif|svg|mp4|mp3|zip|tar|gz|json|xml|csv|html|css|js|ts|py|rs|go|java|cpp|c|h)$/i
    return fileExtensions.test(href)
  }

  function validateLinkForOpening(): string | null {
    if (state.navigationMode !== 'links' || !state.linkElement) return null

    const href = state.linkElement.getAttribute('href')

    if (!href) {
      import('../utils/errorNotification').then(({ errorNotification }) => {
        errorNotification.trigger('Link has no URL')
      })
      return null
    }

    return href
  }

  function handleSectionNavigation(href: string): void {
    const sectionName = href.substring(1)

    const contentElement = deps.focusManager.noteContentElement
    if (!contentElement) {
      import('../utils/errorNotification').then(({ errorNotification }) => {
        errorNotification.trigger('Content not available for navigation')
      })
      return
    }

    const headers = Array.from(
      contentElement.querySelectorAll('h1, h2, h3, h4, h5, h6')
    )

    let targetHeader = headers.find((header) => {
      const headerText = (header.textContent || '').trim().toLowerCase()
      return headerText === sectionName.toLowerCase()
    })

    if (!targetHeader) {
      targetHeader = headers.find((header) => {
        const headerText = (header.textContent || '').trim().toLowerCase()
        return (
          headerText.includes(sectionName.toLowerCase()) ||
          sectionName.toLowerCase().includes(headerText)
        )
      })
    }

    if (targetHeader) {
      const headerElements = getHeaderElements()
      const headerIndex = headerElements.indexOf(targetHeader)
      if (headerIndex >= 0) {
        state.currentIndex = headerIndex
        state.navigationMode = 'headers'
        scrollToElement(targetHeader)
        return
      }
    }

    import('../utils/errorNotification').then(({ errorNotification }) => {
      errorNotification.trigger(`Section not found: ${sectionName}`)
    })
  }

  function handleFilePathOpening(href: string): void {
    import('@tauri-apps/plugin-opener').then(({ openPath }) => {
      openPath(href).catch((error) => {
        console.error('Failed to open file:', error)
        import('../utils/errorNotification').then(({ errorNotification }) => {
          errorNotification.trigger(`Failed to open file: ${href}`)
        })
      })
    })
  }

  function handleUrlOpening(href: string): void {
    try {
      // eslint-disable-next-line svelte/prefer-svelte-reactivity
      new URL(href)

      import('@tauri-apps/plugin-opener').then(({ openUrl }) => {
        openUrl(href).catch((error) => {
          console.error('Failed to open URL:', error)
          import('../utils/errorNotification').then(({ errorNotification }) => {
            errorNotification.trigger(`Failed to open link: ${href}`)
          })
        })
      })
    } catch {
      import('../utils/errorNotification').then(({ errorNotification }) => {
        errorNotification.trigger(`Malformed URL: ${href}`)
      })
    }
  }

  function handleUnsupportedLinkFormat(href: string): void {
    import('../utils/errorNotification').then(({ errorNotification }) => {
      errorNotification.trigger(`Unsupported link format: ${href}`)
    })
  }

  function openCurrentLink(): void {
    const href = validateLinkForOpening()
    if (!href) return

    // Handle #section navigation
    if (isSection(href)) {
      handleSectionNavigation(href)
      return
    }

    // Handle file paths
    if (isFilePath(href)) {
      handleFilePathOpening(href)
      return
    }

    // Handle URLs
    if (isUrl(href)) {
      handleUrlOpening(href)
      return
    }

    // Fallback for unrecognized link types
    handleUnsupportedLinkFormat(href)
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
    state.navigationMode = 'inactive'
    state.codeBlockIndex = -1
    state.linkIndex = -1
  }

  function clearCurrentStyles(): void {
    clearCurrentElementStyle()
  }

  function clearHighlights(): void {
    state.highlightVisibility = 'hidden'
  }

  function showHighlights(): void {
    state.highlightVisibility = 'visible'
  }

  function startHighlightNavigation(): void {
    // Only start if we have highlights and they're visible
    const query = deps.searchManager.query
    const hasQuery = query.trim() !== ''

    if (!hasQuery || state.highlightVisibility !== 'visible') {
      return
    }

    const elements = getHighlightElements()
    if (elements.length === 0) {
      return
    }

    // Set navigation to highlight mode and select first element
    state.navigationMode = 'highlights'
    state.currentIndex = 0
    scrollToElement(elements[0])
  }

  function getFormattedText(element: Element): string {
    if (element.tagName === 'UL' || element.tagName === 'OL') {
      const items = Array.from(element.children)
      const marker =
        element.tagName === 'UL' ? '- ' : (index: number) => `${index + 1}. `
      return items
        .map((item, index) => {
          const prefix = typeof marker === 'string' ? marker : marker(index)
          return prefix + (item.textContent || '').trim()
        })
        .join('\n')
    }

    if (element.tagName === 'LI') {
      const parent = element.parentElement
      if (parent?.tagName === 'UL') {
        return '- ' + (element.textContent || '').trim()
      } else if (parent?.tagName === 'OL') {
        const index = Array.from(parent.children).indexOf(element)
        return `${index + 1}. ` + (element.textContent || '').trim()
      }
    }

    return element.textContent || ''
  }

  async function copyCurrentSection(): Promise<boolean> {
    let textToCopy = ''

    if (state.codeBlockElement) {
      textToCopy = state.codeBlockElement.textContent || ''
    } else if (state.currentElement && state.navigationMode === 'highlights') {
      textToCopy = getFormattedText(state.currentElement)
    } else if (state.currentElement && state.navigationMode === 'headers') {
      const headerText = state.currentElement.textContent || ''
      const content = getContentBetweenHeaders(state.currentElement)
      const contentTexts = content
        .map((el) => getFormattedText(el))
        .filter((text) => text.trim())
      textToCopy = [headerText, ...contentTexts].join('\n\n')
    }

    if (textToCopy.trim()) {
      try {
        await navigator.clipboard.writeText(textToCopy)
        return true
      } catch (error) {
        console.warn('Failed to copy to clipboard:', error)
      }
    }

    return false
  }

  function handleEscape(): EscapeAction {
    // Priority 1: Clear link navigation if actively navigating links
    if (state.navigationMode === 'links') {
      resetNavigation()
      return 'navigation_cleared'
    }

    // Priority 2: Clear code block navigation if navigating code blocks
    if (state.codeBlockElement) {
      clearCurrentElementStyle()
      state.codeBlockIndex = -1
      state.linkIndex = -1 // Also reset link navigation
      state.navigationMode = 'inactive' // Reset navigation mode
      return 'navigation_cleared'
    }

    // Priority 3: Clear highlights immediately if actively navigating highlights
    if (state.navigationMode === 'highlights') {
      resetNavigation()
      clearHighlights()
      return 'highlights_cleared'
    }

    // Priority 4: Clear active header navigation
    if (state.navigationMode !== 'inactive') {
      resetNavigation()
      return 'navigation_cleared'
    }

    // Priority 3: Clear highlights if they exist and query exists
    const query = deps.searchManager.query
    const searchInput = deps.searchManager.searchInput
    const hasQuery = query.trim() !== ''
    const hasSearchInput = searchInput.trim() !== ''

    if (hasQuery && state.highlightVisibility === 'visible') {
      clearHighlights()
      return 'highlights_cleared'
    }

    // Priority 4: Clear search if highlights hidden but query exists
    if (hasQuery && state.highlightVisibility === 'hidden') {
      deps.searchManager.clearSearch()
      showHighlights() // Reset highlights for next search
      return 'search_cleared'
    }

    // Priority 5: Clear search if searchInput has less than 3 characters (typed but not committed)
    if (hasSearchInput && searchInput.length < 3) {
      deps.searchManager.clearSearch()
      showHighlights() // Reset highlights for next search
      return 'search_cleared'
    }

    // Priority 6: Focus search (default case)
    return 'focus_search'
  }

  function htmlToMarkdown(node: Node): string {
    if (node.nodeType === Node.TEXT_NODE) {
      return node.textContent || ''
    }

    if (node.nodeType !== Node.ELEMENT_NODE) return ''

    const el = node as HTMLElement
    const tag = el.tagName.toLowerCase()
    const children = Array.from(el.childNodes).map(htmlToMarkdown).join('')

    switch (tag) {
      case 'strong':
      case 'b':
        return `**${children}**`
      case 'em':
      case 'i':
        return `*${children}*`
      case 'code':
        return `\`${children}\``
      case 'a':
        return `[${children}](${el.getAttribute('href') || ''})`
      case 'del':
      case 's':
        return `~~${children}~~`
      default:
        return children
    }
  }

  function getCurrentHeaderText(): string {
    function toMarkdown(header: Element): string {
      const level = parseInt(header.tagName[1], 10)
      const content = htmlToMarkdown(header)
      return `${'#'.repeat(level)} ${content}`
    }

    // Your previous logic for finding the header:
    let header: Element | null = null

    if (state.navigationMode === 'headers' && state.currentElement) {
      header = state.currentElement
    } else if (state.navigationMode === 'highlights' && state.currentElement) {
      const contentElement = deps.focusManager.noteContentElement
      if (contentElement) {
        const headers = Array.from(
          contentElement.querySelectorAll('h1, h2, h3, h4, h5, h6')
        )
        for (const h of headers) {
          const pos = h.compareDocumentPosition(state.currentElement)
          if (pos & Node.DOCUMENT_POSITION_FOLLOWING) {
            header = h
          } else {
            break
          }
        }
      }
    }

    if (!header) {
      const contentElement = deps.focusManager.noteContentElement
      if (!contentElement) return ''
      const rect = contentElement.getBoundingClientRect()
      const headers = contentElement.querySelectorAll('h1, h2, h3, h4, h5, h6')

      let firstVisibleHeader: Element | null = null
      let lastPassedHeader: Element | null = null

      for (const h of headers) {
        const r = h.getBoundingClientRect()
        const isVisible =
          r.top >= rect.top && r.top <= rect.top + (rect.height || 600)
        if (isVisible && !firstVisibleHeader) firstVisibleHeader = h
        else if (r.top < rect.top) lastPassedHeader = h
      }

      header = firstVisibleHeader || lastPassedHeader
    }

    return header ? toMarkdown(header) : ''
  }

  function markdownToHtml(text: string): string {
    return text
      .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
      .replace(/\*(.*?)\*/g, '<em>$1</em>')
      .replace(/`(.*?)`/g, '<code>$1</code>')
      .replace(/~~(.*?)~~/g, '<del>$1</del>')
      .replace(/\[(.*?)\]\((.*?)\)/g, '<a href="$2">$1</a>')
  }

  function navigateToHeader(headerText: string): boolean {
    if (!headerText.trim()) return false

    const contentElement = deps.focusManager.noteContentElement
    if (!contentElement) return false

    const headers = Array.from(
      contentElement.querySelectorAll('h1, h2, h3, h4, h5, h6')
    )

    // Extract content from markdown header and convert to HTML
    const cleanHeaderText = headerText.replace(/^#+\s*/, '').trim()
    const expectedHtml = markdownToHtml(cleanHeaderText)

    const targetHeader = headers.find((header) => {
      // Get innerHTML content from HTML header
      const headerHtmlContent = header.innerHTML?.trim() || ''
      return headerHtmlContent === expectedHtml
    })

    if (targetHeader) {
      const headerElements = getHeaderElements()
      const headerIndex = headerElements.indexOf(targetHeader)
      if (headerIndex >= 0) {
        state.currentIndex = headerIndex
        state.navigationMode = 'headers'
        scrollToElement(targetHeader)
        return true
      }
    }

    return false
  }

  return {
    navigateNext,
    navigatePrevious,
    navigateCodeNext,
    navigateCodePrevious,
    navigateLinkNext,
    navigateLinkPrevious,
    openCurrentLink,
    resetNavigation,
    clearCurrentStyles,
    clearHighlights,
    showHighlights,
    startHighlightNavigation,
    handleEscape,
    copyCurrentSection,
    getCurrentHeaderText,
    navigateToHeader,

    get isActivelyNavigating(): boolean {
      return state.navigationMode !== 'inactive'
    },

    get hideHighlights(): boolean {
      return state.highlightVisibility === 'hidden'
    },

    get isNavigatingLinks(): boolean {
      return state.navigationMode === 'links'
    },
  }
}
