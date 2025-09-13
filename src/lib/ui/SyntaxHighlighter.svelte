<!--
UI Component - Syntax Highlighter
Non-blocking client-side syntax highlighting using highlight.js.
Processes content after it's rendered to avoid blocking initial load.
-->

<script lang="ts">
  import { onMount, getContext } from 'svelte'
  import hljs from 'highlight.js'
  import type { AppManagers } from '../app/appCoordinator.svelte'

  interface Props {
    content: string
  }

  let { content }: Props = $props()
  const { configManager } = getContext<AppManagers>('managers')
  let containerElement: HTMLDivElement | undefined = $state()

  // Configure highlight.js for automatic language detection
  hljs.configure({
    ignoreUnescapedHTML: true,
  })

  interface HighlightInfo {
    text: string
    positions: { start: number; end: number; originalText: string }[]
  }

  function extractHighlightInfo(codeElement: HTMLElement): HighlightInfo {
    const highlights: { start: number; end: number; originalText: string }[] =
      []
    const walker = document.createTreeWalker(
      codeElement,
      NodeFilter.SHOW_TEXT | NodeFilter.SHOW_ELEMENT,
      null
    )

    let fullText = ''
    let node: Node | null

    while ((node = walker.nextNode())) {
      if (node.nodeType === Node.TEXT_NODE) {
        const textContent = node.textContent || ''
        fullText += textContent
      } else if (node.nodeType === Node.ELEMENT_NODE) {
        const element = node as HTMLElement
        if (element.classList.contains('highlight')) {
          const highlightText = element.textContent || ''
          const start = fullText.length
          const end = start + highlightText.length
          highlights.push({
            start,
            end,
            originalText: highlightText,
          })
          fullText += highlightText
          walker.nextNode()
        }
      }
    }

    return {
      text: codeElement.textContent || '',
      positions: highlights,
    }
  }

  function reapplyHighlights(
    codeElement: HTMLElement,
    highlightInfo: HighlightInfo
  ) {
    if (highlightInfo.positions.length === 0) return

    const walker = document.createTreeWalker(
      codeElement,
      NodeFilter.SHOW_TEXT,
      null
    )

    let textOffset = 0
    const nodesToProcess: {
      node: Text
      highlights: typeof highlightInfo.positions
    }[] = []

    let textNode: Node | null
    while ((textNode = walker.nextNode())) {
      const textContent = textNode.textContent || ''
      const nodeStart = textOffset
      const nodeEnd = textOffset + textContent.length

      const relevantHighlights = highlightInfo.positions
        .filter(
          (highlight) => highlight.start < nodeEnd && highlight.end > nodeStart
        )
        .map((highlight) => ({
          ...highlight,
          start: Math.max(0, highlight.start - nodeStart),
          end: Math.min(textContent.length, highlight.end - nodeStart),
        }))

      if (relevantHighlights.length > 0) {
        nodesToProcess.push({
          node: textNode as Text,
          highlights: relevantHighlights,
        })
      }

      textOffset += textContent.length
    }

    nodesToProcess.reverse().forEach(({ node, highlights }) => {
      const textContent = node.textContent || ''
      const fragments: (string | HTMLElement)[] = []
      let lastEnd = 0

      highlights
        .sort((a, b) => a.start - b.start)
        .forEach((highlight) => {
          if (highlight.start > lastEnd) {
            fragments.push(textContent.slice(lastEnd, highlight.start))
          }

          const mark = document.createElement('mark')
          mark.className = 'highlight'
          mark.textContent = textContent.slice(highlight.start, highlight.end)
          fragments.push(mark)

          lastEnd = highlight.end
        })

      if (lastEnd < textContent.length) {
        fragments.push(textContent.slice(lastEnd))
      }

      const parentElement = node.parentElement
      if (parentElement && fragments.length > 1) {
        fragments.forEach((fragment) => {
          if (typeof fragment === 'string') {
            parentElement.insertBefore(document.createTextNode(fragment), node)
          } else {
            parentElement.insertBefore(fragment, node)
          }
        })
        parentElement.removeChild(node)
      }
    })
  }

  function highlightCodeBlocks() {
    if (!containerElement) return

    const codeBlocks = containerElement.querySelectorAll('pre code:not(.hljs)')

    codeBlocks.forEach((block) => {
      const codeElement = block as HTMLElement

      // Check if already highlighted
      if (codeElement.classList.contains('hljs')) {
        return
      }

      try {
        // Extract highlight info before syntax highlighting
        const highlightInfo = extractHighlightInfo(codeElement)

        // Apply syntax highlighting
        hljs.highlightElement(codeElement)

        // Re-apply search highlights
        reapplyHighlights(codeElement, highlightInfo)
      } catch (error) {
        console.warn(
          'Syntax highlighting failed for block:',
          error,
          codeElement
        )
      }
    })
  }

  // Load theme reactively when it changes
  $effect(() => {
    if (configManager.isThemeInitialized) {
      configManager.loadHighlightJSTheme(configManager.currentCodeTheme)
    }
  })

  // Re-highlight when content changes or component mounts
  $effect(() => {
    // React to content changes by accessing the content prop
    if (containerElement && content) {
      // Small delay to ensure DOM is updated
      setTimeout(highlightCodeBlocks, 10)
    }
  })

  // Also highlight on mount and ensure theme is loaded
  onMount(async () => {
    // Force theme loading if needed
    if (configManager.isThemeInitialized && configManager.currentCodeTheme) {
      await configManager.loadHighlightJSTheme(configManager.currentCodeTheme)
    }

    if (containerElement) {
      setTimeout(highlightCodeBlocks, 50)
    }
  })
</script>

<div bind:this={containerElement} class="syntax-container">
  <!-- eslint-disable-next-line svelte/no-at-html-tags -->
  {@html content}
</div>

<style>
  .syntax-container {
    width: 100%;
    min-height: 100%;
  }

  /* Code block container styling - ensure it shows hljs background */
  .syntax-container :global(pre:has(.hljs)) {
    border-radius: 8px;
    border: 1px solid rgba(128, 128, 128, 0.2);
    margin: 1.5em 0;
    padding: 0;
    overflow: hidden;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
    /* Remove any background so hljs can show through */
    background: none !important;
  }

  /* Code content styling */
  .syntax-container :global(.hljs) {
    padding: 1.25em 1.5em;
    margin: 0;
    overflow-x: auto;
    font-family:
      'JetBrains Mono', 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono',
      Consolas, 'Courier New', monospace;
    font-size: 0.9em;
    line-height: 1.6;
    display: block;
  }

  /* Scrollbar styling for code blocks */
  .syntax-container :global(.hljs::-webkit-scrollbar) {
    height: 8px;
  }

  .syntax-container :global(.hljs::-webkit-scrollbar-track) {
    background: rgba(128, 128, 128, 0.2);
    border-radius: 4px;
  }

  .syntax-container :global(.hljs::-webkit-scrollbar-thumb) {
    background: rgba(128, 128, 128, 0.4);
    border-radius: 4px;
  }

  .syntax-container :global(.hljs::-webkit-scrollbar-thumb:hover) {
    background: rgba(128, 128, 128, 0.6);
  }

  /* Non-highlighted pre elements (fallback) */
  .syntax-container :global(pre code) {
    font-family:
      'JetBrains Mono', 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono',
      Consolas, 'Courier New', monospace;
  }
</style>
