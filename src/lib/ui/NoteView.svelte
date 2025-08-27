<!--
UI Layer - Note View
Note display component that switches between read and edit modes.
Shows highlighted content or renders the CodeMirror editor.
-->

<script lang="ts">
  import CodeEditor from './CodeEditor.svelte'
  import SyntaxHighlighter from './SyntaxHighlighter.svelte'
  import { getContext } from 'svelte'

  import type {
    AppManagers,
    AppState,
    AppActions,
  } from '../app/appCoordinator.svelte'

  const { focusManager, contentManager, editorManager, dialogManager } =
    getContext<AppManagers>('managers')
  const appState = getContext<AppState>('state')
  const actions = getContext<AppActions>('actions')

  // Theme initialization is now handled by configStateManager
  let noteContentElement = $state<HTMLElement | undefined>(undefined)

  function registerNoteContentElement(element: HTMLElement) {
    focusManager.setNoteContentElement(element)
    return {
      destroy() {
        focusManager.setNoteContentElement(null)
      },
    }
  }
</script>

<div class="note-preview">
  {#if appState.selectedNote}
    {#if editorManager.isEditMode}
      <CodeEditor
        bind:value={editorManager.editContent}
        bind:isDirty={editorManager.isDirty}
        filename={appState.selectedNote}
        nearestHeaderText={editorManager.nearestHeaderText}
        onContentChange={editorManager.updateContent}
        onSave={actions.saveNote}
        onExit={actions.exitEditMode}
        onRequestExit={dialogManager.showExitEditDialog}
      />
    {:else}
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
        <div class="markdown-body">
          <SyntaxHighlighter content={contentManager.highlightedContent} />
        </div>
      </div>
    {/if}
  {:else}
    <div class="no-selection">
      <p>Select a note to preview its content</p>
      <p class="help-text">
        Press Enter to edit, Ctrl+F to show in enclosing folder.
      </p>
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
  .note-content {
    flex: 1;
    padding: 0;
    overflow-y: auto;
    overflow-x: hidden;
    outline: none;
    border: 2px solid transparent;
    transition: border-color 0.2s ease;
    background-color: transparent;
  }
  .note-content:focus {
    border-color: #61afef;
  }
  .markdown-body {
    padding-top: 2em;
    padding-left: max(1em, calc((100vw - 65ch) / 2));
    padding-right: max(1em, calc((100vw - 65ch) / 2));
    min-height: 100%;
    width: 100%;
    box-sizing: border-box;
  }
  @media (min-width: 768px) {
    .markdown-body {
      padding-left: max(1.5em, calc((100vw - 70ch) / 2));
      padding-right: max(1.5em, calc((100vw - 70ch) / 2));
    }
  }
  @media (min-width: 1024px) {
    .markdown-body {
      padding-left: max(2em, calc((100vw - 75ch) / 2));
      padding-right: max(2em, calc((100vw - 75ch) / 2));
    }
  }
  :global(.highlight) {
    background-color: rgba(254, 145, 0, 0.75) !important;
    border-radius: 3px !important;
    padding: 0.1em 0.3em !important;
    font-weight: 500 !important;
    color: #f0f0f0 !important;
    display: inline-block !important;
  }
  :global(.highlight-current) {
    background-color: rgba(254, 145, 0, 1) !important;
    border: 2px solid #fe9100 !important;
    box-shadow: 0 0 8px rgba(254, 145, 0, 0.6) !important;
    transform: scale(1.05) !important;
  }
  :global(.header-current) {
    background-color: rgba(97, 175, 239, 0.2);
    border-left: 4px solid #61afef;
    padding-left: 0.5em;
    padding-top: 0.2em;
    padding-bottom: 0.2em;
    margin-left: -0.5em;
    border-radius: 3px;
  }
  :global(.header-expanded) {
    opacity: 1;
    transition: opacity 0.3s ease;
  }
  :global(.header-collapsed) {
    opacity: 0.5;
    font-size: 0.9em;
    margin: 0.2em 0;
    transition: all 0.3s ease;
  }
  :global(.content-collapsed) {
    display: none;
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
