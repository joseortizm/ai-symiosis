<!--
UI Layer - Note View
Note display component that switches between read and edit modes.
Shows highlighted content or renders the CodeMirror editor.
-->

<script lang="ts">
  import CodeMirrorEditor from './CodeMirrorEditor.svelte';
  import hljs from 'highlight.js';
  import 'highlight.js/styles/atom-one-dark.css';
  import { getContext } from 'svelte';
  import { configService } from '../services/configService.svelte';

  const { appCoordinator } = getContext('managers') as any;

  let noteContentElement = $state<HTMLElement | undefined>(undefined);
  let currentTheme = $state<string>('dark_dimmed');
  let themeInitialized = $state<boolean>(false);

  function registerNoteContentElement(element: HTMLElement) {
    appCoordinator.context.focusManager.setNoteContentElement(element);
    return {
      destroy() {
        appCoordinator.context.focusManager.setNoteContentElement(null);
      }
    };
  }

  async function loadTheme(theme: string) {
    const existingLink = document.head.querySelector('link[data-markdown-theme]');
    if (existingLink) {
      existingLink.remove();
    }

    const link = document.createElement('link');
    link.rel = 'stylesheet';
    link.href = `/css/${theme}.css`;
    link.setAttribute('data-markdown-theme', theme);

    document.head.appendChild(link);

    return new Promise<void>((resolve) => {
      link.onload = () => {
        resolve();
      };
      link.onerror = () => {
        resolve();
      };
    });
  }

  async function initializeTheme() {
    try {
      const theme = await configService.getMarkdownTheme();

      if (theme !== currentTheme || !themeInitialized) {
        currentTheme = theme;
        await loadTheme(theme);
        themeInitialized = true;
      }
    } catch (e) {
      console.error('Failed to load markdown theme:', e);
      if (currentTheme !== 'dark_dimmed' || !themeInitialized) {
        currentTheme = 'dark_dimmed';
        await loadTheme('dark_dimmed');
        themeInitialized = true;
      }
    }
  }

  function themeInitializer(element: HTMLElement) {
    initializeTheme();
    return {
      destroy() {
        // Cleanup if needed
      }
    };
  }

  // Watch for config changes and reload theme
  $effect(() => {
    const lastSaved = configService.lastSaved;
    if (lastSaved > 0 && themeInitialized) {
      initializeTheme();
    }
  });

  // Use $effect to highlight code blocks when content changes
  $effect(() => {
    // Run after highlightedContent changes and DOM updates
    if (appCoordinator.context.contentManager.highlightedContent && appCoordinator.context.focusManager.noteContentElement) {
      setTimeout(() => {
        const blocks = appCoordinator.context.focusManager.noteContentElement!.querySelectorAll('pre code');
        blocks.forEach((block: Element) => {
          hljs.highlightElement(block as HTMLElement);
        });
      }, 0);
    }
  });
</script>

<div class="note-preview" use:themeInitializer>
  {#if appCoordinator.context.state.selectedNote}
    {#if appCoordinator.context.editorManager.isEditMode}
      <div class="edit-mode">
        <div class="edit-header">
          <h3>Editing: {appCoordinator.context.state.selectedNote}</h3>
          <div class="edit-controls">
            <button onclick={appCoordinator.context.saveNote} class="save-btn">Save (Ctrl+S)</button>
            <button onclick={appCoordinator.context.exitEditMode} class="cancel-btn">Cancel (Esc)</button>
          </div>
        </div>
        <CodeMirrorEditor
          bind:value={appCoordinator.context.editorManager.editContent}
          bind:isDirty={appCoordinator.context.editorManager.isDirty}
          filename={appCoordinator.context.state.selectedNote}
          nearestHeaderText={appCoordinator.context.editorManager.nearestHeaderText}
          onContentChange={appCoordinator.context.editorManager.updateContent}
          onSave={appCoordinator.context.saveNote}
          onExit={appCoordinator.context.exitEditMode}
          onRequestExit={appCoordinator.context.showExitEditDialog}
        />
      </div>
    {:else}
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="note-content"
        bind:this={noteContentElement}
        use:registerNoteContentElement
        tabindex="-1"
        onfocus={() => appCoordinator.context.focusManager.setNoteContentFocused(true)}
        onblur={() => appCoordinator.context.focusManager.setNoteContentFocused(false)}
        ondblclick={appCoordinator.context.enterEditMode}
      >
        <div class="markdown-body">{@html appCoordinator.context.contentManager.highlightedContent}</div>
      </div>
    {/if}
  {:else}
    <div class="no-selection">
      <p>Select a note to preview its content</p>
      <p class="help-text">Press Enter to edit, Ctrl+F to show in enclosing folder.</p>
    </div>
  {/if}
</div>

<style>
  .note-preview {
    flex: 1.2;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-height: 0;
  }
  .edit-mode {
    flex: 1;
    display: flex;
    flex-direction: column;
    background-color: #21252b;
  }
  .edit-header {
    position: sticky;
    top: 0;
    z-index: 10;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.8em 1em;
    border-bottom: 1px solid #181a1f;
    background-color: #21252b;
    flex-shrink: 0;
  }
  .edit-header h3 {
    margin: 0;
    color: #61afef;
    font-size: 1.1em;
    font-weight: 500;
  }
  .edit-controls {
    display: flex;
    gap: 0.5em;
  }
  .save-btn, .cancel-btn {
    padding: 0.4em 0.8em;
    border: none;
    border-radius: 4px;
    font-size: 0.9em;
    cursor: pointer;
    transition: background-color 0.2s ease;
  }
  .save-btn {
    background-color: #98c379;
    color: #282c34;
  }
  .save-btn:hover {
    background-color: #a7d78b;
  }
  .cancel-btn {
    background-color: #3a3f4b;
    color: #abb2bf;
  }
  .cancel-btn:hover {
    background-color: #4b5263;
  }
  .note-content {
    flex: 1;
    padding: 0;
    overflow-y: auto;
    outline: none;
    border: 2px solid transparent;
    transition: border-color 0.2s ease;
    background-color: transparent;
  }
  .note-content:focus {
    border-color: #61afef;
  }
  .markdown-body {
    max-width: 800px;
    margin: 0 auto;
    padding: 1em;
    min-height: 100%;
    width: 100%;
  }

  :global(.highlight) {
    background-color: rgba(254, 145, 0, 0.45);
    border-radius: 3px;
    padding: 0.1em 0.3em;
    font-weight: 500;
    color: #f0f0f0;
    display: inline-block;
  }

  .no-selection {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: #5c6370;
    font-style: italic;
    text-align: center;
  }

  .help-text {
    font-size: 0.9em;
    margin-top: 0.5em;
    color: #4b5263;
  }

  .note-content::-webkit-scrollbar {
    width: 10px;
  }

  .note-content::-webkit-scrollbar-track {
    background: #21252b;
  }

  .note-content::-webkit-scrollbar-thumb {
    background: #4b5263;
    border-radius: 5px;
  }

  .note-content::-webkit-scrollbar-thumb:hover {
    background: #5c6370;
  }
</style>
