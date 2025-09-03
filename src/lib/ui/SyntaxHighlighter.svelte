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
        // Apply syntax highlighting
        hljs.highlightElement(codeElement)
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
