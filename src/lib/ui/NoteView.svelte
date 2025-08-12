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

  import type { AppManagers, AppState, AppActions } from '../app/appCoordinator.svelte';

  const { focusManager, contentManager, editorManager, dialogManager } = getContext<AppManagers>('managers');
  const appState = getContext<AppState>('state');
  const actions = getContext<AppActions>('actions');

  let noteContentElement = $state<HTMLElement | undefined>(undefined);
  let currentTheme = $state<string>('dark_dimmed');
  let themeInitialized = $state<boolean>(false);

  function registerNoteContentElement(element: HTMLElement) {
    focusManager.setNoteContentElement(element);
    return {
      destroy() {
        focusManager.setNoteContentElement(null);
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
    if (contentManager.highlightedContent && focusManager.noteContentElement) {
      setTimeout(() => {
        const blocks = focusManager.noteContentElement!.querySelectorAll('pre code');
        blocks.forEach((block: Element) => {
          hljs.highlightElement(block as HTMLElement);
        });
      }, 0);
    }
  });
</script>

<div class="note-preview" use:themeInitializer>
  {#if appState.selectedNote}
    {#if editorManager.isEditMode}
      <div class="edit-mode">
        <div class="edit-footer">
          <h3>Editing: {appState.selectedNote}</h3>
          <div class="edit-controls">
            <button onclick={actions.saveNote} class="save-btn">Save (Ctrl+S)</button>
            <button onclick={actions.exitEditMode} class="cancel-btn">Cancel (Esc)</button>
          </div>
        </div>
        <CodeMirrorEditor
          bind:value={editorManager.editContent}
          bind:isDirty={editorManager.isDirty}
          filename={appState.selectedNote}
          nearestHeaderText={editorManager.nearestHeaderText}
          onContentChange={editorManager.updateContent}
          onSave={actions.saveNote}
          onExit={actions.exitEditMode}
          onRequestExit={dialogManager.showExitEditDialog}
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
        onfocus={() => focusManager.setNoteContentFocused(true)}
        onblur={() => focusManager.setNoteContentFocused(false)}
        ondblclick={actions.enterEditMode}
      >
        <div class="markdown-body">{@html contentManager.highlightedContent}</div>
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
  .edit-footer {
  position: fixed;       /* Use fixed to position relative to the viewport */
  bottom: 0;             /* Stick to the bottom */
  left: 0;               /* Stretch from left */
  right: 0;              /* Stretch to right */
  z-index: 10;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.8em 1em;
  border-top: 1px solid #181a1f;  /* Border at top instead of bottom */
  background-color: #21252b;
  flex-shrink: 0;
}
  .edit-footer h3 {
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
    background-color: rgba(254, 145, 0, 0.75) !important;
    border-radius: 3px !important;
    padding: 0.1em 0.3em !important;
    font-weight: 500 !important;
    color: #f0f0f0 !important;
    display: inline-block !important;
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
