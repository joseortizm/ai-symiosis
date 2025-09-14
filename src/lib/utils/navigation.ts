export function getHeaderLevel(header: Element): number {
  return parseInt(header.tagName.charAt(1))
}

export function getContentBetweenHeaders(startHeader: Element): Element[] {
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

export function getFormattedText(element: Element): string {
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

export function isUrl(href: string): boolean {
  return (
    href.startsWith('http://') ||
    href.startsWith('https://') ||
    href.startsWith('mailto:') ||
    href.startsWith('tel:') ||
    href.startsWith('ftp://') ||
    href.startsWith('ftps://')
  )
}

export function isSection(href: string): boolean {
  return href.startsWith('#')
}

export function isFilePath(href: string): boolean {
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

export function getHeaderElements(contentElement: HTMLElement): Element[] {
  return Array.from(contentElement.querySelectorAll('h1, h2, h3, h4, h5, h6'))
}
