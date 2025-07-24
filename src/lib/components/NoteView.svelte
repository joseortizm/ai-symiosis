<script>
  import Editor from './Editor.svelte';

  export let selectedNote;
  export let isEditMode;
  export let editContent;
  export let highlightedContent;
  export let onSave;
  export let onExitEditMode;
  export let onEnterEditMode;
  export let noteContentElement = null;
  export let isNoteContentFocused = false;
</script>

<div class="note-preview">
  {#if selectedNote}
    {#if isEditMode}
      <div class="edit-mode">
        <div class="edit-header">
          <h3>Editing: {selectedNote}</h3>
          <div class="edit-controls">
            <button on:click={onSave} class="save-btn">Save (Ctrl+S)</button>
            <button on:click={onExitEditMode} class="cancel-btn">Cancel (Esc)</button>
          </div>
        </div>
        <Editor
          bind:value={editContent}
          filename={selectedNote}
          onSave={onSave}
          onExit={onExitEditMode}
        />
      </div>
    {:else}
      <div
        class="note-content"
        bind:this={noteContentElement}
        tabindex="0"
        on:focus={() => isNoteContentFocused = true}
        on:blur={() => isNoteContentFocused = false}
        on:dblclick={onEnterEditMode}
      >
        <div class="note-text">{@html highlightedContent}</div>
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
}
.edit-mode {
  flex: 1;
  display: flex;
  flex-direction: column;
  background-color: #32302f;
}
.edit-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.8em 1em;
  border-bottom: 1px solid #504945;
  background-color: #3c3836;
  flex-shrink: 0;
}
.edit-header h3 {
  margin: 0;
  color: #fe8019;
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
  background-color: #b8bb26;
  color: #282828;
}
.save-btn:hover {
  background-color: #98971a;
}
.cancel-btn {
  background-color: #504945;
  color: #ebdbb2;
}
.cancel-btn:hover {
  background-color: #665c54;
}
.note-content {
  flex: 1;
  padding: 1em;
  overflow-y: auto;
  transform: translateZ(0);
  will-change: scroll-position;
  outline: none;
  border: 2px solid transparent;
  transition: border-color 0.2s ease;
  background-color: #32302f;
}
.note-content:focus {
  border-color: #83a598;
}
.note-text {
  color: #fbf1c7;
  font-family: 'Inter', sans-serif;
  font-size: 0.95em;
  line-height: 1.6;
  white-space: normal;
}
.note-text h1,
.note-text h2,
.note-text h3,
.note-text h4 {
  margin: 1em 0 0.5em;
  font-weight: bold;
  color: #fabd2f;
}
.note-text h1 { font-size: 1.5em; }
.note-text h2 { font-size: 1.3em; }
.note-text h3 { font-size: 1.15em; }
.note-text p {
  margin: 0.5em 0;
}
.note-text a {
  color: #83a598;
  text-decoration: underline;
  word-break: break-word;
}
.note-text a:hover {
  color: #b8bb26;
}
.note-text code {
  background: #3c3836;
  padding: 0.2em 0.4em;
  border-radius: 4px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.95em;
  color: #d3869b;
}
.note-text pre {
  background: #3c3836;
  padding: 1em;
  overflow-x: auto;
  border-radius: 6px;
  font-family: 'JetBrains Mono', monospace;
  color: #fbf1c7;
  margin: 1em 0;
  font-size: 0.9em;
}
.note-text ul,
.note-text ol {
  margin: 0.5em 0 0.5em 1.2em;
  padding-left: 1em;
}
.note-text blockquote {
  margin: 1em 0;
  padding-left: 1em;
  border-left: 3px solid #504945;
  color: #d5c4a1;
  font-style: italic;
}
.note-text hr {
  border: none;
  border-top: 1px solid #504945;
  margin: 1em 0;
}
.highlight {
  background-color: #fabd2f;
  color: #282828;
  padding: 0.1em 0.2em;
  border-radius: 3px;
  font-weight: 500;
}
.no-selection {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #928374;
  font-style: italic;
  text-align: center;
}
.help-text {
  font-size: 0.9em;
  margin-top: 0.5em;
  color: #665c54;
}
.note-content::-webkit-scrollbar {
  width: 8px;
}
.note-content::-webkit-scrollbar-track {
  background: #282828;
}
.note-content::-webkit-scrollbar-thumb {
  background: #504945;
  border-radius: 4px;
}
.note-content::-webkit-scrollbar-thumb:hover {
  background: #665c54;
}
</style>