<script lang="ts">
  import Editor from './Editor.svelte';
  import hljs from 'highlight.js';
  import 'highlight.js/styles/atom-one-dark.css';
  import { getAppContext } from '../context/app.svelte';

  const context = getAppContext();

  let noteContentElement = $state<HTMLElement | undefined>(undefined);

  $effect(() => {
    if (noteContentElement) {
      context.focusManager.setNoteContentElement(noteContentElement);
    }
  });

  // Use $effect to highlight code blocks when content changes
  $effect(() => {
    // Run after highlightedContent changes and DOM updates
    if (context.contentManager.highlightedContent && context.focusManager.noteContentElement) {
      setTimeout(() => {
        const blocks = context.focusManager.noteContentElement!.querySelectorAll('pre code');
        blocks.forEach((block: Element) => {
          hljs.highlightElement(block as HTMLElement);
        });
      }, 0);
    }
  });
</script>

<div class="note-preview">
  {#if context.state.selectedNote}
    {#if context.editorManager.isEditMode}
      <div class="edit-mode">
        <div class="edit-header">
          <h3>Editing: {context.state.selectedNote}</h3>
          <div class="edit-controls">
            <button onclick={context.saveNote} class="save-btn">Save (Ctrl+S)</button>
            <button onclick={context.exitEditMode} class="cancel-btn">Cancel (Esc)</button>
          </div>
        </div>
        <Editor
          value={context.editorManager.editContent}
          isDirty={context.editorManager.isDirty}
          filename={context.state.selectedNote}
          nearestHeaderText={context.editorManager.nearestHeaderText}
          onContentChange={context.editorManager.updateContent}
          onSave={context.saveNote}
          onExit={context.exitEditMode}
          onRequestExit={context.showExitEditDialog}
        />
      </div>
    {:else}
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="note-content"
        bind:this={noteContentElement}
        tabindex="-1"
        onfocus={() => context.focusManager.setNoteContentFocused(true)}
        onblur={() => context.focusManager.setNoteContentFocused(false)}
        ondblclick={context.enterEditMode}
      >
        <div class="note-text">{@html context.contentManager.highlightedContent}</div>
      </div>
    {/if}
  {:else}
    <div class="no-selection">
      <p>Select a note to preview its content</p>
      <p class="help-text">Press Enter to edit, E to edit when focused, Ctrl+O to open externally</p>
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
    background-color: #282c34;
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
    padding: 0em 2em 2em 2em;
    overflow-y: auto;
    outline: none;
    border: 2px solid transparent;
    transition: border-color 0.2s ease;
  }
  .note-content:focus {
    border-color: #61afef;
  }
  .note-text {
    color: #abb2bf;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol";
    font-size: 1em;
    line-height: 1.7;
    max-width: 800px;
    margin: 0 auto;
  }
  .note-text :global(h1),
  .note-text :global(h2),
  .note-text :global(h3),
  .note-text :global(h4),
  .note-text :global(h5),
  .note-text :global(h6) {
    color: #f0f0f0;
    margin-top: 1.5em;
    margin-bottom: 0.5em;
    font-weight: 600;
  }
  .note-text :global(h1) { font-size: 2em; }
  .note-text :global(h2) { font-size: 1.5em; }
  .note-text :global(h3) { font-size: 1.25em; }

  .note-text :global(p) {
    margin-bottom: 1em;
  }

  .note-text :global(a) {
    color: #61afef;
    text-decoration: none;
    word-break: break-word;
  }
  .note-text :global(a:hover) {
    text-decoration: underline;
  }

  .note-text :global(code) {
    font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, Courier, monospace;
    font-size: 0.9em;
  }

  .note-text :global(pre) {
    margin: 1.5em 0;
    border-radius: 6px;
    font-size: 0.9em;
  }

  .note-text :global(pre code) {
    padding: 1em;
    display: block;
    overflow-x: auto;
  }

  .note-text :global(ul),
  .note-text :global(ol) {
    margin-bottom: 1em;
    padding-left: 2em;
  }

  .note-text :global(blockquote) {
    margin: 1.5em 0;
    padding-left: 1.5em;
    border-left: 3px solid #5c6370;
    color: #9ca3af;
    font-style: italic;
  }

  .note-text :global(hr) {
    border: none;
    border-top: 1px solid #3a3f4b;
    margin: 2em 0;
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
