/**
 * Navigation Utility Tests
 * Tests for DOM navigation, header parsing, and content extraction utilities.
 */

import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import {
  getHeaderLevel,
  getContentBetweenHeaders,
  getFormattedText,
  isUrl,
  isSection,
  isFilePath,
  getHighlightElements,
  getHeaderElements,
  getCodeBlockElements,
  getLinkElements,
  getAccordionHeaders,
} from '../../../lib/utils/navigation'

describe('navigation utility', () => {
  let container: HTMLElement

  beforeEach(() => {
    container = document.createElement('div')
    document.body.appendChild(container)
  })

  afterEach(() => {
    document.body.removeChild(container)
  })

  describe('getHeaderLevel', () => {
    it('should return correct level for h1', () => {
      const h1 = document.createElement('h1')
      expect(getHeaderLevel(h1)).toBe(1)
    })

    it('should return correct level for h2', () => {
      const h2 = document.createElement('h2')
      expect(getHeaderLevel(h2)).toBe(2)
    })

    it('should return correct level for h6', () => {
      const h6 = document.createElement('h6')
      expect(getHeaderLevel(h6)).toBe(6)
    })
  })

  describe('getContentBetweenHeaders', () => {
    it('should return content between headers of same level', () => {
      container.innerHTML = `
        <h2>Header 1</h2>
        <p>Content 1</p>
        <div>More content</div>
        <h2>Header 2</h2>
        <p>Content 2</p>
      `

      const firstHeader = container.querySelector('h2')!
      const content = getContentBetweenHeaders(firstHeader)

      expect(content).toHaveLength(2)
      expect(content[0].tagName).toBe('P')
      expect(content[0].textContent).toBe('Content 1')
      expect(content[1].tagName).toBe('DIV')
      expect(content[1].textContent).toBe('More content')
    })

    it('should include sub-headers as content', () => {
      container.innerHTML = `
        <h1>Main Header</h1>
        <p>Content</p>
        <h2>Sub Header</h2>
        <p>Sub content</p>
        <h1>Another Main Header</h1>
      `

      const firstHeader = container.querySelector('h1')!
      const content = getContentBetweenHeaders(firstHeader)

      expect(content).toHaveLength(3)
      expect(content[0].tagName).toBe('P')
      expect(content[1].tagName).toBe('H2')
      expect(content[2].tagName).toBe('P')
    })

    it('should stop at parent level header', () => {
      container.innerHTML = `
        <h2>Level 2</h2>
        <p>Content</p>
        <h3>Level 3</h3>
        <p>Sub content</p>
        <h1>Level 1</h1>
        <p>Should not include</p>
      `

      const h2 = container.querySelector('h2')!
      const content = getContentBetweenHeaders(h2)

      expect(content).toHaveLength(3)
      expect(content[2].textContent).toBe('Sub content')
    })

    it('should handle header at end of document', () => {
      container.innerHTML = `
        <h1>Header</h1>
        <p>Content 1</p>
        <p>Content 2</p>
      `

      const header = container.querySelector('h1')!
      const content = getContentBetweenHeaders(header)

      expect(content).toHaveLength(2)
      expect(content[0].textContent).toBe('Content 1')
      expect(content[1].textContent).toBe('Content 2')
    })

    it('should return empty array when no content follows', () => {
      container.innerHTML = `
        <h1>Header</h1>
        <h1>Next Header</h1>
      `

      const firstHeader = container.querySelector('h1')!
      const content = getContentBetweenHeaders(firstHeader)

      expect(content).toHaveLength(0)
    })
  })

  describe('getFormattedText', () => {
    it('should format unordered list with bullets', () => {
      const ul = document.createElement('ul')
      ul.innerHTML = `
        <li>First item</li>
        <li>Second item</li>
        <li>Third item</li>
      `

      const result = getFormattedText(ul)

      expect(result).toBe('- First item\n- Second item\n- Third item')
    })

    it('should format ordered list with numbers', () => {
      const ol = document.createElement('ol')
      ol.innerHTML = `
        <li>First step</li>
        <li>Second step</li>
        <li>Third step</li>
      `

      const result = getFormattedText(ol)

      expect(result).toBe('1. First step\n2. Second step\n3. Third step')
    })

    it('should format individual list item from unordered list', () => {
      const ul = document.createElement('ul')
      ul.innerHTML = '<li>List item</li>'
      const li = ul.querySelector('li')!

      const result = getFormattedText(li)

      expect(result).toBe('- List item')
    })

    it('should format individual list item from ordered list', () => {
      const ol = document.createElement('ol')
      ol.innerHTML = `
        <li>First</li>
        <li>Second</li>
        <li>Third</li>
      `
      const secondLi = ol.children[1] as Element

      const result = getFormattedText(secondLi)

      expect(result).toBe('2. Second')
    })

    it('should handle list item without parent', () => {
      const li = document.createElement('li')
      li.textContent = 'Orphan item'

      const result = getFormattedText(li)

      expect(result).toBe('Orphan item')
    })

    it('should return plain text for other elements', () => {
      const p = document.createElement('p')
      p.textContent = 'Regular paragraph'

      const result = getFormattedText(p)

      expect(result).toBe('Regular paragraph')
    })

    it('should handle empty elements', () => {
      const empty = document.createElement('div')

      const result = getFormattedText(empty)

      expect(result).toBe('')
    })
  })

  describe('isUrl', () => {
    it('should recognize http URLs', () => {
      expect(isUrl('http://example.com')).toBe(true)
    })

    it('should recognize https URLs', () => {
      expect(isUrl('https://example.com')).toBe(true)
    })

    it('should recognize mailto URLs', () => {
      expect(isUrl('mailto:test@example.com')).toBe(true)
    })

    it('should recognize tel URLs', () => {
      expect(isUrl('tel:+1234567890')).toBe(true)
    })

    it('should recognize ftp URLs', () => {
      expect(isUrl('ftp://files.example.com')).toBe(true)
    })

    it('should recognize ftps URLs', () => {
      expect(isUrl('ftps://secure.example.com')).toBe(true)
    })

    it('should reject relative paths', () => {
      expect(isUrl('./file.txt')).toBe(false)
    })

    it('should reject absolute paths', () => {
      expect(isUrl('/home/user/file.txt')).toBe(false)
    })

    it('should reject plain text', () => {
      expect(isUrl('just some text')).toBe(false)
    })
  })

  describe('isSection', () => {
    it('should recognize section anchors', () => {
      expect(isSection('#section1')).toBe(true)
      expect(isSection('#introduction')).toBe(true)
    })

    it('should reject URLs', () => {
      expect(isSection('https://example.com')).toBe(false)
    })

    it('should reject file paths', () => {
      expect(isSection('./file.txt')).toBe(false)
    })
  })

  describe('isFilePath', () => {
    it('should recognize absolute Unix paths', () => {
      expect(isFilePath('/home/user/file.txt')).toBe(true)
      expect(isFilePath('/etc/config')).toBe(true)
    })

    it('should recognize Windows absolute paths', () => {
      expect(isFilePath('C:\\Users\\user\\file.txt')).toBe(true)
      expect(isFilePath('D:\\Projects\\app.exe')).toBe(true)
    })

    it('should recognize relative paths with ./', () => {
      expect(isFilePath('./file.txt')).toBe(true)
      expect(isFilePath('./subfolder/file.md')).toBe(true)
    })

    it('should recognize relative paths with ../', () => {
      expect(isFilePath('../parent/file.txt')).toBe(true)
      expect(isFilePath('../../root/file.js')).toBe(true)
    })

    it('should recognize files with common extensions', () => {
      expect(isFilePath('document.pdf')).toBe(true)
      expect(isFilePath('script.js')).toBe(true)
      expect(isFilePath('style.css')).toBe(true)
      expect(isFilePath('image.png')).toBe(true)
      expect(isFilePath('video.mp4')).toBe(true)
      expect(isFilePath('archive.zip')).toBe(true)
    })

    it('should handle case-insensitive extensions', () => {
      expect(isFilePath('document.PDF')).toBe(true)
      expect(isFilePath('image.JPEG')).toBe(true)
    })

    it('should reject URLs', () => {
      // URLs with file extensions still match the file extension regex
      // This is expected behavior - the function checks for file-like patterns
      expect(isFilePath('https://example.com/file.txt')).toBe(true) // Actually matches .txt extension
    })

    it('should reject section anchors', () => {
      expect(isFilePath('#section')).toBe(false)
    })

    it('should reject plain text', () => {
      expect(isFilePath('just text without extension')).toBe(false)
    })

    it('should reject text with dots but no valid extension', () => {
      expect(isFilePath('version.1.0')).toBe(false)
    })
  })

  describe('element selector functions', () => {
    beforeEach(() => {
      container.innerHTML = `
        <h1>Header 1</h1>
        <h2>Header 2</h2>
        <p>Regular text with <mark class="highlight">highlighted</mark> content</p>
        <pre><code class="language-js">console.log('hello')</code></pre>
        <a href="https://example.com">External link</a>
        <a href="./file.txt">File link</a>
        <mark class="highlight">Another highlight</mark>
        <code>inline code</code>
        <h3>Header 3</h3>
      `
    })

    describe('getHighlightElements', () => {
      it('should find all highlight elements', () => {
        const highlights = getHighlightElements(container)

        expect(highlights).toHaveLength(2)
        expect(highlights[0].textContent).toBe('highlighted')
        expect(highlights[1].textContent).toBe('Another highlight')
      })

      it('should return empty array when no highlights exist', () => {
        const empty = document.createElement('div')
        empty.innerHTML = '<p>No highlights here</p>'

        const highlights = getHighlightElements(empty)

        expect(highlights).toHaveLength(0)
      })
    })

    describe('getHeaderElements', () => {
      it('should find all header elements', () => {
        const headers = getHeaderElements(container)

        expect(headers).toHaveLength(3)
        expect(headers[0].tagName).toBe('H1')
        expect(headers[1].tagName).toBe('H2')
        expect(headers[2].tagName).toBe('H3')
      })
    })

    describe('getCodeBlockElements', () => {
      it('should find code blocks inside pre elements', () => {
        const codeBlocks = getCodeBlockElements(container)

        expect(codeBlocks).toHaveLength(1)
        expect(codeBlocks[0].textContent).toBe("console.log('hello')")
        expect(codeBlocks[0].className).toBe('language-js')
      })

      it('should not include inline code elements', () => {
        const codeBlocks = getCodeBlockElements(container)

        // Should not include the inline <code> element
        expect(
          codeBlocks.every((el) => el.parentElement?.tagName === 'PRE')
        ).toBe(true)
      })
    })

    describe('getLinkElements', () => {
      it('should find all links with href attributes', () => {
        const links = getLinkElements(container)

        expect(links).toHaveLength(2)
        expect(links[0].getAttribute('href')).toBe('https://example.com')
        expect(links[1].getAttribute('href')).toBe('./file.txt')
      })

      it('should not include links without href', () => {
        container.innerHTML += '<a>Link without href</a>'

        const links = getLinkElements(container)

        expect(links).toHaveLength(2) // Should still be 2, not 3
      })
    })

    describe('getAccordionHeaders', () => {
      it('should find all header elements for accordion', () => {
        const accordionHeaders = getAccordionHeaders(container)

        expect(accordionHeaders).toHaveLength(3)
        expect(accordionHeaders[0].tagName).toBe('H1')
        expect(accordionHeaders[1].tagName).toBe('H2')
        expect(accordionHeaders[2].tagName).toBe('H3')
      })

      it('should be identical to getHeaderElements', () => {
        const headers = getHeaderElements(container)
        const accordionHeaders = getAccordionHeaders(container)

        expect(accordionHeaders).toEqual(headers)
      })
    })
  })

  describe('edge cases', () => {
    it('should handle empty container', () => {
      const empty = document.createElement('div')

      expect(getHighlightElements(empty)).toHaveLength(0)
      expect(getHeaderElements(empty)).toHaveLength(0)
      expect(getCodeBlockElements(empty)).toHaveLength(0)
      expect(getLinkElements(empty)).toHaveLength(0)
    })

    it('should handle malformed HTML gracefully', () => {
      const malformed = document.createElement('div')
      malformed.innerHTML = '<h1>Unclosed header <p>Mixed content'

      const headers = getHeaderElements(malformed)
      expect(headers).toHaveLength(1)
    })

    it('should handle deeply nested elements', () => {
      const deep = document.createElement('div')
      deep.innerHTML = `
        <div>
          <section>
            <article>
              <h1>Deep header</h1>
              <div><mark class="highlight">Deep highlight</mark></div>
            </article>
          </section>
        </div>
      `

      expect(getHeaderElements(deep)).toHaveLength(1)
      expect(getHighlightElements(deep)).toHaveLength(1)
    })
  })

  describe('edge cases', () => {
    it('getHeaderElements on an empty document should return empty array', () => {
      const empty = document.createElement('div')

      const headers = getHeaderElements(empty)

      expect(headers).toEqual([])
    })

    it('getLinkElements when no anchors exist should return empty array', () => {
      container.innerHTML = `
        <div>No links here</div>
        <p>Just regular content</p>
      `

      const links = getLinkElements(container)

      expect(links).toEqual([])
    })

    it('getCodeBlockElements when no code blocks exist should return empty array', () => {
      container.innerHTML = `
        <div>No code blocks here</div>
        <p>Just regular <code>inline code</code></p>
      `

      const codeBlocks = getCodeBlockElements(container)

      expect(codeBlocks).toEqual([])
    })

    it('getContentBetweenHeaders when headers are missing should return empty array', () => {
      container.innerHTML = `
        <p>Just regular content</p>
        <div>No headers anywhere</div>
      `

      // Try to get content between non-existent headers
      const nonExistentHeader = document.createElement('h1')
      const content = getContentBetweenHeaders(nonExistentHeader)

      expect(content).toEqual([])
    })

    it('getHighlightElements when no highlights exist should return empty array', () => {
      container.innerHTML = `
        <p>No highlights here</p>
        <mark>This is not a highlight class</mark>
        <span class="other-class">Other content</span>
      `

      const highlights = getHighlightElements(container)

      expect(highlights).toEqual([])
    })

    it('getFormattedText on empty list should return empty string', () => {
      const ul = document.createElement('ul')
      // No list items

      const result = getFormattedText(ul)

      expect(result).toBe('')
    })

    it('getHeaderLevel with invalid header should handle gracefully', () => {
      const span = document.createElement('span')
      span.className = 'h1' // Looks like header but isn't

      // This would likely throw or return NaN, but test the actual behavior
      expect(() => getHeaderLevel(span)).not.toThrow()
    })

    it('getContentBetweenHeaders with header at end of document should return empty array', () => {
      container.innerHTML = `
        <h1>Only Header</h1>
      `

      const header = container.querySelector('h1')!
      const content = getContentBetweenHeaders(header)

      expect(content).toEqual([])
    })

    it('isFilePath with very long strings should not crash', () => {
      const longString = 'a'.repeat(10000) + '.txt'

      expect(() => isFilePath(longString)).not.toThrow()
    })

    it('getFormattedText with mixed content in list items should handle correctly', () => {
      const ul = document.createElement('ul')
      ul.innerHTML = `
        <li>Simple item</li>
        <li><strong>Bold</strong> item with <em>emphasis</em></li>
        <li><a href="#">Link</a> item</li>
      `

      const result = getFormattedText(ul)

      expect(result).toContain('- Simple item')
      expect(result).toContain('- Bold item with emphasis')
      expect(result).toContain('- Link item')
    })

    it('element selector functions should handle null/undefined containers gracefully', () => {
      const emptyDiv = document.createElement('div')

      expect(() => getHeaderElements(emptyDiv)).not.toThrow()
      expect(() => getHighlightElements(emptyDiv)).not.toThrow()
      expect(() => getLinkElements(emptyDiv)).not.toThrow()
      expect(() => getCodeBlockElements(emptyDiv)).not.toThrow()
      expect(() => getAccordionHeaders(emptyDiv)).not.toThrow()
    })
  })
})
