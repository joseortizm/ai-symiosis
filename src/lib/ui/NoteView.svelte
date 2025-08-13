<!--
UI Layer - Note View
Note display component that switches between read and edit modes.
Shows highlighted content or renders the CodeMirror editor.
-->

<script lang="ts">
  import CodeMirrorEditor from './CodeMirrorEditor.svelte'
  import hljs from 'highlight.js'
  import 'highlight.js/styles/atom-one-dark.css'
  import { getContext } from 'svelte'

  import type {
    AppManagers,
    AppState,
    AppActions,
  } from '../app/appCoordinator.svelte'

  const {
    focusManager,
    contentManager,
    editorManager,
    dialogManager,
    themeManager,
  } = getContext<AppManagers>('managers')
  const appState = getContext<AppState>('state')
  const actions = getContext<AppActions>('actions')

  let noteContentElement = $state<HTMLElement | undefined>(undefined)

  function registerNoteContentElement(element: HTMLElement) {
    focusManager.setNoteContentElement(element)
    return {
      destroy() {
        focusManager.setNoteContentElement(null)
      },
    }
  }

  // Use $effect to highlight code blocks when content changes
  $effect(() => {
    // Run after highlightedContent changes and DOM updates
    if (contentManager.highlightedContent && focusManager.noteContentElement) {
      setTimeout(() => {
        const blocks =
          focusManager.noteContentElement!.querySelectorAll('pre code')
        blocks.forEach((block: Element) => {
          hljs.highlightElement(block as HTMLElement)
        })
      }, 0)
    }
  })
</script>

<div class="note-preview" use:themeManager.getThemeInitializer>
  {#if appState.selectedNote}
    {#if editorManager.isEditMode}
      <div class="edit-mode">
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
        <div class="edit-footer">
          <h3>Editing: {appState.selectedNote}</h3>
          <div class="edit-controls">
            <button onclick={actions.saveNote} class="save-btn"
              >Save (Ctrl+S)</button
            >
            <button onclick={actions.exitEditMode} class="cancel-btn"
              >Cancel (Esc)</button
            >
          </div>
        </div>
      </div>
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
          {@html contentManager.highlightedContent}
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
  .edit-mode {
    flex: 1;
    display: flex;
    flex-direction: column;
    background-color: #21252b;
    min-height: 0;
  }
  .edit-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.8em 1em;
    border-top: 1px solid #181a1f;
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
  .save-btn,
  .cancel-btn {
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
